"""Direct retained-state readout for the qualified #973 language cell.

``R4DirectRetainedReadoutLanguagePathV1`` leaves the exact-H4 addresses,
transport, key/value recurrence, gates, residual stream, MLPs, parameter
storage, and persistent state of ``R4RetainedLanguagePathV1`` unchanged.  It
only exposes the already-computed retained reads to the tied language-model
head:

``logits_t = E @ (N(h_t) + g * N(sum_l a_l,t))``.

``a_l,t`` is each layer's post-output-projection, post-state-off-gate retained
residual.  ``N`` is the existing learned final RMSNorm, ``E`` is the existing
tied embedding/head, and the frozen gain ``g`` is one for the candidate.  The
matched V1 control sets ``g`` to zero while executing the same readout work.
"""

from __future__ import annotations

from dataclasses import dataclass
from typing import Literal

import torch
from torch import Tensor
from torch.nn import functional as F

from .group_retention import GroupAddressArtifact
from .group_retention_decoder import (
    DecoderAudit,
    DecoderOutput,
    DecoderState,
    DecoderStepOutput,
    _RetainedDecoderBlock,
)
from .language_path_generalization import (
    HIDDEN_SIZE,
    LAYERS,
    PARAMETER_COUNT,
    STATE_VALUES,
    VALIDITY_BITS,
    R4RetainedLanguagePathV1,
)

POLICY = "R4DirectRetainedReadoutLanguagePathV1"


@dataclass(frozen=True, slots=True)
class DirectRetainedReadoutAudit(DecoderAudit):
    """Base recurrence work plus the fixed direct-readout work ledger."""

    direct_readout_enabled: bool = False
    retained_readout_values: int = 0
    retained_readout_accumulations: int = 0
    auxiliary_normalization_values: int = 0
    readout_gate_multiplications: int = 0
    readout_residual_additions: int = 0

    def work_signature(self) -> tuple[int, ...]:
        """Return work counters shared by candidate, V1, and state-off arms."""

        return (
            *DecoderAudit.work_signature(self),
            self.retained_readout_values,
            self.retained_readout_accumulations,
            self.auxiliary_normalization_values,
            self.readout_gate_multiplications,
            self.readout_residual_additions,
        )


