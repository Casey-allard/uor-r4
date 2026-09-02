# SPDX-License-Identifier: Apache-2.0
# Adapted from HazyResearch/Zoology; Copyright 2021 The Meerkat Team.
"""Stock Zoology causal-attention control adapted for issue #1047.

This is a narrow, device-neutral port of the released Zoology ICLR 2024
Figure-2 attention model.  The model equations, module registration order,
two-stage residual path, and double initialization pass follow the Apache-2.0
source at commit ``de4e258``:

https://github.com/HazyResearch/zoology/tree/de4e258784224e09909c257ff3ea040f089ed660

Only two integration changes are intentional: position IDs follow the input
device (the released implementation hard-coded CUDA), and selected query
positions can be gathered before the tied vocabulary projection so a
query-only training run never materializes ``[batch, time, vocabulary]``.
"""

from __future__ import annotations

import math
import random
from dataclasses import dataclass

import numpy as np
import torch
from torch import Tensor, nn
from torch.nn import functional as F


SOURCE_COMMIT = "de4e258784224e09909c257ff3ea040f089ed660"
SOURCE_URL = f"https://github.com/HazyResearch/zoology/tree/{SOURCE_COMMIT}"
SOURCE_SEED = 123
SOURCE_INITIALIZER_RANGE = 0.02


@dataclass(frozen=True, slots=True)
class ZoologyFigure2Config:
    """Frozen stock-attention shape with #1047 corpus dimensions.

    The released Figure-2 control uses ``d_model=64``, two layers, one head,
    learned absolute positions, attention/embedding dropout of 0.1, and no
    state mixer.  ``vocab_size`` and ``max_position_embeddings`` are adapted
    to the #1047 corpus rather than changing the learned mechanism.
    """

    vocab_size: int = 4096
    d_model: int = 64
    n_layers: int = 2
    num_heads: int = 1
    max_position_embeddings: int = 120
    attention_dropout: float = 0.1
    embed_dropout: float = 0.1
    resid_dropout: float = 0.0
    layer_norm_epsilon: float = 1.0e-5
    pad_vocab_size_multiple: int = 1

    def __post_init__(self) -> None:
        if self.vocab_size < 1:
            raise ValueError("vocab_size must be positive")
        if self.d_model < 1:
            raise ValueError("d_model must be positive")
        if self.n_layers < 1:
            raise ValueError("n_layers must be positive")
        if self.num_heads < 1 or self.d_model % self.num_heads != 0:
            raise ValueError("num_heads must divide d_model")
        if self.max_position_embeddings < 1:
            raise ValueError("max_position_embeddings must be positive")
        if self.pad_vocab_size_multiple < 1:
            raise ValueError("pad_vocab_size_multiple must be positive")
        if self.vocab_size % self.pad_vocab_size_multiple != 0:
            raise ValueError("vocab_size must already include padding multiple")
        for name, value in (
            ("attention_dropout", self.attention_dropout),
            ("embed_dropout", self.embed_dropout),
            ("resid_dropout", self.resid_dropout),
        ):
            if not 0.0 <= value < 1.0:
                raise ValueError(f"{name} must be in [0, 1)")
        if self.layer_norm_epsilon <= 0.0:
            raise ValueError("layer_norm_epsilon must be positive")


@dataclass(slots=True)
class ZoologyModelOutput:
    """Language-model output for either full or selected projection."""

    logits: Tensor
    loss: Tensor | None
    hidden_states: Tensor
    selected_positions: Tensor | None = None
    selected_targets: Tensor | None = None
    # One pre-dropout softmax tensor per layer, each [batch, heads, time, time].
    attention_weights: tuple[Tensor, ...] | None = None


def set_zoology_seed(seed: int = SOURCE_SEED) -> None:
    """Apply the released CPU-relevant Zoology seed contract.

    The upstream helper seeds Python, NumPy, Torch, and CUDA.  This CPU-native
    control intentionally has no CUDA dependency; the first three calls retain
    the released model's construction and dropout determinism.
    """

    random.seed(seed)
    np.random.seed(seed)
    torch.manual_seed(seed)


