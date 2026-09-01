"""Bounded group-addressed recurrent language model for issue #973.

The module deliberately does not construct H4.  It consumes a versioned
geometry artifact whose rows are already the left actions to execute.  This
keeps the learning mechanism identical across the exact-H4, cyclic, and
scrambled-H4 arms while making the action table an explicit input.
"""

from __future__ import annotations

import math
from collections.abc import Mapping
from dataclasses import dataclass

import torch
from torch import Tensor, nn
from torch.nn import functional as F
from torch.utils.checkpoint import checkpoint


POLICY = "R4GroupAddressedRetentionLMV1"
GEOMETRY_ARMS = ("exact_h4", "cyclic_120", "scrambled_h4")

PRODUCTION_GROUP_SIZE = 120
PRODUCTION_HIDDEN_SIZE = 288
PRODUCTION_BANKS = 4
PRODUCTION_VOCAB_SIZE = 4_096
PRODUCTION_CONTEXT = 256
PRODUCTION_CHECKPOINT_CHUNK = 16
PRODUCTION_INITIALIZATION_SEED = 9_736
PRODUCTION_MAX_CANDIDATE_LEAVES = 35

PRODUCTION_PARAMETER_COUNT = 2_359_308
PRODUCTION_STATE_VALUES = 138_240
PRODUCTION_STATE_BYTES_F32 = 552_960

LEARNED_PARAMETER_NAMES = (
    "query_table",
    "value_table",
    "decay_logits",
    "write_logits",
    "bank_logits",
)


@dataclass(frozen=True, slots=True)
class GroupAddressArtifact:
    """Supplied group-addressing data; no group is synthesized in this module.

    ``left_actions[a, h]`` is the prior-state slot read when raw leaf ``a``
    recenters output slot ``h``.  Exact H4 and C120 artifacts therefore contain
    their respective left-regular actions.  A scrambled-H4 artifact contains
    the frozen, identity-fixing scrambled action rows while ``token_leaves``
    remains the true candidate-address map.
    """

    arm: str
    identity_offset: int
    token_leaves: Tensor
    left_actions: Tensor
    artifact_cid: str = ""

    def validate(
        self,
        *,
        group_size: int,
        vocab_size: int,
        max_candidate_leaves: int | None = None,
        require_cid: bool = False,
    ) -> None:
        if self.arm not in GEOMETRY_ARMS:
            raise ValueError(f"unsupported geometry arm: {self.arm!r}")
        if not 0 <= self.identity_offset < group_size:
            raise ValueError("geometry identity offset is outside the group table")
        if self.token_leaves.dtype != torch.long:
            raise ValueError("geometry token leaves must be int64")
        if self.left_actions.dtype != torch.long:
            raise ValueError("geometry left actions must be int64")
        if tuple(self.token_leaves.shape) != (vocab_size,):
            raise ValueError("geometry token leaves must have shape [vocab]")
        if tuple(self.left_actions.shape) != (group_size, group_size):
            raise ValueError("geometry left actions must have shape [group, group]")
        if self.token_leaves.device.type != "cpu" or self.left_actions.device.type != "cpu":
            raise ValueError("geometry artifacts must be validated from CPU tensors")
        if require_cid and not self.artifact_cid:
            raise ValueError("production geometry artifact requires a content identity")
        if vocab_size < 1 or int(self.token_leaves[0]) != self.identity_offset:
            raise ValueError("BOS token 0 must map to the actual identity offset")
        if bool((self.token_leaves < 0).any()) or bool((self.token_leaves >= group_size).any()):
            raise ValueError("geometry token leaves contain an out-of-range offset")
        if bool((self.left_actions < 0).any()) or bool((self.left_actions >= group_size).any()):
            raise ValueError("geometry left actions contain an out-of-range offset")

        expected = torch.arange(group_size, dtype=torch.long)
        if not torch.equal(self.left_actions[self.identity_offset], expected):
            raise ValueError("the supplied identity action is not the identity permutation")
        sorted_rows = self.left_actions.sort(dim=1).values
        if not torch.equal(sorted_rows, expected.expand(group_size, -1)):
            raise ValueError("every supplied left action must be a complete permutation")

        direct_leaf_count = int(torch.unique(self.token_leaves).numel())
        if max_candidate_leaves is not None and direct_leaf_count > max_candidate_leaves:
            raise ValueError(
                f"geometry exposes {direct_leaf_count} candidate leaves; "
                f"the frozen bound is {max_candidate_leaves}"
            )

    @property
    def direct_leaf_count(self) -> int:
        return int(torch.unique(self.token_leaves).numel())


