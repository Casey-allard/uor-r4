#!/usr/bin/env python3
"""Held-out generalization test for the discourse-style geometry-native model."""

from __future__ import annotations

from dataclasses import dataclass
from time import perf_counter

import numpy as np
import torch

from geometry_native_sequence_model_v3 import (
    GeometryNativeSequenceModelV3,
    QUERY_IDS_V3,
    TOKENS_V3,
    TaskConfigV3,
    TOKEN_TO_ID_V3,
    TransformerSequenceTrainerV3,
    _initial_discourse_state,
    _initial_geometry_state,
    _build_feature_vector_v3,
    step_discourse_world,
)


torch.set_num_threads(1)


@dataclass(frozen=True)
class TaskConfigV4:
    seq_len_train: int = 30
    seq_len_test: int = 34
    train_size: int = 1024
    test_size: int = 256


@dataclass
class SequenceMetricsV4:
    model: str
    train_loss: float
    test_loss: float
    test_accuracy: float
    heldout_query_accuracy: float
    param_count: int
    effective_state_size: int
    train_seconds: float
    eval_seconds: float


BASE_WEIGHTS_V4 = np.array(
    [0.11, 0.11, 0.11, 0.08, 0.08, 0.09, 0.08, 0.08, 0.08, 0.09, 0.09],
    dtype=np.float64,
)
BASE_WEIGHTS_V4 /= BASE_WEIGHTS_V4.sum()


def heldout_query_condition(snapshot: dict[str, int | list[int]], token_id: int) -> bool:
    if token_id not in QUERY_IDS_V3:
        return False
    return (
        int(snapshot["style"]) == 2
        and int(snapshot["referent_role"]) == 1
        and int(snapshot["speaker"]) != int(snapshot["topic"])
    )


