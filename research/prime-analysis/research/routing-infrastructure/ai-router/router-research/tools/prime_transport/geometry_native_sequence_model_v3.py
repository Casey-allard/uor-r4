#!/usr/bin/env python3
"""Third bounded geometry-native sequence-model comparison on a language-like task."""

from __future__ import annotations

from dataclasses import dataclass
from time import perf_counter

import numpy as np
import torch
from torch import nn
from torch.utils.data import DataLoader, TensorDataset


torch.set_num_threads(1)


TOKENS_V3 = [
    "MENTION0",
    "MENTION1",
    "MENTION2",
    "SET_SPEAKER",
    "SET_TOPIC",
    "SHIFT_STYLE",
    "TAG0",
    "TAG1",
    "TAG2",
    "REFER",
    "ASK",
]
TOKEN_TO_ID_V3 = {token: idx for idx, token in enumerate(TOKENS_V3)}
NUM_CLASSES_V3 = 3
QUERY_IDS_V3 = {TOKEN_TO_ID_V3["REFER"], TOKEN_TO_ID_V3["ASK"]}

MENTION_IDS = {
    TOKEN_TO_ID_V3["MENTION0"]: 0,
    TOKEN_TO_ID_V3["MENTION1"]: 1,
    TOKEN_TO_ID_V3["MENTION2"]: 2,
}
TAG_IDS = {
    TOKEN_TO_ID_V3["TAG0"]: 0,
    TOKEN_TO_ID_V3["TAG1"]: 1,
    TOKEN_TO_ID_V3["TAG2"]: 2,
}

TOKEN_DB_V3 = np.array([1, 2, 3, 1, 2, 1, 3, 1, 2, 1, 2], dtype=np.int64)
TOKEN_DPHI_V3 = np.array([0, 1, 2, 1, 2, 1, 0, 1, 2, 1, 0], dtype=np.int64)
TOKEN_DR_V3 = np.array([0, 0, 0, 1, 1, 1, 0, 1, 0, 1, 1], dtype=np.int64)
TOKEN_DGAP_V3 = np.array([1, 2, 1, 2, 1, 2, 1, 2, 3, 1, 2], dtype=np.int64)


@dataclass(frozen=True)
class TaskConfigV3:
    seq_len: int = 30
    train_size: int = 1024
    val_size: int = 256
    test_size: int = 256


@dataclass
class SequenceMetricsV3:
    model: str
    train_loss: float
    test_loss: float
    test_accuracy: float
    query_accuracy: float
    param_count: int
    effective_state_size: int
    train_seconds: float
    eval_seconds: float


def _initial_discourse_state() -> tuple[int, int, int, int, int, list[int]]:
    # focus, speaker, topic, style, next_return_gap, tags
    return 0, 0, 0, 0, 1, [0, 1, 2]


def _initial_geometry_state() -> tuple[int, int, int]:
    return 0, 0, 0


def _referent_role(style: int, phi: int, r: int, next_return_gap: int) -> int:
    # 0=focus, 1=speaker, 2=topic
    return (style + phi + r + next_return_gap) % 3


def _role_entity(
    role: int,
    *,
    focus: int,
    speaker: int,
    topic: int,
) -> int:
    if role == 0:
        return focus
    if role == 1:
        return speaker
    return topic


def step_discourse_world(
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
) -> tuple[int, dict[str, int | list[int]], bool]:
    """Apply one discourse-style token under evolving geometry."""

    next_focus = focus
    next_speaker = speaker
    next_topic = topic
    next_style = style
    next_tags = list(tags)
    is_query = token_id in QUERY_IDS_V3

    if token_id in MENTION_IDS:
        next_focus = MENTION_IDS[token_id]
    elif token_id == TOKEN_TO_ID_V3["SET_SPEAKER"]:
        next_speaker = next_focus
    elif token_id == TOKEN_TO_ID_V3["SET_TOPIC"]:
        next_topic = next_focus
    elif token_id == TOKEN_TO_ID_V3["SHIFT_STYLE"]:
        next_style = (next_style + 1) % 3
    elif token_id in TAG_IDS:
        role = _referent_role(next_style, phi, r, next_return_gap)
        referent = _role_entity(role, focus=next_focus, speaker=next_speaker, topic=next_topic)
        next_tags[referent] = TAG_IDS[token_id]
    elif token_id == TOKEN_TO_ID_V3["REFER"]:
        pass
    elif token_id == TOKEN_TO_ID_V3["ASK"]:
        pass
    else:
        raise ValueError(f"unexpected token id: {token_id}")

    role = _referent_role(next_style, phi, r, next_return_gap)
    referent = _role_entity(role, focus=next_focus, speaker=next_speaker, topic=next_topic)

    if token_id == TOKEN_TO_ID_V3["ASK"]:
        target = next_tags[referent]
    else:
        target = referent

    snapshot = {
        "b": b,
        "phi": phi,
        "r": r,
        "focus": next_focus,
        "speaker": next_speaker,
        "topic": next_topic,
        "style": next_style,
        "next_return_gap": next_return_gap,
        "referent_role": role,
        "referent_entity": referent,
        "tags": next_tags,
    }

    next_b = (b + int(TOKEN_DB_V3[token_id])) % 5
    next_phi = (phi + int(TOKEN_DPHI_V3[token_id])) % 3
    next_r = (r + int(TOKEN_DR_V3[token_id])) % 4
    next_gap = 1 + ((next_return_gap - 1 + int(TOKEN_DGAP_V3[token_id])) % 4)
    snapshot["next_b"] = next_b
    snapshot["next_phi"] = next_phi
    snapshot["next_r"] = next_r
    snapshot["next_next_return_gap"] = next_gap
    return int(target), snapshot, is_query


