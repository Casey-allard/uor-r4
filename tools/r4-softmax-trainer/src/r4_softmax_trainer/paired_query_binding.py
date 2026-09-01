"""Paired-query conditional binding over the frozen #1017 attention model.

This is the C1-SB5 mechanism core.  Two questions share the exact same source
prefix and candidate groups.  Candidate states are read before either question,
while each query state is read at the final ``Bind:`` colon.  A small asymmetric
rank-32 biaffine head therefore scores the complete query-by-candidate matrix.

The 32 binding coordinates may be viewed as eight four-lane bookkeeping blocks.
That layout is not an intrinsic-geometry or geometry-advantage claim.
"""

from __future__ import annotations

import math
import time
from collections import defaultdict
from collections.abc import Iterable, Mapping, Sequence
from dataclasses import asdict, dataclass, field
from typing import Any

import torch
from torch import Tensor, nn
from torch.nn import functional as F

from .constants import BOS_TOKEN_ID, EOS_TOKEN_ID, FROZEN_MODEL_CONFIG
from .model import R4SoftmaxForCausalLM, expected_hf_tensor_names
from .source_relation_adapter import (
    LORA_ALPHA,
    LORA_DROPOUT,
    LORA_RANK,
    TRAINABLE_PARAMETER_COUNT as LORA_TRAINABLE_PARAMETER_COUNT,
    LoRALinear,
)
from .train import require_mps


POLICY = "R4PairedQueryCandidateMatrixV1"
REPRESENTATION_UPDATE = "lora_qkvo_all_layers_plus_rank32_binding"
SOURCE_WIDTHS = tuple(range(2, 9))
TARGET_PROJECTIONS = ("q_proj", "k_proj", "v_proj", "o_proj")
TARGET_LAYER_INDICES = tuple(range(FROZEN_MODEL_CONFIG.num_hidden_layers))
BINDING_RANK = 32
BINDING_BLOCK_WIDTH = 4
BINDING_BLOCKS = BINDING_RANK // BINDING_BLOCK_WIDTH
BINDING_HEAD_PARAMETER_COUNT = (
    2 * FROZEN_MODEL_CONFIG.hidden_size * BINDING_RANK + 1
)
TRAINABLE_PARAMETER_COUNT = (
    LORA_TRAINABLE_PARAMETER_COUNT + BINDING_HEAD_PARAMETER_COUNT
)
MARGIN = 1.0
FLIP_MARGIN = 2.0
FIT_SEED = 9_545
PAIRED_RECORDS_PER_WIDTH = 8
RECORDS_PER_STEP = len(SOURCE_WIDTHS)
STEPS_PER_EPOCH = PAIRED_RECORDS_PER_WIDTH
OPTIMIZER_STEPS = 120


@dataclass(frozen=True, slots=True)
class PairedQueryBindingAdapterConfig:
    """The one independently frozen C1-SB5 model contract."""

    rank: int = LORA_RANK
    alpha: int = LORA_ALPHA
    dropout: float = LORA_DROPOUT
    target_layer_indices: tuple[int, ...] = TARGET_LAYER_INDICES
    target_projections: tuple[str, ...] = TARGET_PROJECTIONS
    binding_rank: int = BINDING_RANK
    initialization_seed: int = FIT_SEED

    def validate(self) -> None:
        if self != PairedQueryBindingAdapterConfig():
            raise ValueError("paired-query binding exposes one frozen model contract")
        if self.rank != 8 or self.alpha != 8 or self.dropout != 0.0:
            raise AssertionError("paired-query LoRA shape drifted")
        if self.target_layer_indices != tuple(range(6)):
            raise AssertionError("paired-query binding must adapt all six layers")
        if self.target_projections != TARGET_PROJECTIONS:
            raise AssertionError("paired-query binding must adapt exactly Q/K/V/O")
        if self.binding_rank != 32 or self.binding_rank % 4:
            raise AssertionError("paired-query binding head must contain eight R4 blocks")
        if self.initialization_seed != 9_545:
            raise AssertionError("paired-query initialization seed drifted")

    @property
    def trainable_parameter_count(self) -> int:
        self.validate()
        return TRAINABLE_PARAMETER_COUNT

    def as_contract(self) -> dict[str, Any]:
        self.validate()
        value = asdict(self)
        value["target_layer_indices"] = list(self.target_layer_indices)
        value["target_projections"] = list(self.target_projections)
        value.update(
            {
                "representation_update": REPRESENTATION_UPDATE,
                "lora_trainable_parameters": LORA_TRAINABLE_PARAMETER_COUNT,
                "binding_head_trainable_parameters": BINDING_HEAD_PARAMETER_COUNT,
                "trainable_parameter_count": TRAINABLE_PARAMETER_COUNT,
                "binding_blocks": BINDING_BLOCKS,
                "score": "dot(Wq*hq,Wc*hc)/sqrt(32)+b",
                "decision": "supported iff score > +0.0",
                "generic_lm_or_classification_head": False,
                "geometry_claim": "NONE; eight four-lane blocks are bookkeeping",
            }
        )
        return value


@dataclass(frozen=True, slots=True)
class PairedQueryBindingFitConfig:
    """The sole bounded optimizer schedule; this is intentionally not a sweep."""

    seed: int = FIT_SEED
    optimizer_steps: int = OPTIMIZER_STEPS
    records_per_step: int = RECORDS_PER_STEP
    learning_rate: float = 0.001
    adam_beta1: float = 0.9
    adam_beta2: float = 0.999
    adam_epsilon: float = 1e-8
    weight_decay: float = 0.0
    gradient_clip: float = 1.0
    eta_probe_step: int = 8
    wall_ceiling_seconds: float = 300.0
    progress_interval: int = STEPS_PER_EPOCH

    def validate(self) -> None:
        if self != PairedQueryBindingFitConfig():
            raise ValueError("paired-query binding exposes one frozen fit contract")
        if (
            self.seed != 9_545
            or self.optimizer_steps != 120
            or self.records_per_step != 7
            or self.weight_decay != 0.0
        ):
            raise AssertionError("paired-query fit contract drifted")

    def as_contract(self) -> dict[str, Any]:
        self.validate()
        value = asdict(self)
        value.update(
            {
                "steps_per_epoch": STEPS_PER_EPOCH,
                "epochs": self.optimizer_steps // STEPS_PER_EPOCH,
                "row_margin": MARGIN,
                "flip_margin": FLIP_MARGIN,
                "schedule": (
                    "eight steps per epoch; every step contains one complete "
                    "paired record for each source width 2..8"
                ),
            }
        )
        return value


class PairedQueryBindingWallBudgetExceeded(RuntimeError):
    def __init__(
        self,
        *,
        step: int,
        elapsed_seconds: float,
        projected_seconds_at_eta_probe: float | None,
        wall_ceiling_seconds: float,
    ) -> None:
        super().__init__("UNAVAILABLE_PAIRED_QUERY_BINDING_WALL_BUDGET")
        self.step = step
        self.elapsed_seconds = elapsed_seconds
        self.projected_seconds_at_eta_probe = projected_seconds_at_eta_probe
        self.wall_ceiling_seconds = wall_ceiling_seconds

    def as_result(self) -> dict[str, Any]:
        return {
            "status": "UNAVAILABLE_PAIRED_QUERY_BINDING_WALL_BUDGET",
            "stopped_after_step": self.step,
            "elapsed_seconds": self.elapsed_seconds,
            "projected_seconds_at_eta_probe": self.projected_seconds_at_eta_probe,
            "wall_ceiling_seconds": self.wall_ceiling_seconds,
        }


