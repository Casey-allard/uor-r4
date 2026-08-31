"""Offline C1-SB2 source-relative relation-head qualification.

The module deliberately keeps the failed C1-SB1 cosine pointer immutable.  It
extracts one final #1017 residual for each exact evidence/question pair, fits
the frozen 288 -> 32 -> 1 probe, and emits a head only after every predeclared
gate has passed.
"""

from __future__ import annotations

import json
import math
import os
import struct
import time
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Any, Sequence

import torch
from blake3 import blake3
from safetensors.torch import load_file, save_file
from tokenizers import Tokenizer
from torch import Tensor, nn
from torch.nn import functional as F

from .constants import BOS_TOKEN_ID, EOS_TOKEN_ID, FROZEN_MODEL_CONFIG
from .model import R4SoftmaxForCausalLM
from .provenance import (
    atomic_write,
    atomic_write_json,
    canonical_json_bytes,
    cid_bytes,
    cid_file,
    trainer_implementation_contract,
    verify_bound_manifest,
    write_bound_manifest,
)
from .source_relation_data import (
    OUTCOMES,
    POLICY as DATA_POLICY,
    QUESTION_POLICY,
    RELATION_INPUT_POLICY,
    SENTENCE_POLICY,
    SOURCE_WIDTHS,
    build_relation_preflight,
    build_source_relation_population,
    product_probes,
    render_relation_input,
    shortcut_census,
)
from .train import require_mps


ISSUE = 954
POLICY = "R4SourceRelativeRelationHeadV1"
HEAD_SCHEMA = "uor-r4.source-relative-relation-head/1"
DATASET_MANIFEST_SCHEMA = "uor-r4.source-relative-relation-dataset-manifest/1"
FEATURE_INDEX_SCHEMA = "uor-r4.source-relative-relation-feature-index/1"
FEATURE_MANIFEST_SCHEMA = "uor-r4.source-relative-relation-feature-manifest/1"
RUN_SCHEMA = "uor-r4.source-relative-relation-run/1"
PREFLIGHT_RESULT_SCHEMA = "uor-r4.source-relative-relation-preflight-result/1"
PYTHON_SCORE_FIXTURE_SCHEMA = "uor-r4.source-relative-relation-python-score-fixture/1"
PARITY_ADMISSION_SCHEMA = "uor-r4.source-relative-relation-parity-admission/1"
TRAINING_RESULT_SCHEMA = "uor-r4.source-relative-relation-training-result/1"
FINAL_MANIFEST_SCHEMA = "uor-r4.source-relative-relation-final-manifest/1"
RUST_GROUNDED_REPORT_SCHEMA = "uor-r4.grounded-answer/3"
EXPECTED_WEIGHTS_CID = (
    "blake3:c5bf31aa97a567b3aaad4461ce2fac9cebc12b0a38becb6d02d21b43b493bf5d"
)
EXPECTED_TOKENIZER_CID = (
    "blake3:3f42bcfce7728512076549c63b88387e13c8156fe35c0f91d9b112439f3739cc"
)
RELATION_HIDDEN_SIZE = 32
TRAINABLE_PARAMETER_COUNT = 9_281
PARITY_MAX_ABSOLUTE_TOLERANCE = 0.01
RELATION_INPUT_TEMPLATE = "Evidence:\n<span>\nQuestion:\n<question>"
OUTCOME_TO_ID = {name: index for index, name in enumerate(OUTCOMES)}


@dataclass(frozen=True, slots=True)
class SourceRelationHeadConfig:
    """The sole no-sweep C1-SB2 work contract."""

    seed: int = 9_542
    feature_batch_size: int = 128
    preflight_optimizer_steps: int = 256
    optimizer_steps: int = 512
    batch_size: int = 126
    learning_rate: float = 0.003
    adam_beta1: float = 0.9
    adam_beta2: float = 0.999
    adam_epsilon: float = 1e-8
    progress_interval: int = 16
    required_per_class_accuracy: float = 0.95
    required_positive_relation_recall: float = 0.95
    required_negative_relation_specificity: float = 0.95
    required_pointer_accuracy: float = 0.95
    required_query_swap_relocation: float = 0.95

    def validate(self) -> None:
        if self != SourceRelationHeadConfig():
            raise ValueError("C1-SB2 exposes one frozen relation-head fit, not a sweep")
        if self.seed != 9_542 or self.optimizer_steps != 512 or self.batch_size != 126:
            raise AssertionError("C1-SB2 optimizer contract drifted")
        if self.batch_size % (len(OUTCOMES) * len(SOURCE_WIDTHS)) != 0:
            raise AssertionError("C1-SB2 batch does not balance all outcome/width cells")

    def as_contract(self) -> dict[str, Any]:
        self.validate()
        return asdict(self)


@dataclass(slots=True)
class RelationBatch:
    states: Tensor
    candidate_mask: Tensor
    relation_labels: Tensor
    outcomes: Tensor
    target_spans: Tensor


class R4SourceRelationHead(nn.Module):
    """The frozen 288 -> 32 ReLU -> 1 source-relative relation probe."""

    def __init__(self) -> None:
        super().__init__()
        self.input = nn.Linear(FROZEN_MODEL_CONFIG.hidden_size, RELATION_HIDDEN_SIZE)
        self.output = nn.Linear(RELATION_HIDDEN_SIZE, 1)
        nn.init.xavier_uniform_(self.input.weight)
        nn.init.zeros_(self.input.bias)
        nn.init.xavier_uniform_(self.output.weight)
        nn.init.zeros_(self.output.bias)
        parameter_count = sum(parameter.numel() for parameter in self.parameters())
        if parameter_count != TRAINABLE_PARAMETER_COUNT:
            raise AssertionError(
                f"relation head has {parameter_count} parameters, expected "
                f"{TRAINABLE_PARAMETER_COUNT}"
            )

    def forward(self, states: Tensor) -> Tensor:
        return self.output(F.relu(self.input(states.float()))).squeeze(-1)


def relation_loss(model: R4SourceRelationHead, batch: RelationBatch) -> Tensor:
    logits = model(batch.states)
    per_candidate = F.binary_cross_entropy_with_logits(
        logits, batch.relation_labels, reduction="none"
    )
    masked = per_candidate * batch.candidate_mask.to(dtype=per_candidate.dtype)
    per_record = masked.sum(dim=-1) / batch.candidate_mask.sum(dim=-1).clamp_min(1).float()
    return per_record.mean()


def _canonical_with_cid(value: dict[str, Any], field: str) -> dict[str, Any]:
    if field in value:
        raise ValueError(f"self-CID field already exists: {field}")
    result = dict(value)
    result[field] = cid_bytes(canonical_json_bytes(value))
    return result


def _write_or_verify_json(path: Path, value: dict[str, Any]) -> None:
    encoded = canonical_json_bytes(value)
    if path.exists():
        if path.read_bytes() != encoded:
            raise ValueError(f"existing C1-SB2 artifact differs: {path}")
        return
    atomic_write(path, encoded)


def _validated_predecessor(predecessor: Path) -> dict[str, Any]:
    manifest = verify_bound_manifest(
        predecessor / "export-manifest.json", artifact_root=predecessor
    )
    if manifest.get("model_contract") != FROZEN_MODEL_CONFIG.as_contract():
        raise ValueError("relation predecessor is not the immutable six-layer #1017 model")
    if manifest.get("weights_cid") != EXPECTED_WEIGHTS_CID:
        raise ValueError("relation predecessor weights are not the frozen #1017 weights")
    if manifest.get("tokenizer_cid") != EXPECTED_TOKENIZER_CID:
        raise ValueError("relation predecessor tokenizer is not the frozen #1017 tokenizer")
    if cid_file(predecessor / "model.safetensors") != EXPECTED_WEIGHTS_CID:
        raise ValueError("#1017 model file CID does not reproduce")
    if cid_file(predecessor / "tokenizer.json") != EXPECTED_TOKENIZER_CID:
        raise ValueError("#1017 tokenizer file CID does not reproduce")
    return manifest


def _progress(
    phase: str,
    *,
    completed: int,
    total: int,
    started: float,
    loss: float | None = None,
) -> None:
    elapsed = max(0.0, time.monotonic() - started)
    rate = elapsed / completed if completed else 0.0
    eta = rate * max(0, total - completed)
    suffix = "" if loss is None else f" loss={loss:.6f}"
    print(
        f"relation_phase={phase} completed={completed}/{total}{suffix} "
        f"elapsed_seconds={elapsed:.1f} eta_seconds={eta:.1f}",
        flush=True,
    )


