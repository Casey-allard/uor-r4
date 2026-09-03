"""Matched four-order construction comparison, gated transfer and exact replay."""

from __future__ import annotations

import os
from pathlib import Path
from typing import Any

import torch

from ..provenance import canonical_json_bytes, cid_bytes
from ..zoology_english_binding.campaign import (
    _binding_metrics,
    _Budget,
    _language_decision,
    _plain_score,
    _runtime,
)
from ..zoology_english_diagnostic.analysis import analyze
from ..zoology_english_diagnostic.campaign import _score
from ..zoology_joint_query.campaign import _comparison, _examples, _world_pair_counts
from ..zoology_r4_inference.campaign import (
    ResourceBudgetExceeded,
    _learned_state_cid,
    _load_model,
    _peak_rss_bytes,
    _read_bound,
    _write_exclusive,
)
from . import contract, data
from .augmentation import rotate_inputs, rotation_ledger

RESULT_SCHEMA = "uor-r4.zoology-cyclic-facts-result/1"
REPLAY_SCHEMA = "uor-r4.zoology-cyclic-facts-replay/1"
ROTATIONS = (0, 1, 2, 3)


def _fit_record(root: Path, preparation: dict) -> dict:
    fitted = _read_bound(root / "fit/fit.json", "fit_cid")
    if (
        fitted["schema"] != "uor-r4.zoology-cyclic-facts-fit/1"
        or fitted["issue"] != contract.ISSUE
        or fitted["preparation_cid"] != preparation["preparation_cid"]
        or fitted["training"] != contract.TRAINING
        or fitted["artifact"]["config"] != contract.MODEL_CONFIG
        or "query_encoding" in fitted["artifact"]
    ):
        raise ValueError("fit and plain query-readout preparation differ")
    model_record = fitted["artifact"]
    payload = (root / "fit/model.safetensors").read_bytes()
    if (
        model_record["path"] != "fit/model.safetensors"
        or model_record["bytes"] != len(payload)
        or model_record["cid"] != cid_bytes(payload)
    ):
        raise ValueError("final model artifact changed")
    if fitted["status"] == "FIT_COMPLETE" and (
        fitted["completed_updates"] != contract.TRAINING["total_updates"]
        or fitted["work"] != preparation["lineage"]["baseline"]["fit_work"]
        or fitted["blocks"] != 20
        or fitted["augmentation"]
        != rotation_ledger(
            fitted["completed_updates"], fitted["work"]["unknown_presentations"]
        )
    ):
        raise ValueError("matched optimizer/sampler dose or rotation ledger differs")
    return fitted


def _ordered_views(views: list[dict]) -> None:
    if [row["rotation"] for row in views] != list(ROTATIONS):
        raise ValueError("exactly four ordered cyclic views required")
    if any(row["construction"]["decisions"] != 8192 for row in views):
        raise ValueError("construction population incomplete")


def _construction_fits(views: list[dict]) -> bool:
    _ordered_views(views)
    return all(row["construction"]["top1_correct"] >= 8111 for row in views)


