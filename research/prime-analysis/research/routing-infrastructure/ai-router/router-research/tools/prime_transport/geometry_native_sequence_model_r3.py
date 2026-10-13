#!/usr/bin/env python3
"""Third inner-representation rebuild with native composite transport."""

from __future__ import annotations

from dataclasses import dataclass

import numpy as np

from geometry_native_sequence_model_r1 import GeometryNativeSequenceModelR1, extract_geometry_features_r1
from geometry_native_sequence_model_r2 import (
    GeometryNativeSequenceModelR2,
    OVERLAP_VALUES_R2,
    SEMIPRIMES_R2,
    SequenceMetricsR2,
    StructuredNativeReadoutR2,
)
from geometry_native_sequence_model_v3 import (
    GeometryNativeSequenceModelV3,
    QUERY_IDS_V3,
    TAG_IDS,
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


ROLE_PRIMES_R3 = (2, 3, 5)
SEMIPRIME_FACTORS_R3 = {
    4: (0, 0),
    6: (0, 1),
    9: (1, 1),
    10: (0, 2),
    15: (1, 2),
    25: (2, 2),
}
FACTORS_TO_SEMIPRIME_R3 = {tuple(value): key for key, value in SEMIPRIME_FACTORS_R3.items()}


@dataclass(frozen=True)
class TaskConfigR3:
    seq_len: int = 30
    train_size: int = 1024
    test_size: int = 256


def _semiprime_idx_r3(value: int) -> int:
    return SEMIPRIMES_R2.index(int(value))


def _overlap_idx_r3(left: int, right: int) -> int:
    return OVERLAP_VALUES_R2.index(int(np.gcd(int(left), int(right))))


def _decode_factors_r3(semiprime: int) -> tuple[int, int]:
    return SEMIPRIME_FACTORS_R3[int(semiprime)]


def _encode_factors_r3(left: int, right: int) -> int:
    ordered = tuple(sorted((int(left), int(right))))
    return FACTORS_TO_SEMIPRIME_R3[ordered]


def _native_transport_semiprimes_r3(
    *,
    token_id: int,
    b: int,
    phi: int,
    r: int,
    next_return_gap: int,
    style: int,
    focus: int,
    speaker: int,
    topic: int,
    tags: list[int],
    query_semiprime: int,
    binding_semiprime: int,
    spin_bits: tuple[int, ...],
) -> tuple[int, int, int]:
    q_left, q_right = _decode_factors_r3(query_semiprime)
    b_left, b_right = _decode_factors_r3(binding_semiprime)

    chart_turn = (b + phi + r + (next_return_gap - 1)) % 3
    discourse_turn = (focus + speaker + topic + style) % 3
    spin_turn = sum(int(bit) for bit in spin_bits) % 3
    tag_turn = sum(int(tag) for tag in tags) % 3

    query_anchor = q_left if (token_id + chart_turn) % 2 == 0 else q_right
    binding_anchor = b_left if (token_id + discourse_turn) % 2 == 0 else b_right

    query_partner = (
        query_anchor
        + int(TOKEN_DPHI_V3[token_id])
        + style
        + spin_turn
        + b_left
        + tag_turn
    ) % 3
    binding_partner = (
        binding_anchor
        + int(TOKEN_DR_V3[token_id])
        + chart_turn
        + q_left
        + int(token_id in TAG_IDS)
        + discourse_turn
    ) % 3

    if token_id in QUERY_IDS_V3:
        query_partner = (query_partner + b_right) % 3
        binding_partner = (binding_partner + q_right) % 3

    next_query = _encode_factors_r3(query_anchor, query_partner)
    next_binding = _encode_factors_r3(binding_anchor, binding_partner)
    overlap = int(np.gcd(next_query, next_binding))
    return next_query, next_binding, overlap


def _role_from_native_state_r3(
    *,
    style: int,
    phi: int,
    r: int,
    next_return_gap: int,
    query_semiprime: int,
    overlap: int,
) -> int:
    q_left, q_right = _decode_factors_r3(query_semiprime)
    overlap_turn = _decode_factors_r3(overlap)[0] if overlap in SEMIPRIME_FACTORS_R3 else 0
    return (style + phi + r + next_return_gap + q_left + q_right + overlap_turn) % 3


def step_discourse_world_r3(
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

    current_overlap = int(np.gcd(query_semiprime, binding_semiprime))
    current_binding_role = _role_from_native_state_r3(
        style=style,
        phi=phi,
        r=r,
        next_return_gap=next_return_gap,
        query_semiprime=query_semiprime,
        overlap=current_overlap,
    )
    current_binding_entity = _role_entity(
        current_binding_role,
        focus=next_focus,
        speaker=next_speaker,
        topic=next_topic,
    )

    if token_id == TOKEN_TO_ID_V3["MENTION0"]:
        next_focus = 0
    elif token_id == TOKEN_TO_ID_V3["MENTION1"]:
        next_focus = 1
    elif token_id == TOKEN_TO_ID_V3["MENTION2"]:
        next_focus = 2
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

    q_next, b_next, overlap = _native_transport_semiprimes_r3(
        token_id=token_id,
        b=proposed_b,
        phi=proposed_phi,
        r=proposed_r,
        next_return_gap=proposed_gap,
        style=next_style,
        focus=next_focus,
        speaker=next_speaker,
        topic=next_topic,
        tags=next_tags,
        query_semiprime=query_semiprime,
        binding_semiprime=binding_semiprime,
        spin_bits=spin_bits,
    )

    admissible = int(np.gcd(q_next, query_semiprime) > 1 and np.gcd(b_next, binding_semiprime) > 1)
    next_spin = tuple(list(spin_bits[1:]) + [admissible])

    referent_role = _role_from_native_state_r3(
        style=next_style,
        phi=proposed_phi,
        r=proposed_r,
        next_return_gap=proposed_gap,
        query_semiprime=q_next,
        overlap=overlap,
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


def extract_structured_state_r3(inputs: np.ndarray) -> tuple[dict[str, np.ndarray], np.ndarray]:
    size, seq_len = inputs.shape
    states = {
        "token_id": np.zeros((size, seq_len), dtype=np.int64),
        "b": np.zeros((size, seq_len), dtype=np.int64),
        "phi": np.zeros((size, seq_len), dtype=np.int64),
        "r": np.zeros((size, seq_len), dtype=np.int64),
        "focus": np.zeros((size, seq_len), dtype=np.int64),
        "speaker": np.zeros((size, seq_len), dtype=np.int64),
        "topic": np.zeros((size, seq_len), dtype=np.int64),
        "style": np.zeros((size, seq_len), dtype=np.int64),
        "gap": np.zeros((size, seq_len), dtype=np.int64),
        "query_semiprime": np.zeros((size, seq_len), dtype=np.int64),
        "binding_semiprime": np.zeros((size, seq_len), dtype=np.int64),
        "overlap": np.zeros((size, seq_len), dtype=np.int64),
        "admissible": np.zeros((size, seq_len), dtype=np.int64),
        "spin_bits": np.zeros((size, seq_len, 4), dtype=np.int64),
        "tags": np.zeros((size, seq_len, 3), dtype=np.int64),
    }
    query_mask = np.zeros((size, seq_len), dtype=np.float32)

    for row in range(size):
        b, phi, r = _initial_geometry_state()
        focus, speaker, topic, style, gap, tags = _initial_discourse_state()
        query_semiprime = 6
        binding_semiprime = 10
        spin_bits = (0, 0, 0, 1)

        for col in range(seq_len):
            token_id = int(inputs[row, col])
            _, snapshot, is_query = step_discourse_world_r3(
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

            states["token_id"][row, col] = token_id
            states["b"][row, col] = int(snapshot["b"])
            states["phi"][row, col] = int(snapshot["phi"])
            states["r"][row, col] = int(snapshot["r"])
            states["focus"][row, col] = int(snapshot["focus"])
            states["speaker"][row, col] = int(snapshot["speaker"])
            states["topic"][row, col] = int(snapshot["topic"])
            states["style"][row, col] = int(snapshot["style"])
            states["gap"][row, col] = int(snapshot["next_return_gap"]) - 1
            states["query_semiprime"][row, col] = _semiprime_idx_r3(int(snapshot["query_semiprime"]))
            states["binding_semiprime"][row, col] = _semiprime_idx_r3(int(snapshot["binding_semiprime"]))
            states["overlap"][row, col] = _overlap_idx_r3(
                int(snapshot["query_semiprime"]),
                int(snapshot["binding_semiprime"]),
            )
            states["admissible"][row, col] = int(snapshot["admissible_transition"])
            states["spin_bits"][row, col] = np.array(snapshot["spin_bits"], dtype=np.int64)
            states["tags"][row, col] = np.array(snapshot["tags"], dtype=np.int64)
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

    return states, query_mask


class GeometryNativeSequenceModelR3(GeometryNativeSequenceModelR2):
    def __init__(self, *, seed: int = 0) -> None:
        super().__init__(seed=seed)
        self.readout = StructuredNativeReadoutR2()


def run_bounded_sequence_comparison_r3(config: TaskConfigR3, *, seed: int = 0) -> list[SequenceMetricsR2]:
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

    from geometry_native_sequence_model_r2 import extract_structured_state_r2

    train_states_r2, _ = extract_structured_state_r2(train_inputs)
    test_states_r2, _ = extract_structured_state_r2(test_inputs)
    geometry_r2 = GeometryNativeSequenceModelR2(seed=seed)
    _train_loss_r2, train_seconds_r2 = geometry_r2.fit(train_states_r2, train_targets)
    test_loss_r2, test_acc_r2, query_acc_r2, eval_seconds_r2 = geometry_r2.evaluate(
        test_states_r2,
        test_targets,
        test_query_mask,
    )

    train_states_r3, _ = extract_structured_state_r3(train_inputs)
    test_states_r3, _ = extract_structured_state_r3(test_inputs)
    geometry_r3 = GeometryNativeSequenceModelR3(seed=seed)
    _train_loss_r3, train_seconds_r3 = geometry_r3.fit(train_states_r3, train_targets)
    test_loss_r3, test_acc_r3, query_acc_r3, eval_seconds_r3 = geometry_r3.evaluate(
        test_states_r3,
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
        SequenceMetricsR2(
            model="geometry_native_sequence_model_v3_reference",
            test_loss=test_loss_v3,
            test_accuracy=test_acc_v3,
            query_accuracy=query_acc_v3,
            param_count=geometry_v3.param_count,
            effective_state_size=geometry_v3.effective_state_size,
            train_seconds=train_seconds_v3,
            eval_seconds=eval_seconds_v3,
        ),
        SequenceMetricsR2(
            model="geometry_native_sequence_model_r1",
            test_loss=test_loss_r1,
            test_accuracy=test_acc_r1,
            query_accuracy=query_acc_r1,
            param_count=geometry_r1.param_count,
            effective_state_size=geometry_r1.effective_state_size,
            train_seconds=train_seconds_r1,
            eval_seconds=eval_seconds_r1,
        ),
        SequenceMetricsR2(
            model="geometry_native_sequence_model_r2",
            test_loss=test_loss_r2,
            test_accuracy=test_acc_r2,
            query_accuracy=query_acc_r2,
            param_count=geometry_r2.param_count,
            effective_state_size=geometry_r2.effective_state_size,
            train_seconds=train_seconds_r2,
            eval_seconds=eval_seconds_r2,
        ),
        SequenceMetricsR2(
            model="geometry_native_sequence_model_r3",
            test_loss=test_loss_r3,
            test_accuracy=test_acc_r3,
            query_accuracy=query_acc_r3,
            param_count=geometry_r3.param_count,
            effective_state_size=geometry_r3.effective_state_size,
            train_seconds=train_seconds_r3,
            eval_seconds=eval_seconds_r3,
        ),
        SequenceMetricsR2(
            model="tiny_transformer_baseline_r3",
            test_loss=transformer_test_loss,
            test_accuracy=transformer_test_acc,
            query_accuracy=transformer_query_acc,
            param_count=transformer.param_count,
            effective_state_size=transformer.effective_state_size,
            train_seconds=transformer_train_seconds,
            eval_seconds=transformer_eval_seconds,
        ),
    ]
