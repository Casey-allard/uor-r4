"""Final-artifact English behavior, conditional R4 preservation, exact replay."""

from __future__ import annotations

import json
import math
import os
import platform
import time
from collections.abc import Mapping
from pathlib import Path
from typing import Any

import torch
from torch import Tensor
from torch.nn import functional as F

from ..provenance import atomic_write_json, canonical_json_bytes, cid_bytes
from ..zoology_r4_inference.attention import R4ZoologyInference
from ..zoology_r4_inference.campaign import (
    _WORK_FIELDS,
    ResourceBudgetExceeded,
    _learned_state_cid,
    _load_model,
    _maximum_difference,
    _peak_rss_bytes,
    _read_bound,
    _Scores,
    _write_exclusive,
)
from ..zoology_r4_inference.frames import load_frames
from ..zoology_release.development import _configure_cpu
from . import contract, data

EVALUATION = contract.EVALUATION
RESULT_SCHEMA = "uor-r4.zoology-english-binding-result/1"
REPLAY_SCHEMA = "uor-r4.zoology-english-binding-replay/1"


class _Budget:
    def __init__(self, carried_seconds: float) -> None:
        if not math.isfinite(carried_seconds) or carried_seconds < 0:
            raise ValueError("invalid elapsed campaign time")
        self.carried = carried_seconds
        self.began = time.monotonic()

    @property
    def elapsed(self) -> float:
        return time.monotonic() - self.began

    def check(self) -> None:
        if self.carried + self.elapsed > contract.TRAINING["max_elapsed_seconds"]:
            raise ResourceBudgetExceeded("fit, evaluation and replay exceeded 1800 s")
        if _peak_rss_bytes() > contract.TRAINING["max_rss_bytes"]:
            raise ResourceBudgetExceeded("campaign process exceeded 4 GiB peak RSS")


def _runtime() -> dict[str, Any]:
    _configure_cpu(EVALUATION["threads"])
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


def _fit_record(root: Path, preparation: Mapping[str, Any]) -> dict[str, Any]:
    fitted = _read_bound(root / "fit/fit.json", "fit_cid")
    if (
        fitted["preparation_cid"] != preparation["preparation_cid"]
        or fitted["training"] != contract.TRAINING
        or fitted["artifact"]["config"] != contract.MODEL_CONFIG
    ):
        raise ValueError("fit and preparation differ")
    artifact = contract.prior._record(root, "fit/model.safetensors")
    if any(fitted["artifact"].get(key) != value for key, value in artifact.items()):
        raise ValueError("fixed final model bytes or path changed")
    if fitted["status"] == "FIT_COMPLETE" and (
        fitted["completed_updates"] != contract.TRAINING["total_updates"]
    ):
        raise ValueError("complete fit has the wrong optimizer dose")
    return fitted


def _binding_metrics(
    predictions: Tensor, targets: Tensor, pair_types: Tensor
) -> dict[str, Any]:
    groups = EVALUATION["development_groups"]
    if predictions.numel() != groups * 5 or targets.shape != predictions.shape:
        raise ValueError("development predictions must cover all matched groups")
    observed, expected = predictions.reshape(groups, 5), targets.reshape(groups, 5)
    correct = observed == expected
    types = pair_types.reshape(groups, 5)
    if not bool((types == types[:, :1]).all()) or not bool(
        ((types == 0) | (types == 1)).all()
    ):
        raise ValueError("invalid group question-pair types")
    complete = correct[:, :4].all(dim=1)
    by_type = {}
    for index, name in enumerate(("same_owner", "same_object")):
        selected = types[:, 0] == index
        if int(selected.sum()) != groups // 2:
            raise ValueError("unbalanced question-pair groups")
        by_type[name] = {
            "groups": int(selected.sum()),
            "complete_correct": int(complete[selected].sum()),
        }
    patterns: dict[str, int] = {}
    for row in correct[:, :4].tolist():
        pattern = "".join("1" if value else "0" for value in row)
        patterns[pattern] = patterns.get(pattern, 0) + 1
    return {
        "known_decisions": groups * 4,
        "known_correct": int(correct[:, :4].sum()),
        "known_rate": float(correct[:, :4].double().mean()),
        "unknown_decisions": groups,
        "unknown_correct": int(correct[:, 4].sum()),
        "unknown_rate": float(correct[:, 4].double().mean()),
        "groups": groups,
        "complete_groups_correct": int(complete.sum()),
        "complete_group_rate": float(complete.double().mean()),
        "by_question_type": by_type,
        "quartet_correctness_patterns": patterns,
        "same_history_question_changes": int(
            (observed[:, 0] != observed[:, 1]).sum()
            + (observed[:, 2] != observed[:, 3]).sum()
        ),
        "same_question_history_changes": int(
            (observed[:, 0] != observed[:, 2]).sum()
            + (observed[:, 1] != observed[:, 3]).sum()
        ),
        "history_and_question_pair_decisions": groups * 2,
        "swapped_history_retains_old_answer": int(
            (observed[:, 2] == expected[:, 0]).sum()
            + (observed[:, 3] == expected[:, 1]).sum()
        ),
        "missing_history_retains_base_answer": int(
            (observed[:, 4] == expected[:, 0]).sum()
        ),
        "prediction_ids": observed.tolist(),
        "target_ids": expected.tolist(),
    }


