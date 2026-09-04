"""Bounded open-data fit for #973's sparse quaternion-cube language path."""

from __future__ import annotations

import json
import math
import os
import platform
import resource
import time
from pathlib import Path
from typing import Any, Iterable

import torch
from tokenizers import Tokenizer

from .fixed_recurrent_kv_binding import (
    RECURRENT_STATE_BYTES_F32,
    RecurrentNonlinearAudit,
)
from .group_retention_campaign import load_group_geometry_artifacts
from .h4_spin_frame_sidecar import H4SpinFrameArtifactV1
from .language_path_generalization import CONTEXT, PARAMETER_COUNT, VOCAB_SIZE
from .language_path_generalization_campaign import (
    ADAM_BETA1,
    ADAM_BETA2,
    ADAM_EPSILON,
    GRADIENT_CLIP,
    WEIGHT_DECAY,
    learning_rate,
)
from .language_path_generalization_data import (
    EXPECTED_TRAIN_SLICE_CID,
    TRAIN_WINDOWS,
    LanguagePathWindowStore,
    deterministic_window_order,
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
    EXPECTED_ARTIFACT_CID,
    EXPECTED_TOKENIZER_CID,
    RESULT_RELATIVE_PATH,
    TOKENIZER_RELATIVE_PATH,
    _decode_utf8,
    _verify_arm_result,
)
from .position_r4_language_generation import (
    EXPECTED_GEOMETRY_ARTIFACT_CID,
    EXPECTED_GEOMETRY_FILE_CID,
    EXPECTED_H4_FRAME_ARTIFACT_CID,
    EXPECTED_H4_FRAME_FILE_CID,
)
from .provenance import (
    atomic_write,
    atomic_write_json,
    canonical_json_bytes,
    cid_bytes,
    cid_file,
)
from .quaternion_cube_nonlinear import (
    ACTIVE_PARAMETER_VALUES,
    POLICY as QUATERNION_CUBE_POLICY,
    RETAINED_DENSE_MLP_PARAMETER_VALUES,
    R4H4FrameQuaternionCubeResidualV1,
)
from .sparse_geometric_kv_binding import (
    MAXIMUM_READ_SOURCES,
    POLICY as SPARSE_DENSE_POLICY,
    R4SparseGeometricCandidateSoftmaxKVBindingV1,
)


SCHEMA = "uor-r4.sparse-quaternion-cube-development-fit/1"
STATUS_PASS = "SPARSE_QUATERNION_CUBE_BOUNDED_DEVELOPMENT_PASS"
STATUS_NEGATIVE = "SPARSE_QUATERNION_CUBE_BOUNDED_DEVELOPMENT_NEGATIVE"
OUTPUT_DIRECTORY_RELATIVE_PATH = Path("arms/quaternion-cube-development-fit")
TRAIN_RELATIVE_PATH = Path("data/train.u16")
GEOMETRY_RELATIVE_PATH = Path("geometry/r4-group-address-geometry.json")
BATCH_SIZE = 16
THREADS = 4
MAX_UPDATES = 128
DEFAULT_UPDATES = MAX_UPDATES
MAX_SECONDS = 840.0
POST_FIT_SECONDS_RESERVE = 90.0
PEAK_RSS_LIMIT_BYTES = 4 * 1024**3
OUTPUT_LIMIT_BYTES = 8 * 1024**2
MONITOR_BATCH_INDICES = (0, 32, 64, 96)
MONITOR_WINDOWS = len(MONITOR_BATCH_INDICES) * BATCH_SIZE
TRAIN_NLL_REQUIRED_IMPROVEMENT = 0.10
COMPETITIVE_NLL_TOLERANCE = 0.20
COMPETITIVE_TOP1_TOLERANCE = 0.02
ANCHOR_TOP_K = 8
PROMPT_COMMON_PREFIX_REQUIRED = 4
GENERATION_SEED = 9_738
GENERATION_MAX_NEW_TOKENS = 16
PROMPTS = (
    ("turtle", "A purple turtle found a clock in the garden"),
    ("einstein", "Albert Einstein was born in"),
)
ANCHORS = (
    {
        "name": "squirrel",
        "window_ordinal": 304,
        "expected_order_rank": 12,
        "prompt": "One day, a squirrel named Sam found a big nut. He was",
        "gold_token_ids": (381, 378, 16),
    },
    {
        "name": "blouse",
        "window_ordinal": 34_916,
        "expected_order_rank": 2,
        "prompt": "One day, a girl named Sue wore her favorite blouse. It was",
        "gold_token_ids": (677, 331, 348, 3690, 16),
    },
)


def _input_record(path: Path, expected_cid: str) -> dict[str, Any]:
    if path.is_symlink() or not path.is_file():
        raise ValueError(f"fit input must be a regular non-symlink file: {path}")
    observed_cid = cid_file(path)
    if observed_cid != expected_cid:
        raise ValueError(f"fit input differs from its accepted identity: {path}")
    return {"path": str(path), "bytes": path.stat().st_size, "cid": observed_cid}


def _write_output_bundle(
    output_directory: Path,
    artifact: bytes,
    result: dict[str, Any],
) -> None:
    """Publish the model and result together through one directory rename."""

    staging_directory = output_directory.with_name(f".{output_directory.name}.tmp")
    if output_directory.exists() or staging_directory.exists():
        raise FileExistsError(
            "quaternion-cube canonical or staged output already exists: "
            f"{output_directory}"
        )
    staging_directory.mkdir(parents=True, exist_ok=False)
    atomic_write(staging_directory / "model.safetensors", artifact)
    atomic_write_json(staging_directory / "fit.json", result)
    os.replace(staging_directory, output_directory)