class R4DirectRetainedReadoutLanguagePathV1(R4RetainedLanguagePathV1):
    """Qualified V1 recurrence with one fixed normalized tied-head state skip."""

    def __init__(
        self,
        geometry: GroupAddressArtifact,
        *,
        direct_readout_enabled: bool = True,
    ) -> None:
        if not isinstance(direct_readout_enabled, bool):
            raise TypeError("direct-readout mode must be boolean")
        super().__init__(geometry)
        self.direct_readout_enabled = direct_readout_enabled
        if (
            self.parameter_count() != PARAMETER_COUNT
            or self.state_value_count() != STATE_VALUES
            or self.validity_bit_count() != VALIDITY_BITS
        ):
            raise RuntimeError("direct retained readout changed the qualified ledger")

    @classmethod
    def matched_v1_control(
        cls, geometry: GroupAddressArtifact
    ) -> R4DirectRetainedReadoutLanguagePathV1:
        """Build the equal-work, fixed-zero-gain V1 control."""

        return cls(geometry, direct_readout_enabled=False)

    @property
    def direct_readout_gain(self) -> float:
        """Return the frozen candidate/control gain without learned state."""

        return 1.0 if self.direct_readout_enabled else 0.0

    def _stationary_hidden_with_readout(
        self, token_ids: Tensor, state: DecoderState, *, state_off: bool
    ) -> tuple[Tensor, DecoderState, Tensor]:
        """Run the unchanged stationary recurrence and collect layer reads."""

        batch, time = token_ids.shape
        values = self.token_embedding(token_ids)
        leaves = self.token_leaves.index_select(0, token_ids.reshape(-1)).view(
            batch, time
        )
        actions = self.left_actions.index_select(0, leaves.reshape(-1)).view(
            batch, time, self.config.group_size
        )
        prefix_actions = _RetainedDecoderBlock._inclusive_permutation_scan(actions)
        final_keys: list[Tensor] = []
        final_values: list[Tensor] = []
        final_occupied: list[Tensor] = []
        retained_sum: Tensor | None = None
        for layer_index, layer in enumerate(self.layers):
            values, keys, layer_values, occupied, retained = (
                layer.forward_stationary_with_retained(
                    values,
                    state.keys[layer_index],
                    state.values[layer_index],
                    state.occupied[layer_index],
                    prefix_actions,
                    self.identity_offset,
                    state_off=state_off,
                )
            )
            retained_sum = retained if retained_sum is None else retained_sum + retained
            final_keys.append(keys)
            final_values.append(layer_values)
            final_occupied.append(occupied)
        if retained_sum is None:
            raise RuntimeError("direct readout requires at least one retained layer")
        return (
            values,
            DecoderState(
                keys=torch.stack(final_keys),
                values=torch.stack(final_values),
                occupied=torch.stack(final_occupied),
            ),
            retained_sum,
        )

    def _direct_hidden_with_readout(
        self, token_ids: Tensor, state: DecoderState, *, state_off: bool
    ) -> tuple[Tensor, DecoderState, Tensor]:
        """Run the unchanged direct recurrence and collect layer reads."""

        outputs: list[Tensor] = []
        readouts: list[Tensor] = []
        current = state
        for time_index in range(int(token_ids.shape[1])):
            token = token_ids[:, time_index]
            values = self.token_embedding(token)
            leaves = self.token_leaves.index_select(0, token)
            actions = self.left_actions.index_select(0, leaves)
            keys: list[Tensor] = []
            retained_values: list[Tensor] = []
            occupied: list[Tensor] = []
            retained_sum: Tensor | None = None
            for layer_index, layer in enumerate(self.layers):
                values, layer_keys, layer_values, layer_occupied, retained = (
                    layer.forward_direct_step_with_retained(
                        values,
                        actions,
                        current.keys[layer_index],
                        current.values[layer_index],
                        current.occupied[layer_index],
                        self.identity_offset,
                        state_off=state_off,
                    )
                )
                retained_sum = (
                    retained if retained_sum is None else retained_sum + retained
                )
                keys.append(layer_keys)
                retained_values.append(layer_values)
                occupied.append(layer_occupied)
            if retained_sum is None:
                raise RuntimeError(
                    "direct readout requires at least one retained layer"
                )
            current = DecoderState(
                keys=torch.stack(keys),
                values=torch.stack(retained_values),
                occupied=torch.stack(occupied),
            )
            outputs.append(values)
            readouts.append(retained_sum)
        return torch.stack(outputs, dim=1), current, torch.stack(readouts, dim=1)

    def _readout_audit(
        self,
        batch_size: int,
        time: int,
        *,
        state_off: bool,
        implementation: str,
    ) -> DirectRetainedReadoutAudit:
        base = self._audit(
            batch_size,
            time,
            state_off=state_off,
            implementation=implementation,
        )
        token_steps = batch_size * time
        return DirectRetainedReadoutAudit(
            batch_size=base.batch_size,
            token_steps=base.token_steps,
            layers=base.layers,
            heads=base.heads,
            group_size=base.group_size,
            transported_state_values=base.transported_state_values,
            occupancy_slot_reads=base.occupancy_slot_reads,
            attention_slot_scores=base.attention_slot_scores,
            attention_value_reads=base.attention_value_reads,
            key_delta_writes=base.key_delta_writes,
            value_delta_writes=base.value_delta_writes,
            vocabulary_scores=base.vocabulary_scores,
            state_off=base.state_off,
            implementation=base.implementation,
            forbidden_reads=base.forbidden_reads,
            direct_readout_enabled=self.direct_readout_enabled,
            retained_readout_values=token_steps * LAYERS * HIDDEN_SIZE,
            retained_readout_accumulations=(
                token_steps * max(0, LAYERS - 1) * HIDDEN_SIZE
            ),
            auxiliary_normalization_values=token_steps * HIDDEN_SIZE,
            readout_gate_multiplications=token_steps * HIDDEN_SIZE,
            readout_residual_additions=token_steps * HIDDEN_SIZE,
        )

    def forward(
        self,
        token_ids: Tensor,
        targets: Tensor | None = None,
        *,
        attention_off: bool = False,
        initial_state: DecoderState | None = None,
        implementation: Literal["stationary", "direct"] = "stationary",
    ) -> DecoderOutput:
        """Execute V1 recurrence and the sole normalized direct-readout seam."""

        if token_ids.ndim != 2:
            raise ValueError("token_ids must have shape [batch,time]")
        state = (
            self.initial_state(int(token_ids.shape[0]))
            if initial_state is None
            else initial_state
        )
        self._validate_inputs(token_ids, targets, state)
        if implementation == "stationary":
            hidden, final_state, retained_sum = self._stationary_hidden_with_readout(
                token_ids, state, state_off=attention_off
            )
        elif implementation == "direct":
            hidden, final_state, retained_sum = self._direct_hidden_with_readout(
                token_ids, state, state_off=attention_off
            )
        else:
            raise ValueError("implementation must be 'stationary' or 'direct'")

        base_readout = self.final_norm(hidden)
        retained_readout = self.final_norm(retained_sum)
        head_input = base_readout + retained_readout * self.direct_readout_gain
        logits = F.linear(head_input, self.output_weight)
        loss = None
        if targets is not None:
            loss = F.cross_entropy(
                logits.float().reshape(-1, self.config.vocab_size), targets.reshape(-1)
            )
        return DecoderOutput(
            logits=logits,
            loss=loss,
            final_state=final_state,
            audit=self._readout_audit(
                int(token_ids.shape[0]),
                int(token_ids.shape[1]),
                state_off=attention_off,
                implementation=implementation,
            ),
        )

    def forward_incremental(
        self,
        token_ids: Tensor,
        targets: Tensor | None = None,
        *,
        attention_off: bool = False,
        initial_state: DecoderState | None = None,
    ) -> DecoderOutput:
        """Execute the candidate through its direct retained recurrence."""

        return self.forward(
            token_ids,
            targets,
            attention_off=attention_off,
            initial_state=initial_state,
            implementation="direct",
        )

    def step(
        self,
        token_ids: Tensor,
        state: DecoderState,
        *,
        attention_off: bool = False,
    ) -> DecoderStepOutput:
        """Advance one token without bypassing the direct readout."""

        if token_ids.ndim != 1:
            raise ValueError("incremental token_ids must have shape [batch]")
        output = self.forward(
            token_ids[:, None],
            attention_off=attention_off,
            initial_state=state,
            implementation="direct",
        )
        return DecoderStepOutput(
            logits=output.logits[:, 0, :],
            final_state=output.final_state,
            audit=output.audit,
        )


__all__ = [
    "POLICY",
    "DirectRetainedReadoutAudit",
    "R4DirectRetainedReadoutLanguagePathV1",
]
