"""Prepared, one-shot C1-SB5 paired-query conditional-binding campaign.

Preparation is the only phase allowed to construct the independently frozen
population or open product records.  The optimizer consumes an explicit
four-file training view and cannot rebuild or read the product envelope.
"""

from __future__ import annotations

import json
import math
import os
from pathlib import Path
from typing import Any, Mapping

import torch
from safetensors.torch import save_file
from tokenizers import Tokenizer

from .constants import FROZEN_MODEL_CONFIG
from .paired_query_binding import (
    FIT_SEED,
    EncodedPairedQueryBindingDataset,
    PairedQueryBindingAdapterConfig,
    PairedQueryBindingFitConfig,
    PairedQueryBindingWallBudgetExceeded,
    REPRESENTATION_UPDATE,
    R4PairedQueryCandidateMatrix,
    evaluate_paired_query_binding,
    fit_paired_query_binding,
)
from .paired_query_binding_data import (
    CENSUS_SCHEMA,
    DATASET_SCHEMA,
    POLICY,
    PREFLIGHT_SCHEMA,
    PRODUCT_SCHEMA,
    TOKENIZER_CENSUS_SCHEMA,
    build_paired_query_binding_population,
)
from .provenance import (
    atomic_write_json,
    canonical_json_bytes,
    cid_bytes,
    cid_file,
    trainer_implementation_contract,
    verify_artifact_subset,
    verify_bound_manifest,
    verify_manifest_envelope,
    write_bound_manifest,
)
from .source_relation_adapter import (
    export_merged_attended_relation_checkpoint,
    validate_tokenizer_contract,
)
from .source_relation_adapter_campaign import (
    EXPECTED_CONFIG_CID,
    EXPECTED_TOKENIZER_CID,
    EXPECTED_WEIGHTS_CID,
    _load_base_model,
    _rust_checkpoint_tree_binding,
    _validate_roots,
    _validated_predecessor,
)
from .train import require_mps


ISSUE = 954
EXPECTED_DATASET_CID = (
    "blake3:b8d6b381eb856396b91ba1cc343b94f6abe99e3de9d54953e5071dfdb1d5bc3f"
)
EXPECTED_PREFLIGHT_CID = (
    "blake3:008a302b1e625a7568d04d26492c058ddcd26a7caaa23b63138e57871c4cf4c1"
)
EXPECTED_CENSUS_CID = (
    "blake3:95533608ebe3dae761d925667572cd6de6d2e1f7b94d3efa29374c0a29fe4307"
)
EXPECTED_TOKENIZER_CENSUS_CID = (
    "blake3:bf4eebd5c35f09ce4452e86fe14823888d75d14dc82a025769c43c4801965271"
)
EXPECTED_PRODUCT_CID = (
    "blake3:5489089f7fc73906d07c27308e6a031a2dd373d016b610f09b26657d4238665e"
)
EXPECTED_SPLIT_POLICY_CID = (
    "blake3:3010b5bc37c1f8b9e5afb9adfab34abf98632df9449ba67f502a57c74555ab0c"
)
EXPECTED_PREDECESSOR_MANIFEST_CID = (
    "blake3:77d5735ccfb4f2ac8a89f2f42a7ad8663b96770ea23a0b4bfae87b3daea7d8f3"
)
EXPECTED_PRODUCT_COMMITMENTS = (
    "blake3:6c0fa9773dcde75b5e07e3427d8554645d1093cde74aefd8b14a497647286630",
    "blake3:c281c06533c3cf418a016e98cb0028e8e833a94f865451ffb1f69cede6112e34",
    "blake3:07ee858c95e9da02b746625ed31e23d066680ebda60c48b081b48b47f1cfcaec",
    "blake3:abce456fad476bed4d30c38f862df1ff432cca797b22bd46ebc231b13746f89a",
)
DATA_MANIFEST_SCHEMA = "uor-r4.paired-query-binding-training-view-manifest/1"
PRODUCT_MANIFEST_SCHEMA = "uor-r4.paired-query-binding-product-manifest/1"
RUN_SCHEMA = "uor-r4.paired-query-binding-run/1"
RESULT_SCHEMA = "uor-r4.paired-query-binding-preflight-result/1"
RESULT_MANIFEST_SCHEMA = "uor-r4.paired-query-binding-preflight-manifest/1"
ARTIFACT_SCHEMA = "uor-r4.paired-query-binding-adapter/1"
HEAD_SCHEMA = "uor-r4.paired-query-binding-head/1"
RESEARCH_ADMISSION = "research_only"

DATASET_FILE = "paired-query-binding-dataset.json"
PREFLIGHT_FILE = "paired-query-binding-preflight.json"
STRUCTURAL_CENSUS_FILE = "paired-query-binding-census.json"
TOKENIZER_CENSUS_FILE = "paired-query-binding-tokenizer-census.json"
PRODUCT_FILE = "paired-query-binding-product-probes.json"
PRODUCT_MANIFEST_FILE = "product-commitments-manifest.json"
TRAINING_MANIFEST_FILE = "training-view-manifest.json"
RUN_CONTRACT_FILE = "run-contract.json"
STARTED_FILE = "preflight-started.json"
RESULT_FILE = "preflight-result.json"
RESULT_MANIFEST_FILE = "preflight-manifest.json"
CHECKPOINT_DIRECTORY = "preflight-checkpoint"
HEAD_DIRECTORY = "preflight-binding-head"
HEAD_TENSORS_FILE = "binding-head.safetensors"
HEAD_MANIFEST_FILE = "head-manifest.json"
ADAPTER_ARTIFACT = "paired-query-binding-adapter.json"

TRAINING_VIEW_ARTIFACTS = {
    DATASET_FILE,
    PREFLIGHT_FILE,
    STRUCTURAL_CENSUS_FILE,
    TOKENIZER_CENSUS_FILE,
}

FIT_PAIRS = 56
SEALED_PAIRS = 28
ROWS_PER_PAIR = 2
WIDTHS = tuple(range(2, 9))
STEPS_PER_EPOCH = 8
OPTIMIZER_STEPS = 120
EPOCHS = 15
PAIRS_PER_STEP = len(WIDTHS)
PAIR_PRESENTATIONS = OPTIMIZER_STEPS * PAIRS_PER_STEP
ROW_PRESENTATIONS = PAIR_PRESENTATIONS * ROWS_PER_PAIR
CELL_PRESENTATIONS = 7_980
ETA_PROBE_STEP = 8
WALL_CEILING_SECONDS = 300.0
EXPECTED_PARTITION_METRICS = {
    FIT_PAIRS: {
        "query_rows": 112,
        "matrix_cells": 532,
        "flip_columns": 98,
        "candidate_copies": 42,
        "duplicate_pairs": 14,
        "outcome": {"answer": 42, "abstain": 42, "conflict": 28},
    },
    SEALED_PAIRS: {
        "query_rows": 56,
        "matrix_cells": 266,
        "flip_columns": 49,
        "candidate_copies": 21,
        "duplicate_pairs": 7,
        "outcome": {"answer": 21, "abstain": 21, "conflict": 14},
    },
}