def _relation_inputs(records: Sequence[dict[str, Any]]) -> list[str]:
    values: dict[str, str] = {}
    for record in records:
        question = str(record["question"])
        for span in record["sentence_spans"]:
            text = str(span["text"])
            relation_input = str(span["relation_input"])
            expected = render_relation_input(text, question)
            if relation_input != expected:
                raise ValueError("relation input differs from the frozen renderer")
            if relation_input.endswith("\n") or not relation_input.endswith(question):
                raise ValueError("relation input does not end at the exact question mark")
            relation_input_cid = cid_bytes(relation_input.encode("utf-8"))
            if span.get("relation_input_cid") != relation_input_cid:
                raise ValueError("relation input CID does not reproduce")
            prior = values.setdefault(relation_input_cid, relation_input)
            if prior != relation_input:
                raise RuntimeError("BLAKE3 collision in relation-input inventory")
    return [values[key] for key in sorted(values)]


@torch.no_grad()
def _extract_features(
    feature_root: Path,
    *,
    predecessor: Path,
    predecessor_manifest: dict[str, Any],
    records: Sequence[dict[str, Any]],
    population_cids: dict[str, str],
    config: SourceRelationHeadConfig,
) -> dict[str, Any]:
    manifest_path = feature_root / "feature-manifest.json"
    if manifest_path.exists():
        manifest = verify_bound_manifest(manifest_path, artifact_root=feature_root)
        expected = {
            **population_cids,
            "predecessor_export_manifest_cid": predecessor_manifest["manifest_cid"],
            "model_weights_cid": EXPECTED_WEIGHTS_CID,
            "tokenizer_cid": EXPECTED_TOKENIZER_CID,
        }
        if any(manifest.get(key) != value for key, value in expected.items()):
            raise ValueError("existing relation feature manifest binds different inputs")
        return manifest

    device = require_mps(config.seed)
    tokenizer = Tokenizer.from_file(str(predecessor / "tokenizer.json"))
    texts = _relation_inputs(records)
    encoded: list[list[int]] = []
    for text in texts:
        token_ids = tokenizer.encode(text, add_special_tokens=False).ids
        if not token_ids:
            raise ValueError("relation input encoded to zero tokens")
        if len(token_ids) + 1 > FROZEN_MODEL_CONFIG.max_position_embeddings:
            raise ValueError("relation input exceeds the frozen #1017 context")
        if tokenizer.decode([token_ids[-1]], skip_special_tokens=False) != "?":
            raise ValueError("relation input does not end in a standalone question-mark token")
        encoded.append(token_ids)

    states = torch.zeros(
        (len(texts), FROZEN_MODEL_CONFIG.hidden_size), dtype=torch.float32
    )
    model = R4SoftmaxForCausalLM()
    model.load_state_dict(
        load_file(str(predecessor / "model.safetensors"), device="cpu"), strict=True
    )
    model.requires_grad_(False)
    model.eval()
    model = model.to(device)
    started = time.monotonic()
    batches = math.ceil(len(texts) / config.feature_batch_size)
    for batch_index, base in enumerate(
        range(0, len(texts), config.feature_batch_size), start=1
    ):
        batch_token_ids = encoded[base : base + config.feature_batch_size]
        width = 1 + max(map(len, batch_token_ids))
        inputs = torch.full(
            (len(batch_token_ids), width),
            EOS_TOKEN_ID,
            dtype=torch.long,
            device=device,
        )
        for lane, token_ids in enumerate(batch_token_ids):
            inputs[lane, 0] = BOS_TOKEN_ID
            inputs[lane, 1 : 1 + len(token_ids)] = torch.tensor(
                token_ids, dtype=torch.long, device=device
            )
        hidden = model.model(inputs).float().cpu()
        for lane, token_ids in enumerate(batch_token_ids):
            final_state = hidden[lane, len(token_ids)]
            if final_state.shape != (FROZEN_MODEL_CONFIG.hidden_size,):
                raise RuntimeError("#1017 final relation-state shape differs")
            if not bool(torch.isfinite(final_state).all()):
                raise RuntimeError("#1017 final relation state is nonfinite")
            if float(torch.linalg.vector_norm(final_state)) <= 0.0:
                raise RuntimeError("#1017 final relation state is zero")
            states[base + lane] = final_state
        _progress(
            "feature-extraction",
            completed=batch_index,
            total=batches,
            started=started,
        )
    if hasattr(torch, "mps"):
        torch.mps.synchronize()
    del model
    if hasattr(torch, "mps"):
        torch.mps.empty_cache()

    entries = [
        {
            "row": row,
            "relation_input_cid": cid_bytes(text.encode("utf-8")),
            "token_ids": encoded[row],
            "content_token_count": len(encoded[row]),
        }
        for row, text in enumerate(texts)
    ]
    index = {
        "schema": FEATURE_INDEX_SCHEMA,
        "policy": POLICY,
        "model_weights_cid": EXPECTED_WEIGHTS_CID,
        "tokenizer_cid": EXPECTED_TOKENIZER_CID,
        "state_definition": (
            "one final RMS-normalized residual at the exact question-mark token after "
            "all six immutable #1017 R4/Spin causal-softmax layers; each exact "
            "Evidence/span/Question input encoded independently"
        ),
        "relation_input_policy": RELATION_INPUT_TEMPLATE,
        "relation_input_policy_description": RELATION_INPUT_POLICY,
        "hidden_size": FROZEN_MODEL_CONFIG.hidden_size,
        "entries": entries,
    }
    feature_root.mkdir(parents=True, exist_ok=True)
    atomic_write_json(feature_root / "feature-index.json", index)
    state_path = feature_root / "states.safetensors"
    temporary = feature_root / ".states.safetensors.part"
    save_file(
        {"states": states.contiguous()},
        str(temporary),
        metadata={
            "schema": FEATURE_INDEX_SCHEMA,
            "model_weights_cid": EXPECTED_WEIGHTS_CID,
            "tokenizer_cid": EXPECTED_TOKENIZER_CID,
        },
    )
    os.replace(temporary, state_path)
    return write_bound_manifest(
        manifest_path,
        {
            "schema": FEATURE_MANIFEST_SCHEMA,
            "issue": ISSUE,
            "policy": POLICY,
            **population_cids,
            "predecessor_export_manifest_cid": predecessor_manifest["manifest_cid"],
            "model_weights_cid": EXPECTED_WEIGHTS_CID,
            "tokenizer_cid": EXPECTED_TOKENIZER_CID,
            "relation_input_count": len(texts),
        },
        artifact_root=feature_root,
        relative_paths=["feature-index.json", "states.safetensors"],
    )


class FrozenRelationFeatureStore:
    """Read-only final-state table keyed by exact relation-input CID."""

    def __init__(self, root: Path) -> None:
        verify_bound_manifest(root / "feature-manifest.json", artifact_root=root)
        index = json.loads((root / "feature-index.json").read_text(encoding="utf-8"))
        if index.get("schema") != FEATURE_INDEX_SCHEMA:
            raise ValueError("relation feature index schema differs")
        if (
            index.get("policy") != POLICY
            or index.get("model_weights_cid") != EXPECTED_WEIGHTS_CID
            or index.get("tokenizer_cid") != EXPECTED_TOKENIZER_CID
            or int(index.get("hidden_size", -1)) != FROZEN_MODEL_CONFIG.hidden_size
            or index.get("relation_input_policy") != RELATION_INPUT_TEMPLATE
        ):
            raise ValueError("relation feature index binding differs")
        tensors = load_file(str(root / "states.safetensors"), device="cpu")
        self.states = tensors["states"].float().contiguous()
        self.rows: dict[str, int] = {}
        for entry in index["entries"]:
            row = int(entry["row"])
            relation_input_cid = str(entry["relation_input_cid"])
            if relation_input_cid in self.rows:
                raise ValueError("duplicate relation-input feature CID")
            self.rows[relation_input_cid] = row
        if self.states.shape != (
            len(self.rows),
            FROZEN_MODEL_CONFIG.hidden_size,
        ):
            raise ValueError("relation state tensor shape differs")
        if not bool(torch.isfinite(self.states).all()) or not bool(
            (torch.linalg.vector_norm(self.states, dim=-1) > 0).all()
        ):
            raise ValueError("relation feature states are invalid")

    def get(self, relation_input: str, expected_cid: str) -> Tensor:
        observed_cid = cid_bytes(relation_input.encode("utf-8"))
        if observed_cid != expected_cid:
            raise ValueError("relation input does not reproduce its record CID")
        try:
            row = self.rows[observed_cid]
        except KeyError as error:
            raise KeyError("relation input is absent from frozen feature store") from error
        return self.states[row]