def generate_dataset_v4(
    size: int,
    *,
    seq_len: int,
    seed: int,
    mode: str,
) -> tuple[np.ndarray, np.ndarray, np.ndarray]:
    rng = np.random.default_rng(seed)
    inputs = np.zeros((size, seq_len), dtype=np.int64)
    targets = np.zeros((size, seq_len), dtype=np.int64)
    heldout_query_mask = np.zeros((size, seq_len), dtype=np.float32)

    row = 0
    while row < size:
        b, phi, r = _initial_geometry_state()
        focus, speaker, topic, style, gap, tags = _initial_discourse_state()

        seq_inputs: list[int] = []
        seq_targets: list[int] = []
        seq_mask: list[float] = []
        heldout_count = 0
        valid = True

        for _ in range(seq_len):
            for _attempt in range(32):
                token_id = int(rng.choice(len(TOKENS_V3), p=BASE_WEIGHTS_V4))
                target, snapshot, _ = step_discourse_world(
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
                is_heldout = heldout_query_condition(snapshot, token_id)
                if mode == "train" and is_heldout:
                    continue
                break
            else:
                valid = False
                break

            seq_inputs.append(token_id)
            seq_targets.append(int(target))
            seq_mask.append(1.0 if is_heldout else 0.0)
            heldout_count += 1 if is_heldout else 0

            focus = int(snapshot["focus"])
            speaker = int(snapshot["speaker"])
            topic = int(snapshot["topic"])
            style = int(snapshot["style"])
            tags = list(snapshot["tags"])
            b = int(snapshot["next_b"])
            phi = int(snapshot["next_phi"])
            r = int(snapshot["next_r"])
            gap = int(snapshot["next_next_return_gap"])

        if not valid:
            continue

        if mode == "test" and heldout_count < 2:
            continue

        inputs[row] = np.array(seq_inputs, dtype=np.int64)
        targets[row] = np.array(seq_targets, dtype=np.int64)
        heldout_query_mask[row] = np.array(seq_mask, dtype=np.float32)
        row += 1

    return inputs, targets, heldout_query_mask


def extract_geometry_features_v4(inputs: np.ndarray) -> np.ndarray:
    size, seq_len = inputs.shape
    features: list[np.ndarray] = []

    for row in range(size):
        b, phi, r = _initial_geometry_state()
        focus, speaker, topic, style, gap, tags = _initial_discourse_state()
        for col in range(seq_len):
            token_id = int(inputs[row, col])
            _, snapshot, _ = step_discourse_world(
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
            features.append(_build_feature_vector_v3(token_id, snapshot))
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


def evaluate_geometry_v4(
    model: GeometryNativeSequenceModelV3,
    features: np.ndarray,
    targets: np.ndarray,
    heldout_mask: np.ndarray,
) -> tuple[float, float, float, float]:
    x = torch.tensor(features.reshape(-1, features.shape[-1]), dtype=torch.float32)
    y = torch.tensor(targets.reshape(-1), dtype=torch.long)
    q = torch.tensor(heldout_mask.reshape(-1), dtype=torch.float32)

    start = perf_counter()
    with torch.no_grad():
        logits = model.readout(x)
        loss = float(model.loss_fn(logits, y).item())
        predictions = logits.argmax(dim=-1)
        accuracy = float((predictions == y).float().mean().item())
        heldout_query_accuracy = float((((predictions == y).float() * q).sum() / q.sum()).item())
    elapsed = perf_counter() - start
    return loss, accuracy, heldout_query_accuracy, elapsed


def evaluate_transformer_v4(
    trainer: TransformerSequenceTrainerV3,
    inputs: np.ndarray,
    targets: np.ndarray,
    heldout_mask: np.ndarray,
) -> tuple[float, float, float, float]:
    x = torch.tensor(inputs, dtype=torch.long)
    y = torch.tensor(targets, dtype=torch.long)
    q = torch.tensor(heldout_mask, dtype=torch.float32)

    start = perf_counter()
    with torch.no_grad():
        logits = trainer.model(x)
        loss = float(trainer.loss_fn(logits.reshape(-1, logits.shape[-1]), y.reshape(-1)).item())
        predictions = logits.argmax(dim=-1)
        accuracy = float((predictions == y).float().mean().item())
        heldout_query_accuracy = float((((predictions == y).float() * q).sum() / q.sum()).item())
    elapsed = perf_counter() - start
    return loss, accuracy, heldout_query_accuracy, elapsed


def run_bounded_sequence_comparison_v4(config: TaskConfigV4, *, seed: int = 0) -> list[SequenceMetricsV4]:
    train_inputs, train_targets, _ = generate_dataset_v4(
        config.train_size,
        seq_len=config.seq_len_train,
        seed=seed,
        mode="train",
    )
    test_inputs, test_targets, test_heldout_mask = generate_dataset_v4(
        config.test_size,
        seq_len=config.seq_len_test,
        seed=seed + 1,
        mode="test",
    )

    train_features = extract_geometry_features_v4(train_inputs)
    test_features = extract_geometry_features_v4(test_inputs)

    geometry_model = GeometryNativeSequenceModelV3(train_features.shape[-1], seed=seed)
    geometry_train_loss, geometry_train_seconds = geometry_model.fit(train_features, train_targets)
    geometry_test_loss, geometry_test_acc, geometry_heldout_query_acc, geometry_eval_seconds = evaluate_geometry_v4(
        geometry_model,
        test_features,
        test_targets,
        test_heldout_mask,
    )

    transformer = TransformerSequenceTrainerV3(seq_len=config.seq_len_test, seed=seed)
    transformer_train_loss, transformer_train_seconds = transformer.fit(train_inputs, train_targets)
    transformer_test_loss, transformer_test_acc, transformer_heldout_query_acc, transformer_eval_seconds = evaluate_transformer_v4(
        transformer,
        test_inputs,
        test_targets,
        test_heldout_mask,
    )

    return [
        SequenceMetricsV4(
            model="geometry_native_sequence_model_v4",
            train_loss=geometry_train_loss,
            test_loss=geometry_test_loss,
            test_accuracy=geometry_test_acc,
            heldout_query_accuracy=geometry_heldout_query_acc,
            param_count=geometry_model.param_count,
            effective_state_size=geometry_model.effective_state_size,
            train_seconds=geometry_train_seconds,
            eval_seconds=geometry_eval_seconds,
        ),
        SequenceMetricsV4(
            model="tiny_transformer_baseline_v4",
            train_loss=transformer_train_loss,
            test_loss=transformer_test_loss,
            test_accuracy=transformer_test_acc,
            heldout_query_accuracy=transformer_heldout_query_acc,
            param_count=transformer.param_count,
            effective_state_size=transformer.effective_state_size,
            train_seconds=transformer_train_seconds,
            eval_seconds=transformer_eval_seconds,
        ),
    ]