def _configure_cpu(threads: int) -> dict[str, Any]:
    if threads != THREADS:
        raise ValueError("quaternion-cube fit requires exactly four CPU threads")
    os.environ["OMP_NUM_THREADS"] = str(threads)
    os.environ["VECLIB_MAXIMUM_THREADS"] = str(threads)
    os.environ["OPENBLAS_NUM_THREADS"] = str(threads)
    torch.set_num_threads(threads)
    try:
        torch.set_num_interop_threads(threads)
    except RuntimeError:
        if torch.get_num_interop_threads() != threads:
            raise
    if torch.get_num_threads() != threads or torch.get_num_interop_threads() != threads:
        raise RuntimeError("quaternion-cube fit did not acquire four CPU threads")
    torch.manual_seed(GENERATION_SEED)
    torch.use_deterministic_algorithms(True)
    return {
        "device": "cpu",
        "dtype": "float32",
        "platform": platform.system(),
        "threads": threads,
        "deterministic_algorithms": True,
    }


def _peak_rss_bytes() -> int:
    observed = int(resource.getrusage(resource.RUSAGE_SELF).ru_maxrss)
    return observed if platform.system() == "Darwin" else observed * 1024


def _require_resources(started: float, max_seconds: float) -> None:
    if time.monotonic() - started >= max_seconds:
        raise TimeoutError("quaternion-cube fit reached its whole-process wall")
    if _peak_rss_bytes() >= PEAK_RSS_LIMIT_BYTES:
        raise MemoryError("quaternion-cube fit reached its peak-RSS limit")


def _gradient_census(
    model: R4H4FrameQuaternionCubeResidualV1,
) -> list[dict[str, Any]]:
    records: list[dict[str, Any]] = []
    missing: list[str] = []
    unexpected: list[str] = []
    for name, parameter in model.named_parameters():
        gradient = parameter.grad
        if parameter.requires_grad:
            if (
                gradient is None
                or not bool(torch.isfinite(gradient).all())
                or not bool(torch.count_nonzero(gradient))
            ):
                missing.append(name)
                continue
            records.append(
                {
                    "name": name,
                    "values": parameter.numel(),
                    "nonzero_gradient_values": int(torch.count_nonzero(gradient)),
                    "gradient_l2": float(torch.linalg.vector_norm(gradient.float())),
                }
            )
        elif gradient is not None:
            unexpected.append(name)
    if missing or unexpected:
        raise RuntimeError(
            "quaternion-cube gradient contract failed: "
            f"missing_or_zero={missing}, frozen_with_gradients={unexpected}"
        )
    if len(records) != len(model.trainable_parameter_names()):
        raise RuntimeError("quaternion-cube gradient census omitted an active tensor")
    return records


def _state_is_finite(state: Any) -> bool:
    return all(
        bool(torch.isfinite(getattr(state, field)).all())
        for field in (
            "live_keys",
            "live_values",
            "summary_keys_local",
            "summary_values_local",
        )
    )


def _numerical_record(audit: RecurrentNonlinearAudit) -> dict[str, Any]:
    minimum = audit.minimum_positive_block_norm_squared
    if not math.isfinite(minimum):
        raise RuntimeError("quaternion-cube execution observed no positive R4 block norm")
    values = (
        audit.maximum_block_norm_error,
        audit.maximum_residual_bound_ratio,
        minimum,
        audit.maximum_block_inverse_norm_squared,
    )
    if not all(math.isfinite(value) for value in values):
        raise RuntimeError("quaternion-cube numerical audit is nonfinite")
    return {
        "exact_zero_r4_blocks": audit.exact_zero_r4_blocks,
        "minimum_positive_block_norm_squared": minimum,
        "maximum_block_inverse_norm_squared": audit.maximum_block_inverse_norm_squared,
        "maximum_block_norm_error": audit.maximum_block_norm_error,
        "maximum_residual_bound_ratio": audit.maximum_residual_bound_ratio,
    }


def _selector_record(audit: Any) -> dict[str, int]:
    return {
        "token_steps": audit.token_steps,
        "materialized_attention_scores": audit.materialized_attention_scores,
        "selected_persistent_source_slots": audit.selected_persistent_source_slots,
        "eligible_persistent_source_slots": audit.eligible_persistent_source_slots,
        "geometric_shell_evaluations": audit.geometric_shell_evaluations,
        "pairwise_shell_evaluations": audit.pairwise_shell_evaluations,
        "candidate_cost_comparisons": audit.candidate_cost_comparisons,
        "peak_attention_source_slots": audit.peak_attention_source_slots,
        "complete_prefix_scans": audit.complete_prefix_scans,
        "unselected_key_value_reads": audit.unselected_key_value_reads,
        "provider_calls": audit.provider_calls,
        "teacher_calls": audit.teacher_calls,
        "future_reads": audit.future_reads,
        "forbidden_reads": audit.forbidden_reads,
    }


def _mechanics_pass(selector: dict[str, int]) -> bool:
    return (
        selector["peak_attention_source_slots"] <= MAXIMUM_READ_SOURCES
        and selector["complete_prefix_scans"] == 0
        and selector["unselected_key_value_reads"] == 0
        and selector["provider_calls"] == 0
        and selector["teacher_calls"] == 0
        and selector["future_reads"] == 0
        and selector["forbidden_reads"] == 0
    )


