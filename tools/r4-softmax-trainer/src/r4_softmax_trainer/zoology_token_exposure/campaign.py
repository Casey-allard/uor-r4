"""One bounded reader/pooling diagnostic and one exact independent-process replay."""

from __future__ import annotations

import os
import time
from pathlib import Path

import torch

from ..provenance import atomic_write_json, canonical_json_bytes, cid_bytes
from ..zoology_compound_binding.campaign import _tensor_cid
from ..zoology_language_interface import campaign as ordinary
from ..zoology_language_interface import data
from ..zoology_language_r4 import contract as historical
from ..zoology_r4_inference.campaign import (
    ResourceBudgetExceeded,
    _learned_state_cid,
    _peak_rss_bytes,
    _write_exclusive,
)
from ..zoology_r4_inference.frames import load_frames
from . import contract
from .diagnostic import METRICS, ROLE_NAMES, measure_batch


class _Budget:
    def __init__(self, root):
        self.root, self.started = root, time.monotonic()

    @property
    def elapsed(self):
        return time.monotonic() - self.started

    def check(self):
        if self.elapsed > contract.POLICY["max_elapsed_seconds_per_phase"]:
            raise ResourceBudgetExceeded("diagnostic phase exceeded 120 seconds")
        if _peak_rss_bytes() > contract.POLICY["max_rss_bytes"]:
            raise ResourceBudgetExceeded("diagnostic exceeded 3 GiB RSS")
        if (
            sum(path.stat().st_size for path in self.root.rglob("*") if path.is_file())
            > contract.POLICY["max_disk_bytes"]
        ):
            raise ResourceBudgetExceeded("diagnostic output exceeded 256 MiB")


def _summaries(metrics, supported, changed):
    quantiles = torch.tensor(contract.POLICY["quantiles"], dtype=torch.float64)
    result = {}
    for population, mask in (("supported", supported), ("unknown", ~supported)):
        result[population] = {}
        for answer, selection in (
            ("all", mask),
            ("changed", mask & changed),
            ("retained", mask & ~changed),
        ):
            roles = {}
            for role, name in enumerate(ROLE_NAMES):
                measurements = {}
                for index, metric in enumerate(METRICS):
                    active = selection.clone()
                    if metric == "cancellation_retained_fraction":
                        active &= metrics[:, role, 2] > 0
                    values = metrics[active, role, index]
                    measurements[metric] = {
                        "count": len(values),
                        "mean": float(values.mean()) if len(values) else None,
                        "quantiles": torch.quantile(values, quantiles).tolist()
                        if len(values)
                        else None,
                    }
                roles[name] = measurements
            result[population][answer] = {"rows": int(selection.sum()), "roles": roles}
    return result


def _reject_downstream(_module, _args):
    raise ValueError("diagnostic attempted a forbidden downstream core/head forward")


