#!/usr/bin/env python3
"""Host-only consumer verifier for the `distill-d1` one-time offline donor corpus (D41).

Verifies:
1. Staged registration sheet (`docs/harness/prep/m1-distill-d1-donor-corpus-DRAFT-2026-10-19.md`),
   enforcing top-line and bottom-line `TEST FITNESS` consistency (`sheet_status_contradiction` on
   mismatch), pinning both the pre-FIT draft (`dcf3ec1e4913126dbae71fc8674acc98e3c9a31ecf129c5c874c645201d14e73`,
   `24,263` B) and the post-FIT `#2774` sheet (`51797f33cf13981529fe10f29433ff52020cf1cf354603d61d229a3cc30ad7d6`,
   `25,622` B), D41 record in `docs/integration/DECISIONS.md`, pinned student tokenizer contract
   (`d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89`, vocab 4096, BOS 0),
   `crates/uor-r4-discrete/src/metadata.rs` (`MAX_CONTEXT = 243`),
   `crates/uor-r4-discrete/src/corpus.rs` (`WINDOW_POLICY`, `IDENTITY_POLICY`, `UORT_HEADER_BYTES = 64`),
   and `crates/uor-r4-training/src/knowledge_corpus.rs` (`PANEL_NGRAM = 8`, `MIN_SHORT_REQUEST_WORDS = 4`).
2. §8 13-item registration checklist and D40/D41 dispatch bindings (requiring post-FIT sheet SHA-256,
   pinned 40/64-hex donor commit SHA on `openai/gpt-oss-120b` and rejecting un-amended Qwen3/5090
   fallback, MXFP4 weights, Apache-2.0 license, pinned vLLM image digest, pre-screened prompt set,
   fixed sampling parameters, 1x RTX PRO 6000 96 GB pod <= $8/h, completed driver math optimization
   prerequisite, `student_training_authorized = False`, explicit `merkle_mode`, Casey's personal D40
   release with `UOR_LAB` unset, and hard-targets-only student lineage `distill-d1`).
3. `uor-r4.donor-corpus/1` 5-block generation receipt (`donor_pin`, `generation_pin`, `run_pin`,
   `content_pin`, `readback`), strict non-null/non-empty field check (D31), ascending shard order,
   explicit `content_pin.merkle_mode` (`"hex-colon"` or `"raw-concat"`, matching checklist), and
   canonical binary Merkle root recomputation over per-shard SHA-256 leaves.
4. Pilot tranche gates `P1–P8` and §4.2 Go/No-Go decision grader (`GO_FOR_PRODUCTION_TRANCHE` at
   `>= 300` tok/s, `HOLD_BELOW_300_TOK_S_GO_BAR` at `250..300` tok/s,
   `STOP_THROUGHPUT_SHORTFALL_OWNER_REPLAN` at `< 250` tok/s,
   `PILOT_REJECTED_CORPUS_DEFECT_CONSUMES_D41_PASS` on `G4–G11` quality defect, and
   `INSTRUMENT_UNREACHABLE_D31` on instrument/receipt fault), plus §4.3 shard repair vs
   regeneration enforcement.
5. Production corpus gates `G1–G14` (`G1` donor bind, `G2` receipt completeness, `G3` readback +
   Merkle root, `G4` exact SHA-256 dedup, `G5` 5-word shingle Jaccard >= 0.80 near-dedup < 3.0%,
   `G6` `PanelExclusion` 8-word window + 4–7-word short-turn contamination == 0, `G7` 12-word
   `mw-balp` TRAIN overlap == 0, `G8` sandboxed code execution 100% pass / 0 unevaluated,
   `G9` deterministic math answer check 100% pass / 0 unevaluated, `G10` 200-doc per-slice spot
   audit malformed <= 2.0% and refusals <= 5.0% with at most 1 redraw, `G11a` kept slice shares
   within +-3.0 pp of target or renormalized shares after slice drop, `G11b` truncation < 1.0%,
   `G11c` 8-word n-gram repeated > 3 times in < 2.0% of kept docs, `G11d` length >= 16,
   median in 40..120, p99 <= 200, `G12` UORT v1 admission + dev >= 250,000 eligible targets +
   sealed held-out split mapped to `Split::Fresh`, `G13` C=192 64-row window admission as the
   mandatory seal gate with C=512 recorded as `NOT_RUN_MAX_CONTEXT_243` diagnostic, and
   `G14` seal + D41 exception expiry).
6. Raw binary/text audit helpers (`verify_uort_and_mask_bytes`, `check_panel_exclusion_text`,
   `check_5word_jaccard_clusters`, `check_12word_train_overlap`, `check_8gram_repetition`) and a
   32-test positive and negative self-test suite (`--self-test`).

Host-only ($0 GPU). No model execution, no pod dispatch, and no hold release.
"""

from __future__ import annotations

import argparse
import bisect
import copy
import hashlib
import json
import math
import re
import struct
import sys
from pathlib import Path
from typing import Any, Callable, Sequence

# ---------------------------------------------------------------------------
# Pinned Constants & Thresholds
# ---------------------------------------------------------------------------

DRAFT_SHEET_REL_PATH = "docs/harness/prep/m1-distill-d1-donor-corpus-DRAFT-2026-10-19.md"
EXPECTED_DRAFT_SHEET_SHA256 = "dcf3ec1e4913126dbae71fc8674acc98e3c9a31ecf129c5c874c645201d14e73"
EXPECTED_DRAFT_SHEET_BYTES = 24_263
EXPECTED_FIT_SHEET_SHA256 = "51797f33cf13981529fe10f29433ff52020cf1cf354603d61d229a3cc30ad7d6"
EXPECTED_FIT_SHEET_BYTES = 25_622
INITIAL_DRAFT_SHEET_SHA256 = EXPECTED_DRAFT_SHEET_SHA256

METADATA_RS_REL_PATH = "crates/uor-r4-discrete/src/metadata.rs"
METADATA_RS_SHA256 = "27e53e4feecb391adb4a4db3462712a8d9375aed09c0e57086581134cb6e1e32"

CORPUS_RS_REL_PATH = "crates/uor-r4-discrete/src/corpus.rs"
CORPUS_RS_SHA256 = "826bdaba978b3c54542147656c8aa3cf24d1aa39b6711528b02b545c4fda9f7e"

KNOWLEDGE_CORPUS_RS_REL_PATH = "crates/uor-r4-training/src/knowledge_corpus.rs"
KNOWLEDGE_CORPUS_RS_SHA256 = "d07b19c97905f137fcc6c6878e07ec4989e68bc5a31dc2b6fe85125e669a3c2c"

MILESTONE_WORLD_RS_REL_PATH = "crates/uor-r4-training/src/milestone_world.rs"
MILESTONE_WORLD_RS_SHA256 = "caf82592999c88afc17a183bac0508c996495697527584f4824647f6cbd2dccc"

RUNG0_SHEET_REL_PATH = "docs/labs/rung0-pilot-release-2026-10-14/sheet.md"
RUNG0_SHEET_SHA256 = "1029c3b92771e0938d2ec393ce419db81cdc17125dad4814243c235cfb7463d5"

DECISIONS_MD_REL_PATH = "docs/integration/DECISIONS.md"

EXPECTED_STUDENT_TOKENIZER_SHA256 = (
    "d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89"
)
EXPECTED_STUDENT_VOCABULARY = 4096
EXPECTED_STUDENT_BOS_ID = 0
EXPECTED_STUDENT_EOS_ID = 1
EXPECTED_STUDENT_UNK_ID = 2

EXPECTED_DONOR_REPO = "openai/gpt-oss-120b"
EXPECTED_DONOR_QUANTIZATION = "MXFP4"
EXPECTED_DONOR_LICENSE_SPDX = "Apache-2.0"
EXPECTED_RECEIPT_SCHEMA = "uor-r4.donor-corpus/1"
EXPECTED_LINEAGE_ID = "distill-d1"

SUPPORTED_MERKLE_MODES: tuple[str, ...] = (
    "hex-colon",
    "raw-concat",
    "sha256_concat_hex_odd_promote_v1",
    "sha256_concat_bytes_odd_promote_v1",
)

EXPECTED_WINDOW_POLICY = (
    "response-masked-targets/within-document-predecessor/sliding-prefix/ascending-once/v1"
)
EXPECTED_IDENTITY_POLICY = (
    "corpus-SHA-bound/split-prefix2bits/document-ordinal62bits/absolute-token-occurrence/version0/v1"
)
EXPECTED_UORT_HEADER_BYTES = 64
EXPECTED_MAX_CONTEXT = 243
EXPECTED_OPERATING_CONTEXT = 192
EXPECTED_BATCH_ROWS = 64

CANONICAL_SLICES: tuple[str, ...] = (
    "conversation",
    "qa_instruction",
    "code",
    "reasoning_prose",
    "exposition",
)
EXPECTED_SLICE_SHARES: dict[str, float] = {
    "conversation": 0.35,
    "qa_instruction": 0.20,
    "code": 0.20,
    "reasoning_prose": 0.15,
    "exposition": 0.10,
}
SLICE_SHARE_TOLERANCE = 0.03

PRODUCTION_TOKEN_FLOOR = 20_000_000
PRODUCTION_TOKEN_TARGET = 30_000_000
PRODUCTION_TOKEN_CEILING = 50_000_000
PILOT_TOKEN_TARGET = 1_000_000

DOC_MIN_TOKENS = 16
DOC_MAX_TOKENS = 200
DOC_MEDIAN_MIN_TOKENS = 40
DOC_MEDIAN_MAX_TOKENS = 120
DEV_MIN_ELIGIBLE_TARGETS = 250_000

NEAR_DEDUP_SHINGLE_WORDS = 5
NEAR_DEDUP_JACCARD_THRESHOLD = 0.80
NEAR_DEDUP_MAX_RESIDUAL_RATE = 0.03

PANEL_EXCLUSION_NGRAM_WORDS = 8
PANEL_EXCLUSION_SHORT_MIN_WORDS = 4
PANEL_EXCLUSION_SHORT_MAX_WORDS = 7
TRAIN_OVERLAP_NGRAM_WORDS = 12

CODE_SANDBOX_MAX_TIMEOUT_S = 10
CODE_SANDBOX_MAX_MEMORY_MIB = 1024

SPOT_AUDIT_SAMPLE_PER_SLICE = 200
SPOT_AUDIT_MAX_MALFORMED_RATE = 0.02
SPOT_AUDIT_MAX_REFUSAL_RATE = 0.05
SPOT_AUDIT_MAX_SLICE_REDRAWS = 1

TRUNCATION_MAX_RATE = 0.01
REPETITION_NGRAM_WORDS = 8
REPETITION_MAX_OCCURRENCES = 3
REPETITION_MAX_DOC_RATE = 0.02

PILOT_GO_TOK_PER_S = 300.0
PILOT_STOP_TOK_PER_S = 250.0
MAX_POD_RATE_USD_PER_GPU_HR = 8.0
MAX_RUNNING_PODS = 4

REQUIRED_SHEET_HEADINGS: tuple[str, ...] = (
    "## Bar",
    "## Base artifact",
    "## Data split",
    "## 1. Corpus specification",
    "## 2. Donor binding",
    "### 2.1 Generation receipt — `uor-r4.donor-corpus/1`",
    "## 3. Quality gates",
    "## 4. Pilot design",
    "## 5. Student consumption contract",
    "## 6. Cost and minimum test power",
    "## 7. Steps",
    "## 8. Registration checklist — bound at dispatch",
    "## 9. Stop rule",
    "## 10. What this sheet can and cannot decide",
    "## 11. Why the corpus is sized to the student, not the donor",
)


class VerificationError(ValueError):
    """Structured consumer verification failure with D31/D41 classification."""

    def __init__(
        self,
        message: str,
        *,
        classification: str = "INSTRUMENT_UNREACHABLE_D31",
        gate: str | None = None,
        d41_pass_consumed: bool = False,
    ) -> None:
        super().__init__(message)
        self.classification = classification
        self.gate = gate
        self.d41_pass_consumed = d41_pass_consumed


# ---------------------------------------------------------------------------
# Basic Cryptographic & Validation Helpers
# ---------------------------------------------------------------------------


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def sha256_file(path: Path) -> tuple[str, int]:
    if not path.is_file():
        raise VerificationError(
            f"missing required file: {path}",
            classification="INSTRUMENT_UNREACHABLE_D31",
        )
    raw = path.read_bytes()
    return sha256_bytes(raw), len(raw)


def validate_hex(value: Any, expected_len: int, field_name: str, *, gate: str | None = None) -> str:
    if not isinstance(value, str) or len(value) != expected_len:
        raise VerificationError(
            f"{field_name} must be lowercase {expected_len}-hex (got {value!r})",
            classification="INSTRUMENT_UNREACHABLE_D31",
            gate=gate,
        )
    for ch in value:
        if not ("0" <= ch <= "9" or "a" <= ch <= "f"):
            raise VerificationError(
                f"{field_name} must be lowercase {expected_len}-hex (got {value!r})",
                classification="INSTRUMENT_UNREACHABLE_D31",
                gate=gate,
            )
    return value


def validate_sha256_hex(value: Any, field_name: str, *, gate: str | None = None) -> str:
    return validate_hex(value, 64, field_name, gate=gate)


def validate_commit_sha(value: Any, field_name: str, *, gate: str | None = None) -> str:
    if not isinstance(value, str):
        raise VerificationError(
            f"{field_name} must be a pinned commit SHA string",
            classification="INSTRUMENT_UNREACHABLE_D31",
            gate=gate,
        )
    forbidden = {"main", "master", "head", "latest", "default"}
    if value.strip().lower() in forbidden:
        raise VerificationError(
            f"{field_name} must be a pinned commit SHA, not a mutable ref ({value!r})",
            classification="INSTRUMENT_UNREACHABLE_D31",
            gate=gate,
        )
    if len(value) not in (40, 64):
        raise VerificationError(
            f"{field_name} must be 40-hex or 64-hex pinned commit SHA (got {value!r})",
            classification="INSTRUMENT_UNREACHABLE_D31",
            gate=gate,
        )
    return validate_hex(value, len(value), field_name, gate=gate)


def ensure_no_null_or_empty(obj: Any, path: str, *, gate: str = "G2") -> None:
    """Recursively verify that no field inside a receipt block is None or empty."""
    if obj is None:
        raise VerificationError(
            f"{path} is null (receipt missing mandatory field under D31)",
            classification="INSTRUMENT_UNREACHABLE_D31",
            gate=gate,
        )
    if isinstance(obj, str):
        if not obj.strip():
            raise VerificationError(
                f"{path} is empty string (receipt missing mandatory field under D31)",
                classification="INSTRUMENT_UNREACHABLE_D31",
                gate=gate,
            )
        return
    if isinstance(obj, (bool, int, float)):
        if isinstance(obj, float) and (math.isnan(obj) or math.isinf(obj)):
            raise VerificationError(
                f"{path} is NaN/Inf",
                classification="INSTRUMENT_UNREACHABLE_D31",
                gate=gate,
            )
        return
    if isinstance(obj, list):
        if len(obj) == 0:
            raise VerificationError(
                f"{path} is an empty list",
                classification="INSTRUMENT_UNREACHABLE_D31",
                gate=gate,
            )
        for idx, item in enumerate(obj):
            ensure_no_null_or_empty(item, f"{path}[{idx}]", gate=gate)
        return
    if isinstance(obj, dict):
        if len(obj) == 0:
            raise VerificationError(
                f"{path} is an empty object",
                classification="INSTRUMENT_UNREACHABLE_D31",
                gate=gate,
            )
        for k, v in obj.items():
            if not isinstance(k, str) or not k.strip():
                raise VerificationError(
                    f"{path} has empty key",
                    classification="INSTRUMENT_UNREACHABLE_D31",
                    gate=gate,
                )
            ensure_no_null_or_empty(v, f"{path}.{k}", gate=gate)
        return
    raise VerificationError(
        f"{path} has unsupported JSON type {type(obj).__name__}",
        classification="INSTRUMENT_UNREACHABLE_D31",
        gate=gate,
    )


def compute_shard_merkle_root(
    shard_sha256s: Sequence[str],
    mode: str = "hex-colon",
) -> str:
    """Compute canonical binary Merkle root over ordered shard SHA-256 leaves (§2.1).

    Modes supported:
    - `hex-colon`: pairs `(left, right)` hash `SHA256(left_hex_ascii || ":" || right_hex_ascii)`;
      an odd trailing node is promoted unchanged at each level; a single-shard list hashes
      `SHA256(leaf_hex_ascii)`.
    - `raw-concat` (or alias `sha256_concat_bytes_odd_promote_v1`): pairs `(left, right)` hash
      `SHA256(left_bytes || right_bytes)`; an odd trailing node is promoted unchanged; a
      single-shard list hashes `SHA256(leaf_bytes)`.
    - `sha256_concat_hex_odd_promote_v1`: pairs `(left, right)` hash
      `SHA256(left_hex_ascii || right_hex_ascii)` without colon.
    """
    if not shard_sha256s:
        raise VerificationError(
            "cannot compute Merkle root over empty shard list",
            classification="INSTRUMENT_UNREACHABLE_D31",
            gate="G2",
        )
    leaves = [
        validate_sha256_hex(s, f"shard_sha256s[{i}]", gate="G2")
        for i, s in enumerate(shard_sha256s)
    ]
    if mode not in SUPPORTED_MERKLE_MODES:
        raise VerificationError(
            f"unsupported merkle_mode {mode!r}; expected one of {SUPPORTED_MERKLE_MODES}",
            classification="INSTRUMENT_UNREACHABLE_D31",
            gate="G2",
        )
    if len(leaves) == 1:
        if mode in ("raw-concat", "sha256_concat_bytes_odd_promote_v1"):
            return sha256_bytes(bytes.fromhex(leaves[0]))
        return sha256_bytes(leaves[0].encode("ascii"))

    level = list(leaves)
    while len(level) > 1:
        next_level: list[str] = []
        i = 0
        while i + 1 < len(level):
            left = level[i]
            right = level[i + 1]
            if mode == "hex-colon":
                combined = sha256_bytes(f"{left}:{right}".encode("ascii"))
            elif mode == "sha256_concat_hex_odd_promote_v1":
                combined = sha256_bytes((left + right).encode("ascii"))
            else:
                combined = sha256_bytes(bytes.fromhex(left) + bytes.fromhex(right))
            next_level.append(combined)
            i += 2
        if i < len(level):
            next_level.append(level[i])
        level = next_level
    return level[0]


# ---------------------------------------------------------------------------
# 1. Staged Registration Sheet & Repository Contract Verifier
# ---------------------------------------------------------------------------


def parse_sheet_fitness_status(sheet_text: str) -> str:
    """Check both the top `- **Status:**` line and bottom `TEST FITNESS:` line for consistency."""
    lines = [ln.strip() for ln in sheet_text.splitlines() if ln.strip()]
    status_lines = [ln for ln in lines[:12] if ln.startswith("- **Status:**")]
    if not status_lines:
        raise VerificationError(
            "registration sheet missing top `- **Status:**` line",
            classification="INSTRUMENT_UNREACHABLE_D31",
        )
    top_line = status_lines[0]
    bottom_lines = [ln for ln in lines[-8:] if ln.startswith("TEST FITNESS:")]
    if not bottom_lines:
        raise VerificationError(
            "registration sheet missing bottom `TEST FITNESS:` line",
            classification="INSTRUMENT_UNREACHABLE_D31",
        )
    bottom_line = bottom_lines[-1]

    top_fit = "TEST FITNESS: FIT" in top_line
    top_pending = "TEST FITNESS: PENDING" in top_line
    bottom_fit = bottom_line.startswith("TEST FITNESS: FIT")
    bottom_pending = bottom_line.startswith("TEST FITNESS: PENDING")

    if (top_fit and bottom_pending) or (top_pending and bottom_fit) or (top_fit != bottom_fit) or (top_pending != bottom_pending):
        raise VerificationError(
            f"sheet_status_contradiction: top status line ({top_line!r}) and bottom TEST FITNESS line ({bottom_line!r}) disagree",
            classification="INSTRUMENT_UNREACHABLE_D31",
        )
    if top_fit and bottom_fit:
        return "FIT"
    if top_pending and bottom_pending:
        return "PENDING_DRAFT"
    raise VerificationError(
        "registration sheet must consistently declare TEST FITNESS: PENDING or TEST FITNESS: FIT at both top and bottom",
        classification="INSTRUMENT_UNREACHABLE_D31",
    )


