#!/usr/bin/env python3
"""Learned weighting over lawful H_v7 operator entries using tau-aware transport state."""

from __future__ import annotations

import csv
import math
from dataclasses import dataclass
from pathlib import Path
from time import perf_counter

import numpy as np

from geometry_native_operator_model_v10 import (
    OperatorStateV10,
    class_tuple_v10,
    composite_swap_component_v10,
    composite_twist_component_v10,
    coupled_torus_kick_component_v10,
    fiber_phase_lift_component_v10,
    hold_component_v10,
    initial_operator_state_v10,
    radial_transport_component_v10,
    torus_base_advance_component_v10,
)
from geometry_native_operator_model_v8 import (
    R5_CSV,
    RuntimeDiscourseStateV8,
    TaskConfigV8,
    _class_probabilities_v8,
    _predicted_class_v8,
    _read_reference_metric_v8,
    _update_discourse_v8,
)
from geometry_native_sequence_model_v3 import QUERY_IDS_V3, TAG_IDS, _initial_discourse_state, generate_dataset_v3
from geometry_native_spinH_candidate_v3 import active_transport_lift_v3


OUTPUT_PATH_OPERATOR_WEIGHTING_V3 = Path(
    "/Users/adminamn/AI-Research/ai-router/router-research/results/"
    "prime_transport_recursive_system/prime_transport_operator_weighting_v3.csv"
)
V8_CSV = Path(
    "/Users/adminamn/AI-Research/ai-router/router-research/results/"
    "prime_transport_recursive_system/prime_transport_operator_weighting_v2.csv"
)
V9_CSV = Path(
    "/Users/adminamn/AI-Research/ai-router/router-research/results/"
    "prime_transport_recursive_system/prime_transport_operator_with_tau_v1.csv"
)


@dataclass
class SequenceMetricsV11:
    model: str
    test_loss: float
    test_accuracy: float
    query_accuracy: float
    param_count: int
    effective_state_size: int
    train_seconds: float
    eval_seconds: float
    usage_I: float
    usage_T_b: float
    usage_T_x: float
    usage_T_c: float
    usage_T_y: float
    usage_T_z_prime: float
    usage_T_r: float
    radial_change_fraction: float
    fiber_change_fraction: float
    lawful_spin_change_fraction: float
    illegal_transition_fraction: float
    outside_support_fraction: float


def _generator_apply_v11(state: OperatorStateV10, generator_index: int) -> tuple[OperatorStateV10, str]:
    if generator_index == 0:
        return hold_component_v10(state), "hold"
    if generator_index == 1:
        return torus_base_advance_component_v10(state), "torus_base_advance"
    if generator_index == 2:
        return composite_swap_component_v10(state), "composite_swap"
    if generator_index == 3:
        return coupled_torus_kick_component_v10(state), "coupled_torus_kick"
    if generator_index == 4:
        return composite_twist_component_v10(state), "composite_twist"
    if generator_index == 5:
        return fiber_phase_lift_component_v10(state), "fiber_phase_lift_spin_transport"
    if generator_index == 6:
        return radial_transport_component_v10(state), "radial_transport"
    raise ValueError(f"unknown generator index {generator_index}")


def _context_indices_v11(
    token_id: int,
    *,
    operator_state: OperatorStateV10,
    discourse_state: RuntimeDiscourseStateV8,
) -> tuple[int, ...]:
    transport_state = active_transport_lift_v3(operator_state, operator_state.tau)
    query_idx = 0 if operator_state.query_semiprime == 6 else 1
    binding_idx = 0 if operator_state.binding_semiprime == 6 else 1
    query_flag = int(token_id in QUERY_IDS_V3)
    tag_flag = int(token_id in TAG_IDS)
    compat_idx = int("".join(operator_state.composite_compat_class) != "100100")
    return (
        token_id,
        operator_state.b,
        operator_state.phi,
        operator_state.r,
        operator_state.twist,
        query_idx,
        binding_idx,
        discourse_state.style,
        discourse_state.next_return_gap - 1,
        discourse_state.focus,
        discourse_state.speaker,
        discourse_state.topic,
        query_flag,
        tag_flag,
        int(operator_state.spin_h.bits[0]),
        int(operator_state.spin_h.bits[1]),
        int(operator_state.spin_h.bits[2]),
        int(operator_state.spin_h.bits[3]),
        compat_idx,
        transport_state.spin_H_candidate.tau.swap_phase,
        transport_state.spin_H_candidate.tau.coupled_phase,
        transport_state.spin_H_candidate.tau.twist_phase,
        transport_state.spin_H_candidate.tau.lift_phase,
    )


