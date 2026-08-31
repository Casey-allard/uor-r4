"""Frozen training and qualification lifecycle for the #1019 capacity rung."""

from __future__ import annotations

import importlib.metadata
import json
import math
import os
import platform
import struct
import sys
import time
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Any, Literal

import numpy as np
import torch
from blake3 import blake3
from safetensors.torch import load_file
from tokenizers import Tokenizer
from torch import Tensor

from .capacity_data import (
    CAPACITY_DATASET_MANIFEST_NAME,
    CAPACITY_TRAINING_VIEW_MANIFEST_NAME,
    DEV_TOKEN_CAP,
    INDEX_RELATIVE_PATHS,
    SEALED_DIRECTORY_RELATIVE_PATH,
    SEALED_PROMPT_RELATIVE_PATH,
    TEST_TOKEN_CAP,
    TOKENIZER_CID,
    TOKENIZER_RELATIVE_PATH,
    TOKEN_RELATIVE_PATHS,
    TRAIN_TOKEN_CAP,
    _validate_dataset_envelope,
    _validate_training_view_envelope,
    load_capacity_dataset_manifest,
    load_capacity_training_view_manifest,
    open_sealed_confirmation,
)
from .constants import (
    CAPACITY_MODEL_CONFIG,
    EXPORT_MANIFEST_SCHEMA,
    FROZEN_MODEL_CONFIG,
    LLAMA2_C_REPOSITORY,
    LLAMA2_C_REVISION,
    SEALED_PROMPT_COUNT,
    SEALED_PROMPT_TOKEN_COUNT,
    SEALED_PROMPT_TOKENS_PER_STORY,
)
from .export import export_hugging_face_snapshot
from .finalize import R4_POLICY_IDENTITY
from .model import R4SoftmaxForCausalLM, expected_parameter_count
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
from .train import TokenStore, evaluate


ISSUE = 1019
PARAMETER_COUNT = 13_130_784
OPTIMIZER_STEPS = 16_800
TRAIN_TOKENS = 275_251_200
TOKENS_PER_OPTIMIZER_STEP = 16_384
MAXIMUM_BEST_CHECKPOINT_SAVES = 1 + OPTIMIZER_STEPS // 400
SEALED_TEST_LOSS_CEILING = 1.50
PREFIX_PARITY_TOKENS = 32
PREFIX_LOGIT_ABS_TOLERANCE = 0.005
BASELINE_1017_WEIGHTS_CID = (
    "blake3:c5bf31aa97a567b3aaad4461ce2fac9cebc12b0a38becb6d02d21b43b493bf5d"
)
BASELINE_1017_NLL = 1.5727521962806827

CAPACITY_RUN_SCHEMA = "uor-r4-softmax-trainer-capacity-run/1"
CAPACITY_CHECKPOINT_SCHEMA = "uor-r4-softmax-trainer-capacity-checkpoint/1"
CAPACITY_CHECKPOINT_MANIFEST_SCHEMA = (
    "uor-r4-softmax-trainer-capacity-checkpoint-manifest/1"
)
CAPACITY_ELAPSED_LEDGER_SCHEMA = "uor-r4-softmax-trainer-capacity-elapsed-ledger/1"
CAPACITY_SMOKE_SCHEMA = "uor-r4-softmax-trainer-capacity-overfit-smoke/1"
CAPACITY_HARDWARE_SCHEMA = "uor-r4-softmax-trainer-capacity-hardware-probe/1"
CAPACITY_TRAINING_RESULT_SCHEMA = "uor-r4-softmax-trainer-capacity-selection-result/1"
CAPACITY_SELECTION_SCHEMA = "uor-r4-softmax-trainer-capacity-selection/1"
PYTHON_PREFIX_SCHEMA = "uor-r4.r4-softmax-python-capacity-prefix-logits/1"
RUST_QUALIFICATION_SCHEMA = "uor-r4.r4-softmax-local-capacity-qualification/1"
SMOKE_ADMISSION_SCHEMA = "uor-r4-softmax-trainer-capacity-smoke-admission/1"
PREFIX_ADMISSION_SCHEMA = "uor-r4-softmax-trainer-capacity-prefix-admission/1"
REVEAL_OPENED_SCHEMA = "uor-r4-softmax-trainer-capacity-reveal-opened/1"
REVEAL_RESULT_SCHEMA = "uor-r4-softmax-trainer-capacity-reveal/1"
REVEAL_MANIFEST_SCHEMA = "uor-r4-softmax-trainer-capacity-reveal-manifest/1"

SMOKE_RESULT_RELATIVE_PATH = Path("preflight/capacity-overfit-smoke.json")
SMOKE_PREFIX_RELATIVE_PATH = Path("preflight/python-capacity-smoke-prefix.json")
SMOKE_RUST_RELATIVE_PATH = Path("preflight/rust-capacity-smoke-qualification.json")
SMOKE_ADMISSION_RELATIVE_PATH = Path("preflight/capacity-smoke-admission.json")
TRAINING_RESULT_RELATIVE_PATH = Path("capacity-training-result.json")
TRAINING_STATUS_RELATIVE_PATH = Path("capacity-training-status.json")
ELAPSED_LEDGER_RELATIVE_PATH = Path("capacity-elapsed-ledger.json")
PROGRESS_RELATIVE_PATH = Path("capacity-progress.jsonl")
SELECTION_RELATIVE_PATH = Path("selection/capacity-selection-manifest.json")
PYTHON_PREFIX_RELATIVE_PATH = Path("qualification/python-capacity-prefix-logits.json")
RUST_PREFIX_RELATIVE_PATH = Path("qualification/rust-capacity-prefix-qualification.json")
PREFIX_ADMISSION_RELATIVE_PATH = Path("qualification/capacity-prefix-admission.json")
REVEAL_OPENED_RELATIVE_PATH = Path("reveal/capacity-opened.json")
REVEAL_RESULT_RELATIVE_PATH = Path("reveal/capacity-reveal-result.json")
REVEAL_MANIFEST_RELATIVE_PATH = Path("reveal/capacity-reveal-manifest.json")

Backend = Literal["mps"]


def _require_mps_backend(backend: object) -> None:
    if backend != "mps":
        raise ValueError("#1019 backend must be mps")


def hardware_result_relative_path(backend: Backend) -> Path:
    _require_mps_backend(backend)
    return Path(f"preflight/capacity-hardware-{backend}.json")


def hardware_checkpoint_relative_path(backend: Backend) -> Path:
    _require_mps_backend(backend)
    return Path(f"preflight/capacity-hardware-{backend}-checkpoint.pt")


def hardware_elapsed_sample_relative_path(backend: Backend) -> Path:
    _require_mps_backend(backend)
    return Path(f"preflight/capacity-hardware-{backend}-elapsed-sample.json")


def hardware_evidence_relative_paths(backend: Backend) -> list[str]:
    """Return the complete probe chain that a frozen selection must bind."""
    _require_mps_backend(backend)
    paths: list[str] = []
    paths.extend(
        [
            str(hardware_result_relative_path(backend)),
            str(hardware_checkpoint_relative_path(backend)),
            str(hardware_checkpoint_relative_path(backend)) + ".manifest.json",
            str(hardware_elapsed_sample_relative_path(backend)),
        ]
    )
    return paths


@dataclass(frozen=True, slots=True)
class CapacityTrainConfig:
    """The sole #1019 optimization contract; changing a field requires a new issue."""

    seed: int = 1019
    batch_size: int = 16
    gradient_accumulation_steps: int = 4
    learning_rate: float = 3e-4
    minimum_learning_rate: float = 3e-5
    warmup_steps: int = 100
    weight_decay: float = 0.1
    adam_beta1: float = 0.9
    adam_beta2: float = 0.95
    adam_epsilon: float = 1e-8
    gradient_clip: float = 1.0
    evaluation_interval: int = 400
    checkpoint_interval: int = 100
    progress_interval: int = 10
    optimizer_steps: int = OPTIMIZER_STEPS
    train_tokens: int = TRAIN_TOKENS
    wall_ceiling_seconds: float = 8 * 60 * 60

    @property
    def tokens_per_optimizer_step(self) -> int:
        return (
            self.batch_size
            * self.gradient_accumulation_steps
            * CAPACITY_MODEL_CONFIG.max_position_embeddings
        )

    def validate(self) -> None:
        if self != CapacityTrainConfig():
            raise ValueError("#1019 permits only its exact frozen training config")
        if self.tokens_per_optimizer_step != TOKENS_PER_OPTIMIZER_STEP:
            raise ValueError("#1019 tokens-per-step arithmetic differs")
        if self.optimizer_steps * self.tokens_per_optimizer_step != self.train_tokens:
            raise ValueError("#1019 step and token budgets differ")
        if self.train_tokens != TRAIN_TOKEN_CAP:
            raise ValueError("#1019 training store and presentation budgets differ")
        if expected_parameter_count(CAPACITY_MODEL_CONFIG) != PARAMETER_COUNT:
            raise ValueError("#1019 exact parameter count differs")

    def as_contract(self) -> dict[str, Any]:
        self.validate()
        value = asdict(self)
        value["tokens_per_optimizer_step"] = self.tokens_per_optimizer_step
        value["tokens_per_parameter"] = self.train_tokens / PARAMETER_COUNT
        return value


def capacity_learning_rate(step: int, config: CapacityTrainConfig = CapacityTrainConfig()) -> float:
    config.validate()
    if not 0 <= step <= config.optimizer_steps:
        raise ValueError("#1019 optimizer step is outside the frozen schedule")
    if step <= config.warmup_steps:
        return config.learning_rate * step / config.warmup_steps
    progress = (step - config.warmup_steps) / (
        config.optimizer_steps - config.warmup_steps
    )
    cosine = 0.5 * (1.0 + math.cos(math.pi * progress))
    return config.minimum_learning_rate + cosine * (
        config.learning_rate - config.minimum_learning_rate
    )


def _dependency_versions() -> dict[str, str]:
    return {
        name: importlib.metadata.version(name)
        for name in ["blake3", "numpy", "safetensors", "tokenizers", "torch"]
    }


def _validate_backend_identity(identity: Any, backend: Backend) -> None:
    """Require the complete deterministic accelerator identity for one backend."""
    _require_mps_backend(backend)
    common = {
        "backend",
        "device_count",
        "device_name",
        "deterministic_algorithms",
        "dtype",
    }
    backend_specific = {"recommended_max_memory_bytes", "macos"}
    if not isinstance(identity, dict) or set(identity) != common | backend_specific:
        raise ValueError(f"#1019 {backend} backend identity fields differ")
    if (
        identity.get("backend") != backend
        or identity.get("device_count") != 1
        or not isinstance(identity.get("device_name"), str)
        or not identity["device_name"]
        or identity.get("deterministic_algorithms") is not True
        or identity.get("dtype") != "float32"
    ):
        raise ValueError(f"#1019 {backend} backend identity differs")
    recommended = identity.get("recommended_max_memory_bytes")
    if (
        (
            recommended is not None
            and (
                isinstance(recommended, bool)
                or not isinstance(recommended, int)
                or recommended <= 0
            )
        )
        or not isinstance(identity.get("macos"), str)
        or not identity["macos"]
    ):
        raise ValueError("#1019 MPS identity differs")


def _tool_root() -> Path:
    return Path(__file__).resolve().parents[2]


def _runtime_environment_identity(identity: dict[str, Any], backend: Backend) -> dict[str, Any]:
    _validate_backend_identity(identity, backend)
    lock_path = _tool_root() / "uv.lock"
    if not lock_path.is_file():
        raise FileNotFoundError("uv.lock is required before #1019 can be frozen")
    return {
        "python": ".".join(map(str, sys.version_info[:3])),
        "dependencies": _dependency_versions(),
        "uv_lock_cid": cid_file(lock_path),
        "backend": identity,
        "cpu_fallback": False,
        "dtype": "float32",
    }


def _validate_runtime_environment_envelope(
    environment: Any, identity: dict[str, Any]
) -> None:
    expected_keys = {
        "python",
        "dependencies",
        "uv_lock_cid",
        "backend",
        "cpu_fallback",
        "dtype",
    }
    dependencies = environment.get("dependencies") if isinstance(environment, dict) else None
    lock_cid = environment.get("uv_lock_cid") if isinstance(environment, dict) else None
    _require_cid(lock_cid, label="#1019 uv.lock CID")
    if (
        not isinstance(environment, dict)
        or set(environment) != expected_keys
        or not isinstance(environment.get("python"), str)
        or not environment["python"]
        or not isinstance(dependencies, dict)
        or set(dependencies) != {"blake3", "numpy", "safetensors", "tokenizers", "torch"}
        or any(not isinstance(value, str) or not value for value in dependencies.values())
        or environment.get("backend") != identity
        or environment.get("cpu_fallback") is not False
        or environment.get("dtype") != "float32"
    ):
        raise ValueError("#1019 runtime environment identity differs")


def _device_identity(device: torch.device, backend: Backend) -> dict[str, Any]:
    _require_mps_backend(backend)
    recommended = (
        int(torch.mps.recommended_max_memory())
        if hasattr(torch.mps, "recommended_max_memory")
        else None
    )
    return {
        "backend": "mps",
        "device_count": 1,
        "device_name": platform.machine(),
        "recommended_max_memory_bytes": recommended,
        "macos": platform.mac_ver()[0],
        "deterministic_algorithms": True,
        "dtype": "float32",
    }


def require_backend(seed: int, backend: Backend) -> tuple[torch.device, dict[str, Any]]:
    _require_mps_backend(backend)
    if os.environ.get("PYTORCH_ENABLE_MPS_FALLBACK", "0") not in {"", "0"}:
        raise RuntimeError("PYTORCH_ENABLE_MPS_FALLBACK must be unset or 0")
    if not torch.backends.mps.is_built() or not torch.backends.mps.is_available():
        raise RuntimeError("PyTorch MPS is unavailable; refusing CPU fallback")
    device = torch.device("mps")
    if hasattr(torch.mps, "manual_seed"):
        torch.mps.manual_seed(seed)
    torch.use_deterministic_algorithms(True)
    torch.manual_seed(seed)
    return device, _device_identity(device, backend)


def _sync(backend: Backend) -> None:
    _require_mps_backend(backend)
    if hasattr(torch, "mps"):
        torch.mps.synchronize()


def _memory_sample(backend: Backend, identity: dict[str, Any]) -> tuple[int, int | None]:
    _require_mps_backend(backend)
    allocated = (
        int(torch.mps.driver_allocated_memory())
        if hasattr(torch.mps, "driver_allocated_memory")
        else int(torch.mps.current_allocated_memory())
    )
    available = identity.get("recommended_max_memory_bytes")
    return allocated, int(available) if isinstance(available, int) else None