def _full_context_backward_gate(
    model: R4H4FrameQuaternionCubeResidualV1,
    store: LanguagePathWindowStore,
    ordinal: int,
) -> dict[str, Any]:
    inputs, targets = store.batch((ordinal,))
    model.train()
    model.zero_grad(set_to_none=True)
    output = model(inputs, targets)
    if (
        output.loss is None
        or not bool(torch.isfinite(output.loss))
        or not bool(torch.isfinite(output.logits).all())
        or not _state_is_finite(output.final_state)
    ):
        raise RuntimeError("full-context quaternion-cube backward gate is nonfinite")
    output.loss.backward()
    gradients = _gradient_census(model)
    selector = _selector_record(output.audit)
    if output.audit.evictions == 0 or output.audit.summary_bank_updates == 0:
        raise RuntimeError("full-context backward gate did not exercise recurrent folding")
    if not _mechanics_pass(selector) or output.nonlinear_audit.dense_mlp_calls != 0:
        raise RuntimeError("full-context backward gate left the assembled architecture")
    result = {
        "batch_size": 1,
        "context_tokens": CONTEXT,
        "optimizer_steps": 0,
        "loss_nats": float(output.loss.detach()),
        "active_gradient_tensors": len(gradients),
        "active_gradient_values": sum(record["values"] for record in gradients),
        "gradients": gradients,
        "selector": selector,
        "recurrent_evictions": output.audit.evictions,
        "summary_bank_updates": output.audit.summary_bank_updates,
        "numerical": _numerical_record(output.nonlinear_audit),
    }
    model.zero_grad(set_to_none=True)
    return result


def _evaluate(
    model: torch.nn.Module,
    store: LanguagePathWindowStore,
    ordinals: tuple[int, ...],
) -> dict[str, Any]:
    total_loss = 0.0
    total_rows = 0
    top1_correct = 0
    aggregate_audit: Any | None = None
    aggregate_nonlinear: RecurrentNonlinearAudit | None = None
    model.eval()
    with torch.inference_mode():
        for start in range(0, len(ordinals), BATCH_SIZE):
            inputs, targets = store.batch(ordinals[start : start + BATCH_SIZE])
            output = model(inputs, targets)
            if (
                output.loss is None
                or not bool(torch.isfinite(output.loss))
                or not bool(torch.isfinite(output.logits).all())
                or not _state_is_finite(output.final_state)
            ):
                raise RuntimeError("development monitor produced a nonfinite result")
            rows = targets.numel()
            total_loss += float(output.loss) * rows
            total_rows += rows
            top1_correct += int(
                torch.count_nonzero(output.logits.argmax(dim=-1) == targets)
            )
            aggregate_audit = (
                output.audit
                if aggregate_audit is None
                else aggregate_audit.accumulated_with(output.audit)
            )
            aggregate_nonlinear = (
                output.nonlinear_audit
                if aggregate_nonlinear is None
                else aggregate_nonlinear.accumulated_with(output.nonlinear_audit)
            )
    if aggregate_audit is None or aggregate_nonlinear is None or total_rows == 0:
        raise RuntimeError("development monitor was empty")
    selector = _selector_record(aggregate_audit)
    if not _mechanics_pass(selector):
        raise RuntimeError("development monitor left the sparse causal contract")
    result = {
        "windows": len(ordinals),
        "causal_targets": total_rows,
        "ce_nats": total_loss / total_rows,
        "top1_correct": top1_correct,
        "top1_rate": top1_correct / total_rows,
        "selector": selector,
        "nonlinear": {
            "policy": aggregate_nonlinear.policy,
            "layer_calls": aggregate_nonlinear.layer_calls,
            "dense_mlp_calls": aggregate_nonlinear.dense_mlp_calls,
            "dense_mlp_weight_products": aggregate_nonlinear.dense_mlp_weight_products,
            "r4_block_evaluations": aggregate_nonlinear.r4_block_evaluations,
        },
    }
    if aggregate_nonlinear.policy == QUATERNION_CUBE_POLICY:
        result["nonlinear"]["numerical"] = _numerical_record(aggregate_nonlinear)
    return result


def _anchor_location(
    window: list[int], prompt_ids: list[int], gold_ids: tuple[int, ...]
) -> int:
    matches = [
        offset
        for offset in range(len(window) - len(prompt_ids) - len(gold_ids) + 1)
        if window[offset : offset + len(prompt_ids)] == prompt_ids
        and tuple(
            window[offset + len(prompt_ids) : offset + len(prompt_ids) + len(gold_ids)]
        )
        == gold_ids
    ]
    if len(matches) != 1:
        raise RuntimeError("open anchor is not unique in its declared source window")
    return matches[0] + len(prompt_ids)


def _score_anchor(
    model: torch.nn.Module,
    window: list[int],
    *,
    gold_start: int,
    gold_ids: tuple[int, ...],
) -> dict[str, Any]:
    input_ids = window[: gold_start + len(gold_ids) - 1]
    if len(input_ids) > CONTEXT:
        raise RuntimeError("open anchor exceeds the trained context")
    tokens = torch.tensor([input_ids], dtype=torch.long)
    model.eval()
    with torch.inference_mode():
        output = model(tokens)
    first_scores = output.logits[0, gold_start - 1].float()
    first_gold = gold_ids[0]
    first_rank = 1 + int(torch.count_nonzero(first_scores > first_scores[first_gold]))
    suffix_logits = output.logits[
        0, gold_start - 1 : gold_start - 1 + len(gold_ids)
    ].float()
    suffix_targets = torch.tensor(gold_ids, dtype=torch.long)
    suffix_nll = torch.nn.functional.cross_entropy(suffix_logits, suffix_targets)
    selector = _selector_record(output.audit)
    if not _mechanics_pass(selector):
        raise RuntimeError("open anchor left the sparse causal contract")
    return {
        "gold_first_token_rank": first_rank,
        "argmax_token_id": int(first_scores.argmax()),
        "gold_suffix_ce_nats": float(suffix_nll),
        "selector": selector,
    }


