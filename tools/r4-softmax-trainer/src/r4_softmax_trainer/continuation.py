"""One frozen #1017 quality-capacity continuation of the #1014 checkpoint.

This module is deliberately separate from :mod:`r4_softmax_trainer.train`.
The #1014 campaign, its two-arm attention intervention, and its manifests are
immutable.  #1017 may only inherit the selected model and AdamW state, train on
the independently prepared continuation population, qualify one enabled Rust
prefix, and reveal the new confirmation population once.
"""

from __future__ import annotations

import importlib.metadata
import json
import math
import os
import sys
import time
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Any

import numpy as np
import torch
from torch import Tensor
from tokenizers import Tokenizer

from .constants import (
    BOS_TOKEN_ID,
    FROZEN_MODEL_CONFIG,
    SEALED_PROMPT_COUNT,
    SEALED_PROMPT_TOKEN_COUNT,
    SEALED_PROMPT_TOKENS_PER_STORY,
)
from .continuation_data import (
    INDEX_RELATIVE_PATHS,
    INHERITED_CHECKPOINT_RELATIVE_PATH,
    SEALED_PROMPT_RELATIVE_PATH,
    TOKENIZER_RELATIVE_PATH,
    TOKEN_RELATIVE_PATHS,
    load_continuation_dataset_manifest,
    load_continuation_training_view_manifest,
    open_sealed_confirmation,
)
from .export import export_hugging_face_snapshot
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
from .train import TokenStore, _load_sealed_prompt_fixture, _sync_mps, evaluate, require_mps


ISSUE = 1017

INHERITED_CHECKPOINT_CID = (
    "blake3:9c36e109d8dee67deec0362307ba4a471c967ff574835210f87653d628c95c91"
)
INHERITED_WEIGHTS_CID = (
    "blake3:7d7c26e1a71866dc46973cea3b23b819f4b5060b345d2a0ec1bd067aa493bb7d"
)
INHERITED_TOKENIZER_CID = (
    "blake3:3f42bcfce7728512076549c63b88387e13c8156fe35c0f91d9b112439f3739cc"
)
INHERITED_DATASET_MANIFEST_CID = (
    "blake3:3e4d2ddb006771e5be0d4c76580c8971e6c67a23f8e223da8d81668d03bd9a01"
)
INHERITED_SPLIT_POLICY_CID = (
    "blake3:54f0886d3e906a4aeeaa9328ff236440d61d9f16b2f92dcb8c05cac96e54d1aa"
)
INHERITED_RUN_CONTRACT_CID = (
    "blake3:608005f95c12f3674bda6aead92b154db6d7e081b01bd4092636afb183b9aff4"
)
INHERITED_OPTIMIZER_STEP = 1_831
INHERITED_TRAIN_TOKENS = 29_999_104

CONTINUATION_OPTIMIZER_STEPS = 7_324
CONTINUATION_TRAIN_TOKENS = 119_996_416
CUMULATIVE_TRAIN_TOKENS = 149_995_520
CUMULATIVE_OPTIMIZER_STEPS = INHERITED_OPTIMIZER_STEP + CONTINUATION_OPTIMIZER_STEPS
CONTINUATION_TRAIN_STORE_TOKENS = 119_996_416
CONTINUATION_DEV_STORE_TOKENS = 250_000
CONTINUATION_TEST_STORE_TOKENS = 249_880
CONTINUATION_WALL_SECONDS = 5 * 60 * 60 + 15 * 60

CONTINUATION_RUN_SCHEMA = "uor-r4-softmax-trainer-continuation-run/1"
CONTINUATION_CHECKPOINT_SCHEMA = "uor-r4-softmax-trainer-continuation-checkpoint/1"
CONTINUATION_RESULT_SCHEMA = "uor-r4-softmax-trainer-continuation-selection-result/1"
CONTINUATION_SELECTION_SCHEMA = "uor-r4-softmax-trainer-continuation-selection/1"
ENABLED_PREFIX_SCHEMA = "uor-r4.r4-softmax-python-enabled-prefix-logits/1"
RUST_ENABLED_QUALIFICATION_SCHEMA = "uor-r4.r4-softmax-local-enabled-qualification/1"
ENABLED_PARITY_ADMISSION_SCHEMA = "uor-r4-softmax-trainer-enabled-parity-admission/1"
CONTINUATION_REVEAL_RESULT_SCHEMA = "uor-r4-softmax-trainer-continuation-reveal/1"
CONTINUATION_REVEAL_MANIFEST_SCHEMA = (
    "uor-r4-softmax-trainer-continuation-reveal-manifest/1"
)
CONTINUATION_REVEAL_OPENED_SCHEMA = (
    "uor-r4-softmax-trainer-continuation-reveal-opened/1"
)
UNAVAILABLE_MPS_BUDGET_SCHEMA = "uor-r4-softmax-trainer-continuation-unavailable/1"
CONTINUATION_ELAPSED_SCHEMA = "uor-r4-softmax-trainer-continuation-elapsed/1"

PYTHON_ENABLED_PREFIX_RELATIVE_PATH = Path(
    "qualification/python-enabled-prefix-logits.json"
)
RUST_ENABLED_QUALIFICATION_RELATIVE_PATH = Path(
    "qualification/rust-enabled-prefix-qualification.json"
)
ENABLED_PARITY_ADMISSION_RELATIVE_PATH = Path(
    "qualification/enabled-prefix-admission.json"
)
SELECTION_RELATIVE_PATH = Path("selection/continuation-selection-manifest.json")
TRAINING_RESULT_RELATIVE_PATH = Path("continuation-training-result.json")
TRAINING_STATUS_RELATIVE_PATH = Path("continuation-training-status.json")
ELAPSED_LEDGER_RELATIVE_PATH = Path("continuation-elapsed-ledger.json")
REVEAL_OPENED_RELATIVE_PATH = Path("reveal/continuation-opened.json")
REVEAL_RESULT_RELATIVE_PATH = Path("reveal/continuation-reveal-result.json")
REVEAL_MANIFEST_RELATIVE_PATH = Path("reveal/continuation-reveal-manifest.json")

PREFIX_PARITY_TOKENS = 32
PREFIX_LOGIT_ABS_TOLERANCE = 0.005
SEALED_TEST_LOSS_CEILING = 1.50


@dataclass(frozen=True, slots=True)
class ContinuationConfig:
    """The one optimization contract authorized by #1017."""

    seed: int = 1014
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
    evaluation_interval: int = 100
    checkpoint_interval: int = 100
    optimizer_steps: int = CONTINUATION_OPTIMIZER_STEPS
    continuation_train_tokens: int = CONTINUATION_TRAIN_TOKENS
    wall_ceiling_seconds: float = CONTINUATION_WALL_SECONDS

    @property
    def tokens_per_optimizer_step(self) -> int:
        return (
            self.batch_size
            * self.gradient_accumulation_steps
            * FROZEN_MODEL_CONFIG.max_position_embeddings
        )

    def validate(self) -> None:
        expected = ContinuationConfig()
        if self != expected:
            raise ValueError("#1017 permits only its exact frozen continuation config")
        if self.optimizer_steps * self.tokens_per_optimizer_step != self.continuation_train_tokens:
            raise ValueError("#1017 optimizer-step and token budgets differ")
        if INHERITED_TRAIN_TOKENS + self.continuation_train_tokens != CUMULATIVE_TRAIN_TOKENS:
            raise ValueError("#1017 cumulative token arithmetic differs")

    def as_contract(self) -> dict[str, Any]:
        self.validate()
        value = asdict(self)
        value["tokens_per_optimizer_step"] = self.tokens_per_optimizer_step
        value["inherited_optimizer_steps"] = INHERITED_OPTIMIZER_STEP
        value["cumulative_optimizer_steps"] = CUMULATIVE_OPTIMIZER_STEPS
        value["inherited_train_tokens"] = INHERITED_TRAIN_TOKENS
        value["cumulative_train_tokens"] = CUMULATIVE_TRAIN_TOKENS
        return value


def phase_two_learning_rate(step: int, config: ContinuationConfig) -> float:
    """Return the predeclared floor-to-peak-to-floor phase-two schedule."""
    config.validate()
    if not 0 <= step <= config.optimizer_steps:
        raise ValueError("continuation optimizer step is outside the frozen schedule")
    if step <= config.warmup_steps:
        fraction = step / config.warmup_steps
        return config.minimum_learning_rate + fraction * (
            config.learning_rate - config.minimum_learning_rate
        )
    progress = (step - config.warmup_steps) / (
        config.optimizer_steps - config.warmup_steps
    )
    cosine = 0.5 * (1.0 + math.cos(math.pi * progress))
    return config.minimum_learning_rate + cosine * (
        config.learning_rate - config.minimum_learning_rate
    )


