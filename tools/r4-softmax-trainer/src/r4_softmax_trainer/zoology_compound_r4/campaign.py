"""Preserve the frozen compound model through rectangular coherent R4 transport."""

from __future__ import annotations

import copy
import json
import os
import time
from pathlib import Path

import torch

from ..provenance import atomic_write_json, canonical_json_bytes, cid_bytes
from ..zoology_compound_binding import data
from ..zoology_compound_binding.campaign import _score
from ..zoology_compound_binding.model import load_model
from ..zoology_cyclic_facts.augmentation import rotate_inputs
from ..zoology_english_binding.campaign import _runtime
from ..zoology_r4_inference.campaign import (
    ResourceBudgetExceeded,
    _learned_state_cid,
    _peak_rss_bytes,
    _read_bound,
    _write_exclusive,
)
from ..zoology_r4_inference.frames import load_frames
from . import contract
from .attention import R4CompoundInference, frame_assignment

RESULT_SCHEMA = "uor-r4.compound-r4-inference-result/1"
REPLAY_SCHEMA = "uor-r4.compound-r4-inference-replay/1"
ROWS = {"construction": 10240, "development": 1280}
TRANSFORM_FACTORS = {
    "query_blocks_encoded": 16,
    "key_blocks_encoded": 80,
    "value_blocks_encoded": 80,
    "key_blocks_transported": 80,
    "value_blocks_transported": 80,
    "output_blocks_decoded": 16,
}


class _Budget:
    def __init__(self, carried: float = 0.0) -> None:
        self.carried = carried
        self.started = time.monotonic()

    @property
    def elapsed(self) -> float:
        return time.monotonic() - self.started

    def check(self) -> None:
        if self.carried + self.elapsed > contract.EVALUATION["max_elapsed_seconds"]:
            raise ResourceBudgetExceeded(
                "combined inference/replay exceeded 900 seconds"
            )
        if _peak_rss_bytes() > contract.EVALUATION["max_rss_bytes"]:
            raise ResourceBudgetExceeded("inference process exceeded 4 GiB peak RSS")


def _views(source: dict):
    root = Path(source["root"]) / "data"
    for population, loader in (
        ("construction", data.load_construction),
        ("development", data.load_development),
    ):
        tensors = loader(root)
        if len(tensors["inputs"]) != ROWS[population]:
            raise ValueError("bound population has an incomplete row count")
        for rotation in (0, 1, 2, 3):
            yield (
                population,
                rotation,
                {
                    **tensors,
                    "inputs": rotate_inputs(tensors["inputs"], rotation),
                },
            )


def _frame_view(inputs, supported, frames) -> dict:
    query, sources = frame_assignment(inputs, frames)
    shifted = torch.cat((sources[:, [1, 2, 3, 0]], sources[:, 4:5]), dim=1)
    changed = sources != shifted
    any_changed = changed.any(dim=1)
    known = int(supported.sum())
    if not known or bool(changed[:, 4].any()):
        raise ValueError("preflight needs supported rows and a fixed null frame")
    reached = torch.cat((query.reshape(-1), sources.reshape(-1))).unique(sorted=True)
    eligible = int(any_changed[supported].sum())
    return {
        "rows": len(inputs),
        "supported_rows": known,
        "unknown_rows": len(inputs) - known,
        "admitted_attention_pairs": len(inputs) * 5,
        "null_attention_pairs": len(inputs),
        "prefix_token_positions": 38,
        "future_token_reads": 0,
        "reached_frame_indices": reached.tolist(),
        "source_frame_positions_changed": len(inputs) * 4,
        "source_frame_matrices_changed": int(changed.sum()),
        "rows_with_changed_source_frame": int(any_changed.sum()),
        "supported_rows_with_changed_source_frame": eligible,
        "supported_loss_reachability_ceiling": eligible / known,
        "null_identity": bool((sources[:, 4] == frames.identity_index).all()),
        "passed": eligible / known >= contract.EVALUATION["strong_control_drop"],
    }


