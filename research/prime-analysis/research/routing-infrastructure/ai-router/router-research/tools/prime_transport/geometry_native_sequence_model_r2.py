#!/usr/bin/env python3
"""Second inner-representation rebuild with native structured readout."""

from __future__ import annotations

from dataclasses import dataclass
from time import perf_counter

import numpy as np
import torch
from torch import nn

from geometry_native_sequence_model_r1 import GeometryNativeSequenceModelR1, step_discourse_world_r1
from geometry_native_sequence_model_v3 import (
    GeometryNativeSequenceModelV3,
    QUERY_IDS_V3,
    TAG_IDS,
    TOKENS_V3,
    TOKEN_TO_ID_V3,
    TransformerSequenceTrainerV3,
    _initial_discourse_state,
    _initial_geometry_state,
    extract_geometry_features_v3,
    generate_dataset_v3,
)


torch.set_num_threads(1)


SEMIPRIMES_R2 = (4, 6, 9, 10, 15, 25)
OVERLAP_VALUES_R2 = (1, 2, 3, 4, 5, 6, 9, 10, 15, 25)
NUM_CLASSES_R2 = 3
ASK_ID_R2 = TOKEN_TO_ID_V3["ASK"]


@dataclass(frozen=True)
class TaskConfigR2:
    seq_len: int = 30
    train_size: int = 1024
    test_size: int = 256


@dataclass
class SequenceMetricsR2:
    model: str
    test_loss: float
    test_accuracy: float
    query_accuracy: float
    param_count: int
    effective_state_size: int
    train_seconds: float
    eval_seconds: float


def _semiprime_idx_r2(value: int) -> int:
    return SEMIPRIMES_R2.index(int(value))


def _overlap_idx_r2(left: int, right: int) -> int:
    overlap = int(np.gcd(int(left), int(right)))
    return OVERLAP_VALUES_R2.index(overlap)


