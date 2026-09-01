"""Focused synthetic contracts for the bounded paired-H4 capacity campaign."""

from __future__ import annotations

import tempfile
import unittest
from pathlib import Path
from types import SimpleNamespace
from unittest import mock

import torch

from r4_softmax_trainer import paired_h4_prompt_capacity_campaign as subject
from r4_softmax_trainer.paired_h4_language_path import (
    CANONICAL_IDENTITY_INDEX,
    canonical_layer_token_leaves,
    joint_prefix_collision_census,
)


def _metric(*, ce: float, top1_rate: float) -> dict[str, object]:
    return {
        "rows": subject.FRESH_HELDOUT_DECISIONS,
        "ce_nats": ce,
        "top1_correct": round(top1_rate * subject.FRESH_HELDOUT_DECISIONS),
        "top1_rate": top1_rate,
        "forbidden_reads": 0,
    }


class FrozenContractTests(unittest.TestCase):
    def test_population_training_and_fresh_heldout_arithmetic(self) -> None:
        self.assertEqual(
            subject.EXPECTED_PROMPT_POPULATION_CID,
            "blake3:c11a7c935139ca169460b90c01392d7c9e0929e4c10710e76e6c8f74cbdf0340",
        )
        self.assertEqual(subject.EXPECTED_PROMPT_LAST_SOURCE_STORY_ORDINAL, 153_977)
        self.assertEqual(subject.EXPECTED_PROMPT_ELIGIBLE_STORIES, 4_200)
        self.assertEqual(subject.OPTIMIZER_STEPS, 2_730)
        self.assertEqual(subject.TRAIN_WINDOWS, 43_680)
        self.assertEqual(subject.TRAIN_DECISIONS, 5_241_600)
        self.assertEqual(subject.FRESH_HELDOUT_WINDOWS, 2_066)
        self.assertEqual(subject.FRESH_HELDOUT_TOKENS, 2_066 * 121)
        self.assertEqual(subject.FRESH_HELDOUT_DECISIONS, 247_920)
        self.assertEqual(subject.CPU_PLAN.backend, "cpu")
        self.assertEqual(subject.CPU_PLAN.threads_per_worker, 4)
        self.assertEqual(subject.CPU_PLAN.workers, 1)
        self.assertFalse(subject.CPU_PLAN.concurrent_arms)

    def test_projection_admits_only_bounded_time_and_memory(self) -> None:
        admitted = subject.project_candidate_execution(
            mean_train_step_seconds=0.4,
            evaluation_batch_seconds=0.1,
            checkpoint_seconds=0.01,
            artifact_seconds=0.1,
            replay_seconds=0.1,
            peak_memory_bytes=100,
            memory_budget_bytes=1_000,
        )
        self.assertTrue(admitted["eligible"])
        self.assertEqual(admitted["reason"], "ELIGIBLE")
        self.assertEqual(admitted["safety_factor"], 1.25)
        self.assertEqual(admitted["projection_ceiling_seconds"], 3_000.0)
        self.assertEqual(admitted["hard_wall_ceiling_seconds"], 3_600.0)

        wall = subject.project_candidate_execution(
            mean_train_step_seconds=1.0,
            evaluation_batch_seconds=0.1,
            checkpoint_seconds=0.01,
            artifact_seconds=0.1,
            replay_seconds=0.1,
            peak_memory_bytes=100,
            memory_budget_bytes=1_000,
        )
        self.assertFalse(wall["eligible"])
        self.assertEqual(wall["reason"], "WALL_PROJECTION")

        memory = subject.project_candidate_execution(
            mean_train_step_seconds=0.4,
            evaluation_batch_seconds=0.1,
            checkpoint_seconds=0.01,
            artifact_seconds=0.1,
            replay_seconds=0.1,
            peak_memory_bytes=801,
            memory_budget_bytes=1_000,
        )
        self.assertFalse(memory["eligible"])
        self.assertEqual(memory["reason"], "MEMORY")


class CollisionInstrumentTests(unittest.TestCase):
    def test_compact_address_update_matches_full_permutation_census(self) -> None:
        elements = torch.arange(120, dtype=torch.long)
        actions = (elements[:, None] + elements[None, :] + 1) % 120
        tokens = torch.tensor([[1, 119, 1, 119], [2, 118, 2, 118]], dtype=torch.long)
        leaves = canonical_layer_token_leaves()
        compact = subject._route_repeats(
            tokens,
            layer_token_leaves=leaves,
            left_actions=actions,
            identity_index=CANONICAL_IDENTITY_INDEX,
        )
        full = joint_prefix_collision_census(
            tokens,
            layer_token_leaves=leaves,
            left_actions=actions,
            identity_index=CANONICAL_IDENTITY_INDEX,
        )
        self.assertEqual(compact, full.repeats_per_sequence)


