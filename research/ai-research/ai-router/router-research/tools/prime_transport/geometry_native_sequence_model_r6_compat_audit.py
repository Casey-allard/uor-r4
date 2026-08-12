#!/usr/bin/env python3
"""Compatibility audit for the native r6 transport system."""

from __future__ import annotations

import csv
from collections import Counter
from dataclasses import dataclass
from pathlib import Path

import numpy as np
import torch

from geometry_native_sequence_model_r6 import (
    GeometryNativeSequenceModelR6,
    LearnedJointTransportOperatorR6,
    PAIR_COUNT_R6,
    TaskConfigR6,
    _joint_contexts_r6,
    _pair_index_to_components,
    _prepare_step_context_r6,
)
from geometry_native_sequence_model_r3 import _decode_factors_r3, _encode_factors_r3, _role_from_native_state_r3
from geometry_native_sequence_model_v3 import (
    TOKEN_TO_ID_V3,
    _initial_discourse_state,
    _initial_geometry_state,
    _role_entity,
    generate_dataset_v3,
)


OUTPUT_PATH = Path(
    "/Users/adminamn/AI-Research/ai-router/router-research/results/"
    "prime_transport_recursive_system/prime_transport_radial_compatibility_audit.csv"
)


@dataclass
class EvalSummary:
    loss: float
    accuracy: float
    query_accuracy: float
    fallback_rate: float


def _fiber_depth_proxy(semiprime: int) -> int:
    left, right = _decode_factors_r3(semiprime)
    return 0 if left == right else 1


def _spin_class_proxy(phi: int, spin_bits: tuple[int, ...], left: int, right: int) -> int:
    return int((phi + sum(int(bit) for bit in spin_bits) + left + right) % 3)


def _compatibility_class_proxy(semiprime: int, opposite_semiprime: int) -> str:
    return "".join("1" if semiprime % prime == 0 and opposite_semiprime % prime == 0 else "0" for prime in (2, 3, 5))


def _joint_logits(operator: LearnedJointTransportOperatorR6, context: tuple[int, ...]) -> np.ndarray:
    mode, token, left_factor, right_factor, opposite_left, opposite_right, chart_turn, discourse_turn, spin_turn, tag_turn, query_flag, tag_flag = context
    logits = (
        operator.base[mode]
        + operator.token[mode, token]
        + operator.opposite_left[mode, opposite_left]
        + operator.opposite_right[mode, opposite_right]
        + operator.chart_turn[mode, chart_turn]
        + operator.discourse_turn[mode, discourse_turn]
        + operator.spin_turn[mode, spin_turn]
        + operator.tag_turn[mode, tag_turn]
        + operator.query_flag[mode, query_flag]
        + operator.tag_flag[mode, tag_flag]
    ).copy()
    for pair_index in range(logits.shape[0]):
        anchor_factor, partner_factor = _pair_index_to_components(pair_index, left_factor, right_factor)
        other_factor = right_factor if anchor_factor == left_factor else left_factor
        logits[pair_index] += (
            operator.anchor_factor[mode, anchor_factor, pair_index]
            + operator.other_factor[mode, other_factor, pair_index]
            + operator.partner_factor[mode, partner_factor, pair_index]
        )
    return logits


