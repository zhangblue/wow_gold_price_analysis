import tempfile
import unittest
from pathlib import Path

from tools.build_release import assemble_release


class AssembleReleaseTests(unittest.TestCase):
    def setUp(self):
        self.workspace = tempfile.TemporaryDirectory()
        self.root = Path(self.workspace.name)
        self.binary = self.root / "gold-price-backend"
        self.binary.write_text("binary")
        self.ui = self.root / "ui-dist"
        self.ui.mkdir()
        (self.ui / "index.html").write_text("<main>gold</main>")
        self.template = self.root / ".env.example"
        self.template.write_text("DATABASE_URL=postgres://example/db\n")
        self.release = self.root / "release"

    def tearDown(self):
        self.workspace.cleanup()

    def test_assemble_release_copies_binary_ui_and_runtime_directories(self):
        assemble_release(self.binary, self.ui, self.template, self.release)

        self.assertTrue((self.release / "gold-price-backend").is_file())
        self.assertTrue((self.release / "dist/index.html").is_file())
        self.assertEqual(
            (self.release / "config/.env").read_text(), self.template.read_text()
        )
        self.assertTrue((self.release / "logs").is_dir())

    def test_assemble_release_preserves_existing_runtime_configuration(self):
        config = self.release / "config"
        config.mkdir(parents=True)
        existing = config / ".env"
        existing.write_text("DATABASE_URL=postgres://operator/production\n")

        assemble_release(self.binary, self.ui, self.template, self.release)

        self.assertEqual(existing.read_text(), "DATABASE_URL=postgres://operator/production\n")

    def test_assemble_release_replaces_stale_ui_files_without_touching_config(self):
        stale_dist = self.release / "dist"
        stale_dist.mkdir(parents=True)
        (stale_dist / "removed-in-new-build.js").write_text("stale")
        config = self.release / "config"
        config.mkdir()
        existing = config / ".env"
        existing.write_text("DATABASE_URL=postgres://operator/production\n")

        assemble_release(self.binary, self.ui, self.template, self.release)

        self.assertFalse((stale_dist / "removed-in-new-build.js").exists())
        self.assertTrue((stale_dist / "index.html").is_file())
        self.assertEqual(existing.read_text(), "DATABASE_URL=postgres://operator/production\n")


if __name__ == "__main__":
    unittest.main()
