"""Inference-only token/role and compound R4 transport of the fixed interface."""

from __future__ import annotations

from typing import Literal

import torch
from torch import Tensor

from ..zoology_compound_r4.attention import _gauge_attention
from ..zoology_language_interface.model import LanguageInterfaceModel, _valid_tokens
from ..zoology_r4_inference.frames import R4InferenceFrames

Execution = Literal[
    "plain", "r4", "token_source_frame_permuted", "fact_source_frame_permuted"
]
EXECUTIONS = (
    "plain",
    "r4",
    "token_source_frame_permuted",
    "fact_source_frame_permuted",
)
# These describe support and actual geometric block operations. Control
# corruption counts are separate so all three geometric arms have equal work.
AUDIT_COUNTS = (
    "rows",
    "valid_tokens",
    "role_outputs",
    "role_scores_materialized",
    "admitted_role_scores",
    "binding_score_slots",
    "null_pairs",
    "token_blocks_encoded",
    "token_blocks_transported",
    "role_weighted_blocks",
    "role_blocks_decoded",
    "query_blocks_encoded",
    "key_blocks_encoded",
    "value_blocks_encoded",
    "key_blocks_transported",
    "value_blocks_transported",
    "output_blocks_decoded",
    "padding_blocks_encoded",
    "padding_blocks_transported",
    "future_input_reads",
)
_CONTROL_COUNTS = (
    "token_source_frame_positions_changed",
    "token_source_frame_matrices_changed",
    "fact_source_frame_positions_changed",
    "fact_source_frame_matrices_changed",
)
_FRAME_SETS = (
    "reached_token_frame_indices",
    "reached_clause_frame_indices",
    "reached_frame_indices",
)


def _input_mask(inputs: Tensor, lengths: Tensor) -> Tensor:
    if inputs.device.type != "cpu" or lengths.device.type != "cpu":
        raise ValueError("language R4 transport requires CPU clause inputs")
    return _valid_tokens(inputs, lengths)


def frame_assignment(
    inputs: Tensor, lengths: Tensor, frames: R4InferenceFrames
) -> tuple[Tensor, Tensor]:
    """Return token [B,5,L] and clause-end [B,5] native cumulative indices.

    Fold every valid token, including punctuation, continuously through the five
    clauses. Padding has the identity sentinel but is never looked up or folded.
    No BOS, separator, role position, or answer label is supplied or invented.
    """

    valid = _input_mask(inputs, lengths)
    current = torch.full((len(inputs),), frames.identity_index, dtype=torch.long)
    token_indices = torch.full_like(inputs, frames.identity_index)
    clause_indices = torch.empty_like(lengths)
    for clause in range(5):
        for position in range(inputs.shape[2]):
            active = valid[:, clause, position]
            if bool(active.any()):
                leaves = frames.token_leaf_indices[inputs[active, clause, position]]
                current[active] = frames.multiplication_indices[current[active], leaves]
                token_indices[active, clause, position] = current[active]
        clause_indices[:, clause] = current
    return token_indices, clause_indices


def work_counts(
    inputs: Tensor, lengths: Tensor, execution: Execution
) -> dict[str, int]:
    """Count support and geometric operations; plain keeps its source arithmetic.

    One encoded/transported token value is reused by all three role mixtures.
    Geometric weighted-block counts include all three roles in all five clauses,
    including the unused query-location role. Padding receives no such operation.
    """

    _input_mask(inputs, lengths)
    if execution not in EXECUTIONS:
        raise ValueError("unsupported language R4 execution")
    rows, _, length = inputs.shape
    tokens = int(lengths.sum())
    geometric = execution != "plain"
    counts = {name: 0 for name in AUDIT_COUNTS}
    counts.update(
        rows=rows,
        valid_tokens=tokens,
        role_outputs=15 * rows,
        role_scores_materialized=15 * rows * length,
        admitted_role_scores=3 * tokens,
        binding_score_slots=5 * rows,
        null_pairs=rows,
    )
    if geometric:
        counts.update(
            token_blocks_encoded=16 * tokens,
            token_blocks_transported=16 * tokens,
            role_weighted_blocks=48 * tokens,
            role_blocks_decoded=240 * rows,
            query_blocks_encoded=16 * rows,
            key_blocks_encoded=80 * rows,
            value_blocks_encoded=80 * rows,
            key_blocks_transported=80 * rows,
            value_blocks_transported=80 * rows,
            output_blocks_decoded=16 * rows,
        )
    return counts


