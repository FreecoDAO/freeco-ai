# Freeco.ai Platform — Working Plan

> **Status:** Active — M0 in progress  
> **Updated:** 2026-06-24  
> **Owner:** FreecoDAO  
> **Tracking:** GitHub Project "Freeco.ai Platform" (FreecoDAO org)

---

## Table of Contents

- [1. Project Scope & Boundaries](#1-project-scope--boundaries)
- [2. Monorepo Baseline](#2-monorepo-baseline)
- [3. Migration Inventory](#3-migration-inventory)
- [4. Migration Waves](#4-migration-waves)
- [5. Acceptance Gates](#5-acceptance-gates)
- [6. Operating Model](#6-operating-model)
- [7. Milestones](#7-milestones)
- [8. GitHub Project Setup](#8-github-project-setup)
- [9. Weekly Review Cadence](#9-weekly-review-cadence)

---

## 1. Project Scope & Boundaries

### Inside the Platform

| Domain | What It Includes |
|--------|-----------------|
| **Core runtime** | `freeco-runtime`, `freeco-agent-core`, `freeco-budget-engine`, `freeco-tool-gateway`, `freeco-input-parser` |
| **Agent layer** | `freeco-agent-shopping`, `freeco-agent-secretary`, `freeco-agent-ceo`, future domain agents |
| **OpenFang integration** | `openfang-runtime`, `openfang-kernel`, `openfang-api`, `openfang-types`, `openfang-memory` (shared substrate) |
| **API / UI** | Axum REST server (`openfang-api`), Alpine.js dashboard (`index_body.html`), Tauri desktop (`openfang-desktop`) |
| **Subscription & billing** | Subscription tiers (Free/Concierge/Business), token top-up packs, SQLite budget ledger |
| **Docs** | `docs/`, `README.md`, `MIGRATION.md`, architecture diagrams |
| **CI / release automation** | `.github/workflows/ci.yml`, `.github/workflows/release.yml`, `scripts/` |

### Outside the Platform (Integrate by API)

| System | Integration strategy |
|--------|---------------------|
| **Google Cloud / Vertex AI** | REST driver in `openfang-runtime/drivers/vertex.rs`; credentials via env var |
| **Stripe / payment gateway** | Called from `freeco-budget-engine` via a thin HTTP client; no code lives here |
| **FangHub marketplace** | Remote REST API; `openfang-skills` holds the client only |
| **External A2A agents** | `openfang-runtime` A2A protocol; agent code stays in those repos |

### Archive Candidates

Any standalone repo whose full logic has been absorbed into a `crates/freeco-*` workspace member and has no independent consumers should be archived after cutover. Verify zero open PRs and no CI-dependent downstream jobs before archiving.

---

## 2. Monorepo Baseline

This repository (`FreecoDAO/openfang`) is the **single platform monorepo**.

### Current workspace members — `Cargo.toml`

```
openfang-types          openfang-memory         openfang-runtime
openfang-wire           openfang-api            openfang-kernel
openfang-cli            openfang-channels       openfang-migrate
openfang-skills         openfang-desktop        openfang-hands
openfang-extensions     openfang-learning

freeco-agent-core       freeco-runtime          freeco-tool-gateway
freeco-budget-engine    freeco-input-parser
freeco-agent-shopping   freeco-agent-secretary  freeco-agent-ceo
```

### Crate dependency graph (freeco-* layer)

```
freeco-agent-core   ←──────────────────────────── openfang-types
         ↑
freeco-tool-gateway                              (external APIs)
freeco-input-parser
         ↑
freeco-agent-shopping  ←── freeco-agent-core, freeco-tool-gateway, freeco-input-parser
freeco-agent-secretary ←── freeco-agent-core, freeco-tool-gateway
freeco-agent-ceo       ←── freeco-agent-core, freeco-tool-gateway, openfang-runtime
         ↑
freeco-runtime     ←── freeco-agent-core, freeco-budget-engine, freeco-tool-gateway, openfang-runtime
```

---

## 3. Migration Inventory

Each row describes a Freeco component already in the workspace, its current state, and the target outcome.

| Crate | Function | Status | WASM target | Outcome |
|-------|----------|--------|-------------|---------|
| `freeco-agent-core` | Base `Agent` trait, `Message`, `AgentResponse`, `LearningStore`, OpenFang bridge | ✅ In workspace | ✅ Yes (`cdylib + rlib`) | **Done — already migrated** |
| `freeco-runtime` | Thin re-export shim over `openfang-runtime` | ✅ In workspace | ❌ Native only | **Done — already migrated** |
| `freeco-budget-engine` | SQLite token ledger, subscription tiers (Free/Concierge/Business at CHF 0/11/22/mo), top-up packs | ✅ In workspace | ❌ Native only | **Done — already migrated** |
| `freeco-tool-gateway` | Permission-gated API clients: Tavily, Open Food Facts, Google Places, LLM (Gemma 4 26B via Novita) | ✅ In workspace | ❌ Native only | **Done — already migrated** |
| `freeco-input-parser` | Multi-format shopping list parser: text, voice transcript, image OCR (Google Vision), CSV | ✅ In workspace | ❌ Native only | **Done — already migrated** |
| `freeco-agent-shopping` | Shopping Concierge: Tavily search → Open Food Facts → Google Places → 3-tier recommendation | ✅ In workspace | ✅ Yes (`cdylib + rlib`) | **Done — already migrated** |
| `freeco-agent-secretary` | Primary router: classify intent → route to Shopping, Memo/Calendar, or General LLM reply | ✅ In workspace | ✅ Yes (`cdylib + rlib`) | **Done — already migrated** |
| `freeco-agent-ceo` | Executive routing using OpenFang kernel handles | ✅ In workspace | ✅ Yes (`cdylib + rlib`) | **Done — already migrated** |

### Open migration gaps

| Item | Description | Owner | Target wave |
|------|-------------|-------|-------------|
| **API routes for freeco agents** | REST endpoints to invoke `ShoppingAgent`, `SecretaryAgent`, `CeoAgent` via the Axum server | Platform team | Wave 2 |
| **Budget API endpoints** | `GET /api/freeco/budget`, `POST /api/freeco/budget/tier`, `POST /api/freeco/budget/topup` | Platform team | Wave 1 |
| **Dashboard tabs** | Freeco-specific sections in `index_body.html` (subscription status, agent selection) | UI team | Wave 2 |
| **WASM build pipeline** | `wasm-pack` CI step to compile `freeco-agent-*` to WASM for browser/edge deployment | DevOps | Wave 3 |
| **LLM driver for Novita** | Gemma 4 26B via Novita AI currently routed through generic OpenAI-compat driver; add named driver | Runtime team | Wave 2 |
| **Stripe billing bridge** | Connect `freeco-budget-engine` tier management to Stripe webhooks | Backend team | Wave 3 |
| **Internationalization** | Multi-language shopping list parsing already in `freeco-input-parser`; wire to UI locale | UI team | Wave 3 |

---

## 4. Migration Waves

### Wave 1 — Core Platform (runtime / kernel / API / tool gateway / budget)

**Goal:** All core infrastructure is live in production and end-to-end tested.

| Task | Crates touched | Acceptance gate |
|------|----------------|-----------------|
| Wire `freeco-budget-engine` into `openfang-kernel` metering | `freeco-budget-engine`, `openfang-kernel` | `cargo test --workspace` green; `GET /api/freeco/budget` returns tier data |
| Add Freeco budget API endpoints | `openfang-api/src/routes.rs`, `openfang-api/src/server.rs` | Live integration test: tier set → tokens deducted after agent call |
| Register `ToolGateway` in kernel `AppState` | `openfang-api/src/server.rs` | No-panic boot; tool permission denial returns HTTP 403 |
| Validate `openfang-runtime` ↔ `freeco-runtime` shim | `freeco-runtime` | All `cargo test -p freeco-runtime` pass |

### Wave 2 — Agent Layer (shopping / secretary / ceo)

**Goal:** All three Freeco agents are reachable via the REST API and the web dashboard.

| Task | Crates touched | Acceptance gate |
|------|----------------|-----------------|
| Add `/api/freeco/agents/shopping/message` endpoint | `openfang-api` | POST with shopping query → 3-tier recommendation JSON returned |
| Add `/api/freeco/agents/secretary/message` endpoint | `openfang-api` | Secretary correctly routes: shopping intent → shopping agent, general → LLM reply |
| Add `/api/freeco/agents/ceo/message` endpoint | `openfang-api` | CEO executive routing produces `CeoDecision` with `Action` list |
| Freeco tab in dashboard | `crates/openfang-api/static/index_body.html`, `static/js/` | Dashboard shows Freeco agent list and chat interface |
| Novita AI named LLM driver | `openfang-runtime/src/drivers/` | `NOVITA_API_KEY` env var; Gemma 4 26B resolves correctly in model catalog |

### Wave 3 — Peripheral (WASM / billing / i18n / docs / automation)

**Goal:** Platform is deployment-ready with monetization, edge delivery, and full documentation.

| Task | Crates touched | Acceptance gate |
|------|----------------|-----------------|
| `wasm-pack` build step in CI | `.github/workflows/ci.yml` | `wasm-pack build --target web` succeeds for all `cdylib` agent crates |
| Stripe webhook handler | new `crates/freeco-billing` or `openfang-api` route | Tier upgrade from Stripe event → `BudgetEngine::set_tier()` persisted |
| i18n locale wiring | `freeco-input-parser`, `openfang-api/static/i18n/` | Parsing test with FR/DE/CH locale items passes |
| Architecture doc update | `docs/architecture.md` | Freeco crate layer documented alongside openfang layer |
| Archive deprecated repos | External repos | All PRs merged; repos archived; redirect links added |

---

## 5. Acceptance Gates

Every wave must pass all gates before the next wave begins:

### Gate A — Build

```bash
cargo build --workspace --lib
# Expected: zero errors
```

### Gate B — Tests

```bash
cargo test --workspace
# Expected: all tests pass (currently 2,696+)
```

### Gate C — Lint

```bash
cargo clippy --workspace --all-targets -- -D warnings
# Expected: zero warnings
```

### Gate D — Live Integration

For each new endpoint added in the wave:

```bash
# Start daemon
cargo build --release -p openfang-cli
GROQ_API_KEY=<key> NOVITA_API_KEY=<key> target/release/openfang start &
sleep 6
curl -s http://127.0.0.1:4200/api/health   # Must return {"status":"ok"}

# Test new freeco endpoints
curl -s http://127.0.0.1:4200/api/freeco/budget
curl -s -X POST http://127.0.0.1:4200/api/freeco/agents/secretary/message \
  -H "Content-Type: application/json" \
  -d '{"message": "find me organic oat milk"}'
```

### Gate E — Rollback Documented

Before merging each wave:

- Git tag `freeco-wave-N-start` on the last known-good commit.
- Document any SQLite schema migrations added (table name, columns, rollback DDL).
- Confirm daemon can be restarted from the previous tag without data loss.

---

## 6. Operating Model

### Versioning

- Workspace version is shared (`version.workspace = true` in all `freeco-*` `Cargo.toml`).
- Freeco platform and openfang follow **semver** with a single version bump in root `Cargo.toml`.
- No crate publishes to crates.io until v1.0.

### Release Cadence

| Cadence | Trigger |
|---------|---------|
| **Patch** (0.x.y+1) | Bug fixes, dependency updates — CI auto-tag on merge to `main` |
| **Minor** (0.x+1.0) | New agent, new API surface, new dashboard feature |
| **Major** (1.0.0) | Stable API contract, production SLA, billing live |

### Ownership Matrix

| Area | Primary owner | Backup |
|------|--------------|--------|
| `freeco-agent-*` | Agent team | Platform team |
| `freeco-budget-engine` | Backend team | Platform team |
| `freeco-tool-gateway` | Backend team | Agent team |
| `freeco-input-parser` | Agent team | Backend team |
| `freeco-runtime` | Platform team | — |
| `openfang-*` crates | Platform team | — |
| CI / release | DevOps | Platform team |
| Dashboard / UI | UI team | Platform team |

### Branching & PR Policy

- **Main branch:** `main` — must be green CI at all times.
- **Feature branches:** `freeco/<scope>/<short-description>` (e.g. `freeco/api/budget-endpoints`).
- **Wave branches:** `freeco/wave-N` for larger multi-PR waves; squash-merged to `main`.
- All PRs require: CI green + one owner review + no clippy warnings.
- Draft PRs allowed for WIP; add `[WIP]` prefix to title.

### Deprecation Policy

1. Mark old API with `#[deprecated(since = "0.x.0", note = "...")]`.
2. Keep deprecated surface for one minor version after deprecation.
3. Remove in next minor version after the keeping period.
4. For external repos being archived: add a pinned notice pointing to this monorepo; keep repo read-only for 90 days before full archive.

---

## 7. Milestones

### M0 — Governance + Inventory *(current)*

**Target:** All planning artifacts committed; GitHub Project created; team ownership confirmed.

Deliverables (measurable):
- [ ] This document committed to `docs/freeco-platform-plan.md` ✅
- [ ] GitHub Project "Freeco.ai Platform" created under FreecoDAO with 6 columns
- [ ] Migration inventory reviewed and signed off by each crate owner
- [ ] `cargo build --workspace --lib && cargo test --workspace && cargo clippy ... -- -D warnings` all green on `main`
- [ ] Wave 1 issues created and assigned on the project board

### M1 — Core Repo Migration Complete

**Target:** Budget engine, tool gateway, and input parser are live and tested via the Axum API.

Deliverables (measurable):
- [ ] `GET /api/freeco/budget` returns tier + tokens remaining
- [ ] `POST /api/freeco/budget/tier` persists tier change to SQLite
- [ ] `POST /api/freeco/budget/topup` applies a token top-up
- [ ] `ToolGateway` registered in `AppState`; permission denial returns HTTP 403
- [ ] All Wave 1 acceptance gates (A–E) passed and documented
- [ ] Git tag `freeco-wave-1-done` on merging PR

### M2 — Agent Migration Complete

**Target:** All three Freeco agents are callable via REST and visible in the dashboard.

Deliverables (measurable):
- [ ] `POST /api/freeco/agents/shopping/message` returns 3-tier recommendation
- [ ] `POST /api/freeco/agents/secretary/message` routes correctly (shopping vs general)
- [ ] `POST /api/freeco/agents/ceo/message` returns `CeoDecision`
- [ ] Freeco tab live in dashboard with agent list and chat interface
- [ ] Novita AI LLM driver wired and integration-tested (real LLM call made)
- [ ] All Wave 2 acceptance gates passed
- [ ] Git tag `freeco-wave-2-done`

### M3 — Stabilization + Observability + Docs

**Target:** Platform is instrumented, documented, and billing-ready.

Deliverables (measurable):
- [ ] WASM build for all `cdylib` agent crates passes in CI
- [ ] Stripe webhook handler tested end-to-end (sandbox → tier upgrade persisted)
- [ ] `docs/architecture.md` updated with Freeco crate layer
- [ ] All deprecated external repos archived (read-only + redirect notice)
- [ ] Tracing spans on all Freeco API routes (`tracing::instrument`)
- [ ] All Wave 3 acceptance gates passed
- [ ] Git tag `freeco-wave-3-done`

### M4 — Public Launch Readiness

**Target:** Platform ready for paying users: billing live, install script deployed, SLA defined.

Deliverables (measurable):
- [ ] Stripe production webhooks live; first real subscription tier change tested
- [ ] `https://freeco.ai` install script deployed
- [ ] SLA document published (uptime, response time targets)
- [ ] v1.0.0 release tag cut with signed binaries
- [ ] All API endpoints covered by API reference docs (`docs/api-reference.md`)
- [ ] Zero open P0/P1 bugs on the project board

---

## 8. GitHub Project Setup

The GitHub Project board cannot be created via code. Create it manually with these steps:

### Step 1 — Create the project

1. Go to **github.com/orgs/FreecoDAO/projects/new**
2. Choose **"Board"** layout.
3. Name: **`Freeco.ai Platform`**
4. Description: `End-to-end tracking for the Freeco.ai platform — crate migration, API wiring, agent rollout, and launch readiness.`
5. Set visibility to **Private** until M1 is complete, then make **Public**.

### Step 2 — Add columns (tracks)

Create exactly these columns in order:

| Column | Purpose |
|--------|---------|
| **Backlog** | All planned issues not yet scheduled |
| **Ready** | Issues with a clear spec, assigned owner, and no blockers |
| **In Progress** | Actively being worked on (limit: 3 per person) |
| **Review** | PR open, awaiting review or CI |
| **Done** | Merged and gates passed |
| **Risks** | Blocked issues, open questions, dependency blockers |

### Step 3 — Add milestone issues

Create one issue per milestone sub-task. Label each with:
- `milestone: M0` through `milestone: M4`
- `wave: 1` / `wave: 2` / `wave: 3` (where applicable)
- `area: runtime` / `area: agent` / `area: api` / `area: ui` / `area: infra`

### Step 4 — Link this document

Pin a link to `docs/freeco-platform-plan.md` (or a rendered GitHub Pages URL) in the project description so every contributor can find the current plan.

---

## 9. Weekly Review Cadence

Run a 30-minute review every Monday until all external repos are cut over and M3 is closed.

**Agenda (fixed):**

1. **Board scan** (5 min) — move any completed items to Done; move any blocked items to Risks.
2. **Wave gate check** (10 min) — run `cargo test --workspace` and `cargo clippy ... -- -D warnings`; confirm green or assign fix.
3. **Risk review** (10 min) — triage items in the Risks column; assign owners and due dates.
4. **Next-week commitments** (5 min) — each area owner states exactly which issues move from Backlog → Ready or Ready → In Progress.

**Escalation:** Any item in Risks for more than two consecutive reviews is escalated to the project owner for unblocking.

**Close criteria:** Review cadence ends when:
- M3 tag is cut (`freeco-wave-3-done`)
- Risks column is empty
- No external repos remain un-archived

---

*This document is the single source of truth for the Freeco.ai platform working plan. All milestone tracking happens on the GitHub Project board. Update this document when scope, ownership, or milestone targets change.*
