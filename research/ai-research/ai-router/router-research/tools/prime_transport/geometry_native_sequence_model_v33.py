#!/usr/bin/env python3
"""Sparse region-interaction field on the stronger shifted family."""

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
from geometry_native_sequence_model_v30 import (
    _block_details_v30,
    _build_training_set_v30,
    _collapse_regions_v30,
    _decode_contiguous_chart_path_v30,
    _fit_structured_chart_field,
)


torch.set_num_threads(1)


@dataclass
class SequenceMetricsV33:
    model: str
    test_loss: float
    test_accuracy: float
    transfer_query_accuracy: float
    param_count: int
    effective_state_size: int
    train_seconds: float
    eval_seconds: float


class RegionInteractionSelector(torch.nn.Module):
    def __init__(self, feature_dim: int, label_count: int, *, seed: int) -> None:
        super().__init__()
        torch.manual_seed(seed)
        self.input_proj = torch.nn.Linear(feature_dim, 24)
        self.out = torch.nn.Linear(24, label_count)

    def forward(self, x: torch.Tensor) -> torch.Tensor:
        hidden = torch.tanh(self.input_proj(x))
        return self.out(hidden)

    @property
    def param_count(self) -> int:
        return sum(parameter.numel() for parameter in self.parameters())


def _region_cache_v33(
    raw_snapshots: list[tuple[int, dict[str, int | list[int]]]],
    model: GeometryNativeSequenceModelV3,
    *,
    support_window: int,
) -> tuple[np.ndarray, dict[str, tuple[np.ndarray, np.ndarray, dict[str, np.ndarray]]]]:
    cache = {variant: _variant_features(raw_snapshots, variant) for variant in REGION_VARIANTS_V22}
    summary = _support_summary_features(
        raw_snapshots,
        cache,
        model,
        support_window=min(support_window, len(raw_snapshots)),
    ).astype(np.float32)
    return summary, cache


def _interaction_features_v33(
    summaries: list[np.ndarray],
    region_idx: int,
) -> np.ndarray:
    local = summaries[region_idx]
    if len(summaries) == 1:
        neighbor = np.zeros_like(local)
    else:
        neighbor_parts: list[np.ndarray] = []
        if region_idx > 0:
            neighbor_parts.append(summaries[region_idx - 1])
        if region_idx + 1 < len(summaries):
            neighbor_parts.append(summaries[region_idx + 1])
        neighbor = np.mean(np.stack(neighbor_parts, axis=0), axis=0)
    return np.concatenate(
        [
            local,
            neighbor,
            local - neighbor,
            local * neighbor,
        ],
        axis=0,
    ).astype(np.float32)


def _build_region_training_set_v33(
    calibration_inputs: np.ndarray,
    calibration_targets: np.ndarray,
    calibration_query_mask: np.ndarray,
    model: GeometryNativeSequenceModelV3,
    *,
    chart_field: torch.nn.Module,
    support_window: int,
) -> tuple[np.ndarray, np.ndarray]:
    feature_rows: list[np.ndarray] = []
    labels: list[int] = []

    for row in range(calibration_inputs.shape[0]):
        raw_snapshots = _collect_raw_snapshots_v27(calibration_inputs[row])
        block_features, _block_caches, bounds = _block_details_v30(
            raw_snapshots,
            model,
            support_window=support_window,
        )
        x = torch.tensor(block_features[None, ...], dtype=torch.float32)
        with torch.no_grad():
            logits = chart_field(x).squeeze(0).cpu().numpy()
        block_labels = _decode_contiguous_chart_path_v30(logits)
        regions = _collapse_regions_v30(block_labels, bounds)

        region_summaries: list[np.ndarray] = []
        region_variant_caches: list[dict[str, tuple[np.ndarray, np.ndarray, dict[str, np.ndarray]]]] = []
        for start, end, _variant in regions:
            summary, cache = _region_cache_v33(
                raw_snapshots[start:end],
                model,
                support_window=support_window,
            )
            region_summaries.append(summary)
            region_variant_caches.append(cache)

        for region_idx, (start, end, _variant) in enumerate(regions):
            best_variant = 0
            best_score = -1e9
            for variant_idx, variant in enumerate(REGION_VARIANTS_V22):
                score = _sequence_objective_full(
                    model,
                    region_variant_caches[region_idx][variant][0],
                    calibration_targets[row][start:end],
                    calibration_query_mask[row][start:end],
                )
                if score > best_score:
                    best_score = score
                    best_variant = variant_idx
            feature_rows.append(_interaction_features_v33(region_summaries, region_idx))
            labels.append(best_variant)

    return np.stack(feature_rows, axis=0), np.array(labels, dtype=np.int64)


