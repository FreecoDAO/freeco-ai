# Releasing FreEco.ai

This is the process that has actually produced releases (v0.9.1, v0.9.2,
v0.9.4). It is written down because it was twice re-derived from scratch, and
once replaced with a mechanism that did not work.

## The one rule

**Push the tag from a local machine.** Every successful release in the run
history is `event=push`, `actor=FreecoDAO`.

A tag pushed by a GitHub Actions workflow does **not** start another workflow.
When Auto Tag pushes the tag, the tag appears, `release.yml` never runs, and
nothing reports a failure — the first sign of trouble is a user asking why the
new version never arrived. That is exactly how `v0.9.3` came to exist as a tag
with no release.

## Steps

1. **Bump the version in all four places.** They must agree or the release
   fails at the first job and every build job is skipped:

   | file | what to change |
   |---|---|
   | `Cargo.toml` | `version = "X.Y.Z"` (workspace root, line ~30) |
   | `crates/openfang-desktop/tauri.conf.json` | `"version": "X.Y.Z"` |
   | `CHANGELOG.md` | add a `## [X.Y.Z] - YYYY-MM-DD` section |
   | the tag itself | `vX.Y.Z` |

   `tauri.conf.json` is the one that gets forgotten. It is not covered by the
   workspace version and does not fail any local build, so nothing reminds you.

2. **Check the gate locally before pushing anything.** These are the exact
   commands `release.yml` runs, so if they pass here they pass there:

   ```bash
   version=X.Y.Z
   grep -q "^version = \"$version\"$" Cargo.toml
   grep -q "^## \[$version\]" CHANGELOG.md
   python3 -c 'import json; print(json.load(open("crates/openfang-desktop/tauri.conf.json"))["version"])'
   ```

3. **Commit and push `main` first.** The tag must be an ancestor of
   `origin/main`; it need not be its exact head.

4. **Push the tag yourself:**

   ```bash
   git push origin refs/tags/vX.Y.Z
   ```

   If Auto Tag already created the tag, delete it and re-push it locally, or it
   will never build:

   ```bash
   git push origin :refs/tags/vX.Y.Z
   git tag -f -a vX.Y.Z -m "Release vX.Y.Z"
   git push origin refs/tags/vX.Y.Z
   ```

## What the release produces

14 build jobs — 6 desktop platforms, 7 CLI targets, and a Docker image —
yielding ~35 assets, each `.sig`-signed, plus `latest.json`. That manifest is
what the desktop auto-updater reads from
`https://github.com/FreecoDAO/freeco-ai/releases/latest/download/latest.json`.

## Checking it worked

```bash
curl -s https://api.github.com/repos/FreecoDAO/freeco-ai/actions/workflows/release.yml/runs?per_page=3
```

A release that succeeded shows `conclusion: success`. A tag with no run at all
means the tag was pushed by a workflow — see the one rule above.

## Why Auto Tag still exists

It catches the case where `main` has a version bump that nobody tagged, and it
now fails loudly rather than skipping in silence. Treat a green Auto Tag as
"the version and changelog agree", not as "a release was built" — those are
different claims, and conflating them is what hid the v0.9.0 and v0.9.2
problems at the time.
