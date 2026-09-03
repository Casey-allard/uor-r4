"""Full-head structured binding, four-order qualification and causal value replay."""

from __future__ import annotations

import os
from pathlib import Path
from typing import Any

import torch
from torch.nn import functional as F

from ..provenance import canonical_json_bytes, cid_bytes
from ..zoology_cyclic_facts.augmentation import rotate_inputs
from ..zoology_cyclic_facts.campaign import _all_order_worlds, _behavior, _ordered_views
from ..zoology_english_binding.campaign import (
    _binding_metrics,
    _Budget,
    _language_decision,
    _runtime,
)
from ..zoology_english_diagnostic.analysis import analyze
from ..zoology_joint_query.campaign import _comparison, _examples, _world_pair_counts
from ..zoology_r4_inference.campaign import (
    ResourceBudgetExceeded,
    _learned_state_cid,
    _peak_rss_bytes,
    _read_bound,
    _write_exclusive,
)
from . import contract, data
from .model import load_model as _load_model

RESULT_SCHEMA = "uor-r4.zoology-compound-binding-result/1"
REPLAY_SCHEMA = "uor-r4.zoology-compound-binding-replay/1"
ROTATIONS = (0, 1, 2, 3)


def _fit_record(root: Path, preparation: dict) -> dict:
    fitted = _read_bound(root / "fit/fit.json", "fit_cid")
    if (
        fitted["schema"] != "uor-r4.zoology-compound-binding-fit/1"
        or fitted["issue"] != contract.ISSUE
        or fitted["preparation_cid"] != preparation["preparation_cid"]
        or fitted["training"] != contract.TRAINING
        or fitted["artifact"]["config"] != contract.MODEL_CONFIG
        or fitted["artifact"]["model_policy"] != contract.MODEL_POLICY
    ):
        raise ValueError("fit and compound model preparation differ")
    model = fitted["artifact"]
    payload = (root / "fit/model.safetensors").read_bytes()
    if (
        model["path"] != "fit/model.safetensors"
        or model["bytes"] != len(payload)
        or model["cid"] != cid_bytes(payload)
    ):
        raise ValueError("final model artifact changed")
    work = fitted["work"]
    updates = fitted["completed_updates"]
    if (
        work["optimizer_updates"] != updates
        or work["supported_phase_updates"] != min(updates, 2352)
        or work["mixed_phase_updates"] != max(0, updates - 2352)
        or work["train_query_presentations"] != updates * 512
        or work["supported_presentations"] + work["unknown_presentations"]
        != updates * 512
        or work["development_decisions"]
        or work["old_model_reads"]
        or work["model_frame_reads"]
    ):
        raise ValueError("optimizer or presentation ledger differs")
    if fitted["status"] == "FIT_COMPLETE" and (
        updates != 3920 or fitted["blocks"] != 20
    ):
        raise ValueError("complete fit requires the full fixed dose")
    full_mixed, tail = divmod(max(0, updates - 2352), 20)
    unknown_tail = work["unknown_presentations"] - full_mixed * 2048
    if not max(0, tail * 512 - 8192) <= unknown_tail <= min(tail * 512, 2048):
        raise ValueError("unknown presentation tail differs from complete traversals")
    return fitted


def _tensor_cid(tensor: torch.Tensor) -> str:
    return cid_bytes(tensor.detach().cpu().contiguous().numpy().tobytes(order="C"))


def _record(
    logits: torch.Tensor,
    predictions: torch.Tensor,
    attention: torch.Tensor,
    targets: torch.Tensor,
    losses: torch.Tensor,
) -> dict:
    rows = targets.numel()
    if not rows:
        raise ValueError("empty score stratum")
    correct = int((predictions == targets).sum())
    return {
        "decisions": rows,
        "top1_correct": correct,
        "top1_rate": correct / rows,
        "nll_nats": float(losses.double().sum()) / rows,
        "selected_logits_cid": _tensor_cid(logits),
        "predictions_cid": _tensor_cid(predictions),
        "attention_cid": _tensor_cid(attention),
        "attention_shape": list(attention.shape),
        "attention_axis": "four causal fact entries followed by learned null",
        "future_attention_nonzero": 0,
    }


