"""One construction-only selection campaign for the #973 retained decoder."""

from __future__ import annotations

import gc
import json
import os
import statistics
import time
from collections.abc import Callable, Mapping
from dataclasses import asdict, dataclass, fields, is_dataclass
from pathlib import Path
from typing import Any, Protocol

import torch
from blake3 import blake3
from torch import Tensor

from .group_retention import GEOMETRY_ARMS, GroupAddressArtifact
from .group_retention_campaign import (
    GEOMETRY_RELATIVE_PATH,
    GroupGeometryBundle,
    load_group_geometry_artifacts,
)
from .group_retention_decoder import (
    DecoderConfig,
    R4GroupAddressedRetentionDecoderV1,
)
from .group_retention_decoder_data import (
    CONTEXT,
    DECISIONS_PER_PARTITION,
    EXCLUDED_PRIOR_SMOKE_ORDINALS,
    EXPECTED_GEOMETRY_ARTIFACT_CID,
    EXPECTED_PREDECESSOR_FIT_INDEX_CID,
    EXPECTED_PREDECESSOR_FIT_STORE_CID,
    EXPECTED_PREDECESSOR_POPULATION_CID,
    EXPECTED_PREDECESSOR_TRAINING_VIEW_CID,
    EXPECTED_TOKENIZER_CID,
    STORIES_PER_PARTITION,
    TOKENS_PER_STORY,
    TRAIN_INDEX_RELATIVE_PATH,
    TRAIN_ORDINALS,
    TRAIN_TOKENS_RELATIVE_PATH,
    VALIDATION_INDEX_RELATIVE_PATH,
    VALIDATION_ORDINALS,
    VALIDATION_TOKENS_RELATIVE_PATH,
    build_decoder_construction_data,
    decode_construction_tensor,
)
from .provenance import (
    atomic_write,
    canonical_json_bytes,
    cid_bytes,
    cid_file,
    trainer_implementation_contract,
    verify_bound_manifest,
    write_bound_manifest,
)
from .train import require_mps


ISSUE = 973
POLICY = "R4GroupAddressedRetentionDecoderV1"
TRAINED_ARMS = ("exact_h4", "scrambled_h4")
MECHANICAL_ARMS = GEOMETRY_ARMS

PREPARATION_MANIFEST_NAME = "group-retention-decoder-preparation-manifest.json"
STARTED_RELATIVE_PATH = "preflight/group-retention-decoder-started.json"
RESULT_RELATIVE_PATH = "preflight/group-retention-decoder-result.json"
EXACT_FITTED_RELATIVE_PATH = "fitted/exact-h4.safetensors"
SCRAMBLED_FITTED_RELATIVE_PATH = "fitted/scrambled-h4.safetensors"

PREPARATION_SCHEMA = "uor-r4.group-addressed-retention-decoder-preparation/1"
STARTED_SCHEMA = "uor-r4.group-addressed-retention-decoder-started/1"
RESULT_SCHEMA = "uor-r4.group-addressed-retention-decoder-result/1"

TERMINAL_UNAVAILABLE = "UNAVAILABLE_FULLER_DECODER_CONSTRUCTION"
TERMINAL_PASS = "RETAINED_DECODER_PASS"
TERMINAL_FAIL = "RETAINED_DECODER_FAIL"
H4_SPECIFIC_PASS = "H4_SPECIFIC_PASS"
H4_SPECIFIC_MISS = "H4_SPECIFIC_MISS"
H4_SPECIFIC_NOT_EVALUATED = "NOT_EVALUATED"

EXPECTED_PARAMETER_COUNT = 3_171_760
EXPECTED_STATE_VALUES = 138_240
EXPECTED_STATE_BYTES = 552_960
REACHABLE_VALIDATION_DECISIONS = 4_064


class _ScientificModelFailure(RuntimeError):
    """A finite admitted model became invalid during the frozen fit."""


@dataclass(frozen=True, slots=True)
class DecoderPreflightConfig:
    model: DecoderConfig = DecoderConfig.production_unchecked()
    seed: int = 9_737
    batch_size: int = 8
    context: int = CONTEXT
    optimizer_steps_per_arm: int = 256
    learning_rate: float = 0.003
    beta1: float = 0.9
    beta2: float = 0.95
    epsilon: float = 1e-8
    weight_decay: float = 0.0
    gradient_clip: float = 1.0
    warmup_steps: int = 1
    measured_steps: int = 3
    eta_safety_factor: float = 1.25
    eta_total_steps: int = 512
    wall_ceiling_seconds: float = 600.0
    required_train_reduction: float = 0.50
    required_validation_improvement: float = 0.10
    required_state_off_nll_delta: float = 0.05
    required_state_off_top1_delta: int = 11
    required_h4_nll_delta: float = 0.02
    required_h4_top1_delta: int = 11

    @classmethod
    def production(cls) -> DecoderPreflightConfig:
        value = cls()
        value.validate()
        return value

    def validate(self) -> None:
        self.model.validate()
        if self != DecoderPreflightConfig():
            raise ValueError("#973 exposes one frozen fuller-decoder preflight contract")
        if (
            self.context != CONTEXT
            or STORIES_PER_PARTITION % self.batch_size
            or self.optimizer_steps_per_arm * self.batch_size * self.context != 262_144
            or self.eta_total_steps != 2 * self.optimizer_steps_per_arm
        ):
            raise ValueError("#973 fuller-decoder presentation arithmetic differs")


class DeviceTelemetry(Protocol):
    def synchronize(self) -> None: ...

    def empty_cache(self) -> None: ...

    def recommended_memory(self) -> int: ...

    def allocated_memory(self) -> int: ...


@dataclass(slots=True)
class _MpsTelemetry:
    def synchronize(self) -> None:
        torch.mps.synchronize()

    def empty_cache(self) -> None:
        torch.mps.empty_cache()

    def recommended_memory(self) -> int:
        return int(torch.mps.recommended_max_memory())

    def allocated_memory(self) -> int:
        return max(
            int(torch.mps.current_allocated_memory()),
            int(torch.mps.driver_allocated_memory()),
        )


def _write_exclusive(path: Path, value: bytes) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    descriptor = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
    try:
        with os.fdopen(descriptor, "wb", closefd=True) as target:
            descriptor = -1
            target.write(value)
            target.flush()
            os.fsync(target.fileno())
    finally:
        if descriptor >= 0:
            os.close(descriptor)


