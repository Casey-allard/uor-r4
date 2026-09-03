"""One fixed, resumable cyclic-fact fit; no development scoring."""

from __future__ import annotations

import fcntl
import math
import time
from collections.abc import Mapping
from dataclasses import asdict
from pathlib import Path
from typing import Any

import torch

from ..provenance import canonical_json_bytes, cid_bytes
from ..zoology_clock.development import _step
from ..zoology_clock.sampling import CyclingBatches
from ..zoology_control.model import (
    ZoologyFigure2Config,
    ZoologyFigure2Model,
    set_zoology_seed,
)
from ..zoology_release import development as release
from ..zoology_transfer.development import _save_checkpoint, _write_or_match
from . import contract, data
from .augmentation import augment_training_batch, rotation_ledger


def _empty_block() -> dict[str, int | float]:
    return {"updates": 0, "decisions": 0, "correct": 0, "loss_sum": 0.0}


def _state_cid(model: ZoologyFigure2Model) -> str:
    return release._tensor_mapping_cid(
        {
            name: value
            for name, value in model.state_dict().items()
            if name != "lm_head.weight"
        }
    )


def _record(path: Path, root: Path) -> dict[str, Any]:
    payload = path.read_bytes()
    return {
        "path": str(path.relative_to(root)),
        "bytes": len(payload),
        "cid": cid_bytes(payload),
    }


def _validate_preparation(root: Path, preparation: Mapping[str, Any]) -> None:
    observed = contract.validate_preparation(root)
    if observed != preparation or preparation["training"] != contract.TRAINING:
        raise ValueError("English fit preparation or training contract changed")


def _existing_result(
    root: Path, preparation: Mapping[str, Any], binding: str
) -> dict[str, Any]:
    result = release._read_json(root / "fit/fit.json", cid_field="fit_cid")
    if (
        result["preparation_cid"] != preparation["preparation_cid"]
        or result["binding_cid"] != binding
        or result["training"] != contract.TRAINING
        or result["artifact"]["config"] != contract.MODEL_CONFIG
    ):
        raise ValueError("completed fit belongs to a different preparation")
    for key, relative in (
        ("artifact", "fit/model.safetensors"),
        ("checkpoint", "fit/checkpoint.pt"),
    ):
        record = result[key]
        if record["path"] != relative or any(
            record[name] != value
            for name, value in _record(root / relative, root).items()
        ):
            raise ValueError("completed fit artifact or checkpoint changed")
    return result


def fit(root: Path, preparation: Mapping[str, Any]) -> dict[str, Any]:
    """Fit once or resume the same trajectory; completed results are immutable.

    The deadline conservatively includes pauses since the create-once start.
    Checkpointed active elapsed time also prevents backward-clock renewal.
    The caller evaluates only ``FIT_COMPLETE`` and carries ``elapsed_seconds``
    into the shared fit/evaluation/replay budget.
    """

    root = root.resolve()
    _validate_preparation(root, preparation)
    folder = root / "fit"
    folder.mkdir(parents=True, exist_ok=True)
    with (folder / "fit.lock").open("a+b") as lock:
        try:
            fcntl.flock(lock.fileno(), fcntl.LOCK_EX | fcntl.LOCK_NB)
        except BlockingIOError as error:
            raise ValueError(
                "this English fit already has an active process"
            ) from error
        try:
            return _fit_locked(root, preparation)
        finally:
            fcntl.flock(lock.fileno(), fcntl.LOCK_UN)


