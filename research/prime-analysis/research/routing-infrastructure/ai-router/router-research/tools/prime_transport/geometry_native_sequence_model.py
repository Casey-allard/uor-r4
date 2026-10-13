#!/usr/bin/env python3
"""Bounded geometry-native sequence model and tiny transformer baseline."""

from __future__ import annotations

from dataclasses import dataclass
from time import perf_counter

import numpy as np
import torch
from torch import nn
from torch.utils.data import DataLoader, TensorDataset


torch.set_num_threads(1)


TOKENS = [
    "WRITE0",
    "WRITE1",
    "WRITE2",
    "INC",
    "SWAP",
    "QUERY",
    "SUM",
]
TOKEN_TO_ID = {token: idx for idx, token in enumerate(TOKENS)}
OUTPUT_LABELS = ["0", "1", "2"]

WRITE_IDS = {
    TOKEN_TO_ID["WRITE0"]: 0,
    TOKEN_TO_ID["WRITE1"]: 1,
    TOKEN_TO_ID["WRITE2"]: 2,
}
QUERY_IDS = {TOKEN_TO_ID["QUERY"], TOKEN_TO_ID["SUM"]}

# Token-conditioned geometric transport.
TOKEN_DB = np.array([1, 2, 3, 1, 2, 1, 4], dtype=np.int64)
TOKEN_DPHI = np.array([0, 1, 2, 1, 0, 2, 1], dtype=np.int64)
TOKEN_DR = np.array([0, 0, 0, 1, 1, 0, 1], dtype=np.int64)
TOKEN_DGAP = np.array([1, 2, 1, 2, 1, 1, 2], dtype=np.int64)


@dataclass(frozen=True)
class TaskConfig:
    seq_len: int = 24
    train_size: int = 768
    val_size: int = 192
    test_size: int = 192


@dataclass
class GeometrySequenceMetrics:
    model: str
    train_loss: float
    test_loss: float
    test_accuracy: float
    query_accuracy: float
    param_count: int
    effective_state_size: int
    train_seconds: float
    eval_seconds: float


def _initial_world_state() -> tuple[int, int, int, int, list[int]]:
    return 0, 0, 0, 1, [0, 0, 0]


def _active_entity(b: int, phi: int) -> int:
    return (b + phi) % 3


def _secondary_entity(b: int, r: int, next_return_gap: int) -> int:
    return (b + r + next_return_gap) % 3


def _attractor_index(active: int, secondary: int, next_return_gap: int) -> int:
    return ((active + secondary) % 3) * 3 + (next_return_gap - 1)


def _post_update_signature(values: list[int], b: int, phi: int, r: int, gap: int) -> int:
    return (values[0] + 2 * values[1] + values[2] + b + phi + r + gap) % 3


def step_geometry_world(
    token_id: int,
    *,
    b: int,
    phi: int,
    r: int,
    next_return_gap: int,
    values: list[int],
) -> tuple[int, dict[str, int | list[int]], bool]:
    """Apply one token to the geometry-native world and emit the target label."""

    active = _active_entity(b, phi)
    secondary = _secondary_entity(b, r, next_return_gap)
    updated_values = list(values)
    is_query = token_id in QUERY_IDS

    if token_id in WRITE_IDS:
        updated_values[active] = WRITE_IDS[token_id]
        target = _post_update_signature(updated_values, b, phi, r, next_return_gap)
    elif token_id == TOKEN_TO_ID["INC"]:
        updated_values[active] = (updated_values[active] + 1) % 3
        target = _post_update_signature(updated_values, b, phi, r, next_return_gap)
    elif token_id == TOKEN_TO_ID["SWAP"]:
        updated_values[active], updated_values[secondary] = (
            updated_values[secondary],
            updated_values[active],
        )
        target = _post_update_signature(updated_values, b, phi, r, next_return_gap)
    elif token_id == TOKEN_TO_ID["QUERY"]:
        target = updated_values[active]
    elif token_id == TOKEN_TO_ID["SUM"]:
        target = (updated_values[active] + updated_values[secondary]) % 3
    else:
        raise ValueError(f"unexpected token id: {token_id}")

    snapshot = {
        "b": b,
        "phi": phi,
        "r": r,
        "next_return_gap": next_return_gap,
        "active": active,
        "secondary": secondary,
        "values": updated_values,
        "attractor_index": _attractor_index(active, secondary, next_return_gap),
    }

    next_b = (b + int(TOKEN_DB[token_id])) % 5
    next_phi = (phi + int(TOKEN_DPHI[token_id])) % 3
    next_r = (r + int(TOKEN_DR[token_id])) % 3
    next_gap = 1 + ((next_return_gap - 1 + int(TOKEN_DGAP[token_id])) % 3)
    snapshot["next_b"] = next_b
    snapshot["next_phi"] = next_phi
    snapshot["next_r"] = next_r
    snapshot["next_next_return_gap"] = next_gap
    return int(target), snapshot, is_query


