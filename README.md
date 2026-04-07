# tokio-multicast

Tokio-native helpers for UDP multicast.

## Status

Implemented now:

- builder-based socket construction
- IPv4 ASM join, send, receive, and dynamic leave
- basic packet metadata via `recv_datagram()`
- platform split under `src/sys/`

Not implemented yet:

- source-specific multicast data path
- rich destination/interface receive metadata
- full stream integration with `tokio-stream`

## Crate layout

This crate follows the structure proposed in `internal-impl.MD`:

- `builder.rs` for high-level construction
- `config.rs` for configuration state
- `error.rs` for multicast-specific errors
- `interface.rs` and `membership.rs` for multicast intent types
- `packet.rs` for received datagram metadata
- `socket.rs` for the async multicast socket
- `stream.rs` for stream-style consumption helpers
- `raw.rs` and `sys/*` for socket setup and platform adaptation

## Quick start

```rust
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use tokio_multicast::MulticastSocket;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let socket = MulticastSocket::builder()
        .bind(SocketAddr::from((Ipv4Addr::UNSPECIFIED, 46053)))
        .join(IpAddr::V4(Ipv4Addr::new(239, 1, 1, 10)))
        .build()
        .await?;

    socket.send_to_group(b"hello multicast").await?;
    Ok(())
}
```

## Examples

- `cargo run --example simple_receiver`
- `cargo run --example simple_sender`
- `cargo run --example send_and_receive`
- `cargo run --example diagnostics`
- `cargo run --example multi_group`
- `cargo run --example ssm`

`simple_receiver` blocks waiting for multicast traffic. `send_and_receive` demonstrates the full path in one process. `diagnostics` prints a quick environment support report. `ssm` is intentionally a placeholder example that currently demonstrates the not-yet-implemented path.

## Diagnostics

Use `tokio_multicast::diagnose_multicast()` when you want a quick environment check before debugging higher-level multicast behavior.
If you need to control groups, interface index, or timeout, use `diagnose_multicast_with_config(...)`.

```rust
let report = tokio_multicast::diagnose_multicast();
println!("overall supported: {}", report.supported());
println!("ipv4: {:?}", report.ipv4.stages);
println!("ipv4 details: {:?}", report.ipv4.details);
println!("ipv6: {:?}", report.ipv6.stages);
println!("ipv6 details: {:?}", report.ipv6.details);
```

Example commands:

- `cargo run --example diagnostics`
- `cargo run --example diagnostics -- --json`
- `cargo run --example diagnostics -- --ipv4-only`
- `cargo run --example diagnostics -- --ipv6-only`
- `cargo run --example diagnostics -- --timeout-ms 1000`
- `cargo run --example diagnostics -- --ipv4-group 239.1.1.250 --ipv6-group ff02::114`

Exit codes:

- `0`: selected probes succeeded
- `2`: selected probes did not succeed

## Testing

- `cargo test`
- `cargo test --examples`
- `cargo test --doc`
