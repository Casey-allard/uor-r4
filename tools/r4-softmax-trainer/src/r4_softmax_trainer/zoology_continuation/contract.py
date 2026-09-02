# SPDX-License-Identifier: Apache-2.0
"""Immutable parent-state binding for the single authorized #1057 continuation."""

from __future__ import annotations

import io
import math
from collections.abc import Mapping
from pathlib import Path
from typing import Any

import torch
from blake3 import blake3
from safetensors.torch import load as load_safetensors
from torch import Tensor

from ..provenance import canonical_json_bytes, cid_bytes
from ..zoology_clock import contract as parent_contract
from ..zoology_clock import development as parent_development

ISSUE = 1057
POLICY = "ZoologyCheckpointContinuationV1"
PREPARATION_PATH = "preparation.json"
PREPARATION_SCHEMA = "uor-r4.zoology-continuation-preparation/1"
PARENT_RESULT_CID = (
    "blake3:3cb810f09a118cfb70752643f5d9e60d0e42780dc6e47dc4f99224cbd69af0ee"
)
PARENT_PREPARATION_CID = (
    "blake3:935363a19038bb9573cda29c29179f98b4b2a80f4d2e0ac9b64b46ae399f5916"
)
PARENT_PREFLIGHT_CID = (
    "blake3:9f184ae09fea32d6a797303a3770ad10f5603ec4ac86bd2010bfc063ce39cdf6"
)
PARENT_IMPLEMENTATION_CID = (
    "blake3:bf45a18a2b4ed1aee607220f8d0331c32254bb43e94c2e1d7bece70d33634de3"
)
PARENT_ARTIFACT_CID = (
    "blake3:2a225b691ffde7b40afd41ac888c5a4449a6b3ba48c2773a2108e1e407d6f8b4"
)
PARENT_CHECKPOINT = {
    "path": "primary/checkpoint.pt",
    "cid": "blake3:e41064d74f31d45b51ecf07590ef1ae12e3f1efddce535e92b8b1b1eabd15386",
    "bytes": 3_753_269,
}
PARENT_UPDATES = 3_920
PARENT_BLOCKS = 20
ADDITIONAL_UPDATES = 3_920
ADDITIONAL_BLOCKS = 20
TOTAL_UPDATES = 7_840
TOTAL_BLOCKS = 40
ADDITIONAL_BUDGET_SECONDS = 1_800.0
MEMORY_CEILING_BYTES = 8 * 1024**3
CPU_PLAN = {
    "batch_size": 512,
    "device": "cpu",
    "interop_threads": 1,
    "threads": 8,
    "workers": 1,
}


def training_contract() -> dict[str, Any]:
    inherited = parent_contract.training_contract()
    return {
        **inherited,
        "initialization": "restore_exact_parent_checkpoint_no_new_fit_or_reset",
        "parent_optimizer_updates": PARENT_UPDATES,
        "parent_source_blocks": PARENT_BLOCKS,
        "maximum_additional_optimizer_updates": ADDITIONAL_UPDATES,
        "maximum_additional_source_blocks": ADDITIONAL_BLOCKS,
        "maximum_optimizer_updates": TOTAL_UPDATES,
        "maximum_source_blocks": TOTAL_BLOCKS,
        "maximum_additional_train_query_presentations": ADDITIONAL_UPDATES * 512 * 8,
        "maximum_additional_development_query_presentations": ADDITIONAL_BLOCKS
        * 1024
        * 8,
        "maximum_train_query_presentations": TOTAL_UPDATES * 512 * 8,
        "full_run_complete_training_permutations": 490,
        "source_full_rng_trajectory_equivalence": "NOT_CLAIMED_RESTORE_EXACT_PARENT_RNG",
        "continuation_state": [
            "model",
            "optimizer",
            "scheduler",
            "sampler",
            "torch_rng_state",
            "evaluation_rng",
        ],
        "parent_history": "first_20_entries_unchanged",
        "new_learning_rate_or_scheduler_reset": "FORBIDDEN",
        "checkpoint_interval_updates": 16,
        "additional_budget_seconds": ADDITIONAL_BUDGET_SECONDS,
        "memory_ceiling_bytes": MEMORY_CEILING_BYTES,
        "budget_recovery": "accumulate_additional_elapsed_never_reset_on_restart",
        "automatic_further_extension": "FORBIDDEN",
    }