def _language_decision(
    construction: Mapping[str, Any], development: Mapping[str, Any]
) -> dict[str, Any]:
    criteria = {
        "construction_known_complete": construction["decisions"]
        == EVALUATION["construction_known_rows"],
        "construction_known_fit": construction["top1_correct"]
        >= EVALUATION["construction_known_min_correct"],
        "development_known_complete": development["known_decisions"]
        == EVALUATION["development_known_rows"],
        "development_known_accuracy": development["known_correct"]
        >= EVALUATION["development_known_min_correct"],
        "complete_counterfactual_groups": development["complete_groups_correct"]
        >= EVALUATION["complete_groups_min_correct"],
        "same_owner_question_dependence": development["by_question_type"]["same_owner"][
            "complete_correct"
        ]
        >= EVALUATION["per_question_type_groups_min_correct"],
        "same_object_question_dependence": development["by_question_type"][
            "same_object"
        ]["complete_correct"]
        >= EVALUATION["per_question_type_groups_min_correct"],
        "development_unknown_complete": development["unknown_decisions"]
        == EVALUATION["development_unknown_rows"],
        "development_unknown_accuracy": development["unknown_correct"]
        >= EVALUATION["development_unknown_min_correct"],
    }
    if not criteria["construction_known_fit"]:
        status = "ENGLISH_BINDING_CONSTRUCTION_MISS"
        next_action = "DIAGNOSE_LEXICAL_RECIPE_LEARNING_WITH_RETAINED_ARTIFACT"
    elif not all(
        criteria[name]
        for name in (
            "development_known_accuracy",
            "complete_counterfactual_groups",
            "same_owner_question_dependence",
            "same_object_question_dependence",
        )
    ):
        status = "ENGLISH_BINDING_COMPOSITIONAL_TRANSFER_MISS"
        next_action = "ISOLATE_COMPOSITIONAL_TRANSFER_WITH_RETAINED_ARTIFACT"
    elif not criteria["development_unknown_accuracy"]:
        status = "ENGLISH_BINDING_MISSING_BINDING_MISS"
        next_action = "ISOLATE_MISSING_BINDING_BEHAVIOR_WITH_RETAINED_ARTIFACT"
    elif all(criteria.values()):
        status = "ENGLISH_BINDING_PASSED"
        next_action = "EVALUATE_UNCHANGED_R4_ADAPTER"
    else:
        raise ValueError("incomplete language population")
    return {
        "criteria": criteria,
        "passed": all(criteria.values()),
        "status": status,
        "next_action": next_action,
    }


@torch.inference_mode()
def _plain_score(
    model: Any, inputs: Tensor, positions: Tensor, targets: Tensor, budget: _Budget
) -> tuple[dict[str, Any], Tensor]:
    scores, predictions = _Scores(), []
    conditional_loss = {"supported": 0.0, "unknown": 0.0}
    conditional_count = {"supported": 0, "unknown": 0}
    for start in range(0, inputs.shape[0], EVALUATION["batch_size"]):
        budget.check()
        stop = start + EVALUATION["batch_size"]
        output = model.forward_selected(
            inputs[start:stop], positions[start:stop], return_attention=True
        )
        predictions.append(scores.add(output, targets[start:stop], {}))
        losses = F.cross_entropy(
            output.logits.reshape(-1, output.logits.shape[-1]),
            targets[start:stop].reshape(-1),
            reduction="none",
        )
        absent = targets[start:stop].reshape(-1) == data.UNKNOWN_ID
        for name, mask in (("supported", ~absent), ("unknown", absent)):
            conditional_loss[name] += float(losses[mask].double().sum())
            conditional_count[name] += int(mask.sum())
        del output
        budget.check()
    record = scores.record()
    record["conditional_nll_nats"] = {
        name: conditional_loss[name] / conditional_count[name]
        if conditional_count[name]
        else None
        for name in conditional_loss
    }
    return record, torch.cat(predictions)


