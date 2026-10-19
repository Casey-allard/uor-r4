#!/usr/bin/env python3
"""Host-only consumer verifier for BVG-K Stage T learning law, per-family BlockCursor/BlockReceipt
schedule arithmetic, and decode-bench tokens/s baseline instrument contracts.

Pinned repository contracts verified on origin/main:
  1. docs/harness/prep/m1-bvgk-staget-learning-law-DRAFT-2026-10-19.md
     (0d2775e1aa197f0cff577f01f00fc80ae9072cd7cbe11bb79558e35fb0f32e32, 9,884 B, 128 lines)
  2. docs/harness/prep/m1-batched-verified-coordinates-DRAFT-2026-10-18.md
     (c9f49291e57010769ec94678a95b29170c781947b756f9143086370616908e19, 15,181 B, 153 lines)
  3. docs/harness/prep/m1-decode-tokens-per-second-DRAFT-2026-10-18.md
     (5b6e4cdcf30cbe5252af47afd926757bc4aca59ec08ccf3cf747ada729440527, 4,118 B, 73 lines)
  4. crates/uor-r4-discrete/src/training.rs
     (280f4b9533f6686c87a3f8bc5e18ff839c7fc6cea3e39817d86c47c718e110f3, 126,242 B, 2,969 lines)

Host-only verification tooling: zero GPU/pod compute ($0), no model forward/training execution,
and all training holds remain HELD.
"""

from __future__ import annotations

import argparse
import copy
import hashlib
import json
import re
import sys
from pathlib import Path
from typing import Any, Dict, List, Optional, Sequence, Tuple


class VerificationError(Exception):
    """Raised when a Stage T, BVG-K, or decode-bench contract invariant fails."""


# ---------------------------------------------------------------------------
# Pinned Repository Constants
# ---------------------------------------------------------------------------

STAGET_DRAFT_RELPATH = "docs/harness/prep/m1-bvgk-staget-learning-law-DRAFT-2026-10-19.md"
STAGET_DRAFT_SHA256 = "0d2775e1aa197f0cff577f01f00fc80ae9072cd7cbe11bb79558e35fb0f32e32"
STAGET_DRAFT_BYTES = 9884
STAGET_DRAFT_LINES = 128

BVC_DRAFT_RELPATH = "docs/harness/prep/m1-batched-verified-coordinates-DRAFT-2026-10-18.md"
BVC_DRAFT_SHA256 = "c9f49291e57010769ec94678a95b29170c781947b756f9143086370616908e19"
BVC_DRAFT_BYTES = 15181
BVC_DRAFT_LINES = 153

DECODE_DRAFT_RELPATH = "docs/harness/prep/m1-decode-tokens-per-second-DRAFT-2026-10-18.md"
DECODE_DRAFT_SHA256 = "5b6e4cdcf30cbe5252af47afd926757bc4aca59ec08ccf3cf747ada729440527"
DECODE_DRAFT_BYTES = 4118
DECODE_DRAFT_LINES = 73

TRAINING_RS_RELPATH = "crates/uor-r4-discrete/src/training.rs"
TRAINING_RS_SHA256 = "280f4b9533f6686c87a3f8bc5e18ff839c7fc6cea3e39817d86c47c718e110f3"
TRAINING_RS_BYTES = 126242
TRAINING_RS_LINES = 2969

STAGET_REQUIRED_HEADINGS: Tuple[str, ...] = (
    "# BVG-K Stage T: batched verified coordinates as the learning law — DRAFT",
    "## Why now",
    "## Preconditions before registration (all must hold)",
    "## Arms (one variable: `coordinate_block`)",
    "## Bar (axes inherited from parent §5; numbers fixed at registration)",
    "## Pre-registered first-batch controls (moved here from C2's uncovered list)",
    "## Cost (projection, refined at registration)",
    "## Base artifact",
    "## Data split",
    "## Steps",
    "## Stop rule",
    "## Minimum test power",
    "## Open items for registration review",
)

BVC_REQUIRED_HEADINGS: Tuple[str, ...] = (
    "# Batched verified coordinates (BVG-K): score K coordinates per batch, commit a verified set (DRAFT)",
    "## 1. The current law, and the gap",
    "## 2. The law: BVG-K",
    "### 2a. Why moves interact, so verification is needed",
    "### 2b. K = 1 is the current law",
    "## 3. Implementation sketch (for the registration, not this draft)",
    "## 4. Finite controls (to be derived at registration)",
    "## 5. Bar",
    "## 6. Base artifact",
    "## 7. Data split",
    "## 8. Steps (two stages, registered and released separately)",
    "## 9. Stop rule",
    "## 10. Cost",
    "## 11. Minimum test power",
)

DECODE_REQUIRED_HEADINGS: Tuple[str, ...] = (
    "# Decode tokens/s against context — the optimization rung's baseline instrument (DRAFT)",
    "## 1. What is missing",
    "## 2. Instrument",
    "## 3. Bar",
    "## 4. Base artifact",
    "## 5. Data split",
    "## 6. Steps",
    "## 7. Stop rule",
    "## 8. Cost",
    "## 9. Minimum test power",
)

TRAINING_RS_REQUIRED_SYMBOLS: Tuple[str, ...] = (
    "pub struct BlockCursor",
    "pub struct BlockReceipt",
    "pub fn family_cursors(",
    "pub fn block_at(",
    "pub fn advance_block(",
    "pub(crate) fn transition_block(",
    "pub fn run_block_transitions(",
)

TRAINING_RS_REQUIRED_REFUSALS: Tuple[str, ...] = (
    '"coordinate_block must be 1..=64"',
    '"coordinate_block > 1 requires the rows_v1 counterfactual path"',
    '"resume committed_moves outside accepted_updates..=K*accepted_updates"',
    '"coordinate_block 1 cursor carries no block fields"',
    '"coordinate_block > 1 cursor requires block fields"',
    '"coordinate_block > 1 advances through advance_block"',
    '"coordinate_block > 1 schedule is addressed through block_at"',
    '"coordinate_block > 1 runs through run_block_transitions"',
    '"coordinate_block 1 runs through run_transitions"',
    '"resume o_vec/o_read differ from the per-family block schedule"',
    '"block commit needs 1..=block-length moves, and no commit none"',
)

# D22 §3 2-SE panel floors on DEV
D22_FLOOR_TOP1_ROWS = 20
D22_FLOOR_TOP8_ROWS = 63
D22_FLOOR_B3_ROWS = 87

# Dose 1 data split pins
DOSE1_TRAIN_TOKENS_SHA256 = "3dee858f84b750f507ceb3dc3f393711741950954735efaaf61bf704c017a4c8"
DOSE1_DEV_TOKENS_SHA256 = "0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b"
DOSE1_DEV_MASK_SHA256 = "a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078"
DOSE1_DEV_ROWS = 14287
DOSE1_PSUCC_ROWS = 3847

# Decode-bench grid constants
DECODE_CONTEXT_LENGTHS: Tuple[int, ...] = (16, 32, 64, 128, 192, 243)
DECODE_BATCH_SHAPES: Tuple[Tuple[str, int], ...] = (
    ("single_stream", 1),
    ("batched_64", 64),
)
DECODE_CHECKPOINTS: Tuple[str, ...] = ("treatment", "frozen_zero")
DECODE_GENERATED_TOKENS_N = 256
DECODE_TIMED_REPEATS = 5

HEX64_RE = re.compile(r"^[0-9a-f]{64}$")


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------


def _sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def _require_hex64(value: Any, label: str) -> str:
    if not isinstance(value, str) or not HEX64_RE.match(value):
        raise VerificationError(f"{label} must be a 64-character lowercase hex SHA-256, got {value!r}")
    return value


def _read_checked_file(
    root: Path,
    relpath: str,
    expected_sha256: Optional[str] = None,
    expected_bytes: Optional[int] = None,
    expected_lines: Optional[int] = None,
) -> Tuple[bytes, str, int, int]:
    path = root / relpath
    if not path.is_file():
        raise VerificationError(f"missing required file: {path}")
    raw = path.read_bytes()
    digest = _sha256_bytes(raw)
    n_bytes = len(raw)
    n_lines = raw.count(b"\n")
    if expected_sha256 is not None and digest != expected_sha256:
        raise VerificationError(
            f"{relpath} SHA-256 mismatch: expected {expected_sha256}, got {digest}"
        )
    if expected_bytes is not None and n_bytes != expected_bytes:
        raise VerificationError(
            f"{relpath} byte count mismatch: expected {expected_bytes}, got {n_bytes}"
        )
    if expected_lines is not None and n_lines != expected_lines:
        raise VerificationError(
            f"{relpath} line count mismatch: expected {expected_lines}, got {n_lines}"
        )
    return raw, digest, n_bytes, n_lines


# ---------------------------------------------------------------------------
# 1. Repository & Staged Sheet Contract Verifier
# ---------------------------------------------------------------------------


def parse_sheet_fitness_status(text: str, label: str = "sheet") -> Dict[str, Any]:
    """Verify top-of-sheet status and bottom TEST FITNESS line are mutually consistent."""
    non_empty_lines = [line.strip() for line in text.splitlines() if line.strip()]
    if not non_empty_lines:
        raise VerificationError(f"{label}: empty sheet")
    bottom_line = non_empty_lines[-1]
    if not bottom_line.startswith("TEST FITNESS:"):
        raise VerificationError(
            f"{label}: final non-empty line must start with 'TEST FITNESS:', got {bottom_line!r}"
        )
    bottom_verdict = bottom_line.split("TEST FITNESS:", 1)[1].strip().rstrip(".")
    if bottom_verdict not in {"PENDING", "FIT"}:
        raise VerificationError(
            f"{label}: unsupported bottom TEST FITNESS verdict {bottom_verdict!r}"
        )

    header_region = "\n".join(non_empty_lines[:15])
    top_has_fit = bool(re.search(r"TEST FITNESS:\s*FIT\b", header_region))
    top_has_pending_or_not_requested = (
        "TEST FITNESS: PENDING" in header_region
        or "TEST FITNESS is not requested" in header_region
        or "Not registered" in header_region
        or "DRAFT" in non_empty_lines[0]
    )

    if bottom_verdict == "FIT":
        if not top_has_fit or "Not registered" in header_region:
            raise VerificationError(
                f"{label}: bottom line claims TEST FITNESS: FIT, but top header does not match"
            )
        return {
            "status": "REGISTERED_FIT",
            "top_fit": True,
            "bottom_fit": True,
            "consistent": True,
        }

    # bottom_verdict == "PENDING"
    if top_has_fit or not top_has_pending_or_not_requested:
        raise VerificationError(
            f"{label}: bottom line is TEST FITNESS: PENDING, but top header is inconsistent"
        )
    return {
        "status": "DRAFT_PENDING",
        "top_fit": False,
        "bottom_fit": False,
        "consistent": True,
    }