def _canonical_with_cid(value: dict[str, Any], field: str) -> dict[str, Any]:
    if field in value:
        raise ValueError(f"self-CID field already exists: {field}")
    result = dict(value)
    result[field] = cid_bytes(canonical_json_bytes(value))
    return result


def _verify_self_cid(value: Mapping[str, Any], field: str) -> None:
    unsigned = dict(value)
    observed = unsigned.pop(field, None)
    if not isinstance(observed, str):
        raise ValueError(f"C1-SB5 value has no {field}")
    if observed != cid_bytes(canonical_json_bytes(unsigned)):
        raise ValueError(f"C1-SB5 {field} does not reproduce")


def _write_or_verify(path: Path, value: Mapping[str, Any]) -> None:
    encoded = canonical_json_bytes(value)
    if path.exists():
        if path.read_bytes() != encoded:
            raise ValueError(f"existing C1-SB5 artifact differs: {path}")
        return
    atomic_write_json(path, value)


def _write_exclusive_started_marker(path: Path, value: Mapping[str, Any]) -> None:
    """Claim the sole run atomically; any complete or partial marker is terminal."""
    encoded = canonical_json_bytes(value)
    try:
        with path.open("xb") as target:
            target.write(encoded)
            target.flush()
            os.fsync(target.fileno())
    except FileExistsError as error:
        raise FileExistsError(
            "the sole C1-SB5 preflight was already started"
        ) from error


def _partition_pairs(preflight: Mapping[str, Any], name: str) -> list[dict[str, Any]]:
    records = preflight.get(name)
    if not isinstance(records, list) or not all(isinstance(row, dict) for row in records):
        raise ValueError(f"C1-SB5 {name} partition is not a pair list")
    expected = FIT_PAIRS if name == "fit" else SEALED_PAIRS
    if len(records) != expected:
        raise ValueError(f"C1-SB5 {name} requires exactly {expected} pairs")
    return records


def _validate_population(
    dataset: Mapping[str, Any],
    preflight: Mapping[str, Any],
    products: Mapping[str, Any],
    tokenizer_census: Mapping[str, Any],
) -> None:
    if dataset.get("schema") != DATASET_SCHEMA or dataset.get("policy") != POLICY:
        raise ValueError("C1-SB5 dataset schema or policy drifted")
    if preflight.get("schema") != PREFLIGHT_SCHEMA or preflight.get("policy") != POLICY:
        raise ValueError("C1-SB5 preflight schema or policy drifted")
    if tokenizer_census.get("schema") != TOKENIZER_CENSUS_SCHEMA:
        raise ValueError("C1-SB5 tokenizer-census schema drifted")
    if tokenizer_census.get("policy") != POLICY or not tokenizer_census.get("passed"):
        raise ValueError("C1-SB5 tokenizer census is not positive")
    if products.get("schema") != PRODUCT_SCHEMA or products.get("policy") != POLICY:
        raise ValueError("C1-SB5 product schema or policy drifted")
    census = dataset.get("census")
    if (
        not isinstance(census, Mapping)
        or census.get("schema") != CENSUS_SCHEMA
        or census.get("policy") != POLICY
        or not census.get("passed")
    ):
        raise ValueError("C1-SB5 structural census is not positive")
    fit = _partition_pairs(preflight, "fit")
    sealed = _partition_pairs(preflight, "sealed")
    widths = set(WIDTHS)
    for name, records in (("fit", fit), ("sealed", sealed)):
        observed_widths = {int(record.get("source_width", -1)) for record in records}
        if observed_widths != widths:
            raise ValueError(f"C1-SB5 {name} does not cover widths 2 through 8")
        for record in records:
            queries = record.get("queries")
            matrix = record.get("label_matrix")
            if not isinstance(queries, list) or len(queries) != ROWS_PER_PAIR:
                raise ValueError("C1-SB5 pair must contain exactly two queries")
            if not isinstance(matrix, list) or len(matrix) != ROWS_PER_PAIR:
                raise ValueError("C1-SB5 label matrix must contain exactly two rows")
    commitments = dataset.get("product_probe_commitments")
    if (
        not isinstance(commitments, list)
        or not commitments
        or len(set(commitments)) != len(commitments)
    ):
        raise ValueError("C1-SB5 product commitments are missing or duplicated")
    if products.get("product_probe_commitments") not in (None, commitments):
        raise ValueError("C1-SB5 product envelope commitments disagree")
    for field, value in (
        ("dataset_cid", dataset),
        ("preflight_cid", preflight),
        ("census_cid", census),
        ("tokenizer_census_cid", tokenizer_census),
        ("product_probes_cid", products),
    ):
        _verify_self_cid(value, field)
    frozen = {
        "dataset_cid": dataset["dataset_cid"],
        "preflight_cid": preflight["preflight_cid"],
        "census_cid": census["census_cid"],
        "tokenizer_census_cid": tokenizer_census["tokenizer_census_cid"],
        "product_probes_cid": products["product_probes_cid"],
        "split_policy_cid": dataset.get("split_policy_cid"),
        "product_probe_commitments": tuple(commitments),
    }
    expected = {
        "dataset_cid": EXPECTED_DATASET_CID,
        "preflight_cid": EXPECTED_PREFLIGHT_CID,
        "census_cid": EXPECTED_CENSUS_CID,
        "tokenizer_census_cid": EXPECTED_TOKENIZER_CENSUS_CID,
        "product_probes_cid": EXPECTED_PRODUCT_CID,
        "split_policy_cid": EXPECTED_SPLIT_POLICY_CID,
        "product_probe_commitments": EXPECTED_PRODUCT_COMMITMENTS,
    }
    if frozen != expected:
        raise ValueError("C1-SB5 population identities differ from the freeze")