def _control_decision(
    r4: Mapping[str, Any], control: Mapping[str, Any], known_drop: float
) -> dict[str, Any]:
    work_fields = [
        name
        for name in _WORK_FIELDS
        if name
        not in (
            "source_frame_positions_changed",
            "source_frame_matrices_changed",
        )
    ]
    expected_pairs = (
        (EVALUATION["development_known_rows"] + EVALUATION["development_unknown_rows"])
        * data.SEQUENCE_LENGTH
        * (data.SEQUENCE_LENGTH + 1)
        // 2
        * contract.MODEL_CONFIG["n_layers"]
    )
    criteria = {
        "equal_support_and_work": all(
            r4["audit_totals"][key] is not None
            and r4["audit_totals"][key] == control["audit_totals"][key]
            for key in work_fields
        ),
        "complete_causal_support": r4["audit_totals"]["admitted_attention_pairs"]
        == control["audit_totals"]["admitted_attention_pairs"]
        == expected_pairs,
        "zero_future_contributions": r4["future_attention_nonzero"]
        == control["future_attention_nonzero"]
        == r4["audit_totals"]["future_position_reads"]
        == control["audit_totals"]["future_position_reads"]
        == 0,
        "transport_actually_changed": control["audit_totals"][
            "source_frame_matrices_changed"
        ]
        > 0,
    }
    return {
        "integrity_criteria": criteria,
        "integrity_passed": all(criteria.values()),
        "known_accuracy_drop": known_drop,
        "strong_transport_sensitivity": all(criteria.values())
        and known_drop >= EVALUATION["strong_control_drop"],
    }


def _r4_evaluation(
    model: Any,
    preparation: Mapping[str, Any],
    tensors: Mapping[str, Tensor],
    plain_record: Mapping[str, Any],
    plain_predictions: Tensor,
    budget: _Budget,
) -> dict[str, Any]:
    frames = load_frames(Path(preparation["frames"]["root"]))
    wrapper = R4ZoologyInference(model, frames)
    scores = {name: _Scores() for name in ("plain", "r4")}
    collected = {name: [] for name in scores}
    max_logit, max_attention = 0.0, 0.0
    before = _learned_state_cid(model)
    rows = tensors["inputs"].shape[0]
    with torch.inference_mode():
        for start in range(0, rows, EVALUATION["batch_size"]):
            stop = start + EVALUATION["batch_size"]
            inputs, positions, targets = (
                tensors[key][start:stop] for key in ("inputs", "positions", "targets")
            )
            budget.check()
            plain = wrapper.forward_selected(inputs, positions, execution="plain")
            collected["plain"].append(
                scores["plain"].add(plain, targets, wrapper.last_audit)
            )
            budget.check()
            r4 = wrapper.forward_selected(inputs, positions, execution="r4")
            collected["r4"].append(scores["r4"].add(r4, targets, wrapper.last_audit))
            logit_delta, attention_delta = _maximum_difference(plain, r4)
            max_logit, max_attention = (
                max(max_logit, logit_delta),
                max(max_attention, attention_delta),
            )
            del plain, r4
            budget.check()
        records = {name: score.record() for name, score in scores.items()}
        predictions = {name: torch.cat(values) for name, values in collected.items()}
        differences = {
            "selected_logits_max_abs": max_logit,
            "attention_max_abs": max_attention,
            "top1_changed": int((predictions["plain"] != predictions["r4"]).sum()),
            "nll_abs_difference": abs(
                records["plain"]["nll_nats"] - records["r4"]["nll_nats"]
            ),
        }
        criteria = {
            "original_plain_logits_preserved": records["plain"]["selected_logits_cid"]
            == plain_record["selected_logits_cid"],
            "original_plain_predictions_preserved": torch.equal(
                plain_predictions, predictions["plain"]
            ),
            "identical_top1": differences["top1_changed"] == 0,
            "selected_logit_tolerance": max_logit <= EVALUATION["logit_atol"],
            "attention_tolerance": max_attention <= EVALUATION["attention_atol"],
            "nll_tolerance": differences["nll_abs_difference"]
            <= EVALUATION["nll_atol"],
            "unchanged_learned_state": before == _learned_state_cid(model),
            "tied_head": model.lm_head.weight
            is model.backbone.embeddings.word_embeddings.weight,
            "zero_future_contributions": records["plain"]["future_attention_nonzero"]
            == records["r4"]["future_attention_nonzero"]
            == records["r4"]["audit_totals"]["future_position_reads"]
            == 0,
        }
        primary = {
            "criteria": criteria,
            "passed": all(criteria.values()),
            "differences": differences,
            **records,
        }
        control: dict[str, Any] = {
            "status": "NOT_RUN_R4_PRESERVATION_MISS",
            "decisions": 0,
        }
        if primary["passed"]:
            control_scores, control_predictions = _Scores(), []
            for start in range(0, rows, EVALUATION["batch_size"]):
                stop = start + EVALUATION["batch_size"]
                budget.check()
                output = wrapper.forward_selected(
                    tensors["inputs"][start:stop],
                    tensors["positions"][start:stop],
                    execution="source_frame_permuted",
                )
                control_predictions.append(
                    control_scores.add(
                        output, tensors["targets"][start:stop], wrapper.last_audit
                    )
                )
                del output
                budget.check()
            predicted = torch.cat(control_predictions)
            metrics = _binding_metrics(
                predicted, tensors["targets"], tensors["pair_types"]
            )
            known_r4 = _binding_metrics(
                predictions["r4"], tensors["targets"], tensors["pair_types"]
            )["known_rate"]
            record = control_scores.record()
            control = {
                "status": "TRANSPORT_CONTROL_EXERCISED",
                "scores": record,
                "behavior": metrics,
                "top1_changed": int((predictions["r4"] != predicted).sum()),
                **_control_decision(
                    records["r4"], record, known_r4 - metrics["known_rate"]
                ),
            }
    return {
        "primary": primary,
        "control": control,
        "frames": {
            "native_map_entries": frames.token_leaf_indices.numel(),
            "frame_artifact_cid": frames.frame_artifact_cid,
            "token_map_artifact_cid": frames.artifact_cid,
            "reached_indices": records["r4"]["reached_frame_indices"],
        },
        "optimizer_updates": 0,
    }


