"""Parameter-free, causal R4 gauge execution of the preserved Zoology model.

Only the inner attention is replaced. Learned projections, positions, norms,
residual ordering, and the tied output head belong to the unchanged source.
"""

from __future__ import annotations

import math
from typing import Literal

import torch
from torch import Tensor, nn

from ..zoology_control.model import (
    ZoologyFigure2Model,
    ZoologyModelOutput,
    _SelfAttention,
)
from .frames import R4InferenceFrames

Execution = Literal["plain", "r4", "source_frame_permuted"]
EXECUTIONS = ("plain", "r4", "source_frame_permuted")


class _R4InnerAttention(nn.Module):
    def __init__(self, source: _SelfAttention, frames: R4InferenceFrames) -> None:
        super().__init__()
        self.source = source
        self.register_buffer("frame_matrices", frames.frame_matrices, persistent=False)
        self.frame_indices: Tensor | None = None
        self.execution: Execution | None = None
        self.audit: dict[str, int | None] | None = None

    def forward(self, qkv: Tensor) -> tuple[Tensor, Tensor]:
        if self.training or self.source.training or torch.is_grad_enabled():
            raise RuntimeError(
                "R4 Zoology attention is inference-only and requires eval mode"
            )
        if self.execution not in EXECUTIONS or self.frame_indices is None:
            raise RuntimeError(
                "R4 attention must be called through its bound inference wrapper"
            )
        if (
            qkv.device.type != "cpu"
            or qkv.dtype != torch.float32
            or qkv.ndim != 5
            or qkv.shape[2] != 3
        ):
            raise ValueError("QKV must be CPU f32 [batch,time,3,heads,head_dim]")
        batch, time, _, heads, head_dim = qkv.shape
        if (
            head_dim < 4
            or head_dim % 4
            or tuple(self.frame_indices.shape) != (batch, time)
        ):
            raise ValueError(
                "QKV must use whole four-lane blocks and the prepared frame shape"
            )
        blocks = head_dim // 4
        pairs = batch * heads * time * (time + 1) // 2
        base_blocks = batch * heads * time * blocks
        self.audit = {
            "admitted_attention_pairs": pairs,
            "materialized_score_slots": batch * heads * time * time
            if self.execution == "plain"
            else pairs,
            "future_score_slots_materialized": batch * heads * time * (time - 1) // 2
            if self.execution == "plain"
            else 0,
            "query_blocks_encoded": 0,
            "key_blocks_encoded": 0,
            "value_blocks_encoded": 0,
            "key_blocks_transported": 0,
            "value_blocks_transported": 0,
            "output_blocks_decoded": 0,
            "source_frame_positions_changed": 0,
            "source_frame_matrices_changed": 0,
            # The original source uses a dense masked square. Its physical
            # future reads are not misrepresented as a measured zero.
            "future_position_reads": None if self.execution == "plain" else 0,
        }
        if self.execution == "plain":
            return self.source(qkv)

        query, key, value = qkv.unbind(dim=2)
        frames = self.frame_matrices[self.frame_indices]
        # Match the source's f32 key scaling before the dot product. Gauge
        # transforms use native f64 frames; scores re-enter source f32 softmax.
        key_scaled = key * (1.0 / math.sqrt(head_dim))
        query_blocks = query.to(torch.float64).reshape(batch, time, heads, blocks, 4)
        key_blocks = key_scaled.to(torch.float64).reshape(batch, time, heads, blocks, 4)
        value_blocks = value.to(torch.float64).reshape(batch, time, heads, blocks, 4)
        query_local = torch.einsum("btji,bthdj->bthdi", frames, query_blocks)
        key_local = torch.einsum("bsji,bshdj->bshdi", frames, key_blocks)
        value_local = torch.einsum("bsji,bshdj->bshdi", frames, value_blocks)
        self.audit.update(
            {
                "query_blocks_encoded": base_blocks,
                "key_blocks_encoded": base_blocks,
                "value_blocks_encoded": base_blocks,
                "key_blocks_transported": pairs * blocks,
                "value_blocks_transported": pairs * blocks,
                "output_blocks_decoded": base_blocks,
            }
        )

        output = torch.empty_like(query)
        attention = torch.zeros(batch, heads, time, time, dtype=torch.float32)
        for position in range(time):
            support = position + 1
            query_frame = frames[:, position]
            source_frames = frames[:, :support]
            if self.execution == "source_frame_permuted":
                permutation = (torch.arange(support) + 1) % support
                source_frames = source_frames[:, permutation]
                self.audit["source_frame_positions_changed"] += (
                    batch
                    * heads
                    * int(torch.count_nonzero(permutation != torch.arange(support)))
                )
                self.audit["source_frame_matrices_changed"] += heads * int(
                    torch.count_nonzero(
                        self.frame_indices[:, :support]
                        != self.frame_indices[:, permutation]
                    )
                )
            # Every query reads only source positions 0..position. Future
            # entries of the returned square remain exactly zero, without
            # computing transported future scores/values and masking later.
            connection = torch.einsum("bji,bsjk->bsik", query_frame, source_frames)
            transported_keys = torch.einsum(
                "bsij,bshdj->bhsdi", connection, key_local[:, :support]
            )
            scores = torch.einsum(
                "bhdi,bhsdi->bhs", query_local[:, position], transported_keys
            ).to(torch.float32)
            weights = torch.softmax(scores, dim=-1, dtype=torch.float32)
            transported_values = torch.einsum(
                "bsij,bshdj->bhsdi", connection, value_local[:, :support]
            )
            attended_local = torch.einsum(
                "bhs,bhsdi->bhdi", weights.to(torch.float64), transported_values
            )
            attended_model = torch.einsum("bij,bhdj->bhdi", query_frame, attended_local)
            output[:, position] = attended_model.reshape(batch, heads, head_dim).to(
                torch.float32
            )
            attention[:, :, position, :support] = weights
        return output, attention