def structural_preflight(source: dict, frame_info: dict) -> dict:
    """Inspect bound observed inputs and native frames, never model parameters."""
    frames = load_frames(Path(frame_info["root"]))
    views = [
        {
            "population": population,
            "rotation": rotation,
            **_frame_view(tensors["inputs"], tensors["variant_ids"] < 4, frames),
        }
        for population, rotation, tensors in _views(source)
    ]
    return {
        "passed": all(view["passed"] and view["null_identity"] for view in views),
        "views": views,
        "model_forwards": 0,
        "new_populations": 0,
        "frame_map_tokens": frames.token_leaf_indices.numel(),
        "frame_count": len(frames.frame_matrices),
        "decision_rows_per_arm": sum(view["rows"] for view in views),
        "score_slots_per_arm": sum(view["admitted_attention_pairs"] for view in views),
        "scope": "label-free source-frame mismatch ceiling on already observed rows; not a predicted accuracy loss",
    }


def _historical_records(source: dict, population: str, rotation: int) -> dict:
    evidence = source["baseline_history"]
    if population == "construction":
        view = evidence["construction_views"][rotation]
        names = {"all": "all", "supported": "construction", "unknown": "unknown"}
    else:
        view = evidence["development"]["views"][rotation]
        names = {"all": "record", "supported": "supported", "unknown": "unknown"}
    if view["rotation"] != rotation:
        raise ValueError("historical rotation order differs")
    return {name: view[key] for name, key in names.items()}


def _groups(predictions, tensors) -> dict:
    observed = predictions.reshape(-1, 5)
    targets = tensors["targets"].reshape_as(observed)
    types = tensors["pair_types"].reshape_as(observed)
    if not bool((types == types[:, :1]).all()):
        raise ValueError("question family changes inside a group")
    correct = observed == targets
    complete = correct[:, :4].all(dim=1)
    return {
        "groups": len(observed),
        "complete_supported_quartets": int(complete.sum()),
        "complete_five_answer_groups": int(correct.all(dim=1).sum()),
        "by_question_type": {
            name: {
                "groups": int((types[:, 0] == index).sum()),
                "complete_supported_quartets": int(
                    complete[types[:, 0] == index].sum()
                ),
            }
            for index, name in enumerate(("same_owner", "same_object"))
        },
    }


def _work_valid(audit: dict, rows: int, execution: str) -> bool:
    expected = {
        "rows": rows,
        "admitted_attention_pairs": rows * 5,
        "materialized_score_slots": rows * 5,
        "null_attention_pairs": rows,
        "future_score_slots_materialized": 0,
        "future_position_reads": 0,
        **{
            name: 0 if execution == "plain" else factor * rows
            for name, factor in TRANSFORM_FACTORS.items()
        },
    }
    return all(
        type(audit.get(name)) is int and audit[name] == value
        for name, value in expected.items()
    )


def _differences(left, right, masks: dict) -> dict:
    a_records, a_predictions, a_logits, a_attention = left
    b_records, b_predictions, b_logits, b_attention = right
    return {
        name: {
            "top1_changed": int((a_predictions[mask] != b_predictions[mask]).sum()),
            "logits_max_abs": float((a_logits[mask] - b_logits[mask]).abs().max()),
            "attention_max_abs": float(
                (a_attention[mask] - b_attention[mask]).abs().max()
            ),
            "nll_abs_difference": abs(
                a_records[name]["nll_nats"] - b_records[name]["nll_nats"]
            ),
        }
        for name, mask in masks.items()
    }


