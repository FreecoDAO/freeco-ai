<p align="center">
  <img src="https://github.com/user-attachments/assets/7c67fbb2-e0c9-4634-9cd0-3d78a02027ef" width="160" alt="FreEco.ai Logo" />
</p>

<h1 align="center">FreEco.ai</h1>
<h3 align="center">The Sustainable Ethical Agentic Operating System</h3>

<p align="center">
  Open-source Agent OS built in Rust. 207K Rust LOC. 22 crates plus xtask. 2,846+ tests. Zero clippy warnings.<br/>
  <strong>One binary. Battle-tested. Agents that work for you, your business, and your family.</strong>
</p>

# FRE.ECO AI Node & Marketplace Roadmap

## Executive Architecture (v1.0)

### Vision

FRE.ECO will evolve into a federated AI commerce platform based on self-sovereign user nodes.

Instead of building a traditional cloud-first marketplace, FRE.ECO will first deliver a local AI-powered node ("FRE.ECO Node") running on users' devices. The cloud marketplace will then become a coordination, synchronization, and intelligence layer rather than the primary execution environment.

At the core of the system is **FreEco.ai**, which ships as a **single binary** that you configure for your use case. The same runtime can be assembled into different product experiences depending on the target audience.

FreEco.ai is built on and evolved from the mature, most reliable, advanced, and secure open-source AI Agentic OS **OpenFang**, which advanced the trending **OpenClaw** agentic community into a highly reliable Agentic OS with **16 security levels** and **Ethical Ecological guidelines**. This foundation gives FRE.ECO a strong, trusted, and extensible AI operating base for secure local execution, agent orchestration, and responsible ecosystem growth.

The four supported assembly types are:

| Edition                                                                                                           | Who It's For           | What It Does                                                                                                                                                                                                             |
| ----------------------------------------------------------------------------------------------------------------- | ---------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| 🧑‍💼 **Personal sustainable Free Eco AI Concierge, Personal, work, travel, shopping, Health coaching Assistant** | Individuals            | Hi-end Swiss-style assistant for life, work, shopping, and health coaching. Runs 24/7, proactively surfaces insights, and manages your day autonomously.                                                                 |
| 🧒 **Kids Edition**                                                                                               | Families & schools     | Ethical, ecological child-safe AI companion for study, daily life, and online security. Strictly guardrailed, COPPA-aligned, with parental visibility built in.                                                          |
| 🏢 **Free Eco Business Suite**                                                                                    | Companies & teams      | Autonomous sustainable buyer and executive-agent team for restaurants, shops, businesses, and non-profits: lead generation, competitor intelligence, scheduling, reporting. Agent-to-agent handoffs with approval gates. |
| 🤖 **Free Eco AI Company for running Sustainable businesses, charities, nonprofits**                              | Power users & builders | A self-running, fully autonomous AI company on your device. Spin up specialized agent teams that plan, delegate, execute, and report — end-to-end, no human in the loop.                                                 |

The system combines:

* FreEco.ai (AI kernel)
* FRE.ECO Node
* FRE.ECO Marketplace
* FRE.ECO MCP Gateway
* FRE.ECO DAO
* FRE.ECO Wallet
* FRE.ECO AI Shopping Assistant

---

# Phase 1 — FRE.ECO Node

Deliver native applications for:

* Windows
* macOS
* Linux
* iPhone
* Android

The user installs one application exactly like a crypto wallet.

No Docker knowledge is required.

Each node contains:

* FreEco.ai
* AI Shopping Assistant
* Local encrypted database
* Local vector memory
* Wallet
* Personal agents
* Synchronization engine
* MCP client
* Automatic updater

The node works online and offline.

The FreEco.ai binary is configured at install time or first launch for the selected edition:

* Personal sustainable AI Concierge
* Kids Edition
* Free Eco Business Suite
* Free Eco AI Company

This keeps the runtime unified while allowing the product experience to be tailored to the user’s needs.

---

# Phase 2 — Agent Marketplace

Introduce a signed Agent Store.

Examples include:

* Shopping Agent
* Travel Agent
* Health Information Agent
* Legal Information Agent
* Sustainability Agent
* Carbon Auditor
* DAO Governance Agent
* Personal Secretary

Every agent is cryptographically signed, sandboxed, and permission-based.

Agents can be installed, enabled, disabled, or upgraded independently of the core FreEco.ai binary.

---

# Phase 3 — FRE.ECO MCP Gateway

Deploy a centralized and extensible MCP Gateway exposing secure marketplace tools.

Initial MCP services:

* Product search
* Eco verification
* Marketplace search
* Seller verification
* Wallet operations
* DAO voting
* Shipping quotations
* Carbon footprint estimation
* Translation
* Personal memory retrieval