@torch.inference_mode()
def _score(model: Any, tensors: dict, budget: _Budget, *, control: str = "none"):
    outputs, weights = [], []
    for start in range(0, len(tensors["inputs"]), contract.EVALUATION["batch_size"]):
        budget.check()
        stop = start + contract.EVALUATION["batch_size"]
        positions = tensors["positions"][start:stop]
        if not bool((positions == 37).all()):
            raise ValueError("structured query readout position differs")
        # The model sees causal input fields and selected positions, never labels.
        output = model.forward_selected(
            tensors["inputs"][start:stop],
            positions,
            return_attention=True,
            control=control,
        )
        logits = output.logits.detach().float().contiguous()
        attention = output.attention_weights
        if (
            logits.shape != (len(positions), 1, 4096)
            or not bool(torch.isfinite(logits).all())
            or attention is None
            or len(attention) != 1
        ):
            raise ValueError("invalid compound logits or missing attention")
        attn = attention[0].detach().float().contiguous()
        if attn.shape != (len(positions), 1, 1, 5) or not bool(
            torch.isfinite(attn).all()
        ):
            raise ValueError("compound attention must have four fact entries and null")
        # Keys and values use positions <= the location ending each fact;
        # the learned null has no input source. No square token mask is fabricated.
        source_ends = torch.tensor([7, 15, 23, 31, -1])
        future = source_ends[None, :] > positions
        if bool(((attn[:, 0, 0, :] != 0) & future).any()):
            raise ValueError("compound attention used a future fact")
        outputs.append(logits)
        weights.append(attn)
        budget.check()
    logits, attention = torch.cat(outputs), torch.cat(weights)
    predictions = logits.argmax(-1)
    targets = tensors["targets"]
    losses = F.cross_entropy(
        logits.reshape(-1, 4096), targets.reshape(-1), reduction="none"
    )
    supported = tensors["variant_ids"] < 4
    records = {"all": _record(logits, predictions, attention, targets, losses)}
    for name, selected in (("supported", supported), ("unknown", ~supported)):
        records[name] = _record(
            logits[selected],
            predictions[selected],
            attention[selected],
            targets[selected],
            losses[selected],
        )
    budget.check()
    return records, predictions, logits, attention


def _qualification(views: list[dict]) -> dict:
    _ordered_views(views)
    result = []
    for view in views:
        unknown = view["unknown"]
        if unknown["decisions"] != 2048:
            raise ValueError("construction unknown population incomplete")
        worlds = view["question_pair_world_counts"]
        criteria = {
            "supported_accuracy": view["construction"]["top1_correct"]
            >= contract.EVALUATION["construction_known_min_correct"],
            "unknown_accuracy": unknown["top1_correct"]
            >= contract.EVALUATION["construction_unknown_min_correct"],
            **{
                f"{kind}_complete_quartets": worlds[kind]["complete_quartets"]
                >= contract.EVALUATION[
                    "construction_per_question_type_groups_min_correct"
                ]
                for kind in ("owner_changing", "object_changing")
            },
        }
        result.append(
            {
                "rotation": view["rotation"],
                "criteria": criteria,
                "passed": all(criteria.values()),
            }
        )
    return {"passed": all(row["passed"] for row in result), "views": result}