class RelationDataset:
    """Frozen relation records assembled into record-balanced padded batches."""

    def __init__(
        self,
        records: Sequence[dict[str, Any]],
        features: FrozenRelationFeatureStore,
    ) -> None:
        self.records = list(records)
        if not self.records:
            raise ValueError("source-relative relation dataset is empty")
        self.features = features
        self.cell_indices: dict[tuple[str, int], list[int]] = {
            (outcome, width): [] for outcome in OUTCOMES for width in SOURCE_WIDTHS
        }
        for index, record in enumerate(self.records):
            outcome = str(record["target_outcome"])
            width = int(record["source_width"])
            cell = (outcome, width)
            if cell not in self.cell_indices:
                raise ValueError(f"unknown relation outcome/width cell: {cell}")
            spans = record["sentence_spans"]
            if len(spans) != width:
                raise ValueError("relation source_width differs from candidate count")
            self.cell_indices[cell].append(index)

    def require_all_cells(self) -> None:
        missing = [cell for cell, indices in self.cell_indices.items() if not indices]
        if missing:
            raise ValueError(f"relation dataset has empty outcome/width cells: {missing}")

    def deterministic_indices(self, *, seed: int, step: int, batch_size: int) -> list[int]:
        self.require_all_cells()
        cells = [(outcome, width) for outcome in OUTCOMES for width in SOURCE_WIDTHS]
        if batch_size % len(cells) != 0:
            raise ValueError("relation batch cannot balance all outcome/width cells")
        selected: list[int] = []
        for lane in range(batch_size):
            cell = cells[lane % len(cells)]
            population = self.cell_indices[cell]
            material = struct.pack(">QQQ", seed, step, lane)
            offset = int.from_bytes(blake3(material).digest(), "big") % len(population)
            selected.append(population[offset])
        return selected

    def batch(self, indices: Sequence[int], *, device: torch.device) -> RelationBatch:
        selected = [self.records[index] for index in indices]
        maximum_candidates = max(len(record["sentence_spans"]) for record in selected)
        batch_size = len(selected)
        states = torch.zeros(
            (batch_size, maximum_candidates, FROZEN_MODEL_CONFIG.hidden_size),
            dtype=torch.float32,
        )
        candidate_mask = torch.zeros(
            (batch_size, maximum_candidates), dtype=torch.bool
        )
        relation_labels = torch.zeros(
            (batch_size, maximum_candidates), dtype=torch.float32
        )
        outcomes = torch.empty(batch_size, dtype=torch.long)
        target_spans = torch.full((batch_size,), -1, dtype=torch.long)
        for lane, record in enumerate(selected):
            for candidate_index, span in enumerate(record["sentence_spans"]):
                relation_input = str(span["relation_input"])
                relation_input_cid = str(span["relation_input_cid"])
                states[lane, candidate_index] = self.features.get(
                    relation_input, relation_input_cid
                )
                candidate_mask[lane, candidate_index] = True
                relation_label = int(span["relation_label"])
                if relation_label not in (0, 1):
                    raise ValueError("relation label is not binary")
                relation_labels[lane, candidate_index] = float(relation_label)
            outcome = str(record["target_outcome"])
            outcomes[lane] = OUTCOME_TO_ID[outcome]
            target = record.get("target_span_index")
            if target is not None:
                if not isinstance(target, int) or not 0 <= target < len(
                    record["sentence_spans"]
                ):
                    raise ValueError("relation target span is outside admitted candidates")
                target_spans[lane] = target
        return RelationBatch(
            states=states.to(device),
            candidate_mask=candidate_mask.to(device),
            relation_labels=relation_labels.to(device),
            outcomes=outcomes.to(device),
            target_spans=target_spans.to(device),
        )


def _decision_from_logits(
    record: dict[str, Any], candidate_logits: Sequence[float]
) -> dict[str, Any]:
    spans = record["sentence_spans"]
    if len(candidate_logits) != len(spans):
        raise ValueError("candidate logit count differs from source spans")
    if any(not math.isfinite(float(value)) for value in candidate_logits):
        raise ValueError("relation head produced a nonfinite candidate logit")

    groups: dict[str, list[int]] = {}
    for index, span in enumerate(spans):
        group_cid = str(span["relation_group_cid"])
        groups.setdefault(group_cid, []).append(index)
    group_logits: list[tuple[str, float, int]] = []
    for group_cid, indices in groups.items():
        values = [float(candidate_logits[index]) for index in indices]
        if any(value != values[0] for value in values[1:]):
            raise RuntimeError("exact duplicate relation spans received different logits")
        earliest = min(indices, key=lambda index: int(spans[index]["byte_start"]))
        group_logits.append((group_cid, values[0], earliest))
    positive = [group for group in group_logits if group[1] > 0.0]
    if not positive:
        decision = "abstain"
        selected_span_index: int | None = None
    elif len(positive) >= 2:
        decision = "conflict"
        selected_span_index = None
    else:
        decision = "answer"
        selected_span_index = positive[0][2]

    ranked = sorted(
        range(len(spans)),
        key=lambda index: (-float(candidate_logits[index]), int(spans[index]["byte_start"])),
    )
    return {
        "candidate_logits": [float(value) for value in candidate_logits],
        "ranked_candidate_indices": ranked,
        "positive_relation_group_cids": sorted(group[0] for group in positive),
        "decision": decision,
        "selected_span_index": selected_span_index,
    }


@torch.no_grad()
def score_relation_record(
    model: R4SourceRelationHead,
    dataset: RelationDataset,
    *,
    record_index: int,
    device: torch.device,
) -> dict[str, Any]:
    model.eval()
    record = dataset.records[record_index]
    batch = dataset.batch([record_index], device=device)
    logits = model(batch.states)[0, : len(record["sentence_spans"])]
    candidate_logits = [float(value) for value in logits.detach().cpu().float().tolist()]
    return _decision_from_logits(record, candidate_logits)


@torch.no_grad()
def evaluate_relation_head(
    model: R4SourceRelationHead,
    dataset: RelationDataset,
    *,
    device: torch.device,
    batch_size: int,
    indices: Sequence[int] | None = None,
    include_records: bool = False,
) -> dict[str, Any]:
    model.eval()
    selected = list(range(len(dataset.records))) if indices is None else list(indices)
    outcome_correct = {name: 0 for name in OUTCOMES}
    outcome_total = {name: 0 for name in OUTCOMES}
    positive_correct = 0
    positive_total = 0
    negative_correct = 0
    negative_total = 0
    pointer_correct = 0
    pointer_total = 0
    loss_sum = 0.0
    examples = 0
    evaluations: list[dict[str, Any]] = []
    for base in range(0, len(selected), batch_size):
        batch_indices = selected[base : base + batch_size]
        batch = dataset.batch(batch_indices, device=device)
        logits = model(batch.states)
        loss = relation_loss(model, batch)
        loss_sum += float(loss.detach().cpu()) * len(batch_indices)
        examples += len(batch_indices)
        logits_cpu = logits.detach().cpu().float()
        for lane, record_index in enumerate(batch_indices):
            record = dataset.records[record_index]
            width = len(record["sentence_spans"])
            candidate_logits = [float(value) for value in logits_cpu[lane, :width].tolist()]
            evaluation = _decision_from_logits(record, candidate_logits)
            outcome = str(record["target_outcome"])
            outcome_total[outcome] += 1
            outcome_correct[outcome] += int(evaluation["decision"] == outcome)
            for candidate_index, span in enumerate(record["sentence_spans"]):
                label = int(span["relation_label"])
                predicted = int(candidate_logits[candidate_index] > 0.0)
                if label:
                    positive_total += 1
                    positive_correct += int(predicted == 1)
                else:
                    negative_total += 1
                    negative_correct += int(predicted == 0)
            if outcome == "answer":
                pointer_total += 1
                pointer_correct += int(
                    evaluation["selected_span_index"] == record.get("target_span_index")
                )
            if include_records:
                evaluations.append(
                    {
                        "record_cid": record["record_cid"],
                        "source_cid": record["source_cid"],
                        "question_cid": record["question_cid"],
                        "target_outcome": outcome,
                        "target_span_index": record.get("target_span_index"),
                        **evaluation,
                    }
                )
    if not examples or not positive_total or not negative_total or not pointer_total:
        raise RuntimeError("relation evaluation population is incomplete")
    value: dict[str, Any] = {
        "mean_record_balanced_relation_loss": loss_sum / examples,
        "outcome": {
            outcome: {
                "correct": outcome_correct[outcome],
                "total": outcome_total[outcome],
                "accuracy": (
                    outcome_correct[outcome] / outcome_total[outcome]
                    if outcome_total[outcome]
                    else None
                ),
            }
            for outcome in OUTCOMES
        },
        "positive_relation_recall": {
            "correct": positive_correct,
            "total": positive_total,
            "accuracy": positive_correct / positive_total,
        },
        "negative_relation_specificity": {
            "correct": negative_correct,
            "total": negative_total,
            "accuracy": negative_correct / negative_total,
        },
        "supported_copied_span": {
            "correct": pointer_correct,
            "total": pointer_total,
            "accuracy": pointer_correct / pointer_total,
        },
    }
    if include_records:
        value["records"] = evaluations
    return value