def continuation_batch_index(
    step: int, accumulation: int, config: ContinuationConfig
) -> int:
    """Continue #1014's deterministic sampler ledger on the fresh store."""
    config.validate()
    if not 1 <= step <= config.optimizer_steps:
        raise ValueError("continuation step is outside the frozen sampler ledger")
    if not 0 <= accumulation < config.gradient_accumulation_steps:
        raise ValueError("accumulation index is outside the frozen sampler ledger")
    return (
        (INHERITED_OPTIMIZER_STEP + step - 1) * config.gradient_accumulation_steps
        + accumulation
    )


def _dependency_versions() -> dict[str, str]:
    return {
        name: importlib.metadata.version(name)
        for name in ["blake3", "numpy", "safetensors", "tokenizers", "torch"]
    }


def _tool_root() -> Path:
    return Path(__file__).resolve().parents[2]


def _require_predecessor_identity(training_view: dict[str, Any]) -> None:
    predecessor = training_view.get("predecessor")
    if not isinstance(predecessor, dict):
        raise ValueError("continuation training view has no predecessor identity")
    expected = {
        "dataset_manifest_cid": INHERITED_DATASET_MANIFEST_CID,
        "split_policy_cid": INHERITED_SPLIT_POLICY_CID,
        "checkpoint_cid": INHERITED_CHECKPOINT_CID,
        "weights_cid": INHERITED_WEIGHTS_CID,
        "tokenizer_cid": INHERITED_TOKENIZER_CID,
    }
    for field, identity in expected.items():
        if predecessor.get(field) != identity:
            raise ValueError(f"continuation predecessor {field} differs from #1014")
    if training_view.get("split_policy_cid") != INHERITED_SPLIT_POLICY_CID:
        raise ValueError("continuation split policy differs from #1014")
    if training_view.get("model_contract") != FROZEN_MODEL_CONFIG.as_contract():
        raise ValueError("continuation model contract differs from #1014")
    manifest_cid = training_view.get("continuation_dataset_manifest_cid")
    if not isinstance(manifest_cid, str) or not manifest_cid.startswith("blake3:"):
        raise ValueError("continuation training view has no population manifest CID")


def build_continuation_run_contract(
    training_view: dict[str, Any], config: ContinuationConfig
) -> dict[str, Any]:
    """Build the pre-test contract without opening either sealed population."""
    config.validate()
    _require_predecessor_identity(training_view)
    lock_path = _tool_root() / "uv.lock"
    if not lock_path.is_file():
        raise FileNotFoundError("uv.lock is required before #1017 can be frozen")
    return {
        "schema": CONTINUATION_RUN_SCHEMA,
        "issue": ISSUE,
        "inheritance": {
            "checkpoint_cid": INHERITED_CHECKPOINT_CID,
            "weights_cid": INHERITED_WEIGHTS_CID,
            "tokenizer_cid": INHERITED_TOKENIZER_CID,
            "dataset_manifest_cid": INHERITED_DATASET_MANIFEST_CID,
            "split_policy_cid": INHERITED_SPLIT_POLICY_CID,
            "run_contract_cid": INHERITED_RUN_CONTRACT_CID,
            "optimizer_step": INHERITED_OPTIMIZER_STEP,
            "train_tokens": INHERITED_TRAIN_TOKENS,
            "optimizer_state": "exact inherited AdamW state",
        },
        "continuation_dataset_manifest_cid": training_view[
            "continuation_dataset_manifest_cid"
        ],
        "continuation_training_view_manifest_cid": training_view["manifest_cid"],
        "split_policy_cid": training_view["split_policy_cid"],
        "environment": {
            "python": ".".join(map(str, sys.version_info[:3])),
            "dependencies": _dependency_versions(),
            "uv_lock_cid": cid_file(lock_path),
            "device": "mps",
            "cpu_fallback": False,
            "dtype": "float32",
        },
        "trainer_implementation": trainer_implementation_contract(),
        "model": FROZEN_MODEL_CONFIG.as_contract(),
        "optimization": config.as_contract(),
        "training_sampler": {
            "policy": "BLAKE3(seed,batch_index,lane) modulo fresh-store start range",
            "seed": config.seed,
            "batch_index": (
                "continue after #1014's 1,831 optimizer steps; never address the old store"
            ),
        },
        "selection": (
            "minimum fresh complete-development-token mean causal cross-entropy; "
            "includes inherited step zero and every fixed checkpoint"
        ),
        "test_policy": (
            "fresh sealed confirmation is inaccessible until selection and enabled-only "
            "Python/Rust parity are frozen"
        ),
        "parity": {
            "arms": ["enabled"],
            "prefix_source": "first 32 token IDs of the fresh development store",
            "maximum_absolute_logit_delta": PREFIX_LOGIT_ABS_TOLERANCE,
        },
        "generation": {
            "owner": "Rust all-layer coherent R4/Spin local generator",
            "seeds": list(range(2014, 2019)),
            "new_tokens": 128,
            "selection": (
                "r4-local-top-k-q32-splitmix64/1; temperature 0.8; top-k 40"
            ),
        },
    }


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


def _optimizer_to_device(optimizer: torch.optim.Optimizer, device: torch.device) -> None:
    for state in optimizer.state.values():
        for key, value in state.items():
            if isinstance(value, Tensor):
                state[key] = value.to(device)


def _optimizer_state_step(value: object) -> int:
    if isinstance(value, Tensor):
        if value.numel() != 1:
            raise ValueError("AdamW step state is not scalar")
        return int(value.detach().cpu().item())
    if isinstance(value, (int, float)):
        return int(value)
    raise ValueError("AdamW state has no numeric step")


def _validate_optimizer_state(
    optimizer: torch.optim.Optimizer,
    *,
    expected_step: int,
    expected_learning_rate: float,
) -> None:
    if len(optimizer.param_groups) != 1:
        raise ValueError("#1017 requires one inherited AdamW parameter group")
    group = optimizer.param_groups[0]
    if len(group.get("params", [])) != 56 or len(optimizer.state) != 56:
        raise ValueError("inherited AdamW state does not cover all 56 parameter tensors")
    if tuple(group.get("betas", ())) != (0.9, 0.95):
        raise ValueError("inherited AdamW betas differ")
    if float(group.get("weight_decay", math.nan)) != 0.1:
        raise ValueError("inherited AdamW weight decay differs")
    if float(group.get("eps", math.nan)) != 1e-8:
        raise ValueError("inherited AdamW epsilon differs")
    if not math.isclose(
        float(group.get("lr", math.nan)), expected_learning_rate, rel_tol=0.0, abs_tol=1e-15
    ):
        raise ValueError("AdamW learning rate differs from the frozen phase-two schedule")
    steps = {_optimizer_state_step(state.get("step")) for state in optimizer.state.values()}
    if steps != {expected_step}:
        raise ValueError(f"AdamW state steps {sorted(steps)} != expected {expected_step}")


def _validate_inherited_checkpoint_envelope(checkpoint: dict[str, Any]) -> None:
    if checkpoint.get("schema") != "uor-r4-softmax-trainer-checkpoint/1":
        raise ValueError("unsupported #1014 checkpoint schema")
    if checkpoint.get("run_contract_cid") != INHERITED_RUN_CONTRACT_CID:
        raise ValueError("inherited checkpoint run-contract CID differs from #1014")
    if (
        cid_bytes(canonical_json_bytes(checkpoint.get("run_contract")))
        != INHERITED_RUN_CONTRACT_CID
    ):
        raise ValueError("inherited checkpoint embedded run contract does not reproduce")
    if int(checkpoint.get("optimizer_step", -1)) != INHERITED_OPTIMIZER_STEP:
        raise ValueError("inherited checkpoint optimizer step differs from #1014")
    if int(checkpoint.get("tokens_seen", -1)) != INHERITED_TRAIN_TOKENS:
        raise ValueError("inherited checkpoint token count differs from #1014")
    run_contract = checkpoint["run_contract"]
    if run_contract.get("model") != FROZEN_MODEL_CONFIG.as_contract():
        raise ValueError("inherited checkpoint model contract differs from #1014")


def _load_inherited_checkpoint(
    path: Path,
    *,
    model: R4SoftmaxForCausalLM,
    optimizer: torch.optim.Optimizer,
    device: torch.device,
) -> dict[str, Any]:
    if cid_file(path) != INHERITED_CHECKPOINT_CID:
        raise ValueError("inherited checkpoint CID differs from frozen #1014 selection")
    checkpoint = torch.load(path, map_location="cpu", weights_only=False)
    if not isinstance(checkpoint, dict):
        raise ValueError("inherited checkpoint is not a mapping")
    _validate_inherited_checkpoint_envelope(checkpoint)
    model.load_state_dict(checkpoint["model"], strict=True)
    optimizer.load_state_dict(checkpoint["optimizer"])
    _optimizer_to_device(optimizer, device)
    _validate_optimizer_state(
        optimizer,
        expected_step=INHERITED_OPTIMIZER_STEP,
        expected_learning_rate=ContinuationConfig().minimum_learning_rate,
    )
    cpu_rng_state = checkpoint.get("cpu_rng_state")
    if not isinstance(cpu_rng_state, Tensor):
        raise ValueError("inherited checkpoint has no CPU RNG state")
    torch.set_rng_state(cpu_rng_state)
    return checkpoint


