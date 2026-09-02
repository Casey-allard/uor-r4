"""Fixed-weight, canonical-order R4 integration and fresh-process replay (#1059)."""

from __future__ import annotations

import copy
import json
import math
import os
import platform
import resource
import sys
import time
from collections.abc import Mapping
from pathlib import Path
from typing import Any

import torch
from blake3 import blake3
from safetensors import safe_open
from safetensors.torch import load as load_safetensors
from torch import Tensor
from torch.nn import functional as F

from ..provenance import atomic_write_json, canonical_json_bytes, cid_bytes
from ..zoology_control.model import ZoologyFigure2Config, ZoologyFigure2Model
from ..zoology_release.development import _tensor_mapping_cid
from .attention import R4ZoologyInference
from .contract import EVALUATION, validate_preparation
from .frames import load_frames

RESULT_SCHEMA = "uor-r4.zoology-r4-inference-result/1"
REPLAY_SCHEMA = "uor-r4.zoology-r4-inference-replay/1"
_WORK_FIELDS = (
    "admitted_attention_pairs",
    "materialized_score_slots",
    "future_score_slots_materialized",
    "query_blocks_encoded",
    "key_blocks_encoded",
    "value_blocks_encoded",
    "key_blocks_transported",
    "value_blocks_transported",
    "output_blocks_decoded",
    "source_frame_positions_changed",
    "source_frame_matrices_changed",
    "future_position_reads",
)


class ResourceBudgetExceeded(RuntimeError):
    """Resource interruption, never a scientific negative."""


def _peak_rss_bytes() -> int:
    amount = int(resource.getrusage(resource.RUSAGE_SELF).ru_maxrss)
    return amount if sys.platform == "darwin" else amount * 1024


class _Budget:
    def __init__(self, carried_seconds: float = 0.0) -> None:
        if not math.isfinite(carried_seconds) or carried_seconds < 0:
            raise ValueError("invalid carried inference time")
        self.began = time.monotonic()
        self.carried_seconds = carried_seconds

    @property
    def elapsed(self) -> float:
        return time.monotonic() - self.began

    def check(self) -> None:
        if self.carried_seconds + self.elapsed > EVALUATION["max_elapsed_seconds"]:
            raise ResourceBudgetExceeded("combined run and replay exceeded 900 seconds")
        if _peak_rss_bytes() > EVALUATION["max_rss_bytes"]:
            raise ResourceBudgetExceeded("inference process exceeded 4 GiB peak RSS")


def _configure_cpu() -> dict[str, Any]:
    os.environ["CUDA_VISIBLE_DEVICES"] = ""
    os.environ["PYTORCH_ENABLE_MPS_FALLBACK"] = "0"
    os.environ["OMP_NUM_THREADS"] = str(EVALUATION["threads"])
    os.environ["VECLIB_MAXIMUM_THREADS"] = str(EVALUATION["threads"])
    torch.set_num_threads(EVALUATION["threads"])
    if torch.get_num_interop_threads() != EVALUATION["interop_threads"]:
        torch.set_num_interop_threads(EVALUATION["interop_threads"])
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


def _read_bound(path: Path, cid_field: str) -> dict[str, Any]:
    value = json.loads(path.read_text(encoding="utf-8"))
    body = dict(value)
    expected = body.pop(cid_field, None)
    if expected != cid_bytes(canonical_json_bytes(body)):
        raise ValueError(f"{path.name} self identity differs")
    return value


def _write_exclusive(path: Path, body: Mapping[str, Any], field: str) -> dict[str, Any]:
    result = dict(body)
    result[field] = cid_bytes(canonical_json_bytes(body))
    with path.open("xb") as output:
        output.write(canonical_json_bytes(result))
        output.flush()
        os.fsync(output.fileno())
    return result


def _load_test_tensors(preparation: Mapping[str, Any]) -> dict[str, Tensor]:
    source = preparation["source"]
    path = Path(source["root"]) / source["dataset"]["path"]
    # Hashing is performed by validate_preparation. Only these three tensor
    # values are opened; the training tensors in this container are not loaded.
    with safe_open(path, framework="pt", device="cpu") as handle:
        tensors = {
            name: handle.get_tensor(name).contiguous()
            for name in ("test_inputs", "test_positions", "test_targets")
        }
    expected = {
        "test_inputs": (EVALUATION["rows"], 64),
        "test_positions": (EVALUATION["rows"], EVALUATION["queries_per_row"]),
        "test_targets": (EVALUATION["rows"], EVALUATION["queries_per_row"]),
    }
    if any(
        tensors[key].shape != shape or tensors[key].dtype != torch.long
        for key, shape in expected.items()
    ):
        raise ValueError("retained test tensor shape or dtype differs")
    return tensors


