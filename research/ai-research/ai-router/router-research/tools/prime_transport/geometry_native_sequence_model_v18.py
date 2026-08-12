#!/usr/bin/env python3
"""Micro-window conflict-centered factor repair on the stronger reduced-alignment setting."""

from __future__ import annotations

from dataclasses import dataclass
from math import gcd
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
from geometry_native_sequence_model_v12 import (
    TAG_IDS,
    TaskConfigV12,
    ROLE_PRIMES,
    _candidate_entities,
    _initial_discourse_state,
    _initial_geometry_state,
    _prime,
    _proxy_role_v12,
    evaluate_transformer_v12,
    generate_dataset_transfer_v12,
    step_entangled_transfer_world_v12,
)
from geometry_native_sequence_model_v13 import _selection_score
from geometry_native_sequence_model_v16 import _variant_delta, _variant_features


torch.set_num_threads(1)


@dataclass
class SequenceMetricsV18:
    model: str
    test_loss: float
    test_accuracy: float
    transfer_query_accuracy: float
    param_count: int
    effective_state_size: int
    train_seconds: float
    eval_seconds: float


def _snapshot_from_delta(snapshot: dict[str, int | list[int]], *, delta: int) -> dict[str, int | list[int]]:
    proxy_role = _proxy_role_v12(
        int(snapshot["style"]),
        int(snapshot["phi"]),
        int(snapshot["next_return_gap"]),
        int(snapshot["b"]),
    )
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


def _microrepair_features(
    raw_snapshots: list[tuple[int, dict[str, int | list[int]]]],
    *,
    current_variant: str,
    alt_variant: str,
    start: int,
    end: int,
) -> np.ndarray:
    repaired: list[np.ndarray] = []
    for idx, (token_id, snapshot) in enumerate(raw_snapshots):
        if start <= idx < end:
            current_delta = _variant_delta(token_id, snapshot, current_variant)
            alt_delta = _variant_delta(token_id, snapshot, alt_variant)
            proxy_role = _proxy_role_v12(
                int(snapshot["style"]),
                int(snapshot["phi"]),
                int(snapshot["next_return_gap"]),
                int(snapshot["b"]),
            )
            proxy_prime = _prime(proxy_role)
            current_bridge = proxy_prime * _prime(current_delta)
            alt_bridge = proxy_prime * _prime(alt_delta)
            overlap = gcd(current_bridge, alt_bridge)
            revised_delta = current_delta if overlap == current_bridge else alt_delta
            adjusted = _snapshot_from_delta(snapshot, delta=revised_delta)
        else:
            adjusted = _snapshot_from_delta(snapshot, delta=_variant_delta(token_id, snapshot, current_variant))
        repaired.append(_build_feature_vector_v3(token_id, adjusted))
    return np.stack(repaired, axis=0)