def _continuation_counts(step: int, config: ContinuationConfig) -> dict[str, int]:
    if not 0 <= step <= config.optimizer_steps:
        raise ValueError("continuation step outside frozen budget")
    continuation_tokens = step * config.tokens_per_optimizer_step
    return {
        "continuation_optimizer_step": step,
        "cumulative_optimizer_step": INHERITED_OPTIMIZER_STEP + step,
        "continuation_tokens_seen": continuation_tokens,
        "cumulative_tokens_seen": INHERITED_TRAIN_TOKENS + continuation_tokens,
    }


def _save_continuation_checkpoint(
    path: Path,
    *,
    model: R4SoftmaxForCausalLM,
    optimizer: torch.optim.Optimizer,
    continuation_step: int,
    elapsed_continuation_seconds: float,
    best_dev_loss: float,
    development_candidates: list[dict[str, Any]],
    run_contract: dict[str, Any],
    run_contract_cid: str,
    config: ContinuationConfig,
) -> None:
    counts = _continuation_counts(continuation_step, config)
    _validate_optimizer_state(
        optimizer,
        expected_step=counts["cumulative_optimizer_step"],
        expected_learning_rate=phase_two_learning_rate(continuation_step, config),
    )
    payload: dict[str, Any] = {
        "schema": CONTINUATION_CHECKPOINT_SCHEMA,
        "run_contract": run_contract,
        "run_contract_cid": run_contract_cid,
        "inherited_checkpoint_cid": INHERITED_CHECKPOINT_CID,
        **counts,
        "elapsed_continuation_seconds": elapsed_continuation_seconds,
        "best_dev_loss": best_dev_loss,
        "development_candidates": development_candidates,
        "model": _cpu_tree(model.state_dict()),
        "optimizer": _cpu_tree(optimizer.state_dict()),
        "cpu_rng_state": torch.get_rng_state(),
    }
    path.parent.mkdir(parents=True, exist_ok=True)
    temporary = path.with_name(f".{path.name}.part")
    torch.save(payload, temporary)
    os.replace(temporary, path)


def _load_continuation_checkpoint(
    path: Path,
    *,
    model: R4SoftmaxForCausalLM,
    optimizer: torch.optim.Optimizer | None,
    device: torch.device,
    run_contract_cid: str,
    config: ContinuationConfig,
) -> dict[str, Any]:
    checkpoint = torch.load(path, map_location="cpu", weights_only=False)
    _validate_continuation_checkpoint_envelope(
        checkpoint,
        run_contract_cid=run_contract_cid,
        config=config,
    )
    step = int(checkpoint["continuation_optimizer_step"])
    expected_counts = _continuation_counts(step, config)
    model.load_state_dict(checkpoint["model"], strict=True)
    if optimizer is not None:
        optimizer.load_state_dict(checkpoint["optimizer"])
        _optimizer_to_device(optimizer, device)
        _validate_optimizer_state(
            optimizer,
            expected_step=expected_counts["cumulative_optimizer_step"],
            expected_learning_rate=phase_two_learning_rate(step, config),
        )
    cpu_rng_state = checkpoint.get("cpu_rng_state")
    if not isinstance(cpu_rng_state, Tensor):
        raise ValueError("continuation checkpoint has no CPU RNG state")
    torch.set_rng_state(cpu_rng_state)
    return checkpoint


def _expected_development_candidate_steps(
    continuation_step: int, config: ContinuationConfig
) -> list[int]:
    steps = [
        0,
        *range(
            config.evaluation_interval,
            continuation_step + 1,
            config.evaluation_interval,
        ),
    ]
    if continuation_step == config.optimizer_steps and steps[-1] != continuation_step:
        steps.append(continuation_step)
    return steps


def _validate_development_candidates(
    candidates: object,
    *,
    continuation_step: int,
    config: ContinuationConfig,
) -> list[dict[str, Any]]:
    if not isinstance(candidates, list):
        raise ValueError("continuation checkpoint has no development candidate ledger")
    expected_steps = _expected_development_candidate_steps(continuation_step, config)
    actual_steps: list[int] = []
    validated: list[dict[str, Any]] = []
    for candidate in candidates:
        if not isinstance(candidate, dict):
            raise ValueError("continuation checkpoint has an invalid development candidate")
        step = int(candidate.get("continuation_step", -1))
        counts = _continuation_counts(step, config)
        if int(candidate.get("cumulative_optimizer_step", -1)) != counts[
            "cumulative_optimizer_step"
        ]:
            raise ValueError("continuation development candidate step arithmetic differs")
        if int(candidate.get("cumulative_train_tokens", -1)) != counts[
            "cumulative_tokens_seen"
        ]:
            raise ValueError("continuation development candidate token arithmetic differs")
        loss = float(candidate.get("development_loss", math.nan))
        if not math.isfinite(loss):
            raise ValueError("continuation development candidate loss is not finite")
        actual_steps.append(step)
        validated.append(candidate)
    if actual_steps != expected_steps:
        raise ValueError("continuation development candidate ledger is incomplete")
    return validated


def _validate_continuation_checkpoint_envelope(
    checkpoint: object,
    *,
    run_contract_cid: str,
    config: ContinuationConfig,
) -> int:
    if not isinstance(checkpoint, dict) or checkpoint.get("schema") != CONTINUATION_CHECKPOINT_SCHEMA:
        raise ValueError("unsupported #1017 continuation checkpoint")
    if checkpoint.get("run_contract_cid") != run_contract_cid:
        raise ValueError("continuation checkpoint belongs to a different frozen run")
    if cid_bytes(canonical_json_bytes(checkpoint.get("run_contract"))) != run_contract_cid:
        raise ValueError("continuation checkpoint embedded run contract does not reproduce")
    if checkpoint.get("inherited_checkpoint_cid") != INHERITED_CHECKPOINT_CID:
        raise ValueError("continuation checkpoint inherited a different model")
    step = int(checkpoint.get("continuation_optimizer_step", -1))
    expected_counts = _continuation_counts(step, config)
    for field, expected in expected_counts.items():
        if int(checkpoint.get(field, -1)) != expected:
            raise ValueError(f"continuation checkpoint {field} arithmetic differs")
    candidates = _validate_development_candidates(
        checkpoint.get("development_candidates"),
        continuation_step=step,
        config=config,
    )
    best_dev_loss = float(checkpoint.get("best_dev_loss", math.nan))
    elapsed = float(checkpoint.get("elapsed_continuation_seconds", math.nan))
    if not math.isfinite(best_dev_loss) or not math.isclose(
        best_dev_loss,
        min(float(candidate["development_loss"]) for candidate in candidates),
        rel_tol=0.0,
        abs_tol=1e-12,
    ):
        raise ValueError("continuation checkpoint best development loss differs")
    if not math.isfinite(elapsed) or elapsed < 0.0:
        raise ValueError("continuation checkpoint elapsed time is invalid")
    cpu_rng_state = checkpoint.get("cpu_rng_state")
    if not isinstance(cpu_rng_state, Tensor):
        raise ValueError("continuation checkpoint has no CPU RNG state")
    return step


def _continuation_checkpoint_step(
    path: Path, *, run_contract_cid: str, config: ContinuationConfig
) -> int:
    checkpoint = torch.load(path, map_location="cpu", weights_only=False)
    return _validate_continuation_checkpoint_envelope(
        checkpoint,
        run_contract_cid=run_contract_cid,
        config=config,
    )


def _select_resume_checkpoint(
    *,
    latest_path: Path,
    best_path: Path,
    run_contract_cid: str,
    config: ContinuationConfig,
) -> Path:
    checkpoint_steps = [
        (
            _continuation_checkpoint_step(
                path,
                run_contract_cid=run_contract_cid,
                config=config,
            ),
            path == latest_path,
            path,
        )
        for path in (best_path, latest_path)
        if path.is_file()
    ]
    if not checkpoint_steps:
        raise FileNotFoundError("--resume requested without a #1017 checkpoint")
    _, _, selected = max(checkpoint_steps)
    return selected


def _write_unavailable_mps_budget(
    root: Path,
    *,
    run_contract_cid: str,
    continuation_step: int,
    elapsed_seconds: float,
    config: ContinuationConfig,
) -> dict[str, Any]:
    value: dict[str, Any] = {
        "schema": UNAVAILABLE_MPS_BUDGET_SCHEMA,
        "terminal": "UNAVAILABLE_MPS_BUDGET",
        "issue": ISSUE,
        "run_contract_cid": run_contract_cid,
        **_continuation_counts(continuation_step, config),
        "elapsed_continuation_seconds": elapsed_seconds,
        "wall_ceiling_seconds": config.wall_ceiling_seconds,
        "partial_checkpoint_interpretable": False,
        "selection_export_or_reveal_permitted": False,
    }
    value["result_cid"] = cid_bytes(canonical_json_bytes(value))
    atomic_write_json(root / TRAINING_STATUS_RELATIVE_PATH, value)
    return value


