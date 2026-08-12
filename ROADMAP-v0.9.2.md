# v0.9.2 — carried forward

Written so these survive a session boundary instead of being re-derived. Each
entry says what is already built, so the next session starts from the gap
rather than from zero.

---

## 1. Security agent as CSO

**Foundation that already exists**

| Piece | Where | State |
|---|---|---|
| PII detection, reversible masking, policy | `crates/openfang-datagateway` | branch `feature/data-gateway`, **not merged** |
| Persistent token store | `crates/openfang-datagateway/src/store.rs` | written, 5 tests pass |
| Merkle audit trail | `audit_entries`, migration v8 | live |
| Secret-handling + anti-bypass policy | `openfang-types/src/security_policy.rs` | live, 5 tests |
| Per-agent spend data | `usage_events` table | live, 114 rows |
| Sandbox with real toolchain | `deploy/sandbox/Dockerfile` | live |

**Still to build**

- **Merge the datagateway.** Three defects were found by review: the masking
  key was zeroized in `new()` and never persisted, the token store was an
  in-memory `HashMap`, and `.expect()` in the encrypt path would panic the
  daemon while handling exactly the data that must not reach a crash dump.
  `store.rs` fixes the first two; the panic path still needs converting to
  `Result`.
- **Wire it into the LLM call path.** `grep datagateway` across
  `freeco-kernel-runtime` and `openfang-kernel` currently returns nothing, so it
  inspects zero traffic. A security control that is present but not connected
  is worse than absent: the dashboard implies protection that does not exist.
- **Continuous inspection** of agent traffic, using the gateway as the
  chokepoint rather than a second scanner.
- **Vet agents before hire.** Check manifest, tool grants and model before an
  imported agent runs once, not after.
- **Spend monitoring per agent and total.** The data is already in
  `usage_events`; what is missing is a threshold, an alert, and a per-agent
  cap that can actually stop a run. The EUR 25 incident was visible in that
  table for days with nobody watching it.
- **Owner reporting with a kill switch.** Reports to owner/admin only, and any
  automatic shutdown must be confirmed with them first.

**Design note.** The CSO agent must not be able to grant itself exemptions.
Its findings go to the owner; it does not adjudicate its own alerts.

---

## 2. Unified inbox and address book

**Foundation that already exists**

- **44 channel integrations** in `crates/openfang-channels/src/`: email,
  signal, whatsapp, telegram, slack, teams, matrix, xmpp, discord, linkedin,
  irc, mastodon, webex, zulip, rocketchat, nostr, mqtt, viber, threema, line,
  messenger, reddit, twitch and more.
- **Merkle audit trail** for every message.
- **Inter-agent messaging** — `kernel.rs::send_message`.
- **RBAC** — Viewer / Kid / User / Admin / Owner.

**Still to build**

- One inbox across all 44 channels, threaded per correspondent.
- Address book scoped by permission: global, team, private. Import/export.
- Send from the inbox, or ask Freeco to send on your behalf.
- Search across every past conversation, agent and human alike.
- Funnels, dashboards, mindmaps over the same data, filtered by access rights.

**The plumbing is done; this is the surface over it.** Worth doing in that
order — a half-built inbox on top of 44 working channels is a UI problem, but
a UI on top of missing channels would be a rewrite.

---

## 3. Smaller items, still open

- **wasmtime RUSTSEC-2026-0222.** Needs 43 -> 47.0.3. Note 47.0.2 does *not*
  clear the advisory, which is what dependabot proposed. The API surface is
  four calls in one file (`sandbox.rs`), so the migration is likely small; the
  bump is started on branch `fix/wasmtime-rustsec-2026-0222`.
- **OpenFang -> FreEco rename.** 14 crates, the binary name, `~/.openfang/`
  config path, install script and update URL. Needs a migration path or it
  orphans existing installs.
- **Project management: tasks.** Companies, projects and teams exist with a
  UI. Tasks with status, assignee, dependencies and a board do not.
- **Delete/archive buttons in the main Sessions view.** The backend states are
  non-destructive and the Projects page has them; the Sessions view does not.

---

## Verified facts worth not re-deriving

- **USB**: 52.4 GB unallocated on disk 1. Kubuntu 24.04.3 ISO is on `E:`,
  byte-identical to source, sha256 matches Ubuntu's published sum. Boot setup
  is `scripts/make-usb-bootable.ps1`, needs one elevated run. Nothing is
  erased; GRUB loopback boots the ISO from exFAT via the existing 1 GB ESP.
- **`Downloads/soft` contains Windows installers, not Linux distros.** Docker,
  Rust, Brave, dograh and Akaunting install from Kubuntu's own repos.
- **Twenty CRM is not the place for internal comms monitoring.** It models
  external relationships; internal agent/human traffic is an audit concern
  with different retention and access rules.
- **Business services do not belong in the agent sandbox.** They need network,
  uptime and persistence; the sandbox is `--network none`, read-only and
  throwaway. Separate containers, as in `deploy/demo/`.
