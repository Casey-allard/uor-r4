"""Named checks for question ignoring, abstention masking and shared budgets."""

from __future__ import annotations

import copy
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

import torch

from r4_softmax_trainer.zoology_english_binding import campaign, contract, data


class EnglishBindingDecisionTests(unittest.TestCase):
    def setUp(self) -> None:
        self.targets = (
            torch.tensor([44, 45, 45, 44, data.UNKNOWN_ID], dtype=torch.long)
            .repeat(256)
            .reshape(-1, 1)
        )
        self.types = (torch.arange(256) % 2).repeat_interleave(5)
        self.construction = {"decisions": 8192, "top1_correct": 8192}

    def test_answering_unknown_cannot_mask_failed_supported_binding(self) -> None:
        predictions = torch.full_like(self.targets, data.UNKNOWN_ID)
        behavior = campaign._binding_metrics(predictions, self.targets, self.types)
        self.assertEqual(behavior["unknown_correct"], 256)
        self.assertEqual(behavior["known_correct"], 0)
        result = campaign._language_decision(self.construction, behavior)
        self.assertFalse(result["passed"])
        self.assertEqual(
            result["status"], "ENGLISH_BINDING_COMPOSITIONAL_TRANSFER_MISS"
        )

    def test_following_history_while_ignoring_question_fails_complete_groups(
        self,
    ) -> None:
        predictions = self.targets.clone().reshape(256, 5)
        predictions[:, 1] = predictions[:, 0]
        predictions[:, 3] = predictions[:, 2]
        behavior = campaign._binding_metrics(
            predictions.reshape(-1, 1), self.targets, self.types
        )
        self.assertEqual(behavior["same_question_history_changes"], 512)
        self.assertEqual(behavior["same_history_question_changes"], 0)
        self.assertEqual(behavior["complete_groups_correct"], 0)
        self.assertFalse(
            campaign._language_decision(self.construction, behavior)["passed"]
        )
        perfect = campaign._binding_metrics(self.targets, self.targets, self.types)
        self.assertTrue(
            campaign._language_decision(self.construction, perfect)["passed"]
        )

    def test_destructive_control_requires_matching_work_and_no_future_reads(
        self,
    ) -> None:
        totals = {name: 1 for name in campaign._WORK_FIELDS}
        totals["admitted_attention_pairs"] = 1280 * 41 * 42
        totals["future_position_reads"] = 0
        r4 = {"audit_totals": totals, "future_attention_nonzero": 0}
        control = copy.deepcopy(r4)
        self.assertTrue(
            campaign._control_decision(r4, control, 0.8)["strong_transport_sensitivity"]
        )
        control["audit_totals"]["key_blocks_transported"] += 1
        self.assertFalse(
            campaign._control_decision(r4, control, 0.8)["strong_transport_sensitivity"]
        )
        control = copy.deepcopy(r4)
        control["audit_totals"]["future_position_reads"] = 1
        self.assertFalse(
            campaign._control_decision(r4, control, 0.8)["integrity_passed"]
        )

    def test_replay_budget_carries_time_spent_fitting(self) -> None:
        with patch.object(campaign.time, "monotonic", side_effect=(10.0, 12.0)):
            budget = campaign._Budget(contract.TRAINING["max_elapsed_seconds"] - 1)
            with self.assertRaises(campaign.ResourceBudgetExceeded):
                budget.check()
        with self.assertRaises(ValueError):
            campaign._Budget(float("nan"))

    def test_final_artifact_change_is_detected_after_initial_load(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            (root / "fit").mkdir()
            artifact = root / "fit/model.safetensors"
            artifact.write_bytes(b"unchanged final artifact")
            record = contract.prior._record(root, "fit/model.safetensors")
            preparation = {"preparation_cid": "test-preparation"}
            campaign._write_exclusive(
                root / "fit/fit.json",
                {
                    "preparation_cid": preparation["preparation_cid"],
                    "training": contract.TRAINING,
                    "status": "FIT_COMPLETE",
                    "completed_updates": contract.TRAINING["total_updates"],
                    "artifact": {**record, "config": contract.MODEL_CONFIG},
                },
                "fit_cid",
            )
            campaign._fit_record(root, preparation)
            artifact.write_bytes(b"changed during evaluation")
            with self.assertRaisesRegex(ValueError, "model bytes or path changed"):
                campaign._fit_record(root, preparation)


if __name__ == "__main__":
    unittest.main()
