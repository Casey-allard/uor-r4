"""C1-SB3 attended-relation adapter over the frozen #1017 model.

The adapter changes representation rather than adding another classifier.  It
places one rank-eight LoRA delta on Q, K, V, and O in each of the six existing
causal-softmax layers.  The only score is the difference between two rows of
the model's frozen tied embedding/output table at the final ``Supported:``
position.  A trained adapter can be merged into an ordinary Hugging Face
checkpoint, so the existing Rust R4/Spin executor needs no LoRA runtime path.
"""

from __future__ import annotations

import math
import struct
import time
from collections.abc import Iterable, Mapping, Sequence
from dataclasses import asdict, dataclass, field
from pathlib import Path
from typing import Any

import torch
from blake3 import blake3
from tokenizers import Tokenizer
from torch import Tensor, nn
from torch.nn import functional as F

from .constants import BOS_TOKEN_ID, EOS_TOKEN_ID, FROZEN_MODEL_CONFIG
from .export import export_hugging_face_snapshot
from .model import R4SoftmaxForCausalLM, expected_hf_tensor_names
from .provenance import cid_bytes
from .train import require_mps


ISSUE = 954
POLICY = "R4AttendedRelationAdapterV1"
ARTIFACT_SCHEMA = "uor-r4.attended-relation-adapter/1"
REPRESENTATION_UPDATE = "lora_qkvo_all_layers"
RELATION_INPUT_TEMPLATE = (
    "Evidence:\n<span>\nQuestion:\n<question>\nSupported:"
)
YES_TOKEN_ID = 1_771
NO_TOKEN_ID = 542
YES_TOKEN_TEXT = " yes"
NO_TOKEN_TEXT = " no"
TARGET_PROJECTIONS = ("q_proj", "k_proj", "v_proj", "o_proj")
TARGET_LAYER_INDICES = tuple(range(FROZEN_MODEL_CONFIG.num_hidden_layers))
LORA_RANK = 8
LORA_ALPHA = 8
LORA_DROPOUT = 0.0
LORA_PARAMETERS_PER_PROJECTION = (
    2 * FROZEN_MODEL_CONFIG.hidden_size * LORA_RANK
)
TRAINABLE_PARAMETER_COUNT = (
    len(TARGET_LAYER_INDICES)
    * len(TARGET_PROJECTIONS)
    * LORA_PARAMETERS_PER_PROJECTION
)
TRAIN_BATCH_SIZE = 64
INITIALIZATION_SEED = 9_543


@dataclass(frozen=True, slots=True)
class AttendedRelationAdapterConfig:
    """The fixed no-sweep representation contract."""

    rank: int = LORA_RANK
    alpha: int = LORA_ALPHA
    dropout: float = LORA_DROPOUT
    target_layer_indices: tuple[int, ...] = TARGET_LAYER_INDICES
    target_projections: tuple[str, ...] = TARGET_PROJECTIONS
    yes_token_id: int = YES_TOKEN_ID
    no_token_id: int = NO_TOKEN_ID
    relation_input_template: str = RELATION_INPUT_TEMPLATE
    initialization_seed: int = INITIALIZATION_SEED

    def validate(self) -> None:
        if self != AttendedRelationAdapterConfig():
            raise ValueError("C1-SB3 exposes one fixed all-layer adapter, not a sweep")
        if self.rank != 8 or self.alpha != 8 or self.dropout != 0.0:
            raise AssertionError("C1-SB3 LoRA rank, alpha, or dropout drifted")
        if self.target_layer_indices != tuple(range(6)):
            raise AssertionError("C1-SB3 must adapt all six attention layers")
        if self.target_projections != ("q_proj", "k_proj", "v_proj", "o_proj"):
            raise AssertionError("C1-SB3 must adapt exactly Q, K, V, and O")
        if self.yes_token_id == self.no_token_id:
            raise AssertionError("C1-SB3 verbalizer token IDs must differ")
        if self.initialization_seed != INITIALIZATION_SEED:
            raise AssertionError("C1-SB3 initialization seed drifted")

    @property
    def trainable_parameter_count(self) -> int:
        self.validate()
        return TRAINABLE_PARAMETER_COUNT

    def as_contract(self) -> dict[str, Any]:
        self.validate()
        value = asdict(self)
        value["target_layer_indices"] = list(self.target_layer_indices)
        value["target_projections"] = list(self.target_projections)
        value["trainable_parameter_count"] = self.trainable_parameter_count
        value["representation_update"] = REPRESENTATION_UPDATE
        value["score"] = "tied-logit[1771] - tied-logit[542] at final colon"
        value["decision"] = "supported iff score > +0.0"
        value["merge"] = "W_merged = W_base + (alpha / rank) * B @ A"
        value["initialization"] = (
            "Xavier-uniform A from an isolated CPU generator seeded "
            "initialization_seed + stable layer/projection ordinal; zero B"
        )
        return value


