"""Freeze one learned compound-key attention prototype and its recorded reference."""

from __future__ import annotations

import copy
from pathlib import Path
from typing import Any

from ..provenance import artifact_records, canonical_json_bytes, cid_bytes, tree_cid
from ..zoology_cyclic_facts import contract as previous
from ..zoology_english_diagnostic.contract import _envelope
from . import data
from .model import MODEL_CONFIG, MODEL_POLICY

ISSUE = 1073
POLICY = "ZoologyLearnedCompoundFactBindingV1"
TRAINING = copy.deepcopy(previous.TRAINING)
EVALUATION = {
    **copy.deepcopy(previous.EVALUATION),
    "construction_groups": 2048,
    "construction_unknown_rows": 2048,
    "construction_unknown_min_correct": 1946,
    "construction_per_question_type_groups_min_correct": 973,
}
BEHAVIOR = {
    **copy.deepcopy(previous.BEHAVIOR),
    "require_behavior_for_development": False,
    "decision": "partial progress against the recorded #1067 matching rotations only; full construction, order and value-control qualification govern development",
}
ORDER = {
    "rotations": [0, 1, 2, 3],
    "construction_rows": 10240,
    "top1_exact": True,
    "max_absolute_logit_difference": 1e-4,
    "scope": "same world's four cyclic fact orders; compare all full-head logits with canonical order and require identical predictions",
}
CONTROL = {
    "rotations": [0, 1, 2, 3],
    "value_cycle_shift": 1,
    "null_fixed": True,
    "original_known_correct_drop_min": 4096,
    "replacement_known_correct_min": 7783,
    "unknown_min_correct": 1946,
    "attention_exact": True,
    "target_rule": "at queried fact index j, right-cycle V supplies original location at (j - 1) mod 4; evaluator-only replacement targets",
    "scope": "cycle four projected fact values only; hold Q, K and null fixed; compare attention exactly within each order",
    "model_label_arguments": 0,
}
SOURCE_CIDS = {
    "preparation": "blake3:92ffef681fe6bd4cfc6532bd811496b6e9c4d052b978c172d3a5ca9bcef8ef05",
    "fit": "blake3:7ec8c4208338940b5d06fb91a6f54e67f47445097f982714a8980bcf8c372e94",
    "result": "blake3:b5ab27771843d347d9188d4541b46c34bc4ab1d860d956387b8726a100f513be",
    "replay": "blake3:08d671be8d4e7db36860493d59c85e0ddec2bfab2259b94c628c6d23436685de",
}
SOURCE_FILE_COUNT = 254
INTERVENTION = {
    "reference_issue": 1067,
    "reference_evidence_issue": 1071,
    "reference_model_loads": 0,
    "supervised_position": 37,
    "input_sequence_length": 41,
    "initialization": "fresh seed 123; no prior learned state",
    "training_order": "canonical construction only; no cyclic training augmentation",
    "learning_dose": "same 3920 updates and 2007040 presentations; changed architecture and RNG consumption may change actual supported/unknown partial-tail counts",
    "architecture_change": "shared lexical embedding; independent learned compound Q/K; location V; four facts plus learned null; ordinary softmax and full tied vocabulary head",
    "interface": "fixed-grammar causal lexical-field extraction with explicit answer supervision; no oracle match, absence flag or answer supplied to model",
    "comparison_scope": "new structured architecture at the same dose; not an isolated attention, geometry or language-generalization comparison",
    "dependence": "four cyclic orders and related question pairs are correlated views of the same world",
    "development_admission": "all four construction qualification gates, full-head order criterion and all four value-binding controls must pass",
    "partial_positive": "retain measured structured-binding gains without claiming qualification or opening development",
    "partial_negative": "retain #1067 recorded reference and revise the binding learning recipe",
    "control_miss": "construction fits but value-binding qualification fails; development remains unscored",
    "development_miss": "construction and value binding qualify; fresh-combination transfer remains unresolved",
    "language_pass": "next separate unchanged-R4 inference preservation; no R4 execution in this fit",
}


def _repo() -> Path:
    return Path(__file__).resolve().parents[5]


