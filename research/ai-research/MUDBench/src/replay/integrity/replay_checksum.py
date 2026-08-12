"""Deterministic replay artifact integrity checksum helpers."""

from __future__ import annotations

import hashlib
import json
import re
from dataclasses import dataclass
from typing import Any, Mapping

from replay.logging.replay_artifact import (
    ReplayArtifact,
    ReplayArtifactEmitResult,
    parse_replay_artifact,
)

CHECKSUM_ALGORITHM = "sha256"
_SHA256_HEX_RE = re.compile(r"^[0-9a-f]{64}$")


@dataclass(frozen=True, slots=True)
class ReplayArtifactChecksum:
    """Canonical checksum descriptor for one replay artifact payload."""

    algorithm: str
    digest_hex: str
    canonical_size_bytes: int

    def __post_init__(self) -> None:
        if self.algorithm != CHECKSUM_ALGORITHM:
            raise ValueError(f"algorithm must be {CHECKSUM_ALGORITHM}")
        if not isinstance(self.digest_hex, str) or not _SHA256_HEX_RE.fullmatch(self.digest_hex):
            raise ValueError("digest_hex must be a 64-character lowercase SHA-256 hex string")
        if (
            not isinstance(self.canonical_size_bytes, int)
            or isinstance(self.canonical_size_bytes, bool)
            or self.canonical_size_bytes < 0
        ):
            raise ValueError("canonical_size_bytes must be a non-negative integer")

    def to_dict(self) -> dict[str, Any]:
        return {
            "algorithm": self.algorithm,
            "digest_hex": self.digest_hex,
            "canonical_size_bytes": self.canonical_size_bytes,
        }

    def to_canonical_json(self) -> str:
        return json.dumps(self.to_dict(), sort_keys=True, separators=(",", ":"), ensure_ascii=True)


@dataclass(frozen=True, slots=True)
class ReplayChecksumComputationResult:
    """Explicit accept/reject checksum computation result."""

    accepted: bool
    checksum: ReplayArtifactChecksum | None = None
    reason: str | None = None
    artifact_result: ReplayArtifactEmitResult | None = None

    def __post_init__(self) -> None:
        if self.accepted:
            if self.checksum is None:
                raise ValueError("accepted computation result must include checksum")
            if self.reason is not None:
                raise ValueError("accepted computation result must not include reason")
            return
        if self.reason is None:
            raise ValueError("rejected computation result must include reason")
        if self.checksum is not None:
            raise ValueError("rejected computation result must not include checksum")


@dataclass(frozen=True, slots=True)
class ReplayChecksumVerificationResult:
    """Explicit deterministic checksum verification result."""

    accepted: bool
    matched: bool | None = None
    expected_digest: str | None = None
    actual_checksum: ReplayArtifactChecksum | None = None
    reason: str | None = None
    computation_result: ReplayChecksumComputationResult | None = None

    def __post_init__(self) -> None:
        if self.accepted:
            if self.matched is None:
                raise ValueError("accepted verification result must include matched")
            if self.expected_digest is None:
                raise ValueError("accepted verification result must include expected_digest")
            if self.actual_checksum is None:
                raise ValueError("accepted verification result must include actual_checksum")
            if self.reason is not None:
                raise ValueError("accepted verification result must not include reason")
            return
        if self.reason is None:
            raise ValueError("rejected verification result must include reason")
        if self.matched is not None:
            raise ValueError("rejected verification result must not include matched")


def compute_replay_artifact_checksum(artifact: object) -> ReplayChecksumComputationResult:
    """Compute deterministic replay checksum from canonical artifact bytes."""
    resolved_artifact, artifact_result, reason = _resolve_artifact(artifact)
    if reason is not None:
        return _rejected_computation(reason=reason, artifact_result=artifact_result)
    if resolved_artifact is None:
        raise RuntimeError("internal error resolving replay artifact")

    canonical_bytes = resolved_artifact.to_canonical_bytes()
    digest_hex = hashlib.sha256(canonical_bytes).hexdigest()
    checksum = ReplayArtifactChecksum(
        algorithm=CHECKSUM_ALGORITHM,
        digest_hex=digest_hex,
        canonical_size_bytes=len(canonical_bytes),
    )
    return _accepted_computation(checksum=checksum, artifact_result=artifact_result)


def verify_replay_artifact_checksum(
    *,
    artifact: object,
    expected_digest: str,
) -> ReplayChecksumVerificationResult:
    """Verify deterministic checksum match between artifact and expected digest."""
    normalized_expected = _normalize_expected_digest(expected_digest)
    if normalized_expected is None:
        return _rejected_verification(reason="expected_digest_must_be_sha256_hex")

    computation_result = compute_replay_artifact_checksum(artifact)
    if not computation_result.accepted or computation_result.checksum is None:
        reason = computation_result.reason or "unknown_checksum_computation_failure"
        return _rejected_verification(
            reason=f"cannot_compute_checksum:{reason}",
            computation_result=computation_result,
        )

    checksum = computation_result.checksum
    return ReplayChecksumVerificationResult(
        accepted=True,
        matched=checksum.digest_hex == normalized_expected,
        expected_digest=normalized_expected,
        actual_checksum=checksum,
        computation_result=computation_result,
    )


def _resolve_artifact(
    artifact: object,
) -> tuple[ReplayArtifact | None, ReplayArtifactEmitResult | None, str | None]:
    if isinstance(artifact, ReplayArtifact):
        return artifact, None, None
    if isinstance(artifact, Mapping):
        parse_result = parse_replay_artifact(artifact)
        if not parse_result.accepted or parse_result.artifact is None:
            parse_reason = parse_result.reason or "unknown_parse_rejection"
            return None, parse_result, f"invalid_artifact:{parse_reason}"
        return parse_result.artifact, parse_result, None
    return None, None, "artifact_must_be_replay_artifact_or_mapping"


def _normalize_expected_digest(expected_digest: object) -> str | None:
    if not isinstance(expected_digest, str):
        return None
    candidate = expected_digest.lower()
    if not _SHA256_HEX_RE.fullmatch(candidate):
        return None
    return candidate


def _accepted_computation(
    *,
    checksum: ReplayArtifactChecksum,
    artifact_result: ReplayArtifactEmitResult | None = None,
) -> ReplayChecksumComputationResult:
    return ReplayChecksumComputationResult(
        accepted=True,
        checksum=checksum,
        artifact_result=artifact_result,
    )


def _rejected_computation(
    *,
    reason: str,
    artifact_result: ReplayArtifactEmitResult | None = None,
) -> ReplayChecksumComputationResult:
    return ReplayChecksumComputationResult(
        accepted=False,
        reason=reason,
        artifact_result=artifact_result,
    )


def _rejected_verification(
    *,
    reason: str,
    computation_result: ReplayChecksumComputationResult | None = None,
) -> ReplayChecksumVerificationResult:
    return ReplayChecksumVerificationResult(
        accepted=False,
        reason=reason,
        computation_result=computation_result,
    )
