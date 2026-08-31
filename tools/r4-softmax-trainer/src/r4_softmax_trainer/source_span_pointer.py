"""Offline C1-SB1 frozen-state extraction and source-span pointer fitting."""

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
    verify_manifest_envelope,
    write_bound_manifest,
)
from .source_span_data import (
    MAXIMUM_SOURCE_SPANS,
    OUTCOMES,
    QUESTION_POLICY,
    SENTENCE_POLICY,
    build_source_span_population,
)
from .train import require_mps


ISSUE = 954
POLICY = "R4SourceSpanPointerV1"
HEAD_SCHEMA = "uor-r4.source-span-pointer/1"
DATASET_MANIFEST_SCHEMA = "uor-r4.source-span-pointer-dataset-manifest/1"
FEATURE_INDEX_SCHEMA = "uor-r4.source-span-pointer-feature-index/1"
FEATURE_MANIFEST_SCHEMA = "uor-r4.source-span-pointer-feature-manifest/1"
RUN_SCHEMA = "uor-r4.source-span-pointer-run/1"
PREFLIGHT_RESULT_SCHEMA = "uor-r4.source-span-pointer-preflight/1"
PYTHON_SCORE_FIXTURE_SCHEMA = "uor-r4.source-span-pointer-python-score-fixture/1"
PARITY_ADMISSION_SCHEMA = "uor-r4.source-span-pointer-parity-admission/1"
TRAINING_RESULT_SCHEMA = "uor-r4.source-span-pointer-training-result/1"
FINAL_MANIFEST_SCHEMA = "uor-r4.source-span-pointer-final-manifest/1"
RUST_GROUNDED_REPORT_SCHEMA = "uor-r4.grounded-answer/2"
EXPECTED_WEIGHTS_CID = (
    "blake3:c5bf31aa97a567b3aaad4461ce2fac9cebc12b0a38becb6d02d21b43b493bf5d"
)
EXPECTED_TOKENIZER_CID = (
    "blake3:3f42bcfce7728512076549c63b88387e13c8156fe35c0f91d9b112439f3739cc"
)
OUTCOME_TO_ID = {name: index for index, name in enumerate(OUTCOMES)}
PARITY_MAX_ABSOLUTE_TOLERANCE = 0.01


@dataclass(frozen=True, slots=True)
class SourceSpanPointerConfig:
    """The sole no-sweep C1-SB1 optimization and work contract."""

    seed: int = 9_541
    feature_batch_size: int = 128
    preflight_records_per_class: int = 4
    preflight_optimizer_steps: int = 256
    optimizer_steps: int = 256
    batch_size: int = 128
    learning_rate: float = 0.025
    adam_beta1: float = 0.9
    adam_beta2: float = 0.999
    adam_epsilon: float = 1e-8
    progress_interval: int = 16
    required_per_class_accuracy: float = 0.95
    required_pointer_accuracy: float = 0.95

    def validate(self) -> None:
        if self != SourceSpanPointerConfig():
            raise ValueError("C1-SB1 exposes one frozen pointer fit, not a sweep")
        if self.batch_size != 128 or self.optimizer_steps != 256 or self.seed != 9_541:
            raise AssertionError("C1-SB1 optimizer contract drifted")

    def as_contract(self) -> dict[str, Any]:
        self.validate()
        return asdict(self)


@dataclass(slots=True)
class PointerBatch:
    subject_states: Tensor
    subject_mask: Tensor
    candidate_states: Tensor
    candidate_token_mask: Tensor
    candidate_mask: Tensor
    outcomes: Tensor
    target_spans: Tensor


@dataclass(slots=True)
class PointerOutput:
    candidate_scores: Tensor
    outcome_logits: Tensor
    top_candidate_indices: Tensor


class R4SourceSpanPointer(nn.Module):
    """Positive diagonal weighted cosine plus explicit three-way logits."""

    def __init__(self) -> None:
        super().__init__()
        width = FROZEN_MODEL_CONFIG.hidden_size
        inverse_softplus_one = math.log(math.expm1(1.0))
        inverse_softplus_ten = math.log(math.expm1(10.0))
        self.raw_state_weights = nn.Parameter(
            torch.full((width,), inverse_softplus_one, dtype=torch.float32)
        )
        self.raw_score_scale = nn.Parameter(
            torch.tensor(inverse_softplus_ten, dtype=torch.float32)
        )
        self.class_biases = nn.Parameter(torch.zeros(len(OUTCOMES), dtype=torch.float32))

    def state_weights(self) -> Tensor:
        return F.softplus(self.raw_state_weights)

    def score_scale(self) -> Tensor:
        return F.softplus(self.raw_score_scale)

    def forward(self, batch: PointerBatch) -> PointerOutput:
        weights = self.state_weights()
        square_root_weights = torch.sqrt(weights)
        subject = batch.subject_states.float() * square_root_weights
        candidates = batch.candidate_states.float() * square_root_weights
        numerators = torch.einsum("bsh,bcth->bcst", subject, candidates)
        subject_norms = torch.linalg.vector_norm(subject, dim=-1)
        candidate_norms = torch.linalg.vector_norm(candidates, dim=-1)
        # Padded rows are exactly zero. Give only those masked rows a neutral
        # denominator before division; their scores are removed below. Dividing
        # first and masking later would create NaNs in the autograd graph.
        subject_norms = subject_norms.masked_fill(~batch.subject_mask, 1.0)
        candidate_norms = candidate_norms.masked_fill(~batch.candidate_token_mask, 1.0)
        denominators = subject_norms[:, None, :, None] * candidate_norms[:, :, None, :]
        cosine = numerators / denominators
        cosine = cosine.clamp(-1.0, 1.0)
        cosine = cosine.masked_fill(
            ~batch.candidate_token_mask[:, :, None, :], float("-inf")
        )
        subject_best = cosine.max(dim=-1).values
        subject_best = subject_best.masked_fill(~batch.subject_mask[:, None, :], 0.0)
        subject_counts = batch.subject_mask.sum(dim=-1).clamp_min(1).float()
        candidate_scores = subject_best.sum(dim=-1) / subject_counts[:, None]
        candidate_scores = candidate_scores.masked_fill(~batch.candidate_mask, float("-inf"))

        ordered_scores = torch.sort(candidate_scores, dim=-1, descending=True).values
        top_candidate_indices = candidate_scores.argmax(dim=-1)
        if ordered_scores.shape[1] < 2:
            second = torch.full_like(ordered_scores[:, 0], float("-inf"))
        else:
            second = ordered_scores[:, 1]
        scale = self.score_scale()
        outcome_logits = torch.stack(
            (
                scale * ordered_scores[:, 0] + self.class_biases[0],
                self.class_biases[1].expand_as(ordered_scores[:, 0]),
                scale * second + self.class_biases[2],
            ),
            dim=-1,
        )
        return PointerOutput(
            candidate_scores=candidate_scores,
            outcome_logits=outcome_logits,
            top_candidate_indices=top_candidate_indices,
        )


