"""One bounded MPS fine-tune that adds grounded answer/abstain behavior to #1017."""

from __future__ import annotations

import math
import os
import time
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Any

import torch
from safetensors.torch import load_file
from torch import Tensor

from .constants import FROZEN_MODEL_CONFIG
from .export import export_hugging_face_snapshot
from .grounding_data import GroundingStore, build_grounding_corpus, grounding_split_policy
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
from .train import require_mps


ISSUE = 954
GROUNDING_RUN_SCHEMA = "uor-r4-softmax-grounding-run/1"
GROUNDING_CHECKPOINT_SCHEMA = "uor-r4-softmax-grounding-checkpoint/1"
GROUNDING_RESULT_SCHEMA = "uor-r4-softmax-grounding-result/1"
GROUNDING_PREFIX_SCHEMA = "uor-r4.r4-softmax-python-grounding-prefix-logits/1"
GROUNDING_DATASET_MANIFEST_SCHEMA = "uor-r4-softmax-grounding-dataset-manifest/1"
GROUNDING_FINAL_MANIFEST_SCHEMA = "uor-r4-softmax-grounding-final-manifest/1"
PREFIX_LOGIT_ABS_TOLERANCE = 0.005


@dataclass(frozen=True, slots=True)
class GroundingConfig:
    """The fixed, no-sweep #954 product-grounding optimization contract."""

    seed: int = 954
    batch_size: int = 16
    gradient_accumulation_steps: int = 4
    optimizer_steps: int = 384
    learning_rate: float = 5e-5
    minimum_learning_rate: float = 5e-6
    warmup_steps: int = 32
    weight_decay: float = 0.01
    adam_beta1: float = 0.9
    adam_beta2: float = 0.95
    adam_epsilon: float = 1e-8
    gradient_clip: float = 1.0
    progress_interval: int = 16
    checkpoint_interval: int = 64
    evaluation_steps: tuple[int, ...] = (192, 384)
    wall_ceiling_seconds: float = 45 * 60
    conservative_seconds_per_step: float = 3.491307

    def validate(self) -> None:
        if self != GroundingConfig():
            raise ValueError("#954 exposes one fixed grounding fine-tune, not a sweep")

    def as_contract(self) -> dict[str, Any]:
        self.validate()
        value = asdict(self)
        value["evaluation_steps"] = list(self.evaluation_steps)
        value["estimated_training_seconds"] = (
            self.optimizer_steps * self.conservative_seconds_per_step
        )
        return value


def _learning_rate(step: int, config: GroundingConfig) -> float:
    if not 1 <= step <= config.optimizer_steps:
        raise ValueError("grounding step is outside the fixed schedule")
    if step <= config.warmup_steps:
        return config.learning_rate * step / config.warmup_steps
    progress = (step - config.warmup_steps) / (
        config.optimizer_steps - config.warmup_steps
    )
    cosine = 0.5 * (1.0 + math.cos(math.pi * progress))
    return config.minimum_learning_rate + cosine * (
        config.learning_rate - config.minimum_learning_rate
    )


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


def _write_or_verify_json(path: Path, value: dict[str, Any]) -> None:
    encoded = canonical_json_bytes(value)
    if path.exists():
        if path.read_bytes() != encoded:
            raise ValueError(f"existing grounding artifact differs: {path}")
        return
    atomic_write(path, encoded)


def _save_checkpoint(
    path: Path,
    *,
    model: R4SoftmaxForCausalLM,
    optimizer: torch.optim.Optimizer,
    step: int,
    elapsed_seconds: float,
    evaluations: list[dict[str, Any]],
    run_contract_cid: str,
) -> None:
    payload = {
        "schema": GROUNDING_CHECKPOINT_SCHEMA,
        "issue": ISSUE,
        "run_contract_cid": run_contract_cid,
        "optimizer_step": step,
        "elapsed_seconds": elapsed_seconds,
        "evaluations": evaluations,
        "model": _cpu_tree(model.state_dict()),
        "optimizer": _cpu_tree(optimizer.state_dict()),
    }
    path.parent.mkdir(parents=True, exist_ok=True)
    temporary = path.with_name(f".{path.name}.part")
    torch.save(payload, temporary)
    os.replace(temporary, path)


