#!/usr/bin/env python3
"""Lightly induced split-boundary hybrid regime model on the stronger reduced-alignment setting."""

from __future__ import annotations

from dataclasses import dataclass
from time import perf_counter

import numpy as np
import torch

from geometry_native_sequence_model_v3 import (
    GeometryNativeSequenceModelV3,
    TransformerSequenceTrainerV3,
    extract_geometry_features_v3,
    generate_dataset_v3,
)
from geometry_native_sequence_model_v12 import (
    TaskConfigV12,
    evaluate_transformer_v12,
    generate_dataset_transfer_v12,
)
from geometry_native_sequence_model_v14 import _variant_features
from geometry_native_sequence_model_v21 import (
    _collect_raw_snapshots,
    _sequence_objective,
    _support_summary_features,
)
from geometry_native_sequence_model_v22 import REGION_VARIANTS_V22


torch.set_num_threads(1)


BOUNDARY_CANDIDATES_V23 = (14, 18, 21, 24, 28)


@dataclass
class SequenceMetricsV23:
    model: str
    test_loss: float
    test_accuracy: float
    transfer_query_accuracy: float
    param_count: int
    effective_state_size: int
    train_seconds: float
    eval_seconds: float


def _fit_region_proto_for_boundary(
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
        raw_snapshots = _collect_raw_snapshots(calibration_inputs[row])
        split = min(max(8, boundary), len(raw_snapshots) - 8)
        prefix_raw = raw_snapshots[:split]
        suffix_raw = raw_snapshots[split:]

        prefix_cache = {variant: _variant_features(prefix_raw, variant) for variant in REGION_VARIANTS_V22}
        suffix_cache = {variant: _variant_features(suffix_raw, variant) for variant in REGION_VARIANTS_V22}

        prefix_summaries.append(
            _support_summary_features(
                prefix_raw,
                prefix_cache,
                model,
                support_window=min(support_window, len(prefix_raw)),
            )
        )
        suffix_summaries.append(
            _support_summary_features(
                suffix_raw,
                suffix_cache,
                model,
                support_window=min(support_window, len(suffix_raw)),
            )
        )

        best_prefix = REGION_VARIANTS_V22[0]
        best_prefix_score = -1e9
        for variant in REGION_VARIANTS_V22:
            score = _sequence_objective(
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
            score = _sequence_objective(
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


def _fit_boundary_library(
    calibration_inputs: np.ndarray,
    calibration_targets: np.ndarray,
    calibration_query_mask: np.ndarray,
    model: GeometryNativeSequenceModelV3,
    *,
    support_window: int,
) -> dict[int, tuple[tuple[np.ndarray, np.ndarray, dict[str, np.ndarray]], tuple[np.ndarray, np.ndarray, dict[str, np.ndarray]]]]:
    library = {}
    for boundary in BOUNDARY_CANDIDATES_V23:
        library[boundary] = _fit_region_proto_for_boundary(
            calibration_inputs,
            calibration_targets,
            calibration_query_mask,
            model,
            boundary=boundary,
            support_window=support_window,
        )
    return library


def _boundary_distance_score(
    raw_snapshots: list[tuple[int, dict[str, int | list[int]]]],
    model: GeometryNativeSequenceModelV3,
    *,
    boundary: int,
    boundary_proto: tuple[tuple[np.ndarray, np.ndarray, dict[str, np.ndarray]], tuple[np.ndarray, np.ndarray, dict[str, np.ndarray]]],
    support_window: int,
) -> tuple[float, str, str, np.ndarray]:
    split = min(max(8, boundary), len(raw_snapshots) - 8)
    prefix_raw = raw_snapshots[:split]
    suffix_raw = raw_snapshots[split:]
    prefix_cache = {variant: _variant_features(prefix_raw, variant) for variant in REGION_VARIANTS_V22}
    suffix_cache = {variant: _variant_features(suffix_raw, variant) for variant in REGION_VARIANTS_V22}

    prefix_proto, suffix_proto = boundary_proto
    prefix_mean, prefix_std, prefix_centroids = prefix_proto
    suffix_mean, suffix_std, suffix_centroids = suffix_proto

    prefix_summary = _support_summary_features(
        prefix_raw,
        prefix_cache,
        model,
        support_window=min(support_window, len(prefix_raw)),
    )
    suffix_summary = _support_summary_features(
        suffix_raw,
        suffix_cache,
        model,
        support_window=min(support_window, len(suffix_raw)),
    )
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
    combined = np.concatenate([prefix_cache[prefix_variant][0], suffix_cache[suffix_variant][0]], axis=0)
    return prefix_distance + suffix_distance, prefix_variant, suffix_variant, combined


def extract_geometry_features_v23(
    inputs: np.ndarray,
    model: GeometryNativeSequenceModelV3,
    *,
    support_window: int = 10,
    calibration_size: int = 256,
    calibration_seed: int = 229,
) -> np.ndarray:
    calibration_inputs, calibration_targets, calibration_query_mask = generate_dataset_transfer_v12(
        calibration_size,
        seq_len=inputs.shape[1],
        seed=calibration_seed,
    )
    boundary_library = _fit_boundary_library(
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
        best_features = None
        best_score = float("inf")
        for boundary in BOUNDARY_CANDIDATES_V23:
            score, _prefix_variant, _suffix_variant, candidate = _boundary_distance_score(
                raw_snapshots,
                model,
                boundary=boundary,
                boundary_proto=boundary_library[boundary],
                support_window=support_window,
            )
            if score < best_score:
                best_score = score
                best_features = candidate

        assert best_features is not None
        features.extend(best_features)

    return np.stack(features, axis=0).reshape(size, seq_len, -1)


def evaluate_geometry_v23(
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


def run_bounded_sequence_comparison_v23(config: TaskConfigV12, *, seed: int = 0) -> list[SequenceMetricsV23]:
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

    test_features = extract_geometry_features_v23(
        test_inputs,
        geometry_model,
        calibration_seed=seed + 23,
    )
    geometry_test_loss, geometry_test_acc, geometry_transfer_query_acc, geometry_eval_seconds = evaluate_geometry_v23(
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
        SequenceMetricsV23(
            model="geometry_native_sequence_model_v23",
            test_loss=geometry_test_loss,
            test_accuracy=geometry_test_acc,
            transfer_query_accuracy=geometry_transfer_query_acc,
            param_count=geometry_model.param_count,
            effective_state_size=geometry_model.effective_state_size,
            train_seconds=geometry_train_seconds,
            eval_seconds=geometry_eval_seconds,
        ),
        SequenceMetricsV23(
            model="tiny_transformer_baseline_v23",
            test_loss=transformer_test_loss,
            test_accuracy=transformer_test_acc,
            transfer_query_accuracy=transformer_transfer_query_acc,
            param_count=transformer.param_count,
            effective_state_size=transformer.effective_state_size,
            train_seconds=transformer_train_seconds,
            eval_seconds=transformer_eval_seconds,
        ),
    ]
