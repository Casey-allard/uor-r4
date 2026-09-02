"""Focused lifecycle tests for the credited #1047 Zoology control."""

from __future__ import annotations

import tempfile
import time
import unittest
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import patch

import torch

from r4_softmax_trainer.zoology_control import development as subject


def _score(rate: float) -> subject.ScoreResult:
    decisions = 100
    return subject.ScoreResult(
        decisions=decisions,
        correct=round(rate * decisions),
        loss_sum=10.0,
        selected_logits_cid="blake3:" + "a" * 64,
        work={"target_reads": decisions},
    )


def _passing_fit(population_cid: str) -> dict[str, object]:
    return {
        "status": "COMPLETE",
        "passed": True,
        "population_cid": population_cid,
        "epochs": 2,
        "query_presentations": 2,
        "history": [],
        "final_train": _score(1.0).record(),
        "final_development": _score(1.0).record(),
        "consecutive_passes": 2,
        "incomplete_reason": None,
        "elapsed_seconds": 1.0,
        "optimizer": {},
    }


class ZoologyControlDevelopmentTests(unittest.TestCase):
    def test_source_oracle_is_executed_inside_c0_contract(self) -> None:
        golden = subject._source_oracle_golden()
        self.assertTrue(golden["passed"])
        self.assertTrue(golden["loader"]["inputs_byte_exact"])
        self.assertTrue(golden["loader"]["labels_byte_exact"])
        self.assertTrue(golden["model"]["parameters_byte_exact"])
        self.assertTrue(golden["model"]["logits_scale_aware_parity"])

    def test_plan_selection_uses_both_measured_arms_and_fastest_cpu(self) -> None:
        records = []
        for threads, projected in ((1, 300.0), (4, 100.0), (8, 150.0)):
            arm = {
                "timed_training_batches": subject.TIMED_TRAINING_BATCHES,
                "full_development_evaluation": True,
                "deterministic_replay": True,
            }
            records.append(
                {
                    "plan": subject.ExecutionPlan(threads).record(),
                    "arms": {"c1": dict(arm), "c2": dict(arm)},
                    "timed_training_batches": 2 * subject.TIMED_TRAINING_BATCHES,
                    "full_development_evaluation": True,
                    "deterministic_replay": True,
                    "projected_c1_c2_seconds": projected,
                    "peak_rss_bytes": 1_000_000,
                }
            )

        selection = subject.select_execution_plan(records)
        self.assertTrue(selection["available"])
        self.assertEqual(selection["selected_plan"]["threads"], 4)
        self.assertEqual(selection["selected_projection_seconds"], 100.0)

        missing_c2 = [dict(record) for record in records]
        missing_c2[1] = {**missing_c2[1], "arms": {"c1": records[1]["arms"]["c1"]}}
        degraded = subject.select_execution_plan(missing_c2)
        self.assertNotEqual(degraded["selected_plan"]["threads"], 4)

    def test_projection_reserves_a_train_score_after_every_epoch(self) -> None:
        with patch.object(subject, "MAXIMUM_EPOCHS", 64):
            projected = subject._project_rung_seconds(
                train_batches=128,
                development_batches=16,
                seconds_per_training_batch=0.02,
                seconds_per_evaluation_batch=0.01,
            )
        expected = 64 * 128 * 0.02 + 64 * (16 + 128) * 0.01
        self.assertEqual(projected, expected)

    def test_hard_wall_evidence_covers_runner_exit_shapes(self) -> None:
        complete = {"status": "COMPLETE"}
        incomplete = {"status": "INCOMPLETE_HARD_WALL"}
        self.assertTrue(
            subject._has_hard_wall_evidence(incomplete, None, None, 1.0)
        )
        self.assertTrue(
            subject._has_hard_wall_evidence(complete, incomplete, None, 1.0)
        )
        self.assertTrue(
            subject._has_hard_wall_evidence(
                complete,
                complete,
                "INCOMPLETE_HARD_WALL",
                1.0,
            )
        )
        self.assertTrue(
            subject._has_hard_wall_evidence(
                complete,
                complete,
                "NOT_RUN_HARD_WALL",
                1.0,
            )
        )
        self.assertTrue(
            subject._has_hard_wall_evidence(
                complete,
                complete,
                "COMPLETE",
                subject.HARD_WALL_SECONDS,
            )
        )
        self.assertFalse(
            subject._has_hard_wall_evidence(complete, complete, "COMPLETE", 1.0)
        )

    def test_hard_wall_decision_requires_matching_evidence(self) -> None:
        complete = {"status": "COMPLETE"}
        decision = subject._incomplete_decision("binding score reached wall")
        self.assertTrue(
            subject._validate_hard_wall_decision(
                decision,
                c1=complete,
                c2=complete,
                binding_control="INCOMPLETE_HARD_WALL",
                elapsed_seconds=1.0,
            )
        )
        with self.assertRaisesRegex(ValueError, "lacks hard-wall evidence"):
            subject._validate_hard_wall_decision(
                decision,
                c1=complete,
                c2=complete,
                binding_control="COMPLETE",
                elapsed_seconds=1.0,
            )
        with self.assertRaisesRegex(ValueError, "scientific verdict"):
            subject._validate_hard_wall_decision(
                {"status": "DECIDED", "verdict": "STOCK_CELL_TRANSFER_MISS"},
                c1=complete,
                c2=complete,
                binding_control="NOT_RUN_HARD_WALL",
                elapsed_seconds=1.0,
            )
        forged = {**decision, "passed": True}
        with self.assertRaisesRegex(ValueError, "lacks hard-wall evidence"):
            subject._validate_hard_wall_decision(
                forged,
                c1=complete,
                c2=complete,
                binding_control="INCOMPLETE_HARD_WALL",
                elapsed_seconds=1.0,
            )

    def test_decision_stops_at_each_frozen_boundary(self) -> None:
        self.assertEqual(
            subject.decide_zoology_control(
                c0_passed=False,
                preflight_available=False,
            )["verdict"],
            "INVALID_CONTROL_PORT",
        )
        self.assertEqual(
            subject.decide_zoology_control(
                c0_passed=True,
                preflight_available=True,
                c1_train_rate=1.0,
                c1_development_rate=0.98,
                c1_consecutive_passes=0,
            )["verdict"],
            "SCALED_SOURCE_CALIBRATION_MISS",
        )
        self.assertEqual(
            subject.decide_zoology_control(
                c0_passed=True,
                preflight_available=True,
                c1_train_rate=1.0,
                c1_development_rate=1.0,
                c1_consecutive_passes=2,
                c2_train_rate=1.0,
                c2_development_rate=0.98,
                c2_consecutive_passes=0,
            )["verdict"],
            "STOCK_CELL_TRANSFER_MISS",
        )
        self.assertEqual(
            subject.decide_zoology_control(
                c0_passed=True,
                preflight_available=True,
                c1_train_rate=1.0,
                c1_development_rate=1.0,
                c1_consecutive_passes=2,
                c2_train_rate=0.98,
                c2_development_rate=0.97,
                c2_consecutive_passes=0,
            )["verdict"],
            "STOCK_CELL_EXACT_QUALIFICATION_MISS",
        )
        self.assertEqual(
            subject.decide_zoology_control(
                c0_passed=True,
                preflight_available=True,
                c1_train_rate=1.0,
                c1_development_rate=1.0,
                c1_consecutive_passes=2,
                c2_train_rate=1.0,
                c2_development_rate=1.0,
                c2_consecutive_passes=1,
            )["verdict"],
            "STOCK_CELL_EXACT_QUALIFICATION_MISS",
        )
        shortcut = subject.decide_zoology_control(
            c0_passed=True,
            preflight_available=True,
            c1_train_rate=1.0,
            c1_development_rate=1.0,
            c1_consecutive_passes=2,
            c2_train_rate=1.0,
            c2_development_rate=1.0,
            c2_consecutive_passes=2,
            binding_permuted_rate=0.75,
        )
        self.assertEqual(shortcut["verdict"], "NONASSOCIATIVE_SHORTCUT")
        passed = subject.decide_zoology_control(
            c0_passed=True,
            preflight_available=True,
            c1_train_rate=1.0,
            c1_development_rate=1.0,
            c1_consecutive_passes=2,
            c2_train_rate=1.0,
            c2_development_rate=1.0,
            c2_consecutive_passes=2,
            binding_permuted_rate=0.10,
        )
        self.assertEqual(passed["verdict"], "STOCK_CELL_PASSES_EXACT_BYTES")
        self.assertTrue(passed["passed"])

    def test_hard_wall_cannot_reuse_a_prior_epoch_score(self) -> None:
        model = torch.nn.Linear(1, 1)
        population = SimpleNamespace(
            train=(object(),),
            development=(object(),),
            population_cid="blake3:" + "b" * 64,
        )
        training = {"query_presentations": 1, "complete": True}
        passed_score = _score(1.0)

        def completed_epoch(
            _model: torch.nn.Module,
            optimizer: torch.optim.Optimizer,
            *_args: object,
            **_kwargs: object,
        ) -> dict[str, object]:
            for parameter in model.parameters():
                parameter.grad = torch.zeros_like(parameter)
            optimizer.step()
            return training

        with (
            patch.object(subject, "MAXIMUM_EPOCHS", 2),
            patch.object(subject, "_new_model", return_value=model),
            patch.object(subject, "_train_epoch", side_effect=completed_epoch),
            patch.object(
                subject,
                "_score_rows",
                side_effect=[
                    passed_score,
                    passed_score,
                    subject.HardWallExceeded("wall"),
                ],
            ),
        ):
            _, fit = subject._train_rung(
                population,
                rung="c1",
                device=torch.device("cpu"),
                deadline=time.monotonic() + 60.0,
            )
        self.assertEqual(fit["status"], "INCOMPLETE_HARD_WALL")
        self.assertIsNone(fit["final_train"])
        self.assertIsNone(fit["final_development"])
        self.assertFalse(fit["passed"])

    def test_result_boundary_keeps_w8_and_geometry_out_of_the_control(self) -> None:
        body = subject._result_body(
            preparation={
                "preparation_cid": "blake3:" + "1" * 64,
                "inputs": {
                    "source_attribution_cid": "blake3:" + "2" * 64,
                    "source_native_population_cid": "blake3:" + "3" * 64,
                    "exact_1045_population_cid": "blake3:" + "4" * 64,
                    "source_1045_split_cid": "blake3:" + "5" * 64,
                },
            },
            preflight={
                "preflight_cid": "blake3:" + "6" * 64,
                "implementation": {"tree_cid": "blake3:" + "7" * 64},
            },
            plan=subject.ExecutionPlan(4).record(),
            c1="NOT_RUN",
            c2="NOT_RUN",
            binding_control="NOT_RUN",
            artifacts=(),
            decision={"status": "NOT_RUN", "verdict": None, "passed": False},
            elapsed_seconds=1.0,
        )
        self.assertEqual(body["claim_boundary"]["ordinary_causal_softmax"], "ONLY")
        self.assertEqual(body["claim_boundary"]["r4_or_geometric_attention"], "NOT_CLAIMED")
        self.assertEqual(body["claim_boundary"]["exact_lowering"], "NOT_RUN")
        self.assertEqual(body["read_work_ledger"]["future_value_reads"], 0)

    def test_c2_pass_reaches_binding_control_with_the_correct_call_shape(self) -> None:
        implementation = {
            "implementation_cid": "blake3:" + "8" * 64,
            "tree_cid": "blake3:" + "9" * 64,
        }
        predecessor = {"result_cid": "blake3:" + "a" * 64}
        source = SimpleNamespace(
            train=(object(),),
            development=(object(),),
            population_cid="blake3:" + "b" * 64,
            source_split_cid=None,
        )
        exact = SimpleNamespace(
            train=(object(),),
            development=(object(),),
            population_cid="blake3:" + "c" * 64,
            source_split_cid=subject.EXPECTED_1045_SPLIT_CID,
        )
        plan = subject.ExecutionPlan(4).record()
        preparation = {
            "preparation_cid": "blake3:" + "d" * 64,
            "implementation": implementation,
            "source_root": "/open/source",
            "predecessor_root": "/open/predecessor",
            "predecessor": predecessor,
        }
        preflight = {
            "preflight_cid": "blake3:" + "e" * 64,
            "preparation_cid": preparation["preparation_cid"],
            "implementation": implementation,
            "passed": True,
            "c0": {"passed": True},
            "selection": {"selected_plan": plan},
            "population_cids": {
                "c1_source_native": source.population_cid,
                "c2_exact_1045": exact.population_cid,
                "source_1045_split": exact.source_split_cid,
            },
        }
        model = object()
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)

            def read_record(path: Path, *, cid_field: str):
                del cid_field
                return preparation if path.name == subject.PREPARATION_RELATIVE_PATH else preflight

            with (
                patch.object(subject, "_read_json", side_effect=read_record),
                patch.object(
                    subject,
                    "zoology_control_implementation_contract",
                    return_value=implementation,
                ),
                patch.object(subject, "_load_bound_populations", return_value=(source, exact)),
                patch.object(subject, "_bind_predecessor", return_value=predecessor),
                patch.object(subject, "_configure_cpu", return_value=torch.device("cpu")),
                patch.object(
                    subject,
                    "_train_rung",
                    side_effect=[
                        (model, _passing_fit(source.population_cid)),
                        (model, _passing_fit(exact.population_cid)),
                    ],
                ),
                patch.object(
                    subject,
                    "_write_model_artifact",
                    side_effect=[
                        {"rung": "c1", "path": "c1"},
                        {"rung": "c2", "path": "c2"},
                    ],
                ),
                patch.object(subject, "permute_exact_bindings", return_value=exact.development),
                patch.object(subject, "_score_rows", return_value=_score(0.10)) as score_rows,
                patch.object(subject, "_finish_result", return_value={"done": True}) as finish,
            ):
                result = subject.run_zoology_control(root)

        self.assertEqual(result, {"done": True})
        score_rows.assert_called_once()
        self.assertEqual(score_rows.call_args.args, (model, exact.development))
        self.assertEqual(score_rows.call_args.kwargs["device"], torch.device("cpu"))
        decision = finish.call_args.kwargs["decision"]
        self.assertEqual(decision["verdict"], "STOCK_CELL_PASSES_EXACT_BYTES")


if __name__ == "__main__":
    unittest.main()
