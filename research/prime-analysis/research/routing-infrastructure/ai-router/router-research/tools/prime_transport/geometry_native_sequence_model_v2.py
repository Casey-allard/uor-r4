#!/usr/bin/env python3
"""Second bounded geometry-native sequence-model comparison on a stronger task."""

from __future__ import annotations

from dataclasses import dataclass
from time import perf_counter

import numpy as np
import torch
from torch import nn
from torch.utils.data import DataLoader, TensorDataset


torch.set_num_threads(1)


TOKENS_V2 = [
    "PUSH0",
    "PUSH1",
    "PUSH2",
    "POP",
    "SWAP2",
    "MERGE",
    "QUERY",
]
TOKEN_TO_ID_V2 = {token: idx for idx, token in enumerate(TOKENS_V2)}
EMPTY_CLASS = 3
NUM_CLASSES_V2 = 4  # 0, 1, 2, EMPTY

PUSH_IDS = {
    TOKEN_TO_ID_V2["PUSH0"]: 0,
    TOKEN_TO_ID_V2["PUSH1"]: 1,
    TOKEN_TO_ID_V2["PUSH2"]: 2,
}
QUERY_TOKEN_ID = TOKEN_TO_ID_V2["QUERY"]

TOKEN_DB_V2 = np.array([1, 2, 3, 2, 1, 3, 1], dtype=np.int64)
TOKEN_DPHI_V2 = np.array([0, 1, 2, 1, 2, 0, 1], dtype=np.int64)
TOKEN_DR_V2 = np.array([0, 0, 0, 1, 1, 2, 1], dtype=np.int64)
TOKEN_DGAP_V2 = np.array([1, 2, 1, 2, 3, 1, 2], dtype=np.int64)


@dataclass(frozen=True)
class TaskConfigV2:
    seq_len: int = 32
    train_size: int = 1024
    val_size: int = 256
    test_size: int = 256
    stack_capacity: int = 8


@dataclass
class SequenceMetricsV2:
    model: str
    train_loss: float
    test_loss: float
    test_accuracy: float
    query_accuracy: float
    param_count: int
    effective_state_size: int
    train_seconds: float
    eval_seconds: float


def _initial_geometry_state() -> tuple[int, int, int, int]:
    return 0, 0, 0, 1


def _compute_route_slot(
    *,
    depth: int,
    b: int,
    phi: int,
    r: int,
    next_return_gap: int,
    capacity: int,
) -> int:
    return (7 * depth + 5 * b + 3 * phi + 2 * r + next_return_gap) % capacity


def _push_symbol(stack: list[int], symbol: int, capacity: int) -> None:
    if len(stack) >= capacity:
        del stack[0]
    stack.append(symbol)


def step_stack_world(
    token_id: int,
    *,
    b: int,
    phi: int,
    r: int,
    next_return_gap: int,
    stack: list[int],
    capacity: int,
) -> tuple[int, dict[str, int | list[int]], bool]:
    """Apply one compositional stack operation under geometric routing state."""

    updated_stack = list(stack)
    route_slot = _compute_route_slot(
        depth=len(updated_stack),
        b=b,
        phi=phi,
        r=r,
        next_return_gap=next_return_gap,
        capacity=capacity,
    )
    is_query = token_id == QUERY_TOKEN_ID

    if token_id in PUSH_IDS:
        _push_symbol(updated_stack, PUSH_IDS[token_id], capacity)
    elif token_id == TOKEN_TO_ID_V2["POP"]:
        if updated_stack:
            updated_stack.pop()
    elif token_id == TOKEN_TO_ID_V2["SWAP2"]:
        if len(updated_stack) >= 2:
            updated_stack[-1], updated_stack[-2] = updated_stack[-2], updated_stack[-1]
    elif token_id == TOKEN_TO_ID_V2["MERGE"]:
        if len(updated_stack) >= 2:
            top = updated_stack.pop()
            second = updated_stack.pop()
            merged = (top + 2 * second) % 3
            _push_symbol(updated_stack, merged, capacity)
    elif token_id == QUERY_TOKEN_ID:
        pass
    else:
        raise ValueError(f"unexpected token id: {token_id}")

    target = updated_stack[-1] if updated_stack else EMPTY_CLASS
    top_symbol = updated_stack[-1] if updated_stack else EMPTY_CLASS
    second_symbol = updated_stack[-2] if len(updated_stack) >= 2 else EMPTY_CLASS
    depth_bucket = min(len(updated_stack), capacity)

    snapshot = {
        "b": b,
        "phi": phi,
        "r": r,
        "next_return_gap": next_return_gap,
        "depth_bucket": depth_bucket,
        "top_symbol": top_symbol,
        "second_symbol": second_symbol,
        "route_slot": route_slot,
        "stack": updated_stack,
    }

    next_b = (b + int(TOKEN_DB_V2[token_id])) % 5
    next_phi = (phi + int(TOKEN_DPHI_V2[token_id])) % 3
    next_r = (r + int(TOKEN_DR_V2[token_id])) % 4
    next_gap = 1 + ((next_return_gap - 1 + int(TOKEN_DGAP_V2[token_id])) % 4)
    snapshot["next_b"] = next_b
    snapshot["next_phi"] = next_phi
    snapshot["next_r"] = next_r
    snapshot["next_next_return_gap"] = next_gap
    return int(target), snapshot, is_query