def _behavior(candidate: list[dict], reference: list[dict]) -> dict:
    _ordered_views(candidate)
    _ordered_views(reference)
    limits = contract.BEHAVIOR
    views = []
    for current, prior in zip(candidate, reference, strict=True):
        pairs, families = {}, {}
        for name in ("same_object", "same_owner"):
            observed = current["diagnostic"]["paired"]["question"]["pair_type"][name]
            baseline = prior["diagnostic"]["paired"]["question"]["pair_type"][name]
            if observed["pairs"] != 2048 or baseline["pairs"] != 2048:
                raise ValueError("question-pair population differs")
            required = min(
                baseline["both_correct"] + limits["matched_both_correct_gain_min"], 2048
            )
            pairs[name] = {
                "pairs": 2048,
                "reference_both_correct": baseline["both_correct"],
                "candidate_both_correct": observed["both_correct"],
                "required_both_correct": required,
                "ceiling_limited": baseline["both_correct"]
                + limits["matched_both_correct_gain_min"]
                > 2048,
                "gain": observed["both_correct"] - baseline["both_correct"],
                "gain_percentage_points": 100
                * (observed["both_correct"] - baseline["both_correct"])
                / 2048,
                "passed": observed["both_correct"] >= required,
            }
            c = current["diagnostic"]["strata"]["pair_type"][name]
            r = prior["diagnostic"]["strata"]["pair_type"][name]
            if c["rows"] != 4096 or r["rows"] != 4096:
                raise ValueError("question-family population differs")
            families[name] = {
                "rows": 4096,
                "candidate_correct": c["correct"],
                "reference_correct": r["correct"],
                "gain": c["correct"] - r["correct"],
            }
        slots = current["diagnostic"]["strata"]["target_displayed_slot"]
        if set(slots) != {"0", "1", "2", "3"} or any(
            value["rows"] != 2048 for value in slots.values()
        ):
            raise ValueError("target-slot balance differs")
        overall_gain = (
            current["construction"]["top1_correct"]
            - prior["construction"]["top1_correct"]
        )
        regression = overall_gain < 0 or any(
            row["gain"] < 0 for row in [*pairs.values(), *families.values()]
        )
        criteria = {
            "both_question_type_gains": all(row["passed"] for row in pairs.values()),
            "every_target_slot": all(
                row["correct"] >= limits["target_slot_correct_min"]
                for row in slots.values()
            ),
            "overall_preserved": overall_gain >= 0,
            "question_families_preserved": not regression,
        }
        views.append(
            {
                "rotation": current["rotation"],
                "pairs": pairs,
                "families": families,
                "slots_correct": [slots[str(i)]["correct"] for i in ROTATIONS],
                "worst_slot_correct": min(row["correct"] for row in slots.values()),
                "overall_correct_gain": overall_gain,
                "regression": regression,
                "criteria": criteria,
                "passed": all(criteria.values()),
            }
        )
    return {
        "passed": all(row["passed"] for row in views),
        "any_regression": any(row["regression"] for row in views),
        "views": views,
    }


def _conditional_development(
    root: Path, model: Any, views: list[dict], behavior: dict, budget: _Budget
) -> dict:
    fits = _construction_fits(views)
    if not fits or not behavior["passed"]:
        return {
            "status": "NOT_RUN_CONSTRUCTION_MISS"
            if not fits
            else "NOT_RUN_BEHAVIOR_MISS",
            "model_decisions": 0,
            "views": [],
        }
    tensors = data.load_development(root / "data")
    results = []
    for offset in ROTATIONS:
        rotated = {**tensors, "inputs": rotate_inputs(tensors["inputs"], offset)}
        record, predictions = _plain_score(
            model,
            *(rotated[name] for name in ("inputs", "positions", "targets")),
            budget,
        )
        metrics = _binding_metrics(
            predictions, tensors["targets"], tensors["pair_types"]
        )
        if record["future_attention_nonzero"]:
            raise ValueError("development used future attention")
        results.append(
            {
                "rotation": offset,
                "record": record,
                "behavior": metrics,
                "decision": _language_decision(views[offset]["construction"], metrics),
                "examples": _examples(rotated, predictions, 10),
            }
        )
    return {
        "status": "SCORED_FIXED_FINAL_ARTIFACT",
        "model_decisions": sum(row["record"]["decisions"] for row in results),
        "views": results,
    }


def _decision(views: list[dict], behavior: dict, development: dict) -> dict:
    fits = _construction_fits(views)
    criteria = {
        "all_rotations_behavior": behavior["passed"],
        "all_rotations_construction_fit": fits,
    }
    if not fits or not behavior["passed"]:
        if development["model_decisions"] != 0 or development["views"]:
            raise ValueError(
                "unpassed behavior or construction must leave development unscored"
            )
        if behavior["any_regression"]:
            status = "CYCLIC_FACTS_PRESERVATION_MISS"
        elif not behavior["passed"]:
            status = "CYCLIC_FACTS_BELOW_DECLARED_GAIN_OR_SLOT_FLOOR"
        else:
            status = "CYCLIC_FACTS_PARTIAL_GAIN"
        return {
            "status": status,
            "passed": False,
            "criteria": criteria,
            "next_action": "RETAIN_AUGMENTED_RECIPE_AND_ADDRESS_REMAINING_BINDING_ERRORS"
            if behavior["passed"]
            else "RETAIN_1067_REFERENCE_AND_REVISE_BINDING_LEARNING_RECIPE",
        }
    if development["model_decisions"] != 5120 or [
        row["rotation"] for row in development["views"]
    ] != list(ROTATIONS):
        raise ValueError(
            "passing construction requires all four complete development views"
        )
    passed = all(row["decision"]["passed"] for row in development["views"])
    return {
        "status": "CYCLIC_FACTS_FRESH_BINDING_PASSED"
        if passed
        else "CYCLIC_FACTS_FRESH_TRANSFER_MISS",
        "passed": passed,
        "criteria": {**criteria, "all_rotations_fresh_binding": passed},
        "next_action": "EVALUATE_UNCHANGED_R4_ADAPTER_SEPARATELY"
        if passed
        else "RETAIN_CONSTRUCTION_LEARNING_AND_ADDRESS_FRESH_TRANSFER",
    }


