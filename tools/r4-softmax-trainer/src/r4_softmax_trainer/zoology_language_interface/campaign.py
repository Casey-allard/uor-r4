"""One role-reader fit, conditional language evaluation, and exact replay (#1077)."""

from __future__ import annotations

import json
import os
import platform
import statistics
import time
from pathlib import Path

import torch
from safetensors.torch import load as load_safetensors
from torch.nn import functional as F

from ..provenance import atomic_write_json, canonical_json_bytes, cid_bytes
from ..zoology_compound_binding.campaign import (
    _record,
    _replacement_targets,
    _tensor_cid,
)
from ..zoology_compound_binding.model import load_model
from ..zoology_compound_r4.campaign import _groups
from ..zoology_r4_inference.campaign import (
    ResourceBudgetExceeded,
    _learned_state_cid,
    _peak_rss_bytes,
    _read_bound,
    _write_exclusive,
)
from ..zoology_release.development import (
    _canonical_safetensors,
    _configure_cpu,
    _tensor_mapping_cid,
)
from . import contract, data
from .model import LanguageInterfaceModel, LearnedRoleReader


class _Budget:
    def __init__(self, carried: float = 0.0) -> None:
        self.carried, self.started = carried, time.monotonic()

    @property
    def elapsed(self) -> float:
        return time.monotonic() - self.started

    def check(self) -> None:
        if self.elapsed + self.carried > contract.TRAINING["max_elapsed_seconds"]:
            raise ResourceBudgetExceeded(
                "language-interface cumulative clock exhausted"
            )
        if _peak_rss_bytes() > contract.TRAINING["max_rss_bytes"]:
            raise ResourceBudgetExceeded("language-interface exceeded 4 GiB")


def _runtime(threads: int) -> dict:
    _configure_cpu(threads)
    torch.use_deterministic_algorithms(True)
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


def _state(reader: LearnedRoleReader) -> str:
    return _tensor_mapping_cid(reader.state_dict())


def _role_loss(reader, inputs, lengths, positions):
    scores = reader.role_scores(inputs, lengths)
    return F.cross_entropy(scores.reshape(-1, scores.shape[-1]), positions.reshape(-1))


def _calibrate(budget: _Budget) -> dict:
    """Compare frozen CPU plans on synthetic role steps, never candidate fitting."""
    with torch.random.fork_rng():
        torch.manual_seed(10770)
        probe = LearnedRoleReader()
        inputs = torch.randint(0, 52, (128, 5, data.MAXLEN))
        lengths = torch.full((128, 5), data.MAXLEN, dtype=torch.long)
        positions = torch.randint(0, data.MAXLEN, (128, 5, 3))
        positions[:, 4, 2] = -100
        reference = None
        rows = []
        for threads in (4, 8):
            runtime = _runtime(threads)
            times = []
            for step in range(6):
                budget.check()
                probe.zero_grad(set_to_none=True)
                started = time.monotonic()
                loss = _role_loss(probe, inputs, lengths, positions)
                loss.backward()
                seconds = time.monotonic() - started
                if step >= 2:
                    times.append(seconds)
            gradient = torch.cat([p.grad.reshape(-1) for p in probe.parameters()])
            if reference is None:
                reference = (float(loss.detach()), gradient.clone())
            delta = max(
                abs(float(loss.detach()) - reference[0]),
                float((gradient - reference[1]).abs().max()),
            )
            if delta > 1e-6:
                raise ValueError("synthetic CPU role-step equivalence failed")
            rows.append(
                {
                    "runtime": runtime,
                    "median_seconds": statistics.median(times),
                    "timed_seconds": times,
                    "max_loss_gradient_difference": delta,
                }
            )
        chosen = min(
            rows, key=lambda row: (row["median_seconds"], row["runtime"]["threads"])
        )
        projected = chosen["median_seconds"] * contract.TRAINING["updates"] * 2 + 60
        if budget.elapsed + projected > contract.TRAINING["max_elapsed_seconds"]:
            raise ResourceBudgetExceeded("synthetic calibration does not admit the fit")
        return {
            "plans": rows,
            "selected_threads": chosen["runtime"]["threads"],
            "projected_fit_and_evaluation_seconds": projected,
            "synthetic_seed": 10770,
            "optimizer_updates": 0,
            "candidate_model_loads": 0,
        }


