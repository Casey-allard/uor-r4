"""Direct #1063 bindings and construction-only access for diagnostic #1065."""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any

from safetensors.torch import load as load_safetensors
from torch import Tensor

from ..provenance import artifact_records, canonical_json_bytes, cid_bytes, tree_cid
from ..zoology_english_binding import data

ISSUE = 1065
POLICY = "ZoologyEnglishConstructionDiagnosticV1"
EVALUATION = {
    "batch_size": 256,
    "threads": 8,
    "interop_threads": 1,
    "rows": 8192,
    "max_elapsed_seconds": 300.0,
    "max_rss_bytes": 2 * 1024**3,
}
ANALYSIS_RULES = {
    "scope": "descriptive construction-only localization; no causal or capacity verdict",
    "categories": [
        "target",
        "same_owner_confound",
        "same_object_confound",
        "unrelated_fact_location",
        "out_of_history_location",
        "unknown",
        "other_vocabulary",
    ],
    "classification": "parse each row's actual facts and query; q1 need not have both confounds",
    "strict_majority": "2 * count > denominator, with denominator > 0",
    "empty_denominator": "rate and flag unavailable (null); never majority evidence",
    "slot_denominator": "all in-history predictions",
    "attribute_denominator": "q0 in-history wrong predictions only",
    "availability": "report available fact slots and rows with an available category",
    "question_invariance_denominator": "4096 question pairs: variants 0/1 and 2/3",
    "decision_order": [
        "slot majority and either q0 attribute majority => JOINT_POSITION_ATTRIBUTE",
        "slot majority => POSITION_READOUT",
        "q0 same-owner majority => OBJECT_DISAMBIGUATION",
        "q0 same-object majority => OWNER_DISAMBIGUATION",
        "question invariance majority => QUESTION_READOUT",
        "otherwise => DISTRIBUTED_BINDING",
    ],
    "paired_contrast": "float64: (z_left[a]-z_left[b])-(z_right[a]-z_right[b]); a=left target, b=right target; report positive/zero/negative",
    "pairs": "questions 0/1 and 2/3; location swaps 0/2 and 1/3, within each canonical group",
    "target_margin": "own-target logit minus maximum over all 4095 other vocabulary IDs",
    "logit_difference": "float64 full-vocabulary absolute-difference summaries; no candidate filtering",
}
SOURCE_CIDS = {
    "preparation": "blake3:c926e16516ef0f1d8242dc0af39a04be46cb082bb6c76590bc73f2717e027ca8",
    "fit": "blake3:7c857e5b8a1506cdab8db7d858428cb78639e10fb419b51396192d3e8aa90a79",
    "result": "blake3:aaca100c5c2b8abfb126937523c5cce44bb7e6ca2eb8d48260f42e9281606e0f",
    "replay": "blake3:dd5984c22d507faa1e2cea0f9b0c8051fbd3ec923cf53c896768e62708295e02",
}
SOURCE_PATHS = {
    "preparation": "preparation.json",
    "fit": "fit/fit.json",
    "result": "result.json",
    "replay": "replay.json",
}
SOURCE_FILE_COUNT = 199
PUBLIC_PREFIX = "docs/r4_zoology_english_binding_1063_"
PACKAGE = "tools/r4-softmax-trainer/src/r4_softmax_trainer/zoology_english_diagnostic"


def _repo() -> Path:
    return Path(__file__).resolve().parents[5]


def _path(root: Path, relative: str) -> Path:
    name = Path(relative)
    path = root / name
    if name.is_absolute() or not path.resolve().is_relative_to(root.resolve()):
        raise ValueError("bound file escapes its root")
    return path


def _envelope(path: Path, field: str, expected: str | None = None) -> dict[str, Any]:
    payload = path.read_bytes()
    value = json.loads(payload)
    if not isinstance(value, dict) or payload != canonical_json_bytes(value):
        raise ValueError("envelope is not canonical JSON")
    body = dict(value)
    observed = body.pop(field, None)
    if observed != cid_bytes(canonical_json_bytes(body)) or (
        expected is not None and observed != expected
    ):
        raise ValueError("envelope differs from its bound identity")
    return value


def _record(root: Path, relative: str) -> dict[str, Any]:
    _path(root, relative)
    return artifact_records(root, [relative])[0]


