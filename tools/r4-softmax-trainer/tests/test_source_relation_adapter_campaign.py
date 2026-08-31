"""Decision-bearing checks for C1-SB3 score aggregation."""

from __future__ import annotations

import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

import torch

from r4_softmax_trainer import source_relation_adapter_campaign as campaign
from r4_softmax_trainer.constants import FROZEN_MODEL_CONFIG
from r4_softmax_trainer.provenance import (
    atomic_write_json,
    canonical_json_bytes,
    cid_bytes,
    cid_file,
    write_bound_manifest,
)
from r4_softmax_trainer.source_relation_adapter import REPRESENTATION_UPDATE, RelationExample
from r4_softmax_trainer.source_relation_adapter_campaign import (
    _named_control_metrics,
    _record_metrics,
)


class AttendedRelationCampaignTests(unittest.TestCase):
    def test_scores_follow_unsorted_multi_record_encoded_order(self) -> None:
        relation_input = (
            "Evidence:\nThe amber dial is inside the oak case.\n"
            "Question:\nWhere is the amber dial?\nSupported:"
        )
        records = [
            {
                "record_cid": "record-b",
                "motif": "answer",
                "target_outcome": "answer",
                "answer": "The amber dial is inside the oak case.",
                "target_span_index": 0,
                "sentence_spans": [
                    {
                        "candidate_index": 0,
                        "byte_start": 0,
                        "text": "The amber dial is inside the oak case.",
                        "relation_label": 1,
                    },
                    {
                        "candidate_index": 1,
                        "byte_start": 45,
                        "text": "The bronze bell was polished yesterday.",
                        "relation_label": 0,
                    }
                ],
            },
            {
                "record_cid": "record-a",
                "motif": "abstain",
                "target_outcome": "abstain",
                "answer": "ABSTAIN",
                "target_span_index": None,
                "sentence_spans": [
                    {
                        "candidate_index": 0,
                        "byte_start": 0,
                        "text": "The amber dial was cleaned yesterday.",
                        "relation_label": 0,
                    },
                    {
                        "candidate_index": 1,
                        "byte_start": 46,
                        "text": "The bronze bell is beside the pine door.",
                        "relation_label": 0,
                    }
                ],
            },
        ]
        input_cid = cid_bytes(relation_input.encode("utf-8"))
        sorted_examples = [
            RelationExample("record-a", 0, relation_input, input_cid, 0),
            RelationExample("record-a", 1, relation_input, input_cid, 0),
            RelationExample("record-b", 0, relation_input, input_cid, 1),
            RelationExample("record-b", 1, relation_input, input_cid, 0),
        ]
        scores = [-4.0, -3.0, 4.0, -2.0]
        labels = [0, 0, 1, 0]
        metrics = _record_metrics(
            records,
            raw={
                "scores": scores,
                "labels": labels,
                "mean_binary_cross_entropy": float(
                    torch.nn.functional.binary_cross_entropy_with_logits(
                        torch.tensor(scores), torch.tensor(labels, dtype=torch.float32)
                    )
                ),
            },
            examples=sorted_examples,
            include_records=False,
        )
        self.assertEqual(metrics["positive_relation_recall"]["correct"], 1)
        self.assertEqual(metrics["negative_relation_specificity"]["correct"], 3)
        self.assertEqual(metrics["outcome"]["answer"]["correct"], 1)
        self.assertEqual(metrics["outcome"]["abstain"]["correct"], 1)

    def test_score_labels_and_candidate_identities_fail_closed(self) -> None:
        relation_input = "Evidence:\nA.\nQuestion:\nWhere is A?\nSupported:"
        input_cid = cid_bytes(relation_input.encode("utf-8"))
        example = RelationExample("record-a", 0, relation_input, input_cid, 1)
        raw = {
            "scores": [1.0],
            "labels": [0],
            "mean_binary_cross_entropy": 0.1,
        }
        with self.assertRaisesRegex(RuntimeError, "ordered examples"):
            _record_metrics([], raw=raw, examples=[example], include_records=False)

        duplicate_raw = {
            "scores": [1.0, 1.0],
            "labels": [1, 1],
            "mean_binary_cross_entropy": 0.1,
        }
        with self.assertRaisesRegex(ValueError, "not unique"):
            _record_metrics(
                [],
                raw=duplicate_raw,
                examples=[example, example],
                include_records=False,
            )

    def test_named_controls_use_their_own_record_scopes(self) -> None:
        def evaluation(
            motif: str,
            *,
            exact: bool,
            outcome: str,
            question: str,
            source: str = "source-a",
        ) -> dict[str, object]:
            return {
                "motif": motif,
                "record_exact": exact,
                "target_outcome": outcome,
                "lexical_world": "world-a",
                "source_cid": source,
                "question_cid": question,
            }

        metrics = {
            "record_evaluations": [
                evaluation(
                    "matched-primary-answer",
                    exact=True,
                    outcome="answer",
                    question="question-a",
                ),
                evaluation(
                    "matched-secondary-answer",
                    exact=True,
                    outcome="answer",
                    question="question-b",
                ),
                evaluation(
                    "exact-duplicate-agreement",
                    exact=True,
                    outcome="answer",
                    question="question-a",
                ),
                evaluation(
                    "duplicate-distinct-location-conflict",
                    exact=False,
                    outcome="conflict",
                    question="question-a",
                    source="source-conflict",
                ),
            ]
        }
        controls = _named_control_metrics(metrics, {"record_evaluations": []})
        self.assertIs(controls["same_source_query_relocation_exact"], True)
        self.assertIs(controls["duplicate_agreement_exact"], True)
        self.assertIs(controls["distinct_conflict_exact"], False)

    def test_positive_preflight_persists_checkpoint_artifact_and_bound_manifest(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            temporary_path = Path(temporary)
            root = temporary_path / "campaign"
            root.mkdir()
            predecessor = temporary_path / "predecessor"
            predecessor.mkdir()
            for name in (
                "attended-relation-dataset.json",
                "attended-relation-preflight.json",
                "attended-relation-census.json",
                "training-view-manifest.json",
                "run-contract.json",
                "preflight-started.json",
            ):
                atomic_write_json(root / name, {"fixture": name})

            config_bytes = canonical_json_bytes({"fixture": "config"})
            tokenizer_bytes = canonical_json_bytes({"fixture": "tokenizer"})
            weights_bytes = b"adapted-weights"
            config_cid = cid_bytes(config_bytes)
            tokenizer_cid = cid_bytes(tokenizer_bytes)
            weights_cid = cid_bytes(weights_bytes)
            predecessor_weights_cid = cid_bytes(b"predecessor-weights")

            result = {
                "terminal": "PASS_REPRESENTATION_TRANSFER_PREFLIGHT_AWAITING_RUST_PARITY",
                "result_cid": cid_bytes(b"positive-result"),
            }
            commitments = [cid_bytes(f"product-{index}".encode()) for index in range(4)]
            dataset = {
                "dataset_cid": cid_bytes(b"dataset"),
                "split_policy_cid": cid_bytes(b"split"),
                "product_probe_commitments": commitments,
            }
            preflight = {"preflight_cid": cid_bytes(b"preflight")}
            training_manifest = {"manifest_cid": cid_bytes(b"training-view")}
            run_contract = {"run_contract_cid": cid_bytes(b"run-contract")}
            predecessor_manifest = {
                "weights_cid": predecessor_weights_cid,
                "config_cid": config_cid,
                "tokenizer_cid": tokenizer_cid,
                "model_contract": FROZEN_MODEL_CONFIG.as_contract(),
            }

            def fake_export(
                _adapter: object,
                *,
                output_dir: Path,
                training_result: dict[str, object],
                **_kwargs: object,
            ) -> dict[str, object]:
                output_dir.mkdir()
                (output_dir / "config.json").write_bytes(config_bytes)
                (output_dir / "tokenizer.json").write_bytes(tokenizer_bytes)
                (output_dir / "model.safetensors").write_bytes(weights_bytes)
                atomic_write_json(output_dir / "training-result.json", training_result)
                return write_bound_manifest(
                    output_dir / "export-manifest.json",
                    {
                        "weights_cid": cid_file(output_dir / "model.safetensors"),
                        "config_cid": cid_file(output_dir / "config.json"),
                        "tokenizer_cid": cid_file(output_dir / "tokenizer.json"),
                        "training_result_cid": training_result["result_cid"],
                        "model_contract": FROZEN_MODEL_CONFIG.as_contract(),
                    },
                    artifact_root=output_dir,
                    relative_paths=[
                        "config.json",
                        "model.safetensors",
                        "tokenizer.json",
                        "training-result.json",
                    ],
                )

            with (
                patch.object(
                    campaign,
                    "export_merged_attended_relation_checkpoint",
                    side_effect=fake_export,
                ),
                patch.multiple(
                    campaign,
                    EXPECTED_CONFIG_CID=config_cid,
                    EXPECTED_TOKENIZER_CID=tokenizer_cid,
                    EXPECTED_WEIGHTS_CID=predecessor_weights_cid,
                ),
            ):
                manifest, delivery = campaign._finalize_preflight_result(
                    root,
                    predecessor=predecessor,
                    predecessor_manifest=predecessor_manifest,
                    adapter=object(),  # type: ignore[arg-type]
                    representation_update=REPRESENTATION_UPDATE,
                    result=result,
                    dataset=dataset,
                    preflight=preflight,
                    training_manifest=training_manifest,
                    run_contract=run_contract,
                )

            self.assertIsNotNone(delivery)
            assert delivery is not None
            artifact = delivery["artifact"]
            self.assertEqual(artifact["admission"], "research_only")
            self.assertEqual(artifact["representation_update"], REPRESENTATION_UPDATE)
            self.assertEqual(artifact["model_weights_cid"], weights_cid)
            self.assertNotEqual(
                artifact["model_weights_cid"], artifact["predecessor_model_weights_cid"]
            )
            unsigned = dict(artifact)
            observed_cid = unsigned.pop("artifact_cid")
            self.assertEqual(observed_cid, cid_bytes(canonical_json_bytes(unsigned)))
            self.assertTrue((root / "preflight-checkpoint/model.safetensors").is_file())
            self.assertTrue((root / "attended-relation-adapter.json").is_file())
            paths = {record["path"] for record in manifest["artifacts"]}
            self.assertIn("attended-relation-adapter.json", paths)
            self.assertIn("preflight-checkpoint/model.safetensors", paths)

    def test_wall_budget_result_uses_common_bound_manifest(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary) / "campaign"
            root.mkdir()
            for name in (
                "attended-relation-dataset.json",
                "attended-relation-preflight.json",
                "attended-relation-census.json",
                "training-view-manifest.json",
                "run-contract.json",
                "preflight-started.json",
            ):
                atomic_write_json(root / name, {"fixture": name})
            result = {
                "terminal": "UNAVAILABLE_PREFLIGHT_BUDGET",
                "result_cid": cid_bytes(b"budget-result"),
                "optimization": {
                    "stopped_after_step": 8,
                    "elapsed_seconds": 12.5,
                    "projected_seconds_at_eta_probe": 700.0,
                    "wall_ceiling_seconds": 600.0,
                },
            }
            dataset = {
                "dataset_cid": cid_bytes(b"dataset"),
                "product_probe_commitments": [
                    cid_bytes(f"product-{index}".encode()) for index in range(4)
                ],
            }
            manifest, delivery = campaign._finalize_preflight_result(
                root,
                predecessor=Path(temporary) / "unused-predecessor",
                predecessor_manifest={},
                adapter=None,
                representation_update=REPRESENTATION_UPDATE,
                result=result,
                dataset=dataset,
                preflight={"preflight_cid": cid_bytes(b"preflight")},
                training_manifest={"manifest_cid": cid_bytes(b"training-view")},
                run_contract={"run_contract_cid": cid_bytes(b"run-contract")},
            )
            self.assertIsNone(delivery)
            self.assertEqual(manifest["terminal"], "UNAVAILABLE_PREFLIGHT_BUDGET")
            persisted = (root / "preflight-result.json").read_text(encoding="utf-8")
            self.assertIn('"projected_seconds_at_eta_probe":700.0', persisted)
            self.assertTrue((root / "preflight-manifest.json").is_file())


if __name__ == "__main__":
    unittest.main()
