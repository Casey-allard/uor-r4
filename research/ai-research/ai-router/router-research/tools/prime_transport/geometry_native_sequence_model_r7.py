#!/usr/bin/env python3
"""Seventh inner-representation rebuild with compatibility-conditioned transport."""

from __future__ import annotations

import csv
from dataclasses import dataclass
from pathlib import Path

import numpy as np

from geometry_native_sequence_model_r2 import GeometryNativeSequenceModelR2, SequenceMetricsR2, StructuredNativeReadoutR2
from geometry_native_sequence_model_r3 import (
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
)
from geometry_native_sequence_model_r6 import (
    GeometryNativeSequenceModelR6,
    LearnedJointTransportOperatorR6,
    PAIR_COUNT_R6,
    _joint_contexts_r6,
    _pair_index_to_components,
    _prepare_step_context_r6,
    _semiprime_idx_r6,
)
from geometry_native_sequence_model_v3 import (
    QUERY_IDS_V3,
    TransformerSequenceTrainerV3,
    _initial_discourse_state,
    _initial_geometry_state,
    _role_entity,
    generate_dataset_v3,
)
from geometry_native_sequence_model_r6_compat_audit import run_audit as run_r6_compat_audit


R4_CSV = Path("/Users/adminamn/AI-Research/ai-router/router-research/results/prime_transport_recursive_system/prime_transport_inner_representation_rebuild_v4.csv")
R5_CSV = Path("/Users/adminamn/AI-Research/ai-router/router-research/results/prime_transport_recursive_system/prime_transport_inner_representation_rebuild_v5.csv")
R6_CSV = Path("/Users/adminamn/AI-Research/ai-router/router-research/results/prime_transport_recursive_system/prime_transport_inner_representation_rebuild_v6.csv")


@dataclass(frozen=True)
class TaskConfigR7:
    seq_len: int = 30
    train_size: int = 1024
    test_size: int = 256


@dataclass
class CompatibilityStatsR7:
    selected_radial_mismatch_fraction: float
    selected_spin_mismatch_fraction: float
    selected_compat_mismatch_fraction: float
    avg_scored_candidates_per_side: float
    compared_incompatible_fraction: float
    tier_usage_fraction_t0: float
    tier_usage_fraction_t1: float
    tier_usage_fraction_t2: float
    tier_usage_fraction_t3: float
    tier_usage_fraction_t4: float


def _fiber_depth_proxy_r7(semiprime: int) -> int:
    left, right = _decode_factors_r3(semiprime)
    return 0 if left == right else 1


def _spin_class_proxy_r7(phi: int, spin_bits: tuple[int, ...], left: int, right: int) -> int:
    return int((phi + sum(int(bit) for bit in spin_bits) + left + right) % 3)


def _compatibility_class_proxy_r7(semiprime: int, opposite_semiprime: int) -> str:
    return "".join("1" if semiprime % prime == 0 and opposite_semiprime % prime == 0 else "0" for prime in (2, 3, 5))


def _compatibility_tier_r7(
    *,
    current_r: int,
    proposed_r: int,
    current_depth: int,
    candidate_depth: int,
    current_spin: int,
    candidate_spin: int,
    current_compat: str,
    candidate_compat: str,
) -> int:
    radial_match = int(current_r == proposed_r and current_depth == candidate_depth)
    spin_match = int(current_spin == candidate_spin)
    compat_match = int(current_compat == candidate_compat)
    if compat_match and radial_match and spin_match:
        return 0
    if compat_match and radial_match:
        return 1
    if compat_match and spin_match:
        return 2
    if compat_match:
        return 3
    return 4


def _state_arrays_from_snapshot_r7(snapshot: dict[str, int | list[int] | tuple[int, ...]], token_id: int) -> dict[str, np.ndarray]:
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
        "query_semiprime": np.array([[(4, 6, 9, 10, 15, 25).index(query_semiprime)]], dtype=np.int64),
        "binding_semiprime": np.array([[(4, 6, 9, 10, 15, 25).index(binding_semiprime)]], dtype=np.int64),
        "overlap": np.array([[(1, 2, 3, 4, 5, 6, 9, 10, 15, 25).index(int(np.gcd(query_semiprime, binding_semiprime)))]], dtype=np.int64),
        "admissible": np.array([[int(snapshot["admissible_transition"])]], dtype=np.int64),
        "spin_bits": np.array([[list(snapshot["spin_bits"])]], dtype=np.int64),
        "tags": np.array([[list(snapshot["tags"])]], dtype=np.int64),
    }


