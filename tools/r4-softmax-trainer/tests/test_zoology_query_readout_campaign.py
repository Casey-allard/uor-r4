"""Only #1067's conditional reveal and retention-of-improvement decisions."""

from pathlib import Path
import unittest
from unittest.mock import patch

import torch
from r4_softmax_trainer.zoology_english_binding.data import encode, TOKEN_IDS
from r4_softmax_trainer.zoology_query_readout import campaign


class QueryReadoutDecisionTests(unittest.TestCase):
    def test_final_example_uses_the_source_decoder_and_explicit_answer_position(self):
        prompt = "mara put the key in the drawer. mara put the book in the cabinet. lena put the key in the basket. omar put the coin in the closet. where is mara's key? answer:"
        tensors = {"inputs": torch.tensor([encode(prompt)]), "positions": torch.tensor([[37]]), "targets": torch.tensor([[TOKEN_IDS["drawer"]]])}
        example = campaign._examples(tensors, torch.tensor([[TOKEN_IDS["basket"]]]), 1)[0]
        self.assertEqual(example["prompt"], prompt)
        self.assertEqual(example["supervised_position"], 37)
        self.assertEqual((example["target"], example["prediction"]), ("drawer", "basket"))

    def test_construction_miss_never_loads_or_scores_fresh_development(self):
        record = {"decisions": 8192, "top1_correct": 8110}
        with patch.object(campaign.data, "load_development", side_effect=AssertionError("forbidden development")) as loader, patch.object(campaign, "_plain_score", side_effect=AssertionError("forbidden model score")) as scorer:
            development = campaign._conditional_development(Path("unused"), object(), record, object())
        loader.assert_not_called()
        scorer.assert_not_called()
        self.assertEqual(development["model_decisions"], 0)
        self.assertEqual(campaign._decision(record, development)["status"], "QUERY_OBJECT_READOUT_CONSTRUCTION_MISS")
        self.assertTrue(campaign._construction_fits({"decisions": 8192, "top1_correct": 8111}))

    def test_fresh_transfer_thresholds_and_each_question_type_are_binding(self):
        construction = {"decisions": 8192, "top1_correct": 8111}
        behavior = {
            "known_decisions": 1024, "known_correct": 973,
            "unknown_decisions": 256, "unknown_correct": 244,
            "complete_groups_correct": 232,
            "by_question_type": {name: {"complete_correct": 116} for name in ("same_owner", "same_object")},
        }
        development = {"model_decisions": 1280, "behavior": behavior}
        self.assertEqual(campaign._decision(construction, development)["status"], "QUERY_OBJECT_READOUT_FRESH_BINDING_PASSED")
        for kind in ("same_owner", "same_object"):
            behavior["by_question_type"][kind]["complete_correct"] = 115
            self.assertEqual(campaign._decision(construction, development)["status"], "QUERY_OBJECT_READOUT_FRESH_TRANSFER_MISS")
            behavior["by_question_type"][kind]["complete_correct"] = 116
        behavior["unknown_correct"] = 243
        self.assertFalse(campaign._decision(construction, development)["passed"])

    def test_large_partial_gain_survives_a_construction_miss(self):
        def diagnostic(correct, both):
            pair = {"pairs": 4096, "changed": both, "invariant": 4096-both, "both_correct": both}
            return {
                "paired": {kind: {"all": pair, "pair_type": {name: pair for name in ("same_owner", "same_object")}} for kind in ("question", "location_swap")},
                "strata": {"pair_type": {name: {"correct": correct//2, "rows": 4096} for name in ("same_owner", "same_object")}},
            }
        baseline = {"construction": {"top1_correct": 2396, "top1_rate": 2396/8192, "nll_nats": 1.6}, "diagnostic": diagnostic(2396,20)}
        record = {"decisions": 8192, "top1_correct": 8000, "top1_rate": 8000/8192, "nll_nats": 0.2}
        comparison = campaign._comparison(record, diagnostic(8000,3900), baseline)
        self.assertFalse(campaign._construction_fits(record))
        self.assertEqual(comparison["correct_gain"], 5604)
        self.assertEqual(comparison["pair_comparisons"]["question"]["all"]["both_correct_gain"], 3880)
        self.assertAlmostEqual(comparison["nll_change_nats"], -1.4)


if __name__ == "__main__":
    unittest.main()