def _learned_state_cid(model: ZoologyFigure2Model) -> str:
    return _tensor_mapping_cid(
        {
            name: value
            for name, value in model.state_dict().items()
            if name != "lm_head.weight"
        }
    )


def _load_model(preparation: Mapping[str, Any]) -> ZoologyFigure2Model:
    source = preparation["source"]
    record = source["model"]
    payload = (Path(source["root"]) / record["path"]).read_bytes()
    if len(payload) != record["bytes"] or cid_bytes(payload) != record["cid"]:
        raise ValueError("source model file changed")
    state = load_safetensors(payload)
    if _tensor_mapping_cid(state) != record["state_cid"]:
        raise ValueError("source model tensor identity differs")
    model = ZoologyFigure2Model(ZoologyFigure2Config(**record["config"]))
    missing, unexpected = model.load_state_dict(state, strict=False)
    if missing != ["lm_head.weight"] or unexpected:
        raise ValueError("model must omit exactly the tied lm_head.weight")
    model.requires_grad_(False)
    model.eval()
    if _learned_state_cid(model) != record["state_cid"]:
        raise ValueError("loaded model tensors differ")
    return model


class _Scores:
    """Scoring consumes labels only after label-free model execution."""

    def __init__(self) -> None:
        self.correct = 0
        self.decisions = 0
        self.loss_sum = 0.0
        self.future_attention_nonzero = 0
        self.logits_digest = blake3()
        self.predictions_digest = blake3()
        self.attention_digest = blake3()
        self.audits: list[dict[str, Any]] = []

    def add(self, output: Any, targets: Tensor, audit: Mapping[str, Any]) -> Tensor:
        logits = output.logits.detach().float().contiguous()
        if logits.ndim != 3 or logits.shape[:-1] != targets.shape:
            raise ValueError("selected logits and scoring labels have different shapes")
        if targets.dtype != torch.long or not bool(torch.isfinite(logits).all()):
            raise ValueError("invalid scoring targets or nonfinite logits")
        attention = output.attention_weights
        if not attention:
            raise ValueError("attention weights required for integration comparison")
        predictions = logits.argmax(dim=-1)
        self.correct += int(torch.count_nonzero(predictions == targets))
        self.decisions += targets.numel()
        self.loss_sum += float(
            F.cross_entropy(
                logits.reshape(-1, logits.shape[-1]),
                targets.reshape(-1),
                reduction="sum",
            )
        )
        self.logits_digest.update(logits.numpy().tobytes(order="C"))
        self.predictions_digest.update(predictions.numpy().tobytes(order="C"))
        for weights in attention:
            if (
                weights.ndim != 4
                or weights.shape[-1] != weights.shape[-2]
                or not bool(torch.isfinite(weights).all())
            ):
                raise ValueError("invalid attention tensor")
            self.future_attention_nonzero += int(
                torch.count_nonzero(torch.triu(weights, diagonal=1))
            )
            self.attention_digest.update(
                weights.detach().contiguous().numpy().tobytes(order="C")
            )
        self.audits.append(copy.deepcopy(dict(audit)))
        return predictions

    def record(self) -> dict[str, Any]:
        totals = {}
        for name in _WORK_FIELDS:
            values = [audit.get(name) for audit in self.audits]
            totals[name] = (
                None if any(value is None for value in values) else sum(values)
            )
        return {
            "decisions": self.decisions,
            "top1_correct": self.correct,
            "top1_rate": self.correct / self.decisions if self.decisions else None,
            "nll_nats": self.loss_sum / self.decisions if self.decisions else None,
            "selected_logits_cid": f"blake3:{self.logits_digest.hexdigest()}",
            "predictions_cid": f"blake3:{self.predictions_digest.hexdigest()}",
            "attention_cid": f"blake3:{self.attention_digest.hexdigest()}",
            "future_attention_nonzero": self.future_attention_nonzero,
            "batches": len(self.audits),
            "audit_totals": totals,
            "audit_batches": self.audits,
            "reached_frame_indices": sorted(
                {
                    frame
                    for audit in self.audits
                    for frame in audit.get("reached_frame_indices", [])
                }
            ),
        }