def pointer_loss(output: PointerOutput, batch: PointerBatch) -> Tensor:
    outcome = F.cross_entropy(output.outcome_logits, batch.outcomes)
    supported = batch.outcomes == OUTCOME_TO_ID["answer"]
    safe_targets = batch.target_spans.clamp_min(0)
    pointer_per_record = -F.log_softmax(output.candidate_scores, dim=-1).gather(
        dim=-1, index=safe_targets[:, None]
    )[:, 0]
    supported_float = supported.to(dtype=pointer_per_record.dtype)
    pointer = (pointer_per_record * supported_float).sum() / supported_float.sum().clamp_min(1.0)
    return outcome + pointer


def safety_decisions(logits: Tensor) -> Tensor:
    """Apply the frozen conflict, abstain, answer tie order."""
    answer = logits[:, OUTCOME_TO_ID["answer"]]
    abstain = logits[:, OUTCOME_TO_ID["abstain"]]
    conflict = logits[:, OUTCOME_TO_ID["conflict"]]
    decisions = torch.full_like(answer, OUTCOME_TO_ID["answer"], dtype=torch.long)
    abstain_wins = (abstain >= answer) & (abstain >= conflict)
    decisions[abstain_wins] = OUTCOME_TO_ID["abstain"]
    conflict_wins = (conflict >= answer) & (conflict >= abstain)
    decisions[conflict_wins] = OUTCOME_TO_ID["conflict"]
    return decisions


def _text_cid(text: str) -> str:
    return cid_bytes(text.encode("utf-8"))


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
            raise ValueError(f"existing C1-SB1 artifact differs: {path}")
        return
    atomic_write(path, encoded)


def _validated_predecessor(predecessor: Path) -> dict[str, Any]:
    manifest = verify_bound_manifest(
        predecessor / "export-manifest.json", artifact_root=predecessor
    )
    if manifest.get("model_contract") != FROZEN_MODEL_CONFIG.as_contract():
        raise ValueError("source-span predecessor is not the immutable six-layer #1017 model")
    if manifest.get("weights_cid") != EXPECTED_WEIGHTS_CID:
        raise ValueError("source-span predecessor weights are not the frozen #1017 weights")
    if manifest.get("tokenizer_cid") != EXPECTED_TOKENIZER_CID:
        raise ValueError("source-span predecessor tokenizer is not the frozen #1017 tokenizer")
    if cid_file(predecessor / "model.safetensors") != EXPECTED_WEIGHTS_CID:
        raise ValueError("#1017 model file CID does not reproduce")
    if cid_file(predecessor / "tokenizer.json") != EXPECTED_TOKENIZER_CID:
        raise ValueError("#1017 tokenizer file CID does not reproduce")
    return manifest


def _training_texts(dataset: dict[str, Any]) -> list[str]:
    texts: dict[str, str] = {}
    for record in [*dataset["construction"], *dataset["development"]]:
        values = [record["subject"]]
        values.extend(span["text"] for span in record["sentence_spans"])
        for text in values:
            text = str(text)
            text_cid = _text_cid(text)
            prior = texts.setdefault(text_cid, text)
            if prior != text:
                raise RuntimeError("BLAKE3 collision in source-span text inventory")
    return [texts[text_cid] for text_cid in sorted(texts)]


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
    remaining = max(0, total - completed)
    eta = rate * remaining
    suffix = "" if loss is None else f" loss={loss:.6f}"
    print(
        f"pointer_phase={phase} completed={completed}/{total}{suffix} "
        f"elapsed_seconds={elapsed:.1f} eta_seconds={eta:.1f}",
        flush=True,
    )


@torch.no_grad()
def _extract_features(
    root: Path,
    *,
    predecessor: Path,
    predecessor_manifest: dict[str, Any],
    dataset: dict[str, Any],
    dataset_manifest: dict[str, Any],
    config: SourceSpanPointerConfig,
) -> dict[str, Any]:
    feature_manifest_path = root / "features/feature-manifest.json"
    if feature_manifest_path.exists():
        return verify_bound_manifest(feature_manifest_path, artifact_root=root / "features")

    device = require_mps(config.seed)
    tokenizer = Tokenizer.from_file(str(predecessor / "tokenizer.json"))
    texts = _training_texts(dataset)
    encoded: list[list[int]] = []
    for text in texts:
        token_ids = tokenizer.encode(text, add_special_tokens=False).ids
        if not token_ids:
            raise ValueError("pointer content text encoded to zero tokens")
        if len(token_ids) + 1 > FROZEN_MODEL_CONFIG.max_position_embeddings:
            raise ValueError("pointer content text exceeds the frozen #1017 context")
        encoded.append(token_ids)
    maximum_tokens = max(map(len, encoded))
    states = torch.zeros(
        (len(texts), maximum_tokens, FROZEN_MODEL_CONFIG.hidden_size),
        dtype=torch.float32,
    )
    lengths = torch.tensor([len(token_ids) for token_ids in encoded], dtype=torch.int64)

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
            content = hidden[lane, 1 : 1 + len(token_ids)]
            if content.shape != (len(token_ids), FROZEN_MODEL_CONFIG.hidden_size):
                raise RuntimeError("#1017 content-state extraction shape differs")
            if not bool(torch.isfinite(content).all()):
                raise RuntimeError("#1017 content-state extraction produced nonfinite values")
            if not bool((torch.linalg.vector_norm(content, dim=-1) > 0).all()):
                raise RuntimeError("#1017 content-state extraction produced a zero state")
            states[base + lane, : len(token_ids)] = content
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
            "text_cid": _text_cid(text),
            "token_ids": encoded[row],
            "token_count": len(encoded[row]),
        }
        for row, text in enumerate(texts)
    ]
    index = {
        "schema": FEATURE_INDEX_SCHEMA,
        "model_weights_cid": EXPECTED_WEIGHTS_CID,
        "tokenizer_cid": EXPECTED_TOKENIZER_CID,
        "state_definition": (
            "every non-BOS content token's final RMS-normalized residual after all six "
            "immutable #1017 R4/Spin causal-softmax layers; each text encoded independently"
        ),
        "hidden_size": FROZEN_MODEL_CONFIG.hidden_size,
        "maximum_tokens": maximum_tokens,
        "entries": entries,
    }
    feature_root = root / "features"
    feature_root.mkdir(parents=True, exist_ok=True)
    index_path = feature_root / "feature-index.json"
    atomic_write_json(index_path, index)
    state_path = feature_root / "states.safetensors"
    temporary = feature_root / ".states.safetensors.part"
    save_file(
        {"states": states.contiguous(), "lengths": lengths.contiguous()},
        str(temporary),
        metadata={
            "schema": FEATURE_INDEX_SCHEMA,
            "model_weights_cid": EXPECTED_WEIGHTS_CID,
            "tokenizer_cid": EXPECTED_TOKENIZER_CID,
        },
    )
    os.replace(temporary, state_path)
    return write_bound_manifest(
        feature_manifest_path,
        {
            "schema": FEATURE_MANIFEST_SCHEMA,
            "issue": ISSUE,
            "policy": POLICY,
            "dataset_cid": dataset["dataset_cid"],
            "dataset_manifest_cid": dataset_manifest["manifest_cid"],
            "predecessor_export_manifest_cid": predecessor_manifest["manifest_cid"],
            "model_weights_cid": EXPECTED_WEIGHTS_CID,
            "tokenizer_cid": EXPECTED_TOKENIZER_CID,
            "text_count": len(texts),
            "maximum_tokens": maximum_tokens,
        },
        artifact_root=feature_root,
        relative_paths=["feature-index.json", "states.safetensors"],
    )


