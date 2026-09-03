"""Identity and resource binding for the sole #1094 comparison.

This module performs no model forwards or population generation. Historical
manifests are consulted by the coordinator only; inference receives a compact
artifact manifest containing no historical answers, roles or populations.
"""

from __future__ import annotations

import hashlib
import json
import os
import platform
import subprocess
from pathlib import Path

from blake3 import blake3

SPEC_COMMIT = "3e894820c520f3b7803a48c6a2eeeb5b7d7021c5"
SPEC_SHA256 = "85f928fec94fa0f6793cff4c35e1fc8c9cba691739d34db272465766c7c9dab1"
POLICY_SHA256 = "91cce30a0b78c48130595369d3ea2a47c4de89cab5db1d4219d1874198cf52d0"
RUNTIME = {
    "python": "3.12.14", "torch": "2.7.1", "device": "cpu", "threads": 4,
    "interop_threads": 1, "workers": 1, "blas": "accelerate",
}
LIMITS = {"phase_seconds": 120, "cumulative_seconds": 360,
          "peak_rss_bytes": 3 * 1024**3, "new_bytes": 128 * 1024**2,
          "batch_size": 128, "logical_row_forwards": 6400}


def canonical(value: object) -> bytes:
    return (json.dumps(value, sort_keys=True, separators=(",", ":"),
                       ensure_ascii=True, allow_nan=False) + "\n").encode()


def digest(payload: bytes) -> str:
    return hashlib.sha256(payload).hexdigest()


def record(path: Path) -> dict:
    payload = path.read_bytes()
    return {"path": str(path.resolve()), "bytes": len(payload),
            "sha256": digest(payload), "cid": "blake3:" + blake3(payload).hexdigest()}


def verify_record(item: dict) -> None:
    actual = record(Path(item["path"]))
    for key in ("bytes", "sha256", "cid"):
        if key in item and item[key] != actual[key]:
            raise ValueError(f"identity mismatch: {item['path']} ({key})")


def exclusive(path: Path, value: dict) -> dict:
    payload = canonical(value)
    with path.open("xb") as stream:
        stream.write(payload)
        stream.flush()
        os.fsync(stream.fileno())
    return record(path)


def source_closure(repo: Path) -> list[dict]:
    package = repo / "tools/r4-softmax-trainer/src/r4_softmax_trainer"
    paths = sorted(package.rglob("*.py")) + [
        package / "text_clause_adapter/policy.json",
        repo / "tools/r4-softmax-trainer/pyproject.toml",
        repo / "tools/r4-softmax-trainer/uv.lock",
    ]
    return [record(path) for path in paths]


def hardware_identity() -> dict:
    """Read the physical CPU/memory identity without timing any workload."""
    keys = ("hw.model", "machdep.cpu.brand_string", "hw.memsize", "hw.ncpu")
    return {key: subprocess.check_output(
        ["/usr/sbin/sysctl", "-n", key], text=True).strip() for key in keys}


def make_bindings(repo: Path) -> dict:
    spec = repo / "docs/integration/clause-segmentation-1085.md"
    policy = repo / "tools/r4-softmax-trainer/src/r4_softmax_trainer/text_clause_adapter/policy.json"
    if digest(spec.read_bytes()) != SPEC_SHA256 or digest(policy.read_bytes()) != POLICY_SHA256:
        raise ValueError("the independently frozen specification/policy changed")
    historical = json.loads((repo / "docs/r4_zoology_language_r4_1079_preparation.json").read_bytes())
    source = historical["source"]
    core = source["core"]
    roots = {"reader": Path(source["root"]), "core": Path(core["root"]),
             "frames": Path(historical["frames"]["root"])}
    paths = {
        "reader": roots["reader"] / source["reader"]["path"],
        "core": roots["core"] / core["model"]["path"],
        "vocabulary": roots["reader"] / "data/vocabulary.json",
        "h4_frames": roots["frames"] / "h4-frames.json",
        "token_frames": roots["frames"] / "token-frames.json",
    }
    assets = {name: record(path) for name, path in paths.items()}
    expected = {"reader": source["reader"]["cid"], "core": core["model"]["cid"],
                "vocabulary": "blake3:571d5fbc282b17c8726eebd7b23c3ae55212a3de81b35d27722a0fa5979b8c5b"}
    expected.update({"h4_frames" if item["path"] == "h4-frames.json" else "token_frames": item["cid"]
                     for item in historical["frames"]["files"]})
    if any(assets[name]["cid"] != cid for name, cid in expected.items()):
        raise ValueError("accepted artifact bytes differ")
    return {
        "schema": "uor-r4.text-clause-model-bindings/1", "issue": 1094,
        "specification_commit": SPEC_COMMIT, "specification_sha256": SPEC_SHA256,
        "policy_sha256": POLICY_SHA256, "assets": assets,
        "reader_state_cid": source["reader"]["state_cid"],
        "core_state_cid": core["model"]["state_cid"],
        "frame_tree_cid": historical["frames"]["tree_cid"],
        "core_model": {"source": {"root": str(roots["core"]), "model": core["model"]}},
        "source_files": source_closure(repo), "runtime": RUNTIME, "limits": LIMITS,
        "machine": platform.machine(), "platform": platform.platform(),
        "hardware": hardware_identity(),
        "source_commit": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=repo, text=True).strip(),
    }


def sandbox_profile(repo: Path, python: Path, bindings: Path, assets: dict) -> str:
    """Deny home reads except runtime, package code and exact model identities.

    The coordinator supplies raw requests through stdin. In particular, corpus,
    reference and historical report files are not accessible to either worker.
    The OS profile is an execution-isolation control, not a security claim about
    arbitrary hostile native code.
    """
    home = str(Path.home())
    allowed_trees = [str(repo / "tools/r4-softmax-trainer/src"),
                     str(python.parent.parent), str(python.resolve().parent.parent)]
    exact = [str(bindings.resolve()), str(repo / "tools/r4-softmax-trainer/pyproject.toml"),
             str(repo / "tools/r4-softmax-trainer/uv.lock")] + [item["path"] for item in assets.values()]
    def quoted(value: str) -> str:
        return json.dumps(value)
    exclusions = "\n".join(
        [f"  (require-not (subpath {quoted(path)}))" for path in allowed_trees]
        + [f"  (require-not (literal {quoted(path)}))" for path in exact])
    return ("(version 1)\n(allow default)\n(deny network*)\n"
            f"(deny file-write* (subpath {quoted(home)}))\n"
            f"(deny file-read* (require-all (subpath {quoted(home)})\n{exclusions}\n))\n")
