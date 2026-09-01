"""Two-block source-free R4 group-addressed retained-attention decoder.

``R4GroupAddressedRetentionDecoderV1`` is the construction frozen for issue
#973.  It consumes a supplied group-action artifact, keeps a fixed key/value
field per decoder layer, and applies ordinary stable softmax only across the
120 transported group addresses.  It never constructs a token-position
attention matrix.

The stationary implementation is a vectorized closed form of the direct
read-before-write recurrence.  Both implementations are public so compiler
experiments can prove logits, state, occupancy, and gradient parity before
using the faster path.
"""

from __future__ import annotations

import math
from dataclasses import dataclass
from typing import Literal

import torch
from safetensors.torch import load as load_safetensors
from safetensors.torch import save as save_safetensors
from torch import Tensor, nn
from torch.nn import functional as F

from .group_retention import GroupAddressArtifact
from .model import RMSNorm, SwiGLU

POLICY = "R4GroupAddressedRetentionDecoderV1"
ARTIFACT_SCHEMA = "uor-r4-group-addressed-retention-decoder/1"

PRODUCTION_VOCAB_SIZE = 4_096
PRODUCTION_HIDDEN_SIZE = 288
PRODUCTION_INTERMEDIATE_SIZE = 768
PRODUCTION_LAYERS = 2
PRODUCTION_HEADS = 4
PRODUCTION_HEAD_DIM = 72
PRODUCTION_GROUP_SIZE = 120
PRODUCTION_CONTEXT = 128
PRODUCTION_RMS_NORM_EPS = 1e-5
PRODUCTION_INITIALIZATION_SEED = 9_737
PRODUCTION_INITIALIZATION_STD = 0.02
PRODUCTION_DECAY_HALF_LIVES = (4.0, 16.0, 64.0, 256.0)
PRODUCTION_MAX_CANDIDATE_LEAVES = 35

PRODUCTION_PARAMETER_COUNT = 3_171_760
PRODUCTION_STATE_VALUES = 138_240
PRODUCTION_STATE_BYTES_F32 = 552_960
PRODUCTION_OCCUPANCY_BITS = 240


@dataclass(frozen=True, slots=True)
class DecoderConfig:
    """Frozen shape and initialization contract for the retained decoder."""

    vocab_size: int
    hidden_size: int
    intermediate_size: int
    layers: int
    heads: int
    head_dim: int
    group_size: int
    max_sequence_length: int
    rms_norm_eps: float = PRODUCTION_RMS_NORM_EPS
    initialization_seed: int = PRODUCTION_INITIALIZATION_SEED
    initialization_std: float = PRODUCTION_INITIALIZATION_STD
    decay_half_lives: tuple[float, ...] = PRODUCTION_DECAY_HALF_LIVES

    def validate(self) -> None:
        integer_fields = (
            self.vocab_size,
            self.hidden_size,
            self.intermediate_size,
            self.layers,
            self.heads,
            self.head_dim,
            self.group_size,
            self.max_sequence_length,
        )
        if any(value < 1 for value in integer_fields):
            raise ValueError("decoder dimensions must all be positive")
        if self.vocab_size < 2:
            raise ValueError("decoder vocabulary must contain at least two tokens")
        if self.hidden_size != self.heads * self.head_dim:
            raise ValueError("hidden size must equal heads times head dimension")
        if self.hidden_size % 4 or self.head_dim % 4:
            raise ValueError("hidden and retained-head widths must contain whole R4 blocks")
        if len(self.decay_half_lives) != self.heads:
            raise ValueError("one decay half-life is required per retained head")
        if any(not math.isfinite(value) or value <= 0.0 for value in self.decay_half_lives):
            raise ValueError("decay half-lives must be finite and positive")
        if not math.isfinite(self.rms_norm_eps) or self.rms_norm_eps <= 0.0:
            raise ValueError("RMSNorm epsilon must be finite and positive")
        if not math.isfinite(self.initialization_std) or self.initialization_std <= 0.0:
            raise ValueError("initialization standard deviation must be finite and positive")

    @classmethod
    def production(cls) -> DecoderConfig:
        config = cls.production_unchecked()
        config.validate_production()
        return config

    @classmethod
    def production_unchecked(cls) -> DecoderConfig:
        return cls(
            vocab_size=PRODUCTION_VOCAB_SIZE,
            hidden_size=PRODUCTION_HIDDEN_SIZE,
            intermediate_size=PRODUCTION_INTERMEDIATE_SIZE,
            layers=PRODUCTION_LAYERS,
            heads=PRODUCTION_HEADS,
            head_dim=PRODUCTION_HEAD_DIM,
            group_size=PRODUCTION_GROUP_SIZE,
            max_sequence_length=PRODUCTION_CONTEXT,
            rms_norm_eps=PRODUCTION_RMS_NORM_EPS,
            initialization_seed=PRODUCTION_INITIALIZATION_SEED,
            initialization_std=PRODUCTION_INITIALIZATION_STD,
            decay_half_lives=PRODUCTION_DECAY_HALF_LIVES,
        )

    def validate_production(self) -> None:
        self.validate()
        if self != DecoderConfig.production_unchecked():
            raise ValueError("production retained decoder exposes one frozen model contract")


