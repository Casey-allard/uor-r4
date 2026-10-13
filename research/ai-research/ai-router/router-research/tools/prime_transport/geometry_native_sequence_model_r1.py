#!/usr/bin/env python3
"""First inner-representation rebuild for the geometry-native sequence line."""

from __future__ import annotations

from dataclasses import dataclass
from math import cos, pi, sin
from time import perf_counter

import numpy as np
import torch
from torch import nn

from geometry_native_sequence_model_v3 import (
    GeometryNativeSequenceModelV3,
    QUERY_IDS_V3,
    TAG_IDS,
    TOKENS_V3,
    TOKEN_DB_V3,
    TOKEN_DGAP_V3,
    TOKEN_DPHI_V3,
    TOKEN_DR_V3,
    TOKEN_TO_ID_V3,
    TransformerSequenceTrainerV3,
    _initial_discourse_state,
    _initial_geometry_state,
    _role_entity,
    extract_geometry_features_v3,
    generate_dataset_v3,
)


torch.set_num_threads(1)


ROLE_PRIMES_R1 = (2, 3, 5)
SEMIPRIMES_R1 = (4, 6, 9, 10, 15, 25)
SPIN_H_R1 = 4
NUM_CLASSES_R1 = 3


@dataclass(frozen=True)
class TaskConfigR1:
    seq_len: int = 30
    train_size: int = 1024
    test_size: int = 256


@dataclass
class SequenceMetricsR1:
    model: str
    test_loss: float
    test_accuracy: float
    query_accuracy: float
    param_count: int
    effective_state_size: int
    train_seconds: float
    eval_seconds: float


class GeometryReadoutR1(nn.Module):
    def __init__(self, input_dim: int) -> None:
        super().__init__()
        self.net = nn.Sequential(
            nn.Linear(input_dim, 64),
            nn.GELU(),
            nn.Linear(64, 32),
            nn.GELU(),
            nn.Linear(32, NUM_CLASSES_R1),
        )

    def forward(self, x: torch.Tensor) -> torch.Tensor:
        return self.net(x)


class GeometryNativeSequenceModelR1:
    def __init__(self, input_dim: int, *, seed: int = 0) -> None:
        torch.manual_seed(seed)
        self.readout = GeometryReadoutR1(input_dim)
        self.loss_fn = nn.CrossEntropyLoss()

    @property
    def param_count(self) -> int:
        return sum(parameter.numel() for parameter in self.readout.parameters())

    @property
    def effective_state_size(self) -> int:
        return 16

    def fit(
        self,
        train_features: np.ndarray,
        train_targets: np.ndarray,
        *,
        epochs: int = 90,
        learning_rate: float = 1.2e-2,
    ) -> tuple[float, float]:
        x = torch.tensor(train_features.reshape(-1, train_features.shape[-1]), dtype=torch.float32)
        y = torch.tensor(train_targets.reshape(-1), dtype=torch.long)
        optimizer = torch.optim.Adam(self.readout.parameters(), lr=learning_rate)

        start = perf_counter()
        final_loss = 0.0
        for _ in range(epochs):
            optimizer.zero_grad()
            logits = self.readout(x)
            loss = self.loss_fn(logits, y)
            loss.backward()
            optimizer.step()
            final_loss = float(loss.item())
        elapsed = perf_counter() - start
        return final_loss, elapsed

    def evaluate(
        self,
        features: np.ndarray,
        targets: np.ndarray,
        query_mask: np.ndarray,
    ) -> tuple[float, float, float, float]:
        x = torch.tensor(features.reshape(-1, features.shape[-1]), dtype=torch.float32)
        y = torch.tensor(targets.reshape(-1), dtype=torch.long)
        q = torch.tensor(query_mask.reshape(-1), dtype=torch.float32)

        start = perf_counter()
        with torch.no_grad():
            logits = self.readout(x)
            loss = float(self.loss_fn(logits, y).item())
            predictions = logits.argmax(dim=-1)
            accuracy = float((predictions == y).float().mean().item())
            query_accuracy = float((((predictions == y).float() * q).sum() / q.sum()).item())
        elapsed = perf_counter() - start
        return loss, accuracy, query_accuracy, elapsed