def _state_arrays_from_snapshot(snapshot: dict[str, int | list[int] | tuple[int, ...]], token_id: int) -> dict[str, np.ndarray]:
    query_semiprime = int(snapshot["query_semiprime"])
    binding_semiprime = int(snapshot["binding_semiprime"])
    return {
        "token_id": np.array([[token_id]], dtype=np.int64),
        "b": np.array([[int(snapshot["b"])]], dtype=np.int64),
        "phi": np.array([[int(snapshot["phi"])]], dtype=np.int64),
        "r": np.array([[int(snapshot["r"])]], dtype=np.int64),
        "focus": np.array([[int(snapshot["focus"])]], dtype=np.int64),
        "speaker": np.array([[int(snapshot["speaker"])]], dtype=np.int64),
        "topic": np.array([[int(snapshot["topic"])]], dtype=np.int64),
        "style": np.array([[int(snapshot["style"])]], dtype=np.int64),
        "gap": np.array([[int(snapshot["next_return_gap"]) - 1]], dtype=np.int64),
        "query_semiprime": np.array([[ (4, 6, 9, 10, 15, 25).index(query_semiprime) ]], dtype=np.int64),
        "binding_semiprime": np.array([[ (4, 6, 9, 10, 15, 25).index(binding_semiprime) ]], dtype=np.int64),
        "overlap": np.array([[ (1, 2, 3, 4, 5, 6, 9, 10, 15, 25).index(int(np.gcd(query_semiprime, binding_semiprime))) ]], dtype=np.int64),
        "admissible": np.array([[int(snapshot["admissible_transition"])]], dtype=np.int64),
        "spin_bits": np.array([[list(snapshot["spin_bits"])]], dtype=np.int64),
        "tags": np.array([[list(snapshot["tags"])]], dtype=np.int64),
    }


