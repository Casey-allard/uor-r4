"""Inference-only R4 gauge transport of the frozen five-entry compound mixture."""

from __future__ import annotations

from typing import Literal

import torch
from torch import Tensor

from ..zoology_compound_binding.model import CompoundBindingModel
from ..zoology_control.model import ZoologyModelOutput
from ..zoology_r4_inference.frames import R4InferenceFrames

Execution = Literal["plain", "r4", "source_frame_permuted"]
EXECUTIONS = ("plain", "r4", "source_frame_permuted")
AUDIT_COUNTS = (
    "rows",
    "admitted_attention_pairs",
    "materialized_score_slots",
    "future_score_slots_materialized",
    "future_position_reads",
    "null_attention_pairs",
    "query_blocks_encoded",
    "key_blocks_encoded",
    "value_blocks_encoded",
    "key_blocks_transported",
    "value_blocks_transported",
    "output_blocks_decoded",
    "source_frame_positions_changed",
    "source_frame_matrices_changed",
)


def frame_assignment(
    inputs: Tensor, frames: R4InferenceFrames
) -> tuple[Tensor, Tensor]:
    """Return query [B,1] and source [B,5] indices using only input positions <=37.

    Fact frames end at their last lexical field (location); the final source is
    the learned null in the atlas identity frame, with no invented token source.
    This function does not need a model, targets, or development labels.
    """

    if (
        inputs.device.type != "cpu"
        or inputs.dtype != torch.long
        or inputs.ndim != 2
        or inputs.shape[0] < 1
        or inputs.shape[1] != 41
    ):
        raise ValueError("compound frame assignment requires CPU int64 [batch,41]")
    prefix = frames.cumulative_frame_indices(inputs[:, :38])
    query = prefix[:, 37:38]
    sources = torch.cat(
        (
            prefix[:, [7, 15, 23, 31]],
            torch.full((len(inputs), 1), frames.identity_index, dtype=torch.long),
        ),
        dim=1,
    )
    return query, sources


def _gauge_attention(
    query: Tensor,
    keys: Tensor,
    values: Tensor,
    query_frames: Tensor,
    source_frames: Tensor,
    *,
    permute_source_frames: bool,
) -> tuple[Tensor, Tensor]:
    """Transport all sixteen four-lane blocks of all five keys and values.

    Q is [B,1,64], K/V are [B,5,64]; frame matrices are native f64 [B,4,4]
    and [B,5,4,4]. The caller preserves the unchanged learned projections.
    """

    batch = query.shape[0]
    query_blocks = query.to(torch.float64).reshape(batch, 16, 4)
    key_blocks = keys.to(torch.float64).reshape(batch, 5, 16, 4)
    value_blocks = values.to(torch.float64).reshape(batch, 5, 16, 4)
    query_local = torch.einsum("bji,bdj->bdi", query_frames, query_blocks)
    key_local = torch.einsum("bsji,bsdj->bsdi", source_frames, key_blocks)
    value_local = torch.einsum("bsji,bsdj->bsdi", source_frames, value_blocks)
    transport_frames = (
        source_frames[:, [1, 2, 3, 0, 4]] if permute_source_frames else source_frames
    )
    connection = torch.einsum("bji,bsjk->bsik", query_frames, transport_frames)
    transported_keys = torch.einsum("bsij,bsdj->bsdi", connection, key_local)
    # The compound source rounds its full-width dot product to f32 before /8.
    # Unlike the older Zoology cell, it does not prescale keys before the dot.
    scores = (
        torch.einsum("bdi,bsdi->bs", query_local, transported_keys).to(torch.float32)
        / 8.0
    )
    weights = torch.softmax(scores, dim=-1, dtype=torch.float32)
    transported_values = torch.einsum("bsij,bsdj->bsdi", connection, value_local)
    attended_local = torch.einsum(
        "bs,bsdi->bdi", weights.to(torch.float64), transported_values
    )
    attended_model = torch.einsum("bij,bdj->bdi", query_frames, attended_local)
    return (
        attended_model.reshape(batch, 1, 64).to(torch.float32),
        weights.reshape(batch, 1, 1, 5),
    )


