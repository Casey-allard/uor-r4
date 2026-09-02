# SPDX-License-Identifier: Apache-2.0
"""One clock-corrected transfer, reusing the immutable #1053 cell and bytes."""

from __future__ import annotations

import time
from collections.abc import Mapping, Sequence
from dataclasses import asdict
from pathlib import Path
from typing import Any

import torch
from safetensors.torch import load as load_safetensors
from torch import Tensor

from ..provenance import canonical_json_bytes, cid_bytes
from ..zoology_control.model import set_zoology_seed
from ..zoology_release import development as release
from ..zoology_transfer import development as previous
from . import contract
from .sampling import CyclingBatches

ISSUE = 1055
POLICY = "ZoologyOptimizerClockTransferV1"
UPDATES_PER_BLOCK = 196
MAXIMUM_BLOCKS = 20
MAXIMUM_UPDATES = 3920
COSINE_BLOCKS = 64
CHECKPOINT_INTERVAL = 16
BATCH_SIZE = 512
QUERIES = 8
DEVELOPMENT_ROWS = 1024
LEARNING_RATE = previous.LEARNING_RATE
BUDGET_SECONDS = 1800.0
PREFLIGHT_PATH = "preflight.json"
RESULT_PATH = "result.json"


def _select(records: Sequence[Mapping[str, Any]]) -> dict[str, Any] | None:
    if len(records) != 3 or {row["plan"]["threads"] for row in records} != {1, 4, 8}:
        raise ValueError("CPU plan matrix differs")
    eligible = [
        row
        for row in records
        if row["stable"]
        and row["repeat_deterministic"]
        and row["projected_primary_seconds"] <= BUDGET_SECONDS
        and row["peak_rss_bytes"] <= previous.MEMORY_CEILING_BYTES
    ]
    return (
        dict(
            min(
                eligible,
                key=lambda row: (
                    row["projected_primary_seconds"],
                    row["plan"]["threads"],
                ),
            )
        )
        if eligible
        else None
    )


def _validate_preflight(
    preparation: Mapping[str, Any], observed: Mapping[str, Any]
) -> None:
    if (
        observed["preparation_cid"] != preparation["preparation_cid"]
        or observed["implementation"] != preparation["implementation"]
        or observed["reused_c0"] != preparation["reused_c0"]
    ):
        raise ValueError("preflight binding differs")
    selected = _select(observed["plans"])
    if selected != observed["selected"] or observed["passed"] != bool(selected):
        raise ValueError("preflight selection differs")


def preflight(root: Path) -> dict[str, Any]:
    root = root.resolve()
    preparation = contract.validate_preparation(root)
    if (root / PREFLIGHT_PATH).exists():
        result = release._read_json(root / PREFLIGHT_PATH, cid_field="preflight_cid")
        _validate_preflight(preparation, result)
        return result
    release._write_exclusive_json(
        root / "preflight-started.json",
        {"preparation_cid": preparation["preparation_cid"]},
    )
    began = time.monotonic()
    records = []
    for threads in (1, 4, 8):
        # Reuse the immutable exact-shape timing worker, not its epoch projection.
        row = previous._spawn_probe(threads, Path(preparation["predecessor_root"]))
        row["previous_64_epoch_projection"] = row["projected_primary_seconds"]
        row["projected_primary_seconds"] = 1.25 * (
            MAXIMUM_UPDATES * max(row["training_seconds"]) / row["timed_batches"]
            + MAXIMUM_BLOCKS * max(row["development_seconds"])
        )
        records.append(row)
    selected = _select(records)
    result = release._with_cid(
        {
            "schema": "uor-r4.zoology-clock-preflight/1",
            "issue": ISSUE,
            "policy": POLICY,
            "preparation_cid": preparation["preparation_cid"],
            "implementation": preparation["implementation"],
            "reused_c0": preparation["reused_c0"],
            "plans": records,
            "selected": selected,
            "passed": bool(selected),
            "elapsed_seconds": time.monotonic() - began,
            "cuda": "FORBIDDEN",
            "mps": "FORBIDDEN",
        },
        "preflight_cid",
    )
    contract.validate_preparation(root)
    release._write_exclusive_json(root / PREFLIGHT_PATH, result)
    return result


