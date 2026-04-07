use std::net::SocketAddr;

use crate::{Interface, Membership};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MulticastConfig {
    pub bind_addr: SocketAddr,
    pub memberships: Vec<Membership>,
    pub inbound_interface: Option<Interface>,
    pub outbound_interface: Option<Interface>,
    pub reuse_addr: bool,
    pub reuse_port: bool,
    pub loopback: bool,
    pub ttl: Option<u32>,
}

impl Default for MulticastConfig {
    fn default() -> Self {
        Self {
            bind_addr: SocketAddr::from(([0, 0, 0, 0], 0)),
            memberships: Vec::new(),
            inbound_interface: None,
            outbound_interface: None,
            reuse_addr: true,
            reuse_port: false,
            loopback: true,
            ttl: Some(1),
        }
    }
}
