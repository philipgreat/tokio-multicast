mod builder;
mod config;
mod diagnostics;
mod error;
mod interface;
mod membership;
mod packet;
mod raw;
mod socket;
mod stream;
mod sys;

pub use builder::MulticastSocketBuilder;
pub use config::MulticastConfig;
pub use diagnostics::{
    diagnose_multicast, diagnose_multicast_with_config, MulticastDiagnosticConfig,
    MulticastDiagnostics, ProbeErrorKind, ProbeResult, ProbeStages,
};
pub use error::{MulticastError, Result};
pub use interface::{Interface, InterfaceId};
pub use membership::Membership;
pub use packet::{Datagram, RecvMeta};
pub use socket::MulticastSocket;
pub use stream::MulticastReceiver;

#[cfg(test)]
mod tests {
    use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, UdpSocket as StdUdpSocket};

    use tokio::time::{timeout, Duration};

    use crate::{
        diagnose_multicast, diagnose_multicast_with_config, Interface, Membership,
        MulticastDiagnosticConfig, MulticastError, MulticastReceiver, MulticastSocket,
    };

    fn free_port() -> u16 {
        StdUdpSocket::bind(SocketAddr::from((Ipv4Addr::LOCALHOST, 0)))
            .unwrap()
            .local_addr()
            .unwrap()
            .port()
    }

    fn free_port_v6() -> Option<u16> {
        StdUdpSocket::bind(SocketAddr::from((Ipv6Addr::LOCALHOST, 0)))
            .ok()?
            .local_addr()
            .ok()
            .map(|addr| addr.port())
    }

    #[cfg(target_os = "linux")]
    fn loopback_ifindex_v6() -> Option<u32> {
        crate::sys::loopback_interface_v6()
    }

    #[cfg(target_os = "macos")]
    fn loopback_ifindex_v6() -> Option<u32> {
        crate::sys::loopback_interface_v6()
    }

    #[cfg(target_os = "windows")]
    fn loopback_ifindex_v6() -> Option<u32> {
        crate::sys::loopback_interface_v6()
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    fn loopback_ifindex_v6() -> Option<u32> {
        crate::sys::loopback_interface_v6()
    }

    #[tokio::test(flavor = "current_thread")]
    async fn multicast_round_trip_ipv4() {
        let port = free_port();
        let socket = MulticastSocket::builder()
            .bind(SocketAddr::from((Ipv4Addr::UNSPECIFIED, port)))
            .join(IpAddr::V4(Ipv4Addr::new(224, 0, 0, 251)))
            .build()
            .await
            .unwrap();

        socket.send_to_group(b"ping").await.unwrap();

        let mut buf = [0_u8; 64];
        let (size, _) = timeout(Duration::from_secs(2), socket.recv_from(&mut buf))
            .await
            .unwrap()
            .unwrap();

        assert_eq!(&buf[..size], b"ping");
        assert!(socket.memberships().contains(&Membership::any_source(IpAddr::V4(
            Ipv4Addr::new(224, 0, 0, 251),
        ))));
    }

    #[tokio::test(flavor = "current_thread")]
    async fn recv_datagram_includes_basic_metadata() {
        let port = free_port();
        let socket = MulticastSocket::builder()
            .bind(SocketAddr::from((Ipv4Addr::UNSPECIFIED, port)))
            .join(IpAddr::V4(Ipv4Addr::new(224, 0, 0, 252)))
            .build()
            .await
            .unwrap();

        socket.send_to_group(b"meta").await.unwrap();
        let datagram = timeout(Duration::from_secs(2), socket.recv_datagram(32))
            .await
            .unwrap()
            .unwrap();

        assert_eq!(datagram.payload.as_ref(), b"meta");
        assert_eq!(datagram.meta.group, Some(IpAddr::V4(Ipv4Addr::new(224, 0, 0, 252))));
        assert!(datagram.meta.local_addr.is_some());
        assert!(datagram.meta.timestamp.is_some());
    }

    #[tokio::test(flavor = "current_thread")]
    async fn builder_requires_bind_port() {
        let err = MulticastSocket::builder()
            .bind(SocketAddr::from((Ipv4Addr::UNSPECIFIED, 0)))
            .join(IpAddr::V4(Ipv4Addr::new(224, 0, 0, 251)))
            .build()
            .await
            .unwrap_err();

        assert!(matches!(err, MulticastError::BindAddressRequired));
    }

    #[tokio::test(flavor = "current_thread")]
    async fn builder_requires_membership() {
        let err = MulticastSocket::builder()
            .bind(SocketAddr::from((Ipv4Addr::UNSPECIFIED, free_port())))
            .build()
            .await
            .unwrap_err();

        assert!(matches!(err, MulticastError::NoMembershipsConfigured));
    }

    #[tokio::test(flavor = "current_thread")]
    async fn builder_rejects_non_multicast_group() {
        let err = MulticastSocket::builder()
            .bind(SocketAddr::from((Ipv4Addr::UNSPECIFIED, free_port())))
            .join(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)))
            .build()
            .await
            .unwrap_err();

        assert!(matches!(
            err,
            MulticastError::InvalidGroupAddress(IpAddr::V4(addr)) if addr == Ipv4Addr::new(127, 0, 0, 1)
        ));
    }

    #[tokio::test(flavor = "current_thread")]
    async fn duplicate_join_does_not_duplicate_membership_state() {
        let group = IpAddr::V4(Ipv4Addr::new(224, 0, 0, 253));
        let socket = MulticastSocket::builder()
            .bind(SocketAddr::from((Ipv4Addr::UNSPECIFIED, free_port())))
            .join(group)
            .build()
            .await
            .unwrap();

        socket.join(Membership::any_source(group)).await.unwrap();

        let memberships = socket.memberships();
        assert_eq!(memberships.len(), 1);
        assert!(memberships.contains(&Membership::any_source(group)));
    }

    #[tokio::test(flavor = "current_thread")]
    async fn leave_absent_membership_is_noop() {
        let joined = IpAddr::V4(Ipv4Addr::new(224, 0, 0, 254));
        let absent = Membership::any_source(IpAddr::V4(Ipv4Addr::new(224, 0, 0, 200)));
        let socket = MulticastSocket::builder()
            .bind(SocketAddr::from((Ipv4Addr::UNSPECIFIED, free_port())))
            .join(joined)
            .build()
            .await
            .unwrap();

        socket.leave(&absent).await.unwrap();

        let memberships = socket.memberships();
        assert_eq!(memberships.len(), 1);
        assert!(memberships.contains(&Membership::any_source(joined)));
    }

    #[tokio::test(flavor = "current_thread")]
    async fn dynamic_source_specific_join_is_rejected() {
        let socket = MulticastSocket::builder()
            .bind(SocketAddr::from((Ipv4Addr::UNSPECIFIED, free_port())))
            .join(IpAddr::V4(Ipv4Addr::new(224, 0, 0, 251)))
            .build()
            .await
            .unwrap();

        let err = socket
            .join(Membership::source_specific(
                IpAddr::V4(Ipv4Addr::new(232, 1, 1, 1)),
                IpAddr::V4(Ipv4Addr::new(192, 168, 1, 10)),
            ))
            .await
            .unwrap_err();

        assert!(matches!(
            err,
            MulticastError::UnsupportedOption("dynamic source-specific membership")
        ));
    }

    #[tokio::test(flavor = "current_thread")]
    async fn leave_existing_membership_removes_it_from_state() {
        let group = IpAddr::V4(Ipv4Addr::new(224, 0, 0, 155));
        let membership = Membership::any_source(group);
        let socket = MulticastSocket::builder()
            .bind(SocketAddr::from((Ipv4Addr::UNSPECIFIED, free_port())))
            .join(group)
            .build()
            .await
            .unwrap();

        socket.leave(&membership).await.unwrap();

        let memberships = socket.memberships();
        assert!(!memberships.contains(&membership));
        assert!(memberships.is_empty());
    }

    #[tokio::test(flavor = "current_thread")]
    async fn send_to_unicast_target_works() {
        let receiver = tokio::net::UdpSocket::bind(SocketAddr::from((Ipv4Addr::LOCALHOST, 0)))
            .await
            .unwrap();
        let target = receiver.local_addr().unwrap();

        let socket = MulticastSocket::builder()
            .bind(SocketAddr::from((Ipv4Addr::UNSPECIFIED, free_port())))
            .join(IpAddr::V4(Ipv4Addr::new(224, 0, 0, 156)))
            .build()
            .await
            .unwrap();

        socket.send_to(b"direct", target).await.unwrap();

        let mut buf = [0_u8; 64];
        let (size, from) = timeout(Duration::from_secs(2), receiver.recv_from(&mut buf))
            .await
            .unwrap()
            .unwrap();

        assert_eq!(&buf[..size], b"direct");
        assert_eq!(from.ip(), IpAddr::V4(Ipv4Addr::LOCALHOST));
    }

    #[tokio::test(flavor = "current_thread")]
    async fn receiver_wrapper_reads_datagram() {
        let port = free_port();
        let socket = MulticastSocket::builder()
            .bind(SocketAddr::from((Ipv4Addr::UNSPECIFIED, port)))
            .join(IpAddr::V4(Ipv4Addr::new(224, 0, 0, 157)))
            .build()
            .await
            .unwrap();

        let receiver = MulticastReceiver::new(&socket, 64);
        socket.send_to_group(b"stream").await.unwrap();

        let datagram = timeout(Duration::from_secs(2), receiver.recv())
            .await
            .unwrap()
            .unwrap();

        assert_eq!(datagram.payload.as_ref(), b"stream");
        assert_eq!(datagram.meta.group, Some(IpAddr::V4(Ipv4Addr::new(224, 0, 0, 157))));
    }

    #[tokio::test(flavor = "current_thread")]
    async fn config_and_local_addr_reflect_builder_values() {
        let bind_addr = SocketAddr::from((Ipv4Addr::UNSPECIFIED, free_port()));
        let socket = MulticastSocket::builder()
            .bind(bind_addr)
            .join(IpAddr::V4(Ipv4Addr::new(224, 0, 0, 158)))
            .reuse_addr(false)
            .reuse_port(false)
            .loopback(false)
            .ttl(16)
            .build()
            .await
            .unwrap();

        let config = socket.config();
        assert_eq!(config.bind_addr, bind_addr);
        assert_eq!(config.reuse_addr, false);
        assert_eq!(config.reuse_port, false);
        assert_eq!(config.loopback, false);
        assert_eq!(config.ttl, Some(16));
        assert_eq!(socket.local_addr().unwrap().port(), bind_addr.port());
    }

    #[tokio::test(flavor = "current_thread")]
    async fn ipv6_builder_path_smoke_test_if_supported() {
        let Some(port) = free_port_v6() else {
            return;
        };
        let Some(ifindex) = loopback_ifindex_v6() else {
            return;
        };
        let group: Ipv6Addr = "ff02::114".parse().unwrap();

        let result = MulticastSocket::builder()
            .bind(SocketAddr::from((Ipv6Addr::UNSPECIFIED, port)))
            .join(IpAddr::V6(group))
            .inbound_interface(Interface::V6(ifindex))
            .outbound_interface(Interface::V6(ifindex))
            .build()
            .await;

        match result {
            Ok(socket) => {
                assert_eq!(socket.config().bind_addr, SocketAddr::from((Ipv6Addr::UNSPECIFIED, port)));
                assert!(socket
                    .memberships()
                    .contains(&Membership::any_source(IpAddr::V6(group))));
            }
            Err(
                MulticastError::UnsupportedOption(_)
                | MulticastError::Io(_)
                | MulticastError::BindFailed { .. },
            ) => {
                // Some environments expose IPv6 sockets but do not permit multicast join
                // without a concrete interface index. Treat that as unsupported here.
            }
            Err(err) => panic!("unexpected IPv6 build result: {err}"),
        }
    }

    #[tokio::test(flavor = "current_thread")]
    async fn ipv6_round_trip_with_interface_index_if_supported() {
        let Some(port) = free_port_v6() else {
            return;
        };
        let Some(ifindex) = loopback_ifindex_v6() else {
            return;
        };
        let group: Ipv6Addr = "ff02::114".parse().unwrap();

        let result = MulticastSocket::builder()
            .bind(SocketAddr::from((Ipv6Addr::UNSPECIFIED, port)))
            .join(IpAddr::V6(group))
            .inbound_interface(Interface::V6(ifindex))
            .outbound_interface(Interface::V6(ifindex))
            .build()
            .await;

        let socket = match result {
            Ok(socket) => socket,
            Err(
                MulticastError::UnsupportedOption(_)
                | MulticastError::Io(_)
                | MulticastError::BindFailed { .. },
            ) => return,
            Err(err) => panic!("unexpected IPv6 round-trip build result: {err}"),
        };

        socket.send_to_group(b"ipv6").await.unwrap();

        let mut buf = [0_u8; 64];
        let (size, _) = timeout(Duration::from_secs(2), socket.recv_from(&mut buf))
            .await
            .unwrap()
            .unwrap();

        assert_eq!(&buf[..size], b"ipv6");
    }

    #[test]
    fn diagnostics_returns_structured_report() {
        let report = diagnose_multicast();

        assert!(
            !report.ipv4.label.is_empty() && !report.ipv6.label.is_empty(),
            "probe labels should be populated"
        );
        assert!(
            report.ipv4.details.is_some() || report.ipv4.error.is_some(),
            "ipv4 probe should explain its outcome"
        );
    }

    #[test]
    fn diagnostics_accepts_custom_config() {
        let config = MulticastDiagnosticConfig {
            timeout: Duration::from_millis(200),
            ..MulticastDiagnosticConfig::default()
        };
        let report = diagnose_multicast_with_config(&config);

        assert!(
            report.ipv4.details.as_deref().unwrap_or_default().contains("timeout")
                || report.ipv4.error.is_some()
        );
    }
}
