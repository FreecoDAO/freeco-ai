import importlib.util
import sys
import tempfile
import unittest
from pathlib import Path


SCRIPT_PATH = Path(__file__).with_name("prepare_release_metadata.py")
SPEC = importlib.util.spec_from_file_location("prepare_release_metadata", SCRIPT_PATH)
assert SPEC and SPEC.loader
prepare_release_metadata = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(prepare_release_metadata)


class PrepareReleaseMetadataTests(unittest.TestCase):
    def setUp(self):
        self.temp_dir = tempfile.TemporaryDirectory()
        self.root = Path(self.temp_dir.name)
        (self.root / "scripts").mkdir()
        (self.root / "crates" / "openfang-desktop").mkdir(parents=True)
        (self.root / "Cargo.toml").write_text(
            '[workspace.package]\nversion = "0.9.5"\n', encoding="utf-8"
        )
        (self.root / "CHANGELOG.md").write_text(
            "# Changelog\n\n## [Unreleased]\n\n### Changed\n\n- Release work.\n\n"
            "## [0.9.5] - 2026-08-12\n\n- Existing release.\n",
            encoding="utf-8",
        )
        (self.root / "ROADMAP.md").write_text(
            "# Roadmap\n\n"
            "## Shipped in v0.9.2\n\n- ✅ Existing item.\n\n"
            "## Unreleased\n\n- 🟨 New release item.\n\n"
            "## Partial — do not describe these as done\n\n- ⚠️ Existing partial item.\n\n"
            "## Planned\n\n- 📋 Existing plan.\n",
            encoding="utf-8",
        )
        (self.root / "crates" / "openfang-desktop" / "tauri.conf.json").write_text(
            '{"version": "0.9.5"}\n', encoding="utf-8"
        )

    def tearDown(self):
        self.temp_dir.cleanup()

    def run_script(self):
        original_root = prepare_release_metadata.ROOT
        original_argv = sys.argv
        try:
            prepare_release_metadata.ROOT = self.root
            sys.argv = [
                "prepare_release_metadata.py",
                "--bump",
                "patch",
                "--date",
                "2026-08-13",
            ]
            prepare_release_metadata.main()
        finally:
            prepare_release_metadata.ROOT = original_root
            sys.argv = original_argv

    def test_preserves_every_roadmap_section(self):
        self.run_script()

        roadmap = (self.root / "ROADMAP.md").read_text(encoding="utf-8")
        self.assertIn("## Shipped in v0.9.2", roadmap)
        self.assertIn("## Shipped in v0.9.6", roadmap)
        self.assertIn("## Unreleased", roadmap)
        self.assertIn("## Partial — do not describe these as done", roadmap)
        self.assertIn("## Planned", roadmap)
        self.assertIn("- ✅ New release item.", roadmap)
        self.assertLess(
            roadmap.index("## Shipped in v0.9.6"),
            roadmap.index("## Unreleased"),
        )

    def test_rejects_empty_changelog_unreleased_section(self):
        (self.root / "CHANGELOG.md").write_text(
            "# Changelog\n\n## [Unreleased]\n\n## [0.9.5] - 2026-08-12\n",
            encoding="utf-8",
        )

        with self.assertRaisesRegex(
            SystemExit, "CHANGELOG.md must contain Unreleased release notes"
        ):
            self.run_script()


if __name__ == "__main__":
    unittest.main()