The Gateway becomes the standard interface between AI agents and FRE.ECO services.

It allows the same FreEco.ai runtime to interact with local tools, cloud services, and third-party systems through a consistent protocol layer.

---

# Phase 4 — Cloud Infrastructure

Deploy scalable cloud services using Docker and Kubernetes.

Services include:

* Marketplace APIs
* Authentication
* Payments
* AI orchestration
* Vector databases
* Search indexing
* Analytics
* Notification services

Docker is used for packaging and deployment.

Kubernetes provides scaling, resilience, and automated operations.

The cloud layer supports synchronization, trust, and coordination, while the local FreEco.ai binary remains the primary execution environment for the user.

---

# Phase 5 — Federated AI Commerce Network

Thousands of FRE.ECO Nodes synchronize securely.

Cloud services coordinate:

* Marketplace
* Global product catalog
* Shared AI knowledge
* Reputation
* Governance
* Payments

Users retain ownership of:

* Identity
* Wallet keys
* Personal memory
* Preferences
* Local AI

This creates a federated network where each node is sovereign, but all nodes can participate in a shared economic and intelligence ecosystem.

---

# Phase 6 — Self-Running AI Company (Execution Team)

The long-term goal is a **self-running, self-improving AI company** with a full organizational structure of agents — and humans where a human is genuinely needed — that develops, markets, and supports FreEco.ai itself (and, as a product, any business the user runs).

**What exists today (the engine):**

* A real multi-agent **workflow engine** (`Workflow` → `WorkflowStep` → `WorkflowRun`, sequential and parallel) — the ChatDev-style phase pipeline.
* A **CEO agent** that turns directives into delegation plans, plus specialist agents (developer/coder, code-reviewer, security-auditor, analyst, secretary, autonomous buyer).
* **Approval gates** (human-in-the-loop), a **budget engine** (spend caps per agent), the **Docker execution sandbox**, and inter-agent messaging.

**What Phase 6 adds (the company):**

* **Org structure as a first-class concept** — a defined role hierarchy (CEO → CTO/CMO/COO → developer, tester/ethical-hacker, marketer, support, accountant), each role a signed agent with scoped permissions, composed into an "AI Company" chart.
* **Self-development loop** — the CEO compiles inputs (roadmap, marketing signals, user feedback, metrics, the issue backlog), proposes **prioritized developments and actions**, and delegates to a developer agent (writes code in a sandboxed repo clone) and a tester/ethical-hacker agent (build, test, `cargo audit`, security review). Every change goes through an approval gate → PR → CI → merge. Human in the loop, always.
* **Hire agents or humans on demand** — spawn a specialist agent for a task, or hand a task to a human via a task queue when judgment, credentials, or accountability require it.
* **Business back office** — CRM integration, accounting/ledger and database, metrics and analytics, plus website and social-media development and support — so the company can actually run operations, not just write code.

**Phasing (deliberate — scope beats ambition):**

1. **Dev pod** — developer + tester + ethical-hacker agents work labeled GitHub issues inside the sandbox; human merges everything. (Foundation already ~80% present.)
2. **CEO triage** — the CEO prioritizes the backlog against the roadmap and assigns work.
3. **Signal-driven** — marketing/metrics/feedback feed the CEO's priorities; back-office integrations (CRM, accounting, metrics) come online.
4. **Full company** — coordinated agent+human org running development, marketing, and support with human oversight at the gates.

This is the "AI Company" edition pointed at FreEco.ai itself — the ultimate dogfood.

---

# AI Architecture

FreEco.ai acts as the AI kernel.

Responsibilities:

* Task orchestration
* Agent lifecycle
* Scheduling
* Memory management
* MCP integration
* Security
* Local execution

FreEco.ai inherits the mature Agentic OS foundation of OpenFang, including its 16 security levels and ethical ecological operating principles. This makes the runtime suitable for trusted autonomous execution, responsible agent behavior, and secure multi-edition deployment.

FRE.ECO adds:

* Marketplace logic
* Commerce agents
* DAO governance
* Wallet integration
* Sustainability services
* User experience

FreEco.ai is the single binary foundation. FRE.ECO defines the product layers, editions, policies, and ecosystem around it.

---

# Security Principles

* Private keys never leave the user's device.
* Personal memories remain encrypted.
* Agents execute with least-privilege permissions.
* Every installable agent is signed.
* Cloud services receive only authenticated and authorized requests.
* Docker containers are used only where appropriate in server infrastructure.
* The local FreEco.ai binary is configured securely for the selected edition and use case.
* The system follows the 16 security levels and Ethical Ecological guidelines inherited from the OpenFang Agentic OS foundation.

