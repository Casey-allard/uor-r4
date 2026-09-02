"""Open role-tagged associative-first development ladder for issue #1045.

This is intentionally not a terminal campaign.  It reads only the public and
construction boundary created for #1043, initializes a fresh model from the
ordinary source-free artifact, and stops at the first open-development miss.
No function in this module imports or calls the #1043 reveal/finalize path.
"""

from __future__ import annotations

import json
import math
import multiprocessing as mp
import os
import platform
import queue as queue_module
import resource
import time
import traceback
from collections.abc import Mapping, Sequence
from dataclasses import dataclass, replace
from pathlib import Path
from typing import Any, Literal

import torch
from blake3 import blake3
from torch import Tensor
from torch.nn import functional as F

from .group_retention_campaign import load_group_geometry_artifacts
from .h4_spin_frame_sidecar import H4SpinFrameArtifactV1
from .provenance import (
    canonical_json_bytes,
    cid_bytes,
    cid_file,
    trainer_implementation_contract,
)
from .role_tagged_associative import (
    PARAMETER_COUNT,
    R4RoleTaggedAssociativeCurriculumV1,
    RoleTaggedAssociativeQueryOutput,
)
from .role_tagged_associative_data import (
    RoleTaggedBatch,
    RoleTaggedConstruction,
    RoleTaggedExample,
    batch_role_tagged_examples,
    load_role_tagged_construction,
    validate_role_oracle,
)


ISSUE = 1045
POLICY = "R4RoleTaggedAssociativeCurriculumV1"
SEED = 10_045

ROLE_TEXT = 0
ROLE_KEY = 1
ROLE_VALUE = 2
ROLE_QUERY = 3
IGNORE_INDEX = -100

TRAIN_ROWS = 8_192
DEVELOPMENT_ROWS = 1_024
CONTROL_ROWS = 1_704
OVERFIT_ROWS = 32
OVERFIT_UPDATES = 256
MAXIMUM_EPOCHS = 64
QUERY_DECISIONS_PER_ROW = 8
MAXIMUM_QUERY_PRESENTATIONS = (
    TRAIN_ROWS * QUERY_DECISIONS_PER_ROW * MAXIMUM_EPOCHS
)

LEARNING_RATE = 4.64e-4
WEIGHT_DECAY = 0.1
GRADIENT_CLIP = 1.0
TRAIN_REQUIRED_RATE = 0.995
DEVELOPMENT_REQUIRED_RATE = 0.99
CONTROL_REQUIRED_DROP = 0.50
ROLE_ATTRIBUTION_DROP = 0.25
REQUIRED_CONSECUTIVE_PASSES = 2

HARD_WALL_SECONDS = 1_800.0
PREFLIGHT_WALL_SECONDS = 300.0
MEMORY_CEILING_BYTES = 16 * 1024**3
PROJECTION_SAFETY_FACTOR = 1.25

ELIGIBLE_THREADS = (1, 4, 8)
ELIGIBLE_BATCH_SIZES = (16, 32, 64)

INPUT_INITIAL_ARTIFACT = "inputs/ordinary-initialization.safetensors"
INPUT_GEOMETRY = "inputs/r4-group-address-geometry.json"
INPUT_H4_FRAMES = "inputs/h4-spin-frame-sidecar.json"
INPUT_PUBLIC_MANIFEST = "position-kv-binding-data-manifest.json"
INPUT_PUBLIC_COMMITMENT = "evaluation/commitment.json"
INPUT_CONSTRUCTION_MQAR = "construction/mqar.json"
INPUT_CONSTRUCTION_ENGLISH = "construction/english.json"
INPUT_CONSTRUCTION_NATURAL = "construction/natural.u16"
INPUT_CONSTRUCTION_NATURAL_SELECTION = "construction/natural-selection.json"
FORBIDDEN_INPUT_PREFIXES = ("artifact/", "evaluation/sealed/")

PREPARATION_RELATIVE_PATH = "role-tagged-associative-preparation.json"
PREFLIGHT_RELATIVE_PATH = "preflight/role-tagged-associative-preflight.json"
RESULT_RELATIVE_PATH = "run/role-tagged-associative-result.json"
ARTIFACT_RELATIVE_PATH = "artifact/model.safetensors"

PREPARATION_SCHEMA = "uor-r4.role-tagged-associative-preparation/1"
PREFLIGHT_SCHEMA = "uor-r4.role-tagged-associative-preflight/1"
RESULT_SCHEMA = "uor-r4.role-tagged-associative-result/1"

Verdict = Literal[
    "OPEN_MECHANICS_OR_OPTIMIZER_FAILURE",
    "OPEN_PREFLIGHT_UNAVAILABLE",
    "OPEN_MQAR_NOT_LEARNED",
    "OPEN_MQAR_LEARNED",
    "OPEN_R1_INCOMPLETE",
]


@dataclass(frozen=True, slots=True)
class ScoreResult:
    """One exact query-only open-development score."""

    decisions: int
    correct: int
    loss_sum: float
    selected_logits_cid: str
    work: dict[str, int]

    @property
    def rate(self) -> float:
        return self.correct / self.decisions

    @property
    def nll_nats(self) -> float:
        return self.loss_sum / self.decisions

    def record(self) -> dict[str, Any]:
        return {
            "decisions": self.decisions,
            "top1_correct": self.correct,
            "top1_rate": self.rate,
            "nll_nats": self.nll_nats,
            "selected_logits_cid": self.selected_logits_cid,
            "work": self.work,
        }


class TrainingDeadlineExceeded(TimeoutError):
    """A hard-wall stop carrying all completed-batch evidence."""

    def __init__(self, record: Mapping[str, Any]) -> None:
        super().__init__("#1045 R1 exhausted its 1,800-second wall")
        self.record = dict(record)


@dataclass(frozen=True, slots=True)
class ExecutionPlan:
    """One measured Apple-CPU execution candidate."""

    threads: int
    batch_size: int

    def __post_init__(self) -> None:
        if self.threads not in ELIGIBLE_THREADS:
            raise ValueError("threads are outside the frozen #1045 plans")
        if self.batch_size not in ELIGIBLE_BATCH_SIZES:
            raise ValueError("batch size is outside the frozen #1045 plans")

    def record(self) -> dict[str, Any]:
        body: dict[str, Any] = {
            "name": f"cpu-accelerate-{self.threads}t-b{self.batch_size}",
            "backend": "cpu-apple-accelerate",
            "threads": self.threads,
            "workers": 1,
            "batch_size": self.batch_size,
            "cuda": "FORBIDDEN",
            "mps": "FORBIDDEN",
        }
        body["plan_cid"] = cid_bytes(canonical_json_bytes(body))
        return body


def _with_cid(value: Mapping[str, Any], field: str) -> dict[str, Any]:
    if field in value:
        raise ValueError(f"self-CID field already exists: {field}")
    result = dict(value)
    result[field] = cid_bytes(canonical_json_bytes(value))
    return result


def _verify_self_cid(value: Mapping[str, Any], field: str) -> None:
    unsigned = dict(value)
    observed = unsigned.pop(field, None)
    if observed != cid_bytes(canonical_json_bytes(unsigned)):
        raise ValueError(f"{field} does not reproduce")


def _read_json(path: Path, *, cid_field: str) -> dict[str, Any]:
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, UnicodeError, json.JSONDecodeError) as error:
        raise ValueError(f"cannot read {path.name}: {error}") from error
    if not isinstance(value, dict):
        raise ValueError(f"{path.name} is not a JSON object")
    _verify_self_cid(value, cid_field)
    return value


def _write_exclusive_json(path: Path, value: Mapping[str, Any]) -> None:
    payload = canonical_json_bytes(value)
    path.parent.mkdir(parents=True, exist_ok=True)
    descriptor = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o644)
    try:
        with os.fdopen(descriptor, "wb") as output:
            output.write(payload)
            output.flush()
            os.fsync(output.fileno())
    except BaseException:
        try:
            path.unlink()
        except OSError:
            pass
        raise