def _fit_head(
    model: R4SourceRelationHead,
    dataset: RelationDataset,
    *,
    device: torch.device,
    steps: int,
    batch_size: int,
    config: SourceRelationHeadConfig,
    phase: str,
    fixed_indices: Sequence[int] | None = None,
) -> dict[str, Any]:
    model.train()
    optimizer = torch.optim.AdamW(
        model.parameters(),
        lr=config.learning_rate,
        betas=(config.adam_beta1, config.adam_beta2),
        eps=config.adam_epsilon,
        weight_decay=0.0,
    )
    started = time.monotonic()
    initial_loss: float | None = None
    final_loss = math.nan
    for step in range(1, steps + 1):
        indices = (
            list(fixed_indices)
            if fixed_indices is not None
            else dataset.deterministic_indices(
                seed=config.seed, step=step, batch_size=batch_size
            )
        )
        batch = dataset.batch(indices, device=device)
        optimizer.zero_grad(set_to_none=True)
        loss = relation_loss(model, batch)
        loss.backward()
        optimizer.step()
        report_step = initial_loss is None or step % config.progress_interval == 0 or step == steps
        if report_step:
            final_loss = float(loss.detach().cpu())
            if not math.isfinite(final_loss):
                raise RuntimeError("relation-head fit produced a nonfinite loss")
            if initial_loss is None:
                initial_loss = final_loss
        if step % config.progress_interval == 0 or step == steps:
            _progress(
                phase,
                completed=step,
                total=steps,
                started=started,
                loss=final_loss,
            )
    if hasattr(torch, "mps"):
        torch.mps.synchronize()
    return {
        "optimizer_steps": steps,
        "batch_size": batch_size,
        "initial_relation_loss": initial_loss,
        "final_relation_loss": final_loss,
        "elapsed_seconds": time.monotonic() - started,
    }


def _finite_list(tensor: Tensor, *, length: int, label: str) -> list[float]:
    values = [
        float(value)
        for value in tensor.detach().to(device="cpu", dtype=torch.float32).reshape(-1).tolist()
    ]
    if len(values) != length or any(not math.isfinite(value) for value in values):
        raise RuntimeError(f"relation head {label} violates its finite shape contract")
    return values


def _head_payload(
    model: R4SourceRelationHead,
    *,
    dataset: dict[str, Any],
    run_contract_cid: str,
    training_result_cid: str,
    preflight: dict[str, Any],
    development_metrics: dict[str, Any],
) -> dict[str, Any]:
    output_bias = float(
        model.output.bias.detach().to(device="cpu", dtype=torch.float32).item()
    )
    if not math.isfinite(output_bias):
        raise RuntimeError("relation head output bias is nonfinite")
    commitments = dataset["product_probe_commitments"]
    if len(commitments) != 4:
        raise RuntimeError("C1-SB2 requires exactly four product commitments")
    value: dict[str, Any] = {
        "schema": HEAD_SCHEMA,
        "policy": POLICY,
        "issue": ISSUE,
        "model_weights_cid": EXPECTED_WEIGHTS_CID,
        "tokenizer_cid": EXPECTED_TOKENIZER_CID,
        "hidden_size": FROZEN_MODEL_CONFIG.hidden_size,
        "hidden_width": RELATION_HIDDEN_SIZE,
        "first_layer_weights": _finite_list(
            model.input.weight,
            length=RELATION_HIDDEN_SIZE * FROZEN_MODEL_CONFIG.hidden_size,
            label="input weights",
        ),
        "first_layer_biases": _finite_list(
            model.input.bias, length=RELATION_HIDDEN_SIZE, label="input bias"
        ),
        "output_weights": _finite_list(
            model.output.weight,
            length=RELATION_HIDDEN_SIZE,
            label="output weights",
        ),
        "output_bias": output_bias,
        "threshold": 0.0,
        "maximum_source_spans": max(SOURCE_WIDTHS),
        "relation_input_policy": RELATION_INPUT_TEMPLATE,
        "dataset_cid": dataset["dataset_cid"],
        "split_policy_cid": dataset["split_policy_cid"],
        "run_contract_cid": run_contract_cid,
        "training_result_cid": training_result_cid,
        "preflight": preflight,
        "development_metrics": development_metrics,
        "product_probe_commitments": commitments,
    }
    return _canonical_with_cid(value, "artifact_cid")


def _all_exact(metrics: dict[str, Any]) -> bool:
    return (
        all(
            metrics["outcome"][outcome]["correct"]
            == metrics["outcome"][outcome]["total"]
            for outcome in OUTCOMES
        )
        and metrics["positive_relation_recall"]["correct"]
        == metrics["positive_relation_recall"]["total"]
        and metrics["negative_relation_specificity"]["correct"]
        == metrics["negative_relation_specificity"]["total"]
        and metrics["supported_copied_span"]["correct"]
        == metrics["supported_copied_span"]["total"]
    )


def _reverse_preflight_records(
    records: Sequence[dict[str, Any]],
) -> list[dict[str, Any]]:
    reversed_records: list[dict[str, Any]] = []
    for record in records:
        original_spans = list(record["sentence_spans"])
        mapping = list(reversed(range(len(original_spans))))
        reversed_spans: list[dict[str, Any]] = []
        for candidate_index, original_index in enumerate(mapping):
            span = dict(original_spans[original_index])
            span["candidate_index"] = candidate_index
            reversed_spans.append(span)
        target = record.get("target_span_index")
        reversed_target = None if target is None else mapping.index(int(target))
        value = dict(record)
        value.pop("record_cid", None)
        value.update(
            {
                "population": "preflight-order-control",
                "motif": "exact-candidate-array-reversal",
                "sentence_spans": reversed_spans,
                "positive_span_indices": [
                    index
                    for index, span in enumerate(reversed_spans)
                    if int(span["relation_label"]) == 1
                ],
                "target_span_index": reversed_target,
                "control_kind": "order-reversal",
                "base_record_cid": record["record_cid"],
                "candidate_original_indices": mapping,
            }
        )
        value["record_cid"] = cid_bytes(canonical_json_bytes(value))
        reversed_records.append(value)
    return reversed_records


def _preflight_control_checks(
    preflight: dict[str, Any],
    fit_metrics: dict[str, Any],
    sealed_metrics: dict[str, Any],
    reversed_records: Sequence[dict[str, Any]],
    reversed_metrics: dict[str, Any],
) -> dict[str, Any]:
    evaluations = {
        str(record["record_cid"]): record
        for record in [*fit_metrics["records"], *sealed_metrics["records"]]
    }
    matched_exact = True
    query_swap_sensitive = True
    for pair in preflight["matched_pairs"]:
        left = evaluations[str(pair["left_record_cid"])]
        right = evaluations[str(pair["right_record_cid"])]
        matched_exact &= bool(pair["same_source"])
        matched_exact &= left["decision"] == left["target_outcome"]
        matched_exact &= right["decision"] == right["target_outcome"]
        query_swap_sensitive &= left["decision"] != right["decision"]

    all_records = [*preflight["fit"], *preflight["sealed"]]
    duplicate_records = [
        record for record in all_records if record["motif"] == "exact-duplicate-agreement"
    ]
    conflict_records = [
        record for record in all_records if record["motif"] == "distinct-location-conflict"
    ]
    duplicate_agreement = all(
        evaluations[str(record["record_cid"])]["decision"] == "answer"
        and evaluations[str(record["record_cid"])]["selected_span_index"]
        == record["target_span_index"]
        for record in duplicate_records
    )
    different_value_conflict = all(
        evaluations[str(record["record_cid"])]["decision"] == "conflict"
        and evaluations[str(record["record_cid"])]["selected_span_index"] is None
        for record in conflict_records
    )
    reversed_evaluations = {
        str(record["record_cid"]): record for record in reversed_metrics["records"]
    }
    order_invariance = True
    for record in reversed_records:
        original = evaluations[str(record["base_record_cid"])]
        observed = reversed_evaluations[str(record["record_cid"])]
        mapping = [int(index) for index in record["candidate_original_indices"]]
        logits_exact = all(
            observed["candidate_logits"][new_index]
            == original["candidate_logits"][old_index]
            for new_index, old_index in enumerate(mapping)
        )
        expected_selected = None
        if original["selected_span_index"] is not None:
            expected_selected = mapping.index(int(original["selected_span_index"]))
        order_invariance &= (
            logits_exact
            and observed["decision"] == original["decision"]
            and observed["selected_span_index"] == expected_selected
        )
    value = {
        "same_source_matched_pairs_exact": matched_exact,
        "query_swap_sensitive": query_swap_sensitive,
        "candidate_array_order_equivariant": order_invariance,
        "duplicate_agreement": duplicate_agreement,
        "different_value_conflict": different_value_conflict,
    }
    value["passed"] = all(value.values())
    return value


