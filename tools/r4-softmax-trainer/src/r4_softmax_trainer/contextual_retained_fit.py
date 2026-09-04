"""Bounded training-only adaptation for the contextual retained value write."""

from __future__ import annotations

import os
import platform
import time
from pathlib import Path
from typing import Any

import torch

from .group_retention_campaign import load_group_geometry_artifacts
from .language_path_generalization import (
    CONTEXT,
    PARAMETER_COUNT,
    R4ContextualValueWriteLanguagePathV1,
)
from .language_path_generalization_campaign import (
    ADAM_BETA1,
    ADAM_BETA2,
    ADAM_EPSILON,
    GRADIENT_CLIP,
    WEIGHT_DECAY,
    learning_rate,
)
from .language_path_generalization_data import (
    EXPECTED_TRAIN_SLICE_CID,
    TRAIN_WINDOWS,
    WINDOW_TOKENS,
    LanguagePathWindowStore,
    deterministic_window_order,
)
from .provenance import atomic_write, atomic_write_json, cid_bytes, cid_file


SCHEMA = "uor-r4.contextual-retained-fit/1"
STATUS = "CONTEXTUAL_RETAINED_ADAPTED"
MAX_UPDATES = 128
DEFAULT_UPDATES = MAX_UPDATES
BATCH_SIZE = 16
THREADS = 4
MAX_SECONDS = 840.0
EXPECTED_INITIAL_ARTIFACT_CID = (
    "blake3:d1417b325e7a545057cd38e9f1a723933a3682801877433d20e98774a5e9172d"
)
EXPECTED_GEOMETRY_CID = (
    "blake3:a812cf6749e637f4c486a6ad206b96c90d695b5c4bb2ed029df3c6bef147d702"
)
TRAIN_RELATIVE_PATH = Path("data/train.u16")
GEOMETRY_RELATIVE_PATH = Path("geometry/r4-group-address-geometry.json")
INITIAL_ARTIFACT_RELATIVE_PATH = Path("arms/retained/model.safetensors")
OUTPUT_DIRECTORY_RELATIVE_PATH = Path("arms/contextual-retained")
OUTPUT_ARTIFACT_RELATIVE_PATH = OUTPUT_DIRECTORY_RELATIVE_PATH / "model.safetensors"
OUTPUT_RESULT_RELATIVE_PATH = OUTPUT_DIRECTORY_RELATIVE_PATH / "fit.json"
VALUE_WRITE = "Wv(RMSNorm(x_t + strict_prior_retained_residual))"


def _input_record(path: Path, expected_cid: str) -> dict[str, Any]:
    observed_cid = cid_file(path)
    if observed_cid != expected_cid:
        raise ValueError(f"input artifact differs: {path}")
    return {"path": str(path), "bytes": path.stat().st_size, "cid": observed_cid}


def _configure_cpu(threads: int) -> dict[str, Any]:
    if threads != THREADS:
        raise ValueError("contextual retained fit requires four CPU threads")
    os.environ["OMP_NUM_THREADS"] = str(threads)
    os.environ["VECLIB_MAXIMUM_THREADS"] = str(threads)
    os.environ["OPENBLAS_NUM_THREADS"] = str(threads)
    torch.set_num_threads(threads)
    try:
        torch.set_num_interop_threads(threads)
    except RuntimeError:
        if torch.get_num_interop_threads() != threads:
            raise
    torch.use_deterministic_algorithms(True)
    return {
        "device": "cpu",
        "dtype": "float32",
        "platform": platform.system(),
        "threads": threads,
    }


def _require_first_step_gradients(model: torch.nn.Module) -> int:
    missing: list[str] = []
    for name, parameter in model.named_parameters():
        gradient = parameter.grad
        if (
            gradient is None
            or not bool(torch.isfinite(gradient).all())
            or not bool(torch.count_nonzero(gradient))
        ):
            missing.append(name)
    if missing:
        raise RuntimeError(f"contextual fit has absent, zero, or nonfinite gradients: {missing}")
    return len(tuple(model.parameters()))