def _pool_roles(
    model: LanguageInterfaceModel,
    inputs: Tensor,
    lengths: Tensor,
    role_attention: Tensor,
    token_indices: Tensor,
    clause_indices: Tensor,
    frames: R4InferenceFrames,
    *,
    permute_token_frames: bool,
) -> Tensor:
    """Pool every valid token in its clause gauge, then decode before source LN.

    Equal-length clause groups avoid encoding or transporting padded zeros. The
    unchanged reader coefficients remain f32 until their f64 weighted sum. True
    token encodings remain fixed even when the connection uses the next frame.
    """

    result = torch.empty((len(inputs), 5, 3, 64), dtype=torch.float32)
    for clause in range(5):
        for size in torch.unique(lengths[:, clause], sorted=True).tolist():
            rows = lengths[:, clause] == size
            token_frames = frames.frame_matrices[token_indices[rows, clause, :size]]
            clause_frames = frames.frame_matrices[clause_indices[rows, clause]]
            values = model.core.embedding(inputs[rows, clause, :size])
            blocks = values.double().reshape(-1, size, 16, 4)
            encoded = torch.einsum("btji,btdj->btdi", token_frames, blocks)
            transport_frames = (
                token_frames.roll(shifts=-1, dims=1)
                if permute_token_frames
                else token_frames
            )
            connection = torch.einsum("bji,btjk->btik", clause_frames, transport_frames)
            transported = torch.einsum("btij,btdj->btdi", connection, encoded)
            pooled = torch.einsum(
                "brt,btdi->brdi",
                role_attention[rows, clause, :, :size].double(),
                transported,
            )
            decoded = torch.einsum("bij,brdj->brdi", clause_frames, pooled)
            result[rows, clause] = decoded.reshape(-1, 3, 64).float()
    return result


