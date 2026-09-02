# SPDX-License-Identifier: Apache-2.0
"""One source-positive CPU training arm on the unchanged open #1045 bytes.

The copied cell remains in the credited zoology_control package. Historical
#1049/#1050 implementations and evidence are never modified by this lifecycle.
"""

from __future__ import annotations

import math
import multiprocessing as mp
import os
import queue
import time
from collections.abc import Mapping, Sequence
from dataclasses import asdict
from pathlib import Path
from typing import Any

import torch
from blake3 import blake3
from safetensors.torch import load as load_safetensors
from torch import Tensor
from torch.nn import functional as F
from torch.utils.data import DataLoader, TensorDataset

from ..provenance import canonical_json_bytes, cid_bytes
from ..zoology_control import data as source_data
from ..zoology_control import development as control
from ..zoology_control.model import (
    ZoologyFigure2Config,
    ZoologyFigure2Model,
    set_zoology_seed,
)
from ..zoology_release import development as release
from . import contract

ISSUE = 1053
POLICY = "ZoologyExact1045TransferV1"
VOCAB_SIZE = 4096
CONTEXT = 120
QUERIES = 8
TRAIN_ROWS = 8192
DEVELOPMENT_ROWS = 1024
BATCH_SIZE = 512
SEED = 123
LEARNING_RATE = 0.00046415888336127773
MAXIMUM_EPOCHS = 64
WEIGHT_DECAY = 0.1
REQUIRED_DROP = 0.50
THREADS = (1, 4, 8)
# The issue binds a safety-adjusted admission projection. The additional
# runtime guard is checked between complete epochs, not a batch interrupt.
PRIMARY_BUDGET_SECONDS = 900.0
MEMORY_CEILING_BYTES = 8 * 1024**3
PREFLIGHT_PATH = "preflight/zoology-transfer-preflight.json"
RESULT_PATH = "run/zoology-transfer-result.json"


def _new_model() -> ZoologyFigure2Model:
    return ZoologyFigure2Model(
        ZoologyFigure2Config(
            vocab_size=VOCAB_SIZE,
            d_model=64,
            n_layers=2,
            num_heads=1,
            max_position_embeddings=CONTEXT,
            attention_dropout=0.1,
            embed_dropout=0.1,
            resid_dropout=0.0,
        )
    )


def _loaders(tensors: Mapping[str, Tensor]) -> tuple[DataLoader[Any], DataLoader[Any]]:
    # #1050's batch-512 loaders use the shared global Torch RNG, including the
    # shuffled development iterator; no new per-epoch RNG namespace is added.
    return release._loaders(tensors)


def _score(model: ZoologyFigure2Model, loader: DataLoader[Any]) -> dict[str, Any]:
    model.eval()
    correct = decisions = 0
    loss_sum = 0.0
    digest = blake3()
    with torch.inference_mode():
        for inputs, positions, targets in loader:
            logits = model.forward_selected(inputs, positions).logits
            flat = logits.reshape(-1, model.config.vocab_size)
            labels = targets.reshape(-1)
            correct += int((flat.argmax(dim=-1) == labels).sum())
            decisions += int(labels.numel())
            loss_sum += float(F.cross_entropy(flat, labels, reduction="sum"))
            digest.update(logits.detach().cpu().contiguous().numpy().tobytes(order="C"))
    if not decisions:
        raise ValueError("empty development score")
    return {
        "decisions": decisions,
        "top1_correct": correct,
        "top1_rate": correct / decisions,
        "nll_nats": loss_sum / decisions,
        "selected_logits_cid": f"blake3:{digest.hexdigest()}",
    }


def _train_epoch(
    model: ZoologyFigure2Model,
    optimizer: torch.optim.Optimizer,
    loader: DataLoader[Any],
) -> dict[str, Any]:
    model.train()
    decisions = correct = 0
    loss_sum = 0.0
    for inputs, positions, targets in loader:
        optimizer.zero_grad()
        output = model.forward_selected(inputs, positions, targets)
        if output.loss is None:
            raise RuntimeError("training batch lacks query loss")
        output.loss.backward()
        optimizer.step()
        count = int(targets.numel())
        decisions += count
        loss_sum += float(output.loss.detach()) * count
        correct += int((output.logits.detach().argmax(dim=-1) == targets).sum())
    return {
        "decisions": decisions,
        "online_top1_correct": correct,
        "online_top1_rate": correct / decisions,
        "online_nll_nats": loss_sum / decisions,
    }


