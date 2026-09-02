"""Create-once CPU lifecycle for the credited Zoology MQAR control (#1049).

Source-derived mechanics are attributed in :mod:`.provenance` and NOTICE.md.
This runner never opens a fitted #1045 artifact or a sealed population.  It
uses the exact open #1045 row loader, projects only selected query positions,
and stops at the first frozen control miss.
"""

from __future__ import annotations

import json
import math
import multiprocessing as mp
import os
import platform
import queue as queue_module
import resource
import time
import traceback
from collections.abc import Mapping, Sequence
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Any, Literal

import torch
from blake3 import blake3
from safetensors.torch import load as load_safetensors
from safetensors.torch import save as save_safetensors
from torch import Tensor
from torch.nn import functional as F

from ..provenance import canonical_json_bytes, cid_bytes
from . import data as zoology_data
from .data import (
    ZoologyMQARBatch,
    ZoologyMQARPopulation,
    ZoologyMQARRow,
    batch_rows,
    build_source_calibration,
    deterministic_epoch_order,
    load_exact_1045_population,
    permute_exact_bindings,
)
from .model import (
    ZoologyFigure2Config,
    ZoologyFigure2Model,
    set_zoology_seed,
)
from .provenance import (
    zoology_control_implementation_contract,
    zoology_source_attribution,
)


ISSUE = 1049
POLICY = "ZoologyMQARControlV2MeasuredWall"
SOURCE_MODEL_SEED = 123

SOURCE_TRAIN_ROWS = 8_192
SOURCE_DEVELOPMENT_ROWS = 1_024
EXACT_TRAIN_ROWS = 8_192
EXACT_DEVELOPMENT_ROWS = 1_024
OVERFIT_ROWS = 32
OVERFIT_UPDATES = 256
MAXIMUM_EPOCHS = 64
BATCH_SIZE = 64

LEARNING_RATE = 0.0004641588833612782
WEIGHT_DECAY = 0.1
TRAIN_REQUIRED_RATE = 0.995
DEVELOPMENT_REQUIRED_RATE = 0.99
CONTROL_REQUIRED_DROP = 0.50
REQUIRED_CONSECUTIVE_PASSES = 2
MAXIMUM_C2_QUERY_PRESENTATIONS = 4_194_304

ELIGIBLE_THREADS = (1, 4, 8)
TIMED_TRAINING_BATCHES = 32
PROJECTION_SAFETY_FACTOR = 1.25
HARD_WALL_SECONDS = 1_200.0
MEMORY_CEILING_BYTES = 8 * 1024**3
PROBE_TIMEOUT_SECONDS = 300.0

EXPECTED_1045_RESULT_CID = (
    "blake3:d920ad7b7f373c55cb564e27b3ddb1af8949a20c432e0d7cd2b39f1f69999557"
)
EXPECTED_1045_SPLIT_CID = (
    "blake3:d36937f974e5e96dc697b219db8a7eb448dff7192abdf88bf6b21000f58b1f48"
)
EXPECTED_1045_DIAGNOSTIC_CID = (
    "blake3:ce11698f62561afb6d8ee5e8f816474df389802559d5e6519bff498c735b7736"
)
FORBIDDEN_1045_ARTIFACT_CID = (
    "blake3:92bb13caf71c9ef44885a9da39023d080de075118b5902b716d2ca9b0f61f611"
)
EXPECTED_1047_MERGE_COMMIT = "677fb133b6d6a01fe384450b66beabbbd1b8f9a5"
EXPECTED_1047_IMPLEMENTATION_TREE_CID = (
    "blake3:c848c05ae53bc3adc0a8f7099ceed43657b6348e4e00fe3aaef5cf1368cc38de"
)
EXPECTED_1047_PREFLIGHT_CID = (
    "blake3:78158700e632d303bf674ed544f997a0e14eb89947470f5032e6acc75c830c9b"
)
EXPECTED_1047_RESULT_CID = (
    "blake3:b453abccc6ae0db9cc186c791aba268555dc0e75fe687c994e940254b0ac9ef6"
)

PREPARATION_RELATIVE_PATH = "zoology-control-preparation.json"
PREFLIGHT_STARTED_RELATIVE_PATH = "preflight/zoology-control-started.json"
PREFLIGHT_RELATIVE_PATH = "preflight/zoology-control-preflight.json"
RUN_STARTED_RELATIVE_PATH = "run/zoology-control-started.json"
RESULT_RELATIVE_PATH = "run/zoology-control-result.json"
C1_ARTIFACT_RELATIVE_PATH = "artifact/c1-source-calibration.safetensors"
C2_ARTIFACT_RELATIVE_PATH = "artifact/c2-exact-1045.safetensors"
PREDECESSOR_PREFLIGHT_RELATIVE_PATH = (
    "preflight/role-tagged-associative-preflight.json"
)
PREDECESSOR_RESULT_RELATIVE_PATH = "run/role-tagged-associative-result.json"

PREPARATION_SCHEMA = "uor-r4.zoology-control-preparation/1"
PREFLIGHT_SCHEMA = "uor-r4.zoology-control-preflight/1"
STARTED_SCHEMA = "uor-r4.zoology-control-started/1"
RESULT_SCHEMA = "uor-r4.zoology-control-result/1"

_SOURCE_GOLDEN_INPUTS = (
    (2, 29, 7, 22, 2, 0, 7, 0, 0, 0, 0, 0, 0, 0, 0, 0),
    (3, 21, 5, 16, 3, 0, 5, 0, 0, 0, 0, 0, 0, 0, 0, 0),
    (5, 17, 13, 18, 13, 0, 0, 0, 5, 0, 0, 0, 0, 0, 0, 0),
)
_SOURCE_GOLDEN_LABELS = (
    (-100, -100, -100, -100, 29, -100, 22, -100, -100, -100, -100, -100, -100, -100, -100, -100),
    (-100, -100, -100, -100, 21, -100, 16, -100, -100, -100, -100, -100, -100, -100, -100, -100),
    (-100, -100, -100, -100, 18, -100, -100, -100, 17, -100, -100, -100, -100, -100, -100, -100),
)
_SOURCE_GOLDEN_WORD = (
    0.01682600937783718,
    0.007630460895597935,
    0.03871878609061241,
    0.004360933322459459,
)
_SOURCE_GOLDEN_POSITION = (
    0.002722495701164007,
    -0.0014288985403254628,
    -0.004648478236049414,
    -0.01643170788884163,
)
_SOURCE_GOLDEN_WQKV = (
    0.03611164167523384,
    -0.02012898586690426,
    0.003164451103657484,
    0.00013401817705016583,
)
_SOURCE_GOLDEN_LOGITS = (
    -0.03831605240702629,
    -0.05481904745101929,
    -0.031969670206308365,
    0.005645290017127991,
    0.10721376538276672,
    -0.07025852799415588,
)

Verdict = Literal[
    "INVALID_CONTROL_PORT",
    "SCALED_SOURCE_CALIBRATION_MISS",
    "STOCK_CELL_EXACT_QUALIFICATION_MISS",
    "STOCK_CELL_TRANSFER_MISS",
    "NONASSOCIATIVE_SHORTCUT",
    "STOCK_CELL_PASSES_EXACT_BYTES",
]


@dataclass(frozen=True, slots=True)
class ExecutionPlan:
    """One frozen CPU-only batch-64 plan."""

    threads: int
    batch_size: int = BATCH_SIZE

    def __post_init__(self) -> None:
        if self.threads not in ELIGIBLE_THREADS:
            raise ValueError("threads are outside the frozen #1047 plans")
        if self.batch_size != BATCH_SIZE:
            raise ValueError("#1047 admits only batch 64")

    def record(self) -> dict[str, Any]:
        body: dict[str, Any] = {
            "name": f"cpu-{self.threads}t-b{self.batch_size}",
            "device": "cpu",
            "threads": self.threads,
            "workers": 1,
            "batch_size": self.batch_size,
            "cuda": "FORBIDDEN",
            "mps": "FORBIDDEN",
        }
        body["plan_cid"] = cid_bytes(canonical_json_bytes(body))
        return body


@dataclass(frozen=True, slots=True)
class ScoreResult:
    """One query-only population score and its causal/work ledger."""

    decisions: int
    correct: int
    loss_sum: float
    selected_logits_cid: str
    work: dict[str, int]

    @property
    def rate(self) -> float:
        return self.correct / self.decisions

    @property
    def nll_nats(self) -> float:
        return self.loss_sum / self.decisions

    def record(self) -> dict[str, Any]:
        return {
            "decisions": self.decisions,
            "top1_correct": self.correct,
            "top1_rate": self.rate,
            "nll_nats": self.nll_nats,
            "selected_logits_cid": self.selected_logits_cid,
            "work": dict(self.work),
        }


class HardWallExceeded(TimeoutError):
    """The combined C1+C2 execution reached its frozen resource wall."""

    def __init__(self, message: str, partial: Mapping[str, Any] | None = None) -> None:
        super().__init__(message)
        self.partial = None if partial is None else dict(partial)


def _with_cid(value: Mapping[str, Any], field: str) -> dict[str, Any]:
    if field in value:
        raise ValueError(f"self-CID field already exists: {field}")
    result = dict(value)
    result[field] = cid_bytes(canonical_json_bytes(value))
    return result


def _verify_self_cid(value: Mapping[str, Any], field: str) -> None:
    unsigned = dict(value)
    observed = unsigned.pop(field, None)
    if observed != cid_bytes(canonical_json_bytes(unsigned)):
        raise ValueError(f"{field} does not reproduce")


def _read_json(path: Path, *, cid_field: str) -> dict[str, Any]:
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, UnicodeError, json.JSONDecodeError) as error:
        raise ValueError(f"cannot read {path.name}: {error}") from error
    if not isinstance(value, dict):
        raise ValueError(f"{path.name} is not a JSON object")
    _verify_self_cid(value, cid_field)
    return value


def _read_bound_json(
    path: Path,
    *,
    cid_field: str,
    relative_path: str,
) -> tuple[dict[str, Any], dict[str, Any]]:
    try:
        payload = path.read_bytes()
        value = json.loads(payload)
    except (OSError, UnicodeError, json.JSONDecodeError) as error:
        raise ValueError(f"cannot bind {relative_path}: {error}") from error
    if not isinstance(value, dict):
        raise ValueError(f"{relative_path} is not a JSON object")
    _verify_self_cid(value, cid_field)
    return value, {
        "path": relative_path,
        "source_path": str(path),
        "bytes": len(payload),
        "file_cid": cid_bytes(payload),
        cid_field: value[cid_field],
    }


