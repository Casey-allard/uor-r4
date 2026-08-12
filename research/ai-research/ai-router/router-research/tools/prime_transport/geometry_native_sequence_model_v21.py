#!/usr/bin/env python3
"""Bounded regional schema induction on the stronger reduced-alignment setting."""

from __future__ import annotations

from dataclasses import dataclass
from time import perf_counter

import numpy as np
import torch

from geometry_native_sequence_model_v3 import (
    GeometryNativeSequenceModelV3,
    QUERY_IDS_V3,
    TransformerSequenceTrainerV3,
    extract_geometry_features_v3,
    generate_dataset_v3,
)
from geometry_native_sequence_model_v12 import (
    TAG_IDS,
    TaskConfigV12,
    _initial_discourse_state,
    _initial_geometry_state,
    _proxy_role_v12,
    evaluate_transformer_v12,
    generate_dataset_transfer_v12,
    step_entangled_transfer_world_v12,
)
from geometry_native_sequence_model_v13 import _selection_score
from geometry_native_sequence_model_v14 import _variant_features
from geometry_native_sequence_model_v16 import _variant_delta
from geometry_native_sequence_model_v20 import _grownrepair_features, _window_conflict


torch.set_num_threads(1)


@dataclass
class SequenceMetricsV21:
    model: str
    test_loss: float
    test_accuracy: float
    transfer_query_accuracy: float
    param_count: int
    effective_state_size: int
    train_seconds: float
    eval_seconds: float


STRATEGIES_V21 = ("v11_like", "v12_base", "hybrid", "midpoint_switch", "grownrepair")


