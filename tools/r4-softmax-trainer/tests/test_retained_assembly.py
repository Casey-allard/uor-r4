"""Focused synthetic checks of retained evidence admission, with no model work."""

import importlib.util
import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch


ROOT = Path(__file__).resolve().parents[3]
MODULE = ROOT / "tools/r4-softmax-trainer/src/r4_softmax_trainer/text_clause_adapter/retained.py"
SPEC = importlib.util.spec_from_file_location("retained_protocol_for_check", MODULE)
retained = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(retained)


class RetainedAssemblyChecks(unittest.TestCase):
    def test_executing_module_must_be_the_bound_coordinator(self):
        retained._executing_repo(ROOT)
        other, output = Path("/synthetic/other-checkout"), Path("/synthetic/output")
        with patch.object(retained, "_git", side_effect=AssertionError("source read before identity")) as reads:
            for action in (lambda: retained.assemble(other, output),
                           lambda: retained.load_for_release(output / "retained-preparation.json", repo=other, output=output),
                           lambda: retained.verify_runtime({"coordinator_source": {"repo": str(other)}})):
                with self.assertRaisesRegex(ValueError, "executing retained module"):
                    action()
            reads.assert_not_called()

    def test_committed_source_rejects_changed_missing_and_extra_python(self):
        with tempfile.TemporaryDirectory() as directory:
            repo = Path(directory).resolve()
            paths = [retained.PACKAGE + "/__init__.py",
                     retained.PACKAGE + "/text_clause_adapter/worker.py", *retained.SOURCE_EXTRAS]
            for name in paths:
                path = repo / name
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_bytes(b"# synthetic source only\n")
            def git(*args):
                return subprocess.check_output(["/usr/bin/git", "-C", str(repo), *args], stderr=subprocess.DEVNULL)
            git("init", "-q")
            git("add", "--", *paths)
            git("-c", "user.name=synthetic", "-c", "user.email=synthetic@invalid", "commit", "-qm", "fixture")
            revision = git("rev-parse", "HEAD").decode().strip()
            expected = retained.source_identity(repo, revision)
            self.assertEqual(len(paths), len(expected["files"]))
            worker = repo / paths[1]
            worker.write_bytes(b"# changed bytes\n")
            with self.assertRaisesRegex(ValueError, "committed bytes"):
                retained.source_identity(repo, revision)
            worker.write_bytes(b"# synthetic source only\n")
            worker.unlink()
            with self.assertRaisesRegex(ValueError, "missing or uncommitted"):
                retained.source_identity(repo, revision)
            worker.write_bytes(b"# synthetic source only\n")
            extra = repo / retained.PACKAGE / "extra.py"
            extra.write_bytes(b"# uncommitted importable source\n")
            with self.assertRaisesRegex(ValueError, "missing or uncommitted"):
                retained.source_identity(repo, revision)
            extra.unlink()
            extra.symlink_to(worker)
            with self.assertRaisesRegex(ValueError, "symlink"):
                retained.source_identity(repo, revision)

    def test_profile_reproduces_readiness_then_only_rebinds_and_narrows(self):
        base = ROOT / "docs/r4_isolated_runtime_readiness_1096_evidence/admitted-freeze02"
        ready = json.loads((base / "manifest.json").read_bytes())
        binding = json.loads((base / "bindings.json").read_bytes())
        old = (base / "worker.sb").read_text()
        repo = Path(ready["repo"])
        extras = (str(Path(ready["profile"]["path"]).parent / "manifest.json"), ready["profile"]["path"])
        recreated = retained.profile_from_metadata(repo, Path(ready["bindings"]["path"]), ready,
                                                    binding["assets"], extras=extras)
        self.assertEqual(old, recreated)
        output = Path.home() / ".codex/uor/synthetic-retained-assembly-not-created"
        new = retained.profile_from_metadata(repo, output / "bindings.json", ready, binding["assets"])
        delta = retained._profile_delta(old, new, ready, output)
        self.assertEqual(2, len(delta["removed_readiness_literals"]))
        self.assertNotIn(extras[0], new)
        self.assertNotIn(extras[1], new)
        self.assertIn(str(output / "bindings.json"), new)
        self.assertFalse(delta["worker_source_tree_rebound"])
        for item in delta["added"]:
            self.assertNotIn("subpath", item["line"])
        self.assertEqual(3, next(i["occurrences"] for i in delta["added"]
                                if str(output / "bindings.json") in i["line"]))

    def test_clean_environment_does_not_inherit_parent_python_or_loader_flags(self):
        with patch.dict("os.environ", {"PYTHONHOME": "/synthetic/bad", "DYLD_INSERT_LIBRARIES": "bad",
                                       "PYTHONPATH": "/synthetic/injected", "OMP_NUM_THREADS": "99"}):
            env = retained._environment(Path("/synthetic/worker"), Path("/synthetic/output"))
        self.assertNotIn("PYTHONHOME", env)
        self.assertNotIn("DYLD_INSERT_LIBRARIES", env)
        self.assertEqual("/synthetic/worker/tools/r4-softmax-trainer/src", env["PYTHONPATH"])
        self.assertEqual("4", env["OMP_NUM_THREADS"])
        self.assertEqual("1", env["PYTHONNOUSERSITE"])
        self.assertEqual("/synthetic/output/isolation-probe.txt", env["UOR_ISOLATION_PROBE"])

    def test_output_path_rejects_corpus_overlap_and_symlink_without_payload_reads(self):
        with tempfile.TemporaryDirectory() as directory:
            home = Path(directory).resolve()
            corpus = home / "corpus"
            corpus.mkdir()
            sealed = corpus / "withheld"
            sealed.mkdir(mode=0)
            with patch.object(Path, "home", return_value=home):
                for output in (corpus, home, sealed / "output"):
                    with self.assertRaisesRegex(ValueError, "overlaps"):
                        retained._disjoint_output(output, [corpus])
                safe = home / "new-output"
                retained._disjoint_output(safe, [corpus])
                link = home / "alias"
                link.symlink_to(corpus, target_is_directory=True)
                with self.assertRaisesRegex(ValueError, "symlink"):
                    retained._disjoint_output(link / "output", [corpus])
            sealed.chmod(0o700)  # synthetic fixture cleanup only

    def test_sealed_metadata_accepts_only_original_identity_and_release_read_mode(self):
        with tempfile.TemporaryDirectory() as directory:
            corpus = Path(directory).resolve()
            sealed = corpus / "withheld"
            sealed.mkdir(mode=0)
            first = retained._sealed(corpus, require_sealed=True)
            sealed.chmod(0o500)
            self.assertEqual(first, retained._sealed(corpus, require_sealed=False))
            with self.assertRaises(ValueError):
                retained._sealed(corpus, require_sealed=True)
            sealed.chmod(0o700)
            with self.assertRaises(ValueError):
                retained._sealed(corpus, require_sealed=False)

    def test_release_requires_new_status_and_every_exact_hash(self):
        assembly = {key: {"sha256": key + "-digest"} for key in
                    ("assembly_record", "bindings", "sandbox", "corpus_manifest", "selection")}
        for key in ("coordinator_source", "worker_source", "clean_environment", "runtime_identity", "profile_delta"):
            assembly[key] = {"synthetic": key}
        valid = {"schema": retained.RELEASE_SCHEMA, "status": retained.RELEASE_STATUS,
                 "issue": 1094, "reviewer": "synthetic independent fixture", **retained.release_bindings(assembly)}
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory).resolve() / "release.json"
            assembly["output_paths"] = {"output": str(path.parent)}
            path.write_bytes(retained.canonical(valid))
            self.assertEqual(valid["status"], retained.verify_release(assembly, path)["status"])
            for key in retained.release_bindings(assembly):
                wrong = dict(valid)
                wrong[key] = "different"
                path.write_bytes(retained.canonical(wrong))
                with self.assertRaisesRegex(ValueError, "exact assembled"):
                    retained.verify_release(assembly, path)
            old = dict(valid, status="ACCEPTED_FOR_FROZEN_COMPARISON")
            path.write_bytes(retained.canonical(old))
            with self.assertRaisesRegex(ValueError, "release is missing"):
                retained.verify_release(assembly, path)
            path.unlink()
            alias_source = path.parent / "synthetic-payload-never-read"
            alias_source.write_bytes(b"not JSON; must never reach the parser")
            os.link(alias_source, path)
            with patch.object(retained, "_json", side_effect=AssertionError("payload parsed")) as parser:
                with self.assertRaisesRegex(ValueError, "unique regular file"):
                    retained.verify_release(assembly, path)
                with self.assertRaisesRegex(ValueError, "unique regular file"):
                    retained._read_metadata(path)
                parser.assert_not_called()


if __name__ == "__main__":
    unittest.main()
