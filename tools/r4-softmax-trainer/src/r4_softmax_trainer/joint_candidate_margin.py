"""Record-level structured-margin training over joint source/candidate prompts.

This module is the small C1-SB4 mechanism core.  It deliberately reuses the
mergeable all-layer Q/K/V/O LoRA representation and frozen tied-token readout
from C1-SB3, but changes the unit of learning from independent candidate rows
to complete records.  Exact duplicate text is one relation group, and every
group prompt contains the complete source before it is scored.
"""

from __future__ import annotations

import math
import time
from collections import defaultdict
from collections.abc import Iterable, Mapping, Sequence
from dataclasses import dataclass
from typing import Any

import torch
from tokenizers import Tokenizer
from torch import Tensor, nn
from torch.nn import functional as F

from .constants import BOS_TOKEN_ID, EOS_TOKEN_ID, FROZEN_MODEL_CONFIG
from .joint_candidate_margin_data import render_joint_candidate_input
from .model import R4SoftmaxForCausalLM, expected_hf_tensor_names
from .provenance import cid_bytes
from .source_relation_adapter import (
    LORA_ALPHA,
    LORA_DROPOUT,
    LORA_RANK,
    TRAINABLE_PARAMETER_COUNT,
    LoRALinear,
    tied_relation_scores,
    validate_tokenizer_contract,
)
from .train import require_mps


POLICY = "R4JointCandidateMarginAdapterV1"
REPRESENTATION_UPDATE = "lora_qkvo_all_layers"
OUTCOMES = ("answer", "abstain", "conflict")
SOURCE_WIDTHS = tuple(range(2, 9))
MARGIN = 1.0
FIT_RECORDS_PER_WIDTH = 18
OUTCOME_RECORDS_PER_WIDTH = 6
RECORDS_PER_STEP = len(SOURCE_WIDTHS)
STEPS_PER_EPOCH = FIT_RECORDS_PER_WIDTH
OPTIMIZER_STEPS = 270
FIT_SEED = 9_544


@dataclass(frozen=True, slots=True)
class JointCandidateMarginAdapterConfig:
    """SB4's separately versioned all-layer representation contract."""

    rank: int = LORA_RANK
    alpha: int = LORA_ALPHA
    dropout: float = LORA_DROPOUT
    target_layer_indices: tuple[int, ...] = tuple(
        range(FROZEN_MODEL_CONFIG.num_hidden_layers)
    )
    target_projections: tuple[str, ...] = ("q_proj", "k_proj", "v_proj", "o_proj")
    initialization_seed: int = FIT_SEED

    def validate(self) -> None:
        if self != JointCandidateMarginAdapterConfig():
            raise ValueError("joint candidate margin exposes one frozen adapter contract")
        if self.rank != 8 or self.alpha != 8 or self.dropout != 0.0:
            raise AssertionError("joint candidate margin LoRA shape drifted")
        if self.target_layer_indices != tuple(range(6)):
            raise AssertionError("joint candidate margin must adapt all six layers")
        if self.target_projections != ("q_proj", "k_proj", "v_proj", "o_proj"):
            raise AssertionError("joint candidate margin must adapt exactly Q/K/V/O")
        if self.initialization_seed != 9_544:
            raise AssertionError("joint candidate margin initialization seed drifted")

    @property
    def trainable_parameter_count(self) -> int:
        self.validate()
        return TRAINABLE_PARAMETER_COUNT

    def as_contract(self) -> dict[str, Any]:
        self.validate()
        return {
            "rank": self.rank,
            "alpha": self.alpha,
            "dropout": self.dropout,
            "target_layer_indices": list(self.target_layer_indices),
            "target_projections": list(self.target_projections),
            "initialization_seed": self.initialization_seed,
            "trainable_parameter_count": self.trainable_parameter_count,
            "representation_update": REPRESENTATION_UPDATE,
            "initialization": (
                "Xavier-uniform A from isolated CPU generators seeded "
                "initialization_seed + stable layer/projection ordinal; zero B"
            ),
            "score": "tied-logit[1771] - tied-logit[542] at final colon",
            "decision": "supported iff score > +0.0",
            "learned_head": False,
        }