def verify_sheet_and_repo_contract(
    repo_root: Path,
    sheet_path: Path | None = None,
) -> dict[str, Any]:
    """Verify the staged `distill-d1` sheet and repository source contracts."""
    target_sheet = sheet_path if sheet_path is not None else (repo_root / DRAFT_SHEET_REL_PATH)
    sheet_sha, sheet_bytes = sha256_file(target_sheet)
    sheet_text = target_sheet.read_text(encoding="utf-8")

    missing_headings = [h for h in REQUIRED_SHEET_HEADINGS if h not in sheet_text]
    if missing_headings:
        raise VerificationError(
            f"registration sheet missing required headings: {missing_headings}",
            classification="INSTRUMENT_UNREACHABLE_D31",
        )

    required_tokens = (
        EXPECTED_STUDENT_TOKENIZER_SHA256,
        EXPECTED_DONOR_REPO,
        EXPECTED_DONOR_QUANTIZATION,
        EXPECTED_DONOR_LICENSE_SPDX,
        EXPECTED_RECEIPT_SCHEMA,
        "30,000,000",
        "20,000,000",
        "50,000,000",
        "250,000",
        "RTX PRO 6000",
        "G1",
        "G14",
        "P1",
        "P8",
        "D40",
        "D41",
    )
    missing_tokens = [tok for tok in required_tokens if tok not in sheet_text]
    if missing_tokens:
        raise VerificationError(
            f"registration sheet missing required contract tokens: {missing_tokens}",
            classification="INSTRUMENT_UNREACHABLE_D31",
        )

    fitness_status = parse_sheet_fitness_status(sheet_text)

    metadata_path = repo_root / METADATA_RS_REL_PATH
    metadata_sha, metadata_bytes = sha256_file(metadata_path)
    metadata_text = metadata_path.read_text(encoding="utf-8")
    if "pub const MAX_CONTEXT: usize = 243;" not in metadata_text:
        raise VerificationError(
            "metadata.rs missing `pub const MAX_CONTEXT: usize = 243;`",
            classification="INSTRUMENT_UNREACHABLE_D31",
        )

    corpus_path = repo_root / CORPUS_RS_REL_PATH
    corpus_sha, corpus_bytes = sha256_file(corpus_path)
    corpus_text = corpus_path.read_text(encoding="utf-8")
    for snippet in (
        EXPECTED_WINDOW_POLICY,
        EXPECTED_IDENTITY_POLICY,
        "pub const UORT_HEADER_BYTES: usize = 64;",
        "pub enum Split",
    ):
        if snippet not in corpus_text:
            raise VerificationError(
                f"corpus.rs missing required contract snippet: {snippet!r}",
                classification="INSTRUMENT_UNREACHABLE_D31",
            )

    knowledge_path = repo_root / KNOWLEDGE_CORPUS_RS_REL_PATH
    knowledge_sha, knowledge_bytes = sha256_file(knowledge_path)
    knowledge_text = knowledge_path.read_text(encoding="utf-8")
    for snippet in (
        "pub const PANEL_NGRAM: usize = EXCLUSION_NGRAM;",
        "pub const MIN_SHORT_REQUEST_WORDS: usize = 4;",
    ):
        if snippet not in knowledge_text:
            raise VerificationError(
                f"knowledge_corpus.rs missing required snippet: {snippet!r}",
                classification="INSTRUMENT_UNREACHABLE_D31",
            )

    milestone_world_path = repo_root / MILESTONE_WORLD_RS_REL_PATH
    milestone_world_sha, milestone_world_bytes = sha256_file(milestone_world_path)

    rung0_sheet_path = repo_root / RUNG0_SHEET_REL_PATH
    rung0_sheet_sha, rung0_sheet_bytes = sha256_file(rung0_sheet_path)
    rung0_text = rung0_sheet_path.read_text(encoding="utf-8")
    if EXPECTED_STUDENT_TOKENIZER_SHA256 not in rung0_text or "vocab 4096, bos 0 / eos 1 / unk 2" not in rung0_text:
        raise VerificationError(
            "rung0-pilot-release-2026-10-14/sheet.md missing pinned tokenizer contract",
            classification="INSTRUMENT_UNREACHABLE_D31",
        )

    decisions_path = repo_root / DECISIONS_MD_REL_PATH
    decisions_sha, decisions_bytes = sha256_file(decisions_path)
    decisions_text = decisions_path.read_text(encoding="utf-8")
    for snippet in (
        "## D41 — One-time offline donor corpus exception",
        "openai/gpt-oss-120b",
        "distill-d1",
    ):
        if snippet not in decisions_text:
            raise VerificationError(
                f"DECISIONS.md missing required D41 snippet: {snippet!r}",
                classification="INSTRUMENT_UNREACHABLE_D31",
            )

    return {
        "status": "PASS",
        "sheet": {
            "path": str(target_sheet.relative_to(repo_root))
            if target_sheet.is_relative_to(repo_root)
            else str(target_sheet),
            "sha256": sheet_sha,
            "bytes": sheet_bytes,
            "fitness_status": fitness_status,
            "matches_initial_draft_sha256": sheet_sha == EXPECTED_DRAFT_SHEET_SHA256,
            "matches_post_fit_sha256": sheet_sha == EXPECTED_FIT_SHEET_SHA256,
            "expected_draft_sha256": EXPECTED_DRAFT_SHEET_SHA256,
            "expected_fit_sha256": EXPECTED_FIT_SHEET_SHA256,
            "headings_verified": len(REQUIRED_SHEET_HEADINGS),
        },
        "repo_pins": {
            METADATA_RS_REL_PATH: {"sha256": metadata_sha, "bytes": metadata_bytes, "max_context": EXPECTED_MAX_CONTEXT},
            CORPUS_RS_REL_PATH: {"sha256": corpus_sha, "bytes": corpus_bytes, "uort_header_bytes": EXPECTED_UORT_HEADER_BYTES},
            KNOWLEDGE_CORPUS_RS_REL_PATH: {"sha256": knowledge_sha, "bytes": knowledge_bytes},
            MILESTONE_WORLD_RS_REL_PATH: {"sha256": milestone_world_sha, "bytes": milestone_world_bytes},
            RUNG0_SHEET_REL_PATH: {"sha256": rung0_sheet_sha, "bytes": rung0_sheet_bytes},
            DECISIONS_MD_REL_PATH: {"sha256": decisions_sha, "bytes": decisions_bytes},
        },
        "student_tokenizer_pin": {
            "sha256": EXPECTED_STUDENT_TOKENIZER_SHA256,
            "vocabulary": EXPECTED_STUDENT_VOCABULARY,
            "bos_id": EXPECTED_STUDENT_BOS_ID,
            "eos_id": EXPECTED_STUDENT_EOS_ID,
            "unk_id": EXPECTED_STUDENT_UNK_ID,
        },
    }


# ---------------------------------------------------------------------------
# 2. §8 13-Item Registration Checklist & D40/D41 Dispatch Verifier
# ---------------------------------------------------------------------------


def verify_registration_checklist(
    checklist: dict[str, Any],
    *,
    allow_draft_sheet_sha: bool = False,
) -> dict[str, Any]:
    """Verify all 13 mandatory items of §8 before any pilot or production run."""
    if not isinstance(checklist, dict):
        raise VerificationError(
            "registration_checklist must be a JSON object",
            classification="INSTRUMENT_UNREACHABLE_D31",
        )

    # Item 1: Post-FIT sheet SHA-256 and verdict
    sheet_sha = validate_sha256_hex(checklist.get("sheet_sha256"), "checklist.sheet_sha256")
    if not allow_draft_sheet_sha and sheet_sha == EXPECTED_DRAFT_SHEET_SHA256:
        raise VerificationError(
            "checklist.sheet_sha256 equals the un-FIT'd initial draft SHA-256; §8 item 1 requires post-FIT sheet SHA-256",
            classification="INSTRUMENT_UNREACHABLE_D31",
        )
    if checklist.get("sheet_verdict") != "FIT":
        raise VerificationError(
            f"checklist.sheet_verdict must be 'FIT' before dispatch (got {checklist.get('sheet_verdict')!r})",
            classification="INSTRUMENT_UNREACHABLE_D31",
        )

    # Item 2: D41 record reference and owner ratification id
    d41_ref = checklist.get("d41_record_ref")
    ratification_id = checklist.get("owner_ratification_id")
    if not isinstance(d41_ref, str) or "d41" not in d41_ref.lower():
        raise VerificationError(
            f"checklist.d41_record_ref must reference D41 (got {d41_ref!r})",
            classification="INSTRUMENT_UNREACHABLE_D31",
        )
    if not isinstance(ratification_id, str) or not ratification_id.strip():
        raise VerificationError(
            "checklist.owner_ratification_id must be non-empty",
            classification="INSTRUMENT_UNREACHABLE_D31",
        )

    # Item 3: Donor repo, revision commit, per-file and combined weight SHA-256, quantization
    donor = checklist.get("donor")
    if not isinstance(donor, dict):
        raise VerificationError("checklist.donor must be an object", classification="INSTRUMENT_UNREACHABLE_D31")
    if donor.get("repo") != EXPECTED_DONOR_REPO:
        raise VerificationError(
            f"checklist.donor.repo must be {EXPECTED_DONOR_REPO!r} (got {donor.get('repo')!r}); informational fallback rows (e.g. Qwen3-30B-A3B) are not authorized under D41 without a new owner decision",
            classification="INSTRUMENT_UNREACHABLE_D31",
        )
    validate_commit_sha(donor.get("revision"), "checklist.donor.revision")
    if donor.get("quantization") != EXPECTED_DONOR_QUANTIZATION:
        raise VerificationError(
            f"checklist.donor.quantization must be {EXPECTED_DONOR_QUANTIZATION!r}",
            classification="INSTRUMENT_UNREACHABLE_D31",
        )
    if donor.get("license_spdx") != EXPECTED_DONOR_LICENSE_SPDX:
        raise VerificationError(
            f"checklist.donor.license_spdx must be {EXPECTED_DONOR_LICENSE_SPDX!r}",
            classification="INSTRUMENT_UNREACHABLE_D31",
        )
    weight_files = donor.get("weight_files")
    if not isinstance(weight_files, list) or not weight_files:
        raise VerificationError(
            "checklist.donor.weight_files must be a non-empty list",
            classification="INSTRUMENT_UNREACHABLE_D31",
        )
    seen_weight_paths: set[str] = set()
    for idx, wf in enumerate(weight_files):
        if not isinstance(wf, dict):
            raise VerificationError(f"weight_files[{idx}] must be an object", classification="INSTRUMENT_UNREACHABLE_D31")
        wpath = wf.get("path")
        if not isinstance(wpath, str) or not wpath.strip() or wpath in seen_weight_paths:
            raise VerificationError(
                f"weight_files[{idx}].path must be a unique non-empty string",
                classification="INSTRUMENT_UNREACHABLE_D31",
            )
        seen_weight_paths.add(wpath)
        validate_sha256_hex(wf.get("sha256"), f"weight_files[{idx}].sha256")
    validate_sha256_hex(donor.get("combined_weights_sha256"), "checklist.donor.combined_weights_sha256")

    # Item 4: vLLM version, container image digest, driver & CUDA version
    engine = checklist.get("engine")
    if not isinstance(engine, dict):
        raise VerificationError("checklist.engine must be an object", classification="INSTRUMENT_UNREACHABLE_D31")
    if engine.get("name") != "vllm":
        raise VerificationError(f"checklist.engine.name must be 'vllm' (got {engine.get('name')!r})", classification="INSTRUMENT_UNREACHABLE_D31")
    version = engine.get("version")
    if not isinstance(version, str) or not version.strip() or version.strip().lower() == "latest":
        raise VerificationError("checklist.engine.version must be a pinned version string", classification="INSTRUMENT_UNREACHABLE_D31")
    img_digest = engine.get("container_image_digest")
    if not isinstance(img_digest, str) or not img_digest.startswith("sha256:"):
        raise VerificationError("checklist.engine.container_image_digest must start with 'sha256:'", classification="INSTRUMENT_UNREACHABLE_D31")
    validate_sha256_hex(img_digest.removeprefix("sha256:"), "checklist.engine.container_image_digest")
    for k in ("driver_version", "cuda_version"):
        if not isinstance(engine.get(k), str) or not engine.get(k).strip():
            raise VerificationError(f"checklist.engine.{k} must be non-empty", classification="INSTRUMENT_UNREACHABLE_D31")
    if engine.get("runtime_compilation_of_project_code") is not False:
        raise VerificationError("checklist.engine.runtime_compilation_of_project_code must be False", classification="INSTRUMENT_UNREACHABLE_D31")

    # Item 5: Prompt-set SHA-256, slice map, source licences per slice
    prompt_set = checklist.get("prompt_set")
    if not isinstance(prompt_set, dict):
        raise VerificationError("checklist.prompt_set must be an object", classification="INSTRUMENT_UNREACHABLE_D31")
    validate_sha256_hex(prompt_set.get("prompts_sha256"), "checklist.prompt_set.prompts_sha256")
    if not isinstance(prompt_set.get("prompts_path"), str) or not prompt_set["prompts_path"].strip():
        raise VerificationError("checklist.prompt_set.prompts_path must be non-empty", classification="INSTRUMENT_UNREACHABLE_D31")
    slice_map = prompt_set.get("slice_map")
    if not isinstance(slice_map, dict) or set(slice_map.keys()) != set(CANONICAL_SLICES):
        raise VerificationError(
            f"checklist.prompt_set.slice_map must have exact keys {CANONICAL_SLICES}",
            classification="INSTRUMENT_UNREACHABLE_D31",
        )
    for sname, expected_share in EXPECTED_SLICE_SHARES.items():
        sentry = slice_map[sname]
        if not isinstance(sentry, dict):
            raise VerificationError(f"slice_map.{sname} must be an object", classification="INSTRUMENT_UNREACHABLE_D31")
        if abs(float(sentry.get("target_share", -1.0)) - expected_share) > 1e-6:
            raise VerificationError(
                f"slice_map.{sname}.target_share must equal {expected_share} (got {sentry.get('target_share')!r})",
                classification="INSTRUMENT_UNREACHABLE_D31",
            )
        if not isinstance(sentry.get("prompt_count"), int) or sentry["prompt_count"] <= 0:
            raise VerificationError(f"slice_map.{sname}.prompt_count must be a positive int", classification="INSTRUMENT_UNREACHABLE_D31")
    licenses = prompt_set.get("slice_licenses_spdx")
    if not isinstance(licenses, dict) or set(licenses.keys()) != set(CANONICAL_SLICES):
        raise VerificationError("checklist.prompt_set.slice_licenses_spdx must cover all 5 slices", classification="INSTRUMENT_UNREACHABLE_D31")
    for sname, spdx in licenses.items():
        if not isinstance(spdx, str) or not spdx.strip():
            raise VerificationError(f"slice_licenses_spdx.{sname} must be non-empty SPDX string", classification="INSTRUMENT_UNREACHABLE_D31")
    if prompt_set.get("panel_prescreened") is not True or prompt_set.get("authored_from_heldout_panel") is not False:
        raise VerificationError(
            "checklist.prompt_set requires panel_prescreened=True and authored_from_heldout_panel=False",
            classification="INSTRUMENT_UNREACHABLE_D31",
        )

    # Item 6: Sampling parameters
    sampling = checklist.get("sampling")
    if not isinstance(sampling, dict):
        raise VerificationError("checklist.sampling must be an object", classification="INSTRUMENT_UNREACHABLE_D31")
    if (
        abs(float(sampling.get("temperature", -1.0)) - 0.7) > 1e-9
        or abs(float(sampling.get("top_p", -1.0)) - 0.95) > 1e-9
        or sampling.get("top_k") != 20
        or sampling.get("max_tokens") != 512
        or sampling.get("reasoning_effort") != "medium"
        or sampling.get("tuned_on_outputs") is not False
    ):
        raise VerificationError(
            "checklist.sampling must match fixed registration (temp=0.7, top_p=0.95, top_k=20, max_tokens=512, reasoning_effort='medium', tuned_on_outputs=False)",
            classification="INSTRUMENT_UNREACHABLE_D31",
        )
    shard_seeds = sampling.get("shard_seeds")
    if not isinstance(shard_seeds, list) or not shard_seeds or not all(isinstance(x, int) and not isinstance(x, bool) for x in shard_seeds):
        raise VerificationError("checklist.sampling.shard_seeds must be a non-empty list of integers", classification="INSTRUMENT_UNREACHABLE_D31")

    # Item 7: Student tokenizer
    tok = checklist.get("student_tokenizer")
    if not isinstance(tok, dict):
        raise VerificationError("checklist.student_tokenizer must be an object", classification="INSTRUMENT_UNREACHABLE_D31")
    if (
        tok.get("sha256") != EXPECTED_STUDENT_TOKENIZER_SHA256
        or tok.get("vocabulary") != EXPECTED_STUDENT_VOCABULARY
        or tok.get("bos_id") != EXPECTED_STUDENT_BOS_ID
    ):
        raise VerificationError(
            "checklist.student_tokenizer must match pinned d36d3e87… / vocab 4096 / bos_id 0",
            classification="INSTRUMENT_UNREACHABLE_D31",
        )

    # Item 8: Pod declaration and cap check
    pod = checklist.get("pod")
    if not isinstance(pod, dict):
        raise VerificationError("checklist.pod must be an object", classification="INSTRUMENT_UNREACHABLE_D31")
    rate = float(pod.get("rate_usd_per_gpu_hr", -1.0))
    running_pods = pod.get("running_pods_total")
    if (
        not isinstance(pod.get("provider"), str)
        or not pod["provider"].strip()
        or pod.get("gpu_type") != "RTX PRO 6000"
        or pod.get("gpu_count") != 1
        or int(pod.get("vram_gb", 0)) < 96
        or not isinstance(pod.get("lease_id"), str)
        or not pod["lease_id"].strip()
        or rate <= 0.0
        or rate > MAX_POD_RATE_USD_PER_GPU_HR
        or not isinstance(running_pods, int)
        or running_pods < 1
        or running_pods > MAX_RUNNING_PODS
        or pod.get("local_weights_only") is not True
        or pod.get("external_api_used") is not False
    ):
        raise VerificationError(
            "checklist.pod violates hardware/cost/isolation contract (1x RTX PRO 6000 >=96 GB, <= $8/h, <= 4 pods, local weights only)",
            classification="INSTRUMENT_UNREACHABLE_D31",
        )

    # Item 9: Budget & owner-sequence prerequisite & out-of-scope cost rows check
    budget = checklist.get("budget")
    if not isinstance(budget, dict):
        raise VerificationError("checklist.budget must be an object", classification="INSTRUMENT_UNREACHABLE_D31")
    mode = budget.get("mode", "production")
    token_target = budget.get("token_target")
    if mode == "pilot":
        if token_target != PILOT_TOKEN_TARGET:
            raise VerificationError(
                f"pilot checklist.budget.token_target must be {PILOT_TOKEN_TARGET}",
                classification="INSTRUMENT_UNREACHABLE_D31",
            )
    elif mode == "production":
        if not isinstance(token_target, int) or not (PRODUCTION_TOKEN_FLOOR <= token_target <= PRODUCTION_TOKEN_CEILING):
            raise VerificationError(
                f"production checklist.budget.token_target must be in [{PRODUCTION_TOKEN_FLOOR}, {PRODUCTION_TOKEN_CEILING}]",
                classification="INSTRUMENT_UNREACHABLE_D31",
            )
    else:
        raise VerificationError(f"unsupported budget.mode {mode!r}", classification="INSTRUMENT_UNREACHABLE_D31")
    shard_plan = budget.get("shard_plan")
    if not isinstance(shard_plan, list) or len(shard_plan) != len(shard_seeds):
        raise VerificationError(
            "checklist.budget.shard_plan must be a list matching sampling.shard_seeds length",
            classification="INSTRUMENT_UNREACHABLE_D31",
        )
    if shard_plan != sorted(shard_plan):
        raise VerificationError(
            "checklist.budget.shard_plan must be in ascending shard order",
            classification="INSTRUMENT_UNREACHABLE_D31",
        )
    if float(budget.get("max_hours", 0.0)) <= 0.0 or float(budget.get("max_dollars", 0.0)) <= 0.0:
        raise VerificationError("checklist.budget requires max_hours > 0 and max_dollars > 0", classification="INSTRUMENT_UNREACHABLE_D31")
    if budget.get("driver_math_optimization_completed") is not True:
        raise VerificationError(
            "checklist.budget.driver_math_optimization_completed must be True before steps 4–8 dispatch (D41 clause 3 / §7 dependency note)",
            classification="INSTRUMENT_UNREACHABLE_D31",
        )
    if budget.get("student_training_authorized") is not False:
        raise VerificationError(
            "checklist.budget.student_training_authorized must be False (§6 student consumption rows are informational only; student training requires a separate FIT'd sheet)",
            classification="INSTRUMENT_UNREACHABLE_D31",
        )

    # Item 10: Store destination, readback method, MD5/index path, and merkle_mode
    store = checklist.get("store")
    if not isinstance(store, dict):
        raise VerificationError("checklist.store must be an object", classification="INSTRUMENT_UNREACHABLE_D31")
    for k in ("destination_path", "readback_method", "md5_index_path"):
        if not isinstance(store.get(k), str) or not store[k].strip():
            raise VerificationError(f"checklist.store.{k} must be non-empty", classification="INSTRUMENT_UNREACHABLE_D31")
    merkle_mode = store.get("merkle_mode")
    if merkle_mode not in SUPPORTED_MERKLE_MODES:
        raise VerificationError(
            f"checklist.store.merkle_mode must be explicitly bound to one of {SUPPORTED_MERKLE_MODES} (got {merkle_mode!r})",
            classification="INSTRUMENT_UNREACHABLE_D31",
        )

    # Item 11: Gate thresholds G1–G14 & P1–P8
    thresholds = checklist.get("gate_thresholds")
    if not isinstance(thresholds, dict):
        raise VerificationError("checklist.gate_thresholds must be an object", classification="INSTRUMENT_UNREACHABLE_D31")
    expected_thresholds = {
        "g4_max_exact_duplicates": 0,
        "g5_shingle_words": NEAR_DEDUP_SHINGLE_WORDS,
        "g5_jaccard_threshold": NEAR_DEDUP_JACCARD_THRESHOLD,
        "g5_max_residual_rate": NEAR_DEDUP_MAX_RESIDUAL_RATE,
        "g6_panel_ngram_words": PANEL_EXCLUSION_NGRAM_WORDS,
        "g6_short_turn_min_words": PANEL_EXCLUSION_SHORT_MIN_WORDS,
        "g6_short_turn_max_words": PANEL_EXCLUSION_SHORT_MAX_WORDS,
        "g6_max_contaminated_docs": 0,
        "g7_train_overlap_words": TRAIN_OVERLAP_NGRAM_WORDS,
        "g7_max_overlap_docs": 0,
        "g8_code_pass_rate": 1.0,
        "g8_code_max_unevaluated": 0,
        "g9_math_pass_rate": 1.0,
        "g9_math_max_unevaluated": 0,
        "g10_spot_audit_docs_per_slice": SPOT_AUDIT_SAMPLE_PER_SLICE,
        "g10_max_malformed_rate": SPOT_AUDIT_MAX_MALFORMED_RATE,
        "g10_max_refusal_rate": SPOT_AUDIT_MAX_REFUSAL_RATE,
        "g11a_slice_share_tolerance_pp": SLICE_SHARE_TOLERANCE,
        "g11b_max_truncation_rate": TRUNCATION_MAX_RATE,
        "g11c_repetition_ngram_words": REPETITION_NGRAM_WORDS,
        "g11c_repetition_max_count": REPETITION_MAX_OCCURRENCES,
        "g11c_max_doc_rate": REPETITION_MAX_DOC_RATE,
        "g11d_min_tokens": DOC_MIN_TOKENS,
        "g11d_max_tokens": DOC_MAX_TOKENS,
        "g11d_median_min": DOC_MEDIAN_MIN_TOKENS,
        "g11d_median_max": DOC_MEDIAN_MAX_TOKENS,
        "g12_dev_min_eligible_targets": DEV_MIN_ELIGIBLE_TARGETS,
        "p_go_tok_per_s": PILOT_GO_TOK_PER_S,
        "p_stop_tok_per_s": PILOT_STOP_TOK_PER_S,
    }
    for k, exp_v in expected_thresholds.items():
        got_v = thresholds.get(k)
        if got_v is None or (isinstance(exp_v, float) and abs(float(got_v) - exp_v) > 1e-9) or (isinstance(exp_v, int) and got_v != exp_v):
            raise VerificationError(
                f"checklist.gate_thresholds.{k} must equal registered bar {exp_v!r} (got {got_v!r})",
                classification="INSTRUMENT_UNREACHABLE_D31",
            )

    # Item 12: PM dispatch id and D40 release record id
    dispatch = checklist.get("dispatch")
    if not isinstance(dispatch, dict):
        raise VerificationError("checklist.dispatch must be an object", classification="INSTRUMENT_UNREACHABLE_D31")
    for k in ("pm_dispatch_id", "d40_release_record_id"):
        if not isinstance(dispatch.get(k), str) or not dispatch[k].strip():
            raise VerificationError(f"checklist.dispatch.{k} must be non-empty", classification="INSTRUMENT_UNREACHABLE_D31")
    if str(dispatch.get("d40_owner", "")).strip().lower() != "casey":
        raise VerificationError("checklist.dispatch.d40_owner must be 'casey' (D40 §2)", classification="INSTRUMENT_UNREACHABLE_D31")
    if dispatch.get("d40_uor_lab_env_unset") is not True or dispatch.get("d40_typed_by_lab") is not False:
        raise VerificationError(
            "checklist.dispatch requires d40_uor_lab_env_unset=True and d40_typed_by_lab=False (D40 §2)",
            classification="INSTRUMENT_UNREACHABLE_D31",
        )

    # Item 13: Student-side target lineage & hard-targets-only contract
    student_target = checklist.get("student_target")
    if not isinstance(student_target, dict):
        raise VerificationError("checklist.student_target must be an object", classification="INSTRUMENT_UNREACHABLE_D31")
    if (
        student_target.get("lineage_id") != EXPECTED_LINEAGE_ID
        or student_target.get("inherits_mw_balp") is not False
        or not isinstance(student_target.get("separate_training_sheet_path"), str)
        or not student_target["separate_training_sheet_path"].strip()
        or student_target.get("hard_targets_only") is not True
        or student_target.get("logits_or_probs_stored") is not False
        or student_target.get("ranks_opened_by_training") is not False
    ):
        raise VerificationError(
            "checklist.student_target violates §5.1/§5.2 contract (lineage_id='distill-d1', inherits_mw_balp=False, hard_targets_only=True, logits_or_probs_stored=False, ranks_opened_by_training=False)",
            classification="INSTRUMENT_UNREACHABLE_D31",
        )

    return {
        "status": "PASS",
        "items_verified": 13,
        "sheet_sha256": sheet_sha,
        "donor_revision": donor["revision"],
        "combined_weights_sha256": donor["combined_weights_sha256"],
        "prompts_sha256": prompt_set["prompts_sha256"],
        "shard_count": len(shard_seeds),
        "merkle_mode": merkle_mode,
        "mode": mode,
        "token_target": token_target,
    }


