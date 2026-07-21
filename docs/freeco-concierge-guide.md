# Building a Self‑Running Company or Nonprofit with Freeco

*Freeco is your AI concierge — a hi‑end executive assistant, secretary, and operations
mate rolled into one. This guide shows you how to go from an idea to a running
organization where AI agent teams handle planning, communications, a website, sales,
production, development, and accounting — with you approving the important decisions.*

> **Where to start:** open FreEco.ai and click the **✦ Freeco** button in the bottom‑right
> corner of any screen. Tell it — by typing or by holding the mic to talk — what you want
> to build. Freeco routes you to the right builder and walks you through each step.

---

## 1. The big picture

A self‑running organization in FreEco.ai is made of four building blocks:

| Block | What it is | Where in the app |
|-------|------------|------------------|
| **Agents** | Individual AI workers, each with a role, model, and personality | **Agents** page |
| **Teams / workflows** | Agents wired together into a repeatable process | **Workflows** page |
| **Tools & MCP** | The hands agents use — web, files, email, code, external APIs | **Skills / Hands** pages |
| **Channels** | How agents reach the outside world — email, chat, site, webhooks | **Channels** page |

Freeco's job is to help you assemble these into an **organizational chart** that mirrors a
real company or charity, then keep it tuned and running.

---

## 2. A reference org chart for an AI‑run organization

Freeco can help you set up each of these as an agent (or a small team). Start with the ones
you need today and add the rest later.

```
                        ┌───────────────────────┐
                        │  Freeco (Concierge)   │  ← guides you, delegates, reports
                        └──────────┬────────────┘
        ┌──────────────┬──────────┼───────────┬───────────────┬──────────────┐
        │              │          │           │               │              │
   ┌────▼───┐   ┌──────▼───┐ ┌────▼────┐ ┌────▼────┐    ┌──────▼─────┐ ┌──────▼──────┐
   │Strategy│   │Marketing │ │ Sales   │ │Product /│    │Development │ │ Operations  │
   │& Plan  │   │& Content │ │ & CRM   │ │Delivery │    │& Eng       │ │ Finance/ERP │
   └────────┘   └──────────┘ └─────────┘ └─────────┘    └────────────┘ └─────────────┘
   idea,         site, SEO,   leads,       production,    code, infra,   accounting,
   structure,    email,       pipeline,    QA, fulfil.    releases        invoices,
   OKRs          social       proposals                                   reporting
```

### Suggested first five agents
1. **Strategist** — turns your idea into a plan, structure, and OKRs.
2. **Comms/Marketer** — writes your site copy, emails, and social posts.
3. **Sales/CRM** — captures leads, drafts proposals, follows up.
4. **Builder/Developer** — builds the product, site, or automations.
5. **Bookkeeper** — tracks income/expenses and produces simple reports.

> Ask Freeco: *"Create these five agents for a small nonprofit and give each a good
> starter personality and model."*

---

## 3. Step‑by‑step setup

### Step 1 — Turn on an AI brain (model)
- **Free & private:** open **Settings → Local AI** (or ask Freeco *"set up free local AI"*).
  This installs Ollama and pulls a Gemma model that runs on your machine. The installer now
  resumes automatically on slow or flaky connections.
- **Cloud (more powerful):** add a provider API key in **Settings → Providers**
  (e.g. Novita, Groq, OpenAI). Keys are stored encrypted at rest.

### Step 2 — Create your agents
Open **Agents → New**, or let Freeco do it. For each agent set:
- **Role & name** (e.g. "Sales — CRM").
- **Model** — a strong cloud model for reasoning, a light local model for cheap/private work.
- **Personality / system prompt** — Freeco can draft this for you.
- **Budget** — an optional per‑agent spend cap so costs never surprise you.

### Step 3 — Give agents tools (their "hands")
Agents can only act if they have tools. See **Section 4** below for the full catalog and how
to connect external ones via **MCP**.

### Step 4 — Wire a workflow
Open **Workflows → Builder** and connect agents into a process, e.g.
`Strategist → Marketer → Sales → Bookkeeper`. Each step passes its output to the next.
Ask Freeco: *"Design a workflow that turns a new lead into a sent proposal and a logged
invoice."*