def _prime_r1(index: int) -> int:
    return ROLE_PRIMES_R1[int(index) % 3]


def _semiprime_idx(value: int) -> int:
    return SEMIPRIMES_R1.index(int(value))


def _prime_membership(value: int) -> np.ndarray:
    return np.array([1.0 if value % prime == 0 else 0.0 for prime in ROLE_PRIMES_R1], dtype=np.float32)


def _decode_delta_from_semiprime(value: int) -> int:
    mapping = {
        4: 0,
        6: 1,
        10: 2,
        9: 2,
        15: 0,
        25: 1,
    }
    return mapping[int(value)]


def _compose_semiprime(left_idx: int, right_idx: int) -> int:
    return _prime_r1(left_idx) * _prime_r1(right_idx)


def _project_to_admissible_semiprime(
    proposed: int,
    previous: int,
    partner: int,
) -> int:
    best = previous
    best_score = -1
    for candidate in SEMIPRIMES_R1:
        score = 0
        if np.gcd(candidate, proposed) > 1:
            score += 3
        if np.gcd(candidate, previous) > 1:
            score += 2
        if np.gcd(candidate, partner) > 1:
            score += 1
        if score > best_score:
            best = candidate
            best_score = score
    return int(best)


def _cycle_coords(index: int, modulus: int) -> np.ndarray:
    angle = 2.0 * pi * (float(index) / float(modulus))
    return np.array([sin(angle), cos(angle)], dtype=np.float32)


def _one_hot(index: int, size: int) -> np.ndarray:
    vec = np.zeros(size, dtype=np.float32)
    vec[int(index)] = 1.0
    return vec


def _role_from_native_state(
    *,
    style: int,
    phi: int,
    r: int,
    next_return_gap: int,
    query_semiprime: int,
) -> int:
    proxy_role = (style + phi + r + next_return_gap) % 3
    return (proxy_role + _decode_delta_from_semiprime(query_semiprime)) % 3


def _build_feature_vector_r1(token_id: int, snapshot: dict[str, int | list[int]]) -> np.ndarray:
    chart = np.concatenate(
        [
            _cycle_coords(int(snapshot["b"]), 5),
            _cycle_coords(int(snapshot["phi"]), 3),
            _cycle_coords(int(snapshot["r"]), 4),
            _cycle_coords(int(snapshot["next_return_gap"]) - 1, 4),
        ],
        axis=0,
    )
    spin = np.array(list(snapshot["spin_bits"]), dtype=np.float32)
    query_membership = _prime_membership(int(snapshot["query_semiprime"]))
    binding_membership = _prime_membership(int(snapshot["binding_semiprime"]))

    gcd_value = int(np.gcd(int(snapshot["query_semiprime"]), int(snapshot["binding_semiprime"])))
    overlap_membership = _prime_membership(gcd_value)

    parts: list[np.ndarray] = [
        _one_hot(token_id, len(TOKENS_V3)),
        chart,
        spin,
        query_membership,
        binding_membership,
        overlap_membership,
        _one_hot(int(snapshot["focus"]), 3),
        _one_hot(int(snapshot["speaker"]), 3),
        _one_hot(int(snapshot["topic"]), 3),
        _one_hot(int(snapshot["style"]), 3),
        _one_hot(int(snapshot["referent_role"]), 3),
        _one_hot(int(snapshot["referent_entity"]), 3),
        _one_hot(_semiprime_idx(int(snapshot["query_semiprime"])), len(SEMIPRIMES_R1)),
        _one_hot(_semiprime_idx(int(snapshot["binding_semiprime"])), len(SEMIPRIMES_R1)),
        np.array([float(snapshot["admissible_transition"])], dtype=np.float32),
    ]

    tags = list(snapshot["tags"])
    for tag in tags:
        parts.append(_one_hot(int(tag), 3))

    chart_vs_comp = np.concatenate(
        [
            chart[:3] + query_membership,
            chart[:3] - binding_membership,
            query_membership * binding_membership,
        ],
        axis=0,
    )
    parts.append(chart_vs_comp.astype(np.float32))

    return np.concatenate(parts, axis=0)


