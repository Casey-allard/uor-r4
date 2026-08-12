#!/usr/bin/env python3
"""Slightly richer learned regional regime field on the stronger shifted family."""

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
from geometry_native_sequence_model_v26 import _sequence_objective_full
from geometry_native_sequence_model_v27 import (
    BOUNDARY_CANDIDATES_V27,
    TaskConfigV27,
    _boundary_candidate_details_v27,
    _fit_boundary_library_v27,
    _build_boundary_training_set_v27,
    _collect_raw_snapshots_v27,
    generate_dataset_transfer_v27,
)


torch.set_num_threads(1)


@dataclass
class SequenceMetricsV28:
    model: str
    test_loss: float
    test_accuracy: float
    transfer_query_accuracy: float
    param_count: int
    effective_state_size: int
    train_seconds: float
    eval_seconds: float


class RichRegionalBoundaryField(torch.nn.Module):
    def __init__(self, feature_dim: int, *, seed: int) -> None:
        super().__init__()
        torch.manual_seed(seed)
        self.input_proj = torch.nn.Linear(feature_dim, 24)
        self.hidden = torch.nn.Linear(24, 16)
        self.out = torch.nn.Linear(16, 1)

    def forward(self, x: torch.Tensor) -> torch.Tensor:
        base = torch.tanh(self.input_proj(x))
        mixed = torch.tanh(self.hidden(base) + base[:, :, :16])
        return self.out(mixed).squeeze(-1)

    @property
    def param_count(self) -> int:
        return sum(parameter.numel() for parameter in self.parameters())


def _fit_rich_boundary_field(
    train_x: np.ndarray,
    train_y: np.ndarray,
    *,
    seed: int,
    epochs: int = 180,
    lr: float = 0.02,
) -> RichRegionalBoundaryField:
    model = RichRegionalBoundaryField(train_x.shape[-1], seed=seed)
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


def extract_geometry_features_v28(
    inputs: np.ndarray,
    model: GeometryNativeSequenceModelV3,
    *,
    support_window: int = 10,
    calibration_size: int = 256,
    calibration_seed: int = 257,
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
    boundary_field = _fit_rich_boundary_field(train_x, train_y, seed=field_seed)

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


def evaluate_geometry_v28(
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


def run_bounded_sequence_comparison_v28(config: TaskConfigV27, *, seed: int = 0) -> list[SequenceMetricsV28]:
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

    test_features, field_param_count = extract_geometry_features_v28(
        test_inputs,
        geometry_model,
        calibration_seed=seed + 53,
        field_seed=seed + 59,
    )
    geometry_test_loss, geometry_test_acc, geometry_transfer_query_acc, geometry_eval_seconds = evaluate_geometry_v28(
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
        SequenceMetricsV28(
            model="geometry_native_sequence_model_v28",
            test_loss=geometry_test_loss,
            test_accuracy=geometry_test_acc,
            transfer_query_accuracy=geometry_transfer_query_acc,
            param_count=total_param_count,
            effective_state_size=total_state_size,
            train_seconds=geometry_train_seconds,
            eval_seconds=geometry_eval_seconds,
        ),
        SequenceMetricsV28(
            model="tiny_transformer_baseline_v28",
            test_loss=transformer_test_loss,
            test_accuracy=transformer_test_acc,
            transfer_query_accuracy=transformer_transfer_query_acc,
            param_count=transformer.param_count,
            effective_state_size=transformer.effective_state_size,
            train_seconds=transformer_train_seconds,
            eval_seconds=transformer_eval_seconds,
        ),
    ]
