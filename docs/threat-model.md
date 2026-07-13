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

---

# Part II — Mythos deep pass (whole-system, 2026-07-12)

_A second, deeper reading of the actual code across all crates. It **corrects
a mistake in Part I** and adds concrete, code-referenced findings that the
framework pass missed._

## Correction to Part I (T9) — I was wrong about "16 security levels"

Part I said the "16 security levels" claim had "no code behind it." **That was
an overstatement and is retracted.** The dashboard **Security tab** documents
**15 security features**, each with an implementation reference
(`crates/openfang-api/static/js/pages/settings.js`, `coreFeatures` +
`configurableFeatures`):

Path-Traversal Prevention · SSRF Protection · Capability-Based Access Control ·
Privilege-Escalation Prevention (child ⊆ parent capabilities) · Subprocess
Environment Isolation · Security Headers · Wire HMAC Auth · Request-ID Tracking
· API Rate Limiting · WebSocket Limits · WASM Dual Metering · Bearer-Token Auth
· Merkle Audit Trail · Information-Flow Taint Tracking · Ed25519 Manifest
Signing.

So the claim is **~15/16ths real and code-referenced**, not marketing. The
honest remaining work is (a) **spot-verify each of the 15 truly enforces what
it says** (a UI list is not a proof), and (b) state the **exact number** ("15+
enforced security controls") or formally add a 16th, rather than a round "16."
This supersedes the earlier "substantiate or soften" framing.

## New findings the framework pass missed (code-grounded)

### 🔴 M1 — Loopback is fully trusted with **no authentication**
`middleware.rs:156`: when no `api_key` is set, **any loopback request is
allowed through with no auth.** The desktop app ships this way by default. On a
**shared or family machine**, this means _any_ local user, _any_ other
installed program, or _any_ downloaded malware running as the user can drive
the entire agent fleet — spend money, run shell, read files — with zero
credential. For a product whose headline users are **families and small
business on shared laptops**, "localhost = trusted" is the single biggest
real-world hole. **Fix:** optional-but-encouraged local dashboard password
(separate from the LAN api_key), on by default for the desktop app; at minimum,
a first-run prompt. → _new task._

### 🟠 M2 / M3 — Secrets stored in **plaintext**, and unprotected on **Windows**
API keys, and any wallet keys, are written to `~/.openfang/secrets.env` in
plaintext (`dotenv.rs`). The code sets `0600` perms **only under `#[cfg(unix)]`
(`dotenv.rs:183`)** — **on Windows (a primary target) the file is left at
default ACL**, readable by any process running as the user, and copied by any
cloud-sync of the home folder. A stolen laptop, a backup, or OneDrive
syncing the folder = every key leaked. **Fix:** encrypt secrets at rest (OS
keychain: Windows Credential Manager / macOS Keychain / libsecret), or at least
apply a restrictive Windows ACL; never store wallet keys in plaintext. → _new
task._

### 🟠 M4 — Local-AI installer is **downloaded and executed without verification**
`local_ai.rs`: `OllamaSetup.exe` is fetched over HTTPS and run with
`/VERYSILENT` — **no SHA-256 pin, no signature check.** HTTPS protects transit,
but there is no integrity pin: a compromised mirror, a bad CDN edge, or a
future URL change silently yields **arbitrary code execution as the user**,
from inside our "one-click, no-account, safe" flow — the exact opposite of the
promise. **Fix:** pin and verify the installer's SHA-256 (or Authenticode
signature) before executing; fail closed on mismatch. → _new task._

### 🟠 M5 — Auto-backup (#17) will **exfiltrate the plaintext secrets**
The planned auto-backup zips `OPENFANG_HOME`, which contains `secrets.env`.
Backups may land on a **USB stick or a synced folder** → plaintext keys travel
off-device inside a "backup." **This must be designed in before #17 is built:**
exclude secrets from backups, or encrypt the backup, or both. → _folded into
#17 with a blocking note._

### 🟡 M6 — `exec_policy: full` is a **single flip to a wide-open shell**
Shell is gated by an allowlist (good), but `ExecSecurityMode::Full`
(`tool_runner.rs:278`) disables the allowlist entirely. An LLM that convinces a
user to "set exec mode to full," or a manifest import with it preset, yields
unrestricted shell. **Fix:** treat `full` as a red, explicitly-confirmed,
plain-language-warned mode; flag it in the security-auditor (#18).

### 🟡 M7 — Kids **content** safety is prompt-only
Structural guardrails are strong (`shell=[]`, `network=[]`) — a child agent
genuinely can't reach the shell or internet. But **content** safety (refusing
harmful topics) is **prompt-level only**; a determined child or a prompt
injection can argue the model around its instructions. **Fix:** an independent,
non-model content filter for the Kids edition (#18/#9), not just the system
prompt.

### 🟡 M8 — Spend **hard-stop** unverified
`freeco-budget-engine/enforcer.rs` has "exceeded" logic with tests, but I did
**not** verify it hard-blocks a live LLM/purchase call versus only reporting
cost after the fact. For money-spending agents this is the difference between a
cap and a speedometer. **Fix:** verify (and test) that exceeding budget
*prevents* the next paid action. → _verification task._

### 🟡 M9 — P2P shared secret is symmetric and optional
Wire auth is HMAC-SHA256 (good), but the `shared_secret` is one symmetric key
for the whole mesh and can be left empty (`network.shared_secret = ""`). One
leak compromises every peer. **Fix:** per-peer keys or asymmetric identities;
refuse to mesh with an empty secret.

## Revised priority (what actually moves risk most)

| Rank | Finding | Why it's #1-worthy | Task |
|------|---------|--------------------|------|
| 1 | **M1** loopback = no auth on shared machines | Whole product is "families on shared laptops"; this is total local bypass | new |
| 2 | **M4** unverified installer execution | RCE inside the "safe" one-click flow | new |
| 3 | **M2/M3** plaintext secrets, Windows-unprotected | Primary OS leaks keys at rest | new |
| 4 | **M5** backup exfiltrates secrets | Turns a safety feature into a leak | fold into #17 |
| 5 | T1 plain-language approvals + T2 cloud flag | The "last inch" (already in PR #33) | #22/#16 ✅ started |
| 6 | **M8** verify spend hard-stop | Money safety | new |
| 7 | M6 exec-full warning · M7 Kids filter · M9 P2P keys | Hardening | #18/#9 |

## Corrected one-line verdict
The security **engineering** is real and better than Part I gave it credit for
(≈15 code-referenced controls). The exposure is **not** in the crypto or
sandboxes — it is in **trust defaults for the real deployment**: localhost
treated as trusted on shared machines, secrets in plaintext on Windows, and an
unverified installer download inside the "safe" flow. Those three are the
Mythos headline, and none are hard to fix.