def expected_parameter_count(config: DecoderConfig) -> int:
    """Return tied embedding plus two-norm Q/K/V/O-SwiGLU block parameters."""

    config.validate()
    embedding = config.vocab_size * config.hidden_size
    per_layer = (
        4 * config.hidden_size * config.hidden_size
        + 3 * config.hidden_size * config.intermediate_size
        + 2 * config.hidden_size
        + 2 * config.heads
    )
    return embedding + config.layers * per_layer + config.hidden_size


def expected_state_value_count(config: DecoderConfig) -> int:
    """Return learned recurrent f32 values per sequence, excluding occupancy."""

    config.validate()
    return (
        config.layers
        * 2
        * config.heads
        * config.group_size
        * config.head_dim
    )


def expected_occupancy_bit_count(config: DecoderConfig) -> int:
    """Return logical (not byte-packed) occupancy bits per sequence."""

    config.validate()
    return config.layers * config.group_size


@dataclass(slots=True)
class DecoderState:
    """Fixed-size recurrent state for every decoder layer.

    ``keys`` and ``values`` are ``[layers,batch,heads,group,head_dim]``;
    ``occupied`` is ``[layers,batch,group]`` and is deliberately reported
    separately from the learned f32 state budget.
    """

    keys: Tensor
    values: Tensor
    occupied: Tensor


@dataclass(frozen=True, slots=True)
class DecoderAudit:
    """Logical work/read ledger shared by stationary and incremental paths."""

    batch_size: int
    token_steps: int
    layers: int
    heads: int
    group_size: int
    transported_state_values: int
    occupancy_slot_reads: int
    attention_slot_scores: int
    attention_value_reads: int
    key_delta_writes: int
    value_delta_writes: int
    vocabulary_scores: int
    state_off: bool
    implementation: str
    forbidden_reads: int = 0

    def work_signature(self) -> tuple[int, ...]:
        """Return counters that must match geometry and state interventions."""

        return (
            self.batch_size,
            self.token_steps,
            self.layers,
            self.heads,
            self.group_size,
            self.transported_state_values,
            self.occupancy_slot_reads,
            self.attention_slot_scores,
            self.attention_value_reads,
            self.key_delta_writes,
            self.value_delta_writes,
            self.vocabulary_scores,
            self.forbidden_reads,
        )


@dataclass(slots=True)
class DecoderOutput:
    logits: Tensor
    loss: Tensor | None
    final_state: DecoderState
    audit: DecoderAudit


@dataclass(slots=True)
class DecoderStepOutput:
    logits: Tensor
    final_state: DecoderState
    audit: DecoderAudit


@dataclass(slots=True)
class _StationarySchedule:
    """Geometry/gate-only schedule shared by key and value fields."""

    write_addresses: Tensor
    read_addresses: Tensor
    write_coefficients: Tensor
    initial_write_coefficients: Tensor
    latest_read_write: Tensor
    read_has_write: Tensor
    read_elapsed: Tensor
    initial_read_decay: Tensor
    read_occupied: Tensor
    final_addresses: Tensor
    latest_final_write: Tensor
    final_has_write: Tensor
    final_elapsed: Tensor
    initial_final_decay: Tensor
    final_occupied: Tensor


