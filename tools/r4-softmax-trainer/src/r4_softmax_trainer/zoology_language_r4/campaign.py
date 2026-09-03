"""One frozen two-seam R4 preservation campaign and complete process replay."""

from __future__ import annotations

import copy
import json
import os
import time
from pathlib import Path

import torch

from ..provenance import atomic_write_json, canonical_json_bytes, cid_bytes
from ..zoology_compound_binding.campaign import _tensor_cid
from ..zoology_language_interface import campaign as ordinary
from ..zoology_language_interface import data
from ..zoology_r4_inference.campaign import (
    ResourceBudgetExceeded,
    _learned_state_cid,
    _peak_rss_bytes,
    _read_bound,
    _write_exclusive,
)
from ..zoology_r4_inference.frames import load_frames
from . import contract
from .attention import AUDIT_COUNTS, R4LanguageInterfaceInference, work_counts

RESULT_SCHEMA = "uor-r4.language-r4-inference-result/1"
REPLAY_SCHEMA = "uor-r4.language-r4-inference-replay/1"
EXECUTIONS = (
    "plain",
    "r4",
    "token_source_frame_permuted",
    "fact_source_frame_permuted",
)


class _Budget:
    def __init__(self, carried: float = 0.0) -> None:
        self.carried, self.started = carried, time.monotonic()

    @property
    def elapsed(self) -> float:
        return time.monotonic() - self.started

    def check(self) -> None:
        if self.elapsed + self.carried > contract.EVALUATION["max_elapsed_seconds"]:
            raise ResourceBudgetExceeded(
                "combined inference/replay exceeded 900 seconds"
            )
        if _peak_rss_bytes() > contract.EVALUATION["max_rss_bytes"]:
            raise ResourceBudgetExceeded("inference exceeded 4 GiB peak RSS")


def _views(source: dict):
    root = Path(source["root"]) / "data"
    for population, loader, view_ids, rows in (
        ("construction", data.load_construction, (0, 1), 10240),
        ("development", data.load_development, (0, 1, 2, 3), 1280),
    ):
        tensors = loader(root)
        for view_id in view_ids:
            view = ordinary._view(tensors, view_id)
            if len(view["inputs"]) != rows:
                raise ValueError("bound language view is incomplete")
            yield population, view_id, view


class _Capture:
    """Reuse the unchanged historical scorer, retaining only missing comparisons."""

    def __init__(self, wrapper) -> None:
        self.wrapper = wrapper
        self.logits, self.vectors = [], []

    def __call__(self, inputs, lengths, *, control="none"):
        output = self.wrapper(inputs, lengths, control=control)
        self.logits.append(output["logits"])
        self.vectors.append(output["role_vectors"])
        return output


def _score(wrapper, tensors, budget) -> tuple[dict, dict]:
    wrapper.reset_audit()
    capture = _Capture(wrapper)
    row, predictions, attention, roles = ordinary._score(capture, tensors, budget)
    # No canonical hard-field oracle forwards: all learned-path source fields
    # are reproduced, while the historical oracle is not a comparator here.
    logits, vectors = torch.cat(capture.logits), torch.cat(capture.vectors)
    if vectors.shape != (len(predictions), 5, 3, 64) or not bool(
        torch.isfinite(vectors).all()
    ):
        raise ValueError("incomplete/nonfinite computed soft role vectors")
    row["role_vectors_cid"] = _tensor_cid(vectors)
    row["audit"] = copy.deepcopy(wrapper.audit)
    row["work_valid"] = all(
        type(row["audit"].get(key)) is int and row["audit"][key] == value
        for key, value in work_counts(
            tensors["inputs"], tensors["lengths"], wrapper.execution
        ).items()
    )
    budget.check()
    return row, {
        "logits": logits,
        "predictions": predictions,
        "binding_attention": attention,
        "role_attention": roles,
        "role_vectors": vectors,
    }


