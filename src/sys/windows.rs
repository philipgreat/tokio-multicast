use std::net::IpAddr;

use socket2::Socket;

use crate::{raw, Interface, Membership, MulticastError, Result};

pub(crate) fn set_reuse_port(socket: &Socket, enabled: bool) -> Result<()> {
    // Winsock does not expose SO_REUSEPORT. For UDP multicast, SO_REUSEADDR is the
    // socket option that allows multiple sockets to bind the same local port and
    // each receive multicast traffic for the joined group.
    socket.set_reuse_address(enabled)?;
    Ok(())
}

pub(crate) fn join_membership(
    socket: &Socket,
    membership: &Membership,
    inbound_interface: Option<&Interface>,
) -> Result<()> {
    match membership {
        Membership::AnySource {
            group: IpAddr::V4(group),
        } => {
            let interface = raw::default_interface_v4(inbound_interface)?;
            raw::membership_target(membership)?;
            socket.join_multicast_v4(group, &interface)?;
            Ok(())
        }
        Membership::AnySource {
            group: IpAddr::V6(group),
        } => {
            let index = raw::default_interface_v6(inbound_interface)?;
            raw::membership_target(membership)?;
            socket.join_multicast_v6(group, index)?;
            Ok(())
        }
        Membership::SourceSpecific { .. } => Err(MulticastError::UnsupportedOption(
            "source-specific multicast",
        )),
    }
}

pub(crate) fn loopback_interface_v6() -> Option<u32> {
    None
}
