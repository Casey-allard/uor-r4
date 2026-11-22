"""Execute FA-02 against the real authoring adapter in isolated source copies."""
import pathlib
import shutil
import subprocess
import sys
import tempfile
import unittest

STUDIO = pathlib.Path(__file__).resolve().parents[1]
OUTPUTS = [f"src/{name}.lex.tex" for name in ("Workspace", "Inference", "Object", "Publish", "Forge")] + ["model/roots.txt"]


class AuthoringBoundary(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = pathlib.Path(self.temp.name)
        shutil.copytree(STUDIO / "tools", self.root / "tools", ignore=shutil.ignore_patterns("__pycache__"))
        for name in OUTPUTS:
            target = self.root / name
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(STUDIO / name, target)

    def snapshot(self):
        return {str(p.relative_to(self.root)): p.read_bytes() for p in self.root.rglob("*") if p.is_file()}

    def run_author(self, *args):
        return subprocess.run([sys.executable, "-B", str(self.root / "tools/author.py"), *args], cwd=self.root, capture_output=True, text=True, timeout=30)

    def check(self, expected):
        before = self.snapshot()
        result = self.run_author("--check")
        self.assertEqual(result.returncode, expected, result.stderr)
        self.assertEqual(before, self.snapshot(), "check mode modified its inputs or projections")
        return result

    def test_current(self):
        self.assertIn("current (6 projections)", self.check(0).stdout)

    def test_every_projection_drift(self):
        for name in OUTPUTS:
            with self.subTest(name=name):
                path = self.root / name
                original = path.read_bytes()
                path.write_bytes(original + b"unexpected byte\n")
                self.assertIn(f"author:divergedProjection: {name}", self.check(1).stderr)
                path.write_bytes(original)
                self.check(0)

    def test_every_missing_projection(self):
        for name in OUTPUTS:
            with self.subTest(name=name):
                path = self.root / name
                original = path.read_bytes()
                path.unlink()
                self.assertIn(f"author:missingProjection: {name}", self.check(1).stderr)
                self.assertFalse(path.exists())
                path.write_bytes(original)

    def test_unreadable_projection(self):
        path = self.root / OUTPUTS[0]
        path.unlink()
        path.mkdir()
        self.assertIn(f"author:unreadableProjection: {OUTPUTS[0]}", self.check(1).stderr)
        self.assertTrue(path.is_dir())

    def test_all_failures_are_reported(self):
        for name in OUTPUTS:
            (self.root / name).unlink()
        result = self.check(1)
        self.assertEqual(result.stderr.count("author:missingProjection:"), len(OUTPUTS))

    def test_newline_bytes_are_not_normalized(self):
        path = self.root / OUTPUTS[0]
        path.write_bytes(path.read_bytes().replace(b"\n", b"\r\n"))
        self.assertIn("author:divergedProjection:", self.check(1).stderr)

    def test_authoring_drift_requires_regeneration(self):
        path = self.root / "tools/model_workspace.py"
        original = path.read_text()
        changed = original.replace("Own Your Ideas", "Changed Ideas")
        self.assertNotEqual(original, changed)
        path.write_text(changed)
        self.assertIn("author:divergedProjection: src/Forge.lex.tex", self.check(1).stderr)
        self.assertEqual(self.run_author().returncode, 0)
        self.check(0)

    def test_regeneration_is_byte_identical(self):
        before = self.snapshot()
        result = self.run_author()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(before, self.snapshot())
        self.check(0)


if __name__ == "__main__":
    unittest.main()