def _probe_once(threads: int, tensors: Mapping[str, Tensor]) -> dict[str, Any]:
    release._configure_cpu(threads)
    durations, scores, losses, cpu_seconds = [], [], [], []
    for _ in range(2):
        set_zoology_seed(SEED)
        model = _new_model()
        optimizer = torch.optim.AdamW(
            model.parameters(), lr=LEARNING_RATE, weight_decay=WEIGHT_DECAY
        )
        train, development = _loaders(tensors)
        iterator = iter(train)
        model.train()
        release._train_batch(model, optimizer, next(iterator))
        began, cpu_began = time.monotonic(), time.process_time()
        losses.append(
            [release._train_batch(model, optimizer, next(iterator)) for _ in range(8)]
        )
        durations.append(time.monotonic() - began)
        cpu_seconds.append(time.process_time() - cpu_began)
        began = time.monotonic()
        _score(model, development)
        scores.append(time.monotonic() - began)
    projected = MAXIMUM_EPOCHS * (
        math.ceil(TRAIN_ROWS / BATCH_SIZE) * max(durations) / 8 + max(scores)
    )
    return {
        "plan": {
            "device": "cpu",
            "threads": threads,
            "interop_threads": 1,
            "workers": 1,
            "batch_size": BATCH_SIZE,
        },
        "training_seconds": durations,
        "training_cpu_seconds": cpu_seconds,
        "cpu_core_equivalents": [
            cpu / wall for cpu, wall in zip(cpu_seconds, durations, strict=True)
        ],
        "development_seconds": scores,
        "timed_batches": 8,
        "repeats": 2,
        "losses": losses,
        "repeat_deterministic": losses[0] == losses[1],
        "stable": max(durations) / min(durations) <= 1.35,
        "projected_primary_seconds": projected * 1.25,
        "safety_factor": 1.25,
        "peak_rss_bytes": release._peak_rss_bytes(),
        "torch_config": torch.__config__.show(),
    }


def _probe_worker(channel: Any, threads: int, root: str) -> None:
    try:
        preparation = contract.validate_preparation(Path(root))
        channel.put(
            {
                "record": _probe_once(
                    threads, contract.load_dataset(Path(root), preparation)
                )
            }
        )
    except Exception as error:
        channel.put({"error": f"{type(error).__name__}: {error}"})


def _spawn_probe(threads: int, root: Path) -> dict[str, Any]:
    context = mp.get_context("spawn")
    channel = context.Queue()
    process = context.Process(target=_probe_worker, args=(channel, threads, str(root)))
    process.start()
    try:
        # Read while the worker is alive so the queue feeder cannot deadlock
        # process.join when an environment fingerprint grows past pipe capacity.
        envelope = channel.get(timeout=300)
        process.join(timeout=10)
        if "error" in envelope:
            raise RuntimeError(envelope["error"])
        return dict(envelope["record"])
    except queue.Empty as error:
        raise RuntimeError("CPU timing worker timed out") from error
    finally:
        if process.is_alive():
            process.terminate()
            process.join(timeout=10)
        channel.close()
        channel.join_thread()


