# openfang-learning

**Self-improving learning loop for OpenFang agents**, inspired by the [Hermes Agent](https://github.com/IncomeStreamSurfer/paperclip-surfers) self-improvement architecture.

Developed for [Freeco AI](https://fre.eco) — a Swiss high-end sustainable shopping concierge.

---

## Overview

`openfang-learning` adds a persistent, structured **self-improving learning loop** to any OpenFang agent. Agents can capture corrections, knowledge gaps, errors, and best practices during task execution, and automatically promote high-value learnings to their core memory files — enabling true cross-session improvement.

## Architecture

```
Agent Task Execution
       │
       ▼
┌─────────────────┐    capture()    ┌──────────────────────────┐
│  LearningLoop   │ ─────────────► │  .learnings/              │
│  (this crate)   │                │  ├── LEARNINGS.md          │
└─────────────────┘                │  ├── ERRORS.md             │
       │                           │  ├── SECURITY.md           │
       │ promote_to_core_memory()  │  └── FEATURE_REQUESTS.md   │
       ▼                           └──────────────────────────┘
┌─────────────────┐
│  Core Memory    │
│  ├── SOUL.md    │  ← Agent identity & values
│  ├── AGENTS.md  │  ← Multi-agent knowledge
│  └── TOOLS.md   │  ← Tool usage patterns
└─────────────────┘
```

## Features

- **Capture** corrections, knowledge gaps, errors, best practices, and security observations
- **Score** learnings by recurrence and impact using a composite heuristic
- **Auto-promote** high-scoring learnings to core memory files (SOUL.md, AGENTS.md, TOOLS.md)
- **Replay** accumulated learnings into new sessions via system prompt injection
- **Security-first** — security observations are always promoted immediately
- **Optional** integration with the OpenFang memory substrate (SQLite + semantic store)

## Usage

```rust
use openfang_learning::{LearningLoop, LearningType};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut loop_ = LearningLoop::new("/path/to/agent/home").await?;

    // Capture a correction
    loop_.capture(
        LearningType::Correction,
        "web_search",
        "Tavily requires a non-empty query string",
    ).await?;

    // Capture an error with resolution
    loop_.capture_error(
        "novita_api",
        "HTTP 403 NOT_ENOUGH_BALANCE",
        Some("Top up Novita AI account at https://novita.ai/console".into()),
    ).await?;

    // Capture a security observation (always promoted immediately)
    loop_.capture_security(
        "api_keys",
        "API key was found hardcoded in a config file — moved to .env",
    ).await?;

    // Load all learnings for injection into a new session
    let context = loop_.load_for_replay().await?;
    println!("Session context:\n{}", context);

    Ok(())
}
```

## Modules

| Module | Description |
|---|---|
| `event` | `LearningEvent` and `LearningType` data structures |
| `loop_manager` | Main `LearningLoop` struct and public API |
| `scoring` | Composite scoring heuristic for promotion decisions |
| `promotion` | `PromotionPolicy` — configures thresholds and target files |
| `replay` | Load accumulated learnings for session seeding |

## Integration with Freeco AI

This crate is integrated into the **FreecoDAO/openfang** fork as the `openfang-learning` workspace crate. It powers the self-improving capabilities of all Freeco AI agents, including:

- **Freeco CEO (Manus AI)** — strategic learning and cross-department knowledge
- **CEO Secretary (Hermes/OpenFang)** — operational pattern learning
- **Freeco Shopping Concierge** — product recommendation improvement over time

## License

Apache-2.0 OR MIT — same as OpenFang.
