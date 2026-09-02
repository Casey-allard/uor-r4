"""Decision-bearing owner gain, preservation, reveal and dependent-pair accounting."""

import unittest
from pathlib import Path
from unittest.mock import patch

import torch

from r4_softmax_trainer.zoology_english_binding.data import TOKEN_IDS, encode
from r4_softmax_trainer.zoology_joint_query import campaign


def diagnostic(owner=150, obj=447):
    return {
        "paired": {
            "question": {
                "pair_type": {
                    "same_object": {"pairs": 2048, "both_correct": owner},
                    "same_owner": {"pairs": 2048, "both_correct": obj},
                }
            }
        }
    }


UNSCORED = {"model_decisions": 0, "behavior": None}


class JointQueryDecisionTests(unittest.TestCase):
    def test_final_example_preserves_explicit_answer_interface(self):
        prompt = "mara put the key in the drawer. mara put the book in the cabinet. lena put the key in the basket. omar put the coin in the closet. where is mara's key? answer:"
        tensors = {
            "inputs": torch.tensor([encode(prompt)]),
            "positions": torch.tensor([[37]]),
            "targets": torch.tensor([[TOKEN_IDS["drawer"]]]),
        }
        example = campaign._examples(tensors, torch.tensor([[TOKEN_IDS["basket"]]]), 1)[
            0
        ]
        self.assertEqual(example["prompt"], prompt)
        self.assertEqual(example["supervised_position"], 37)
        self.assertEqual(
            (example["target"], example["prediction"]), ("drawer", "basket")
        )

    def test_construction_miss_never_loads_or_scores_fresh_development(self):
        record = {"decisions": 8192, "top1_correct": 8110}
        with (
            patch.object(
                campaign.data,
                "load_development",
                side_effect=AssertionError("forbidden development"),
            ) as loader,
            patch.object(
                campaign,
                "_plain_score",
                side_effect=AssertionError("forbidden model score"),
            ) as scorer,
        ):
            development = campaign._conditional_development(
                Path("unused"), object(), record, object()
            )
        loader.assert_not_called()
        scorer.assert_not_called()
        self.assertEqual(development["model_decisions"], 0)
        self.assertEqual(
            campaign._decision(record, development, diagnostic(1966, 1966))["status"],
            "JOINT_QUERY_PARTIAL_GAIN",
        )
        self.assertTrue(
            campaign._construction_fits({"decisions": 8192, "top1_correct": 8111})
        )
        with self.assertRaisesRegex(ValueError, "unscored"):
            campaign._decision(
                record, {"model_decisions": 1280, "behavior": {}}, diagnostic()
            )

    def test_owner_effect_and_both_preservation_floors_have_distinct_actions(self):
        record = {"decisions": 8192, "top1_correct": 3735}
        positive = campaign._decision(record, UNSCORED, diagnostic())
        self.assertEqual(positive["status"], "JOINT_QUERY_PARTIAL_GAIN")
        self.assertTrue(positive["behavior"]["passed"])
        self.assertFalse(positive["passed"])
        self.assertEqual(positive["behavior"]["owner_both_correct_gain"], 103)
        self.assertEqual(
            positive["behavior"]["owner_gain_percentage_points"], 10300 / 2048
        )
        smaller = campaign._decision(record, UNSCORED, diagnostic(149))
        self.assertEqual(smaller["status"], "JOINT_QUERY_BELOW_DECLARED_OWNER_GAIN")
        self.assertEqual(smaller["behavior"]["owner_both_correct_gain"], 102)
        self.assertNotEqual(smaller["next_action"], positive["next_action"])
        for overall, obj in ((3734, 447), (3735, 446)):
            with self.subTest(overall=overall, obj=obj):
                lost = campaign._decision(
                    {**record, "top1_correct": overall}, UNSCORED, diagnostic(150, obj)
                )
                self.assertEqual(lost["status"], "JOINT_QUERY_PRESERVATION_MISS")
                self.assertTrue(lost["behavior"]["criteria"]["owner_pair_gain"])
                self.assertFalse(lost["behavior"]["passed"])

    def test_fresh_transfer_thresholds_remain_separate_and_type_specific(self):
        construction = {"decisions": 8192, "top1_correct": 8111}
        behavior = {
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
        development = {"model_decisions": 1280, "behavior": behavior}
        diag = diagnostic(1967, 1967)
        self.assertEqual(
            campaign._decision(construction, development, diag)["status"],
            "JOINT_QUERY_FRESH_BINDING_PASSED",
        )
        for kind in ("same_owner", "same_object"):
            behavior["by_question_type"][kind]["complete_correct"] = 115
            self.assertEqual(
                campaign._decision(construction, development, diag)["status"],
                "JOINT_QUERY_FRESH_TRANSFER_MISS",
            )
            behavior["by_question_type"][kind]["complete_correct"] = 116
        behavior["unknown_correct"] = 243
        self.assertFalse(campaign._decision(construction, development, diag)["passed"])
        with self.assertRaisesRegex(ValueError, "contradicts"):
            campaign._decision(construction, development, diagnostic(149, 1967))

    def test_world_accounting_retains_two_related_pairs_per_world(self):
        targets = torch.zeros((8192, 1), dtype=torch.long)
        predictions = torch.ones_like(targets).reshape(2048, 4)
        # One world of each type with one successful pair; another with two.
        predictions[0:2, :2] = 0
        predictions[2:4] = 0
        tensors = {
            "targets": targets,
            "variant_ids": torch.arange(4).repeat(2048),
            "pair_types": (torch.arange(2048) % 2).repeat_interleave(4),
        }
        result = campaign._world_pair_counts(predictions.reshape(8192, 1), tensors)
        for kind in ("owner_changing", "object_changing"):
            self.assertEqual(
                result[kind]["worlds_by_successful_question_pairs"], [1022, 1, 1]
            )
            self.assertEqual(result[kind]["complete_quartets"], 1)
            self.assertEqual(result[kind]["worlds"], 1024)
        tensors["variant_ids"][0] = 1
        with self.assertRaisesRegex(ValueError, "order"):
            campaign._world_pair_counts(predictions.reshape(8192, 1), tensors)


if __name__ == "__main__":
    unittest.main()