def _fit_region_interaction_selector(
    train_x: np.ndarray,
    train_y: np.ndarray,
    *,
    seed: int,
    epochs: int = 180,
    lr: float = 0.02,
) -> RegionInteractionSelector:
    selector = RegionInteractionSelector(train_x.shape[-1], len(REGION_VARIANTS_V22), seed=seed)
    optimizer = torch.optim.Adam(selector.parameters(), lr=lr)
    loss_fn = torch.nn.CrossEntropyLoss()

    x = torch.tensor(train_x, dtype=torch.float32)
    y = torch.tensor(train_y, dtype=torch.long)
    for _ in range(epochs):
        optimizer.zero_grad()
        logits = selector(x)
        loss = loss_fn(logits, y)
        loss.backward()
        optimizer.step()
    return selector


def extract_geometry_features_v33(
    inputs: np.ndarray,
    model: GeometryNativeSequenceModelV3,
    *,
    support_window: int = 10,
    calibration_size: int = 256,
    calibration_seed: int = 307,
    field_seed: int = 0,
    selector_seed: int = 0,
) -> tuple[np.ndarray, int]:
    calibration_inputs, calibration_targets, calibration_query_mask = generate_dataset_transfer_v27(
        calibration_size,
        seq_len=inputs.shape[1],
        seed=calibration_seed,
    )
    train_x_v30, train_y_v30 = _build_training_set_v30(
        calibration_inputs,
        calibration_targets,
        calibration_query_mask,
        model,
        support_window=support_window,
    )
    chart_field = _fit_structured_chart_field(train_x_v30, train_y_v30, seed=field_seed)
    train_x_region, train_y_region = _build_region_training_set_v33(
        calibration_inputs,
        calibration_targets,
        calibration_query_mask,
        model,
        chart_field=chart_field,
        support_window=support_window,
    )
    selector = _fit_region_interaction_selector(train_x_region, train_y_region, seed=selector_seed)

    size, seq_len = inputs.shape
    features: list[np.ndarray] = []
    for row in range(size):
        raw_snapshots = _collect_raw_snapshots_v27(inputs[row])
        block_features, _block_caches, bounds = _block_details_v30(
            raw_snapshots,
            model,
            support_window=support_window,
        )
        x = torch.tensor(block_features[None, ...], dtype=torch.float32)
        with torch.no_grad():
            logits = chart_field(x).squeeze(0).cpu().numpy()
        block_labels = _decode_contiguous_chart_path_v30(logits)
        regions = _collapse_regions_v30(block_labels, bounds)

        region_summaries: list[np.ndarray] = []
        region_variant_caches: list[dict[str, tuple[np.ndarray, np.ndarray, dict[str, np.ndarray]]]] = []
        for start, end, _variant in regions:
            summary, cache = _region_cache_v33(
                raw_snapshots[start:end],
                model,
                support_window=support_window,
            )
            region_summaries.append(summary)
            region_variant_caches.append(cache)

        combined_regions: list[np.ndarray] = []
        for region_idx, (_start, _end, _variant) in enumerate(regions):
            selector_x = torch.tensor(
                _interaction_features_v33(region_summaries, region_idx)[None, :],
                dtype=torch.float32,
            )
            with torch.no_grad():
                variant_idx = int(torch.argmax(selector(selector_x), dim=-1).item())
            combined_regions.append(region_variant_caches[region_idx][REGION_VARIANTS_V22[variant_idx]][0])

        combined = np.concatenate(combined_regions, axis=0)
        features.extend(combined)

    total_params = chart_field.param_count + selector.param_count
    return np.stack(features, axis=0).reshape(size, seq_len, -1), total_params


def evaluate_geometry_v33(
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


def run_bounded_sequence_comparison_v33(config: TaskConfigV27, *, seed: int = 0) -> list[SequenceMetricsV33]:
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

    test_features, field_param_count = extract_geometry_features_v33(
        test_inputs,
        geometry_model,
        calibration_seed=seed + 109,
        field_seed=seed + 113,
        selector_seed=seed + 127,
    )
    geometry_test_loss, geometry_test_acc, geometry_transfer_query_acc, geometry_eval_seconds = evaluate_geometry_v33(
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
    total_state_size = geometry_model.effective_state_size + 8

    return [
        SequenceMetricsV33(
            model="geometry_native_sequence_model_v33",
            test_loss=geometry_test_loss,
            test_accuracy=geometry_test_acc,
            transfer_query_accuracy=geometry_transfer_query_acc,
            param_count=total_param_count,
            effective_state_size=total_state_size,
            train_seconds=geometry_train_seconds,
            eval_seconds=geometry_eval_seconds,
        ),
        SequenceMetricsV33(
            model="tiny_transformer_baseline_v33",
            test_loss=transformer_test_loss,
            test_accuracy=transformer_test_acc,
            transfer_query_accuracy=transformer_transfer_query_acc,
            param_count=transformer.param_count,
            effective_state_size=transformer.effective_state_size,
            train_seconds=transformer_train_seconds,
            eval_seconds=transformer_eval_seconds,
        ),
    ]