def _load_checkpoint(
    path: Path,
    *,
    model: R4SoftmaxForCausalLM,
    optimizer: torch.optim.Optimizer,
    device: torch.device,
    run_contract_cid: str,
) -> tuple[int, float, list[dict[str, Any]]]:
    checkpoint = torch.load(path, map_location="cpu", weights_only=False)
    if (
        not isinstance(checkpoint, dict)
        or checkpoint.get("schema") != GROUNDING_CHECKPOINT_SCHEMA
        or checkpoint.get("issue") != ISSUE
        or checkpoint.get("run_contract_cid") != run_contract_cid
    ):
        raise ValueError("grounding resume checkpoint belongs to another run")
    model.load_state_dict(checkpoint["model"], strict=True)
    optimizer.load_state_dict(checkpoint["optimizer"])
    _optimizer_to_device(optimizer, device)
    step = int(checkpoint.get("optimizer_step", -1))
    if not 0 <= step <= GroundingConfig().optimizer_steps:
        raise ValueError("grounding checkpoint step is outside the fixed run")
    return (
        step,
        float(checkpoint.get("elapsed_seconds", 0.0)),
        list(checkpoint.get("evaluations", [])),
    )


@torch.no_grad()
def _evaluate_answer_loss(
    model: R4SoftmaxForCausalLM,
    store: GroundingStore,
    device: torch.device,
    batch_size: int,
) -> float:
    model.eval()
    total_loss = 0.0
    total_tokens = 0
    for inputs, targets in store.sequential_batches(batch_size):
        inputs = inputs.to(device)
        targets = targets.to(device)
        output = model(inputs, targets)
        assert output.loss is not None
        supervised = int((targets != -100).sum().detach().cpu())
        total_loss += float(output.loss.detach().cpu()) * supervised
        total_tokens += supervised
    if total_tokens == 0:
        raise RuntimeError("grounding development set has no supervised answer tokens")
    return total_loss / total_tokens


def _write_prefix_fixture(
    root: Path,
    *,
    model: R4SoftmaxForCausalLM,
    development_store: GroundingStore,
    device: torch.device,
    weights_cid: str,
    dataset_cid: str,
) -> dict[str, Any]:
    prefix_token_ids = development_store.parity_prefix()
    with torch.no_grad():
        inputs = torch.tensor([prefix_token_ids], dtype=torch.long, device=device)
        logits = model(inputs).logits[0, -1].float().cpu()
    if logits.numel() != FROZEN_MODEL_CONFIG.vocab_size or not torch.isfinite(logits).all():
        raise RuntimeError("grounding parity prefix produced invalid logits")
    value: dict[str, Any] = {
        "schema": GROUNDING_PREFIX_SCHEMA,
        "token_store_cid": dataset_cid,
        "weights_cid": weights_cid,
        "prefix_token_ids": prefix_token_ids,
        "maximum_absolute_logit_delta_limit": PREFIX_LOGIT_ABS_TOLERANCE,
        "enabled": {
            "top1_token_id": int(torch.argmax(logits).item()),
            "logits": logits.tolist(),
        },
    }
    value["result_cid"] = cid_bytes(canonical_json_bytes(value))
    atomic_write_json(root / "qualification/python-grounding-prefix-logits.json", value)
    return value


def _validated_predecessor(predecessor: Path) -> dict[str, Any]:
    manifest = verify_bound_manifest(
        predecessor / "export-manifest.json", artifact_root=predecessor
    )
    if manifest.get("model_contract") != FROZEN_MODEL_CONFIG.as_contract():
        raise ValueError("grounding predecessor is not the six-layer #1017 architecture")
    for path in ("config.json", "model.safetensors", "tokenizer.json"):
        if not (predecessor / path).is_file():
            raise FileNotFoundError(predecessor / path)
    return manifest