def _primary_decision(plain: dict, r4: dict, deltas: dict, groups_equal: bool) -> dict:
    limits = contract.EVALUATION
    rows = plain["records"]["all"]["decisions"]
    criteria = {
        "complete_strata": set(deltas) == {"all", "supported", "unknown"}
        and set(plain["records"]) == set(r4["records"]) == set(deltas),
        "exact_ordinary_reproduction": plain["historical_exact"],
        "complete_r4_decisions": r4["records"]["all"]["decisions"] == rows,
        "plain_causal_support": _work_valid(plain["audit"], rows, "plain"),
        "r4_complete_transport": _work_valid(r4["audit"], rows, "r4"),
        "binding_groups_preserved": groups_equal,
        "native_frame_coverage_preserved": plain["audit"]["reached_frame_indices"]
        == r4["audit"]["reached_frame_indices"],
        **{
            f"{name}_{metric}": delta[key] <= limit
            for name, delta in deltas.items()
            for metric, key, limit in (
                ("top1_exact", "top1_changed", 0),
                ("logit_tolerance", "logits_max_abs", limits["logit_atol"]),
                ("attention_tolerance", "attention_max_abs", limits["attention_atol"]),
                ("nll_tolerance", "nll_abs_difference", limits["nll_atol"]),
            )
        },
    }
    return {"passed": all(criteria.values()), "criteria": criteria}


def _control_decision(primary: dict, controlled: dict, preflight: dict) -> dict:
    rows = primary["plain"]["records"]["all"]["decisions"]
    work_fields = {
        "rows",
        "admitted_attention_pairs",
        "materialized_score_slots",
        "null_attention_pairs",
        "future_score_slots_materialized",
        "future_position_reads",
        *TRANSFORM_FACTORS,
    }
    audit, coherent = controlled["audit"], primary["r4"]["audit"]
    integrity = {
        "complete_transport": _work_valid(audit, rows, "source_frame_permuted"),
        "same_work": all(audit.get(key) == coherent.get(key) for key in work_fields),
        "all_four_fact_positions_shifted": audit["source_frame_positions_changed"]
        == rows * 4,
        "actual_frame_changes_match_preflight": audit["source_frame_matrices_changed"]
        == preflight["source_frame_matrices_changed"],
        "nontrivial_frame_changes": audit["source_frame_matrices_changed"] > 0,
    }
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
        "unknown_scope": "reported descriptively; unknown preservation is a primary criterion, not a broken-transport requirement",
    }


def _decision(reference_exact: bool, primary: dict, control: dict) -> dict:
    if not reference_exact:
        status, action = (
            "COMPOUND_R4_REFERENCE_MISMATCH",
            "STOP_AND_RESOLVE_ORDINARY_REPRODUCTION",
        )
    elif not primary["passed"]:
        status, action = (
            "COMPOUND_R4_PRESERVATION_MISS",
            "RETAIN_SOURCE_AND_RESOLVE_ADAPTER_PRESERVATION",
        )
    elif not control["valid"]:
        status, action = (
            "COMPOUND_R4_PRESERVED_CONTROL_INVALID",
            "RETAIN_PRESERVATION_AND_REPAIR_CONTROL_INTEGRITY",
        )
    elif not control["strong_transport_sensitivity"]:
        status, action = (
            "COMPOUND_R4_PRESERVED_CONTROL_WEAK",
            "RETAIN_PRESERVATION_AND_REVIEW_TRANSPORT_INTERVENTION",
        )
    else:
        status, action = (
            "COMPOUND_R4_PRESERVED",
            "RETAIN_R4_BINDING_AND_FREEZE_A_LANGUAGE_INTERFACE_STEP",
        )
    return {
        "status": status,
        "preserved": reference_exact and primary["passed"],
        "next_action": action,
    }


def _score_view(wrapper, tensors, budget) -> tuple[dict, tuple]:
    wrapper.reset_audit()
    scored = _score(wrapper, tensors, budget)
    records, predictions, _, _ = scored
    return {
        "records": records,
        "prediction_ids": predictions.reshape(-1).tolist(),
        "groups": _groups(predictions, tensors),
        "audit": copy.deepcopy(wrapper.audit),
    }, scored


def _conditional_controls(primary: dict, execute):
    if not primary["passed"]:
        return {
            "status": "NOT_RUN_PRIMARY_MISS",
            "model_decisions": 0,
            "views": [],
            "valid": False,
            "strong_transport_sensitivity": False,
        }
    return execute()


