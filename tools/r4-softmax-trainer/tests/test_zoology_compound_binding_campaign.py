"""Focused qualification, structured attention, counterfactual and access checks."""

import copy
import unittest
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import patch

import torch

from r4_softmax_trainer.zoology_compound_binding import campaign


def views():
    return [
        {
            "rotation": offset,
            "construction": {"decisions": 8192, "top1_correct": 8111},
            "unknown": {"decisions": 2048, "top1_correct": 1946},
            "question_pair_world_counts": {
                kind: {"worlds": 1024, "complete_quartets": 973}
                for kind in ("owner_changing", "object_changing")
            },
        }
        for offset in range(4)
    ]


class CompoundCampaignTests(unittest.TestCase):
    def test_known_unknown_and_each_family_quartet_qualify_each_order(self):
        original = views()
        self.assertTrue(campaign._qualification(original)["passed"])
        for path in (
            ("construction", "top1_correct"),
            ("unknown", "top1_correct"),
            ("question_pair_world_counts", "owner_changing", "complete_quartets"),
            ("question_pair_world_counts", "object_changing", "complete_quartets"),
        ):
            modified = copy.deepcopy(original)
            target = modified[3]
            for key in path[:-1]:
                target = target[key]
            target[path[-1]] -= 1
            self.assertFalse(campaign._qualification(modified)["passed"])
        original[2]["unknown"]["decisions"] = 2047
        with self.assertRaisesRegex(ValueError, "unknown population"):
            campaign._qualification(original)

    def test_order_gate_checks_full_head_and_predictions_with_aligned_fact_attention(
        self,
    ):
        predicted = torch.zeros((3, 1), dtype=torch.long)
        logits = torch.zeros((3, 1, 4096))
        attention = (
            torch.tensor([0.1, 0.2, 0.3, 0.15, 0.25])
            .reshape(1, 1, 1, 5)
            .repeat(3, 1, 1, 1)
        )
        rotated = torch.cat(
            (torch.roll(attention[..., :4], 1, -1), attention[..., 4:]), -1
        )
        baseline = predicted, logits, attention
        result = campaign._order_comparison(predicted, logits, rotated, baseline, 1)
        self.assertTrue(result["passed"])
        self.assertEqual(result["max_aligned_attention_difference"], 0)
        moved = logits.clone()
        moved[2, 0, 4095] = 0.0002
        self.assertFalse(
            campaign._order_comparison(predicted, moved, rotated, baseline, 1)["passed"]
        )
        changed = predicted.clone()
        changed[2, 0] = 1
        self.assertFalse(
            campaign._order_comparison(changed, logits, rotated, baseline, 1)["passed"]
        )

    def test_replacement_labels_use_inverse_right_cycle_and_preserve_absence(self):
        inputs = torch.zeros((5, 41), dtype=torch.long)
        inputs[:, [7, 15, 23, 31]] = torch.tensor([20, 21, 22, 23])
        targets = torch.tensor([[20], [21], [22], [23], [11]])
        tensors = {"inputs": inputs, "targets": targets, "variant_ids": torch.arange(5)}
        observed = campaign._replacement_targets(tensors)
        self.assertEqual(observed.reshape(-1).tolist(), [23, 20, 21, 22, 11])
        self.assertTrue(torch.equal(tensors["targets"], targets))
        inputs[0, 15] = 20
        with self.assertRaisesRegex(ValueError, "exactly one"):
            campaign._replacement_targets(tensors)

    def test_control_requires_replacement_recovery_drop_unknown_and_attention(self):
        primary = views()[0]
        controlled = {
            "supported": {"top1_correct": 4015},
            "unknown": {"top1_correct": 1946},
        }
        self.assertTrue(
            campaign._control_decision(primary, controlled, 7783, True)["passed"]
        )
        for counts, recovered, equal in (
            (
                {
                    "supported": {"top1_correct": 4016},
                    "unknown": {"top1_correct": 1946},
                },
                7783,
                True,
            ),
            (controlled, 7782, True),
            (
                {
                    "supported": {"top1_correct": 4015},
                    "unknown": {"top1_correct": 1945},
                },
                7783,
                True,
            ),
            (controlled, 7783, False),
        ):
            self.assertFalse(
                campaign._control_decision(primary, counts, recovered, equal)["passed"]
            )

    def test_no_control_or_development_before_its_qualification(self):
        for q, o, expected in (
            (False, True, "NOT_RUN_CONSTRUCTION_MISS"),
            (True, False, "NOT_RUN_ORDER_MISS"),
        ):
            with patch.object(
                campaign, "_score", side_effect=AssertionError("forbidden scoring")
            ) as scorer:
                result = campaign._conditional_controls(
                    object(), {}, views(), {"passed": q}, {"passed": o}, object()
                )
            self.assertEqual(result["status"], expected)
            scorer.assert_not_called()
        for q, o, c, expected in (
            (False, True, True, "NOT_RUN_CONSTRUCTION_MISS"),
            (True, False, True, "NOT_RUN_ORDER_MISS"),
            (True, True, False, "NOT_RUN_CONTROL_MISS"),
        ):
            with patch.object(
                campaign.data,
                "load_development",
                side_effect=AssertionError("forbidden development"),
            ) as loader:
                result = campaign._conditional_development(
                    Path("unused"),
                    object(),
                    views(),
                    {"passed": q},
                    {"passed": o},
                    {"passed": c},
                    object(),
                )
            loader.assert_not_called()
            self.assertEqual(result["status"], expected)
            self.assertEqual(result["model_decisions"], 0)

    def test_decision_distinguishes_partial_control_and_fresh_outcomes(self):
        unscored = {"model_decisions": 0, "views": []}
        positive = {"passed": True, "any_regression": False}
        partial = campaign._decision(
            positive, {"passed": False}, {"passed": True}, {"passed": False}, unscored
        )
        self.assertEqual(partial["status"], "COMPOUND_BINDING_PARTIAL_GAIN")
        negative = campaign._decision(
            {"passed": False, "any_regression": True},
            {"passed": False},
            {"passed": True},
            {"passed": False},
            unscored,
        )
        self.assertNotEqual(partial["next_action"], negative["next_action"])
        control = campaign._decision(
            positive, {"passed": True}, {"passed": True}, {"passed": False}, unscored
        )
        self.assertEqual(control["status"], "COMPOUND_BINDING_CONTROL_MISS")
        development = {
            "model_decisions": 5120,
            "views": [{"rotation": i, "decision": {"passed": True}} for i in range(4)],
        }
        self.assertTrue(
            campaign._decision(
                positive,
                {"passed": True},
                {"passed": True},
                {"passed": True},
                development,
            )["passed"]
        )
        development["views"][3]["decision"]["passed"] = False
        self.assertEqual(
            campaign._decision(
                positive,
                {"passed": True},
                {"passed": True},
                {"passed": True},
                development,
            )["status"],
            "COMPOUND_BINDING_FRESH_TRANSFER_MISS",
        )
        with self.assertRaisesRegex(ValueError, "unscored"):
            campaign._decision(
                positive,
                {"passed": False},
                {"passed": True},
                {"passed": False},
                development,
            )

    def test_scorer_uses_actual_rectangular_attention_and_never_passes_labels(self):
        class Model:
            def forward_selected(self, inputs, positions, *, return_attention, control):
                return SimpleNamespace(
                    logits=torch.zeros((len(inputs), 1, 4096)),
                    attention_weights=(torch.full((len(inputs), 1, 1, 5), 0.2),),
                )

        tensors = {
            "inputs": torch.zeros((5, 41), dtype=torch.long),
            "positions": torch.full((5, 1), 37, dtype=torch.long),
            "targets": torch.tensor([[1], [2], [3], [4], [11]]),
            "variant_ids": torch.arange(5),
        }
        record, predicted, logits, attention = campaign._score(
            Model(), tensors, SimpleNamespace(check=lambda: None)
        )
        self.assertEqual(record["all"]["attention_shape"], [5, 1, 1, 5])
        self.assertEqual(record["supported"]["decisions"], 4)
        self.assertEqual(record["unknown"]["decisions"], 1)
        changed = {**tensors, "targets": torch.tensor([[7], [8], [9], [10], [11]])}
        second, other, new_logits, new_attention = campaign._score(
            Model(), changed, SimpleNamespace(check=lambda: None)
        )
        self.assertTrue(torch.equal(predicted, other))
        self.assertTrue(torch.equal(logits, new_logits))
        self.assertTrue(torch.equal(attention, new_attention))
        self.assertEqual(
            second["all"]["selected_logits_cid"], record["all"]["selected_logits_cid"]
        )


if __name__ == "__main__":
    unittest.main()
