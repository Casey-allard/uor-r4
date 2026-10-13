#!/usr/bin/env python3
"""GCD-style factor-overlap bridge revision on the stronger reduced-alignment setting."""

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
from geometry_native_sequence_model_v11 import _binding_bridge_delta as _binding_bridge_delta_v11
from geometry_native_sequence_model_v11 import _query_bridge_delta as _query_bridge_delta_v11
from geometry_native_sequence_model_v12 import (
    ROLE_PRIMES,
    TAG_IDS,
    TaskConfigV12,
    _candidate_entities,
    _initial_discourse_state,
    _initial_geometry_state,
    _prime,
    _proxy_role_v12,
    _binding_bridge_delta_v12,
    _query_bridge_delta_v12,
    evaluate_transformer_v12,
    generate_dataset_transfer_v12,
    step_entangled_transfer_world_v12,
)
from geometry_native_sequence_model_v13 import _selection_score


torch.set_num_threads(1)


@dataclass
class SequenceMetricsV16:
    model: str
    test_loss: float
    test_accuracy: float
    transfer_query_accuracy: float
    param_count: int
    effective_state_size: int
    train_seconds: float
    eval_seconds: float


def _variant_delta(token_id: int, snapshot: dict[str, int | list[int]], variant: str) -> int:
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
        return delta_v11
    if variant == "v12_base":
        return delta_v12
    if variant == "hybrid":
        return (delta_v11 + delta_v12) % 3
    raise ValueError(f"unknown bridge variant: {variant}")


def _snapshot_from_delta(
    snapshot: dict[str, int | list[int]],
    *,
    delta: int,
) -> dict[str, int | list[int]]:
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


def _variant_features(
    raw_snapshots: list[tuple[int, dict[str, int | list[int]]]],
    variant: str,
) -> tuple[np.ndarray, list[dict[str, int | list[int]]], list[int]]:
    adjusted_snapshots: list[dict[str, int | list[int]]] = []
    tokens: list[int] = []
    for token_id, snapshot in raw_snapshots:
        tokens.append(token_id)
        adjusted_snapshots.append(_snapshot_from_delta(snapshot, delta=_variant_delta(token_id, snapshot, variant)))
    features = np.stack(
        [
            _build_feature_vector_v3(token_id, adjusted_snapshot)
            for (token_id, _), adjusted_snapshot in zip(raw_snapshots, adjusted_snapshots)
        ],
        axis=0,
    )
    return features, adjusted_snapshots, tokens


def _revised_suffix_features(
    suffix_raw: list[tuple[int, dict[str, int | list[int]]]],
    *,
    current_variant: str,
    alt_variant: str,
) -> np.ndarray:
    revised_features: list[np.ndarray] = []
    for token_id, snapshot in suffix_raw:
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

        if overlap == current_bridge:
            revised_delta = current_delta
        else:
            # Retain the shared proxy factor and replace only the partner prime.
            revised_delta = alt_delta

        revised_snapshot = _snapshot_from_delta(snapshot, delta=revised_delta)
        revised_features.append(_build_feature_vector_v3(token_id, revised_snapshot))

    return np.stack(revised_features, axis=0)


def extract_geometry_features_v16(
    inputs: np.ndarray,
    model: GeometryNativeSequenceModelV3,
    *,
    support_window: int = 10,
    trigger_window: int = 8,
    disagreement_threshold: float = 0.35,
    score_margin: float = 0.03,
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
        trigger_start: int | None = None
        trigger_alt: str | None = None

        min_trigger_start = max(support_window, seq_len // 3)
        max_trigger_start = max(min_trigger_start, seq_len - trigger_window)
        for start in range(min_trigger_start, max_trigger_start + 1):
            end = min(start + trigger_window, seq_len)
            current_features, current_snapshots, current_tokens = variant_cache[current_variant]
            current_score = _selection_score(
                model,
                current_features[start:end],
                current_snapshots[start:end],
                current_tokens[start:end],
                support_window=min(trigger_window, end - start),
            )

            best_alt_variant = None
            best_alt_score = -1e9
            best_disagreement = 0.0
            for variant in variants:
                if variant == current_variant:
                    continue
                var_features, var_snapshots, var_tokens = variant_cache[variant]
                alt_score = _selection_score(
                    model,
                    var_features[start:end],
                    var_snapshots[start:end],
                    var_tokens[start:end],
                    support_window=min(trigger_window, end - start),
                )
                disagreements = 0
                total = 0
                for offset in range(start, end):
                    token_id, snapshot = raw_snapshots[offset]
                    total += 1
                    if _variant_delta(token_id, snapshot, current_variant) != _variant_delta(token_id, snapshot, variant):
                        disagreements += 1
                disagreement_rate = disagreements / total if total else 0.0
                if alt_score > best_alt_score:
                    best_alt_score = alt_score
                    best_alt_variant = variant
                    best_disagreement = disagreement_rate

            if (
                best_alt_variant is not None
                and best_disagreement >= disagreement_threshold
                and best_alt_score >= current_score + score_margin
            ):
                trigger_start = start
                trigger_alt = best_alt_variant
                break

        current_features, _, _ = variant_cache[current_variant]
        if trigger_start is None or trigger_alt is None:
            features.extend(current_features)
        else:
            prefix = current_features[:trigger_start]
            revised_suffix = _revised_suffix_features(
                raw_snapshots[trigger_start:],
                current_variant=current_variant,
                alt_variant=trigger_alt,
            )
            combined = np.concatenate([prefix, revised_suffix], axis=0)
            features.extend(combined)

    return np.stack(features, axis=0).reshape(size, seq_len, -1)


def evaluate_geometry_v16(
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


def run_bounded_sequence_comparison_v16(config: TaskConfigV12, *, seed: int = 0) -> list[SequenceMetricsV16]:
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

    test_features = extract_geometry_features_v16(test_inputs, geometry_model)
    geometry_test_loss, geometry_test_acc, geometry_transfer_query_acc, geometry_eval_seconds = evaluate_geometry_v16(
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
        SequenceMetricsV16(
            model="geometry_native_sequence_model_v16",
            test_loss=geometry_test_loss,
            test_accuracy=geometry_test_acc,
            transfer_query_accuracy=geometry_transfer_query_acc,
            param_count=geometry_model.param_count,
            effective_state_size=geometry_model.effective_state_size,
            train_seconds=geometry_train_seconds,
            eval_seconds=geometry_eval_seconds,
        ),
        SequenceMetricsV16(
            model="tiny_transformer_baseline_v16",
            test_loss=transformer_test_loss,
            test_accuracy=transformer_test_acc,
            transfer_query_accuracy=transformer_transfer_query_acc,
            param_count=transformer.param_count,
            effective_state_size=transformer.effective_state_size,
            train_seconds=transformer_train_seconds,
            eval_seconds=transformer_eval_seconds,
        ),
    ]
