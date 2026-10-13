#!/usr/bin/env python3
"""Framework enforcement step 1: explicit native truncated spin_h plus gate scaffold."""

from __future__ import annotations

import csv
from dataclasses import dataclass
from pathlib import Path

import numpy as np

from geometry_native_sequence_model_r3 import _encode_factors_r3, _role_from_native_state_r3
from geometry_native_sequence_model_r6 import _joint_contexts_r6, _pair_index_to_components, _prepare_step_context_r6
from geometry_native_sequence_model_r7 import CompatibilityConditionedJointOperatorR7
from geometry_native_sequence_model_v3 import (
    TOKEN_TO_ID_V3,
    _initial_discourse_state,
    _initial_geometry_state,
    _role_entity,
    generate_dataset_v3,
)


OUTPUT_PATH_LOCK1 = Path(
    "/Users/adminamn/AI-Research/ai-router/router-research/results/"
    "prime_transport_recursive_system/prime_transport_framework_enforcement_step1.csv"
)

SPIN_H_HORIZON_LOCK1 = 4
SPIN_H_PAD_LOCK1 = -1


@dataclass(frozen=True)
class TaskConfigLock1:
    seq_len: int = 16
    train_size: int = 256
    audit_size: int = 16


@dataclass(frozen=True)
class NativeSpinHLock1:
    horizon: int
    valid_length: int
    bits: tuple[int, ...]


@dataclass(frozen=True)
class FrameworkClassTupleLock1:
    radial_class: int
    fiber_class: int
    spin_identity: NativeSpinHLock1
    composite_compat_class: tuple[str, str]


@dataclass(frozen=True)
class NativeStateLock1:
    b: int
    phi: int
    r: int
    focus: int
    speaker: int
    topic: int
    style: int
    next_return_gap: int
    tags: tuple[int, ...]
    query_semiprime: int
    binding_semiprime: int
    spin_bits: tuple[int, ...]
    admissible_transition: int


@dataclass(frozen=True)
class GateCheckLock1:
    inputs_computable: bool
    passes: bool
    radial_match: bool
    fiber_match: bool
    spin_match: bool
    compat_match: bool
    reason: str


def _compat_mask_lock1(semiprime: int, opposite_semiprime: int) -> str:
    return "".join(
        "1" if semiprime % prime == 0 and opposite_semiprime % prime == 0 else "0"
        for prime in (2, 3, 5)
    )


def _composite_compat_class_lock1(query_semiprime: int, binding_semiprime: int) -> tuple[str, str]:
    return (
        _compat_mask_lock1(query_semiprime, binding_semiprime),
        _compat_mask_lock1(binding_semiprime, query_semiprime),
    )


def _snapshot_to_state_lock1(snapshot: dict[str, int | list[int] | tuple[int, ...]]) -> NativeStateLock1:
    return NativeStateLock1(
        b=int(snapshot["b"]),
        phi=int(snapshot["phi"]),
        r=int(snapshot["r"]),
        focus=int(snapshot["focus"]),
        speaker=int(snapshot["speaker"]),
        topic=int(snapshot["topic"]),
        style=int(snapshot["style"]),
        next_return_gap=int(snapshot["next_return_gap"]),
        tags=tuple(int(tag) for tag in snapshot["tags"]),
        query_semiprime=int(snapshot["query_semiprime"]),
        binding_semiprime=int(snapshot["binding_semiprime"]),
        spin_bits=tuple(int(bit) for bit in snapshot["spin_bits"]),
        admissible_transition=int(snapshot["admissible_transition"]),
    )


def _state_dict_lock1(state: NativeStateLock1) -> dict[str, int | list[int] | tuple[int, ...]]:
    return {
        "b": state.b,
        "phi": state.phi,
        "r": state.r,
        "focus": state.focus,
        "speaker": state.speaker,
        "topic": state.topic,
        "style": state.style,
        "next_return_gap": state.next_return_gap,
        "tags": list(state.tags),
        "query_semiprime": state.query_semiprime,
        "binding_semiprime": state.binding_semiprime,
        "spin_bits": tuple(state.spin_bits),
        "admissible_transition": state.admissible_transition,
    }