def _score_anchors(
    model: torch.nn.Module,
    store: LanguagePathWindowStore,
    tokenizer: Tokenizer,
    order: tuple[int, ...],
) -> list[dict[str, Any]]:
    results: list[dict[str, Any]] = []
    for anchor in ANCHORS:
        ordinal = int(anchor["window_ordinal"])
        rank = int(anchor["expected_order_rank"])
        if order[rank] != ordinal:
            raise RuntimeError("open anchor changed deterministic training rank")
        prompt = str(anchor["prompt"])
        prompt_ids = list(tokenizer.encode(prompt, add_special_tokens=False).ids)
        window = [int(value) for value in store.window(ordinal)]
        gold_ids = tuple(int(value) for value in anchor["gold_token_ids"])
        gold_start = _anchor_location(window, prompt_ids, gold_ids)
        results.append(
            {
                "name": anchor["name"],
                "window_ordinal": ordinal,
                "training_order_rank": rank,
                "prompt": prompt,
                "gold_token_ids": list(gold_ids),
                "gold_start_position": gold_start,
                "score": _score_anchor(
                    model, window, gold_start=gold_start, gold_ids=gold_ids
                ),
            }
        )
    return results


def _generate(
    model: torch.nn.Module,
    tokenizer: Tokenizer,
    raw_decoder: ByteLevelRawDecoder,
    prompt: str,
) -> dict[str, Any]:
    prompt_ids = list(tokenizer.encode(prompt, add_special_tokens=False).ids)
    if not prompt_ids or raw_decoder.decode_bytes(prompt_ids) != prompt.encode("utf-8"):
        raise ValueError("development prompt does not round-trip through the tokenizer")
    inputs = [BOS_TOKEN_ID, *prompt_ids]
    if len(inputs) + GENERATION_MAX_NEW_TOKENS - 1 > CONTEXT:
        raise ValueError("development prompt exceeds the trained context")
    sampler = SplitMix64(GENERATION_SEED)
    generated: list[int] = []
    stop: str | dict[str, Any] = "maximum_new_tokens"
    nonlinear: RecurrentNonlinearAudit | None = None
    model.eval()
    with torch.inference_mode():
        state = model.initial_state(1)
        logits: torch.Tensor | None = None
        for token_id in inputs:
            output = model.step(torch.tensor([token_id], dtype=torch.long), state)
            state = output.final_state
            nonlinear = (
                output.nonlinear_audit
                if nonlinear is None
                else nonlinear.accumulated_with(output.nonlinear_audit)
            )
            logits = output.logits[0].detach().float().cpu()
        if logits is None:
            raise RuntimeError("development generation processed no input")
        for decision in range(GENERATION_MAX_NEW_TOKENS):
            token = int(sample_top_k_q32(logits, sampler))
            generated.append(token)
            if token == EOS_TOKEN_ID:
                stop = "eos"
                break
            period = short_cycle_period(generated)
            if period is not None:
                stop = {"short_cycle": {"period": period}}
                break
            if decision + 1 == GENERATION_MAX_NEW_TOKENS:
                break
            output = model.step(torch.tensor([token], dtype=torch.long), state)
            state = output.final_state
            nonlinear = (
                output.nonlinear_audit
                if nonlinear is None
                else nonlinear.accumulated_with(output.nonlinear_audit)
            )
            logits = output.logits[0].detach().float().cpu()
    if nonlinear is None:
        raise RuntimeError("development generation omitted nonlinear execution")
    response_ids = generated[:-1] if generated and generated[-1] == EOS_TOKEN_ID else generated
    continuation, utf8_decodable = _decode_utf8(raw_decoder.decode_bytes(response_ids))
    unknown_ids = {
        token_id
        for token in ("<unk>", "<|unk|>")
        if (token_id := tokenizer.token_to_id(token)) is not None
    }
    clean_text = (
        bool(response_ids)
        and bool(continuation.strip())
        and utf8_decodable
        and "\ufffd" not in continuation
        and not any(token in unknown_ids for token in response_ids)
        and not any(
            ord(character) < 32 and character not in "\n\r\t"
            for character in continuation
        )
    )
    selector = _selector_record(state.audit)
    if not _mechanics_pass(selector):
        raise RuntimeError("development generation left the sparse causal contract")
    result = {
        "prompt": prompt,
        "prompt_token_ids": prompt_ids,
        "seed": GENERATION_SEED,
        "top_k": TOP_K,
        "temperature": TEMPERATURE,
        "maximum_new_tokens": GENERATION_MAX_NEW_TOKENS,
        "generated_token_ids": generated,
        "continuation": continuation,
        "utf8_decodable": utf8_decodable,
        "clean_text": clean_text,
        "stop": stop,
        "selector": selector,
        "nonlinear_policy": nonlinear.policy,
    }
    if nonlinear.policy == QUATERNION_CUBE_POLICY:
        result["numerical"] = _numerical_record(nonlinear)
    return result


def _common_prefix(left: Iterable[int], right: Iterable[int]) -> int:
    count = 0
    for left_value, right_value in zip(left, right):
        if left_value != right_value:
            break
        count += 1
    return count