def _historical_exact(row, historical) -> bool:
    fields = set(row) - {"role_vectors_cid", "audit", "work_valid"}
    return all(row[key] == historical[key] for key in fields) and (
        ordinary._qualified(row) == historical["qualification"]
    )


def _differences(left, right, plain, r4, tensors) -> dict:
    supported = tensors["variant_ids"] < 4
    masks = {
        "all": torch.ones_like(supported),
        "supported": supported,
        "unknown": ~supported,
    }
    strata = {}
    for name, mask in masks.items():
        strata[name] = {
            "top1_changed": int(
                (left["predictions"][mask] != right["predictions"][mask]).sum()
            ),
            "logits_max_abs": float(
                (left["logits"][mask] - right["logits"][mask]).abs().max()
            ),
            "binding_attention_max_abs": float(
                (left["binding_attention"][mask] - right["binding_attention"][mask])
                .abs()
                .max()
            ),
            "role_vectors_max_abs": float(
                (left["role_vectors"][mask] - right["role_vectors"][mask]).abs().max()
            ),
            "nll_abs_difference": abs(
                plain["records"][name]["nll_nats"] - r4["records"][name]["nll_nats"]
            ),
        }
    return {
        "strata": strata,
        "role_attention_exact": torch.equal(
            left["role_attention"], right["role_attention"]
        ),
        "role_predictions_exact": plain["role_prediction_positions"]
        == r4["role_prediction_positions"],
    }


def _primary_decision(plain, r4, differences) -> dict:
    limits = contract.EVALUATION
    criteria = {
        "exact_ordinary_reproduction": plain["historical_exact"],
        "ordinary_work_valid": plain["work_valid"],
        "r4_work_valid": r4["work_valid"],
        "complete_strata": set(differences["strata"])
        == {"all", "supported", "unknown"},
        "complete_same_work_support": plain["work"] == r4["work"],
        "exact_role_attention": differences["role_attention_exact"],
        "exact_role_predictions": differences["role_predictions_exact"],
        "exact_role_accuracy": plain["role_accuracy"] == r4["role_accuracy"],
        "binding_groups_preserved": plain["groups"] == r4["groups"],
        "syntax_pairs_preserved": plain["syntax_pairs"] == r4["syntax_pairs"],
        "native_frame_coverage_preserved": all(
            plain["audit"][key] == r4["audit"][key]
            for key in (
                "reached_token_frame_indices",
                "reached_clause_frame_indices",
                "reached_frame_indices",
            )
        ),
    }
    for name, delta in differences["strata"].items():
        for metric, threshold in (
            ("top1_changed", 0),
            ("logits_max_abs", limits["logit_atol"]),
            ("binding_attention_max_abs", limits["attention_atol"]),
            ("role_vectors_max_abs", limits["role_vector_atol"]),
            ("nll_abs_difference", limits["nll_atol"]),
        ):
            criteria[f"{name}_{metric}"] = delta[metric] <= threshold
    return {"passed": all(criteria.values()), "criteria": criteria}