class StructuredLawfulOperatorSelectorV11:
    """Structured additive selector over the seven lawful H_v7 generators."""

    def __init__(self) -> None:
        self.base = np.zeros(7, dtype=np.float64)
        self.token = np.zeros((11, 7), dtype=np.float64)
        self.b = np.zeros((5, 7), dtype=np.float64)
        self.phi = np.zeros((3, 7), dtype=np.float64)
        self.r = np.zeros((3, 7), dtype=np.float64)
        self.twist = np.zeros((2, 7), dtype=np.float64)
        self.query_semiprime = np.zeros((2, 7), dtype=np.float64)
        self.binding_semiprime = np.zeros((2, 7), dtype=np.float64)
        self.style = np.zeros((3, 7), dtype=np.float64)
        self.gap = np.zeros((4, 7), dtype=np.float64)
        self.focus = np.zeros((3, 7), dtype=np.float64)
        self.speaker = np.zeros((3, 7), dtype=np.float64)
        self.topic = np.zeros((3, 7), dtype=np.float64)
        self.query_flag = np.zeros((2, 7), dtype=np.float64)
        self.tag_flag = np.zeros((2, 7), dtype=np.float64)
        self.spin_bit0 = np.zeros((2, 7), dtype=np.float64)
        self.spin_bit1 = np.zeros((2, 7), dtype=np.float64)
        self.spin_bit2 = np.zeros((2, 7), dtype=np.float64)
        self.spin_bit3 = np.zeros((2, 7), dtype=np.float64)
        self.compat = np.zeros((2, 7), dtype=np.float64)
        self.tau_swap = np.zeros((2, 7), dtype=np.float64)
        self.tau_coupled = np.zeros((5, 7), dtype=np.float64)
        self.tau_twist = np.zeros((2, 7), dtype=np.float64)
        self.tau_lift = np.zeros((12, 7), dtype=np.float64)

    @property
    def param_count(self) -> int:
        return sum(
            table.size
            for table in (
                self.base,
                self.token,
                self.b,
                self.phi,
                self.r,
                self.twist,
                self.query_semiprime,
                self.binding_semiprime,
                self.style,
                self.gap,
                self.focus,
                self.speaker,
                self.topic,
                self.query_flag,
                self.tag_flag,
                self.spin_bit0,
                self.spin_bit1,
                self.spin_bit2,
                self.spin_bit3,
                self.compat,
                self.tau_swap,
                self.tau_coupled,
                self.tau_twist,
                self.tau_lift,
            )
        )

    def scores(self, context: tuple[int, ...]) -> np.ndarray:
        (
            token_id,
            b_idx,
            phi_idx,
            r_idx,
            twist_idx,
            query_idx,
            binding_idx,
            style_idx,
            gap_idx,
            focus_idx,
            speaker_idx,
            topic_idx,
            query_flag_idx,
            tag_flag_idx,
            spin0_idx,
            spin1_idx,
            spin2_idx,
            spin3_idx,
            compat_idx,
            tau_swap_idx,
            tau_coupled_idx,
            tau_twist_idx,
            tau_lift_idx,
        ) = context
        return (
            self.base
            + self.token[token_id]
            + self.b[b_idx]
            + self.phi[phi_idx]
            + self.r[r_idx]
            + self.twist[twist_idx]
            + self.query_semiprime[query_idx]
            + self.binding_semiprime[binding_idx]
            + self.style[style_idx]
            + self.gap[gap_idx]
            + self.focus[focus_idx]
            + self.speaker[speaker_idx]
            + self.topic[topic_idx]
            + self.query_flag[query_flag_idx]
            + self.tag_flag[tag_flag_idx]
            + self.spin_bit0[spin0_idx]
            + self.spin_bit1[spin1_idx]
            + self.spin_bit2[spin2_idx]
            + self.spin_bit3[spin3_idx]
            + self.compat[compat_idx]
            + self.tau_swap[tau_swap_idx]
            + self.tau_coupled[tau_coupled_idx]
            + self.tau_twist[tau_twist_idx]
            + self.tau_lift[tau_lift_idx]
        )

    def _update(self, context: tuple[int, ...], generator_index: int, delta: float) -> None:
        (
            token_id,
            b_idx,
            phi_idx,
            r_idx,
            twist_idx,
            query_idx,
            binding_idx,
            style_idx,
            gap_idx,
            focus_idx,
            speaker_idx,
            topic_idx,
            query_flag_idx,
            tag_flag_idx,
            spin0_idx,
            spin1_idx,
            spin2_idx,
            spin3_idx,
            compat_idx,
            tau_swap_idx,
            tau_coupled_idx,
            tau_twist_idx,
            tau_lift_idx,
        ) = context
        self.base[generator_index] += delta
        self.token[token_id, generator_index] += delta
        self.b[b_idx, generator_index] += delta
        self.phi[phi_idx, generator_index] += delta
        self.r[r_idx, generator_index] += delta
        self.twist[twist_idx, generator_index] += delta
        self.query_semiprime[query_idx, generator_index] += delta
        self.binding_semiprime[binding_idx, generator_index] += delta
        self.style[style_idx, generator_index] += delta
        self.gap[gap_idx, generator_index] += delta
        self.focus[focus_idx, generator_index] += delta
        self.speaker[speaker_idx, generator_index] += delta
        self.topic[topic_idx, generator_index] += delta
        self.query_flag[query_flag_idx, generator_index] += delta
        self.tag_flag[tag_flag_idx, generator_index] += delta
        self.spin_bit0[spin0_idx, generator_index] += delta
        self.spin_bit1[spin1_idx, generator_index] += delta
        self.spin_bit2[spin2_idx, generator_index] += delta
        self.spin_bit3[spin3_idx, generator_index] += delta
        self.compat[compat_idx, generator_index] += delta
        self.tau_swap[tau_swap_idx, generator_index] += delta
        self.tau_coupled[tau_coupled_idx, generator_index] += delta
        self.tau_twist[tau_twist_idx, generator_index] += delta
        self.tau_lift[tau_lift_idx, generator_index] += delta

    def fit(
        self,
        train_inputs: np.ndarray,
        train_targets: np.ndarray,
        *,
        epochs: int = 5,
        learning_rate: float = 0.20,
    ) -> float:
        start = perf_counter()
        for _ in range(epochs):
            for row in range(train_inputs.shape[0]):
                operator_state = initial_operator_state_v10()
                focus, speaker, topic, style, gap, tags = _initial_discourse_state()
                discourse_state = RuntimeDiscourseStateV8(
                    focus=int(focus),
                    speaker=int(speaker),
                    topic=int(topic),
                    style=int(style),
                    next_return_gap=int(gap),
                    tags=(int(tags[0]), int(tags[1]), int(tags[2])),
                )
                for col in range(train_inputs.shape[1]):
                    token_id = int(train_inputs[row, col])
                    target = int(train_targets[row, col])
                    context = _context_indices_v11(
                        token_id,
                        operator_state=operator_state,
                        discourse_state=discourse_state,
                    )
                    scores = self.scores(context)

                    predicted_classes = []
                    next_operator_states = []
                    next_discourse_states = []
                    for generator_index in range(7):
                        next_operator, _ = _generator_apply_v11(operator_state, generator_index)
                        next_discourse = _update_discourse_v8(
                            token_id,
                            operator_state=next_operator,
                            discourse_state=discourse_state,
                        )
                        pred_class = _predicted_class_v8(
                            token_id,
                            operator_state=next_operator,
                            discourse_state=next_discourse,
                        )
                        next_operator_states.append(next_operator)
                        next_discourse_states.append(next_discourse)
                        predicted_classes.append(pred_class)

                    chosen = int(np.argmax(scores))
                    correct_generators = [idx for idx, pred in enumerate(predicted_classes) if pred == target]
                    if correct_generators:
                        oracle = max(correct_generators, key=lambda idx: scores[idx])
                        if oracle != chosen:
                            self._update(context, oracle, learning_rate)
                            self._update(context, chosen, -learning_rate)
                        chosen = oracle

                    operator_state = next_operator_states[chosen]
                    discourse_state = next_discourse_states[chosen]
        return perf_counter() - start


