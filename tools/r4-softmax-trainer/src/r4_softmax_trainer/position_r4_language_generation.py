"""Artifact-only generation through position-preserving R4 attention.

This path loads the completed ordinary #973 artifact into the existing
chronological K/V implementation and executes its coherent H4 content transport.
It does not fit, load the historical #1043 fitted artifact, or read corpus or
teacher data.
"""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any

import torch
from tokenizers import Tokenizer

from .group_retention_campaign import load_group_geometry_artifacts
from .h4_spin_frame_sidecar import H4SpinFrameArtifactV1
from .language_path_generalization import (
    CONTEXT,
    HEAD_DIM,
    HEADS,
    LAYERS,
    PARAMETER_COUNT,
    STATE_BYTES_F32,
    STATE_VALUES,
    VALIDITY_BITS,
    VOCAB_SIZE,
)
from .language_path_generation import (
    BOS_TOKEN_ID,
    EOS_TOKEN_ID,
    TEMPERATURE,
    TOP_K,
    ByteLevelRawDecoder,
    SplitMix64,
    sample_top_k_q32,
    short_cycle_period,
)
from .ordinary_language_generation import (
    ARTIFACT_RELATIVE_PATH,
    DEFAULT_MAX_NEW_TOKENS,
    DEFAULT_SEED,
    EXPECTED_ARTIFACT_CID,
    EXPECTED_TOKENIZER_CID,
    RESULT_RELATIVE_PATH,
    TOKENIZER_RELATIVE_PATH,
    _decode_utf8,
    _record,
    _verify_arm_result,
)
from .position_kv_binding import (
    POLICY,
    R4_BLOCKS_PER_HEAD,
    R4PositionPreservingCausalKVBindingV1,
)


SCHEMA = "uor-r4.position-r4-language-generation/1"
STATUS = "GENERATED"
EXECUTION = "r4"
INTERVENTION = "native"
EXPECTED_GEOMETRY_ARTIFACT_CID = (
    "blake3:55447c00c1eb86a1d05324d6c83d044407bdc89f653f46957bf6f0bccb6c000b"
)
EXPECTED_GEOMETRY_FILE_CID = (
    "blake3:a812cf6749e637f4c486a6ad206b96c90d695b5c4bb2ed029df3c6bef147d702"
)
EXPECTED_H4_FRAME_ARTIFACT_CID = (
    "blake3:f1f556d3c93a2e21593c4f48de13efd64705fec11f7660e0b6fac7ba49263099"
)
EXPECTED_H4_FRAME_FILE_CID = (
    "blake3:9df624162d14ba133fed34c560e4828961a4dc8d6a9438c731e8f8c209c16ad4"
)


def _validate_step(output: Any, *, source_count: int) -> None:
    audit = output.audit
    materialized = LAYERS * HEADS * source_count
    expected_signature = (
        1,
        1,
        LAYERS,
        HEADS,
        LAYERS * 2 * HEADS * HEAD_DIM,
        materialized,
        materialized,
        materialized * 2 * R4_BLOCKS_PER_HEAD,
        materialized * HEAD_DIM,
        VOCAB_SIZE,
        0,
        1,
        0,
        0,
        0,
        0,
    )
    if (
        audit.execution != EXECUTION
        or audit.intervention != INTERVENTION
        or audit.work_signature() != expected_signature
        or output.final_state.length != source_count
        or output.final_state.audit.token_steps != source_count
        or tuple(output.logits.shape) != (1, VOCAB_SIZE)
        or tuple(output.attention_weights.shape)
        != (LAYERS, 1, HEADS, CONTEXT)
        or not bool(torch.isfinite(output.logits).all())
        or not bool(torch.isfinite(output.attention_weights).all())
    ):
        raise RuntimeError("generation left the native position-preserving R4 path")
    _validate_cache_state(output.final_state, source_count=source_count)


def _expected_cumulative_signature(source_count: int) -> tuple[int, ...]:
    materialized = LAYERS * HEADS * source_count * (source_count + 1) // 2
    return (
        1,
        source_count,
        LAYERS,
        HEADS,
        source_count * LAYERS * 2 * HEADS * HEAD_DIM,
        materialized,
        materialized,
        materialized * 2 * R4_BLOCKS_PER_HEAD,
        materialized * HEAD_DIM,
        source_count * VOCAB_SIZE,
        0,
        source_count,
        0,
        0,
        0,
        0,
    )


def _validate_cache_state(state: Any, *, source_count: int) -> None:
    expected_valid = torch.arange(CONTEXT) < source_count
    if (
        state.length != source_count
        or state.audit.work_signature()
        != _expected_cumulative_signature(source_count)
        or not torch.equal(
            state.valid.cpu(),
            expected_valid.view(1, 1, CONTEXT).expand(LAYERS, 1, -1),
        )
        or not bool(torch.isfinite(state.keys).all())
        or not bool(torch.isfinite(state.values).all())
    ):
        raise RuntimeError("position-preserving R4 cache ledger differs")