class FrozenFeatureStore:
    """Read-only final-state table keyed by exact UTF-8 text CID."""

    def __init__(self, root: Path) -> None:
        verify_bound_manifest(root / "feature-manifest.json", artifact_root=root)
        index = json.loads((root / "feature-index.json").read_text(encoding="utf-8"))
        if index.get("schema") != FEATURE_INDEX_SCHEMA:
            raise ValueError("source-span feature index schema differs")
        tensors = load_file(str(root / "states.safetensors"), device="cpu")
        self.states = tensors["states"].float().contiguous()
        self.lengths = tensors["lengths"].to(dtype=torch.long).contiguous()
        self.rows: dict[str, int] = {}
        for entry in index["entries"]:
            row = int(entry["row"])
            text_cid = str(entry["text_cid"])
            if text_cid in self.rows:
                raise ValueError("duplicate source-span feature text CID")
            if int(entry["token_count"]) != int(self.lengths[row]):
                raise ValueError("source-span feature length/index mismatch")
            content = self.states[row, : int(self.lengths[row])]
            if not bool(torch.isfinite(content).all()) or not bool(
                (torch.linalg.vector_norm(content, dim=-1) > 0).all()
            ):
                raise ValueError("source-span feature content states are invalid")
            self.rows[text_cid] = row
        if self.states.ndim != 3 or self.states.shape[2] != FROZEN_MODEL_CONFIG.hidden_size:
            raise ValueError("source-span state tensor shape differs")
        if self.states.shape[0] != len(self.rows) or self.lengths.shape != (
            self.states.shape[0],
        ):
            raise ValueError("source-span feature inventory shape differs")

    def get(self, text: str) -> Tensor:
        try:
            row = self.rows[_text_cid(text)]
        except KeyError as error:
            raise KeyError("text is absent from frozen source-span feature store") from error
        length = int(self.lengths[row])
        return self.states[row, :length]


class PointerDataset:
    """Records plus immutable base states, assembled into bounded padded batches."""

    def __init__(self, records: Sequence[dict[str, Any]], features: FrozenFeatureStore) -> None:
        self.records = list(records)
        if not self.records:
            raise ValueError("source-span pointer dataset is empty")
        self.features = features
        self.class_indices: dict[str, list[int]] = {name: [] for name in OUTCOMES}
        for index, record in enumerate(self.records):
            outcome = str(record["target_outcome"])
            if outcome not in self.class_indices:
                raise ValueError(f"unknown pointer outcome: {outcome}")
            self.class_indices[outcome].append(index)
        if any(not indices for indices in self.class_indices.values()):
            raise ValueError("pointer dataset is not three-way populated")

    def preflight_indices(self, per_class: int) -> list[int]:
        selected: list[int] = []
        for outcome in OUTCOMES:
            ordered = sorted(
                self.class_indices[outcome],
                key=lambda index: str(self.records[index]["record_cid"]),
            )
            if len(ordered) < per_class:
                raise ValueError("pointer dataset cannot populate balanced preflight")
            selected.extend(ordered[:per_class])
        return selected

    def deterministic_indices(self, *, seed: int, step: int, batch_size: int) -> list[int]:
        selected: list[int] = []
        for lane in range(batch_size):
            outcome = OUTCOMES[lane % len(OUTCOMES)]
            population = self.class_indices[outcome]
            material = struct.pack(">QQQ", seed, step, lane)
            offset = int.from_bytes(blake3(material).digest(), "big") % len(population)
            selected.append(population[offset])
        return selected

    def batch(self, indices: Sequence[int], *, device: torch.device) -> PointerBatch:
        selected = [self.records[index] for index in indices]
        subject_states = [self.features.get(str(record["subject"])) for record in selected]
        candidate_states = [
            [self.features.get(str(span["text"])) for span in record["sentence_spans"]]
            for record in selected
        ]
        maximum_subject_tokens = max(state.shape[0] for state in subject_states)
        maximum_candidates = max(len(states) for states in candidate_states)
        maximum_candidate_tokens = max(
            state.shape[0] for states in candidate_states for state in states
        )
        batch_size = len(selected)
        hidden = FROZEN_MODEL_CONFIG.hidden_size
        subjects = torch.zeros(
            (batch_size, maximum_subject_tokens, hidden), dtype=torch.float32
        )
        subject_mask = torch.zeros(
            (batch_size, maximum_subject_tokens), dtype=torch.bool
        )
        candidates = torch.zeros(
            (batch_size, maximum_candidates, maximum_candidate_tokens, hidden),
            dtype=torch.float32,
        )
        candidate_token_mask = torch.zeros(
            (batch_size, maximum_candidates, maximum_candidate_tokens), dtype=torch.bool
        )
        candidate_mask = torch.zeros(
            (batch_size, maximum_candidates), dtype=torch.bool
        )
        outcomes = torch.empty(batch_size, dtype=torch.long)
        target_spans = torch.full((batch_size,), -1, dtype=torch.long)
        for lane, record in enumerate(selected):
            subject = subject_states[lane]
            subjects[lane, : subject.shape[0]] = subject
            subject_mask[lane, : subject.shape[0]] = True
            for candidate_index, state in enumerate(candidate_states[lane]):
                candidates[lane, candidate_index, : state.shape[0]] = state
                candidate_token_mask[lane, candidate_index, : state.shape[0]] = True
                candidate_mask[lane, candidate_index] = True
            outcome = str(record["target_outcome"])
            outcomes[lane] = OUTCOME_TO_ID[outcome]
            if outcome == "answer":
                target = record["target_span_index"]
                if not isinstance(target, int) or not 0 <= target < len(candidate_states[lane]):
                    raise ValueError("supported pointer target is outside candidate spans")
                target_spans[lane] = target
        return PointerBatch(
            subject_states=subjects.to(device),
            subject_mask=subject_mask.to(device),
            candidate_states=candidates.to(device),
            candidate_token_mask=candidate_token_mask.to(device),
            candidate_mask=candidate_mask.to(device),
            outcomes=outcomes.to(device),
            target_spans=target_spans.to(device),
        )


