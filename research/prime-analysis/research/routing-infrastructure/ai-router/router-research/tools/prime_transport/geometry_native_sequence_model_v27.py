#!/usr/bin/env python3
"""Transfer robustness of the learned regional field on a stronger shifted family."""

from __future__ import annotations

from dataclasses import dataclass
from time import perf_counter

import numpy as np
import torch

from geometry_native_sequence_model_v3 import (
    GeometryNativeSequenceModelV3,
    QUERY_IDS_V3,
    TOKENS_V3,
    TransformerSequenceTrainerV3,
    extract_geometry_features_v3,
    generate_dataset_v3,
)
from geometry_native_sequence_model_v4 import BASE_WEIGHTS_V4
from geometry_native_sequence_model_v7 import (
    MENTION_IDS,
    SET_SPEAKER_ID,
    SET_TOPIC_ID,
    SHIFT_STYLE_ID,
    TAG_IDS,
    TOKEN_DB_V7,
    TOKEN_DGAP_V7,
    TOKEN_DPHI_V7,
    TOKEN_DR_V7,
    _candidate_entities,
)
from geometry_native_sequence_model_v12 import (
    ASK_ID,
    ROLE_PRIMES,
    _initial_discourse_state,
    _initial_geometry_state,
    _prime,
    evaluate_transformer_v12,
)
from geometry_native_sequence_model_v14 import _variant_features
from geometry_native_sequence_model_v21 import _support_summary_features
from geometry_native_sequence_model_v22 import REGION_VARIANTS_V22
from geometry_native_sequence_model_v25 import _boundary_objectives
from geometry_native_sequence_model_v26 import (
    RegionalBoundaryField,
    _fit_boundary_field,
    _sequence_objective_full,
)


torch.set_num_threads(1)


BOUNDARY_CANDIDATES_V27 = (16, 20, 24, 28, 32)


@dataclass(frozen=True)
class TaskConfigV27:
    train_seq_len: int = 30
    test_seq_len: int = 48
    train_size: int = 1024
    test_size: int = 256


@dataclass
class SequenceMetricsV27:
    model: str
    test_loss: float
    test_accuracy: float
    transfer_query_accuracy: float
    param_count: int
    effective_state_size: int
    train_seconds: float
    eval_seconds: float


def _actual_binding_role_v27(
    style: int,
    b: int,
    phi: int,
    r: int,
    next_return_gap: int,
    speaker: int,
    topic: int,
    focus: int,
    tags: list[int],
) -> int:
    unequal = 1 if speaker != topic else 0
    gap_bias = 1 if next_return_gap in {2, 4} else 0
    tag_bias = tags[speaker] % 2
    focus_bias = 1 if focus == topic else 0
    return (style + 2 * b + phi + r + unequal + gap_bias + tag_bias + focus_bias) % 3


def _actual_query_role_v27(
    style: int,
    phi: int,
    r: int,
    next_return_gap: int,
    b: int,
    speaker: int,
    topic: int,
    focus: int,
    tags: list[int],
) -> int:
    parity = 1 if tags[speaker] == tags[topic] else 0
    drift = (b + speaker + topic + focus) % 2
    focus_tag = tags[focus] % 2
    focus_match = 1 if focus == speaker else 0
    return (2 * style + phi + r + next_return_gap + parity + drift + focus_tag + focus_match) % 3


def _proxy_role_v27(style: int, phi: int, next_return_gap: int, b: int, r: int) -> int:
    return (style + phi + (next_return_gap % 2) + ((b + r) % 2)) % 3


def _proxy_entity_v27(proxy_role: int, *, focus: int, speaker: int, topic: int, r: int, style: int) -> int:
    basis = (
        (focus + speaker + style) % 3,
        (speaker + topic + r) % 3,
        (focus + topic + style + r + 1) % 3,
    )
    return basis[proxy_role]


