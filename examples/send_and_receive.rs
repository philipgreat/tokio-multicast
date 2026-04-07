use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use tokio::time::{timeout, Duration};
use tokio_multicast::MulticastSocket;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let group = Ipv4Addr::new(239, 1, 1, 11);
    let receiver_port = 46055;
    let sender_port = 46056;

    let receiver = MulticastSocket::builder()
        .bind(SocketAddr::from((Ipv4Addr::UNSPECIFIED, receiver_port)))
        .join(IpAddr::V4(group))
        .build()
        .await?;

    let sender = MulticastSocket::builder()
        .bind(SocketAddr::from((Ipv4Addr::UNSPECIFIED, sender_port)))
        .join(IpAddr::V4(group))
        .build()
        .await?;

    sender
        .send_to(b"hello multicast", SocketAddr::from((group, receiver_port)))
        .await?;

    let mut buf = [0_u8; 2048];
    let (n, from) = timeout(Duration::from_secs(2), receiver.recv_from(&mut buf))
        .await??
        ;

    println!("received {} bytes from {}", n, from);
    println!("payload: {}", String::from_utf8_lossy(&buf[..n]));

    Ok(())
}