def _write_exclusive(path: Path, payload: bytes) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    descriptor = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o644)
    try:
        with os.fdopen(descriptor, "wb") as output:
            output.write(payload)
            output.flush()
            os.fsync(output.fileno())
    except BaseException:
        try:
            path.unlink()
        except OSError:
            pass
        raise


def _write_exclusive_json(path: Path, value: Mapping[str, Any]) -> None:
    _write_exclusive(path, canonical_json_bytes(value))


def _peak_rss_bytes() -> int:
    peak = int(resource.getrusage(resource.RUSAGE_SELF).ru_maxrss)
    return peak if platform.system() == "Darwin" else peak * 1024


def _row_query_count(rows: Sequence[ZoologyMQARRow]) -> int:
    return sum(len(row.selected_positions) for row in rows)


def _population_record(population: ZoologyMQARPopulation) -> dict[str, Any]:
    role_rows = sum(
        row.role_ids is not None
        for row in (*population.train, *population.development)
    )
    return {
        "name": population.name,
        "population_cid": population.population_cid,
        "vocab_size": population.vocab_size,
        "input_seq_len": population.input_seq_len,
        "num_kv_pairs": population.num_kv_pairs,
        "train_seed": population.train_seed,
        "development_seed": population.development_seed,
        "source_split_cid": population.source_split_cid,
        "train_rows": len(population.train),
        "development_rows": len(population.development),
        "train_queries": _row_query_count(population.train),
        "development_queries": _row_query_count(population.development),
        "role_sidecar_rows": role_rows,
    }


def _load_bound_populations(
    source_root: Path,
) -> tuple[ZoologyMQARPopulation, ZoologyMQARPopulation]:
    source = build_source_calibration()
    exact = load_exact_1045_population(source_root)
    if (
        len(source.train) != SOURCE_TRAIN_ROWS
        or len(source.development) != SOURCE_DEVELOPMENT_ROWS
        or source.vocab_size != 8_192
        or source.input_seq_len != 64
        or source.num_kv_pairs != 4
        or source.train_seed != 0
        or source.development_seed != 10
    ):
        raise ValueError("source-native C1 population differs from the freeze")
    if (
        len(exact.train) != EXACT_TRAIN_ROWS
        or len(exact.development) != EXACT_DEVELOPMENT_ROWS
        or exact.vocab_size != 4_096
        or exact.input_seq_len != 120
        or exact.num_kv_pairs != 8
        or exact.source_split_cid != EXPECTED_1045_SPLIT_CID
    ):
        raise ValueError("exact #1045 C2 population differs from the freeze")
    if any(row.role_ids is not None for row in (*source.train, *source.development)):
        raise ValueError("source-native population unexpectedly carries roles")
    if any(row.role_ids is None for row in (*exact.train, *exact.development)):
        raise ValueError("exact #1045 population lacks provenance role bytes")
    return source, exact


def _bind_predecessor(predecessor_root: Path) -> dict[str, Any]:
    """Bind only #1045's immutable open preflight/result JSON envelopes."""

    preflight, preflight_file = _read_bound_json(
        predecessor_root / PREDECESSOR_PREFLIGHT_RELATIVE_PATH,
        cid_field="preflight_cid",
        relative_path=PREDECESSOR_PREFLIGHT_RELATIVE_PATH,
    )
    result, result_file = _read_bound_json(
        predecessor_root / PREDECESSOR_RESULT_RELATIVE_PATH,
        cid_field="result_cid",
        relative_path=PREDECESSOR_RESULT_RELATIVE_PATH,
    )
    decision = result.get("decision")
    if (
        result.get("result_cid") != EXPECTED_1045_RESULT_CID
        or result.get("preflight_cid") != preflight.get("preflight_cid")
        or preflight.get("source_split_cid") != EXPECTED_1045_SPLIT_CID
        or not isinstance(decision, Mapping)
        or decision.get("verdict") != "OPEN_MQAR_NOT_LEARNED"
    ):
        raise ValueError("#1045 predecessor result/preflight differs from the freeze")
    return {
        "root": str(predecessor_root),
        "preflight": preflight_file,
        "result": result_file,
        "source_split_cid": preflight["source_split_cid"],
        "result_cid": result["result_cid"],
        "verdict": decision["verdict"],
        "artifact_access": "FORBIDDEN_NOT_READ",
        "sealed_access": "FORBIDDEN_NOT_READ",
    }


def _capacity_predecessor_record() -> dict[str, Any]:
    """Bind #1047 by its protected merge and immutable create-once identities."""

    return {
        "issue": 1047,
        "merge_commit": EXPECTED_1047_MERGE_COMMIT,
        "implementation_tree_cid": EXPECTED_1047_IMPLEMENTATION_TREE_CID,
        "preflight_cid": EXPECTED_1047_PREFLIGHT_CID,
        "result_cid": EXPECTED_1047_RESULT_CID,
        "status": "NOT_RUN_PREFLIGHT",
        "fastest_plan": "cpu-8t-b64",
        "projected_seconds": 959.2125811270671,
        "hard_wall_seconds": 900.0,
        "scientific_result": "NOT_AVAILABLE",
    }


def prepare_zoology_control(
    root: Path,
    *,
    source_root: Path,
    predecessor_root: Path,
) -> dict[str, Any]:
    """Bind a new run root to source authority and the two open populations."""

    root = root.resolve()
    source_root = source_root.resolve()
    predecessor_root = predecessor_root.resolve()
    if not source_root.is_dir():
        raise FileNotFoundError("#1045 open source root is absent")
    if not predecessor_root.is_dir():
        raise FileNotFoundError("#1045 predecessor run root is absent")
    if len({root, source_root, predecessor_root}) != 3:
        raise ValueError("run, source, and predecessor roots must differ")
    path = root / PREPARATION_RELATIVE_PATH
    source_population, exact_population = _load_bound_populations(source_root)
    predecessor = _bind_predecessor(predecessor_root)
    capacity_predecessor = _capacity_predecessor_record()
    implementation = zoology_control_implementation_contract()
    attribution = zoology_source_attribution()
    if path.exists():
        preparation = _read_json(path, cid_field="preparation_cid")
        if (
            preparation.get("source_root") != str(source_root)
            or preparation.get("predecessor_root") != str(predecessor_root)
            or preparation.get("predecessor") != predecessor
            or preparation.get("capacity_predecessor") != capacity_predecessor
            or preparation.get("implementation") != implementation
            or preparation.get("source_attribution") != attribution
            or preparation.get("populations")
            != {
                "c1_source_native": _population_record(source_population),
                "c2_exact_1045": _population_record(exact_population),
            }
        ):
            raise ValueError("cached #1047 preparation no longer reproduces")
        return preparation
    if root.exists() and any(root.iterdir()):
        raise FileExistsError("#1047 preparation requires an empty run root")

    body = {
        "schema": PREPARATION_SCHEMA,
        "issue": ISSUE,
        "policy": POLICY,
        "source_root": str(source_root),
        "predecessor_root": str(predecessor_root),
        "predecessor": predecessor,
        "capacity_predecessor": capacity_predecessor,
        "implementation": implementation,
        "source_attribution": attribution,
        "inputs": {
            "source_attribution_cid": attribution["attribution_cid"],
            "implementation_cid": implementation["implementation_cid"],
            "implementation_tree_cid": implementation["tree_cid"],
            "source_native_population_cid": source_population.population_cid,
            "exact_1045_population_cid": exact_population.population_cid,
            "source_1045_split_cid": exact_population.source_split_cid,
            "source_1045_result_cid": EXPECTED_1045_RESULT_CID,
            "source_1045_diagnostic_cid": EXPECTED_1045_DIAGNOSTIC_CID,
            "source_1045_preflight_file_cid": predecessor["preflight"][
                "file_cid"
            ],
            "source_1045_result_file_cid": predecessor["result"]["file_cid"],
            "source_1047_implementation_tree_cid": (
                capacity_predecessor["implementation_tree_cid"]
            ),
            "source_1047_preflight_cid": capacity_predecessor["preflight_cid"],
            "source_1047_result_cid": capacity_predecessor["result_cid"],
        },
        "populations": {
            "c1_source_native": _population_record(source_population),
            "c2_exact_1045": _population_record(exact_population),
        },
        "forbidden_inputs": {
            "source_1045_fitted_artifact_cid": FORBIDDEN_1045_ARTIFACT_CID,
            "sealed_payloads": "FORBIDDEN",
            "roles_as_model_input": "FORBIDDEN",
            "h4_frames_as_model_input": "FORBIDDEN",
            "teacher_provider_ollama_gemma": "FORBIDDEN",
        },
        "read_ledger": {
            "allowed_predecessor_json_reads": 2,
            "failed_source_artifact_reads": 0,
            "sealed_input_reads": 0,
            "provider_calls": 0,
            "teacher_calls": 0,
            "future_value_reads": 0,
            "role_model_input_reads": 0,
            "h4_model_input_reads": 0,
            "cache_reads": 0,
            "transport_reads": 0,
        },
        "mod256_boundary": {
            "discrete_role_provenance": "VALIDATED_ONLY",
            "softmax_probability_normalization": "REAL_VALUED_NOT_MOD256",
        },
    }
    preparation = _with_cid(body, "preparation_cid")
    _write_exclusive_json(path, preparation)
    return preparation