def _validate_prime(output: Any, *, source_count: int) -> None:
    if (
        output.loss is not None
        or output.audit.work_signature()
        != _expected_cumulative_signature(source_count)
        or tuple(output.logits.shape) != (1, source_count, VOCAB_SIZE)
        or tuple(output.attention_weights.shape)
        != (LAYERS, 1, HEADS, source_count, CONTEXT)
        or not bool(torch.isfinite(output.logits).all())
        or not bool(torch.isfinite(output.attention_weights).all())
    ):
        raise RuntimeError("prompt left the native position-preserving R4 path")
    _validate_cache_state(output.final_state, source_count=source_count)


def generate_position_r4_language_path(
    root: Path,
    *,
    geometry_path: Path,
    frame_path: Path,
    prompt: str,
    max_new_tokens: int = DEFAULT_MAX_NEW_TOKENS,
    seed: int = DEFAULT_SEED,
) -> dict[str, Any]:
    """Generate one continuation through chronological H4-transported K/V."""

    if not isinstance(prompt, str) or not prompt:
        raise ValueError("prompt must be a nonempty string")
    if isinstance(max_new_tokens, bool) or not isinstance(max_new_tokens, int):
        raise TypeError("max_new_tokens must be an integer")
    if max_new_tokens < 1:
        raise ValueError("max_new_tokens must be positive")

    resolved_root = root.expanduser().resolve()
    resolved_geometry = geometry_path.expanduser().resolve()
    resolved_frames = frame_path.expanduser().resolve()
    tokenizer_path = resolved_root / TOKENIZER_RELATIVE_PATH
    artifact_path = resolved_root / ARTIFACT_RELATIVE_PATH
    result_path = resolved_root / RESULT_RELATIVE_PATH

    tokenizer_payload = tokenizer_path.read_bytes()
    artifact_payload = artifact_path.read_bytes()
    result_payload = result_path.read_bytes()
    geometry_payload = resolved_geometry.read_bytes()
    frame_payload = resolved_frames.read_bytes()
    if _record(tokenizer_path, tokenizer_payload)["cid"] != EXPECTED_TOKENIZER_CID:
        raise ValueError("language-path tokenizer differs from the fitted control")
    if _record(artifact_path, artifact_payload)["cid"] != EXPECTED_ARTIFACT_CID:
        raise ValueError("ordinary artifact differs from the completed control")
    if (
        _record(resolved_geometry, geometry_payload)["cid"]
        != EXPECTED_GEOMETRY_FILE_CID
    ):
        raise ValueError("position-R4 geometry differs from the validated sidecar")
    if (
        _record(resolved_frames, frame_payload)["cid"]
        != EXPECTED_H4_FRAME_FILE_CID
    ):
        raise ValueError("position-R4 frame file differs from the validated sidecar")

    arm_result = json.loads(result_payload)
    if not isinstance(arm_result, dict):
        raise ValueError("ordinary arm result must be a JSON object")
    _verify_arm_result(arm_result, artifact_payload=artifact_payload)

    tokenizer = Tokenizer.from_file(str(tokenizer_path))
    if tokenizer.get_vocab_size(with_added_tokens=True) != VOCAB_SIZE:
        raise ValueError("language-path tokenizer vocabulary differs")
    raw_decoder = ByteLevelRawDecoder.from_tokenizer_json(tokenizer_path)
    prompt_token_ids = list(tokenizer.encode(prompt, add_special_tokens=False).ids)
    if not prompt_token_ids:
        raise ValueError("prompt produced no tokenizer IDs")
    if any(token < 0 or token >= VOCAB_SIZE for token in prompt_token_ids):
        raise ValueError("prompt produced an out-of-vocabulary token ID")
    if raw_decoder.decode_bytes(prompt_token_ids) != prompt.encode("utf-8"):
        raise ValueError("prompt does not round-trip through the language tokenizer")

    input_token_ids = [BOS_TOKEN_ID, *prompt_token_ids]
    processed_ceiling = len(input_token_ids) + max_new_tokens - 1
    if processed_ceiling > CONTEXT:
        raise ValueError(
            "prompt plus generated prefix exceeds the model's 120-token context"
        )

    geometry_bundle = load_group_geometry_artifacts(resolved_geometry)
    if (
        geometry_bundle.geometry_file_cid != EXPECTED_GEOMETRY_FILE_CID
        or geometry_bundle.artifact_cid != EXPECTED_GEOMETRY_ARTIFACT_CID
    ):
        raise ValueError("position-R4 geometry artifact identity differs")
    frames = H4SpinFrameArtifactV1.from_bytes(frame_payload)
    if (
        frames.file_cid != EXPECTED_H4_FRAME_FILE_CID
        or frames.artifact_cid != EXPECTED_H4_FRAME_ARTIFACT_CID
        or frames.transport_control_source_cid != geometry_bundle.artifact_cid
    ):
        raise ValueError("position-R4 frame artifact identity differs")
    model = R4PositionPreservingCausalKVBindingV1.from_learned_artifact(
        artifact_payload,
        geometry=geometry_bundle.exact_h4,
        frames=frames,
    )
    model.to(device=torch.device("cpu"), dtype=torch.float32)
    model.eval()
    torch.use_deterministic_algorithms(True)
    if (
        model.parameter_count() != PARAMETER_COUNT
        or model.export_learned_artifact() != artifact_payload
        or model.geometry_artifact_cid != EXPECTED_GEOMETRY_ARTIFACT_CID
        or model.frame_artifact_cid != EXPECTED_H4_FRAME_ARTIFACT_CID
    ):
        raise RuntimeError("position-R4 wrapper changed the bound model artifact")

    sampler = SplitMix64(seed)
    generated: list[int] = []
    stop: str | dict[str, Any] = "maximum_new_tokens"

    with torch.inference_mode():
        prime = model.forward_incremental(
            torch.tensor([input_token_ids], dtype=torch.long),
            execution=EXECUTION,
            intervention=INTERVENTION,
        )
        _validate_prime(prime, source_count=len(input_token_ids))
        state = prime.final_state
        logits = prime.logits[0, -1].detach().cpu().to(torch.float32).contiguous()
        for decision in range(max_new_tokens):
            token = int(sample_top_k_q32(logits, sampler))
            if not 0 <= token < VOCAB_SIZE:
                raise RuntimeError("sampler selected a token outside the vocabulary")
            generated.append(token)
            if token == EOS_TOKEN_ID:
                stop = "eos"
                break
            period = short_cycle_period(generated)
            if period is not None:
                stop = {"short_cycle": {"period": period}}
                break
            if decision + 1 == max_new_tokens:
                break
            step = model.step(
                torch.tensor([token], dtype=torch.long),
                state,
                execution=EXECUTION,
                intervention=INTERVENTION,
            )
            _validate_step(step, source_count=state.length + 1)
            state = step.final_state
            logits = step.logits[0].detach().cpu().to(torch.float32).contiguous()

    response_ids = (
        generated[:-1]
        if generated and generated[-1] == EOS_TOKEN_ID
        else generated
    )
    continuation, utf8_decodable = _decode_utf8(
        raw_decoder.decode_bytes(response_ids)
    )
    audit = state.audit

    return {
        "schema": SCHEMA,
        "status": STATUS,
        "model": {
            "policy": POLICY,
            "execution": EXECUTION,
            "intervention": INTERVENTION,
            "parameter_count": PARAMETER_COUNT,
            "context_tokens": CONTEXT,
            "cache_state_values": STATE_VALUES,
            "cache_state_bytes_f32": STATE_BYTES_F32,
            "validity_bits": VALIDITY_BITS,
        },
        "inputs": {
            "model_artifact": _record(artifact_path, artifact_payload),
            "artifact_result": _record(result_path, result_payload),
            "tokenizer": _record(tokenizer_path, tokenizer_payload),
            "geometry": _record(resolved_geometry, geometry_payload),
            "h4_frames": _record(resolved_frames, frame_payload),
            "corpus_files_read": 0,
            "teacher_files_read": 0,
            "checkpoint_files_read": 0,
            "historical_fitted_artifact_files_read": 0,
            "provider_files_read": 0,
        },
        "fit_provenance": {
            "arm_result_cid": arm_result["arm_result_cid"],
            "completed_steps": arm_result.get("completed_steps"),
            "presentations": arm_result.get("presentations"),
            "train_order_cid": arm_result.get("train_order_cid"),
        },
        "geometry": {
            "group_artifact_cid": geometry_bundle.artifact_cid,
            "frame_artifact_cid": frames.artifact_cid,
            "frame_matrix_convention": frames.matrix_convention,
            "identity_index": frames.identity_index,
        },
        "execution": {
            "implementation": "incremental-position-kv-cache",
            "step_calls": audit.token_steps,
            "generation_decisions": len(generated),
            "processed_tokens": state.length,
            "final_frame_index": int(state.current_frame_indices[0]),
            "forbidden_reads": audit.forbidden_reads,
            "future_reads": audit.future_reads,
            "provider_calls": audit.provider_calls,
            "teacher_calls": audit.teacher_calls,
            "work": {
                "cache_writes": audit.cache_writes,
                "materialized_attention_scores": audit.materialized_attention_scores,
                "admitted_attention_scores": audit.admitted_attention_scores,
                "transported_r4_blocks": audit.transported_r4_blocks,
                "value_reads": audit.value_reads,
                "vocabulary_scores": audit.vocabulary_scores,
            },
        },
        "prompt": prompt,
        "prompt_token_ids": prompt_token_ids,
        "seed": seed,
        "sampler": {
            "type": "top-k-q32-splitmix64",
            "top_k": TOP_K,
            "temperature": TEMPERATURE,
        },
        "max_new_tokens": max_new_tokens,
        "generated_token_ids": generated,
        "continuation": continuation,
        "text": prompt + continuation,
        "utf8_decodable": utf8_decodable,
        "stop": stop,
    }


__all__ = [
    "DEFAULT_MAX_NEW_TOKENS",
    "DEFAULT_SEED",
    "SCHEMA",
    "STATUS",
    "generate_position_r4_language_path",
]