class R4LanguageInterfaceInference:
    """Wrap the fixed interface without installing modules or changing state."""

    def __init__(
        self,
        model: LanguageInterfaceModel,
        frames: R4InferenceFrames,
        execution: Execution = "plain",
    ) -> None:
        if not isinstance(model, LanguageInterfaceModel):
            raise TypeError("language R4 requires the fixed language-interface model")
        if execution not in EXECUTIONS:
            raise ValueError("unsupported language R4 execution")
        if (
            frames.frame_matrices.shape != (120, 4, 4)
            or frames.frame_matrices.dtype != torch.float64
            or frames.frame_matrices.device.type != "cpu"
            or frames.token_leaf_indices.numel() != 8192
        ):
            raise ValueError(
                "language R4 requires native CPU f64 frames and 8192 leaves"
            )
        self.model, self.frames, self.execution = model, frames, execution
        self.core, self.reader = model.core, model.reader
        self._active = False
        self._validate_model()
        self.reset_audit()

    def _validate_model(self) -> None:
        if any(module.training for module in self.model.modules()):
            raise RuntimeError("language R4 requires complete model.eval()")
        if any(
            parameter.requires_grad
            or parameter.device.type != "cpu"
            or parameter.dtype != torch.float32
            for parameter in self.model.parameters()
        ):
            raise RuntimeError("language R4 requires frozen CPU f32 parameters")

    def reset_audit(self) -> None:
        self.audit: dict[str, int | list[int]] = {
            name: 0 for name in (*AUDIT_COUNTS, *_CONTROL_COUNTS)
        }
        self.audit.update({name: [] for name in _FRAME_SETS})

    def _record_audit(
        self,
        inputs: Tensor,
        lengths: Tensor,
        token_indices: Tensor,
        clause_indices: Tensor,
    ) -> None:
        for name, value in work_counts(inputs, lengths, self.execution).items():
            self.audit[name] += value
        valid = _input_mask(inputs, lengths)
        token_reached = set(token_indices[valid].tolist())
        clause_reached = set(clause_indices.flatten().tolist())
        for name, reached in (
            ("reached_token_frame_indices", token_reached),
            ("reached_clause_frame_indices", clause_reached),
            (
                "reached_frame_indices",
                token_reached | clause_reached | {self.frames.identity_index},
            ),
        ):
            self.audit[name] = sorted(set(self.audit[name]) | reached)
        if self.execution == "token_source_frame_permuted":
            positions = (torch.arange(inputs.shape[2]) + 1) % lengths.unsqueeze(-1)
            shifted = token_indices.gather(2, positions)
            self.audit["token_source_frame_positions_changed"] += int(
                (valid & (lengths.unsqueeze(-1) > 1)).sum()
            )
            self.audit["token_source_frame_matrices_changed"] += int(
                (
                    (
                        self.frames.frame_matrices[token_indices]
                        != self.frames.frame_matrices[shifted]
                    )
                    .any(dim=-1)
                    .any(dim=-1)
                    & valid
                ).sum()
            )
        if self.execution == "fact_source_frame_permuted":
            self.audit["fact_source_frame_positions_changed"] += 4 * len(inputs)
            self.audit["fact_source_frame_matrices_changed"] += int(
                (
                    self.frames.frame_matrices[clause_indices[:, :4]]
                    != self.frames.frame_matrices[clause_indices[:, [1, 2, 3, 0]]]
                )
                .any(dim=-1)
                .any(dim=-1)
                .sum()
            )

    @torch.inference_mode()
    def forward(
        self, inputs: Tensor, lengths: Tensor, *, control: str = "none"
    ) -> dict[str, Tensor]:
        if control != "none":
            raise ValueError("language R4 accepts only control='none'")
        if self.execution not in EXECUTIONS:
            raise ValueError("unsupported language R4 execution")
        if self._active:
            raise RuntimeError("language R4 does not permit nested forwards")
        self._validate_model()
        token_indices, clause_indices = frame_assignment(inputs, lengths, self.frames)
        self._active = True
        try:
            if self.execution == "plain":
                result = self.model(inputs, lengths)
            else:
                role_attention = self.reader(inputs, lengths)
                role_vectors = _pool_roles(
                    self.model,
                    inputs,
                    lengths,
                    role_attention,
                    token_indices,
                    clause_indices,
                    self.frames,
                    permute_token_frames=self.execution
                    == "token_source_frame_permuted",
                )
                rows = len(inputs)
                query = self.core.query_projection(
                    self.core.compound_norm(
                        role_vectors[:, 4, :2].reshape(rows, 1, 128)
                    )
                )
                keys = self.core.key_projection(
                    self.core.compound_norm(
                        role_vectors[:, :4, :2].reshape(rows, 4, 128)
                    )
                )
                values = self.core.value_projection(
                    self.core.location_norm(role_vectors[:, :4, 2])
                )
                keys = torch.cat((keys, self.core.null_key.expand(rows, 1, -1)), dim=1)
                values = torch.cat(
                    (values, self.core.null_value.expand(rows, 1, -1)), dim=1
                )
                source_indices = torch.cat(
                    (
                        clause_indices[:, :4],
                        torch.full(
                            (rows, 1), self.frames.identity_index, dtype=torch.long
                        ),
                    ),
                    dim=1,
                )
                context, weights = _gauge_attention(
                    query,
                    keys,
                    values,
                    self.frames.frame_matrices[clause_indices[:, 4]],
                    self.frames.frame_matrices[source_indices],
                    permute_source_frames=self.execution
                    == "fact_source_frame_permuted",
                )
                hidden = self.core.output_norm(self.core.output_projection(context))
                result = {
                    "logits": self.core.lm_head(hidden)[:, 0],
                    "binding_attention": weights[:, 0, 0],
                    "role_attention": role_attention,
                    "role_vectors": role_vectors,
                }
            self._record_audit(inputs, lengths, token_indices, clause_indices)
            return result
        finally:
            self._active = False

    def __call__(
        self, inputs: Tensor, lengths: Tensor, *, control: str = "none"
    ) -> dict[str, Tensor]:
        return self.forward(inputs, lengths, control=control)