def extract_geometry_features_v18(
    inputs: np.ndarray,
    model: GeometryNativeSequenceModelV3,
    *,
    support_window: int = 10,
    conflict_window: int = 6,
    score_margin: float = 0.02,
    conflict_threshold: float = 0.30,
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

        variant_cache: dict[str, tuple[np.ndarray, list[dict[str, int | list[int]]], list[int]]] = {}
        initial_scores: list[float] = []
        for variant in variants:
            var_features, var_snapshots, var_tokens = _variant_features(raw_snapshots, variant)
            variant_cache[variant] = (var_features, var_snapshots, var_tokens)
            initial_scores.append(
                _selection_score(
                    model,
                    var_features,
                    var_snapshots,
                    var_tokens,
                    support_window=support_window,
                )
            )

        current_idx = int(np.argmax(np.array(initial_scores)))
        current_variant = variants[current_idx]
        repair_start: int | None = None
        repair_alt: str | None = None

        min_start = max(support_window, seq_len // 3)
        max_start = max(min_start, seq_len - conflict_window)
        for start in range(min_start, max_start + 1):
            end = min(start + conflict_window, seq_len)
            window_raw = raw_snapshots[start:end]
            query_count = sum(1 for token_id, _ in window_raw if token_id in QUERY_IDS_V3)
            bind_count = sum(1 for token_id, _ in window_raw if token_id in TAG_IDS)
            if query_count == 0 or bind_count == 0:
                continue

            current_features, current_snapshots, current_tokens = variant_cache[current_variant]
            current_score = _selection_score(
                model,
                current_features[start:end],
                current_snapshots[start:end],
                current_tokens[start:end],
                support_window=min(conflict_window, end - start),
            )

            best_alt_variant = None
            best_alt_score = -1e9
            best_conflict = 0.0
            for variant in variants:
                if variant == current_variant:
                    continue
                var_features, var_snapshots, var_tokens = variant_cache[variant]
                alt_score = _selection_score(
                    model,
                    var_features[start:end],
                    var_snapshots[start:end],
                    var_tokens[start:end],
                    support_window=min(conflict_window, end - start),
                )
                query_conflicts = 0
                bind_conflicts = 0
                total_queries = 0
                total_binds = 0
                for token_id, snapshot in window_raw:
                    if token_id in QUERY_IDS_V3:
                        total_queries += 1
                        if _variant_delta(token_id, snapshot, current_variant) != _variant_delta(token_id, snapshot, variant):
                            query_conflicts += 1
                    elif token_id in TAG_IDS:
                        total_binds += 1
                        if _variant_delta(token_id, snapshot, current_variant) != _variant_delta(token_id, snapshot, variant):
                            bind_conflicts += 1
                query_conflict_rate = query_conflicts / total_queries if total_queries else 0.0
                bind_conflict_rate = bind_conflicts / total_binds if total_binds else 0.0
                joint_conflict = min(query_conflict_rate, bind_conflict_rate)
                if alt_score > best_alt_score:
                    best_alt_score = alt_score
                    best_alt_variant = variant
                    best_conflict = joint_conflict

            if (
                best_alt_variant is not None
                and best_conflict >= conflict_threshold
                and best_alt_score >= current_score + score_margin
            ):
                repair_start = start
                repair_alt = best_alt_variant
                break

        if repair_start is None or repair_alt is None:
            current_features, _, _ = variant_cache[current_variant]
            features.extend(current_features)
        else:
            repaired = _microrepair_features(
                raw_snapshots,
                current_variant=current_variant,
                alt_variant=repair_alt,
                start=repair_start,
                end=min(repair_start + conflict_window, seq_len),
            )
            features.extend(repaired)

    return np.stack(features, axis=0).reshape(size, seq_len, -1)


def evaluate_geometry_v18(
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


def run_bounded_sequence_comparison_v18(config: TaskConfigV12, *, seed: int = 0) -> list[SequenceMetricsV18]:
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

    test_features = extract_geometry_features_v18(test_inputs, geometry_model)
    geometry_test_loss, geometry_test_acc, geometry_transfer_query_acc, geometry_eval_seconds = evaluate_geometry_v18(
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
        SequenceMetricsV18(
            model="geometry_native_sequence_model_v18",
            test_loss=geometry_test_loss,
            test_accuracy=geometry_test_acc,
            transfer_query_accuracy=geometry_transfer_query_acc,
            param_count=geometry_model.param_count,
            effective_state_size=geometry_model.effective_state_size,
            train_seconds=geometry_train_seconds,
            eval_seconds=geometry_eval_seconds,
        ),
        SequenceMetricsV18(
            model="tiny_transformer_baseline_v18",
            test_loss=transformer_test_loss,
            test_accuracy=transformer_test_acc,
            transfer_query_accuracy=transformer_transfer_query_acc,
            param_count=transformer.param_count,
            effective_state_size=transformer.effective_state_size,
            train_seconds=transformer_train_seconds,
            eval_seconds=transformer_eval_seconds,
        ),
    ]
