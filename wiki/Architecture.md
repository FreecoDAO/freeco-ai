# Architecture

FreEco.ai is a Cargo workspace containing 23 Rust crates plus `xtask`. The
`openfang-*` crates provide the Agent OS foundation; `freeco-*` crates provide
FreEco-specific agents, runtime services, budget handling, and tool gateways.

## Runtime layers

1. **Types and memory** — shared models, configuration, capabilities, and
   SQLite-backed state.
2. **Runtime** — agent loops, model drivers, tools, sessions, audit trail,
   sandboxing, MCP, and A2A.
3. **Kernel** — agent registry, scheduling, workflows, approvals, model routing,
   metering, and lifecycle management.
4. **API** — Axum HTTP, WebSocket, and SSE interfaces plus the dashboard.
5. **Clients** — the CLI and Tauri desktop app.

## Request flow

The API server authenticates a request before route handlers access kernel state.
Handlers use `AppState` to reach the kernel. Channel adapters communicate through
the bridge layer; the WhatsApp Web gateway is a separate loopback-only Node
process managed by the kernel.

## Data locations

By default, runtime configuration and state are under `~/.openfang/`:

- `config.toml` — configuration
- `data/` — SQLite databases and persistent state
- `agents/` — installed manifests
- `workspaces/` — agent private state and sessions