def generate_dataset_v2(
    size: int,
    *,
    seq_len: int,
    capacity: int,
    seed: int,
) -> tuple[np.ndarray, np.ndarray, np.ndarray]:
    rng = np.random.default_rng(seed)
    inputs = np.zeros((size, seq_len), dtype=np.int64)
    targets = np.zeros((size, seq_len), dtype=np.int64)
    query_mask = np.zeros((size, seq_len), dtype=np.float32)

    push_weights = np.array([0.17, 0.17, 0.17], dtype=np.float64)

    for row in range(size):
        b, phi, r, gap = _initial_geometry_state()
        stack: list[int] = []
        for col in range(seq_len):
            if len(stack) == 0:
                choices = [
                    TOKEN_TO_ID_V2["PUSH0"],
                    TOKEN_TO_ID_V2["PUSH1"],
                    TOKEN_TO_ID_V2["PUSH2"],
                    TOKEN_TO_ID_V2["QUERY"],
                ]
                weights = np.array([0.26, 0.26, 0.26, 0.22], dtype=np.float64)
            elif len(stack) == 1:
                choices = [
                    TOKEN_TO_ID_V2["PUSH0"],
                    TOKEN_TO_ID_V2["PUSH1"],
                    TOKEN_TO_ID_V2["PUSH2"],
                    TOKEN_TO_ID_V2["POP"],
                    TOKEN_TO_ID_V2["QUERY"],
                ]
                weights = np.array([0.18, 0.18, 0.18, 0.21, 0.25], dtype=np.float64)
            else:
                choices = list(range(len(TOKENS_V2)))
                weights = np.array([0.14, 0.14, 0.14, 0.17, 0.13, 0.13, 0.15], dtype=np.float64)

            weights = weights / weights.sum()
            token_id = int(rng.choice(choices, p=weights))
            target, snapshot, is_query = step_stack_world(
                token_id,
                b=b,
                phi=phi,
                r=r,
                next_return_gap=gap,
                stack=stack,
                capacity=capacity,
            )
            inputs[row, col] = token_id
            targets[row, col] = target
            query_mask[row, col] = 1.0 if is_query else 0.0
            stack = list(snapshot["stack"])
            b = int(snapshot["next_b"])
            phi = int(snapshot["next_phi"])
            r = int(snapshot["next_r"])
            gap = int(snapshot["next_next_return_gap"])
    return inputs, targets, query_mask


def _one_hot(index: int, size: int) -> np.ndarray:
    vector = np.zeros(size, dtype=np.float32)
    vector[index] = 1.0
    return vector


def _build_feature_vector_v2(token_id: int, snapshot: dict[str, int | list[int]], capacity: int) -> np.ndarray:
    parts: list[np.ndarray] = []
    parts.append(_one_hot(token_id, len(TOKENS_V2)))
    parts.append(_one_hot(int(snapshot["b"]), 5))
    parts.append(_one_hot(int(snapshot["phi"]), 3))
    parts.append(_one_hot(int(snapshot["r"]), 4))
    parts.append(_one_hot(int(snapshot["next_return_gap"]) - 1, 4))
    parts.append(_one_hot(int(snapshot["depth_bucket"]), capacity + 1))
    parts.append(_one_hot(int(snapshot["top_symbol"]), NUM_CLASSES_V2))
    parts.append(_one_hot(int(snapshot["second_symbol"]), NUM_CLASSES_V2))
    parts.append(_one_hot(int(snapshot["route_slot"]), capacity))

    stack = list(snapshot["stack"])
    occupancy = np.zeros(capacity, dtype=np.float32)
    content = np.zeros((capacity, NUM_CLASSES_V2), dtype=np.float32)
    for idx, symbol in enumerate(reversed(stack[-capacity:])):
        occupancy[idx] = 1.0
        content[idx, symbol] = 1.0
    parts.append(occupancy)
    parts.append(content.reshape(-1))

    return np.concatenate(parts, axis=0)