def _evaluate(root: Path, preparation: dict, budget: _Budget, phase: str) -> dict:
    frames = load_frames(Path(preparation["frames"]["root"]))
    model = load_model(preparation)
    before = _learned_state_cid(model)
    if before != preparation["source"]["model"]["state_cid"]:
        raise ValueError("loaded learned state differs")
    wrappers = {
        name: R4CompoundInference(model, frames, execution=name)
        for name in ("plain", "r4", "source_frame_permuted")
    }
    reference, primary_views = {}, []

    def progress(arm, population, rotation):
        atomic_write_json(
            root / f"{phase}-progress.json",
            {
                "phase": phase,
                "arm": arm,
                "population": population,
                "rotation": rotation,
                "elapsed_seconds": budget.elapsed,
            },
        )
        budget.check()

    # Reproduce ALL ordinary views before any coherent or control forward.
    ordinary = []
    for population, rotation, tensors in _views(preparation["source"]):
        record, scored = _score_view(wrappers["plain"], tensors, budget)
        record["historical_exact"] = record["records"] == _historical_records(
            preparation["source"], population, rotation
        )
        entry = {"population": population, "rotation": rotation, **record}
        ordinary.append(entry)
        reference[(population, rotation)] = (entry, scored)
        progress("plain", population, rotation)
    reference_exact = all(view["historical_exact"] for view in ordinary)
    if reference_exact:
        for population, rotation, tensors in _views(preparation["source"]):
            plain, scored_plain = reference[(population, rotation)]
            r4, scored_r4 = _score_view(wrappers["r4"], tensors, budget)
            supported = tensors["variant_ids"] < 4
            masks = {
                "all": torch.ones_like(supported, dtype=torch.bool),
                "supported": supported,
                "unknown": ~supported,
            }
            deltas = _differences(scored_plain, scored_r4, masks)
            primary_views.append(
                {
                    "population": population,
                    "rotation": rotation,
                    "plain": plain,
                    "r4": r4,
                    "differences": deltas,
                    **_primary_decision(
                        plain, r4, deltas, plain["groups"] == r4["groups"]
                    ),
                }
            )
            del scored_r4
            progress("r4", population, rotation)
    primary = {
        "passed": reference_exact
        and len(primary_views) == 8
        and all(v["passed"] for v in primary_views),
        "views": primary_views,
    }

    def controls():
        views = []
        for index, (population, rotation, tensors) in enumerate(
            _views(preparation["source"])
        ):
            record, scored = _score_view(
                wrappers["source_frame_permuted"], tensors, budget
            )
            plain_predictions = reference[(population, rotation)][1][1]
            supported = tensors["variant_ids"] < 4
            views.append(
                {
                    "population": population,
                    "rotation": rotation,
                    **record,
                    "changed_predictions": {
                        name: int((scored[1][mask] != plain_predictions[mask]).sum())
                        for name, mask in (
                            ("supported", supported),
                            ("unknown", ~supported),
                        )
                    },
                    **_control_decision(
                        primary_views[index],
                        record,
                        preparation["preflight"]["views"][index],
                    ),
                }
            )
            del scored
            progress("source_frame_permuted", population, rotation)
        return {
            "status": "SCORED_FIXED_ARTIFACT",
            "model_decisions": sum(v["records"]["all"]["decisions"] for v in views),
            "views": views,
            "valid": all(v["valid"] for v in views),
            "strong_transport_sensitivity": all(
                v["strong_transport_sensitivity"] for v in views
            ),
        }

    control = _conditional_controls(primary, controls)
    after = _learned_state_cid(model)
    if (
        before != after
        or model.lm_head.weight is not model.embedding.weight
        or any(m.training for m in model.modules())
        or any(p.requires_grad or p.grad is not None for p in model.parameters())
    ):
        raise ValueError("inference changed learned state, tying or evaluation mode")
    budget.check()
    return {
        **_decision(reference_exact, primary, control),
        "ordinary_exact_reproduction": reference_exact,
        "ordinary_views": ordinary,
        "primary": primary,
        "control": control,
        "learned_state_before": before,
        "learned_state_after": after,
        "parameter_count": model.parameter_count(),
        "model_file_cid": preparation["source"]["model"]["cid"],
        "preflight": preparation["preflight"],
        "optimizer_updates": 0,
        "new_parameters": 0,
        "checkpoint_optimizer_rng_payload_reads": 0,
        "model_label_arguments": 0,
        "new_population_generation": 0,
        "pre_1073_development_payload_reads": 0,
        "geometry_changes": 0,
        "native_geometry_exports": 0,
        "candidate_model_loads": 1,
        "scope": "preservation on #1073 observed construction/development; no new generalization, parsing, generation or geometry-advantage claim",
    }