def _binding(preparation: Mapping[str, Any], admitted: Mapping[str, Any]) -> str:
    return cid_bytes(
        canonical_json_bytes(
            {
                "preparation_cid": preparation["preparation_cid"],
                "preflight_cid": admitted["preflight_cid"],
                "plan": admitted["selected"]["plan"],
            }
        )
    )


def _history_pass(history: Sequence[Mapping[str, Any]]) -> bool:
    if len(history) > MAXIMUM_BLOCKS:
        raise ValueError("too many source-clock blocks")
    for index, row in enumerate(history):
        passed = release._source_pass(row["development"]["top1_rate"])
        if (
            row["block"] != index + 1
            or row["completed_updates"] != (index + 1) * UPDATES_PER_BLOCK
            or row["strict_source_pass"] is not passed
        ):
            raise ValueError("clock history differs")
        if passed and index != len(history) - 1:
            raise ValueError("updates after a passing source block")
    return bool(history and history[-1]["strict_source_pass"])


def _empty_accumulator() -> dict[str, Any]:
    return {"updates": 0, "decisions": 0, "correct": 0, "loss_sum": 0.0}


def _step(model: Any, optimizer: Any, batch: Sequence[Tensor]) -> dict[str, Any]:
    optimizer.zero_grad()
    output = model.forward_selected(*batch)
    if output.loss is None:
        raise ValueError("missing selected-query loss")
    output.loss.backward()
    optimizer.step()
    count = int(batch[2].numel())
    return {
        "updates": 1,
        "decisions": count,
        "correct": int((output.logits.detach().argmax(dim=-1) == batch[2]).sum()),
        "loss_sum": float(output.loss.detach()) * count,
    }