def _evaluate_policy_v11(
    inputs: np.ndarray,
    targets: np.ndarray,
    query_mask: np.ndarray,
    *,
    policy_name: str,
    selector: StructuredLawfulOperatorSelectorV11,
    param_count: int,
    train_seconds: float,
) -> SequenceMetricsV11:
    start = perf_counter()
    total_loss = 0.0
    total_count = 0
    correct = 0
    query_correct = 0
    query_total = 0
    usage = np.zeros(7, dtype=np.float64)
    radial_change_hits = 0
    fiber_change_hits = 0
    lawful_spin_change_hits = 0

    for row in range(inputs.shape[0]):
        operator_state = initial_operator_state_v10()
        focus, speaker, topic, style, gap, tags = _initial_discourse_state()
        discourse_state = RuntimeDiscourseStateV8(
            focus=int(focus),
            speaker=int(speaker),
            topic=int(topic),
            style=int(style),
            next_return_gap=int(gap),
            tags=(int(tags[0]), int(tags[1]), int(tags[2])),
        )
        for col in range(inputs.shape[1]):
            token_id = int(inputs[row, col])
            target = int(targets[row, col])
            context = _context_indices_v11(
                token_id,
                operator_state=operator_state,
                discourse_state=discourse_state,
            )

            predicted_classes = []
            next_operator_states = []
            next_discourse_states = []
            for generator_index in range(7):
                next_operator, _ = _generator_apply_v11(operator_state, generator_index)
                next_discourse = _update_discourse_v8(
                    token_id,
                    operator_state=next_operator,
                    discourse_state=discourse_state,
                )
                pred_class = _predicted_class_v8(
                    token_id,
                    operator_state=next_operator,
                    discourse_state=next_discourse,
                )
                next_operator_states.append(next_operator)
                next_discourse_states.append(next_discourse)
                predicted_classes.append(pred_class)

            scores = selector.scores(context)
            generator_index = int(np.argmax(scores))
            probs = _class_probabilities_v8(scores, predicted_classes)

            usage[generator_index] += 1.0
            next_operator = next_operator_states[generator_index]
            next_discourse = next_discourse_states[generator_index]
            pred_class = predicted_classes[generator_index]
            total_loss += -math.log(max(probs[target], 1.0e-12))
            correct += int(pred_class == target)
            total_count += 1
            if query_mask[row, col] > 0.5:
                query_correct += int(pred_class == target)
                query_total += 1

            if next_operator.r != operator_state.r:
                radial_change_hits += 1
            if next_operator.phi != operator_state.phi:
                fiber_change_hits += 1
            if next_operator.spin_h != operator_state.spin_h:
                lawful_spin_change_hits += 1

            operator_state = next_operator
            discourse_state = next_discourse

    elapsed = perf_counter() - start
    total_usage = usage.sum()
    return SequenceMetricsV11(
        model=policy_name,
        test_loss=total_loss / max(total_count, 1),
        test_accuracy=correct / max(total_count, 1),
        query_accuracy=query_correct / max(query_total, 1),
        param_count=param_count,
        effective_state_size=17,
        train_seconds=train_seconds,
        eval_seconds=elapsed,
        usage_I=usage[0] / max(total_usage, 1.0),
        usage_T_b=usage[1] / max(total_usage, 1.0),
        usage_T_x=usage[2] / max(total_usage, 1.0),
        usage_T_c=usage[3] / max(total_usage, 1.0),
        usage_T_y=usage[4] / max(total_usage, 1.0),
        usage_T_z_prime=usage[5] / max(total_usage, 1.0),
        usage_T_r=usage[6] / max(total_usage, 1.0),
        radial_change_fraction=radial_change_hits / max(total_usage, 1.0),
        fiber_change_fraction=fiber_change_hits / max(total_usage, 1.0),
        lawful_spin_change_fraction=lawful_spin_change_hits / max(total_usage, 1.0),
        illegal_transition_fraction=0.0,
        outside_support_fraction=0.0,
    )


