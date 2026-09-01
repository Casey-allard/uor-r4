from __future__ import annotations

import inspect
import math
import tempfile
import unittest
from pathlib import Path
from unittest import mock

import torch

from r4_softmax_trainer import language_path_generalization_campaign as subject


def _probe_arm(
    *,
    train_seconds: float,
    evaluation_seconds: float = 0.1,
    peak_bytes: int = 100,
    memory_budget: int = 1_000,
    vector_delta: float = 0.0,
) -> dict[str, object]:
    return {
        "ok": True,
        "result": {
            "mean_train_step_seconds": train_seconds,
            "evaluation_batch_seconds": evaluation_seconds,
            "checkpoint_seconds": 0.01,
            "checkpoint_hash_seconds": 0.001,
            "artifact_export_seconds": 0.01,
            "progress_write_seconds": 0.001,
            "fixed_prefix_replay_seconds": 0.01,
            "peak_memory_bytes": peak_bytes,
            "memory_budget_bytes": memory_budget,
            "probe_vector": [vector_delta] * 128,
        },
    }


def _probe_record(
    plan: subject.ExecutionPlan,
    *,
    train_seconds: float,
    vector_delta: float = 0.0,
    peak_bytes: int = 100,
    memory_budget: int = 1_000,
) -> dict[str, object]:
    return {
        "plan": plan.identity(),
        "arms": {
            arm: _probe_arm(
                train_seconds=train_seconds,
                vector_delta=vector_delta,
                peak_bytes=peak_bytes,
                memory_budget=memory_budget,
            )
            for arm in subject.ARMS
        },
    }


def _metrics(
    *,
    initial_ce: float,
    final_ce: float,
    initial_top1: int,
    final_top1: int,
    state_off_ce: float | None = None,
    state_off_top1: int | None = None,
) -> dict[str, object]:
    value: dict[str, object] = {
        "initial_validation": {
            "rows": subject.VALIDATION_DECISIONS,
            "ce_nats": initial_ce,
            "top1_correct": initial_top1,
        },
        "final_validation": {
            "rows": subject.VALIDATION_DECISIONS,
            "ce_nats": final_ce,
            "top1_correct": final_top1,
        },
    }
    if state_off_ce is not None and state_off_top1 is not None:
        value["state_off_validation"] = {
            "rows": subject.VALIDATION_DECISIONS,
            "ce_nats": state_off_ce,
            "top1_correct": state_off_top1,
        }
    return value


class LearningRateTests(unittest.TestCase):
    def test_warmup_and_cosine_endpoints_match_freeze(self) -> None:
        self.assertEqual(subject.learning_rate(0), 0.0)
        self.assertEqual(subject.learning_rate(subject.WARMUP_STEPS), 3e-4)
        self.assertEqual(subject.learning_rate(subject.OPTIMIZER_STEPS), 3e-5)
        self.assertEqual(subject.learning_rate(50), 1.5e-4)
        self.assertGreater(subject.learning_rate(101), 3e-5)

    def test_learning_rate_rejects_steps_outside_epoch(self) -> None:
        with self.assertRaises(ValueError):
            subject.learning_rate(-1)
        with self.assertRaises(ValueError):
            subject.learning_rate(subject.OPTIMIZER_STEPS + 1)


class ExecutionSelectionTests(unittest.TestCase):
    def test_selects_lowest_projected_eligible_plan(self) -> None:
        records = [
            _probe_record(subject.ELIGIBLE_PLANS[0], train_seconds=0.40),
            _probe_record(subject.ELIGIBLE_PLANS[1], train_seconds=0.35),
            _probe_record(subject.ELIGIBLE_PLANS[2], train_seconds=0.30),
            _probe_record(subject.ELIGIBLE_PLANS[3], train_seconds=0.45),
        ]
        selected = subject.select_execution_plan(records)
        self.assertTrue(selected["available"])
        self.assertEqual(
            selected["selected_plan"]["name"],
            "cpu-accelerate-2x2t-concurrent",
        )
        projection = selected["selected_projection"]
        self.assertEqual(projection["checkpoint_interval_steps"], 100)
        self.assertEqual(projection["checkpoint_writes_per_arm"], 29)
        self.assertEqual(projection["checkpoint_hashes_per_arm"], 308)
        self.assertEqual(projection["checkpoint_sidecar_writes_per_arm"], 29)
        self.assertEqual(projection["progress_writes_per_arm"], 279)

    def test_faster_numerically_different_plan_is_ineligible(self) -> None:
        records = [
            _probe_record(subject.ELIGIBLE_PLANS[0], train_seconds=0.40),
            _probe_record(
                subject.ELIGIBLE_PLANS[1],
                train_seconds=0.10,
                vector_delta=subject.EQUIVALENCE_ABS_TOLERANCE * 2,
            ),
            _probe_record(subject.ELIGIBLE_PLANS[2], train_seconds=1.00),
            _probe_record(subject.ELIGIBLE_PLANS[3], train_seconds=1.20),
        ]
        selected = subject.select_execution_plan(records)
        self.assertEqual(
            selected["selected_plan"]["name"],
            "cpu-accelerate-4t-sequential",
        )
        eight = next(
            record
            for record in selected["plans"]
            if record["plan"]["name"] == "cpu-accelerate-8t-sequential"
        )
        self.assertEqual(eight["projection"]["reason"], "EQUIVALENCE")

    def test_memory_and_wall_fail_closed(self) -> None:
        records = [
            _probe_record(
                plan,
                train_seconds=10.0,
                peak_bytes=900,
                memory_budget=1_000,
            )
            for plan in subject.ELIGIBLE_PLANS
        ]
        selected = subject.select_execution_plan(records)
        self.assertFalse(selected["available"])
        self.assertIsNone(selected["selected_plan"])