def _prepare_contract(
    root: Path,
    *,
    predecessor_manifest: dict[str, Any],
    config: SourceRelationHeadConfig,
) -> tuple[
    dict[str, Any],
    dict[str, Any],
    dict[str, Any],
    dict[str, Any],
    dict[str, Any],
    dict[str, Any],
]:
    dataset, preflight, probes = build_source_relation_population()
    if DATA_POLICY != POLICY or dataset.get("policy") != POLICY:
        raise ValueError("C1-SB2 data and trainer policies differ")
    if preflight != build_relation_preflight() or probes != product_probes():
        raise RuntimeError("C1-SB2 independently rebuilt commitments differ")
    census = shortcut_census(dataset["construction"], dataset["development"], probes)
    if census != dataset["shortcut_census"] or not census.get("passed"):
        raise RuntimeError("C1-SB2 shortcut census did not reproduce before MPS work")

    root.mkdir(parents=True, exist_ok=True)
    _write_or_verify_json(root / "source-relation-dataset.json", dataset)
    _write_or_verify_json(root / "source-relation-preflight.json", preflight)
    _write_or_verify_json(root / "product-probes.json", probes)
    _write_or_verify_json(root / "shortcut-census.json", census)
    dataset_manifest_path = root / "source-relation-dataset-manifest.json"
    if dataset_manifest_path.exists():
        dataset_manifest = verify_bound_manifest(dataset_manifest_path, artifact_root=root)
        if (
            dataset_manifest.get("dataset_cid") != dataset["dataset_cid"]
            or dataset_manifest.get("preflight_cid") != preflight["preflight_cid"]
            or dataset_manifest.get("product_probes_cid") != probes["product_probes_cid"]
        ):
            raise ValueError("existing C1-SB2 dataset manifest differs")
    else:
        dataset_manifest = write_bound_manifest(
            dataset_manifest_path,
            {
                "schema": DATASET_MANIFEST_SCHEMA,
                "issue": ISSUE,
                "policy": POLICY,
                "dataset_cid": dataset["dataset_cid"],
                "split_policy_cid": dataset["split_policy_cid"],
                "generator_contract_cid": dataset["generator_contract_cid"],
                "preflight_cid": preflight["preflight_cid"],
                "product_probes_cid": probes["product_probes_cid"],
                "product_probe_commitments": dataset["product_probe_commitments"],
                "shortcut_census_cid": census["census_cid"],
                "predecessor_weights_cid": EXPECTED_WEIGHTS_CID,
                "predecessor_tokenizer_cid": EXPECTED_TOKENIZER_CID,
            },
            artifact_root=root,
            relative_paths=[
                "source-relation-dataset.json",
                "source-relation-preflight.json",
                "product-probes.json",
                "shortcut-census.json",
            ],
        )

    run_contract = _canonical_with_cid(
        {
            "schema": RUN_SCHEMA,
            "issue": ISSUE,
            "policy": POLICY,
            "predecessor": {
                "export_manifest_cid": predecessor_manifest["manifest_cid"],
                "weights_cid": EXPECTED_WEIGHTS_CID,
                "tokenizer_cid": EXPECTED_TOKENIZER_CID,
                "failed_grounding_sft_weights": "EXCLUDED",
                "failed_source_span_pointer": "EXCLUDED",
            },
            "model": FROZEN_MODEL_CONFIG.as_contract(),
            "dataset_cid": dataset["dataset_cid"],
            "dataset_manifest_cid": dataset_manifest["manifest_cid"],
            "split_policy_cid": dataset["split_policy_cid"],
            "generator_contract_cid": dataset["generator_contract_cid"],
            "preflight_cid": preflight["preflight_cid"],
            "product_probes_cid": probes["product_probes_cid"],
            "product_probe_commitments": dataset["product_probe_commitments"],
            "shortcut_census_cid": census["census_cid"],
            "state_extraction": (
                "MPS-only immutable #1017 final normalized residual at the exact final "
                "question-mark token for each independently encoded evidence/question input"
            ),
            "question_policy": QUESTION_POLICY,
            "sentence_policy": SENTENCE_POLICY,
            "relation_input_policy": RELATION_INPUT_TEMPLATE,
            "relation_input_policy_description": RELATION_INPUT_POLICY,
            "mechanism": (
                "f32 288-to-32 affine, ReLU, 32-to-1 affine; strict positive logits; "
                "exact-duplicate relation groups collapsed before answer/abstain/conflict"
            ),
            "parameter_count": TRAINABLE_PARAMETER_COUNT,
            "loss": "record-balanced mean candidate binary cross entropy with logits",
            "optimization": config.as_contract(),
            "cheap_gate": {
                "fit_records": 12,
                "sealed_transfer_records": 12,
                "optimizer_steps": config.preflight_optimizer_steps,
                "required_outcomes_relations_and_pointers": "exact",
                "required_shortcut_controls": "exact",
                "python_rust_records": 3,
                "python_rust_candidate_logit_tolerance": PARITY_MAX_ABSOLUTE_TOLERANCE,
            },
            "development_gate": {
                "minimum_per_class_outcome_accuracy": config.required_per_class_accuracy,
                "minimum_positive_relation_recall": config.required_positive_relation_recall,
                "minimum_negative_relation_specificity": (
                    config.required_negative_relation_specificity
                ),
                "minimum_supported_copied_span_accuracy": config.required_pointer_accuracy,
                "minimum_query_swap_relocation": config.required_query_swap_relocation,
                "order_equivariance": "exact",
            },
            "product_behavior": (
                "four records committed before training; never feature-extracted, "
                "scored, or evaluated by Python"
            ),
            "implementation": trainer_implementation_contract(),
        },
        "run_contract_cid",
    )
    _write_or_verify_json(root / "source-relation-run-contract.json", run_contract)
    return dataset, preflight, probes, census, dataset_manifest, run_contract