def _init_weights(
    module: nn.Module,
    *,
    n_layers: int,
    initializer_range: float = SOURCE_INITIALIZER_RANGE,
    rescale_prenorm_residual: bool = True,
) -> None:
    """Port of ``zoology.model._init_weights`` at ``de4e258``.

    The recursive ``named_parameters`` walk is deliberately retained.  In
    combination with ``Module.apply`` in both the backbone and outer model,
    this reproduces the released double-application initialization behavior.
    """

    if isinstance(module, nn.Linear):
        nn.init.normal_(module.weight, std=initializer_range)
        if module.bias is not None:
            nn.init.zeros_(module.bias)
    elif isinstance(module, nn.Embedding):
        nn.init.normal_(module.weight, std=initializer_range)

    if rescale_prenorm_residual:
        for name, parameter in module.named_parameters():
            if "out_proj.weight" in name or "fc2.weight" in name:
                nn.init.normal_(
                    parameter,
                    std=initializer_range / math.sqrt(2 * n_layers),
                )
            elif "output_linear.0.weight" in name:
                nn.init.normal_(
                    parameter,
                    std=initializer_range / math.sqrt(2 * n_layers),
                )


class _SelfAttention(nn.Module):
    """Released scaled dot-product causal-softmax equation."""

    def __init__(self, *, dropout_p: float) -> None:
        super().__init__()
        self.dropout_p = dropout_p

    def forward(self, qkv: Tensor) -> tuple[Tensor, Tensor]:
        if qkv.ndim != 5 or qkv.shape[2] != 3:
            raise ValueError("qkv must have shape [batch,time,3,heads,head_dim]")
        sequence_length = qkv.shape[1]
        query, key, value = qkv.unbind(dim=2)
        softmax_scale = 1.0 / math.sqrt(query.shape[-1])
        scores = torch.einsum(
            "bthd,bshd->bhts",
            query,
            key * softmax_scale,
        )
        causal_mask = torch.triu(
            torch.full(
                (sequence_length, sequence_length),
                -10000.0,
                device=scores.device,
            ),
            diagonal=1,
        )
        scores = scores + causal_mask.to(dtype=scores.dtype)
        attention = torch.softmax(scores, dim=-1, dtype=value.dtype)
        attention_drop = F.dropout(
            attention,
            self.dropout_p if self.training else 0.0,
        )
        output = torch.einsum("bhts,bshd->bthd", attention_drop, value)
        return output, attention


class _MultiHeadAttention(nn.Module):
    """Released biased combined-QKV and biased output projection."""

    def __init__(self, config: ZoologyFigure2Config) -> None:
        super().__init__()
        self.d_model = config.d_model
        self.num_heads = config.num_heads
        self.head_dim = config.d_model // config.num_heads
        self.Wqkv = nn.Linear(config.d_model, 3 * config.d_model, bias=True)
        self.inner_attn = _SelfAttention(dropout_p=config.attention_dropout)
        self.out_proj = nn.Linear(config.d_model, config.d_model, bias=True)

    def forward(self, hidden_states: Tensor) -> tuple[Tensor, Tensor]:
        batch, time, _ = hidden_states.shape
        qkv = self.Wqkv(hidden_states).reshape(
            batch,
            time,
            3,
            self.num_heads,
            self.head_dim,
        )
        context, attention = self.inner_attn(qkv)
        output = self.out_proj(context.reshape(batch, time, self.d_model))
        return output, attention


class _TransformerBlock(nn.Module):
    """Exact released pre-norm two-residual-path block with Identity state."""

    def __init__(
        self,
        config: ZoologyFigure2Config,
        *,
        layer_index: int,
    ) -> None:
        super().__init__()
        self.sequence_mixer = _MultiHeadAttention(config)
        self.state_mixer = nn.Identity()
        self.dropout1 = nn.Dropout(
            config.embed_dropout if layer_index == 0 else config.resid_dropout
        )
        # Source StochasticDepth is exactly the identity at drop_path=0.
        self.drop_path1 = nn.Identity()
        self.norm1 = nn.LayerNorm(config.d_model)
        self.dropout2 = nn.Dropout(config.resid_dropout)
        self.drop_path2 = nn.Identity()
        self.norm2 = nn.LayerNorm(config.d_model)

    def forward(
        self,
        hidden_states: Tensor,
        residual: Tensor | None = None,
    ) -> tuple[Tensor, Tensor, Tensor]:
        dropped = self.drop_path1(self.dropout1(hidden_states))
        residual = dropped + residual if residual is not None else dropped
        hidden_states = self.norm1(residual.to(dtype=self.norm1.weight.dtype))
        hidden_states, attention = self.sequence_mixer(hidden_states)

        dropped = self.drop_path2(self.dropout2(hidden_states))
        residual = dropped + residual if residual is not None else dropped
        hidden_states = self.norm2(residual.to(dtype=self.norm2.weight.dtype))
        hidden_states = self.state_mixer(hidden_states)
        return hidden_states, residual, attention