def _step_with_forced_pair_lock1(
    token_id: int,
    *,
    state: NativeStateLock1,
    operator: CompatibilityConditionedJointOperatorR7,
    forced_query_pair_index: int | None = None,
    forced_binding_pair_index: int | None = None,
) -> tuple[NativeStateLock1, bool]:
    step_ctx = _prepare_step_context_r6(
        token_id,
        b=state.b,
        phi=state.phi,
        r=state.r,
        focus=state.focus,
        speaker=state.speaker,
        topic=state.topic,
        style=state.style,
        next_return_gap=state.next_return_gap,
        tags=list(state.tags),
        query_semiprime=state.query_semiprime,
        binding_semiprime=state.binding_semiprime,
        spin_bits=tuple(state.spin_bits),
    )
    q_ctx, b_ctx = _joint_contexts_r6(step_ctx)

    if forced_query_pair_index is None:
        q_pair, _q_stats = operator.predict_pair_with_stats(
            q_ctx,
            current_r=state.r,
            proposed_r=int(step_ctx["proposed_r"]),
            current_phi=state.phi,
            proposed_phi=int(step_ctx["proposed_phi"]),
            spin_bits=tuple(state.spin_bits),
            current_semiprime=state.query_semiprime,
            opposite_semiprime=state.binding_semiprime,
        )
    else:
        q_pair = _pair_index_to_components(forced_query_pair_index, q_ctx[2], q_ctx[3])

    if forced_binding_pair_index is None:
        b_pair, _b_stats = operator.predict_pair_with_stats(
            b_ctx,
            current_r=state.r,
            proposed_r=int(step_ctx["proposed_r"]),
            current_phi=state.phi,
            proposed_phi=int(step_ctx["proposed_phi"]),
            spin_bits=tuple(state.spin_bits),
            current_semiprime=state.binding_semiprime,
            opposite_semiprime=state.query_semiprime,
        )
    else:
        b_pair = _pair_index_to_components(forced_binding_pair_index, b_ctx[2], b_ctx[3])

    q_next = _encode_factors_r3(*q_pair)
    b_next = _encode_factors_r3(*b_pair)
    admissible = int(np.gcd(q_next, state.query_semiprime) > 1 and np.gcd(b_next, state.binding_semiprime) > 1)
    next_spin_bits = tuple(list(state.spin_bits[1:]) + [admissible])
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
    _target = next_tags[referent_entity] if token_id == TOKEN_TO_ID_V3["ASK"] else referent_entity

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
        "spin_bits": next_spin_bits,
        "admissible_transition": admissible,
    }
    return _snapshot_to_state_lock1(snapshot), bool(step_ctx["is_query"])


def _rollout_spin_h_lock1(
    state: NativeStateLock1,
    future_tokens: np.ndarray,
    *,
    operator: CompatibilityConditionedJointOperatorR7,
    horizon: int,
    forced_side: str | None = None,
    forced_pair_index: int | None = None,
) -> NativeSpinHLock1:
    bits: list[int] = []
    current = state
    for offset, token_id in enumerate(int(x) for x in future_tokens[:horizon]):
        next_state, _is_query = _step_with_forced_pair_lock1(
            token_id,
            state=current,
            operator=operator,
            forced_query_pair_index=forced_pair_index if (offset == 0 and forced_side == "query") else None,
            forced_binding_pair_index=forced_pair_index if (offset == 0 and forced_side == "binding") else None,
        )
        bits.append(int(next_state.admissible_transition))
        current = next_state
    padded = tuple(bits + [SPIN_H_PAD_LOCK1] * (horizon - len(bits)))
    return NativeSpinHLock1(horizon=horizon, valid_length=len(bits), bits=padded)


def _class_tuple_lock1(
    *,
    radial_class: int,
    fiber_class: int,
    spin_identity: NativeSpinHLock1,
    query_semiprime: int,
    binding_semiprime: int,
) -> FrameworkClassTupleLock1:
    return FrameworkClassTupleLock1(
        radial_class=radial_class,
        fiber_class=fiber_class,
        spin_identity=spin_identity,
        composite_compat_class=_composite_compat_class_lock1(query_semiprime, binding_semiprime),
    )


def _scaffold_gate_lock1(source: FrameworkClassTupleLock1, candidate: FrameworkClassTupleLock1) -> GateCheckLock1:
    inputs_computable = (
        source.spin_identity.horizon > 0
        and candidate.spin_identity.horizon > 0
        and len(source.spin_identity.bits) == source.spin_identity.horizon
        and len(candidate.spin_identity.bits) == candidate.spin_identity.horizon
    )
    if not inputs_computable:
        return GateCheckLock1(False, False, False, False, False, False, "incomplete_gate_inputs")

    radial_match = source.radial_class == candidate.radial_class
    fiber_match = source.fiber_class == candidate.fiber_class
    spin_match = source.spin_identity == candidate.spin_identity
    compat_match = source.composite_compat_class == candidate.composite_compat_class
    passes = radial_match and fiber_match and spin_match and compat_match
    if passes:
        reason = "pass"
    else:
        failed = []
        if not radial_match:
            failed.append("radial")
        if not fiber_match:
            failed.append("fiber")
        if not spin_match:
            failed.append("spin")
        if not compat_match:
            failed.append("compat")
        reason = "|".join(failed)
    return GateCheckLock1(inputs_computable, passes, radial_match, fiber_match, spin_match, compat_match, reason)


