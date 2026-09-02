"""Final matched construction comparison, conditional fresh transfer and replay."""

from __future__ import annotations

import os
from pathlib import Path
from typing import Any

import torch

from ..provenance import canonical_json_bytes, cid_bytes
from ..zoology_english_binding.campaign import (
    _Budget, _binding_metrics, _language_decision, _plain_score, _runtime,
)
from ..zoology_english_binding.data import decode
from ..zoology_english_diagnostic.analysis import analyze
from ..zoology_english_diagnostic.campaign import _score
from ..zoology_r4_inference.campaign import (
    ResourceBudgetExceeded, _learned_state_cid, _load_model, _peak_rss_bytes,
    _read_bound, _write_exclusive,
)
from . import contract, data

RESULT_SCHEMA = "uor-r4.zoology-query-readout-result/1"
REPLAY_SCHEMA = "uor-r4.zoology-query-readout-replay/1"


def _fit_record(root: Path, preparation: dict) -> dict:
    fitted = _read_bound(root / "fit/fit.json", "fit_cid")
    if (
        fitted["schema"] != "uor-r4.zoology-query-readout-fit/1"
        or fitted["issue"] != contract.ISSUE
        or fitted["preparation_cid"] != preparation["preparation_cid"]
        or fitted["training"] != contract.TRAINING
        or fitted["artifact"]["config"] != contract.MODEL_CONFIG
    ):
        raise ValueError("fit and readout preparation differ")
    model_record = fitted["artifact"]
    payload = (root / "fit/model.safetensors").read_bytes()
    if (
        model_record["path"] != "fit/model.safetensors"
        or model_record["bytes"] != len(payload)
        or model_record["cid"] != cid_bytes(payload)
    ):
        raise ValueError("final model artifact changed")
    if fitted["status"] == "FIT_COMPLETE":
        expected = preparation["lineage"]["baseline"]["fit_work"]
        if (
            fitted["completed_updates"] != contract.TRAINING["total_updates"]
            or fitted["work"] != expected
            or fitted["blocks"] != 20
        ):
            raise ValueError("matched optimizer/sampler dose or work differs")
    return fitted


def _construction_fits(record: dict) -> bool:
    if record["decisions"] != contract.EVALUATION["construction_known_rows"]:
        raise ValueError("construction population incomplete")
    return record["top1_correct"] >= contract.EVALUATION["construction_known_min_correct"]


def _examples(tensors: dict, predictions: torch.Tensor, count: int) -> list[dict]:
    return [
        {
            "row": row,
            "prompt": decode(tensors["inputs"][row], skip_bos=True),
            "supervised_position": int(tensors["positions"][row, 0]),
            "target": data.VOCABULARY[int(tensors["targets"][row, 0])],
            "prediction": data.VOCABULARY[int(predictions[row, 0])],
        }
        for row in range(count)
    ]


def _conditional_development(root: Path, model: Any, construction: dict, budget: _Budget) -> dict:
    if not _construction_fits(construction):
        return {
            "status": "NOT_RUN_CONSTRUCTION_MISS", "model_decisions": 0,
            "record": None, "behavior": None, "examples": [],
        }
    # The only model-facing fresh-development path; never reached on a fit miss.
    tensors = data.load_development(root / "data")
    record, predictions = _plain_score(
        model, *(tensors[name] for name in ("inputs", "positions", "targets")), budget
    )
    return {
        "status": "SCORED_FIXED_FINAL_ARTIFACT",
        "model_decisions": record["decisions"], "record": record,
        "behavior": _binding_metrics(predictions, tensors["targets"], tensors["pair_types"]),
        "examples": _examples(tensors, predictions, 10),
    }


