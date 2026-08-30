"""MPS-only overfit gate, bounded main training, selection, and generation."""

from __future__ import annotations

import importlib.metadata
import json
import math
import os
import struct
import sys
import time
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Any, Iterator

import numpy as np
import torch
from blake3 import blake3
from tokenizers import Tokenizer
from torch import Tensor

from .admission import MAIN_ADMISSION_MANIFEST_PATH, load_main_admission
from .constants import (
    CHECKPOINT_SCHEMA,
    EXPORT_MANIFEST_SCHEMA,
    FROZEN_MODEL_CONFIG,
    LLAMA2_C_REPOSITORY,
    LLAMA2_C_REVISION,
    SEALED_PROMPT_COUNT,
    SEALED_PROMPT_TOKEN_COUNT,
    SEALED_PROMPT_TOKENS_PER_STORY,
    SELECTION_MANIFEST_SCHEMA,
    SMOKE_SCHEMA,
    TRAIN_TOKEN_CAP,
)
from .data import (
    INDEX_RELATIVE_PATHS,
    SEALED_PROMPT_RELATIVE_PATH,
    TOKENIZER_RELATIVE_PATH,
    TOKEN_RELATIVE_PATHS,
    load_dataset_manifest,
    load_training_view_manifest,
)
from .export import export_hugging_face_snapshot
from .model import R4SoftmaxForCausalLM
from .provenance import (
    atomic_write_json,
    canonical_json_bytes,
    cid_bytes,
    cid_file,
    trainer_implementation_contract,
    tree_cid,
    verify_bound_manifest,
    write_bound_manifest,
)


@dataclass(frozen=True, slots=True)
class TrainConfig:
    seed: int = 1014
    batch_size: int = 16
    gradient_accumulation_steps: int = 4
    learning_rate: float = 3e-4
    minimum_learning_rate: float = 3e-5
    warmup_steps: int = 100
    weight_decay: float = 0.1
    adam_beta1: float = 0.9
    adam_beta2: float = 0.95
    gradient_clip: float = 1.0
    evaluation_interval: int = 100
    checkpoint_interval: int = 100
    max_train_tokens: int = TRAIN_TOKEN_CAP
    wall_ceiling_seconds: float = 6 * 60 * 60

    def validate(self) -> None:
        if self.seed < 0 or self.batch_size < 1 or self.gradient_accumulation_steps < 1:
            raise ValueError("seed must be nonnegative and batch/accumulation positive")
        if self.max_train_tokens < self.tokens_per_optimizer_step:
            raise ValueError("train-token budget cannot complete one optimizer step")
        if self.max_train_tokens > TRAIN_TOKEN_CAP:
            raise ValueError("train-token budget exceeds #1014's 30M cap")
        if not 0 < self.minimum_learning_rate <= self.learning_rate:
            raise ValueError("invalid learning-rate bounds")
        if self.evaluation_interval < 1 or self.checkpoint_interval < 1:
            raise ValueError("evaluation/checkpoint intervals must be positive")
        if not 0 < self.wall_ceiling_seconds <= 6 * 60 * 60:
            raise ValueError("#1014's main campaign wall ceiling is at most six hours")

    @property
    def tokens_per_microbatch(self) -> int:
        return self.batch_size * FROZEN_MODEL_CONFIG.max_position_embeddings

    @property
    def tokens_per_optimizer_step(self) -> int:
        return self.tokens_per_microbatch * self.gradient_accumulation_steps

    @property
    def optimizer_steps(self) -> int:
        return self.max_train_tokens // self.tokens_per_optimizer_step

    @property
    def effective_train_tokens(self) -> int:
        return self.optimizer_steps * self.tokens_per_optimizer_step

    def as_contract(self) -> dict[str, Any]:
        self.validate()
        contract = asdict(self)
        contract["optimizer_steps"] = self.optimizer_steps
        contract["effective_train_tokens"] = self.effective_train_tokens
        return contract


