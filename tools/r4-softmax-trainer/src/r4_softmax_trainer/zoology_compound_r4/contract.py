"""Bind the disclosed #1073 populations and weights to the qualified R4 bundle.

Preparation hashes the retained fitted artifact without loading its tensors.
The structural preflight reads already-observed inputs, never a checkpoint or
fitted model. No population is generated or copied by this inference contract.
"""

from __future__ import annotations

from pathlib import Path
from typing import Any

from ..provenance import artifact_records, canonical_json_bytes, cid_bytes, tree_cid
from ..zoology_compound_binding import data
from ..zoology_compound_binding.model import MODEL_CONFIG, MODEL_POLICY
from ..zoology_r4_inference import contract as prior

ISSUE = 1075
POLICY = "CompoundBindingUnchangedR4InferenceV1"
SCHEMA = "uor-r4.zoology-compound-r4-preparation/1"
EVALUATION = {
    "batch_size": 256,
    "threads": 8,
    "interop_threads": 1,
    "max_elapsed_seconds": 900,
    "max_rss_bytes": 4 * 1024**3,
    "logit_atol": 0.005,
    "attention_atol": 1e-5,
    "nll_atol": 1e-5,
    "rotations": [0, 1, 2, 3],
    "construction_rows": 10240,
    "construction_supported_rows": 8192,
    "construction_unknown_rows": 2048,
    "development_rows": 1280,
    "development_supported_rows": 1024,
    "development_unknown_rows": 256,
    "queries_per_row": 1,
    "sequence_length": 41,
    "vocabulary_size": 4096,
    "strong_control_drop": 0.5,
    "row_order": "canonical retained file order; four right-cyclic fact orders",
    "ordinary_reproduction": "exact all/supported/unknown historical records in each population/order",
    "optimizer_updates": 0,
}
INTERVENTION = {
    "source_issue": 1073,
    "frame_qualification_issue": 1061,
    "population": "previously observed #1073 construction and development; no new generalization claim",
    "prefix_length": 38,
    "query_position": 37,
    "source_positions": [7, 15, 23, 31],
    "null_frame": "identity",
    "mixture": "all four fact entries plus learned null; full softmax mixture retained",
    "attention_shape": "[batch,1,1,5]",
    "score_arithmetic": "float64 frame transport and dot accumulation; cast completed dot to float32 before division by 8",
    "control": "encode in true source frame; transport four facts with source frame (s+1) mod 4; null remains identity",
    "value_cycle": False,
    "optimizer_updates": 0,
    "new_parameters": 0,
    "geometry_changes": 0,
    "checkpoint_optimizer_rng_reads": 0,
    "model_label_arguments": 0,
    "native_map_entries": 8192,
    "primary": "ordinary exact reproduction, coherent full-head/attention/NLL parity and identical predictions in every view",
    "control_admission": "only after complete primary preservation; same work and causal reads required before sensitivity attribution",
    "weak_control": "retain primary preservation separately; no strong transport-sensitivity claim",
    "scope": "fitted structured-binding preservation, not learned English parsing, new transfer, geometry superiority or softmax removal",
}
SOURCE_CIDS = {
    "preparation": "blake3:360b6c6bb63f4040d0baff06c9e56b3038111587bdf234681e6f4d1cdc89d038",
    "fit": "blake3:9c6bc25f9bcfa8279fbba6acf15d0bf1279653652c46222473fcc23fb95daf84",
    "result": "blake3:1f3c5bee5ebd0e8e34f9f1a5fa03d514b397928638fd66deaf64b8abf7946041",
    "replay": "blake3:9500d279e228eb3fff646a537fcbbbf861aafc216cd7e7ad750444381c4a17f2",
}
INTEGRATION_CIDS = {
    "preparation": "blake3:c8e97664f7feab8c83ad15d298620da675bc3f156a9b0dcfcfa98ac69fad6c35",
    "result": "blake3:ac2ec4d533ac47d25f8eb9dfd7a41147147d73c0e2d9531352d9f9fb2eb84e58",
    "replay": "blake3:af6c239ec2d0e11f26f50f74150c992dea345ec21257141fcec1096a573e708e",
}
SOURCE_FILE_COUNT = 270
SOURCE_PATHS = {
    "preparation": "preparation.json",
    "fit": "fit/fit.json",
    "result": "result.json",
    "replay": "replay.json",
}


def _repo() -> Path:
    return Path(__file__).resolve().parents[5]