def fit(root: Path) -> dict:
    root = root.resolve()
    budget = _Budget()
    prep = contract.validate_preparation(root)
    _write_exclusive(
        root / "fit-started.json",
        {"preparation_cid": prep["preparation_cid"], "process_id": os.getpid()},
        "started_cid",
    )
    updates = 0
    calibration = None
    runtime = None
    try:
        calibration = _calibrate(budget)
        runtime = _runtime(calibration["selected_threads"])
        tensors = data.load_construction(root / "data")
        torch.manual_seed(contract.TRAINING["seed"])
        reader = LearnedRoleReader()
        initial = _state(reader)
        settings = contract.TRAINING
        optimizer = torch.optim.AdamW(
            reader.parameters(),
            lr=settings["learning_rate"],
            betas=tuple(settings["betas"]),
            eps=settings["eps"],
            weight_decay=settings["weight_decay"],
        )
        generator = torch.Generator().manual_seed(10771)
        order = torch.empty(0, dtype=torch.long)
        offset = 0
        early_seconds, blocks = [], []
        loss_sum = 0.0
        admission = None
        while updates < settings["updates"]:
            budget.check()
            if offset == len(order):
                order = torch.randperm(len(tensors["inputs"]), generator=generator)
                offset = 0
            indices = order[offset : offset + settings["batch_size"]]
            if len(indices) != settings["batch_size"]:
                raise ValueError("construction must have whole frozen batches")
            offset += len(indices)
            started = time.monotonic()
            optimizer.zero_grad(set_to_none=True)
            loss = _role_loss(
                reader,
                tensors["inputs"][indices],
                tensors["lengths"][indices],
                tensors["role_positions"][indices],
            )
            if not bool(torch.isfinite(loss)):
                raise ValueError("nonfinite role loss")
            loss.backward()
            torch.nn.utils.clip_grad_norm_(
                reader.parameters(), settings["clip_grad_norm"]
            )
            optimizer.step()
            updates += 1
            seconds = time.monotonic() - started
            loss_sum += float(loss.detach())
            if updates <= settings["admission_steps"]:
                early_seconds.append(seconds)
            if updates == settings["admission_steps"]:
                remaining = (
                    statistics.median(early_seconds)
                    * (settings["updates"] - updates)
                    * settings["admission_remaining_multiplier"]
                    + settings["evaluation_allowance_seconds"]
                )
                admission = {
                    "retained_steps": updates,
                    "median_step_seconds": statistics.median(early_seconds),
                    "projected_remaining_seconds": remaining,
                    "admitted": budget.elapsed + remaining
                    <= settings["max_elapsed_seconds"],
                }
                if not admission["admitted"]:
                    raise ResourceBudgetExceeded(
                        "retained initial steps do not admit the remaining fit"
                    )
            if updates % 64 == 0 or updates == settings["updates"]:
                blocks.append(
                    {"completed_updates": updates, "mean_role_loss": loss_sum / 64}
                )
                loss_sum = 0.0
                atomic_write_json(
                    root / "fit-progress.json",
                    {
                        "updates": updates,
                        "total_updates": settings["updates"],
                        "elapsed_seconds": budget.elapsed,
                        "blocks": blocks,
                    },
                )
        reader.eval().requires_grad_(False)
        payload = _canonical_safetensors(reader.state_dict())
        directory = root / "fit"
        directory.mkdir()
        with (directory / "reader.safetensors").open("xb") as handle:
            handle.write(payload)
        artifact = {
            "path": "fit/reader.safetensors",
            "bytes": len(payload),
            "cid": cid_bytes(payload),
            "state_cid": _state(reader),
            "parameter_count": reader.parameter_count(),
        }
        if contract.validate_preparation(root) != prep:
            raise ValueError("preparation changed during role fitting")
        budget.check()
        body = {
            "status": "FIT_COMPLETE",
            "artifact": artifact,
            "initial_state_cid": initial,
            "admission": admission,
            "blocks": blocks,
        }
    except ResourceBudgetExceeded as error:
        body = {"status": "INCOMPLETE_RESOURCE", "reason": str(error)}
    return _write_exclusive(
        root / "fit.json",
        {
            "schema": "uor-r4.language-interface-fit/1",
            "issue": 1077,
            "preparation_cid": prep["preparation_cid"],
            "implementation_cid": prep["implementation"]["tree_cid"],
            "core_file_cid": prep["source"]["model"]["cid"],
            "core_state_cid": prep["source"]["model"]["state_cid"],
            "optimizer_updates": updates,
            "row_presentations": updates * 128,
            "role_label_presentations": updates * 128 * 14,
            "core_optimizer_updates": 0,
            "core_model_loads": 0,
            "development_tensor_reads": 0,
            "calibration": calibration,
            "runtime": runtime,
            "elapsed_seconds": budget.elapsed,
            "peak_rss_bytes": _peak_rss_bytes(),
            **body,
        },
        "fit_cid",
    )


