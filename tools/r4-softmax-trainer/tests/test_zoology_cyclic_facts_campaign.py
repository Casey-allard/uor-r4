"""Decision-bearing matched-order, reveal, reproduction and dependence checks."""

import copy
import unittest
from pathlib import Path
from unittest.mock import patch

import torch

from r4_softmax_trainer.zoology_cyclic_facts import campaign


def views(owner=47, obj=447, correct=3735, slot=1024, family=1800):
    return [
        {
            "rotation": offset,
            "construction": {"decisions": 8192, "top1_correct": correct},
            "diagnostic": {
                "paired": {
                    "question": {
                        "pair_type": {
                            "same_object": {"pairs": 2048, "both_correct": owner},
                            "same_owner": {"pairs": 2048, "both_correct": obj},
                        }
                    }
                },
                "strata": {
                    "pair_type": {
                        name: {"rows": 4096, "correct": family}
                        for name in ("same_owner", "same_object")
                    },
                    "target_displayed_slot": {
                        str(i): {"rows": 2048, "correct": slot} for i in range(4)
                    },
                },
            },
        }
        for offset in range(4)
    ]


class CyclicFactsDecisionTests(unittest.TestCase):
    def test_matched_gain_caps_and_each_rotation_slot_and_family(self):
        reference = views()
        candidate = views(252, 652)
        self.assertTrue(campaign._behavior(candidate, reference)["passed"])
        for field in ("same_object", "same_owner"):
            changed = copy.deepcopy(candidate)
            changed[2]["diagnostic"]["paired"]["question"]["pair_type"][field][
                "both_correct"
            ] -= 1
            self.assertFalse(campaign._behavior(changed, reference)["passed"])
        changed = copy.deepcopy(candidate)
        changed[3]["diagnostic"]["strata"]["target_displayed_slot"]["0"]["correct"] = (
            1023
        )
        self.assertFalse(campaign._behavior(changed, reference)["passed"])
        changed = copy.deepcopy(candidate)
        changed[1]["diagnostic"]["strata"]["pair_type"]["same_owner"]["correct"] -= 1
        outcome = campaign._behavior(changed, reference)
        self.assertTrue(outcome["any_regression"])
        self.assertFalse(outcome["passed"])
        capped = campaign._behavior(views(2048, 2048), views(2000, 2000))
        self.assertTrue(capped["passed"])
        self.assertTrue(capped["views"][0]["pairs"]["same_owner"]["ceiling_limited"])
        self.assertEqual(
            capped["views"][0]["pairs"]["same_owner"]["required_both_correct"], 2048
        )
        with self.assertRaisesRegex(ValueError, "four ordered"):
            campaign._behavior(candidate + candidate[:1], reference)

    def test_both_behavior_and_every_construction_gate_control_reveal(self):
        for candidate, reference, expected in (
            (views(252, 652, 8110), views(), "NOT_RUN_CONSTRUCTION_MISS"),
            (
                views(1967, 1967, 8111, 1967, 4000),
                views(2000, 2000, 8100, 2000, 4000),
                "NOT_RUN_BEHAVIOR_MISS",
            ),
        ):
            behavior = campaign._behavior(candidate, reference)
            with (
                patch.object(
                    campaign.data,
                    "load_development",
                    side_effect=AssertionError("forbidden"),
                ) as loader,
                patch.object(
                    campaign, "_plain_score", side_effect=AssertionError("forbidden")
                ) as scorer,
            ):
                development = campaign._conditional_development(
                    Path("unused"), object(), candidate, behavior, object()
                )
            loader.assert_not_called()
            scorer.assert_not_called()
            self.assertEqual(development["status"], expected)
            self.assertEqual(development["model_decisions"], 0)
            self.assertFalse(
                campaign._decision(candidate, behavior, development)["passed"]
            )
        candidate = views(1967, 1967, 8111, 1967, 4000)
        self.assertTrue(campaign._construction_fits(candidate))
        candidate[3]["construction"]["top1_correct"] = 8110
        self.assertFalse(campaign._construction_fits(candidate))

    def test_partial_gain_and_regression_have_divergent_actions(self):
        candidate, reference = views(252, 652), views()
        unscored = {"model_decisions": 0, "views": []}
        positive = campaign._decision(
            candidate, campaign._behavior(candidate, reference), unscored
        )
        self.assertEqual(positive["status"], "CYCLIC_FACTS_PARTIAL_GAIN")
        candidate[1]["construction"]["top1_correct"] -= 1
        negative = campaign._decision(
            candidate, campaign._behavior(candidate, reference), unscored
        )
        self.assertEqual(negative["status"], "CYCLIC_FACTS_PRESERVATION_MISS")
        self.assertNotEqual(positive["next_action"], negative["next_action"])
        with self.assertRaisesRegex(ValueError, "unscored"):
            campaign._decision(
                candidate,
                campaign._behavior(candidate, reference),
                {"model_decisions": 1, "views": []},
            )

    def test_all_four_development_views_use_unchanged_family_thresholds(self):
        candidate, reference = views(1967, 1967, 8111, 1967, 4000), views()
        metrics = {
            "known_decisions": 1024,
            "known_correct": 973,
            "unknown_decisions": 256,
            "unknown_correct": 244,
            "complete_groups_correct": 232,
            "by_question_type": {
                name: {"complete_correct": 116}
                for name in ("same_owner", "same_object")
            },
        }
        dev = {
            "model_decisions": 5120,
            "views": [
                {
                    "rotation": i,
                    "decision": campaign._language_decision(
                        candidate[i]["construction"], metrics
                    ),
                }
                for i in range(4)
            ],
        }
        behavior = campaign._behavior(candidate, reference)
        self.assertEqual(
            campaign._decision(candidate, behavior, dev)["status"],
            "CYCLIC_FACTS_FRESH_BINDING_PASSED",
        )
        metrics["by_question_type"]["same_object"]["complete_correct"] = 115
        dev["views"][3]["decision"] = campaign._language_decision(
            candidate[3]["construction"], metrics
        )
        self.assertEqual(
            campaign._decision(candidate, behavior, dev)["status"],
            "CYCLIC_FACTS_FRESH_TRANSFER_MISS",
        )
        dev["model_decisions"] = 1280
        with self.assertRaisesRegex(ValueError, "all four"):
            campaign._decision(candidate, behavior, dev)

    def test_all_order_accounting_uses_worlds_not_independent_rows(self):
        targets = torch.zeros((8192, 1), dtype=torch.long)
        predictions = [torch.ones_like(targets) for _ in range(4)]
        for p in predictions:
            p[:4] = 0
        predictions[0][4:8] = 0
        tensors = {
            "targets": targets,
            "variant_ids": torch.arange(4).repeat(2048),
            "pair_types": (torch.arange(2048) % 2).repeat_interleave(4),
        }
        result = campaign._all_order_worlds(predictions, tensors)
        self.assertEqual(result["complete_in_all_rotations"], 1)
        self.assertEqual(
            result["worlds_by_number_of_complete_rotations"], [2046, 1, 0, 0, 1]
        )
        self.assertEqual(
            result["by_question_type"]["same_owner"]["complete_in_all_rotations"], 1
        )
        self.assertEqual(
            result["by_question_type"]["same_object"]["complete_in_all_rotations"], 0
        )

    def test_reference_must_reproduce_score_and_diagnostic_before_other_views(self):
        tensors = {
            name: torch.zeros((8192, 1), dtype=torch.long)
            for name in ("inputs", "targets", "group_ids", "variant_ids", "pair_types")
        }
        record, diagnostic = {"top1_correct": 3735}, {"a": 1}
        for baseline in (
            {"construction": {"top1_correct": 3734}, "diagnostic": diagnostic},
            {"construction": record, "diagnostic": {"a": 2}},
        ):
            with (
                patch.object(campaign, "rotate_inputs", return_value=tensors["inputs"]),
                patch.object(
                    campaign,
                    "_score",
                    return_value=(record, tensors["targets"], torch.zeros(1)),
                ) as scorer,
                patch.object(campaign, "analyze", return_value=diagnostic),
                self.assertRaisesRegex(ValueError, "exact full-score/diagnostic"),
            ):
                campaign._score_views(object(), tensors, object(), baseline)
            self.assertEqual(scorer.call_count, 1)


if __name__ == "__main__":
    unittest.main()