@dataclass(frozen=True, slots=True)
class RelationExample:
    """One candidate label; all record-specific metadata remains opaque."""

    record_id: str
    candidate_index: int
    relation_input: str
    relation_input_cid: str
    relation_label: int
    metadata: Mapping[str, Any] = field(default_factory=dict, repr=False, compare=False)


@dataclass(frozen=True, slots=True)
class EncodedRelationExample:
    record_id: str
    candidate_index: int
    token_ids: tuple[int, ...]
    relation_label: int

    @property
    def terminal_index(self) -> int:
        # BOS occupies index zero.  The final content token is the colon in
        # ``Supported:``, so its index equals the content-token count.
        return len(self.token_ids)


@dataclass(slots=True)
class RelationTokenBatch:
    input_ids: Tensor
    terminal_indices: Tensor
    labels: Tensor
    example_indices: tuple[int, ...]


@dataclass(frozen=True, slots=True)
class AdapterFitConfig:
    """Optimizer values are supplied by the independently frozen run contract."""

    seed: int
    optimizer_steps: int
    batch_size: int
    learning_rate: float
    adam_beta1: float = 0.9
    adam_beta2: float = 0.999
    adam_epsilon: float = 1e-8
    weight_decay: float = 0.0
    gradient_clip: float = 1.0
    progress_interval: int = 16
    eta_probe_step: int = 8
    wall_ceiling_seconds: float = 600.0

    def validate(self) -> None:
        if self.seed < 0 or self.optimizer_steps < 1 or self.batch_size < 1:
            raise ValueError("adapter seed, optimizer steps, and batch size are invalid")
        if self.batch_size != TRAIN_BATCH_SIZE:
            raise ValueError("C1-SB3 training batches are frozen at 64 candidates")
        if not math.isfinite(self.learning_rate) or self.learning_rate <= 0.0:
            raise ValueError("adapter learning rate must be finite and positive")
        if not 0.0 <= self.weight_decay or not math.isfinite(self.weight_decay):
            raise ValueError("adapter weight decay must be finite and nonnegative")
        if not math.isfinite(self.gradient_clip) or self.gradient_clip <= 0.0:
            raise ValueError("adapter gradient clip must be finite and positive")
        if self.progress_interval < 1:
            raise ValueError("adapter progress interval must be positive")
        if not 1 <= self.eta_probe_step <= self.optimizer_steps:
            raise ValueError("adapter ETA probe step is outside the run")
        if (
            not math.isfinite(self.wall_ceiling_seconds)
            or self.wall_ceiling_seconds <= 0.0
        ):
            raise ValueError("adapter wall ceiling must be finite and positive")


class AttendedRelationWallBudgetExceeded(RuntimeError):
    """Decision-bearing wall-budget stop with the measurements retained."""

    def __init__(
        self,
        *,
        step: int,
        elapsed_seconds: float,
        projected_seconds_at_eta_probe: float | None,
        wall_ceiling_seconds: float,
    ) -> None:
        super().__init__("UNAVAILABLE_ATTENDED_RELATION_ADAPTER_WALL_BUDGET")
        self.step = step
        self.elapsed_seconds = elapsed_seconds
        self.projected_seconds_at_eta_probe = projected_seconds_at_eta_probe
        self.wall_ceiling_seconds = wall_ceiling_seconds

    def as_result(self) -> dict[str, Any]:
        return {
            "status": "UNAVAILABLE_ATTENDED_RELATION_ADAPTER_WALL_BUDGET",
            "stopped_after_step": self.step,
            "elapsed_seconds": self.elapsed_seconds,
            "projected_seconds_at_eta_probe": self.projected_seconds_at_eta_probe,
            "wall_ceiling_seconds": self.wall_ceiling_seconds,
        }


