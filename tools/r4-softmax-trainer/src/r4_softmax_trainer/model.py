"""Frozen Llama/R4-block causal-softmax model implementations."""

from __future__ import annotations

import math
from dataclasses import dataclass

import torch
from torch import Tensor, nn
from torch.nn import functional as F

from .constants import FROZEN_MODEL_CONFIG, ModelConfig


def expected_parameter_count(config: ModelConfig) -> int:
    """Return the exact tied-head parameter count implied by ``config``."""
    config.validate()
    embedding = config.vocab_size * config.hidden_size
    per_layer = (
        4 * config.hidden_size * config.hidden_size
        + 3 * config.hidden_size * config.intermediate_size
        + 2 * config.hidden_size
    )
    final_norm = config.hidden_size
    return embedding + config.num_hidden_layers * per_layer + final_norm


# Retain the original public constant for callers that bind the #1014 contract.
EXPECTED_PARAMETER_COUNT = expected_parameter_count(FROZEN_MODEL_CONFIG)


class RMSNorm(nn.Module):
    def __init__(self, width: int, epsilon: float) -> None:
        super().__init__()
        self.weight = nn.Parameter(torch.ones(width))
        self.epsilon = epsilon

    def forward(self, values: Tensor) -> Tensor:
        normalized = values.float() * torch.rsqrt(values.float().pow(2).mean(-1, keepdim=True) + self.epsilon)
        return normalized.to(values.dtype) * self.weight


class RotaryEmbedding(nn.Module):
    """Unscaled half-rotation RoPE matching `HuggingFaceLlamaOracle`."""

    def __init__(self, head_dim: int, theta: float, context: int) -> None:
        super().__init__()
        exponent = torch.arange(0, head_dim, 2, dtype=torch.float32) / head_dim
        inverse_frequency = 1.0 / (theta**exponent)
        positions = torch.arange(context, dtype=torch.float32)
        angles = torch.outer(positions, inverse_frequency)
        self.register_buffer("cosine", angles.cos(), persistent=False)
        self.register_buffer("sine", angles.sin(), persistent=False)

    def forward(self, values: Tensor) -> Tensor:
        # values: [batch, heads, time, head_dim]. HF Llama's default mode
        # rotates first-half coordinates against second-half coordinates.
        time = values.shape[-2]
        half = values.shape[-1] // 2
        first, second = values[..., :half], values[..., half:]
        cosine = self.cosine[:time].view(1, 1, time, half).to(values.device)
        sine = self.sine[:time].view(1, 1, time, half).to(values.device)
        return torch.cat((first * cosine - second * sine, second * cosine + first * sine), dim=-1)


class CausalSelfAttention(nn.Module):
    """Ordinary stable causal Q/K/V softmax and weighted-value aggregation."""

    def __init__(self, config: ModelConfig) -> None:
        super().__init__()
        width = config.hidden_size
        self.q_proj = nn.Linear(width, width, bias=False)
        self.k_proj = nn.Linear(width, width, bias=False)
        self.v_proj = nn.Linear(width, width, bias=False)
        self.o_proj = nn.Linear(width, width, bias=False)
        self.heads = config.num_attention_heads
        self.head_dim = config.head_dim
        self.rope = RotaryEmbedding(config.head_dim, config.rope_theta, config.max_position_embeddings)
        causal_mask = torch.triu(
            torch.ones(config.max_position_embeddings, config.max_position_embeddings, dtype=torch.bool),
            diagonal=1,
        )
        self.register_buffer("causal_mask", causal_mask, persistent=False)

    def _heads(self, values: Tensor) -> Tensor:
        batch, time, _ = values.shape
        return values.view(batch, time, self.heads, self.head_dim).transpose(1, 2)

    def forward(self, values: Tensor) -> Tensor:
        _, time, _ = values.shape
        query = self.rope(self._heads(self.q_proj(values)))
        key = self.rope(self._heads(self.k_proj(values)))
        value = self._heads(self.v_proj(values))
        scores = torch.matmul(query.float(), key.float().transpose(-2, -1)) / math.sqrt(self.head_dim)
        scores = scores.masked_fill(self.causal_mask[:time, :time], float("-inf"))
        weights = torch.softmax(scores, dim=-1, dtype=torch.float32)
        attended = torch.matmul(weights, value.float()).to(values.dtype)
        attended = attended.transpose(1, 2).contiguous().view(values.shape)
        return self.o_proj(attended)


class SwiGLU(nn.Module):
    def __init__(self, config: ModelConfig) -> None:
        super().__init__()
        self.gate_proj = nn.Linear(config.hidden_size, config.intermediate_size, bias=False)
        self.up_proj = nn.Linear(config.hidden_size, config.intermediate_size, bias=False)
        self.down_proj = nn.Linear(config.intermediate_size, config.hidden_size, bias=False)

    def forward(self, values: Tensor) -> Tensor:
        return self.down_proj(F.silu(self.gate_proj(values)) * self.up_proj(values))


