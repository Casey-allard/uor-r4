"""Immutable source, native frame, and implementation bindings for #1059.

Preparation reads public source envelopes and hashes retained artifacts. It
does not construct a model, load training tensors, or score fitted weights.
"""

from __future__ import annotations

import ast
import json
import tomllib
from pathlib import Path
from typing import Any

from ..provenance import artifact_records, canonical_json_bytes, cid_bytes, tree_cid

ISSUE = 1059
POLICY = "ZoologyCoherentR4InferenceV1"
SOURCE_RESULT_CID = (
    "blake3:bd16d012c01262ffb8c5197e4cf316c6fee1d722cf0700a0048386180a8122e0"
)
SOURCE_PREPARATION_CID = (
    "blake3:bdb9dd01ea0e115eaff54c0536833f2d13cae3d32c9c60cf90070225f031a335"
)
SOURCE_MODEL_CID = (
    "blake3:163cf3e5375b3e721fa7a826acdb2dfc809e5989209b03fb2a3eea3e3d5459e9"
)
SOURCE_STATE_CID = (
    "blake3:600bdc76cefff79f4be8709197b15252cb531892fad0db2156b36b865c01877e"
)
SOURCE_DATA_CID = (
    "blake3:f6dd39f9e0554df7409ee051e353798b89de8047d9f3ce32b983fa83623754b8"
)
SOURCE_ARM = "arms/00-4p6415888336127773e-4/result.json"
EVALUATION = {
    "rows": 3000,
    "queries_per_row": 4,
    "batch_size": 512,
    "threads": 4,
    "interop_threads": 1,
    "max_elapsed_seconds": 900,
    "max_rss_bytes": 4 * 1024**3,
    "logit_atol": 0.005,
    "attention_atol": 1e-5,
    "nll_atol": 1e-5,
    "historical_correct": 11900,
    "strong_control_drop": 0.50,
    "row_order": "canonical_file_order_0_to_2999",
    "control": "transport_source_position_(s+1)_mod_(q+1)_encode_true_source",
    "optimizer_updates": 0,
}
PREPARATION_NAME = "preparation.json"


def _envelope(path: Path, key: str, expected: str | None = None) -> dict[str, Any]:
    value = json.loads(path.read_text())
    unsigned = dict(value)
    observed = unsigned.pop(key, None)
    if observed != cid_bytes(canonical_json_bytes(unsigned)):
        raise ValueError(f"{path.name} self-CID differs")
    if expected is not None and observed != expected:
        raise ValueError(f"{path.name} differs from the frozen predecessor")
    return value


def _path(root: Path, relative: str) -> Path:
    candidate = root.joinpath(relative).resolve()
    if not candidate.is_relative_to(root.resolve()):
        raise ValueError("artifact path escapes its bound root")
    return candidate


def _record(root: Path, relative: str, *, cid: str | None = None) -> dict[str, Any]:
    _path(root, relative)
    record = artifact_records(root, [relative])[0]
    if cid is not None and record["cid"] != cid:
        raise ValueError(f"{relative} artifact CID differs")
    return record


def _python_closure(trainer: Path) -> set[Path]:
    """Follow static local imports, including package initializers."""
    source = trainer / "src"
    pending = list(
        (source / "r4_softmax_trainer" / "zoology_r4_inference").glob("*.py")
    )
    seen: set[Path] = set()
    while pending:
        path = pending.pop()
        if path in seen:
            continue
        if path.is_symlink():
            raise ValueError("implementation source must not be a symlink")
        seen.add(path)
        parts = list(path.relative_to(source).with_suffix("").parts)
        package = parts[:-1]
        for depth in range(1, len(package) + 1):
            initializer = source.joinpath(*package[:depth], "__init__.py")
            if initializer.is_file():
                pending.append(initializer)
        modules: set[str] = set()
        for node in ast.walk(ast.parse(path.read_text(), filename=str(path))):
            if isinstance(node, ast.Import):
                modules.update(alias.name for alias in node.names)
            elif isinstance(node, ast.ImportFrom):
                if node.level:
                    if node.level > len(package):
                        raise ValueError("relative import escapes package")
                    prefix = package[: len(package) - node.level + 1]
                    if node.module:
                        prefix += node.module.split(".")
                    module = ".".join(prefix)
                else:
                    module = node.module or ""
                modules.add(module)
                modules.update(f"{module}.{alias.name}" for alias in node.names)
        for module in modules:
            if not (
                module == "r4_softmax_trainer"
                or module.startswith("r4_softmax_trainer.")
            ):
                continue
            stem = source.joinpath(*module.split("."))
            for imported in (stem.with_suffix(".py"), stem / "__init__.py"):
                if imported.is_file():
                    pending.append(imported)
                    break
    return seen


