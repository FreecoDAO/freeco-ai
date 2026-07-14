#!/usr/bin/env python3
"""Prepare version, changelog, and roadmap metadata for a release."""

from __future__ import annotations

import argparse
import datetime
import re
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
VERSION_PATTERN = re.compile(r'(?m)^version = "(\d+)\.(\d+)\.(\d+)"$')


def next_version(current: tuple[int, int, int], bump: str) -> tuple[int, int, int]:
    major, minor, patch = current
    if bump == "major":
        return major + 1, 0, 0
    if bump == "minor":
        return major, minor + 1, 0
    return major, minor, patch + 1


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--bump", choices=("major", "minor", "patch"), required=True)
    parser.add_argument("--date", default=datetime.date.today().isoformat())
    parser.add_argument("--github-output", type=Path)
    args = parser.parse_args()

    cargo_path = ROOT / "Cargo.toml"
    changelog_path = ROOT / "CHANGELOG.md"
    roadmap_path = ROOT / "ROADMAP.md"
    cargo = cargo_path.read_text(encoding="utf-8")
    changelog = changelog_path.read_text(encoding="utf-8")
    roadmap = roadmap_path.read_text(encoding="utf-8")

    version_match = VERSION_PATTERN.search(cargo)
    if version_match is None:
        raise SystemExit("Could not find the workspace version in Cargo.toml.")
    current = tuple(map(int, version_match.groups()))
    previous_version = ".".join(map(str, current))
    version = ".".join(map(str, next_version(current, args.bump)))

    changelog_match = re.search(
        r"(?ms)^## \[Unreleased\]\n(?P<body>.*?)(?=^## \[)", changelog
    )
    if changelog_match is None or not changelog_match.group("body").strip():
        raise SystemExit("CHANGELOG.md must contain Unreleased release notes.")
    release_notes = changelog_match.group("body").strip()
    changelog = (
        changelog[: changelog_match.start()]
        + f"## [Unreleased]\n\n## [{version}] - {args.date}\n\n{release_notes}\n\n"
        + changelog[changelog_match.end() :]
    )

    roadmap_match = re.search(
        r"(?ms)^## Unreleased\n(?P<body>.*?)(?=^## Planned\n)", roadmap
    )
    if roadmap_match is None or not roadmap_match.group("body").strip():
        raise SystemExit("ROADMAP.md must contain Unreleased roadmap items.")
    shipped_items = re.sub(r"(?m)^- 🟨", "- ✅", roadmap_match.group("body").strip())
    roadmap = roadmap[: roadmap_match.start()] + "## Unreleased\n\n" + roadmap[roadmap_match.end() :]
    shipped_match = re.search(
        r"(?ms)^## Shipped in v[^\n]+\n\n(?P<body>.*?)(?=^## Unreleased\n)", roadmap
    )
    if shipped_match is None:
        raise SystemExit("ROADMAP.md must contain a Shipped section before Unreleased.")
    shipped_body = shipped_match.group("body").strip()
    roadmap = (
        roadmap[: shipped_match.start()]
        + f"## Shipped in v{version}\n\n{shipped_body}\n{shipped_items}\n\n"
        + roadmap[shipped_match.end() :]
    )
    roadmap = re.sub(
        r"(?m)^_Last verified: \*\*[^*]+\*\* against this repository's `v[^`]+` tag,",
        f"_Last verified: **{args.date}** against this repository's `v{version}` tag,",
        roadmap,
        count=1,
    )

    cargo = VERSION_PATTERN.sub(f'version = "{version}"', cargo, count=1)
    cargo_path.write_text(cargo, encoding="utf-8")
    changelog_path.write_text(changelog, encoding="utf-8")
    roadmap_path.write_text(roadmap, encoding="utf-8")
    if args.github_output:
        with args.github_output.open("a", encoding="utf-8") as output:
            output.write(f"previous_version={previous_version}\n")
            output.write(f"version={version}\n")


if __name__ == "__main__":
    main()
