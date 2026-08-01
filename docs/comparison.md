# FreEco.ai compared

Where FreEco.ai sits against the tools it gets compared to. The claims about
other products describe what they are designed for, not a judgement of how
well they do it — each is good at the job it was built for, and those jobs
differ from this one.

## The short version

Most of the tools below are **assistants**: you type, they answer, the session
ends. FreEco.ai is an **operating system for agents that keep working when you
close the laptop** — on schedules, with budgets, with approval gates, and with
the back office (CRM, accounting, voice) wired in rather than bolted on.

## Feature comparison

| | FreEco.ai | Claude / ChatGPT | Manus | GitHub Copilot | VS Code + extensions | OpenClaw / OSS frameworks |
|---|---|---|---|---|---|---|
| **Runs when you are away** | ✅ scheduled, 24/7 | ❌ session-bound | ✅ | ❌ | ❌ | ⚠️ you build it |
| **Self-hosted, your machine** | ✅ single binary | ❌ vendor cloud | ❌ | ❌ | ✅ editor only | ✅ |
| **Open source** | ✅ | ❌ | ❌ | ❌ | ⚠️ mixed | ✅ |
| **Works offline** | ✅ local models | ❌ | ❌ | ❌ | ⚠️ | ⚠️ |
| **Multi-agent org chart** | ✅ CEO → specialists | ❌ | ⚠️ opaque | ❌ | ❌ | ⚠️ you build it |
| **Spend caps per agent** | ✅ budget engine | ❌ | ❌ | ❌ | ❌ | ❌ |
| **Approval gates** | ✅ allow once/always/deny | ⚠️ per-tool | ⚠️ | ❌ | ❌ | ⚠️ |
| **Isolated Linux sandbox** | ✅ no network, caps dropped | ⚠️ vendor-side | ✅ | ❌ | ❌ | ⚠️ |
| **CRM built in** | ✅ Twenty | ❌ | ❌ | ❌ | ❌ | ❌ |
| **Accounting built in** | ✅ Akaunting | ❌ | ❌ | ❌ | ❌ | ❌ |
| **Real-time voice** | ✅ dograh, MCP-native | ⚠️ chat voice | ❌ | ❌ | ❌ | ❌ |
| **MCP support** | ✅ client + gateway | ✅ client | ❌ | ⚠️ | ✅ client | ⚠️ |
| **Roles: kids / family / business** | ✅ RBAC | ❌ | ❌ | ❌ | ❌ | ❌ |
| **Your data stays yours** | ✅ local DB | ❌ | ❌ | ❌ | ✅ | ✅ |
| **Install** | one binary | web signup | web signup | subscription | editor + setup | build it yourself |

✅ built in · ⚠️ partial or DIY · ❌ not offered

## What this actually means

**Against Claude and ChatGPT.** They are better conversationalists and have
larger models behind them. They stop when the tab closes, and everything you
type goes to a vendor. FreEco.ai runs on your hardware, keeps working on a
schedule, and can drive the same frontier models through your own key — with a
spend cap, which no chat interface offers.

**Against Manus.** The closest in ambition: autonomous, sandboxed, long-running.
Manus is a hosted product. FreEco.ai is a binary you own, can read the source
of, and can run with no internet at all.

**Against Copilot and VS Code.** Different category. They make one developer
faster inside an editor. FreEco.ai runs a team of agents across a business —
sales, books, calls, support — and only some of that is code.

**Against open-source frameworks.** LangChain, CrewAI, AutoGPT and similar are
libraries: you assemble the runtime, persistence, scheduling, permissions and
budgets yourself. FreEco.ai ships those as the product. The trade is
flexibility for not having to build an OS before you build an agent.

## Honest limits

A comparison table that only flatters its author is worth nothing, so:

- **Frontier reasoning still comes from a frontier model.** Local Gemma 4 is
  free and private, and it is not Opus. Hard tasks route to a cloud model.
- **Local models need a real GPU.** Without a discrete GPU, one agent turn can
  take around an hour, so local inference is off by default on such machines
  and the reason is stated plainly in the UI.
- **The business services are integrations, not rewrites.** Twenty and
  Akaunting are excellent existing products, run as containers alongside.
- **Self-hosting is work.** A VPS, DNS and TLS are your responsibility in a way
  a SaaS signup is not.
- **Younger than the incumbents**, with the rough edges that implies.
