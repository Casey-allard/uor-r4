"""Fixed-size recurrent R4/H4 memory for the compact #973 language path.

The learned tokenizer and decoder tensors remain those of the accepted ordinary
causal-softmax artifact.  This module changes only inference memory: eight
chronological K/V records remain exact and evicted records enter four
hierarchical H4-framed summary banks.  A step reads the prior persistent state
plus its transient current K/V before committing that K/V, so state updates
cannot affect the logits that caused them.

Summary banks store K/V means in their own local H4 frame.  Binary carry merges
make the first three banks multirate; the final bank absorbs overflow so memory
stays fixed for every supported sequence length.  Counts weight only the
compression average; the existing Q/K score and softmax law remain unchanged.
This is an unfitted compression law, not a claim of language improvement or an
attention replacement: Q/K/V/O, softmax, RMSNorm, SwiGLU, and the vocabulary
head remain unchanged.
"""

from __future__ import annotations

import math
from dataclasses import dataclass, replace

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
    VOCAB_SIZE,
)
from .position_kv_binding import (
    R4_BLOCKS_PER_HEAD,
    R4_WIDTH,
    R4PositionPreservingCausalKVBindingV1,
)


POLICY = "R4FixedRecurrentCausalKVBindingV1"
LIVE_WINDOW = 8
SUMMARY_BANKS = 4
PERSISTENT_SOURCE_SLOTS = LIVE_WINDOW + SUMMARY_BANKS
MAXIMUM_READ_SOURCES = PERSISTENT_SOURCE_SLOTS + 1
RECURRENT_STATE_VALUES = (
    LAYERS * HEADS * PERSISTENT_SOURCE_SLOTS * HEAD_DIM * 2
)
RECURRENT_STATE_BYTES_F32 = RECURRENT_STATE_VALUES * 4
RECURRENT_METADATA_I64_VALUES = LIVE_WINDOW * 2 + SUMMARY_BANKS * 3 + 1


@dataclass(frozen=True, slots=True)
class FixedRecurrentKVBindingAudit:
    """Bounded work and causal provenance for recurrent inference."""

    policy: str
    batch_size: int
    token_steps: int
    layers: int
    heads: int
    live_window: int
    summary_banks: int
    cache_writes: int
    evictions: int
    summary_bank_updates: int
    summary_merges: int
    materialized_attention_scores: int
    admitted_attention_scores: int
    live_attention_scores: int
    summary_attention_scores: int
    current_attention_scores: int
    summary_slots_read: int
    attention_transported_r4_blocks: int
    compression_transported_r4_blocks: int
    value_reads: int
    vocabulary_scores: int
    source_reads: int
    peak_attention_source_slots: int
    provider_calls: int = 0
    teacher_calls: int = 0
    future_reads: int = 0
    forbidden_reads: int = 0

    def accumulated_with(
        self, later: FixedRecurrentKVBindingAudit
    ) -> FixedRecurrentKVBindingAudit:
        """Accumulate consecutive calls under the same fixed policy."""

        if (
            self.policy != later.policy
            or self.batch_size != later.batch_size
            or self.live_window != later.live_window
            or self.summary_banks != later.summary_banks
        ):
            raise ValueError("cannot accumulate unlike fixed-recurrent policies")
        summed = {
            field: getattr(self, field) + getattr(later, field)
            for field in (
                "token_steps",
                "cache_writes",
                "evictions",
                "summary_bank_updates",
                "summary_merges",
                "materialized_attention_scores",
                "admitted_attention_scores",
                "live_attention_scores",
                "summary_attention_scores",
                "current_attention_scores",
                "summary_slots_read",
                "attention_transported_r4_blocks",
                "compression_transported_r4_blocks",
                "value_reads",
                "vocabulary_scores",
                "source_reads",
                "provider_calls",
                "teacher_calls",
                "future_reads",
                "forbidden_reads",
            )
        }
        return replace(
            self,
            **summed,
            peak_attention_source_slots=max(
                self.peak_attention_source_slots,
                later.peak_attention_source_slots,
            ),
        )


@dataclass(slots=True)
class FixedRecurrentKVState:
    """One constant-size recurrent state.

    Live K/V tensors remain in model coordinates.  Summary K/V tensors are
    stored in the local coordinates named by ``summary_frame_indices``.
    Counts and positions are shared across layers and heads because every token
    updates every layer at the same causal step.
    """

    live_keys: Tensor
    live_values: Tensor
    live_frame_indices: Tensor
    live_positions: Tensor
    summary_keys_local: Tensor
    summary_values_local: Tensor
    summary_counts: Tensor
    summary_frame_indices: Tensor
    summary_last_positions: Tensor
    current_frame_indices: Tensor
    tokens_seen: int
    live_length: int
    audit: FixedRecurrentKVBindingAudit