def _write_exclusive_json(path: Path, value: Mapping[str, Any]) -> None:
    _write_exclusive(path, canonical_json_bytes(value))


def _with_cid(value: Mapping[str, Any], field: str) -> dict[str, Any]:
    if field in value:
        raise ValueError(f"self-CID field already exists: {field}")
    result = dict(value)
    result[field] = cid_bytes(canonical_json_bytes(value))
    return result


def prepare_group_retention_decoder_data(
    root: Path,
    *,
    predecessor: Path,
    _manifest_name: str = PREPARATION_MANIFEST_NAME,
    _schema: str = PREPARATION_SCHEMA,
    _policy: str = POLICY,
) -> dict[str, Any]:
    """Freeze the independent construction slices and inherited geometry."""
    root = root.resolve()
    predecessor = predecessor.resolve()
    managed = [
        root / _manifest_name,
        root / "construction",
        root / "geometry",
        root / "preflight",
        root / "fitted",
    ]
    if any(path.exists() or path.is_symlink() for path in managed):
        raise FileExistsError("#973 fuller-decoder preparation is create-once")
    if root == predecessor:
        raise ValueError("successor root must differ from the immutable predecessor")

    data = build_decoder_construction_data(predecessor)
    predecessor_geometry = predecessor / GEOMETRY_RELATIVE_PATH
    geometry_bytes = predecessor_geometry.read_bytes()
    geometry = load_group_geometry_artifacts(predecessor_geometry)
    if geometry.artifact_cid != EXPECTED_GEOMETRY_ARTIFACT_CID:
        raise ValueError("predecessor geometry artifact differs from the fuller-decoder freeze")

    artifact_values = {
        TRAIN_TOKENS_RELATIVE_PATH: data.train.tokens,
        TRAIN_INDEX_RELATIVE_PATH: data.train.index,
        VALIDATION_TOKENS_RELATIVE_PATH: data.validation.tokens,
        VALIDATION_INDEX_RELATIVE_PATH: data.validation.index,
        GEOMETRY_RELATIVE_PATH: geometry_bytes,
    }
    root.mkdir(parents=True, exist_ok=True)
    for relative, value in artifact_values.items():
        atomic_write(root / relative, value)

    selection = {
        "policy": (
            "exclude predecessor fit ordinals 0-7; construction train ordinals "
            "8-39; construction validation ordinals 40-71; first 129 tokens"
        ),
        "excluded_prior_smoke_ordinals": list(EXCLUDED_PRIOR_SMOKE_ORDINALS),
        "train": {
            "ordinals": list(TRAIN_ORDINALS),
            "story_cids": list(data.train.story_cids),
            "selected_span_cids": list(data.train.span_cids),
            "stories": STORIES_PER_PARTITION,
            "decisions": DECISIONS_PER_PARTITION,
        },
        "validation": {
            "ordinals": list(VALIDATION_ORDINALS),
            "story_cids": list(data.validation.story_cids),
            "selected_span_cids": list(data.validation.span_cids),
            "stories": STORIES_PER_PARTITION,
            "decisions": DECISIONS_PER_PARTITION,
        },
        "tokens_per_story": TOKENS_PER_STORY,
        "context": CONTEXT,
        "story_disjoint": True,
        "model_heldout_reads": 0,
    }
    manifest = write_bound_manifest(
        root / _manifest_name,
        {
            "schema": _schema,
            "issue": ISSUE,
            "policy": _policy,
            "predecessor": dict(data.predecessor),
            "geometry": {
                "artifact_cid": geometry.artifact_cid,
                "file_cid": geometry.geometry_file_cid,
                "generated_state_coverage": {
                    "exact_h4": geometry.h4_generated_count,
                    "cyclic_120": geometry.c120_generated_count,
                    "scrambled_h4": geometry.scrambled_generated_count,
                },
            },
            "selection": selection,
            "selection_cid": cid_bytes(canonical_json_bytes(selection)),
            "implementation": trainer_implementation_contract(),
            "optimization": "NOT_RUN",
            "model_heldout": {"status": "NOT_RUN", "reads": 0},
        },
        artifact_root=root,
        relative_paths=artifact_values,
    )
    return manifest