def _phase(root: Path, replay: bool) -> dict:
    root = root.resolve()
    expected = _read_bound(root / "result.json", "result_cid") if replay else None
    runtime = _runtime()
    budget = _Budget(expected["elapsed_seconds"] if expected else 0.0)
    preparation = contract.validate_preparation(root)
    if runtime != preparation["source"]["runtime"]:
        raise ValueError("inference runtime must match #1073 ordinary reproduction")
    if expected:
        if (
            expected["schema"] != RESULT_SCHEMA
            or expected["issue"] != contract.ISSUE
            or expected["preparation_cid"] != preparation["preparation_cid"]
            or expected["source_result_cid"] != preparation["source"]["result_cid"]
            or expected["model"] != preparation["source"]["model"]
            or expected["frames"] != preparation["frames"]
            or expected["runtime"] != runtime
            or expected["process_id"] == os.getpid()
            or expected["implementation_cid"]
            != preparation["implementation"]["tree_cid"]
            or expected["evidence_cid"]
            != cid_bytes(canonical_json_bytes(expected["evidence"]))
        ):
            raise ValueError(
                "replay requires the bound result and a fresh matched process"
            )
        if expected["evidence"]["status"] == "INCOMPLETE_RESOURCE":
            raise ResourceBudgetExceeded(
                "incomplete inference cannot receive successful replay"
            )
    phase = "replay" if replay else "run"
    _write_exclusive(
        root / f"{phase}-started.json",
        {
            "preparation_cid": preparation["preparation_cid"],
            "process_id": os.getpid(),
            "carried_elapsed_seconds": budget.carried,
            "runtime": runtime,
        },
        "started_cid",
    )
    try:
        budget.check()
        evidence = _evaluate(root, preparation, budget, phase)
        if contract.validate_preparation(root) != preparation:
            raise ValueError("bound inputs or source changed during inference")
        budget.check()
    except ResourceBudgetExceeded as error:
        if replay:
            raise
        path = root / "run-progress.json"
        evidence = {
            "status": "INCOMPLETE_RESOURCE",
            "reason": str(error),
            "last_completed_progress": json.loads(path.read_text())
            if path.exists()
            else None,
            "optimizer_updates": 0,
        }
    evidence_cid = cid_bytes(canonical_json_bytes(evidence))
    if expected and canonical_json_bytes(evidence) != canonical_json_bytes(
        expected["evidence"]
    ):
        raise ValueError("fresh-process complete evidence differs")
    body = {
        "schema": REPLAY_SCHEMA if replay else RESULT_SCHEMA,
        "issue": contract.ISSUE,
        "preparation_cid": preparation["preparation_cid"],
        "implementation_cid": preparation["implementation"]["tree_cid"],
        "source_result_cid": preparation["source"]["result_cid"],
        "model": preparation["source"]["model"],
        "frames": preparation["frames"],
        "runtime": runtime,
        "process_id": os.getpid(),
        "evidence_cid": evidence_cid,
        "elapsed_seconds": budget.elapsed,
        "peak_rss_bytes": _peak_rss_bytes(),
    }
    if replay:
        body.update(
            result_cid=expected["result_cid"],
            exact_replay=True,
            fresh_process=True,
            optimizer_updates=0,
            combined_elapsed_seconds=budget.carried + budget.elapsed,
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
