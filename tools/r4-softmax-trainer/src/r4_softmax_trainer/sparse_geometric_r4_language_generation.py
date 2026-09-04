"""Artifact-only generation through sparse geometric recurrent R4/H4 memory."""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any

import torch
from tokenizers import Tokenizer

from .fixed_recurrent_kv_binding import (
    LIVE_WINDOW,
    RECURRENT_METADATA_I64_VALUES,
    RECURRENT_STATE_BYTES_F32,
    RECURRENT_STATE_VALUES,
    SUMMARY_BANKS,
)
from .group_retention_campaign import load_group_geometry_artifacts
from .h4_spin_frame_sidecar import H4SpinFrameArtifactV1
from .language_path_generalization import (
    CONTEXT,
    PARAMETER_COUNT,
    STATE_BYTES_F32,
    STATE_VALUES,
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
from .position_r4_language_generation import (
    EXPECTED_GEOMETRY_ARTIFACT_CID,
    EXPECTED_GEOMETRY_FILE_CID,
    EXPECTED_H4_FRAME_ARTIFACT_CID,
    EXPECTED_H4_FRAME_FILE_CID,
)
from .sparse_geometric_kv_binding import (
    MAXIMUM_READ_SOURCES,
    PERSISTENT_CANDIDATE_BUDGET,
    POLICY,
    SIGNED_S3_SHELL_DEGREES,
    R4SparseGeometricCandidateSoftmaxKVBindingV1,
)


SCHEMA = "uor-r4.sparse-geometric-r4-language-generation/1"
STATUS = "GENERATED"


def _validate_state(
    model: R4SparseGeometricCandidateSoftmaxKVBindingV1,
    state: Any,
) -> None:
    state_values = (
        state.live_keys.numel()
        + state.live_values.numel()
        + state.summary_keys_local.numel()
        + state.summary_values_local.numel()
    )
    represented = int(state.summary_counts[0].sum()) + state.live_length
    if (
        state_values != RECURRENT_STATE_VALUES
        or model.recurrent_state_value_count() != RECURRENT_STATE_VALUES
        or model.recurrent_state_byte_count_f32() != RECURRENT_STATE_BYTES_F32
        or represented != state.tokens_seen
        or state.audit.policy != POLICY
        or state.audit.peak_attention_source_slots > MAXIMUM_READ_SOURCES
        or state.audit.unselected_key_value_reads != 0
        or state.audit.complete_prefix_scans != 0
        or state.audit.provider_calls != 0
        or state.audit.teacher_calls != 0
        or state.audit.future_reads != 0
        or state.audit.forbidden_reads != 0
    ):
        raise RuntimeError("generation left the sparse geometric R4 state contract")


def generate_sparse_geometric_r4_language_path(
    root: Path,
    *,
    geometry_path: Path,
    frame_path: Path,
    prompt: str,
    max_new_tokens: int = DEFAULT_MAX_NEW_TOKENS,
    seed: int = DEFAULT_SEED,
) -> dict[str, Any]:
    """Generate once with unchanged weights and bounded H4 candidate admission."""

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
        raise ValueError("sparse-R4 geometry differs from the validated sidecar")
    if (
        _record(resolved_frames, frame_payload)["cid"]
        != EXPECTED_H4_FRAME_FILE_CID
    ):
        raise ValueError("sparse-R4 frame file differs from the validated sidecar")

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
            "prompt plus generated prefix exceeds the model's trained "
            "120-position RoPE context"
        )

    geometry_bundle = load_group_geometry_artifacts(resolved_geometry)
    if (
        geometry_bundle.geometry_file_cid != EXPECTED_GEOMETRY_FILE_CID
        or geometry_bundle.artifact_cid != EXPECTED_GEOMETRY_ARTIFACT_CID
    ):
        raise ValueError("sparse-R4 geometry artifact identity differs")
    frames = H4SpinFrameArtifactV1.from_bytes(frame_payload)
    if (
        frames.file_cid != EXPECTED_H4_FRAME_FILE_CID
        or frames.artifact_cid != EXPECTED_H4_FRAME_ARTIFACT_CID
        or frames.transport_control_source_cid != geometry_bundle.artifact_cid
    ):
        raise ValueError("sparse-R4 frame artifact identity differs")

    model = R4SparseGeometricCandidateSoftmaxKVBindingV1.from_learned_artifact(
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
        raise RuntimeError("sparse geometric wrapper changed the bound artifact")

    sampler = SplitMix64(seed)
    generated: list[int] = []
    candidate_trace: list[dict[str, Any]] = []
    stop: str | dict[str, Any] = "maximum_new_tokens"
    initial_state_values = model.recurrent_state_value_count()

    with torch.inference_mode():
        state = model.initial_state(1)
        logits: torch.Tensor | None = None
        for token_id in input_token_ids:
            step = model.step(torch.tensor([token_id], dtype=torch.long), state)
            if step.candidate_selection is None:
                raise RuntimeError("sparse step omitted its candidate selection")
            candidate_trace.append(
                model.describe_candidate_selection(step.candidate_selection)
            )
            state = step.final_state
            logits = step.logits[0].detach().cpu().to(torch.float32).contiguous()
        if logits is None:
            raise RuntimeError("generation did not process a causal input token")

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
            step = model.step(torch.tensor([token], dtype=torch.long), state)
            if step.candidate_selection is None:
                raise RuntimeError("sparse step omitted its candidate selection")
            candidate_trace.append(
                model.describe_candidate_selection(step.candidate_selection)
            )
            state = step.final_state
            logits = step.logits[0].detach().cpu().to(torch.float32).contiguous()

    _validate_state(model, state)
    if model.recurrent_state_value_count() != initial_state_values:
        raise RuntimeError(
            "sparse recurrent state storage changed with sequence length"
        )
    if len(candidate_trace) != state.tokens_seen:
        raise RuntimeError("candidate trace does not cover every processed token")

    response_ids = (
        generated[:-1]
        if generated and generated[-1] == EOS_TOKEN_ID
        else generated
    )
    continuation, utf8_decodable = _decode_utf8(
        raw_decoder.decode_bytes(response_ids)
    )
    audit = state.audit
    represented_summary_tokens = int(state.summary_counts[0].sum())
    first_summary_read_position = next(
        (
            trace["position"]
            for trace in candidate_trace
            if any(
                source["source_kind"] == "summary"
                for source in trace["admitted"]
            )
        ),
        None,
    )

    return {
        "schema": SCHEMA,
        "status": STATUS,
        "model": {
            "policy": POLICY,
            "execution": "r4",
            "intervention": "native",
            "parameter_count": PARAMETER_COUNT,
            "trained_rope_context_tokens": CONTEXT,
            "live_window_tokens": LIVE_WINDOW,
            "summary_banks": SUMMARY_BANKS,
            "persistent_candidate_budget": PERSISTENT_CANDIDATE_BUDGET,
            "maximum_attention_sources": MAXIMUM_READ_SOURCES,
            "recurrent_state_values": RECURRENT_STATE_VALUES,
            "recurrent_state_bytes_f32": RECURRENT_STATE_BYTES_F32,
            "recurrent_metadata_i64_values": RECURRENT_METADATA_I64_VALUES,
            "exact_cache_state_values": STATE_VALUES,
            "exact_cache_state_bytes_f32": STATE_BYTES_F32,
            "f32_state_byte_reduction_fraction": (
                1.0 - RECURRENT_STATE_BYTES_F32 / STATE_BYTES_F32
            ),
        },
        "compression": {
            "partition": "chronological-binary-age-levels",
            "selection": "exact-signed-s3-shell-then-maximin-full-h4-root",
            "selection_status": "unfitted-engineering-hypothesis",
            "signed_s3_shell_degrees": list(SIGNED_S3_SHELL_DEGREES),
            "summary_coordinates": "local-h4-frame",
            "merge": "count-weighted-key-value-mean-in-newer-anchor",
            "highest_bank": "absorbs-overflow",
            "score_law": "unchanged-qk-softmax-over-admitted-sources",
            "selection_inputs": "fixed-recurrent-source-metadata-only",
            "exact_h4_heatmap_role": "trace-only-not-admission",
            "group_address_collision": False,
            "learned_gate": False,
            "fit_performed": False,
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
            "implementation": "sparse-geometric-fixed-recurrent-r4-kv-cache",
            "read_before_persistent_write": True,
            "processed_tokens": state.tokens_seen,
            "generation_decisions": len(generated),
            "final_live_length": state.live_length,
            "final_summary_counts": state.summary_counts[0].tolist(),
            "final_summary_last_positions": (
                state.summary_last_positions[0].tolist()
            ),
            "represented_summary_tokens": represented_summary_tokens,
            "represented_total_tokens": represented_summary_tokens
            + state.live_length,
            "first_eviction_after_position": (
                LIVE_WINDOW if audit.evictions else None
            ),
            "first_summary_read_position": first_summary_read_position,
            "state_values_initial": initial_state_values,
            "state_values_final": model.recurrent_state_value_count(),
            "state_storage_constant": True,
            "final_frame_index": int(state.current_frame_indices[0]),
            "provider_calls": audit.provider_calls,
            "teacher_calls": audit.teacher_calls,
            "future_reads": audit.future_reads,
            "forbidden_reads": audit.forbidden_reads,
            "candidate_trace": candidate_trace,
            "work": {
                "cache_writes": audit.cache_writes,
                "evictions": audit.evictions,
                "summary_bank_updates": audit.summary_bank_updates,
                "summary_merges": audit.summary_merges,
                "summary_slots_read": audit.summary_slots_read,
                "materialized_attention_scores": (
                    audit.materialized_attention_scores
                ),
                "admitted_attention_scores": audit.admitted_attention_scores,
                "live_attention_scores": audit.live_attention_scores,
                "summary_attention_scores": audit.summary_attention_scores,
                "current_attention_scores": audit.current_attention_scores,
                "eligible_persistent_source_slots": (
                    audit.eligible_persistent_source_slots
                ),
                "selected_persistent_source_slots": (
                    audit.selected_persistent_source_slots
                ),
                "geometric_shell_evaluations": audit.geometric_shell_evaluations,
                "pairwise_shell_evaluations": audit.pairwise_shell_evaluations,
                "candidate_cost_comparisons": audit.candidate_cost_comparisons,
                "unselected_key_value_reads": audit.unselected_key_value_reads,
                "complete_prefix_scans": audit.complete_prefix_scans,
                "attention_transported_r4_blocks": (
                    audit.attention_transported_r4_blocks
                ),
                "compression_transported_r4_blocks": (
                    audit.compression_transported_r4_blocks
                ),
                "value_reads": audit.value_reads,
                "vocabulary_scores": audit.vocabulary_scores,
                "peak_attention_source_slots": (
                    audit.peak_attention_source_slots
                ),
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
    "generate_sparse_geometric_r4_language_path",
]
