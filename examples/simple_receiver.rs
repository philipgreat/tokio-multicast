use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use tokio_multicast::MulticastSocket;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Listens on a multicast group until interrupted.
    let socket = MulticastSocket::builder()
        .bind(SocketAddr::from((Ipv4Addr::UNSPECIFIED, 46053)))
        .join(IpAddr::V4(Ipv4Addr::new(239, 1, 1, 10)))
        .build()
        .await?;

    let mut buf = [0_u8; 2048];
    loop {
        let (n, from) = socket.recv_from(&mut buf).await?;
        println!("received {n} bytes from {from}");
    }
}
