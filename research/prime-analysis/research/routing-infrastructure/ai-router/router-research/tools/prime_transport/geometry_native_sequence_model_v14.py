#!/usr/bin/env python3
"""Within-sequence bridge switching on the stronger reduced-alignment setting."""

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
from geometry_native_sequence_model_v12 import (
    TaskConfigV12,
    _initial_discourse_state,
    _initial_geometry_state,
    evaluate_transformer_v12,
    generate_dataset_transfer_v12,
    step_entangled_transfer_world_v12,
)
from geometry_native_sequence_model_v13 import _bridged_snapshot, _selection_score


torch.set_num_threads(1)


@dataclass
class SequenceMetricsV14:
    model: str
    test_loss: float
    test_accuracy: float
    transfer_query_accuracy: float
    param_count: int
    effective_state_size: int
    train_seconds: float
    eval_seconds: float


def _variant_features(
    raw_snapshots: list[tuple[int, dict[str, int | list[int]]]],
    variant: str,
) -> tuple[np.ndarray, list[dict[str, int | list[int]]], list[int]]:
    adjusted_snapshots = [_bridged_snapshot(token_id, snapshot, variant=variant) for token_id, snapshot in raw_snapshots]
    features = np.stack(
        [
            _build_feature_vector_v3(token_id, adjusted_snapshot)
            for (token_id, _), adjusted_snapshot in zip(raw_snapshots, adjusted_snapshots)
        ],
        axis=0,
    )
    tokens = [token_id for token_id, _ in raw_snapshots]
    return features, adjusted_snapshots, tokens


def extract_geometry_features_v14(
    inputs: np.ndarray,
    model: GeometryNativeSequenceModelV3,
    *,
    support_window: int = 10,
) -> np.ndarray:
    size, seq_len = inputs.shape
    features: list[np.ndarray] = []
    variants = ("v11_like", "v12_base", "hybrid")

    for row in range(size):
        b, phi, r = _initial_geometry_state()
        focus, speaker, topic, style, gap, tags = _initial_discourse_state()
        raw_snapshots: list[tuple[int, dict[str, int | list[int]]]] = []

        for col in range(seq_len):
            token_id = int(inputs[row, col])
            _, snapshot, _ = step_entangled_transfer_world_v12(
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

        mid = seq_len // 2
        prefix_raw = raw_snapshots[:mid]
        suffix_raw = raw_snapshots[mid:]

        prefix_scores: list[float] = []
        prefix_variants: list[np.ndarray] = []
        for variant in variants:
            prefix_features, prefix_snapshots, prefix_tokens = _variant_features(prefix_raw, variant)
            prefix_scores.append(
                _selection_score(
                    model,
                    prefix_features,
                    prefix_snapshots,
                    prefix_tokens,
                    support_window=support_window,
                )
            )
            prefix_variants.append(prefix_features)
        prefix_best_idx = int(np.argmax(np.array(prefix_scores)))

        suffix_scores: list[float] = []
        suffix_variants: list[np.ndarray] = []
        for variant in variants:
            suffix_features, suffix_snapshots, suffix_tokens = _variant_features(suffix_raw, variant)
            suffix_scores.append(
                _selection_score(
                    model,
                    suffix_features,
                    suffix_snapshots,
                    suffix_tokens,
                    support_window=min(support_window, max(4, len(suffix_features))),
                )
            )
            suffix_variants.append(suffix_features)
        suffix_best_idx = int(np.argmax(np.array(suffix_scores)))

        combined = np.concatenate(
            [
                prefix_variants[prefix_best_idx],
                suffix_variants[suffix_best_idx],
            ],
            axis=0,
        )
        features.extend(combined)

    return np.stack(features, axis=0).reshape(size, seq_len, -1)


def evaluate_geometry_v14(
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


def run_bounded_sequence_comparison_v14(config: TaskConfigV12, *, seed: int = 0) -> list[SequenceMetricsV14]:
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

    test_features = extract_geometry_features_v14(test_inputs, geometry_model)
    geometry_test_loss, geometry_test_acc, geometry_transfer_query_acc, geometry_eval_seconds = evaluate_geometry_v14(
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
        SequenceMetricsV14(
            model="geometry_native_sequence_model_v14",
            test_loss=geometry_test_loss,
            test_accuracy=geometry_test_acc,
            transfer_query_accuracy=geometry_transfer_query_acc,
            param_count=geometry_model.param_count,
            effective_state_size=geometry_model.effective_state_size,
            train_seconds=geometry_train_seconds,
            eval_seconds=geometry_eval_seconds,
        ),
        SequenceMetricsV14(
            model="tiny_transformer_baseline_v14",
            test_loss=transformer_test_loss,
            test_accuracy=transformer_test_acc,
            transfer_query_accuracy=transformer_transfer_query_acc,
            param_count=transformer.param_count,
            effective_state_size=transformer.effective_state_size,
            train_seconds=transformer_train_seconds,
            eval_seconds=transformer_eval_seconds,
        ),
    ]
