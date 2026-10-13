#!/usr/bin/env python3
"""Reduced-schema-alignment transfer test for the geometry-native engine."""

from __future__ import annotations

from dataclasses import dataclass
from time import perf_counter

import numpy as np
import torch

from geometry_native_sequence_model_v3 import (
    GeometryNativeSequenceModelV3,
    QUERY_IDS_V3,
    TOKENS_V3,
    TransformerSequenceTrainerV3,
    _build_feature_vector_v3,
    _initial_discourse_state,
    _initial_geometry_state,
    extract_geometry_features_v3,
    generate_dataset_v3,
)
from geometry_native_sequence_model_v4 import BASE_WEIGHTS_V4


torch.set_num_threads(1)


MENTION_IDS = {
    TOKENS_V3.index("MENTION0"): 0,
    TOKENS_V3.index("MENTION1"): 1,
    TOKENS_V3.index("MENTION2"): 2,
}
TAG_IDS = {
    TOKENS_V3.index("TAG0"): 0,
    TOKENS_V3.index("TAG1"): 1,
    TOKENS_V3.index("TAG2"): 2,
}
SET_SPEAKER_ID = TOKENS_V3.index("SET_SPEAKER")
SET_TOPIC_ID = TOKENS_V3.index("SET_TOPIC")
SHIFT_STYLE_ID = TOKENS_V3.index("SHIFT_STYLE")
ASK_ID = TOKENS_V3.index("ASK")

TOKEN_DB_V7 = np.array([1, 2, 3, 1, 2, 2, 3, 1, 2, 1, 2], dtype=np.int64)
TOKEN_DPHI_V7 = np.array([0, 1, 2, 2, 1, 2, 0, 1, 2, 0, 1], dtype=np.int64)
TOKEN_DR_V7 = np.array([0, 0, 0, 1, 1, 1, 0, 1, 0, 1, 1], dtype=np.int64)
TOKEN_DGAP_V7 = np.array([1, 2, 1, 2, 1, 3, 1, 2, 3, 1, 2], dtype=np.int64)


@dataclass(frozen=True)
class TaskConfigV7:
    train_seq_len: int = 30
    test_seq_len: int = 34
    train_size: int = 1024
    test_size: int = 256


@dataclass
class SequenceMetricsV7:
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


def _actual_binding_role(style: int, b: int, phi: int, speaker: int, topic: int) -> int:
    return (style + b + phi + (1 if speaker != topic else 0)) % 3


def _actual_query_role(
    style: int,
    phi: int,
    r: int,
    next_return_gap: int,
    speaker: int,
    topic: int,
    tags: list[int],
) -> int:
    parity = 1 if tags[speaker] == tags[topic] else 0
    return (2 * style + phi + r + next_return_gap + parity) % 3


def _proxy_role(style: int, phi: int, next_return_gap: int) -> int:
    return (style + phi + next_return_gap) % 3


def _proxy_entity(proxy_role: int, *, focus: int, speaker: int, topic: int) -> int:
    if proxy_role == 0:
        return focus
    if proxy_role == 1:
        return speaker
    return topic


def step_entangled_transfer_world(
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
) -> tuple[int, dict[str, int | list[int]], bool]:
    """Apply one step in a related family with reduced schema alignment."""

    next_focus = focus
    next_speaker = speaker
    next_topic = topic
    next_style = style
    next_tags = list(tags)
    is_query = token_id in QUERY_IDS_V3

    if token_id in MENTION_IDS:
        next_focus = MENTION_IDS[token_id]
    elif token_id == SET_SPEAKER_ID:
        next_speaker = next_focus
    elif token_id == SET_TOPIC_ID:
        next_topic = (next_focus + next_speaker + next_style) % 3
    elif token_id == SHIFT_STYLE_ID:
        next_style = (next_style + 2) % 3
        if (b + phi) % 2 == 1:
            next_speaker, next_topic = next_topic, next_speaker
    elif token_id in TAG_IDS:
        candidates = _candidate_entities(next_focus, next_speaker, next_topic)
        binding_role = _actual_binding_role(next_style, b, phi, next_speaker, next_topic)
        bound_entity = candidates[binding_role]
        next_tags[bound_entity] = TAG_IDS[token_id]
    elif token_id in QUERY_IDS_V3:
        pass
    else:
        raise ValueError(f"unexpected token id: {token_id}")

    candidates = _candidate_entities(next_focus, next_speaker, next_topic)
    actual_query_role = _actual_query_role(
        next_style,
        phi,
        r,
        next_return_gap,
        next_speaker,
        next_topic,
        next_tags,
    )
    actual_referent = candidates[actual_query_role]
    target = next_tags[actual_referent] if token_id == ASK_ID else actual_referent

    # Reduced-alignment projection back into the shared schema.
    proxy_role = _proxy_role(next_style, phi, next_return_gap)
    proxy_referent = _proxy_entity(proxy_role, focus=next_focus, speaker=next_speaker, topic=next_topic)

    snapshot = {
        "b": b,
        "phi": phi,
        "r": r,
        "focus": next_focus,
        "speaker": next_speaker,
        "topic": next_topic,
        "style": next_style,
        "next_return_gap": next_return_gap,
        "referent_role": proxy_role,
        "referent_entity": proxy_referent,
        "tags": next_tags,
    }

    next_b = (b + int(TOKEN_DB_V7[token_id])) % 5
    next_phi = (phi + int(TOKEN_DPHI_V7[token_id])) % 3
    next_r = (r + int(TOKEN_DR_V7[token_id])) % 4
    next_gap = 1 + ((next_return_gap - 1 + int(TOKEN_DGAP_V7[token_id])) % 4)
    snapshot["next_b"] = next_b
    snapshot["next_phi"] = next_phi
    snapshot["next_r"] = next_r
    snapshot["next_next_return_gap"] = next_gap
    return int(target), snapshot, is_query