def _run_preflight(
    root: Path,
    *,
    predecessor: Path,
    predecessor_manifest: dict[str, Any],
    dataset: dict[str, Any],
    preflight: dict[str, Any],
    dataset_manifest: dict[str, Any],
    run_contract: dict[str, Any],
    config: SourceRelationHeadConfig,
) -> dict[str, Any]:
    records = [*preflight["fit"], *preflight["sealed"]]
    feature_manifest = _extract_features(
        root / "preflight/features",
        predecessor=predecessor,
        predecessor_manifest=predecessor_manifest,
        records=records,
        population_cids={
            "preflight_cid": preflight["preflight_cid"],
            "dataset_manifest_cid": dataset_manifest["manifest_cid"],
        },
        config=config,
    )
    device = require_mps(config.seed)
    features = FrozenRelationFeatureStore(root / "preflight/features")
    fit_dataset = RelationDataset(preflight["fit"], features)
    sealed_dataset = RelationDataset(preflight["sealed"], features)
    torch.manual_seed(config.seed)
    if hasattr(torch.mps, "manual_seed"):
        torch.mps.manual_seed(config.seed)
    model = R4SourceRelationHead().to(device)
    fit = _fit_head(
        model,
        fit_dataset,
        device=device,
        steps=config.preflight_optimizer_steps,
        batch_size=len(fit_dataset.records),
        config=config,
        phase="12-fit-matched-transfer",
        fixed_indices=list(range(len(fit_dataset.records))),
    )
    fit_metrics = evaluate_relation_head(
        model,
        fit_dataset,
        device=device,
        batch_size=len(fit_dataset.records),
        include_records=True,
    )
    sealed_metrics = evaluate_relation_head(
        model,
        sealed_dataset,
        device=device,
        batch_size=len(sealed_dataset.records),
        include_records=True,
    )
    reversed_records = _reverse_preflight_records(records)
    reversed_dataset = RelationDataset(reversed_records, features)
    reversed_metrics = evaluate_relation_head(
        model,
        reversed_dataset,
        device=device,
        batch_size=len(reversed_dataset.records),
        include_records=True,
    )
    controls = _preflight_control_checks(
        preflight,
        fit_metrics,
        sealed_metrics,
        reversed_records,
        reversed_metrics,
    )
    passed = _all_exact(fit_metrics) and _all_exact(sealed_metrics) and controls["passed"]
    deterministic_fit = {key: value for key, value in fit.items() if key != "elapsed_seconds"}
    result = _canonical_with_cid(
        {
            "schema": PREFLIGHT_RESULT_SCHEMA,
            "issue": ISSUE,
            "policy": POLICY,
            "terminal": (
                "PASS_MATCHED_TRANSFER_AWAIT_RUST_RELATION_PARITY"
                if passed
                else "FAIL_MATCHED_TRANSFER_PREFLIGHT_STOP"
            ),
            "run_contract_cid": run_contract["run_contract_cid"],
            "dataset_cid": dataset["dataset_cid"],
            "preflight_cid": preflight["preflight_cid"],
            "feature_manifest_cid": feature_manifest["manifest_cid"],
            "fit_record_cids": [record["record_cid"] for record in preflight["fit"]],
            "sealed_record_cids": [
                record["record_cid"] for record in preflight["sealed"]
            ],
            "optimization": deterministic_fit,
            "fit_metrics": fit_metrics,
            "sealed_transfer_metrics": sealed_metrics,
            "reversed_order_metrics": reversed_metrics,
            "shortcut_controls": controls,
            "passed": passed,
        },
        "result_cid",
    )
    preflight_root = root / "preflight"
    atomic_write_json(preflight_root / "preflight-result.json", result)
    if not passed:
        manifest = write_bound_manifest(
            preflight_root / "preflight-manifest.json",
            {
                "schema": PREFLIGHT_RESULT_SCHEMA,
                "terminal": result["terminal"],
                "run_contract_cid": run_contract["run_contract_cid"],
                "preflight_result_cid": result["result_cid"],
            },
            artifact_root=root,
            relative_paths=[
                "source-relation-dataset.json",
                "source-relation-preflight.json",
                "product-probes.json",
                "shortcut-census.json",
                "source-relation-dataset-manifest.json",
                "source-relation-run-contract.json",
                "preflight/features/feature-index.json",
                "preflight/features/states.safetensors",
                "preflight/features/feature-manifest.json",
                "preflight/preflight-result.json",
            ],
        )
        return {
            "terminal": result["terminal"],
            "preflight_result_cid": result["result_cid"],
            "preflight_manifest_cid": manifest["manifest_cid"],
        }

    embedded_preflight = {
        "status": "PASS_12_FIT_12_SEALED_MATCHED_TRANSFER",
        "preflight_result_cid": result["result_cid"],
        "optimizer_steps": config.preflight_optimizer_steps,
        "fit_metrics": fit_metrics,
        "sealed_transfer_metrics": sealed_metrics,
        "shortcut_controls": controls,
        "rust_score_parity": "AWAITING",
    }
    head = _head_payload(
        model,
        dataset=dataset,
        run_contract_cid=run_contract["run_contract_cid"],
        training_result_cid=result["result_cid"],
        preflight=embedded_preflight,
        development_metrics={"status": "NOT_RUN_BEFORE_SOLE_512_STEP_FIT"},
    )
    atomic_write_json(preflight_root / "preflight-head.json", head)

    fixtures: list[dict[str, Any]] = []
    fixture_paths: list[str] = []
    for outcome in OUTCOMES:
        record_index = next(
            index
            for index, record in enumerate(sealed_dataset.records)
            if record["target_outcome"] == outcome
        )
        record = sealed_dataset.records[record_index]
        evaluation = score_relation_record(
            model, sealed_dataset, record_index=record_index, device=device
        )
        if (
            evaluation["decision"] != record["target_outcome"]
            or evaluation["selected_span_index"] != record.get("target_span_index")
        ):
            raise RuntimeError("passing matched-transfer preflight yielded a bad parity case")
        source_relative = f"preflight/parity/{outcome}-source.txt"
        source_path = root / source_relative
        atomic_write(source_path, str(record["source"]).encode("utf-8"))
        fixture = _canonical_with_cid(
            {
                "schema": PYTHON_SCORE_FIXTURE_SCHEMA,
                "policy": POLICY,
                "outcome": outcome,
                "preflight_artifact_cid": head["artifact_cid"],
                "record_cid": record["record_cid"],
                "source_cid": record["source_cid"],
                "source_path": source_relative,
                "question": record["question"],
                "question_cid": record["question_cid"],
                **evaluation,
                "maximum_absolute_tolerance": PARITY_MAX_ABSOLUTE_TOLERANCE,
            },
            "fixture_cid",
        )
        fixture_relative = f"preflight/parity/{outcome}-python-score-fixture.json"
        atomic_write_json(root / fixture_relative, fixture)
        fixtures.append(fixture)
        fixture_paths.extend([source_relative, fixture_relative])

    parity_index = _canonical_with_cid(
        {
            "schema": PYTHON_SCORE_FIXTURE_SCHEMA,
            "policy": POLICY,
            "preflight_artifact_cid": head["artifact_cid"],
            "fixtures": [
                {
                    "outcome": fixture["outcome"],
                    "fixture_cid": fixture["fixture_cid"],
                    "record_cid": fixture["record_cid"],
                }
                for fixture in fixtures
            ],
        },
        "index_cid",
    )
    atomic_write_json(preflight_root / "parity-index.json", parity_index)
    manifest = write_bound_manifest(
        preflight_root / "preflight-manifest.json",
        {
            "schema": PREFLIGHT_RESULT_SCHEMA,
            "terminal": "AWAITING_THREE_RUST_RELATION_PARITY_REPORTS",
            "run_contract_cid": run_contract["run_contract_cid"],
            "dataset_manifest_cid": dataset_manifest["manifest_cid"],
            "preflight_result_cid": result["result_cid"],
            "preflight_artifact_cid": head["artifact_cid"],
            "parity_index_cid": parity_index["index_cid"],
        },
        artifact_root=root,
        relative_paths=[
            "source-relation-dataset.json",
            "source-relation-preflight.json",
            "product-probes.json",
            "shortcut-census.json",
            "source-relation-dataset-manifest.json",
            "source-relation-run-contract.json",
            "preflight/features/feature-index.json",
            "preflight/features/states.safetensors",
            "preflight/features/feature-manifest.json",
            "preflight/preflight-result.json",
            "preflight/preflight-head.json",
            "preflight/parity-index.json",
            *fixture_paths,
        ],
    )
    return {
        "terminal": "AWAITING_THREE_RUST_RELATION_PARITY_REPORTS",
        "preflight_artifact": str(preflight_root / "preflight-head.json"),
        "preflight_artifact_cid": head["artifact_cid"],
        "parity_fixtures": [
            {
                "outcome": fixture["outcome"],
                "source": str(root / fixture["source_path"]),
                "question": fixture["question"],
                "fixture": str(
                    root
                    / f"preflight/parity/{fixture['outcome']}-python-score-fixture.json"
                ),
                "fixture_cid": fixture["fixture_cid"],
            }
            for fixture in fixtures
        ],
        "preflight_manifest_cid": manifest["manifest_cid"],
    }


def _numeric_logits(value: Any) -> list[float]:
    if not isinstance(value, list) or not value:
        raise ValueError("Rust relation candidate_logits must be a nonempty list")
    logits = [float(item) for item in value]
    if any(not math.isfinite(item) for item in logits):
        raise ValueError("Rust relation candidate_logits contain a nonfinite value")
    return logits