def _write_atomic(path: Path, payload: bytes) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    temporary = path.with_name(f".{path.name}.{os.getpid()}.tmp")
    descriptor = os.open(
        temporary,
        os.O_WRONLY | os.O_CREAT | os.O_EXCL,
        0o644,
    )
    try:
        with os.fdopen(descriptor, "wb") as output:
            output.write(payload)
            output.flush()
            os.fsync(output.fileno())
        os.replace(temporary, path)
    finally:
        if temporary.exists():
            temporary.unlink()


def _peak_rss_bytes() -> int:
    peak = int(resource.getrusage(resource.RUSAGE_SELF).ru_maxrss)
    # Darwin reports bytes; Linux reports KiB.
    return peak if platform.system() == "Darwin" else peak * 1024


def _source_input_records(source_root: Path) -> list[dict[str, Any]]:
    records: list[dict[str, Any]] = []
    for relative in (
        INPUT_INITIAL_ARTIFACT,
        INPUT_GEOMETRY,
        INPUT_H4_FRAMES,
        INPUT_PUBLIC_MANIFEST,
        INPUT_PUBLIC_COMMITMENT,
        INPUT_CONSTRUCTION_MQAR,
        INPUT_CONSTRUCTION_ENGLISH,
        INPUT_CONSTRUCTION_NATURAL,
        INPUT_CONSTRUCTION_NATURAL_SELECTION,
    ):
        if relative.startswith(FORBIDDEN_INPUT_PREFIXES):
            raise AssertionError("#1045 allowed-input table contains a forbidden path")
        path = source_root / relative
        if not path.is_file():
            raise FileNotFoundError(f"required open #1045 input is absent: {relative}")
        records.append(
            {
                "path": relative,
                "bytes": path.stat().st_size,
                "cid": cid_file(path),
            }
        )
    try:
        manifest = json.loads(
            (source_root / INPUT_PUBLIC_MANIFEST).read_text(encoding="utf-8")
        )
        tokenizer = manifest["tokenizer"]
        tokenizer_path = Path(str(tokenizer["path"])).resolve()
    except (OSError, UnicodeError, json.JSONDecodeError, KeyError, TypeError) as error:
        raise ValueError("#1045 public source manifest lacks its tokenizer") from error
    if not tokenizer_path.is_file() or tokenizer_path.is_symlink():
        raise FileNotFoundError("#1045 external tokenizer is absent or a symlink")
    tokenizer_record = {
        "path": "external/tokenizer.json",
        "source_path": str(tokenizer_path),
        "bytes": tokenizer_path.stat().st_size,
        "cid": cid_file(tokenizer_path),
    }
    if tokenizer_record["cid"] != tokenizer.get("cid"):
        raise ValueError("#1045 external tokenizer differs from the public manifest")
    records.append(tokenizer_record)
    return records


def prepare_role_tagged_associative_development(
    root: Path,
    *,
    source_root: Path,
) -> dict[str, Any]:
    """Bind one new open run root to #1043's construction-only boundary."""

    root = root.resolve()
    source_root = source_root.resolve()
    preparation_path = root / PREPARATION_RELATIVE_PATH
    if preparation_path.exists():
        preparation = _read_json(preparation_path, cid_field="preparation_cid")
        if preparation.get("source_root") != str(source_root):
            raise ValueError("cached #1045 preparation binds another source root")
        return preparation
    if root.exists() and any(root.iterdir()):
        raise FileExistsError("#1045 preparation requires an empty run root")
    records = _source_input_records(source_root)
    body = {
        "schema": PREPARATION_SCHEMA,
        "issue": ISSUE,
        "policy": POLICY,
        "source_issue": 1043,
        "source_root": str(source_root),
        "inputs": records,
        "forbidden_input_prefixes": list(FORBIDDEN_INPUT_PREFIXES),
        "role_codes": {
            "TEXT": ROLE_TEXT,
            "KEY": ROLE_KEY,
            "VALUE": ROLE_VALUE,
            "QUERY": ROLE_QUERY,
        },
        "substrate": {
            "carrier": "uint8/W8-compatible categorical identity",
            "modular_distance_semantics": "NOT_CLAIMED",
            "softmax": "compiler-side-f32-stable",
        },
        "terminal_population": "NONE_OPEN_DEVELOPMENT_ONLY",
        "failed_source_artifact_reads": 0,
        "sealed_input_reads": 0,
    }
    preparation = _with_cid(body, "preparation_cid")
    _write_exclusive_json(preparation_path, preparation)
    return preparation


def select_execution_plan(records: Sequence[Mapping[str, Any]]) -> dict[str, Any]:
    """Choose measured query throughput under the wall and memory bounds."""

    normalized: list[dict[str, Any]] = []
    eligible: list[dict[str, Any]] = []
    expected = {
        (threads, batch_size)
        for threads in ELIGIBLE_THREADS
        for batch_size in ELIGIBLE_BATCH_SIZES
    }
    observed: set[tuple[int, int]] = set()
    for source in records:
        record = dict(source)
        plan = record.get("plan")
        if not isinstance(plan, Mapping):
            raise ValueError("#1045 probe record lacks a plan")
        identity = (int(plan.get("threads", -1)), int(plan.get("batch_size", -1)))
        if identity in observed:
            raise ValueError("#1045 probe repeats an execution plan")
        observed.add(identity)
        seconds = record.get("projected_r1_seconds")
        memory = record.get("peak_memory_bytes")
        finite = all(
            isinstance(value, (int, float))
            and not isinstance(value, bool)
            and math.isfinite(float(value))
            and float(value) >= 0.0
            for value in (seconds, memory)
        )
        record["eligible"] = bool(
            finite
            and record.get("deterministic_replay") is True
            and float(seconds) <= HARD_WALL_SECONDS
            and int(memory) <= MEMORY_CEILING_BYTES
        )
        normalized.append(record)
        if record["eligible"]:
            eligible.append(record)
    if observed != expected:
        raise ValueError("#1045 probe matrix is incomplete")
    selected = min(
        eligible,
        key=lambda value: (
            float(value["projected_r1_seconds"]),
            int(value["plan"]["threads"]),
            int(value["plan"]["batch_size"]),
        ),
        default=None,
    )
    return {
        "available": selected is not None,
        "plans": normalized,
        "selected_plan": None if selected is None else selected["plan"],
        "selected_projection_seconds": (
            None if selected is None else selected["projected_r1_seconds"]
        ),
        "hard_wall_seconds": HARD_WALL_SECONDS,
        "memory_ceiling_bytes": MEMORY_CEILING_BYTES,
    }


def decide_mqar(
    *,
    mechanics_passed: bool,
    preflight_available: bool,
    train_rate: float | None,
    native_development_rate: float | None,
    consecutive_passes: int,
    native_control_rate: float | None,
    current_only_rate: float | None,
    value_permuted_rate: float | None,
    binding_permuted_rate: float | None,
) -> dict[str, Any]:
    """Return the first divergent #1045 decision without interpreting later rungs."""

    if not mechanics_passed:
        verdict: Verdict = "OPEN_MECHANICS_OR_OPTIMIZER_FAILURE"
        action = "stop before R1; repair the role/oracle/optimizer mechanics"
        passed = False
    elif not preflight_available:
        verdict = "OPEN_PREFLIGHT_UNAVAILABLE"
        action = "stop before R1; reduce the bounded CPU batch plan"
        passed = False
    else:
        primary = (train_rate, native_development_rate)
        if any(
            value is None or not math.isfinite(value) or not 0.0 <= value <= 1.0
            for value in primary
        ):
            raise ValueError("finite train/development MQAR rates are required after R1")
        assert train_rate is not None and native_development_rate is not None
        primary_gates = {
            "train_absolute": train_rate >= TRAIN_REQUIRED_RATE,
            "development_absolute": (
                native_development_rate >= DEVELOPMENT_REQUIRED_RATE
            ),
            "two_consecutive_passes": (
                consecutive_passes >= REQUIRED_CONSECUTIVE_PASSES
            ),
        }
        if not all(primary_gates.values()):
            return {
                "verdict": "OPEN_MQAR_NOT_LEARNED",
                "passed": False,
                "gates": primary_gates,
                "action": (
                    "stop and port the stock Zoology MQAR cell as the "
                    "integration control"
                ),
            }
        controls = (
            native_control_rate,
            current_only_rate,
            value_permuted_rate,
            binding_permuted_rate,
        )
        if any(
            value is None or not math.isfinite(value) or not 0.0 <= value <= 1.0
            for value in controls
        ):
            raise ValueError("complete finite MQAR control rates follow a native pass")
        assert native_control_rate is not None
        assert current_only_rate is not None
        assert value_permuted_rate is not None
        assert binding_permuted_rate is not None
        gates = {
            **primary_gates,
            "control_native_absolute": (
                native_control_rate >= DEVELOPMENT_REQUIRED_RATE
            ),
            "current_only_drop": native_control_rate - current_only_rate
            >= CONTROL_REQUIRED_DROP,
            "value_permuted_drop": native_control_rate - value_permuted_rate
            >= CONTROL_REQUIRED_DROP,
            "binding_permuted_drop": native_control_rate - binding_permuted_rate
            >= CONTROL_REQUIRED_DROP,
        }
        passed = all(gates.values())
        if passed:
            verdict = "OPEN_MQAR_LEARNED"
            action = "advance to open English transfer; do not generate"
        else:
            verdict = "OPEN_MQAR_NOT_LEARNED"
            action = "stop and port the stock Zoology MQAR cell as the integration control"
        return {
            "verdict": verdict,
            "passed": passed,
            "gates": gates,
            "action": action,
        }
    return {"verdict": verdict, "passed": passed, "gates": {}, "action": action}


