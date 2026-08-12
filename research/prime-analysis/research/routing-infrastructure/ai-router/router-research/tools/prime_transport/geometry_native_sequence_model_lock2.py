#!/usr/bin/env python3
"""Framework enforcement step 2: actual pre-scoring / pre-transport class gate."""

from __future__ import annotations

import csv
from collections import Counter
from dataclasses import dataclass
from pathlib import Path

import numpy as np

from geometry_native_sequence_model_lock1 import (
    OUTPUT_PATH_LOCK1,
    FrameworkClassTupleLock1,
    GateCheckLock1,
    NativeStateLock1,
    SPIN_H_HORIZON_LOCK1,
    TaskConfigLock1,
    _class_tuple_lock1,
    _rollout_spin_h_lock1,
    _scaffold_gate_lock1,
    _step_with_forced_pair_lock1,
)
from geometry_native_sequence_model_r6 import _joint_contexts_r6, _pair_index_to_components, _prepare_step_context_r6
from geometry_native_sequence_model_r7 import CompatibilityConditionedJointOperatorR7
from geometry_native_sequence_model_v3 import _initial_discourse_state, _initial_geometry_state, generate_dataset_v3


OUTPUT_PATH_LOCK2 = Path(
    "/Users/adminamn/AI-Research/ai-router/router-research/results/"
    "prime_transport_recursive_system/prime_transport_framework_enforcement_step2.csv"
)


@dataclass(frozen=True)
class LawfulChoiceLock2:
    pair_index: int
    gate: GateCheckLock1


@dataclass(frozen=True)
class SideSelectionResultLock2:
    has_lawful_candidate: bool
    selected_pair_index: int | None
    lawful_candidate_count: int
    failure_reason: str | None


def _pair_logit_lock2(operator: CompatibilityConditionedJointOperatorR7, context: tuple[int, ...], pair_index: int) -> float:
    mode, token, left_factor, right_factor, opposite_left, opposite_right, chart_turn, discourse_turn, spin_turn, tag_turn, query_flag, tag_flag = context
    anchor_factor, partner_factor = _pair_index_to_components(pair_index, left_factor, right_factor)
    other_factor = right_factor if anchor_factor == left_factor else left_factor
    return float(
        operator.base[mode, pair_index]
        + operator.token[mode, token, pair_index]
        + operator.opposite_left[mode, opposite_left, pair_index]
        + operator.opposite_right[mode, opposite_right, pair_index]
        + operator.chart_turn[mode, chart_turn, pair_index]
        + operator.discourse_turn[mode, discourse_turn, pair_index]
        + operator.spin_turn[mode, spin_turn, pair_index]
        + operator.tag_turn[mode, tag_turn, pair_index]
        + operator.query_flag[mode, query_flag, pair_index]
        + operator.tag_flag[mode, tag_flag, pair_index]
        + operator.anchor_factor[mode, anchor_factor, pair_index]
        + operator.other_factor[mode, other_factor, pair_index]
        + operator.partner_factor[mode, partner_factor, pair_index]
    )


def _candidate_class_tuple_lock2(
    *,
    token_id: int,
    current_state: NativeStateLock1,
    future_tokens: np.ndarray,
    operator: CompatibilityConditionedJointOperatorR7,
    side_name: str,
    pair_index: int,
) -> FrameworkClassTupleLock1:
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
    return _class_tuple_lock1(
        radial_class=candidate_state.r,
        fiber_class=candidate_state.phi,
        spin_identity=candidate_spin_h,
        query_semiprime=candidate_state.query_semiprime,
        binding_semiprime=candidate_state.binding_semiprime,
    )


