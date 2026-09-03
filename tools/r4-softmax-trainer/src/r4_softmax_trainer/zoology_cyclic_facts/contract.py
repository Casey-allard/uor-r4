"""Freeze cyclic fact-order learning and the retained plain readout reference."""

from __future__ import annotations

import copy
from pathlib import Path
from typing import Any

from ..provenance import artifact_records, canonical_json_bytes, cid_bytes, tree_cid
from ..zoology_english_diagnostic.contract import _envelope, _record
from ..zoology_joint_query import contract as previous
from . import data
from .augmentation import AUGMENTATION

ISSUE = 1071
POLICY = "ZoologyCyclicFactOrderAugmentationV1"
MODEL_CONFIG = copy.deepcopy(previous.MODEL_CONFIG)
TRAINING = copy.deepcopy(previous.TRAINING)
EVALUATION = copy.deepcopy(previous.EVALUATION)
BEHAVIOR = {
    "rotations": [0, 1, 2, 3],
    "matched_both_correct_gain_min": 205,
    "both_correct_ceiling": 2048,
    "owner_pairs": 2048,
    "owner_worlds": 1024,
    "object_pairs": 2048,
    "object_worlds": 1024,
    "target_slot_correct_min": 1024,
    "target_slot_rows": 2048,
    "construction_correct_min": 8111,
    "require_overall_reference_preservation": True,
    "require_question_family_reference_preservation": True,
    "require_behavior_for_development": True,
    "gain_rule": "candidate both-correct >= min(same-rotation reference + 205, 2048) for each question type",
    "decision": "all four rotations must pass paired gains, slot floors, family and overall preservation; behavior plus every construction fit gate controls development",
}
SOURCE_CIDS = {
    "preparation": "blake3:e2709d32436f7979aeda795f2ec735d99932cdd1037e7e17321d72a7009ad7e1",
    "fit": "blake3:ff7dd732a914ea7377095d25cfac8a57f039c4c5185550cf4f6bf9c38654825b",
    "result": "blake3:bc1066eb0e9bbf08304ab296ca0c1681b7e8af4b0ea9026945ebef83c7fb9d53",
    "replay": "blake3:6a6ad3ce5ef9e9541fec4994006b39d7a15ce52432a1af3cff351f0b9d96fcf2",
}
SOURCE_FILE_COUNT = 238
INTERVENTION = {
    "reference_issue": 1067,
    "supervised_position_before": 37,
    "supervised_position_after": 37,
    "input_sequence_length": 41,
    "owner_residual": False,
    "initialization": "fresh seed 123; no prior learned state",
    "training_change": "cyclic rotation of complete fact blocks at the frozen traversal offset; same labels, query, model, optimizer, sampler and dropout work",
    "interface": "explicit answer label at query object; literal next input token is question mark",
    "reference_comparison": "retained plain #1067; same supported construction population and each matching rotation; canonical order is rotation zero",
    "dependence": "rotations and question pairs from one world are correlated views; report world-level counts",
    "partial_positive": "retain augmentation as improved construction recipe; no development decisions until every fit gate also passes",
    "partial_negative": "retain measured gains and regressions; keep #1067 reference and revise the binding learning recipe",
    "development_miss": "construction qualifies; fresh-combination transfer unresolved",
    "language_pass": "next separate unchanged-R4 inference preservation; no R4 execution in this fit",
}


def _repo() -> Path:
    return Path(__file__).resolve().parents[5]


def _lineage() -> dict[str, Any]:
    repo = _repo()
    historical = previous._lineage()
    documents, paths = {}, []
    for name, expected in SOURCE_CIDS.items():
        relative = f"docs/r4_zoology_joint_query_1069_{name}.json"
        paths.append(relative)
        documents[name] = _envelope(repo / relative, f"{name}_cid", expected)
    preparation, fitted = documents["preparation"], documents["fit"]
    result, replay = documents["result"], documents["replay"]
    if (
        preparation["training"] != TRAINING
        or preparation["model_config"] != MODEL_CONFIG
        or preparation["evaluation"] != EVALUATION
        or preparation["lineage"]["baseline"] != historical["baseline"]
        or fitted["status"] != "FIT_COMPLETE"
        or fitted["completed_updates"] != TRAINING["total_updates"]
        or fitted["work"] != historical["baseline"]["fit_work"]
        or fitted["preparation_cid"] != preparation["preparation_cid"]
        or result["fit_cid"] != fitted["fit_cid"]
        or result["preparation_cid"] != preparation["preparation_cid"]
        or result["evidence"]["status"] != "JOINT_QUERY_PRESERVATION_MISS"
        or not replay["exact_replay"]
        or not replay["fresh_process"]
        or replay["result_cid"] != result["result_cid"]
        or replay["evidence_cid"] != result["evidence_cid"]
    ):
        raise ValueError(
            "joint-query lineage, plain reference or learning clock differs"
        )
    implementation = preparation["implementation"]
    files = artifact_records(repo, [row["path"] for row in implementation["files"]])
    if (
        len(files) != SOURCE_FILE_COUNT
        or files != implementation["files"]
        or tree_cid(files) != implementation["tree_cid"]
    ):
        raise ValueError("historical source implementation changed")
    return {
        "joint_query_cids": dict(SOURCE_CIDS),
        "readout_cids": historical["readout_cids"],
        "documents": artifact_records(repo, paths),
        "implementation_files": files,
        "construction_files": historical["construction_files"],
        "reference_issue": 1067,
        "baseline": historical["baseline"],
    }


