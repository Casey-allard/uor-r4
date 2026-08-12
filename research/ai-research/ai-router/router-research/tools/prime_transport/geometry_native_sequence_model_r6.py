#!/usr/bin/env python3
"""Sixth inner-representation rebuild with joint native transition operator."""

from __future__ import annotations

from dataclasses import dataclass

import numpy as np

from geometry_native_sequence_model_r1 import GeometryNativeSequenceModelR1, extract_geometry_features_r1
from geometry_native_sequence_model_r2 import GeometryNativeSequenceModelR2, SequenceMetricsR2, StructuredNativeReadoutR2
from geometry_native_sequence_model_r3 import (
    GeometryNativeSequenceModelR3,
    OVERLAP_VALUES_R2,
    SEMIPRIMES_R2,
    TAG_IDS,
    TOKEN_DB_V3,
    TOKEN_DGAP_V3,
    TOKEN_DPHI_V3,
    TOKEN_DR_V3,
    TOKEN_TO_ID_V3,
    _decode_factors_r3,
    _encode_factors_r3,
    _overlap_idx_r3,
    _role_from_native_state_r3,
    extract_structured_state_r3,
    step_discourse_world_r3,
)
from geometry_native_sequence_model_r4 import GeometryNativeSequenceModelR4, extract_structured_state_r4
from geometry_native_sequence_model_r5 import GeometryNativeSequenceModelR5, extract_structured_state_r5
from geometry_native_sequence_model_v3 import (
    GeometryNativeSequenceModelV3,
    QUERY_IDS_V3,
    TransformerSequenceTrainerV3,
    _initial_discourse_state,
    _initial_geometry_state,
    _role_entity,
    extract_geometry_features_v3,
    generate_dataset_v3,
)


MODE_QUERY_R6 = 0
MODE_BINDING_R6 = 1
PAIR_COUNT_R6 = 6


@dataclass(frozen=True)
class TaskConfigR6:
    seq_len: int = 30
    train_size: int = 1024
    test_size: int = 256


def _semiprime_idx_r6(value: int) -> int:
    return SEMIPRIMES_R2.index(int(value))


def _prepare_step_context_r6(
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
) -> dict[str, object]:
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

    return {
        "token_id": token_id,
        "is_query": is_query,
        "proposed_b": proposed_b,
        "proposed_phi": proposed_phi,
        "proposed_r": proposed_r,
        "proposed_gap": proposed_gap,
        "next_focus": next_focus,
        "next_speaker": next_speaker,
        "next_topic": next_topic,
        "next_style": next_style,
        "next_tags": next_tags,
        "query_semiprime": query_semiprime,
        "binding_semiprime": binding_semiprime,
        "spin_bits": spin_bits,
    }


def _joint_contexts_r6(step_ctx: dict[str, object]) -> tuple[tuple[int, ...], tuple[int, ...]]:
    token_id = int(step_ctx["token_id"])
    q_left, q_right = _decode_factors_r3(int(step_ctx["query_semiprime"]))
    b_left, b_right = _decode_factors_r3(int(step_ctx["binding_semiprime"]))

    proposed_b = int(step_ctx["proposed_b"])
    proposed_phi = int(step_ctx["proposed_phi"])
    proposed_r = int(step_ctx["proposed_r"])
    proposed_gap = int(step_ctx["proposed_gap"])
    next_focus = int(step_ctx["next_focus"])
    next_speaker = int(step_ctx["next_speaker"])
    next_topic = int(step_ctx["next_topic"])
    next_style = int(step_ctx["next_style"])
    next_tags = list(step_ctx["next_tags"])
    spin_bits = tuple(int(bit) for bit in step_ctx["spin_bits"])

    chart_turn = (proposed_b + proposed_phi + proposed_r + (proposed_gap - 1)) % 3
    discourse_turn = (next_focus + next_speaker + next_topic + next_style) % 3
    spin_turn = sum(spin_bits) % 3
    tag_turn = sum(int(tag) for tag in next_tags) % 3
    query_flag = int(token_id in QUERY_IDS_V3)
    tag_flag = int(token_id in TAG_IDS)

    query_context = (
        MODE_QUERY_R6,
        token_id,
        q_left,
        q_right,
        b_left,
        b_right,
        chart_turn,
        discourse_turn,
        spin_turn,
        tag_turn,
        query_flag,
        tag_flag,
    )
    binding_context = (
        MODE_BINDING_R6,
        token_id,
        b_left,
        b_right,
        q_left,
        q_right,
        chart_turn,
        discourse_turn,
        spin_turn,
        tag_turn,
        query_flag,
        tag_flag,
    )
    return query_context, binding_context


