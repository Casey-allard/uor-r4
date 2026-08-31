"""Create-once finalization of the already-open #1017 confirmation campaign.

This stage never executes the model, opens a new population, or regenerates a
continuation.  It validates the frozen reveal and the ten existing Rust
reports, binds an independently supplied human rubric, and writes the terminal
evidence exactly once.
"""

from __future__ import annotations

import copy
import json
import math
import stat
from pathlib import Path
from typing import Any

from .constants import FROZEN_MODEL_CONFIG, ModelConfig
from .continuation import (
    CONTINUATION_REVEAL_MANIFEST_SCHEMA,
    CONTINUATION_REVEAL_RESULT_SCHEMA,
    ISSUE,
    REVEAL_MANIFEST_RELATIVE_PATH,
    REVEAL_OPENED_RELATIVE_PATH,
    REVEAL_RESULT_RELATIVE_PATH,
    SEALED_TEST_LOSS_CEILING,
    SELECTION_RELATIVE_PATH,
    _load_frozen_continuation_selection,
    _manifest_artifact_paths,
    _verify_signed_result,
    load_enabled_parity_admission,
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


GENERATION_REPORT_SCHEMA = "uor-r4.r4-softmax-local-generation/1"
GENERATION_POLICY_SCHEMA = "R4SoftmaxLocalGeneratorV1"
HUMAN_RUBRIC_SCHEMA = "uor-r4-softmax-trainer-continuation-human-rubric/1"
FINAL_RESULT_SCHEMA = "uor-r4-softmax-trainer-continuation-final-result/1"
FINAL_MANIFEST_SCHEMA = "uor-r4-softmax-trainer-continuation-final-manifest/1"

HUMAN_RUBRIC_CRITERION = (
    "PASS iff the response retains the prompt subject or scene as a coherent "
    "continuation; machine audits independently enforce UTF-8 and no period-1..4 "
    "token cycle."
)
SEEDED_SAMPLER_POLICY = (
    "r4-local-top-k-q32-splitmix64/1;temperature=0.8;top-k=40;"
    "rank=logit-desc-token-asc"
)
R4_POLICY_IDENTITY = (
    "schema=helm-d-r4-gauge-softmax/1\n"
    "scope=offline-full-prefix-causal-softmax-oracle\n"
    "head-layout=complete-consecutive-R4-blocks\n"
    "frame=exact-cumulative-UOR-Spin-H4-left-quaternion\n"
    "encode=F_position_transpose-times-model-vector\n"
    "transport=P_source_to_query=F_query_transpose-times-F_source\n"
    "transported-state=every-causal-key-and-value\n"
    "score=unchanged-scaled-dot-product-in-query-gauge\n"
    "selector=unchanged-stable-causal-softmax\n"
    "aggregate=unchanged-weighted-value-sum-in-query-gauge\n"
    "decode=F_query-times-query-gauge-output-before-Wo\n"
    "control=source-frame-permuted-with-identical-shape-and-work\n"
    "expected=ordinary-attention-numerical-and-behavioral-parity\n"
    "not-claimed=geometry-advantage,intrinsic-distance,transformerless-serving,"
    "softmax-removal,source-free-language-model"
)

RUBRIC_INPUT_RELATIVE_PATH = Path("final/human-rubric.json")
FINAL_RESULT_RELATIVE_PATH = Path("final/continuation-final-result.json")
FINAL_MANIFEST_RELATIVE_PATH = Path("final/continuation-final-manifest.json")

_REVEAL_ARTIFACT_PATHS = {
    str(REVEAL_OPENED_RELATIVE_PATH),
    str(REVEAL_RESULT_RELATIVE_PATH),
    "sealed-confirmation/prompts.json",
    "sealed-confirmation/test-index.jsonl",
    "sealed-confirmation/test.u16",
}
_CHECKPOINT_FILE_PATHS = {
    "config.json",
    "export-manifest.json",
    "model.safetensors",
    "tokenizer.json",
    "training-result.json",
}
_RUBRIC_FIELDS = {
    "index",
    "story_cid",
    "seed",
    "decision",
    "response_text",
    "reason",
}
_CORE_CID_FIELDS = (
    "decision_cid",
    "generation_policy_cid",
    "output_cid",
    "audit_cid",
)


def _require_cid(value: object, *, label: str) -> str:
    if (
        not isinstance(value, str)
        or not value.startswith("blake3:")
        or len(value) != len("blake3:") + 64
        or any(character not in "0123456789abcdef" for character in value[7:])
    ):
        raise ValueError(f"{label} is not a lowercase BLAKE3 CID")
    return value


def _reject_duplicate_pairs(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
    value: dict[str, Any] = {}
    for key, item in pairs:
        if key in value:
            raise ValueError(f"JSON object repeats key {key}")
        value[key] = item
    return value


def _load_json_object_bytes(path: Path, *, label: str) -> tuple[bytes, dict[str, Any]]:
    metadata = path.lstat()
    if not stat.S_ISREG(metadata.st_mode) or path.is_symlink():
        raise ValueError(f"{label} must be a regular non-symlink file")
    raw = path.read_bytes()
    value = json.loads(raw, object_pairs_hook=_reject_duplicate_pairs)
    if not isinstance(value, dict):
        raise ValueError(f"{label} must be a JSON object")
    return raw, value


def _raw_json_cid(value: Any) -> str:
    raw = json.dumps(
        value,
        ensure_ascii=False,
        allow_nan=False,
        separators=(",", ":"),
    ).encode("utf-8")
    return cid_bytes(raw)


def _canonical_value_cid(value: Any) -> str:
    return cid_bytes(canonical_json_bytes(value))


def _generation_paths(seed_base: int = 2014) -> list[tuple[int, int, Path, Path]]:
    return [
        (
            index,
            seed_base + index,
            Path(f"generations/prompt-{index}-seed-{seed_base + index}.json"),
            Path(f"generations/replay/prompt-{index}-seed-{seed_base + index}.json"),
        )
        for index in range(5)
    ]


def _verify_exact_generation_files(
    root: Path,
    *,
    generation_paths: list[tuple[int, int, Path, Path]] | None = None,
    issue: int = ISSUE,
) -> None:
    generation_root = root / "generations"
    if not generation_root.is_dir() or generation_root.is_symlink():
        raise FileNotFoundError(f"#{issue} generation directory is unavailable")
    paths = generation_paths or _generation_paths()
    expected = {
        str(path.relative_to("generations"))
        for _, _, primary, replay in paths
        for path in (primary, replay)
    }
    observed: set[str] = set()
    for path in generation_root.rglob("*"):
        if path.is_symlink():
            raise ValueError(f"#{issue} generation directory contains a symlink")
        if path.is_file():
            observed.add(str(path.relative_to(generation_root)))
    if observed != expected:
        raise ValueError(
            f"#{issue} generation directory must contain exactly five primary and five "
            f"replay reports; expected={sorted(expected)}, observed={sorted(observed)}"
        )


def _load_reveal(root: Path) -> tuple[dict[str, Any], dict[str, Any]]:
    selection = _load_frozen_continuation_selection(root)
    parity_admission = load_enabled_parity_admission(root)
    manifest = verify_bound_manifest(
        root / REVEAL_MANIFEST_RELATIVE_PATH, artifact_root=root
    )
    if manifest.get("schema") != CONTINUATION_REVEAL_MANIFEST_SCHEMA:
        raise ValueError("unsupported #1017 reveal manifest")
    if _manifest_artifact_paths(manifest, label="continuation reveal") != (
        _REVEAL_ARTIFACT_PATHS
    ):
        raise ValueError("#1017 reveal manifest binds unexpected artifacts")
    _, result = _load_json_object_bytes(
        root / REVEAL_RESULT_RELATIVE_PATH, label="#1017 reveal result"
    )
    if result.get("schema") != CONTINUATION_REVEAL_RESULT_SCHEMA:
        raise ValueError("unsupported #1017 reveal result")
    _verify_signed_result(result, label="continuation reveal")
    expected_terminal = (
        "PASS_ENABLED_NLL"
        if result.get("sealed_test_loss_passed") is True
        else "FAIL_ENABLED_NLL"
    )
    loss = float(result.get("enabled_sealed_test_loss", math.nan))
    loss_passed = math.isfinite(loss) and loss < SEALED_TEST_LOSS_CEILING
    if (
        result.get("issue") != ISSUE
        or result.get("terminal") != expected_terminal
        or result.get("sealed_test_loss_ceiling") != SEALED_TEST_LOSS_CEILING
        or result.get("sealed_test_loss_passed") is not loss_passed
        or result.get("attention_off_executions") != 0
        or result.get("autonomous_generation_status")
        != "NOT_RUN_RUST_SEEDED_SAMPLER_REQUIRED"
        or result.get("selection_manifest_cid") != selection.get("manifest_cid")
        or result.get("selected_checkpoint_cid")
        != selection.get("selected_checkpoint_cid")
        or result.get("weights_cid") != selection.get("weights_cid")
        or result.get("tokenizer_cid") != selection.get("tokenizer_cid")
        or result.get("enabled_parity_admission_manifest_cid")
        != parity_admission.get("manifest_cid")
        or manifest.get("reveal_result_cid") != result.get("result_cid")
        or manifest.get("terminal") != expected_terminal
        or manifest.get("enabled_sealed_test_loss") != loss
        or manifest.get("sealed_test_loss_passed") is not loss_passed
        or manifest.get("attention_off_executions") != 0
    ):
        raise ValueError("#1017 reveal decision or identity does not reproduce")
    prompts = result.get("prompts")
    if not isinstance(prompts, list) or len(prompts) != 5:
        raise ValueError("#1017 reveal must bind exactly five prompts")
    for index, prompt in enumerate(prompts):
        if not isinstance(prompt, dict):
            raise ValueError("#1017 reveal prompt is not an object")
        if (
            prompt.get("index") != index
            or prompt.get("seed") != 2014 + index
            or prompt.get("prompt_tokens") != 24
            or not isinstance(prompt.get("prompt_text"), str)
            or not isinstance(prompt.get("prompt_token_ids"), list)
            or len(prompt["prompt_token_ids"]) != 24
        ):
            raise ValueError(f"#1017 reveal prompt {index} differs from its frozen contract")
        _require_cid(prompt.get("story_cid"), label=f"reveal prompt {index} story CID")
    return manifest, result


def _checkpoint_records(root: Path) -> list[dict[str, Any]]:
    export = root / "export"
    observed = {
        str(path.relative_to(export))
        for path in export.iterdir()
        if path.is_file() and not path.is_symlink()
    }
    if observed != _CHECKPOINT_FILE_PATHS:
        raise ValueError("#1017 export tree differs from the five-file Rust checkpoint")
    return [
        {
            "path": relative,
            "bytes": (export / relative).stat().st_size,
            "kappa": cid_file(export / relative),
        }
        for relative in sorted(_CHECKPOINT_FILE_PATHS)
    ]


def _reproduce_generation_cids(report: dict[str, Any]) -> None:
    for field in _CORE_CID_FIELDS:
        _require_cid(report.get(field), label=f"Rust generation {field}")
    checkpoint = report["checkpoint"]
    transcript = report["transcript"]
    attention = report["attention_audit"]
    output_policy = report["attention_output_policy_audit"]
    decode = report["decode_audit"]
    source = report["source_read_audit"]
    decision_identity: dict[str, Any] = {
        "schema": GENERATION_REPORT_SCHEMA,
        "policy": GENERATION_POLICY_SCHEMA,
        "checkpoint_tree_cid": checkpoint["checkpoint_tree_cid"],
        "config_cid": checkpoint["config_cid"],
        "tokenizer_cid": checkpoint["tokenizer_cid"],
        "weights_cid": checkpoint["weights_cid"],
        "model_shape": report["model_shape"],
        "prompt": report["prompt"],
        "prompt_token_ids": report["prompt_token_ids"],
        "generated_token_ids": transcript["generated_token_ids"],
        "sampler_policy": decode["sampler_policy"],
        "seed": decode["seed"],
        "stop_reason": report["stop_reason"],
        "persistent_state_cid": report["persistent_state_cid"],
        "attention_audit": attention,
        "attention_output_policy_audit": output_policy,
        "source_read_audit": source,
    }
    expected = {
        "decision_cid": _raw_json_cid(decision_identity),
        "generation_policy_cid": _canonical_value_cid(
            [
                GENERATION_POLICY_SCHEMA,
                R4_POLICY_IDENTITY,
                "causal-attention-output-enabled/1",
                SEEDED_SAMPLER_POLICY,
                decode["seed"],
                128,
            ]
        ),
        "output_cid": _canonical_value_cid(
            [
                transcript["generated_token_ids"],
                transcript["raw_decoded"],
                transcript["response_text"],
                report["stop_reason"],
            ]
        ),
        "audit_cid": _canonical_value_cid(
            [attention, output_policy, decode, source]
        ),
    }
    for field, identity in expected.items():
        if report.get(field) != identity:
            raise ValueError(f"Rust generation {field} does not reproduce")


def _validate_generation_report(
    root: Path,
    report: dict[str, Any],
    *,
    prompt: dict[str, Any],
    index: int,
    seed: int,
    checkpoint_records: list[dict[str, Any]],
    reveal: dict[str, Any],
    model_config: ModelConfig = FROZEN_MODEL_CONFIG,
) -> dict[str, Any]:
    if report.get("schema") != GENERATION_REPORT_SCHEMA:
        raise ValueError(f"Rust generation {index} has an unsupported schema")
    required_objects = (
        "checkpoint",
        "model_shape",
        "transcript",
        "attention_audit",
        "attention_output_policy_audit",
        "decode_audit",
        "source_read_audit",
    )
    if any(not isinstance(report.get(field), dict) for field in required_objects):
        raise ValueError(f"Rust generation {index} omits a required object")

    checkpoint = report["checkpoint"]
    export_path = (root / "export").resolve()
    try:
        reported_model_path = Path(checkpoint.get("model_path", "")).resolve(strict=True)
    except (OSError, RuntimeError) as error:
        raise ValueError(f"Rust generation {index} checkpoint path is unavailable") from error
    checkpoint_tree_identity = {
        "schema": "uor-r4.r4-softmax-local-checkpoint-tree/1",
        "files": checkpoint_records,
    }
    if (
        reported_model_path != export_path
        or checkpoint.get("files") != checkpoint_records
        or checkpoint.get("checkpoint_tree_cid")
        != _raw_json_cid(checkpoint_tree_identity)
        or checkpoint.get("config_cid") != cid_file(root / "export/config.json")
        or checkpoint.get("tokenizer_cid") != reveal.get("tokenizer_cid")
        or checkpoint.get("tokenizer_cid") != cid_file(root / "export/tokenizer.json")
        or checkpoint.get("weights_cid") != reveal.get("weights_cid")
        or checkpoint.get("weights_cid") != cid_file(root / "export/model.safetensors")
        or checkpoint.get("bos_token_id") != 0
        or checkpoint.get("eos_token_id") != 1
    ):
        raise ValueError(f"Rust generation {index} checkpoint binding differs")

    shape = report["model_shape"]
    expected_shape = {
        "dimension": model_config.hidden_size,
        "hidden_dimension": model_config.intermediate_size,
        "layers": model_config.num_hidden_layers,
        "query_heads": model_config.num_attention_heads,
        "key_value_heads": model_config.num_key_value_heads,
        "head_size": model_config.head_dim,
        "vocabulary": model_config.vocab_size,
        "sequence_capacity": 153,
    }
    expected_prompt_ids = [0, *prompt["prompt_token_ids"]]
    transcript = report["transcript"]
    generated = transcript.get("generated_token_ids")
    if (
        shape != expected_shape
        or report.get("prompt") != prompt["prompt_text"]
        or report.get("prompt_token_ids") != expected_prompt_ids
        or transcript.get("prompt") != prompt["prompt_text"]
        or transcript.get("input_tokens") != len(expected_prompt_ids)
        or not isinstance(generated, list)
        or not 1 <= len(generated) <= 128
        or any(not isinstance(token, int) or not 0 <= token < shape["vocabulary"] for token in generated)
        or transcript.get("utf8_decodable") is not True
        or not isinstance(transcript.get("response_text"), str)
    ):
        raise ValueError(f"Rust generation {index} prompt, output, or shape differs")

    decode = report["decode_audit"]
    if (
        decode.get("selection")
        != "deterministic seeded temperature/top-k sampling over local checkpoint logits"
        or decode.get("deterministic_greedy") is not False
        or decode.get("sampler_policy") != SEEDED_SAMPLER_POLICY
        or decode.get("seed") != seed
        or decode.get("bos_insertions") != 1
        or decode.get("utf8_decodable") is not True
        or decode.get("cycles_checked") != [1, 2, 3, 4]
        or decode.get("short_cycle_period") != transcript.get("short_cycle_period")
    ):
        raise ValueError(f"Rust generation {index} sampler or decode audit differs")

    stop_reason = report.get("stop_reason")
    first_eos = transcript.get("first_eos_offset")
    if stop_reason == "eos":
        if first_eos != len(generated) - 1 or generated[-1] != 1:
            raise ValueError(f"Rust generation {index} EOS binding differs")
    elif stop_reason == "maximum_new_tokens":
        if len(generated) != 128 or first_eos is not None:
            raise ValueError(f"Rust generation {index} max-token binding differs")
    elif isinstance(stop_reason, dict) and set(stop_reason) == {"short_cycle"}:
        short_cycle = stop_reason["short_cycle"]
        if (
            not isinstance(short_cycle, dict)
            or set(short_cycle) != {"period"}
            or short_cycle.get("period") not in [1, 2, 3, 4]
            or transcript.get("short_cycle_period") != short_cycle.get("period")
        ):
            raise ValueError(f"Rust generation {index} short-cycle binding differs")
    else:
        raise ValueError(f"Rust generation {index} has an unknown stop reason")

    positions = len(expected_prompt_ids) + len(generated) - 1
    attention = report["attention_audit"]
    implementation = attention.get("r4_implementation")
    if not isinstance(implementation, dict):
        raise ValueError(f"Rust generation {index} has no R4 implementation audit")
    r4_audit = implementation.get("audit")
    if (
        attention.get("selected_layer_count") != model_config.num_hidden_layers
        or attention.get("positions_executed") != positions
        or attention.get("observed_causal") != attention.get("expected_causal")
        or attention.get("causal_audit_exact") is not True
        or attention.get("observed_projection") != attention.get("expected_projection")
        or attention.get("projection_audit_exact") is not True
        or implementation.get("schema") != "uor-r4.r4-spin-transport-evidence/1"
        or implementation.get("policy_identity") != R4_POLICY_IDENTITY
        or implementation.get("intervention") != "coherent"
        or not isinstance(implementation.get("frame_table_offsets"), list)
        or len(implementation["frame_table_offsets"]) != positions
        or r4_audit != attention.get("expected_r4")
        or attention.get("r4_audit_exact") is not True
        or attention.get("zero_future_reads") is not True
        or attention.get("all_layers_selected") is not True
        or attention.get("observed_causal", {}).get("future_reads") != 0
        or not isinstance(r4_audit, dict)
        or r4_audit.get("future_position_reads") != 0
        or r4_audit.get("source_frame_permutations") != 0
    ):
        raise ValueError(f"Rust generation {index} is not exact all-layer causal R4")

    policy = report["attention_output_policy_audit"]
    applications = positions * model_config.num_hidden_layers
    lanes = applications * model_config.hidden_size
    if (
        policy.get("policy") != "causal-attention-output-enabled/1"
        or policy.get("applications") != applications
        or policy.get("enabled_applications") != applications
        or policy.get("zeroed_applications") != 0
        or policy.get("output_lanes") != lanes
        or policy.get("nonzero_lanes_before_policy")
        != policy.get("nonzero_lanes_after_policy")
        or policy.get("applications_by_layer")
        != [positions] * model_config.num_hidden_layers
        or policy.get("maximum_query_position") != positions - 1
        or policy.get("exact") is not True
    ):
        raise ValueError(f"Rust generation {index} output-policy audit differs")

    source = report["source_read_audit"]
    if (
        source.get("checkpoint_tree_scans") != 2
        or source.get("checkpoint_tree_file_reads") != len(checkpoint_records) * 2
        or source.get("tokenizer_loads") != 1
        or source.get("oracle_loads") != 1
        or source.get("local_checkpoint_forward_steps") != positions
        or any(
            source.get(field) != 0
            for field in ("provider_calls", "ollama_calls", "prior_trace_reads")
        )
        or source.get("tree_unchanged_across_execution") is not True
    ):
        raise ValueError(f"Rust generation {index} source-read audit differs")

    _require_cid(report.get("persistent_state_cid"), label="persistent state CID")
    _reproduce_generation_cids(report)
    return {
        "utf8_decodable": True,
        "short_cycle_period": transcript.get("short_cycle_period"),
        "nonlooping": transcript.get("short_cycle_period") is None
        and not isinstance(stop_reason, dict),
    }


def _normalized_generation_report(report: dict[str, Any]) -> dict[str, Any]:
    normalized = copy.deepcopy(report)
    normalized.pop("timing", None)
    return normalized


def _load_generation_pairs(
    root: Path,
    *,
    reveal: dict[str, Any],
    generation_paths: list[tuple[int, int, Path, Path]] | None = None,
    model_config: ModelConfig = FROZEN_MODEL_CONFIG,
    issue: int = ISSUE,
) -> list[dict[str, Any]]:
    paths = generation_paths or _generation_paths()
    _verify_exact_generation_files(root, generation_paths=paths, issue=issue)
    checkpoint_records = _checkpoint_records(root)
    pairs: list[dict[str, Any]] = []
    for index, seed, primary_relative, replay_relative in paths:
        _, primary = _load_json_object_bytes(
            root / primary_relative, label=f"primary Rust generation {index}"
        )
        _, replay = _load_json_object_bytes(
            root / replay_relative, label=f"replay Rust generation {index}"
        )
        prompt = reveal["prompts"][index]
        primary_quality = _validate_generation_report(
            root,
            primary,
            prompt=prompt,
            index=index,
            seed=seed,
            checkpoint_records=checkpoint_records,
            reveal=reveal,
            model_config=model_config,
        )
        replay_quality = _validate_generation_report(
            root,
            replay,
            prompt=prompt,
            index=index,
            seed=seed,
            checkpoint_records=checkpoint_records,
            reveal=reveal,
            model_config=model_config,
        )
        if _normalized_generation_report(primary) != _normalized_generation_report(replay):
            raise ValueError(
                f"Rust generation {index} replay differs after timing normalization"
            )
        primary_core = {field: primary[field] for field in _CORE_CID_FIELDS}
        replay_core = {field: replay[field] for field in _CORE_CID_FIELDS}
        if primary_core != replay_core:
            raise ValueError(f"Rust generation {index} replay core CIDs differ")
        pairs.append(
            {
                "index": index,
                "story_cid": prompt["story_cid"],
                "seed": seed,
                "prompt_text": prompt["prompt_text"],
                "response_text": primary["transcript"]["response_text"],
                "generated_tokens": len(primary["transcript"]["generated_token_ids"]),
                "stop_reason": primary["stop_reason"],
                "nonlooping": primary_quality["nonlooping"]
                and replay_quality["nonlooping"],
                "primary_path": str(primary_relative),
                "primary_report_cid": cid_file(root / primary_relative),
                "replay_path": str(replay_relative),
                "replay_report_cid": cid_file(root / replay_relative),
                "core_cids": primary_core,
            }
        )
    return pairs


def _validate_rubric(
    rubric: dict[str, Any],
    *,
    generation_pairs: list[dict[str, Any]],
    issue: int = ISSUE,
    schema: str = HUMAN_RUBRIC_SCHEMA,
) -> list[dict[str, Any]]:
    if set(rubric) != {"schema", "issue", "criterion", "records"}:
        raise ValueError(f"#{issue} human rubric has extra or missing top-level fields")
    if (
        rubric.get("schema") != schema
        or rubric.get("issue") != issue
        or rubric.get("criterion") != HUMAN_RUBRIC_CRITERION
    ):
        raise ValueError(f"#{issue} human rubric identity or criterion differs")
    records = rubric.get("records")
    if not isinstance(records, list) or len(records) != 5:
        raise ValueError(f"#{issue} human rubric must contain exactly five records")
    validated: list[dict[str, Any]] = []
    for index, (record, generation) in enumerate(zip(records, generation_pairs, strict=True)):
        if not isinstance(record, dict) or set(record) != _RUBRIC_FIELDS:
            raise ValueError(
                f"#{issue} human rubric record {index} has extra or missing fields"
            )
        if (
            record.get("index") != index
            or record.get("story_cid") != generation["story_cid"]
            or record.get("seed") != generation["seed"]
            or record.get("response_text") != generation["response_text"]
            or record.get("decision") not in {"PASS", "FAIL"}
            or not isinstance(record.get("reason"), str)
            or not record["reason"].strip()
        ):
            raise ValueError(
                f"#{issue} human rubric record {index} does not bind its rollout"
            )
        validated.append(record)
    return validated


def _terminal(failures: list[str]) -> str:
    if not failures:
        return "PASS_COHERENT_R4_SOFTMAX_GENERATION"
    return "FAIL_" + "_AND_".join(failures)


def finalize_continuation(root: Path, rubric_path: Path) -> dict[str, Any]:
    """Bind the frozen reveal, ten existing Rust reports, and one rubric once."""
    root = root.resolve()
    rubric_path = rubric_path.resolve()
    final_paths = [
        root / RUBRIC_INPUT_RELATIVE_PATH,
        root / FINAL_RESULT_RELATIVE_PATH,
        root / FINAL_MANIFEST_RELATIVE_PATH,
    ]
    if any(path.exists() or path.is_symlink() for path in final_paths):
        raise FileExistsError("#1017 final evidence is already present or partially written")
    if rubric_path in final_paths:
        raise ValueError("human rubric input must be outside the create-once final directory")

    reveal_manifest, reveal = _load_reveal(root)
    generation_pairs = _load_generation_pairs(root, reveal=reveal)
    rubric_bytes, rubric = _load_json_object_bytes(
        rubric_path, label="#1017 human rubric input"
    )
    rubric_records = _validate_rubric(rubric, generation_pairs=generation_pairs)

    nll_passed = reveal["sealed_test_loss_passed"] is True
    replay_passed = len(generation_pairs) == 5
    all_nonlooping = all(pair["nonlooping"] for pair in generation_pairs)
    rubric_pass_count = sum(record["decision"] == "PASS" for record in rubric_records)
    rubric_passed = rubric_pass_count >= 4
    failures: list[str] = []
    if not nll_passed:
        failures.append("ENABLED_NLL")
    if not replay_passed:
        failures.append("DETERMINISTIC_REPLAY")
    if not all_nonlooping:
        failures.append("SHORT_CYCLE")
    if not rubric_passed:
        failures.append("GENERATION_RUBRIC")
    terminal = _terminal(failures)

    # Copy the exact independent input bytes before writing any derived result.
    atomic_write(root / RUBRIC_INPUT_RELATIVE_PATH, rubric_bytes)
    rubric_cid = cid_file(root / RUBRIC_INPUT_RELATIVE_PATH)
    result: dict[str, Any] = {
        "schema": FINAL_RESULT_SCHEMA,
        "issue": ISSUE,
        "terminal": terminal,
        "claim_scope": (
            "frozen #1017 enabled-attention NLL plus five autonomous all-layer "
            "R4/Spin Rust continuations and exact deterministic replays"
        ),
        "selection_manifest_cid": reveal["selection_manifest_cid"],
        "reveal_manifest_cid": reveal_manifest["manifest_cid"],
        "reveal_result_cid": reveal["result_cid"],
        "selected_checkpoint_cid": reveal["selected_checkpoint_cid"],
        "weights_cid": reveal["weights_cid"],
        "tokenizer_cid": reveal["tokenizer_cid"],
        "human_rubric_schema": HUMAN_RUBRIC_SCHEMA,
        "human_rubric_cid": rubric_cid,
        "enabled_sealed_test_loss": reveal["enabled_sealed_test_loss"],
        "sealed_test_loss_ceiling": SEALED_TEST_LOSS_CEILING,
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
            "sealed_enabled_nll_below_1_50": nll_passed,
            "exactly_five_primary_and_five_replay_reports": replay_passed,
            "all_reports_bind_checkpoint_tokenizer_weights_prompt_seed_sampler": True,
            "all_reports_exact_six_layer_causal_projection_r4_output_policy": True,
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
        "qualification/enabled-prefix-admission.json",
        str(REVEAL_OPENED_RELATIVE_PATH),
        str(REVEAL_RESULT_RELATIVE_PATH),
        str(REVEAL_MANIFEST_RELATIVE_PATH),
        *{f"export/{path}" for path in _CHECKPOINT_FILE_PATHS},
        *{
            str(path)
            for _, _, primary, replay in _generation_paths()
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
            "enabled_sealed_test_loss": reveal["enabled_sealed_test_loss"],
            "rubric_pass_count": rubric_pass_count,
            "definition_of_done": result["definition_of_done"],
            "failed_gates": failures,
            "attention_off_executions": 0,
            "generation_executions_by_finalizer": 0,
            "reveal_executions_by_finalizer": 0,
        },
        artifact_root=root,
        relative_paths=sorted(artifact_paths),
    )