def extract_structured_state_r2(inputs: np.ndarray) -> tuple[dict[str, np.ndarray], np.ndarray]:
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
            _, snapshot, is_query = step_discourse_world_r1(
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
            states["query_semiprime"][row, col] = _semiprime_idx_r2(int(snapshot["query_semiprime"]))
            states["binding_semiprime"][row, col] = _semiprime_idx_r2(int(snapshot["binding_semiprime"]))
            states["overlap"][row, col] = _overlap_idx_r2(
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


class StructuredNativeReadoutR2(nn.Module):
    def __init__(self) -> None:
        super().__init__()
        role_count = 3
        self.token_role = nn.Parameter(torch.zeros(len(TOKENS_V3), role_count))
        self.base_role = nn.Parameter(torch.zeros(5, role_count))
        self.phi_role = nn.Parameter(torch.zeros(3, role_count))
        self.radial_role = nn.Parameter(torch.zeros(4, role_count))
        self.gap_role = nn.Parameter(torch.zeros(4, role_count))
        self.style_role = nn.Parameter(torch.zeros(3, role_count))
        self.query_semiprime_role = nn.Parameter(torch.zeros(len(SEMIPRIMES_R2), role_count))
        self.binding_semiprime_role = nn.Parameter(torch.zeros(len(SEMIPRIMES_R2), role_count))
        self.overlap_role = nn.Parameter(torch.zeros(len(OVERLAP_VALUES_R2), role_count))
        self.admissibility_role = nn.Parameter(torch.zeros(2, role_count))
        self.spin_role = nn.Parameter(torch.zeros(4, 2, role_count))
        self.token_class_bias = nn.Parameter(torch.zeros(len(TOKENS_V3), NUM_CLASSES_R2))

        for parameter in self.parameters():
            nn.init.normal_(parameter, mean=0.0, std=0.05)

    def forward(self, states: dict[str, torch.Tensor]) -> torch.Tensor:
        token_id = states["token_id"]
        role_logits = (
            self.token_role[token_id]
            + self.base_role[states["b"]]
            + self.phi_role[states["phi"]]
            + self.radial_role[states["r"]]
            + self.gap_role[states["gap"]]
            + self.style_role[states["style"]]
            + self.query_semiprime_role[states["query_semiprime"]]
            + self.binding_semiprime_role[states["binding_semiprime"]]
            + self.overlap_role[states["overlap"]]
            + self.admissibility_role[states["admissible"]]
        )

        for bit_idx in range(4):
            role_logits = role_logits + self.spin_role[bit_idx, states["spin_bits"][:, bit_idx]]

        role_entities = torch.stack(
            [states["focus"], states["speaker"], states["topic"]],
            dim=-1,
        )
        role_tags = torch.gather(states["tags"], 1, role_entities)

        entity_logits = []
        tag_logits = []
        for cls in range(NUM_CLASSES_R2):
            entity_mask = role_entities == cls
            tag_mask = role_tags == cls
            entity_logits.append(torch.logsumexp(role_logits.masked_fill(~entity_mask, -30.0), dim=-1))
            tag_logits.append(torch.logsumexp(role_logits.masked_fill(~tag_mask, -30.0), dim=-1))

        entity_logits_tensor = torch.stack(entity_logits, dim=-1)
        tag_logits_tensor = torch.stack(tag_logits, dim=-1)

        ask_mask = token_id == ASK_ID_R2
        logits = torch.where(ask_mask.unsqueeze(-1), tag_logits_tensor, entity_logits_tensor)
        return logits + self.token_class_bias[token_id]


class GeometryNativeSequenceModelR2:
    def __init__(self, *, seed: int = 0) -> None:
        torch.manual_seed(seed)
        self.readout = StructuredNativeReadoutR2()
        self.loss_fn = nn.CrossEntropyLoss()

    @property
    def param_count(self) -> int:
        return sum(parameter.numel() for parameter in self.readout.parameters())

    @property
    def effective_state_size(self) -> int:
        return 16

    def _tensorize_states(self, states: dict[str, np.ndarray]) -> dict[str, torch.Tensor]:
        flat: dict[str, torch.Tensor] = {}
        for key, value in states.items():
            if key == "spin_bits":
                flat[key] = torch.tensor(value.reshape(-1, 4), dtype=torch.long)
            elif key == "tags":
                flat[key] = torch.tensor(value.reshape(-1, 3), dtype=torch.long)
            else:
                flat[key] = torch.tensor(value.reshape(-1), dtype=torch.long)
        return flat

    def fit(
        self,
        train_states: dict[str, np.ndarray],
        train_targets: np.ndarray,
        *,
        epochs: int = 120,
        learning_rate: float = 1.0e-2,
    ) -> tuple[float, float]:
        x = self._tensorize_states(train_states)
        y = torch.tensor(train_targets.reshape(-1), dtype=torch.long)
        optimizer = torch.optim.Adam(self.readout.parameters(), lr=learning_rate)

        start = perf_counter()
        final_loss = 0.0
        for _ in range(epochs):
            optimizer.zero_grad()
            logits = self.readout(x)
            loss = self.loss_fn(logits, y)
            loss.backward()
            optimizer.step()
            final_loss = float(loss.item())
        elapsed = perf_counter() - start
        return final_loss, elapsed

    def evaluate(
        self,
        states: dict[str, np.ndarray],
        targets: np.ndarray,
        query_mask: np.ndarray,
    ) -> tuple[float, float, float, float]:
        x = self._tensorize_states(states)
        y = torch.tensor(targets.reshape(-1), dtype=torch.long)
        q = torch.tensor(query_mask.reshape(-1), dtype=torch.float32)

        start = perf_counter()
        with torch.no_grad():
            logits = self.readout(x)
            loss = float(self.loss_fn(logits, y).item())
            predictions = logits.argmax(dim=-1)
            accuracy = float((predictions == y).float().mean().item())
            query_accuracy = float((((predictions == y).float() * q).sum() / q.sum()).item())
        elapsed = perf_counter() - start
        return loss, accuracy, query_accuracy, elapsed


def run_bounded_sequence_comparison_r2(config: TaskConfigR2, *, seed: int = 0) -> list[SequenceMetricsR2]:
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

    from geometry_native_sequence_model_r1 import extract_geometry_features_r1

    train_features_r1, _ = extract_geometry_features_r1(train_inputs)
    test_features_r1, _ = extract_geometry_features_r1(test_inputs)
    geometry_r1 = GeometryNativeSequenceModelR1(train_features_r1.shape[-1], seed=seed)
    _train_loss_r1, train_seconds_r1 = geometry_r1.fit(train_features_r1, train_targets)
    test_loss_r1, test_acc_r1, query_acc_r1, eval_seconds_r1 = geometry_r1.evaluate(
        test_features_r1,
        test_targets,
        test_query_mask,
    )

    train_states_r2, _ = extract_structured_state_r2(train_inputs)
    test_states_r2, _ = extract_structured_state_r2(test_inputs)
    geometry_r2 = GeometryNativeSequenceModelR2(seed=seed)
    _train_loss_r2, train_seconds_r2 = geometry_r2.fit(train_states_r2, train_targets)
    test_loss_r2, test_acc_r2, query_acc_r2, eval_seconds_r2 = geometry_r2.evaluate(
        test_states_r2,
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
            model="tiny_transformer_baseline_r2",
            test_loss=transformer_test_loss,
            test_accuracy=transformer_test_acc,
            query_accuracy=transformer_query_acc,
            param_count=transformer.param_count,
            effective_state_size=transformer.effective_state_size,
            train_seconds=transformer_train_seconds,
            eval_seconds=transformer_eval_seconds,
        ),
    ]