class LoRALinear(nn.Module):
    """A frozen bias-free linear projection plus one mergeable LoRA delta."""

    def __init__(
        self,
        base: nn.Linear,
        *,
        rank: int = LORA_RANK,
        alpha: int = LORA_ALPHA,
        dropout: float = LORA_DROPOUT,
        initialization_seed: int = INITIALIZATION_SEED,
    ) -> None:
        super().__init__()
        if base.bias is not None:
            raise ValueError("C1-SB3 attention projections must remain bias-free")
        if rank != LORA_RANK or alpha != LORA_ALPHA or dropout != LORA_DROPOUT:
            raise ValueError("C1-SB3 LoRA shape is frozen at rank-8 alpha-8 dropout-0")
        if base.weight.device.type != "cpu":
            raise ValueError("C1-SB3 LoRA must be initialized on CPU before MPS transfer")
        if initialization_seed < 0:
            raise ValueError("C1-SB3 LoRA initialization seed must be nonnegative")
        base.requires_grad_(False)
        self.base = base
        self.rank = rank
        self.alpha = alpha
        self.scaling = float(alpha) / float(rank)
        self.initialization_seed = initialization_seed
        self.lora_a = nn.Parameter(
            torch.empty(
                rank,
                base.in_features,
                dtype=base.weight.dtype,
                device=base.weight.device,
            )
        )
        self.lora_b = nn.Parameter(
            torch.zeros(
                base.out_features,
                rank,
                dtype=base.weight.dtype,
                device=base.weight.device,
            )
        )
        generator = torch.Generator(device="cpu")
        generator.manual_seed(initialization_seed)
        nn.init.xavier_uniform_(self.lora_a, generator=generator)

    def forward(self, values: Tensor) -> Tensor:
        delta = F.linear(F.linear(values, self.lora_a), self.lora_b)
        return self.base(values) + delta * self.scaling

    def merged_weight(self) -> Tensor:
        delta = torch.matmul(self.lora_b.float(), self.lora_a.float())
        merged = self.base.weight.detach().float() + delta * self.scaling
        return merged.to(device="cpu", dtype=torch.float32).contiguous()


