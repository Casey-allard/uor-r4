"""Pure decision gates; no retained data, fitted model or native frame access."""

import copy
import unittest

from r4_softmax_trainer.zoology_language_r4 import campaign


def _row():
    return {
        "records": {
            name: {
                "decisions": count,
                "top1_correct": count,
                "top1_rate": 1.0,
                "nll_nats": 0.01,
                "selected_logits_cid": "frozen-full-head",
            }
            for name, count in (("all", 10), ("supported", 8), ("unknown", 2))
        },
        "prediction_ids": list(range(10)),
        "role_prediction_positions": [[0, 1, 2]] * 10,
        "role_attention_cid": "frozen-full-role-attention",
        "role_accuracy": {
            "correct": 140,
            "decisions": 140,
            "rate": 1.0,
            "by_role": {
                name: {"rate": 1.0} for name in ("owner", "object", "location")
            },
        },
        "groups": {
            "by_question_type": {
                name: {"complete_supported_quartets": 1, "groups": 1}
                for name in ("same_owner", "same_object")
            }
        },
        "syntax_pairs": {"pairs": 2, "both_answers_correct": 2, "complete_rate": 1.0},
        "work": {"rows": 10, "role_decisions": 140, "binding_score_slots": 50},
        "role_vectors_cid": "computed-fifteen-role-vectors",
        "audit": {
            **{key: 0 for key in campaign.AUDIT_COUNTS},
            "token_source_frame_positions_changed": 0,
            "token_source_frame_matrices_changed": 0,
            "fact_source_frame_positions_changed": 0,
            "fact_source_frame_matrices_changed": 0,
            "reached_token_frame_indices": [0, 1, 2],
            "reached_clause_frame_indices": [0, 2],
            "reached_frame_indices": [0, 1, 2],
        },
        "work_valid": True,
    }


def _differences():
    return {
        "strata": {
            name: {
                "top1_changed": 0,
                "logits_max_abs": 0.0,
                "binding_attention_max_abs": 0.0,
                "role_vectors_max_abs": 0.0,
                "nll_abs_difference": 0.0,
            }
            for name in ("all", "supported", "unknown")
        },
        "role_attention_exact": True,
        "role_predictions_exact": True,
    }