@torch.no_grad()
def evaluate_pointer(
    model: R4SourceSpanPointer,
    dataset: PointerDataset,
    *,
    device: torch.device,
    batch_size: int,
    indices: Sequence[int] | None = None,
) -> dict[str, Any]:
    model.eval()
    selected = list(range(len(dataset.records))) if indices is None else list(indices)
    outcome_correct = {name: 0 for name in OUTCOMES}
    outcome_total = {name: 0 for name in OUTCOMES}
    pointer_correct = 0
    pointer_total = 0
    loss_sum = 0.0
    examples = 0
    for base in range(0, len(selected), batch_size):
        batch_indices = selected[base : base + batch_size]
        batch = dataset.batch(batch_indices, device=device)
        output = model(batch)
        loss = pointer_loss(output, batch)
        predictions = safety_decisions(output.outcome_logits)
        loss_sum += float(loss.detach().cpu()) * len(batch_indices)
        examples += len(batch_indices)
        predicted_outcomes = predictions.detach().cpu().tolist()
        predicted_spans = output.top_candidate_indices.detach().cpu().tolist()
        for lane, record_index in enumerate(batch_indices):
            record = dataset.records[record_index]
            outcome = str(record["target_outcome"])
            expected = OUTCOME_TO_ID[outcome]
            outcome_total[outcome] += 1
            outcome_correct[outcome] += int(int(predicted_outcomes[lane]) == expected)
            if outcome == "answer":
                pointer_total += 1
                pointer_correct += int(
                    int(predicted_spans[lane]) == int(record["target_span_index"])
                )
    if not examples or not pointer_total:
        raise RuntimeError("pointer evaluation population is incomplete")
    per_class = {
        outcome: {
            "correct": outcome_correct[outcome],
            "total": outcome_total[outcome],
            "accuracy": outcome_correct[outcome] / outcome_total[outcome],
        }
        for outcome in OUTCOMES
    }
    return {
        "mean_combined_loss": loss_sum / examples,
        "outcome": per_class,
        "supported_span_pointer": {
            "correct": pointer_correct,
            "total": pointer_total,
            "accuracy": pointer_correct / pointer_total,
        },
    }


def _development_gate(metrics: dict[str, Any], config: SourceSpanPointerConfig) -> bool:
    return all(
        float(metrics["outcome"][outcome]["accuracy"])
        >= config.required_per_class_accuracy
        for outcome in OUTCOMES
    ) and (
        float(metrics["supported_span_pointer"]["accuracy"])
        >= config.required_pointer_accuracy
    )


def _fit_head(
    model: R4SourceSpanPointer,
    dataset: PointerDataset,
    *,
    device: torch.device,
    steps: int,
    batch_size: int,
    config: SourceSpanPointerConfig,
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
        output = model(batch)
        loss = pointer_loss(output, batch)
        loss.backward()
        optimizer.step()
        report_step = initial_loss is None or step % config.progress_interval == 0 or step == steps
        if report_step:
            final_loss = float(loss.detach().cpu())
            if not math.isfinite(final_loss):
                raise RuntimeError("source-span pointer fit produced a nonfinite loss")
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
        "initial_combined_loss": initial_loss,
        "final_combined_loss": final_loss,
        "elapsed_seconds": time.monotonic() - started,
    }


def _finite_f32(value: Tensor, label: str, *, positive: bool = False) -> float:
    scalar = float(value.detach().to(device="cpu", dtype=torch.float32).item())
    if not math.isfinite(scalar) or (positive and scalar <= 0.0):
        raise RuntimeError(f"pointer {label} is outside its finite contract")
    return scalar


def _head_payload(
    model: R4SourceSpanPointer,
    *,
    dataset: dict[str, Any],
    run_contract_cid: str,
    training_result_cid: str,
    preflight: dict[str, Any],
    development_metrics: dict[str, Any],
) -> dict[str, Any]:
    weights_tensor = model.state_weights().detach().to(device="cpu", dtype=torch.float32)
    weights = [float(value) for value in weights_tensor.tolist()]
    if len(weights) != FROZEN_MODEL_CONFIG.hidden_size or any(
        not math.isfinite(value) or value <= 0.0 for value in weights
    ):
        raise RuntimeError("pointer state weights violate the positive 288-lane contract")
    biases = model.class_biases.detach().to(device="cpu", dtype=torch.float32)
    value: dict[str, Any] = {
        "schema": HEAD_SCHEMA,
        "policy": POLICY,
        "issue": ISSUE,
        "model_weights_cid": EXPECTED_WEIGHTS_CID,
        "tokenizer_cid": EXPECTED_TOKENIZER_CID,
        "hidden_size": FROZEN_MODEL_CONFIG.hidden_size,
        "state_weights": weights,
        "score_scale": _finite_f32(model.score_scale(), "score scale", positive=True),
        "answer_bias": _finite_f32(biases[0], "answer bias"),
        "abstain_bias": _finite_f32(biases[1], "abstain bias"),
        "conflict_bias": _finite_f32(biases[2], "conflict bias"),
        "maximum_source_spans": MAXIMUM_SOURCE_SPANS,
        "question_policy": QUESTION_POLICY,
        "sentence_policy": SENTENCE_POLICY,
        "dataset_cid": dataset["dataset_cid"],
        "split_policy_cid": dataset["split_policy_cid"],
        "run_contract_cid": run_contract_cid,
        "training_result_cid": training_result_cid,
        "preflight": preflight,
        "development_metrics": development_metrics,
        "product_probe_commitments": dataset["product_probe_commitments"],
    }
    return _canonical_with_cid(value, "artifact_cid")