@dataclass(frozen=True, slots=True)
class EncodedPairedQueryLane:
    question: str
    token_ids: tuple[int, ...]
    query_terminal_index: int
    candidate_terminal_indices: tuple[int, ...]
    source_prefix_token_count: int
    target_outcome: str
    metadata: Mapping[str, Any] = field(default_factory=dict, repr=False, compare=False)


@dataclass(frozen=True, slots=True)
class EncodedPairedCandidateGroup:
    relation_group_cid: str
    occurrence_indices: tuple[int, ...]
    earliest_occurrence_index: int
    text: str = ""


@dataclass(frozen=True, slots=True)
class EncodedPairedQueryRecord:
    record_id: str
    source_width: int
    pair_slot: int
    lexical_world: str
    queries: tuple[EncodedPairedQueryLane, EncodedPairedQueryLane]
    candidate_groups: tuple[EncodedPairedCandidateGroup, ...]
    label_matrix: tuple[tuple[int, ...], tuple[int, ...]]
    flip_group_indices: tuple[int, ...]
    source_prefix_token_ids_cid: str
    metadata: Mapping[str, Any] = field(default_factory=dict, repr=False, compare=False)


@dataclass(slots=True)
class PairedQueryTokenBatch:
    """A padded batch of complete paired records and variable candidate groups."""

    input_ids: Tensor
    query_indices: Tensor
    candidate_indices: Tensor
    group_mask: Tensor
    labels: Tensor
    flip_mask: Tensor
    source_prefix_lengths: Tensor
    record_indices: tuple[int, ...]
    group_identities: tuple[tuple[str, ...], ...]
    validated: bool = False

    def validate(self) -> None:
        if self.input_ids.ndim != 3 or self.input_ids.shape[1] != 2:
            raise ValueError("paired input_ids must have shape [pairs, 2, time]")
        pairs, _, time_width = self.input_ids.shape
        if self.input_ids.dtype != torch.long:
            raise ValueError("paired input_ids must be int64")
        if self.query_indices.shape != (pairs, 2):
            raise ValueError("paired query indices must have shape [pairs, 2]")
        if self.query_indices.dtype != torch.long:
            raise ValueError("paired query indices must be int64")
        if self.candidate_indices.ndim != 2 or self.candidate_indices.shape[0] != pairs:
            raise ValueError("candidate indices must have shape [pairs, groups]")
        groups = self.candidate_indices.shape[1]
        if self.candidate_indices.dtype != torch.long:
            raise ValueError("candidate indices must be int64")
        if self.group_mask.shape != (pairs, groups) or self.group_mask.dtype != torch.bool:
            raise ValueError("group mask must be bool [pairs, groups]")
        if self.labels.shape != (pairs, 2, groups):
            raise ValueError("label matrix must have shape [pairs, 2, groups]")
        if self.labels.dtype not in (torch.float16, torch.float32, torch.float64):
            raise ValueError("paired labels must be floating point")
        if self.flip_mask.shape != (pairs, groups) or self.flip_mask.dtype != torch.bool:
            raise ValueError("flip mask must be bool [pairs, groups]")
        if self.source_prefix_lengths.shape != (pairs,):
            raise ValueError("source-prefix lengths must have shape [pairs]")
        if self.source_prefix_lengths.dtype != torch.long:
            raise ValueError("source-prefix lengths must be int64")
        if len(self.record_indices) != pairs or len(self.group_identities) != pairs:
            raise ValueError("paired batch identities do not align with pair rows")
        if groups < 1 or bool((~self.group_mask.any(dim=1)).any()):
            raise ValueError("every paired record must contain a candidate group")
        if bool((self.flip_mask & ~self.group_mask).any()):
            raise ValueError("required flips must refer to admitted candidate groups")
        if bool((~self.flip_mask.any(dim=1)).any()):
            raise ValueError("every paired record must contain a required label flip")
        active_labels = self.labels.masked_select(self.group_mask[:, None, :])
        if bool(((active_labels != 0.0) & (active_labels != 1.0)).any()):
            raise ValueError("paired labels must be binary on admitted groups")
        inactive_labels = self.labels.masked_select(~self.group_mask[:, None, :])
        if inactive_labels.numel() and bool((inactive_labels != 0.0).any()):
            raise ValueError("padded paired labels must be zero")
        flip_labels = self.labels.permute(0, 2, 1)[self.flip_mask]
        if bool((flip_labels.sum(dim=1) != 1.0).any()):
            raise ValueError("each required flip must contain one positive query row")
        if bool(((self.source_prefix_lengths < 1) | (self.source_prefix_lengths > time_width)).any()):
            raise ValueError("source-prefix length is outside the token sequence")
        if bool(((self.query_indices < 0) | (self.query_indices >= time_width)).any()):
            raise ValueError("query terminal index is outside the token sequence")
        if bool((self.query_indices < self.source_prefix_lengths[:, None]).any()):
            raise ValueError("query state must occur after the complete source prefix")
        active_candidates = self.candidate_indices.masked_select(self.group_mask)
        active_prefixes = self.source_prefix_lengths[:, None].expand_as(
            self.candidate_indices
        ).masked_select(self.group_mask)
        if bool(((active_candidates < 0) | (active_candidates >= active_prefixes)).any()):
            raise ValueError("candidate state must occur before the question")
        for pair_index, prefix_length in enumerate(self.source_prefix_lengths.tolist()):
            if not torch.equal(
                self.input_ids[pair_index, 0, :prefix_length],
                self.input_ids[pair_index, 1, :prefix_length],
            ):
                raise ValueError("paired source token prefixes are not bit-identical")
            if len(self.group_identities[pair_index]) != int(
                self.group_mask[pair_index].sum().item()
            ):
                raise ValueError("candidate group identities do not match the mask")
        self.validated = True

    def to(self, device: torch.device) -> "PairedQueryTokenBatch":
        if not self.validated:
            self.validate()
        return PairedQueryTokenBatch(
            input_ids=self.input_ids.to(device),
            query_indices=self.query_indices.to(device),
            candidate_indices=self.candidate_indices.to(device),
            group_mask=self.group_mask.to(device),
            labels=self.labels.to(device),
            flip_mask=self.flip_mask.to(device),
            source_prefix_lengths=self.source_prefix_lengths.to(device),
            record_indices=self.record_indices,
            group_identities=self.group_identities,
            validated=True,
        )

    def swapped_query_rows(self) -> "PairedQueryTokenBatch":
        swapped = PairedQueryTokenBatch(
            input_ids=self.input_ids.flip(1),
            query_indices=self.query_indices.flip(1),
            candidate_indices=self.candidate_indices,
            group_mask=self.group_mask,
            labels=self.labels.flip(1),
            flip_mask=self.flip_mask,
            source_prefix_lengths=self.source_prefix_lengths,
            record_indices=self.record_indices,
            group_identities=self.group_identities,
            validated=True,
        )
        return swapped