def _decision(construction: dict, development: dict) -> dict:
    if not _construction_fits(construction):
        if development["model_decisions"] != 0 or development["behavior"] is not None:
            raise ValueError("construction miss must leave fresh development unscored")
        return {
            "status": "QUERY_OBJECT_READOUT_CONSTRUCTION_MISS", "passed": False,
            "next_action": "RETAIN_GAINS_AND_REDESIGN_JOINT_QUERY_BINDING",
            "criteria": {"construction_known_fit": False},
            "interpretation": "Placement alone is insufficient at this fixed dose; retain any partial improvement.",
        }
    if development["model_decisions"] != 1280 or development["behavior"] is None:
        raise ValueError("construction fit requires complete final development evaluation")
    language = _language_decision(construction, development["behavior"])
    return {
        "status": "QUERY_OBJECT_READOUT_FRESH_BINDING_PASSED"
        if language["passed"] else "QUERY_OBJECT_READOUT_FRESH_TRANSFER_MISS",
        "passed": language["passed"], "criteria": language["criteria"],
        "next_action": "EVALUATE_UNCHANGED_R4_ADAPTER_SEPARATELY"
        if language["passed"] else "RETAIN_CONSTRUCTION_LEARNING_AND_ADDRESS_FRESH_TRANSFER",
        "interpretation": "Fresh combinations in the familiar held-out-pair task; explicit supervised answer readout.",
    }


def _comparison(construction: dict, diagnostic: dict, baseline: dict) -> dict:
    previous = baseline["construction"]
    old_diagnostic = baseline["diagnostic"]
    result = {
        "baseline_readout_position": 40, "candidate_readout_position": 37,
        "baseline_correct": previous["top1_correct"],
        "candidate_correct": construction["top1_correct"],
        "correct_gain": construction["top1_correct"] - previous["top1_correct"],
        "accuracy_gain_percentage_points": 100 * (construction["top1_rate"] - previous["top1_rate"]),
        "nll_change_nats": construction["nll_nats"] - previous["nll_nats"],
        "pair_comparisons": {},
    }
    for kind in ("question", "location_swap"):
        current, old = diagnostic["paired"][kind], old_diagnostic["paired"][kind]
        comparisons = {}
        for label, value, reference in (
            ("all", current["all"], old["all"]),
            *((name, current["pair_type"][name], old["pair_type"][name])
              for name in ("same_owner", "same_object")),
        ):
            comparisons[label] = {
                "pairs": value["pairs"],
                **{f"{metric}_{which}": score[metric]
                   for metric in ("changed", "invariant", "both_correct")
                   for which, score in (("baseline", reference), ("candidate", value))},
                "both_correct_gain": value["both_correct"] - reference["both_correct"],
            }
        result["pair_comparisons"][kind] = comparisons
    result["by_question_type"] = {
        name: {
            "baseline_correct": old_diagnostic["strata"]["pair_type"][name]["correct"],
            "candidate_correct": diagnostic["strata"]["pair_type"][name]["correct"],
            "rows": diagnostic["strata"]["pair_type"][name]["rows"],
        }
        for name in ("same_owner", "same_object")
    }
    return result


def _evaluate(root: Path, preparation: dict, fitted: dict, budget: _Budget) -> dict:
    budget.check()
    tensors = data.load_construction(root / "data")
    supported = tensors["variant_ids"] < 4
    tensors = {key: value[supported].contiguous() for key, value in tensors.items()}
    model = _load_model({"source": {"root": str(root), "model": fitted["artifact"]}})
    before = _learned_state_cid(model)
    record, predictions, logits = _score(model, tensors, budget)
    diagnostic = analyze(
        tensors["inputs"], tensors["targets"], predictions, logits,
        tensors["group_ids"], tensors["variant_ids"], tensors["pair_types"],
    )
    comparison = _comparison(record, diagnostic, preparation["lineage"]["baseline"])
    development = _conditional_development(root, model, record, budget)
    decision = _decision(record, development)
    after = _learned_state_cid(model)
    if before != after or after != fitted["artifact"]["state_cid"]:
        raise ValueError("evaluation changed learned state")
    if record["future_attention_nonzero"] or (
        development["record"] and development["record"]["future_attention_nonzero"]
    ):
        raise ValueError("evaluation used future attention")
    budget.check()
    return {
        **decision,
        "construction": record, "construction_diagnostic": diagnostic,
        "matched_comparison": comparison, "development": development,
        "construction_examples": _examples(tensors, predictions, 4),
        "learned_state_before": before, "learned_state_after": after,
        "evaluation_optimizer_updates": 0, "evaluation_checkpoint_optimizer_rng_reads": 0,
        "prior_model_checkpoint_reads": 0, "old_development_payload_reads": 0,
        "model_label_arguments": 0, "vocabulary_filtering": False,
        "geometry_changes": 0, "native_frame_payload_reads": 0,
        "r4": "NOT_RUN_SEPARATE_INFERENCE_STEP",
    }