def _build_snapshot_from_choices(
    step_ctx: dict[str, object],
    q_anchor: int,
    q_partner: int,
    b_anchor: int,
    b_partner: int,
) -> dict[str, int | list[int] | tuple[int, ...]]:
    q_next = _encode_factors_r3(q_anchor, q_partner)
    b_next = _encode_factors_r3(b_anchor, b_partner)
    prev_q = int(step_ctx["query_semiprime"])
    prev_b = int(step_ctx["binding_semiprime"])
    admissible = int(np.gcd(q_next, prev_q) > 1 and np.gcd(b_next, prev_b) > 1)
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
    return {
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


def _choose_pair(logits: np.ndarray, allowed_mask: np.ndarray | None) -> tuple[int, bool]:
    if allowed_mask is not None and allowed_mask.any():
        masked = logits.copy()
        masked[~allowed_mask] = -1.0e18
        return int(np.argmax(masked)), False
    return int(np.argmax(logits)), bool(allowed_mask is not None and not allowed_mask.any())


def run_audit(config: TaskConfigR6, *, seed: int = 241) -> tuple[list[dict[str, object]], dict[str, EvalSummary]]:
    train_inputs, train_targets, _ = generate_dataset_v3(config.train_size, seq_len=config.seq_len, seed=seed)
    test_inputs, test_targets, test_query_mask = generate_dataset_v3(config.test_size, seq_len=config.seq_len, seed=seed + 1)

    operator = LearnedJointTransportOperatorR6()
    operator.fit(train_inputs)

    # train readout on the native r6 trajectory
    from geometry_native_sequence_model_r6 import extract_structured_state_r6

    train_states, _ = extract_structured_state_r6(train_inputs, operator)
    model = GeometryNativeSequenceModelR6(seed=seed)
    model.fit(train_states, train_targets)

    rows: list[dict[str, object]] = []
    summaries: dict[str, EvalSummary] = {}

    for filter_mode in ("unfiltered", "compat_filtered"):
        total_loss = 0.0
        total_correct = 0
        total_count = 0
        total_query_correct = 0
        total_query_count = 0
        fallback_count = 0
        step_count = 0

        for row_idx in range(test_inputs.shape[0]):
            b, phi, r = _initial_geometry_state()
            focus, speaker, topic, style, gap, tags = _initial_discourse_state()
            query_semiprime = 6
            binding_semiprime = 10
            spin_bits = (0, 0, 0, 1)

            for col_idx in range(test_inputs.shape[1]):
                token_id = int(test_inputs[row_idx, col_idx])
                target = int(test_targets[row_idx, col_idx])
                is_query = bool(test_query_mask[row_idx, col_idx] > 0.5)
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

                current_q_left, current_q_right = q_ctx[2], q_ctx[3]
                current_b_left, current_b_right = b_ctx[2], b_ctx[3]
                current_q_spin = _spin_class_proxy(phi, spin_bits, current_q_left, current_q_right)
                current_b_spin = _spin_class_proxy(phi, spin_bits, current_b_left, current_b_right)
                current_q_depth = _fiber_depth_proxy(query_semiprime)
                current_b_depth = _fiber_depth_proxy(binding_semiprime)
                current_q_compat = _compatibility_class_proxy(query_semiprime, binding_semiprime)
                current_b_compat = _compatibility_class_proxy(binding_semiprime, query_semiprime)

                q_logits = _joint_logits(operator, q_ctx)
                b_logits = _joint_logits(operator, b_ctx)
                q_allowed = np.ones(PAIR_COUNT_R6, dtype=bool)
                b_allowed = np.ones(PAIR_COUNT_R6, dtype=bool)
                q_incompatible = 0
                b_incompatible = 0

                candidate_meta: list[dict[str, object]] = []
                for side_name, context, logits, current_semiprime, current_depth, current_spin, current_compat in (
                    ("query", q_ctx, q_logits, query_semiprime, current_q_depth, current_q_spin, current_q_compat),
                    ("binding", b_ctx, b_logits, binding_semiprime, current_b_depth, current_b_spin, current_b_compat),
                ):
                    current_r = r
                    proposed_r = int(step_ctx["proposed_r"])
                    proposed_phi = int(step_ctx["proposed_phi"])
                    current_left, current_right = context[2], context[3]
                    opposite_semiprime = binding_semiprime if side_name == "query" else query_semiprime
                    for pair_index in range(PAIR_COUNT_R6):
                        anchor, partner = _pair_index_to_components(pair_index, current_left, current_right)
                        candidate_semiprime = _encode_factors_r3(anchor, partner)
                        candidate_depth = _fiber_depth_proxy(candidate_semiprime)
                        candidate_spin = _spin_class_proxy(proposed_phi, spin_bits, anchor, partner)
                        candidate_compat = _compatibility_class_proxy(candidate_semiprime, opposite_semiprime)
                        radial_class_match = int((current_r == proposed_r) and (current_depth == candidate_depth))
                        spin_class_match = int(current_spin == candidate_spin)
                        compat_match = int(current_compat == candidate_compat)
                        incompatible = int((not radial_class_match) or (not spin_class_match))
                        if filter_mode == "compat_filtered" and incompatible:
                            if side_name == "query":
                                q_allowed[pair_index] = False
                            else:
                                b_allowed[pair_index] = False
                        if side_name == "query":
                            q_incompatible += incompatible
                        else:
                            b_incompatible += incompatible

                        candidate_meta.append(
                            {
                                "model": "r6",
                                "filter_mode": filter_mode,
                                "row": row_idx,
                                "col": col_idx,
                                "side": side_name,
                                "token_id": token_id,
                                "query_token": int(is_query),
                                "current_semiprime": current_semiprime,
                                "current_factor_pair": f"{current_left}:{current_right}",
                                "candidate_anchor": anchor,
                                "candidate_partner": partner,
                                "candidate_semiprime": candidate_semiprime,
                                "current_r": current_r,
                                "proposed_r": proposed_r,
                                "current_fiber_depth": current_depth,
                                "candidate_fiber_depth": candidate_depth,
                                "current_spin_class": current_spin,
                                "candidate_spin_class": candidate_spin,
                                "current_compat_class": current_compat,
                                "candidate_compat_class": candidate_compat,
                                "radial_class_match": radial_class_match,
                                "spin_class_match": spin_class_match,
                                "compat_class_match": compat_match,
                                "score_logit": float(logits[pair_index]),
                                "selected": 0,
                                "correct": 0,
                                "step_loss": 0.0,
                                "prediction": -1,
                                "target": target,
                                "query_accuracy_denominator": int(is_query),
                                "incompatible_candidate_count_side": 0,
                                "fallback_used_side": 0,
                            }
                        )

                q_choice, q_fallback = _choose_pair(q_logits, q_allowed if filter_mode == "compat_filtered" else None)
                b_choice, b_fallback = _choose_pair(b_logits, b_allowed if filter_mode == "compat_filtered" else None)
                fallback_count += int(q_fallback) + int(b_fallback)
                step_count += 2

                q_anchor, q_partner = _pair_index_to_components(q_choice, current_q_left, current_q_right)
                b_anchor, b_partner = _pair_index_to_components(b_choice, current_b_left, current_b_right)
                snapshot = _build_snapshot_from_choices(step_ctx, q_anchor, q_partner, b_anchor, b_partner)
                state_arrays = _state_arrays_from_snapshot(snapshot, token_id)
                tensor_state = model._tensorize_states(state_arrays)
                with torch.no_grad():
                    logits = model.readout(tensor_state)
                    loss = float(model.loss_fn(logits, torch.tensor([target], dtype=torch.long)).item())
                    prediction = int(logits.argmax(dim=-1).item())
                correct = int(prediction == target)
                total_loss += loss
                total_correct += correct
                total_count += 1
                if is_query:
                    total_query_correct += correct
                    total_query_count += 1

                for entry in candidate_meta:
                    side_name = str(entry["side"])
                    pair_index = (
                        (0 if entry["candidate_anchor"] == current_q_left else 1) * 3 + int(entry["candidate_partner"])
                        if side_name == "query"
                        else (0 if entry["candidate_anchor"] == current_b_left else 1) * 3 + int(entry["candidate_partner"])
                    )
                    entry["selected"] = int((side_name == "query" and pair_index == q_choice) or (side_name == "binding" and pair_index == b_choice))
                    entry["correct"] = correct
                    entry["step_loss"] = loss
                    entry["prediction"] = prediction
                    entry["incompatible_candidate_count_side"] = q_incompatible if side_name == "query" else b_incompatible
                    entry["fallback_used_side"] = int(q_fallback if side_name == "query" else b_fallback)
                    rows.append(entry)

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

        summaries[filter_mode] = EvalSummary(
            loss=total_loss / total_count,
            accuracy=total_correct / total_count,
            query_accuracy=(total_query_correct / total_query_count) if total_query_count else 0.0,
            fallback_rate=fallback_count / step_count,
        )

    return rows, summaries


def main() -> None:
    rows, summaries = run_audit(TaskConfigR6())
    OUTPUT_PATH.parent.mkdir(parents=True, exist_ok=True)
    with OUTPUT_PATH.open("w", newline="", encoding="utf-8") as handle:
        writer = csv.DictWriter(handle, fieldnames=list(rows[0].keys()))
        writer.writeheader()
        writer.writerows(rows)

    selected_unfiltered = [row for row in rows if row["filter_mode"] == "unfiltered" and row["selected"]]
    radial_selected_mismatch = sum(1 for row in selected_unfiltered if not row["radial_class_match"])
    spin_selected_mismatch = sum(1 for row in selected_unfiltered if not row["spin_class_match"])
    avg_incompat = sum(int(row["incompatible_candidate_count_side"]) for row in selected_unfiltered) / len(selected_unfiltered)
    dist = Counter(int(row["incompatible_candidate_count_side"]) for row in selected_unfiltered)

    print(f"wrote {OUTPUT_PATH}")
    print(
        "unfiltered "
        f"loss={summaries['unfiltered'].loss:.4f} "
        f"acc={summaries['unfiltered'].accuracy:.4f} "
        f"query={summaries['unfiltered'].query_accuracy:.4f}"
    )
    print(
        "compat_filtered "
        f"loss={summaries['compat_filtered'].loss:.4f} "
        f"acc={summaries['compat_filtered'].accuracy:.4f} "
        f"query={summaries['compat_filtered'].query_accuracy:.4f} "
        f"fallback_rate={summaries['compat_filtered'].fallback_rate:.4f}"
    )
    print(
        f"selected_radial_mismatch_fraction={radial_selected_mismatch / len(selected_unfiltered):.4f} "
        f"selected_spin_mismatch_fraction={spin_selected_mismatch / len(selected_unfiltered):.4f}"
    )
    print(f"avg_incompatible_candidates_per_selected_side={avg_incompat:.4f}")
    print(f"incompatible_candidate_distribution={dict(sorted(dist.items()))}")


if __name__ == "__main__":
    main()
