# Development and testing

## Prerequisites

Install Rust 1.75+, Node.js 18+ for the optional WhatsApp gateway, ShellCheck,
and the platform libraries needed by the desktop crate.

## Required checks

Run these from the repository root:

```bash
cargo build --workspace --lib
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
shellcheck scripts/install.sh
```

Run `cargo fmt --all -- --check` before submitting formatting-sensitive changes.
Security-sensitive changes should also run dependency and secret scanning.

## Integration testing

For API wiring changes, start a fresh daemon and call each changed endpoint over
HTTP. Confirm that writes persist and that authentication rejects missing or
invalid credentials. Do not use production keys for local development tests.
