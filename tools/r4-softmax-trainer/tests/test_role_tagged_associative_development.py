"""Focused contract tests for the #1045 open-development runner."""

from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path

from r4_softmax_trainer import role_tagged_associative_development as subject


class RoleTaggedAssociativeDevelopmentTests(unittest.TestCase):
    def test_execution_plan_selection_uses_measured_fastest_eligible_plan(self) -> None:
        records = []
        for threads in subject.ELIGIBLE_THREADS:
            for batch_size in subject.ELIGIBLE_BATCH_SIZES:
                projected = float(2_000 - 100 * threads - batch_size)
                if threads == 4 and batch_size == 64:
                    projected = 120.0
                records.append(
                    {
                        "plan": subject.ExecutionPlan(threads, batch_size).record(),
                        "deterministic_replay": True,
                        "projected_r1_seconds": projected,
                        "peak_memory_bytes": 1_000_000,
                    }
                )
        selection = subject.select_execution_plan(records)
        self.assertTrue(selection["available"])
        self.assertEqual(selection["selected_plan"]["threads"], 4)
        self.assertEqual(selection["selected_plan"]["batch_size"], 64)
        self.assertEqual(selection["selected_projection_seconds"], 120.0)

        with self.assertRaisesRegex(ValueError, "incomplete"):
            subject.select_execution_plan(records[:-1])

    def test_decision_stops_at_the_first_failed_gate(self) -> None:
        mechanics = subject.decide_mqar(
            mechanics_passed=False,
            preflight_available=False,
            train_rate=None,
            native_development_rate=None,
            consecutive_passes=0,
            native_control_rate=None,
            current_only_rate=None,
            value_permuted_rate=None,
            binding_permuted_rate=None,
        )
        self.assertEqual(
            mechanics["verdict"], "OPEN_MECHANICS_OR_OPTIMIZER_FAILURE"
        )

        native = subject.decide_mqar(
            mechanics_passed=True,
            preflight_available=True,
            train_rate=0.999,
            native_development_rate=0.98,
            consecutive_passes=0,
            native_control_rate=None,
            current_only_rate=None,
            value_permuted_rate=None,
            binding_permuted_rate=None,
        )
        self.assertEqual(native["verdict"], "OPEN_MQAR_NOT_LEARNED")
        self.assertFalse(native["gates"]["development_absolute"])

        one_pass = subject.decide_mqar(
            mechanics_passed=True,
            preflight_available=True,
            train_rate=0.999,
            native_development_rate=0.995,
            consecutive_passes=1,
            native_control_rate=None,
            current_only_rate=None,
            value_permuted_rate=None,
            binding_permuted_rate=None,
        )
        self.assertEqual(one_pass["verdict"], "OPEN_MQAR_NOT_LEARNED")
        self.assertFalse(one_pass["gates"]["two_consecutive_passes"])

        attributed = subject.decide_mqar(
            mechanics_passed=True,
            preflight_available=True,
            train_rate=0.999,
            native_development_rate=0.995,
            consecutive_passes=2,
            native_control_rate=0.995,
            current_only_rate=0.20,
            value_permuted_rate=0.10,
            binding_permuted_rate=0.15,
        )
        self.assertEqual(attributed["verdict"], "OPEN_MQAR_LEARNED")
        self.assertTrue(attributed["passed"])

        weak_control = subject.decide_mqar(
            mechanics_passed=True,
            preflight_available=True,
            train_rate=0.999,
            native_development_rate=0.995,
            consecutive_passes=2,
            native_control_rate=0.995,
            current_only_rate=0.60,
            value_permuted_rate=0.10,
            binding_permuted_rate=0.15,
        )
        self.assertEqual(weak_control["verdict"], "OPEN_MQAR_NOT_LEARNED")
        self.assertFalse(weak_control["gates"]["current_only_drop"])

    def test_preparation_binds_only_the_named_open_inputs(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            base = Path(directory)
            source = base / "source"
            run = base / "run"
            tokenizer = base / "tokenizer.json"
            tokenizer.write_bytes(b"tokenizer")
            for relative in (
                subject.INPUT_INITIAL_ARTIFACT,
                subject.INPUT_GEOMETRY,
                subject.INPUT_H4_FRAMES,
                subject.INPUT_PUBLIC_MANIFEST,
                subject.INPUT_PUBLIC_COMMITMENT,
                subject.INPUT_CONSTRUCTION_MQAR,
                subject.INPUT_CONSTRUCTION_ENGLISH,
                subject.INPUT_CONSTRUCTION_NATURAL,
                subject.INPUT_CONSTRUCTION_NATURAL_SELECTION,
            ):
                path = source / relative
                path.parent.mkdir(parents=True, exist_ok=True)
                if relative == subject.INPUT_PUBLIC_MANIFEST:
                    path.write_text(
                        json.dumps(
                            {
                                "tokenizer": {
                                    "path": str(tokenizer),
                                    "cid": subject.cid_file(tokenizer),
                                }
                            }
                        ),
                        encoding="utf-8",
                    )
                else:
                    path.write_bytes(relative.encode("utf-8"))
            forbidden = source / "artifact/model.safetensors"
            forbidden.parent.mkdir(parents=True)
            forbidden.write_bytes(b"must-not-be-read")
            sealed = source / "evaluation/sealed/mqar.json"
            sealed.parent.mkdir(parents=True)
            sealed.write_bytes(b"must-not-be-read")

            prepared = subject.prepare_role_tagged_associative_development(
                run,
                source_root=source,
            )
            self.assertEqual(prepared["sealed_input_reads"], 0)
            self.assertEqual(prepared["failed_source_artifact_reads"], 0)
            self.assertEqual(
                {record["path"] for record in prepared["inputs"]},
                {
                    subject.INPUT_INITIAL_ARTIFACT,
                    subject.INPUT_GEOMETRY,
                    subject.INPUT_H4_FRAMES,
                    subject.INPUT_PUBLIC_MANIFEST,
                    subject.INPUT_PUBLIC_COMMITMENT,
                    subject.INPUT_CONSTRUCTION_MQAR,
                    subject.INPUT_CONSTRUCTION_ENGLISH,
                    subject.INPUT_CONSTRUCTION_NATURAL,
                    subject.INPUT_CONSTRUCTION_NATURAL_SELECTION,
                    "external/tokenizer.json",
                },
            )
            unsigned = dict(prepared)
            observed = unsigned.pop("preparation_cid")
            self.assertEqual(
                observed,
                subject.cid_bytes(subject.canonical_json_bytes(unsigned)),
            )
            self.assertEqual(forbidden.read_bytes(), b"must-not-be-read")
            self.assertEqual(sealed.read_bytes(), b"must-not-be-read")

            on_disk = json.loads(
                (run / subject.PREPARATION_RELATIVE_PATH).read_text("utf-8")
            )
            self.assertEqual(on_disk, prepared)

    def test_result_boundary_marks_later_work_not_run(self) -> None:
        body = subject._result_body(
            preparation={"preparation_cid": "blake3:" + "1" * 64},
            preflight={
                "preflight_cid": "blake3:" + "2" * 64,
                "implementation": {"tree_cid": "blake3:" + "3" * 64},
            },
            plan=subject.ExecutionPlan(4, 64).record(),
            artifact=b"model",
            fit={"status": "COMPLETE"},
            metrics={"mqar": {}},
            decision={
                "verdict": "OPEN_MQAR_NOT_LEARNED",
                "passed": False,
            },
            elapsed_seconds=1.0,
        )
        self.assertEqual(body["generation"], "NOT_RUN")
        self.assertEqual(body["reasoning"], "NOT_RUN")
        self.assertEqual(body["lowering"], "NOT_RUN")
        self.assertEqual(body["later_rungs"]["english_transfer"]["status"], "NOT_RUN")
        self.assertEqual(body["sealed_input_reads"], 0)


if __name__ == "__main__":
    unittest.main()