def fit_contextual_retained(
    root: Path,
    *,
    updates: int = DEFAULT_UPDATES,
    threads: int = THREADS,
    max_seconds: float = MAX_SECONDS,
) -> dict[str, Any]:
    """Warm-start and adapt all retained parameters using open training bytes."""

    if isinstance(updates, bool) or not isinstance(updates, int):
        raise TypeError("updates must be an integer")
    if not 1 <= updates <= MAX_UPDATES:
        raise ValueError(f"updates must be between 1 and {MAX_UPDATES}")
    if (
        isinstance(max_seconds, bool)
        or not isinstance(max_seconds, (int, float))
        or not 0.0 < float(max_seconds) <= MAX_SECONDS
    ):
        raise ValueError(f"max_seconds must be positive and at most {MAX_SECONDS}")

    started = time.monotonic()
    resolved_root = root.expanduser().resolve()
    train_path = resolved_root / TRAIN_RELATIVE_PATH
    geometry_path = resolved_root / GEOMETRY_RELATIVE_PATH
    initial_artifact_path = resolved_root / INITIAL_ARTIFACT_RELATIVE_PATH
    output_directory = resolved_root / OUTPUT_DIRECTORY_RELATIVE_PATH
    output_artifact_path = resolved_root / OUTPUT_ARTIFACT_RELATIVE_PATH
    output_result_path = resolved_root / OUTPUT_RESULT_RELATIVE_PATH
    if output_directory.exists():
        raise FileExistsError(f"contextual fit output already exists: {output_directory}")

    training_input = _input_record(train_path, EXPECTED_TRAIN_SLICE_CID)
    geometry_input = _input_record(geometry_path, EXPECTED_GEOMETRY_CID)
    initial_input = _input_record(
        initial_artifact_path, EXPECTED_INITIAL_ARTIFACT_CID
    )
    execution = _configure_cpu(threads)
    store = LanguagePathWindowStore(train_path, window_count=TRAIN_WINDOWS)
    order = deterministic_window_order(TRAIN_WINDOWS)
    geometry = load_group_geometry_artifacts(geometry_path).exact_h4
    model = R4ContextualValueWriteLanguagePathV1(geometry).to(
        device=torch.device("cpu"), dtype=torch.float32
    )
    model.load_learned_artifact(initial_artifact_path.read_bytes())
    if model.parameter_count() != PARAMETER_COUNT:
        raise RuntimeError("contextual retained parameter count changed")

    parameters = tuple(model.parameters())
    optimizer = torch.optim.AdamW(
        parameters,
        lr=learning_rate(0),
        betas=(ADAM_BETA1, ADAM_BETA2),
        eps=ADAM_EPSILON,
        weight_decay=WEIGHT_DECAY,
    )
    optimizer_values = sum(
        parameter.numel()
        for group in optimizer.param_groups
        for parameter in group["params"]
    )
    if optimizer_values != PARAMETER_COUNT:
        raise RuntimeError("optimizer does not include every contextual retained parameter")

    losses: list[float] = []
    gradient_parameter_tensors = 0
    model.train()
    for update in range(1, updates + 1):
        if time.monotonic() - started >= float(max_seconds):
            raise TimeoutError("contextual retained fit reached its hard wall limit")
        offset = (update - 1) * BATCH_SIZE
        inputs, targets = store.batch(order[offset : offset + BATCH_SIZE])
        optimizer.zero_grad(set_to_none=True)
        output = model(inputs, targets)
        if output.loss is None or not bool(torch.isfinite(output.loss)):
            raise RuntimeError("contextual retained fit produced a nonfinite loss")
        output.loss.backward()
        if update == 1:
            gradient_parameter_tensors = _require_first_step_gradients(model)
        gradient_norm = torch.nn.utils.clip_grad_norm_(parameters, GRADIENT_CLIP)
        if not bool(torch.isfinite(gradient_norm)):
            raise RuntimeError("contextual retained fit produced a nonfinite gradient norm")
        rate = learning_rate(update)
        for group in optimizer.param_groups:
            group["lr"] = rate
        optimizer.step()
        losses.append(float(output.loss.detach()))
        if update % 16 == 0 or update == updates:
            print(
                f"contextual_retained_fit update={update}/{updates} "
                f"loss={losses[-1]:.6f} elapsed={time.monotonic() - started:.3f}s",
                flush=True,
            )

    elapsed_seconds = time.monotonic() - started
    if elapsed_seconds >= float(max_seconds):
        raise TimeoutError("contextual retained fit reached its hard wall limit")
    if not all(bool(torch.isfinite(parameter).all()) for parameter in parameters):
        raise RuntimeError("contextual retained fit produced a nonfinite parameter")

    artifact = model.export_learned_artifact()
    result = {
        "schema": SCHEMA,
        "status": STATUS,
        "model": {
            "policy": model.policy,
            "parameter_count": PARAMETER_COUNT,
            "value_write": VALUE_WRITE,
            "initialization": "qualified retained V1 artifact",
        },
        "fit": {
            "updates": updates,
            "maximum_updates": MAX_UPDATES,
            "batch_size": BATCH_SIZE,
            "training_windows": updates * BATCH_SIZE,
            "causal_targets": updates * BATCH_SIZE * CONTEXT,
            "learning_rate_first": learning_rate(1),
            "learning_rate_last": learning_rate(updates),
            "loss_first": losses[0],
            "loss_last": losses[-1],
            "loss_mean_first_16": sum(losses[:16]) / min(16, len(losses)),
            "loss_mean_last_16": sum(losses[-16:]) / min(16, len(losses)),
            "elapsed_seconds": elapsed_seconds,
            "hard_wall_seconds": float(max_seconds),
            "gradient_parameter_tensors": gradient_parameter_tensors,
            "optimizer_parameter_values": optimizer_values,
        },
        "execution": execution,
        "inputs": {
            "train": training_input,
            "geometry": geometry_input,
            "initial_artifact": initial_input,
            "validation_files_read": 0,
            "sealed_files_read": 0,
            "teacher_files_read": 0,
            "source_model_files_read": 0,
        },
        "artifact": {
            "path": str(output_artifact_path),
            "bytes": len(artifact),
            "cid": cid_bytes(artifact),
        },
    }
    atomic_write(output_artifact_path, artifact)
    atomic_write_json(output_result_path, result)
    return result


__all__ = [
    "DEFAULT_UPDATES",
    "MAX_SECONDS",
    "MAX_UPDATES",
    "SCHEMA",
    "STATUS",
    "fit_contextual_retained",
]