def _maximum_difference(left: Any, right: Any) -> tuple[float, float]:
    if left.logits.shape != right.logits.shape:
        raise ValueError("matched logit shapes differ")
    left_attention, right_attention = left.attention_weights, right.attention_weights
    if (
        not left_attention
        or not right_attention
        or len(left_attention) != len(right_attention)
    ):
        raise ValueError("matched attention layers differ")
    if any(
        a.shape != b.shape for a, b in zip(left_attention, right_attention, strict=True)
    ):
        raise ValueError("matched attention shapes differ")
    return (
        float((left.logits.double() - right.logits.double()).abs().max()),
        max(
            float((a.double() - b.double()).abs().max())
            for a, b in zip(left_attention, right_attention, strict=True)
        ),
    )


def _primary_decision(
    plain: Mapping[str, Any],
    r4: Mapping[str, Any],
    differences: Mapping[str, Any],
    *,
    state_unchanged: bool,
) -> dict[str, Any]:
    expected = EVALUATION["rows"] * EVALUATION["queries_per_row"]
    criteria = {
        "complete_decisions": plain["decisions"] == r4["decisions"] == expected,
        "historical_correct_reproduced": plain["top1_correct"]
        == EVALUATION["historical_correct"],
        "identical_top1": differences["top1_changed"] == 0,
        "selected_logit_tolerance": differences["selected_logits_max_abs"]
        <= EVALUATION["logit_atol"],
        "attention_tolerance": differences["attention_max_abs"]
        <= EVALUATION["attention_atol"],
        "nll_tolerance": differences["nll_abs_difference"] <= EVALUATION["nll_atol"],
        "zero_future_attention_weight": plain["future_attention_nonzero"]
        == r4["future_attention_nonzero"]
        == 0,
        "causal_r4_source_reads": r4["audit_totals"]["future_position_reads"] == 0,
        "unchanged_learned_state": state_unchanged,
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
        "claim_boundary": "sensitivity to deliberately inconsistent transport; not H4 superiority",
    }


