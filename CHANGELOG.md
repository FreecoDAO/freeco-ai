# Changelog

All notable changes to FreEco.ai will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Security (threat-model hardening)

- **M4 — installer verification**: the one-click local-AI setup now verifies the downloaded Ollama installer's Windows Authenticode signature (must be Valid *and* signed by Ollama) before executing it. Previously the `.exe` was downloaded over HTTPS and run with no integrity check — a compromised mirror/CDN could have yielded arbitrary code execution inside the "safe" flow. Fails closed with a clear message pointing to manual install.
- **M2/M3 — secrets protected at rest on Windows**: `~/.openfang/secrets.env` file permissions were restricted only on Unix (`0600`); on Windows (a primary target) the file was left readable by any local account and copyable by folder-sync. Both write paths (CLI `config set-key` and the dashboard "Set API Key" button) now also apply a restrictive Windows ACL via `icacls` (strip inheritance, current user only).


## [0.7.4] - 2026-07-10

### Fixed

- Local AI model downloads now auto-resume after network drops (up to 6 total attempts with backoff; Ollama keeps finished layers) instead of erroring out and appearing to restart from zero.
- README: dummy-proof Windows install guide with a direct `-setup.exe` link, SmartScreen note, and desktop-shortcut mention — users were downloading the bare CLI `.zip` and finding "nothing installs". Broken `freeco.ai/install` one-liners replaced with working raw-GitHub URLs; stale v0.5.10 banner updated.
- Release CI: macOS desktop no longer fails by attempting notarization without an Apple Team ID after ad-hoc signing — the unsigned `.dmg` now builds.


### Added

- **FreEco.ai Editions in the dashboard** — the Overview page now offers one-click setup cards for the four editions: Personal Concierge, Kids (strictly guardrailed: no web, no shell, no agent messaging, parental visibility), Business Suite (Secretary + autonomous buyer team), and AI Company (CEO + Secretary + Shopping team). Five new bundled agent templates: `freeco-concierge`, `freeco-kids`, `freeco-ceo`, `freeco-secretary`, `freeco-shopping`.
- **Native freeco agents wired into the kernel** — the `freeco-agent-*` crates (previously compiled but unreachable) now execute via new `freeco:*` module ids (`freeco:ceo`, `freeco:secretary`, `freeco:shopping`): kernel dispatch in `execute_agent`, message conversion via `freeco-agent-core`'s bridge, deterministic CEO delegation planning with zero LLM cost, Secretary intent routing (LLM-classified when a key exists, keyword heuristic otherwise), Shopping three-tier research through the permission-gated tool gateway.
- **Self-development Dev Pod (Phase 6, stage 1)** — new `freeco-developer` and `freeco-tester` bundled agents: the developer implements labeled GitHub issues in a sandboxed repo clone and opens draft PRs; the tester/ethical-hacker verifies with build/test/clippy/`cargo audit` plus a security review of the diff and posts a PASS/FAIL verdict. Narrow manifests (shell limited to git/cargo/gh, network to github.com/crates.io, tester has no write access); humans merge. Dashboard gains a Dev Pod setup card; guide in `docs/self-development.md`; Phase 6 (Self-Running AI Company) added to the README roadmap.
- **One-click free local AI** — `POST /api/local-ai/setup` + `GET /api/local-ai/status`: detects Ollama, downloads/installs it silently per-user on Windows (official installer only, with progress), streams the model pull (default: lightweight Gemma), and points `[default_model]` at the local runtime. Settings → Providers gains a "Free local AI — no account needed" card with progress bar; macOS/Linux get a guided one-step path.

### CI

- macOS desktop bundles with ad-hoc signing when no Apple certificate is configured (previous fix skipped the import but left an empty signing identity, failing the bundle step).
- Tauri updater artifacts (`latest.json` + signatures) are now generated (`createUpdaterArtifacts: true`) — v0.7.3 is the first release existing installs can auto-update from.
- GHCR package-visibility step uses the user endpoint (FreecoDAO is not an org) and no longer fails the release.

### Earlier in this cycle