def _read_metric_row(csv_path: Path, model_name: str) -> SequenceMetricsR2:
    with csv_path.open("r", encoding="utf-8") as handle:
        for row in csv.DictReader(handle):
            if row["model"] == model_name:
                return SequenceMetricsR2(
                    model=row["model"],
                    test_loss=float(row["test_loss"]),
                    test_accuracy=float(row["test_accuracy"]),
                    query_accuracy=float(row["query_accuracy"]),
                    param_count=int(row["param_count"]),
                    effective_state_size=int(row["effective_state_size"]),
                    train_seconds=float(row["train_seconds"]),
                    eval_seconds=float(row["eval_seconds"]),
                )
    raise ValueError(f"{model_name} not found in {csv_path}")


class CompatibilityConditionedJointOperatorR7(LearnedJointTransportOperatorR6):
    """Joint native operator with compatibility tiers inside the transport law."""

    def predict_pair_with_stats(
        self,
        context: tuple[int, ...],
        *,
        current_r: int,
        proposed_r: int,
        current_phi: int,
        proposed_phi: int,
        spin_bits: tuple[int, ...],
        current_semiprime: int,
        opposite_semiprime: int,
    ) -> tuple[tuple[int, int], dict[str, float]]:
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
        ).copy()

        current_depth = _fiber_depth_proxy_r7(current_semiprime)
        current_spin = _spin_class_proxy_r7(current_phi, spin_bits, left_factor, right_factor)
        current_compat = _compatibility_class_proxy_r7(current_semiprime, opposite_semiprime)

        tiers = np.zeros(PAIR_COUNT_R6, dtype=np.int64)
        radial_mismatch = np.zeros(PAIR_COUNT_R6, dtype=np.int64)
        spin_mismatch = np.zeros(PAIR_COUNT_R6, dtype=np.int64)
        compat_mismatch = np.zeros(PAIR_COUNT_R6, dtype=np.int64)

        for pair_index in range(PAIR_COUNT_R6):
            anchor_factor, partner_factor = _pair_index_to_components(pair_index, left_factor, right_factor)
            other_factor = right_factor if anchor_factor == left_factor else left_factor
            candidate_semiprime = _encode_factors_r3(anchor_factor, partner_factor)
            candidate_depth = _fiber_depth_proxy_r7(candidate_semiprime)
            candidate_spin = _spin_class_proxy_r7(proposed_phi, spin_bits, anchor_factor, partner_factor)
            candidate_compat = _compatibility_class_proxy_r7(candidate_semiprime, opposite_semiprime)
            tier = _compatibility_tier_r7(
                current_r=current_r,
                proposed_r=proposed_r,
                current_depth=current_depth,
                candidate_depth=candidate_depth,
                current_spin=current_spin,
                candidate_spin=candidate_spin,
                current_compat=current_compat,
                candidate_compat=candidate_compat,
            )
            tiers[pair_index] = tier
            radial_mismatch[pair_index] = int(not (current_r == proposed_r and current_depth == candidate_depth))
            spin_mismatch[pair_index] = int(current_spin != candidate_spin)
            compat_mismatch[pair_index] = int(current_compat != candidate_compat)
            logits[pair_index] += (
                self.anchor_factor[mode, anchor_factor, pair_index]
                + self.other_factor[mode, other_factor, pair_index]
                + self.partner_factor[mode, partner_factor, pair_index]
            )

        min_tier = int(tiers.min())
        allowed = tiers == min_tier
        masked = logits.copy()
        masked[~allowed] = -1.0e18
        best_pair = int(np.argmax(masked))
        pair = _pair_index_to_components(best_pair, left_factor, right_factor)
        stats = {
            "selected_radial_mismatch": float(radial_mismatch[best_pair]),
            "selected_spin_mismatch": float(spin_mismatch[best_pair]),
            "selected_compat_mismatch": float(compat_mismatch[best_pair]),
            "scored_candidates": float(int(allowed.sum())),
            "compared_incompatible_fraction": float(((allowed & ((radial_mismatch == 1) | (spin_mismatch == 1))).sum()) / allowed.sum()),
            "tier": float(min_tier),
        }
        return pair, stats