def select_execution_plan(records: Sequence[Mapping[str, Any]]) -> dict[str, Any]:
    """Select the fastest complete deterministic 1/4/8-thread measurement."""

    expected = set(ELIGIBLE_THREADS)
    observed: set[int] = set()
    normalized: list[dict[str, Any]] = []
    eligible: list[dict[str, Any]] = []
    for source in records:
        record = dict(source)
        plan = record.get("plan")
        if not isinstance(plan, Mapping):
            raise ValueError("#1047 timing record lacks its plan")
        threads = int(plan.get("threads", -1))
        if threads in observed:
            raise ValueError("#1047 timing matrix repeats a thread plan")
        observed.add(threads)
        projection = record.get("projected_c1_c2_seconds")
        memory = record.get("peak_rss_bytes")
        arms = record.get("arms")
        measured_each_arm = bool(
            isinstance(arms, Mapping)
            and set(arms) == {"c1", "c2"}
            and all(
                isinstance(arms[name], Mapping)
                and int(arms[name].get("timed_training_batches", 0))
                >= TIMED_TRAINING_BATCHES
                and arms[name].get("full_development_evaluation") is True
                and arms[name].get("deterministic_replay") is True
                for name in ("c1", "c2")
            )
        )
        finite = all(
            isinstance(value, (int, float))
            and not isinstance(value, bool)
            and math.isfinite(float(value))
            and float(value) >= 0.0
            for value in (projection, memory)
        )
        record["eligible"] = bool(
            finite
            and measured_each_arm
            and record.get("deterministic_replay") is True
            and int(record.get("timed_training_batches", 0))
            >= TIMED_TRAINING_BATCHES
            and record.get("full_development_evaluation") is True
            and float(projection) <= HARD_WALL_SECONDS
            and int(memory) <= MEMORY_CEILING_BYTES
        )
        normalized.append(record)
        if record["eligible"]:
            eligible.append(record)
    if observed != expected:
        raise ValueError("#1047 1/4/8-thread timing matrix is incomplete")
    selected = min(
        eligible,
        key=lambda value: (
            float(value["projected_c1_c2_seconds"]),
            int(value["plan"]["threads"]),
        ),
        default=None,
    )
    return {
        "available": selected is not None,
        "plans": normalized,
        "selected_plan": None if selected is None else selected["plan"],
        "selected_projection_seconds": (
            None if selected is None else selected["projected_c1_c2_seconds"]
        ),
        "hard_wall_seconds": HARD_WALL_SECONDS,
        "memory_ceiling_bytes": MEMORY_CEILING_BYTES,
        "projection_safety_factor": PROJECTION_SAFETY_FACTOR,
    }


def _project_rung_seconds(
    *,
    train_batches: int,
    development_batches: int,
    seconds_per_training_batch: float,
    seconds_per_evaluation_batch: float,
) -> float:
    """Reserve the worst-case training and full-score work for one rung."""

    return (
        MAXIMUM_EPOCHS * train_batches * seconds_per_training_batch
        + MAXIMUM_EPOCHS
        * (development_batches + train_batches)
        * seconds_per_evaluation_batch
    )


def decide_zoology_control(
    *,
    c0_passed: bool,
    preflight_available: bool,
    c1_train_rate: float | None = None,
    c1_development_rate: float | None = None,
    c1_consecutive_passes: int = 0,
    c2_train_rate: float | None = None,
    c2_development_rate: float | None = None,
    c2_consecutive_passes: int = 0,
    binding_permuted_rate: float | None = None,
) -> dict[str, Any]:
    """Return the first frozen #1047 decision without adding a stop code."""

    c1_train = c1_train_rate is not None and c1_train_rate >= TRAIN_REQUIRED_RATE
    c1_development = (
        c1_development_rate is not None
        and c1_development_rate >= DEVELOPMENT_REQUIRED_RATE
    )
    c1_two = c1_consecutive_passes >= REQUIRED_CONSECUTIVE_PASSES
    c2_train = c2_train_rate is not None and c2_train_rate >= TRAIN_REQUIRED_RATE
    c2_development = (
        c2_development_rate is not None
        and c2_development_rate >= DEVELOPMENT_REQUIRED_RATE
    )
    c2_two = c2_consecutive_passes >= REQUIRED_CONSECUTIVE_PASSES
    gates: dict[str, Any] = {
        "c0_source_mechanics": c0_passed,
        "preflight_available": preflight_available,
        "c1_train": c1_train,
        "c1_development": c1_development,
        "c1_two_consecutive": c1_two,
        "c2_train": c2_train,
        "c2_development": c2_development,
        "c2_two_consecutive": c2_two,
        "binding_permuted_drop": None,
    }
    if not c0_passed:
        verdict: Verdict | None = "INVALID_CONTROL_PORT"
        status = "DECIDED"
        action = "repair only source parity; make no scientific inference"
        passed = False
    elif not preflight_available:
        verdict = None
        status = "NOT_RUN_PREFLIGHT"
        action = "stop without a scientific verdict; do not tune #1049"
        passed = False
    elif not (c1_train and c1_development and c1_two):
        verdict = "SCALED_SOURCE_CALIBRATION_MISS"
        status = "DECIDED"
        action = "stop before UOR bytes; do not modify R4"
        passed = False
    elif not c2_train:
        verdict = "STOCK_CELL_EXACT_QUALIFICATION_MISS"
        status = "DECIDED"
        action = (
            "make no assignment-disjoint transfer inference; isolate exact-byte "
            "fit mechanics only in a new frozen contract"
        )
        passed = False
    elif not c2_development:
        verdict = "STOCK_CELL_TRANSFER_MISS"
        status = "DECIDED"
        action = (
            "isolate source serialization versus assignment-disjointness in a "
            "new frozen contract"
        )
        passed = False
    elif not c2_two:
        verdict = "STOCK_CELL_EXACT_QUALIFICATION_MISS"
        status = "DECIDED"
        action = (
            "make no stable exact-byte transfer claim; isolate qualification "
            "mechanics only in a new frozen contract"
        )
        passed = False
    elif binding_permuted_rate is None:
        verdict = None
        status = "INCOMPLETE_BINDING_CONTROL"
        action = "run only the frozen data-level binding-permuted control"
        passed = False
    else:
        drop = float(c2_development_rate) - binding_permuted_rate
        gates["binding_permuted_drop"] = drop >= CONTROL_REQUIRED_DROP
        if drop < CONTROL_REQUIRED_DROP:
            verdict = "NONASSOCIATIVE_SHORTCUT"
            action = "do not accept the primary score as attention evidence"
            passed = False
        else:
            verdict = "STOCK_CELL_PASSES_EXACT_BYTES"
            action = (
                "align the R4 cell to the demonstrated one-head width-64 "
                "addressing boundary before English transfer"
            )
            passed = True
        status = "DECIDED"
    return {
        "status": status,
        "verdict": verdict,
        "passed": passed,
        "gates": gates,
        "thresholds": {
            "train": TRAIN_REQUIRED_RATE,
            "development": DEVELOPMENT_REQUIRED_RATE,
            "consecutive": REQUIRED_CONSECUTIVE_PASSES,
            "binding_permuted_drop": CONTROL_REQUIRED_DROP,
        },
        "action": action,
    }


def _preflight_stop_marker(c0_passed: bool) -> str:
    return "NOT_RUN_PREFLIGHT" if c0_passed else "NOT_RUN_C0_MISS"


def _result_body(
    *,
    preparation: Mapping[str, Any],
    preflight: Mapping[str, Any],
    plan: Mapping[str, Any] | None,
    c1: Mapping[str, Any] | str,
    c2: Mapping[str, Any] | str,
    binding_control: Mapping[str, Any] | str,
    artifacts: Sequence[Mapping[str, Any]],
    decision: Mapping[str, Any],
    elapsed_seconds: float,
) -> dict[str, Any]:
    """Build the bounded result envelope without making out-of-scope claims."""

    return {
        "schema": RESULT_SCHEMA,
        "issue": ISSUE,
        "policy": POLICY,
        "preparation_cid": preparation["preparation_cid"],
        "preflight_cid": preflight["preflight_cid"],
        "implementation": preflight["implementation"],
        "source_attribution_cid": preparation["inputs"][
            "source_attribution_cid"
        ],
        "population_cids": {
            "c1_source_native": preparation["inputs"][
                "source_native_population_cid"
            ],
            "c2_exact_1045": preparation["inputs"][
                "exact_1045_population_cid"
            ],
            "source_1045_split": preparation["inputs"]["source_1045_split_cid"],
        },
        "plan": None if plan is None else dict(plan),
        "rungs": {"c1": c1, "c2": c2},
        "binding_permuted_control": binding_control,
        "artifacts": [dict(value) for value in artifacts],
        "decision": dict(decision),
        "elapsed_seconds": elapsed_seconds,
        "hard_wall_seconds": HARD_WALL_SECONDS,
        "read_work_ledger": {
            "failed_source_artifact_reads": 0,
            "sealed_input_reads": 0,
            "provider_calls": 0,
            "teacher_calls": 0,
            "future_value_reads": 0,
            "role_model_input_reads": 0,
            "h4_model_input_reads": 0,
            "cache_reads": 0,
            "transport_reads": 0,
        },
        "controls_not_invented": {
            "current_only": "NOT_RUN_NO_STOCK_INTERVENTION",
            "cache": "NOT_RUN_NO_STOCK_INTERVENTION",
            "transport": "NOT_RUN_NO_STOCK_INTERVENTION",
            "attention_off": "NOT_RUN_NO_STOCK_INTERVENTION",
        },
        "claim_boundary": {
            "ordinary_causal_softmax": "ONLY",
            "r4_or_geometric_attention": "NOT_CLAIMED",
            "english": "NOT_RUN",
            "generation": "NOT_RUN",
            "reasoning": "NOT_RUN",
            "recurrence": "NOT_RUN",
            "quantization": "NOT_RUN",
            "exact_lowering": "NOT_RUN",
            "browser_wasm_product_release": "NOT_RUN",
        },
    }


def _configure_cpu(threads: int) -> torch.device:
    ExecutionPlan(threads)
    os.environ["OMP_NUM_THREADS"] = str(threads)
    os.environ["VECLIB_MAXIMUM_THREADS"] = str(threads)
    torch.set_num_threads(threads)
    if torch.get_num_interop_threads() != 1:
        try:
            torch.set_num_interop_threads(1)
        except RuntimeError as error:
            if torch.get_num_interop_threads() != 1:
                raise RuntimeError(
                    "#1047 CPU inter-op threads were initialized before the frozen plan"
                ) from error
    return torch.device("cpu")


def _tensor_mapping_cid(tensors: Mapping[str, Tensor]) -> str:
    digest = blake3()
    for name in sorted(tensors):
        tensor = tensors[name].detach().cpu().contiguous()
        digest.update(
            canonical_json_bytes(
                {
                    "name": name,
                    "shape": list(tensor.shape),
                    "dtype": str(tensor.dtype),
                }
            )
        )
        digest.update(tensor.numpy().tobytes(order="C"))
    return f"blake3:{digest.hexdigest()}"


def _model_config(population: ZoologyMQARPopulation) -> ZoologyFigure2Config:
    return ZoologyFigure2Config(
        vocab_size=population.vocab_size,
        max_position_embeddings=population.input_seq_len,
    )


def _new_model(
    population: ZoologyMQARPopulation,
    *,
    device: torch.device,
) -> ZoologyFigure2Model:
    set_zoology_seed(SOURCE_MODEL_SEED)
    return ZoologyFigure2Model(_model_config(population)).to(device)


