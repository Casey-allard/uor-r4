"""Bind a new language interface to the preserved, unchanged compound core."""

from __future__ import annotations

from pathlib import Path
from typing import Any

from ..provenance import artifact_records, canonical_json_bytes, cid_bytes, tree_cid
from ..zoology_compound_r4 import contract as previous
from .model import MODEL_CONFIG, MODEL_POLICY

ISSUE = 1077
POLICY = "LearnedClauseRoleInterfaceV1"
SCHEMA = "uor-r4.zoology-language-interface-preparation/1"
TRAINING = {
    "seed": 123,
    "updates": 512,
    "batch_size": 128,
    "learning_rate": 0.003,
    "optimizer": "AdamW",
    "betas": [0.9, 0.999],
    "eps": 1e-8,
    "weight_decay": 0.01,
    "clip_grad_norm": 1.0,
    "role_labels_per_row": 14,
    "row_presentations": 65536,
    "role_label_presentations": 917504,
    "answer_loss_weight": 0.0,
    "core_optimizer_updates": 0,
    "runtime_plans": [4, 8],
    "interop_threads": 1,
    "available_cpu_threads": 8,
    "calibration_seed": 10770,
    "calibration_warmup_steps": 2,
    "calibration_timed_steps": 4,
    "calibration_optimizer_updates": 0,
    "calibration_loss_atol": 1e-6,
    "calibration_gradient_atol": 1e-6,
    "calibration_projection_multiplier": 2.0,
    "admission_steps": 8,
    "admission_remaining_multiplier": 2.0,
    "evaluation_allowance_seconds": 60.0,
    "max_elapsed_seconds": 900,
    "max_rss_bytes": 4 * 1024**3,
}
EVALUATION = {
    "batch_size": 256,
    "construction_views": [0, 1],
    "development_seen_views": [0, 1],
    "development_heldout_views": [2, 3],
    "construction_rows": 10240,
    "development_rows": 1280,
    "supported_min_rate": 0.95,
    "unknown_min_rate": 0.95,
    "complete_quartet_min_rate": 0.95,
    "role_top1_min_rate": 0.99,
    "each_role_top1_min_rate": 0.99,
    "syntax_paired_complete_min_rate": 0.95,
    "control_supported_drop": 0.5,
    "control_reassigned_min_rate": 0.95,
    "control_unknown_min_rate": 0.95,
    "control_exact_attention": True,
    "oracle_min_rate": 1.0,
    "max_elapsed_seconds": 900,
    "max_rss_bytes": 4 * 1024**3,
    "runtime": "selected qualified CPU plan from fit; identical replay plan",
}
INTERVENTION = {
    "source_issue": 1073,
    "preservation_issue": 1075,
    "core_parameter_count": 286976,
    "core_frozen": True,
    "role_supervision": "14 owner/object/location pointer labels per construction row",
    "interface": "given clause boundaries; learned all-token soft role pooling; no absolute role positions",
    "lexical_scope": "local owner disambiguation; object/location vocabularies remain disjoint",
    "syntax_preflight": "paired owner contrasts have identical query token bags and identical fact inputs",
    "oracle": "canonical-input frozen core must score 100% supported and unknown before each primary view is interpreted",
    "model_oracle_arguments": 0,
    "model_role_label_arguments": 0,
    "geometry_changes": 0,
    "r4_execution": 0,
    "development_admission": "all construction primary criteria and all construction value controls pass",
    "development_selection": "one fixed final reader; seen and held-out syntax views judged separately",
    "control": "right-cycle four projected fact values only; Q/K/attention/null remain fixed",
    "success_status": "LANGUAGE_INTERFACE_HELDOUT_PASSED",
    "miss_action": "retain measured role improvements and frozen binding core; do not score development after construction/control miss",
    "scope": "supervised bounded language interface on observed #1073 semantics; new syntax combinations for the reader, not general English or new semantic generalization",
}