def _write_elapsed_ledger(
    root: Path,
    *,
    run_contract_cid: str,
    continuation_step: int,
    elapsed_seconds: float,
    config: ContinuationConfig,
) -> dict[str, Any]:
    value: dict[str, Any] = {
        "schema": CONTINUATION_ELAPSED_SCHEMA,
        "issue": ISSUE,
        "run_contract_cid": run_contract_cid,
        **_continuation_counts(continuation_step, config),
        "elapsed_continuation_seconds": elapsed_seconds,
        "wall_ceiling_seconds": config.wall_ceiling_seconds,
    }
    value["result_cid"] = cid_bytes(canonical_json_bytes(value))
    atomic_write_json(root / ELAPSED_LEDGER_RELATIVE_PATH, value)
    return value


def _load_elapsed_ledger(
    root: Path, *, run_contract_cid: str, config: ContinuationConfig
) -> dict[str, Any]:
    value = json.loads((root / ELAPSED_LEDGER_RELATIVE_PATH).read_text(encoding="utf-8"))
    if not isinstance(value, dict):
        raise ValueError("continuation elapsed ledger is not a JSON object")
    _verify_signed_result(value, label="continuation elapsed")
    if (
        value.get("schema") != CONTINUATION_ELAPSED_SCHEMA
        or value.get("issue") != ISSUE
        or value.get("run_contract_cid") != run_contract_cid
        or float(value.get("wall_ceiling_seconds", math.nan))
        != config.wall_ceiling_seconds
    ):
        raise ValueError("continuation elapsed ledger belongs to another run")
    step = int(value.get("continuation_optimizer_step", -1))
    for field, expected in _continuation_counts(step, config).items():
        if int(value.get(field, -1)) != expected:
            raise ValueError(f"continuation elapsed ledger {field} arithmetic differs")
    elapsed = float(value.get("elapsed_continuation_seconds", math.nan))
    if not math.isfinite(elapsed) or elapsed < 0.0:
        raise ValueError("continuation elapsed ledger time is invalid")
    return value


def _write_enabled_prefix_fixture(
    root: Path,
    *,
    model: R4SoftmaxForCausalLM,
    dev_store: TokenStore,
    device: torch.device,
    weights_cid: str,
) -> dict[str, Any]:
    prefix_token_ids = np.asarray(
        dev_store.tokens[:PREFIX_PARITY_TOKENS], dtype=np.int64
    ).tolist()
    if len(prefix_token_ids) != PREFIX_PARITY_TOKENS:
        raise ValueError("fresh development store cannot supply the parity prefix")
    if prefix_token_ids[0] != BOS_TOKEN_ID:
        raise ValueError("fresh development parity prefix does not begin with BOS")
    with torch.no_grad():
        inputs = torch.tensor([prefix_token_ids], dtype=torch.long, device=device)
        logits = model(inputs).logits[0, -1].float().cpu().tolist()
    if len(logits) != FROZEN_MODEL_CONFIG.vocab_size or not all(
        math.isfinite(value) for value in logits
    ):
        raise ValueError("enabled Python prefix logits are not finite vocabulary logits")
    result: dict[str, Any] = {
        "schema": ENABLED_PREFIX_SCHEMA,
        "weights_cid": weights_cid,
        "token_store_cid": cid_file(root / TOKEN_RELATIVE_PATHS["dev"]),
        "prefix_token_ids": prefix_token_ids,
        "maximum_absolute_logit_delta_limit": PREFIX_LOGIT_ABS_TOLERANCE,
        "enabled": {
            "top1_token_id": int(np.argmax(np.asarray(logits))),
            "logits": logits,
        },
    }
    result["result_cid"] = cid_bytes(canonical_json_bytes(result))
    atomic_write_json(root / PYTHON_ENABLED_PREFIX_RELATIVE_PATH, result)
    return result


_SELECTION_ARTIFACT_PATHS = {
    str(INHERITED_CHECKPOINT_RELATIVE_PATH),
    "checkpoints/best.pt",
    str(ELAPSED_LEDGER_RELATIVE_PATH),
    str(TRAINING_RESULT_RELATIVE_PATH),
    "export/config.json",
    "export/model.safetensors",
    "export/tokenizer.json",
    "export/training-result.json",
    "export/export-manifest.json",
    str(PYTHON_ENABLED_PREFIX_RELATIVE_PATH),
}


def _manifest_artifact_paths(manifest: dict[str, Any], *, label: str) -> set[str]:
    records = manifest.get("artifacts")
    if not isinstance(records, list):
        raise ValueError(f"{label} has no artifact records")
    paths: set[str] = set()
    for record in records:
        if not isinstance(record, dict) or not isinstance(record.get("path"), str):
            raise ValueError(f"{label} has an invalid artifact record")
        path = str(record["path"])
        if path in paths:
            raise ValueError(f"{label} repeats artifact path {path}")
        paths.add(path)
    return paths


def _verify_signed_result(value: dict[str, Any], *, label: str) -> None:
    unsigned = dict(value)
    expected = unsigned.pop("result_cid", None)
    if expected != cid_bytes(canonical_json_bytes(unsigned)):
        raise ValueError(f"{label} result CID does not reproduce")


def _load_frozen_continuation_selection(root: Path) -> dict[str, Any]:
    selection = verify_bound_manifest(root / SELECTION_RELATIVE_PATH, artifact_root=root)
    if selection.get("schema") != CONTINUATION_SELECTION_SCHEMA:
        raise ValueError("unsupported #1017 selection manifest")
    if _manifest_artifact_paths(selection, label="continuation selection") != (
        _SELECTION_ARTIFACT_PATHS
    ):
        raise ValueError("continuation selection does not bind its exact frozen artifacts")
    if selection.get("sealed_confirmation_status") != "UNOPENED_BEFORE_THIS_MANIFEST":
        raise ValueError("continuation selection has an invalid sealed boundary")
    if selection.get("selected_checkpoint_cid") != cid_file(root / "checkpoints/best.pt"):
        raise ValueError("continuation selected checkpoint CID does not reproduce")
    if selection.get("inherited_checkpoint_cid") != cid_file(
        root / INHERITED_CHECKPOINT_RELATIVE_PATH
    ):
        raise ValueError("continuation inherited checkpoint CID does not reproduce")

    result = json.loads((root / TRAINING_RESULT_RELATIVE_PATH).read_text(encoding="utf-8"))
    if not isinstance(result, dict) or result.get("schema") != CONTINUATION_RESULT_SCHEMA:
        raise ValueError("unsupported #1017 training result")
    if (
        result.get("terminal") != "FINAL_CHECKPOINT_FROZEN_CONFIRMATION_UNOPENED"
        or result.get("sealed_confirmation_status") != "UNOPENED"
    ):
        raise ValueError("#1017 result is not a frozen pre-reveal selection")
    _verify_signed_result(result, label="continuation training")
    run_contract = result.get("run_contract")
    if not isinstance(run_contract, dict):
        raise ValueError("continuation result has no run contract")
    run_contract_cid = cid_bytes(canonical_json_bytes(run_contract))
    if result.get("run_contract_cid") != run_contract_cid:
        raise ValueError("continuation result run contract CID does not reproduce")
    if selection.get("run_contract_cid") != run_contract_cid:
        raise ValueError("continuation selection and result run contracts differ")
    if result.get("optimizer_steps_completed") != CONTINUATION_OPTIMIZER_STEPS:
        raise ValueError("continuation did not complete exactly 7,324 steps")
    if result.get("cumulative_train_tokens") != CUMULATIVE_TRAIN_TOKENS:
        raise ValueError("continuation did not reach the frozen cumulative token count")
    candidates = result.get("development_selection_candidates")
    candidates = _validate_development_candidates(
        candidates,
        continuation_step=CONTINUATION_OPTIMIZER_STEPS,
        config=ContinuationConfig(),
    )
    selected_candidate = min(
        candidates,
        key=lambda candidate: (
            float(candidate["development_loss"]),
            int(candidate["continuation_step"]),
        ),
    )
    if selected_candidate.get("continuation_step") != result.get(
        "selected_checkpoint_continuation_step"
    ):
        raise ValueError("continuation selection is not the development minimum")

    export = verify_bound_manifest(
        root / "export/export-manifest.json", artifact_root=root / "export"
    )
    if export.get("weights_cid") != cid_file(root / "export/model.safetensors"):
        raise ValueError("continuation export weights CID does not reproduce")
    if export.get("tokenizer_cid") != INHERITED_TOKENIZER_CID:
        raise ValueError("continuation export tokenizer differs from #1014")
    for field in (
        "continuation_dataset_manifest_cid",
        "continuation_training_view_manifest_cid",
        "split_policy_cid",
        "run_contract_cid",
        "selected_checkpoint_cid",
        "training_result_cid",
        "elapsed_ledger_result_cid",
        "weights_cid",
        "tokenizer_cid",
        "python_enabled_prefix_result_cid",
    ):
        if field not in selection:
            raise ValueError(f"continuation selection omits {field}")
    if export.get("dataset_manifest_cid") != selection[
        "continuation_dataset_manifest_cid"
    ]:
        raise ValueError("continuation export population identity differs")
    if export.get("training_view_manifest_cid") != selection[
        "continuation_training_view_manifest_cid"
    ]:
        raise ValueError("continuation export training view differs")
    if export.get("run_contract_cid") != run_contract_cid:
        raise ValueError("continuation export run contract differs")
    if export.get("selected_checkpoint_cid") != selection["selected_checkpoint_cid"]:
        raise ValueError("continuation export selected checkpoint differs")
    if export.get("manifest_cid") != selection["export_manifest_cid"]:
        raise ValueError("continuation export manifest identity differs")
    if export.get("training_result_cid") != selection["training_result_cid"]:
        raise ValueError("continuation export training result differs")
    if result.get("result_cid") != selection["training_result_cid"]:
        raise ValueError("continuation selection binds a different training result")
    elapsed_ledger = _load_elapsed_ledger(
        root,
        run_contract_cid=run_contract_cid,
        config=ContinuationConfig(),
    )
    if (
        elapsed_ledger.get("continuation_optimizer_step")
        != CONTINUATION_OPTIMIZER_STEPS
        or elapsed_ledger.get("result_cid")
        != result.get("elapsed_ledger_result_cid")
        or elapsed_ledger.get("result_cid")
        != selection["elapsed_ledger_result_cid"]
        or float(elapsed_ledger["elapsed_continuation_seconds"])
        != float(result["elapsed_continuation_seconds"])
        or float(elapsed_ledger["elapsed_continuation_seconds"])
        >= ContinuationConfig().wall_ceiling_seconds
    ):
        raise ValueError("continuation elapsed ledger does not bind the completed run")
    if export.get("weights_cid") != selection["weights_cid"]:
        raise ValueError("continuation selection binds different exported weights")
    if export.get("tokenizer_cid") != selection["tokenizer_cid"]:
        raise ValueError("continuation selection binds a different tokenizer")
    if export.get("split_policy_cid") != selection["split_policy_cid"]:
        raise ValueError("continuation export split policy differs")

    prefix = json.loads(
        (root / PYTHON_ENABLED_PREFIX_RELATIVE_PATH).read_text(encoding="utf-8")
    )
    if not isinstance(prefix, dict) or prefix.get("schema") != ENABLED_PREFIX_SCHEMA:
        raise ValueError("unsupported enabled Python prefix fixture")
    _verify_signed_result(prefix, label="enabled Python prefix")
    if "attention_off" in prefix:
        raise ValueError("#1017 enabled prefix fixture executed an attention-off arm")
    if prefix.get("result_cid") != selection["python_enabled_prefix_result_cid"]:
        raise ValueError("continuation selection binds a different prefix result")
    if prefix.get("weights_cid") != selection["weights_cid"]:
        raise ValueError("enabled prefix fixture binds different weights")
    return selection