def _configure_cpu(threads: int) -> torch.device:
    if threads not in ELIGIBLE_THREADS:
        raise ValueError("#1045 thread plan is not eligible")
    os.environ["OMP_NUM_THREADS"] = str(threads)
    os.environ["MKL_NUM_THREADS"] = str(threads)
    os.environ["VECLIB_MAXIMUM_THREADS"] = str(threads)
    torch.set_num_threads(threads)
    try:
        torch.set_num_interop_threads(1)
    except RuntimeError:
        if torch.get_num_interop_threads() != 1:
            raise
    return torch.device("cpu")


def _load_geometry_and_frames(source_root: Path) -> tuple[Any, H4SpinFrameArtifactV1]:
    geometry = load_group_geometry_artifacts(source_root / INPUT_GEOMETRY).exact_h4
    frames = H4SpinFrameArtifactV1.load(source_root / INPUT_H4_FRAMES)
    return geometry, frames


def _build_model(
    source_root: Path,
    *,
    device: torch.device,
) -> R4RoleTaggedAssociativeCurriculumV1:
    geometry, frames = _load_geometry_and_frames(source_root)
    payload = (source_root / INPUT_INITIAL_ARTIFACT).read_bytes()
    model = R4RoleTaggedAssociativeCurriculumV1.from_ordinary_artifact(
        payload,
        geometry=geometry,
        frames=frames,
    ).to(device)
    if model.parameter_count() != PARAMETER_COUNT:
        raise RuntimeError("#1045 model parameter count differs")
    if bool(model.role_embedding.weight.detach().count_nonzero()):
        raise RuntimeError("#1045 role table is not zero at ordinary initialization")
    return model


def _load_fitted_model(
    source_root: Path,
    payload: bytes,
    *,
    device: torch.device,
) -> R4RoleTaggedAssociativeCurriculumV1:
    model = _build_model(source_root, device=device)
    model.load_learned_artifact(payload)
    if model.export_learned_artifact() != payload:
        raise RuntimeError("#1045 fitted artifact does not replay byte-identically")
    return model


def _query_output(
    model: R4RoleTaggedAssociativeCurriculumV1,
    batch: RoleTaggedBatch,
    *,
    intervention: str = "native",
    role_off: bool = False,
) -> RoleTaggedAssociativeQueryOutput:
    roles = torch.zeros_like(batch.role_ids) if role_off else batch.role_ids
    output = model(
        batch.input_ids,
        roles,
        batch.targets,
        selected_positions=batch.selected_positions,
        execution="plain",
        intervention=intervention,  # type: ignore[arg-type]
    )
    if not isinstance(output, RoleTaggedAssociativeQueryOutput):
        raise RuntimeError("#1045 query-only path returned the full-logit surface")
    return output


def _permuted_binding_rows(
    rows: Sequence[RoleTaggedExample],
) -> tuple[RoleTaggedExample, ...]:
    """Rotate physical MQAR values while retaining the native query targets."""

    from .position_kv_binding_data import _assignment_cid, _sequence_cid
    from .role_tagged_associative_data import tag_mqar_example

    controls: list[RoleTaggedExample] = []
    for row in rows:
        source = row.source
        if source.population != "mqar":
            raise ValueError("binding permutation is defined only for MQAR")
        tokens = list(source.input_ids)
        native_values = tuple(tokens[index * 4 + 1] for index in range(8))
        rotated = native_values[1:] + native_values[:1]
        for index, value in enumerate(rotated):
            tokens[index * 4 + 1] = value
        # The control intentionally retains native labels.  Reconstruct all
        # integrity fields that depend on the changed physical serialization.
        assignment_cid = _assignment_cid(source.binding_keys, rotated)
        control_source = replace(
            source,
            input_ids=tuple(tokens),
            binding_values=rotated,
            assignment_cid=assignment_cid,
            world_cid=assignment_cid,
            sequence_cid=_sequence_cid(tokens, source.label_ids),
        )
        controls.append(tag_mqar_example(control_source))
    return tuple(controls)


def _audit_record(audit: Any) -> dict[str, Any]:
    names = (
        "execution",
        "intervention",
        "batch_size",
        "token_steps",
        "layers",
        "heads",
        "cache_writes",
        "materialized_attention_scores",
        "admitted_attention_scores",
        "transported_r4_blocks",
        "value_reads",
        "vocabulary_scores",
        "target_reads",
        "source_reads",
        "provider_calls",
        "teacher_calls",
        "future_reads",
        "forbidden_reads",
    )
    record = {name: getattr(audit, name) for name in names}
    if any(
        int(record[name]) != 0
        for name in ("provider_calls", "teacher_calls", "future_reads", "forbidden_reads")
    ):
        raise RuntimeError("#1045 observed a forbidden causal/provider read")
    return record


def _accumulate_work(total: dict[str, int], audit: Any) -> None:
    for name, value in _audit_record(audit).items():
        if isinstance(value, int) and name not in {
            "batch_size",
            "layers",
            "heads",
        }:
            total[name] = total.get(name, 0) + value


def _score_rows(
    model: R4RoleTaggedAssociativeCurriculumV1,
    rows: Sequence[RoleTaggedExample],
    *,
    device: torch.device,
    batch_size: int,
    intervention: str = "native",
    role_off: bool = False,
    deadline: float | None = None,
) -> ScoreResult:
    if not rows:
        raise ValueError("#1045 score population cannot be empty")
    was_training = model.training
    model.eval()
    decisions = 0
    correct = 0
    loss_sum = 0.0
    digest = blake3()
    work: dict[str, int] = {
        "target_reads": 0,
        "provider_calls": 0,
        "teacher_calls": 0,
        "future_reads": 0,
        "forbidden_reads": 0,
    }
    with torch.inference_mode():
        for start in range(0, len(rows), batch_size):
            if deadline is not None and time.monotonic() >= deadline:
                raise TimeoutError("#1045 scoring exhausted the 1,800-second wall")
            batch = batch_role_tagged_examples(
                rows[start : start + batch_size],
                device=device,
            )
            output = _query_output(
                model,
                batch,
                intervention=intervention,
                role_off=role_off,
            )
            targets = output.selected_targets
            if targets is None:
                raise RuntimeError("#1045 score output lacks query targets")
            flat_logits = output.logits.float().reshape(-1, output.logits.shape[-1])
            flat_targets = targets.reshape(-1)
            loss_sum += float(
                F.cross_entropy(flat_logits, flat_targets, reduction="sum")
            )
            predictions = flat_logits.argmax(dim=-1)
            decisions += int(flat_targets.numel())
            correct += int(torch.count_nonzero(predictions == flat_targets))
            cpu_logits = output.logits.detach().cpu().contiguous()
            digest.update(canonical_json_bytes({"shape": list(cpu_logits.shape)}))
            digest.update(cpu_logits.numpy().tobytes(order="C"))
            _accumulate_work(work, output.audit)
            if deadline is not None and time.monotonic() >= deadline:
                raise TimeoutError("#1045 scoring exhausted the 1,800-second wall")
    model.train(was_training)
    expected_decisions = sum(
        sum(label != IGNORE_INDEX for label in row.labels) for row in rows
    )
    if decisions != expected_decisions:
        raise RuntimeError("#1045 MQAR score decision ledger differs")
    if work.get("target_reads") != decisions:
        raise RuntimeError("#1045 MQAR score target-read ledger differs")
    return ScoreResult(
        decisions=decisions,
        correct=correct,
        loss_sum=loss_sum,
        selected_logits_cid=f"blake3:{digest.hexdigest()}",
        work=work,
    )