def _admit_rust_score_parity(
    root: Path, report_paths: Sequence[Path]
) -> dict[str, Any]:
    if len(report_paths) != len(OUTCOMES):
        raise ValueError("C1-SB2 requires exactly three Rust parity reports")
    resolved = [path.expanduser().resolve() for path in report_paths]
    if len(set(resolved)) != len(resolved) or any(not path.is_file() for path in resolved):
        raise ValueError("Rust parity paths must be three distinct regular files")
    head = json.loads((root / "preflight/preflight-head.json").read_text(encoding="utf-8"))
    fixtures = {
        outcome: json.loads(
            (
                root
                / f"preflight/parity/{outcome}-python-score-fixture.json"
            ).read_text(encoding="utf-8")
        )
        for outcome in OUTCOMES
    }
    reports = [json.loads(path.read_text(encoding="utf-8")) for path in resolved]
    report_by_outcome: dict[str, dict[str, Any]] = {}
    admissions: list[dict[str, Any]] = []
    for report in reports:
        if report.get("schema") != RUST_GROUNDED_REPORT_SCHEMA:
            raise ValueError("Rust parity report is not uor-r4.grounded-answer/3")
        relation = report.get("relation")
        evaluation = report.get("relation_evaluation")
        source = report.get("source")
        state_encoding = report.get("state_encoding")
        if not isinstance(relation, dict) or not isinstance(evaluation, dict):
            raise ValueError("Rust parity report omits relation binding/evaluation")
        if not isinstance(source, dict) or not isinstance(state_encoding, dict):
            raise ValueError("Rust parity report omits source/state encoding")
        if relation.get("artifact_cid") != head["artifact_cid"]:
            raise ValueError("Rust parity loaded a different preflight relation head")
        if relation.get("policy") != POLICY:
            raise ValueError("Rust parity loaded a different relation policy")
        if relation.get("model_weights_cid") != EXPECTED_WEIGHTS_CID:
            raise ValueError("Rust relation binding names different model weights")
        if relation.get("tokenizer_cid") != EXPECTED_TOKENIZER_CID:
            raise ValueError("Rust relation binding names a different tokenizer")
        checkpoint = state_encoding.get("checkpoint")
        model_shape = state_encoding.get("model_shape")
        if not isinstance(checkpoint, dict) or not isinstance(model_shape, dict):
            raise ValueError("Rust parity state encoding omits checkpoint/model shape")
        if checkpoint.get("weights_cid") != EXPECTED_WEIGHTS_CID:
            raise ValueError("Rust parity state encoder loaded different model weights")
        if checkpoint.get("tokenizer_cid") != EXPECTED_TOKENIZER_CID:
            raise ValueError("Rust parity state encoder loaded a different tokenizer")
        if int(model_shape.get("dimension", -1)) != FROZEN_MODEL_CONFIG.hidden_size:
            raise ValueError("Rust parity state encoder hidden width differs")
        if state_encoding.get("model_weights_cid") != EXPECTED_WEIGHTS_CID:
            raise ValueError("Rust parity public state binding names different weights")
        if state_encoding.get("tokenizer_cid") != EXPECTED_TOKENIZER_CID:
            raise ValueError("Rust parity public state binding names a different tokenizer")
        if int(state_encoding.get("hidden_size", -1)) != FROZEN_MODEL_CONFIG.hidden_size:
            raise ValueError("Rust parity public state binding names a different hidden width")

        matching = [
            fixture
            for fixture in fixtures.values()
            if source.get("source_cid") == fixture["source_cid"]
            and report.get("question") == fixture["question"]
        ]
        if len(matching) != 1:
            raise ValueError("Rust parity source/question does not identify one fixture")
        fixture = matching[0]
        outcome = str(fixture["outcome"])
        if outcome in report_by_outcome:
            raise ValueError("Rust parity reports duplicate one outcome")
        rust_logits = _numeric_logits(evaluation.get("candidate_logits"))
        python_logits = _numeric_logits(fixture["candidate_logits"])
        if len(rust_logits) != len(python_logits):
            raise ValueError("Rust parity candidate count differs")
        maximum_delta = max(
            abs(left - right) for left, right in zip(rust_logits, python_logits)
        )
        tolerance = float(fixture["maximum_absolute_tolerance"])
        if tolerance != PARITY_MAX_ABSOLUTE_TOLERANCE or maximum_delta > tolerance:
            raise ValueError(
                "Rust relation parity exceeded tolerance: "
                f"delta={maximum_delta}, limit={tolerance}"
            )
        decision = evaluation.get("decision")
        selected_span = evaluation.get("selected_span_index")
        if decision not in OUTCOMES:
            raise ValueError("Rust parity decision is outside answer/abstain/conflict")
        if selected_span is not None and not isinstance(selected_span, int):
            raise ValueError("Rust parity selected span is not an integer or null")
        if decision != fixture["decision"]:
            raise ValueError("Rust parity decision differs from Python")
        if selected_span != fixture["selected_span_index"]:
            raise ValueError("Rust parity selected span differs from Python")
        report_by_outcome[outcome] = report
        admissions.append(
            {
                "outcome": outcome,
                "fixture_cid": fixture["fixture_cid"],
                "record_cid": fixture["record_cid"],
                "rust_report_cid": cid_bytes(canonical_json_bytes(report)),
                "maximum_absolute_candidate_logit_delta": maximum_delta,
                "decision": decision,
                "selected_span_index": selected_span,
            }
        )
    if set(report_by_outcome) != set(OUTCOMES):
        raise ValueError("Rust parity reports do not cover all three outcomes")

    copied_paths: list[str] = []
    for outcome in OUTCOMES:
        relative = f"preflight/rust-score-parity-{outcome}.json"
        atomic_write_json(root / relative, report_by_outcome[outcome])
        copied_paths.append(relative)
    admission = _canonical_with_cid(
        {
            "schema": PARITY_ADMISSION_SCHEMA,
            "issue": ISSUE,
            "policy": POLICY,
            "terminal": "PASS_THREE_OUTCOME_PYTHON_RUST_RELATION_PARITY",
            "preflight_artifact_cid": head["artifact_cid"],
            "maximum_absolute_tolerance": PARITY_MAX_ABSOLUTE_TOLERANCE,
            "reports": sorted(admissions, key=lambda item: OUTCOMES.index(item["outcome"])),
        },
        "admission_cid",
    )
    atomic_write_json(root / "preflight/score-parity-admission.json", admission)
    return {**admission, "copied_paths": copied_paths}


def _development_control_metrics(
    base_metrics: dict[str, Any],
    reversal_records: Sequence[dict[str, Any]],
    reversal_metrics: dict[str, Any],
    query_swap_records: Sequence[dict[str, Any]],
    query_swap_metrics: dict[str, Any],
) -> dict[str, Any]:
    base = {str(item["record_cid"]): item for item in base_metrics["records"]}
    reversal = {
        str(item["record_cid"]): item for item in reversal_metrics["records"]
    }
    order_correct = 0
    for record in reversal_records:
        original = base[str(record["base_record_cid"])]
        observed = reversal[str(record["record_cid"])]
        mapping = [int(index) for index in record["candidate_original_indices"]]
        logits_exact = all(
            observed["candidate_logits"][new_index]
            == original["candidate_logits"][old_index]
            for new_index, old_index in enumerate(mapping)
        )
        expected_selected = record.get("target_span_index")
        order_correct += int(
            logits_exact
            and observed["decision"] == original["decision"]
            and observed["selected_span_index"] == expected_selected
        )

    swaps = {str(item["record_cid"]): item for item in query_swap_metrics["records"]}
    relocation_correct = 0
    for record in query_swap_records:
        original = base[str(record["base_record_cid"])]
        observed = swaps[str(record["record_cid"])]
        base_exact = original["decision"] == original["target_outcome"] and (
            original["target_outcome"] != "answer"
            or original["selected_span_index"] == original["target_span_index"]
        )
        relocation_correct += int(
            base_exact
            and observed["source_cid"] == original["source_cid"]
            and record["source_cid"] == original["source_cid"]
            and observed["decision"] == "answer"
            and observed["selected_span_index"] == record["target_span_index"]
            and (
                original["decision"] != "answer"
                or original["selected_span_index"] != observed["selected_span_index"]
            )
        )
    if not reversal_records or not query_swap_records:
        raise RuntimeError("C1-SB2 development controls are empty")
    return {
        "order_equivariance": {
            "correct": order_correct,
            "total": len(reversal_records),
            "exact": order_correct == len(reversal_records),
        },
        "query_swap_relocation": {
            "correct": relocation_correct,
            "total": len(query_swap_records),
            "accuracy": relocation_correct / len(query_swap_records),
        },
    }


def _development_gate(
    metrics: dict[str, Any],
    controls: dict[str, Any],
    config: SourceRelationHeadConfig,
) -> bool:
    return (
        all(
            float(metrics["outcome"][outcome]["accuracy"])
            >= config.required_per_class_accuracy
            for outcome in OUTCOMES
        )
        and float(metrics["positive_relation_recall"]["accuracy"])
        >= config.required_positive_relation_recall
        and float(metrics["negative_relation_specificity"]["accuracy"])
        >= config.required_negative_relation_specificity
        and float(metrics["supported_copied_span"]["accuracy"])
        >= config.required_pointer_accuracy
        and float(controls["query_swap_relocation"]["accuracy"])
        >= config.required_query_swap_relocation
        and bool(controls["order_equivariance"]["exact"])
    )


