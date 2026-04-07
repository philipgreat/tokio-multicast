# tokio-multicast

Small Tokio helpers for UDP multicast.

## What it provides

- `bind_multicast(config)` to create a Tokio `UdpSocket` joined to a multicast group.
- `connect_multicast(config)` to create a Tokio `UdpSocket` connected to a multicast group for sending.
- IPv4 and IPv6 support through `MulticastAddr`.

## Example

```rust
use std::net::Ipv4Addr;
use tokio_multicast::{bind_multicast, connect_multicast, MulticastConfig};

#[tokio::main(flavor = "current_thread")]
async fn main() -> std::io::Result<()> {
    let config = MulticastConfig::new(Ipv4Addr::new(224, 0, 0, 251), 46001);

    let receiver = bind_multicast(config).await?;
    let sender = connect_multicast(config).await?;

    sender.send(b"hello").await?;

    let mut buf = [0_u8; 256];
    let n = receiver.recv(&mut buf).await?;
    assert_eq!(&buf[..n], b"hello");

    Ok(())
}
```

## Notes

- IPv4 defaults to `0.0.0.0` as the multicast interface and TTL `1`.
- IPv6 uses interface index `0` by default.
- `REAME.md` is intentionally kept as the project reference file because that is the file currently present in this repository.