def implementation_contract(trainer_root: Path | None = None) -> dict[str, Any]:
    trainer_root = (
        Path(__file__).resolve().parents[3]
        if trainer_root is None
        else trainer_root.resolve()
    )
    inherited = parent_contract.implementation_contract(trainer_root)
    if inherited["implementation_cid"] != PARENT_IMPLEMENTATION_CID:
        raise ValueError("immutable #1055 implementation or dependencies changed")
    package = trainer_root / "src/r4_softmax_trainer/zoology_continuation"
    sources = set(package.rglob("*.py"))
    if not sources:
        raise ValueError("continuation source is absent")
    added = sources | set(
        (trainer_root / "tests").glob("test_zoology_continuation*.py")
    )
    inherited_paths = {trainer_root / record["path"] for record in inherited["files"]}
    parent_contract._require_imports_bound(
        trainer_root, sources, inherited_paths | sources
    )
    records = list(inherited["files"])
    for path in sorted(added):
        if path.is_symlink():
            raise ValueError("continuation source cannot be a symlink")
        payload = path.read_bytes()
        records.append(
            {
                "path": str(path.relative_to(trainer_root)),
                "bytes": len(payload),
                "cid": cid_bytes(payload),
            }
        )
    records.sort(key=lambda record: record["path"])
    if len({record["path"] for record in records}) != len(records):
        raise ValueError("continuation source closure has duplicate paths")
    digest = blake3()
    for record in records:
        digest.update(canonical_json_bytes(record))
    return parent_contract.previous.release._with_cid(
        {
            "schema": "uor-r4/zoology-continuation-implementation/v1",
            "issue": ISSUE,
            "policy": POLICY,
            "parent_implementation_cid": inherited["implementation_cid"],
            "inherited_file_count": len(inherited["files"]),
            "new_file_count": len(added),
            "files": records,
            "tree_cid": f"blake3:{digest.hexdigest()}",
        },
        "implementation_cid",
    )


def _root(value: str | Path) -> Path:
    root = Path(value)
    if not root.is_absolute() or root.is_symlink():
        raise ValueError("parent/data root must be absolute and not a symlink")
    return root


def _read_record(root: Path, record: Mapping[str, Any]) -> bytes:
    path = parent_contract.previous._safe_path(root, str(record["path"]))
    payload = path.read_bytes()
    if cid_bytes(payload) != record["cid"] or (
        "bytes" in record and len(payload) != record["bytes"]
    ):
        raise ValueError(f"parent payload changed: {record['path']}")
    return payload


def _validate_checkpoint_state(
    saved: Mapping[str, Any], preparation: Mapping[str, Any]
) -> None:
    """Validate inherited mechanics directly, without creating or evaluating a model."""

    primary = preparation["parent_primary"]
    history = primary["history"]
    if (
        saved["binding_cid"] != preparation["parent_binding_cid"]
        or saved["model_config"] != primary["artifact"]["config"]
        or saved["history"] != history
        or cid_bytes(canonical_json_bytes(history)) != preparation["parent_history_cid"]
        or len(history) != PARENT_BLOCKS
        or saved["completed_updates"] != PARENT_UPDATES
        or parent_development._history_pass(history)
        or saved["accumulator"]
        != {"updates": 0, "decisions": 0, "correct": 0, "loss_sum": 0.0}
    ):
        raise ValueError("parent checkpoint binding/history/update/accumulator differs")
    if any(
        row["train"]["updates"] != 196
        or row["train"]["decisions"] != 196 * 512 * 8
        or row["development"]["decisions"] != 8192
        for row in history
    ):
        raise ValueError("parent checkpoint history work differs")
    scheduler = saved["scheduler"]
    if (
        scheduler["last_epoch"] != 20
        or scheduler["T_max"] != 64
        or scheduler["eta_min"] != 0
    ):
        raise ValueError("parent scheduler is not at the frozen block-20 clock")
    expected_rate = (
        parent_development.LEARNING_RATE * (1 + math.cos(math.pi * 20 / 64)) / 2
    )
    groups = saved["optimizer"]["param_groups"]
    states = saved["optimizer"]["state"]
    if (
        len(groups) != 1
        or not states
        or any(int(state["step"]) != PARENT_UPDATES for state in states.values())
    ):
        raise ValueError("parent optimizer counters differ")
    if not math.isclose(groups[0]["lr"], expected_rate, rel_tol=1e-14, abs_tol=1e-18):
        raise ValueError("parent optimizer learning rate differs")
    sampler = saved["sampler"]
    if (
        sampler["cycles"] != 245
        or sampler["cursor"] != 8192
        or not torch.equal(
            sampler["permutation"].sort().values, torch.arange(8192, dtype=torch.long)
        )
    ):
        raise ValueError("parent sampler does not retain the exact completed traversal")
    for name in ("torch_rng_state", "evaluation_rng"):
        state = saved[name]
        if (
            state.dtype != torch.uint8
            or state.device.type != "cpu"
            or state.shape != torch.get_rng_state().shape
        ):
            raise ValueError("parent RNG state shape/type differs")
    state = {
        name: value
        for name, value in saved["model"].items()
        if name != "lm_head.weight"
    }
    if (
        len(state) != 20
        or parent_contract.previous.release._tensor_mapping_cid(state)
        != primary["artifact"]["state_cid"]
    ):
        raise ValueError(
            "parent checkpoint model state differs from its artifact identity"
        )
    if not torch.equal(
        saved["model"]["lm_head.weight"],
        saved["model"]["backbone.embeddings.word_embeddings.weight"],
    ):
        raise ValueError("parent tied vocabulary head differs")


