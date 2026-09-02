# SPDX-License-Identifier: Apache-2.0
"""Bind #1055 to the existing #1053 bytes without copying or regenerating data."""

from __future__ import annotations

import ast
from collections.abc import Mapping
from pathlib import Path
from typing import Any

from blake3 import blake3
from torch import Tensor

from ..provenance import canonical_json_bytes, cid_bytes
from ..zoology_transfer import contract as previous
from ..zoology_transfer import development as previous_development

ISSUE = 1055
POLICY = "ZoologyOptimizerClockTransferV1"
PREPARATION_PATH = "preparation.json"
PREPARATION_SCHEMA = "uor-r4.zoology-clock-preparation/1"
EXPECTED_PREPARATION_CID = (
    "blake3:1de63b0377a0f64f3181de37e706306dfaa8242c40f667faf47f4aba812272f2"
)
EXPECTED_PREFLIGHT_CID = (
    "blake3:93f11afcc7734d6305e965646a88f54a6c7d1db11e7d7891f50f1f30845a1cf9"
)
EXPECTED_RESULT_CID = (
    "blake3:e2d1deb55a4612015ba924a94051beacd517f3c062c714c4972ba954f57621a1"
)
EXPECTED_IMPLEMENTATION_CID = (
    "blake3:e01665c060f03cd2dac1dadd8df9b8cfdcfea94d99eab6732a878f7fd97babdf"
)
EXPECTED_C0_CID = (
    "blake3:b6c38a031747c8da07ce6b6ddf487f2091d41d2ecedb96ec5c365c53d0a4a712"
)
SOURCE_BLOCKS = 20
UPDATES_PER_BLOCK = 196
MAXIMUM_UPDATES = SOURCE_BLOCKS * UPDATES_PER_BLOCK


def training_contract() -> dict[str, Any]:
    """Keep #1053's cell and optimizer while replacing only its optimizer clock."""

    inherited = previous.training_contract()
    inherited.pop("maximum_epochs")
    inherited.pop("scheduler")
    inherited.pop("dataloader_rng")
    return {
        **inherited,
        "maximum_source_blocks": SOURCE_BLOCKS,
        "updates_per_source_block": UPDATES_PER_BLOCK,
        "maximum_optimizer_updates": MAXIMUM_UPDATES,
        "maximum_train_query_presentations": MAXIMUM_UPDATES * 512 * 8,
        "scheduler": "CosineAnnealingLR_T_max_64_source_blocks_eta_min_0",
        "evaluation_clock": "after_each_196_optimizer_updates",
        "training_sampler": "cyclic_complete_8192_row_shuffle_permutations",
        "sampler_cursor": "retained_across_block_boundaries_and_checkpoint_resume",
        "all_training_batch_sizes": [512],
        "dataloader_rng": "stock_global_base_seed_and_sampler_seed_draws_per_training_traversal",
        "sampler_rng": "stock_RandomSampler_private_generator_seeded_from_global_rng",
        "development_rng": "same_global_torch_rng_in_shuffled_development_loader",
        "source_full_rng_trajectory_equivalence": "NOT_CLAIMED_245_SMALLER_TRAINING_TRAVERSALS_VERSUS_20_SOURCE_TRAVERSALS",
        "full_run_complete_training_permutations": 245,
        "changed_from_1053": "optimizer_and_scheduler_clock_only",
        "source_clock_reference": {
            "issue": 1050,
            "result_cid": previous.EXPECTED_1050_RESULT_CID,
            "positive_epoch": 20,
            "updates_per_source_epoch": 196,
            "matched_quantity": "optimizer_updates_not_unique_examples_or_query_presentations",
        },
    }