@dataclass(frozen=True, slots=True)
class PairedQueryBindingOutput:
    scores: Tensor
    query_states: Tensor
    candidate_states: Tensor
    candidate_states_by_lane: Tensor
    paired_candidate_states_exact: bool | None
    attention_off: bool
    mean_query_ablation: bool

    @property
    def supported(self) -> Tensor:
        return self.scores > 0.0


class AsymmetricR4BindingHead(nn.Module):
    """Bias-free asymmetric rank-32 projections plus one scalar threshold bias."""

    def __init__(self, *, initialization_seed: int = FIT_SEED) -> None:
        super().__init__()
        self.query_weight = nn.Parameter(
            torch.empty(BINDING_RANK, FROZEN_MODEL_CONFIG.hidden_size)
        )
        self.candidate_weight = nn.Parameter(
            torch.empty(BINDING_RANK, FROZEN_MODEL_CONFIG.hidden_size)
        )
        self.bias = nn.Parameter(torch.zeros(()))
        query_generator = torch.Generator(device="cpu")
        candidate_generator = torch.Generator(device="cpu")
        query_generator.manual_seed(initialization_seed + 24)
        candidate_generator.manual_seed(initialization_seed + 25)
        nn.init.xavier_uniform_(self.query_weight, generator=query_generator)
        nn.init.xavier_uniform_(self.candidate_weight, generator=candidate_generator)
        self.register_buffer(
            "_initial_query_weight", self.query_weight.detach().clone(), persistent=False
        )
        self.register_buffer(
            "_initial_candidate_weight",
            self.candidate_weight.detach().clone(),
            persistent=False,
        )
        self.register_buffer("_initial_bias", self.bias.detach().clone(), persistent=False)

    def forward(self, query_states: Tensor, candidate_states: Tensor) -> Tensor:
        if query_states.ndim != 3 or query_states.shape[-1] != FROZEN_MODEL_CONFIG.hidden_size:
            raise ValueError("query states must have shape [pairs, 2, 288]")
        if query_states.shape[1] != 2:
            raise ValueError("binding head requires exactly two query rows")
        if candidate_states.ndim != 3 or candidate_states.shape[-1] != FROZEN_MODEL_CONFIG.hidden_size:
            raise ValueError("candidate states must have shape [pairs, groups, 288]")
        if candidate_states.shape[0] != query_states.shape[0]:
            raise ValueError("query and candidate pair counts differ")
        query = F.linear(query_states.float(), self.query_weight.float())
        candidate = F.linear(candidate_states.float(), self.candidate_weight.float())
        return (
            torch.matmul(query, candidate.transpose(-2, -1)) / math.sqrt(BINDING_RANK)
            + self.bias.float()
        )

    def state_for_artifact(self) -> dict[str, Tensor]:
        return {
            "query_weight": self.query_weight.detach().float().cpu().contiguous(),
            "candidate_weight": self.candidate_weight.detach().float().cpu().contiguous(),
            "bias": self.bias.detach().float().cpu().contiguous(),
        }

    def audit(self) -> dict[str, Any]:
        rows = (
            ("query_weight", self.query_weight, self._initial_query_weight),
            ("candidate_weight", self.candidate_weight, self._initial_candidate_weight),
            ("bias", self.bias, self._initial_bias),
        )
        tensors = [
            {
                "name": name,
                "parameter_count": parameter.numel(),
                "finite": bool(torch.isfinite(parameter.detach()).all()),
                "changed_from_initialization": not torch.equal(
                    parameter.detach(), initial.detach()
                ),
                "l2_norm": float(torch.linalg.vector_norm(parameter.detach().float()).cpu()),
            }
            for name, parameter, initial in rows
        ]
        return {
            "tensor_count": len(tensors),
            "parameter_count": sum(int(row["parameter_count"]) for row in tensors),
            "changed_tensor_count": sum(
                bool(row["changed_from_initialization"]) for row in tensors
            ),
            "all_finite": all(bool(row["finite"]) for row in tensors),
            "tensors": tensors,
        }