class R4CompoundInference:
    """Reuse the same learned model with a bounded rectangular transport seam.

    This wrapper installs no modules or parameters and mutates no source state.
    Audit values accumulate over successful batches until ``reset_audit``.
    """

    def __init__(
        self,
        model: CompoundBindingModel,
        frames: R4InferenceFrames,
        execution: Execution = "plain",
    ) -> None:
        if not isinstance(model, CompoundBindingModel):
            raise TypeError("R4 compound inference requires the frozen compound model")
        if execution not in EXECUTIONS:
            raise ValueError(f"unsupported compound R4 execution {execution!r}")
        if model.config.vocab_size > frames.token_leaf_indices.numel():
            raise ValueError("native token map does not cover the compound vocabulary")
        if (
            frames.frame_matrices.device.type != "cpu"
            or frames.frame_matrices.dtype != torch.float64
            or tuple(frames.frame_matrices.shape) != (120, 4, 4)
        ):
            raise ValueError("R4 compound inference requires native CPU f64 frames")
        self.model = model
        self.frames = frames
        self.execution = execution
        self._validate_model()
        self._active = False
        self.audit: dict[str, int | list[int]] = {}
        self.reset_audit()

    def _validate_model(self) -> None:
        if any(
            parameter.device.type != "cpu" or parameter.dtype != torch.float32
            for parameter in self.model.parameters()
        ):
            raise ValueError("R4 compound inference requires CPU f32 learned tensors")
        if any(module.training for module in self.model.modules()):
            raise RuntimeError("R4 compound inference requires complete model.eval()")

    def reset_audit(self) -> None:
        self.audit = {name: 0 for name in AUDIT_COUNTS}
        self.audit["reached_frame_indices"] = []

    def _record_audit(self, query_indices: Tensor, source_indices: Tensor) -> None:
        rows = len(query_indices)
        self.audit["rows"] += rows
        self.audit["admitted_attention_pairs"] += 5 * rows
        self.audit["materialized_score_slots"] += 5 * rows
        self.audit["null_attention_pairs"] += rows
        reached = set(self.audit["reached_frame_indices"])
        reached.update(query_indices.reshape(-1).tolist())
        reached.update(source_indices.reshape(-1).tolist())
        self.audit["reached_frame_indices"] = sorted(reached)
        if self.execution != "plain":
            for name in ("query_blocks_encoded", "output_blocks_decoded"):
                self.audit[name] += 16 * rows
            for name in (
                "key_blocks_encoded",
                "value_blocks_encoded",
                "key_blocks_transported",
                "value_blocks_transported",
            ):
                self.audit[name] += 80 * rows
        if self.execution == "source_frame_permuted":
            self.audit["source_frame_positions_changed"] += 4 * rows
            self.audit["source_frame_matrices_changed"] += int(
                torch.count_nonzero(
                    source_indices[:, :4] != source_indices[:, [1, 2, 3, 0]]
                )
            )

    @torch.inference_mode()
    def forward_selected(
        self,
        inputs: Tensor,
        positions: Tensor,
        return_attention: bool = True,
        *,
        control: str = "none",
    ) -> ZoologyModelOutput:
        if control != "none":
            raise ValueError("compound R4 inference accepts only control='none'")
        if not isinstance(return_attention, bool):
            raise TypeError(
                "return_attention must be bool; model labels are not accepted"
            )
        if self.execution not in EXECUTIONS:
            raise ValueError(f"unsupported compound R4 execution {self.execution!r}")
        if self._active:
            raise RuntimeError("compound R4 wrapper does not permit nested calls")
        self._validate_model()
        query_indices, source_indices = frame_assignment(inputs, self.frames)
        if (
            positions.device.type != "cpu"
            or positions.dtype != torch.long
            or positions.shape != (len(inputs), 1)
            or not bool((positions == 37).all())
        ):
            raise ValueError(
                "compound R4 inference requires selected position37 [batch,1]"
            )
        self._active = True
        try:
            if self.execution == "plain":
                result = self.model.forward_selected(
                    inputs, positions, return_attention=return_attention
                )
            else:
                # These unchanged source modules reproduce #1073's role
                # projections. Neither targets nor historical value_cycle enter.
                query_owner = self.model.embedding(inputs[:, 35:36])
                query_object = self.model.embedding(inputs[:, 37:38])
                query = self.model.query_projection(
                    self.model.compound_norm(
                        torch.cat((query_owner, query_object), dim=-1)
                    )
                )
                fact_owner = self.model.embedding(inputs[:, 1:33:8])
                fact_object = self.model.embedding(inputs[:, 4:33:8])
                keys = self.model.key_projection(
                    self.model.compound_norm(
                        torch.cat((fact_owner, fact_object), dim=-1)
                    )
                )
                values = self.model.value_projection(
                    self.model.location_norm(self.model.embedding(inputs[:, 7:33:8]))
                )
                rows = len(inputs)
                keys = torch.cat((keys, self.model.null_key.expand(rows, 1, -1)), dim=1)
                values = torch.cat(
                    (values, self.model.null_value.expand(rows, 1, -1)), dim=1
                )
                attended, weights = _gauge_attention(
                    query,
                    keys,
                    values,
                    self.frames.frame_matrices[query_indices[:, 0]],
                    self.frames.frame_matrices[source_indices],
                    permute_source_frames=self.execution == "source_frame_permuted",
                )
                hidden = self.model.output_norm(self.model.output_projection(attended))
                result = ZoologyModelOutput(
                    logits=self.model.lm_head(hidden),
                    loss=None,
                    hidden_states=hidden,
                    selected_positions=positions,
                    selected_targets=None,
                    attention_weights=(weights,) if return_attention else None,
                )
            self._record_audit(query_indices, source_indices)
            return result
        finally:
            self._active = False