def _load_fit(root: Path, prep: dict) -> tuple[dict, LearnedRoleReader]:
    fitted = _read_bound(root / "fit.json", "fit_cid")
    if (
        fitted["status"] != "FIT_COMPLETE"
        or fitted["preparation_cid"] != prep["preparation_cid"]
        or fitted["implementation_cid"] != prep["implementation"]["tree_cid"]
        or fitted["optimizer_updates"] != 512
        or fitted["row_presentations"] != 65536
        or fitted["role_label_presentations"] != 917504
        or fitted["core_optimizer_updates"] != 0
        or fitted["development_tensor_reads"] != 0
        or fitted["core_file_cid"] != prep["source"]["model"]["cid"]
        or fitted["core_state_cid"] != prep["source"]["model"]["state_cid"]
    ):
        raise ValueError("fitted interface does not match the complete frozen dose")
    artifact = fitted["artifact"]
    payload = (root / artifact["path"]).read_bytes()
    if len(payload) != artifact["bytes"] or cid_bytes(payload) != artifact["cid"]:
        raise ValueError("fitted role-reader file differs")
    state = load_safetensors(payload)
    if _tensor_mapping_cid(state) != artifact["state_cid"]:
        raise ValueError("reader state CID differs")
    with torch.random.fork_rng():
        reader = LearnedRoleReader()
    reader.load_state_dict(state, strict=True)
    reader.eval().requires_grad_(False)
    if (
        reader.parameter_count() != 141571
        or artifact["parameter_count"] != 141571
        or _state(reader) != artifact["state_cid"]
    ):
        raise ValueError("reader parameter identity differs")
    return fitted, reader


def _view(tensors: dict, view_id: int) -> dict:
    mask = tensors["view_ids"] == view_id
    return {key: value[mask] for key, value in tensors.items()}


