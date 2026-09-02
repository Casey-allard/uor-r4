"""Position-preserving causal K/V attention with a coherent R4/H4 gauge lift.

This module is the frozen model-mechanics boundary for issue #1043.  It keeps
the qualified ordinary two-layer decoder weights unchanged while adding a
bounded, per-position incremental cache and a second execution of the same
attention law in local H4 frames.  The R4 path is a coherent change of basis,
not a separately fitted model.
"""

from __future__ import annotations

import math
from dataclasses import dataclass, replace
from typing import TYPE_CHECKING, Literal

import torch
from torch import Tensor
from torch.nn import functional as F

from .group_retention import GroupAddressArtifact
from .language_path_generalization import (
    CONTEXT,
    HEAD_DIM,
    HEADS,
    HIDDEN_SIZE,
    LAYERS,
    PARAMETER_COUNT,
    STATE_BYTES_F32,
    STATE_VALUES,
    VALIDITY_BITS,
    VOCAB_SIZE,
    OrdinaryCausalSoftmaxLanguagePathV1,
)

if TYPE_CHECKING:
    from .h4_spin_frame_sidecar import H4SpinFrameArtifactV1


POLICY = "R4PositionPreservingCausalKVBindingV1"
R4_WIDTH = 4
R4_BLOCKS_PER_HEAD = HEAD_DIM // R4_WIDTH

Execution = Literal["plain", "r4"]
Intervention = Literal[
    "native", "current_only", "value_permuted", "transport_mismatch"
]


@dataclass(frozen=True, slots=True)
class PositionKVBindingAudit:
    """Executed work and provenance ledger for one model call.

    Cache writes and value reads count scalar f32 lanes.  Admitted scores count
    query/source pairs after the selected causal intervention.  Source and
    target reads count token/label records, not tensor lanes.
    """

    execution: str
    intervention: str
    batch_size: int
    token_steps: int
    layers: int
    heads: int
    cache_writes: int
    materialized_attention_scores: int
    admitted_attention_scores: int
    transported_r4_blocks: int
    value_reads: int
    vocabulary_scores: int
    target_reads: int
    source_reads: int
    provider_calls: int = 0
    teacher_calls: int = 0
    future_reads: int = 0
    forbidden_reads: int = 0

    def work_signature(self) -> tuple[int, ...]:
        """Return the numerical work/provenance signature."""

        return (
            self.batch_size,
            self.token_steps,
            self.layers,
            self.heads,
            self.cache_writes,
            self.materialized_attention_scores,
            self.admitted_attention_scores,
            self.transported_r4_blocks,
            self.value_reads,
            self.vocabulary_scores,
            self.target_reads,
            self.source_reads,
            self.provider_calls,
            self.teacher_calls,
            self.future_reads,
            self.forbidden_reads,
        )

    def accumulated_with(self, later: PositionKVBindingAudit) -> PositionKVBindingAudit:
        """Accumulate two consecutive calls under one execution policy."""

        if (
            self.execution != later.execution
            or self.intervention != later.intervention
            or self.batch_size != later.batch_size
        ):
            raise ValueError("cannot accumulate unlike position-K/V audit policies")
        summed = {
            field: getattr(self, field) + getattr(later, field)
            for field in (
                "token_steps",
                "cache_writes",
                "materialized_attention_scores",
                "admitted_attention_scores",
                "transported_r4_blocks",
                "value_reads",
                "vocabulary_scores",
                "target_reads",
                "source_reads",
                "provider_calls",
                "teacher_calls",
                "future_reads",
                "forbidden_reads",
            )
        }
        return replace(self, **summed)


@dataclass(slots=True)
class PositionKVCacheState:
    """Exact bounded K/V cache for every layer and causal position.

    Keys are stored after RoPE and values are stored in model coordinates.
    Local R4 coordinates are derived at read time so the plain and geometric
    executions share exactly one cache meaning.
    """

    keys: Tensor
    values: Tensor
    valid: Tensor
    source_frame_indices: Tensor
    current_frame_indices: Tensor
    length: int
    audit: PositionKVBindingAudit


@dataclass(slots=True)
class PositionKVBindingOutput:
    logits: Tensor
    loss: Tensor | None
    final_state: PositionKVCacheState
    audit: PositionKVBindingAudit
    # [layers,batch,heads,new_time,context], with unadmitted slots exactly zero.
    attention_weights: Tensor