def _epoch_order(
    rows: Sequence[RoleTaggedExample],
    *,
    epoch: int,
) -> tuple[RoleTaggedExample, ...]:
    if epoch < 0:
        raise ValueError("#1045 epoch cannot be negative")
    prefix = f"uor-r4/1045/mqar/epoch/{epoch}/".encode("ascii")
    return tuple(
        sorted(
            rows,
            key=lambda row: (
                blake3(prefix + row.stable_id.encode("ascii")).digest(),
                row.stable_id,
            ),
        )
    )


def _train_epoch(
    model: R4RoleTaggedAssociativeCurriculumV1,
    optimizer: torch.optim.Optimizer,
    rows: Sequence[RoleTaggedExample],
    *,
    epoch: int,
    device: torch.device,
    batch_size: int,
    deadline: float | None,
) -> dict[str, Any]:
    model.train()
    ordered = _epoch_order(rows, epoch=epoch)
    loss_sum = 0.0
    decisions = 0
    correct = 0
    gradient_norms: list[float] = []
    work: dict[str, int] = {
        "target_reads": 0,
        "provider_calls": 0,
        "teacher_calls": 0,
        "future_reads": 0,
        "forbidden_reads": 0,
    }

    def record(*, complete: bool) -> dict[str, Any]:
        return {
            "epoch": epoch,
            "complete": complete,
            "decisions": decisions,
            "online_top1_correct": correct,
            "online_top1_rate": None if decisions == 0 else correct / decisions,
            "online_nll_nats": None if decisions == 0 else loss_sum / decisions,
            "mean_gradient_norm_before_clip": (
                None if not gradient_norms else sum(gradient_norms) / len(gradient_norms)
            ),
            "maximum_gradient_norm_before_clip": (
                None if not gradient_norms else max(gradient_norms)
            ),
            "work": work,
        }

    for start in range(0, len(ordered), batch_size):
        if deadline is not None and time.monotonic() >= deadline:
            raise TrainingDeadlineExceeded(record(complete=False))
        batch = batch_role_tagged_examples(
            ordered[start : start + batch_size],
            device=device,
        )
        optimizer.zero_grad(set_to_none=True)
        output = _query_output(model, batch)
        if output.loss is None or output.selected_targets is None:
            raise RuntimeError("#1045 training output lacks query loss/targets")
        output.loss.backward()
        gradient_norm = torch.nn.utils.clip_grad_norm_(
            model.parameters(),
            GRADIENT_CLIP,
        )
        optimizer.step()
        if bool(model.role_embedding.weight.detach()[ROLE_TEXT].count_nonzero()):
            raise RuntimeError("#1045 TEXT role row changed from exact zero")
        batch_decisions = int(output.selected_targets.numel())
        decisions += batch_decisions
        loss_sum += float(output.loss.detach()) * batch_decisions
        predictions = output.logits.detach().argmax(dim=-1)
        correct += int(torch.count_nonzero(predictions == output.selected_targets))
        gradient_norms.append(float(gradient_norm))
        _accumulate_work(work, output.audit)
        if deadline is not None and time.monotonic() >= deadline:
            raise TrainingDeadlineExceeded(record(complete=False))
    expected = sum(
        sum(label != IGNORE_INDEX for label in row.labels) for row in rows
    )
    if decisions != expected or work.get("target_reads") != expected:
        raise RuntimeError("#1045 training decision ledger differs")
    return record(complete=True)


def _merge_work(total: dict[str, int], later: Mapping[str, int]) -> None:
    for name, value in later.items():
        total[name] = total.get(name, 0) + int(value)


def _run_overfit_once(
    source_root: Path,
    *,
    threads: int,
) -> dict[str, Any]:
    device = _configure_cpu(threads)
    torch.manual_seed(SEED)
    construction = load_role_tagged_construction(source_root)
    rows = construction.mqar_train[:OVERFIT_ROWS]
    oracle = validate_role_oracle(rows)
    model = _build_model(source_root, device=device).train()
    optimizer = torch.optim.AdamW(
        model.parameters(),
        lr=2.15e-3,
        weight_decay=WEIGHT_DECAY,
    )
    scheduler = torch.optim.lr_scheduler.CosineAnnealingLR(
        optimizer,
        T_max=OVERFIT_UPDATES,
        eta_min=0.0,
    )
    began = time.monotonic()
    deadline = began + PREFLIGHT_WALL_SECONDS
    first_loss: float | None = None
    final_loss: float | None = None
    trace = blake3()
    work: dict[str, int] = {}
    for update in range(OVERFIT_UPDATES):
        record = _train_epoch(
            model,
            optimizer,
            rows,
            epoch=update,
            device=device,
            batch_size=OVERFIT_ROWS,
            deadline=deadline,
        )
        scheduler.step()
        numeric_loss = float(record["online_nll_nats"])
        if first_loss is None:
            first_loss = numeric_loss
        final_loss = numeric_loss
        trace.update(
            canonical_json_bytes(
                {
                    "update": update + 1,
                    "loss": numeric_loss,
                    "correct": record["online_top1_correct"],
                }
            )
        )
        _merge_work(work, record["work"])
    score = _score_rows(
        model,
        rows,
        device=device,
        batch_size=OVERFIT_ROWS,
    )
    assert first_loss is not None and final_loss is not None
    artifact = model.export_learned_artifact()
    return {
        "passed": bool(
            oracle.passed
            and score.correct == score.decisions
            and final_loss < first_loss
        ),
        "updates": OVERFIT_UPDATES,
        "rows": OVERFIT_ROWS,
        "first_loss_nats": first_loss,
        "final_loss_nats": final_loss,
        "score": score.record(),
        "oracle": {
            "rows": oracle.rows,
            "positions": oracle.positions,
            "prefix_checks": oracle.prefix_checks,
            "exact_rows": oracle.exact_rows,
            "label_reads": oracle.label_reads,
            "metadata_reads": oracle.metadata_reads,
            "passed": oracle.passed,
        },
        "loss_trace_cid": f"blake3:{trace.hexdigest()}",
        "artifact_cid": cid_bytes(artifact),
        "artifact_bytes": len(artifact),
        "artifact": artifact,
        "elapsed_seconds": time.monotonic() - began,
        "peak_memory_bytes": _peak_rss_bytes(),
        "work": work,
    }


def _overfit_worker(source_root: str, queue: Any) -> None:
    try:
        first = _run_overfit_once(Path(source_root), threads=4)
        second = _run_overfit_once(Path(source_root), threads=4)
        artifact_equal = first.pop("artifact") == second.pop("artifact")
        deterministic = bool(
            artifact_equal
            and first["loss_trace_cid"] == second["loss_trace_cid"]
            and first["score"]["selected_logits_cid"]
            == second["score"]["selected_logits_cid"]
        )
        first["deterministic_replay"] = deterministic
        first["passed"] = bool(first["passed"] and second["passed"] and deterministic)
        first["repeat_elapsed_seconds"] = second["elapsed_seconds"]
        first["peak_memory_bytes"] = max(
            int(first["peak_memory_bytes"]), int(second["peak_memory_bytes"])
        )
        queue.put({"ok": True, "result": first})
    except BaseException as error:
        queue.put(
            {
                "ok": False,
                "error": {
                    "type": type(error).__name__,
                    "reason": str(error),
                    "traceback": traceback.format_exc(),
                },
            }
        )