class R4PairedQueryCandidateMatrix(nn.Module):
    """Six-layer Q/K/V/O LoRA representation plus the explicit binding head."""

    def __init__(
        self,
        model: R4SoftmaxForCausalLM,
        config: PairedQueryBindingAdapterConfig = PairedQueryBindingAdapterConfig(),
    ) -> None:
        super().__init__()
        config.validate()
        if model.config != FROZEN_MODEL_CONFIG:
            raise ValueError("paired-query binding requires the exact #1017 architecture")
        model.requires_grad_(False)
        self.model = model
        self.config = config
        self._adapters: dict[str, LoRALinear] = {}
        adapter_ordinal = 0
        for layer_index in config.target_layer_indices:
            attention = self.model.model.layers[layer_index].self_attn
            for projection_name in config.target_projections:
                projection = getattr(attention, projection_name)
                if not isinstance(projection, nn.Linear):
                    raise TypeError(
                        f"layer {layer_index} {projection_name} is not an unwrapped linear"
                    )
                adapter = LoRALinear(
                    projection,
                    rank=config.rank,
                    alpha=config.alpha,
                    dropout=config.dropout,
                    initialization_seed=config.initialization_seed + adapter_ordinal,
                )
                setattr(attention, projection_name, adapter)
                self._adapters[
                    self._projection_weight_path(layer_index, projection_name)
                ] = adapter
                adapter_ordinal += 1
        self.binding_head = AsymmetricR4BindingHead(
            initialization_seed=config.initialization_seed
        )
        observed = sum(
            parameter.numel()
            for parameter in self.parameters()
            if parameter.requires_grad
        )
        if observed != config.trainable_parameter_count:
            raise AssertionError(
                f"paired-query binding has {observed} trainable parameters, "
                f"expected {config.trainable_parameter_count}"
            )

    @staticmethod
    def _projection_weight_path(layer_index: int, projection_name: str) -> str:
        return f"model.layers.{layer_index}.self_attn.{projection_name}.weight"

    def adapter_parameters(self) -> Iterable[nn.Parameter]:
        return (parameter for parameter in self.parameters() if parameter.requires_grad)

    def trainable_parameter_names(self) -> tuple[str, ...]:
        return tuple(
            name for name, parameter in self.named_parameters() if parameter.requires_grad
        )

    def binding_head_state_dict(self) -> dict[str, Tensor]:
        return self.binding_head.state_for_artifact()

    def forward(
        self,
        batch: PairedQueryTokenBatch,
        *,
        attention_off: bool = False,
        mean_query_ablation: bool = False,
        verify_candidate_state_identity: bool = True,
    ) -> PairedQueryBindingOutput:
        if not batch.validated:
            batch.validate()
        pairs, lanes, time_width = batch.input_ids.shape
        hidden = self.model.model(
            batch.input_ids.reshape(pairs * lanes, time_width),
            attention_off=attention_off,
        ).view(pairs, lanes, time_width, FROZEN_MODEL_CONFIG.hidden_size)
        query_gather = batch.query_indices[:, :, None, None].expand(
            pairs, lanes, 1, FROZEN_MODEL_CONFIG.hidden_size
        )
        query_states = torch.gather(hidden, 2, query_gather).squeeze(2)
        groups = batch.candidate_indices.shape[1]
        candidate_gather = batch.candidate_indices[:, None, :, None].expand(
            pairs, lanes, groups, FROZEN_MODEL_CONFIG.hidden_size
        )
        candidate_states_by_lane = torch.gather(hidden, 2, candidate_gather)
        candidate_states_exact: bool | None = None
        if verify_candidate_state_identity:
            active_state_mask = batch.group_mask[:, None, :, None].expand_as(
                candidate_states_by_lane
            )
            left = candidate_states_by_lane[:, 0:1].expand_as(candidate_states_by_lane)
            candidate_states_exact = torch.equal(
                candidate_states_by_lane.masked_select(active_state_mask),
                left.masked_select(active_state_mask),
            )
            if not candidate_states_exact:
                raise RuntimeError(
                    "paired candidate states differ despite an identical causal source prefix"
                )
        candidate_states = candidate_states_by_lane[:, 0]
        if mean_query_ablation:
            query_states = query_states.mean(dim=1, keepdim=True).expand_as(query_states)
        scores = self.binding_head(query_states, candidate_states)
        scores = torch.where(batch.group_mask[:, None, :], scores, torch.zeros_like(scores))
        if mean_query_ablation and not torch.equal(scores[:, 0], scores[:, 1]):
            raise RuntimeError("pair-mean query ablation did not produce identical rows")
        return PairedQueryBindingOutput(
            scores=scores,
            query_states=query_states,
            candidate_states=candidate_states,
            candidate_states_by_lane=candidate_states_by_lane,
            paired_candidate_states_exact=candidate_states_exact,
            attention_off=attention_off,
            mean_query_ablation=mean_query_ablation,
        )

    def merged_state_dict(self) -> dict[str, Tensor]:
        wrapped_state = self.model.state_dict()
        expected = expected_hf_tensor_names(self.model.config)
        merged: dict[str, Tensor] = {}
        for name in sorted(expected):
            adapter = self._adapters.get(name)
            if adapter is not None:
                merged[name] = adapter.merged_weight()
                continue
            try:
                value = wrapped_state[name]
            except KeyError as error:
                raise RuntimeError(f"paired-query wrapped model is missing {name}") from error
            merged[name] = value.detach().float().cpu().contiguous()
        if set(merged) != expected or len(self._adapters) != 24:
            raise RuntimeError("paired-query merge did not cover 24 Q/K/V/O tensors")
        return merged

    def delta_audit(self) -> dict[str, Any]:
        tensors: list[dict[str, Any]] = []
        for name in sorted(self._adapters):
            adapter = self._adapters[name]
            delta = (
                torch.matmul(
                    adapter.lora_b.detach().float(), adapter.lora_a.detach().float()
                )
                * adapter.scaling
            )
            tensors.append(
                {
                    "name": name,
                    "parameter_count": adapter.lora_a.numel()
                    + adapter.lora_b.numel(),
                    "nonzero": int(torch.count_nonzero(delta).item()),
                    "changed": bool(torch.count_nonzero(delta).item()),
                    "finite": bool(torch.isfinite(delta).all()),
                    "l2_norm": float(torch.linalg.vector_norm(delta).cpu()),
                    "initialization_seed": adapter.initialization_seed,
                }
            )
        if len(tensors) != 24:
            raise RuntimeError("paired-query delta census does not contain 24 tensors")
        return {
            "target_tensor_count": len(tensors),
            "trainable_parameter_count": sum(
                int(tensor["parameter_count"]) for tensor in tensors
            ),
            "changed_tensor_count": sum(bool(tensor["changed"]) for tensor in tensors),
            "all_finite": all(bool(tensor["finite"]) for tensor in tensors),
            "tensors": tensors,
        }

    def binding_head_audit(self) -> dict[str, Any]:
        return self.binding_head.audit()

    def representation_audit(
        self, base_state: Mapping[str, Tensor]
    ) -> dict[str, Any]:
        """Compare the merged representation to its immutable predecessor."""
        merged = self.merged_state_dict()
        expected = expected_hf_tensor_names(self.model.config)
        if set(base_state) != expected:
            raise ValueError("paired-query base-state tensor names differ from #1017")
        target_names = set(self._adapters)
        changed_targets = sorted(
            name for name in target_names if not torch.equal(merged[name], base_state[name])
        )
        changed_nontargets = sorted(
            name
            for name in expected - target_names
            if not torch.equal(merged[name], base_state[name])
        )
        target_finite = all(
            bool(torch.isfinite(merged[name]).all()) for name in target_names
        )
        head = self.binding_head_audit()
        passed = (
            len(changed_targets) == 24
            and not changed_nontargets
            and target_finite
            and int(head["changed_tensor_count"]) == 3
            and bool(head["all_finite"])
        )
        return {
            "target_tensor_count": len(target_names),
            "changed_target_tensor_count": len(changed_targets),
            "changed_target_tensors": changed_targets,
            "changed_nontarget_tensors": changed_nontargets,
            "all_target_tensors_finite": target_finite,
            "binding_head": head,
            "passed": passed,
        }

    def merged_model(self) -> R4SoftmaxForCausalLM:
        """Return an ordinary #1017 model with all 24 LoRA deltas folded."""
        clean = R4SoftmaxForCausalLM(self.model.config)
        clean.load_state_dict(self.merged_state_dict(), strict=True)
        clean.eval()
        return clean


def _mapping_sequence(value: Any, *, field_name: str) -> Sequence[Any]:
    if not isinstance(value, Sequence) or isinstance(value, (str, bytes)):
        raise ValueError(f"paired-query {field_name} must be a sequence")
    return value