- Dashboard software updates: Settings → System now shows the installed vs latest version with a "Check for updates" button, a once-a-day automatic check (on by default, toggleable, throttled via localStorage), and a green "update available" pill in the sidebar linking to the download. Desktop-app CSP updated to allow the GitHub releases API.
- Dashboard sidebar now shows the fre.eco logo (replaced the old logo asset served at `/logo.png`).
- Portable / USB edition: OS-detecting launcher scripts (`scripts/portable/`) that run FreEco.ai from any folder or USB drive with all data kept alongside the binary via `OPENFANG_HOME`, plus `scripts/build-portable.sh` to assemble the bundle from release binaries, and `docs/usb-portable.md`.
- Landing page (`docs/index.html`) — self-contained green-brand site with edition overview, download links, terminal install commands, and PayPal donation QR; deployable via GitHub Pages or any static host.
- README roadmap: "Planned: Self-Contained Isolation Layer" (bundled agent browser + prebuilt sandbox image, Manus-style all-included isolation).
- First unit tests for `openfang-learning` (`PromotionPolicy::target_file` mapping).

### Changed

- Updated user-facing dashboard branding to display `FreEco.ai` in the sidebar header.
- Updated documentation to explicitly state that FreEco.ai is built on the OpenFang base with ethical and structural enhancements.

### Fixed

- Branding: browser-tab favicon and the entire desktop/installer icon set (shortcut, taskbar, Add/Remove Programs) still showed the old OpenFang cobra — all regenerated from the fre.eco logo.
- Voice-message transcription: `media_transcribe` resolved paths against the daemon's working directory instead of the agent workspace, and dashboard audio uploads were stored only under a UUID in a temp folder — so agents were told a filename that existed nowhere they could reach ("Failed to read audio file"). The tool now resolves workspace-relative paths first and falls back to the uploads folder by filename, and audio uploads keep a sanitized filename-addressable copy.
- README typos ("buisiness" → "business", "an SUSTAINABLE" → "a SUSTAINABLE").
- Dashboard light theme was a duplicate of the dark palette — it is now a real white day theme (the Light/System/Dark switcher finally has visible effect). Previously-undefined modal variables (`--bg-card`, `--bg-input`, `--bg-secondary`) no longer fall back to dark values.
- Desktop auto-updater pointed at the upstream repository (RightNow-AI/openfang) with the upstream signing key; it now points at FreecoDAO/freeco-ai signed with the project's own key.
- Release CI no longer fails the macOS desktop build when Apple signing certificates are absent — it produces an unsigned .dmg instead.
- Installer branding: product renamed OpenFang → FreEco.ai (shortcuts, Add/Remove Programs, Start Menu folder), installer version no longer stuck at 0.6.9, Windows installs per-user without admin prompts.

## [0.5.10] - 2026-04-17

### Fixed

- Non-loopback requests with no `api_key` configured now return 401 by default. Opt out with `OPENFANG_ALLOW_NO_AUTH=1`. Fixes the B1/B2 authentication bypass from #1034.
- Agent `context.md` is re-read on every turn so external updates take effect mid-session. Opt out per agent with `cache_context = true` on the manifest. Fixes #843.
- `openfang config get default_model.base_url` now prints the configured URL instead of an empty string. Missing keys return a clear "not found" error. Fixes #905.
- `schedule_create`, `schedule_list`, and `schedule_delete` tools plus the `/api/schedules` routes now use the kernel cron scheduler, so scheduled jobs actually fire. One-shot idempotent migration imports legacy shared-memory entries at startup. Fixes #1069.
- Multimodal user messages now combine text and image blocks into a single message so the LLM sees both. Fixes #1043.

### Added

- `openfang hand config <id>` subcommand: get, set, unset, and list settings on an active hand instance. Fixes #809.
- Optional per-channel `prefix_agent_name` setting (`off` / `bracket` / `bold_bracket`). Wraps outbound agent responses so users in multi-agent channels can see which agent replied. Default is off, byte-identical to prior behavior. Fixes #980.

### Closed as invalid

- #818 and #819. Both reference a knowledge-domain API that does not exist on `main`. Filed against an unmerged feature branch (`plan/013-audit-remediation`). Close with a note to build the proposed validation and stale-timestamp surfacing into that feature when it lands.