def step_discourse_world_r1(
    token_id: int,
    *,
    b: int,
    phi: int,
    r: int,
    focus: int,
    speaker: int,
    topic: int,
    style: int,
    next_return_gap: int,
    tags: list[int],
    query_semiprime: int,
    binding_semiprime: int,
    spin_bits: tuple[int, ...],
) -> tuple[int, dict[str, int | list[int] | tuple[int, ...]], bool]:
    next_focus = focus
    next_speaker = speaker
    next_topic = topic
    next_style = style
    next_tags = list(tags)
    is_query = token_id in QUERY_IDS_V3

    current_binding_role = (style + b + _decode_delta_from_semiprime(binding_semiprime)) % 3
    current_binding_entity = _role_entity(
        current_binding_role,
        focus=next_focus,
        speaker=next_speaker,
        topic=next_topic,
    )

    if token_id in {
        TOKEN_TO_ID_V3["MENTION0"],
        TOKEN_TO_ID_V3["MENTION1"],
        TOKEN_TO_ID_V3["MENTION2"],
    }:
        next_focus = token_id
    elif token_id == TOKEN_TO_ID_V3["SET_SPEAKER"]:
        next_speaker = next_focus
    elif token_id == TOKEN_TO_ID_V3["SET_TOPIC"]:
        next_topic = next_focus
    elif token_id == TOKEN_TO_ID_V3["SHIFT_STYLE"]:
        next_style = (next_style + 1) % 3
    elif token_id in TAG_IDS:
        next_tags[current_binding_entity] = TAG_IDS[token_id]

    proposed_b = (b + int(TOKEN_DB_V3[token_id])) % 5
    proposed_phi = (phi + int(TOKEN_DPHI_V3[token_id])) % 3
    proposed_r = (r + int(TOKEN_DR_V3[token_id])) % 4
    proposed_gap = 1 + ((next_return_gap - 1 + int(TOKEN_DGAP_V3[token_id])) % 4)

    tag_parity = sum(next_tags) % 3
    q_proposed = _compose_semiprime(
        next_style + tag_parity,
        proposed_r + proposed_gap + next_focus,
    )
    b_proposed = _compose_semiprime(
        proposed_b + int(next_speaker != next_topic),
        (proposed_gap - 1) + int(token_id in TAG_IDS),
    )

    admissible = int(np.gcd(q_proposed, query_semiprime) > 1 and np.gcd(b_proposed, binding_semiprime) > 1)
    q_next = q_proposed
    b_next = b_proposed
    if not admissible:
        q_next = _project_to_admissible_semiprime(q_proposed, query_semiprime, binding_semiprime)
        b_next = _project_to_admissible_semiprime(b_proposed, binding_semiprime, query_semiprime)

    next_spin = tuple(list(spin_bits[1:]) + [admissible])
    referent_role = _role_from_native_state(
        style=next_style,
        phi=proposed_phi,
        r=proposed_r,
        next_return_gap=proposed_gap,
        query_semiprime=q_next,
    )
    referent_entity = _role_entity(
        referent_role,
        focus=next_focus,
        speaker=next_speaker,
        topic=next_topic,
    )
    target = next_tags[referent_entity] if token_id == TOKEN_TO_ID_V3["ASK"] else referent_entity

    snapshot = {
        "b": proposed_b,
        "phi": proposed_phi,
        "r": proposed_r,
        "focus": next_focus,
        "speaker": next_speaker,
        "topic": next_topic,
        "style": next_style,
        "next_return_gap": proposed_gap,
        "referent_role": referent_role,
        "referent_entity": referent_entity,
        "tags": next_tags,
        "query_semiprime": q_next,
        "binding_semiprime": b_next,
        "spin_bits": next_spin,
        "admissible_transition": admissible,
    }
    return int(target), snapshot, is_query