class _TokenEmbeddings(nn.Module):
    def __init__(self, config: ZoologyFigure2Config) -> None:
        super().__init__()
        self.word_embeddings = nn.Embedding(config.vocab_size, config.d_model)
        self.project_in = None
        self.position_embeddings = nn.Embedding(
            config.max_position_embeddings,
            config.d_model,
        )

    def forward(self, input_ids: Tensor, position_ids: Tensor) -> Tensor:
        embeddings = self.word_embeddings(input_ids)
        if self.project_in is not None:
            embeddings = self.project_in(embeddings)
        return embeddings + self.position_embeddings(position_ids)


class _LanguageModelBackbone(nn.Module):
    def __init__(self, config: ZoologyFigure2Config) -> None:
        super().__init__()
        self.config = config
        self.embeddings = _TokenEmbeddings(config)
        self.layers = nn.ModuleList(
            [
                _TransformerBlock(config, layer_index=layer_index)
                for layer_index in range(config.n_layers)
            ]
        )
        self.drop_f = nn.Dropout(config.resid_dropout)
        self.ln_f = nn.LayerNorm(
            config.d_model,
            eps=config.layer_norm_epsilon,
        )

        # This is the first of the released source's two apply passes.
        self.apply(lambda module: _init_weights(module, n_layers=config.n_layers))

    def forward(
        self,
        input_ids: Tensor,
        *,
        position_ids: Tensor | None = None,
        return_attention: bool = False,
    ) -> tuple[Tensor, tuple[Tensor, ...] | None]:
        batch, time = input_ids.shape
        if position_ids is None:
            position_ids = torch.arange(
                time,
                dtype=torch.long,
                device=input_ids.device,
            ).unsqueeze(0)
        hidden_states = self.embeddings(input_ids, position_ids)
        residual = None
        attention_weights: list[Tensor] | None = [] if return_attention else None
        for layer in self.layers:
            hidden_states, residual, attention = layer(hidden_states, residual)
            if attention_weights is not None:
                attention_weights.append(attention)

        dropped = self.drop_f(hidden_states)
        residual = dropped + residual if residual is not None else dropped
        hidden_states = self.ln_f(residual.to(dtype=self.ln_f.weight.dtype))
        return hidden_states, (
            tuple(attention_weights) if attention_weights is not None else None
        )