def _read_record(root: Path, record: dict[str, Any], relative: str) -> bytes:
    if record["path"] != relative:
        raise ValueError("bound file path differs")
    payload = _path(root, relative).read_bytes()
    if len(payload) != record["bytes"] or cid_bytes(payload) != record["cid"]:
        raise ValueError("bound file identity differs")
    return payload


def _source(root: Path) -> tuple[dict[str, Any], list[dict[str, Any]]]:
    repo = _repo()
    documents = {}
    public_paths = []
    for name, relative in SOURCE_PATHS.items():
        public = f"{PUBLIC_PREFIX}{name}.json"
        public_paths.append(public)
        published = _envelope(repo / public, f"{name}_cid", SOURCE_CIDS[name])
        local = _envelope(_path(root, relative), f"{name}_cid", SOURCE_CIDS[name])
        if local != published:
            raise ValueError("local source differs from the public evidence")
        documents[name] = local
    preparation, fitted, result, replay = (
        documents[name] for name in ("preparation", "fit", "result", "replay")
    )
    if (
        any(
            doc["preparation_cid"] != preparation["preparation_cid"]
            for doc in (fitted, result, replay)
        )
        or any(doc["fit_cid"] != fitted["fit_cid"] for doc in (result, replay))
        or replay["result_cid"] != result["result_cid"]
        or not replay["exact_replay"]
        or not replay["fresh_process"]
        or result["evidence_cid"] != replay["evidence_cid"]
        or result["evidence_cid"] != cid_bytes(canonical_json_bytes(result["evidence"]))
        or fitted["status"] != "FIT_COMPLETE"
        or fitted["completed_updates"] != 3920
        or fitted["training"] != preparation["training"]
        or fitted["artifact"]["config"] != preparation["model_config"]
        or any(doc["artifact"] != fitted["artifact"] for doc in (result, replay))
    ):
        raise ValueError("source fit/result/replay relationship differs")

    files = preparation["implementation"]["files"]
    for row in files:
        _path(repo, row["path"])
    if (
        len(files) != SOURCE_FILE_COUNT
        or len({row["path"] for row in files}) != SOURCE_FILE_COUNT
        or artifact_records(repo, [row["path"] for row in files]) != files
        or tree_cid(files) != preparation["implementation"]["tree_cid"]
        or any(doc["implementation_cid"] != tree_cid(files) for doc in (result, replay))
    ):
        raise ValueError("historical source implementation differs")
    model = fitted["artifact"]
    _read_record(root, model, "fit/model.safetensors")
    data_root = root / "data"
    manifest = _envelope(
        data_root / "manifest.json",
        "manifest_cid",
        preparation["dataset"]["manifest_cid"],
    )
    if manifest != preparation["dataset"] or manifest["policy"] != data.DATA_POLICY:
        raise ValueError("source data manifest differs")
    inventory = manifest["files"]
    if [row["path"] for row in inventory] != [
        "construction.safetensors",
        "development.safetensors",
        "vocabulary.json",
    ] or tree_cid(inventory) != manifest["tree_cid"]:
        raise ValueError("source data inventory differs")
    _read_record(data_root, inventory[0], "construction.safetensors")
    vocabulary = _read_record(data_root, inventory[2], "vocabulary.json")
    if vocabulary != data._vocabulary_bytes():
        raise ValueError("source lexical encoding differs")
    expected = result["evidence"]["language"]["construction"]
    if expected["decisions"] != EVALUATION["rows"] or expected["batches"] != 32:
        raise ValueError("source construction score population differs")
    return {
        "root": str(root),
        "cids": dict(SOURCE_CIDS),
        "model": model,
        "expected_construction": expected,
        "documents": artifact_records(root, list(SOURCE_PATHS.values())),
        "dataset": {
            "root": str(data_root),
            "manifest": _record(data_root, "manifest.json"),
            "manifest_cid": manifest["manifest_cid"],
            "construction": inventory[0],
            "vocabulary": inventory[2],
        },
        "implementation_files": files,
        "implementation_tree_cid": tree_cid(files),
        "access": {
            "development_payload_reads": 0,
            "optimizer_checkpoint_reads": 0,
            "native_frame_artifact_reads": 0,
        },
    }, artifact_records(repo, public_paths)