def _published(stem: str, expected: dict[str, str]) -> tuple[dict, list[dict]]:
    paths = [f"docs/{stem}_{name}.json" for name in expected]
    documents = {
        name: prior._envelope(_repo() / path, f"{name}_cid", expected[name])
        for name, path in zip(expected, paths, strict=True)
    }
    return documents, artifact_records(_repo(), paths)


def _source_contract(root: Path) -> dict[str, Any]:
    documents, published = _published("r4_zoology_compound_binding_1073", SOURCE_CIDS)
    for name, relative in SOURCE_PATHS.items():
        if (
            prior._envelope(root / relative, f"{name}_cid", SOURCE_CIDS[name])
            != documents[name]
        ):
            raise ValueError("retained #1073 envelope differs from its publication")
    preparation, fitted = documents["preparation"], documents["fit"]
    result, replay = documents["result"], documents["replay"]
    evidence = result["evidence"]
    if (
        preparation["model_config"] != MODEL_CONFIG
        or preparation["model_policy"] != MODEL_POLICY
        or fitted["status"] != "FIT_COMPLETE"
        or fitted["completed_updates"] != 3920
        or fitted["work"]["optimizer_updates"] != 3920
        or fitted["work"]["train_query_presentations"] != 2007040
        or fitted["preparation_cid"] != preparation["preparation_cid"]
        or result["preparation_cid"] != preparation["preparation_cid"]
        or result["fit_cid"] != fitted["fit_cid"]
        or result["artifact"] != fitted["artifact"]
        or evidence["status"] != "COMPOUND_BINDING_FRESH_PASSED"
        or not evidence["passed"]
        or not all(evidence["criteria"].values())
        or result["evidence_cid"] != cid_bytes(canonical_json_bytes(evidence))
        or not replay["exact_replay"]
        or not replay["fresh_process"]
        or replay["optimizer_updates"] != 0
        or replay["preparation_cid"] != preparation["preparation_cid"]
        or replay["fit_cid"] != fitted["fit_cid"]
        or replay["result_cid"] != result["result_cid"]
        or replay["evidence_cid"] != result["evidence_cid"]
        or replay["artifact"] != fitted["artifact"]
        or replay["runtime"] != result["runtime"]
    ):
        raise ValueError("qualified #1073 source/result/replay relationship differs")
    for name in ("construction_views", "development"):
        views = (
            evidence[name] if name == "construction_views" else evidence[name]["views"]
        )
        if [row["rotation"] for row in views] != EVALUATION["rotations"]:
            raise ValueError("qualified #1073 population/orders differ")
    if evidence["development"]["model_decisions"] != 5120:
        raise ValueError("qualified #1073 development population is incomplete")
    implementation = preparation["implementation"]
    files = artifact_records(_repo(), [row["path"] for row in implementation["files"]])
    if (
        len(files) != SOURCE_FILE_COUNT
        or files != implementation["files"]
        or tree_cid(files) != implementation["tree_cid"]
    ):
        raise ValueError("historical #1073 implementation changed")
    model = fitted["artifact"]
    if (
        model["path"] != "fit/model.safetensors"
        or model["config"] != MODEL_CONFIG
        or model["model_policy"] != MODEL_POLICY
        or model["bytes"] != 1148672
        or evidence["learned_state_before"] != model["state_cid"]
        or evidence["learned_state_after"] != model["state_cid"]
    ):
        raise ValueError("retained #1073 model policy or tensor identity differs")
    actual = prior._record(root, model["path"], cid=model["cid"])
    if any(model[key] != value for key, value in actual.items()):
        raise ValueError("retained #1073 model file changed")
    dataset = data.validate(root / "data", inspect_development=False)
    if dataset != preparation["dataset"]:
        raise ValueError("retained #1073 data manifest or construction differs")
    # Hash the already-observed development file without loading its tensors.
    for record in dataset["files"]:
        if prior._record(root / "data", record["path"], cid=record["cid"]) != record:
            raise ValueError("retained #1073 population file changed")
    return {
        "root": str(root),
        **{f"{name}_cid": documents[name][f"{name}_cid"] for name in SOURCE_CIDS},
        "evidence_cid": result["evidence_cid"],
        "model": model,
        "dataset": dataset,
        "runtime": result["runtime"],
        "baseline_history": evidence,
        "documents": artifact_records(root, list(SOURCE_PATHS.values())),
        "public_documents": published,
        "implementation_files": files,
        "checkpoint_reads": 0,
        "new_population_generation": 0,
    }