def _load_prepared(
    root: Path,
    *,
    manifest_name: str = PREPARATION_MANIFEST_NAME,
    schema: str = PREPARATION_SCHEMA,
    policy: str = POLICY,
) -> tuple[dict[str, Any], GroupGeometryBundle, Tensor, Tensor]:
    root = root.resolve()
    manifest = verify_bound_manifest(root / manifest_name, artifact_root=root)
    geometry = load_group_geometry_artifacts(root / GEOMETRY_RELATIVE_PATH)
    coverage = manifest.get("geometry", {}).get("generated_state_coverage")
    expected_predecessor = {
        "training_view_manifest_cid": EXPECTED_PREDECESSOR_TRAINING_VIEW_CID,
        "population_manifest_cid": EXPECTED_PREDECESSOR_POPULATION_CID,
        "fit_store_cid": EXPECTED_PREDECESSOR_FIT_STORE_CID,
        "fit_index_cid": EXPECTED_PREDECESSOR_FIT_INDEX_CID,
        "tokenizer_cid": EXPECTED_TOKENIZER_CID,
    }
    expected_artifacts = {
        TRAIN_TOKENS_RELATIVE_PATH,
        TRAIN_INDEX_RELATIVE_PATH,
        VALIDATION_TOKENS_RELATIVE_PATH,
        VALIDATION_INDEX_RELATIVE_PATH,
        GEOMETRY_RELATIVE_PATH,
    }
    artifact_paths = {
        str(record.get("path"))
        for record in manifest.get("artifacts", [])
        if isinstance(record, Mapping)
    }
    selection = manifest.get("selection")
    selection_valid = isinstance(selection, Mapping) and (
        selection.get("excluded_prior_smoke_ordinals")
        == list(EXCLUDED_PRIOR_SMOKE_ORDINALS)
        and selection.get("train", {}).get("ordinals") == list(TRAIN_ORDINALS)
        and selection.get("validation", {}).get("ordinals")
        == list(VALIDATION_ORDINALS)
        and selection.get("train", {}).get("stories") == STORIES_PER_PARTITION
        and selection.get("validation", {}).get("stories")
        == STORIES_PER_PARTITION
        and selection.get("train", {}).get("decisions")
        == DECISIONS_PER_PARTITION
        and selection.get("validation", {}).get("decisions")
        == DECISIONS_PER_PARTITION
        and selection.get("tokens_per_story") == TOKENS_PER_STORY
        and selection.get("context") == CONTEXT
        and selection.get("story_disjoint") is True
        and selection.get("model_heldout_reads") == 0
        and manifest.get("selection_cid")
        == cid_bytes(canonical_json_bytes(selection))
    )
    if (
        manifest.get("schema") != schema
        or manifest.get("issue") != ISSUE
        or manifest.get("policy") != policy
        or manifest.get("predecessor") != expected_predecessor
        or artifact_paths != expected_artifacts
        or not selection_valid
        or manifest.get("geometry", {}).get("artifact_cid") != geometry.artifact_cid
        or manifest.get("geometry", {}).get("file_cid") != geometry.geometry_file_cid
        or geometry.artifact_cid != EXPECTED_GEOMETRY_ARTIFACT_CID
        or coverage != {arm: 120 for arm in MECHANICAL_ARMS}
        or manifest.get("implementation") != trainer_implementation_contract()
        or manifest.get("optimization") != "NOT_RUN"
        or manifest.get("model_heldout") != {"status": "NOT_RUN", "reads": 0}
    ):
        raise ValueError("fuller-decoder preparation differs from the frozen contract")
    train = decode_construction_tensor(
        (root / TRAIN_TOKENS_RELATIVE_PATH).read_bytes(), partition="train"
    )
    validation = decode_construction_tensor(
        (root / VALIDATION_TOKENS_RELATIVE_PATH).read_bytes(), partition="validation"
    )
    assert isinstance(selection, Mapping)
    train_story_ids = selection["train"].get("story_cids", [])
    validation_story_ids = selection["validation"].get("story_cids", [])
    train_span_ids = selection["train"].get("selected_span_cids", [])
    validation_span_ids = selection["validation"].get("selected_span_cids", [])
    if (
        len(train_story_ids) != STORIES_PER_PARTITION
        or len(validation_story_ids) != STORIES_PER_PARTITION
        or len(set(train_story_ids)) != STORIES_PER_PARTITION
        or len(set(validation_story_ids)) != STORIES_PER_PARTITION
        or set(train_story_ids) & set(validation_story_ids)
        or len(train_span_ids) != STORIES_PER_PARTITION
        or len(validation_span_ids) != STORIES_PER_PARTITION
    ):
        raise ValueError("fuller-decoder construction selection is not disjoint and complete")
    for partition, expected_ordinals, expected_story_ids, expected_span_ids in (
        ("train", TRAIN_ORDINALS, train_story_ids, train_span_ids),
        ("validation", VALIDATION_ORDINALS, validation_story_ids, validation_span_ids),
    ):
        index_path = root / (
            TRAIN_INDEX_RELATIVE_PATH
            if partition == "train"
            else VALIDATION_INDEX_RELATIVE_PATH
        )
        records = []
        for line in index_path.read_bytes().splitlines(keepends=True):
            record = json.loads(line)
            if not isinstance(record, dict) or canonical_json_bytes(record) != line:
                raise ValueError("fuller-decoder construction index is not canonical")
            records.append(record)
        if len(records) != STORIES_PER_PARTITION:
            raise ValueError("fuller-decoder construction index length differs")
        for ordinal, (record, source_ordinal, story_id, span_id) in enumerate(
            zip(
                records,
                expected_ordinals,
                expected_story_ids,
                expected_span_ids,
                strict=True,
            )
        ):
            if (
                record.get("construction_ordinal") != ordinal
                or record.get("construction_partition") != partition
                or record.get("source_fit_ordinal") != source_ordinal
                or record.get("story_cid") != story_id
                or record.get("selected_span_cid") != span_id
                or record.get("copied_token_offset") != ordinal * TOKENS_PER_STORY
                or record.get("copied_token_count") != TOKENS_PER_STORY
                or record.get("scored_next_tokens") != CONTEXT
            ):
                raise ValueError("fuller-decoder construction index differs from selection")
    return manifest, geometry, train, validation


def _initialization_identity(
    arms: Mapping[str, GroupAddressArtifact], config: DecoderConfig
) -> tuple[dict[str, Any], dict[str, bytes]]:
    exports: dict[str, bytes] = {}
    cids: dict[str, str] = {}
    ledgers: dict[str, dict[str, int]] = {}
    for arm in MECHANICAL_ARMS:
        model = R4GroupAddressedRetentionDecoderV1(config, arms[arm])
        tied_output_storage = (
            model.output_weight.data_ptr() == model.token_embedding.weight.data_ptr()
        )
        if not tied_output_storage:
            raise ValueError("decoder output head is not tied to token embedding storage")
        export = model.export_learned_artifact()
        exports[arm] = export
        cids[arm] = cid_bytes(export)
        ledgers[arm] = {
            "parameters": model.parameter_count(),
            "state_values_per_sequence": model.state_value_count(),
            "state_bytes_f32_per_sequence": model.state_value_count() * 4,
            "tied_output_storage": int(tied_output_storage),
        }
    if len(set(cids.values())) != 1 or len({tuple(value.items()) for value in ledgers.values()}) != 1:
        raise ValueError("fuller-decoder arm initialization or ledgers differ")
    if any(export != exports["exact_h4"] for export in exports.values()):
        raise ValueError("fuller-decoder learned initialization is not byte-identical")
    ledger = next(iter(ledgers.values()))
    if (
        ledger["parameters"] != EXPECTED_PARAMETER_COUNT
        or ledger["state_values_per_sequence"] != EXPECTED_STATE_VALUES
        or ledger["state_bytes_f32_per_sequence"] != EXPECTED_STATE_BYTES
    ):
        raise ValueError("fuller-decoder parameter or state count differs")
    return (
        {
            "seed": 9_737,
            "learned_initialization_cid": next(iter(cids.values())),
            "arm_cids": cids,
            "byte_identical": True,
            "ledgers": ledgers,
            "equal_ledgers": True,
        },
        exports,
    )


