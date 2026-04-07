#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "linux")]
pub(crate) use linux::*;
#[cfg(target_os = "macos")]
pub(crate) use macos::*;
#[cfg(target_os = "windows")]
pub(crate) use windows::*;

#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
mod unsupported {
    use socket2::Socket;

    use crate::{Membership, MulticastError, Result};

    pub(crate) fn set_reuse_port(_: &Socket, enabled: bool) -> Result<()> {
        if enabled {
            return Err(MulticastError::UnsupportedOption("reuse_port"));
        }
        Ok(())
    }

    pub(crate) fn join_membership(
        _: &Socket,
        _: &Membership,
        _: Option<&crate::Interface>,
    ) -> Result<()> {
        Err(MulticastError::UnsupportedOption("multicast membership"))
    }

    pub(crate) fn loopback_interface_v6() -> Option<u32> {
        None
    }
}

#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
pub(crate) use unsupported::*;
