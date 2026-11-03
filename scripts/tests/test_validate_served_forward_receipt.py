import argparse
import copy
import hashlib
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest


SCRIPT = Path(__file__).parents[1] / "validate_served_forward_receipt.py"
SPEC = importlib.util.spec_from_file_location("validate_served_forward_receipt", SCRIPT)
assert SPEC is not None and SPEC.loader is not None
receipt = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(receipt)


def report():
    decision = "a" * 64
    return {
        "schema": receipt.GATE.SCHEMA,
        "substrate": receipt.GATE.SUBSTRATE,
        "qualification": receipt.GATE.QUALIFICATION,
        "artifact": {"sha256": "b" * 64, "bytes": 123},
        "timing": None,
        "token_source": {
            "format": "uort",
            "source_sha256": "c" * 64,
            "offset": 7,
            "count": 3,
            "total_tokens": 20,
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


class ReceiptTests(unittest.TestCase):
    def validate(self, value, *, artifact_sha="b" * 64, artifact_bytes=123):
        receipt.validate_receipt(
            value,
            artifact_sha256=artifact_sha,
            artifact_bytes=artifact_bytes,
        )

    def test_valid_local_and_remote_identities_pass(self):
        self.validate(report())
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "artifact.uorlut"
            path.write_bytes(b"model")
            digest = hashlib.sha256(b"model").hexdigest()
            args = argparse.Namespace(
                artifact=str(path),
                artifact_sha256=digest,
                artifact_bytes=5,
            )
            self.assertEqual(receipt.artifact_identity(args), (digest, 5))

    def test_artifact_and_receipt_hashes_are_bound(self):
        with self.assertRaises(receipt.ReceiptError):
            self.validate(report(), artifact_sha="d" * 64)
        with self.assertRaises(receipt.ReceiptError):
            self.validate(report(), artifact_bytes=124)
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "audit.json"
            path.write_text(json.dumps(report()), encoding="utf-8")
            actual = hashlib.sha256(path.read_bytes()).hexdigest()
            loaded, found = receipt.load_receipt(path, actual)
            self.assertEqual(loaded, report())
            self.assertEqual(found, actual)
            with self.assertRaises(receipt.ReceiptError):
                receipt.load_receipt(path, "f" * 64)

    def test_parity_and_provenance_tampering_is_refused(self):
        changes = [
            ("parity", "query_coordinates_compared", 191),
            ("parity", "token_mismatches", 1),
            ("parity", "kept_cell_mismatches", 1),
            ("parity", "instrument_fault", True),
            ("parity", "serving_decision_sha256", "d" * 64),
            ("token_source", "offset", -1),
            ("token_source", "count", 0),
            ("token_source", "total_tokens", 9),
        ]
        for section, key, value in changes:
            changed = copy.deepcopy(report())
            changed[section][key] = value
            with self.subTest(section=section, key=key), self.assertRaises(
                receipt.ReceiptError
            ):
                self.validate(changed)

    def test_non_hex_hashes_and_mismatch_details_are_refused(self):
        changed = copy.deepcopy(report())
        changed["token_sequence_sha256"] = "z" * 64
        with self.assertRaises(receipt.ReceiptError):
            self.validate(changed)
        changed = copy.deepcopy(report())
        changed["parity"]["mismatches_recorded"] = 1
        changed["parity"]["mismatches"] = [{"component": "planted"}]
        with self.assertRaises(receipt.ReceiptError):
            self.validate(changed)


if __name__ == "__main__":
    unittest.main()