## Planned: Self-Contained Isolation Layer

Today, agent browsing and code execution borrow tools from the host machine: browser automation drives an installed Chrome/Chromium via CDP, and the OS-level sandbox uses the host's Docker. Two roadmap items will remove those dependencies and add a further security layer, giving a Manus-style "everything included" experience:

* **Bundled agent browser** — ship a dedicated Chromium build inside the FRE.ECO Node. Agents browse in a fully separate browser with its own empty profile, so they never touch the user's own browser, saved passwords, cookies, or sessions — and browser automation works even on machines where only Firefox is installed.
* **Prebuilt sandbox image** — ship a ready-made, minimal Linux execution environment with the Node, so agent code runs in an isolated sandbox out of the box, without requiring the user to install or understand Docker.
* **Built-in local AI** — a one-click "free local AI, no account needed" setup that fetches a lightweight model (e.g. Gemma) and runs it on-device, so every fresh install has a private, working assistant before any API key is entered; longer term, the inference engine ships inside the binary itself.

Both layers keep the existing WASM, subprocess, and workspace sandboxes underneath — isolation is added, never traded away.

---

# Long-Term Objective

Create a sovereign, AI-native, privacy-first economic operating environment where users own their identity, data, and AI while participating in a globally scalable sustainable marketplace.
<p align="center">
  <a href="https://freeco.ai/docs">Documentation</a> &bull;
  <a href="https://freeco.ai/docs/getting-started">Quick Start</a> &bull;
  <a href="https://x.com/FreEcoAI">Twitter / X</a>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/language-Rust-orange?style=flat-square" alt="Rust" />
  <img src="https://img.shields.io/badge/license-MIT-blue?style=flat-square" alt="MIT" />
  <img src="https://img.shields.io/badge/version-0.7.4-green?style=flat-square" alt="v0.7.4" />
  <img src="https://img.shields.io/badge/tests-2,696%2B%20passing-brightgreen?style=flat-square" alt="Tests" />
  <img src="https://img.shields.io/badge/clippy-0%20warnings-brightgreen?style=flat-square" alt="Clippy" />
  <a href="https://www.buymeacoffee.com/openfang" target="_blank"><img src="https://img.shields.io/badge/Buy%20Me%20a%20Coffee-FFDD00?style=flat-square&logo=buy-me-a-coffee&logoColor=black" alt="Buy Me A Coffee" /></a>
</p>

---

## ⬇️ Download & Install

> Choose a **desktop installer** for your operating system. The
> `openfang-<target>.zip` and `.tar.gz` files are advanced CLI archives, not
> desktop installers. The portable USB bundle is a separate download.

### 🪟 Windows — step by step