def run_operator_weighting_v3(config: TaskConfigV8, *, seed: int = 241) -> list[SequenceMetricsV11]:
    train_inputs, train_targets, _ = generate_dataset_v3(config.train_size, seq_len=config.seq_len, seed=seed)
    test_inputs, test_targets, test_query_mask = generate_dataset_v3(config.test_size, seq_len=config.seq_len, seed=seed + 1)

    selector = StructuredLawfulOperatorSelectorV11()
    train_seconds = selector.fit(train_inputs, train_targets)
    v11_learned = _evaluate_policy_v11(
        test_inputs,
        test_targets,
        test_query_mask,
        policy_name="geometry_native_operator_model_v11",
        selector=selector,
        param_count=selector.param_count,
        train_seconds=train_seconds,
    )

    def _convert_ref(ref) -> SequenceMetricsV11:
        return SequenceMetricsV11(
            model=ref.model,
            test_loss=ref.test_loss,
            test_accuracy=ref.test_accuracy,
            query_accuracy=ref.query_accuracy,
            param_count=ref.param_count,
            effective_state_size=ref.effective_state_size,
            train_seconds=ref.train_seconds,
            eval_seconds=ref.eval_seconds,
            usage_I=getattr(ref, "usage_I", 0.0),
            usage_T_b=getattr(ref, "usage_T_b", 0.0),
            usage_T_x=getattr(ref, "usage_T_x", 0.0),
            usage_T_c=getattr(ref, "usage_T_c", 0.0),
            usage_T_y=getattr(ref, "usage_T_y", 0.0),
            usage_T_z_prime=getattr(ref, "usage_T_z_prime", 0.0),
            usage_T_r=0.0,
            radial_change_fraction=0.0,
            fiber_change_fraction=0.0,
            lawful_spin_change_fraction=getattr(ref, "lawful_spin_change_fraction", 0.0),
            illegal_transition_fraction=ref.illegal_transition_fraction,
            outside_support_fraction=ref.outside_support_fraction,
        )

    v3_ref = _convert_ref(_read_reference_metric_v8(R5_CSV, "geometry_native_sequence_model_v3_reference"))
    r5_ref = _convert_ref(_read_reference_metric_v8(R5_CSV, "geometry_native_sequence_model_r5"))
    v8_ref = _convert_ref(_read_reference_metric_v8(V8_CSV, "geometry_native_operator_model_v8"))
    v9_ref = _convert_ref(_read_reference_metric_v8(V9_CSV, "geometry_native_operator_model_v9"))
    transformer_ref = _convert_ref(_read_reference_metric_v8(R5_CSV, "tiny_transformer_baseline_r5"))
    transformer_ref.model = "tiny_transformer_baseline_v11_reference"
    return [v3_ref, r5_ref, v8_ref, v9_ref, v11_learned, transformer_ref]