def _syntax_pairs(tensors: dict, predictions: torch.Tensor) -> dict:
    inputs = tensors["inputs"].reshape(-1, 5, *tensors["inputs"].shape[1:])
    lengths = tensors["lengths"].reshape(-1, 5, 5)
    targets = tensors["targets"].reshape(-1, 5)
    correct = (predictions == tensors["targets"]).reshape(-1, 5)
    same_object = tensors["pair_types"].reshape(-1, 5)[:, 0] == 1
    total, complete, same_bag = 0, 0, 0
    for first, second in ((0, 1), (2, 3)):
        left, right = inputs[same_object, first], inputs[same_object, second]
        if (
            not torch.equal(left[:, :4], right[:, :4])
            or not torch.equal(
                lengths[same_object, first], lengths[same_object, second]
            )
            or not bool(
                (targets[same_object, first] != targets[same_object, second]).all()
            )
        ):
            raise ValueError(
                "syntax pairs must keep identical facts and distinct answers"
            )
        bags = (left[:, 4].sort(-1).values == right[:, 4].sort(-1).values).all(-1)
        total += len(left)
        same_bag += int(bags.sum())
        complete += int(
            (correct[same_object, first] & correct[same_object, second]).sum()
        )
    if not total or same_bag != total:
        raise ValueError("syntax pair token bags differ")
    return {
        "pairs": total,
        "same_fact_and_query_bag_pairs": same_bag,
        "both_answers_correct": complete,
        "complete_rate": complete / total,
        "bag_only_complete_pair_ceiling": 0.0,
    }


@torch.inference_mode()
def _score(
    model: LanguageInterfaceModel,
    tensors: dict,
    budget: _Budget,
    *,
    control="none",
    oracle=False,
) -> tuple[dict, torch.Tensor, torch.Tensor, torch.Tensor]:
    output_rows, binding_rows, role_rows, oracle_rows = [], [], [], []
    for start in range(0, len(tensors["inputs"]), contract.EVALUATION["batch_size"]):
        budget.check()
        stop = start + contract.EVALUATION["batch_size"]
        out = model(
            tensors["inputs"][start:stop],
            tensors["lengths"][start:stop],
            control=control,
        )
        output_rows.append(out["logits"][:, None])
        binding_rows.append(out["binding_attention"][:, None, None])
        role_rows.append(out["role_attention"])
        if oracle:
            canon = tensors["canonical_inputs"][start:stop]
            ordinary = model.core.forward_selected(
                canon,
                torch.full((len(canon), 1), 37, dtype=torch.long),
                return_attention=False,
            )
            oracle_rows.append(ordinary.logits)
    logits, attention, roles = (
        torch.cat(output_rows),
        torch.cat(binding_rows),
        torch.cat(role_rows),
    )
    if (
        logits.shape != (len(tensors["inputs"]), 1, 4096)
        or not bool(torch.isfinite(logits).all())
        or not bool(torch.isfinite(roles).all())
        or not bool(torch.isfinite(attention).all())
    ):
        raise ValueError("incomplete or nonfinite interface outputs")
    pred = logits.argmax(-1)
    targets = tensors["targets"]
    losses = F.cross_entropy(
        logits.reshape(-1, 4096), targets.reshape(-1), reduction="none"
    )
    records = {"all": _record(logits, pred, attention, targets, losses)}
    for name, mask in (
        ("supported", tensors["variant_ids"] < 4),
        ("unknown", tensors["variant_ids"] == 4),
    ):
        records[name] = _record(
            logits[mask], pred[mask], attention[mask], targets[mask], losses[mask]
        )
    gold = tensors["role_positions"]
    valid = gold >= 0
    role_correct = (roles.argmax(-1) == gold) & valid
    by_role = {
        name: {
            "correct": int(role_correct[:, :, j].sum()),
            "decisions": int(valid[:, :, j].sum()),
            "rate": float(role_correct[:, :, j].sum()) / int(valid[:, :, j].sum()),
        }
        for j, name in enumerate(("owner", "object", "location"))
    }
    row = {
        "records": records,
        "prediction_ids": pred[:, 0].tolist(),
        "role_prediction_positions": roles.argmax(-1).tolist(),
        "role_attention_cid": _tensor_cid(roles),
        "role_accuracy": {
            "correct": int(role_correct.sum()),
            "decisions": int(valid.sum()),
            "rate": int(role_correct.sum()) / int(valid.sum()),
            "by_role": by_role,
        },
        "groups": _groups(pred, tensors),
        "syntax_pairs": _syntax_pairs(tensors, pred),
        "work": {
            "rows": len(pred),
            "role_decisions": int(valid.sum()),
            "role_scores_materialized": roles.numel(),
            "admitted_role_scores": int(tensors["lengths"].sum()) * 3,
            "binding_score_slots": len(pred) * 5,
            "null_pairs": len(pred),
            "future_input_reads": 0,
            "model_label_arguments": 0,
        },
    }
    if oracle:
        truth = torch.cat(oracle_rows)
        row["oracle"] = {
            "correct": int((truth.argmax(-1) == targets).sum()),
            "decisions": len(targets),
            "full_head_cid": _tensor_cid(truth),
            "predictions_cid": _tensor_cid(truth.argmax(-1)),
            "learned_max_logit_difference": float((truth - logits).abs().max()),
        }
        if row["oracle"]["correct"] != row["oracle"]["decisions"]:
            raise ValueError(
                "frozen source core no longer provides a perfect semantic oracle"
            )
    budget.check()
    return row, pred, attention, roles