def extract_geometry_features_r1(inputs: np.ndarray) -> tuple[np.ndarray, np.ndarray]:
    size, seq_len = inputs.shape
    features: list[np.ndarray] = []
    query_mask = np.zeros((size, seq_len), dtype=np.float32)

    for row in range(size):
        b, phi, r = _initial_geometry_state()
        focus, speaker, topic, style, gap, tags = _initial_discourse_state()
        query_semiprime = 6
        binding_semiprime = 10
        spin_bits = (0, 0, 0, 1)
        for col in range(seq_len):
            token_id = int(inputs[row, col])
            _, snapshot, is_query = step_discourse_world_r1(
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
                query_semiprime=query_semiprime,
                binding_semiprime=binding_semiprime,
                spin_bits=spin_bits,
            )
            features.append(_build_feature_vector_r1(token_id, snapshot))
            query_mask[row, col] = 1.0 if is_query else 0.0

            b = int(snapshot["b"])
            phi = int(snapshot["phi"])
            r = int(snapshot["r"])
            focus = int(snapshot["focus"])
            speaker = int(snapshot["speaker"])
            topic = int(snapshot["topic"])
            style = int(snapshot["style"])
            gap = int(snapshot["next_return_gap"])
            tags = list(snapshot["tags"])
            query_semiprime = int(snapshot["query_semiprime"])
            binding_semiprime = int(snapshot["binding_semiprime"])
            spin_bits = tuple(int(bit) for bit in snapshot["spin_bits"])

    feature_array = np.stack(features, axis=0).reshape(size, seq_len, -1)
    return feature_array, query_mask


def run_bounded_sequence_comparison_r1(config: TaskConfigR1, *, seed: int = 0) -> list[SequenceMetricsR1]:
    train_inputs, train_targets, _ = generate_dataset_v3(
        config.train_size,
        seq_len=config.seq_len,
        seed=seed,
    )
    test_inputs, test_targets, test_query_mask = generate_dataset_v3(
        config.test_size,
        seq_len=config.seq_len,
        seed=seed + 1,
    )

    train_features_v3, _ = extract_geometry_features_v3(train_inputs)
    test_features_v3, _ = extract_geometry_features_v3(test_inputs)
    geometry_v3 = GeometryNativeSequenceModelV3(train_features_v3.shape[-1], seed=seed)
    _train_loss_v3, train_seconds_v3 = geometry_v3.fit(train_features_v3, train_targets)
    test_loss_v3, test_acc_v3, query_acc_v3, eval_seconds_v3 = geometry_v3.evaluate(
        test_features_v3,
        test_targets,
        test_query_mask,
    )

    train_features_r1, _ = extract_geometry_features_r1(train_inputs)
    test_features_r1, _ = extract_geometry_features_r1(test_inputs)
    geometry_r1 = GeometryNativeSequenceModelR1(train_features_r1.shape[-1], seed=seed)
    _train_loss_r1, train_seconds_r1 = geometry_r1.fit(train_features_r1, train_targets)
    test_loss_r1, test_acc_r1, query_acc_r1, eval_seconds_r1 = geometry_r1.evaluate(
        test_features_r1,
        test_targets,
        test_query_mask,
    )

    transformer = TransformerSequenceTrainerV3(seq_len=config.seq_len, seed=seed)
    _transformer_train_loss, transformer_train_seconds = transformer.fit(train_inputs, train_targets)
    transformer_test_loss, transformer_test_acc, transformer_query_acc, transformer_eval_seconds = transformer.evaluate(
        test_inputs,
        test_targets,
        test_query_mask,
    )

    return [
        SequenceMetricsR1(
            model="geometry_native_sequence_model_v3_reference",
            test_loss=test_loss_v3,
            test_accuracy=test_acc_v3,
            query_accuracy=query_acc_v3,
            param_count=geometry_v3.param_count,
            effective_state_size=geometry_v3.effective_state_size,
            train_seconds=train_seconds_v3,
            eval_seconds=eval_seconds_v3,
        ),
        SequenceMetricsR1(
            model="geometry_native_sequence_model_r1",
            test_loss=test_loss_r1,
            test_accuracy=test_acc_r1,
            query_accuracy=query_acc_r1,
            param_count=geometry_r1.param_count,
            effective_state_size=geometry_r1.effective_state_size,
            train_seconds=train_seconds_r1,
            eval_seconds=eval_seconds_r1,
        ),
        SequenceMetricsR1(
            model="tiny_transformer_baseline_r1",
            test_loss=transformer_test_loss,
            test_accuracy=transformer_test_acc,
            query_accuracy=transformer_query_acc,
            param_count=transformer.param_count,
            effective_state_size=transformer.effective_state_size,
            train_seconds=transformer_train_seconds,
            eval_seconds=transformer_eval_seconds,
        ),
    ]