def _probe_trajectory(
    source_root: Path,
    rows: Sequence[RoleTaggedExample],
    *,
    device: torch.device,
    batch_size: int,
) -> dict[str, Any]:
    torch.manual_seed(SEED)
    model = _build_model(source_root, device=device).train()
    optimizer = torch.optim.AdamW(
        model.parameters(),
        lr=LEARNING_RATE,
        weight_decay=WEIGHT_DECAY,
    )
    losses: list[float] = []
    measured: list[float] = []
    for offset in range(4):
        began = time.monotonic()
        record = _train_epoch(
            model,
            optimizer,
            rows,
            epoch=offset,
            device=device,
            batch_size=batch_size,
            deadline=None,
        )
        elapsed = time.monotonic() - began
        losses.append(float(record["online_nll_nats"]))
        if offset > 0:
            measured.append(elapsed)
    began = time.monotonic()
    score = _score_rows(
        model,
        rows,
        device=device,
        batch_size=batch_size,
    )
    evaluation_seconds = time.monotonic() - began
    artifact = model.export_learned_artifact()
    return {
        "artifact": artifact,
        "artifact_cid": cid_bytes(artifact),
        "losses": losses,
        "mean_train_batch_seconds": sum(measured) / len(measured),
        "evaluation_batch_seconds": evaluation_seconds,
        "score_cid": score.selected_logits_cid,
    }


def _probe_thread(source_root: Path, *, threads: int) -> list[dict[str, Any]]:
    device = _configure_cpu(threads)
    construction = load_role_tagged_construction(source_root)
    records: list[dict[str, Any]] = []
    for batch_size in ELIGIBLE_BATCH_SIZES:
        rows = construction.mqar_train[:batch_size]
        first = _probe_trajectory(
            source_root,
            rows,
            device=device,
            batch_size=batch_size,
        )
        second = _probe_trajectory(
            source_root,
            rows,
            device=device,
            batch_size=batch_size,
        )
        train_seconds = max(
            float(first["mean_train_batch_seconds"]),
            float(second["mean_train_batch_seconds"]),
        )
        evaluation_seconds = max(
            float(first["evaluation_batch_seconds"]),
            float(second["evaluation_batch_seconds"]),
        )
        train_batches = math.ceil(TRAIN_ROWS / batch_size)
        development_batches = math.ceil(DEVELOPMENT_ROWS / batch_size)
        control_batches = math.ceil(CONTROL_ROWS / batch_size)
        raw_projection = (
            train_seconds * train_batches * MAXIMUM_EPOCHS
            + evaluation_seconds * development_batches * MAXIMUM_EPOCHS
            + evaluation_seconds * train_batches * MAXIMUM_EPOCHS
            + evaluation_seconds * control_batches * 5
        )
        deterministic = bool(
            first["artifact"] == second["artifact"]
            and first["losses"] == second["losses"]
            and first["score_cid"] == second["score_cid"]
        )
        records.append(
            {
                "plan": ExecutionPlan(threads, batch_size).record(),
                "deterministic_replay": deterministic,
                "mean_train_batch_seconds": train_seconds,
                "evaluation_batch_seconds": evaluation_seconds,
                "projected_r1_seconds": PROJECTION_SAFETY_FACTOR * raw_projection,
                "projection_safety_factor": PROJECTION_SAFETY_FACTOR,
                "peak_memory_bytes": _peak_rss_bytes(),
                "probe_artifact_cid": first["artifact_cid"],
                "probe_score_cid": first["score_cid"],
                "losses": first["losses"],
            }
        )
    return records


def _probe_worker(source_root: str, threads: int, queue: Any) -> None:
    try:
        queue.put(
            {
                "ok": True,
                "records": _probe_thread(Path(source_root), threads=threads),
            }
        )
    except BaseException as error:
        queue.put(
            {
                "ok": False,
                "threads": threads,
                "error": {
                    "type": type(error).__name__,
                    "reason": str(error),
                    "traceback": traceback.format_exc(),
                },
            }
        )


def _spawn_worker(
    target: Any,
    arguments: tuple[Any, ...],
    *,
    timeout_seconds: float,
) -> dict[str, Any]:
    context = mp.get_context("spawn")
    queue = context.Queue()
    process = context.Process(target=target, args=(*arguments, queue))
    process.start()
    process.join(timeout=timeout_seconds)
    if process.is_alive():
        process.terminate()
        process.join(timeout=10.0)
        return {
            "ok": False,
            "error": {
                "type": "TimeoutError",
                "reason": f"worker exceeded {timeout_seconds:.0f} seconds",
            },
        }
    try:
        result = dict(queue.get(timeout=2.0))
    except queue_module.Empty:
        return {
            "ok": False,
            "error": {
                "type": "RuntimeError",
                "reason": f"worker exited {process.exitcode} without evidence",
            },
        }
    finally:
        queue.close()
        queue.join_thread()
    return result


def preflight_role_tagged_associative_development(root: Path) -> dict[str, Any]:
    """Run the binding oracle, disposable overfit, and measured CPU admission."""

    root = root.resolve()
    path = root / PREFLIGHT_RELATIVE_PATH
    if path.exists():
        preflight = _read_json(path, cid_field="preflight_cid")
        if preflight.get("implementation") != trainer_implementation_contract():
            raise ValueError("#1045 trainer implementation changed after preflight")
        return preflight
    implementation = trainer_implementation_contract()
    preparation = _read_json(
        root / PREPARATION_RELATIVE_PATH,
        cid_field="preparation_cid",
    )
    source_root = Path(str(preparation["source_root"]))
    if preparation.get("inputs") != _source_input_records(source_root):
        raise ValueError("#1045 source inputs changed after preparation")
    began = time.monotonic()
    construction = load_role_tagged_construction(source_root)
    oracle = validate_role_oracle(construction.mqar_train[:OVERFIT_ROWS])
    overfit_envelope = _spawn_worker(
        _overfit_worker,
        (str(source_root),),
        timeout_seconds=max(1.0, PREFLIGHT_WALL_SECONDS - (time.monotonic() - began)),
    )
    overfit = overfit_envelope.get("result")
    mechanics_passed = bool(
        overfit_envelope.get("ok") is True
        and isinstance(overfit, Mapping)
        and overfit.get("passed") is True
        and oracle.passed
    )
    probe_records: list[dict[str, Any]] = []
    if mechanics_passed:
        for threads in ELIGIBLE_THREADS:
            remaining = PREFLIGHT_WALL_SECONDS - (time.monotonic() - began)
            if remaining <= 0.0:
                envelope: dict[str, Any] = {
                    "ok": False,
                    "error": {
                        "type": "TimeoutError",
                        "reason": "#1045 preflight exhausted its 300-second wall",
                    },
                }
            else:
                envelope = _spawn_worker(
                    _probe_worker,
                    (str(source_root), threads),
                    timeout_seconds=min(remaining, 90.0),
                )
            records = envelope.get("records")
            if envelope.get("ok") is True and isinstance(records, list):
                probe_records.extend(dict(record) for record in records)
            else:
                for batch_size in ELIGIBLE_BATCH_SIZES:
                    probe_records.append(
                        {
                            "plan": ExecutionPlan(threads, batch_size).record(),
                            "deterministic_replay": False,
                            "projected_r1_seconds": HARD_WALL_SECONDS + 1.0,
                            "peak_memory_bytes": 0,
                            "error": envelope.get("error"),
                        }
                    )
    else:
        for threads in ELIGIBLE_THREADS:
            for batch_size in ELIGIBLE_BATCH_SIZES:
                probe_records.append(
                    {
                        "plan": ExecutionPlan(threads, batch_size).record(),
                        "deterministic_replay": False,
                        "projected_r1_seconds": HARD_WALL_SECONDS + 1.0,
                        "peak_memory_bytes": 0,
                        "error": {
                            "type": "NotRun",
                            "reason": "R0 mechanics did not pass",
                        },
                    }
                )
    selection = select_execution_plan(probe_records)
    elapsed = time.monotonic() - began
    if implementation != trainer_implementation_contract():
        raise ValueError("#1045 trainer implementation changed during preflight")
    body = {
        "schema": PREFLIGHT_SCHEMA,
        "issue": ISSUE,
        "policy": POLICY,
        "preparation_cid": preparation["preparation_cid"],
        "implementation": implementation,
        "source_split_cid": construction.split_cid,
        "counts": {
            "mqar_train": len(construction.mqar_train),
            "mqar_development": len(construction.mqar_development),
            "mqar_controls": len(construction.mqar_controls),
        },
        "role_oracle": {
            "rows": oracle.rows,
            "positions": oracle.positions,
            "prefix_checks": oracle.prefix_checks,
            "exact_rows": oracle.exact_rows,
            "label_reads": oracle.label_reads,
            "metadata_reads": oracle.metadata_reads,
            "passed": oracle.passed,
        },
        "overfit": overfit if isinstance(overfit, Mapping) else overfit_envelope,
        "selection": selection,
        "elapsed_seconds": elapsed,
        "wall_seconds": PREFLIGHT_WALL_SECONDS,
        "passed": bool(
            mechanics_passed
            and selection["available"]
            and elapsed <= PREFLIGHT_WALL_SECONDS
        ),
        "failed_source_artifact_reads": 0,
        "sealed_input_reads": 0,
        "provider_calls": 0,
        "teacher_calls": 0,
        "cuda": "FORBIDDEN",
        "mps": "FORBIDDEN",
    }
    preflight = _with_cid(body, "preflight_cid")
    _write_exclusive_json(path, preflight)
    return preflight


