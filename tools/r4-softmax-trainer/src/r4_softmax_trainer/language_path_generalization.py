"""Compact matched decoders for the #973 language-path generalization rung.

``R4RetainedLanguagePathV1`` keeps the already qualified two-block,
group-addressed retained-attention law at a data-supported width.
``OrdinaryCausalSoftmaxLanguagePathV1`` is its equal-parameter,
equal-persistent-state ordinary causal-softmax positive control.  This module
contains only the frozen model contract; population and campaign policy live in
their own create-once modules.
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
from .group_retention_decoder import (
    DecoderConfig,
    DecoderOutput,
    DecoderState,
    DecoderStepOutput,
    R4GroupAddressedRetentionDecoderV1,
    expected_occupancy_bit_count,
    expected_parameter_count,
    expected_state_value_count,
)
from .model import RMSNorm, RotaryEmbedding, SwiGLU


POLICY = "R4RetainedLanguagePathV1"
CONTEXTUAL_VALUE_WRITE_POLICY = "R4ContextualValueWriteLanguagePathV1"
CONTEXTUAL_KEY_VALUE_WRITE_POLICY = "R4ContextualKeyValueWriteLanguagePathV1"
CONTEXTUAL_KEY_VALUE_ADDRESS_READ_POLICY = (
    "R4ContextualKeyValueAddressReadLanguagePathV1"
)
ORDINARY_POLICY = "OrdinaryCausalSoftmaxLanguagePathV1"

VOCAB_SIZE = 4_096
HIDDEN_SIZE = 48
INTERMEDIATE_SIZE = 128
LAYERS = 2
HEADS = 4
HEAD_DIM = 12
GROUP_SIZE = 120
CONTEXT = 120
RMS_NORM_EPS = 1e-5
ROPE_THETA = 10_000.0
INITIALIZATION_SEED = 9_738
INITIALIZATION_STD = 0.02
DECAY_HALF_LIVES = (4.0, 16.0, 64.0, 256.0)

PARAMETER_COUNT = 252_160
ADDRESS_SCORE_BIAS_PARAMETER_COUNT = LAYERS * HEADS * GROUP_SIZE
ADDRESS_READ_PARAMETER_COUNT = PARAMETER_COUNT + ADDRESS_SCORE_BIAS_PARAMETER_COUNT
STATE_VALUES = 23_040
STATE_BYTES_F32 = 92_160
VALIDITY_BITS = 240


def language_path_config() -> DecoderConfig:
    """Return the one frozen retained-language-path shape."""

    config = DecoderConfig(
        vocab_size=VOCAB_SIZE,
        hidden_size=HIDDEN_SIZE,
        intermediate_size=INTERMEDIATE_SIZE,
        layers=LAYERS,
        heads=HEADS,
        head_dim=HEAD_DIM,
        group_size=GROUP_SIZE,
        max_sequence_length=CONTEXT,
        rms_norm_eps=RMS_NORM_EPS,
        initialization_seed=INITIALIZATION_SEED,
        initialization_std=INITIALIZATION_STD,
        decay_half_lives=DECAY_HALF_LIVES,
    )
    config.validate()
    if (
        expected_parameter_count(config) != PARAMETER_COUNT
        or expected_state_value_count(config) != STATE_VALUES
        or expected_occupancy_bit_count(config) != VALIDITY_BITS
    ):
        raise RuntimeError("frozen language-path ledger differs from its constants")
    return config


@dataclass(frozen=True, slots=True)
class ArchitectureLedger:
    """Trainable and persistent-state budget for one arm."""

    parameters: int
    state_values: int
    state_bytes_f32: int
    validity_bits: int


@dataclass(frozen=True, slots=True)
class WorkLedger:
    """Analytic attention/output work for one full-prefix call."""

    arm: Literal["retained", "ordinary"]
    batch_size: int
    time: int
    token_steps: int
    materialized_attention_scores: int
    admitted_attention_scores: int
    attention_value_reads: int
    vocabulary_scores: int

    def work_signature(self) -> tuple[int, ...]:
        return (
            self.batch_size,
            self.time,
            self.token_steps,
            self.materialized_attention_scores,
            self.admitted_attention_scores,
            self.attention_value_reads,
            self.vocabulary_scores,
        )


def architecture_ledger(
    arm: Literal["retained", "ordinary"],
) -> ArchitectureLedger:
    """Return the exact matched architecture budget for either arm."""

    if arm not in ("retained", "ordinary"):
        raise ValueError("arm must be 'retained' or 'ordinary'")
    return ArchitectureLedger(
        parameters=PARAMETER_COUNT,
        state_values=STATE_VALUES,
        state_bytes_f32=STATE_BYTES_F32,
        validity_bits=VALIDITY_BITS,
    )


def work_ledger(
    arm: Literal["retained", "ordinary"], *, batch_size: int, time: int
) -> WorkLedger:
    """Return actual materialized and causally admitted score counts.

    The vectorized ordinary implementation materializes a square score matrix
    before masking its strict future.  The retained implementation scores the
    fixed 120-slot field and masks unoccupied slots.  At the frozen 120-token
    context both therefore materialize the same score/value tensor budget even
    though the number of causally admitted entries need not be equal.
    """

    if arm not in ("retained", "ordinary"):
        raise ValueError("arm must be 'retained' or 'ordinary'")
    if batch_size < 1 or not 1 <= time <= CONTEXT:
        raise ValueError("work ledger requires a positive batch and frozen context bound")
    token_steps = batch_size * time
    if arm == "retained":
        materialized = token_steps * LAYERS * HEADS * GROUP_SIZE
        admitted = materialized
    else:
        materialized = batch_size * LAYERS * HEADS * time * time
        admitted = batch_size * LAYERS * HEADS * time * (time + 1) // 2
    return WorkLedger(
        arm=arm,
        batch_size=batch_size,
        time=time,
        token_steps=token_steps,
        materialized_attention_scores=materialized,
        admitted_attention_scores=admitted,
        attention_value_reads=materialized * HEAD_DIM,
        vocabulary_scores=token_steps * VOCAB_SIZE,
    )


class R4RetainedLanguagePathV1(R4GroupAddressedRetentionDecoderV1):
    """Frozen compact exact-H4 retained-attention arm."""

    def __init__(self, geometry: GroupAddressArtifact) -> None:
        if geometry.arm != "exact_h4":
            raise ValueError("language-path retained arm requires exact_h4 geometry")
        super().__init__(language_path_config(), geometry)
        if self.parameter_count() != PARAMETER_COUNT:
            raise RuntimeError("retained language-path parameter ledger differs")
        if self.state_value_count() != STATE_VALUES:
            raise RuntimeError("retained language-path state ledger differs")

    def validity_bit_count(self) -> int:
        """Return the logical occupancy-mask budget per sequence."""

        return self.occupancy_bit_count()

    def forward(
        self,
        token_ids: Tensor,
        targets: Tensor | None = None,
        *,
        attention_off: bool = False,
        initial_state: DecoderState | None = None,
        implementation: Literal["stationary", "direct"] = "stationary",
    ) -> DecoderOutput:
        """Use the shared arm interface while preserving state-off semantics."""

        return R4GroupAddressedRetentionDecoderV1.forward(
            self,
            token_ids,
            targets,
            initial_state=initial_state,
            state_off=attention_off,
            implementation=implementation,
        )

    def forward_incremental(
        self,
        token_ids: Tensor,
        targets: Tensor | None = None,
        *,
        attention_off: bool = False,
        initial_state: DecoderState | None = None,
    ) -> DecoderOutput:
        """Execute the retained direct recurrence under the shared arm API."""

        return R4GroupAddressedRetentionDecoderV1.forward(
            self,
            token_ids,
            targets,
            initial_state=initial_state,
            state_off=attention_off,
            implementation="direct",
        )

    def step(
        self,
        token_ids: Tensor,
        state: DecoderState,
        *,
        attention_off: bool = False,
    ) -> DecoderStepOutput:
        """Advance one direct retained step under the shared arm API."""

        if token_ids.ndim != 1:
            raise ValueError("incremental token_ids must have shape [batch]")
        output = R4GroupAddressedRetentionDecoderV1.forward(
            self,
            token_ids[:, None],
            initial_state=state,
            state_off=attention_off,
            implementation="direct",
        )
        return DecoderStepOutput(
            logits=output.logits[:, 0, :],
            final_state=output.final_state,
            audit=output.audit,
        )


class R4ContextualValueWriteLanguagePathV1(R4RetainedLanguagePathV1):
    """Versioned retained path that writes its ungated causal attention context.

    The learned parameters, recurrent-state tensors, geometry, and output head
    are byte-compatible with ``R4RetainedLanguagePathV1``.  Only the value
    written at the transported identity address changes.  Full-sequence,
    incremental, and one-token calls all traverse the same causal direct cell;
    the stationary closed form is inapplicable once a write depends on its
    strict-prior retained read.
    """

    policy = CONTEXTUAL_VALUE_WRITE_POLICY

    def __init__(self, geometry: GroupAddressArtifact) -> None:
        super().__init__(geometry)
        if (
            self.parameter_count() != PARAMETER_COUNT
            or self.state_value_count() != STATE_VALUES
            or self.validity_bit_count() != VALIDITY_BITS
        ):
            raise RuntimeError("contextual value-write path changed the qualified ledger")

    def _contextual_direct_hidden(
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
                values, layer_keys, layer_values, layer_occupied = (
                    layer.forward_direct_step_with_contextual_value_write(
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

    def forward(
        self,
        token_ids: Tensor,
        targets: Tensor | None = None,
        *,
        attention_off: bool = False,
        initial_state: DecoderState | None = None,
        implementation: Literal["direct"] = "direct",
    ) -> DecoderOutput:
        """Run a full token sequence through the contextual direct recurrence."""

        if token_ids.ndim != 2:
            raise ValueError("token_ids must have shape [batch,time]")
        if implementation != "direct":
            raise ValueError("contextual value-write path requires 'direct' implementation")
        state = (
            self.initial_state(int(token_ids.shape[0]))
            if initial_state is None
            else initial_state
        )
        self._validate_inputs(token_ids, targets, state)
        hidden, final_state = self._contextual_direct_hidden(
            token_ids, state, state_off=attention_off
        )
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
                state_off=attention_off,
                implementation="direct",
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
        """Run one or more tokens through the same contextual direct recurrence."""

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
        """Advance one token through the contextual direct recurrence."""

        if token_ids.ndim != 1:
            raise ValueError("incremental token_ids must have shape [batch]")
        output = self.forward(
            token_ids[:, None],
            initial_state=state,
            attention_off=attention_off,
            implementation="direct",
        )
        return DecoderStepOutput(
            logits=output.logits[:, 0, :],
            final_state=output.final_state,
            audit=output.audit,
        )


class R4ContextualKeyValueWriteLanguagePathV1(
    R4ContextualValueWriteLanguagePathV1
):
    """Versioned retained path with one shared contextual key/value source.

    Query construction and the strict-prior retained read remain token-local.
    Only the later identity-slot write changes: both its key and value derive
    from the same current residual plus ungated retained context.  All learned
    tensors, recurrent state, geometry, and output-head semantics remain byte
    compatible with the earlier retained artifacts.
    """

    policy = CONTEXTUAL_KEY_VALUE_WRITE_POLICY

    def _contextual_direct_hidden(
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
                values, layer_keys, layer_values, layer_occupied = (
                    layer.forward_direct_step_with_contextual_key_value_write(
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


class R4ContextualKeyValueAddressReadLanguagePathV1(
    R4ContextualKeyValueWriteLanguagePathV1
):
    """Contextual K/V retention whose softmax consumes relative H4 address.

    The historical retained readers score only query/key content.  Because
    exact-H4 transport applies the same slot permutation to keys, values, and
    occupancy, that read is invariant to the transport itself.  This version
    adds one learned scalar for every layer, head, and destination address to
    the attention logit.  A zero bias is exactly the prior read law.
    """

    policy = CONTEXTUAL_KEY_VALUE_ADDRESS_READ_POLICY

    def __init__(self, geometry: GroupAddressArtifact) -> None:
        super().__init__(geometry)
        self.address_score_bias = nn.Parameter(
            torch.zeros(LAYERS, HEADS, GROUP_SIZE, dtype=torch.float32)
        )
        if (
            self.parameter_count() != ADDRESS_READ_PARAMETER_COUNT
            or self.state_value_count() != STATE_VALUES
            or self.validity_bit_count() != VALIDITY_BITS
        ):
            raise RuntimeError("address-aware retained path changed its frozen ledger")

    def load_learned_artifact(self, payload: bytes) -> None:
        """Load an exact address-aware artifact for inference."""

        loaded = load_safetensors(payload)
        expected = dict(self.named_parameters())
        if set(loaded) != set(expected):
            raise ValueError("learned artifact parameter names differ from address reader")
        with torch.no_grad():
            for name in sorted(expected):
                source = loaded[name]
                target = expected[name]
                if source.dtype != target.dtype or tuple(source.shape) != tuple(target.shape):
                    raise ValueError(f"learned artifact tensor contract differs for {name}")
                target.copy_(source.to(device=target.device))

    def load_retained_v1_warm_start(self, payload: bytes) -> None:
        """Load only the historical parameter set and initialize the new bias."""

        loaded = load_safetensors(payload)
        expected = dict(self.named_parameters())
        base_names = set(expected) - {"address_score_bias"}
        if set(loaded) != base_names:
            raise ValueError("warm-start artifact parameter names differ from retained V1")
        with torch.no_grad():
            for name in sorted(base_names):
                source = loaded[name]
                target = expected[name]
                if source.dtype != target.dtype or tuple(source.shape) != tuple(target.shape):
                    raise ValueError(f"warm-start tensor contract differs for {name}")
                target.copy_(source.to(device=target.device))
            self.address_score_bias.zero_()

    def _contextual_direct_hidden(
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
                values, layer_keys, layer_values, layer_occupied = (
                    layer.forward_direct_step_with_contextual_key_value_address_read(
                        values,
                        actions,
                        current.keys[layer_index],
                        current.values[layer_index],
                        current.occupied[layer_index],
                        self.identity_offset,
                        self.address_score_bias[layer_index],
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


@dataclass(frozen=True, slots=True)
class OrdinaryDecoderAudit:
    """Causal-softmax work ledger for one ordinary forward call."""

    batch_size: int
    token_steps: int
    layers: int
    heads: int
    materialized_attention_scores: int
    admitted_attention_scores: int
    attention_value_reads: int
    vocabulary_scores: int
    attention_off: bool
    forbidden_reads: int = 0

    def work_signature(self) -> tuple[int, ...]:
        return (
            self.batch_size,
            self.token_steps,
            self.layers,
            self.heads,
            self.materialized_attention_scores,
            self.admitted_attention_scores,
            self.attention_value_reads,
            self.vocabulary_scores,
            self.forbidden_reads,
        )


@dataclass(slots=True)
class OrdinaryDecoderOutput:
    logits: Tensor
    loss: Tensor | None
    audit: OrdinaryDecoderAudit


class _OrdinaryCausalSoftmaxBlock(nn.Module):
    """One ordinary pre-norm RoPE causal-softmax decoder block."""

    def __init__(self, config: DecoderConfig) -> None:
        super().__init__()
        self.config = config
        self.log_score_gains = nn.Parameter(torch.zeros(config.heads))
        self.log_output_gains = nn.Parameter(torch.zeros(config.heads))
        self.input_layernorm = RMSNorm(config.hidden_size, config.rms_norm_eps)
        self.q_proj = nn.Linear(config.hidden_size, config.hidden_size, bias=False)
        self.k_proj = nn.Linear(config.hidden_size, config.hidden_size, bias=False)
        self.v_proj = nn.Linear(config.hidden_size, config.hidden_size, bias=False)
        self.o_proj = nn.Linear(config.hidden_size, config.hidden_size, bias=False)
        self.rope = RotaryEmbedding(config.head_dim, ROPE_THETA, config.max_sequence_length)
        self.post_attention_layernorm = RMSNorm(
            config.hidden_size, config.rms_norm_eps
        )
        self.mlp = SwiGLU(config)  # type: ignore[arg-type]

        causal_mask = torch.triu(
            torch.ones(
                config.max_sequence_length,
                config.max_sequence_length,
                dtype=torch.bool,
            ),
            diagonal=1,
        )
        self.register_buffer("causal_mask", causal_mask, persistent=False)

    def _heads(self, values: Tensor) -> Tensor:
        batch, time, _ = values.shape
        return values.view(
            batch, time, self.config.heads, self.config.head_dim
        ).transpose(1, 2)

    def forward(self, values: Tensor, *, attention_off: bool) -> Tensor:
        time = int(values.shape[1])
        normalized = self.input_layernorm(values)
        query = self.rope(self._heads(self.q_proj(normalized)))
        key = self.rope(self._heads(self.k_proj(normalized)))
        value = self._heads(self.v_proj(normalized))
        scores = torch.matmul(query.float(), key.float().transpose(-2, -1))
        scores = scores / math.sqrt(self.config.head_dim)
        scores = scores * self.log_score_gains.exp().view(1, -1, 1, 1)
        scores = scores.masked_fill(self.causal_mask[:time, :time], float("-inf"))
        weights = torch.softmax(scores, dim=-1, dtype=torch.float32)
        attended = torch.matmul(weights, value.float()).to(values.dtype)
        attended = attended * self.log_output_gains.exp().view(1, -1, 1, 1)
        attended = attended.transpose(1, 2).contiguous().view(values.shape)
        attended = self.o_proj(attended)
        attended = attended * (0.0 if attention_off else 1.0)
        values = values + attended
        return values + self.mlp(self.post_attention_layernorm(values))


class OrdinaryCausalSoftmaxLanguagePathV1(nn.Module):
    """Equal-budget ordinary strictly causal full-prefix positive control."""

    def __init__(self) -> None:
        super().__init__()
        self.config = language_path_config()
        self.token_embedding = nn.Embedding(VOCAB_SIZE, HIDDEN_SIZE)
        self.layers = nn.ModuleList(
            _OrdinaryCausalSoftmaxBlock(self.config) for _ in range(LAYERS)
        )
        self.final_norm = RMSNorm(HIDDEN_SIZE, RMS_NORM_EPS)
        self._initialize_learned_weights()
        if self.parameter_count() != PARAMETER_COUNT:
            raise RuntimeError("ordinary language-path parameter ledger differs")

    def _initialize_learned_weights(self) -> None:
        generator = torch.Generator(device="cpu")
        generator.manual_seed(INITIALIZATION_SEED)
        with torch.no_grad():
            for module in self.modules():
                if isinstance(module, (nn.Embedding, nn.Linear)):
                    module.weight.normal_(
                        mean=0.0,
                        std=INITIALIZATION_STD,
                        generator=generator,
                    )

    @property
    def output_weight(self) -> nn.Parameter:
        return self.token_embedding.weight

    def parameter_count(self) -> int:
        return sum(parameter.numel() for parameter in self.parameters())

    def state_value_count(self) -> int:
        """Return the full-context incremental K/V cache budget."""

        return LAYERS * 2 * HEADS * CONTEXT * HEAD_DIM

    def validity_bit_count(self) -> int:
        """Return the per-layer valid-position mask budget."""

        return LAYERS * CONTEXT

    def export_learned_artifact(self) -> bytes:
        """Return deterministic Safetensors bytes for the ordinary arm."""

        tensors = {
            name: parameter.detach().cpu().contiguous()
            for name, parameter in sorted(self.named_parameters())
        }
        return save_safetensors(tensors)

    def load_learned_artifact(self, payload: bytes) -> None:
        """Load an exact shape- and name-bound ordinary-arm artifact."""

        loaded = load_safetensors(payload)
        expected = dict(self.named_parameters())
        if set(loaded) != set(expected):
            raise ValueError("learned artifact parameter names differ from ordinary decoder")
        with torch.no_grad():
            for name in sorted(expected):
                source = loaded[name]
                target = expected[name]
                if source.dtype != target.dtype or tuple(source.shape) != tuple(target.shape):
                    raise ValueError(
                        f"learned artifact tensor contract differs for {name}"
                    )
                target.copy_(source.to(device=target.device))

    def _validate_inputs(self, token_ids: Tensor, targets: Tensor | None) -> None:
        if token_ids.ndim != 2 or token_ids.dtype != torch.long:
            raise ValueError("token_ids must be int64 [batch,time]")
        batch, time = token_ids.shape
        if batch < 1 or not 1 <= time <= CONTEXT:
            raise ValueError("tokens must contain a nonempty frozen-context batch")
        if bool((token_ids < 0).any()) or bool((token_ids >= VOCAB_SIZE).any()):
            raise ValueError("token_ids contain an out-of-vocabulary value")
        if targets is not None:
            if targets.shape != token_ids.shape or targets.dtype != torch.long:
                raise ValueError("targets must be int64 and match token_ids")
            valid = targets != -100
            if bool(valid.any()):
                selected = targets[valid]
                if bool((selected < 0).any()) or bool((selected >= VOCAB_SIZE).any()):
                    raise ValueError("targets contain an out-of-vocabulary value")

    def forward(
        self,
        token_ids: Tensor,
        targets: Tensor | None = None,
        *,
        attention_off: bool = False,
    ) -> OrdinaryDecoderOutput:
        self._validate_inputs(token_ids, targets)
        values = self.token_embedding(token_ids)
        for layer in self.layers:
            values = layer(values, attention_off=attention_off)
        values = self.final_norm(values)
        logits = F.linear(values, self.output_weight)
        loss = None
        if targets is not None:
            loss = F.cross_entropy(
                logits.float().reshape(-1, VOCAB_SIZE), targets.reshape(-1)
            )
        batch_size, time = token_ids.shape
        ledger = work_ledger("ordinary", batch_size=batch_size, time=time)
        return OrdinaryDecoderOutput(
            logits=logits,
            loss=loss,
            audit=OrdinaryDecoderAudit(
                batch_size=batch_size,
                token_steps=ledger.token_steps,
                layers=LAYERS,
                heads=HEADS,
                materialized_attention_scores=ledger.materialized_attention_scores,
                admitted_attention_scores=ledger.admitted_attention_scores,
                attention_value_reads=ledger.attention_value_reads,
                vocabulary_scores=ledger.vocabulary_scores,
                attention_off=attention_off,
            ),
        )
