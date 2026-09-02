# SPDX-License-Identifier: Apache-2.0
"""Continue, never restart, the immutable #1055 optimizer trajectory."""

from __future__ import annotations

import math
import time
from collections.abc import Mapping, Sequence
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Any

import torch
from safetensors.torch import load as load_safetensors
from torch import Tensor

from ..provenance import canonical_json_bytes, cid_bytes
from ..zoology_clock import development as clock
from ..zoology_clock.sampling import CyclingBatches
from ..zoology_release import development as release
from ..zoology_transfer import development as previous
from . import contract

ISSUE = 1057
POLICY = "ZoologyCheckpointContinuationV1"
RESULT_PATH = "result.json"


@dataclass(frozen=True)
class _Limits:
    # The public commands always use these frozen values. Tiny private fixtures
    # exercise continuation semantics without another real model experiment.
    inherited_blocks: int = 20
    maximum_blocks: int = 40
    updates_per_block: int = 196
    batch_size: int = 512
    queries: int = 8
    train_rows: int = 8192
    development_rows: int = 1024
    checkpoint_interval: int = 16
    additional_budget_seconds: float = 1800.0

    @property
    def inherited_updates(self) -> int:
        return self.inherited_blocks * self.updates_per_block

    @property
    def maximum_updates(self) -> int:
        return self.maximum_blocks * self.updates_per_block


LIMITS = _Limits()


def _binding(preparation: Mapping[str, Any]) -> str:
    return cid_bytes(
        canonical_json_bytes(
            {
                "preparation_cid": preparation["preparation_cid"],
                "parent_checkpoint_cid": preparation["parent_checkpoint"]["cid"],
                "parent_binding_cid": preparation["parent_binding_cid"],
                "parent_history_cid": preparation["parent_history_cid"],
                "cpu_plan": preparation["cpu_plan"],
            }
        )
    )


def _history_pass(
    history: Sequence[Mapping[str, Any]],
    preparation: Mapping[str, Any],
    limits: _Limits,
) -> bool:
    inherited = preparation["parent_primary"]["history"]
    if (
        len(inherited) != limits.inherited_blocks
        or not limits.inherited_blocks <= len(history) <= limits.maximum_blocks
        or list(history[: limits.inherited_blocks]) != inherited
    ):
        raise ValueError("inherited history changed")
    for index, row in enumerate(history):
        passed = release._source_pass(row["development"]["top1_rate"])
        if (
            row["block"] != index + 1
            or row["completed_updates"] != (index + 1) * limits.updates_per_block
            or row["strict_source_pass"] is not passed
            or row["development"]["decisions"]
            != limits.development_rows * limits.queries
        ):
            raise ValueError("continuation history clock differs")
        if passed and (index < limits.inherited_blocks or index != len(history) - 1):
            raise ValueError("updates after a passing block")
    return bool(history[-1]["strict_source_pass"])


def _work(updates: int, blocks: int, limits: _Limits) -> dict[str, Any]:
    def counts(update_count: int, block_count: int) -> dict[str, int]:
        return {
            "optimizer_updates": update_count,
            "development_blocks": block_count,
            "train_query_presentations": update_count
            * limits.batch_size
            * limits.queries,
            "development_query_presentations": block_count
            * limits.development_rows
            * limits.queries,
        }

    return {
        "inherited": counts(limits.inherited_updates, limits.inherited_blocks),
        "additional": counts(
            updates - limits.inherited_updates, blocks - limits.inherited_blocks
        ),
        "total": counts(updates, blocks),
    }