def _evaluate(
    root: Path, preparation: Mapping[str, Any], budget: _Budget, *, phase: str
) -> dict[str, Any]:
    tensors = _load_test_tensors(preparation)
    model = _load_model(preparation)
    expected_state = preparation["source"]["model"]["state_cid"]
    frames = load_frames(Path(preparation["frames"]["root"]))
    wrapper = R4ZoologyInference(model, frames)
    # Replacement nn.Modules default to training=True even when their parent
    # was already in eval mode. Reapply eval to the complete installed model.
    model.eval()
    model.requires_grad_(False)
    if any(module.training for module in model.modules()) or any(
        parameter.requires_grad for parameter in model.parameters()
    ):
        raise ValueError("inference adapter left training mode enabled")
    if model.lm_head.weight is not model.backbone.embeddings.word_embeddings.weight:
        raise ValueError("tied head identity changed")
    if _learned_state_cid(model) != expected_state:
        raise ValueError("adapter installation changed learned tensors")
    plain, r4 = _Scores(), _Scores()
    plain_logits: list[Tensor] = []
    max_logit = max_attention = 0.0
    changed = 0
    rows, batch_size = EVALUATION["rows"], EVALUATION["batch_size"]
    batches = math.ceil(rows / batch_size)

    def progress(arm: str, index: int, scores: Mapping[str, Any]) -> None:
        record = {
            "phase": phase,
            "arm": arm,
            "batch": index,
            "batches": batches,
            "elapsed_seconds": budget.elapsed,
            "scores": scores,
        }
        atomic_write_json(root / f"{phase}-progress.json", record)
        print(
            f"#1059 {phase} {arm} batch={index}/{batches} elapsed={budget.elapsed:.3f}s",
            flush=True,
        )

    with torch.inference_mode():
        for index, start in enumerate(range(0, rows, batch_size), 1):
            budget.check()
            inputs = tensors["test_inputs"][start : start + batch_size]
            positions = tensors["test_positions"][start : start + batch_size]
            targets = tensors["test_targets"][start : start + batch_size]
            reference = wrapper.forward_selected(
                inputs, positions, execution="plain", return_attention=True
            )
            reference_predictions = plain.add(reference, targets, wrapper.last_audit)
            plain_logits.append(reference.logits.detach().float().clone())
            budget.check()
            treatment = wrapper.forward_selected(
                inputs, positions, execution="r4", return_attention=True
            )
            treatment_predictions = r4.add(treatment, targets, wrapper.last_audit)
            logits_delta, attention_delta = _maximum_difference(reference, treatment)
            max_logit, max_attention = (
                max(max_logit, logits_delta),
                max(max_attention, attention_delta),
            )
            changed += int(
                torch.count_nonzero(reference_predictions != treatment_predictions)
            )
            del reference, treatment
            progress("plain+r4", index, {"plain": plain.record(), "r4": r4.record()})
            budget.check()

        plain_record, r4_record = plain.record(), r4.record()
        differences = {
            "selected_logits_max_abs": max_logit,
            "attention_max_abs": max_attention,
            "top1_changed": changed,
            "nll_abs_difference": abs(plain_record["nll_nats"] - r4_record["nll_nats"]),
        }
        state_unchanged = _learned_state_cid(model) == expected_state
        primary = {
            "plain": plain_record,
            "r4": r4_record,
            "differences": differences,
            **_primary_decision(
                plain_record, r4_record, differences, state_unchanged=state_unchanged
            ),
        }
        control: dict[str, Any] = {"status": "NOT_RUN_PRIMARY_MISS", "decisions": 0}
        if primary["passed"]:
            control_scores = _Scores()
            control_changed = 0
            control_logit_max = 0.0
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
                reference_logits = plain_logits[index - 1]
                control_changed += int(
                    torch.count_nonzero(reference_logits.argmax(dim=-1) != predictions)
                )
                control_logit_max = max(
                    control_logit_max,
                    float(
                        (reference_logits.double() - output.logits.double()).abs().max()
                    ),
                )
                del output
                progress("source_frame_permuted", index, control_scores.record())
                budget.check()
            record = control_scores.record()
            control = {
                "metrics": record,
                "top1_changed": control_changed,
                "selected_logits_max_abs": control_logit_max,
                **_control_decision(plain_record, r4_record, record),
            }

    if (
        _learned_state_cid(model) != expected_state
        or model.lm_head.weight is not model.backbone.embeddings.word_embeddings.weight
    ):
        raise ValueError("inference changed learned tensors or tied weights")
    if any(parameter.grad is not None for parameter in model.parameters()):
        raise ValueError("inference produced parameter gradients")
    retained_bytes = sum(value.numel() * value.element_size() for value in plain_logits)
    del plain_logits
    budget.check()
    return {
        "status": "R4_INTEGRATION_PRESERVED"
        if primary["passed"]
        else "R4_INTEGRATION_MISS",
        "primary": primary,
        "control": control,
        "learned_state_before": expected_state,
        "learned_state_after": _learned_state_cid(model),
        "tied_head_preserved": True,
        "optimizer_updates": 0,
        "training_tensor_values_loaded": 0,
        "model_label_arguments": 0,
        "reference_logits_retained_bytes": retained_bytes,
        "frame_coverage": {
            "vocabulary_entries": frames.token_leaf_indices.numel(),
            "direct_leaf_count": frames.direct_leaf_count,
            "native_witness_frame_count": frames.witness_frame_count,
            "inference_reached_frame_count": len(r4_record["reached_frame_indices"]),
            "inference_reached_frame_indices": r4_record["reached_frame_indices"],
            "frame_artifact_cid": frames.frame_artifact_cid,
            "token_map_artifact_cid": frames.artifact_cid,
        },
        "row_order": "canonical 0..2999; no shuffle or evaluation RNG",
        "evaluation": dict(EVALUATION),
    }