def step_entangled_transfer_world_v27(
    token_id: int,
    *,
    b: int,
    phi: int,
    r: int,
    focus: int,
    speaker: int,
    topic: int,
    style: int,
    next_return_gap: int,
    tags: list[int],
) -> tuple[int, dict[str, int | list[int]], bool]:
    next_focus = focus
    next_speaker = speaker
    next_topic = topic
    next_style = style
    next_tags = list(tags)
    is_query = token_id in QUERY_IDS_V3

    if token_id in MENTION_IDS:
        next_focus = (MENTION_IDS[token_id] + style + (b % 2)) % 3
    elif token_id == SET_SPEAKER_ID:
        next_speaker = (next_focus + next_topic + (phi % 2)) % 3
    elif token_id == SET_TOPIC_ID:
        next_topic = (next_focus + 2 * next_speaker + next_style + (b % 2) + (next_return_gap % 2)) % 3
    elif token_id == SHIFT_STYLE_ID:
        next_style = (next_style + 1 + (r % 2)) % 3
        if (b + phi + r) % 2 == 1:
            next_focus, next_topic = next_topic, next_focus
    elif token_id in TAG_IDS:
        candidates = _candidate_entities(next_focus, next_speaker, next_topic)
        binding_role = _actual_binding_role_v27(
            next_style,
            b,
            phi,
            r,
            next_return_gap,
            next_speaker,
            next_topic,
            next_focus,
            next_tags,
        )
        bound_entity = candidates[binding_role]
        next_tags[bound_entity] = TAG_IDS[token_id]
    elif token_id in QUERY_IDS_V3:
        pass
    else:
        raise ValueError(f"unexpected token id: {token_id}")

    candidates = _candidate_entities(next_focus, next_speaker, next_topic)
    actual_query_role = _actual_query_role_v27(
        next_style,
        phi,
        r,
        next_return_gap,
        b,
        next_speaker,
        next_topic,
        next_focus,
        next_tags,
    )
    actual_referent = candidates[actual_query_role]
    target = next_tags[actual_referent] if token_id == ASK_ID else actual_referent

    proxy_role = _proxy_role_v27(next_style, phi, next_return_gap, b, r)
    proxy_referent = _proxy_entity_v27(
        proxy_role,
        focus=next_focus,
        speaker=next_speaker,
        topic=next_topic,
        r=r,
        style=next_style,
    )

    snapshot = {
        "b": b,
        "phi": phi,
        "r": r,
        "focus": next_focus,
        "speaker": next_speaker,
        "topic": next_topic,
        "style": next_style,
        "next_return_gap": next_return_gap,
        "referent_role": proxy_role,
        "referent_entity": proxy_referent,
        "tags": next_tags,
    }

    next_b = (b + int(TOKEN_DB_V7[token_id]) + (next_style % 2)) % 5
    next_phi = (phi + int(TOKEN_DPHI_V7[token_id]) + (next_topic % 2)) % 3
    next_r = (r + int(TOKEN_DR_V7[token_id]) + (next_focus % 2)) % 4
    next_gap = 1 + ((next_return_gap - 1 + int(TOKEN_DGAP_V7[token_id]) + (next_speaker % 2)) % 4)
    snapshot["next_b"] = next_b
    snapshot["next_phi"] = next_phi
    snapshot["next_r"] = next_r
    snapshot["next_next_return_gap"] = next_gap
    return int(target), snapshot, is_query