class DecoderLayer(nn.Module):
    def __init__(self, config: ModelConfig) -> None:
        super().__init__()
        self.input_layernorm = RMSNorm(config.hidden_size, config.rms_norm_eps)
        self.self_attn = CausalSelfAttention(config)
        self.post_attention_layernorm = RMSNorm(config.hidden_size, config.rms_norm_eps)
        self.mlp = SwiGLU(config)

    def forward(self, values: Tensor, *, attention_off: bool = False) -> Tensor:
        attention_output = self.self_attn(self.input_layernorm(values))
        if attention_off:
            attention_output = torch.zeros_like(attention_output)
        values = values + attention_output
        return values + self.mlp(self.post_attention_layernorm(values))


class LlamaBackbone(nn.Module):
    def __init__(self, config: ModelConfig) -> None:
        super().__init__()
        self.embed_tokens = nn.Embedding(config.vocab_size, config.hidden_size)
        self.layers = nn.ModuleList(DecoderLayer(config) for _ in range(config.num_hidden_layers))
        self.norm = RMSNorm(config.hidden_size, config.rms_norm_eps)

    def forward(self, token_ids: Tensor, *, attention_off: bool = False) -> Tensor:
        values = self.embed_tokens(token_ids)
        for layer in self.layers:
            values = layer(values, attention_off=attention_off)
        return self.norm(values)


@dataclass(slots=True)
class ModelOutput:
    logits: Tensor
    loss: Tensor | None


class R4SoftmaxForCausalLM(nn.Module):
    """Trainable source model with a tied output head and HF-compatible keys."""

    def __init__(self, config: ModelConfig = FROZEN_MODEL_CONFIG) -> None:
        super().__init__()
        config.validate()
        self.config = config
        self.model = LlamaBackbone(config)
        self.apply(self._initialize)
        actual_parameter_count = self.parameter_count()
        configured_parameter_count = expected_parameter_count(config)
        if actual_parameter_count != configured_parameter_count:
            raise RuntimeError(
                "parameter count "
                f"{actual_parameter_count} != config-derived {configured_parameter_count}"
            )

    @staticmethod
    def _initialize(module: nn.Module) -> None:
        if isinstance(module, (nn.Linear, nn.Embedding)):
            nn.init.normal_(module.weight, mean=0.0, std=0.02)

    def forward(
        self,
        token_ids: Tensor,
        targets: Tensor | None = None,
        *,
        attention_off: bool = False,
    ) -> ModelOutput:
        if token_ids.ndim != 2:
            raise ValueError("token_ids must have shape [batch, time]")
        if token_ids.shape[1] > self.config.max_position_embeddings:
            raise ValueError("sequence exceeds frozen context")
        hidden = self.model(token_ids, attention_off=attention_off)
        logits = F.linear(hidden, self.model.embed_tokens.weight)
        loss = None
        if targets is not None:
            if targets.shape != token_ids.shape:
                raise ValueError("targets must match token_ids shape")
            loss = F.cross_entropy(logits.float().reshape(-1, logits.shape[-1]), targets.reshape(-1))
        return ModelOutput(logits=logits, loss=loss)

    def parameter_count(self) -> int:
        return sum(parameter.numel() for parameter in self.parameters())


def expected_hf_tensor_names(config: ModelConfig = FROZEN_MODEL_CONFIG) -> set[str]:
    names = {"model.embed_tokens.weight", "model.norm.weight"}
    suffixes = [
        "input_layernorm.weight",
        "self_attn.q_proj.weight",
        "self_attn.k_proj.weight",
        "self_attn.v_proj.weight",
        "self_attn.o_proj.weight",
        "post_attention_layernorm.weight",
        "mlp.gate_proj.weight",
        "mlp.down_proj.weight",
        "mlp.up_proj.weight",
    ]
    for layer in range(config.num_hidden_layers):
        names.update(f"model.layers.{layer}.{suffix}" for suffix in suffixes)
    return names


def export_state_dict(model: R4SoftmaxForCausalLM) -> dict[str, Tensor]:
    state = {name: tensor.detach().to(device="cpu", dtype=torch.float32).contiguous() for name, tensor in model.state_dict().items()}
    expected = expected_hf_tensor_names(model.config)
    if set(state) != expected:
        missing = sorted(expected - set(state))
        unexpected = sorted(set(state) - expected)
        raise RuntimeError(f"HF tensor contract mismatch: missing={missing}, unexpected={unexpected}")
    return state
