#!/usr/bin/env python3
"""Framework enforcement step 3: add one explicit lawful within-class move primitive."""

from __future__ import annotations

import csv
from collections import Counter
from dataclasses import dataclass
from pathlib import Path

from geometry_native_sequence_model_lock1 import (
    FrameworkClassTupleLock1,
    NativeStateLock1,
    SPIN_H_HORIZON_LOCK1,
    TaskConfigLock1,
    _class_tuple_lock1,
    _rollout_spin_h_lock1,
    _scaffold_gate_lock1,
    _step_with_forced_pair_lock1,
)
from geometry_native_sequence_model_lock2 import _pair_logit_lock2
from geometry_native_sequence_model_r6 import _joint_contexts_r6, _prepare_step_context_r6
from geometry_native_sequence_model_r7 import CompatibilityConditionedJointOperatorR7
from geometry_native_sequence_model_v3 import _initial_discourse_state, _initial_geometry_state, generate_dataset_v3


OUTPUT_PATH_LOCK3 = Path(
    "/Users/adminamn/AI-Research/ai-router/router-research/results/"
    "prime_transport_recursive_system/prime_transport_framework_enforcement_step3.csv"
)

HOLD_PAIR_INDEX_LOCK3 = 6


@dataclass(frozen=True)
class SideSelectionResultLock3:
    has_lawful_candidate: bool
    selected_pair_index: int | None
    lawful_candidate_count: int
    failure_reason: str | None


def _candidate_class_tuple_lock3(
    *,
    token_id: int,
    current_state: NativeStateLock1,
    future_tokens,
    operator: CompatibilityConditionedJointOperatorR7,
    side_name: str,
    pair_index: int,
) -> FrameworkClassTupleLock1:
    if pair_index == HOLD_PAIR_INDEX_LOCK3:
        candidate_state = current_state
        candidate_spin_h = _rollout_spin_h_lock1(
            candidate_state,
            future_tokens,
            operator=operator,
            horizon=SPIN_H_HORIZON_LOCK1,
        )
    else:
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


def _select_with_enforced_gate_lock3(
    *,
    token_id: int,
    current_state: NativeStateLock1,
    future_tokens,
    operator: CompatibilityConditionedJointOperatorR7,
    side_name: str,
    context: tuple[int, ...],
    source_tuple: FrameworkClassTupleLock1,
) -> tuple[SideSelectionResultLock3, list[tuple[int, object]]]:
    lawful_indices: list[int] = []
    gate_checks: list[tuple[int, object]] = []
    for pair_index in range(7):
        candidate_tuple = _candidate_class_tuple_lock3(
            token_id=token_id,
            current_state=current_state,
            future_tokens=future_tokens,
            operator=operator,
            side_name=side_name,
            pair_index=pair_index,
        )
        gate = _scaffold_gate_lock1(source_tuple, candidate_tuple)
        gate_checks.append((pair_index, gate))
        if gate.passes:
            lawful_indices.append(pair_index)

    if not lawful_indices:
        return (
            SideSelectionResultLock3(
                has_lawful_candidate=False,
                selected_pair_index=None,
                lawful_candidate_count=0,
                failure_reason="no_lawful_move",
            ),
            gate_checks,
        )

    ordinary_lawful = [idx for idx in lawful_indices if idx != HOLD_PAIR_INDEX_LOCK3]
    if ordinary_lawful:
        selected_pair_index = max(ordinary_lawful, key=lambda idx: _pair_logit_lock2(operator, context, idx))
    else:
        selected_pair_index = HOLD_PAIR_INDEX_LOCK3
    return (
        SideSelectionResultLock3(
            has_lawful_candidate=True,
            selected_pair_index=selected_pair_index,
            lawful_candidate_count=len(lawful_indices),
            failure_reason=None,
        ),
        gate_checks,
    )


def run_framework_enforcement_step3(config: TaskConfigLock1, *, seed: int = 241) -> list[dict[str, object]]:
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
    selected_hold_count = 0
    selected_ordinary_count = 0

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

            per_side_results: dict[str, SideSelectionResultLock3] = {}
            for side_name, context in (("query", q_ctx), ("binding", b_ctx)):
                result, gate_checks = _select_with_enforced_gate_lock3(
                    token_id=token_id,
                    current_state=current_state,
                    future_tokens=future_tokens,
                    operator=operator,
                    side_name=side_name,
                    context=context,
                    source_tuple=source_tuple,
                )
                total_side_steps += 1
                total_candidate_count_before_gate += 7
                total_candidate_count_after_gate += result.lawful_candidate_count
                lawful_candidate_distribution[result.lawful_candidate_count] += 1

                if result.has_lawful_candidate:
                    side_steps_with_lawful += 1
                    if result.selected_pair_index == HOLD_PAIR_INDEX_LOCK3:
                        selected_hold_count += 1
                    else:
                        selected_ordinary_count += 1
                else:
                    side_steps_zero_lawful += 1
                    explicit_failure_side_transports += 1

                for pair_index, gate in gate_checks:
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

            if (
                per_side_results["query"].selected_pair_index == HOLD_PAIR_INDEX_LOCK3
                and per_side_results["binding"].selected_pair_index == HOLD_PAIR_INDEX_LOCK3
            ):
                current_state = current_state
            else:
                current_state, _ = _step_with_forced_pair_lock1(
                    token_id,
                    state=current_state,
                    operator=operator,
                    forced_query_pair_index=None if per_side_results["query"].selected_pair_index == HOLD_PAIR_INDEX_LOCK3 else per_side_results["query"].selected_pair_index,
                    forced_binding_pair_index=None if per_side_results["binding"].selected_pair_index == HOLD_PAIR_INDEX_LOCK3 else per_side_results["binding"].selected_pair_index,
                )

    rows = [
        {
            "scope": "candidate_supply",
            "metric": "total_candidate_count_before_gate",
            "count": total_candidate_count_before_gate,
            "total": total_candidate_count_before_gate,
            "fraction": 1.0,
            "note": "all side-step candidates including explicit within-class hold primitive",
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
            "scope": "selection",
            "metric": "selected_within_class_hold_primitive",
            "count": selected_hold_count,
            "total": total_side_steps,
            "fraction": selected_hold_count / max(total_side_steps, 1),
            "note": "side-steps selecting the new lawful within-class hold primitive",
        },
        {
            "scope": "selection",
            "metric": "selected_ordinary_lawful_candidate",
            "count": selected_ordinary_count,
            "total": total_side_steps,
            "fraction": selected_ordinary_count / max(total_side_steps, 1),
            "note": "side-steps selecting an ordinary lawful candidate",
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


def write_framework_enforcement_step3(rows: list[dict[str, object]], output_path: Path = OUTPUT_PATH_LOCK3) -> None:
    output_path.parent.mkdir(parents=True, exist_ok=True)
    with output_path.open("w", encoding="utf-8", newline="") as handle:
        writer = csv.DictWriter(handle, fieldnames=["scope", "metric", "count", "total", "fraction", "note"])
        writer.writeheader()
        writer.writerows(rows)


if __name__ == "__main__":
    summary_rows = run_framework_enforcement_step3(TaskConfigLock1(), seed=241)
    write_framework_enforcement_step3(summary_rows)
    print(f"wrote {OUTPUT_PATH_LOCK3}")
    for row in summary_rows:
        print(
            f"{row['scope']}:{row['metric']} count={row['count']} "
            f"total={row['total']} fraction={float(row['fraction']):.4f}"
        )
