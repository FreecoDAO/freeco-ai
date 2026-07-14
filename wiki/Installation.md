# Installation

## Desktop application

Download the platform installer from the
[releases page](https://github.com/FreecoDAO/freeco-ai/releases). The desktop
application starts the local server and opens the dashboard.

## Build from source

Install Rust 1.75 or newer and the native dependencies for your platform, then:

```bash
cargo build --release -p openfang-cli
target/release/openfang init
target/release/openfang start
```

Open `http://127.0.0.1:4200` after the daemon starts.

## First-run checklist

1. Run `openfang init`.
2. Configure a model provider or local model.
3. Set an API credential before exposing the API outside loopback.
4. Start the daemon and verify `/api/health`.

See [Configuration](Configuration) for configuration and credential guidance.