# ---------------------------------------------------------------------------
# 3. `uor-r4.donor-corpus/1` 5-Block Receipt & Merkle Root Verifier
# ---------------------------------------------------------------------------


def verify_generation_receipt(
    receipt: dict[str, Any],
    checklist: dict[str, Any] | None = None,
    *,
    is_pilot: bool = False,
) -> dict[str, Any]:
    """Verify `uor-r4.donor-corpus/1` 5 mandatory blocks (`G1`, `G2`, `G3`, `P2`, `P3`)."""
    if not isinstance(receipt, dict):
        raise VerificationError("receipt must be a JSON object", classification="INSTRUMENT_UNREACHABLE_D31", gate="G2")
    if receipt.get("schema") != EXPECTED_RECEIPT_SCHEMA:
        raise VerificationError(
            f"receipt.schema must be {EXPECTED_RECEIPT_SCHEMA!r} (got {receipt.get('schema')!r})",
            classification="INSTRUMENT_UNREACHABLE_D31",
            gate="G2",
        )

    mandatory_blocks = ("donor_pin", "generation_pin", "run_pin", "content_pin", "readback")
    for bname in mandatory_blocks:
        if bname not in receipt:
            raise VerificationError(
                f"receipt missing mandatory block {bname!r}",
                classification="INSTRUMENT_UNREACHABLE_D31",
                gate="G2",
            )
        ensure_no_null_or_empty(receipt[bname], f"receipt.{bname}", gate="G2")

    # Block 1: Donor pin (G1)
    donor_pin = receipt["donor_pin"]
    for req_key in ("repo", "revision", "weights_sha256", "quantization", "total_params", "active_params", "license_spdx"):
        if req_key not in donor_pin:
            raise VerificationError(f"donor_pin missing {req_key!r}", classification="INSTRUMENT_UNREACHABLE_D31", gate="G2")
    if donor_pin["repo"] != EXPECTED_DONOR_REPO:
        raise VerificationError(f"donor_pin.repo mismatch: {donor_pin['repo']!r}", classification="INSTRUMENT_UNREACHABLE_D31", gate="G1")
    validate_commit_sha(donor_pin["revision"], "donor_pin.revision", gate="G1")
    if donor_pin["quantization"] != EXPECTED_DONOR_QUANTIZATION:
        raise VerificationError(f"donor_pin.quantization mismatch: {donor_pin['quantization']!r}", classification="INSTRUMENT_UNREACHABLE_D31", gate="G1")
    if donor_pin["license_spdx"] != EXPECTED_DONOR_LICENSE_SPDX:
        raise VerificationError(f"donor_pin.license_spdx mismatch: {donor_pin['license_spdx']!r}", classification="INSTRUMENT_UNREACHABLE_D31", gate="G1")
    weights_sha = donor_pin["weights_sha256"]
    if not isinstance(weights_sha, dict) or "files" not in weights_sha or "combined_sha256" not in weights_sha:
        raise VerificationError("donor_pin.weights_sha256 requires 'files' and 'combined_sha256'", classification="INSTRUMENT_UNREACHABLE_D31", gate="G2")
    validate_sha256_hex(weights_sha["combined_sha256"], "donor_pin.weights_sha256.combined_sha256", gate="G1")
    for idx, wf in enumerate(weights_sha["files"]):
        validate_sha256_hex(wf.get("sha256"), f"donor_pin.weights_sha256.files[{idx}].sha256", gate="G1")

    if checklist is not None:
        cdonor = checklist["donor"]
        if donor_pin["revision"] != cdonor["revision"]:
            raise VerificationError(
                f"G1 donor revision mismatch: receipt {donor_pin['revision']} != checklist {cdonor['revision']}",
                classification="INSTRUMENT_UNREACHABLE_D31",
                gate="G1",
            )
        if weights_sha["combined_sha256"] != cdonor["combined_weights_sha256"]:
            raise VerificationError(
                "G1 combined_weights_sha256 mismatch against registration checklist",
                classification="INSTRUMENT_UNREACHABLE_D31",
                gate="G1",
            )
        receipt_wmap = {wf["path"]: wf["sha256"] for wf in weights_sha["files"]}
        checklist_wmap = {wf["path"]: wf["sha256"] for wf in cdonor["weight_files"]}
        if receipt_wmap != checklist_wmap:
            raise VerificationError(
                "G1 per-file weight SHA-256 map differs from registration checklist",
                classification="INSTRUMENT_UNREACHABLE_D31",
                gate="G1",
            )

    # Block 2: Generation pin
    gen_pin = receipt["generation_pin"]
    for req_key in ("prompt_set_sha256", "slice_map", "temperature", "top_p", "top_k", "max_tokens", "reasoning_effort", "shard_seeds", "shard_plan"):
        if req_key not in gen_pin:
            raise VerificationError(f"generation_pin missing {req_key!r}", classification="INSTRUMENT_UNREACHABLE_D31", gate="G2")
    validate_sha256_hex(gen_pin["prompt_set_sha256"], "generation_pin.prompt_set_sha256", gate="G2")
    if (
        abs(float(gen_pin["temperature"]) - 0.7) > 1e-9
        or abs(float(gen_pin["top_p"]) - 0.95) > 1e-9
        or gen_pin["top_k"] != 20
        or gen_pin["max_tokens"] != 512
        or gen_pin["reasoning_effort"] != "medium"
    ):
        raise VerificationError("generation_pin sampling parameters diverge from fixed §2 spec", classification="INSTRUMENT_UNREACHABLE_D31", gate="G2")
    if checklist is not None:
        if gen_pin["prompt_set_sha256"] != checklist["prompt_set"]["prompts_sha256"]:
            raise VerificationError("generation_pin.prompt_set_sha256 mismatch against checklist", classification="INSTRUMENT_UNREACHABLE_D31", gate="G2")

    # Block 3: Run pin
    run_pin = receipt["run_pin"]
    for req_key in (
        "pod_id",
        "provider",
        "gpu_type",
        "gpu_count",
        "lease_id",
        "rate_usd_per_gpu_hr",
        "utc_start",
        "utc_end",
        "engine_version",
        "image_digest",
        "driver_version",
        "cuda_version",
    ):
        if req_key not in run_pin:
            raise VerificationError(f"run_pin missing {req_key!r}", classification="INSTRUMENT_UNREACHABLE_D31", gate="G2")
    if (
        run_pin["gpu_type"] != "RTX PRO 6000"
        or run_pin["gpu_count"] != 1
        or float(run_pin["rate_usd_per_gpu_hr"]) <= 0.0
        or float(run_pin["rate_usd_per_gpu_hr"]) > MAX_POD_RATE_USD_PER_GPU_HR
    ):
        raise VerificationError("run_pin hardware or rate violates 1x RTX PRO 6000 <= $8/h contract", classification="INSTRUMENT_UNREACHABLE_D31", gate="G2")
    if not str(run_pin["image_digest"]).startswith("sha256:"):
        raise VerificationError("run_pin.image_digest must start with 'sha256:'", classification="INSTRUMENT_UNREACHABLE_D31", gate="G2")
    validate_sha256_hex(str(run_pin["image_digest"]).removeprefix("sha256:"), "run_pin.image_digest", gate="G2")

    # Block 4: Content pin (explicit merkle_mode + ascending shard order)
    content_pin = receipt["content_pin"]
    for req_key in ("shards", "document_count_total", "token_total", "merkle_mode", "merkle_root"):
        if req_key not in content_pin:
            raise VerificationError(f"content_pin missing mandatory field {req_key!r}", classification="INSTRUMENT_UNREACHABLE_D31", gate="G2")
    merkle_mode = content_pin["merkle_mode"]
    if merkle_mode not in SUPPORTED_MERKLE_MODES:
        raise VerificationError(
            f"content_pin.merkle_mode must be one of {SUPPORTED_MERKLE_MODES} (got {merkle_mode!r})",
            classification="INSTRUMENT_UNREACHABLE_D31",
            gate="G2",
        )
    if checklist is not None and merkle_mode != checklist["store"]["merkle_mode"]:
        raise VerificationError(
            f"content_pin.merkle_mode ({merkle_mode!r}) != checklist.store.merkle_mode ({checklist['store']['merkle_mode']!r})",
            classification="INSTRUMENT_UNREACHABLE_D31",
            gate="G2",
        )
    shards = content_pin["shards"]
    if not isinstance(shards, list) or not shards:
        raise VerificationError("content_pin.shards must be a non-empty list", classification="INSTRUMENT_UNREACHABLE_D31", gate="G2")
    doc_sum = 0
    tok_sum = 0
    shard_hashes: list[str] = []
    shard_ids: list[str] = []
    shard_paths: list[str] = []
    for idx, sh in enumerate(shards):
        for sk in ("shard_id", "path", "sha256", "document_count", "student_token_count"):
            if sk not in sh:
                raise VerificationError(f"content_pin.shards[{idx}] missing {sk!r}", classification="INSTRUMENT_UNREACHABLE_D31", gate="G2")
        sh_hash = validate_sha256_hex(sh["sha256"], f"content_pin.shards[{idx}].sha256", gate="G2")
        if int(sh["document_count"]) <= 0 or int(sh["student_token_count"]) <= 0:
            raise VerificationError(f"content_pin.shards[{idx}] counts must be positive", classification="INSTRUMENT_UNREACHABLE_D31", gate="G2")
        doc_sum += int(sh["document_count"])
        tok_sum += int(sh["student_token_count"])
        shard_hashes.append(sh_hash)
        shard_ids.append(str(sh["shard_id"]))
        shard_paths.append(str(sh["path"]))

    if shard_ids != sorted(shard_ids) or len(set(shard_ids)) != len(shard_ids) or shard_paths != sorted(shard_paths):
        raise VerificationError(
            "content_pin.shards must be in strictly ascending order by shard_id and relative path",
            classification="INSTRUMENT_UNREACHABLE_D31",
            gate="G2",
        )
    if doc_sum != int(content_pin["document_count_total"]):
        raise VerificationError(
            f"content_pin.document_count_total ({content_pin['document_count_total']}) != sum of shard document counts ({doc_sum})",
            classification="INSTRUMENT_UNREACHABLE_D31",
            gate="G2",
        )
    if tok_sum != int(content_pin["token_total"]):
        raise VerificationError(
            f"content_pin.token_total ({content_pin['token_total']}) != sum of shard student_token_count ({tok_sum})",
            classification="INSTRUMENT_UNREACHABLE_D31",
            gate="G2",
        )
    if not is_pilot and not (PRODUCTION_TOKEN_FLOOR <= tok_sum <= PRODUCTION_TOKEN_CEILING):
        raise VerificationError(
            f"production content_pin.token_total ({tok_sum}) outside [{PRODUCTION_TOKEN_FLOOR}, {PRODUCTION_TOKEN_CEILING}]",
            classification="INSTRUMENT_UNREACHABLE_D31",
            gate="G2",
        )

    recorded_merkle = validate_sha256_hex(content_pin["merkle_root"], "content_pin.merkle_root", gate="G2")
    expected_merkle = compute_shard_merkle_root(shard_hashes, mode=merkle_mode)
    if recorded_merkle != expected_merkle:
        raise VerificationError(
            f"content_pin.merkle_root ({recorded_merkle}) != recomputed Merkle root ({expected_merkle})",
            classification="INSTRUMENT_UNREACHABLE_D31",
            gate="G3",
        )

    # Block 5: Readback (G3)
    readback = receipt["readback"]
    for req_key in (
        "destination_store_path",
        "shards_readback",
        "recomputed_merkle_root",
        "md5_index_record",
        "verification_timestamp_utc",
        "operator",
    ):
        if req_key not in readback:
            raise VerificationError(f"readback missing {req_key!r}", classification="INSTRUMENT_UNREACHABLE_D31", gate="G2")
    rb_shards = readback["shards_readback"]
    if not isinstance(rb_shards, list) or len(rb_shards) != len(shards):
        raise VerificationError("readback.shards_readback length mismatch against content_pin.shards", classification="INSTRUMENT_UNREACHABLE_D31", gate="G3")
    rb_hashes: list[str] = []
    for idx, (orig, rb) in enumerate(zip(shards, rb_shards)):
        rb_hash = validate_sha256_hex(rb.get("sha256"), f"readback.shards_readback[{idx}].sha256", gate="G3")
        if rb.get("shard_id") != orig["shard_id"] or rb_hash != orig["sha256"]:
            raise VerificationError(
                f"G3 shard readback mismatch at index {idx}: pod {orig['sha256']} != store readback {rb_hash}",
                classification="INSTRUMENT_UNREACHABLE_D31",
                gate="G3",
            )
        rb_hashes.append(rb_hash)

    rb_merkle = validate_sha256_hex(readback["recomputed_merkle_root"], "readback.recomputed_merkle_root", gate="G3")
    if rb_merkle != expected_merkle or rb_merkle != recorded_merkle:
        raise VerificationError(
            f"G3 readback.recomputed_merkle_root ({rb_merkle}) != content_pin.merkle_root ({recorded_merkle})",
            classification="INSTRUMENT_UNREACHABLE_D31",
            gate="G3",
        )
    md5_rec = readback["md5_index_record"]
    if not isinstance(md5_rec, dict):
        raise VerificationError("readback.md5_index_record must be an object", classification="INSTRUMENT_UNREACHABLE_D31", gate="G3")
    validate_hex(md5_rec.get("md5"), 32, "readback.md5_index_record.md5", gate="G3")
    validate_sha256_hex(md5_rec.get("sha256"), "readback.md5_index_record.sha256", gate="G3")

    return {
        "status": "PASS",
        "schema": EXPECTED_RECEIPT_SCHEMA,
        "g1_donor_bind": "PASS",
        "g2_receipt_completeness": "PASS",
        "g3_readback_merkle": "PASS",
        "shard_count": len(shards),
        "document_count_total": doc_sum,
        "token_total": tok_sum,
        "merkle_mode": merkle_mode,
        "merkle_root": expected_merkle,
    }