def _pair_index_from_pair_lock1(pair: tuple[int, int], left_factor: int, right_factor: int) -> int:
    for idx in range(6):
        if _pair_index_to_components(idx, left_factor, right_factor) == pair:
            return idx
    raise ValueError("pair not found")


def run_framework_enforcement_step1(config: TaskConfigLock1, *, seed: int = 241) -> list[dict[str, object]]:
    train_inputs, _train_targets, _train_query_mask = generate_dataset_v3(
        config.train_size,
        seq_len=config.seq_len,
        seed=seed,
    )
    audit_inputs, _audit_targets, _audit_query_mask = generate_dataset_v3(
        config.audit_size,
        seq_len=config.seq_len,
        seed=seed + 1,
    )

    operator = CompatibilityConditionedJointOperatorR7()
    operator.fit(train_inputs)

    source_state_count = 0
    source_spin_present_count = 0
    compared_pair_count = 0
    compared_spin_present_count = 0
    gate_inputs_computable_count = 0
    gate_pass_count = 0
    selected_transport_count = 0
    selected_transport_gate_pass_count = 0
    fail_radial = 0
    fail_fiber = 0
    fail_spin = 0
    fail_compat = 0
    fail_incomplete = 0

    for row in range(audit_inputs.shape[0]):
        b, phi, r = _initial_geometry_state()
        focus, speaker, topic, style, gap, tags = _initial_discourse_state()
        current_state = NativeStateLock1(
            b=b,
            phi=phi,
            r=r,
            focus=focus,
            speaker=speaker,
            topic=topic,
            style=style,
            next_return_gap=gap,
            tags=tuple(tags),
            query_semiprime=6,
            binding_semiprime=10,
            spin_bits=(0, 0, 0, 1),
            admissible_transition=1,
        )

        for col in range(audit_inputs.shape[1]):
            future_tokens = audit_inputs[row, col:]
            source_spin_h = _rollout_spin_h_lock1(
                current_state,
                future_tokens,
                operator=operator,
                horizon=SPIN_H_HORIZON_LOCK1,
            )
            source_tuple = _class_tuple_lock1(
                radial_class=current_state.r,
                fiber_class=current_state.phi,
                spin_identity=source_spin_h,
                query_semiprime=current_state.query_semiprime,
                binding_semiprime=current_state.binding_semiprime,
            )
            source_state_count += 1
            if source_spin_h.horizon == SPIN_H_HORIZON_LOCK1:
                source_spin_present_count += 1

            token_id = int(future_tokens[0])
            step_ctx = _prepare_step_context_r6(
                token_id,
                b=current_state.b,
                phi=current_state.phi,
                r=current_state.r,
                focus=current_state.focus,
                speaker=current_state.speaker,
                topic=current_state.topic,
                style=current_state.style,
                next_return_gap=current_state.next_return_gap,
                tags=list(current_state.tags),
                query_semiprime=current_state.query_semiprime,
                binding_semiprime=current_state.binding_semiprime,
                spin_bits=tuple(current_state.spin_bits),
            )
            q_ctx, b_ctx = _joint_contexts_r6(step_ctx)
            selected_q_pair, _q_stats = operator.predict_pair_with_stats(
                q_ctx,
                current_r=current_state.r,
                proposed_r=int(step_ctx["proposed_r"]),
                current_phi=current_state.phi,
                proposed_phi=int(step_ctx["proposed_phi"]),
                spin_bits=tuple(current_state.spin_bits),
                current_semiprime=current_state.query_semiprime,
                opposite_semiprime=current_state.binding_semiprime,
            )
            selected_b_pair, _b_stats = operator.predict_pair_with_stats(
                b_ctx,
                current_r=current_state.r,
                proposed_r=int(step_ctx["proposed_r"]),
                current_phi=current_state.phi,
                proposed_phi=int(step_ctx["proposed_phi"]),
                spin_bits=tuple(current_state.spin_bits),
                current_semiprime=current_state.binding_semiprime,
                opposite_semiprime=current_state.query_semiprime,
            )
            selected_q_index = _pair_index_from_pair_lock1(selected_q_pair, q_ctx[2], q_ctx[3])
            selected_b_index = _pair_index_from_pair_lock1(selected_b_pair, b_ctx[2], b_ctx[3])

            for side_name in ("query", "binding"):
                selected_index = selected_q_index if side_name == "query" else selected_b_index
                for pair_index in range(6):
                    candidate_state, _ = _step_with_forced_pair_lock1(
                        token_id,
                        state=current_state,
                        operator=operator,
                        forced_query_pair_index=pair_index if side_name == "query" else None,
                        forced_binding_pair_index=pair_index if side_name == "binding" else None,
                    )
                    candidate_spin_h = _rollout_spin_h_lock1(
                        candidate_state,
                        future_tokens[1:],
                        operator=operator,
                        horizon=SPIN_H_HORIZON_LOCK1,
                    )
                    candidate_tuple = _class_tuple_lock1(
                        radial_class=candidate_state.r,
                        fiber_class=candidate_state.phi,
                        spin_identity=candidate_spin_h,
                        query_semiprime=candidate_state.query_semiprime,
                        binding_semiprime=candidate_state.binding_semiprime,
                    )
                    gate = _scaffold_gate_lock1(source_tuple, candidate_tuple)
                    compared_pair_count += 1
                    if candidate_spin_h.horizon == SPIN_H_HORIZON_LOCK1:
                        compared_spin_present_count += 1
                    if gate.inputs_computable:
                        gate_inputs_computable_count += 1
                    else:
                        fail_incomplete += 1
                    if gate.passes:
                        gate_pass_count += 1
                    else:
                        fail_radial += int(not gate.radial_match)
                        fail_fiber += int(not gate.fiber_match)
                        fail_spin += int(not gate.spin_match)
                        fail_compat += int(not gate.compat_match)
                    if pair_index == selected_index:
                        selected_transport_count += 1
                        if gate.passes:
                            selected_transport_gate_pass_count += 1

            current_state, _ = _step_with_forced_pair_lock1(
                token_id,
                state=current_state,
                operator=operator,
            )

    rows = [
        {
            "scope": "state",
            "metric": "native_spin_h_present",
            "count": source_spin_present_count,
            "total": source_state_count,
            "fraction": source_spin_present_count / max(source_state_count, 1),
            "note": f"explicit_native_truncated_spin_h_h={SPIN_H_HORIZON_LOCK1}",
        },
        {
            "scope": "pair",
            "metric": "gate_inputs_computable",
            "count": gate_inputs_computable_count,
            "total": compared_pair_count,
            "fraction": gate_inputs_computable_count / max(compared_pair_count, 1),
            "note": "radial,fiber,spin_h,composite_compat all computable",
        },
        {
            "scope": "pair",
            "metric": "scaffold_gate_pass",
            "count": gate_pass_count,
            "total": compared_pair_count,
            "fraction": gate_pass_count / max(compared_pair_count, 1),
            "note": "direct comparison rule only; no lawful lifts implemented",
        },
        {
            "scope": "selected_transport",
            "metric": "scaffold_gate_pass",
            "count": selected_transport_gate_pass_count,
            "total": selected_transport_count,
            "fraction": selected_transport_gate_pass_count / max(selected_transport_count, 1),
            "note": "selected side-transports whose target class tuple matches source exactly",
        },
        {
            "scope": "pair_fail",
            "metric": "radial_class_mismatch",
            "count": fail_radial,
            "total": compared_pair_count,
            "fraction": fail_radial / max(compared_pair_count, 1),
            "note": "candidate radial_class != source radial_class",
        },
        {
            "scope": "pair_fail",
            "metric": "fiber_class_mismatch",
            "count": fail_fiber,
            "total": compared_pair_count,
            "fraction": fail_fiber / max(compared_pair_count, 1),
            "note": "candidate fiber_class != source fiber_class",
        },
        {
            "scope": "pair_fail",
            "metric": "spin_class_mismatch",
            "count": fail_spin,
            "total": compared_pair_count,
            "fraction": fail_spin / max(compared_pair_count, 1),
            "note": "candidate spin_h != source spin_h",
        },
        {
            "scope": "pair_fail",
            "metric": "composite_compat_class_mismatch",
            "count": fail_compat,
            "total": compared_pair_count,
            "fraction": fail_compat / max(compared_pair_count, 1),
            "note": "candidate composite compatibility class != source",
        },
        {
            "scope": "pair_fail",
            "metric": "incomplete_gate_input",
            "count": fail_incomplete,
            "total": compared_pair_count,
            "fraction": fail_incomplete / max(compared_pair_count, 1),
            "note": "gate could not compute full inputs",
        },
    ]
    return rows


def write_framework_enforcement_step1(rows: list[dict[str, object]], output_path: Path = OUTPUT_PATH_LOCK1) -> None:
    output_path.parent.mkdir(parents=True, exist_ok=True)
    with output_path.open("w", encoding="utf-8", newline="") as handle:
        writer = csv.DictWriter(handle, fieldnames=["scope", "metric", "count", "total", "fraction", "note"])
        writer.writeheader()
        writer.writerows(rows)


if __name__ == "__main__":
    summary_rows = run_framework_enforcement_step1(TaskConfigLock1(), seed=241)
    write_framework_enforcement_step1(summary_rows)
    print(f"wrote {OUTPUT_PATH_LOCK1}")
    for row in summary_rows:
        print(
            f"{row['scope']}:{row['metric']} count={row['count']} "
            f"total={row['total']} fraction={float(row['fraction']):.4f}"
        )
