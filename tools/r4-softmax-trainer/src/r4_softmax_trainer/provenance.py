"""Canonical JSON and BLAKE3 content-addressed evidence helpers."""

from __future__ import annotations

import json
import os
from pathlib import Path
from typing import Any, Iterable

from blake3 import blake3


def canonical_json_bytes(value: Any) -> bytes:
    """Encode JSON with one stable byte representation."""
    return (
        json.dumps(value, ensure_ascii=False, allow_nan=False, sort_keys=True, separators=(",", ":"))
        + "\n"
    ).encode("utf-8")


def cid_bytes(value: bytes) -> str:
    return f"blake3:{blake3(value).hexdigest()}"


def cid_file(path: Path, *, chunk_size: int = 8 * 1024 * 1024) -> str:
    digest = blake3()
    with path.open("rb") as source:
        while chunk := source.read(chunk_size):
            digest.update(chunk)
    return f"blake3:{digest.hexdigest()}"


def atomic_write(path: Path, value: bytes) -> None:
    """Write one file atomically without leaving a successful partial result."""
    path.parent.mkdir(parents=True, exist_ok=True)
    temporary = path.with_name(f".{path.name}.tmp")
    with temporary.open("wb") as target:
        target.write(value)
        target.flush()
        os.fsync(target.fileno())
    os.replace(temporary, path)


def atomic_write_json(path: Path, value: Any) -> None:
    atomic_write(path, canonical_json_bytes(value))


def artifact_records(root: Path, relative_paths: Iterable[str]) -> list[dict[str, Any]]:
    records: list[dict[str, Any]] = []
    for relative in sorted(set(relative_paths)):
        candidate = (root / relative).resolve()
        try:
            candidate.relative_to(root.resolve())
        except ValueError as error:
            raise ValueError(f"artifact escapes root: {relative}") from error
        if not candidate.is_file():
            raise FileNotFoundError(candidate)
        records.append(
            {
                "bytes": candidate.stat().st_size,
                "cid": cid_file(candidate),
                "path": relative,
            }
        )
    return records


def tree_cid(records: list[dict[str, Any]]) -> str:
    """Bind sorted path, byte length, and file CID records."""
    canonical = sorted(records, key=lambda record: str(record["path"]))
    return cid_bytes(canonical_json_bytes(canonical))


def trainer_implementation_contract() -> dict[str, Any]:
    """Bind every executable trainer module plus its locked dependency inputs."""
    root = Path(__file__).resolve().parents[2]
    relative_paths = ["pyproject.toml", "uv.lock"]
    relative_paths.extend(
        str(path.relative_to(root))
        for path in sorted((root / "src" / "r4_softmax_trainer").glob("*.py"))
    )
    records = artifact_records(root, relative_paths)
    return {"files": records, "tree_cid": tree_cid(records)}


def write_bound_manifest(
    manifest_path: Path,
    payload: dict[str, Any],
    *,
    artifact_root: Path,
    relative_paths: Iterable[str],
) -> dict[str, Any]:
    """Write a non-self-referential manifest bound to an exact artifact tree."""
    records = artifact_records(artifact_root, relative_paths)
    body = dict(payload)
    body["artifacts"] = records
    body["tree_cid"] = tree_cid(records)
    body["manifest_cid"] = cid_bytes(canonical_json_bytes(body))
    atomic_write_json(manifest_path, body)
    return body


def verify_bound_manifest(manifest_path: Path, *, artifact_root: Path) -> dict[str, Any]:
    """Fail closed if a manifest or any artifact no longer reproduces its CID."""
    manifest = verify_manifest_envelope(manifest_path)
    records = manifest["artifacts"]
    reproduced = artifact_records(artifact_root, [str(record["path"]) for record in records])
    if reproduced != records:
        raise ValueError("artifact records do not reproduce")
    return manifest


def verify_manifest_envelope(manifest_path: Path) -> dict[str, Any]:
    """Verify manifest/tree commitments without opening any artifact file."""
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    expected_manifest_cid = manifest.get("manifest_cid")
    unsigned = dict(manifest)
    unsigned.pop("manifest_cid", None)
    actual_manifest_cid = cid_bytes(canonical_json_bytes(unsigned))
    if expected_manifest_cid != actual_manifest_cid:
        raise ValueError(
            f"manifest CID mismatch: expected {expected_manifest_cid}, got {actual_manifest_cid}"
        )
    records = manifest.get("artifacts")
    if not isinstance(records, list):
        raise ValueError("manifest artifacts must be a list")
    actual_tree_cid = tree_cid(records)
    if manifest.get("tree_cid") != actual_tree_cid:
        raise ValueError("artifact tree CID does not reproduce")
    return manifest


def verify_artifact_subset(
    manifest: dict[str, Any],
    *,
    artifact_root: Path,
    relative_paths: Iterable[str],
) -> None:
    """Reproduce only named artifacts, leaving all other committed paths unopened."""
    expected_by_path = {str(record["path"]): record for record in manifest["artifacts"]}
    requested = sorted(set(relative_paths))
    missing = [path for path in requested if path not in expected_by_path]
    if missing:
        raise ValueError(f"manifest does not commit requested artifacts: {missing}")
    reproduced = artifact_records(artifact_root, requested)
    expected = [expected_by_path[path] for path in requested]
    expected.sort(key=lambda record: str(record["path"]))
    if reproduced != expected:
        raise ValueError("selected artifact records do not reproduce")
