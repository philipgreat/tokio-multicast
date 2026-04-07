use std::net::Ipv4Addr;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Interface {
    V4(Ipv4Addr),
    V6(u32),
    Name(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InterfaceId(pub u32);