class R4AttendedRelationAdapter(nn.Module):
    """All-layer LoRA representation with a fixed tied-token verbalizer."""

    def __init__(
        self,
        model: R4SoftmaxForCausalLM,
        config: AttendedRelationAdapterConfig = AttendedRelationAdapterConfig(),
    ) -> None:
        super().__init__()
        config.validate()
        if model.config != FROZEN_MODEL_CONFIG:
            raise ValueError("C1-SB3 requires the exact six-layer #1017 architecture")
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
                f"C1-SB3 has {observed} trainable parameters, expected "
                f"{config.trainable_parameter_count}"
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
            raise ValueError("adapter input_ids must have shape [batch, time]")
        hidden = self.model.model(input_ids)
        batch, time, width = hidden.shape
        if width != FROZEN_MODEL_CONFIG.hidden_size:
            raise RuntimeError("adapter hidden width differs from #1017")
        if terminal_indices is None:
            terminal_indices = torch.full(
                (batch,), time - 1, dtype=torch.long, device=hidden.device
            )
        if terminal_indices.shape != (batch,) or terminal_indices.dtype != torch.long:
            raise ValueError("terminal_indices must be int64 with shape [batch]")
        if bool(((terminal_indices < 0) | (terminal_indices >= time)).any()):
            raise ValueError("adapter terminal index is outside the encoded sequence")
        terminal = hidden[torch.arange(batch, device=hidden.device), terminal_indices]
        return tied_relation_scores(terminal, self.model.model.embed_tokens.weight)

    def merged_state_dict(self) -> dict[str, Tensor]:
        """Return only the ordinary HF tensor names, with every LoRA delta folded."""
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
                raise RuntimeError(f"wrapped model is missing ordinary tensor {name}") from error
            merged[name] = value.detach().to(device="cpu", dtype=torch.float32).contiguous()
        if set(merged) != expected or len(self._adapters) != 24:
            raise RuntimeError("C1-SB3 merge did not cover exactly 24 attention tensors")
        return merged

    def delta_audit(self) -> dict[str, Any]:
        """Report every and only mergeable attention delta before export."""
        tensors: list[dict[str, Any]] = []
        for name in sorted(self._adapters):
            adapter = self._adapters[name]
            delta = (
                torch.matmul(adapter.lora_b.detach().float(), adapter.lora_a.detach().float())
                * adapter.scaling
            )
            finite = bool(torch.isfinite(delta).all())
            tensors.append(
                {
                    "name": name,
                    "parameter_count": adapter.lora_a.numel()
                    + adapter.lora_b.numel(),
                    "nonzero": int(torch.count_nonzero(delta).item()),
                    "l2_norm": float(torch.linalg.vector_norm(delta).cpu()),
                    "finite": finite,
                    "initialization_seed": adapter.initialization_seed,
                }
            )
        if len(tensors) != 24 or any(not tensor["finite"] for tensor in tensors):
            raise RuntimeError("C1-SB3 delta census is incomplete or nonfinite")
        return {
            "target_tensor_count": len(tensors),
            "trainable_parameter_count": sum(
                int(tensor["parameter_count"]) for tensor in tensors
            ),
            "tensors": tensors,
        }

    def merged_model(self) -> R4SoftmaxForCausalLM:
        """Build a clean model with no wrapper or adapter tensors."""
        clean = R4SoftmaxForCausalLM(self.model.config)
        clean.load_state_dict(self.merged_state_dict(), strict=True)
        clean.eval()
        return clean


def tied_relation_scores(final_hidden: Tensor, embedding_weight: Tensor) -> Tensor:
    """Return `yes - no` from the frozen tied output rows; no head is learned."""
    if final_hidden.ndim != 2 or final_hidden.shape[-1] != FROZEN_MODEL_CONFIG.hidden_size:
        raise ValueError("final_hidden must have shape [batch, 288]")
    expected_embedding_shape = (
        FROZEN_MODEL_CONFIG.vocab_size,
        FROZEN_MODEL_CONFIG.hidden_size,
    )
    if tuple(embedding_weight.shape) != expected_embedding_shape:
        raise ValueError(
            f"embedding weight must have shape {expected_embedding_shape}"
        )
    token_ids = torch.tensor(
        [YES_TOKEN_ID, NO_TOKEN_ID], dtype=torch.long, device=embedding_weight.device
    )
    verbalizer_rows = embedding_weight.index_select(0, token_ids)
    logits = F.linear(final_hidden.float(), verbalizer_rows.float())
    return logits[:, 0] - logits[:, 1]


def relation_binary_loss(scores: Tensor, labels: Tensor) -> Tensor:
    if scores.ndim != 1 or labels.shape != scores.shape:
        raise ValueError("relation scores and labels must be matching vectors")
    if labels.dtype not in (torch.float16, torch.float32, torch.float64):
        raise ValueError("relation labels must be floating point")
    if bool(((labels != 0.0) & (labels != 1.0)).any()):
        raise ValueError("relation labels must be binary")
    return F.binary_cross_entropy_with_logits(scores.float(), labels.float())


def _validate_relation_input(value: str) -> None:
    if not value.startswith("Evidence:\n"):
        raise ValueError("relation input does not begin with exact Evidence header")
    if value.count("\nQuestion:\n") != 1:
        raise ValueError("relation input does not contain one exact Question header")
    if not value.endswith("\nSupported:") or value.endswith("\nSupported:\n"):
        raise ValueError("relation input must end at exact Supported: with no newline")
    evidence, question_and_suffix = value[len("Evidence:\n") :].split(
        "\nQuestion:\n", 1
    )
    question = question_and_suffix[: -len("\nSupported:")]
    if not evidence or evidence != evidence.strip() or not question.endswith("?"):
        raise ValueError("relation evidence or question is not canonical")


