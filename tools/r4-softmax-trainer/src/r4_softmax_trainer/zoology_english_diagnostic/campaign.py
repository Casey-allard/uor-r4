"""Single fixed-weight construction pass, source reproduction and exact replay."""

from __future__ import annotations

import math
import os
import platform
import time
from pathlib import Path
from typing import Any

import torch
from torch.nn import functional as F

from ..provenance import canonical_json_bytes, cid_bytes
from ..zoology_english_binding import data
from ..zoology_r4_inference.campaign import (
    ResourceBudgetExceeded,
    _Scores,
    _learned_state_cid,
    _load_model,
    _peak_rss_bytes,
    _read_bound,
    _write_exclusive,
)
from ..zoology_release.development import _configure_cpu, _tensor_mapping_cid
from . import analysis, contract


class _Budget:
    def __init__(self, carried_seconds: float = 0.0) -> None:
        if not math.isfinite(carried_seconds) or carried_seconds < 0:
            raise ValueError("invalid carried diagnostic time")
        self.carried = carried_seconds
        self.began = time.monotonic()

    @property
    def elapsed(self) -> float:
        return time.monotonic() - self.began

    def check(self) -> None:
        if self.carried + self.elapsed > contract.EVALUATION["max_elapsed_seconds"]:
            raise ResourceBudgetExceeded("combined diagnostic and replay exceeded 300 s")
        if _peak_rss_bytes() > contract.EVALUATION["max_rss_bytes"]:
            raise ResourceBudgetExceeded("diagnostic process exceeded 2 GiB peak RSS")


def _runtime() -> dict[str, Any]:
    _configure_cpu(contract.EVALUATION["threads"])
    return {
        "python": platform.python_version(),
        "torch": str(torch.__version__),
        "device": "cpu",
        "threads": torch.get_num_threads(),
        "interop_threads": torch.get_num_interop_threads(),
        "workers": 1,
        "blas": "accelerate"
        if "BLAS_INFO=accelerate" in torch.__config__.show()
        else "other",
    }


@torch.inference_mode()
def _score(model: Any, tensors: dict[str, torch.Tensor], budget: _Budget):
    scores, predictions, logits = _Scores(), [], []
    loss_sum = 0.0
    inputs, positions, targets = (tensors[k] for k in ("inputs", "positions", "targets"))
    for start in range(0, len(inputs), contract.EVALUATION["batch_size"]):
        budget.check()
        stop = start + contract.EVALUATION["batch_size"]
        # Metadata and labels are supplied only to the subsequent diagnostic.
        output = model.forward_selected(
            inputs[start:stop], positions[start:stop], return_attention=True
        )
        predictions.append(scores.add(output, targets[start:stop], {}))
        losses = F.cross_entropy(
            output.logits.reshape(-1, output.logits.shape[-1]),
            targets[start:stop].reshape(-1),
            reduction="none",
        )
        loss_sum += float(losses.double().sum())
        logits.append(output.logits.detach().float().contiguous())
        del output
        budget.check()
    record = scores.record()
    record["conditional_nll_nats"] = {
        "supported": loss_sum / targets.numel(), "unknown": None
    }
    return record, torch.cat(predictions), torch.cat(logits)


def _reproduction(observed: dict, expected: dict) -> dict:
    keys = sorted(set(observed) | set(expected))
    comparisons = {
        key: key in observed and key in expected and observed[key] == expected[key]
        for key in keys
    }
    return {"exact": observed == expected, "comparisons": comparisons}


def _examples(tensors: dict, predictions: torch.Tensor) -> list[dict]:
    # Freeze the first quartet for a readable example; no error-driven selection.
    return [
        {
            "row": i,
            "variant": int(tensors["variant_ids"][i]),
            "text": data.decode(tensors["inputs"][i].tolist()),
            "target": data.VOCABULARY[int(tensors["targets"][i, 0])],
            "prediction": data.VOCABULARY[int(predictions[i, 0])],
        }
        for i in range(4)
    ]