def train_continuation(
    root: Path,
    *,
    config: ContinuationConfig = ContinuationConfig(),
    resume: bool = False,
) -> dict[str, Any]:
    """Run or resume the sole #1017 continuation without opening sealed data."""
    config.validate()
    selection_path = root / SELECTION_RELATIVE_PATH
    if selection_path.exists():
        _load_frozen_continuation_selection(root)
        raise FileExistsError("the #1017 selection is frozen and immutable")
    if (root / REVEAL_OPENED_RELATIVE_PATH).exists() or (
        root / REVEAL_MANIFEST_RELATIVE_PATH
    ).exists():
        raise FileExistsError("the #1017 sealed confirmation was already opened")
    status_path = root / TRAINING_STATUS_RELATIVE_PATH
    if status_path.exists():
        status = json.loads(status_path.read_text(encoding="utf-8"))
        _verify_signed_result(status, label="continuation unavailable")
        raise FileExistsError(str(status.get("terminal", "continuation is terminal")))

    training_view = load_continuation_training_view_manifest(root)
    _require_predecessor_identity(training_view)
    if cid_file(root / INHERITED_CHECKPOINT_RELATIVE_PATH) != INHERITED_CHECKPOINT_CID:
        raise ValueError("#1017 inherited checkpoint CID does not reproduce")
    if cid_file(root / TOKENIZER_RELATIVE_PATH) != INHERITED_TOKENIZER_CID:
        raise ValueError("#1017 tokenizer CID does not reproduce")
    run_contract = build_continuation_run_contract(training_view, config)
    run_contract_cid = cid_bytes(canonical_json_bytes(run_contract))

    checkpoint_dir = root / "checkpoints"
    latest_path = checkpoint_dir / "latest.pt"
    best_path = checkpoint_dir / "best.pt"
    elapsed_ledger_path = root / ELAPSED_LEDGER_RELATIVE_PATH
    if not resume and (
        latest_path.exists() or best_path.exists() or elapsed_ledger_path.exists()
    ):
        raise FileExistsError("#1017 checkpoints exist; use --resume for the same run")

    train_store = TokenStore(root / TOKEN_RELATIVE_PATHS["train"])
    dev_store = TokenStore(root / TOKEN_RELATIVE_PATHS["dev"])
    if len(train_store.tokens) != CONTINUATION_TRAIN_STORE_TOKENS:
        raise ValueError("fresh continuation train store has the wrong exact length")
    if len(dev_store.tokens) != CONTINUATION_DEV_STORE_TOKENS:
        raise ValueError("fresh continuation development store has the wrong exact length")

    device = require_mps(config.seed)
    model = R4SoftmaxForCausalLM().to(device)
    optimizer = torch.optim.AdamW(
        model.parameters(),
        lr=config.minimum_learning_rate,
        betas=(config.adam_beta1, config.adam_beta2),
        eps=config.adam_epsilon,
        weight_decay=config.weight_decay,
    )
    continuation_step = 0
    elapsed_before_resume = 0.0
    best_dev_loss = math.inf
    development_candidates: list[dict[str, Any]] = []
    elapsed_ledger_step = 0
    started = time.monotonic()

    if resume:
        resume_path = _select_resume_checkpoint(
            latest_path=latest_path,
            best_path=best_path,
            run_contract_cid=run_contract_cid,
            config=config,
        )
        checkpoint = _load_continuation_checkpoint(
            resume_path,
            model=model,
            optimizer=optimizer,
            device=device,
            run_contract_cid=run_contract_cid,
            config=config,
        )
        continuation_step = int(checkpoint["continuation_optimizer_step"])
        elapsed_ledger = _load_elapsed_ledger(
            root,
            run_contract_cid=run_contract_cid,
            config=config,
        )
        elapsed_ledger_step = int(elapsed_ledger["continuation_optimizer_step"])
        if elapsed_ledger_step < continuation_step:
            raise ValueError("continuation elapsed ledger trails the resume checkpoint")
        elapsed_before_resume = max(
            float(checkpoint["elapsed_continuation_seconds"]),
            float(elapsed_ledger["elapsed_continuation_seconds"]),
        )
        best_dev_loss = float(checkpoint["best_dev_loss"])
        development_candidates = list(checkpoint.get("development_candidates", []))
        if elapsed_before_resume >= config.wall_ceiling_seconds:
            _write_unavailable_mps_budget(
                root,
                run_contract_cid=run_contract_cid,
                continuation_step=elapsed_ledger_step,
                elapsed_seconds=elapsed_before_resume,
                config=config,
            )
            raise RuntimeError("UNAVAILABLE_MPS_BUDGET")
    else:
        _load_inherited_checkpoint(
            root / INHERITED_CHECKPOINT_RELATIVE_PATH,
            model=model,
            optimizer=optimizer,
            device=device,
        )
        initial_dev_loss = evaluate(model, dev_store, device, config.batch_size)
        best_dev_loss = initial_dev_loss
        development_candidates = [
            {
                "continuation_step": 0,
                "cumulative_optimizer_step": INHERITED_OPTIMIZER_STEP,
                "cumulative_train_tokens": INHERITED_TRAIN_TOKENS,
                "development_loss": initial_dev_loss,
            }
        ]
        initial_elapsed = time.monotonic() - started
        _write_elapsed_ledger(
            root,
            run_contract_cid=run_contract_cid,
            continuation_step=0,
            elapsed_seconds=initial_elapsed,
            config=config,
        )
        if initial_elapsed >= config.wall_ceiling_seconds:
            _save_continuation_checkpoint(
                latest_path,
                model=model,
                optimizer=optimizer,
                continuation_step=0,
                elapsed_continuation_seconds=initial_elapsed,
                best_dev_loss=best_dev_loss,
                development_candidates=development_candidates,
                run_contract=run_contract,
                run_contract_cid=run_contract_cid,
                config=config,
            )
            _write_unavailable_mps_budget(
                root,
                run_contract_cid=run_contract_cid,
                continuation_step=0,
                elapsed_seconds=initial_elapsed,
                config=config,
            )
            raise RuntimeError("UNAVAILABLE_MPS_BUDGET")
        _save_continuation_checkpoint(
            best_path,
            model=model,
            optimizer=optimizer,
            continuation_step=0,
            elapsed_continuation_seconds=initial_elapsed,
            best_dev_loss=best_dev_loss,
            development_candidates=development_candidates,
            run_contract=run_contract,
            run_contract_cid=run_contract_cid,
            config=config,
        )

    for step in range(continuation_step + 1, config.optimizer_steps + 1):
        model.train()
        optimizer.zero_grad(set_to_none=True)
        accumulated_loss = 0.0
        for accumulation in range(config.gradient_accumulation_steps):
            batch_index = continuation_batch_index(step, accumulation, config)
            inputs, targets = train_store.random_batch(
                seed=config.seed,
                batch_index=batch_index,
                batch_size=config.batch_size,
            )
            inputs = inputs.to(device)
            targets = targets.to(device)
            output = model(inputs, targets)
            assert output.loss is not None
            (output.loss / config.gradient_accumulation_steps).backward()
            accumulated_loss += float(output.loss.detach().cpu())
        torch.nn.utils.clip_grad_norm_(model.parameters(), config.gradient_clip)
        learning_rate = phase_two_learning_rate(step, config)
        for group in optimizer.param_groups:
            group["lr"] = learning_rate
        optimizer.step()
        continuation_step = step
        elapsed_total = elapsed_before_resume + (time.monotonic() - started)
        elapsed_ledger_step = max(elapsed_ledger_step, continuation_step)
        _write_elapsed_ledger(
            root,
            run_contract_cid=run_contract_cid,
            continuation_step=elapsed_ledger_step,
            elapsed_seconds=elapsed_total,
            config=config,
        )

        should_evaluate = step % config.evaluation_interval == 0 or step == config.optimizer_steps
        if should_evaluate and elapsed_total < config.wall_ceiling_seconds:
            dev_loss = evaluate(model, dev_store, device, config.batch_size)
            elapsed_total = elapsed_before_resume + (time.monotonic() - started)
            _write_elapsed_ledger(
                root,
                run_contract_cid=run_contract_cid,
                continuation_step=elapsed_ledger_step,
                elapsed_seconds=elapsed_total,
                config=config,
            )
            development_candidates.append(
                {
                    "continuation_step": step,
                    "cumulative_optimizer_step": INHERITED_OPTIMIZER_STEP + step,
                    "cumulative_train_tokens": INHERITED_TRAIN_TOKENS
                    + step * config.tokens_per_optimizer_step,
                    "development_loss": dev_loss,
                }
            )
            if elapsed_total < config.wall_ceiling_seconds and dev_loss < best_dev_loss:
                best_dev_loss = dev_loss
                _save_continuation_checkpoint(
                    best_path,
                    model=model,
                    optimizer=optimizer,
                    continuation_step=step,
                    elapsed_continuation_seconds=elapsed_total,
                    best_dev_loss=best_dev_loss,
                    development_candidates=development_candidates,
                    run_contract=run_contract,
                    run_contract_cid=run_contract_cid,
                    config=config,
                )
            print(
                f"continuation_step={step}/{config.optimizer_steps} "
                f"train_loss={accumulated_loss / config.gradient_accumulation_steps:.6f} "
                f"dev_loss={dev_loss:.6f} best_dev_loss={best_dev_loss:.6f} "
                f"lr={learning_rate:.8f}",
                flush=True,
            )

        if elapsed_total >= config.wall_ceiling_seconds:
            _save_continuation_checkpoint(
                latest_path,
                model=model,
                optimizer=optimizer,
                continuation_step=step,
                elapsed_continuation_seconds=elapsed_total,
                best_dev_loss=best_dev_loss,
                development_candidates=development_candidates,
                run_contract=run_contract,
                run_contract_cid=run_contract_cid,
                config=config,
            )
            _write_unavailable_mps_budget(
                root,
                run_contract_cid=run_contract_cid,
                continuation_step=step,
                elapsed_seconds=elapsed_total,
                config=config,
            )
            raise RuntimeError("UNAVAILABLE_MPS_BUDGET")

        if step % config.checkpoint_interval == 0 or step == config.optimizer_steps:
            _save_continuation_checkpoint(
                latest_path,
                model=model,
                optimizer=optimizer,
                continuation_step=step,
                elapsed_continuation_seconds=elapsed_total,
                best_dev_loss=best_dev_loss,
                development_candidates=development_candidates,
                run_contract=run_contract,
                run_contract_cid=run_contract_cid,
                config=config,
            )

    _sync_mps()
    elapsed = elapsed_before_resume + (time.monotonic() - started)
    selected = _load_continuation_checkpoint(
        best_path,
        model=model,
        optimizer=None,
        device=device,
        run_contract_cid=run_contract_cid,
        config=config,
    )
    selected_step = int(selected["continuation_optimizer_step"])
    selected_dev_loss = evaluate(model, dev_store, device, config.batch_size)
    elapsed = elapsed_before_resume + (time.monotonic() - started)
    elapsed_ledger = _write_elapsed_ledger(
        root,
        run_contract_cid=run_contract_cid,
        continuation_step=CONTINUATION_OPTIMIZER_STEPS,
        elapsed_seconds=elapsed,
        config=config,
    )
    if elapsed >= config.wall_ceiling_seconds:
        _write_unavailable_mps_budget(
            root,
            run_contract_cid=run_contract_cid,
            continuation_step=CONTINUATION_OPTIMIZER_STEPS,
            elapsed_seconds=elapsed,
            config=config,
        )
        raise RuntimeError("UNAVAILABLE_MPS_BUDGET")
    if not math.isclose(
        selected_dev_loss,
        float(selected["best_dev_loss"]),
        rel_tol=0.0,
        abs_tol=1e-6,
    ):
        raise RuntimeError("selected fresh-development loss does not replay")
    expected_candidate_steps = _expected_development_candidate_steps(
        CONTINUATION_OPTIMIZER_STEPS, config
    )
    candidate_steps = [
        int(candidate["continuation_step"]) for candidate in development_candidates
    ]
    if candidate_steps != expected_candidate_steps:
        raise RuntimeError("fresh-development selection ledger is incomplete or reordered")
    selected_candidate = min(
        development_candidates,
        key=lambda candidate: (
            float(candidate["development_loss"]),
            int(candidate["continuation_step"]),
        ),
    )
    if int(selected_candidate["continuation_step"]) != selected_step or not math.isclose(
        float(selected_candidate["development_loss"]),
        selected_dev_loss,
        rel_tol=0.0,
        abs_tol=1e-6,
    ):
        raise RuntimeError("selected checkpoint is not the frozen development minimum")

    result: dict[str, Any] = {
        "schema": CONTINUATION_RESULT_SCHEMA,
        "terminal": "FINAL_CHECKPOINT_FROZEN_CONFIRMATION_UNOPENED",
        "issue": ISSUE,
        "continuation_dataset_manifest_cid": training_view[
            "continuation_dataset_manifest_cid"
        ],
        "continuation_training_view_manifest_cid": training_view["manifest_cid"],
        "split_policy_cid": training_view["split_policy_cid"],
        "inherited_checkpoint_cid": INHERITED_CHECKPOINT_CID,
        "run_contract": run_contract,
        "run_contract_cid": run_contract_cid,
        "optimizer_steps_completed": CONTINUATION_OPTIMIZER_STEPS,
        "continuation_train_tokens": CONTINUATION_TRAIN_TOKENS,
        "cumulative_train_tokens": CUMULATIVE_TRAIN_TOKENS,
        "selected_checkpoint_continuation_step": selected_step,
        "selected_checkpoint_cumulative_step": INHERITED_OPTIMIZER_STEP + selected_step,
        "selected_checkpoint_cumulative_tokens": INHERITED_TRAIN_TOKENS
        + selected_step * config.tokens_per_optimizer_step,
        "selected_dev_loss": selected_dev_loss,
        "development_selection_candidates": development_candidates,
        "elapsed_continuation_seconds": elapsed,
        "elapsed_ledger_result_cid": elapsed_ledger["result_cid"],
        "sealed_confirmation_status": "UNOPENED",
        "attention_off_executions": 0,
    }
    result["result_cid"] = cid_bytes(canonical_json_bytes(result))
    atomic_write_json(root / TRAINING_RESULT_RELATIVE_PATH, result)

    selected_checkpoint_cid = cid_file(best_path)
    export = export_hugging_face_snapshot(
        model,
        output_dir=root / "export",
        tokenizer_path=root / TOKENIZER_RELATIVE_PATH,
        training_result=result,
        dataset_manifest_cid=str(training_view["continuation_dataset_manifest_cid"]),
        training_view_manifest_cid=str(training_view["manifest_cid"]),
        split_policy_cid=str(training_view["split_policy_cid"]),
        run_contract_cid=run_contract_cid,
        selected_checkpoint_cid=selected_checkpoint_cid,
    )
    if export["tokenizer_cid"] != INHERITED_TOKENIZER_CID:
        raise RuntimeError("selected export changed the inherited tokenizer")
    prefix = _write_enabled_prefix_fixture(
        root,
        model=model,
        dev_store=dev_store,
        device=device,
        weights_cid=str(export["weights_cid"]),
    )

    selection_payload: dict[str, Any] = {
        "schema": CONTINUATION_SELECTION_SCHEMA,
        "issue": ISSUE,
        "continuation_dataset_manifest_cid": training_view[
            "continuation_dataset_manifest_cid"
        ],
        "continuation_training_view_manifest_cid": training_view["manifest_cid"],
        "split_policy_cid": training_view["split_policy_cid"],
        "inherited_checkpoint_cid": INHERITED_CHECKPOINT_CID,
        "run_contract_cid": run_contract_cid,
        "selected_checkpoint_cid": selected_checkpoint_cid,
        "selected_checkpoint_continuation_step": selected_step,
        "selected_dev_loss": selected_dev_loss,
        "export_manifest_cid": export["manifest_cid"],
        "weights_cid": export["weights_cid"],
        "tokenizer_cid": export["tokenizer_cid"],
        "training_result_cid": result["result_cid"],
        "elapsed_ledger_result_cid": elapsed_ledger["result_cid"],
        "python_enabled_prefix_result_cid": prefix["result_cid"],
        "enabled_parity_status": "AWAITING_RUST",
        "sealed_confirmation_status": "UNOPENED_BEFORE_THIS_MANIFEST",
        "attention_off_executions": 0,
    }
    return write_bound_manifest(
        selection_path,
        selection_payload,
        artifact_root=root,
        relative_paths=sorted(_SELECTION_ARTIFACT_PATHS),
    )