def _bindings(root: Path) -> dict[str, Any]:
    lineage = _lineage()
    repo = _repo()
    paths = {
        row["path"] for row in lineage["implementation_files"] + lineage["documents"]
    }
    paths.update(str(p.relative_to(repo)) for p in Path(__file__).parent.glob("*.py"))
    paths.update(
        str(p.relative_to(repo))
        for p in (repo / "tools/r4-softmax-trainer/tests").glob(
            "test_zoology_cyclic_facts*.py"
        )
    )
    files = artifact_records(repo, sorted(paths))
    dataset = data.validate(root / "data", inspect_development=False)
    if dataset["source"]["files"] != lineage["construction_files"]:
        raise ValueError(
            "copied source data differs from the published English population"
        )
    return {
        "lineage": lineage,
        "implementation": {
            "root": str(repo),
            "files": files,
            "tree_cid": tree_cid(files),
        },
        "dataset": dataset,
    }


def _reference(
    reference_root: Path,
    lineage: dict[str, Any],
    *,
    inspect_model: bool = False,
) -> dict[str, Any]:
    reference_root = reference_root.resolve()
    model = copy.deepcopy(lineage["baseline"]["model"])
    if (
        model.get("path") != "fit/model.safetensors"
        or model.get("config") != MODEL_CONFIG
        or "query_encoding" in model
    ):
        raise ValueError("reference must be the retained plain #1067 model")
    if inspect_model:
        observed = _record(reference_root, model["path"])
        if any(observed[key] != model[key] for key in ("path", "bytes", "cid")):
            raise ValueError(
                "reference model bytes differ from the published #1067 fit"
            )
    return {"root": str(reference_root), "model": model}


def prepare(root: Path, source_root: Path, reference_root: Path) -> dict[str, Any]:
    root, source_root = root.resolve(), source_root.resolve()
    root.mkdir(parents=True, exist_ok=True)
    if (root / "preparation.json").exists():
        raise FileExistsError("cyclic-facts preparation already exists")
    lineage = _lineage()
    reference = _reference(reference_root, lineage, inspect_model=True)
    dataset = data.build(root / "data", source_root / "data")
    # Build audits fresh labels and exclusions. Later validation binds the
    # envelope without opening development or the retained reference weights.
    body = {
        "schema": "uor-r4.zoology-cyclic-facts-preparation/1",
        "issue": ISSUE,
        "policy": POLICY,
        "model_config": MODEL_CONFIG,
        "training": TRAINING,
        "evaluation": EVALUATION,
        "behavior": BEHAVIOR,
        "intervention": INTERVENTION,
        "augmentation": AUGMENTATION,
        "reference": reference,
        **_bindings(root),
    }
    if (
        body["dataset"] != dataset
        or body["lineage"] != lineage
        or reference != _reference(reference_root, lineage, inspect_model=True)
    ):
        raise ValueError("preparation inputs changed during construction")
    body["preparation_cid"] = cid_bytes(canonical_json_bytes(body))
    with (root / "preparation.json").open("xb") as handle:
        handle.write(canonical_json_bytes(body))
    return body


def validate_preparation(root: Path) -> dict[str, Any]:
    root = root.resolve()
    body = _envelope(root / "preparation.json", "preparation_cid")
    for key, expected in (
        ("schema", "uor-r4.zoology-cyclic-facts-preparation/1"),
        ("issue", ISSUE),
        ("policy", POLICY),
        ("model_config", MODEL_CONFIG),
        ("training", TRAINING),
        ("evaluation", EVALUATION),
        ("behavior", BEHAVIOR),
        ("intervention", INTERVENTION),
        ("augmentation", AUGMENTATION),
    ):
        if body.get(key) != expected:
            raise ValueError(f"frozen cyclic-facts {key} differs")
    current = _bindings(root)
    if any(body.get(key) != value for key, value in current.items()):
        raise ValueError("cyclic-facts source, data or implementation changed")
    reference = body.get("reference")
    if not isinstance(reference, dict) or not isinstance(reference.get("root"), str):
        raise TypeError("frozen reference binding is missing")
    if reference != _reference(Path(reference["root"]), current["lineage"]):
        raise ValueError("frozen reference path or model metadata differs")
    return body
