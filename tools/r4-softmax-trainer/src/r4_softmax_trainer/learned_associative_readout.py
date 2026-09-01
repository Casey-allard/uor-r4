"""Learned candidate-leaf associative readout over qualified retained V1.

``R4LearnedCandidateLeafAssociativeReadoutV1`` freezes the complete qualified
``R4RetainedLanguagePathV1`` backbone and adds two independent zero-initialized
candidate-query tables.  The geometric head scores each candidate against the
strict-prior retained value at its exact-H4 relative leaf.  The matched pooled
head scores the same candidate against the occupied-address mean of that value
field.  Both exact-leaf gathering and occupied pooling execute before either
head is selected.

The learned rows are explicitly ``[layers,vocabulary,12,4]``: twelve R4
vectors per layer and candidate.  This module makes no intrinsic Spin(4) or H4
superiority claim; it implements the bounded candidate/address association
frozen for issue #973.
"""

from __future__ import annotations

import math
from collections.abc import Iterable
from dataclasses import asdict, dataclass
from typing import Literal

import torch
from safetensors.torch import load as load_safetensors
from safetensors.torch import save as save_safetensors
from torch import Tensor, nn
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
    HEAD_DIM,
    HEADS,
    HIDDEN_SIZE,
    LAYERS,
    PARAMETER_COUNT,
    STATE_VALUES,
    VALIDITY_BITS,
    VOCAB_SIZE,
    R4RetainedLanguagePathV1,
)


POLICY = "R4LearnedCandidateLeafAssociativeReadoutV1"
R4_WIDTH = 4
R4_BLOCKS = HIDDEN_SIZE // R4_WIDTH
QUERY_SHAPE = (LAYERS, VOCAB_SIZE, R4_BLOCKS, R4_WIDTH)
HEAD_PARAMETER_COUNT = math.prod(QUERY_SHAPE)
EFFECTIVE_ARM_PARAMETER_COUNT = PARAMETER_COUNT + HEAD_PARAMETER_COUNT
BUNDLE_PARAMETER_COUNT = PARAMETER_COUNT + 2 * HEAD_PARAMETER_COUNT
ASSOCIATIVE_SCALE = 1.0 / (LAYERS * math.sqrt(HIDDEN_SIZE))

Arm = Literal["geometric", "pooled", "deranged"]


@dataclass(frozen=True, slots=True)
class LearnedAssociativeReadoutAudit(DecoderAudit):
    """Qualified recurrence work plus one associative-head work ledger."""

    arm: str = ""
    head_off: bool = False
    used_candidate_leaves: int = 0
    strict_prior_value_field_reads: int = 0
    exact_leaf_value_reads: int = 0
    occupied_pool_value_reads: int = 0
    normalized_value_count: int = 0
    grouped_candidate_dot_products: int = 0
    grouped_executed_dot_products: int = 0
    associative_logit_additions: int = 0

    def work_signature(self) -> tuple[int, ...]:
        """Exclude only intervention labels; all executed counts must match."""

        return (
            *DecoderAudit.work_signature(self),
            self.used_candidate_leaves,
            self.strict_prior_value_field_reads,
            self.exact_leaf_value_reads,
            self.occupied_pool_value_reads,
            self.normalized_value_count,
            self.grouped_candidate_dot_products,
            self.grouped_executed_dot_products,
            self.associative_logit_additions,
        )


@dataclass(slots=True)
class LearnedAssociativeReadoutBundleOutput:
    """The two independently learned heads evaluated on shared frozen features."""

    geometric: DecoderOutput
    pooled: DecoderOutput


