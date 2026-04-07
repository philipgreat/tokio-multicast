# tokio-multicast v0.7.1

Patch release for Linux socket portability.

## Highlights

- Fix Linux builds that failed when `socket2::Socket::set_reuse_port(...)` was not available in downstream dependency graphs.
- Replace the Linux `reuse_port` implementation with a direct `setsockopt(SO_REUSEPORT)` call.

## Release checks

The following checks were run for `v0.7.1`:

- `cargo check`
- `cargo test`
- `cargo package --allow-dirty`
- `cargo publish --dry-run --allow-dirty`