def _collect_raw_snapshots(
    tokens: np.ndarray,
) -> list[tuple[int, dict[str, int | list[int]]]]:
    b, phi, r = _initial_geometry_state()
    focus, speaker, topic, style, gap, tags = _initial_discourse_state()
    raw_snapshots: list[tuple[int, dict[str, int | list[int]]]] = []

    for token_id in tokens:
        _, snapshot, _ = step_entangled_transfer_world_v12(
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


def _variant_cache(
    raw_snapshots: list[tuple[int, dict[str, int | list[int]]]],
) -> dict[str, tuple[np.ndarray, list[dict[str, int | list[int]]], list[int]]]:
    return {variant: _variant_features(raw_snapshots, variant) for variant in ("v11_like", "v12_base", "hybrid")}


def _support_summary_features(
    raw_snapshots: list[tuple[int, dict[str, int | list[int]]]],
    variant_cache: dict[str, tuple[np.ndarray, list[dict[str, int | list[int]]], list[int]]],
    model: GeometryNativeSequenceModelV3,
    *,
    support_window: int,
) -> np.ndarray:
    window = min(support_window, len(raw_snapshots))
    states = [snapshot for _, snapshot in raw_snapshots[:window]]
    tokens = [token_id for token_id, _ in raw_snapshots[:window]]

    query_rate = sum(1 for token_id in tokens if token_id in QUERY_IDS_V3) / max(1, window)
    bind_rate = sum(1 for token_id in tokens if token_id in TAG_IDS) / max(1, window)

    b_vals = np.array([int(snapshot["b"]) for snapshot in states], dtype=np.float32)
    phi_vals = np.array([int(snapshot["phi"]) for snapshot in states], dtype=np.float32)
    r_vals = np.array([int(snapshot["r"]) for snapshot in states], dtype=np.float32)
    style_vals = np.array([int(snapshot["style"]) for snapshot in states], dtype=np.float32)
    gap_vals = np.array([int(snapshot["next_return_gap"]) for snapshot in states], dtype=np.float32)
    speaker_topic_eq = np.array(
        [1.0 if int(snapshot["speaker"]) == int(snapshot["topic"]) else 0.0 for snapshot in states],
        dtype=np.float32,
    )

    proxy_roles = np.array(
        [
            _proxy_role_v12(
                int(snapshot["style"]),
                int(snapshot["phi"]),
                int(snapshot["next_return_gap"]),
                int(snapshot["b"]),
            )
            for snapshot in states
        ],
        dtype=np.int64,
    )
    proxy_hist = np.bincount(proxy_roles, minlength=3).astype(np.float32) / max(1, len(proxy_roles))

    selection_scores = []
    for variant in ("v11_like", "v12_base", "hybrid"):
        features, snapshots, variant_tokens = variant_cache[variant]
        selection_scores.append(
            _selection_score(
                model,
                features[:window],
                snapshots[:window],
                variant_tokens[:window],
                support_window=window,
            )
        )

    pair_conflicts = []
    pair_names = (("v11_like", "v12_base"), ("v11_like", "hybrid"), ("v12_base", "hybrid"))
    for left_variant, right_variant in pair_names:
        disagreements = 0
        total = 0
        for token_id, snapshot in raw_snapshots[:window]:
            total += 1
            if _variant_delta(token_id, snapshot, left_variant) != _variant_delta(token_id, snapshot, right_variant):
                disagreements += 1
        pair_conflicts.append(disagreements / max(1, total))

    summary = np.array(
        [
            float(query_rate),
            float(bind_rate),
            float(b_vals.mean()),
            float(b_vals.std()),
            float(phi_vals.mean()),
            float(phi_vals.std()),
            float(r_vals.mean()),
            float(r_vals.std()),
            float(style_vals.mean()),
            float(style_vals.std()),
            float(gap_vals.mean()),
            float(gap_vals.std()),
            float(speaker_topic_eq.mean()),
            float(proxy_hist[0]),
            float(proxy_hist[1]),
            float(proxy_hist[2]),
            float(selection_scores[0]),
            float(selection_scores[1]),
            float(selection_scores[2]),
            float(pair_conflicts[0]),
            float(pair_conflicts[1]),
            float(pair_conflicts[2]),
        ],
        dtype=np.float32,
    )
    return summary


def _midpoint_features(
    raw_snapshots: list[tuple[int, dict[str, int | list[int]]]],
    variant_cache: dict[str, tuple[np.ndarray, list[dict[str, int | list[int]]], list[int]]],
    model: GeometryNativeSequenceModelV3,
    *,
    support_window: int,
) -> np.ndarray:
    seq_len = len(raw_snapshots)
    mid = seq_len // 2
    prefix_raw = raw_snapshots[:mid]
    suffix_raw = raw_snapshots[mid:]

    prefix_scores: list[float] = []
    prefix_features: list[np.ndarray] = []
    for variant in ("v11_like", "v12_base", "hybrid"):
        features, snapshots, tokens = _variant_features(prefix_raw, variant)
        prefix_scores.append(
            _selection_score(
                model,
                features,
                snapshots,
                tokens,
                support_window=min(support_window, len(features)),
            )
        )
        prefix_features.append(features)
    prefix_idx = int(np.argmax(np.array(prefix_scores)))

    suffix_scores: list[float] = []
    suffix_features: list[np.ndarray] = []
    for variant in ("v11_like", "v12_base", "hybrid"):
        features, snapshots, tokens = _variant_features(suffix_raw, variant)
        suffix_scores.append(
            _selection_score(
                model,
                features,
                snapshots,
                tokens,
                support_window=min(support_window, max(4, len(features))),
            )
        )
        suffix_features.append(features)
    suffix_idx = int(np.argmax(np.array(suffix_scores)))

    return np.concatenate([prefix_features[prefix_idx], suffix_features[suffix_idx]], axis=0)


def _grownrepair_strategy_features(
    raw_snapshots: list[tuple[int, dict[str, int | list[int]]]],
    variant_cache: dict[str, tuple[np.ndarray, list[dict[str, int | list[int]]], list[int]]],
    model: GeometryNativeSequenceModelV3,
    *,
    support_window: int,
    trigger_window: int = 8,
    seed_window: int = 6,
    max_window: int = 20,
    score_margin: float = 0.02,
    conflict_threshold: float = 0.30,
    stabilize_threshold: float = 0.18,
) -> np.ndarray:
    seq_len = len(raw_snapshots)
    initial_scores = []
    for variant in ("v11_like", "v12_base", "hybrid"):
        features, snapshots, tokens = variant_cache[variant]
        initial_scores.append(
            _selection_score(
                model,
                features,
                snapshots,
                tokens,
                support_window=support_window,
            )
        )

    current_variant = ("v11_like", "v12_base", "hybrid")[int(np.argmax(np.array(initial_scores)))]
    repair_center: int | None = None
    repair_alt: str | None = None

    min_start = max(support_window, seq_len // 3)
    max_start = max(min_start, seq_len - trigger_window)
    for start in range(min_start, max_start + 1):
        end = min(start + trigger_window, seq_len)
        current_features, current_snapshots, current_tokens = variant_cache[current_variant]
        current_score = _selection_score(
            model,
            current_features[start:end],
            current_snapshots[start:end],
            current_tokens[start:end],
            support_window=min(trigger_window, end - start),
        )

        best_alt_variant = None
        best_alt_score = -1e9
        best_conflict = 0.0
        for variant in ("v11_like", "v12_base", "hybrid"):
            if variant == current_variant:
                continue
            features, snapshots, tokens = variant_cache[variant]
            alt_score = _selection_score(
                model,
                features[start:end],
                snapshots[start:end],
                tokens[start:end],
                support_window=min(trigger_window, end - start),
            )
            joint_conflict = _window_conflict(
                raw_snapshots,
                current_variant=current_variant,
                alt_variant=variant,
                start=start,
                end=end,
            )
            if alt_score > best_alt_score:
                best_alt_score = alt_score
                best_alt_variant = variant
                best_conflict = joint_conflict

        if (
            best_alt_variant is not None
            and best_conflict >= conflict_threshold
            and best_alt_score >= current_score + score_margin
        ):
            repair_center = start + (end - start) // 2
            repair_alt = best_alt_variant
            break

    if repair_center is None or repair_alt is None:
        return variant_cache[current_variant][0]

    start = max(0, repair_center - seed_window // 2)
    end = min(seq_len, start + seed_window)
    start = max(0, end - seed_window)
    prev_conflict = _window_conflict(
        raw_snapshots,
        current_variant=current_variant,
        alt_variant=repair_alt,
        start=start,
        end=end,
    )

    while end - start < max_window:
        expanded = False
        if start > 0:
            left_conflict = _window_conflict(
                raw_snapshots,
                current_variant=current_variant,
                alt_variant=repair_alt,
                start=start - 1,
                end=end,
            )
            if left_conflict >= stabilize_threshold and left_conflict >= prev_conflict - 0.05:
                start -= 1
                prev_conflict = left_conflict
                expanded = True
        if end < seq_len and end - start < max_window:
            right_conflict = _window_conflict(
                raw_snapshots,
                current_variant=current_variant,
                alt_variant=repair_alt,
                start=start,
                end=end + 1,
            )
            if right_conflict >= stabilize_threshold and right_conflict >= prev_conflict - 0.05:
                end += 1
                prev_conflict = max(prev_conflict, right_conflict)
                expanded = True
        if not expanded:
            break

    return _grownrepair_features(
        raw_snapshots,
        current_variant=current_variant,
        alt_variant=repair_alt,
        start=start,
        end=end,
    )


def _strategy_features(
    raw_snapshots: list[tuple[int, dict[str, int | list[int]]]],
    variant_cache: dict[str, tuple[np.ndarray, list[dict[str, int | list[int]]], list[int]]],
    model: GeometryNativeSequenceModelV3,
    *,
    support_window: int,
    strategy: str,
) -> np.ndarray:
    if strategy in {"v11_like", "v12_base", "hybrid"}:
        return variant_cache[strategy][0]
    if strategy == "midpoint_switch":
        return _midpoint_features(raw_snapshots, variant_cache, model, support_window=support_window)
    if strategy == "grownrepair":
        return _grownrepair_strategy_features(raw_snapshots, variant_cache, model, support_window=support_window)
    raise ValueError(f"unknown strategy: {strategy}")


def _sequence_objective(
    model: GeometryNativeSequenceModelV3,
    features: np.ndarray,
    targets: np.ndarray,
    query_mask: np.ndarray,
) -> float:
    x = torch.tensor(features, dtype=torch.float32)
    y = torch.tensor(targets, dtype=torch.long)
    q = torch.tensor(query_mask, dtype=torch.float32)
    with torch.no_grad():
        logits = model.readout(x)
        predictions = logits.argmax(dim=-1)
        correct = (predictions == y).float()
        accuracy = float(correct.mean().item())
        query_acc = float(((correct * q).sum() / q.sum()).item())
    return query_acc + 0.30 * accuracy


def _fit_schema_centroids(
    calibration_inputs: np.ndarray,
    calibration_targets: np.ndarray,
    calibration_query_mask: np.ndarray,
    model: GeometryNativeSequenceModelV3,
    *,
    support_window: int,
) -> tuple[np.ndarray, np.ndarray, dict[str, np.ndarray]]:
    summaries: list[np.ndarray] = []
    labels: list[str] = []

    for row in range(calibration_inputs.shape[0]):
        raw_snapshots = _collect_raw_snapshots(calibration_inputs[row])
        variant_cache = _variant_cache(raw_snapshots)
        summary = _support_summary_features(raw_snapshots, variant_cache, model, support_window=support_window)
        summaries.append(summary)

        best_strategy = STRATEGIES_V21[0]
        best_score = -1e9
        for strategy in STRATEGIES_V21:
            features = _strategy_features(raw_snapshots, variant_cache, model, support_window=support_window, strategy=strategy)
            score = _sequence_objective(model, features, calibration_targets[row], calibration_query_mask[row])
            if score > best_score:
                best_score = score
                best_strategy = strategy
        labels.append(best_strategy)

    summary_matrix = np.stack(summaries, axis=0)
    mean = summary_matrix.mean(axis=0)
    std = summary_matrix.std(axis=0) + 1e-6
    normalized = (summary_matrix - mean) / std

    centroids: dict[str, np.ndarray] = {}
    for strategy in STRATEGIES_V21:
        mask = np.array([label == strategy for label in labels], dtype=bool)
        if mask.any():
            centroids[strategy] = normalized[mask].mean(axis=0)
        else:
            centroids[strategy] = normalized.mean(axis=0)
    return mean, std, centroids


def extract_geometry_features_v21(
    inputs: np.ndarray,
    model: GeometryNativeSequenceModelV3,
    *,
    support_window: int = 10,
    calibration_size: int = 256,
    calibration_seed: int = 211,
) -> np.ndarray:
    calibration_inputs, calibration_targets, calibration_query_mask = generate_dataset_transfer_v12(
        calibration_size,
        seq_len=inputs.shape[1],
        seed=calibration_seed,
    )
    mean, std, centroids = _fit_schema_centroids(
        calibration_inputs,
        calibration_targets,
        calibration_query_mask,
        model,
        support_window=support_window,
    )

    size, seq_len = inputs.shape
    features: list[np.ndarray] = []
    for row in range(size):
        raw_snapshots = _collect_raw_snapshots(inputs[row])
        variant_cache = _variant_cache(raw_snapshots)
        summary = _support_summary_features(raw_snapshots, variant_cache, model, support_window=support_window)
        normalized = (summary - mean) / std

        best_strategy = None
        best_distance = float("inf")
        for strategy in STRATEGIES_V21:
            distance = float(np.square(normalized - centroids[strategy]).sum())
            if distance < best_distance:
                best_distance = distance
                best_strategy = strategy

        assert best_strategy is not None
        chosen = _strategy_features(raw_snapshots, variant_cache, model, support_window=support_window, strategy=best_strategy)
        features.extend(chosen)

    return np.stack(features, axis=0).reshape(size, seq_len, -1)


def evaluate_geometry_v21(
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


def run_bounded_sequence_comparison_v21(config: TaskConfigV12, *, seed: int = 0) -> list[SequenceMetricsV21]:
    train_inputs, train_targets, _ = generate_dataset_v3(
        config.train_size,
        seq_len=config.train_seq_len,
        seed=seed,
    )
    test_inputs, test_targets, test_query_mask = generate_dataset_transfer_v12(
        config.test_size,
        seq_len=config.test_seq_len,
        seed=seed + 1,
    )

    train_features, _ = extract_geometry_features_v3(train_inputs)
    geometry_model = GeometryNativeSequenceModelV3(train_features.shape[-1], seed=seed)
    _train_loss, geometry_train_seconds = geometry_model.fit(train_features, train_targets)

    test_features = extract_geometry_features_v21(
        test_inputs,
        geometry_model,
        calibration_seed=seed + 17,
    )
    geometry_test_loss, geometry_test_acc, geometry_transfer_query_acc, geometry_eval_seconds = evaluate_geometry_v21(
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

    return [
        SequenceMetricsV21(
            model="geometry_native_sequence_model_v21",
            test_loss=geometry_test_loss,
            test_accuracy=geometry_test_acc,
            transfer_query_accuracy=geometry_transfer_query_acc,
            param_count=geometry_model.param_count,
            effective_state_size=geometry_model.effective_state_size,
            train_seconds=geometry_train_seconds,
            eval_seconds=geometry_eval_seconds,
        ),
        SequenceMetricsV21(
            model="tiny_transformer_baseline_v21",
            test_loss=transformer_test_loss,
            test_accuracy=transformer_test_acc,
            transfer_query_accuracy=transformer_transfer_query_acc,
            param_count=transformer.param_count,
            effective_state_size=transformer.effective_state_size,
            train_seconds=transformer_train_seconds,
            eval_seconds=transformer_eval_seconds,
        ),
    ]