def relation_examples_from_records(
    records: Sequence[Mapping[str, Any]],
) -> list[RelationExample]:
    """Flatten either candidate rows or SB2-shaped records without owning their schema."""
    examples: list[RelationExample] = []
    for record_offset, record in enumerate(records):
        record_id = record.get("record_id", record.get("record_cid"))
        if not isinstance(record_id, str) or not record_id:
            raise ValueError(f"relation record {record_offset} has no record identity")
        spans = record.get("sentence_spans")
        rows: Sequence[Mapping[str, Any]]
        if spans is None:
            rows = [record]
        elif isinstance(spans, Sequence) and not isinstance(spans, (str, bytes)):
            rows = spans
        else:
            raise ValueError(f"relation record {record_id} has invalid sentence_spans")

        question = record.get("question")
        for row_offset, row in enumerate(rows):
            if not isinstance(row, Mapping):
                raise ValueError(
                    f"relation candidate {record_id}/{row_offset} is not a mapping"
                )
            relation_input = row.get("relation_input")
            label = row.get("relation_label")
            if not isinstance(relation_input, str):
                raise ValueError(f"relation candidate {record_id}/{row_offset} has no input")
            _validate_relation_input(relation_input)
            if label not in (0, 1):
                raise ValueError(f"relation candidate {record_id}/{row_offset} is not binary")
            candidate_index = row.get("candidate_index", row_offset)
            if not isinstance(candidate_index, int) or candidate_index < 0:
                raise ValueError(f"relation candidate {record_id}/{row_offset} has bad index")
            relation_input_cid = cid_bytes(relation_input.encode("utf-8"))
            embedded_cid = row.get("relation_input_cid")
            if embedded_cid is not None and embedded_cid != relation_input_cid:
                raise ValueError(
                    f"relation candidate {record_id}/{candidate_index} input CID differs"
                )
            span_text = row.get("text")
            if isinstance(span_text, str) and isinstance(question, str):
                expected = (
                    f"Evidence:\n{span_text}\nQuestion:\n{question}\nSupported:"
                )
                if relation_input != expected:
                    raise ValueError(
                        f"relation candidate {record_id}/{candidate_index} renderer differs"
                    )
            examples.append(
                RelationExample(
                    record_id=record_id,
                    candidate_index=candidate_index,
                    relation_input=relation_input,
                    relation_input_cid=relation_input_cid,
                    relation_label=int(label),
                    metadata={
                        key: row[key]
                        for key in ("text_cid", "role", "relation_group_cid")
                        if key in row
                    },
                )
            )
    if not examples:
        raise ValueError("relation example population is empty")
    return examples


def validate_tokenizer_contract(tokenizer: Tokenizer) -> None:
    for token_id, text in (
        (YES_TOKEN_ID, YES_TOKEN_TEXT),
        (NO_TOKEN_ID, NO_TOKEN_TEXT),
    ):
        if tokenizer.decode([token_id], skip_special_tokens=False) != text:
            raise ValueError(
                f"fixed relation verbalizer token {token_id} does not decode {text!r}"
            )
        if tokenizer.encode(text, add_special_tokens=False).ids != [token_id]:
            raise ValueError(f"fixed relation verbalizer {text!r} is not one token")