def prepare_paired_query_binding_data(
    root: Path, *, predecessor: Path
) -> dict[str, Any]:
    """Commit the frozen population and opaque products without optimization."""
    root, predecessor = _validate_roots(root, predecessor)
    predecessor_manifest = _validated_predecessor(predecessor)
    try:
        tokenizer = Tokenizer.from_file(str(predecessor / "tokenizer.json"))
        validate_tokenizer_contract(tokenizer)
        dataset, preflight, products = build_paired_query_binding_population(
            tokenizer, tokenizer_cid=EXPECTED_TOKENIZER_CID
        )
        tokenizer_census = dataset.get("tokenizer_census")
        if not isinstance(tokenizer_census, Mapping):
            raise ValueError("C1-SB5 bound dataset omits its tokenizer census")
        _validate_population(dataset, preflight, products, tokenizer_census)
    except (RuntimeError, ValueError) as error:
        # Population construction is a zero-training admission gate.  Keep the
        # failure typed and in memory: no campaign root or partial training view
        # may be created when the semantic/tokenizer frame is unavailable.
        return {
            "terminal": "UNAVAILABLE_FRAME_OR_POPULATION",
            "issue": ISSUE,
            "policy": POLICY,
            "failure_stage": "POPULATION_BUILD_OR_VALIDATION",
            "failure_type": type(error).__name__,
            "failure": str(error),
            "predecessor_export_manifest_cid": predecessor_manifest["manifest_cid"],
            "population_root": "NOT_CREATED",
            "training_view": "NOT_COMMITTED",
            "artifacts": "NOT_EMITTED",
            "training": "NOT_STARTED",
            "optimizer_steps": 0,
        }

    # Do not expose a partially admitted population if any check above fails.
    root.mkdir(parents=True, exist_ok=True)
    _write_or_verify(root / DATASET_FILE, dataset)
    _write_or_verify(root / PREFLIGHT_FILE, preflight)
    _write_or_verify(root / STRUCTURAL_CENSUS_FILE, dataset["census"])
    _write_or_verify(root / TOKENIZER_CENSUS_FILE, tokenizer_census)
    _write_or_verify(root / PRODUCT_FILE, products)

    product_manifest_path = root / PRODUCT_MANIFEST_FILE
    if product_manifest_path.exists():
        product_manifest = verify_bound_manifest(product_manifest_path, artifact_root=root)
    else:
        product_manifest = write_bound_manifest(
            product_manifest_path,
            {
                "schema": PRODUCT_MANIFEST_SCHEMA,
                "issue": ISSUE,
                "policy": POLICY,
                "product_probes_cid": products["product_probes_cid"],
                "product_probe_commitments": dataset["product_probe_commitments"],
                "access_policy": "opaque to every optimizer and preflight evaluator",
            },
            artifact_root=root,
            relative_paths=[PRODUCT_FILE],
        )

    training_manifest_path = root / TRAINING_MANIFEST_FILE
    if training_manifest_path.exists():
        training_manifest = verify_bound_manifest(training_manifest_path, artifact_root=root)
    else:
        training_manifest = write_bound_manifest(
            training_manifest_path,
            {
                "schema": DATA_MANIFEST_SCHEMA,
                "issue": ISSUE,
                "policy": POLICY,
                "dataset_cid": dataset["dataset_cid"],
                "preflight_cid": preflight["preflight_cid"],
                "split_policy_cid": dataset["split_policy_cid"],
                "census_cid": dataset["census_cid"],
                "tokenizer_census_cid": tokenizer_census["tokenizer_census_cid"],
                "product_probes_cid": products["product_probes_cid"],
                "product_probe_commitments": dataset["product_probe_commitments"],
                "product_probe_count": len(dataset["product_probe_commitments"]),
                "product_manifest_cid": product_manifest["manifest_cid"],
                "predecessor_export_manifest_cid": predecessor_manifest["manifest_cid"],
                "predecessor_weights_cid": EXPECTED_WEIGHTS_CID,
                "predecessor_tokenizer_cid": EXPECTED_TOKENIZER_CID,
                "product_text_access": "DENIED_TO_TRAINING_VIEW",
            },
            artifact_root=root,
            relative_paths=sorted(TRAINING_VIEW_ARTIFACTS),
        )
    artifact_paths = {str(record["path"]) for record in training_manifest["artifacts"]}
    if artifact_paths != TRAINING_VIEW_ARTIFACTS:
        raise ValueError("C1-SB5 training-view artifact whitelist drifted")
    if PRODUCT_FILE in artifact_paths or PRODUCT_MANIFEST_FILE in artifact_paths:
        raise ValueError("C1-SB5 training view contains product material")
    return {
        "terminal": "PAIRED_QUERY_BINDING_DATA_COMMITTED_NO_TRAINING",
        "dataset_cid": dataset["dataset_cid"],
        "preflight_cid": preflight["preflight_cid"],
        "census_cid": dataset["census_cid"],
        "tokenizer_census_cid": tokenizer_census["tokenizer_census_cid"],
        "product_probes_cid": products["product_probes_cid"],
        "product_probe_commitments": dataset["product_probe_commitments"],
        "training_view_manifest_cid": training_manifest["manifest_cid"],
        "product_manifest_cid": product_manifest["manifest_cid"],
        "product_text_status": "COMMITTED_UNOPENED_BY_TRAINER",
    }