def _cpu_tree(value: Any) -> Any:
    if isinstance(value, Tensor):
        return value.detach().cpu()
    if isinstance(value, dict):
        return {key: _cpu_tree(item) for key, item in value.items()}
    if isinstance(value, list):
        return [_cpu_tree(item) for item in value]
    if isinstance(value, tuple):
        return tuple(_cpu_tree(item) for item in value)
    return value


def _accelerator_rng_state(backend: Backend) -> Any:
    _require_mps_backend(backend)
    if hasattr(torch.mps, "get_rng_state"):
        return torch.mps.get_rng_state().cpu()
    return None


def _restore_accelerator_rng_state(backend: Backend, state: Any) -> None:
    if state is None:
        return
    _require_mps_backend(backend)
    if hasattr(torch.mps, "set_rng_state"):
        torch.mps.set_rng_state(state)


def _optimizer_to_device(optimizer: torch.optim.Optimizer, device: torch.device) -> None:
    for state in optimizer.state.values():
        for key, value in state.items():
            if isinstance(value, Tensor):
                state[key] = value.to(device)


def _save_checkpoint(
    path: Path,
    *,
    model: R4SoftmaxForCausalLM,
    optimizer: torch.optim.Optimizer,
    optimizer_step: int,
    elapsed_seconds: float,
    best_dev_loss: float,
    development_candidates: list[dict[str, Any]],
    run_contract: dict[str, Any],
    run_contract_cid: str,
    backend: Backend,
) -> None:
    payload = {
        "schema": CAPACITY_CHECKPOINT_SCHEMA,
        "run_contract": run_contract,
        "run_contract_cid": run_contract_cid,
        "optimizer_step": optimizer_step,
        "tokens_seen": optimizer_step * TOKENS_PER_OPTIMIZER_STEP,
        "elapsed_seconds": elapsed_seconds,
        "best_dev_loss": best_dev_loss,
        "development_candidates": development_candidates,
        "model": _cpu_tree(model.state_dict()),
        "optimizer": _cpu_tree(optimizer.state_dict()),
        "cpu_rng_state": torch.get_rng_state(),
        "accelerator_rng_state": _accelerator_rng_state(backend),
        "backend": backend,
    }
    path.parent.mkdir(parents=True, exist_ok=True)
    temporary = path.with_name(f".{path.name}.part")
    try:
        torch.save(payload, temporary)
        with temporary.open("rb") as checkpoint_file:
            os.fsync(checkpoint_file.fileno())
        os.replace(temporary, path)
        _write_signed(
            path.with_suffix(path.suffix + ".manifest.json"),
            {
                "schema": CAPACITY_CHECKPOINT_MANIFEST_SCHEMA,
                "issue": ISSUE,
                "checkpoint_filename": path.name,
                "checkpoint_cid": cid_file(path),
                "run_contract_cid": run_contract_cid,
                "optimizer_step": optimizer_step,
                "tokens_seen": optimizer_step * TOKENS_PER_OPTIMIZER_STEP,
                "elapsed_seconds": elapsed_seconds,
                "best_dev_loss": best_dev_loss,
                "development_candidates": development_candidates,
                "backend": backend,
            },
        )
    except BaseException:
        temporary.unlink(missing_ok=True)
        raise


def _load_checkpoint(
    path: Path,
    *,
    model: R4SoftmaxForCausalLM,
    optimizer: torch.optim.Optimizer | None,
    device: torch.device,
    backend: Backend,
    run_contract_cid: str,
) -> dict[str, Any]:
    manifest_path = path.with_suffix(path.suffix + ".manifest.json")
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    if not isinstance(manifest, dict):
        raise ValueError("#1019 checkpoint manifest must be a JSON object")
    _verify_signed(manifest, label="#1019 checkpoint manifest")
    elapsed = _finite_float(
        manifest.get("elapsed_seconds"), label="#1019 checkpoint elapsed"
    )
    optimizer_step = manifest.get("optimizer_step")
    if (
        manifest.get("schema") != CAPACITY_CHECKPOINT_MANIFEST_SCHEMA
        or manifest.get("issue") != ISSUE
        or manifest.get("checkpoint_filename") != path.name
        or manifest.get("checkpoint_cid") != cid_file(path)
        or manifest.get("run_contract_cid") != run_contract_cid
        or manifest.get("backend") != backend
        or isinstance(optimizer_step, bool)
        or not isinstance(optimizer_step, int)
        or not 0 <= optimizer_step <= OPTIMIZER_STEPS
        or manifest.get("tokens_seen")
        != optimizer_step * TOKENS_PER_OPTIMIZER_STEP
        or elapsed < 0
        or not isinstance(manifest.get("development_candidates"), list)
    ):
        raise ValueError("#1019 checkpoint manifest identity differs")
    checkpoint = torch.load(path, map_location="cpu", weights_only=False)
    if checkpoint.get("schema") != CAPACITY_CHECKPOINT_SCHEMA:
        raise ValueError("unsupported #1019 checkpoint schema")
    if checkpoint.get("run_contract_cid") != run_contract_cid:
        raise ValueError("#1019 checkpoint belongs to another frozen run")
    if cid_bytes(canonical_json_bytes(checkpoint.get("run_contract"))) != run_contract_cid:
        raise ValueError("#1019 embedded run contract does not reproduce")
    if checkpoint.get("backend") != backend:
        raise ValueError("#1019 resume backend differs from the frozen run")
    for field in (
        "optimizer_step",
        "tokens_seen",
        "elapsed_seconds",
        "best_dev_loss",
        "development_candidates",
        "backend",
    ):
        if checkpoint.get(field) != manifest.get(field):
            raise ValueError(f"#1019 checkpoint {field} differs from its manifest")
    if not isinstance(checkpoint.get("model"), dict):
        raise ValueError("#1019 checkpoint model state is missing")
    if optimizer is not None and not isinstance(checkpoint.get("optimizer"), dict):
        raise ValueError("#1019 checkpoint optimizer state is missing")
    candidates = checkpoint.get("development_candidates")
    if checkpoint.get("run_contract", {}).get("schema") == (
        "uor-r4-softmax-trainer-capacity-hardware-probe-contract/1"
    ):
        if candidates != []:
            raise ValueError("#1019 hardware checkpoint has development candidates")
    else:
        validated = _validate_development_candidates(
            candidates, optimizer_step=optimizer_step
        )
        best_loss = _finite_float(
            checkpoint.get("best_dev_loss"), label="#1019 checkpoint best dev loss"
        )
        if best_loss != min(float(candidate["development_loss"]) for candidate in validated):
            raise ValueError("#1019 checkpoint best development loss differs")
    model.load_state_dict(checkpoint["model"], strict=True)
    if optimizer is not None:
        optimizer.load_state_dict(checkpoint["optimizer"])
        _optimizer_to_device(optimizer, device)
    torch.set_rng_state(checkpoint["cpu_rng_state"])
    _restore_accelerator_rng_state(backend, checkpoint.get("accelerator_rng_state"))
    return checkpoint


def _write_signed(path: Path, value: dict[str, Any]) -> dict[str, Any]:
    value = dict(value)
    value["result_cid"] = cid_bytes(canonical_json_bytes(value))
    atomic_write_json(path, value)
    return value


def _verify_signed(value: dict[str, Any], *, label: str) -> None:
    unsigned = dict(value)
    expected = unsigned.pop("result_cid", None)
    if expected != cid_bytes(canonical_json_bytes(unsigned)):
        raise ValueError(f"{label} result CID does not reproduce")


def _require_cid(value: object, *, label: str) -> str:
    if (
        not isinstance(value, str)
        or not value.startswith("blake3:")
        or len(value) != 71
        or any(character not in "0123456789abcdef" for character in value[7:])
    ):
        raise ValueError(f"{label} is not a lowercase BLAKE3 CID")
    return value


def _finite_float(value: object, *, label: str) -> float:
    if isinstance(value, bool) or not isinstance(value, (int, float)):
        raise ValueError(f"{label} must be a finite number")
    converted = float(value)
    if not math.isfinite(converted):
        raise ValueError(f"{label} must be a finite number")
    return converted


def _manifest_paths(manifest: dict[str, Any], *, label: str) -> set[str]:
    records = manifest.get("artifacts")
    if not isinstance(records, list) or not all(
        isinstance(record, dict) and isinstance(record.get("path"), str)
        for record in records
    ):
        raise ValueError(f"{label} has malformed artifact records")
    paths = [str(record["path"]) for record in records]
    if len(paths) != len(set(paths)):
        raise ValueError(f"{label} repeats an artifact path")
    return set(paths)


def _validate_development_candidates(
    value: object, *, optimizer_step: int
) -> list[dict[str, Any]]:
    expected_steps = [0, *range(400, optimizer_step + 1, 400)]
    if not isinstance(value, list) or len(value) != len(expected_steps):
        raise ValueError("#1019 development candidate lattice differs")
    candidates: list[dict[str, Any]] = []
    for expected_step, candidate in zip(expected_steps, value, strict=True):
        if not isinstance(candidate, dict) or set(candidate) != {
            "optimizer_step",
            "train_tokens",
            "development_loss",
        }:
            raise ValueError("#1019 development candidate shape differs")
        loss = _finite_float(
            candidate.get("development_loss"),
            label=f"#1019 development loss at step {expected_step}",
        )
        if (
            candidate.get("optimizer_step") != expected_step
            or candidate.get("train_tokens")
            != expected_step * TOKENS_PER_OPTIMIZER_STEP
            or loss < 0
        ):
            raise ValueError("#1019 development candidate identity differs")
        candidates.append(candidate)
    return candidates


def _as_f32(value: object, *, label: str) -> float:
    if isinstance(value, bool) or not isinstance(value, (int, float)):
        raise ValueError(f"{label} is not a finite f32 number")
    try:
        converted = struct.unpack("<f", struct.pack("<f", float(value)))[0]
    except (OverflowError, struct.error) as error:
        raise ValueError(f"{label} is not a finite f32 number") from error
    if not math.isfinite(converted):
        raise ValueError(f"{label} is not a finite f32 number")
    return converted


def _qualification_output_cid(policy: str, top1: int, logits: list[float]) -> str:
    digest = blake3()
    digest.update(policy.encode("utf-8"))
    digest.update(struct.pack("<I", top1))
    for value in logits:
        digest.update(struct.pack("<f", value))
    return f"blake3:{digest.hexdigest()}"


def _capacity_decision_cid(
    report: dict[str, Any],
    *,
    checkpoint: dict[str, Any],
    provenance: dict[str, Any],
    shape: dict[str, Any],
    evaluation_input: dict[str, Any],
    enabled: dict[str, Any],
    parity: dict[str, Any],
) -> str:
    delta = float(parity["maximum_absolute_logit_delta"])
    limit = float(parity["maximum_absolute_logit_delta_limit"])
    identity = {
        "schema": RUST_QUALIFICATION_SCHEMA,
        "checkpoint_tree_cid": checkpoint["checkpoint_tree_cid"],
        "weights_cid": checkpoint["weights_cid"],
        "provenance": provenance,
        "model_shape": shape,
        "token_store_cid": evaluation_input["token_store_cid"],
        "python_prefix_logits_cid": evaluation_input["python_prefix_logits_cid"],
        "python_prefix_result_cid": evaluation_input["python_prefix_result_cid"],
        "prefix_token_ids": evaluation_input["prefix_token_ids"],
        "enabled_policy_cid": enabled["policy_cid"],
        "enabled_output_cid": enabled["output_cid"],
        "enabled_audit_cid": enabled["audit_cid"],
        "python_top1_token_id": parity["python_top1_token_id"],
        "rust_top1_token_id": parity["rust_top1_token_id"],
        "identical_top1": parity["identical_top1"],
        "maximum_absolute_logit_delta_bits": struct.unpack(
            "<Q", struct.pack("<d", delta)
        )[0],
        "maximum_absolute_logit_delta_limit_bits": struct.unpack(
            "<Q", struct.pack("<d", limit)
        )[0],
        "maximum_absolute_logit_delta_within_limit": parity[
            "maximum_absolute_logit_delta_within_limit"
        ],
        "parity_passed": parity["passed"],
        "attention_off_executions": 0,
        "qualification_passed": report["qualification_passed"],
    }
    return cid_bytes(canonical_json_bytes(identity))


def _write_python_prefix(
    path: Path,
    *,
    model: R4SoftmaxForCausalLM,
    store: TokenStore,
    store_path: Path,
    device: torch.device,
    weights_cid: str,
) -> dict[str, Any]:
    token_ids = np.asarray(store.tokens[:PREFIX_PARITY_TOKENS], dtype=np.int64).tolist()
    if len(token_ids) != PREFIX_PARITY_TOKENS:
        raise ValueError("#1019 prefix store is too short")
    with torch.no_grad():
        inputs = torch.tensor([token_ids], dtype=torch.long, device=device)
        logits = model(inputs).logits[0, -1].float().cpu().tolist()
    if len(logits) != CAPACITY_MODEL_CONFIG.vocab_size or not all(
        math.isfinite(value) for value in logits
    ):
        raise ValueError("#1019 prefix logits are not finite vocabulary logits")
    return _write_signed(
        path,
        {
            "schema": PYTHON_PREFIX_SCHEMA,
            "weights_cid": weights_cid,
            "token_store_cid": cid_file(store_path),
            "prefix_token_ids": token_ids,
            "maximum_absolute_logit_delta_limit": PREFIX_LOGIT_ABS_TOLERANCE,
            "enabled": {
                "top1_token_id": int(np.argmax(np.asarray(logits))),
                "logits": logits,
            },
        },
    )


def _evaluate_fixed_sequences(
    model: R4SoftmaxForCausalLM,
    inputs: Tensor,
    targets: Tensor,
    device: torch.device,
    *,
    microbatch: int,
) -> float:
    model.eval()
    total = 0.0
    tokens = 0
    with torch.no_grad():
        for base in range(0, len(inputs), microbatch):
            batch_targets = targets[base : base + microbatch].to(device)
            output = model(inputs[base : base + microbatch].to(device), batch_targets)
            assert output.loss is not None
            count = batch_targets.numel()
            total += float(output.loss.detach().cpu()) * count
            tokens += count
    if tokens == 0:
        raise ValueError("#1019 fixed-sequence evaluation is empty")
    return total / tokens


