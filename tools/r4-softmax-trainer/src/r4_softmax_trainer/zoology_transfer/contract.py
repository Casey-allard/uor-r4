# SPDX-License-Identifier: Apache-2.0
"""Create-once exact-#1045 MQAR inputs for the #1053 transfer.

Only public envelopes and the already-open MQAR payload are read. The old
broad construction loader is deliberately not called: English, natural text,
tokenizer bytes, sealed populations, and fitted weights stay unopened. Roles
are re-derived solely to validate serialization and construct the frozen
physical-value permutation; neither tensor file contains a role channel.
"""

from __future__ import annotations

import ast
from collections.abc import Mapping
from pathlib import Path
from typing import Any

import torch
from blake3 import blake3
from safetensors.torch import load as load_safetensors
from torch import Tensor

from .. import position_kv_binding_data as position_data
from .. import role_tagged_associative_data as role_data
from ..provenance import canonical_json_bytes, cid_bytes, verify_artifact_subset
from ..zoology_control import data as exact_data
from ..zoology_control import development as control_development
from ..zoology_control.model import SOURCE_COMMIT
from ..zoology_release import development as release

ISSUE = 1053
POLICY = "ZoologyExact1045TransferV1"
PREPARATION_RELATIVE_PATH = "zoology-transfer-preparation.json"
DATA_RELATIVE_PATH = "data/exact-1045.safetensors"
CONTROL_RELATIVE_PATH = "data/binding-permuted.safetensors"
PREPARATION_SCHEMA = "uor-r4.zoology-transfer-preparation/1"
TRAIN_ROWS = 8_192
TEST_ROWS = 1_024
INPUT_SEQ_LEN = 120
NUM_KV_PAIRS = 8
VOCAB_SIZE = 4_096
EXPECTED_1050_RESULT_CID = (
    "blake3:bd16d012c01262ffb8c5197e4cf316c6fee1d722cf0700a0048386180a8122e0"
)
EXPECTED_1043_MANIFEST_CID = (
    "blake3:8b5ace868fa9c81ea2f7ab8066cb29a04f14bbda19b9475b2974a8c8b7475f0d"
)
EXPECTED_1043_COMMITMENT_CID = (
    "blake3:d59943e3b502c64659ddc726c3eedb63ab8cb8f82b6ce4c2eeb42f192bca6626"
)
EXPECTED_1043_MQAR_CID = (
    "blake3:dd7e89a4c41e2431a01fb103240df17429f934f437a02b0e9762a01736d70b2e"
)
EXPECTED_1045_POPULATION_CID = (
    "blake3:54982556bc986ad8aa59bb408945fad85a5990b2afe29eb1d1b11d5db19e44c9"
)


def training_contract() -> dict[str, Any]:
    """The sole independently initialized source-positive training arm."""

    rate = 0.00046415888336127773
    return {
        "vocab_size": VOCAB_SIZE,
        "context": INPUT_SEQ_LEN,
        "query_positions_per_row": NUM_KV_PAIRS,
        "train_rows": TRAIN_ROWS,
        "development_rows": TEST_ROWS,
        "d_model": 64,
        "n_layers": 2,
        "num_heads": 1,
        "attention_dropout": 0.1,
        "embed_dropout": 0.1,
        "resid_dropout": 0.0,
        "state_mixer": "Identity",
        "initialization": "independent_source_seed_123_no_fitted_weights",
        "seed": 123,
        "learning_rate": rate,
        "learning_rate_float_hex": rate.hex(),
        "batch_size": 512,
        "optimizer": "AdamW",
        "weight_decay": 0.1,
        "betas": [0.9, 0.999],
        "epsilon": 1e-8,
        "maximum_epochs": 64,
        "scheduler": "CosineAnnealingLR_T_max_64_eta_min_0",
        "scheduler_step": "after_failed_development_only",
        "train_and_development_shuffle": True,
        "dataloader_rng": "shared_global_torch_rng",
        "dataloader_workers": 0,
        "strict_early_stop": "development_top1_rate > 0.99",
        "control": "one_frozen_physical_value_permutation_after_primary_pass_only",
        "source_adaptations": ["CPU_placement", "query_only_tied_head_projection"],
    }