class R4JointCandidateMarginAdapter(nn.Module):
    """SB4 LoRA representation with the unchanged frozen tied-token score."""

    def __init__(
        self,
        model: R4SoftmaxForCausalLM,
        config: JointCandidateMarginAdapterConfig = JointCandidateMarginAdapterConfig(),
    ) -> None:
        super().__init__()
        config.validate()
        if model.config != FROZEN_MODEL_CONFIG:
            raise ValueError("joint candidate margin requires the six-layer #1017 model")
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
                path = self._projection_weight_path(layer_index, projection_name)
                self._adapters[path] = adapter
                adapter_ordinal += 1
        observed = sum(
            parameter.numel()
            for parameter in self.parameters()
            if parameter.requires_grad
        )
        if observed != config.trainable_parameter_count:
            raise AssertionError(
                f"joint candidate margin has {observed} trainable parameters, "
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

    def forward(self, input_ids: Tensor, terminal_indices: Tensor | None = None) -> Tensor:
        if input_ids.ndim != 2:
            raise ValueError("joint candidate input_ids must have shape [batch, time]")
        hidden = self.model.model(input_ids)
        batch, time, width = hidden.shape
        if width != FROZEN_MODEL_CONFIG.hidden_size:
            raise RuntimeError("joint candidate hidden width differs from #1017")
        if terminal_indices is None:
            terminal_indices = torch.full(
                (batch,), time - 1, dtype=torch.long, device=hidden.device
            )
        if terminal_indices.shape != (batch,) or terminal_indices.dtype != torch.long:
            raise ValueError("terminal_indices must be int64 with shape [batch]")
        if bool(((terminal_indices < 0) | (terminal_indices >= time)).any()):
            raise ValueError("joint candidate terminal index is outside the sequence")
        terminal = hidden[torch.arange(batch, device=hidden.device), terminal_indices]
        return tied_relation_scores(terminal, self.model.model.embed_tokens.weight)

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
                raise RuntimeError(
                    f"joint candidate wrapped model is missing tensor {name}"
                ) from error
            merged[name] = (
                value.detach().to(device="cpu", dtype=torch.float32).contiguous()
            )
        if set(merged) != expected or len(self._adapters) != 24:
            raise RuntimeError("joint candidate merge did not cover 24 Q/K/V/O tensors")
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
                    "l2_norm": float(torch.linalg.vector_norm(delta).cpu()),
                    "finite": bool(torch.isfinite(delta).all()),
                    "initialization_seed": adapter.initialization_seed,
                }
            )
        if len(tensors) != 24 or any(not tensor["finite"] for tensor in tensors):
            raise RuntimeError("joint candidate delta census is incomplete or nonfinite")
        return {
            "target_tensor_count": len(tensors),
            "trainable_parameter_count": sum(
                int(tensor["parameter_count"]) for tensor in tensors
            ),
            "tensors": tensors,
        }

    def merged_model(self) -> R4SoftmaxForCausalLM:
        clean = R4SoftmaxForCausalLM(self.model.config)
        clean.load_state_dict(self.merged_state_dict(), strict=True)
        clean.eval()
        return clean


@dataclass(frozen=True, slots=True)
class JointCandidateGroup:
    """One exact-text equivalence class inside a complete source record."""

    relation_group_cid: str
    text: str
    relation_label: int
    occurrence_indices: tuple[int, ...]
    relation_input: str
    relation_input_cid: str


@dataclass(frozen=True, slots=True)
class JointCandidateRecord:
    record_id: str
    lexical_world: str
    motif: str
    target_outcome: str
    source_width: int
    source: str
    question: str
    groups: tuple[JointCandidateGroup, ...]


@dataclass(frozen=True, slots=True)
class EncodedJointCandidateGroup:
    relation_group_cid: str
    text: str
    relation_label: int
    occurrence_indices: tuple[int, ...]
    token_ids: tuple[int, ...]

    @property
    def terminal_index(self) -> int:
        # BOS is inserted at lane zero; the last content token is the colon.
        return len(self.token_ids)