class DecisionGateTests(unittest.TestCase):
    def setUp(self) -> None:
        self.ordinary = _metrics(
            initial_ce=7.0,
            final_ce=5.5,
            initial_top1=1_000,
            final_top1=15_000,
        )
        self.retained = _metrics(
            initial_ce=7.0,
            final_ce=5.6,
            initial_top1=1_000,
            final_top1=14_500,
            state_off_ce=5.8,
            state_off_top1=11_500,
        )

    def test_full_pass_requires_control_state_and_competitiveness(self) -> None:
        decision = subject.combine_language_path_gates(
            {"ordinary": self.ordinary, "retained": self.retained},
            mechanics_passed=True,
        )
        self.assertEqual(decision["verdict"], subject.TERMINAL_PASS)
        self.assertTrue(decision["ordinary_generalizes"])
        self.assertTrue(decision["retained_state_pass"])
        self.assertTrue(decision["competitive"])

    def test_ordinary_failure_invalidates_recipe_without_retained_verdict(self) -> None:
        ordinary = _metrics(
            initial_ce=7.0,
            final_ce=6.5,
            initial_top1=1_000,
            final_top1=3_000,
        )
        decision = subject.combine_language_path_gates(
            {"ordinary": ordinary, "retained": self.retained},
            mechanics_passed=True,
        )
        self.assertEqual(decision["verdict"], subject.TERMINAL_INVALID_RECIPE)
        self.assertEqual(decision["retained_scientific_verdict"], "NOT_EVALUATED")

    def test_generalization_without_competitiveness_has_distinct_action(self) -> None:
        retained = _metrics(
            initial_ce=7.0,
            final_ce=5.8,
            initial_top1=1_000,
            final_top1=14_000,
            state_off_ce=6.0,
            state_off_top1=11_000,
        )
        decision = subject.combine_language_path_gates(
            {"ordinary": self.ordinary, "retained": retained},
            mechanics_passed=True,
        )
        self.assertEqual(decision["verdict"], subject.TERMINAL_NOT_COMPETITIVE)

    def test_retained_state_miss_retires_only_compact_path(self) -> None:
        retained = _metrics(
            initial_ce=7.0,
            final_ce=5.6,
            initial_top1=1_000,
            final_top1=14_500,
            state_off_ce=5.65,
            state_off_top1=14_000,
        )
        decision = subject.combine_language_path_gates(
            {"ordinary": self.ordinary, "retained": retained},
            mechanics_passed=True,
        )
        self.assertEqual(decision["verdict"], subject.TERMINAL_RETAINED_FAIL)

    def test_mechanical_failure_is_not_a_model_verdict(self) -> None:
        decision = subject.combine_language_path_gates(
            {"ordinary": self.ordinary, "retained": self.retained},
            mechanics_passed=False,
        )
        self.assertEqual(decision["verdict"], subject.TERMINAL_INVALID_IMPLEMENTATION)
        self.assertEqual(decision["retained_scientific_verdict"], "NOT_EVALUATED")