def _all_order_worlds(predictions: list[torch.Tensor], tensors: dict) -> dict:
    if len(predictions) != 4:
        raise ValueError("four prediction views required")
    for prediction in predictions:
        _world_pair_counts(prediction, tensors)
    correct = torch.stack([p == tensors["targets"] for p in predictions]).reshape(
        4, 2048, 4
    )
    complete = correct.all(dim=2).sum(dim=0)
    types = tensors["pair_types"].reshape(2048, 4)[:, 0]
    return {
        "worlds": 2048,
        "rotations_per_world": 4,
        "answers_per_rotation": 4,
        "complete_in_all_rotations": int((complete == 4).sum()),
        "worlds_by_number_of_complete_rotations": torch.bincount(
            complete, minlength=5
        ).tolist(),
        "by_question_type": {
            name: {
                "worlds": 1024,
                "complete_in_all_rotations": int((complete[types == kind] == 4).sum()),
                "worlds_by_number_of_complete_rotations": torch.bincount(
                    complete[types == kind], minlength=5
                ).tolist(),
            }
            for name, kind in (("same_owner", 0), ("same_object", 1))
        },
        "scope": "Four correlated cyclic views and two related question pairs per world; not independent trials and not all 24 fact orders.",
    }


def _score_views(
    model: Any, tensors: dict, budget: _Budget, baseline: dict | None = None
) -> tuple[list[dict], dict]:
    views, predictions = [], []
    for offset in ROTATIONS:
        rotated = {**tensors, "inputs": rotate_inputs(tensors["inputs"], offset)}
        record, predicted, logits = _score(model, rotated, budget)
        diagnostic = analyze(
            rotated["inputs"],
            tensors["targets"],
            predicted,
            logits,
            tensors["group_ids"],
            tensors["variant_ids"],
            tensors["pair_types"],
        )
        del logits
        if (
            offset == 0
            and baseline is not None
            and (
                record != baseline["construction"]
                or diagnostic != baseline["diagnostic"]
            )
        ):
            raise ValueError(
                "retained canonical reference failed exact full-score/diagnostic reproduction"
            )
        if record["future_attention_nonzero"]:
            raise ValueError("construction used future attention")
        views.append(
            {
                "rotation": offset,
                "construction": record,
                "diagnostic": diagnostic,
                "question_pair_world_counts": _world_pair_counts(predicted, tensors),
                "examples": _examples(rotated, predicted, 4),
            }
        )
        predictions.append(predicted)
        budget.check()
    return views, _all_order_worlds(predictions, tensors)