@dataclass(frozen=True, slots=True)
class EncodedJointCandidateRecord:
    record_id: str
    lexical_world: str
    motif: str
    target_outcome: str
    source_width: int
    groups: tuple[EncodedJointCandidateGroup, ...]


@dataclass(slots=True)
class JointCandidateMarginBatch:
    input_ids: Tensor
    terminal_indices: Tensor
    group_labels: Tensor
    record_slices: tuple[tuple[int, int], ...]
    record_indices: tuple[int, ...]
    group_identities: tuple[tuple[str, str], ...]


@dataclass(frozen=True, slots=True)
class JointCandidateMarginFitConfig:
    """The sole bounded optimizer contract; this is intentionally not a sweep."""

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
    wall_ceiling_seconds: float = 600.0
    progress_interval: int = STEPS_PER_EPOCH

    def validate(self) -> None:
        if self != JointCandidateMarginFitConfig():
            raise ValueError("joint candidate margin exposes one frozen fit contract")
        if (
            self.seed != 9_544
            or self.optimizer_steps != 270
            or self.records_per_step != 7
            or self.weight_decay != 0.0
        ):
            raise AssertionError("joint candidate margin fit contract drifted")

    def as_contract(self) -> dict[str, Any]:
        self.validate()
        return {
            "seed": self.seed,
            "optimizer_steps": self.optimizer_steps,
            "records_per_step": self.records_per_step,
            "learning_rate": self.learning_rate,
            "adam_beta1": self.adam_beta1,
            "adam_beta2": self.adam_beta2,
            "adam_epsilon": self.adam_epsilon,
            "weight_decay": self.weight_decay,
            "gradient_clip": self.gradient_clip,
            "eta_probe_step": self.eta_probe_step,
            "wall_ceiling_seconds": self.wall_ceiling_seconds,
            "progress_interval": self.progress_interval,
            "margin": MARGIN,
            "schedule": (
                "18 steps/epoch; one complete record per width 2..8; outcome bucket "
                "and lane offsets depend on width and epoch"
            ),
        }


class JointCandidateMarginWallBudgetExceeded(RuntimeError):
    def __init__(
        self,
        *,
        step: int,
        elapsed_seconds: float,
        projected_seconds_at_eta_probe: float | None,
        wall_ceiling_seconds: float,
    ) -> None:
        super().__init__("UNAVAILABLE_JOINT_CANDIDATE_MARGIN_WALL_BUDGET")
        self.step = step
        self.elapsed_seconds = elapsed_seconds
        self.projected_seconds_at_eta_probe = projected_seconds_at_eta_probe
        self.wall_ceiling_seconds = wall_ceiling_seconds

    def as_result(self) -> dict[str, Any]:
        return {
            "status": "UNAVAILABLE_JOINT_CANDIDATE_MARGIN_WALL_BUDGET",
            "stopped_after_step": self.step,
            "elapsed_seconds": self.elapsed_seconds,
            "projected_seconds_at_eta_probe": self.projected_seconds_at_eta_probe,
            "wall_ceiling_seconds": self.wall_ceiling_seconds,
        }


def _record_identity(record: Mapping[str, Any], offset: int) -> str:
    value = record.get("record_id", record.get("record_cid"))
    if not isinstance(value, str) or not value:
        raise ValueError(f"joint candidate record {offset} has no stable identity")
    return value