def _select_with_enforced_gate_lock2(
    *,
    token_id: int,
    current_state: NativeStateLock1,
    future_tokens: np.ndarray,
    operator: CompatibilityConditionedJointOperatorR7,
    side_name: str,
    context: tuple[int, ...],
    source_tuple: FrameworkClassTupleLock1,
) -> tuple[SideSelectionResultLock2, list[GateCheckLock1]]:
    lawful: list[LawfulChoiceLock2] = []
    gate_checks: list[GateCheckLock1] = []
    for pair_index in range(6):
        candidate_tuple = _candidate_class_tuple_lock2(
            token_id=token_id,
            current_state=current_state,
            future_tokens=future_tokens,
            operator=operator,
            side_name=side_name,
            pair_index=pair_index,
        )
        gate = _scaffold_gate_lock1(source_tuple, candidate_tuple)
        gate_checks.append(gate)
        if gate.passes:
            lawful.append(LawfulChoiceLock2(pair_index=pair_index, gate=gate))

    if not lawful:
        return (
            SideSelectionResultLock2(
                has_lawful_candidate=False,
                selected_pair_index=None,
                lawful_candidate_count=0,
                failure_reason="no_lawful_move",
            ),
            gate_checks,
        )

    best_pair_index = max(lawful, key=lambda item: _pair_logit_lock2(operator, context, item.pair_index)).pair_index
    return (
        SideSelectionResultLock2(
            has_lawful_candidate=True,
            selected_pair_index=best_pair_index,
            lawful_candidate_count=len(lawful),
            failure_reason=None,
        ),
        gate_checks,
    )


