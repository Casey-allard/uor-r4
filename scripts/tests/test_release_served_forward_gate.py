import hashlib
import importlib.util
import json
from pathlib import Path
import subprocess
import tempfile
import unittest


REPO = Path(__file__).parents[2]
PACKAGE_SCRIPT = REPO / "scripts/package_release_bundle_integer.py"
SPEC = importlib.util.spec_from_file_location("package_release_bundle_integer", PACKAGE_SCRIPT)
assert SPEC is not None and SPEC.loader is not None
package = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(package)


def uorlut_header(width=64, kept_cells=8, vocab=512):
    header = json.dumps(
        {"schema": "uor-r4.test/1", "shape": {"width": width, "kept_cells": kept_cells, "vocab": vocab}},
        separators=(",", ":"),
    ).encode()
    return b"UORLUT01" + len(header).to_bytes(8, "little") + header + b"\0" * 8


def audit_report(artifact):
    decision = "a" * 64
    return {
        "schema": "uor-r4.cpu-served-forward-audit/1",
        "substrate": "cpu-development-research",
        "qualification": "gpu-required",
        "artifact": {
            "sha256": hashlib.sha256(artifact).hexdigest(),
            "bytes": len(artifact),
        },
        "timing": None,
        "token_source": {
            "format": "uort",
            "source_sha256": "c" * 64,
            "offset": 0,
            "count": 3,
            "total_tokens": 3,
            "vocabulary": 512,
        },
        "token_sequence_sha256": "e" * 64,
        "manifest": {"shape": {"width": 64, "kept_cells": 8}},
        "parity": {
            "positions_checked": 3,
            "query_coordinates_compared": 192,
            "token_decisions_compared": 3,
            "kept_cells_compared": 24,
            "query_coordinate_mismatches": 0,
            "token_mismatches": 0,
            "kept_cell_mismatches": 0,
            "instrument_fault": False,
            "cpu_decision_sha256": decision,
            "serving_decision_sha256": decision,
            "mismatches_recorded": 0,
            "mismatches_omitted": 0,
            "mismatches": [],
        },
    }


def write_receipt(path, artifact, report=None):
    value = report if report is not None else audit_report(artifact)
    path.write_text(json.dumps(value, indent=2) + "\n", encoding="utf-8")
    return hashlib.sha256(path.read_bytes()).hexdigest()


def studio_source(root):
    source = root / "studio-src"
    (source / "pkg").mkdir(parents=True)
    (source / "index.html").write_text("<!doctype html>\n", encoding="utf-8")
    for name in ("uor_r4_integer.js", "uor_r4_integer_bg.wasm"):
        (source / "pkg" / name).write_bytes(name.encode())
    (source / "pkg/synthetic-stack.lut").write_bytes(uorlut_header())
    (source / "pkg/tokenizer.json").write_text("{}\n", encoding="utf-8")
    return source


