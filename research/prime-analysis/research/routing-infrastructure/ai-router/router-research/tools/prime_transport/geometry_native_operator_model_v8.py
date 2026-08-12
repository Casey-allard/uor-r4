#!/usr/bin/env python3
"""Learned weighting over lawful H_v6 operator entries only."""

from __future__ import annotations

import csv
import math
from dataclasses import dataclass
from pathlib import Path
from time import perf_counter

import numpy as np

from geometry_native_operator_model_v4 import OperatorStateV4, class_tuple_v4, initial_operator_state_v4
from geometry_native_operator_model_v7 import fiber_phase_lift_component_v6
from geometry_native_operator_model_v4 import (
    composite_swap_component_v4,
    composite_twist_component_v4,
    coupled_torus_kick_component_v4,
    hold_component_v4,
    torus_base_advance_component_v4,
)
from geometry_native_sequence_model_r3 import _role_from_native_state_r3
from geometry_native_sequence_model_v3 import (
    QUERY_IDS_V3,
    TAG_IDS,
    TOKEN_DGAP_V3,
    TOKEN_TO_ID_V3,
    _initial_discourse_state,
    _role_entity,
    generate_dataset_v3,
)


OUTPUT_PATH_OPERATOR_WEIGHTING_V2 = Path(
    "/Users/adminamn/AI-Research/ai-router/router-research/results/"
    "prime_transport_recursive_system/prime_transport_operator_weighting_v2.csv"
)
R5_CSV = Path(
    "/Users/adminamn/AI-Research/ai-router/router-research/results/"
    "prime_transport_recursive_system/prime_transport_inner_representation_rebuild_v5.csv"
)
V1_CSV = Path(
    "/Users/adminamn/AI-Research/ai-router/router-research/results/"
    "prime_transport_recursive_system/prime_transport_operator_weighting_v1.csv"
)


@dataclass(frozen=True)
class TaskConfigV8:
    seq_len: int = 30
    train_size: int = 1024
    test_size: int = 256


@dataclass
class SequenceMetricsV8:
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
    original_class_fraction: float
    lifted_class_fraction: float
    lawful_spin_change_fraction: float
    illegal_transition_fraction: float
    outside_support_fraction: float


@dataclass(frozen=True)
class RuntimeDiscourseStateV8:
    focus: int
    speaker: int
    topic: int
    style: int
    next_return_gap: int
    tags: tuple[int, int, int]


def _generator_apply_v8(state: OperatorStateV4, generator_index: int) -> OperatorStateV4:
    if generator_index == 0:
        return hold_component_v4(state)
    if generator_index == 1:
        return torus_base_advance_component_v4(state)
    if generator_index == 2:
        return composite_swap_component_v4(state)
    if generator_index == 3:
        return coupled_torus_kick_component_v4(state)
    if generator_index == 4:
        return composite_twist_component_v4(state)
    if generator_index == 5:
        return fiber_phase_lift_component_v6(state)
    raise ValueError(f"unknown generator index {generator_index}")


def _update_discourse_v8(
    token_id: int,
    *,
    operator_state: OperatorStateV4,
    discourse_state: RuntimeDiscourseStateV8,
) -> RuntimeDiscourseStateV8:
    focus = discourse_state.focus
    speaker = discourse_state.speaker
    topic = discourse_state.topic
    style = discourse_state.style
    tags = list(discourse_state.tags)

    current_overlap = math.gcd(operator_state.query_semiprime, operator_state.binding_semiprime)
    current_role = _role_from_native_state_r3(
        style=style,
        phi=operator_state.phi,
        r=operator_state.r,
        next_return_gap=discourse_state.next_return_gap,
        query_semiprime=operator_state.query_semiprime,
        overlap=current_overlap,
    )
    current_entity = _role_entity(current_role, focus=focus, speaker=speaker, topic=topic)

    if token_id == TOKEN_TO_ID_V3["MENTION0"]:
        focus = 0
    elif token_id == TOKEN_TO_ID_V3["MENTION1"]:
        focus = 1
    elif token_id == TOKEN_TO_ID_V3["MENTION2"]:
        focus = 2
    elif token_id == TOKEN_TO_ID_V3["SET_SPEAKER"]:
        speaker = focus
    elif token_id == TOKEN_TO_ID_V3["SET_TOPIC"]:
        topic = focus
    elif token_id == TOKEN_TO_ID_V3["SHIFT_STYLE"]:
        style = (style + 1) % 3
    elif token_id in TAG_IDS:
        tags[current_entity] = TAG_IDS[token_id]

    next_return_gap = 1 + ((discourse_state.next_return_gap - 1 + int(TOKEN_DGAP_V3[token_id])) % 4)
    return RuntimeDiscourseStateV8(
        focus=focus,
        speaker=speaker,
        topic=topic,
        style=style,
        next_return_gap=next_return_gap,
        tags=(int(tags[0]), int(tags[1]), int(tags[2])),
    )