def verify_repair_vs_regeneration(
    original_receipt: dict[str, Any],
    updated_receipt: dict[str, Any],
) -> dict[str, Any]:
    """Enforce §4.3 Repair vs Regeneration rule between two receipts."""
    verify_generation_receipt(original_receipt, is_pilot=True)
    verify_generation_receipt(updated_receipt, is_pilot=True)

    orig_donor = original_receipt["donor_pin"]
    upd_donor = updated_receipt["donor_pin"]
    for k in ("repo", "revision", "weights_sha256", "quantization", "license_spdx"):
        if orig_donor.get(k) != upd_donor.get(k):
            raise VerificationError(
                f"§4.3 violation: donor_pin.{k} changed between receipts; regeneration requires a new owner decision (D41 clause 6)",
                classification="REGENERATION_FORBIDDEN_REQUIRES_NEW_OWNER_DECISION_D41",
                gate="G1",
                d41_pass_consumed=True,
            )

    orig_gen = original_receipt["generation_pin"]
    upd_gen = updated_receipt["generation_pin"]
    for k in (
        "prompt_set_sha256",
        "slice_map",
        "temperature",
        "top_p",
        "top_k",
        "max_tokens",
        "reasoning_effort",
        "shard_seeds",
        "shard_plan",
    ):
        if orig_gen.get(k) != upd_gen.get(k):
            raise VerificationError(
                f"§4.3 violation: generation_pin.{k} changed between receipts; regeneration requires a new owner decision (D41 clause 6)",
                classification="REGENERATION_FORBIDDEN_REQUIRES_NEW_OWNER_DECISION_D41",
                gate="G2",
                d41_pass_consumed=True,
            )

    if original_receipt["content_pin"]["merkle_mode"] != updated_receipt["content_pin"]["merkle_mode"]:
        raise VerificationError(
            "§4.3 violation: content_pin.merkle_mode changed between receipts",
            classification="REGENERATION_FORBIDDEN_REQUIRES_NEW_OWNER_DECISION_D41",
            gate="G2",
            d41_pass_consumed=True,
        )

    return {
        "status": "PASS",
        "classification": "PERMITTED_SHARD_REPAIR",
        "consumes_new_pass": False,
        "updated_merkle_root": updated_receipt["content_pin"]["merkle_root"],
    }


# ---------------------------------------------------------------------------
# 4. Raw Text, Contamination, Shingle Jaccard & UORT v1 Binary Helpers
# ---------------------------------------------------------------------------


def normalize_words(text: str, *, mode: str = "milestone_world") -> list[str]:
    """Deterministic word splitter for G5/G6/G7/G11c (§2.1).

    - `milestone_world` (default): exact port of `crates/uor-r4-training/src/milestone_world.rs::words`
      (lowercased, non-alphanumeric split with internal apostrophes preserved, outer apostrophes trimmed).
    - `ascii_alnum`: lowercase ASCII `[a-z0-9]+` regex tokenization.
    """
    if mode == "ascii_alnum":
        return re.findall(r"[a-z0-9]+", text.lower())
    normalized = text.replace("\u2018", "'").replace("\u2019", "'")
    parts: list[str] = []
    cur: list[str] = []
    for ch in normalized:
        if ch.isalnum() or ch == "'":
            cur.append(ch)
        else:
            if cur:
                word = "".join(cur).strip("'").lower()
                if word:
                    parts.append(word)
                cur = []
    if cur:
        word = "".join(cur).strip("'").lower()
        if word:
            parts.append(word)
    return parts


def check_panel_exclusion_text(
    documents: Sequence[dict[str, Any]],
    panel_requests: Sequence[dict[str, Any]],
) -> dict[str, Any]:
    """Exact port of `PanelExclusion` (`crates/uor-r4-training/src/knowledge_corpus.rs`).

    Matches:
    - Any 8-word window equal to an 8-word window of a panel user turn (`panel_8gram`), or
    - Whole-sequence containment of any 4–7-word panel user turn (`panel_short_request`).
    Turns with < 4 words are counted as `uncovered_turns` and not matched.
    """
    by_len: dict[int, dict[tuple[str, ...], str]] = {}
    ngram_turns = 0
    short_turns = 0
    uncovered_turns = 0
    total_turns = 0

    for req in panel_requests:
        req_id = str(req["id"])
        for turn in req.get("user_turns", []):
            total_turns += 1
            w = normalize_words(str(turn))
            if len(w) >= PANEL_EXCLUSION_NGRAM_WORDS:
                ngram_turns += 1
                table = by_len.setdefault(PANEL_EXCLUSION_NGRAM_WORDS, {})
                for i in range(len(w) - PANEL_EXCLUSION_NGRAM_WORDS + 1):
                    seq = tuple(w[i : i + PANEL_EXCLUSION_NGRAM_WORDS])
                    table.setdefault(seq, req_id)
            elif len(w) >= PANEL_EXCLUSION_SHORT_MIN_WORDS:
                short_turns += 1
                table = by_len.setdefault(len(w), {})
                table.setdefault(tuple(w), req_id)
            else:
                uncovered_turns += 1

    hits: list[dict[str, Any]] = []
    for doc in documents:
        doc_id = str(doc.get("id", ""))
        w = normalize_words(str(doc.get("text", "")))
        matched: dict[str, Any] | None = None
        for seq_len in sorted(by_len.keys()):
            if len(w) < seq_len:
                continue
            table = by_len[seq_len]
            for i in range(len(w) - seq_len + 1):
                win = tuple(w[i : i + seq_len])
                if win in table:
                    matched = {
                        "doc_id": doc_id,
                        "kind": "panel_8gram" if seq_len == PANEL_EXCLUSION_NGRAM_WORDS else "panel_short_request",
                        "words": " ".join(win),
                        "request_id": table[win],
                    }
                    break
            if matched is not None:
                break
        if matched is not None:
            hits.append(matched)

    return {
        "clean": len(hits) == 0,
        "contaminated_count": len(hits),
        "hits": hits,
        "panel_stats": {
            "requests": len(panel_requests),
            "turns": total_turns,
            "ngram_turns": ngram_turns,
            "short_turns": short_turns,
            "uncovered_turns": uncovered_turns,
        },
    }


def check_5word_jaccard_clusters(
    documents: Sequence[dict[str, Any]],
    *,
    threshold: float = NEAR_DEDUP_JACCARD_THRESHOLD,
) -> dict[str, Any]:
    """Compute exact 5-word shingle Jaccard similarity and residual near-duplicate rate (`G5`)."""
    if not documents:
        return {"residual_near_duplicate_rate": 0.0, "duplicate_docs": 0, "total_docs": 0, "pass": True}

    shingles: list[set[tuple[str, ...]]] = []
    for doc in documents:
        w = normalize_words(str(doc.get("text", "")))
        if len(w) < NEAR_DEDUP_SHINGLE_WORDS:
            shingles.append({tuple(w)} if w else set())
        else:
            shingles.append(
                {tuple(w[i : i + NEAR_DEDUP_SHINGLE_WORDS]) for i in range(len(w) - NEAR_DEDUP_SHINGLE_WORDS + 1)}
            )

    parent = list(range(len(documents)))

    def find(x: int) -> int:
        while parent[x] != x:
            parent[x] = parent[parent[x]]
            x = parent[x]
        return x

    def union(a: int, b: int) -> None:
        ra, rb = find(a), find(b)
        if ra != rb:
            if ra < rb:
                parent[rb] = ra
            else:
                parent[ra] = rb

    near_dup_pairs: list[tuple[int, int, float]] = []
    for i in range(len(documents)):
        si = shingles[i]
        if not si:
            continue
        for j in range(i + 1, len(documents)):
            sj = shingles[j]
            if not sj:
                continue
            union_len = len(si | sj)
            if union_len == 0:
                continue
            jacc = len(si & sj) / union_len
            if jacc >= threshold - 1e-12:
                union(i, j)
                near_dup_pairs.append((i, j, jacc))

    clusters = len({find(i) for i in range(len(documents))})
    duplicate_docs = len(documents) - clusters
    rate = duplicate_docs / len(documents)
    return {
        "total_docs": len(documents),
        "clusters": clusters,
        "duplicate_docs": duplicate_docs,
        "residual_near_duplicate_rate": rate,
        "near_dup_pairs": len(near_dup_pairs),
        "pass": rate < NEAR_DEDUP_MAX_RESIDUAL_RATE,
    }


def check_12word_train_overlap(
    documents: Sequence[dict[str, Any]],
    mw_balp_train_texts: Sequence[str],
) -> dict[str, Any]:
    """Check that 0 kept documents share a 12-word sequence with `mw-balp` TRAIN (`G7`)."""
    train_12grams: set[tuple[str, ...]] = set()
    for txt in mw_balp_train_texts:
        w = normalize_words(txt)
        for i in range(len(w) - TRAIN_OVERLAP_NGRAM_WORDS + 1):
            train_12grams.add(tuple(w[i : i + TRAIN_OVERLAP_NGRAM_WORDS]))

    overlaps: list[dict[str, Any]] = []
    for doc in documents:
        w = normalize_words(str(doc.get("text", "")))
        for i in range(len(w) - TRAIN_OVERLAP_NGRAM_WORDS + 1):
            seq = tuple(w[i : i + TRAIN_OVERLAP_NGRAM_WORDS])
            if seq in train_12grams:
                overlaps.append({"doc_id": doc.get("id"), "words": " ".join(seq)})
                break

    return {
        "overlap_count": len(overlaps),
        "overlaps": overlaps,
        "pass": len(overlaps) == 0,
    }


def check_8gram_repetition(documents: Sequence[dict[str, Any]]) -> dict[str, Any]:
    """Check `G11c`: < 2.0% of kept documents have an 8-word n-gram repeated > 3 times."""
    if not documents:
        return {"flagged_docs": 0, "total_docs": 0, "repeated_8gram_doc_rate": 0.0, "pass": True}

    flagged = 0
    for doc in documents:
        w = normalize_words(str(doc.get("text", "")))
        counts: dict[tuple[str, ...], int] = {}
        doc_flagged = False
        for i in range(len(w) - REPETITION_NGRAM_WORDS + 1):
            ng = tuple(w[i : i + REPETITION_NGRAM_WORDS])
            c = counts.get(ng, 0) + 1
            counts[ng] = c
            if c > REPETITION_MAX_OCCURRENCES:
                doc_flagged = True
                break
        if doc_flagged:
            flagged += 1

    rate = flagged / len(documents)
    return {
        "flagged_docs": flagged,
        "total_docs": len(documents),
        "repeated_8gram_doc_rate": rate,
        "pass": rate < REPETITION_MAX_DOC_RATE,
    }


def build_uort_bytes(tokens: Sequence[int], vocabulary: int = EXPECTED_STUDENT_VOCABULARY) -> bytes:
    """Build canonical 64-byte UORT v1 header + little-endian u16 payload."""
    header = bytearray(64)
    header[0:4] = b"UORT"
    header[4:8] = struct.pack("<I", 1)
    header[8:16] = struct.pack("<Q", len(tokens))
    header[16:20] = struct.pack("<I", vocabulary)
    payload = b"".join(struct.pack("<H", int(t)) for t in tokens)
    return bytes(header) + payload


def verify_uort_and_mask_bytes(
    uort_bytes: bytes,
    mask_bytes: bytes,
    *,
    split: str,
    expected_payload_tokens: int,
    expected_eligible_targets: int,
    vocabulary: int = EXPECTED_STUDENT_VOCABULARY,
    bos_id: int = EXPECTED_STUDENT_BOS_ID,
    context: int = EXPECTED_OPERATING_CONTEXT,
    rows: int = EXPECTED_BATCH_ROWS,
) -> dict[str, Any]:
    """Exact Python port of `PreparedCorpus::from_bytes` and `batch(0, context, rows)` (`G12`, `G13`)."""
    split_map = {"train": 0, "dev": 1, "fresh": 2}
    if split not in split_map:
        raise VerificationError(
            f"invalid split {split!r}; expected one of ('train', 'dev', 'fresh')",
            classification="INSTRUMENT_UNREACHABLE_D31",
            gate="G12",
        )
    if len(uort_bytes) < EXPECTED_UORT_HEADER_BYTES or uort_bytes[:4] != b"UORT":
        raise VerificationError(
            "requires canonical UORT 64-byte header, no raw-u16 fallback",
            classification="INSTRUMENT_UNREACHABLE_D31",
            gate="G12",
        )
    version = struct.unpack("<I", uort_bytes[4:8])[0]
    count = struct.unpack("<Q", uort_bytes[8:16])[0]
    header_vocab = struct.unpack("<I", uort_bytes[16:20])[0]
    if version != 1 or header_vocab != vocabulary or any(b != 0 for b in uort_bytes[20:64]):
        raise VerificationError(
            "UORT version/vocabulary/reserved-header mismatch",
            classification="INSTRUMENT_UNREACHABLE_D31",
            gate="G12",
        )
    physical = 64 + count * 2
    if physical != len(uort_bytes) or count != expected_payload_tokens or count == 0:
        raise VerificationError(
            "UORT payload length/count differs from pinned inventory",
            classification="INSTRUMENT_UNREACHABLE_D31",
            gate="G12",
        )
    if len(mask_bytes) != count or any(m > 1 for m in mask_bytes):
        raise VerificationError(
            "mask must be binary and exactly UORT payload length",
            classification="INSTRUMENT_UNREACHABLE_D31",
            gate="G12",
        )
    tokens = [
        struct.unpack("<H", uort_bytes[64 + 2 * i : 64 + 2 * (i + 1)])[0]
        for i in range(count)
    ]
    if bos_id >= vocabulary or tokens[0] != bos_id or any(t >= vocabulary for t in tokens):
        raise VerificationError(
            "corpus must begin at pinned BOS and contain only valid token IDs",
            classification="INSTRUMENT_UNREACHABLE_D31",
            gate="G12",
        )
    documents = [p for p, t in enumerate(tokens) if t == bos_id]
    if len(documents) >= (1 << 62):
        raise VerificationError(
            "document inventory exceeds split-prefixed identity namespace",
            classification="INSTRUMENT_UNREACHABLE_D31",
            gate="G12",
        )
    eligible = [
        p
        for p, (t, m) in enumerate(zip(tokens, mask_bytes))
        if p > 0 and t != bos_id and m == 1
    ]
    if not eligible or len(eligible) != expected_eligible_targets:
        raise VerificationError(
            "eligible response targets differ from positive expected inventory",
            classification="INSTRUMENT_UNREACHABLE_D31",
            gate="G12",
        )

    doc_non_bos_lengths: list[int] = []
    for idx, start_pos in enumerate(documents):
        end_pos = documents[idx + 1] if idx + 1 < len(documents) else len(tokens)
        non_bos_len = end_pos - start_pos - 1
        doc_non_bos_lengths.append(non_bos_len)

    # Execute `batch(0, context, rows)` (G13 / P6)
    if not (1 <= context <= EXPECTED_MAX_CONTEXT) or not (1 <= rows <= 64):
        raise VerificationError(
            "corpus batches require context 1..243 and rows 1..64",
            classification="INSTRUMENT_UNREACHABLE_D31",
            gate="G13",
        )
    if len(eligible) < rows:
        raise VerificationError(
            f"G13/P6 requires at least {rows} eligible targets to form a 64-row batch (got {len(eligible)})",
            classification="INSTRUMENT_UNREACHABLE_D31",
            gate="G13",
        )

    windows_len: list[int] = []
    target_positions: list[int] = []
    targets: list[int] = []
    split_prefix = split_map[split] << 62
    for pos in eligible[:rows]:
        doc_idx = bisect.bisect_right(documents, pos) - 1
        if doc_idx < 0:
            raise VerificationError("target has no document", classification="INSTRUMENT_UNREACHABLE_D31", gate="G13")
        start = max(pos - context, documents[doc_idx])
        if start >= pos:
            raise VerificationError(
                "target has no in-document predecessor",
                classification="INSTRUMENT_UNREACHABLE_D31",
                gate="G13",
            )
        win_len = pos - start
        _record_id = split_prefix | doc_idx
        windows_len.append(win_len)
        target_positions.append(pos)
        targets.append(tokens[pos])

    return {
        "status": "PASS",
        "tokens_sha256": sha256_bytes(uort_bytes),
        "mask_sha256": sha256_bytes(mask_bytes),
        "payload_tokens": count,
        "mask_bytes": len(mask_bytes),
        "mask_selected_positions": sum(mask_bytes),
        "eligible_targets": len(eligible),
        "documents": len(documents),
        "doc_non_bos_lengths": doc_non_bos_lengths,
        "window_policy": EXPECTED_WINDOW_POLICY,
        "identity_policy": EXPECTED_IDENTITY_POLICY,
        "batch_0_c192_r64": {
            "windows_returned": len(windows_len),
            "min_window_len": min(windows_len),
            "max_window_len": max(windows_len),
            "first_target_pos": target_positions[0],
            "last_target_pos": target_positions[-1],
            "next_target": rows,
        },
    }


# ---------------------------------------------------------------------------
# 5. Quality Gates G4–G11 Verifier (Shared by Pilot P7 and Production §3)
# ---------------------------------------------------------------------------