### Step 5 — Connect channels
In **Channels**, connect email, a website/webhook, or a chat surface so your agents can
receive and send in the real world. Start with email — it's the backbone of most org ops.

### Step 6 — Set guardrails, then let it run
- **Approvals** — require your sign‑off for risky actions (sending money, publishing, mass
  email). You'll see plain‑language approval cards.
- **Emergency freeze** — one button halts every agent instantly.
- **Budgets** — global and per‑agent caps.

---

## 4. Tools & MCP — giving agents real capabilities

### Built‑in tools (available now)
FreEco.ai ships tools your agents can use out of the box, including:
- **Web** — fetch pages, search, browse (headless Chrome).
- **Files & workspace** — read/write files in a sandboxed workspace.
- **Code execution** — run code in a sandbox (WASM / subprocess / Docker).
- **Media** — speech‑to‑text and text‑to‑speech (powers Freeco's voice).
- **Messaging** — send through connected channels.

Turn tools on per agent on the **Agents** (tools tab) or **Hands** page.

### Connecting external tools with MCP
[MCP (Model Context Protocol)](https://modelcontextprotocol.io) is the open standard for
plugging external tools and data sources into an AI. FreEco.ai speaks MCP, so you can connect
community and vendor MCP servers (CRMs, calendars, databases, GitHub, cloud drives, etc.).

**How to connect an MCP server:**
1. Find or run an MCP server (see the open‑source list below).
2. In **Settings → MCP / Connectors**, add the server:
   - **Command** servers (run locally): give the launch command, e.g. `npx -y @some/mcp-server`.
   - **URL** servers (remote/SSE): paste the server URL.
3. Save — its tools appear in the tool list and can be granted to agents.
4. Ask Freeco to test it: *"Call the new MCP tool and show me the result."*

> Details and A2A (agent‑to‑agent) federation: see [`docs/mcp-a2a.md`](mcp-a2a.md).

### Open‑source tools on our GitHub
We curate ready‑to‑use tools and MCP connectors under the FreEco org so you don't start from
scratch:

- **Main repo:** <https://github.com/FreecoDAO/freeco-ai> — the platform, built‑in tools, and
  agent templates (`docs/agent-templates.md`).
- **Community MCP servers:** browse the public directory at
  <https://github.com/modelcontextprotocol/servers> for CRM, email, calendar, database,
  GitHub, Slack, Google Drive, and many more — each connects with the steps above.

> Tip: ask Freeco *"what tools do I need for a sales team, and which MCP servers give me a
> CRM?"* — it will recommend a set and walk you through connecting them.

---

## 5. Recipes for common org functions

| You want… | Give the agent these tools | Ask Freeco |
|-----------|---------------------------|------------|
| **A website + domain** | Web, files, code exec, a hosting/DNS MCP | *"Help me plan a site and pick a domain, then draft the pages."* |
| **Email operations** | Email channel, an inbox/IMAP MCP | *"Set up an email agent that triages my inbox and drafts replies."* |
| **Sales & CRM** | Web, a CRM MCP (e.g. HubSpot/Salesforce/an open CRM) | *"Create a sales pipeline and log leads to my CRM."* |
| **Accounting / ERP** | Files, a spreadsheet or accounting MCP | *"Track income and expenses and give me a monthly report."* |
| **Development** | Code exec, GitHub MCP, files | *"Build and ship a small tool, opening a PR for review."* |
| **Fundraising (nonprofit)** | Web, email, a payments/donation MCP | *"Draft a donor outreach campaign and a thank‑you flow."* |

---

## 6. Tips for a smooth self‑running org

- **Start tiny, then grow.** One agent that does one job well beats ten half‑configured ones.
- **Keep a human in the loop for irreversible actions.** Money, publishing, and mass
  messaging should route through Approvals until you trust the flow.
- **Give every agent a budget.** It caps cost and makes runaway loops harmless.
- **Prefer local models for private/cheap work**, cloud models for hard reasoning.
- **Name agents like real roles** ("Finance — Bookkeeper"), so the org chart reads clearly and
  Freeco can delegate to the right one.
- **Review the logs.** The **Logs** and **Analytics** pages show what each agent did and spent.
- **Talk to Freeco.** Anything in this guide, you can just ask the concierge to do or explain.

---

*Have a question this guide doesn't answer? Open Freeco and ask — that's what it's for.*