def _require_mps_device(backend: str) -> tuple[torch.device, DeviceTelemetry]:
    if backend != "mps":
        raise ValueError("#973 fuller-decoder construction permits only backend='mps'")
    return require_mps(seed=9_737), _MpsTelemetry()


def _audit_signature(audit: Any) -> tuple[int, ...]:
    if hasattr(audit, "work_signature"):
        signature = audit.work_signature()
        return tuple(int(value) for value in signature)
    raise ValueError("decoder output audit has no equal-work signature")


def _state_tensors(value: Any) -> list[Tensor]:
    if isinstance(value, Tensor):
        return [value]
    if is_dataclass(value):
        result: list[Tensor] = []
        for item in fields(value):
            result.extend(_state_tensors(getattr(value, item.name)))
        return result
    if isinstance(value, (tuple, list)):
        result = []
        for item in value:
            result.extend(_state_tensors(item))
        return result
    return []


def _maximum_state_delta(left: Any, right: Any) -> float:
    left_tensors = _state_tensors(left)
    right_tensors = _state_tensors(right)
    if len(left_tensors) != len(right_tensors) or not left_tensors:
        raise ValueError("decoder final-state tensor structures differ")
    return max(
        float((first.float() - second.float()).abs().max().detach().cpu())
        for first, second in zip(left_tensors, right_tensors, strict=True)
    )


def _optimizer(
    model: R4GroupAddressedRetentionDecoderV1, config: DecoderPreflightConfig
) -> torch.optim.Optimizer:
    return torch.optim.AdamW(
        model.parameters(),
        lr=config.learning_rate,
        betas=(config.beta1, config.beta2),
        eps=config.epsilon,
        weight_decay=config.weight_decay,
    )


def _training_step(
    model: R4GroupAddressedRetentionDecoderV1,
    optimizer: torch.optim.Optimizer,
    batch: Tensor,
    config: DecoderPreflightConfig,
) -> tuple[float, Any]:
    optimizer.zero_grad(set_to_none=True)
    output = model(batch[:, :-1], batch[:, 1:])
    if output.loss is None or not bool(torch.isfinite(output.loss).item()):
        raise _ScientificModelFailure(
            "fuller-decoder construction loss is missing or non-finite"
        )
    output.loss.backward()
    gradient_norm = torch.nn.utils.clip_grad_norm_(
        model.parameters(), config.gradient_clip
    )
    if not bool(torch.isfinite(gradient_norm).item()):
        raise _ScientificModelFailure(
            "fuller-decoder construction gradient norm is non-finite"
        )
    optimizer.step()
    return float(output.loss.detach().cpu()), output.audit


def _gradient_census(model: R4GroupAddressedRetentionDecoderV1) -> dict[str, Any]:
    parameters: dict[str, Any] = {}
    for name, parameter in model.named_parameters():
        gradient = parameter.grad
        parameters[name] = {
            "finite": gradient is not None and bool(torch.isfinite(gradient).all().item()),
            "nonzero_values": 0 if gradient is None else int(torch.count_nonzero(gradient).item()),
            "total_values": parameter.numel(),
        }
    passed = bool(parameters) and all(
        value["finite"] and value["nonzero_values"] > 0 for value in parameters.values()
    )
    return {"parameters": parameters, "passed": passed}


@torch.no_grad()
def _evaluate(
    model: R4GroupAddressedRetentionDecoderV1,
    sequences: Tensor,
    *,
    device: torch.device,
    batch_size: int,
    state_off: bool = False,
) -> dict[str, Any]:
    model.eval()
    loss_sum = 0.0
    rows = 0
    correct = 0
    digest = blake3()
    signature: tuple[int, ...] | None = None
    for start in range(0, len(sequences), batch_size):
        batch = sequences[start : start + batch_size].to(device)
        output = model(batch[:, :-1], batch[:, 1:], state_off=state_off)
        if output.loss is None or not bool(torch.isfinite(output.loss).item()):
            raise _ScientificModelFailure(
                "fuller-decoder evaluation loss is missing or non-finite"
            )
        targets = batch[:, 1:]
        count = targets.numel()
        loss_sum += float(output.loss.detach().cpu()) * count
        rows += count
        correct += int((output.logits.argmax(dim=-1) == targets).sum().detach().cpu())
        logits = output.logits.detach().to(device="cpu", dtype=torch.float32).contiguous()
        digest.update(logits.view(torch.uint8).numpy().tobytes(order="C"))
        observed = _audit_signature(output.audit)
        if signature is None:
            signature = observed
        elif signature != observed:
            raise RuntimeError("evaluation work signature changed between equal batches")
    if rows != DECISIONS_PER_PARTITION or signature is None:
        raise RuntimeError("construction evaluation row ledger differs")
    return {
        "ce_nats": loss_sum / rows,
        "top1_correct": correct,
        "rows": rows,
        "logits_cid": f"blake3:{digest.hexdigest()}",
        "work_signature": list(signature),
    }


def _evaluate_emitted_exact(
    artifact: bytes,
    geometry: GroupAddressArtifact,
    config: DecoderConfig,
    validation_sequences: Tensor,
    *,
    device: torch.device,
    batch_size: int,
) -> tuple[dict[str, Any], dict[str, Any]]:
    """Reload emitted bytes into a fresh exact-H4 model before replay."""
    model = R4GroupAddressedRetentionDecoderV1(config, geometry).to(device)
    model.load_learned_artifact(artifact)
    replay = _evaluate(
        model,
        validation_sequences,
        device=device,
        batch_size=batch_size,
    )
    state_off = _evaluate(
        model,
        validation_sequences,
        device=device,
        batch_size=batch_size,
        state_off=True,
    )
    return replay, state_off


def _save_learned_artifact(
    root: Path, relative: str, artifact: bytes
) -> dict[str, Any]:
    path = root / relative
    _write_exclusive(path, artifact)
    return {"path": relative, "bytes": path.stat().st_size, "cid": cid_file(path)}


def _release(telemetry: DeviceTelemetry) -> None:
    gc.collect()
    telemetry.empty_cache()
    telemetry.synchronize()