def verify_quality_gates_g4_to_g11(
    gates: dict[str, Any],
    *,
    is_pilot: bool = False,
) -> dict[str, Any]:
    """Verify quality gates `G4` through `G11d`.

    Distinguishes:
    - Missing/unevaluated gate or sandbox violation -> `INSTRUMENT_UNREACHABLE_D31` (`d41_pass_consumed=False`)
    - Quality threshold failure -> `CORPUS_REJECTED_CONSUMES_D41_PASS` (or `PILOT_REJECTED_CORPUS_DEFECT_CONSUMES_D41_PASS`, `d41_pass_consumed=True`)
    """
    defect_class = (
        "PILOT_REJECTED_CORPUS_DEFECT_CONSUMES_D41_PASS"
        if is_pilot
        else "CORPUS_REJECTED_CONSUMES_D41_PASS"
    )
    if not isinstance(gates, dict):
        raise VerificationError("quality_gates must be an object", classification="INSTRUMENT_UNREACHABLE_D31")

    for req_gate in ("g4_exact_dedup", "g5_near_dedup", "g6_panel_contamination", "g7_train_overlap", "g8_code_slice", "g9_math_slice", "g10_spot_audit", "g11_distribution"):
        if req_gate not in gates or not isinstance(gates[req_gate], dict):
            raise VerificationError(
                f"quality_gates missing {req_gate!r} (gate unreachable under D31)",
                classification="INSTRUMENT_UNREACHABLE_D31",
                gate=req_gate.split("_")[0].upper(),
            )
        if gates[req_gate].get("evaluated") is not True:
            raise VerificationError(
                f"{req_gate}.evaluated is not True (gate unreachable under D31)",
                classification="INSTRUMENT_UNREACHABLE_D31",
                gate=req_gate.split("_")[0].upper(),
            )

    # G4: Exact dedup
    g4 = gates["g4_exact_dedup"]
    if int(g4.get("kept_documents", 0)) <= 0:
        raise VerificationError("G4 kept_documents must be positive", classification="INSTRUMENT_UNREACHABLE_D31", gate="G4")
    if int(g4.get("duplicate_document_sha256_count", -1)) != 0:
        raise VerificationError(
            f"G4 failed: duplicate_document_sha256_count = {g4.get('duplicate_document_sha256_count')} != 0",
            classification=defect_class,
            gate="G4",
            d41_pass_consumed=True,
        )

    # G5: Near dedup (< 3.0%, 5-word shingles, Jaccard >= 0.8)
    g5 = gates["g5_near_dedup"]
    if g5.get("shingle_words") != NEAR_DEDUP_SHINGLE_WORDS or abs(float(g5.get("jaccard_threshold", 0.0)) - NEAR_DEDUP_JACCARD_THRESHOLD) > 1e-9:
        raise VerificationError(
            "G5 instrument config mismatch: requires 5-word shingles and Jaccard >= 0.80",
            classification="INSTRUMENT_UNREACHABLE_D31",
            gate="G5",
        )
    near_dup_rate = float(g5.get("residual_near_duplicate_rate", 1.0))
    if near_dup_rate < 0.0 or near_dup_rate >= NEAR_DEDUP_MAX_RESIDUAL_RATE:
        raise VerificationError(
            f"G5 failed: residual_near_duplicate_rate = {near_dup_rate:.6f} >= {NEAR_DEDUP_MAX_RESIDUAL_RATE}",
            classification=defect_class,
            gate="G5",
            d41_pass_consumed=True,
        )

    # G6: Panel contamination (0 matches against heldout-200-a, heldout-200-b, sealed panels)
    g6 = gates["g6_panel_contamination"]
    panels_checked = g6.get("panels_checked", [])
    if not isinstance(panels_checked, list) or "heldout-200-a.json" not in panels_checked or "heldout-200-b.json" not in panels_checked:
        raise VerificationError(
            "G6 must check heldout-200-a.json and heldout-200-b.json (plus sealed panels)",
            classification="INSTRUMENT_UNREACHABLE_D31",
            gate="G6",
        )
    if (
        g6.get("ngram_window_words") != PANEL_EXCLUSION_NGRAM_WORDS
        or g6.get("short_turn_min_words") != PANEL_EXCLUSION_SHORT_MIN_WORDS
        or g6.get("short_turn_max_words") != PANEL_EXCLUSION_SHORT_MAX_WORDS
    ):
        raise VerificationError(
            "G6 rule mismatch: requires 8-word window and 4–7-word full-turn PanelExclusion rule",
            classification="INSTRUMENT_UNREACHABLE_D31",
            gate="G6",
        )
    if int(g6.get("matched_kept_documents", -1)) != 0:
        raise VerificationError(
            f"G6 failed: matched_kept_documents = {g6.get('matched_kept_documents')} != 0",
            classification=defect_class,
            gate="G6",
            d41_pass_consumed=True,
        )

    # G7: Train overlap (0 kept documents share a 12-word sequence with mw-balp TRAIN)
    g7 = gates["g7_train_overlap"]
    if g7.get("ngram_words") != TRAIN_OVERLAP_NGRAM_WORDS or g7.get("reference_corpus") != "mw-balp/train":
        raise VerificationError(
            "G7 rule mismatch: requires 12-word sequence check against mw-balp/train",
            classification="INSTRUMENT_UNREACHABLE_D31",
            gate="G7",
        )
    if int(g7.get("shared_12word_kept_documents", -1)) != 0:
        raise VerificationError(
            f"G7 failed: shared_12word_kept_documents = {g7.get('shared_12word_kept_documents')} != 0",
            classification=defect_class,
            gate="G7",
            d41_pass_consumed=True,
        )

    # Determine dropped slices (if a slice failed its quality gate and was dropped per §1.2)
    g10 = gates["g10_spot_audit"]
    dropped_slices_raw = g10.get("dropped_slices", [])
    if not isinstance(dropped_slices_raw, list) or any(s not in CANONICAL_SLICES for s in dropped_slices_raw):
        raise VerificationError("G10 dropped_slices must be a subset of canonical slices", classification="INSTRUMENT_UNREACHABLE_D31", gate="G10")
    dropped_slices = set(dropped_slices_raw)
    kept_slices = [s for s in CANONICAL_SLICES if s not in dropped_slices]
    if not kept_slices:
        raise VerificationError("all slices were dropped", classification=defect_class, gate="G10", d41_pass_consumed=True)

    # G8: Code slice (100% of kept code samples execute and pass tests; 0 unevaluated)
    g8 = gates["g8_code_slice"]
    sandbox = g8.get("sandbox")
    if (
        not isinstance(sandbox, dict)
        or sandbox.get("network_disabled") is not True
        or sandbox.get("read_only_root") is not True
        or float(sandbox.get("timeout_seconds", 999.0)) <= 0.0
        or float(sandbox.get("timeout_seconds", 999.0)) > CODE_SANDBOX_MAX_TIMEOUT_S
        or int(sandbox.get("memory_limit_mib", 99999)) <= 0
        or int(sandbox.get("memory_limit_mib", 99999)) > CODE_SANDBOX_MAX_MEMORY_MIB
    ):
        raise VerificationError(
            "G8 sandbox violation: requires disposable container with network_disabled=True, read_only_root=True, timeout <= 10s, memory <= 1024 MiB",
            classification="INSTRUMENT_UNREACHABLE_D31",
            gate="G8",
        )
    if "code" in kept_slices:
        code_kept = int(g8.get("kept_samples", 0))
        code_passed = int(g8.get("passed_samples", -1))
        code_uneval = int(g8.get("unevaluated_samples", -1))
        if code_kept <= 0:
            raise VerificationError("G8 kept_samples must be positive when code slice is kept", classification="INSTRUMENT_UNREACHABLE_D31", gate="G8")
        if code_passed != code_kept or code_uneval != 0:
            raise VerificationError(
                f"G8 failed: passed_samples={code_passed}/{code_kept}, unevaluated_samples={code_uneval}",
                classification=defect_class,
                gate="G8",
                d41_pass_consumed=True,
            )

    # G9: Math slice (100% pass deterministic answer check; 0 unevaluated)
    g9 = gates["g9_math_slice"]
    if g9.get("deterministic_answer_check") is not True:
        raise VerificationError("G9 requires deterministic_answer_check=True", classification="INSTRUMENT_UNREACHABLE_D31", gate="G9")
    math_kept = int(g9.get("kept_samples", 0))
    math_passed = int(g9.get("passed_samples", -1))
    math_uneval = int(g9.get("unevaluated_samples", -1))
    if math_kept <= 0:
        raise VerificationError("G9 kept_samples must be positive", classification="INSTRUMENT_UNREACHABLE_D31", gate="G9")
    if math_passed != math_kept or math_uneval != 0:
        raise VerificationError(
            f"G9 failed: passed_samples={math_passed}/{math_kept}, unevaluated_samples={math_uneval}",
            classification=defect_class,
            gate="G9",
            d41_pass_consumed=True,
        )

    # G10: Spot audit (per kept slice, 200 random docs: malformed <= 2.0%, refusals <= 5.0%, <= 1 redraw)
    per_slice_audit = g10.get("per_slice")
    if not isinstance(per_slice_audit, dict):
        raise VerificationError("G10 per_slice must be an object", classification="INSTRUMENT_UNREACHABLE_D31", gate="G10")
    for sname in kept_slices:
        if sname not in per_slice_audit or not isinstance(per_slice_audit[sname], dict):
            raise VerificationError(f"G10 missing audit for kept slice {sname!r}", classification="INSTRUMENT_UNREACHABLE_D31", gate="G10")
        sa = per_slice_audit[sname]
        sampled = int(sa.get("sampled_documents", 0))
        if sampled < SPOT_AUDIT_SAMPLE_PER_SLICE:
            raise VerificationError(
                f"G10 slice {sname!r} sampled_documents ({sampled}) < {SPOT_AUDIT_SAMPLE_PER_SLICE}",
                classification="INSTRUMENT_UNREACHABLE_D31",
                gate="G10",
            )
        redraws = int(sa.get("redraws_used", 0))
        if redraws < 0 or redraws > SPOT_AUDIT_MAX_SLICE_REDRAWS:
            raise VerificationError(
                f"G10 slice {sname!r} redraws_used={redraws} exceeds max {SPOT_AUDIT_MAX_SLICE_REDRAWS} (second failure stops for owner direction)",
                classification=defect_class,
                gate="G10",
                d41_pass_consumed=True,
            )
        malformed_rate = int(sa.get("malformed_count", sampled)) / sampled
        refusal_rate = int(sa.get("refusal_count", sampled)) / sampled
        if malformed_rate > SPOT_AUDIT_MAX_MALFORMED_RATE + 1e-12 or refusal_rate > SPOT_AUDIT_MAX_REFUSAL_RATE + 1e-12:
            raise VerificationError(
                f"G10 failed on kept slice {sname!r}: malformed_rate={malformed_rate:.4f} (max {SPOT_AUDIT_MAX_MALFORMED_RATE}), refusal_rate={refusal_rate:.4f} (max {SPOT_AUDIT_MAX_REFUSAL_RATE})",
                classification=defect_class,
                gate="G10",
                d41_pass_consumed=True,
            )

    # G11a–d: Distribution
    g11 = gates["g11_distribution"]
    for sub in ("g11a_slice_shares", "g11b_truncation", "g11c_repetition", "g11d_length"):
        if sub not in g11 or not isinstance(g11[sub], dict):
            raise VerificationError(f"G11 missing {sub!r}", classification="INSTRUMENT_UNREACHABLE_D31", gate="G11")

    # G11a: Slice shares within +-3.0 pp of §1.2 (renormalized if a slice was dropped)
    g11a = g11["g11a_slice_shares"]
    actual_shares = g11a.get("kept_token_shares")
    if not isinstance(actual_shares, dict) or set(actual_shares.keys()) != set(kept_slices):
        raise VerificationError(
            f"G11a kept_token_shares keys {list(actual_shares.keys()) if isinstance(actual_shares, dict) else None} != kept_slices {kept_slices}",
            classification="INSTRUMENT_UNREACHABLE_D31",
            gate="G11a",
        )
    raw_target_sum = sum(EXPECTED_SLICE_SHARES[s] for s in kept_slices)
    renorm_targets = {s: EXPECTED_SLICE_SHARES[s] / raw_target_sum for s in kept_slices}
    share_sum = sum(float(actual_shares[s]) for s in kept_slices)
    if abs(share_sum - 1.0) > 1e-4:
        raise VerificationError(
            f"G11a kept_token_shares sum to {share_sum:.6f} != 1.0",
            classification="INSTRUMENT_UNREACHABLE_D31",
            gate="G11a",
        )
    for sname in kept_slices:
        diff = abs(float(actual_shares[sname]) - renorm_targets[sname])
        if diff > SLICE_SHARE_TOLERANCE + 1e-9:
            raise VerificationError(
                f"G11a failed on slice {sname!r}: actual={float(actual_shares[sname]):.4f}, target={renorm_targets[sname]:.4f}, diff={diff:.4f} > {SLICE_SHARE_TOLERANCE}",
                classification=defect_class,
                gate="G11a",
                d41_pass_consumed=True,
            )

    # G11b: Truncation (< 1.0%)
    g11b = g11["g11b_truncation"]
    trunc_rate = float(g11b.get("truncation_rate", 1.0))
    if trunc_rate < 0.0 or trunc_rate >= TRUNCATION_MAX_RATE:
        raise VerificationError(
            f"G11b failed: truncation_rate={trunc_rate:.6f} >= {TRUNCATION_MAX_RATE}",
            classification=defect_class,
            gate="G11b",
            d41_pass_consumed=True,
        )

    # G11c: Repetition (< 2.0% of kept docs have an 8-word n-gram repeated > 3 times)
    g11c = g11["g11c_repetition"]
    if g11c.get("ngram_words") != REPETITION_NGRAM_WORDS or g11c.get("max_allowed_occurrences") != REPETITION_MAX_OCCURRENCES:
        raise VerificationError("G11c rule mismatch: requires 8-word n-gram repeated > 3 times", classification="INSTRUMENT_UNREACHABLE_D31", gate="G11c")
    rep_rate = float(g11c.get("repeated_8gram_doc_rate", 1.0))
    if rep_rate < 0.0 or rep_rate >= REPETITION_MAX_DOC_RATE:
        raise VerificationError(
            f"G11c failed: repeated_8gram_doc_rate={rep_rate:.6f} >= {REPETITION_MAX_DOC_RATE}",
            classification=defect_class,
            gate="G11c",
            d41_pass_consumed=True,
        )

    # G11d: Length (min >= 16, median in [40, 120], p99 <= 200, max <= 200)
    g11d = g11["g11d_length"]
    min_tok = int(g11d.get("min_student_tokens", 0))
    median_tok = float(g11d.get("median_student_tokens", 0.0))
    p99_tok = float(g11d.get("p99_student_tokens", 999.0))
    max_tok = int(g11d.get("max_student_tokens", 999))
    if (
        min_tok < DOC_MIN_TOKENS
        or not (DOC_MEDIAN_MIN_TOKENS <= median_tok <= DOC_MEDIAN_MAX_TOKENS)
        or p99_tok > DOC_MAX_TOKENS
        or max_tok > DOC_MAX_TOKENS
    ):
        raise VerificationError(
            f"G11d failed: min={min_tok} (>=16), median={median_tok} (40..120), p99={p99_tok} (<=200), max={max_tok} (<=200)",
            classification=defect_class,
            gate="G11d",
            d41_pass_consumed=True,
        )

    return {
        "status": "PASS",
        "kept_slices": kept_slices,
        "dropped_slices": sorted(dropped_slices),
        "renormalized_target_shares": renorm_targets,
        "g4_exact_dedup": "PASS",
        "g5_near_dedup_rate": near_dup_rate,
        "g6_panel_contamination": "PASS",
        "g7_train_overlap": "PASS",
        "g8_code_slice": "PASS" if "code" in kept_slices else "DROPPED",
        "g9_math_slice": "PASS",
        "g10_spot_audit": "PASS",
        "g11a_slice_shares": "PASS",
        "g11b_truncation_rate": trunc_rate,
        "g11c_repetition_rate": rep_rate,
        "g11d_length": {
            "min": min_tok,
            "median": median_tok,
            "p99": p99_tok,
            "max": max_tok,
        },
    }


# ---------------------------------------------------------------------------
# 6. Pilot Gates P1–P8 & §4.2 Go/No-Go Grader
# ---------------------------------------------------------------------------


def verify_pilot_bundle(pilot_bundle: dict[str, Any]) -> dict[str, Any]:
    """Verify pilot gates `P1–P8` and compute the §4.2 Go/No-Go decision."""
    if not isinstance(pilot_bundle, dict):
        raise VerificationError("pilot_bundle must be an object", classification="INSTRUMENT_UNREACHABLE_D31")

    checklist_res = verify_registration_checklist(pilot_bundle.get("registration_checklist", {}))

    # P2 & P3: Receipt uor-r4.donor-corpus/1 with 5 blocks, Merkle root, and store readback
    receipt_res = verify_generation_receipt(
        pilot_bundle.get("receipt", {}),
        pilot_bundle.get("registration_checklist"),
        is_pilot=True,
    )

    # P1: Generation throughput on declared pod, per-shard measured tok/s
    p1 = pilot_bundle.get("p1_generation")
    if not isinstance(p1, dict):
        raise VerificationError("pilot_bundle missing p1_generation", classification="INSTRUMENT_UNREACHABLE_D31", gate="P1")
    shard_rates = p1.get("per_shard_tok_per_s")
    sustained_tok_s = float(p1.get("sustained_tok_per_s", -1.0))
    if (
        not isinstance(shard_rates, list)
        or len(shard_rates) != receipt_res["shard_count"]
        or any(float(r) <= 0.0 for r in shard_rates)
        or sustained_tok_s <= 0.0
    ):
        raise VerificationError(
            "P1 requires positive measured per_shard_tok_per_s for every shard and sustained_tok_per_s > 0",
            classification="INSTRUMENT_UNREACHABLE_D31",
            gate="P1",
        )

    # P4: Rust tokenization with pinned 4096-vocab tokenizer, no Python, no float
    p4 = pilot_bundle.get("p4_rust_tokenization")
    if not isinstance(p4, dict):
        raise VerificationError("pilot_bundle missing p4_rust_tokenization", classification="INSTRUMENT_UNREACHABLE_D31", gate="P4")
    if (
        p4.get("tokenizer_sha256") != EXPECTED_STUDENT_TOKENIZER_SHA256
        or p4.get("vocabulary") != EXPECTED_STUDENT_VOCABULARY
        or p4.get("bos_id") != EXPECTED_STUDENT_BOS_ID
        or p4.get("python_used") is not False
        or p4.get("float_used") is not False
        or int(p4.get("roundtrip_decode_failures", -1)) != 0
    ):
        raise VerificationError(
            "P4 failed: Rust tokenization must use pinned d36d3e87… (vocab 4096, bos 0) with python_used=False, float_used=False, roundtrip_decode_failures=0",
            classification="INSTRUMENT_UNREACHABLE_D31",
            gate="P4",
        )

    # P5: PreparedCorpus::load acceptance + manifest counts parity
    p5 = pilot_bundle.get("p5_prepared_corpus_load")
    if not isinstance(p5, dict):
        raise VerificationError("pilot_bundle missing p5_prepared_corpus_load", classification="INSTRUMENT_UNREACHABLE_D31", gate="P5")
    if (
        p5.get("load_succeeded") is not True
        or p5.get("counts_match_manifest") is not True
        or p5.get("window_policy") != EXPECTED_WINDOW_POLICY
        or p5.get("identity_policy") != EXPECTED_IDENTITY_POLICY
        or int(p5.get("payload_tokens", 0)) != receipt_res["token_total"]
        or int(p5.get("documents", 0)) != receipt_res["document_count_total"]
        or int(p5.get("eligible_targets", 0)) <= 0
    ):
        raise VerificationError(
            "P5 failed: PreparedCorpus::load must accept the pilot UORT artifact and match manifest counts",
            classification="INSTRUMENT_UNREACHABLE_D31",
            gate="P5",
        )

    # P6: Window admission at C=192 (`batch(0, 192, 64)`); C=512 is NOT_RUN diagnostic under MAX_CONTEXT=243
    p6 = pilot_bundle.get("p6_window_admission")
    if not isinstance(p6, dict):
        raise VerificationError("pilot_bundle missing p6_window_admission", classification="INSTRUMENT_UNREACHABLE_D31", gate="P6")
    c512_status = p6.get("context_512_status", "NOT_RUN_MAX_CONTEXT_243")
    if (
        p6.get("context") != EXPECTED_OPERATING_CONTEXT
        or p6.get("rows_requested") != EXPECTED_BATCH_ROWS
        or p6.get("windows_returned") != EXPECTED_BATCH_ROWS
        or p6.get("all_have_in_document_predecessor") is not True
        or c512_status not in ("NOT_RUN_MAX_CONTEXT_243", "PASS")
    ):
        raise VerificationError(
            "P6 failed: batch(0, 192, 64) must return 64 valid windows with in-document predecessors (with context_512_status='NOT_RUN_MAX_CONTEXT_243' or 'PASS')",
            classification="INSTRUMENT_UNREACHABLE_D31",
            gate="P6",
        )

    # P7: Quality gates G4–G11 at pilot scale
    p7_res = verify_quality_gates_g4_to_g11(
        pilot_bundle.get("p7_quality_gates", {}),
        is_pilot=True,
    )

    # P8: Complete cost accounting
    p8 = pilot_bundle.get("p8_cost_accounting")
    if not isinstance(p8, dict):
        raise VerificationError("pilot_bundle missing p8_cost_accounting", classification="INSTRUMENT_UNREACHABLE_D31", gate="P8")
    for req_key in ("pod_hours", "rate_usd_per_gpu_hr", "storage_cost_usd", "transfer_cost_usd", "total_cost_usd"):
        if req_key not in p8 or float(p8[req_key]) < 0.0:
            raise VerificationError(f"P8 missing or negative {req_key!r}", classification="INSTRUMENT_UNREACHABLE_D31", gate="P8")
    if float(p8["pod_hours"]) <= 0.0 or float(p8["total_cost_usd"]) <= 0.0:
        raise VerificationError("P8 pod_hours and total_cost_usd must be positive", classification="INSTRUMENT_UNREACHABLE_D31", gate="P8")
    if float(p8["rate_usd_per_gpu_hr"]) > MAX_POD_RATE_USD_PER_GPU_HR:
        raise VerificationError(
            f"P8 rate_usd_per_gpu_hr ({p8['rate_usd_per_gpu_hr']}) exceeds ${MAX_POD_RATE_USD_PER_GPU_HR}/h cap",
            classification="STOP_COST_OR_LICENSE_BLOCKER",
            gate="P8",
        )
    if p8.get("projected_production_cost_within_approved_plan") is not True or p8.get("license_or_redistribution_unresolved") is not False:
        raise VerificationError(
            "P8 projected production cost exceeds approved plan or license/redistribution question unresolved",
            classification="STOP_COST_OR_LICENSE_BLOCKER",
            gate="P8",
        )

    # §4.2 Go / No-Go evaluation
    if sustained_tok_s < PILOT_STOP_TOK_PER_S:
        decision = "STOP_THROUGHPUT_SHORTFALL_OWNER_REPLAN"
        go_for_production = False
    elif sustained_tok_s < PILOT_GO_TOK_PER_S:
        decision = "HOLD_BELOW_300_TOK_S_GO_BAR"
        go_for_production = False
    else:
        decision = "GO_FOR_PRODUCTION_TRANCHE"
        go_for_production = True

    return {
        "status": "PASS",
        "decision": decision,
        "go_for_production": go_for_production,
        "d41_pass_consumed": False,
        "mechanism_negative": False,
        "sustained_tok_per_s": sustained_tok_s,
        "context_512_status": c512_status,
        "checklist": checklist_res,
        "receipt": receipt_res,
        "quality_gates": p7_res,
    }


# ---------------------------------------------------------------------------
# 7. Production Gates G1–G14 & Seal Verifier
# ---------------------------------------------------------------------------


