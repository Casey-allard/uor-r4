"""Layerwise-normalized retained-state readout for issue #973.

``R4LayerwiseNormalizedRetainedReadoutLanguagePathV1`` leaves the qualified
exact-H4 addresses, transport, key/value recurrence, gates, residual stream,
MLPs, learned parameters, and persistent state unchanged.  It changes only
the already-computed retained-state contribution to the tied language-model
head:

``logits_t = E @ (N(h_t) + (g / sqrt(L)) * sum_l N(a_l,t))``.

``a_l,t`` is layer ``l``'s post-output-projection, post-state-off-gate
retained residual.  ``N`` is the existing learned final RMSNorm, ``E`` is the
existing tied embedding/head, ``L`` is the frozen two-layer count, and ``g``
is fixed to one for the candidate.  The equal-work V1 control fixes ``g`` to
zero while still executing every layer normalization, accumulation, scale,
residual addition, and vocabulary score.
"""

from __future__ import annotations

import math
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

POLICY = "R4LayerwiseNormalizedRetainedReadoutLanguagePathV1"
LAYERWISE_READOUT_SCALE = 1.0 / math.sqrt(LAYERS)


@dataclass(frozen=True, slots=True)
class LayerwiseNormalizedRetainedReadoutAudit(DecoderAudit):
    """Base recurrence work plus the fixed layerwise-readout work ledger."""

    layerwise_readout_enabled: bool = False
    retained_readout_values: int = 0
    retained_readout_accumulations: int = 0
    auxiliary_normalization_values: int = 0
    readout_scale_multiplications: int = 0
    readout_residual_additions: int = 0

    def work_signature(self) -> tuple[int, ...]:
        """Return work counters shared by candidate, V1, and state-off arms."""

        return (
            *DecoderAudit.work_signature(self),
            self.retained_readout_values,
            self.retained_readout_accumulations,
            self.auxiliary_normalization_values,
            self.readout_scale_multiplications,
            self.readout_residual_additions,
        )


class R4LayerwiseNormalizedRetainedReadoutLanguagePathV1(R4RetainedLanguagePathV1):
    """Qualified V1 recurrence with one fixed layer-normalized state skip."""

    def __init__(
        self,
        geometry: GroupAddressArtifact,
        *,
        layerwise_readout_enabled: bool = True,
    ) -> None:
        if not isinstance(layerwise_readout_enabled, bool):
            raise TypeError("layerwise-readout mode must be boolean")
        if LAYERS != 2:
            raise RuntimeError("layerwise retained readout is frozen to two layers")
        super().__init__(geometry)
        self.layerwise_readout_enabled = layerwise_readout_enabled
        if (
            self.parameter_count() != PARAMETER_COUNT
            or self.state_value_count() != STATE_VALUES
            or self.validity_bit_count() != VALIDITY_BITS
        ):
            raise RuntimeError(
                "layerwise retained readout changed the qualified ledger"
            )

    @classmethod
    def matched_v1_control(
        cls, geometry: GroupAddressArtifact
    ) -> R4LayerwiseNormalizedRetainedReadoutLanguagePathV1:
        """Build the equal-work, fixed-zero-gain V1 control."""

        return cls(geometry, layerwise_readout_enabled=False)

    @property
    def layerwise_readout_gain(self) -> float:
        """Return the frozen dimensionless candidate/control gain ``g``."""

        return 1.0 if self.layerwise_readout_enabled else 0.0

    @property
    def layerwise_readout_scale(self) -> float:
        """Return the single frozen multiplier ``g / sqrt(L)``."""

        return self.layerwise_readout_gain * LAYERWISE_READOUT_SCALE

    def _stationary_hidden_with_layerwise_readout(
        self, token_ids: Tensor, state: DecoderState, *, state_off: bool
    ) -> tuple[Tensor, DecoderState, Tensor]:
        """Run the unchanged stationary recurrence and normalize each read."""

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
        normalized_sum: Tensor | None = None
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
            normalized = self.final_norm(retained)
            normalized_sum = (
                normalized if normalized_sum is None else normalized_sum + normalized
            )
            final_keys.append(keys)
            final_values.append(layer_values)
            final_occupied.append(occupied)
        if normalized_sum is None:
            raise RuntimeError(
                "layerwise retained readout requires at least one retained layer"
            )
        return (
            values,
            DecoderState(
                keys=torch.stack(final_keys),
                values=torch.stack(final_values),
                occupied=torch.stack(final_occupied),
            ),
            normalized_sum,
        )

    def _direct_hidden_with_layerwise_readout(
        self, token_ids: Tensor, state: DecoderState, *, state_off: bool
    ) -> tuple[Tensor, DecoderState, Tensor]:
        """Run the unchanged direct recurrence and normalize each layer read."""

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
            normalized_sum: Tensor | None = None
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
                normalized = self.final_norm(retained)
                normalized_sum = (
                    normalized
                    if normalized_sum is None
                    else normalized_sum + normalized
                )
                keys.append(layer_keys)
                retained_values.append(layer_values)
                occupied.append(layer_occupied)
            if normalized_sum is None:
                raise RuntimeError(
                    "layerwise retained readout requires at least one retained layer"
                )
            current = DecoderState(
                keys=torch.stack(keys),
                values=torch.stack(retained_values),
                occupied=torch.stack(occupied),
            )
            outputs.append(values)
            readouts.append(normalized_sum)
        return torch.stack(outputs, dim=1), current, torch.stack(readouts, dim=1)

    def _layerwise_readout_audit(
        self,
        batch_size: int,
        time: int,
        *,
        state_off: bool,
        implementation: str,
    ) -> LayerwiseNormalizedRetainedReadoutAudit:
        base = self._audit(
            batch_size,
            time,
            state_off=state_off,
            implementation=implementation,
        )
        token_steps = batch_size * time
        return LayerwiseNormalizedRetainedReadoutAudit(
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
            layerwise_readout_enabled=self.layerwise_readout_enabled,
            retained_readout_values=token_steps * LAYERS * HIDDEN_SIZE,
            retained_readout_accumulations=(
                token_steps * max(0, LAYERS - 1) * HIDDEN_SIZE
            ),
            auxiliary_normalization_values=token_steps * LAYERS * HIDDEN_SIZE,
            readout_scale_multiplications=token_steps * HIDDEN_SIZE,
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
        """Execute V1 recurrence and the layerwise-normalized readout seam."""

        if token_ids.ndim != 2:
            raise ValueError("token_ids must have shape [batch,time]")
        state = (
            self.initial_state(int(token_ids.shape[0]))
            if initial_state is None
            else initial_state
        )
        self._validate_inputs(token_ids, targets, state)
        if implementation == "stationary":
            hidden, final_state, normalized_sum = (
                self._stationary_hidden_with_layerwise_readout(
                    token_ids, state, state_off=attention_off
                )
            )
        elif implementation == "direct":
            hidden, final_state, normalized_sum = (
                self._direct_hidden_with_layerwise_readout(
                    token_ids, state, state_off=attention_off
                )
            )
        else:
            raise ValueError("implementation must be 'stationary' or 'direct'")

        base_readout = self.final_norm(hidden)
        head_input = base_readout + normalized_sum * self.layerwise_readout_scale
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
            audit=self._layerwise_readout_audit(
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
        """Advance one token without bypassing the layerwise readout."""

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
    "LAYERWISE_READOUT_SCALE",
    "POLICY",
    "LayerwiseNormalizedRetainedReadoutAudit",
    "R4LayerwiseNormalizedRetainedReadoutLanguagePathV1",
]