class R4ZoologyInference:
    """Install an inference-only inner seam without changing learned state.

    This object owns one model invocation at a time. All learned tensor names,
    bytes, and aliases survive installation; frame buffers are nonpersistent.
    """

    def __init__(self, model: ZoologyFigure2Model, frames: R4InferenceFrames) -> None:
        if not isinstance(model, ZoologyFigure2Model):
            raise TypeError("R4 inference requires the unchanged Zoology source model")
        if model.config.vocab_size > frames.token_leaf_indices.numel():
            raise ValueError("native token map does not cover the model vocabulary")
        if model.config.d_model // model.config.num_heads % 4:
            raise ValueError("Zoology head width must be divisible by four")
        if any(
            parameter.device.type != "cpu" or parameter.dtype != torch.float32
            for parameter in model.parameters()
        ):
            raise ValueError("R4 inference requires CPU f32 learned tensors")
        sources = [layer.sequence_mixer.inner_attn for layer in model.backbone.layers]
        if any(not isinstance(source, _SelfAttention) for source in sources):
            raise ValueError(
                "inner attention was already replaced or is not the preserved source"
            )
        self.model = model
        self.frames = frames
        self.adapters: list[_R4InnerAttention] = []
        self.last_audit: dict[str, str | int | None | list[int]] | None = None
        self._active = False
        for layer, source in zip(model.backbone.layers, sources, strict=True):
            adapter = _R4InnerAttention(source, frames)
            layer.sequence_mixer.inner_attn = adapter
            self.adapters.append(adapter)
        # New modules start in training mode; disable all dropout after the
        # replacement, preserving the source's trained dropout configuration.
        self.model.eval()

    @torch.inference_mode()
    def forward_selected(
        self,
        inputs: Tensor,
        positions: Tensor,
        *,
        execution: Execution = "plain",
        return_attention: bool = True,
    ) -> ZoologyModelOutput:
        if execution not in EXECUTIONS:
            raise ValueError(f"unsupported inference execution {execution!r}")
        if self._active:
            raise RuntimeError(
                "R4 inference wrapper does not permit concurrent or nested calls"
            )
        if any(module.training for module in self.model.modules()):
            raise RuntimeError("R4 Zoology inference requires complete model.eval()")
        self.model._validate_input_ids(inputs, self.model.config)
        self.model._validate_selected_positions(inputs, positions)
        frame_indices = self.frames.cumulative_frame_indices(inputs)
        self.last_audit = None
        self._active = True
        try:
            for adapter in self.adapters:
                adapter.execution = execution
                adapter.frame_indices = frame_indices
                adapter.audit = None
            result = self.model.forward_selected(
                inputs, positions, return_attention=return_attention
            )
            first = self.adapters[0].audit
            if first is None or any(adapter.audit is None for adapter in self.adapters):
                raise RuntimeError("not every layer executed its R4 inference seam")
            audit: dict[str, str | int | None | list[int]] = {
                "execution": execution,
                "batch_size": inputs.shape[0],
                "sequence_length": inputs.shape[1],
                "layers": len(self.adapters),
                "heads": self.model.config.num_heads,
                "r4_blocks_per_head": self.model.config.d_model
                // self.model.config.num_heads
                // 4,
                "reached_frame_indices": frame_indices.unique(sorted=True).tolist(),
            }
            for key in first:
                values = [adapter.audit[key] for adapter in self.adapters]
                audit[key] = (
                    None if any(value is None for value in values) else sum(values)
                )
            self.last_audit = audit
            return result
        finally:
            for adapter in self.adapters:
                adapter.execution = None
                adapter.frame_indices = None
            self._active = False