def verify_staget_draft_repairs(staget_text: str) -> Dict[str, Any]:
    """Verify all 5 #2029 comment 6257754797 Stage-T draft repairs in m1-bvgk-staget-learning-law-DRAFT."""
    # Repair 1: <= 4-pod wave plan (2 pods x 2 GPUs = 4 slots, Wave 1 T1+T16, Wave 2 optional T4 on 1 pod)
    r1_required = (
        "**Wave plan within the caps** (≤ 4 pods, ≤ $8/h). Using 2 pods × 2 GPUs (4 slots, about $3.8/h)",
        "**Wave 1:** T1 and T16 on seeds 1 and 2, four arm-seeds at once.",
        "**Wave 2 (only if T4 is registered):** T4 on seeds 1 and 2, on 1 pod.",
    )
    r1_forbidden = ("3 pod-pairs", "6 pods")
    for snippet in r1_required:
        if snippet not in staget_text:
            raise VerificationError(f"Stage T draft repair 1 (<=4-pod wave plan) missing: {snippet!r}")
    for forbidden in r1_forbidden:
        if forbidden in staget_text:
            raise VerificationError(
                f"Stage T draft repair 1 violated by forbidden phrase: {forbidden!r}"
            )

    # Repair 2: Base-lineage per-batch time explicitly labeled "not measured", C1/C2 scoped to K16 Route shape
    r2_required = (
        "The only measured per-call numbers are at the **K16 Route shape**:",
        "C1: 125.5 ms for one proposal; 123.6 ms for 16 coordinates, scoring only.",
        "C2 TP: 129.7 ms at K = 1 and 259 ms at K = 16 including the joint verify, about 8× per coordinate.",
        "The base lineage's per-batch time is **not measured**.",
        "The 1.31 s/batch figure is the injective sheet's planning assumption, not a measurement.",
    )
    for snippet in r2_required:
        if snippet not in staget_text:
            raise VerificationError(
                f"Stage T draft repair 2 (unmeasured base batch time vs K16 Route C1/C2) missing: {snippet!r}"
            )

    # Repair 3: Precondition 3 two-seed dispersion requirement
    r3_required = (
        "It is fixed from a **measured two-seed dispersion** of the chosen base at matched wall clock, on DEV `P_succ` conditional top-1 and full DEV loss.",
        "If the base has only one seed, as the injective runbook does, registration must first run a second-seed T1, at its own cost line. Alternatively, it may adopt a named measured two-seed dispersion from the parent lineage, with the reason stated.",
        "It is never set from a single seed, and never after any Stage T result is visible.",
    )
    for snippet in r3_required:
        if snippet not in staget_text:
            raise VerificationError(
                f"Stage T draft repair 3 (two-seed dispersion precondition) missing: {snippet!r}"
            )

    # Repair 4: Equal-wall-clock W start/stop, DEV eval & checkpoint I/O inclusion, last checkpoint <= W, Q1 at W/4
    r4_required = (
        "**Primary axis: equal training wall clock, `W`, fixed at registration.**",
        "Each arm's clock starts at its first training batch, after admission and the checkpoint-0 evaluation, and stops at elapsed `W`. Build, table load and pre-flight are excluded. DEV evaluations and checkpoint I/O are **included** in every arm's own clock, so T16 and T4 pay for their own evaluations.",
        "Each arm is read at its last completed checkpoint at or before `W`.",
        "The base lineage's Q1 stop maps to `W/4` on the same axis.",
    )
    for snippet in r4_required:
        if snippet not in staget_text:
            raise VerificationError(
                f"Stage T draft repair 4 (equal-wall-clock W protocol) missing: {snippet!r}"
            )

    # Repair 5: Quality metrics & KEEP / non-inferiority bars
    r5_required = (
        "**Quality metrics.** **Primary:** DEV `P_succ` conditional top-1. **Co-primary:** full DEV loss. **Reported, not gating:** top-8 and B3.",
        "**KEEP for T16** requires, on both seeds, primary top-1 better than T1 beyond the margin **and** full DEV loss non-inferior. Fresh transfer must be reported (D22).",
        "**Non-inferiority:** neither primary metric may trail T1 by more than the margin at `W`.",
        "never narrower than the panel's 2-SE row floors (D22 §3): 20 rows for top-1, 63 for top-8 and 87 for B3.",
    )
    for snippet in r5_required:
        if snippet not in staget_text:
            raise VerificationError(
                f"Stage T draft repair 5 (quality metrics & KEEP/non-inferiority bars) missing: {snippet!r}"
            )

    return {
        "repair_1_wave_plan_within_4_pod_cap": "PASS",
        "repair_2_base_batch_time_not_measured_scoped_c1_c2": "PASS",
        "repair_3_precondition_3_two_seed_dispersion": "PASS",
        "repair_4_equal_wall_clock_w_protocol": "PASS",
        "repair_5_quality_metrics_and_keep_bars": "PASS",
    }


def verify_repo_and_sheet_contracts(repo_root: Path) -> Dict[str, Any]:
    """Verify exact SHA-256, byte/line counts, headings, symbols, and 6257754797 repairs on repo_root."""
    staget_raw, staget_sha, staget_bytes, staget_lines = _read_checked_file(
        repo_root,
        STAGET_DRAFT_RELPATH,
        expected_sha256=STAGET_DRAFT_SHA256,
        expected_bytes=STAGET_DRAFT_BYTES,
        expected_lines=STAGET_DRAFT_LINES,
    )
    bvc_raw, bvc_sha, bvc_bytes, bvc_lines = _read_checked_file(
        repo_root,
        BVC_DRAFT_RELPATH,
        expected_sha256=BVC_DRAFT_SHA256,
        expected_bytes=BVC_DRAFT_BYTES,
        expected_lines=BVC_DRAFT_LINES,
    )
    decode_raw, decode_sha, decode_bytes, decode_lines = _read_checked_file(
        repo_root,
        DECODE_DRAFT_RELPATH,
        expected_sha256=DECODE_DRAFT_SHA256,
        expected_bytes=DECODE_DRAFT_BYTES,
        expected_lines=DECODE_DRAFT_LINES,
    )
    training_raw, training_sha, training_bytes, training_lines = _read_checked_file(
        repo_root,
        TRAINING_RS_RELPATH,
        expected_sha256=TRAINING_RS_SHA256,
        expected_bytes=TRAINING_RS_BYTES,
        expected_lines=TRAINING_RS_LINES,
    )

    staget_text = staget_raw.decode("utf-8")
    bvc_text = bvc_raw.decode("utf-8")
    decode_text = decode_raw.decode("utf-8")
    training_text = training_raw.decode("utf-8")

    for heading in STAGET_REQUIRED_HEADINGS:
        if heading not in staget_text:
            raise VerificationError(f"{STAGET_DRAFT_RELPATH} missing heading: {heading!r}")
    for heading in BVC_REQUIRED_HEADINGS:
        if heading not in bvc_text:
            raise VerificationError(f"{BVC_DRAFT_RELPATH} missing heading: {heading!r}")
    for heading in DECODE_REQUIRED_HEADINGS:
        if heading not in decode_text:
            raise VerificationError(f"{DECODE_DRAFT_RELPATH} missing heading: {heading!r}")

    staget_fitness = parse_sheet_fitness_status(staget_text, STAGET_DRAFT_RELPATH)
    bvc_fitness = parse_sheet_fitness_status(bvc_text, BVC_DRAFT_RELPATH)
    decode_fitness = parse_sheet_fitness_status(decode_text, DECODE_DRAFT_RELPATH)

    for sym in TRAINING_RS_REQUIRED_SYMBOLS:
        if sym not in training_text:
            raise VerificationError(f"{TRAINING_RS_RELPATH} missing symbol: {sym!r}")
    for refusal in TRAINING_RS_REQUIRED_REFUSALS:
        if refusal not in training_text:
            raise VerificationError(f"{TRAINING_RS_RELPATH} missing refusal string: {refusal}")

    repairs = verify_staget_draft_repairs(staget_text)

    return {
        "status": "PASS",
        "files": {
            STAGET_DRAFT_RELPATH: {
                "sha256": staget_sha,
                "bytes": staget_bytes,
                "lines": staget_lines,
                "fitness": staget_fitness,
            },
            BVC_DRAFT_RELPATH: {
                "sha256": bvc_sha,
                "bytes": bvc_bytes,
                "lines": bvc_lines,
                "fitness": bvc_fitness,
            },
            DECODE_DRAFT_RELPATH: {
                "sha256": decode_sha,
                "bytes": decode_bytes,
                "lines": decode_lines,
                "fitness": decode_fitness,
            },
            TRAINING_RS_RELPATH: {
                "sha256": training_sha,
                "bytes": training_bytes,
                "lines": training_lines,
                "verified_symbols": len(TRAINING_RS_REQUIRED_SYMBOLS),
                "verified_refusals": len(TRAINING_RS_REQUIRED_REFUSALS),
            },
        },
        "staget_draft_repairs_6257754797": repairs,
    }


# ---------------------------------------------------------------------------
# 2. Exact Rust Schedule & BlockReceipt / BlockCursor Verifier
# ---------------------------------------------------------------------------


def _div_ceil(a: int, b: int) -> int:
    return (a + b - 1) // b


def _normalize_schedule(schedule: str) -> str:
    if schedule in {
        "interleaved_read_every8_v1",
        "interleaved_read_every_8_v1",
        "InterleavedReadEvery8V1",
    }:
        return "interleaved_read_every_8_v1"
    if schedule in {"canonical_vector_then_read_v1", "CanonicalVectorThenReadV1"}:
        return "canonical_vector_then_read_v1"
    raise VerificationError(f"unsupported schedule policy: {schedule!r}")