def extract_geometry_features_v2(inputs: np.ndarray, *, capacity: int) -> tuple[np.ndarray, np.ndarray]:
    size, seq_len = inputs.shape
    features: list[np.ndarray] = []
    query_mask = np.zeros((size, seq_len), dtype=np.float32)

    for row in range(size):
        b, phi, r, gap = _initial_geometry_state()
        stack: list[int] = []
        for col in range(seq_len):
            token_id = int(inputs[row, col])
            _, snapshot, is_query = step_stack_world(
                token_id,
                b=b,
                phi=phi,
                r=r,
                next_return_gap=gap,
                stack=stack,
                capacity=capacity,
            )
            features.append(_build_feature_vector_v2(token_id, snapshot, capacity))
            query_mask[row, col] = 1.0 if is_query else 0.0
            stack = list(snapshot["stack"])
            b = int(snapshot["next_b"])
            phi = int(snapshot["next_phi"])
            r = int(snapshot["next_r"])
            gap = int(snapshot["next_next_return_gap"])

    feature_array = np.stack(features, axis=0).reshape(size, seq_len, -1)
    return feature_array, query_mask


class GeometryReadoutV2(nn.Module):
    def __init__(self, input_dim: int, num_classes: int) -> None:
        super().__init__()
        self.net = nn.Sequential(
            nn.Linear(input_dim, 64),
            nn.GELU(),
            nn.Linear(64, 32),
            nn.GELU(),
            nn.Linear(32, num_classes),
        )

    def forward(self, x: torch.Tensor) -> torch.Tensor:
        return self.net(x)


class GeometryNativeSequenceModelV2:
    def __init__(self, input_dim: int, *, seed: int = 0) -> None:
        torch.manual_seed(seed)
        self.readout = GeometryReadoutV2(input_dim, NUM_CLASSES_V2)
        self.loss_fn = nn.CrossEntropyLoss()

    @property
    def param_count(self) -> int:
        return sum(parameter.numel() for parameter in self.readout.parameters())

    @property
    def effective_state_size(self) -> int:
        return 4 + 8  # geometric coordinates plus bounded stack payload

    def fit(
        self,
        train_features: np.ndarray,
        train_targets: np.ndarray,
        *,
        epochs: int = 110,
        learning_rate: float = 1.2e-2,
    ) -> tuple[float, float]:
        x = torch.tensor(train_features.reshape(-1, train_features.shape[-1]), dtype=torch.float32)
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
        features: np.ndarray,
        targets: np.ndarray,
        query_mask: np.ndarray,
    ) -> tuple[float, float, float, float]:
        x = torch.tensor(features.reshape(-1, features.shape[-1]), dtype=torch.float32)
        y = torch.tensor(targets.reshape(-1), dtype=torch.long)
        q = torch.tensor(query_mask.reshape(-1), dtype=torch.float32)

        start = perf_counter()
        with torch.no_grad():
            logits = self.readout(x)
            loss = float(self.loss_fn(logits, y).item())
            predictions = logits.argmax(dim=-1)
            accuracy = float((predictions == y).float().mean().item())
            if float(q.sum().item()) > 0.0:
                query_accuracy = float((((predictions == y).float() * q).sum() / q.sum()).item())
            else:
                query_accuracy = accuracy
        elapsed = perf_counter() - start
        return loss, accuracy, query_accuracy, elapsed


class TinyTransformerBaselineV2(nn.Module):
    def __init__(
        self,
        *,
        vocab_size: int,
        num_classes: int,
        d_model: int = 64,
        nhead: int = 4,
        num_layers: int = 2,
        dim_feedforward: int = 128,
        max_len: int = 48,
    ) -> None:
        super().__init__()
        self.token_embedding = nn.Embedding(vocab_size, d_model)
        self.position_embedding = nn.Embedding(max_len, d_model)
        encoder_layer = nn.TransformerEncoderLayer(
            d_model=d_model,
            nhead=nhead,
            dim_feedforward=dim_feedforward,
            dropout=0.0,
            batch_first=True,
            activation="gelu",
        )
        self.encoder = nn.TransformerEncoder(encoder_layer, num_layers=num_layers)
        self.readout = nn.Linear(d_model, num_classes)

    def forward(self, tokens: torch.Tensor) -> torch.Tensor:
        batch_size, seq_len = tokens.shape
        positions = torch.arange(seq_len, device=tokens.device).unsqueeze(0).expand(batch_size, seq_len)
        hidden = self.token_embedding(tokens) + self.position_embedding(positions)
        mask = torch.triu(torch.full((seq_len, seq_len), float("-inf"), device=tokens.device), diagonal=1)
        encoded = self.encoder(hidden, mask=mask)
        return self.readout(encoded)