class LanguageR4DecisionTests(unittest.TestCase):
    def test_historical_reproduction_binds_full_learned_fields_not_hard_oracle(self):
        row = _row()
        historical = {
            key: value
            for key, value in copy.deepcopy(row).items()
            if key not in ("role_vectors_cid", "audit", "work_valid")
        }
        historical.update(
            qualification=campaign.ordinary._qualified(row),
            oracle={"learned_max_logit_difference": 0.0623424},
            view_id=3,
            syntax="heldout",
        )
        self.assertTrue(campaign._historical_exact(row, historical))
        for key, replacement in (
            ("role_attention_cid", "changed-role-weights"),
            ("prediction_ids", [0] * 10),
            ("role_prediction_positions", [[1, 0, 2]] * 10),
        ):
            changed = copy.deepcopy(row)
            changed[key] = replacement
            self.assertFalse(campaign._historical_exact(changed, historical), key)
        changed = copy.deepcopy(row)
        changed["records"]["unknown"]["nll_nats"] += 1e-12
        self.assertFalse(campaign._historical_exact(changed, historical))
        historical["oracle"] = {"not_a_preservation_comparator": True}
        self.assertTrue(campaign._historical_exact(row, historical))

    def test_all_strata_and_computed_role_vector_parity_are_binding(self):
        plain, coherent = _row(), _row()
        plain["historical_exact"] = True
        self.assertTrue(
            campaign._primary_decision(plain, coherent, _differences())["passed"]
        )
        for stratum in ("all", "supported", "unknown"):
            for metric, value in (
                ("top1_changed", 1),
                ("logits_max_abs", 0.005001),
                ("binding_attention_max_abs", 1.0001e-5),
                ("role_vectors_max_abs", 1.0001e-5),
                ("nll_abs_difference", 1.0001e-5),
            ):
                delta = _differences()
                delta["strata"][stratum][metric] = value
                self.assertFalse(
                    campaign._primary_decision(plain, coherent, delta)["passed"],
                    (stratum, metric),
                )
        for field in ("role_attention_exact", "role_predictions_exact"):
            delta = _differences()
            delta[field] = False
            self.assertFalse(
                campaign._primary_decision(plain, coherent, delta)["passed"]
            )
        delta = _differences()
        del delta["strata"]["unknown"]
        self.assertFalse(campaign._primary_decision(plain, coherent, delta)["passed"])
        coherent["work_valid"] = False
        self.assertFalse(
            campaign._primary_decision(plain, coherent, _differences())["passed"]
        )

    def test_controls_are_unreachable_after_reference_or_primary_miss(self):
        def forbidden():
            self.fail("control must not execute after a primary miss")

        control = campaign._conditional_controls({"passed": False}, forbidden)
        self.assertEqual(control["model_decisions"], 0)
        self.assertEqual(
            campaign._decision(False, {"passed": False}, control)["status"],
            "LANGUAGE_R4_REFERENCE_MISMATCH",
        )
        self.assertEqual(
            campaign._decision(True, {"passed": False}, control)["status"],
            "LANGUAGE_R4_PRESERVATION_MISS",
        )
        self.assertEqual(
            campaign._conditional_controls({"passed": True}, lambda: "executed"),
            "executed",
        )

    def test_control_attribution_never_erases_valid_primary_preservation(self):
        primary = {"plain": _row(), "r4": _row()}
        for execution, seam in (
            ("token_source_frame_permuted", "token"),
            ("fact_source_frame_permuted", "fact"),
        ):
            controlled = _row()
            controlled["records"]["supported"].update(top1_correct=4, top1_rate=0.5)
            controlled["audit"][f"{seam}_source_frame_positions_changed"] = 10
            controlled["audit"][f"{seam}_source_frame_matrices_changed"] = 8
            preflight = {
                "passed": True,
                "controls": {
                    execution: {
                        "passed": True,
                        "source_frame_positions_changed": 10,
                        "source_frame_matrices_changed": 8,
                    }
                },
            }
            at_threshold = campaign._control_decision(
                primary, controlled, execution, preflight
            )
            self.assertTrue(at_threshold["strong_transport_sensitivity"])
            self.assertEqual(at_threshold["supported_drop_percentage_points"], 50)
            changed_count = copy.deepcopy(controlled)
            changed_count["audit"][f"{seam}_source_frame_matrices_changed"] -= 1
            self.assertFalse(
                campaign._control_decision(
                    primary, changed_count, execution, preflight
                )["valid"]
            )
            wrong_work = copy.deepcopy(controlled)
            wrong_work["audit"]["token_blocks_transported"] += 16
            self.assertFalse(
                campaign._control_decision(primary, wrong_work, execution, preflight)[
                    "valid"
                ]
            )
            if seam == "fact":
                wrong_vectors = copy.deepcopy(controlled)
                wrong_vectors["role_vectors_cid"] = "changed-first-seam"
                self.assertFalse(
                    campaign._control_decision(
                        primary, wrong_vectors, execution, preflight
                    )["valid"]
                )
        for control, expected in (
            (
                {"valid": False, "strong_transport_sensitivity": False},
                "LANGUAGE_R4_PRESERVED_CONTROL_INVALID",
            ),
            (
                {"valid": True, "strong_transport_sensitivity": False},
                "LANGUAGE_R4_PRESERVED_CONTROL_WEAK",
            ),
            (
                {"valid": True, "strong_transport_sensitivity": True},
                "LANGUAGE_R4_PRESERVED",
            ),
        ):
            decision = campaign._decision(True, {"passed": True}, control)
            self.assertTrue(decision["preserved"])
            self.assertEqual(decision["status"], expected)
        self.assertEqual(
            campaign._decision(True, {"passed": True}, control)["next_action"],
            "RETAIN_TWO_STAGE_R4_AND_SEPARATELY_FREEZE_CAUSAL_OUTPUT_STATE_PROTOTYPE",
        )


if __name__ == "__main__":
    unittest.main()