def _pair_index_to_components(pair_index: int, left_factor: int, right_factor: int) -> tuple[int, int]:
    anchor_index = int(pair_index) // 3
    partner_factor = int(pair_index) % 3
    anchor_factor = (left_factor, right_factor)[anchor_index]
    return anchor_factor, partner_factor


class LearnedJointTransportOperatorR6:
    """Joint logits over admissible (anchor choice, partner factor) pairs."""

    def __init__(self) -> None:
        self.base = np.zeros((2, PAIR_COUNT_R6), dtype=np.float64)
        self.token = np.zeros((2, 11, PAIR_COUNT_R6), dtype=np.float64)
        self.anchor_factor = np.zeros((2, 3, PAIR_COUNT_R6), dtype=np.float64)
        self.other_factor = np.zeros((2, 3, PAIR_COUNT_R6), dtype=np.float64)
        self.partner_factor = np.zeros((2, 3, PAIR_COUNT_R6), dtype=np.float64)
        self.opposite_left = np.zeros((2, 3, PAIR_COUNT_R6), dtype=np.float64)
        self.opposite_right = np.zeros((2, 3, PAIR_COUNT_R6), dtype=np.float64)
        self.chart_turn = np.zeros((2, 3, PAIR_COUNT_R6), dtype=np.float64)
        self.discourse_turn = np.zeros((2, 3, PAIR_COUNT_R6), dtype=np.float64)
        self.spin_turn = np.zeros((2, 3, PAIR_COUNT_R6), dtype=np.float64)
        self.tag_turn = np.zeros((2, 3, PAIR_COUNT_R6), dtype=np.float64)
        self.query_flag = np.zeros((2, 2, PAIR_COUNT_R6), dtype=np.float64)
        self.tag_flag = np.zeros((2, 2, PAIR_COUNT_R6), dtype=np.float64)

    @property
    def param_count(self) -> int:
        return (
            self.base.size
            + self.token.size
            + self.anchor_factor.size
            + self.other_factor.size
            + self.partner_factor.size
            + self.opposite_left.size
            + self.opposite_right.size
            + self.chart_turn.size
            + self.discourse_turn.size
            + self.spin_turn.size
            + self.tag_turn.size
            + self.query_flag.size
            + self.tag_flag.size
        )

    def fit(self, inputs: np.ndarray) -> None:
        smoothing = 1.0
        counts = {
            "base": np.full_like(self.base, smoothing),
            "token": np.full_like(self.token, smoothing),
            "anchor_factor": np.full_like(self.anchor_factor, smoothing),
            "other_factor": np.full_like(self.other_factor, smoothing),
            "partner_factor": np.full_like(self.partner_factor, smoothing),
            "opposite_left": np.full_like(self.opposite_left, smoothing),
            "opposite_right": np.full_like(self.opposite_right, smoothing),
            "chart_turn": np.full_like(self.chart_turn, smoothing),
            "discourse_turn": np.full_like(self.discourse_turn, smoothing),
            "spin_turn": np.full_like(self.spin_turn, smoothing),
            "tag_turn": np.full_like(self.tag_turn, smoothing),
            "query_flag": np.full_like(self.query_flag, smoothing),
            "tag_flag": np.full_like(self.tag_flag, smoothing),
        }

        for row in range(inputs.shape[0]):
            b, phi, r = _initial_geometry_state()
            focus, speaker, topic, style, gap, tags = _initial_discourse_state()
            query_semiprime = 6
            binding_semiprime = 10
            spin_bits = (0, 0, 0, 1)

            for col in range(inputs.shape[1]):
                token_id = int(inputs[row, col])
                step_ctx = _prepare_step_context_r6(
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
                q_ctx, b_ctx = _joint_contexts_r6(step_ctx)

                _, snapshot, _ = step_discourse_world_r3(
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

                q_next = int(snapshot["query_semiprime"])
                b_next = int(snapshot["binding_semiprime"])

                q_left, q_right = q_ctx[2], q_ctx[3]
                b_left, b_right = b_ctx[2], b_ctx[3]
                q_anchor_idx = 0 if q_left in _decode_factors_r3(q_next) else 1
                b_anchor_idx = 0 if b_left in _decode_factors_r3(b_next) else 1
                q_partner = _decode_factors_r3(q_next)[0 if _decode_factors_r3(q_next)[0] != (q_left, q_right)[q_anchor_idx] else 1]
                b_partner = _decode_factors_r3(b_next)[0 if _decode_factors_r3(b_next)[0] != (b_left, b_right)[b_anchor_idx] else 1]
                q_pair = q_anchor_idx * 3 + q_partner
                b_pair = b_anchor_idx * 3 + b_partner

                for context, pair_index in ((q_ctx, q_pair), (b_ctx, b_pair)):
                    mode, token, left_factor, right_factor, opposite_left, opposite_right, chart_turn, discourse_turn, spin_turn, tag_turn, query_flag, tag_flag = context
                    anchor_factor, partner_factor = _pair_index_to_components(pair_index, left_factor, right_factor)
                    other_factor = right_factor if anchor_factor == left_factor else left_factor
                    counts["base"][mode, pair_index] += 1.0
                    counts["token"][mode, token, pair_index] += 1.0
                    counts["anchor_factor"][mode, anchor_factor, pair_index] += 1.0
                    counts["other_factor"][mode, other_factor, pair_index] += 1.0
                    counts["partner_factor"][mode, partner_factor, pair_index] += 1.0
                    counts["opposite_left"][mode, opposite_left, pair_index] += 1.0
                    counts["opposite_right"][mode, opposite_right, pair_index] += 1.0
                    counts["chart_turn"][mode, chart_turn, pair_index] += 1.0
                    counts["discourse_turn"][mode, discourse_turn, pair_index] += 1.0
                    counts["spin_turn"][mode, spin_turn, pair_index] += 1.0
                    counts["tag_turn"][mode, tag_turn, pair_index] += 1.0
                    counts["query_flag"][mode, query_flag, pair_index] += 1.0
                    counts["tag_flag"][mode, tag_flag, pair_index] += 1.0

                b = int(snapshot["b"])
                phi = int(snapshot["phi"])
                r = int(snapshot["r"])
                focus = int(snapshot["focus"])
                speaker = int(snapshot["speaker"])
                topic = int(snapshot["topic"])
                style = int(snapshot["style"])
                gap = int(snapshot["next_return_gap"])
                tags = list(snapshot["tags"])
                query_semiprime = q_next
                binding_semiprime = b_next
                spin_bits = tuple(int(bit) for bit in snapshot["spin_bits"])

        self.base = np.log(counts["base"] / counts["base"].sum(axis=-1, keepdims=True))
        self.token = np.log(counts["token"] / counts["token"].sum(axis=-1, keepdims=True))
        self.anchor_factor = np.log(counts["anchor_factor"] / counts["anchor_factor"].sum(axis=-1, keepdims=True))
        self.other_factor = np.log(counts["other_factor"] / counts["other_factor"].sum(axis=-1, keepdims=True))
        self.partner_factor = np.log(counts["partner_factor"] / counts["partner_factor"].sum(axis=-1, keepdims=True))
        self.opposite_left = np.log(counts["opposite_left"] / counts["opposite_left"].sum(axis=-1, keepdims=True))
        self.opposite_right = np.log(counts["opposite_right"] / counts["opposite_right"].sum(axis=-1, keepdims=True))
        self.chart_turn = np.log(counts["chart_turn"] / counts["chart_turn"].sum(axis=-1, keepdims=True))
        self.discourse_turn = np.log(counts["discourse_turn"] / counts["discourse_turn"].sum(axis=-1, keepdims=True))
        self.spin_turn = np.log(counts["spin_turn"] / counts["spin_turn"].sum(axis=-1, keepdims=True))
        self.tag_turn = np.log(counts["tag_turn"] / counts["tag_turn"].sum(axis=-1, keepdims=True))
        self.query_flag = np.log(counts["query_flag"] / counts["query_flag"].sum(axis=-1, keepdims=True))
        self.tag_flag = np.log(counts["tag_flag"] / counts["tag_flag"].sum(axis=-1, keepdims=True))

    def predict_pair(self, context: tuple[int, ...]) -> tuple[int, int]:
        mode, token, left_factor, right_factor, opposite_left, opposite_right, chart_turn, discourse_turn, spin_turn, tag_turn, query_flag, tag_flag = context
        logits = (
            self.base[mode]
            + self.token[mode, token]
            + self.opposite_left[mode, opposite_left]
            + self.opposite_right[mode, opposite_right]
            + self.chart_turn[mode, chart_turn]
            + self.discourse_turn[mode, discourse_turn]
            + self.spin_turn[mode, spin_turn]
            + self.tag_turn[mode, tag_turn]
            + self.query_flag[mode, query_flag]
            + self.tag_flag[mode, tag_flag]
        )
        for pair_index in range(PAIR_COUNT_R6):
            anchor_factor, partner_factor = _pair_index_to_components(pair_index, left_factor, right_factor)
            other_factor = right_factor if anchor_factor == left_factor else left_factor
            logits[pair_index] += (
                self.anchor_factor[mode, anchor_factor, pair_index]
                + self.other_factor[mode, other_factor, pair_index]
                + self.partner_factor[mode, partner_factor, pair_index]
            )
        best_pair = int(np.argmax(logits))
        return _pair_index_to_components(best_pair, left_factor, right_factor)


def _transport_param_count_r6(transport_r4) -> int:
    return (
        transport_r4.base.size
        + transport_r4.token.size
        + transport_r4.anchor.size
        + transport_r4.coupled.size
        + transport_r4.chart_turn.size
        + transport_r4.discourse_turn.size
        + transport_r4.spin_turn.size
        + transport_r4.tag_turn.size
        + transport_r4.query_flag.size
        + transport_r4.tag_flag.size
    )


def step_discourse_world_r6(
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
    joint_operator: LearnedJointTransportOperatorR6,
) -> tuple[int, dict[str, int | list[int] | tuple[int, ...]], bool]:
    step_ctx = _prepare_step_context_r6(
        token_id,
        b=b,
        phi=phi,
        r=r,
        focus=focus,
        speaker=speaker,
        topic=topic,
        style=style,
        next_return_gap=next_return_gap,
        tags=tags,
        query_semiprime=query_semiprime,
        binding_semiprime=binding_semiprime,
        spin_bits=spin_bits,
    )
    q_ctx, b_ctx = _joint_contexts_r6(step_ctx)
    q_anchor, q_partner = joint_operator.predict_pair(q_ctx)
    b_anchor, b_partner = joint_operator.predict_pair(b_ctx)

    q_next = _encode_factors_r3(q_anchor, q_partner)
    b_next = _encode_factors_r3(b_anchor, b_partner)
    admissible = int(np.gcd(q_next, query_semiprime) > 1 and np.gcd(b_next, binding_semiprime) > 1)
    next_spin = tuple(list(tuple(int(bit) for bit in step_ctx["spin_bits"])[1:]) + [admissible])
    overlap = int(np.gcd(q_next, b_next))

    referent_role = _role_from_native_state_r3(
        style=int(step_ctx["next_style"]),
        phi=int(step_ctx["proposed_phi"]),
        r=int(step_ctx["proposed_r"]),
        next_return_gap=int(step_ctx["proposed_gap"]),
        query_semiprime=q_next,
        overlap=overlap,
    )
    referent_entity = _role_entity(
        referent_role,
        focus=int(step_ctx["next_focus"]),
        speaker=int(step_ctx["next_speaker"]),
        topic=int(step_ctx["next_topic"]),
    )
    next_tags = list(step_ctx["next_tags"])
    target = next_tags[referent_entity] if token_id == TOKEN_TO_ID_V3["ASK"] else referent_entity

    snapshot = {
        "b": int(step_ctx["proposed_b"]),
        "phi": int(step_ctx["proposed_phi"]),
        "r": int(step_ctx["proposed_r"]),
        "focus": int(step_ctx["next_focus"]),
        "speaker": int(step_ctx["next_speaker"]),
        "topic": int(step_ctx["next_topic"]),
        "style": int(step_ctx["next_style"]),
        "next_return_gap": int(step_ctx["proposed_gap"]),
        "referent_role": referent_role,
        "referent_entity": referent_entity,
        "tags": next_tags,
        "query_semiprime": q_next,
        "binding_semiprime": b_next,
        "spin_bits": next_spin,
        "admissible_transition": admissible,
    }
    return int(target), snapshot, bool(step_ctx["is_query"])


def extract_structured_state_r6(inputs: np.ndarray, joint_operator: LearnedJointTransportOperatorR6) -> tuple[dict[str, np.ndarray], np.ndarray]:
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
            _, snapshot, is_query = step_discourse_world_r6(
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
                joint_operator=joint_operator,
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
            states["query_semiprime"][row, col] = _semiprime_idx_r6(int(snapshot["query_semiprime"]))
            states["binding_semiprime"][row, col] = _semiprime_idx_r6(int(snapshot["binding_semiprime"]))
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


class GeometryNativeSequenceModelR6(GeometryNativeSequenceModelR2):
    def __init__(self, *, seed: int = 0) -> None:
        super().__init__(seed=seed)
        self.readout = StructuredNativeReadoutR2()


def run_bounded_sequence_comparison_r6(config: TaskConfigR6, *, seed: int = 0) -> list[SequenceMetricsR2]:
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

    from geometry_native_sequence_model_r4 import LearnedFactorTransportOperatorR4

    transport_r4 = LearnedFactorTransportOperatorR4()
    transport_r4.fit(train_inputs)
    transport_r4_param_count = _transport_param_count_r6(transport_r4)
    train_states_r4, _ = extract_structured_state_r4(train_inputs, transport_r4)
    test_states_r4, _ = extract_structured_state_r4(test_inputs, transport_r4)
    geometry_r4 = GeometryNativeSequenceModelR4(seed=seed)
    _train_loss_r4, train_seconds_r4 = geometry_r4.fit(train_states_r4, train_targets)
    test_loss_r4, test_acc_r4, query_acc_r4, eval_seconds_r4 = geometry_r4.evaluate(
        test_states_r4,
        test_targets,
        test_query_mask,
    )

    from geometry_native_sequence_model_r5 import LearnedAnchorSelectorR5

    anchor_r5 = LearnedAnchorSelectorR5()
    anchor_r5.fit(train_inputs)
    train_states_r5, _ = extract_structured_state_r5(train_inputs, anchor_r5, transport_r4)
    test_states_r5, _ = extract_structured_state_r5(test_inputs, anchor_r5, transport_r4)
    geometry_r5 = GeometryNativeSequenceModelR5(seed=seed)
    _train_loss_r5, train_seconds_r5 = geometry_r5.fit(train_states_r5, train_targets)
    test_loss_r5, test_acc_r5, query_acc_r5, eval_seconds_r5 = geometry_r5.evaluate(
        test_states_r5,
        test_targets,
        test_query_mask,
    )

    joint_r6 = LearnedJointTransportOperatorR6()
    joint_r6.fit(train_inputs)
    train_states_r6, _ = extract_structured_state_r6(train_inputs, joint_r6)
    test_states_r6, _ = extract_structured_state_r6(test_inputs, joint_r6)
    geometry_r6 = GeometryNativeSequenceModelR6(seed=seed)
    _train_loss_r6, train_seconds_r6 = geometry_r6.fit(train_states_r6, train_targets)
    test_loss_r6, test_acc_r6, query_acc_r6, eval_seconds_r6 = geometry_r6.evaluate(
        test_states_r6,
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
            model="geometry_native_sequence_model_r4",
            test_loss=test_loss_r4,
            test_accuracy=test_acc_r4,
            query_accuracy=query_acc_r4,
            param_count=geometry_r4.param_count + transport_r4_param_count,
            effective_state_size=geometry_r4.effective_state_size,
            train_seconds=train_seconds_r4,
            eval_seconds=eval_seconds_r4,
        ),
        SequenceMetricsR2(
            model="geometry_native_sequence_model_r5",
            test_loss=test_loss_r5,
            test_accuracy=test_acc_r5,
            query_accuracy=query_acc_r5,
            param_count=geometry_r5.param_count + transport_r4_param_count + anchor_r5.param_count,
            effective_state_size=geometry_r5.effective_state_size,
            train_seconds=train_seconds_r5,
            eval_seconds=eval_seconds_r5,
        ),
        SequenceMetricsR2(
            model="geometry_native_sequence_model_r6",
            test_loss=test_loss_r6,
            test_accuracy=test_acc_r6,
            query_accuracy=query_acc_r6,
            param_count=geometry_r6.param_count + joint_r6.param_count,
            effective_state_size=geometry_r6.effective_state_size,
            train_seconds=train_seconds_r6,
            eval_seconds=eval_seconds_r6,
        ),
        SequenceMetricsR2(
            model="tiny_transformer_baseline_r6",
            test_loss=transformer_test_loss,
            test_accuracy=transformer_test_acc,
            query_accuracy=transformer_query_acc,
            param_count=transformer.param_count,
            effective_state_size=transformer.effective_state_size,
            train_seconds=transformer_train_seconds,
            eval_seconds=transformer_eval_seconds,
        ),
    ]
