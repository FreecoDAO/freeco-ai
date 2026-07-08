# Self-Development Pod (Phase 6, stage 1)

FreEco.ai can work on its own codebase: a **developer agent** implements
scoped GitHub issues in a sandboxed repository clone, and a **tester /
ethical-hacker agent** tries to break the result before any human merges.
This is stage 1 of the [Phase 6 — Self-Running AI Company](../README.md#phase-6--self-running-ai-company-execution-team)
roadmap.

## The loop

```
GitHub issue (labeled `agent-ok`)
   → freeco-developer: branch → implement → verify → draft PR
   → freeco-tester: build/test/clippy/cargo-audit + security review → PR verdict
   → HUMAN reviews the diff and the verdict → merge (or send back)
   → tag vX.Y.Z → release CI ships installers
```

Humans stay at the gate: agents never push to `main`, never merge, never tag.

## Setup

1. Spawn the pod from the dashboard: Overview → FreEco.ai Editions → **Dev Pod**
   (or spawn the `freeco-developer` and `freeco-tester` templates individually).
2. The agents need two tools on the host (or in the sandbox image):
   `git` and the GitHub CLI `gh`, authenticated with a **fine-scoped PAT**
   (`repo` on this repository only — not an admin token).
3. Pick an issue, label it `agent-ok`, and tell the developer agent:
   *"Implement issue #123"*.

## Permission design

Both agents run with deliberately narrow manifests
([developer](../agents/freeco-developer/agent.toml),
[tester](../agents/freeco-tester/agent.toml)):

- shell limited to `git`, `cargo`, `gh` (+ `rustup` for the developer);
- network limited to github.com and crates.io;
- the tester has **no file_write** — it verifies and reports, it cannot "fix";
- both are subject to the budget engine's per-agent spend caps.

## What to hand the pod (and what not to)

Good: typo/docs fixes, small bugfixes with a reproduction, adding tests,
dependency bumps with changelogs, well-specified small features.
Not yet: architecture changes, security-sensitive code paths, anything
without a crisp definition of done. Scope discipline is what makes agent
teams productive — split big issues before assigning them.