@torch.no_grad()
def _score_record(
    model: R4SourceSpanPointer,
    dataset: PointerDataset,
    *,
    record_index: int,
    device: torch.device,
) -> dict[str, Any]:
    model.eval()
    record = dataset.records[record_index]
    batch = dataset.batch([record_index], device=device)
    output = model(batch)
    candidate_count = len(record["sentence_spans"])
    scores = output.candidate_scores[0, :candidate_count].detach().cpu().float()
    logits = output.outcome_logits[0].detach().cpu().float()
    if not bool(torch.isfinite(scores).all()) or not bool(torch.isfinite(logits).all()):
        raise RuntimeError("pointer score fixture produced nonfinite values")
    decision_id = int(safety_decisions(logits[None, :])[0])
    decision = OUTCOMES[decision_id]
    top_candidate = int(output.top_candidate_indices[0].detach().cpu())
    selected_span_index = top_candidate if decision == "answer" else None
    return {
        "candidate_scores": [float(value) for value in scores.tolist()],
        "logits": {
            "answer": float(logits[OUTCOME_TO_ID["answer"]]),
            "abstain": float(logits[OUTCOME_TO_ID["abstain"]]),
            "conflict": float(logits[OUTCOME_TO_ID["conflict"]]),
        },
        "decision": decision,
        "selected_span_index": selected_span_index,
    }


def _run_preflight(
    root: Path,
    *,
    dataset_value: dict[str, Any],
    dataset_manifest: dict[str, Any],
    feature_manifest: dict[str, Any],
    run_contract: dict[str, Any],
    config: SourceSpanPointerConfig,
) -> dict[str, Any]:
    device = require_mps(config.seed)
    features = FrozenFeatureStore(root / "features")
    construction = PointerDataset(dataset_value["construction"], features)
    selected = construction.preflight_indices(config.preflight_records_per_class)
    torch.manual_seed(config.seed)
    if hasattr(torch.mps, "manual_seed"):
        torch.mps.manual_seed(config.seed)
    model = R4SourceSpanPointer().to(device)
    fit = _fit_head(
        model,
        construction,
        device=device,
        steps=config.preflight_optimizer_steps,
        batch_size=len(selected),
        config=config,
        phase="12-record-overfit",
        fixed_indices=selected,
    )
    metrics = evaluate_pointer(
        model,
        construction,
        device=device,
        batch_size=len(selected),
        indices=selected,
    )
    passed = all(
        metrics["outcome"][outcome]["correct"]
        == metrics["outcome"][outcome]["total"]
        for outcome in OUTCOMES
    ) and (
        metrics["supported_span_pointer"]["correct"]
        == metrics["supported_span_pointer"]["total"]
    )
    deterministic_fit = {key: value for key, value in fit.items() if key != "elapsed_seconds"}
    preflight_result = _canonical_with_cid(
        {
            "schema": PREFLIGHT_RESULT_SCHEMA,
            "issue": ISSUE,
            "policy": POLICY,
            "terminal": (
                "PASS_12_RECORD_BALANCED_OVERFIT_AWAIT_RUST_SCORE_PARITY"
                if passed
                else "FAIL_12_RECORD_BALANCED_OVERFIT_STOP"
            ),
            "run_contract_cid": run_contract["run_contract_cid"],
            "dataset_cid": dataset_value["dataset_cid"],
            "feature_manifest_cid": feature_manifest["manifest_cid"],
            "record_cids": [construction.records[index]["record_cid"] for index in selected],
            "optimization": deterministic_fit,
            "metrics": metrics,
            "passed": passed,
        },
        "result_cid",
    )
    preflight_root = root / "preflight"
    preflight_root.mkdir(parents=True, exist_ok=True)
    atomic_write_json(preflight_root / "preflight-result.json", preflight_result)
    if not passed:
        write_bound_manifest(
            preflight_root / "preflight-manifest.json",
            {
                "schema": PREFLIGHT_RESULT_SCHEMA,
                "terminal": preflight_result["terminal"],
                "run_contract_cid": run_contract["run_contract_cid"],
                "preflight_result_cid": preflight_result["result_cid"],
            },
            artifact_root=root,
            relative_paths=[
                "source-span-dataset.json",
                "product-probes.json",
                "source-span-dataset-manifest.json",
                "source-span-run-contract.json",
                "features/feature-index.json",
                "features/states.safetensors",
                "features/feature-manifest.json",
                "preflight/preflight-result.json",
            ],
        )
        return {
            "terminal": preflight_result["terminal"],
            "preflight_result_cid": preflight_result["result_cid"],
        }

    embedded_preflight = {
        "status": "PASS_12_RECORD_BALANCED_OVERFIT",
        "record_cids": preflight_result["record_cids"],
        "optimizer_steps": config.preflight_optimizer_steps,
        "metrics": metrics,
        "preflight_result_cid": preflight_result["result_cid"],
        "rust_score_parity": "AWAITING",
    }
    preflight_head = _head_payload(
        model,
        dataset=dataset_value,
        run_contract_cid=run_contract["run_contract_cid"],
        training_result_cid=preflight_result["result_cid"],
        preflight=embedded_preflight,
        development_metrics={"status": "NOT_RUN_BEFORE_SOLE_256_STEP_FIT"},
    )
    atomic_write_json(preflight_root / "preflight-head.json", preflight_head)

    parity_index = next(
        index
        for index in selected
        if construction.records[index]["target_outcome"] == "answer"
    )
    parity_record = construction.records[parity_index]
    parity_source = str(parity_record["source"])
    parity_source_path = preflight_root / "parity-source.txt"
    atomic_write(parity_source_path, parity_source.encode("utf-8"))
    evaluation = _score_record(
        model,
        construction,
        record_index=parity_index,
        device=device,
    )
    if (
        evaluation["decision"] != "answer"
        or evaluation["selected_span_index"] != parity_record["target_span_index"]
    ):
        raise RuntimeError("passing pointer preflight did not yield an answer parity record")
    fixture = _canonical_with_cid(
        {
            "schema": PYTHON_SCORE_FIXTURE_SCHEMA,
            "preflight_artifact_cid": preflight_head["artifact_cid"],
            "source_cid": cid_bytes(parity_source.encode("utf-8")),
            "source_path": "preflight/parity-source.txt",
            "question": parity_record["question"],
            "record_cid": parity_record["record_cid"],
            **evaluation,
            "maximum_absolute_tolerance": PARITY_MAX_ABSOLUTE_TOLERANCE,
        },
        "fixture_cid",
    )
    atomic_write_json(preflight_root / "python-score-fixture.json", fixture)
    preflight_manifest = write_bound_manifest(
        preflight_root / "preflight-manifest.json",
        {
            "schema": PREFLIGHT_RESULT_SCHEMA,
            "terminal": "AWAITING_RUST_SCORE_PARITY",
            "run_contract_cid": run_contract["run_contract_cid"],
            "dataset_manifest_cid": dataset_manifest["manifest_cid"],
            "feature_manifest_cid": feature_manifest["manifest_cid"],
            "preflight_result_cid": preflight_result["result_cid"],
            "preflight_artifact_cid": preflight_head["artifact_cid"],
            "python_score_fixture_cid": fixture["fixture_cid"],
        },
        artifact_root=root,
        relative_paths=[
            "source-span-dataset.json",
            "product-probes.json",
            "source-span-dataset-manifest.json",
            "source-span-run-contract.json",
            "features/feature-index.json",
            "features/states.safetensors",
            "features/feature-manifest.json",
            "preflight/preflight-result.json",
            "preflight/preflight-head.json",
            "preflight/parity-source.txt",
            "preflight/python-score-fixture.json",
        ],
    )
    return {
        "terminal": "AWAITING_RUST_SCORE_PARITY",
        "preflight_artifact": str(preflight_root / "preflight-head.json"),
        "preflight_artifact_cid": preflight_head["artifact_cid"],
        "parity_source": str(parity_source_path),
        "question": parity_record["question"],
        "python_score_fixture": str(preflight_root / "python-score-fixture.json"),
        "python_score_fixture_cid": fixture["fixture_cid"],
        "preflight_manifest_cid": preflight_manifest["manifest_cid"],
    }