def _fit_locked(root: Path, preparation: Mapping[str, Any]) -> dict[str, Any]:
    limits = contract.TRAINING
    binding = cid_bytes(
        canonical_json_bytes(
            {
                "preparation_cid": preparation["preparation_cid"],
                "training": limits,
                "model_config": contract.MODEL_CONFIG,
            }
        )
    )
    folder = root / "fit"
    if (folder / "fit.json").exists():
        return _existing_result(root, preparation, binding)

    start_path = folder / "started.json"
    if start_path.exists():
        started = release._read_json(start_path, cid_field="start_cid")
        if started["binding_cid"] != binding:
            raise ValueError("fit start belongs to a different preparation")
    else:
        started = release._with_cid(
            {
                "schema": "uor-r4.zoology-cyclic-facts-fit-start/1",
                "binding_cid": binding,
                "started_unix_seconds": time.time(),
            },
            "start_cid",
        )
        release._write_exclusive_json(start_path, started)
    began = time.monotonic()
    carried = 0.0

    def elapsed() -> float:
        return max(
            carried + time.monotonic() - began,
            time.time() - started["started_unix_seconds"],
        )

    release._configure_cpu(limits["threads"])
    set_zoology_seed(limits["seed"])
    model = ZoologyFigure2Model(ZoologyFigure2Config(**contract.MODEL_CONFIG))
    optimizer = torch.optim.AdamW(
        model.parameters(),
        lr=limits["learning_rate"],
        weight_decay=limits["weight_decay"],
    )
    scheduler = torch.optim.lr_scheduler.CosineAnnealingLR(
        optimizer, T_max=limits["cosine_blocks"], eta_min=0.0
    )
    checkpoint_path = folder / "checkpoint.pt"
    saved = (
        torch.load(checkpoint_path, map_location="cpu", weights_only=True)
        if checkpoint_path.exists()
        else None
    )
    updates = 0
    phase = "supported"
    history: list[dict[str, Any]] = []
    accumulator = _empty_block()
    step_seconds: list[float] = []
    admission: dict[str, Any] | None = None
    admission_interruption: dict[str, Any] | None = None
    supported_presentations = unknown_presentations = 0
    numerical_failure: dict[str, Any] | None = None
    peak_rss = release._peak_rss_bytes()
    if saved is not None:
        if (
            saved["binding_cid"] != binding
            or saved["model_config"] != contract.MODEL_CONFIG
            or saved["start_cid"] != started["start_cid"]
        ):
            raise ValueError("checkpoint belongs to a different English fit")
        updates, phase = saved["completed_updates"], saved["phase"]
        history, accumulator = saved["history"], saved["accumulator"]
        step_seconds, admission = saved["step_seconds"], saved["admission"]
        admission_interruption = saved["admission_interruption"]
        supported_presentations = saved["supported_presentations"]
        unknown_presentations = saved["unknown_presentations"]
        carried = saved["elapsed_seconds"]
        numerical_failure = saved["numerical_failure"]
        peak_rss = max(peak_rss, saved["peak_rss_bytes"])
        model.load_state_dict(saved["model"])
        optimizer.load_state_dict(saved["optimizer"])
        scheduler.load_state_dict(saved["scheduler"])
    expected_phase = "mixed" if updates >= limits["supported_updates"] else "supported"
    if phase != expected_phase:
        raise ValueError("checkpoint curriculum phase differs from its update count")
    sampler = CyclingBatches(
        data.load_training(root / "data", mixed=phase == "mixed"),
        batch_size=limits["batch_size"],
    )
    if saved is not None:
        sampler.load_state_dict(saved["sampler"])
        torch.set_rng_state(saved["torch_rng_state"])

    complete_blocks = updates // limits["updates_per_block"]
    if (
        not 0 <= updates <= limits["total_updates"]
        or len(history) != complete_blocks
        or scheduler.last_epoch != complete_blocks
        or accumulator["updates"] != updates % limits["updates_per_block"]
        or accumulator["decisions"] != accumulator["updates"] * limits["batch_size"]
        or supported_presentations + unknown_presentations
        != updates * limits["batch_size"]
        or len(step_seconds) != min(updates, limits["admission_updates"])
        or any(not math.isfinite(value) or value <= 0 for value in step_seconds)
        or (admission is None) != (updates < limits["admission_updates"])
        or (
            admission is not None
            and not admission["passed"]
            and updates != limits["admission_updates"]
        )
    ):
        raise ValueError(
            "checkpoint update, schedule, admission or query counts differ"
        )
    phase_updates = updates - (limits["supported_updates"] if phase == "mixed" else 0)
    batches_per_cycle = (10240 if phase == "mixed" else 8192) // limits["batch_size"]
    sampler_state = sampler.state_dict()
    expected_cycles = (phase_updates + batches_per_cycle - 1) // batches_per_cycle
    expected_cursor = (
        ((phase_updates - 1) % batches_per_cycle + 1) * limits["batch_size"]
        if phase_updates
        else 0
    )
    if (
        sampler_state["cycles"] != expected_cycles
        or sampler_state["cursor"] != expected_cursor
        or not 0
        <= unknown_presentations
        <= max(0, updates - limits["supported_updates"]) * limits["batch_size"]
    ):
        raise ValueError("checkpoint sampler position or UNKNOWN counts differ")
    for index, row in enumerate(history):
        if (
            row["block"] != index + 1
            or row["completed_updates"] != (index + 1) * limits["updates_per_block"]
            or row["train"]["updates"] != limits["updates_per_block"]
            or row["train"]["decisions"]
            != limits["updates_per_block"] * limits["batch_size"]
        ):
            raise ValueError("checkpoint construction history differs")

    admission_marker = folder / "admission-inflight.json"
    if admission_marker.exists():
        interrupted = release._read_json(admission_marker, cid_field="step_cid")
        if interrupted["binding_cid"] != binding:
            raise ValueError("admission update belongs to a different fit")
        if interrupted["update"] == updates:
            # The checkpoint became durable before marker cleanup.
            admission_marker.unlink()
        elif interrupted["update"] == updates + 1:
            # It is unknowable whether this update ran before interruption.
            # Export the retained checkpoint instead of repeating its timing.
            admission_interruption = {
                "unresolved_update": interrupted["update"],
                "uncommitted_update_may_have_run": True,
            }
        else:
            raise ValueError("admission marker and checkpoint clocks differ")

    def update_peak() -> None:
        nonlocal peak_rss
        peak_rss = max(peak_rss, release._peak_rss_bytes())

    def save() -> None:
        update_peak()
        _save_checkpoint(
            checkpoint_path,
            {
                "schema": "uor-r4.zoology-cyclic-facts-fit-checkpoint/1",
                "binding_cid": binding,
                "start_cid": started["start_cid"],
                "model_config": asdict(model.config),
                "model": model.state_dict(),
                "optimizer": optimizer.state_dict(),
                "scheduler": scheduler.state_dict(),
                "sampler": sampler.state_dict(),
                "phase": phase,
                "completed_updates": updates,
                "history": history,
                "accumulator": accumulator,
                "supported_presentations": supported_presentations,
                "unknown_presentations": unknown_presentations,
                "step_seconds": step_seconds,
                "admission": admission,
                "admission_interruption": admission_interruption,
                "numerical_failure": numerical_failure,
                "torch_rng_state": torch.get_rng_state(),
                "elapsed_seconds": elapsed(),
                "peak_rss_bytes": peak_rss,
            },
        )

    def progress() -> None:
        record = {
            "completed_updates": updates,
            "remaining_updates": limits["total_updates"] - updates,
            "phase": phase,
            "completed_blocks": len(history),
            "elapsed_seconds": elapsed(),
            "peak_rss_bytes": peak_rss,
            "admission": admission,
            "latest_construction_block": history[-1] if history else None,
        }
        release._atomic_json(folder / "progress.json", record)
        print(canonical_json_bytes(record).decode().rstrip(), flush=True)

    if saved is None:
        save()
    model.train()
    while updates < limits["total_updates"]:
        update_peak()
        if (
            numerical_failure is not None
            or admission_interruption is not None
            or admission is not None
            and not admission["passed"]
            or elapsed() >= limits["max_elapsed_seconds"]
            or peak_rss > limits["max_rss_bytes"]
        ):
            break
        if updates < limits["admission_updates"]:
            release._write_exclusive_json(
                admission_marker,
                release._with_cid(
                    {"binding_cid": binding, "update": updates + 1}, "step_cid"
                ),
            )
        step_began = time.monotonic()
        batch = sampler.next_batch()
        batch = augment_training_batch(batch, completed_updates=updates)
        unknown_count = int((batch[2] == data.UNKNOWN_ID).sum())
        measured = _step(model, optimizer, batch)
        duration = time.monotonic() - step_began
        updates += 1
        unknown_presentations += unknown_count
        supported_presentations += int(batch[2].numel()) - unknown_count
        if not math.isfinite(measured["loss_sum"]):
            numerical_failure = {"reason": "nonfinite_training_loss", "update": updates}
            measured["loss_sum"] = 0.0
        for name in accumulator:
            accumulator[name] += measured[name]
        if updates <= limits["admission_updates"]:
            step_seconds.append(duration)
        if updates == limits["admission_updates"]:
            update_peak()
            spent = elapsed()
            mean_step = math.fsum(step_seconds) / len(step_seconds)
            projected_training = (
                limits["projection_safety_factor"]
                * mean_step
                * (limits["total_updates"] - updates)
            )
            projected_remaining = (
                projected_training + limits["evaluation_allowance_seconds"]
            )
            admission = {
                "measured_updates": updates,
                "step_seconds": list(step_seconds),
                "mean_step_seconds": mean_step,
                "elapsed_seconds": spent,
                "projected_remaining_training_seconds": projected_training,
                "evaluation_allowance_seconds": limits["evaluation_allowance_seconds"],
                "projected_remaining_seconds": projected_remaining,
                "available_seconds": limits["max_elapsed_seconds"] - spent,
                "peak_rss_bytes": peak_rss,
                "passed": projected_remaining <= limits["max_elapsed_seconds"] - spent
                and peak_rss <= limits["max_rss_bytes"],
            }
        if updates % limits["updates_per_block"] == 0:
            history.append(
                {
                    "block": len(history) + 1,
                    "completed_updates": updates,
                    "phase": phase,
                    "learning_rate": float(optimizer.param_groups[0]["lr"]),
                    "train": {
                        **accumulator,
                        "online_top1_rate": accumulator["correct"]
                        / accumulator["decisions"],
                        "online_nll_nats": accumulator["loss_sum"]
                        / accumulator["decisions"]
                        if numerical_failure is None
                        else None,
                    },
                    "elapsed_seconds": elapsed(),
                }
            )
            scheduler.step()
            accumulator = _empty_block()
        if updates == limits["supported_updates"]:
            phase = "mixed"
            sampler = CyclingBatches(
                data.load_training(root / "data", mixed=True),
                batch_size=limits["batch_size"],
            )
        if (
            updates <= limits["admission_updates"]
            or updates % limits["checkpoint_interval"] == 0
            or updates % limits["updates_per_block"] == 0
            or updates == limits["supported_updates"]
            or numerical_failure is not None
        ):
            save()
            if updates <= limits["admission_updates"]:
                admission_marker.unlink()
        if (
            updates == limits["admission_updates"]
            or updates % limits["updates_per_block"] == 0
        ):
            progress()

    save()
    payload = release._artifact_payload(model, learning_rate=limits["learning_rate"])
    _write_or_match(folder / "model.safetensors", payload)
    _validate_preparation(root, preparation)
    update_peak()
    spent = elapsed()
    status = (
        "INCOMPLETE_NUMERICAL"
        if numerical_failure is not None
        else "UNAVAILABLE_TRAJECTORY_ADMISSION"
        if admission_interruption is not None
        or admission is not None
        and not admission["passed"]
        else "INCOMPLETE_RESOURCE"
        if updates < limits["total_updates"]
        or spent >= limits["max_elapsed_seconds"]
        or peak_rss > limits["max_rss_bytes"]
        else "FIT_COMPLETE"
    )
    result = release._with_cid(
        {
            "schema": "uor-r4.zoology-cyclic-facts-fit/1",
            "issue": contract.ISSUE,
            "preparation_cid": preparation["preparation_cid"],
            "binding_cid": binding,
            "training": dict(limits),
            "status": status,
            "completed_updates": updates,
            "phase": phase,
            "blocks": len(history),
            "history": history,
            "partial_block": accumulator,
            "admission": admission,
            "admission_interruption": admission_interruption,
            "numerical_failure": numerical_failure,
            "elapsed_seconds": spent,
            "peak_rss_bytes": peak_rss,
            "work": {
                "optimizer_updates": updates,
                "supported_phase_updates": min(updates, limits["supported_updates"]),
                "mixed_phase_updates": max(0, updates - limits["supported_updates"]),
                "train_query_presentations": supported_presentations
                + unknown_presentations,
                "supported_presentations": supported_presentations,
                "unknown_presentations": unknown_presentations,
                "development_decisions": 0,
                "old_model_reads": 0,
                "model_frame_reads": 0,
            },
            "artifact": {
                **_record(folder / "model.safetensors", root),
                "state_cid": _state_cid(model),
                "config": asdict(model.config),
            },
            "checkpoint": _record(checkpoint_path, root),
            "augmentation": rotation_ledger(updates, unknown_presentations),
        },
        "fit_cid",
    )
    release._write_exclusive_json(folder / "fit.json", result)
    progress()
    return result