def _evaluate(root, preparation, budget, replay):
    bound = preparation["historical"]
    frames = load_frames(Path(bound["frames"]["root"]))
    model = historical.load_source_model({"source": bound["source"]})
    before = {
        "core": _learned_state_cid(model.core),
        "reader": ordinary._state(model.reader),
    }
    tensors = data.load_construction(Path(bound["source"]["root"]) / "data")
    handles = [
        child.register_forward_pre_hook(_reject_downstream)
        for name, child in model.core.named_children()
        if name != "embedding"
    ]
    handles.append(model.register_forward_pre_hook(_reject_downstream))
    phase = "replay" if replay else "run"
    reports = []
    batches = 0
    try:
        for reference, declared in zip(
            bound["construction"], preparation["construction"], strict=True
        ):
            view_id = reference["view_id"]
            view = ordinary._view(tensors, view_id)
            if (
                _tensor_cid(view["inputs"]) != declared["input_cid"]
                or _tensor_cid(view["lengths"]) != declared["lengths_cid"]
            ):
                raise ValueError("selected construction row order changed")
            captured = {
                name: []
                for name in ("role_attention", "coherent", "controlled", "metrics")
            }
            changed_matrices, closure_error = 0, 0.0
            for start in range(0, len(view["inputs"]), contract.POLICY["batch_size"]):
                budget.check()
                stop = start + contract.POLICY["batch_size"]
                measured = measure_batch(
                    model,
                    view["inputs"][start:stop],
                    view["lengths"][start:stop],
                    frames,
                )
                for key in captured:
                    captured[key].append(measured[key])
                batches += 1
                changed_matrices += measured["changed_source_matrices"]
                closure_error = max(
                    closure_error, measured["max_f64_pool_closure_error"]
                )
                atomic_write_json(
                    root / f"{phase}-progress.json",
                    {
                        "view_id": view_id,
                        "completed_rows": view_id * 10240
                        + min(stop, len(view["inputs"])),
                        "total_rows": 20480,
                        "elapsed_seconds": budget.elapsed,
                    },
                )
            complete = {key: torch.cat(value) for key, value in captured.items()}
            exact = {
                "role_attention": _tensor_cid(complete["role_attention"])
                == reference["coherent"]["role_attention_cid"]
                == reference["controlled"]["role_attention_cid"],
                "coherent_role_vectors": _tensor_cid(complete["coherent"])
                == reference["coherent"]["role_vectors_cid"],
                "controlled_role_vectors": _tensor_cid(complete["controlled"])
                == reference["controlled"]["role_vectors_cid"],
                "changed_source_matrices": changed_matrices
                == declared["changed_source_matrices"],
            }
            if not all(exact.values()):
                raise ValueError(
                    f"construction view {view_id} differs from recorded #1079 evidence: {exact}"
                )
            coherent_ids = torch.tensor(reference["coherent"]["prediction_ids"])
            controlled_ids = torch.tensor(reference["controlled"]["prediction_ids"])
            changed = coherent_ids != controlled_ids
            supported = view["variant_ids"] < 4
            answer_counts = {
                name: {
                    "changed": int((changed & mask).sum()),
                    "retained": int((~changed & mask).sum()),
                }
                for name, mask in (("supported", supported), ("unknown", ~supported))
            }
            if any(
                answer_counts[name]["changed"]
                != reference["controlled"]["changed_predictions"][name]
                for name in answer_counts
            ):
                raise ValueError("recorded answer comparisons differ")
            metrics = complete["metrics"]
            payload = (
                metrics.contiguous()
                .numpy()
                .astype("<f8", copy=False)
                .tobytes(order="C")
            )
            path = root / f"construction-{view_id}.f64le"
            if replay:
                if path.read_bytes() != payload:
                    raise ValueError("fresh-process metric artifact differs")
            else:
                with path.open("xb") as handle:
                    handle.write(payload)
            reports.append(
                {
                    "view_id": view_id,
                    "input_binding": declared,
                    "historical_reproduction": exact,
                    "recorded_answer_comparisons": answer_counts,
                    "max_f64_pool_closure_error": closure_error,
                    "metric_artifact": {
                        "path": path.name,
                        "bytes": len(payload),
                        "cid": cid_bytes(payload),
                        "shape": list(metrics.shape),
                        "dtype": "little-endian f64",
                        "order": "C",
                    },
                    "zero_mass_role_rows": int((metrics[:, :, 0] == 0).sum()),
                    "zero_individual_displacement_role_rows": int(
                        (metrics[:, :, 2] == 0).sum()
                    ),
                    "summaries": _summaries(metrics, supported, changed),
                }
            )
            del captured, complete, metrics
            budget.check()
    finally:
        for handle in handles:
            handle.remove()
    after = {
        "core": _learned_state_cid(model.core),
        "reader": ordinary._state(model.reader),
    }
    if (
        before != after
        or after["core"] != bound["source"]["core"]["model"]["state_cid"]
        or after["reader"] != bound["source"]["reader"]["state_cid"]
        or model.core.lm_head.weight is not model.core.embedding.weight
        or any(module.training for module in model.modules())
        or any(
            parameter.requires_grad or parameter.grad is not None
            for parameter in model.parameters()
        )
    ):
        raise ValueError("frozen model state, tying or inference mode changed")
    return {
        "status": "TOKEN_EXPOSURE_DESCRIPTIVE_COMPLETE",
        "views": reports,
        "learned_state_before": before,
        "learned_state_after": after,
        "reader_forward_rows": 20480,
        "reader_forward_batches": batches,
        "measured_used_role_rows": 286720,
        "reconstructed_role_vectors_per_arm": 307200,
        "reference_helper_role_vectors_per_arm": 307200,
        "total_pooled_role_vector_evaluations": 1228800,
        "new_head_forwards": 0,
        "new_answer_predictions": 0,
        "development_tensor_reads": 0,
        "optimizer_updates": 0,
        "new_parameters": 0,
        "new_population_generation": 0,
        "new_controls": 0,
        "geometry_changes": 0,
        "native_exports": 0,
        "generation": 0,
        "interpretation": "descriptive association with recorded answers; no causal attribution or successor promotion",
        "next_action": "independently review these distributions and freeze a separate successor; retain #1079 preservation and its weak-control terminal",
    }