SOURCE_CIDS = {
    "preparation": "blake3:f034f6af62f0f1e6f1f7be33a93f728bfd2582b1ea10025ec1c1f5ae1835240b",
    "result": "blake3:358506253f842a1843dce2652f1004a792a8a6b056a361f8ac66ddd5babb31af",
    "replay": "blake3:0310a7797f8db047cc54b9641a5b90b987b3a3b4ac706f4d9e580738f0fa8de1",
}
SOURCE_FILE_COUNT = 282
FRAME_TREE_CID = (
    "blake3:94762441a43b03f596a66131ec34af15bba3afbc2bbc5d28ab7dfdabd9b6d68c"
)


def _repo() -> Path:
    return Path(__file__).resolve().parents[5]


def _lineage(source_root: Path, prior_root: Path) -> dict[str, Any]:
    """Validate published #1075 evidence without executing its frame preflight.

    The existing #1073 source helper binds its model bytes, data and source.
    Only the retained source root may move; all content identities must remain.
    Native frames are bound by the published envelope and are never opened.
    """
    source_root, prior_root = source_root.resolve(), prior_root.resolve()
    source = previous._source_contract(source_root)
    paths = {
        name: f"docs/r4_zoology_compound_r4_1075_{name}.json" for name in SOURCE_CIDS
    }
    documents = {}
    for name, path in paths.items():
        public = previous.prior._envelope(
            _repo() / path, f"{name}_cid", SOURCE_CIDS[name]
        )
        local = previous.prior._envelope(
            prior_root / f"{name}.json", f"{name}_cid", SOURCE_CIDS[name]
        )
        if local != public:
            raise ValueError("retained #1075 evidence differs from publication")
        documents[name] = public
    preparation, result, replay = (
        documents[name] for name in ("preparation", "result", "replay")
    )
    if {key: value for key, value in source.items() if key != "root"} != {
        key: value for key, value in preparation["source"].items() if key != "root"
    }:
        raise ValueError("retained #1073 source differs from preserved #1075 source")
    evidence = result["evidence"]
    model = source["model"]
    if (
        preparation["issue"] != 1075
        or result["issue"] != 1075
        or replay["issue"] != 1075
        or result["preparation_cid"] != preparation["preparation_cid"]
        or result["source_result_cid"] != source["result_cid"]
        or result["model"] != model
        or result["frames"] != preparation["frames"]
        or preparation["frames"]["tree_cid"] != FRAME_TREE_CID
        or result["runtime"] != source["runtime"]
        or result["implementation_cid"] != preparation["implementation"]["tree_cid"]
        or evidence["status"] != "COMPOUND_R4_PRESERVED"
        or not evidence["preserved"]
        or not evidence["ordinary_exact_reproduction"]
        or not evidence["primary"]["passed"]
        or not evidence["control"]["valid"]
        or not evidence["control"]["strong_transport_sensitivity"]
        or evidence["learned_state_before"] != model["state_cid"]
        or evidence["learned_state_after"] != model["state_cid"]
        or evidence["model_file_cid"] != model["cid"]
        or evidence["parameter_count"] != 286976
        or evidence["optimizer_updates"] != 0
        or evidence["new_parameters"] != 0
        or evidence["geometry_changes"] != 0
        or evidence["native_geometry_exports"] != 0
        or result["evidence_cid"] != cid_bytes(canonical_json_bytes(evidence))
        or not replay["exact_replay"]
        or not replay["fresh_process"]
        or replay["optimizer_updates"] != 0
        or replay["result_cid"] != result["result_cid"]
        or replay["process_id"] == result["process_id"]
        or any(
            replay[key] != result[key]
            for key in (
                "preparation_cid",
                "implementation_cid",
                "source_result_cid",
                "model",
                "frames",
                "runtime",
                "evidence_cid",
            )
        )
    ):
        raise ValueError("qualified #1075 source/result/replay relationship differs")
    files = artifact_records(
        _repo(), [row["path"] for row in preparation["implementation"]["files"]]
    )
    if (
        len(files) != SOURCE_FILE_COUNT
        or files != preparation["implementation"]["files"]
        or tree_cid(files) != preparation["implementation"]["tree_cid"]
    ):
        raise ValueError("historical #1075 implementation changed")
    return {
        "source": source,
        "prior": {
            "root": str(prior_root),
            **{f"{name}_cid": SOURCE_CIDS[name] for name in SOURCE_CIDS},
            "evidence_cid": result["evidence_cid"],
            "frame_tree_cid": FRAME_TREE_CID,
            "documents": artifact_records(
                prior_root, [f"{name}.json" for name in SOURCE_CIDS]
            ),
            "public_documents": artifact_records(_repo(), paths.values()),
            "implementation_files": files,
            "native_frame_reads": 0,
        },
    }


