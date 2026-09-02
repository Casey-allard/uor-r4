"""Freeze direct owner-plus-object query encoding at the matched learning dose."""

from __future__ import annotations

import copy
from pathlib import Path
from typing import Any

from ..provenance import artifact_records, canonical_json_bytes, cid_bytes, tree_cid
from ..zoology_english_diagnostic.contract import _envelope
from ..zoology_query_readout import contract as previous
from . import data
from .model import QUERY_ENCODING

ISSUE = 1069
POLICY = "ZoologyDirectJointQueryEncodingV1"
MODEL_CONFIG = copy.deepcopy(previous.MODEL_CONFIG)
TRAINING = copy.deepcopy(previous.TRAINING)
EVALUATION = copy.deepcopy(previous.EVALUATION)
BEHAVIOR = {
    "baseline_owner_both_correct": 47,
    "owner_both_correct_min": 150,
    "owner_pairs": 2048,
    "owner_worlds": 1024,
    "construction_correct_min": 3735,
    "object_both_correct_min": 447,
    "object_pairs": 2048,
    "decision": "owner gain and overall/object preservation; construction gate separately controls development",
}
SOURCE_CIDS = {
    "preparation": "blake3:b6df21b50fe67696910e8e01ae2aa590c9e9b6aebf15b090cc11b98edc5a82d3",
    "fit": "blake3:ec2dea07ef3b2eaf3d6532830c0434c33935f72df982b16929ecc6fc48be08e8",
    "result": "blake3:c6dfcb3a856963ab4493c3d26bf729f6d9cad70147316ef2b9b62e87c3116369",
    "replay": "blake3:98c799c7844e36d68b56c6948824c1dacb53fb36e9f38bc7c052ec6fe0873fac",
}
INTERVENTION = {
    "query_encoding": QUERY_ENCODING,
    "supervised_position_before": 37,
    "supervised_position_after": 37,
    "input_sequence_length": 41,
    "initialization": "fresh seed 123; no prior learned state",
    "training_change": "fixed owner-word embedding residual at the query object; same inputs, labels, metadata, parameters, optimizer, sampler and dropout work",
    "interface": "explicit answer label at query object; literal next input token is question mark",
    "causal_scope": "owner already in causal prefix; intervention changes direct access, embedding scale and gradient paths together",
    "partial_positive": "retain improved construction baseline; address recorded remaining error pattern in a separate step",
    "partial_negative": "retain evidence but reject residual as improved baseline; retain #1067 and revise binding learning recipe",
    "development_miss": "construction fits; fresh-combination transfer unresolved",
    "language_pass": "next separate unchanged-R4 inference preservation; no R4 execution in this fit",
}


def _repo() -> Path:
    return Path(__file__).resolve().parents[5]


def _lineage() -> dict[str, Any]:
    repo = _repo()
    historical = previous._lineage()
    documents, paths = {}, []
    for name, expected in SOURCE_CIDS.items():
        relative = f"docs/r4_zoology_query_readout_1067_{name}.json"
        paths.append(relative)
        documents[name] = _envelope(repo / relative, f"{name}_cid", expected)
    preparation, fitted = documents["preparation"], documents["fit"]
    result, replay = documents["result"], documents["replay"]
    if (
        preparation["training"] != TRAINING
        or preparation["model_config"] != MODEL_CONFIG
        or fitted["status"] != "FIT_COMPLETE"
        or fitted["completed_updates"] != TRAINING["total_updates"]
        or fitted["work"] != historical["baseline"]["fit_work"]
        or result["fit_cid"] != fitted["fit_cid"]
        or result["preparation_cid"] != preparation["preparation_cid"]
        or result["evidence"]["status"] != "QUERY_OBJECT_READOUT_CONSTRUCTION_MISS"
        or not replay["exact_replay"]
        or replay["result_cid"] != result["result_cid"]
    ):
        raise ValueError("matched query-readout lineage or learning clock differs")
    implementation = preparation["implementation"]
    files = artifact_records(repo, [row["path"] for row in implementation["files"]])
    if (
        files != implementation["files"]
        or tree_cid(files) != implementation["tree_cid"]
    ):
        raise ValueError("historical source implementation changed")
    baseline = result["evidence"]
    paired = baseline["construction_diagnostic"]["paired"]["question"]["pair_type"]
    if (
        baseline["construction"]["top1_correct"] != BEHAVIOR["construction_correct_min"]
        or paired["same_object"]["both_correct"]
        != BEHAVIOR["baseline_owner_both_correct"]
        or paired["same_owner"]["both_correct"] != BEHAVIOR["object_both_correct_min"]
    ):
        raise ValueError("frozen behavior comparison does not match predecessor")
    return {
        "readout_cids": dict(SOURCE_CIDS),
        "documents": artifact_records(repo, paths),
        "implementation_files": files,
        "construction_files": historical["construction_files"],
        "baseline": {
            "construction": baseline["construction"],
            "diagnostic": baseline["construction_diagnostic"],
            "fit_work": fitted["work"],
            "model": fitted["artifact"],
        },
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
            "test_zoology_joint_query*.py"
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


def prepare(root: Path, source_root: Path) -> dict[str, Any]:
    root, source_root = root.resolve(), source_root.resolve()
    root.mkdir(parents=True, exist_ok=True)
    if (root / "preparation.json").exists():
        raise FileExistsError("joint-query preparation already exists")
    lineage = _lineage()
    dataset = data.build(root / "data", source_root / "data")
    # Build independently audits all fresh labels, world exclusions and balance.
    # Later validation binds its envelope without opening development tensors.
    body = {
        "schema": "uor-r4.zoology-joint-query-preparation/1",
        "issue": ISSUE,
        "policy": POLICY,
        "model_config": MODEL_CONFIG,
        "training": TRAINING,
        "evaluation": EVALUATION,
        "behavior": BEHAVIOR,
        "intervention": INTERVENTION,
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
        ("schema", "uor-r4.zoology-joint-query-preparation/1"),
        ("issue", ISSUE),
        ("policy", POLICY),
        ("model_config", MODEL_CONFIG),
        ("training", TRAINING),
        ("evaluation", EVALUATION),
        ("behavior", BEHAVIOR),
        ("intervention", INTERVENTION),
    ):
        if body.get(key) != expected:
            raise ValueError(f"frozen joint-query {key} differs")
    current = _bindings(root)
    if any(body.get(key) != value for key, value in current.items()):
        raise ValueError("joint-query source, data or implementation changed")
    return body