def run_capacity_overfit_smoke(
    root: Path,
    *,
    backend: Backend,
    max_seconds: float = 600.0,
) -> dict[str, Any]:
    """Overfit exactly 64 fixed sequences and export an enabled-only Rust fixture."""
    if not 0 < max_seconds <= 600:
        raise ValueError("#1019 smoke ceiling must be in (0, 600]")
    if any((root / path).exists() for path in [SMOKE_RESULT_RELATIVE_PATH, SMOKE_ADMISSION_RELATIVE_PATH]):
        raise FileExistsError("#1019 smoke is create-once")
    training_view = load_capacity_training_view_manifest(root)
    device, device_identity = require_backend(1019, backend)
    store_path = root / TOKEN_RELATIVE_PATHS["train"]
    store = TokenStore(store_path)
    inputs, targets = store.first_sequences(64)
    model = R4SoftmaxForCausalLM(CAPACITY_MODEL_CONFIG).to(device)
    optimizer = torch.optim.AdamW(
        model.parameters(), lr=3e-3, betas=(0.9, 0.95), weight_decay=0.0
    )
    started = time.monotonic()
    initial = _evaluate_fixed_sequences(
        model, inputs, targets, device, microbatch=8
    )
    final = initial
    steps = 0
    while final > initial * 0.20 and time.monotonic() - started < max_seconds:
        base = (steps * 8) % 64
        indices = torch.arange(base, base + 8) % 64
        batch_inputs = inputs[indices].to(device)
        batch_targets = targets[indices].to(device)
        model.train()
        optimizer.zero_grad(set_to_none=True)
        output = model(batch_inputs, batch_targets)
        assert output.loss is not None
        output.loss.backward()
        torch.nn.utils.clip_grad_norm_(model.parameters(), 1.0)
        optimizer.step()
        steps += 1
        if steps % 16 == 0:
            final = _evaluate_fixed_sequences(
                model, inputs, targets, device, microbatch=8
            )
    _sync(backend)
    elapsed = time.monotonic() - started
    reduction = 1.0 - final / initial
    passed = reduction >= 0.80 and elapsed <= max_seconds
    result = _write_signed(
        root / SMOKE_RESULT_RELATIVE_PATH,
        {
            "schema": CAPACITY_SMOKE_SCHEMA,
            "issue": ISSUE,
            "terminal": "PASS" if passed else "FAIL",
            "dataset_manifest_cid": training_view["dataset_manifest_cid"],
            "training_view_manifest_cid": training_view["manifest_cid"],
            "split_policy_cid": training_view["split_policy_cid"],
            "trainer_implementation": trainer_implementation_contract(),
            "model": CAPACITY_MODEL_CONFIG.as_contract(),
            "parameter_count": model.parameter_count(),
            "backend": device_identity,
            "sequences": 64,
            "initial_loss": initial,
            "final_loss": final,
            "loss_reduction_fraction": reduction,
            "required_loss_reduction_fraction": 0.80,
            "optimizer_steps": steps,
            "elapsed_seconds": elapsed,
            "wall_ceiling_seconds": max_seconds,
            "attention_off_executions": 0,
        },
    )
    export = export_hugging_face_snapshot(
        model,
        output_dir=root / "preflight/smoke-export",
        tokenizer_path=root / TOKENIZER_RELATIVE_PATH,
        training_result=result,
        dataset_manifest_cid=str(training_view["dataset_manifest_cid"]),
        training_view_manifest_cid=str(training_view["manifest_cid"]),
        split_policy_cid=str(training_view["split_policy_cid"]),
        run_contract_cid=str(result["result_cid"]),
        selected_checkpoint_cid=None,
    )
    _write_python_prefix(
        root / SMOKE_PREFIX_RELATIVE_PATH,
        model=model,
        store=store,
        store_path=store_path,
        device=device,
        weights_cid=str(export["weights_cid"]),
    )
    if not passed:
        raise RuntimeError("#1019 64-sequence overfit gate failed")
    return result


def _validate_rust_report(
    report: dict[str, Any],
    *,
    prefix: dict[str, Any],
    prefix_path: Path,
    export: dict[str, Any],
    export_root: Path,
) -> None:
    expected_report_fields = {
        "schema",
        "issue",
        "decision_cid",
        "checkpoint",
        "provenance",
        "model_shape",
        "evaluation_input",
        "enabled",
        "enabled_prefix_parity",
        "attention_off_executions",
        "qualification_passed",
        "source_read_audit",
        "execution",
        "timing",
        "nonclaims",
    }
    if (
        set(report) != expected_report_fields
        or
        report.get("schema") != RUST_QUALIFICATION_SCHEMA
        or report.get("issue") != ISSUE
        or report.get("qualification_passed") is not True
        or report.get("attention_off_executions") != 0
    ):
        raise ValueError("#1019 enabled-only Rust qualification did not pass")

    checkpoint = report.get("checkpoint")
    provenance = report.get("provenance")
    shape = report.get("model_shape")
    evaluation_input = report.get("evaluation_input")
    enabled = report.get("enabled")
    parity = report.get("enabled_prefix_parity")
    source = report.get("source_read_audit")
    if not all(
        isinstance(value, dict)
        for value in (checkpoint, provenance, shape, evaluation_input, enabled, parity, source)
    ):
        raise ValueError("#1019 Rust qualification lacks exact evidence objects")
    if not isinstance(report.get("execution"), dict) or not isinstance(
        report.get("timing"), dict
    ) or not isinstance(report.get("nonclaims"), list):
        raise ValueError("#1019 Rust qualification lacks execution metadata")

    expected_provenance = {
        "export_manifest_cid": export["manifest_cid"],
        "export_tree_cid": export["tree_cid"],
        "dataset_manifest_cid": export["dataset_manifest_cid"],
        "training_view_manifest_cid": export["training_view_manifest_cid"],
        "split_policy_cid": export["split_policy_cid"],
        "run_contract_cid": export["run_contract_cid"],
        "training_result_cid": export["training_result_cid"],
        "selected_checkpoint_cid": export["selected_checkpoint_cid"],
        "config_cid": export["config_cid"],
        "tokenizer_cid": export["tokenizer_cid"],
        "weights_cid": export["weights_cid"],
        "reveal_manifest_cid": None,
        "reveal_tree_cid": None,
    }
    if set(provenance) != set(expected_provenance):
        raise ValueError("#1019 Rust qualification provenance fields differ")
    for field, expected in expected_provenance.items():
        if provenance.get(field) != expected:
            raise ValueError(f"#1019 Rust qualification {field} differs")
        if expected is not None:
            _require_cid(provenance[field], label=f"#1019 provenance.{field}")

    expected_checkpoint_files = []
    for relative in (
        "config.json",
        "export-manifest.json",
        "model.safetensors",
        "tokenizer.json",
        "training-result.json",
    ):
        path = export_root / relative
        if not path.is_file() or path.is_symlink():
            raise ValueError(f"#1019 export {relative} is not a regular file")
        expected_checkpoint_files.append(
            {"path": relative, "bytes": path.stat().st_size, "kappa": cid_file(path)}
        )
    if (
        checkpoint.get("files") != expected_checkpoint_files
        or checkpoint.get("config_cid") != export["config_cid"]
        or checkpoint.get("tokenizer_cid") != export["tokenizer_cid"]
        or checkpoint.get("weights_cid") != export["weights_cid"]
        or checkpoint.get("bos_token_id") != CAPACITY_MODEL_CONFIG.bos_token_id
        or checkpoint.get("eos_token_id") != CAPACITY_MODEL_CONFIG.eos_token_id
        or checkpoint.get("weights_cid_scope")
        != (
            "Safetensors shard bytes in the loader's canonical shard order; "
            "not the checkpoint-tree CID"
        )
        or not isinstance(checkpoint.get("model_path"), str)
        or not checkpoint["model_path"]
        or not isinstance(checkpoint.get("tokenizer"), dict)
        or checkpoint["tokenizer"].get("tokenizer_cid") != export["tokenizer_cid"]
        or not isinstance(checkpoint.get("exact_backend"), dict)
    ):
        raise ValueError("#1019 Rust checkpoint binding differs from the exact export")
    _require_cid(
        checkpoint.get("checkpoint_tree_cid"), label="#1019 checkpoint.checkpoint_tree_cid"
    )

    expected_shape = {
        "dimension": CAPACITY_MODEL_CONFIG.hidden_size,
        "hidden_dimension": CAPACITY_MODEL_CONFIG.intermediate_size,
        "layers": CAPACITY_MODEL_CONFIG.num_hidden_layers,
        "query_heads": CAPACITY_MODEL_CONFIG.num_attention_heads,
        "key_value_heads": CAPACITY_MODEL_CONFIG.num_key_value_heads,
        "head_size": CAPACITY_MODEL_CONFIG.head_dim,
        "vocabulary": CAPACITY_MODEL_CONFIG.vocab_size,
        "sequence_capacity": PREFIX_PARITY_TOKENS,
    }
    if shape != expected_shape:
        raise ValueError("#1019 Rust qualification used a non-frozen model shape")

    if (
        set(evaluation_input)
        != {
            "token_store_cid",
            "python_prefix_logits_path",
            "python_prefix_logits_cid",
            "python_prefix_result_cid",
            "prefix_token_ids",
            "sources_unchanged_across_execution",
        }
        or evaluation_input.get("python_prefix_result_cid") != prefix["result_cid"]
        or evaluation_input.get("token_store_cid") != prefix["token_store_cid"]
        or evaluation_input.get("prefix_token_ids") != prefix["prefix_token_ids"]
        or evaluation_input.get("python_prefix_logits_cid") != cid_file(prefix_path)
        or not isinstance(evaluation_input.get("python_prefix_logits_path"), str)
        or not evaluation_input["python_prefix_logits_path"]
        or evaluation_input.get("sources_unchanged_across_execution") is not True
    ):
        raise ValueError("#1019 Rust qualification used a different prefix input")
    for field in ("token_store_cid", "python_prefix_logits_cid", "python_prefix_result_cid"):
        _require_cid(evaluation_input[field], label=f"#1019 evaluation_input.{field}")

    fixture_arm = prefix.get("enabled")
    if not isinstance(fixture_arm, dict) or set(fixture_arm) != {"top1_token_id", "logits"}:
        raise ValueError("#1019 Python prefix enabled arm differs")
    fixture_logits_value = fixture_arm.get("logits")
    if not isinstance(fixture_logits_value, list) or len(fixture_logits_value) != 4096:
        raise ValueError("#1019 Python prefix must contain 4096 logits")
    fixture_logits = [
        _as_f32(value, label=f"#1019 Python logit {index}")
        for index, value in enumerate(fixture_logits_value)
    ]
    python_top1 = max(
        range(len(fixture_logits)), key=lambda index: (fixture_logits[index], -index)
    )
    if fixture_arm.get("top1_token_id") != python_top1:
        raise ValueError("#1019 Python prefix top-1 does not reproduce")

    expected_parity_fields = {
        "attention_output_policy",
        "python_top1_token_id",
        "rust_top1_token_id",
        "identical_top1",
        "maximum_absolute_logit_delta",
        "maximum_absolute_logit_delta_limit",
        "maximum_absolute_logit_delta_within_limit",
        "python_logits",
        "rust_logits",
        "passed",
    }
    if (
        set(parity) != expected_parity_fields
        or parity.get("attention_output_policy") != "causal-attention-output-enabled/1"
        or parity.get("identical_top1") is not True
        or parity.get("maximum_absolute_logit_delta_within_limit") is not True
        or parity.get("passed") is not True
        or parity.get("python_top1_token_id") != python_top1
        or parity.get("maximum_absolute_logit_delta_limit")
        != PREFIX_LOGIT_ABS_TOLERANCE
    ):
        raise ValueError("#1019 Python/Rust prefix parity failed")
    parity_python_value = parity.get("python_logits")
    parity_rust_value = parity.get("rust_logits")
    if not isinstance(parity_python_value, list) or not isinstance(parity_rust_value, list):
        raise ValueError("#1019 Rust parity omits its logit vectors")
    if len(parity_python_value) != 4096 or len(parity_rust_value) != 4096:
        raise ValueError("#1019 Rust parity logit vectors have the wrong length")
    parity_python = [
        _as_f32(value, label=f"#1019 Rust-bound Python logit {index}")
        for index, value in enumerate(parity_python_value)
    ]
    rust_logits = [
        _as_f32(value, label=f"#1019 Rust logit {index}")
        for index, value in enumerate(parity_rust_value)
    ]
    if parity_python != fixture_logits:
        raise ValueError("#1019 Rust parity binds different Python logits")
    rust_top1 = max(range(len(rust_logits)), key=lambda index: (rust_logits[index], -index))
    reproduced_delta = max(
        abs(python - rust)
        for python, rust in zip(parity_python, rust_logits, strict=True)
    )
    delta = parity.get("maximum_absolute_logit_delta")
    if (
        isinstance(delta, bool)
        or not isinstance(delta, (int, float))
        or not math.isfinite(float(delta))
        or float(delta) != reproduced_delta
        or reproduced_delta > PREFIX_LOGIT_ABS_TOLERANCE
        or parity.get("rust_top1_token_id") != rust_top1
        or parity.get("identical_top1") != (python_top1 == rust_top1)
    ):
        raise ValueError("#1019 Python/Rust prefix parity arithmetic differs")

    if set(enabled) != {
        "attention_output_policy",
        "policy_cid",
        "top1_token_id",
        "output_cid",
        "audit_cid",
        "audit",
    } or enabled.get("attention_output_policy") != "causal-attention-output-enabled/1":
        raise ValueError("#1019 Rust enabled arm differs")
    if enabled.get("top1_token_id") != rust_top1:
        raise ValueError("#1019 Rust enabled top-1 differs from its logits")
    audit = enabled.get("audit")
    if not isinstance(audit, dict):
        raise ValueError("#1019 Rust qualification has no exact audit")
    applications = PREFIX_PARITY_TOKENS * CAPACITY_MODEL_CONFIG.num_hidden_layers
    if (
        audit.get("sessions") != 1
        or audit.get("positions_per_session") != PREFIX_PARITY_TOKENS
        or audit.get("total_positions") != PREFIX_PARITY_TOKENS
        or audit.get("selected_layer_count") != 12
        or audit.get("all_layers_selected") is not True
        or audit.get("causal_audits_exact") != 1
        or audit.get("projection_audits_exact") != 1
        or audit.get("r4_audits_exact") != 1
        or audit.get("output_policy_audits_exact") != 1
        or audit.get("future_reads") != 0
        or audit.get("output_policy_applications") != applications
        or audit.get("enabled_applications") != applications
        or audit.get("zeroed_applications") != 0
        or audit.get("output_lanes")
        != applications * CAPACITY_MODEL_CONFIG.hidden_size
        or audit.get("nonzero_lanes_before_policy")
        != audit.get("nonzero_lanes_after_policy")
        or audit.get("applications_by_layer")
        != [PREFIX_PARITY_TOKENS] * CAPACITY_MODEL_CONFIG.num_hidden_layers
    ):
        raise ValueError("#1019 Rust qualification is not exact all-12-layer R4")
    _require_cid(audit.get("state_ledger_cid"), label="#1019 audit.state_ledger_cid")
    expected_policy_cid = cid_bytes(
        canonical_json_bytes(
            [
                R4_POLICY_IDENTITY,
                "causal-attention-output-enabled/1",
                "all-decoder-layers",
            ]
        )
    )
    expected_output_cid = _qualification_output_cid(
        "causal-attention-output-enabled/1", rust_top1, rust_logits
    )
    expected_audit_cid = cid_bytes(canonical_json_bytes(audit))
    if (
        enabled.get("policy_cid") != expected_policy_cid
        or enabled.get("output_cid") != expected_output_cid
        or enabled.get("audit_cid") != expected_audit_cid
    ):
        raise ValueError("#1019 Rust enabled policy/output/audit CIDs do not reproduce")

    if (
        source.get("checkpoint_tree_scans") != 2
        or source.get("checkpoint_tree_file_reads") != 10
        or source.get("tokenizer_loads") != 1
        or source.get("oracle_loads") != 1
        or source.get("local_checkpoint_forward_steps") != PREFIX_PARITY_TOKENS
        or source.get("provider_calls") != 0
        or source.get("ollama_calls") != 0
        or source.get("prior_trace_reads") != 0
        or source.get("tree_unchanged_across_execution") is not True
    ):
        raise ValueError("#1019 Rust qualification used a prohibited source")
    expected_decision_cid = _capacity_decision_cid(
        report,
        checkpoint=checkpoint,
        provenance=provenance,
        shape=shape,
        evaluation_input=evaluation_input,
        enabled=enabled,
        parity=parity,
    )
    if report.get("decision_cid") != expected_decision_cid:
        raise ValueError("#1019 Rust qualification decision CID does not reproduce")


