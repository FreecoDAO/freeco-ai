# FreEco.ai — Threat Model & Architectural Weakness Review

_Method: STRIDE + MITRE ATT&CK-flavoured red-team analysis, viewed through
the lens of our real target user — **a non-technical person, family, or small
business with no coding or AI experience.** Verified against the codebase on
`main`, 2026-07-11. Severity: 🔴 critical / 🟠 high / 🟡 medium / ⚪ low._

> Guiding principle for every finding: **the user must never be able to harm
> themselves, and must always understand the consequence before an
> irreversible or outward-facing action.** Safety is a default, not a setting.

---

## 1. What we are protecting (assets)

1. The user's **data & files** (documents, memory, agent workspaces).
2. The user's **secrets** (API keys, wallet keys, `secrets.env`).
3. The user's **money** (agents that can buy, pay, or transact).
4. The user's **privacy** (nothing leaves the device without informed consent).
5. **Children's safety** (Kids edition).
6. The **integrity of the agent fleet** (no agent exceeding its mandate).
7. **Trust in the brand** (a single scary incident kills a family product).

## 2. Real trust boundaries (as built)

```
[User] ⇄ [Dashboard UI] ⇄ [API :4200] ⇄ [Kernel] ⇄ [Agents]
                                              │
        ┌─────────────────────────────────────┼───────────────┐
        ▼                    ▼                 ▼               ▼
   [LLM provider]      [Shell/tools]     [Sandboxes:        [Channels:
   (cloud = data       (capability-       WASM, Docker,      Telegram,
    leaves device)      gated)            subprocess,        email, etc.]
                                          workspace]
```

Real security primitives that **exist** in code (good — these are genuine):
- **Capability system** (`Capability`, per-agent `shell`/`network`/`memory`
  allowlists in manifests).
- **Approval queue** (`openfang-kernel/src/approval.rs`) — human-in-the-loop.
- **Sandboxes**: WASM (`sandbox.rs`), Docker (`docker_sandbox.rs`),
  subprocess env-stripping (`subprocess_sandbox.rs`), workspace path-jail
  (`workspace_sandbox.rs`).
- **Audit log** (`audit.rs`), **auth** (`auth.rs`, `session_auth.rs`),
  SSRF checks before browser navigation, external-content tainting.

---

## 3. Threats by category (STRIDE) — with real status

### 🔴 T1 — Spoofing / over-trust of agent output → **social-engineering the user**
A cloud LLM (or a poisoned web page an agent read) instructs the agent to ask
the user to "approve this shell command / paste this key / send this payment."
Our non-technical user cannot judge it and clicks Approve.
**Status:** approval queue exists, but approvals show *what* without *plain-language
consequence*. **Gap:** no "this will let the agent read all your files / spend
money / contact the internet — are you sure?" translation layer.
**ATT&CK:** T1204 (User Execution), T1656 (Impersonation).