def _encoded_record(record: Mapping[str, Any], offset: int) -> EncodedPairedQueryRecord:
    record_id = record.get("record_id", record.get("record_cid"))
    if not isinstance(record_id, str) or not record_id:
        raise ValueError(f"paired-query record {offset} has no stable identity")
    source_width = record.get("source_width")
    pair_slot = record.get("pair_slot")
    lexical_world = record.get("lexical_world", record.get("world", ""))
    if source_width not in SOURCE_WIDTHS:
        raise ValueError(f"paired-query record {record_id} has width outside 2..8")
    if not isinstance(pair_slot, int) or pair_slot not in range(4):
        raise ValueError(f"paired-query record {record_id} has pair_slot outside 0..3")
    if not isinstance(lexical_world, str) or not lexical_world:
        raise ValueError(f"paired-query record {record_id} has no lexical world")
    if record.get("source_prefix_identity_exact") is not True:
        raise ValueError(f"paired-query record {record_id} lacks exact prefix identity")
    prefix_cid = record.get("source_prefix_token_ids_cid")
    if not isinstance(prefix_cid, str) or not prefix_cid:
        raise ValueError(f"paired-query record {record_id} has no prefix token CID")

    raw_groups = _mapping_sequence(
        record.get("candidate_groups"), field_name="candidate_groups"
    )
    groups: list[EncodedPairedCandidateGroup] = []
    for group_offset, raw_group in enumerate(raw_groups):
        if not isinstance(raw_group, Mapping):
            raise ValueError(f"paired-query record {record_id} has a nonmapping group")
        group_cid = raw_group.get("relation_group_cid")
        occurrences = _mapping_sequence(
            raw_group.get("occurrence_indices"), field_name="occurrence_indices"
        )
        occurrence_indices = tuple(int(value) for value in occurrences)
        if (
            not isinstance(group_cid, str)
            or not group_cid
            or not occurrence_indices
            or tuple(sorted(set(occurrence_indices))) != occurrence_indices
        ):
            raise ValueError(
                f"paired-query record {record_id} group {group_offset} is noncanonical"
            )
        earliest = raw_group.get("earliest_occurrence_index", occurrence_indices[0])
        if earliest != occurrence_indices[0]:
            raise ValueError("paired-query group does not bind its earliest occurrence")
        groups.append(
            EncodedPairedCandidateGroup(
                relation_group_cid=group_cid,
                occurrence_indices=occurrence_indices,
                earliest_occurrence_index=int(earliest),
                text=str(raw_group.get("text", "")),
            )
        )
    if not groups:
        raise ValueError(f"paired-query record {record_id} has no candidate groups")

    raw_labels = _mapping_sequence(record.get("label_matrix"), field_name="label_matrix")
    if len(raw_labels) != 2:
        raise ValueError("paired-query label matrix must contain exactly two rows")
    label_matrix = tuple(
        tuple(int(value) for value in _mapping_sequence(row, field_name="label row"))
        for row in raw_labels
    )
    if any(len(row) != len(groups) for row in label_matrix):
        raise ValueError("paired-query label rows do not align with candidate groups")
    if any(value not in (0, 1) for row in label_matrix for value in row):
        raise ValueError("paired-query label matrix is nonbinary")
    flip_cids = set(
        str(value)
        for value in _mapping_sequence(
            record.get("flip_group_cids"), field_name="flip_group_cids"
        )
    )
    group_cids = [group.relation_group_cid for group in groups]
    if not flip_cids or not flip_cids.issubset(group_cids):
        raise ValueError("paired-query flips do not bind admitted candidate groups")
    flip_indices = tuple(
        index for index, group_cid in enumerate(group_cids) if group_cid in flip_cids
    )
    if any(label_matrix[0][index] + label_matrix[1][index] != 1 for index in flip_indices):
        raise ValueError("paired-query required flip is not a complementary label column")

    raw_queries = _mapping_sequence(record.get("queries"), field_name="queries")
    if len(raw_queries) != 2:
        raise ValueError("paired-query record must contain exactly two query lanes")
    queries: list[EncodedPairedQueryLane] = []
    for query_offset, raw_query in enumerate(raw_queries):
        if not isinstance(raw_query, Mapping):
            raise ValueError("paired-query lane must be a mapping")
        question = raw_query.get("question")
        token_ids = tuple(
            int(value)
            for value in _mapping_sequence(
                raw_query.get("token_ids"), field_name="query token_ids"
            )
        )
        terminal = raw_query.get("query_terminal_index")
        candidate_terminals = tuple(
            int(value)
            for value in _mapping_sequence(
                raw_query.get("candidate_terminal_indices"),
                field_name="candidate terminal indices",
            )
        )
        prefix_count = raw_query.get("source_prefix_token_count")
        if not isinstance(question, str) or not question.endswith("?"):
            raise ValueError("paired-query question is not canonical")
        if not token_ids or any(value < 0 for value in token_ids):
            raise ValueError("paired-query token IDs are empty or negative")
        if terminal != len(token_ids):
            raise ValueError("query terminal must be the final Bind colon including BOS")
        if len(candidate_terminals) != len(groups):
            raise ValueError("candidate terminal anchors do not align with groups")
        if not isinstance(prefix_count, int) or not 1 <= prefix_count <= terminal:
            raise ValueError("paired-query source-prefix token count is invalid")
        if any(not 0 <= value < prefix_count for value in candidate_terminals):
            raise ValueError("paired-query candidate terminal is not before Q")
        positives = sum(label_matrix[query_offset])
        derived_outcome = (
            "abstain" if positives == 0 else "answer" if positives == 1 else "conflict"
        )
        target_outcome = str(raw_query.get("target_outcome", derived_outcome))
        if target_outcome != derived_outcome:
            raise ValueError("paired-query target outcome differs from its label row")
        queries.append(
            EncodedPairedQueryLane(
                question=question,
                token_ids=token_ids,
                query_terminal_index=int(terminal),
                candidate_terminal_indices=candidate_terminals,
                source_prefix_token_count=prefix_count,
                target_outcome=target_outcome,
                metadata=dict(raw_query),
            )
        )
    if queries[0].question == queries[1].question:
        raise ValueError("paired-query questions must differ")
    if queries[0].source_prefix_token_count != queries[1].source_prefix_token_count:
        raise ValueError("paired-query source-prefix lengths differ")
    if queries[0].candidate_terminal_indices != queries[1].candidate_terminal_indices:
        raise ValueError("paired-query candidate anchors differ between lanes")
    left = (BOS_TOKEN_ID, *queries[0].token_ids)
    right = (BOS_TOKEN_ID, *queries[1].token_ids)
    prefix_count = queries[0].source_prefix_token_count
    if left[:prefix_count] != right[:prefix_count]:
        raise ValueError("paired-query encoded source prefixes differ")
    if max(len(left), len(right)) > FROZEN_MODEL_CONFIG.max_position_embeddings:
        raise ValueError("paired-query input exceeds the 256-token context")

    return EncodedPairedQueryRecord(
        record_id=record_id,
        source_width=int(source_width),
        pair_slot=pair_slot,
        lexical_world=lexical_world,
        queries=(queries[0], queries[1]),
        candidate_groups=tuple(groups),
        label_matrix=(label_matrix[0], label_matrix[1]),
        flip_group_indices=flip_indices,
        source_prefix_token_ids_cid=prefix_cid,
        metadata=dict(record),
    )


