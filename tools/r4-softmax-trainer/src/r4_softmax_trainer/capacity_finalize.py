"""Create-once terminal evidence binding for frozen capacity issue #1019."""

from __future__ import annotations

import json
import math
from pathlib import Path
from typing import Any

from tokenizers import Tokenizer

from .capacity import (
    BASELINE_1017_NLL,
    BASELINE_1017_WEIGHTS_CID,
    ISSUE,
    PREFIX_ADMISSION_RELATIVE_PATH,
    REVEAL_MANIFEST_RELATIVE_PATH,
    REVEAL_MANIFEST_SCHEMA,
    REVEAL_OPENED_RELATIVE_PATH,
    REVEAL_OPENED_SCHEMA,
    REVEAL_RESULT_RELATIVE_PATH,
    REVEAL_RESULT_SCHEMA,
    SEALED_TEST_LOSS_CEILING,
    SELECTION_RELATIVE_PATH,
    _verify_signed,
    load_capacity_prefix_admission,
    load_frozen_capacity_selection,
)
from .capacity_data import (
    INDEX_RELATIVE_PATHS,
    PREVIOUS_PROMPT_CIDS,
    SEALED_PROMPT_SCHEMA,
    SEALED_PROMPT_RELATIVE_PATH,
    TEST_TOKEN_CAP,
    TOKEN_RELATIVE_PATHS,
    TOKENIZER_CID,
    TOKENIZER_RELATIVE_PATH,
)
from .constants import (
    CAPACITY_MODEL_CONFIG,
    SEALED_PROMPT_TOKEN_COUNT,
)
from .finalize import (
    GENERATION_REPORT_SCHEMA,
    HUMAN_RUBRIC_CRITERION,
    _CHECKPOINT_FILE_PATHS,
    _generation_paths,
    _load_generation_pairs,
    _load_json_object_bytes,
    _require_cid,
    _validate_rubric,
)
from .provenance import (
    atomic_write,
    atomic_write_json,
    canonical_json_bytes,
    cid_bytes,
    cid_file,
    verify_bound_manifest,
    write_bound_manifest,
)


HUMAN_RUBRIC_SCHEMA = "uor-r4-softmax-trainer-capacity-human-rubric/1"
FINAL_RESULT_SCHEMA = "uor-r4-softmax-trainer-capacity-final-result/1"
FINAL_MANIFEST_SCHEMA = "uor-r4-softmax-trainer-capacity-final-manifest/1"
RUBRIC_INPUT_RELATIVE_PATH = Path("final/capacity-human-rubric.json")
FINAL_RESULT_RELATIVE_PATH = Path("final/capacity-final-result.json")
FINAL_MANIFEST_RELATIVE_PATH = Path("final/capacity-final-manifest.json")
GENERATION_SEED_BASE = 3019
_REVEAL_ARTIFACT_PATHS = {
    str(REVEAL_OPENED_RELATIVE_PATH),
    str(REVEAL_RESULT_RELATIVE_PATH),
    TOKEN_RELATIVE_PATHS["test"],
    INDEX_RELATIVE_PATHS["test"],
    SEALED_PROMPT_RELATIVE_PATH,
}


def _manifest_artifact_paths(manifest: dict[str, Any]) -> set[str]:
    artifacts = manifest.get("artifacts")
    if not isinstance(artifacts, list) or not all(
        isinstance(record, dict) and isinstance(record.get("path"), str)
        for record in artifacts
    ):
        raise ValueError("#1019 reveal manifest has malformed artifacts")
    paths = [str(record["path"]) for record in artifacts]
    if len(paths) != len(set(paths)):
        raise ValueError("#1019 reveal manifest repeats an artifact")
    return set(paths)


def _finite_nll(value: object, *, label: str) -> float:
    if isinstance(value, bool) or not isinstance(value, (int, float)):
        raise ValueError(f"{label} must be a finite number")
    converted = float(value)
    if not math.isfinite(converted):
        raise ValueError(f"{label} must be a finite number")
    return converted


def verify_capacity_generation_ready(root: Path) -> dict[str, Any]:
    """Read-only semantic gate immediately before irreversible generation."""
    manifest, reveal = _load_capacity_reveal(root.resolve())
    return {
        "schema": "uor-r4-softmax-trainer-capacity-generation-ready/1",
        "issue": ISSUE,
        "status": "READY_FOR_IRREVERSIBLE_GENERATION",
        "reveal_manifest_cid": manifest["manifest_cid"],
        "reveal_result_cid": reveal["result_cid"],
        "prompt_count": len(reveal["prompts"]),
    }