class ScientificDecisionTests(unittest.TestCase):
    def test_fresh_generalization_passes_learning_and_predecessor_nonregression(
        self,
    ) -> None:
        decision = subject.fresh_generalization_gates(
            candidate_initial=_metric(ce=8.3, top1_rate=0.001),
            candidate_final=_metric(ce=3.9, top1_rate=0.30),
            predecessor=_metric(ce=3.88, top1_rate=0.305),
        )
        self.assertTrue(decision["passed"])
        self.assertTrue(all(decision["gates"].values()))

    def test_fresh_generalization_rejects_a_prompt_only_regression(self) -> None:
        decision = subject.fresh_generalization_gates(
            candidate_initial=_metric(ce=8.3, top1_rate=0.001),
            candidate_final=_metric(ce=4.2, top1_rate=0.25),
            predecessor=_metric(ce=3.8, top1_rate=0.30),
        )
        self.assertFalse(decision["passed"])
        self.assertFalse(decision["gates"]["candidate_final_nll_ceiling"])
        self.assertFalse(decision["gates"]["predecessor_nll_nonregression"])

    def test_terminal_outcomes_have_distinct_actions(self) -> None:
        cases = {
            subject.VERDICT_PASS: subject.TERMINAL_PASS,
            subject.VERDICT_ABSOLUTE_NO_CAPACITY_GAIN: (
                subject.TERMINAL_ABSOLUTE_NO_CAPACITY_GAIN
            ),
            subject.VERDICT_PARTIAL: subject.TERMINAL_PARTIAL,
            subject.VERDICT_FAIL: subject.TERMINAL_FAIL,
            subject.VERDICT_INVALID: subject.TERMINAL_INVALID,
        }
        actions = set()
        for prompt, expected in cases.items():
            result = subject.combine_terminal_verdict(
                prompt_verdict=prompt,
                language_passed=True,
                mechanics_passed=True,
            )
            self.assertEqual(result["verdict"], expected)
            actions.add(result["action"])
        self.assertEqual(len(actions), len(cases))

        regression = subject.combine_terminal_verdict(
            prompt_verdict=subject.VERDICT_PASS,
            language_passed=False,
            mechanics_passed=True,
        )
        self.assertEqual(
            regression["verdict"], subject.TERMINAL_GENERAL_LANGUAGE_REGRESSION
        )


class RecoveryAndProbeTests(unittest.TestCase):
    def test_checkpoint_round_trip_and_tamper_rejection(self) -> None:
        model = torch.nn.Linear(2, 2, bias=False)
        optimizer = torch.optim.AdamW(model.parameters(), lr=0.0)
        initial = _metric(ce=8.3, top1_rate=0.0)
        contract_cid = "blake3:" + "1" * 64
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            subject._save_checkpoint(
                root,
                model=model,
                optimizer=optimizer,
                step=0,
                elapsed_seconds=1.0,
                initial_heldout=initial,
                run_contract_cid=contract_cid,
                last_loss=None,
            )
            checkpoint = subject._load_checkpoint(
                root,
                model=model,
                optimizer=optimizer,
                device=torch.device("cpu"),
                run_contract_cid=contract_cid,
            )
            self.assertEqual(checkpoint["step"], 0)
            with (root / subject.CHECKPOINT_RELATIVE_PATH).open("ab") as target:
                target.write(b"tamper")
            with mock.patch.object(subject.torch, "load") as deserialize:
                with self.assertRaisesRegex(ValueError, "checkpoint CID"):
                    subject._load_checkpoint(
                        root,
                        model=model,
                        optimizer=optimizer,
                        device=torch.device("cpu"),
                        run_contract_cid=contract_cid,
                    )
                deserialize.assert_not_called()

    def test_probe_is_one_candidate_cpu4_five_step_gate(self) -> None:
        execution = {
            "probe_steps": subject.PROBE_STEPS,
            "mechanics": {"passed": True},
            "projection": {"eligible": True},
        }
        implementation = {"files": [], "tree_cid": "blake3:" + "2" * 64}
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            preparation = SimpleNamespace(
                root=root,
                manifest={
                    "preparation_cid": "blake3:" + "3" * 64,
                    "implementation": implementation,
                },
                commitment={"commitment_cid": "blake3:" + "4" * 64},
            )
            with (
                mock.patch.object(
                    subject,
                    "load_paired_h4_prompt_capacity_preparation",
                    return_value=preparation,
                ),
                mock.patch.object(
                    subject,
                    "trainer_implementation_contract",
                    return_value=implementation,
                ),
            ):
                result = subject.probe_paired_h4_prompt_capacity(
                    root,
                    _executor=lambda prepared: execution,
                )
        self.assertTrue(result["eligible"])
        self.assertEqual(result["plan"], subject.CPU_PLAN.identity())
        self.assertEqual(result["execution"]["probe_steps"], 5)
        self.assertEqual(result["cuda"], "FORBIDDEN")
        self.assertEqual(result["mps"], "NOT_USED")


if __name__ == "__main__":
    unittest.main()