class TransformerSequenceTrainerV2:
    def __init__(self, *, seq_len: int, seed: int = 0) -> None:
        torch.manual_seed(seed)
        self.model = TinyTransformerBaselineV2(
            vocab_size=len(TOKENS_V2),
            num_classes=NUM_CLASSES_V2,
            max_len=seq_len,
        )
        self.loss_fn = nn.CrossEntropyLoss()

    @property
    def param_count(self) -> int:
        return sum(parameter.numel() for parameter in self.model.parameters())

    @property
    def effective_state_size(self) -> int:
        return 64

    def fit(
        self,
        train_inputs: np.ndarray,
        train_targets: np.ndarray,
        *,
        epochs: int = 22,
        batch_size: int = 64,
        learning_rate: float = 2.0e-3,
    ) -> tuple[float, float]:
        dataset = TensorDataset(
            torch.tensor(train_inputs, dtype=torch.long),
            torch.tensor(train_targets, dtype=torch.long),
        )
        loader = DataLoader(dataset, batch_size=batch_size, shuffle=True)
        optimizer = torch.optim.Adam(self.model.parameters(), lr=learning_rate)

        start = perf_counter()
        final_loss = 0.0
        for _ in range(epochs):
            for batch_inputs, batch_targets in loader:
                optimizer.zero_grad()
                logits = self.model(batch_inputs)
                loss = self.loss_fn(logits.reshape(-1, logits.shape[-1]), batch_targets.reshape(-1))
                loss.backward()
                optimizer.step()
                final_loss = float(loss.item())
        elapsed = perf_counter() - start
        return final_loss, elapsed

    def evaluate(
        self,
        inputs: np.ndarray,
        targets: np.ndarray,
        query_mask: np.ndarray,
    ) -> tuple[float, float, float, float]:
        x = torch.tensor(inputs, dtype=torch.long)
        y = torch.tensor(targets, dtype=torch.long)
        q = torch.tensor(query_mask, dtype=torch.float32)

        start = perf_counter()
        with torch.no_grad():
            logits = self.model(x)
            loss = float(self.loss_fn(logits.reshape(-1, logits.shape[-1]), y.reshape(-1)).item())
            predictions = logits.argmax(dim=-1)
            accuracy = float((predictions == y).float().mean().item())
            if float(q.sum().item()) > 0.0:
                query_accuracy = float((((predictions == y).float() * q).sum() / q.sum()).item())
            else:
                query_accuracy = accuracy
        elapsed = perf_counter() - start
        return loss, accuracy, query_accuracy, elapsed


def run_bounded_sequence_comparison_v2(config: TaskConfigV2, *, seed: int = 0) -> list[SequenceMetricsV2]:
    train_inputs, train_targets, _ = generate_dataset_v2(
        config.train_size,
        seq_len=config.seq_len,
        capacity=config.stack_capacity,
        seed=seed,
    )
    test_inputs, test_targets, test_query_mask = generate_dataset_v2(
        config.test_size,
        seq_len=config.seq_len,
        capacity=config.stack_capacity,
        seed=seed + 1,
    )

    train_features, _ = extract_geometry_features_v2(train_inputs, capacity=config.stack_capacity)
    test_features, _ = extract_geometry_features_v2(test_inputs, capacity=config.stack_capacity)

    geometry_model = GeometryNativeSequenceModelV2(train_features.shape[-1], seed=seed)
    geometry_train_loss, geometry_train_seconds = geometry_model.fit(train_features, train_targets)
    geometry_test_loss, geometry_test_acc, geometry_query_acc, geometry_eval_seconds = geometry_model.evaluate(
        test_features,
        test_targets,
        test_query_mask,
    )

    transformer = TransformerSequenceTrainerV2(seq_len=config.seq_len, seed=seed)
    transformer_train_loss, transformer_train_seconds = transformer.fit(train_inputs, train_targets)
    transformer_test_loss, transformer_test_acc, transformer_query_acc, transformer_eval_seconds = transformer.evaluate(
        test_inputs,
        test_targets,
        test_query_mask,
    )

    return [
        SequenceMetricsV2(
            model="geometry_native_sequence_model_v2",
            train_loss=geometry_train_loss,
            test_loss=geometry_test_loss,
            test_accuracy=geometry_test_acc,
            query_accuracy=geometry_query_acc,
            param_count=geometry_model.param_count,
            effective_state_size=geometry_model.effective_state_size,
            train_seconds=geometry_train_seconds,
            eval_seconds=geometry_eval_seconds,
        ),
        SequenceMetricsV2(
            model="tiny_transformer_baseline_v2",
            train_loss=transformer_train_loss,
            test_loss=transformer_test_loss,
            test_accuracy=transformer_test_acc,
            query_accuracy=transformer_query_acc,
            param_count=transformer.param_count,
            effective_state_size=transformer.effective_state_size,
            train_seconds=transformer_train_seconds,
            eval_seconds=transformer_eval_seconds,
        ),
    ]
