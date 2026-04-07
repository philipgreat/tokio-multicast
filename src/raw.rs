use std::net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV6, UdpSocket as StdUdpSocket};

use socket2::{Domain, Protocol, Socket, Type};

use crate::{Interface, Membership, MulticastConfig, MulticastError, Result};

pub(crate) fn build_std_socket(config: &MulticastConfig) -> Result<StdUdpSocket> {
    let domain = match config.bind_addr {
        SocketAddr::V4(_) => Domain::IPV4,
        SocketAddr::V6(_) => Domain::IPV6,
    };

    let socket = Socket::new(domain, Type::DGRAM, Some(Protocol::UDP))?;

    if config.reuse_addr {
        socket.set_reuse_address(true)?;
    }

    if config.reuse_port {
        crate::sys::set_reuse_port(&socket, true)?;
    }

    socket
        .bind(&config.bind_addr.into())
        .map_err(|source| MulticastError::BindFailed {
            addr: config.bind_addr,
            source,
        })?;

    configure_runtime_options(&socket, config)?;

    let socket: StdUdpSocket = socket.into();
    socket.set_nonblocking(true)?;
    Ok(socket)
}

fn configure_runtime_options(socket: &Socket, config: &MulticastConfig) -> Result<()> {
    match config.bind_addr {
        SocketAddr::V4(_) => {
            socket.set_multicast_loop_v4(config.loopback)?;
            if let Some(ttl) = config.ttl {
                socket.set_multicast_ttl_v4(ttl)?;
            }
            if let Some(Interface::V4(addr)) = config.outbound_interface {
                socket.set_multicast_if_v4(&addr)?;
            }
        }
        SocketAddr::V6(_) => {
            socket.set_multicast_loop_v6(config.loopback)?;
            if let Some(ttl) = config.ttl {
                if ttl > u32::from(u8::MAX) {
                    return Err(MulticastError::UnsupportedOption("ipv6 ttl > 255"));
                }
                socket.set_unicast_hops_v6(ttl)?;
            }
            if let Some(Interface::V6(index)) = config.outbound_interface {
                socket.set_multicast_if_v6(index)?;
            }
        }
    }

    for membership in &config.memberships {
        crate::sys::join_membership(socket, membership, config.inbound_interface.as_ref())?;
    }

    Ok(())
}

pub(crate) fn membership_target(membership: &Membership) -> Result<IpAddr> {
    let group = membership.group();
    if !group.is_multicast() {
        return Err(MulticastError::InvalidGroupAddress(group));
    }
    Ok(group)
}

pub(crate) fn default_interface_v4(interface: Option<&Interface>) -> Result<Ipv4Addr> {
    match interface {
        None => Ok(Ipv4Addr::UNSPECIFIED),
        Some(Interface::V4(addr)) => Ok(*addr),
        Some(Interface::Name(_)) => Err(MulticastError::UnsupportedOption("interface name lookup")),
        Some(Interface::V6(_)) => Err(MulticastError::UnsupportedOption("ipv6 interface on ipv4 socket")),
    }
}

pub(crate) fn default_interface_v6(interface: Option<&Interface>) -> Result<u32> {
    match interface {
        None => Ok(0),
        Some(Interface::V6(index)) => Ok(*index),
        Some(Interface::Name(_)) => Err(MulticastError::UnsupportedOption("interface name lookup")),
        Some(Interface::V4(_)) => Err(MulticastError::UnsupportedOption("ipv4 interface on ipv6 socket")),
    }
}

pub(crate) fn group_as_v6_socket(group: std::net::Ipv6Addr, port: u16, scope_id: u32) -> SocketAddrV6 {
    SocketAddrV6::new(group, port, 0, scope_id)
}
