use std::io;
use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr, UdpSocket};
use std::time::Duration;

use socket2::{Domain, Protocol, Socket, Type};

use crate::{raw, sys, Interface, Membership};

#[derive(Debug, Clone)]
pub struct MulticastDiagnosticConfig {
    pub ipv4_group: Ipv4Addr,
    pub ipv6_group: Ipv6Addr,
    pub ipv6_interface: Option<u32>,
    pub timeout: Duration,
}

impl Default for MulticastDiagnosticConfig {
    fn default() -> Self {
        Self {
            ipv4_group: Ipv4Addr::new(239, 1, 1, 250),
            ipv6_group: Ipv6Addr::new(0xff02, 0, 0, 0, 0, 0, 0, 0x114),
            ipv6_interface: sys::loopback_interface_v6(),
            timeout: Duration::from_millis(500),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ProbeStages {
    pub socket_created: bool,
    pub bound: bool,
    pub joined: bool,
    pub loopback_sent: bool,
    pub loopback_received: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProbeErrorKind {
    Unavailable,
    SocketCreate,
    Bind,
    Join,
    Send,
    Receive,
    InvalidData,
    Other,
}

#[derive(Debug, Clone)]
pub struct ProbeResult {
    pub label: &'static str,
    pub supported: bool,
    pub stages: ProbeStages,
    pub error_kind: Option<ProbeErrorKind>,
    pub details: Option<String>,
    pub error: Option<String>,
}

impl ProbeResult {
    fn success(label: &'static str, stages: ProbeStages, details: impl Into<String>) -> Self {
        Self {
            label,
            supported: true,
            stages,
            error_kind: None,
            details: Some(details.into()),
            error: None,
        }
    }

    fn failure(
        label: &'static str,
        stages: ProbeStages,
        error_kind: ProbeErrorKind,
        error: impl Into<String>,
        details: Option<String>,
    ) -> Self {
        Self {
            label,
            supported: false,
            stages,
            error_kind: Some(error_kind),
            details,
            error: Some(error.into()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct MulticastDiagnostics {
    pub ipv4: ProbeResult,
    pub ipv6: ProbeResult,
}

impl MulticastDiagnostics {
    pub fn supported(&self) -> bool {
        self.ipv4.supported || self.ipv6.supported
    }
}

pub fn diagnose_multicast() -> MulticastDiagnostics {
    diagnose_multicast_with_config(&MulticastDiagnosticConfig::default())
}

pub fn diagnose_multicast_with_config(config: &MulticastDiagnosticConfig) -> MulticastDiagnostics {
    MulticastDiagnostics {
        ipv4: diagnose_ipv4(config),
        ipv6: diagnose_ipv6(config),
    }
}

fn diagnose_ipv4(config: &MulticastDiagnosticConfig) -> ProbeResult {
    let label = "ipv4";
    let group = config.ipv4_group;

    match try_ipv4_probe(group, config.timeout) {
        Ok((receiver_addr, sender_addr)) => ProbeResult::success(
            label,
            ProbeStages {
                socket_created: true,
                bound: true,
                joined: true,
                loopback_sent: true,
                loopback_received: true,
            },
            format!(
                "bound receiver {receiver_addr}, sent from {sender_addr}, joined {group}, timeout {:?}",
                config.timeout
            ),
        ),
        Err((stages, kind, err, details)) => {
            ProbeResult::failure(label, stages, kind, err.to_string(), details)
        }
    }
}

fn diagnose_ipv6(config: &MulticastDiagnosticConfig) -> ProbeResult {
    let label = "ipv6";
    let group = config.ipv6_group;
    let Some(ifindex) = config.ipv6_interface else {
        return ProbeResult::failure(
            label,
            ProbeStages::default(),
            ProbeErrorKind::Unavailable,
            "no loopback IPv6 interface index available",
            None,
        );
    };

    match try_ipv6_probe(group, ifindex, config.timeout) {
        Ok((receiver_addr, sender_addr)) => ProbeResult::success(
            label,
            ProbeStages {
                socket_created: true,
                bound: true,
                joined: true,
                loopback_sent: true,
                loopback_received: true,
            },
            format!(
                "bound receiver {receiver_addr}, sent from {sender_addr}, joined {group}%{ifindex}, timeout {:?}",
                config.timeout
            ),
        ),
        Err((stages, kind, err, details)) => {
            ProbeResult::failure(label, stages, kind, err.to_string(), details)
        }
    }
}

fn try_ipv4_probe(
    group: Ipv4Addr,
    timeout: Duration,
) -> std::result::Result<
    (SocketAddr, SocketAddr),
    (ProbeStages, ProbeErrorKind, io::Error, Option<String>),
> {
    let mut stages = ProbeStages::default();

    let receiver = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))
        .map_err(|err| (stages.clone(), ProbeErrorKind::SocketCreate, err, None))?;
    stages.socket_created = true;
    receiver
        .set_reuse_address(true)
        .map_err(|err| (stages.clone(), ProbeErrorKind::Other, err, None))?;
    receiver
        .bind(&SocketAddr::from((Ipv4Addr::UNSPECIFIED, 0)).into())
        .map_err(|err| (stages.clone(), ProbeErrorKind::Bind, err, None))?;
    stages.bound = true;
    receiver
        .join_multicast_v4(&group, &Ipv4Addr::UNSPECIFIED)
        .map_err(|err| (stages.clone(), ProbeErrorKind::Join, err, None))?;
    stages.joined = true;

    let receiver: UdpSocket = receiver.into();
    receiver
        .set_read_timeout(Some(timeout))
        .map_err(|err| (stages.clone(), ProbeErrorKind::Other, err, None))?;
    let receiver_addr = receiver
        .local_addr()
        .map_err(|err| (stages.clone(), ProbeErrorKind::Other, err, None))?;

    let sender = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))
        .map_err(|err| {
            (
                stages.clone(),
                ProbeErrorKind::SocketCreate,
                err,
                Some(format!("receiver bound at {receiver_addr}")),
            )
        })?;
    sender
        .bind(&SocketAddr::from((Ipv4Addr::UNSPECIFIED, 0)).into())
        .map_err(|err| {
            (
                stages.clone(),
                ProbeErrorKind::Bind,
                err,
                Some(format!("receiver bound at {receiver_addr}")),
            )
        })?;
    sender
        .set_multicast_loop_v4(true)
        .map_err(|err| {
            (
                stages.clone(),
                ProbeErrorKind::Other,
                err,
                Some(format!("receiver bound at {receiver_addr}")),
            )
        })?;

    let sender: UdpSocket = sender.into();
    let sender_addr = sender
        .local_addr()
        .map_err(|err| {
            (
                stages.clone(),
                ProbeErrorKind::Other,
                err,
                Some(format!("receiver bound at {receiver_addr}")),
            )
        })?;
    sender
        .send_to(b"diag", SocketAddr::from((group, receiver_addr.port())))
        .map_err(|err| {
            (
                stages.clone(),
                ProbeErrorKind::Send,
                err,
                Some(format!("receiver bound at {receiver_addr}")),
            )
        })?;
    stages.loopback_sent = true;

    let mut buf = [0_u8; 16];
    let (n, _) = receiver.recv_from(&mut buf).map_err(|err| {
        (
            stages.clone(),
            ProbeErrorKind::Receive,
            err,
            Some(format!(
                "receiver bound at {receiver_addr}, sender bound at {sender_addr}"
            )),
        )
    })?;
    if &buf[..n] != b"diag" {
        return Err((
            stages,
            ProbeErrorKind::InvalidData,
            io::Error::new(io::ErrorKind::InvalidData, "unexpected loopback payload"),
            Some(format!(
                "receiver bound at {receiver_addr}, sender bound at {sender_addr}"
            )),
        ));
    }

    Ok((
        receiver_addr,
        sender_addr,
    ))
}

fn try_ipv6_probe(
    group: Ipv6Addr,
    ifindex: u32,
    timeout: Duration,
) -> std::result::Result<
    (SocketAddr, SocketAddr),
    (ProbeStages, ProbeErrorKind, io::Error, Option<String>),
> {
    let mut stages = ProbeStages::default();

    let receiver = Socket::new(Domain::IPV6, Type::DGRAM, Some(Protocol::UDP))
        .map_err(|err| (stages.clone(), ProbeErrorKind::SocketCreate, err, None))?;
    stages.socket_created = true;
    receiver
        .set_reuse_address(true)
        .map_err(|err| (stages.clone(), ProbeErrorKind::Other, err, None))?;
    receiver
        .bind(&raw::group_as_v6_socket(Ipv6Addr::UNSPECIFIED, 0, 0).into())
        .map_err(|err| (stages.clone(), ProbeErrorKind::Bind, err, None))?;
    stages.bound = true;
    sys::join_membership(
        &receiver,
        &Membership::any_source(group.into()),
        Some(&Interface::V6(ifindex)),
    )
    .map_err(|err| (stages.clone(), ProbeErrorKind::Join, map_multicast_error(err), None))?;
    stages.joined = true;

    let receiver: UdpSocket = receiver.into();
    receiver
        .set_read_timeout(Some(timeout))
        .map_err(|err| (stages.clone(), ProbeErrorKind::Other, err, None))?;
    let receiver_addr = receiver
        .local_addr()
        .map_err(|err| (stages.clone(), ProbeErrorKind::Other, err, None))?;

    let sender = Socket::new(Domain::IPV6, Type::DGRAM, Some(Protocol::UDP))
        .map_err(|err| {
            (
                stages.clone(),
                ProbeErrorKind::SocketCreate,
                err,
                Some(format!("receiver bound at {receiver_addr}")),
            )
        })?;
    sender
        .bind(&raw::group_as_v6_socket(Ipv6Addr::UNSPECIFIED, 0, 0).into())
        .map_err(|err| {
            (
                stages.clone(),
                ProbeErrorKind::Bind,
                err,
                Some(format!("receiver bound at {receiver_addr}")),
            )
        })?;
    sender
        .set_multicast_if_v6(ifindex)
        .map_err(|err| {
            (
                stages.clone(),
                ProbeErrorKind::Other,
                err,
                Some(format!("receiver bound at {receiver_addr}")),
            )
        })?;
    sender
        .set_multicast_loop_v6(true)
        .map_err(|err| {
            (
                stages.clone(),
                ProbeErrorKind::Other,
                err,
                Some(format!("receiver bound at {receiver_addr}")),
            )
        })?;

    let sender: UdpSocket = sender.into();
    let sender_addr = sender
        .local_addr()
        .map_err(|err| {
            (
                stages.clone(),
                ProbeErrorKind::Other,
                err,
                Some(format!("receiver bound at {receiver_addr}")),
            )
        })?;
    sender
        .send_to(b"diag", raw::group_as_v6_socket(group, receiver_addr.port(), ifindex))
        .map_err(|err| {
            (
                stages.clone(),
                ProbeErrorKind::Send,
                err,
                Some(format!(
                    "receiver bound at {receiver_addr}, sender bound at {sender_addr}"
                )),
            )
        })?;
    stages.loopback_sent = true;

    let mut buf = [0_u8; 16];
    let (n, _) = receiver.recv_from(&mut buf).map_err(|err| {
        (
            stages.clone(),
            ProbeErrorKind::Receive,
            err,
            Some(format!(
                "receiver bound at {receiver_addr}, sender bound at {sender_addr}, ifindex {ifindex}"
            )),
        )
    })?;
    if &buf[..n] != b"diag" {
        return Err((
            stages,
            ProbeErrorKind::InvalidData,
            io::Error::new(io::ErrorKind::InvalidData, "unexpected loopback payload"),
            Some(format!(
                "receiver bound at {receiver_addr}, sender bound at {sender_addr}, ifindex {ifindex}"
            )),
        ));
    }

    Ok((
        receiver_addr,
        sender_addr,
    ))
}

fn map_multicast_error(err: crate::MulticastError) -> io::Error {
    match err {
        crate::MulticastError::Io(source) => source,
        crate::MulticastError::BindFailed { source, .. } => source,
        other => io::Error::other(other.to_string()),
    }
}