def _select_plan(records: Sequence[Mapping[str, Any]]) -> dict[str, Any] | None:
    if {record["plan"]["threads"] for record in records} != set(THREADS):
        raise ValueError("CPU timing matrix is incomplete")
    eligible = [
        record
        for record in records
        if record["stable"]
        and record["repeat_deterministic"]
        and record["projected_primary_seconds"] <= PRIMARY_BUDGET_SECONDS
        and record["peak_rss_bytes"] <= MEMORY_CEILING_BYTES
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


def preflight_transfer(root: Path) -> dict[str, Any]:
    root = root.resolve()
    preparation = contract.validate_preparation(root)
    if (root / PREFLIGHT_PATH).exists():
        result = release._read_json(root / PREFLIGHT_PATH, cid_field="preflight_cid")
        if result["preparation_cid"] != preparation["preparation_cid"]:
            raise ValueError("preflight preparation changed")
        return result
    release._write_exclusive_json(
        root / "preflight/started.json",
        {"preparation_cid": preparation["preparation_cid"]},
    )
    began = time.monotonic()
    release._configure_cpu(4)
    c0 = control._run_c0(
        source_data.build_source_calibration(), device=torch.device("cpu")
    )
    records = (
        [_spawn_probe(threads, root) for threads in THREADS] if c0["passed"] else []
    )
    selected = _select_plan(records) if records else None
    body = {
        "schema": "uor-r4.zoology-transfer-preflight/1",
        "issue": ISSUE,
        "policy": POLICY,
        "preparation_cid": preparation["preparation_cid"],
        "implementation": preparation["implementation"],
        "c0": c0,
        "plans": records,
        "selected": selected,
        "passed": bool(c0["passed"] and selected),
        "elapsed_seconds": time.monotonic() - began,
        "cpu_only": True,
        "cuda": "FORBIDDEN",
        "mps": "FORBIDDEN",
    }
    contract.validate_preparation(root)
    result = release._with_cid(body, "preflight_cid")
    release._write_exclusive_json(root / PREFLIGHT_PATH, result)
    return result


def _validate_preflight(
    preparation: Mapping[str, Any], preflight: Mapping[str, Any]
) -> None:
    if (
        preflight["preparation_cid"] != preparation["preparation_cid"]
        or preflight["implementation"] != preparation["implementation"]
    ):
        raise ValueError("preflight no longer binds this transfer")
    selected = _select_plan(preflight["plans"]) if preflight["c0"]["passed"] else None
    if preflight["selected"] != selected or preflight["passed"] != bool(
        preflight["c0"]["passed"] and selected
    ):
        raise ValueError("preflight admission differs from measured plans")


def _primary_binding(
    preparation: Mapping[str, Any], preflight: Mapping[str, Any]
) -> tuple[dict[str, Any], str]:
    binding = {
        "preparation_cid": preparation["preparation_cid"],
        "preflight_cid": preflight["preflight_cid"],
        "plan": preflight["selected"]["plan"],
    }
    return binding, cid_bytes(canonical_json_bytes(binding))


def _write_or_match(path: Path, payload: bytes) -> None:
    if path.exists():
        if path.read_bytes() != payload:
            raise ValueError(f"existing {path.name} differs from frozen finalization")
    else:
        release._write_exclusive(path, payload)


def _history_pass(history: Sequence[Mapping[str, Any]]) -> bool:
    for index, row in enumerate(history):
        passed = release._source_pass(float(row["development"]["top1_rate"]))
        if row["epoch"] != index + 1 or row["strict_source_pass"] is not passed:
            raise ValueError("checkpoint epoch/threshold history differs")
        if passed and index != len(history) - 1:
            raise ValueError("checkpoint trained beyond the first passing epoch")
    return bool(history and history[-1]["strict_source_pass"])


def _save_checkpoint(path: Path, value: Mapping[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    temporary = path.with_suffix(".tmp")
    torch.save(dict(value), temporary)
    os.replace(temporary, path)


def _run_primary(
    root: Path, tensors: Mapping[str, Tensor], *, threads: int, binding_cid: str
) -> dict[str, Any]:
    folder = root / "primary"
    if (folder / "result.json").exists():
        result = release._read_json(folder / "result.json", cid_field="primary_cid")
        if result["binding_cid"] != binding_cid:
            raise ValueError("primary binding changed")
        return result
    release._configure_cpu(threads)
    set_zoology_seed(SEED)
    model = _new_model()
    optimizer = torch.optim.AdamW(
        model.parameters(), lr=LEARNING_RATE, weight_decay=WEIGHT_DECAY
    )
    scheduler = torch.optim.lr_scheduler.CosineAnnealingLR(
        optimizer, T_max=MAXIMUM_EPOCHS, eta_min=0.0
    )
    train_loader, development_loader = _loaders(tensors)
    history: list[dict[str, Any]] = []
    carried_elapsed = 0.0
    evaluation_rng = torch.get_rng_state()
    checkpoint_path = folder / "checkpoint.pt"
    if checkpoint_path.exists():
        checkpoint = torch.load(checkpoint_path, map_location="cpu", weights_only=True)
        if checkpoint["binding_cid"] != binding_cid or checkpoint[
            "model_config"
        ] != asdict(model.config):
            raise ValueError("checkpoint belongs to a different transfer")
        model.load_state_dict(checkpoint["model"])
        optimizer.load_state_dict(checkpoint["optimizer"])
        scheduler.load_state_dict(checkpoint["scheduler"])
        history = checkpoint["history"]
        if (
            checkpoint["completed_epochs"] != len(history)
            or len(history) > MAXIMUM_EPOCHS
        ):
            raise ValueError("checkpoint epoch count differs")
        evaluation_rng = checkpoint["evaluation_rng"]
        torch.set_rng_state(checkpoint["torch_rng_state"])
        carried_elapsed = float(checkpoint["elapsed_seconds"])
    passed = _history_pass(history)
    status = "PRIMARY_POSITIVE" if passed else "STOCK_CELL_TRANSFER_MISS"
    began = time.monotonic()
    # The passing-checkpoint repair: never enter the epoch loop after a pass.
    for epoch in range(len(history), MAXIMUM_EPOCHS) if not passed else ():
        if carried_elapsed + time.monotonic() - began >= PRIMARY_BUDGET_SECONDS:
            status = "INCOMPLETE_EPOCH_BOUNDARY_BUDGET"
            break
        epoch_began = time.monotonic()
        learning_rate = float(optimizer.param_groups[0]["lr"])
        train = _train_epoch(model, optimizer, train_loader)
        evaluation_rng = torch.get_rng_state().clone()
        development = _score(model, development_loader)
        passed = release._source_pass(float(development["top1_rate"]))
        if not passed:
            scheduler.step()
        elapsed = carried_elapsed + time.monotonic() - began
        row = {
            "epoch": epoch + 1,
            "learning_rate": learning_rate,
            "train": train,
            "development": development,
            "strict_source_pass": passed,
            "epoch_seconds": time.monotonic() - epoch_began,
            "elapsed_seconds": elapsed,
        }
        history.append(row)
        _save_checkpoint(
            checkpoint_path,
            {
                "binding_cid": binding_cid,
                "model_config": asdict(model.config),
                "model": model.state_dict(),
                "optimizer": optimizer.state_dict(),
                "scheduler": scheduler.state_dict(),
                "torch_rng_state": torch.get_rng_state(),
                "evaluation_rng": evaluation_rng,
                "history": history,
                "completed_epochs": len(history),
                "elapsed_seconds": elapsed,
            },
        )
        release._atomic_json(
            folder / "progress.json",
            {
                "issue": ISSUE,
                "latest": row,
                "remaining_epochs": MAXIMUM_EPOCHS - len(history),
            },
        )
        print(
            f"#1053 epoch={epoch + 1}/64 dev={development['top1_rate']:.6%} nll={development['nll_nats']:.6f} wall={elapsed:.1f}s",
            flush=True,
        )
        if passed:
            status = "PRIMARY_POSITIVE"
            break
    model_payload = release._artifact_payload(model, learning_rate=LEARNING_RATE)
    _write_or_match(folder / "model.safetensors", model_payload)
    rng_payload = release._canonical_safetensors({"evaluation_rng": evaluation_rng})
    _write_or_match(folder / "evaluation-rng.safetensors", rng_payload)
    state = {
        name: tensor
        for name, tensor in model.state_dict().items()
        if name != "lm_head.weight"
    }
    body = {
        "schema": "uor-r4.zoology-transfer-primary/1",
        "issue": ISSUE,
        "policy": POLICY,
        "binding_cid": binding_cid,
        "status": status,
        "passed": status == "PRIMARY_POSITIVE",
        "epochs": len(history),
        "history": history,
        "final_development": history[-1]["development"] if history else None,
        "elapsed_seconds": carried_elapsed + time.monotonic() - began,
        "artifact": {
            "path": "primary/model.safetensors",
            "bytes": len(model_payload),
            "cid": cid_bytes(model_payload),
            "state_cid": release._tensor_mapping_cid(state),
            "config": asdict(model.config),
        },
        "evaluation_rng": {
            "path": "primary/evaluation-rng.safetensors",
            "cid": cid_bytes(rng_payload),
        },
        "work": {
            "train_query_presentations": sum(
                row["train"]["decisions"] for row in history
            ),
            "development_query_presentations": sum(
                row["development"]["decisions"] for row in history
            ),
        },
    }
    result = release._with_cid(body, "primary_cid")
    release._write_exclusive_json(folder / "result.json", result)
    return result


def _load_artifact(root: Path, primary: Mapping[str, Any]) -> ZoologyFigure2Model:
    record = primary["artifact"]
    payload = (root / record["path"]).read_bytes()
    if len(payload) != record["bytes"] or cid_bytes(payload) != record["cid"]:
        raise ValueError("transfer artifact changed")
    state = load_safetensors(payload)
    if release._tensor_mapping_cid(state) != record["state_cid"]:
        raise ValueError("transfer model state changed")
    model = _new_model()
    if asdict(model.config) != record["config"]:
        raise ValueError("transfer model config changed")
    missing, unexpected = model.load_state_dict(state, strict=False)
    if missing != ["lm_head.weight"] or unexpected:
        raise ValueError("artifact missing more than the tied vocabulary head")
    return model


def _control_loader(tensors: Mapping[str, Tensor]) -> DataLoader[Any]:
    return DataLoader(
        TensorDataset(
            tensors["test_inputs"], tensors["test_positions"], tensors["test_targets"]
        ),
        batch_size=BATCH_SIZE,
        shuffle=False,
        num_workers=0,
    )


def run_transfer(root: Path) -> dict[str, Any]:
    root = root.resolve()
    preparation = contract.validate_preparation(root)
    if (root / RESULT_PATH).exists():
        return verify_transfer(root)
    preflight = release._read_json(root / PREFLIGHT_PATH, cid_field="preflight_cid")
    _validate_preflight(preparation, preflight)
    primary = None
    intervention: dict[str, Any] = {"status": "NOT_RUN_PRIMARY_MISS"}
    began = time.monotonic()
    verdict = (
        "INVALID_TRANSFER_MECHANICS"
        if not preflight["c0"]["passed"]
        else "NOT_RUN_PREFLIGHT"
    )
    if preflight["passed"]:
        selected = preflight["selected"]["plan"]
        binding, binding_cid = _primary_binding(preparation, preflight)
        _write_or_match(
            root / "run/started.json",
            canonical_json_bytes({**binding, "binding_cid": binding_cid}),
        )
        primary = _run_primary(
            root,
            contract.load_dataset(root, preparation),
            threads=selected["threads"],
            binding_cid=binding_cid,
        )
        verdict = primary["status"]
        if primary["passed"]:
            # This is the first model-facing read of the destructive control.
            control_tensors = contract.load_control(root, preparation)
            score = _score(
                _load_artifact(root, primary), _control_loader(control_tensors)
            )
            drop = primary["final_development"]["top1_rate"] - score["top1_rate"]
            intervention = release._with_cid(
                {
                    "status": "COMPLETE",
                    "dataset": preparation["control"],
                    "score": score,
                    "drop": drop,
                    "required_drop": REQUIRED_DROP,
                    "passed": drop >= REQUIRED_DROP,
                },
                "control_cid",
            )
            verdict = (
                "STOCK_CELL_PASSES_EXACT_BYTES"
                if intervention["passed"]
                else "NONASSOCIATIVE_SHORTCUT"
            )
    contract.validate_preparation(root)
    body = {
        "schema": "uor-r4.zoology-transfer-result/1",
        "issue": ISSUE,
        "policy": POLICY,
        "preparation_cid": preparation["preparation_cid"],
        "preflight_cid": preflight["preflight_cid"],
        "implementation": preparation["implementation"],
        "dataset": preparation["dataset"],
        "primary": primary,
        "control": intervention,
        "elapsed_seconds": time.monotonic() - began,
        "decision": {
            "verdict": verdict,
            "passed": verdict == "STOCK_CELL_PASSES_EXACT_BYTES",
            "action": "scope coherent-R4 transport/replacement parity"
            if verdict == "STOCK_CELL_PASSES_EXACT_BYTES"
            else "stop; no R4 changes or same-issue tuning",
        },
        "read_ledger": {
            "preparation": preparation["read_ledger"],
            "model_role_reads": 0,
            "model_geometry_reads": 0,
            "future_value_reads": 0,
            "teacher_calls": 0,
            "provider_calls": 0,
            "predecessor_weight_reads": 0,
            "sealed_reads": 0,
            "control_query_decisions": intervention.get("score", {}).get(
                "decisions", 0
            ),
        },
        "nonclaims": [
            "R4/geometric attention",
            "generation",
            "reasoning",
            "modulo-256 softmax",
            "exact runtime lowering",
            "release readiness",
        ],
    }
    result = release._with_cid(body, "result_cid")
    release._write_exclusive_json(root / RESULT_PATH, result)
    return result


def verify_transfer(root: Path) -> dict[str, Any]:
    root = root.resolve()
    preparation = contract.validate_preparation(root)
    preflight = release._read_json(root / PREFLIGHT_PATH, cid_field="preflight_cid")
    _validate_preflight(preparation, preflight)
    result = release._read_json(root / RESULT_PATH, cid_field="result_cid")
    if (
        result["preparation_cid"] != preparation["preparation_cid"]
        or result["preflight_cid"] != preflight["preflight_cid"]
        or result["implementation"] != preparation["implementation"]
        or result["dataset"] != preparation["dataset"]
    ):
        raise ValueError("transfer result bindings changed")
    primary = result["primary"]
    if bool(primary is not None) != bool(preflight["passed"]):
        raise ValueError("primary presence differs from admission")
    if primary is not None:
        _, binding_cid = _primary_binding(preparation, preflight)
        if primary["binding_cid"] != binding_cid:
            raise ValueError("primary causal binding differs")
        if (
            release._read_json(root / "primary/result.json", cid_field="primary_cid")
            != primary
        ):
            raise ValueError("embedded primary differs")
        history = primary["history"]
        passed = _history_pass(history)
        if (
            passed != primary["passed"]
            or primary["epochs"] != len(history)
            or len(history) > MAXIMUM_EPOCHS
        ):
            raise ValueError("primary terminal differs from its history")
        if primary["final_development"] != (
            history[-1]["development"] if history else None
        ):
            raise ValueError("primary final development differs from history")
        if passed:
            expected_status = "PRIMARY_POSITIVE"
        elif len(history) == MAXIMUM_EPOCHS:
            expected_status = "STOCK_CELL_TRANSFER_MISS"
        else:
            expected_status = "INCOMPLETE_EPOCH_BOUNDARY_BUDGET"
        if primary["status"] != expected_status:
            raise ValueError("primary terminal status differs")
        if primary["work"] != {
            "train_query_presentations": len(history) * TRAIN_ROWS * QUERIES,
            "development_query_presentations": len(history)
            * DEVELOPMENT_ROWS
            * QUERIES,
        }:
            raise ValueError("primary work differs")
        release._configure_cpu(preflight["selected"]["plan"]["threads"])
        model = _load_artifact(root, primary)
        rng_payload = (root / primary["evaluation_rng"]["path"]).read_bytes()
        if cid_bytes(rng_payload) != primary["evaluation_rng"]["cid"]:
            raise ValueError("evaluation RNG changed")
        _, loader = _loaders(contract.load_dataset(root, preparation))
        torch.set_rng_state(load_safetensors(rng_payload)["evaluation_rng"])
        if history and _score(model, loader) != primary["final_development"]:
            raise ValueError("fresh artifact inference does not reproduce")
        if passed:
            intervention = result["control"]
            release._verify_self_cid(intervention, "control_cid")
            observed = _score(
                model, _control_loader(contract.load_control(root, preparation))
            )
            drop = primary["final_development"]["top1_rate"] - observed["top1_rate"]
            expected = (
                "STOCK_CELL_PASSES_EXACT_BYTES"
                if drop >= REQUIRED_DROP
                else "NONASSOCIATIVE_SHORTCUT"
            )
            if (
                observed != intervention["score"]
                or drop != intervention["drop"]
                or result["decision"]["verdict"] != expected
                or intervention["passed"] != (drop >= REQUIRED_DROP)
                or intervention["required_drop"] != REQUIRED_DROP
                or intervention["dataset"] != preparation["control"]
            ):
                raise ValueError("control/decision does not reproduce")
        elif (
            result["control"]["status"] != "NOT_RUN_PRIMARY_MISS"
            or result["decision"]["verdict"] != primary["status"]
        ):
            raise ValueError("negative primary crossed the control boundary")
    else:
        expected = (
            "INVALID_TRANSFER_MECHANICS"
            if not preflight["c0"]["passed"]
            else "NOT_RUN_PREFLIGHT"
        )
        if (
            result["decision"]["verdict"] != expected
            or result["control"]["status"] != "NOT_RUN_PRIMARY_MISS"
        ):
            raise ValueError("non-admission decision differs")
    if result["decision"]["passed"] != (
        result["decision"]["verdict"] == "STOCK_CELL_PASSES_EXACT_BYTES"
    ):
        raise ValueError("decision pass flag differs")
    return result
