"""Focused create-once finalization test for frozen capacity issue #1019."""

from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path
from types import SimpleNamespace
from unittest import mock

from r4_softmax_trainer.capacity import (
    BASELINE_1017_NLL,
    BASELINE_1017_WEIGHTS_CID,
    REVEAL_OPENED_RELATIVE_PATH,
    REVEAL_OPENED_SCHEMA,
    REVEAL_MANIFEST_SCHEMA,
    REVEAL_RESULT_RELATIVE_PATH,
    REVEAL_RESULT_SCHEMA,
    _write_signed,
)
from r4_softmax_trainer.capacity_finalize import (
    FINAL_RESULT_RELATIVE_PATH,
    HUMAN_RUBRIC_CRITERION,
    HUMAN_RUBRIC_SCHEMA,
    finalize_capacity,
    verify_capacity_generation_ready,
    write_capacity_rubric_template,
)
from r4_softmax_trainer.capacity_data import (
    PREVIOUS_PROMPT_CIDS,
    SEALED_PROMPT_RELATIVE_PATH,
    SEALED_PROMPT_SCHEMA,
    TOKENIZER_CID,
)
from r4_softmax_trainer.provenance import canonical_json_bytes, cid_bytes


class CapacityFinalizerTests(unittest.TestCase):
    def test_generation_ready_binds_the_exact_signed_prompt_fixture(self) -> None:
        selection = {
            "manifest_cid": "blake3:" + "1" * 64,
            "dataset_manifest_cid": "blake3:" + "2" * 64,
            "weights_cid": "blake3:" + "3" * 64,
            "tokenizer_cid": TOKENIZER_CID,
        }
        parity = {"manifest_cid": "blake3:" + "4" * 64}
        token_ids = list(range(2, 26))
        prompts = [
            {
                "index": index,
                "story_cid": "blake3:" + f"{index + 1_000:064x}",
                "seed": 3019 + index,
                "prompt_tokens": 24,
                "prompt_token_ids": token_ids,
                "prompt_text": "prompt",
            }
            for index in range(5)
        ]
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            fixture = {
                "schema": SEALED_PROMPT_SCHEMA,
                "issue": 1019,
                "selection": (
                    "first 24 content tokens of the five lowest eligible test-story "
                    "CIDs strictly after the #1019 NLL-store boundary"
                ),
                "excluded_previous_prompt_cids": sorted(PREVIOUS_PROMPT_CIDS),
                "revealed_token_ids": 120,
                "tokenizer_cid": TOKENIZER_CID,
                "prompts": [
                    {
                        "story_cid": prompt["story_cid"],
                        "token_ids": prompt["prompt_token_ids"],
                        "text": prompt["prompt_text"],
                    }
                    for prompt in prompts
                ],
            }
            fixture["fixture_cid"] = cid_bytes(canonical_json_bytes(fixture))
            fixture_path = root / SEALED_PROMPT_RELATIVE_PATH
            fixture_path.parent.mkdir(parents=True, exist_ok=True)
            fixture_path.write_text(json.dumps(fixture), encoding="utf-8")
            marker = _write_signed(
                root / REVEAL_OPENED_RELATIVE_PATH,
                {
                    "schema": REVEAL_OPENED_SCHEMA,
                    "issue": 1019,
                    "selection_manifest_cid": selection["manifest_cid"],
                    "prefix_admission_manifest_cid": parity["manifest_cid"],
                    "candidate_weights_cid": selection["weights_cid"],
                    "baseline_1017_weights_cid": BASELINE_1017_WEIGHTS_CID,
                    "sealed_confirmation_status_before_marker": "UNOPENED",
                    "repeat_reveal_permitted": False,
                },
            )
            candidate_nll = 1.49
            baseline_nll = 1.57

            def write_reveal(prompt_records: list[dict[str, object]]) -> dict[str, object]:
                return _write_signed(
                    root / REVEAL_RESULT_RELATIVE_PATH,
                    {
                        "schema": REVEAL_RESULT_SCHEMA,
                        "issue": 1019,
                        "terminal": "PASS_CAPACITY_NLL_ADVANCE_GENERATION",
                        "selection_manifest_cid": selection["manifest_cid"],
                        "prefix_admission_manifest_cid": parity["manifest_cid"],
                        "reveal_opened_result_cid": marker["result_cid"],
                        "dataset_manifest_cid": selection["dataset_manifest_cid"],
                        "candidate_weights_cid": selection["weights_cid"],
                        "weights_cid": selection["weights_cid"],
                        "tokenizer_cid": selection["tokenizer_cid"],
                        "baseline_1017_weights_cid": BASELINE_1017_WEIGHTS_CID,
                        "candidate_enabled_sealed_nll": candidate_nll,
                        "baseline_1017_same_tranche_enabled_nll": baseline_nll,
                        "historical_1017_sealed_nll": BASELINE_1017_NLL,
                        "candidate_minus_baseline_same_tranche_nll": (
                            candidate_nll - baseline_nll
                        ),
                        "sealed_nll_ceiling": 1.50,
                        "absolute_nll_passed": True,
                        "relative_nll_passed": True,
                        "sealed_test_store_token_ids": 249_880,
                        "sealed_test_scored_next_tokens": 249_856,
                        "sealed_prompt_token_ids": 120,
                        "autonomous_generation_status": (
                            "NOT_RUN_RUST_SEEDED_SAMPLER_REQUIRED"
                        ),
                        "attention_off_executions": 0,
                        "prior_sealed_artifact_reads": 0,
                        "prompts": prompt_records,
                    },
                )

            reveal = write_reveal(prompts)
            manifest = {
                "schema": REVEAL_MANIFEST_SCHEMA,
                "issue": 1019,
                "manifest_cid": "blake3:" + "5" * 64,
                "terminal": "PASS_CAPACITY_NLL_ADVANCE_GENERATION",
                "selection_manifest_cid": selection["manifest_cid"],
                "prefix_admission_manifest_cid": parity["manifest_cid"],
                "reveal_opened_result_cid": marker["result_cid"],
                "reveal_result_cid": reveal["result_cid"],
                "candidate_enabled_sealed_nll": candidate_nll,
                "baseline_1017_same_tranche_enabled_nll": baseline_nll,
                "absolute_nll_passed": True,
                "relative_nll_passed": True,
                "attention_off_executions": 0,
                "artifacts": [
                    {"path": "reveal/capacity-opened.json"},
                    {"path": "reveal/capacity-reveal-result.json"},
                    {"path": "sealed-confirmation/test.u16"},
                    {"path": "sealed-confirmation/test-index.jsonl"},
                    {"path": "sealed-confirmation/prompts.json"},
                ],
            }
            tokenizer = mock.Mock()
            tokenizer.decode.return_value = "prompt"
            tokenizer.encode.return_value = SimpleNamespace(ids=token_ids)

            def patches() -> tuple[object, ...]:
                return (
                    mock.patch(
                        "r4_softmax_trainer.capacity_finalize.load_frozen_capacity_selection",
                        return_value=selection,
                    ),
                    mock.patch(
                        "r4_softmax_trainer.capacity_finalize.load_capacity_prefix_admission",
                        return_value=parity,
                    ),
                    mock.patch(
                        "r4_softmax_trainer.capacity_finalize.verify_bound_manifest",
                        return_value=manifest,
                    ),
                    mock.patch(
                        "r4_softmax_trainer.capacity_finalize.cid_file",
                        return_value=selection["tokenizer_cid"],
                    ),
                    mock.patch(
                        "r4_softmax_trainer.capacity_finalize.Tokenizer.from_file",
                        return_value=tokenizer,
                    ),
                )

            ready_patches = patches()
            with (
                ready_patches[0],
                ready_patches[1],
                ready_patches[2],
                ready_patches[3],
                ready_patches[4],
            ):
                ready = verify_capacity_generation_ready(root)
            self.assertEqual(ready["status"], "READY_FOR_IRREVERSIBLE_GENERATION")
            self.assertEqual(ready["prompt_count"], 5)

            mismatched_prompts = json.loads(json.dumps(prompts))
            mismatched_prompts[0]["story_cid"] = "blake3:" + "f" * 64
            reveal = write_reveal(mismatched_prompts)
            manifest["reveal_result_cid"] = reveal["result_cid"]
            mismatch_patches = patches()
            with (
                mismatch_patches[0],
                mismatch_patches[1],
                mismatch_patches[2],
                mismatch_patches[3],
                mismatch_patches[4],
            ):
                with self.assertRaisesRegex(ValueError, "reveal prompt 0 differs"):
                    verify_capacity_generation_ready(root)

    def test_rubric_and_finalization_reject_a_negative_nll_reveal(self) -> None:
        selection = {
            "manifest_cid": "blake3:" + "1" * 64,
            "dataset_manifest_cid": "blake3:" + "5" * 64,
            "weights_cid": "blake3:" + "2" * 64,
            "tokenizer_cid": "blake3:" + "3" * 64,
        }
        parity = {"manifest_cid": "blake3:" + "4" * 64}
        prompts = [
            {
                "index": index,
                "seed": 3019 + index,
                "prompt_tokens": 24,
                "prompt_text": "prompt",
                "prompt_token_ids": list(range(24)),
                "story_cid": "blake3:" + f"{index + 100:064x}",
            }
            for index in range(5)
        ]
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            fixture = {
                "schema": SEALED_PROMPT_SCHEMA,
                "issue": 1019,
                "selection": (
                    "first 24 content tokens of the five lowest eligible test-story "
                    "CIDs strictly after the #1019 NLL-store boundary"
                ),
                "excluded_previous_prompt_cids": sorted(PREVIOUS_PROMPT_CIDS),
                "revealed_token_ids": 120,
                "tokenizer_cid": TOKENIZER_CID,
                "prompts": [
                    {
                        "story_cid": prompt["story_cid"],
                        "token_ids": prompt["prompt_token_ids"],
                        "text": prompt["prompt_text"],
                    }
                    for prompt in prompts
                ],
            }
            fixture["fixture_cid"] = cid_bytes(canonical_json_bytes(fixture))
            fixture_path = root / SEALED_PROMPT_RELATIVE_PATH
            fixture_path.parent.mkdir(parents=True, exist_ok=True)
            fixture_path.write_text(json.dumps(fixture), encoding="utf-8")
            marker = _write_signed(
                root / REVEAL_OPENED_RELATIVE_PATH,
                {
                    "schema": REVEAL_OPENED_SCHEMA,
                    "issue": 1019,
                    "selection_manifest_cid": selection["manifest_cid"],
                    "prefix_admission_manifest_cid": parity["manifest_cid"],
                    "candidate_weights_cid": selection["weights_cid"],
                    "baseline_1017_weights_cid": BASELINE_1017_WEIGHTS_CID,
                    "sealed_confirmation_status_before_marker": "UNOPENED",
                    "repeat_reveal_permitted": False,
                },
            )
            candidate_nll = 1.60
            baseline_nll = 1.57
            reveal = _write_signed(
                root / REVEAL_RESULT_RELATIVE_PATH,
                {
                    "schema": REVEAL_RESULT_SCHEMA,
                    "issue": 1019,
                    "terminal": "FAIL_CAPACITY_NLL",
                    "selection_manifest_cid": selection["manifest_cid"],
                    "prefix_admission_manifest_cid": parity["manifest_cid"],
                    "reveal_opened_result_cid": marker["result_cid"],
                    "dataset_manifest_cid": selection["dataset_manifest_cid"],
                    "candidate_weights_cid": selection["weights_cid"],
                    "weights_cid": selection["weights_cid"],
                    "tokenizer_cid": selection["tokenizer_cid"],
                    "baseline_1017_weights_cid": BASELINE_1017_WEIGHTS_CID,
                    "candidate_enabled_sealed_nll": candidate_nll,
                    "baseline_1017_same_tranche_enabled_nll": baseline_nll,
                    "historical_1017_sealed_nll": BASELINE_1017_NLL,
                    "candidate_minus_baseline_same_tranche_nll": (
                        candidate_nll - baseline_nll
                    ),
                    "sealed_nll_ceiling": 1.50,
                    "absolute_nll_passed": False,
                    "relative_nll_passed": False,
                    "sealed_test_store_token_ids": 249_880,
                    "sealed_test_scored_next_tokens": 249_856,
                    "sealed_prompt_token_ids": 120,
                    "autonomous_generation_status": (
                        "NOT_RUN_RUST_SEEDED_SAMPLER_REQUIRED"
                    ),
                    "attention_off_executions": 0,
                    "prior_sealed_artifact_reads": 0,
                    "prompts": prompts,
                },
            )
            manifest = {
                "schema": REVEAL_MANIFEST_SCHEMA,
                "issue": 1019,
                "terminal": "FAIL_CAPACITY_NLL",
                "selection_manifest_cid": selection["manifest_cid"],
                "prefix_admission_manifest_cid": parity["manifest_cid"],
                "reveal_opened_result_cid": marker["result_cid"],
                "reveal_result_cid": reveal["result_cid"],
                "candidate_enabled_sealed_nll": candidate_nll,
                "baseline_1017_same_tranche_enabled_nll": baseline_nll,
                "absolute_nll_passed": False,
                "relative_nll_passed": False,
                "attention_off_executions": 0,
                "artifacts": [
                    {"path": "reveal/capacity-opened.json"},
                    {"path": "reveal/capacity-reveal-result.json"},
                    {"path": "sealed-confirmation/test.u16"},
                    {"path": "sealed-confirmation/test-index.jsonl"},
                    {"path": "sealed-confirmation/prompts.json"},
                ],
            }
            tokenizer = mock.Mock()
            tokenizer.decode.return_value = "prompt"
            tokenizer.encode.return_value = SimpleNamespace(ids=list(range(24)))

            def patches() -> tuple[object, ...]:
                return (
                    mock.patch(
                        "r4_softmax_trainer.capacity_finalize.load_frozen_capacity_selection",
                        return_value=selection,
                    ),
                    mock.patch(
                        "r4_softmax_trainer.capacity_finalize.load_capacity_prefix_admission",
                        return_value=parity,
                    ),
                    mock.patch(
                        "r4_softmax_trainer.capacity_finalize.verify_bound_manifest",
                        return_value=manifest,
                    ),
                    mock.patch(
                        "r4_softmax_trainer.capacity_finalize.cid_file",
                        return_value=selection["tokenizer_cid"],
                    ),
                    mock.patch(
                        "r4_softmax_trainer.capacity_finalize.Tokenizer.from_file",
                        return_value=tokenizer,
                    ),
                )

            rubric_output = root / "review/rubric.json"
            rubric_patches = patches()
            with (
                rubric_patches[0],
                rubric_patches[1],
                rubric_patches[2],
                rubric_patches[3],
                rubric_patches[4],
            ):
                with self.assertRaisesRegex(ValueError, "forbidden after a negative NLL"):
                    write_capacity_rubric_template(root, rubric_output)
            self.assertFalse(rubric_output.exists())

            rubric_input = root / "outside-rubric.json"
            finalize_patches = patches()
            with (
                finalize_patches[0],
                finalize_patches[1],
                finalize_patches[2],
                finalize_patches[3],
                finalize_patches[4],
            ):
                with self.assertRaisesRegex(ValueError, "forbidden after a negative NLL"):
                    finalize_capacity(root, rubric_input)
            self.assertFalse((root / FINAL_RESULT_RELATIVE_PATH).exists())

    def test_rubric_template_binds_validated_generation_pairs_before_review(self) -> None:
        pairs = [
            {
                "index": index,
                "story_cid": "blake3:" + f"{index + 1:064x}",
                "seed": 3019 + index,
                "response_text": f"response {index}",
            }
            for index in range(5)
        ]
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            output = root / "review/rubric.json"
            with (
                mock.patch(
                    "r4_softmax_trainer.capacity_finalize._load_capacity_reveal",
                    return_value=({}, {"prompts": []}),
                ),
                mock.patch(
                    "r4_softmax_trainer.capacity_finalize._load_generation_pairs",
                    return_value=pairs,
                ),
            ):
                result = write_capacity_rubric_template(root, output)
            template = json.loads(output.read_text(encoding="utf-8"))
            self.assertEqual(result["records"], 5)
            self.assertEqual(template["schema"], HUMAN_RUBRIC_SCHEMA)
            self.assertEqual(
                template["records"][0]["decision"],
                "REVIEW_REPLACE_WITH_PASS_OR_FAIL",
            )
            self.assertEqual(template["records"][4]["seed"], 3023)
            with self.assertRaisesRegex(FileExistsError, "create-once"):
                write_capacity_rubric_template(root, output)

    def test_positive_result_is_bound_once_without_model_execution(self) -> None:
        reveal = {
            "selection_manifest_cid": "blake3:" + "1" * 64,
            "result_cid": "blake3:" + "2" * 64,
            "weights_cid": "blake3:" + "3" * 64,
            "tokenizer_cid": "blake3:" + "4" * 64,
            "candidate_enabled_sealed_nll": 1.49,
            "baseline_1017_same_tranche_enabled_nll": 1.57,
            "absolute_nll_passed": True,
            "relative_nll_passed": True,
        }
        manifest = {"manifest_cid": "blake3:" + "5" * 64}
        pairs = [
            {
                "index": index,
                "story_cid": "blake3:" + f"{index + 6:064x}",
                "seed": 3019 + index,
                "response_text": f"response {index}",
                "nonlooping": True,
            }
            for index in range(5)
        ]
        rubric = {
            "schema": HUMAN_RUBRIC_SCHEMA,
            "issue": 1019,
            "criterion": HUMAN_RUBRIC_CRITERION,
            "records": [],
        }
        validated = [
            {"decision": "PASS", "reason": "retains subject"} for _ in range(5)
        ]
        rubric_bytes = json.dumps(rubric).encode("utf-8")
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            rubric_path = root / "outside-rubric.json"
            rubric_path.write_bytes(rubric_bytes)
            with (
                mock.patch(
                    "r4_softmax_trainer.capacity_finalize._load_capacity_reveal",
                    return_value=(manifest, reveal),
                ),
                mock.patch(
                    "r4_softmax_trainer.capacity_finalize._load_generation_pairs",
                    return_value=pairs,
                ),
                mock.patch(
                    "r4_softmax_trainer.capacity_finalize._load_json_object_bytes",
                    return_value=(rubric_bytes, rubric),
                ),
                mock.patch(
                    "r4_softmax_trainer.capacity_finalize._validate_rubric",
                    return_value=validated,
                ),
                mock.patch(
                    "r4_softmax_trainer.capacity_finalize.write_bound_manifest",
                    side_effect=lambda _path, payload, **_kwargs: payload,
                ),
            ):
                final = finalize_capacity(root, rubric_path)
            self.assertEqual(final["terminal"], "PASS_CAPACITY_QUALITY_BASELINE")
            result = json.loads((root / FINAL_RESULT_RELATIVE_PATH).read_text(encoding="utf-8"))
            self.assertTrue(result["definition_of_done"]["overall_pass"])
            with self.assertRaisesRegex(FileExistsError, "create-once"):
                finalize_capacity(root, rubric_path)


if __name__ == "__main__":
    unittest.main()