class TokenStore:
    """Read-only little-endian uint16 store with deterministic batch access."""

    def __init__(self, path: Path) -> None:
        if path.stat().st_size % 2:
            raise ValueError(f"odd byte length for uint16 token store: {path}")
        self.path = path
        self.tokens = np.memmap(path, mode="r", dtype="<u2")
        if len(self.tokens) <= FROZEN_MODEL_CONFIG.max_position_embeddings:
            raise ValueError(f"token store too short: {path}")

    def _pair(self, start: int) -> tuple[np.ndarray, np.ndarray]:
        context = FROZEN_MODEL_CONFIG.max_position_embeddings
        chunk = np.asarray(self.tokens[start : start + context + 1], dtype=np.int64)
        if len(chunk) != context + 1:
            raise ValueError("sample crosses token-store boundary")
        return chunk[:-1].copy(), chunk[1:].copy()

    def random_batch(self, *, seed: int, batch_index: int, batch_size: int) -> tuple[Tensor, Tensor]:
        context = FROZEN_MODEL_CONFIG.max_position_embeddings
        maximum_start = len(self.tokens) - context - 1
        inputs: list[np.ndarray] = []
        targets: list[np.ndarray] = []
        for lane in range(batch_size):
            material = struct.pack(">QQQ", seed, batch_index, lane)
            start = int.from_bytes(blake3(material).digest(), "big") % (maximum_start + 1)
            input_ids, target_ids = self._pair(start)
            inputs.append(input_ids)
            targets.append(target_ids)
        return torch.from_numpy(np.stack(inputs)), torch.from_numpy(np.stack(targets))

    def sequential_batches(self, batch_size: int) -> Iterator[tuple[Tensor, Tensor]]:
        context = FROZEN_MODEL_CONFIG.max_position_embeddings
        sequence_count = (len(self.tokens) - 1) // context
        for base in range(0, sequence_count, batch_size):
            inputs: list[np.ndarray] = []
            targets: list[np.ndarray] = []
            for index in range(base, min(base + batch_size, sequence_count)):
                input_ids, target_ids = self._pair(index * context)
                inputs.append(input_ids)
                targets.append(target_ids)
            yield torch.from_numpy(np.stack(inputs)), torch.from_numpy(np.stack(targets))

    def first_sequences(self, count: int) -> tuple[Tensor, Tensor]:
        context = FROZEN_MODEL_CONFIG.max_position_embeddings
        if len(self.tokens) < count * (context + 1):
            raise ValueError("token store cannot provide requested fixed sequences")
        inputs: list[np.ndarray] = []
        targets: list[np.ndarray] = []
        for index in range(count):
            input_ids, target_ids = self._pair(index * (context + 1))
            inputs.append(input_ids)
            targets.append(target_ids)
        return torch.from_numpy(np.stack(inputs)), torch.from_numpy(np.stack(targets))

    @property
    def scored_next_tokens(self) -> int:
        context = FROZEN_MODEL_CONFIG.max_position_embeddings
        return ((len(self.tokens) - 1) // context) * context


def require_mps(seed: int) -> torch.device:
    """Refuse CPU fallback: #1014's wall contract is specifically M1 Metal."""
    fallback = os.environ.get("PYTORCH_ENABLE_MPS_FALLBACK", "0")
    if fallback not in {"", "0"}:
        raise RuntimeError("PYTORCH_ENABLE_MPS_FALLBACK must be unset or 0")
    if not torch.backends.mps.is_built() or not torch.backends.mps.is_available():
        raise RuntimeError("PyTorch MPS is unavailable; refusing slow CPU fallback")
    torch.use_deterministic_algorithms(True)
    torch.manual_seed(seed)
    if hasattr(torch.mps, "manual_seed"):
        torch.mps.manual_seed(seed)
    return torch.device("mps")


def _sync_mps() -> None:
    if hasattr(torch, "mps"):
        torch.mps.synchronize()


@torch.no_grad()
def evaluate(
    model: R4SoftmaxForCausalLM,
    store: TokenStore,
    device: torch.device,
    batch_size: int,
    *,
    attention_off: bool = False,
) -> float:
    model.eval()
    total_loss = 0.0
    total_tokens = 0
    for inputs, targets in store.sequential_batches(batch_size):
        inputs = inputs.to(device)
        targets = targets.to(device)
        output = model(inputs, targets, attention_off=attention_off)
        assert output.loss is not None
        tokens = targets.numel()
        total_loss += float(output.loss.detach().cpu()) * tokens
        total_tokens += tokens
    if total_tokens == 0:
        raise RuntimeError("evaluation store has no complete context")
    return total_loss / total_tokens


def _evaluate_fixed(
    model: R4SoftmaxForCausalLM,
    inputs: Tensor,
    targets: Tensor,
    device: torch.device,
    batch_size: int,
) -> float:
    model.eval()
    total = 0.0
    tokens = 0
    with torch.no_grad():
        for base in range(0, len(inputs), batch_size):
            batch_inputs = inputs[base : base + batch_size].to(device)
            batch_targets = targets[base : base + batch_size].to(device)
            output = model(batch_inputs, batch_targets)
            assert output.loss is not None
            count = batch_targets.numel()
            total += float(output.loss.detach().cpu()) * count
            tokens += count
    return total / tokens


def _dependency_versions() -> dict[str, str]:
    return {
        name: importlib.metadata.version(name)
        for name in ["blake3", "numpy", "safetensors", "tokenizers", "torch"]
    }


def _tool_root() -> Path:
    return Path(__file__).resolve().parents[2]


def _trainer_implementation_contract() -> dict[str, Any]:
    return trainer_implementation_contract()


def build_run_contract(
    training_view: dict[str, Any], config: TrainConfig, admission: dict[str, Any]
) -> dict[str, Any]:
    lock_path = _tool_root() / "uv.lock"
    if not lock_path.is_file():
        raise FileNotFoundError("uv.lock is required before a run can be frozen")
    return {
        "schema": "uor-r4-softmax-trainer-run/1",
        "architecture_reference": {
            "repository": LLAMA2_C_REPOSITORY,
            "revision": LLAMA2_C_REVISION,
            "license": "MIT",
        },
        "dataset_manifest_cid": training_view["dataset_manifest_cid"],
        "training_view_manifest_cid": training_view["manifest_cid"],
        "split_policy_cid": training_view["split_policy_cid"],
        "main_campaign_admission": {
            "manifest_cid": admission["manifest_cid"],
            "smoke_manifest_cid": admission["smoke_manifest_cid"],
            "smoke_export_manifest_cid": admission["smoke_export_manifest_cid"],
            "rust_qualification_report_cid": admission["rust_qualification_report_cid"],
            "rust_qualification_decision_cid": admission[
                "rust_qualification_decision_cid"
            ],
            "smoke_trainer_implementation_tree_cid": admission[
                "smoke_trainer_implementation_tree_cid"
            ],
            "campaign_trainer_implementation_tree_cid": admission[
                "trainer_implementation_tree_cid"
            ],
            "smoke_reuse_transition_cid": admission["smoke_reuse_transition_cid"],
        },
        "environment": {
            "python": ".".join(map(str, sys.version_info[:3])),
            "dependencies": _dependency_versions(),
            "uv_lock_cid": cid_file(lock_path),
            "device": "mps",
            "cpu_fallback": False,
            "dtype": "float32",
        },
        "trainer_implementation": _trainer_implementation_contract(),
        "model": FROZEN_MODEL_CONFIG.as_contract(),
        "optimization": config.as_contract(),
        "selection": "minimum complete-dev-token mean causal cross-entropy",
        "test_policy": "open test token store only after best checkpoint selection",
        "generation": {
            "owner": "Rust R4 local generator; Python reveal emits prompts but no continuations",
            "prompts": "first 24 content tokens of the five lowest full-story-CID sealed-test stories",
            "new_tokens": 128,
            "selection": "r4-local-top-k-q32-splitmix64/1; temperature 0.8; top-k 40; explicit seed",
            "eos": "stop when selected; otherwise stop after 128 new tokens",
        },
    }


def _learning_rate(step: int, total_steps: int, config: TrainConfig) -> float:
    if step <= config.warmup_steps:
        return config.learning_rate * step / max(1, config.warmup_steps)
    progress = (step - config.warmup_steps) / max(1, total_steps - config.warmup_steps)
    cosine = 0.5 * (1.0 + math.cos(math.pi * min(1.0, progress)))
    return config.minimum_learning_rate + cosine * (config.learning_rate - config.minimum_learning_rate)


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


def _save_checkpoint(
    path: Path,
    *,
    model: R4SoftmaxForCausalLM,
    optimizer: torch.optim.Optimizer,
    optimizer_step: int,
    tokens_seen: int,
    elapsed_training_seconds: float,
    best_dev_loss: float,
    run_contract: dict[str, Any],
    run_contract_cid: str,
) -> None:
    payload = {
        "schema": CHECKPOINT_SCHEMA,
        "run_contract": run_contract,
        "run_contract_cid": run_contract_cid,
        "optimizer_step": optimizer_step,
        "tokens_seen": tokens_seen,
        "elapsed_training_seconds": elapsed_training_seconds,
        "best_dev_loss": best_dev_loss,
        "model": _cpu_tree(model.state_dict()),
        "optimizer": _cpu_tree(optimizer.state_dict()),
        "cpu_rng_state": torch.get_rng_state(),
    }
    path.parent.mkdir(parents=True, exist_ok=True)
    temporary = path.with_name(f".{path.name}.part")
    torch.save(payload, temporary)
    os.replace(temporary, path)


def _optimizer_to_device(optimizer: torch.optim.Optimizer, device: torch.device) -> None:
    for state in optimizer.state.values():
        for key, value in state.items():
            if isinstance(value, Tensor):
                state[key] = value.to(device)


def _load_checkpoint(
    path: Path,
    *,
    model: R4SoftmaxForCausalLM,
    optimizer: torch.optim.Optimizer | None,
    device: torch.device,
    run_contract_cid: str,
) -> dict[str, Any]:
    checkpoint = torch.load(path, map_location="cpu", weights_only=False)
    if checkpoint.get("schema") != CHECKPOINT_SCHEMA:
        raise ValueError("unsupported checkpoint schema")
    if checkpoint.get("run_contract_cid") != run_contract_cid:
        raise ValueError("checkpoint belongs to a different frozen run")
    if cid_bytes(canonical_json_bytes(checkpoint.get("run_contract"))) != run_contract_cid:
        raise ValueError("checkpoint embedded run contract does not reproduce")
    model.load_state_dict(checkpoint["model"], strict=True)
    if optimizer is not None:
        optimizer.load_state_dict(checkpoint["optimizer"])
        _optimizer_to_device(optimizer, device)
    torch.set_rng_state(checkpoint["cpu_rng_state"])
    return checkpoint


_SELECTION_ARTIFACT_PATHS = {
    "admission/main-admission-manifest.json",
    "checkpoints/best.pt",
    "training-result.json",
    "export/config.json",
    "export/model.safetensors",
    "export/tokenizer.json",
    "export/training-result.json",
    "export/export-manifest.json",
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


def _verify_export_payload_file_cids(export_manifest: dict[str, Any], export_root: Path) -> None:
    """Tie export payload identities to both verified records and exact file bytes."""
    records = {
        str(record["path"]): record
        for record in export_manifest["artifacts"]
        if isinstance(record, dict) and isinstance(record.get("path"), str)
    }
    for field, relative_path in (
        ("config_cid", "config.json"),
        ("tokenizer_cid", "tokenizer.json"),
        ("weights_cid", "model.safetensors"),
    ):
        record = records.get(relative_path)
        direct_cid = cid_file(export_root / relative_path)
        if record is None or record.get("cid") != direct_cid:
            raise ValueError(f"verified export artifact record differs from {relative_path}")
        if export_manifest.get(field) != direct_cid:
            raise ValueError(f"export {field} does not match verified {relative_path}")


def _load_frozen_selection(root: Path) -> dict[str, Any]:
    """Strictly reproduce a completed campaign without mutating any artifact."""
    selection = verify_bound_manifest(
        root / "selection" / "selection-manifest.json", artifact_root=root
    )
    if selection.get("schema") != SELECTION_MANIFEST_SCHEMA:
        raise ValueError("unsupported selection manifest schema")
    if _manifest_artifact_paths(selection, label="selection manifest") != _SELECTION_ARTIFACT_PATHS:
        raise ValueError("selection manifest does not bind the exact frozen campaign")
    if selection.get("sealed_test_status") != "UNOPENED_BEFORE_THIS_MANIFEST":
        raise ValueError("selection manifest has an invalid sealed-test boundary")
    if selection.get("selected_checkpoint_cid") != cid_file(root / "checkpoints" / "best.pt"):
        raise ValueError("selected checkpoint CID does not reproduce")

    training_result = json.loads((root / "training-result.json").read_text(encoding="utf-8"))
    if not isinstance(training_result, dict):
        raise ValueError("training result must be a JSON object")
    if (
        training_result.get("schema") != "uor-r4-softmax-trainer-selection-result/1"
        or training_result.get("terminal") != "FINAL_CHECKPOINT_FROZEN_TEST_UNOPENED"
        or training_result.get("sealed_test_status") != "UNOPENED"
    ):
        raise ValueError("training result is not a frozen pre-reveal selection")
    unsigned_result = dict(training_result)
    expected_result_cid = unsigned_result.pop("result_cid", None)
    if expected_result_cid != cid_bytes(canonical_json_bytes(unsigned_result)):
        raise ValueError("training result CID does not reproduce")
    run_contract = training_result.get("run_contract")
    if not isinstance(run_contract, dict):
        raise ValueError("training result has no embedded run contract")
    run_contract_cid = cid_bytes(canonical_json_bytes(run_contract))
    if training_result.get("run_contract_cid") != run_contract_cid:
        raise ValueError("training-result run contract CID does not reproduce")
    if (
        training_result.get("dataset_manifest_cid") != run_contract.get("dataset_manifest_cid")
        or training_result.get("training_view_manifest_cid")
        != run_contract.get("training_view_manifest_cid")
    ):
        raise ValueError("training result and run contract dataset identities differ")

    admission = load_main_admission(root, require_current_trainer=False)
    expected_admission_identity = {
        "manifest_cid": admission["manifest_cid"],
        "smoke_manifest_cid": admission["smoke_manifest_cid"],
        "smoke_export_manifest_cid": admission["smoke_export_manifest_cid"],
        "rust_qualification_report_cid": admission["rust_qualification_report_cid"],
        "rust_qualification_decision_cid": admission["rust_qualification_decision_cid"],
        "smoke_trainer_implementation_tree_cid": admission[
            "smoke_trainer_implementation_tree_cid"
        ],
        "campaign_trainer_implementation_tree_cid": admission[
            "trainer_implementation_tree_cid"
        ],
        "smoke_reuse_transition_cid": admission["smoke_reuse_transition_cid"],
    }
    if run_contract.get("main_campaign_admission") != expected_admission_identity:
        raise ValueError("selected run contract does not bind the admitted smoke/Rust gate")
    if (
        run_contract.get("dataset_manifest_cid") != admission.get("dataset_manifest_cid")
        or run_contract.get("training_view_manifest_cid")
        != admission.get("training_view_manifest_cid")
        or run_contract.get("split_policy_cid") != admission.get("split_policy_cid")
    ):
        raise ValueError("selected run contract and main admission datasets differ")
    trainer_identity = run_contract.get("trainer_implementation")
    if (
        not isinstance(trainer_identity, dict)
        or not isinstance(trainer_identity.get("files"), list)
        or tree_cid(trainer_identity["files"]) != trainer_identity.get("tree_cid")
        or trainer_identity.get("tree_cid")
        != admission.get("trainer_implementation_tree_cid")
    ):
        raise ValueError("selected run used a trainer different from the admitted campaign")

    export_manifest = verify_bound_manifest(
        root / "export" / "export-manifest.json", artifact_root=root / "export"
    )
    if export_manifest.get("schema") != EXPORT_MANIFEST_SCHEMA:
        raise ValueError("unsupported final export manifest schema")
    if _manifest_artifact_paths(export_manifest, label="export manifest") != {
        "config.json",
        "model.safetensors",
        "tokenizer.json",
        "training-result.json",
    }:
        raise ValueError("final export manifest does not bind the exact HF snapshot")
    _verify_export_payload_file_cids(export_manifest, root / "export")
    exported_result = json.loads(
        (root / "export" / "training-result.json").read_text(encoding="utf-8")
    )
    if exported_result != training_result:
        raise ValueError("root and exported training results differ")

    common = {
        "dataset_manifest_cid": training_result.get("dataset_manifest_cid"),
        "training_view_manifest_cid": training_result.get("training_view_manifest_cid"),
        "split_policy_cid": run_contract.get("split_policy_cid"),
        "run_contract_cid": run_contract_cid,
        "selected_checkpoint_cid": selection.get("selected_checkpoint_cid"),
        "training_result_cid": expected_result_cid,
        "weights_cid": export_manifest.get("weights_cid"),
        "tokenizer_cid": export_manifest.get("tokenizer_cid"),
        "export_manifest_cid": export_manifest.get("manifest_cid"),
        "main_admission_manifest_cid": admission.get("manifest_cid"),
    }
    for field, expected in common.items():
        if field in selection and selection.get(field) != expected:
            raise ValueError(f"selection {field} identity mismatch")
    for field in (
        "dataset_manifest_cid",
        "training_view_manifest_cid",
        "split_policy_cid",
        "run_contract_cid",
        "selected_checkpoint_cid",
        "training_result_cid",
        "weights_cid",
        "tokenizer_cid",
    ):
        if export_manifest.get(field) != common[field]:
            raise ValueError(f"export {field} identity mismatch")
    for required in common:
        if required not in selection:
            raise ValueError(f"selection omits required identity {required}")
    if training_result.get("selected_checkpoint_step") != selection.get("selected_checkpoint_step"):
        raise ValueError("selection checkpoint step differs from the training result")
    if training_result.get("selected_dev_loss") != selection.get("selected_dev_loss"):
        raise ValueError("selection dev loss differs from the training result")
    return selection


def run_overfit_smoke(root: Path, *, max_seconds: float = 300.0) -> dict[str, Any]:
    """Require >=80% mean-loss reduction on exactly 64 fixed train sequences."""
    if not 0 < max_seconds <= 300:
        raise ValueError("smoke wall ceiling must be in (0, 300] seconds")
    training_view = load_training_view_manifest(root, verify_development=False)
    lock_path = _tool_root() / "uv.lock"
    if not lock_path.is_file():
        raise FileNotFoundError("uv.lock is required before the smoke can be frozen")
    smoke_contract: dict[str, Any] = {
        "schema": "uor-r4-softmax-trainer-smoke-contract/1",
        "dataset_manifest_cid": training_view["dataset_manifest_cid"],
        "training_view_manifest_cid": training_view["manifest_cid"],
        "split_policy_cid": training_view["split_policy_cid"],
        "environment": {
            "dependencies": _dependency_versions(),
            "uv_lock_cid": cid_file(lock_path),
            "device": "mps",
            "cpu_fallback": False,
        },
        "trainer_implementation": _trainer_implementation_contract(),
        "model": FROZEN_MODEL_CONFIG.as_contract(),
        "sequences": 64,
        "context": FROZEN_MODEL_CONFIG.max_position_embeddings,
        "optimizer": {"name": "AdamW", "learning_rate": 3e-3, "microbatch": 8},
        "required_loss_reduction_fraction": 0.80,
        "wall_ceiling_seconds": max_seconds,
    }
    smoke_contract_cid = cid_bytes(canonical_json_bytes(smoke_contract))
    device = require_mps(seed=1014)
    store = TokenStore(root / TOKEN_RELATIVE_PATHS["train"])
    inputs, targets = store.first_sequences(64)
    started = time.monotonic()
    model = R4SoftmaxForCausalLM().to(device)
    optimizer = torch.optim.AdamW(model.parameters(), lr=3e-3, betas=(0.9, 0.95), weight_decay=0.0)
    microbatch = 8
    initial_loss = _evaluate_fixed(model, inputs, targets, device, microbatch)
    final_loss = initial_loss
    step = 0
    target_loss = initial_loss * 0.20
    while final_loss > target_loss:
        if time.monotonic() - started >= max_seconds:
            break
        model.train()
        base = (step * microbatch) % 64
        indices = torch.arange(base, base + microbatch) % 64
        batch_inputs = inputs[indices].to(device)
        batch_targets = targets[indices].to(device)
        optimizer.zero_grad(set_to_none=True)
        output = model(batch_inputs, batch_targets)
        assert output.loss is not None
        output.loss.backward()
        torch.nn.utils.clip_grad_norm_(model.parameters(), 1.0)
        optimizer.step()
        step += 1
        if step % 16 == 0:
            final_loss = _evaluate_fixed(model, inputs, targets, device, microbatch)
    _sync_mps()
    elapsed = time.monotonic() - started
    reduction = 1.0 - final_loss / initial_loss
    passed = reduction >= 0.80 and elapsed <= 300.0
    result: dict[str, Any] = {
        "schema": SMOKE_SCHEMA,
        "terminal": "PASS" if passed else "FAIL",
        "dataset_manifest_cid": training_view["dataset_manifest_cid"],
        "training_view_manifest_cid": training_view["manifest_cid"],
        "split_policy_cid": training_view["split_policy_cid"],
        "smoke_contract": smoke_contract,
        "smoke_contract_cid": smoke_contract_cid,
        "device": "mps",
        "sequences": 64,
        "context": FROZEN_MODEL_CONFIG.max_position_embeddings,
        "initial_loss": initial_loss,
        "final_loss": final_loss,
        "loss_reduction_fraction": reduction,
        "required_reduction_fraction": 0.80,
        "optimizer_steps": step,
        "elapsed_seconds": elapsed,
        "wall_ceiling_seconds": 300.0,
    }
    result["result_cid"] = cid_bytes(canonical_json_bytes(result))
    atomic_write_json(root / "smoke" / "smoke-result.json", result)
    if not passed:
        raise RuntimeError(
            f"64-sequence overfit gate failed: reduction={reduction:.3%}, elapsed={elapsed:.1f}s"
        )

    export_manifest = export_hugging_face_snapshot(
        model,
        output_dir=root / "smoke" / "export",
        tokenizer_path=root / TOKENIZER_RELATIVE_PATH,
        training_result=result,
        dataset_manifest_cid=str(training_view["dataset_manifest_cid"]),
        training_view_manifest_cid=str(training_view["manifest_cid"]),
        split_policy_cid=str(training_view["split_policy_cid"]),
        run_contract_cid=smoke_contract_cid,
        selected_checkpoint_cid=None,
    )
    prefix_token_ids = np.asarray(store.tokens[:32], dtype=np.int64).tolist()
    with torch.no_grad():
        prefix_inputs = torch.tensor([prefix_token_ids], dtype=torch.long, device=device)
        enabled_logits = model(prefix_inputs).logits[0, -1].float().cpu().tolist()
        attention_off_logits = (
            model(prefix_inputs, attention_off=True).logits[0, -1].float().cpu().tolist()
        )
    prefix_fixture: dict[str, Any] = {
        "schema": "uor-r4.r4-softmax-python-prefix-logits/1",
        "weights_cid": export_manifest["weights_cid"],
        "token_store_cid": cid_file(root / TOKEN_RELATIVE_PATHS["train"]),
        "prefix_token_ids": prefix_token_ids,
        "maximum_absolute_logit_delta_limit": 0.005,
        "enabled": {
            "top1_token_id": int(np.argmax(np.asarray(enabled_logits))),
            "logits": enabled_logits,
        },
        "attention_off": {
            "top1_token_id": int(np.argmax(np.asarray(attention_off_logits))),
            "logits": attention_off_logits,
        },
    }
    prefix_fixture["result_cid"] = cid_bytes(canonical_json_bytes(prefix_fixture))
    atomic_write_json(root / "smoke" / "python-prefix-logits.json", prefix_fixture)
    return write_bound_manifest(
        root / "smoke" / "smoke-manifest.json",
        {
            "schema": "uor-r4-softmax-trainer-smoke-manifest/1",
            "terminal": "PASS_EXPORT_AWAITING_RUST_PARITY",
            "dataset_manifest_cid": training_view["dataset_manifest_cid"],
            "training_view_manifest_cid": training_view["manifest_cid"],
            "split_policy_cid": training_view["split_policy_cid"],
            "smoke_contract_cid": smoke_contract_cid,
            "smoke_result_cid": result["result_cid"],
            "export_manifest_cid": export_manifest["manifest_cid"],
            "weights_cid": export_manifest["weights_cid"],
            "prefix_result_cid": prefix_fixture["result_cid"],
        },
        artifact_root=root,
        relative_paths=[
            "smoke/smoke-result.json",
            "smoke/python-prefix-logits.json",
            "smoke/export/config.json",
            "smoke/export/model.safetensors",
            "smoke/export/tokenizer.json",
            "smoke/export/training-result.json",
            "smoke/export/export-manifest.json",
        ],
    )


def _load_sealed_prompt_fixture(path: Path) -> list[dict[str, object]]:
    fixture = json.loads(path.read_text(encoding="utf-8"))
    expected_cid = fixture.get("fixture_cid")
    unsigned = dict(fixture)
    unsigned.pop("fixture_cid", None)
    if expected_cid != cid_bytes(canonical_json_bytes(unsigned)):
        raise ValueError("sealed prompt fixture CID does not reproduce")
    prompts = fixture.get("prompts")
    if not isinstance(prompts, list) or len(prompts) != SEALED_PROMPT_COUNT:
        raise ValueError("sealed prompt fixture must contain exactly five prompts")
    story_cids = [str(prompt["story_cid"]) for prompt in prompts]
    if story_cids != sorted(story_cids) or len(set(story_cids)) != SEALED_PROMPT_COUNT:
        raise ValueError("sealed prompt stories must be unique and ordered by CID")
    for prompt in prompts:
        token_ids = prompt.get("token_ids")
        if not isinstance(token_ids, list) or len(token_ids) != SEALED_PROMPT_TOKENS_PER_STORY:
            raise ValueError("each sealed prompt must contain exactly 24 token ids")
        if not isinstance(prompt.get("text"), str):
            raise ValueError("each sealed prompt must carry round-trippable UTF-8 text")
    return prompts


def train_main(root: Path, *, config: TrainConfig, resume: bool = False) -> dict[str, Any]:
    """Run the single frozen main campaign and export its selected checkpoint."""
    config.validate()
    checkpoint_dir = root / "checkpoints"
    latest_path = checkpoint_dir / "latest.pt"
    best_path = checkpoint_dir / "best.pt"
    selection_path = root / "selection" / "selection-manifest.json"
    if selection_path.exists():
        _load_frozen_selection(root)
        raise FileExistsError(
            "the selected campaign is CID-frozen; train and --resume may not mutate it"
        )
    if (root / "reveal" / "reveal-manifest.json").exists():
        raise FileExistsError("sealed test was already revealed; the campaign is immutable")
    training_view = load_training_view_manifest(root)
    admission = load_main_admission(
        root, training_view=training_view, require_current_trainer=True
    )
    run_contract = build_run_contract(training_view, config, admission)
    if run_contract["trainer_implementation"]["tree_cid"] != admission[
        "trainer_implementation_tree_cid"
    ]:
        raise RuntimeError("campaign trainer changed while admission was being verified")
    run_contract_cid = cid_bytes(canonical_json_bytes(run_contract))
    if not resume and any(path.exists() for path in [latest_path, best_path]):
        raise FileExistsError(
            "campaign checkpoints already exist; use --resume for the same admitted run or a fresh root"
        )
    device = require_mps(config.seed)
    train_store = TokenStore(root / TOKEN_RELATIVE_PATHS["train"])
    dev_store = TokenStore(root / TOKEN_RELATIVE_PATHS["dev"])
    model = R4SoftmaxForCausalLM().to(device)
    optimizer = torch.optim.AdamW(
        model.parameters(),
        lr=config.learning_rate,
        betas=(config.adam_beta1, config.adam_beta2),
        weight_decay=config.weight_decay,
    )
    optimizer_step = 0
    tokens_seen = 0
    elapsed_before_resume = 0.0
    best_dev_loss = math.inf
    if resume:
        resume_path = latest_path if latest_path.is_file() else best_path
        if not resume_path.is_file():
            raise FileNotFoundError("--resume requested but no same-run checkpoint is present")
        checkpoint = _load_checkpoint(
            resume_path,
            model=model,
            optimizer=optimizer,
            device=device,
            run_contract_cid=run_contract_cid,
        )
        optimizer_step = int(checkpoint["optimizer_step"])
        tokens_seen = int(checkpoint["tokens_seen"])
        elapsed_before_resume = float(checkpoint["elapsed_training_seconds"])
        best_dev_loss = float(checkpoint["best_dev_loss"])
    else:
        initial_dev_loss = evaluate(model, dev_store, device, config.batch_size)
        best_dev_loss = initial_dev_loss
        _save_checkpoint(
            best_path,
            model=model,
            optimizer=optimizer,
            optimizer_step=0,
            tokens_seen=0,
            elapsed_training_seconds=0.0,
            best_dev_loss=best_dev_loss,
            run_contract=run_contract,
            run_contract_cid=run_contract_cid,
        )

    started = time.monotonic()
    for step in range(optimizer_step + 1, config.optimizer_steps + 1):
        model.train()
        optimizer.zero_grad(set_to_none=True)
        accumulated_loss = 0.0
        for accumulation in range(config.gradient_accumulation_steps):
            batch_index = (step - 1) * config.gradient_accumulation_steps + accumulation
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
        learning_rate = _learning_rate(step, config.optimizer_steps, config)
        for group in optimizer.param_groups:
            group["lr"] = learning_rate
        optimizer.step()
        optimizer_step = step
        tokens_seen = step * config.tokens_per_optimizer_step
        elapsed_total = elapsed_before_resume + (time.monotonic() - started)

        if elapsed_total >= config.wall_ceiling_seconds:
            _save_checkpoint(
                latest_path,
                model=model,
                optimizer=optimizer,
                optimizer_step=optimizer_step,
                tokens_seen=tokens_seen,
                elapsed_training_seconds=elapsed_total,
                best_dev_loss=best_dev_loss,
                run_contract=run_contract,
                run_contract_cid=run_contract_cid,
            )
            raise RuntimeError("main campaign reached its six-hour wall ceiling")

        should_evaluate = step % config.evaluation_interval == 0 or step == config.optimizer_steps
        if should_evaluate:
            dev_loss = evaluate(model, dev_store, device, config.batch_size)
            if dev_loss < best_dev_loss:
                best_dev_loss = dev_loss
                _save_checkpoint(
                    best_path,
                    model=model,
                    optimizer=optimizer,
                    optimizer_step=optimizer_step,
                    tokens_seen=tokens_seen,
                    elapsed_training_seconds=elapsed_total,
                    best_dev_loss=best_dev_loss,
                    run_contract=run_contract,
                    run_contract_cid=run_contract_cid,
                )
            print(
                f"step={step}/{config.optimizer_steps} train_loss={accumulated_loss / config.gradient_accumulation_steps:.6f} "
                f"dev_loss={dev_loss:.6f} best_dev_loss={best_dev_loss:.6f} lr={learning_rate:.8f}",
                flush=True,
            )
        if step % config.checkpoint_interval == 0 or step == config.optimizer_steps:
            _save_checkpoint(
                latest_path,
                model=model,
                optimizer=optimizer,
                optimizer_step=optimizer_step,
                tokens_seen=tokens_seen,
                elapsed_training_seconds=elapsed_total,
                best_dev_loss=best_dev_loss,
                run_contract=run_contract,
                run_contract_cid=run_contract_cid,
            )

    _sync_mps()
    elapsed = elapsed_before_resume + (time.monotonic() - started)
    selected = _load_checkpoint(
        best_path,
        model=model,
        optimizer=None,
        device=device,
        run_contract_cid=run_contract_cid,
    )
    selected_step = int(selected["optimizer_step"])
    selected_dev_loss = evaluate(model, dev_store, device, config.batch_size)

    result: dict[str, Any] = {
        "schema": "uor-r4-softmax-trainer-selection-result/1",
        "terminal": "FINAL_CHECKPOINT_FROZEN_TEST_UNOPENED",
        "dataset_manifest_cid": training_view["dataset_manifest_cid"],
        "training_view_manifest_cid": training_view["manifest_cid"],
        "run_contract": run_contract,
        "run_contract_cid": run_contract_cid,
        "optimizer_steps_completed": optimizer_step,
        "train_tokens_seen": tokens_seen,
        "selected_checkpoint_step": selected_step,
        "selected_dev_loss": selected_dev_loss,
        "elapsed_training_seconds": elapsed,
        "sealed_test_status": "UNOPENED",
    }
    result["result_cid"] = cid_bytes(canonical_json_bytes(result))
    atomic_write_json(root / "training-result.json", result)
    selected_checkpoint_cid = cid_file(best_path)
    export_manifest = export_hugging_face_snapshot(
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
    selection_payload: dict[str, Any] = {
        "schema": SELECTION_MANIFEST_SCHEMA,
        "dataset_manifest_cid": training_view["dataset_manifest_cid"],
        "training_view_manifest_cid": training_view["manifest_cid"],
        "run_contract_cid": run_contract_cid,
        "split_policy_cid": training_view["split_policy_cid"],
        "selected_checkpoint_cid": selected_checkpoint_cid,
        "selected_checkpoint_step": selected_step,
        "selected_dev_loss": selected_dev_loss,
        "export_manifest_cid": export_manifest["manifest_cid"],
        "weights_cid": export_manifest["weights_cid"],
        "tokenizer_cid": export_manifest["tokenizer_cid"],
        "training_result_cid": result["result_cid"],
        "main_admission_manifest_cid": admission["manifest_cid"],
        "sealed_test_status": "UNOPENED_BEFORE_THIS_MANIFEST",
    }
    return write_bound_manifest(
        root / "selection" / "selection-manifest.json",
        selection_payload,
        artifact_root=root,
        relative_paths=[
            str(MAIN_ADMISSION_MANIFEST_PATH),
            "checkpoints/best.pt",
            "training-result.json",
            "export/config.json",
            "export/model.safetensors",
            "export/tokenizer.json",
            "export/training-result.json",
            "export/export-manifest.json",
        ],
    )


def reveal_sealed_test(root: Path) -> dict[str, Any]:
    """Reveal/evaluate only after the final selection manifest reproduces."""
    if (root / "reveal" / "reveal-manifest.json").exists():
        raise FileExistsError("sealed reveal is already frozen; use `verify` instead")
    selection = _load_frozen_selection(root)
    training_result = json.loads((root / "training-result.json").read_text(encoding="utf-8"))
    run_contract = training_result["run_contract"]
    run_contract_cid = str(training_result["run_contract_cid"])
    if cid_bytes(canonical_json_bytes(run_contract)) != run_contract_cid:
        raise ValueError("selected run contract does not reproduce before reveal")
    if run_contract.get("trainer_implementation", {}).get("tree_cid") != (
        trainer_implementation_contract()["tree_cid"]
    ):
        raise ValueError("trainer implementation changed after checkpoint selection")

    device = require_mps(int(run_contract["optimization"]["seed"]))
    model = R4SoftmaxForCausalLM().to(device)
    _load_checkpoint(
        root / "checkpoints" / "best.pt",
        model=model,
        optimizer=None,
        device=device,
        run_contract_cid=run_contract_cid,
    )

    # The final checkpoint and selection manifest are now frozen and verified.
    # Only this post-freeze phase opens the full manifest and sealed artifacts.
    dataset_manifest = load_dataset_manifest(root)
    if dataset_manifest.get("manifest_cid") != selection.get("dataset_manifest_cid"):
        raise ValueError("revealed dataset differs from the precommitted dataset CID")
    if dataset_manifest.get("split_policy_cid") != selection.get("split_policy_cid"):
        raise ValueError("revealed split policy differs from the frozen selection")
    test_store = TokenStore(root / TOKEN_RELATIVE_PATHS["test"])
    batch_size = int(run_contract["optimization"]["batch_size"])
    enabled_test_loss = evaluate(model, test_store, device, batch_size)
    attention_off_test_loss = evaluate(
        model, test_store, device, batch_size, attention_off=True
    )
    attention_off_loss_delta = attention_off_test_loss - enabled_test_loss
    tokenizer = Tokenizer.from_file(str(root / TOKENIZER_RELATIVE_PATH))
    prompts = _load_sealed_prompt_fixture(root / SEALED_PROMPT_RELATIVE_PATH)

    prefix_token_ids = np.asarray(test_store.tokens[:32], dtype=np.int64).tolist()
    with torch.no_grad():
        prefix_inputs = torch.tensor([prefix_token_ids], dtype=torch.long, device=device)
        enabled_prefix_logits = (
            model(prefix_inputs).logits[0, -1]
            .float()
            .cpu()
            .tolist()
        )
        attention_off_prefix_logits = (
            model(prefix_inputs, attention_off=True)
            .logits[0, -1]
            .float()
            .cpu()
            .tolist()
        )
    weights_cid = cid_file(root / "export" / "model.safetensors")
    if weights_cid != selection.get("weights_cid"):
        raise ValueError("exported weights differ from the frozen selection")
    prefix_reference: dict[str, Any] = {
        "schema": "uor-r4.r4-softmax-python-prefix-logits/1",
        "weights_cid": weights_cid,
        "token_store_cid": cid_file(root / TOKEN_RELATIVE_PATHS["test"]),
        "prefix_token_ids": prefix_token_ids,
        "maximum_absolute_logit_delta_limit": 0.005,
        "enabled": {
            "top1_token_id": int(np.argmax(np.asarray(enabled_prefix_logits))),
            "logits": enabled_prefix_logits,
        },
        "attention_off": {
            "top1_token_id": int(np.argmax(np.asarray(attention_off_prefix_logits))),
            "logits": attention_off_prefix_logits,
        },
    }
    prefix_reference["result_cid"] = cid_bytes(canonical_json_bytes(prefix_reference))
    atomic_write_json(root / "reveal" / "python-prefix-logits.json", prefix_reference)

    prompt_records: list[dict[str, Any]] = []
    for index, prompt_record in enumerate(prompts):
        prompt = list(prompt_record["token_ids"])
        prompt_text = tokenizer.decode(prompt, skip_special_tokens=True)
        if prompt_text != prompt_record["text"]:
            raise ValueError("sealed prompt text does not reproduce from token ids")
        if tokenizer.encode(prompt_text, add_special_tokens=False).ids != prompt:
            raise ValueError("sealed prompt token ids do not reproduce from text")
        prompt_text.encode("utf-8", errors="strict")
        prompt_records.append(
            {
                "index": index,
                "story_cid": prompt_record["story_cid"],
                "prompt_token_ids": prompt,
                "prompt_tokens": SEALED_PROMPT_TOKENS_PER_STORY,
                "prompt_text": prompt_text,
            }
        )
    passed = enabled_test_loss <= 1.50 and attention_off_loss_delta >= 0.10
    result: dict[str, Any] = {
        "schema": "uor-r4-softmax-trainer-sealed-reveal/1",
        "terminal": "PASS_PYTHON_REVEAL" if passed else "FAIL_PYTHON_REVEAL",
        "selection_manifest_cid": selection["manifest_cid"],
        "selected_checkpoint_cid": selection["selected_checkpoint_cid"],
        "dataset_manifest_cid": dataset_manifest["manifest_cid"],
        "training_view_manifest_cid": selection["training_view_manifest_cid"],
        "split_policy_cid": dataset_manifest["split_policy_cid"],
        "weights_cid": weights_cid,
        "tokenizer_cid": selection["tokenizer_cid"],
        "export_manifest_cid": selection["export_manifest_cid"],
        "training_result_cid": selection["training_result_cid"],
        "enabled_sealed_test_loss": enabled_test_loss,
        "sealed_test_loss_ceiling": 1.50,
        "attention_off_sealed_test_loss": attention_off_test_loss,
        "attention_off_minus_enabled_loss": attention_off_loss_delta,
        "attention_off_minimum_loss_delta": 0.10,
        "sealed_test_store_token_ids": len(test_store.tokens),
        "sealed_test_scored_next_tokens": test_store.scored_next_tokens,
        "sealed_prompt_token_ids": SEALED_PROMPT_TOKEN_COUNT,
        "total_revealed_test_token_ids": len(test_store.tokens) + SEALED_PROMPT_TOKEN_COUNT,
        "prompt_selection": "first 24 content tokens of five lowest full-story CIDs",
        "python_prefix_result_cid": prefix_reference["result_cid"],
        "prompts": prompt_records,
        "autonomous_generation_status": "NOT_RUN_RUST_SEEDED_SAMPLER_REQUIRED",
        "scope": "Python NLL/parity reveal; Rust R4/Spin qualification and autonomous outputs remain separate",
    }
    result["result_cid"] = cid_bytes(canonical_json_bytes(result))
    result_path = root / "reveal" / "reveal-result.json"
    atomic_write_json(result_path, result)
    reveal_manifest = write_bound_manifest(
        root / "reveal" / "reveal-manifest.json",
        {
            "schema": "uor-r4-softmax-trainer-reveal-manifest/1",
            "selection_manifest_cid": selection["manifest_cid"],
            "selected_checkpoint_cid": selection["selected_checkpoint_cid"],
            "dataset_manifest_cid": dataset_manifest["manifest_cid"],
            "training_view_manifest_cid": selection["training_view_manifest_cid"],
            "split_policy_cid": dataset_manifest["split_policy_cid"],
            "weights_cid": weights_cid,
            "tokenizer_cid": selection["tokenizer_cid"],
            "export_manifest_cid": selection["export_manifest_cid"],
            "training_result_cid": selection["training_result_cid"],
            "sealed_test_store_token_ids": len(test_store.tokens),
            "sealed_test_scored_next_tokens": test_store.scored_next_tokens,
            "sealed_prompt_token_ids": SEALED_PROMPT_TOKEN_COUNT,
            "total_revealed_test_token_ids": len(test_store.tokens)
            + SEALED_PROMPT_TOKEN_COUNT,
            "enabled_sealed_test_loss": enabled_test_loss,
            "attention_off_sealed_test_loss": attention_off_test_loss,
            "attention_off_minus_enabled_loss": attention_off_loss_delta,
            "python_prefix_result_cid": prefix_reference["result_cid"],
            "reveal_result_cid": result["result_cid"],
        },
        artifact_root=root,
        relative_paths=[
            TOKEN_RELATIVE_PATHS["test"],
            INDEX_RELATIVE_PATHS["test"],
            SEALED_PROMPT_RELATIVE_PATH,
            "reveal/python-prefix-logits.json",
            "reveal/reveal-result.json",
        ],
    )
    if not passed:
        raise RuntimeError(
            "sealed reveal failed: "
            f"enabled_loss={enabled_test_loss:.6f}, "
            f"attention_off_delta={attention_off_loss_delta:.6f}"
        )
    return reveal_manifest
