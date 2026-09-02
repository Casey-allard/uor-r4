"""Predictive R4 block-delta binding over the immutable retained V1 backbone.

This is the bounded write-law successor frozen for issue #973.  The qualified
language model supplies causal prefix features and logits; only four bias-free
48-by-48 maps and twelve bank-gate logits are learned here.  A fixed-size bank
of twelve independent R4 linear maps binds the previous causal context to the
token that is subsequently observed.
"""

from __future__ import annotations

import math
from collections.abc import Iterable
from dataclasses import asdict, dataclass
from typing import TYPE_CHECKING, Literal

import torch
from safetensors.torch import load as load_safetensors
from safetensors.torch import save as save_safetensors
from torch import Tensor, nn
from torch.nn import functional as F

from .group_retention import GroupAddressArtifact
from .group_retention_decoder import (
    DecoderAudit,
    DecoderState,
    R4GroupAddressedRetentionDecoderV1,
)
from .language_path_generalization import (
    DECAY_HALF_LIVES,
    HIDDEN_SIZE,
    PARAMETER_COUNT,
    VOCAB_SIZE,
    R4RetainedLanguagePathV1,
)

if TYPE_CHECKING:
    from .h4_spin_frame_sidecar import H4SpinFrameArtifactV1


POLICY = "R4PredictiveBlockDeltaBindingV1"
R4_WIDTH = 4
R4_BLOCKS = HIDDEN_SIZE // R4_WIDTH
BANKS = 4
INITIALIZATION_SEED = 9_739
INITIALIZATION_STD = 0.02
TRAINABLE_PARAMETER_COUNT = 4 * HIDDEN_SIZE * HIDDEN_SIZE + 3 * BANKS
MATRIX_STATE_VALUES = BANKS * R4_BLOCKS * R4_WIDTH * R4_WIDTH
PREVIOUS_KEY_VALUES = R4_BLOCKS * R4_WIDTH
BINDING_STATE_VALUES = MATRIX_STATE_VALUES + PREVIOUS_KEY_VALUES
BINDING_STATE_BYTES_F32 = BINDING_STATE_VALUES * 4
LOGIT_SCALE = 1.0 / math.sqrt(HIDDEN_SIZE)

Arm = Literal["geometric", "plain"]
Intervention = Literal["native", "transport_permuted", "no_delta", "state_off"]


@dataclass(slots=True)
class PredictiveBlockDeltaState:
    """Qualified V1 state plus the bounded predictive binding state."""

    backbone: DecoderState
    matrices: Tensor
    previous_key: Tensor
    frame_indices: Tensor
    key_valid: Tensor


@dataclass(frozen=True, slots=True)
class PredictiveBlockDeltaAudit(DecoderAudit):
    """Executed work ledger; labels are deliberately absent from its signature."""

    arm: str = ""
    intervention: str = ""
    binding_state_transports: int = 0
    previous_key_transports: int = 0
    delta_predictions: int = 0
    delta_outer_products: int = 0
    hebbian_outer_products: int = 0
    memory_reads: int = 0
    value_anchor_transforms: int = 0
    candidate_anchor_transforms: int = 0
    candidate_dot_products: int = 0
    binding_logit_additions: int = 0

    def work_signature(self) -> tuple[int, ...]:
        return (
            *DecoderAudit.work_signature(self),
            self.binding_state_transports,
            self.previous_key_transports,
            self.delta_predictions,
            self.delta_outer_products,
            self.hebbian_outer_products,
            self.memory_reads,
            self.value_anchor_transforms,
            self.candidate_anchor_transforms,
            self.candidate_dot_products,
            self.binding_logit_additions,
        )


@dataclass(slots=True)
class PredictiveBlockDeltaOutput:
    logits: Tensor
    loss: Tensor | None
    final_state: PredictiveBlockDeltaState
    audit: PredictiveBlockDeltaAudit
    base_logits: Tensor
    head_logits: Tensor


@dataclass(slots=True)
class PredictiveBlockDeltaStepOutput:
    logits: Tensor
    final_state: PredictiveBlockDeltaState
    audit: PredictiveBlockDeltaAudit


