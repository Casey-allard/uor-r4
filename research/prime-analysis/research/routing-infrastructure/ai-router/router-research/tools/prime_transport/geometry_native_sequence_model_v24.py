#!/usr/bin/env python3
"""Contradiction-aware boundary induction on the stronger reduced-alignment setting."""

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
    QUERY_IDS_V3,
    TAG_IDS,
    TaskConfigV12,
    evaluate_transformer_v12,
    generate_dataset_transfer_v12,
)
from geometry_native_sequence_model_v23 import (
    BOUNDARY_CANDIDATES_V23,
    _boundary_distance_score,
    _fit_boundary_library,
)
from geometry_native_sequence_model_v16 import _variant_delta
from geometry_native_sequence_model_v21 import _collect_raw_snapshots


torch.set_num_threads(1)


@dataclass
class SequenceMetricsV24:
    model: str
    test_loss: float
    test_accuracy: float
    transfer_query_accuracy: float
    param_count: int
    effective_state_size: int
    train_seconds: float
    eval_seconds: float


def _boundary_conflict_score(
    raw_snapshots: list[tuple[int, dict[str, int | list[int]]]],
    *,
    boundary: int,
    half_window: int = 4,
) -> float:
    split = min(max(8, boundary), len(raw_snapshots) - 8)
    start = max(0, split - half_window)
    end = min(len(raw_snapshots), split + half_window)
    window = raw_snapshots[start:end]

    query_disagreements = 0
    query_total = 0
    bind_disagreements = 0
    bind_total = 0
    for token_id, snapshot in window:
        if token_id in QUERY_IDS_V3:
            query_total += 1
            deltas = {
                _variant_delta(token_id, snapshot, "v11_like"),
                _variant_delta(token_id, snapshot, "v12_base"),
                _variant_delta(token_id, snapshot, "hybrid"),
            }
            if len(deltas) > 1:
                query_disagreements += 1
        elif token_id in TAG_IDS:
            bind_total += 1
            deltas = {
                _variant_delta(token_id, snapshot, "v11_like"),
                _variant_delta(token_id, snapshot, "v12_base"),
                _variant_delta(token_id, snapshot, "hybrid"),
            }
            if len(deltas) > 1:
                bind_disagreements += 1

    query_rate = query_disagreements / max(1, query_total)
    bind_rate = bind_disagreements / max(1, bind_total)
    density = (query_total + bind_total) / max(1, len(window))
    return 0.5 * (query_rate + bind_rate) * density


def extract_geometry_features_v24(
    inputs: np.ndarray,
    model: GeometryNativeSequenceModelV3,
    *,
    support_window: int = 10,
    calibration_size: int = 256,
    calibration_seed: int = 233,
    contradiction_weight: float = 1.1,
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
        best_boundary = None
        best_features = None
        best_score = float("inf")

        for boundary in BOUNDARY_CANDIDATES_V23:
            proto_score, _prefix_variant, _suffix_variant, candidate = _boundary_distance_score(
                raw_snapshots,
                model,
                boundary=boundary,
                boundary_proto=boundary_library[boundary],
                support_window=support_window,
            )
            contradiction_bonus = _boundary_conflict_score(raw_snapshots, boundary=boundary)
            combined_score = proto_score - contradiction_weight * contradiction_bonus
            if combined_score < best_score:
                best_score = combined_score
                best_boundary = boundary
                best_features = candidate

        assert best_boundary is not None
        refinement_candidates = sorted(
            {
                best_boundary,
                max(8, best_boundary - 1),
                min(seq_len - 8, best_boundary + 1),
            }
        )
        for boundary in refinement_candidates:
            proto_score, _prefix_variant, _suffix_variant, candidate = _boundary_distance_score(
                raw_snapshots,
                model,
                boundary=boundary,
                boundary_proto=boundary_library[min(BOUNDARY_CANDIDATES_V23, key=lambda b: abs(b - boundary))],
                support_window=support_window,
            )
            contradiction_bonus = _boundary_conflict_score(raw_snapshots, boundary=boundary)
            combined_score = proto_score - contradiction_weight * contradiction_bonus
            if combined_score < best_score:
                best_score = combined_score
                best_features = candidate

        assert best_features is not None
        features.extend(best_features)

    return np.stack(features, axis=0).reshape(size, seq_len, -1)


def evaluate_geometry_v24(
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


def run_bounded_sequence_comparison_v24(config: TaskConfigV12, *, seed: int = 0) -> list[SequenceMetricsV24]:
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

    test_features = extract_geometry_features_v24(
        test_inputs,
        geometry_model,
        calibration_seed=seed + 29,
    )
    geometry_test_loss, geometry_test_acc, geometry_transfer_query_acc, geometry_eval_seconds = evaluate_geometry_v24(
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
        SequenceMetricsV24(
            model="geometry_native_sequence_model_v24",
            test_loss=geometry_test_loss,
            test_accuracy=geometry_test_acc,
            transfer_query_accuracy=geometry_transfer_query_acc,
            param_count=geometry_model.param_count,
            effective_state_size=geometry_model.effective_state_size,
            train_seconds=geometry_train_seconds,
            eval_seconds=geometry_eval_seconds,
        ),
        SequenceMetricsV24(
            model="tiny_transformer_baseline_v24",
            test_loss=transformer_test_loss,
            test_accuracy=transformer_test_acc,
            transfer_query_accuracy=transformer_transfer_query_acc,
            param_count=transformer.param_count,
            effective_state_size=transformer.effective_state_size,
            train_seconds=transformer_train_seconds,
            eval_seconds=transformer_eval_seconds,
        ),
    ]