def _evaluate(root: Path, preparation: dict, fitted: dict, budget: _Budget) -> dict:
    budget.check()
    tensors = data.load_construction(root / "data")
    supported = tensors["variant_ids"] < 4
    tensors = {key: value[supported].contiguous() for key, value in tensors.items()}
    reference_model = _load_model({"source": preparation["reference"]})
    reference_before = _learned_state_cid(reference_model)
    reference_views, reference_worlds = _score_views(
        reference_model, tensors, budget, preparation["lineage"]["baseline"]
    )
    reference_after = _learned_state_cid(reference_model)
    if (
        reference_before != reference_after
        or reference_after != preparation["reference"]["model"]["state_cid"]
    ):
        raise ValueError("reference evaluation changed learned state")
    del reference_model
    model = _load_model({"source": {"root": str(root), "model": fitted["artifact"]}})
    before = _learned_state_cid(model)
    views, worlds = _score_views(model, tensors, budget)
    behavior = _behavior(views, reference_views)
    development = _conditional_development(root, model, views, behavior, budget)
    decision = _decision(views, behavior, development)
    after = _learned_state_cid(model)
    if before != after or after != fitted["artifact"]["state_cid"]:
        raise ValueError("candidate evaluation changed learned state")
    budget.check()
    return {
        **decision,
        "behavior": behavior,
        "construction_views": views,
        "reference_views": reference_views,
        "matched_comparisons": [
            {
                "rotation": c["rotation"],
                **_comparison(
                    c["construction"],
                    c["diagnostic"],
                    {"construction": r["construction"], "diagnostic": r["diagnostic"]},
                ),
            }
            for c, r in zip(views, reference_views, strict=True)
        ],
        "reference_canonical_exact_reproduction": True,
        "all_order_worlds": {"candidate": worlds, "reference": reference_worlds},
        "development": development,
        "reference_state_before": reference_before,
        "reference_state_after": reference_after,
        "learned_state_before": before,
        "learned_state_after": after,
        "evaluation_optimizer_updates": 0,
        "evaluation_checkpoint_optimizer_rng_reads": 0,
        "reference_model_loads": 1,
        "candidate_model_loads": 1,
        "training_prior_model_loads": 0,
        "old_development_payload_reads": 0,
        "model_label_arguments": 0,
        "vocabulary_filtering": False,
        "geometry_changes": 0,
        "native_frame_payload_reads": 0,
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
            expected["schema"] != RESULT_SCHEMA
            or expected["issue"] != contract.ISSUE
            or expected["process_id"] == os.getpid()
            or expected["runtime"] != runtime
            or expected["preparation_cid"] != preparation["preparation_cid"]
            or expected["fit_cid"] != fitted["fit_cid"]
            or expected["implementation_cid"]
            != preparation["implementation"]["tree_cid"]
            or expected["artifact"] != fitted["artifact"]
            or expected["evidence_cid"]
            != cid_bytes(canonical_json_bytes(expected["evidence"]))
            or fitted["status"] != "FIT_COMPLETE"
            or expected["evidence"]["status"] == "INCOMPLETE_RESOURCE"
        ):
            raise ValueError(
                "replay needs a complete bound result and fresh matched process"
            )
        carried += float(expected["elapsed_seconds"])
    budget = _Budget(carried)
    phase, field = ("replay", "replay_cid") if replay else ("run", "result_cid")
    _write_exclusive(
        root / f"{phase}-started.json",
        {
            "preparation_cid": preparation["preparation_cid"],
            "fit_cid": fitted["fit_cid"],
            "process_id": os.getpid(),
            "carried_elapsed_seconds": carried,
        },
        "started_cid",
    )
    if fitted["status"] != "FIT_COMPLETE":
        evidence = {
            "status": fitted["status"],
            "completed_updates": fitted["completed_updates"],
            "next_action": "RETAIN_PARTIAL_ARTIFACT_AND_REPORT_LIMIT",
            "construction": "NOT_RUN_INCOMPLETE_FIT",
            "development": "NOT_RUN_INCOMPLETE_FIT",
        }
    else:
        try:
            evidence = _evaluate(root, preparation, fitted, budget)
            if (
                contract.validate_preparation(root) != preparation
                or _fit_record(root, preparation) != fitted
            ):
                raise ValueError("campaign bindings changed during evaluation")
            budget.check()
        except ResourceBudgetExceeded as error:
            if replay:
                raise
            evidence = {"status": "INCOMPLETE_RESOURCE", "reason": str(error)}
    if expected and evidence != expected["evidence"]:
        raise ValueError("fresh-process construction or transfer evidence differs")
    body = {
        "schema": REPLAY_SCHEMA if replay else RESULT_SCHEMA,
        "issue": contract.ISSUE,
        "preparation_cid": preparation["preparation_cid"],
        "fit_cid": fitted["fit_cid"],
        "implementation_cid": preparation["implementation"]["tree_cid"],
        "artifact": fitted["artifact"],
        "runtime": runtime,
        "process_id": os.getpid(),
        "evidence_cid": cid_bytes(canonical_json_bytes(evidence)),
        "elapsed_seconds": budget.elapsed,
        "combined_elapsed_seconds": budget.carried + budget.elapsed,
        "peak_rss_bytes": _peak_rss_bytes(),
    }
    if expected:
        body.update(
            result_cid=expected["result_cid"],
            exact_replay=True,
            fresh_process=True,
            optimizer_updates=0,
        )
    else:
        body["evidence"] = evidence
    return _write_exclusive(
        root / ("replay.json" if replay else "result.json"), body, field
    )


def run(root: Path) -> dict:
    return _phase(root, replay=False)


def verify(root: Path) -> dict:
    return _phase(root, replay=True)