def _local_module_path(source_root: Path, module: str) -> Path | None:
    if not (module == "r4_softmax_trainer" or module.startswith("r4_softmax_trainer.")):
        return None
    stem = source_root.joinpath(*module.split("."))
    for path in (stem.with_suffix(".py"), stem / "__init__.py"):
        if path.is_file():
            return path
    return None


def _source_closure(trainer_root: Path) -> set[Path]:
    """Bind the complete statically imported local-module closure.

    Every transfer module is an entrypoint. Package initializers and both
    absolute and relative imports are followed, including imports inside
    functions. Third-party implementations are pinned by ``uv.lock``.
    """

    source_root = trainer_root / "src"
    package_root = source_root / "r4_softmax_trainer" / "zoology_transfer"
    pending = list(package_root.rglob("*.py"))
    if not pending:
        raise ValueError("#1053 source entrypoints are absent")
    seen: set[Path] = set()
    while pending:
        path = pending.pop()
        if path in seen:
            continue
        if path.is_symlink():
            raise ValueError("implementation source cannot be a symlink")
        seen.add(path)
        relative = path.relative_to(source_root).with_suffix("")
        module_parts = list(relative.parts)
        package_parts = module_parts[:-1]
        for depth in range(1, len(package_parts) + 1):
            initializer = source_root.joinpath(*package_parts[:depth], "__init__.py")
            if initializer.is_file():
                pending.append(initializer)
        tree = ast.parse(path.read_text(encoding="utf-8"), filename=str(path))
        modules: set[str] = set()
        for node in ast.walk(tree):
            if isinstance(node, ast.Import):
                modules.update(alias.name for alias in node.names)
            elif isinstance(node, ast.ImportFrom):
                if node.level:
                    if node.level > len(package_parts):
                        raise ValueError(f"relative import escapes package in {path}")
                    base_parts = package_parts[: len(package_parts) - node.level + 1]
                    if node.module:
                        base_parts.extend(node.module.split("."))
                    base = ".".join(base_parts)
                else:
                    base = node.module or ""
                modules.add(base)
                modules.update(f"{base}.{alias.name}" for alias in node.names)
        for module in modules:
            imported = _local_module_path(source_root, module)
            if imported is not None:
                pending.append(imported)
    return seen


def implementation_contract(trainer_root: Path | None = None) -> dict[str, Any]:
    """Hash source dependencies, activated tests, attribution, and environment."""

    trainer_root = (
        Path(__file__).resolve().parents[3]
        if trainer_root is None
        else Path(trainer_root).resolve()
    )
    paths = _source_closure(trainer_root)
    for pattern in (
        "test_zoology_transfer*.py",
        "test_zoology_release.py",
        "test_zoology_control_data.py",
        "test_zoology_control_model.py",
        "test_zoology_control_development.py",
        "test_role_tagged_associative_data.py",
        "test_position_kv_binding_data.py",
    ):
        paths.update((trainer_root / "tests").glob(pattern))
    paths.update(
        trainer_root / relative
        for relative in (
            "src/r4_softmax_trainer/zoology_control/NOTICE.md",
            "src/r4_softmax_trainer/zoology_control/LICENSE-APACHE-2.0.md",
            "pyproject.toml",
            "uv.lock",
        )
    )
    digest = blake3()
    records = []
    for path in sorted(paths):
        if path.is_symlink():
            raise ValueError("implementation input cannot be a symlink")
        payload = path.read_bytes()
        record = {
            "path": str(path.relative_to(trainer_root)),
            "bytes": len(payload),
            "cid": cid_bytes(payload),
        }
        records.append(record)
        digest.update(canonical_json_bytes(record))
    return release._with_cid(
        {
            "schema": "uor-r4/zoology-transfer-implementation/v1",
            "issue": ISSUE,
            "policy": POLICY,
            "source_commit": SOURCE_COMMIT,
            "closure": "all transfer entrypoints and their static local import closure",
            "files": records,
            "tree_cid": f"blake3:{digest.hexdigest()}",
        },
        "implementation_cid",
    )


