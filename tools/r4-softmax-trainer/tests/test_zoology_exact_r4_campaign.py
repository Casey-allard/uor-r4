"""Only new exact-data decision/accounting boundaries; no fitted model reads."""

from __future__ import annotations

import copy
import unittest

from r4_softmax_trainer.zoology_exact_r4_inference.campaign import (
    _control_decision,
    _primary_decision,
)
from r4_softmax_trainer.zoology_r4_inference.campaign import _WORK_FIELDS


def _score() -> dict:
    work = dict.fromkeys(_WORK_FIELDS, 0)
    work["admitted_attention_pairs"] = 14868480
    return {
        "decisions": 8192,
        "top1_correct": 8071,
        "top1_rate": 8071 / 8192,
        "future_attention_nonzero": 0,
        "audit_totals": work,
    }


def _differences() -> dict:
    return {
        "top1_changed": 0,
        "selected_logits_max_abs": 0.005,
        "attention_max_abs": 1e-5,
        "nll_abs_difference": 1e-5,
    }


class ZoologyExactR4CampaignTests(unittest.TestCase):
    def test_preserves_the_final_checkpoint_without_requiring_a_new_99_percent_score(
        self,
    ) -> None:
        score = _score()
        result = _primary_decision(
            score, score, _differences(), state_unchanged=True, vocabulary_covered=True
        )
        self.assertTrue(result["passed"])
        changed = {**score, "top1_correct": 8070}
        self.assertFalse(
            _primary_decision(
                changed,
                changed,
                _differences(),
                state_unchanged=True,
                vocabulary_covered=True,
            )["passed"]
        )

    def test_requires_all_exact_data_decisions_and_all_causal_pairs(self) -> None:
        score = _score()
        missing = copy.deepcopy(score)
        missing["audit_totals"]["admitted_attention_pairs"] -= 1
        self.assertFalse(
            _primary_decision(
                score,
                missing,
                _differences(),
                state_unchanged=True,
                vocabulary_covered=True,
            )["passed"]
        )
        for differences in (
            {**_differences(), "top1_changed": 1},
            {**_differences(), "attention_max_abs": 2e-5},
        ):
            self.assertFalse(
                _primary_decision(
                    score,
                    score,
                    differences,
                    state_unchanged=True,
                    vocabulary_covered=True,
                )["passed"]
            )
        self.assertFalse(
            _primary_decision(
                score,
                score,
                _differences(),
                state_unchanged=False,
                vocabulary_covered=True,
            )["passed"]
        )

    def test_control_attribution_requires_same_complete_work_and_preserves_primary(
        self,
    ) -> None:
        plain, r4, control = _score(), _score(), _score()
        control["top1_rate"] = 0.10
        self.assertTrue(
            _control_decision(plain, r4, control)["strong_transport_sensitivity"]
        )
        control["decisions"] = 8191
        result = _control_decision(plain, r4, control)
        self.assertEqual(result["status"], "INVALID_CONTROL_INTEGRITY")
        self.assertFalse(result["strong_transport_sensitivity"])
        self.assertTrue(
            _primary_decision(
                plain, r4, _differences(), state_unchanged=True, vocabulary_covered=True
            )["passed"]
        )
        control["decisions"] = 8192
        control["audit_totals"]["key_blocks_transported"] += 1
        self.assertFalse(
            _control_decision(plain, r4, control)["strong_transport_sensitivity"]
        )


if __name__ == "__main__":
    unittest.main()