def _load_training_view(
    root: Path,
) -> tuple[dict[str, Any], dict[str, Any], dict[str, Any], dict[str, Any]]:
    """Load only the four artifacts admitted to the optimizer view."""
    manifest = verify_manifest_envelope(root / TRAINING_MANIFEST_FILE)
    if manifest.get("schema") != DATA_MANIFEST_SCHEMA or manifest.get("policy") != POLICY:
        raise ValueError("unexpected C1-SB5 training-view schema or policy")
    records = manifest.get("artifacts")
    paths = (
        [record.get("path") for record in records]
        if isinstance(records, list)
        and all(isinstance(record, Mapping) for record in records)
        else []
    )
    if (
        len(paths) != len(TRAINING_VIEW_ARTIFACTS)
        or any(not isinstance(path, str) for path in paths)
        or set(paths) != TRAINING_VIEW_ARTIFACTS
    ):
        raise ValueError("C1-SB5 training-view artifact whitelist drifted")
    verify_artifact_subset(
        manifest,
        artifact_root=root,
        relative_paths=TRAINING_VIEW_ARTIFACTS,
    )
    dataset = json.loads((root / DATASET_FILE).read_text(encoding="utf-8"))
    preflight = json.loads((root / PREFLIGHT_FILE).read_text(encoding="utf-8"))
    structural = json.loads((root / STRUCTURAL_CENSUS_FILE).read_text(encoding="utf-8"))
    tokenizer_census = json.loads((root / TOKENIZER_CENSUS_FILE).read_text(encoding="utf-8"))
    _verify_self_cid(dataset, "dataset_cid")
    _verify_self_cid(preflight, "preflight_cid")
    _verify_self_cid(structural, "census_cid")
    _verify_self_cid(tokenizer_census, "tokenizer_census_cid")
    if dataset.get("census") != structural or not structural.get("passed"):
        raise ValueError("C1-SB5 structural census differs or is not positive")
    expected = {
        "dataset_cid": dataset["dataset_cid"],
        "preflight_cid": preflight["preflight_cid"],
        "split_policy_cid": dataset["split_policy_cid"],
        "census_cid": structural["census_cid"],
        "tokenizer_census_cid": tokenizer_census["tokenizer_census_cid"],
        "product_probes_cid": dataset["product_probes_cid"],
        "product_probe_commitments": dataset["product_probe_commitments"],
        "product_probe_count": len(dataset["product_probe_commitments"]),
        "predecessor_weights_cid": EXPECTED_WEIGHTS_CID,
        "predecessor_tokenizer_cid": EXPECTED_TOKENIZER_CID,
        "predecessor_export_manifest_cid": EXPECTED_PREDECESSOR_MANIFEST_CID,
        "product_text_access": "DENIED_TO_TRAINING_VIEW",
    }
    if any(manifest.get(key) != value for key, value in expected.items()):
        raise ValueError("C1-SB5 training view differs from its commitments")
    frozen = {
        "dataset_cid": EXPECTED_DATASET_CID,
        "preflight_cid": EXPECTED_PREFLIGHT_CID,
        "split_policy_cid": EXPECTED_SPLIT_POLICY_CID,
        "census_cid": EXPECTED_CENSUS_CID,
        "tokenizer_census_cid": EXPECTED_TOKENIZER_CENSUS_CID,
        "product_probes_cid": EXPECTED_PRODUCT_CID,
        "product_probe_commitments": list(EXPECTED_PRODUCT_COMMITMENTS),
    }
    if any(manifest.get(key) != value for key, value in frozen.items()):
        raise ValueError("C1-SB5 training view does not bind the frozen population")
    if (
        dataset.get("schema") != DATASET_SCHEMA
        or dataset.get("policy") != POLICY
        or preflight.get("schema") != PREFLIGHT_SCHEMA
        or preflight.get("policy") != POLICY
        or tokenizer_census.get("schema") != TOKENIZER_CENSUS_SCHEMA
        or tokenizer_census.get("policy") != POLICY
        or not tokenizer_census.get("passed")
    ):
        raise ValueError("C1-SB5 loaded training identities or gates drifted")
    _partition_pairs(preflight, "fit")
    _partition_pairs(preflight, "sealed")
    return dataset, preflight, tokenizer_census, manifest


def _run_contract(
    *,
    predecessor_manifest: Mapping[str, Any],
    dataset: Mapping[str, Any],
    preflight: Mapping[str, Any],
    tokenizer_census: Mapping[str, Any],
    training_manifest: Mapping[str, Any],
) -> dict[str, Any]:
    fit_config = PairedQueryBindingFitConfig()
    fit_contract = fit_config.as_contract()
    frozen_schedule = {
        "seed": FIT_SEED,
        "widths": list(WIDTHS),
        "pairs_per_step": PAIRS_PER_STEP,
        "rows_per_pair": ROWS_PER_PAIR,
        "steps_per_epoch": STEPS_PER_EPOCH,
        "optimizer_steps": OPTIMIZER_STEPS,
        "epochs": EPOCHS,
        "pair_presentations": PAIR_PRESENTATIONS,
        "row_presentations": ROW_PRESENTATIONS,
        "cell_presentations": CELL_PRESENTATIONS,
        "eta_probe_step": ETA_PROBE_STEP,
        "wall_ceiling_seconds": WALL_CEILING_SECONDS,
    }
    if any(
        fit_contract.get(key) != value
        for key, value in frozen_schedule.items()
        if key in fit_contract
    ):
        raise ValueError("C1-SB5 fit configuration differs from the frozen schedule")
    return _canonical_with_cid(
        {
            "schema": RUN_SCHEMA,
            "issue": ISSUE,
            "policy": POLICY,
            "predecessor_export_manifest_cid": predecessor_manifest["manifest_cid"],
            "predecessor_weights_cid": EXPECTED_WEIGHTS_CID,
            "predecessor_tokenizer_cid": EXPECTED_TOKENIZER_CID,
            "model_contract": FROZEN_MODEL_CONFIG.as_contract(),
            "adapter_contract": PairedQueryBindingAdapterConfig().as_contract(),
            "optimizer_contract": fit_contract,
            "frozen_schedule": frozen_schedule,
            "dataset_cid": dataset["dataset_cid"],
            "preflight_cid": preflight["preflight_cid"],
            "split_policy_cid": dataset["split_policy_cid"],
            "census_cid": dataset["census_cid"],
            "tokenizer_census_cid": tokenizer_census["tokenizer_census_cid"],
            "training_view_manifest_cid": training_manifest["manifest_cid"],
            "product_probes_cid": dataset["product_probes_cid"],
            "product_probe_commitments": dataset["product_probe_commitments"],
            "product_text_access": "DENIED_TO_PYTHON_TRAINER_AND_EVALUATOR",
            "mechanism": (
                "two query rows share one exact source/candidate encoding; an asymmetric "
                "subject-binding head must make a candidate change sign with the query"
            ),
            "objective": (
                "paired row/cell classification plus a columnwise counterfactual flip "
                "margin over the complete candidate matrix"
            ),
            "execution_backend": (
                "Apple MPS required; CPU fallback refused; no CUDA path"
            ),
            "reachability_ceiling": "84/84 fit+sealed pairs = 100%",
            "preflight_gate": {
                "fit_pairs": FIT_PAIRS,
                "sealed_pairs": SEALED_PAIRS,
                "main_semantics": (
                    "all pairs, rows, cells, flips, copies, duplicates, and "
                    "outcomes exact"
                ),
                "row_swap": (
                    "28/28 sealed pairs exact and identity-aligned main/swapped "
                    "traces bit-exact"
                ),
                "mean_query_ablation": "identical paired rows and 0/28 exact with higher loss",
                "attention_off": "0/28 exact with higher loss",
                "delta": (
                    "24 LoRA plus 3 head tensors changed and finite; no base "
                    "nontarget changed"
                ),
                "deterministic_replay": "exact",
                "wall_ceiling_seconds": WALL_CEILING_SECONDS,
            },
            "if_positive": "emit one research-only merged checkpoint and separate binding head",
            "if_negative": "emit no checkpoint or head and retire C1-SB5 without retry",
            "implementation": trainer_implementation_contract(),
        },
        "run_contract_cid",
    )