def _safe_path(root: Path, relative: str) -> Path:
    if root.is_symlink():
        raise ValueError("evidence root cannot be a symlink")
    path = root
    for component in Path(relative).parts:
        if component in ("..", "/"):
            raise ValueError("evidence path must remain inside its root")
        path /= component
        if path.is_symlink():
            raise ValueError("evidence input cannot be a symlink")
    return path


def _load_exact_population_narrow(
    source_root: Path,
) -> tuple[exact_data.ZoologyMQARPopulation, dict[str, Any]]:
    paths = (
        position_data.MANIFEST_RELATIVE_PATH,
        position_data.COMMITMENT_RELATIVE_PATH,
        position_data.CONSTRUCTION_MQAR_RELATIVE_PATH,
    )
    for relative in paths:
        _safe_path(source_root, relative)
    manifest, commitment = position_data._validate_public_envelopes(source_root)
    if (
        manifest.get("manifest_cid") != EXPECTED_1043_MANIFEST_CID
        or commitment.get("commitment_cid") != EXPECTED_1043_COMMITMENT_CID
    ):
        raise ValueError("#1043 public envelope identity differs")
    verify_artifact_subset(
        manifest,
        artifact_root=source_root,
        relative_paths=(position_data.CONSTRUCTION_MQAR_RELATIVE_PATH,),
    )
    mqar_record = release._file_record(
        source_root / position_data.CONSTRUCTION_MQAR_RELATIVE_PATH,
        relative_path=position_data.CONSTRUCTION_MQAR_RELATIVE_PATH,
    )
    if mqar_record["file_cid"] != EXPECTED_1043_MQAR_CID:
        raise ValueError("#1043 open MQAR payload identity differs")
    examples = position_data._load_examples_payload(
        source_root / position_data.CONSTRUCTION_MQAR_RELATIVE_PATH,
        population="mqar",
        split="construction",
    )
    split = role_data.split_mqar_construction(examples)
    if split.split_cid != control_development.EXPECTED_1045_SPLIT_CID:
        raise ValueError("#1045 open split identity differs")
    population = exact_data._make_population(
        train=tuple(exact_data._adapt_exact_row(row) for row in split.train),
        development=tuple(
            exact_data._adapt_exact_row(row) for row in split.development
        ),
        name="exact_1045_open_bytes",
        vocab_size=VOCAB_SIZE,
        input_seq_len=INPUT_SEQ_LEN,
        num_kv_pairs=NUM_KV_PAIRS,
        source_split_cid=split.split_cid,
    )
    if population.population_cid != EXPECTED_1045_POPULATION_CID:
        raise ValueError("#1045 exact population identity differs")
    return population, {
        "root": str(source_root),
        "manifest_cid": manifest["manifest_cid"],
        "commitment_cid": commitment["commitment_cid"],
        "mqar": mqar_record,
        "files_read": list(paths),
        "open_rows_validated": role_data.MQAR_TOTAL_ROWS,
        "train_rows": len(split.train),
        "development_rows": len(split.development),
        "unused_open_control_rows": len(split.controls),
        "split_cid": split.split_cid,
        "population_cid": population.population_cid,
        "row_assignment_and_kv_pair_disjointness": "PASS",
        "sealed_payload_access": "FORBIDDEN_NOT_READ",
    }


def _bind_1050(release_root: Path) -> dict[str, Any]:
    path = _safe_path(release_root, release.RESULT_RELATIVE_PATH)
    result = release._read_json(path, cid_field="result_cid")
    decision = result.get("decision", {})
    if (
        result.get("result_cid") != EXPECTED_1050_RESULT_CID
        or result.get("issue") != 1050
        or decision.get("verdict") != "SOURCE_REPRODUCTION_POSITIVE"
        or decision.get("passed") is not True
    ):
        raise ValueError("#1050 positive source result differs")
    return {
        **release._file_record(path, relative_path=release.RESULT_RELATIVE_PATH),
        "result_cid": result["result_cid"],
        "preparation_cid": result["preparation_cid"],
        "verdict": decision["verdict"],
        "positive_learning_rate": result["arms"][0]["learning_rate"],
        "artifact_access": "FORBIDDEN_NOT_READ",
    }


