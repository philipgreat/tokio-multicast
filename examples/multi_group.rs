use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use tokio_multicast::MulticastSocket;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Joins multiple groups and prints the tracked membership count.
    let socket = MulticastSocket::builder()
        .bind(SocketAddr::from((Ipv4Addr::UNSPECIFIED, 5000)))
        .join(IpAddr::V4(Ipv4Addr::new(239, 1, 1, 1)))
        .join(IpAddr::V4(Ipv4Addr::new(239, 1, 1, 2)))
        .build()
        .await?;

    println!("listening on {}", socket.local_addr()?);
    let groups = socket.memberships();
    println!("joined {} groups", groups.len());
    Ok(())
}
