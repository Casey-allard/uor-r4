"""Focused preparation, aggregation, and product-isolation tests for C1-SB4."""

from __future__ import annotations

import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

import torch

from r4_softmax_trainer import joint_candidate_margin_campaign as campaign
from r4_softmax_trainer.joint_candidate_margin_data import (
    build_joint_candidate_margin_population,
    render_joint_candidate_input,
)
from r4_softmax_trainer.provenance import canonical_json_bytes, cid_bytes


def _with_cid(value: dict[str, object], field: str) -> dict[str, object]:
    result = dict(value)
    result[field] = cid_bytes(canonical_json_bytes(value))
    return result


def _fake_tokenizer_census() -> dict[str, object]:
    partitions: dict[str, object] = {}
    for name, frozen in campaign.EXPECTED_TOKENIZER_PARTITIONS.items():
        maximum = int(frozen["maximum_positions_including_bos"])
        partitions[name] = {
            "records": int(frozen["records"]),
            "groups": int(frozen["groups"]),
            "minimum_content_tokens": 1,
            "maximum_content_tokens": maximum - 1,
            "minimum_positions_including_bos": 2,
            "maximum_positions_including_bos": maximum,
            "context_ceiling_including_bos": 256,
            "terminal_token": "standalone colon",
            "truncation": "FORBIDDEN_NOT_USED",
            "passed": True,
        }
    return _with_cid(
        {
            "schema": campaign.TOKENIZER_CENSUS_SCHEMA,
            "policy": campaign.POLICY,
            "issue": 954,
            "tokenizer_cid": campaign.EXPECTED_TOKENIZER_CID,
            "input_policy": campaign.JOINT_INPUT_POLICY,
            "partitions": partitions,
            "all_prompts_end_at_standalone_colon": True,
            "no_prompt_truncated": True,
            "passed": True,
        },
        "tokenizer_census_cid",
    )


def _raw_exact(records: list[dict[str, object]]) -> dict[str, object]:
    evaluations = []
    total_margin = 0.0
    for record in records:
        groups: dict[str, dict[str, object]] = {}
        for span in record["sentence_spans"]:  # type: ignore[index]
            group_cid = str(span["relation_group_cid"])
            group = groups.setdefault(
                group_cid,
                {
                    "relation_group_cid": group_cid,
                    "text": str(span["text"]),
                    "relation_label": int(span["relation_label"]),
                    "occurrence_indices": [],
                },
            )
            group["occurrence_indices"].append(int(span["candidate_index"]))  # type: ignore[union-attr]
        rows = []
        for group in groups.values():
            row = dict(group)
            row["score"] = 2.0 if row["relation_label"] == 1 else -2.0
            rows.append(row)
        evaluations.append(
            {
                "record_id": record["record_cid"],
                "source_width": record["source_width"],
                "target_outcome": record["target_outcome"],
                "structured_margin": 0.0,
                "group_scores": rows,
            }
        )
    return {
        "records": len(records),
        "groups": sum(len(row["group_scores"]) for row in evaluations),
        "mean_structured_margin": total_margin,
        "record_evaluations": list(reversed(evaluations)),
    }


def _mock_metrics(records: list[dict[str, object]], *, exact: bool) -> dict[str, object]:
    details = [
        {
            "record_id": record.get("record_id", record.get("record_cid")),
            "lexical_world": record["lexical_world"],
            "motif": record["motif"],
            "target_outcome": record["target_outcome"],
            "record_exact": exact or index > 0,
        }
        for index, record in enumerate(records)
    ]
    correct = len(records) if exact else len(records) - 1
    return {
        "records": len(records),
        "record_exact": {
            "correct": correct,
            "total": len(records),
            "accuracy": correct / len(records),
        },
        "record_evaluations": details,
    }


class JointCandidateMarginCampaignTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.dataset, cls.preflight, cls.products = (
            build_joint_candidate_margin_population()
        )

    def test_metrics_align_by_record_and_group_identity(self) -> None:
        records = list(self.preflight["sealed"][:9])
        metrics = campaign._record_metrics(
            records, _raw_exact(records), include_records=True
        )
        self.assertTrue(campaign._all_exact(metrics))
        self.assertEqual(metrics["record_exact"]["correct"], 9)
        self.assertEqual(
            metrics["positive_group_recall"]["correct"],
            metrics["positive_group_recall"]["total"],
        )
        self.assertEqual(
            metrics["negative_group_specificity"]["correct"],
            metrics["negative_group_specificity"]["total"],
        )

    def test_full_source_reversal_rebuilds_every_bound_field(self) -> None:
        reversed_records, scope = campaign._reversed_records(
            list(self.preflight["sealed"])
        )
        self.assertEqual(
            scope,
            {
                "records": 63,
                "nontrivial_reversals": 62,
                "byte_identical_reversals": 1,
            },
        )
        for record in reversed_records:
            parsed = campaign.split_sentence_spans(record["source"])
            self.assertEqual(len(parsed), record["source_width"])
            for index, (parsed_span, span) in enumerate(
                zip(parsed, record["sentence_spans"])
            ):
                self.assertEqual(span["candidate_index"], index)
                self.assertEqual(span["byte_start"], parsed_span["byte_start"])
                self.assertEqual(span["byte_end"], parsed_span["byte_end"])
                expected = render_joint_candidate_input(
                    record["source"], record["question"], span["text"]
                )
                self.assertEqual(span["relation_input"], expected)
                self.assertEqual(
                    span["relation_input_cid"], cid_bytes(expected.encode("utf-8"))
                )

    def test_prepare_binds_product_outside_training_view(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            base = Path(temporary)
            root = base / "run"
            predecessor = base / "predecessor"
            predecessor.mkdir()
            fake_census = _fake_tokenizer_census()
            predecessor_manifest = {
                "manifest_cid": campaign.EXPECTED_PREDECESSOR_MANIFEST_CID,
                "model_contract": campaign.FROZEN_MODEL_CONFIG.as_contract(),
            }
            with (
                patch.object(campaign, "_validated_predecessor", return_value=predecessor_manifest),
                patch.object(campaign.Tokenizer, "from_file", return_value=object()),
                patch.object(campaign, "_build_tokenizer_census", return_value=fake_census),
                patch.object(
                    campaign,
                    "EXPECTED_TOKENIZER_CENSUS_CID",
                    fake_census["tokenizer_census_cid"],
                ),
            ):
                result = campaign.prepare_joint_candidate_margin_data(
                    root, predecessor=predecessor
                )
                self.assertEqual(
                    result["terminal"],
                    "JOINT_CANDIDATE_MARGIN_DATA_COMMITTED_NO_TRAINING",
                )
                _, _, census, manifest = campaign._load_training_view(root)
            self.assertTrue(census["passed"])
            paths = {record["path"] for record in manifest["artifacts"]}
            self.assertNotIn(campaign.PRODUCT_FILE, paths)
            self.assertNotIn(campaign.PRODUCT_MANIFEST_FILE, paths)
            self.assertTrue((root / campaign.PRODUCT_FILE).is_file())
            with (
                patch.object(
                    campaign,
                    "EXPECTED_TOKENIZER_CENSUS_CID",
                    fake_census["tokenizer_census_cid"],
                ),
                patch.object(
                    campaign, "EXPECTED_DATASET_CID", "blake3:" + "0" * 64
                ),
                self.assertRaisesRegex(ValueError, "frozen campaign"),
            ):
                campaign._load_training_view(root)

    def test_negative_run_never_rebuilds_or_reads_product(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            base = Path(temporary)
            root = base / "run"
            predecessor = base / "predecessor"
            predecessor.mkdir()
            predecessor_manifest = {
                "manifest_cid": campaign.EXPECTED_PREDECESSOR_MANIFEST_CID,
                "model_contract": campaign.FROZEN_MODEL_CONFIG.as_contract(),
            }
            fake_census = _fake_tokenizer_census()
            with (
                patch.object(campaign, "_validated_predecessor", return_value=predecessor_manifest),
                patch.object(campaign.Tokenizer, "from_file", return_value=object()),
                patch.object(campaign, "_build_tokenizer_census", return_value=fake_census),
                patch.object(
                    campaign,
                    "EXPECTED_TOKENIZER_CENSUS_CID",
                    fake_census["tokenizer_census_cid"],
                ),
            ):
                campaign.prepare_joint_candidate_margin_data(
                    root, predecessor=predecessor
                )

            original_read_text = Path.read_text

            def guarded_read_text(path: Path, *args: object, **kwargs: object) -> str:
                if path.name == campaign.PRODUCT_FILE:
                    raise AssertionError("optimizer attempted to read sealed product text")
                return original_read_text(path, *args, **kwargs)

            calls = {"evaluation": 0}

            def fake_evaluate(
                _adapter: object,
                records: list[dict[str, object]],
                **_kwargs: object,
            ) -> dict[str, object]:
                calls["evaluation"] += 1
                # Untouched sealed misses; fit is exact; trained sealed retains one miss.
                return _mock_metrics(
                    records,
                    exact=calls["evaluation"] == 2,
                )

            class FakeAdapter:
                def to(self, _device: torch.device) -> "FakeAdapter":
                    return self

            partition_by_count = {
                126: fake_census["partitions"]["fit"],  # type: ignore[index]
                63: fake_census["partitions"]["sealed"],  # type: ignore[index]
            }
            with (
                patch.object(Path, "read_text", guarded_read_text),
                patch.object(
                    campaign,
                    "build_joint_candidate_margin_population",
                    side_effect=AssertionError("optimizer rebuilt sealed products"),
                ),
                patch.object(
                    campaign,
                    "EXPECTED_TOKENIZER_CENSUS_CID",
                    fake_census["tokenizer_census_cid"],
                ),
                patch.object(campaign, "_validated_predecessor", return_value=predecessor_manifest),
                patch.object(campaign, "require_mps", return_value=torch.device("cpu")),
                patch.object(campaign.Tokenizer, "from_file", return_value=object()),
                patch.object(campaign, "validate_tokenizer_contract"),
                patch.object(
                    campaign,
                    "_tokenizer_partition",
                    side_effect=lambda records, _tokenizer: partition_by_count[len(records)],
                ),
                patch.object(campaign, "_load_base_model", return_value=(object(), {})),
                patch.object(campaign, "R4JointCandidateMarginAdapter", return_value=FakeAdapter()),
                patch.object(campaign, "_evaluate_records", side_effect=fake_evaluate),
                patch.object(
                    campaign,
                    "EncodedJointCandidateMarginDataset",
                    return_value=object(),
                ),
                patch.object(
                    campaign,
                    "fit_joint_candidate_margin_adapter",
                    return_value={
                        "optimizer_steps": 270,
                        "records_per_step": 7,
                        "initial_structured_margin": 3.0,
                        "final_structured_margin": 0.5,
                    },
                ),
                patch.object(
                    campaign,
                    "_delta_contract",
                    return_value={"passed": True},
                ),
            ):
                result = campaign.run_joint_candidate_margin_preflight(
                    root, predecessor=predecessor
                )
            self.assertEqual(
                result["terminal"], "FAIL_JOINT_CANDIDATE_MARGIN_PREFLIGHT"
            )
            self.assertEqual(result["product"], "UNOPENED_NOT_RUN")

    def test_sealed_exact_base_selects_frozen_readout_even_if_fit_misses(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            base = Path(temporary)
            root = base / "run"
            predecessor = base / "predecessor"
            predecessor.mkdir()
            predecessor_manifest = {
                "manifest_cid": campaign.EXPECTED_PREDECESSOR_MANIFEST_CID,
                "model_contract": campaign.FROZEN_MODEL_CONFIG.as_contract(),
            }
            fake_census = _fake_tokenizer_census()
            token_census_cid = fake_census["tokenizer_census_cid"]
            with (
                patch.object(
                    campaign,
                    "_validated_predecessor",
                    return_value=predecessor_manifest,
                ),
                patch.object(campaign.Tokenizer, "from_file", return_value=object()),
                patch.object(
                    campaign,
                    "_build_tokenizer_census",
                    return_value=fake_census,
                ),
                patch.object(
                    campaign, "EXPECTED_TOKENIZER_CENSUS_CID", token_census_cid
                ),
            ):
                campaign.prepare_joint_candidate_margin_data(
                    root, predecessor=predecessor
                )

            calls = {"evaluation": 0}

            def fake_evaluate(
                _adapter: object,
                records: list[dict[str, object]],
                **_kwargs: object,
            ) -> dict[str, object]:
                calls["evaluation"] += 1
                # Frozen sealed, diagnostic fit, then reversed sealed.
                exact = calls["evaluation"] != 2
                return _mock_metrics(records, exact=exact)

            class FakeAdapter:
                def to(self, _device: torch.device) -> "FakeAdapter":
                    return self

            partition_by_count = {
                126: fake_census["partitions"]["fit"],  # type: ignore[index]
                63: fake_census["partitions"]["sealed"],  # type: ignore[index]
            }
            with (
                patch.object(
                    campaign, "EXPECTED_TOKENIZER_CENSUS_CID", token_census_cid
                ),
                patch.object(
                    campaign,
                    "_validated_predecessor",
                    return_value=predecessor_manifest,
                ),
                patch.object(campaign, "require_mps", return_value=torch.device("cpu")),
                patch.object(campaign.Tokenizer, "from_file", return_value=object()),
                patch.object(campaign, "validate_tokenizer_contract"),
                patch.object(
                    campaign,
                    "_tokenizer_partition",
                    side_effect=lambda records, _tokenizer: partition_by_count[
                        len(records)
                    ],
                ),
                patch.object(campaign, "_load_base_model", return_value=(object(), {})),
                patch.object(
                    campaign,
                    "R4JointCandidateMarginAdapter",
                    return_value=FakeAdapter(),
                ),
                patch.object(campaign, "_evaluate_records", side_effect=fake_evaluate),
                patch.object(
                    campaign,
                    "fit_joint_candidate_margin_adapter",
                    side_effect=AssertionError("frozen branch started optimization"),
                ),
                patch.object(
                    campaign,
                    "_delta_contract",
                    return_value={"passed": True},
                ),
                patch.object(
                    campaign,
                    "_finalize_result",
                    return_value=({"manifest_cid": "blake3:" + "d" * 64}, None),
                ),
            ):
                result = campaign.run_joint_candidate_margin_preflight(
                    root, predecessor=predecessor
                )
            self.assertEqual(
                result["terminal"],
                "PASS_FROZEN_JOINT_SOURCE_READOUT_AWAITING_RUST_PARITY",
            )
            self.assertEqual(calls["evaluation"], 3)


if __name__ == "__main__":
    unittest.main()
