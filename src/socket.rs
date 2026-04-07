use std::collections::HashSet;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::sync::Mutex;
use std::time::SystemTime;

use bytes::Bytes;
use tokio::net::UdpSocket;

use crate::{
    raw, Datagram, Interface, Membership, MulticastConfig, MulticastError,
    MulticastSocketBuilder, RecvMeta, Result,
};

#[derive(Debug)]
pub struct MulticastSocket {
    socket: UdpSocket,
    config: MulticastConfig,
    memberships: Arc<Mutex<HashSet<Membership>>>,
}

impl MulticastSocket {
    pub fn builder() -> MulticastSocketBuilder {
        MulticastSocketBuilder::new()
    }

    pub(crate) async fn from_config(config: MulticastConfig) -> Result<Self> {
        let std_socket = raw::build_std_socket(&config)?;
        let socket = UdpSocket::from_std(std_socket)?;
        let memberships = config.memberships.iter().cloned().collect();

        Ok(Self {
            socket,
            config,
            memberships: Arc::new(Mutex::new(memberships)),
        })
    }

    pub fn local_addr(&self) -> std::io::Result<SocketAddr> {
        self.socket.local_addr()
    }

    pub fn config(&self) -> &MulticastConfig {
        &self.config
    }

    pub fn memberships(&self) -> HashSet<Membership> {
        self.memberships.lock().unwrap().clone()
    }

    pub async fn join(&self, membership: Membership) -> Result<()> {
        let mut state = self.memberships.lock().unwrap();
        if state.contains(&membership) {
            return Ok(());
        }

        self.apply_join(&membership)?;
        state.insert(membership);
        Ok(())
    }

    pub async fn leave(&self, membership: &Membership) -> Result<()> {
        let mut state = self.memberships.lock().unwrap();
        if !state.contains(membership) {
            return Ok(());
        }

        self.apply_leave(membership)?;
        state.remove(membership);
        Ok(())
    }

    pub async fn recv_from(&self, buf: &mut [u8]) -> std::io::Result<(usize, SocketAddr)> {
        self.socket.recv_from(buf).await
    }

    pub async fn recv(&self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.socket.recv(buf).await
    }

    pub async fn recv_datagram(&self, buf_size: usize) -> std::io::Result<Datagram> {
        let mut buf = vec![0_u8; buf_size];
        let (size, peer) = self.socket.recv_from(&mut buf).await?;
        buf.truncate(size);

        Ok(Datagram {
            payload: Bytes::from(buf),
            meta: RecvMeta {
                peer,
                local_addr: self.local_addr().ok(),
                interface: None,
                group: self.primary_group(),
                timestamp: Some(SystemTime::now()),
            },
        })
    }

    pub async fn send_to(&self, payload: &[u8], target: SocketAddr) -> std::io::Result<usize> {
        self.socket.send_to(payload, target).await
    }

    pub async fn send_to_group(&self, payload: &[u8]) -> Result<usize> {
        let group = self
            .primary_group()
            .ok_or(MulticastError::NoMembershipsConfigured)?;
        let target = match group {
            IpAddr::V4(group) => SocketAddr::from((group, self.config.bind_addr.port())),
            IpAddr::V6(group) => {
                let scope_id = self.ipv6_scope_id();
                raw::group_as_v6_socket(group, self.config.bind_addr.port(), scope_id).into()
            }
        };
        Ok(self.socket.send_to(payload, target).await?)
    }

    fn ipv6_scope_id(&self) -> u32 {
        match self.config.outbound_interface.as_ref() {
            Some(Interface::V6(index)) => *index,
            _ => match self.config.inbound_interface.as_ref() {
                Some(Interface::V6(index)) => *index,
                _ => 0,
            },
        }
    }

    fn primary_group(&self) -> Option<IpAddr> {
        self.config.memberships.first().map(Membership::group)
    }

    fn apply_join(&self, membership: &Membership) -> Result<()> {
        match membership {
            Membership::AnySource {
                group: IpAddr::V4(group),
            } => {
                let interface = match &self.config.inbound_interface {
                    Some(crate::Interface::V4(addr)) => *addr,
                    _ => Ipv4Addr::UNSPECIFIED,
                };
                self.socket.join_multicast_v4(*group, interface)?;
                Ok(())
            }
            Membership::AnySource {
                group: IpAddr::V6(group),
            } => {
                let index = match &self.config.inbound_interface {
                    Some(crate::Interface::V6(index)) => *index,
                    _ => 0,
                };
                self.socket.join_multicast_v6(group, index)?;
                Ok(())
            }
            Membership::SourceSpecific { .. } => Err(MulticastError::UnsupportedOption(
                "dynamic source-specific membership",
            )),
        }
    }

    fn apply_leave(&self, membership: &Membership) -> Result<()> {
        match membership {
            Membership::AnySource {
                group: IpAddr::V4(group),
            } => {
                let interface = match &self.config.inbound_interface {
                    Some(crate::Interface::V4(addr)) => *addr,
                    _ => Ipv4Addr::UNSPECIFIED,
                };
                self.socket.leave_multicast_v4(*group, interface)?;
                Ok(())
            }
            Membership::AnySource {
                group: IpAddr::V6(group),
            } => {
                let index = match &self.config.inbound_interface {
                    Some(crate::Interface::V6(index)) => *index,
                    _ => 0,
                };
                self.socket.leave_multicast_v6(group, index)?;
                Ok(())
            }
            Membership::SourceSpecific { .. } => Err(MulticastError::UnsupportedOption(
                "dynamic source-specific membership",
            )),
        }
    }
}