def generate_dataset(size: int, *, seq_len: int, seed: int) -> tuple[np.ndarray, np.ndarray, np.ndarray]:
    rng = np.random.default_rng(seed)
    inputs = np.zeros((size, seq_len), dtype=np.int64)
    targets = np.zeros((size, seq_len), dtype=np.int64)
    query_mask = np.zeros((size, seq_len), dtype=np.float32)

    token_weights = np.array([0.11, 0.11, 0.11, 0.19, 0.16, 0.17, 0.15], dtype=np.float64)
    token_weights /= token_weights.sum()

    for row in range(size):
        b, phi, r, gap, values = _initial_world_state()
        for col in range(seq_len):
            token_id = int(rng.choice(len(TOKENS), p=token_weights))
            target, snapshot, is_query = step_geometry_world(
                token_id,
                b=b,
                phi=phi,
                r=r,
                next_return_gap=gap,
                values=values,
            )
            inputs[row, col] = token_id
            targets[row, col] = target
            query_mask[row, col] = 1.0 if is_query else 0.0
            values = list(snapshot["values"])
            b = int(snapshot["next_b"])
            phi = int(snapshot["next_phi"])
            r = int(snapshot["next_r"])
            gap = int(snapshot["next_next_return_gap"])
    return inputs, targets, query_mask


def _build_feature_vector(token_id: int, snapshot: dict[str, int | list[int]]) -> np.ndarray:
    parts: list[np.ndarray] = []

    token_one_hot = np.zeros(len(TOKENS), dtype=np.float32)
    token_one_hot[token_id] = 1.0
    parts.append(token_one_hot)

    active_one_hot = np.zeros(3, dtype=np.float32)
    active_one_hot[int(snapshot["active"])] = 1.0
    parts.append(active_one_hot)

    secondary_one_hot = np.zeros(3, dtype=np.float32)
    secondary_one_hot[int(snapshot["secondary"])] = 1.0
    parts.append(secondary_one_hot)

    values = list(snapshot["values"])
    for value in values:
        value_one_hot = np.zeros(3, dtype=np.float32)
        value_one_hot[int(value)] = 1.0
        parts.append(value_one_hot)

    b_one_hot = np.zeros(5, dtype=np.float32)
    b_one_hot[int(snapshot["b"])] = 1.0
    parts.append(b_one_hot)

    phi_one_hot = np.zeros(3, dtype=np.float32)
    phi_one_hot[int(snapshot["phi"])] = 1.0
    parts.append(phi_one_hot)

    r_one_hot = np.zeros(3, dtype=np.float32)
    r_one_hot[int(snapshot["r"])] = 1.0
    parts.append(r_one_hot)

    gap_one_hot = np.zeros(3, dtype=np.float32)
    gap_one_hot[int(snapshot["next_return_gap"]) - 1] = 1.0
    parts.append(gap_one_hot)

    attractor_one_hot = np.zeros(9, dtype=np.float32)
    attractor_one_hot[int(snapshot["attractor_index"])] = 1.0
    parts.append(attractor_one_hot)

    return np.concatenate(parts, axis=0)


def extract_geometry_features(inputs: np.ndarray) -> tuple[np.ndarray, np.ndarray]:
    size, seq_len = inputs.shape
    features: list[np.ndarray] = []
    query_mask = np.zeros((size, seq_len), dtype=np.float32)

    for row in range(size):
        b, phi, r, gap, values = _initial_world_state()
        for col in range(seq_len):
            token_id = int(inputs[row, col])
            _, snapshot, is_query = step_geometry_world(
                token_id,
                b=b,
                phi=phi,
                r=r,
                next_return_gap=gap,
                values=values,
            )
            features.append(_build_feature_vector(token_id, snapshot))
            query_mask[row, col] = 1.0 if is_query else 0.0
            values = list(snapshot["values"])
            b = int(snapshot["next_b"])
            phi = int(snapshot["next_phi"])
            r = int(snapshot["next_r"])
            gap = int(snapshot["next_next_return_gap"])

    feature_array = np.stack(features, axis=0).reshape(size, seq_len, -1)
    return feature_array, query_mask


