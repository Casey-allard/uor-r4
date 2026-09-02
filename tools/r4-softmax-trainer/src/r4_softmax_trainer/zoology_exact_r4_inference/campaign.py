"""Preserve the final #1057 checkpoint through the unchanged #1059 adapter."""

from __future__ import annotations

import json
import math
import os
from collections.abc import Mapping
from pathlib import Path
from typing import Any

import torch
from torch import Tensor

from ..provenance import atomic_write_json, canonical_json_bytes, cid_bytes
from ..zoology_r4_inference.attention import R4ZoologyInference
from ..zoology_r4_inference.campaign import (
    _WORK_FIELDS,
    ResourceBudgetExceeded,
    _Budget,
    _configure_cpu,
    _learned_state_cid,
    _load_model,
    _maximum_difference,
    _peak_rss_bytes,
    _read_bound,
    _Scores,
    _write_exclusive,
)
from ..zoology_r4_inference.contract import EVALUATION as REUSED_POLICY
from ..zoology_r4_inference.frames import load_frames
from .contract import EVALUATION, load_development, validate_preparation

RESULT_SCHEMA = "uor-r4.zoology-exact-r4-inference-result/1"
REPLAY_SCHEMA = "uor-r4.zoology-exact-r4-inference-replay/1"


def _primary_decision(
    plain: Mapping[str, Any],
    r4: Mapping[str, Any],
    differences: Mapping[str, Any],
    *,
    state_unchanged: bool,
    vocabulary_covered: bool,
) -> dict[str, Any]:
    decisions = EVALUATION["rows"] * EVALUATION["queries_per_row"]
    length = EVALUATION["sequence_length"]
    # Both preserved source layers have one head and admit every prefix pair.
    causal_pairs = EVALUATION["rows"] * 2 * length * (length + 1) // 2
    criteria = {
        "complete_decisions": plain["decisions"] == r4["decisions"] == decisions,
        "historical_correct_reproduced": plain["top1_correct"]
        == EVALUATION["historical_correct"],
        "identical_top1": differences["top1_changed"] == 0,
        "selected_logit_tolerance": differences["selected_logits_max_abs"]
        <= EVALUATION["logit_atol"],
        "attention_tolerance": differences["attention_max_abs"]
        <= EVALUATION["attention_atol"],
        "nll_tolerance": differences["nll_abs_difference"] <= EVALUATION["nll_atol"],
        "all_causal_pairs_admitted": plain["audit_totals"]["admitted_attention_pairs"]
        == r4["audit_totals"]["admitted_attention_pairs"]
        == causal_pairs,
        "zero_future_attention_weight": plain["future_attention_nonzero"]
        == r4["future_attention_nonzero"]
        == 0,
        "causal_r4_source_reads": r4["audit_totals"]["future_position_reads"] == 0,
        "unchanged_learned_state": state_unchanged,
        "complete_vocabulary_coverage": vocabulary_covered,
    }
    return {"passed": all(criteria.values()), "criteria": criteria}


def _control_decision(
    plain: Mapping[str, Any], r4: Mapping[str, Any], control: Mapping[str, Any]
) -> dict[str, Any]:
    work_fields = set(_WORK_FIELDS) - {
        "source_frame_positions_changed",
        "source_frame_matrices_changed",
    }
    integrity = {
        "complete_decisions": control["decisions"]
        == EVALUATION["rows"] * EVALUATION["queries_per_row"],
        "zero_future_attention_weight": control["future_attention_nonzero"] == 0,
        "causal_control_source_reads": control["audit_totals"]["future_position_reads"]
        == 0,
        "same_work": all(
            isinstance(control["audit_totals"].get(name), int)
            and not isinstance(control["audit_totals"].get(name), bool)
            and control["audit_totals"][name] >= 0
            and control["audit_totals"][name] == r4["audit_totals"].get(name)
            for name in work_fields
        ),
    }
    valid = all(integrity.values())
    drop = plain["top1_rate"] - control["top1_rate"]
    return {
        "status": "RUN" if valid else "INVALID_CONTROL_INTEGRITY",
        "integrity": integrity,
        "recall_drop": drop,
        "recall_drop_percentage_points": 100.0 * drop,
        "strong_transport_sensitivity": valid
        and drop >= EVALUATION["strong_control_drop"],
        "claim_boundary": "new inconsistent-transport control; not the unrun #1057 binding control or H4 superiority",
    }


