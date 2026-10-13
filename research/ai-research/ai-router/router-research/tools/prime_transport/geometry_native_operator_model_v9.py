#!/usr/bin/env python3
"""Learned weighting over lawful operator support using active transport state with tau."""

from __future__ import annotations

import csv
import math
from dataclasses import dataclass
from pathlib import Path
from time import perf_counter

import numpy as np

from geometry_native_operator_model_v4 import initial_operator_state_v4
from geometry_native_operator_model_v8 import (
    OUTPUT_PATH_OPERATOR_WEIGHTING_V2,
    R5_CSV,
    SequenceMetricsV8,
    TaskConfigV8,
    V1_CSV,
    _class_probabilities_v8,
    _predicted_class_v8,
    _read_reference_metric_v8,
    _update_discourse_v8,
)
from geometry_native_sequence_model_v3 import QUERY_IDS_V3, TAG_IDS, _initial_discourse_state, generate_dataset_v3
from geometry_native_spinH_candidate_v3 import (
    AugmentedOperatorStateV3,
    OUTPUT_PATH_SPINH_CANDIDATE_V3,
    active_transport_lift_v3,
    initial_tau_v3,
    update_tau_v3,
)
from geometry_native_operator_model_v8 import _generator_apply_v8


OUTPUT_PATH_OPERATOR_WITH_TAU_V1 = Path(
    "/Users/adminamn/AI-Research/ai-router/router-research/results/"
    "prime_transport_recursive_system/prime_transport_operator_with_tau_v1.csv"
)
V2_OPERATOR_CSV = Path(
    "/Users/adminamn/AI-Research/ai-router/router-research/results/"
    "prime_transport_recursive_system/prime_transport_operator_weighting_v2.csv"
)


