"""Role-tagged associative curriculum model for issue #1045.

The learned attention law stays exactly at the frozen #1043 boundary.  This
module adds one causal, categorical role input at the embedding boundary and a
query-only vocabulary projection used by the open associative curriculum.
Roles never participate in H4 frame accumulation: canonical frames remain a
function of admitted token IDs alone.
"""

from __future__ import annotations

from contextlib import contextmanager
from contextvars import ContextVar
from dataclasses import dataclass, replace
from typing import TYPE_CHECKING, Iterator

import torch
from safetensors.torch import load as load_safetensors
from torch import Tensor, nn
from torch.nn import functional as F

from .group_retention import GroupAddressArtifact
from .language_path_generalization import (
    CONTEXT,
    HEADS,
    HIDDEN_SIZE,
    LAYERS,
    PARAMETER_COUNT as BASE_PARAMETER_COUNT,
    STATE_BYTES_F32,
    STATE_VALUES,
    VALIDITY_BITS,
    VOCAB_SIZE,
)
from .position_kv_binding import (
    Execution,
    Intervention,
    PositionKVBindingAudit,
    PositionKVBindingOutput,
    PositionKVBindingStepOutput,
    PositionKVCacheState,
    R4PositionPreservingCausalKVBindingV1,
)

if TYPE_CHECKING:
    from .h4_spin_frame_sidecar import H4SpinFrameArtifactV1


POLICY = "R4RoleTaggedAssociativeCurriculumV1"

TEXT_ROLE = 0
KEY_ROLE = 1
VALUE_ROLE = 2
QUERY_ROLE = 3
ROLE_COUNT = 4
ROLE_PARAMETER_COUNT = ROLE_COUNT * HIDDEN_SIZE
PARAMETER_COUNT = BASE_PARAMETER_COUNT + ROLE_PARAMETER_COUNT


@dataclass(slots=True)
class RoleTaggedAssociativeQueryOutput:
    """Query-only logits plus the unchanged position-K/V evidence surface.

    ``selected_positions`` identifies input positions whose hidden states are
    projected to next-token logits.  Labels use the existing #1043 convention:
    the target for a selected position is stored at that same position.
    """

    logits: Tensor
    loss: Tensor | None
    selected_positions: Tensor
    selected_targets: Tensor | None
    final_state: PositionKVCacheState
    audit: PositionKVBindingAudit
    # [layers,batch,heads,time,context], with unadmitted slots exactly zero.
    attention_weights: Tensor


