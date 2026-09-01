"""Layer-paired exact-H4 addressing for the qualified retained language cell.

``R4PairedH4LanguagePathV1`` changes only which exact-H4 action row each of
the two already-qualified decoder layers applies.  Token zero remains the
identity in both layers.  Every other token receives the reversible radix
pair ``((t - 1) mod 120, floor((t - 1) / 120) mod 120)``.  The learned
parameters, recurrent field shapes, Q/K/V/O projections, decay and delta-write
gates, occupied-slot softmax, and read-before-write ordering remain those of
``R4RetainedLanguagePathV1``.

The collision helpers are construction-only instruments.  They count repeated
cumulative *joint* addresses without reading targets or model outputs.
"""

from __future__ import annotations

from dataclasses import dataclass

import torch
from torch import Tensor

from .group_retention import GroupAddressArtifact
from .group_retention_decoder import DecoderState
from .language_path_generalization import (
    GROUP_SIZE,
    LAYERS,
    PARAMETER_COUNT,
    STATE_VALUES,
    VALIDITY_BITS,
    VOCAB_SIZE,
    R4RetainedLanguagePathV1,
)

POLICY = "R4PairedH4LanguagePathV1"
CANONICAL_IDENTITY_INDEX = GROUP_SIZE - 1


def canonical_layer_token_leaves(
    *, identity_index: int = CANONICAL_IDENTITY_INDEX
) -> Tensor:
    """Return the frozen ``[layer, token]`` canonical H4 radix codebook.

    The exact-H4 artifact enumerates its identity at index 119.  Requiring
    that enumeration keeps BOS distinct from the 4,095 non-BOS radix pairs
    and makes all 4,096 token pairs injective.
    """

    if isinstance(identity_index, bool) or not isinstance(identity_index, int):
        raise TypeError("identity index must be an integer")
    if identity_index != CANONICAL_IDENTITY_INDEX:
        raise ValueError("paired-H4 language path requires canonical identity index 119")

    token_ids = torch.arange(VOCAB_SIZE, dtype=torch.long)
    non_bos = token_ids[1:] - 1
    leaves = torch.empty((LAYERS, VOCAB_SIZE), dtype=torch.long)
    leaves[:, 0] = identity_index
    leaves[0, 1:] = non_bos.remainder(GROUP_SIZE)
    leaves[1, 1:] = torch.div(non_bos, GROUP_SIZE, rounding_mode="floor").remainder(
        GROUP_SIZE
    )
    return leaves


@dataclass(frozen=True, slots=True)
class JointPrefixCollisionCensus:
    """Repeated cumulative joint addresses over a rectangular token batch."""

    sequences: int
    positions_per_sequence: int
    repeated_joint_addresses: int
    collision_free_sequences: int
    repeats_per_sequence: tuple[int, ...]

    @property
    def mean_repeated_joint_addresses(self) -> float:
        """Return the exact aggregate divided by the sequence count."""

        return self.repeated_joint_addresses / self.sequences


def _validate_collision_inputs(
    token_ids: Tensor,
    *,
    layer_token_leaves: Tensor,
    left_actions: Tensor,
    identity_index: int,
) -> None:
    if token_ids.ndim != 2 or token_ids.dtype != torch.long:
        raise ValueError("collision census token_ids must be int64 [batch,time]")
    if token_ids.shape[0] < 1 or token_ids.shape[1] < 1:
        raise ValueError("collision census requires a nonempty rectangular batch")
    if bool((token_ids < 0).any()) or bool((token_ids >= VOCAB_SIZE).any()):
        raise ValueError("collision census contains an out-of-vocabulary token")
    if layer_token_leaves.dtype != torch.long or tuple(layer_token_leaves.shape) != (
        LAYERS,
        VOCAB_SIZE,
    ):
        raise ValueError("layer token leaves must be int64 [2,4096]")
    if left_actions.dtype != torch.long or tuple(left_actions.shape) != (
        GROUP_SIZE,
        GROUP_SIZE,
    ):
        raise ValueError("left actions must be int64 [120,120]")
    if not 0 <= identity_index < GROUP_SIZE:
        raise ValueError("identity index is outside the H4 action table")
    if (
        token_ids.device != layer_token_leaves.device
        or token_ids.device != left_actions.device
    ):
        raise ValueError("collision census tensors must share one device")
    if bool((layer_token_leaves < 0).any()) or bool(
        (layer_token_leaves >= GROUP_SIZE).any()
    ):
        raise ValueError("layer token leaves contain an out-of-range H4 index")


