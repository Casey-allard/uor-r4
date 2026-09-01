"""Measured-fast execution and one frozen #973 language-path campaign.

The model and population contracts live in ``language_path_generalization``
and ``language_path_generalization_data``.  This module owns only execution
selection, optimizer/recovery mechanics, evaluation, and the predeclared
two-arm decision.  Probe fits are disposable and run in spawned processes so
PyTorch thread pools and devices cannot leak between eligible plans.
"""

from __future__ import annotations

import gc
import json
import math
import multiprocessing as mp
import os
import platform
import resource
import statistics
import tempfile
import time
import traceback
from collections.abc import Callable, Mapping, Sequence
from dataclasses import asdict, dataclass
from pathlib import Path
from queue import Empty
from typing import Any, Literal

import torch
from blake3 import blake3
from torch import Tensor
from torch.nn import functional as F

from .language_path_generalization import (
    CONTEXT,
    INITIALIZATION_SEED,
    PARAMETER_COUNT,
    POLICY,
    STATE_BYTES_F32,
    STATE_VALUES,
    VALIDITY_BITS,
    VOCAB_SIZE,
    OrdinaryCausalSoftmaxLanguagePathV1,
    R4RetainedLanguagePathV1,
    architecture_ledger,
    work_ledger,
)
from .provenance import (
    atomic_write,
    atomic_write_json,
    canonical_json_bytes,
    cid_bytes,
    cid_file,
    trainer_implementation_contract,
)


ISSUE = 973
ARMS = ("retained", "ordinary")

TRAIN_WINDOWS = 43_680
VALIDATION_WINDOWS = 2_066
WINDOW_TOKENS = CONTEXT + 1
TRAIN_DECISIONS = TRAIN_WINDOWS * CONTEXT
VALIDATION_DECISIONS = VALIDATION_WINDOWS * CONTEXT
REACHABLE_VALIDATION_DECISIONS = VALIDATION_DECISIONS - VALIDATION_WINDOWS

BATCH_SIZE = 16
OPTIMIZER_STEPS = TRAIN_WINDOWS // BATCH_SIZE
WARMUP_STEPS = 100
MAXIMUM_LEARNING_RATE = 3e-4
MINIMUM_LEARNING_RATE = 3e-5
ADAM_BETA1 = 0.9
ADAM_BETA2 = 0.95
ADAM_EPSILON = 1e-8
WEIGHT_DECAY = 0.1
GRADIENT_CLIP = 1.0

PROBE_WARMUP_STEPS = 1
PROBE_MEASURED_STEPS = 5
PROJECTION_SAFETY_FACTOR = 1.25
MEMORY_FRACTION_CEILING = 0.80
WALL_CEILING_SECONDS = 7_200.0
PROGRESS_INTERVAL = 10
CHECKPOINT_INTERVAL = 100
EQUIVALENCE_ABS_TOLERANCE = 5e-5

ORDINARY_REQUIRED_NLL_IMPROVEMENT = 1.0
ORDINARY_REQUIRED_TOP1_POINT_IMPROVEMENT = 5.0
RETAINED_REQUIRED_NLL_IMPROVEMENT = 1.0
RETAINED_REQUIRED_TOP1_POINT_IMPROVEMENT = 5.0
REQUIRED_STATE_OFF_NLL_DELTA = 0.10
REQUIRED_STATE_OFF_TOP1_DECISIONS = 2_480
COMPETITIVE_NLL_TOLERANCE = 0.20
COMPETITIVE_TOP1_POINT_TOLERANCE = 2.0

PROBE_RELATIVE_PATH = "preflight/language-path-execution-probe.json"
STARTED_RELATIVE_PATH = "run/language-path-started.json"
RESULT_RELATIVE_PATH = "run/language-path-result.json"

PROBE_SCHEMA = "uor-r4.retained-language-path-execution-probe/1"
STARTED_SCHEMA = "uor-r4.retained-language-path-started/1"
RESULT_SCHEMA = "uor-r4.retained-language-path-result/1"
CHECKPOINT_SCHEMA = "uor-r4.retained-language-path-checkpoint/1"
ARM_RESULT_SCHEMA = "uor-r4.retained-language-path-arm-result/1"

TERMINAL_PASS = "RETAINED_LANGUAGE_PATH_PASS"
TERMINAL_NOT_COMPETITIVE = "GENERALIZES_BUT_NOT_COMPETITIVE"
TERMINAL_RETAINED_FAIL = "RETAINED_LANGUAGE_PATH_FAIL"
TERMINAL_INVALID_RECIPE = "INVALID_LANGUAGE_RECIPE"
TERMINAL_UNAVAILABLE = "UNAVAILABLE_COMPUTE"
TERMINAL_INVALID_IMPLEMENTATION = "INVALID_LANGUAGE_IMPLEMENTATION"


@dataclass(frozen=True, slots=True)
class ExecutionPlan:
    """One eligible deterministic backend/thread/worker arrangement."""

    name: str
    backend: Literal["cpu", "mps"]
    threads_per_worker: int
    workers: int
    concurrent_arms: bool

    def validate(self) -> None:
        if self.backend not in ("cpu", "mps"):
            raise ValueError("language-path execution backend must be cpu or mps")
        if self.threads_per_worker < 1 or self.workers < 1:
            raise ValueError("language-path execution threads/workers must be positive")
        if self.concurrent_arms != (self.workers == 2):
            raise ValueError("only the frozen two-worker concurrent plan may overlap arms")
        if self.backend == "mps" and (self.workers != 1 or self.concurrent_arms):
            raise ValueError("the frozen MPS plan is sequential")

    def identity(self) -> dict[str, Any]:
        self.validate()
        value = asdict(self)
        value["cuda"] = "FORBIDDEN"
        value["plan_cid"] = cid_bytes(canonical_json_bytes(value))
        return value


ELIGIBLE_PLANS = (
    ExecutionPlan("cpu-accelerate-4t-sequential", "cpu", 4, 1, False),
    ExecutionPlan("cpu-accelerate-8t-sequential", "cpu", 8, 1, False),
    ExecutionPlan("cpu-accelerate-2x2t-concurrent", "cpu", 2, 2, True),
    ExecutionPlan("mps-deterministic-sequential", "mps", 1, 1, False),
)


def learning_rate(step: int) -> float:
    """Return the frozen warmup/cosine AdamW learning rate."""

    if not 0 <= step <= OPTIMIZER_STEPS:
        raise ValueError("language-path optimizer step is outside the frozen epoch")
    if step <= WARMUP_STEPS:
        return MAXIMUM_LEARNING_RATE * step / WARMUP_STEPS
    progress = (step - WARMUP_STEPS) / (OPTIMIZER_STEPS - WARMUP_STEPS)
    cosine = 0.5 * (1.0 + math.cos(math.pi * progress))
    return MINIMUM_LEARNING_RATE + cosine * (
        MAXIMUM_LEARNING_RATE - MINIMUM_LEARNING_RATE
    )


def _wall_exhausted(
    *, elapsed_before_seconds: float, elapsed_current_seconds: float, arm_ceiling_seconds: float
) -> bool:
    """Apply one arm's share of the whole-process wall without double counting."""

    if min(elapsed_before_seconds, elapsed_current_seconds, arm_ceiling_seconds) < 0.0:
        raise ValueError("language-path wall accounting cannot be negative")
    return elapsed_before_seconds + elapsed_current_seconds >= arm_ceiling_seconds


def _arm_wall_ceiling(*, concurrent: bool, completed_other_arm_seconds: float) -> float:
    if completed_other_arm_seconds < 0.0:
        raise ValueError("completed arm time cannot be negative")
    return WALL_CEILING_SECONDS if concurrent else max(
        0.0, WALL_CEILING_SECONDS - completed_other_arm_seconds
    )


def _with_cid(value: Mapping[str, Any], field: str) -> dict[str, Any]:
    if field in value:
        raise ValueError(f"self-CID field already exists: {field}")
    result = dict(value)
    result[field] = cid_bytes(canonical_json_bytes(value))
    return result


def _verify_self_cid(value: Mapping[str, Any], field: str) -> None:
    expected = value.get(field)
    unsigned = dict(value)
    unsigned.pop(field, None)
    if expected != cid_bytes(canonical_json_bytes(unsigned)):
        raise ValueError(f"{field} does not reproduce")


def _load_json(path: Path) -> dict[str, Any]:
    value = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(value, dict):
        raise ValueError(f"expected a JSON object: {path}")
    return value


