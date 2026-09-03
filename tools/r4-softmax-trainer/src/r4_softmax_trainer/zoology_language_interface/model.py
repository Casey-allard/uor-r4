"""A learned local role reader over a frozen ordinary compound-binding core.

Fact/query clause boundaries are supplied. Role positions, entity masks and
answer labels are not model inputs. Each role remains a full token mixture.
"""

from __future__ import annotations

from dataclasses import asdict, dataclass

import torch
from torch import Tensor, nn
from torch.nn import functional as F

from ..zoology_compound_binding.model import CompoundBindingModel


@dataclass(frozen=True, slots=True)
class LearnedRoleReaderConfig:
    vocab_size: int = 4096
    embedding_width: int = 32
    hidden_width: int = 64
    kernel_size: int = 5
    roles: int = 3
    segments: int = 5

    def __post_init__(self) -> None:
        if (
            self.vocab_size,
            self.embedding_width,
            self.hidden_width,
            self.kernel_size,
            self.roles,
            self.segments,
        ) != (4096, 32, 64, 5, 3, 5):
            raise ValueError("role reader requires the frozen #1077 configuration")


MODEL_CONFIG = asdict(LearnedRoleReaderConfig())
MODEL_POLICY = {
    "name": "LearnedLocalRoleInterfaceV1",
    "issue": 1077,
    "input": "int64 [batch,5,length]; lengths [batch,5]",
    "segmentation": "four complete fact clauses, then one complete question clause",
    "role_order": ["owner", "object", "location"],
    "query_location": "computed but unused by binding and training supervision",
    "reader": "Embedding(4096,32) -> Conv1d(32,64,kernel=5,padding=2) -> GELU -> Linear(64,3)",
    "context": "bidirectional radius two within each supplied, already available clause",
    "position_embeddings": False,
    "entity_masks": False,
    "argmax_routing": False,
    "padding": "zero before convolution; excluded from each role softmax",
    "roles": "softmax across every valid clause token, including grammar and distractors",
    "role_values": "weighted frozen source embedding E[token]; no learned value projection",
    "trainable_parameter_count": 141571,
    "frozen_core_parameter_count": 286976,
    "initialization": "PyTorch defaults once in order: reader Embedding, Conv1d, Linear; seed external",
    "dropout": 0.0,
    "training": "role-position cross entropy outside the model; core parameters frozen",
    "binding": "unchanged source compound norms/Q/K/V, four facts plus learned null, score/8, full softmax mixture, Wout/LN/tied4096 head",
    "control": "eval-only value_cycle: projected fact values right-roll one; null and Q/K/attention unchanged",
    "scope": "learned local roles with supplied clause boundaries; not unrestricted parsing",
}


def _valid_tokens(inputs: Tensor, lengths: Tensor) -> Tensor:
    if (
        inputs.ndim != 3
        or inputs.shape[0] < 1
        or inputs.shape[1] != 5
        or inputs.shape[2] < 1
        or inputs.dtype != torch.long
    ):
        raise ValueError("role reader requires int64 inputs [batch,5,length]")
    if (
        lengths.shape != inputs.shape[:2]
        or lengths.dtype != torch.long
        or lengths.device != inputs.device
        or not bool(((lengths >= 1) & (lengths <= inputs.shape[2])).all())
    ):
        raise ValueError("role reader requires valid int64 lengths [batch,5]")
    valid = torch.arange(inputs.shape[2], device=inputs.device) < lengths.unsqueeze(-1)
    if bool((((inputs < 0) | (inputs >= 4096)) & valid).any()):
        raise ValueError("valid clause tokens must be vocabulary IDs")
    return valid