class _RetainedDecoderBlock(nn.Module):
    """One pre-norm retained-attention plus SwiGLU residual block."""

    def __init__(self, config: DecoderConfig) -> None:
        super().__init__()
        self.config = config
        self.input_layernorm = RMSNorm(config.hidden_size, config.rms_norm_eps)
        self.q_proj = nn.Linear(config.hidden_size, config.hidden_size, bias=False)
        self.k_proj = nn.Linear(config.hidden_size, config.hidden_size, bias=False)
        self.v_proj = nn.Linear(config.hidden_size, config.hidden_size, bias=False)
        self.o_proj = nn.Linear(config.hidden_size, config.hidden_size, bias=False)
        self.post_attention_layernorm = RMSNorm(config.hidden_size, config.rms_norm_eps)
        self.mlp = SwiGLU(config)  # type: ignore[arg-type]

        half_lives = torch.tensor(config.decay_half_lives, dtype=torch.float32)
        rho = torch.exp(math.log(0.5) / half_lives)
        self.decay_logits = nn.Parameter(torch.logit(rho))
        self.write_logits = nn.Parameter(torch.zeros(config.heads, dtype=torch.float32))

    def resolved_gates(self) -> tuple[Tensor, Tensor]:
        return torch.sigmoid(self.decay_logits), torch.sigmoid(self.write_logits)

    def _heads(self, values: Tensor) -> Tensor:
        return values.view(*values.shape[:-1], self.config.heads, self.config.head_dim)

    @staticmethod
    def _attend(query: Tensor, keys: Tensor, values: Tensor, occupied: Tensor) -> Tensor:
        """Stable softmax over occupied slots; all-empty rows return exact zero."""

        head_dim = int(query.shape[-1])
        scores = torch.einsum("...hd,...hgd->...hg", query.float(), keys.float())
        scores = scores / math.sqrt(head_dim)
        valid = occupied.unsqueeze(-2)
        has_any = occupied.any(dim=-1, keepdim=True).unsqueeze(-2)
        masked = scores.masked_fill(~valid, torch.finfo(scores.dtype).min)
        safe = torch.where(has_any, masked, torch.zeros_like(masked))
        weights = torch.softmax(safe, dim=-1, dtype=torch.float32)
        weights = weights * valid.to(weights.dtype)
        return torch.einsum("...hg,...hgd->...hd", weights, values.float()).to(values.dtype)

    @staticmethod
    def _inclusive_permutation_scan(actions: Tensor) -> Tensor:
        """Compose supplied new-slot-to-old-slot permutations in parallel."""

        prefix = actions
        stride = 1
        time = int(actions.shape[1])
        while stride < time:
            older = prefix[:, :-stride, :]
            newer = prefix[:, stride:, :]
            composed = torch.gather(older, dim=2, index=newer)
            prefix = torch.cat((prefix[:, :stride, :], composed), dim=1)
            stride *= 2
        return prefix

    @staticmethod
    def _inclusive_prefix_max(values: Tensor) -> Tensor:
        """Inclusive maximum scan using MPS-native maximum and concatenation.

        Apple MPS does not implement ``aten::_cummax_helper``.  This exact
        Hillis-Steele scan has logarithmic launch depth and uses the same
        parallel-prefix layout as the permutation scan above.
        """

        prefix = values
        stride = 1
        time = int(values.shape[1])
        while stride < time:
            combined = torch.maximum(prefix[:, :-stride, :], prefix[:, stride:, :])
            prefix = torch.cat((prefix[:, :stride, :], combined), dim=1)
            stride *= 2
        return prefix

    def _build_stationary_schedule(
        self,
        initial_occupied: Tensor,
        prefix_actions: Tensor,
        identity_offset: int,
        rho: Tensor,
        eta: Tensor,
    ) -> _StationarySchedule:
        """Build one O(N*H*T*T) affine schedule for both K and V.

        Only the write recurrence needs a dense time-by-time coefficient field.
        Latest strict-prior writes are found per stationary group address with a
        cumulative maximum, avoiding the former ``[N,T,H,G,T]`` float tensor.
        """

        batch, time, group = prefix_actions.shape
        heads = self.config.heads
        positions = torch.arange(time, device=prefix_actions.device, dtype=torch.long)
        write_addresses = prefix_actions[:, :, identity_offset]
        read_addresses = prefix_actions

        same_writes = write_addresses[:, :, None] == write_addresses[:, None, :]
        inclusive = positions[None, :] <= positions[:, None]
        write_predecessors = same_writes & inclusive[None, :, :]
        write_rank = write_predecessors.sum(dim=2) - 1
        rank_delta = (
            write_rank[:, :, None] - write_rank[:, None, :]
        ).clamp_min(0)
        write_elapsed = (positions[:, None] - positions[None, :]).clamp_min(0)
        write_coefficients = (
            eta.view(1, heads, 1, 1)
            * rho.view(1, heads, 1, 1).pow(
                write_elapsed.view(1, 1, time, time)
            )
            * (1.0 - eta).view(1, heads, 1, 1).pow(
                rank_delta.unsqueeze(1)
            )
            * write_predecessors.unsqueeze(1)
        )
        initial_write_coefficients = (
            rho.view(1, heads, 1).pow(positions.view(1, 1, time) + 1)
            * (1.0 - eta).view(1, heads, 1).pow(write_rank.unsqueeze(1) + 1)
        )

        address_grid = torch.arange(
            group, device=prefix_actions.device, dtype=torch.long
        )
        write_time_by_address = torch.where(
            write_addresses[:, :, None] == address_grid.view(1, 1, group),
            positions.view(1, time, 1),
            torch.full((), -1, device=prefix_actions.device, dtype=torch.long),
        )
        latest_inclusive = self._inclusive_prefix_max(write_time_by_address)
        latest_strict = torch.cat(
            (
                torch.full(
                    (batch, 1, group),
                    -1,
                    device=prefix_actions.device,
                    dtype=torch.long,
                ),
                latest_inclusive[:, :-1, :],
            ),
            dim=1,
        )
        latest_read_write = torch.gather(latest_strict, dim=2, index=read_addresses)
        read_has_write = latest_read_write >= 0
        latest_read_write = latest_read_write.clamp_min(0)
        read_elapsed = positions.view(1, time, 1) - latest_read_write
        initial_read_decay = rho.view(1, heads).pow(
            (positions + 1).view(time, 1)
        )

        batch_index = torch.arange(batch, device=prefix_actions.device)[:, None, None]
        initial_read_occupied = initial_occupied[batch_index, read_addresses]
        read_occupied = initial_read_occupied | read_has_write

        final_addresses = prefix_actions[:, -1, :]
        latest_final_write = torch.gather(
            latest_inclusive[:, -1, :], dim=1, index=final_addresses
        )
        final_has_write = latest_final_write >= 0
        latest_final_write = latest_final_write.clamp_min(0)
        final_elapsed = (time - 1) - latest_final_write
        initial_final_decay = rho.pow(time)
        initial_final_occupied = initial_occupied[
            torch.arange(batch, device=prefix_actions.device)[:, None], final_addresses
        ]
        final_occupied = initial_final_occupied | final_has_write

        return _StationarySchedule(
            write_addresses=write_addresses,
            read_addresses=read_addresses,
            write_coefficients=write_coefficients,
            initial_write_coefficients=initial_write_coefficients,
            latest_read_write=latest_read_write,
            read_has_write=read_has_write,
            read_elapsed=read_elapsed,
            initial_read_decay=initial_read_decay,
            read_occupied=read_occupied,
            final_addresses=final_addresses,
            latest_final_write=latest_final_write,
            final_has_write=final_has_write,
            final_elapsed=final_elapsed,
            initial_final_decay=initial_final_decay,
            final_occupied=final_occupied,
        )

    def _apply_stationary_schedule(
        self,
        source: Tensor,
        initial: Tensor,
        schedule: _StationarySchedule,
        rho: Tensor,
    ) -> tuple[Tensor, Tensor]:
        """Apply a shared stationary schedule to one learned field."""

        batch, time, heads, head_dim = source.shape
        group = self.config.group_size
        initial_write_indices = schedule.write_addresses[:, None, :, None].expand(
            batch, heads, time, head_dim
        )
        initial_at_writes = torch.gather(initial, dim=2, index=initial_write_indices)
        written = torch.einsum(
            "nhts,nshd->nthd", schedule.write_coefficients, source
        ) + schedule.initial_write_coefficients.permute(0, 2, 1)[..., None] * (
            initial_at_writes.permute(0, 2, 1, 3)
        )

        batch_index = torch.arange(batch, device=source.device)[:, None, None]
        latest_written = written[batch_index, schedule.latest_read_write]
        latest_written = latest_written.permute(0, 1, 3, 2, 4)
        decayed_written = latest_written * rho.view(1, 1, heads, 1, 1).pow(
            schedule.read_elapsed.unsqueeze(2).unsqueeze(-1)
        )

        initial_by_address = initial.permute(0, 2, 1, 3)
        initial_reads = initial_by_address[batch_index, schedule.read_addresses]
        initial_reads = initial_reads.permute(0, 1, 3, 2, 4)
        decayed_initial = initial_reads * schedule.initial_read_decay.view(
            1, time, heads, 1, 1
        )
        reads = torch.where(
            schedule.read_has_write.unsqueeze(2).unsqueeze(-1),
            decayed_written,
            decayed_initial,
        )

        final_latest = written[
            torch.arange(batch, device=source.device)[:, None],
            schedule.latest_final_write,
        ].permute(0, 2, 1, 3)
        decayed_final = final_latest * rho.view(1, heads, 1).pow(
            schedule.final_elapsed.unsqueeze(1)
        )[..., None]
        final_initial_indices = schedule.final_addresses[:, None, :, None].expand(
            batch, heads, group, head_dim
        )
        final_initial = torch.gather(initial, dim=2, index=final_initial_indices)
        decayed_final_initial = final_initial * schedule.initial_final_decay.view(
            1, heads, 1, 1
        )
        final = torch.where(
            schedule.final_has_write.unsqueeze(1).unsqueeze(-1),
            decayed_final,
            decayed_final_initial,
        )
        return reads, final

    def forward_stationary(
        self,
        values: Tensor,
        key_state: Tensor,
        value_state: Tensor,
        occupied: Tensor,
        prefix_actions: Tensor,
        identity_offset: int,
        *,
        state_off: bool,
    ) -> tuple[Tensor, Tensor, Tensor, Tensor]:
        """Run the qualified stationary transition without exposing its read."""

        values, final_keys, final_values, final_occupied, _ = (
            self.forward_stationary_with_retained(
                values,
                key_state,
                value_state,
                occupied,
                prefix_actions,
                identity_offset,
                state_off=state_off,
            )
        )
        return values, final_keys, final_values, final_occupied

    def forward_stationary_with_retained(
        self,
        values: Tensor,
        key_state: Tensor,
        value_state: Tensor,
        occupied: Tensor,
        prefix_actions: Tensor,
        identity_offset: int,
        *,
        state_off: bool,
    ) -> tuple[Tensor, Tensor, Tensor, Tensor, Tensor]:
        """Run one stationary transition and expose its already-gated read.

        The fifth return value is the existing post-``o_proj``, post-state-off
        retained residual. It is observational: the qualified hidden and
        recurrent-state transition below is unchanged.
        """

        (
            values,
            final_keys,
            final_values,
            final_occupied,
            retained,
            _,
            _,
        ) = self.forward_stationary_with_retained_value_field(
            values,
            key_state,
            value_state,
            occupied,
            prefix_actions,
            identity_offset,
            state_off=state_off,
        )
        return values, final_keys, final_values, final_occupied, retained

    def forward_stationary_with_retained_value_field(
        self,
        values: Tensor,
        key_state: Tensor,
        value_state: Tensor,
        occupied: Tensor,
        prefix_actions: Tensor,
        identity_offset: int,
        *,
        state_off: bool,
    ) -> tuple[Tensor, Tensor, Tensor, Tensor, Tensor, Tensor, Tensor]:
        """Also expose the strict-prior transported/decayed value field.

        The sixth and seventh return values are respectively the value field
        immediately before the current write, shaped
        ``[batch,time,heads,group,head_dim]``, and its logical occupancy mask,
        shaped ``[batch,time,group]``.  This is an observational seam: the
        qualified attention, residual, MLP, and recurrent update below are the
        same operations used by :meth:`forward_stationary_with_retained`.
        """

        normalized = self.input_layernorm(values)
        query = self._heads(self.q_proj(normalized))
        key = self._heads(self.k_proj(normalized))
        value = self._heads(self.v_proj(normalized))
        rho, eta = self.resolved_gates()
        schedule = self._build_stationary_schedule(
            occupied,
            prefix_actions,
            identity_offset,
            rho,
            eta,
        )
        key_reads, final_keys = self._apply_stationary_schedule(
            key, key_state, schedule, rho
        )
        value_reads, final_values = self._apply_stationary_schedule(
            value,
            value_state,
            schedule,
            rho,
        )
        retained = self._attend(query, key_reads, value_reads, schedule.read_occupied)
        retained = self.o_proj(retained.reshape(*values.shape))
        retained = retained * (0.0 if state_off else 1.0)
        values = values + retained
        values = values + self.mlp(self.post_attention_layernorm(values))
        return (
            values,
            final_keys,
            final_values,
            schedule.final_occupied,
            retained,
            value_reads,
            schedule.read_occupied,
        )

    def forward_direct_step(
        self,
        values: Tensor,
        token_actions: Tensor,
        key_state: Tensor,
        value_state: Tensor,
        occupied: Tensor,
        identity_offset: int,
        *,
        state_off: bool,
    ) -> tuple[Tensor, Tensor, Tensor, Tensor]:
        """Execute one exact transport/read-before-write/update transition."""

        values, final_keys, final_values, final_occupied, _ = (
            self.forward_direct_step_with_retained(
                values,
                token_actions,
                key_state,
                value_state,
                occupied,
                identity_offset,
                state_off=state_off,
            )
        )
        return values, final_keys, final_values, final_occupied

    def forward_direct_step_with_retained(
        self,
        values: Tensor,
        token_actions: Tensor,
        key_state: Tensor,
        value_state: Tensor,
        occupied: Tensor,
        identity_offset: int,
        *,
        state_off: bool,
    ) -> tuple[Tensor, Tensor, Tensor, Tensor, Tensor]:
        """Run one direct transition and expose its already-gated read."""

        (
            values,
            final_keys,
            final_values,
            final_occupied,
            retained,
            _,
            _,
        ) = self.forward_direct_step_with_retained_value_field(
            values,
            token_actions,
            key_state,
            value_state,
            occupied,
            identity_offset,
            state_off=state_off,
        )
        return values, final_keys, final_values, final_occupied, retained

    def forward_direct_step_with_retained_value_field(
        self,
        values: Tensor,
        token_actions: Tensor,
        key_state: Tensor,
        value_state: Tensor,
        occupied: Tensor,
        identity_offset: int,
        *,
        state_off: bool,
    ) -> tuple[Tensor, Tensor, Tensor, Tensor, Tensor, Tensor, Tensor]:
        """Also expose the direct strict-prior transported/decayed value field.

        The sixth value is ``decayed_values`` before the identity-slot delta
        write and the seventh is its transported occupancy mask.  Existing V1
        callers discard both through :meth:`forward_direct_step_with_retained`.
        """

        batch = int(values.shape[0])
        heads = self.config.heads
        group = self.config.group_size
        head_dim = self.config.head_dim
        normalized = self.input_layernorm(values)
        query = self._heads(self.q_proj(normalized))
        key = self._heads(self.k_proj(normalized))
        value = self._heads(self.v_proj(normalized))
        rho, eta = self.resolved_gates()

        indices = token_actions[:, None, :, None].expand(batch, heads, group, head_dim)
        transported_keys = torch.gather(key_state, dim=2, index=indices)
        transported_values = torch.gather(value_state, dim=2, index=indices)
        transported_occupied = torch.gather(occupied, dim=1, index=token_actions)
        decayed_keys = transported_keys * rho.view(1, heads, 1, 1)
        decayed_values = transported_values * rho.view(1, heads, 1, 1)

        retained = self._attend(query, decayed_keys, decayed_values, transported_occupied)
        retained = self.o_proj(retained.reshape(batch, self.config.hidden_size))
        retained = retained * (0.0 if state_off else 1.0)
        values = values + retained
        values = values + self.mlp(self.post_attention_layernorm(values))

        identity_mask = F.one_hot(
            torch.tensor(identity_offset, device=values.device), num_classes=group
        ).to(decayed_keys.dtype)
        prior_keys = decayed_keys[:, :, identity_offset, :]
        prior_values = decayed_values[:, :, identity_offset, :]
        key_delta = eta.view(1, heads, 1) * (key - prior_keys)
        value_delta = eta.view(1, heads, 1) * (value - prior_values)
        final_keys = decayed_keys + key_delta[:, :, None, :] * identity_mask.view(
            1, 1, group, 1
        )
        final_values = decayed_values + value_delta[:, :, None, :] * identity_mask.view(
            1, 1, group, 1
        )
        final_occupied = transported_occupied | identity_mask.bool().view(1, group)
        return (
            values,
            final_keys,
            final_values,
            final_occupied,
            retained,
            decayed_values,
            transported_occupied,
        )


