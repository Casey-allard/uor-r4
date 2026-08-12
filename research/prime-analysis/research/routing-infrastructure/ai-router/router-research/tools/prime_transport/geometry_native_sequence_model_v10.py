#!/usr/bin/env python3
"""Chart-calibrated geometry-native transfer test on the reduced-alignment setting."""

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
from geometry_native_sequence_model_v7 import TaskConfigV7, generate_dataset_transfer_v7, step_entangled_transfer_world
from geometry_native_sequence_model_v8 import _candidate_entities, _chart_rotation, _proxy_role


torch.set_num_threads(1)


@dataclass
class SequenceMetricsV10:
    model: str
    test_loss: float
    test_accuracy: float
    transfer_query_accuracy: float
    param_count: int
    effective_state_size: int
    train_seconds: float
    eval_seconds: float


def _candidate_snapshot(snapshot: dict[str, int | list[int]], offset: int) -> dict[str, int | list[int]]:
    base_rotation = _chart_rotation(
        int(snapshot["b"]),
        int(snapshot["r"]),
        int(snapshot["style"]),
    )
    proxy_role = _proxy_role(
        int(snapshot["style"]),
        int(snapshot["phi"]),
        int(snapshot["next_return_gap"]),
    )
    realigned_role = (proxy_role + base_rotation + offset) % 3
    basis = _candidate_entities(
        int(snapshot["focus"]),
        int(snapshot["speaker"]),
        int(snapshot["topic"]),
    )
    realigned_referent = basis[realigned_role]
    adjusted = dict(snapshot)
    adjusted["referent_role"] = realigned_role
    adjusted["referent_entity"] = realigned_referent
    return adjusted


def _selection_score(
    model: GeometryNativeSequenceModelV3,
    candidate_features: np.ndarray,
    candidate_snapshots: list[dict[str, int | list[int]]],
    candidate_tokens: list[int],
    *,
    calibration_window: int,
) -> float:
    window = min(calibration_window, len(candidate_features))
    x = torch.tensor(candidate_features[:window], dtype=torch.float32)
    with torch.no_grad():
        logits = model.readout(x)
        probs = torch.softmax(logits, dim=-1)
        max_probs = probs.max(dim=-1).values
        pred_labels = logits.argmax(dim=-1).cpu().numpy()

    query_indices = [idx for idx, token in enumerate(candidate_tokens[:window]) if token in {9, 10}]
    if query_indices:
        query_conf = float(max_probs[query_indices].mean().item())
    else:
        query_conf = float(max_probs.mean().item())

    global_conf = float(max_probs.mean().item())

    grouped_predictions: dict[tuple[int, int], list[int]] = {}
    for idx in range(window):
        snap = candidate_snapshots[idx]
        key = (int(snap["referent_role"]), int(snap["referent_entity"]))
        grouped_predictions.setdefault(key, []).append(int(pred_labels[idx]))

    disagreement = 0.0
    groups = 0
    for values in grouped_predictions.values():
        if len(values) < 2:
            continue
        groups += 1
        disagreement += sum(1 for value in values[1:] if value != values[0]) / (len(values) - 1)
    disagreement_penalty = disagreement / groups if groups else 0.0

    return query_conf + 0.35 * global_conf - 0.4 * disagreement_penalty


def extract_geometry_features_v10(
    inputs: np.ndarray,
    model: GeometryNativeSequenceModelV3,
    *,
    calibration_window: int = 8,
) -> np.ndarray:
    size, seq_len = inputs.shape
    features: list[np.ndarray] = []

    from geometry_native_sequence_model_v3 import _initial_discourse_state, _initial_geometry_state

    with torch.no_grad():
        for row in range(size):
            b, phi, r = _initial_geometry_state()
            focus, speaker, topic, style, gap, tags = _initial_discourse_state()
            raw_snapshots: list[tuple[int, dict[str, int | list[int]]]] = []

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
                raw_snapshots.append((token_id, snapshot))
                focus = int(snapshot["focus"])
                speaker = int(snapshot["speaker"])
                topic = int(snapshot["topic"])
                style = int(snapshot["style"])
                tags = list(snapshot["tags"])
                b = int(snapshot["next_b"])
                phi = int(snapshot["next_phi"])
                r = int(snapshot["next_r"])
                gap = int(snapshot["next_next_return_gap"])

            candidates: list[np.ndarray] = []
            candidate_scores: list[float] = []
            for offset in range(3):
                adjusted_snapshots = [_candidate_snapshot(snapshot, offset) for _, snapshot in raw_snapshots]
                candidate_features = np.stack(
                    [
                        _build_feature_vector_v3(token_id, adjusted_snapshot)
                        for (token_id, _), adjusted_snapshot in zip(raw_snapshots, adjusted_snapshots)
                    ],
                    axis=0,
                )
                candidate_tokens = [token_id for token_id, _ in raw_snapshots]
                score = _selection_score(
                    model,
                    candidate_features,
                    adjusted_snapshots,
                    candidate_tokens,
                    calibration_window=calibration_window,
                )
                candidates.append(candidate_features)
                candidate_scores.append(score)

            best_idx = int(np.argmax(np.array(candidate_scores)))
            features.extend(candidates[best_idx])

    return np.stack(features, axis=0).reshape(size, seq_len, -1)


def evaluate_geometry_v10(
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


def evaluate_transformer_v10(
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


def run_bounded_sequence_comparison_v10(config: TaskConfigV7, *, seed: int = 0) -> list[SequenceMetricsV10]:
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
    geometry_model = GeometryNativeSequenceModelV3(train_features.shape[-1], seed=seed)
    _train_loss, geometry_train_seconds = geometry_model.fit(train_features, train_targets)

    test_features = extract_geometry_features_v10(test_inputs, geometry_model)
    geometry_test_loss, geometry_test_acc, geometry_transfer_query_acc, geometry_eval_seconds = evaluate_geometry_v10(
        geometry_model,
        test_features,
        test_targets,
        test_query_mask,
    )

    transformer = TransformerSequenceTrainerV3(seq_len=config.test_seq_len, seed=seed)
    _transformer_train_loss, transformer_train_seconds = transformer.fit(train_inputs, train_targets)
    transformer_test_loss, transformer_test_acc, transformer_transfer_query_acc, transformer_eval_seconds = evaluate_transformer_v10(
        transformer,
        test_inputs,
        test_targets,
        test_query_mask,
    )

    return [
        SequenceMetricsV10(
            model="geometry_native_sequence_model_v10",
            test_loss=geometry_test_loss,
            test_accuracy=geometry_test_acc,
            transfer_query_accuracy=geometry_transfer_query_acc,
            param_count=geometry_model.param_count,
            effective_state_size=geometry_model.effective_state_size,
            train_seconds=geometry_train_seconds,
            eval_seconds=geometry_eval_seconds,
        ),
        SequenceMetricsV10(
            model="tiny_transformer_baseline_v10",
            test_loss=transformer_test_loss,
            test_accuracy=transformer_test_acc,
            transfer_query_accuracy=transformer_transfer_query_acc,
            param_count=transformer.param_count,
            effective_state_size=transformer.effective_state_size,
            train_seconds=transformer_train_seconds,
            eval_seconds=transformer_eval_seconds,
        ),
    ]