def admit_capacity_smoke(root: Path, rust_report_path: Path) -> dict[str, Any]:
    if (root / SMOKE_ADMISSION_RELATIVE_PATH).exists():
        raise FileExistsError("#1019 smoke admission is create-once")
    smoke = json.loads((root / SMOKE_RESULT_RELATIVE_PATH).read_text(encoding="utf-8"))
    _verify_signed(smoke, label="#1019 smoke")
    if smoke.get("terminal") != "PASS":
        raise ValueError("#1019 smoke did not pass")
    prefix = json.loads((root / SMOKE_PREFIX_RELATIVE_PATH).read_text(encoding="utf-8"))
    _verify_signed(prefix, label="#1019 smoke prefix")
    report_bytes = rust_report_path.read_bytes()
    report = json.loads(report_bytes)
    if not isinstance(report, dict):
        raise ValueError("#1019 Rust smoke qualification must be a JSON object")
    export_root = root / "preflight/smoke-export"
    export = verify_bound_manifest(
        export_root / "export-manifest.json", artifact_root=export_root
    )
    if (
        export.get("dataset_manifest_cid") != smoke["dataset_manifest_cid"]
        or export.get("training_view_manifest_cid")
        != smoke["training_view_manifest_cid"]
        or export.get("split_policy_cid") != smoke["split_policy_cid"]
        or export.get("run_contract_cid") != smoke["result_cid"]
        or export.get("weights_cid") != prefix["weights_cid"]
        or export.get("tokenizer_cid") != TOKENIZER_CID
    ):
        raise ValueError("#1019 smoke export identity differs")
    _validate_rust_report(
        report,
        prefix=prefix,
        prefix_path=root / SMOKE_PREFIX_RELATIVE_PATH,
        export=export,
        export_root=export_root,
    )
    atomic_write(root / SMOKE_RUST_RELATIVE_PATH, report_bytes)
    return write_bound_manifest(
        root / SMOKE_ADMISSION_RELATIVE_PATH,
        {
            "schema": SMOKE_ADMISSION_SCHEMA,
            "issue": ISSUE,
            "smoke_result_cid": smoke["result_cid"],
            "python_prefix_result_cid": prefix["result_cid"],
            "rust_decision_cid": report.get("decision_cid"),
            "qualification_passed": True,
            "attention_off_executions": 0,
            "trainer_implementation_tree_cid": smoke["trainer_implementation"]["tree_cid"],
        },
        artifact_root=root,
        relative_paths=[
            str(SMOKE_RESULT_RELATIVE_PATH),
            str(SMOKE_PREFIX_RELATIVE_PATH),
            str(SMOKE_RUST_RELATIVE_PATH),
            "preflight/smoke-export/config.json",
            "preflight/smoke-export/model.safetensors",
            "preflight/smoke-export/tokenizer.json",
            "preflight/smoke-export/training-result.json",
            "preflight/smoke-export/export-manifest.json",
        ],
    )


def load_capacity_smoke_admission(root: Path) -> dict[str, Any]:
    admission = verify_bound_manifest(root / SMOKE_ADMISSION_RELATIVE_PATH, artifact_root=root)
    if (
        admission.get("schema") != SMOKE_ADMISSION_SCHEMA
        or admission.get("qualification_passed") is not True
        or admission.get("attention_off_executions") != 0
    ):
        raise ValueError("#1019 smoke admission differs")
    smoke = json.loads((root / SMOKE_RESULT_RELATIVE_PATH).read_text(encoding="utf-8"))
    prefix_path = root / SMOKE_PREFIX_RELATIVE_PATH
    prefix = json.loads(prefix_path.read_text(encoding="utf-8"))
    report = json.loads((root / SMOKE_RUST_RELATIVE_PATH).read_text(encoding="utf-8"))
    export_root = root / "preflight/smoke-export"
    export = verify_bound_manifest(
        export_root / "export-manifest.json", artifact_root=export_root
    )
    if not all(isinstance(value, dict) for value in (smoke, prefix, report)):
        raise ValueError("#1019 smoke admission artifacts are not JSON objects")
    _verify_signed(smoke, label="#1019 smoke")
    _verify_signed(prefix, label="#1019 smoke prefix")
    if (
        export.get("dataset_manifest_cid") != smoke["dataset_manifest_cid"]
        or export.get("training_view_manifest_cid")
        != smoke["training_view_manifest_cid"]
        or export.get("split_policy_cid") != smoke["split_policy_cid"]
        or export.get("run_contract_cid") != smoke["result_cid"]
        or export.get("weights_cid") != prefix["weights_cid"]
        or export.get("tokenizer_cid") != TOKENIZER_CID
    ):
        raise ValueError("#1019 smoke admission export identity differs")
    _validate_rust_report(
        report,
        prefix=prefix,
        prefix_path=prefix_path,
        export=export,
        export_root=export_root,
    )
    if (
        admission.get("smoke_result_cid") != smoke["result_cid"]
        or admission.get("python_prefix_result_cid") != prefix["result_cid"]
        or admission.get("rust_decision_cid") != report["decision_cid"]
        or admission.get("trainer_implementation_tree_cid")
        != smoke["trainer_implementation"]["tree_cid"]
    ):
        raise ValueError("#1019 smoke admission evidence identities differ")
    return admission


def run_capacity_hardware_probe(
    root: Path,
    *,
    backend: Backend,
    steps: int = 200,
) -> dict[str, Any]:
    """Measure the exact full-shape optimizer step before authorizing the main run."""
    _require_mps_backend(backend)
    if steps != 200:
        raise ValueError("#1019 hardware probe is exactly 200 optimizer steps")
    mps_failure_prerequisite = None
    result_path = root / hardware_result_relative_path(backend)
    if result_path.exists():
        raise FileExistsError(f"#1019 {backend} hardware probe is create-once")
    training_view = load_capacity_training_view_manifest(root)
    smoke_admission = load_capacity_smoke_admission(root)
    current_tree = trainer_implementation_contract()
    if current_tree["tree_cid"] != smoke_admission["trainer_implementation_tree_cid"]:
        raise ValueError("trainer changed after #1019 smoke admission")
    device, identity = require_backend(1019, backend)
    store = TokenStore(root / TOKEN_RELATIVE_PATHS["train"])
    dev_store = TokenStore(root / TOKEN_RELATIVE_PATHS["dev"])
    model = R4SoftmaxForCausalLM(CAPACITY_MODEL_CONFIG).to(device)
    optimizer = torch.optim.AdamW(
        model.parameters(),
        lr=3e-4,
        betas=(0.9, 0.95),
        eps=1e-8,
        weight_decay=0.1,
    )
    probe_contract = {
        "schema": "uor-r4-softmax-trainer-capacity-hardware-probe-contract/1",
        "issue": ISSUE,
        "dataset_manifest_cid": training_view["dataset_manifest_cid"],
        "training_view_manifest_cid": training_view["manifest_cid"],
        "smoke_admission_manifest_cid": smoke_admission["manifest_cid"],
        "trainer_implementation": current_tree,
        "backend": identity,
        "environment": _runtime_environment_identity(identity, backend),
        "mps_failure_prerequisite": mps_failure_prerequisite,
        "model": CAPACITY_MODEL_CONFIG.as_contract(),
        "parameter_count": PARAMETER_COUNT,
        "optimizer_steps": steps,
        "tokens_per_optimizer_step": TOKENS_PER_OPTIMIZER_STEP,
        "checkpoint_interval": 100,
        "maximum_best_checkpoint_saves": MAXIMUM_BEST_CHECKPOINT_SAVES,
    }
    probe_contract_cid = cid_bytes(canonical_json_bytes(probe_contract))
    checkpoint_path = root / hardware_checkpoint_relative_path(backend)
    checkpoint_save_seconds_samples: list[float] = []
    peak = 0
    started = time.monotonic()
    for step in range(1, steps + 1):
        model.train()
        optimizer.zero_grad(set_to_none=True)
        microbatch_losses: list[Tensor] = []
        for accumulation in range(4):
            inputs, targets = store.random_batch(
                seed=1019,
                batch_index=(step - 1) * 4 + accumulation,
                batch_size=16,
            )
            output = model(inputs.to(device), targets.to(device))
            assert output.loss is not None
            (output.loss / 4).backward()
            microbatch_losses.append(output.loss.detach())
        torch.nn.utils.clip_grad_norm_(model.parameters(), 1.0)
        optimizer.step()
        if step % 10 == 0 or step == steps:
            _write_signed(
                root / hardware_elapsed_sample_relative_path(backend),
                {
                    "schema": "uor-r4-softmax-trainer-capacity-hardware-elapsed-sample/1",
                    "issue": ISSUE,
                    "backend": backend,
                    "probe_contract_cid": probe_contract_cid,
                    "optimizer_step": step,
                    "train_tokens": step * TOKENS_PER_OPTIMIZER_STEP,
                    "elapsed_seconds": time.monotonic() - started,
                },
            )
        allocated, _ = _memory_sample(backend, identity)
        peak = max(peak, allocated)
        if step % 100 == 0:
            checkpoint_started = time.monotonic()
            _save_checkpoint(
                checkpoint_path,
                model=model,
                optimizer=optimizer,
                optimizer_step=step,
                elapsed_seconds=time.monotonic() - started,
                best_dev_loss=0.0,
                development_candidates=[],
                run_contract=probe_contract,
                run_contract_cid=probe_contract_cid,
                backend=backend,
            )
            checkpoint_save_seconds_samples.append(
                time.monotonic() - checkpoint_started
            )
        if step % 10 == 0 or step == steps:
            _sync(backend)
            elapsed_so_far = time.monotonic() - started
            rate = step / elapsed_so_far if elapsed_so_far > 0 else 0.0
            mean_microbatch_loss = float(torch.stack(microbatch_losses).mean().cpu())
            progress = {
                "schema": "uor-r4-softmax-trainer-capacity-hardware-progress/1",
                "issue": ISSUE,
                "backend": backend,
                "optimizer_step": step,
                "optimizer_steps_total": steps,
                "optimizer_steps_remaining": steps - step,
                "mean_microbatch_loss": mean_microbatch_loss,
                "elapsed_seconds": elapsed_so_far,
                "optimizer_steps_per_second": rate,
                "probe_eta_seconds": (steps - step) / rate if rate > 0 else None,
                "projected_optimizer_and_checkpoint_seconds": (
                    OPTIMIZER_STEPS / rate if rate > 0 else None
                ),
                "safety_projected_optimizer_and_checkpoint_seconds": (
                    OPTIMIZER_STEPS / rate * 1.25 if rate > 0 else None
                ),
                "peak_accelerator_memory_bytes": peak,
            }
            print(json.dumps(progress, sort_keys=True), flush=True)
    _sync(backend)
    optimizer_loop_seconds = time.monotonic() - started
    reload_started = time.monotonic()
    _load_checkpoint(
        checkpoint_path,
        model=model,
        optimizer=optimizer,
        device=device,
        backend=backend,
        run_contract_cid=probe_contract_cid,
    )
    _sync(backend)
    checkpoint_reload_seconds = time.monotonic() - reload_started
    dev_started = time.monotonic()
    probe_dev_loss = evaluate(model, dev_store, device, 16)
    _sync(backend)
    complete_dev_evaluation_seconds = time.monotonic() - dev_started
    allocated, _ = _memory_sample(backend, identity)
    peak = max(peak, allocated)
    complete_dev_evaluations = 2 + OPTIMIZER_STEPS // 400
    projected_optimizer_and_checkpoint_seconds = (
        optimizer_loop_seconds / steps * OPTIMIZER_STEPS
    )
    projected_development_seconds = (
        complete_dev_evaluation_seconds * complete_dev_evaluations
    )
    maximum_checkpoint_save_seconds = max(checkpoint_save_seconds_samples)
    projected_best_checkpoint_seconds = (
        maximum_checkpoint_save_seconds * MAXIMUM_BEST_CHECKPOINT_SAVES
    )
    projected = (
        projected_optimizer_and_checkpoint_seconds
        + projected_development_seconds
        + projected_best_checkpoint_seconds
        + checkpoint_reload_seconds
    )
    elapsed = (
        optimizer_loop_seconds
        + checkpoint_reload_seconds
        + complete_dev_evaluation_seconds
    )
    safety_projected = projected * 1.25
    _, available = _memory_sample(backend, identity)
    memory_fraction = peak / available if available else None
    time_passed = safety_projected <= 8 * 60 * 60
    memory_passed = memory_fraction is not None and memory_fraction <= 0.80
    passed = time_passed and memory_passed
    result = _write_signed(
        result_path,
        {
            "schema": CAPACITY_HARDWARE_SCHEMA,
            "issue": ISSUE,
            "terminal": "PASS_HARDWARE_ADMISSION" if passed else "UNAVAILABLE_HARDWARE_BUDGET",
            "dataset_manifest_cid": training_view["dataset_manifest_cid"],
            "training_view_manifest_cid": training_view["manifest_cid"],
            "smoke_admission_manifest_cid": smoke_admission["manifest_cid"],
            "trainer_implementation": current_tree,
            "backend": identity,
            "mps_failure_prerequisite": mps_failure_prerequisite,
            "probe_contract": probe_contract,
            "probe_contract_cid": probe_contract_cid,
            "probe_checkpoint_path": str(hardware_checkpoint_relative_path(backend)),
            "probe_checkpoint_cid": cid_file(checkpoint_path),
            "probe_checkpoint_manifest_path": str(
                hardware_checkpoint_relative_path(backend)
            )
            + ".manifest.json",
            "probe_checkpoint_manifest_cid": cid_file(
                checkpoint_path.with_suffix(checkpoint_path.suffix + ".manifest.json")
            ),
            "checkpoint_interval": 100,
            "checkpoint_reload_passed": True,
            "elapsed_sample_path": str(hardware_elapsed_sample_relative_path(backend)),
            "elapsed_sample_cid": cid_file(
                root / hardware_elapsed_sample_relative_path(backend)
            ),
            "probe_optimizer_steps": steps,
            "probe_token_presentations": steps * TOKENS_PER_OPTIMIZER_STEP,
            "elapsed_seconds": elapsed,
            "optimizer_loop_seconds": optimizer_loop_seconds,
            "optimizer_steps_per_second": steps / optimizer_loop_seconds,
            "checkpoint_reload_seconds": checkpoint_reload_seconds,
            "checkpoint_save_seconds_samples": checkpoint_save_seconds_samples,
            "maximum_measured_checkpoint_save_seconds": (
                maximum_checkpoint_save_seconds
            ),
            "projected_best_checkpoint_saves": MAXIMUM_BEST_CHECKPOINT_SAVES,
            "projected_best_checkpoint_seconds": projected_best_checkpoint_seconds,
            "probe_complete_dev_loss": probe_dev_loss,
            "complete_dev_evaluation_seconds": complete_dev_evaluation_seconds,
            "projected_complete_dev_evaluations": complete_dev_evaluations,
            "projected_optimizer_and_checkpoint_seconds": (
                projected_optimizer_and_checkpoint_seconds
            ),
            "projected_development_evaluation_seconds": projected_development_seconds,
            "projected_training_seconds": projected,
            "safety_factor": 1.25,
            "safety_projected_training_seconds": safety_projected,
            "maximum_safety_projected_seconds": 8 * 60 * 60,
            "peak_accelerator_memory_bytes": peak,
            "available_accelerator_memory_bytes": available,
            "peak_memory_fraction": memory_fraction,
            "maximum_memory_fraction": 0.80,
            "time_passed": time_passed,
            "memory_passed": memory_passed,
            "main_run_authorized": passed,
            "partial_checkpoint_interpretable": False,
        },
    )
    if not passed:
        print(json.dumps(result, sort_keys=True), flush=True)
        raise RuntimeError("UNAVAILABLE_HARDWARE_BUDGET")
    return result


