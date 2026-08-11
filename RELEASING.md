# Releasing FreEco.ai

## The entry point: merge a labelled PR

**Open a PR to `main`, give it the `release` label, merge it.** That is the
whole process. Everything after it is automatic.

Do not bump versions or push tags by hand. If you start doing it manually you
have to finish it manually, and the manual path silently skips several things
the automation does — see "What you lose" below.

### What the merge sets off

`prepare-release.yml` runs on a merged PR carrying the `release` label:

1. **Derives the version increment from the PR title** — `feat*` → minor,
   a title containing `!` or `BREAKING CHANGE` → major, anything else → patch.
   So the PR title is the release type. Name it deliberately.
2. **Runs `scripts/prepare_release_metadata.py`**, which updates all four
   version locations together:
   - `Cargo.toml`
   - `crates/openfang-desktop/tauri.conf.json` — this one sets the **installer
     filename**, not `Cargo.toml` (PR #54). Every release between v0.7.5 and
     v0.7.7 shipped installers named with a stale version because the script
     did not yet touch it.
   - `CHANGELOG.md` — promotes the `Unreleased` section, which must exist and
     be non-empty
   - `ROADMAP.md` — moves `Unreleased` items into `Shipped`, both of which
     must exist
3. **Commits `chore(release): vX.Y.Z`, tags it, and pushes with
   `--follow-tags`** using `RELEASE_TOKEN`.
4. **Closes the current milestone and opens the next.**
5. **Moves Project board items** for the previous milestone to Released —
   though this step has never actually run; see below.

`release.yml` then fires on the tag and builds 15 jobs — a metadata gate, 5
desktop platforms, 7 CLI targets, Docker, and the publish step — producing 35
assets (9 `.sig`-signed) plus `latest.json`, which is what the desktop
auto-updater reads. Measured on the v0.9.4 run.

### Why `RELEASE_TOKEN` and not `GITHUB_TOKEN`

GitHub will not let a tag pushed with the default `GITHUB_TOKEN` trigger
another workflow. That is why `RELEASE_TOKEN` must be a PAT with
`contents: write` and `workflow` scope. Without it the tag appears, nothing
builds, and — before this was made loud — no job failed.

## Guards that will stop you

- **`guard` job**: fails any merged PR that changes the `Cargo.toml` version
  without the `release` label. A version bump is a release; it should not
  arrive by accident.
- **`verify-release` job**: before anything builds, the tag, `Cargo.toml`,
  `CHANGELOG.md` and `tauri.conf.json` must all agree. If one disagrees the
  first job fails and every build job is skipped (PR #56).

Check the gate locally with the same commands CI uses:

```bash
version=X.Y.Z
grep -q "^version = \"$version\"$" Cargo.toml
grep -q "^## \[$version\]" CHANGELOG.md
python3 -c 'import json; print(json.load(open("crates/openfang-desktop/tauri.conf.json"))["version"])'
```

## `auto-tag.yml` is a fallback, not the path

It runs on every push to `main` and tags when the version was bumped but no
tag exists — self-healing for a release that was prepared without the label
(PR #50). **It exits silently** when `RELEASE_TOKEN` is unset, when the tag
already exists, or when `CHANGELOG.md` has no section for the version — by
design, so an ordinary push to `main` is not a failure. That silence is also
why v0.9.0 and v0.9.2 showed a green tick while tagging nothing.

**A green Auto Tag means "the version and changelog agree", not "a release was
built".** Those are different claims and conflating them is what hid the
problem for two releases.

## What you lose by doing it manually

v0.9.4 was cut by hand, and these did not happen until they were done manually
afterwards:

- `ROADMAP.md` was not updated — `Unreleased` items were never moved to
  `Shipped`
- no v0.9.4 milestone was created

Note that replaying the script's ROADMAP regex by hand is not safe: it matches
from the *first* `## Shipped in v...` heading through to `## Unreleased`, so
running it against a file that already has several shipped sections relabels
the oldest one with the new version and loses its heading. Add the new section
directly instead.

The Project board step is a separate case: it has never run on any release.
It is guarded by `if: env.PROJECTS_TOKEN != '' && ...` and none of
`PROJECTS_TOKEN`, `PROJECT_NUMBER`, `PROJECT_STATUS_FIELD` or
`PROJECT_RELEASED_OPTION` are configured on the repository. That is not
something a manual release skips — it is unconfigured for everyone.

## Repository secrets, as configured today

Only `RELEASE_TOKEN` and `TAURI_SIGNING_PRIVATE_KEY` exist. Consequences worth
knowing before blaming a build:

- `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` is unset.
- The macOS signing and notarization secrets are unset, so macOS ships an
  unsigned `.dmg` that users open with right-click → Open. The workflow does
  this deliberately rather than failing (see the ad-hoc signing step), so it
  is a choice, not a bug.

## Checking a release actually built

```bash
curl -s "https://api.github.com/repos/FreecoDAO/freeco-ai/actions/workflows/release.yml/runs?per_page=3"
```

A tag with no run at all means the tag never triggered the build — check the
token, not the workflow. `v0.9.3` is the example: a tag, no release, no
failure anywhere.