def _order_comparison(
    predictions: torch.Tensor,
    logits: torch.Tensor,
    attention: torch.Tensor,
    baseline: tuple,
    offset: int,
) -> dict:
    old_predictions, old_logits, old_attention = baseline
    aligned = torch.cat(
        (torch.roll(attention[..., :4], shifts=-offset, dims=-1), attention[..., 4:]),
        dim=-1,
    )
    delta = float((logits - old_logits).abs().max())
    equal = torch.equal(predictions, old_predictions)
    return {
        "rotation": offset,
        "decisions": predictions.numel(),
        "top1_exact": equal,
        "changed_predictions": int((predictions != old_predictions).sum()),
        "max_absolute_logit_difference": delta,
        "max_aligned_attention_difference": float(
            (aligned - old_attention).abs().max()
        ),
        "passed": equal and delta <= contract.ORDER["max_absolute_logit_difference"],
    }


def _construction_views(model: Any, tensors: dict, budget: _Budget):
    views, predictions, order = [], [], []
    baseline = None
    supported = tensors["variant_ids"] < 4
    known = {key: value[supported].contiguous() for key, value in tensors.items()}
    for offset in ROTATIONS:
        rotated = {**tensors, "inputs": rotate_inputs(tensors["inputs"], offset)}
        records, predicted, logits, attention = _score(model, rotated, budget)
        if baseline is None:
            baseline = predicted, logits, attention
        order.append(_order_comparison(predicted, logits, attention, baseline, offset))
        diagnostic = analyze(
            rotated["inputs"][supported],
            known["targets"],
            predicted[supported],
            logits[supported],
            known["group_ids"],
            known["variant_ids"],
            known["pair_types"],
        )
        views.append(
            {
                "rotation": offset,
                "construction": records["supported"],
                "unknown": records["unknown"],
                "unknown_prediction_ids": predicted[~supported].reshape(-1).tolist(),
                "all": records["all"],
                "diagnostic": diagnostic,
                "question_pair_world_counts": _world_pair_counts(
                    predicted[supported], known
                ),
                "examples": _examples(
                    {**known, "inputs": rotated["inputs"][supported]},
                    predicted[supported],
                    4,
                ),
                "null_attention": {
                    name: float(attention[selected, 0, 0, 4].double().mean())
                    for name, selected in (
                        ("supported", supported),
                        ("unknown", ~supported),
                    )
                },
            }
        )
        predictions.append(predicted[supported])
        del logits, attention
        budget.check()
    return (
        views,
        _all_order_worlds(predictions, known),
        {"passed": all(row["passed"] for row in order), "views": order},
    )


def _replacement_targets(tensors: dict) -> torch.Tensor:
    """Scorer-only counterfactual labels; never used by the model or its control."""
    targets = tensors["targets"].clone()
    supported = tensors["variant_ids"] < 4
    locations = tensors["inputs"][supported][:, [7, 15, 23, 31]]
    match = locations == targets[supported]
    if not bool((match.sum(-1) == 1).all()):
        raise ValueError(
            "supported target must identify exactly one original fact value"
        )
    index = match.long().argmax(-1)
    # Right roll means the value delivered at key j came from old slot j-1.
    targets[supported, 0] = locations.gather(1, ((index - 1) % 4)[:, None])[:, 0]
    if bool((targets[supported] == tensors["targets"][supported]).any()):
        raise ValueError("value cycle must replace every supported target")
    return targets


def _control_decision(
    view: dict, controlled: dict, replacement_correct: int, attention_exact: bool
) -> dict:
    limits = contract.CONTROL
    loss = (
        view["construction"]["top1_correct"] - controlled["supported"]["top1_correct"]
    )
    criteria = {
        "original_supported_drop": loss >= limits["original_known_correct_drop_min"],
        "replacement_supported_recovery": replacement_correct
        >= limits["replacement_known_correct_min"],
        "unknown_preserved": controlled["unknown"]["top1_correct"]
        >= limits["unknown_min_correct"],
        "attention_exact": attention_exact,
    }
    return {
        "passed": all(criteria.values()),
        "criteria": criteria,
        "original_supported_correct_drop": loss,
        "replacement_supported_correct": replacement_correct,
    }


