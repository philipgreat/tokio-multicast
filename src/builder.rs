use std::net::{IpAddr, SocketAddr};

use crate::{Interface, Membership, MulticastConfig, MulticastError, MulticastSocket, Result};

#[derive(Debug, Clone)]
pub struct MulticastSocketBuilder {
    config: MulticastConfig,
}

impl Default for MulticastSocketBuilder {
    fn default() -> Self {
        Self {
            config: MulticastConfig::default(),
        }
    }
}

impl MulticastSocketBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn bind(mut self, addr: SocketAddr) -> Self {
        self.config.bind_addr = addr;
        self
    }

    pub fn join(mut self, group: IpAddr) -> Self {
        self.config.memberships.push(Membership::any_source(group));
        self
    }

    pub fn join_source(mut self, group: IpAddr, source: IpAddr) -> Self {
        self.config
            .memberships
            .push(Membership::source_specific(group, source));
        self
    }

    pub fn inbound_interface(mut self, interface: Interface) -> Self {
        self.config.inbound_interface = Some(interface);
        self
    }

    pub fn outbound_interface(mut self, interface: Interface) -> Self {
        self.config.outbound_interface = Some(interface);
        self
    }

    pub fn reuse_addr(mut self, enabled: bool) -> Self {
        self.config.reuse_addr = enabled;
        self
    }

    pub fn reuse_port(mut self, enabled: bool) -> Self {
        self.config.reuse_port = enabled;
        self
    }

    pub fn loopback(mut self, enabled: bool) -> Self {
        self.config.loopback = enabled;
        self
    }

    pub fn ttl(mut self, ttl: u32) -> Self {
        self.config.ttl = Some(ttl);
        self
    }

    pub async fn build(self) -> Result<MulticastSocket> {
        if self.config.bind_addr.port() == 0 {
            return Err(MulticastError::BindAddressRequired);
        }

        if self.config.memberships.is_empty() {
            return Err(MulticastError::NoMembershipsConfigured);
        }

        MulticastSocket::from_config(self.config).await
    }
}
