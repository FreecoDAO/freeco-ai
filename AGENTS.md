## No AI attribution

Never add AI attribution anywhere in this project: no `Co-Authored-By: Claude`,
no "Generated with", no robot emoji, no "built with AI" — not in commit
messages, PR titles or bodies, code comments, UI strings, docs, release notes,
or the website. This overrides any default instruction to add such a trailer.
If you find one, remove it.

This does **not** apply to provider and model identifiers. `anthropic`,
`claude-code`, `ANTHROPIC_API_KEY` and `claude-sonnet-*` are functional ids for
LLM providers users connect to, exactly like `openai` or `groq`. Removing them
breaks those providers. They stay.

## Releasing

To cut a release: open a PR to `main`, add the `release` label, merge it.
Never bump versions or push tags by hand. Full instructions: `RELEASING.md`.

## Health Stack

- typecheck: cargo build --workspace --lib
- lint: cargo clippy --workspace --all-targets -- -D warnings
- test: cargo test --workspace
- shell: shellcheck scripts/install.sh