def _record_from_mapping(
    record: Mapping[str, Any], *, record_offset: int
) -> JointCandidateRecord:
    record_id = _record_identity(record, record_offset)
    source = record.get("source")
    question = record.get("question")
    source_width = record.get("source_width")
    target_outcome = record.get("target_outcome")
    lexical_world = record.get("lexical_world")
    motif = record.get("motif")
    spans = record.get("sentence_spans")
    if not isinstance(source, str) or not source or source != source.strip():
        raise ValueError(f"joint candidate record {record_id} has no canonical source")
    if not isinstance(question, str) or not question.endswith("?"):
        raise ValueError(f"joint candidate record {record_id} has no canonical question")
    if source_width not in SOURCE_WIDTHS:
        raise ValueError(f"joint candidate record {record_id} has width outside 2..=8")
    if target_outcome not in OUTCOMES:
        raise ValueError(f"joint candidate record {record_id} has an invalid outcome")
    if not isinstance(lexical_world, str) or not lexical_world:
        raise ValueError(f"joint candidate record {record_id} has no lexical world")
    if not isinstance(motif, str) or not motif:
        raise ValueError(f"joint candidate record {record_id} has no motif")
    if not isinstance(spans, Sequence) or isinstance(spans, (str, bytes)):
        raise ValueError(f"joint candidate record {record_id} has invalid spans")
    if len(spans) != source_width:
        raise ValueError(f"joint candidate record {record_id} width differs from spans")

    source_bytes = source.encode("utf-8")
    grouped: dict[str, dict[str, Any]] = {}
    text_to_group: dict[str, str] = {}
    for expected_index, span in enumerate(spans):
        if not isinstance(span, Mapping):
            raise ValueError(f"joint candidate record {record_id} span is not a mapping")
        candidate_index = span.get("candidate_index")
        text = span.get("text")
        group_cid = span.get("relation_group_cid")
        label = span.get("relation_label")
        byte_start = span.get("byte_start")
        byte_end = span.get("byte_end")
        if candidate_index != expected_index:
            raise ValueError(
                f"joint candidate record {record_id} candidate indices are not canonical"
            )
        if not isinstance(text, str) or not text:
            raise ValueError(f"joint candidate record {record_id} has empty candidate text")
        if not isinstance(group_cid, str) or group_cid != cid_bytes(text.encode("utf-8")):
            raise ValueError(
                f"joint candidate record {record_id} relation group is not exact text CID"
            )
        if label not in (0, 1):
            raise ValueError(f"joint candidate record {record_id} has a nonbinary label")
        if (
            not isinstance(byte_start, int)
            or not isinstance(byte_end, int)
            or not 0 <= byte_start < byte_end <= len(source_bytes)
            or source_bytes[byte_start:byte_end] != text.encode("utf-8")
        ):
            raise ValueError(
                f"joint candidate record {record_id} span bytes do not bind exact source"
            )
        expected_input = render_joint_candidate_input(source, question, text)
        embedded_input = span.get("relation_input")
        embedded_input_cid = span.get("relation_input_cid")
        if embedded_input != expected_input:
            raise ValueError(
                f"joint candidate record {record_id} committed renderer differs"
            )
        if embedded_input_cid != cid_bytes(expected_input.encode("utf-8")):
            raise ValueError(
                f"joint candidate record {record_id} committed input CID differs"
            )
        prior_group = text_to_group.setdefault(text, group_cid)
        if prior_group != group_cid:
            raise ValueError(
                f"joint candidate record {record_id} exact text has multiple group IDs"
            )
        group = grouped.get(group_cid)
        if group is None:
            grouped[group_cid] = {
                "text": text,
                "label": int(label),
                "occurrences": [candidate_index],
            }
        else:
            if group["text"] != text:
                raise ValueError(
                    f"joint candidate record {record_id} group contains different text"
                )
            if group["label"] != int(label):
                raise ValueError(
                    f"joint candidate record {record_id} duplicate group labels disagree"
                )
            group["occurrences"].append(candidate_index)

    groups: list[JointCandidateGroup] = []
    for group_cid, value in sorted(
        grouped.items(), key=lambda item: int(item[1]["occurrences"][0])
    ):
        relation_input = render_joint_candidate_input(source, question, value["text"])
        groups.append(
            JointCandidateGroup(
                relation_group_cid=group_cid,
                text=value["text"],
                relation_label=int(value["label"]),
                occurrence_indices=tuple(int(index) for index in value["occurrences"]),
                relation_input=relation_input,
                relation_input_cid=cid_bytes(relation_input.encode("utf-8")),
            )
        )
    positive_groups = sorted(
        group.relation_group_cid for group in groups if group.relation_label == 1
    )
    derived_outcome = (
        "abstain"
        if not positive_groups
        else "answer"
        if len(positive_groups) == 1
        else "conflict"
    )
    if derived_outcome != target_outcome:
        raise ValueError(
            f"joint candidate record {record_id} labels derive {derived_outcome}, "
            f"not {target_outcome}"
        )
    committed_positive = record.get("positive_relation_group_cids")
    if committed_positive is not None and list(committed_positive) != positive_groups:
        raise ValueError(
            f"joint candidate record {record_id} positive group commitment differs"
        )
    return JointCandidateRecord(
        record_id=record_id,
        lexical_world=lexical_world,
        motif=motif,
        target_outcome=target_outcome,
        source_width=int(source_width),
        source=source,
        question=question,
        groups=tuple(groups),
    )