def _validate_checkpoint(
    saved: Mapping[str, Any], preparation: Mapping[str, Any], limits: _Limits
) -> bool:
    if (
        saved["binding_cid"] != _binding(preparation)
        or saved["parent_checkpoint_cid"] != preparation["parent_checkpoint"]["cid"]
        or saved["parent_binding_cid"] != preparation["parent_binding_cid"]
        or saved["parent_history_cid"] != preparation["parent_history_cid"]
    ):
        raise ValueError("continuation checkpoint binding differs")
    passed = _history_pass(saved["history"], preparation, limits)
    updates, accumulator = saved["completed_updates"], saved["accumulator"]
    if (
        not limits.inherited_updates <= updates <= limits.maximum_updates
        or accumulator["updates"]
        != updates - len(saved["history"]) * limits.updates_per_block
        or not 0 <= accumulator["updates"] <= limits.updates_per_block
        or accumulator["decisions"]
        != accumulator["updates"] * limits.batch_size * limits.queries
        or (passed and accumulator["updates"])
        or saved["scheduler"]["last_epoch"] != len(saved["history"]) - int(passed)
        or saved["scheduler"]["T_max"] != clock.COSINE_BLOCKS
        or not saved["optimizer"]["state"]
        or any(
            int(row["step"]) != updates for row in saved["optimizer"]["state"].values()
        )
    ):
        raise ValueError("continuation checkpoint optimizer clock differs")
    sampler = saved["sampler"]
    rows = limits.train_rows
    if (sampler["cycles"] - 1) * (rows // limits.batch_size) + sampler[
        "cursor"
    ] // limits.batch_size != updates:
        raise ValueError("continuation sampler clock differs")
    inherited, additional = (
        saved["inherited_elapsed_seconds"],
        saved["additional_elapsed_seconds"],
    )
    if (
        not all(
            math.isfinite(value) and value >= 0 for value in (inherited, additional)
        )
        or saved["elapsed_seconds"] != inherited + additional
    ):
        raise ValueError("continuation elapsed accounting differs")
    return passed