def _qualified(row: dict) -> dict:
    limits = contract.EVALUATION
    criteria = {
        "supported": row["records"]["supported"]["top1_rate"]
        >= limits["supported_min_rate"],
        "unknown": row["records"]["unknown"]["top1_rate"] >= limits["unknown_min_rate"],
        "role_total": row["role_accuracy"]["rate"] >= limits["role_top1_min_rate"],
        "syntax_pairs": row["syntax_pairs"]["complete_rate"]
        >= limits["syntax_paired_complete_min_rate"],
        "complete_roles": row["work"]["role_decisions"] == row["work"]["rows"] * 14,
        "full_binding_support": row["work"]["binding_score_slots"]
        == row["work"]["rows"] * 5,
    }
    for role, score in row["role_accuracy"]["by_role"].items():
        criteria[role] = score["rate"] >= limits["each_role_top1_min_rate"]
    for family, score in row["groups"]["by_question_type"].items():
        criteria[family] = (
            score["complete_supported_quartets"] / score["groups"]
            >= limits["complete_quartet_min_rate"]
        )
    return {"passed": all(criteria.values()), "criteria": criteria}


def _control(
    primary, row, tensors, pred, attention, roles, primary_attention, primary_roles
) -> dict:
    replacement = _replacement_targets(
        {
            "inputs": tensors["canonical_inputs"],
            **{
                key: tensors[key]
                for key in ("targets", "variant_ids", "pair_types", "group_ids")
            },
        }
    )
    known = tensors["variant_ids"] < 4
    correct = int((pred[known] == replacement[known]).sum())
    total = int(known.sum())
    drop = (
        primary["records"]["supported"]["top1_rate"]
        - row["records"]["supported"]["top1_rate"]
    )
    result = {
        "replacement_supported_correct": correct,
        "supported_decisions": total,
        "replacement_rate": correct / total,
        "supported_drop": drop,
        "binding_attention_exact": torch.equal(attention, primary_attention),
        "role_attention_exact": torch.equal(roles, primary_roles),
        "same_work": row["work"] == primary["work"],
    }
    limits = contract.EVALUATION
    result["passed"] = (
        result["binding_attention_exact"]
        and result["role_attention_exact"]
        and result["same_work"]
        and correct / total >= limits["control_reassigned_min_rate"]
        and drop >= limits["control_supported_drop"]
        and row["records"]["unknown"]["top1_rate"] >= limits["control_unknown_min_rate"]
    )
    return result