def _result_body(
    *,
    preparation: Mapping[str, Any],
    preflight: Mapping[str, Any],
    plan: Mapping[str, Any] | None,
    artifact: bytes | None,
    fit: Mapping[str, Any],
    metrics: Mapping[str, Any],
    decision: Mapping[str, Any],
    elapsed_seconds: float,
) -> dict[str, Any]:
    artifact_record: dict[str, Any] | None = None
    if artifact is not None:
        artifact_record = {
            "path": ARTIFACT_RELATIVE_PATH,
            "bytes": len(artifact),
            "cid": cid_bytes(artifact),
        }
    return {
        "schema": RESULT_SCHEMA,
        "issue": ISSUE,
        "policy": POLICY,
        "preparation_cid": preparation["preparation_cid"],
        "preflight_cid": preflight["preflight_cid"],
        "implementation": preflight["implementation"],
        "plan": None if plan is None else dict(plan),
        "fit": dict(fit),
        "metrics": dict(metrics),
        "decision": dict(decision),
        "artifact": artifact_record,
        "elapsed_seconds": elapsed_seconds,
        "hard_wall_seconds": HARD_WALL_SECONDS,
        "maximum_query_presentations": MAXIMUM_QUERY_PRESENTATIONS,
        "later_rungs": _later_rungs(
            "AUTHORIZED_NOT_RUN" if decision.get("passed") is True else "NOT_RUN"
        ),
        "population": "OPEN_CONSTRUCTION_AND_DEVELOPMENT_ONLY",
        "failed_source_artifact_reads": 0,
        "sealed_input_reads": 0,
        "provider_calls": 0,
        "teacher_calls": 0,
        "future_reads": 0,
        "forbidden_reads": 0,
        "generation": "NOT_RUN",
        "reasoning": "NOT_RUN",
        "lowering": "NOT_RUN",
    }