def _primary(
    root: Path,
    tensors: Mapping[str, Tensor],
    preparation: Mapping[str, Any],
    *,
    limits: _Limits = LIMITS,
) -> dict[str, Any]:
    folder = root / "primary"
    release._configure_cpu(preparation["cpu_plan"]["threads"])
    binding = _binding(preparation)
    if (folder / "result.json").exists():
        result = release._read_json(folder / "result.json", cid_field="primary_cid")
        if result["binding_cid"] != binding:
            raise ValueError("cached continuation binding differs")
        return result
    checkpoint_path = folder / "checkpoint.pt"
    parent = contract.load_checkpoint(preparation)
    if checkpoint_path.exists():
        saved = torch.load(checkpoint_path, map_location="cpu", weights_only=True)
    else:
        saved = {
            **parent,
            "binding_cid": binding,
            "parent_checkpoint_cid": preparation["parent_checkpoint"]["cid"],
            "parent_binding_cid": preparation["parent_binding_cid"],
            "parent_history_cid": preparation["parent_history_cid"],
            "inherited_elapsed_seconds": parent["elapsed_seconds"],
            "additional_elapsed_seconds": 0.0,
        }
    passed = _validate_checkpoint(saved, preparation, limits)
    if saved["inherited_elapsed_seconds"] != parent["elapsed_seconds"]:
        raise ValueError("inherited elapsed time changed")
    model = previous._new_model()
    if saved["model_config"] != asdict(model.config):
        raise ValueError("continuation model config differs")
    optimizer = torch.optim.AdamW(
        model.parameters(),
        lr=previous.LEARNING_RATE,
        weight_decay=previous.WEIGHT_DECAY,
    )
    scheduler = torch.optim.lr_scheduler.CosineAnnealingLR(
        optimizer, T_max=clock.COSINE_BLOCKS, eta_min=0.0
    )
    sampler = CyclingBatches(tensors, batch_size=limits.batch_size)
    _, development = previous._loaders(tensors)
    model.load_state_dict(saved["model"])
    optimizer.load_state_dict(saved["optimizer"])
    scheduler.load_state_dict(saved["scheduler"])
    sampler.load_state_dict(saved["sampler"])
    updates, history, accumulator = (
        saved["completed_updates"],
        list(saved["history"]),
        dict(saved["accumulator"]),
    )
    inherited, carried = (
        saved["inherited_elapsed_seconds"],
        saved["additional_elapsed_seconds"],
    )
    evaluation_rng = saved["evaluation_rng"]
    # Construction/loaders may consume randomness. Restore the trajectory last.
    torch.set_rng_state(saved["torch_rng_state"])
    began = time.monotonic()
    model.train()

    def elapsed() -> float:
        return carried + time.monotonic() - began

    def save() -> None:
        additional = elapsed()
        previous._save_checkpoint(
            checkpoint_path,
            {
                "binding_cid": binding,
                "parent_checkpoint_cid": preparation["parent_checkpoint"]["cid"],
                "parent_binding_cid": preparation["parent_binding_cid"],
                "parent_history_cid": preparation["parent_history_cid"],
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
                "inherited_elapsed_seconds": inherited,
                "additional_elapsed_seconds": additional,
                "elapsed_seconds": inherited + additional,
            },
        )

    def progress() -> None:
        additional, done = elapsed(), updates - limits.inherited_updates
        remaining = limits.maximum_updates - updates
        eta = additional * remaining / done if done else None
        record = {
            "issue": ISSUE,
            "completed_updates": updates,
            "additional_updates": done,
            "remaining_updates": remaining,
            "maximum_updates": limits.maximum_updates,
            "completed_blocks": len(history),
            "inherited_elapsed_seconds": inherited,
            "additional_elapsed_seconds": additional,
            "elapsed_seconds": inherited + additional,
            "updates_per_second": done / additional if additional else 0.0,
            "eta_seconds": eta,
            "peak_rss_bytes": release._peak_rss_bytes(),
            "latest_development": history[-1]["development"],
        }
        release._atomic_json(folder / "progress.json", record)
        print(
            f"#1057 additional={done}/{limits.maximum_updates - limits.inherited_updates} "
            f"total={updates}/{limits.maximum_updates} blocks={len(history)}/{limits.maximum_blocks} "
            f"new_wall={additional:.1f}s eta={eta if eta is not None else 'unknown'}s "
            f"dev={history[-1]['development']['top1_rate']:.6%}",
            flush=True,
        )

    incomplete = False
    while not passed:
        # Evaluate a checkpointed complete block before the cap, without taking
        # another batch or drawing the next traversal's seeds.
        if accumulator["updates"] == limits.updates_per_block:
            evaluation_rng = torch.get_rng_state().clone()
            score = previous._score(model, development)
            passed = release._source_pass(score["top1_rate"])
            additional = elapsed()
            history.append(
                {
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
                    "elapsed_seconds": inherited + additional,
                    "additional_elapsed_seconds": additional,
                }
            )
            if not passed:
                scheduler.step()
            accumulator = clock._empty_accumulator()
            save()
            progress()
            model.train()
        if passed or updates == limits.maximum_updates:
            break
        if (
            elapsed() >= limits.additional_budget_seconds
            or release._peak_rss_bytes() > previous.MEMORY_CEILING_BYTES
        ):
            incomplete = True
            break
        measured = clock._step(model, optimizer, sampler.next_batch())
        for name in accumulator:
            accumulator[name] += measured[name]
        updates += 1
        if updates % limits.checkpoint_interval == 0:
            save()
            progress()
    save()
    payload = release._artifact_payload(model, learning_rate=previous.LEARNING_RATE)
    previous._write_or_match(folder / "model.safetensors", payload)
    rng_payload = release._canonical_safetensors({"evaluation_rng": evaluation_rng})
    previous._write_or_match(folder / "evaluation-rng.safetensors", rng_payload)
    checkpoint_payload = checkpoint_path.read_bytes()
    additional = elapsed()
    result = release._with_cid(
        {
            "schema": "uor-r4.zoology-continuation-primary/1",
            "issue": ISSUE,
            "policy": POLICY,
            "binding_cid": binding,
            "status": "PRIMARY_POSITIVE"
            if passed
            else "INCOMPLETE"
            if incomplete
            else "CONTINUATION_MISS",
            "passed": passed,
            "completed_updates": updates,
            "blocks": len(history),
            "history": history,
            "final_development": history[-1]["development"]
            if not accumulator["updates"]
            else None,
            "inherited_elapsed_seconds": inherited,
            "additional_elapsed_seconds": additional,
            "elapsed_seconds": inherited + additional,
            "peak_rss_bytes": release._peak_rss_bytes(),
            "artifact": {
                "path": "primary/model.safetensors",
                "bytes": len(payload),
                "cid": cid_bytes(payload),
                "state_cid": release._tensor_mapping_cid(
                    {
                        name: value
                        for name, value in model.state_dict().items()
                        if name != "lm_head.weight"
                    }
                ),
                "config": asdict(model.config),
            },
            "checkpoint": {
                "path": "primary/checkpoint.pt",
                "bytes": len(checkpoint_payload),
                "cid": cid_bytes(checkpoint_payload),
            },
            "evaluation_rng": {
                "path": "primary/evaluation-rng.safetensors",
                "cid": cid_bytes(rng_payload),
            },
            "work": _work(updates, len(history), limits),
        },
        "primary_cid",
    )
    release._write_exclusive_json(folder / "result.json", result)
    return result