def _control_decision(primary, controlled, execution, preflight) -> dict:
    coherent = primary["r4"]
    expected = preflight["controls"][execution]
    seam = "token" if execution == "token_source_frame_permuted" else "fact"
    other_seam = "fact" if seam == "token" else "token"
    integrity = {
        "complete_transport": controlled["work_valid"],
        "same_work": all(
            controlled["audit"].get(k) == coherent["audit"].get(k) for k in AUDIT_COUNTS
        ),
        "same_support": controlled["work"] == coherent["work"],
        "reader_attention_exact": controlled["role_attention_cid"]
        == coherent["role_attention_cid"],
        "frame_preflight_passed": preflight["passed"] and expected["passed"],
        "actual_changed_positions_match_preflight": controlled["audit"][
            f"{seam}_source_frame_positions_changed"
        ]
        == expected["source_frame_positions_changed"],
        "actual_changed_matrices_match_preflight": controlled["audit"][
            f"{seam}_source_frame_matrices_changed"
        ]
        == expected["source_frame_matrices_changed"],
        "nontrivial_frame_changes": controlled["audit"][
            f"{seam}_source_frame_matrices_changed"
        ]
        > 0,
        "other_transport_seam_coherent": controlled["audit"][
            f"{other_seam}_source_frame_positions_changed"
        ]
        == 0
        and controlled["audit"][f"{other_seam}_source_frame_matrices_changed"] == 0,
        "native_frame_coverage_unchanged": controlled["audit"]["reached_frame_indices"]
        == coherent["audit"]["reached_frame_indices"],
    }
    if execution == "fact_source_frame_permuted":
        integrity["coherent_role_vectors_exact"] = (
            controlled["role_vectors_cid"] == coherent["role_vectors_cid"]
        )
    before = primary["plain"]["records"]["supported"]
    after = controlled["records"]["supported"]
    integrity["complete_supported_decisions"] = (
        before["decisions"] == after["decisions"]
    )
    drop = before["top1_rate"] - after["top1_rate"]
    valid = all(integrity.values())
    return {
        "valid": valid,
        "integrity": integrity,
        "supported_correct_drop": before["top1_correct"] - after["top1_correct"],
        "supported_drop_percentage_points": 100 * drop,
        "strong_transport_sensitivity": valid
        and drop >= contract.EVALUATION["strong_control_drop"],
        "unknown_scope": "descriptive; a fixed null frame does not fix its attention weight",
    }


def _conditional_controls(primary, execute):
    if primary["passed"]:
        return execute()
    return {
        "status": "NOT_RUN_PRIMARY_MISS",
        "views": [],
        "model_decisions": 0,
        "valid": False,
        "strong_transport_sensitivity": False,
    }


def _decision(reference_exact, primary, controls) -> dict:
    if not reference_exact:
        status, action = (
            "LANGUAGE_R4_REFERENCE_MISMATCH",
            "STOP_AND_RESOLVE_ORDINARY_REPRODUCTION",
        )
    elif not primary["passed"]:
        status, action = (
            "LANGUAGE_R4_PRESERVATION_MISS",
            "RETAIN_LEARNED_ORDINARY_INTERFACE_AND_LOCALIZE_TRANSPORT_MISMATCH",
        )
    elif not controls["valid"]:
        status, action = (
            "LANGUAGE_R4_PRESERVED_CONTROL_INVALID",
            "RETAIN_PRESERVATION_AND_REPAIR_CONTROL_INTEGRITY_SEPARATELY",
        )
    elif not controls["strong_transport_sensitivity"]:
        status, action = (
            "LANGUAGE_R4_PRESERVED_CONTROL_WEAK",
            "RETAIN_PRESERVATION_AND_REVIEW_TRANSPORT_INTERVENTION_SEPARATELY",
        )
    else:
        status, action = (
            "LANGUAGE_R4_PRESERVED",
            "RETAIN_TWO_STAGE_R4_AND_SEPARATELY_FREEZE_CAUSAL_OUTPUT_STATE_PROTOTYPE",
        )
    return {
        "status": status,
        "preserved": reference_exact and primary["passed"],
        "next_action": action,
    }


