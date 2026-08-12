#!/usr/bin/env python3
"""Multi-axis held-out generalization test for the discourse-style family."""

from __future__ import annotations

from dataclasses import dataclass
from time import perf_counter

import numpy as np
import torch

from geometry_native_sequence_model_v3 import (
    GeometryNativeSequenceModelV3,
    TOKENS_V3,
    TransformerSequenceTrainerV3,
    _build_feature_vector_v3,
    _initial_discourse_state,
    _initial_geometry_state,
    step_discourse_world,
)
from geometry_native_sequence_model_v4 import BASE_WEIGHTS_V4, heldout_query_condition


torch.set_num_threads(1)


SHIFT_STYLE_ID = TOKENS_V3.index("SHIFT_STYLE")


@dataclass(frozen=True)
class TaskConfigV5:
    seq_len_train: int = 26
    seq_len_test: int = 42
    train_size: int = 1024
    test_size: int = 256
    max_train_style_shifts: int = 1
    min_test_style_shifts: int = 4
    min_test_heldout_queries: int = 3


@dataclass
class SequenceMetricsV5:
    model: str
    train_loss: float
    test_loss: float
    test_accuracy: float
    heldout_query_accuracy: float
    param_count: int
    effective_state_size: int
    train_seconds: float
    eval_seconds: float


def generate_dataset_v5(
    size: int,
    *,
    seq_len: int,
    seed: int,
    mode: str,
    max_train_style_shifts: int,
    min_test_style_shifts: int,
    min_test_heldout_queries: int,
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
        style_shift_count = 0
        valid = True

        for _ in range(seq_len):
            for _attempt in range(64):
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
                proposed_style_shifts = style_shift_count + (1 if token_id == SHIFT_STYLE_ID else 0)

                if mode == "train":
                    if is_heldout:
                        continue
                    if proposed_style_shifts > max_train_style_shifts:
                        continue
                break
            else:
                valid = False
                break

            seq_inputs.append(token_id)
            seq_targets.append(int(target))
            seq_mask.append(1.0 if is_heldout else 0.0)
            heldout_count += 1 if is_heldout else 0
            if token_id == SHIFT_STYLE_ID:
                style_shift_count += 1

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

        if mode == "test":
            if heldout_count < min_test_heldout_queries:
                continue
            if style_shift_count < min_test_style_shifts:
                continue

        inputs[row] = np.array(seq_inputs, dtype=np.int64)
        targets[row] = np.array(seq_targets, dtype=np.int64)
        heldout_query_mask[row] = np.array(seq_mask, dtype=np.float32)
        row += 1

    return inputs, targets, heldout_query_mask


def extract_geometry_features_v5(inputs: np.ndarray) -> np.ndarray:
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


def evaluate_geometry_v5(
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


def evaluate_transformer_v5(
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


def run_bounded_sequence_comparison_v5(config: TaskConfigV5, *, seed: int = 0) -> list[SequenceMetricsV5]:
    train_inputs, train_targets, _ = generate_dataset_v5(
        config.train_size,
        seq_len=config.seq_len_train,
        seed=seed,
        mode="train",
        max_train_style_shifts=config.max_train_style_shifts,
        min_test_style_shifts=config.min_test_style_shifts,
        min_test_heldout_queries=config.min_test_heldout_queries,
    )
    test_inputs, test_targets, test_heldout_mask = generate_dataset_v5(
        config.test_size,
        seq_len=config.seq_len_test,
        seed=seed + 1,
        mode="test",
        max_train_style_shifts=config.max_train_style_shifts,
        min_test_style_shifts=config.min_test_style_shifts,
        min_test_heldout_queries=config.min_test_heldout_queries,
    )

    train_features = extract_geometry_features_v5(train_inputs)
    test_features = extract_geometry_features_v5(test_inputs)

    geometry_model = GeometryNativeSequenceModelV3(train_features.shape[-1], seed=seed)
    geometry_train_loss, geometry_train_seconds = geometry_model.fit(train_features, train_targets)
    geometry_test_loss, geometry_test_acc, geometry_heldout_query_acc, geometry_eval_seconds = evaluate_geometry_v5(
        geometry_model,
        test_features,
        test_targets,
        test_heldout_mask,
    )

    transformer = TransformerSequenceTrainerV3(seq_len=config.seq_len_test, seed=seed)
    transformer_train_loss, transformer_train_seconds = transformer.fit(train_inputs, train_targets)
    transformer_test_loss, transformer_test_acc, transformer_heldout_query_acc, transformer_eval_seconds = evaluate_transformer_v5(
        transformer,
        test_inputs,
        test_targets,
        test_heldout_mask,
    )

    return [
        SequenceMetricsV5(
            model="geometry_native_sequence_model_v5",
            train_loss=geometry_train_loss,
            test_loss=geometry_test_loss,
            test_accuracy=geometry_test_acc,
            heldout_query_accuracy=geometry_heldout_query_acc,
            param_count=geometry_model.param_count,
            effective_state_size=geometry_model.effective_state_size,
            train_seconds=geometry_train_seconds,
            eval_seconds=geometry_eval_seconds,
        ),
        SequenceMetricsV5(
            model="tiny_transformer_baseline_v5",
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