def _counter(metrics: Mapping[str, Any], key: str) -> tuple[int, int]:
    value = metrics.get(key)
    if not isinstance(value, Mapping):
        raise ValueError(f"C1-SB5 metrics have no {key} counter")
    correct = value.get("correct")
    total = value.get("total")
    if not isinstance(correct, int) or not isinstance(total, int) or not 0 <= correct <= total:
        raise ValueError(f"C1-SB5 {key} counter is invalid")
    return correct, total


def _main_metrics_exact(metrics: Mapping[str, Any], *, expected_pairs: int) -> bool:
    if metrics.get("pairs") != expected_pairs:
        raise ValueError("C1-SB5 evaluation pair count drifted")
    expected = EXPECTED_PARTITION_METRICS.get(expected_pairs)
    if expected is None:
        raise ValueError("C1-SB5 metrics requested an unknown partition")
    if (
        metrics.get("query_rows") != expected["query_rows"]
        or metrics.get("matrix_cells") != expected["matrix_cells"]
        or metrics.get("flip_columns") != expected["flip_columns"]
    ):
        raise ValueError("C1-SB5 row, cell, or flip denominator drifted")
    required = {
        "pair_exact": expected_pairs,
        "row_exact": expected_pairs * ROWS_PER_PAIR,
        "candidate_state_bit_identity": expected_pairs,
    }
    exact = True
    for key, expected_total in required.items():
        correct, total = _counter(metrics, key)
        if total != expected_total:
            raise ValueError(f"C1-SB5 {key} total drifted")
        exact = exact and correct == total
    exact_totals = {
        "cell_exact": expected["matrix_cells"],
        "flip_exact": expected["flip_columns"],
        "candidate_copy_exact": expected["candidate_copies"],
        "duplicate_pair_exact": expected["duplicate_pairs"],
    }
    for key, expected_total in exact_totals.items():
        correct, total = _counter(metrics, key)
        if total != expected_total:
            raise ValueError(f"C1-SB5 {key} denominator drifted")
        exact = exact and correct == total
    outcomes = metrics.get("outcome")
    if not isinstance(outcomes, Mapping) or set(outcomes) != {"answer", "abstain", "conflict"}:
        raise ValueError("C1-SB5 outcome counters drifted")
    for name, expected_total in expected["outcome"].items():
        correct, total = _counter(outcomes, name)
        if total != expected_total:
            raise ValueError(f"C1-SB5 {name} outcome denominator drifted")
        exact = exact and correct == total
    return exact


def _finite_loss(metrics: Mapping[str, Any]) -> float:
    value = metrics.get("mean_loss")
    if not isinstance(value, (int, float)) or not math.isfinite(float(value)):
        raise ValueError("C1-SB5 mean loss is missing or nonfinite")
    return float(value)


def _row_swap_equivariance(
    sealed: Mapping[str, Any], row_swap: Mapping[str, Any]
) -> dict[str, Any]:
    """Compare sealed and row-swapped traces after restoring query identity."""
    if sealed.get("row_swap") not in (None, False) or row_swap.get("row_swap") is not True:
        raise ValueError("C1-SB5 row-swap trace flags are missing or inconsistent")
    if any(
        value.get(flag) not in (None, False)
        for value in (sealed, row_swap)
        for flag in ("attention_off", "mean_query_ablation")
    ):
        raise ValueError("C1-SB5 row-swap trace contains another active control")
    sealed_evaluations = sealed.get("pair_evaluations")
    swapped_evaluations = row_swap.get("pair_evaluations")
    if not isinstance(sealed_evaluations, list) or not isinstance(
        swapped_evaluations, list
    ):
        raise ValueError("C1-SB5 row-swap trace populations are missing")
    if len(sealed_evaluations) != SEALED_PAIRS or len(swapped_evaluations) != SEALED_PAIRS:
        raise ValueError("C1-SB5 row-swap trace cardinality drifted")

    if not all(
        isinstance(row, Mapping)
        for row in (*sealed_evaluations, *swapped_evaluations)
    ):
        raise ValueError("C1-SB5 row-swap pair trace is not an object")
    sealed_ids = [row.get("record_id") for row in sealed_evaluations]
    swapped_ids = [row.get("record_id") for row in swapped_evaluations]
    if not all(isinstance(record_id, str) for record_id in sealed_ids + swapped_ids):
        raise ValueError("C1-SB5 row-swap traces contain an invalid record identity")
    record_order_exact = sealed_ids == swapped_ids and len(set(sealed_ids)) == SEALED_PAIRS

    sealed_pairs: list[dict[str, Any]] = []
    aligned_swapped_pairs: list[dict[str, Any]] = []
    for sealed_pair, swapped_pair in zip(sealed_evaluations, swapped_evaluations):
        sealed_rows = sealed_pair.get("query_rows")
        swapped_rows = swapped_pair.get("query_rows")
        if (
            not isinstance(sealed_rows, list)
            or len(sealed_rows) != ROWS_PER_PAIR
            or not isinstance(swapped_rows, list)
            or len(swapped_rows) != ROWS_PER_PAIR
        ):
            raise ValueError("C1-SB5 row-swap pair does not contain two query rows")
        sealed_value = dict(sealed_pair)
        swapped_value = dict(swapped_pair)
        swapped_value["query_rows"] = list(reversed(swapped_rows))
        sealed_pairs.append(sealed_value)
        aligned_swapped_pairs.append(swapped_value)

    aggregate_keys = (
        "pairs",
        "query_rows",
        "matrix_cells",
        "flip_columns",
        "mean_row_margin",
        "mean_flip_margin",
        "mean_total_loss",
        "mean_loss",
        "pair_exact",
        "row_exact",
        "cell_exact",
        "flip_exact",
        "candidate_copy_exact",
        "duplicate_pair_exact",
        "outcome",
        "candidate_state_bit_identity",
        "paired_rows_identical",
        "candidate_state_identity_exact",
    )
    sealed_aggregate = {key: sealed.get(key) for key in aggregate_keys}
    swapped_aggregate = {key: row_swap.get(key) for key in aggregate_keys}
    pair_trace_exact = canonical_json_bytes(sealed_pairs) == canonical_json_bytes(
        aligned_swapped_pairs
    )
    aggregate_exact = canonical_json_bytes(sealed_aggregate) == canonical_json_bytes(
        swapped_aggregate
    )
    sealed_trace_cid = cid_bytes(canonical_json_bytes(sealed_pairs))
    swapped_trace_cid = cid_bytes(canonical_json_bytes(aligned_swapped_pairs))
    return {
        "record_order_exact": record_order_exact,
        "pair_trace_bit_exact": pair_trace_exact,
        "aggregate_bit_exact": aggregate_exact,
        "sealed_identity_aligned_trace_cid": sealed_trace_cid,
        "swapped_identity_aligned_trace_cid": swapped_trace_cid,
        "passed": record_order_exact
        and pair_trace_exact
        and aggregate_exact
        and sealed_trace_cid == swapped_trace_cid,
    }


