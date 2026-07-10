# Dashboard Coverage Audit (v0.7.4)

Comparing all 157 `/api/*` routes against what the dashboard surfaces. The
recurring finding this session: **abilities exist in the backend but are
hidden or missing in the UI.** This audit drives the v0.7.4 UI work.

## Confirmed: backend exists, UI hides it (fix in v0.7.4)

| Ability | Backend | UI gap |
|---|---|---|
| Delete an LLM provider key | `DELETE /api/providers/{name}/key` (exists) | No delete button in Settings → Providers |
| Change a provider's base URL | `PUT /api/providers/{name}/url` | Not exposed |
| Change model per agent | `PUT /api/agents/{id}/model` (exists, used in agent detail + `/model` chat cmd) | Discoverable only via slash command or buried detail panel — needs a visible per-agent model picker |
| Custom models | `/api/models/custom` (GET/POST/DELETE) | Add form exists; delete/edit weak |
| Emergency stop everything | per-agent `/api/agents/{id}/stop`; no global | No global control (added in v0.7.4: `POST /api/agents/stop-all` + floating button) |

## Missing entirely (build in v0.7.4+)

- **Global emergency freeze** — floating red button on every screen → pause
  all agents without deleting them. (v0.7.4)
- **Provider delete/edit UX** — trash + edit on each configured provider card. (v0.7.4)
- **Per-agent model picker** — dropdown on the agent card/detail, populated
  from `/api/models`, writing `PUT /api/agents/{id}/model`. (v0.7.4)
- **Local vs cloud model guidance** — model catalog should mark local
  models, show minimum RAM/requirements, and explain the privacy +
  ecological benefit of local-first; auto/manual routing control. (v0.7.5,
  smart model router)
- **Backup/restore UI** — Settings → System card. (v0.7.4)
- **Company charts** — no org-structure view. (v0.7.4)
- **Global assistant widget** — help on every tab. (v0.7.4)

## Sandboxing / root-access answer (for docs & UI security page)

FreEco.ai does **not** need root. It runs as a normal user process. Agent
code execution is layered: WASM sandbox (deny-by-default, no fs/net/creds
unless granted), Docker sandbox (optional, uses host Docker), subprocess
env-stripping (secrets never reach child processes), and a workspace
filesystem sandbox (path-traversal/symlink-escape prevention). The browser
tool drives an installed Chromium over CDP with SSRF checks. Data lives in
`$OPENFANG_HOME` (default `~/.openfang`), never system directories. This
should be surfaced on a Settings → Security page.

## Aesthetic (v0.7.4 luxury redesign)

- Luxury-magazine serif display face for headings (self-hosted, CSP-safe).
- fre.eco green logo present on the light/day theme (not only dark).
- Manus-inspired: persistent agent-activity panel, task timeline, calmer
  whitespace, single-accent discipline.