class EncodedRelationDataset:
    """Immutable tokenizer-bound relation examples with deterministic batches."""

    def __init__(self, examples: Sequence[RelationExample], tokenizer: Tokenizer) -> None:
        if not examples:
            raise ValueError("relation dataset is empty")
        validate_tokenizer_contract(tokenizer)
        self.examples = sorted(
            examples, key=lambda example: (example.record_id, example.candidate_index)
        )
        self.encoded: list[EncodedRelationExample] = []
        self._epoch_orders: dict[tuple[int, int], tuple[int, ...]] = {}
        for example in self.examples:
            token_ids = tokenizer.encode(
                example.relation_input, add_special_tokens=False
            ).ids
            if not token_ids:
                raise ValueError("relation input encoded to zero tokens")
            if len(token_ids) + 1 > FROZEN_MODEL_CONFIG.max_position_embeddings:
                raise ValueError("relation input exceeds the frozen #1017 context")
            if tokenizer.decode([token_ids[-1]], skip_special_tokens=False) != ":":
                raise ValueError("relation input does not end in a standalone colon token")
            self.encoded.append(
                EncodedRelationExample(
                    record_id=example.record_id,
                    candidate_index=example.candidate_index,
                    token_ids=tuple(int(token_id) for token_id in token_ids),
                    relation_label=example.relation_label,
                )
            )

    def deterministic_indices(
        self, *, seed: int, step: int, batch_size: int
    ) -> tuple[int, ...]:
        if seed < 0 or step < 1 or batch_size < 1:
            raise ValueError("deterministic batch coordinates are invalid")
        selected: list[int] = []
        first_global_position = (step - 1) * batch_size
        for lane in range(batch_size):
            global_position = first_global_position + lane
            epoch, position = divmod(global_position, len(self.encoded))
            cache_key = (seed, epoch)
            order = self._epoch_orders.get(cache_key)
            if order is None:
                order = tuple(
                    sorted(
                        range(len(self.encoded)),
                        key=lambda index: blake3(
                            struct.pack(">QQQ", seed, epoch, index)
                        ).digest(),
                    )
                )
                self._epoch_orders[cache_key] = order
            selected.append(order[position])
        return tuple(selected)

    def batch(
        self, indices: Sequence[int], *, device: torch.device
    ) -> RelationTokenBatch:
        if not indices:
            raise ValueError("relation batch is empty")
        selected = [self.encoded[index] for index in indices]
        width = 1 + max(len(example.token_ids) for example in selected)
        input_ids = torch.full(
            (len(selected), width), EOS_TOKEN_ID, dtype=torch.long
        )
        terminal_indices = torch.empty(len(selected), dtype=torch.long)
        labels = torch.empty(len(selected), dtype=torch.float32)
        for lane, example in enumerate(selected):
            input_ids[lane, 0] = BOS_TOKEN_ID
            input_ids[lane, 1 : 1 + len(example.token_ids)] = torch.tensor(
                example.token_ids, dtype=torch.long
            )
            terminal_indices[lane] = example.terminal_index
            labels[lane] = float(example.relation_label)
        return RelationTokenBatch(
            input_ids=input_ids.to(device),
            terminal_indices=terminal_indices.to(device),
            labels=labels.to(device),
            example_indices=tuple(int(index) for index in indices),
        )


@torch.no_grad()
def evaluate_attended_relation_adapter(
    adapter: R4AttendedRelationAdapter,
    dataset: EncodedRelationDataset,
    *,
    device: torch.device,
    batch_size: int,
) -> dict[str, Any]:
    if batch_size < 1:
        raise ValueError("relation evaluation batch size must be positive")
    adapter.eval()
    scores: list[float] = []
    labels: list[int] = []
    for base in range(0, len(dataset.encoded), batch_size):
        indices = tuple(range(base, min(base + batch_size, len(dataset.encoded))))
        batch = dataset.batch(indices, device=device)
        observed = adapter(batch.input_ids, batch.terminal_indices)
        scores.extend(float(value) for value in observed.detach().cpu().tolist())
        labels.extend(int(value) for value in batch.labels.detach().cpu().tolist())
    correct = sum(int((score > 0.0) == bool(label)) for score, label in zip(scores, labels))
    mean_loss = float(
        relation_binary_loss(
            torch.tensor(scores, dtype=torch.float32),
            torch.tensor(labels, dtype=torch.float32),
        )
    )
    return {
        "examples": len(labels),
        "correct": correct,
        "accuracy": correct / len(labels),
        "mean_binary_cross_entropy": mean_loss,
        "scores": scores,
        "labels": labels,
    }