def _execute_preflight(
    root: Path,
    train_sequences: Tensor,
    validation_sequences: Tensor,
    arms: Mapping[str, GroupAddressArtifact],
    *,
    device: torch.device,
    telemetry: DeviceTelemetry,
    config: DecoderPreflightConfig,
    initial_exports: Mapping[str, bytes],
    wall_ceiling_seconds: float | None = None,
    timing_is_admission_gate: bool = True,
    started_monotonic: float | None = None,
    fitted_relative_paths: tuple[str, str] = (
        EXACT_FITTED_RELATIVE_PATH,
        SCRAMBLED_FITTED_RELATIVE_PATH,
    ),
) -> dict[str, Any]:
    """Run the sole mechanical/timing instrument and two-arm construction fit."""
    config.validate()
    recommended_memory = telemetry.recommended_memory()
    if recommended_memory <= 0:
        raise RuntimeError("device memory recommendation is unavailable")
    started = time.monotonic() if started_monotonic is None else started_monotonic
    wall_ceiling = (
        config.wall_ceiling_seconds
        if wall_ceiling_seconds is None
        else wall_ceiling_seconds
    )

    small = train_sequences[:2, :9].to(device)
    stationary_model = R4GroupAddressedRetentionDecoderV1(
        config.model, arms["exact_h4"]
    ).to(device)
    direct_model = R4GroupAddressedRetentionDecoderV1(
        config.model, arms["exact_h4"]
    ).to(device)
    stationary_model.load_learned_artifact(initial_exports["exact_h4"])
    direct_model.load_learned_artifact(initial_exports["exact_h4"])
    stationary = stationary_model(
        small[:, :-1], small[:, 1:], implementation="stationary"
    )
    direct = direct_model(small[:, :-1], small[:, 1:], implementation="direct")
    logit_delta = float((stationary.logits - direct.logits).abs().max().detach().cpu())
    state_delta = _maximum_state_delta(stationary.final_state, direct.final_state)
    if stationary.loss is None or direct.loss is None:
        raise RuntimeError("parity instrument produced no construction loss")
    stationary.loss.backward()
    direct.loss.backward()
    direct_gradients = dict(direct_model.named_parameters())
    gradient_delta = 0.0
    for name, parameter in stationary_model.named_parameters():
        other = direct_gradients[name]
        if parameter.grad is None or other.grad is None:
            raise RuntimeError(f"parity gradient is absent for {name}")
        gradient_delta = max(
            gradient_delta,
            float((parameter.grad - other.grad).abs().max().detach().cpu()),
        )
    parity_pass = (
        logit_delta <= 1e-5 and state_delta <= 1e-5 and gradient_delta <= 1e-5
    )

    changed_suffix = small.clone()
    changed_suffix[:, -2:] = (changed_suffix[:, -2:] + 1) % config.model.vocab_size
    with torch.no_grad():
        causal_first = stationary_model(small[:, :-1]).logits[:, :-1]
        causal_second = stationary_model(changed_suffix[:, :-1]).logits[:, :-1]
    causality_delta = float((causal_first - causal_second).abs().max().detach().cpu())
    causality_pass = causality_delta == 0.0
    del stationary_model, direct_model, stationary, direct, causal_first, causal_second
    _release(telemetry)

    timing: dict[str, Any] = {}
    timing_seconds: list[float] = []
    signatures: dict[str, tuple[int, ...]] = {}
    gradients_pass = True
    peak_memory = telemetry.allocated_memory()
    timing_batch = train_sequences[: config.batch_size].to(device)
    for arm in TRAINED_ARMS:
        model = R4GroupAddressedRetentionDecoderV1(config.model, arms[arm]).to(device)
        model.load_learned_artifact(initial_exports[arm])
        optimizer = _optimizer(model, config)
        for _ in range(config.warmup_steps):
            _training_step(model, optimizer, timing_batch, config)
            telemetry.synchronize()
        observed: list[float] = []
        last_audit = None
        for _ in range(config.measured_steps):
            telemetry.synchronize()
            step_started = time.perf_counter()
            _, last_audit = _training_step(model, optimizer, timing_batch, config)
            telemetry.synchronize()
            observed.append(time.perf_counter() - step_started)
            peak_memory = max(peak_memory, telemetry.allocated_memory())
        if last_audit is None:
            raise RuntimeError("timing instrument produced no work audit")
        gradient = _gradient_census(model)
        gradients_pass = gradients_pass and bool(gradient["passed"])
        signatures[arm] = _audit_signature(last_audit)
        timing_seconds.extend(observed)
        timing[arm] = {
            "warmup_steps": config.warmup_steps,
            "measured_steps": config.measured_steps,
            "step_seconds": observed,
            "mean_step_seconds": statistics.fmean(observed),
            "gradients": gradient,
            "work_signature": list(signatures[arm]),
        }
        del model, optimizer
        _release(telemetry)

    # C120 is a mechanical equal-work arm only; it never enters an optimizer.
    mechanical_signatures: dict[str, tuple[int, ...]] = dict(signatures)
    for arm in ("cyclic_120",):
        model = R4GroupAddressedRetentionDecoderV1(config.model, arms[arm]).to(device)
        model.load_learned_artifact(initial_exports[arm])
        output = model(timing_batch[:, :-1], timing_batch[:, 1:])
        mechanical_signatures[arm] = _audit_signature(output.audit)
        del model, output
        _release(telemetry)
    state_off_model = R4GroupAddressedRetentionDecoderV1(
        config.model, arms["exact_h4"]
    ).to(device)
    state_off_model.load_learned_artifact(initial_exports["exact_h4"])
    state_off_output = state_off_model(
        timing_batch[:, :-1], timing_batch[:, 1:], state_off=True
    )
    mechanical_signatures["exact_h4_state_off"] = _audit_signature(state_off_output.audit)
    equal_work = len(set(mechanical_signatures.values())) == 1
    del state_off_model, state_off_output
    _release(telemetry)

    mean_step_seconds = statistics.fmean(timing_seconds)
    projected_seconds = config.eta_safety_factor * mean_step_seconds * config.eta_total_steps
    projection_within_ceiling = projected_seconds <= wall_ceiling
    timing_pass = projection_within_ceiling if timing_is_admission_gate else True
    memory_pass = peak_memory < recommended_memory
    mechanical_pass = bool(
        parity_pass
        and causality_pass
        and gradients_pass
        and equal_work
        and timing_pass
        and memory_pass
    )
    mechanical = {
        "full_sequence_incremental_parity": {
            "maximum_logit_delta": logit_delta,
            "maximum_final_state_delta": state_delta,
            "maximum_gradient_delta": gradient_delta,
            "passed": parity_pass,
        },
        "strict_prefix_causality": {
            "maximum_shared_prefix_logit_delta": causality_delta,
            "passed": causality_pass,
        },
        "timing": {
            "arms": timing,
            "global_mean_step_seconds": mean_step_seconds,
            "eta_safety_factor": config.eta_safety_factor,
            "projected_512_step_seconds": projected_seconds,
            "ceiling_seconds": wall_ceiling,
            "admission_role": (
                "BINDING" if timing_is_admission_gate else "TELEMETRY_ONLY"
            ),
            "projection_within_ceiling": projection_within_ceiling,
            "passed": timing_pass,
        },
        "memory": {
            "peak_allocated_bytes": peak_memory,
            "recommended_bytes": recommended_memory,
            "passed": memory_pass,
        },
        "gradients_passed": gradients_pass,
        "work_signatures": {
            arm: list(signature) for arm, signature in mechanical_signatures.items()
        },
        "equal_work": equal_work,
        "passed": mechanical_pass,
    }
    if not mechanical_pass:
        return {
            "available": False,
            "classification": "UNAVAILABLE",
            "mechanical": mechanical,
            "optimization": "NOT_RUN",
            "retained_decoder_pass": False,
            "h4_specific_pass": False,
            "elapsed_seconds": time.monotonic() - started,
            "passed": False,
        }

    completed_steps = {arm: 0 for arm in TRAINED_ARMS}
    metrics: dict[str, Any] = {}
    fitted_exports: dict[str, bytes] = {}

    def wall_stop(
        phase: str, *, artifacts: Mapping[str, Any] | str = "NOT_WRITTEN"
    ) -> dict[str, Any]:
        elapsed = time.monotonic() - started
        wall_mechanical = dict(mechanical)
        wall_mechanical["hard_wall"] = {
            "phase": phase,
            "elapsed_seconds": elapsed,
            "ceiling_seconds": wall_ceiling,
            "passed": False,
        }
        wall_mechanical["passed"] = False
        all_steps_completed = all(
            value == config.optimizer_steps_per_arm
            for value in completed_steps.values()
        )
        return {
            "available": False,
            "classification": "UNAVAILABLE",
            "mechanical": wall_mechanical,
            "optimization": {
                "status": (
                    "COMPLETED_STEPS_WALL_STOP"
                    if all_steps_completed
                    else "PARTIAL_WALL_STOP"
                ),
                "phase": phase,
                "arms": metrics,
                "completed_steps_per_arm": dict(completed_steps),
                "required_steps_per_arm": config.optimizer_steps_per_arm,
                "artifacts": artifacts,
            },
            "retained_decoder_pass": False,
            "h4_specific_pass": False,
            "elapsed_seconds": elapsed,
            "wall_ceiling_seconds": wall_ceiling,
            "wall_passed": False,
            "passed": False,
        }

    def scientific_failure(
        phase: str, error: _ScientificModelFailure
    ) -> dict[str, Any]:
        elapsed = time.monotonic() - started
        if elapsed >= wall_ceiling:
            return wall_stop(phase)
        all_steps_completed = all(
            value == config.optimizer_steps_per_arm
            for value in completed_steps.values()
        )
        return {
            "available": True,
            "classification": "SCIENTIFIC_FAIL",
            "mechanical": mechanical,
            "optimization": {
                "status": (
                    "COMPLETED_STEPS_MODEL_FAILURE"
                    if all_steps_completed
                    else "PARTIAL_MODEL_FAILURE"
                ),
                "phase": phase,
                "arms": metrics,
                "completed_steps_per_arm": dict(completed_steps),
                "required_steps_per_arm": config.optimizer_steps_per_arm,
                "artifacts": "NOT_WRITTEN",
                "failure": {
                    "type": type(error).__name__,
                    "reason": str(error),
                },
            },
            "retained_decoder_pass": False,
            "h4_specific_pass": False,
            "elapsed_seconds": elapsed,
            "wall_ceiling_seconds": wall_ceiling,
            "wall_passed": True,
            "passed": False,
        }

    def admitted_call(
        phase: str, operation: Callable[[], Any]
    ) -> tuple[Any, dict[str, Any] | None]:
        try:
            return operation(), None
        except _ScientificModelFailure as error:
            return None, scientific_failure(phase, error)

    if time.monotonic() - started >= wall_ceiling:
        return wall_stop("before_optimization")
    # Keep the fixed 32-story construction partitions resident on the selected
    # device for all 512 optimizer steps; repeated transfers have no decision
    # value.
    train_sequences = train_sequences.to(device)
    validation_sequences = validation_sequences.to(device)

    for arm in TRAINED_ARMS:
        if time.monotonic() - started >= wall_ceiling:
            return wall_stop(f"before_{arm}")
        model = R4GroupAddressedRetentionDecoderV1(config.model, arms[arm]).to(device)
        model.load_learned_artifact(initial_exports[arm])
        initial_train, terminal = admitted_call(
            f"{arm}_initial_train_evaluation",
            lambda: _evaluate(
                model,
                train_sequences,
                device=device,
                batch_size=config.batch_size,
            ),
        )
        if terminal is not None:
            return terminal
        initial_validation, terminal = admitted_call(
            f"{arm}_initial_validation_evaluation",
            lambda: _evaluate(
                model,
                validation_sequences,
                device=device,
                batch_size=config.batch_size,
            ),
        )
        if terminal is not None:
            return terminal
        if time.monotonic() - started >= wall_ceiling:
            return wall_stop(f"{arm}_initial_evaluation")
        optimizer = _optimizer(model, config)
        model.train()
        for step in range(config.optimizer_steps_per_arm):
            if time.monotonic() - started >= wall_ceiling:
                return wall_stop(f"{arm}_optimization")
            batch_index = step % (STORIES_PER_PARTITION // config.batch_size)
            base = batch_index * config.batch_size
            batch = train_sequences[base : base + config.batch_size].to(device)
            _, terminal = admitted_call(
                f"{arm}_optimization_step_{step + 1}",
                lambda: _training_step(model, optimizer, batch, config),
            )
            if terminal is not None:
                return terminal
            completed_steps[arm] = step + 1
            if time.monotonic() - started >= wall_ceiling:
                return wall_stop(f"{arm}_optimization")
        telemetry.synchronize()
        final_train, terminal = admitted_call(
            f"{arm}_final_train_evaluation",
            lambda: _evaluate(
                model,
                train_sequences,
                device=device,
                batch_size=config.batch_size,
            ),
        )
        if terminal is not None:
            return terminal
        final_validation, terminal = admitted_call(
            f"{arm}_final_validation_evaluation",
            lambda: _evaluate(
                model,
                validation_sequences,
                device=device,
                batch_size=config.batch_size,
            ),
        )
        if terminal is not None:
            return terminal
        train_reduction = (
            initial_train["ce_nats"] - final_train["ce_nats"]
        ) / initial_train["ce_nats"]
        metrics[arm] = {
            "initial_train": initial_train,
            "final_train": final_train,
            "train_ce_reduction_fraction": train_reduction,
            "required_train_ce_reduction_fraction": config.required_train_reduction,
            "initial_validation": initial_validation,
            "final_validation": final_validation,
        }
        fitted_exports[arm] = model.export_learned_artifact()
        del model, optimizer
        _release(telemetry)
        if time.monotonic() - started >= wall_ceiling:
            return wall_stop(f"{arm}_final_evaluation")

    replay_pair, terminal = admitted_call(
        "emitted_exact_replay_and_state_intervention",
        lambda: _evaluate_emitted_exact(
            fitted_exports["exact_h4"],
            arms["exact_h4"],
            config.model,
            validation_sequences,
            device=device,
            batch_size=config.batch_size,
        ),
    )
    if terminal is not None:
        return terminal
    replay, exact_state_off = replay_pair
    _release(telemetry)
    if time.monotonic() - started >= wall_ceiling:
        return wall_stop("state_intervention_and_replay")
    exact_validation = metrics["exact_h4"]["final_validation"]
    scrambled_validation = metrics["scrambled_h4"]["final_validation"]
    state_off_nll_delta = exact_state_off["ce_nats"] - exact_validation["ce_nats"]
    state_off_top1_delta = (
        exact_validation["top1_correct"] - exact_state_off["top1_correct"]
    )
    h4_nll_delta = scrambled_validation["ce_nats"] - exact_validation["ce_nats"]
    h4_top1_delta = (
        exact_validation["top1_correct"] - scrambled_validation["top1_correct"]
    )
    replay_pass = replay == exact_validation
    validation_improvement = (
        metrics["exact_h4"]["initial_validation"]["ce_nats"]
        - exact_validation["ce_nats"]
    )
    learned = all(
        metrics[arm]["train_ce_reduction_fraction"] >= config.required_train_reduction
        for arm in TRAINED_ARMS
    )
    retained_decoder_pass = bool(
        learned
        and validation_improvement >= config.required_validation_improvement
        and state_off_nll_delta >= config.required_state_off_nll_delta
        and state_off_top1_delta >= config.required_state_off_top1_delta
        and replay_pass
    )
    h4_specific_pass = bool(
        retained_decoder_pass
        and h4_nll_delta >= config.required_h4_nll_delta
        and h4_top1_delta >= config.required_h4_top1_delta
    )
    emitted_exact_cid = cid_bytes(fitted_exports["exact_h4"])
    if time.monotonic() - started >= wall_ceiling:
        return wall_stop("before_artifact_write")
    artifacts = {
        "exact_h4": _save_learned_artifact(
            root, fitted_relative_paths[0], fitted_exports["exact_h4"]
        ),
        "scrambled_h4": _save_learned_artifact(
            root,
            fitted_relative_paths[1],
            fitted_exports["scrambled_h4"],
        ),
    }
    if artifacts["exact_h4"]["cid"] != emitted_exact_cid:
        raise RuntimeError("persisted exact-H4 bytes differ from the replayed artifact")
    elapsed = time.monotonic() - started
    if elapsed > wall_ceiling:
        return wall_stop("artifact_write", artifacts=artifacts)
    return {
        "available": True,
        "classification": (
            "SCIENTIFIC_PASS" if retained_decoder_pass else "SCIENTIFIC_FAIL"
        ),
        "mechanical": mechanical,
        "optimization": {
            "status": "COMPLETE",
            "arms": metrics,
            "completed_steps_per_arm": dict(completed_steps),
            "optimizer_steps_per_arm": config.optimizer_steps_per_arm,
            "presentations_per_arm": config.optimizer_steps_per_arm
            * config.batch_size
            * config.context,
            "total_presentations": 2
            * config.optimizer_steps_per_arm
            * config.batch_size
            * config.context,
            "artifacts": artifacts,
        },
        "interventions": {
            "state_off": exact_state_off,
            "state_off_nll_delta": state_off_nll_delta,
            "required_state_off_nll_delta": config.required_state_off_nll_delta,
            "state_off_top1_delta": state_off_top1_delta,
            "required_state_off_top1_delta": config.required_state_off_top1_delta,
            "validation_improvement_nats": validation_improvement,
            "required_validation_improvement_nats": config.required_validation_improvement,
            "h4_nll_delta": h4_nll_delta,
            "required_h4_nll_delta": config.required_h4_nll_delta,
            "h4_top1_delta": h4_top1_delta,
            "required_h4_top1_delta": config.required_h4_top1_delta,
            "replay_passed": replay_pass,
            "replayed_emitted_exact_artifact_cid": emitted_exact_cid,
        },
        "retained_decoder_pass": retained_decoder_pass,
        "h4_specific_pass": h4_specific_pass,
        "elapsed_seconds": elapsed,
        "wall_ceiling_seconds": wall_ceiling,
        "wall_passed": True,
        "passed": retained_decoder_pass,
    }


PreflightExecutor = Callable[..., Mapping[str, Any]]


def _contract(config: DecoderPreflightConfig) -> dict[str, Any]:
    return {
        "backend": "mps",
        "model": asdict(config.model),
        "population": {
            "train_ordinals": list(TRAIN_ORDINALS),
            "validation_ordinals": list(VALIDATION_ORDINALS),
            "stories_per_partition": STORIES_PER_PARTITION,
            "decisions_per_partition": DECISIONS_PER_PARTITION,
            "reachable_validation_decisions": REACHABLE_VALIDATION_DECISIONS,
            "model_heldout_reads": 0,
        },
        "mechanical": {
            "trained_arms": list(TRAINED_ARMS),
            "c120_role": "MECHANICAL_EQUAL_LEDGER_ONLY",
            "warmup_steps_per_trained_arm": config.warmup_steps,
            "measured_steps_per_trained_arm": config.measured_steps,
            "eta_formula": "1.25 * mean_step_seconds * 512",
            "eta_ceiling_seconds": config.wall_ceiling_seconds,
        },
        "optimizer": {
            "name": "AdamW",
            "seed": config.seed,
            "learning_rate": config.learning_rate,
            "schedule": "constant",
            "beta1": config.beta1,
            "beta2": config.beta2,
            "epsilon": config.epsilon,
            "weight_decay": config.weight_decay,
            "gradient_clip": config.gradient_clip,
            "batch_size": config.batch_size,
            "context": config.context,
            "steps_per_trained_arm": config.optimizer_steps_per_arm,
            "presentations_per_trained_arm": 262_144,
            "total_presentations": 524_288,
            "batch_order": "deterministic cyclic four-batch order",
            "retry_or_sweep": "FORBIDDEN",
            "cpu_fallback": False,
            "cuda": False,
            "wall_ceiling_seconds": config.wall_ceiling_seconds,
        },
        "thresholds": {
            "train_ce_reduction_fraction_each_arm": config.required_train_reduction,
            "exact_validation_improvement_nats": config.required_validation_improvement,
            "state_off_nll_delta_nats": config.required_state_off_nll_delta,
            "state_off_top1_net_decisions": config.required_state_off_top1_delta,
            "h4_nll_delta_nats": config.required_h4_nll_delta,
            "h4_top1_net_decisions": config.required_h4_top1_delta,
        },
        "model_heldout": {"status": "NOT_RUN", "reads": 0},
        "promotion": "NOT_AUTHORIZED_BY_CONSTRUCTION",
    }


def run_group_retention_decoder_preflight(
    root: Path,
    *,
    backend: str = "mps",
    _executor: PreflightExecutor | None = None,
    _device_provider: Callable[[str], tuple[torch.device, DeviceTelemetry]] | None = None,
) -> dict[str, Any]:
    """Run one create-once construction terminal; no reveal or main path exists."""
    root = root.resolve()
    config = DecoderPreflightConfig.production()
    if backend != "mps":
        raise ValueError("#973 fuller-decoder construction permits only backend='mps'")
    if any(
        (root / relative).exists() or (root / relative).is_symlink()
        for relative in (STARTED_RELATIVE_PATH, RESULT_RELATIVE_PATH)
    ):
        raise FileExistsError("the sole #973 fuller-decoder preflight is already terminal")
    preparation, geometry, train_sequences, validation_sequences = _load_prepared(root)
    initialization, initial_exports = _initialization_identity(geometry.arms, config.model)
    contract = _contract(config)
    started = _with_cid(
        {
            "schema": STARTED_SCHEMA,
            "issue": ISSUE,
            "policy": POLICY,
            "preparation_manifest_cid": preparation["manifest_cid"],
            "geometry_artifact_cid": geometry.artifact_cid,
            "implementation": trainer_implementation_contract(),
            "initialization": initialization,
            "contract": contract,
            "model_heldout": {"status": "NOT_RUN", "reads": 0},
        },
        "started_cid",
    )
    _write_exclusive_json(root / STARTED_RELATIVE_PATH, started)

    executor = _execute_preflight if _executor is None else _executor
    device_provider = _require_mps_device if _device_provider is None else _device_provider
    execution: Mapping[str, Any] | None = None
    failure: dict[str, str] | None = None
    try:
        device, telemetry = device_provider(backend)
        execution = executor(
            root,
            train_sequences,
            validation_sequences,
            geometry.arms,
            device=device,
            telemetry=telemetry,
            config=config,
            initial_exports=initial_exports,
        )
        if not isinstance(execution, Mapping):
            raise RuntimeError("fuller-decoder executor returned no evidence mapping")
    except Exception as error:
        failure = {"type": type(error).__name__, "reason": str(error)}

    mechanical_pass = bool(
        failure is None
        and execution is not None
        and isinstance(execution.get("mechanical"), Mapping)
        and execution["mechanical"].get("passed") is True
    )
    scientific_available = bool(
        mechanical_pass
        and execution is not None
        and execution.get("available") is True
        and execution.get("wall_passed") is True
    )
    retained_pass = bool(
        scientific_available
        and execution is not None
        and execution.get("retained_decoder_pass") is True
        and execution.get("passed") is True
    )
    h4_pass = bool(retained_pass and execution.get("h4_specific_pass") is True)
    if not scientific_available:
        verdict = TERMINAL_UNAVAILABLE
    elif retained_pass:
        verdict = TERMINAL_PASS
    else:
        verdict = TERMINAL_FAIL
    if not retained_pass:
        h4_verdict = H4_SPECIFIC_NOT_EVALUATED
    elif h4_pass:
        h4_verdict = H4_SPECIFIC_PASS
    else:
        h4_verdict = H4_SPECIFIC_MISS
    result = _with_cid(
        {
            "schema": RESULT_SCHEMA,
            "issue": ISSUE,
            "policy": POLICY,
            "started_cid": started["started_cid"],
            "preparation_manifest_cid": preparation["manifest_cid"],
            "geometry_artifact_cid": geometry.artifact_cid,
            "initialization": initialization,
            "contract": contract,
            "construction_execution": dict(execution) if execution is not None else None,
            "verdict": verdict,
            "h4_specific_verdict": h4_verdict,
            "failure": failure,
            "model_heldout": {"status": "NOT_RUN", "reads": 0},
            "promotion": "NOT_AUTHORIZED_BY_CONSTRUCTION",
            "main_command": "ABSENT",
            "reveal_command": "ABSENT",
        },
        "result_cid",
    )
    _write_exclusive_json(root / RESULT_RELATIVE_PATH, result)
    return result