def run_role_tagged_associative_development(root: Path) -> dict[str, Any]:
    """Execute R1 from a fresh ordinary artifact and stop at its first result."""

    root = root.resolve()
    result_path = root / RESULT_RELATIVE_PATH
    if result_path.exists():
        return verify_role_tagged_associative_development(root)
    preparation = _read_json(
        root / PREPARATION_RELATIVE_PATH,
        cid_field="preparation_cid",
    )
    preflight = _read_json(
        root / PREFLIGHT_RELATIVE_PATH,
        cid_field="preflight_cid",
    )
    if preflight.get("preparation_cid") != preparation["preparation_cid"]:
        raise ValueError("#1045 preflight binds another preparation")
    implementation = preflight.get("implementation")
    if implementation != trainer_implementation_contract():
        raise ValueError("#1045 trainer implementation changed after preflight")
    selection = preflight.get("selection")
    selected_plan = (
        selection.get("selected_plan") if isinstance(selection, Mapping) else None
    )
    mechanics_passed = bool(
        isinstance(preflight.get("overfit"), Mapping)
        and preflight["overfit"].get("passed") is True
        and isinstance(preflight.get("role_oracle"), Mapping)
        and preflight["role_oracle"].get("passed") is True
    )
    if preflight.get("passed") is not True or not isinstance(selected_plan, Mapping):
        decision = decide_mqar(
            mechanics_passed=mechanics_passed,
            preflight_available=False,
            train_rate=None,
            native_development_rate=None,
            consecutive_passes=0,
            native_control_rate=None,
            current_only_rate=None,
            value_permuted_rate=None,
            binding_permuted_rate=None,
        )
        body = _result_body(
            preparation=preparation,
            preflight=preflight,
            plan=None,
            artifact=None,
            fit={"status": "NOT_RUN_PREFLIGHT"},
            metrics={},
            decision=decision,
            elapsed_seconds=0.0,
        )
        result = _with_cid(body, "result_cid")
        _write_exclusive_json(result_path, result)
        return result

    threads = int(selected_plan["threads"])
    batch_size = int(selected_plan["batch_size"])
    plan = ExecutionPlan(threads, batch_size).record()
    if plan != dict(selected_plan):
        raise ValueError("#1045 selected CPU plan does not reproduce")
    device = _configure_cpu(threads)
    source_root = Path(str(preparation["source_root"]))
    if preparation.get("inputs") != _source_input_records(source_root):
        raise ValueError("#1045 source inputs changed before R1")
    construction = load_role_tagged_construction(source_root)
    if construction.split_cid != preflight.get("source_split_cid"):
        raise ValueError("#1045 open split changed after preflight")
    torch.manual_seed(SEED)
    model = _build_model(source_root, device=device).train()
    optimizer = torch.optim.AdamW(
        model.parameters(),
        lr=LEARNING_RATE,
        weight_decay=WEIGHT_DECAY,
    )
    scheduler = torch.optim.lr_scheduler.CosineAnnealingLR(
        optimizer,
        T_max=MAXIMUM_EPOCHS,
        eta_min=0.0,
    )
    began = time.monotonic()
    deadline = began + HARD_WALL_SECONDS
    history: list[dict[str, Any]] = []
    consecutive = 0
    presentations = 0
    final_development: ScoreResult | None = None
    final_train: ScoreResult | None = None
    incomplete_reason: str | None = None
    try:
        for epoch in range(MAXIMUM_EPOCHS):
            training = _train_epoch(
                model,
                optimizer,
                construction.mqar_train,
                epoch=epoch,
                device=device,
                batch_size=batch_size,
                deadline=deadline,
            )
            scheduler.step()
            presentations += int(training["decisions"])
            final_development = None
            final_train = None
            try:
                final_development = _score_rows(
                    model,
                    construction.mqar_development,
                    device=device,
                    batch_size=batch_size,
                    deadline=deadline,
                )
                if final_development.rate >= DEVELOPMENT_REQUIRED_RATE:
                    final_train = _score_rows(
                        model,
                        construction.mqar_train,
                        device=device,
                        batch_size=batch_size,
                        deadline=deadline,
                    )
            except TimeoutError as error:
                history.append(
                    {
                        "epoch": epoch + 1,
                        "learning_rate": float(scheduler.get_last_lr()[0]),
                        "training": training,
                        "development": (
                            None
                            if final_development is None
                            else final_development.record()
                        ),
                        "train_evaluation": None,
                        "consecutive_passes": consecutive,
                        "elapsed_seconds": time.monotonic() - began,
                    }
                )
                incomplete_reason = str(error)
                break
            passed_epoch = bool(
                final_train is not None
                and final_train.rate >= TRAIN_REQUIRED_RATE
                and final_development.rate >= DEVELOPMENT_REQUIRED_RATE
            )
            consecutive = consecutive + 1 if passed_epoch else 0
            history.append(
                {
                    "epoch": epoch + 1,
                    "learning_rate": float(scheduler.get_last_lr()[0]),
                    "training": training,
                    "development": final_development.record(),
                    "train_evaluation": (
                        None if final_train is None else final_train.record()
                    ),
                    "consecutive_passes": consecutive,
                    "elapsed_seconds": time.monotonic() - began,
                }
            )
            if consecutive >= REQUIRED_CONSECUTIVE_PASSES:
                break
    except TrainingDeadlineExceeded as error:
        partial = error.record
        presentations += int(partial["decisions"])
        history.append(
            {
                "epoch": len(history) + 1,
                "learning_rate": float(optimizer.param_groups[0]["lr"]),
                "training": partial,
                "development": None,
                "train_evaluation": None,
                "consecutive_passes": consecutive,
                "elapsed_seconds": time.monotonic() - began,
            }
        )
        incomplete_reason = str(error)

    if final_development is None:
        native_record: dict[str, Any] | None = None
    else:
        native_record = final_development.record()
    if final_train is None and final_development is not None and incomplete_reason is None:
        try:
            final_train = _score_rows(
                model,
                construction.mqar_train,
                device=device,
                batch_size=batch_size,
                deadline=deadline,
            )
        except TimeoutError as error:
            incomplete_reason = str(error)

    controls: dict[str, Any] = {
        "native": "NOT_RUN_PRIMARY_MISS",
        "role_off": "NOT_RUN_NATIVE_MISS",
        "current_only": "NOT_RUN_NATIVE_MISS",
        "value_permuted": "NOT_RUN_NATIVE_MISS",
        "binding_permuted": "NOT_RUN_NATIVE_MISS",
        "attention_off": "UNAVAILABLE_FROZEN_MECHANICS",
    }
    control_rates: dict[str, float | None] = {
        "native": None,
        "current_only": None,
        "value_permuted": None,
        "binding_permuted": None,
    }
    role_attribution: dict[str, Any] = {"status": "NOT_RUN_NATIVE_MISS"}
    if (
        incomplete_reason is None
        and final_development is not None
        and final_train is not None
        and final_train.rate >= TRAIN_REQUIRED_RATE
        and final_development.rate >= DEVELOPMENT_REQUIRED_RATE
        and consecutive >= REQUIRED_CONSECUTIVE_PASSES
    ):
        try:
            control_rows = construction.mqar_controls
            control_native = _score_rows(
                model,
                control_rows,
                device=device,
                batch_size=batch_size,
                deadline=deadline,
            )
            role_off = _score_rows(
                model,
                control_rows,
                device=device,
                batch_size=batch_size,
                role_off=True,
                deadline=deadline,
            )
            current_only = _score_rows(
                model,
                control_rows,
                device=device,
                batch_size=batch_size,
                intervention="current_only",
                deadline=deadline,
            )
            value_permuted = _score_rows(
                model,
                control_rows,
                device=device,
                batch_size=batch_size,
                intervention="value_permuted",
                deadline=deadline,
            )
            binding_rows = _permuted_binding_rows(control_rows)
            binding_permuted = _score_rows(
                model,
                binding_rows,
                device=device,
                batch_size=batch_size,
                deadline=deadline,
            )
            controls.update(
                {
                    "native": control_native.record(),
                    "role_off": role_off.record(),
                    "current_only": current_only.record(),
                    "value_permuted": value_permuted.record(),
                    "binding_permuted": binding_permuted.record(),
                }
            )
            control_rates = {
                "native": control_native.rate,
                "current_only": current_only.rate,
                "value_permuted": value_permuted.rate,
                "binding_permuted": binding_permuted.rate,
            }
            role_drop = control_native.rate - role_off.rate
            role_attribution = {
                "status": (
                    "ATTRIBUTED"
                    if role_drop >= ROLE_ATTRIBUTION_DROP
                    else "NOT_ATTRIBUTED"
                ),
                "drop": role_drop,
                "required_drop": ROLE_ATTRIBUTION_DROP,
                "gating": False,
            }
        except TimeoutError as error:
            incomplete_reason = str(error)

    if incomplete_reason is None and time.monotonic() >= deadline:
        incomplete_reason = "#1045 R1 exhausted its 1,800-second wall"

    artifact = model.export_learned_artifact()
    _write_atomic(root / ARTIFACT_RELATIVE_PATH, artifact)
    elapsed_seconds = time.monotonic() - began
    if incomplete_reason is None and elapsed_seconds >= HARD_WALL_SECONDS:
        incomplete_reason = "#1045 R1 exhausted its 1,800-second wall"
    if incomplete_reason is not None:
        decision: dict[str, Any] = {
            "verdict": "OPEN_R1_INCOMPLETE",
            "passed": False,
            "gates": {},
            "action": "resume or redesign only after accounting for the bounded interruption",
            "reason": incomplete_reason,
        }
    else:
        decision = decide_mqar(
            mechanics_passed=True,
            preflight_available=True,
            train_rate=None if final_train is None else final_train.rate,
            native_development_rate=(
                None if final_development is None else final_development.rate
            ),
            consecutive_passes=consecutive,
            native_control_rate=control_rates["native"],
            current_only_rate=control_rates["current_only"],
            value_permuted_rate=control_rates["value_permuted"],
            binding_permuted_rate=control_rates["binding_permuted"],
        )
    metrics = {
        "mqar": {
            "train": None if final_train is None else final_train.record(),
            "development": native_record,
            "controls": controls,
            "role_attribution": role_attribution,
        },
        "work_and_leakage": {
            "provider_calls": 0,
            "teacher_calls": 0,
            "future_reads": 0,
            "forbidden_reads": 0,
        },
    }
    fit = {
        "status": "INCOMPLETE" if incomplete_reason is not None else "COMPLETE",
        "epochs": len(history),
        "query_presentations": presentations,
        "history": history,
        "optimizer": {
            "name": "AdamW",
            "learning_rate": LEARNING_RATE,
            "weight_decay": WEIGHT_DECAY,
            "gradient_clip": GRADIENT_CLIP,
            "schedule": "CosineAnnealingLR(epoch)",
            "maximum_epochs": MAXIMUM_EPOCHS,
            "checkpoint_selection": "first two consecutive open passes",
        },
    }
    if implementation != trainer_implementation_contract():
        raise ValueError("#1045 trainer implementation changed during R1")
    body = _result_body(
        preparation=preparation,
        preflight=preflight,
        plan=plan,
        artifact=artifact,
        fit=fit,
        metrics=metrics,
        decision=decision,
        elapsed_seconds=elapsed_seconds,
    )
    result = _with_cid(body, "result_cid")
    _write_exclusive_json(result_path, result)
    return result