def _population_tensors(
    population: exact_data.ZoologyMQARPopulation,
) -> tuple[dict[str, Tensor], dict[str, Tensor]]:
    primary: dict[str, Tensor] = {}
    for prefix, rows in (("train", population.train), ("test", population.development)):
        batch = exact_data.batch_rows(rows)
        primary.update(
            {
                f"{prefix}_inputs": batch.input_ids,
                f"{prefix}_positions": batch.selected_positions,
                f"{prefix}_targets": batch.targets,
            }
        )
    permuted = exact_data.batch_rows(
        exact_data.permute_exact_bindings(population.development)
    )
    control = {
        "test_inputs": permuted.input_ids,
        "test_positions": permuted.selected_positions,
        "test_targets": permuted.targets,
    }
    return primary, control


def _tensor_record(
    tensors: Mapping[str, Tensor], payload: bytes, path: str
) -> dict[str, Any]:
    return {
        "path": path,
        "bytes": len(payload),
        "file_cid": cid_bytes(payload),
        "tensor_cid": release._tensor_mapping_cid(tensors),
        "shapes": {name: list(value.shape) for name, value in sorted(tensors.items())},
        "dtype": "torch.int64",
    }


def _validate_shapes(tensors: Mapping[str, Tensor], *, control: bool) -> None:
    expected = {}
    for prefix, rows in (
        (("test", TEST_ROWS),)
        if control
        else (("train", TRAIN_ROWS), ("test", TEST_ROWS))
    ):
        for field, width in (
            ("inputs", INPUT_SEQ_LEN),
            ("positions", NUM_KV_PAIRS),
            ("targets", NUM_KV_PAIRS),
        ):
            expected[f"{prefix}_{field}"] = (rows, width)
    if set(tensors) != set(expected):
        raise ValueError(
            "#1053 model ABI must contain only inputs, positions, and targets"
        )
    for name, shape in expected.items():
        value = tensors[name]
        if value.dtype != torch.long or tuple(value.shape) != shape:
            raise ValueError(f"#1053 tensor shape/dtype differs: {name}")
        ceiling = INPUT_SEQ_LEN if name.endswith("_positions") else VOCAB_SIZE
        if bool((value < 0).any()) or bool((value >= ceiling).any()):
            raise ValueError(f"#1053 tensor value outside its domain: {name}")
        if name.endswith("_positions") and not bool(
            (value[:, 1:] > value[:, :-1]).all()
        ):
            raise ValueError("#1053 query positions are not strictly ordered")


def _load_bound_tensors(
    root: Path, record: Mapping[str, Any], *, control: bool
) -> dict[str, Tensor]:
    relative = CONTROL_RELATIVE_PATH if control else DATA_RELATIVE_PATH
    if record.get("path") != relative:
        raise ValueError("#1053 tensor path differs")
    payload = _safe_path(root, relative).read_bytes()
    if len(payload) != record.get("bytes") or cid_bytes(payload) != record.get(
        "file_cid"
    ):
        raise ValueError("#1053 tensor file bytes/CID differ")
    tensors = load_safetensors(payload)
    _validate_shapes(tensors, control=control)
    if _tensor_record(tensors, payload, relative) != dict(record):
        raise ValueError("#1053 tensor identity or metadata differs")
    return tensors


def load_dataset(root: Path, preparation: Mapping[str, Any]) -> dict[str, Tensor]:
    """Read only the primary three-tensor train/test ABI."""

    return _load_bound_tensors(root, preparation["dataset"], control=False)


def load_control(root: Path, preparation: Mapping[str, Any]) -> dict[str, Tensor]:
    """Read the frozen control only after the runner's strict primary pass."""

    return _load_bound_tensors(root, preparation["control"], control=True)