class R4LearnedCandidateLeafAssociativeReadoutV1(R4RetainedLanguagePathV1):
    """Frozen retained V1 plus geometric and address-blind associative heads."""

    def __init__(self, geometry: GroupAddressArtifact) -> None:
        super().__init__(geometry)
        if HEADS * HEAD_DIM != HIDDEN_SIZE or HIDDEN_SIZE % R4_WIDTH:
            raise RuntimeError("learned associative readout requires whole R4 blocks")

        self._qualified_base_parameter_names = tuple(
            name for name, _ in self.named_parameters()
        )
        for parameter in self.parameters():
            parameter.requires_grad_(False)

        self.geometric_queries = nn.Parameter(torch.zeros(QUERY_SHAPE))
        self.pooled_queries = nn.Parameter(torch.zeros(QUERY_SHAPE))

        (
            used_leaves,
            grouped_indices,
            group_mask,
        ) = self._build_candidate_groups(self.token_leaves)
        if int(used_leaves.numel()) < 2:
            raise ValueError("candidate-leaf derangement requires at least two used leaves")
        self.register_buffer("used_candidate_leaves", used_leaves)
        self.register_buffer("candidate_group_indices", grouped_indices)
        self.register_buffer("candidate_group_mask", group_mask)
        self.register_buffer(
            "candidate_group_flat_valid_indices",
            torch.nonzero(group_mask.reshape(-1), as_tuple=False).flatten(),
        )
        ordered_candidates = grouped_indices[group_mask]
        score_positions = torch.empty(VOCAB_SIZE, dtype=torch.long)
        score_positions[ordered_candidates] = torch.arange(VOCAB_SIZE)
        self.register_buffer("candidate_score_positions", score_positions)

        group_positions = torch.arange(int(used_leaves.numel()), dtype=torch.long)
        deranged_positions = torch.roll(group_positions, shifts=-1)
        self.register_buffer("deranged_group_positions", deranged_positions)
        self.register_buffer(
            "deranged_candidate_leaves",
            used_leaves.index_select(0, deranged_positions),
        )

        if tuple(self.geometric_queries.shape) != QUERY_SHAPE:
            raise RuntimeError("geometric query-table shape drifted")
        if tuple(self.pooled_queries.shape) != QUERY_SHAPE:
            raise RuntimeError("pooled query-table shape drifted")
        if self.parameter_count() != BUNDLE_PARAMETER_COUNT:
            raise RuntimeError("learned associative bundle parameter ledger drifted")
        if (
            self.state_value_count() != STATE_VALUES
            or self.validity_bit_count() != VALIDITY_BITS
        ):
            raise RuntimeError("learned associative readout changed retained state")

    @staticmethod
    def _build_candidate_groups(leaves: Tensor) -> tuple[Tensor, Tensor, Tensor]:
        used_leaves = torch.unique(leaves.detach().cpu(), sorted=True)
        members = [
            torch.nonzero(leaves.detach().cpu() == leaf, as_tuple=False).flatten()
            for leaf in used_leaves
        ]
        maximum = max(int(member.numel()) for member in members)
        indices = torch.zeros(len(members), maximum, dtype=torch.long)
        mask = torch.zeros(len(members), maximum, dtype=torch.bool)
        for row, member in enumerate(members):
            width = int(member.numel())
            indices[row, :width] = member
            mask[row, :width] = True
        return used_leaves, indices, mask

    @property
    def used_candidate_leaf_count(self) -> int:
        return int(self.used_candidate_leaves.numel())

    @property
    def maximum_candidates_per_leaf(self) -> int:
        return int(self.candidate_group_indices.shape[1])

    def effective_arm_parameter_count(self) -> int:
        return EFFECTIVE_ARM_PARAMETER_COUNT

    def head_parameter_count(self) -> int:
        return HEAD_PARAMETER_COUNT

    def geometric_parameters(self) -> Iterable[nn.Parameter]:
        return (self.geometric_queries,)

    def pooled_parameters(self) -> Iterable[nn.Parameter]:
        return (self.pooled_queries,)

    def head_parameters(self, arm: Literal["geometric", "pooled"]) -> Iterable[nn.Parameter]:
        if arm == "geometric":
            return self.geometric_parameters()
        if arm == "pooled":
            return self.pooled_parameters()
        raise ValueError("head parameters require 'geometric' or 'pooled'")

    def frozen_base_parameters(self) -> Iterable[nn.Parameter]:
        expected = set(self._qualified_base_parameter_names)
        return (
            parameter
            for name, parameter in self.named_parameters()
            if name in expected
        )

    def export_qualified_base_artifact(self) -> bytes:
        """Export the frozen base with byte-compatible V1 parameter names."""

        expected = set(self._qualified_base_parameter_names)
        tensors = {
            name: parameter.detach().cpu().contiguous()
            for name, parameter in sorted(self.named_parameters())
            if name in expected
        }
        return save_safetensors(tensors)

    def load_qualified_base_artifact(self, payload: bytes) -> None:
        """Load only a qualified V1 artifact; never alter either learned head."""

        loaded = load_safetensors(payload)
        parameters = dict(self.named_parameters())
        expected = set(self._qualified_base_parameter_names)
        if set(loaded) != expected:
            raise ValueError("qualified base artifact parameter names differ from V1")
        with torch.no_grad():
            for name in sorted(expected):
                source = loaded[name]
                target = parameters[name]
                if source.dtype != target.dtype or tuple(source.shape) != tuple(target.shape):
                    raise ValueError(
                        f"qualified base artifact tensor contract differs for {name}"
                    )
                if not bool(torch.isfinite(source).all()):
                    raise ValueError(f"qualified base artifact tensor is nonfinite: {name}")
                target.copy_(source.to(device=target.device))

    def _query_parameter(self, arm: Literal["geometric", "pooled"]) -> nn.Parameter:
        if arm == "geometric":
            return self.geometric_queries
        if arm == "pooled":
            return self.pooled_queries
        raise ValueError("query artifact arm must be 'geometric' or 'pooled'")

    def export_head_artifact(self, arm: Literal["geometric", "pooled"]) -> bytes:
        """Export one disjoint deterministic query-table artifact."""

        parameter = self._query_parameter(arm)
        return save_safetensors(
            {f"{arm}_queries": parameter.detach().cpu().contiguous()}
        )

    def load_head_artifact(
        self, arm: Literal["geometric", "pooled"], payload: bytes
    ) -> None:
        """Load exactly one head without reading or changing its matched peer."""

        name = f"{arm}_queries"
        loaded = load_safetensors(payload)
        if set(loaded) != {name}:
            raise ValueError(f"{arm} head artifact tensor names differ")
        source = loaded[name]
        target = self._query_parameter(arm)
        if source.dtype != target.dtype or tuple(source.shape) != QUERY_SHAPE:
            raise ValueError(f"{arm} head artifact tensor contract differs")
        if not bool(torch.isfinite(source).all()):
            raise ValueError(f"{arm} head artifact contains nonfinite values")
        with torch.no_grad():
            target.copy_(source.to(device=target.device))

    @staticmethod
    def _parameter_free_rms(values: Tensor, epsilon: float) -> Tensor:
        """RMS-normalize the final axis; an all-zero field remains exact zero."""

        floated = values.float()
        normalized = floated * torch.rsqrt(
            floated.square().mean(dim=-1, keepdim=True) + epsilon
        )
        return normalized.to(values.dtype)

    def _features_from_value_field(
        self, value_field: Tensor, occupied: Tensor
    ) -> tuple[Tensor, Tensor]:
        """Return exact-leaf and occupied-mean features from one layer."""

        if value_field.ndim != 5 or occupied.ndim != 3:
            raise ValueError("stationary value-field observation has invalid rank")
        route_values = value_field.index_select(3, self.used_candidate_leaves)
        route_occupied = occupied.index_select(2, self.used_candidate_leaves)
        route_values = route_values * route_occupied[:, :, None, :, None].to(
            route_values.dtype
        )
        route_values = route_values.permute(0, 1, 3, 2, 4).contiguous().view(
            value_field.shape[0],
            value_field.shape[1],
            self.used_candidate_leaf_count,
            HIDDEN_SIZE,
        )

        occupancy = occupied[:, :, None, :, None].to(value_field.dtype)
        pooled = (value_field * occupancy).sum(dim=3)
        count = occupied.sum(dim=2, keepdim=True).clamp_min(1).to(value_field.dtype)
        pooled = pooled / count[:, :, :, None]
        pooled = pooled.reshape(value_field.shape[0], value_field.shape[1], HIDDEN_SIZE)
        return (
            self._parameter_free_rms(route_values, self.config.rms_norm_eps),
            self._parameter_free_rms(pooled, self.config.rms_norm_eps),
        )

    def _direct_features_from_value_field(
        self, value_field: Tensor, occupied: Tensor
    ) -> tuple[Tensor, Tensor]:
        exact, pooled = self._features_from_value_field(
            value_field[:, None, :, :, :], occupied[:, None, :]
        )
        return exact[:, 0], pooled[:, 0]

    def _stationary_hidden_with_associative_features(
        self, token_ids: Tensor, state: DecoderState, *, state_off: bool
    ) -> tuple[Tensor, DecoderState, Tensor, Tensor]:
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
        exact_features: list[Tensor] = []
        pooled_features: list[Tensor] = []
        for layer_index, layer in enumerate(self.layers):
            (
                values,
                keys,
                layer_values,
                layer_occupied,
                _,
                strict_prior_values,
                read_occupied,
            ) = layer.forward_stationary_with_retained_value_field(
                values,
                state.keys[layer_index],
                state.values[layer_index],
                state.occupied[layer_index],
                prefix_actions,
                self.identity_offset,
                state_off=state_off,
            )
            exact, pooled = self._features_from_value_field(
                strict_prior_values, read_occupied
            )
            exact_features.append(exact)
            pooled_features.append(pooled)
            final_keys.append(keys)
            final_values.append(layer_values)
            final_occupied.append(layer_occupied)
        return (
            values,
            DecoderState(
                keys=torch.stack(final_keys),
                values=torch.stack(final_values),
                occupied=torch.stack(final_occupied),
            ),
            torch.stack(exact_features, dim=2),
            torch.stack(pooled_features, dim=2),
        )

    def _direct_hidden_with_associative_features(
        self, token_ids: Tensor, state: DecoderState, *, state_off: bool
    ) -> tuple[Tensor, DecoderState, Tensor, Tensor]:
        outputs: list[Tensor] = []
        exact_outputs: list[Tensor] = []
        pooled_outputs: list[Tensor] = []
        current = state
        for time_index in range(int(token_ids.shape[1])):
            token = token_ids[:, time_index]
            values = self.token_embedding(token)
            leaves = self.token_leaves.index_select(0, token)
            actions = self.left_actions.index_select(0, leaves)
            keys: list[Tensor] = []
            retained_values: list[Tensor] = []
            occupied: list[Tensor] = []
            exact_layers: list[Tensor] = []
            pooled_layers: list[Tensor] = []
            for layer_index, layer in enumerate(self.layers):
                (
                    values,
                    layer_keys,
                    layer_values,
                    layer_occupied,
                    _,
                    strict_prior_values,
                    read_occupied,
                ) = layer.forward_direct_step_with_retained_value_field(
                    values,
                    actions,
                    current.keys[layer_index],
                    current.values[layer_index],
                    current.occupied[layer_index],
                    self.identity_offset,
                    state_off=state_off,
                )
                exact, pooled = self._direct_features_from_value_field(
                    strict_prior_values, read_occupied
                )
                exact_layers.append(exact)
                pooled_layers.append(pooled)
                keys.append(layer_keys)
                retained_values.append(layer_values)
                occupied.append(layer_occupied)
            current = DecoderState(
                keys=torch.stack(keys),
                values=torch.stack(retained_values),
                occupied=torch.stack(occupied),
            )
            outputs.append(values)
            exact_outputs.append(torch.stack(exact_layers, dim=1))
            pooled_outputs.append(torch.stack(pooled_layers, dim=1))
        return (
            torch.stack(outputs, dim=1),
            current,
            torch.stack(exact_outputs, dim=1),
            torch.stack(pooled_outputs, dim=1),
        )

    def _hidden_with_associative_features(
        self,
        token_ids: Tensor,
        targets: Tensor | None,
        initial_state: DecoderState | None,
        *,
        attention_off: bool,
        implementation: Literal["stationary", "direct"],
    ) -> tuple[Tensor, DecoderState, Tensor, Tensor]:
        state = (
            self.initial_state(int(token_ids.shape[0]))
            if initial_state is None
            else initial_state
        )
        self._validate_inputs(token_ids, targets, state)
        if implementation == "stationary":
            return self._stationary_hidden_with_associative_features(
                token_ids, state, state_off=attention_off
            )
        if implementation == "direct":
            return self._direct_hidden_with_associative_features(
                token_ids, state, state_off=attention_off
            )
        raise ValueError("implementation must be 'stationary' or 'direct'")

    def _grouped_score(self, features: Tensor, query_table: Tensor) -> Tensor:
        """Score grouped candidates without materializing ``[B,T,V,48]``."""

        layers, vocabulary, _, _ = query_table.shape
        if (layers, vocabulary) != (LAYERS, VOCAB_SIZE):
            raise RuntimeError("associative query-table ledger drifted")
        queries = query_table.view(LAYERS, VOCAB_SIZE, HIDDEN_SIZE)
        grouped_queries = queries.index_select(
            1, self.candidate_group_indices.reshape(-1)
        ).view(
            LAYERS,
            self.used_candidate_leaf_count,
            self.maximum_candidates_per_leaf,
            HIDDEN_SIZE,
        )
        grouped_scores = torch.einsum(
            "btlud,lucd->btuc", features.float(), grouped_queries.float()
        )
        ordered = grouped_scores.flatten(2).index_select(
            2, self.candidate_group_flat_valid_indices
        )
        return ordered.index_select(2, self.candidate_score_positions) * ASSOCIATIVE_SCALE

    def _score_for_arm(
        self,
        arm: Arm,
        exact_features: Tensor,
        pooled_features: Tensor,
    ) -> Tensor:
        if arm == "geometric":
            features = exact_features
            queries = self.geometric_queries
        elif arm == "pooled":
            features = pooled_features.unsqueeze(3).expand(
                -1, -1, -1, self.used_candidate_leaf_count, -1
            )
            queries = self.pooled_queries
        elif arm == "deranged":
            features = exact_features.index_select(3, self.deranged_group_positions)
            queries = self.geometric_queries
        else:
            raise ValueError("arm must be 'geometric', 'pooled', or 'deranged'")
        return self._grouped_score(features, queries)

    def _associative_audit(
        self,
        batch_size: int,
        time: int,
        *,
        arm: Arm,
        attention_off: bool,
        head_off: bool,
        implementation: str,
    ) -> LearnedAssociativeReadoutAudit:
        base = self._audit(
            batch_size,
            time,
            state_off=attention_off,
            implementation=implementation,
        )
        token_steps = batch_size * time
        field_values = LAYERS * HEADS * self.config.group_size * HEAD_DIM
        exact_values = LAYERS * self.used_candidate_leaf_count * HIDDEN_SIZE
        normalized_values = LAYERS * (self.used_candidate_leaf_count + 1) * HIDDEN_SIZE
        return LearnedAssociativeReadoutAudit(
            **asdict(base),
            arm=arm,
            head_off=head_off,
            used_candidate_leaves=self.used_candidate_leaf_count,
            strict_prior_value_field_reads=token_steps * field_values,
            exact_leaf_value_reads=token_steps * exact_values,
            occupied_pool_value_reads=token_steps * field_values,
            normalized_value_count=token_steps * normalized_values,
            grouped_candidate_dot_products=token_steps * LAYERS * VOCAB_SIZE,
            grouped_executed_dot_products=(
                token_steps
                * LAYERS
                * self.used_candidate_leaf_count
                * self.maximum_candidates_per_leaf
            ),
            associative_logit_additions=token_steps * VOCAB_SIZE,
        )

    def _arm_output(
        self,
        arm: Arm,
        base_logits: Tensor,
        associative_scores: Tensor,
        targets: Tensor | None,
        final_state: DecoderState,
        *,
        attention_off: bool,
        head_off: bool,
        implementation: str,
    ) -> DecoderOutput:
        logits = (
            base_logits
            if attention_off or head_off
            else base_logits + associative_scores.to(base_logits.dtype)
        )
        loss = None
        if targets is not None:
            loss = F.cross_entropy(
                logits.float().reshape(-1, VOCAB_SIZE), targets.reshape(-1)
            )
        return DecoderOutput(
            logits=logits,
            loss=loss,
            final_state=final_state,
            audit=self._associative_audit(
                int(logits.shape[0]),
                int(logits.shape[1]),
                arm=arm,
                attention_off=attention_off,
                head_off=head_off,
                implementation=implementation,
            ),
        )

    def forward(
        self,
        token_ids: Tensor,
        targets: Tensor | None = None,
        *,
        attention_off: bool = False,
        head_off: bool = False,
        initial_state: DecoderState | None = None,
        implementation: Literal["stationary", "direct"] = "stationary",
    ) -> LearnedAssociativeReadoutBundleOutput:
        """Evaluate both independent heads on one frozen-base feature pass."""

        if token_ids.ndim != 2:
            raise ValueError("token_ids must have shape [batch,time]")
        hidden, final_state, exact, pooled = self._hidden_with_associative_features(
            token_ids,
            targets,
            initial_state,
            attention_off=attention_off,
            implementation=implementation,
        )
        base_logits = F.linear(self.final_norm(hidden), self.output_weight)
        geometric_scores = self._score_for_arm("geometric", exact, pooled)
        pooled_scores = self._score_for_arm("pooled", exact, pooled)
        return LearnedAssociativeReadoutBundleOutput(
            geometric=self._arm_output(
                "geometric",
                base_logits,
                geometric_scores,
                targets,
                final_state,
                attention_off=attention_off,
                head_off=head_off,
                implementation=implementation,
            ),
            pooled=self._arm_output(
                "pooled",
                base_logits,
                pooled_scores,
                targets,
                final_state,
                attention_off=attention_off,
                head_off=head_off,
                implementation=implementation,
            ),
        )

    def forward_arm(
        self,
        arm: Arm,
        token_ids: Tensor,
        targets: Tensor | None = None,
        *,
        attention_off: bool = False,
        head_off: bool = False,
        initial_state: DecoderState | None = None,
        implementation: Literal["stationary", "direct"] = "stationary",
    ) -> DecoderOutput:
        """Evaluate one head/control while still constructing both feature views."""

        if token_ids.ndim != 2:
            raise ValueError("token_ids must have shape [batch,time]")
        hidden, final_state, exact, pooled = self._hidden_with_associative_features(
            token_ids,
            targets,
            initial_state,
            attention_off=attention_off,
            implementation=implementation,
        )
        base_logits = F.linear(self.final_norm(hidden), self.output_weight)
        scores = self._score_for_arm(arm, exact, pooled)
        return self._arm_output(
            arm,
            base_logits,
            scores,
            targets,
            final_state,
            attention_off=attention_off,
            head_off=head_off,
            implementation=implementation,
        )

    def forward_incremental(
        self,
        token_ids: Tensor,
        targets: Tensor | None = None,
        *,
        attention_off: bool = False,
        head_off: bool = False,
        initial_state: DecoderState | None = None,
    ) -> LearnedAssociativeReadoutBundleOutput:
        return self.forward(
            token_ids,
            targets,
            attention_off=attention_off,
            head_off=head_off,
            initial_state=initial_state,
            implementation="direct",
        )

    def forward_incremental_arm(
        self,
        arm: Arm,
        token_ids: Tensor,
        targets: Tensor | None = None,
        *,
        attention_off: bool = False,
        head_off: bool = False,
        initial_state: DecoderState | None = None,
    ) -> DecoderOutput:
        return self.forward_arm(
            arm,
            token_ids,
            targets,
            attention_off=attention_off,
            head_off=head_off,
            initial_state=initial_state,
            implementation="direct",
        )

    def step(
        self,
        token_ids: Tensor,
        state: DecoderState,
        *,
        arm: Arm = "geometric",
        attention_off: bool = False,
        head_off: bool = False,
    ) -> DecoderStepOutput:
        if token_ids.ndim != 1:
            raise ValueError("incremental token_ids must have shape [batch]")
        output = self.forward_incremental_arm(
            arm,
            token_ids[:, None],
            attention_off=attention_off,
            head_off=head_off,
            initial_state=state,
        )
        return DecoderStepOutput(
            logits=output.logits[:, 0, :],
            final_state=output.final_state,
            audit=output.audit,
        )


__all__ = [
    "ASSOCIATIVE_SCALE",
    "BUNDLE_PARAMETER_COUNT",
    "EFFECTIVE_ARM_PARAMETER_COUNT",
    "HEAD_PARAMETER_COUNT",
    "POLICY",
    "QUERY_SHAPE",
    "LearnedAssociativeReadoutAudit",
    "LearnedAssociativeReadoutBundleOutput",
    "R4LearnedCandidateLeafAssociativeReadoutV1",
]