def generate_dataset_v3(size: int, *, seq_len: int, seed: int) -> tuple[np.ndarray, np.ndarray, np.ndarray]:
    rng = np.random.default_rng(seed)
    inputs = np.zeros((size, seq_len), dtype=np.int64)
    targets = np.zeros((size, seq_len), dtype=np.int64)
    query_mask = np.zeros((size, seq_len), dtype=np.float32)

    base_weights = np.array(
        [0.11, 0.11, 0.11, 0.08, 0.08, 0.09, 0.08, 0.08, 0.08, 0.09, 0.09],
        dtype=np.float64,
    )
    base_weights /= base_weights.sum()

    for row in range(size):
        b, phi, r = _initial_geometry_state()
        focus, speaker, topic, style, gap, tags = _initial_discourse_state()
        for col in range(seq_len):
            token_id = int(rng.choice(len(TOKENS_V3), p=base_weights))
            target, snapshot, is_query = step_discourse_world(
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
            )
            inputs[row, col] = token_id
            targets[row, col] = target
            query_mask[row, col] = 1.0 if is_query else 0.0
            focus = int(snapshot["focus"])
            speaker = int(snapshot["speaker"])
            topic = int(snapshot["topic"])
            style = int(snapshot["style"])
            tags = list(snapshot["tags"])
            b = int(snapshot["next_b"])
            phi = int(snapshot["next_phi"])
            r = int(snapshot["next_r"])
            gap = int(snapshot["next_next_return_gap"])
    return inputs, targets, query_mask


def _one_hot(index: int, size: int) -> np.ndarray:
    vector = np.zeros(size, dtype=np.float32)
    vector[index] = 1.0
    return vector


def _build_feature_vector_v3(token_id: int, snapshot: dict[str, int | list[int]]) -> np.ndarray:
    parts: list[np.ndarray] = []
    parts.append(_one_hot(token_id, len(TOKENS_V3)))
    parts.append(_one_hot(int(snapshot["b"]), 5))
    parts.append(_one_hot(int(snapshot["phi"]), 3))
    parts.append(_one_hot(int(snapshot["r"]), 4))
    parts.append(_one_hot(int(snapshot["next_return_gap"]) - 1, 4))
    parts.append(_one_hot(int(snapshot["focus"]), 3))
    parts.append(_one_hot(int(snapshot["speaker"]), 3))
    parts.append(_one_hot(int(snapshot["topic"]), 3))
    parts.append(_one_hot(int(snapshot["style"]), 3))
    parts.append(_one_hot(int(snapshot["referent_role"]), 3))
    parts.append(_one_hot(int(snapshot["referent_entity"]), 3))

    tags = list(snapshot["tags"])
    for tag in tags:
        parts.append(_one_hot(int(tag), 3))

    return np.concatenate(parts, axis=0)