def generate_dataset_transfer_v7(
    size: int,
    *,
    seq_len: int,
    seed: int,
) -> tuple[np.ndarray, np.ndarray, np.ndarray]:
    rng = np.random.default_rng(seed)
    inputs = np.zeros((size, seq_len), dtype=np.int64)
    targets = np.zeros((size, seq_len), dtype=np.int64)
    query_mask = np.zeros((size, seq_len), dtype=np.float32)

    for row in range(size):
        b, phi, r = _initial_geometry_state()
        focus, speaker, topic, style, gap, tags = _initial_discourse_state()
        for col in range(seq_len):
            token_id = int(rng.choice(len(TOKENS_V3), p=BASE_WEIGHTS_V4))
            target, snapshot, is_query = step_entangled_transfer_world(
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
            inputs[row, col] = token_id
            targets[row, col] = int(target)
            query_mask[row, col] = 1.0 if is_query else 0.0
            focus = int(snapshot["focus"])
            speaker = int(snapshot["speaker"])
            topic = int(snapshot["topic"])
            style = int(snapshot["style"])
            tags = list(snapshot["tags"])
            b = int(snapshot["next_b"])
            phi = int(snapshot["next_phi"])
            r = int(snapshot["next_r"])
            gap = int(snapshot["next_next_return_gap"])
    return inputs, targets, query_mask


def extract_geometry_features_transfer_v7(inputs: np.ndarray) -> np.ndarray:
    size, seq_len = inputs.shape
    features: list[np.ndarray] = []

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


def evaluate_geometry_v7(
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


def evaluate_transformer_v7(
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


def run_bounded_sequence_comparison_v7(config: TaskConfigV7, *, seed: int = 0) -> list[SequenceMetricsV7]:
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
    test_features = extract_geometry_features_transfer_v7(test_inputs)

    geometry_model = GeometryNativeSequenceModelV3(train_features.shape[-1], seed=seed)
    geometry_train_loss, geometry_train_seconds = geometry_model.fit(train_features, train_targets)
    geometry_test_loss, geometry_test_acc, geometry_transfer_query_acc, geometry_eval_seconds = evaluate_geometry_v7(
        geometry_model,
        test_features,
        test_targets,
        test_query_mask,
    )

    transformer = TransformerSequenceTrainerV3(seq_len=config.test_seq_len, seed=seed)
    transformer_train_loss, transformer_train_seconds = transformer.fit(train_inputs, train_targets)
    transformer_test_loss, transformer_test_acc, transformer_transfer_query_acc, transformer_eval_seconds = evaluate_transformer_v7(
        transformer,
        test_inputs,
        test_targets,
        test_query_mask,
    )

    return [
        SequenceMetricsV7(
            model="geometry_native_sequence_model_v7",
            train_loss=geometry_train_loss,
            test_loss=geometry_test_loss,
            test_accuracy=geometry_test_acc,
            transfer_query_accuracy=geometry_transfer_query_acc,
            param_count=geometry_model.param_count,
            effective_state_size=geometry_model.effective_state_size,
            train_seconds=geometry_train_seconds,
            eval_seconds=geometry_eval_seconds,
        ),
        SequenceMetricsV7(
            model="tiny_transformer_baseline_v7",
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