def _artifact_payload(
    model: ZoologyFigure2Model,
    *,
    rung: str,
    population_cid: str,
) -> bytes:
    tensors = {
        name: tensor.detach().cpu().contiguous()
        for name, tensor in sorted(model.state_dict().items())
        if name != "lm_head.weight"
    }
    if "backbone.embeddings.word_embeddings.weight" not in tensors:
        raise RuntimeError("Zoology artifact lacks its canonical tied embedding")
    metadata = {
        "schema": "uor-r4.zoology-control-model/1",
        "issue": str(ISSUE),
        "policy": POLICY,
        "rung": rung,
        "population_cid": population_cid,
        "config": canonical_json_bytes(asdict(model.config)).decode("utf-8"),
        "tied_omission": "lm_head.weight",
    }
    return save_safetensors(tensors, metadata=metadata)


def _write_model_artifact(
    root: Path,
    model: ZoologyFigure2Model,
    *,
    rung: str,
    population_cid: str,
) -> dict[str, Any]:
    relative = (
        C1_ARTIFACT_RELATIVE_PATH if rung == "c1" else C2_ARTIFACT_RELATIVE_PATH
    )
    payload = _artifact_payload(model, rung=rung, population_cid=population_cid)
    _write_exclusive(root / relative, payload)
    return {
        "rung": rung,
        "path": relative,
        "bytes": len(payload),
        "cid": cid_bytes(payload),
        "state_cid": _tensor_mapping_cid(
            {
                name: tensor
                for name, tensor in model.state_dict().items()
                if name != "lm_head.weight"
            }
        ),
        "population_cid": population_cid,
    }


def _batch_work(batch: ZoologyMQARBatch, *, vocab_size: int) -> dict[str, int]:
    batch_size, time_width = batch.input_ids.shape
    visible = 0
    masked_future = 0
    for positions in batch.selected_positions.detach().cpu().tolist():
        for position in positions:
            visible += int(position) + 1
            masked_future += time_width - int(position) - 1
    decisions = int(batch.targets.numel())
    return {
        "batches": 1,
        "rows": batch_size,
        "input_tensor_tokens": int(batch.input_ids.numel()),
        "query_decisions": decisions,
        "target_reads": decisions,
        "visible_query_token_pairs": visible,
        "causally_masked_future_token_pairs": masked_future,
        "materialized_attention_scores": 2 * batch_size * time_width * time_width,
        "materialized_vocabulary_scores": int(batch.targets.numel()) * vocab_size,
        "cache_reads": 0,
        "transport_reads": 0,
        "future_value_reads": 0,
        "role_model_input_reads": 0,
        "h4_model_input_reads": 0,
        "provider_calls": 0,
        "teacher_calls": 0,
        "forbidden_reads": 0,
        "sealed_input_reads": 0,
    }


def _merge_work(total: dict[str, int], later: Mapping[str, int]) -> None:
    for name, value in later.items():
        total[name] = total.get(name, 0) + int(value)


def _score_rows(
    model: ZoologyFigure2Model,
    rows: Sequence[ZoologyMQARRow],
    *,
    device: torch.device,
    batch_size: int = BATCH_SIZE,
    deadline: float | None = None,
) -> ScoreResult:
    if not rows:
        raise ValueError("#1047 score population cannot be empty")
    was_training = model.training
    model.eval()
    decisions = 0
    correct = 0
    loss_sum = 0.0
    digest = blake3()
    work: dict[str, int] = {}
    try:
        with torch.inference_mode():
            for start in range(0, len(rows), batch_size):
                if deadline is not None and time.monotonic() >= deadline:
                    raise HardWallExceeded(
                        f"#1049 scoring reached the {HARD_WALL_SECONDS:.0f}-second wall"
                    )
                batch = batch_rows(rows[start : start + batch_size], device=device)
                output = model.forward_selected(
                    batch.input_ids,
                    batch.selected_positions,
                    batch.targets,
                )
                if output.selected_targets is None:
                    raise RuntimeError("#1047 query score lacks selected targets")
                flat_logits = output.logits.float().reshape(
                    -1, output.logits.shape[-1]
                )
                flat_targets = output.selected_targets.reshape(-1)
                loss_sum += float(
                    F.cross_entropy(flat_logits, flat_targets, reduction="sum")
                )
                predictions = flat_logits.argmax(dim=-1)
                decisions += int(flat_targets.numel())
                correct += int(torch.count_nonzero(predictions == flat_targets))
                cpu_logits = output.logits.detach().cpu().contiguous()
                digest.update(canonical_json_bytes({"shape": list(cpu_logits.shape)}))
                digest.update(cpu_logits.numpy().tobytes(order="C"))
                _merge_work(
                    work,
                    _batch_work(batch, vocab_size=model.config.vocab_size),
                )
                if deadline is not None and time.monotonic() >= deadline:
                    raise HardWallExceeded(
                        f"#1049 scoring reached the {HARD_WALL_SECONDS:.0f}-second wall"
                    )
    finally:
        model.train(was_training)
    expected = _row_query_count(rows)
    if decisions != expected or work.get("target_reads") != expected:
        raise RuntimeError("#1047 query-score work ledger differs")
    return ScoreResult(
        decisions=decisions,
        correct=correct,
        loss_sum=loss_sum,
        selected_logits_cid=f"blake3:{digest.hexdigest()}",
        work=work,
    )


def _train_epoch(
    model: ZoologyFigure2Model,
    optimizer: torch.optim.Optimizer,
    rows: Sequence[ZoologyMQARRow],
    *,
    epoch: int,
    namespace: str,
    device: torch.device,
    deadline: float,
) -> dict[str, Any]:
    model.train()
    ordered = deterministic_epoch_order(rows, epoch, namespace)
    decisions = 0
    correct = 0
    loss_sum = 0.0
    work: dict[str, int] = {}

    def record(*, complete: bool) -> dict[str, Any]:
        return {
            "epoch": epoch + 1,
            "complete": complete,
            "query_presentations": decisions,
            "online_top1_correct": correct,
            "online_top1_rate": None if decisions == 0 else correct / decisions,
            "online_nll_nats": None if decisions == 0 else loss_sum / decisions,
            "work": dict(work),
        }

    for start in range(0, len(ordered), BATCH_SIZE):
        if time.monotonic() >= deadline:
            partial = record(complete=False)
            raise HardWallExceeded(
                f"#1049 training reached the {HARD_WALL_SECONDS:.0f}-second wall",
                partial,
            )
        batch = batch_rows(ordered[start : start + BATCH_SIZE], device=device)
        optimizer.zero_grad(set_to_none=True)
        output = model.forward_selected(
            batch.input_ids,
            batch.selected_positions,
            batch.targets,
        )
        if output.loss is None or output.selected_targets is None:
            raise RuntimeError("#1047 training output lacks query loss/targets")
        output.loss.backward()
        optimizer.step()
        batch_decisions = int(output.selected_targets.numel())
        decisions += batch_decisions
        loss_sum += float(output.loss.detach()) * batch_decisions
        predictions = output.logits.detach().argmax(dim=-1)
        correct += int(torch.count_nonzero(predictions == output.selected_targets))
        _merge_work(
            work,
            _batch_work(batch, vocab_size=model.config.vocab_size),
        )
        if time.monotonic() >= deadline:
            partial = record(complete=False)
            raise HardWallExceeded(
                f"#1049 training reached the {HARD_WALL_SECONDS:.0f}-second wall",
                partial,
            )
    expected = _row_query_count(rows)
    if decisions != expected or work.get("target_reads") != expected:
        raise RuntimeError("#1047 training work ledger differs")
    return record(complete=True)


def _train_rung(
    population: ZoologyMQARPopulation,
    *,
    rung: str,
    device: torch.device,
    deadline: float,
) -> tuple[ZoologyFigure2Model, dict[str, Any]]:
    model = _new_model(population, device=device)
    optimizer = torch.optim.AdamW(
        model.parameters(),
        lr=LEARNING_RATE,
        weight_decay=WEIGHT_DECAY,
    )
    scheduler = torch.optim.lr_scheduler.CosineAnnealingLR(
        optimizer,
        T_max=MAXIMUM_EPOCHS,
        eta_min=0.0,
    )
    history: list[dict[str, Any]] = []
    consecutive = 0
    presentations = 0
    final_train: ScoreResult | None = None
    final_development: ScoreResult | None = None
    incomplete_reason: str | None = None
    began = time.monotonic()
    for epoch in range(MAXIMUM_EPOCHS):
        try:
            training = _train_epoch(
                model,
                optimizer,
                population.train,
                epoch=epoch,
                # Preserve V1 ordering exactly; #1049 changes resource policy only.
                namespace=f"uor-r4/1047/{rung}",
                device=device,
                deadline=deadline,
            )
        except HardWallExceeded as error:
            if error.partial is not None:
                presentations += int(error.partial["query_presentations"])
                history.append(
                    {
                        "epoch": epoch + 1,
                        "learning_rate": float(optimizer.param_groups[0]["lr"]),
                        "training": error.partial,
                        "development": None,
                        "train_evaluation": None,
                        "consecutive_passes": consecutive,
                    }
                )
            incomplete_reason = str(error)
            break
        presentations += int(training["query_presentations"])
        scheduler.step()
        final_train = None
        final_development = None
        try:
            final_development = _score_rows(
                model,
                population.development,
                device=device,
                deadline=deadline,
            )
            if final_development.rate >= DEVELOPMENT_REQUIRED_RATE:
                final_train = _score_rows(
                    model,
                    population.train,
                    device=device,
                    deadline=deadline,
                )
        except HardWallExceeded as error:
            incomplete_reason = str(error)
        passed_epoch = bool(
            incomplete_reason is None
            and final_train is not None
            and final_development is not None
            and final_train.rate >= TRAIN_REQUIRED_RATE
            and final_development.rate >= DEVELOPMENT_REQUIRED_RATE
        )
        consecutive = consecutive + 1 if passed_epoch else 0
        history.append(
            {
                "epoch": epoch + 1,
                "learning_rate": float(scheduler.get_last_lr()[0]),
                "training": training,
                "development": (
                    None if final_development is None else final_development.record()
                ),
                "train_evaluation": (
                    None if final_train is None else final_train.record()
                ),
                "consecutive_passes": consecutive,
            }
        )
        if incomplete_reason is not None or consecutive >= REQUIRED_CONSECUTIVE_PASSES:
            break

    if incomplete_reason is None and final_train is None:
        try:
            final_train = _score_rows(
                model,
                population.train,
                device=device,
                deadline=deadline,
            )
        except HardWallExceeded as error:
            incomplete_reason = str(error)
    passed = bool(
        incomplete_reason is None
        and final_train is not None
        and final_development is not None
        and final_train.rate >= TRAIN_REQUIRED_RATE
        and final_development.rate >= DEVELOPMENT_REQUIRED_RATE
        and consecutive >= REQUIRED_CONSECUTIVE_PASSES
    )
    if rung == "c2" and presentations > MAXIMUM_C2_QUERY_PRESENTATIONS:
        raise RuntimeError("#1047 C2 query-presentation cap was exceeded")
    return model, {
        "status": "INCOMPLETE_HARD_WALL" if incomplete_reason else "COMPLETE",
        "passed": passed,
        "population_cid": population.population_cid,
        "epochs": len(history),
        "query_presentations": presentations,
        "history": history,
        "final_train": None if final_train is None else final_train.record(),
        "final_development": (
            None if final_development is None else final_development.record()
        ),
        "consecutive_passes": consecutive,
        "incomplete_reason": incomplete_reason,
        "elapsed_seconds": time.monotonic() - began,
        "optimizer": {
            "name": "AdamW",
            "learning_rate": LEARNING_RATE,
            "weight_decay": WEIGHT_DECAY,
            "betas": [0.9, 0.999],
            "epsilon": 1e-8,
            "gradient_clip": "NONE",
            "schedule": "CosineAnnealingLR(epoch,to_zero)",
            "maximum_epochs": MAXIMUM_EPOCHS,
            "checkpoint_selection": "first two consecutive open passes",
        },
    }