def _validate_enabled_rust_report(
    report: dict[str, Any],
    *,
    selection: dict[str, Any],
    prefix: dict[str, Any],
) -> None:
    if report.get("schema") != RUST_ENABLED_QUALIFICATION_SCHEMA:
        raise ValueError("unexpected enabled-only Rust qualification schema")
    if report.get("issue") != ISSUE or report.get("qualification_passed") is not True:
        raise ValueError("enabled-only Rust qualification did not pass for #1017")
    if "attention_off" in report or "attention_off_prefix_parity" in report:
        raise ValueError("#1017 Rust qualification contains a prohibited attention-off arm")
    if report.get("attention_off_executions") != 0:
        raise ValueError("#1017 Rust qualification executed attention-off")
    provenance = report.get("provenance")
    if not isinstance(provenance, dict):
        raise ValueError("enabled Rust qualification has no provenance")
    expected_provenance = {
        "dataset_manifest_cid": selection["continuation_dataset_manifest_cid"],
        "training_view_manifest_cid": selection[
            "continuation_training_view_manifest_cid"
        ],
        "split_policy_cid": selection["split_policy_cid"],
        "run_contract_cid": selection["run_contract_cid"],
        "selected_checkpoint_cid": selection["selected_checkpoint_cid"],
        "weights_cid": selection["weights_cid"],
        "tokenizer_cid": selection["tokenizer_cid"],
    }
    for field, expected in expected_provenance.items():
        if provenance.get(field) != expected:
            raise ValueError(f"enabled Rust qualification {field} differs")
    evaluation_input = report.get("evaluation_input")
    if not isinstance(evaluation_input, dict):
        raise ValueError("enabled Rust qualification has no input binding")
    if evaluation_input.get("python_prefix_result_cid") != prefix["result_cid"]:
        raise ValueError("enabled Rust qualification used a different Python prefix")
    if evaluation_input.get("token_store_cid") != prefix["token_store_cid"]:
        raise ValueError("enabled Rust qualification used a different development store")
    if evaluation_input.get("prefix_token_ids") != prefix["prefix_token_ids"]:
        raise ValueError("enabled Rust qualification used different prefix tokens")

    parity = report.get("enabled_prefix_parity")
    if not isinstance(parity, dict):
        raise ValueError("enabled Rust qualification has no parity result")
    if (
        parity.get("passed") is not True
        or parity.get("identical_top1") is not True
        or parity.get("maximum_absolute_logit_delta_within_limit") is not True
        or float(parity.get("maximum_absolute_logit_delta", math.inf))
        >= PREFIX_LOGIT_ABS_TOLERANCE
    ):
        raise ValueError("enabled Python/Rust prefix parity failed")
    enabled = report.get("enabled")
    audit = enabled.get("audit") if isinstance(enabled, dict) else None
    if not isinstance(audit, dict):
        raise ValueError("enabled Rust qualification has no execution audit")
    if (
        audit.get("selected_layer_count") != FROZEN_MODEL_CONFIG.num_hidden_layers
        or audit.get("all_layers_selected") is not True
        or audit.get("causal_audits_exact") != 1
        or audit.get("projection_audits_exact") != 1
        or audit.get("r4_audits_exact") != 1
        or audit.get("output_policy_audits_exact") != 1
        or audit.get("future_reads") != 0
        or audit.get("zeroed_applications") != 0
    ):
        raise ValueError("enabled Rust qualification audit is not exact all-layer R4")
    source = report.get("source_read_audit")
    if not isinstance(source, dict) or any(
        source.get(field) != 0
        for field in ("provider_calls", "ollama_calls", "prior_trace_reads")
    ):
        raise ValueError("enabled Rust qualification used a prohibited external source")