class RecoveryContractTests(unittest.TestCase):
    def test_checkpoint_round_trip_binds_plan_and_step(self) -> None:
        model = torch.nn.Linear(2, 2, bias=False)
        optimizer = torch.optim.AdamW(model.parameters(), lr=subject.learning_rate(0))
        initial = {
            "rows": subject.VALIDATION_DECISIONS,
            "ce_nats": 7.0,
            "top1_correct": 0,
        }
        with tempfile.TemporaryDirectory() as temporary:
            path = Path(temporary) / "checkpoint.pt"
            subject._save_checkpoint(
                path,
                arm="ordinary",
                model=model,
                optimizer=optimizer,
                step=0,
                elapsed_arm_seconds=1.0,
                initial_validation=initial,
                run_contract_cid="blake3:" + "1" * 64,
                plan_cid="blake3:" + "2" * 64,
                last_loss=None,
            )
            loaded = subject._load_checkpoint(
                path,
                arm="ordinary",
                model=model,
                optimizer=optimizer,
                device=torch.device("cpu"),
                run_contract_cid="blake3:" + "1" * 64,
                plan_cid="blake3:" + "2" * 64,
            )
        self.assertEqual(loaded["step"], 0)
        self.assertEqual(loaded["initial_validation"], initial)

    def test_checkpoint_cid_is_verified_before_deserialization(self) -> None:
        model = torch.nn.Linear(2, 2, bias=False)
        optimizer = torch.optim.AdamW(model.parameters(), lr=subject.learning_rate(0))
        with tempfile.TemporaryDirectory() as temporary:
            path = Path(temporary) / "checkpoint.pt"
            subject._save_checkpoint(
                path,
                arm="ordinary",
                model=model,
                optimizer=optimizer,
                step=0,
                elapsed_arm_seconds=0.0,
                initial_validation={"rows": subject.VALIDATION_DECISIONS},
                run_contract_cid="blake3:" + "1" * 64,
                plan_cid="blake3:" + "2" * 64,
                last_loss=None,
            )
            with path.open("ab") as target:
                target.write(b"tamper")
            with mock.patch.object(torch, "load") as deserialize:
                with self.assertRaisesRegex(ValueError, "checkpoint CID"):
                    subject._load_checkpoint(
                        path,
                        arm="ordinary",
                        model=model,
                        optimizer=optimizer,
                        device=torch.device("cpu"),
                        run_contract_cid="blake3:" + "1" * 64,
                        plan_cid="blake3:" + "2" * 64,
                    )
                deserialize.assert_not_called()

    def test_implementation_binding_rejects_drift(self) -> None:
        implementation = {"files": [], "tree_cid": "blake3:" + "1" * 64}
        self.assertEqual(
            subject._require_current_implementation(
                implementation,
                label="test freeze",
                current=implementation,
            ),
            implementation,
        )
        with self.assertRaisesRegex(ValueError, "differs from test freeze"):
            subject._require_current_implementation(
                implementation,
                label="test freeze",
                current={"files": [], "tree_cid": "blake3:" + "2" * 64},
            )

    def test_resume_wall_baseline_uses_newer_durable_progress(self) -> None:
        model = torch.nn.Linear(2, 2, bias=False)
        optimizer = torch.optim.AdamW(model.parameters(), lr=subject.learning_rate(0))
        run_contract_cid = "blake3:" + "1" * 64
        plan_cid = "blake3:" + "2" * 64
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            path = root / "arms" / "ordinary" / "checkpoint.pt"
            subject._save_checkpoint(
                path,
                arm="ordinary",
                model=model,
                optimizer=optimizer,
                step=0,
                elapsed_arm_seconds=1.0,
                initial_validation={"rows": subject.VALIDATION_DECISIONS},
                run_contract_cid=run_contract_cid,
                plan_cid=plan_cid,
                last_loss=None,
            )
            progress = subject._write_progress(
                root,
                arm="ordinary",
                step=10,
                elapsed_arm_seconds=5.0,
                last_loss=7.0,
                checkpoint=path,
                status="RUNNING",
            )
            self.assertEqual(progress["checkpoint"]["completed_step"], 0)
            loaded = subject._load_checkpoint(
                path,
                arm="ordinary",
                model=model,
                optimizer=optimizer,
                device=torch.device("cpu"),
                run_contract_cid=run_contract_cid,
                plan_cid=plan_cid,
            )
            elapsed = subject._resume_elapsed_baseline(
                root,
                arm="ordinary",
                checkpoint=loaded,
                checkpoint_path=path,
                run_contract_cid=run_contract_cid,
                plan_cid=plan_cid,
            )
        self.assertEqual(elapsed, 5.0)

    def test_wall_accounting_does_not_subtract_resume_time_twice(self) -> None:
        ceiling = subject._arm_wall_ceiling(
            concurrent=False, completed_other_arm_seconds=1_000.0
        )
        self.assertEqual(ceiling, 6_200.0)
        self.assertFalse(
            subject._wall_exhausted(
                elapsed_before_seconds=2_000.0,
                elapsed_current_seconds=4_199.0,
                arm_ceiling_seconds=ceiling,
            )
        )
        self.assertTrue(
            subject._wall_exhausted(
                elapsed_before_seconds=2_000.0,
                elapsed_current_seconds=4_200.0,
                arm_ceiling_seconds=ceiling,
            )
        )

    def test_public_campaign_apis_have_bounded_signatures(self) -> None:
        probe = inspect.signature(subject.probe_language_path_execution)
        run = inspect.signature(subject.run_language_path_generalization)
        self.assertEqual(tuple(probe.parameters), ("root",))
        self.assertEqual(tuple(run.parameters), ("root", "resume"))
        self.assertFalse(run.parameters["resume"].default)

    def test_frozen_arithmetic(self) -> None:
        self.assertEqual(subject.OPTIMIZER_STEPS, 2_730)
        self.assertEqual(subject.TRAIN_DECISIONS, 5_241_600)
        self.assertEqual(subject.VALIDATION_DECISIONS, 247_920)
        self.assertEqual(subject.REACHABLE_VALIDATION_DECISIONS, 245_854)
        self.assertTrue(
            math.isclose(
                subject.TRAIN_DECISIONS / subject.PARAMETER_COUNT,
                20.7868020307,
                rel_tol=1e-9,
            )
        )


if __name__ == "__main__":
    unittest.main()