def _native_sources(repo: Path) -> set[Path]:
    """Bind native package sources and recursively local Cargo dependencies."""
    pending = [repo / "crates" / "uor-r4-core"]
    workspace = tomllib.loads((repo / "Cargo.toml").read_text())
    for replacements in workspace.get("patch", {}).values():
        for entry in replacements.values():
            if isinstance(entry, dict) and "path" in entry:
                pending.append(repo / entry["path"])
    seen: set[Path] = set()
    paths = {repo / "Cargo.toml", repo / "Cargo.lock", repo / "rust-toolchain.toml"}
    paths.update((repo / ".cargo").glob("*.toml"))
    while pending:
        package = pending.pop().resolve()
        if package in seen:
            continue
        if not package.is_relative_to(repo):
            raise ValueError("native package escaped repository")
        seen.add(package)
        manifest = package / "Cargo.toml"
        paths.add(manifest)
        paths.update((package / "src").rglob("*.rs"))
        if (package / "build.rs").is_file():
            paths.add(package / "build.rs")
        data = tomllib.loads(manifest.read_text())
        sections = [data]
        sections.extend(data.get("target", {}).values())
        for section in sections:
            for kind in ("dependencies", "build-dependencies"):
                for entry in section.get(kind, {}).values():
                    if isinstance(entry, dict) and "path" in entry:
                        pending.append(package / entry["path"])
    # Include text compiled into these native modules without hashing raw corpora.
    for relative in (
        "docs/hologram_r4_formal_monograph.md",
        "docs/transformerless/INFERENCE_OPERATION_CONTRACT.md",
    ):
        candidate = repo / relative
        if candidate.is_file():
            paths.add(candidate)
    return paths


def implementation_contract() -> dict[str, Any]:
    trainer = Path(__file__).resolve().parents[3]
    repo = trainer.parents[1]
    paths = _python_closure(trainer) | _native_sources(repo)
    paths.update((trainer / "tests").glob("test_zoology_r4*.py"))
    paths.update(trainer / name for name in ("pyproject.toml", "uv.lock"))
    paths.update(
        trainer / "src" / "r4_softmax_trainer" / "zoology_control" / name
        for name in ("NOTICE.md", "LICENSE-APACHE-2.0.md")
    )
    records = artifact_records(repo, [str(path.relative_to(repo)) for path in paths])
    return {"root": str(repo), "files": records, "tree_cid": tree_cid(records)}


