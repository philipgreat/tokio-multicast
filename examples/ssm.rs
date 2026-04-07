use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use tokio_multicast::MulticastSocket;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Placeholder example for the future source-specific multicast path.
    let result = MulticastSocket::builder()
        .bind(SocketAddr::from((Ipv4Addr::UNSPECIFIED, 5000)))
        .join_source(
            IpAddr::V4(Ipv4Addr::new(232, 1, 1, 1)),
            IpAddr::V4(Ipv4Addr::new(192, 168, 1, 10)),
        )
        .build()
        .await;

    match result {
        Ok(_) => println!("source-specific multicast socket created"),
        Err(err) => println!("source-specific multicast is not implemented yet: {err}"),
    }

    Ok(())
}