def _evaluate(
    root: Path,
    preparation: Mapping[str, Any],
    fitted: Mapping[str, Any],
    budget: _Budget,
    phase: str,
) -> dict[str, Any]:
    model = _load_model({"source": {"root": str(root), "model": fitted["artifact"]}})
    state_before = _learned_state_cid(model)
    construction = data.load_training(root / "data", mixed=False)
    construction_record, _ = _plain_score(
        model,
        *(construction[f"train_{name}"] for name in ("inputs", "positions", "targets")),
        budget,
    )
    del construction
    atomic_write_json(
        root / f"{phase}-progress.json",
        {
            "phase": "construction_scored",
            "correct": construction_record["top1_correct"],
            "elapsed_seconds": budget.elapsed,
        },
    )
    # This is the first model-facing development access. Training never loads it.
    tensors = data.load_development(root / "data")
    development_record, predictions = _plain_score(
        model, *(tensors[name] for name in ("inputs", "positions", "targets")), budget
    )
    behavior = _binding_metrics(predictions, tensors["targets"], tensors["pair_types"])
    language = _language_decision(construction_record, behavior)
    print(
        f"#1063 {phase}: construction={construction_record['top1_correct']}/8192 development={behavior['known_correct']}/1024 groups={behavior['complete_groups_correct']}/256 unknown={behavior['unknown_correct']}/256",
        flush=True,
    )
    r4: dict[str, Any] = {"status": "NOT_RUN_ENGLISH_BINDING_MISS", "decisions": 0}
    status, next_action = language["status"], language["next_action"]
    if language["passed"]:
        r4 = _r4_evaluation(
            model, preparation, tensors, development_record, predictions, budget
        )
        if r4["primary"]["passed"]:
            status = "ENGLISH_BINDING_R4_PRESERVED"
            next_action = "FREEZE_BROADER_LANGUAGE_CONTEXT_APPLICATION"
        else:
            status = "ENGLISH_BINDING_R4_PRESERVATION_MISS"
            next_action = "ISOLATE_R4_INTEGRATION_WITHOUT_RETRAINING"
    if state_before != _learned_state_cid(model) or any(
        parameter.grad is not None for parameter in model.parameters()
    ):
        raise ValueError("evaluation changed learned state or accumulated gradients")
    if (
        construction_record["future_attention_nonzero"]
        or development_record["future_attention_nonzero"]
    ):
        raise ValueError("ordinary English evaluation used future attention")
    budget.check()
    return {
        "status": status,
        "next_action": next_action,
        "language": {
            **language,
            "construction": construction_record,
            "development": development_record,
            "behavior": behavior,
        },
        "r4": r4,
        "learned_state_before": state_before,
        "learned_state_after": _learned_state_cid(model),
        "evaluation_optimizer_updates": 0,
        "prior_model_checkpoint_reads": 0,
        "evaluation_checkpoint_optimizer_rng_reads": 0,
        "model_label_arguments": 0,
        "vocabulary_filtering": False,
        "population": "frozen construction-disjoint owner-object combinations; final-artifact evaluation only",
        "historical_1057_control": "NOT_RUN_PRIMARY_MISS; unchanged",
        "examples": [
            {
                "group": row // 5,
                "variant": data.VARIANTS[row % 5],
                "prompt": data.decode(tensors["inputs"][row], skip_bos=True),
                "expected": data.VOCABULARY[int(tensors["targets"][row, 0])],
                "predicted": data.VOCABULARY[int(predictions[row, 0])],
            }
            for row in range(20)
        ],
    }