def _mechanics_checks(
    population: ZoologyMQARPopulation,
    *,
    device: torch.device,
) -> dict[str, Any]:
    first = _new_model(population, device=device).eval()
    first_state_cid = _tensor_mapping_cid(first.state_dict())
    second = _new_model(population, device=device).eval()
    second_state_cid = _tensor_mapping_cid(second.state_dict())
    batch = batch_rows(population.train[:2], device=device)
    with torch.inference_mode():
        full = first.forward_full(batch.input_ids)
        selected = first.forward_selected(
            batch.input_ids,
            batch.selected_positions,
        )
        gather_index = batch.selected_positions.unsqueeze(-1).expand(
            -1,
            -1,
            population.vocab_size,
        )
        gathered = torch.gather(full.logits, dim=1, index=gather_index)
        query_projection_parity = bool(
            torch.allclose(gathered, selected.logits, rtol=2e-5, atol=2e-6)
        )

        position = int(batch.selected_positions[0, 0])
        one_position = batch.selected_positions[:1, :1]
        full_query = first.forward_selected(
            batch.input_ids[:1],
            one_position,
        ).logits[:, 0]
        prefix_logits = first(batch.input_ids[:1, : position + 1])[:, -1]
        causal_prefix_parity = bool(
            torch.allclose(full_query, prefix_logits, rtol=2e-5, atol=2e-6)
        )
    return {
        "initialization_state_cid": first_state_cid,
        "initialization_replay_state_cid": second_state_cid,
        "initialization_exact_replay": first_state_cid == second_state_cid,
        "query_projection_scale_aware_parity": query_projection_parity,
        "causal_prefix_scale_aware_parity": causal_prefix_parity,
        "passed": bool(
            first_state_cid == second_state_cid
            and query_projection_parity
            and causal_prefix_parity
        ),
    }


def _source_oracle_golden() -> dict[str, Any]:
    """Execute the literal-source tiny loader/model goldens in the preflight."""

    inputs, labels = zoology_data._released_mqar(
        vocab_size=32,
        num_examples=3,
        input_seq_len=16,
        seed=0,
        num_kv_pairs=2,
    )
    expected_inputs = torch.tensor(_SOURCE_GOLDEN_INPUTS, dtype=torch.long)
    expected_labels = torch.tensor(_SOURCE_GOLDEN_LABELS, dtype=torch.long)
    loader_inputs_exact = bool(torch.equal(inputs, expected_inputs))
    loader_labels_exact = bool(torch.equal(labels, expected_labels))

    tiny = ZoologyFigure2Config(
        vocab_size=32,
        d_model=8,
        n_layers=2,
        num_heads=1,
        max_position_embeddings=8,
        attention_dropout=0.1,
        embed_dropout=0.1,
        resid_dropout=0.0,
    )
    set_zoology_seed(SOURCE_MODEL_SEED)
    model = ZoologyFigure2Model(tiny).eval()
    word = model.backbone.embeddings.word_embeddings.weight[0, :4].detach().cpu()
    position = (
        model.backbone.embeddings.position_embeddings.weight[0, :4]
        .detach()
        .cpu()
    )
    wqkv = (
        model.backbone.layers[0].sequence_mixer.Wqkv.weight[0, :4]
        .detach()
        .cpu()
    )
    expected_word = torch.tensor(_SOURCE_GOLDEN_WORD)
    expected_position = torch.tensor(_SOURCE_GOLDEN_POSITION)
    expected_wqkv = torch.tensor(_SOURCE_GOLDEN_WQKV)
    parameter_byte_exact = bool(
        torch.equal(word, expected_word)
        and torch.equal(position, expected_position)
        and torch.equal(wqkv, expected_wqkv)
    )
    with torch.inference_mode():
        observed_logits = model(
            torch.tensor([[1, 2, 3, 4], [4, 3, 2, 1]], dtype=torch.long)
        )[0, -1, :6].cpu()
    expected_logits = torch.tensor(_SOURCE_GOLDEN_LOGITS)
    logits_byte_exact = bool(torch.equal(observed_logits, expected_logits))
    logits_scale_aware = bool(
        torch.allclose(observed_logits, expected_logits, rtol=2e-6, atol=2e-7)
    )
    loader_digest = blake3()
    loader_digest.update(inputs.contiguous().numpy().tobytes(order="C"))
    loader_digest.update(labels.contiguous().numpy().tobytes(order="C"))
    return {
        "authority": "HazyResearch/Zoology@de4e258 ICLR24 literal execution",
        "loader": {
            "inputs_byte_exact": loader_inputs_exact,
            "labels_byte_exact": loader_labels_exact,
            "golden_cid": f"blake3:{loader_digest.hexdigest()}",
            "passed": bool(loader_inputs_exact and loader_labels_exact),
        },
        "model": {
            "parameters_byte_exact": parameter_byte_exact,
            "logits_byte_exact": logits_byte_exact,
            "logits_scale_aware_parity": logits_scale_aware,
            "rtol": 2e-6,
            "atol": 2e-7,
            "passed": bool(parameter_byte_exact and logits_scale_aware),
        },
        "passed": bool(
            loader_inputs_exact
            and loader_labels_exact
            and parameter_byte_exact
            and logits_scale_aware
        ),
    }


def _run_c0(
    population: ZoologyMQARPopulation,
    *,
    device: torch.device,
) -> dict[str, Any]:
    source_oracle = _source_oracle_golden()
    replay = build_source_calibration()
    loader_exact_replay = replay.population_cid == population.population_cid
    mechanics = _mechanics_checks(population, device=device)
    rows = population.train[:OVERFIT_ROWS]
    model = _new_model(population, device=device)
    optimizer = torch.optim.AdamW(
        model.parameters(),
        lr=LEARNING_RATE,
        weight_decay=WEIGHT_DECAY,
    )
    scheduler = torch.optim.lr_scheduler.CosineAnnealingLR(
        optimizer,
        T_max=OVERFIT_UPDATES,
        eta_min=0.0,
    )
    batch = batch_rows(rows, device=device)
    passed = False
    score: ScoreResult | None = None
    began = time.monotonic()
    updates = 0
    for update in range(OVERFIT_UPDATES):
        model.train()
        optimizer.zero_grad(set_to_none=True)
        output = model.forward_selected(
            batch.input_ids,
            batch.selected_positions,
            batch.targets,
        )
        if output.loss is None:
            raise RuntimeError("#1047 C0 lacks its query-only loss")
        output.loss.backward()
        optimizer.step()
        scheduler.step()
        updates = update + 1
        score = _score_rows(model, rows, device=device, batch_size=OVERFIT_ROWS)
        if score.rate == 1.0:
            passed = True
            break
    return {
        "rows": OVERFIT_ROWS,
        "maximum_updates": OVERFIT_UPDATES,
        "updates": updates,
        "loader_population_cid": population.population_cid,
        "loader_replay_population_cid": replay.population_cid,
        "loader_exact_replay": loader_exact_replay,
        "source_oracle_golden": source_oracle,
        "mechanics": mechanics,
        "overfit": None if score is None else score.record(),
        "query_presentations": updates * _row_query_count(rows),
        "elapsed_seconds": time.monotonic() - began,
        "passed": bool(
            source_oracle["passed"]
            and loader_exact_replay
            and mechanics["passed"]
            and passed
        ),
    }


def _determinism_signature(
    population: ZoologyMQARPopulation,
    *,
    device: torch.device,
    namespace: str,
) -> dict[str, Any]:
    model = _new_model(population, device=device)
    optimizer = torch.optim.AdamW(
        model.parameters(), lr=LEARNING_RATE, weight_decay=WEIGHT_DECAY
    )
    ordered = deterministic_epoch_order(
        population.train,
        0,
        f"uor-r4/1047/preflight-replay/{namespace}",
    )
    losses: list[float] = []
    for start in (0, BATCH_SIZE):
        batch = batch_rows(ordered[start : start + BATCH_SIZE], device=device)
        optimizer.zero_grad(set_to_none=True)
        output = model.forward_selected(
            batch.input_ids,
            batch.selected_positions,
            batch.targets,
        )
        if output.loss is None:
            raise RuntimeError("#1047 timing replay lacks loss")
        output.loss.backward()
        optimizer.step()
        losses.append(float(output.loss.detach()))
    return {"losses": losses, "state_cid": _tensor_mapping_cid(model.state_dict())}