def load_checkpoint(preparation: Mapping[str, Any]) -> dict[str, Any]:
    if preparation["parent_checkpoint"] != PARENT_CHECKPOINT:
        raise ValueError("parent checkpoint record differs from the frozen identity")
    root = _root(preparation["parent_root"])
    payload = _read_record(root, preparation["parent_checkpoint"])
    saved = torch.load(io.BytesIO(payload), map_location="cpu", weights_only=True)
    _validate_checkpoint_state(saved, preparation)
    artifact = load_safetensors(
        _read_record(root, preparation["parent_primary"]["artifact"])
    )
    state = {
        name: value
        for name, value in saved["model"].items()
        if name != "lm_head.weight"
    }
    if set(artifact) != set(state) or any(
        not torch.equal(artifact[name], tensor) for name, tensor in state.items()
    ):
        raise ValueError("parent checkpoint tensors differ from the published artifact")
    rng = load_safetensors(
        _read_record(root, preparation["parent_primary"]["evaluation_rng"])
    )
    if not torch.equal(rng["evaluation_rng"], saved["evaluation_rng"]):
        raise ValueError("parent evaluation RNG differs from its published artifact")
    return saved


def _bind_parent(parent_root: Path) -> dict[str, Any]:
    release = parent_contract.previous.release
    parent = parent_contract.validate_preparation(parent_root)
    preflight = release._read_json(
        parent_contract.previous._safe_path(
            parent_root, parent_development.PREFLIGHT_PATH
        ),
        cid_field="preflight_cid",
    )
    result = release._read_json(
        parent_contract.previous._safe_path(
            parent_root, parent_development.RESULT_PATH
        ),
        cid_field="result_cid",
    )
    primary = release._read_json(
        parent_contract.previous._safe_path(parent_root, "primary/result.json"),
        cid_field="primary_cid",
    )
    if (
        parent["preparation_cid"] != PARENT_PREPARATION_CID
        or parent["implementation"]["implementation_cid"] != PARENT_IMPLEMENTATION_CID
        or preflight["preflight_cid"] != PARENT_PREFLIGHT_CID
        or result["result_cid"] != PARENT_RESULT_CID
        or result["preparation_cid"] != preflight["preparation_cid"]
        or result["preparation_cid"] != parent["preparation_cid"]
        or result["preflight_cid"] != preflight["preflight_cid"]
        or result["implementation"] != preflight["implementation"]
        or result["implementation"] != parent["implementation"]
        or result["primary"] != primary
        or result["decision"]["verdict"] != "CLOCK_MATCHED_TRANSFER_MISS"
        or primary["status"] != "CLOCK_MATCHED_TRANSFER_MISS"
        or primary["passed"]
        or primary["blocks"] != 20
        or primary["completed_updates"] != 3920
        or result["control"] != {"status": "NOT_RUN_PRIMARY_MISS"}
        or result["read_ledger"]["control_query_decisions"] != 0
        or primary["artifact"]["cid"] != PARENT_ARTIFACT_CID
        or primary["binding_cid"] != parent_development._binding(parent, preflight)
    ):
        raise ValueError("parent terminal/preparation/preflight identity differs")
    selected = preflight["selected"]
    projected = 1.25 * float(primary["elapsed_seconds"])
    if (
        selected["plan"] != CPU_PLAN
        or not selected["stable"]
        or not selected["repeat_deterministic"]
        or "BLAS_INFO=accelerate" not in selected["torch_config"]
        or selected["torch_config"] != torch.__config__.show()
        or not math.isfinite(projected)
        or not 0 < projected <= ADDITIONAL_BUDGET_SECONDS
        or primary["peak_rss_bytes"] > MEMORY_CEILING_BYTES
    ):
        raise ValueError(
            "recent same-shape CPU execution does not admit the continuation"
        )
    bound = {
        "parent_root": str(parent_root),
        "parent_result_cid": result["result_cid"],
        "parent_preparation_cid": parent["preparation_cid"],
        "parent_preflight_cid": preflight["preflight_cid"],
        "parent_primary": primary,
        "parent_checkpoint": dict(PARENT_CHECKPOINT),
        "parent_binding_cid": primary["binding_cid"],
        "parent_history_cid": cid_bytes(canonical_json_bytes(primary["history"])),
        "dataset_root": parent["predecessor_root"],
        "dataset": parent["dataset"],
        "control": parent["control"],
        "cpu_plan": dict(CPU_PLAN),
        "admission": {
            "passed": True,
            "cpu_plan": dict(CPU_PLAN),
            "parent_primary_elapsed_seconds": primary["elapsed_seconds"],
            "safety_factor": 1.25,
            "projected_additional_seconds": projected,
            "additional_budget_seconds": ADDITIONAL_BUDGET_SECONDS,
            "memory_ceiling_bytes": MEMORY_CEILING_BYTES,
            "reused_preflight": selected,
            "new_benchmark_runs": 0,
            "c0_runs": 0,
            "cuda": "FORBIDDEN",
            "mps": "FORBIDDEN",
        },
    }
    load_checkpoint(bound)
    return bound