def _validate_capacity_hardware_evidence(
    root: Path,
    *,
    backend: Backend,
    training_view: dict[str, Any],
    smoke_admission: dict[str, Any],
    result: dict[str, Any],
    require_pass: bool,
    require_current_environment: bool,
) -> dict[str, Any]:
    """Recompute one signed hardware decision without touching sealed inputs."""
    _require_mps_backend(backend)
    _verify_signed(result, label="#1019 hardware probe")
    backend_identity = result.get("backend")
    _validate_backend_identity(backend_identity, backend)
    elapsed = _finite_float(result.get("elapsed_seconds"), label="#1019 probe elapsed")
    projected = _finite_float(
        result.get("projected_training_seconds"), label="#1019 projected time"
    )
    safety_projected = _finite_float(
        result.get("safety_projected_training_seconds"),
        label="#1019 safety-projected time",
    )
    optimizer_loop_seconds = _finite_float(
        result.get("optimizer_loop_seconds"), label="#1019 optimizer probe time"
    )
    checkpoint_reload_seconds = _finite_float(
        result.get("checkpoint_reload_seconds"), label="#1019 checkpoint reload time"
    )
    raw_checkpoint_samples = result.get("checkpoint_save_seconds_samples")
    if not isinstance(raw_checkpoint_samples, list) or len(raw_checkpoint_samples) != 2:
        raise ValueError("#1019 checkpoint-save timing samples differ")
    checkpoint_samples = [
        _finite_float(value, label="#1019 checkpoint-save timing")
        for value in raw_checkpoint_samples
    ]
    if any(value <= 0 for value in checkpoint_samples):
        raise ValueError("#1019 checkpoint-save timing must be positive")
    maximum_checkpoint_save_seconds = max(checkpoint_samples)
    dev_seconds = _finite_float(
        result.get("complete_dev_evaluation_seconds"),
        label="#1019 development evaluation time",
    )
    probe_dev_loss = _finite_float(
        result.get("probe_complete_dev_loss"), label="#1019 probe development loss"
    )
    peak = result.get("peak_accelerator_memory_bytes")
    available = result.get("available_accelerator_memory_bytes")
    memory_fraction = _finite_float(
        result.get("peak_memory_fraction"), label="#1019 peak memory fraction"
    )
    if (
        isinstance(peak, bool)
        or not isinstance(peak, int)
        or peak < 0
        or isinstance(available, bool)
        or not isinstance(available, int)
        or available <= 0
    ):
        raise ValueError("#1019 probe memory measurements differ")
    complete_dev_evaluations = 2 + OPTIMIZER_STEPS // 400
    expected_optimizer_projected = optimizer_loop_seconds / 200 * OPTIMIZER_STEPS
    expected_development_projected = dev_seconds * complete_dev_evaluations
    expected_best_checkpoint_projected = (
        maximum_checkpoint_save_seconds * MAXIMUM_BEST_CHECKPOINT_SAVES
    )
    expected_projected = (
        expected_optimizer_projected
        + expected_development_projected
        + expected_best_checkpoint_projected
        + checkpoint_reload_seconds
    )
    expected_memory_fraction = peak / available
    expected_time_passed = safety_projected <= 8 * 60 * 60
    expected_memory_passed = memory_fraction <= 0.80
    expected_passed = expected_time_passed and expected_memory_passed
    expected_terminal = (
        "PASS_HARDWARE_ADMISSION"
        if expected_passed
        else "UNAVAILABLE_HARDWARE_BUDGET"
    )
    probe_contract = result.get("probe_contract")
    if not isinstance(probe_contract, dict):
        raise ValueError("#1019 hardware probe contract is missing")
    recorded_environment = probe_contract.get("environment")
    _validate_runtime_environment_envelope(recorded_environment, backend_identity)
    environment_matches_current = (
        not require_current_environment
        or recorded_environment
        == _runtime_environment_identity(backend_identity, backend)
    )
    mps_prerequisite = result.get("mps_failure_prerequisite")
    if probe_contract.get("mps_failure_prerequisite") != mps_prerequisite:
        raise ValueError("#1019 hardware prerequisite identities differ")
    if mps_prerequisite is not None:
        raise ValueError("#1019 MPS probe cannot have an MPS-failure prerequisite")
    probe_contract_cid = cid_bytes(canonical_json_bytes(probe_contract))
    checkpoint_path = root / hardware_checkpoint_relative_path(backend)
    checkpoint_manifest_path = checkpoint_path.with_suffix(
        checkpoint_path.suffix + ".manifest.json"
    )
    checkpoint_manifest = json.loads(checkpoint_manifest_path.read_text(encoding="utf-8"))
    if not isinstance(checkpoint_manifest, dict):
        raise ValueError("#1019 probe checkpoint manifest must be a JSON object")
    _verify_signed(checkpoint_manifest, label="#1019 probe checkpoint manifest")
    elapsed_sample_path = root / hardware_elapsed_sample_relative_path(backend)
    elapsed_sample = json.loads(elapsed_sample_path.read_text(encoding="utf-8"))
    if not isinstance(elapsed_sample, dict):
        raise ValueError("#1019 probe elapsed sample must be a JSON object")
    _verify_signed(elapsed_sample, label="#1019 probe elapsed sample")
    if (
        result.get("schema") != CAPACITY_HARDWARE_SCHEMA
        or result.get("issue") != ISSUE
        or result.get("terminal") != expected_terminal
        or result.get("main_run_authorized") != expected_passed
        or result.get("backend", {}).get("backend") != backend
        or result.get("dataset_manifest_cid") != training_view["dataset_manifest_cid"]
        or result.get("training_view_manifest_cid") != training_view["manifest_cid"]
        or result.get("smoke_admission_manifest_cid") != smoke_admission["manifest_cid"]
        or result.get("probe_optimizer_steps") != 200
        or result.get("probe_token_presentations") != 200 * TOKENS_PER_OPTIMIZER_STEP
        or result.get("checkpoint_interval") != 100
        or result.get("probe_contract_cid") != probe_contract_cid
        or result.get("probe_checkpoint_path")
        != str(hardware_checkpoint_relative_path(backend))
        or result.get("probe_checkpoint_cid") != cid_file(checkpoint_path)
        or result.get("probe_checkpoint_manifest_path")
        != str(hardware_checkpoint_relative_path(backend)) + ".manifest.json"
        or result.get("probe_checkpoint_manifest_cid")
        != cid_file(checkpoint_manifest_path)
        or checkpoint_manifest.get("schema") != CAPACITY_CHECKPOINT_MANIFEST_SCHEMA
        or checkpoint_manifest.get("issue") != ISSUE
        or checkpoint_manifest.get("checkpoint_filename") != checkpoint_path.name
        or checkpoint_manifest.get("checkpoint_cid") != cid_file(checkpoint_path)
        or checkpoint_manifest.get("run_contract_cid") != probe_contract_cid
        or checkpoint_manifest.get("optimizer_step") != 200
        or checkpoint_manifest.get("tokens_seen")
        != 200 * TOKENS_PER_OPTIMIZER_STEP
        or checkpoint_manifest.get("backend") != backend
        or result.get("elapsed_sample_path")
        != str(hardware_elapsed_sample_relative_path(backend))
        or result.get("elapsed_sample_cid") != cid_file(elapsed_sample_path)
        or elapsed_sample.get("schema")
        != "uor-r4-softmax-trainer-capacity-hardware-elapsed-sample/1"
        or elapsed_sample.get("issue") != ISSUE
        or elapsed_sample.get("backend") != backend
        or elapsed_sample.get("probe_contract_cid") != probe_contract_cid
        or elapsed_sample.get("optimizer_step") != 200
        or elapsed_sample.get("train_tokens")
        != 200 * TOKENS_PER_OPTIMIZER_STEP
        or _finite_float(
            elapsed_sample.get("elapsed_seconds"),
            label="#1019 probe elapsed sample time",
        )
        > optimizer_loop_seconds
        or probe_contract.get("schema")
        != "uor-r4-softmax-trainer-capacity-hardware-probe-contract/1"
        or probe_contract.get("issue") != ISSUE
        or probe_contract.get("dataset_manifest_cid")
        != training_view["dataset_manifest_cid"]
        or probe_contract.get("training_view_manifest_cid")
        != training_view["manifest_cid"]
        or probe_contract.get("smoke_admission_manifest_cid")
        != smoke_admission["manifest_cid"]
        or probe_contract.get("backend") != result.get("backend")
        or not environment_matches_current
        or probe_contract.get("model") != CAPACITY_MODEL_CONFIG.as_contract()
        or probe_contract.get("parameter_count") != PARAMETER_COUNT
        or probe_contract.get("optimizer_steps") != 200
        or probe_contract.get("tokens_per_optimizer_step")
        != TOKENS_PER_OPTIMIZER_STEP
        or probe_contract.get("checkpoint_interval") != 100
        or probe_contract.get("maximum_best_checkpoint_saves")
        != MAXIMUM_BEST_CHECKPOINT_SAVES
        or result.get("checkpoint_reload_passed") is not True
        or elapsed <= 0
        or optimizer_loop_seconds <= 0
        or checkpoint_reload_seconds <= 0
        or dev_seconds <= 0
        or probe_dev_loss < 0
        or not math.isclose(
            elapsed,
            optimizer_loop_seconds + checkpoint_reload_seconds + dev_seconds,
            rel_tol=1e-12,
            abs_tol=1e-9,
        )
        or not math.isclose(
            _finite_float(
                result.get("optimizer_steps_per_second"),
                label="#1019 probe optimizer rate",
            ),
            200 / optimizer_loop_seconds,
            rel_tol=1e-12,
            abs_tol=1e-12,
        )
        or result.get("projected_complete_dev_evaluations")
        != complete_dev_evaluations
        or not math.isclose(
            _finite_float(
                result.get("projected_optimizer_and_checkpoint_seconds"),
                label="#1019 projected optimizer time",
            ),
            expected_optimizer_projected,
            rel_tol=1e-12,
            abs_tol=1e-9,
        )
        or not math.isclose(
            _finite_float(
                result.get("projected_development_evaluation_seconds"),
                label="#1019 projected development time",
            ),
            expected_development_projected,
            rel_tol=1e-12,
            abs_tol=1e-9,
        )
        or not math.isclose(
            _finite_float(
                result.get("maximum_measured_checkpoint_save_seconds"),
                label="#1019 maximum checkpoint-save time",
            ),
            maximum_checkpoint_save_seconds,
            rel_tol=1e-12,
            abs_tol=1e-12,
        )
        or result.get("projected_best_checkpoint_saves")
        != MAXIMUM_BEST_CHECKPOINT_SAVES
        or not math.isclose(
            _finite_float(
                result.get("projected_best_checkpoint_seconds"),
                label="#1019 projected best-checkpoint time",
            ),
            expected_best_checkpoint_projected,
            rel_tol=1e-12,
            abs_tol=1e-9,
        )
        or not math.isclose(projected, expected_projected, rel_tol=1e-12, abs_tol=1e-9)
        or not math.isclose(
            safety_projected, expected_projected * 1.25, rel_tol=1e-12, abs_tol=1e-9
        )
        or not math.isclose(
            memory_fraction, expected_memory_fraction, rel_tol=1e-12, abs_tol=1e-12
        )
        or result.get("safety_factor") != 1.25
        or result.get("maximum_safety_projected_seconds") != 8 * 60 * 60
        or result.get("maximum_memory_fraction") != 0.80
        or result.get("time_passed") != expected_time_passed
        or result.get("memory_passed") != expected_memory_passed
        or result.get("partial_checkpoint_interpretable") is not False
    ):
        raise ValueError("#1019 hardware admission does not authorize this backend")
    recorded_tree = result.get("trainer_implementation")
    if recorded_tree != probe_contract.get("trainer_implementation"):
        raise ValueError("#1019 hardware trainer identities differ")
    if require_current_environment and recorded_tree != trainer_implementation_contract():
        raise ValueError("trainer changed after #1019 hardware admission")
    if require_pass and not expected_passed:
        raise ValueError("#1019 hardware measurement does not authorize training")
    return result


