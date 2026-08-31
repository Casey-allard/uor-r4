"""Focused tests for the create-once #1017 final evidence binder."""

from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path
from unittest import mock

from r4_softmax_trainer.finalize import (
    FINAL_RESULT_RELATIVE_PATH,
    HUMAN_RUBRIC_CRITERION,
    HUMAN_RUBRIC_SCHEMA,
    RUBRIC_INPUT_RELATIVE_PATH,
    _normalized_generation_report,
    _terminal,
    _validate_rubric,
    _verify_exact_generation_files,
    finalize_continuation,
)
from r4_softmax_trainer.provenance import canonical_json_bytes, cid_bytes


def _cid(character: str) -> str:
    return "blake3:" + character * 64


def _pairs() -> list[dict[str, object]]:
    return [
        {
            "index": index,
            "story_cid": f"blake3:{index + 1:064x}",
            "seed": 2014 + index,
            "prompt_text": f"prompt {index}",
            "response_text": f"response {index}",
            "generated_tokens": 16,
            "stop_reason": "maximum_new_tokens",
            "nonlooping": True,
            "primary_path": f"generations/prompt-{index}-seed-{2014 + index}.json",
            "primary_report_cid": _cid("1"),
            "replay_path": (
                f"generations/replay/prompt-{index}-seed-{2014 + index}.json"
            ),
            "replay_report_cid": _cid("2"),
            "core_cids": {
                "decision_cid": _cid("3"),
                "generation_policy_cid": _cid("4"),
                "output_cid": _cid("5"),
                "audit_cid": _cid("6"),
            },
        }
        for index in range(5)
    ]


def _rubric(*, decisions: list[str] | None = None) -> dict[str, object]:
    pairs = _pairs()
    decisions = decisions or ["PASS"] * 5
    return {
        "schema": HUMAN_RUBRIC_SCHEMA,
        "issue": 1017,
        "criterion": HUMAN_RUBRIC_CRITERION,
        "records": [
            {
                "index": index,
                "story_cid": pair["story_cid"],
                "seed": pair["seed"],
                "decision": decisions[index],
                "response_text": pair["response_text"],
                "reason": f"bounded reason {index}",
            }
            for index, pair in enumerate(pairs)
        ],
    }


class GenerationInventoryTests(unittest.TestCase):
    def test_exact_five_primary_and_five_replays_are_required(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            for index in range(5):
                seed = 2014 + index
                primary = root / "generations" / f"prompt-{index}-seed-{seed}.json"
                replay = (
                    root
                    / "generations"
                    / "replay"
                    / f"prompt-{index}-seed-{seed}.json"
                )
                primary.parent.mkdir(parents=True, exist_ok=True)
                replay.parent.mkdir(parents=True, exist_ok=True)
                primary.write_text("{}", encoding="utf-8")
                replay.write_text("{}", encoding="utf-8")
            _verify_exact_generation_files(root)
            (root / "generations" / "extra.json").write_text("{}", encoding="utf-8")
            with self.assertRaisesRegex(ValueError, "exactly five primary"):
                _verify_exact_generation_files(root)

    def test_replay_normalization_removes_timing_only(self) -> None:
        primary = {
            "decision_cid": _cid("1"),
            "output": {"tokens": [1, 2, 3]},
            "timing": {"total_seconds": 1.0},
        }
        replay = {
            **primary,
            "timing": {"total_seconds": 99.0},
        }
        self.assertEqual(
            _normalized_generation_report(primary),
            _normalized_generation_report(replay),
        )
        replay["output"] = {"tokens": [1, 2, 4]}
        self.assertNotEqual(
            _normalized_generation_report(primary),
            _normalized_generation_report(replay),
        )


class HumanRubricTests(unittest.TestCase):
    def test_rubric_binds_all_five_exact_responses(self) -> None:
        records = _validate_rubric(_rubric(), generation_pairs=_pairs())
        self.assertEqual([record["decision"] for record in records], ["PASS"] * 5)

    def test_rubric_rejects_response_or_decision_drift(self) -> None:
        rubric = _rubric()
        rubric["records"][2]["response_text"] = "different output"
        with self.assertRaisesRegex(ValueError, "does not bind its rollout"):
            _validate_rubric(rubric, generation_pairs=_pairs())
        rubric = _rubric()
        rubric["records"][2]["decision"] = "MAYBE"
        with self.assertRaisesRegex(ValueError, "does not bind its rollout"):
            _validate_rubric(rubric, generation_pairs=_pairs())

    def test_terminal_names_each_failed_gate(self) -> None:
        self.assertEqual(_terminal([]), "PASS_COHERENT_R4_SOFTMAX_GENERATION")
        self.assertEqual(_terminal(["ENABLED_NLL"]), "FAIL_ENABLED_NLL")
        self.assertEqual(
            _terminal(["ENABLED_NLL", "GENERATION_RUBRIC"]),
            "FAIL_ENABLED_NLL_AND_GENERATION_RUBRIC",
        )


class CreateOnceFinalizerTests(unittest.TestCase):
    def test_failed_nll_writes_one_bound_negative_result_without_execution(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            rubric_path = root / "rubric-input.json"
            rubric_path.write_bytes(canonical_json_bytes(_rubric()))
            reveal = {
                "selection_manifest_cid": _cid("1"),
                "result_cid": _cid("2"),
                "selected_checkpoint_cid": _cid("3"),
                "weights_cid": _cid("4"),
                "tokenizer_cid": _cid("5"),
                "enabled_sealed_test_loss": 1.5727521962806827,
                "sealed_test_loss_passed": False,
            }
            reveal_manifest = {"manifest_cid": _cid("6")}

            def fake_manifest(
                _path: Path,
                payload: dict[str, object],
                *,
                artifact_root: Path,
                relative_paths: object,
            ) -> dict[str, object]:
                del artifact_root, relative_paths
                return payload

            with (
                mock.patch(
                    "r4_softmax_trainer.finalize._load_reveal",
                    return_value=(reveal_manifest, reveal),
                ) as load_reveal,
                mock.patch(
                    "r4_softmax_trainer.finalize._load_generation_pairs",
                    return_value=_pairs(),
                ) as load_pairs,
                mock.patch(
                    "r4_softmax_trainer.finalize.write_bound_manifest",
                    side_effect=fake_manifest,
                ),
            ):
                manifest = finalize_continuation(root, rubric_path)
            self.assertEqual(manifest["terminal"], "FAIL_ENABLED_NLL")
            self.assertEqual(manifest["generation_executions_by_finalizer"], 0)
            self.assertEqual(manifest["reveal_executions_by_finalizer"], 0)
            load_reveal.assert_called_once()
            load_pairs.assert_called_once()
            result = json.loads((root / FINAL_RESULT_RELATIVE_PATH).read_text())
            self.assertEqual(result["terminal"], "FAIL_ENABLED_NLL")
            self.assertFalse(result["definition_of_done"]["overall_pass"])
            self.assertEqual(result["rubric_pass_count"], 5)
            unsigned = dict(result)
            result_cid = unsigned.pop("result_cid")
            self.assertEqual(result_cid, cid_bytes(canonical_json_bytes(unsigned)))
            self.assertEqual(
                (root / RUBRIC_INPUT_RELATIVE_PATH).read_bytes(), rubric_path.read_bytes()
            )
            with self.assertRaisesRegex(FileExistsError, "already present"):
                finalize_continuation(root, rubric_path)


if __name__ == "__main__":
    unittest.main()