def _augmented_context_indices_v9(
    token_id: int,
    *,
    augmented_state: AugmentedOperatorStateV3,
    discourse_state: object,
) -> tuple[int, ...]:
    operator_state = augmented_state.operator_state
    transport_state = active_transport_lift_v3(operator_state, augmented_state.tau)
    compat_idx = int("".join(operator_state.composite_compat_class) != "100100")
    query_idx = 0 if operator_state.query_semiprime == 6 else 1
    binding_idx = 0 if operator_state.binding_semiprime == 6 else 1
    query_flag = int(token_id in QUERY_IDS_V3)
    tag_flag = int(token_id in TAG_IDS)
    return (
        token_id,
        operator_state.b,
        operator_state.phi,
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


def _generator_apply_v9(augmented_state: AugmentedOperatorStateV3, generator_index: int) -> AugmentedOperatorStateV3:
    source_operator = augmented_state.operator_state
    if generator_index == 0:
        component = "hold"
    elif generator_index == 1:
        component = "torus_base_advance"
    elif generator_index == 2:
        component = "composite_swap"
    elif generator_index == 3:
        component = "coupled_torus_kick"
    elif generator_index == 4:
        component = "composite_twist"
    elif generator_index == 5:
        component = "fiber_phase_lift_spin_transport"
    else:
        raise ValueError(f"unknown generator index {generator_index}")
    target_operator = _generator_apply_v8(source_operator, generator_index)
    target_tau = update_tau_v3(source_operator, component, augmented_state.tau)
    return AugmentedOperatorStateV3(operator_state=target_operator, tau=target_tau)


class StructuredLawfulOperatorSelectorV9:
    """Structured additive selector over lawful operator edges with tau-aware context."""

    def __init__(self) -> None:
        self.base = np.zeros(6, dtype=np.float64)
        self.token = np.zeros((11, 6), dtype=np.float64)
        self.b = np.zeros((5, 6), dtype=np.float64)
        self.phi = np.zeros((3, 6), dtype=np.float64)
        self.twist = np.zeros((2, 6), dtype=np.float64)
        self.query_semiprime = np.zeros((2, 6), dtype=np.float64)
        self.binding_semiprime = np.zeros((2, 6), dtype=np.float64)
        self.style = np.zeros((3, 6), dtype=np.float64)
        self.gap = np.zeros((4, 6), dtype=np.float64)
        self.focus = np.zeros((3, 6), dtype=np.float64)
        self.speaker = np.zeros((3, 6), dtype=np.float64)
        self.topic = np.zeros((3, 6), dtype=np.float64)
        self.query_flag = np.zeros((2, 6), dtype=np.float64)
        self.tag_flag = np.zeros((2, 6), dtype=np.float64)
        self.spin_bit0 = np.zeros((2, 6), dtype=np.float64)
        self.spin_bit1 = np.zeros((2, 6), dtype=np.float64)
        self.spin_bit2 = np.zeros((2, 6), dtype=np.float64)
        self.spin_bit3 = np.zeros((2, 6), dtype=np.float64)
        self.compat = np.zeros((2, 6), dtype=np.float64)
        self.tau_swap = np.zeros((2, 6), dtype=np.float64)
        self.tau_coupled = np.zeros((5, 6), dtype=np.float64)
        self.tau_twist = np.zeros((2, 6), dtype=np.float64)
        self.tau_lift = np.zeros((12, 6), dtype=np.float64)

    @property
    def param_count(self) -> int:
        return sum(
            table.size
            for table in (
                self.base,
                self.token,
                self.b,
                self.phi,
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
                augmented_state = AugmentedOperatorStateV3(
                    operator_state=initial_operator_state_v4(),
                    tau=initial_tau_v3(),
                )
                focus, speaker, topic, style, gap, tags = (0, 1, 2, 0, 1, (0, 1, 2))
                discourse_state = type(
                    "RuntimeDiscourseStateV9",
                    (),
                    {
                        "focus": int(focus),
                        "speaker": int(speaker),
                        "topic": int(topic),
                        "style": int(style),
                        "next_return_gap": int(gap),
                        "tags": (int(tags[0]), int(tags[1]), int(tags[2])),
                    },
                )()
                for col in range(train_inputs.shape[1]):
                    token_id = int(train_inputs[row, col])
                    target = int(train_targets[row, col])
                    context = _augmented_context_indices_v9(
                        token_id,
                        augmented_state=augmented_state,
                        discourse_state=discourse_state,
                    )
                    scores = self.scores(context)

                    predicted_classes = []
                    next_augmented_states = []
                    next_discourse_states = []
                    for generator_index in range(6):
                        next_augmented = _generator_apply_v9(augmented_state, generator_index)
                        next_discourse = _update_discourse_v8(
                            token_id,
                            operator_state=next_augmented.operator_state,
                            discourse_state=discourse_state,
                        )
                        pred_class = _predicted_class_v8(
                            token_id,
                            operator_state=next_augmented.operator_state,
                            discourse_state=next_discourse,
                        )
                        next_augmented_states.append(next_augmented)
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

                    augmented_state = next_augmented_states[chosen]
                    discourse_state = next_discourse_states[chosen]
        return perf_counter() - start


def _initial_discourse_runtime_v9():
    focus, speaker, topic, style, gap, tags = _initial_discourse_state()
    return type(
        "RuntimeDiscourseStateV9",
        (),
        {
            "focus": int(focus),
            "speaker": int(speaker),
            "topic": int(topic),
            "style": int(style),
            "next_return_gap": int(gap),
            "tags": (int(tags[0]), int(tags[1]), int(tags[2])),
        },
    )()


def _evaluate_policy_v9(
    inputs: np.ndarray,
    targets: np.ndarray,
    query_mask: np.ndarray,
    *,
    policy_name: str,
    selector: StructuredLawfulOperatorSelectorV9,
    param_count: int,
    train_seconds: float,
) -> SequenceMetricsV8:
    start = perf_counter()
    total_loss = 0.0
    total_count = 0
    correct = 0
    query_correct = 0
    query_total = 0
    usage = np.zeros(6, dtype=np.float64)
    initial_augmented = AugmentedOperatorStateV3(operator_state=initial_operator_state_v4(), tau=initial_tau_v3())
    original_transport = active_transport_lift_v3(initial_augmented.operator_state, initial_augmented.tau)
    original_class_hits = 0
    lifted_class_hits = 0
    lawful_spin_change_hits = 0

    for row in range(inputs.shape[0]):
        augmented_state = AugmentedOperatorStateV3(
            operator_state=initial_operator_state_v4(),
            tau=initial_tau_v3(),
        )
        discourse_state = _initial_discourse_runtime_v9()
        for col in range(inputs.shape[1]):
            token_id = int(inputs[row, col])
            target = int(targets[row, col])
            context = _augmented_context_indices_v9(
                token_id,
                augmented_state=augmented_state,
                discourse_state=discourse_state,
            )

            predicted_classes = []
            next_augmented_states = []
            next_discourse_states = []
            for generator_index in range(6):
                next_augmented = _generator_apply_v9(augmented_state, generator_index)
                next_discourse = _update_discourse_v8(
                    token_id,
                    operator_state=next_augmented.operator_state,
                    discourse_state=discourse_state,
                )
                pred_class = _predicted_class_v8(
                    token_id,
                    operator_state=next_augmented.operator_state,
                    discourse_state=next_discourse,
                )
                next_augmented_states.append(next_augmented)
                next_discourse_states.append(next_discourse)
                predicted_classes.append(pred_class)

            scores = selector.scores(context)
            generator_index = int(np.argmax(scores))
            probs = _class_probabilities_v8(scores, predicted_classes)

            usage[generator_index] += 1.0
            next_augmented = next_augmented_states[generator_index]
            next_discourse = next_discourse_states[generator_index]
            pred_class = predicted_classes[generator_index]
            total_loss += -math.log(max(probs[target], 1.0e-12))
            correct += int(pred_class == target)
            total_count += 1
            if query_mask[row, col] > 0.5:
                query_correct += int(pred_class == target)
                query_total += 1

            next_transport = active_transport_lift_v3(next_augmented.operator_state, next_augmented.tau)
            if next_transport == original_transport:
                original_class_hits += 1
            else:
                lifted_class_hits += 1
            if next_augmented.operator_state.spin_h != augmented_state.operator_state.spin_h:
                lawful_spin_change_hits += 1

            augmented_state = next_augmented
            discourse_state = next_discourse

    elapsed = perf_counter() - start
    total_usage = usage.sum()
    return SequenceMetricsV8(
        model=policy_name,
        test_loss=total_loss / max(total_count, 1),
        test_accuracy=correct / max(total_count, 1),
        query_accuracy=query_correct / max(query_total, 1),
        param_count=param_count,
        effective_state_size=13,
        train_seconds=train_seconds,
        eval_seconds=elapsed,
        usage_I=usage[0] / max(total_usage, 1.0),
        usage_T_b=usage[1] / max(total_usage, 1.0),
        usage_T_x=usage[2] / max(total_usage, 1.0),
        usage_T_c=usage[3] / max(total_usage, 1.0),
        usage_T_y=usage[4] / max(total_usage, 1.0),
        usage_T_z_prime=usage[5] / max(total_usage, 1.0),
        original_class_fraction=original_class_hits / max(total_usage, 1.0),
        lifted_class_fraction=lifted_class_hits / max(total_usage, 1.0),
        lawful_spin_change_fraction=lawful_spin_change_hits / max(total_usage, 1.0),
        illegal_transition_fraction=0.0,
        outside_support_fraction=0.0,
    )


def run_operator_with_tau_v1(config: TaskConfigV8, *, seed: int = 241) -> list[SequenceMetricsV8]:
    train_inputs, train_targets, _ = generate_dataset_v3(config.train_size, seq_len=config.seq_len, seed=seed)
    test_inputs, test_targets, test_query_mask = generate_dataset_v3(config.test_size, seq_len=config.seq_len, seed=seed + 1)

    selector = StructuredLawfulOperatorSelectorV9()
    train_seconds = selector.fit(train_inputs, train_targets)
    v9_learned = _evaluate_policy_v9(
        test_inputs,
        test_targets,
        test_query_mask,
        policy_name="geometry_native_operator_model_v9",
        selector=selector,
        param_count=selector.param_count,
        train_seconds=train_seconds,
    )

    v3_ref = _read_reference_metric_v8(R5_CSV, "geometry_native_sequence_model_v3_reference")
    r5_ref = _read_reference_metric_v8(R5_CSV, "geometry_native_sequence_model_r5")
    v8_ref = _read_reference_metric_v8(V2_OPERATOR_CSV, "geometry_native_operator_model_v8")
    transformer_ref = _read_reference_metric_v8(R5_CSV, "tiny_transformer_baseline_r5")
    transformer_ref.model = "tiny_transformer_baseline_v9_reference"
    return [v3_ref, r5_ref, v8_ref, v9_learned, transformer_ref]


def write_operator_with_tau_v1(
    rows: list[SequenceMetricsV8],
    output_path: Path = OUTPUT_PATH_OPERATOR_WITH_TAU_V1,
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
                "original_class_fraction",
                "lifted_class_fraction",
                "lawful_spin_change_fraction",
                "illegal_transition_fraction",
                "outside_support_fraction",
            ],
        )
        writer.writeheader()
        for row in rows:
            writer.writerow(row.__dict__)


if __name__ == "__main__":
    metrics = run_operator_with_tau_v1(TaskConfigV8(), seed=241)
    write_operator_with_tau_v1(metrics)
    print(f"wrote {OUTPUT_PATH_OPERATOR_WITH_TAU_V1}")
    for row in metrics:
        print(
            f"{row.model}: test_accuracy={row.test_accuracy:.4f} "
            f"query_accuracy={row.query_accuracy:.4f} "
            f"test_loss={row.test_loss:.4f}"
        )