def load_dataset(preparation: Mapping[str, Any]) -> dict[str, Tensor]:
    return parent_contract.previous.load_dataset(
        _root(preparation["dataset_root"]), preparation
    )


def load_control(preparation: Mapping[str, Any]) -> dict[str, Tensor]:
    return parent_contract.previous.load_control(
        _root(preparation["dataset_root"]), preparation
    )


def validate_preparation(root: Path) -> dict[str, Any]:
    preparation = parent_contract.previous.release._read_json(
        parent_contract.previous._safe_path(root, PREPARATION_PATH),
        cid_field="preparation_cid",
    )
    if (
        preparation.get("schema") != PREPARATION_SCHEMA
        or preparation.get("issue") != ISSUE
        or preparation.get("policy") != POLICY
        or preparation.get("training_contract") != training_contract()
        or preparation.get("implementation") != implementation_contract()
    ):
        raise ValueError("continuation preparation/source/training contract differs")
    bound = _bind_parent(_root(preparation["parent_root"]))
    if any(preparation.get(name) != value for name, value in bound.items()):
        raise ValueError("continuation parent/data/admission binding changed")
    return preparation


def prepare(root: Path, parent_root: Path) -> dict[str, Any]:
    path = parent_contract.previous._safe_path(root, PREPARATION_PATH)
    if path.exists():
        raise FileExistsError(f"continuation preparation already exists: {path}")
    parent_root = _root(parent_root)
    bound = _bind_parent(parent_root)
    body = {
        "schema": PREPARATION_SCHEMA,
        "issue": ISSUE,
        "policy": POLICY,
        "implementation": implementation_contract(),
        "training_contract": training_contract(),
        **bound,
        "read_ledger": {
            "parent_checkpoint_payloads_read": 1,
            "parent_artifact_payloads_read": 1,
            "parent_evaluation_rng_payloads_read": 1,
            "parent_model_inferences": 0,
            "source_teacher_weight_reads": 0,
            "new_initializations": 0,
            "new_corpus_payloads": 0,
            "copied_corpus_payloads": 0,
            "model_role_reads": 0,
            "model_geometry_reads": 0,
            "future_value_reads": 0,
            "control_query_decisions": 0,
            "sealed_reads": 0,
            "teacher_calls": 0,
            "provider_calls": 0,
            "c0_training_updates": 0,
            "new_timing_matrix_runs": 0,
        },
    }
    preparation = parent_contract.previous.release._with_cid(body, "preparation_cid")
    parent_contract.previous.release._write_exclusive_json(path, preparation)
    return preparation