@dataclass(slots=True)
class PositionKVBindingStepOutput:
    logits: Tensor
    final_state: PositionKVCacheState
    audit: PositionKVBindingAudit
    # [layers,batch,heads,context], with unadmitted slots exactly zero.
    attention_weights: Tensor


class R4PositionPreservingCausalKVBindingV1(
    OrdinaryCausalSoftmaxLanguagePathV1
):
    """One fitted ordinary decoder with plain and coherent transported reads."""

    def __init__(
        self,
        geometry: GroupAddressArtifact,
        frames: H4SpinFrameArtifactV1,
    ) -> None:
        super().__init__()
        if geometry.arm != "exact_h4":
            raise ValueError("position-preserving R4 attention requires exact_h4 geometry")
        geometry.validate(group_size=120, vocab_size=VOCAB_SIZE)

        validate_frames = getattr(frames, "validate", None)
        if callable(validate_frames):
            validate_frames(group_size=120)
        frame_matrices = torch.as_tensor(
            frames.frame_matrices, dtype=torch.float32
        ).contiguous()
        multiplication = torch.as_tensor(
            frames.multiplication_indices, dtype=torch.long
        ).contiguous()
        permutation = torch.as_tensor(
            frames.transport_permutation, dtype=torch.long
        ).contiguous()
        if tuple(frame_matrices.shape) != (120, R4_WIDTH, R4_WIDTH):
            raise ValueError("H4 frame matrices must have shape [120,4,4]")
        if tuple(multiplication.shape) != (120, 120):
            raise ValueError("H4 multiplication table must have shape [120,120]")
        if tuple(permutation.shape) != (120,):
            raise ValueError("H4 transport permutation must have shape [120]")
        if int(frames.identity_index) != geometry.identity_offset:
            raise ValueError("H4 frame and group-address identities differ")
        if not torch.equal(multiplication.cpu(), geometry.left_actions.cpu()):
            raise ValueError("H4 frame and group-address multiplication tables differ")
        expected = torch.arange(120, dtype=torch.long)
        if not torch.equal(permutation.cpu().sort().values, expected):
            raise ValueError("H4 transport control must be a permutation")
        if int(permutation[geometry.identity_offset]) != geometry.identity_offset:
            raise ValueError("H4 transport control must fix the identity")
        if not bool(torch.isfinite(frame_matrices).all()):
            raise ValueError("H4 frames must be finite")
        eye = torch.eye(R4_WIDTH, dtype=torch.float32)
        orthogonality = torch.matmul(
            frame_matrices.transpose(-1, -2), frame_matrices
        )
        if not torch.allclose(orthogonality, eye, rtol=0.0, atol=3.0e-6):
            raise ValueError("H4 frames must be orthogonal in f32")

        self.identity_index = geometry.identity_offset
        self.geometry_artifact_cid = geometry.artifact_cid
        self.frame_artifact_cid = str(frames.artifact_cid)
        self.register_buffer(
            "token_leaves", geometry.token_leaves.detach().clone().contiguous()
        )
        self.register_buffer("frame_matrices", frame_matrices)
        self.register_buffer("frame_multiplication", multiplication)
        self.register_buffer("transport_permutation", permutation)

        if self.parameter_count() != PARAMETER_COUNT:
            raise RuntimeError("position-preserving K/V wrapper changed parameter count")
        if self.state_value_count() != STATE_VALUES:
            raise RuntimeError("position-preserving K/V state ledger drifted")
        if self.validity_bit_count() != VALIDITY_BITS:
            raise RuntimeError("position-preserving K/V validity ledger drifted")

    @classmethod
    def from_learned_artifact(
        cls,
        payload: bytes,
        *,
        geometry: GroupAddressArtifact,
        frames: H4SpinFrameArtifactV1,
    ) -> R4PositionPreservingCausalKVBindingV1:
        """Construct the policy and load the exact ordinary-arm artifact."""

        model = cls(geometry, frames)
        model.load_learned_artifact(payload)
        return model

    def state_byte_count_f32(self) -> int:
        """Return the exact f32 K/V value-byte budget per sequence."""

        return STATE_BYTES_F32

    @staticmethod
    def _empty_audit(
        *, batch_size: int, execution: Execution, intervention: Intervention
    ) -> PositionKVBindingAudit:
        return PositionKVBindingAudit(
            execution=execution,
            intervention=intervention,
            batch_size=batch_size,
            token_steps=0,
            layers=LAYERS,
            heads=HEADS,
            cache_writes=0,
            materialized_attention_scores=0,
            admitted_attention_scores=0,
            transported_r4_blocks=0,
            value_reads=0,
            vocabulary_scores=0,
            target_reads=0,
            source_reads=0,
        )

    def initial_state(
        self,
        batch_size: int,
        *,
        device: torch.device | str | None = None,
        dtype: torch.dtype | None = None,
        execution: Execution = "plain",
        intervention: Intervention = "native",
    ) -> PositionKVCacheState:
        """Create one empty, full-capacity cache without trainable state."""

        self._validate_policy(execution, intervention)
        if batch_size < 1:
            raise ValueError("batch size must be positive")
        resolved_device = self.output_weight.device if device is None else torch.device(device)
        resolved_dtype = self.output_weight.dtype if dtype is None else dtype
        keys = torch.zeros(
            LAYERS,
            batch_size,
            HEADS,
            CONTEXT,
            HEAD_DIM,
            device=resolved_device,
            dtype=resolved_dtype,
        )
        values = torch.zeros_like(keys)
        valid = torch.zeros(
            LAYERS,
            batch_size,
            CONTEXT,
            device=resolved_device,
            dtype=torch.bool,
        )
        source_frames = torch.full(
            (batch_size, CONTEXT),
            self.identity_index,
            device=resolved_device,
            dtype=torch.long,
        )
        current_frames = torch.full(
            (batch_size,),
            self.identity_index,
            device=resolved_device,
            dtype=torch.long,
        )
        return PositionKVCacheState(
            keys=keys,
            values=values,
            valid=valid,
            source_frame_indices=source_frames,
            current_frame_indices=current_frames,
            length=0,
            audit=self._empty_audit(
                batch_size=batch_size,
                execution=execution,
                intervention=intervention,
            ),
        )

    @staticmethod
    def _validate_policy(
        execution: Execution, intervention: Intervention
    ) -> None:
        if execution not in ("plain", "r4"):
            raise ValueError("execution must be 'plain' or 'r4'")
        if intervention not in (
            "native",
            "current_only",
            "value_permuted",
            "transport_mismatch",
        ):
            raise ValueError("unsupported position-K/V intervention")
        if execution != "r4" and intervention == "transport_mismatch":
            raise ValueError("transport_mismatch requires R4 execution")

    def _validate_state(
        self,
        state: PositionKVCacheState,
        *,
        batch_size: int,
        device: torch.device,
        execution: Execution,
        intervention: Intervention,
    ) -> None:
        expected_cache = (LAYERS, batch_size, HEADS, CONTEXT, HEAD_DIM)
        if tuple(state.keys.shape) != expected_cache or tuple(state.values.shape) != expected_cache:
            raise ValueError("position-K/V cache tensor shape differs from the frozen bound")
        if tuple(state.valid.shape) != (LAYERS, batch_size, CONTEXT):
            raise ValueError("position-K/V validity mask shape differs")
        if tuple(state.source_frame_indices.shape) != (batch_size, CONTEXT):
            raise ValueError("source-frame index shape differs")
        if tuple(state.current_frame_indices.shape) != (batch_size,):
            raise ValueError("current-frame index shape differs")
        tensors = (
            state.keys,
            state.values,
            state.valid,
            state.source_frame_indices,
            state.current_frame_indices,
        )
        if any(tensor.device != device for tensor in tensors):
            raise ValueError("tokens and position-K/V state must share a device")
        if state.keys.dtype != self.output_weight.dtype or state.values.dtype != self.output_weight.dtype:
            raise ValueError("position-K/V cache dtype must match learned weights")
        if state.valid.dtype != torch.bool:
            raise ValueError("position-K/V validity mask must be bool")
        if state.source_frame_indices.dtype != torch.long or state.current_frame_indices.dtype != torch.long:
            raise ValueError("position-K/V frame indices must be int64")
        if not 0 <= state.length <= CONTEXT:
            raise ValueError("position-K/V cache length is outside the frozen context")
        expected_valid = torch.arange(CONTEXT, device=device) < state.length
        if not torch.equal(
            state.valid,
            expected_valid.view(1, 1, CONTEXT).expand(LAYERS, batch_size, -1),
        ):
            raise ValueError("position-K/V validity mask is not the exact prefix mask")
        used_frames = state.source_frame_indices[:, : state.length]
        if bool((used_frames < 0).any()) or bool((used_frames >= 120).any()):
            raise ValueError("position-K/V cache contains an out-of-range source frame")
        if bool((state.current_frame_indices < 0).any()) or bool(
            (state.current_frame_indices >= 120).any()
        ):
            raise ValueError("position-K/V cache contains an out-of-range current frame")
        expected_current = (
            torch.full_like(state.current_frame_indices, self.identity_index)
            if state.length == 0
            else state.source_frame_indices[:, state.length - 1]
        )
        if not torch.equal(state.current_frame_indices, expected_current):
            raise ValueError("current frame does not equal the latest canonical source frame")
        if not bool(torch.isfinite(state.keys).all()) or not bool(
            torch.isfinite(state.values).all()
        ):
            raise ValueError("position-K/V cache contains a non-finite value")
        if (
            state.audit.batch_size != batch_size
            or state.audit.execution != execution
            or state.audit.intervention != intervention
        ):
            raise ValueError("position-K/V state audit policy differs from this call")

    @staticmethod
    def _heads(values: Tensor) -> Tensor:
        batch, time, _ = values.shape
        return values.view(batch, time, HEADS, HEAD_DIM).transpose(1, 2)

    @staticmethod
    def _r4_blocks(values: Tensor) -> Tensor:
        return values.view(*values.shape[:-1], R4_BLOCKS_PER_HEAD, R4_WIDTH)

    @staticmethod
    def _from_r4_blocks(values: Tensor) -> Tensor:
        return values.flatten(-2)

    @staticmethod
    def _prefix_derangement_indices(time: int, device: torch.device) -> Tensor:
        sources = torch.arange(time, device=device)
        lengths = torch.arange(1, time + 1, device=device)
        mapped = (sources.view(1, -1) + 1) % lengths.view(-1, 1)
        return torch.where(
            sources.view(1, -1) <= torch.arange(time, device=device).view(-1, 1),
            mapped,
            torch.zeros_like(mapped),
        )

    def _cumulative_frame_indices(
        self,
        token_ids: Tensor,
        initial_frames: Tensor | None = None,
    ) -> Tensor:
        batch, time = token_ids.shape
        current = (
            torch.full(
                (batch,),
                self.identity_index,
                device=token_ids.device,
                dtype=torch.long,
            )
            if initial_frames is None
            else initial_frames
        )
        leaves = self.token_leaves.index_select(0, token_ids.reshape(-1)).view(
            batch, time
        )
        steps: list[Tensor] = []
        for position in range(time):
            current = self.frame_multiplication[current, leaves[:, position]]
            steps.append(current)
        return torch.stack(steps, dim=1)

    @staticmethod
    def _target_reads(targets: Tensor | None) -> int:
        return 0 if targets is None else int(torch.count_nonzero(targets != -100))

    @staticmethod
    def _call_audit(
        *,
        execution: Execution,
        intervention: Intervention,
        batch_size: int,
        time: int,
        prior_length: int,
        target_reads: int,
        full_square: bool = False,
    ) -> PositionKVBindingAudit:
        token_steps = batch_size * time
        materialized = (
            batch_size * LAYERS * HEADS * time * time if full_square else 0
        )
        admitted = 0
        for offset in range(time):
            source_count = prior_length + offset + 1
            if not full_square:
                materialized += batch_size * LAYERS * HEADS * source_count
            admitted += batch_size * LAYERS * HEADS * (
                1 if intervention == "current_only" else source_count
            )
        transported = (
            materialized * 2 * R4_BLOCKS_PER_HEAD if execution == "r4" else 0
        )
        return PositionKVBindingAudit(
            execution=execution,
            intervention=intervention,
            batch_size=batch_size,
            token_steps=token_steps,
            layers=LAYERS,
            heads=HEADS,
            cache_writes=token_steps * LAYERS * 2 * HEADS * HEAD_DIM,
            materialized_attention_scores=materialized,
            admitted_attention_scores=admitted,
            transported_r4_blocks=transported,
            value_reads=materialized * HEAD_DIM,
            vocabulary_scores=token_steps * VOCAB_SIZE,
            target_reads=target_reads,
            source_reads=token_steps,
        )

    def _frames(self, indices: Tensor, *, dtype: torch.dtype) -> Tensor:
        selected = self.frame_matrices.index_select(0, indices.reshape(-1))
        return selected.view(*indices.shape, R4_WIDTH, R4_WIDTH).to(dtype=dtype)

    @staticmethod
    def _masked_scores(
        scores: Tensor, *, intervention: Intervention, time: int
    ) -> Tensor:
        future = torch.triu(
            torch.ones(time, time, dtype=torch.bool, device=scores.device), diagonal=1
        )
        if intervention == "current_only":
            future = ~torch.eye(time, dtype=torch.bool, device=scores.device)
        return scores.masked_fill(future.view(1, 1, time, time), float("-inf"))

    def _full_plain_attention(
        self,
        query: Tensor,
        key: Tensor,
        value: Tensor,
        *,
        intervention: Intervention,
        score_gains: Tensor,
    ) -> tuple[Tensor, Tensor]:
        time = int(query.shape[-2])
        scores = torch.matmul(query.float(), key.float().transpose(-2, -1))
        scores = scores / math.sqrt(HEAD_DIM)
        scores = scores * score_gains.exp().view(1, -1, 1, 1)
        scores = self._masked_scores(scores, intervention=intervention, time=time)
        weights = torch.softmax(scores, dim=-1, dtype=torch.float32)
        if intervention == "value_permuted":
            indices = self._prefix_derangement_indices(time, value.device)
            expanded = value[:, :, None, :, :].expand(-1, -1, time, -1, -1)
            gather = indices.view(1, 1, time, time, 1).expand(
                value.shape[0], HEADS, time, time, HEAD_DIM
            )
            selected = torch.gather(expanded, 3, gather)
            attended = torch.einsum("bhts,bhtsd->bhtd", weights, selected.float())
        else:
            attended = torch.matmul(weights, value.float())
        return attended.to(query.dtype), weights

    def _full_r4_attention(
        self,
        query: Tensor,
        key: Tensor,
        value: Tensor,
        frame_indices: Tensor,
        *,
        intervention: Intervention,
        score_gains: Tensor,
    ) -> tuple[Tensor, Tensor]:
        batch, _, time, _ = query.shape
        canonical_frames = self._frames(frame_indices, dtype=torch.float32)
        transport_indices = frame_indices
        if intervention == "transport_mismatch":
            transport_indices = self.transport_permutation.index_select(
                0, frame_indices.reshape(-1)
            ).view_as(frame_indices)
        transport_source_frames = self._frames(
            transport_indices, dtype=torch.float32
        )
        transport = torch.einsum(
            "btji,bsjk->btsik", canonical_frames, transport_source_frames
        )

        query_blocks = self._r4_blocks(query.float())
        key_blocks = self._r4_blocks(key.float())
        value_blocks = self._r4_blocks(value.float())
        query_local = torch.einsum(
            "btji,bhtdj->bhtdi", canonical_frames, query_blocks
        )
        key_local = torch.einsum(
            "bsji,bhsdj->bhsdi", canonical_frames, key_blocks
        )
        transported_key = torch.einsum(
            "btsij,bhsdj->bhtsdi", transport, key_local
        )
        scores = torch.einsum(
            "bhtdi,bhtsdi->bhts", query_local, transported_key
        )
        scores = scores / math.sqrt(HEAD_DIM)
        scores = scores * score_gains.exp().view(1, -1, 1, 1)
        scores = self._masked_scores(scores, intervention=intervention, time=time)
        weights = torch.softmax(scores, dim=-1, dtype=torch.float32)

        if intervention == "value_permuted":
            indices = self._prefix_derangement_indices(time, value.device)
            expanded = value_blocks[:, :, None, :, :, :].expand(
                -1, -1, time, -1, -1, -1
            )
            gather = indices.view(1, 1, time, time, 1, 1).expand(
                batch,
                HEADS,
                time,
                time,
                R4_BLOCKS_PER_HEAD,
                R4_WIDTH,
            )
            selected_values = torch.gather(expanded, 3, gather)
            value_local = torch.einsum(
                "bsji,bhtsdj->bhtsdi", canonical_frames, selected_values
            )
        else:
            value_local_at_source = torch.einsum(
                "bsji,bhsdj->bhsdi", canonical_frames, value_blocks
            )
            value_local = value_local_at_source[:, :, None, :, :, :].expand(
                -1, -1, time, -1, -1, -1
            )
        transported_value = torch.einsum(
            "btsij,bhtsdj->bhtsdi", transport, value_local
        )
        attended_local = torch.einsum(
            "bhts,bhtsdi->bhtdi", weights, transported_value
        )
        attended_model = torch.einsum(
            "btij,bhtdj->bhtdi", canonical_frames, attended_local
        )
        return self._from_r4_blocks(attended_model).to(query.dtype), weights

    def _full_block(
        self,
        layer: torch.nn.Module,
        values: Tensor,
        frame_indices: Tensor,
        *,
        execution: Execution,
        intervention: Intervention,
    ) -> tuple[Tensor, Tensor, Tensor, Tensor]:
        normalized = layer.input_layernorm(values)
        query = layer.rope(self._heads(layer.q_proj(normalized)))
        key = layer.rope(self._heads(layer.k_proj(normalized)))
        value = self._heads(layer.v_proj(normalized))
        if execution == "plain":
            attended, weights = self._full_plain_attention(
                query,
                key,
                value,
                intervention=intervention,
                score_gains=layer.log_score_gains,
            )
        else:
            attended, weights = self._full_r4_attention(
                query,
                key,
                value,
                frame_indices,
                intervention=intervention,
                score_gains=layer.log_score_gains,
            )
        attended = attended * layer.log_output_gains.exp().view(1, -1, 1, 1)
        attended = attended.transpose(1, 2).contiguous().view(values.shape)
        values = values + layer.o_proj(attended)
        values = values + layer.mlp(layer.post_attention_layernorm(values))
        return values, key, value, weights

    def _full_forward(
        self,
        token_ids: Tensor,
        targets: Tensor | None,
        *,
        execution: Execution,
        intervention: Intervention,
    ) -> PositionKVBindingOutput:
        batch, time = token_ids.shape
        frame_indices = self._cumulative_frame_indices(token_ids)
        state = self.initial_state(
            batch,
            device=token_ids.device,
            dtype=self.output_weight.dtype,
            execution=execution,
            intervention=intervention,
        )
        keys = state.keys.clone()
        cached_values = state.values.clone()
        valid = state.valid.clone()
        source_frames = state.source_frame_indices.clone()
        values = self.token_embedding(token_ids)
        layer_weights: list[Tensor] = []
        for layer_offset, layer in enumerate(self.layers):
            values, key, value, weights = self._full_block(
                layer,
                values,
                frame_indices,
                execution=execution,
                intervention=intervention,
            )
            keys[layer_offset, :, :, :time, :] = key
            cached_values[layer_offset, :, :, :time, :] = value
            valid[layer_offset, :, :time] = True
            padded = torch.zeros(
                batch,
                HEADS,
                time,
                CONTEXT,
                device=weights.device,
                dtype=weights.dtype,
            )
            padded[..., :time] = weights
            layer_weights.append(padded)
        source_frames[:, :time] = frame_indices
        values = self.final_norm(values)
        logits = F.linear(values, self.output_weight)
        loss = None
        if targets is not None:
            loss = F.cross_entropy(
                logits.float().reshape(-1, VOCAB_SIZE), targets.reshape(-1)
            )
        audit = self._call_audit(
            execution=execution,
            intervention=intervention,
            batch_size=batch,
            time=time,
            prior_length=0,
            target_reads=self._target_reads(targets),
            full_square=True,
        )
        final_state = PositionKVCacheState(
            keys=keys,
            values=cached_values,
            valid=valid,
            source_frame_indices=source_frames,
            current_frame_indices=frame_indices[:, -1],
            length=time,
            audit=audit,
        )
        return PositionKVBindingOutput(
            logits=logits,
            loss=loss,
            final_state=final_state,
            audit=audit,
            attention_weights=torch.stack(layer_weights, dim=0),
        )

    @staticmethod
    def _rope_at(layer: torch.nn.Module, values: Tensor, position: int) -> Tensor:
        half = values.shape[-1] // 2
        first, second = values[..., :half], values[..., half:]
        cosine = layer.rope.cosine[position].view(1, 1, half).to(values.device)
        sine = layer.rope.sine[position].view(1, 1, half).to(values.device)
        return torch.cat(
            (first * cosine - second * sine, second * cosine + first * sine),
            dim=-1,
        )

    def _step_plain_attention(
        self,
        query: Tensor,
        keys: Tensor,
        values: Tensor,
        *,
        intervention: Intervention,
        score_gains: Tensor,
    ) -> tuple[Tensor, Tensor]:
        scores = torch.einsum("bhd,bhsd->bhs", query.float(), keys.float())
        scores = scores / math.sqrt(HEAD_DIM)
        scores = scores * score_gains.exp().view(1, -1, 1)
        if intervention == "current_only":
            scores[..., :-1] = float("-inf")
        weights = torch.softmax(scores, dim=-1, dtype=torch.float32)
        selected_values = values
        if intervention == "value_permuted":
            count = int(values.shape[-2])
            indices = (torch.arange(count, device=values.device) + 1) % count
            selected_values = values.index_select(-2, indices)
        attended = torch.einsum(
            "bhs,bhsd->bhd", weights, selected_values.float()
        )
        return attended.to(query.dtype), weights

    def _step_r4_attention(
        self,
        query: Tensor,
        keys: Tensor,
        values: Tensor,
        source_frame_indices: Tensor,
        current_frame_indices: Tensor,
        *,
        intervention: Intervention,
        score_gains: Tensor,
    ) -> tuple[Tensor, Tensor]:
        batch, _, source_count, _ = keys.shape
        source_frames = self._frames(source_frame_indices, dtype=torch.float32)
        current_frames = self._frames(current_frame_indices, dtype=torch.float32)
        transport_indices = source_frame_indices
        if intervention == "transport_mismatch":
            transport_indices = self.transport_permutation.index_select(
                0, source_frame_indices.reshape(-1)
            ).view_as(source_frame_indices)
        transport_source = self._frames(transport_indices, dtype=torch.float32)
        transport = torch.einsum(
            "bji,bsjk->bsik", current_frames, transport_source
        )

        query_blocks = self._r4_blocks(query.float())
        key_blocks = self._r4_blocks(keys.float())
        value_blocks = self._r4_blocks(values.float())
        query_local = torch.einsum(
            "bji,bhdj->bhdi", current_frames, query_blocks
        )
        key_local = torch.einsum(
            "bsji,bhsdj->bhsdi", source_frames, key_blocks
        )
        transported_key = torch.einsum(
            "bsij,bhsdj->bhsdi", transport, key_local
        )
        scores = torch.einsum(
            "bhdi,bhsdi->bhs", query_local, transported_key
        )
        scores = scores / math.sqrt(HEAD_DIM)
        scores = scores * score_gains.exp().view(1, -1, 1)
        if intervention == "current_only":
            scores[..., :-1] = float("-inf")
        weights = torch.softmax(scores, dim=-1, dtype=torch.float32)

        selected_values = value_blocks
        if intervention == "value_permuted":
            indices = (
                torch.arange(source_count, device=values.device) + 1
            ) % source_count
            selected_values = value_blocks.index_select(-3, indices)
        value_local = torch.einsum(
            "bsji,bhsdj->bhsdi", source_frames, selected_values
        )
        transported_value = torch.einsum(
            "bsij,bhsdj->bhsdi", transport, value_local
        )
        attended_local = torch.einsum(
            "bhs,bhsdi->bhdi", weights, transported_value
        )
        attended_model = torch.einsum(
            "bij,bhdj->bhdi", current_frames, attended_local
        )
        return self._from_r4_blocks(attended_model).to(query.dtype), weights

    def step(
        self,
        token_ids: Tensor,
        state: PositionKVCacheState,
        *,
        execution: Execution = "plain",
        intervention: Intervention = "native",
    ) -> PositionKVBindingStepOutput:
        """Append one observed token and return its next-token logits."""

        self._validate_policy(execution, intervention)
        if token_ids.ndim != 1 or token_ids.dtype != torch.long:
            raise ValueError("incremental token_ids must be int64 [batch]")
        if bool((token_ids < 0).any()) or bool((token_ids >= VOCAB_SIZE).any()):
            raise ValueError("incremental token_ids contain an out-of-vocabulary value")
        batch = int(token_ids.shape[0])
        self._validate_state(
            state,
            batch_size=batch,
            device=token_ids.device,
            execution=execution,
            intervention=intervention,
        )
        if state.length >= CONTEXT:
            raise ValueError("position-K/V cache is full")

        position = state.length
        leaves = self.token_leaves.index_select(0, token_ids)
        current_frames = self.frame_multiplication[
            state.current_frame_indices, leaves
        ]
        keys = state.keys.clone()
        cached_values = state.values.clone()
        valid = state.valid.clone()
        source_frames = state.source_frame_indices.clone()
        source_frames[:, position] = current_frames

        values = self.token_embedding(token_ids)
        layer_weights: list[Tensor] = []
        for layer_offset, layer in enumerate(self.layers):
            normalized = layer.input_layernorm(values)
            query = layer.q_proj(normalized).view(batch, HEADS, HEAD_DIM)
            key = layer.k_proj(normalized).view(batch, HEADS, HEAD_DIM)
            value = layer.v_proj(normalized).view(batch, HEADS, HEAD_DIM)
            query = self._rope_at(layer, query, position)
            key = self._rope_at(layer, key, position)
            keys[layer_offset, :, :, position, :] = key
            cached_values[layer_offset, :, :, position, :] = value
            valid[layer_offset, :, position] = True
            admitted_keys = keys[layer_offset, :, :, : position + 1, :]
            admitted_values = cached_values[
                layer_offset, :, :, : position + 1, :
            ]
            if execution == "plain":
                attended, weights = self._step_plain_attention(
                    query,
                    admitted_keys,
                    admitted_values,
                    intervention=intervention,
                    score_gains=layer.log_score_gains,
                )
            else:
                attended, weights = self._step_r4_attention(
                    query,
                    admitted_keys,
                    admitted_values,
                    source_frames[:, : position + 1],
                    current_frames,
                    intervention=intervention,
                    score_gains=layer.log_score_gains,
                )
            attended = attended * layer.log_output_gains.exp().view(1, -1, 1)
            attended = attended.reshape(batch, HIDDEN_SIZE)
            values = values + layer.o_proj(attended)
            values = values + layer.mlp(layer.post_attention_layernorm(values))
            padded = torch.zeros(
                batch,
                HEADS,
                CONTEXT,
                device=weights.device,
                dtype=weights.dtype,
            )
            padded[..., : position + 1] = weights
            layer_weights.append(padded)
        logits = F.linear(self.final_norm(values), self.output_weight)
        call_audit = self._call_audit(
            execution=execution,
            intervention=intervention,
            batch_size=batch,
            time=1,
            prior_length=position,
            target_reads=0,
        )
        cumulative = state.audit.accumulated_with(call_audit)
        final_state = PositionKVCacheState(
            keys=keys,
            values=cached_values,
            valid=valid,
            source_frame_indices=source_frames,
            current_frame_indices=current_frames,
            length=position + 1,
            audit=cumulative,
        )
        return PositionKVBindingStepOutput(
            logits=logits,
            final_state=final_state,
            audit=call_audit,
            attention_weights=torch.stack(layer_weights, dim=0),
        )

    def forward_incremental(
        self,
        token_ids: Tensor,
        targets: Tensor | None = None,
        *,
        execution: Execution = "plain",
        intervention: Intervention = "native",
        initial_state: PositionKVCacheState | None = None,
    ) -> PositionKVBindingOutput:
        """Run the real cache-backed step path for a token block."""

        self._validate_policy(execution, intervention)
        self._validate_inputs(token_ids, targets)
        batch, time = token_ids.shape
        state = (
            self.initial_state(
                batch,
                device=token_ids.device,
                dtype=self.output_weight.dtype,
                execution=execution,
                intervention=intervention,
            )
            if initial_state is None
            else initial_state
        )
        self._validate_state(
            state,
            batch_size=batch,
            device=token_ids.device,
            execution=execution,
            intervention=intervention,
        )
        if state.length + time > CONTEXT:
            raise ValueError("token block exceeds the frozen position-K/V context")
        prior_audit = state.audit
        logits: list[Tensor] = []
        weights: list[Tensor] = []
        for position in range(time):
            output = self.step(
                token_ids[:, position],
                state,
                execution=execution,
                intervention=intervention,
            )
            state = output.final_state
            logits.append(output.logits)
            weights.append(output.attention_weights)
        stacked_logits = torch.stack(logits, dim=1)
        loss = None
        if targets is not None:
            loss = F.cross_entropy(
                stacked_logits.float().reshape(-1, VOCAB_SIZE), targets.reshape(-1)
            )
        call_audit = self._call_audit(
            execution=execution,
            intervention=intervention,
            batch_size=batch,
            time=time,
            prior_length=initial_state.length if initial_state is not None else 0,
            target_reads=self._target_reads(targets),
        )
        # Step calls have no labels; install the one block-level target census.
        cumulative = prior_audit.accumulated_with(call_audit)
        state.audit = cumulative
        return PositionKVBindingOutput(
            logits=stacked_logits,
            loss=loss,
            final_state=state,
            audit=call_audit,
            attention_weights=torch.stack(weights, dim=3),
        )

    def forward(
        self,
        token_ids: Tensor,
        targets: Tensor | None = None,
        *,
        execution: Execution = "plain",
        intervention: Intervention = "native",
        initial_state: PositionKVCacheState | None = None,
    ) -> PositionKVBindingOutput:
        """Run full-square attention, or continue an explicitly supplied cache."""

        self._validate_policy(execution, intervention)
        self._validate_inputs(token_ids, targets)
        if initial_state is not None:
            return self.forward_incremental(
                token_ids,
                targets,
                execution=execution,
                intervention=intervention,
                initial_state=initial_state,
            )
        return self._full_forward(
            token_ids,
            targets,
            execution=execution,
            intervention=intervention,
        )