def step_discourse_world_r7(
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
    operator: CompatibilityConditionedJointOperatorR7,
) -> tuple[int, dict[str, int | list[int] | tuple[int, ...]], bool, dict[str, float]]:
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
    q_pair, q_stats = operator.predict_pair_with_stats(
        q_ctx,
        current_r=r,
        proposed_r=int(step_ctx["proposed_r"]),
        current_phi=phi,
        proposed_phi=int(step_ctx["proposed_phi"]),
        spin_bits=spin_bits,
        current_semiprime=query_semiprime,
        opposite_semiprime=binding_semiprime,
    )
    b_pair, b_stats = operator.predict_pair_with_stats(
        b_ctx,
        current_r=r,
        proposed_r=int(step_ctx["proposed_r"]),
        current_phi=phi,
        proposed_phi=int(step_ctx["proposed_phi"]),
        spin_bits=spin_bits,
        current_semiprime=binding_semiprime,
        opposite_semiprime=query_semiprime,
    )

    q_next = _encode_factors_r3(*q_pair)
    b_next = _encode_factors_r3(*b_pair)
    admissible = int(np.gcd(q_next, query_semiprime) > 1 and np.gcd(b_next, binding_semiprime) > 1)
    next_spin = tuple(list(spin_bits[1:]) + [admissible])
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

    merged_stats = {
        "query_selected_radial_mismatch": q_stats["selected_radial_mismatch"],
        "query_selected_spin_mismatch": q_stats["selected_spin_mismatch"],
        "binding_selected_radial_mismatch": b_stats["selected_radial_mismatch"],
        "binding_selected_spin_mismatch": b_stats["selected_spin_mismatch"],
        "query_selected_compat_mismatch": q_stats["selected_compat_mismatch"],
        "binding_selected_compat_mismatch": b_stats["selected_compat_mismatch"],
        "query_scored_candidates": q_stats["scored_candidates"],
        "binding_scored_candidates": b_stats["scored_candidates"],
        "query_compared_incompatible_fraction": q_stats["compared_incompatible_fraction"],
        "binding_compared_incompatible_fraction": b_stats["compared_incompatible_fraction"],
        "query_tier": q_stats["tier"],
        "binding_tier": b_stats["tier"],
    }
    return int(target), snapshot, bool(step_ctx["is_query"]), merged_stats


