"""Freeze the lexical curriculum, source cell, learning clock and R4 lineage."""

from __future__ import annotations

from dataclasses import asdict
from pathlib import Path
from typing import Any

from ..provenance import artifact_records, canonical_json_bytes, cid_bytes, tree_cid
from ..zoology_control.model import ZoologyFigure2Config
from ..zoology_r4_inference import contract as prior
from . import data

ISSUE = 1063
POLICY = "ZoologyEnglishSuppliedContextBindingV1"
MODEL_CONFIG = asdict(ZoologyFigure2Config())
TRAINING = {
    "seed": 123,
    "batch_size": 512,
    "supported_updates": 2352,
    "total_updates": 3920,
    "updates_per_block": 196,
    "cosine_blocks": 64,
    "learning_rate": 0.00046415888336127773,
    "weight_decay": 0.1,
    "threads": 8,
    "interop_threads": 1,
    "admission_updates": 8,
    "projection_safety_factor": 1.25,
    "evaluation_allowance_seconds": 60.0,
    "max_elapsed_seconds": 1800.0,
    "max_rss_bytes": 4 * 1024**3,
    "checkpoint_interval": 16,
}
EVALUATION = {
    "batch_size": 256,
    "threads": 8,
    "interop_threads": 1,
    "construction_known_rows": 8192,
    "development_groups": 256,
    "development_known_rows": 1024,
    "development_unknown_rows": 256,
    "construction_known_min_correct": 8111,
    "development_known_min_correct": 973,
    "complete_groups_min_correct": 231,
    "per_question_type_groups_min_correct": 116,
    "development_unknown_min_correct": 244,
    "logit_atol": prior.EVALUATION["logit_atol"],
    "attention_atol": prior.EVALUATION["attention_atol"],
    "nll_atol": prior.EVALUATION["nll_atol"],
    "strong_control_drop": prior.EVALUATION["strong_control_drop"],
    "row_order": "canonical group order; base q0,q1,swapped q0,q1,missing q0",
    "control": prior.EVALUATION["control"],
    "artifact_selection": "fixed final update 3920; no development selection",
}
PREDECESSOR = {
    "preparation": "blake3:c8e97664f7feab8c83ad15d298620da675bc3f156a9b0dcfcfa98ac69fad6c35",
    "result": "blake3:ac2ec4d533ac47d25f8eb9dfd7a41147147d73c0e2d9531352d9f9fb2eb84e58",
    "replay": "blake3:af6c239ec2d0e11f26f50f74150c992dea345ec21257141fcec1096a573e708e",
}


def _repo() -> Path:
    return Path(__file__).resolve().parents[5]


def _lineage() -> dict[str, Any]:
    repo = _repo()
    prefix = "docs/r4_zoology_exact_coherent_inference_1061_"
    documents = {}
    for name, expected in PREDECESSOR.items():
        documents[name] = prior._envelope(
            repo / f"{prefix}{name}.json", f"{name}_cid", expected
        )
    preparation, result, replay = (
        documents[name] for name in ("preparation", "result", "replay")
    )
    if (
        result["preparation_cid"] != preparation["preparation_cid"]
        or replay["result_cid"] != result["result_cid"]
        or replay["exact_replay"] is not True
        or result["evidence"]["primary"]["passed"] is not True
        or result["evidence"]["control"]["strong_transport_sensitivity"] is not True
    ):
        raise ValueError("qualified exact-data R4 lineage differs")
    implementation = preparation["implementation"]
    records = artifact_records(repo, [row["path"] for row in implementation["files"]])
    if (
        records != implementation["files"]
        or tree_cid(records) != implementation["tree_cid"]
    ):
        raise ValueError("qualified predecessor implementation changed")
    return {
        "cids": dict(PREDECESSOR),
        "frames": preparation["frames"],
        "implementation_files": records,
        "documents": artifact_records(
            repo, [f"{prefix}{name}.json" for name in PREDECESSOR]
        ),
    }


def _bindings(root: Path, frames_root: Path) -> dict[str, Any]:
    lineage = _lineage()
    frames = prior._frame_contract(frames_root)
    if {k: v for k, v in frames.items() if k != "root"} != {
        k: v for k, v in lineage["frames"].items() if k != "root"
    }:
        raise ValueError("native frames changed from #1061")
    repo = _repo()
    paths = {row["path"] for row in lineage["implementation_files"]}
    paths.update(row["path"] for row in lineage["documents"])
    paths.update(str(p.relative_to(repo)) for p in Path(__file__).parent.glob("*.py"))
    paths.update(
        str(p.relative_to(repo))
        for p in (repo / "tools/r4-softmax-trainer/tests").glob(
            "test_zoology_english_binding*.py"
        )
    )
    records = artifact_records(repo, sorted(paths))
    return {
        "lineage": {"cids": lineage["cids"], "documents": lineage["documents"]},
        "frames": frames,
        "implementation": {
            "root": str(repo),
            "files": records,
            "tree_cid": tree_cid(records),
        },
        "dataset": data.validate(root / "data"),
    }


def prepare(root: Path, frames_root: Path) -> dict[str, Any]:
    root, frames_root = root.resolve(), frames_root.resolve()
    root.mkdir(parents=True, exist_ok=True)
    if (root / "preparation.json").exists():
        raise FileExistsError("English preparation exists; do not overwrite")
    data.build(root / "data")
    body = {
        "schema": "uor-r4.zoology-english-binding-preparation/1",
        "issue": ISSUE,
        "policy": POLICY,
        "model_config": dict(MODEL_CONFIG),
        "training": dict(TRAINING),
        "evaluation": dict(EVALUATION),
        "data_policy": dict(data.DATA_POLICY),
        **_bindings(root, frames_root),
    }
    body["preparation_cid"] = cid_bytes(canonical_json_bytes(body))
    with (root / "preparation.json").open("xb") as handle:
        handle.write(canonical_json_bytes(body))
    return body


def validate_preparation(root: Path) -> dict[str, Any]:
    root = root.resolve()
    body = prior._envelope(root / "preparation.json", "preparation_cid")
    for field, expected in (
        ("schema", "uor-r4.zoology-english-binding-preparation/1"),
        ("issue", ISSUE),
        ("policy", POLICY),
        ("model_config", MODEL_CONFIG),
        ("training", TRAINING),
        ("evaluation", EVALUATION),
        ("data_policy", data.DATA_POLICY),
    ):
        if body.get(field) != expected:
            raise ValueError(f"frozen English {field} differs")
    current = _bindings(root, Path(body["frames"]["root"]))
    if any(body.get(key) != value for key, value in current.items()):
        raise ValueError("English data, implementation, lineage or frames changed")
    return body