def _lineage() -> dict[str, Any]:
    repo = _repo()
    historical = previous._lineage()
    documents, paths = {}, []
    for name, expected in SOURCE_CIDS.items():
        relative = f"docs/r4_zoology_cyclic_facts_1071_{name}.json"
        paths.append(relative)
        documents[name] = _envelope(repo / relative, f"{name}_cid", expected)
    preparation, fitted = documents["preparation"], documents["fit"]
    result, replay = documents["result"], documents["replay"]
    evidence = result["evidence"]
    if (
        preparation["training"] != TRAINING
        or preparation["model_config"] != previous.MODEL_CONFIG
        or preparation["evaluation"] != previous.EVALUATION
        or preparation["lineage"]["baseline"] != historical["baseline"]
        or fitted["status"] != "FIT_COMPLETE"
        or fitted["completed_updates"] != TRAINING["total_updates"]
        or fitted["work"] != historical["baseline"]["fit_work"]
        or fitted["preparation_cid"] != preparation["preparation_cid"]
        or result["fit_cid"] != fitted["fit_cid"]
        or result["artifact"] != fitted["artifact"]
        or result["preparation_cid"] != preparation["preparation_cid"]
        or evidence["status"] != "CYCLIC_FACTS_PRESERVATION_MISS"
        or not evidence["reference_canonical_exact_reproduction"]
        or result["evidence_cid"] != cid_bytes(canonical_json_bytes(evidence))
        or not replay["exact_replay"]
        or not replay["fresh_process"]
        or replay["result_cid"] != result["result_cid"]
        or replay["evidence_cid"] != result["evidence_cid"]
    ):
        raise ValueError(
            "cyclic-facts lineage, recorded reference or learning dose differs"
        )
    implementation = preparation["implementation"]
    files = artifact_records(repo, [row["path"] for row in implementation["files"]])
    if (
        len(files) != SOURCE_FILE_COUNT
        or files != implementation["files"]
        or tree_cid(files) != implementation["tree_cid"]
    ):
        raise ValueError("historical source implementation changed")
    views = evidence["reference_views"]
    if (
        [row["rotation"] for row in views] != [0, 1, 2, 3]
        or any(row["construction"]["decisions"] != 8192 for row in views)
        or views[0]["construction"] != historical["baseline"]["construction"]
        or views[0]["diagnostic"] != historical["baseline"]["diagnostic"]
    ):
        raise ValueError(
            "recorded four-order reference or canonical reproduction differs"
        )
    return {
        "cyclic_facts_cids": dict(SOURCE_CIDS),
        "readout_cids": historical["readout_cids"],
        "documents": artifact_records(repo, paths),
        "implementation_files": files,
        "construction_files": historical["construction_files"],
        "reference_issue": 1067,
        "reference_evidence_issue": 1071,
        "baseline": historical["baseline"],
        "reference_views": views,
        "reference_worlds": evidence["all_order_worlds"]["reference"],
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
            "test_zoology_compound_binding*.py"
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
        raise FileExistsError("compound-binding preparation already exists")
    lineage = _lineage()
    dataset = data.build(root / "data", source_root / "data")
    # Preparation audits fresh labels/exclusions. Routine validation opens neither
    # development payload nor historical weights; comparisons use bound evidence.
    body = {
        "schema": "uor-r4.zoology-compound-binding-preparation/1",
        "issue": ISSUE,
        "policy": POLICY,
        "model_config": MODEL_CONFIG,
        "model_policy": MODEL_POLICY,
        "training": TRAINING,
        "evaluation": EVALUATION,
        "behavior": BEHAVIOR,
        "order": ORDER,
        "control": CONTROL,
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
        ("schema", "uor-r4.zoology-compound-binding-preparation/1"),
        ("issue", ISSUE),
        ("policy", POLICY),
        ("model_config", MODEL_CONFIG),
        ("model_policy", MODEL_POLICY),
        ("training", TRAINING),
        ("evaluation", EVALUATION),
        ("behavior", BEHAVIOR),
        ("order", ORDER),
        ("control", CONTROL),
        ("intervention", INTERVENTION),
    ):
        if body.get(key) != expected:
            raise ValueError(f"frozen compound-binding {key} differs")
    current = _bindings(root)
    if any(body.get(key) != value for key, value in current.items()):
        raise ValueError("compound-binding source, data or implementation changed")
    return body