class R4PredictiveBlockDeltaBindingV1(R4RetainedLanguagePathV1):
    """Frozen retained V1 plus one predictive geometric delta-memory arm."""

    def __init__(
        self,
        geometry: GroupAddressArtifact,
        frames: H4SpinFrameArtifactV1,
        *,
        arm: Arm,
    ) -> None:
        if arm not in ("geometric", "plain"):
            raise ValueError("arm must be 'geometric' or 'plain'")
        super().__init__(geometry)
        self.binding_arm = arm
        self._qualified_base_parameter_names = tuple(
            name for name, _ in self.named_parameters()
        )
        for parameter in self.parameters():
            parameter.requires_grad_(False)

        validate_frames = getattr(frames, "validate", None)
        if callable(validate_frames):
            validate_frames(group_size=120)

        frame_matrices = torch.as_tensor(frames.frame_matrices, dtype=torch.float32)
        multiplication = torch.as_tensor(frames.multiplication_indices, dtype=torch.long)
        inverses = torch.as_tensor(frames.inverse_indices, dtype=torch.long)
        permutation = torch.as_tensor(
            frames.transport_permutation, dtype=torch.long
        )
        if tuple(frame_matrices.shape) != (120, R4_WIDTH, R4_WIDTH):
            raise ValueError("H4 frame matrices must have shape [120,4,4]")
        if tuple(multiplication.shape) != (120, 120):
            raise ValueError("H4 multiplication table must have shape [120,120]")
        if tuple(inverses.shape) != (120,):
            raise ValueError("H4 inverse table must have shape [120]")
        if tuple(permutation.shape) != (120,):
            raise ValueError("transport permutation must have shape [120]")
        if int(frames.identity_index) != self.identity_offset:
            raise ValueError("H4 frame identity differs from the qualified geometry")
        if not torch.equal(multiplication.cpu(), self.left_actions.cpu()):
            raise ValueError("H4 frame multiplication differs from qualified geometry")
        expected = torch.arange(120, dtype=torch.long)
        if not torch.equal(permutation.sort().values.cpu(), expected):
            raise ValueError("transport permutation must be bijective")
        if int(permutation[self.identity_offset]) != self.identity_offset:
            raise ValueError("transport permutation must fix the H4 identity")
        if not bool(torch.isfinite(frame_matrices).all()):
            raise ValueError("H4 frame matrices must be finite")

        self.frame_artifact_cid = frames.artifact_cid
        self.register_buffer("frame_matrices", frame_matrices.contiguous())
        self.register_buffer("frame_multiplication", multiplication.contiguous())
        self.register_buffer("frame_inverses", inverses.contiguous())
        self.register_buffer("transport_permutation", permutation.contiguous())

        used_leaves, grouped_indices, group_mask = self._build_candidate_groups(
            self.token_leaves
        )
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

        self.wq = nn.Parameter(torch.empty(HIDDEN_SIZE, HIDDEN_SIZE))
        self.wk = nn.Parameter(torch.empty(HIDDEN_SIZE, HIDDEN_SIZE))
        self.wv = nn.Parameter(torch.empty(HIDDEN_SIZE, HIDDEN_SIZE))
        self.we = nn.Parameter(torch.empty(HIDDEN_SIZE, HIDDEN_SIZE))
        half_lives = torch.tensor(DECAY_HALF_LIVES, dtype=torch.float32)
        rho = torch.exp(math.log(0.5) / half_lives)
        self.rho_logits = nn.Parameter(torch.logit(rho))
        self.eta_logits = nn.Parameter(torch.zeros(BANKS))
        self.alpha_logits = nn.Parameter(torch.zeros(BANKS))
        self._initialize_binding_weights()

        if self.trainable_parameter_count() != TRAINABLE_PARAMETER_COUNT:
            raise RuntimeError("predictive binding trainable-parameter ledger drifted")
        if self.parameter_count() != PARAMETER_COUNT + TRAINABLE_PARAMETER_COUNT:
            raise RuntimeError("predictive binding total-parameter ledger drifted")

    @staticmethod
    def _build_candidate_groups(leaves: Tensor) -> tuple[Tensor, Tensor, Tensor]:
        used = torch.unique(leaves.detach().cpu(), sorted=True)
        members = [
            torch.nonzero(leaves.detach().cpu() == leaf, as_tuple=False).flatten()
            for leaf in used
        ]
        maximum = max(int(member.numel()) for member in members)
        indices = torch.zeros(len(members), maximum, dtype=torch.long)
        mask = torch.zeros(len(members), maximum, dtype=torch.bool)
        for row, member in enumerate(members):
            width = int(member.numel())
            indices[row, :width] = member
            mask[row, :width] = True
        return used, indices, mask

    def _initialize_binding_weights(self) -> None:
        generator = torch.Generator(device="cpu")
        generator.manual_seed(INITIALIZATION_SEED)
        with torch.no_grad():
            for parameter in (self.wq, self.wk, self.wv, self.we):
                parameter.normal_(0.0, INITIALIZATION_STD, generator=generator)

    def trainable_parameters(self) -> Iterable[nn.Parameter]:
        return (
            self.wq,
            self.wk,
            self.wv,
            self.we,
            self.rho_logits,
            self.eta_logits,
            self.alpha_logits,
        )

    def trainable_parameter_count(self) -> int:
        return sum(parameter.numel() for parameter in self.trainable_parameters())

    def frozen_base_parameters(self) -> Iterable[nn.Parameter]:
        expected = set(self._qualified_base_parameter_names)
        return (
            parameter
            for name, parameter in self.named_parameters()
            if name in expected
        )

    def export_qualified_base_artifact(self) -> bytes:
        expected = set(self._qualified_base_parameter_names)
        tensors = {
            name: parameter.detach().cpu().contiguous()
            for name, parameter in sorted(self.named_parameters())
            if name in expected
        }
        return save_safetensors(tensors)

    def load_qualified_base_artifact(self, payload: bytes) -> None:
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
                    raise ValueError(f"qualified base tensor contract differs for {name}")
                if not bool(torch.isfinite(source).all()):
                    raise ValueError(f"qualified base tensor is nonfinite: {name}")
                target.copy_(source.to(device=target.device))

    def export_binding_artifact(self) -> bytes:
        names = (
            "alpha_logits",
            "eta_logits",
            "rho_logits",
            "we",
            "wk",
            "wq",
            "wv",
        )
        parameters = dict(self.named_parameters())
        return save_safetensors(
            {name: parameters[name].detach().cpu().contiguous() for name in names}
        )

    def load_binding_artifact(self, payload: bytes) -> None:
        loaded = load_safetensors(payload)
        parameters = dict(self.named_parameters())
        expected = {
            "alpha_logits",
            "eta_logits",
            "rho_logits",
            "we",
            "wk",
            "wq",
            "wv",
        }
        if set(loaded) != expected:
            raise ValueError("predictive binding artifact parameter names differ")
        with torch.no_grad():
            for name in sorted(expected):
                source = loaded[name]
                target = parameters[name]
                if source.dtype != target.dtype or tuple(source.shape) != tuple(target.shape):
                    raise ValueError(f"binding artifact tensor contract differs for {name}")
                if not bool(torch.isfinite(source).all()):
                    raise ValueError(f"binding artifact tensor is nonfinite: {name}")
                target.copy_(source.to(device=target.device))

    def initial_backbone_state(
        self,
        batch_size: int,
        *,
        device: torch.device | str | None = None,
        dtype: torch.dtype | None = None,
    ) -> DecoderState:
        return R4GroupAddressedRetentionDecoderV1.initial_state(
            self, batch_size, device=device, dtype=dtype
        )

    def initial_state(
        self,
        batch_size: int,
        *,
        device: torch.device | str | None = None,
        dtype: torch.dtype | None = None,
    ) -> PredictiveBlockDeltaState:
        backbone = self.initial_backbone_state(batch_size, device=device, dtype=dtype)
        resolved_device = backbone.keys.device
        resolved_dtype = backbone.keys.dtype
        matrices = torch.zeros(
            batch_size,
            BANKS,
            R4_BLOCKS,
            R4_WIDTH,
            R4_WIDTH,
            device=resolved_device,
            dtype=resolved_dtype,
        )
        previous_key = torch.zeros(
            batch_size,
            R4_BLOCKS,
            R4_WIDTH,
            device=resolved_device,
            dtype=resolved_dtype,
        )
        frame_indices = torch.full(
            (batch_size,),
            self.identity_offset,
            device=resolved_device,
            dtype=torch.long,
        )
        key_valid = torch.zeros(batch_size, device=resolved_device, dtype=torch.bool)
        return PredictiveBlockDeltaState(
            backbone=backbone,
            matrices=matrices,
            previous_key=previous_key,
            frame_indices=frame_indices,
            key_valid=key_valid,
        )

    def _validate_predictive_state(
        self, token_ids: Tensor, targets: Tensor | None, state: PredictiveBlockDeltaState
    ) -> None:
        R4GroupAddressedRetentionDecoderV1._validate_inputs(
            self, token_ids, targets, state.backbone
        )
        batch = int(token_ids.shape[0])
        if tuple(state.matrices.shape) != (batch, BANKS, R4_BLOCKS, 4, 4):
            raise ValueError("binding matrices have the wrong shape")
        if tuple(state.previous_key.shape) != (batch, R4_BLOCKS, 4):
            raise ValueError("previous binding key has the wrong shape")
        if tuple(state.frame_indices.shape) != (batch,) or state.frame_indices.dtype != torch.long:
            raise ValueError("frame indices must be int64 [batch]")
        if tuple(state.key_valid.shape) != (batch,) or state.key_valid.dtype != torch.bool:
            raise ValueError("key-valid mask must be bool [batch]")
        tensors = (
            state.matrices,
            state.previous_key,
            state.frame_indices,
            state.key_valid,
        )
        if any(tensor.device != token_ids.device for tensor in tensors):
            raise ValueError("binding state and tokens must share a device")
        if state.matrices.dtype != self.wq.dtype or state.previous_key.dtype != self.wq.dtype:
            raise ValueError("binding f32 state must match learned-parameter dtype")
        if bool((state.frame_indices < 0).any()) or bool((state.frame_indices >= 120).any()):
            raise ValueError("binding state contains an out-of-range frame")

    @staticmethod
    def _normalize_key_blocks(key: Tensor) -> Tensor:
        norm = torch.linalg.vector_norm(key.float(), dim=-1, keepdim=True)
        return key / norm.clamp_min(1e-6).to(key.dtype)

    def _frames_for_arm(self, indices: Tensor) -> Tensor:
        selected = self.frame_matrices.index_select(0, indices)
        if self.binding_arm == "geometric":
            return selected
        identity = torch.eye(4, dtype=selected.dtype, device=selected.device)
        return selected * 0.0 + identity

    def _grouped_candidate_values(self) -> Tensor:
        candidate_values = F.linear(self.token_embedding.weight, self.we)
        return candidate_values.index_select(
            0, self.candidate_group_indices.reshape(-1)
        ).view(
            self.used_candidate_leaves.numel(),
            self.candidate_group_indices.shape[1],
            HIDDEN_SIZE,
        )

    def _candidate_scores(
        self, current_frames: Tensor, read: Tensor, grouped_candidates: Tensor
    ) -> Tensor:
        """Score all candidates while materializing only the 35 used leaves."""

        leaf_frames = self._frames_for_arm(self.used_candidate_leaves)
        # Convert the current-frame read to each candidate leaf frame.  This is
        # algebraically the frozen dot(T(leaf->current) e_c, r_current).
        current_to_leaf = torch.matmul(
            leaf_frames.transpose(-1, -2).unsqueeze(0), current_frames[:, None]
        )
        reads_at_leaf = torch.einsum("buij,bdj->budi", current_to_leaf, read)
        reads_at_leaf = reads_at_leaf.reshape(
            read.shape[0], self.used_candidate_leaves.numel(), HIDDEN_SIZE
        )

        grouped_scores = torch.einsum(
            "bud,ucd->buc", reads_at_leaf.float(), grouped_candidates.float()
        )
        ordered = grouped_scores.flatten(1).index_select(
            1, self.candidate_group_flat_valid_indices
        )
        return ordered.index_select(1, self.candidate_score_positions) * LOGIT_SCALE

    def _step_transport(
        self,
        leaves: Tensor,
        *,
        intervention: Intervention,
    ) -> Tensor:
        """Return the local connection action for this token step.

        The canonical cumulative frame is deliberately not permuted by the
        destructive control.  Only the newly traversed leaf action is replaced
        by ``pi(leaf)``; value, query, and candidate anchors continue to use the
        true cumulative frame.  This is the continuous R4 realization of the
        registered transport-only scrambled-H4 control.
        """

        transport_leaves = leaves
        if intervention == "transport_permuted":
            if self.binding_arm != "geometric":
                raise ValueError("transport-permuted intervention requires geometric arm")
            transport_leaves = self.transport_permutation.index_select(0, leaves)
        step_frames = self._frames_for_arm(transport_leaves)
        return step_frames.transpose(-1, -2)

    def _binding_step(
        self,
        token_ids: Tensor,
        normalized_hidden: Tensor,
        state: PredictiveBlockDeltaState,
        *,
        intervention: Intervention,
    ) -> tuple[Tensor, Tensor, Tensor, Tensor]:
        leaves = self.token_leaves.index_select(0, token_ids)
        current_indices = self.frame_multiplication[
            state.frame_indices, leaves
        ]
        current_frames = self._frames_for_arm(current_indices)
        transport = self._step_transport(leaves, intervention=intervention)
        inverse_transport = transport.transpose(-1, -2)
        transported = torch.matmul(
            transport[:, None, None], state.matrices
        )
        transported = torch.matmul(
            transported, inverse_transport[:, None, None]
        )
        transported_key = torch.matmul(
            transport[:, None], state.previous_key.unsqueeze(-1)
        ).squeeze(-1)

        rho = torch.sigmoid(self.rho_logits).view(1, BANKS, 1, 1, 1)
        eta = torch.sigmoid(self.eta_logits).view(1, BANKS, 1, 1, 1)
        decayed = transported * rho

        token_values = F.linear(self.token_embedding(token_ids), self.wv).view(
            token_ids.shape[0], R4_BLOCKS, R4_WIDTH
        )
        leaf_frames = self._frames_for_arm(leaves)
        leaf_to_current = torch.matmul(
            current_frames.transpose(-1, -2), leaf_frames
        )
        anchored_values = torch.matmul(
            leaf_to_current[:, None], token_values.unsqueeze(-1)
        ).squeeze(-1)

        predicted = torch.matmul(
            decayed, transported_key[:, None, :, :, None]
        ).squeeze(-1)
        residual = anchored_values[:, None] - predicted
        delta_outer = residual.unsqueeze(-1) * transported_key[:, None, :, None, :]
        hebbian_outer = (
            anchored_values[:, None, :, :, None]
            * transported_key[:, None, :, None, :]
        )
        update = hebbian_outer if intervention == "no_delta" else delta_outer
        valid = state.key_valid[:, None, None, None, None].to(decayed.dtype)
        matrices = decayed + eta * update * valid

        query_model = F.linear(normalized_hidden, self.wq).view(
            token_ids.shape[0], R4_BLOCKS, R4_WIDTH
        )
        query = torch.matmul(
            current_frames.transpose(-1, -2)[:, None], query_model.unsqueeze(-1)
        ).squeeze(-1)
        bank_reads = torch.matmul(
            matrices, query[:, None, :, :, None]
        ).squeeze(-1)
        bank_weights = torch.softmax(self.alpha_logits.float(), dim=0).to(
            bank_reads.dtype
        )
        read = torch.einsum("k,bkdi->bdi", bank_weights, bank_reads)
        key_model = F.linear(normalized_hidden, self.wk).view(
            token_ids.shape[0], R4_BLOCKS, R4_WIDTH
        )
        next_key = torch.matmul(
            current_frames.transpose(-1, -2)[:, None], key_model.unsqueeze(-1)
        ).squeeze(-1)
        next_key = self._normalize_key_blocks(next_key)
        return matrices, next_key, current_indices, read

    def _audit_binding(
        self,
        batch_size: int,
        time: int,
        *,
        intervention: Intervention,
        implementation: str,
    ) -> PredictiveBlockDeltaAudit:
        base = R4GroupAddressedRetentionDecoderV1._audit(
            self,
            batch_size,
            time,
            state_off=False,
            implementation=implementation,
        )
        steps = batch_size * time
        return PredictiveBlockDeltaAudit(
            **asdict(base),
            arm=self.binding_arm,
            intervention=intervention,
            binding_state_transports=steps * MATRIX_STATE_VALUES,
            previous_key_transports=steps * PREVIOUS_KEY_VALUES,
            delta_predictions=steps * BANKS * R4_BLOCKS * 16,
            delta_outer_products=steps * BANKS * R4_BLOCKS * 16,
            hebbian_outer_products=steps * BANKS * R4_BLOCKS * 16,
            memory_reads=steps * BANKS * R4_BLOCKS * 16,
            value_anchor_transforms=steps * R4_BLOCKS * 16,
            candidate_anchor_transforms=(
                steps * int(self.used_candidate_leaves.numel()) * R4_BLOCKS * 16
            ),
            candidate_dot_products=steps * VOCAB_SIZE * HIDDEN_SIZE,
            binding_logit_additions=steps * VOCAB_SIZE,
        )

    def forward(
        self,
        token_ids: Tensor,
        targets: Tensor | None = None,
        *,
        initial_state: PredictiveBlockDeltaState | None = None,
        implementation: Literal["stationary", "direct"] = "stationary",
        intervention: Intervention = "native",
    ) -> PredictiveBlockDeltaOutput:
        if intervention not in (
            "native",
            "transport_permuted",
            "no_delta",
            "state_off",
        ):
            raise ValueError("unknown predictive-binding intervention")
        if token_ids.ndim != 2:
            raise ValueError("token_ids must have shape [batch,time]")
        state = (
            self.initial_state(int(token_ids.shape[0]))
            if initial_state is None
            else initial_state
        )
        self._validate_predictive_state(token_ids, targets, state)
        if implementation == "stationary":
            hidden, backbone = R4GroupAddressedRetentionDecoderV1._stationary_hidden(
                self, token_ids, state.backbone, state_off=False
            )
        elif implementation == "direct":
            hidden, backbone = R4GroupAddressedRetentionDecoderV1._direct_hidden(
                self, token_ids, state.backbone, state_off=False
            )
        else:
            raise ValueError("implementation must be 'stationary' or 'direct'")
        normalized = self.final_norm(hidden)
        base_logits = F.linear(normalized, self.output_weight)

        matrices = state.matrices
        previous_key = state.previous_key
        frame_indices = state.frame_indices
        key_valid = state.key_valid
        frame_steps: list[Tensor] = []
        read_steps: list[Tensor] = []
        for position in range(int(token_ids.shape[1])):
            step_state = PredictiveBlockDeltaState(
                backbone=state.backbone,
                matrices=matrices,
                previous_key=previous_key,
                frame_indices=frame_indices,
                key_valid=key_valid,
            )
            matrices, previous_key, frame_indices, read = self._binding_step(
                token_ids[:, position],
                normalized[:, position],
                step_state,
                intervention=intervention,
            )
            key_valid = torch.ones_like(key_valid)
            frame_steps.append(self._frames_for_arm(frame_indices))
            read_steps.append(read)
        grouped_candidates = self._grouped_candidate_values()
        head_steps = [
            self._candidate_scores(frame, read, grouped_candidates)
            for frame, read in zip(frame_steps, read_steps, strict=True)
        ]
        head_logits = torch.stack(head_steps, dim=1).to(base_logits.dtype)
        addition_scale = 0.0 if intervention == "state_off" else 1.0
        logits = base_logits + head_logits * addition_scale
        loss = None
        if targets is not None:
            loss = F.cross_entropy(
                logits.float().reshape(-1, VOCAB_SIZE), targets.reshape(-1)
            )
        final_state = PredictiveBlockDeltaState(
            backbone=backbone,
            matrices=matrices,
            previous_key=previous_key,
            frame_indices=frame_indices,
            key_valid=key_valid,
        )
        return PredictiveBlockDeltaOutput(
            logits=logits,
            loss=loss,
            final_state=final_state,
            audit=self._audit_binding(
                int(token_ids.shape[0]),
                int(token_ids.shape[1]),
                intervention=intervention,
                implementation=implementation,
            ),
            base_logits=base_logits,
            head_logits=head_logits,
        )

    def forward_incremental(
        self,
        token_ids: Tensor,
        targets: Tensor | None = None,
        *,
        initial_state: PredictiveBlockDeltaState | None = None,
        intervention: Intervention = "native",
    ) -> PredictiveBlockDeltaOutput:
        return self.forward(
            token_ids,
            targets,
            initial_state=initial_state,
            implementation="direct",
            intervention=intervention,
        )

    def step(
        self,
        token_ids: Tensor,
        state: PredictiveBlockDeltaState,
        *,
        intervention: Intervention = "native",
    ) -> PredictiveBlockDeltaStepOutput:
        if token_ids.ndim != 1:
            raise ValueError("incremental token_ids must have shape [batch]")
        output = self.forward_incremental(
            token_ids[:, None], initial_state=state, intervention=intervention
        )
        return PredictiveBlockDeltaStepOutput(
            logits=output.logits[:, 0],
            final_state=output.final_state,
            audit=output.audit,
        )


__all__ = [
    "BANKS",
    "BINDING_STATE_BYTES_F32",
    "BINDING_STATE_VALUES",
    "MATRIX_STATE_VALUES",
    "POLICY",
    "PREVIOUS_KEY_VALUES",
    "R4_BLOCKS",
    "R4PredictiveBlockDeltaBindingV1",
    "TRAINABLE_PARAMETER_COUNT",
    "PredictiveBlockDeltaAudit",
    "PredictiveBlockDeltaOutput",
    "PredictiveBlockDeltaState",
    "PredictiveBlockDeltaStepOutput",
]
