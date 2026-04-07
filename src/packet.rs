use std::net::{IpAddr, SocketAddr};
use std::time::SystemTime;

use bytes::Bytes;

use crate::InterfaceId;

#[derive(Debug, Clone)]
pub struct RecvMeta {
    pub peer: SocketAddr,
    pub local_addr: Option<SocketAddr>,
    pub interface: Option<InterfaceId>,
    pub group: Option<IpAddr>,
    pub timestamp: Option<SystemTime>,
}

#[derive(Debug, Clone)]
pub struct Datagram {
    pub payload: Bytes,
    pub meta: RecvMeta,
}