def extract_structured_state_r7(
    inputs: np.ndarray,
    operator: CompatibilityConditionedJointOperatorR7,
) -> tuple[dict[str, np.ndarray], np.ndarray, CompatibilityStatsR7]:
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
    radial_mismatch_total = 0.0
    spin_mismatch_total = 0.0
    compat_mismatch_total = 0.0
    scored_candidates_total = 0.0
    compared_incompatible_total = 0.0
    tier_counter = np.zeros(5, dtype=np.float64)
    side_count = 0.0

    for row in range(size):
        b, phi, r = _initial_geometry_state()
        focus, speaker, topic, style, gap, tags = _initial_discourse_state()
        query_semiprime = 6
        binding_semiprime = 10
        spin_bits = (0, 0, 0, 1)

        for col in range(seq_len):
            token_id = int(inputs[row, col])
            _, snapshot, is_query, stats = step_discourse_world_r7(
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
                operator=operator,
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

            radial_mismatch_total += stats["query_selected_radial_mismatch"] + stats["binding_selected_radial_mismatch"]
            spin_mismatch_total += stats["query_selected_spin_mismatch"] + stats["binding_selected_spin_mismatch"]
            compat_mismatch_total += stats["query_selected_compat_mismatch"] + stats["binding_selected_compat_mismatch"]
            scored_candidates_total += stats["query_scored_candidates"] + stats["binding_scored_candidates"]
            compared_incompatible_total += stats["query_compared_incompatible_fraction"] + stats["binding_compared_incompatible_fraction"]
            tier_counter[int(stats["query_tier"])] += 1.0
            tier_counter[int(stats["binding_tier"])] += 1.0
            side_count += 2.0

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

    stats = CompatibilityStatsR7(
        selected_radial_mismatch_fraction=radial_mismatch_total / side_count,
        selected_spin_mismatch_fraction=spin_mismatch_total / side_count,
        selected_compat_mismatch_fraction=compat_mismatch_total / side_count,
        avg_scored_candidates_per_side=scored_candidates_total / side_count,
        compared_incompatible_fraction=compared_incompatible_total / side_count,
        tier_usage_fraction_t0=tier_counter[0] / side_count,
        tier_usage_fraction_t1=tier_counter[1] / side_count,
        tier_usage_fraction_t2=tier_counter[2] / side_count,
        tier_usage_fraction_t3=tier_counter[3] / side_count,
        tier_usage_fraction_t4=tier_counter[4] / side_count,
    )
    return states, query_mask, stats


class GeometryNativeSequenceModelR7(GeometryNativeSequenceModelR2):
    def __init__(self, *, seed: int = 0) -> None:
        super().__init__(seed=seed)
        self.readout = StructuredNativeReadoutR2()


def run_bounded_sequence_comparison_r7(config: TaskConfigR7, *, seed: int = 241) -> tuple[list[SequenceMetricsR2], CompatibilityStatsR7]:
    train_inputs, train_targets, _ = generate_dataset_v3(config.train_size, seq_len=config.seq_len, seed=seed)
    test_inputs, test_targets, test_query_mask = generate_dataset_v3(config.test_size, seq_len=config.seq_len, seed=seed + 1)

    ref_v4 = _read_metric_row(R4_CSV, "geometry_native_sequence_model_r4")
    ref_v5 = _read_metric_row(R5_CSV, "geometry_native_sequence_model_r5")
    ref_v6 = _read_metric_row(R6_CSV, "geometry_native_sequence_model_r6")
    ref_v3 = _read_metric_row(R6_CSV, "geometry_native_sequence_model_v3_reference")
    ref_transformer = _read_metric_row(R6_CSV, "tiny_transformer_baseline_r6")

    _, audit_summaries = run_r6_compat_audit(TaskConfigR7(), seed=seed)
    r6_filtered = SequenceMetricsR2(
        model="geometry_native_sequence_model_r6_compat_filtered_reference",
        test_loss=audit_summaries["compat_filtered"].loss,
        test_accuracy=audit_summaries["compat_filtered"].accuracy,
        query_accuracy=audit_summaries["compat_filtered"].query_accuracy,
        param_count=ref_v6.param_count,
        effective_state_size=ref_v6.effective_state_size,
        train_seconds=ref_v6.train_seconds,
        eval_seconds=ref_v6.eval_seconds,
    )

    operator = CompatibilityConditionedJointOperatorR7()
    operator.fit(train_inputs)
    train_states, _, _ = extract_structured_state_r7(train_inputs, operator)
    test_states, _, compat_stats = extract_structured_state_r7(test_inputs, operator)

    model = GeometryNativeSequenceModelR7(seed=seed)
    _train_loss_r7, train_seconds_r7 = model.fit(train_states, train_targets)
    test_loss_r7, test_acc_r7, query_acc_r7, eval_seconds_r7 = model.evaluate(
        test_states,
        test_targets,
        test_query_mask,
    )

    r7_metrics = SequenceMetricsR2(
        model="geometry_native_sequence_model_r7",
        test_loss=test_loss_r7,
        test_accuracy=test_acc_r7,
        query_accuracy=query_acc_r7,
        param_count=model.param_count + operator.param_count,
        effective_state_size=model.effective_state_size,
        train_seconds=train_seconds_r7,
        eval_seconds=eval_seconds_r7,
    )

    return [
        ref_v3,
        ref_v4,
        ref_v5,
        ref_v6,
        r6_filtered,
        r7_metrics,
        SequenceMetricsR2(
            model="tiny_transformer_baseline_r7",
            test_loss=ref_transformer.test_loss,
            test_accuracy=ref_transformer.test_accuracy,
            query_accuracy=ref_transformer.query_accuracy,
            param_count=ref_transformer.param_count,
            effective_state_size=ref_transformer.effective_state_size,
            train_seconds=ref_transformer.train_seconds,
            eval_seconds=ref_transformer.eval_seconds,
        ),
    ], compat_stats