def _phase(root: Path, *, replay: bool) -> dict[str, Any]:
    root = root.resolve()
    preparation = contract.validate_preparation(root)
    fitted = _fit_record(root, preparation)
    expected = _read_bound(root / "result.json", "result_cid") if replay else None
    carried = float(fitted["elapsed_seconds"])
    if expected:
        if (
            expected["preparation_cid"] != preparation["preparation_cid"]
            or expected["fit_cid"] != fitted["fit_cid"]
        ):
            raise ValueError("replay source and frozen result differ")
        if expected["schema"] != RESULT_SCHEMA or expected["evidence_cid"] != cid_bytes(
            canonical_json_bytes(expected["evidence"])
        ):
            raise ValueError("invalid frozen result evidence")
        if (
            fitted["status"] != "FIT_COMPLETE"
            or expected["evidence"]["status"] == "INCOMPLETE_RESOURCE"
        ):
            raise ResourceBudgetExceeded(
                "incomplete campaign cannot receive successful replay"
            )
        if expected["process_id"] == os.getpid():
            raise ValueError("replay requires a fresh process")
        carried += float(expected["elapsed_seconds"])
    phase, field = ("replay", "replay_cid") if replay else ("run", "result_cid")
    path = root / ("replay.json" if replay else "result.json")
    if path.exists():
        previous = _read_bound(path, field)
        if (
            previous["preparation_cid"] != preparation["preparation_cid"]
            or previous["fit_cid"] != fitted["fit_cid"]
            or (expected and previous["result_cid"] != expected["result_cid"])
        ):
            raise ValueError("existing result belongs to another campaign")
        return previous
    budget = _Budget(carried)
    runtime = _runtime()
    if expected and runtime != expected["runtime"]:
        raise ValueError("replay runtime differs")
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
            "next_action": "RETAIN_PARTIAL_ARTIFACT_AND_REPORT_RESOURCE_LIMIT",
            "language": "NOT_RUN_INCOMPLETE_FIT",
            "r4": "NOT_RUN_INCOMPLETE_FIT",
            "completed_updates": fitted["completed_updates"],
        }
    else:
        try:
            budget.check()
            evidence = _evaluate(root, preparation, fitted, budget, phase)
            if (
                contract.validate_preparation(root) != preparation
                or _fit_record(root, preparation) != fitted
            ):
                raise ValueError("campaign bindings changed during evaluation")
            budget.check()
        except ResourceBudgetExceeded as error:
            if replay:
                raise
            progress = root / f"{phase}-progress.json"
            evidence = {
                "status": "INCOMPLETE_RESOURCE",
                "reason": str(error),
                "next_action": "RETAIN_ARTIFACT_AND_REPORT_RESOURCE_LIMIT",
                "progress": json.loads(progress.read_text())
                if progress.exists()
                else None,
            }
    if expected and canonical_json_bytes(evidence) != canonical_json_bytes(
        expected["evidence"]
    ):
        raise ValueError("fresh-process behavior, logits or audits differ")
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
    result = _write_exclusive(path, body, field)
    print(f"#1063 {phase}={result[field]} status={evidence['status']}", flush=True)
    return result


def run(root: Path) -> dict[str, Any]:
    return _phase(root, replay=False)


def verify(root: Path) -> dict[str, Any]:
    return _phase(root, replay=True)