def write_capacity_rubric_template(root: Path, output_path: Path) -> dict[str, Any]:
    """Validate the ten reports and write an exact review-only rubric template."""
    root = root.resolve()
    output_path = output_path.resolve()
    if output_path.exists() or output_path.is_symlink():
        raise FileExistsError("#1019 rubric template output is create-once")
    _, reveal = _load_capacity_reveal(root)
    generation_pairs = _load_generation_pairs(
        root,
        reveal=reveal,
        generation_paths=_generation_paths(GENERATION_SEED_BASE),
        model_config=CAPACITY_MODEL_CONFIG,
        issue=ISSUE,
    )
    template = {
        "schema": HUMAN_RUBRIC_SCHEMA,
        "issue": ISSUE,
        "criterion": HUMAN_RUBRIC_CRITERION,
        "records": [
            {
                "index": pair["index"],
                "story_cid": pair["story_cid"],
                "seed": pair["seed"],
                "response_text": pair["response_text"],
                "decision": "REVIEW_REPLACE_WITH_PASS_OR_FAIL",
                "reason": "REVIEW_REPLACE_WITH_A_NONEMPTY_INDEPENDENT_REASON",
            }
            for pair in generation_pairs
        ],
    }
    atomic_write_json(output_path, template)
    return {
        "schema": "uor-r4-softmax-trainer-capacity-rubric-template-result/1",
        "issue": ISSUE,
        "output_path": str(output_path),
        "output_cid": cid_file(output_path),
        "records": len(generation_pairs),
        "status": "REVIEW_REQUIRED_NOT_ADMISSIBLE_UNTIL_EDITED",
    }


