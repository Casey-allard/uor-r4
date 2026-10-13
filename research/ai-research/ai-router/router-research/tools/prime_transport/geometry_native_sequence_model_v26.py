#!/usr/bin/env python3
"""Small learned regional boundary/regime field on the stronger reduced-alignment setting."""

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
from geometry_native_sequence_model_v23 import (
    BOUNDARY_CANDIDATES_V23,
    _fit_boundary_library,
)
from geometry_native_sequence_model_v25 import _boundary_objectives
from geometry_native_sequence_model_v21 import _collect_raw_snapshots, _support_summary_features
from geometry_native_sequence_model_v22 import REGION_VARIANTS_V22
from geometry_native_sequence_model_v14 import _variant_features


torch.set_num_threads(1)


@dataclass
class SequenceMetricsV26:
    model: str
    test_loss: float
    test_accuracy: float
    transfer_query_accuracy: float
    param_count: int
    effective_state_size: int
    train_seconds: float
    eval_seconds: float


class RegionalBoundaryField(torch.nn.Module):
    def __init__(self, feature_dim: int, *, seed: int) -> None:
        super().__init__()
        torch.manual_seed(seed)
        self.net = torch.nn.Sequential(
            torch.nn.Linear(feature_dim, 16),
            torch.nn.Tanh(),
            torch.nn.Linear(16, 1),
        )

    def forward(self, x: torch.Tensor) -> torch.Tensor:
        return self.net(x).squeeze(-1)

    @property
    def param_count(self) -> int:
        return sum(parameter.numel() for parameter in self.parameters())


def _boundary_candidate_details(
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

    candidate_features = np.concatenate(
        [
            np.array(
                [
                    boundary / max(1, len(raw_snapshots)),
                    prefix_distance,
                    suffix_distance,
                    float(_boundary_objectives(raw_snapshots, boundary=boundary)[0]),
                    float(_boundary_objectives(raw_snapshots, boundary=boundary)[1]),
                ],
                dtype=np.float32,
            ),
            prefix_summary.astype(np.float32),
            suffix_summary.astype(np.float32),
        ],
        axis=0,
    )

    combined = np.concatenate(
        [
            prefix_cache[prefix_variant][0],
            suffix_cache[suffix_variant][0],
        ],
        axis=0,
    )
    return candidate_features, combined


def _sequence_objective_full(
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
        query_accuracy = float(((correct * q).sum() / q.sum()).item())
    return query_accuracy + 0.30 * accuracy


def _build_boundary_training_set(
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
        raw_snapshots = _collect_raw_snapshots(calibration_inputs[row])
        candidate_vectors: list[np.ndarray] = []
        candidate_scores: list[float] = []
        for idx, boundary in enumerate(BOUNDARY_CANDIDATES_V23):
            vector, combined = _boundary_candidate_details(
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


def _fit_boundary_field(
    train_x: np.ndarray,
    train_y: np.ndarray,
    *,
    seed: int,
    epochs: int = 120,
    lr: float = 0.03,
) -> RegionalBoundaryField:
    model = RegionalBoundaryField(train_x.shape[-1], seed=seed)
    optimizer = torch.optim.Adam(model.parameters(), lr=lr)
    loss_fn = torch.nn.CrossEntropyLoss()

    x = torch.tensor(train_x, dtype=torch.float32)
    y = torch.tensor(train_y, dtype=torch.long)
    for _ in range(epochs):
        optimizer.zero_grad()
        logits = model(x)
        loss = loss_fn(logits, y)
        loss.backward()
        optimizer.step()
    return model


def extract_geometry_features_v26(
    inputs: np.ndarray,
    model: GeometryNativeSequenceModelV3,
    *,
    support_window: int = 10,
    calibration_size: int = 256,
    calibration_seed: int = 241,
    field_seed: int = 0,
) -> tuple[np.ndarray, int]:
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
    train_x, train_y = _build_boundary_training_set(
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
        raw_snapshots = _collect_raw_snapshots(inputs[row])
        candidate_vectors: list[np.ndarray] = []
        candidate_outputs: list[np.ndarray] = []
        for boundary in BOUNDARY_CANDIDATES_V23:
            vector, combined = _boundary_candidate_details(
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


def evaluate_geometry_v26(
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


def run_bounded_sequence_comparison_v26(config: TaskConfigV12, *, seed: int = 0) -> list[SequenceMetricsV26]:
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

    test_features, field_param_count = extract_geometry_features_v26(
        test_inputs,
        geometry_model,
        calibration_seed=seed + 37,
        field_seed=seed + 41,
    )
    geometry_test_loss, geometry_test_acc, geometry_transfer_query_acc, geometry_eval_seconds = evaluate_geometry_v26(
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
    total_state_size = geometry_model.effective_state_size + len(BOUNDARY_CANDIDATES_V23)

    return [
        SequenceMetricsV26(
            model="geometry_native_sequence_model_v26",
            test_loss=geometry_test_loss,
            test_accuracy=geometry_test_acc,
            transfer_query_accuracy=geometry_transfer_query_acc,
            param_count=total_param_count,
            effective_state_size=total_state_size,
            train_seconds=geometry_train_seconds,
            eval_seconds=geometry_eval_seconds,
        ),
        SequenceMetricsV26(
            model="tiny_transformer_baseline_v26",
            test_loss=transformer_test_loss,
            test_accuracy=transformer_test_acc,
            transfer_query_accuracy=transformer_transfer_query_acc,
            param_count=transformer.param_count,
            effective_state_size=transformer.effective_state_size,
            train_seconds=transformer_train_seconds,
            eval_seconds=transformer_eval_seconds,
        ),
    ]
