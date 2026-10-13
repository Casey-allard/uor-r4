#!/usr/bin/env python3
"""Divisibility-bridge realignment on the reduced-alignment transfer setting."""

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
from geometry_native_sequence_model_v7 import (
    ASK_ID,
    TAG_IDS,
    TaskConfigV7,
    _candidate_entities,
    _proxy_role,
    generate_dataset_transfer_v7,
    step_entangled_transfer_world,
)


torch.set_num_threads(1)

ROLE_PRIMES = (2, 3, 5)


@dataclass
class SequenceMetricsV11:
    model: str
    test_loss: float
    test_accuracy: float
    transfer_query_accuracy: float
    param_count: int
    effective_state_size: int
    train_seconds: float
    eval_seconds: float


def _prime(index: int) -> int:
    return ROLE_PRIMES[int(index) % 3]


def _query_bridge_delta(snapshot: dict[str, int | list[int]]) -> int:
    speaker = int(snapshot["speaker"])
    topic = int(snapshot["topic"])
    tags = list(snapshot["tags"])
    parity = 1 if tags[speaker] == tags[topic] else 0

    style_prime = _prime(int(snapshot["style"]))
    radial_parity_prime = _prime(int(snapshot["r"]) + parity)
    hub_semiprime = style_prime * radial_parity_prime

    # The semiprime hub mediates the missing query-side transition
    # between the lossy proxy role and the actual contextual role.
    return {
        4: 0,   # 2 * 2
        6: 1,   # 2 * 3
        10: 2,  # 2 * 5
        9: 2,   # 3 * 3
        15: 0,  # 3 * 5
        25: 1,  # 5 * 5
    }[hub_semiprime]


def _binding_bridge_delta(snapshot: dict[str, int | list[int]]) -> int:
    unequal = 1 if int(snapshot["speaker"]) != int(snapshot["topic"]) else 0
    gap_band = (int(snapshot["next_return_gap"]) - 1) % 3

    base_prime = _prime(int(snapshot["b"]))
    gap_ineq_prime = _prime(gap_band + unequal)
    hub_semiprime = base_prime * gap_ineq_prime

    # The binding bridge approximates how tagging moves through an entangled
    # local factorization before re-entering the cleaner shared chart.
    return {
        4: 2,   # 2 * 2
        6: 0,   # 2 * 3
        10: 1,  # 2 * 5
        9: 1,   # 3 * 3
        15: 2,  # 3 * 5
        25: 0,  # 5 * 5
    }[hub_semiprime]


def _realigned_snapshot(token_id: int, snapshot: dict[str, int | list[int]]) -> dict[str, int | list[int]]:
    proxy_role = _proxy_role(
        int(snapshot["style"]),
        int(snapshot["phi"]),
        int(snapshot["next_return_gap"]),
    )

    if token_id in QUERY_IDS_V3:
        delta = _query_bridge_delta(snapshot)
    elif token_id in TAG_IDS:
        delta = _binding_bridge_delta(snapshot)
    else:
        # Default local bridge for non-query, non-binding steps.
        delta = _query_bridge_delta(snapshot)

    bridge_prime = _prime(delta)
    proxy_prime = _prime(proxy_role)
    bridge_semiprime = proxy_prime * bridge_prime

    # Divisibility hub: recover the bridged role class by reading the
    # semiprime's transition prime against the proxy factor.
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


def extract_geometry_features_v11(inputs: np.ndarray) -> np.ndarray:
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
            features.append(_build_feature_vector_v3(token_id, _realigned_snapshot(token_id, snapshot)))

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


def evaluate_geometry_v11(
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


def evaluate_transformer_v11(
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


def run_bounded_sequence_comparison_v11(config: TaskConfigV7, *, seed: int = 0) -> list[SequenceMetricsV11]:
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
    test_features = extract_geometry_features_v11(test_inputs)

    geometry_model = GeometryNativeSequenceModelV3(train_features.shape[-1], seed=seed)
    _train_loss, geometry_train_seconds = geometry_model.fit(train_features, train_targets)
    geometry_test_loss, geometry_test_acc, geometry_transfer_query_acc, geometry_eval_seconds = evaluate_geometry_v11(
        geometry_model,
        test_features,
        test_targets,
        test_query_mask,
    )

    transformer = TransformerSequenceTrainerV3(seq_len=config.test_seq_len, seed=seed)
    _transformer_train_loss, transformer_train_seconds = transformer.fit(train_inputs, train_targets)
    transformer_test_loss, transformer_test_acc, transformer_transfer_query_acc, transformer_eval_seconds = evaluate_transformer_v11(
        transformer,
        test_inputs,
        test_targets,
        test_query_mask,
    )

    return [
        SequenceMetricsV11(
            model="geometry_native_sequence_model_v11",
            test_loss=geometry_test_loss,
            test_accuracy=geometry_test_acc,
            transfer_query_accuracy=geometry_transfer_query_acc,
            param_count=geometry_model.param_count,
            effective_state_size=geometry_model.effective_state_size,
            train_seconds=geometry_train_seconds,
            eval_seconds=geometry_eval_seconds,
        ),
        SequenceMetricsV11(
            model="tiny_transformer_baseline_v11",
            test_loss=transformer_test_loss,
            test_accuracy=transformer_test_acc,
            transfer_query_accuracy=transformer_transfer_query_acc,
            param_count=transformer.param_count,
            effective_state_size=transformer.effective_state_size,
            train_seconds=transformer_train_seconds,
            eval_seconds=transformer_eval_seconds,
        ),
    ]