def _conditional_controls(
    model: Any,
    tensors: dict,
    views: list[dict],
    qualification: dict,
    order: dict,
    budget: _Budget,
) -> dict:
    if not qualification["passed"] or not order["passed"]:
        return {
            "status": "NOT_RUN_CONSTRUCTION_MISS"
            if not qualification["passed"]
            else "NOT_RUN_ORDER_MISS",
            "passed": False,
            "model_decisions": 0,
            "views": [],
        }
    results = []
    supported = tensors["variant_ids"] < 4
    for offset, view in enumerate(views):
        rotated = {**tensors, "inputs": rotate_inputs(tensors["inputs"], offset)}
        records, predicted, logits, _ = _score(
            model, rotated, budget, control="value_cycle"
        )
        # The attention CID is over actual rectangular weights, in the same order.
        attention_exact = (
            records["all"]["attention_cid"] == view["all"]["attention_cid"]
        )
        replacement = _replacement_targets(rotated)
        recovered = int((predicted[supported] == replacement[supported]).sum())
        decision = _control_decision(view, records, recovered, attention_exact)
        results.append(
            {
                "rotation": offset,
                **decision,
                "original_target_scores": records,
                "replacement_targets_cid": _tensor_cid(replacement),
                "replacement_supported_nll_nats": float(
                    F.cross_entropy(
                        logits[supported].reshape(-1, 4096),
                        replacement[supported].reshape(-1),
                        reduction="none",
                    )
                    .double()
                    .mean()
                ),
                "unknown_predictions_cid": _tensor_cid(predicted[~supported]),
                "unknown_predictions_exact": records["unknown"]["predictions_cid"]
                == view["unknown"]["predictions_cid"],
                "unknown_changed_predictions": int(
                    (
                        predicted[~supported].reshape(-1)
                        != torch.tensor(view["unknown_prediction_ids"])
                    ).sum()
                ),
            }
        )
        del logits
        budget.check()
    return {
        "status": "SCORED_FIXED_FINAL_ARTIFACT",
        "passed": all(row["passed"] for row in results),
        "model_decisions": 40960,
        "views": results,
    }


def _conditional_development(
    root: Path,
    model: Any,
    construction_views: list[dict],
    qualification: dict,
    order: dict,
    controls: dict,
    budget: _Budget,
) -> dict:
    stop = (
        "NOT_RUN_CONSTRUCTION_MISS"
        if not qualification["passed"]
        else "NOT_RUN_ORDER_MISS"
        if not order["passed"]
        else "NOT_RUN_CONTROL_MISS"
        if not controls["passed"]
        else None
    )
    if stop is not None:
        return {"status": stop, "model_decisions": 0, "views": []}
    tensors = data.load_development(root / "data")
    views = []
    for offset in ROTATIONS:
        rotated = {**tensors, "inputs": rotate_inputs(tensors["inputs"], offset)}
        records, predictions, logits, _ = _score(model, rotated, budget)
        metrics = _binding_metrics(
            predictions, tensors["targets"], tensors["pair_types"]
        )
        language = _language_decision(
            construction_views[offset]["construction"], metrics
        )
        views.append(
            {
                "rotation": offset,
                "record": records["all"],
                "supported": records["supported"],
                "unknown": records["unknown"],
                "behavior": metrics,
                "decision": language,
                "examples": _examples(rotated, predictions, 10),
            }
        )
        del logits
        budget.check()
    return {
        "status": "SCORED_FIXED_FINAL_ARTIFACT",
        "model_decisions": 5120,
        "views": views,
    }


