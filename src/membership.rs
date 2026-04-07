use std::net::IpAddr;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Membership {
    AnySource { group: IpAddr },
    SourceSpecific { group: IpAddr, source: IpAddr },
}

impl Membership {
    pub fn any_source(group: IpAddr) -> Self {
        Self::AnySource { group }
    }

    pub fn source_specific(group: IpAddr, source: IpAddr) -> Self {
        Self::SourceSpecific { group, source }
    }

    pub fn group(&self) -> IpAddr {
        match self {
            Self::AnySource { group } | Self::SourceSpecific { group, .. } => *group,
        }
    }
}