def _integration() -> dict[str, Any]:
    documents, published = _published(
        "r4_zoology_exact_coherent_inference_1061", INTEGRATION_CIDS
    )
    preparation, result, replay = (
        documents[name] for name in ("preparation", "result", "replay")
    )
    if (
        result["preparation_cid"] != preparation["preparation_cid"]
        or not result["evidence"]["primary"]["passed"]
        or not result["evidence"]["control"]["strong_transport_sensitivity"]
        or result["evidence_cid"] != cid_bytes(canonical_json_bytes(result["evidence"]))
        or not replay["exact_replay"]
        or not replay["fresh_process"]
        or replay["result_cid"] != result["result_cid"]
        or replay["evidence_cid"] != result["evidence_cid"]
    ):
        raise ValueError("qualified #1061 frame integration relationship differs")
    records = preparation["implementation"]["files"]
    if (
        artifact_records(_repo(), [row["path"] for row in records]) != records
        or tree_cid(records) != preparation["implementation"]["tree_cid"]
    ):
        raise ValueError("historical #1061 implementation changed")
    return {
        **{f"{name}_cid": documents[name][f"{name}_cid"] for name in INTEGRATION_CIDS},
        "frames": preparation["frames"],
        "implementation_files": records,
        "documents": published,
    }


def _preflight(source: dict, frames: dict) -> dict:
    from .campaign import structural_preflight

    result = structural_preflight(source, frames)
    if not result.get("passed"):
        raise ValueError("compound R4 structural preflight did not qualify")
    return result


def _frames(frame_root: Path, integration: dict) -> dict:
    frames = prior._frame_contract(frame_root)
    if {key: value for key, value in frames.items() if key != "root"} != {
        key: value for key, value in integration["frames"].items() if key != "root"
    }:
        raise ValueError("native frame bundle differs from qualified #1061")
    return frames


def _bindings(source_root: Path, frame_root: Path) -> dict[str, Any]:
    source, integration = _source_contract(source_root), _integration()
    frames = _frames(frame_root, integration)
    paths = {
        row["path"]
        for row in source["implementation_files"]
        + source["public_documents"]
        + integration["implementation_files"]
        + integration["documents"]
    }
    paths.update(
        str(path.relative_to(_repo())) for path in Path(__file__).parent.glob("*.py")
    )
    paths.update(
        str(path.relative_to(_repo()))
        for path in (_repo() / "tools/r4-softmax-trainer/tests").glob(
            "test_zoology_compound_r4*.py"
        )
    )
    files = artifact_records(_repo(), sorted(paths))
    return {
        "source": source,
        "frames": frames,
        "integration": {
            key: value for key, value in integration.items() if key != "frames"
        },
        "implementation": {
            "root": str(_repo()),
            "files": files,
            "tree_cid": tree_cid(files),
        },
        "preflight": _preflight(source, frames),
    }


def prepare(root: Path, source_root: Path, frame_root: Path) -> dict[str, Any]:
    root, source_root, frame_root = (
        path.resolve() for path in (root, source_root, frame_root)
    )
    root.mkdir(parents=True, exist_ok=True)
    path = root / "preparation.json"
    if path.exists():
        raise FileExistsError("compound R4 preparation exists; do not overwrite")
    body = {
        "schema": SCHEMA,
        "issue": ISSUE,
        "policy": POLICY,
        "evaluation": EVALUATION,
        "intervention": INTERVENTION,
        **_bindings(source_root, frame_root),
    }
    body["preparation_cid"] = cid_bytes(canonical_json_bytes(body))
    with path.open("xb") as handle:
        handle.write(canonical_json_bytes(body))
    return body


def validate_preparation(root: Path) -> dict[str, Any]:
    body = prior._envelope(root.resolve() / "preparation.json", "preparation_cid")
    for key, expected in (
        ("schema", SCHEMA),
        ("issue", ISSUE),
        ("policy", POLICY),
        ("evaluation", EVALUATION),
        ("intervention", INTERVENTION),
    ):
        if body.get(key) != expected:
            raise ValueError(f"frozen compound R4 {key} differs")
    current = _bindings(Path(body["source"]["root"]), Path(body["frames"]["root"]))
    if any(body.get(key) != value for key, value in current.items()):
        raise ValueError("compound R4 source/frame/implementation/preflight changed")
    return body