class LearnedRoleReader(nn.Module):
    """Shared role scores with no token-position or entity-type input."""

    def __init__(self, config: LearnedRoleReaderConfig | None = None) -> None:
        super().__init__()
        self.config = config or LearnedRoleReaderConfig()
        self.embedding = nn.Embedding(
            self.config.vocab_size, self.config.embedding_width
        )
        self.context = nn.Conv1d(
            self.config.embedding_width,
            self.config.hidden_width,
            self.config.kernel_size,
            padding=self.config.kernel_size // 2,
        )
        self.role_projection = nn.Linear(self.config.hidden_width, self.config.roles)

    def parameter_count(self) -> int:
        return sum(parameter.numel() for parameter in self.parameters())

    def role_scores(self, inputs: Tensor, lengths: Tensor) -> Tensor:
        """Return masked [batch,5,3,length] scores; CE targets stay outside."""

        valid = _valid_tokens(inputs, lengths)
        safe_inputs = inputs.masked_fill(~valid, 0)
        embeddings = self.embedding(safe_inputs) * valid.unsqueeze(-1)
        batch, segments, length, width = embeddings.shape
        hidden = self.context(
            embeddings.reshape(batch * segments, length, width).transpose(1, 2)
        )
        scores = self.role_projection(F.gelu(hidden.transpose(1, 2)))
        scores = scores.transpose(1, 2).reshape(batch, segments, 3, length)
        return scores.masked_fill(~valid.unsqueeze(2), -torch.inf)

    def forward(self, inputs: Tensor, lengths: Tensor) -> Tensor:
        return torch.softmax(self.role_scores(inputs, lengths), dim=-1)


class LanguageInterfaceModel(nn.Module):
    """Feed learned role mixtures into the unchanged, frozen #1073 operations."""

    def __init__(self, core: CompoundBindingModel, reader: LearnedRoleReader) -> None:
        super().__init__()
        if not isinstance(core, CompoundBindingModel):
            raise TypeError(
                "language interface requires the source compound-binding core"
            )
        self.core = core
        self.reader = reader
        self.core.requires_grad_(False)
        self.core.eval()

    def train(self, mode: bool = True) -> LanguageInterfaceModel:
        super().train(mode)
        self.core.eval()
        return self

    def forward(
        self, inputs: Tensor, lengths: Tensor, *, control: str = "none"
    ) -> dict[str, Tensor]:
        if control not in ("none", "value_cycle"):
            raise ValueError("unknown language-interface control")
        if self.training and control != "none":
            raise ValueError(
                "language-interface controls are forbidden during training"
            )
        valid = _valid_tokens(inputs, lengths)
        role_attention = self.reader(inputs, lengths)
        token_values = self.core.embedding(inputs.masked_fill(~valid, 0))
        token_values = token_values * valid.unsqueeze(-1)
        role_vectors = torch.matmul(role_attention, token_values)

        query_roles = role_vectors[:, 4, :2]
        query = self.core.query_projection(
            self.core.compound_norm(query_roles.reshape(inputs.shape[0], 1, 128))
        )
        fact_roles = role_vectors[:, :4]
        keys = self.core.key_projection(
            self.core.compound_norm(
                fact_roles[:, :, :2].reshape(inputs.shape[0], 4, 128)
            )
        )
        values = self.core.value_projection(
            self.core.location_norm(fact_roles[:, :, 2])
        )
        if control == "value_cycle":
            values = torch.roll(values, shifts=1, dims=1)
        keys = torch.cat(
            (keys, self.core.null_key.expand(inputs.shape[0], 1, -1)), dim=1
        )
        values = torch.cat(
            (values, self.core.null_value.expand(inputs.shape[0], 1, -1)), dim=1
        )
        scores = torch.matmul(query, keys.transpose(-2, -1)) / 8.0
        binding_attention = torch.softmax(scores, dim=-1)
        hidden = self.core.output_norm(
            self.core.output_projection(torch.matmul(binding_attention, values))
        )
        logits = self.core.lm_head(hidden)
        return {
            "logits": logits[:, 0],
            "binding_attention": binding_attention[:, 0],
            "role_attention": role_attention,
            "role_vectors": role_vectors,
        }