@dataclass(slots=True)
class FixedRecurrentKVStepOutput:
    logits: Tensor
    final_state: FixedRecurrentKVState
    audit: FixedRecurrentKVBindingAudit
    # [layers,batch,heads,MAXIMUM_READ_SOURCES]
    attention_weights: Tensor


@dataclass(slots=True)
class FixedRecurrentKVBindingOutput:
    logits: Tensor
    loss: Tensor | None
    final_state: FixedRecurrentKVState
    audit: FixedRecurrentKVBindingAudit
    # [layers,batch,heads,time,MAXIMUM_READ_SOURCES]
    attention_weights: Tensor


class R4FixedRecurrentCausalKVBindingV1(
    R4PositionPreservingCausalKVBindingV1
):
    """Accepted ordinary weights with bounded hierarchical H4 K/V memory."""

    def __init__(
        self,
        geometry: GroupAddressArtifact,
        frames: object,
    ) -> None:
        super().__init__(geometry, frames)  # type: ignore[arg-type]
        if self.parameter_count() != PARAMETER_COUNT:
            raise RuntimeError("fixed recurrent wrapper changed learned parameters")

    @classmethod
    def from_learned_artifact(
        cls,
        payload: bytes,
        *,
        geometry: GroupAddressArtifact,
        frames: object,
    ) -> R4FixedRecurrentCausalKVBindingV1:
        model = cls(geometry, frames)
        model.load_learned_artifact(payload)
        return model

    @staticmethod
    def recurrent_state_value_count() -> int:
        return RECURRENT_STATE_VALUES

    @staticmethod
    def recurrent_state_byte_count_f32() -> int:
        return RECURRENT_STATE_BYTES_F32

    @staticmethod
    def recurrent_metadata_i64_value_count() -> int:
        return RECURRENT_METADATA_I64_VALUES

    @staticmethod
    def _empty_recurrent_audit(batch_size: int) -> FixedRecurrentKVBindingAudit:
        return FixedRecurrentKVBindingAudit(
            policy=POLICY,
            batch_size=batch_size,
            token_steps=0,
            layers=LAYERS,
            heads=HEADS,
            live_window=LIVE_WINDOW,
            summary_banks=SUMMARY_BANKS,
            cache_writes=0,
            evictions=0,
            summary_bank_updates=0,
            summary_merges=0,
            materialized_attention_scores=0,
            admitted_attention_scores=0,
            live_attention_scores=0,
            summary_attention_scores=0,
            current_attention_scores=0,
            summary_slots_read=0,
            attention_transported_r4_blocks=0,
            compression_transported_r4_blocks=0,
            value_reads=0,
            vocabulary_scores=0,
            source_reads=0,
            peak_attention_source_slots=0,
        )

    def initial_recurrent_state(
        self,
        batch_size: int,
        *,
        device: torch.device | str | None = None,
        dtype: torch.dtype | None = None,
    ) -> FixedRecurrentKVState:
        """Create an empty fixed-size cache without trainable state."""

        if batch_size < 1:
            raise ValueError("batch size must be positive")
        resolved_device = (
            self.output_weight.device if device is None else torch.device(device)
        )
        resolved_dtype = self.output_weight.dtype if dtype is None else dtype
        live_keys = torch.zeros(
            LAYERS,
            batch_size,
            HEADS,
            LIVE_WINDOW,
            HEAD_DIM,
            device=resolved_device,
            dtype=resolved_dtype,
        )
        summary_keys = torch.zeros(
            LAYERS,
            batch_size,
            HEADS,
            SUMMARY_BANKS,
            HEAD_DIM,
            device=resolved_device,
            dtype=resolved_dtype,
        )
        return FixedRecurrentKVState(
            live_keys=live_keys,
            live_values=torch.zeros_like(live_keys),
            live_frame_indices=torch.full(
                (batch_size, LIVE_WINDOW),
                self.identity_index,
                device=resolved_device,
                dtype=torch.long,
            ),
            live_positions=torch.full(
                (batch_size, LIVE_WINDOW),
                -1,
                device=resolved_device,
                dtype=torch.long,
            ),
            summary_keys_local=summary_keys,
            summary_values_local=torch.zeros_like(summary_keys),
            summary_counts=torch.zeros(
                batch_size,
                SUMMARY_BANKS,
                device=resolved_device,
                dtype=torch.long,
            ),
            summary_frame_indices=torch.full(
                (batch_size, SUMMARY_BANKS),
                self.identity_index,
                device=resolved_device,
                dtype=torch.long,
            ),
            summary_last_positions=torch.full(
                (batch_size, SUMMARY_BANKS),
                -1,
                device=resolved_device,
                dtype=torch.long,
            ),
            current_frame_indices=torch.full(
                (batch_size,),
                self.identity_index,
                device=resolved_device,
                dtype=torch.long,
            ),
            tokens_seen=0,
            live_length=0,
            audit=self._empty_recurrent_audit(batch_size),
        )

    def _validate_recurrent_state(
        self,
        state: FixedRecurrentKVState,
        *,
        batch_size: int,
        device: torch.device,
    ) -> None:
        live_shape = (LAYERS, batch_size, HEADS, LIVE_WINDOW, HEAD_DIM)
        summary_shape = (LAYERS, batch_size, HEADS, SUMMARY_BANKS, HEAD_DIM)
        if (
            tuple(state.live_keys.shape) != live_shape
            or tuple(state.live_values.shape) != live_shape
            or tuple(state.summary_keys_local.shape) != summary_shape
            or tuple(state.summary_values_local.shape) != summary_shape
        ):
            raise ValueError("fixed-recurrent K/V tensor shape differs")
        expected_shapes = {
            "live_frame_indices": (batch_size, LIVE_WINDOW),
            "live_positions": (batch_size, LIVE_WINDOW),
            "summary_counts": (batch_size, SUMMARY_BANKS),
            "summary_frame_indices": (batch_size, SUMMARY_BANKS),
            "summary_last_positions": (batch_size, SUMMARY_BANKS),
            "current_frame_indices": (batch_size,),
        }
        for name, shape in expected_shapes.items():
            if tuple(getattr(state, name).shape) != shape:
                raise ValueError(f"fixed-recurrent {name} shape differs")
        tensors = (
            state.live_keys,
            state.live_values,
            state.live_frame_indices,
            state.live_positions,
            state.summary_keys_local,
            state.summary_values_local,
            state.summary_counts,
            state.summary_frame_indices,
            state.summary_last_positions,
            state.current_frame_indices,
        )
        if any(tensor.device != device for tensor in tensors):
            raise ValueError("tokens and fixed-recurrent state must share a device")
        if (
            state.live_keys.dtype != self.output_weight.dtype
            or state.live_values.dtype != self.output_weight.dtype
            or state.summary_keys_local.dtype != self.output_weight.dtype
            or state.summary_values_local.dtype != self.output_weight.dtype
        ):
            raise ValueError("fixed-recurrent K/V dtype must match learned weights")
        for name in (
            "live_frame_indices",
            "live_positions",
            "summary_counts",
            "summary_frame_indices",
            "summary_last_positions",
            "current_frame_indices",
        ):
            if getattr(state, name).dtype != torch.long:
                raise ValueError(f"fixed-recurrent {name} must be int64")
        if not 0 <= state.tokens_seen <= CONTEXT:
            raise ValueError(
                "fixed-recurrent position exceeds the trained RoPE context"
            )
        if state.live_length != min(state.tokens_seen, LIVE_WINDOW):
            raise ValueError("fixed-recurrent live length disagrees with tokens seen")
        expected_positions = torch.arange(
            state.tokens_seen - state.live_length,
            state.tokens_seen,
            device=device,
            dtype=torch.long,
        )
        if state.live_length and not torch.equal(
            state.live_positions[:, : state.live_length],
            expected_positions.view(1, -1).expand(batch_size, -1),
        ):
            raise ValueError("fixed-recurrent live positions are not chronological")
        if bool((state.summary_counts < 0).any()):
            raise ValueError("fixed-recurrent summary count is negative")
        represented = state.summary_counts.sum(dim=1)
        expected_represented = torch.full_like(
            represented, state.tokens_seen - state.live_length
        )
        if not torch.equal(represented, expected_represented):
            raise ValueError("fixed-recurrent summaries do not conserve token count")
        occupied = state.summary_counts > 0
        if not torch.equal(
            occupied,
            occupied[0].view(1, -1).expand(batch_size, -1),
        ):
            raise ValueError(
                "fixed-recurrent batch lanes have different summary levels"
            )
        for bank in range(SUMMARY_BANKS - 1):
            bank_counts = state.summary_counts[:, bank]
            if bool(
                torch.any(
                    (bank_counts != 0) & (bank_counts != (1 << bank))
                )
            ):
                raise ValueError("fixed-recurrent lower summary count is not binary")
        highest_counts = state.summary_counts[:, SUMMARY_BANKS - 1]
        highest_unit = 1 << (SUMMARY_BANKS - 1)
        if bool(
            torch.any(
                (highest_counts != 0) & ((highest_counts % highest_unit) != 0)
            )
        ):
            raise ValueError("fixed-recurrent highest summary count is not aligned")
        empty = ~occupied
        if bool(
            (
                state.summary_frame_indices[empty] != self.identity_index
            ).any()
        ):
            raise ValueError("empty recurrent summary carries a nonidentity frame")
        summary_by_slot = state.summary_keys_local.permute(1, 3, 0, 2, 4)
        values_by_slot = state.summary_values_local.permute(1, 3, 0, 2, 4)
        if bool((summary_by_slot[empty] != 0).any()) or bool(
            (values_by_slot[empty] != 0).any()
        ):
            raise ValueError("empty recurrent summary carries K/V state")
        if bool((state.summary_last_positions[occupied] < 0).any()):
            raise ValueError("occupied recurrent summary has no age position")
        if bool((state.summary_last_positions[~occupied] != -1).any()):
            raise ValueError("empty recurrent summary carries an age position")
        if state.live_length:
            oldest_live = state.tokens_seen - state.live_length
            if bool((state.summary_last_positions[occupied] >= oldest_live).any()):
                raise ValueError("recurrent summary overlaps the exact live window")
        frame_tensors = (
            state.live_frame_indices[:, : state.live_length],
            state.summary_frame_indices[occupied],
            state.current_frame_indices,
        )
        if any(
            bool((values < 0).any()) or bool((values >= 120).any())
            for values in frame_tensors
        ):
            raise ValueError("fixed-recurrent state contains an invalid H4 frame")
        expected_current = (
            torch.full_like(state.current_frame_indices, self.identity_index)
            if state.tokens_seen == 0
            else state.live_frame_indices[:, state.live_length - 1]
        )
        if not torch.equal(state.current_frame_indices, expected_current):
            raise ValueError("fixed-recurrent current frame is not the latest frame")
        for tensor in (
            state.live_keys,
            state.live_values,
            state.summary_keys_local,
            state.summary_values_local,
        ):
            if not bool(torch.isfinite(tensor).all()):
                raise ValueError("fixed-recurrent state contains a non-finite value")
        if state.audit.policy != POLICY or state.audit.batch_size != batch_size:
            raise ValueError("fixed-recurrent audit policy differs")

    def _model_to_local(self, values: Tensor, frame_indices: Tensor) -> Tensor:
        frames = self._frames(frame_indices, dtype=values.dtype)
        blocks = self._r4_blocks(values)
        if frame_indices.ndim == 0:
            local = torch.einsum("ji,...dj->...di", frames, blocks)
        elif frame_indices.ndim == 1 and values.shape[0] == frame_indices.shape[0]:
            local = torch.einsum("aji,ahdj->ahdi", frames, blocks)
        else:
            raise ValueError("frame indices do not align with model-coordinate values")
        return self._from_r4_blocks(local)

    def _transport_local(
        self,
        values_local: Tensor,
        source_frame_index: Tensor,
        destination_frame_index: Tensor,
    ) -> Tensor:
        source = self._frames(source_frame_index.view(1), dtype=values_local.dtype)[0]
        destination = self._frames(
            destination_frame_index.view(1), dtype=values_local.dtype
        )[0]
        transport = torch.einsum("ji,jk->ik", destination, source)
        blocks = self._r4_blocks(values_local)
        return self._from_r4_blocks(
            torch.einsum("ij,...dj->...di", transport, blocks)
        )

    def _merge_local_summaries(
        self,
        older_local: Tensor,
        older_count: int,
        older_frame_index: Tensor,
        newer_local: Tensor,
        newer_count: int,
        newer_frame_index: Tensor,
    ) -> Tensor:
        older_in_newer = self._transport_local(
            older_local, older_frame_index, newer_frame_index
        )
        total = older_count + newer_count
        return (
            older_in_newer * (older_count / total)
            + newer_local * (newer_count / total)
        )

    def _fold_evicted(
        self,
        summary_keys: Tensor,
        summary_values: Tensor,
        summary_counts: Tensor,
        summary_frames: Tensor,
        summary_last_positions: Tensor,
        evicted_keys: Tensor,
        evicted_values: Tensor,
        evicted_frames: Tensor,
        evicted_positions: Tensor,
    ) -> tuple[int, int, int]:
        """Fold one evicted record per batch by binary carry.

        Returns ``(bank_updates, merges, transported_blocks)``.
        """

        batch_size = int(evicted_keys.shape[1])
        bank_updates = 0
        merges = 0
        transported_blocks = 0
        for batch_offset in range(batch_size):
            frame = evicted_frames[batch_offset]
            carry_key = self._model_to_local(
                evicted_keys[:, batch_offset], frame
            )
            carry_value = self._model_to_local(
                evicted_values[:, batch_offset], frame
            )
            carry_count = 1
            carry_last_position = evicted_positions[batch_offset]
            transported_blocks += 2 * LAYERS * HEADS * R4_BLOCKS_PER_HEAD
            for bank in range(SUMMARY_BANKS):
                existing_count = int(summary_counts[batch_offset, bank])
                bank_updates += 1
                if existing_count == 0:
                    summary_keys[:, batch_offset, :, bank, :] = carry_key
                    summary_values[:, batch_offset, :, bank, :] = carry_value
                    summary_counts[batch_offset, bank] = carry_count
                    summary_frames[batch_offset, bank] = frame
                    summary_last_positions[
                        batch_offset, bank
                    ] = carry_last_position
                    break

                carry_key = self._merge_local_summaries(
                    summary_keys[:, batch_offset, :, bank, :],
                    existing_count,
                    summary_frames[batch_offset, bank],
                    carry_key,
                    carry_count,
                    frame,
                )
                carry_value = self._merge_local_summaries(
                    summary_values[:, batch_offset, :, bank, :],
                    existing_count,
                    summary_frames[batch_offset, bank],
                    carry_value,
                    carry_count,
                    frame,
                )
                transported_blocks += 2 * LAYERS * HEADS * R4_BLOCKS_PER_HEAD
                carry_count += existing_count
                merges += 1
                if bank + 1 < SUMMARY_BANKS:
                    summary_keys[:, batch_offset, :, bank, :].zero_()
                    summary_values[:, batch_offset, :, bank, :].zero_()
                    summary_counts[batch_offset, bank] = 0
                    summary_frames[batch_offset, bank] = self.identity_index
                    summary_last_positions[batch_offset, bank] = -1
                    continue

                summary_keys[:, batch_offset, :, bank, :] = carry_key
                summary_values[:, batch_offset, :, bank, :] = carry_value
                summary_counts[batch_offset, bank] = carry_count
                summary_frames[batch_offset, bank] = frame
                summary_last_positions[batch_offset, bank] = carry_last_position
        return bank_updates, merges, transported_blocks

    def _fixed_r4_attention(
        self,
        query: Tensor,
        current_key: Tensor,
        current_value: Tensor,
        state: FixedRecurrentKVState,
        layer_offset: int,
        current_frames: Tensor,
        score_gains: Tensor,
    ) -> tuple[Tensor, Tensor, int, int, int]:
        batch_size = int(query.shape[0])
        summary_order = torch.arange(
            SUMMARY_BANKS - 1,
            -1,
            -1,
            device=query.device,
        )
        summary_keys_local = state.summary_keys_local[layer_offset].index_select(
            2, summary_order
        )
        summary_values_local = state.summary_values_local[layer_offset].index_select(
            2, summary_order
        )
        summary_frames = state.summary_frame_indices.index_select(1, summary_order)
        summary_counts = state.summary_counts.index_select(1, summary_order)
        summary_valid = summary_counts > 0
        occupied_offsets = torch.nonzero(
            summary_valid[0], as_tuple=False
        ).flatten()
        summary_keys_local = summary_keys_local.index_select(2, occupied_offsets)
        summary_values_local = summary_values_local.index_select(
            2, occupied_offsets
        )
        summary_frames = summary_frames.index_select(1, occupied_offsets)

        live_keys = state.live_keys[
            layer_offset, ..., : state.live_length, :
        ]
        live_values = state.live_values[
            layer_offset, ..., : state.live_length, :
        ]
        live_frame_indices = state.live_frame_indices[:, : state.live_length]
        if occupied_offsets.numel() == 0:
            exact_keys = torch.cat(
                (live_keys, current_key.unsqueeze(2)), dim=2
            )
            exact_values = torch.cat(
                (live_values, current_value.unsqueeze(2)), dim=2
            )
            exact_frames = torch.cat(
                (live_frame_indices, current_frames.unsqueeze(1)), dim=1
            )
            attended, weights = self._step_r4_attention(
                query,
                exact_keys,
                exact_values,
                exact_frames,
                current_frames,
                intervention="native",
                score_gains=score_gains,
            )
            live_sources = batch_size * state.live_length
            return (
                attended,
                weights,
                live_sources + batch_size,
                live_sources,
                0,
            )

        live_frames = self._frames(live_frame_indices, dtype=torch.float32)
        live_keys_local = self._from_r4_blocks(
            torch.einsum(
                "bsji,bhsdj->bhsdi",
                live_frames,
                self._r4_blocks(live_keys.float()),
            )
        )
        live_values_local = self._from_r4_blocks(
            torch.einsum(
                "bsji,bhsdj->bhsdi",
                live_frames,
                self._r4_blocks(live_values.float()),
            )
        )
        current_key_local = self._model_to_local(current_key.float(), current_frames)
        current_value_local = self._model_to_local(
            current_value.float(), current_frames
        )

        source_keys_local = torch.cat(
            (
                summary_keys_local.float(),
                live_keys_local,
                current_key_local.unsqueeze(2),
            ),
            dim=2,
        )
        source_values_local = torch.cat(
            (
                summary_values_local.float(),
                live_values_local,
                current_value_local.unsqueeze(2),
            ),
            dim=2,
        )
        source_frames = torch.cat(
            (
                summary_frames,
                live_frame_indices,
                current_frames.unsqueeze(1),
            ),
            dim=1,
        )

        current_frame_matrices = self._frames(current_frames, dtype=torch.float32)
        source_frame_matrices = self._frames(source_frames, dtype=torch.float32)
        transport = torch.einsum(
            "bji,bsjk->bsik", current_frame_matrices, source_frame_matrices
        )
        query_local = torch.einsum(
            "bji,bhdj->bhdi",
            current_frame_matrices,
            self._r4_blocks(query.float()),
        )
        transported_keys = torch.einsum(
            "bsij,bhsdj->bhsdi",
            transport,
            self._r4_blocks(source_keys_local),
        )
        scores = torch.einsum("bhdi,bhsdi->bhs", query_local, transported_keys)
        scores = scores / math.sqrt(HEAD_DIM)
        scores = scores * score_gains.exp().view(1, -1, 1)
        weights = torch.softmax(scores, dim=-1, dtype=torch.float32)

        transported_values = torch.einsum(
            "bsij,bhsdj->bhsdi",
            transport,
            self._r4_blocks(source_values_local),
        )
        attended_local = torch.einsum(
            "bhs,bhsdi->bhdi", weights, transported_values
        )
        attended_model = torch.einsum(
            "bij,bhdj->bhdi", current_frame_matrices, attended_local
        )

        summary_sources = batch_size * int(occupied_offsets.numel())
        live_sources = batch_size * state.live_length
        admitted_sources = summary_sources + live_sources + batch_size
        return (
            self._from_r4_blocks(attended_model).to(query.dtype),
            weights,
            admitted_sources,
            live_sources,
            summary_sources,
        )

    def step_recurrent(
        self,
        token_ids: Tensor,
        state: FixedRecurrentKVState,
    ) -> FixedRecurrentKVStepOutput:
        """Read prior fixed state, produce logits, then persist one observed token."""

        if token_ids.ndim != 1 or token_ids.dtype != torch.long:
            raise ValueError("incremental token_ids must be int64 [batch]")
        if bool((token_ids < 0).any()) or bool((token_ids >= VOCAB_SIZE).any()):
            raise ValueError("incremental token_ids contain an out-of-vocabulary value")
        batch_size = int(token_ids.shape[0])
        self._validate_recurrent_state(
            state, batch_size=batch_size, device=token_ids.device
        )
        if state.tokens_seen >= CONTEXT:
            raise ValueError(
                "fixed-recurrent sequence exceeds the trained RoPE context"
            )

        position = state.tokens_seen
        leaves = self.token_leaves.index_select(0, token_ids)
        current_frames = self.frame_multiplication[
            state.current_frame_indices, leaves
        ]
        values = self.token_embedding(token_ids)
        current_keys: list[Tensor] = []
        current_values: list[Tensor] = []
        layer_weights: list[Tensor] = []
        admitted_sources_total = 0
        live_sources_total = 0
        summary_sources_total = 0

        # The persistent state is not mutated in this loop.  Current K/V is a
        # transient causal source and is committed only after all logits exist.
        for layer_offset, layer in enumerate(self.layers):
            normalized = layer.input_layernorm(values)
            query = layer.q_proj(normalized).view(batch_size, HEADS, HEAD_DIM)
            key = layer.k_proj(normalized).view(batch_size, HEADS, HEAD_DIM)
            value = layer.v_proj(normalized).view(batch_size, HEADS, HEAD_DIM)
            query = self._rope_at(layer, query, position)
            key = self._rope_at(layer, key, position)
            attended, weights, admitted, live_sources, summary_sources = (
                self._fixed_r4_attention(
                    query,
                    key,
                    value,
                    state,
                    layer_offset,
                    current_frames,
                    layer.log_score_gains,
                )
            )
            admitted_sources_total += admitted
            live_sources_total += live_sources
            summary_sources_total += summary_sources
            attended = attended * layer.log_output_gains.exp().view(1, -1, 1)
            values = values + layer.o_proj(attended.reshape(batch_size, HIDDEN_SIZE))
            values = values + layer.mlp(layer.post_attention_layernorm(values))
            current_keys.append(key)
            current_values.append(value)
            padded_weights = torch.zeros(
                batch_size,
                HEADS,
                MAXIMUM_READ_SOURCES,
                device=weights.device,
                dtype=weights.dtype,
            )
            padded_weights[..., : weights.shape[-1]] = weights
            layer_weights.append(padded_weights)

        logits = F.linear(self.final_norm(values), self.output_weight)
        staged_keys = torch.stack(current_keys, dim=0)
        staged_values = torch.stack(current_values, dim=0)

        live_keys = state.live_keys.clone()
        live_values = state.live_values.clone()
        live_frames = state.live_frame_indices.clone()
        live_positions = state.live_positions.clone()
        summary_keys = state.summary_keys_local.clone()
        summary_values = state.summary_values_local.clone()
        summary_counts = state.summary_counts.clone()
        summary_frames = state.summary_frame_indices.clone()
        summary_last_positions = state.summary_last_positions.clone()

        evictions = 0
        bank_updates = 0
        summary_merges = 0
        fold_transports = 0
        if state.live_length == LIVE_WINDOW:
            evictions = batch_size
            bank_updates, summary_merges, fold_transports = self._fold_evicted(
                summary_keys,
                summary_values,
                summary_counts,
                summary_frames,
                summary_last_positions,
                live_keys[..., 0, :].clone(),
                live_values[..., 0, :].clone(),
                live_frames[:, 0].clone(),
                live_positions[:, 0].clone(),
            )
            live_keys[..., :-1, :] = live_keys[..., 1:, :].clone()
            live_values[..., :-1, :] = live_values[..., 1:, :].clone()
            live_frames[:, :-1] = live_frames[:, 1:].clone()
            live_positions[:, :-1] = live_positions[:, 1:].clone()
            insertion = LIVE_WINDOW - 1
            live_length = LIVE_WINDOW
        else:
            insertion = state.live_length
            live_length = state.live_length + 1

        live_keys[..., insertion, :] = staged_keys
        live_values[..., insertion, :] = staged_values
        live_frames[:, insertion] = current_frames
        live_positions[:, insertion] = position

        admitted_scores = admitted_sources_total * HEADS
        live_scores = live_sources_total * HEADS
        summary_scores = summary_sources_total * HEADS
        current_scores = batch_size * LAYERS * HEADS
        materialized = admitted_scores
        peak_sources = state.live_length + int(
            torch.count_nonzero(state.summary_counts[0])
        ) + 1
        call_audit = FixedRecurrentKVBindingAudit(
            policy=POLICY,
            batch_size=batch_size,
            token_steps=batch_size,
            layers=LAYERS,
            heads=HEADS,
            live_window=LIVE_WINDOW,
            summary_banks=SUMMARY_BANKS,
            cache_writes=batch_size * LAYERS * 2 * HEADS * HEAD_DIM,
            evictions=evictions,
            summary_bank_updates=bank_updates,
            summary_merges=summary_merges,
            materialized_attention_scores=materialized,
            admitted_attention_scores=admitted_scores,
            live_attention_scores=live_scores,
            summary_attention_scores=summary_scores,
            current_attention_scores=current_scores,
            summary_slots_read=summary_sources_total,
            attention_transported_r4_blocks=(
                materialized * 2 * R4_BLOCKS_PER_HEAD
            ),
            compression_transported_r4_blocks=fold_transports,
            value_reads=materialized * HEAD_DIM,
            vocabulary_scores=batch_size * VOCAB_SIZE,
            source_reads=batch_size,
            peak_attention_source_slots=peak_sources,
        )
        cumulative = state.audit.accumulated_with(call_audit)
        final_state = FixedRecurrentKVState(
            live_keys=live_keys,
            live_values=live_values,
            live_frame_indices=live_frames,
            live_positions=live_positions,
            summary_keys_local=summary_keys,
            summary_values_local=summary_values,
            summary_counts=summary_counts,
            summary_frame_indices=summary_frames,
            summary_last_positions=summary_last_positions,
            current_frame_indices=current_frames,
            tokens_seen=position + 1,
            live_length=live_length,
            audit=cumulative,
        )
        self._validate_recurrent_state(
            final_state, batch_size=batch_size, device=token_ids.device
        )
        return FixedRecurrentKVStepOutput(
            logits=logits,
            final_state=final_state,
            audit=call_audit,
            attention_weights=torch.stack(layer_weights, dim=0),
        )

    @staticmethod
    def _require_native_r4(*, execution: str, intervention: str) -> None:
        if execution != "r4" or intervention != "native":
            raise ValueError(
                "fixed recurrent V1 supports only execution='r4' and "
                "intervention='native'"
            )

    def initial_state(
        self,
        batch_size: int,
        *,
        device: torch.device | str | None = None,
        dtype: torch.dtype | None = None,
        execution: str = "r4",
        intervention: str = "native",
    ) -> FixedRecurrentKVState:
        self._require_native_r4(
            execution=execution, intervention=intervention
        )
        return self.initial_recurrent_state(
            batch_size, device=device, dtype=dtype
        )

    def step(
        self,
        token_ids: Tensor,
        state: FixedRecurrentKVState,
        *,
        execution: str = "r4",
        intervention: str = "native",
    ) -> FixedRecurrentKVStepOutput:
        self._require_native_r4(
            execution=execution, intervention=intervention
        )
        if not isinstance(state, FixedRecurrentKVState):
            raise TypeError("fixed recurrent step requires FixedRecurrentKVState")
        return self.step_recurrent(token_ids, state)

    def forward_incremental(
        self,
        token_ids: Tensor,
        targets: Tensor | None = None,
        *,
        execution: str = "r4",
        intervention: str = "native",
        initial_state: FixedRecurrentKVState | None = None,
    ) -> FixedRecurrentKVBindingOutput:
        """Run a token block exclusively through the recurrent state."""

        self._require_native_r4(
            execution=execution, intervention=intervention
        )
        self._validate_inputs(token_ids, targets)
        batch_size, time = token_ids.shape
        state = (
            self.initial_recurrent_state(
                batch_size,
                device=token_ids.device,
                dtype=self.output_weight.dtype,
            )
            if initial_state is None
            else initial_state
        )
        self._validate_recurrent_state(
            state, batch_size=batch_size, device=token_ids.device
        )
        if state.tokens_seen + time > CONTEXT:
            raise ValueError(
                "token block exceeds the trained fixed-recurrent RoPE context"
            )
        call_audit = self._empty_recurrent_audit(batch_size)
        logits: list[Tensor] = []
        weights: list[Tensor] = []
        for position in range(time):
            output = self.step_recurrent(token_ids[:, position], state)
            state = output.final_state
            call_audit = call_audit.accumulated_with(output.audit)
            logits.append(output.logits)
            weights.append(output.attention_weights)
        stacked_logits = torch.stack(logits, dim=1)
        loss = None
        if targets is not None:
            loss = F.cross_entropy(
                stacked_logits.float().reshape(-1, VOCAB_SIZE),
                targets.reshape(-1),
            )
        return FixedRecurrentKVBindingOutput(
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
        execution: str = "r4",
        intervention: str = "native",
        initial_state: FixedRecurrentKVState | None = None,
    ) -> FixedRecurrentKVBindingOutput:
        """Keep inherited exact/full-square execution unreachable."""

        return self.forward_incremental(
            token_ids,
            targets,
            execution=execution,
            intervention=intervention,
            initial_state=initial_state,
        )


__all__ = [
    "LIVE_WINDOW",
    "MAXIMUM_READ_SOURCES",
    "POLICY",
    "RECURRENT_STATE_BYTES_F32",
    "RECURRENT_STATE_VALUES",
    "RECURRENT_METADATA_I64_VALUES",
    "SUMMARY_BANKS",
    "FixedRecurrentKVBindingAudit",
    "FixedRecurrentKVBindingOutput",
    "FixedRecurrentKVState",
    "FixedRecurrentKVStepOutput",
    "R4FixedRecurrentCausalKVBindingV1",
]