def admit_enabled_prefix_parity(root: Path, rust_report_path: Path) -> dict[str, Any]:
    """Validate and bind the sole enabled-only Rust parity report before reveal."""
    admission_path = root / ENABLED_PARITY_ADMISSION_RELATIVE_PATH
    if admission_path.exists():
        raise FileExistsError("enabled-only parity admission is already frozen")
    selection = _load_frozen_continuation_selection(root)
    prefix = json.loads(
        (root / PYTHON_ENABLED_PREFIX_RELATIVE_PATH).read_text(encoding="utf-8")
    )
    report_bytes = rust_report_path.read_bytes()
    report = json.loads(report_bytes)
    if not isinstance(report, dict):
        raise ValueError("enabled Rust qualification must be a JSON object")
    _validate_enabled_rust_report(report, selection=selection, prefix=prefix)
    canonical_report_path = root / RUST_ENABLED_QUALIFICATION_RELATIVE_PATH
    atomic_write(canonical_report_path, report_bytes)
    payload: dict[str, Any] = {
        "schema": ENABLED_PARITY_ADMISSION_SCHEMA,
        "issue": ISSUE,
        "selection_manifest_cid": selection["manifest_cid"],
        "selected_checkpoint_cid": selection["selected_checkpoint_cid"],
        "weights_cid": selection["weights_cid"],
        "python_enabled_prefix_result_cid": prefix["result_cid"],
        "rust_qualification_decision_cid": report.get("decision_cid"),
        "rust_qualification_report_cid": cid_file(canonical_report_path),
        "qualification_passed": True,
        "attention_off_executions": 0,
        "sealed_confirmation_status": "UNOPENED",
    }
    return write_bound_manifest(
        admission_path,
        payload,
        artifact_root=root,
        relative_paths=[
            str(SELECTION_RELATIVE_PATH),
            str(PYTHON_ENABLED_PREFIX_RELATIVE_PATH),
            str(RUST_ENABLED_QUALIFICATION_RELATIVE_PATH),
        ],
    )


def load_enabled_parity_admission(root: Path) -> dict[str, Any]:
    """Reproduce the enabled-only parity admission without opening sealed data."""
    selection = _load_frozen_continuation_selection(root)
    admission = verify_bound_manifest(
        root / ENABLED_PARITY_ADMISSION_RELATIVE_PATH, artifact_root=root
    )
    if admission.get("schema") != ENABLED_PARITY_ADMISSION_SCHEMA:
        raise ValueError("unsupported enabled parity admission")
    expected_paths = {
        str(SELECTION_RELATIVE_PATH),
        str(PYTHON_ENABLED_PREFIX_RELATIVE_PATH),
        str(RUST_ENABLED_QUALIFICATION_RELATIVE_PATH),
    }
    if _manifest_artifact_paths(admission, label="enabled parity admission") != expected_paths:
        raise ValueError("enabled parity admission binds unexpected artifacts")
    if (
        admission.get("selection_manifest_cid") != selection["manifest_cid"]
        or admission.get("selected_checkpoint_cid") != selection["selected_checkpoint_cid"]
        or admission.get("weights_cid") != selection["weights_cid"]
        or admission.get("qualification_passed") is not True
        or admission.get("attention_off_executions") != 0
        or admission.get("sealed_confirmation_status") != "UNOPENED"
    ):
        raise ValueError("enabled parity admission identity or decision differs")
    prefix = json.loads(
        (root / PYTHON_ENABLED_PREFIX_RELATIVE_PATH).read_text(encoding="utf-8")
    )
    report = json.loads(
        (root / RUST_ENABLED_QUALIFICATION_RELATIVE_PATH).read_text(encoding="utf-8")
    )
    _validate_enabled_rust_report(report, selection=selection, prefix=prefix)
    if admission.get("rust_qualification_report_cid") != cid_file(
        root / RUST_ENABLED_QUALIFICATION_RELATIVE_PATH
    ):
        raise ValueError("enabled Rust qualification report CID differs")
    return admission