def _write_exclusive_json(path: Path, value: Mapping[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    descriptor = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
    try:
        with os.fdopen(descriptor, "wb", closefd=True) as target:
            descriptor = -1
            target.write(canonical_json_bytes(value))
            target.flush()
            os.fsync(target.fileno())
    finally:
        if descriptor >= 0:
            os.close(descriptor)


def _physical_memory_bytes() -> int:
    return int(os.sysconf("SC_PAGE_SIZE")) * int(os.sysconf("SC_PHYS_PAGES"))


def _peak_rss_bytes() -> int:
    value = int(resource.getrusage(resource.RUSAGE_SELF).ru_maxrss)
    return value if platform.system() == "Darwin" else value * 1_024


def _sync(device: torch.device) -> None:
    if device.type == "mps":
        torch.mps.synchronize()


def _configure_device(plan: ExecutionPlan) -> tuple[torch.device, dict[str, Any]]:
    plan.validate()
    if torch.cuda.is_available() and os.environ.get("CUDA_VISIBLE_DEVICES") not in (
        None,
        "",
        "-1",
    ):
        raise RuntimeError("CUDA is forbidden by the language-path contract")
    torch.use_deterministic_algorithms(True)
    torch.manual_seed(INITIALIZATION_SEED)
    if plan.backend == "cpu":
        if platform.system() != "Darwin":
            raise RuntimeError("the frozen CPU probe requires Darwin")
        build = torch.__config__.show().lower()
        if "blas_info=accelerate" not in build:
            raise RuntimeError("the frozen CPU probe requires Apple Accelerate BLAS")
        os.environ["OMP_NUM_THREADS"] = str(plan.threads_per_worker)
        os.environ["VECLIB_MAXIMUM_THREADS"] = str(plan.threads_per_worker)
        os.environ["OPENBLAS_NUM_THREADS"] = str(plan.threads_per_worker)
        torch.set_num_threads(plan.threads_per_worker)
        try:
            torch.set_num_interop_threads(plan.threads_per_worker)
        except RuntimeError as error:
            if torch.get_num_interop_threads() != plan.threads_per_worker:
                raise RuntimeError("could not establish frozen interop threads") from error
        if (
            torch.get_num_threads() != plan.threads_per_worker
            or torch.get_num_interop_threads() != plan.threads_per_worker
        ):
            raise RuntimeError("CPU thread counts differ from the selected plan")
        return torch.device("cpu"), {
            "platform": platform.system(),
            "blas": "Apple Accelerate",
            "threads": plan.threads_per_worker,
            "memory_budget_bytes": _physical_memory_bytes(),
        }
    if not torch.backends.mps.is_available():
        raise RuntimeError("deterministic MPS is unavailable")
    return torch.device("mps"), {
        "platform": platform.system(),
        "blas": "MPS",
        "threads": 1,
        "memory_budget_bytes": int(torch.mps.recommended_max_memory()),
    }


def _prepared_inputs(root: Path) -> Any:
    """Load the data agent's verified, nonsealed training view lazily."""

    from .language_path_generalization_data import load_language_path_preparation

    return load_language_path_preparation(root)


def _input_field(prepared: Any, *names: str) -> Any:
    for name in names:
        if isinstance(prepared, Mapping) and name in prepared:
            return prepared[name]
        if hasattr(prepared, name):
            return getattr(prepared, name)
    raise AttributeError(f"prepared language-path data lacks any of {names}")


def _train_windows(prepared: Any) -> Any:
    return _input_field(prepared, "train_windows", "training_windows", "train")


def _validation_windows(prepared: Any) -> Any:
    return _input_field(prepared, "validation_windows", "validation")


def _exact_geometry(prepared: Any) -> Any:
    geometry = _input_field(prepared, "geometry", "exact_h4_geometry")
    if hasattr(geometry, "exact_h4"):
        return geometry.exact_h4
    if isinstance(geometry, Mapping) and "exact_h4" in geometry:
        return geometry["exact_h4"]
    return geometry


def _preparation_manifest(prepared: Any) -> Mapping[str, Any]:
    value = _input_field(prepared, "manifest", "preparation_manifest")
    if not isinstance(value, Mapping):
        raise TypeError("language-path preparation manifest is not a mapping")
    return value


def _window_count(windows: Any) -> int:
    if hasattr(windows, "window_count"):
        return int(windows.window_count)
    raw = windows.windows if hasattr(windows, "windows") else windows
    return int(raw.shape[0] if hasattr(raw, "shape") else len(raw))


def _window_batch(windows: Any, start: int, count: int, device: torch.device) -> Tensor:
    raw = windows.windows if hasattr(windows, "windows") else windows
    # The verified token stores are read-only memmaps. Materialize a writable
    # tensor rather than asking PyTorch to alias non-writable storage.
    batch = torch.tensor(raw[start : start + count], dtype=torch.long)
    if batch.ndim != 2 or int(batch.shape[1]) != WINDOW_TOKENS:
        raise ValueError("language-path windows must have shape [N, context+1]")
    return batch.to(device=device)


def _ordered_train_batch(prepared: Any, step: int, device: torch.device) -> Tensor:
    if not 1 <= step <= OPTIMIZER_STEPS:
        raise ValueError("ordered language-path step is outside the frozen epoch")
    order = _input_field(prepared, "train_order")
    start = (step - 1) * BATCH_SIZE
    ordinals = order[start : start + BATCH_SIZE]
    if len(ordinals) != BATCH_SIZE:
        raise ValueError("frozen train order cannot supply a complete batch")
    store = _train_windows(prepared)
    if hasattr(store, "batch"):
        inputs, targets = store.batch(ordinals)
        return torch.cat((inputs[:, :1], targets), dim=1).to(device=device)
    raw = store.windows if hasattr(store, "windows") else store
    return torch.tensor(raw[list(ordinals)], dtype=torch.long).to(device=device)


def _train_order_identity(prepared: Any) -> dict[str, Any]:
    """Bind the exact without-replacement order shared by both fit arms."""

    raw_order = _input_field(prepared, "train_order")
    order = tuple(int(ordinal) for ordinal in raw_order)
    if len(order) != TRAIN_WINDOWS or sorted(order) != list(range(TRAIN_WINDOWS)):
        raise ValueError("language-path train order is not the frozen permutation")
    policy = _preparation_manifest(prepared).get("window_order")
    if not isinstance(policy, Mapping):
        raise ValueError("language-path train-order policy is absent")
    return {
        "windows": len(order),
        "policy": dict(policy),
        "order_cid": cid_bytes(canonical_json_bytes(list(order))),
    }


def _require_current_implementation(
    bound: Any, *, label: str, current: Mapping[str, Any] | None = None
) -> dict[str, Any]:
    """Fail closed when executable trainer files changed after a freeze point."""

    observed = dict(current) if current is not None else trainer_implementation_contract()
    if bound != observed:
        raise ValueError(f"current trainer implementation differs from {label}")
    return observed


def _build_model(arm: str, geometry: Any) -> torch.nn.Module:
    if arm == "retained":
        return R4RetainedLanguagePathV1(geometry)
    if arm == "ordinary":
        return OrdinaryCausalSoftmaxLanguagePathV1()
    raise ValueError(f"unknown language-path arm: {arm}")


def _optimizer(model: torch.nn.Module) -> torch.optim.Optimizer:
    return torch.optim.AdamW(
        model.parameters(),
        lr=learning_rate(0),
        betas=(ADAM_BETA1, ADAM_BETA2),
        eps=ADAM_EPSILON,
        weight_decay=WEIGHT_DECAY,
    )


def _train_step(
    model: torch.nn.Module,
    optimizer: torch.optim.Optimizer,
    batch: Tensor,
    *,
    step: int,
) -> tuple[float, float]:
    model.train()
    optimizer.zero_grad(set_to_none=True)
    output = model(batch[:, :-1], batch[:, 1:])
    if output.loss is None or not bool(torch.isfinite(output.loss)):
        raise RuntimeError("language-path training loss is not finite")
    output.loss.backward()
    gradient_norm = torch.nn.utils.clip_grad_norm_(model.parameters(), GRADIENT_CLIP)
    if not bool(torch.isfinite(gradient_norm)):
        raise RuntimeError("language-path gradient norm is not finite")
    rate = learning_rate(step)
    for group in optimizer.param_groups:
        group["lr"] = rate
    optimizer.step()
    return float(output.loss.detach().cpu()), float(gradient_norm.detach().cpu())


def _probe_vector(model: torch.nn.Module, logits: Tensor) -> list[float]:
    values: list[float] = logits.detach().float().reshape(-1)[:64].cpu().tolist()
    for _, parameter in sorted(model.named_parameters()):
        if len(values) >= 128:
            break
        remaining = 128 - len(values)
        values.extend(parameter.detach().float().reshape(-1)[:remaining].cpu().tolist())
    return values


def _probe_arm(root: Path, arm: str, plan: ExecutionPlan) -> dict[str, Any]:
    device, backend = _configure_device(plan)
    prepared = _prepared_inputs(root)
    train = _train_windows(prepared)
    validation = _validation_windows(prepared)
    geometry = _exact_geometry(prepared)
    if _window_count(train) != TRAIN_WINDOWS or _window_count(validation) != VALIDATION_WINDOWS:
        raise ValueError("language-path probe population counts differ")
    model = _build_model(arm, geometry).to(device)
    optimizer = _optimizer(model)
    measured: list[float] = []
    final_loss = math.nan
    final_gradient_norm = math.nan
    total_probe_steps = PROBE_WARMUP_STEPS + PROBE_MEASURED_STEPS
    for offset in range(total_probe_steps):
        _sync(device)
        started = time.perf_counter()
        batch = _ordered_train_batch(prepared, offset + 1, device)
        final_loss, final_gradient_norm = _train_step(
            model, optimizer, batch, step=offset + 1
        )
        _sync(device)
        elapsed = time.perf_counter() - started
        if offset >= PROBE_WARMUP_STEPS:
            measured.append(elapsed)
    model.eval()
    _sync(device)
    evaluation_started = time.perf_counter()
    evaluation_batch = _window_batch(validation, 0, BATCH_SIZE, device)
    with torch.no_grad():
        evaluation = model(evaluation_batch[:, :-1], evaluation_batch[:, 1:])
    _sync(device)
    evaluation_seconds = time.perf_counter() - evaluation_started
    if evaluation.loss is None or not bool(torch.isfinite(evaluation.loss)):
        raise RuntimeError("language-path probe evaluation loss is not finite")
    forbidden_reads = int(getattr(evaluation.audit, "forbidden_reads", -1))
    if forbidden_reads != 0:
        raise RuntimeError("language-path probe made a forbidden read")
    if device.type == "mps":
        peak_bytes = max(
            int(torch.mps.current_allocated_memory()),
            int(torch.mps.driver_allocated_memory()),
        )
    else:
        peak_bytes = _peak_rss_bytes()
    (root / "preflight").mkdir(parents=True, exist_ok=True)
    artifact_started = time.perf_counter()
    artifact = model.export_learned_artifact()
    with tempfile.NamedTemporaryFile(
        prefix="language-path-probe-artifact-",
        suffix=".safetensors",
        dir=root / "preflight",
        delete=True,
    ) as target:
        target.write(artifact)
        target.flush()
        os.fsync(target.fileno())
        artifact_cid = cid_file(Path(target.name))
    artifact_seconds = time.perf_counter() - artifact_started
    with tempfile.NamedTemporaryFile(
        prefix="language-path-probe-checkpoint-",
        suffix=".pt",
        dir=root / "preflight",
        delete=True,
    ) as target:
        checkpoint_started = time.perf_counter()
        torch.save(
            {
                "model": model.state_dict(),
                "optimizer": optimizer.state_dict(),
                "step": total_probe_steps,
            },
            target.name,
        )
        with open(target.name, "rb+") as written:
            os.fsync(written.fileno())
        checkpoint_seconds = time.perf_counter() - checkpoint_started
        checkpoint_bytes = os.path.getsize(target.name)
        checkpoint_hash_started = time.perf_counter()
        cid_file(Path(target.name))
        checkpoint_hash_seconds = time.perf_counter() - checkpoint_hash_started
    with tempfile.NamedTemporaryFile(
        prefix="language-path-probe-progress-",
        suffix=".json",
        dir=root / "preflight",
        delete=True,
    ) as target:
        progress_started = time.perf_counter()
        target.write(
            canonical_json_bytes(
                {
                    "completed_steps": 10,
                    "total_steps": OPTIMIZER_STEPS,
                    "eta_seconds": 1.0,
                }
            )
        )
        target.flush()
        os.fsync(target.fileno())
        progress_seconds = time.perf_counter() - progress_started
    replay_started = time.perf_counter()
    replay_model = _build_model(arm, geometry).to(device)
    replay_model.load_learned_artifact(artifact)
    replay_model.eval()
    with torch.no_grad():
        replay_output = replay_model(evaluation_batch[:1, :64])
    _sync(device)
    replay_seconds = time.perf_counter() - replay_started
    result = {
        "arm": arm,
        "backend": backend,
        "mean_train_step_seconds": statistics.fmean(measured),
        "median_train_step_seconds": statistics.median(measured),
        "measured_train_step_seconds": measured,
        "evaluation_batch_seconds": evaluation_seconds,
        "checkpoint_seconds": checkpoint_seconds,
        "checkpoint_bytes": checkpoint_bytes,
        "checkpoint_hash_seconds": checkpoint_hash_seconds,
        "artifact_export_seconds": artifact_seconds,
        "progress_write_seconds": progress_seconds,
        "fixed_prefix_replay_seconds": replay_seconds,
        "peak_memory_bytes": peak_bytes,
        "memory_budget_bytes": int(backend["memory_budget_bytes"]),
        "final_probe_train_loss": final_loss,
        "final_probe_gradient_norm": final_gradient_norm,
        "evaluation_loss": float(evaluation.loss.detach().cpu()),
        "probe_vector": _probe_vector(model, evaluation.logits),
        "artifact_cid": artifact_cid,
        "forbidden_reads": forbidden_reads,
    }
    del model, optimizer, evaluation, replay_model, replay_output
    gc.collect()
    if device.type == "mps":
        torch.mps.empty_cache()
    return result


def _probe_worker(root: str, arm: str, plan_value: Mapping[str, Any], queue: Any) -> None:
    try:
        plan = ExecutionPlan(
            name=str(plan_value["name"]),
            backend=str(plan_value["backend"]),  # type: ignore[arg-type]
            threads_per_worker=int(plan_value["threads_per_worker"]),
            workers=int(plan_value["workers"]),
            concurrent_arms=bool(plan_value["concurrent_arms"]),
        )
        queue.put({"ok": True, "result": _probe_arm(Path(root), arm, plan)})
    except BaseException as error:  # worker boundary must report every terminal
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


def _collect_process(process: Any, queue: Any, *, timeout: float = 900.0) -> dict[str, Any]:
    process.join(timeout)
    if process.is_alive():
        process.terminate()
        process.join(10.0)
        return {"ok": False, "error": {"type": "TimeoutError", "reason": "probe worker timed out"}}
    try:
        message = queue.get(timeout=5.0)
    except Empty:
        return {
            "ok": False,
            "error": {
                "type": "WorkerExitError",
                "reason": f"probe worker exited {process.exitcode} without evidence",
            },
        }
    if not isinstance(message, dict):
        raise RuntimeError("probe worker returned a non-object")
    return message


def _spawned_probe_executor(root: Path, plan: ExecutionPlan) -> dict[str, Any]:
    context = mp.get_context("spawn")
    plan_value = asdict(plan)
    outcomes: dict[str, Any] = {}
    if plan.concurrent_arms:
        active: dict[str, tuple[Any, Any]] = {}
        for arm in ARMS:
            queue = context.Queue()
            process = context.Process(
                target=_probe_worker,
                args=(str(root), arm, plan_value, queue),
                name=f"language-path-probe-{plan.name}-{arm}",
            )
            process.start()
            active[arm] = (process, queue)
        for arm, (process, queue) in active.items():
            outcomes[arm] = _collect_process(process, queue)
    else:
        for arm in ARMS:
            queue = context.Queue()
            process = context.Process(
                target=_probe_worker,
                args=(str(root), arm, plan_value, queue),
                name=f"language-path-probe-{plan.name}-{arm}",
            )
            process.start()
            outcomes[arm] = _collect_process(process, queue)
    return {"plan": plan.identity(), "arms": outcomes}


ProbeExecutor = Callable[[Path, ExecutionPlan], Mapping[str, Any]]


def _equivalence_against_reference(
    records: Sequence[Mapping[str, Any]], *, reference_name: str
) -> list[dict[str, Any]]:
    by_name = {str(record.get("plan", {}).get("name")): record for record in records}
    reference = by_name.get(reference_name)
    if reference is None:
        raise ValueError("CPU-four-thread reference probe is absent")
    reference_arms = reference.get("arms")
    if not isinstance(reference_arms, Mapping):
        raise ValueError("reference probe has no arm evidence")
    output: list[dict[str, Any]] = []
    for record in records:
        value = dict(record)
        equivalent = True
        deltas: dict[str, float | None] = {}
        for arm in ARMS:
            arm_record = value.get("arms", {}).get(arm, {})
            ref_record = reference_arms.get(arm, {})
            if not arm_record.get("ok") or not ref_record.get("ok"):
                equivalent = False
                deltas[arm] = None
                continue
            observed = arm_record["result"].get("probe_vector", [])
            expected = ref_record["result"].get("probe_vector", [])
            if len(observed) != len(expected) or not observed:
                equivalent = False
                deltas[arm] = None
                continue
            delta = max(abs(float(left) - float(right)) for left, right in zip(observed, expected))
            deltas[arm] = delta
            equivalent = equivalent and delta <= EQUIVALENCE_ABS_TOLERANCE
        value["equivalence"] = {
            "reference_plan": reference_name,
            "absolute_tolerance": EQUIVALENCE_ABS_TOLERANCE,
            "maximum_deltas": deltas,
            "passed": equivalent,
        }
        output.append(value)
    return output


def _project_probe(record: Mapping[str, Any]) -> dict[str, Any]:
    value = dict(record)
    plan = value.get("plan", {})
    arms = value.get("arms", {})
    available = all(bool(arms.get(arm, {}).get("ok")) for arm in ARMS)
    equivalent = bool(value.get("equivalence", {}).get("passed"))
    if not available:
        value["projection"] = {"eligible": False, "reason": "PROBE_UNAVAILABLE"}
        return value
    results = {arm: arms[arm]["result"] for arm in ARMS}
    validation_batches = math.ceil(VALIDATION_WINDOWS / BATCH_SIZE)
    per_arm_seconds = {
        arm: float(results[arm]["mean_train_step_seconds"]) * OPTIMIZER_STEPS
        + float(results[arm]["evaluation_batch_seconds"])
        * validation_batches
        * (3 if arm == "retained" else 2)
        + float(results[arm]["checkpoint_seconds"])
        * (1 + math.ceil(OPTIMIZER_STEPS / CHECKPOINT_INTERVAL))
        + float(results[arm]["progress_write_seconds"])
        * (
            7
            + math.ceil(OPTIMIZER_STEPS / PROGRESS_INTERVAL)
            + math.ceil(OPTIMIZER_STEPS / CHECKPOINT_INTERVAL)
        )
        + float(results[arm]["checkpoint_hash_seconds"])
        * (
            7
            + math.ceil(OPTIMIZER_STEPS / CHECKPOINT_INTERVAL)
            + math.ceil(OPTIMIZER_STEPS / PROGRESS_INTERVAL)
        )
        + float(results[arm]["artifact_export_seconds"])
        + float(results[arm]["fixed_prefix_replay_seconds"])
        for arm in ARMS
    }
    concurrent = bool(plan.get("concurrent_arms"))
    raw_seconds = max(per_arm_seconds.values()) if concurrent else sum(per_arm_seconds.values())
    projected_seconds = PROJECTION_SAFETY_FACTOR * raw_seconds
    peak_bytes = (
        sum(int(results[arm]["peak_memory_bytes"]) for arm in ARMS)
        if concurrent
        else max(int(results[arm]["peak_memory_bytes"]) for arm in ARMS)
    )
    memory_budget = min(int(results[arm]["memory_budget_bytes"]) for arm in ARMS)
    memory_fraction = peak_bytes / memory_budget if memory_budget > 0 else math.inf
    eligible = bool(
        equivalent
        and projected_seconds <= WALL_CEILING_SECONDS
        and memory_fraction <= MEMORY_FRACTION_CEILING
    )
    value["projection"] = {
        "eligible": eligible,
        "per_arm_seconds": per_arm_seconds,
        "raw_aggregate_seconds": raw_seconds,
        "safety_factor": PROJECTION_SAFETY_FACTOR,
        "projected_aggregate_seconds": projected_seconds,
        "wall_ceiling_seconds": WALL_CEILING_SECONDS,
        "peak_memory_bytes": peak_bytes,
        "memory_budget_bytes": memory_budget,
        "memory_fraction": memory_fraction,
        "memory_fraction_ceiling": MEMORY_FRACTION_CEILING,
        "checkpoint_interval_steps": CHECKPOINT_INTERVAL,
        "checkpoint_writes_per_arm": 1
        + math.ceil(OPTIMIZER_STEPS / CHECKPOINT_INTERVAL),
        "checkpoint_hashes_per_arm": 7
        + math.ceil(OPTIMIZER_STEPS / CHECKPOINT_INTERVAL)
        + math.ceil(OPTIMIZER_STEPS / PROGRESS_INTERVAL),
        "checkpoint_sidecar_writes_per_arm": 1
        + math.ceil(OPTIMIZER_STEPS / CHECKPOINT_INTERVAL),
        "progress_writes_per_arm": 6
        + math.ceil(OPTIMIZER_STEPS / PROGRESS_INTERVAL),
        "artifact_exports_per_arm": 1,
        "reason": (
            "ELIGIBLE"
            if eligible
            else "EQUIVALENCE"
            if not equivalent
            else "WALL"
            if projected_seconds > WALL_CEILING_SECONDS
            else "MEMORY"
        ),
    }
    return value


def select_execution_plan(records: Sequence[Mapping[str, Any]]) -> dict[str, Any]:
    """Apply equivalence, time, and memory gates and select measured-fastest."""

    if not records:
        raise ValueError("language-path execution selection requires probe records")
    compared = _equivalence_against_reference(
        records, reference_name="cpu-accelerate-4t-sequential"
    )
    projected = [_project_probe(record) for record in compared]
    eligible = [record for record in projected if record["projection"]["eligible"]]
    selected = min(
        eligible,
        key=lambda item: (
            float(item["projection"]["projected_aggregate_seconds"]),
            str(item["plan"]["name"]),
        ),
        default=None,
    )
    return {
        "plans": projected,
        "selected_plan": selected["plan"] if selected is not None else None,
        "selected_projection": selected["projection"] if selected is not None else None,
        "available": selected is not None,
    }


def _probe_language_path_execution(
    root: Path, *, executor: ProbeExecutor
) -> dict[str, Any]:
    root = root.resolve()
    existing = root / PROBE_RELATIVE_PATH
    if existing.exists():
        value = _load_json(existing)
        _verify_self_cid(value, "probe_cid")
        _require_current_implementation(
            value.get("implementation"), label="existing execution probe"
        )
        return value
    prepared = _prepared_inputs(root)
    preparation = _preparation_manifest(prepared)
    implementation = trainer_implementation_contract()
    raw_records = [dict(executor(root, plan)) for plan in ELIGIBLE_PLANS]
    _require_current_implementation(
        implementation,
        label="probe launch",
    )
    selection = select_execution_plan(raw_records)
    result = _with_cid(
        {
            "schema": PROBE_SCHEMA,
            "issue": ISSUE,
            "policy": POLICY,
            "preparation_manifest_cid": preparation.get("manifest_cid"),
            "implementation": implementation,
            "probe_contract": {
                "plans": [plan.identity() for plan in ELIGIBLE_PLANS],
                "warmup_steps_per_arm": PROBE_WARMUP_STEPS,
                "measured_steps_per_arm": PROBE_MEASURED_STEPS,
                "evaluation_batches_per_arm": 1,
                "projection_includes": [
                    "train batches",
                    "initial/final validation",
                    "retained state-off validation",
                    "checkpoint serialization and hashing",
                    "progress writes and checkpoint hashing",
                    "artifact export",
                    "fixed-prefix reload replay",
                ],
                "cuda": "FORBIDDEN",
                "safety_factor": PROJECTION_SAFETY_FACTOR,
                "wall_ceiling_seconds": WALL_CEILING_SECONDS,
                "memory_fraction_ceiling": MEMORY_FRACTION_CEILING,
            },
            "selection": selection,
            "verdict": "EXECUTION_PLAN_SELECTED" if selection["available"] else TERMINAL_UNAVAILABLE,
        },
        "probe_cid",
    )
    _write_exclusive_json(existing, result)
    return result


def probe_language_path_execution(root: Path) -> dict[str, Any]:
    """Measure every frozen plan in spawned processes and bind the fastest."""

    return _probe_language_path_execution(root, executor=_spawned_probe_executor)


def _metric_improvement(initial: Mapping[str, Any], final: Mapping[str, Any]) -> dict[str, float]:
    rows = int(final["rows"])
    if rows != VALIDATION_DECISIONS or int(initial["rows"]) != rows:
        raise ValueError("language-path validation row count differs")
    return {
        "nll_nats": float(initial["ce_nats"]) - float(final["ce_nats"]),
        "top1_points": 100.0
        * (int(final["top1_correct"]) - int(initial["top1_correct"]))
        / rows,
    }


def combine_language_path_gates(
    arms: Mapping[str, Mapping[str, Any]], *, mechanics_passed: bool
) -> dict[str, Any]:
    """Combine the public result thresholds into one divergent action."""

    if set(arms) != set(ARMS):
        raise ValueError("language-path gate requires retained and ordinary arms")
    if not mechanics_passed:
        return {
            "verdict": TERMINAL_INVALID_IMPLEMENTATION,
            "action": "repair the failed causal/replay/isolation mechanic; do not interpret model metrics",
            "retained_scientific_verdict": "NOT_EVALUATED",
        }
    ordinary = arms["ordinary"]
    retained = arms["retained"]
    ordinary_improvement = _metric_improvement(
        ordinary["initial_validation"], ordinary["final_validation"]
    )
    retained_improvement = _metric_improvement(
        retained["initial_validation"], retained["final_validation"]
    )
    ordinary_generalizes = bool(
        mechanics_passed
        and ordinary_improvement["nll_nats"] >= ORDINARY_REQUIRED_NLL_IMPROVEMENT
        and ordinary_improvement["top1_points"]
        >= ORDINARY_REQUIRED_TOP1_POINT_IMPROVEMENT
    )
    retained_generalizes = bool(
        mechanics_passed
        and retained_improvement["nll_nats"] >= RETAINED_REQUIRED_NLL_IMPROVEMENT
        and retained_improvement["top1_points"]
        >= RETAINED_REQUIRED_TOP1_POINT_IMPROVEMENT
    )
    state_off = retained["state_off_validation"]
    final_retained = retained["final_validation"]
    state_off_nll_delta = float(state_off["ce_nats"]) - float(final_retained["ce_nats"])
    state_off_top1_delta = int(final_retained["top1_correct"]) - int(
        state_off["top1_correct"]
    )
    retained_state_pass = bool(
        mechanics_passed
        and state_off_nll_delta >= REQUIRED_STATE_OFF_NLL_DELTA
        and state_off_top1_delta >= REQUIRED_STATE_OFF_TOP1_DECISIONS
    )
    ordinary_final = ordinary["final_validation"]
    competitive_nll_delta = float(final_retained["ce_nats"]) - float(
        ordinary_final["ce_nats"]
    )
    retained_top1_rate = int(final_retained["top1_correct"]) / VALIDATION_DECISIONS
    ordinary_top1_rate = int(ordinary_final["top1_correct"]) / VALIDATION_DECISIONS
    competitive_top1_point_delta = 100.0 * (retained_top1_rate - ordinary_top1_rate)
    competitive = bool(
        mechanics_passed
        and competitive_nll_delta <= COMPETITIVE_NLL_TOLERANCE
        and competitive_top1_point_delta >= -COMPETITIVE_TOP1_POINT_TOLERANCE
    )
    if not ordinary_generalizes:
        verdict = TERMINAL_INVALID_RECIPE
        action = "do not interpret retained; no parameter or optimizer sweep"
        retained_scientific_verdict = "NOT_EVALUATED"
    elif retained_generalizes and retained_state_pass and competitive:
        verdict = TERMINAL_PASS
        action = "preserve geometric checkpoint; run fixed 5x64 autonomous generation smoke"
        retained_scientific_verdict = "PASS"
    elif retained_generalizes and retained_state_pass:
        verdict = TERMINAL_NOT_COMPETITIVE
        action = "repair only decoder conditioning/readout before scale"
        retained_scientific_verdict = "GENERALIZES"
    else:
        verdict = TERMINAL_RETAINED_FAIL
        action = "retire this compact group-addressed language path"
        retained_scientific_verdict = "FAIL"
    return {
        "verdict": verdict,
        "action": action,
        "ordinary_generalizes": ordinary_generalizes,
        "retained_generalizes": retained_generalizes,
        "retained_state_pass": retained_state_pass,
        "competitive": competitive,
        "retained_scientific_verdict": retained_scientific_verdict,
        "ordinary_improvement": ordinary_improvement,
        "retained_improvement": retained_improvement,
        "state_off_nll_delta": state_off_nll_delta,
        "state_off_top1_delta": state_off_top1_delta,
        "competitive_nll_delta": competitive_nll_delta,
        "competitive_top1_point_delta": competitive_top1_point_delta,
        "thresholds": {
            "ordinary_nll_improvement": ORDINARY_REQUIRED_NLL_IMPROVEMENT,
            "ordinary_top1_point_improvement": ORDINARY_REQUIRED_TOP1_POINT_IMPROVEMENT,
            "retained_nll_improvement": RETAINED_REQUIRED_NLL_IMPROVEMENT,
            "retained_top1_point_improvement": RETAINED_REQUIRED_TOP1_POINT_IMPROVEMENT,
            "state_off_nll_delta": REQUIRED_STATE_OFF_NLL_DELTA,
            "state_off_top1_decisions": REQUIRED_STATE_OFF_TOP1_DECISIONS,
            "competitive_nll_tolerance": COMPETITIVE_NLL_TOLERANCE,
            "competitive_top1_point_tolerance": COMPETITIVE_TOP1_POINT_TOLERANCE,
        },
    }


def _aggregate_work(arm: str, batch_sizes: Sequence[int]) -> dict[str, int]:
    kind: Literal["retained", "ordinary"] = (
        "retained" if arm == "retained" else "ordinary"
    )
    totals = {
        "token_steps": 0,
        "materialized_attention_scores": 0,
        "admitted_attention_scores": 0,
        "attention_value_reads": 0,
        "vocabulary_scores": 0,
    }
    for size in batch_sizes:
        ledger = work_ledger(kind, batch_size=size, time=CONTEXT)
        for field in totals:
            totals[field] += int(getattr(ledger, field))
    return totals


@torch.no_grad()
def _evaluate(
    model: torch.nn.Module,
    windows: Any,
    device: torch.device,
    *,
    arm: str,
    attention_off: bool = False,
) -> dict[str, Any]:
    model.eval()
    loss_sum = 0.0
    correct = 0
    rows = 0
    forbidden_reads = 0
    batch_sizes: list[int] = []
    digest = blake3()
    for start in range(0, _window_count(windows), BATCH_SIZE):
        count = min(BATCH_SIZE, _window_count(windows) - start)
        batch = _window_batch(windows, start, count, device)
        inputs = batch[:, :-1]
        targets = batch[:, 1:]
        output = model(inputs, attention_off=attention_off)
        logits = output.logits.float()
        loss_sum += float(
            F.cross_entropy(
                logits.reshape(-1, logits.shape[-1]),
                targets.reshape(-1),
                reduction="sum",
            ).detach().cpu()
        )
        correct += int((logits.argmax(dim=-1) == targets).sum().detach().cpu())
        rows += int(targets.numel())
        batch_sizes.append(count)
        forbidden_reads += int(getattr(output.audit, "forbidden_reads", -1))
        digest.update(logits.detach().cpu().contiguous().numpy().tobytes())
    if rows != VALIDATION_DECISIONS or forbidden_reads != 0:
        raise RuntimeError("language-path validation rows or forbidden reads differ")
    return {
        "rows": rows,
        "ce_nats": loss_sum / rows,
        "top1_correct": correct,
        "top1_rate": correct / rows,
        "logits_cid": f"blake3:{digest.hexdigest()}",
        "forbidden_reads": forbidden_reads,
        "attention_off": attention_off,
        "work": _aggregate_work(arm, batch_sizes),
    }


def _common_initialization_identity(
    retained: torch.nn.Module, ordinary: torch.nn.Module
) -> dict[str, Any]:
    retained_parameters = dict(retained.named_parameters())
    ordinary_parameters = dict(ordinary.named_parameters())
    common = sorted(set(retained_parameters) & set(ordinary_parameters))
    if not common:
        raise RuntimeError("matched language-path arms have no shared parameters")
    unequal: list[str] = []
    digest = blake3()
    for name in common:
        left = retained_parameters[name].detach().cpu().contiguous()
        right = ordinary_parameters[name].detach().cpu().contiguous()
        if left.shape != right.shape or left.dtype != right.dtype or not torch.equal(left, right):
            unequal.append(name)
            continue
        digest.update(name.encode("utf-8"))
        digest.update(left.numpy().tobytes())
    if unequal:
        raise RuntimeError(f"shared learned initialization differs: {unequal}")
    retained_only = sorted(set(retained_parameters) - set(ordinary_parameters))
    ordinary_only = sorted(set(ordinary_parameters) - set(retained_parameters))
    expected_retained_only = [
        f"layers.{layer}.{name}"
        for layer in range(2)
        for name in ("decay_logits", "write_logits")
    ]
    expected_ordinary_only = [
        f"layers.{layer}.{name}"
        for layer in range(2)
        for name in ("log_output_gains", "log_score_gains")
    ]
    if retained_only != sorted(expected_retained_only) or ordinary_only != sorted(
        expected_ordinary_only
    ):
        raise RuntimeError("matched arm-only gain parameters differ from the freeze")
    return {
        "shared_parameter_tensors": len(common),
        "shared_parameters_byte_identical": True,
        "shared_initialization_cid": f"blake3:{digest.hexdigest()}",
        "retained_only": retained_only,
        "ordinary_only": ordinary_only,
    }


def _maximum_state_delta(left: Any, right: Any) -> float:
    deltas = [
        float((left.keys - right.keys).abs().max().detach().cpu()),
        float((left.values - right.values).abs().max().detach().cpu()),
        float((left.occupied != right.occupied).any().detach().cpu()),
    ]
    return max(deltas)


def _admission(root: Path) -> dict[str, Any]:
    prepared = _prepared_inputs(root)
    geometry = _exact_geometry(prepared)
    validation = _validation_windows(prepared)
    device = torch.device("cpu")
    models = {
        "retained": _build_model("retained", geometry).to(device),
        "ordinary": _build_model("ordinary", geometry).to(device),
    }
    for arm, model in models.items():
        ledger = architecture_ledger(arm)  # type: ignore[arg-type]
        if (
            model.parameter_count() != PARAMETER_COUNT
            or model.state_value_count() != STATE_VALUES
            or model.validity_bit_count() != VALIDITY_BITS
            or ledger.parameters != PARAMETER_COUNT
            or ledger.state_values != STATE_VALUES
            or ledger.state_bytes_f32 != STATE_BYTES_F32
            or ledger.validity_bits != VALIDITY_BITS
            or model.output_weight.data_ptr() != model.token_embedding.weight.data_ptr()
        ):
            raise RuntimeError(f"{arm} architecture ledger differs from the freeze")
    initialization = _common_initialization_identity(models["retained"], models["ordinary"])
    batch = _window_batch(validation, 0, 2, device)
    gradient_evidence: dict[str, Any] = {}
    causal_evidence: dict[str, Any] = {}
    replay_evidence: dict[str, Any] = {}
    for arm, model in models.items():
        model.zero_grad(set_to_none=True)
        output = model(batch[:, :-1], batch[:, 1:])
        if output.loss is None:
            raise RuntimeError(f"{arm} admission produced no loss")
        output.loss.backward()
        missing: list[str] = []
        for name, parameter in model.named_parameters():
            gradient = parameter.grad
            if (
                gradient is None
                or not bool(torch.isfinite(gradient).all())
                or not bool((gradient != 0).any())
            ):
                missing.append(name)
        if missing:
            raise RuntimeError(f"{arm} admission has inactive gradients: {missing}")
        gradient_evidence[arm] = {
            "parameters_with_finite_nonzero_gradient": len(list(model.parameters())),
            "passed": True,
        }

        model.zero_grad(set_to_none=True)
        original = batch[:1, :-1]
        altered = original.clone()
        altered[:, 61:] = (altered[:, 61:] + 1) % VOCAB_SIZE
        if torch.equal(original[:, 61:], altered[:, 61:]):
            raise RuntimeError(f"{arm} causality probe did not alter the future suffix")
        with torch.no_grad():
            first = model(original).logits[:, :61]
            second = model(altered).logits[:, :61]
        causal_delta = float((first - second).abs().max().detach().cpu())
        if causal_delta != 0.0:
            raise RuntimeError(f"{arm} future suffix changed shared-prefix logits")
        causal_evidence[arm] = {"shared_prefix_tokens": 61, "maximum_delta": causal_delta}

        artifact = model.export_learned_artifact()
        replay_model = _build_model(arm, geometry).to(device)
        replay_model.load_learned_artifact(artifact)
        with torch.no_grad():
            expected = model(original).logits
            observed = replay_model(original).logits
        replay_delta = float((expected - observed).abs().max().detach().cpu())
        if replay_delta != 0.0:
            raise RuntimeError(f"{arm} admission artifact replay differs")
        replay_evidence[arm] = {
            "artifact_cid": cid_bytes(artifact),
            "maximum_logits_delta": replay_delta,
        }
    retained = models["retained"]
    retained.zero_grad(set_to_none=True)
    with torch.no_grad():
        stationary = retained(batch[:1, :-1], implementation="stationary")
        direct = retained(batch[:1, :-1], implementation="direct")
    logits_delta = float(
        (stationary.logits - direct.logits).abs().max().detach().cpu()
    )
    state_delta = _maximum_state_delta(stationary.final_state, direct.final_state)
    if logits_delta > 2e-5 or state_delta > 2e-5:
        raise RuntimeError("retained stationary/direct parity differs")
    with torch.no_grad():
        forbidden_reads = sum(
            int(getattr(model(batch[:1, :-1]).audit, "forbidden_reads", -1))
            for model in models.values()
        )
    if forbidden_reads != 0:
        raise RuntimeError("language-path admission made a forbidden read")
    return {
        "passed": True,
        "architecture": {
            arm: asdict(architecture_ledger(arm)) for arm in ARMS  # type: ignore[arg-type]
        },
        "full_context_work": {
            arm: asdict(work_ledger(arm, batch_size=BATCH_SIZE, time=CONTEXT))  # type: ignore[arg-type]
            for arm in ARMS
        },
        "initialization": initialization,
        "gradients": gradient_evidence,
        "causality": causal_evidence,
        "artifact_replay": replay_evidence,
        "retained_full_incremental_parity": {
            "maximum_logits_delta": logits_delta,
            "maximum_state_delta": state_delta,
        },
        "population": {
            "train_windows": _window_count(_train_windows(prepared)),
            "validation_windows": _window_count(validation),
            "train_decisions": TRAIN_DECISIONS,
            "validation_decisions": VALIDATION_DECISIONS,
            "optimizer_steps_per_arm": OPTIMIZER_STEPS,
            "batch_size": BATCH_SIZE,
            "train_order": _train_order_identity(prepared),
        },
        "forbidden_reads": forbidden_reads,
    }


def _atomic_torch_save(path: Path, value: Mapping[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    temporary = path.with_name(f".{path.name}.tmp")
    torch.save(dict(value), temporary)
    with temporary.open("rb+") as target:
        os.fsync(target.fileno())
    os.replace(temporary, path)


def _checkpoint_path(root: Path, arm: str) -> Path:
    return root / "arms" / arm / "checkpoint.pt"


def _checkpoint_cid_path(path: Path) -> Path:
    return path.with_suffix(f"{path.suffix}.cid.json")


def _progress_path(root: Path, arm: str) -> Path:
    return root / "arms" / arm / "progress.json"


def _arm_result_path(root: Path, arm: str) -> Path:
    return root / "arms" / arm / "result.json"


def _artifact_path(root: Path, arm: str) -> Path:
    return root / "arms" / arm / "model.safetensors"


def _verify_arm_result_binding(
    result: Mapping[str, Any],
    *,
    arm: str,
    run_contract_cid: str,
    plan_cid: str,
) -> None:
    _verify_self_cid(result, "arm_result_cid")
    elapsed = result.get("elapsed_arm_seconds")
    if (
        result.get("schema") != ARM_RESULT_SCHEMA
        or result.get("arm") != arm
        or result.get("run_contract_cid") != run_contract_cid
        or result.get("plan_cid") != plan_cid
        or not isinstance(elapsed, (int, float))
        or isinstance(elapsed, bool)
        or not math.isfinite(float(elapsed))
        or float(elapsed) < 0.0
    ):
        raise ValueError(f"{arm} completed result differs from the current run")


def _optimizer_to_device(optimizer: torch.optim.Optimizer, device: torch.device) -> None:
    for state in optimizer.state.values():
        for name, value in tuple(state.items()):
            if isinstance(value, Tensor):
                state[name] = value.to(device)


def _save_checkpoint(
    path: Path,
    *,
    arm: str,
    model: torch.nn.Module,
    optimizer: torch.optim.Optimizer,
    step: int,
    elapsed_arm_seconds: float,
    initial_validation: Mapping[str, Any],
    run_contract_cid: str,
    plan_cid: str,
    last_loss: float | None,
) -> dict[str, Any]:
    checkpoint = {
        "schema": CHECKPOINT_SCHEMA,
        "issue": ISSUE,
        "policy": POLICY,
        "arm": arm,
        "step": step,
        "elapsed_arm_seconds": elapsed_arm_seconds,
        "initial_validation": dict(initial_validation),
        "run_contract_cid": run_contract_cid,
        "plan_cid": plan_cid,
        "last_loss": last_loss,
        "learning_rate": learning_rate(step),
        "model_state": model.state_dict(),
        "optimizer_state": optimizer.state_dict(),
        "cpu_rng_state": torch.get_rng_state(),
    }
    _atomic_torch_save(path, checkpoint)
    checkpoint_cid = cid_file(path)
    atomic_write_json(
        _checkpoint_cid_path(path),
        {
            "schema": "uor-r4.retained-language-path-checkpoint-cid/1",
            "arm": arm,
            "step": step,
            "bytes": path.stat().st_size,
            "checkpoint_cid": checkpoint_cid,
            "run_contract_cid": run_contract_cid,
            "plan_cid": plan_cid,
        },
    )
    return checkpoint


def _load_checkpoint(
    path: Path,
    *,
    arm: str,
    model: torch.nn.Module,
    optimizer: torch.optim.Optimizer,
    device: torch.device,
    run_contract_cid: str,
    plan_cid: str,
) -> dict[str, Any]:
    cid_path = _checkpoint_cid_path(path)
    if not cid_path.is_file():
        raise ValueError(f"{arm} checkpoint CID sidecar is absent")
    sidecar = _load_json(cid_path)
    if (
        sidecar.get("schema") != "uor-r4.retained-language-path-checkpoint-cid/1"
        or sidecar.get("arm") != arm
        or sidecar.get("bytes") != path.stat().st_size
        or sidecar.get("checkpoint_cid") != cid_file(path)
        or sidecar.get("run_contract_cid") != run_contract_cid
        or sidecar.get("plan_cid") != plan_cid
    ):
        raise ValueError(f"{arm} checkpoint CID does not reproduce")
    checkpoint = torch.load(path, map_location="cpu", weights_only=True)
    if (
        not isinstance(checkpoint, dict)
        or checkpoint.get("schema") != CHECKPOINT_SCHEMA
        or checkpoint.get("arm") != arm
        or checkpoint.get("run_contract_cid") != run_contract_cid
        or checkpoint.get("plan_cid") != plan_cid
    ):
        raise ValueError(f"{arm} checkpoint envelope differs")
    step = checkpoint.get("step")
    if isinstance(step, bool) or not isinstance(step, int) or not 0 <= step <= OPTIMIZER_STEPS:
        raise ValueError(f"{arm} checkpoint step differs")
    if sidecar.get("step") != step:
        raise ValueError(f"{arm} checkpoint CID sidecar step differs")
    model.load_state_dict(checkpoint["model_state"], strict=True)
    optimizer.load_state_dict(checkpoint["optimizer_state"])
    _optimizer_to_device(optimizer, device)
    rng = checkpoint.get("cpu_rng_state")
    if not isinstance(rng, Tensor):
        raise ValueError(f"{arm} checkpoint has no CPU RNG state")
    torch.set_rng_state(rng)
    expected_rate = learning_rate(step)
    if any(
        not math.isclose(float(group["lr"]), expected_rate, rel_tol=0.0, abs_tol=1e-15)
        for group in optimizer.param_groups
    ):
        raise ValueError(f"{arm} checkpoint learning rate differs")
    return checkpoint


def _write_progress(
    root: Path,
    *,
    arm: str,
    step: int,
    elapsed_arm_seconds: float,
    last_loss: float | None,
    checkpoint: Path,
    status: str,
) -> dict[str, Any]:
    sidecar = _load_json(_checkpoint_cid_path(checkpoint))
    if (
        sidecar.get("schema")
        != "uor-r4.retained-language-path-checkpoint-cid/1"
        or sidecar.get("arm") != arm
        or sidecar.get("bytes") != checkpoint.stat().st_size
        or sidecar.get("checkpoint_cid") != cid_file(checkpoint)
    ):
        raise ValueError(f"{arm} progress cannot bind an invalid checkpoint")
    rate = step / elapsed_arm_seconds if elapsed_arm_seconds > 0.0 else 0.0
    remaining = OPTIMIZER_STEPS - step
    progress = {
        "schema": "uor-r4.retained-language-path-progress/1",
        "issue": ISSUE,
        "policy": POLICY,
        "arm": arm,
        "status": status,
        "completed_steps": step,
        "total_steps": OPTIMIZER_STEPS,
        "completed_presentations": step * BATCH_SIZE * CONTEXT,
        "total_presentations": TRAIN_DECISIONS,
        "elapsed_arm_seconds": elapsed_arm_seconds,
        "steps_per_second": rate,
        "eta_seconds": remaining / rate if rate > 0.0 else None,
        "last_loss": last_loss,
        "learning_rate": learning_rate(step),
        "run_contract_cid": sidecar["run_contract_cid"],
        "plan_cid": sidecar["plan_cid"],
        "checkpoint": {
            "path": str(checkpoint.relative_to(root)),
            "completed_step": int(sidecar["step"]),
            "bytes": int(sidecar["bytes"]),
            "cid": sidecar["checkpoint_cid"],
        },
        "resume": "run-language-path --resume",
    }
    atomic_write_json(_progress_path(root, arm), progress)
    return progress


def _resume_elapsed_baseline(
    root: Path,
    *,
    arm: str,
    checkpoint: Mapping[str, Any],
    checkpoint_path: Path,
    run_contract_cid: str,
    plan_cid: str,
) -> float:
    """Recover cumulative wall usage without pretending replayed work was free."""

    checkpoint_step = int(checkpoint["step"])
    checkpoint_elapsed = float(checkpoint["elapsed_arm_seconds"])
    progress_path = _progress_path(root, arm)
    if not progress_path.exists():
        return checkpoint_elapsed
    progress = _load_json(progress_path)
    progress_step = progress.get("completed_steps")
    progress_elapsed = progress.get("elapsed_arm_seconds")
    progress_checkpoint = progress.get("checkpoint")
    if (
        progress.get("schema") != "uor-r4.retained-language-path-progress/1"
        or progress.get("issue") != ISSUE
        or progress.get("policy") != POLICY
        or progress.get("arm") != arm
        or progress.get("run_contract_cid") != run_contract_cid
        or progress.get("plan_cid") != plan_cid
        or isinstance(progress_step, bool)
        or not isinstance(progress_step, int)
        or not 0 <= progress_step <= OPTIMIZER_STEPS
        or progress.get("total_steps") != OPTIMIZER_STEPS
        or progress.get("completed_presentations")
        != progress_step * BATCH_SIZE * CONTEXT
        or progress.get("total_presentations") != TRAIN_DECISIONS
        or not isinstance(progress_elapsed, (int, float))
        or isinstance(progress_elapsed, bool)
        or not math.isfinite(float(progress_elapsed))
        or float(progress_elapsed) < 0.0
        or not isinstance(progress_checkpoint, Mapping)
    ):
        raise ValueError(f"{arm} durable progress differs from the resume contract")
    progress_elapsed_float = float(progress_elapsed)
    if progress_step <= checkpoint_step and progress_elapsed_float <= checkpoint_elapsed:
        return checkpoint_elapsed
    if progress_step < checkpoint_step or progress_elapsed_float < checkpoint_elapsed:
        raise ValueError(f"{arm} progress/checkpoint ordering is contradictory")
    if (
        progress_checkpoint.get("path") != str(checkpoint_path.relative_to(root))
        or progress_checkpoint.get("completed_step") != checkpoint_step
        or progress_checkpoint.get("bytes") != checkpoint_path.stat().st_size
        or progress_checkpoint.get("cid") != cid_file(checkpoint_path)
    ):
        raise ValueError(f"{arm} progress does not bind the resumed checkpoint")
    return progress_elapsed_float


def _fixed_prefix_replay(
    model: torch.nn.Module,
    *,
    arm: str,
    geometry: Any,
    validation: Any,
    artifact: bytes,
    device: torch.device,
) -> dict[str, Any]:
    raw = validation.windows if hasattr(validation, "windows") else validation
    prefix = torch.tensor(raw[0:1, :65], dtype=torch.long, device=device)
    model.eval()
    with torch.no_grad():
        expected = model(prefix[:, :-1]).logits
    replay = _build_model(arm, geometry).to(device)
    replay.load_learned_artifact(artifact)
    replay.eval()
    with torch.no_grad():
        observed = replay(prefix[:, :-1]).logits
    reload_delta = float((expected - observed).abs().max().detach().cpu())
    incremental_delta: float | None = None
    if arm == "retained":
        with torch.no_grad():
            direct = replay(prefix[:, :-1], implementation="direct").logits
        incremental_delta = float((observed - direct).abs().max().detach().cpu())
    passed = reload_delta == 0.0 and (
        incremental_delta is None or incremental_delta <= 2e-5
    )
    return {
        "prefix_tokens": int(prefix.shape[1] - 1),
        "artifact_reload_maximum_logits_delta": reload_delta,
        "retained_direct_maximum_logits_delta": incremental_delta,
        "passed": passed,
    }


def _run_arm(
    root: Path,
    arm: str,
    plan: ExecutionPlan,
    *,
    run_contract_cid: str,
    resume: bool,
    remaining_wall_seconds: float,
) -> dict[str, Any]:
    device, backend = _configure_device(plan)
    prepared = _prepared_inputs(root)
    geometry = _exact_geometry(prepared)
    validation = _validation_windows(prepared)
    train_order = _train_order_identity(prepared)
    plan_cid = plan.identity()["plan_cid"]
    model = _build_model(arm, geometry).to(device)
    optimizer = _optimizer(model)
    checkpoint_path = _checkpoint_path(root, arm)
    result_path = _arm_result_path(root, arm)
    if result_path.exists():
        result = _load_json(result_path)
        _verify_arm_result_binding(
            result,
            arm=arm,
            run_contract_cid=run_contract_cid,
            plan_cid=plan_cid,
        )
        return result
    if checkpoint_path.exists() and not resume:
        raise FileExistsError(f"{arm} checkpoint exists; resume is required")
    if resume and not checkpoint_path.exists() and _progress_path(root, arm).exists():
        raise ValueError(f"{arm} progress exists without its resume checkpoint")
    started = time.monotonic()
    step = 0
    elapsed_before = 0.0
    last_loss: float | None = None
    if resume and checkpoint_path.exists():
        checkpoint = _load_checkpoint(
            checkpoint_path,
            arm=arm,
            model=model,
            optimizer=optimizer,
            device=device,
            run_contract_cid=run_contract_cid,
            plan_cid=plan_cid,
        )
        step = int(checkpoint["step"])
        elapsed_before = _resume_elapsed_baseline(
            root,
            arm=arm,
            checkpoint=checkpoint,
            checkpoint_path=checkpoint_path,
            run_contract_cid=run_contract_cid,
            plan_cid=plan_cid,
        )
        initial_validation = dict(checkpoint["initial_validation"])
        last_loss = checkpoint.get("last_loss")
    else:
        initial_validation = _evaluate(
            model, validation, device, arm=arm, attention_off=False
        )
        elapsed = time.monotonic() - started
        _save_checkpoint(
            checkpoint_path,
            arm=arm,
            model=model,
            optimizer=optimizer,
            step=0,
            elapsed_arm_seconds=elapsed,
            initial_validation=initial_validation,
            run_contract_cid=run_contract_cid,
            plan_cid=plan_cid,
            last_loss=None,
        )
        _write_progress(
            root,
            arm=arm,
            step=0,
            elapsed_arm_seconds=elapsed,
            last_loss=None,
            checkpoint=checkpoint_path,
            status="RUNNING",
        )

    elapsed_current_before_training = time.monotonic() - started
    elapsed_before_training = elapsed_before + elapsed_current_before_training
    if _wall_exhausted(
        elapsed_before_seconds=elapsed_before,
        elapsed_current_seconds=elapsed_current_before_training,
        arm_ceiling_seconds=remaining_wall_seconds,
    ):
        _save_checkpoint(
            checkpoint_path,
            arm=arm,
            model=model,
            optimizer=optimizer,
            step=step,
            elapsed_arm_seconds=elapsed_before_training,
            initial_validation=initial_validation,
            run_contract_cid=run_contract_cid,
            plan_cid=plan_cid,
            last_loss=last_loss,
        )
        progress = _write_progress(
            root,
            arm=arm,
            step=step,
            elapsed_arm_seconds=elapsed_before_training,
            last_loss=last_loss,
            checkpoint=checkpoint_path,
            status=TERMINAL_UNAVAILABLE,
        )
        return {
            "schema": ARM_RESULT_SCHEMA,
            "arm": arm,
            "status": TERMINAL_UNAVAILABLE,
            "stage": "initial_validation_or_resume",
            "progress": progress,
            "backend": backend,
        }
    for next_step in range(step + 1, OPTIMIZER_STEPS + 1):
        batch = _ordered_train_batch(prepared, next_step, device)
        last_loss, _ = _train_step(model, optimizer, batch, step=next_step)
        step = next_step
        elapsed_current = time.monotonic() - started
        elapsed = elapsed_before + elapsed_current
        should_checkpoint = step % CHECKPOINT_INTERVAL == 0 or step == OPTIMIZER_STEPS
        should_report = step % PROGRESS_INTERVAL == 0 or step == OPTIMIZER_STEPS
        if should_checkpoint:
            _save_checkpoint(
                checkpoint_path,
                arm=arm,
                model=model,
                optimizer=optimizer,
                step=step,
                elapsed_arm_seconds=elapsed,
                initial_validation=initial_validation,
                run_contract_cid=run_contract_cid,
                plan_cid=plan_cid,
                last_loss=last_loss,
            )
        if should_report:
            report_elapsed = elapsed_before + (time.monotonic() - started)
            progress = _write_progress(
                root,
                arm=arm,
                step=step,
                elapsed_arm_seconds=report_elapsed,
                last_loss=last_loss,
                checkpoint=checkpoint_path,
                status="RUNNING",
            )
            print(
                f"language_path arm={arm} step={step}/{OPTIMIZER_STEPS} "
                f"loss={last_loss:.6f} lr={learning_rate(step):.8f} "
                f"eta={progress['eta_seconds']}",
                flush=True,
            )
        elapsed_current_after_io = time.monotonic() - started
        elapsed_after_io = elapsed_before + elapsed_current_after_io
        if _wall_exhausted(
            elapsed_before_seconds=elapsed_before,
            elapsed_current_seconds=elapsed_current_after_io,
            arm_ceiling_seconds=remaining_wall_seconds,
        ):
            if not should_checkpoint:
                _save_checkpoint(
                    checkpoint_path,
                    arm=arm,
                    model=model,
                    optimizer=optimizer,
                    step=step,
                    elapsed_arm_seconds=elapsed_after_io,
                    initial_validation=initial_validation,
                    run_contract_cid=run_contract_cid,
                    plan_cid=plan_cid,
                    last_loss=last_loss,
                )
            progress = _write_progress(
                root,
                arm=arm,
                step=step,
                elapsed_arm_seconds=elapsed_after_io,
                last_loss=last_loss,
                checkpoint=checkpoint_path,
                status=TERMINAL_UNAVAILABLE,
            )
            return {
                "schema": ARM_RESULT_SCHEMA,
                "arm": arm,
                "status": TERMINAL_UNAVAILABLE,
                "progress": progress,
                "backend": backend,
            }

    def final_wall_terminal(stage: str) -> dict[str, Any] | None:
        elapsed_current_final = time.monotonic() - started
        elapsed_final = elapsed_before + elapsed_current_final
        _write_progress(
            root,
            arm=arm,
            step=OPTIMIZER_STEPS,
            elapsed_arm_seconds=elapsed_final,
            last_loss=last_loss,
            checkpoint=checkpoint_path,
            status=f"{stage.upper()}_COMPLETE",
        )
        elapsed_current_final = time.monotonic() - started
        elapsed_final = elapsed_before + elapsed_current_final
        if not _wall_exhausted(
            elapsed_before_seconds=elapsed_before,
            elapsed_current_seconds=elapsed_current_final,
            arm_ceiling_seconds=remaining_wall_seconds,
        ):
            return None
        progress_final = _write_progress(
            root,
            arm=arm,
            step=OPTIMIZER_STEPS,
            elapsed_arm_seconds=elapsed_final,
            last_loss=last_loss,
            checkpoint=checkpoint_path,
            status=TERMINAL_UNAVAILABLE,
        )
        return {
            "schema": ARM_RESULT_SCHEMA,
            "arm": arm,
            "status": TERMINAL_UNAVAILABLE,
            "stage": stage,
            "progress": progress_final,
            "backend": backend,
        }

    final_validation = _evaluate(model, validation, device, arm=arm)
    wall_terminal = final_wall_terminal("final_validation")
    if wall_terminal is not None:
        return wall_terminal
    state_off_validation = (
        _evaluate(model, validation, device, arm=arm, attention_off=True)
        if arm == "retained"
        else None
    )
    wall_terminal = final_wall_terminal("state_off_validation")
    if wall_terminal is not None:
        return wall_terminal
    artifact = model.export_learned_artifact()
    artifact_path = _artifact_path(root, arm)
    atomic_write(artifact_path, artifact)
    artifact_record = {
        "path": str(artifact_path.relative_to(root)),
        "bytes": artifact_path.stat().st_size,
        "cid": cid_file(artifact_path),
    }
    wall_terminal = final_wall_terminal("artifact_export")
    if wall_terminal is not None:
        return wall_terminal
    replay = _fixed_prefix_replay(
        model,
        arm=arm,
        geometry=geometry,
        validation=validation,
        artifact=artifact,
        device=device,
    )
    wall_terminal = final_wall_terminal("artifact_replay")
    if wall_terminal is not None:
        return wall_terminal
    elapsed = elapsed_before + (time.monotonic() - started)
    _write_progress(
        root,
        arm=arm,
        step=OPTIMIZER_STEPS,
        elapsed_arm_seconds=elapsed,
        last_loss=last_loss,
        checkpoint=checkpoint_path,
        status="COMPLETE",
    )
    elapsed_current_complete = time.monotonic() - started
    elapsed = elapsed_before + elapsed_current_complete
    if _wall_exhausted(
        elapsed_before_seconds=elapsed_before,
        elapsed_current_seconds=elapsed_current_complete,
        arm_ceiling_seconds=remaining_wall_seconds,
    ):
        progress = _write_progress(
            root,
            arm=arm,
            step=OPTIMIZER_STEPS,
            elapsed_arm_seconds=elapsed,
            last_loss=last_loss,
            checkpoint=checkpoint_path,
            status=TERMINAL_UNAVAILABLE,
        )
        return {
            "schema": ARM_RESULT_SCHEMA,
            "arm": arm,
            "status": TERMINAL_UNAVAILABLE,
            "stage": "result_envelope",
            "progress": progress,
            "backend": backend,
        }
    body = {
        "schema": ARM_RESULT_SCHEMA,
        "issue": ISSUE,
        "policy": POLICY,
        "arm": arm,
        "status": "COMPLETE",
        "run_contract_cid": run_contract_cid,
        "plan_cid": plan_cid,
        "backend": backend,
        "completed_steps": OPTIMIZER_STEPS,
        "presentations": TRAIN_DECISIONS,
        "train_order_cid": train_order["order_cid"],
        "elapsed_arm_seconds": elapsed,
        "initial_validation": initial_validation,
        "final_validation": final_validation,
        "state_off_validation": state_off_validation,
        "artifact": artifact_record,
        "replay": replay,
        "forbidden_reads": int(final_validation["forbidden_reads"])
        + int(state_off_validation["forbidden_reads"] if state_off_validation else 0),
    }
    result = _with_cid(body, "arm_result_cid")
    _write_exclusive_json(result_path, result)
    return result


def _arm_worker(
    root: str,
    arm: str,
    plan_value: Mapping[str, Any],
    run_contract_cid: str,
    resume: bool,
    remaining_wall_seconds: float,
    queue: Any,
) -> None:
    try:
        plan = ExecutionPlan(
            name=str(plan_value["name"]),
            backend=str(plan_value["backend"]),  # type: ignore[arg-type]
            threads_per_worker=int(plan_value["threads_per_worker"]),
            workers=int(plan_value["workers"]),
            concurrent_arms=bool(plan_value["concurrent_arms"]),
        )
        result = _run_arm(
            Path(root),
            arm,
            plan,
            run_contract_cid=run_contract_cid,
            resume=resume,
            remaining_wall_seconds=remaining_wall_seconds,
        )
        queue.put({"ok": True, "result": result})
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


def _prior_arm_elapsed(root: Path, arm: str) -> float:
    path = _progress_path(root, arm)
    if not path.exists():
        return 0.0
    return float(_load_json(path).get("elapsed_arm_seconds", 0.0))


def _spawned_arm_runner(
    root: Path,
    plan: ExecutionPlan,
    *,
    run_contract_cid: str,
    resume: bool,
) -> dict[str, Any]:
    context = mp.get_context("spawn")
    plan_value = asdict(plan)
    outcomes: dict[str, Any] = {}
    already_complete: dict[str, Any] = {}
    for arm in ARMS:
        result_path = _arm_result_path(root, arm)
        if result_path.exists():
            result = _load_json(result_path)
            _verify_arm_result_binding(
                result,
                arm=arm,
                run_contract_cid=run_contract_cid,
                plan_cid=plan.identity()["plan_cid"],
            )
            already_complete[arm] = result
    if plan.concurrent_arms:
        arm_wall_ceiling = _arm_wall_ceiling(
            concurrent=True, completed_other_arm_seconds=0.0
        )
        active: dict[str, tuple[Any, Any]] = {}
        for arm in ARMS:
            if arm in already_complete:
                outcomes[arm] = {"ok": True, "result": already_complete[arm]}
                continue
            queue = context.Queue()
            process = context.Process(
                target=_arm_worker,
                args=(
                    str(root),
                    arm,
                    plan_value,
                    run_contract_cid,
                    resume,
                    arm_wall_ceiling,
                    queue,
                ),
                name=f"language-path-{arm}",
            )
            process.start()
            active[arm] = (process, queue)
        for arm, (process, queue) in active.items():
            outcomes[arm] = _collect_process(process, queue, timeout=WALL_CEILING_SECONDS + 60.0)
    else:
        consumed = sum(float(value.get("elapsed_arm_seconds", 0.0)) for value in already_complete.values())
        for arm in ARMS:
            if arm in already_complete:
                outcomes[arm] = {"ok": True, "result": already_complete[arm]}
                continue
            arm_wall_ceiling = _arm_wall_ceiling(
                concurrent=False, completed_other_arm_seconds=consumed
            )
            queue = context.Queue()
            process = context.Process(
                target=_arm_worker,
                args=(
                    str(root),
                    arm,
                    plan_value,
                    run_contract_cid,
                    resume,
                    arm_wall_ceiling,
                    queue,
                ),
                name=f"language-path-{arm}",
            )
            process.start()
            remaining_process_time = max(
                0.0, arm_wall_ceiling - _prior_arm_elapsed(root, arm)
            )
            outcomes[arm] = _collect_process(
                process, queue, timeout=remaining_process_time + 60.0
            )
            if not outcomes[arm].get("ok"):
                break
            outcome_result = outcomes[arm]["result"]
            consumed += float(
                outcome_result.get(
                    "elapsed_arm_seconds",
                    outcome_result.get("progress", {}).get(
                        "elapsed_arm_seconds", 0.0
                    ),
                )
            )
            if outcome_result.get("status") == TERMINAL_UNAVAILABLE:
                break
    return outcomes


ArmRunner = Callable[..., Mapping[str, Any]]


def _run_language_path_generalization(
    root: Path,
    *,
    resume: bool,
    arm_runner: ArmRunner,
) -> dict[str, Any]:
    root = root.resolve()
    result_path = root / RESULT_RELATIVE_PATH
    if result_path.exists():
        value = _load_json(result_path)
        _verify_self_cid(value, "result_cid")
        return value
    probe = _load_json(root / PROBE_RELATIVE_PATH)
    _verify_self_cid(probe, "probe_cid")
    current_implementation = _require_current_implementation(
        probe.get("implementation"), label="execution probe"
    )
    selected = probe.get("selection", {}).get("selected_plan")
    if not isinstance(selected, Mapping):
        body = _with_cid(
            {
                "schema": RESULT_SCHEMA,
                "issue": ISSUE,
                "policy": POLICY,
                "probe_cid": probe["probe_cid"],
                "verdict": TERMINAL_UNAVAILABLE,
                "reason": "execution probe selected no eligible plan",
                "arms": {},
                "h4_specific": "NOT_EVALUATED",
            },
            "result_cid",
        )
        _write_exclusive_json(result_path, body)
        return body
    plan = ExecutionPlan(
        name=str(selected["name"]),
        backend=str(selected["backend"]),  # type: ignore[arg-type]
        threads_per_worker=int(selected["threads_per_worker"]),
        workers=int(selected["workers"]),
        concurrent_arms=bool(selected["concurrent_arms"]),
    )
    prepared = _prepared_inputs(root)
    preparation = _preparation_manifest(prepared)
    preparation_manifest_cid = preparation.get("manifest_cid")
    if probe.get("preparation_manifest_cid") != preparation_manifest_cid:
        raise ValueError("language-path preparation differs from the execution probe")
    train_order = _train_order_identity(prepared)
    started_path = root / STARTED_RELATIVE_PATH
    if resume:
        if not started_path.exists():
            raise FileNotFoundError("language-path resume requested without started envelope")
        started = _load_json(started_path)
        _verify_self_cid(started, "started_cid")
        _require_current_implementation(
            started.get("implementation"),
            label="started envelope",
            current=current_implementation,
        )
        admission = started["admission"]
        run_contract = started["run_contract"]
        run_contract_cid = str(started["run_contract_cid"])
        if cid_bytes(canonical_json_bytes(run_contract)) != run_contract_cid:
            raise ValueError("language-path resume run contract does not reproduce")
        if run_contract.get("plan") != plan.identity():
            raise ValueError("language-path resume selected plan differs")
        if run_contract.get("implementation") != current_implementation:
            raise ValueError("language-path resume implementation contract differs")
        if (
            started.get("preparation_manifest_cid") != preparation_manifest_cid
            or run_contract.get("preparation_manifest_cid")
            != preparation_manifest_cid
        ):
            raise ValueError("language-path resume preparation manifest differs")
        if run_contract.get("population", {}).get("train_order") != train_order:
            raise ValueError("language-path resume train order differs")
    else:
        if started_path.exists():
            raise FileExistsError("language-path run already started; use resume")
        if any(_checkpoint_path(root, arm).exists() for arm in ARMS):
            raise FileExistsError("language-path checkpoints exist before started envelope")
        admission = _admission(root)
        run_contract = {
            "policy": POLICY,
            "preparation_manifest_cid": preparation_manifest_cid,
            "probe_cid": probe["probe_cid"],
            "plan": plan.identity(),
            "implementation": current_implementation,
            "model": {
                "parameters_per_arm": PARAMETER_COUNT,
                "state_values_per_arm": STATE_VALUES,
                "state_bytes_f32_per_arm": STATE_BYTES_F32,
                "validity_bits_per_arm": VALIDITY_BITS,
            },
            "population": {
                "train_windows": TRAIN_WINDOWS,
                "validation_windows": VALIDATION_WINDOWS,
                "context": CONTEXT,
                "train_decisions_per_arm": TRAIN_DECISIONS,
                "validation_decisions": VALIDATION_DECISIONS,
                "reachable_validation_decisions": REACHABLE_VALIDATION_DECISIONS,
                "forbidden_reads": 0,
                "train_decisions_per_parameter": TRAIN_DECISIONS / PARAMETER_COUNT,
                "train_order": train_order,
            },
            "optimizer": {
                "name": "AdamW",
                "steps_per_arm": OPTIMIZER_STEPS,
                "batch_size": BATCH_SIZE,
                "one_epoch_without_replacement": True,
                "betas": [ADAM_BETA1, ADAM_BETA2],
                "epsilon": ADAM_EPSILON,
                "weight_decay": WEIGHT_DECAY,
                "gradient_clip": GRADIENT_CLIP,
                "warmup_steps": WARMUP_STEPS,
                "maximum_learning_rate": MAXIMUM_LEARNING_RATE,
                "minimum_learning_rate": MINIMUM_LEARNING_RATE,
                "schedule": "linear warmup then cosine decay",
                "seed": INITIALIZATION_SEED,
            },
            "evaluation": "initial and final validation only; retained final state-off",
            "retry": "same-trajectory checkpoint resume only",
            "wall_ceiling_seconds": WALL_CEILING_SECONDS,
            "cuda": "FORBIDDEN",
        }
        run_contract_cid = cid_bytes(canonical_json_bytes(run_contract))
        started = _with_cid(
            {
                "schema": STARTED_SCHEMA,
                "issue": ISSUE,
                "policy": POLICY,
                "preparation_manifest_cid": preparation_manifest_cid,
                "probe_cid": probe["probe_cid"],
                "implementation": current_implementation,
                "admission": admission,
                "run_contract": run_contract,
                "run_contract_cid": run_contract_cid,
                "model_heldout": {"status": "NOT_RUN", "reads": 0},
            },
            "started_cid",
        )
        _write_exclusive_json(started_path, started)
    outcomes = dict(
        arm_runner(
            root,
            plan,
            run_contract_cid=run_contract_cid,
            resume=resume,
        )
    )
    failures = {
        arm: outcome.get("error")
        for arm, outcome in outcomes.items()
        if not outcome.get("ok")
    }
    arm_results = {
        arm: outcome["result"]
        for arm, outcome in outcomes.items()
        if outcome.get("ok") and isinstance(outcome.get("result"), Mapping)
    }
    hard_wall_consumed = any(
        result.get("status") == TERMINAL_UNAVAILABLE for result in arm_results.values()
    )
    if failures and not hard_wall_consumed:
        atomic_write_json(
            root / "run" / "language-path-last-worker-error.json",
            {
                "schema": "uor-r4.retained-language-path-worker-error/1",
                "issue": ISSUE,
                "policy": POLICY,
                "run_contract_cid": run_contract_cid,
                "failures": failures,
                "resume": "run-language-path --resume",
            },
        )
        raise RuntimeError(
            "language-path worker interrupted; the same trajectory remains resumable"
        )
    unavailable = bool(
        hard_wall_consumed
        or set(arm_results) != set(ARMS)
        or any(result.get("status") != "COMPLETE" for result in arm_results.values())
    )
    if set(arm_results) != set(ARMS) and not hard_wall_consumed:
        raise RuntimeError(
            "language-path worker returned incomplete evidence; the same trajectory remains resumable"
        )
    if unavailable:
        decision = {
            "verdict": TERMINAL_UNAVAILABLE,
            "action": "stop; the frozen compute contract is unavailable and model metrics are not interpreted",
        }
    else:
        mechanics_passed = bool(
            admission.get("passed") is True
            and all(result.get("forbidden_reads") == 0 for result in arm_results.values())
            and all(result.get("replay", {}).get("passed") is True for result in arm_results.values())
            and all(result.get("completed_steps") == OPTIMIZER_STEPS for result in arm_results.values())
            and all(result.get("presentations") == TRAIN_DECISIONS for result in arm_results.values())
            and all(
                result.get("train_order_cid") == train_order["order_cid"]
                for result in arm_results.values()
            )
        )
        if not mechanics_passed:
            decision = {
                "verdict": TERMINAL_INVALID_IMPLEMENTATION,
                "action": "repair the failed causal/replay/isolation mechanic; do not interpret model metrics",
                "retained_scientific_verdict": "NOT_EVALUATED",
            }
        else:
            decision = combine_language_path_gates(
                {
                    arm: {
                        "initial_validation": arm_results[arm]["initial_validation"],
                        "final_validation": arm_results[arm]["final_validation"],
                        "state_off_validation": arm_results[arm].get("state_off_validation"),
                    }
                    for arm in ARMS
                },
                mechanics_passed=True,
            )
    body = {
        "schema": RESULT_SCHEMA,
        "issue": ISSUE,
        "policy": POLICY,
        "started_cid": started["started_cid"],
        "run_contract_cid": run_contract_cid,
        "probe_cid": probe["probe_cid"],
        "preparation_manifest_cid": preparation_manifest_cid,
        "selected_plan": plan.identity(),
        "implementation": current_implementation,
        "train_order": train_order,
        "mechanics": admission,
        "arms": arm_results,
        "failures": failures,
        "decision": decision,
        "verdict": decision["verdict"],
        "h4_specific": "NOT_EVALUATED",
        "forbidden_reads": 0
        if set(arm_results) == set(ARMS)
        and all(result.get("forbidden_reads") == 0 for result in arm_results.values())
        else "NONZERO_OR_UNAVAILABLE",
        "model_heldout": {"status": "NOT_RUN", "reads": 0},
        "generation": "NOT_RUN",
        "reasoning": "NOT_RUN",
        "lowering": "NOT_RUN",
    }
    result = _with_cid(body, "result_cid")
    _write_exclusive_json(result_path, result)
    return result


def run_language_path_generalization(
    root: Path, resume: bool = False
) -> dict[str, Any]:
    """Run or resume the one selected two-arm language-path experiment."""

    return _run_language_path_generalization(
        root,
        resume=resume,
        arm_runner=_spawned_arm_runner,
    )