def _require_imports_bound(
    trainer_root: Path, new_sources: set[Path], bound_paths: set[Path]
) -> None:
    """Fail closed if a new clock module imports unbound local implementation."""

    source_root = trainer_root / "src"
    for path in sorted(new_sources):
        package_parts = list(path.relative_to(source_root).parts[:-1])
        tree = ast.parse(path.read_text(encoding="utf-8"), filename=str(path))
        modules: set[str] = set()
        for node in ast.walk(tree):
            if isinstance(node, ast.Import):
                modules.update(alias.name for alias in node.names)
            elif isinstance(node, ast.ImportFrom):
                if node.level:
                    if node.level > len(package_parts):
                        raise ValueError("clock relative import escapes its package")
                    parts = package_parts[: len(package_parts) - node.level + 1]
                    if node.module:
                        parts.extend(node.module.split("."))
                    base = ".".join(parts)
                else:
                    base = node.module or ""
                modules.add(base)
                modules.update(f"{base}.{alias.name}" for alias in node.names)
            elif isinstance(node, ast.Call):
                function = node.func
                name = (
                    function.id
                    if isinstance(function, ast.Name)
                    else function.attr
                    if isinstance(function, ast.Attribute)
                    else ""
                )
                if name in ("__import__", "import_module"):
                    raise ValueError(
                        "dynamic imports are not allowed in the clock source closure"
                    )
        for module in modules:
            imported = previous._local_module_path(source_root, module)
            if imported is not None and imported not in bound_paths:
                raise ValueError(
                    f"clock import is outside its bound source closure: {module}"
                )


def implementation_contract(trainer_root: Path | None = None) -> dict[str, Any]:
    """Extend the unchanged #1053 closure with only this sibling and its tests."""

    trainer_root = (
        Path(__file__).resolve().parents[3]
        if trainer_root is None
        else Path(trainer_root).resolve()
    )
    inherited = previous.implementation_contract(trainer_root)
    if inherited["implementation_cid"] != EXPECTED_IMPLEMENTATION_CID:
        raise ValueError("the inherited #1053 implementation or lockfile changed")
    package = trainer_root / "src/r4_softmax_trainer/zoology_clock"
    new_sources = set(package.rglob("*.py"))
    if not new_sources:
        raise ValueError("clock implementation sources are absent")
    new_paths = new_sources | set(
        (trainer_root / "tests").glob("test_zoology_clock*.py")
    )
    inherited_paths = {trainer_root / record["path"] for record in inherited["files"]}
    _require_imports_bound(trainer_root, new_sources, inherited_paths | new_sources)
    records = list(inherited["files"])
    for path in sorted(new_paths):
        if path.is_symlink():
            raise ValueError("clock implementation inputs cannot be symlinks")
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
        raise ValueError("clock and inherited implementation inputs overlap")
    digest = blake3()
    for record in records:
        digest.update(canonical_json_bytes(record))
    return previous.release._with_cid(
        {
            "schema": "uor-r4/zoology-clock-implementation/v1",
            "issue": ISSUE,
            "policy": POLICY,
            "source_commit": previous.SOURCE_COMMIT,
            "predecessor_implementation_cid": inherited["implementation_cid"],
            "inherited_file_count": len(inherited["files"]),
            "new_file_count": len(new_paths),
            "files": records,
            "tree_cid": f"blake3:{digest.hexdigest()}",
        },
        "implementation_cid",
    )


def _load_predecessor(predecessor_root: Path) -> dict[str, Any]:
    """Read exact old JSON evidence and primary tensors, never old fitted weights."""

    release = previous.release
    preparation = previous.validate_preparation(predecessor_root)
    preflight = release._read_json(
        previous._safe_path(predecessor_root, previous_development.PREFLIGHT_PATH),
        cid_field="preflight_cid",
    )
    result = release._read_json(
        previous._safe_path(predecessor_root, previous_development.RESULT_PATH),
        cid_field="result_cid",
    )
    c0 = preflight.get("c0", {})
    c0_cid = cid_bytes(canonical_json_bytes(c0))
    if (
        preparation.get("preparation_cid") != EXPECTED_PREPARATION_CID
        or preparation.get("implementation", {}).get("implementation_cid")
        != EXPECTED_IMPLEMENTATION_CID
        or preflight.get("preflight_cid") != EXPECTED_PREFLIGHT_CID
        or preflight.get("preparation_cid") != EXPECTED_PREPARATION_CID
        or preflight.get("implementation") != preparation["implementation"]
        or result.get("result_cid") != EXPECTED_RESULT_CID
        or result.get("preparation_cid") != EXPECTED_PREPARATION_CID
        or result.get("preflight_cid") != EXPECTED_PREFLIGHT_CID
        or result.get("implementation") != preparation["implementation"]
        or result.get("dataset") != preparation["dataset"]
        or result.get("decision", {}).get("verdict") != "STOCK_CELL_TRANSFER_MISS"
        or c0_cid != EXPECTED_C0_CID
        or c0.get("passed") is not True
    ):
        raise ValueError(
            "#1053 predecessor evidence differs from the frozen optimizer-clock authority"
        )
    return {
        "dataset": preparation["dataset"],
        "control": preparation["control"],
        "predecessor_root": str(predecessor_root),
        "predecessor_preparation_cid": preparation["preparation_cid"],
        "predecessor_preflight_cid": preflight["preflight_cid"],
        "predecessor_result_cid": result["result_cid"],
        "reused_c0": {
            "status": "REUSED_IDENTICAL_SOURCE_MECHANICS_NO_RERUN",
            "passed": True,
            "c0_cid": c0_cid,
            "preflight_cid": preflight["preflight_cid"],
            "record": c0,
            "reuse_boundary": "copied_cell_and_query_projection_only; new_clock_sampling_checked_separately",
        },
        "source_1050_result_cid": preparation["release_1050"]["result_cid"],
    }