@dataclass(frozen=True, slots=True)
class GroupRetentionConfig:
    """Shape and initialization contract for the recurrent cell."""

    vocab_size: int
    hidden_size: int
    group_size: int
    banks: int
    max_sequence_length: int
    checkpoint_chunk_size: int = PRODUCTION_CHECKPOINT_CHUNK
    initialization_seed: int = PRODUCTION_INITIALIZATION_SEED
    initialization_std: float = 0.02

    def validate(self) -> None:
        if self.vocab_size < 2:
            raise ValueError("vocabulary must contain at least two tokens")
        if self.hidden_size < 4 or self.hidden_size % 4:
            raise ValueError("hidden size must contain a whole number of R4 blocks")
        if self.group_size < 2:
            raise ValueError("group size must be at least two")
        if self.banks < 1:
            raise ValueError("at least one retention bank is required")
        if self.max_sequence_length < 1:
            raise ValueError("maximum sequence length must be positive")
        if self.checkpoint_chunk_size < 1:
            raise ValueError("checkpoint chunk size must be positive")
        if not math.isfinite(self.initialization_std) or self.initialization_std <= 0.0:
            raise ValueError("initialization standard deviation must be finite and positive")

    @classmethod
    def production(cls) -> GroupRetentionConfig:
        config = cls(
            vocab_size=PRODUCTION_VOCAB_SIZE,
            hidden_size=PRODUCTION_HIDDEN_SIZE,
            group_size=PRODUCTION_GROUP_SIZE,
            banks=PRODUCTION_BANKS,
            max_sequence_length=PRODUCTION_CONTEXT,
        )
        config.validate_production()
        return config

    def validate_production(self) -> None:
        self.validate()
        if self != GroupRetentionConfig.production_unchecked():
            raise ValueError("production group retention exposes one frozen model contract")

    @classmethod
    def production_unchecked(cls) -> GroupRetentionConfig:
        """Construct the frozen tuple without recursively validating it."""
        return cls(
            vocab_size=PRODUCTION_VOCAB_SIZE,
            hidden_size=PRODUCTION_HIDDEN_SIZE,
            group_size=PRODUCTION_GROUP_SIZE,
            banks=PRODUCTION_BANKS,
            max_sequence_length=PRODUCTION_CONTEXT,
            checkpoint_chunk_size=PRODUCTION_CHECKPOINT_CHUNK,
            initialization_seed=PRODUCTION_INITIALIZATION_SEED,
            initialization_std=0.02,
        )


def expected_parameter_count(config: GroupRetentionConfig) -> int:
    """Return ``Q + V + (rho, eta, alpha)`` trainable values."""
    config.validate()
    return 2 * config.vocab_size * config.hidden_size + 3 * config.banks


def expected_state_value_count(config: GroupRetentionConfig) -> int:
    """Return the fixed recurrent-state size for one sequence."""
    config.validate()
    return config.banks * config.group_size * config.hidden_size


@dataclass(frozen=True, slots=True)
class GroupRetentionAudit:
    """Analytic operation/read census for one forward call."""

    batch_size: int
    token_steps: int
    candidate_leaf_groups: int
    recenter_slot_reads: int
    identity_delta_writes: int
    weighted_bank_reads: int
    current_candidate_dot_products: int
    retained_executed_dot_products: int
    retained_candidate_dot_products: int
    checkpoint_chunks: int
    state_off: bool
    forbidden_reads: int = 0

    def work_signature(self) -> tuple[int, ...]:
        """Counters that must remain identical under the state-off intervention."""
        return (
            self.batch_size,
            self.token_steps,
            self.candidate_leaf_groups,
            self.recenter_slot_reads,
            self.identity_delta_writes,
            self.weighted_bank_reads,
            self.current_candidate_dot_products,
            self.retained_executed_dot_products,
            self.retained_candidate_dot_products,
            self.checkpoint_chunks,
            self.forbidden_reads,
        )


