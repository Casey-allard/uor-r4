"""Dependency-light mechanism tests for R4SourceSpanPointerV1."""

from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path

import torch

from r4_softmax_trainer.constants import FROZEN_MODEL_CONFIG
from r4_softmax_trainer.provenance import canonical_json_bytes, cid_bytes
from r4_softmax_trainer.source_span_pointer import (
    EXPECTED_TOKENIZER_CID,
    EXPECTED_WEIGHTS_CID,
    HEAD_SCHEMA,
    OUTCOME_TO_ID,
    POLICY,
    PointerBatch,
    R4SourceSpanPointer,
    SourceSpanPointerConfig,
    _admit_rust_score_parity,
    _head_payload,
    pointer_loss,
    safety_decisions,
)


def synthetic_batch() -> PointerBatch:
    hidden = FROZEN_MODEL_CONFIG.hidden_size
    subjects = torch.zeros((1, 2, hidden), dtype=torch.float32)
    subjects[0, 0, 0] = 1.0
    candidates = torch.zeros((1, 2, 2, hidden), dtype=torch.float32)
    candidates[0, 0, 0, 1] = 1.0
    candidates[0, 1, 0, 0] = 1.0
    return PointerBatch(
        subject_states=subjects,
        subject_mask=torch.tensor([[True, False]]),
        candidate_states=candidates,
        candidate_token_mask=torch.tensor([[[True, False], [True, False]]]),
        candidate_mask=torch.ones((1, 2), dtype=torch.bool),
        outcomes=torch.tensor([OUTCOME_TO_ID["answer"]], dtype=torch.long),
        target_spans=torch.tensor([1], dtype=torch.long),
    )


class PointerMechanismTests(unittest.TestCase):
    def test_weighted_cosine_selects_matching_candidate(self) -> None:
        model = R4SourceSpanPointer()
        output = model(synthetic_batch())
        self.assertAlmostEqual(float(output.candidate_scores[0, 0]), 0.0, places=6)
        self.assertAlmostEqual(float(output.candidate_scores[0, 1]), 1.0, places=6)
        self.assertEqual(int(output.top_candidate_indices[0]), 1)
        self.assertEqual(int(safety_decisions(output.outcome_logits)[0]), OUTCOME_TO_ID["answer"])
        self.assertTrue(torch.isfinite(pointer_loss(output, synthetic_batch())))

    def test_safety_ties_choose_conflict_then_abstain_then_answer(self) -> None:
        logits = torch.tensor(
            [
                [0.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
                [2.0, 1.0, 0.0],
            ]
        )
        self.assertEqual(
            safety_decisions(logits).tolist(),
            [
                OUTCOME_TO_ID["conflict"],
                OUTCOME_TO_ID["abstain"],
                OUTCOME_TO_ID["answer"],
            ],
        )

    def test_serialized_head_has_exact_loader_contract_and_embedded_cid(self) -> None:
        model = R4SourceSpanPointer()
        dataset = {
            "dataset_cid": "blake3:" + "1" * 64,
            "split_policy_cid": "blake3:" + "2" * 64,
            "product_probe_commitments": ["blake3:" + "3" * 64],
        }
        head = _head_payload(
            model,
            dataset=dataset,
            run_contract_cid="blake3:" + "4" * 64,
            training_result_cid="blake3:" + "5" * 64,
            preflight={"status": "PASS"},
            development_metrics={"status": "PASS"},
        )
        expected = {
            "schema",
            "policy",
            "issue",
            "model_weights_cid",
            "tokenizer_cid",
            "hidden_size",
            "state_weights",
            "score_scale",
            "answer_bias",
            "abstain_bias",
            "conflict_bias",
            "maximum_source_spans",
            "question_policy",
            "sentence_policy",
            "dataset_cid",
            "split_policy_cid",
            "run_contract_cid",
            "training_result_cid",
            "preflight",
            "development_metrics",
            "product_probe_commitments",
            "artifact_cid",
        }
        self.assertEqual(set(head), expected)
        self.assertEqual(head["schema"], HEAD_SCHEMA)
        self.assertEqual(head["policy"], POLICY)
        self.assertEqual(len(head["state_weights"]), 288)
        self.assertTrue(all(value > 0.0 for value in head["state_weights"]))
        unsigned = dict(head)
        artifact_cid = unsigned.pop("artifact_cid")
        self.assertEqual(artifact_cid, cid_bytes(canonical_json_bytes(unsigned)))

    def test_config_refuses_sweeps(self) -> None:
        SourceSpanPointerConfig().validate()
        with self.assertRaises(ValueError):
            SourceSpanPointerConfig(optimizer_steps=255).validate()

    def test_admits_exact_public_grounded_answer_report_shape(self) -> None:
        artifact_cid = "blake3:" + "a" * 64
        source_cid = "blake3:" + "b" * 64
        fixture_cid = "blake3:" + "c" * 64
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            preflight = root / "preflight"
            preflight.mkdir()
            fixture = {
                "preflight_artifact_cid": artifact_cid,
                "source_cid": source_cid,
                "question": "Where is the blue book?",
                "candidate_scores": [0.25, 0.75],
                "logits": {"answer": 1.0, "abstain": 0.0, "conflict": -1.0},
                "decision": "answer",
                "selected_span_index": 1,
                "maximum_absolute_tolerance": 0.01,
                "fixture_cid": fixture_cid,
            }
            head = {"artifact_cid": artifact_cid}
            report = {
                "schema": "uor-r4.grounded-answer/2",
                "source": {"source_cid": source_cid},
                "question": fixture["question"],
                "pointer": {"artifact_cid": artifact_cid, "policy": POLICY},
                "pointer_evaluation": {
                    "candidate_scores": [0.2501, 0.7499],
                    "ranked_candidate_indices": [1, 0],
                    "logits": {"answer": 1.0001, "abstain": 0.0, "conflict": -1.0},
                    "decision": "answer",
                    "selected_span_index": 1,
                },
                "state_encoding": {
                    "model_weights_cid": EXPECTED_WEIGHTS_CID,
                    "tokenizer_cid": EXPECTED_TOKENIZER_CID,
                    "hidden_size": 288,
                    "checkpoint": {
                        "weights_cid": EXPECTED_WEIGHTS_CID,
                        "tokenizer_cid": EXPECTED_TOKENIZER_CID,
                    },
                    "model_shape": {"dimension": 288},
                },
            }
            (preflight / "python-score-fixture.json").write_text(
                json.dumps(fixture), encoding="utf-8"
            )
            (preflight / "preflight-head.json").write_text(
                json.dumps(head), encoding="utf-8"
            )
            report_path = root / "rust-parity.json"
            report_path.write_text(json.dumps(report), encoding="utf-8")

            admission = _admit_rust_score_parity(root, report_path)

            self.assertEqual(
                admission["terminal"], "PASS_PYTHON_RUST_SOURCE_SPAN_SCORE_PARITY"
            )
            self.assertLessEqual(admission["maximum_absolute_score_delta"], 0.01)
            self.assertLessEqual(admission["maximum_absolute_logit_delta"], 0.01)


if __name__ == "__main__":
    unittest.main()