def _implementation(lineage: dict) -> dict:
    paths = {
        row["path"]
        for row in lineage["prior"]["implementation_files"]
        + lineage["prior"]["public_documents"]
    }
    paths.update(
        str(path.relative_to(_repo())) for path in Path(__file__).parent.glob("*.py")
    )
    paths.update(
        str(path.relative_to(_repo()))
        for path in (_repo() / "tools/r4-softmax-trainer/tests").glob(
            "test_zoology_language_interface*.py"
        )
    )
    files = artifact_records(_repo(), sorted(paths))
    return {"root": str(_repo()), "files": files, "tree_cid": tree_cid(files)}


def _policies() -> dict:
    from .data import DATA_POLICY

    return {
        "schema": SCHEMA,
        "issue": ISSUE,
        "policy": POLICY,
        "model_config": MODEL_CONFIG,
        "model_policy": MODEL_POLICY,
        "data_policy": DATA_POLICY,
        "training": TRAINING,
        "evaluation": EVALUATION,
        "intervention": INTERVENTION,
    }


def _dataset(root: Path) -> dict:
    from . import data

    manifest = data.validate(root / "data", inspect_development=False)
    records = manifest["files"]
    if (
        artifact_records(root / "data", [row["path"] for row in records]) != records
        or tree_cid(records) != manifest["tree_cid"]
    ):
        raise ValueError("language-interface dataset bytes or tree changed")
    return manifest


def _exclusive(path: Path, body: dict, field: str) -> dict:
    value = {**body, field: cid_bytes(canonical_json_bytes(body))}
    with path.open("xb") as handle:
        handle.write(canonical_json_bytes(value))
    return value


def prepare(root: Path, source_root: Path, prior_root: Path) -> dict:
    from . import data

    root, source_root, prior_root = (
        path.resolve() for path in (root, source_root, prior_root)
    )
    root.mkdir(parents=True, exist_ok=True)
    if (root / "preparation.json").exists() or (root / "data").exists():
        raise FileExistsError("language-interface preparation/data already exists")
    start = _exclusive(
        root / "preparation-started.json",
        {
            "issue": ISSUE,
            "source_root": str(source_root),
            "prior_root": str(prior_root),
        },
        "started_cid",
    )
    lineage = _lineage(source_root, prior_root)
    created = data.prepare(root / "data", source_root)
    dataset = _dataset(root)
    if dataset != created:
        raise ValueError("new dataset changed during preparation")
    body = {
        **_policies(),
        **lineage,
        "dataset": dataset,
        "implementation": _implementation(lineage),
        "preparation_started_cid": start["started_cid"],
    }
    return _exclusive(root / "preparation.json", body, "preparation_cid")


def validate_preparation(root: Path) -> dict:
    root = root.resolve()
    body = previous.prior._envelope(root / "preparation.json", "preparation_cid")
    start = previous.prior._envelope(root / "preparation-started.json", "started_cid")
    if (
        start["issue"] != ISSUE
        or start["source_root"] != body["source"]["root"]
        or start["prior_root"] != body["prior"]["root"]
        or start["started_cid"] != body["preparation_started_cid"]
        or any(body.get(key) != value for key, value in _policies().items())
    ):
        raise ValueError(
            "frozen language-interface policy or preparation phase differs"
        )
    lineage = _lineage(Path(body["source"]["root"]), Path(body["prior"]["root"]))
    current = {
        **lineage,
        "dataset": _dataset(root),
        "implementation": _implementation(lineage),
    }
    if any(body.get(key) != value for key, value in current.items()):
        raise ValueError("language-interface source/data/implementation changed")
    return body