def load_capacity_hardware_admission(
    root: Path, *, backend: Backend, require_pass: bool = True
) -> dict[str, Any]:
    training_view = load_capacity_training_view_manifest(root)
    smoke_admission = load_capacity_smoke_admission(root)
    result = json.loads(
        (root / hardware_result_relative_path(backend)).read_text(encoding="utf-8")
    )
    if not isinstance(result, dict):
        raise ValueError("#1019 hardware probe must be a JSON object")
    return _validate_capacity_hardware_evidence(
        root,
        backend=backend,
        training_view=training_view,
        smoke_admission=smoke_admission,
        result=result,
        require_pass=require_pass,
        require_current_environment=True,
    )


def build_capacity_run_contract(
    training_view: dict[str, Any],
    hardware: dict[str, Any],
    config: CapacityTrainConfig,
) -> dict[str, Any]:
    config.validate()
    backend_identity = hardware.get("backend")
    if not isinstance(backend_identity, dict):
        raise ValueError("#1019 hardware admission has no backend identity")
    backend = backend_identity.get("backend")
    if backend != "mps":
        raise ValueError("#1019 hardware admission backend differs")
    _validate_backend_identity(backend_identity, backend)
    probe_contract = hardware.get("probe_contract")
    if not isinstance(probe_contract, dict):
        raise ValueError("#1019 hardware admission has no probe contract")
    environment = probe_contract.get("environment")
    _validate_runtime_environment_envelope(environment, backend_identity)
    trainer_implementation = hardware.get("trainer_implementation")
    if (
        probe_contract.get("backend") != backend_identity
        or probe_contract.get("trainer_implementation") != trainer_implementation
    ):
        raise ValueError("#1019 hardware admission contract identities differ")
    return {
        "schema": CAPACITY_RUN_SCHEMA,
        "issue": ISSUE,
        "architecture_reference": {
            "repository": LLAMA2_C_REPOSITORY,
            "revision": LLAMA2_C_REVISION,
            "license": "MIT",
        },
        "dataset_manifest_cid": training_view["dataset_manifest_cid"],
        "training_view_manifest_cid": training_view["manifest_cid"],
        "split_policy_cid": training_view["split_policy_cid"],
        "hardware_admission_result_cid": hardware["result_cid"],
        "environment": environment,
        "trainer_implementation": trainer_implementation,
        "model": CAPACITY_MODEL_CONFIG.as_contract(),
        "parameter_count": PARAMETER_COUNT,
        "optimization": config.as_contract(),
        "selection": "minimum complete-development mean causal cross-entropy",
        "confirmation_policy": "unavailable until selection and enabled Rust parity freeze",
        "attention_off_executions": 0,
    }


def _append_progress(root: Path, value: dict[str, Any]) -> None:
    path = root / PROGRESS_RELATIVE_PATH
    path.parent.mkdir(parents=True, exist_ok=True)
    payload = json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":"))
    with path.open("a", encoding="utf-8") as handle:
        handle.write(payload + "\n")
        handle.flush()
        os.fsync(handle.fileno())


def _write_elapsed_ledger(
    root: Path,
    *,
    run_contract_cid: str,
    backend: Backend,
    optimizer_step: int,
    campaign_started_unix_seconds: float,
    elapsed_seconds: float,
) -> dict[str, Any]:
    recorded = time.time()
    wall_elapsed = recorded - campaign_started_unix_seconds
    if wall_elapsed < 0:
        raise RuntimeError("#1019 wall clock moved before the campaign start")
    effective_elapsed = max(elapsed_seconds, wall_elapsed)
    return _write_signed(
        root / ELAPSED_LEDGER_RELATIVE_PATH,
        {
            "schema": CAPACITY_ELAPSED_LEDGER_SCHEMA,
            "issue": ISSUE,
            "run_contract_cid": run_contract_cid,
            "backend": backend,
            "optimizer_step": optimizer_step,
            "train_tokens": optimizer_step * TOKENS_PER_OPTIMIZER_STEP,
            "campaign_started_unix_seconds": campaign_started_unix_seconds,
            "recorded_unix_seconds": recorded,
            "elapsed_seconds": effective_elapsed,
            "wall_ceiling_seconds": CapacityTrainConfig().wall_ceiling_seconds,
        },
    )


def _load_elapsed_ledger(
    root: Path, *, run_contract_cid: str, backend: Backend
) -> dict[str, Any]:
    value = json.loads((root / ELAPSED_LEDGER_RELATIVE_PATH).read_text(encoding="utf-8"))
    if not isinstance(value, dict):
        raise ValueError("#1019 elapsed ledger must be a JSON object")
    _verify_signed(value, label="#1019 elapsed ledger")
    optimizer_step = value.get("optimizer_step")
    started = _finite_float(
        value.get("campaign_started_unix_seconds"), label="#1019 campaign start"
    )
    recorded = _finite_float(
        value.get("recorded_unix_seconds"), label="#1019 ledger wall time"
    )
    elapsed = _finite_float(value.get("elapsed_seconds"), label="#1019 elapsed time")
    if (
        value.get("schema") != CAPACITY_ELAPSED_LEDGER_SCHEMA
        or value.get("issue") != ISSUE
        or value.get("run_contract_cid") != run_contract_cid
        or value.get("backend") != backend
        or isinstance(optimizer_step, bool)
        or not isinstance(optimizer_step, int)
        or not 0 <= optimizer_step <= OPTIMIZER_STEPS
        or value.get("train_tokens") != optimizer_step * TOKENS_PER_OPTIMIZER_STEP
        or started <= 0
        or recorded < started
        or elapsed < recorded - started
        or value.get("wall_ceiling_seconds")
        != CapacityTrainConfig().wall_ceiling_seconds
    ):
        raise ValueError("#1019 elapsed ledger identity differs")
    return value


def _write_training_unavailable(
    root: Path,
    *,
    run_contract_cid: str,
    optimizer_step: int,
    elapsed_seconds: float,
    wall_ceiling_seconds: float,
) -> dict[str, Any]:
    return _write_signed(
        root / TRAINING_STATUS_RELATIVE_PATH,
        {
            "schema": "uor-r4-softmax-trainer-capacity-unavailable/1",
            "issue": ISSUE,
            "terminal": "UNAVAILABLE_HARDWARE_BUDGET",
            "run_contract_cid": run_contract_cid,
            "optimizer_step": optimizer_step,
            "elapsed_seconds": elapsed_seconds,
            "wall_ceiling_seconds": wall_ceiling_seconds,
            "partial_checkpoint_interpretable": False,
            "resume_permitted": False,
        },
    )


def train_capacity(
    root: Path,
    *,
    backend: Backend,
    config: CapacityTrainConfig = CapacityTrainConfig(),
    resume: bool = False,
) -> dict[str, Any]:
    """Run or resume the sole #1019 campaign after all cheap gates pass."""
    config.validate()
    status_path = root / TRAINING_STATUS_RELATIVE_PATH
    if status_path.exists():
        status = json.loads(status_path.read_text(encoding="utf-8"))
        if not isinstance(status, dict):
            raise ValueError("#1019 terminal training status is not a JSON object")
        _verify_signed(status, label="#1019 terminal training status")
        if (
            status.get("schema")
            != "uor-r4-softmax-trainer-capacity-unavailable/1"
            or status.get("issue") != ISSUE
            or status.get("terminal") != "UNAVAILABLE_HARDWARE_BUDGET"
            or status.get("partial_checkpoint_interpretable") is not False
            or status.get("resume_permitted") is not False
        ):
            raise ValueError("#1019 terminal training status differs")
        raise RuntimeError("UNAVAILABLE_HARDWARE_BUDGET: terminal; resume is prohibited")
    if (root / SELECTION_RELATIVE_PATH).exists():
        load_frozen_capacity_selection(root)
        raise FileExistsError("#1019 selection is frozen")
    if (root / REVEAL_OPENED_RELATIVE_PATH).exists():
        raise FileExistsError("#1019 confirmation was already opened")
    training_view = load_capacity_training_view_manifest(root)
    hardware = load_capacity_hardware_admission(root, backend=backend)
    run_contract = build_capacity_run_contract(training_view, hardware, config)
    run_contract_cid = cid_bytes(canonical_json_bytes(run_contract))
    checkpoint_dir = root / "checkpoints"
    latest_path = checkpoint_dir / "latest.pt"
    best_path = checkpoint_dir / "best.pt"
    checkpoint_artifacts = [
        latest_path,
        latest_path.with_suffix(latest_path.suffix + ".manifest.json"),
        best_path,
        best_path.with_suffix(best_path.suffix + ".manifest.json"),
        root / ELAPSED_LEDGER_RELATIVE_PATH,
    ]
    if not resume and any(path.exists() for path in checkpoint_artifacts):
        raise FileExistsError("#1019 checkpoints exist; use --resume for the same run")
    device, identity = require_backend(config.seed, backend)
    if identity != hardware["backend"]:
        raise ValueError("#1019 runtime accelerator differs from hardware admission")
    train_store = TokenStore(root / TOKEN_RELATIVE_PATHS["train"])
    dev_store = TokenStore(root / TOKEN_RELATIVE_PATHS["dev"])
    if len(train_store.tokens) != TRAIN_TOKEN_CAP or len(dev_store.tokens) != DEV_TOKEN_CAP:
        raise ValueError("#1019 token stores have unexpected exact lengths")
    model = R4SoftmaxForCausalLM(CAPACITY_MODEL_CONFIG).to(device)
    optimizer = torch.optim.AdamW(
        model.parameters(),
        lr=0.0,
        betas=(config.adam_beta1, config.adam_beta2),
        eps=config.adam_epsilon,
        weight_decay=config.weight_decay,
    )
    optimizer_step = 0
    elapsed_before = 0.0
    best_dev_loss = math.inf
    candidates: list[dict[str, Any]] = []
    campaign_started_unix_seconds = time.time()
    started = time.monotonic()
    highest_optimizer_step_attempted = 0
    if resume:
        resume_path = latest_path if latest_path.is_file() else best_path
        if not resume_path.is_file():
            raise FileNotFoundError("--resume requested without a #1019 checkpoint")
        checkpoint = _load_checkpoint(
            resume_path,
            model=model,
            optimizer=optimizer,
            device=device,
            backend=backend,
            run_contract_cid=run_contract_cid,
        )
        optimizer_step = int(checkpoint["optimizer_step"])
        best_dev_loss = float(checkpoint["best_dev_loss"])
        candidates = list(checkpoint["development_candidates"])
        ledger = _load_elapsed_ledger(
            root, run_contract_cid=run_contract_cid, backend=backend
        )
        if (
            int(ledger["optimizer_step"]) < optimizer_step
            or float(ledger["elapsed_seconds"]) < float(checkpoint["elapsed_seconds"])
            or time.time() < float(ledger["recorded_unix_seconds"])
        ):
            raise ValueError("#1019 resume ledger regressed behind its checkpoint")
        campaign_started_unix_seconds = float(
            ledger["campaign_started_unix_seconds"]
        )
        highest_optimizer_step_attempted = int(ledger["optimizer_step"])
        elapsed_before = max(
            float(ledger["elapsed_seconds"]),
            time.time() - campaign_started_unix_seconds,
        )
        started = time.monotonic()
        if elapsed_before >= config.wall_ceiling_seconds:
            _write_training_unavailable(
                root,
                run_contract_cid=run_contract_cid,
                optimizer_step=optimizer_step,
                elapsed_seconds=elapsed_before,
                wall_ceiling_seconds=config.wall_ceiling_seconds,
            )
            raise RuntimeError("UNAVAILABLE_HARDWARE_BUDGET: terminal; resume is prohibited")
    else:
        _write_elapsed_ledger(
            root,
            run_contract_cid=run_contract_cid,
            backend=backend,
            optimizer_step=0,
            campaign_started_unix_seconds=campaign_started_unix_seconds,
            elapsed_seconds=0.0,
        )
        initial_dev = evaluate(model, dev_store, device, config.batch_size)
        best_dev_loss = initial_dev
        candidates = [{"optimizer_step": 0, "train_tokens": 0, "development_loss": initial_dev}]
        initial_elapsed = time.monotonic() - started
        ledger = _write_elapsed_ledger(
            root,
            run_contract_cid=run_contract_cid,
            backend=backend,
            optimizer_step=0,
            campaign_started_unix_seconds=campaign_started_unix_seconds,
            elapsed_seconds=initial_elapsed,
        )
        _save_checkpoint(
            best_path,
            model=model,
            optimizer=optimizer,
            optimizer_step=0,
            elapsed_seconds=float(ledger["elapsed_seconds"]),
            best_dev_loss=best_dev_loss,
            development_candidates=candidates,
            run_contract=run_contract,
            run_contract_cid=run_contract_cid,
            backend=backend,
        )
        elapsed_before = float(ledger["elapsed_seconds"])
        if elapsed_before >= config.wall_ceiling_seconds:
            _write_training_unavailable(
                root,
                run_contract_cid=run_contract_cid,
                optimizer_step=0,
                elapsed_seconds=elapsed_before,
                wall_ceiling_seconds=config.wall_ceiling_seconds,
            )
            raise RuntimeError("UNAVAILABLE_HARDWARE_BUDGET")
        started = time.monotonic()
    for step in range(optimizer_step + 1, config.optimizer_steps + 1):
        model.train()
        optimizer.zero_grad(set_to_none=True)
        microbatch_losses: list[Tensor] = []
        for accumulation in range(config.gradient_accumulation_steps):
            inputs, targets = train_store.random_batch(
                seed=config.seed,
                batch_index=(step - 1) * config.gradient_accumulation_steps + accumulation,
                batch_size=config.batch_size,
            )
            output = model(inputs.to(device), targets.to(device))
            assert output.loss is not None
            (output.loss / config.gradient_accumulation_steps).backward()
            microbatch_losses.append(output.loss.detach())
        torch.nn.utils.clip_grad_norm_(model.parameters(), config.gradient_clip)
        learning_rate = capacity_learning_rate(step, config)
        for group in optimizer.param_groups:
            group["lr"] = learning_rate
        optimizer.step()
        optimizer_step = step
        highest_optimizer_step_attempted = max(highest_optimizer_step_attempted, step)
        should_evaluate = (
            step % config.evaluation_interval == 0 or step == config.optimizer_steps
        )
        should_checkpoint = (
            step % config.checkpoint_interval == 0 or step == config.optimizer_steps
        )
        should_report = step % config.progress_interval == 0 or step == config.optimizer_steps
        if should_evaluate:
            dev_loss = evaluate(model, dev_store, device, config.batch_size)
            candidates.append(
                {
                    "optimizer_step": step,
                    "train_tokens": step * config.tokens_per_optimizer_step,
                    "development_loss": dev_loss,
                }
            )
            if dev_loss < best_dev_loss:
                best_dev_loss = dev_loss
                _save_checkpoint(
                    best_path,
                    model=model,
                    optimizer=optimizer,
                    optimizer_step=step,
                    elapsed_seconds=elapsed_before + (time.monotonic() - started),
                    best_dev_loss=best_dev_loss,
                    development_candidates=candidates,
                    run_contract=run_contract,
                    run_contract_cid=run_contract_cid,
                    backend=backend,
                )
        if should_checkpoint:
            _save_checkpoint(
                latest_path,
                model=model,
                optimizer=optimizer,
                optimizer_step=step,
                elapsed_seconds=elapsed_before + (time.monotonic() - started),
                best_dev_loss=best_dev_loss,
                development_candidates=candidates,
                run_contract=run_contract,
                run_contract_cid=run_contract_cid,
                backend=backend,
            )
        elapsed = max(
            elapsed_before + (time.monotonic() - started),
            time.time() - campaign_started_unix_seconds,
        )
        if should_report or should_checkpoint or should_evaluate:
            ledger = _write_elapsed_ledger(
                root,
                run_contract_cid=run_contract_cid,
                backend=backend,
                optimizer_step=highest_optimizer_step_attempted,
                campaign_started_unix_seconds=campaign_started_unix_seconds,
                elapsed_seconds=elapsed,
            )
            elapsed = float(ledger["elapsed_seconds"])
        if elapsed >= config.wall_ceiling_seconds:
            _save_checkpoint(
                latest_path,
                model=model,
                optimizer=optimizer,
                optimizer_step=step,
                elapsed_seconds=elapsed,
                best_dev_loss=best_dev_loss,
                development_candidates=candidates,
                run_contract=run_contract,
                run_contract_cid=run_contract_cid,
                backend=backend,
            )
            _write_elapsed_ledger(
                root,
                run_contract_cid=run_contract_cid,
                backend=backend,
                optimizer_step=highest_optimizer_step_attempted,
                campaign_started_unix_seconds=campaign_started_unix_seconds,
                elapsed_seconds=elapsed,
            )
            _write_training_unavailable(
                root,
                run_contract_cid=run_contract_cid,
                optimizer_step=step,
                elapsed_seconds=elapsed,
                wall_ceiling_seconds=config.wall_ceiling_seconds,
            )
            raise RuntimeError("UNAVAILABLE_HARDWARE_BUDGET")
        if should_report:
            rate = step / elapsed if elapsed > 0 else 0.0
            eta = (config.optimizer_steps - step) / rate if rate > 0 else None
            mean_microbatch_loss = float(torch.stack(microbatch_losses).mean().cpu())
            progress = {
                "schema": "uor-r4-softmax-trainer-capacity-progress/1",
                "issue": ISSUE,
                "optimizer_step": step,
                "optimizer_steps_total": config.optimizer_steps,
                "optimizer_steps_remaining": config.optimizer_steps - step,
                "train_tokens": step * config.tokens_per_optimizer_step,
                "train_tokens_total": config.train_tokens,
                "mean_microbatch_loss": mean_microbatch_loss,
                "best_dev_loss": best_dev_loss,
                "learning_rate": learning_rate,
                "elapsed_seconds": elapsed,
                "optimizer_steps_per_second": rate,
                "eta_seconds": eta,
            }
            _append_progress(root, progress)
            print(json.dumps(progress, sort_keys=True), flush=True)
    _sync(backend)
    selected = _load_checkpoint(
        best_path,
        model=model,
        optimizer=None,
        device=device,
        backend=backend,
        run_contract_cid=run_contract_cid,
    )
    selected_step = int(selected["optimizer_step"])
    selected_dev = evaluate(model, dev_store, device, config.batch_size)
    _sync(backend)
    ledger = _write_elapsed_ledger(
        root,
        run_contract_cid=run_contract_cid,
        backend=backend,
        optimizer_step=OPTIMIZER_STEPS,
        campaign_started_unix_seconds=campaign_started_unix_seconds,
        elapsed_seconds=elapsed_before + (time.monotonic() - started),
    )
    elapsed = float(ledger["elapsed_seconds"])
    if elapsed >= config.wall_ceiling_seconds:
        _write_training_unavailable(
            root,
            run_contract_cid=run_contract_cid,
            optimizer_step=optimizer_step,
            elapsed_seconds=elapsed,
            wall_ceiling_seconds=config.wall_ceiling_seconds,
        )
        raise RuntimeError("UNAVAILABLE_HARDWARE_BUDGET")
    selected_checkpoint_cid = cid_file(best_path)
    result = _write_signed(
        root / TRAINING_RESULT_RELATIVE_PATH,
        {
            "schema": CAPACITY_TRAINING_RESULT_SCHEMA,
            "issue": ISSUE,
            "terminal": "FINAL_CHECKPOINT_FROZEN_CONFIRMATION_UNOPENED",
            "dataset_manifest_cid": training_view["dataset_manifest_cid"],
            "training_view_manifest_cid": training_view["manifest_cid"],
            "run_contract": run_contract,
            "run_contract_cid": run_contract_cid,
            "optimizer_steps_completed": optimizer_step,
            "train_tokens_seen": optimizer_step * config.tokens_per_optimizer_step,
            "selected_checkpoint_step": selected_step,
            "selected_dev_loss": selected_dev,
            "development_selection_candidates": candidates,
            "elapsed_training_seconds": elapsed,
            "elapsed_ledger_result_cid": ledger["result_cid"],
            "sealed_confirmation_status": "UNOPENED",
            "attention_off_executions": 0,
        },
    )
    export = export_hugging_face_snapshot(
        model,
        output_dir=root / "export",
        tokenizer_path=root / TOKENIZER_RELATIVE_PATH,
        training_result=result,
        dataset_manifest_cid=str(training_view["dataset_manifest_cid"]),
        training_view_manifest_cid=str(training_view["manifest_cid"]),
        split_policy_cid=str(training_view["split_policy_cid"]),
        run_contract_cid=run_contract_cid,
        selected_checkpoint_cid=selected_checkpoint_cid,
    )
    prefix = _write_python_prefix(
        root / PYTHON_PREFIX_RELATIVE_PATH,
        model=model,
        store=dev_store,
        store_path=root / TOKEN_RELATIVE_PATHS["dev"],
        device=device,
        weights_cid=str(export["weights_cid"]),
    )
    return write_bound_manifest(
        root / SELECTION_RELATIVE_PATH,
        {
            "schema": CAPACITY_SELECTION_SCHEMA,
            "issue": ISSUE,
            "dataset_manifest_cid": training_view["dataset_manifest_cid"],
            "training_view_manifest_cid": training_view["manifest_cid"],
            "split_policy_cid": training_view["split_policy_cid"],
            "run_contract_cid": run_contract_cid,
            "hardware_admission_result_cid": hardware["result_cid"],
            "hardware_admission_path": str(hardware_result_relative_path(backend)),
            "mps_failure_prerequisite_result_cid": None,
            "selected_checkpoint_cid": selected_checkpoint_cid,
            "selected_checkpoint_step": selected_step,
            "selected_dev_loss": selected_dev,
            "export_manifest_cid": export["manifest_cid"],
            "weights_cid": export["weights_cid"],
            "tokenizer_cid": export["tokenizer_cid"],
            "training_result_cid": result["result_cid"],
            "elapsed_ledger_result_cid": ledger["result_cid"],
            "python_prefix_result_cid": prefix["result_cid"],
            "sealed_confirmation_status": "UNOPENED_BEFORE_THIS_MANIFEST",
            "attention_off_executions": 0,
        },
        artifact_root=root,
        relative_paths=[
            *hardware_evidence_relative_paths(backend),
            "checkpoints/best.pt",
            "checkpoints/best.pt.manifest.json",
            str(ELAPSED_LEDGER_RELATIVE_PATH),
            str(TRAINING_RESULT_RELATIVE_PATH),
            "export/config.json",
            "export/model.safetensors",
            "export/tokenizer.json",
            "export/training-result.json",
            "export/export-manifest.json",
            str(PYTHON_PREFIX_RELATIVE_PATH),
        ],
    )