def _run_full_fit(
    root: Path,
    *,
    predecessor: Path,
    predecessor_manifest: dict[str, Any],
    dataset: dict[str, Any],
    probes: dict[str, Any],
    dataset_manifest: dict[str, Any],
    run_contract: dict[str, Any],
    parity_admission: dict[str, Any],
    config: SourceRelationHeadConfig,
) -> dict[str, Any]:
    reversal_records = list(dataset["development_controls"]["reversal"])
    query_swap_records = list(dataset["development_controls"]["query_swap"])
    full_records = [
        *dataset["construction"],
        *dataset["development"],
        *reversal_records,
        *query_swap_records,
    ]
    feature_manifest = _extract_features(
        root / "features",
        predecessor=predecessor,
        predecessor_manifest=predecessor_manifest,
        records=full_records,
        population_cids={
            "dataset_cid": dataset["dataset_cid"],
            "dataset_manifest_cid": dataset_manifest["manifest_cid"],
        },
        config=config,
    )

    fit_marker = root / "fit-started.json"
    if fit_marker.exists():
        raise FileExistsError(
            "the sole C1-SB2 fit was already started; no rerun or resume is authorized"
        )
    atomic_write_json(
        fit_marker,
        {
            "schema": RUN_SCHEMA,
            "phase": "SOLE_512_STEP_FIT_STARTED",
            "run_contract_cid": run_contract["run_contract_cid"],
            "parity_admission_cid": parity_admission["admission_cid"],
            "feature_manifest_cid": feature_manifest["manifest_cid"],
        },
    )
    device = require_mps(config.seed)
    features = FrozenRelationFeatureStore(root / "features")
    construction = RelationDataset(dataset["construction"], features)
    development = RelationDataset(dataset["development"], features)
    reversal = RelationDataset(reversal_records, features)
    query_swap = RelationDataset(query_swap_records, features)
    construction.require_all_cells()
    development.require_all_cells()
    torch.manual_seed(config.seed)
    if hasattr(torch.mps, "manual_seed"):
        torch.mps.manual_seed(config.seed)
    model = R4SourceRelationHead().to(device)
    fit = _fit_head(
        model,
        construction,
        device=device,
        steps=config.optimizer_steps,
        batch_size=config.batch_size,
        config=config,
        phase="sole-512-step-fit",
    )
    development_raw = evaluate_relation_head(
        model,
        development,
        device=device,
        batch_size=config.batch_size,
        include_records=True,
    )
    reversal_raw = evaluate_relation_head(
        model,
        reversal,
        device=device,
        batch_size=config.batch_size,
        include_records=True,
    )
    query_swap_raw = evaluate_relation_head(
        model,
        query_swap,
        device=device,
        batch_size=config.batch_size,
        include_records=True,
    )
    controls = _development_control_metrics(
        development_raw,
        reversal_records,
        reversal_raw,
        query_swap_records,
        query_swap_raw,
    )
    development_metrics = {
        key: value for key, value in development_raw.items() if key != "records"
    }
    development_metrics["controls"] = controls
    passed = _development_gate(development_metrics, controls, config)
    preflight_result = json.loads(
        (root / "preflight/preflight-result.json").read_text(encoding="utf-8")
    )
    deterministic_fit = {key: value for key, value in fit.items() if key != "elapsed_seconds"}
    result = _canonical_with_cid(
        {
            "schema": TRAINING_RESULT_SCHEMA,
            "issue": ISSUE,
            "policy": POLICY,
            "terminal": (
                "SOURCE_RELATION_HEAD_FIT_COMPLETE_AWAITING_RUST_PRODUCT_REVEAL"
                if passed
                else "FAIL_SOURCE_RELATION_HEAD_DEVELOPMENT_GATE_STOP"
            ),
            "run_contract_cid": run_contract["run_contract_cid"],
            "dataset_cid": dataset["dataset_cid"],
            "dataset_manifest_cid": dataset_manifest["manifest_cid"],
            "feature_manifest_cid": feature_manifest["manifest_cid"],
            "model_weights_cid": EXPECTED_WEIGHTS_CID,
            "tokenizer_cid": EXPECTED_TOKENIZER_CID,
            "preflight_result_cid": preflight_result["result_cid"],
            "rust_score_parity_admission_cid": parity_admission["admission_cid"],
            "optimization": deterministic_fit,
            "development_metrics": development_metrics,
            "development_gate_passed": passed,
            "product_probe_commitments": dataset["product_probe_commitments"],
            "product_probe_file_cid": probes["product_probes_cid"],
            "product_behavior_status": "NOT_RUN",
        },
        "result_cid",
    )
    atomic_write_json(root / "source-relation-training-result.json", result)

    final_paths = [
        "source-relation-dataset.json",
        "source-relation-preflight.json",
        "product-probes.json",
        "shortcut-census.json",
        "source-relation-dataset-manifest.json",
        "source-relation-run-contract.json",
        "preflight/features/feature-index.json",
        "preflight/features/states.safetensors",
        "preflight/features/feature-manifest.json",
        "preflight/preflight-result.json",
        "preflight/preflight-head.json",
        "preflight/parity-index.json",
        "preflight/preflight-manifest.json",
        *[
            f"preflight/parity/{outcome}-source.txt"
            for outcome in OUTCOMES
        ],
        *[
            f"preflight/parity/{outcome}-python-score-fixture.json"
            for outcome in OUTCOMES
        ],
        *parity_admission["copied_paths"],
        "preflight/score-parity-admission.json",
        "features/feature-index.json",
        "features/states.safetensors",
        "features/feature-manifest.json",
        "fit-started.json",
        "source-relation-training-result.json",
    ]
    head: dict[str, Any] | None = None
    if passed:
        embedded_preflight = {
            "status": "PASS_MATCHED_TRANSFER_AND_THREE_RUST_PARITY_REPORTS",
            "preflight_result_cid": preflight_result["result_cid"],
            "rust_score_parity_admission_cid": parity_admission["admission_cid"],
        }
        head = _head_payload(
            model,
            dataset=dataset,
            run_contract_cid=run_contract["run_contract_cid"],
            training_result_cid=result["result_cid"],
            preflight=embedded_preflight,
            development_metrics=development_metrics,
        )
        atomic_write_json(root / "source-relation-head.json", head)
        final_paths.append("source-relation-head.json")

    final_manifest = write_bound_manifest(
        root / "source-relation-final-manifest.json",
        {
            "schema": FINAL_MANIFEST_SCHEMA,
            "issue": ISSUE,
            "policy": POLICY,
            "terminal": result["terminal"],
            "run_contract_cid": run_contract["run_contract_cid"],
            "dataset_cid": dataset["dataset_cid"],
            "dataset_manifest_cid": dataset_manifest["manifest_cid"],
            "feature_manifest_cid": feature_manifest["manifest_cid"],
            "training_result_cid": result["result_cid"],
            "relation_head_artifact_cid": None if head is None else head["artifact_cid"],
            "development_gate_passed": passed,
            "product_behavior_status": "NOT_RUN" if passed else "BLOCKED_BY_DEVELOPMENT",
        },
        artifact_root=root,
        relative_paths=final_paths,
    )
    return {
        "terminal": result["terminal"],
        "optimizer_steps_completed": config.optimizer_steps,
        "development_metrics": development_metrics,
        "relation_head_artifact": None
        if head is None
        else str(root / "source-relation-head.json"),
        "relation_head_artifact_cid": None if head is None else head["artifact_cid"],
        "training_result_cid": result["result_cid"],
        "final_manifest_cid": final_manifest["manifest_cid"],
        "product_behavior_status": "NOT_RUN" if passed else "BLOCKED_BY_DEVELOPMENT",
    }


def train_source_relation_head(
    root: Path,
    *,
    predecessor: Path,
    rust_score_parity: Sequence[Path] | None = None,
    config: SourceRelationHeadConfig = SourceRelationHeadConfig(),
) -> dict[str, Any]:
    """Advance C1-SB2 without feature-extracting, scoring, or evaluating product probes."""
    config.validate()
    root = root.expanduser().resolve()
    predecessor = predecessor.expanduser().resolve()
    if (
        root == predecessor
        or predecessor in root.parents
        or root in predecessor.parents
    ):
        raise ValueError("relation-head output must be disjoint from immutable #1017")
    if (root / "source-relation-final-manifest.json").exists():
        raise FileExistsError("the sole C1-SB2 relation-head fit is already complete")

    predecessor_manifest = _validated_predecessor(predecessor)
    (
        dataset,
        preflight,
        probes,
        _census,
        dataset_manifest,
        run_contract,
    ) = _prepare_contract(
        root,
        predecessor_manifest=predecessor_manifest,
        config=config,
    )
    preflight_manifest_path = root / "preflight/preflight-manifest.json"
    if not preflight_manifest_path.exists():
        result = _run_preflight(
            root,
            predecessor=predecessor,
            predecessor_manifest=predecessor_manifest,
            dataset=dataset,
            preflight=preflight,
            dataset_manifest=dataset_manifest,
            run_contract=run_contract,
            config=config,
        )
        if rust_score_parity is not None:
            result["note"] = (
                "preflight was created in this invocation; run all three emitted Rust "
                "parity commands and invoke this subcommand again with their reports"
            )
        return result

    preflight_manifest = verify_bound_manifest(
        preflight_manifest_path, artifact_root=root
    )
    if preflight_manifest.get("terminal") != "AWAITING_THREE_RUST_RELATION_PARITY_REPORTS":
        return {
            "terminal": preflight_manifest.get("terminal", "PREFLIGHT_STOPPED"),
            "preflight_manifest_cid": preflight_manifest["manifest_cid"],
        }
    if rust_score_parity is None:
        parity_index = json.loads(
            (root / "preflight/parity-index.json").read_text(encoding="utf-8")
        )
        fixtures = []
        for item in parity_index["fixtures"]:
            outcome = str(item["outcome"])
            fixture_path = (
                root / f"preflight/parity/{outcome}-python-score-fixture.json"
            )
            fixture = json.loads(fixture_path.read_text(encoding="utf-8"))
            fixtures.append(
                {
                    "outcome": outcome,
                    "source": str(root / fixture["source_path"]),
                    "question": fixture["question"],
                    "fixture": str(fixture_path),
                    "fixture_cid": fixture["fixture_cid"],
                }
            )
        return {
            "terminal": "AWAITING_THREE_RUST_RELATION_PARITY_REPORTS",
            "preflight_artifact": str(root / "preflight/preflight-head.json"),
            "preflight_artifact_cid": parity_index["preflight_artifact_cid"],
            "parity_fixtures": fixtures,
        }

    parity_admission = _admit_rust_score_parity(root, rust_score_parity)
    return _run_full_fit(
        root,
        predecessor=predecessor,
        predecessor_manifest=predecessor_manifest,
        dataset=dataset,
        probes=probes,
        dataset_manifest=dataset_manifest,
        run_contract=run_contract,
        parity_admission=parity_admission,
        config=config,
    )
