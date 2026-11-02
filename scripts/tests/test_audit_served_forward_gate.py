import copy
import importlib.util
import hashlib
from pathlib import Path
import tempfile
import unittest


SCRIPT = Path(__file__).parents[1] / "audit_served_forward_gate.py"
SPEC = importlib.util.spec_from_file_location("audit_served_forward_gate", SCRIPT)
assert SPEC is not None and SPEC.loader is not None
gate = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(gate)


def report():
    decision = "a" * 64
    return {
        "schema": gate.SCHEMA,
        "substrate": gate.SUBSTRATE,
        "qualification": gate.QUALIFICATION,
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


def validate(value):
    gate.validate_report(
        value,
        artifact_bytes=123,
        artifact_sha256="b" * 64,
        token_source_sha256="c" * 64,
        token_sequence_sha256="e" * 64,
        token_offset=7,
        token_count=3,
        total_tokens=20,
        vocabulary=512,
        expected_artifact_sha256=None,
    )


class GateTests(unittest.TestCase):
    def test_complete_report_passes(self):
        validate(report())

    def test_every_decision_and_provenance_class_is_enforced(self):
        replacements = [
            ("timing", {}),
            ("token_source", {"format": "text"}),
            ("manifest", {"shape": {"width": 65, "kept_cells": 8}}),
            ("token_sequence_sha256", "f" * 64),
        ]
        for key, value in replacements:
            changed = report()
            changed[key] = value
            with self.assertRaises(gate.GateError):
                validate(changed)
        for key in (
            "query_coordinate_mismatches",
            "token_mismatches",
            "kept_cell_mismatches",
        ):
            changed = report()
            changed["parity"][key] = 1
            with self.assertRaises(gate.GateError):
                validate(changed)
        changed = report()
        changed["parity"]["serving_decision_sha256"] = "d" * 64
        with self.assertRaises(gate.GateError):
            validate(changed)
        changed = report()
        changed["parity"]["cpu_decision_sha256"] = "z" * 64
        changed["parity"]["serving_decision_sha256"] = "z" * 64
        with self.assertRaises(gate.GateError):
            validate(changed)
        changed = report()
        changed["parity"]["mismatches_recorded"] = 1
        changed["parity"]["mismatches"] = [{"component": "fake"}]
        with self.assertRaises(gate.GateError):
            validate(changed)

    def test_uort_span_and_selected_sequence_hash_are_checked(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "tokens.u16"
            data = bytearray(64)
            data[:4] = b"UORT"
            data[4:8] = (1).to_bytes(4, "little")
            data[8:16] = (4).to_bytes(8, "little")
            data[16:20] = (512).to_bytes(4, "little")
            data.extend(b"".join(token.to_bytes(2, "little") for token in [5, 7, 11, 13]))
            path.write_bytes(data)
            source, sequence, total, vocabulary = gate.read_uort(path, 1, 2)
            self.assertEqual(source, hashlib.sha256(data).hexdigest())
            self.assertEqual(
                sequence,
                hashlib.sha256(b"".join(token.to_bytes(4, "little") for token in [7, 11])).hexdigest(),
            )
            self.assertEqual((total, vocabulary), (4, 512))
            with self.assertRaises(gate.GateError):
                gate.read_uort(path, 3, 2)

    def test_output_aliases_are_refused_up_front(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            artifact = root / "artifact"
            uort = root / "tokens"
            with self.assertRaises(gate.GateError):
                gate.validate_output_paths(
                    artifact=artifact,
                    uort=uort,
                    report=root / "output",
                    promote=root / "output",
                )
            with self.assertRaises(gate.GateError):
                gate.validate_output_paths(
                    artifact=artifact,
                    uort=uort,
                    report=artifact,
                    promote=None,
                )
            with self.assertRaises(gate.GateError):
                gate.validate_output_paths(
                    artifact=artifact,
                    uort=uort,
                    report=None,
                    promote=uort,
                )

    def test_promotion_refuses_an_existing_destination(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / "source"
            destination = root / "destination"
            source.write_bytes(b"model")
            destination.write_bytes(b"keep")
            with self.assertRaises(gate.GateError):
                gate.atomic_copy(source, destination, gate.sha256_bytes(b"model"))
            self.assertEqual(destination.read_bytes(), b"keep")

    def test_promotion_copies_the_exact_bytes(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / "source"
            destination = root / "destination"
            source.write_bytes(b"model")
            gate.atomic_copy(source, destination, gate.sha256_bytes(b"model"))
            self.assertEqual(destination.read_bytes(), b"model")

    def test_promotion_is_bound_to_the_audited_hash(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / "source"
            destination = root / "destination"
            source.write_bytes(b"changed")
            with self.assertRaises(gate.GateError):
                gate.atomic_copy(source, destination, gate.sha256_bytes(b"audited"))
            self.assertFalse(destination.exists())

    def test_json_publication_refuses_an_existing_destination(self):
        with tempfile.TemporaryDirectory() as directory:
            destination = Path(directory) / "report.json"
            destination.write_text("keep\n", encoding="utf-8")
            with self.assertRaises(gate.GateError):
                gate.atomic_json(destination, {"new": True})
            self.assertEqual(destination.read_text(encoding="utf-8"), "keep\n")

    def test_report_tampering_is_not_accepted(self):
        changed = copy.deepcopy(report())
        changed["parity"]["query_coordinates_compared"] = 191
        with self.assertRaises(gate.GateError):
            validate(changed)


if __name__ == "__main__":
    unittest.main()