def _primary(
    root: Path, tensors: Mapping[str, Tensor], *, threads: int, binding_cid: str
) -> dict[str, Any]:
    folder = root / "primary"
    # Recovery after primary publication must use the admitted CPU plan for
    # the still-pending control, even when no optimizer is reconstructed.
    release._configure_cpu(threads)
    if (folder / "result.json").exists():
        observed = release._read_json(folder / "result.json", cid_field="primary_cid")
        if observed["binding_cid"] != binding_cid:
            raise ValueError("primary binding differs")
        return observed
    set_zoology_seed(previous.SEED)
    model = previous._new_model()
    optimizer = torch.optim.AdamW(
        model.parameters(), lr=LEARNING_RATE, weight_decay=previous.WEIGHT_DECAY
    )
    scheduler = torch.optim.lr_scheduler.CosineAnnealingLR(
        optimizer, T_max=COSINE_BLOCKS, eta_min=0.0
    )
    sampler = CyclingBatches(tensors, batch_size=BATCH_SIZE)
    _, development = previous._loaders(tensors)
    history: list[dict[str, Any]] = []
    accumulator = _empty_accumulator()
    updates = 0
    carried = 0.0
    evaluation_rng = torch.get_rng_state().clone()
    checkpoint_path = folder / "checkpoint.pt"
    if checkpoint_path.exists():
        saved = torch.load(checkpoint_path, map_location="cpu", weights_only=True)
        if saved["binding_cid"] != binding_cid or saved["model_config"] != asdict(
            model.config
        ):
            raise ValueError("checkpoint belongs to a different clock transfer")
        model.load_state_dict(saved["model"])
        optimizer.load_state_dict(saved["optimizer"])
        scheduler.load_state_dict(saved["scheduler"])
        sampler.load_state_dict(saved["sampler"])
        updates, history, accumulator = (
            saved["completed_updates"],
            saved["history"],
            saved["accumulator"],
        )
        carried, evaluation_rng = saved["elapsed_seconds"], saved["evaluation_rng"]
        torch.set_rng_state(saved["torch_rng_state"])
    passed = _history_pass(history)
    if (
        updates < len(history) * UPDATES_PER_BLOCK
        or updates > MAXIMUM_UPDATES
        or accumulator["updates"] != updates - len(history) * UPDATES_PER_BLOCK
        or accumulator["updates"] > UPDATES_PER_BLOCK
        or accumulator["decisions"] != accumulator["updates"] * BATCH_SIZE * QUERIES
    ):
        raise ValueError("checkpoint optimizer/block counts differ")
    if scheduler.last_epoch != len(history) - int(passed):
        raise ValueError("checkpoint cosine clock differs")
    if passed and accumulator["updates"]:
        raise ValueError("checkpoint advanced after its passing block")
    began = time.monotonic()
    model.train()

    def elapsed() -> float:
        return carried + time.monotonic() - began

    def save() -> None:
        previous._save_checkpoint(
            checkpoint_path,
            {
                "binding_cid": binding_cid,
                "model_config": asdict(model.config),
                "model": model.state_dict(),
                "optimizer": optimizer.state_dict(),
                "scheduler": scheduler.state_dict(),
                "sampler": sampler.state_dict(),
                "completed_updates": updates,
                "history": history,
                "accumulator": accumulator,
                "evaluation_rng": evaluation_rng,
                "torch_rng_state": torch.get_rng_state(),
                "elapsed_seconds": elapsed(),
            },
        )

    def progress() -> None:
        spent = elapsed()
        record = {
            "issue": ISSUE,
            "completed_updates": updates,
            "remaining_updates": MAXIMUM_UPDATES - updates,
            "total_updates": MAXIMUM_UPDATES,
            "completed_blocks": len(history),
            "updates_per_second": updates / spent if spent else 0.0,
            "eta_seconds": spent * (MAXIMUM_UPDATES - updates) / updates
            if updates
            else None,
            "elapsed_seconds": spent,
            "peak_rss_bytes": release._peak_rss_bytes(),
            "latest_development": history[-1]["development"] if history else None,
        }
        release._atomic_json(folder / "progress.json", record)
        print(
            f"#1055 updates={updates}/{MAXIMUM_UPDATES} blocks={len(history)}/{MAXIMUM_BLOCKS} wall={spent:.1f}s eta={record['eta_seconds']:.1f}s"
            + (
                f" dev={history[-1]['development']['top1_rate']:.6%}" if history else ""
            ),
            flush=True,
        )

    incomplete = False
    while not passed:
        # A saved last batch may still need its block evaluation. Do that before
        # checking the maximum update count, and do not start another traversal.
        if accumulator["updates"] == UPDATES_PER_BLOCK:
            evaluation_rng = torch.get_rng_state().clone()
            score = previous._score(model, development)
            passed = release._source_pass(score["top1_rate"])
            row = {
                "block": len(history) + 1,
                "completed_updates": updates,
                "learning_rate": float(optimizer.param_groups[0]["lr"]),
                "train": {
                    **accumulator,
                    "online_top1_rate": accumulator["correct"]
                    / accumulator["decisions"],
                    "online_nll_nats": accumulator["loss_sum"]
                    / accumulator["decisions"],
                },
                "development": score,
                "strict_source_pass": passed,
                "elapsed_seconds": elapsed(),
            }
            history.append(row)
            if not passed:
                scheduler.step()
            accumulator = _empty_accumulator()
            save()
            progress()
            model.train()
        if passed or updates == MAXIMUM_UPDATES:
            break
        if (
            elapsed() >= BUDGET_SECONDS
            or release._peak_rss_bytes() > previous.MEMORY_CEILING_BYTES
        ):
            incomplete = True
            save()
            break
        measured = _step(model, optimizer, sampler.next_batch())
        for name in accumulator:
            accumulator[name] += measured[name]
        updates += 1
        if updates % CHECKPOINT_INTERVAL == 0:
            save()
            progress()
    status = (
        "PRIMARY_POSITIVE"
        if passed
        else "INCOMPLETE"
        if incomplete
        else "CLOCK_MATCHED_TRANSFER_MISS"
    )
    payload = release._artifact_payload(model, learning_rate=LEARNING_RATE)
    previous._write_or_match(folder / "model.safetensors", payload)
    rng_payload = release._canonical_safetensors({"evaluation_rng": evaluation_rng})
    previous._write_or_match(folder / "evaluation-rng.safetensors", rng_payload)
    state = {
        name: value
        for name, value in model.state_dict().items()
        if name != "lm_head.weight"
    }
    result = release._with_cid(
        {
            "schema": "uor-r4.zoology-clock-primary/1",
            "issue": ISSUE,
            "policy": POLICY,
            "binding_cid": binding_cid,
            "status": status,
            "passed": passed,
            "completed_updates": updates,
            "blocks": len(history),
            "history": history,
            "final_development": history[-1]["development"]
            if history and not accumulator["updates"]
            else None,
            "elapsed_seconds": elapsed(),
            "peak_rss_bytes": release._peak_rss_bytes(),
            "artifact": {
                "path": "primary/model.safetensors",
                "bytes": len(payload),
                "cid": cid_bytes(payload),
                "state_cid": release._tensor_mapping_cid(state),
                "config": asdict(model.config),
            },
            "evaluation_rng": {
                "path": "primary/evaluation-rng.safetensors",
                "cid": cid_bytes(rng_payload),
            },
            "work": {
                "optimizer_updates": updates,
                "train_query_presentations": updates * BATCH_SIZE * QUERIES,
                "development_query_presentations": sum(
                    row["development"]["decisions"] for row in history
                ),
            },
        },
        "primary_cid",
    )
    release._write_exclusive_json(folder / "result.json", result)
    return result