def fit_quaternion_cube_r4_language_path(
    root: Path,
    *,
    frame_path: Path,
    updates: int = DEFAULT_UPDATES,
    threads: int = THREADS,
    max_seconds: float = MAX_SECONDS,
) -> dict[str, Any]:
    """Run one create-once 128-update screen against the fixed dense arm."""

    if isinstance(updates, bool) or not isinstance(updates, int) or updates != MAX_UPDATES:
        raise ValueError(f"quaternion-cube decision requires exactly {MAX_UPDATES} updates")
    if (
        isinstance(max_seconds, bool)
        or not isinstance(max_seconds, (int, float))
        or float(max_seconds) != MAX_SECONDS
    ):
        raise ValueError(f"quaternion-cube decision requires a {MAX_SECONDS:g}-second wall")
    started = time.monotonic()
    resolved_root = root.expanduser().resolve()
    resolved_frames = frame_path.expanduser().resolve()
    output_directory = resolved_root / OUTPUT_DIRECTORY_RELATIVE_PATH
    staging_directory = output_directory.with_name(f".{output_directory.name}.tmp")
    output_artifact_path = output_directory / "model.safetensors"
    output_result_path = output_directory / "fit.json"
    if output_directory.exists() or staging_directory.exists():
        raise FileExistsError(
            "quaternion-cube canonical or staged fit output already exists: "
            f"{output_directory}"
        )

    train_path = resolved_root / TRAIN_RELATIVE_PATH
    tokenizer_path = resolved_root / TOKENIZER_RELATIVE_PATH
    geometry_path = resolved_root / GEOMETRY_RELATIVE_PATH
    artifact_path = resolved_root / ARTIFACT_RELATIVE_PATH
    arm_result_path = resolved_root / RESULT_RELATIVE_PATH
    inputs = {
        "train": _input_record(train_path, EXPECTED_TRAIN_SLICE_CID),
        "tokenizer": _input_record(tokenizer_path, EXPECTED_TOKENIZER_CID),
        "geometry": _input_record(geometry_path, EXPECTED_GEOMETRY_FILE_CID),
        "h4_frames": _input_record(resolved_frames, EXPECTED_H4_FRAME_FILE_CID),
        "initial_artifact": _input_record(artifact_path, EXPECTED_ARTIFACT_CID),
    }
    artifact_payload = artifact_path.read_bytes()
    arm_result_payload = arm_result_path.read_bytes()
    arm_result = json.loads(arm_result_payload)
    if not isinstance(arm_result, dict):
        raise ValueError("accepted ordinary arm result must be a JSON object")
    _verify_arm_result(arm_result, artifact_payload=artifact_payload)
    inputs["initial_artifact_result"] = {
        "path": str(arm_result_path),
        "bytes": len(arm_result_payload),
        "cid": cid_bytes(arm_result_payload),
    }

    execution = _configure_cpu(threads)
    store = LanguagePathWindowStore(train_path, window_count=TRAIN_WINDOWS)
    order = deterministic_window_order(TRAIN_WINDOWS)
    training_ordinals = order[: updates * BATCH_SIZE]
    monitor_ordinals = tuple(
        ordinal
        for batch_index in MONITOR_BATCH_INDICES
        for ordinal in order[
            batch_index * BATCH_SIZE : (batch_index + 1) * BATCH_SIZE
        ]
    )
    if len(training_ordinals) != updates * BATCH_SIZE or len(monitor_ordinals) != MONITOR_WINDOWS:
        raise RuntimeError("quaternion-cube fit population arithmetic changed")

    geometry_bundle = load_group_geometry_artifacts(geometry_path)
    frames = H4SpinFrameArtifactV1.from_bytes(resolved_frames.read_bytes())
    if (
        geometry_bundle.artifact_cid != EXPECTED_GEOMETRY_ARTIFACT_CID
        or frames.artifact_cid != EXPECTED_H4_FRAME_ARTIFACT_CID
        or frames.transport_control_source_cid != geometry_bundle.artifact_cid
    ):
        raise ValueError("quaternion-cube fit geometry/frame artifact identity differs")
    tokenizer = Tokenizer.from_file(str(tokenizer_path))
    if tokenizer.get_vocab_size(with_added_tokens=True) != VOCAB_SIZE:
        raise ValueError("quaternion-cube fit tokenizer vocabulary differs")
    raw_decoder = ByteLevelRawDecoder.from_tokenizer_json(tokenizer_path)

    candidate = R4H4FrameQuaternionCubeResidualV1.from_learned_artifact(
        artifact_payload,
        geometry=geometry_bundle.exact_h4,
        frames=frames,
    ).to(device=torch.device("cpu"), dtype=torch.float32)
    comparator = R4SparseGeometricCandidateSoftmaxKVBindingV1.from_learned_artifact(
        artifact_payload,
        geometry=geometry_bundle.exact_h4,
        frames=frames,
    ).to(device=torch.device("cpu"), dtype=torch.float32)
    if (
        candidate.parameter_count() != PARAMETER_COUNT
        or candidate.trainable_parameter_count() != ACTIVE_PARAMETER_VALUES
        or candidate.export_learned_artifact() != artifact_payload
        or comparator.export_learned_artifact() != artifact_payload
    ):
        raise RuntimeError("quaternion-cube fit changed the accepted initialization")
    initial_parameters = {
        name: parameter.detach().clone()
        for name, parameter in candidate.named_parameters()
    }
    frozen_names = tuple(
        name for name, parameter in candidate.named_parameters() if not parameter.requires_grad
    )
    if (
        len(frozen_names) != 6
        or sum(initial_parameters[name].numel() for name in frozen_names)
        != RETAINED_DENSE_MLP_PARAMETER_VALUES
        or any(".mlp." not in name for name in frozen_names)
    ):
        raise RuntimeError("quaternion-cube retained dense tensor contract changed")

    _require_resources(started, float(max_seconds))
    backward_gate = _full_context_backward_gate(candidate, store, training_ordinals[0])
    _require_resources(started, float(max_seconds))
    candidate_pre = _evaluate(candidate, store, monitor_ordinals)
    comparator_monitor = _evaluate(comparator, store, monitor_ordinals)
    candidate_anchors_pre = _score_anchors(candidate, store, tokenizer, order)
    comparator_anchors = _score_anchors(comparator, store, tokenizer, order)
    if candidate_pre["selector"] != comparator_monitor["selector"]:
        raise RuntimeError(
            "candidate and comparator sparse selection work ledgers differ"
        )

    active_parameters = candidate.trainable_parameters()
    optimizer = torch.optim.AdamW(
        active_parameters,
        lr=learning_rate(0),
        betas=(ADAM_BETA1, ADAM_BETA2),
        eps=ADAM_EPSILON,
        weight_decay=WEIGHT_DECAY,
    )
    optimizer_values = sum(
        parameter.numel()
        for group in optimizer.param_groups
        for parameter in group["params"]
    )
    if optimizer_values != ACTIVE_PARAMETER_VALUES:
        raise RuntimeError("quaternion-cube optimizer contains the wrong parameters")

    losses: list[float] = []
    gradient_norms: list[float] = []
    first_gradients: list[dict[str, Any]] | None = None
    last_gradients: list[dict[str, Any]] | None = None
    training_audit: Any | None = None
    training_nonlinear: RecurrentNonlinearAudit | None = None
    update_eight_projection: dict[str, Any] | None = None
    fit_started = time.monotonic()
    candidate.train()
    for update in range(1, updates + 1):
        _require_resources(started, float(max_seconds))
        offset = (update - 1) * BATCH_SIZE
        batch_ordinals = training_ordinals[offset : offset + BATCH_SIZE]
        batch_inputs, batch_targets = store.batch(batch_ordinals)
        optimizer.zero_grad(set_to_none=True)
        output = candidate(batch_inputs, batch_targets)
        if (
            output.loss is None
            or not bool(torch.isfinite(output.loss))
            or not bool(torch.isfinite(output.logits).all())
            or not _state_is_finite(output.final_state)
        ):
            raise RuntimeError("quaternion-cube fit produced a nonfinite forward")
        output.loss.backward()
        if update == 1:
            first_gradients = _gradient_census(candidate)
        if update == updates:
            last_gradients = _gradient_census(candidate)
        gradient_norm = torch.nn.utils.clip_grad_norm_(active_parameters, GRADIENT_CLIP)
        if not bool(torch.isfinite(gradient_norm)):
            raise RuntimeError("quaternion-cube fit produced a nonfinite gradient norm")
        rate = learning_rate(update)
        for group in optimizer.param_groups:
            group["lr"] = rate
        optimizer.step()
        losses.append(float(output.loss.detach()))
        gradient_norms.append(float(gradient_norm.detach()))
        training_audit = (
            output.audit
            if training_audit is None
            else training_audit.accumulated_with(output.audit)
        )
        training_nonlinear = (
            output.nonlinear_audit
            if training_nonlinear is None
            else training_nonlinear.accumulated_with(output.nonlinear_audit)
        )
        if update == 8:
            observed_fit_seconds = time.monotonic() - fit_started
            projected_remaining = (
                observed_fit_seconds / update * (updates - update) * 1.5
            )
            update_eight_projection = {
                "completed_updates": update,
                "observed_fit_seconds": observed_fit_seconds,
                "safety_factor": 1.5,
                "projected_remaining_fit_seconds": projected_remaining,
                "whole_process_elapsed_seconds": time.monotonic() - started,
                "reserved_post_fit_seconds": POST_FIT_SECONDS_RESERVE,
                "whole_process_hard_wall_seconds": float(max_seconds),
            }
            print(
                "quaternion_cube_fit projection="
                + json.dumps(update_eight_projection, sort_keys=True),
                flush=True,
            )
            if (
                time.monotonic() - started
                + projected_remaining
                + POST_FIT_SECONDS_RESERVE
                >= float(max_seconds)
            ):
                raise TimeoutError("quaternion-cube fit projection cannot meet its wall")
        if update == 1 or update % 16 == 0:
            _require_resources(started, float(max_seconds))
            print(
                "quaternion_cube_fit "
                f"update={update}/{updates} loss={losses[-1]:.6f} "
                f"gradient_norm={gradient_norms[-1]:.6f} "
                f"elapsed={time.monotonic() - started:.3f}s "
                f"peak_rss={_peak_rss_bytes()}",
                flush=True,
            )

    fit_elapsed_seconds = time.monotonic() - fit_started
    if (
        first_gradients is None
        or last_gradients is None
        or training_audit is None
        or training_nonlinear is None
    ):
        raise RuntimeError("quaternion-cube fit omitted required measurements")
    if not all(bool(torch.isfinite(parameter).all()) for parameter in active_parameters):
        raise RuntimeError("quaternion-cube fit produced a nonfinite parameter")
    frozen_unchanged = all(
        torch.equal(initial_parameters[name], dict(candidate.named_parameters())[name])
        for name in frozen_names
    )
    active_changed_names = [
        name
        for name, parameter in candidate.named_parameters()
        if parameter.requires_grad and not torch.equal(initial_parameters[name], parameter)
    ]
    if not frozen_unchanged or not active_changed_names:
        raise RuntimeError("quaternion-cube fit violated its active/frozen tensor contract")
    if training_nonlinear.dense_mlp_calls != 0 or not _mechanics_pass(
        _selector_record(training_audit)
    ):
        raise RuntimeError("quaternion-cube fit left the assembled architecture")

    _require_resources(started, float(max_seconds))
    candidate_post = _evaluate(candidate, store, monitor_ordinals)
    candidate_anchors_post = _score_anchors(candidate, store, tokenizer, order)
    if (
        candidate_post["selector"] != candidate_pre["selector"]
        or candidate_post["selector"] != comparator_monitor["selector"]
    ):
        raise RuntimeError("fitting changed the fixed sparse selection work ledger")

    artifact = candidate.export_learned_artifact()
    reloaded = R4H4FrameQuaternionCubeResidualV1.from_learned_artifact(
        artifact,
        geometry=geometry_bundle.exact_h4,
        frames=frames,
    ).to(device=torch.device("cpu"), dtype=torch.float32)
    reload_inputs, _ = store.batch(monitor_ordinals[:BATCH_SIZE])
    candidate.eval()
    reloaded.eval()
    with torch.inference_mode():
        trained_logits = candidate(reload_inputs).logits
        reloaded_logits = reloaded(reload_inputs).logits
    if (
        not torch.equal(trained_logits, reloaded_logits)
        or reloaded.export_learned_artifact() != artifact
    ):
        raise RuntimeError("quaternion-cube fitted artifact did not reload exactly")

    prompt_results: list[dict[str, Any]] = []
    for name, prompt in PROMPTS:
        fitted = _generate(reloaded, tokenizer, raw_decoder, prompt)
        dense = _generate(comparator, tokenizer, raw_decoder, prompt)
        prompt_results.append(
            {
                "name": name,
                "prompt": prompt,
                "fitted_candidate": fitted,
                "fixed_sparse_dense_comparator": dense,
                "common_generated_prefix_tokens": _common_prefix(
                    fitted["generated_token_ids"], dense["generated_token_ids"]
                ),
            }
        )
    if comparator.export_learned_artifact() != artifact_payload:
        raise RuntimeError("fixed sparse-dense comparator changed during evaluation")
    _require_resources(started, float(max_seconds))

    monitor_nll_improvement = candidate_pre["ce_nats"] - candidate_post["ce_nats"]
    monitor_nll_gap = candidate_post["ce_nats"] - comparator_monitor["ce_nats"]
    monitor_top1_gap = comparator_monitor["top1_rate"] - candidate_post["top1_rate"]
    anchor_top8 = all(
        anchor["score"]["gold_first_token_rank"] <= ANCHOR_TOP_K
        for anchor in candidate_anchors_post
    )
    anchor_argmax_prompt_dependent = (
        candidate_anchors_post[0]["score"]["argmax_token_id"]
        != candidate_anchors_post[1]["score"]["argmax_token_id"]
    )
    generated_prompt_dependent = (
        prompt_results[0]["fitted_candidate"]["generated_token_ids"]
        != prompt_results[1]["fitted_candidate"]["generated_token_ids"]
    )
    clean_generations = all(
        prompt["fitted_candidate"]["clean_text"] for prompt in prompt_results
    )
    dense_prefix_recovery = max(
        prompt["common_generated_prefix_tokens"] for prompt in prompt_results
    ) >= PROMPT_COMMON_PREFIX_REQUIRED
    decision_gates = {
        "completed_fixed_dose": True,
        "active_gradients_finite_nonzero": (
            len(first_gradients) == len(candidate.trainable_parameter_names())
        ),
        "retained_dense_tensors_byte_identical": frozen_unchanged,
        "monitor_nll_improved_at_least_0_10": (
            monitor_nll_improvement >= TRAIN_NLL_REQUIRED_IMPROVEMENT
        ),
        "monitor_nll_within_0_20_of_dense": (
            monitor_nll_gap <= COMPETITIVE_NLL_TOLERANCE
        ),
        "monitor_top1_within_0_02_of_dense": (
            monitor_top1_gap <= COMPETITIVE_TOP1_TOLERANCE
        ),
        "both_seen_anchor_first_tokens_top8": anchor_top8,
        "seen_anchor_argmaxes_prompt_dependent": anchor_argmax_prompt_dependent,
        "generated_outputs_clean": clean_generations,
        "generated_outputs_prompt_dependent": generated_prompt_dependent,
        "one_prompt_recovers_four_dense_prefix_tokens": dense_prefix_recovery,
        "sparse_and_causal_contract_preserved": (
            _mechanics_pass(candidate_post["selector"])
            and all(
                _mechanics_pass(prompt["fitted_candidate"]["selector"])
                for prompt in prompt_results
            )
        ),
    }
    passed = all(decision_gates.values())
    status = STATUS_PASS if passed else STATUS_NEGATIVE
    elapsed_seconds = time.monotonic() - started

    result: dict[str, Any] = {
        "schema": SCHEMA,
        "status": status,
        "issue": 973,
        "decision": {
            "passed": passed,
            "gates": decision_gates,
            "if_positive": (
                "advance the same assembled architecture to one larger bounded "
                "open-data dose"
            ),
            "if_negative": (
                "change the nonlinear parameterization or nonlinear capacity "
                "before another fit"
            ),
            "automatic_retry": False,
        },
        "model": {
            "candidate_policy": QUATERNION_CUBE_POLICY,
            "fixed_comparator_policy": SPARSE_DENSE_POLICY,
            "serialized_parameter_values": PARAMETER_COUNT,
            "candidate_active_parameter_values": ACTIVE_PARAMETER_VALUES,
            "candidate_active_parameter_tensors": len(candidate.trainable_parameter_names()),
            "retained_dense_parameter_values": RETAINED_DENSE_MLP_PARAMETER_VALUES,
            "retained_dense_parameter_tensors": len(frozen_names),
            "retained_dense_parameter_names": list(frozen_names),
            "retained_dense_tensors_byte_identical": frozen_unchanged,
            "active_changed_parameter_names": active_changed_names,
            "new_trainable_parameter_values": 0,
            "nonlinear_law_changed": False,
            "sparse_reader_changed": False,
            "recurrent_state_bytes_f32": RECURRENT_STATE_BYTES_F32,
        },
        "population": {
            "kind": "seen_open_training_development_screen",
            "training_windows": len(training_ordinals),
            "causal_targets": len(training_ordinals) * CONTEXT,
            "training_order_rank_start": 0,
            "training_order_rank_end_exclusive": len(training_ordinals),
            "training_ordinals_cid": cid_bytes(canonical_json_bytes(training_ordinals)),
            "monitor_windows": len(monitor_ordinals),
            "monitor_batch_indices": list(MONITOR_BATCH_INDICES),
            "monitor_ordinals_cid": cid_bytes(canonical_json_bytes(monitor_ordinals)),
            "monitor_is_subset_of_training": True,
            "generalization_claim": False,
        },
        "backward_gate": backward_gate,
        "fit": {
            "updates": updates,
            "batch_size": BATCH_SIZE,
            "optimizer": "AdamW",
            "optimizer_parameter_values": optimizer_values,
            "learning_rate_first": learning_rate(1),
            "learning_rate_last": learning_rate(updates),
            "adam_beta1": ADAM_BETA1,
            "adam_beta2": ADAM_BETA2,
            "adam_epsilon": ADAM_EPSILON,
            "weight_decay": WEIGHT_DECAY,
            "gradient_clip": GRADIENT_CLIP,
            "losses": losses,
            "gradient_norms": gradient_norms,
            "loss_mean_first_16": sum(losses[:16]) / 16,
            "loss_mean_last_16": sum(losses[-16:]) / 16,
            "first_gradients": first_gradients,
            "last_gradients": last_gradients,
            "fit_elapsed_seconds": fit_elapsed_seconds,
            "update_eight_projection": update_eight_projection,
            "selector": _selector_record(training_audit),
            "nonlinear": {
                "layer_calls": training_nonlinear.layer_calls,
                "dense_mlp_calls": training_nonlinear.dense_mlp_calls,
                "dense_mlp_weight_products": training_nonlinear.dense_mlp_weight_products,
                "r4_block_evaluations": training_nonlinear.r4_block_evaluations,
                "h4_frame_maps": training_nonlinear.h4_frame_maps,
                "h4_frame_coefficient_products": training_nonlinear.h4_frame_coefficient_products,
                "quaternion_cube_scalar_products": (
                    training_nonlinear.quaternion_cube_scalar_products
                ),
                "quaternion_cube_reciprocals": training_nonlinear.quaternion_cube_reciprocals,
                "residual_subtractions": training_nonlinear.residual_subtractions,
                "numerical": _numerical_record(training_nonlinear),
            },
        },
        "development_monitor": {
            "candidate_before_fit": candidate_pre,
            "candidate_after_fit": candidate_post,
            "fixed_sparse_dense_comparator": comparator_monitor,
            "candidate_nll_improvement": monitor_nll_improvement,
            "candidate_nll_gap_to_dense": monitor_nll_gap,
            "candidate_top1_gap_to_dense": monitor_top1_gap,
        },
        "seen_text_anchors": {
            "candidate_before_fit": candidate_anchors_pre,
            "candidate_after_fit": candidate_anchors_post,
            "fixed_sparse_dense_comparator": comparator_anchors,
        },
        "prompt_comparison": prompt_results,
        "execution": {
            **execution,
            "whole_process_elapsed_seconds": elapsed_seconds,
            "whole_process_hard_wall_seconds": float(max_seconds),
            "post_fit_reserve_seconds": POST_FIT_SECONDS_RESERVE,
            "peak_rss_bytes": _peak_rss_bytes(),
            "peak_rss_limit_bytes": PEAK_RSS_LIMIT_BYTES,
        },
        "inputs": {
            **inputs,
            "validation_files_read": 0,
            "heldout_files_read": 0,
            "teacher_files_read": 0,
            "source_model_files_read": 0,
            "provider_files_read": 0,
        },
        "artifact": {
            "path": str(output_artifact_path),
            "bytes": len(artifact),
            "cid": cid_bytes(artifact),
            "reload_logits_exact": True,
        },
        "limits": [
            (
                "The monitor and text anchors are sampled from the fitted open "
                "training slice; this is fit-capacity behavior, not held-out "
                "generalization."
            ),
            (
                "The dense comparator is the retained accepted artifact and "
                "receives no additional optimizer updates."
            ),
            (
                "A negative result applies to this warm start, fixed 128-update "
                "dose, sparse reader, cube residual, and f32 implementation; it "
                "does not prove from-scratch untrainability."
            ),
            (
                "This comparison does not isolate H4 advantage, because "
                "nonlinear law, cross-block mixing, and active parameter count "
                "differ."
            ),
            (
                "Reasoning, coding, instruction following, longer context, "
                "table-native lowering, and release behavior are not evaluated."
            ),
        ],
    }
    result_payload = canonical_json_bytes(result)
    if len(artifact) + len(result_payload) > OUTPUT_LIMIT_BYTES:
        raise RuntimeError("quaternion-cube fit output exceeds its storage ceiling")
    _require_resources(started, float(max_seconds))
    _write_output_bundle(output_directory, artifact, result)
    return result


__all__ = [
    "DEFAULT_UPDATES",
    "MAX_SECONDS",
    "MAX_UPDATES",
    "SCHEMA",
    "STATUS_NEGATIVE",
    "STATUS_PASS",
    "fit_quaternion_cube_r4_language_path",
]
