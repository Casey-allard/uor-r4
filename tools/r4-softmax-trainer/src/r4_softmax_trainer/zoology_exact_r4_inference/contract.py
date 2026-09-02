"""Bind #1057 inputs to the unchanged #1059 inference implementation."""

from __future__ import annotations

from pathlib import Path
from typing import Any

import torch
from safetensors import safe_open
from torch import Tensor

from ..provenance import artifact_records, canonical_json_bytes, cid_bytes, tree_cid
from ..zoology_r4_inference import contract as prior

ISSUE = 1061
POLICY = "ZoologyExactDataCoherentR4InferenceV1"
EVALUATION = {
    **prior.EVALUATION,
    "rows": 1024,
    "queries_per_row": 8,
    "sequence_length": 120,
    "vocab_size": 4096,
    "historical_correct": 8071,
    "row_order": "canonical_file_order_0_to_1023",
}
SOURCE_PREPARATION_CID = (
    "blake3:4a01fada99551fa7360520fdda18890bdd3d4f3b8c5ed03cfb41aee20150eeaa"
)
SOURCE_RESULT_CID = (
    "blake3:35b1cedfd51385bf98277a4527b1ce05f5dd3b93fffe125a5ea28c2a34b6387c"
)
SOURCE_MODEL_CID = (
    "blake3:69af5586eccfceab4214e9f13524eeea578eb3facaea4fdedec89f0b5d217445"
)
SOURCE_STATE_CID = (
    "blake3:f2a67ec0cc7ac44f586b815da43efabcc81d444b1bab9954b5536c37cb96ff90"
)
SOURCE_DATA_CID = (
    "blake3:96f154042f0fd920c7f6f3b1b650a6ce20f11c401f9ae0c81734f47ae231b7f1"
)
INTEGRATION_PREPARATION_CID = (
    "blake3:bed7eae03c7f3bfa7e2b5ff3786f87d878f42c9eb5d8465b5e37322073cdd588"
)
INTEGRATION_RESULT_CID = (
    "blake3:bdf5a440562bf31a6c0d6d53cef0454270638b87508f0a758aaf9eb3a0031f7d"
)
INTEGRATION_REPLAY_CID = (
    "blake3:458f6f8817203e57089580d851971d7d32234c5d9e4edf96967984097bd7f181"
)
PREPARATION_NAME = "preparation.json"


def _repo() -> Path:
    return Path(__file__).resolve().parents[5]


def _integration() -> dict[str, Any]:
    repo = _repo()
    prefix = "docs/r4_zoology_coherent_inference_1059_"
    preparation = prior._envelope(
        repo / f"{prefix}preparation.json",
        "preparation_cid",
        INTEGRATION_PREPARATION_CID,
    )
    result = prior._envelope(
        repo / f"{prefix}result.json", "result_cid", INTEGRATION_RESULT_CID
    )
    replay = prior._envelope(
        repo / f"{prefix}replay.json", "replay_cid", INTEGRATION_REPLAY_CID
    )
    if (
        result["preparation_cid"] != preparation["preparation_cid"]
        or replay["result_cid"] != result["result_cid"]
        or not replay["exact_replay"]
        or not result["evidence"]["primary"]["passed"]
        or not result["evidence"]["control"]["strong_transport_sensitivity"]
    ):
        raise ValueError("qualified #1059 result/replay relationship differs")
    records = preparation["implementation"]["files"]
    if artifact_records(repo, [row["path"] for row in records]) != records:
        raise ValueError("historical #1059 inference implementation changed")
    return {
        "preparation_cid": preparation["preparation_cid"],
        "result_cid": result["result_cid"],
        "replay_cid": replay["replay_cid"],
        "frames": preparation["frames"],
        "implementation_files": records,
        "documents": artifact_records(
            repo,
            [f"{prefix}{name}.json" for name in ("preparation", "result", "replay")],
        ),
    }


def _source_contract(root: Path) -> dict[str, Any]:
    preparation = prior._envelope(
        root / "preparation.json", "preparation_cid", SOURCE_PREPARATION_CID
    )
    result = prior._envelope(root / "result.json", "result_cid", SOURCE_RESULT_CID)
    primary = prior._envelope(root / "primary/result.json", "primary_cid")
    if (
        result["primary"] != primary
        or result["preparation_cid"] != preparation["preparation_cid"]
        or result["dataset"] != preparation["dataset"]
        or primary["blocks"] != 40
        or primary["completed_updates"] != 7840
        or primary["final_development"]["top1_correct"] != 8071
        or primary["final_development"]["decisions"] != 8192
        or result["control"]["status"] != "NOT_RUN_PRIMARY_MISS"
    ):
        raise ValueError("retained final checkpoint evidence differs")
    trainer = _repo() / "tools/r4-softmax-trainer"
    records = preparation["implementation"]["files"]
    if artifact_records(trainer, [row["path"] for row in records]) != records:
        raise ValueError("historical #1057 source implementation changed")
    artifact = primary["artifact"]
    model = prior._record(root, artifact["path"], cid=SOURCE_MODEL_CID)
    config = artifact["config"]
    if (
        model["bytes"] != artifact["bytes"]
        or model["bytes"] != 1_217_024
        or artifact["state_cid"] != SOURCE_STATE_CID
        or (
            config["vocab_size"],
            config["max_position_embeddings"],
            config["d_model"],
            config["n_layers"],
            config["num_heads"],
        )
        != (4096, 120, 64, 2, 1)
    ):
        raise ValueError("retained artifact/config/state identity differs")
    model.update(state_cid=SOURCE_STATE_CID, config=config)
    data_root = Path(preparation["dataset_root"])
    if not data_root.is_absolute():
        raise ValueError("retained dataset root must be absolute")
    data_root = data_root.resolve()
    dataset = prior._record(
        data_root, preparation["dataset"]["path"], cid=SOURCE_DATA_CID
    )
    if dataset["bytes"] != preparation["dataset"]["bytes"]:
        raise ValueError("retained data size differs")
    dataset.update(
        root=str(data_root),
        shapes={
            key: preparation["dataset"]["shapes"][key]
            for key in ("test_inputs", "test_positions", "test_targets")
        },
    )
    return {
        "root": str(root),
        "preparation_cid": preparation["preparation_cid"],
        "result_cid": result["result_cid"],
        "primary_cid": primary["primary_cid"],
        "model": model,
        "dataset": dataset,
        "historical_development": primary["final_development"],
        "historical_control": result["control"],
        "documents": artifact_records(
            root, ["preparation.json", "result.json", "primary/result.json"]
        ),
        "implementation_files": records,
        "checkpoint_reads": 0,
        "evaluation_rng_reads": 0,
        "physical_binding_control_reads": 0,
    }