class GeometryReadout(nn.Module):
    def __init__(self, input_dim: int, num_classes: int) -> None:
        super().__init__()
        self.net = nn.Sequential(
            nn.Linear(input_dim, 48),
            nn.GELU(),
            nn.Linear(48, num_classes),
        )

    def forward(self, x: torch.Tensor) -> torch.Tensor:
        return self.net(x)


class GeometryNativeSequenceModel:
    """Deterministic geometric sequence engine with a trained linear readout."""

    def __init__(self, input_dim: int, num_classes: int = 3, seed: int = 0) -> None:
        torch.manual_seed(seed)
        self.readout = GeometryReadout(input_dim, num_classes)
        self.loss_fn = nn.CrossEntropyLoss()

    @property
    def param_count(self) -> int:
        return sum(parameter.numel() for parameter in self.readout.parameters())

    @property
    def effective_state_size(self) -> int:
        # Three entity values plus four exact geometric coordinates.
        return 7

    def fit(
        self,
        train_features: np.ndarray,
        train_targets: np.ndarray,
        *,
        epochs: int = 90,
        learning_rate: float = 1.5e-2,
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


class TinyTransformerBaseline(nn.Module):
    def __init__(
        self,
        vocab_size: int,
        num_classes: int = 3,
        d_model: int = 48,
        nhead: int = 4,
        num_layers: int = 2,
        dim_feedforward: int = 96,
        max_len: int = 32,
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


class TransformerSequenceTrainer:
    def __init__(self, *, seq_len: int, seed: int = 0) -> None:
        torch.manual_seed(seed)
        self.model = TinyTransformerBaseline(vocab_size=len(TOKENS), max_len=seq_len)
        self.loss_fn = nn.CrossEntropyLoss()

    @property
    def param_count(self) -> int:
        return sum(parameter.numel() for parameter in self.model.parameters())

    @property
    def effective_state_size(self) -> int:
        return 48

    def fit(
        self,
        train_inputs: np.ndarray,
        train_targets: np.ndarray,
        *,
        epochs: int = 16,
        batch_size: int = 64,
        learning_rate: float = 2.5e-3,
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


def run_bounded_sequence_comparison(config: TaskConfig, *, seed: int = 0) -> list[GeometrySequenceMetrics]:
    train_inputs, train_targets, train_query_mask = generate_dataset(
        config.train_size,
        seq_len=config.seq_len,
        seed=seed,
    )
    val_inputs, val_targets, val_query_mask = generate_dataset(
        config.val_size,
        seq_len=config.seq_len,
        seed=seed + 1,
    )
    test_inputs, test_targets, test_query_mask = generate_dataset(
        config.test_size,
        seq_len=config.seq_len,
        seed=seed + 2,
    )

    train_features, _ = extract_geometry_features(train_inputs)
    _val_features, _ = extract_geometry_features(val_inputs)
    test_features, _ = extract_geometry_features(test_inputs)

    geometry_model = GeometryNativeSequenceModel(input_dim=train_features.shape[-1], seed=seed)
    geometry_train_loss, geometry_train_seconds = geometry_model.fit(train_features, train_targets)
    geometry_test_loss, geometry_test_acc, geometry_query_acc, geometry_eval_seconds = geometry_model.evaluate(
        test_features,
        test_targets,
        test_query_mask,
    )

    transformer = TransformerSequenceTrainer(seq_len=config.seq_len, seed=seed)
    transformer_train_loss, transformer_train_seconds = transformer.fit(train_inputs, train_targets)
    transformer_test_loss, transformer_test_acc, transformer_query_acc, transformer_eval_seconds = transformer.evaluate(
        test_inputs,
        test_targets,
        test_query_mask,
    )

    return [
        GeometrySequenceMetrics(
            model="geometry_native_sequence_model",
            train_loss=geometry_train_loss,
            test_loss=geometry_test_loss,
            test_accuracy=geometry_test_acc,
            query_accuracy=geometry_query_acc,
            param_count=geometry_model.param_count,
            effective_state_size=geometry_model.effective_state_size,
            train_seconds=geometry_train_seconds,
            eval_seconds=geometry_eval_seconds,
        ),
        GeometrySequenceMetrics(
            model="tiny_transformer_baseline",
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