def _decision(
    behavior: dict, qualification: dict, order: dict, controls: dict, development: dict
) -> dict:
    qualified = qualification["passed"] and order["passed"] and controls["passed"]
    criteria = {
        "construction": qualification["passed"],
        "order": order["passed"],
        "value_binding_control": controls["passed"],
    }
    if not qualified:
        if development["model_decisions"] or development["views"]:
            raise ValueError("unqualified model must leave fresh development unscored")
        if not order["passed"]:
            status, action = (
                "COMPOUND_BINDING_ORDER_MISS",
                "RETAIN_REFERENCE_AND_REPORT_ORDER_INTERFACE_MISS",
            )
        elif not qualification["passed"]:
            if behavior["any_regression"]:
                status, action = (
                    "COMPOUND_BINDING_PRESERVATION_MISS",
                    "RETAIN_1067_REFERENCE_AND_REVISE_COMPOUND_BINDING_RECIPE",
                )
            elif behavior["passed"]:
                status, action = (
                    "COMPOUND_BINDING_PARTIAL_GAIN",
                    "RETAIN_STRUCTURED_CONSTRUCTION_GAIN_AND_ADDRESS_REMAINING_ERRORS",
                )
            else:
                status, action = (
                    "COMPOUND_BINDING_BELOW_DECLARED_GAIN_OR_SLOT_FLOOR",
                    "RETAIN_1067_REFERENCE_AND_REVISE_COMPOUND_BINDING_RECIPE",
                )
        else:
            status, action = (
                "COMPOUND_BINDING_CONTROL_MISS",
                "RETAIN_CONSTRUCTION_ONLY_AND_REPORT_CONTROL_MISS",
            )
        return {
            "status": status,
            "passed": False,
            "criteria": criteria,
            "next_action": action,
        }
    if development["model_decisions"] != 5120 or [
        v["rotation"] for v in development["views"]
    ] != list(ROTATIONS):
        raise ValueError(
            "qualified construction requires complete four-order development"
        )
    passed = all(row["decision"]["passed"] for row in development["views"])
    return {
        "status": "COMPOUND_BINDING_FRESH_PASSED"
        if passed
        else "COMPOUND_BINDING_FRESH_TRANSFER_MISS",
        "passed": passed,
        "criteria": {**criteria, "fresh_binding": passed},
        "next_action": "EVALUATE_UNCHANGED_R4_ADAPTER_SEPARATELY"
        if passed
        else "RETAIN_STRUCTURED_CONSTRUCTION_AND_ADDRESS_FRESH_TRANSFER",
    }


def _evaluate(root: Path, preparation: dict, fitted: dict, budget: _Budget) -> dict:
    budget.check()
    tensors = data.load_construction(root / "data")
    model = _load_model({"source": {"root": str(root), "model": fitted["artifact"]}})
    before = _learned_state_cid(model)
    views, worlds, order = _construction_views(model, tensors, budget)
    reference = preparation["lineage"]["reference_views"]
    behavior = _behavior(views, reference)
    qualification = _qualification(views)
    controls = _conditional_controls(
        model, tensors, views, qualification, order, budget
    )
    development = _conditional_development(
        root, model, views, qualification, order, controls, budget
    )
    decision = _decision(behavior, qualification, order, controls, development)
    after = _learned_state_cid(model)
    if before != after or after != fitted["artifact"]["state_cid"]:
        raise ValueError("evaluation changed learned state")
    budget.check()
    return {
        **decision,
        "construction_views": views,
        "construction_qualification": qualification,
        "partial_behavior_comparison": behavior,
        "order_consistency": order,
        "matched_comparisons": [
            {
                "rotation": c["rotation"],
                **_comparison(
                    c["construction"],
                    c["diagnostic"],
                    {"construction": r["construction"], "diagnostic": r["diagnostic"]},
                ),
            }
            for c, r in zip(views, reference, strict=True)
        ],
        "reference_scope": "retained #1067 scores from frozen #1071 evidence; no new reference model execution",
        "all_order_worlds": worlds,
        "value_binding_control": controls,
        "development": development,
        "learned_state_before": before,
        "learned_state_after": after,
        "parameter_count": sum(p.numel() for p in model.parameters()),
        "model_policy": contract.MODEL_POLICY,
        "evaluation_optimizer_updates": 0,
        "evaluation_checkpoint_optimizer_rng_reads": 0,
        "reference_model_loads": 0,
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