@dataclass(slots=True)
class GroupRetentionOutput:
    logits: Tensor
    loss: Tensor | None
    final_state: Tensor
    audit: GroupRetentionAudit


@dataclass(slots=True)
class GroupRetentionLastOutput:
    """Last-position logits and the exact work ledger used to obtain them."""

    logits: Tensor
    audit: GroupRetentionAudit


class R4GroupAddressedRetentionLMV1(nn.Module):
    """Full-vocabulary causal LM backed by a bounded group-addressed state."""

    def __init__(self, config: GroupRetentionConfig, geometry: GroupAddressArtifact) -> None:
        super().__init__()
        config.validate()
        geometry.validate(group_size=config.group_size, vocab_size=config.vocab_size)
        self.config = config
        self.arm = geometry.arm
        self.geometry_artifact_cid = geometry.artifact_cid
        self.identity_offset = geometry.identity_offset

        generator = torch.Generator(device="cpu")
        generator.manual_seed(config.initialization_seed)
        query = torch.empty(config.vocab_size, config.hidden_size, dtype=torch.float32)
        value = torch.empty(config.vocab_size, config.hidden_size, dtype=torch.float32)
        query.normal_(mean=0.0, std=config.initialization_std, generator=generator)
        value.normal_(mean=0.0, std=config.initialization_std, generator=generator)
        self.query_table = nn.Parameter(query)
        self.value_table = nn.Parameter(value)

        # Distinct deterministic bank timescales make every scalar live at
        # initialization while remaining byte-identical across experiment arms.
        decay_base = torch.linspace(1.5, 3.0, config.banks, dtype=torch.float32)
        write_base = torch.linspace(-1.0, 1.0, config.banks, dtype=torch.float32)
        decay_noise = torch.randn(config.banks, generator=generator) * 0.01
        write_noise = torch.randn(config.banks, generator=generator) * 0.01
        bank_noise = torch.randn(config.banks, generator=generator) * 0.01
        self.decay_logits = nn.Parameter(decay_base + decay_noise)
        self.write_logits = nn.Parameter(write_base + write_noise)
        self.bank_logits = nn.Parameter(bank_noise)

        leaves = geometry.token_leaves.detach().clone().contiguous()
        actions = geometry.left_actions.detach().clone().contiguous()
        self.register_buffer("token_leaves", leaves)
        self.register_buffer("left_actions", actions)
        identity_mask = torch.zeros(config.group_size, dtype=torch.float32)
        identity_mask[geometry.identity_offset] = 1.0
        self.register_buffer("identity_mask", identity_mask)

        group_leaves, grouped_indices, group_mask = self._build_candidate_groups(leaves)
        self.register_buffer("candidate_group_leaves", group_leaves)
        self.register_buffer("candidate_group_indices", grouped_indices)
        self.register_buffer("candidate_group_mask", group_mask)
        self.register_buffer(
            "candidate_group_flat_valid_indices",
            torch.nonzero(group_mask.reshape(-1), as_tuple=False).flatten(),
        )
        ordered_candidates = grouped_indices[group_mask]
        candidate_score_positions = torch.empty(config.vocab_size, dtype=torch.long)
        candidate_score_positions[ordered_candidates] = torch.arange(
            config.vocab_size, dtype=torch.long
        )
        self.register_buffer("candidate_score_positions", candidate_score_positions)

        # These schedules depend only on the frozen maximum context.  Keeping
        # them as non-persistent buffers avoids rebuilding and dispatching the
        # same arange/causal kernels in every measured MPS training step while
        # leaving the serialized learned-artifact contract unchanged.
        sequence_positions = torch.arange(config.max_sequence_length, dtype=torch.long)
        self.register_buffer("sequence_positions", sequence_positions, persistent=False)
        self.register_buffer(
            "sequence_causal_mask",
            sequence_positions[None, :] <= sequence_positions[:, None],
            persistent=False,
        )
        self.register_buffer(
            "sequence_time_delta",
            (sequence_positions[:, None] - sequence_positions[None, :]).clamp_min(0),
            persistent=False,
        )

        actual = self.parameter_count()
        expected = expected_parameter_count(config)
        if actual != expected:
            raise RuntimeError(f"group-retention parameter count {actual} != expected {expected}")

    @classmethod
    def production(cls, geometry: GroupAddressArtifact) -> R4GroupAddressedRetentionLMV1:
        config = GroupRetentionConfig.production()
        geometry.validate(
            group_size=PRODUCTION_GROUP_SIZE,
            vocab_size=PRODUCTION_VOCAB_SIZE,
            max_candidate_leaves=PRODUCTION_MAX_CANDIDATE_LEAVES,
            require_cid=True,
        )
        model = cls(config, geometry)
        if model.parameter_count() != PRODUCTION_PARAMETER_COUNT:
            raise RuntimeError("frozen production parameter count drifted")
        if model.state_value_count() != PRODUCTION_STATE_VALUES:
            raise RuntimeError("frozen production state size drifted")
        return model

    @staticmethod
    def _build_candidate_groups(leaves: Tensor) -> tuple[Tensor, Tensor, Tensor]:
        group_leaves = torch.unique(leaves, sorted=True)
        members = [torch.nonzero(leaves == leaf, as_tuple=False).flatten() for leaf in group_leaves]
        maximum = max(int(member.numel()) for member in members)
        indices = torch.zeros(len(members), maximum, dtype=torch.long)
        mask = torch.zeros(len(members), maximum, dtype=torch.bool)
        for row, member in enumerate(members):
            width = int(member.numel())
            indices[row, :width] = member
            mask[row, :width] = True
        return group_leaves, indices, mask

    def parameter_count(self) -> int:
        return sum(parameter.numel() for parameter in self.parameters())

    def state_value_count(self) -> int:
        return expected_state_value_count(self.config)

    @property
    def candidate_leaf_group_count(self) -> int:
        return int(self.candidate_group_leaves.numel())

    def initial_state(
        self,
        batch_size: int,
        *,
        device: torch.device | str | None = None,
        dtype: torch.dtype | None = None,
    ) -> Tensor:
        if batch_size < 1:
            raise ValueError("batch size must be positive")
        reference = self.value_table
        return torch.zeros(
            batch_size,
            self.config.banks,
            self.config.group_size,
            self.config.hidden_size,
            device=reference.device if device is None else device,
            dtype=reference.dtype if dtype is None else dtype,
        )

    def resolved_coefficients(self) -> dict[str, Tensor]:
        return {
            "rho": torch.sigmoid(self.decay_logits),
            "eta": torch.sigmoid(self.write_logits),
            "alpha": torch.softmax(self.bank_logits, dim=0),
        }

    def _advance_state(
        self,
        state: Tensor,
        token_ids: Tensor,
        *,
        rho: Tensor | None = None,
        eta: Tensor | None = None,
    ) -> tuple[Tensor, Tensor]:
        leaves = self.token_leaves.index_select(0, token_ids)
        action_rows = self.left_actions.index_select(0, leaves)
        gather_indices = action_rows[:, None, :, None].expand(
            -1,
            self.config.banks,
            -1,
            self.config.hidden_size,
        )
        recentered = torch.gather(state, dim=2, index=gather_indices)

        if rho is None or eta is None:
            coefficients = self.resolved_coefficients()
            rho = coefficients["rho"]
            eta = coefficients["eta"]
        decayed = recentered * rho.view(1, -1, 1, 1)
        current_value = self.value_table.index_select(0, token_ids)
        identity_value = decayed[:, :, self.identity_offset, :]
        overwritten_identity = identity_value + eta.view(1, -1, 1) * (
            current_value[:, None, :] - identity_value
        )
        identity_delta = overwritten_identity - identity_value
        state = decayed + identity_delta[:, :, None, :] * self.identity_mask.view(1, 1, -1, 1)
        return state, current_value

    def _score_sequence(
        self,
        reads: Tensor,
        current_values: Tensor,
        *,
        state_off: bool,
    ) -> Tensor:
        """Score a complete sequence in two batched contractions.

        ``reads`` is the already bank-weighted ``[batch,time,leaves,hidden]``
        field.  Keeping full-vocabulary scoring outside the checkpoint chunks
        is important on Apple MPS: it executes one large contraction rather
        than sixteen small copies of the same candidate-table work, and the
        expensive score is not recomputed during checkpoint backward.
        """
        current_scores = F.linear(current_values, self.query_table)

        # One padded grouped contraction covers every candidate while holding
        # only <=35 address reads per sequence row.  It never materializes a
        # [batch, time, vocab, hidden] or [banks, time, vocab, hidden] tensor.
        grouped_queries = self.query_table.index_select(
            0, self.candidate_group_indices.reshape(-1)
        ).view(
            self.candidate_group_indices.shape[0],
            self.candidate_group_indices.shape[1],
            self.config.hidden_size,
        )
        grouped_scores = torch.einsum("ntud,ucd->ntuc", reads, grouped_queries)
        ordered_scores = grouped_scores.flatten(2).index_select(
            2, self.candidate_group_flat_valid_indices
        )
        # ``index_copy.out`` is unavailable on Apple MPS.  The frozen inverse
        # ordering turns the same leaf-grouped scores back into token order
        # using an MPS-native gather and performs no data-dependent dispatch.
        retained_scores = ordered_scores.index_select(2, self.candidate_score_positions)
        # Both interventions execute the same multiply; only the frozen scalar
        # changes, keeping the actual state-on/state-off work ledger matched.
        retained_scores = retained_scores * (0.0 if state_off else 1.0)
        return (current_scores + retained_scores) / math.sqrt(self.config.hidden_size)

    @staticmethod
    def _inclusive_permutation_scan(actions: Tensor) -> Tensor:
        """Return inclusive prefix compositions for permutation actions.

        Each action row maps a new local slot to the preceding frame.  The
        scan therefore composes ``older[newer[h]]``.  It deliberately composes
        the supplied permutations themselves rather than assuming that their
        row labels form a multiplication table; that distinction is required
        by the independently scrambled-H4 control.
        """
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

    def _run_chunk(
        self,
        state: Tensor,
        token_ids: Tensor,
        *,
        last_only: bool = False,
        return_final_state: bool = True,
    ) -> tuple[Tensor, Tensor, Tensor | None]:
        """Advance one chunk with the exact recurrence in a stationary frame.

        Recentring a 120-slot field 256 times is mathematically unnecessary.
        A prefix composition identifies the stationary address of every write
        and read.  Slots then evolve independently: between two writes they
        decay by ``rho`` and a repeated write applies the same gated overwrite
        as the direct recurrence.  The closed form below is the exact affine
        recurrence, while replacing hundreds of tiny full-state MPS launches
        with a handful of dense, bounded contractions.
        """
        coefficients = self.resolved_coefficients()
        rho = coefficients["rho"]
        eta = coefficients["eta"]
        alpha = coefficients["alpha"]
        batch_size, time = token_ids.shape
        banks = self.config.banks
        hidden = self.config.hidden_size

        leaves = self.token_leaves.index_select(0, token_ids.reshape(-1)).view(
            batch_size, time
        )
        actions = self.left_actions.index_select(0, leaves.reshape(-1)).view(
            batch_size, time, self.config.group_size
        )
        prefix_actions = self._inclusive_permutation_scan(actions)
        write_addresses = prefix_actions[:, :, self.identity_offset]
        read_addresses = prefix_actions.index_select(2, self.candidate_group_leaves)
        current_values = self.value_table.index_select(0, token_ids.reshape(-1)).view(
            batch_size, time, hidden
        )

        positions = self.sequence_positions[:time]
        causal = self.sequence_causal_mask[:time, :time]
        same_write_address = write_addresses[:, :, None] == write_addresses[:, None, :]
        write_predecessors = same_write_address & causal[None, :, :]
        write_rank = write_predecessors.sum(dim=2) - 1

        time_delta = self.sequence_time_delta[:time, :time]
        rank_delta = (write_rank[:, :, None] - write_rank[:, None, :]).clamp_min(0)
        write_coefficients = (
            eta[None, :, None, None]
            * rho[None, :, None, None].pow(time_delta[None, None, :, :])
            * (1.0 - eta[None, :, None, None]).pow(rank_delta[:, None, :, :])
            * write_predecessors[:, None, :, :]
        )

        initial_write_indices = write_addresses[:, None, :, None].expand(
            -1, banks, -1, hidden
        )
        initial_at_writes = torch.gather(state, dim=2, index=initial_write_indices)
        initial_write_coefficients = (
            rho[None, :, None].pow(positions[None, None, :] + 1)
            * (1.0 - eta[None, :, None]).pow(write_rank[:, None, :] + 1)
        )
        written_states = torch.einsum(
            "nkts,nsd->nktd", write_coefficients, current_values
        ) + initial_write_coefficients[:, :, :, None] * initial_at_writes

        # Identify the most recent write to each addressed slot.  The index
        # schedule is geometry-only, so it carries no learned gradient.
        if last_only:
            read_addresses = read_addresses[:, -1:, :]
            read_time_positions = positions[-1:]
            current_values = current_values[:, -1:, :]
        else:
            read_time_positions = positions

        write_positions = positions.view(1, 1, 1, time)
        read_matches = write_addresses[:, None, None, :] == read_addresses[:, :, :, None]
        read_causal = positions[None, :] <= read_time_positions[:, None]
        read_matches = read_matches & read_causal[None, :, None, :]
        latest_read_write = torch.where(
            read_matches,
            write_positions,
            torch.full((), -1, device=token_ids.device, dtype=torch.long),
        ).amax(dim=3)
        read_has_write = latest_read_write >= 0
        latest_read_write = latest_read_write.clamp_min(0)
        read_elapsed = read_time_positions[None, :, None] - latest_read_write

        batch_index = torch.arange(batch_size, device=token_ids.device)[:, None, None]
        reads = torch.zeros(
            batch_size,
            int(read_time_positions.numel()),
            self.candidate_leaf_group_count,
            hidden,
            device=state.device,
            dtype=state.dtype,
        )
        for bank in range(banks):
            latest_written = written_states[:, bank][batch_index, latest_read_write]
            decayed_written = latest_written * rho[bank].pow(read_elapsed)[..., None]
            initial_reads = state[:, bank][batch_index, read_addresses]
            decayed_initial = initial_reads * rho[bank].pow(read_time_positions + 1)[
                None, :, None, None
            ]
            bank_reads = torch.where(
                read_has_write[..., None], decayed_written, decayed_initial
            )
            reads = reads + alpha[bank] * bank_reads

        if not return_final_state:
            return reads, current_values, None

        # Return the final field in the new local frame.  This is the same
        # state that a following chunk (or the caller) would receive from the
        # direct recenter/decay/overwrite recurrence.
        final_addresses = prefix_actions[:, -1, :]
        final_matches = write_addresses[:, None, :] == final_addresses[:, :, None]
        final_write = torch.where(
            final_matches,
            positions.view(1, 1, time),
            torch.full((), -1, device=token_ids.device, dtype=torch.long),
        ).amax(dim=2)
        final_has_write = final_write >= 0
        final_write = final_write.clamp_min(0)
        final_elapsed = (time - 1) - final_write
        final_banks: list[Tensor] = []
        final_batch_index = torch.arange(batch_size, device=token_ids.device)[:, None]
        for bank in range(banks):
            latest_written = written_states[:, bank][final_batch_index, final_write]
            decayed_written = latest_written * rho[bank].pow(final_elapsed)[..., None]
            initial_final = state[:, bank][final_batch_index, final_addresses]
            decayed_initial = initial_final * rho[bank].pow(time)
            final_banks.append(
                torch.where(final_has_write[..., None], decayed_written, decayed_initial)
            )
        final_state = torch.stack(final_banks, dim=1)
        return reads, current_values, final_state

    def _validate_inputs(self, token_ids: Tensor, targets: Tensor | None, state: Tensor) -> None:
        if token_ids.ndim != 2 or token_ids.dtype != torch.long:
            raise ValueError("token_ids must be int64 [batch, time]")
        batch_size, time = token_ids.shape
        if batch_size < 1 or time < 1:
            raise ValueError("token_ids must contain at least one sequence and one token")
        if time > self.config.max_sequence_length:
            raise ValueError("sequence exceeds the configured context")
        if bool((token_ids < 0).any()) or bool((token_ids >= self.config.vocab_size).any()):
            raise ValueError("token_ids contain an out-of-vocabulary value")
        if targets is not None:
            if targets.shape != token_ids.shape or targets.dtype != torch.long:
                raise ValueError("targets must be int64 and match token_ids")
            valid_targets = targets != -100
            if bool(valid_targets.any()):
                selected = targets[valid_targets]
                if bool((selected < 0).any()) or bool((selected >= self.config.vocab_size).any()):
                    raise ValueError("targets contain an out-of-vocabulary value")
        expected_state_shape = (
            batch_size,
            self.config.banks,
            self.config.group_size,
            self.config.hidden_size,
        )
        if tuple(state.shape) != expected_state_shape:
            raise ValueError(f"initial state must have shape {expected_state_shape}")
        if state.device != token_ids.device or state.device != self.value_table.device:
            raise ValueError("tokens, recurrent state, and model parameters must share a device")
        if state.dtype != self.value_table.dtype:
            raise ValueError("recurrent state dtype must match learned tables")

    def forward(
        self,
        token_ids: Tensor,
        targets: Tensor | None = None,
        *,
        initial_state: Tensor | None = None,
        state_off: bool = False,
        use_checkpoint: bool = False,
    ) -> GroupRetentionOutput:
        if token_ids.ndim != 2:
            raise ValueError("token_ids must have shape [batch, time]")
        batch_size, time = token_ids.shape
        state = self.initial_state(batch_size) if initial_state is None else initial_state
        self._validate_inputs(token_ids, targets, state)

        chunk_size = self.config.checkpoint_chunk_size if use_checkpoint else time
        read_chunks: list[Tensor] = []
        value_chunks: list[Tensor] = []
        for start in range(0, time, chunk_size):
            token_chunk = token_ids[:, start : start + chunk_size]
            if use_checkpoint and torch.is_grad_enabled():
                def run_chunk(
                    chunk_state: Tensor, chunk_tokens: Tensor
                ) -> tuple[Tensor, Tensor, Tensor]:
                    return self._run_chunk(chunk_state, chunk_tokens)

                chunk_reads, chunk_values, state = checkpoint(
                    run_chunk,
                    state,
                    token_chunk,
                    use_reentrant=False,
                )
            else:
                chunk_reads, chunk_values, state = self._run_chunk(state, token_chunk)
            read_chunks.append(chunk_reads)
            value_chunks.append(chunk_values)
        logits = self._score_sequence(
            torch.cat(read_chunks, dim=1),
            torch.cat(value_chunks, dim=1),
            state_off=state_off,
        )

        loss = None
        if targets is not None:
            loss = F.cross_entropy(
                logits.float().reshape(-1, self.config.vocab_size),
                targets.reshape(-1),
            )

        sequence_steps = batch_size * time
        checkpoint_chunks = math.ceil(time / self.config.checkpoint_chunk_size) if use_checkpoint else 0
        audit = GroupRetentionAudit(
            batch_size=batch_size,
            token_steps=sequence_steps,
            candidate_leaf_groups=self.candidate_leaf_group_count,
            recenter_slot_reads=(
                sequence_steps * self.config.banks * self.config.group_size
            ),
            identity_delta_writes=sequence_steps * self.config.banks,
            weighted_bank_reads=(
                sequence_steps
                * self.config.banks
                * self.candidate_leaf_group_count
            ),
            current_candidate_dot_products=sequence_steps * self.config.vocab_size,
            retained_executed_dot_products=(
                sequence_steps
                * int(self.candidate_group_indices.shape[0])
                * int(self.candidate_group_indices.shape[1])
            ),
            retained_candidate_dot_products=sequence_steps * self.config.vocab_size,
            checkpoint_chunks=checkpoint_chunks,
            state_off=state_off,
        )
        return GroupRetentionOutput(logits=logits, loss=loss, final_state=state, audit=audit)

    def score_last(
        self,
        token_ids: Tensor,
        *,
        initial_state: Tensor | None = None,
        state_off: bool = False,
    ) -> GroupRetentionLastOutput:
        """Score only the final causal position of each equal-length prefix.

        This is the evaluation path for the frozen prefix-order intervention.
        It performs the complete recurrent update for every supplied token but
        reads and scores the vocabulary only at the final position, avoiding a
        wasteful ``[batch,time,vocab]`` allocation.  No target is accepted, so
        the result cannot read a label or a future token.
        """
        if token_ids.ndim != 2:
            raise ValueError("token_ids must have shape [batch, time]")
        batch_size, time = token_ids.shape
        state = self.initial_state(batch_size) if initial_state is None else initial_state
        self._validate_inputs(token_ids, None, state)
        reads, current_values, final_state = self._run_chunk(
            state,
            token_ids,
            last_only=True,
            return_final_state=False,
        )
        if final_state is not None:
            raise RuntimeError("last-position scoring unexpectedly retained a final state")
        logits = self._score_sequence(
            reads,
            current_values,
            state_off=state_off,
        )[:, 0, :]

        sequence_steps = batch_size * time
        scored_steps = batch_size
        audit = GroupRetentionAudit(
            batch_size=batch_size,
            token_steps=sequence_steps,
            candidate_leaf_groups=self.candidate_leaf_group_count,
            recenter_slot_reads=(
                sequence_steps * self.config.banks * self.config.group_size
            ),
            identity_delta_writes=sequence_steps * self.config.banks,
            weighted_bank_reads=(
                scored_steps * self.config.banks * self.candidate_leaf_group_count
            ),
            current_candidate_dot_products=scored_steps * self.config.vocab_size,
            retained_executed_dot_products=(
                scored_steps
                * int(self.candidate_group_indices.shape[0])
                * int(self.candidate_group_indices.shape[1])
            ),
            retained_candidate_dot_products=scored_steps * self.config.vocab_size,
            checkpoint_chunks=0,
            state_off=state_off,
        )
        return GroupRetentionLastOutput(logits=logits, audit=audit)

    def export_learned_artifact(self) -> dict[str, Tensor]:
        """Return a deterministic, CPU-contiguous copy of learned values only."""
        return {
            name: getattr(self, name).detach().to(device="cpu", dtype=torch.float32).contiguous()
            for name in LEARNED_PARAMETER_NAMES
        }

    def load_learned_artifact_(self, artifact: Mapping[str, Tensor]) -> None:
        """Load an exact learned-value export without touching geometry buffers."""
        if set(artifact) != set(LEARNED_PARAMETER_NAMES):
            missing = sorted(set(LEARNED_PARAMETER_NAMES) - set(artifact))
            unexpected = sorted(set(artifact) - set(LEARNED_PARAMETER_NAMES))
            raise ValueError(
                f"learned artifact tensor contract mismatch: missing={missing}, "
                f"unexpected={unexpected}"
            )
        with torch.no_grad():
            for name in LEARNED_PARAMETER_NAMES:
                source = artifact[name]
                destination = getattr(self, name)
                if source.dtype != torch.float32 or tuple(source.shape) != tuple(destination.shape):
                    raise ValueError(f"learned artifact tensor {name!r} has the wrong dtype or shape")
                if not bool(torch.isfinite(source).all()):
                    raise ValueError(f"learned artifact tensor {name!r} is not finite")
                destination.copy_(source.to(device=destination.device, dtype=destination.dtype))