def family_cursors(
    completed_batches: int,
    block: int,
    schedule: str,
    vectors: int,
    reads: int,
) -> Tuple[int, int]:
    """Exact Python replica of crates/uor-r4-discrete/src/training.rs::family_cursors."""
    if completed_batches < 0:
        raise VerificationError("completed_batches must be non-negative")
    if block <= 0 or vectors <= 0 or reads <= 0:
        raise VerificationError("block schedule needs K and both families nonzero")
    sched = _normalize_schedule(schedule)
    k = int(block)
    vector_blocks = _div_ceil(vectors, k)
    read_blocks = _div_ceil(reads, k)

    if sched == "interleaved_read_every_8_v1":
        vector_batches = completed_batches - (completed_batches // 8)
        read_batches = completed_batches // 8
    else:
        cycle = vector_blocks + read_blocks
        cycles = completed_batches // cycle
        phase = completed_batches % cycle
        vector_batches = cycles * vector_blocks + min(phase, vector_blocks)
        read_batches = cycles * read_blocks + max(0, phase - vector_blocks)

    def _ordinal(batches: int, blocks: int, size: int) -> int:
        return (batches // blocks) * size + (batches % blocks) * k

    return (
        _ordinal(vector_batches, vector_blocks, vectors),
        _ordinal(read_batches, read_blocks, reads),
    )


def is_read_batch(
    completed_batches: int,
    block: int,
    schedule: str,
    vectors: int,
    reads: int,
) -> bool:
    """Exact Python replica of PreparedDriver::is_read_batch."""
    sched = _normalize_schedule(schedule)
    if sched == "interleaved_read_every_8_v1":
        return (completed_batches % 8) == 7
    vector_blocks = _div_ceil(vectors, block)
    read_blocks = _div_ceil(reads, block)
    return (completed_batches % (vector_blocks + read_blocks)) >= vector_blocks


def coordinate_at_k1(
    completed_batches: int,
    schedule: str,
    vectors: int,
    reads: int,
    read_spec: str = "gate",
) -> Dict[str, Any]:
    """Exact Python replica of PreparedDriver::coordinate_at at K = 1."""
    sched = _normalize_schedule(schedule)
    read_kind = "route" if read_spec == "route" else "gate"
    if sched == "canonical_vector_then_read_v1":
        total = vectors + reads
        idx = completed_batches % total
        if idx < vectors:
            return {"kind": "vector_block", "index": idx}
        return {"kind": read_kind, "index": idx - vectors}
    if (completed_batches % 8) == 7:
        return {"kind": read_kind, "index": (completed_batches // 8) % reads}
    return {
        "kind": "vector_block",
        "index": (completed_batches - (completed_batches // 8)) % vectors,
    }


def block_at(
    completed_batches: int,
    block: int,
    schedule: str,
    vectors: int,
    reads: int,
    read_spec: str = "gate",
) -> Dict[str, Any]:
    """Exact Python replica of PreparedDriver::block_at (training.rs:665-693)."""
    o_vec, o_read = family_cursors(completed_batches, block, schedule, vectors, reads)
    if is_read_batch(completed_batches, block, schedule, vectors, reads):
        kind = "route" if read_spec == "route" else "gate"
        ordinal = o_read
        size = reads
    else:
        kind = "vector_block"
        ordinal = o_vec
        size = vectors
    start = ordinal % size
    end = min(size, start + block)
    indices = list(range(start, end))
    if not indices or len(indices) > block or end > size:
        raise VerificationError("block_at produced invalid coordinate slice")
    return {
        "kind": kind,
        "indices": indices,
        "o_vec": o_vec,
        "o_read": o_read,
        "start": start,
        "end": end,
        "family_size": size,
    }


def initial_cursor_for(coordinate_block: int) -> Dict[str, Any]:
    """Exact Python replica of training.rs::initial_cursor_for."""
    if not (1 <= coordinate_block <= 64):
        raise VerificationError("coordinate_block must be 1..=64")
    cursor: Dict[str, Any] = {
        "completed_batches": 0,
        "accepted_updates": 0,
        "corpus_pass": 0,
        "next_target": 0,
        "next_coordinate": 0,
    }
    if coordinate_block > 1:
        cursor["block"] = {
            "o_vec": 0,
            "o_read": 0,
            "committed_moves": 0,
        }
    return cursor


def validate_cursor(
    cursor: Dict[str, Any],
    coordinate_block: int,
    schedule: str,
    vectors: int,
    reads: int,
    train_batches_per_pass: int,
    batch_rows: int,
    schedule_cycle: int,
) -> None:
    """Exact Python replica of PreparedDriver::validate_cursor (training.rs:595-637)."""
    if not (1 <= coordinate_block <= 64):
        raise VerificationError("coordinate_block must be 1..=64")
    completed = int(cursor["completed_batches"])
    accepted = int(cursor["accepted_updates"])
    corpus_pass = int(cursor["corpus_pass"])
    next_target = int(cursor["next_target"])
    next_coordinate = int(cursor["next_coordinate"])

    if (
        accepted > completed
        or corpus_pass != completed // train_batches_per_pass
        or next_target != (completed % train_batches_per_pass) * batch_rows
        or next_coordinate != (completed % schedule_cycle)
    ):
        raise VerificationError("resume cursor differs from declared corpus/schedule cycle")

    block_field = cursor.get("block")
    if coordinate_block == 1:
        if block_field is not None:
            raise VerificationError("coordinate_block 1 cursor carries no block fields")
        return

    if block_field is None:
        raise VerificationError("coordinate_block > 1 cursor requires block fields")

    expected_o_vec, expected_o_read = family_cursors(
        completed, coordinate_block, schedule, vectors, reads
    )
    if (int(block_field["o_vec"]), int(block_field["o_read"])) != (expected_o_vec, expected_o_read):
        raise VerificationError("resume o_vec/o_read differ from the per-family block schedule")

    committed_moves = int(block_field["committed_moves"])
    if committed_moves < accepted or committed_moves > coordinate_block * accepted:
        raise VerificationError(
            "resume committed_moves outside accepted_updates..=K*accepted_updates"
        )


def advance_block_cursor(
    cursor: Dict[str, Any],
    committed: bool,
    moves: int,
    coordinate_block: int,
    schedule: str,
    vectors: int,
    reads: int,
    train_batches_per_pass: int,
    batch_rows: int,
    schedule_cycle: int,
    read_spec: str = "gate",
) -> Dict[str, Any]:
    """Exact Python replica of PreparedDriver::advance_block (training.rs:697-717)."""
    validate_cursor(
        cursor,
        coordinate_block,
        schedule,
        vectors,
        reads,
        train_batches_per_pass,
        batch_rows,
        schedule_cycle,
    )
    blk = block_at(
        int(cursor["completed_batches"]),
        coordinate_block,
        schedule,
        vectors,
        reads,
        read_spec,
    )
    length = len(blk["indices"])
    if committed != (moves > 0) or moves < 0 or moves > length:
        raise VerificationError("block commit needs 1..=block-length moves, and no commit none")
    completed = int(cursor["completed_batches"]) + 1
    accepted = int(cursor["accepted_updates"]) + (1 if committed else 0)
    next_cursor: Dict[str, Any] = {
        "completed_batches": completed,
        "accepted_updates": accepted,
        "corpus_pass": completed // train_batches_per_pass,
        "next_target": (completed % train_batches_per_pass) * batch_rows,
        "next_coordinate": completed % schedule_cycle,
    }
    if coordinate_block > 1:
        o_vec, o_read = family_cursors(completed, coordinate_block, schedule, vectors, reads)
        next_cursor["block"] = {
            "o_vec": o_vec,
            "o_read": o_read,
            "committed_moves": int(cursor["block"]["committed_moves"]) + moves,
        }
    return next_cursor


def verify_family_cursors_and_block_receipts(
    receipts: Sequence[Dict[str, Any]],
    coordinate_block: int,
    schedule: str,
    vectors: int,
    reads: int,
    train_batches_per_pass: int,
    batch_rows: int,
    schedule_cycle: int,
    read_spec: str = "gate",
    relation_learning_policy: str = "learn_v1",
    combine_gate_count: int = 20,
) -> Dict[str, Any]:
    """Validate a contiguous sequence of BlockReceipt records against training.rs invariants."""
    if not (1 <= coordinate_block <= 64):
        raise VerificationError("coordinate_block must be 1..=64")
    if not receipts:
        raise VerificationError("receipts sequence cannot be empty")

    total_moves = 0
    joint_commits = 0
    single_commits = 0
    noop_batches = 0

    for idx, receipt in enumerate(receipts):
        if int(receipt.get("k", -1)) != coordinate_block:
            raise VerificationError(
                f"receipt[{idx}]: k={receipt.get('k')} does not match coordinate_block={coordinate_block}"
            )
        cursor_before = receipt.get("cursor_before")
        cursor_after = receipt.get("cursor_after")
        if not isinstance(cursor_before, dict) or not isinstance(cursor_after, dict):
            raise VerificationError(f"receipt[{idx}]: missing cursor_before or cursor_after")

        validate_cursor(
            cursor_before,
            coordinate_block,
            schedule,
            vectors,
            reads,
            train_batches_per_pass,
            batch_rows,
            schedule_cycle,
        )
        completed = int(cursor_before["completed_batches"])
        if int(receipt.get("completed_batches", -1)) != completed:
            raise VerificationError(
                f"receipt[{idx}]: completed_batches mismatch ({receipt.get('completed_batches')} != {completed})"
            )

        blk = block_at(completed, coordinate_block, schedule, vectors, reads, read_spec)
        if receipt.get("kind") != blk["kind"]:
            raise VerificationError(
                f"receipt[{idx}]: kind={receipt.get('kind')!r} != scheduled {blk['kind']!r}"
            )
        if int(receipt.get("o_vec", -1)) != blk["o_vec"] or int(receipt.get("o_read", -1)) != blk["o_read"]:
            raise VerificationError(
                f"receipt[{idx}]: (o_vec, o_read)=({receipt.get('o_vec')}, {receipt.get('o_read')}) "
                f"!= scheduled ({blk['o_vec']}, {blk['o_read']})"
            )

        block_length = len(blk["indices"])
        s_len = int(receipt.get("s_len", -1))
        if not (0 <= s_len <= block_length):
            raise VerificationError(
                f"receipt[{idx}]: s_len={s_len} outside 0..={block_length}"
            )

        commit_ordinals = receipt.get("commit_ordinals")
        if not isinstance(commit_ordinals, list):
            raise VerificationError(f"receipt[{idx}]: commit_ordinals must be a list")
        moves = len(commit_ordinals)
        for ord_pos, ord_val in enumerate(commit_ordinals):
            if not isinstance(ord_val, int) or not (0 <= ord_val < block_length):
                raise VerificationError(
                    f"receipt[{idx}]: commit_ordinals[{ord_pos}]={ord_val!r} outside 0..{block_length}"
                )
            if ord_pos > 0 and ord_val <= commit_ordinals[ord_pos - 1]:
                raise VerificationError(
                    f"receipt[{idx}]: commit_ordinals must be strictly ascending, got {commit_ordinals}"
                )
            if relation_learning_policy == "frozen_zero_v1" and blk["kind"] == "gate":
                coord_index = blk["indices"][ord_val]
                if coord_index >= combine_gate_count:
                    raise VerificationError(
                        f"receipt[{idx}]: FrozenZeroV1 committed relation gate index {coord_index}"
                    )

        l_before = int(receipt["l_before"])
        l_best = int(receipt["l_best"])
        l_joint = receipt.get("l_joint")
        committed_loss = int(receipt["committed_loss"])
        joint = bool(receipt.get("joint", False))

        if s_len == 0:
            if l_joint is not None:
                raise VerificationError(f"receipt[{idx}]: l_joint must be absent when s_len == 0")
            if joint or moves != 0 or committed_loss != l_before:
                raise VerificationError(
                    f"receipt[{idx}]: s_len == 0 requires joint=False, moves=0, committed_loss==l_before"
                )
            noop_batches += 1
        elif s_len == 1:
            if l_joint is not None:
                raise VerificationError(f"receipt[{idx}]: l_joint must be absent when s_len == 1")
            if joint or moves != 1:
                raise VerificationError(
                    f"receipt[{idx}]: s_len == 1 requires joint=False and moves=1"
                )
            if not (l_best < l_before and committed_loss == l_best):
                raise VerificationError(
                    f"receipt[{idx}]: s_len == 1 requires committed_loss == l_best < l_before"
                )
            single_commits += 1
        else:
            if l_joint is None:
                raise VerificationError(f"receipt[{idx}]: l_joint is required when s_len >= 2")
            l_joint_int = int(l_joint)
            if not (l_best < l_before):
                raise VerificationError(
                    f"receipt[{idx}]: s_len >= 2 requires l_best ({l_best}) < l_before ({l_before})"
                )
            # Strict improvement required for joint commit: L(S_t) < L({best})
            if l_joint_int < l_best:
                if not joint or moves != s_len or committed_loss != l_joint_int:
                    raise VerificationError(
                        f"receipt[{idx}]: l_joint ({l_joint_int}) < l_best ({l_best}) requires "
                        f"joint=True, moves==s_len ({s_len}), committed_loss==l_joint"
                    )
                joint_commits += 1
            else:
                if joint or moves != 1 or committed_loss != l_best:
                    raise VerificationError(
                        f"receipt[{idx}]: l_joint ({l_joint_int}) >= l_best ({l_best}) must fall back "
                        f"to single best (joint=False, moves=1, committed_loss==l_best)"
                    )
                single_commits += 1

        committed = moves > 0
        if committed and not (committed_loss < l_before):
            raise VerificationError(
                f"receipt[{idx}]: committed batch must strictly lower loss ({committed_loss} vs {l_before})"
            )

        expected_after = advance_block_cursor(
            cursor_before,
            committed=committed,
            moves=moves,
            coordinate_block=coordinate_block,
            schedule=schedule,
            vectors=vectors,
            reads=reads,
            train_batches_per_pass=train_batches_per_pass,
            batch_rows=batch_rows,
            schedule_cycle=schedule_cycle,
            read_spec=read_spec,
        )
        if cursor_after != expected_after:
            raise VerificationError(
                f"receipt[{idx}]: cursor_after {cursor_after} != expected {expected_after}"
            )
        if int(receipt.get("model_epoch", -1)) != int(cursor_after["accepted_updates"]):
            raise VerificationError(
                f"receipt[{idx}]: model_epoch ({receipt.get('model_epoch')}) != "
                f"accepted_updates ({cursor_after['accepted_updates']})"
            )
        if idx > 0 and cursor_before != receipts[idx - 1]["cursor_after"]:
            raise VerificationError(
                f"receipt[{idx}]: cursor_before does not chain from receipt[{idx - 1}].cursor_after"
            )
        total_moves += moves

    final_after = receipts[-1]["cursor_after"]
    if coordinate_block > 1:
        initial_moves = int(receipts[0]["cursor_before"]["block"]["committed_moves"])
        if int(final_after["block"]["committed_moves"]) - initial_moves != total_moves:
            raise VerificationError("cumulative committed_moves mismatch across receipt sequence")

    return {
        "status": "PASS",
        "batches_verified": len(receipts),
        "total_committed_moves": total_moves,
        "joint_commits": joint_commits,
        "single_commits": single_commits,
        "noop_batches": noop_batches,
    }


# ---------------------------------------------------------------------------
# 3. Stage T Preconditions, Registration Gate & First-Batch Controls Verifier
# ---------------------------------------------------------------------------


def verify_staget_bundle(bundle: Dict[str, Any]) -> Dict[str, Any]:
    """Verify a Stage T registration + execution bundle against m1-bvgk-staget-learning-law-DRAFT."""
    # Precondition 1: C2 device-level qualification PASS (#2029 6257768576)
    pre1 = bundle.get("precondition_1_c2_qualification")
    if not isinstance(pre1, dict) or not pre1.get("c2_qualification_pass", False):
        raise VerificationError("Precondition 1 failed: C2 device-level qualification must PASS")
    if str(pre1.get("c2_comment_id", "")) != "6257768576" or int(pre1.get("c2_controls_passed", 0)) != 5:
        raise VerificationError(
            "Precondition 1 failed: must bind #2029 comment 6257768576 with 5/5 C2 controls PASS"
        )

    # Precondition 2: Base lineage reaches admission (pre-flight B3_init >= 500 and Q1 pass)
    pre2 = bundle.get("precondition_2_base_lineage")
    if not isinstance(pre2, dict):
        raise VerificationError("Precondition 2 missing: base_lineage block required")
    if not pre2.get("preflight_passed", False) or int(pre2.get("b3_init_psucc", 0)) < 500:
        return {
            "status": "HELD",
            "verdict": "STAGE_T_HELD_PRECONDITION_2_BASE_LINEAGE_NOT_ADMITTED",
            "reason": "base lineage did not pass pre-flight B3_init >= 500 gate",
            "mechanism_negative": False,
        }
    if not pre2.get("q1_passed", False) or pre2.get("q1_admission_collapse", False):
        return {
            "status": "HELD",
            "verdict": "STAGE_T_HELD_PRECONDITION_2_BASE_LINEAGE_NOT_ADMITTED",
            "reason": "base lineage did not pass Q1 admission gate",
            "mechanism_negative": False,
        }

    base_config = pre2.get("base_config")
    if not isinstance(base_config, dict):
        raise VerificationError("Precondition 2: base_config dictionary is required")
    if base_config.get("counterfactual_path") != "rows_v1":
        raise VerificationError(
            "Precondition 2: Stage T base_config requires counterfactual_path == 'rows_v1'"
        )
    if base_config.get("objective_policy", "nearest_address_margin_v1") != "nearest_address_margin_v1":
        raise VerificationError(
            "Precondition 2: Stage T re-bases on objective A (nearest_address_margin_v1)"
        )

    # Precondition 3: Numeric non-inferiority margin from measured two-seed dispersion + D22 floors
    pre3 = bundle.get("precondition_3_margin")
    if not isinstance(pre3, dict):
        raise VerificationError("Precondition 3 missing: margin specification required")
    if not pre3.get("fixed_before_results", False):
        raise VerificationError(
            "Precondition 3 violated: margin is never set after any Stage T result is visible"
        )
    if int(pre3.get("source_seeds_count", 0)) < 2:
        raise VerificationError(
            "Precondition 3 violated: margin is never set from a single seed (needs >= 2 seeds)"
        )
    margin_source = pre3.get("margin_source")
    if margin_source not in {"base_lineage_two_seed_t1", "named_parent_lineage_two_seed_dispersion"}:
        raise VerificationError(
            f"Precondition 3 violated: invalid margin_source {margin_source!r}"
        )
    if margin_source == "named_parent_lineage_two_seed_dispersion" and not str(
        pre3.get("reason_stated", "")
    ).strip():
        raise VerificationError(
            "Precondition 3 violated: adopting parent lineage two-seed dispersion requires reason_stated"
        )

    margin_top1 = int(pre3.get("margin_top1_rows", 0))
    margin_top8 = int(pre3.get("margin_top8_rows", 0))
    margin_b3 = int(pre3.get("margin_b3_rows", 0))
    margin_loss = float(pre3.get("margin_full_dev_loss", 0.0))
    if (
        margin_top1 < D22_FLOOR_TOP1_ROWS
        or margin_top8 < D22_FLOOR_TOP8_ROWS
        or margin_b3 < D22_FLOOR_B3_ROWS
        or margin_loss <= 0.0
    ):
        raise VerificationError(
            f"Precondition 3 violated: margins ({margin_top1}, {margin_top8}, {margin_b3}, {margin_loss}) "
            f"narrower than D22 §3 2-SE floors ({D22_FLOOR_TOP1_ROWS}, {D22_FLOOR_TOP8_ROWS}, {D22_FLOOR_B3_ROWS}, >0)"
        )

    # Registration, D40 release, and resource caps
    reg = bundle.get("registration_and_release")
    if not isinstance(reg, dict):
        raise VerificationError("registration_and_release block required")
    if reg.get("sheet_verdict") != "FIT":
        raise VerificationError("Stage T requires registered sheet_verdict == 'FIT'")
    if not reg.get("reviewer_subagent_finding_comment_ids"):
        raise VerificationError(
            "Stage T requires fresh-context adversarial reviewer subagent findings on #2029"
        )
    if not str(reg.get("pm_dispatch_id", "")).strip():
        raise VerificationError("Stage T requires PM dispatch before execution")
    if int(reg.get("pods_concurrent", 99)) > 4:
        raise VerificationError("Resource cap violated: pods_concurrent must be <= 4")
    if float(reg.get("rate_usd_per_hr", 99.0)) > 8.0:
        raise VerificationError("Resource cap violated: rate_usd_per_hr must be <= 8.0")
    if float(reg.get("disk_free_gib", 0.0)) < 30.0:
        raise VerificationError("Disk floor violated: disk_free_gib must be >= 30.0")

    releases = reg.get("d40_releases", {})
    if not isinstance(releases, dict) or not releases:
        raise VerificationError("Stage T requires per-arm Casey D40 releases")
    for arm_key, rel in releases.items():
        if rel.get("owner") != "Casey" or not rel.get("uor_lab_env_unset", False):
            raise VerificationError(
                f"D40 release invalid for {arm_key}: must be typed by Casey with UOR_LAB unset"
            )

    # Arms single-variable check (only coordinate_block changes from base_config)
    arms_cfg = bundle.get("arms_config", {})
    if "T1" not in arms_cfg or "T16" not in arms_cfg:
        raise VerificationError("Stage T requires at least T1 and T16 in arms_config")
    expected_k = {"T1": 1, "T16": 16, "T4": 4}
    base_without_k = {k: v for k, v in base_config.items() if k != "coordinate_block"}
    for arm_name, cfg in arms_cfg.items():
        if arm_name not in expected_k:
            raise VerificationError(f"unexpected Stage T arm: {arm_name!r}")
        if int(cfg.get("coordinate_block", 1)) != expected_k[arm_name]:
            raise VerificationError(
                f"arm {arm_name} coordinate_block={cfg.get('coordinate_block')} != {expected_k[arm_name]}"
            )
        cfg_without_k = {k: v for k, v in cfg.items() if k != "coordinate_block"}
        if cfg_without_k != base_without_k:
            raise VerificationError(
                f"Single-variable rule violated on {arm_name}: differs from base_config outside coordinate_block"
            )
        if cfg.get("counterfactual_path") != "rows_v1":
            raise VerificationError(
                f"arm {arm_name} must use counterfactual_path == 'rows_v1'"
            )

    # First-batch controls (V1d, KId, V8d, V9d, AKd) — must PASS before any Stage T result is read
    controls = bundle.get("first_batch_controls")
    if not isinstance(controls, dict):
        raise VerificationError("first_batch_controls block required")

    failed_controls: List[str] = []

    v1d = controls.get("V1d", {})
    if (
        int(v1d.get("batches", 0)) != 24
        or not v1d.get("state_hash_equal", False)
        or not v1d.get("epoch_equal", False)
        or not v1d.get("accepted_updates_equal", False)
        or not v1d.get("cursor_equal", False)
    ):
        failed_controls.append("V1d")

    kid = controls.get("KId", {})
    if (
        not kid.get("all_epochs_match_accepted_updates", False)
        or not kid.get("committed_moves_equals_sum_moves", False)
        or not kid.get("commit_ordinals_len_equals_moves", False)
        or not kid.get("committed_loss_equals_fresh_device_score", False)
        or not kid.get("committed_loss_strictly_below_l_before_on_commits", False)
    ):
        failed_controls.append("KId")

    if bundle.get("frozen_zero_arm_registered", False):
        v8d = controls.get("V8d", {})
        if (
            not v8d.get("all_relation_trits_zero", False)
            or int(v8d.get("committed_relation_ordinals", -1)) != 0
        ):
            failed_controls.append("V8d")

    v9d = controls.get("V9d", {})
    if (
        int(v9d.get("continued_batches", 0)) != 64
        or not v9d.get("resumed_equals_uninterrupted_bitwise", False)
        or v9d.get("resumed_state_sha256") != v9d.get("uninterrupted_state_sha256")
        or v9d.get("resumed_cursor") != v9d.get("uninterrupted_cursor")
    ):
        failed_controls.append("V9d")

    akd = controls.get("AKd", {})
    if (
        int(akd.get("single_move_batches_checked", 0)) != 32
        or not akd.get("shadow_k1_transition_state_equals_t16_byte_for_byte", False)
        or not akd.get("shadow_admission_authorized", False)
    ):
        failed_controls.append("AKd")

    if failed_controls:
        return {
            "status": "BLOCKED_D31",
            "verdict": "STAGE_T_BLOCKED_FIRST_BATCH_CONTROL_FAIL_D31",
            "failed_controls": failed_controls,
            "results_readable": False,
            "mechanism_negative": False,
        }

    # Equal-wall-clock W protocol & data split check
    w_sec = float(bundle.get("wall_clock_w_sec", 0.0))
    if w_sec <= 0.0:
        raise VerificationError("wall_clock_w_sec must be positive")

    data_split = bundle.get("data_split", {})
    if (
        data_split.get("train_tokens_sha256") != DOSE1_TRAIN_TOKENS_SHA256
        or data_split.get("dev_tokens_sha256") != DOSE1_DEV_TOKENS_SHA256
        or data_split.get("dev_mask_sha256") != DOSE1_DEV_MASK_SHA256
        or int(data_split.get("dev_rows", 0)) != DOSE1_DEV_ROWS
        or int(data_split.get("psucc_rows", 0)) != DOSE1_PSUCC_ROWS
    ):
        raise VerificationError("data_split pins do not match Dose 1 TRAIN/DEV contract")

    runs = bundle.get("arm_runs", {})
    for arm_name in arms_cfg:
        if arm_name not in runs:
            raise VerificationError(f"missing arm_runs entry for {arm_name}")
        for seed_key in ("seed1", "seed2"):
            run = runs[arm_name].get(seed_key)
            if not isinstance(run, dict):
                raise VerificationError(f"missing {arm_name}.{seed_key} run record")
            if not run.get("clock_starts_at_first_training_batch_after_admission_and_cp0_eval", False):
                raise VerificationError(
                    f"{arm_name}.{seed_key}: clock must start at first training batch after admission & cp0 eval"
                )
            if not run.get("build_table_load_preflight_excluded", False):
                raise VerificationError(
                    f"{arm_name}.{seed_key}: build, table load, and pre-flight must be excluded from clock"
                )
            if not run.get("dev_evals_and_checkpoint_io_included", False):
                raise VerificationError(
                    f"{arm_name}.{seed_key}: DEV evaluations and checkpoint I/O must be included in clock"
                )
            read_cp_sec = float(run.get("read_checkpoint_elapsed_sec", w_sec + 1.0))
            next_cp_sec = run.get("next_checkpoint_elapsed_sec")
            if read_cp_sec > w_sec or read_cp_sec <= 0.0:
                raise VerificationError(
                    f"{arm_name}.{seed_key}: arm must be read at last completed checkpoint <= W ({w_sec}s), got {read_cp_sec}s"
                )
            if next_cp_sec is not None and float(next_cp_sec) <= w_sec:
                raise VerificationError(
                    f"{arm_name}.{seed_key}: next checkpoint ({next_cp_sec}s) is <= W ({w_sec}s), so read checkpoint was not the last <= W"
                )
            q1_sec = float(run.get("q1_wall_clock_sec", -1.0))
            if abs(q1_sec - (w_sec / 4.0)) > 1e-6:
                raise VerificationError(
                    f"{arm_name}.{seed_key}: Q1 stop must map to W/4 ({w_sec / 4.0}s), got {q1_sec}s"
                )

    # Evaluate T16 (and optional T4) against T1 across seeds 1 and 2
    arm_verdicts: Dict[str, Any] = {}
    for cand_arm in [a for a in ("T16", "T4") if a in arms_cfg]:
        seed_comparisons: Dict[str, Any] = {}
        both_keep_top1 = True
        both_non_inferior_top1 = True
        both_non_inferior_loss = True
        both_faster_tok_s = True

        for seed_key in ("seed1", "seed2"):
            t1_run = runs["T1"][seed_key]
            tk_run = runs[cand_arm][seed_key]
            d_top1 = int(tk_run["dev_psucc_top1"]) - int(t1_run["dev_psucc_top1"])
            d_loss = float(tk_run["full_dev_loss"]) - float(t1_run["full_dev_loss"])
            t1_tok_s = float(t1_run["target_tokens_per_s"])
            tk_tok_s = float(tk_run["target_tokens_per_s"])

            seed_keep_top1 = d_top1 > margin_top1
            seed_ni_top1 = d_top1 >= -margin_top1
            seed_ni_loss = d_loss <= margin_loss
            seed_faster = tk_tok_s > t1_tok_s

            both_keep_top1 = both_keep_top1 and seed_keep_top1
            both_non_inferior_top1 = both_non_inferior_top1 and seed_ni_top1
            both_non_inferior_loss = both_non_inferior_loss and seed_ni_loss
            both_faster_tok_s = both_faster_tok_s and seed_faster

            seed_comparisons[seed_key] = {
                "d_psucc_top1_rows": d_top1,
                "d_full_dev_loss": d_loss,
                "t1_target_tokens_per_s": t1_tok_s,
                "tk_target_tokens_per_s": tk_tok_s,
                "keep_top1_beyond_margin": seed_keep_top1,
                "non_inferior_top1": seed_ni_top1,
                "non_inferior_full_dev_loss": seed_ni_loss,
            }

        if both_keep_top1 and both_non_inferior_loss:
            fresh = bundle.get("fresh_transfer", {})
            if (
                fresh.get("fresh_transfer_reported", False)
                and fresh.get("pm_typed_fresh_authorization", False)
                and _require_hex64(fresh.get("fresh_tokens_sha256", ""), "fresh_tokens_sha256")
                and fresh.get("fresh_tokens_sha256") != DOSE1_TRAIN_TOKENS_SHA256
                and fresh.get("fresh_tokens_sha256") != DOSE1_DEV_TOKENS_SHA256
                and not fresh.get("reused_consumed_slice", False)
            ):
                verdict = "KEEP_STAGE_T_ARM"
                config_closed = False
            else:
                verdict = "DEV_KEEP_PENDING_FRESH_TRANSFER_D22"
                config_closed = False
        elif both_non_inferior_top1 and both_non_inferior_loss and both_faster_tok_s:
            verdict = "NON_INFERIOR_THROUGHPUT_GAIN"
            config_closed = False
        else:
            verdict = "STAGE_T_ARM_NOT_KEPT"
            config_closed = True

        arm_verdicts[cand_arm] = {
            "verdict": verdict,
            "configuration_closed": config_closed,
            "mechanism_negative": False,
            "seeds": seed_comparisons,
        }

    return {
        "status": "PASS",
        "verdict": arm_verdicts["T16"]["verdict"],
        "results_readable": True,
        "mechanism_negative": False,
        "arm_verdicts": arm_verdicts,
    }


# ---------------------------------------------------------------------------
# 4. Decode Tokens/s Baseline Instrument Verifier
# ---------------------------------------------------------------------------


def verify_decode_bench_bundle(bundle: Dict[str, Any]) -> Dict[str, Any]:
    """Verify a decode-bench baseline (and optional candidate kernel comparison) bundle."""
    inst = bundle.get("instrument", {})
    if inst.get("binary_name") != "decode-bench":
        raise VerificationError("decode-bench requires separate binary_name == 'decode-bench'")
    if not inst.get("driver_identity_unchanged", False):
        raise VerificationError("decode-bench requires driver_identity_unchanged == True")
    if not inst.get("release_profile", False):
        raise VerificationError("decode-bench requires release_profile == True")
    if inst.get("host_model_arithmetic", True):
        raise VerificationError("D24 violated: host_model_arithmetic must be False")
    if not inst.get("d11_integer_discrete", False) or not inst.get("gpu_only", False):
        raise VerificationError("D11/D24 violated: requires d11_integer_discrete and gpu_only")

    _require_hex64(inst.get("binary_sha256"), "instrument.binary_sha256")
    _require_hex64(inst.get("source_sha256"), "instrument.source_sha256")

    caps = bundle.get("resource_caps", {})
    if (
        int(caps.get("pods_concurrent", 99)) > 4
        or float(caps.get("rate_usd_per_hr", 99.0)) > 8.0
        or float(caps.get("disk_free_gib", 0.0)) < 30.0
    ):
        raise VerificationError("decode-bench resource caps or 30 GiB disk floor violated")

    split = bundle.get("data_split", {})
    if (
        split.get("prompt_source_split") != "dev"
        or split.get("generation_scored_for_quality", True)
        or split.get("uses_train_or_fresh_data", True)
    ):
        raise VerificationError(
            "decode-bench data split violated: prompts must come from DEV only, unscored, no TRAIN/fresh"
        )

    controls = bundle.get("finite_controls", {})
    c1 = bool(controls.get("generated_ids_equal_across_repeats", False))
    c2 = bool(controls.get("batched_stream0_equals_single_stream", False))
    c3 = bool(controls.get("sliding_window_c243_refuses_nothing", False))
    if not (c1 and c2 and c3):
        return {
            "status": "BLOCKED_D31",
            "verdict": "DECODE_BENCH_CONTROL_FAIL_D31",
            "stop_for_diagnosis": True,
            "controls": {
                "generated_ids_equal_across_repeats": c1,
                "batched_stream0_equals_single_stream": c2,
                "sliding_window_c243_refuses_nothing": c3,
            },
        }

    checkpoints = bundle.get("checkpoints", {})
    for cp_name in DECODE_CHECKPOINTS:
        _require_hex64(checkpoints.get(cp_name), f"checkpoints.{cp_name}")

    cells = bundle.get("cells")
    if not isinstance(cells, list) or len(cells) != 24:
        raise VerificationError(
            f"decode-bench requires exactly 24 cells (6 c x 2 batch shapes x 2 checkpoints), got {len(cells) if isinstance(cells, list) else type(cells).__name__}"
        )

    cell_map: Dict[Tuple[str, int, int], Dict[str, Any]] = {}
    for idx, cell in enumerate(cells):
        cp_name = cell.get("checkpoint")
        c_len = int(cell.get("context_length", -1))
        shape_name = cell.get("batch_shape")
        rows = int(cell.get("rows", -1))
        if cp_name not in DECODE_CHECKPOINTS:
            raise VerificationError(f"cell[{idx}]: invalid checkpoint {cp_name!r}")
        if c_len not in DECODE_CONTEXT_LENGTHS:
            raise VerificationError(f"cell[{idx}]: invalid context_length {c_len}")
        if (shape_name, rows) not in DECODE_BATCH_SHAPES:
            raise VerificationError(
                f"cell[{idx}]: invalid (batch_shape, rows)=({shape_name!r}, {rows})"
            )
        key = (cp_name, c_len, rows)
        if key in cell_map:
            raise VerificationError(f"duplicate decode-bench cell {key}")

        if int(cell.get("generated_tokens_n", 0)) != DECODE_GENERATED_TOKENS_N:
            raise VerificationError(f"cell[{idx}]: generated_tokens_n must be 256")
        if int(cell.get("warmup_repeats", 0)) < 1:
            raise VerificationError(f"cell[{idx}]: warmup_repeats must be >= 1")
        if int(cell.get("timed_repeats", 0)) != DECODE_TIMED_REPEATS:
            raise VerificationError(f"cell[{idx}]: timed_repeats must be 5")
        if cell.get("timing_unit") != "ns":
            raise VerificationError(f"cell[{idx}]: timing_unit must be 'ns'")

        repeats = cell.get("repeat_total_ns")
        if (
            not isinstance(repeats, list)
            or len(repeats) != DECODE_TIMED_REPEATS
            or any(not isinstance(x, int) or x <= 0 for x in repeats)
        ):
            raise VerificationError(f"cell[{idx}]: repeat_total_ns must be 5 positive integers")

        expected_p50 = sorted(repeats)[2]
        expected_p95 = sorted(repeats)[4]
        p50_ns = int(cell.get("p50_ns", -1))
        p95_ns = int(cell.get("p95_ns", -1))
        if p50_ns != expected_p50 or p95_ns < p50_ns or p95_ns != expected_p95:
            raise VerificationError(
                f"cell[{idx}]: p50_ns/p95_ns ({p50_ns}/{p95_ns}) mismatch repeat_total_ns ({expected_p50}/{expected_p95})"
            )

        carry_ns = int(cell.get("carry_ns", -1))
        forward_ns = int(cell.get("forward_ns", -1))
        host_id_move_ns = int(cell.get("host_id_move_ns", -1))
        if carry_ns <= 0 or forward_ns <= 0 or host_id_move_ns <= 0:
            raise VerificationError(
                f"cell[{idx}]: carry_ns, forward_ns, host_id_move_ns must be positive integers"
            )
        if carry_ns + forward_ns + host_id_move_ns != p50_ns:
            raise VerificationError(
                f"cell[{idx}]: carry_ns + forward_ns + host_id_move_ns ({carry_ns + forward_ns + host_id_move_ns}) != p50_ns ({p50_ns})"
            )

        per_tok_p50 = int(cell.get("per_token_p50_ns", -1))
        per_tok_p95 = int(cell.get("per_token_p95_ns", -1))
        if per_tok_p50 <= 0 or per_tok_p95 < per_tok_p50:
            raise VerificationError(f"cell[{idx}]: invalid per_token_p50_ns / per_token_p95_ns")

        expected_stream_tps = (DECODE_GENERATED_TOKENS_N * 1_000_000_000.0) / float(p50_ns)
        expected_agg_tps = (rows * DECODE_GENERATED_TOKENS_N * 1_000_000_000.0) / float(p50_ns)
        if abs(float(cell.get("per_stream_tokens_per_s", 0.0)) - expected_stream_tps) > 1e-3:
            raise VerificationError(f"cell[{idx}]: per_stream_tokens_per_s mismatch")
        if abs(float(cell.get("aggregate_tokens_per_s", 0.0)) - expected_agg_tps) > 1e-3:
            raise VerificationError(f"cell[{idx}]: aggregate_tokens_per_s mismatch")

        _require_hex64(cell.get("generated_tokens_sha256"), f"cell[{idx}].generated_tokens_sha256")
        cell_map[key] = cell

    # Cross-check stream 0 token parity between batched_64 and single_stream
    for cp_name in DECODE_CHECKPOINTS:
        for c_len in DECODE_CONTEXT_LENGTHS:
            single_cell = cell_map.get((cp_name, c_len, 1))
            batch_cell = cell_map.get((cp_name, c_len, 64))
            if single_cell is None or batch_cell is None:
                raise VerificationError(f"missing cell pair for ({cp_name}, c={c_len})")
            if batch_cell.get("stream0_tokens_sha256") != single_cell["generated_tokens_sha256"]:
                return {
                    "status": "BLOCKED_D31",
                    "verdict": "DECODE_BENCH_CONTROL_FAIL_D31",
                    "stop_for_diagnosis": True,
                    "reason": f"stream0_tokens_sha256 mismatch at ({cp_name}, c={c_len})",
                }

    # Optional candidate kernel comparison against baseline
    candidate_cells = bundle.get("candidate_cells")
    if candidate_cells is not None:
        if not isinstance(candidate_cells, list) or len(candidate_cells) != 24:
            raise VerificationError("candidate_cells must contain all 24 cells")
        for cand in candidate_cells:
            key = (cand.get("checkpoint"), int(cand.get("context_length", -1)), int(cand.get("rows", -1)))
            base = cell_map.get(key)
            if base is None:
                raise VerificationError(f"candidate cell {key} has no matching baseline cell")
            if cand.get("generated_tokens_sha256") != base["generated_tokens_sha256"]:
                return {
                    "status": "REJECTED",
                    "verdict": "CANDIDATE_REJECTED_TOKEN_PARITY_MISMATCH",
                    "cell": key,
                }
            if float(cand.get("aggregate_tokens_per_s", 0.0)) <= float(base["aggregate_tokens_per_s"]):
                return {
                    "status": "REJECTED",
                    "verdict": "CANDIDATE_REJECTED_NO_THROUGHPUT_GAIN",
                    "cell": key,
                }
        return {
            "status": "PASS",
            "verdict": "CANDIDATE_KERNEL_PASS",
            "cells_verified": 24,
        }

    return {
        "status": "PASS",
        "verdict": "DECODE_BENCH_BASELINE_RECORDED",
        "cells_verified": 24,
    }


# ---------------------------------------------------------------------------
# Synthetic Fixture Builders for 32-Test Self-Test Suite
# ---------------------------------------------------------------------------


def _make_valid_staget_bundle() -> Dict[str, Any]:
    base_cfg = {
        "schema": "uor-r4.discrete-corpus-driver/1",
        "objective_policy": "nearest_address_margin_v1",
        "counterfactual_path": "rows_v1",
        "coordinate_block": 1,
        "schedule": "interleaved_read_every8_v1",
        "batch_rows": 64,
        "vocabulary": 4096,
        "blocks": 4,
        "context": 192,
        "kept_cells": 8,
    }
    t1_cfg = copy.deepcopy(base_cfg)
    t16_cfg = copy.deepcopy(base_cfg)
    t16_cfg["coordinate_block"] = 16
    t4_cfg = copy.deepcopy(base_cfg)
    t4_cfg["coordinate_block"] = 4

    def _run_entry(top1: int, loss: float, tok_s: float) -> Dict[str, Any]:
        return {
            "clock_starts_at_first_training_batch_after_admission_and_cp0_eval": True,
            "build_table_load_preflight_excluded": True,
            "dev_evals_and_checkpoint_io_included": True,
            "read_checkpoint_elapsed_sec": 21540.0,
            "next_checkpoint_elapsed_sec": 21660.0,
            "q1_wall_clock_sec": 5400.0,
            "dev_psucc_top1": top1,
            "full_dev_loss": loss,
            "dev_top8": top1 + 380,
            "dev_b3": top1 + 900,
            "target_tokens_per_s": tok_s,
        }

    return {
        "precondition_1_c2_qualification": {
            "c2_qualification_pass": True,
            "c2_comment_id": "6257768576",
            "c2_controls_passed": 5,
            "record": "docs/labs/injective-preflight-and-c2-qualify-2026-10-19/",
        },
        "precondition_2_base_lineage": {
            "lineage_id": "successor-a8-admission-preserving-v1",
            "preflight_passed": True,
            "b3_init_psucc": 1351,
            "q1_passed": True,
            "q1_admission_collapse": False,
            "base_config": base_cfg,
        },
        "precondition_3_margin": {
            "fixed_before_results": True,
            "source_seeds_count": 2,
            "margin_source": "base_lineage_two_seed_t1",
            "margin_top1_rows": 24,
            "margin_top8_rows": 65,
            "margin_b3_rows": 90,
            "margin_full_dev_loss": 1200.0,
        },
        "registration_and_release": {
            "sheet_verdict": "FIT",
            "reviewer_subagent_finding_comment_ids": ["6257754797"],
            "pm_dispatch_id": "6263000001",
            "pods_concurrent": 2,
            "rate_usd_per_hr": 3.8,
            "disk_free_gib": 64.0,
            "d40_releases": {
                "T1_seed1": {"owner": "Casey", "uor_lab_env_unset": True},
                "T1_seed2": {"owner": "Casey", "uor_lab_env_unset": True},
                "T16_seed1": {"owner": "Casey", "uor_lab_env_unset": True},
                "T16_seed2": {"owner": "Casey", "uor_lab_env_unset": True},
                "T4_seed1": {"owner": "Casey", "uor_lab_env_unset": True},
                "T4_seed2": {"owner": "Casey", "uor_lab_env_unset": True},
            },
        },
        "arms_config": {
            "T1": t1_cfg,
            "T16": t16_cfg,
            "T4": t4_cfg,
        },
        "frozen_zero_arm_registered": True,
        "first_batch_controls": {
            "V1d": {
                "batches": 24,
                "state_hash_equal": True,
                "epoch_equal": True,
                "accepted_updates_equal": True,
                "cursor_equal": True,
            },
            "KId": {
                "all_epochs_match_accepted_updates": True,
                "committed_moves_equals_sum_moves": True,
                "commit_ordinals_len_equals_moves": True,
                "committed_loss_equals_fresh_device_score": True,
                "committed_loss_strictly_below_l_before_on_commits": True,
            },
            "V8d": {
                "all_relation_trits_zero": True,
                "committed_relation_ordinals": 0,
            },
            "V9d": {
                "continued_batches": 64,
                "resumed_equals_uninterrupted_bitwise": True,
                "resumed_state_sha256": "a" * 64,
                "uninterrupted_state_sha256": "a" * 64,
                "resumed_cursor": {"completed_batches": 128, "accepted_updates": 40},
                "uninterrupted_cursor": {"completed_batches": 128, "accepted_updates": 40},
            },
            "AKd": {
                "single_move_batches_checked": 32,
                "shadow_k1_transition_state_equals_t16_byte_for_byte": True,
                "shadow_admission_authorized": True,
            },
        },
        "wall_clock_w_sec": 21600.0,
        "data_split": {
            "train_tokens_sha256": DOSE1_TRAIN_TOKENS_SHA256,
            "dev_tokens_sha256": DOSE1_DEV_TOKENS_SHA256,
            "dev_mask_sha256": DOSE1_DEV_MASK_SHA256,
            "dev_rows": DOSE1_DEV_ROWS,
            "psucc_rows": DOSE1_PSUCC_ROWS,
        },
        "arm_runs": {
            "T1": {
                "seed1": _run_entry(top1=180, loss=750000.0, tok_s=510.0),
                "seed2": _run_entry(top1=176, loss=751000.0, tok_s=508.0),
            },
            "T16": {
                "seed1": _run_entry(top1=215, loss=748500.0, tok_s=3950.0),
                "seed2": _run_entry(top1=210, loss=749200.0, tok_s=3920.0),
            },
            "T4": {
                "seed1": _run_entry(top1=208, loss=749000.0, tok_s=1850.0),
                "seed2": _run_entry(top1=204, loss=749500.0, tok_s=1840.0),
            },
        },
        "fresh_transfer": {
            "fresh_transfer_reported": True,
            "pm_typed_fresh_authorization": True,
            "fresh_tokens_sha256": "0b7e754cfd242bf53145fe9cbca563be8503e6cd75df5449abb325b5c51ee1b8",
            "reused_consumed_slice": False,
        },
    }


def _make_valid_decode_bench_bundle() -> Dict[str, Any]:
    cells: List[Dict[str, Any]] = []
    for cp_idx, cp_name in enumerate(DECODE_CHECKPOINTS):
        for c_idx, c_len in enumerate(DECODE_CONTEXT_LENGTHS):
            tok_hash = hashlib.sha256(f"{cp_name}:{c_len}".encode("utf-8")).hexdigest()
            for shape_name, rows in DECODE_BATCH_SHAPES:
                base_ns = (12_000_000 + c_idx * 1_000_000 + cp_idx * 100_000) * (
                    1 if rows == 1 else 8
                )
                repeats = [base_ns - 2000, base_ns - 1000, base_ns, base_ns + 1000, base_ns + 3000]
                carry_ns = base_ns // 4
                forward_ns = (base_ns * 3) // 4 - 5000
                host_ns = base_ns - carry_ns - forward_ns
                cell: Dict[str, Any] = {
                    "checkpoint": cp_name,
                    "context_length": c_len,
                    "batch_shape": shape_name,
                    "rows": rows,
                    "generated_tokens_n": DECODE_GENERATED_TOKENS_N,
                    "warmup_repeats": 1,
                    "timed_repeats": DECODE_TIMED_REPEATS,
                    "timing_unit": "ns",
                    "repeat_total_ns": repeats,
                    "p50_ns": base_ns,
                    "p95_ns": base_ns + 3000,
                    "carry_ns": carry_ns,
                    "forward_ns": forward_ns,
                    "host_id_move_ns": host_ns,
                    "per_token_p50_ns": base_ns // DECODE_GENERATED_TOKENS_N,
                    "per_token_p95_ns": (base_ns + 3000) // DECODE_GENERATED_TOKENS_N,
                    "per_stream_tokens_per_s": (DECODE_GENERATED_TOKENS_N * 1_000_000_000.0)
                    / float(base_ns),
                    "aggregate_tokens_per_s": (
                        rows * DECODE_GENERATED_TOKENS_N * 1_000_000_000.0
                    )
                    / float(base_ns),
                    "generated_tokens_sha256": tok_hash,
                }
                if rows == 64:
                    cell["stream0_tokens_sha256"] = tok_hash
                cells.append(cell)

    return {
        "instrument": {
            "binary_name": "decode-bench",
            "driver_identity_unchanged": True,
            "release_profile": True,
            "host_model_arithmetic": False,
            "d11_integer_discrete": True,
            "gpu_only": True,
            "binary_sha256": "1" * 64,
            "source_sha256": "2" * 64,
        },
        "resource_caps": {
            "pods_concurrent": 1,
            "rate_usd_per_hr": 1.9,
            "disk_free_gib": 50.0,
        },
        "data_split": {
            "prompt_source_split": "dev",
            "generation_scored_for_quality": False,
            "uses_train_or_fresh_data": False,
        },
        "finite_controls": {
            "generated_ids_equal_across_repeats": True,
            "batched_stream0_equals_single_stream": True,
            "sliding_window_c243_refuses_nothing": True,
        },
        "checkpoints": {
            "treatment": "3" * 64,
            "frozen_zero": "4" * 64,
        },
        "cells": cells,
    }


# ---------------------------------------------------------------------------
# 5. 32-Test Self-Test Suite
# ---------------------------------------------------------------------------


def run_self_tests(repo_root: Path) -> Dict[str, Any]:
    """Run all 32 positive and negative self-tests."""
    results: List[Dict[str, Any]] = []

    def _record(idx: int, name: str) -> None:
        print(f"[{idx}/32] {name}: PASS")
        results.append({"test": idx, "name": name, "status": "PASS"})

    def _expect_raises(fn: Any, substring: Optional[str] = None) -> None:
        try:
            fn()
        except VerificationError as exc:
            if substring is not None and substring not in str(exc):
                raise AssertionError(
                    f"Expected VerificationError containing {substring!r}, got {str(exc)!r}"
                ) from exc
            return
        raise AssertionError("Expected VerificationError, but call succeeded")

    # [1/32] Live repo & sheet contracts pass
    repo_report = verify_repo_and_sheet_contracts(repo_root)
    assert repo_report["status"] == "PASS"
    _record(1, "repo_and_sheet_contracts_pinned_digests_pass")

    staget_text = (repo_root / STAGET_DRAFT_RELPATH).read_text(encoding="utf-8")

    # [2/32] Sheet fitness status mismatch rejected
    _expect_raises(
        lambda: parse_sheet_fitness_status(
            staget_text.replace("TEST FITNESS: PENDING.", "TEST FITNESS: FIT."),
            "mutated_top_fit",
        ),
        "inconsistent",
    )
    _expect_raises(
        lambda: parse_sheet_fitness_status(
            staget_text.rstrip()[:-len("TEST FITNESS: PENDING")] + "TEST FITNESS: FIT\n",
            "mutated_bottom_fit",
        ),
        "top header does not match",
    )
    _record(2, "sheet_fitness_status_mismatch_rejected")

    # [3/32] Stage T draft repair 1 (<= 4-pod wave plan) violation rejected
    _expect_raises(
        lambda: verify_staget_draft_repairs(staget_text + "\nUsing 3 pod-pairs (6 pods)\n"),
        "repair 1",
    )
    _expect_raises(
        lambda: verify_staget_draft_repairs(
            staget_text.replace("Using 2 pods × 2 GPUs (4 slots, about $3.8/h)", "Using 1 pod")
        ),
        "repair 1",
    )
    _record(3, "staget_draft_repair_1_pod_cap_violation_rejected")

    # [4/32] Stage T draft repair 2 (unmeasured base batch time vs K16 Route C1/C2) violation rejected
    _expect_raises(
        lambda: verify_staget_draft_repairs(
            staget_text.replace(
                "The base lineage's per-batch time is **not measured**.",
                "The base lineage's per-batch time is measured.",
            )
        ),
        "repair 2",
    )
    _record(4, "staget_draft_repair_2_unmeasured_batch_time_violation_rejected")

    # [5/32] Stage T draft repair 3 (two-seed dispersion precondition) violation rejected
    _expect_raises(
        lambda: verify_staget_draft_repairs(
            staget_text.replace("measured two-seed dispersion", "single-seed estimate")
        ),
        "repair 3",
    )
    _record(5, "staget_draft_repair_3_single_seed_margin_violation_rejected")

    # [6/32] Stage T draft repair 4 (equal-wall-clock W protocol) violation rejected
    _expect_raises(
        lambda: verify_staget_draft_repairs(
            staget_text.replace(
                "DEV evaluations and checkpoint I/O are **included** in every arm's own clock",
                "DEV evaluations are excluded",
            )
        ),
        "repair 4",
    )
    _record(6, "staget_draft_repair_4_wall_clock_w_violation_rejected")

    # [7/32] Stage T draft repair 5 (quality metrics & KEEP bars) violation rejected
    _expect_raises(
        lambda: verify_staget_draft_repairs(
            staget_text.replace(
                "**Quality metrics.** **Primary:** DEV `P_succ` conditional top-1. **Co-primary:** full DEV loss. **Reported, not gating:** top-8 and B3.",
                "**Quality metrics.** Top-8 only.",
            )
        ),
        "repair 5",
    )
    _record(7, "staget_draft_repair_5_quality_metrics_violation_rejected")

    # [8/32] family_cursors and block_at at K=1 reproduce coordinate_at under both schedules
    for sched in ("interleaved_read_every_8_v1", "canonical_vector_then_read_v1"):
        for step in range(200):
            c1 = coordinate_at_k1(step, sched, vectors=20, reads=4, read_spec="gate")
            b1 = block_at(step, 1, sched, vectors=20, reads=4, read_spec="gate")
            assert b1["kind"] == c1["kind"] and b1["indices"] == [c1["index"]]
    _record(8, "family_cursors_k1_reproduces_coordinate_at_both_schedules")

    # [9/32] block_at at K=3 truncates at pass boundaries under both schedules (training.rs:1890)
    expected_vec_pass = [
        [0, 1, 2],
        [3, 4, 5],
        [6, 7, 8],
        [9, 10, 11],
        [12, 13, 14],
        [15, 16, 17],
        [18, 19],
    ]
    expected_read_pass = [[0, 1, 2], [3]]
    canon_vec, canon_read = [], []
    for step in range(9):
        blk = block_at(step, 3, "canonical_vector_then_read_v1", vectors=20, reads=4)
        (canon_vec if blk["kind"] == "vector_block" else canon_read).append(blk["indices"])
    assert canon_vec == expected_vec_pass and canon_read == expected_read_pass

    inter_vec, inter_read = [], []
    for step in range(16):
        blk = block_at(step, 3, "interleaved_read_every_8_v1", vectors=20, reads=4)
        (inter_vec if blk["kind"] == "vector_block" else inter_read).append(blk["indices"])
    assert inter_vec == expected_vec_pass + expected_vec_pass and inter_read == expected_read_pass
    _record(9, "family_cursors_k3_truncates_at_pass_boundaries_both_schedules")

    # [10/32] Dose 1 shape (vectors=16384, reads=45, K=16) schedule & truncation check
    # 3rd read batch (completed_batches = 23 under interleaved, where read_batches = 2) visits [32..44] (13 gates)
    blk_r0 = block_at(7, 16, "interleaved_read_every_8_v1", vectors=16384, reads=45)
    blk_r1 = block_at(15, 16, "interleaved_read_every_8_v1", vectors=16384, reads=45)
    blk_r2 = block_at(23, 16, "interleaved_read_every_8_v1", vectors=16384, reads=45)
    blk_r3 = block_at(31, 16, "interleaved_read_every_8_v1", vectors=16384, reads=45)
    assert blk_r0["indices"] == list(range(0, 16))
    assert blk_r1["indices"] == list(range(16, 32))
    assert blk_r2["indices"] == list(range(32, 45)) and len(blk_r2["indices"]) == 13
    assert blk_r3["indices"] == list(range(0, 16)) and blk_r3["o_read"] == 45
    _record(10, "family_cursors_dose1_k16_interleaved_and_canonical_invariants")

    # [11/32] Cursor validation: K=1 with block fields and K=4 without block fields rejected
    k1_bad = initial_cursor_for(4)
    _expect_raises(
        lambda: validate_cursor(k1_bad, 1, "interleaved_read_every_8_v1", 20, 4, 2, 3, 24),
        "coordinate_block 1 cursor carries no block fields",
    )
    k4_bad = initial_cursor_for(1)
    _expect_raises(
        lambda: validate_cursor(k4_bad, 4, "interleaved_read_every_8_v1", 20, 4, 2, 3, 24),
        "coordinate_block > 1 cursor requires block fields",
    )
    _record(11, "cursor_validation_k1_with_block_and_k4_without_block_rejected")

    # [12/32] Cursor validation: bad family cursors and committed_moves bounds rejected
    cur = initial_cursor_for(4)
    cur = advance_block_cursor(cur, True, 4, 4, "interleaved_read_every_8_v1", 20, 4, 2, 3, 24)
    cur = advance_block_cursor(cur, False, 0, 4, "interleaved_read_every_8_v1", 20, 4, 2, 3, 24)
    cur = advance_block_cursor(cur, True, 1, 4, "interleaved_read_every_8_v1", 20, 4, 2, 3, 24)
    validate_cursor(cur, 4, "interleaved_read_every_8_v1", 20, 4, 2, 3, 24)

    bad_ovec = copy.deepcopy(cur)
    bad_ovec["block"]["o_vec"] += 1
    _expect_raises(
        lambda: validate_cursor(bad_ovec, 4, "interleaved_read_every_8_v1", 20, 4, 2, 3, 24),
        "resume o_vec/o_read differ from the per-family block schedule",
    )
    bad_moves_low = copy.deepcopy(cur)
    bad_moves_low["block"]["committed_moves"] = 1  # accepted_updates == 2
    _expect_raises(
        lambda: validate_cursor(bad_moves_low, 4, "interleaved_read_every_8_v1", 20, 4, 2, 3, 24),
        "resume committed_moves outside accepted_updates..=K*accepted_updates",
    )
    bad_moves_high = copy.deepcopy(cur)
    bad_moves_high["block"]["committed_moves"] = 9  # K*accepted_updates == 8
    _expect_raises(
        lambda: validate_cursor(bad_moves_high, 4, "interleaved_read_every_8_v1", 20, 4, 2, 3, 24),
        "resume committed_moves outside accepted_updates..=K*accepted_updates",
    )
    _record(12, "cursor_validation_bad_family_cursors_and_committed_moves_bounds_rejected")

    # [13/32] BlockReceipt sequence (no-op, single commit, joint commit, fallback-to-best on tie) passes
    c0 = initial_cursor_for(4)
    c1 = advance_block_cursor(c0, False, 0, 4, "interleaved_read_every_8_v1", 20, 4, 2, 3, 24)
    c2 = advance_block_cursor(c1, True, 1, 4, "interleaved_read_every_8_v1", 20, 4, 2, 3, 24)
    c3 = advance_block_cursor(c2, True, 3, 4, "interleaved_read_every_8_v1", 20, 4, 2, 3, 24)
    c4 = advance_block_cursor(c3, True, 1, 4, "interleaved_read_every_8_v1", 20, 4, 2, 3, 24)
    receipts_seq = [
        {
            "lineage_sha256": "a" * 64,
            "cursor_before": c0,
            "cursor_after": c1,
            "kind": "vector_block",
            "k": 4,
            "s_len": 0,
            "joint": False,
            "l_before": 100,
            "l_joint": None,
            "l_best": 100,
            "committed_loss": 100,
            "commit_ordinals": [],
            "o_vec": 0,
            "o_read": 0,
            "completed_batches": 0,
            "model_epoch": 0,
        },
        {
            "lineage_sha256": "a" * 64,
            "cursor_before": c1,
            "cursor_after": c2,
            "kind": "vector_block",
            "k": 4,
            "s_len": 1,
            "joint": False,
            "l_before": 100,
            "l_joint": None,
            "l_best": 92,
            "committed_loss": 92,
            "commit_ordinals": [2],
            "o_vec": 4,
            "o_read": 0,
            "completed_batches": 1,
            "model_epoch": 1,
        },
        {
            "lineage_sha256": "a" * 64,
            "cursor_before": c2,
            "cursor_after": c3,
            "kind": "vector_block",
            "k": 4,
            "s_len": 3,
            "joint": True,
            "l_before": 92,
            "l_joint": 85,
            "l_best": 88,
            "committed_loss": 85,
            "commit_ordinals": [0, 2, 3],
            "o_vec": 8,
            "o_read": 0,
            "completed_batches": 2,
            "model_epoch": 2,
        },
        {
            "lineage_sha256": "a" * 64,
            "cursor_before": c3,
            "cursor_after": c4,
            "kind": "vector_block",
            "k": 4,
            "s_len": 2,
            "joint": False,
            "l_before": 85,
            "l_joint": 80,
            "l_best": 80,
            "committed_loss": 80,
            "commit_ordinals": [1],
            "o_vec": 12,
            "o_read": 0,
            "completed_batches": 3,
            "model_epoch": 3,
        },
    ]
    seq_res = verify_family_cursors_and_block_receipts(
        receipts_seq, 4, "interleaved_read_every_8_v1", 20, 4, 2, 3, 24
    )
    assert seq_res["total_committed_moves"] == 5 and seq_res["joint_commits"] == 1
    _record(13, "block_receipt_sequence_noop_single_and_joint_commits_pass")

    # [14/32] BlockReceipt joint commit on tie (L(S_t) == L({best})) rejected
    bad_tie = copy.deepcopy(receipts_seq)
    bad_tie[3]["joint"] = True
    bad_tie[3]["commit_ordinals"] = [1, 2]
    bad_tie[3]["cursor_after"] = advance_block_cursor(
        c3, True, 2, 4, "interleaved_read_every_8_v1", 20, 4, 2, 3, 24
    )
    _expect_raises(
        lambda: verify_family_cursors_and_block_receipts(
            bad_tie, 4, "interleaved_read_every_8_v1", 20, 4, 2, 3, 24
        ),
        "must fall back to single best",
    )
    _record(14, "block_receipt_joint_tie_or_worse_joint_commit_rejected")

    # [15/32] BlockReceipt l_joint presence and monotonicity violations rejected
    bad_ljoint = copy.deepcopy(receipts_seq)
    bad_ljoint[1]["l_joint"] = 92
    _expect_raises(
        lambda: verify_family_cursors_and_block_receipts(
            bad_ljoint, 4, "interleaved_read_every_8_v1", 20, 4, 2, 3, 24
        ),
        "l_joint must be absent when s_len == 1",
    )
    bad_mono = copy.deepcopy(receipts_seq)
    bad_mono[1]["l_best"] = 100
    bad_mono[1]["committed_loss"] = 100
    _expect_raises(
        lambda: verify_family_cursors_and_block_receipts(
            bad_mono, 4, "interleaved_read_every_8_v1", 20, 4, 2, 3, 24
        ),
        "requires committed_loss == l_best < l_before",
    )
    _record(15, "block_receipt_l_joint_presence_and_monotonicity_violations_rejected")

    # [16/32] Stage T Precondition 1: unqualified C2 rejected
    b_p1 = _make_valid_staget_bundle()
    b_p1["precondition_1_c2_qualification"]["c2_qualification_pass"] = False
    _expect_raises(lambda: verify_staget_bundle(b_p1), "Precondition 1 failed")
    _record(16, "staget_precondition_1_unqualified_c2_rejected")

    # [17/32] Stage T Precondition 2: unadmitted base lineage (pre-flight fail or Q1 stop) stays HELD
    b_p2_pf = _make_valid_staget_bundle()
    b_p2_pf["precondition_2_base_lineage"]["preflight_passed"] = False
    b_p2_pf["precondition_2_base_lineage"]["b3_init_psucc"] = 127
    res_p2_pf = verify_staget_bundle(b_p2_pf)
    assert res_p2_pf["verdict"] == "STAGE_T_HELD_PRECONDITION_2_BASE_LINEAGE_NOT_ADMITTED"
    assert res_p2_pf["mechanism_negative"] is False

    b_p2_q1 = _make_valid_staget_bundle()
    b_p2_q1["precondition_2_base_lineage"]["q1_passed"] = False
    b_p2_q1["precondition_2_base_lineage"]["q1_admission_collapse"] = True
    res_p2_q1 = verify_staget_bundle(b_p2_q1)
    assert res_p2_q1["verdict"] == "STAGE_T_HELD_PRECONDITION_2_BASE_LINEAGE_NOT_ADMITTED"
    _record(17, "staget_precondition_2_unadmitted_base_lineage_held")

    # [18/32] Stage T Precondition 2: multi-variable config drift rejected
    b_drift = _make_valid_staget_bundle()
    b_drift["arms_config"]["T16"]["batch_rows"] = 32
    _expect_raises(lambda: verify_staget_bundle(b_drift), "Single-variable rule violated")
    _record(18, "staget_precondition_2_multi_variable_config_drift_rejected")

    # [19/32] Stage T Precondition 3: single-seed or post-hoc margin rejected
    b_1seed = _make_valid_staget_bundle()
    b_1seed["precondition_3_margin"]["source_seeds_count"] = 1
    _expect_raises(lambda: verify_staget_bundle(b_1seed), "never set from a single seed")

    b_posthoc = _make_valid_staget_bundle()
    b_posthoc["precondition_3_margin"]["fixed_before_results"] = False
    _expect_raises(lambda: verify_staget_bundle(b_posthoc), "never set after any Stage T result")
    _record(19, "staget_precondition_3_single_seed_or_posthoc_margin_rejected")

    # [20/32] Stage T Precondition 3: D22 §3 2-SE floor violation rejected
    b_floor = _make_valid_staget_bundle()
    b_floor["precondition_3_margin"]["margin_top1_rows"] = 19
    _expect_raises(lambda: verify_staget_bundle(b_floor), "D22 §3 2-SE floors")
    _record(20, "staget_precondition_3_d22_2se_floor_violation_rejected")

    # [21/32] Stage T D40 release and resource cap violations rejected
    b_pods = _make_valid_staget_bundle()
    b_pods["registration_and_release"]["pods_concurrent"] = 6
    _expect_raises(lambda: verify_staget_bundle(b_pods), "pods_concurrent must be <= 4")

    b_d40 = _make_valid_staget_bundle()
    b_d40["registration_and_release"]["d40_releases"]["T16_seed1"]["uor_lab_env_unset"] = False
    _expect_raises(lambda: verify_staget_bundle(b_d40), "UOR_LAB unset")
    _record(21, "staget_d40_and_resource_cap_violations_rejected")

    # [22/32] Stage T first-batch control V1d failure stops line under D31
    b_v1d = _make_valid_staget_bundle()
    b_v1d["first_batch_controls"]["V1d"]["state_hash_equal"] = False
    res_v1d = verify_staget_bundle(b_v1d)
    assert res_v1d["verdict"] == "STAGE_T_BLOCKED_FIRST_BATCH_CONTROL_FAIL_D31"
    assert res_v1d["failed_controls"] == ["V1d"] and res_v1d["mechanism_negative"] is False
    _record(22, "staget_first_batch_control_v1d_k1_identity_fail_stops_line_d31")

    # [23/32] Stage T first-batch controls KId and V8d failure stop line under D31
    b_kid_v8d = _make_valid_staget_bundle()
    b_kid_v8d["first_batch_controls"]["KId"]["committed_loss_equals_fresh_device_score"] = False
    b_kid_v8d["first_batch_controls"]["V8d"]["committed_relation_ordinals"] = 1
    res_kid_v8d = verify_staget_bundle(b_kid_v8d)
    assert res_kid_v8d["verdict"] == "STAGE_T_BLOCKED_FIRST_BATCH_CONTROL_FAIL_D31"
    assert res_kid_v8d["failed_controls"] == ["KId", "V8d"]
    _record(23, "staget_first_batch_control_kid_and_v8d_fail_stops_line_d31")

    # [24/32] Stage T first-batch controls V9d and AKd failure stop line under D31
    b_v9d_akd = _make_valid_staget_bundle()
    b_v9d_akd["first_batch_controls"]["V9d"]["resumed_equals_uninterrupted_bitwise"] = False
    b_v9d_akd["first_batch_controls"]["AKd"][
        "shadow_k1_transition_state_equals_t16_byte_for_byte"
    ] = False
    res_v9d_akd = verify_staget_bundle(b_v9d_akd)
    assert res_v9d_akd["verdict"] == "STAGE_T_BLOCKED_FIRST_BATCH_CONTROL_FAIL_D31"
    assert res_v9d_akd["failed_controls"] == ["V9d", "AKd"]
    _record(24, "staget_first_batch_control_v9d_and_akd_fail_stops_line_d31")

    # [25/32] Stage T equal-wall-clock W protocol violations rejected
    b_clock = _make_valid_staget_bundle()
    b_clock["arm_runs"]["T16"]["seed1"]["read_checkpoint_elapsed_sec"] = 21700.0
    _expect_raises(lambda: verify_staget_bundle(b_clock), "last completed checkpoint <= W")

    b_q1_clock = _make_valid_staget_bundle()
    b_q1_clock["arm_runs"]["T16"]["seed2"]["q1_wall_clock_sec"] = 3600.0
    _expect_raises(lambda: verify_staget_bundle(b_q1_clock), "Q1 stop must map to W/4")
    _record(25, "staget_wall_clock_w_protocol_violation_rejected")

    # [26/32] Stage T 2-seed KEEP verdict and fresh transfer gate pass
    b_keep = _make_valid_staget_bundle()
    res_keep = verify_staget_bundle(b_keep)
    assert res_keep["verdict"] == "KEEP_STAGE_T_ARM"
    assert res_keep["arm_verdicts"]["T16"]["verdict"] == "KEEP_STAGE_T_ARM"
    assert res_keep["arm_verdicts"]["T4"]["verdict"] == "KEEP_STAGE_T_ARM"

    b_no_fresh = _make_valid_staget_bundle()
    b_no_fresh["fresh_transfer"]["fresh_transfer_reported"] = False
    res_no_fresh = verify_staget_bundle(b_no_fresh)
    assert res_no_fresh["verdict"] == "DEV_KEEP_PENDING_FRESH_TRANSFER_D22"
    _record(26, "staget_two_seed_keep_verdict_and_fresh_transfer_gate_pass")

    # [27/32] Stage T 2-seed NON_INFERIOR_THROUGHPUT_GAIN and STAGE_T_ARM_NOT_KEPT verdicts
    b_ni = _make_valid_staget_bundle()
    # Within non-inferiority margin (-24..+24) but not > +24 on both seeds
    b_ni["arm_runs"]["T16"]["seed1"]["dev_psucc_top1"] = 190  # +10 vs T1 (180)
    b_ni["arm_runs"]["T16"]["seed2"]["dev_psucc_top1"] = 185  # +9 vs T1 (176)
    res_ni = verify_staget_bundle(b_ni)
    assert res_ni["verdict"] == "NON_INFERIOR_THROUGHPUT_GAIN"

    b_fail = _make_valid_staget_bundle()
    b_fail["arm_runs"]["T16"]["seed2"]["dev_psucc_top1"] = 140  # -36 vs T1 (176), trails by > 24
    res_fail = verify_staget_bundle(b_fail)
    assert res_fail["verdict"] == "STAGE_T_ARM_NOT_KEPT"
    assert res_fail["arm_verdicts"]["T16"]["configuration_closed"] is True
    assert res_fail["arm_verdicts"]["T16"]["mechanism_negative"] is False
    _record(27, "staget_two_seed_non_inferior_throughput_gain_and_not_kept_verdicts")

    # [28/32] decode-bench 24-cell baseline bundle passes
    db_valid = _make_valid_decode_bench_bundle()
    res_db = verify_decode_bench_bundle(db_valid)
    assert res_db["status"] == "PASS" and res_db["verdict"] == "DECODE_BENCH_BASELINE_RECORDED"
    assert res_db["cells_verified"] == 24
    _record(28, "decode_bench_24_cell_baseline_bundle_pass")

    # [29/32] decode-bench finite control failures return DECODE_BENCH_CONTROL_FAIL_D31
    db_c_fail = _make_valid_decode_bench_bundle()
    db_c_fail["finite_controls"]["sliding_window_c243_refuses_nothing"] = False
    res_db_c = verify_decode_bench_bundle(db_c_fail)
    assert res_db_c["verdict"] == "DECODE_BENCH_CONTROL_FAIL_D31"

    db_stream0 = _make_valid_decode_bench_bundle()
    db_stream0["cells"][1]["stream0_tokens_sha256"] = "f" * 64
    res_stream0 = verify_decode_bench_bundle(db_stream0)
    assert res_stream0["verdict"] == "DECODE_BENCH_CONTROL_FAIL_D31"
    _record(29, "decode_bench_finite_control_failures_return_d31_stop")

    # [30/32] decode-bench missing cell or D24 host arithmetic rejected
    db_23 = _make_valid_decode_bench_bundle()
    db_23["cells"].pop()
    _expect_raises(lambda: verify_decode_bench_bundle(db_23), "exactly 24 cells")

    db_host = _make_valid_decode_bench_bundle()
    db_host["instrument"]["host_model_arithmetic"] = True
    _expect_raises(lambda: verify_decode_bench_bundle(db_host), "D24 violated")
    _record(30, "decode_bench_missing_cell_or_d24_host_arithmetic_rejected")

    # [31/32] decode-bench candidate kernel speedup with exact token parity passes
    db_cand_pass = _make_valid_decode_bench_bundle()
    cand_cells = copy.deepcopy(db_cand_pass["cells"])
    for cell in cand_cells:
        cell["aggregate_tokens_per_s"] *= 1.35
    db_cand_pass["candidate_cells"] = cand_cells
    res_cand_pass = verify_decode_bench_bundle(db_cand_pass)
    assert res_cand_pass["verdict"] == "CANDIDATE_KERNEL_PASS"
    _record(31, "decode_bench_candidate_kernel_speedup_with_exact_token_parity_pass")

    # [32/32] decode-bench candidate kernel token mismatch or slowdown rejected
    db_cand_tok = copy.deepcopy(db_cand_pass)
    db_cand_tok["candidate_cells"][0]["generated_tokens_sha256"] = "e" * 64
    res_cand_tok = verify_decode_bench_bundle(db_cand_tok)
    assert res_cand_tok["verdict"] == "CANDIDATE_REJECTED_TOKEN_PARITY_MISMATCH"

    db_cand_slow = copy.deepcopy(db_cand_pass)
    db_cand_slow["candidate_cells"][0]["aggregate_tokens_per_s"] = db_cand_slow["cells"][0][
        "aggregate_tokens_per_s"
    ]
    res_cand_slow = verify_decode_bench_bundle(db_cand_slow)
    assert res_cand_slow["verdict"] == "CANDIDATE_REJECTED_NO_THROUGHPUT_GAIN"
    _record(32, "decode_bench_candidate_kernel_token_mismatch_or_slowdown_rejected")

    return {
        "status": "PASS",
        "self_tests_passed": len(results),
        "self_tests_total": 32,
        "tests": results,
    }


def main(argv: Optional[Sequence[str]] = None) -> int:
    parser = argparse.ArgumentParser(
        description="Verify BVG-K Stage T, BlockCursor/BlockReceipt, and decode-bench contracts."
    )
    parser.add_argument(
        "--repo-root",
        type=Path,
        default=Path(__file__).resolve().parents[3],
        help="Repository root directory",
    )
    parser.add_argument(
        "--verify-repo-contract",
        action="store_true",
        help="Verify pinned draft sheets, training.rs symbols/refusals, and 6257754797 repairs",
    )
    parser.add_argument(
        "--staget-bundle",
        type=Path,
        default=None,
        help="Path to a Stage T registration/execution JSON bundle",
    )
    parser.add_argument(
        "--decode-bench-bundle",
        type=Path,
        default=None,
        help="Path to a decode-bench baseline/candidate JSON bundle",
    )
    parser.add_argument(
        "--self-test",
        action="store_true",
        help="Run the 32 positive and negative self-tests",
    )
    args = parser.parse_args(argv)

    output: Dict[str, Any] = {}
    if args.verify_repo_contract:
        output["repo_contract"] = verify_repo_and_sheet_contracts(args.repo_root)
    if args.staget_bundle is not None:
        bundle = json.loads(args.staget_bundle.read_text(encoding="utf-8"))
        output["staget_bundle"] = verify_staget_bundle(bundle)
    if args.decode_bench_bundle is not None:
        bundle = json.loads(args.decode_bench_bundle.read_text(encoding="utf-8"))
        output["decode_bench_bundle"] = verify_decode_bench_bundle(bundle)
    if args.self_test:
        output["self_test"] = run_self_tests(args.repo_root)

    if not output:
        output["repo_contract"] = verify_repo_and_sheet_contracts(args.repo_root)
        output["self_test"] = run_self_tests(args.repo_root)

    print(json.dumps(output, indent=2, sort_keys=True))
    return 0


if __name__ == "__main__":
    sys.exit(main())