def _phase(root: Path, *, replay: bool) -> dict:
    root = root.resolve()
    preparation = contract.validate_preparation(root)
    fitted = _fit_record(root, preparation)
    expected = _read_bound(root / "result.json", "result_cid") if replay else None
    carried = float(fitted["elapsed_seconds"])
    runtime = _runtime()
    if expected:
        if (
            expected["schema"] != RESULT_SCHEMA or expected["issue"] != contract.ISSUE
            or expected["process_id"] == os.getpid() or expected["runtime"] != runtime
            or expected["preparation_cid"] != preparation["preparation_cid"]
            or expected["fit_cid"] != fitted["fit_cid"]
            or expected["implementation_cid"] != preparation["implementation"]["tree_cid"]
            or expected["artifact"] != fitted["artifact"]
            or expected["evidence_cid"] != cid_bytes(canonical_json_bytes(expected["evidence"]))
            or fitted["status"] != "FIT_COMPLETE"
            or expected["evidence"]["status"] == "INCOMPLETE_RESOURCE"
        ):
            raise ValueError("replay needs a complete bound result and fresh matched process")
        carried += float(expected["elapsed_seconds"])
    budget = _Budget(carried)
    phase, field = ("replay", "replay_cid") if replay else ("run", "result_cid")
    _write_exclusive(root / f"{phase}-started.json", {
        "preparation_cid": preparation["preparation_cid"],
        "fit_cid": fitted["fit_cid"], "process_id": os.getpid(),
        "carried_elapsed_seconds": carried,
    }, "started_cid")
    if fitted["status"] != "FIT_COMPLETE":
        evidence = {
            "status": fitted["status"], "completed_updates": fitted["completed_updates"],
            "next_action": "RETAIN_PARTIAL_ARTIFACT_AND_REPORT_LIMIT",
            "construction": "NOT_RUN_INCOMPLETE_FIT", "development": "NOT_RUN_INCOMPLETE_FIT",
        }
    else:
        try:
            evidence = _evaluate(root, preparation, fitted, budget)
            if contract.validate_preparation(root) != preparation or _fit_record(root, preparation) != fitted:
                raise ValueError("campaign bindings changed during evaluation")
            budget.check()
        except ResourceBudgetExceeded as error:
            if replay:
                raise
            evidence = {"status": "INCOMPLETE_RESOURCE", "reason": str(error)}
    if expected and evidence != expected["evidence"]:
        raise ValueError("fresh-process construction or transfer evidence differs")
    body = {
        "schema": REPLAY_SCHEMA if replay else RESULT_SCHEMA, "issue": contract.ISSUE,
        "preparation_cid": preparation["preparation_cid"], "fit_cid": fitted["fit_cid"],
        "implementation_cid": preparation["implementation"]["tree_cid"],
        "artifact": fitted["artifact"], "runtime": runtime, "process_id": os.getpid(),
        "evidence_cid": cid_bytes(canonical_json_bytes(evidence)),
        "elapsed_seconds": budget.elapsed, "combined_elapsed_seconds": budget.carried + budget.elapsed,
        "peak_rss_bytes": _peak_rss_bytes(),
    }
    if expected:
        body.update(result_cid=expected["result_cid"], exact_replay=True, fresh_process=True, optimizer_updates=0)
    else:
        body["evidence"] = evidence
    return _write_exclusive(root / ("replay.json" if replay else "result.json"), body, field)


def run(root: Path) -> dict:
    return _phase(root, replay=False)


def verify(root: Path) -> dict:
    return _phase(root, replay=True)