def _predicted_class_v8(
    token_id: int,
    *,
    operator_state: OperatorStateV4,
    discourse_state: RuntimeDiscourseStateV8,
) -> int:
    overlap = math.gcd(operator_state.query_semiprime, operator_state.binding_semiprime)
    role = _role_from_native_state_r3(
        style=discourse_state.style,
        phi=operator_state.phi,
        r=operator_state.r,
        next_return_gap=discourse_state.next_return_gap,
        query_semiprime=operator_state.query_semiprime,
        overlap=overlap,
    )
    entity = _role_entity(role, focus=discourse_state.focus, speaker=discourse_state.speaker, topic=discourse_state.topic)
    return discourse_state.tags[entity] if token_id == TOKEN_TO_ID_V3["ASK"] else entity


def _context_indices_v8(
    token_id: int,
    *,
    operator_state: OperatorStateV4,
    discourse_state: RuntimeDiscourseStateV8,
) -> tuple[int, ...]:
    query_idx = 0 if operator_state.query_semiprime == 6 else 1
    binding_idx = 0 if operator_state.binding_semiprime == 6 else 1
    query_flag = int(token_id in QUERY_IDS_V3)
    tag_flag = int(token_id in TAG_IDS)
    compat_idx = int("".join(operator_state.composite_compat_class) != "100100")
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
    )


class StructuredLawfulOperatorSelectorV8:
    """Structured additive selector over the six lawful H_v6 generators."""

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
                operator_state = initial_operator_state_v4()
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
                    context = _context_indices_v8(
                        token_id,
                        operator_state=operator_state,
                        discourse_state=discourse_state,
                    )
                    scores = self.scores(context)

                    predicted_classes = []
                    next_operator_states = []
                    next_discourse_states = []
                    for generator_index in range(6):
                        next_operator = _generator_apply_v8(operator_state, generator_index)
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


def _class_probabilities_v8(scores: np.ndarray, predicted_classes: list[int]) -> np.ndarray:
    class_logits = np.full(3, -1.0e18, dtype=np.float64)
    for cls in range(3):
        cls_scores = [score for score, pred in zip(scores, predicted_classes) if pred == cls]
        if cls_scores:
            max_score = max(cls_scores)
            class_logits[cls] = max_score + math.log(sum(math.exp(score - max_score) for score in cls_scores))
    finite_mask = np.isfinite(class_logits)
    max_logit = np.max(class_logits[finite_mask])
    exp_logits = np.zeros(3, dtype=np.float64)
    exp_logits[finite_mask] = np.exp(class_logits[finite_mask] - max_logit)
    return exp_logits / exp_logits.sum()