def _bound_predecessor_root(preparation: Mapping[str, Any]) -> Path:
    root = Path(str(preparation["predecessor_root"]))
    if not root.is_absolute() or root.is_symlink():
        raise ValueError("predecessor root must be an absolute nonsymlink path")
    return root


def load_dataset(root: Path, preparation: Mapping[str, Any]) -> dict[str, Tensor]:
    """Reuse existing exact primary tensors in place, without copies on disk."""

    return previous.load_dataset(_bound_predecessor_root(preparation), preparation)


def load_control(root: Path, preparation: Mapping[str, Any]) -> dict[str, Tensor]:
    """Open the old-label control only when the runner has a strict primary pass."""

    return previous.load_control(_bound_predecessor_root(preparation), preparation)


def validate_preparation(root: Path) -> dict[str, Any]:
    preparation = previous.release._read_json(
        previous._safe_path(root, PREPARATION_PATH), cid_field="preparation_cid"
    )
    if (
        preparation.get("schema") != PREPARATION_SCHEMA
        or preparation.get("issue") != ISSUE
        or preparation.get("policy") != POLICY
        or preparation.get("implementation") != implementation_contract()
        or preparation.get("training_contract") != training_contract()
    ):
        raise ValueError(
            "#1055 preparation differs from its current implementation or training contract"
        )
    bound = _load_predecessor(_bound_predecessor_root(preparation))
    if any(preparation.get(name) != value for name, value in bound.items()):
        raise ValueError("#1055 predecessor data/C0/result binding changed")
    return preparation


def prepare(root: Path, predecessor_root: Path) -> dict[str, Any]:
    """Create only a new JSON preparation; source payloads and weights stay put."""

    path = previous._safe_path(root, PREPARATION_PATH)
    if path.exists():
        raise FileExistsError(f"#1055 preparation already exists: {path}")
    if predecessor_root.is_symlink():
        raise ValueError("predecessor root cannot be a symlink")
    predecessor_root = predecessor_root.resolve()
    bound = _load_predecessor(predecessor_root)
    preparation = previous.release._with_cid(
        {
            "schema": PREPARATION_SCHEMA,
            "issue": ISSUE,
            "policy": POLICY,
            "implementation": implementation_contract(),
            **bound,
            "training_contract": training_contract(),
            "read_ledger": {
                "predecessor_json_envelopes_read": 3,
                "existing_primary_tensor_payloads_read": 1,
                "control_tensor_payloads_read": 0,
                "new_corpus_payloads": 0,
                "copied_corpus_payloads": 0,
                "role_payload_reads": 0,
                "role_validation_rows": 0,
                "model_role_reads": 0,
                "model_geometry_reads": 0,
                "future_value_reads": 0,
                "predecessor_weight_reads": 0,
                "sealed_reads": 0,
                "english_payload_reads": 0,
                "natural_payload_reads": 0,
                "teacher_calls": 0,
                "provider_calls": 0,
                "c0_training_updates": 0,
            },
        },
        "preparation_cid",
    )
    previous.release._write_exclusive_json(path, preparation)
    return preparation
