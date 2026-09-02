"""Synthetic semantic diagnostics only; no fitted model or artifact reads."""

from __future__ import annotations

import json
import unittest

import torch
from r4_softmax_trainer.zoology_english_binding.data import TOKEN_IDS, encode
from r4_softmax_trainer.zoology_english_diagnostic.analysis import (
    CATEGORIES,
    choose_focus,
    classify_row,
    pair_summary,
    summarize_rows,
)


def _input(owner: str = "mara", obj: str = "key") -> list[int]:
    return encode(
        "mara put the key in the drawer. "
        "mara put the book in the cabinet. "
        "lena put the key in the basket. "
        "omar put the coin in the closet. "
        f"where is {owner}'s {obj}? answer:"
    )


class EnglishDiagnosticAnalysisTests(unittest.TestCase):
    def test_decoded_roles_partition_full_head_and_q1_eligibility_is_explicit(
        self,
    ) -> None:
        selected_words = (
            "drawer",
            "cabinet",
            "basket",
            "closet",
            "pouch",
            "unknown",
            "mara",
        )
        rows = [
            classify_row(_input(), TOKEN_IDS["drawer"], TOKEN_IDS[word])
            for word in selected_words
        ]
        self.assertEqual(tuple(row.category for row in rows), CATEGORIES)
        self.assertTrue(all(row.eligible_facts == (1, 1, 1, 1) for row in rows))
        self.assertEqual(rows[2].selected_slot, 2)
        self.assertIsNone(rows[4].selected_slot)
        reserved = classify_row(_input(), TOKEN_IDS["drawer"], 4095)
        self.assertEqual(reserved.category, "other_vocabulary")
        same_owner_q1 = classify_row(
            _input("mara", "book"), TOKEN_IDS["cabinet"], TOKEN_IDS["basket"]
        )
        self.assertEqual(same_owner_q1.category, "unrelated_fact_location")
        self.assertEqual(same_owner_q1.eligible_facts, (1, 1, 0, 2))
        summary = summarize_rows([same_owner_q1])
        absent = summary["categories"]["same_object_confound"]
        self.assertEqual(
            (absent["count"], absent["eligible_rows"], absent["eligible_facts"]),
            (0, 0, 0),
        )
        self.assertIsNone(absent["rate_per_eligible_row"])
        self.assertIsNone(absent["rate_per_eligible_fact"])
        unrelated = summary["categories"]["unrelated_fact_location"]
        self.assertEqual(unrelated["rate_per_eligible_row"], 1.0)
        self.assertEqual(unrelated["rate_per_eligible_fact"], 0.5)
        same_object_q1 = classify_row(
            _input("lena", "key"), TOKEN_IDS["basket"], TOKEN_IDS["cabinet"]
        )
        self.assertEqual(same_object_q1.eligible_facts, (1, 0, 1, 2))
        self.assertIsNone(
            summarize_rows([same_object_q1])["categories"]["same_owner_confound"][
                "rate_per_eligible_row"
            ]
        )
        slots = summary["displayed_slots"]
        self.assertEqual(slots["1"]["target_exposure"], 1)
        self.assertEqual(slots["2"]["selections"], 1)
        self.assertEqual(slots["1"]["accuracy_when_target_at_slot"], 0.0)
        self.assertIsNone(slots["2"]["accuracy_when_target_at_slot"])
        with self.assertRaisesRegex(ValueError, "parsed input"):
            classify_row(_input(), TOKEN_IDS["cabinet"], TOKEN_IDS["drawer"])
        json.dumps(summary, allow_nan=False)

    def test_pairs_distinguish_prediction_invariance_from_fixed_target_logit_change(
        self,
    ) -> None:
        logits = torch.tensor(
            [
                [5, 1, 0, -1, 9],
                [2, 4, 0, -1, 9],
                [0, 0, 3, 1, 0],
                [0, 0, 1, 3, 0],
                [6, 1, 0, 0, 10],
                [0, 3, 0, 4, 1],
            ],
            dtype=torch.float32,
        )
        before = logits.clone()
        targets = torch.tensor([[0], [1], [2], [3], [0], [1]])
        predictions = logits.argmax(dim=1, keepdim=True)
        pairs = torch.tensor([[0, 1], [2, 3], [4, 5]])
        summary = pair_summary(logits, targets, predictions, pairs)
        self.assertEqual(summary["pairs"], 3)
        self.assertEqual(summary["changed"], 2)
        self.assertEqual(summary["invariant"], 1)
        self.assertEqual(summary["both_correct"], 1)
        self.assertEqual(summary["changed_but_not_both_correct"], 1)
        contrast = summary["target_contrast_delta"]
        self.assertEqual(
            (contrast["mean"], contrast["median"], contrast["min"], contrast["max"]),
            (6.0, 6.0, 4.0, 8.0),
        )
        self.assertEqual(
            (contrast["positive"], contrast["zero"], contrast["negative"]), (3, 0, 0)
        )
        absolute = summary["full_vocabulary_absolute_logit_difference"]
        self.assertEqual(absolute["max"], 9.0)
        self.assertAlmostEqual(absolute["mean"], 31 / 15)
        self.assertEqual(summary["left_target_versus_best_other_margin"]["min"], -4.0)
        self.assertEqual(summary["right_target_versus_best_other_margin"]["min"], -5.0)
        self.assertTrue(torch.equal(logits, before))
        empty = pair_summary(
            logits, targets, predictions, torch.empty((0, 2), dtype=torch.long)
        )
        self.assertIsNone(empty["invariant_rate"])
        self.assertIsNone(empty["target_contrast_delta"]["mean"])
        self.assertIsNone(empty["full_vocabulary_absolute_logit_difference"]["mean"])
        with self.assertRaisesRegex(ValueError, "distinct answers"):
            pair_summary(logits, targets, predictions, torch.tensor([[0, 4]]))
        json.dumps(summary, allow_nan=False)

    def test_focus_uses_strict_majorities_correct_attribute_direction_and_empty_rates(
        self,
    ) -> None:
        cases = (
            ([6, 2, 1, 1], [6, 2, 2], 6, "JOINT_POSITION_ATTRIBUTE"),
            ([6, 2, 1, 1], [4, 3, 3], 6, "POSITION_READOUT"),
            ([3, 3, 2, 2], [6, 2, 2], 6, "OBJECT_DISAMBIGUATION"),
            ([3, 3, 2, 2], [2, 6, 2], 6, "OWNER_DISAMBIGUATION"),
            ([3, 3, 2, 2], [4, 3, 3], 6, "QUESTION_READOUT"),
            ([5, 3, 1, 1], [5, 3, 2], 5, "DISTRIBUTED_BINDING"),
        )
        for slots, counts, invariant, expected in cases:
            with self.subTest(expected=expected):
                errors = dict(zip(CATEGORIES[1:4], counts, strict=True))
                decision = choose_focus(slots, errors, 10, invariant)
                self.assertEqual(decision["label"], expected)
        empty = choose_focus([0, 0, 0, 0], dict.fromkeys(CATEGORIES[1:4], 0), 0, 0)
        self.assertTrue(
            all(item["flag"] is None for item in empty["majority_flags"].values())
        )
        self.assertTrue(
            all(item["rate"] is None for item in empty["majority_flags"].values())
        )
        json.dumps(empty, allow_nan=False)


if __name__ == "__main__":
    unittest.main()
