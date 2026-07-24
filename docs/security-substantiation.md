# Security substantiation — FreEco.ai

**Purpose.** The Free Eco Association & DAO (a Swiss nonprofit) will hold
personal data of donors, grant givers and partners. That triggers the Swiss
**FADP** (revFADP, in force since Sept 2023) and, for EU data subjects, the
**GDPR**. This document substantiates which security controls are *actually
implemented* (with file references you can verify), which are *partial*, and
which are *not started* — so we never publish a security claim we can't back,
and so we know exactly what must close **before real donor data is onboarded**.

Status legend: ✅ implemented · 🟡 partial · ⬜ not started.

## A. Technical controls in the codebase (verifiable)

| # | Control | Status | Where |
|---|---------|--------|-------|
| 1 | Admin identity + password auth (argon2id) | ✅ | `session_auth.rs` (`hash_password`/`verify_password`), `routes.rs::auth_login` |
| 2 | RBAC roles (owner/admin/user/kid/viewer) | ✅ (channels) 🟡 (dashboard) | `openfang-kernel/src/auth.rs`; enforced in `channel_bridge.rs`; dashboard enforcement is task #31 |
| 3 | Secrets encrypted at rest | ✅ | secrets encryption (aes-gcm) + argon2 KDF |
| 4 | Windows ACL hardening on secret files | ✅ | `routes.rs` icacls helper (DOMAIN\\USER, absolute System32\\icacls.exe) |
| 5 | Installer signature verification (M4) | ✅ | `local_ai.rs::verify_windows_signature` (Authenticode) |
| 6 | Path-traversal prevention | ✅ | `host_functions.rs::safe_resolve_path` |
| 7 | SSRF protection (private IPs, cloud metadata) | ✅ | `host_functions.rs::is_ssrf_target` |
| 8 | Capability-based access control (deny by default) | ✅ | `host_functions.rs::check_capability` |
| 9 | Privilege-escalation prevention on agent spawn | ✅ | `kernel_handle.rs::spawn_agent_checked` |
| 10 | Subprocess env isolation (no secret leakage) | ✅ | `subprocess_sandbox.rs` (`env_clear` + allowlist) |
| 11 | Security headers (CSP, X-Frame-Options, nosniff) | ✅ | `middleware.rs::security_headers` |
| 12 | Wire HMAC-SHA256 auth (agent-to-agent) | ✅ | `peer.rs::hmac_sign`/`hmac_verify` |
| 13 | Request-ID tracking + audit log | ✅ | `middleware.rs`; `audit_log` (hash-chained) |
| 14 | API rate limiting (GCRA) | ✅ | `rate_limiter.rs` |
| 15 | Local-first / no-cloud option (data never leaves device) | ✅ | local Ollama models + cloud-leak warning |
| 16 | Dashboard request-level RBAC enforcement + Kid lockdown | 🟡 | identity resolved; enforcement = task #31 |

**The "16 controls" claim is substantiated for 1–15; #16 is in progress.** Do not
market #16 as complete until task #31 lands.

## B. GDPR / FADP process controls (mostly NOT code — NOT started)

These are the gate for donor data and are **process/legal**, not features:

| Control | Status | Note |
|---------|--------|------|
| Record of processing activities (RoPA) | ⬜ | list every place PII lives: CRM (Twenty), accounting (Akaunting), memory (Graphify), channel logs, DB |
| Lawful basis + privacy notice for donors | ⬜ | consent / legitimate interest; a public privacy policy |
| Data subject rights (access, deletion, export) | ⬜ | a documented process; deletion must reach CRM + accounting + memory + backups |
| Data retention + minimisation policy | ⬜ | how long donor records are kept; auto-purge |
| Data Processing Agreements with sub-processors | ⬜ | every cloud LLM/STT/TTS provider that sees PII needs a DPA; prefer local models for PII |
| Encryption in transit everywhere | 🟡 | dashboard is localhost; any remote connector (CRM/accounting/dograh) must be TLS |
| Breach notification process | ⬜ | FADP: notify the FDPIC "as soon as possible" |
| Backup encryption + restore test | 🟡 | backups exist (`start_backup_scheduler`); verify they're encrypted and test a restore |
| Access logging / least privilege for staff | 🟡 | RBAC + audit log exist; needs an access-review routine |
| Independent security review / pen test | ⬜ | before production donor data |

## C. Gate before onboarding real donor/grant data
Minimum bar (do NOT skip for a live Association):
1. **Task #31** — dashboard RBAC enforcement + Kid lockdown (technical #16).
2. **RoPA** — write down every store that holds PII (Section B row 1).
3. **Local models for PII** — route any prompt containing donor data to a local
   model, or sign DPAs with the cloud provider. The cloud-leak warning already
   exists; make it a hard policy for PII.
4. **TLS on every connector** that leaves the machine (CRM/accounting/dograh).
5. **Privacy notice + deletion process** covering CRM + accounting + memory + backups.
6. **Encrypted backups + one tested restore.**

Until 1–6 are done, run with **synthetic/test data only**.

---
*This file is the single source of truth for the security claim. Update the
status column as controls land; never let the marketing copy outrun this table.*