def train_grounding(
    root: Path,
    *,
    predecessor: Path,
    resume: bool = False,
    config: GroundingConfig = GroundingConfig(),
) -> dict[str, Any]:
    """Run the fixed 384-step #954 MPS SFT and export the standard local model."""
    config.validate()
    root = root.expanduser().resolve()
    predecessor = predecessor.expanduser().resolve()
    if root == predecessor or predecessor in root.parents:
        raise ValueError("grounding output must not overwrite the #1017 predecessor")
    final_manifest_path = root / "grounding-final-manifest.json"
    if final_manifest_path.exists():
        raise FileExistsError("the #954 grounding export is already complete")

    predecessor_manifest = _validated_predecessor(predecessor)
    corpus = build_grounding_corpus()
    root.mkdir(parents=True, exist_ok=True)
    dataset_path = root / "grounding-dataset.json"
    _write_or_verify_json(dataset_path, corpus)
    split_policy = grounding_split_policy()
    split_policy_cid = cid_bytes(canonical_json_bytes(split_policy))
    dataset_manifest = write_bound_manifest(
        root / "grounding-dataset-manifest.json",
        {
            "schema": GROUNDING_DATASET_MANIFEST_SCHEMA,
            "issue": ISSUE,
            "dataset_cid": corpus["dataset_cid"],
            "split_policy": split_policy,
            "split_policy_cid": split_policy_cid,
            "predecessor_tokenizer_cid": predecessor_manifest["tokenizer_cid"],
        },
        artifact_root=root,
        relative_paths=["grounding-dataset.json"],
    )

    run_contract: dict[str, Any] = {
        "schema": GROUNDING_RUN_SCHEMA,
        "issue": ISSUE,
        "predecessor": {
            "export_manifest_cid": predecessor_manifest["manifest_cid"],
            "weights_cid": predecessor_manifest["weights_cid"],
            "tokenizer_cid": predecessor_manifest["tokenizer_cid"],
            "optimizer_state": "fresh AdamW; #1017 moments are not inherited",
        },
        "model": FROZEN_MODEL_CONFIG.as_contract(),
        "dataset_manifest_cid": dataset_manifest["manifest_cid"],
        "split_policy_cid": split_policy_cid,
        "optimization": config.as_contract(),
        "loss": (
            "causal cross entropy on answer and EOS tokens only; context, question, "
            "instruction, and right padding targets are -100"
        ),
        "implementation": trainer_implementation_contract(),
        "product_gate": corpus["product_probes"],
    }
    run_contract_cid = cid_bytes(canonical_json_bytes(run_contract))

    tokenizer_path = predecessor / "tokenizer.json"
    train_store = GroundingStore(corpus["train"], tokenizer_path)
    development_store = GroundingStore(corpus["development"], tokenizer_path)
    device = require_mps(config.seed)
    model = R4SoftmaxForCausalLM()
    predecessor_state = load_file(str(predecessor / "model.safetensors"), device="cpu")
    model.load_state_dict(predecessor_state, strict=True)
    model = model.to(device)
    optimizer = torch.optim.AdamW(
        model.parameters(),
        lr=config.learning_rate,
        betas=(config.adam_beta1, config.adam_beta2),
        eps=config.adam_epsilon,
        weight_decay=config.weight_decay,
    )

    latest_path = root / "checkpoints/latest.pt"
    step = 0
    elapsed_before_resume = 0.0
    evaluations: list[dict[str, Any]] = []
    if resume:
        if not latest_path.is_file():
            raise FileNotFoundError("--resume requires checkpoints/latest.pt")
        step, elapsed_before_resume, evaluations = _load_checkpoint(
            latest_path,
            model=model,
            optimizer=optimizer,
            device=device,
            run_contract_cid=run_contract_cid,
        )
    elif latest_path.exists():
        raise FileExistsError("grounding checkpoint exists; use --resume")

    started = time.monotonic()
    last_loss: Tensor | None = None
    for optimizer_step in range(step + 1, config.optimizer_steps + 1):
        model.train()
        optimizer.zero_grad(set_to_none=True)
        for accumulation in range(config.gradient_accumulation_steps):
            batch_index = (
                (optimizer_step - 1) * config.gradient_accumulation_steps + accumulation
            )
            inputs, targets = train_store.deterministic_batch(
                seed=config.seed,
                batch_index=batch_index,
                batch_size=config.batch_size,
            )
            output = model(inputs.to(device), targets.to(device))
            assert output.loss is not None
            (output.loss / config.gradient_accumulation_steps).backward()
            last_loss = output.loss.detach()
        torch.nn.utils.clip_grad_norm_(model.parameters(), config.gradient_clip)
        learning_rate = _learning_rate(optimizer_step, config)
        for group in optimizer.param_groups:
            group["lr"] = learning_rate
        optimizer.step()
        step = optimizer_step
        elapsed = elapsed_before_resume + (time.monotonic() - started)

        if step in config.evaluation_steps:
            evaluations.append(
                {
                    "optimizer_step": step,
                    "answer_token_development_loss": _evaluate_answer_loss(
                        model, development_store, device, config.batch_size
                    ),
                }
            )
            elapsed = elapsed_before_resume + (time.monotonic() - started)

        if step % config.checkpoint_interval == 0 or step == config.optimizer_steps:
            _save_checkpoint(
                latest_path,
                model=model,
                optimizer=optimizer,
                step=step,
                elapsed_seconds=elapsed,
                evaluations=evaluations,
                run_contract_cid=run_contract_cid,
            )
        if step % config.progress_interval == 0 or step == config.optimizer_steps:
            observed_loss = float(last_loss.cpu()) if last_loss is not None else math.nan
            print(
                f"grounding_step={step}/{config.optimizer_steps} "
                f"answer_loss={observed_loss:.6f} lr={learning_rate:.8f} "
                f"elapsed_seconds={elapsed:.1f}",
                flush=True,
            )
        if elapsed >= config.wall_ceiling_seconds:
            raise RuntimeError("UNAVAILABLE_GROUNDING_WALL_BUDGET")

    if hasattr(torch, "mps"):
        torch.mps.synchronize()
    elapsed = elapsed_before_resume + (time.monotonic() - started)
    final_checkpoint_path = root / "checkpoints/final.pt"
    _save_checkpoint(
        final_checkpoint_path,
        model=model,
        optimizer=optimizer,
        step=step,
        elapsed_seconds=elapsed,
        evaluations=evaluations,
        run_contract_cid=run_contract_cid,
    )
    selected_checkpoint_cid = cid_file(final_checkpoint_path)

    result: dict[str, Any] = {
        "schema": GROUNDING_RESULT_SCHEMA,
        "terminal": "GROUNDING_SFT_COMPLETE_AWAITING_RUST_PRODUCT_CHECK",
        "issue": ISSUE,
        "run_contract": run_contract,
        "run_contract_cid": run_contract_cid,
        "dataset_manifest_cid": dataset_manifest["manifest_cid"],
        "split_policy_cid": split_policy_cid,
        "predecessor_weights_cid": predecessor_manifest["weights_cid"],
        "selected_checkpoint_cid": selected_checkpoint_cid,
        "optimizer_steps_completed": step,
        "elapsed_training_seconds": elapsed,
        "development_evaluations": evaluations,
        "product_probes": corpus["product_probes"],
        "product_behavior_status": "NOT_RUN",
        "rust_prefix_parity_status": "NOT_RUN",
    }
    result["result_cid"] = cid_bytes(canonical_json_bytes(result))
    atomic_write_json(root / "grounding-training-result.json", result)

    export = export_hugging_face_snapshot(
        model,
        output_dir=root / "export",
        tokenizer_path=tokenizer_path,
        training_result=result,
        dataset_manifest_cid=dataset_manifest["manifest_cid"],
        training_view_manifest_cid=dataset_manifest["manifest_cid"],
        split_policy_cid=split_policy_cid,
        run_contract_cid=run_contract_cid,
        selected_checkpoint_cid=selected_checkpoint_cid,
    )
    prefix = _write_prefix_fixture(
        root,
        model=model,
        development_store=development_store,
        device=device,
        weights_cid=str(export["weights_cid"]),
        dataset_cid=str(corpus["dataset_cid"]),
    )
    final_manifest = write_bound_manifest(
        final_manifest_path,
        {
            "schema": GROUNDING_FINAL_MANIFEST_SCHEMA,
            "issue": ISSUE,
            "run_contract_cid": run_contract_cid,
            "dataset_manifest_cid": dataset_manifest["manifest_cid"],
            "training_result_cid": result["result_cid"],
            "export_manifest_cid": export["manifest_cid"],
            "weights_cid": export["weights_cid"],
            "tokenizer_cid": export["tokenizer_cid"],
            "python_prefix_result_cid": prefix["result_cid"],
            "product_behavior_status": "AWAITING_RUST",
        },
        artifact_root=root,
        relative_paths=[
            "grounding-dataset.json",
            "grounding-dataset-manifest.json",
            "grounding-training-result.json",
            "checkpoints/final.pt",
            "export/config.json",
            "export/model.safetensors",
            "export/tokenizer.json",
            "export/training-result.json",
            "export/export-manifest.json",
            "qualification/python-grounding-prefix-logits.json",
        ],
    )
    return {
        "terminal": result["terminal"],
        "optimizer_steps_completed": step,
        "elapsed_training_seconds": elapsed,
        "export": str(root / "export"),
        "weights_cid": export["weights_cid"],
        "python_prefix_result_cid": prefix["result_cid"],
        "final_manifest_cid": final_manifest["manifest_cid"],
        "product_behavior_status": "AWAITING_RUST",
    }
