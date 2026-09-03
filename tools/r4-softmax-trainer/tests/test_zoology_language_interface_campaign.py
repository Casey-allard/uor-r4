"""Decision-bearing checks for the bounded learned language interface."""

from __future__ import annotations

import copy
import json
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

import torch

from r4_softmax_trainer.provenance import canonical_json_bytes, cid_bytes
from r4_softmax_trainer.zoology_language_interface import campaign


class LanguageCampaignTests(unittest.TestCase):
    def test_same_bag_pairs_require_both_changed_answers(self):
        inputs = torch.ones((5, 5, 4), dtype=torch.long)
        inputs[0, 4] = torch.tensor([2, 3, 4, 5])
        inputs[1, 4] = torch.tensor([4, 3, 2, 5])
        inputs[2:4, 4] = inputs[:2, 4]
        targets = torch.tensor([[10], [11], [11], [10], [12]])
        tensors = {
            "inputs": inputs,
            "lengths": torch.full((5, 5), 4),
            "targets": targets,
            "pair_types": torch.ones(5, dtype=torch.long),
        }
        exact = campaign._syntax_pairs(tensors, targets)
        self.assertEqual(exact["both_answers_correct"], 2)
        constant = campaign._syntax_pairs(tensors, torch.full_like(targets, 10))
        self.assertEqual(constant["both_answers_correct"], 0)
        tensors["inputs"][1, 0, 0] = 9
        with self.assertRaisesRegex(ValueError, "identical facts"):
            campaign._syntax_pairs(tensors, targets)

    def test_high_aggregate_cannot_hide_owner_or_pair_failure(self):
        row = {
            "records": {"supported": {"top1_rate": 1.0}, "unknown": {"top1_rate": 1.0}},
            "role_accuracy": {
                "rate": 1.0,
                "by_role": {r: {"rate": 1.0} for r in ("owner", "object", "location")},
            },
            "syntax_pairs": {"complete_rate": 1.0},
            "work": {"rows": 10, "role_decisions": 140, "binding_score_slots": 50},
            "groups": {
                "by_question_type": {
                    r: {"complete_supported_quartets": 10, "groups": 10}
                    for r in ("same_owner", "same_object")
                }
            },
        }
        self.assertTrue(campaign._qualified(row)["passed"])
        bad = copy.deepcopy(row)
        bad["role_accuracy"]["by_role"]["owner"]["rate"] = 0.98
        self.assertFalse(campaign._qualified(bad)["passed"])
        bad = copy.deepcopy(row)
        bad["syntax_pairs"]["complete_rate"] = 0.94
        self.assertFalse(campaign._qualified(bad)["passed"])
        self.assertEqual(
            campaign._decision(False, False, None)["status"],
            "LANGUAGE_INTERFACE_CONSTRUCTION_MISS",
        )
        self.assertEqual(
            campaign._decision(True, False, None)["status"],
            "LANGUAGE_INTERFACE_CONTROL_MISS",
        )
        self.assertEqual(
            campaign._decision(True, True, False)["status"],
            "LANGUAGE_INTERFACE_HELDOUT_MISS",
        )

    def test_replay_budget_carries_prior_phases(self):
        with patch.object(campaign.time, "monotonic", side_effect=[100.0, 106.0]):
            budget = campaign._Budget(895.0)
            with self.assertRaises(campaign.ResourceBudgetExceeded):
                budget.check()

    def test_partial_fit_is_rejected_before_reader_load(self):
        fitted = {
            "status": "FIT_COMPLETE",
            "preparation_cid": "prep",
            "implementation_cid": "impl",
            "optimizer_updates": 511,
        }
        fitted["fit_cid"] = cid_bytes(canonical_json_bytes(fitted))
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            (root / "fit.json").write_text(json.dumps(fitted))
            with (
                patch.object(
                    campaign,
                    "load_safetensors",
                    side_effect=AssertionError("must not load partial artifact"),
                ),
                self.assertRaisesRegex(ValueError, "complete frozen dose"),
            ):
                campaign._load_fit(
                    root,
                    {
                        "preparation_cid": "prep",
                        "implementation": {"tree_cid": "impl"},
                    },
                )


if __name__ == "__main__":
    unittest.main()
