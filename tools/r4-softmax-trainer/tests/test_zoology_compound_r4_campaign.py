"""Decision checks for rectangular preservation and conditional transport controls."""

import copy
import unittest
from types import SimpleNamespace

import torch

from r4_softmax_trainer.zoology_compound_r4 import campaign


def audit(rows=10, execution="r4"):
    return {
        "rows": rows,
        "admitted_attention_pairs": rows * 5,
        "materialized_score_slots": rows * 5,
        "null_attention_pairs": rows,
        "future_score_slots_materialized": 0,
        "future_position_reads": 0,
        "source_frame_positions_changed": rows * 4
        if execution == "source_frame_permuted"
        else 0,
        "source_frame_matrices_changed": rows * 4
        if execution == "source_frame_permuted"
        else 0,
        "reached_frame_indices": [0, 1, 2, 3, 4],
        **{
            key: 0 if execution == "plain" else value * rows
            for key, value in campaign.TRANSFORM_FACTORS.items()
        },
    }


def view(execution="r4", known_correct=8, unknown_correct=2):
    counts = {
        "all": (10, known_correct + unknown_correct),
        "supported": (8, known_correct),
        "unknown": (2, unknown_correct),
    }
    return {
        "records": {
            key: {
                "decisions": total,
                "top1_correct": correct,
                "top1_rate": correct / total,
                "nll_nats": 0.1,
            }
            for key, (total, correct) in counts.items()
        },
        "audit": audit(execution=execution),
        "historical_exact": True,
    }


def differences():
    return {
        key: {
            "top1_changed": 0,
            "logits_max_abs": 0.0,
            "attention_max_abs": 0.0,
            "nll_abs_difference": 0.0,
        }
        for key in ("all", "supported", "unknown")
    }


class CompoundR4CampaignTests(unittest.TestCase):
    def test_historical_mapping_requires_full_records_for_both_populations(self):
        records = view()["records"]
        source = {
            "baseline_history": {
                "construction_views": [
                    {
                        "rotation": 0,
                        "all": records["all"],
                        "construction": records["supported"],
                        "unknown": records["unknown"],
                    }
                ],
                "development": {
                    "views": [
                        {
                            "rotation": 0,
                            "record": records["all"],
                            "supported": records["supported"],
                            "unknown": records["unknown"],
                        }
                    ]
                },
            }
        }
        for population in campaign.ROWS:
            self.assertEqual(
                campaign._historical_records(source, population, 0), records
            )
        changed = copy.deepcopy(records)
        changed["unknown"]["nll_nats"] += 1e-12
        self.assertNotEqual(
            campaign._historical_records(source, "construction", 0), changed
        )

    def test_every_stratum_and_causal_work_gate_is_binding(self):
        plain, coherent = view("plain"), view()
        self.assertTrue(
            campaign._primary_decision(plain, coherent, differences(), True)["passed"]
        )
        for stratum in ("all", "supported", "unknown"):
            for key, value in (
                ("top1_changed", 1),
                ("logits_max_abs", 0.00501),
                ("attention_max_abs", 1.01e-5),
                ("nll_abs_difference", 1.01e-5),
            ):
                delta = differences()
                delta[stratum][key] = value
                self.assertFalse(
                    campaign._primary_decision(plain, coherent, delta, True)["passed"]
                )
        delta = differences()
        del delta["unknown"]
        self.assertFalse(
            campaign._primary_decision(plain, coherent, delta, True)["passed"]
        )
        bad = copy.deepcopy(coherent)
        bad["audit"]["null_attention_pairs"] = 0
        self.assertFalse(
            campaign._primary_decision(plain, bad, differences(), True)["passed"]
        )
        bad = copy.deepcopy(coherent)
        bad["audit"]["future_position_reads"] = 1
        self.assertFalse(
            campaign._primary_decision(plain, bad, differences(), True)["passed"]
        )
        self.assertFalse(
            campaign._primary_decision(plain, coherent, differences(), False)["passed"]
        )

    def test_control_threshold_and_integrity_preserve_primary_interpretation(self):
        primary = {"plain": view("plain"), "r4": view()}
        broken = view("source_frame_permuted", known_correct=4, unknown_correct=0)
        preflight = {"source_frame_matrices_changed": 40}
        decision = campaign._control_decision(primary, broken, preflight)
        self.assertTrue(decision["strong_transport_sensitivity"])
        self.assertEqual(decision["supported_drop_percentage_points"], 50)
        weak = campaign._control_decision(
            primary, view("source_frame_permuted", known_correct=5), preflight
        )
        terminal = campaign._decision(True, {"passed": True}, weak)
        self.assertTrue(terminal["preserved"])
        self.assertEqual(terminal["status"], "COMPOUND_R4_PRESERVED_CONTROL_WEAK")
        broken["audit"]["value_blocks_transported"] -= 16
        invalid = campaign._control_decision(primary, broken, preflight)
        self.assertFalse(invalid["valid"])
        self.assertEqual(
            campaign._decision(True, {"passed": True}, invalid)["status"],
            "COMPOUND_R4_PRESERVED_CONTROL_INVALID",
        )

    def test_control_is_unreachable_after_any_primary_miss(self):
        def forbidden():
            self.fail("control must not execute")

        control = campaign._conditional_controls({"passed": False}, forbidden)
        self.assertEqual(control["model_decisions"], 0)
        self.assertEqual(
            campaign._decision(False, {"passed": False}, control)["status"],
            "COMPOUND_R4_REFERENCE_MISMATCH",
        )
        self.assertEqual(
            campaign._decision(True, {"passed": False}, control)["status"],
            "COMPOUND_R4_PRESERVATION_MISS",
        )
        self.assertTrue(
            campaign._conditional_controls({"passed": True}, lambda: {"called": True})[
                "called"
            ]
        )

    def test_preflight_measures_label_free_ceiling_and_keeps_null_fixed(self):
        inputs = torch.zeros(4, 41, dtype=torch.long)
        supported = torch.tensor([True, True, False, False])
        frames = SimpleNamespace(
            identity_index=0,
            cumulative_frame_indices=lambda prefix: (
                torch.arange(38).remainder(5).expand(len(prefix), -1)
            ),
        )
        result = campaign._frame_view(inputs, supported, frames)
        self.assertTrue(result["passed"])
        self.assertEqual(result["supported_loss_reachability_ceiling"], 1.0)
        self.assertEqual(result["source_frame_matrices_changed"], 16)
        self.assertEqual(result["null_attention_pairs"], 4)
        inputs[:, 38:] = -900
        self.assertEqual(campaign._frame_view(inputs, supported, frames), result)
        frames.cumulative_frame_indices = lambda prefix: torch.zeros(
            len(prefix), 38, dtype=torch.long
        )
        self.assertFalse(campaign._frame_view(inputs, supported, frames)["passed"])

    def test_group_summary_separates_supported_quartets_from_absence(self):
        targets = torch.arange(10).reshape(10, 1)
        tensors = {"targets": targets, "pair_types": torch.tensor([0] * 5 + [1] * 5)}
        result = campaign._groups(targets, tensors)
        self.assertEqual(result["complete_supported_quartets"], 2)
        self.assertEqual(result["complete_five_answer_groups"], 2)
        predictions = targets.clone()
        predictions[4] = 999
        result = campaign._groups(predictions, tensors)
        self.assertEqual(result["complete_supported_quartets"], 2)
        self.assertEqual(result["complete_five_answer_groups"], 1)


if __name__ == "__main__":
    unittest.main()