class EncodedPairedQueryBindingDataset:
    """Tokenizer-bound pairs, variable candidate groups, and frozen fit schedule."""

    def __init__(self, records: Sequence[Mapping[str, Any]]) -> None:
        if not records:
            raise ValueError("paired-query dataset is empty")
        encoded = [_encoded_record(record, offset) for offset, record in enumerate(records)]
        self.records = tuple(sorted(encoded, key=lambda record: record.record_id))
        if len({record.record_id for record in self.records}) != len(self.records):
            raise ValueError("paired-query record identities are not unique")
        mutable: dict[int, list[int]] = defaultdict(list)
        for index, record in enumerate(self.records):
            mutable[record.source_width].append(index)
        self._by_width = {
            width: tuple(
                sorted(
                    indices,
                    key=lambda index: (
                        self.records[index].lexical_world,
                        self.records[index].pair_slot,
                        self.records[index].record_id,
                    ),
                )
            )
            for width, indices in mutable.items()
        }

    def __len__(self) -> int:
        return len(self.records)

    def validate_fit_schedule(self) -> None:
        if set(self._by_width) != set(SOURCE_WIDTHS):
            raise ValueError("paired-query fit schedule lacks a source width")
        if any(
            len(self._by_width[width]) != PAIRED_RECORDS_PER_WIDTH
            for width in SOURCE_WIDTHS
        ):
            raise ValueError("paired-query fit requires eight records per source width")
        if len(self.records) != RECORDS_PER_STEP * STEPS_PER_EPOCH:
            raise ValueError("paired-query fit requires exactly 56 paired records")
        for step in range(1, STEPS_PER_EPOCH + 1):
            selected = self.record_indices_for_step(step)
            if len(selected) != RECORDS_PER_STEP or len(set(selected)) != len(selected):
                raise RuntimeError("paired-query epoch schedule is not seven unique pairs")
        observed = {
            index
            for step in range(1, STEPS_PER_EPOCH + 1)
            for index in self.record_indices_for_step(step)
        }
        if observed != set(range(len(self.records))):
            raise RuntimeError("paired-query schedule does not cover one exact epoch")

    def record_indices_for_step(self, step: int) -> tuple[int, ...]:
        if step < 1:
            raise ValueError("paired-query schedule step must be positive")
        within_epoch = (step - 1) % STEPS_PER_EPOCH
        epoch = (step - 1) // STEPS_PER_EPOCH
        selected = tuple(
            self._by_width[width][
                (within_epoch + epoch + width - SOURCE_WIDTHS[0])
                % PAIRED_RECORDS_PER_WIDTH
            ]
            for width in SOURCE_WIDTHS
        )
        if len(selected) != RECORDS_PER_STEP:
            raise RuntimeError("paired-query schedule did not select seven records")
        return selected

    def batch(
        self, record_indices: Sequence[int], *, device: torch.device
    ) -> PairedQueryTokenBatch:
        if not record_indices or len(set(record_indices)) != len(record_indices):
            raise ValueError("paired-query batch must contain unique records")
        selected = [self.records[index] for index in record_indices]
        time_width = max(
            len(query.token_ids) + 1
            for record in selected
            for query in record.queries
        )
        groups = max(len(record.candidate_groups) for record in selected)
        pairs = len(selected)
        input_ids = torch.full(
            (pairs, 2, time_width), EOS_TOKEN_ID, dtype=torch.long
        )
        query_indices = torch.empty((pairs, 2), dtype=torch.long)
        candidate_indices = torch.zeros((pairs, groups), dtype=torch.long)
        group_mask = torch.zeros((pairs, groups), dtype=torch.bool)
        labels = torch.zeros((pairs, 2, groups), dtype=torch.float32)
        flip_mask = torch.zeros((pairs, groups), dtype=torch.bool)
        source_prefix_lengths = torch.empty(pairs, dtype=torch.long)
        group_identities: list[tuple[str, ...]] = []
        for pair_index, record in enumerate(selected):
            group_count = len(record.candidate_groups)
            group_mask[pair_index, :group_count] = True
            labels[pair_index, :, :group_count] = torch.tensor(
                record.label_matrix, dtype=torch.float32
            )
            flip_mask[pair_index, list(record.flip_group_indices)] = True
            candidate_indices[pair_index, :group_count] = torch.tensor(
                record.queries[0].candidate_terminal_indices, dtype=torch.long
            )
            source_prefix_lengths[pair_index] = (
                record.queries[0].source_prefix_token_count
            )
            group_identities.append(
                tuple(group.relation_group_cid for group in record.candidate_groups)
            )
            for query_index, query in enumerate(record.queries):
                lane = torch.tensor(
                    (BOS_TOKEN_ID, *query.token_ids), dtype=torch.long
                )
                input_ids[pair_index, query_index, : lane.numel()] = lane
                query_indices[pair_index, query_index] = query.query_terminal_index
        batch = PairedQueryTokenBatch(
            input_ids=input_ids,
            query_indices=query_indices,
            candidate_indices=candidate_indices,
            group_mask=group_mask,
            labels=labels,
            flip_mask=flip_mask,
            source_prefix_lengths=source_prefix_lengths,
            record_indices=tuple(record_indices),
            group_identities=tuple(group_identities),
        )
        batch.validate()
        return batch.to(device)


@dataclass(frozen=True, slots=True)
class PairedQueryLossTerms:
    row_losses: Tensor
    flip_losses: Tensor
    mean_row_loss: Tensor
    mean_flip_loss: Tensor
    total: Tensor


def paired_query_loss_terms(
    scores: Tensor,
    labels: Tensor,
    group_mask: Tensor,
    flip_mask: Tensor,
) -> PairedQueryLossTerms:
    """Return row-extrema and direct counterfactual-column margin terms."""
    if scores.ndim != 3 or scores.shape[1] != 2:
        raise ValueError("paired-query scores must have shape [pairs, 2, groups]")
    if labels.shape != scores.shape:
        raise ValueError("paired-query labels do not match the score matrix")
    pairs, _, groups = scores.shape
    if group_mask.shape != (pairs, groups) or group_mask.dtype != torch.bool:
        raise ValueError("paired-query group mask shape or dtype differs")
    if flip_mask.shape != (pairs, groups) or flip_mask.dtype != torch.bool:
        raise ValueError("paired-query flip mask shape or dtype differs")
    if labels.dtype not in (torch.float16, torch.float32, torch.float64):
        raise ValueError("paired-query labels must be floating point")
    if not (
        scores.device == labels.device == group_mask.device == flip_mask.device
    ):
        raise ValueError("paired-query loss tensors must share one device")
    # Content invariants are checked once on the CPU token-batch boundary.  Keep
    # direct CPU calls fail-closed without introducing device synchronizations in
    # every MPS optimizer step.
    if scores.device.type == "cpu":
        active_labels = labels.masked_select(group_mask[:, None, :])
        if bool(((active_labels != 0.0) & (active_labels != 1.0)).any()):
            raise ValueError("paired-query active labels must be binary")
        if bool((flip_mask & ~group_mask).any()) or bool(
            (~flip_mask.any(dim=1)).any()
        ):
            raise ValueError("every paired record requires an admitted flip column")
        flip_labels = labels.permute(0, 2, 1)[flip_mask]
        if bool((flip_labels.sum(dim=1) != 1.0).any()):
            raise ValueError("flip column must contain one positive query row")

    score_values = scores.float()
    admitted = group_mask[:, None, :]
    positive_mask = admitted & (labels == 1.0)
    negative_mask = admitted & (labels == 0.0)
    positive_minimum = score_values.masked_fill(~positive_mask, float("inf")).amin(
        dim=-1
    )
    negative_maximum = score_values.masked_fill(~negative_mask, float("-inf")).amax(
        dim=-1
    )
    positive_loss = torch.where(
        positive_mask.any(dim=-1),
        F.relu(score_values.new_tensor(MARGIN) - positive_minimum),
        torch.zeros_like(positive_minimum),
    )
    negative_loss = torch.where(
        negative_mask.any(dim=-1),
        F.relu(score_values.new_tensor(MARGIN) + negative_maximum),
        torch.zeros_like(negative_maximum),
    )
    row_tensor = (positive_loss + negative_loss).reshape(pairs * 2)

    # Each required column contains exactly one positive row.  Orient the row
    # difference so a correct pair always has ``positive - negative >= 2``.
    orientation = labels[:, 0].float().mul(2.0).sub(1.0)
    oriented_gap = (score_values[:, 0] - score_values[:, 1]) * orientation
    flip_tensor = F.relu(
        score_values.new_tensor(FLIP_MARGIN) - oriented_gap[flip_mask]
    )
    if not row_tensor.numel() or not flip_tensor.numel():
        raise ValueError("paired-query loss has no admitted rows or flips")
    mean_row = row_tensor.mean()
    mean_flip = flip_tensor.mean()
    return PairedQueryLossTerms(
        row_losses=row_tensor,
        flip_losses=flip_tensor,
        mean_row_loss=mean_row,
        mean_flip_loss=mean_flip,
        total=mean_row + mean_flip,
    )