class R4GroupAddressedRetentionDecoderV1(nn.Module):
    """Two-block tied-head language model with fixed-state geometric attention."""

    def __init__(self, config: DecoderConfig, geometry: GroupAddressArtifact) -> None:
        super().__init__()
        config.validate()
        geometry.validate(group_size=config.group_size, vocab_size=config.vocab_size)
        self.config = config
        self.arm = geometry.arm
        self.geometry_artifact_cid = geometry.artifact_cid
        self.identity_offset = geometry.identity_offset

        self.token_embedding = nn.Embedding(config.vocab_size, config.hidden_size)
        self.layers = nn.ModuleList(_RetainedDecoderBlock(config) for _ in range(config.layers))
        self.final_norm = RMSNorm(config.hidden_size, config.rms_norm_eps)
        self.register_buffer("token_leaves", geometry.token_leaves.detach().clone().contiguous())
        self.register_buffer("left_actions", geometry.left_actions.detach().clone().contiguous())
        self._initialize_learned_weights()

        actual = self.parameter_count()
        expected = expected_parameter_count(config)
        if actual != expected:
            raise RuntimeError(f"retained-decoder parameter count {actual} != expected {expected}")

    @classmethod
    def production(
        cls, geometry: GroupAddressArtifact
    ) -> R4GroupAddressedRetentionDecoderV1:
        config = DecoderConfig.production()
        geometry.validate(
            group_size=PRODUCTION_GROUP_SIZE,
            vocab_size=PRODUCTION_VOCAB_SIZE,
            max_candidate_leaves=PRODUCTION_MAX_CANDIDATE_LEAVES,
            require_cid=True,
        )
        model = cls(config, geometry)
        if model.parameter_count() != PRODUCTION_PARAMETER_COUNT:
            raise RuntimeError("frozen production retained-decoder parameter count drifted")
        if model.state_value_count() != PRODUCTION_STATE_VALUES:
            raise RuntimeError("frozen production retained-decoder state size drifted")
        if model.occupancy_bit_count() != PRODUCTION_OCCUPANCY_BITS:
            raise RuntimeError("frozen production retained-decoder occupancy size drifted")
        return model

    def _initialize_learned_weights(self) -> None:
        generator = torch.Generator(device="cpu")
        generator.manual_seed(self.config.initialization_seed)
        with torch.no_grad():
            for module in self.modules():
                if isinstance(module, (nn.Embedding, nn.Linear)):
                    module.weight.normal_(
                        mean=0.0,
                        std=self.config.initialization_std,
                        generator=generator,
                    )

    @property
    def output_weight(self) -> nn.Parameter:
        """Return the genuinely shared embedding/language-model-head storage."""

        return self.token_embedding.weight

    def parameter_count(self) -> int:
        return sum(parameter.numel() for parameter in self.parameters())

    def state_value_count(self) -> int:
        return expected_state_value_count(self.config)

    def occupancy_bit_count(self) -> int:
        return expected_occupancy_bit_count(self.config)

    def initial_state(
        self,
        batch_size: int,
        *,
        device: torch.device | str | None = None,
        dtype: torch.dtype | None = None,
    ) -> DecoderState:
        if batch_size < 1:
            raise ValueError("batch size must be positive")
        reference = self.token_embedding.weight
        resolved_device = reference.device if device is None else torch.device(device)
        resolved_dtype = reference.dtype if dtype is None else dtype
        shape = (
            self.config.layers,
            batch_size,
            self.config.heads,
            self.config.group_size,
            self.config.head_dim,
        )
        field = torch.zeros(shape, device=resolved_device, dtype=resolved_dtype)
        occupied = torch.zeros(
            self.config.layers,
            batch_size,
            self.config.group_size,
            device=resolved_device,
            dtype=torch.bool,
        )
        return DecoderState(keys=field, values=field.clone(), occupied=occupied)

    def _validate_inputs(
        self, token_ids: Tensor, targets: Tensor | None, state: DecoderState
    ) -> None:
        if token_ids.ndim != 2 or token_ids.dtype != torch.long:
            raise ValueError("token_ids must be int64 [batch,time]")
        batch, time = token_ids.shape
        if batch < 1 or time < 1:
            raise ValueError("token_ids must contain at least one token")
        if time > self.config.max_sequence_length:
            raise ValueError("sequence exceeds the configured context")
        if bool((token_ids < 0).any()) or bool((token_ids >= self.config.vocab_size).any()):
            raise ValueError("token_ids contain an out-of-vocabulary value")
        if targets is not None:
            if targets.shape != token_ids.shape or targets.dtype != torch.long:
                raise ValueError("targets must be int64 and match token_ids")
            valid = targets != -100
            if bool(valid.any()):
                selected = targets[valid]
                if bool((selected < 0).any()) or bool((selected >= self.config.vocab_size).any()):
                    raise ValueError("targets contain an out-of-vocabulary value")
        expected_field = (
            self.config.layers,
            batch,
            self.config.heads,
            self.config.group_size,
            self.config.head_dim,
        )
        expected_occupancy = (
            self.config.layers,
            batch,
            self.config.group_size,
        )
        if tuple(state.keys.shape) != expected_field or tuple(state.values.shape) != expected_field:
            raise ValueError(f"key/value state must have shape {expected_field}")
        if tuple(state.occupied.shape) != expected_occupancy or state.occupied.dtype != torch.bool:
            raise ValueError(f"occupancy state must be bool with shape {expected_occupancy}")
        if (
            state.keys.device != token_ids.device
            or state.values.device != token_ids.device
            or state.occupied.device != token_ids.device
            or self.token_embedding.weight.device != token_ids.device
        ):
            raise ValueError("tokens, recurrent state, and model parameters must share a device")
        if (
            state.keys.dtype != self.token_embedding.weight.dtype
            or state.values.dtype != self.token_embedding.weight.dtype
        ):
            raise ValueError("recurrent fields must match learned-parameter dtype")

    def _audit(
        self,
        batch_size: int,
        time: int,
        *,
        state_off: bool,
        implementation: str,
    ) -> DecoderAudit:
        token_steps = batch_size * time
        state_values_per_token = (
            self.config.layers
            * 2
            * self.config.heads
            * self.config.group_size
            * self.config.head_dim
        )
        return DecoderAudit(
            batch_size=batch_size,
            token_steps=token_steps,
            layers=self.config.layers,
            heads=self.config.heads,
            group_size=self.config.group_size,
            transported_state_values=token_steps * state_values_per_token,
            occupancy_slot_reads=token_steps * self.config.layers * self.config.group_size,
            attention_slot_scores=(
                token_steps
                * self.config.layers
                * self.config.heads
                * self.config.group_size
            ),
            attention_value_reads=(
                token_steps
                * self.config.layers
                * self.config.heads
                * self.config.group_size
                * self.config.head_dim
            ),
            key_delta_writes=(
                token_steps * self.config.layers * self.config.heads * self.config.head_dim
            ),
            value_delta_writes=(
                token_steps * self.config.layers * self.config.heads * self.config.head_dim
            ),
            vocabulary_scores=token_steps * self.config.vocab_size,
            state_off=state_off,
            implementation=implementation,
        )

    def _stationary_hidden(
        self, token_ids: Tensor, state: DecoderState, *, state_off: bool
    ) -> tuple[Tensor, DecoderState]:
        batch, time = token_ids.shape
        values = self.token_embedding(token_ids)
        leaves = self.token_leaves.index_select(0, token_ids.reshape(-1)).view(batch, time)
        actions = self.left_actions.index_select(0, leaves.reshape(-1)).view(
            batch, time, self.config.group_size
        )
        prefix_actions = _RetainedDecoderBlock._inclusive_permutation_scan(actions)
        final_keys: list[Tensor] = []
        final_values: list[Tensor] = []
        final_occupied: list[Tensor] = []
        for layer_index, layer in enumerate(self.layers):
            values, keys, layer_values, occupied = layer.forward_stationary(
                values,
                state.keys[layer_index],
                state.values[layer_index],
                state.occupied[layer_index],
                prefix_actions,
                self.identity_offset,
                state_off=state_off,
            )
            final_keys.append(keys)
            final_values.append(layer_values)
            final_occupied.append(occupied)
        return values, DecoderState(
            keys=torch.stack(final_keys),
            values=torch.stack(final_values),
            occupied=torch.stack(final_occupied),
        )

    def _direct_hidden(
        self, token_ids: Tensor, state: DecoderState, *, state_off: bool
    ) -> tuple[Tensor, DecoderState]:
        outputs: list[Tensor] = []
        current = state
        for time_index in range(int(token_ids.shape[1])):
            token = token_ids[:, time_index]
            values = self.token_embedding(token)
            leaves = self.token_leaves.index_select(0, token)
            actions = self.left_actions.index_select(0, leaves)
            keys: list[Tensor] = []
            retained_values: list[Tensor] = []
            occupied: list[Tensor] = []
            for layer_index, layer in enumerate(self.layers):
                values, layer_keys, layer_values, layer_occupied = layer.forward_direct_step(
                    values,
                    actions,
                    current.keys[layer_index],
                    current.values[layer_index],
                    current.occupied[layer_index],
                    self.identity_offset,
                    state_off=state_off,
                )
                keys.append(layer_keys)
                retained_values.append(layer_values)
                occupied.append(layer_occupied)
            current = DecoderState(
                keys=torch.stack(keys),
                values=torch.stack(retained_values),
                occupied=torch.stack(occupied),
            )
            outputs.append(values)
        return torch.stack(outputs, dim=1), current

    def forward(
        self,
        token_ids: Tensor,
        targets: Tensor | None = None,
        *,
        initial_state: DecoderState | None = None,
        state_off: bool = False,
        implementation: Literal["stationary", "direct"] = "stationary",
    ) -> DecoderOutput:
        if token_ids.ndim != 2:
            raise ValueError("token_ids must have shape [batch,time]")
        state = self.initial_state(int(token_ids.shape[0])) if initial_state is None else initial_state
        self._validate_inputs(token_ids, targets, state)
        if implementation == "stationary":
            hidden, final_state = self._stationary_hidden(token_ids, state, state_off=state_off)
        elif implementation == "direct":
            hidden, final_state = self._direct_hidden(token_ids, state, state_off=state_off)
        else:
            raise ValueError("implementation must be 'stationary' or 'direct'")
        hidden = self.final_norm(hidden)
        logits = F.linear(hidden, self.output_weight)
        loss = None
        if targets is not None:
            loss = F.cross_entropy(
                logits.float().reshape(-1, self.config.vocab_size), targets.reshape(-1)
            )
        return DecoderOutput(
            logits=logits,
            loss=loss,
            final_state=final_state,
            audit=self._audit(
                int(token_ids.shape[0]),
                int(token_ids.shape[1]),
                state_off=state_off,
                implementation=implementation,
            ),
        )

    def forward_incremental(
        self,
        token_ids: Tensor,
        targets: Tensor | None = None,
        *,
        initial_state: DecoderState | None = None,
        state_off: bool = False,
    ) -> DecoderOutput:
        """Explicit direct-recurrence alias used by parity and inference callers."""

        return self.forward(
            token_ids,
            targets,
            initial_state=initial_state,
            state_off=state_off,
            implementation="direct",
        )

    def step(
        self,
        token_ids: Tensor,
        state: DecoderState,
        *,
        state_off: bool = False,
    ) -> DecoderStepOutput:
        """Advance one token per sequence through the direct incremental kernel."""

        if token_ids.ndim != 1:
            raise ValueError("incremental token_ids must have shape [batch]")
        output = self.forward_incremental(
            token_ids[:, None], initial_state=state, state_off=state_off
        )
        return DecoderStepOutput(
            logits=output.logits[:, 0, :],
            final_state=output.final_state,
            audit=output.audit,
        )

    def export_learned_artifact(self) -> bytes:
        """Return deterministic Safetensors bytes containing learned values only."""

        tensors = {
            name: parameter.detach().cpu().contiguous()
            for name, parameter in sorted(self.named_parameters())
        }
        # Safetensors tensor headers are canonical for sorted names.  Metadata
        # is intentionally absent: its underlying map order is not guaranteed,
        # which would make byte identity depend on process hash iteration.
        return save_safetensors(tensors)

    def load_learned_artifact(self, payload: bytes) -> None:
        """Load exact learned values while leaving the supplied geometry untouched."""

        loaded = load_safetensors(payload)
        expected = dict(self.named_parameters())
        if set(loaded) != set(expected):
            raise ValueError("learned artifact parameter names differ from decoder")
        with torch.no_grad():
            for name in sorted(expected):
                source = loaded[name]
                target = expected[name]
                if source.dtype != target.dtype or tuple(source.shape) != tuple(target.shape):
                    raise ValueError(f"learned artifact tensor contract differs for {name}")
                target.copy_(source.to(device=target.device))