def run(root: Path) -> dict[str, Any]:
    root = root.resolve()
    preparation = contract.validate_preparation(root)
    if (root / RESULT_PATH).exists():
        return verify(root)
    admitted = release._read_json(root / PREFLIGHT_PATH, cid_field="preflight_cid")
    _validate_preflight(preparation, admitted)
    primary = None
    intervention: dict[str, Any] = {"status": "NOT_RUN_PRIMARY_MISS"}
    verdict = "NOT_RUN_PREFLIGHT"
    began = time.monotonic()
    if admitted["passed"]:
        binding = _binding(preparation, admitted)
        previous._write_or_match(
            root / "run-started.json", canonical_json_bytes({"binding_cid": binding})
        )
        primary = _primary(
            root,
            contract.load_dataset(root, preparation),
            threads=admitted["selected"]["plan"]["threads"],
            binding_cid=binding,
        )
        verdict = primary["status"]
        if primary["passed"]:
            model = previous._load_artifact(root, primary)
            score = previous._score(
                model,
                previous._control_loader(contract.load_control(root, preparation)),
            )
            drop = primary["final_development"]["top1_rate"] - score["top1_rate"]
            intervention = release._with_cid(
                {
                    "status": "COMPLETE",
                    "dataset": preparation["control"],
                    "score": score,
                    "drop": drop,
                    "required_drop": previous.REQUIRED_DROP,
                    "passed": drop >= previous.REQUIRED_DROP,
                },
                "control_cid",
            )
            verdict = (
                "CLOCK_MATCHED_TRANSFER_PASSES"
                if intervention["passed"]
                else "NONASSOCIATIVE_SHORTCUT"
            )
    result = release._with_cid(
        {
            "schema": "uor-r4.zoology-clock-result/1",
            "issue": ISSUE,
            "policy": POLICY,
            "preparation_cid": preparation["preparation_cid"],
            "preflight_cid": admitted["preflight_cid"],
            "implementation": preparation["implementation"],
            "dataset": preparation["dataset"],
            "primary": primary,
            "control": intervention,
            "decision": {
                "verdict": verdict,
                "passed": verdict == "CLOCK_MATCHED_TRANSFER_PASSES",
            },
            "elapsed_seconds": time.monotonic() - began,
            "read_ledger": {
                "preparation": preparation["read_ledger"],
                "model_role_reads": 0,
                "model_geometry_reads": 0,
                "future_value_reads": 0,
                "sealed_reads": 0,
                "teacher_calls": 0,
                "provider_calls": 0,
                "predecessor_weight_reads": 0,
                "control_query_decisions": intervention.get("score", {}).get(
                    "decisions", 0
                ),
            },
        },
        "result_cid",
    )
    contract.validate_preparation(root)
    release._write_exclusive_json(root / RESULT_PATH, result)
    return result