class ZoologyFigure2Model(nn.Module):
    """Released two-layer attention control with a query-only projection API."""

    def __init__(self, config: ZoologyFigure2Config | None = None) -> None:
        super().__init__()
        self.config = config if config is not None else ZoologyFigure2Config()
        self.backbone = _LanguageModelBackbone(self.config)
        self.lm_head = nn.Linear(
            self.config.d_model,
            self.config.vocab_size,
            bias=False,
        )

        # The outer source model applies the initializer to the whole model
        # again, consuming the same RNG sequence before tying the head.
        self.apply(
            lambda module: _init_weights(
                module,
                n_layers=self.config.n_layers,
            )
        )
        self.tie_weights()

    def tie_weights(self) -> None:
        self.lm_head.weight = self.backbone.embeddings.word_embeddings.weight

    @staticmethod
    def _validate_input_ids(
        input_ids: Tensor,
        config: ZoologyFigure2Config,
    ) -> None:
        if input_ids.ndim != 2 or input_ids.dtype != torch.long:
            raise ValueError("input_ids must be int64 [batch,time]")
        if input_ids.shape[0] < 1 or input_ids.shape[1] < 1:
            raise ValueError("input_ids must contain at least one token")
        if input_ids.shape[1] > config.max_position_embeddings:
            raise ValueError("input sequence exceeds learned position table")
        if bool((input_ids < 0).any()) or bool(
            (input_ids >= config.vocab_size).any()
        ):
            raise ValueError("input_ids contain an out-of-vocabulary value")

    @staticmethod
    def _validate_position_ids(
        input_ids: Tensor,
        position_ids: Tensor | None,
        config: ZoologyFigure2Config,
    ) -> None:
        if position_ids is None:
            return
        batch, time = input_ids.shape
        if (
            position_ids.dtype != torch.long
            or position_ids.device != input_ids.device
            or position_ids.ndim != 2
            or position_ids.shape[1] != time
            or position_ids.shape[0] not in (1, batch)
        ):
            raise ValueError(
                "position_ids must be int64 [1,time] or [batch,time] on input device"
            )
        if bool((position_ids < 0).any()) or bool(
            (position_ids >= config.max_position_embeddings).any()
        ):
            raise ValueError("position_ids exceed learned position table")

    @staticmethod
    def _validate_targets(
        targets: Tensor,
        *,
        shape: torch.Size,
        device: torch.device,
        vocab_size: int,
    ) -> None:
        if targets.dtype != torch.long or targets.device != device:
            raise ValueError("targets must be int64 on the input device")
        if targets.shape != shape:
            raise ValueError("targets have the wrong shape")
        valid = targets != -100
        if bool(valid.any()):
            admitted = targets[valid]
            if bool((admitted < 0).any()) or bool((admitted >= vocab_size).any()):
                raise ValueError("targets contain an out-of-vocabulary value")

    @staticmethod
    def _validate_selected_positions(
        input_ids: Tensor,
        selected_positions: Tensor,
    ) -> None:
        batch, time = input_ids.shape
        if (
            selected_positions.ndim != 2
            or selected_positions.dtype != torch.long
            or selected_positions.device != input_ids.device
            or selected_positions.shape[0] != batch
            or selected_positions.shape[1] < 1
        ):
            raise ValueError(
                "selected_positions must be int64 [batch,queries] on input device"
            )
        if bool((selected_positions < 0).any()) or bool(
            (selected_positions >= time).any()
        ):
            raise ValueError("selected_positions contain an out-of-prefix value")

    def hidden_states(
        self,
        input_ids: Tensor,
        *,
        position_ids: Tensor | None = None,
        return_attention: bool = False,
    ) -> tuple[Tensor, tuple[Tensor, ...] | None]:
        self._validate_input_ids(input_ids, self.config)
        self._validate_position_ids(input_ids, position_ids, self.config)
        return self.backbone(
            input_ids,
            position_ids=position_ids,
            return_attention=return_attention,
        )

    def forward(
        self,
        input_ids: Tensor,
        *,
        position_ids: Tensor | None = None,
    ) -> Tensor:
        """Match the released stock API: return full vocabulary logits."""

        hidden_states, _ = self.hidden_states(
            input_ids,
            position_ids=position_ids,
        )
        return self.lm_head(hidden_states)

    def forward_full(
        self,
        input_ids: Tensor,
        targets: Tensor | None = None,
        *,
        position_ids: Tensor | None = None,
        return_attention: bool = False,
    ) -> ZoologyModelOutput:
        """Project all positions, optionally computing masked token CE."""

        hidden_states, attention_weights = self.hidden_states(
            input_ids,
            position_ids=position_ids,
            return_attention=return_attention,
        )
        logits = self.lm_head(hidden_states)
        loss = None
        if targets is not None:
            self._validate_targets(
                targets,
                shape=input_ids.shape,
                device=input_ids.device,
                vocab_size=self.config.vocab_size,
            )
            loss = F.cross_entropy(
                logits.reshape(-1, self.config.vocab_size),
                targets.reshape(-1),
            )
        return ZoologyModelOutput(
            logits=logits,
            loss=loss,
            hidden_states=hidden_states,
            attention_weights=attention_weights,
        )

    def forward_selected(
        self,
        input_ids: Tensor,
        selected_positions: Tensor,
        targets: Tensor | None = None,
        *,
        position_ids: Tensor | None = None,
        return_attention: bool = False,
    ) -> ZoologyModelOutput:
        """Gather query hidden states before projecting them to vocabulary."""

        self._validate_selected_positions(input_ids, selected_positions)
        hidden_states, attention_weights = self.hidden_states(
            input_ids,
            position_ids=position_ids,
            return_attention=return_attention,
        )
        gather_index = selected_positions.unsqueeze(-1).expand(
            -1,
            -1,
            self.config.d_model,
        )
        selected_hidden = torch.gather(hidden_states, dim=1, index=gather_index)
        logits = self.lm_head(selected_hidden)

        selected_targets = None
        loss = None
        if targets is not None:
            if targets.shape == input_ids.shape:
                self._validate_targets(
                    targets,
                    shape=input_ids.shape,
                    device=input_ids.device,
                    vocab_size=self.config.vocab_size,
                )
                selected_targets = torch.gather(targets, 1, selected_positions)
            else:
                self._validate_targets(
                    targets,
                    shape=selected_positions.shape,
                    device=input_ids.device,
                    vocab_size=self.config.vocab_size,
                )
                selected_targets = targets
            loss = F.cross_entropy(
                logits.reshape(-1, self.config.vocab_size),
                selected_targets.reshape(-1),
            )

        return ZoologyModelOutput(
            logits=logits,
            loss=loss,
            hidden_states=selected_hidden,
            selected_positions=selected_positions,
            selected_targets=selected_targets,
            attention_weights=attention_weights,
        )

    def parameter_count(self) -> int:
        """Return unique trainable scalar parameters (the tied head counts once)."""

        return sum(parameter.numel() for parameter in self.parameters())