def _numeric_list(value: Any, label: str) -> list[float]:
    if not isinstance(value, list) or not value:
        raise ValueError(f"Rust parity {label} must be a nonempty list")
    values = [float(item) for item in value]
    if any(not math.isfinite(item) for item in values):
        raise ValueError(f"Rust parity {label} contains a nonfinite value")
    return values


def _numeric_logits(value: Any) -> dict[str, float]:
    if not isinstance(value, dict) or set(value) != set(OUTCOMES):
        raise ValueError("Rust parity logits must contain answer, abstain, and conflict")
    logits = {name: float(value[name]) for name in OUTCOMES}
    if any(not math.isfinite(item) for item in logits.values()):
        raise ValueError("Rust parity logits contain a nonfinite value")
    return logits


def _admit_rust_score_parity(root: Path, report_path: Path) -> dict[str, Any]:
    fixture = json.loads(
        (root / "preflight/python-score-fixture.json").read_text(encoding="utf-8")
    )
    preflight_head = json.loads(
        (root / "preflight/preflight-head.json").read_text(encoding="utf-8")
    )
    report = json.loads(report_path.read_text(encoding="utf-8"))
    if report.get("schema") != RUST_GROUNDED_REPORT_SCHEMA:
        raise ValueError("Rust parity report is not uor-r4.grounded-answer/2")
    pointer = report.get("pointer")
    evaluation = report.get("pointer_evaluation")
    source = report.get("source")
    state_encoding = report.get("state_encoding")
    if not isinstance(pointer, dict) or not isinstance(evaluation, dict):
        raise ValueError("Rust parity report omits pointer bindings/evaluation")
    if not isinstance(source, dict) or not isinstance(state_encoding, dict):
        raise ValueError("Rust parity report omits source/state encoding")
    if pointer.get("artifact_cid") != preflight_head["artifact_cid"]:
        raise ValueError("Rust parity loaded a different preflight pointer artifact")
    if pointer.get("policy") != POLICY:
        raise ValueError("Rust parity loaded a different pointer policy")
    if source.get("source_cid") != fixture["source_cid"]:
        raise ValueError("Rust parity source CID differs from the Python fixture")
    if report.get("question") != fixture["question"]:
        raise ValueError("Rust parity question differs from the Python fixture")
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

    rust_scores = _numeric_list(
        evaluation.get("candidate_scores"), "candidate scores"
    )
    python_scores = _numeric_list(fixture["candidate_scores"], "Python candidate scores")
    if len(rust_scores) != len(python_scores):
        raise ValueError("Rust parity candidate count differs")
    rust_logits = _numeric_logits(evaluation.get("logits"))
    python_logits = _numeric_logits(fixture["logits"])
    score_delta = max(abs(left - right) for left, right in zip(rust_scores, python_scores))
    logit_delta = max(
        abs(rust_logits[outcome] - python_logits[outcome]) for outcome in OUTCOMES
    )
    tolerance = float(fixture["maximum_absolute_tolerance"])
    if tolerance != PARITY_MAX_ABSOLUTE_TOLERANCE:
        raise ValueError("Python score fixture parity tolerance differs")
    if score_delta > tolerance or logit_delta > tolerance:
        raise ValueError(
            "Rust pointer score parity exceeded tolerance: "
            f"scores={score_delta}, logits={logit_delta}, limit={tolerance}"
        )
    rust_decision = evaluation.get("decision")
    if rust_decision not in OUTCOMES:
        raise ValueError("Rust parity decision is outside answer/abstain/conflict")
    rust_selected_span = evaluation.get("selected_span_index")
    if rust_selected_span is not None and not isinstance(rust_selected_span, int):
        raise ValueError("Rust parity selected span is not an integer or null")
    if rust_decision != fixture["decision"]:
        raise ValueError("Rust parity decision differs from Python")
    if rust_selected_span != fixture["selected_span_index"]:
        raise ValueError("Rust parity selected span differs from Python")

    copied_report_path = root / "preflight/rust-score-parity.json"
    atomic_write_json(copied_report_path, report)
    admission = _canonical_with_cid(
        {
            "schema": PARITY_ADMISSION_SCHEMA,
            "issue": ISSUE,
            "policy": POLICY,
            "terminal": "PASS_PYTHON_RUST_SOURCE_SPAN_SCORE_PARITY",
            "preflight_artifact_cid": preflight_head["artifact_cid"],
            "python_score_fixture_cid": fixture["fixture_cid"],
            "rust_report_cid": cid_bytes(canonical_json_bytes(report)),
            "maximum_absolute_score_delta": score_delta,
            "maximum_absolute_logit_delta": logit_delta,
            "maximum_absolute_tolerance": tolerance,
            "decision": fixture["decision"],
            "selected_span_index": fixture["selected_span_index"],
        },
        "admission_cid",
    )
    atomic_write_json(root / "preflight/score-parity-admission.json", admission)
    return admission