def generate_dataset_transfer_v27(
    size: int,
    *,
    seq_len: int,
    seed: int,
) -> tuple[np.ndarray, np.ndarray, np.ndarray]:
    rng = np.random.default_rng(seed)
    weights = np.array(BASE_WEIGHTS_V4, dtype=np.float64)
    weights[ASK_ID] *= 1.25
    weights[SHIFT_STYLE_ID] *= 1.35
    for token_id in TAG_IDS:
        weights[token_id] *= 1.15
    weights = weights / weights.sum()

    inputs = np.zeros((size, seq_len), dtype=np.int64)
    targets = np.zeros((size, seq_len), dtype=np.int64)
    query_mask = np.zeros((size, seq_len), dtype=np.float32)

    for row in range(size):
        b, phi, r = _initial_geometry_state()
        focus, speaker, topic, style, gap, tags = _initial_discourse_state()
        for col in range(seq_len):
            token_id = int(rng.choice(len(TOKENS_V3), p=weights))
            target, snapshot, is_query = step_entangled_transfer_world_v27(
                token_id,
                b=b,
                phi=phi,
                r=r,
                focus=focus,
                speaker=speaker,
                topic=topic,
                style=style,
                next_return_gap=gap,
                tags=tags,
            )
            inputs[row, col] = token_id
            targets[row, col] = int(target)
            query_mask[row, col] = 1.0 if is_query else 0.0
            focus = int(snapshot["focus"])
            speaker = int(snapshot["speaker"])
            topic = int(snapshot["topic"])
            style = int(snapshot["style"])
            tags = list(snapshot["tags"])
            b = int(snapshot["next_b"])
            phi = int(snapshot["next_phi"])
            r = int(snapshot["next_r"])
            gap = int(snapshot["next_next_return_gap"])
    return inputs, targets, query_mask


def _collect_raw_snapshots_v27(tokens: np.ndarray) -> list[tuple[int, dict[str, int | list[int]]]]:
    b, phi, r = _initial_geometry_state()
    focus, speaker, topic, style, gap, tags = _initial_discourse_state()
    raw_snapshots: list[tuple[int, dict[str, int | list[int]]]] = []

    for token_id in tokens:
        _, snapshot, _ = step_entangled_transfer_world_v27(
            int(token_id),
            b=b,
            phi=phi,
            r=r,
            focus=focus,
            speaker=speaker,
            topic=topic,
            style=style,
            next_return_gap=gap,
            tags=tags,
        )
        raw_snapshots.append((int(token_id), snapshot))
        focus = int(snapshot["focus"])
        speaker = int(snapshot["speaker"])
        topic = int(snapshot["topic"])
        style = int(snapshot["style"])
        tags = list(snapshot["tags"])
        b = int(snapshot["next_b"])
        phi = int(snapshot["next_phi"])
        r = int(snapshot["next_r"])
        gap = int(snapshot["next_next_return_gap"])
    return raw_snapshots


def _fit_region_proto_for_boundary_v27(
    calibration_inputs: np.ndarray,
    calibration_targets: np.ndarray,
    calibration_query_mask: np.ndarray,
    model: GeometryNativeSequenceModelV3,
    *,
    boundary: int,
    support_window: int,
) -> tuple[tuple[np.ndarray, np.ndarray, dict[str, np.ndarray]], tuple[np.ndarray, np.ndarray, dict[str, np.ndarray]]]:
    prefix_summaries: list[np.ndarray] = []
    suffix_summaries: list[np.ndarray] = []
    prefix_labels: list[str] = []
    suffix_labels: list[str] = []

    for row in range(calibration_inputs.shape[0]):
        raw_snapshots = _collect_raw_snapshots_v27(calibration_inputs[row])
        split = min(max(8, boundary), len(raw_snapshots) - 8)
        prefix_raw = raw_snapshots[:split]
        suffix_raw = raw_snapshots[split:]
        prefix_cache = {variant: _variant_features(prefix_raw, variant) for variant in REGION_VARIANTS_V22}
        suffix_cache = {variant: _variant_features(suffix_raw, variant) for variant in REGION_VARIANTS_V22}

        prefix_summaries.append(
            _support_summary_features(prefix_raw, prefix_cache, model, support_window=min(support_window, len(prefix_raw)))
        )
        suffix_summaries.append(
            _support_summary_features(suffix_raw, suffix_cache, model, support_window=min(support_window, len(suffix_raw)))
        )

        best_prefix = REGION_VARIANTS_V22[0]
        best_prefix_score = -1e9
        for variant in REGION_VARIANTS_V22:
            score = _sequence_objective_full(
                model,
                prefix_cache[variant][0],
                calibration_targets[row][:split],
                calibration_query_mask[row][:split],
            )
            if score > best_prefix_score:
                best_prefix_score = score
                best_prefix = variant
        prefix_labels.append(best_prefix)

        best_suffix = REGION_VARIANTS_V22[0]
        best_suffix_score = -1e9
        for variant in REGION_VARIANTS_V22:
            score = _sequence_objective_full(
                model,
                suffix_cache[variant][0],
                calibration_targets[row][split:],
                calibration_query_mask[row][split:],
            )
            if score > best_suffix_score:
                best_suffix_score = score
                best_suffix = variant
        suffix_labels.append(best_suffix)

    def build_centroids(
        summaries: list[np.ndarray],
        labels: list[str],
    ) -> tuple[np.ndarray, np.ndarray, dict[str, np.ndarray]]:
        matrix = np.stack(summaries, axis=0)
        mean = matrix.mean(axis=0)
        std = matrix.std(axis=0) + 1e-6
        normalized = (matrix - mean) / std
        centroids: dict[str, np.ndarray] = {}
        for variant in REGION_VARIANTS_V22:
            mask = np.array([label == variant for label in labels], dtype=bool)
            if mask.any():
                centroids[variant] = normalized[mask].mean(axis=0)
            else:
                centroids[variant] = normalized.mean(axis=0)
        return mean, std, centroids

    return build_centroids(prefix_summaries, prefix_labels), build_centroids(suffix_summaries, suffix_labels)