def verify_role_tagged_associative_development(root: Path) -> dict[str, Any]:
    """Recheck one open result, its self-CIDs, inputs, and learned artifact."""

    root = root.resolve()
    preparation = _read_json(
        root / PREPARATION_RELATIVE_PATH,
        cid_field="preparation_cid",
    )
    preflight = _read_json(
        root / PREFLIGHT_RELATIVE_PATH,
        cid_field="preflight_cid",
    )
    result = _read_json(root / RESULT_RELATIVE_PATH, cid_field="result_cid")
    if (
        result.get("schema") != RESULT_SCHEMA
        or result.get("issue") != ISSUE
        or result.get("policy") != POLICY
        or result.get("preparation_cid") != preparation["preparation_cid"]
        or result.get("preflight_cid") != preflight["preflight_cid"]
        or result.get("implementation") != preflight.get("implementation")
    ):
        raise ValueError("#1045 result envelope differs")
    if preflight.get("implementation") != trainer_implementation_contract():
        raise ValueError("#1045 trainer implementation changed after the run")
    source_root = Path(str(preparation["source_root"]))
    if preparation.get("inputs") != _source_input_records(source_root):
        raise ValueError("#1045 source inputs changed after result")
    for name in (
        "failed_source_artifact_reads",
        "sealed_input_reads",
        "provider_calls",
        "teacher_calls",
        "future_reads",
        "forbidden_reads",
    ):
        if result.get(name) != 0:
            raise ValueError(f"#1045 result reports forbidden work: {name}")
    fit = result.get("fit")
    metrics = result.get("metrics")
    decision = result.get("decision")
    if not isinstance(fit, Mapping) or not isinstance(metrics, Mapping) or not isinstance(
        decision, Mapping
    ):
        raise ValueError("#1045 semantic result sections are malformed")
    status = fit.get("status")
    selection = preflight.get("selection")
    selected_plan = (
        selection.get("selected_plan") if isinstance(selection, Mapping) else None
    )
    if status == "NOT_RUN_PREFLIGHT":
        if result.get("plan") is not None or result.get("artifact") is not None:
            raise ValueError("#1045 preflight stop unexpectedly has a plan/artifact")
        mechanics = bool(
            isinstance(preflight.get("overfit"), Mapping)
            and preflight["overfit"].get("passed") is True
            and isinstance(preflight.get("role_oracle"), Mapping)
            and preflight["role_oracle"].get("passed") is True
        )
        expected_decision = decide_mqar(
            mechanics_passed=mechanics,
            preflight_available=False,
            train_rate=None,
            native_development_rate=None,
            consecutive_passes=0,
            native_control_rate=None,
            current_only_rate=None,
            value_permuted_rate=None,
            binding_permuted_rate=None,
        )
        if dict(decision) != expected_decision:
            raise ValueError("#1045 preflight-stop decision does not reproduce")
    elif status in ("COMPLETE", "INCOMPLETE"):
        if not isinstance(selected_plan, Mapping) or result.get("plan") != selected_plan:
            raise ValueError("#1045 result plan differs from preflight selection")
        if not isinstance(result.get("artifact"), Mapping):
            raise ValueError("#1045 executed result lacks its learned artifact")
        elapsed_seconds = result.get("elapsed_seconds")
        if (
            isinstance(elapsed_seconds, bool)
            or not isinstance(elapsed_seconds, (int, float))
            or not math.isfinite(float(elapsed_seconds))
            or float(elapsed_seconds) < 0.0
            or (status == "COMPLETE" and float(elapsed_seconds) >= HARD_WALL_SECONDS)
        ):
            raise ValueError("#1045 fit wall ledger differs")
        history = fit.get("history")
        if (
            not isinstance(history, list)
            or fit.get("epochs") != len(history)
            or not 1 <= len(history) <= MAXIMUM_EPOCHS
        ):
            raise ValueError("#1045 fit history length differs")

        def validate_work(work: Any, *, decisions: int, label: str) -> None:
            if not isinstance(work, Mapping) or work.get("target_reads") != decisions:
                raise ValueError(f"#1045 {label} work ledger differs")
            for name in (
                "provider_calls",
                "teacher_calls",
                "future_reads",
                "forbidden_reads",
            ):
                if work.get(name) != 0:
                    raise ValueError(f"#1045 {label} reports forbidden work: {name}")

        def validate_score(value: Any, label: str) -> float | None:
            if not isinstance(value, Mapping):
                return None
            decisions = value.get("decisions")
            correct = value.get("top1_correct")
            rate = value.get("top1_rate")
            if (
                isinstance(decisions, bool)
                or not isinstance(decisions, int)
                or decisions < 1
                or isinstance(correct, bool)
                or not isinstance(correct, int)
                or not 0 <= correct <= decisions
                or not isinstance(rate, (int, float))
                or not math.isclose(
                    float(rate), correct / decisions, rel_tol=0.0, abs_tol=1e-15
                )
            ):
                raise ValueError(f"#1045 {label} score ledger differs")
            validate_work(value.get("work"), decisions=decisions, label=label)
            return float(rate)

        presentations = 0
        reproduced_consecutive = 0
        for expected_epoch, entry in enumerate(history, start=1):
            if not isinstance(entry, Mapping) or entry.get("epoch") != expected_epoch:
                raise ValueError("#1045 fit epoch ordering differs")
            training = entry.get("training")
            if not isinstance(training, Mapping):
                raise ValueError("#1045 fit epoch lacks training evidence")
            decisions = training.get("decisions")
            work = training.get("work")
            if (
                isinstance(decisions, bool)
                or not isinstance(decisions, int)
                or decisions < 0
            ):
                raise ValueError("#1045 fit work/presentation ledger differs")
            validate_work(work, decisions=decisions, label=f"epoch {expected_epoch} training")
            complete = training.get("complete")
            if not isinstance(complete, bool):
                raise ValueError("#1045 training completion ledger differs")
            if status == "COMPLETE" and not complete:
                raise ValueError("#1045 complete fit contains a partial epoch")
            development_epoch_rate = validate_score(
                entry.get("development"), f"epoch {expected_epoch} development"
            )
            train_epoch_rate = validate_score(
                entry.get("train_evaluation"), f"epoch {expected_epoch} train"
            )
            unknown_interrupted_check = bool(
                status == "INCOMPLETE"
                and expected_epoch == len(history)
                and (
                    development_epoch_rate is None
                    or (
                        development_epoch_rate >= DEVELOPMENT_REQUIRED_RATE
                        and train_epoch_rate is None
                    )
                )
            )
            if not unknown_interrupted_check:
                passed_epoch = bool(
                    development_epoch_rate is not None
                    and train_epoch_rate is not None
                    and development_epoch_rate >= DEVELOPMENT_REQUIRED_RATE
                    and train_epoch_rate >= TRAIN_REQUIRED_RATE
                )
                reproduced_consecutive = (
                    reproduced_consecutive + 1 if passed_epoch else 0
                )
            if entry.get("consecutive_passes") != reproduced_consecutive:
                raise ValueError("#1045 consecutive-pass history does not reproduce")
            presentations += decisions
        if (
            fit.get("query_presentations") != presentations
            or presentations > MAXIMUM_QUERY_PRESENTATIONS
        ):
            raise ValueError("#1045 total query presentation ledger differs")
        mqar = metrics.get("mqar")
        if not isinstance(mqar, Mapping):
            raise ValueError("#1045 MQAR metrics are absent")

        train_rate = validate_score(mqar.get("train"), "train")
        development_rate = validate_score(mqar.get("development"), "development")
        controls = mqar.get("controls")
        if not isinstance(controls, Mapping):
            raise ValueError("#1045 MQAR controls are malformed")
        native_control_rate = validate_score(controls.get("native"), "control native")
        current_only_rate = validate_score(controls.get("current_only"), "current-only")
        value_permuted_rate = validate_score(
            controls.get("value_permuted"), "value-permuted"
        )
        binding_permuted_rate = validate_score(
            controls.get("binding_permuted"), "binding-permuted"
        )
        final_consecutive = reproduced_consecutive
        if status == "COMPLETE":
            expected_decision = decide_mqar(
                mechanics_passed=True,
                preflight_available=True,
                train_rate=train_rate,
                native_development_rate=development_rate,
                consecutive_passes=final_consecutive,
                native_control_rate=native_control_rate,
                current_only_rate=current_only_rate,
                value_permuted_rate=value_permuted_rate,
                binding_permuted_rate=binding_permuted_rate,
            )
            if dict(decision) != expected_decision:
                raise ValueError("#1045 final MQAR decision does not reproduce")
        elif decision.get("verdict") != "OPEN_R1_INCOMPLETE" or decision.get(
            "passed"
        ) is not False:
            raise ValueError("#1045 incomplete result has another decision")
    else:
        raise ValueError("#1045 fit status is unknown")
    artifact_record = result.get("artifact")
    if artifact_record is not None:
        if not isinstance(artifact_record, Mapping):
            raise ValueError("#1045 artifact record is malformed")
        artifact_path = root / str(artifact_record.get("path"))
        payload = artifact_path.read_bytes()
        if (
            len(payload) != artifact_record.get("bytes")
            or cid_bytes(payload) != artifact_record.get("cid")
        ):
            raise ValueError("#1045 learned artifact does not reproduce")
        _load_fitted_model(source_root, payload, device=torch.device("cpu"))
    return result


def execute_role_tagged_associative_development(
    root: Path,
    *,
    source_root: Path,
) -> dict[str, Any]:
    """Prepare, preflight, execute, and independently validate the open rung."""

    prepare_role_tagged_associative_development(root, source_root=source_root)
    preflight_role_tagged_associative_development(root)
    run_role_tagged_associative_development(root)
    return verify_role_tagged_associative_development(root)


def _later_rungs(status: str) -> dict[str, Any]:
    return {
        "english_transfer": {
            "status": status,
            "reason": "R2 is conditional on a passing open MQAR result",
        },
        "natural_language_preservation": {
            "status": status,
            "reason": "R3 is conditional on passing MQAR and English transfer",
        },
        "generation": "NOT_RUN_OUT_OF_SCOPE",
        "reasoning": "NOT_RUN_OUT_OF_SCOPE",
        "lowering": "NOT_RUN_OUT_OF_SCOPE",
    }