class ReleaseReceiptTests(unittest.TestCase):
    def test_package_requires_an_adjacent_valid_receipt(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            artifact = root / "model.uorlut"
            artifact.write_bytes(uorlut_header())
            with self.assertRaises(package.ManifestError):
                package.served_forward_audit(str(artifact))
            receipt_path = Path(str(artifact) + ".audit.json")
            write_receipt(receipt_path, artifact.read_bytes())
            reading = package.served_forward_audit(str(artifact))
            self.assertEqual(reading["positions"], 3)
            self.assertEqual(reading["query_coordinates"], 192)
            self.assertEqual(
                reading["cpu_decision_sha256"], reading["serving_decision_sha256"]
            )

    def test_studio_stages_only_a_receipted_trained_artifact(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = studio_source(root)
            artifact = root / "trained.uorlut"
            artifact.write_bytes(uorlut_header())
            artifact_sha = hashlib.sha256(artifact.read_bytes()).hexdigest()
            tokenizer = root / "trained.tokenizer.json"
            tokenizer.write_text(
                json.dumps({"stack_sha256": artifact_sha, "vocab_size": 512}),
                encoding="utf-8",
            )
            audit = root / "trained.audit.json"
            audit_sha = write_receipt(audit, artifact.read_bytes())
            manifest = root / "models.json"
            manifest.write_text(
                json.dumps(
                    [
                        {
                            "id": "trained-model",
                            "label": "Trained model",
                            "file": str(artifact),
                            "tokenizer": str(tokenizer),
                            "audit": str(audit),
                            "audit_sha256": audit_sha,
                        },
                        {
                            "id": "remote-model",
                            "label": "Remote model",
                            "hf_url": "https://example.invalid/remote-model.uorlut",
                            "hf_tokenizer_url": "https://example.invalid/tokenizer.json",
                            "sha256": artifact_sha,
                            "size_bytes": len(artifact.read_bytes()),
                            "audit": str(audit),
                            "audit_sha256": audit_sha,
                        }
                    ]
                ),
                encoding="utf-8",
            )
            destination = root / "site"
            completed = subprocess.run(
                [
                    "bash",
                    str(REPO / "scripts/stage-studio-site.sh"),
                    str(source),
                    str(destination),
                    "--trained-manifest",
                    str(manifest),
                ],
                capture_output=True,
                text=True,
            )
            self.assertEqual(completed.returncode, 0, completed.stderr)
            self.assertTrue((destination / "models/trained-model.uorlut").is_file())
            self.assertTrue((destination / "models/trained-model.uorlut.audit.json").is_file())
            models = json.loads((destination / "models.json").read_text(encoding="utf-8"))
            staged = next(model for model in models["models"] if model["id"] == "trained-model")
            self.assertEqual(staged["served_forward_audit_sha256"], audit_sha)
            self.assertEqual(staged["audit_url"], "./models/trained-model.uorlut.audit.json")
            remote = next(model for model in models["models"] if model["id"] == "remote-model")
            self.assertEqual(remote["served_forward_audit_sha256"], audit_sha)
            self.assertNotIn("audit_url", remote)

    def test_studio_refuses_missing_and_tampered_receipts_before_copy(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = studio_source(root)
            artifact = root / "trained.uorlut"
            artifact.write_bytes(uorlut_header())
            artifact_sha = hashlib.sha256(artifact.read_bytes()).hexdigest()
            tokenizer = root / "trained.tokenizer.json"
            tokenizer.write_text(
                json.dumps({"stack_sha256": artifact_sha, "vocab_size": 512}),
                encoding="utf-8",
            )
            audit = root / "trained.audit.json"
            write_receipt(audit, artifact.read_bytes())
            missing = root / "missing.json"
            missing.write_text(
                json.dumps(
                    [
                        {
                            "id": "missing-audit",
                            "label": "Missing audit",
                            "file": str(artifact),
                            "tokenizer": str(tokenizer),
                        }
                    ]
                ),
                encoding="utf-8",
            )
            destination = root / "missing-site"
            completed = subprocess.run(
                [
                    "bash",
                    str(REPO / "scripts/stage-studio-site.sh"),
                    str(source),
                    str(destination),
                    "--trained-manifest",
                    str(missing),
                ],
                capture_output=True,
                text=True,
            )
            self.assertNotEqual(completed.returncode, 0)
            self.assertFalse((destination / "models/missing-audit.uorlut").exists())

            tampered = audit_report(artifact.read_bytes())
            tampered["parity"]["token_mismatches"] = 1
            tampered_sha = write_receipt(audit, artifact.read_bytes(), tampered)
            bad = root / "bad.json"
            bad.write_text(
                json.dumps(
                    [
                        {
                            "id": "bad-audit",
                            "label": "Bad audit",
                            "file": str(artifact),
                            "tokenizer": str(tokenizer),
                            "audit": str(audit),
                            "audit_sha256": tampered_sha,
                        }
                    ]
                ),
                encoding="utf-8",
            )
            destination = root / "bad-site"
            completed = subprocess.run(
                [
                    "bash",
                    str(REPO / "scripts/stage-studio-site.sh"),
                    str(source),
                    str(destination),
                    "--trained-manifest",
                    str(bad),
                ],
                capture_output=True,
                text=True,
            )
            self.assertNotEqual(completed.returncode, 0)
            self.assertFalse((destination / "models/bad-audit.uorlut").exists())


if __name__ == "__main__":
    unittest.main()