def _source_contract(source: Path) -> dict[str, Any]:
    trainer = Path(__file__).resolve().parents[3]
    preparation = _envelope(
        source / "zoology-release-preparation.json",
        "preparation_cid",
        SOURCE_PREPARATION_CID,
    )
    result = _envelope(
        source / "run/zoology-release-result.json", "result_cid", SOURCE_RESULT_CID
    )
    arm = _envelope(source / SOURCE_ARM, "arm_cid")
    if arm["status"] != "SOURCE_REPRODUCTION_POSITIVE" or not arm["passed"]:
        raise ValueError("source arm is not the qualified reference")
    if len(result["arms"]) != 1 or result["arms"][0]["arm_cid"] != arm["arm_cid"]:
        raise ValueError("source result and arm differ")
    if result["preparation_cid"] != preparation["preparation_cid"]:
        raise ValueError("source preparation/result link differs")
    old_records = preparation["implementation"]["files"]
    current_records = artifact_records(
        trainer, [entry["path"] for entry in old_records]
    )
    if current_records != old_records:
        raise ValueError("historical #1050 implementation has changed")
    artifact = arm["artifact"]
    model = _record(source, artifact["path"], cid=SOURCE_MODEL_CID)
    if model["bytes"] != artifact["bytes"] or artifact["state_cid"] != SOURCE_STATE_CID:
        raise ValueError("source model state/size differs")
    model.update(state_cid=SOURCE_STATE_CID, config=artifact["config"])
    dataset = _record(source, preparation["dataset"]["path"], cid=SOURCE_DATA_CID)
    if dataset["bytes"] != preparation["dataset"]["bytes"]:
        raise ValueError("source data size differs")
    documents = artifact_records(
        source,
        [
            "zoology-release-preparation.json",
            "run/zoology-release-result.json",
            SOURCE_ARM,
        ],
    )
    return {
        "root": str(source),
        "result_cid": result["result_cid"],
        "arm_cid": arm["arm_cid"],
        "preparation_cid": preparation["preparation_cid"],
        "model": model,
        "dataset": dataset,
        "documents": documents,
        "historical_test": arm["final_test"],
        "checkpoint_reads": 0,
    }


def _frame_contract(frames_root: Path) -> dict[str, Any]:
    from .frames import load_frames

    frames = load_frames(frames_root)
    records = artifact_records(frames_root, ["h4-frames.json", "token-frames.json"])
    return {
        "root": str(frames_root),
        "files": records,
        "tree_cid": tree_cid(records),
        "vocabulary_size": frames.token_leaf_indices.numel(),
        "frame_artifact_cid": frames.frame_artifact_cid,
        "token_map_artifact_cid": frames.artifact_cid,
        "validated_native_prefix_witnesses": True,
    }


def prepare(root: Path, source_root: Path, frames_root: Path) -> dict[str, Any]:
    root, source_root, frames_root = (
        path.resolve() for path in (root, source_root, frames_root)
    )
    root.mkdir(parents=True, exist_ok=True)
    path = root / PREPARATION_NAME
    if path.exists():
        raise FileExistsError("inference preparation already exists; do not overwrite")
    body = {
        "schema": "uor-r4.zoology-r4-inference-preparation/1",
        "issue": ISSUE,
        "policy": POLICY,
        "source": _source_contract(source_root),
        "frames": _frame_contract(frames_root),
        "implementation": implementation_contract(),
        "evaluation": dict(EVALUATION),
        "preserved_1057": {
            "artifact_cid": "blake3:69af5586eccfceab4214e9f13524eeea578eb3facaea4fdedec89f0b5d217445",
            "checkpoint_cid": "blake3:fd24e6b84af9891c1dad1eb2a13c6d86b8e8833ac02ab40f0e4e95d4530d140a",
            "inference_or_training_reads": 0,
        },
    }
    body["preparation_cid"] = cid_bytes(canonical_json_bytes(body))
    with path.open("xb") as output:
        output.write(canonical_json_bytes(body))
    return body


def validate_preparation(root: Path) -> dict[str, Any]:
    preparation = _envelope(root / PREPARATION_NAME, "preparation_cid")
    if (
        preparation.get("issue") != ISSUE
        or preparation.get("policy") != POLICY
        or preparation.get("evaluation") != EVALUATION
    ):
        raise ValueError("inference policy differs from its frozen contract")
    if _source_contract(Path(preparation["source"]["root"])) != preparation["source"]:
        raise ValueError("source artifacts changed after preparation")
    if _frame_contract(Path(preparation["frames"]["root"])) != preparation["frames"]:
        raise ValueError("native frames changed after preparation")
    if implementation_contract() != preparation["implementation"]:
        raise ValueError("implementation changed after preparation")
    return preparation