def run_framework_enforcement_step2(config: TaskConfigLock1, *, seed: int = 241) -> list[dict[str, object]]:
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

    total_candidate_count_before_gate = 0
    total_candidate_count_after_gate = 0
    total_side_steps = 0
    side_steps_with_lawful = 0
    side_steps_zero_lawful = 0
    explicit_failure_side_transports = 0
    failure_counter: Counter[str] = Counter()
    lawful_candidate_distribution: Counter[int] = Counter()
    terminated_sequence_count = 0

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

        sequence_failed = False
        for col in range(audit_inputs.shape[1]):
            if sequence_failed:
                break

            future_tokens = audit_inputs[row, col:]
            token_id = int(future_tokens[0])
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

            per_side_results: dict[str, SideSelectionResultLock2] = {}
            for side_name, context in (("query", q_ctx), ("binding", b_ctx)):
                result, gate_checks = _select_with_enforced_gate_lock2(
                    token_id=token_id,
                    current_state=current_state,
                    future_tokens=future_tokens,
                    operator=operator,
                    side_name=side_name,
                    context=context,
                    source_tuple=source_tuple,
                )
                total_side_steps += 1
                total_candidate_count_before_gate += 6
                total_candidate_count_after_gate += result.lawful_candidate_count
                lawful_candidate_distribution[result.lawful_candidate_count] += 1

                if result.has_lawful_candidate:
                    side_steps_with_lawful += 1
                else:
                    side_steps_zero_lawful += 1
                    explicit_failure_side_transports += 1

                for gate in gate_checks:
                    if gate.passes:
                        continue
                    if not gate.radial_match:
                        failure_counter["radial_mismatch"] += 1
                    if not gate.fiber_match:
                        failure_counter["fiber_mismatch"] += 1
                    if not gate.spin_match:
                        failure_counter["spin_mismatch"] += 1
                    if not gate.compat_match:
                        failure_counter["composite_compat_mismatch"] += 1

                per_side_results[side_name] = result

            if not per_side_results["query"].has_lawful_candidate or not per_side_results["binding"].has_lawful_candidate:
                sequence_failed = True
                terminated_sequence_count += 1
                continue

            current_state, _ = _step_with_forced_pair_lock1(
                token_id,
                state=current_state,
                operator=operator,
                forced_query_pair_index=per_side_results["query"].selected_pair_index,
                forced_binding_pair_index=per_side_results["binding"].selected_pair_index,
            )

    rows = [
        {
            "scope": "candidate_supply",
            "metric": "total_candidate_count_before_gate",
            "count": total_candidate_count_before_gate,
            "total": total_candidate_count_before_gate,
            "fraction": 1.0,
            "note": "all side-step candidates before class enforcement",
        },
        {
            "scope": "candidate_supply",
            "metric": "total_candidate_count_after_gate",
            "count": total_candidate_count_after_gate,
            "total": total_candidate_count_before_gate,
            "fraction": total_candidate_count_after_gate / max(total_candidate_count_before_gate, 1),
            "note": "lawful candidates remaining after enforced direct-comparison gate",
        },
        {
            "scope": "side_step",
            "metric": "has_at_least_one_lawful_candidate",
            "count": side_steps_with_lawful,
            "total": total_side_steps,
            "fraction": side_steps_with_lawful / max(total_side_steps, 1),
            "note": "side-steps with lawful supply > 0",
        },
        {
            "scope": "side_step",
            "metric": "zero_lawful_candidates",
            "count": side_steps_zero_lawful,
            "total": total_side_steps,
            "fraction": side_steps_zero_lawful / max(total_side_steps, 1),
            "note": "side-steps that become explicit no_lawful_move",
        },
        {
            "scope": "side_step",
            "metric": "explicit_failure_no_lawful_move",
            "count": explicit_failure_side_transports,
            "total": total_side_steps,
            "fraction": explicit_failure_side_transports / max(total_side_steps, 1),
            "note": "failed side-transports with no fallback",
        },
        {
            "scope": "candidate_supply",
            "metric": "average_lawful_candidates_per_side_step",
            "count": total_candidate_count_after_gate,
            "total": total_side_steps,
            "fraction": total_candidate_count_after_gate / max(total_side_steps, 1),
            "note": "mean lawful candidate count per side-step",
        },
        {
            "scope": "sequence",
            "metric": "terminated_sequences_due_to_no_lawful_move",
            "count": terminated_sequence_count,
            "total": audit_inputs.shape[0],
            "fraction": terminated_sequence_count / max(audit_inputs.shape[0], 1),
            "note": "sequences halted because one or both sides had no lawful move",
        },
        {
            "scope": "candidate_fail",
            "metric": "radial_mismatch",
            "count": failure_counter["radial_mismatch"],
            "total": total_candidate_count_before_gate,
            "fraction": failure_counter["radial_mismatch"] / max(total_candidate_count_before_gate, 1),
            "note": "candidates rejected for radial class mismatch",
        },
        {
            "scope": "candidate_fail",
            "metric": "fiber_mismatch",
            "count": failure_counter["fiber_mismatch"],
            "total": total_candidate_count_before_gate,
            "fraction": failure_counter["fiber_mismatch"] / max(total_candidate_count_before_gate, 1),
            "note": "candidates rejected for fiber class mismatch",
        },
        {
            "scope": "candidate_fail",
            "metric": "spin_mismatch",
            "count": failure_counter["spin_mismatch"],
            "total": total_candidate_count_before_gate,
            "fraction": failure_counter["spin_mismatch"] / max(total_candidate_count_before_gate, 1),
            "note": "candidates rejected for spin_h mismatch",
        },
        {
            "scope": "candidate_fail",
            "metric": "composite_compat_mismatch",
            "count": failure_counter["composite_compat_mismatch"],
            "total": total_candidate_count_before_gate,
            "fraction": failure_counter["composite_compat_mismatch"] / max(total_candidate_count_before_gate, 1),
            "note": "candidates rejected for composite compatibility mismatch",
        },
    ]

    for lawful_count in sorted(lawful_candidate_distribution):
        rows.append(
            {
                "scope": "lawful_candidate_distribution",
                "metric": f"lawful_count_{lawful_count}",
                "count": lawful_candidate_distribution[lawful_count],
                "total": total_side_steps,
                "fraction": lawful_candidate_distribution[lawful_count] / max(total_side_steps, 1),
                "note": "distribution over lawful candidate counts per side-step",
            }
        )

    return rows


def write_framework_enforcement_step2(rows: list[dict[str, object]], output_path: Path = OUTPUT_PATH_LOCK2) -> None:
    output_path.parent.mkdir(parents=True, exist_ok=True)
    with output_path.open("w", encoding="utf-8", newline="") as handle:
        writer = csv.DictWriter(handle, fieldnames=["scope", "metric", "count", "total", "fraction", "note"])
        writer.writeheader()
        writer.writerows(rows)


if __name__ == "__main__":
    summary_rows = run_framework_enforcement_step2(TaskConfigLock1(), seed=241)
    write_framework_enforcement_step2(summary_rows)
    print(f"wrote {OUTPUT_PATH_LOCK2}")
    for row in summary_rows:
        print(
            f"{row['scope']}:{row['metric']} count={row['count']} "
            f"total={row['total']} fraction={float(row['fraction']):.4f}"
        )