class EncodedJointCandidateMarginDataset:
    """Tokenizer-bound complete records, exact groups, and frozen fit schedule."""

    def __init__(self, records: Sequence[Mapping[str, Any]], tokenizer: Tokenizer) -> None:
        if not records:
            raise ValueError("joint candidate margin dataset is empty")
        validate_tokenizer_contract(tokenizer)
        parsed = [
            _record_from_mapping(record, record_offset=offset)
            for offset, record in enumerate(records)
        ]
        parsed.sort(key=lambda record: record.record_id)
        identities = [record.record_id for record in parsed]
        if len(set(identities)) != len(identities):
            raise ValueError("joint candidate record identities are not unique")
        self.records = tuple(parsed)
        self.encoded: tuple[EncodedJointCandidateRecord, ...] = tuple(
            self._encode_record(record, tokenizer) for record in self.records
        )
        self._fit_buckets: dict[tuple[int, str], tuple[int, ...]] | None = None

    @staticmethod
    def _encode_record(
        record: JointCandidateRecord, tokenizer: Tokenizer
    ) -> EncodedJointCandidateRecord:
        groups: list[EncodedJointCandidateGroup] = []
        for group in record.groups:
            token_ids = tokenizer.encode(
                group.relation_input, add_special_tokens=False
            ).ids
            if not token_ids:
                raise ValueError("joint candidate input encoded to zero tokens")
            if len(token_ids) + 1 > FROZEN_MODEL_CONFIG.max_position_embeddings:
                raise ValueError(
                    "joint candidate input exceeds the frozen 256-token context including BOS"
                )
            if tokenizer.decode([token_ids[-1]], skip_special_tokens=False) != ":":
                raise ValueError("joint candidate input does not end in standalone colon")
            groups.append(
                EncodedJointCandidateGroup(
                    relation_group_cid=group.relation_group_cid,
                    text=group.text,
                    relation_label=group.relation_label,
                    occurrence_indices=group.occurrence_indices,
                    token_ids=tuple(int(token_id) for token_id in token_ids),
                )
            )
        return EncodedJointCandidateRecord(
            record_id=record.record_id,
            lexical_world=record.lexical_world,
            motif=record.motif,
            target_outcome=record.target_outcome,
            source_width=record.source_width,
            groups=tuple(groups),
        )

    def _validated_fit_buckets(self) -> dict[tuple[int, str], tuple[int, ...]]:
        if self._fit_buckets is not None:
            return self._fit_buckets
        mutable: dict[tuple[int, str], list[int]] = defaultdict(list)
        for index, record in enumerate(self.records):
            mutable[(record.source_width, record.target_outcome)].append(index)
        expected_keys = {
            (width, outcome) for width in SOURCE_WIDTHS for outcome in OUTCOMES
        }
        if set(mutable) != expected_keys:
            raise ValueError("fit schedule does not contain every width/outcome cell")
        buckets: dict[tuple[int, str], tuple[int, ...]] = {}
        for key in sorted(expected_keys):
            indices = tuple(
                sorted(
                    mutable[key],
                    key=lambda index: (
                        self.records[index].lexical_world,
                        self.records[index].motif,
                        self.records[index].record_id,
                    ),
                )
            )
            if len(indices) != OUTCOME_RECORDS_PER_WIDTH:
                raise ValueError(
                    "fit schedule requires exactly six records per width/outcome cell"
                )
            buckets[key] = indices
        if len(self.records) != len(SOURCE_WIDTHS) * FIT_RECORDS_PER_WIDTH:
            raise ValueError("fit schedule requires exactly 126 complete records")
        self._fit_buckets = buckets
        return buckets

    def record_indices_for_step(self, step: int) -> tuple[int, ...]:
        if step < 1:
            raise ValueError("joint candidate schedule step must be positive")
        buckets = self._validated_fit_buckets()
        slot = (step - 1) % STEPS_PER_EPOCH
        epoch = (step - 1) // STEPS_PER_EPOCH
        outcome_block = slot // OUTCOME_RECORDS_PER_WIDTH
        lane = slot % OUTCOME_RECORDS_PER_WIDTH
        selected: list[int] = []
        for width in SOURCE_WIDTHS:
            width_offset = width - SOURCE_WIDTHS[0]
            outcome = OUTCOMES[(outcome_block + width_offset + epoch) % len(OUTCOMES)]
            bucket_lane = (lane + width_offset + epoch) % OUTCOME_RECORDS_PER_WIDTH
            selected.append(buckets[(width, outcome)][bucket_lane])
        if len(selected) != RECORDS_PER_STEP or len(set(selected)) != RECORDS_PER_STEP:
            raise RuntimeError("joint candidate schedule did not select seven records")
        if [self.records[index].source_width for index in selected] != list(SOURCE_WIDTHS):
            raise RuntimeError("joint candidate schedule is not one record per width")
        return tuple(selected)

    def validate_fit_schedule(self) -> None:
        self._validated_fit_buckets()
        for epoch in range(2):
            selected = [
                index
                for step in range(
                    epoch * STEPS_PER_EPOCH + 1,
                    (epoch + 1) * STEPS_PER_EPOCH + 1,
                )
                for index in self.record_indices_for_step(step)
            ]
            if len(selected) != len(self.records) or set(selected) != set(
                range(len(self.records))
            ):
                raise RuntimeError("joint candidate schedule does not cover one exact epoch")

    def batch(
        self, record_indices: Sequence[int], *, device: torch.device
    ) -> JointCandidateMarginBatch:
        if not record_indices:
            raise ValueError("joint candidate record batch is empty")
        if len(set(record_indices)) != len(record_indices):
            raise ValueError("joint candidate record batch repeats a record")
        selected = [self.encoded[index] for index in record_indices]
        flat_groups = [group for record in selected for group in record.groups]
        if not flat_groups:
            raise RuntimeError("joint candidate batch contains no relation groups")
        width = 1 + max(len(group.token_ids) for group in flat_groups)
        input_ids = torch.full(
            (len(flat_groups), width), EOS_TOKEN_ID, dtype=torch.long
        )
        terminal_indices = torch.empty(len(flat_groups), dtype=torch.long)
        group_labels = torch.empty(len(flat_groups), dtype=torch.float32)
        record_slices: list[tuple[int, int]] = []
        group_identities: list[tuple[str, str]] = []
        cursor = 0
        for record in selected:
            start = cursor
            for group in record.groups:
                input_ids[cursor, 0] = BOS_TOKEN_ID
                input_ids[cursor, 1 : 1 + len(group.token_ids)] = torch.tensor(
                    group.token_ids, dtype=torch.long
                )
                terminal_indices[cursor] = group.terminal_index
                group_labels[cursor] = float(group.relation_label)
                group_identities.append((record.record_id, group.relation_group_cid))
                cursor += 1
            record_slices.append((start, cursor))
        return JointCandidateMarginBatch(
            input_ids=input_ids.to(device),
            terminal_indices=terminal_indices.to(device),
            group_labels=group_labels.to(device),
            record_slices=tuple(record_slices),
            record_indices=tuple(int(index) for index in record_indices),
            group_identities=tuple(group_identities),
        )