def verify_production_bundle(prod_bundle: dict[str, Any]) -> dict[str, Any]:
    """Verify production gates `G1–G14` and corpus seal."""
    if not isinstance(prod_bundle, dict):
        raise VerificationError("production_bundle must be an object", classification="INSTRUMENT_UNREACHABLE_D31")

    checklist_res = verify_registration_checklist(prod_bundle.get("registration_checklist", {}))

    # G1, G2, G3: Provenance, 5-block receipt, readback, and Merkle root
    receipt_res = verify_generation_receipt(
        prod_bundle.get("receipt", {}),
        prod_bundle.get("registration_checklist"),
        is_pilot=False,
    )

    # G4–G11d: Dedup, contamination, slice correctness, distribution
    quality_res = verify_quality_gates_g4_to_g11(
        prod_bundle.get("quality_gates", {}),
        is_pilot=False,
    )

    # G12: UORT admission & student consumption contract (§5.1, §5.2)
    g12 = prod_bundle.get("g12_uort_admission")
    if not isinstance(g12, dict):
        raise VerificationError("production_bundle missing g12_uort_admission", classification="INSTRUMENT_UNREACHABLE_D31", gate="G12")
    if (
        g12.get("lineage_id") != EXPECTED_LINEAGE_ID
        or g12.get("inherits_mw_balp") is not False
        or g12.get("tokenizer_sha256") != EXPECTED_STUDENT_TOKENIZER_SHA256
        or g12.get("vocabulary") != EXPECTED_STUDENT_VOCABULARY
        or g12.get("bos_id") != EXPECTED_STUDENT_BOS_ID
        or g12.get("uort_header_bytes") != EXPECTED_UORT_HEADER_BYTES
        or g12.get("window_policy") != EXPECTED_WINDOW_POLICY
        or g12.get("identity_policy") != EXPECTED_IDENTITY_POLICY
        or g12.get("hard_targets_only") is not True
        or g12.get("logits_or_probs_stored") is not False
        or g12.get("ranks_opened_by_training") is not False
        or g12.get("split_by_document_hash_with_fixed_salt") is not True
        or g12.get("cross_split_document_leakage") != 0
        or g12.get("heldout_sealed") is not True
        or g12.get("heldout_opened_during_prep_or_gates") is not False
    ):
        raise VerificationError(
            "G12 failed: student consumption or UORT contract mismatch",
            classification="INSTRUMENT_UNREACHABLE_D31",
            gate="G12",
        )

    splits = g12.get("splits")
    if not isinstance(splits, dict) or set(splits.keys()) != {"train", "dev", "heldout"}:
        raise VerificationError("G12 splits must contain 'train', 'dev', and 'heldout'", classification="INSTRUMENT_UNREACHABLE_D31", gate="G12")

    expected_rust_split = {"train": "train", "dev": "dev", "heldout": "fresh"}
    total_split_tokens = 0
    total_split_docs = 0
    for sname, sdata in splits.items():
        if not isinstance(sdata, dict):
            raise VerificationError(f"G12 splits.{sname} must be an object", classification="INSTRUMENT_UNREACHABLE_D31", gate="G12")
        if sdata.get("rust_split_enum") != expected_rust_split[sname]:
            raise VerificationError(
                f"G12 splits.{sname}.rust_split_enum must be {expected_rust_split[sname]!r}",
                classification="INSTRUMENT_UNREACHABLE_D31",
                gate="G12",
            )
        validate_sha256_hex(sdata.get("tokens_sha256"), f"G12 splits.{sname}.tokens_sha256", gate="G12")
        validate_sha256_hex(sdata.get("mask_sha256"), f"G12 splits.{sname}.mask_sha256", gate="G12")
        if (
            sdata.get("prepared_corpus_load_succeeded") is not True
            or sdata.get("first_token_is_bos_0") is not True
            or sdata.get("mask_binary_and_matches_payload") is not True
            or sdata.get("eligible_targets_match_manifest") is not True
            or int(sdata.get("payload_tokens", 0)) <= 0
            or int(sdata.get("documents", 0)) <= 0
            or int(sdata.get("eligible_targets", 0)) <= 0
        ):
            raise VerificationError(f"G12 splits.{sname} failed PreparedCorpus::load invariants", classification="INSTRUMENT_UNREACHABLE_D31", gate="G12")
        total_split_tokens += int(sdata["payload_tokens"])
        total_split_docs += int(sdata["documents"])

    if int(splits["dev"]["eligible_targets"]) < DEV_MIN_ELIGIBLE_TARGETS:
        raise VerificationError(
            f"G12 failed: dev.eligible_targets ({splits['dev']['eligible_targets']}) < {DEV_MIN_ELIGIBLE_TARGETS} (§5.1 power floor)",
            classification="INSTRUMENT_UNREACHABLE_D31",
            gate="G12",
        )
    if total_split_tokens != receipt_res["token_total"] or total_split_docs != receipt_res["document_count_total"]:
        raise VerificationError(
            "G12 sum of split payload_tokens/documents does not equal receipt content_pin totals",
            classification="INSTRUMENT_UNREACHABLE_D31",
            gate="G12",
        )

    # Optional embedded raw UORT fixture check inside G12
    if "embedded_uort_fixture" in g12:
        fx = g12["embedded_uort_fixture"]
        verify_uort_and_mask_bytes(
            bytes.fromhex(fx["uort_hex"]),
            bytes.fromhex(fx["mask_hex"]),
            split=fx.get("split", "train"),
            expected_payload_tokens=int(fx["expected_payload_tokens"]),
            expected_eligible_targets=int(fx["expected_eligible_targets"]),
        )

    # G13: Window admission at C=192 (`batch(0, 192, 64)`) is the mandatory seal gate;
    # C=512 is `NOT_RUN_MAX_CONTEXT_243` diagnostic until `MAX_CONTEXT` is raised in source.
    g13 = prod_bundle.get("g13_window_admission")
    if not isinstance(g13, dict):
        raise VerificationError("production_bundle missing g13_window_admission", classification="INSTRUMENT_UNREACHABLE_D31", gate="G13")
    c512_status = g13.get("context_512_status", "NOT_RUN_MAX_CONTEXT_243")
    if (
        g13.get("context") != EXPECTED_OPERATING_CONTEXT
        or g13.get("rows_requested") != EXPECTED_BATCH_ROWS
        or g13.get("train_windows_returned") != EXPECTED_BATCH_ROWS
        or g13.get("dev_windows_returned") != EXPECTED_BATCH_ROWS
        or g13.get("all_have_in_document_predecessor") is not True
        or c512_status not in ("NOT_RUN_MAX_CONTEXT_243", "PASS")
    ):
        raise VerificationError(
            "G13 failed: batch(0, 192, 64) must return 64 valid windows with in-document predecessors on train and dev (with context_512_status='NOT_RUN_MAX_CONTEXT_243' or 'PASS')",
            classification="INSTRUMENT_UNREACHABLE_D31",
            gate="G13",
        )

    # G14: Seal & D41 exception expiry
    g14 = prod_bundle.get("g14_seal")
    if not isinstance(g14, dict):
        raise VerificationError("production_bundle missing g14_seal", classification="INSTRUMENT_UNREACHABLE_D31", gate="G14")
    if (
        g14.get("receipt_written") is not True
        or g14.get("manifest_written") is not True
        or g14.get("merkle_root") != receipt_res["merkle_root"]
        or g14.get("store_readback_reverified") is not True
        or g14.get("d41_exception_expired") is not True
        or not isinstance(g14.get("seal_timestamp_utc"), str)
        or not g14["seal_timestamp_utc"].strip()
    ):
        raise VerificationError(
            "G14 failed: seal requires receipt_written, manifest_written, matching merkle_root, store_readback_reverified=True, and d41_exception_expired=True",
            classification="INSTRUMENT_UNREACHABLE_D31",
            gate="G14",
        )

    return {
        "status": "PASS",
        "decision": "PRODUCTION_CORPUS_SEALED_D41_EXPIRED",
        "corpus_sealed": True,
        "d41_exception_expired": True,
        "d41_pass_consumed": True,
        "mechanism_negative": False,
        "merkle_root": receipt_res["merkle_root"],
        "merkle_mode": receipt_res["merkle_mode"],
        "context_512_status": c512_status,
        "token_total": receipt_res["token_total"],
        "document_count_total": receipt_res["document_count_total"],
        "dev_eligible_targets": int(splits["dev"]["eligible_targets"]),
        "checklist": checklist_res,
        "receipt": receipt_res,
        "quality_gates": quality_res,
    }


# ---------------------------------------------------------------------------
# 8. Synthetic Fixtures & 32-Test Positive/Negative Self-Test Suite
# ---------------------------------------------------------------------------


def make_valid_checklist(*, mode: str = "production") -> dict[str, Any]:
    w1_sha = sha256_bytes(b"gpt-oss-120b-mxfp4-part-00001")
    w2_sha = sha256_bytes(b"gpt-oss-120b-mxfp4-part-00002")
    combined_w_sha = sha256_bytes((w1_sha + w2_sha).encode("ascii"))
    return {
        "sheet_sha256": EXPECTED_FIT_SHEET_SHA256,
        "sheet_verdict": "FIT",
        "d41_record_ref": "docs/integration/DECISIONS.md#d41--one-time-offline-donor-corpus-exception-owner-2026-10-19",
        "owner_ratification_id": "owner-d41-ratification-2026-10-19",
        "donor": {
            "repo": EXPECTED_DONOR_REPO,
            "revision": "a1b2c3d4e5f60718293a4b5c6d7e8f90a1b2c3d4",
            "quantization": EXPECTED_DONOR_QUANTIZATION,
            "license_spdx": EXPECTED_DONOR_LICENSE_SPDX,
            "weight_files": [
                {"path": "model-00001-of-00002.safetensors", "sha256": w1_sha},
                {"path": "model-00002-of-00002.safetensors", "sha256": w2_sha},
            ],
            "combined_weights_sha256": combined_w_sha,
        },
        "engine": {
            "name": "vllm",
            "version": "0.8.4",
            "container_image_digest": "sha256:" + sha256_bytes(b"vllm-0.8.4-image"),
            "driver_version": "570.124.06",
            "cuda_version": "12.8",
            "runtime_compilation_of_project_code": False,
        },
        "prompt_set": {
            "prompts_path": "prompts/prompts.jsonl.zst",
            "prompts_sha256": sha256_bytes(b"distill-d1-prompts-v1"),
            "slice_map": {
                "conversation": {"target_share": 0.35, "prompt_count": 52500},
                "qa_instruction": {"target_share": 0.20, "prompt_count": 30000},
                "code": {"target_share": 0.20, "prompt_count": 30000},
                "reasoning_prose": {"target_share": 0.15, "prompt_count": 22500},
                "exposition": {"target_share": 0.10, "prompt_count": 15000},
            },
            "slice_licenses_spdx": {
                "conversation": "Apache-2.0",
                "qa_instruction": "Apache-2.0",
                "code": "MIT",
                "reasoning_prose": "Apache-2.0",
                "exposition": "CC-BY-4.0",
            },
            "panel_prescreened": True,
            "authored_from_heldout_panel": False,
        },
        "sampling": {
            "temperature": 0.7,
            "top_p": 0.95,
            "top_k": 20,
            "max_tokens": 512,
            "reasoning_effort": "medium",
            "shard_seeds": [1001, 1002, 1003, 1004, 1005],
            "tuned_on_outputs": False,
        },
        "student_tokenizer": {
            "sha256": EXPECTED_STUDENT_TOKENIZER_SHA256,
            "vocabulary": EXPECTED_STUDENT_VOCABULARY,
            "bos_id": EXPECTED_STUDENT_BOS_ID,
        },
        "pod": {
            "provider": "vast",
            "gpu_type": "RTX PRO 6000",
            "gpu_count": 1,
            "vram_gb": 96,
            "lease_id": "vast-pro6000-lease-01",
            "rate_usd_per_gpu_hr": 2.50,
            "running_pods_total": 1,
            "local_weights_only": True,
            "external_api_used": False,
        },
        "budget": {
            "mode": mode,
            "token_target": PILOT_TOKEN_TARGET if mode == "pilot" else PRODUCTION_TOKEN_TARGET,
            "shard_plan": ["shard-00", "shard-01", "shard-02", "shard-03", "shard-04"],
            "max_hours": 1.5 if mode == "pilot" else 28.0,
            "max_dollars": 5.0 if mode == "pilot" else 75.0,
            "driver_math_optimization_completed": True,
            "student_training_authorized": False,
        },
        "store": {
            "destination_path": "datasets/caseyallard/uor-r4-store/distill-d1/",
            "readback_method": "hf-hub-download-sha256-verify",
            "md5_index_path": "datasets/caseyallard/uor-r4-store/distill-d1/MD5SUMS",
            "merkle_mode": "hex-colon",
        },
        "gate_thresholds": {
            "g4_max_exact_duplicates": 0,
            "g5_shingle_words": NEAR_DEDUP_SHINGLE_WORDS,
            "g5_jaccard_threshold": NEAR_DEDUP_JACCARD_THRESHOLD,
            "g5_max_residual_rate": NEAR_DEDUP_MAX_RESIDUAL_RATE,
            "g6_panel_ngram_words": PANEL_EXCLUSION_NGRAM_WORDS,
            "g6_short_turn_min_words": PANEL_EXCLUSION_SHORT_MIN_WORDS,
            "g6_short_turn_max_words": PANEL_EXCLUSION_SHORT_MAX_WORDS,
            "g6_max_contaminated_docs": 0,
            "g7_train_overlap_words": TRAIN_OVERLAP_NGRAM_WORDS,
            "g7_max_overlap_docs": 0,
            "g8_code_pass_rate": 1.0,
            "g8_code_max_unevaluated": 0,
            "g9_math_pass_rate": 1.0,
            "g9_math_max_unevaluated": 0,
            "g10_spot_audit_docs_per_slice": SPOT_AUDIT_SAMPLE_PER_SLICE,
            "g10_max_malformed_rate": SPOT_AUDIT_MAX_MALFORMED_RATE,
            "g10_max_refusal_rate": SPOT_AUDIT_MAX_REFUSAL_RATE,
            "g11a_slice_share_tolerance_pp": SLICE_SHARE_TOLERANCE,
            "g11b_max_truncation_rate": TRUNCATION_MAX_RATE,
            "g11c_repetition_ngram_words": REPETITION_NGRAM_WORDS,
            "g11c_repetition_max_count": REPETITION_MAX_OCCURRENCES,
            "g11c_max_doc_rate": REPETITION_MAX_DOC_RATE,
            "g11d_min_tokens": DOC_MIN_TOKENS,
            "g11d_max_tokens": DOC_MAX_TOKENS,
            "g11d_median_min": DOC_MEDIAN_MIN_TOKENS,
            "g11d_median_max": DOC_MEDIAN_MAX_TOKENS,
            "g12_dev_min_eligible_targets": DEV_MIN_ELIGIBLE_TARGETS,
            "p_go_tok_per_s": PILOT_GO_TOK_PER_S,
            "p_stop_tok_per_s": PILOT_STOP_TOK_PER_S,
        },
        "dispatch": {
            "pm_dispatch_id": "kimi-pm-dispatch-distill-d1-01",
            "d40_release_record_id": "d40-release-distill-d1-20261019",
            "d40_owner": "casey",
            "d40_uor_lab_env_unset": True,
            "d40_typed_by_lab": False,
        },
        "student_target": {
            "lineage_id": EXPECTED_LINEAGE_ID,
            "inherits_mw_balp": False,
            "separate_training_sheet_path": "docs/harness/prep/m1-distill-d1-student-training-DRAFT.md",
            "hard_targets_only": True,
            "logits_or_probs_stored": False,
            "ranks_opened_by_training": False,
        },
    }


def make_valid_receipt(checklist: dict[str, Any], *, is_pilot: bool = False) -> dict[str, Any]:
    scale = 1 if is_pilot else 30
    shards = []
    for idx in range(5):
        sh_sha = sha256_bytes(f"distill-d1-shard-{idx}-scale-{scale}".encode("ascii"))
        shards.append(
            {
                "shard_id": f"shard-0{idx}",
                "path": f"shards/shard-0{idx}.jsonl.zst",
                "sha256": sh_sha,
                "document_count": 1000 * scale,
                "student_token_count": 200_000 * scale,
            }
        )
    merkle_mode = checklist["store"]["merkle_mode"]
    merkle = compute_shard_merkle_root([s["sha256"] for s in shards], mode=merkle_mode)
    return {
        "schema": EXPECTED_RECEIPT_SCHEMA,
        "donor_pin": {
            "repo": checklist["donor"]["repo"],
            "revision": checklist["donor"]["revision"],
            "weights_sha256": {
                "files": copy.deepcopy(checklist["donor"]["weight_files"]),
                "combined_sha256": checklist["donor"]["combined_weights_sha256"],
            },
            "quantization": checklist["donor"]["quantization"],
            "total_params": "116.8B",
            "active_params": "5.1B",
            "license_spdx": checklist["donor"]["license_spdx"],
        },
        "generation_pin": {
            "prompt_set_sha256": checklist["prompt_set"]["prompts_sha256"],
            "slice_map": copy.deepcopy(checklist["prompt_set"]["slice_map"]),
            "temperature": 0.7,
            "top_p": 0.95,
            "top_k": 20,
            "max_tokens": 512,
            "reasoning_effort": "medium",
            "shard_seeds": copy.deepcopy(checklist["sampling"]["shard_seeds"]),
            "shard_plan": copy.deepcopy(checklist["budget"]["shard_plan"]),
        },
        "run_pin": {
            "pod_id": "vast-pro6000-pod-01",
            "provider": checklist["pod"]["provider"],
            "gpu_type": checklist["pod"]["gpu_type"],
            "gpu_count": checklist["pod"]["gpu_count"],
            "lease_id": checklist["pod"]["lease_id"],
            "rate_usd_per_gpu_hr": checklist["pod"]["rate_usd_per_gpu_hr"],
            "utc_start": "2026-10-20T00:00:00Z",
            "utc_end": "2026-10-20T20:48:00Z",
            "engine_version": checklist["engine"]["version"],
            "image_digest": checklist["engine"]["container_image_digest"],
            "driver_version": checklist["engine"]["driver_version"],
            "cuda_version": checklist["engine"]["cuda_version"],
        },
        "content_pin": {
            "shards": shards,
            "document_count_total": 5000 * scale,
            "token_total": 1_000_000 * scale,
            "merkle_mode": merkle_mode,
            "merkle_root": merkle,
        },
        "readback": {
            "destination_store_path": checklist["store"]["destination_path"],
            "shards_readback": [
                {"shard_id": s["shard_id"], "path": s["path"], "sha256": s["sha256"]}
                for s in shards
            ],
            "recomputed_merkle_root": merkle,
            "md5_index_record": {
                "path": checklist["store"]["md5_index_path"],
                "md5": hashlib.md5(b"distill-d1-md5-index").hexdigest(),
                "sha256": sha256_bytes(b"distill-d1-md5-index"),
            },
            "verification_timestamp_utc": "2026-10-20T21:00:00Z",
            "operator": "kimi-pm1",
        },
    }


def make_valid_quality_gates(*, is_pilot: bool = False) -> dict[str, Any]:
    scale = 1 if is_pilot else 30
    return {
        "g4_exact_dedup": {
            "evaluated": True,
            "kept_documents": 5000 * scale,
            "duplicate_document_sha256_count": 0,
        },
        "g5_near_dedup": {
            "evaluated": True,
            "shingle_words": 5,
            "jaccard_threshold": 0.80,
            "residual_near_duplicate_rate": 0.012,
        },
        "g6_panel_contamination": {
            "evaluated": True,
            "panels_checked": ["heldout-200-a.json", "heldout-200-b.json", "sealed-panel-v1.json"],
            "ngram_window_words": 8,
            "short_turn_min_words": 4,
            "short_turn_max_words": 7,
            "matched_kept_documents": 0,
        },
        "g7_train_overlap": {
            "evaluated": True,
            "reference_corpus": "mw-balp/train",
            "ngram_words": 12,
            "shared_12word_kept_documents": 0,
        },
        "g8_code_slice": {
            "evaluated": True,
            "kept_samples": 1000 * scale,
            "passed_samples": 1000 * scale,
            "unevaluated_samples": 0,
            "sandbox": {
                "network_disabled": True,
                "read_only_root": True,
                "timeout_seconds": 10,
                "memory_limit_mib": 1024,
            },
        },
        "g9_math_slice": {
            "evaluated": True,
            "deterministic_answer_check": True,
            "kept_samples": 750 * scale,
            "passed_samples": 750 * scale,
            "unevaluated_samples": 0,
        },
        "g10_spot_audit": {
            "evaluated": True,
            "dropped_slices": [],
            "per_slice": {
                s: {
                    "sampled_documents": 200,
                    "malformed_count": 2,
                    "refusal_count": 4,
                    "redraws_used": 0,
                }
                for s in CANONICAL_SLICES
            },
        },
        "g11_distribution": {
            "evaluated": True,
            "g11a_slice_shares": {
                "kept_token_shares": {
                    "conversation": 0.352,
                    "qa_instruction": 0.198,
                    "code": 0.201,
                    "reasoning_prose": 0.149,
                    "exposition": 0.100,
                }
            },
            "g11b_truncation": {
                "truncation_rate": 0.004,
            },
            "g11c_repetition": {
                "ngram_words": 8,
                "max_allowed_occurrences": 3,
                "repeated_8gram_doc_rate": 0.008,
            },
            "g11d_length": {
                "min_student_tokens": 24,
                "median_student_tokens": 92.0,
                "p99_student_tokens": 198.0,
                "max_student_tokens": 200,
            },
        },
    }