def _control_gate(
    sealed: Mapping[str, Any],
    row_swap: Mapping[str, Any],
    mean_query: Mapping[str, Any],
    attention_off: Mapping[str, Any],
) -> dict[str, Any]:
    sealed_loss = _finite_loss(sealed)
    row_swap_semantic_exact = _main_metrics_exact(
        row_swap, expected_pairs=SEALED_PAIRS
    )
    row_swap_equivariance = _row_swap_equivariance(sealed, row_swap)
    row_swap_exact = row_swap_semantic_exact and row_swap_equivariance["passed"]
    mean_correct, mean_total = _counter(mean_query, "paired_rows_identical")
    mean_pair_correct, mean_pair_total = _counter(mean_query, "pair_exact")
    off_pair_correct, off_pair_total = _counter(attention_off, "pair_exact")
    mean_passed = (
        mean_correct == mean_total == SEALED_PAIRS
        and mean_pair_correct == 0
        and mean_pair_total == SEALED_PAIRS
        and _finite_loss(mean_query) > sealed_loss
    )
    attention_passed = (
        off_pair_correct == 0
        and off_pair_total == SEALED_PAIRS
        and _finite_loss(attention_off) > sealed_loss
    )
    return {
        "row_swap_exact": row_swap_exact,
        "row_swap_equivariance": row_swap_equivariance,
        "mean_query_ablation": {
            "paired_rows_identical": {"correct": mean_correct, "total": mean_total},
            "pair_exact": {"correct": mean_pair_correct, "total": mean_pair_total},
            "loss_higher_than_main": _finite_loss(mean_query) > sealed_loss,
            "passed": mean_passed,
        },
        "attention_off": {
            "pair_exact": {"correct": off_pair_correct, "total": off_pair_total},
            "loss_higher_than_main": _finite_loss(attention_off) > sealed_loss,
            "passed": attention_passed,
        },
        "passed": row_swap_exact and mean_passed and attention_passed,
    }


def _delta_gate(delta: Mapping[str, Any]) -> bool:
    head = delta.get("binding_head")
    return bool(
        delta.get("target_tensor_count") == 24
        and delta.get("changed_target_tensor_count") == 24
        and delta.get("all_target_tensors_finite") is True
        and delta.get("changed_nontarget_tensors") == []
        and isinstance(head, Mapping)
        and head.get("tensor_count") == 3
        and head.get("changed_tensor_count") == 3
        and head.get("all_finite") is True
        and delta.get("passed") is True
    )


def _sanitized_optimization(value: Mapping[str, Any]) -> dict[str, Any]:
    result = {
        key: item
        for key, item in value.items()
        if key not in {"elapsed_seconds", "projected_seconds_at_eta_probe"}
    }
    result["operational_budget_observation"] = {
        "eta_probe_step": ETA_PROBE_STEP,
        "wall_ceiling_seconds": WALL_CEILING_SECONDS,
        "eta_probe_passed": True,
        "completed_within_wall_ceiling": True,
        "timing_values": "OMITTED_FROM_CONTENT_ADDRESS_FOR_DETERMINISM",
    }
    return result


def _sanitized_budget_failure(error: PairedQueryBindingWallBudgetExceeded) -> dict[str, Any]:
    value = error.as_result()
    for key in ("stopped_after_step", "elapsed_seconds", "projected_seconds_at_eta_probe"):
        value.pop(key, None)
    value.update(
        {
            "budget_gate_passed": False,
            "timing_values": "OMITTED_FROM_CONTENT_ADDRESS_FOR_DETERMINISM",
        }
    )
    return value


def _schedule_observation(
    value: Mapping[str, Any], dataset: EncodedPairedQueryBindingDataset
) -> dict[str, int]:
    dataset.validate_fit_schedule()
    cell_presentations = 0
    for step in range(1, OPTIMIZER_STEPS + 1):
        for index in dataset.record_indices_for_step(step):
            cell_presentations += ROWS_PER_PAIR * len(
                dataset.records[index].candidate_groups
            )
    observation = {
        "optimizer_steps": int(value.get("optimizer_steps", -1)),
        "steps_per_epoch": STEPS_PER_EPOCH,
        "epochs": EPOCHS,
        "pairs_per_step": int(value.get("paired_records_per_step", -1)),
        "pair_presentations": PAIR_PRESENTATIONS,
        "row_presentations": ROW_PRESENTATIONS,
        "cell_presentations": cell_presentations,
    }
    expected = {
        "optimizer_steps": OPTIMIZER_STEPS,
        "steps_per_epoch": STEPS_PER_EPOCH,
        "epochs": EPOCHS,
        "pairs_per_step": PAIRS_PER_STEP,
        "pair_presentations": PAIR_PRESENTATIONS,
        "row_presentations": ROW_PRESENTATIONS,
        "cell_presentations": CELL_PRESENTATIONS,
    }
    if observation != expected:
        raise ValueError("C1-SB5 optimizer did not execute the frozen schedule")
    return observation


