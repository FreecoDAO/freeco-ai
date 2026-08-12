# FreEco.ai Roadmap & verified status

_Last verified: **2026-08-12** against this repository's `v0.9.5` tag, the
running daemon on this machine, and the live OpenRouter model list._

Every claim below was checked against something that can disagree with it — a
running endpoint, a database row, a test, or a published API — rather than
against the commit that was supposed to implement it. Several items previously
described as done turned out to be schema-only or unreachable, and are now
recorded as such.

## Status legend

- ✅ **Shipped** — present in a tagged release.
- 🟨 **Unreleased** — merged on the current branch, not yet tagged.
- ⚠️ **Partial** — some layers exist, but the feature cannot be used.
- 📋 **Planned** — intended work that is not implemented.

## Shipped in v0.7.7

- ✅ Local-first Agent OS with desktop and CLI clients.
- ✅ Multi-agent workflows, approvals, budgets, audit records, and channel
  adapters.
- ✅ Native FreEco CEO, secretary, and shopping agents plus Dev Pod templates.
- ✅ Local-AI setup and model-provider configuration.
- ✅ Cloud-data disclosure confirmation and plain-language approval
  consequences.
- ✅ Shell-execution policy gate verification.
- ✅ Operational API authentication hardening, including the audit-log SSE
  stream.
- ✅ Token-authenticated, loopback-only WhatsApp Web gateway with a trusted
  local browser origin.
- ✅ Reproducible TruffleHog CI installation with a pinned checksum.
- ✅ Model tuning controls for the latest FreEco.ai workflows.
- ✅ Privacy-aware routing between local and cloud model paths.
- ✅ Encrypted recovery flow fixes for protected reasoning/persistence data.
- ✅ Security scanning coverage for bundled skills and release paths.
- ✅ Operator and contributor documentation in `wiki/`.

## Shipped in v0.9.2

- ✅ Chat history is no longer deleted during compaction. Two separate call
  sites were discarding messages; both are removed (`542f980`, and the earlier
  fix it followed).
- ✅ Historical tool results are truncated rather than re-sent in full on every
  turn (`e2ff16b`). This was the source of a ~31:1 token amplification.
- ✅ wasmtime 47.0.3, clearing RUSTSEC-2026-0222.
- ✅ Approval requests state what will run, why, and at what cost.
- ✅ Project, company, and team records with an `/api/org/*` API and a
  dashboard tab — verified by creating a project against the running daemon
  and reading it back.

## Shipped in v0.9.5

- ✅ Agent ids are derived from agent names instead of being random, so a
  reinstall reattaches an agent to its own history. Previously the registry
  came back with new ids and every conversation was orphaned in place: intact
  in the database, invisible in the product, indistinguishable from deletion.
- ✅ Recovery for history orphaned before that fix — `GET
  /api/sessions/orphaned`, `POST /api/sessions/orphaned/adopt`, and a button on
  the sessions page. Changes ownership only; no message is rewritten.
- ✅ OpenRouter model ids are sent in the form OpenRouter accepts. The
  catalog's `openrouter/` prefix is stripped only when a vendor/model pair
  remains, so `openrouter/free` and `openrouter/auto` — real models under the
  `openrouter` vendor — are passed through unchanged.
- ✅ Signup page, one-line instruction, and cost class for all 42 providers,
  served from the backend beside the provider registry. A test fails if a
  provider is added without them. Previously 13 of 42 were explained, in a
  hand-written table in the dashboard JavaScript.
- ✅ The agent model field is a list built from the live catalogue rather than
  a blank box that only worked if you already knew the exact id.
- ✅ Dashboard agent configuration presents and updates all persisted model
  settings: provider, model, output-token limit, temperature, API-key
  environment variable, and provider base URL.

## Unreleased

- 🟨 Label-driven release preparation that validates private-distribution
  prerequisites, creates release metadata, tags the commit, and triggers the
  artifact build without manual version or tag changes.
- 🟨 FreEco.ai sandbox and distribution naming with safe legacy configuration
  compatibility and per-workspace reusable-container isolation.

## Partial — do not describe these as done

- ⚠️ **Unified inbox and contacts (CRM).** The tables exist
  (`inbox_messages`, `contacts`, `contact_handles`, migration v12) and are
  empty. There is no API — `/api/inbox` and `/api/contacts` both return 404 —
  and no dashboard tab. Nothing about this is reachable by a user. Building the
  schema first was reasonable; reporting it as a delivered feature was not.

## Known limits that shape the product

- **OpenRouter's free tier is capped at roughly 50 requests per day** without
  credits, returning `429 free-models-per-day`. An agent run spends several
  requests, so a new user on a free key can hit this within a session and will
  read it as the product being broken — the 429 is currently swallowed and
  shown as a generic failure. Adding $10 of credit raises the cap to 1000/day
  and is not consumed by free models. Any "works for free out of the box"
  claim has to account for this.
- **The desktop app and the CLI daemon share one database** at `~/.openfang`.
  Running both at once puts two kernels on the same SQLite file and the same
  agents, which produces `Agent is unresponsive` heartbeat warnings.

## Planned

- 📋 **Social login, and the keys that come with it.** Sign in with Google,
  GitHub, Meta, or OpenAI, and offer to connect the AI subscriptions and keys
  that account already carries. Most people arriving at FreEco.ai already pay
  for at least one model provider without knowing they hold an API key; asking
  them to find and paste one is the step where they give up. Keys obtained
  this way follow the existing rule — stored in the hands/connector/env layer,
  never in plaintext, never in config committed to a repo.
- 📋 **Offer two or three free providers at setup, not one.** OpenRouter's free
  tier alone runs out within a session (see Known limits). Groq, Google AI
  Studio, and GitHub Copilot have independent free allowances, so proposing
  several and configuring them together gives a new user a working day rather
  than a working ten minutes.
- 📋 **Automatic switch to a backup provider when one is exhausted.** The
  fallback chain already exists and is honoured by the agent loop; what is
  missing is populating it by default from whichever providers the user
  connected, and treating a daily-quota 429 as a reason to move down the chain
  rather than to fail. Mixing a top-tier paid model with free ones works the
  same way: the paid model leads, the free ones catch the overflow.
- 📋 **AI gateway.** Not built — the crate is not in the tree and nothing in
  the runtime or kernel references it. Until it exists, no claim can be made
  about it preventing data leaving the machine.
- 📋 Surface provider rate-limit errors (429) as themselves instead of a
  generic failure, with the remedy stated.
- 📋 Unified inbox and contacts: API and UI over the existing schema.
- 📋 Per-model tuning fields exposed for every provider, not just a few.
- 📋 Import agents and settings from other platforms.
- 📋 Emergency-freeze control on every dashboard screen.
- 📋 Agent deletion-confirmation UI.
- 📋 Automated backup and recovery workflows.
- 📋 Company chart and live multi-agent view.
- 📋 Global assistant widget and voice experience.
- 📋 Multi-user and multi-company tenancy.
- 📋 Expanded language support, CRM integrations, Deskmate, and the FreEco.ai
  OS distribution.

## Release policy

Only work included in a Git tag is described as shipped. Work on the main
branch is unreleased until a new tagged release is published.

A feature is "shipped" when a user can reach it, not when the code that would
support it exists. Schema without an API, or an API without a route
registration, is recorded as Partial.
