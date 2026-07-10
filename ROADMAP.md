# FreEco.ai Roadmap & Honest Status

_Last verified: 2026-07-10, against git tags, merged/open PRs, and a live
daemon at `127.0.0.1:4200`. This file states what is **actually** shipped
vs. built-but-unreleased vs. planned — no aspirational checkmarks._

## Legend
- ✅ **Shipped** — merged to `main` **and** in a tagged release users can download.
- 🟦 **Built, unreleased** — code exists on a branch/local build, verified working, but **not** in any release yet.
- 🐞 **Shipped-with-bug** — released, but broken or partially non-functional; fix status noted.
- 🛠️ **In progress** — actively being built.
- 📋 **Planned** — agreed, not started.

## Releases that actually exist (git tags with installers)
`v0.6.5` … `v0.6.9`, then **`v0.7.1`, `v0.7.2`, `v0.7.3`**. Latest downloadable: **v0.7.3**.
**`v0.7.4` is not released** — it is open PR #28 (branch `feat/v0.7.4-backup-ui-audit`), not merged, not tagged.

---

## Status table

| # | Feature | Intended | Real status | Where | Verified how |
|---|---------|----------|-------------|-------|--------------|
| 1 | Portable USB launcher + landing page | v0.7.2 | ✅ Shipped | PR #15, tag v0.7.2 | Live-tested daemon boot from bundle |
| 2 | White day theme + update checker + fre.eco branding | v0.7.2 | ✅ Shipped | PR #16, tag v0.7.2 | Served CSS + favicon hash |
| 3 | Release workflow secrets-in-`if` fix | v0.7.2 | ✅ Shipped | PR #17 | Release CI ran with jobs |
| 4 | Native freeco agents (`freeco:*` dispatch) | v0.7.3 | ✅ Shipped | PR #25, tag v0.7.3 | CEO delegation live @ 0 tokens |
| 5 | One-click local AI (Ollama + Gemma) | v0.7.3 | 🐞 Shipped, download breaks on network hiccup | PR #25, tag v0.7.3 | Reproduced "error decoding response body" live |
| 5a | ↳ Download auto-resume fix | v0.7.4 | 🟦 Built, unreleased | branch, local build | Code compiled; not yet re-tested end-to-end |
| 6 | Dev Pod (developer + tester agents) | v0.7.3 | ✅ Shipped (templates) | PR #25, tag v0.7.3 | Templates present via API |
| 7 | Editions cards on dashboard | v0.7.3 | 🐞 Shipped but **rendered out-of-scope → invisible** | PR #25, tag v0.7.3 | Found block misplaced in chat page, outside `overviewPage` data scope |
| 7a | ↳ Editions moved into Overview scope | v0.7.4 | 🟦 Built, unreleased | branch (this PR) | Source verified in `overviewPage` scope; **needs rebuild to render** |
| 8 | Windows install docs (setup.exe, shortcut, SmartScreen) | v0.7.4 | 🟦 Built, unreleased | PR #28 | README updated |
| 9 | macOS `.dmg` (ad-hoc, no notarization) | v0.7.4 | 🟦 Built, unreleased | PR #28 (also Copilot PR #26) | Workflow edited; unproven until a release runs |
| 10 | 🔴 Emergency freeze — backend | v0.7.4 | 🟦 Built, unreleased | branch, local build | Live: froze 3 / resumed 3 agents |
| 11 | 🔴 Emergency freeze — button on every screen | v0.7.4 | 📋 Planned | — | UI not built |
| 12 | Agent tuning in UI (skills, capabilities, prompt) | v0.7.4 | 📋 Planned | — | Does not exist anywhere yet |
| 13 | Confirmation-code before deleting agents | v0.7.4 | 📋 Planned | — | Not built |
| 14 | Continuous security-auditor agent | v0.7.4 | 📋 Planned | — | Not built |
| 15 | Backend-vs-UI coverage audit | v0.7.4 | 🛠️ In progress | `docs/ui-coverage-audit.md` (stub) | Groundwork only |
| 16 | Auto-backup & recovery | v0.7.4 | 📋 Planned | design in notes | Not built |
| 17 | Membership tiers + parental lock | v0.7.4 | 📋 Planned (badges only shipped) | badges in v0.7.3 | Lock logic not built |
| 18 | Company chart + **ChatDev-style live view** | v0.7.4 | 📋 Planned | — | **Does not exist** — engine (workflows/messaging) exists, cinematic view does not |
| 19 | Global assistant widget + voice | v0.7.4 | 📋 Planned | — | Not built |
| 20 | Luxury UI redesign (fonts, Manus-style) | v0.7.4 | 📋 Planned | design proposal shared | Mockup only |
| 21 | Desktop shortcut / no-black-window launch | v0.7.4 | 🐞 Partial | Tauri installer creates shortcut; portable `.bat` shows console | Desktop app = no console; portable script = console |
| 22 | UI languages (10 langs) | v0.8.0 | 📋 Planned | — | Not started |
| 23 | Multi-user + multi-company tenancy | v0.8.0 | 📋 Planned | — | Not started |
| 24 | Smart model router + local-first messaging | v0.7.5 | 📋 Planned | — | Not started |
| 25 | Open-source CRM connector (Odoo/EspoCRM) | v0.8.x | 📋 Planned | — | Not started |
| 26 | FreEco Deskmate companion app (iOS-first) | v0.8.x | 📋 Planned | logo-mascot sketches shared | Design only |
| 27 | FreEco.ai OS — Kubuntu live-USB distro | v0.8.x | 📋 Planned | stated in README | Not started |

---

## Honest summary
- **Downloadable today (v0.7.3):** native agents, local-AI setup (with the
  download bug), Dev Pod templates, portable USB, white theme, update checker.
  The **Editions cards shipped broken** (invisible) in v0.7.3.
- **Fixed locally, awaiting the v0.7.4 release:** editions rendering,
  download auto-resume, emergency-freeze backend, macOS `.dmg`, install docs.
- **Not built yet (the big v0.7.4 UI work):** agent tuning UI, freeze button,
  delete-confirmation, security-auditor, company chart / ChatDev view,
  backups, assistant widget, luxury redesign.

## Nearest next steps
1. Merge PR #28 → tag **v0.7.4** so the fixes above reach users.
2. Build the v0.7.4 UI batch (freeze button, agent tuning, company view) in one branch.
3. Then v0.7.5 (model UX) and the v0.8.x platform work (tenancy, distro, Deskmate).