def load_frozen_capacity_selection(root: Path) -> dict[str, Any]:
    selection = verify_bound_manifest(root / SELECTION_RELATIVE_PATH, artifact_root=root)
    hardware_relative = selection.get("hardware_admission_path")
    expected_hardware_path = str(hardware_result_relative_path("mps"))
    if hardware_relative != expected_hardware_path:
        raise ValueError("#1019 selection has an invalid hardware path")
    hardware_backend: Backend = "mps"
    expected_selection_paths = {
        *hardware_evidence_relative_paths(hardware_backend),
        "checkpoints/best.pt",
        "checkpoints/best.pt.manifest.json",
        str(ELAPSED_LEDGER_RELATIVE_PATH),
        str(TRAINING_RESULT_RELATIVE_PATH),
        "export/config.json",
        "export/model.safetensors",
        "export/tokenizer.json",
        "export/training-result.json",
        "export/export-manifest.json",
        str(PYTHON_PREFIX_RELATIVE_PATH),
    }
    if (
        selection.get("schema") != CAPACITY_SELECTION_SCHEMA
        or selection.get("issue") != ISSUE
        or selection.get("sealed_confirmation_status") != "UNOPENED_BEFORE_THIS_MANIFEST"
        or selection.get("attention_off_executions") != 0
        or _manifest_paths(selection, label="#1019 selection")
        != expected_selection_paths
    ):
        raise ValueError("#1019 selection identity differs")
    if selection.get("selected_checkpoint_cid") != cid_file(root / "checkpoints/best.pt"):
        raise ValueError("#1019 selected checkpoint CID differs")
    result = json.loads((root / TRAINING_RESULT_RELATIVE_PATH).read_text(encoding="utf-8"))
    if not isinstance(result, dict):
        raise ValueError("#1019 training result must be a JSON object")
    _verify_signed(result, label="#1019 training")
    run_contract = result.get("run_contract")
    if not isinstance(run_contract, dict):
        raise ValueError("#1019 training result has no run contract")
    run_contract_cid = cid_bytes(canonical_json_bytes(run_contract))
    training_view = verify_manifest_envelope(
        root / CAPACITY_TRAINING_VIEW_MANIFEST_NAME
    )
    dataset = verify_manifest_envelope(root / CAPACITY_DATASET_MANIFEST_NAME)
    _validate_dataset_envelope(dataset)
    _validate_training_view_envelope(training_view, dataset)
    hardware = json.loads((root / str(hardware_relative)).read_text(encoding="utf-8"))
    if not isinstance(hardware, dict):
        raise ValueError("#1019 selected hardware result must be a JSON object")
    _verify_signed(hardware, label="#1019 selected hardware probe")
    backend = hardware.get("backend", {}).get("backend")
    if backend != "mps":
        raise ValueError("#1019 selected hardware backend differs")
    smoke_admission = load_capacity_smoke_admission(root)
    _validate_capacity_hardware_evidence(
        root,
        backend=hardware_backend,
        training_view=training_view,
        smoke_admission=smoke_admission,
        result=hardware,
        require_pass=True,
        require_current_environment=False,
    )
    expected_mps_prerequisite_cid = None
    expected_run_contract = build_capacity_run_contract(
        training_view, hardware, CapacityTrainConfig()
    )
    if (
        result.get("schema") != CAPACITY_TRAINING_RESULT_SCHEMA
        or result.get("issue") != ISSUE
        or result.get("terminal")
        != "FINAL_CHECKPOINT_FROZEN_CONFIRMATION_UNOPENED"
        or result.get("optimizer_steps_completed") != OPTIMIZER_STEPS
        or result.get("train_tokens_seen") != TRAIN_TOKENS
        or result.get("sealed_confirmation_status") != "UNOPENED"
        or result.get("attention_off_executions") != 0
        or result.get("run_contract_cid") != run_contract_cid
        or selection.get("run_contract_cid") != run_contract_cid
        or run_contract != expected_run_contract
        or result.get("dataset_manifest_cid") != dataset.get("manifest_cid")
        or result.get("dataset_manifest_cid")
        != training_view.get("dataset_manifest_cid")
        or result.get("training_view_manifest_cid")
        != training_view.get("manifest_cid")
        or selection.get("dataset_manifest_cid") != result.get("dataset_manifest_cid")
        or selection.get("training_view_manifest_cid")
        != result.get("training_view_manifest_cid")
        or selection.get("split_policy_cid") != training_view.get("split_policy_cid")
        or selection.get("hardware_admission_result_cid") != hardware.get("result_cid")
        or selection.get("mps_failure_prerequisite_result_cid")
        != expected_mps_prerequisite_cid
        or hardware.get("terminal") != "PASS_HARDWARE_ADMISSION"
        or hardware.get("main_run_authorized") is not True
        or hardware.get("time_passed") is not True
        or hardware.get("memory_passed") is not True
        or hardware.get("safety_projected_training_seconds", math.inf)
        > CapacityTrainConfig().wall_ceiling_seconds
        or hardware.get("peak_memory_fraction", math.inf) > 0.80
    ):
        raise ValueError("#1019 training did not complete its frozen budget")
    candidates = _validate_development_candidates(
        result.get("development_selection_candidates"),
        optimizer_step=OPTIMIZER_STEPS,
    )
    selected_candidate = min(
        candidates,
        key=lambda candidate: (
            float(candidate["development_loss"]),
            int(candidate["optimizer_step"]),
        ),
    )
    selected_step = result.get("selected_checkpoint_step")
    selected_dev = _finite_float(
        result.get("selected_dev_loss"), label="#1019 selected dev loss"
    )
    elapsed = _finite_float(
        result.get("elapsed_training_seconds"), label="#1019 training elapsed"
    )
    if (
        selected_step != selected_candidate["optimizer_step"]
        or selection.get("selected_checkpoint_step") != selected_step
        or not math.isclose(
            selected_dev,
            float(selected_candidate["development_loss"]),
            rel_tol=0.0,
            abs_tol=1e-9,
        )
        or selection.get("selected_dev_loss") != result.get("selected_dev_loss")
        or not 0 < elapsed < CapacityTrainConfig().wall_ceiling_seconds
    ):
        raise ValueError("#1019 development-minimum selection differs")
    ledger = _load_elapsed_ledger(root, run_contract_cid=run_contract_cid, backend=backend)
    if (
        ledger.get("optimizer_step") != OPTIMIZER_STEPS
        or ledger.get("result_cid") != result.get("elapsed_ledger_result_cid")
        or ledger.get("result_cid") != selection.get("elapsed_ledger_result_cid")
        or float(ledger["elapsed_seconds"]) != elapsed
    ):
        raise ValueError("#1019 elapsed ledger differs from the frozen selection")
    checkpoint_manifest_path = root / "checkpoints/best.pt.manifest.json"
    checkpoint_manifest = json.loads(checkpoint_manifest_path.read_text(encoding="utf-8"))
    if not isinstance(checkpoint_manifest, dict):
        raise ValueError("#1019 best-checkpoint manifest must be a JSON object")
    _verify_signed(checkpoint_manifest, label="#1019 best-checkpoint manifest")
    if (
        checkpoint_manifest.get("schema") != CAPACITY_CHECKPOINT_MANIFEST_SCHEMA
        or checkpoint_manifest.get("checkpoint_filename") != "best.pt"
        or checkpoint_manifest.get("checkpoint_cid")
        != selection.get("selected_checkpoint_cid")
        or checkpoint_manifest.get("run_contract_cid") != run_contract_cid
        or checkpoint_manifest.get("optimizer_step") != selected_step
        or checkpoint_manifest.get("backend") != backend
    ):
        raise ValueError("#1019 best-checkpoint manifest differs")
    export = verify_bound_manifest(root / "export/export-manifest.json", artifact_root=root / "export")
    exported_result = json.loads(
        (root / "export/training-result.json").read_text(encoding="utf-8")
    )
    if (
        export.get("schema") != EXPORT_MANIFEST_SCHEMA
        or _manifest_paths(export, label="#1019 export")
        != {
            "config.json",
            "model.safetensors",
            "tokenizer.json",
            "training-result.json",
        }
        or exported_result != result
        or export.get("model_contract") != CAPACITY_MODEL_CONFIG.as_contract()
        or export.get("weights_cid") != cid_file(root / "export/model.safetensors")
        or export.get("weights_cid") != selection.get("weights_cid")
        or export.get("tokenizer_cid") != TOKENIZER_CID
        or export.get("tokenizer_cid") != selection.get("tokenizer_cid")
        or export.get("dataset_manifest_cid") != result.get("dataset_manifest_cid")
        or export.get("training_view_manifest_cid")
        != result.get("training_view_manifest_cid")
        or export.get("split_policy_cid") != selection.get("split_policy_cid")
        or export.get("run_contract_cid") != run_contract_cid
        or export.get("selected_checkpoint_cid")
        != selection.get("selected_checkpoint_cid")
        or export.get("training_result_cid") != result.get("result_cid")
        or export.get("manifest_cid") != selection.get("export_manifest_cid")
        or selection.get("training_result_cid") != result.get("result_cid")
    ):
        raise ValueError("#1019 export identity differs")
    prefix = json.loads((root / PYTHON_PREFIX_RELATIVE_PATH).read_text(encoding="utf-8"))
    if not isinstance(prefix, dict):
        raise ValueError("#1019 Python prefix must be a JSON object")
    _verify_signed(prefix, label="#1019 Python prefix")
    enabled = prefix.get("enabled")
    if not isinstance(enabled, dict):
        raise ValueError("#1019 Python prefix enabled output is missing")
    logits = enabled.get("logits")
    token_ids = prefix.get("prefix_token_ids")
    if (
        prefix.get("schema") != PYTHON_PREFIX_SCHEMA
        or prefix.get("result_cid") != selection.get("python_prefix_result_cid")
        or prefix.get("weights_cid") != selection.get("weights_cid")
        or prefix.get("token_store_cid") != cid_file(root / TOKEN_RELATIVE_PATHS["dev"])
        or prefix.get("maximum_absolute_logit_delta_limit")
        != PREFIX_LOGIT_ABS_TOLERANCE
        or not isinstance(token_ids, list)
        or len(token_ids) != PREFIX_PARITY_TOKENS
        or any(
            isinstance(token, bool)
            or not isinstance(token, int)
            or not 0 <= token < CAPACITY_MODEL_CONFIG.vocab_size
            for token in token_ids
        )
        or not isinstance(logits, list)
        or len(logits) != CAPACITY_MODEL_CONFIG.vocab_size
        or any(
            isinstance(logit, bool)
            or not isinstance(logit, (int, float))
            or not math.isfinite(float(logit))
            for logit in logits
        )
        or enabled.get("top1_token_id")
        != int(np.argmax(np.asarray(logits, dtype=np.float64)))
        or "attention_off" in prefix
    ):
        raise ValueError("#1019 Python prefix identity differs")
    return selection