### 🔴 T2 — Data exfiltration to cloud model owners
Any cloud provider (GLM/OpenAI/etc.) receives everything the agent sends.
For a business's sensitive data this is a silent leak the user didn't grasp.
**Status:** works, but **no warning** when a cloud provider is configured.
**Gap:** local-first isn't the enforced default; cloud has no red flag.
(Being fixed in the model-policy work, task #16.)
**ATT&CK:** T1567 (Exfiltration Over Web Service).

### 🟠 T3 — Capability escalation / permission-enforcement gaps
An agent with `shell = []` should be *unable* to run shell — the request must
be denied **before** reaching the user, not offered for approval. If the
runtime ever prompts the user for a capability the manifest forbids, that's a
bypass. **Status: UNVERIFIED — must be tested** (open question raised by a live
report that a `shell=[]` secretary appeared to request a shell command).
**ATT&CK:** T1068 (Privilege Escalation). → task #18.

### 🟠 T4 — Destructive actions without a brake
Deleting an agent, wiping memory, overwriting config — one click, irreversible,
by a user who didn't understand. **Status:** confirmation-code before delete
(#14) and auto-backup (#17) are **planned, not built.** The emergency freeze
backend exists (#11) but has no always-visible button (#12).
**ATT&CK:** T1485 (Data Destruction).

### 🟠 T5 — Secrets leakage
`secrets.env` on disk; keys pasted into chat; keys shown in UI/logs.
**Status:** subprocess sandbox strips env (good). **Gap:** no secret-scanning
of what agents emit; no masking of keys in the dashboard; users are invited to
paste keys into config with little warning.
**ATT&CK:** T1552 (Unsecured Credentials).

### 🟡 T6 — Malicious / poisoned model or agent template
A model pulled from an untrusted registry, or an imported agent manifest with
over-broad capabilities. **Status:** manifests are signed (`manifest_signing.rs`)
— good. **Gap:** no capability-diff shown on import ("this agent wants shell +
full network — allow?"); model source not pinned/verified by default.
**ATT&CK:** T1195 (Supply Chain Compromise).

### 🟡 T7 — Kids edition circumvention
A child (or an adult prompt-injecting through the child) coaxes the Kids agent
past its guardrails. **Status:** guardrails are prompt-level + `shell=[]`,
`network=[]` (structurally strong). **Gap:** prompt-only rules can be talked
around; no independent content filter; parental lock (#18/#9) not built.

### 🟡 T8 — Network-exposed dashboard
If a user sets `0.0.0.0` to reach it from their phone, the dashboard + all agent
control is on the LAN. **Status:** auth exists but defaults are permissive on
loopback. **Gap:** non-technical users won't understand the exposure.
**ATT&CK:** T1133 (External Remote Services).

### ⚪ T9 — The "16 security levels" claim
**Finding:** no `SecurityLevel` construct exists in the codebase; the real model
is capability-based. The README's "16 security levels" is **not substantiated in
code.** For a *trust*-branded product, an unbackable security claim is itself a
risk (credibility, and arguably misleading). **Recommendation: substantiate it
(define and implement the 16 levels) or soften the claim to describe the real
capability + sandbox model.**

---

## 4. The non-technical-user danger map (our core audience)

| If the user… | …the risk is | Guardrail needed |
|---|---|---|
| Clicks "Approve" on anything | agent does something harmful they didn't grasp | **Plain-language consequence + risk color** on every approval (T1) |
| Configures a cloud model | silent data leak (T2) | **Red-flag warning + local-first default** (#16) |
| Deletes an agent/data | irreversible loss (T4) | **Type-to-confirm + auto-backup** (#14, #17) |
| Pastes an API key | key leak (T5) | **Masked input + "never share this" note** |
| Runs an agent with shell/buy powers | money/file damage | **Spend caps + capability summary in plain words** |
| Panics | can't stop it | **Always-visible 🔴 STOP button** (#12) |

---

## 5. Prioritized roadmap recommendations

**Tier 0 — before wide release (safety-critical):**
1. **Plain-language approval layer** (T1): every approval states the consequence
   ("lets this agent spend up to $X / read your files / use the internet") with
   a severity color. _New task._
2. **Local-first default + cloud red-flag** (T2, #16).
3. **Verify & harden capability enforcement** (T3, #18) — prove `shell=[]` means
   no shell, add a test, log any bypass attempt.
4. **Type-to-confirm delete + auto-backup** (T4, #14, #17).
5. **Always-visible emergency STOP** (T4, #12).

**Tier 1 — hardening:**
6. Secret masking + emit-scanning (T5).
7. Capability-diff prompt on agent/model import (T6).
8. Independent content filter for Kids (T7).
9. Clear LAN-exposure warning (T8).
10. Substantiate or soften "16 security levels" (T9).

**Tier 2 — the sandbox endgame (strongest privacy story):**
11. **Kubuntu live-USB with no-network isolation** (#21) — makes T2/T3/T6 moot
    for the paranoid/sensitive user by construction.

---

## 6. Honest one-paragraph verdict

The **foundation is genuinely good** — a real capability system, four sandbox
layers, an approval queue, signed manifests, audit logging. That is more than
most agent products ship. **The weakness is not the plumbing; it is the last
inch to the non-technical user:** approvals don't explain consequences, cloud
leakage isn't flagged, destructive actions lack brakes, and one marketing claim
("16 levels") isn't backed by code. None of these are hard to fix, and every one
of them is squarely in the v0.7.4–v0.7.5 window. Fix the "last inch" and
FreEco.ai can honestly market itself as the *safe* agent OS for families and
small business — which no competitor currently is.