def verify(root: Path) -> dict[str, Any]:
    root = root.resolve()
    preparation = contract.validate_preparation(root)
    admitted = release._read_json(root / PREFLIGHT_PATH, cid_field="preflight_cid")
    _validate_preflight(preparation, admitted)
    result = release._read_json(root / RESULT_PATH, cid_field="result_cid")
    if (
        result["preparation_cid"] != preparation["preparation_cid"]
        or result["preflight_cid"] != admitted["preflight_cid"]
        or result["implementation"] != preparation["implementation"]
        or result["dataset"] != preparation["dataset"]
    ):
        raise ValueError("result causal binding differs")
    primary = result["primary"]
    expected = "NOT_RUN_PREFLIGHT"
    if bool(primary is not None) != admitted["passed"]:
        raise ValueError("primary presence differs from admission")
    if primary is not None:
        if (
            primary["binding_cid"] != _binding(preparation, admitted)
            or release._read_json(root / "primary/result.json", cid_field="primary_cid")
            != primary
        ):
            raise ValueError("primary binding differs")
        history = primary["history"]
        passed = _history_pass(history)
        updates = primary["completed_updates"]
        if (
            primary["passed"] != passed
            or primary["blocks"] != len(history)
            or not len(history) * UPDATES_PER_BLOCK <= updates <= MAXIMUM_UPDATES
        ):
            raise ValueError("primary clock accounting differs")
        if primary["work"] != {
            "optimizer_updates": updates,
            "train_query_presentations": updates * BATCH_SIZE * QUERIES,
            "development_query_presentations": len(history)
            * DEVELOPMENT_ROWS
            * QUERIES,
        }:
            raise ValueError("primary query accounting differs")
        expected = (
            "PRIMARY_POSITIVE"
            if passed
            else "CLOCK_MATCHED_TRANSFER_MISS"
            if updates == MAXIMUM_UPDATES
            else "INCOMPLETE"
        )
        if primary["status"] != expected or (
            passed and updates != len(history) * UPDATES_PER_BLOCK
        ):
            raise ValueError("primary terminal differs")
        release._configure_cpu(admitted["selected"]["plan"]["threads"])
        model = previous._load_artifact(root, primary)
        checkpoint = torch.load(
            root / "primary/checkpoint.pt", map_location="cpu", weights_only=True
        )
        exported_state = {
            name: value
            for name, value in checkpoint["model"].items()
            if name != "lm_head.weight"
        }
        if (
            checkpoint["binding_cid"] != primary["binding_cid"]
            or checkpoint["completed_updates"] != updates
            or checkpoint["history"] != history
            or release._tensor_mapping_cid(exported_state)
            != primary["artifact"]["state_cid"]
            or any(
                int(value["step"]) != updates
                for value in checkpoint["optimizer"]["state"].values()
            )
        ):
            raise ValueError("checkpoint/artifact mismatch")
        complete_block = updates == len(history) * UPDATES_PER_BLOCK and bool(history)
        if primary["final_development"] != (
            history[-1]["development"] if complete_block else None
        ):
            raise ValueError("final evaluation does not match artifact clock")
        if complete_block:
            rng_payload = (root / primary["evaluation_rng"]["path"]).read_bytes()
            if cid_bytes(rng_payload) != primary["evaluation_rng"]["cid"]:
                raise ValueError("evaluation RNG changed")
            _, development = previous._loaders(contract.load_dataset(root, preparation))
            torch.set_rng_state(load_safetensors(rng_payload)["evaluation_rng"])
            if previous._score(model, development) != primary["final_development"]:
                raise ValueError("fresh artifact inference differs")
        if passed:
            intervention = result["control"]
            release._verify_self_cid(intervention, "control_cid")
            score = previous._score(
                model,
                previous._control_loader(contract.load_control(root, preparation)),
            )
            drop = primary["final_development"]["top1_rate"] - score["top1_rate"]
            control_pass = drop >= previous.REQUIRED_DROP
            if (
                intervention["score"] != score
                or intervention["drop"] != drop
                or intervention["passed"] != control_pass
                or intervention["dataset"] != preparation["control"]
                or intervention["required_drop"] != previous.REQUIRED_DROP
            ):
                raise ValueError("binding control differs")
            expected = (
                "CLOCK_MATCHED_TRANSFER_PASSES"
                if control_pass
                else "NONASSOCIATIVE_SHORTCUT"
            )
    if (primary is None or not primary["passed"]) and result["control"] != {
        "status": "NOT_RUN_PRIMARY_MISS"
    }:
        raise ValueError("control crossed primary boundary")
    if result["decision"] != {
        "verdict": expected,
        "passed": expected == "CLOCK_MATCHED_TRANSFER_PASSES",
    }:
        raise ValueError("terminal decision differs")
    return result