def _evaluate_policy_v8(
    inputs: np.ndarray,
    targets: np.ndarray,
    query_mask: np.ndarray,
    *,
    policy_name: str,
    selector: StructuredLawfulOperatorSelectorV8 | None,
    fixed_policy: str | None,
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
    original_class = class_tuple_v4(initial_operator_state_v4())
    original_class_hits = 0
    lifted_class_hits = 0
    lawful_spin_change_hits = 0

    for row in range(inputs.shape[0]):
        operator_state = initial_operator_state_v4()
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
            context = _context_indices_v8(
                token_id,
                operator_state=operator_state,
                discourse_state=discourse_state,
            )

            predicted_classes = []
            next_operator_states = []
            next_discourse_states = []
            for generator_index in range(6):
                next_operator = _generator_apply_v8(operator_state, generator_index)
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

            if selector is not None:
                scores = selector.scores(context)
                generator_index = int(np.argmax(scores))
                probs = _class_probabilities_v8(scores, predicted_classes)
            else:
                assert fixed_policy is not None
                if fixed_policy == "token_cycle":
                    generator_index = token_id % 6
                else:
                    raise ValueError(f"unknown fixed policy {fixed_policy}")
                probs = np.full(3, 1.0e-6, dtype=np.float64)
                probs[predicted_classes[generator_index]] = 1.0 - 2.0e-6

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

            if class_tuple_v4(next_operator) == original_class:
                original_class_hits += 1
            else:
                lifted_class_hits += 1
            if next_operator.spin_h != operator_state.spin_h:
                lawful_spin_change_hits += 1

            operator_state = next_operator
            discourse_state = next_discourse

    elapsed = perf_counter() - start
    total_usage = usage.sum()
    return SequenceMetricsV8(
        model=policy_name,
        test_loss=total_loss / max(total_count, 1),
        test_accuracy=correct / max(total_count, 1),
        query_accuracy=query_correct / max(query_total, 1),
        param_count=param_count,
        effective_state_size=9,
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


def _read_reference_metric_v8(csv_path: Path, model_name: str) -> SequenceMetricsV8:
    with csv_path.open("r", encoding="utf-8") as handle:
        for row in csv.DictReader(handle):
            if row["model"] == model_name:
                usage_tz = float(row.get("usage_T_z_prime", row.get("usage_T_z", 0.0)))
                lawful_spin = float(row.get("lawful_spin_change_fraction", 0.0))
                return SequenceMetricsV8(
                    model=row["model"],
                    test_loss=float(row["test_loss"]),
                    test_accuracy=float(row["test_accuracy"]),
                    query_accuracy=float(row["query_accuracy"]),
                    param_count=int(row["param_count"]),
                    effective_state_size=int(row["effective_state_size"]),
                    train_seconds=float(row["train_seconds"]),
                    eval_seconds=float(row["eval_seconds"]),
                    usage_I=float(row.get("usage_I", 0.0)),
                    usage_T_b=float(row.get("usage_T_b", 0.0)),
                    usage_T_x=float(row.get("usage_T_x", 0.0)),
                    usage_T_c=float(row.get("usage_T_c", 0.0)),
                    usage_T_y=float(row.get("usage_T_y", 0.0)),
                    usage_T_z_prime=usage_tz,
                    original_class_fraction=float(row.get("original_class_fraction", 0.0)),
                    lifted_class_fraction=float(row.get("lifted_class_fraction", 0.0)),
                    lawful_spin_change_fraction=lawful_spin,
                    illegal_transition_fraction=float(row.get("illegal_transition_fraction", 0.0)),
                    outside_support_fraction=float(row.get("outside_support_fraction", 0.0)),
                )
    raise ValueError(f"{model_name} not found in {csv_path}")


def run_operator_weighting_v2(config: TaskConfigV8, *, seed: int = 241) -> list[SequenceMetricsV8]:
    train_inputs, train_targets, _ = generate_dataset_v3(config.train_size, seq_len=config.seq_len, seed=seed)
    test_inputs, test_targets, test_query_mask = generate_dataset_v3(config.test_size, seq_len=config.seq_len, seed=seed + 1)

    selector = StructuredLawfulOperatorSelectorV8()
    train_seconds = selector.fit(train_inputs, train_targets)

    h_v6_fixed = _evaluate_policy_v8(
        test_inputs,
        test_targets,
        test_query_mask,
        policy_name="geometry_native_operator_model_v7_fixed_policy",
        selector=None,
        fixed_policy="token_cycle",
        param_count=0,
        train_seconds=0.0,
    )
    v8_learned = _evaluate_policy_v8(
        test_inputs,
        test_targets,
        test_query_mask,
        policy_name="geometry_native_operator_model_v8",
        selector=selector,
        fixed_policy=None,
        param_count=selector.param_count,
        train_seconds=train_seconds,
    )

    v3_ref = _read_reference_metric_v8(R5_CSV, "geometry_native_sequence_model_v3_reference")
    r5_ref = _read_reference_metric_v8(R5_CSV, "geometry_native_sequence_model_r5")
    v6_ref = _read_reference_metric_v8(V1_CSV, "geometry_native_operator_model_v6")
    transformer_ref = _read_reference_metric_v8(R5_CSV, "tiny_transformer_baseline_r5")
    transformer_ref.model = "tiny_transformer_baseline_v8_reference"
    return [v3_ref, r5_ref, v6_ref, h_v6_fixed, v8_learned, transformer_ref]


def write_operator_weighting_v2(
    rows: list[SequenceMetricsV8],
    output_path: Path = OUTPUT_PATH_OPERATOR_WEIGHTING_V2,
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
    metrics = run_operator_weighting_v2(TaskConfigV8(), seed=241)
    write_operator_weighting_v2(metrics)
    print(f"wrote {OUTPUT_PATH_OPERATOR_WEIGHTING_V2}")
    for row in metrics:
        print(
            f"{row.model}: test_accuracy={row.test_accuracy:.4f} "
            f"query_accuracy={row.query_accuracy:.4f} "
            f"test_loss={row.test_loss:.4f}"
        )
