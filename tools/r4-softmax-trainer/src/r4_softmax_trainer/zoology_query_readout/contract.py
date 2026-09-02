"""Freeze one changed readout position, matched learning clock and fresh data."""

from __future__ import annotations

import copy
from pathlib import Path
from typing import Any

from ..provenance import artifact_records, canonical_json_bytes, cid_bytes, tree_cid
from ..zoology_english_binding import contract as english
from ..zoology_english_diagnostic.contract import SOURCE_CIDS, _envelope
from . import data

ISSUE = 1067
POLICY = "ZoologyQueryObjectAnswerReadoutV1"
MODEL_CONFIG = copy.deepcopy(english.MODEL_CONFIG)
TRAINING = copy.deepcopy(english.TRAINING)
EVALUATION = {
    key: english.EVALUATION[key]
    for key in (
        "batch_size", "threads", "interop_threads", "construction_known_rows",
        "development_groups", "development_known_rows", "development_unknown_rows",
        "construction_known_min_correct", "development_known_min_correct",
        "complete_groups_min_correct", "per_question_type_groups_min_correct",
        "development_unknown_min_correct",
    )
}
DIAGNOSTIC_CIDS = {
    "preparation": "blake3:5679f721fa60c16601a4d3a8ca46397055b89d7769dfae1ca099a1e3f3fbe5a9",
    "result": "blake3:65b23631b10fe62b215411932cd9fe45f76b43d6b8503d0f2e74dc3d256c9b61",
    "replay": "blake3:7222a680c300552ab097ce184500c90c0e44ede8248c4c3f752aa09f4232c0ca",
}
INTERVENTION = {
    "supervised_position_before": 40,
    "supervised_position_after": 37,
    "input_sequence_length": 41,
    "initialization": "fresh seed 123; no prior learned state",
    "only_training_change": "selected supervised position; all inputs, labels, metadata, model, optimizer, sampler and dropout work retained",
    "interface": "explicit answer label at query object; literal next input token is question mark",
    "construction_miss": "retain partial gains; placement insufficient at fixed dose; no development model decisions or extra dose",
    "development_miss": "construction fits; fresh-combination transfer unresolved",
    "language_pass": "next separate unchanged-R4 inference preservation; no R4 execution in this fit",
}


def _repo() -> Path:
    return Path(__file__).resolve().parents[5]


def _lineage() -> dict[str, Any]:
    repo = _repo()
    documents = {}
    paths = []
    for prefix, cids in (
        ("r4_zoology_english_binding_1063", SOURCE_CIDS),
        ("r4_zoology_english_diagnostic_1065", DIAGNOSTIC_CIDS),
    ):
        documents[prefix] = {}
        for name, expected in cids.items():
            relative = f"docs/{prefix}_{name}.json"
            paths.append(relative)
            documents[prefix][name] = _envelope(repo / relative, f"{name}_cid", expected)
    previous = documents["r4_zoology_english_binding_1063"]
    diagnostic = documents["r4_zoology_english_diagnostic_1065"]
    fitted, original = previous["fit"], previous["result"]
    if (
        previous["preparation"]["training"] != TRAINING
        or previous["preparation"]["model_config"] != MODEL_CONFIG
        or fitted["status"] != "FIT_COMPLETE"
        or fitted["completed_updates"] != TRAINING["total_updates"]
        or diagnostic["result"]["evidence"]["status"] != "CONSTRUCTION_DIAGNOSTIC_COMPLETE"
        or not diagnostic["replay"]["exact_replay"]
        or diagnostic["replay"]["result_cid"] != diagnostic["result"]["result_cid"]
        or diagnostic["result"]["evidence"]["construction"] != original["evidence"]["language"]["construction"]
    ):
        raise ValueError("matched learning clock or diagnostic lineage differs")
    implementation = diagnostic["preparation"]["implementation"]
    files = artifact_records(repo, [row["path"] for row in implementation["files"]])
    if files != implementation["files"] or tree_cid(files) != implementation["tree_cid"]:
        raise ValueError("historical source implementation changed")
    baseline = diagnostic["result"]["evidence"]
    return {
        "english_cids": dict(SOURCE_CIDS),
        "diagnostic_cids": dict(DIAGNOSTIC_CIDS),
        "documents": artifact_records(repo, paths),
        "implementation_files": files,
        "construction_files": previous["preparation"]["dataset"]["files"],
        "baseline": {
            "construction": baseline["construction"],
            "diagnostic": baseline["diagnostic"],
            "fit_work": fitted["work"],
            "model": fitted["artifact"],
        },
    }


def _bindings(root: Path) -> dict[str, Any]:
    lineage = _lineage()
    repo = _repo()
    paths = {row["path"] for row in lineage["implementation_files"] + lineage["documents"]}
    paths.update(str(p.relative_to(repo)) for p in Path(__file__).parent.glob("*.py"))
    paths.update(
        str(p.relative_to(repo))
        for p in (repo / "tools/r4-softmax-trainer/tests").glob("test_zoology_query_readout*.py")
    )
    files = artifact_records(repo, sorted(paths))
    dataset = data.validate(root / "data", inspect_development=False)
    if dataset["source"]["files"] != lineage["construction_files"]:
        raise ValueError("copied source data differs from the published English population")
    return {
        "lineage": lineage,
        "implementation": {"root": str(repo), "files": files, "tree_cid": tree_cid(files)},
        "dataset": dataset,
    }


def prepare(root: Path, source_root: Path) -> dict[str, Any]:
    root, source_root = root.resolve(), source_root.resolve()
    root.mkdir(parents=True, exist_ok=True)
    if (root / "preparation.json").exists():
        raise FileExistsError("readout preparation already exists")
    lineage = _lineage()
    dataset = data.build(root / "data", source_root / "data")
    # Build independently audits all fresh labels, world exclusions and balance.
    # Later validation binds its envelope without opening development tensors.
    body = {
        "schema": "uor-r4.zoology-query-readout-preparation/1",
        "issue": ISSUE, "policy": POLICY,
        "model_config": MODEL_CONFIG, "training": TRAINING,
        "evaluation": EVALUATION, "intervention": INTERVENTION,
        **_bindings(root),
    }
    if body["dataset"] != dataset or body["lineage"] != lineage:
        raise ValueError("preparation inputs changed during construction")
    body["preparation_cid"] = cid_bytes(canonical_json_bytes(body))
    with (root / "preparation.json").open("xb") as handle:
        handle.write(canonical_json_bytes(body))
    return body


def validate_preparation(root: Path) -> dict[str, Any]:
    root = root.resolve()
    body = _envelope(root / "preparation.json", "preparation_cid")
    for key, expected in (
        ("schema", "uor-r4.zoology-query-readout-preparation/1"),
        ("issue", ISSUE), ("policy", POLICY), ("model_config", MODEL_CONFIG),
        ("training", TRAINING), ("evaluation", EVALUATION),
        ("intervention", INTERVENTION),
    ):
        if body.get(key) != expected:
            raise ValueError(f"frozen query-readout {key} differs")
    current = _bindings(root)
    if any(body.get(key) != value for key, value in current.items()):
        raise ValueError("readout source, data or implementation changed")
    return body