def _measure_probe_arm(
    population: ZoologyMQARPopulation,
    *,
    rung: str,
    threads: int,
    device: torch.device,
) -> dict[str, Any]:
    replay_a = _determinism_signature(
        population,
        device=device,
        namespace=rung,
    )
    replay_b = _determinism_signature(
        population,
        device=device,
        namespace=rung,
    )
    model = _new_model(population, device=device)
    optimizer = torch.optim.AdamW(
        model.parameters(), lr=LEARNING_RATE, weight_decay=WEIGHT_DECAY
    )
    ordered = deterministic_epoch_order(
        population.train,
        0,
        f"uor-r4/1047/preflight/{rung}/{threads}",
    )
    timed_rows = ordered[: TIMED_TRAINING_BATCHES * BATCH_SIZE]
    began = time.monotonic()
    for start in range(0, len(timed_rows), BATCH_SIZE):
        batch = batch_rows(timed_rows[start : start + BATCH_SIZE], device=device)
        optimizer.zero_grad(set_to_none=True)
        output = model.forward_selected(
            batch.input_ids,
            batch.selected_positions,
            batch.targets,
        )
        if output.loss is None:
            raise RuntimeError("#1047 timing probe lacks loss")
        output.loss.backward()
        optimizer.step()
    training_seconds = time.monotonic() - began
    evaluation_began = time.monotonic()
    development = _score_rows(model, population.development, device=device)
    evaluation_seconds = time.monotonic() - evaluation_began

    train_batches = math.ceil(len(population.train) / BATCH_SIZE)
    development_batches = math.ceil(len(population.development) / BATCH_SIZE)
    seconds_per_training_batch = training_seconds / TIMED_TRAINING_BATCHES
    seconds_per_evaluation_batch = evaluation_seconds / development_batches
    projected = _project_rung_seconds(
        train_batches=train_batches,
        development_batches=development_batches,
        seconds_per_training_batch=seconds_per_training_batch,
        seconds_per_evaluation_batch=seconds_per_evaluation_batch,
    )
    return {
        "rung": rung,
        "population_cid": population.population_cid,
        "timed_training_batches": TIMED_TRAINING_BATCHES,
        "training_seconds": training_seconds,
        "seconds_per_training_batch": seconds_per_training_batch,
        "full_development_evaluation": True,
        "development_evaluation": development.record(),
        "development_evaluation_seconds": evaluation_seconds,
        "deterministic_replay": replay_a == replay_b,
        "deterministic_replay_records": [replay_a, replay_b],
        "projected_rung_seconds_before_safety": projected,
    }


def _probe_once(threads: int, source_root: Path) -> dict[str, Any]:
    device = _configure_cpu(threads)
    source_population, exact_population = _load_bound_populations(source_root)
    c1 = _measure_probe_arm(
        source_population,
        rung="c1",
        threads=threads,
        device=device,
    )
    c2 = _measure_probe_arm(
        exact_population,
        rung="c2",
        threads=threads,
        device=device,
    )
    # A passing C2 performs one additional full binding-permuted development
    # evaluation. Its cost is measured directly from the C2 development pass.
    projected_before_safety = (
        float(c1["projected_rung_seconds_before_safety"])
        + float(c2["projected_rung_seconds_before_safety"])
        + float(c2["development_evaluation_seconds"])
    )
    return {
        "plan": ExecutionPlan(threads).record(),
        "arms": {"c1": c1, "c2": c2},
        "timed_training_batches": 2 * TIMED_TRAINING_BATCHES,
        "timed_training_batches_per_arm": TIMED_TRAINING_BATCHES,
        "full_development_evaluation": True,
        "full_development_evaluation_per_arm": True,
        "deterministic_replay": bool(
            c1["deterministic_replay"] and c2["deterministic_replay"]
        ),
        "projected_c1_c2_seconds_before_safety": projected_before_safety,
        "projected_c1_c2_seconds": (
            PROJECTION_SAFETY_FACTOR * projected_before_safety
        ),
        "projection_safety_factor": PROJECTION_SAFETY_FACTOR,
        "peak_rss_bytes": _peak_rss_bytes(),
    }


def _probe_worker(queue: Any, threads: int, source_root: str) -> None:
    try:
        queue.put(
            {
                "ok": True,
                "record": _probe_once(threads, Path(source_root)),
            }
        )
    except BaseException as error:
        queue.put(
            {
                "ok": False,
                "error": {
                    "type": type(error).__name__,
                    "reason": str(error),
                    "traceback": traceback.format_exc(),
                },
            }
        )


def _spawn_probe(threads: int, source_root: Path) -> dict[str, Any]:
    context = mp.get_context("spawn")
    queue = context.Queue()
    process = context.Process(
        target=_probe_worker,
        args=(queue, threads, str(source_root)),
    )
    process.start()
    process.join(PROBE_TIMEOUT_SECONDS)
    if process.is_alive():
        process.terminate()
        process.join(timeout=10.0)
        return {
            "ok": False,
            "error": {
                "type": "TimeoutError",
                "reason": f"{threads}-thread probe exceeded {PROBE_TIMEOUT_SECONDS:.0f}s",
            },
        }
    try:
        return dict(queue.get(timeout=2.0))
    except queue_module.Empty:
        return {
            "ok": False,
            "error": {
                "type": "RuntimeError",
                "reason": f"{threads}-thread probe exited without evidence",
                "exitcode": process.exitcode,
            },
        }
    finally:
        queue.close()
        queue.join_thread()


def _started_record(
    *,
    phase: str,
    preparation: Mapping[str, Any],
    implementation: Mapping[str, Any],
) -> dict[str, Any]:
    return _with_cid(
        {
            "schema": STARTED_SCHEMA,
            "issue": ISSUE,
            "policy": POLICY,
            "phase": phase,
            "preparation_cid": preparation["preparation_cid"],
            "implementation_cid": implementation["implementation_cid"],
            "implementation_tree_cid": implementation["tree_cid"],
        },
        "started_cid",
    )


def _failed_probe_record(threads: int, error: Mapping[str, Any]) -> dict[str, Any]:
    empty_arm = {
        "timed_training_batches": 0,
        "full_development_evaluation": False,
        "deterministic_replay": False,
    }
    return {
        "plan": ExecutionPlan(threads).record(),
        "arms": {"c1": dict(empty_arm), "c2": dict(empty_arm)},
        "timed_training_batches": 0,
        "timed_training_batches_per_arm": 0,
        "full_development_evaluation": False,
        "full_development_evaluation_per_arm": False,
        "deterministic_replay": False,
        "projected_c1_c2_seconds": HARD_WALL_SECONDS + 1.0,
        "peak_rss_bytes": 0,
        "error": dict(error),
    }


def preflight_zoology_control(root: Path) -> dict[str, Any]:
    """Run C0 and the create-once measured 1/4/8-thread CPU admission."""

    root = root.resolve()
    path = root / PREFLIGHT_RELATIVE_PATH
    started_path = root / PREFLIGHT_STARTED_RELATIVE_PATH
    preparation = _read_json(
        root / PREPARATION_RELATIVE_PATH,
        cid_field="preparation_cid",
    )
    implementation = zoology_control_implementation_contract()
    if preparation.get("implementation") != implementation:
        raise ValueError("#1047 implementation changed after preparation")
    source_root = Path(str(preparation["source_root"]))
    predecessor_root = Path(str(preparation["predecessor_root"]))
    source_population, exact_population = _load_bound_populations(source_root)
    if preparation.get("predecessor") != _bind_predecessor(predecessor_root):
        raise ValueError("#1045 predecessor changed after preparation")
    if preparation.get("capacity_predecessor") != _capacity_predecessor_record():
        raise ValueError("#1047 capacity predecessor changed after preparation")
    if preparation.get("populations") != {
        "c1_source_native": _population_record(source_population),
        "c2_exact_1045": _population_record(exact_population),
    }:
        raise ValueError("#1047 populations changed after preparation")

    if path.exists():
        preflight = _read_json(path, cid_field="preflight_cid")
        started = _read_json(started_path, cid_field="started_cid")
        if (
            preflight.get("preparation_cid") != preparation["preparation_cid"]
            or preflight.get("implementation") != implementation
            or started.get("phase") != "preflight"
            or started.get("preparation_cid") != preparation["preparation_cid"]
        ):
            raise ValueError("cached #1047 preflight no longer reproduces")
        return preflight
    if started_path.exists():
        raise FileExistsError("#1047 preflight already started and cannot be rerun")
    _write_exclusive_json(
        started_path,
        _started_record(
            phase="preflight",
            preparation=preparation,
            implementation=implementation,
        ),
    )

    began = time.monotonic()
    device = _configure_cpu(4)
    c0 = _run_c0(source_population, device=device)
    records: list[dict[str, Any]] = []
    if c0["passed"]:
        for threads in ELIGIBLE_THREADS:
            envelope = _spawn_probe(threads, source_root)
            record = envelope.get("record")
            if envelope.get("ok") is True and isinstance(record, Mapping):
                records.append(dict(record))
            else:
                error = envelope.get("error")
                records.append(
                    _failed_probe_record(
                        threads,
                        error if isinstance(error, Mapping) else {"type": "Unknown"},
                    )
                )
    else:
        error = {
            "type": "NotRun",
            "reason": "C0 source/mechanics did not pass",
        }
        records.extend(_failed_probe_record(threads, error) for threads in ELIGIBLE_THREADS)
    selection = select_execution_plan(records)
    if implementation != zoology_control_implementation_contract():
        raise ValueError("#1047 implementation changed during preflight")
    body = {
        "schema": PREFLIGHT_SCHEMA,
        "issue": ISSUE,
        "policy": POLICY,
        "preparation_cid": preparation["preparation_cid"],
        "implementation": implementation,
        "source_attribution_cid": preparation["inputs"][
            "source_attribution_cid"
        ],
        "population_cids": {
            "c1_source_native": source_population.population_cid,
            "c2_exact_1045": exact_population.population_cid,
            "source_1045_split": exact_population.source_split_cid,
        },
        "c0": c0,
        "selection": selection,
        "elapsed_seconds": time.monotonic() - began,
        "passed": bool(c0["passed"] and selection["available"]),
        "read_work_ledger": {
            "failed_source_artifact_reads": 0,
            "sealed_input_reads": 0,
            "provider_calls": 0,
            "teacher_calls": 0,
            "future_value_reads": 0,
            "role_model_input_reads": 0,
            "h4_model_input_reads": 0,
            "cache_reads": 0,
            "transport_reads": 0,
        },
        "cuda": "FORBIDDEN",
        "mps": "FORBIDDEN",
    }
    preflight = _with_cid(body, "preflight_cid")
    _write_exclusive_json(path, preflight)
    return preflight