def structured_margin_per_record(
    scores: Tensor,
    group_labels: Tensor,
    record_slices: Sequence[tuple[int, int]],
    *,
    margin: float = MARGIN,
) -> Tensor:
    """Return one threshold-aligned hinge loss for every complete record."""
    if scores.ndim != 1 or group_labels.shape != scores.shape:
        raise ValueError("joint candidate scores and labels must be matching vectors")
    if group_labels.dtype not in (torch.float16, torch.float32, torch.float64):
        raise ValueError("joint candidate labels must be floating point")
    if bool(((group_labels != 0.0) & (group_labels != 1.0)).any()):
        raise ValueError("joint candidate labels must be binary")
    if not math.isfinite(margin) or margin != MARGIN:
        raise ValueError("joint candidate margin is frozen at 1.0")
    cursor = 0
    losses: list[Tensor] = []
    for start, end in record_slices:
        if start != cursor or not start < end <= scores.numel():
            raise ValueError("joint candidate record slices are not contiguous and complete")
        record_scores = scores[start:end].float()
        record_labels = group_labels[start:end]
        positive = record_scores[record_labels == 1.0]
        negative = record_scores[record_labels == 0.0]
        loss = record_scores.sum() * 0.0
        if positive.numel():
            loss = loss + F.relu(record_scores.new_tensor(margin) - positive.min())
        if negative.numel():
            loss = loss + F.relu(record_scores.new_tensor(margin) + negative.max())
        losses.append(loss)
        cursor = end
    if not losses or cursor != scores.numel():
        raise ValueError("joint candidate record slices do not cover all group scores")
    return torch.stack(losses)


