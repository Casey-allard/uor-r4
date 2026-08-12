#!/usr/bin/env python3
"""Support-window bridge calibration on the stronger reduced-alignment setting."""

from __future__ import annotations

from dataclasses import dataclass
from time import perf_counter

import numpy as np
import torch

from geometry_native_sequence_model_v3 import (
    GeometryNativeSequenceModelV3,
    QUERY_IDS_V3,
    TransformerSequenceTrainerV3,
    _build_feature_vector_v3,
    extract_geometry_features_v3,
    generate_dataset_v3,
)
from geometry_native_sequence_model_v11 import _binding_bridge_delta as _binding_bridge_delta_v11
from geometry_native_sequence_model_v11 import _query_bridge_delta as _query_bridge_delta_v11
from geometry_native_sequence_model_v12 import (
    ROLE_PRIMES,
    SequenceMetricsV12,
    TaskConfigV12,
    TAG_IDS,
    _binding_bridge_delta_v12,
    _candidate_entities,
    _initial_discourse_state,
    _initial_geometry_state,
    _prime,
    _proxy_role_v12,
    _query_bridge_delta_v12,
    evaluate_transformer_v12,
    generate_dataset_transfer_v12,
    step_entangled_transfer_world_v12,
)


torch.set_num_threads(1)


@dataclass
class SequenceMetricsV13:
    model: str
    test_loss: float
    test_accuracy: float
    transfer_query_accuracy: float
    param_count: int
    effective_state_size: int
    train_seconds: float
    eval_seconds: float


def _bridged_snapshot(
    token_id: int,
    snapshot: dict[str, int | list[int]],
    *,
    variant: str,
) -> dict[str, int | list[int]]:
    proxy_role = _proxy_role_v12(
        int(snapshot["style"]),
        int(snapshot["phi"]),
        int(snapshot["next_return_gap"]),
        int(snapshot["b"]),
    )

    if token_id in QUERY_IDS_V3:
        delta_v11 = _query_bridge_delta_v11(snapshot)
        delta_v12 = _query_bridge_delta_v12(snapshot)
    elif token_id in TAG_IDS:
        delta_v11 = _binding_bridge_delta_v11(snapshot)
        delta_v12 = _binding_bridge_delta_v12(snapshot)
    else:
        delta_v11 = _query_bridge_delta_v11(snapshot)
        delta_v12 = _query_bridge_delta_v12(snapshot)

    if variant == "v11_like":
        delta = delta_v11
    elif variant == "v12_base":
        delta = delta_v12
    elif variant == "hybrid":
        delta = (delta_v11 + delta_v12) % 3
    else:
        raise ValueError(f"unknown bridge variant: {variant}")

    bridge_prime = _prime(delta)
    proxy_prime = _prime(proxy_role)
    bridge_semiprime = proxy_prime * bridge_prime

    if bridge_semiprime % (proxy_prime * ROLE_PRIMES[0]) == 0:
        bridged_delta = 0
    elif bridge_semiprime % (proxy_prime * ROLE_PRIMES[1]) == 0:
        bridged_delta = 1
    else:
        bridged_delta = 2

    bridged_role = (proxy_role + bridged_delta) % 3
    candidates = _candidate_entities(
        int(snapshot["focus"]),
        int(snapshot["speaker"]),
        int(snapshot["topic"]),
    )
    bridged_referent = candidates[bridged_role]

    adjusted = dict(snapshot)
    adjusted["referent_role"] = bridged_role
    adjusted["referent_entity"] = bridged_referent
    return adjusted


def _selection_score(
    model: GeometryNativeSequenceModelV3,
    candidate_features: np.ndarray,
    candidate_snapshots: list[dict[str, int | list[int]]],
    candidate_tokens: list[int],
    *,
    support_window: int,
) -> float:
    window = min(support_window, len(candidate_features))
    x = torch.tensor(candidate_features[:window], dtype=torch.float32)
    with torch.no_grad():
        logits = model.readout(x)
        probs = torch.softmax(logits, dim=-1)
        max_probs = probs.max(dim=-1).values
        preds = logits.argmax(dim=-1).cpu().numpy()

    query_indices = [idx for idx, token in enumerate(candidate_tokens[:window]) if token in QUERY_IDS_V3]
    query_conf = float(max_probs[query_indices].mean().item()) if query_indices else float(max_probs.mean().item())
    global_conf = float(max_probs.mean().item())

    grouped_predictions: dict[tuple[int, int], list[int]] = {}
    for idx in range(window):
        snap = candidate_snapshots[idx]
        key = (int(snap["referent_role"]), int(snap["referent_entity"]))
        grouped_predictions.setdefault(key, []).append(int(preds[idx]))

    disagreement = 0.0
    groups = 0
    for values in grouped_predictions.values():
        if len(values) < 2:
            continue
        groups += 1
        disagreement += sum(1 for value in values[1:] if value != values[0]) / (len(values) - 1)
    disagreement_penalty = disagreement / groups if groups else 0.0

    return query_conf + 0.4 * global_conf - 0.35 * disagreement_penalty


def extract_geometry_features_v13(
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

        candidate_features: list[np.ndarray] = []
        candidate_scores: list[float] = []
        for variant in variants:
            adjusted_snapshots = [_bridged_snapshot(token_id, snapshot, variant=variant) for token_id, snapshot in raw_snapshots]
            features_variant = np.stack(
                [
                    _build_feature_vector_v3(token_id, adjusted_snapshot)
                    for (token_id, _), adjusted_snapshot in zip(raw_snapshots, adjusted_snapshots)
                ],
                axis=0,
            )
            score = _selection_score(
                model,
                features_variant,
                adjusted_snapshots,
                [token_id for token_id, _ in raw_snapshots],
                support_window=support_window,
            )
            candidate_features.append(features_variant)
            candidate_scores.append(score)

        best_idx = int(np.argmax(np.array(candidate_scores)))
        features.extend(candidate_features[best_idx])

    return np.stack(features, axis=0).reshape(size, seq_len, -1)


def evaluate_geometry_v13(
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


def run_bounded_sequence_comparison_v13(config: TaskConfigV12, *, seed: int = 0) -> list[SequenceMetricsV13]:
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

    test_features = extract_geometry_features_v13(test_inputs, geometry_model)
    geometry_test_loss, geometry_test_acc, geometry_transfer_query_acc, geometry_eval_seconds = evaluate_geometry_v13(
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
        SequenceMetricsV13(
            model="geometry_native_sequence_model_v13",
            test_loss=geometry_test_loss,
            test_accuracy=geometry_test_acc,
            transfer_query_accuracy=geometry_transfer_query_acc,
            param_count=geometry_model.param_count,
            effective_state_size=geometry_model.effective_state_size,
            train_seconds=geometry_train_seconds,
            eval_seconds=geometry_eval_seconds,
        ),
        SequenceMetricsV13(
            model="tiny_transformer_baseline_v13",
            test_loss=transformer_test_loss,
            test_accuracy=transformer_test_acc,
            transfer_query_accuracy=transformer_transfer_query_acc,
            param_count=transformer.param_count,
            effective_state_size=transformer.effective_state_size,
            train_seconds=transformer_train_seconds,
            eval_seconds=transformer_eval_seconds,
        ),
    ]