def _evaluate(root: Path, preparation: dict, budget: _Budget) -> dict:
    budget.check()
    tensors = contract.load_construction(preparation)
    model = _load_model(preparation)
    before = _learned_state_cid(model)
    observed, predictions, logits = _score(model, tensors, budget)
    reproduction = _reproduction(observed, preparation["source"]["expected_construction"])
    # No interpretation is permitted if the retained construction behavior differs.
    diagnostic = None
    if reproduction["exact"]:
        diagnostic = analysis.analyze(
            tensors["inputs"], tensors["targets"], predictions, logits,
            tensors["group_ids"], tensors["variant_ids"], tensors["pair_types"],
        )
    after = _learned_state_cid(model)
    if before != after or after != preparation["source"]["model"]["state_cid"]:
        raise ValueError("retained learned state changed")
    if contract.validate_preparation(root) != preparation:
        raise ValueError("source bindings changed during diagnostic")
    budget.check()
    return {
        "status": "CONSTRUCTION_DIAGNOSTIC_COMPLETE"
        if reproduction["exact"] else "UNAVAILABLE_CONSTRUCTION_REPRODUCTION",
        "reproduction": reproduction,
        "construction": observed,
        "diagnostic": diagnostic,
        "examples": _examples(tensors, predictions) if reproduction["exact"] else [],
        "population_cid": _tensor_mapping_cid(tensors),
        "learned_state_before": before,
        "learned_state_after": after,
        "optimizer_updates": 0,
        "development_model_decisions": 0,
        "development_payload_reads": 0,
        "checkpoint_optimizer_rng_reads": 0,
        "native_frame_payload_reads": 0,
        "model_label_arguments": 0,
        "vocabulary_filtering": False,
        "new_data_rows": 0,
        "geometry_changes": 0,
    }


def run(root: Path) -> dict:
    root = root.resolve()
    budget = _Budget()
    preparation = contract.validate_preparation(root)
    runtime = _runtime()
    budget.check()
    _write_exclusive(root / "run-started.json", {
        "issue": contract.ISSUE, "process_id": os.getpid(),
        "preparation_cid": preparation["preparation_cid"],
    }, "started_cid")
    try:
        evidence = _evaluate(root, preparation, budget)
    except ResourceBudgetExceeded as error:
        evidence = {"status": "UNAVAILABLE_RESOURCE_BUDGET", "reason": str(error)}
    body = {
        "schema": "uor-r4.zoology-english-diagnostic-result/1",
        "issue": contract.ISSUE,
        "preparation_cid": preparation["preparation_cid"],
        "implementation_cid": preparation["implementation"]["tree_cid"],
        "artifact": preparation["source"]["model"],
        "process_id": os.getpid(), "runtime": runtime,
        "evidence": evidence,
        "evidence_cid": cid_bytes(canonical_json_bytes(evidence)),
        "elapsed_seconds": budget.elapsed,
        "peak_rss_bytes": _peak_rss_bytes(),
    }
    return _write_exclusive(root / "result.json", body, "result_cid")


def verify(root: Path) -> dict:
    root = root.resolve()
    original = _read_bound(root / "result.json", "result_cid")
    budget = _Budget(original["elapsed_seconds"])
    preparation = contract.validate_preparation(root)
    runtime = _runtime()
    if (
        original["process_id"] == os.getpid()
        or original["schema"] != "uor-r4.zoology-english-diagnostic-result/1"
        or original["issue"] != contract.ISSUE
        or original["runtime"] != runtime
        or original["preparation_cid"] != preparation["preparation_cid"]
        or original["implementation_cid"] != preparation["implementation"]["tree_cid"]
        or original["artifact"] != preparation["source"]["model"]
        or original["evidence_cid"] != cid_bytes(canonical_json_bytes(original["evidence"]))
        or original["evidence"]["status"] != "CONSTRUCTION_DIAGNOSTIC_COMPLETE"
        or original["peak_rss_bytes"] > contract.EVALUATION["max_rss_bytes"]
    ):
        raise ValueError("replay requires the complete bound result in a fresh process")
    budget.check()
    _write_exclusive(root / "replay-started.json", {
        "issue": contract.ISSUE, "process_id": os.getpid(),
        "result_cid": original["result_cid"],
    }, "started_cid")
    try:
        evidence = _evaluate(root, preparation, budget)
        exact = evidence == original["evidence"]
        reason = None
    except ResourceBudgetExceeded as error:
        evidence, exact, reason = {}, False, str(error)
    body = {
        "schema": "uor-r4.zoology-english-diagnostic-replay/1",
        "issue": contract.ISSUE,
        "preparation_cid": preparation["preparation_cid"],
        "implementation_cid": preparation["implementation"]["tree_cid"],
        "result_cid": original["result_cid"],
        "evidence_cid": cid_bytes(canonical_json_bytes(evidence)),
        "exact_replay": exact, "fresh_process": True,
        "reason": reason,
        "artifact": preparation["source"]["model"],
        "process_id": os.getpid(), "runtime": runtime,
        "elapsed_seconds": budget.elapsed,
        "combined_elapsed_seconds": budget.carried + budget.elapsed,
        "peak_rss_bytes": _peak_rss_bytes(),
        "optimizer_updates": 0, "development_model_decisions": 0,
    }
    return _write_exclusive(root / "replay.json", body, "replay_cid")