def _bindings(source_root: Path) -> dict[str, Any]:
    source, public = _source(source_root)
    repo = _repo()
    paths = {row["path"] for row in source["implementation_files"] + public}
    paths.update(str(p.relative_to(repo)) for p in (repo / PACKAGE).glob("*.py"))
    paths.update(
        str(p.relative_to(repo))
        for p in (repo / "tools/r4-softmax-trainer/tests").glob(
            "test_zoology_english_diagnostic*.py"
        )
    )
    files = artifact_records(repo, sorted(paths))
    return {
        "source": source,
        "implementation": {
            "root": str(repo),
            "files": files,
            "tree_cid": tree_cid(files),
        },
    }


def prepare(root: Path, source_root: Path) -> dict[str, Any]:
    root, source_root = root.resolve(), source_root.resolve()
    root.mkdir(parents=True, exist_ok=True)
    path = root / "preparation.json"
    if path.exists():
        raise FileExistsError("diagnostic preparation already exists")
    body = {
        "schema": "uor-r4.zoology-english-diagnostic-preparation/1",
        "issue": ISSUE,
        "policy": POLICY,
        "evaluation": dict(EVALUATION),
        "analysis_rules": ANALYSIS_RULES,
        **_bindings(source_root),
    }
    body["preparation_cid"] = cid_bytes(canonical_json_bytes(body))
    with path.open("xb") as output:
        output.write(canonical_json_bytes(body))
    return body


def validate_preparation(root: Path) -> dict[str, Any]:
    body = _envelope(root.resolve() / "preparation.json", "preparation_cid")
    for key, expected in (
        ("schema", "uor-r4.zoology-english-diagnostic-preparation/1"),
        ("issue", ISSUE),
        ("policy", POLICY),
        ("evaluation", EVALUATION),
        ("analysis_rules", ANALYSIS_RULES),
    ):
        if body.get(key) != expected:
            raise ValueError("diagnostic policy differs")
    source_root = Path(body["source"]["root"])
    if not source_root.is_absolute():
        raise ValueError("diagnostic source root must be absolute")
    current = _bindings(source_root.resolve())
    if any(body.get(key) != value for key, value in current.items()):
        raise ValueError("diagnostic source or implementation binding changed")
    return body


def load_construction(preparation: dict[str, Any]) -> dict[str, Tensor]:
    """Load the original supported rows; no model, checkpoint, frames or development."""
    dataset = preparation["source"]["dataset"]
    root = Path(dataset["root"])
    manifest_bytes = _read_record(root, dataset["manifest"], "manifest.json")
    manifest = json.loads(manifest_bytes)
    if manifest["manifest_cid"] != dataset["manifest_cid"]:
        raise ValueError("construction manifest identity differs")
    if (
        _read_record(root, dataset["vocabulary"], "vocabulary.json")
        != data._vocabulary_bytes()
    ):
        raise ValueError("construction lexical encoding differs")
    tensors = load_safetensors(
        _read_record(root, dataset["construction"], "construction.safetensors")
    )
    data._check_shapes(tensors, development=False)
    selected = tensors["variant_ids"] < 4
    supported = {key: value[selected].contiguous() for key, value in tensors.items()}
    if supported["inputs"].shape != (EVALUATION["rows"], data.SEQUENCE_LENGTH):
        raise ValueError("supported construction count differs")
    parsed = []
    for ids, target in zip(
        supported["inputs"].tolist(),
        supported["targets"].flatten().tolist(),
        strict=True,
    ):
        facts, query, answer = data.parse_row(ids)
        if answer == "unknown" or data.TOKEN_IDS[answer] != target:
            raise ValueError("construction label differs from parsed input")
        parsed.append((facts, query))
    for group in range(EVALUATION["rows"] // 4):
        (facts, q0), (same, q1), (swapped, sq0), (same_swapped, sq1) = parsed[
            group * 4 : group * 4 + 4
        ]
        same_owner = group % 2 == 0
        if (
            facts != same
            or swapped != same_swapped
            or q0 != sq0
            or q1 != sq1
            or (same_owner and (q0[0] != q1[0] or q0[1] == q1[1]))
            or (not same_owner and (q0[1] != q1[1] or q0[0] == q1[0]))
        ):
            raise ValueError("construction question-pair metadata differs")
        first = next(index for index, fact in enumerate(facts) if fact[:2] == q0)
        second = next(index for index, fact in enumerate(facts) if fact[:2] == q1)
        expected = list(facts)
        expected[first] = (*facts[first][:2], facts[second][2])
        expected[second] = (*facts[second][:2], facts[first][2])
        if tuple(expected) != swapped:
            raise ValueError("construction location-swap pairing differs")
    return supported
