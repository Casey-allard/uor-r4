"""Focused synthetic scorer and frozen-decision checks; no fitted artifact."""

from __future__ import annotations

import copy
import unittest
from types import SimpleNamespace

import torch

from r4_softmax_trainer.zoology_r4_inference.campaign import (
    _WORK_FIELDS,
    _control_decision,
    _maximum_difference,
    _primary_decision,
    _Scores,
)


def _output(logits: torch.Tensor) -> SimpleNamespace:
    return SimpleNamespace(
        logits=logits, attention_weights=(torch.tensor([[[[1.0, 0.0], [0.25, 0.75]]]]),)
    )


class ZoologyR4CampaignTests(unittest.TestCase):
    def test_scorer_accounts_labels_without_changing_outputs_and_keeps_dense_read_unknown(
        self,
    ) -> None:
        logits = torch.tensor([[[2.0, 0.0], [0.0, 2.0]]])
        output = _output(logits)
        before = logits.clone()
        good, bad = _Scores(), _Scores()
        audit = {
            "execution": "plain",
            "future_position_reads": None,
            "materialized_score_slots": 4,
        }
        good.add(output, torch.tensor([[0, 1]]), audit)
        bad.add(output, torch.tensor([[1, 0]]), audit)
        self.assertTrue(torch.equal(before, logits))
        self.assertEqual(good.record()["top1_correct"], 2)
        self.assertEqual(bad.record()["top1_correct"], 0)
        self.assertEqual(
            good.record()["selected_logits_cid"], bad.record()["selected_logits_cid"]
        )
        self.assertEqual(
            good.record()["predictions_cid"], bad.record()["predictions_cid"]
        )
        self.assertEqual(good.record()["future_attention_nonzero"], 0)
        self.assertIsNone(good.record()["audit_totals"]["future_position_reads"])
        self.assertEqual(good.record()["audit_totals"]["materialized_score_slots"], 4)

    def test_comparison_checks_attention_as_well_as_top1(self) -> None:
        plain = _output(torch.tensor([[[2.0, 0.0], [0.0, 2.0]]]))
        changed = copy.deepcopy(plain)
        changed.logits[0, 0, 0] += 0.125
        changed.attention_weights[0][0, 0, 1] = torch.tensor([0.5, 0.5])
        self.assertEqual(_maximum_difference(plain, changed), (0.125, 0.25))
        changed.attention_weights = ()
        with self.assertRaisesRegex(ValueError, "attention layers"):
            _maximum_difference(plain, changed)

    def test_primary_uses_all_decisions_frozen_tolerances_and_state_integrity(
        self,
    ) -> None:
        score = {
            "decisions": 12000,
            "top1_correct": 11900,
            "future_attention_nonzero": 0,
            "audit_totals": {"future_position_reads": 0},
        }
        differences = {
            "top1_changed": 0,
            "selected_logits_max_abs": 0.005,
            "attention_max_abs": 1e-5,
            "nll_abs_difference": 1e-5,
        }
        self.assertTrue(
            _primary_decision(score, score, differences, state_unchanged=True)["passed"]
        )
        self.assertFalse(
            _primary_decision(score, score, differences, state_unchanged=False)[
                "passed"
            ]
        )
        mismatch = {**differences, "top1_changed": 1}
        self.assertFalse(
            _primary_decision(score, score, mismatch, state_unchanged=True)["passed"]
        )
        incomplete = {**score, "decisions": 11999}
        self.assertFalse(
            _primary_decision(incomplete, score, differences, state_unchanged=True)[
                "passed"
            ]
        )

    def test_control_drop_requires_matching_causal_work_for_attribution(self) -> None:
        plain = {"top1_rate": 0.99}
        r4 = {"audit_totals": dict.fromkeys(_WORK_FIELDS, 0)}
        r4["audit_totals"]["admitted_attention_pairs"] = 120
        control = {
            "decisions": 12000,
            "top1_rate": 0.20,
            "future_attention_nonzero": 0,
            "audit_totals": dict(r4["audit_totals"]),
        }
        self.assertTrue(
            _control_decision(plain, r4, control)["strong_transport_sensitivity"]
        )
        control["audit_totals"]["admitted_attention_pairs"] = 119
        decision = _control_decision(plain, r4, control)
        self.assertEqual(decision["status"], "INVALID_CONTROL_INTEGRITY")
        self.assertFalse(decision["strong_transport_sensitivity"])
        control["audit_totals"] = dict(r4["audit_totals"])
        control["future_attention_nonzero"] = 1
        self.assertFalse(
            _control_decision(plain, r4, control)["strong_transport_sensitivity"]
        )


if __name__ == "__main__":
    unittest.main()
