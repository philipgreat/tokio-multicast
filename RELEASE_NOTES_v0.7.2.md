# tokio-multicast v0.7.2

Patch release for `reuse_port` portability on macOS and Windows.

## Highlights

- Implement `reuse_port` on macOS with a direct `setsockopt(SO_REUSEPORT)` call.
- Map `reuse_port` to `SO_REUSEADDR` on Windows so UDP multicast sockets can share a bound port using Winsock-supported semantics.
- Add a platform smoke test that exercises the `set_reuse_port` path directly.

## Release checks

The following checks were run for `v0.7.2`:

- `cargo check`
- `cargo test`
- `cargo package --allow-dirty`
- `cargo publish --dry-run --allow-dirty`