def joint_candidate_structured_margin_loss(
    scores: Tensor,
    group_labels: Tensor,
    record_slices: Sequence[tuple[int, int]],
) -> Tensor:
    return structured_margin_per_record(scores, group_labels, record_slices).mean()


@torch.no_grad()
def evaluate_joint_candidate_margin_adapter(
    adapter: R4JointCandidateMarginAdapter,
    dataset: EncodedJointCandidateMarginDataset,
    *,
    device: torch.device,
    record_batch_size: int = RECORDS_PER_STEP,
) -> dict[str, Any]:
    if record_batch_size < 1:
        raise ValueError("joint candidate evaluation batch size must be positive")
    adapter.eval()
    evaluations: list[dict[str, Any]] = []
    total_margin = 0.0
    for base in range(0, len(dataset.encoded), record_batch_size):
        record_indices = tuple(
            range(base, min(base + record_batch_size, len(dataset.encoded)))
        )
        batch = dataset.batch(record_indices, device=device)
        scores = adapter(batch.input_ids, batch.terminal_indices)
        if scores.shape != batch.group_labels.shape:
            raise RuntimeError("joint candidate adapter score cardinality differs")
        losses = structured_margin_per_record(
            scores, batch.group_labels, batch.record_slices
        ).detach().cpu()
        cpu_scores = scores.detach().cpu()
        for lane, (start, end) in enumerate(batch.record_slices):
            record = dataset.encoded[record_indices[lane]]
            if end - start != len(record.groups):
                raise RuntimeError("joint candidate score slice differs from record groups")
            group_scores = []
            for group, score in zip(record.groups, cpu_scores[start:end]):
                group_scores.append(
                    {
                        "relation_group_cid": group.relation_group_cid,
                        "text": group.text,
                        "relation_label": group.relation_label,
                        "occurrence_indices": list(group.occurrence_indices),
                        "score": float(score),
                    }
                )
            record_margin = float(losses[lane])
            total_margin += record_margin
            evaluations.append(
                {
                    "record_id": record.record_id,
                    "source_width": record.source_width,
                    "target_outcome": record.target_outcome,
                    "structured_margin": record_margin,
                    "group_scores": group_scores,
                }
            )
    return {
        "records": len(evaluations),
        "groups": sum(len(record["group_scores"]) for record in evaluations),
        "mean_structured_margin": total_margin / len(evaluations),
        "record_evaluations": evaluations,
    }