def validate_preparation(root: Path) -> dict[str, Any]:
    """Validate current implementation and primary bytes without opening control."""

    preparation = release._read_json(
        _safe_path(root, PREPARATION_RELATIVE_PATH), cid_field="preparation_cid"
    )
    if (
        preparation.get("schema") != PREPARATION_SCHEMA
        or preparation.get("issue") != ISSUE
        or preparation.get("policy") != POLICY
        or preparation.get("implementation") != implementation_contract()
        or preparation.get("training_contract") != training_contract()
        or preparation.get("source_split_cid")
        != control_development.EXPECTED_1045_SPLIT_CID
        or preparation.get("source_population_cid") != EXPECTED_1045_POPULATION_CID
        or preparation.get("release_1050", {}).get("result_cid")
        != EXPECTED_1050_RESULT_CID
        or preparation.get("predecessor_1045", {}).get("result_cid")
        != control_development.EXPECTED_1045_RESULT_CID
    ):
        raise ValueError("#1053 preparation differs from the current frozen contract")
    load_dataset(root, preparation)
    return preparation


def prepare_transfer(
    root: Path, *, source_root: Path, predecessor_root: Path, release_root: Path
) -> dict[str, Any]:
    """Freeze exact open source bytes plus the one predeclared binding control."""

    path = _safe_path(root, PREPARATION_RELATIVE_PATH)
    if path.exists():
        raise FileExistsError(f"#1053 preparation already exists: {path}")
    for relative in (
        control_development.PREDECESSOR_PREFLIGHT_RELATIVE_PATH,
        control_development.PREDECESSOR_RESULT_RELATIVE_PATH,
    ):
        _safe_path(predecessor_root, relative)
    predecessor = control_development._bind_predecessor(predecessor_root)
    released = _bind_1050(release_root)
    population, source = _load_exact_population_narrow(source_root)
    tensors, control = _population_tensors(population)
    _validate_shapes(tensors, control=False)
    _validate_shapes(control, control=True)
    payload = release._canonical_safetensors(tensors)
    control_payload = release._canonical_safetensors(control)
    body = {
        "schema": PREPARATION_SCHEMA,
        "issue": ISSUE,
        "policy": POLICY,
        "implementation": implementation_contract(),
        "training_contract": training_contract(),
        "source_1043": source,
        "predecessor_1045": predecessor,
        "release_1050": released,
        "source_split_cid": population.source_split_cid,
        "source_population_cid": population.population_cid,
        "dataset": _tensor_record(tensors, payload, DATA_RELATIVE_PATH),
        "control": _tensor_record(control, control_payload, CONTROL_RELATIVE_PATH),
        "control_policy": {
            "intervention": "rotate_physical_values_left_one",
            "positions_and_targets": "UNCHANGED",
            "parameter_updates": 0,
            "evaluation_authority": "ONLY_AFTER_PRIMARY_STRICTLY_GREATER_THAN_0.99",
        },
        "read_ledger": {
            "public_envelopes_read": 5,
            "open_mqar_payloads_read": 1,
            "role_validation_rows": role_data.MQAR_TOTAL_ROWS,
            "role_validation_tokens": role_data.MQAR_TOTAL_ROWS * INPUT_SEQ_LEN,
            "role_validation_count_basis": "unique open serialization rows; adapters revalidate subsets",
            "role_adapter_revalidation_rows": TRAIN_ROWS + TEST_ROWS,
            "control_role_revalidation_rows": TEST_ROWS,
            "role_reads": 0,
            "model_role_reads": 0,
            "future_value_reads": 0,
            "english_payload_reads": 0,
            "natural_payload_reads": 0,
            "tokenizer_payload_reads": 0,
            "sealed_payload_reads": 0,
            "fitted_weight_reads": 0,
            "r4_geometry_reads": 0,
            "uor_byte_reads": 0,
            "teacher_calls": 0,
            "provider_calls": 0,
        },
    }
    preparation = release._with_cid(body, "preparation_cid")
    release._write_exclusive(_safe_path(root, DATA_RELATIVE_PATH), payload)
    release._write_exclusive(_safe_path(root, CONTROL_RELATIVE_PATH), control_payload)
    release._write_exclusive_json(path, preparation)
    return preparation