def _fit_boundary_library_v27(
    calibration_inputs: np.ndarray,
    calibration_targets: np.ndarray,
    calibration_query_mask: np.ndarray,
    model: GeometryNativeSequenceModelV3,
    *,
    support_window: int,
) -> dict[int, tuple[tuple[np.ndarray, np.ndarray, dict[str, np.ndarray]], tuple[np.ndarray, np.ndarray, dict[str, np.ndarray]]]]:
    library = {}
    for boundary in BOUNDARY_CANDIDATES_V27:
        library[boundary] = _fit_region_proto_for_boundary_v27(
            calibration_inputs,
            calibration_targets,
            calibration_query_mask,
            model,
            boundary=boundary,
            support_window=support_window,
        )
    return library


def _boundary_candidate_details_v27(
    raw_snapshots: list[tuple[int, dict[str, int | list[int]]]],
    model: GeometryNativeSequenceModelV3,
    *,
    boundary: int,
    boundary_proto: tuple[tuple[np.ndarray, np.ndarray, dict[str, np.ndarray]], tuple[np.ndarray, np.ndarray, dict[str, np.ndarray]]],
    support_window: int,
) -> tuple[np.ndarray, np.ndarray]:
    split = min(max(8, boundary), len(raw_snapshots) - 8)
    prefix_raw = raw_snapshots[:split]
    suffix_raw = raw_snapshots[split:]
    prefix_cache = {variant: _variant_features(prefix_raw, variant) for variant in REGION_VARIANTS_V22}
    suffix_cache = {variant: _variant_features(suffix_raw, variant) for variant in REGION_VARIANTS_V22}

    prefix_proto, suffix_proto = boundary_proto
    prefix_mean, prefix_std, prefix_centroids = prefix_proto
    suffix_mean, suffix_std, suffix_centroids = suffix_proto

    prefix_summary = _support_summary_features(prefix_raw, prefix_cache, model, support_window=min(support_window, len(prefix_raw)))
    suffix_summary = _support_summary_features(suffix_raw, suffix_cache, model, support_window=min(support_window, len(suffix_raw)))
    prefix_norm = (prefix_summary - prefix_mean) / prefix_std
    suffix_norm = (suffix_summary - suffix_mean) / suffix_std

    prefix_variant = min(
        REGION_VARIANTS_V22,
        key=lambda variant: float(np.square(prefix_norm - prefix_centroids[variant]).sum()),
    )
    suffix_variant = min(
        REGION_VARIANTS_V22,
        key=lambda variant: float(np.square(suffix_norm - suffix_centroids[variant]).sum()),
    )
    prefix_distance = float(np.square(prefix_norm - prefix_centroids[prefix_variant]).sum())
    suffix_distance = float(np.square(suffix_norm - suffix_centroids[suffix_variant]).sum())
    overall_objective, query_objective = _boundary_objectives(raw_snapshots, boundary=boundary)

    candidate_features = np.concatenate(
        [
            np.array(
                [
                    boundary / max(1, len(raw_snapshots)),
                    prefix_distance,
                    suffix_distance,
                    float(overall_objective),
                    float(query_objective),
                ],
                dtype=np.float32,
            ),
            prefix_summary.astype(np.float32),
            suffix_summary.astype(np.float32),
        ],
        axis=0,
    )
    combined = np.concatenate([prefix_cache[prefix_variant][0], suffix_cache[suffix_variant][0]], axis=0)
    return candidate_features, combined