def run(root: Path) -> dict[str, Any]:
    """Run one frozen primary and its conditional control, without a fit."""
    root = root.resolve()
    budget = _Budget()
    preparation = validate_preparation(root)
    result_path = root / "result.json"
    if result_path.exists():
        result = _read_bound(result_path, "result_cid")
        if result.get("preparation_cid") != preparation["preparation_cid"]:
            raise ValueError("existing result belongs to another preparation")
        return result
    runtime = _configure_cpu()
    _write_exclusive(
        root / "run-started.json",
        {
            "issue": 1059,
            "preparation_cid": preparation["preparation_cid"],
            "process_id": os.getpid(),
            "runtime": runtime,
        },
        "started_cid",
    )
    try:
        budget.check()
        evidence = _evaluate(root, preparation, budget, phase="run")
        if validate_preparation(root) != preparation:
            raise ValueError(
                "source, frame or implementation bindings changed during inference"
            )
        budget.check()
    except ResourceBudgetExceeded as error:
        progress_path = root / "run-progress.json"
        evidence = {
            "status": "INCOMPLETE_RESOURCE",
            "reason": str(error),
            "last_completed_progress": json.loads(progress_path.read_text())
            if progress_path.exists()
            else None,
            "optimizer_updates": 0,
        }
    body = {
        "schema": RESULT_SCHEMA,
        "issue": 1059,
        "preparation_cid": preparation["preparation_cid"],
        "source": preparation["source"],
        "frames": preparation["frames"],
        "implementation": preparation["implementation"],
        "runtime": runtime,
        "process_id": os.getpid(),
        "evidence": evidence,
        "evidence_cid": cid_bytes(canonical_json_bytes(evidence)),
        "elapsed_seconds": budget.elapsed,
        "peak_rss_bytes": _peak_rss_bytes(),
    }
    result = _write_exclusive(result_path, body, "result_cid")
    print(
        f"#1059 result={result['result_cid']} status={evidence['status']}", flush=True
    )
    return result


def verify(root: Path) -> dict[str, Any]:
    """Replay the committed result in another process under the shared budget."""
    root = root.resolve()
    expected = _read_bound(root / "result.json", "result_cid")
    budget = _Budget(float(expected["elapsed_seconds"]))
    preparation = validate_preparation(root)
    if (
        expected.get("schema") != RESULT_SCHEMA
        or expected.get("preparation_cid") != preparation["preparation_cid"]
    ):
        raise ValueError("result and preparation differ")
    if expected.get("evidence_cid") != cid_bytes(
        canonical_json_bytes(expected["evidence"])
    ):
        raise ValueError("result evidence identity differs")
    if expected["evidence"]["status"] == "INCOMPLETE_RESOURCE":
        raise ResourceBudgetExceeded(
            "resource-interrupted result cannot receive a successful replay"
        )
    replay_path = root / "replay.json"
    if replay_path.exists():
        replay = _read_bound(replay_path, "replay_cid")
        if replay.get("result_cid") != expected["result_cid"]:
            raise ValueError("existing replay belongs to another result")
        return replay
    if expected["process_id"] == os.getpid():
        raise ValueError("verification requires a fresh process")
    runtime = _configure_cpu()
    if runtime != expected["runtime"]:
        raise ValueError("runtime differs from frozen inference result")
    budget.check()
    # A failed/interrupted replay cannot silently reset the shared allowance
    # on another invocation. Successful repeat verification returns above.
    _write_exclusive(
        root / "replay-started.json",
        {
            "issue": 1059,
            "result_cid": expected["result_cid"],
            "preparation_cid": preparation["preparation_cid"],
            "process_id": os.getpid(),
            "run_elapsed_seconds": budget.carried_seconds,
            "runtime": runtime,
        },
        "started_cid",
    )
    observed = _evaluate(root, preparation, budget, phase="replay")
    if canonical_json_bytes(observed) != canonical_json_bytes(expected["evidence"]):
        raise ValueError("fresh-process inference metrics, logits or audits differ")
    if validate_preparation(root) != preparation:
        raise ValueError("bindings changed during fresh-process replay")
    budget.check()
    replay = _write_exclusive(
        replay_path,
        {
            "schema": REPLAY_SCHEMA,
            "issue": 1059,
            "result_cid": expected["result_cid"],
            "preparation_cid": preparation["preparation_cid"],
            "evidence_cid": expected["evidence_cid"],
            "exact_replay": True,
            "fresh_process": True,
            "process_id": os.getpid(),
            "optimizer_updates": 0,
            "elapsed_seconds": budget.elapsed,
            "combined_elapsed_seconds": budget.carried_seconds + budget.elapsed,
            "peak_rss_bytes": _peak_rss_bytes(),
            "runtime": runtime,
        },
        "replay_cid",
    )
    print(f"#1059 replay={replay['replay_cid']} exact_replay=true", flush=True)
    return replay
