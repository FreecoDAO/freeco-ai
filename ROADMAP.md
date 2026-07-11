# FreEco.ai Roadmap & Honest Status

_Last verified: **2026-07-11**, against git tags, merged/open PRs, the GitHub
release assets, and the source on `main`. This file states what is **actually**
shipped vs. merged-but-unreleased vs. planned — no aspirational checkmarks._

## Legend
- ✅ **Shipped & downloadable** — merged to `main` **and** in a tagged release users can install.
- 🟨 **Merged, not released** — code is in `main` but **no tag/release contains it yet** (nothing to download).
- 🐞 **Shipped-with-bug** — in a release, but broken/partial; fix status noted.
- 🛠️ **In progress** — actively being built, not finished.
- 📋 **Planned** — agreed, not started.

## Releases that actually exist (git tags with installers)
`v0.6.5` … `v0.6.9`, then **`v0.7.1`, `v0.7.2`, `v0.7.3`**.
**Latest downloadable: `v0.7.3`.** Its assets: Windows `-setup.exe` + `.msi`
(x64 & arm64), Linux `.AppImage` + `.deb` + `.rpm`. **No macOS `.dmg` in any
release yet.** **No `v0.7.4` tag exists** — v0.7.4 code is merged to `main`
(PRs #28 + #30, 2026-07-10) but has never been tagged or built into installers.

---

## Status table

| # | Feature | Target | Real status | Evidence |
|---|---------|--------|-------------|----------|
| 1 | Portable USB launcher + landing page | v0.7.2 | ✅ Shipped | PR #15 → tag v0.7.2; live-tested |
| 2 | White day theme + update checker + fre.eco branding | v0.7.2 | ✅ Shipped | PR #16 → tag v0.7.2; served CSS + favicon hash |
| 3 | Native freeco agents (`freeco:*` dispatch) | v0.7.3 | ✅ Shipped | PR #25 → tag v0.7.3; CEO delegation live @ 0 tokens |
| 4 | Dev Pod templates (developer + tester) | v0.7.3 | ✅ Shipped | PR #25; templates present via `/api/templates` |
| 5 | One-click local AI (Ollama + Gemma) | v0.7.3 | 🐞 Shipped; **download breaks on any network hiccup** | Reproduced "error decoding response body" live |
| 6 | Editions cards on dashboard | v0.7.3 | 🐞 **Shipped invisible** — block was rendered inside the Agents page scope, so it never appeared on Overview | Confirmed in source; matches your "I don't see them" |
| 7 | ↳ Download auto-resume (6 retries, keeps layers) | v0.7.4 | 🟨 Merged, **not released** | Code on `main`; not re-tested against a live Ollama end-to-end |
| 8 | ↳ Editions moved into Overview scope | v0.7.4 | 🟨 Merged, **not released** | commit 9475504 on `main`; the binary you're running predates it → still invisible for you until rebuilt |
| 9 | Windows install docs (setup.exe, shortcut, SmartScreen) | v0.7.4 | 🟨 Merged, not released | README on `main` |
| 10 | macOS `.dmg` (ad-hoc, no notarization) | v0.7.4 | 🟨 Merged, **unproven** | Workflow edited on `main`; will only be proven when a v0.7.4 tag runs the release |
| 11 | 🔴 Emergency freeze — backend endpoints | v0.7.4 | 🟨 Merged, not released | Live test: froze 3 / resumed 3 agents on local build |
| 12 | 🔴 Emergency freeze — button on every screen (UI) | v0.7.4 | 📋 Planned | Not built |
| 13 | Agent tuning in UI (edit skills, capabilities, prompt, model) | v0.7.4 | 📋 Planned | **Does not exist anywhere** — matches your report |
| 14 | Confirmation-code before deleting an agent | v0.7.4 | 📋 Planned | Not built |
| 15 | Continuous security-auditor agent | v0.7.4 | 📋 Planned | Not built |
| 16 | Backend-vs-UI coverage audit | v0.7.4 | 🛠️ In progress | `docs/ui-coverage-audit.md` groundwork only |
| 17 | Auto-backup & recovery | v0.7.4 | 📋 Planned | Design notes only |
| 18 | Membership tiers + parental lock | v0.7.4 | 📋 Planned (tier badges shipped in v0.7.3) | Lock/auth logic not built |
| 19 | Company chart + **ChatDev-style live company view** | v0.7.4 | 📋 Planned | **Does not exist.** Engine (workflow + agent messaging) exists; the visual/animated company view does not |
| 20 | Global assistant widget on all tabs + voice | v0.7.4 | 📋 Planned | Not built |
| 21 | Luxury UI redesign (fonts, Manus-style layout) | v0.7.4 | 📋 Planned | Design proposal shared; not implemented |
| 22 | Desktop shortcut / no-black-window launch | — | ⚠️ **Already true for the desktop app; not for the test `.bat`** | See note below |
| 23 | UI languages (EN/DE/de-CH/RM/FR/IT/ES/PL/RU/UK) | v0.8.0 | 📋 Planned | Not started |
| 24 | Multi-user + multi-company tenancy | v0.8.0 | 📋 Planned | Not started |
| 25 | Smart model router + local-first messaging + model catalog UX | v0.7.5 | 📋 Planned | Not started |
| 26 | Open-source CRM connector (Odoo/EspoCRM) | v0.8.x | 📋 Planned | Not started |
| 27 | FreEco Deskmate companion app (iOS-first, logo mascot) | v0.8.x | 📋 Planned | Logo-mascot sketches shared; no app code |
| 28 | FreEco.ai OS — Kubuntu 24.04 live-USB distro + local Gemma | v0.8.x | 📋 Planned | Stated in README; not started |

---

## Honest summary

- **Downloadable today (v0.7.3):** native agents, local-AI setup (with the
  download bug, #5), Dev Pod templates, portable USB, white theme, update
  checker, fre.eco branding. **The Editions cards shipped invisible (#6).**
- **Merged to `main` but NOT in any release (needs a `v0.7.4` tag to reach
  anyone):** editions-visible fix, download auto-resume, emergency-freeze
  backend, macOS `.dmg` build, install docs. **These help nobody until v0.7.4
  is tagged.**
- **Not built at all (the big v0.7.4 UI work you keep asking about):** agent
  tuning UI (#13), freeze button (#12), delete-confirmation (#14),
  security-auditor (#15), **company chart / ChatDev live view (#19)**, backups
  (#17), assistant widget (#20), luxury redesign (#21).

## "Run it like a normal app — no black window, with a desktop shortcut"

Two different things have been in play:
- **The test launcher `run-windows.bat`** (what's been used during development)
  intentionally opens a **console window** and requires the dev machine.
- **The real product — the desktop app** (`FreEco.ai_x.y.z_x64-setup.exe`)
  **already** installs like any app: creates a Desktop + Start-Menu shortcut,
  runs as a normal window with **no console**, per-user (no admin prompt).
  It exists today for **v0.7.3** — but that build still has the invisible-editions
  bug (#6). A **v0.7.4** desktop build would carry all the #7–#11 fixes.

➡️ **The single highest-leverage next action: tag `v0.7.4`.** That triggers CI
to build the v0.7.4 desktop installers (editions visible, download resume,
`.dmg` attempt) that a user can double-click — closing #6, #7, #8, #9, #10, #11
for real.

## Nearest next steps (in order)
1. **Tag `v0.7.4`** → real installers with the merged fixes reach users.
2. Build the v0.7.4 **UI batch** in one branch: freeze button (#12), agent
   tuning (#13), delete-confirmation (#14), security-auditor (#15),
   **company/ChatDev view (#19)**, backups (#17), assistant widget (#20),
   luxury redesign (#21).
3. Then v0.7.5 (model UX, #25) and the v0.8.x platform work (tenancy #24,
   languages #23, CRM #26, Deskmate #27, Kubuntu distro #28).