def _write_positive_delivery(
    root: Path,
    *,
    predecessor: Path,
    adapter: R4PairedQueryCandidateMatrix,
    result: Mapping[str, Any],
    dataset: Mapping[str, Any],
    training_manifest: Mapping[str, Any],
    run_contract: Mapping[str, Any],
) -> dict[str, Any]:
    checkpoint = root / CHECKPOINT_DIRECTORY
    head_root = root / HEAD_DIRECTORY
    if checkpoint.exists() or head_root.exists():
        raise FileExistsError("C1-SB5 positive artifacts already exist")
    export_merged_attended_relation_checkpoint(
        adapter,
        output_dir=checkpoint,
        tokenizer_path=predecessor / "tokenizer.json",
        training_result=dict(result),
        dataset_manifest_cid=str(dataset["dataset_cid"]),
        training_view_manifest_cid=str(training_manifest["manifest_cid"]),
        split_policy_cid=str(dataset["split_policy_cid"]),
        run_contract_cid=str(run_contract["run_contract_cid"]),
        selected_checkpoint_cid=None,
        selected_checkpoint_identity="C1-SB5 merged LoRA training output",
    )
    checkpoint_manifest = verify_bound_manifest(
        checkpoint / "export-manifest.json", artifact_root=checkpoint
    )
    if (
        checkpoint_manifest.get("weights_cid") == EXPECTED_WEIGHTS_CID
        or checkpoint_manifest.get("config_cid") != EXPECTED_CONFIG_CID
        or checkpoint_manifest.get("tokenizer_cid") != EXPECTED_TOKENIZER_CID
        or checkpoint_manifest.get("training_result_cid") != result["result_cid"]
    ):
        raise ValueError("C1-SB5 positive checkpoint identity drifted")

    head_state = adapter.binding_head_state_dict()
    if len(head_state) != 3 or any(
        not torch.isfinite(tensor).all() for tensor in head_state.values()
    ):
        raise ValueError("C1-SB5 binding head is not exactly three finite tensors")
    head_root.mkdir(parents=True)
    head_path = head_root / HEAD_TENSORS_FILE
    temporary = head_root / f".{HEAD_TENSORS_FILE}.part"
    save_file(
        {
            name: tensor.detach().to(device="cpu").contiguous()
            for name, tensor in sorted(head_state.items())
        },
        str(temporary),
        metadata={"format": "pt"},
    )
    temporary.replace(head_path)
    head_manifest = write_bound_manifest(
        head_root / HEAD_MANIFEST_FILE,
        {
            "schema": HEAD_SCHEMA,
            "issue": ISSUE,
            "policy": POLICY,
            "tensor_names": sorted(head_state),
            "tensor_count": 3,
            "head_weights_cid": cid_file(head_path),
            "training_result_cid": result["result_cid"],
            "run_contract_cid": run_contract["run_contract_cid"],
        },
        artifact_root=head_root,
        relative_paths=[HEAD_TENSORS_FILE],
    )
    checkpoint_tree = _rust_checkpoint_tree_binding(checkpoint)
    artifact = _canonical_with_cid(
        {
            "schema": ARTIFACT_SCHEMA,
            "issue": ISSUE,
            "policy": POLICY,
            "admission": RESEARCH_ADMISSION,
            "representation_update": REPRESENTATION_UPDATE,
            "predecessor_model_weights_cid": EXPECTED_WEIGHTS_CID,
            "model_weights_cid": checkpoint_manifest["weights_cid"],
            "checkpoint_tree_cid": checkpoint_tree["checkpoint_tree_cid"],
            "binding_head_manifest_cid": head_manifest["manifest_cid"],
            "binding_head_weights_cid": head_manifest["head_weights_cid"],
            "config_cid": checkpoint_manifest["config_cid"],
            "tokenizer_cid": checkpoint_manifest["tokenizer_cid"],
            "dataset_cid": dataset["dataset_cid"],
            "split_policy_cid": dataset["split_policy_cid"],
            "run_contract_cid": run_contract["run_contract_cid"],
            "training_result_cid": result["result_cid"],
            "product_probe_commitments": dataset["product_probe_commitments"],
        },
        "artifact_cid",
    )
    _write_or_verify(root / ADAPTER_ARTIFACT, artifact)
    relative_paths = [
        *(
            f"{CHECKPOINT_DIRECTORY}/{record['path']}"
            for record in checkpoint_manifest["artifacts"]
        ),
        f"{CHECKPOINT_DIRECTORY}/export-manifest.json",
        f"{HEAD_DIRECTORY}/{HEAD_TENSORS_FILE}",
        f"{HEAD_DIRECTORY}/{HEAD_MANIFEST_FILE}",
        ADAPTER_ARTIFACT,
    ]
    return {
        "artifact": artifact,
        "checkpoint_manifest": checkpoint_manifest,
        "checkpoint_tree": checkpoint_tree,
        "head_manifest": head_manifest,
        "relative_paths": relative_paths,
    }


def _finalize_result(
    root: Path,
    *,
    predecessor: Path,
    adapter: R4PairedQueryCandidateMatrix | None,
    result: dict[str, Any],
    dataset: Mapping[str, Any],
    preflight: Mapping[str, Any],
    tokenizer_census: Mapping[str, Any],
    training_manifest: Mapping[str, Any],
    run_contract: Mapping[str, Any],
) -> tuple[dict[str, Any], dict[str, Any] | None]:
    delivery = None
    if str(result["terminal"]).startswith("PASS_"):
        if adapter is None:
            raise ValueError("C1-SB5 passing result has no adapter")
        delivery = _write_positive_delivery(
            root,
            predecessor=predecessor,
            adapter=adapter,
            result=result,
            dataset=dataset,
            training_manifest=training_manifest,
            run_contract=run_contract,
        )
    atomic_write_json(root / RESULT_FILE, result)
    relative_paths = [
        DATASET_FILE,
        PREFLIGHT_FILE,
        STRUCTURAL_CENSUS_FILE,
        TOKENIZER_CENSUS_FILE,
        TRAINING_MANIFEST_FILE,
        RUN_CONTRACT_FILE,
        STARTED_FILE,
        RESULT_FILE,
    ]
    payload: dict[str, Any] = {
        "schema": RESULT_MANIFEST_SCHEMA,
        "issue": ISSUE,
        "policy": POLICY,
        "terminal": result["terminal"],
        "representation_update": REPRESENTATION_UPDATE,
        "dataset_cid": dataset["dataset_cid"],
        "preflight_cid": preflight["preflight_cid"],
        "tokenizer_census_cid": tokenizer_census["tokenizer_census_cid"],
        "run_contract_cid": run_contract["run_contract_cid"],
        "result_cid": result["result_cid"],
        "product_status": "UNOPENED_NOT_RUN",
    }
    if delivery is not None:
        payload.update(
            {
                "adapter_artifact_cid": delivery["artifact"]["artifact_cid"],
                "checkpoint_tree_cid": delivery["checkpoint_tree"]["checkpoint_tree_cid"],
                "binding_head_manifest_cid": delivery["head_manifest"]["manifest_cid"],
            }
        )
        relative_paths.extend(delivery["relative_paths"])
    manifest = write_bound_manifest(
        root / RESULT_MANIFEST_FILE,
        payload,
        artifact_root=root,
        relative_paths=relative_paths,
    )
    return manifest, delivery


