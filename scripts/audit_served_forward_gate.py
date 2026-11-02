#!/usr/bin/env python3
"""Run and validate the exact CPU/serving artifact audit before promotion."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
from typing import Any

SCHEMA = "uor-r4.cpu-served-forward-audit/1"
SUBSTRATE = "cpu-development-research"
QUALIFICATION = "gpu-required"


class GateError(ValueError):
    pass


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as source:
        while chunk := source.read(1 << 20):
            digest.update(chunk)
    return digest.hexdigest()


def read_uort(path: Path, offset: int, count: int) -> tuple[str, str, int, int]:
    data = path.read_bytes()
    require(len(data) >= 64 and data[:4] == b"UORT", "token source is not UORT")
    version = int.from_bytes(data[4:8], "little")
    total = int.from_bytes(data[8:16], "little")
    vocabulary = int.from_bytes(data[16:20], "little")
    require(version == 1, "UORT version differs")
    require(not any(data[20:64]), "UORT reserved header bytes are nonzero")
    require(total * 2 + 64 == len(data), "UORT byte length differs")
    require(count > 0 and offset + count <= total, "UORT token span differs")
    start = 64 + offset * 2
    selected = data[start : start + count * 2]
    ids = [
        int.from_bytes(selected[index : index + 2], "little")
        for index in range(0, len(selected), 2)
    ]
    require(all(token < vocabulary for token in ids), "UORT token is outside vocabulary")
    token_bytes = b"".join(token.to_bytes(4, "little") for token in ids)
    return sha256_bytes(data), sha256_bytes(token_bytes), total, vocabulary


def validate_output_paths(
    *,
    artifact: Path,
    uort: Path,
    report: Path | None,
    promote: Path | None,
) -> None:
    inputs = [artifact.resolve(), uort.resolve()]
    outputs = [path.resolve() for path in (report, promote) if path is not None]
    require(len(outputs) == len(set(outputs)), "report and promotion destinations alias")
    for output in outputs:
        require(output not in inputs, "an output destination aliases an input")


def require(condition: bool, message: str) -> None:
    if not condition:
        raise GateError(message)


def expected_counts(report: dict[str, Any]) -> tuple[int, int, int, int]:
    shape = report["manifest"]["shape"]
    positions = report["parity"]["positions_checked"]
    return (
        positions,
        positions * shape["width"],
        positions,
        positions * shape["kept_cells"],
    )


def validate_report(
    report: dict[str, Any],
    *,
    artifact_bytes: int,
    artifact_sha256: str,
    token_source_sha256: str,
    token_sequence_sha256: str,
    token_offset: int,
    token_count: int,
    total_tokens: int,
    vocabulary: int,
    expected_artifact_sha256: str | None,
) -> None:
    require(report.get("schema") == SCHEMA, "audit schema differs")
    require(report.get("substrate") == SUBSTRATE, "audit substrate differs")
    require(report.get("qualification") == QUALIFICATION, "audit qualification differs")
    require(report.get("timing") is None, "deterministic gate report contains timing")

    artifact = report.get("artifact", {})
    require(artifact.get("sha256") == artifact_sha256, "report artifact hash differs")
    require(artifact.get("bytes") == artifact_bytes, "report artifact byte count differs")
    if expected_artifact_sha256 is not None:
        require(
            artifact_sha256 == expected_artifact_sha256,
            "artifact hash differs from the expected hash",
        )

    source = report.get("token_source")
    require(source is not None, "report has no token-source provenance")
    require(source.get("format") == "uort", "token-source format is not UORT")
    require(source.get("source_sha256") == token_source_sha256, "token-source hash differs")
    require(source.get("offset") == token_offset, "token-source offset differs")
    require(source.get("count") == token_count, "token-source count differs")
    require(source.get("total_tokens") == total_tokens, "token-source total differs")
    require(source.get("vocabulary") == vocabulary, "token-source vocabulary differs")
    require(
        report.get("token_sequence_sha256") == token_sequence_sha256,
        "selected token sequence hash differs",
    )

    parity = report.get("parity", {})
    positions, coordinates, tokens, kept_cells = expected_counts(report)
    require(positions == token_count, "position count differs from selected tokens")
    require(parity.get("positions_checked") == positions, "position count differs")
    require(parity.get("query_coordinates_compared") == coordinates, "Q12 count differs")
    require(parity.get("token_decisions_compared") == tokens, "token count differs")
    require(parity.get("kept_cells_compared") == kept_cells, "kept-cell count differs")
    require(parity.get("query_coordinate_mismatches") == 0, "Q12 mismatch")
    require(parity.get("token_mismatches") == 0, "token mismatch")
    require(parity.get("kept_cell_mismatches") == 0, "kept-cell mismatch")
    require(parity.get("instrument_fault") is False, "audit reports an instrument fault")
    require(
        parity.get("cpu_decision_sha256") == parity.get("serving_decision_sha256"),
        "CPU and serving decision hashes differ",
    )
    decision_hash = parity.get("cpu_decision_sha256", "")
    require(
        len(decision_hash) == 64
        and all(character in "0123456789abcdef" for character in decision_hash),
        "decision hash is not 64 lowercase hex characters",
    )
    mismatches = parity.get("mismatches")
    require(isinstance(mismatches, list), "mismatch details are not a list")
    require(
        parity.get("mismatches_recorded") == len(mismatches),
        "recorded mismatch count differs",
    )
    require(parity.get("mismatches_omitted") == 0, "audit omitted mismatch details")
    require(not mismatches, "a passing audit carries mismatch details")


def atomic_copy(source: Path, destination: Path, expected_sha256: str) -> None:
    require(not destination.exists(), f"{destination} already exists")
    destination.parent.mkdir(parents=True, exist_ok=True)
    temporary = destination.with_name(f".{destination.name}.gate-{os.getpid()}")
    try:
        shutil.copyfile(source, temporary)
        require(
            sha256_file(temporary) == expected_sha256,
            "artifact changed after its audited hash was taken",
        )
        try:
            os.link(temporary, destination)
        except FileExistsError as error:
            raise GateError(f"{destination} was created concurrently") from error
    finally:
        temporary.unlink(missing_ok=True)


def atomic_json(destination: Path, value: dict[str, Any]) -> None:
    require(not destination.exists(), f"{destination} already exists")
    destination.parent.mkdir(parents=True, exist_ok=True)
    temporary = destination.with_name(f".{destination.name}.gate-{os.getpid()}")
    try:
        temporary.write_text(json.dumps(value, indent=2) + "\n", encoding="utf-8")
        try:
            os.link(temporary, destination)
        except FileExistsError as error:
            raise GateError(f"{destination} was created concurrently") from error
    finally:
        temporary.unlink(missing_ok=True)


def run_gate(args: argparse.Namespace) -> dict[str, Any]:
    artifact = Path(args.artifact)
    uort = Path(args.uort)
    report_path = Path(args.report) if args.report else None
    promote = Path(args.promote) if args.promote else None
    validate_output_paths(
        artifact=artifact,
        uort=uort,
        report=report_path,
        promote=promote,
    )
    if promote is not None:
        require(not promote.exists(), f"{promote} already exists")
    if report_path is not None:
        require(not report_path.exists(), f"{report_path} already exists")

    artifact_hash = sha256_file(artifact)
    token_hash, sequence_hash, total_tokens, vocabulary = read_uort(
        uort, args.token_offset, args.token_count
    )
    with tempfile.TemporaryDirectory(prefix="uor-served-forward-gate-") as temporary:
        raw_report = Path(temporary) / "report.json"
        command = [
            args.binary,
            "--artifact",
            str(artifact),
            "--uort",
            str(uort),
            "--token-offset",
            str(args.token_offset),
            "--token-count",
            str(args.token_count),
            "--expected-artifact-sha256",
            artifact_hash,
            "--no-timing",
            "--output",
            str(raw_report),
        ]
        if args.reply_start is not None:
            command.extend(["--reply-start", str(args.reply_start)])
        completed = subprocess.run(command, text=True, capture_output=True, check=False)
        require(
            completed.returncode == 0,
            f"audit failed ({completed.returncode}): {completed.stderr.strip()}",
        )
        report = json.loads(raw_report.read_text(encoding="utf-8"))
        validate_report(
            report,
            artifact_bytes=artifact.stat().st_size,
            artifact_sha256=artifact_hash,
            token_source_sha256=token_hash,
            token_sequence_sha256=sequence_hash,
            token_offset=args.token_offset,
            token_count=args.token_count,
            total_tokens=total_tokens,
            vocabulary=vocabulary,
            expected_artifact_sha256=args.expected_artifact_sha256,
        )

    if report_path is not None:
        atomic_json(report_path, report)
    if promote is not None:
        atomic_copy(artifact, promote, artifact_hash)
    return report


def parser() -> argparse.ArgumentParser:
    result = argparse.ArgumentParser(description=__doc__)
    result.add_argument("--binary", required=True, help="cpu-served-forward-audit executable")
    result.add_argument("--artifact", required=True)
    result.add_argument("--uort", required=True)
    result.add_argument("--token-offset", required=True, type=int)
    result.add_argument("--token-count", required=True, type=int)
    result.add_argument("--reply-start", type=int)
    result.add_argument("--expected-artifact-sha256", type=str.lower)
    result.add_argument("--report")
    result.add_argument("--promote", help="copy the artifact here only after the gate passes")
    return result


def main() -> int:
    args = parser().parse_args()
    try:
        report = run_gate(args)
    except (OSError, ValueError, json.JSONDecodeError) as error:
        print(f"audit_served_forward_gate: {error}", file=sys.stderr)
        return 1
    parity = report["parity"]
    print(
        f"PASS positions={parity['positions_checked']} "
        f"q12={parity['query_coordinates_compared']} "
        f"tokens={parity['token_decisions_compared']} "
        f"kept_cells={parity['kept_cells_compared']}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