def _evaluate(
    root: Path, preparation: Mapping[str, Any], budget: _Budget, *, phase: str
) -> dict[str, Any]:
    tensors = load_development(preparation)
    model = _load_model(preparation)
    expected_state = preparation["source"]["model"]["state_cid"]
    frames = load_frames(Path(preparation["frames"]["root"]))
    wrapper = R4ZoologyInference(model, frames)
    model.eval()
    model.requires_grad_(False)
    vocabulary_covered = (
        model.config.vocab_size == EVALUATION["vocab_size"]
        and frames.token_leaf_indices.numel() == 8192
    )

    def check_state() -> bool:
        return (
            _learned_state_cid(model) == expected_state
            and model.lm_head.weight is model.backbone.embeddings.word_embeddings.weight
            and not any(module.training for module in model.modules())
            and not any(
                parameter.requires_grad or parameter.grad is not None
                for parameter in model.parameters()
            )
        )

    if not check_state() or not vocabulary_covered:
        raise ValueError(
            "adapter changed learned state, eval mode or complete vocabulary coverage"
        )
    scores = {name: _Scores() for name in ("plain", "r4")}
    reference_logits: list[Tensor] = []
    max_logit = max_attention = 0.0
    top1_changed = 0
    rows, batch_size = EVALUATION["rows"], EVALUATION["batch_size"]
    batch_count = math.ceil(rows / batch_size)

    def progress(arm: str, index: int, metrics: Mapping[str, Any]) -> None:
        atomic_write_json(
            root / f"{phase}-progress.json",
            {
                "phase": phase,
                "arm": arm,
                "batch": index,
                "batches": batch_count,
                "elapsed_seconds": budget.elapsed,
                "scores": metrics,
            },
        )
        print(
            f"#1061 {phase} {arm} batch={index}/{batch_count} elapsed={budget.elapsed:.3f}s",
            flush=True,
        )

    with torch.inference_mode():
        for index, start in enumerate(range(0, rows, batch_size), 1):
            inputs = tensors["test_inputs"][start : start + batch_size]
            positions = tensors["test_positions"][start : start + batch_size]
            targets = tensors["test_targets"][start : start + batch_size]
            budget.check()
            plain = wrapper.forward_selected(
                inputs, positions, execution="plain", return_attention=True
            )
            plain_predictions = scores["plain"].add(plain, targets, wrapper.last_audit)
            reference_logits.append(plain.logits.detach().float().clone())
            budget.check()
            r4 = wrapper.forward_selected(
                inputs, positions, execution="r4", return_attention=True
            )
            r4_predictions = scores["r4"].add(r4, targets, wrapper.last_audit)
            logits_delta, attention_delta = _maximum_difference(plain, r4)
            max_logit = max(max_logit, logits_delta)
            max_attention = max(max_attention, attention_delta)
            top1_changed += int(
                torch.count_nonzero(plain_predictions != r4_predictions)
            )
            del plain, r4
            progress(
                "plain+r4",
                index,
                {name: score.record() for name, score in scores.items()},
            )
            budget.check()

        plain_record, r4_record = scores["plain"].record(), scores["r4"].record()
        differences = {
            "selected_logits_max_abs": max_logit,
            "attention_max_abs": max_attention,
            "top1_changed": top1_changed,
            "nll_abs_difference": abs(plain_record["nll_nats"] - r4_record["nll_nats"]),
        }
        primary = {
            "plain": plain_record,
            "r4": r4_record,
            "differences": differences,
            **_primary_decision(
                plain_record,
                r4_record,
                differences,
                state_unchanged=check_state(),
                vocabulary_covered=vocabulary_covered,
            ),
        }
        control: dict[str, Any] = {"status": "NOT_RUN_PRIMARY_MISS", "decisions": 0}
        if primary["passed"]:
            control_scores = _Scores()
            changed = 0
            control_max = 0.0
            for index, start in enumerate(range(0, rows, batch_size), 1):
                budget.check()
                inputs = tensors["test_inputs"][start : start + batch_size]
                positions = tensors["test_positions"][start : start + batch_size]
                targets = tensors["test_targets"][start : start + batch_size]
                output = wrapper.forward_selected(
                    inputs,
                    positions,
                    execution="source_frame_permuted",
                    return_attention=True,
                )
                predictions = control_scores.add(output, targets, wrapper.last_audit)
                reference = reference_logits[index - 1]
                changed += int(
                    torch.count_nonzero(reference.argmax(dim=-1) != predictions)
                )
                control_max = max(
                    control_max,
                    float((reference.double() - output.logits.double()).abs().max()),
                )
                del output
                progress("source_frame_permuted", index, control_scores.record())
                budget.check()
            record = control_scores.record()
            control = {
                "metrics": record,
                "top1_changed": changed,
                "selected_logits_max_abs": control_max,
                **_control_decision(plain_record, r4_record, record),
            }

    if not check_state():
        raise ValueError(
            "inference changed learned tensors, tied weights or gradient state"
        )
    retained_bytes = sum(
        value.numel() * value.element_size() for value in reference_logits
    )
    del reference_logits
    budget.check()
    return {
        "status": "EXACT_DATA_R4_PRESERVED"
        if primary["passed"]
        else "EXACT_DATA_R4_PRESERVATION_MISS",
        "primary": primary,
        "control": control,
        "historical_1057_binding_control": "NOT_RUN_PRIMARY_MISS; unchanged",
        "population": "previously observed assignment-disjoint development",
        "learned_state_before": expected_state,
        "learned_state_after": _learned_state_cid(model),
        "tied_head_preserved": True,
        "optimizer_updates": 0,
        "training_tensor_values_loaded": 0,
        "checkpoint_optimizer_rng_reads": 0,
        "physical_binding_control_reads": 0,
        "model_label_arguments": 0,
        "reference_logits_retained_bytes": retained_bytes,
        "frame_coverage": {
            "model_vocabulary_entries": model.config.vocab_size,
            "native_map_entries": frames.token_leaf_indices.numel(),
            "direct_leaf_count": frames.direct_leaf_count,
            "native_witness_frame_count": frames.witness_frame_count,
            "inference_reached_frame_count": len(r4_record["reached_frame_indices"]),
            "inference_reached_frame_indices": r4_record["reached_frame_indices"],
            "frame_artifact_cid": frames.frame_artifact_cid,
            "token_map_artifact_cid": frames.artifact_cid,
        },
        "row_order": "canonical 0..1023; no shuffle or evaluation RNG",
        "evaluation": dict(EVALUATION),
    }