def run_paired_query_binding_preflight(
    root: Path, *, predecessor: Path
) -> dict[str, Any]:
    """Run exactly one frozen C1-SB5 preflight without product access."""
    root, predecessor = _validate_roots(root, predecessor)
    predecessor_manifest = _validated_predecessor(predecessor)
    dataset, preflight, tokenizer_census, training_manifest = _load_training_view(root)
    if (root / STARTED_FILE).exists() or (root / RESULT_FILE).exists():
        raise FileExistsError("the sole C1-SB5 preflight was already started")
    run_contract = _run_contract(
        predecessor_manifest=predecessor_manifest,
        dataset=dataset,
        preflight=preflight,
        tokenizer_census=tokenizer_census,
        training_manifest=training_manifest,
    )
    _write_or_verify(root / RUN_CONTRACT_FILE, run_contract)
    _write_exclusive_started_marker(
        root / STARTED_FILE,
        {
            "schema": RUN_SCHEMA,
            "phase": "SOLE_C1_SB5_PREFLIGHT_STARTED",
            "run_contract_cid": run_contract["run_contract_cid"],
        },
    )

    device = require_mps(FIT_SEED)
    tokenizer = Tokenizer.from_file(str(predecessor / "tokenizer.json"))
    validate_tokenizer_contract(tokenizer)
    fit_records = _partition_pairs(preflight, "fit")
    sealed_records = _partition_pairs(preflight, "sealed")
    fit_dataset = EncodedPairedQueryBindingDataset(fit_records)
    sealed_dataset = EncodedPairedQueryBindingDataset(sealed_records)
    model, base_state = _load_base_model(predecessor)
    adapter = R4PairedQueryCandidateMatrix(model).to(device)

    try:
        optimization = fit_paired_query_binding(adapter, fit_dataset)
    except PairedQueryBindingWallBudgetExceeded as error:
        result = _canonical_with_cid(
            {
                "schema": RESULT_SCHEMA,
                "issue": ISSUE,
                "policy": POLICY,
                "terminal": "UNAVAILABLE_PAIRED_QUERY_BINDING_BUDGET",
                "representation_update": REPRESENTATION_UPDATE,
                "run_contract_cid": run_contract["run_contract_cid"],
                "dataset_cid": dataset["dataset_cid"],
                "preflight_cid": preflight["preflight_cid"],
                "operational_budget_observation": _sanitized_budget_failure(error),
                "fit_metrics": "NOT_RUN",
                "sealed_metrics": "NOT_RUN",
                "controls": "NOT_RUN",
                "delta_audit": adapter.representation_audit(base_state),
                "deterministic_replay": "NOT_RUN",
                "access_audit": {
                    "training_view_artifacts": sorted(TRAINING_VIEW_ARTIFACTS),
                    "product_or_forbidden_reads": 0,
                    "product_status": "COMMITTED_UNOPENED",
                },
                "product": "UNOPENED_NOT_RUN",
                "rust_parity": "NOT_RUN",
                "development": "NOT_RUN",
                "retry": "FORBIDDEN",
                "rung_disposition": "RETIRED_UNAVAILABLE_NO_RETRY",
            },
            "result_cid",
        )
        manifest, _ = _finalize_result(
            root,
            predecessor=predecessor,
            adapter=None,
            result=result,
            dataset=dataset,
            preflight=preflight,
            tokenizer_census=tokenizer_census,
            training_manifest=training_manifest,
            run_contract=run_contract,
        )
        return {**result, "preflight_manifest_cid": manifest["manifest_cid"]}

    schedule_observation = _schedule_observation(optimization, fit_dataset)
    fit_metrics = evaluate_paired_query_binding(adapter, fit_dataset, device=device)
    sealed_metrics = evaluate_paired_query_binding(adapter, sealed_dataset, device=device)
    replay_metrics = evaluate_paired_query_binding(adapter, sealed_dataset, device=device)
    row_swap_metrics = evaluate_paired_query_binding(
        adapter, sealed_dataset, device=device, row_swap=True
    )
    mean_query_metrics = evaluate_paired_query_binding(
        adapter, sealed_dataset, device=device, mean_query_ablation=True
    )
    attention_off_metrics = evaluate_paired_query_binding(
        adapter, sealed_dataset, device=device, attention_off=True
    )
    replay_exact = canonical_json_bytes(sealed_metrics) == canonical_json_bytes(replay_metrics)
    controls = _control_gate(
        sealed_metrics, row_swap_metrics, mean_query_metrics, attention_off_metrics
    )
    delta = adapter.representation_audit(base_state)
    semantic_passed = _main_metrics_exact(
        fit_metrics, expected_pairs=FIT_PAIRS
    ) and _main_metrics_exact(sealed_metrics, expected_pairs=SEALED_PAIRS)
    passed = semantic_passed and controls["passed"] and _delta_gate(delta) and replay_exact
    terminal = (
        "PASS_PAIRED_QUERY_BINDING_PREFLIGHT_RESEARCH_ONLY"
        if passed
        else "FAIL_PAIRED_QUERY_BINDING_PREFLIGHT"
    )
    result = _canonical_with_cid(
        {
            "schema": RESULT_SCHEMA,
            "issue": ISSUE,
            "policy": POLICY,
            "terminal": terminal,
            "representation_update": REPRESENTATION_UPDATE,
            "run_contract_cid": run_contract["run_contract_cid"],
            "dataset_cid": dataset["dataset_cid"],
            "preflight_cid": preflight["preflight_cid"],
            "tokenizer_census_cid": tokenizer_census["tokenizer_census_cid"],
            "training_view_manifest_cid": training_manifest["manifest_cid"],
            "optimization": {
                **_sanitized_optimization(optimization),
                "frozen_schedule_observation": schedule_observation,
            },
            "fit_metrics": fit_metrics,
            "sealed_metrics": sealed_metrics,
            "controls": controls,
            "delta_audit": delta,
            "deterministic_replay": {
                "exact": replay_exact,
                "timing_excluded": True,
            },
            "access_audit": {
                "training_view_artifacts": sorted(TRAINING_VIEW_ARTIFACTS),
                "product_or_forbidden_reads": 0,
                "product_status": "COMMITTED_UNOPENED",
            },
            "product": "UNOPENED_NOT_RUN",
            "rust_parity": "NOT_RUN",
            "development": "NOT_RUN",
            "retry": "FORBIDDEN",
            "rung_disposition": (
                "RESEARCH_CHECKPOINT_ONLY_AWAITING_SEPARATE_NEXT_DECISION"
                if passed
                else "RETIRED_NO_RETRY"
            ),
        },
        "result_cid",
    )
    manifest, delivery = _finalize_result(
        root,
        predecessor=predecessor,
        adapter=adapter if passed else None,
        result=result,
        dataset=dataset,
        preflight=preflight,
        tokenizer_census=tokenizer_census,
        training_manifest=training_manifest,
        run_contract=run_contract,
    )
    return {
        **result,
        "preflight_manifest_cid": manifest["manifest_cid"],
        "research_checkpoint": (
            delivery["checkpoint_manifest"]["weights_cid"] if delivery else "NOT_EMITTED"
        ),
        "binding_head": (
            delivery["head_manifest"]["head_weights_cid"] if delivery else "NOT_EMITTED"
        ),
    }