def fit_attended_relation_adapter(
    adapter: R4AttendedRelationAdapter,
    dataset: EncodedRelationDataset,
    *,
    config: AdapterFitConfig,
) -> dict[str, Any]:
    """Run one caller-frozen MPS fit; this function never opens sealed/product data."""
    config.validate()
    if config.seed != adapter.config.initialization_seed:
        raise ValueError("adapter fit seed differs from the frozen initialization seed")
    device = require_mps(config.seed)
    adapter.to(device)
    adapter.train()
    parameters = list(adapter.adapter_parameters())
    if sum(parameter.numel() for parameter in parameters) != TRAINABLE_PARAMETER_COUNT:
        raise RuntimeError("adapter optimizer parameter census differs")
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
        indices = dataset.deterministic_indices(
            seed=config.seed, step=step, batch_size=config.batch_size
        )
        batch = dataset.batch(indices, device=device)
        optimizer.zero_grad(set_to_none=True)
        scores = adapter(batch.input_ids, batch.terminal_indices)
        loss = relation_binary_loss(scores, batch.labels)
        loss.backward()
        torch.nn.utils.clip_grad_norm_(parameters, config.gradient_clip)
        optimizer.step()
        final_loss = float(loss.detach().cpu())
        if initial_loss is None:
            initial_loss = final_loss
        if not math.isfinite(final_loss):
            raise RuntimeError("attended-relation adapter produced a nonfinite loss")
        if step == config.eta_probe_step:
            if hasattr(torch, "mps"):
                torch.mps.synchronize()
            elapsed = time.monotonic() - started
            projected_seconds = elapsed * config.optimizer_steps / step
            print(
                f"relation_adapter_eta_probe_step={step} "
                f"elapsed_seconds={elapsed:.3f} "
                f"projected_seconds={projected_seconds:.3f} "
                f"ceiling_seconds={config.wall_ceiling_seconds:.3f}",
                flush=True,
            )
            if projected_seconds > config.wall_ceiling_seconds:
                raise AttendedRelationWallBudgetExceeded(
                    step=step,
                    elapsed_seconds=elapsed,
                    projected_seconds_at_eta_probe=projected_seconds,
                    wall_ceiling_seconds=config.wall_ceiling_seconds,
                )
        elapsed = time.monotonic() - started
        if elapsed > config.wall_ceiling_seconds:
            raise AttendedRelationWallBudgetExceeded(
                step=step,
                elapsed_seconds=elapsed,
                projected_seconds_at_eta_probe=projected_seconds,
                wall_ceiling_seconds=config.wall_ceiling_seconds,
            )
        if step % config.progress_interval == 0 or step == config.optimizer_steps:
            print(
                f"relation_adapter_step={step}/{config.optimizer_steps} "
                f"loss={final_loss:.6f}",
                flush=True,
            )
    if hasattr(torch, "mps"):
        torch.mps.synchronize()
    return {
        "optimizer_steps": config.optimizer_steps,
        "batch_size": config.batch_size,
        "initial_binary_cross_entropy": initial_loss,
        "final_binary_cross_entropy": final_loss,
        "elapsed_seconds": time.monotonic() - started,
        "eta_probe_step": config.eta_probe_step,
        "projected_seconds_at_eta_probe": projected_seconds,
        "wall_ceiling_seconds": config.wall_ceiling_seconds,
        "trainable_parameter_count": TRAINABLE_PARAMETER_COUNT,
        "delta_audit": adapter.delta_audit(),
    }


def export_merged_attended_relation_checkpoint(
    adapter: R4AttendedRelationAdapter,
    *,
    output_dir: Path,
    tokenizer_path: Path,
    training_result: dict[str, Any],
    dataset_manifest_cid: str,
    training_view_manifest_cid: str,
    split_policy_cid: str,
    run_contract_cid: str,
    selected_checkpoint_cid: str | None,
    selected_checkpoint_identity: str | None = None,
) -> dict[str, Any]:
    """Merge LoRA and reuse the standard checkpoint format accepted by Rust."""
    clean = adapter.merged_model()
    return export_hugging_face_snapshot(
        clean,
        output_dir=output_dir,
        tokenizer_path=tokenizer_path,
        training_result=training_result,
        dataset_manifest_cid=dataset_manifest_cid,
        training_view_manifest_cid=training_view_manifest_cid,
        split_policy_cid=split_policy_cid,
        run_contract_cid=run_contract_cid,
        selected_checkpoint_cid=selected_checkpoint_cid,
        selected_checkpoint_identity=selected_checkpoint_identity,
    )
