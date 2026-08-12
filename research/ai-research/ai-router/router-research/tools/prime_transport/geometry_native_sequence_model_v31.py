#!/usr/bin/env python3
"""Small global learned chart field on the stronger shifted family."""

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
from geometry_native_sequence_model_v12 import evaluate_transformer_v12
from geometry_native_sequence_model_v14 import _variant_features
from geometry_native_sequence_model_v21 import _support_summary_features
from geometry_native_sequence_model_v22 import REGION_VARIANTS_V22
from geometry_native_sequence_model_v26 import _sequence_objective_full
from geometry_native_sequence_model_v27 import (
    TaskConfigV27,
    _collect_raw_snapshots_v27,
    generate_dataset_transfer_v27,
)
from geometry_native_sequence_model_v30 import _block_bounds


torch.set_num_threads(1)


BLOCK_COUNT_V31 = 8
MAX_REGIONS_V31 = 4
TRANSITION_PENALTY_V31 = 0.28


@dataclass
class SequenceMetricsV31:
    model: str
    test_loss: float
    test_accuracy: float
    transfer_query_accuracy: float
    param_count: int
    effective_state_size: int
    train_seconds: float
    eval_seconds: float


class GlobalChartField(torch.nn.Module):
    def __init__(self, feature_dim: int, label_count: int, block_count: int, *, seed: int) -> None:
        super().__init__()
        torch.manual_seed(seed)
        self.local = torch.nn.Linear(feature_dim, 24)
        self.global_pool = torch.nn.Linear(feature_dim, 12)
        self.global_basis = torch.nn.Linear(12, block_count * label_count)
        self.out = torch.nn.Linear(24, label_count)
        self.block_count = block_count
        self.label_count = label_count

    def forward(self, x: torch.Tensor) -> torch.Tensor:
        # x: [batch, blocks, feature_dim]
        local_hidden = torch.tanh(self.local(x))
        local_logits = self.out(local_hidden)
        pooled = x.mean(dim=1)
        global_latent = torch.tanh(self.global_pool(pooled))
        global_map = self.global_basis(global_latent).reshape(-1, self.block_count, self.label_count)
        return local_logits + global_map

    @property
    def param_count(self) -> int:
        return sum(parameter.numel() for parameter in self.parameters())


def _block_details_v31(
    raw_snapshots: list[tuple[int, dict[str, int | list[int]]]],
    model: GeometryNativeSequenceModelV3,
    *,
    support_window: int,
) -> tuple[np.ndarray, list[dict[str, tuple[np.ndarray, np.ndarray, dict[str, np.ndarray]]]], list[tuple[int, int]]]:
    bounds = _block_bounds(len(raw_snapshots), block_count=BLOCK_COUNT_V31)
    summaries: list[np.ndarray] = []
    caches: list[dict[str, tuple[np.ndarray, np.ndarray, dict[str, np.ndarray]]]] = []

    for start, end in bounds:
        block_raw = raw_snapshots[start:end]
        cache = {variant: _variant_features(block_raw, variant) for variant in REGION_VARIANTS_V22}
        summary = _support_summary_features(
            block_raw,
            cache,
            model,
            support_window=min(support_window, len(block_raw)),
        )
        summaries.append(summary.astype(np.float32))
        caches.append(cache)

    return np.stack(summaries, axis=0), caches, bounds


def _build_training_set_v31(
    calibration_inputs: np.ndarray,
    calibration_targets: np.ndarray,
    calibration_query_mask: np.ndarray,
    model: GeometryNativeSequenceModelV3,
    *,
    support_window: int,
) -> tuple[np.ndarray, np.ndarray]:
    feature_rows: list[np.ndarray] = []
    label_rows: list[np.ndarray] = []

    for row in range(calibration_inputs.shape[0]):
        raw_snapshots = _collect_raw_snapshots_v27(calibration_inputs[row])
        block_features, block_caches, bounds = _block_details_v31(
            raw_snapshots,
            model,
            support_window=support_window,
        )

        labels: list[int] = []
        for block_idx, (start, end) in enumerate(bounds):
            best_variant = 0
            best_score = -1e9
            for variant_idx, variant in enumerate(REGION_VARIANTS_V22):
                score = _sequence_objective_full(
                    model,
                    block_caches[block_idx][variant][0],
                    calibration_targets[row][start:end],
                    calibration_query_mask[row][start:end],
                )
                if score > best_score:
                    best_score = score
                    best_variant = variant_idx
            labels.append(best_variant)

        feature_rows.append(block_features)
        label_rows.append(np.array(labels, dtype=np.int64))

    return np.stack(feature_rows, axis=0), np.stack(label_rows, axis=0)


def _fit_global_chart_field(
    train_x: np.ndarray,
    train_y: np.ndarray,
    *,
    seed: int,
    epochs: int = 220,
    lr: float = 0.02,
) -> GlobalChartField:
    field = GlobalChartField(train_x.shape[-1], len(REGION_VARIANTS_V22), train_x.shape[1], seed=seed)
    optimizer = torch.optim.Adam(field.parameters(), lr=lr)
    loss_fn = torch.nn.CrossEntropyLoss()

    x = torch.tensor(train_x, dtype=torch.float32)
    y = torch.tensor(train_y, dtype=torch.long)
    for _ in range(epochs):
        optimizer.zero_grad()
        logits = field(x)
        loss = loss_fn(logits.reshape(-1, logits.shape[-1]), y.reshape(-1))
        loss.backward()
        optimizer.step()
    return field