def _write_reveal_opened_marker(
    root: Path,
    *,
    selection: dict[str, Any],
    parity_admission: dict[str, Any],
) -> dict[str, Any]:
    path = root / REVEAL_OPENED_RELATIVE_PATH
    if path.exists() or (root / REVEAL_RESULT_RELATIVE_PATH).exists() or (
        root / REVEAL_MANIFEST_RELATIVE_PATH
    ).exists():
        raise FileExistsError("fresh sealed confirmation was already opened")
    marker: dict[str, Any] = {
        "schema": CONTINUATION_REVEAL_OPENED_SCHEMA,
        "issue": ISSUE,
        "terminal": "SEALED_CONFIRMATION_OPEN_INITIATED",
        "selection_manifest_cid": selection["manifest_cid"],
        "selected_checkpoint_cid": selection["selected_checkpoint_cid"],
        "enabled_parity_admission_manifest_cid": parity_admission["manifest_cid"],
        "continuation_dataset_manifest_cid": selection[
            "continuation_dataset_manifest_cid"
        ],
        "repeat_reveal_permitted": False,
    }
    marker["result_cid"] = cid_bytes(canonical_json_bytes(marker))
    atomic_write_json(path, marker)
    return marker


def reveal_continuation(root: Path) -> dict[str, Any]:
    """Open the fresh confirmation once and evaluate enabled attention only."""
    selection = _load_frozen_continuation_selection(root)
    parity_admission = load_enabled_parity_admission(root)
    training_result = json.loads(
        (root / TRAINING_RESULT_RELATIVE_PATH).read_text(encoding="utf-8")
    )
    run_contract = training_result["run_contract"]
    run_contract_cid = str(training_result["run_contract_cid"])
    if cid_bytes(canonical_json_bytes(run_contract)) != run_contract_cid:
        raise ValueError("continuation run contract does not reproduce before reveal")
    if run_contract.get("trainer_implementation", {}).get("tree_cid") != (
        trainer_implementation_contract()["tree_cid"]
    ):
        raise ValueError("trainer implementation changed after #1017 selection")

    config = ContinuationConfig()
    device = require_mps(config.seed)
    model = R4SoftmaxForCausalLM().to(device)
    _load_continuation_checkpoint(
        root / "checkpoints/best.pt",
        model=model,
        optimizer=None,
        device=device,
        run_contract_cid=run_contract_cid,
        config=config,
    )

    # This durable marker is written before the first full-manifest/test read.
    # A crash after this point remains an honest one-time reveal, not permission
    # to open the confirmation population again.
    opened = _write_reveal_opened_marker(
        root, selection=selection, parity_admission=parity_admission
    )
    denial = open_sealed_confirmation(root)
    dataset = load_continuation_dataset_manifest(root)
    if dataset.get("manifest_cid") != selection["continuation_dataset_manifest_cid"]:
        raise ValueError("revealed continuation population differs from selection")
    if dataset.get("split_policy_cid") != selection["split_policy_cid"]:
        raise ValueError("revealed continuation split policy differs from selection")

    test_store = TokenStore(root / TOKEN_RELATIVE_PATHS["test"])
    if len(test_store.tokens) != CONTINUATION_TEST_STORE_TOKENS:
        raise ValueError("fresh sealed test store has the wrong exact length")
    enabled_test_loss = evaluate(model, test_store, device, config.batch_size)
    tokenizer = Tokenizer.from_file(str(root / TOKENIZER_RELATIVE_PATH))
    prompts = _load_sealed_prompt_fixture(root / SEALED_PROMPT_RELATIVE_PATH)
    prompt_records: list[dict[str, Any]] = []
    for index, prompt_record in enumerate(prompts):
        prompt = list(prompt_record["token_ids"])
        text = tokenizer.decode(prompt, skip_special_tokens=True)
        if text != prompt_record["text"]:
            raise ValueError("fresh sealed prompt text does not reproduce")
        if tokenizer.encode(text, add_special_tokens=False).ids != prompt:
            raise ValueError("fresh sealed prompt token IDs do not reproduce")
        text.encode("utf-8", errors="strict")
        prompt_records.append(
            {
                "index": index,
                "story_cid": prompt_record["story_cid"],
                "seed": 2014 + index,
                "prompt_token_ids": prompt,
                "prompt_tokens": SEALED_PROMPT_TOKENS_PER_STORY,
                "prompt_text": text,
            }
        )

    passed = enabled_test_loss < SEALED_TEST_LOSS_CEILING
    result: dict[str, Any] = {
        "schema": CONTINUATION_REVEAL_RESULT_SCHEMA,
        "terminal": "PASS_ENABLED_NLL" if passed else "FAIL_ENABLED_NLL",
        "issue": ISSUE,
        "selection_manifest_cid": selection["manifest_cid"],
        "selected_checkpoint_cid": selection["selected_checkpoint_cid"],
        "continuation_dataset_manifest_cid": dataset["manifest_cid"],
        "continuation_training_view_manifest_cid": selection[
            "continuation_training_view_manifest_cid"
        ],
        "split_policy_cid": dataset["split_policy_cid"],
        "weights_cid": selection["weights_cid"],
        "tokenizer_cid": selection["tokenizer_cid"],
        "enabled_parity_admission_manifest_cid": parity_admission["manifest_cid"],
        "reveal_opened_result_cid": opened["result_cid"],
        "sealed_denial_result_cid": denial["result_cid"],
        "enabled_sealed_test_loss": enabled_test_loss,
        "sealed_test_loss_ceiling": SEALED_TEST_LOSS_CEILING,
        "sealed_test_loss_passed": passed,
        "attention_off_executions": 0,
        "sealed_test_store_token_ids": len(test_store.tokens),
        "sealed_test_scored_next_tokens": test_store.scored_next_tokens,
        "sealed_prompt_token_ids": SEALED_PROMPT_TOKEN_COUNT,
        "total_revealed_test_token_ids": len(test_store.tokens)
        + SEALED_PROMPT_TOKEN_COUNT,
        "prompts": prompt_records,
        "autonomous_generation_status": "NOT_RUN_RUST_SEEDED_SAMPLER_REQUIRED",
        "quality_failure_is_frozen_not_retriable": not passed,
    }
    if len(prompt_records) != SEALED_PROMPT_COUNT:
        raise RuntimeError("fresh reveal did not retain exactly five prompts")
    if result["total_revealed_test_token_ids"] != 250_000:
        raise RuntimeError("fresh reveal exceeded or undershot its exact budget")
    result["result_cid"] = cid_bytes(canonical_json_bytes(result))
    atomic_write_json(root / REVEAL_RESULT_RELATIVE_PATH, result)
    return write_bound_manifest(
        root / REVEAL_MANIFEST_RELATIVE_PATH,
        {
            "schema": CONTINUATION_REVEAL_MANIFEST_SCHEMA,
            "issue": ISSUE,
            "terminal": result["terminal"],
            "selection_manifest_cid": selection["manifest_cid"],
            "selected_checkpoint_cid": selection["selected_checkpoint_cid"],
            "continuation_dataset_manifest_cid": dataset["manifest_cid"],
            "continuation_training_view_manifest_cid": selection[
                "continuation_training_view_manifest_cid"
            ],
            "split_policy_cid": dataset["split_policy_cid"],
            "weights_cid": selection["weights_cid"],
            "tokenizer_cid": selection["tokenizer_cid"],
            "enabled_parity_admission_manifest_cid": parity_admission["manifest_cid"],
            "reveal_opened_result_cid": opened["result_cid"],
            "sealed_denial_result_cid": denial["result_cid"],
            "reveal_result_cid": result["result_cid"],
            "enabled_sealed_test_loss": enabled_test_loss,
            "sealed_test_loss_passed": passed,
            "attention_off_executions": 0,
            "sealed_test_store_token_ids": len(test_store.tokens),
            "sealed_test_scored_next_tokens": test_store.scored_next_tokens,
            "sealed_prompt_token_ids": SEALED_PROMPT_TOKEN_COUNT,
            "total_revealed_test_token_ids": 250_000,
        },
        artifact_root=root,
        relative_paths=[
            str(REVEAL_OPENED_RELATIVE_PATH),
            str(REVEAL_RESULT_RELATIVE_PATH),
            str(TOKEN_RELATIVE_PATHS["test"]),
            str(INDEX_RELATIVE_PATHS["test"]),
            str(SEALED_PROMPT_RELATIVE_PATH),
        ],
    )