def _phase(root: Path, *, replay: bool) -> dict[str, Any]:
    root = root.resolve()
    expected = _read_bound(root / "result.json", "result_cid") if replay else None
    budget = _Budget(float(expected["elapsed_seconds"]) if expected else 0.0)
    preparation = validate_preparation(root)
    for field in ("threads", "interop_threads", "max_elapsed_seconds", "max_rss_bytes"):
        if EVALUATION[field] != REUSED_POLICY[field]:
            raise ValueError("new campaign changed the reused CPU/resource policy")
    if expected:
        if (
            expected.get("schema") != RESULT_SCHEMA
            or expected.get("preparation_cid") != preparation["preparation_cid"]
        ):
            raise ValueError("result and preparation differ")
        for key in ("source", "frames", "implementation"):
            if expected.get(key) != preparation[key]:
                raise ValueError(f"result {key} differs from preparation")
        if expected.get("evidence_cid") != cid_bytes(
            canonical_json_bytes(expected["evidence"])
        ):
            raise ValueError("result evidence identity differs")
        if expected["evidence"]["status"] == "INCOMPLETE_RESOURCE":
            raise ResourceBudgetExceeded(
                "resource-interrupted result cannot receive a successful replay"
            )
    phase = "replay" if replay else "run"
    output_path = root / ("replay.json" if replay else "result.json")
    cid_field = "replay_cid" if replay else "result_cid"
    if output_path.exists():
        previous = _read_bound(output_path, cid_field)
        if previous.get("preparation_cid") != preparation["preparation_cid"] or (
            expected and previous.get("result_cid") != expected["result_cid"]
        ):
            raise ValueError("existing output belongs to another source/result")
        return previous
    if expected and expected["process_id"] == os.getpid():
        raise ValueError("verification requires a fresh process")
    runtime = _configure_cpu()
    if expected and runtime != expected["runtime"]:
        raise ValueError("runtime differs from frozen inference result")
    budget.check()
    # Existing interrupted markers reject another attempt before model loading.
    _write_exclusive(
        root / f"{phase}-started.json",
        {
            "issue": 1061,
            "preparation_cid": preparation["preparation_cid"],
            "result_cid": expected["result_cid"] if expected else None,
            "process_id": os.getpid(),
            "carried_elapsed_seconds": budget.carried_seconds,
            "runtime": runtime,
        },
        "started_cid",
    )
    try:
        evidence = _evaluate(root, preparation, budget, phase=phase)
        if validate_preparation(root) != preparation:
            raise ValueError(
                "bound source, frames or implementation changed during inference"
            )
        budget.check()
    except ResourceBudgetExceeded as error:
        if replay:
            raise
        progress = root / "run-progress.json"
        evidence = {
            "status": "INCOMPLETE_RESOURCE",
            "reason": str(error),
            "last_completed_progress": json.loads(progress.read_text())
            if progress.exists()
            else None,
            "optimizer_updates": 0,
        }
    evidence_cid = cid_bytes(canonical_json_bytes(evidence))
    if expected and canonical_json_bytes(evidence) != canonical_json_bytes(
        expected["evidence"]
    ):
        raise ValueError("fresh-process inference metrics, logits or audits differ")
    body = {
        "schema": REPLAY_SCHEMA if replay else RESULT_SCHEMA,
        "issue": 1061,
        "preparation_cid": preparation["preparation_cid"],
        "source": preparation["source"],
        "frames": preparation["frames"],
        "implementation": preparation["implementation"],
        "runtime": runtime,
        "process_id": os.getpid(),
        "evidence_cid": evidence_cid,
        "elapsed_seconds": budget.elapsed,
        "peak_rss_bytes": _peak_rss_bytes(),
    }
    if expected:
        body.update(
            {
                "result_cid": expected["result_cid"],
                "exact_replay": True,
                "fresh_process": True,
                "optimizer_updates": 0,
                "combined_elapsed_seconds": budget.carried_seconds + budget.elapsed,
            }
        )
    else:
        body["evidence"] = evidence
    result = _write_exclusive(output_path, body, cid_field)
    print(f"#1061 {phase}={result[cid_field]} status={evidence['status']}", flush=True)
    return result


def run(root: Path) -> dict[str, Any]:
    """Execute one matched primary and its conditional transport control."""
    return _phase(root, replay=False)


def verify(root: Path) -> dict[str, Any]:
    """Replay all evidence exactly in a fresh process under the shared budget."""
    return _phase(root, replay=True)