## [0.5.9] - 2026-04-10

### Changed

- **BREAKING:** Dashboard password hashing switched from SHA256 to Argon2id. Existing `password_hash` values in `config.toml` must be regenerated with `openfang auth hash-password`. Only affects users with `[auth] enabled = true`.

### Fixed

- Dashboard passwords were hashed with plain SHA256 (no salt), making them vulnerable to rainbow table and GPU-accelerated brute force attacks. Now uses Argon2id with random salts.

## [0.1.0] - 2026-02-24

### Added

#### Core Platform
- 15-crate Rust workspace: types, memory, runtime, kernel, api, channels, wire, cli, migrate, skills, hands, extensions, desktop, xtask
- Agent lifecycle management: spawn, list, kill, clone, mode switching (Full/Assist/Observe)
- SQLite-backed memory substrate with structured KV, semantic recall, vector embeddings
- 41 built-in tools (filesystem, web, shell, browser, scheduling, collaboration, image analysis, inter-agent, TTS, media)
- WASM sandbox with dual metering (fuel + epoch interruption with watchdog thread)
- Workflow engine with pipelines, fan-out parallelism, conditional steps, loops, and variable expansion
- Visual workflow builder with drag-and-drop node graph, 7 node types, and TOML export
- Trigger system with event pattern matching, content filters, and fire limits
- Event bus with publish/subscribe and correlation IDs
- 7 Hands packages for autonomous agent actions

#### LLM Support
- 3 native LLM drivers: Anthropic, Google Gemini, OpenAI-compatible
- 27 providers: Anthropic, Gemini, OpenAI, Groq, OpenRouter, DeepSeek, Together, Mistral, Fireworks, Cohere, Perplexity, xAI, AI21, Cerebras, SambaNova, Hugging Face, Replicate, Ollama, vLLM, LM Studio, and more
- Model catalog with 130+ built-in models, 23 aliases, tier classification
- Intelligent model routing with task complexity scoring
- Fallback driver for automatic failover between providers
- Cost estimation and metering engine with per-model pricing
- Streaming support (SSE) across all drivers

#### Token Management & Context
- Token-aware session compaction (chars/4 heuristic, triggers at 70% context capacity)
- In-loop emergency trimming at 70%/90% thresholds with summary injection
- Tool profile filtering (cuts default 41 tools to 4-10 for chat agents, saving 15-20K tokens)
- Context budget allocation for system prompt, tools, history, and response
- MAX_TOOL_RESULT_CHARS reduced from 50K to 15K to prevent tool result bloat
- Default token quota raised from 100K to 1M per hour

#### Security
- Capability-based access control with privilege escalation prevention
- Path traversal protection in all file tools
- SSRF protection blocking private IPs and cloud metadata endpoints
- Ed25519 signed agent manifests
- Merkle hash chain audit trail with tamper detection
- Information flow taint tracking
- HMAC-SHA256 mutual authentication for peer wire protocol
- API key authentication with Bearer token
- GCRA rate limiter with cost-aware token buckets
- Security headers middleware (CSP, X-Frame-Options, HSTS)
- Secret zeroization on all API key fields
- Subprocess environment isolation
- Health endpoint redaction (public minimal, auth full)
- Loop guard with SHA256-based detection and circuit breaker thresholds
- Session repair (validates and fixes orphaned tool results, empty messages)

#### Channels
- 40 channel adapters: Telegram, Discord, Slack, WhatsApp, Signal, Matrix, Email, Teams, Mattermost, Google Chat, Webex, Feishu/Lark, LINE, Viber, Facebook Messenger, Mastodon, Bluesky, Reddit, LinkedIn, Twitch, IRC, XMPP, and 18 more
- Unified bridge with agent routing, command handling, message splitting
- Per-channel user filtering and RBAC enforcement
- Graceful shutdown, exponential backoff, secret zeroization on all adapters

#### API
- 100+ REST/WS/SSE API endpoints (axum 0.8)
- WebSocket real-time streaming with per-agent connections
- OpenAI-compatible `/v1/chat/completions` API (streaming SSE + non-streaming)
- OpenAI-compatible `/v1/models` endpoint
- WebChat embedded UI with Alpine.js
- Google A2A protocol support (agent card, task send/get/cancel)
- Prometheus text-format `/api/metrics` endpoint for monitoring
- Multi-session management: list, create, switch, label sessions per agent
- Usage analytics: summary, by-model, daily breakdown
- Config hot-reload via polling (30-second interval, no restart required)