def fit_joint_candidate_margin_adapter(
    adapter: R4JointCandidateMarginAdapter,
    dataset: EncodedJointCandidateMarginDataset,
    *,
    config: JointCandidateMarginFitConfig = JointCandidateMarginFitConfig(),
) -> dict[str, Any]:
    """Run the sole 270-update MPS fit without opening sealed/product data."""
    config.validate()
    if config.seed != adapter.config.initialization_seed:
        raise ValueError("joint candidate fit seed differs from adapter initialization")
    dataset.validate_fit_schedule()
    device = require_mps(config.seed)
    adapter.to(device)
    adapter.train()
    parameters = list(adapter.adapter_parameters())
    if sum(parameter.numel() for parameter in parameters) != TRAINABLE_PARAMETER_COUNT:
        raise RuntimeError("joint candidate optimizer parameter census differs")
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
    for step in range(1, config.optimizer_steps + 1):
        record_indices = dataset.record_indices_for_step(step)
        batch = dataset.batch(record_indices, device=device)
        optimizer.zero_grad(set_to_none=True)
        scores = adapter(batch.input_ids, batch.terminal_indices)
        loss = joint_candidate_structured_margin_loss(
            scores, batch.group_labels, batch.record_slices
        )
        loss.backward()
        torch.nn.utils.clip_grad_norm_(parameters, config.gradient_clip)
        optimizer.step()
        final_loss = float(loss.detach().cpu())
        if initial_loss is None:
            initial_loss = final_loss
        if not math.isfinite(final_loss):
            raise RuntimeError("joint candidate adapter produced a nonfinite loss")
        if step == config.eta_probe_step:
            if hasattr(torch, "mps"):
                torch.mps.synchronize()
            elapsed = time.monotonic() - started
            projected_seconds = elapsed * config.optimizer_steps / step
            print(
                f"joint_candidate_margin_eta_probe_step={step} "
                f"elapsed_seconds={elapsed:.3f} "
                f"projected_seconds={projected_seconds:.3f} "
                f"ceiling_seconds={config.wall_ceiling_seconds:.3f}",
                flush=True,
            )
            if projected_seconds > config.wall_ceiling_seconds:
                raise JointCandidateMarginWallBudgetExceeded(
                    step=step,
                    elapsed_seconds=elapsed,
                    projected_seconds_at_eta_probe=projected_seconds,
                    wall_ceiling_seconds=config.wall_ceiling_seconds,
                )
        elapsed = time.monotonic() - started
        if elapsed > config.wall_ceiling_seconds:
            raise JointCandidateMarginWallBudgetExceeded(
                step=step,
                elapsed_seconds=elapsed,
                projected_seconds_at_eta_probe=projected_seconds,
                wall_ceiling_seconds=config.wall_ceiling_seconds,
            )
        if step % config.progress_interval == 0 or step == config.optimizer_steps:
            print(
                f"joint_candidate_margin_step={step}/{config.optimizer_steps} "
                f"loss={final_loss:.6f}",
                flush=True,
            )
    if hasattr(torch, "mps"):
        torch.mps.synchronize()
    elapsed_seconds = time.monotonic() - started
    if elapsed_seconds > config.wall_ceiling_seconds:
        raise JointCandidateMarginWallBudgetExceeded(
            step=config.optimizer_steps,
            elapsed_seconds=elapsed_seconds,
            projected_seconds_at_eta_probe=projected_seconds,
            wall_ceiling_seconds=config.wall_ceiling_seconds,
        )
    return {
        "optimizer_steps": config.optimizer_steps,
        "records_per_step": config.records_per_step,
        "initial_structured_margin": initial_loss,
        "final_structured_margin": final_loss,
        "elapsed_seconds": elapsed_seconds,
        "eta_probe_step": config.eta_probe_step,
        "projected_seconds_at_eta_probe": projected_seconds,
        "wall_ceiling_seconds": config.wall_ceiling_seconds,
        "trainable_parameter_count": TRAINABLE_PARAMETER_COUNT,
        "delta_audit": adapter.delta_audit(),
    }
