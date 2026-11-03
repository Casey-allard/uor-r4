#!/usr/bin/env python3
"""Validate a saved deterministic CPU/serving audit against one artifact."""

from __future__ import annotations

import argparse
import hashlib
import importlib.util
import json
from pathlib import Path
import sys
from typing import Any


class ReceiptError(ValueError):
    pass


def _load_gate():
    path = Path(__file__).with_name("audit_served_forward_gate.py")
    spec = importlib.util.spec_from_file_location("audit_served_forward_gate", path)
    if spec is None or spec.loader is None:
        raise ReceiptError("cannot load the served-forward gate")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


GATE = _load_gate()


def require(condition: bool, message: str) -> None:
    if not condition:
        raise ReceiptError(message)


def require_sha256(value: Any, field: str) -> str:
    require(
        isinstance(value, str)
        and len(value) == 64
        and all(character in "0123456789abcdef" for character in value),
        f"{field} is not 64 lowercase hex characters",
    )
    return value


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as source:
        while chunk := source.read(1 << 20):
            digest.update(chunk)
    return digest.hexdigest()


def load_receipt(path: Path, expected_sha256: str | None) -> tuple[dict[str, Any], str]:
    data = path.read_bytes()
    actual = hashlib.sha256(data).hexdigest()
    if expected_sha256 is not None:
        require_sha256(expected_sha256, "expected receipt hash")
        require(actual == expected_sha256, "receipt hash differs")
    try:
        report = json.loads(data)
    except json.JSONDecodeError as error:
        raise ReceiptError(f"receipt is not valid JSON: {error}") from error
    require(isinstance(report, dict), "receipt root is not an object")
    return report, actual


def artifact_identity(args: argparse.Namespace) -> tuple[str, int]:
    if args.artifact is not None:
        path = Path(args.artifact)
        require(path.is_file(), f"artifact not found: {path}")
        actual_sha = sha256_file(path)
        actual_bytes = path.stat().st_size
        if args.artifact_sha256 is not None:
            require_sha256(args.artifact_sha256, "artifact hash")
            require(actual_sha == args.artifact_sha256, "artifact hash differs from its pin")
        if args.artifact_bytes is not None:
            require(actual_bytes == args.artifact_bytes, "artifact byte count differs from its pin")
        return actual_sha, actual_bytes
    require(args.artifact_sha256 is not None, "--artifact or --artifact-sha256 is required")
    require(args.artifact_bytes is not None, "--artifact-bytes is required with --artifact-sha256")
    return require_sha256(args.artifact_sha256, "artifact hash"), args.artifact_bytes


def validate_receipt(
    report: dict[str, Any],
    *,
    artifact_sha256: str,
    artifact_bytes: int,
    expected_artifact_sha256: str | None = None,
) -> None:
    artifact_sha256 = require_sha256(artifact_sha256, "artifact hash")
    require(artifact_bytes > 0, "artifact byte count is not positive")
    if expected_artifact_sha256 is not None:
        require_sha256(expected_artifact_sha256, "expected artifact hash")
        require(
            artifact_sha256 == expected_artifact_sha256,
            "artifact hash differs from the expected hash",
        )

    source = report.get("token_source")
    require(isinstance(source, dict), "receipt has no token-source provenance")
    offset = source.get("offset")
    count = source.get("count")
    total = source.get("total_tokens")
    vocabulary = source.get("vocabulary")
    require(isinstance(offset, int) and offset >= 0, "token-source offset differs")
    require(isinstance(count, int) and count > 0, "token-source count differs")
    require(isinstance(total, int) and total >= offset + count, "token-source total differs")
    require(isinstance(vocabulary, int) and vocabulary > 0, "token-source vocabulary differs")
    source_sha = require_sha256(source.get("source_sha256"), "token-source hash")
    sequence_sha = require_sha256(
        report.get("token_sequence_sha256"), "token-sequence hash"
    )

    try:
        GATE.validate_report(
            report,
            artifact_bytes=artifact_bytes,
            artifact_sha256=artifact_sha256,
            token_source_sha256=source_sha,
            token_sequence_sha256=sequence_sha,
            token_offset=offset,
            token_count=count,
            total_tokens=total,
            vocabulary=vocabulary,
            expected_artifact_sha256=expected_artifact_sha256,
        )
    except GATE.GateError as error:
        raise ReceiptError(str(error)) from error


def parser() -> argparse.ArgumentParser:
    result = argparse.ArgumentParser(description=__doc__)
    result.add_argument("--receipt", required=True)
    result.add_argument("--expected-receipt-sha256", type=str.lower)
    result.add_argument("--artifact")
    result.add_argument("--artifact-sha256", type=str.lower)
    result.add_argument("--artifact-bytes", type=int)
    result.add_argument("--expected-artifact-sha256", type=str.lower)
    return result


def main() -> int:
    try:
        args = parser().parse_args()
        artifact_sha, artifact_bytes = artifact_identity(args)
        report, receipt_sha = load_receipt(
            Path(args.receipt), args.expected_receipt_sha256
        )
        validate_receipt(
            report,
            artifact_sha256=artifact_sha,
            artifact_bytes=artifact_bytes,
            expected_artifact_sha256=args.expected_artifact_sha256,
        )
    except (OSError, ReceiptError) as error:
        print(f"validate_served_forward_receipt: {error}", file=sys.stderr)
        return 1
    parity = report["parity"]
    print(
        f"PASS receipt={receipt_sha} artifact={artifact_sha} "
        f"positions={parity['positions_checked']} "
        f"q12={parity['query_coordinates_compared']} "
        f"tokens={parity['token_decisions_compared']} "
        f"kept_cells={parity['kept_cells_compared']}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