def make_valid_pilot_bundle() -> dict[str, Any]:
    chk = make_valid_checklist(mode="pilot")
    rec = make_valid_receipt(chk, is_pilot=True)
    return {
        "registration_checklist": chk,
        "receipt": rec,
        "p1_generation": {
            "per_shard_tok_per_s": [402.5, 410.0, 398.0, 406.5, 408.0],
            "sustained_tok_per_s": 405.0,
        },
        "p4_rust_tokenization": {
            "tokenizer_sha256": EXPECTED_STUDENT_TOKENIZER_SHA256,
            "vocabulary": EXPECTED_STUDENT_VOCABULARY,
            "bos_id": EXPECTED_STUDENT_BOS_ID,
            "python_used": False,
            "float_used": False,
            "roundtrip_decode_failures": 0,
        },
        "p5_prepared_corpus_load": {
            "load_succeeded": True,
            "counts_match_manifest": True,
            "window_policy": EXPECTED_WINDOW_POLICY,
            "identity_policy": EXPECTED_IDENTITY_POLICY,
            "payload_tokens": 1_000_000,
            "documents": 5000,
            "eligible_targets": 680_000,
        },
        "p6_window_admission": {
            "context": EXPECTED_OPERATING_CONTEXT,
            "rows_requested": EXPECTED_BATCH_ROWS,
            "windows_returned": EXPECTED_BATCH_ROWS,
            "all_have_in_document_predecessor": True,
            "context_512_status": "NOT_RUN_MAX_CONTEXT_243",
        },
        "p7_quality_gates": make_valid_quality_gates(is_pilot=True),
        "p8_cost_accounting": {
            "pod_hours": 0.72,
            "rate_usd_per_gpu_hr": 2.50,
            "storage_cost_usd": 0.05,
            "transfer_cost_usd": 0.02,
            "total_cost_usd": 1.87,
            "projected_production_cost_within_approved_plan": True,
            "license_or_redistribution_unresolved": False,
        },
    }


def make_valid_production_bundle() -> dict[str, Any]:
    chk = make_valid_checklist(mode="production")
    rec = make_valid_receipt(chk, is_pilot=False)
    fixture_tokens = [0] + [(i % 4000) + 3 for i in range(64)] + [0, 10, 11]
    fixture_mask = bytes([0] + [1] * 64 + [0, 1, 1])
    fixture_uort = build_uort_bytes(fixture_tokens, EXPECTED_STUDENT_VOCABULARY)
    return {
        "registration_checklist": chk,
        "receipt": rec,
        "quality_gates": make_valid_quality_gates(is_pilot=False),
        "g12_uort_admission": {
            "lineage_id": EXPECTED_LINEAGE_ID,
            "inherits_mw_balp": False,
            "tokenizer_sha256": EXPECTED_STUDENT_TOKENIZER_SHA256,
            "vocabulary": EXPECTED_STUDENT_VOCABULARY,
            "bos_id": EXPECTED_STUDENT_BOS_ID,
            "uort_header_bytes": EXPECTED_UORT_HEADER_BYTES,
            "window_policy": EXPECTED_WINDOW_POLICY,
            "identity_policy": EXPECTED_IDENTITY_POLICY,
            "hard_targets_only": True,
            "logits_or_probs_stored": False,
            "ranks_opened_by_training": False,
            "split_by_document_hash_with_fixed_salt": True,
            "cross_split_document_leakage": 0,
            "heldout_sealed": True,
            "heldout_opened_during_prep_or_gates": False,
            "splits": {
                "train": {
                    "rust_split_enum": "train",
                    "tokens_sha256": sha256_bytes(b"distill-d1-train-uort"),
                    "mask_sha256": sha256_bytes(b"distill-d1-train-mask"),
                    "payload_tokens": 27_000_000,
                    "documents": 135_000,
                    "eligible_targets": 18_500_000,
                    "prepared_corpus_load_succeeded": True,
                    "first_token_is_bos_0": True,
                    "mask_binary_and_matches_payload": True,
                    "eligible_targets_match_manifest": True,
                },
                "dev": {
                    "rust_split_enum": "dev",
                    "tokens_sha256": sha256_bytes(b"distill-d1-dev-uort"),
                    "mask_sha256": sha256_bytes(b"distill-d1-dev-mask"),
                    "payload_tokens": 1_500_000,
                    "documents": 7_500,
                    "eligible_targets": 1_020_000,
                    "prepared_corpus_load_succeeded": True,
                    "first_token_is_bos_0": True,
                    "mask_binary_and_matches_payload": True,
                    "eligible_targets_match_manifest": True,
                },
                "heldout": {
                    "rust_split_enum": "fresh",
                    "tokens_sha256": sha256_bytes(b"distill-d1-heldout-uort"),
                    "mask_sha256": sha256_bytes(b"distill-d1-heldout-mask"),
                    "payload_tokens": 1_500_000,
                    "documents": 7_500,
                    "eligible_targets": 1_020_000,
                    "prepared_corpus_load_succeeded": True,
                    "first_token_is_bos_0": True,
                    "mask_binary_and_matches_payload": True,
                    "eligible_targets_match_manifest": True,
                },
            },
            "embedded_uort_fixture": {
                "split": "train",
                "uort_hex": fixture_uort.hex(),
                "mask_hex": fixture_mask.hex(),
                "expected_payload_tokens": 68,
                "expected_eligible_targets": 66,
            },
        },
        "g13_window_admission": {
            "context": EXPECTED_OPERATING_CONTEXT,
            "rows_requested": EXPECTED_BATCH_ROWS,
            "train_windows_returned": EXPECTED_BATCH_ROWS,
            "dev_windows_returned": EXPECTED_BATCH_ROWS,
            "all_have_in_document_predecessor": True,
            "context_512_status": "NOT_RUN_MAX_CONTEXT_243",
        },
        "g14_seal": {
            "receipt_written": True,
            "manifest_written": True,
            "merkle_root": rec["content_pin"]["merkle_root"],
            "store_readback_reverified": True,
            "d41_exception_expired": True,
            "seal_timestamp_utc": "2026-10-20T22:00:00Z",
        },
    }


def _expect_failure(
    fn: Callable[[], Any],
    *,
    expected_classification: str | None = None,
    expected_gate: str | None = None,
    expected_d41_consumed: bool | None = None,
) -> VerificationError:
    try:
        fn()
    except VerificationError as exc:
        if expected_classification is not None and exc.classification != expected_classification:
            raise AssertionError(
                f"expected classification {expected_classification!r}, got {exc.classification!r} ({exc})"
            ) from exc
        if expected_gate is not None and exc.gate != expected_gate:
            raise AssertionError(
                f"expected gate {expected_gate!r}, got {exc.gate!r} ({exc})"
            ) from exc
        if expected_d41_consumed is not None and exc.d41_pass_consumed != expected_d41_consumed:
            raise AssertionError(
                f"expected d41_pass_consumed={expected_d41_consumed}, got {exc.d41_pass_consumed} ({exc})"
            ) from exc
        return exc
    raise AssertionError("expected VerificationError, but call succeeded")