def _phase(root: Path, replay: bool):
    root = root.resolve()
    phase = "replay" if replay else "run"
    budget = _Budget(root)
    _write_exclusive(
        root / f"{phase}-started.json",
        {"issue": contract.ISSUE, "process_id": os.getpid()},
        "started_cid",
    )
    try:
        preparation = contract.validate_preparation(root)
        runtime = ordinary._runtime(contract.POLICY["threads"])
        if runtime != preparation["historical"]["source"]["runtime"]:
            raise ValueError("runtime differs from frozen #1079")
        expected = (
            historical.prior._envelope(root / "result.json", "result_cid")
            if replay
            else None
        )
        if replay and (
            expected["process_id"] == os.getpid()
            or expected["preparation_cid"] != preparation["preparation_cid"]
            or expected["evidence"]["status"] != "TOKEN_EXPOSURE_DESCRIPTIVE_COMPLETE"
            or expected["evidence_cid"]
            != cid_bytes(canonical_json_bytes(expected["evidence"]))
        ):
            raise ValueError(
                "replay requires a complete bound result and fresh process"
            )
        budget.check()
        evidence = _evaluate(root, preparation, budget, replay)
        if contract.validate_preparation(root) != preparation:
            raise ValueError("source or implementation changed during diagnostic")
        budget.check()
        if replay and canonical_json_bytes(evidence) != canonical_json_bytes(
            expected["evidence"]
        ):
            raise ValueError("fresh-process complete diagnostic evidence differs")
    except (ValueError, OSError, ResourceBudgetExceeded) as error:
        return _write_exclusive(
            root / f"{phase}-refusal.json",
            {
                "issue": contract.ISSUE,
                "status": "INCOMPLETE_RESOURCE"
                if isinstance(error, ResourceBudgetExceeded)
                else "UNAVAILABLE_BINDING_OR_DIAGNOSTIC",
                "reason": str(error),
                "elapsed_seconds": budget.elapsed,
                "process_id": os.getpid(),
                "diagnosis_permitted": False,
            },
            "refusal_cid",
        )
    result = {
        "schema": f"uor-r4.token-exposure-{phase}/1",
        "issue": contract.ISSUE,
        "preparation_cid": preparation["preparation_cid"],
        "implementation_cid": preparation["implementation"]["tree_cid"],
        "runtime": runtime,
        "process_id": os.getpid(),
        "elapsed_seconds": budget.elapsed,
        "peak_rss_bytes": _peak_rss_bytes(),
        "evidence_cid": cid_bytes(canonical_json_bytes(evidence)),
    }
    if replay:
        result.update(
            result_cid=expected["result_cid"], exact_replay=True, fresh_process=True
        )
    else:
        result["evidence"] = evidence
    return _write_exclusive(
        root / ("replay.json" if replay else "result.json"),
        result,
        "replay_cid" if replay else "result_cid",
    )


def run(root: Path):
    return _phase(root, False)


def verify(root: Path):
    return _phase(root, True)