#### Web UI
- Chat message search with Ctrl+F, real-time filtering, text highlighting
- Voice input with hold-to-record mic button (WebM/Opus codec)
- TTS audio playback inline in tool cards
- Browser screenshot rendering in chat (inline images)
- Canvas rendering with iframe sandbox and CSP support
- Session switcher dropdown in chat header
- 6-step first-run setup wizard with provider API key help (12 providers)
- Skill marketplace with 4 tabs (Installed, ClawHub, MCP Servers, Quick Start)
- Copy-to-clipboard on messages, message timestamps
- Visual workflow builder with drag-and-drop canvas

#### Client SDKs
- JavaScript SDK (`@openfang/sdk`): full REST API client with streaming, TypeScript declarations
- Python client SDK (`openfang_client`): zero-dependency stdlib client with SSE streaming
- Python agent SDK (`openfang_sdk`): decorator-based framework for writing Python agents
- Usage examples for both languages (basic + streaming)

#### CLI
- 14+ subcommands: init, start, agent, workflow, trigger, migrate, skill, channel, config, chat, status, doctor, dashboard, mcp
- Daemon auto-detection via PID file
- Shell completion generation (bash, zsh, fish, PowerShell)
- MCP server mode for IDE integration

#### Skills Ecosystem
- 60 bundled skills across 14 categories
- Skill registry with TOML manifests
- 4 runtimes: Python, Node.js, WASM, PromptOnly
- FangHub marketplace with search/install
- ClawHub client for OpenClaw skill compatibility
- SKILL.md parser with auto-conversion
- SHA256 checksum verification
- Prompt injection scanning on skill content

#### Desktop App
- Tauri 2.0 native desktop app
- System tray with status and quick actions
- Single-instance enforcement
- Hide-to-tray on close
- Updated CSP for media, frame, and blob sources

#### Session Management
- LLM-based session compaction with token-aware triggers
- Multi-session per agent with named labels
- Session switching via API and UI
- Cross-channel canonical sessions
- Extended chat commands: `/new`, `/compact`, `/model`, `/stop`, `/usage`, `/think`

#### Image Support
- `ContentBlock::Image` with base64 inline data
- Media type validation (png, jpeg, gif, webp only)
- 5MB size limit enforcement
- Mapped to all 3 native LLM drivers

#### Usage Tracking
- Per-response cost estimation with model-aware pricing
- Usage footer in WebSocket responses and WebChat UI
- Usage events persisted to SQLite
- Quota enforcement with hourly windows

#### Interoperability
- OpenClaw migration engine (YAML/JSON5 to TOML)
- MCP client (JSON-RPC 2.0 over stdio/SSE, tool namespacing)
- MCP server (exposes OpenFang tools via MCP protocol)
- A2A protocol client and server
- Tool name compatibility mappings (21 OpenClaw tool names)

#### Infrastructure
- Multi-stage Dockerfile (debian:bookworm-slim runtime)
- docker-compose.yml with volume persistence
- GitHub Actions CI (check, test, clippy, format)
- GitHub Actions release (multi-platform, GHCR push, SHA256 checksums)
- Cross-platform install script (curl/irm one-liner)
- systemd service file for Linux deployment

#### Multi-User
- RBAC with Owner/Admin/User/Viewer roles
- Channel identity resolution
- Per-user authorization checks
- Device pairing and approval system

#### Production Readiness
- 1731+ tests across 15 crates, 0 failures
- Cross-platform support (Linux, macOS, Windows)
- Graceful shutdown with signal handling (SIGINT/SIGTERM on Unix, Ctrl+C on Windows)
- Daemon PID file with stale process detection
- Release profile with LTO, single codegen unit, symbol stripping
- Prometheus metrics for monitoring
- Config hot-reload without restart

[0.1.0]: https://github.com/FreecoDAO/FreEco-ai/releases/tag/v0.1.0