def _control(
    root: Path, primary: Mapping[str, Any], preparation: Mapping[str, Any]
) -> dict[str, Any]:
    if not primary["passed"]:
        return {"status": "NOT_RUN_PRIMARY_MISS"}
    model = previous._load_artifact(root, primary)
    score = previous._score(
        model, previous._control_loader(contract.load_control(preparation))
    )
    drop = primary["final_development"]["top1_rate"] - score["top1_rate"]
    return release._with_cid(
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


def _decision(
    primary: Mapping[str, Any] | None, intervention: Mapping[str, Any]
) -> dict[str, Any]:
    verdict = "NOT_RUN_ADMISSION" if primary is None else primary["status"]
    if primary is not None and primary["passed"]:
        verdict = (
            "CONTINUATION_PASSES"
            if intervention["passed"]
            else "NONASSOCIATIVE_SHORTCUT"
        )
    return {"verdict": verdict, "passed": verdict == "CONTINUATION_PASSES"}


def run(root: Path) -> dict[str, Any]:
    root = root.resolve()
    preparation = contract.validate_preparation(root)
    if (root / RESULT_PATH).exists():
        return verify(root)
    primary = None
    intervention = {"status": "NOT_RUN_PRIMARY_MISS"}
    if preparation["admission"]["passed"]:
        previous._write_or_match(
            root / "run-started.json",
            canonical_json_bytes({"binding_cid": _binding(preparation)}),
        )
        primary = _primary(root, contract.load_dataset(preparation), preparation)
        intervention = _control(root, primary, preparation)
    result = release._with_cid(
        {
            "schema": "uor-r4.zoology-continuation-result/1",
            "issue": ISSUE,
            "policy": POLICY,
            "preparation_cid": preparation["preparation_cid"],
            "implementation": preparation["implementation"],
            "parent_checkpoint": preparation["parent_checkpoint"],
            "dataset": preparation["dataset"],
            "primary": primary,
            "control": intervention,
            "decision": _decision(primary, intervention),
            "read_ledger": {
                "model_role_reads": 0,
                "model_geometry_reads": 0,
                "future_value_reads": 0,
                "sealed_reads": 0,
                "teacher_calls": 0,
                "provider_calls": 0,
                "parent_checkpoint_restore": primary is not None,
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
    result = release._read_json(root / RESULT_PATH, cid_field="result_cid")
    if any(
        result[name] != preparation[name]
        for name in (
            "preparation_cid",
            "implementation",
            "parent_checkpoint",
            "dataset",
        )
    ):
        raise ValueError("continuation result binding differs")
    primary = result["primary"]
    if bool(primary is not None) != preparation["admission"]["passed"]:
        raise ValueError("continuation admission differs")
    expected_control = {"status": "NOT_RUN_PRIMARY_MISS"}
    if primary is not None:
        if (
            primary["binding_cid"] != _binding(preparation)
            or release._read_json(root / "primary/result.json", cid_field="primary_cid")
            != primary
        ):
            raise ValueError("continuation primary binding differs")
        checkpoint_payload = (root / primary["checkpoint"]["path"]).read_bytes()
        if (
            cid_bytes(checkpoint_payload) != primary["checkpoint"]["cid"]
            or len(checkpoint_payload) != primary["checkpoint"]["bytes"]
        ):
            raise ValueError("continuation checkpoint bytes changed")
        saved = torch.load(
            root / primary["checkpoint"]["path"], map_location="cpu", weights_only=True
        )
        parent = contract.load_checkpoint(preparation)
        passed = _validate_checkpoint(saved, preparation, LIMITS)
        updates, history = saved["completed_updates"], saved["history"]
        expected_status = (
            "PRIMARY_POSITIVE"
            if passed
            else "CONTINUATION_MISS"
            if updates == LIMITS.maximum_updates
            else "INCOMPLETE"
        )
        if (
            primary["passed"] != passed
            or primary["status"] != expected_status
            or primary["completed_updates"] != updates
            or primary["blocks"] != len(history)
            or primary["history"] != history
            or primary["work"] != _work(updates, len(history), LIMITS)
            or primary["inherited_elapsed_seconds"] != parent["elapsed_seconds"]
            or saved["inherited_elapsed_seconds"] != parent["elapsed_seconds"]
            or not math.isfinite(primary["additional_elapsed_seconds"])
            or primary["additional_elapsed_seconds"]
            < saved["additional_elapsed_seconds"]
            or primary["elapsed_seconds"]
            != primary["inherited_elapsed_seconds"]
            + primary["additional_elapsed_seconds"]
        ):
            raise ValueError("continuation result accounting differs")
        release._configure_cpu(preparation["cpu_plan"]["threads"])
        model = previous._load_artifact(root, primary)
        if (
            saved["model_config"] != primary["artifact"]["config"]
            or release._tensor_mapping_cid(
                {
                    name: value
                    for name, value in saved["model"].items()
                    if name != "lm_head.weight"
                }
            )
            != primary["artifact"]["state_cid"]
        ):
            raise ValueError("continuation checkpoint/artifact mismatch")
        tensors = contract.load_dataset(preparation)
        CyclingBatches(tensors, batch_size=LIMITS.batch_size).load_state_dict(
            saved["sampler"]
        )
        complete_block = not saved["accumulator"]["updates"]
        if primary["final_development"] != (
            history[-1]["development"] if complete_block else None
        ):
            raise ValueError("continuation final evaluation clock differs")
        if complete_block:
            rng_payload = (root / primary["evaluation_rng"]["path"]).read_bytes()
            if cid_bytes(rng_payload) != primary["evaluation_rng"]["cid"]:
                raise ValueError("continuation evaluation RNG changed")
            evaluation_rng = load_safetensors(rng_payload)["evaluation_rng"]
            if not torch.equal(evaluation_rng, saved["evaluation_rng"]):
                raise ValueError("continuation checkpoint/evaluation RNG mismatch")
            _, development = previous._loaders(tensors)
            torch.set_rng_state(evaluation_rng)
            if previous._score(model, development) != primary["final_development"]:
                raise ValueError("continuation fresh artifact inference differs")
        expected_control = _control(root, primary, preparation)
    if result["control"] != expected_control:
        raise ValueError("continuation binding control differs")
    if result["decision"] != _decision(primary, expected_control):
        raise ValueError("continuation terminal decision differs")
    return result