def _fit_rates(fit: Mapping[str, Any]) -> tuple[float | None, float | None, int]:
    train = fit.get("final_train")
    development = fit.get("final_development")
    train_rate = (
        float(train["top1_rate"])
        if isinstance(train, Mapping) and isinstance(train.get("top1_rate"), (int, float))
        else None
    )
    development_rate = (
        float(development["top1_rate"])
        if isinstance(development, Mapping)
        and isinstance(development.get("top1_rate"), (int, float))
        else None
    )
    return train_rate, development_rate, int(fit.get("consecutive_passes", 0))


def _incomplete_decision(reason: str) -> dict[str, Any]:
    return {
        "status": "INCOMPLETE_HARD_WALL",
        "verdict": None,
        "passed": False,
        "gates": {},
        "thresholds": {
            "hard_wall_seconds": HARD_WALL_SECONDS,
        },
        "action": "stop without scientific inference or same-issue tuning",
        "reason": reason,
    }


def _has_hard_wall_evidence(
    c1: Mapping[str, Any] | str | None,
    c2: Mapping[str, Any] | str | None,
    binding_control: Mapping[str, Any] | str | None,
    elapsed_seconds: float,
) -> bool:
    """Recognize every explicit hard-wall marker emitted by the runner."""

    def marked(value: Mapping[str, Any] | str | None) -> bool:
        if isinstance(value, Mapping):
            return value.get("status") == "INCOMPLETE_HARD_WALL"
        return value in {"INCOMPLETE_HARD_WALL", "NOT_RUN_HARD_WALL"}

    return bool(
        marked(c1)
        or marked(c2)
        or marked(binding_control)
        or elapsed_seconds >= HARD_WALL_SECONDS
    )


def _validate_hard_wall_decision(
    decision: Mapping[str, Any],
    *,
    c1: Mapping[str, Any] | str | None,
    c2: Mapping[str, Any] | str | None,
    binding_control: Mapping[str, Any] | str | None,
    elapsed_seconds: float,
) -> bool:
    """Validate the hard-wall ledger and report whether execution was incomplete."""

    evidence = _has_hard_wall_evidence(
        c1,
        c2,
        binding_control,
        elapsed_seconds,
    )
    if decision.get("status") == "INCOMPLETE_HARD_WALL":
        reason = decision.get("reason")
        if (
            not isinstance(reason, str)
            or not reason.strip()
            or not evidence
            or dict(decision) != _incomplete_decision(reason)
        ):
            raise ValueError("#1047 incomplete result lacks hard-wall evidence")
        return True
    if evidence:
        raise ValueError("#1047 hard-wall result has a scientific verdict")
    return False


def _finish_result(
    root: Path,
    *,
    preparation: Mapping[str, Any],
    preflight: Mapping[str, Any],
    plan: Mapping[str, Any] | None,
    c1: Mapping[str, Any] | str,
    c2: Mapping[str, Any] | str,
    binding_control: Mapping[str, Any] | str,
    artifacts: Sequence[Mapping[str, Any]],
    decision: Mapping[str, Any],
    elapsed_seconds: float,
) -> dict[str, Any]:
    body = _result_body(
        preparation=preparation,
        preflight=preflight,
        plan=plan,
        c1=c1,
        c2=c2,
        binding_control=binding_control,
        artifacts=artifacts,
        decision=decision,
        elapsed_seconds=elapsed_seconds,
    )
    result = _with_cid(body, "result_cid")
    _write_exclusive_json(root / RESULT_RELATIVE_PATH, result)
    return result


def run_zoology_control(root: Path) -> dict[str, Any]:
    """Execute C1 then C2 once, stopping at the first frozen miss."""

    root = root.resolve()
    result_path = root / RESULT_RELATIVE_PATH
    if result_path.exists():
        return verify_zoology_control(root)
    preparation = _read_json(
        root / PREPARATION_RELATIVE_PATH,
        cid_field="preparation_cid",
    )
    preflight = _read_json(
        root / PREFLIGHT_RELATIVE_PATH,
        cid_field="preflight_cid",
    )
    implementation = zoology_control_implementation_contract()
    if (
        preflight.get("preparation_cid") != preparation["preparation_cid"]
        or preflight.get("implementation") != implementation
        or preparation.get("implementation") != implementation
    ):
        raise ValueError("#1047 implementation/preflight binding differs")
    source_root = Path(str(preparation["source_root"]))
    predecessor_root = Path(str(preparation["predecessor_root"]))
    source_population, exact_population = _load_bound_populations(source_root)
    if preparation.get("predecessor") != _bind_predecessor(predecessor_root):
        raise ValueError("#1045 predecessor changed before execution")
    if preparation.get("capacity_predecessor") != _capacity_predecessor_record():
        raise ValueError("#1047 capacity predecessor changed before execution")
    if preflight.get("population_cids") != {
        "c1_source_native": source_population.population_cid,
        "c2_exact_1045": exact_population.population_cid,
        "source_1045_split": exact_population.source_split_cid,
    }:
        raise ValueError("#1047 preflight populations changed before execution")

    started_path = root / RUN_STARTED_RELATIVE_PATH
    if started_path.exists():
        raise FileExistsError("#1047 run already started and cannot be rerun")
    _write_exclusive_json(
        started_path,
        _started_record(
            phase="run",
            preparation=preparation,
            implementation=implementation,
        ),
    )

    selection = preflight.get("selection")
    selected_plan = (
        selection.get("selected_plan") if isinstance(selection, Mapping) else None
    )
    c0_passed = bool(
        isinstance(preflight.get("c0"), Mapping)
        and preflight["c0"].get("passed") is True
    )
    if not c0_passed:
        marker = _preflight_stop_marker(False)
        decision = decide_zoology_control(
            c0_passed=False,
            preflight_available=False,
        )
        return _finish_result(
            root,
            preparation=preparation,
            preflight=preflight,
            plan=None,
            c1=marker,
            c2=marker,
            binding_control=marker,
            artifacts=(),
            decision=decision,
            elapsed_seconds=0.0,
        )
    if preflight.get("passed") is not True or not isinstance(selected_plan, Mapping):
        marker = _preflight_stop_marker(True)
        decision = decide_zoology_control(
            c0_passed=True,
            preflight_available=False,
        )
        return _finish_result(
            root,
            preparation=preparation,
            preflight=preflight,
            plan=None,
            c1=marker,
            c2=marker,
            binding_control=marker,
            artifacts=(),
            decision=decision,
            elapsed_seconds=0.0,
        )

    plan = ExecutionPlan(int(selected_plan["threads"])).record()
    if plan != dict(selected_plan):
        raise ValueError("#1047 selected CPU plan does not reproduce")
    device = _configure_cpu(int(plan["threads"]))
    began = time.monotonic()
    deadline = began + HARD_WALL_SECONDS
    artifacts: list[dict[str, Any]] = []

    c1_model, c1 = _train_rung(
        source_population,
        rung="c1",
        device=device,
        deadline=deadline,
    )
    artifacts.append(
        _write_model_artifact(
            root,
            c1_model,
            rung="c1",
            population_cid=source_population.population_cid,
        )
    )
    if c1["status"] == "INCOMPLETE_HARD_WALL" or time.monotonic() >= deadline:
        decision = _incomplete_decision(
            str(c1.get("incomplete_reason") or "C1/export reached the hard wall")
        )
        return _finish_result(
            root,
            preparation=preparation,
            preflight=preflight,
            plan=plan,
            c1=c1,
            c2="NOT_RUN_HARD_WALL",
            binding_control="NOT_RUN_HARD_WALL",
            artifacts=artifacts,
            decision=decision,
            elapsed_seconds=time.monotonic() - began,
        )
    c1_train_rate, c1_development_rate, c1_consecutive = _fit_rates(c1)
    if c1.get("passed") is not True:
        decision = decide_zoology_control(
            c0_passed=True,
            preflight_available=True,
            c1_train_rate=c1_train_rate,
            c1_development_rate=c1_development_rate,
            c1_consecutive_passes=c1_consecutive,
        )
        return _finish_result(
            root,
            preparation=preparation,
            preflight=preflight,
            plan=plan,
            c1=c1,
            c2="NOT_RUN_C1_MISS",
            binding_control="NOT_RUN_C1_MISS",
            artifacts=artifacts,
            decision=decision,
            elapsed_seconds=time.monotonic() - began,
        )

    c2_model, c2 = _train_rung(
        exact_population,
        rung="c2",
        device=device,
        deadline=deadline,
    )
    artifacts.append(
        _write_model_artifact(
            root,
            c2_model,
            rung="c2",
            population_cid=exact_population.population_cid,
        )
    )
    if c2["status"] == "INCOMPLETE_HARD_WALL" or time.monotonic() >= deadline:
        decision = _incomplete_decision(
            str(c2.get("incomplete_reason") or "C2/export reached the hard wall")
        )
        return _finish_result(
            root,
            preparation=preparation,
            preflight=preflight,
            plan=plan,
            c1=c1,
            c2=c2,
            binding_control="NOT_RUN_HARD_WALL",
            artifacts=artifacts,
            decision=decision,
            elapsed_seconds=time.monotonic() - began,
        )
    c2_train_rate, c2_development_rate, c2_consecutive = _fit_rates(c2)
    binding: dict[str, Any] | str = "NOT_RUN_C2_PRIMARY_MISS"
    binding_rate: float | None = None
    if c2.get("passed") is True:
        try:
            permuted_rows = permute_exact_bindings(exact_population.development)
            permuted = _score_rows(
                c2_model,
                permuted_rows,
                device=device,
                deadline=deadline,
            )
            binding_rate = permuted.rate
            binding = {
                "status": "COMPLETE",
                "source_population_cid": exact_population.population_cid,
                "rows": len(permuted_rows),
                "native_development_rate": c2_development_rate,
                "permuted": permuted.record(),
                "drop": float(c2_development_rate) - binding_rate,
                "required_drop": CONTROL_REQUIRED_DROP,
            }
        except HardWallExceeded as error:
            decision = _incomplete_decision(str(error))
            return _finish_result(
                root,
                preparation=preparation,
                preflight=preflight,
                plan=plan,
                c1=c1,
                c2=c2,
                binding_control="INCOMPLETE_HARD_WALL",
                artifacts=artifacts,
                decision=decision,
                elapsed_seconds=time.monotonic() - began,
            )
    decision = decide_zoology_control(
        c0_passed=True,
        preflight_available=True,
        c1_train_rate=c1_train_rate,
        c1_development_rate=c1_development_rate,
        c1_consecutive_passes=c1_consecutive,
        c2_train_rate=c2_train_rate,
        c2_development_rate=c2_development_rate,
        c2_consecutive_passes=c2_consecutive,
        binding_permuted_rate=binding_rate,
    )
    elapsed = time.monotonic() - began
    if elapsed >= HARD_WALL_SECONDS:
        decision = _incomplete_decision("final scoring reached the hard wall")
    if implementation != zoology_control_implementation_contract():
        raise ValueError("#1047 implementation changed during execution")
    return _finish_result(
        root,
        preparation=preparation,
        preflight=preflight,
        plan=plan,
        c1=c1,
        c2=c2,
        binding_control=binding,
        artifacts=artifacts,
        decision=decision,
        elapsed_seconds=elapsed,
    )