def _prepare_contract(
    root: Path,
    *,
    predecessor_manifest: dict[str, Any],
    config: SourceSpanPointerConfig,
) -> tuple[dict[str, Any], dict[str, Any], dict[str, Any], dict[str, Any]]:
    dataset, probes = build_source_span_population()
    root.mkdir(parents=True, exist_ok=True)
    _write_or_verify_json(root / "source-span-dataset.json", dataset)
    _write_or_verify_json(root / "product-probes.json", probes)
    dataset_manifest_path = root / "source-span-dataset-manifest.json"
    if dataset_manifest_path.exists():
        dataset_manifest = verify_bound_manifest(dataset_manifest_path, artifact_root=root)
        if (
            dataset_manifest.get("dataset_cid") != dataset["dataset_cid"]
            or dataset_manifest.get("split_policy_cid") != dataset["split_policy_cid"]
        ):
            raise ValueError("existing source-span dataset manifest differs")
    else:
        dataset_manifest = write_bound_manifest(
            dataset_manifest_path,
            {
                "schema": DATASET_MANIFEST_SCHEMA,
                "issue": ISSUE,
                "policy": POLICY,
                "dataset_cid": dataset["dataset_cid"],
                "split_policy_cid": dataset["split_policy_cid"],
                "product_probes_cid": probes["product_probes_cid"],
                "product_probe_commitments": dataset["product_probe_commitments"],
                "predecessor_weights_cid": EXPECTED_WEIGHTS_CID,
                "predecessor_tokenizer_cid": EXPECTED_TOKENIZER_CID,
            },
            artifact_root=root,
            relative_paths=["source-span-dataset.json", "product-probes.json"],
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
            },
            "model": FROZEN_MODEL_CONFIG.as_contract(),
            "dataset_cid": dataset["dataset_cid"],
            "dataset_manifest_cid": dataset_manifest["manifest_cid"],
            "split_policy_cid": dataset["split_policy_cid"],
            "product_probe_commitments": dataset["product_probe_commitments"],
            "state_extraction": (
                "MPS-only immutable #1017 final normalized residual states for each "
                "independently encoded subject and sentence; content tokens only"
            ),
            "mechanism": (
                "positive 288-lane diagonal weighted cosine; subject-token maxima "
                "averaged per candidate; positive score scale; answer/abstain/conflict biases"
            ),
            "loss": (
                "three-way outcome cross entropy for every construction record plus "
                "candidate-span cross entropy for supported records"
            ),
            "optimization": config.as_contract(),
            "cheap_gate": {
                "balanced_overfit_records": 12,
                "required_outcome_accuracy": 1.0,
                "required_supported_pointer_accuracy": 1.0,
                "python_rust_score_tolerance": PARITY_MAX_ABSOLUTE_TOLERANCE,
            },
            "development_gate": {
                "minimum_per_class_outcome_accuracy": config.required_per_class_accuracy,
                "minimum_supported_span_pointer_accuracy": config.required_pointer_accuracy,
            },
            "implementation": trainer_implementation_contract(),
        },
        "run_contract_cid",
    )
    _write_or_verify_json(root / "source-span-run-contract.json", run_contract)
    return dataset, probes, dataset_manifest, run_contract