def _evaluate(root, prep, budget, phase) -> dict:
    frames = load_frames(Path(prep["frames"]["root"]))
    model = contract.load_source_model(prep)
    before = {
        "core": _learned_state_cid(model.core),
        "reader": ordinary._state(model.reader),
    }
    wrappers = {
        name: R4LanguageInterfaceInference(model, frames, execution=name)
        for name in EXECUTIONS
    }
    references, ordinary_views, primary_views = {}, [], []

    def progress(execution, population, view_id):
        atomic_write_json(
            root / f"{phase}-progress.json",
            {
                "phase": phase,
                "execution": execution,
                "population": population,
                "view_id": view_id,
                "elapsed_seconds": budget.elapsed,
            },
        )
        budget.check()

    # All ordinary rows must reproduce before any R4 learned forward.
    for population, view_id, tensors in _views(prep["source"]):
        row, raw = _score(wrappers["plain"], tensors, budget)
        historical = prep["source"]["baseline_history"][population][view_id]
        if historical["view_id"] != view_id:
            raise ValueError("historical view order differs")
        row["historical_exact"] = _historical_exact(row, historical)
        row.update(population=population, view_id=view_id)
        ordinary_views.append(row)
        references[(population, view_id)] = (row, raw)
        progress("plain", population, view_id)
    reference_exact = all(row["historical_exact"] for row in ordinary_views)
    if reference_exact:
        for population, view_id, tensors in _views(prep["source"]):
            plain, raw_plain = references[(population, view_id)]
            row, raw = _score(wrappers["r4"], tensors, budget)
            differences = _differences(raw_plain, raw, plain, row, tensors)
            primary_views.append(
                {
                    "population": population,
                    "view_id": view_id,
                    "plain": plain,
                    "r4": row,
                    "differences": differences,
                    **_primary_decision(plain, row, differences),
                }
            )
            del raw
            progress("r4", population, view_id)
    primary = {
        "passed": reference_exact
        and len(primary_views) == 6
        and all(v["passed"] for v in primary_views),
        "views": primary_views,
    }

    def controls():
        views = []
        for execution in EXECUTIONS[2:]:
            for index, (population, view_id, tensors) in enumerate(
                _views(prep["source"])
            ):
                row, raw = _score(wrappers[execution], tensors, budget)
                preflight = prep["preflight"]["views"][index]
                plain = references[(population, view_id)][1]
                supported = tensors["variant_ids"] < 4
                views.append(
                    {
                        "execution": execution,
                        "population": population,
                        "view_id": view_id,
                        **row,
                        "changed_predictions": {
                            name: int(
                                (
                                    raw["predictions"][mask]
                                    != plain["predictions"][mask]
                                ).sum()
                            )
                            for name, mask in (
                                ("supported", supported),
                                ("unknown", ~supported),
                            )
                        },
                        **_control_decision(
                            primary_views[index], row, execution, preflight
                        ),
                    }
                )
                del raw
                progress(execution, population, view_id)
        return {
            "status": "SCORED_FIXED_ARTIFACT",
            "views": views,
            "model_decisions": sum(v["records"]["all"]["decisions"] for v in views),
            "valid": len(views) == 12 and all(v["valid"] for v in views),
            "strong_transport_sensitivity": len(views) == 12
            and all(v["strong_transport_sensitivity"] for v in views),
        }

    controlled = _conditional_controls(primary, controls)
    after = {
        "core": _learned_state_cid(model.core),
        "reader": ordinary._state(model.reader),
    }
    if (
        before != after
        or after["core"] != prep["source"]["core"]["model"]["state_cid"]
        or after["reader"] != prep["source"]["reader"]["state_cid"]
        or model.core.lm_head.weight is not model.core.embedding.weight
        or any(m.training for m in model.modules())
        or any(p.requires_grad or p.grad is not None for p in model.parameters())
    ):
        raise ValueError(
            "inference changed learned state, tying or frozen evaluation mode"
        )
    budget.check()
    return {
        **_decision(reference_exact, primary, controlled),
        "ordinary_exact_reproduction": reference_exact,
        "ordinary_views": ordinary_views,
        "primary": primary,
        "controls": controlled,
        "learned_state_before": before,
        "learned_state_after": after,
        "core_parameters": model.core.parameter_count(),
        "reader_parameters": model.reader.parameter_count(),
        "total_parameters": sum(p.numel() for p in model.parameters()),
        "preflight": prep["preflight"],
        "optimizer_updates": 0,
        "new_parameters": 0,
        "checkpoint_optimizer_rng_payload_reads": 0,
        "model_label_arguments": 0,
        "canonical_hard_field_oracle_forwards": 0,
        "new_population_generation": 0,
        "geometry_changes": 0,
        "native_geometry_exports": 0,
        "candidate_model_loads": 1,
        "scope": "preservation of both learned soft mixtures on six observed #1077 views; supplied clauses, fixed lexicon and question form; no generation, new generalization or geometry advantage",
    }