class R4RoleTaggedAssociativeCurriculumV1(
    R4PositionPreservingCausalKVBindingV1
):
    """Frozen causal-softmax/R4 attention with a four-way causal role input."""

    def __init__(
        self,
        geometry: GroupAddressArtifact,
        frames: H4SpinFrameArtifactV1,
    ) -> None:
        super().__init__(geometry, frames)
        self.role_embedding = nn.Embedding(
            ROLE_COUNT,
            HIDDEN_SIZE,
            padding_idx=TEXT_ROLE,
        )
        with torch.no_grad():
            self.role_embedding.weight.zero_()

        # The frozen parent calls ``token_embedding`` directly.  A context-local
        # hook lets those exact mechanics consume roles without replacing the
        # tied token table or changing its artifact parameter name.
        self._active_role_ids: ContextVar[Tensor | None] = ContextVar(
            f"{POLICY}.active_role_ids",
            default=None,
        )
        self._role_embedding_hook_handle = self.token_embedding.register_forward_hook(
            self._add_active_role_embedding
        )

        if self.parameter_count() != PARAMETER_COUNT:
            raise RuntimeError("role-tagged associative parameter ledger differs")
        if self.state_value_count() != STATE_VALUES:
            raise RuntimeError("role-tagged associative state ledger differs")
        if self.state_byte_count_f32() != STATE_BYTES_F32:
            raise RuntimeError("role-tagged associative state-byte ledger differs")
        if self.validity_bit_count() != VALIDITY_BITS:
            raise RuntimeError("role-tagged associative validity ledger differs")

    def _add_active_role_embedding(
        self,
        _module: nn.Module,
        inputs: tuple[object, ...],
        output: Tensor,
    ) -> Tensor:
        role_ids = self._active_role_ids.get()
        if role_ids is None:
            return output
        token_ids = inputs[0]
        if not isinstance(token_ids, Tensor) or token_ids.shape != role_ids.shape:
            raise RuntimeError("active roles do not match the embedded token tensor")
        return output + self.role_embedding(role_ids.long())

    @contextmanager
    def _using_roles(self, role_ids: Tensor) -> Iterator[None]:
        marker = self._active_role_ids.set(role_ids)
        try:
            yield
        finally:
            self._active_role_ids.reset(marker)

    @staticmethod
    def _validate_roles(token_ids: Tensor, role_ids: Tensor) -> None:
        if role_ids.dtype != torch.uint8 or role_ids.shape != token_ids.shape:
            raise ValueError("role_ids must be uint8 and match token_ids")
        if role_ids.device != token_ids.device:
            raise ValueError("role_ids and token_ids must share a device")
        if bool((role_ids >= ROLE_COUNT).any()):
            raise ValueError("role_ids contain an unsupported categorical role")

    @staticmethod
    def _validate_selected_positions(
        token_ids: Tensor,
        selected_positions: Tensor,
    ) -> None:
        batch, time = token_ids.shape
        if (
            selected_positions.ndim != 2
            or selected_positions.dtype != torch.long
            or selected_positions.shape[0] != batch
            or selected_positions.shape[1] < 1
        ):
            raise ValueError("selected_positions must be int64 [batch,queries]")
        if selected_positions.device != token_ids.device:
            raise ValueError("selected_positions and token_ids must share a device")
        if bool((selected_positions < 0).any()) or bool(
            (selected_positions >= time).any()
        ):
            raise ValueError("selected_positions contain an out-of-prefix position")
        if selected_positions.shape[1] > 1:
            ordered = selected_positions.sort(dim=1).values
            if bool((ordered[:, 1:] == ordered[:, :-1]).any()):
                raise ValueError("selected_positions must be unique within each row")

    def load_ordinary_artifact(self, payload: bytes) -> None:
        """Load only the original decoder tensors and reset all role rows.

        The accepted schema is the ordinary/#1043 base schema and deliberately
        excludes ``role_embedding.weight``.  Campaign provenance separately
        binds ``payload`` to ``inputs/ordinary-initialization.safetensors``.
        """

        loaded = load_safetensors(payload)
        expected = {
            name: parameter
            for name, parameter in self.named_parameters()
            if name != "role_embedding.weight"
        }
        if set(loaded) != set(expected):
            raise ValueError(
                "ordinary initialization parameter names differ from the frozen decoder"
            )
        with torch.no_grad():
            for name in sorted(expected):
                source = loaded[name]
                target = expected[name]
                if source.dtype != target.dtype or tuple(source.shape) != tuple(
                    target.shape
                ):
                    raise ValueError(
                        f"ordinary initialization tensor contract differs for {name}"
                    )
                target.copy_(source.to(device=target.device))
            self.role_embedding.weight.zero_()

    @classmethod
    def from_ordinary_artifact(
        cls,
        payload: bytes,
        *,
        geometry: GroupAddressArtifact,
        frames: H4SpinFrameArtifactV1,
    ) -> R4RoleTaggedAssociativeCurriculumV1:
        """Construct from the independently frozen ordinary initialization."""

        model = cls(geometry, frames)
        model.load_ordinary_artifact(payload)
        return model

    @staticmethod
    def _selected_targets(
        token_ids: Tensor,
        selected_positions: Tensor,
        targets: Tensor | None,
    ) -> Tensor | None:
        if targets is None:
            return None
        if targets.dtype != torch.long or targets.device != token_ids.device:
            raise ValueError("targets must be int64 on the token device")
        selected_shape = selected_positions.shape
        if targets.shape == token_ids.shape:
            selected = torch.gather(targets, 1, selected_positions)
        elif targets.shape == selected_shape:
            selected = targets
        else:
            raise ValueError("targets must match token_ids or selected_positions")
        valid = selected != -100
        if bool(valid.any()):
            admitted = selected[valid]
            if bool((admitted < 0).any()) or bool((admitted >= VOCAB_SIZE).any()):
                raise ValueError("selected targets contain an out-of-vocabulary value")
        return selected

    def _full_query_forward(
        self,
        token_ids: Tensor,
        role_ids: Tensor,
        selected_positions: Tensor,
        targets: Tensor | None,
        *,
        execution: Execution,
        intervention: Intervention,
    ) -> RoleTaggedAssociativeQueryOutput:
        """Run full causal attention but project only selected hidden states."""

        batch, time = token_ids.shape
        query_count = int(selected_positions.shape[1])
        selected_targets = self._selected_targets(
            token_ids, selected_positions, targets
        )
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
        with self._using_roles(role_ids):
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

        gather = selected_positions.unsqueeze(-1).expand(
            batch, query_count, HIDDEN_SIZE
        )
        selected_hidden = torch.gather(values, 1, gather)
        selected_hidden = self.final_norm(selected_hidden)
        logits = F.linear(selected_hidden, self.output_weight)
        loss = None
        if selected_targets is not None:
            loss = F.cross_entropy(
                logits.float().reshape(-1, VOCAB_SIZE),
                selected_targets.reshape(-1),
            )

        target_reads = (
            0
            if selected_targets is None
            else int(torch.count_nonzero(selected_targets != -100))
        )
        audit = self._call_audit(
            execution=execution,
            intervention=intervention,
            batch_size=batch,
            time=time,
            prior_length=0,
            target_reads=target_reads,
            full_square=True,
        )
        audit = replace(
            audit,
            vocabulary_scores=batch * query_count * VOCAB_SIZE,
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
        return RoleTaggedAssociativeQueryOutput(
            logits=logits,
            loss=loss,
            selected_positions=selected_positions,
            selected_targets=selected_targets,
            final_state=final_state,
            audit=audit,
            attention_weights=torch.stack(layer_weights, dim=0),
        )

    def step(
        self,
        token_ids: Tensor,
        role_ids: Tensor,
        state: PositionKVCacheState,
        *,
        execution: Execution = "plain",
        intervention: Intervention = "native",
    ) -> PositionKVBindingStepOutput:
        """Append one token and its causally derived role to the exact cache."""

        self._validate_roles(token_ids, role_ids)
        with self._using_roles(role_ids):
            return super().step(
                token_ids,
                state,
                execution=execution,
                intervention=intervention,
            )

    def forward_incremental(
        self,
        token_ids: Tensor,
        role_ids: Tensor,
        targets: Tensor | None = None,
        *,
        execution: Execution = "plain",
        intervention: Intervention = "native",
        initial_state: PositionKVCacheState | None = None,
    ) -> PositionKVBindingOutput:
        """Run the role-aware real cache-backed path for one token block."""

        self._validate_policy(execution, intervention)
        self._validate_inputs(token_ids, targets)
        self._validate_roles(token_ids, role_ids)
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
                role_ids[:, position],
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
        role_ids: Tensor,
        targets: Tensor | None = None,
        *,
        selected_positions: Tensor | None = None,
        execution: Execution = "plain",
        intervention: Intervention = "native",
        initial_state: PositionKVCacheState | None = None,
    ) -> PositionKVBindingOutput | RoleTaggedAssociativeQueryOutput:
        """Run full logits or an allocation-saving selected-query projection."""

        self._validate_policy(execution, intervention)
        validation_targets = None if selected_positions is not None else targets
        self._validate_inputs(token_ids, validation_targets)
        self._validate_roles(token_ids, role_ids)
        if selected_positions is not None:
            if initial_state is not None:
                raise ValueError(
                    "selected query projection requires a complete admitted prefix"
                )
            self._validate_selected_positions(token_ids, selected_positions)
            return self._full_query_forward(
                token_ids,
                role_ids,
                selected_positions,
                targets,
                execution=execution,
                intervention=intervention,
            )
        if initial_state is not None:
            return self.forward_incremental(
                token_ids,
                role_ids,
                targets,
                execution=execution,
                intervention=intervention,
                initial_state=initial_state,
            )
        with self._using_roles(role_ids):
            return super()._full_forward(
                token_ids,
                targets,
                execution=execution,
                intervention=intervention,
            )
