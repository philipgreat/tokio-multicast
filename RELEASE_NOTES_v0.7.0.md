# tokio-multicast v0.7.0

Initial public release of `tokio-multicast`.

## Highlights

- Tokio-oriented multicast socket API with builder-based construction
- IPv4 ASM send/receive support
- IPv6 multicast build and round-trip coverage with interface index handling
- Dynamic join/leave tracking for memberships
- Basic packet metadata through `recv_datagram()`
- Built-in multicast environment diagnostics for developers
- Configurable diagnostics API with protocol filtering and JSON-friendly output
- Example programs for send, receive, combined send/receive, diagnostics, multi-group, and SSM placeholder behavior

## Diagnostics

This release includes a developer-facing diagnostics API:

- `diagnose_multicast()`
- `diagnose_multicast_with_config(...)`

It can verify:

- socket creation
- bind
- group join
- loopback send
- loopback receive

There is also a runnable example:

```bash
cargo run --example diagnostics
cargo run --example diagnostics -- --json
cargo run --example diagnostics -- --ipv4-only
cargo run --example diagnostics -- --ipv6-only
```

## Release checks

The following checks were run for `v0.7.0`:

- `cargo test`
- `cargo package --allow-dirty`
- `cargo publish --dry-run --allow-dirty`

## Notes

- Source-specific multicast is not fully implemented yet.
- Rich destination/interface receive metadata is not fully implemented yet.