def write_operator_weighting_v3(
    rows: list[SequenceMetricsV11],
    output_path: Path = OUTPUT_PATH_OPERATOR_WEIGHTING_V3,
) -> None:
    output_path.parent.mkdir(parents=True, exist_ok=True)
    with output_path.open("w", encoding="utf-8", newline="") as handle:
        writer = csv.DictWriter(
            handle,
            fieldnames=[
                "model",
                "test_loss",
                "test_accuracy",
                "query_accuracy",
                "param_count",
                "effective_state_size",
                "train_seconds",
                "eval_seconds",
                "usage_I",
                "usage_T_b",
                "usage_T_x",
                "usage_T_c",
                "usage_T_y",
                "usage_T_z_prime",
                "usage_T_r",
                "radial_change_fraction",
                "fiber_change_fraction",
                "lawful_spin_change_fraction",
                "illegal_transition_fraction",
                "outside_support_fraction",
            ],
        )
        writer.writeheader()
        for row in rows:
            writer.writerow(row.__dict__)


if __name__ == "__main__":
    metrics = run_operator_weighting_v3(TaskConfigV8(), seed=241)
    write_operator_weighting_v3(metrics)
    print(f"wrote {OUTPUT_PATH_OPERATOR_WEIGHTING_V3}")
    for row in metrics:
        print(
            f"{row.model}: test_accuracy={row.test_accuracy:.4f} "
            f"query_accuracy={row.query_accuracy:.4f} "
            f"test_loss={row.test_loss:.4f}"
        )