def paired_query_binding_loss(
    scores: Tensor,
    labels: Tensor,
    group_mask: Tensor,
    flip_mask: Tensor,
) -> Tensor:
    return paired_query_loss_terms(scores, labels, group_mask, flip_mask).total


def _row_evaluation(
    record: EncodedPairedQueryRecord,
    *,
    query_index: int,
    scores: Sequence[float],
    labels: Sequence[int],
) -> dict[str, Any]:
    supported = [index for index, score in enumerate(scores) if score > 0.0]
    predicted_outcome = (
        "abstain"
        if not supported
        else "answer"
        if len(supported) == 1
        else "conflict"
    )
    positive = [index for index, label in enumerate(labels) if label == 1]
    target_outcome = record.queries[query_index].target_outcome
    predicted_copy = (
        record.candidate_groups[supported[0]].earliest_occurrence_index
        if predicted_outcome == "answer"
        else None
    )
    target_copy = (
        record.candidate_groups[positive[0]].earliest_occurrence_index
        if target_outcome == "answer"
        else None
    )
    cells_exact = all((score > 0.0) == bool(label) for score, label in zip(scores, labels))
    return {
        "question": record.queries[query_index].question,
        "target_outcome": target_outcome,
        "predicted_outcome": predicted_outcome,
        "outcome_exact": predicted_outcome == target_outcome,
        "target_copy_candidate_index": target_copy,
        "predicted_copy_candidate_index": predicted_copy,
        "copy_exact": predicted_copy == target_copy,
        "positive_group_indices": positive,
        "predicted_positive_group_indices": supported,
        "cells_exact": cells_exact,
        "cells": [
            {
                "relation_group_cid": group.relation_group_cid,
                "label": int(label),
                "score": float(score),
                "supported": bool(score > 0.0),
            }
            for group, label, score in zip(record.candidate_groups, labels, scores)
        ],
    }


@torch.no_grad()
def evaluate_paired_query_binding(
    adapter: R4PairedQueryCandidateMatrix,
    dataset: EncodedPairedQueryBindingDataset,
    *,
    device: torch.device,
    pair_batch_size: int = RECORDS_PER_STEP,
    attention_off: bool = False,
    mean_query_ablation: bool = False,
    row_swap: bool = False,
) -> dict[str, Any]:
    if pair_batch_size < 1:
        raise ValueError("paired-query evaluation batch size must be positive")
    adapter.eval()
    evaluations: list[dict[str, Any]] = []
    row_loss_sum = 0.0
    row_loss_count = 0
    flip_loss_sum = 0.0
    flip_loss_count = 0
    for base in range(0, len(dataset), pair_batch_size):
        indices = tuple(range(base, min(base + pair_batch_size, len(dataset))))
        batch = dataset.batch(indices, device=device)
        if row_swap:
            batch = batch.swapped_query_rows()
        output = adapter(
            batch,
            attention_off=attention_off,
            mean_query_ablation=mean_query_ablation,
        )
        terms = paired_query_loss_terms(
            output.scores, batch.labels, batch.group_mask, batch.flip_mask
        )
        row_loss_sum += float(terms.row_losses.sum().cpu())
        row_loss_count += terms.row_losses.numel()
        flip_loss_sum += float(terms.flip_losses.sum().cpu())
        flip_loss_count += terms.flip_losses.numel()
        for lane, record_index in enumerate(indices):
            record = dataset.records[record_index]
            group_count = len(record.candidate_groups)
            scores = output.scores[lane, :, :group_count].detach().float().cpu()
            labels = batch.labels[lane, :, :group_count].detach().int().cpu()
            query_order = (1, 0) if row_swap else (0, 1)
            rows = [
                _row_evaluation(
                    record,
                    query_index=source_query_index,
                    scores=scores[output_query_index].tolist(),
                    labels=labels[output_query_index].tolist(),
                )
                for output_query_index, source_query_index in enumerate(query_order)
            ]
            flip_columns = [
                {
                    "relation_group_cid": record.candidate_groups[
                        group_index
                    ].relation_group_cid,
                    "exact": (
                        rows[0]["cells"][group_index]["supported"]
                        != rows[1]["cells"][group_index]["supported"]
                        and rows[0]["cells"][group_index]["supported"]
                        == bool(labels[0, group_index])
                        and rows[1]["cells"][group_index]["supported"]
                        == bool(labels[1, group_index])
                    ),
                }
                for group_index in record.flip_group_indices
            ]
            flip_exact = all(bool(column["exact"]) for column in flip_columns)
            paired_rows_identical = torch.equal(scores[0], scores[1])
            evaluations.append(
                {
                    "record_id": record.record_id,
                    "source_width": record.source_width,
                    "pair_slot": record.pair_slot,
                    "candidate_state_identity": output.paired_candidate_states_exact,
                    "query_rows": rows,
                    "flip_columns": flip_columns,
                    "flip_exact": flip_exact,
                    "paired_rows_identical": paired_rows_identical,
                    "pair_exact": flip_exact
                    and all(
                        bool(row["cells_exact"])
                        and bool(row["outcome_exact"])
                        and bool(row["copy_exact"])
                        for row in rows
                    ),
                }
            )
    mean_row = row_loss_sum / row_loss_count
    mean_flip = flip_loss_sum / flip_loss_count
    all_rows = [
        row
        for evaluation in evaluations
        for row in evaluation["query_rows"]
    ]
    all_cells = [cell for row in all_rows for cell in row["cells"]]
    all_flips = [
        column
        for evaluation in evaluations
        for column in evaluation["flip_columns"]
    ]
    answer_rows = [row for row in all_rows if row["target_outcome"] == "answer"]
    duplicate_pairs = [
        evaluation for evaluation in evaluations if evaluation["pair_slot"] == 3
    ]

    def fraction(correct: int, total: int) -> dict[str, int]:
        return {"correct": correct, "total": total}

    outcome = {
        target: fraction(
            sum(
                bool(row["outcome_exact"])
                and bool(row["cells_exact"])
                and bool(row["copy_exact"])
                for row in all_rows
                if row["target_outcome"] == target
            ),
            sum(row["target_outcome"] == target for row in all_rows),
        )
        for target in ("answer", "abstain", "conflict")
    }
    return {
        "pairs": len(evaluations),
        "query_rows": 2 * len(evaluations),
        "matrix_cells": sum(
            2 * len(dataset.records[index].candidate_groups)
            for index in range(len(dataset))
        ),
        "flip_columns": sum(
            len(dataset.records[index].flip_group_indices)
            for index in range(len(dataset))
        ),
        "mean_row_margin": mean_row,
        "mean_flip_margin": mean_flip,
        "mean_total_loss": mean_row + mean_flip,
        "mean_loss": mean_row + mean_flip,
        "pair_exact": fraction(
            sum(bool(row["pair_exact"]) for row in evaluations), len(evaluations)
        ),
        "row_exact": fraction(
            sum(
                bool(row["cells_exact"])
                and bool(row["outcome_exact"])
                and bool(row["copy_exact"])
                for row in all_rows
            ),
            len(all_rows),
        ),
        "cell_exact": fraction(
            sum(bool(cell["supported"]) == bool(cell["label"]) for cell in all_cells),
            len(all_cells),
        ),
        "flip_exact": fraction(
            sum(bool(column["exact"]) for column in all_flips), len(all_flips)
        ),
        "candidate_copy_exact": fraction(
            sum(bool(row["copy_exact"]) for row in answer_rows), len(answer_rows)
        ),
        "duplicate_pair_exact": fraction(
            sum(bool(row["pair_exact"]) for row in duplicate_pairs),
            len(duplicate_pairs),
        ),
        "outcome": outcome,
        "candidate_state_bit_identity": fraction(
            sum(bool(row["candidate_state_identity"]) for row in evaluations),
            len(evaluations),
        ),
        "paired_rows_identical": fraction(
            sum(bool(row["paired_rows_identical"]) for row in evaluations),
            len(evaluations),
        ),
        "attention_off": attention_off,
        "mean_query_ablation": mean_query_ablation,
        "row_swap": row_swap,
        "candidate_state_identity_exact": all(
            bool(row["candidate_state_identity"]) for row in evaluations
        ),
        "pair_evaluations": evaluations,
    }


