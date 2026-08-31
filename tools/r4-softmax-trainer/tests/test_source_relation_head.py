from __future__ import annotations

import unittest

import torch

from r4_softmax_trainer.provenance import canonical_json_bytes, cid_bytes
from r4_softmax_trainer.source_relation_data import (
    build_relation_preflight,
    build_source_relation_population,
)
from r4_softmax_trainer.source_relation_head import (
    R4SourceRelationHead,
    _decision_from_logits,
    _development_control_metrics,
    _head_payload,
)


def _perfect_evaluation(record: dict[str, object]) -> dict[str, object]:
    spans = list(record["sentence_spans"])
    logits = [1.0 if int(span["relation_label"]) else -1.0 for span in spans]
    return {
        "record_cid": record["record_cid"],
        "source_cid": record["source_cid"],
        "question_cid": record["question_cid"],
        "target_outcome": record["target_outcome"],
        "target_span_index": record["target_span_index"],
        **_decision_from_logits(record, logits),
    }


class SourceRelationHeadTests(unittest.TestCase):
    def test_strict_positive_duplicate_aware_decisions(self) -> None:
        preflight = build_relation_preflight()
        records = [*preflight["fit"], *preflight["sealed"]]
        by_motif = {record["motif"]: record for record in records}

        absent = by_motif["same-source-absent-query"]
        absent_result = _decision_from_logits(
            absent, [0.0] * len(absent["sentence_spans"])
        )
        self.assertEqual(absent_result["decision"], "abstain")

        duplicate = by_motif["exact-duplicate-agreement"]
        duplicate_result = _decision_from_logits(
            duplicate,
            [
                1.0 if int(span["relation_label"]) else -1.0
                for span in duplicate["sentence_spans"]
            ],
        )
        self.assertEqual(duplicate_result["decision"], "answer")
        self.assertEqual(
            duplicate_result["selected_span_index"], duplicate["target_span_index"]
        )

        conflict = by_motif["distinct-location-conflict"]
        conflict_result = _decision_from_logits(
            conflict,
            [
                1.0 if int(span["relation_label"]) else -1.0
                for span in conflict["sentence_spans"]
            ],
        )
        self.assertEqual(conflict_result["decision"], "conflict")
        self.assertIsNone(conflict_result["selected_span_index"])

    def test_head_payload_matches_the_strict_rust_schema_and_self_cid(self) -> None:
        torch.manual_seed(9_542)
        model = R4SourceRelationHead()
        dataset, _, _ = build_source_relation_population()
        payload = _head_payload(
            model,
            dataset=dataset,
            run_contract_cid="blake3:" + "1" * 64,
            training_result_cid="blake3:" + "2" * 64,
            preflight={"status": "TEST_ONLY"},
            development_metrics={"status": "TEST_ONLY"},
        )
        self.assertEqual(
            set(payload),
            {
                "artifact_cid",
                "dataset_cid",
                "development_metrics",
                "first_layer_biases",
                "first_layer_weights",
                "hidden_size",
                "hidden_width",
                "issue",
                "maximum_source_spans",
                "model_weights_cid",
                "output_bias",
                "output_weights",
                "policy",
                "preflight",
                "product_probe_commitments",
                "relation_input_policy",
                "run_contract_cid",
                "schema",
                "split_policy_cid",
                "threshold",
                "tokenizer_cid",
                "training_result_cid",
            },
        )
        unsigned = dict(payload)
        embedded = unsigned.pop("artifact_cid")
        self.assertEqual(embedded, cid_bytes(canonical_json_bytes(unsigned)))

    def test_perfect_reversal_and_all_outcome_query_swaps_are_reachable(self) -> None:
        dataset, _, _ = build_source_relation_population()
        development = list(dataset["development"])
        reversals = list(dataset["development_controls"]["reversal"])
        swaps = list(dataset["development_controls"]["query_swap"])
        base_metrics = {"records": [_perfect_evaluation(record) for record in development]}
        reversal_metrics = {"records": [_perfect_evaluation(record) for record in reversals]}
        swap_metrics = {"records": [_perfect_evaluation(record) for record in swaps]}

        controls = _development_control_metrics(
            base_metrics,
            reversals,
            reversal_metrics,
            swaps,
            swap_metrics,
        )
        self.assertTrue(controls["order_equivariance"]["exact"])
        self.assertEqual(controls["query_swap_relocation"]["accuracy"], 1.0)


if __name__ == "__main__":
    unittest.main()