def _decode_global_chart_path_v31(
    logits: np.ndarray,
    *,
    transition_penalty: float = TRANSITION_PENALTY_V31,
    max_regions: int = MAX_REGIONS_V31,
) -> list[int]:
    block_count, label_count = logits.shape
    max_changes = max_regions - 1
    dp = np.full((block_count, label_count, max_changes + 1), -1e9, dtype=np.float32)
    back = np.full((block_count, label_count, max_changes + 1, 2), -1, dtype=np.int64)

    for label in range(label_count):
        dp[0, label, 0] = float(logits[0, label])

    for block in range(1, block_count):
        for label in range(label_count):
            score_here = float(logits[block, label])
            for changes in range(max_changes + 1):
                best_score = dp[block - 1, label, changes] + score_here
                best_prev = (label, changes)
                if changes > 0:
                    for prev_label in range(label_count):
                        if prev_label == label:
                            continue
                        candidate = dp[block - 1, prev_label, changes - 1] + score_here - transition_penalty
                        if candidate > best_score:
                            best_score = candidate
                            best_prev = (prev_label, changes - 1)
                dp[block, label, changes] = best_score
                back[block, label, changes] = np.array(best_prev, dtype=np.int64)

    end_index = np.unravel_index(int(np.argmax(dp[-1])), dp[-1].shape)
    label = int(end_index[0])
    changes = int(end_index[1])
    path = [label]
    for block in range(block_count - 1, 0, -1):
        prev_label, prev_changes = back[block, label, changes]
        label = int(prev_label)
        changes = int(prev_changes)
        path.append(label)
    path.reverse()
    return path


def _collapse_regions_v31(
    labels: list[int],
    bounds: list[tuple[int, int]],
) -> list[tuple[int, int, str]]:
    regions: list[tuple[int, int, str]] = []
    current_label = labels[0]
    region_start = bounds[0][0]
    region_end = bounds[0][1]

    for block_idx in range(1, len(labels)):
        start, end = bounds[block_idx]
        if labels[block_idx] == current_label:
            region_end = end
            continue
        regions.append((region_start, region_end, REGION_VARIANTS_V22[current_label]))
        current_label = labels[block_idx]
        region_start = start
        region_end = end
    regions.append((region_start, region_end, REGION_VARIANTS_V22[current_label]))
    return regions


def extract_geometry_features_v31(
    inputs: np.ndarray,
    model: GeometryNativeSequenceModelV3,
    *,
    support_window: int = 10,
    calibration_size: int = 256,
    calibration_seed: int = 281,
    field_seed: int = 0,
) -> tuple[np.ndarray, int]:
    calibration_inputs, calibration_targets, calibration_query_mask = generate_dataset_transfer_v27(
        calibration_size,
        seq_len=inputs.shape[1],
        seed=calibration_seed,
    )
    train_x, train_y = _build_training_set_v31(
        calibration_inputs,
        calibration_targets,
        calibration_query_mask,
        model,
        support_window=support_window,
    )
    chart_field = _fit_global_chart_field(train_x, train_y, seed=field_seed)

    size, seq_len = inputs.shape
    features: list[np.ndarray] = []
    for row in range(size):
        raw_snapshots = _collect_raw_snapshots_v27(inputs[row])
        block_features, _block_caches, bounds = _block_details_v31(
            raw_snapshots,
            model,
            support_window=support_window,
        )
        x = torch.tensor(block_features[None, ...], dtype=torch.float32)
        with torch.no_grad():
            logits = chart_field(x).squeeze(0).cpu().numpy()
        labels = _decode_global_chart_path_v31(logits)
        regions = _collapse_regions_v31(labels, bounds)

        combined_regions: list[np.ndarray] = []
        for start, end, variant in regions:
            region_raw = raw_snapshots[start:end]
            combined_regions.append(_variant_features(region_raw, variant)[0])
        combined = np.concatenate(combined_regions, axis=0)
        features.extend(combined)

    return np.stack(features, axis=0).reshape(size, seq_len, -1), chart_field.param_count


def evaluate_geometry_v31(
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


def run_bounded_sequence_comparison_v31(config: TaskConfigV27, *, seed: int = 0) -> list[SequenceMetricsV31]:
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

    test_features, field_param_count = extract_geometry_features_v31(
        test_inputs,
        geometry_model,
        calibration_seed=seed + 83,
        field_seed=seed + 89,
    )
    geometry_test_loss, geometry_test_acc, geometry_transfer_query_acc, geometry_eval_seconds = evaluate_geometry_v31(
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
    total_state_size = geometry_model.effective_state_size + BLOCK_COUNT_V31 + MAX_REGIONS_V31

    return [
        SequenceMetricsV31(
            model="geometry_native_sequence_model_v31",
            test_loss=geometry_test_loss,
            test_accuracy=geometry_test_acc,
            transfer_query_accuracy=geometry_transfer_query_acc,
            param_count=total_param_count,
            effective_state_size=total_state_size,
            train_seconds=geometry_train_seconds,
            eval_seconds=geometry_eval_seconds,
        ),
        SequenceMetricsV31(
            model="tiny_transformer_baseline_v31",
            test_loss=transformer_test_loss,
            test_accuracy=transformer_test_acc,
            transfer_query_accuracy=transformer_transfer_query_acc,
            param_count=transformer.param_count,
            effective_state_size=transformer.effective_state_size,
            train_seconds=transformer_train_seconds,
            eval_seconds=transformer_eval_seconds,
        ),
    ]