def _run_full_fit(
    root: Path,
    *,
    dataset_value: dict[str, Any],
    probes: dict[str, Any],
    dataset_manifest: dict[str, Any],
    feature_manifest: dict[str, Any],
    run_contract: dict[str, Any],
    parity_admission: dict[str, Any],
    config: SourceSpanPointerConfig,
) -> dict[str, Any]:
    fit_marker = root / "fit-started.json"
    if fit_marker.exists():
        raise FileExistsError(
            "the sole C1-SB1 fit was already started; no rerun or resume is authorized"
        )
    atomic_write_json(
        fit_marker,
        {
            "schema": RUN_SCHEMA,
            "phase": "SOLE_256_STEP_FIT_STARTED",
            "run_contract_cid": run_contract["run_contract_cid"],
            "parity_admission_cid": parity_admission["admission_cid"],
        },
    )
    device = require_mps(config.seed)
    features = FrozenFeatureStore(root / "features")
    construction = PointerDataset(dataset_value["construction"], features)
    development = PointerDataset(dataset_value["development"], features)
    torch.manual_seed(config.seed)
    if hasattr(torch.mps, "manual_seed"):
        torch.mps.manual_seed(config.seed)
    model = R4SourceSpanPointer().to(device)
    fit = _fit_head(
        model,
        construction,
        device=device,
        steps=config.optimizer_steps,
        batch_size=config.batch_size,
        config=config,
        phase="sole-256-step-fit",
    )
    development_metrics = evaluate_pointer(
        model,
        development,
        device=device,
        batch_size=config.batch_size,
    )
    passed = _development_gate(development_metrics, config)
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
                "SOURCE_SPAN_POINTER_FIT_COMPLETE_AWAITING_RUST_PRODUCT_REVEAL"
                if passed
                else "FAIL_SOURCE_SPAN_POINTER_DEVELOPMENT_GATE_STOP"
            ),
            "run_contract_cid": run_contract["run_contract_cid"],
            "dataset_cid": dataset_value["dataset_cid"],
            "dataset_manifest_cid": dataset_manifest["manifest_cid"],
            "feature_manifest_cid": feature_manifest["manifest_cid"],
            "model_weights_cid": EXPECTED_WEIGHTS_CID,
            "tokenizer_cid": EXPECTED_TOKENIZER_CID,
            "preflight_result_cid": preflight_result["result_cid"],
            "rust_score_parity_admission_cid": parity_admission["admission_cid"],
            "optimization": deterministic_fit,
            "development_metrics": development_metrics,
            "development_gate_passed": passed,
            "product_probe_commitments": dataset_value["product_probe_commitments"],
            "product_probe_file_cid": probes["product_probes_cid"],
            "product_behavior_status": "NOT_RUN",
        },
        "result_cid",
    )
    atomic_write_json(root / "source-span-training-result.json", result)

    final_paths = [
        "source-span-dataset.json",
        "product-probes.json",
        "source-span-dataset-manifest.json",
        "source-span-run-contract.json",
        "features/feature-index.json",
        "features/states.safetensors",
        "features/feature-manifest.json",
        "preflight/preflight-result.json",
        "preflight/preflight-head.json",
        "preflight/parity-source.txt",
        "preflight/python-score-fixture.json",
        "preflight/rust-score-parity.json",
        "preflight/score-parity-admission.json",
        "fit-started.json",
        "source-span-training-result.json",
    ]
    head: dict[str, Any] | None = None
    if passed:
        embedded_preflight = {
            "status": "PASS_12_RECORD_OVERFIT_AND_PYTHON_RUST_SCORE_PARITY",
            "record_cids": preflight_result["record_cids"],
            "optimizer_steps": config.preflight_optimizer_steps,
            "metrics": preflight_result["metrics"],
            "preflight_result_cid": preflight_result["result_cid"],
            "rust_score_parity_admission_cid": parity_admission["admission_cid"],
        }
        head = _head_payload(
            model,
            dataset=dataset_value,
            run_contract_cid=run_contract["run_contract_cid"],
            training_result_cid=result["result_cid"],
            preflight=embedded_preflight,
            development_metrics=development_metrics,
        )
        atomic_write_json(root / "source-span-pointer.json", head)
        final_paths.append("source-span-pointer.json")

    final_manifest = write_bound_manifest(
        root / "source-span-final-manifest.json",
        {
            "schema": FINAL_MANIFEST_SCHEMA,
            "issue": ISSUE,
            "policy": POLICY,
            "terminal": result["terminal"],
            "run_contract_cid": run_contract["run_contract_cid"],
            "dataset_cid": dataset_value["dataset_cid"],
            "dataset_manifest_cid": dataset_manifest["manifest_cid"],
            "feature_manifest_cid": feature_manifest["manifest_cid"],
            "training_result_cid": result["result_cid"],
            "pointer_artifact_cid": None if head is None else head["artifact_cid"],
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
        "pointer_artifact": None if head is None else str(root / "source-span-pointer.json"),
        "pointer_artifact_cid": None if head is None else head["artifact_cid"],
        "training_result_cid": result["result_cid"],
        "final_manifest_cid": final_manifest["manifest_cid"],
        "product_behavior_status": "NOT_RUN" if passed else "BLOCKED_BY_DEVELOPMENT",
    }


def train_source_span_pointer(
    root: Path,
    *,
    predecessor: Path,
    rust_score_parity: Path | None = None,
    config: SourceSpanPointerConfig = SourceSpanPointerConfig(),
) -> dict[str, Any]:
    """Advance the two-invocation C1-SB1 gate without opening product probes."""
    config.validate()
    root = root.expanduser().resolve()
    predecessor = predecessor.expanduser().resolve()
    if root == predecessor or predecessor in root.parents:
        raise ValueError("source-span output must not overwrite immutable #1017")
    if (root / "source-span-final-manifest.json").exists():
        raise FileExistsError("the sole C1-SB1 pointer fit is already complete")

    predecessor_manifest = _validated_predecessor(predecessor)
    dataset_value, probes, dataset_manifest, run_contract = _prepare_contract(
        root,
        predecessor_manifest=predecessor_manifest,
        config=config,
    )
    feature_manifest = _extract_features(
        root,
        predecessor=predecessor,
        predecessor_manifest=predecessor_manifest,
        dataset=dataset_value,
        dataset_manifest=dataset_manifest,
        config=config,
    )

    preflight_manifest_path = root / "preflight/preflight-manifest.json"
    if not preflight_manifest_path.exists():
        result = _run_preflight(
            root,
            dataset_value=dataset_value,
            dataset_manifest=dataset_manifest,
            feature_manifest=feature_manifest,
            run_contract=run_contract,
            config=config,
        )
        if rust_score_parity is not None:
            result["note"] = (
                "preflight was created in this invocation; run the emitted Rust command "
                "and invoke this same subcommand again with --rust-score-parity"
            )
        return result

    preflight_manifest = verify_bound_manifest(
        preflight_manifest_path, artifact_root=root
    )
    if preflight_manifest.get("terminal") != "AWAITING_RUST_SCORE_PARITY":
        return {
            "terminal": preflight_manifest.get("terminal", "PREFLIGHT_STOPPED"),
            "preflight_manifest_cid": preflight_manifest["manifest_cid"],
        }
    if rust_score_parity is None:
        fixture = json.loads(
            (root / "preflight/python-score-fixture.json").read_text(encoding="utf-8")
        )
        return {
            "terminal": "AWAITING_RUST_SCORE_PARITY",
            "preflight_artifact": str(root / "preflight/preflight-head.json"),
            "preflight_artifact_cid": fixture["preflight_artifact_cid"],
            "parity_source": str(root / fixture["source_path"]),
            "question": fixture["question"],
            "python_score_fixture": str(root / "preflight/python-score-fixture.json"),
            "python_score_fixture_cid": fixture["fixture_cid"],
        }

    report_path = rust_score_parity.expanduser().resolve()
    if not report_path.is_file():
        raise FileNotFoundError(report_path)
    parity_admission = _admit_rust_score_parity(root, report_path)
    return _run_full_fit(
        root,
        dataset_value=dataset_value,
        probes=probes,
        dataset_manifest=dataset_manifest,
        feature_manifest=feature_manifest,
        run_contract=run_contract,
        parity_admission=parity_admission,
        config=config,
    )