def _build_boundary_training_set_v27(
    calibration_inputs: np.ndarray,
    calibration_targets: np.ndarray,
    calibration_query_mask: np.ndarray,
    model: GeometryNativeSequenceModelV3,
    *,
    boundary_library: dict[int, tuple[tuple[np.ndarray, np.ndarray, dict[str, np.ndarray]], tuple[np.ndarray, np.ndarray, dict[str, np.ndarray]]]],
    support_window: int,
) -> tuple[np.ndarray, np.ndarray]:
    all_features: list[np.ndarray] = []
    labels: list[int] = []

    for row in range(calibration_inputs.shape[0]):
        raw_snapshots = _collect_raw_snapshots_v27(calibration_inputs[row])
        candidate_vectors: list[np.ndarray] = []
        candidate_scores: list[float] = []
        for idx, boundary in enumerate(BOUNDARY_CANDIDATES_V27):
            vector, combined = _boundary_candidate_details_v27(
                raw_snapshots,
                model,
                boundary=boundary,
                boundary_proto=boundary_library[boundary],
                support_window=support_window,
            )
            candidate_vectors.append(vector)
            candidate_scores.append(
                _sequence_objective_full(
                    model,
                    combined,
                    calibration_targets[row],
                    calibration_query_mask[row],
                )
            )
        all_features.append(np.stack(candidate_vectors, axis=0))
        labels.append(int(np.argmax(np.array(candidate_scores))))
    return np.stack(all_features, axis=0), np.array(labels, dtype=np.int64)


def extract_geometry_features_v27(
    inputs: np.ndarray,
    model: GeometryNativeSequenceModelV3,
    *,
    support_window: int = 10,
    calibration_size: int = 256,
    calibration_seed: int = 251,
    field_seed: int = 0,
) -> tuple[np.ndarray, int]:
    calibration_inputs, calibration_targets, calibration_query_mask = generate_dataset_transfer_v27(
        calibration_size,
        seq_len=inputs.shape[1],
        seed=calibration_seed,
    )
    boundary_library = _fit_boundary_library_v27(
        calibration_inputs,
        calibration_targets,
        calibration_query_mask,
        model,
        support_window=support_window,
    )
    train_x, train_y = _build_boundary_training_set_v27(
        calibration_inputs,
        calibration_targets,
        calibration_query_mask,
        model,
        boundary_library=boundary_library,
        support_window=support_window,
    )
    boundary_field = _fit_boundary_field(train_x, train_y, seed=field_seed)

    size, seq_len = inputs.shape
    features: list[np.ndarray] = []
    for row in range(size):
        raw_snapshots = _collect_raw_snapshots_v27(inputs[row])
        candidate_vectors: list[np.ndarray] = []
        candidate_outputs: list[np.ndarray] = []
        for boundary in BOUNDARY_CANDIDATES_V27:
            vector, combined = _boundary_candidate_details_v27(
                raw_snapshots,
                model,
                boundary=boundary,
                boundary_proto=boundary_library[boundary],
                support_window=support_window,
            )
            candidate_vectors.append(vector)
            candidate_outputs.append(combined)

        x = torch.tensor(np.stack(candidate_vectors, axis=0)[None, ...], dtype=torch.float32)
        with torch.no_grad():
            logits = boundary_field(x).squeeze(0)
            best_idx = int(torch.argmax(logits).item())
        features.extend(candidate_outputs[best_idx])

    return np.stack(features, axis=0).reshape(size, seq_len, -1), boundary_field.param_count