def joint_prefix_collision_census(
    token_ids: Tensor,
    *,
    layer_token_leaves: Tensor,
    left_actions: Tensor,
    identity_index: int,
) -> JointPrefixCollisionCensus:
    """Count repeated cumulative ordered-H4-pair addresses.

    Composition matches the decoder's inclusive permutation scan:
    ``older[newer[h]]``.  Only token IDs and supplied action tables are read;
    targets, model weights, logits, and continuations are absent.
    """

    _validate_collision_inputs(
        token_ids,
        layer_token_leaves=layer_token_leaves,
        left_actions=left_actions,
        identity_index=identity_index,
    )
    batch, time = token_ids.shape
    identity = torch.arange(
        GROUP_SIZE, device=token_ids.device, dtype=torch.long
    ).expand(batch, -1)
    cumulative_actions = [identity.clone(), identity.clone()]
    joint_addresses: list[Tensor] = []

    for position in range(time):
        token = token_ids[:, position]
        addresses: list[Tensor] = []
        for layer_index in range(LAYERS):
            leaves = layer_token_leaves[layer_index].index_select(0, token)
            next_actions = left_actions.index_select(0, leaves)
            cumulative_actions[layer_index] = torch.gather(
                cumulative_actions[layer_index], dim=1, index=next_actions
            )
            addresses.append(cumulative_actions[layer_index][:, identity_index])
        joint_addresses.append(addresses[0] * GROUP_SIZE + addresses[1])

    encoded = torch.stack(joint_addresses, dim=1)
    repeats = tuple(
        time - int(torch.unique(encoded[row]).numel()) for row in range(batch)
    )
    return JointPrefixCollisionCensus(
        sequences=batch,
        positions_per_sequence=time,
        repeated_joint_addresses=sum(repeats),
        collision_free_sequences=sum(value == 0 for value in repeats),
        repeats_per_sequence=repeats,
    )


class R4PairedH4LanguagePathV1(R4RetainedLanguagePathV1):
    """The qualified retained cell with one canonical H4 coordinate per layer."""

    def __init__(self, geometry: GroupAddressArtifact) -> None:
        super().__init__(geometry)
        if self.identity_offset != CANONICAL_IDENTITY_INDEX:
            raise ValueError("paired-H4 language path requires canonical identity index 119")
        self.register_buffer(
            "layer_token_leaves",
            canonical_layer_token_leaves(identity_index=self.identity_offset),
        )
        if (
            self.parameter_count() != PARAMETER_COUNT
            or self.state_value_count() != STATE_VALUES
            or self.validity_bit_count() != VALIDITY_BITS
        ):
            raise RuntimeError("paired-H4 language path changed the qualified cell ledger")

    def _stationary_hidden(
        self, token_ids: Tensor, state: DecoderState, *, state_off: bool
    ) -> tuple[Tensor, DecoderState]:
        """Run the unchanged stationary cell with a distinct layer action row."""

        batch, time = token_ids.shape
        values = self.token_embedding(token_ids)
        final_keys: list[Tensor] = []
        final_values: list[Tensor] = []
        final_occupied: list[Tensor] = []
        for layer_index, layer in enumerate(self.layers):
            leaves = self.layer_token_leaves[layer_index].index_select(
                0, token_ids.reshape(-1)
            ).view(batch, time)
            actions = self.left_actions.index_select(0, leaves.reshape(-1)).view(
                batch, time, self.config.group_size
            )
            prefix_actions = layer._inclusive_permutation_scan(actions)
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
        """Run the unchanged direct recurrence with layer-specific actions."""

        outputs: list[Tensor] = []
        current = state
        for time_index in range(int(token_ids.shape[1])):
            token = token_ids[:, time_index]
            values = self.token_embedding(token)
            keys: list[Tensor] = []
            retained_values: list[Tensor] = []
            occupied: list[Tensor] = []
            for layer_index, layer in enumerate(self.layers):
                leaves = self.layer_token_leaves[layer_index].index_select(0, token)
                actions = self.left_actions.index_select(0, leaves)
                values, layer_keys, layer_values, layer_occupied = (
                    layer.forward_direct_step(
                        values,
                        actions,
                        current.keys[layer_index],
                        current.values[layer_index],
                        current.occupied[layer_index],
                        self.identity_offset,
                        state_off=state_off,
                    )
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


__all__ = [
    "CANONICAL_IDENTITY_INDEX",
    "POLICY",
    "JointPrefixCollisionCensus",
    "R4PairedH4LanguagePathV1",
    "canonical_layer_token_leaves",
    "joint_prefix_collision_census",
]