def run_self_tests(repo_root: Path) -> dict[str, Any]:
    """Run all 32 positive and negative consumer verifier self-tests."""
    results: list[dict[str, Any]] = []

    def record(idx: int, name: str, detail: str) -> None:
        line = f"[{idx}/32] PASS {name}: {detail}"
        print(line)
        results.append({"test": idx, "name": name, "status": "PASS", "detail": detail})

    # [1/32] Live repo & staged sheet contract + top/bottom status contradiction check
    r1 = verify_sheet_and_repo_contract(repo_root)
    assert r1["status"] == "PASS"
    _expect_failure(
        lambda: parse_sheet_fitness_status(
            "- **Status:** DRAFT, **TEST FITNESS: PENDING**\n---\nTEST FITNESS: FIT\n"
        ),
        expected_classification="INSTRUMENT_UNREACHABLE_D31",
    )
    record(1, "repo_and_sheet_contract", f"sheet_sha256={r1['sheet']['sha256'][:16]}… ({r1['sheet']['fitness_status']})")

    # [2/32] Valid 13-item §8 registration checklist
    chk = make_valid_checklist()
    r2 = verify_registration_checklist(chk)
    assert r2["items_verified"] == 13 and r2["sheet_sha256"] == EXPECTED_FIT_SHEET_SHA256
    record(2, "valid_registration_checklist", "13/13 §8 items verified against post-FIT sheet SHA")

    # [3/32] Checklist rejects unpinned donor revision, un-amended Qwen3 fallback, draft SHA, lab-typed D40, or student_training_authorized=True
    bad_chk_rev = copy.deepcopy(chk)
    bad_chk_rev["donor"]["revision"] = "main"
    _expect_failure(lambda: verify_registration_checklist(bad_chk_rev), expected_classification="INSTRUMENT_UNREACHABLE_D31")
    bad_chk_fallback = copy.deepcopy(chk)
    bad_chk_fallback["donor"]["repo"] = "Qwen/Qwen3-30B-A3B"
    _expect_failure(lambda: verify_registration_checklist(bad_chk_fallback), expected_classification="INSTRUMENT_UNREACHABLE_D31")
    bad_chk_draft = copy.deepcopy(chk)
    bad_chk_draft["sheet_sha256"] = EXPECTED_DRAFT_SHEET_SHA256
    _expect_failure(lambda: verify_registration_checklist(bad_chk_draft), expected_classification="INSTRUMENT_UNREACHABLE_D31")
    bad_chk_d40 = copy.deepcopy(chk)
    bad_chk_d40["dispatch"]["d40_typed_by_lab"] = True
    _expect_failure(lambda: verify_registration_checklist(bad_chk_d40), expected_classification="INSTRUMENT_UNREACHABLE_D31")
    bad_chk_opt = copy.deepcopy(chk)
    bad_chk_opt["budget"]["driver_math_optimization_completed"] = False
    _expect_failure(lambda: verify_registration_checklist(bad_chk_opt), expected_classification="INSTRUMENT_UNREACHABLE_D31")
    bad_chk_student_spend = copy.deepcopy(chk)
    bad_chk_student_spend["budget"]["student_training_authorized"] = True
    _expect_failure(lambda: verify_registration_checklist(bad_chk_student_spend), expected_classification="INSTRUMENT_UNREACHABLE_D31")
    bad_chk_soft = copy.deepcopy(chk)
    bad_chk_soft["student_target"]["logits_or_probs_stored"] = True
    _expect_failure(lambda: verify_registration_checklist(bad_chk_soft), expected_classification="INSTRUMENT_UNREACHABLE_D31")
    record(3, "checklist_rejects_unpinned_or_unauthorized_bindings", "rejected 'main' revision, Qwen3 fallback, un-FIT draft SHA, lab-typed D40, uncompleted math opt, student training spend, stored probs")

    # [4/32] Valid 5-block receipt & Merkle root modes (`hex-colon` and `raw-concat`) + merkle_mode mismatch rejection
    rec = make_valid_receipt(chk)
    r4 = verify_generation_receipt(rec, chk)
    assert r4["status"] == "PASS" and r4["merkle_mode"] == "hex-colon"
    leaves = [s["sha256"] for s in rec["content_pin"]["shards"]]
    raw_concat_merkle = compute_shard_merkle_root(leaves, mode="raw-concat")
    assert len(raw_concat_merkle) == 64 and raw_concat_merkle != r4["merkle_root"]
    single_merkle = compute_shard_merkle_root([leaves[0]], mode="hex-colon")
    assert len(single_merkle) == 64 and single_merkle != leaves[0]
    bad_rec_mode = copy.deepcopy(rec)
    bad_rec_mode["content_pin"]["merkle_mode"] = "raw-concat"
    bad_rec_mode["content_pin"]["merkle_root"] = raw_concat_merkle
    bad_rec_mode["readback"]["recomputed_merkle_root"] = raw_concat_merkle
    _expect_failure(lambda: verify_generation_receipt(bad_rec_mode, chk), expected_classification="INSTRUMENT_UNREACHABLE_D31", expected_gate="G2")
    record(4, "valid_receipt_and_merkle_root", f"merkle_root={r4['merkle_root'][:16]}… (hex-colon & raw-concat verified)")

    # [5/32] Receipt missing or null/empty field or unsorted shards -> INSTRUMENT_UNREACHABLE_D31
    bad_rec_missing = copy.deepcopy(rec)
    del bad_rec_missing["readback"]
    _expect_failure(lambda: verify_generation_receipt(bad_rec_missing, chk), expected_classification="INSTRUMENT_UNREACHABLE_D31", expected_gate="G2", expected_d41_consumed=False)
    bad_rec_null = copy.deepcopy(rec)
    bad_rec_null["run_pin"]["driver_version"] = None
    _expect_failure(lambda: verify_generation_receipt(bad_rec_null, chk), expected_classification="INSTRUMENT_UNREACHABLE_D31", expected_gate="G2", expected_d41_consumed=False)
    bad_rec_empty = copy.deepcopy(rec)
    bad_rec_empty["readback"]["operator"] = "   "
    _expect_failure(lambda: verify_generation_receipt(bad_rec_empty, chk), expected_classification="INSTRUMENT_UNREACHABLE_D31", expected_gate="G2", expected_d41_consumed=False)
    bad_rec_order = copy.deepcopy(rec)
    bad_rec_order["content_pin"]["shards"][0], bad_rec_order["content_pin"]["shards"][1] = (
        bad_rec_order["content_pin"]["shards"][1],
        bad_rec_order["content_pin"]["shards"][0],
    )
    _expect_failure(lambda: verify_generation_receipt(bad_rec_order, chk), expected_classification="INSTRUMENT_UNREACHABLE_D31", expected_gate="G2", expected_d41_consumed=False)
    record(5, "receipt_missing_or_null_field_is_d31", "missing block, null field, blank string, and unsorted shards rejected as D31 without consuming pass")

    # [6/32] Raw UORT v1 binary & C=192 64-row window admission verifier
    doc1_tokens = [0] + [(i % 4000) + 3 for i in range(200)]  # BOS + 200 student tokens (exact §1.3 cap)
    doc2_tokens = [0] + [(i % 4000) + 50 for i in range(20)]  # BOS + 20 student tokens
    all_tokens = doc1_tokens + doc2_tokens
    mask_bytes = bytes([0] + [1] * 200 + [0] + [1] * 20)
    uort_bytes = build_uort_bytes(all_tokens, EXPECTED_STUDENT_VOCABULARY)
    r6 = verify_uort_and_mask_bytes(
        uort_bytes,
        mask_bytes,
        split="train",
        expected_payload_tokens=len(all_tokens),
        expected_eligible_targets=220,
    )
    assert r6["documents"] == 2 and r6["doc_non_bos_lengths"] == [200, 20]
    assert r6["batch_0_c192_r64"]["windows_returned"] == 64
    assert r6["batch_0_c192_r64"]["max_window_len"] == 64
    _expect_failure(
        lambda: verify_uort_and_mask_bytes(b"BAD!" + uort_bytes[4:], mask_bytes, split="train", expected_payload_tokens=len(all_tokens), expected_eligible_targets=220),
        expected_gate="G12",
    )
    _expect_failure(
        lambda: verify_uort_and_mask_bytes(uort_bytes, bytes([2]) + mask_bytes[1:], split="train", expected_payload_tokens=len(all_tokens), expected_eligible_targets=220),
        expected_gate="G12",
    )
    bad_bos_uort = build_uort_bytes([5] + all_tokens[1:], EXPECTED_STUDENT_VOCABULARY)
    _expect_failure(
        lambda: verify_uort_and_mask_bytes(bad_bos_uort, mask_bytes, split="train", expected_payload_tokens=len(all_tokens), expected_eligible_targets=220),
        expected_gate="G12",
    )
    bad_id_uort = build_uort_bytes([0, 4096] + all_tokens[2:], EXPECTED_STUDENT_VOCABULARY)
    _expect_failure(
        lambda: verify_uort_and_mask_bytes(bad_id_uort, mask_bytes, split="train", expected_payload_tokens=len(all_tokens), expected_eligible_targets=220),
        expected_gate="G12",
    )
    record(6, "raw_uort_v1_and_c192_window_verifier", f"verified 222-token UORT v1 + mask + 64 windows (tokens_sha={r6['tokens_sha256'][:12]}…)")

    # [7/32] Raw PanelExclusion 8-word window and 4–7-word short-turn contamination verifier
    panel_reqs = [
        {"id": "req-long-01", "user_turns": ["Can you explain how the discrete ternary binder groups three atoms together?"]},
        {"id": "req-short-02", "user_turns": ["Why do leaves change color?"]},
        {"id": "req-tiny-03", "user_turns": ["Hi Dr. Smith"]},
    ]
    clean_docs = [
        {"id": "d1", "text": "Hi Dr. Smith! Today we study integer geometry and prime factorisation."},
        {"id": "d2", "text": "Leaves contain chlorophyll and carotenoids in autumn forests."},
    ]
    r7_clean = check_panel_exclusion_text(clean_docs, panel_reqs)
    assert r7_clean["clean"] is True and r7_clean["panel_stats"]["uncovered_turns"] == 1
    dirty_8gram = [{"id": "d3", "text": "Here we explain how the discrete ternary binder groups three atoms in stages."}]
    r7_8gram = check_panel_exclusion_text(dirty_8gram, panel_reqs)
    assert r7_8gram["clean"] is False and r7_8gram["hits"][0]["kind"] == "panel_8gram"
    dirty_short = [{"id": "d4", "text": "A child asked: why do leaves change color during October?"}]
    r7_short = check_panel_exclusion_text(dirty_short, panel_reqs)
    assert r7_short["clean"] is False and r7_short["hits"][0]["kind"] == "panel_short_request"
    record(7, "raw_panel_exclusion_8gram_and_short_turn_verifier", "verified clean, 8-gram hit, 5-word short-turn hit, and 3-word uncovered turn")

    # [8/32] Raw 5-word shingle Jaccard, 12-word mw-balp overlap, and 8-gram repetition helpers
    base_sentence = " ".join(f"word{i}" for i in range(30))
    near_dup_docs = [
        {"id": "nd1", "text": base_sentence},
        {"id": "nd2", "text": base_sentence + " extra"},
    ] + [{"id": f"uniq{k}", "text": " ".join(f"u{k}_{m}" for m in range(25))} for k in range(10)]
    r8_jacc = check_5word_jaccard_clusters(near_dup_docs)
    assert r8_jacc["duplicate_docs"] == 1 and r8_jacc["pass"] is False
    mw_train = ["alpha beta gamma delta epsilon zeta eta theta iota kappa lambda mu nu xi"]
    r8_ov_bad = check_12word_train_overlap(
        [{"id": "ov1", "text": "intro alpha beta gamma delta epsilon zeta eta theta iota kappa lambda mu outro"}],
        mw_train,
    )
    assert r8_ov_bad["pass"] is False and r8_ov_bad["overlap_count"] == 1
    rep_phrase = "one two three four five six seven eight"
    r8_rep_bad = check_8gram_repetition(
        [{"id": "rep1", "text": " ".join([rep_phrase] * 4)}]
    )
    assert r8_rep_bad["pass"] is False and r8_rep_bad["flagged_docs"] == 1
    record(8, "raw_5word_jaccard_12word_overlap_and_8gram_repetition_helpers", "verified 5-word Jaccard, 12-word train overlap, and >3x 8-gram repetition")

    # [9/32] Valid pilot bundle -> GO_FOR_PRODUCTION_TRANCHE
    pilot = make_valid_pilot_bundle()
    r9 = verify_pilot_bundle(pilot)
    assert r9["decision"] == "GO_FOR_PRODUCTION_TRANCHE" and r9["go_for_production"] is True and r9["d41_pass_consumed"] is False
    record(9, "valid_pilot_bundle_go_decision", f"decision={r9['decision']} at {r9['sustained_tok_per_s']} tok/s")

    # [10/32] Pilot throughput shortfall (< 250 tok/s) -> STOP_THROUGHPUT_SHORTFALL_OWNER_REPLAN
    pilot_slow = copy.deepcopy(pilot)
    pilot_slow["p1_generation"]["sustained_tok_per_s"] = 242.0
    r10 = verify_pilot_bundle(pilot_slow)
    assert r10["decision"] == "STOP_THROUGHPUT_SHORTFALL_OWNER_REPLAN" and r10["go_for_production"] is False and r10["d41_pass_consumed"] is False
    record(10, "pilot_throughput_shortfall_below_250_stops_for_owner_replan", f"decision={r10['decision']} at 242.0 tok/s")

    # [11/32] Pilot throughput gray zone (250 <= tok/s < 300) -> HOLD_BELOW_300_TOK_S_GO_BAR
    pilot_mid = copy.deepcopy(pilot)
    pilot_mid["p1_generation"]["sustained_tok_per_s"] = 285.0
    r11 = verify_pilot_bundle(pilot_mid)
    assert r11["decision"] == "HOLD_BELOW_300_TOK_S_GO_BAR" and r11["go_for_production"] is False and r11["d41_pass_consumed"] is False
    record(11, "pilot_throughput_gray_zone_250_to_300_holds", f"decision={r11['decision']} at 285.0 tok/s")

    # [12/32] Pilot quality gate failure -> PILOT_REJECTED_CORPUS_DEFECT_CONSUMES_D41_PASS
    pilot_bad_q = copy.deepcopy(pilot)
    pilot_bad_q["p7_quality_gates"]["g6_panel_contamination"]["matched_kept_documents"] = 1
    _expect_failure(
        lambda: verify_pilot_bundle(pilot_bad_q),
        expected_classification="PILOT_REJECTED_CORPUS_DEFECT_CONSUMES_D41_PASS",
        expected_gate="G6",
        expected_d41_consumed=True,
    )
    record(12, "pilot_quality_gate_failure_rejects_and_consumes_d41_pass", "pilot G6 defect rejected and marked d41_pass_consumed=True")

    # [13/32] Pilot P4 Python or float in student tokenization -> INSTRUMENT_UNREACHABLE_D31
    pilot_bad_p4 = copy.deepcopy(pilot)
    pilot_bad_p4["p4_rust_tokenization"]["python_used"] = True
    _expect_failure(
        lambda: verify_pilot_bundle(pilot_bad_p4),
        expected_classification="INSTRUMENT_UNREACHABLE_D31",
        expected_gate="P4",
        expected_d41_consumed=False,
    )
    pilot_bad_p4_flt = copy.deepcopy(pilot)
    pilot_bad_p4_flt["p4_rust_tokenization"]["float_used"] = True
    _expect_failure(
        lambda: verify_pilot_bundle(pilot_bad_p4_flt),
        expected_classification="INSTRUMENT_UNREACHABLE_D31",
        expected_gate="P4",
        expected_d41_consumed=False,
    )
    record(13, "pilot_p4_python_or_float_in_tokenization_rejected_d31", "python_used=True and float_used=True rejected in P4")

    # [14/32] Pilot P8 cost cap or unresolved license -> STOP_COST_OR_LICENSE_BLOCKER
    pilot_bad_p8 = copy.deepcopy(pilot)
    pilot_bad_p8["p8_cost_accounting"]["license_or_redistribution_unresolved"] = True
    _expect_failure(
        lambda: verify_pilot_bundle(pilot_bad_p8),
        expected_classification="STOP_COST_OR_LICENSE_BLOCKER",
        expected_gate="P8",
        expected_d41_consumed=False,
    )
    record(14, "pilot_p8_cost_or_rate_cap_violation_stops", "unresolved license/cost cap rejected in P8")

    # [15/32] §4.3 Repair vs Regeneration enforcement
    orig_rec = make_valid_receipt(chk, is_pilot=True)
    repaired_rec = copy.deepcopy(orig_rec)
    new_sh0_sha = sha256_bytes(b"repaired-shard-00-bytes")
    repaired_rec["content_pin"]["shards"][0]["sha256"] = new_sh0_sha
    repaired_rec["readback"]["shards_readback"][0]["sha256"] = new_sh0_sha
    new_merkle = compute_shard_merkle_root(
        [s["sha256"] for s in repaired_rec["content_pin"]["shards"]],
        mode=repaired_rec["content_pin"]["merkle_mode"],
    )
    repaired_rec["content_pin"]["merkle_root"] = new_merkle
    repaired_rec["readback"]["recomputed_merkle_root"] = new_merkle
    r15 = verify_repair_vs_regeneration(orig_rec, repaired_rec)
    assert r15["classification"] == "PERMITTED_SHARD_REPAIR" and r15["consumes_new_pass"] is False
    regen_rec = copy.deepcopy(repaired_rec)
    regen_rec["donor_pin"]["revision"] = "b2c3d4e5f60718293a4b5c6d7e8f90a1b2c3d4e5"
    _expect_failure(
        lambda: verify_repair_vs_regeneration(orig_rec, regen_rec),
        expected_classification="REGENERATION_FORBIDDEN_REQUIRES_NEW_OWNER_DECISION_D41",
        expected_d41_consumed=True,
    )
    record(15, "repair_vs_regeneration_enforcement", "permitted shard repair vs forbidden donor-revision regeneration verified")

    # [16/32] Valid 30M production bundle -> PRODUCTION_CORPUS_SEALED_D41_EXPIRED
    prod = make_valid_production_bundle()
    r16 = verify_production_bundle(prod)
    assert (
        r16["decision"] == "PRODUCTION_CORPUS_SEALED_D41_EXPIRED"
        and r16["corpus_sealed"] is True
        and r16["d41_exception_expired"] is True
        and r16["context_512_status"] == "NOT_RUN_MAX_CONTEXT_243"
        and r16["mechanism_negative"] is False
    )
    record(16, "valid_production_bundle_all_g1_to_g14_pass_and_seal", f"sealed 30M corpus merkle={r16['merkle_root'][:16]}…")

    # [17/32] G1 donor weight or revision mismatch -> INSTRUMENT_UNREACHABLE_D31
    prod_bad_g1 = copy.deepcopy(prod)
    prod_bad_g1["receipt"]["donor_pin"]["weights_sha256"]["combined_sha256"] = sha256_bytes(b"tampered-weights")
    _expect_failure(
        lambda: verify_production_bundle(prod_bad_g1),
        expected_classification="INSTRUMENT_UNREACHABLE_D31",
        expected_gate="G1",
        expected_d41_consumed=False,
    )
    record(17, "g1_donor_weight_or_revision_mismatch_is_d31", "G1 weight SHA mismatch classified as D31 instrument fault")

    # [18/32] G2 receipt incompleteness -> INSTRUMENT_UNREACHABLE_D31
    prod_bad_g2 = copy.deepcopy(prod)
    del prod_bad_g2["receipt"]["run_pin"]["lease_id"]
    _expect_failure(
        lambda: verify_production_bundle(prod_bad_g2),
        expected_classification="INSTRUMENT_UNREACHABLE_D31",
        expected_gate="G2",
        expected_d41_consumed=False,
    )
    record(18, "g2_receipt_incompleteness_is_d31", "G2 missing run_pin.lease_id classified as D31 instrument fault")

    # [19/32] G3 readback or Merkle mismatch -> INSTRUMENT_UNREACHABLE_D31
    prod_bad_g3 = copy.deepcopy(prod)
    prod_bad_g3["receipt"]["readback"]["shards_readback"][2]["sha256"] = sha256_bytes(b"corrupt-transfer-shard-02")
    _expect_failure(
        lambda: verify_production_bundle(prod_bad_g3),
        expected_classification="INSTRUMENT_UNREACHABLE_D31",
        expected_gate="G3",
        expected_d41_consumed=False,
    )
    record(19, "g3_readback_or_merkle_mismatch_is_d31", "G3 shard transfer hash mismatch classified as D31 instrument fault")

    # [20/32] G4 exact dedup > 0 -> CORPUS_REJECTED_CONSUMES_D41_PASS
    prod_bad_g4 = copy.deepcopy(prod)
    prod_bad_g4["quality_gates"]["g4_exact_dedup"]["duplicate_document_sha256_count"] = 2
    _expect_failure(
        lambda: verify_production_bundle(prod_bad_g4),
        expected_classification="CORPUS_REJECTED_CONSUMES_D41_PASS",
        expected_gate="G4",
        expected_d41_consumed=True,
    )
    record(20, "g4_exact_dedup_nonzero_rejects_corpus", "G4 duplicate document SHA rejected and consumes D41 pass")

    # [21/32] G5 near-dedup rate >= 3.0% -> CORPUS_REJECTED_CONSUMES_D41_PASS
    prod_bad_g5 = copy.deepcopy(prod)
    prod_bad_g5["quality_gates"]["g5_near_dedup"]["residual_near_duplicate_rate"] = 0.030
    _expect_failure(
        lambda: verify_production_bundle(prod_bad_g5),
        expected_classification="CORPUS_REJECTED_CONSUMES_D41_PASS",
        expected_gate="G5",
        expected_d41_consumed=True,
    )
    record(21, "g5_near_dedup_rate_at_or_above_3pct_rejects_corpus", "G5 residual near-dedup 3.0% rejected and consumes D41 pass")

    # [22/32] G6 panel contamination > 0 -> CORPUS_REJECTED_CONSUMES_D41_PASS
    prod_bad_g6 = copy.deepcopy(prod)
    prod_bad_g6["quality_gates"]["g6_panel_contamination"]["matched_kept_documents"] = 1
    _expect_failure(
        lambda: verify_production_bundle(prod_bad_g6),
        expected_classification="CORPUS_REJECTED_CONSUMES_D41_PASS",
        expected_gate="G6",
        expected_d41_consumed=True,
    )
    record(22, "g6_panel_contamination_nonzero_rejects_corpus", "G6 panel contamination rejected and consumes D41 pass")

    # [23/32] G7 mw-balp TRAIN 12-word overlap > 0 -> CORPUS_REJECTED_CONSUMES_D41_PASS
    prod_bad_g7 = copy.deepcopy(prod)
    prod_bad_g7["quality_gates"]["g7_train_overlap"]["shared_12word_kept_documents"] = 3
    _expect_failure(
        lambda: verify_production_bundle(prod_bad_g7),
        expected_classification="CORPUS_REJECTED_CONSUMES_D41_PASS",
        expected_gate="G7",
        expected_d41_consumed=True,
    )
    record(23, "g7_mw_balp_train_12word_overlap_nonzero_rejects_corpus", "G7 12-word mw-balp overlap rejected and consumes D41 pass")

    # [24/32] G8 code slice failure vs sandbox fault
    prod_bad_g8 = copy.deepcopy(prod)
    prod_bad_g8["quality_gates"]["g8_code_slice"]["passed_samples"] = 29_999
    _expect_failure(
        lambda: verify_production_bundle(prod_bad_g8),
        expected_classification="CORPUS_REJECTED_CONSUMES_D41_PASS",
        expected_gate="G8",
        expected_d41_consumed=True,
    )
    prod_bad_g8_sbx = copy.deepcopy(prod)
    prod_bad_g8_sbx["quality_gates"]["g8_code_slice"]["sandbox"]["network_disabled"] = False
    _expect_failure(
        lambda: verify_production_bundle(prod_bad_g8_sbx),
        expected_classification="INSTRUMENT_UNREACHABLE_D31",
        expected_gate="G8",
        expected_d41_consumed=False,
    )
    record(24, "g8_code_slice_execution_or_sandbox_failure", "G8 failing code sample -> corpus defect; network-enabled sandbox -> D31")

    # [25/32] G9 math slice answer check or unevaluated sample failure
    prod_bad_g9 = copy.deepcopy(prod)
    prod_bad_g9["quality_gates"]["g9_math_slice"]["unevaluated_samples"] = 1
    _expect_failure(
        lambda: verify_production_bundle(prod_bad_g9),
        expected_classification="CORPUS_REJECTED_CONSUMES_D41_PASS",
        expected_gate="G9",
        expected_d41_consumed=True,
    )
    record(25, "g9_math_slice_answer_check_or_unevaluated_failure", "G9 unevaluated math sample rejected and consumes D41 pass")

    # [26/32] G10 spot audit exceedance & slice-drop renormalization in G11a
    prod_bad_g10 = copy.deepcopy(prod)
    prod_bad_g10["quality_gates"]["g10_spot_audit"]["per_slice"]["reasoning_prose"]["malformed_count"] = 5
    _expect_failure(
        lambda: verify_production_bundle(prod_bad_g10),
        expected_classification="CORPUS_REJECTED_CONSUMES_D41_PASS",
        expected_gate="G10",
        expected_d41_consumed=True,
    )
    prod_bad_redraw = copy.deepcopy(prod)
    prod_bad_redraw["quality_gates"]["g10_spot_audit"]["per_slice"]["conversation"]["redraws_used"] = 2
    _expect_failure(
        lambda: verify_production_bundle(prod_bad_redraw),
        expected_classification="CORPUS_REJECTED_CONSUMES_D41_PASS",
        expected_gate="G10",
        expected_d41_consumed=True,
    )
    prod_drop_slice = copy.deepcopy(prod)
    prod_drop_slice["quality_gates"]["g10_spot_audit"]["dropped_slices"] = ["reasoning_prose"]
    del prod_drop_slice["quality_gates"]["g10_spot_audit"]["per_slice"]["reasoning_prose"]
    prod_drop_slice["quality_gates"]["g11_distribution"]["g11a_slice_shares"]["kept_token_shares"] = {
        "conversation": round(0.35 / 0.85, 6),
        "qa_instruction": round(0.20 / 0.85, 6),
        "code": round(0.20 / 0.85, 6),
        "exposition": round(1.0 - round(0.35 / 0.85, 6) - 2 * round(0.20 / 0.85, 6), 6),
    }
    r26_drop = verify_production_bundle(prod_drop_slice)
    assert r26_drop["quality_gates"]["dropped_slices"] == ["reasoning_prose"]
    record(26, "g10_spot_audit_and_slice_drop_renormalization", "verified G10 malformed/redraw rejection and 4-slice renormalization after slice drop")

    # [27/32] G11a slice share drift > 3.0 pp -> CORPUS_REJECTED_CONSUMES_D41_PASS
    prod_bad_g11a = copy.deepcopy(prod)
    prod_bad_g11a["quality_gates"]["g11_distribution"]["g11a_slice_shares"]["kept_token_shares"] = {
        "conversation": 0.385,
        "qa_instruction": 0.165,
        "code": 0.200,
        "reasoning_prose": 0.150,
        "exposition": 0.100,
    }
    _expect_failure(
        lambda: verify_production_bundle(prod_bad_g11a),
        expected_classification="CORPUS_REJECTED_CONSUMES_D41_PASS",
        expected_gate="G11a",
        expected_d41_consumed=True,
    )
    record(27, "g11a_slice_share_drift_beyond_3pp_rejects_corpus", "G11a +-3.5 pp slice drift rejected and consumes D41 pass")

    # [28/32] G11b truncation rate >= 1.0% -> CORPUS_REJECTED_CONSUMES_D41_PASS
    prod_bad_g11b = copy.deepcopy(prod)
    prod_bad_g11b["quality_gates"]["g11_distribution"]["g11b_truncation"]["truncation_rate"] = 0.011
    _expect_failure(
        lambda: verify_production_bundle(prod_bad_g11b),
        expected_classification="CORPUS_REJECTED_CONSUMES_D41_PASS",
        expected_gate="G11b",
        expected_d41_consumed=True,
    )
    record(28, "g11b_truncation_rate_at_or_above_1pct_rejects_corpus", "G11b 1.1% truncation rate rejected and consumes D41 pass")

    # [29/32] G11c repetition rate >= 2.0% -> CORPUS_REJECTED_CONSUMES_D41_PASS
    prod_bad_g11c = copy.deepcopy(prod)
    prod_bad_g11c["quality_gates"]["g11_distribution"]["g11c_repetition"]["repeated_8gram_doc_rate"] = 0.022
    _expect_failure(
        lambda: verify_production_bundle(prod_bad_g11c),
        expected_classification="CORPUS_REJECTED_CONSUMES_D41_PASS",
        expected_gate="G11c",
        expected_d41_consumed=True,
    )
    record(29, "g11c_repetition_8gram_rate_at_or_above_2pct_rejects_corpus", "G11c 2.2% repeated 8-gram rate rejected and consumes D41 pass")

    # [30/32] G11d length bounds violation -> CORPUS_REJECTED_CONSUMES_D41_PASS
    for bad_field, bad_val in (("min_student_tokens", 12), ("median_student_tokens", 125.0), ("p99_student_tokens", 201.0)):
        prod_bad_g11d = copy.deepcopy(prod)
        prod_bad_g11d["quality_gates"]["g11_distribution"]["g11d_length"][bad_field] = bad_val
        _expect_failure(
            lambda: verify_production_bundle(prod_bad_g11d),
            expected_classification="CORPUS_REJECTED_CONSUMES_D41_PASS",
            expected_gate="G11d",
            expected_d41_consumed=True,
        )
    record(30, "g11d_length_bounds_violation_rejects_corpus", "G11d min<16, median>120, and p99>200 rejected and consume D41 pass")

    # [31/32] G12 & G13 UORT admission, dev power < 250k, or window admission failure -> D31
    prod_bad_dev = copy.deepcopy(prod)
    prod_bad_dev["g12_uort_admission"]["splits"]["dev"]["eligible_targets"] = 249_999
    _expect_failure(
        lambda: verify_production_bundle(prod_bad_dev),
        expected_classification="INSTRUMENT_UNREACHABLE_D31",
        expected_gate="G12",
        expected_d41_consumed=False,
    )
    prod_bad_seal_split = copy.deepcopy(prod)
    prod_bad_seal_split["g12_uort_admission"]["heldout_opened_during_prep_or_gates"] = True
    _expect_failure(
        lambda: verify_production_bundle(prod_bad_seal_split),
        expected_classification="INSTRUMENT_UNREACHABLE_D31",
        expected_gate="G12",
        expected_d41_consumed=False,
    )
    prod_bad_g13 = copy.deepcopy(prod)
    prod_bad_g13["g13_window_admission"]["train_windows_returned"] = 63
    _expect_failure(
        lambda: verify_production_bundle(prod_bad_g13),
        expected_classification="INSTRUMENT_UNREACHABLE_D31",
        expected_gate="G13",
        expected_d41_consumed=False,
    )
    record(31, "g12_and_g13_uort_or_window_admission_or_dev_power_failure", "G12 dev<250k, opened heldout split, and G13 <64 windows rejected as D31")

    # [32/32] G14 unsealed or unexpired D41 exception refused -> INSTRUMENT_UNREACHABLE_D31
    prod_bad_g14 = copy.deepcopy(prod)
    prod_bad_g14["g14_seal"]["d41_exception_expired"] = False
    _expect_failure(
        lambda: verify_production_bundle(prod_bad_g14),
        expected_classification="INSTRUMENT_UNREACHABLE_D31",
        expected_gate="G14",
        expected_d41_consumed=False,
    )
    record(32, "g14_unsealed_or_unexpired_d41_exception_refused", "G14 d41_exception_expired=False refused at seal")

    summary = {
        "self_test": "PASS",
        "passed": len(results),
        "total": 32,
        "repo_contract": r1,
    }
    print(json.dumps({"self_test": "PASS", "passed": len(results), "total": 32}, sort_keys=True))
    return summary


# ---------------------------------------------------------------------------
# CLI Entrypoint
# ---------------------------------------------------------------------------


def main(argv: Sequence[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        description="Host-only consumer verifier for the distill-d1 donor corpus sheet, receipt, pilot gates P1–P8, and production gates G1–G14."
    )
    parser.add_argument(
        "--repo-root",
        type=Path,
        default=Path(__file__).resolve().parents[3],
        help="Path to the repository root (default: 3 levels up from this script).",
    )
    parser.add_argument(
        "--sheet-path",
        type=Path,
        default=None,
        help="Optional override path to the distill-d1 registration sheet.",
    )
    parser.add_argument(
        "--verify-repo-contract",
        action="store_true",
        help="Verify the staged registration sheet and repository source contracts.",
    )
    parser.add_argument(
        "--checklist-json",
        type=Path,
        default=None,
        help="Path to a §8 registration checklist JSON file to verify.",
    )
    parser.add_argument(
        "--receipt-json",
        type=Path,
        default=None,
        help="Path to a uor-r4.donor-corpus/1 generation receipt JSON file to verify.",
    )
    parser.add_argument(
        "--pilot-bundle",
        type=Path,
        default=None,
        help="Path to a pilot bundle JSON file (P1–P8) to verify.",
    )
    parser.add_argument(
        "--production-bundle",
        type=Path,
        default=None,
        help="Path to a production bundle JSON file (G1–G14) to verify.",
    )
    parser.add_argument(
        "--self-test",
        action="store_true",
        help="Run the 32-case positive and negative self-test suite.",
    )
    args = parser.parse_args(argv)

    repo_root = args.repo_root.resolve()
    ran_any = False
    out: dict[str, Any] = {}

    try:
        if args.self_test:
            ran_any = True
            out["self_test_report"] = run_self_tests(repo_root)
        if args.verify_repo_contract:
            ran_any = True
            out["repo_contract"] = verify_sheet_and_repo_contract(repo_root, args.sheet_path)
        if args.checklist_json is not None:
            ran_any = True
            chk_data = json.loads(args.checklist_json.read_text(encoding="utf-8"))
            out["checklist_verification"] = verify_registration_checklist(chk_data)
        if args.receipt_json is not None:
            ran_any = True
            rec_data = json.loads(args.receipt_json.read_text(encoding="utf-8"))
            out["receipt_verification"] = verify_generation_receipt(rec_data)
        if args.pilot_bundle is not None:
            ran_any = True
            pilot_data = json.loads(args.pilot_bundle.read_text(encoding="utf-8"))
            out["pilot_verification"] = verify_pilot_bundle(pilot_data)
        if args.production_bundle is not None:
            ran_any = True
            prod_data = json.loads(args.production_bundle.read_text(encoding="utf-8"))
            out["production_verification"] = verify_production_bundle(prod_data)

        if not ran_any:
            out["repo_contract"] = verify_sheet_and_repo_contract(repo_root, args.sheet_path)
            print(json.dumps(out, indent=2, sort_keys=True))
        elif not args.self_test or len(out) > 1:
            print(json.dumps(out, indent=2, sort_keys=True))
        return 0
    except VerificationError as exc:
        err_payload = {
            "status": "FAIL",
            "classification": exc.classification,
            "gate": exc.gate,
            "d41_pass_consumed": exc.d41_pass_consumed,
            "mechanism_negative": False,
            "error": str(exc),
        }
        print(json.dumps(err_payload, indent=2, sort_keys=True), file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