def evaluate_geometry_v27(
    model: GeometryNativeSequenceModelV3,
    features: np.ndarray,
    targets: np.ndarray,
    query_mask: np.ndarray,
) -> tuple[float, float, float, float]:
    x = torch.tensor(features.reshape(-1, features.shape[-1]), dtype=torch.float32)
    y = torch.tensor(targets.reshape(-1), dtype=torch.long)
    q = torch.tensor(query_mask.reshape(-1), dtype=torch.float32)

    start = perf_counter()
    with torch.no_grad():
        logits = model.readout(x)
        loss = float(model.loss_fn(logits, y).item())
        predictions = logits.argmax(dim=-1)
        accuracy = float((predictions == y).float().mean().item())
        transfer_query_accuracy = float((((predictions == y).float() * q).sum() / q.sum()).item())
    elapsed = perf_counter() - start
    return loss, accuracy, transfer_query_accuracy, elapsed


def run_bounded_sequence_comparison_v27(config: TaskConfigV27, *, seed: int = 0) -> list[SequenceMetricsV27]:
    train_inputs, train_targets, _ = generate_dataset_v3(
        config.train_size,
        seq_len=config.train_seq_len,
        seed=seed,
    )
    test_inputs, test_targets, test_query_mask = generate_dataset_transfer_v27(
        config.test_size,
        seq_len=config.test_seq_len,
        seed=seed + 1,
    )

    train_features, _ = extract_geometry_features_v3(train_inputs)
    geometry_model = GeometryNativeSequenceModelV3(train_features.shape[-1], seed=seed)
    _train_loss, geometry_train_seconds = geometry_model.fit(train_features, train_targets)

    test_features, field_param_count = extract_geometry_features_v27(
        test_inputs,
        geometry_model,
        calibration_seed=seed + 43,
        field_seed=seed + 47,
    )
    geometry_test_loss, geometry_test_acc, geometry_transfer_query_acc, geometry_eval_seconds = evaluate_geometry_v27(
        geometry_model,
        test_features,
        test_targets,
        test_query_mask,
    )

    transformer = TransformerSequenceTrainerV3(seq_len=config.test_seq_len, seed=seed)
    _transformer_train_loss, transformer_train_seconds = transformer.fit(train_inputs, train_targets)
    transformer_test_loss, transformer_test_acc, transformer_transfer_query_acc, transformer_eval_seconds = evaluate_transformer_v12(
        transformer,
        test_inputs,
        test_targets,
        test_query_mask,
    )

    total_param_count = geometry_model.param_count + field_param_count
    total_state_size = geometry_model.effective_state_size + len(BOUNDARY_CANDIDATES_V27)

    return [
        SequenceMetricsV27(
            model="geometry_native_sequence_model_v27",
            test_loss=geometry_test_loss,
            test_accuracy=geometry_test_acc,
            transfer_query_accuracy=geometry_transfer_query_acc,
            param_count=total_param_count,
            effective_state_size=total_state_size,
            train_seconds=geometry_train_seconds,
            eval_seconds=geometry_eval_seconds,
        ),
        SequenceMetricsV27(
            model="tiny_transformer_baseline_v27",
            test_loss=transformer_test_loss,
            test_accuracy=transformer_test_acc,
            transfer_query_accuracy=transformer_transfer_query_acc,
            param_count=transformer.param_count,
            effective_state_size=transformer.effective_state_size,
            train_seconds=transformer_train_seconds,
            eval_seconds=transformer_eval_seconds,
        ),
    ]