def _phase(root: Path, replay: bool) -> dict:
    root = root.resolve()
    expected = _read_bound(root / "result.json", "result_cid") if replay else None
    budget = _Budget(expected["elapsed_seconds"] if expected else 0.0)
    runtime = ordinary._runtime(4)
    prep = contract.validate_preparation(root)
    if runtime != prep["source"]["runtime"]:
        raise ValueError(
            "inference must reproduce the frozen four-thread source runtime"
        )
    identities = {
        "issue": contract.ISSUE,
        "preparation_cid": prep["preparation_cid"],
        "implementation_cid": prep["implementation"]["tree_cid"],
        "source_result_cid": prep["source"]["result_cid"],
        "reader": prep["source"]["reader"],
        "core": prep["source"]["core"]["model"],
        "frames": prep["frames"],
        "runtime": runtime,
    }
    if expected and (
        expected["schema"] != RESULT_SCHEMA
        or any(expected[key] != value for key, value in identities.items())
        or expected["process_id"] == os.getpid()
        or expected["evidence_cid"]
        != cid_bytes(canonical_json_bytes(expected["evidence"]))
    ):
        raise ValueError(
            "replay requires the bound complete result and fresh matched process"
        )
    if expected and expected["evidence"]["status"] == "INCOMPLETE_RESOURCE":
        raise ResourceBudgetExceeded(
            "incomplete result cannot receive successful replay"
        )
    phase = "replay" if replay else "run"
    _write_exclusive(
        root / f"{phase}-started.json",
        {
            "preparation_cid": prep["preparation_cid"],
            "process_id": os.getpid(),
            "carried_elapsed_seconds": budget.carried,
            "runtime": runtime,
        },
        "started_cid",
    )
    try:
        budget.check()
        evidence = _evaluate(root, prep, budget, phase)
        if contract.validate_preparation(root) != prep:
            raise ValueError("bound source, frames, data or implementation changed")
        budget.check()
    except ResourceBudgetExceeded as error:
        if replay:
            raise
        progress = root / "run-progress.json"
        evidence = {
            "status": "INCOMPLETE_RESOURCE",
            "reason": str(error),
            "optimizer_updates": 0,
            "last_completed_progress": json.loads(progress.read_text())
            if progress.exists()
            else None,
        }
    evidence_cid = cid_bytes(canonical_json_bytes(evidence))
    if expected and canonical_json_bytes(evidence) != canonical_json_bytes(
        expected["evidence"]
    ):
        raise ValueError("fresh-process complete evidence differs")
    body = {
        "schema": REPLAY_SCHEMA if replay else RESULT_SCHEMA,
        **identities,
        "process_id": os.getpid(),
        "evidence_cid": evidence_cid,
        "elapsed_seconds": budget.elapsed,
        "combined_elapsed_seconds": budget.carried + budget.elapsed,
        "peak_rss_bytes": _peak_rss_bytes(),
    }
    if replay:
        body.update(
            result_cid=expected["result_cid"],
            exact_replay=True,
            fresh_process=True,
            optimizer_updates=0,
        )
    else:
        body["evidence"] = evidence
    return _write_exclusive(
        root / ("replay.json" if replay else "result.json"),
        body,
        "replay_cid" if replay else "result_cid",
    )


def run(root: Path) -> dict:
    return _phase(root, False)


def verify(root: Path) -> dict:
    return _phase(root, True)