def admit_capacity_prefix_parity(root: Path, rust_report_path: Path) -> dict[str, Any]:
    if (root / PREFIX_ADMISSION_RELATIVE_PATH).exists():
        raise FileExistsError("#1019 prefix parity admission is create-once")
    selection = load_frozen_capacity_selection(root)
    prefix = json.loads((root / PYTHON_PREFIX_RELATIVE_PATH).read_text(encoding="utf-8"))
    report_bytes = rust_report_path.read_bytes()
    report = json.loads(report_bytes)
    if not isinstance(report, dict):
        raise ValueError("#1019 Rust prefix qualification must be a JSON object")
    export_root = root / "export"
    export = verify_bound_manifest(
        export_root / "export-manifest.json", artifact_root=export_root
    )
    _validate_rust_report(
        report,
        prefix=prefix,
        prefix_path=root / PYTHON_PREFIX_RELATIVE_PATH,
        export=export,
        export_root=export_root,
    )
    atomic_write(root / RUST_PREFIX_RELATIVE_PATH, report_bytes)
    return write_bound_manifest(
        root / PREFIX_ADMISSION_RELATIVE_PATH,
        {
            "schema": PREFIX_ADMISSION_SCHEMA,
            "issue": ISSUE,
            "selection_manifest_cid": selection["manifest_cid"],
            "selected_checkpoint_cid": selection["selected_checkpoint_cid"],
            "weights_cid": selection["weights_cid"],
            "python_prefix_result_cid": prefix["result_cid"],
            "rust_decision_cid": report.get("decision_cid"),
            "qualification_passed": True,
            "attention_off_executions": 0,
            "sealed_confirmation_status": "UNOPENED",
        },
        artifact_root=root,
        relative_paths=[
            str(SELECTION_RELATIVE_PATH),
            str(PYTHON_PREFIX_RELATIVE_PATH),
            str(RUST_PREFIX_RELATIVE_PATH),
        ],
    )


def load_capacity_prefix_admission(root: Path) -> dict[str, Any]:
    selection = load_frozen_capacity_selection(root)
    admission = verify_bound_manifest(root / PREFIX_ADMISSION_RELATIVE_PATH, artifact_root=root)
    if (
        admission.get("schema") != PREFIX_ADMISSION_SCHEMA
        or admission.get("selection_manifest_cid") != selection["manifest_cid"]
        or admission.get("qualification_passed") is not True
        or admission.get("attention_off_executions") != 0
        or admission.get("sealed_confirmation_status") != "UNOPENED"
    ):
        raise ValueError("#1019 prefix admission differs")
    prefix_path = root / PYTHON_PREFIX_RELATIVE_PATH
    prefix = json.loads(prefix_path.read_text(encoding="utf-8"))
    report = json.loads((root / RUST_PREFIX_RELATIVE_PATH).read_text(encoding="utf-8"))
    export_root = root / "export"
    export = verify_bound_manifest(
        export_root / "export-manifest.json", artifact_root=export_root
    )
    if not isinstance(prefix, dict) or not isinstance(report, dict):
        raise ValueError("#1019 prefix admission artifacts are not JSON objects")
    _verify_signed(prefix, label="#1019 Python prefix")
    _validate_rust_report(
        report,
        prefix=prefix,
        prefix_path=prefix_path,
        export=export,
        export_root=export_root,
    )
    if (
        admission.get("python_prefix_result_cid") != prefix["result_cid"]
        or admission.get("rust_decision_cid") != report["decision_cid"]
    ):
        raise ValueError("#1019 prefix admission evidence identities differ")
    return admission


def _load_prompt_fixture(path: Path) -> list[dict[str, Any]]:
    fixture = json.loads(path.read_text(encoding="utf-8"))
    unsigned = dict(fixture)
    expected = unsigned.pop("fixture_cid", None)
    if expected != cid_bytes(canonical_json_bytes(unsigned)):
        raise ValueError("#1019 prompt fixture CID does not reproduce")
    prompts = fixture.get("prompts")
    if not isinstance(prompts, list) or len(prompts) != SEALED_PROMPT_COUNT:
        raise ValueError("#1019 reveal must contain five prompts")
    return prompts


def _load_safetensor_model(path: Path, *, capacity: bool, device: torch.device) -> R4SoftmaxForCausalLM:
    config = CAPACITY_MODEL_CONFIG if capacity else FROZEN_MODEL_CONFIG
    model = R4SoftmaxForCausalLM(config)
    state = load_file(str(path), device="cpu")
    model.load_state_dict(state, strict=True)
    return model.to(device)


def reveal_capacity(root: Path, *, baseline_1017_root: Path) -> dict[str, Any]:
    """Open the fresh confirmation once and score candidate plus frozen #1017."""
    if any((root / path).exists() for path in [REVEAL_OPENED_RELATIVE_PATH, REVEAL_MANIFEST_RELATIVE_PATH]):
        raise FileExistsError("#1019 reveal is create-once")
    selection = load_frozen_capacity_selection(root)
    parity = load_capacity_prefix_admission(root)
    training_result = json.loads((root / TRAINING_RESULT_RELATIVE_PATH).read_text(encoding="utf-8"))
    backend = training_result["run_contract"]["environment"]["backend"]["backend"]
    device, identity = require_backend(1019, backend)
    if identity != training_result["run_contract"]["environment"]["backend"]:
        raise ValueError("#1019 reveal accelerator differs from training")
    baseline_weights = baseline_1017_root.resolve() / "export/model.safetensors"
    if cid_file(baseline_weights) != BASELINE_1017_WEIGHTS_CID:
        raise ValueError("frozen #1017 baseline weights CID differs")
    marker = _write_signed(
        root / REVEAL_OPENED_RELATIVE_PATH,
        {
            "schema": REVEAL_OPENED_SCHEMA,
            "issue": ISSUE,
            "selection_manifest_cid": selection["manifest_cid"],
            "prefix_admission_manifest_cid": parity["manifest_cid"],
            "candidate_weights_cid": selection["weights_cid"],
            "baseline_1017_weights_cid": BASELINE_1017_WEIGHTS_CID,
            "sealed_confirmation_status_before_marker": "UNOPENED",
            "repeat_reveal_permitted": False,
        },
    )
    open_sealed_confirmation(root)
    dataset = load_capacity_dataset_manifest(root)
    test_path = root / TOKEN_RELATIVE_PATHS["test"]
    test_store = TokenStore(test_path)
    candidate = _load_safetensor_model(
        root / "export/model.safetensors", capacity=True, device=device
    )
    baseline = _load_safetensor_model(baseline_weights, capacity=False, device=device)
    batch_size = int(training_result["run_contract"]["optimization"]["batch_size"])
    candidate_nll = evaluate(candidate, test_store, device, batch_size)
    baseline_nll = evaluate(baseline, test_store, device, batch_size)
    prompts = _load_prompt_fixture(root / SEALED_PROMPT_RELATIVE_PATH)
    tokenizer = Tokenizer.from_file(str(root / TOKENIZER_RELATIVE_PATH))
    prompt_records: list[dict[str, Any]] = []
    for index, prompt in enumerate(prompts):
        token_ids = list(prompt["token_ids"])
        prompt_text = tokenizer.decode(token_ids, skip_special_tokens=True)
        if prompt_text != prompt["text"]:
            raise ValueError("#1019 sealed prompt text does not reproduce")
        prompt_records.append(
            {
                "index": index,
                "story_cid": prompt["story_cid"],
                "seed": 3019 + index,
                "prompt_tokens": SEALED_PROMPT_TOKENS_PER_STORY,
                "prompt_token_ids": token_ids,
                "prompt_text": prompt_text,
            }
        )
    absolute_passed = candidate_nll < SEALED_TEST_LOSS_CEILING
    relative_passed = candidate_nll < baseline_nll
    terminal = (
        "PASS_CAPACITY_NLL_ADVANCE_GENERATION"
        if absolute_passed and relative_passed
        else "FAIL_CAPACITY_NLL"
    )
    result = _write_signed(
        root / REVEAL_RESULT_RELATIVE_PATH,
        {
            "schema": REVEAL_RESULT_SCHEMA,
            "issue": ISSUE,
            "terminal": terminal,
            "selection_manifest_cid": selection["manifest_cid"],
            "prefix_admission_manifest_cid": parity["manifest_cid"],
            "reveal_opened_result_cid": marker["result_cid"],
            "dataset_manifest_cid": dataset["manifest_cid"],
            "candidate_weights_cid": selection["weights_cid"],
            "weights_cid": selection["weights_cid"],
            "tokenizer_cid": selection["tokenizer_cid"],
            "baseline_1017_weights_cid": BASELINE_1017_WEIGHTS_CID,
            "candidate_enabled_sealed_nll": candidate_nll,
            "baseline_1017_same_tranche_enabled_nll": baseline_nll,
            "historical_1017_sealed_nll": BASELINE_1017_NLL,
            "candidate_minus_baseline_same_tranche_nll": candidate_nll - baseline_nll,
            "sealed_nll_ceiling": SEALED_TEST_LOSS_CEILING,
            "absolute_nll_passed": absolute_passed,
            "relative_nll_passed": relative_passed,
            "sealed_test_store_token_ids": len(test_store.tokens),
            "sealed_test_scored_next_tokens": test_store.scored_next_tokens,
            "sealed_prompt_token_ids": SEALED_PROMPT_TOKEN_COUNT,
            "prompts": prompt_records,
            "autonomous_generation_status": "NOT_RUN_RUST_SEEDED_SAMPLER_REQUIRED",
            "attention_off_executions": 0,
            "prior_sealed_artifact_reads": 0,
        },
    )
    return write_bound_manifest(
        root / REVEAL_MANIFEST_RELATIVE_PATH,
        {
            "schema": REVEAL_MANIFEST_SCHEMA,
            "issue": ISSUE,
            "terminal": terminal,
            "selection_manifest_cid": selection["manifest_cid"],
            "prefix_admission_manifest_cid": parity["manifest_cid"],
            "reveal_opened_result_cid": marker["result_cid"],
            "reveal_result_cid": result["result_cid"],
            "candidate_enabled_sealed_nll": candidate_nll,
            "baseline_1017_same_tranche_enabled_nll": baseline_nll,
            "absolute_nll_passed": absolute_passed,
            "relative_nll_passed": relative_passed,
            "attention_off_executions": 0,
        },
        artifact_root=root,
        relative_paths=[
            str(REVEAL_OPENED_RELATIVE_PATH),
            str(REVEAL_RESULT_RELATIVE_PATH),
            TOKEN_RELATIVE_PATHS["test"],
            INDEX_RELATIVE_PATHS["test"],
            SEALED_PROMPT_RELATIVE_PATH,
        ],
    )