def _decision(primary: bool, controls: bool, development: bool | None) -> dict:
    if not primary:
        return {
            "status": "LANGUAGE_INTERFACE_CONSTRUCTION_MISS",
            "passed": False,
            "next_action": "REPAIR_ROLE_READER_ON_CONSTRUCTION_KEEP_CORE_AND_DEVELOPMENT_CLOSED",
        }
    if not controls:
        return {
            "status": "LANGUAGE_INTERFACE_CONTROL_MISS",
            "passed": False,
            "next_action": "RETAIN_ROLE_PROGRESS_REPAIR_INTERFACE_BINDING_CONTROL",
        }
    if not development:
        return {
            "status": "LANGUAGE_INTERFACE_HELDOUT_MISS",
            "passed": False,
            "next_action": "RETAIN_CORE_AND_LEARNED_ROLE_PROGRESS_DIAGNOSE_SEEN_VS_UNSEEN_SYNTAX",
        }
    return {
        "status": "LANGUAGE_INTERFACE_HELDOUT_PASSED",
        "passed": True,
        "next_action": "RETAIN_LEARNED_INTERFACE_AND_FREEZE_UNCHANGED_R4_QUALIFICATION",
    }


def _evaluate(
    root: Path, prep: dict, fitted: dict, reader, budget: _Budget, phase: str
) -> dict:
    core = load_model({"source": prep["source"]})
    model = LanguageInterfaceModel(core, reader)
    model.eval().requires_grad_(False)
    before_core, before_reader = _learned_state_cid(core), _state(reader)
    construction = data.load_construction(root / "data")
    rows, retained, controls, devrows = [], [], [], []
    for view_id in contract.EVALUATION["construction_views"]:
        tensors = _view(construction, view_id)
        if len(tensors["inputs"]) != contract.EVALUATION["construction_rows"]:
            raise ValueError("construction view is incomplete")
        row, pred, attn, roles = _score(model, tensors, budget, oracle=True)
        row.update(view_id=view_id, qualification=_qualified(row))
        rows.append(row)
        retained.append((tensors, attn, roles))
        atomic_write_json(
            root / f"{phase}-progress.json",
            {
                "phase": "construction",
                "completed_view": view_id,
                "elapsed_seconds": budget.elapsed,
            },
        )
    primary = all(row["qualification"]["passed"] for row in rows)
    if primary:
        for original, (tensors, attn, roles) in zip(rows, retained, strict=True):
            row, pred, ca, cr = _score(model, tensors, budget, control="value_cycle")
            row.update(
                view_id=original["view_id"],
                control=_control(original, row, tensors, pred, ca, cr, attn, roles),
            )
            controls.append(row)
    control_pass = primary and all(row["control"]["passed"] for row in controls)
    if control_pass:
        development = data.load_development(root / "data")
        for view_id in (
            contract.EVALUATION["development_seen_views"]
            + contract.EVALUATION["development_heldout_views"]
        ):
            tensors = _view(development, view_id)
            if len(tensors["inputs"]) != contract.EVALUATION["development_rows"]:
                raise ValueError("development view is incomplete")
            row, _, _, _ = _score(model, tensors, budget, oracle=True)
            row.update(
                view_id=view_id,
                syntax="seen"
                if view_id in contract.EVALUATION["development_seen_views"]
                else "heldout",
                qualification=_qualified(row),
            )
            devrows.append(row)
            atomic_write_json(
                root / f"{phase}-progress.json",
                {
                    "phase": "development",
                    "completed_view": view_id,
                    "elapsed_seconds": budget.elapsed,
                },
            )
    devpassed = (
        all(row["qualification"]["passed"] for row in devrows) if control_pass else None
    )
    after_core, after_reader = _learned_state_cid(core), _state(reader)
    if (
        before_core != after_core
        or after_core != prep["source"]["model"]["state_cid"]
        or before_reader != after_reader
        or after_reader != fitted["artifact"]["state_cid"]
        or core.lm_head.weight is not core.embedding.weight
        or any(p.requires_grad or p.grad is not None for p in model.parameters())
        or any(m.training for m in model.modules())
    ):
        raise ValueError("evaluation mutated fixed core, reader, tying or mode")
    return {
        **_decision(primary, control_pass, devpassed),
        "construction": rows,
        "controls": controls if primary else {"status": "NOT_RUN_CONSTRUCTION_MISS"},
        "development": devrows
        if control_pass
        else {"status": "NOT_RUN_CONSTRUCTION_OR_CONTROL_MISS"},
        "development_tensor_reads": int(control_pass),
        "core_state_before": before_core,
        "core_state_after": after_core,
        "reader_state_before": before_reader,
        "reader_state_after": after_reader,
        "core_parameters": core.parameter_count(),
        "reader_parameters": reader.parameter_count(),
        "evaluation_optimizer_updates": 0,
        "core_optimizer_updates": 0,
        "geometry_changes": 0,
        "r4_forwards": 0,
        "scope": "supervised local owner disambiguation and role reading on known lexicon with supplied clause boundaries; observed source worlds; no general English or geometry-advantage claim",
    }


