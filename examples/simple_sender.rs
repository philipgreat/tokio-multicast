use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use tokio_multicast::MulticastSocket;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Sends a single datagram to the same group used by simple_receiver.
    let socket = MulticastSocket::builder()
        .bind(SocketAddr::from((Ipv4Addr::UNSPECIFIED, 46054)))
        .join(IpAddr::V4(Ipv4Addr::new(239, 1, 1, 10)))
        .build()
        .await?;

    socket
        .send_to(
            b"hello multicast",
            SocketAddr::from((Ipv4Addr::new(239, 1, 1, 10), 46053)),
        )
        .await?;
    Ok(())
}