def _verify_artifact(
    root: Path,
    record: Mapping[str, Any],
    *,
    population: ZoologyMQARPopulation,
) -> None:
    path = root / str(record.get("path"))
    payload = path.read_bytes()
    if (
        len(payload) != record.get("bytes")
        or cid_bytes(payload) != record.get("cid")
        or record.get("population_cid") != population.population_cid
    ):
        raise ValueError("#1047 artifact bytes/CID differ")
    tensors = load_safetensors(payload)
    if (
        "lm_head.weight" in tensors
        or "backbone.embeddings.word_embeddings.weight" not in tensors
        or _tensor_mapping_cid(tensors) != record.get("state_cid")
    ):
        raise ValueError("#1047 deterministic tied-artifact structure differs")
    expected_model = _new_model(population, device=torch.device("cpu"))
    expected = {
        name: tensor
        for name, tensor in expected_model.state_dict().items()
        if name != "lm_head.weight"
    }
    if set(tensors) != set(expected) or any(
        tensors[name].shape != expected[name].shape
        or tensors[name].dtype != expected[name].dtype
        for name in expected
    ):
        raise ValueError("#1047 artifact tensor schema differs")


def _validate_execution_policy_envelope(
    result: Mapping[str, Any],
    *,
    preflight: Mapping[str, Any],
    selection: Mapping[str, Any] | object,
    preflight_started: Mapping[str, Any],
    run_started: Mapping[str, Any],
    preparation: Mapping[str, Any],
    implementation: Mapping[str, Any],
) -> None:
    """Bind the V2 wall, selected CPU plan, and both create-once starts."""

    if not isinstance(selection, Mapping):
        raise ValueError("#1049 execution selection is malformed")
    expected_plan = selection.get("selected_plan")
    c0 = preflight.get("c0")
    expected_preflight_passed = bool(
        isinstance(c0, Mapping)
        and c0.get("passed") is True
        and selection.get("available") is True
    )
    if (
        preparation.get("schema") != PREPARATION_SCHEMA
        or preparation.get("issue") != ISSUE
        or preparation.get("policy") != POLICY
        or preflight.get("schema") != PREFLIGHT_SCHEMA
        or preflight.get("issue") != ISSUE
        or preflight.get("policy") != POLICY
        or preflight.get("preparation_cid") != preparation.get("preparation_cid")
        or preflight.get("passed") is not expected_preflight_passed
        or result.get("hard_wall_seconds") != HARD_WALL_SECONDS
        or result.get("plan") != expected_plan
        or dict(preflight_started)
        != _started_record(
            phase="preflight",
            preparation=preparation,
            implementation=implementation,
        )
        or dict(run_started)
        != _started_record(
            phase="run",
            preparation=preparation,
            implementation=implementation,
        )
    ):
        raise ValueError("#1049 execution policy envelope differs")


def verify_zoology_control(root: Path) -> dict[str, Any]:
    """Verify envelopes, CIDs, ledgers, decisions, and artifacts without rescore."""

    root = root.resolve()
    preparation = _read_json(
        root / PREPARATION_RELATIVE_PATH,
        cid_field="preparation_cid",
    )
    preflight = _read_json(
        root / PREFLIGHT_RELATIVE_PATH,
        cid_field="preflight_cid",
    )
    result = _read_json(root / RESULT_RELATIVE_PATH, cid_field="result_cid")
    preflight_started = _read_json(
        root / PREFLIGHT_STARTED_RELATIVE_PATH,
        cid_field="started_cid",
    )
    run_started = _read_json(
        root / RUN_STARTED_RELATIVE_PATH,
        cid_field="started_cid",
    )
    implementation = zoology_control_implementation_contract()
    selection = preflight.get("selection")
    _validate_execution_policy_envelope(
        result,
        preflight=preflight,
        selection=selection,
        preflight_started=preflight_started,
        run_started=run_started,
        preparation=preparation,
        implementation=implementation,
    )
    if (
        result.get("schema") != RESULT_SCHEMA
        or result.get("issue") != ISSUE
        or result.get("policy") != POLICY
        or result.get("preparation_cid") != preparation["preparation_cid"]
        or result.get("preflight_cid") != preflight["preflight_cid"]
        or result.get("implementation") != implementation
        or preflight.get("implementation") != implementation
        or preparation.get("implementation") != implementation
        or preflight_started.get("phase") != "preflight"
        or run_started.get("phase") != "run"
    ):
        raise ValueError("#1047 lifecycle envelope differs")
    source_root = Path(str(preparation["source_root"]))
    predecessor_root = Path(str(preparation["predecessor_root"]))
    source_population, exact_population = _load_bound_populations(source_root)
    if preparation.get("predecessor") != _bind_predecessor(predecessor_root):
        raise ValueError("#1045 predecessor changed after result")
    if preparation.get("capacity_predecessor") != _capacity_predecessor_record():
        raise ValueError("#1047 capacity predecessor changed after result")
    if not isinstance(selection, Mapping) or select_execution_plan(
        selection.get("plans", [])
    ) != selection:
        raise ValueError("#1047 preflight selection does not reproduce")
    ledger = result.get("read_work_ledger")
    if not isinstance(ledger, Mapping) or any(int(value) != 0 for value in ledger.values()):
        raise ValueError("#1047 result reports a forbidden read")

    artifacts = result.get("artifacts")
    if not isinstance(artifacts, list) or len(artifacts) > 2:
        raise ValueError("#1047 artifact list is malformed")
    expected_by_rung = {"c1": source_population, "c2": exact_population}
    observed_rungs: set[str] = set()
    for record in artifacts:
        if not isinstance(record, Mapping):
            raise ValueError("#1047 artifact record is malformed")
        rung = str(record.get("rung"))
        if rung in observed_rungs or rung not in expected_by_rung:
            raise ValueError("#1047 artifact rung is repeated or unknown")
        observed_rungs.add(rung)
        _verify_artifact(root, record, population=expected_by_rung[rung])

    decision = result.get("decision")
    rungs = result.get("rungs")
    if not isinstance(decision, Mapping) or not isinstance(rungs, Mapping):
        raise ValueError("#1047 decision/rung sections are malformed")
    verdict = decision.get("verdict")
    allowed: set[str | None] = {
        None,
        "INVALID_CONTROL_PORT",
        "SCALED_SOURCE_CALIBRATION_MISS",
        "STOCK_CELL_EXACT_QUALIFICATION_MISS",
        "STOCK_CELL_TRANSFER_MISS",
        "NONASSOCIATIVE_SHORTCUT",
        "STOCK_CELL_PASSES_EXACT_BYTES",
    }
    if verdict not in allowed:
        raise ValueError("#1047 result invented a decision code")
    elapsed = result.get("elapsed_seconds")
    if (
        isinstance(elapsed, bool)
        or not isinstance(elapsed, (int, float))
        or not math.isfinite(float(elapsed))
        or float(elapsed) < 0.0
    ):
        raise ValueError("#1047 hard-wall ledger differs")
    elapsed_seconds = float(elapsed)
    c1 = rungs.get("c1")
    c2 = rungs.get("c2")
    binding = result.get("binding_permuted_control")
    reached_rungs = {
        name for name, value in (("c1", c1), ("c2", c2)) if isinstance(value, Mapping)
    }
    if reached_rungs != observed_rungs:
        raise ValueError("#1049 artifacts do not match the reached rungs")
    preflight_passed = preflight.get("passed") is True
    c0 = preflight.get("c0")
    c0_passed = isinstance(c0, Mapping) and c0.get("passed") is True
    if preflight_passed and not isinstance(c1, Mapping):
        raise ValueError("#1049 admitted execution lacks its C1 record")
    if not preflight_passed:
        marker = _preflight_stop_marker(c0_passed)
        expected_decision = decide_zoology_control(
            c0_passed=c0_passed,
            preflight_available=False,
        )
        if (
            reached_rungs
            or artifacts
            or c1 != marker
            or c2 != marker
            or binding != marker
            or dict(decision) != expected_decision
        ):
            raise ValueError("#1049 preflight stop does not reproduce")
    incomplete = _validate_hard_wall_decision(
        decision,
        c1=c1,
        c2=c2,
        binding_control=binding,
        elapsed_seconds=elapsed_seconds,
    )
    if not incomplete and isinstance(c1, Mapping):
        c1_train, c1_development, c1_consecutive = _fit_rates(c1)
        if isinstance(c2, Mapping):
            c2_train, c2_development, c2_consecutive = _fit_rates(c2)
        else:
            c2_train = c2_development = None
            c2_consecutive = 0
        binding_rate = (
            float(binding["permuted"]["top1_rate"])
            if isinstance(binding, Mapping)
            and isinstance(binding.get("permuted"), Mapping)
            else None
        )
        expected_decision = decide_zoology_control(
            c0_passed=True,
            preflight_available=True,
            c1_train_rate=c1_train,
            c1_development_rate=c1_development,
            c1_consecutive_passes=c1_consecutive,
            c2_train_rate=c2_train,
            c2_development_rate=c2_development,
            c2_consecutive_passes=c2_consecutive,
            binding_permuted_rate=binding_rate,
        )
        if dict(decision) != expected_decision:
            raise ValueError("#1047 final decision does not reproduce")
    return result


def execute_zoology_control(
    root: Path,
    *,
    source_root: Path,
    predecessor_root: Path,
) -> dict[str, Any]:
    """Prepare, preflight, run, and structurally verify the open control."""

    prepare_zoology_control(
        root,
        source_root=source_root,
        predecessor_root=predecessor_root,
    )
    preflight_zoology_control(root)
    run_zoology_control(root)
    return verify_zoology_control(root)


__all__ = [
    "ExecutionPlan",
    "decide_zoology_control",
    "execute_zoology_control",
    "prepare_zoology_control",
    "preflight_zoology_control",
    "run_zoology_control",
    "select_execution_plan",
    "verify_zoology_control",
]