1. Open the **[latest release](https://github.com/FreecoDAO/freeco-ai/releases/latest)** and download the
   `FreEco.ai_*_x64-setup.exe` installer (64-bit — for almost every PC). On
   an ARM tablet, download the `FreEco.ai_*_arm64-setup.exe` installer instead.
2. **Double-click** the downloaded file.
3. Windows may show a blue **"Windows protected your PC"** box (because the
   app is new and not yet code-signed). Click **More info → Run anyway**.
   This is expected and safe.
4. Click through the installer. It creates a **FreEco.ai shortcut on your
   Desktop** and in the Start Menu.
5. **Double-click the Desktop shortcut** to launch. The dashboard opens
   automatically at `http://localhost:4200`.
6. On first launch a short setup wizard asks for an AI provider key — or
   pick **"Free local AI (no account)"** to run entirely on your machine.

### 🍎 macOS

Download the **[.dmg](https://github.com/FreecoDAO/freeco-ai/releases/latest)**,
open it, drag **FreEco.ai** to Applications. First launch: **right-click the
app → Open** (needed once, because it isn't Apple-signed yet).

### 🐧 Linux

Download the **[.AppImage](https://github.com/FreecoDAO/freeco-ai/releases/latest)**
(`chmod +x` then double-click), or the **`.deb`** (`sudo apt install ./FreEco.ai_*.deb`).

### 🔌 Portable USB and Kubuntu live USB

The **portable USB bundle** runs the matching Windows, macOS, or Linux binary
from a `freeco-portable` folder while keeping all FreEco.ai data on the drive.
For maximum host isolation, write a verified official Kubuntu LTS live ISO
using [`scripts/kubuntu-usb.sh`](scripts/kubuntu-usb.sh), then run the portable
bundle inside Kubuntu. A signed FreEco-customized Kubuntu image is not yet
available, so no such image is offered as a download.

👉 **[All downloads](https://github.com/FreecoDAO/freeco-ai/releases/latest)** — choose a Windows `-setup.exe` (x64 or ARM64), macOS `.dmg` (Intel or Apple Silicon), Linux `.AppImage`/`.deb`, or a portable bundle. Use CLI archives only for command-line deployments.

---

> **Latest release: v0.7.4 (July 2026)**
>
> FreEco.ai is feature complete but still pre-1.0. Expect rough edges and breaking changes between minor versions. We ship fast and fix fast. Pin to a specific commit for production use until v1.0. [Report issues here.](https://github.com/FreecoDAO/freeco-ai/issues)
>
> The next `main` release includes the completed rand 0.9 migration, a Kubuntu
> live-USB helper, and release-workflow hardening. These changes are validated
> before being tagged and published.

---

## What is FreEco.ai?

FreEco.ai is a SUSTAINABLE ETHICAL **open-source Agent Operating System**. Not a chatbot framework. Not a Python wrapper around an LLM. Not a "multi-agent orchestrator." A full operating system for autonomous agents, with ethical and structural enhancements for personal use, business, family and kids.

Traditional agent frameworks wait for you to type something. FreEco.ai runs **autonomous agents that work for you**: on schedules, 24/7, building knowledge graphs, monitoring targets, generating leads, managing your social media, and reporting results to your dashboard.

The entire system compiles to a **single ~32MB binary**. One install, one command, your agents are live.

```bash
curl -fsSL https://raw.githubusercontent.com/FreecoDAO/freeco-ai/main/scripts/install.sh | sh
openfang init
openfang start
# Dashboard live at http://localhost:4200
```

<details>
<summary><strong>Windows</strong></summary>

```powershell
irm https://raw.githubusercontent.com/FreecoDAO/freeco-ai/main/scripts/install.ps1 | iex
openfang init
openfang start
```

</details>

<details>
<summary><strong>Running from Source (Linux / macOS / Windows)</strong></summary>

**Prerequisites:** [Rust 1.75+](https://rustup.rs/) and at least one LLM API key.

```bash
git clone https://github.com/FreecoDAO/freeco-ai.git
cd freeco-ai
cargo build --release -p openfang-cli
./target/release/openfang init          # creates ~/.openfang/config.toml
export GROQ_API_KEY=gsk_...             # or ANTHROPIC_API_KEY / OPENAI_API_KEY
./target/release/openfang start
# Dashboard live at http://localhost:4200
```

On Windows use `target\release\openfang.exe`. See [docs/getting-started.md](docs/getting-started.md) for the full guide.

</details>

---

## Four Editions of FreEco.ai

FreEco.ai ships as a single binary that you configure for your use case. The four supported assembly types are:

| Edition | Who It's For | What It Does |
|---------|-------------|--------------|
| 🧑‍💼 **Personal sustainable Free Eco AI Concierge, Personal, work, travel, shopping, Health coaching Assistant ** | Individuals | Hi-end Swiss-style assistant for life, work, shopping, and health coaching. Runs 24/7, proactively surfaces insights, and manages your day autonomously. |
| 🧒 **Kids Edition** | Families & schools | Ethical, Ecoligical child-safe AI companion for study, daily life, and online security. Strictly guardrailed, COPPA-aligned, with parental visibility built in. |
| 🏢 **Free Eco Business Suite** | Companies & teams | Autonomous sustainable buyer and executive-agent team for restaurants, shpops, business and non-profits: lead generation, competitor intelligence, scheduling, reporting. Agent-to-agent handoffs with approval gates. |
| 🤖 **Free Eco AI Company for running Sustainable businesses, charities, nonprofits  ** | Power users & builders | A self-running, fully autonomous AI company on your device. Spin up specialized agent teams that plan, delegate, execute, and report — end-to-end, no human in the loop. |

You can also import skills from [ClawHub](https://clawhub.ai), which are staged and security-scanned before installation, and create custom agents using `agent.toml` manifests. See [Spawn Your First Agent](docs/getting-started.md#spawn-your-first-agent).

---

## Hands: Agents That Actually Do Things

<p align="center"><em>"Traditional agents wait for you to type. Freeco.ai makes AI work for you, your business, Community, Charity. Hands work <strong>for</strong> you."</em></p>

**Hands** are FreEco.ai's core innovation. Pre-built autonomous capability packages that run independently, on schedules, without you having to prompt them. This is not a chatbot. This is an agent that wakes up at 6 AM, researches your competitors, builds a knowledge graph, scores the findings, and delivers a report to your Telegram before you've had coffee.

Each Hand bundles:
- **HAND.toml**: manifest declaring tools, settings, requirements, and dashboard metrics.
- **System Prompt**: multi-phase operational playbook. Not a one-liner. These are 500+ word expert procedures.
- **SKILL.md**: domain expertise reference injected into context at runtime.
- **Guardrails**: approval gates for sensitive actions (e.g. Browser Hand requires approval before any purchase).

All compiled into the binary. No downloading, no pip install, no Docker pull.

### The 7 Bundled Hands

| Hand | What It Actually Does |
|------|----------------------|
| **Clip** | Takes a YouTube URL, downloads it, identifies the best moments, cuts them into vertical shorts with captions and thumbnails, optionally adds AI voice-over, and publishes to Telegram and WhatsApp. 8-phase pipeline. FFmpeg + yt-dlp + 5 STT backends. |
| **Lead** | Runs daily. Discovers prospects matching your ICP, enriches them with web research, scores 0-100, deduplicates against your existing database, and delivers qualified leads in CSV/JSON/Markdown. Builds ICP profiles over time. |
| **Collector** | OSINT grade intelligence. You give it a target (company, person, topic). It monitors continuously: change detection, sentiment tracking, knowledge graph construction, and critical alerts when something important shifts. |
| **Predictor** | Superforecasting engine. Collects signals from multiple sources, builds calibrated reasoning chains, makes predictions with confidence intervals, and tracks its own accuracy using Brier scores. Has a contrarian mode that deliberately argues against consensus. |
| **Researcher** | Deep autonomous researcher. Cross-references multiple sources, evaluates credibility using CRAAP criteria (Currency, Relevance, Authority, Accuracy, Purpose), generates cited reports with APA formatting, supports multiple languages. |
| **Twitter** | Autonomous Twitter/X account manager. Creates content in 7 rotating formats, schedules posts for optimal engagement, responds to mentions, tracks performance metrics. Has an approval queue, so nothing posts without your OK. |
| **Browser** | Web automation agent. Navigates sites, fills forms, clicks buttons, handles multi-step workflows. Uses Playwright bridge with session persistence. **Mandatory purchase approval gate**: it will never spend your money without explicit confirmation. |

```bash
# Activate the Researcher Hand. It starts working immediately.
openfang hand activate researcher

# Check its progress anytime
openfang hand status researcher

# Activate lead generation on a daily schedule
openfang hand activate lead

# Pause without losing state
openfang hand pause lead

# See all available Hands
openfang hand list
```

**Build your own.** Define a `HAND.toml` with tools, settings, and a system prompt. Publish to FangHub.

---

## FreEco.ai vs The Landscape

<p align="center">
  <img src="public/assets/openfang-vs-claws.png" width="600" alt="FreEco.ai vs OpenClaw vs ZeroClaw" />
</p>

### Benchmarks: Measured, Not Marketed

All data from official documentation and public repositories, February 2026.

#### Cold Start Time (lower is better)

```
ZeroClaw   ██░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░   10 ms
FreEco.ai   ██████░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░  180 ms    ★
LangGraph  █████████████████░░░░░░░░░░░░░░░░░░░░░░░░░  2.5 sec
CrewAI     ████████████████████░░░░░░░░░░░░░░░░░░░░░░  3.0 sec
AutoGen    ██████████████████████████░░░░░░░░░░░░░░░░░  4.0 sec
OpenClaw   █████████████████████████████████████████░░  5.98 sec
```

#### Idle Memory Usage (lower is better)

```
ZeroClaw   █░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░    5 MB
FreEco.ai   ████░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░   40 MB    ★
LangGraph  ██████████████████░░░░░░░░░░░░░░░░░░░░░░░░░  180 MB
CrewAI     ████████████████████░░░░░░░░░░░░░░░░░░░░░░░  200 MB
AutoGen    █████████████████████████░░░░░░░░░░░░░░░░░░  250 MB
OpenClaw   ████████████████████████████████████████░░░░  394 MB
```

#### Install Size (lower is better)

```
ZeroClaw   █░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░  8.8 MB
FreEco.ai   ███░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░   32 MB    ★
CrewAI     ████████░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░  100 MB
LangGraph  ████████████░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░  150 MB
AutoGen    ████████████████░░░░░░░░░░░░░░░░░░░░░░░░░░░  200 MB
OpenClaw   ████████████████████████████████████████░░░░  500 MB
```

#### Security Systems (higher is better)

```
FreEco.ai   ████████████████████████████████████████████   16      ★
ZeroClaw   ███████████████░░░░░░░░░░░░░░░░░░░░░░░░░░░░    6
OpenClaw   ████████░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░    3
AutoGen    █████░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░    2
LangGraph  █████░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░    2
CrewAI     ███░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░    1
```

#### Channel Adapters (higher is better)

```
FreEco.ai   ████████████████████████████████████████████   40      ★
ZeroClaw   ███████████████░░░░░░░░░░░░░░░░░░░░░░░░░░░░   15
OpenClaw   █████████████░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░   13
CrewAI     ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░    0
AutoGen    ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░    0
LangGraph  ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░    0
```

#### LLM Providers (higher is better)

```
ZeroClaw   ████████████████████████████████████████████   28
FreEco.ai   ██████████████████████████████████████████░░   27      ★
LangGraph  ██████████████████████░░░░░░░░░░░░░░░░░░░░░   15
CrewAI     ██████████████░░░░░░░░░░░░░░░░░░░░░░░░░░░░░   10
OpenClaw   ██████████████░░░░░░░░░░░░░░░░░░░░░░░░░░░░░   10
AutoGen    ███████████░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░    8
```

### Feature-by-Feature Comparison

| Feature | FreEco.ai | OpenClaw | ZeroClaw | CrewAI | AutoGen | LangGraph |
|---------|----------|----------|----------|--------|---------|-----------|
| **Language** | **Rust** | TypeScript | **Rust** | Python | Python | Python |
| **Autonomous Hands** | **7 built-in** | None | None | None | None | None |
| **Security Layers** | **16 discrete** | 3 basic | 6 layers | 1 basic | Docker | AES enc. |
| **Agent Sandbox** | **WASM dual-metered** | None | Allowlists | None | Docker | None |
| **Channel Adapters** | **40** | 13 | 15 | 0 | 0 | 0 |
| **Built-in Tools** | **53 + MCP + A2A** | 50+ | 12 | Plugins | MCP | LC tools |
| **Memory** | **SQLite + vector** | File-based | SQLite FTS5 | 4-layer | External | Checkpoints |
| **Desktop App** | **Tauri 2.0** | None | None | None | Studio | None |
| **Audit Trail** | **Merkle hash-chain** | Logs | Logs | Tracing | Logs | Checkpoints |
| **Cold Start** | **<200ms** | ~6s | ~10ms | ~3s | ~4s | ~2.5s |
| **Install Size** | **~32 MB** | ~500 MB | ~8.8 MB | ~100 MB | ~200 MB | ~150 MB |
| **License** | MIT | MIT | MIT | MIT | Apache 2.0 | MIT |

---

## 16 Security Systems: Defense in Depth

FreEco.ai doesn't bolt security on after the fact. Every layer is independently testable and operates without a single point of failure.

| # | System | What It Does |
|---|--------|-------------|
| 1 | **WASM Dual-Metered Sandbox** | Tool code runs in WebAssembly with fuel metering + epoch interruption. A watchdog thread kills runaway code. |
| 2 | **Merkle Hash-Chain Audit Trail** | Every action is cryptographically linked to the previous one. Tamper with one entry and the entire chain breaks. |
| 3 | **Information Flow Taint Tracking** | Labels propagate through execution. Secrets are tracked from source to sink. |
| 4 | **Ed25519 Signed Agent Manifests** | Every agent identity and capability set is cryptographically signed. |
| 5 | **SSRF Protection** | Blocks private IPs, cloud metadata endpoints, and DNS rebinding attacks. |
| 6 | **Secret Zeroization** | `Zeroizing<String>` auto-wipes API keys from memory the instant they're no longer needed. |
| 7 | **OFP Mutual Authentication** | HMAC-SHA256 nonce-based, constant-time verification for P2P networking. |
| 8 | **Capability Gates** | Role based access control. Agents declare required tools, the kernel enforces it. |
| 9 | **Security Headers** | CSP, X-Frame-Options, HSTS, X-Content-Type-Options on every response. |
| 10 | **Health Endpoint Redaction** | Public health check returns minimal info. Full diagnostics require authentication. |
| 11 | **Subprocess Sandbox** | `env_clear()` + selective variable passthrough. Process tree isolation with cross-platform kill. |
| 12 | **Prompt Injection Scanner** | Detects override attempts, data exfiltration patterns, and shell reference injection in skills. |
| 13 | **Loop Guard** | SHA256-based tool call loop detection with circuit breaker. Handles ping-pong patterns. |
| 14 | **Session Repair** | 7-phase message history validation and automatic recovery from corruption. |
| 15 | **Path Traversal Prevention** | Canonicalization with symlink escape prevention. ``../`` doesn't work here. |
| 16 | **GCRA Rate Limiter** | Cost-aware token bucket rate limiting with per-IP tracking and stale cleanup. |

---

## Architecture

14 Rust crates. 137,728 lines of code. Modular kernel design.

```
openfang-kernel      Orchestration, workflows, metering, RBAC, scheduler, budget tracking
openfang-runtime     Agent loop, 3 LLM drivers, 53 tools, WASM sandbox, MCP, A2A
openfang-api         140+ REST/WS/SSE endpoints, OpenAI-compatible API, dashboard
openfang-channels    40 messaging adapters with rate limiting, DM/group policies
openfang-memory      SQLite persistence, vector embeddings, canonical sessions, compaction
openfang-types       Core types, taint tracking, Ed25519 manifest signing, model catalog
openfang-skills      61 bundled skills, SKILL.md parser, FangHub marketplace
openfang-hands       7 autonomous Hands, HAND.toml parser, lifecycle management
openfang-extensions  25 MCP templates, AES-256-GCM credential vault, OAuth2 PKCE
openfang-wire        OFP P2P protocol with HMAC-SHA256 mutual authentication
openfang-cli         CLI with daemon management, TUI dashboard, MCP server mode
openfang-desktop     Tauri 2.0 native app (system tray, notifications, global shortcuts)
openfang-migrate     OpenClaw, LangChain, AutoGPT migration engine
xtask                Build automation
```

---

## 40 Channel Adapters

Connect your agents to every platform your users are on.

**Core:** Telegram, Discord, Slack, WhatsApp, Signal, Matrix, Email (IMAP/SMTP)
**Enterprise:** Microsoft Teams, Mattermost, Google Chat, Webex, Feishu/Lark, Zulip
**Social:** LINE, Viber, Facebook Messenger, Mastodon, Bluesky, Reddit, LinkedIn, Twitch
**Community:** IRC, XMPP, Guilded, Revolt, Keybase, Discourse, Gitter
**Privacy:** Threema, Nostr, Mumble, Nextcloud Talk, Rocket.Chat, Ntfy, Gotify
**Workplace:** Pumble, Flock, Twist, DingTalk, Zalo, Webhooks

Each adapter supports per-channel model overrides, DM/group policies, rate limiting, and output formatting.

---

## WhatsApp Web Gateway (QR Code)

Connect your personal WhatsApp account to FreEco.ai via QR code, just like WhatsApp Web. No Meta Business account required.

### Prerequisites

- **Node.js >= 18** installed ([download](https://nodejs.org/))
- FreEco.ai installed and initialized

### Setup

**1. Install the gateway dependencies:**

```bash
cd packages/whatsapp-gateway
npm install
```

**2. Configure `config.toml`:**

```toml
[channels.whatsapp]
mode = "web"
default_agent = "assistant"
```

**3. Set the gateway URL (choose one):**

Add to your shell profile for persistence:

```bash
# macOS / Linux
echo 'export WHATSAPP_WEB_GATEWAY_URL="http://127.0.0.1:3009"' >> ~/.zshrc
source ~/.zshrc
```

Or set it inline when starting the gateway:

```bash
export WHATSAPP_WEB_GATEWAY_URL="http://127.0.0.1:3009"
```

**4. Start the gateway:**

```bash
node packages/whatsapp-gateway/index.js
```

The gateway listens on port `3009` by default. Override with `WHATSAPP_GATEWAY_PORT`.

**5. Start FreEco.ai:**

```bash
openfang start
# Dashboard at http://localhost:4200
```

**6. Scan the QR code:**

Open the dashboard → **Channels** → **WhatsApp**. A QR code will appear. Scan it with your phone:

> **WhatsApp** → **Settings** → **Linked Devices** → **Link a Device**

Once scanned, the status changes to `connected` and incoming messages are routed to your configured agent.

### Gateway Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `WHATSAPP_WEB_GATEWAY_URL` | Gateway URL for FreEco.ai to connect to | _(empty = disabled)_ |
| `WHATSAPP_GATEWAY_PORT` | Port the gateway listens on | `3009` |
| `OPENFANG_URL` | FreEco.ai API URL the gateway reports to | `http://127.0.0.1:4200` |
| `OPENFANG_DEFAULT_AGENT` | Agent that handles incoming messages | `assistant` |

### Gateway API Endpoints

| Method | Route | Description |
|--------|-------|-------------|
| `POST` | `/login/start` | Generate QR code (returns base64 PNG) |
| `GET` | `/login/status` | Connection status (`disconnected`, `qr_ready`, `connected`) |
| `POST` | `/message/send` | Send a message (`{ "to": "5511999999999", "text": "Hello" }`) |
| `GET` | `/health` | Health check |

### Alternative: WhatsApp Cloud API

For production workloads, use the [WhatsApp Cloud API](https://developers.facebook.com/docs/whatsapp/cloud-api) with a Meta Business account. See the [Cloud API configuration docs](https://freeco.ai/docs/channels/whatsapp).



---

## 27 LLM Providers, 123+ Models

3 native drivers (Anthropic, Gemini, OpenAI-compatible) route to 27 providers:

Anthropic, Gemini, OpenAI, Groq, DeepSeek, OpenRouter, Together, Mistral, Fireworks, Cohere, Perplexity, xAI, AI21, Cerebras, SambaNova, HuggingFace, Replicate, Ollama, vLLM, LM Studio, Qwen, MiniMax, Zhipu, Moonshot, Qianfan, Bedrock, and more.

Intelligent routing with task complexity scoring, automatic fallback, cost tracking, and per-model pricing.

### Tune an agent in the dashboard

Open an agent's **Config** tab to view and update every model setting: provider,
model, system prompt, output-token limit, temperature, API-key environment
variable, and provider base URL. Changes persist across restarts and apply to
the agent's next request.

---

## Migrate from OpenClaw

Already running OpenClaw? One command:

```bash
# Migrate everything: agents, memory, skills, configs.
openfang migrate --from openclaw

# Migrate from a specific path
openfang migrate --from openclaw --path ~/.openclaw

# Dry run first to see what would change
openfang migrate --from openclaw --dry-run
```

The migration engine imports your agents, conversation history, skills, and configuration. FreEco.ai reads SKILL.md natively and is compatible with the ClawHub marketplace.

---

## OpenAI-Compatible API

Drop-in replacement. Point your existing tools at FreEco.ai:

```bash
curl -X POST localhost:4200/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "researcher",
    "messages": [{"role": "user", "content": "Analyze Q4 market trends"}],
    "stream": true
  }'
```

140+ REST/WS/SSE endpoints covering agents, memory, workflows, channels, models, skills, A2A, Hands, and more.

---

## Quick Start

```bash
# 1. Install (macOS/Linux)
curl -fsSL https://raw.githubusercontent.com/FreecoDAO/freeco-ai/main/scripts/install.sh | sh

# 2. Initialize. Walks you through provider setup.
openfang init

# 3. Start the daemon
openfang start

# 4. Dashboard is live at http://localhost:4200

# 5. Activate a Hand. It starts working for you.
openfang hand activate researcher

# 6. Chat with an agent
openfang chat researcher
> "What are the emerging trends in AI agent frameworks?"

# 7. Spawn a pre-built agent
openfang agent spawn coder
```

<details>
<summary><strong>Windows (PowerShell)</strong></summary>

```powershell
irm https://raw.githubusercontent.com/FreecoDAO/freeco-ai/main/scripts/install.ps1 | iex
openfang init
openfang start
```

</details>

---

## Development

```bash
# Build the workspace
cargo build --workspace --lib

# Run all tests (2,846+)
cargo test --workspace

# Lint (must be 0 warnings)
cargo clippy --workspace --all-targets -- -D warnings

# Format
cargo fmt --all -- --check
```

---

## Stability Notice

FreEco.ai v0.5.10 is pre-1.0. The architecture is solid, the test suite is comprehensive, and the security model is deep. That said:

- **Breaking changes** may occur between minor versions until v1.0.
- **Some Hands** are more mature than others. Browser and Researcher are the most battle tested.
- **Edge cases** exist. If you find one, [open an issue](https://github.com/FreecoDAO/FreEco-ai/issues).
- **Pin to a specific commit** for production deployments until v1.0.

We ship fast and fix fast. The goal is a rock solid v1.0 by mid 2026.

---

## Security

To report a security vulnerability, email **freeco.ch@proton.me**. We take all reports seriously and will respond within 48 hours.

---

## License

MIT. Use it Good of the People, Earth, Life

---

## Links

- [Website & Documentation](https://freeco.ai)
- [Quick Start Guide](https://freeco.ai/docs/getting-started)
- [GitHub](https://github.com/FreecoDAO/FreEco-ai)
- [Twitter / X](https://x.com/FreEcoAI)

---

## Built by FreEco.ai for the Earth and Free Eco People

<p align="center">
  <a href="https://www.freeco.ai/">
    <img src="public/assets/rightnow-logo.webp" width="60" alt="RightNow Logo" />
  </a>
</p>

<p align="center">
  FreEco.ai 
<p align="center">
  <a href="https://www.freeco.ai/">Website</a> &bull;

  <a href="https://www.buymeacoffee.com/FreecoDAO/freeco-ai" target="_blank"><img src="https://cdn.buymeacoffee.com/buttons/v2/default-yellow.png" alt="Buy Me A Coffee" style="height: 60px !important;width: 217px !important;" ></a>
</p>

---

<p align="center">
  <strong>Built with Rust. Secured with 16 layers. Agents that actually work for you.</strong>
</p>