def _phase(root: Path, replay: bool) -> dict:
    root = root.resolve()
    expected = _read_bound(root / "result.json", "result_cid") if replay else None
    fitted_header = _read_bound(root / "fit.json", "fit_cid")
    budget = _Budget(
        fitted_header["elapsed_seconds"]
        + (expected["elapsed_seconds"] if expected else 0)
    )
    prep = contract.validate_preparation(root)
    fitted, reader = _load_fit(root, prep)
    runtime = _runtime(fitted["runtime"]["threads"])
    if runtime != fitted["runtime"]:
        raise ValueError("evaluation runtime differs from selected fit plan")
    if expected and (
        expected["schema"] != "uor-r4.language-interface-run/1"
        or expected["issue"] != contract.ISSUE
        or expected["reader"] != fitted["artifact"]
        or expected["core"] != prep["source"]["model"]
        or expected["preparation_cid"] != prep["preparation_cid"]
        or expected["fit_cid"] != fitted["fit_cid"]
        or expected["implementation_cid"] != prep["implementation"]["tree_cid"]
        or expected["process_id"] == os.getpid()
        or expected["runtime"] != runtime
        or expected["evidence_cid"]
        != cid_bytes(canonical_json_bytes(expected["evidence"]))
    ):
        raise ValueError("replay requires bound result and a fresh matched process")
    if expected and expected["evidence"]["status"] == "INCOMPLETE_RESOURCE":
        raise ResourceBudgetExceeded(
            "incomplete result cannot receive successful replay"
        )
    phase = "replay" if replay else "run"
    _write_exclusive(
        root / f"{phase}-started.json",
        {
            "preparation_cid": prep["preparation_cid"],
            "fit_cid": fitted["fit_cid"],
            "process_id": os.getpid(),
            "carried_elapsed_seconds": budget.carried,
        },
        "started_cid",
    )
    try:
        evidence = _evaluate(root, prep, fitted, reader, budget, phase)
        if contract.validate_preparation(root) != prep:
            raise ValueError("source/data/implementation changed during evaluation")
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
            "evaluation_optimizer_updates": 0,
        }
    evidence_cid = cid_bytes(canonical_json_bytes(evidence))
    if expected and canonical_json_bytes(evidence) != canonical_json_bytes(
        expected["evidence"]
    ):
        raise ValueError("fresh-process complete evidence differs")
    body = {
        "schema": f"uor-r4.language-interface-{phase}/1",
        "issue": 1077,
        "preparation_cid": prep["preparation_cid"],
        "implementation_cid": prep["implementation"]["tree_cid"],
        "fit_cid": fitted["fit_cid"],
        "reader": fitted["artifact"],
        "core": prep["source"]["model"],
        "runtime": runtime,
        "process_id": os.getpid(),
        "evidence_cid": evidence_cid,
        "elapsed_seconds": budget.elapsed,
        "cumulative_elapsed_seconds": budget.carried + budget.elapsed,
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