def _implementation(
    integration: dict[str, Any], source: dict[str, Any]
) -> dict[str, Any]:
    repo = _repo()
    trainer = repo / "tools/r4-softmax-trainer"
    paths = {row["path"] for row in integration["implementation_files"]}
    paths.update(row["path"] for row in integration["documents"])
    paths.update(
        f"tools/r4-softmax-trainer/{row['path']}"
        for row in source["implementation_files"]
    )
    paths.update(str(p.relative_to(repo)) for p in Path(__file__).parent.glob("*.py"))
    paths.update(
        str(p.relative_to(repo))
        for p in (trainer / "tests").glob("test_zoology_exact_r4*.py")
    )
    records = artifact_records(repo, sorted(paths))
    return {"root": str(repo), "files": records, "tree_cid": tree_cid(records)}


def _bindings(source_root: Path, frames_root: Path) -> dict[str, Any]:
    integration = _integration()
    source = _source_contract(source_root)
    frames = prior._frame_contract(frames_root)
    # Relocation is allowed; the exact native files and their identities are not changed.
    if {k: v for k, v in frames.items() if k != "root"} != {
        k: v for k, v in integration["frames"].items() if k != "root"
    }:
        raise ValueError("native frames differ from the qualified #1059 bundle")
    return {
        "source": source,
        "frames": frames,
        "integration": {
            k: v
            for k, v in integration.items()
            if k not in ("frames", "implementation_files")
        },
        "implementation": _implementation(integration, source),
    }


def prepare(root: Path, source_root: Path, frames_root: Path) -> dict[str, Any]:
    root, source_root, frames_root = (
        p.resolve() for p in (root, source_root, frames_root)
    )
    root.mkdir(parents=True, exist_ok=True)
    path = root / PREPARATION_NAME
    if path.exists():
        raise FileExistsError("preparation exists; do not overwrite")
    body = {
        "schema": "uor-r4.zoology-exact-r4-inference-preparation/1",
        "issue": ISSUE,
        "policy": POLICY,
        "evaluation": dict(EVALUATION),
        **_bindings(source_root, frames_root),
    }
    body["preparation_cid"] = cid_bytes(canonical_json_bytes(body))
    with path.open("xb") as handle:
        handle.write(canonical_json_bytes(body))
    return body


def validate_preparation(root: Path) -> dict[str, Any]:
    body = prior._envelope(root / PREPARATION_NAME, "preparation_cid")
    if (
        body.get("issue") != ISSUE
        or body.get("policy") != POLICY
        or body.get("evaluation") != EVALUATION
    ):
        raise ValueError("exact-data inference policy changed")
    current = _bindings(Path(body["source"]["root"]), Path(body["frames"]["root"]))
    if any(body.get(key) != value for key, value in current.items()):
        raise ValueError("source/frame/implementation binding changed")
    return body


def load_development(preparation: dict[str, Any]) -> dict[str, Tensor]:
    """Read the three development tensors in stored order from their own root."""
    source = preparation["source"]
    data = source["dataset"]
    path = prior._path(Path(data["root"]), data["path"])
    shapes = {
        "test_inputs": (EVALUATION["rows"], EVALUATION["sequence_length"]),
        "test_positions": (EVALUATION["rows"], EVALUATION["queries_per_row"]),
        "test_targets": (EVALUATION["rows"], EVALUATION["queries_per_row"]),
    }
    with safe_open(path, framework="pt", device="cpu") as handle:
        tensors = {key: handle.get_tensor(key).contiguous() for key in shapes}
    if any(
        tensors[key].dtype != torch.long or tensors[key].shape != shape
        for key, shape in shapes.items()
    ):
        raise ValueError("development tensor shapes/dtypes differ")
    if (
        torch.any(tensors["test_inputs"] < 0)
        or torch.any(tensors["test_inputs"] >= EVALUATION["vocab_size"])
        or torch.any(tensors["test_targets"] < 0)
        or torch.any(tensors["test_targets"] >= EVALUATION["vocab_size"])
        or torch.any(tensors["test_positions"] < 0)
        or torch.any(tensors["test_positions"] >= EVALUATION["sequence_length"])
    ):
        raise ValueError("development token/position outside bound model domain")
    return tensors