def _load_capacity_reveal(root: Path) -> tuple[dict[str, Any], dict[str, Any]]:
    selection = load_frozen_capacity_selection(root)
    parity = load_capacity_prefix_admission(root)
    manifest = verify_bound_manifest(root / REVEAL_MANIFEST_RELATIVE_PATH, artifact_root=root)
    if (
        manifest.get("schema") != REVEAL_MANIFEST_SCHEMA
        or manifest.get("issue") != ISSUE
        or _manifest_artifact_paths(manifest) != _REVEAL_ARTIFACT_PATHS
    ):
        raise ValueError("unsupported #1019 reveal manifest")
    marker = json.loads((root / REVEAL_OPENED_RELATIVE_PATH).read_text(encoding="utf-8"))
    if not isinstance(marker, dict):
        raise ValueError("unsupported #1019 reveal-opened marker")
    _verify_signed(marker, label="#1019 reveal-opened marker")
    result = json.loads((root / REVEAL_RESULT_RELATIVE_PATH).read_text(encoding="utf-8"))
    if not isinstance(result, dict) or result.get("schema") != REVEAL_RESULT_SCHEMA:
        raise ValueError("unsupported #1019 reveal result")
    _verify_signed(result, label="#1019 reveal")
    candidate_nll = _finite_nll(
        result.get("candidate_enabled_sealed_nll"), label="#1019 candidate NLL"
    )
    baseline_nll = _finite_nll(
        result.get("baseline_1017_same_tranche_enabled_nll"),
        label="#1019 same-tranche baseline NLL",
    )
    absolute = candidate_nll < SEALED_TEST_LOSS_CEILING
    relative = candidate_nll < baseline_nll
    expected_terminal = (
        "PASS_CAPACITY_NLL_ADVANCE_GENERATION"
        if absolute and relative
        else "FAIL_CAPACITY_NLL"
    )
    if (
        marker.get("schema") != REVEAL_OPENED_SCHEMA
        or marker.get("issue") != ISSUE
        or marker.get("selection_manifest_cid") != selection["manifest_cid"]
        or marker.get("prefix_admission_manifest_cid") != parity["manifest_cid"]
        or marker.get("candidate_weights_cid") != selection["weights_cid"]
        or marker.get("baseline_1017_weights_cid") != BASELINE_1017_WEIGHTS_CID
        or marker.get("sealed_confirmation_status_before_marker") != "UNOPENED"
        or marker.get("repeat_reveal_permitted") is not False
        or result.get("issue") != ISSUE
        or result.get("terminal") != expected_terminal
        or result.get("absolute_nll_passed") is not absolute
        or result.get("relative_nll_passed") is not relative
        or result.get("selection_manifest_cid") != selection["manifest_cid"]
        or result.get("prefix_admission_manifest_cid") != parity["manifest_cid"]
        or result.get("reveal_opened_result_cid") != marker["result_cid"]
        or result.get("dataset_manifest_cid") != selection["dataset_manifest_cid"]
        or result.get("candidate_weights_cid") != selection["weights_cid"]
        or result.get("weights_cid") != selection["weights_cid"]
        or result.get("tokenizer_cid") != selection["tokenizer_cid"]
        or result.get("baseline_1017_weights_cid") != BASELINE_1017_WEIGHTS_CID
        or not math.isclose(
            _finite_nll(
                result.get("candidate_minus_baseline_same_tranche_nll"),
                label="#1019 candidate-minus-baseline NLL",
            ),
            candidate_nll - baseline_nll,
            rel_tol=0.0,
            abs_tol=1e-12,
        )
        or result.get("historical_1017_sealed_nll") != BASELINE_1017_NLL
        or result.get("sealed_nll_ceiling") != SEALED_TEST_LOSS_CEILING
        or result.get("sealed_test_store_token_ids") != TEST_TOKEN_CAP
        or result.get("sealed_test_scored_next_tokens")
        != ((TEST_TOKEN_CAP - 1) // CAPACITY_MODEL_CONFIG.max_position_embeddings)
        * CAPACITY_MODEL_CONFIG.max_position_embeddings
        or result.get("sealed_prompt_token_ids") != SEALED_PROMPT_TOKEN_COUNT
        or result.get("autonomous_generation_status")
        != "NOT_RUN_RUST_SEEDED_SAMPLER_REQUIRED"
        or result.get("attention_off_executions") != 0
        or result.get("prior_sealed_artifact_reads") != 0
        or manifest.get("reveal_result_cid") != result.get("result_cid")
        or manifest.get("terminal") != expected_terminal
        or manifest.get("selection_manifest_cid") != selection["manifest_cid"]
        or manifest.get("prefix_admission_manifest_cid") != parity["manifest_cid"]
        or manifest.get("reveal_opened_result_cid") != marker["result_cid"]
        or manifest.get("candidate_enabled_sealed_nll") != candidate_nll
        or manifest.get("baseline_1017_same_tranche_enabled_nll") != baseline_nll
        or manifest.get("absolute_nll_passed") is not absolute
        or manifest.get("relative_nll_passed") is not relative
        or manifest.get("attention_off_executions") != 0
    ):
        raise ValueError("#1019 reveal identity or decision does not reproduce")
    prompts = result.get("prompts")
    if not isinstance(prompts, list) or len(prompts) != 5:
        raise ValueError("#1019 reveal must bind exactly five prompts")
    fixture = json.loads((root / SEALED_PROMPT_RELATIVE_PATH).read_text(encoding="utf-8"))
    if not isinstance(fixture, dict):
        raise ValueError("#1019 sealed prompt fixture must be a JSON object")
    unsigned_fixture = dict(fixture)
    fixture_cid = unsigned_fixture.pop("fixture_cid", None)
    fixture_prompts = fixture.get("prompts")
    if (
        fixture_cid != cid_bytes(canonical_json_bytes(unsigned_fixture))
        or fixture.get("schema") != SEALED_PROMPT_SCHEMA
        or fixture.get("issue") != ISSUE
        or fixture.get("selection")
        != (
            "first 24 content tokens of the five lowest eligible test-story CIDs "
            "strictly after the #1019 NLL-store boundary"
        )
        or fixture.get("excluded_previous_prompt_cids")
        != sorted(PREVIOUS_PROMPT_CIDS)
        or fixture.get("revealed_token_ids") != SEALED_PROMPT_TOKEN_COUNT
        or fixture.get("tokenizer_cid") != TOKENIZER_CID
        or not isinstance(fixture_prompts, list)
        or len(fixture_prompts) != 5
    ):
        raise ValueError("#1019 sealed prompt fixture identity differs")
    tokenizer_path = root / TOKENIZER_RELATIVE_PATH
    if cid_file(tokenizer_path) != selection["tokenizer_cid"]:
        raise ValueError("#1019 reveal tokenizer CID differs")
    tokenizer = Tokenizer.from_file(str(tokenizer_path))
    story_cids: set[str] = set()
    for index, prompt in enumerate(prompts):
        fixture_prompt = fixture_prompts[index]
        token_ids = prompt.get("prompt_token_ids") if isinstance(prompt, dict) else None
        prompt_text = prompt.get("prompt_text") if isinstance(prompt, dict) else None
        if (
            not isinstance(prompt, dict)
            or not isinstance(fixture_prompt, dict)
            or set(fixture_prompt) != {"story_cid", "token_ids", "text"}
            or prompt.get("index") != index
            or prompt.get("seed") != GENERATION_SEED_BASE + index
            or prompt.get("prompt_tokens") != 24
            or not isinstance(prompt_text, str)
            or not prompt_text
            or not isinstance(token_ids, list)
            or len(token_ids) != 24
            or any(
                isinstance(token, bool)
                or not isinstance(token, int)
                or not 0 <= token < CAPACITY_MODEL_CONFIG.vocab_size
                for token in token_ids
            )
            or tokenizer.decode(token_ids, skip_special_tokens=True) != prompt_text
            or tokenizer.encode(prompt_text, add_special_tokens=False).ids != token_ids
            or prompt.get("story_cid") != fixture_prompt.get("story_cid")
            or token_ids != fixture_prompt.get("token_ids")
            or prompt_text != fixture_prompt.get("text")
        ):
            raise ValueError(f"#1019 reveal prompt {index} differs")
        story_cid = _require_cid(
            prompt.get("story_cid"), label=f"#1019 prompt {index} story CID"
        )
        if story_cid in PREVIOUS_PROMPT_CIDS or story_cid in story_cids:
            raise ValueError(f"#1019 reveal prompt {index} is not fresh and distinct")
        story_cids.add(story_cid)
    if expected_terminal != "PASS_CAPACITY_NLL_ADVANCE_GENERATION":
        raise ValueError("#1019 generation is forbidden after a negative NLL reveal")
    return manifest, result


def _terminal(failures: list[str]) -> str:
    if not failures:
        return "PASS_CAPACITY_QUALITY_BASELINE"
    return "FAIL_" + "_AND_".join(failures)


def finalize_capacity(root: Path, rubric_path: Path) -> dict[str, Any]:
    """Bind the reveal, five Rust runs plus replays, and one independent rubric."""
    root = root.resolve()
    rubric_path = rubric_path.resolve()
    final_paths = [
        root / RUBRIC_INPUT_RELATIVE_PATH,
        root / FINAL_RESULT_RELATIVE_PATH,
        root / FINAL_MANIFEST_RELATIVE_PATH,
    ]
    if any(path.exists() or path.is_symlink() for path in final_paths):
        raise FileExistsError("#1019 final evidence is create-once")
    if rubric_path in final_paths:
        raise ValueError("#1019 rubric input must be outside the final directory")
    reveal_manifest, reveal = _load_capacity_reveal(root)
    generation_paths = _generation_paths(GENERATION_SEED_BASE)
    generation_pairs = _load_generation_pairs(
        root,
        reveal=reveal,
        generation_paths=generation_paths,
        model_config=CAPACITY_MODEL_CONFIG,
        issue=ISSUE,
    )
    rubric_bytes, rubric = _load_json_object_bytes(
        rubric_path, label="#1019 human rubric input"
    )
    rubric_records = _validate_rubric(
        rubric,
        generation_pairs=generation_pairs,
        issue=ISSUE,
        schema=HUMAN_RUBRIC_SCHEMA,
    )
    absolute_nll = reveal["absolute_nll_passed"] is True
    relative_nll = reveal["relative_nll_passed"] is True
    replay_passed = len(generation_pairs) == 5
    all_nonlooping = all(pair["nonlooping"] for pair in generation_pairs)
    rubric_pass_count = sum(record["decision"] == "PASS" for record in rubric_records)
    rubric_passed = rubric_pass_count >= 4
    failures: list[str] = []
    if not absolute_nll:
        failures.append("ABSOLUTE_NLL")
    if not relative_nll:
        failures.append("RELATIVE_NLL")
    if not replay_passed:
        failures.append("DETERMINISTIC_REPLAY")
    if not all_nonlooping:
        failures.append("SHORT_CYCLE")
    if not rubric_passed:
        failures.append("GENERATION_RUBRIC")
    terminal = _terminal(failures)

    atomic_write(root / RUBRIC_INPUT_RELATIVE_PATH, rubric_bytes)
    rubric_cid = cid_file(root / RUBRIC_INPUT_RELATIVE_PATH)
    result: dict[str, Any] = {
        "schema": FINAL_RESULT_SCHEMA,
        "issue": ISSUE,
        "terminal": terminal,
        "claim_scope": (
            "one frozen 13,130,784-parameter ordinary causal-softmax model, "
            "all twelve layers executed through coherent R4/Spin frames"
        ),
        "selection_manifest_cid": reveal["selection_manifest_cid"],
        "reveal_manifest_cid": reveal_manifest["manifest_cid"],
        "reveal_result_cid": reveal["result_cid"],
        "weights_cid": reveal["weights_cid"],
        "tokenizer_cid": reveal["tokenizer_cid"],
        "candidate_enabled_sealed_nll": reveal["candidate_enabled_sealed_nll"],
        "baseline_1017_same_tranche_enabled_nll": reveal[
            "baseline_1017_same_tranche_enabled_nll"
        ],
        "absolute_nll_passed": absolute_nll,
        "relative_nll_passed": relative_nll,
        "human_rubric_schema": HUMAN_RUBRIC_SCHEMA,
        "human_rubric_cid": rubric_cid,
        "rubric_pass_count": rubric_pass_count,
        "rubric_required_pass_count": 4,
        "attention_off_executions": 0,
        "generations": [
            {
                **pair,
                "rubric_decision": rubric_records[pair["index"]]["decision"],
                "rubric_reason": rubric_records[pair["index"]]["reason"],
            }
            for pair in generation_pairs
        ],
        "definition_of_done": {
            "candidate_enabled_sealed_nll_below_1_50": absolute_nll,
            "candidate_below_1017_on_same_fresh_tranche": relative_nll,
            "exactly_five_primary_and_five_replay_reports": replay_passed,
            "all_reports_exact_twelve_layer_causal_projection_r4_output_policy": True,
            "all_reports_external_source_reads_zero": True,
            "all_replays_equal_excluding_timing_with_identical_core_cids": replay_passed,
            "all_outputs_utf8_and_without_period_1_to_4_short_cycle": all_nonlooping,
            "at_least_four_of_five_human_rubric_pass": rubric_passed,
            "overall_pass": not failures,
        },
        "failed_gates": failures,
        "rerun_generation_permitted": False,
        "repeat_reveal_permitted": False,
    }
    result["result_cid"] = cid_bytes(canonical_json_bytes(result))
    atomic_write_json(root / FINAL_RESULT_RELATIVE_PATH, result)
    artifact_paths = {
        str(SELECTION_RELATIVE_PATH),
        str(PREFIX_ADMISSION_RELATIVE_PATH),
        str(REVEAL_OPENED_RELATIVE_PATH),
        str(REVEAL_RESULT_RELATIVE_PATH),
        str(REVEAL_MANIFEST_RELATIVE_PATH),
        *{f"export/{path}" for path in _CHECKPOINT_FILE_PATHS},
        *{
            str(path)
            for _, _, primary, replay in generation_paths
            for path in (primary, replay)
        },
        str(RUBRIC_INPUT_RELATIVE_PATH),
        str(FINAL_RESULT_RELATIVE_PATH),
    }
    return write_bound_manifest(
        root / FINAL_MANIFEST_RELATIVE_PATH,
        {
            "schema": FINAL_MANIFEST_SCHEMA,
            "issue": ISSUE,
            "terminal": terminal,
            "final_result_cid": result["result_cid"],
            "selection_manifest_cid": reveal["selection_manifest_cid"],
            "reveal_manifest_cid": reveal_manifest["manifest_cid"],
            "reveal_result_cid": reveal["result_cid"],
            "human_rubric_cid": rubric_cid,
            "definition_of_done": result["definition_of_done"],
            "failed_gates": failures,
            "attention_off_executions": 0,
            "generation_executions_by_finalizer": 0,
            "reveal_executions_by_finalizer": 0,
        },
        artifact_root=root,
        relative_paths=sorted(artifact_paths),
    )