def extract_geometry_features_v3(inputs: np.ndarray) -> tuple[np.ndarray, np.ndarray]:
    size, seq_len = inputs.shape
    features: list[np.ndarray] = []
    query_mask = np.zeros((size, seq_len), dtype=np.float32)

    for row in range(size):
        b, phi, r = _initial_geometry_state()
        focus, speaker, topic, style, gap, tags = _initial_discourse_state()
        for col in range(seq_len):
            token_id = int(inputs[row, col])
            _, snapshot, is_query = step_discourse_world(
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
            )
            features.append(_build_feature_vector_v3(token_id, snapshot))
            query_mask[row, col] = 1.0 if is_query else 0.0
            focus = int(snapshot["focus"])
            speaker = int(snapshot["speaker"])
            topic = int(snapshot["topic"])
            style = int(snapshot["style"])
            tags = list(snapshot["tags"])
            b = int(snapshot["next_b"])
            phi = int(snapshot["next_phi"])
            r = int(snapshot["next_r"])
            gap = int(snapshot["next_next_return_gap"])

    feature_array = np.stack(features, axis=0).reshape(size, seq_len, -1)
    return feature_array, query_mask


class GeometryReadoutV3(nn.Module):
    def __init__(self, input_dim: int) -> None:
        super().__init__()
        self.net = nn.Sequential(
            nn.Linear(input_dim, 64),
            nn.GELU(),
            nn.Linear(64, 32),
            nn.GELU(),
            nn.Linear(32, NUM_CLASSES_V3),
        )

    def forward(self, x: torch.Tensor) -> torch.Tensor:
        return self.net(x)


class GeometryNativeSequenceModelV3:
    def __init__(self, input_dim: int, *, seed: int = 0) -> None:
        torch.manual_seed(seed)
        self.readout = GeometryReadoutV3(input_dim)
        self.loss_fn = nn.CrossEntropyLoss()

    @property
    def param_count(self) -> int:
        return sum(parameter.numel() for parameter in self.readout.parameters())

    @property
    def effective_state_size(self) -> int:
        return 10  # geometry plus discourse roles and entity tags

    def fit(
        self,
        train_features: np.ndarray,
        train_targets: np.ndarray,
        *,
        epochs: int = 90,
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


class TinyTransformerBaselineV3(nn.Module):
    def __init__(
        self,
        *,
        vocab_size: int,
        d_model: int = 64,
        nhead: int = 4,
        num_layers: int = 2,
        dim_feedforward: int = 128,
        max_len: int = 40,
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
        self.readout = nn.Linear(d_model, NUM_CLASSES_V3)

    def forward(self, tokens: torch.Tensor) -> torch.Tensor:
        batch_size, seq_len = tokens.shape
        positions = torch.arange(seq_len, device=tokens.device).unsqueeze(0).expand(batch_size, seq_len)
        hidden = self.token_embedding(tokens) + self.position_embedding(positions)
        mask = torch.triu(torch.full((seq_len, seq_len), float("-inf"), device=tokens.device), diagonal=1)
        encoded = self.encoder(hidden, mask=mask)
        return self.readout(encoded)


class TransformerSequenceTrainerV3:
    def __init__(self, *, seq_len: int, seed: int = 0) -> None:
        torch.manual_seed(seed)
        self.model = TinyTransformerBaselineV3(vocab_size=len(TOKENS_V3), max_len=seq_len)
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


def run_bounded_sequence_comparison_v3(config: TaskConfigV3, *, seed: int = 0) -> list[SequenceMetricsV3]:
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

    train_features, _ = extract_geometry_features_v3(train_inputs)
    test_features, _ = extract_geometry_features_v3(test_inputs)

    geometry_model = GeometryNativeSequenceModelV3(train_features.shape[-1], seed=seed)
    geometry_train_loss, geometry_train_seconds = geometry_model.fit(train_features, train_targets)
    geometry_test_loss, geometry_test_acc, geometry_query_acc, geometry_eval_seconds = geometry_model.evaluate(
        test_features,
        test_targets,
        test_query_mask,
    )

    transformer = TransformerSequenceTrainerV3(seq_len=config.seq_len, seed=seed)
    transformer_train_loss, transformer_train_seconds = transformer.fit(train_inputs, train_targets)
    transformer_test_loss, transformer_test_acc, transformer_query_acc, transformer_eval_seconds = transformer.evaluate(
        test_inputs,
        test_targets,
        test_query_mask,
    )

    return [
        SequenceMetricsV3(
            model="geometry_native_sequence_model_v3",
            train_loss=geometry_train_loss,
            test_loss=geometry_test_loss,
            test_accuracy=geometry_test_acc,
            query_accuracy=geometry_query_acc,
            param_count=geometry_model.param_count,
            effective_state_size=geometry_model.effective_state_size,
            train_seconds=geometry_train_seconds,
            eval_seconds=geometry_eval_seconds,
        ),
        SequenceMetricsV3(
            model="tiny_transformer_baseline_v3",
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
