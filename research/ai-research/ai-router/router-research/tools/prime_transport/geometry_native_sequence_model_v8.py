#!/usr/bin/env python3
"""Geometry-native chart realignment test on the reduced-alignment transfer setting."""

from __future__ import annotations

from dataclasses import dataclass
from time import perf_counter

import numpy as np
import torch

from geometry_native_sequence_model_v3 import (
    GeometryNativeSequenceModelV3,
    TransformerSequenceTrainerV3,
    _build_feature_vector_v3,
    extract_geometry_features_v3,
    generate_dataset_v3,
)
from geometry_native_sequence_model_v7 import (
    TaskConfigV7,
    generate_dataset_transfer_v7,
    step_entangled_transfer_world,
)


torch.set_num_threads(1)


@dataclass
class SequenceMetricsV8:
    model: str
    train_loss: float
    test_loss: float
    test_accuracy: float
    transfer_query_accuracy: float
    param_count: int
    effective_state_size: int
    train_seconds: float
    eval_seconds: float


def _candidate_entities(focus: int, speaker: int, topic: int) -> tuple[int, int, int]:
    return (
        focus,
        (speaker + topic) % 3,
        (focus + speaker + 1) % 3,
    )


def _proxy_role(style: int, phi: int, next_return_gap: int) -> int:
    return (style + phi + next_return_gap) % 3


def _chart_rotation(b: int, r: int, style: int) -> int:
    # Small geometry-native chart selector: one of three local angular role charts.
    return (b + r + style) % 3


def extract_geometry_features_v8(inputs: np.ndarray) -> np.ndarray:
    size, seq_len = inputs.shape
    features: list[np.ndarray] = []

    from geometry_native_sequence_model_v3 import _initial_discourse_state, _initial_geometry_state

    for row in range(size):
        b, phi, r = _initial_geometry_state()
        focus, speaker, topic, style, gap, tags = _initial_discourse_state()
        for col in range(seq_len):
            token_id = int(inputs[row, col])
            _, snapshot, _ = step_entangled_transfer_world(
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

            chart_rotation = _chart_rotation(
                int(snapshot["b"]),
                int(snapshot["r"]),
                int(snapshot["style"]),
            )
            proxy_role = _proxy_role(
                int(snapshot["style"]),
                int(snapshot["phi"]),
                int(snapshot["next_return_gap"]),
            )
            realigned_role = (proxy_role + chart_rotation) % 3
            basis = _candidate_entities(
                int(snapshot["focus"]),
                int(snapshot["speaker"]),
                int(snapshot["topic"]),
            )
            realigned_referent = basis[realigned_role]

            # Present the existing engine with a locally realigned chart rather than
            # the lossy direct projection used in v7.
            realigned_snapshot = dict(snapshot)
            realigned_snapshot["referent_role"] = realigned_role
            realigned_snapshot["referent_entity"] = realigned_referent
            features.append(_build_feature_vector_v3(token_id, realigned_snapshot))

            focus = int(snapshot["focus"])
            speaker = int(snapshot["speaker"])
            topic = int(snapshot["topic"])
            style = int(snapshot["style"])
            tags = list(snapshot["tags"])
            b = int(snapshot["next_b"])
            phi = int(snapshot["next_phi"])
            r = int(snapshot["next_r"])
            gap = int(snapshot["next_next_return_gap"])

    return np.stack(features, axis=0).reshape(size, seq_len, -1)


def evaluate_geometry_v8(
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


def evaluate_transformer_v8(
    trainer: TransformerSequenceTrainerV3,
    inputs: np.ndarray,
    targets: np.ndarray,
    query_mask: np.ndarray,
) -> tuple[float, float, float, float]:
    x = torch.tensor(inputs, dtype=torch.long)
    y = torch.tensor(targets, dtype=torch.long)
    q = torch.tensor(query_mask, dtype=torch.float32)

    start = perf_counter()
    with torch.no_grad():
        logits = trainer.model(x)
        loss = float(trainer.loss_fn(logits.reshape(-1, logits.shape[-1]), y.reshape(-1)).item())
        predictions = logits.argmax(dim=-1)
        accuracy = float((predictions == y).float().mean().item())
        transfer_query_accuracy = float((((predictions == y).float() * q).sum() / q.sum()).item())
    elapsed = perf_counter() - start
    return loss, accuracy, transfer_query_accuracy, elapsed


def run_bounded_sequence_comparison_v8(config: TaskConfigV7, *, seed: int = 0) -> list[SequenceMetricsV8]:
    train_inputs, train_targets, _ = generate_dataset_v3(
        config.train_size,
        seq_len=config.train_seq_len,
        seed=seed,
    )
    test_inputs, test_targets, test_query_mask = generate_dataset_transfer_v7(
        config.test_size,
        seq_len=config.test_seq_len,
        seed=seed + 1,
    )

    train_features, _ = extract_geometry_features_v3(train_inputs)
    test_features = extract_geometry_features_v8(test_inputs)

    geometry_model = GeometryNativeSequenceModelV3(train_features.shape[-1], seed=seed)
    geometry_train_loss, geometry_train_seconds = geometry_model.fit(train_features, train_targets)
    geometry_test_loss, geometry_test_acc, geometry_transfer_query_acc, geometry_eval_seconds = evaluate_geometry_v8(
        geometry_model,
        test_features,
        test_targets,
        test_query_mask,
    )

    transformer = TransformerSequenceTrainerV3(seq_len=config.test_seq_len, seed=seed)
    transformer_train_loss, transformer_train_seconds = transformer.fit(train_inputs, train_targets)
    transformer_test_loss, transformer_test_acc, transformer_transfer_query_acc, transformer_eval_seconds = evaluate_transformer_v8(
        transformer,
        test_inputs,
        test_targets,
        test_query_mask,
    )

    return [
        SequenceMetricsV8(
            model="geometry_native_sequence_model_v8",
            train_loss=geometry_train_loss,
            test_loss=geometry_test_loss,
            test_accuracy=geometry_test_acc,
            transfer_query_accuracy=geometry_transfer_query_acc,
            param_count=geometry_model.param_count,
            effective_state_size=geometry_model.effective_state_size,
            train_seconds=geometry_train_seconds,
            eval_seconds=geometry_eval_seconds,
        ),
        SequenceMetricsV8(
            model="tiny_transformer_baseline_v8",
            train_loss=transformer_train_loss,
            test_loss=transformer_test_loss,
            test_accuracy=transformer_test_acc,
            transfer_query_accuracy=transformer_transfer_query_acc,
            param_count=transformer.param_count,
            effective_state_size=transformer.effective_state_size,
            train_seconds=transformer_train_seconds,
            eval_seconds=transformer_eval_seconds,
        ),
    ]