def fit_paired_query_binding(
    adapter: R4PairedQueryCandidateMatrix,
    dataset: EncodedPairedQueryBindingDataset,
    *,
    config: PairedQueryBindingFitConfig = PairedQueryBindingFitConfig(),
) -> dict[str, Any]:
    """Run the sole 120-update MPS fit without reading sealed or product data."""
    config.validate()
    if config.seed != adapter.config.initialization_seed:
        raise ValueError("paired-query fit seed differs from model initialization")
    dataset.validate_fit_schedule()
    device = require_mps(config.seed)
    adapter.to(device)
    adapter.train()
    parameters = list(adapter.adapter_parameters())
    if sum(parameter.numel() for parameter in parameters) != TRAINABLE_PARAMETER_COUNT:
        raise RuntimeError("paired-query optimizer parameter census differs")
    optimizer = torch.optim.AdamW(
        parameters,
        lr=config.learning_rate,
        betas=(config.adam_beta1, config.adam_beta2),
        eps=config.adam_epsilon,
        weight_decay=config.weight_decay,
    )
    started = time.monotonic()
    initial_loss: float | None = None
    final_loss = math.nan
    projected_seconds: float | None = None
    elapsed_seconds = 0.0
    for step in range(1, config.optimizer_steps + 1):
        indices = dataset.record_indices_for_step(step)
        batch = dataset.batch(indices, device=device)
        optimizer.zero_grad(set_to_none=True)
        # Exact causal candidate-state identity is established by the final
        # fit/sealed evaluators.  Rechecking it here would turn every optimizer
        # step into a device-to-host synchronization.
        output = adapter(batch, verify_candidate_state_identity=False)
        loss = paired_query_binding_loss(
            output.scores, batch.labels, batch.group_mask, batch.flip_mask
        )
        loss.backward()
        torch.nn.utils.clip_grad_norm_(parameters, config.gradient_clip)
        optimizer.step()
        synchronized_boundary = (
            step == 1
            or step % config.progress_interval == 0
            or step == config.optimizer_steps
        )
        if synchronized_boundary:
            if device.type == "mps":
                torch.mps.synchronize()
            final_loss = float(loss.detach().cpu())
            if initial_loss is None:
                initial_loss = final_loss
            if not math.isfinite(final_loss):
                raise RuntimeError("paired-query binding produced a nonfinite loss")
            elapsed_seconds = time.monotonic() - started
            if step == config.eta_probe_step:
                projected_seconds = elapsed_seconds * config.optimizer_steps / step
                print(
                    f"paired_query_binding_eta_probe_step={step} "
                    f"elapsed_seconds={elapsed_seconds:.3f} "
                    f"projected_seconds={projected_seconds:.3f} "
                    f"ceiling_seconds={config.wall_ceiling_seconds:.3f}",
                    flush=True,
                )
                if projected_seconds > config.wall_ceiling_seconds:
                    raise PairedQueryBindingWallBudgetExceeded(
                        step=step,
                        elapsed_seconds=elapsed_seconds,
                        projected_seconds_at_eta_probe=projected_seconds,
                        wall_ceiling_seconds=config.wall_ceiling_seconds,
                    )
            if elapsed_seconds > config.wall_ceiling_seconds:
                raise PairedQueryBindingWallBudgetExceeded(
                    step=step,
                    elapsed_seconds=elapsed_seconds,
                    projected_seconds_at_eta_probe=projected_seconds,
                    wall_ceiling_seconds=config.wall_ceiling_seconds,
                )
            if step % config.progress_interval == 0 or step == config.optimizer_steps:
                print(
                    f"paired_query_binding_step={step}/{config.optimizer_steps} "
                    f"loss={final_loss:.6f}",
                    flush=True,
                )
        else:
            # Host elapsed time above the ceiling is already sufficient to stop;
            # below the ceiling it is not treated as device-completion timing.
            dispatch_elapsed = time.monotonic() - started
            if dispatch_elapsed > config.wall_ceiling_seconds:
                raise PairedQueryBindingWallBudgetExceeded(
                    step=step,
                    elapsed_seconds=dispatch_elapsed,
                    projected_seconds_at_eta_probe=projected_seconds,
                    wall_ceiling_seconds=config.wall_ceiling_seconds,
                )
    return {
        "optimizer_steps": config.optimizer_steps,
        "paired_records_per_step": config.records_per_step,
        "initial_loss": initial_loss,
        "final_loss": final_loss,
        "elapsed_seconds": elapsed_seconds,
        "eta_probe_step": config.eta_probe_step,
        "projected_seconds_at_eta_probe": projected_seconds,
        "wall_ceiling_seconds": config.wall_ceiling_seconds,
        "trainable_parameter_count": TRAINABLE_PARAMETER_COUNT,
        "delta_audit": adapter.delta_audit(),
        "binding_head_audit": adapter.binding_head_audit(),
    }


__all__ = [
    "BINDING_BLOCKS",
    "BINDING_HEAD_PARAMETER_COUNT",
    "BINDING_RANK",
    "FIT_SEED",
    "FLIP_MARGIN",
    "MARGIN",
    "OPTIMIZER_STEPS",
    "POLICY",
    "RECORDS_PER_STEP",
    "STEPS_PER_EPOCH",
    "TRAINABLE_PARAMETER_COUNT",
    "AsymmetricR4BindingHead",
    "EncodedPairedQueryBindingDataset",
    "PairedQueryBindingAdapterConfig",
    "PairedQueryBindingFitConfig",
    "PairedQueryBindingOutput",
    "PairedQueryBindingWallBudgetExceeded",
    "PairedQueryTokenBatch",
    "R4PairedQueryCandidateMatrix",
    "evaluate_paired_query_binding",
    "fit_paired_query_binding",
    "paired_query_binding_loss",
    "paired_query_loss_terms",
]
