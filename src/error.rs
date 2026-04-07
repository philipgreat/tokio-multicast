use std::fmt::{Display, Formatter};
use std::io;
use std::net::{IpAddr, SocketAddr};

#[derive(Debug)]
pub enum MulticastError {
    InvalidGroupAddress(IpAddr),
    UnsupportedOption(&'static str),
    BindAddressRequired,
    NoMembershipsConfigured,
    BindFailed { addr: SocketAddr, source: io::Error },
    Io(io::Error),
}

impl Display for MulticastError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidGroupAddress(addr) => write!(f, "invalid multicast group address: {addr}"),
            Self::UnsupportedOption(option) => write!(f, "unsupported multicast option: {option}"),
            Self::BindAddressRequired => write!(f, "bind address is required"),
            Self::NoMembershipsConfigured => write!(f, "at least one multicast membership is required"),
            Self::BindFailed { addr, source } => write!(f, "failed to bind {addr}: {source}"),
            Self::Io(source) => Display::fmt(source, f),
        }
    }
}

impl std::error::Error for MulticastError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::BindFailed { source, .. } | Self::Io(source) => Some(source),
            _ => None,
        }
    }
}

impl From<io::Error> for MulticastError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

pub type Result<T> = std::result::Result<T, MulticastError>;
