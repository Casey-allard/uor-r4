#!/usr/bin/env python3
"""
Standalone Host-Only Consumer Verifier for W2 Read-Code Capacity Diagnostic (D39 item 4 / #2572)
Schema: uor-r4.read-code-capacity/1
Sheet: docs/harness/prep/m1-read-code-capacity-2026-10-17.md (sha256: 1fd08b79dee4f5ff5812ff9ed5221700a7b0ee7bc760cdb00b14b2f4cefedc89)
Readiness Map: w2-bvg-source-readiness-20261018/REPORT.md (sha256: 428d9bead6733593447a7bb44eecad8ef6a6d064cf6e51e913345cd925c180f4)

Strictly host-only ($0 GPU). Verifies stored/emitted records, canonical UORT 64-byte header
and PreparedCorpus eligibility semantics (crates/uor-r4-discrete/src/corpus.rs:143-297),
authentic original Q4 model/kernel/driver/domain pins (e1836149..., 931a511b..., cb090e85...,
49fa448c..., f43742cb...) with explicit rejection of K16 qualification graph 035b3c... and
fail-closed runtime-manifest verification, 224-batch evaluation-16404.json (4fff1d3c...)
flattening with exact target_admitted = admitted_cells - address_rivals (1,920), lossless
DEV/TRAIN row and count-table conservation, exact nibble order, and Precedence 0..5 evaluation.
Note: Complete standalone D27 Rust validator & CLI is in verify_w2_read_code_capacity.rs.
"""

from __future__ import annotations

import argparse
from bisect import bisect_right
import hashlib
import json
import math
import pathlib
import sys
from typing import Any, Dict, List, Optional, Sequence, Tuple

READ_CODE_CAPACITY_SCHEMA = "uor-r4.read-code-capacity/1"
DRIVER_SCHEMA = "uor-r4.discrete-corpus-driver/1"
COMPLETION_EVAL_SCHEMA = "uor-r4.completion-evaluation/1"

# Section 4.9 Pinned Identities (m1-read-code-capacity-2026-10-17.md & Codex W2 handoffs)
PINNED_SHEET_SHA256 = "1fd08b79dee4f5ff5812ff9ed5221700a7b0ee7bc760cdb00b14b2f4cefedc89"
PINNED_READINESS_MAP_SHA256 = "428d9bead6733593447a7bb44eecad8ef6a6d064cf6e51e913345cd925c180f4"
PINNED_INTERFACE_PREP_REPORT_SHA256 = "e8360545e43ce22cf820fbcb077f49acafe51f72efb22b4354056f82fe6bfdd3"
PINNED_INTERFACE_PREP_MANIFEST_SHA256 = "c73bfeff532c5cd7437f0c09722a017d8f37324834e247d261f2aba598ad5f24"
PINNED_INTERFACE_REVIEW_SHA256 = "a51226c82d8665afd3b4f5ca07a3b642684d3b3c22fa584299152e6fafef8e72"
PINNED_RETAINED_INPUT_RECEIPT_SHA256 = "c020d3c01ae447b77b7eb69b9eaaa01ca439d995653795cd7e2db2265029d2b6"

# Authenticated Historical Q4 Model / Kernel / Driver / Domain Identities (6255411191 item 1)
PINNED_ORIGINAL_SOURCE_SHA256 = "e18361495276e491525674fb3fdce308b0ad787c7fd6860a811adb60dacd9e1a"
PINNED_ORIGINAL_KERNEL_SHA256 = "931a511bece20641ae00c6b0e943f4f2a380c0f35a824d85fcc0447f1e1ccf5b"
PINNED_ORIGINAL_DRIVER_SHA256 = "cb090e85e78805acd2b2347373340168a9e2f87aaf72a1dcb42f091e9ff1f7b4"
PINNED_ORIGINAL_MODEL_DOMAIN_A_SHA256 = "49fa448cb3349ac5711966082a9c53918a693d45e11f282221d240cbb463fa73"
PINNED_ORIGINAL_SNAPSHOT_DOMAIN_B_SHA256 = "f43742cb951b974856747fc3c162ff9cac1b4477df5296998481800e3a6c0512"
FORBIDDEN_K16_QUAL_SOURCE_SHA256 = "035b3c127bb5a0eb84e50901cabb09391922e40ec6ffb3b4f638abccf6930b85"

PINNED_Q1_CHECKPOINT_SHA256 = "a5a22dce8a174bbfe368cd8d166170a6c34bf59137b1418943a9d0422cae4cdd"
PINNED_Q2_CHECKPOINT_SHA256 = "dde889aa6137c0e28e18dd5ae99a823b6850342286a0d065bdaa82de49b05822"
PINNED_Q3_CHECKPOINT_SHA256 = "102beed4608f1ddd70d68039bdae8d9372b3c05de2dbc4bc98b96d7595e40982"
PINNED_Q4_CHECKPOINT_SHA256 = "4477eb5c2480b0b85cc94036a2e15b0d22b654e25deee9c8717aa6e30f8ef7bf"
PINNED_Q4_EVALUATION_SHA256 = "4fff1d3c04ff6107666b2b8281ef2bbedf3019eeff6bff682bbbb8687bf888c8"
PINNED_Q4_EVALUATION_BYTES = 2906559
PINNED_DEV_TOKENS_SHA256 = "0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b"
PINNED_DEV_MASK_SHA256 = "a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078"
PINNED_TRAIN_TOKENS_SHA256 = "3dee858f84b750f507ceb3dc3f393711741950954735efaaf61bf704c017a4c8"
PINNED_TRAIN_MASK_SHA256 = "b5e7352d67d6883a833155a5fd397a03f6d4f1b2649e3d4ec3e2ea7d95b949e3"
PINNED_TOKENIZER_SHA256 = "d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89"
PINNED_ORDERED_ROW_BINDING_SHA256 = "e4ef44fb81d6c3a833212e4857117db71cafb8259b81c286e5b1391129d74446"
PINNED_GEOMETRY_SHA256 = "46ddbb76b7fbe6ac6222ea24779fc10055b1d6cea6a8da786895676fd29fe327"
PINNED_D34_RECORD_SHA256 = "2717d34bcfcdd1486991925ecab066266a6b56ba34b89727c37f78c821fe3bec"

# Canonical UORT & PreparedCorpus Constants (crates/uor-r4-discrete/src/corpus.rs:143-297)
UORT_MAGIC = b"UORT"
UORT_VERSION = 1
UORT_HEADER_BYTES = 64
UORT_RESERVED_ZERO = b"\x00" * 44
PINNED_BOS_TOKEN = "<|bos|>"
PINNED_BOS_ID = 0
CONTEXT_WINDOW = 192
SPLIT_TAG_TRAIN = 0
SPLIT_TAG_DEV = 1
SPLIT_TAG_FRESH = 2

TRAIN_TOKENS_BYTES = 15025658       # 64 + 2 * 7,512,797
TRAIN_PAYLOAD_TOKENS = 7512797
TRAIN_MASK_BYTES = 7512797
TRAIN_DOCUMENTS = 60000

DEV_TOKENS_BYTES = 85776            # 64 + 2 * 42,856
DEV_PAYLOAD_TOKENS = 42856
DEV_MASK_BYTES = 42856
DEV_DOCUMENTS = 400

# Pinned Counts & Constants
DEV_ROWS = 14287
DEV_BATCH_ROWS = 64
DEV_EXPECTED_BATCHES = 224          # 223 * 64 + 15 = 14,287
DEV_LAST_BATCH_ROWS = 15
MATCHED_TRAIN_TARGETS = 1049856
FULL_TRAIN_TARGETS = 2425925
COMPACT_TRAIN_PRIMARY_BYTES = 4199424  # 1,049,856 * 4 bytes (u16 code + u16 target, little-endian)
COMPACT_TRAIN_FULL_BYTES = 9703700     # 2,425,925 * 4 bytes (u16 code + u16 target, little-endian)
QUERY_SIDECAR_PRIMARY_BYTES = 2099712  # 1,049,856 * 2 bytes (u16 query token sidecar, little-endian)
QUERY_SIDECAR_FULL_BYTES = 4851850     # 2,425,925 * 2 bytes (u16 query token sidecar, little-endian)
SENTINEL_U16 = 0xFFFF                  # exact sentinel conversion u32::MAX -> u16::FFFF; cell 255 remains valid
VOCABULARY = 4096
MAX_CODE_COUNT = 65536
Q4_EPOCH = 407
Q4_COMPLETED_BATCHES = 16404
Q4_WRITER_TOP1 = 149
Q4_WRITER_TOP8 = 747
Q4_WRITER_ADMITTED = 1920
FULL_TRAIN_UNIGRAM_TOP1 = 1226
FULL_TRAIN_UNIGRAM_TOP8 = 4060
MIN_COVERAGE_ROWS = 12859  # ceil(0.90 * 14287) = 12859 (90.0049%)
MIN_MULTIPLICITY_CODES = 1024
MIN_DISCORDANCE_NORMAL = 100
FULL_PANEL_MIN_FLOOR = 24
TOP8_MIN_FLOOR = 63
PSUCC_SUBPANEL_ROWS = 3847
PSUCC_TOP8_FLOOR = 39
COMPLEMENT_SUBPANEL_ROWS = 10440
CONFLICT_SUBPANEL_ROWS = 1335


class W2VerificationError(ValueError):
    """Raised when a W2 read-code capacity record fails schema, pin, or conservation checks."""


class W2InstrumentFault(W2VerificationError):
    """Raised or classified when Precedence 0 (INSTRUMENT FAULT, D31) triggers."""


def is_strict_int(value: Any) -> bool:
    return isinstance(value, int) and not isinstance(value, bool)


def require_strict_int(
    value: Any,
    field_name: str,
    min_value: Optional[int] = None,
    max_value: Optional[int] = None,
) -> int:
    if not is_strict_int(value):
        raise W2VerificationError(f"invalid {field_name}: expected strict integer, got {value!r}")
    if min_value is not None and value < min_value:
        raise W2VerificationError(f"invalid {field_name}: {value} < minimum {min_value}")
    if max_value is not None and value > max_value:
        raise W2VerificationError(f"invalid {field_name}: {value} > maximum {max_value}")
    return int(value)


def is_hex64(value: Any) -> bool:
    if not isinstance(value, str) or len(value) != 64:
        return False
    return all(c in "0123456789abcdef" for c in value)


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def parse_uort_header(
    header_bytes: bytes,
    *,
    expected_payload_tokens: Optional[int] = None,
    expected_vocabulary: int = VOCABULARY,
) -> Dict[str, int]:
    """Validates the canonical 64-byte UORT header per crates/uor-r4-discrete/src/corpus.rs:155-189."""
    if len(header_bytes) < UORT_HEADER_BYTES:
        raise W2VerificationError(
            f"token file is shorter than the 64-byte UORT header (len={len(header_bytes)})"
        )
    if header_bytes[0:4] != UORT_MAGIC:
        raise W2VerificationError("token file does not start with UORT magic")
    version = int.from_bytes(header_bytes[4:8], "little")
    if version != UORT_VERSION:
        raise W2VerificationError(f"unsupported UORT version {version} != {UORT_VERSION}")
    count = int.from_bytes(header_bytes[8:16], "little")
    if count <= 0:
        raise W2VerificationError("UORT token count is zero")
    if expected_payload_tokens is not None and count != expected_payload_tokens:
        raise W2VerificationError(
            f"UORT header token count {count} != expected {expected_payload_tokens}"
        )
    vocabulary = int.from_bytes(header_bytes[16:20], "little")
    if vocabulary != expected_vocabulary:
        raise W2VerificationError(
            f"UORT header vocabulary {vocabulary} != expected {expected_vocabulary}"
        )
    if header_bytes[20:64] != UORT_RESERVED_ZERO:
        raise W2VerificationError("UORT header reserved bytes 20..64 are non-zero")
    return {
        "version": version,
        "count": count,
        "vocabulary": vocabulary,
        "expected_file_bytes": UORT_HEADER_BYTES + 2 * count,
    }


def parse_prepared_corpus_uort(
    tokens_bytes: bytes,
    mask_bytes: bytes,
    *,
    bos_id: int = PINNED_BOS_ID,
    bos_token: str = PINNED_BOS_TOKEN,
    split_tag: int = SPLIT_TAG_TRAIN,
    context: int = CONTEXT_WINDOW,
    expected_payload_tokens: Optional[int] = None,
    expected_documents: Optional[int] = None,
    expected_eligible_targets: Optional[int] = None,
    limit_rows: Optional[int] = None,
) -> Dict[str, Any]:
    """Reuses exact canonical PreparedCorpus::from_bytes reader and eligibility semantics
    (crates/uor-r4-discrete/src/corpus.rs:143-297):
    - Validates 64-byte UORT header (magic, version=1, count, vocabulary=4096, reserved 20..64 zero).
    - Validates tokens_bytes.len() == 64 + 2 * count and slices bytes[64..].
    - Validates mask_bytes.len() == count (exact binary mask length) and every mask byte in {0, 1}.
    - Validates pinned BOS identity (token='<|bos|>', id=0) and tokens[0] == bos_id.
    - Builds documents = [p for p, id in enumerate(tokens) if id == bos_id] (< 2^62).
    - Selects eligible = [p for p, (id, m) in enumerate(zip(tokens, mask)) if p > 0 and id != bos_id and m == 1].
    - For each eligible target position, locates owning document via bisect_right(documents, position) - 1,
      validates start = max(0, position - context, doc_start) < position (in-document predecessor),
      and binds query_occurrence = position - 1, query_token = tokens[position - 1],
      target_position = position, target_token = tokens[position], document_id = doc,
      and split-tagged record = (split_tag << 62) | doc.
    """
    if sys.byteorder != "little":
        raise W2VerificationError("host byteorder must be little-endian for u16 memoryview cast")
    if bos_id != PINNED_BOS_ID or bos_token != PINNED_BOS_TOKEN:
        raise W2VerificationError(
            f"BOS identity ({bos_token!r}, {bos_id}) != pinned ({PINNED_BOS_TOKEN!r}, {PINNED_BOS_ID})"
        )
    if context <= 0:
        raise W2VerificationError(f"context window must be > 0, got {context}")

    hdr = parse_uort_header(
        tokens_bytes[:UORT_HEADER_BYTES],
        expected_payload_tokens=expected_payload_tokens,
        expected_vocabulary=VOCABULARY,
    )
    count = hdr["count"]
    vocabulary = hdr["vocabulary"]
    if len(tokens_bytes) != hdr["expected_file_bytes"]:
        raise W2VerificationError(
            f"UORT token file byte length {len(tokens_bytes)} != 64 + 2 * {count} ({hdr['expected_file_bytes']})"
        )
    if len(mask_bytes) != count:
        raise W2VerificationError(
            f"response mask length {len(mask_bytes)} != UORT token count {count}"
        )

    payload_mv = memoryview(tokens_bytes)[UORT_HEADER_BYTES:].cast("H")
    mask_mv = memoryview(mask_bytes)
    if len(payload_mv) != count:
        raise W2VerificationError(f"UORT payload u16 length {len(payload_mv)} != {count}")

    if int(payload_mv[0]) != bos_id:
        raise W2VerificationError(
            f"corpus does not start with BOS token id {bos_id} (tokens[0]={int(payload_mv[0])})"
        )

    documents: List[int] = []
    eligible: List[int] = []
    for p in range(count):
        tok_id = int(payload_mv[p])
        if tok_id < 0 or tok_id >= vocabulary:
            raise W2VerificationError(
                f"token id {tok_id} at position {p} >= vocabulary {vocabulary}"
            )
        m = int(mask_mv[p])
        if m != 0 and m != 1:
            raise W2VerificationError(
                f"response mask byte {m} at position {p} is not binary (0 or 1)"
            )
        if tok_id == bos_id:
            documents.append(p)
        elif p > 0 and m == 1:
            eligible.append(p)

    if not documents:
        raise W2VerificationError("corpus contains no BOS document starts")
    if len(documents) >= (1 << 62):
        raise W2VerificationError("document count exceeds 62-bit split-prefixed record id space")
    if expected_documents is not None and len(documents) != expected_documents:
        raise W2VerificationError(
            f"document count {len(documents)} != expected {expected_documents}"
        )

    if not eligible:
        raise W2VerificationError("corpus contains no eligible masked response targets")
    if expected_eligible_targets is not None and len(eligible) != expected_eligible_targets:
        raise W2VerificationError(
            f"eligible target count {len(eligible)} != expected {expected_eligible_targets}"
        )

    row_limit = len(eligible) if limit_rows is None else min(limit_rows, len(eligible))
    if limit_rows is not None and limit_rows > len(eligible):
        raise W2VerificationError(
            f"requested limit_rows {limit_rows} > eligible targets {len(eligible)}"
        )

    rows: List[Dict[str, int]] = []
    for ordinal in range(row_limit):
        position = eligible[ordinal]
        doc = bisect_right(documents, position) - 1
        if doc < 0 or doc >= len(documents):
            raise W2VerificationError(
                f"eligible target ordinal {ordinal} at position {position} has invalid document index {doc}"
            )
        doc_start = documents[doc]
        start = max(0, position - context, doc_start)
        if start >= position:
            raise W2VerificationError(
                f"eligible target ordinal {ordinal} at position {position} has no in-document predecessor (doc_start={doc_start})"
            )
        query_occ = position - 1
        q_tok = int(payload_mv[query_occ])
        t_tok = int(payload_mv[position])
        record_id = ((split_tag & 0x3) << 62) | doc
        rows.append(
            {
                "row_ordinal": ordinal,
                "document_id": doc,
                "record": record_id,
                "doc_start": doc_start,
                "context_start": start,
                "query_occurrence": query_occ,
                "query_token": q_tok,
                "target_position": position,
                "target_token": t_tok,
            }
        )

    return {
        "uort_version": hdr["version"],
        "payload_tokens": count,
        "vocabulary": vocabulary,
        "bos_token": bos_token,
        "bos_id": bos_id,
        "document_count": len(documents),
        "eligible_target_count": len(eligible),
        "rows_materialized": len(rows),
        "rows": rows,
    }


def wilson_95_interval(k: int, n: int, z: float = 1.959963984540054) -> Tuple[float, float]:
    """Computes the Wilson 95% confidence interval for k hits in n trials (§4.6)."""
    if n <= 0 or k < 0 or k > n:
        raise W2VerificationError(f"invalid Wilson parameters k={k}, n={n}")
    p = k / n
    z2 = z * z
    denom = 1.0 + z2 / n
    center = (p + z2 / (2.0 * n)) / denom
    half = (z / denom) * math.sqrt((p * (1.0 - p) / n) + (z2 / (4.0 * n * n)))
    return (max(0.0, center - half), min(1.0, center + half))


def exact_two_sided_mcnemar_pvalue(b: int, c: int) -> float:
    """Computes the exact two-sided binomial (McNemar) p-value under p=0.5 for n = b + c (§4.6)."""
    if b < 0 or c < 0:
        raise W2VerificationError(f"negative discordance counts b={b}, c={c}")
    n = b + c
    if n == 0:
        return 1.0
    k = min(b, c)
    tail = sum(math.comb(n, i) for i in range(0, k + 1)) / (2 ** n)
    return min(1.0, 2.0 * tail)


def paired_floor(b: int, c: int, min_floor: int = FULL_PANEL_MIN_FLOOR) -> float:
    """F_X = max(min_floor, 2 * sqrt(b + c)) (§4.2, §4.6)."""
    return max(float(min_floor), 2.0 * math.sqrt(b + c))


def exceeds_paired_floor_strict(diff: int, b_plus_c: int, min_floor: int = FULL_PANEL_MIN_FLOOR) -> bool:
    """Exact integer predicate for T > X + max(min_floor, 2*sqrt(b+c)):
    Since diff = T - X is an integer:
      diff > min_floor AND diff > 2*sqrt(b+c) <=> diff > min_floor AND diff^2 > 4*(b+c).
    """
    return diff > min_floor and (diff * diff) > 4 * b_plus_c


def ge_paired_floor(diff: int, b_plus_c: int, min_floor: int = FULL_PANEL_MIN_FLOOR) -> bool:
    """Exact integer predicate for T >= X + max(min_floor, 2*sqrt(b+c)):
    Since diff = T - X is an integer:
      diff >= min_floor AND diff >= 2*sqrt(b+c) <=> diff >= min_floor AND diff^2 >= 4*(b+c).
    """
    return diff >= min_floor and (diff * diff) >= 4 * b_plus_c


def ge_minus_paired_floor(diff: int, b_plus_c: int, min_floor: int = FULL_PANEL_MIN_FLOOR) -> bool:
    """Exact integer predicate for T >= W - max(min_floor, 2*sqrt(b_W+c_W)):
    Let deficit = W - T = -diff.
    T >= W - H <=> deficit <= H = max(min_floor, 2*sqrt(b_W+c_W))
               <=> deficit <= 0 OR deficit <= min_floor OR deficit^2 <= 4*(b_W+c_W).
    """
    deficit = -diff
    if deficit <= 0:
        return True
    return deficit <= min_floor or (deficit * deficit) <= 4 * b_plus_c


def verify_runtime_manifest_bytes(raw_bytes: bytes) -> Dict[str, str]:
    """Verifies an authenticated runtime manifest JSON file on disk (6255411191 item 1):
    - Requires original_source_sha256 == PINNED_ORIGINAL_SOURCE_SHA256 (e1836149...),
      original_kernel_sha256 == PINNED_ORIGINAL_KERNEL_SHA256 (931a511b...),
      original_operational_driver_sha256 == PINNED_ORIGINAL_DRIVER_SHA256 (cb090e85...),
      original_model_domain_a_sha256 == PINNED_ORIGINAL_MODEL_DOMAIN_A_SHA256 (49fa448c...).
    - Requires bound 64-hex target_compiled_source_sha256, target_kernel_sha256, target_binary_sha256,
      and explicitly rejects K16 qualification graph FORBIDDEN_K16_QUAL_SOURCE_SHA256 (035b3c...).
    """
    doc = json.loads(raw_bytes.decode("utf-8"))
    if not isinstance(doc, dict):
        raise W2VerificationError("runtime manifest root must be a JSON object")
    orig_src = doc.get("original_source_sha256")
    orig_ker = doc.get("original_kernel_sha256")
    orig_drv = doc.get("original_operational_driver_sha256")
    orig_dom_a = doc.get("original_model_domain_a_sha256")
    if orig_src != PINNED_ORIGINAL_SOURCE_SHA256:
        raise W2VerificationError(
            f"runtime manifest original_source_sha256 {orig_src!r} != authenticated Q4 {PINNED_ORIGINAL_SOURCE_SHA256!r}"
        )
    if orig_ker != PINNED_ORIGINAL_KERNEL_SHA256:
        raise W2VerificationError(
            f"runtime manifest original_kernel_sha256 {orig_ker!r} != authenticated Q4 {PINNED_ORIGINAL_KERNEL_SHA256!r}"
        )
    if orig_drv != PINNED_ORIGINAL_DRIVER_SHA256:
        raise W2VerificationError(
            f"runtime manifest original_operational_driver_sha256 {orig_drv!r} != authenticated Q4 {PINNED_ORIGINAL_DRIVER_SHA256!r}"
        )
    if orig_dom_a != PINNED_ORIGINAL_MODEL_DOMAIN_A_SHA256:
        raise W2VerificationError(
            f"runtime manifest original_model_domain_a_sha256 {orig_dom_a!r} != authenticated Q4 {PINNED_ORIGINAL_MODEL_DOMAIN_A_SHA256!r}"
        )

    tgt_src = doc.get("target_compiled_source_sha256")
    tgt_ker = doc.get("target_kernel_sha256")
    tgt_bin = doc.get("target_binary_sha256")
    for k, v in (
        ("target_compiled_source_sha256", tgt_src),
        ("target_kernel_sha256", tgt_ker),
        ("target_binary_sha256", tgt_bin),
    ):
        if not is_hex64(v):
            raise W2VerificationError(f"runtime manifest {k} must be bound 64-hex, got {v!r}")
    if tgt_src == FORBIDDEN_K16_QUAL_SOURCE_SHA256:
        raise W2VerificationError(
            "runtime manifest target_compiled_source_sha256 cannot substitute K16 qualification graph 035b3c..."
        )
    return {
        "manifest_sha256": sha256_bytes(raw_bytes),
        "original_source_sha256": str(orig_src),
        "original_kernel_sha256": str(orig_ker),
        "original_driver_sha256": str(orig_drv),
        "original_model_domain_a_sha256": str(orig_dom_a),
        "target_compiled_source_sha256": str(tgt_src),
        "target_kernel_sha256": str(tgt_ker),
        "target_binary_sha256": str(tgt_bin),
    }


def verify_instrument_validity(
    record: Dict[str, Any],
    *,
    strict_k8_q4_pins: bool = True,
    require_bound_runtime_pins: bool = True,
    runtime_manifest: Optional[Dict[str, str]] = None,
) -> Dict[str, Any]:
    """Verifies §4.8 Instrument Validity (Precedence 0), including original->target source bridge,
    authenticated runtime manifest binding, and producer file pins.
    Returns a dict with `instrument_fault: bool` and `fault_reasons: List[str]`.
    """
    faults: List[str] = []

    if record.get("schema") != READ_CODE_CAPACITY_SCHEMA:
        faults.append(f"schema {record.get('schema')!r} != {READ_CODE_CAPACITY_SCHEMA!r}")

    if record.get("zero_apply") is not True:
        faults.append("zero_apply must be true")
    if record.get("applied_updates") != 0:
        faults.append(f"applied_updates must be 0, got {record.get('applied_updates')!r}")
    if record.get("updates_applied", 0) != 0:
        faults.append(f"updates_applied must be 0, got {record.get('updates_applied')!r}")
    if record.get("snapshot_unchanged") is not True:
        faults.append("snapshot_unchanged must be true")
    if record.get("model_compute_executed") is not True:
        faults.append("model_compute_executed must be true")

    if strict_k8_q4_pins:
        if record.get("epoch") != Q4_EPOCH:
            faults.append(f"epoch {record.get('epoch')!r} != {Q4_EPOCH}")
        if record.get("checkpoint_sha256") != PINNED_Q4_CHECKPOINT_SHA256:
            faults.append(
                f"checkpoint_sha256 {record.get('checkpoint_sha256')!r} != {PINNED_Q4_CHECKPOINT_SHA256!r}"
            )
        if record.get("panel_binding_sha256") != PINNED_ORDERED_ROW_BINDING_SHA256:
            faults.append(
                f"panel_binding_sha256 {record.get('panel_binding_sha256')!r} != {PINNED_ORDERED_ROW_BINDING_SHA256!r}"
            )

    rec_src = record.get("source_sha256")
    if not is_hex64(rec_src):
        faults.append("missing or invalid 64-hex source_sha256")
    if rec_src == FORBIDDEN_K16_QUAL_SOURCE_SHA256:
        faults.append("source_sha256 cannot substitute K16 qualification graph 035b3c... for W2")
    if not is_hex64(record.get("checkpoint_sha256")):
        faults.append("missing or invalid 64-hex checkpoint_sha256")
    if not is_hex64(record.get("panel_binding_sha256")):
        faults.append("missing or invalid 64-hex panel_binding_sha256")

    # Original -> target source bridge & authenticated runtime manifest binding (6255411191 item 1)
    bridge = record.get("original_to_target_bridge")
    if not isinstance(bridge, dict):
        if require_bound_runtime_pins:
            faults.append("missing original_to_target_bridge object required for verified runtime mode")
    else:
        if bridge.get("original_source_sha256") != PINNED_ORIGINAL_SOURCE_SHA256:
            faults.append(
                f"original_to_target_bridge.original_source_sha256 {bridge.get('original_source_sha256')!r} != authenticated Q4 {PINNED_ORIGINAL_SOURCE_SHA256!r}"
            )
        if bridge.get("original_kernel_sha256") != PINNED_ORIGINAL_KERNEL_SHA256:
            faults.append(
                f"original_to_target_bridge.original_kernel_sha256 {bridge.get('original_kernel_sha256')!r} != authenticated Q4 {PINNED_ORIGINAL_KERNEL_SHA256!r}"
            )
        target_src = bridge.get("target_compiled_source_sha256")
        target_ker = bridge.get("target_kernel_sha256")
        target_bin = bridge.get("target_binary_sha256")
        rt_manifest = bridge.get("runtime_manifest_sha256")
        if require_bound_runtime_pins:
            if not is_hex64(target_src):
                faults.append(
                    f"target_compiled_source_sha256 must be bound 64-hex in runtime mode, got {target_src!r}"
                )
            elif target_src == FORBIDDEN_K16_QUAL_SOURCE_SHA256:
                faults.append(
                    "target_compiled_source_sha256 cannot substitute K16 qualification graph 035b3c..."
                )
            elif rec_src != target_src:
                faults.append(
                    f"record.source_sha256 {rec_src!r} != bridge.target_compiled_source_sha256 {target_src!r}"
                )
            if not is_hex64(target_ker):
                faults.append(
                    f"target_kernel_sha256 must be bound 64-hex in runtime mode, got {target_ker!r}"
                )
            if not is_hex64(target_bin):
                faults.append(
                    f"target_binary_sha256 must be bound 64-hex in runtime mode, got {target_bin!r}"
                )
            if not is_hex64(rt_manifest):
                faults.append(
                    f"runtime_manifest_sha256 must be bound 64-hex in runtime mode, got {rt_manifest!r}"
                )
            if runtime_manifest is None:
                faults.append(
                    "verified runtime mode requires an authenticated runtime manifest file (--runtime-manifest)"
                )
            else:
                if runtime_manifest.get("manifest_sha256") != rt_manifest:
                    faults.append("runtime manifest file SHA-256 != bridge.runtime_manifest_sha256")
                if runtime_manifest.get("target_compiled_source_sha256") != target_src:
                    faults.append(
                        "runtime manifest target_compiled_source_sha256 != bridge.target_compiled_source_sha256"
                    )
                if runtime_manifest.get("target_kernel_sha256") != target_ker:
                    faults.append(
                        "runtime manifest target_kernel_sha256 != bridge.target_kernel_sha256"
                    )
                if runtime_manifest.get("target_binary_sha256") != target_bin:
                    faults.append(
                        "runtime manifest target_binary_sha256 != bridge.target_binary_sha256"
                    )
        else:
            for field_key, field_val in (
                ("target_compiled_source_sha256", target_src),
                ("target_kernel_sha256", target_ker),
                ("target_binary_sha256", target_bin),
                ("runtime_manifest_sha256", rt_manifest),
            ):
                if field_val != "UNBOUND" and not is_hex64(field_val):
                    faults.append(f"bridge.{field_key} must be 'UNBOUND' or 64-hex, got {field_val!r}")
                if field_val == FORBIDDEN_K16_QUAL_SOURCE_SHA256:
                    faults.append(
                        f"bridge.{field_key} cannot substitute K16 qualification graph 035b3c..."
                    )

    val = record.get("instrument_validity")
    if not isinstance(val, dict):
        faults.append("missing instrument_validity object (§4.8)")
        return {"instrument_fault": True, "fault_reasons": faults, "capture_mode": "unknown"}

    # 1. Root round-trip (§4.8 item 1)
    rrt = val.get("root_round_trip")
    if not isinstance(rrt, dict):
        faults.append("missing instrument_validity.root_round_trip")
    else:
        if rrt.get("sample_rows", 0) < 1024:
            faults.append(f"root_round_trip.sample_rows {rrt.get('sample_rows')!r} < 1024")
        if rrt.get("mismatches") != 0:
            faults.append(f"root_round_trip.mismatches {rrt.get('mismatches')!r} != 0")
        if rrt.get("absent_roots") != 0:
            faults.append(f"root_round_trip.absent_roots {rrt.get('absent_roots')!r} != 0")

    # 2. TRAIN capture agreement or fallback disclosure (§4.5, §4.8 item 2)
    tca = val.get("train_capture")
    capture_mode = "unknown"
    if not isinstance(tca, dict):
        faults.append("missing instrument_validity.train_capture")
    else:
        capture_mode = str(tca.get("mode", ""))
        if capture_mode == "q_only_2425925":
            if tca.get("sample_rows", 0) < 1024:
                faults.append(f"train_capture.sample_rows {tca.get('sample_rows')!r} < 1024")
            if tca.get("production_forward_disagreements") != 0:
                faults.append(
                    f"train_capture.production_forward_disagreements {tca.get('production_forward_disagreements')!r} != 0"
                )
        elif capture_mode == "production_forward_matched_1049856_fallback":
            if tca.get("smaller_table_bias_disclosed") is not True:
                faults.append(
                    "production_forward_matched_1049856_fallback requires smaller_table_bias_disclosed=true (§4.5)"
                )
        else:
            faults.append(f"unrecognized train_capture.mode {capture_mode!r}")

    # 3. Sealed-receipt agreement (§4.8 item 3)
    sra = val.get("sealed_receipt_agreement")
    if not isinstance(sra, dict):
        faults.append("missing instrument_validity.sealed_receipt_agreement")
    else:
        if sra.get("dev_rows") != DEV_ROWS:
            faults.append(f"sealed_receipt_agreement.dev_rows {sra.get('dev_rows')!r} != {DEV_ROWS}")
        if sra.get("row_disagreements") != 0:
            faults.append(
                f"sealed_receipt_agreement.row_disagreements {sra.get('row_disagreements')!r} != 0"
            )
        if strict_k8_q4_pins:
            if sra.get("evaluation_sha256") != PINNED_Q4_EVALUATION_SHA256:
                faults.append(
                    f"sealed_receipt_agreement.evaluation_sha256 {sra.get('evaluation_sha256')!r} != {PINNED_Q4_EVALUATION_SHA256!r}"
                )
            if sra.get("writer_top1") != Q4_WRITER_TOP1:
                faults.append(
                    f"sealed_receipt_agreement.writer_top1 {sra.get('writer_top1')!r} != {Q4_WRITER_TOP1}"
                )
            if sra.get("writer_top8") != Q4_WRITER_TOP8:
                faults.append(
                    f"sealed_receipt_agreement.writer_top8 {sra.get('writer_top8')!r} != {Q4_WRITER_TOP8}"
                )
            if sra.get("writer_admitted") != Q4_WRITER_ADMITTED:
                faults.append(
                    f"sealed_receipt_agreement.writer_admitted {sra.get('writer_admitted')!r} != {Q4_WRITER_ADMITTED}"
                )

    # 4. Count-table conservation (§4.8 item 4)
    ctc = val.get("count_table_conservation")
    if not isinstance(ctc, dict):
        faults.append("missing instrument_validity.count_table_conservation")
    else:
        if ctc.get("matched_train_total") != MATCHED_TRAIN_TARGETS:
            faults.append(
                f"matched_train_total {ctc.get('matched_train_total')!r} != {MATCHED_TRAIN_TARGETS}"
            )
        if ctc.get("bigram_train_total") != MATCHED_TRAIN_TARGETS:
            faults.append(
                f"bigram_train_total {ctc.get('bigram_train_total')!r} != {MATCHED_TRAIN_TARGETS}"
            )
        if capture_mode == "q_only_2425925" and ctc.get("full_train_total") != FULL_TRAIN_TARGETS:
            faults.append(
                f"full_train_total {ctc.get('full_train_total')!r} != {FULL_TRAIN_TARGETS}"
            )
        if ctc.get("invalid_tokens") != 0:
            faults.append(f"invalid_tokens {ctc.get('invalid_tokens')!r} != 0")
        if ctc.get("negative_counts") != 0:
            faults.append(f"negative_counts {ctc.get('negative_counts')!r} != 0")

    # 5. Determinism & Symmetry root validity (§4.4 item 6, §4.8 items 6, 8)
    det = val.get("determinism")
    if not isinstance(det, dict):
        faults.append("missing instrument_validity.determinism")
    else:
        if det.get("repeat_rows", 0) < 1000 or det.get("byte_identical") is not True:
            faults.append("determinism requires repeat_rows >= 1000 and byte_identical == true")

    if val.get("symmetry_action_non_roots", 0) != 0:
        faults.append(
            f"symmetry_action_non_roots {val.get('symmetry_action_non_roots')!r} != 0 (§4.4 item 6)"
        )
    if val.get("cuda_or_status_fault", False) is not False:
        faults.append("cuda_or_status_fault reported true")
    if val.get("d23_d25_d32_ratchets_pass") is not True:
        faults.append("d23_d25_d32_ratchets_pass must be true (§4.8 item 8)")

    # 6. Producer file pins check when require_bound_runtime_pins=True (6255245665 item 3)
    pins = record.get("producer_file_pins")
    if require_bound_runtime_pins:
        if not isinstance(pins, dict):
            faults.append("missing producer_file_pins object required in verified runtime mode")
        else:
            if pins.get("dev_tokens_sha256") != PINNED_DEV_TOKENS_SHA256:
                faults.append("producer_file_pins.dev_tokens_sha256 mismatch")
            if pins.get("dev_mask_sha256") != PINNED_DEV_MASK_SHA256:
                faults.append("producer_file_pins.dev_mask_sha256 mismatch")
            if pins.get("train_tokens_sha256") != PINNED_TRAIN_TOKENS_SHA256:
                faults.append("producer_file_pins.train_tokens_sha256 mismatch")
            if pins.get("train_mask_sha256") != PINNED_TRAIN_MASK_SHA256:
                faults.append("producer_file_pins.train_mask_sha256 mismatch")
            if pins.get("tokenizer_sha256") != PINNED_TOKENIZER_SHA256:
                faults.append("producer_file_pins.tokenizer_sha256 mismatch")
            if pins.get("bos_token") != PINNED_BOS_TOKEN or pins.get("bos_id") != PINNED_BOS_ID:
                faults.append("producer_file_pins BOS identity mismatch")
            if not is_hex64(pins.get("compact_train_stream_sha256")):
                faults.append("producer_file_pins.compact_train_stream_sha256 must be 64-hex")
            expected_stream_bytes = (
                COMPACT_TRAIN_FULL_BYTES
                if capture_mode == "q_only_2425925"
                else COMPACT_TRAIN_PRIMARY_BYTES
            )
            if pins.get("compact_train_stream_bytes") != expected_stream_bytes:
                faults.append(
                    f"producer_file_pins.compact_train_stream_bytes {pins.get('compact_train_stream_bytes')!r} != {expected_stream_bytes} for mode {capture_mode!r}"
                )

    return {
        "instrument_fault": len(faults) > 0,
        "fault_reasons": faults,
        "capture_mode": capture_mode,
    }


def flatten_sealed_evaluation_batches(
    eval_doc: Dict[str, Any],
    dev_corpus_rows: Sequence[Dict[str, int]],
    *,
    strict_k8_q4_pins: bool = True,
) -> List[Dict[str, Any]]:
    """Flattens and validates the real 224-batch (`223 x 64 + 15 = 14,287` rows) sealed Q4
    evaluation-16404.json (`4fff1d3c...`, 2,906,559 B) per 6255411191 items 2 & 3:
      - Requires `schema == "uor-r4.discrete-corpus-driver/1"`, `evaluation_mutates_model == false`,
        `updates_applied_by_evaluation == 0`, and `model_epoch == 407`.
      - Requires the real producer row fields (`top8`, integer `top1_correct`/`top8_correct`,
        numeric `admitted_cells` in `1..=8`, `address_rivals` in `0..=8`, `token_hinge`,
        `address_hinge`, `total_loss_numerator == 8*token_hinge + 32*address_hinge`, `token_rivals`).
      - Computes the exact GPU target-cell admission indicator:
          `target_admitted = admitted_cells - address_rivals` (`0` or `1`, `top1_correct <= top8_correct <= target_admitted`),
        which sums to `1,920` across the 14,287 DEV rows.
      - Joins ordered DEV targets and verifies `top8` membership and hit indicators on every row.
    """
    if not isinstance(eval_doc, dict):
        raise W2VerificationError("sealed evaluation root must be a JSON object")
    if eval_doc.get("schema") == COMPLETION_EVAL_SCHEMA:
        inner = eval_doc.get("evaluation")
        if not isinstance(inner, dict):
            raise W2VerificationError("completion evaluation wrapper missing inner evaluation object")
        eval_obj = inner
    elif (
        isinstance(eval_doc.get("baseline_evaluation"), dict)
        and eval_doc["baseline_evaluation"].get("schema") == DRIVER_SCHEMA
    ):
        eval_obj = eval_doc["baseline_evaluation"]
    else:
        eval_obj = eval_doc

    if eval_obj.get("schema") != DRIVER_SCHEMA:
        raise W2VerificationError(f"sealed evaluation schema {eval_obj.get('schema')!r} != {DRIVER_SCHEMA!r}")
    if eval_obj.get("evaluation_mutates_model") is not False:
        raise W2VerificationError("sealed evaluation_mutates_model must be false")
    if require_strict_int(eval_obj.get("updates_applied_by_evaluation"), "updates_applied_by_evaluation", 0, 0) != 0:
        raise W2VerificationError("sealed updates_applied_by_evaluation must be 0")
    if strict_k8_q4_pins:
        if require_strict_int(eval_obj.get("model_epoch"), "model_epoch") != Q4_EPOCH:
            raise W2VerificationError(f"sealed evaluation model_epoch != {Q4_EPOCH}")
        if "source_sha256" in eval_obj and eval_obj["source_sha256"] != PINNED_ORIGINAL_SOURCE_SHA256:
            raise W2VerificationError("sealed evaluation source_sha256 != PINNED_ORIGINAL_SOURCE_SHA256")
        if "kernel_sha256" in eval_obj and eval_obj["kernel_sha256"] != PINNED_ORIGINAL_KERNEL_SHA256:
            raise W2VerificationError("sealed evaluation kernel_sha256 != PINNED_ORIGINAL_KERNEL_SHA256")
        if (
            "operational_driver_sha256" in eval_obj
            and eval_obj["operational_driver_sha256"] != PINNED_ORIGINAL_DRIVER_SHA256
        ):
            raise W2VerificationError("sealed evaluation operational_driver_sha256 != PINNED_ORIGINAL_DRIVER_SHA256")
        if "model_sha256" in eval_obj and eval_obj["model_sha256"] != PINNED_ORIGINAL_MODEL_DOMAIN_A_SHA256:
            raise W2VerificationError("sealed evaluation model_sha256 != PINNED_ORIGINAL_MODEL_DOMAIN_A_SHA256")
        if "rows" in eval_obj and require_strict_int(eval_obj["rows"], "rows") != DEV_ROWS:
            raise W2VerificationError("sealed evaluation top-level rows != DEV_ROWS")
        if (
            "admitted_rows" in eval_obj
            and require_strict_int(eval_obj["admitted_rows"], "admitted_rows") != Q4_WRITER_ADMITTED
        ):
            raise W2VerificationError("sealed evaluation top-level admitted_rows != Q4_WRITER_ADMITTED")
        if (
            "top1_correct_rows" in eval_obj
            and require_strict_int(eval_obj["top1_correct_rows"], "top1_correct_rows") != Q4_WRITER_TOP1
        ):
            raise W2VerificationError("sealed evaluation top-level top1_correct_rows != Q4_WRITER_TOP1")
        if (
            "top8_correct_rows" in eval_obj
            and require_strict_int(eval_obj["top8_correct_rows"], "top8_correct_rows") != Q4_WRITER_TOP8
        ):
            raise W2VerificationError("sealed evaluation top-level top8_correct_rows != Q4_WRITER_TOP8")

    batches = eval_obj.get("batches")
    if not isinstance(batches, list):
        raise W2VerificationError("sealed evaluation missing `batches` array (batches[*].rows)")
    expected_batches = (len(dev_corpus_rows) + DEV_BATCH_ROWS - 1) // DEV_BATCH_ROWS
    if len(batches) != expected_batches:
        raise W2VerificationError(
            f"sealed evaluation batch count {len(batches)} != expected {expected_batches}"
        )

    out: List[Dict[str, Any]] = []
    top1_sum = 0
    top8_sum = 0
    target_admitted_sum = 0

    for b_idx, batch in enumerate(batches):
        if not isinstance(batch, dict):
            raise W2VerificationError(f"sealed evaluation batch[{b_idx}] is not a JSON object")
        b_rows = batch.get("rows")
        if not isinstance(b_rows, list):
            raise W2VerificationError(f"sealed evaluation batch[{b_idx}] missing `rows` array")
        expected_b_len = (
            DEV_BATCH_ROWS if (b_idx + 1) < expected_batches else (len(dev_corpus_rows) - b_idx * DEV_BATCH_ROWS)
        )
        if len(b_rows) != expected_b_len:
            raise W2VerificationError(
                f"sealed evaluation batch[{b_idx}] row count {len(b_rows)} != expected {expected_b_len}"
            )

        for r_val in b_rows:
            ordinal = len(out)
            if not isinstance(r_val, dict):
                raise W2VerificationError(f"sealed row[{ordinal}] is not a JSON object")
            c_row = dev_corpus_rows[ordinal]
            top8_arr = r_val.get("top8")
            if not isinstance(top8_arr, list) or len(top8_arr) != 8:
                raise W2VerificationError(f"sealed row[{ordinal}].top8 must be an 8-element token array")
            top8 = [
                require_strict_int(v, f"sealed row[{ordinal}].top8[{k}]", 0, VOCABULARY - 1)
                for k, v in enumerate(top8_arr)
            ]
            if len(set(top8)) != 8:
                raise W2VerificationError(f"sealed row[{ordinal}].top8 contains duplicate tokens: {top8}")

            h1 = require_strict_int(r_val.get("top1_correct"), f"sealed row[{ordinal}].top1_correct", 0, 1)
            h8 = require_strict_int(r_val.get("top8_correct"), f"sealed row[{ordinal}].top8_correct", 0, 1)
            cells = require_strict_int(r_val.get("admitted_cells"), f"sealed row[{ordinal}].admitted_cells", 1, 8)
            ar = require_strict_int(r_val.get("address_rivals"), f"sealed row[{ordinal}].address_rivals", 0, 8)
            if cells < ar:
                raise W2VerificationError(
                    f"sealed row[{ordinal}] admitted_cells ({cells}) < address_rivals ({ar})"
                )
            target_admitted = cells - ar
            if target_admitted not in (0, 1):
                raise W2VerificationError(
                    f"sealed row[{ordinal}] target_admitted = admitted_cells - address_rivals = {target_admitted} not in {{0, 1}}"
                )
            if not (h1 <= h8 <= target_admitted):
                raise W2VerificationError(
                    f"sealed row[{ordinal}] rank/admission nesting violated: h1={h1}, h8={h8}, target_admitted={target_admitted}"
                )

            t_hinge = require_strict_int(r_val.get("token_hinge"), f"sealed row[{ordinal}].token_hinge", 0)
            a_hinge = require_strict_int(r_val.get("address_hinge"), f"sealed row[{ordinal}].address_hinge", 0)
            total_loss = require_strict_int(
                r_val.get("total_loss_numerator"), f"sealed row[{ordinal}].total_loss_numerator", 0
            )
            t_rivals = require_strict_int(
                r_val.get("token_rivals"), f"sealed row[{ordinal}].token_rivals", 0, VOCABULARY - 1
            )
            if total_loss != 8 * t_hinge + 32 * a_hinge:
                raise W2VerificationError(
                    f"sealed row[{ordinal}] total_loss_numerator {total_loss} != 8*{t_hinge} + 32*{a_hinge}"
                )

            target_tok = c_row["target_token"]
            if (h1 == 1) != (top8[0] == target_tok):
                raise W2VerificationError(
                    f"sealed row[{ordinal}] top1_correct ({h1}) != (top8[0] {top8[0]} == target_token {target_tok})"
                )
            if (h8 == 1) != (target_tok in top8):
                raise W2VerificationError(
                    f"sealed row[{ordinal}] top8_correct ({h8}) != (target_token {target_tok} in top8)"
                )
            if "target_position" in r_val:
                if require_strict_int(r_val["target_position"], f"sealed row[{ordinal}].target_position") != c_row["target_position"]:
                    raise W2VerificationError(f"sealed row[{ordinal}].target_position mismatch")
            if "target_token" in r_val:
                if require_strict_int(r_val["target_token"], f"sealed row[{ordinal}].target_token") != target_tok:
                    raise W2VerificationError(f"sealed row[{ordinal}].target_token mismatch")

            top1_sum += h1
            top8_sum += h8
            target_admitted_sum += target_admitted
            out.append(
                {
                    "row_ordinal": ordinal,
                    "top8": top8,
                    "top1_correct": h1,
                    "top8_correct": h8,
                    "admitted_cells": cells,
                    "address_rivals": ar,
                    "target_admitted": target_admitted,
                    "token_hinge": t_hinge,
                    "address_hinge": a_hinge,
                    "total_loss_numerator": total_loss,
                    "token_rivals": t_rivals,
                }
            )

    if len(out) != len(dev_corpus_rows):
        raise W2VerificationError(f"flattened sealed evaluation rows {len(out)} != DEV rows {len(dev_corpus_rows)}")
    if strict_k8_q4_pins:
        if top1_sum != Q4_WRITER_TOP1:
            raise W2VerificationError(f"sealed evaluation top1_sum {top1_sum} != {Q4_WRITER_TOP1}")
        if top8_sum != Q4_WRITER_TOP8:
            raise W2VerificationError(f"sealed evaluation top8_sum {top8_sum} != {Q4_WRITER_TOP8}")
        if target_admitted_sum != Q4_WRITER_ADMITTED:
            raise W2VerificationError(
                f"sealed evaluation target_admitted_sum (sum(admitted_cells - address_rivals)) {target_admitted_sum} != {Q4_WRITER_ADMITTED}"
            )
    return out


def verify_per_row_records(
    rows: Sequence[Dict[str, Any]],
    *,
    expected_dev_rows: int = DEV_ROWS,
    expected_admitted_k: int = 8,
    dev_corpus_rows: Optional[Sequence[Dict[str, int]]] = None,
    sealed_eval_rows: Optional[Sequence[Dict[str, Any]]] = None,
    token_cells: Optional[Sequence[int]] = None,
) -> Dict[str, Any]:
    """Verifies §4.3 per-row schema, exact nibble order (r0 low nibble, r3 high nibble),
    16-slot `admitted_cell_slots` (without invented `admitted_tokens`), exact target-cell admission
    indicator `target_admitted = admitted_cells - address_rivals` (6255411191 item 3),
    real producer writer fields (`top8`, `top1_correct`, `top8_correct`), paired discordance counts,
    and optional canonical DEV corpus, sealed Q4 evaluation, and 4,096-entry `token_cells` joins.
    """
    if len(rows) != expected_dev_rows:
        raise W2VerificationError(f"per-row count {len(rows)} != expected {expected_dev_rows}")
    if dev_corpus_rows is not None and len(dev_corpus_rows) != expected_dev_rows:
        raise W2VerificationError(
            f"dev_corpus_rows count {len(dev_corpus_rows)} != expected {expected_dev_rows}"
        )
    if sealed_eval_rows is not None and len(sealed_eval_rows) != expected_dev_rows:
        raise W2VerificationError(
            f"sealed_eval_rows count {len(sealed_eval_rows)} != expected {expected_dev_rows}"
        )
    if token_cells is not None:
        if len(token_cells) != VOCABULARY:
            raise W2VerificationError(f"token_cells len {len(token_cells)} != {VOCABULARY}")
        for tok_id, cell_v in enumerate(token_cells):
            require_strict_int(cell_v, f"token_cells[{tok_id}]", 0, 255)

    writer_hit1_sum = 0
    writer_hit8_sum = 0
    target_admitted_sum = 0
    admitted_cell_count_sum = 0
    address_rival_count_sum = 0
    table_hit1_sum = 0
    table_hit8_sum = 0
    unigram_hit1_sum = 0
    bigram_hit1_sum = 0
    seen_code_sum = 0

    b_U = 0
    c_U = 0
    b_G = 0
    c_G = 0
    b_W = 0
    c_W = 0

    for idx, row in enumerate(rows):
        if not isinstance(row, dict):
            raise W2VerificationError(f"row[{idx}] is not a JSON object")
        ord_val = require_strict_int(row.get("row_ordinal"), f"row[{idx}].row_ordinal", 0, expected_dev_rows - 1)
        if ord_val != idx:
            raise W2VerificationError(f"row[{idx}].row_ordinal {ord_val} != {idx}")

        doc_id = require_strict_int(row.get("document_id"), f"row[{idx}].document_id", 0)
        target_pos = require_strict_int(row.get("target_position"), f"row[{idx}].target_position", 1)
        query_occ = require_strict_int(
            row.get("query_occurrence", target_pos - 1), f"row[{idx}].query_occurrence", 0
        )
        if query_occ != target_pos - 1:
            raise W2VerificationError(
                f"row[{idx}].query_occurrence {query_occ} != target_position - 1 ({target_pos - 1})"
            )
        q_tok = require_strict_int(row.get("query_token"), f"row[{idx}].query_token", 0, VOCABULARY - 1)
        t_tok = require_strict_int(row.get("target_token"), f"row[{idx}].target_token", 1, VOCABULARY - 1)

        if dev_corpus_rows is not None:
            c_row = dev_corpus_rows[idx]
            if doc_id != c_row["document_id"]:
                raise W2VerificationError(
                    f"row[{idx}].document_id {doc_id} != DEV corpus document_id {c_row['document_id']}"
                )
            if "record" in row and require_strict_int(row["record"], f"row[{idx}].record", 0) != c_row["record"]:
                raise W2VerificationError(
                    f"row[{idx}].record {row['record']} != DEV corpus record {c_row['record']}"
                )
            if target_pos != c_row["target_position"]:
                raise W2VerificationError(
                    f"row[{idx}].target_position {target_pos} != DEV corpus target_position {c_row['target_position']}"
                )
            if q_tok != c_row["query_token"]:
                raise W2VerificationError(
                    f"row[{idx}].query_token {q_tok} != DEV corpus query_token {c_row['query_token']}"
                )
            if t_tok != c_row["target_token"]:
                raise W2VerificationError(
                    f"row[{idx}].target_token {t_tok} != DEV corpus target_token {c_row['target_token']}"
                )

        # Exact nibble order (6255245665 item 2): code = r0 | (r1 << 4) | (r2 << 8) | (r3 << 12)
        # root_first MUST equal low nibble r0 = code & 0xF; root_last MUST equal high nibble r3 = (code >> 12) & 0xF
        code = require_strict_int(row.get("code"), f"row[{idx}].code", 0, MAX_CODE_COUNT - 1)
        rf = require_strict_int(row.get("root_first"), f"row[{idx}].root_first", 0, 15)
        rl = require_strict_int(row.get("root_last"), f"row[{idx}].root_last", 0, 15)
        if rf != (code & 0xF):
            raise W2VerificationError(
                f"row[{idx}] root_first {rf} != low nibble r0 ({code & 0xF}) of code {code:#06x}"
            )
        if rl != ((code >> 12) & 0xF):
            raise W2VerificationError(
                f"row[{idx}] root_last {rl} != high nibble r3 ({(code >> 12) & 0xF}) of code {code:#06x}"
            )

        # 16 physical cell slots (`admitted_cell_slots`, with 0xFFFF sentinel; no invented `admitted_tokens` array!)
        slots_arr = row.get("admitted_cell_slots")
        if not isinstance(slots_arr, list) or len(slots_arr) != 16:
            raise W2VerificationError(f"row[{idx}].admitted_cell_slots must be a 16-element u16 array")

        non_sentinel_cells: List[int] = []
        for slot_i in range(16):
            c_int = require_strict_int(
                slots_arr[slot_i], f"row[{idx}].admitted_cell_slots[{slot_i}]", 0, SENTINEL_U16
            )
            if c_int != SENTINEL_U16:
                if c_int > 255:
                    raise W2VerificationError(
                        f"row[{idx}].admitted_cell_slots[{slot_i}] = {c_int} outside valid cell range 0..255 or sentinel 0xFFFF"
                    )
                non_sentinel_cells.append(c_int)

        admitted_cells_cnt = require_strict_int(
            row.get("admitted_cells"), f"row[{idx}].admitted_cells", 1, expected_admitted_k
        )
        if admitted_cells_cnt != len(non_sentinel_cells):
            raise W2VerificationError(
                f"row[{idx}].admitted_cells {admitted_cells_cnt} != non-0xFFFF slot count {len(non_sentinel_cells)}"
            )
        address_rivals = require_strict_int(
            row.get("address_rivals"), f"row[{idx}].address_rivals", 0, expected_admitted_k
        )
        if admitted_cells_cnt < address_rivals:
            raise W2VerificationError(
                f"row[{idx}] admitted_cells ({admitted_cells_cnt}) < address_rivals ({address_rivals})"
            )
        # Exact target-cell admission indicator (6255411191 item 3): admitted_cells - address_rivals in {0, 1}
        target_admitted = admitted_cells_cnt - address_rivals
        if target_admitted not in (0, 1):
            raise W2VerificationError(
                f"row[{idx}] target_admitted = admitted_cells - address_rivals = {target_admitted} not in {{0, 1}}"
            )
        if "target_cell_admitted" in row:
            raw_tca = row["target_cell_admitted"]
            decl_ta = 1 if (raw_tca is True or raw_tca == 1) else (0 if (raw_tca is False or raw_tca == 0) else -1)
            if decl_ta != target_admitted:
                raise W2VerificationError(
                    f"row[{idx}].target_cell_admitted {raw_tca!r} != admitted_cells - address_rivals ({target_admitted})"
                )
        if token_cells is not None:
            target_cell = token_cells[t_tok]
            cell_present = target_cell in non_sentinel_cells
            if cell_present != (target_admitted == 1):
                raise W2VerificationError(
                    f"row[{idx}] token_cells[{t_tok}]={target_cell} membership in admitted_cell_slots ({cell_present}) != target_admitted ({target_admitted})"
                )

        # Writer top-8 and hits (using canonical producer names `top8`, `top1_correct`, `top8_correct`)
        w_top8_arr = row.get("top8")
        if not isinstance(w_top8_arr, list) or len(w_top8_arr) != 8:
            raise W2VerificationError(f"row[{idx}].top8 must be an 8-element token array")
        w_top8 = [
            require_strict_int(v, f"row[{idx}].top8[{k}]", 0, VOCABULARY - 1)
            for k, v in enumerate(w_top8_arr)
        ]
        if len(set(w_top8)) != 8:
            raise W2VerificationError(f"row[{idx}].top8 contains duplicate tokens: {w_top8}")
        w_top1 = w_top8[0]
        w_hit1_int = require_strict_int(row.get("top1_correct"), f"row[{idx}].top1_correct", 0, 1)
        w_hit8_int = require_strict_int(row.get("top8_correct"), f"row[{idx}].top8_correct", 0, 1)
        w_hit1 = w_hit1_int == 1
        w_hit8 = w_hit8_int == 1
        if w_hit1 != (w_top1 == t_tok):
            raise W2VerificationError(f"row[{idx}].top1_correct disagrees with top8[0] == target_token")
        if w_hit8 != (t_tok in w_top8):
            raise W2VerificationError(f"row[{idx}].top8_correct disagrees with target_token in top8")
        if not (w_hit1_int <= w_hit8_int <= target_admitted):
            raise W2VerificationError(f"row[{idx}] rank/admission nesting violated")

        if sealed_eval_rows is not None:
            s_row = sealed_eval_rows[idx]
            if w_top8 != s_row["top8"]:
                raise W2VerificationError(f"row[{idx}].top8 != sealed evaluation top8")
            if w_hit1_int != s_row["top1_correct"]:
                raise W2VerificationError(f"row[{idx}].top1_correct != sealed evaluation top1_correct")
            if w_hit8_int != s_row["top8_correct"]:
                raise W2VerificationError(f"row[{idx}].top8_correct != sealed evaluation top8_correct")
            if admitted_cells_cnt != s_row["admitted_cells"]:
                raise W2VerificationError(f"row[{idx}].admitted_cells != sealed evaluation admitted_cells")
            if address_rivals != s_row["address_rivals"]:
                raise W2VerificationError(f"row[{idx}].address_rivals != sealed evaluation address_rivals")
            if target_admitted != s_row["target_admitted"]:
                raise W2VerificationError(f"row[{idx}] target_admitted != sealed evaluation target_admitted")

        # Table & ordinary comparator predictions
        t_top1 = require_strict_int(row.get("table_top1"), f"row[{idx}].table_top1", 0, VOCABULARY - 1)
        t_top8_arr = row.get("table_top8")
        if not isinstance(t_top8_arr, list) or len(t_top8_arr) != 8:
            raise W2VerificationError(f"row[{idx}].table_top8 must be an 8-element token array")
        t_top8 = [
            require_strict_int(v, f"row[{idx}].table_top8[{k}]", 0, VOCABULARY - 1)
            for k, v in enumerate(t_top8_arr)
        ]
        if len(set(t_top8)) != 8:
            raise W2VerificationError(f"row[{idx}].table_top8 contains duplicate tokens: {t_top8}")
        if t_top8[0] != t_top1:
            raise W2VerificationError(f"row[{idx}].table_top8[0] ({t_top8[0]}) != table_top1 ({t_top1})")

        u_top1 = require_strict_int(
            row.get("unigram_1049856_top1"), f"row[{idx}].unigram_1049856_top1", 0, VOCABULARY - 1
        )
        g_top1 = require_strict_int(row.get("bigram_top1"), f"row[{idx}].bigram_top1", 0, VOCABULARY - 1)

        t_hit1 = row.get("table_hit1")
        t_hit8 = row.get("table_hit8")
        u_hit1 = row.get("unigram_1049856_hit1")
        g_hit1 = row.get("bigram_hit1")
        seen_code = row.get("seen_code")
        for bname, bval in (
            ("table_hit1", t_hit1),
            ("table_hit8", t_hit8),
            ("unigram_1049856_hit1", u_hit1),
            ("bigram_hit1", g_hit1),
            ("seen_code", seen_code),
        ):
            if not isinstance(bval, bool):
                raise W2VerificationError(f"row[{idx}].{bname} must be boolean, got {bval!r}")

        if t_hit1 != (t_top1 == t_tok):
            raise W2VerificationError(f"row[{idx}].table_hit1 {t_hit1} != (table_top1 == target_token)")
        if t_hit8 != (t_tok in t_top8):
            raise W2VerificationError(f"row[{idx}].table_hit8 {t_hit8} != (target_token in table_top8)")
        if u_hit1 != (u_top1 == t_tok):
            raise W2VerificationError(
                f"row[{idx}].unigram_1049856_hit1 {u_hit1} != (unigram_1049856_top1 == target_token)"
            )
        if g_hit1 != (g_top1 == t_tok):
            raise W2VerificationError(f"row[{idx}].bigram_hit1 {g_hit1} != (bigram_top1 == target_token)")

        if w_hit1:
            writer_hit1_sum += 1
        if w_hit8:
            writer_hit8_sum += 1
        target_admitted_sum += target_admitted
        admitted_cell_count_sum += admitted_cells_cnt
        address_rival_count_sum += address_rivals
        if t_hit1:
            table_hit1_sum += 1
        if t_hit8:
            table_hit8_sum += 1
        if u_hit1:
            unigram_hit1_sum += 1
        if g_hit1:
            bigram_hit1_sum += 1
        if seen_code:
            seen_code_sum += 1

        if (not t_hit1) and u_hit1:
            b_U += 1
        elif t_hit1 and (not u_hit1):
            c_U += 1

        if (not t_hit1) and g_hit1:
            b_G += 1
        elif t_hit1 and (not g_hit1):
            c_G += 1

        if (not t_hit1) and w_hit1:
            b_W += 1
        elif t_hit1 and (not w_hit1):
            c_W += 1

    return {
        "rows_verified": len(rows),
        "dev_corpus_joined": dev_corpus_rows is not None,
        "sealed_eval_joined": sealed_eval_rows is not None,
        "token_cells_joined": token_cells is not None,
        "writer_hit1_sum": writer_hit1_sum,
        "writer_hit8_sum": writer_hit8_sum,
        "target_admitted_sum": target_admitted_sum,
        "admitted_cell_count_sum": admitted_cell_count_sum,
        "address_rival_count_sum": address_rival_count_sum,
        "table_hit1_sum": table_hit1_sum,
        "table_hit8_sum": table_hit8_sum,
        "unigram_hit1_sum": unigram_hit1_sum,
        "bigram_hit1_sum": bigram_hit1_sum,
        "seen_code_sum": seen_code_sum,
        "paired_discordance": {
            "U": {"b": b_U, "c": c_U},
            "G": {"b": b_G, "c": c_G},
            "W": {"b": b_W, "c": c_W},
        },
    }


def evaluate_w2_decision_table(
    record: Dict[str, Any],
    *,
    strict_k8_q4_pins: bool = True,
    require_lossless_evidence: bool = True,
    dev_corpus_rows: Optional[Sequence[Dict[str, int]]] = None,
    sealed_eval_rows: Optional[Sequence[Dict[str, Any]]] = None,
    compact_train_summary: Optional[Dict[str, Any]] = None,
    runtime_manifest: Optional[Dict[str, str]] = None,
    token_cells: Optional[Sequence[int]] = None,
) -> Dict[str, Any]:
    """Evaluates the §4.2 / §4.6 / §4.7 / §4.8 precedence table (Precedence 0..5) on a W2 record.
    - When `require_lossless_evidence=True` (default verified runtime mode), requires:
      * Bound original->target source/kernel/binary/runtime-manifest bridge, verified runtime manifest
        file on disk, and producer file pins;
      * Lossless 14,287-row DEV observation array joined to canonical DEV PreparedCorpus (`UORT`)
        and flattened 224-batch sealed Q4 evaluation (`4fff1d3c...`), with exact equality between
        row-derived totals/target_admitted_sum (`1,920`)/paired discordances and declared `aggregates`/`evaluability`;
      * Verified compact TRAIN stream summary joined to `producer_file_pins` and `codes_with_count_ge_2`.
    - When `require_lossless_evidence=False` (`--allow-summary-preview` or synthetic unit tests),
      labels the output `verification_tier = "SUMMARY_PREVIEW_ONLY"` and `runtime_verdict_authorized = False`.
    """
    inst = verify_instrument_validity(
        record,
        strict_k8_q4_pins=strict_k8_q4_pins,
        require_bound_runtime_pins=require_lossless_evidence,
        runtime_manifest=runtime_manifest,
    )
    if inst["instrument_fault"]:
        return {
            "verification_tier": (
                "VERIFIED_RUNTIME_EVIDENCE" if require_lossless_evidence else "SUMMARY_PREVIEW_ONLY"
            ),
            "runtime_verdict_authorized": False,
            "precedence": 0,
            "verdict": "INSTRUMENT_FAULT",
            "d31_disposition": "DIAGNOSE_AND_REPAIR_NEVER_MECHANISM_NEGATIVE",
            "fault_reasons": inst["fault_reasons"],
        }

    eval_blk = record.get("evaluability")
    if not isinstance(eval_blk, dict):
        raise W2VerificationError("missing evaluability object (§4.6)")
    agg = record.get("aggregates")
    if not isinstance(agg, dict):
        raise W2VerificationError("missing aggregates object (§4.2, §4.3)")

    dev_rows = require_strict_int(eval_blk.get("dev_rows", DEV_ROWS), "evaluability.dev_rows", min_value=1)
    if strict_k8_q4_pins and dev_rows != DEV_ROWS:
        raise W2VerificationError(f"evaluability.dev_rows {dev_rows} != {DEV_ROWS}")

    coverage_rows = require_strict_int(
        eval_blk.get("coverage_rows"),
        "evaluability.coverage_rows",
        min_value=0,
        max_value=dev_rows,
    )
    codes_ge_2 = require_strict_int(
        eval_blk.get("codes_with_count_ge_2"),
        "evaluability.codes_with_count_ge_2",
        min_value=0,
        max_value=MAX_CODE_COUNT,
    )

    T = require_strict_int(agg.get("T_table_matched_top1"), "aggregates.T_table_matched_top1", 0, dev_rows)
    U = require_strict_int(agg.get("U_unigram_matched_top1"), "aggregates.U_unigram_matched_top1", 0, dev_rows)
    G = require_strict_int(agg.get("G_bigram_matched_top1"), "aggregates.G_bigram_matched_top1", 0, dev_rows)
    W = require_strict_int(agg.get("W_writer_top1"), "aggregates.W_writer_top1", 0, dev_rows)
    if strict_k8_q4_pins and W != Q4_WRITER_TOP1:
        return {
            "verification_tier": (
                "VERIFIED_RUNTIME_EVIDENCE" if require_lossless_evidence else "SUMMARY_PREVIEW_ONLY"
            ),
            "runtime_verdict_authorized": False,
            "precedence": 0,
            "verdict": "INSTRUMENT_FAULT",
            "d31_disposition": "DIAGNOSE_AND_REPAIR_NEVER_MECHANISM_NEGATIVE",
            "fault_reasons": [f"aggregates.W_writer_top1 {W} != sealed Q4 writer top-1 {Q4_WRITER_TOP1}"],
        }

    disc = agg.get("paired_discordance")
    if not isinstance(disc, dict):
        raise W2VerificationError("missing aggregates.paired_discordance")

    def extract_bc(name: str, comparator_top1: int) -> Tuple[int, int]:
        entry = disc.get(name)
        if not isinstance(entry, dict):
            raise W2VerificationError(f"missing paired_discordance.{name}")
        b = require_strict_int(entry.get("b"), f"paired_discordance.{name}.b", 0, dev_rows)
        c = require_strict_int(entry.get("c"), f"paired_discordance.{name}.c", 0, dev_rows)
        if b + c > dev_rows:
            raise W2VerificationError(f"paired_discordance.{name} b+c={b+c} > dev_rows={dev_rows}")
        if c - b != T - comparator_top1:
            raise W2VerificationError(
                f"paired_discordance.{name} conservation failure: c - b ({c} - {b} = {c - b}) != T - {name} ({T} - {comparator_top1} = {T - comparator_top1})"
            )
        return b, c

    try:
        b_U, c_U = extract_bc("U", U)
        b_G, c_G = extract_bc("G", G)
        b_W, c_W = extract_bc("W", W)
    except W2VerificationError as exc:
        return {
            "verification_tier": (
                "VERIFIED_RUNTIME_EVIDENCE" if require_lossless_evidence else "SUMMARY_PREVIEW_ONLY"
            ),
            "runtime_verdict_authorized": False,
            "precedence": 0,
            "verdict": "INSTRUMENT_FAULT",
            "d31_disposition": "DIAGNOSE_AND_REPAIR_NEVER_MECHANISM_NEGATIVE",
            "fault_reasons": [str(exc)],
        }

    # Lossless evidence binding (6255245665 item 1 & 6255411191 items 1, 2, 3)
    row_verification: Optional[Dict[str, Any]] = None
    rows_obj = record.get("rows")
    if require_lossless_evidence:
        if not isinstance(rows_obj, list):
            raise W2VerificationError(
                "verified runtime mode requires lossless 14,287-row `rows` array (use --allow-summary-preview for summary-only preview)"
            )
        if dev_corpus_rows is None:
            raise W2VerificationError(
                "verified runtime mode requires canonical DEV PreparedCorpus join (dev_corpus_rows)"
            )
        if strict_k8_q4_pins and sealed_eval_rows is None:
            raise W2VerificationError(
                "verified runtime mode requires sealed Q4 evaluation-16404.json row join (sealed_eval_rows)"
            )
        if compact_train_summary is None:
            raise W2VerificationError(
                "verified runtime mode requires verified compact TRAIN stream join (--compact-train-stream)"
            )
        if runtime_manifest is None:
            raise W2VerificationError(
                "verified runtime mode requires verified runtime manifest file (--runtime-manifest)"
            )

    if isinstance(rows_obj, list):
        row_verification = verify_per_row_records(
            rows_obj,
            expected_dev_rows=dev_rows,
            dev_corpus_rows=dev_corpus_rows,
            sealed_eval_rows=sealed_eval_rows,
            token_cells=token_cells,
        )
        # Join row-derived totals and paired discordance to declared aggregates/evaluability
        if row_verification["table_hit1_sum"] != T:
            raise W2VerificationError(
                f"row-derived table_hit1_sum ({row_verification['table_hit1_sum']}) != aggregates.T_table_matched_top1 ({T})"
            )
        if "T_table_matched_top8" in agg:
            t_top8_agg = require_strict_int(agg["T_table_matched_top8"], "aggregates.T_table_matched_top8", 0, dev_rows)
            if row_verification["table_hit8_sum"] != t_top8_agg:
                raise W2VerificationError(
                    f"row-derived table_hit8_sum ({row_verification['table_hit8_sum']}) != aggregates.T_table_matched_top8 ({t_top8_agg})"
                )
        if row_verification["unigram_hit1_sum"] != U:
            raise W2VerificationError(
                f"row-derived unigram_hit1_sum ({row_verification['unigram_hit1_sum']}) != aggregates.U_unigram_matched_top1 ({U})"
            )
        if row_verification["bigram_hit1_sum"] != G:
            raise W2VerificationError(
                f"row-derived bigram_hit1_sum ({row_verification['bigram_hit1_sum']}) != aggregates.G_bigram_matched_top1 ({G})"
            )
        if row_verification["writer_hit1_sum"] != W:
            raise W2VerificationError(
                f"row-derived writer_hit1_sum ({row_verification['writer_hit1_sum']}) != aggregates.W_writer_top1 ({W})"
            )
        if strict_k8_q4_pins and row_verification["writer_hit8_sum"] != Q4_WRITER_TOP8:
            raise W2VerificationError(
                f"row-derived writer_hit8_sum ({row_verification['writer_hit8_sum']}) != sealed Q4 writer top-8 ({Q4_WRITER_TOP8})"
            )
        if strict_k8_q4_pins and row_verification["target_admitted_sum"] != Q4_WRITER_ADMITTED:
            raise W2VerificationError(
                f"row-derived target_admitted_sum ({row_verification['target_admitted_sum']}) != sealed Q4 target-admitted rows ({Q4_WRITER_ADMITTED})"
            )
        if row_verification["seen_code_sum"] != coverage_rows:
            raise W2VerificationError(
                f"row-derived seen_code_sum ({row_verification['seen_code_sum']}) != evaluability.coverage_rows ({coverage_rows})"
            )
        row_disc = row_verification["paired_discordance"]
        if row_disc["U"] != {"b": b_U, "c": c_U}:
            raise W2VerificationError(
                f"row-derived paired_discordance.U {row_disc['U']} != declared {{'b': {b_U}, 'c': {c_U}}}"
            )
        if row_disc["G"] != {"b": b_G, "c": c_G}:
            raise W2VerificationError(
                f"row-derived paired_discordance.G {row_disc['G']} != declared {{'b': {b_G}, 'c': {c_G}}}"
            )
        if row_disc["W"] != {"b": b_W, "c": c_W}:
            raise W2VerificationError(
                f"row-derived paired_discordance.W {row_disc['W']} != declared {{'b': {b_W}, 'c': {c_W}}}"
            )

    if compact_train_summary is not None:
        if compact_train_summary.get("codes_with_count_ge_2") != codes_ge_2:
            raise W2VerificationError(
                f"compact TRAIN stream codes_with_count_ge_2 ({compact_train_summary.get('codes_with_count_ge_2')}) != evaluability.codes_with_count_ge_2 ({codes_ge_2})"
            )
        pins = record.get("producer_file_pins")
        if isinstance(pins, dict):
            if compact_train_summary.get("stream_sha256") != pins.get("compact_train_stream_sha256"):
                raise W2VerificationError(
                    f"compact TRAIN stream SHA-256 ({compact_train_summary.get('stream_sha256')}) != producer_file_pins.compact_train_stream_sha256 ({pins.get('compact_train_stream_sha256')})"
                )
            if compact_train_summary.get("stream_bytes") != pins.get("compact_train_stream_bytes"):
                raise W2VerificationError(
                    f"compact TRAIN stream bytes ({compact_train_summary.get('stream_bytes')}) != producer_file_pins.compact_train_stream_bytes ({pins.get('compact_train_stream_bytes')})"
                )

    # Evaluability checks (§4.6, Precedence 1)
    underpowered_reasons: List[str] = []
    # Exact 90.00% check: coverage_rows * 100 >= 90 * dev_rows (for 14287, coverage_rows >= 12859)
    if coverage_rows * 100 < 90 * dev_rows:
        underpowered_reasons.append(
            f"coverage {coverage_rows}/{dev_rows} ({coverage_rows / dev_rows:.6%}) < 90.00% (min {MIN_COVERAGE_ROWS})"
        )
    if codes_ge_2 < MIN_MULTIPLICITY_CODES:
        underpowered_reasons.append(
            f"codes_with_count_ge_2 ({codes_ge_2}) < {MIN_MULTIPLICITY_CODES}"
        )

    for comp_name, (b_val, c_val) in (("U", (b_U, c_U)), ("G", (b_G, c_G))):
        bc_sum = b_val + c_val
        if bc_sum < MIN_DISCORDANCE_NORMAL:
            pval = exact_two_sided_mcnemar_pvalue(b_val, c_val)
            if pval > 0.05:
                underpowered_reasons.append(
                    f"co-primary {comp_name} has b+c={bc_sum} < 100 and exact two-sided McNemar p={pval:.6g} > 0.05"
                )

    F_U = paired_floor(b_U, c_U, FULL_PANEL_MIN_FLOOR)
    F_G = paired_floor(b_G, c_G, FULL_PANEL_MIN_FLOOR)
    F = max(F_U, F_G)
    M = max(U, G)
    H = paired_floor(b_W, c_W, FULL_PANEL_MIN_FLOOR)

    verification_tier = (
        "VERIFIED_RUNTIME_EVIDENCE" if require_lossless_evidence else "SUMMARY_PREVIEW_ONLY"
    )
    if underpowered_reasons:
        return {
            "verification_tier": verification_tier,
            "runtime_verdict_authorized": require_lossless_evidence,
            "precedence": 1,
            "verdict": "UNDERPOWERED" if require_lossless_evidence else "PREVIEW_UNDERPOWERED",
            "candidate_verdict": "UNDERPOWERED",
            "underpowered_reasons": underpowered_reasons,
            "T": T,
            "U": U,
            "G": G,
            "M": M,
            "W": W,
            "F_U": F_U,
            "F_G": F_G,
            "F": F,
            "H": H,
            "coverage_rows": coverage_rows,
            "codes_with_count_ge_2": codes_ge_2,
        }

    # Check pathological-ordering guard (§4.2, §4.7): M + F < W - H
    pathological_ordering = (M + F) < (W - H)
    t_ge_w_minus_h = ge_minus_paired_floor(T - W, b_W + c_W, FULL_PANEL_MIN_FLOOR)
    t_lt_w_minus_h = not t_ge_w_minus_h

    max_bc_ug = max(b_U + c_U, b_G + c_G)
    t_gt_m_plus_f = exceeds_paired_floor_strict(T - M, max_bc_ug, FULL_PANEL_MIN_FLOOR)
    t_ge_w_plus_h = ge_paired_floor(T - W, b_W + c_W, FULL_PANEL_MIN_FLOOR)

    if pathological_ordering and t_lt_w_minus_h:
        precedence = 4
        candidate_verdict = "H_rep"
        note = "pathological-ordering guard fired (M + F < W - H and T < W - H -> H_rep per §4.2/§4.7)"
    elif t_gt_m_plus_f:
        precedence = 2
        candidate_verdict = "H_law"
        note = "T > M + F: exact-code table beats both co-primary ordinary references by the paired floor"
    elif t_ge_w_plus_h:
        precedence = 3
        candidate_verdict = "H_mixed"
        note = "W + H <= T <= M + F: exact-code table beats trained writer by paired floor but not both ordinary references"
    elif t_ge_w_minus_h:
        precedence = 4
        candidate_verdict = "H_rep"
        note = "W - H <= T < W + H and T <= M + F: exact-code table not distinguishable from trained writer"
    else:
        precedence = 5
        candidate_verdict = "WRITER_DOMINATES"
        note = "T < W - H and T <= M + F: trained geometric map extracts more than per-code counts"

    declared_verdict = record.get("verdict")
    if declared_verdict is not None and declared_verdict != candidate_verdict:
        raise W2VerificationError(
            f"declared record verdict {declared_verdict!r} != recomputed decision-table verdict {candidate_verdict!r}"
        )

    out: Dict[str, Any] = {
        "verification_tier": verification_tier,
        "runtime_verdict_authorized": require_lossless_evidence,
        "precedence": precedence,
        "verdict": candidate_verdict if require_lossless_evidence else f"PREVIEW_{candidate_verdict}",
        "candidate_verdict": candidate_verdict,
        "note": note,
        "pathological_ordering": pathological_ordering,
        "T": T,
        "U": U,
        "G": G,
        "M": M,
        "W": W,
        "F_U": F_U,
        "F_G": F_G,
        "F": F,
        "H": H,
        "M_plus_F": M + F,
        "W_plus_H": W + H,
        "W_minus_H": W - H,
        "coverage_rows": coverage_rows,
        "coverage_fraction": coverage_rows / dev_rows,
        "codes_with_count_ge_2": codes_ge_2,
        "wilson_95": {
            "T": wilson_95_interval(T, dev_rows),
            "U": wilson_95_interval(U, dev_rows),
            "G": wilson_95_interval(G, dev_rows),
            "W": wilson_95_interval(W, dev_rows),
        },
        "capture_mode": inst["capture_mode"],
    }
    if row_verification is not None:
        out["per_row_verification"] = row_verification
    if compact_train_summary is not None:
        out["compact_train_verification"] = compact_train_summary
    return out


def verify_compact_train_stream_bytes(
    data: bytes,
    *,
    capture_mode: str = "q_only_2425925",
    expected_rows: Optional[int] = None,
    allow_synthetic_length: bool = False,
    require_query_handoff: bool = True,
    query_sidecar_bytes: Optional[bytes] = None,
    corpus_tokens_bytes: Optional[bytes] = None,
    corpus_mask_bytes: Optional[bytes] = None,
) -> Dict[str, Any]:
    """Verifies a compact TRAIN (code: u16 LE, target: u16 LE) stream per W2 §4.10 / interface-prep §4 / addendum:
    - Fails closed on missing or arbitrary row counts: requires exact primary (`1,049,856` rows = `4,199,424` B)
      or full (`2,425,925` rows = `9,703,700` B) stream length matching `capture_mode`, unless
      `allow_synthetic_length=True` is explicitly set in synthetic unit tests.
    - Enforces nibble order 0..3: c(q) = r_0 | (r_1 << 4) | (r_2 << 8) | (r_3 << 12), each r_b in 0..15.
    - Every target token t must be in 1..4095 (never BOS 0 and never sentinel 0xFFFF).
    - Requires a query-token handoff source when `require_query_handoff=True`:
      * Mode A (preferred zero-sidecar PreparedCorpus join): parses `corpus_tokens_bytes` and `corpus_mask_bytes`
        using `parse_prepared_corpus_uort` (validating the 64-byte `UORT` header, slicing `bytes[64..]`,
        requiring `mask.len() == count` with binary `{0, 1}` values, pinned `BOS = ("<|bos|>", 0)`,
        `p > 0 && tokens[p] != bos_id && mask[p] == 1`, and in-document predecessor `position > doc_start`),
        and verifies `target == corpus_row["target_token"]` at every row.
      * Mode B (bound u16 query-token sidecar): verifies `2 * total_rows` bytes of little-endian u16 query tokens
        in `0..4095`.
    - Exact count conservation: sum_{c, t} N[c, t] == total_rows.
    """
    if len(data) == 0 or len(data) % 4 != 0:
        raise W2VerificationError(
            f"compact TRAIN stream byte length {len(data)} must be positive and a multiple of 4"
        )
    total_rows = len(data) // 4

    if not allow_synthetic_length:
        if capture_mode == "q_only_2425925":
            mode_expected = FULL_TRAIN_TARGETS
        elif capture_mode in ("production_forward_matched_1049856_fallback", "matched_prefix_1049856"):
            mode_expected = MATCHED_TRAIN_TARGETS
        else:
            raise W2VerificationError(f"unrecognized capture_mode {capture_mode!r} for compact TRAIN stream")
        if expected_rows is not None and expected_rows != mode_expected:
            raise W2VerificationError(
                f"expected_rows {expected_rows} conflicts with capture_mode {capture_mode!r} ({mode_expected})"
            )
        expected_rows = mode_expected

    if expected_rows is None or total_rows != expected_rows:
        raise W2VerificationError(
            f"compact TRAIN stream rows {total_rows} != expected {expected_rows} (bytes={len(data)})"
        )

    if sys.byteorder != "little":
        raise W2VerificationError("host byteorder must be little-endian for u16 memoryview cast")
    mv = memoryview(data).cast("H")

    sidecar_mv = None
    if query_sidecar_bytes is not None:
        if len(query_sidecar_bytes) != 2 * total_rows:
            raise W2VerificationError(
                f"query sidecar byte length {len(query_sidecar_bytes)} != 2 * {total_rows}"
            )
        sidecar_mv = memoryview(query_sidecar_bytes).cast("H")

    corpus_rows: Optional[List[Dict[str, int]]] = None
    corpus_meta: Optional[Dict[str, Any]] = None
    if corpus_tokens_bytes is not None or corpus_mask_bytes is not None:
        if corpus_tokens_bytes is None or corpus_mask_bytes is None:
            raise W2VerificationError("both corpus_tokens_bytes and corpus_mask_bytes are required for corpus join")
        corpus_meta = parse_prepared_corpus_uort(
            corpus_tokens_bytes,
            corpus_mask_bytes,
            bos_id=PINNED_BOS_ID,
            bos_token=PINNED_BOS_TOKEN,
            split_tag=SPLIT_TAG_TRAIN,
            context=CONTEXT_WINDOW,
            expected_payload_tokens=None if allow_synthetic_length else TRAIN_PAYLOAD_TOKENS,
            expected_documents=None if allow_synthetic_length else TRAIN_DOCUMENTS,
            expected_eligible_targets=None if allow_synthetic_length else FULL_TRAIN_TARGETS,
            limit_rows=total_rows,
        )
        corpus_rows = corpus_meta["rows"]

    if require_query_handoff and corpus_rows is None and sidecar_mv is None:
        raise W2VerificationError(
            "compact TRAIN stream verification requires either canonical PreparedCorpus join (Mode A) or query sidecar (Mode B)"
        )

    code_counts: Dict[int, int] = {}
    matched_prefix_code_counts: Dict[int, int] = {}
    matched_limit = min(total_rows, MATCHED_TRAIN_TARGETS)
    bigram_total = 0
    total_counted = 0

    for idx in range(total_rows):
        code = int(mv[2 * idx])
        target = int(mv[2 * idx + 1])
        if target == PINNED_BOS_ID or target == SENTINEL_U16 or target >= VOCABULARY:
            raise W2VerificationError(
                f"compact TRAIN stream row {idx}: invalid, BOS, or sentinel target token {target:#06x}"
            )
        r0 = code & 0xF
        r1 = (code >> 4) & 0xF
        r2 = (code >> 8) & 0xF
        r3 = (code >> 12) & 0xF
        repacked = r0 | (r1 << 4) | (r2 << 8) | (r3 << 12)
        if repacked != code:
            raise W2VerificationError(f"compact TRAIN stream row {idx}: code {code:#06x} nibble mismatch")

        if corpus_rows is not None:
            c_row = corpus_rows[idx]
            if target != c_row["target_token"]:
                raise W2VerificationError(
                    f"compact TRAIN stream row {idx} (pos={c_row['target_position']}): target {target} != corpus target {c_row['target_token']}"
                )
            q_tok = c_row["query_token"]
            if q_tok == SENTINEL_U16 or q_tok >= VOCABULARY:
                raise W2VerificationError(
                    f"compact TRAIN stream row {idx}: invalid corpus query token {q_tok:#06x}"
                )
            if sidecar_mv is not None and int(sidecar_mv[idx]) != q_tok:
                raise W2VerificationError(
                    f"compact TRAIN stream row {idx}: sidecar query {int(sidecar_mv[idx])} != corpus query {q_tok}"
                )
            if idx < matched_limit:
                bigram_total += 1
        elif sidecar_mv is not None:
            q_tok = int(sidecar_mv[idx])
            if q_tok == SENTINEL_U16 or q_tok >= VOCABULARY:
                raise W2VerificationError(
                    f"compact TRAIN stream row {idx}: invalid or sentinel sidecar query token {q_tok:#06x}"
                )
            if idx < matched_limit:
                bigram_total += 1

        code_counts[code] = code_counts.get(code, 0) + 1
        if idx < matched_limit:
            matched_prefix_code_counts[code] = matched_prefix_code_counts.get(code, 0) + 1
        total_counted += 1

    if total_counted != total_rows or sum(code_counts.values()) != total_rows:
        raise W2VerificationError("compact TRAIN stream count conservation failure")

    # Multiplicity bar (§4.6) is evaluated on the primary 1,049,856-target matched table
    codes_ge_2_matched = sum(1 for cnt in matched_prefix_code_counts.values() if cnt >= 2)
    codes_ge_2_total = sum(1 for cnt in code_counts.values() if cnt >= 2)
    out: Dict[str, Any] = {
        "capture_mode": capture_mode,
        "stream_bytes": len(data),
        "stream_sha256": sha256_bytes(data),
        "rows_verified": total_rows,
        "matched_prefix_rows": matched_limit,
        "distinct_codes_seen": len(matched_prefix_code_counts),
        "distinct_codes_seen_total": len(code_counts),
        "codes_with_count_ge_2": codes_ge_2_matched,
        "codes_with_count_ge_2_total": codes_ge_2_total,
    }
    if sidecar_mv is not None:
        out["query_sidecar_bytes"] = len(query_sidecar_bytes)  # type: ignore[arg-type]
        out["query_sidecar_sha256"] = sha256_bytes(query_sidecar_bytes)  # type: ignore[arg-type]
        out["bigram_rows_verified"] = bigram_total
    if corpus_meta is not None:
        out["corpus_join_verified"] = True
        out["uort_header_verified"] = True
        out["corpus_payload_tokens"] = corpus_meta["payload_tokens"]
        out["corpus_documents"] = corpus_meta["document_count"]
        out["corpus_eligible_targets"] = corpus_meta["eligible_target_count"]
        out["bigram_rows_verified"] = bigram_total
    return out


def verify_host_pinned_inputs() -> Dict[str, Any]:
    """Fails closed if ANY required host-pinned W2 artifact or corpus file is missing or mismatched.
    Validates the 64-byte UORT headers of DEV and TRAIN tokens.u16 and reports future compiled/runtime
    pins explicitly as 'UNBOUND' until runtime binding is supplied.
    """
    required_files: List[Tuple[str, pathlib.Path, str, Optional[int]]] = [
        (
            "sheet_sha256",
            pathlib.Path("/Users/casey.allard/uor-r4/docs/harness/prep/m1-read-code-capacity-2026-10-17.md"),
            PINNED_SHEET_SHA256,
            None,
        ),
        (
            "readiness_map_sha256",
            pathlib.Path("/Users/casey.allard/uor-r4-local/w2-bvg-source-readiness-20261018/REPORT.md"),
            PINNED_READINESS_MAP_SHA256,
            None,
        ),
        (
            "interface_prep_report_sha256",
            pathlib.Path("/Users/casey.allard/uor-r4-local/w2-input-interface-preparation-20261018/REPORT.md"),
            PINNED_INTERFACE_PREP_REPORT_SHA256,
            None,
        ),
        (
            "interface_prep_manifest_sha256",
            pathlib.Path("/Users/casey.allard/uor-r4-local/w2-input-interface-preparation-20261018/SHA256SUMS"),
            PINNED_INTERFACE_PREP_MANIFEST_SHA256,
            None,
        ),
        (
            "interface_review_sha256",
            pathlib.Path("/Users/casey.allard/uor-r4-local/w2-input-interface-preparation-review-20261018/REPORT.md"),
            PINNED_INTERFACE_REVIEW_SHA256,
            None,
        ),
        (
            "retained_input_receipt_sha256",
            pathlib.Path("/Users/casey.allard/uor-r4-local/w2-retained-input-authentication-20261018/receipt.json"),
            PINNED_RETAINED_INPUT_RECEIPT_SHA256,
            None,
        ),
        (
            "dev_tokens_sha256",
            pathlib.Path(
                "/Users/casey.allard/uor-r4-local/codex-cf1-k16-rows-fastpath/qualification-prep/operator/data/ft/dev/tokens.u16"
            ),
            PINNED_DEV_TOKENS_SHA256,
            DEV_TOKENS_BYTES,
        ),
        (
            "dev_mask_sha256",
            pathlib.Path(
                "/Users/casey.allard/uor-r4-local/codex-cf1-k16-rows-fastpath/qualification-prep/operator/data/ft/dev/response_mask.u8"
            ),
            PINNED_DEV_MASK_SHA256,
            DEV_MASK_BYTES,
        ),
        (
            "train_tokens_sha256",
            pathlib.Path(
                "/Users/casey.allard/uor-r4-local/codex-cf1-k16-rows-fastpath/qualification-prep/operator/data/mw-balp/train/tokens.u16"
            ),
            PINNED_TRAIN_TOKENS_SHA256,
            TRAIN_TOKENS_BYTES,
        ),
        (
            "train_mask_sha256",
            pathlib.Path(
                "/Users/casey.allard/uor-r4-local/codex-cf1-k16-rows-fastpath/qualification-prep/operator/data/mw-balp/train/response_mask.u8"
            ),
            PINNED_TRAIN_MASK_SHA256,
            TRAIN_MASK_BYTES,
        ),
        (
            "d34_record_sha256",
            pathlib.Path(
                "/Users/casey.allard/uor-r4-local/codex-attention-preflight-inputs-v2/d34-successor-table-record.json"
            ),
            PINNED_D34_RECORD_SHA256,
            None,
        ),
    ]

    checked: Dict[str, Any] = {}
    for key, path, expected_sha, expected_len in required_files:
        if not path.is_file():
            raise W2VerificationError(f"missing required host-pinned input for {key}: {path}")
        raw = path.read_bytes()
        if expected_len is not None and len(raw) != expected_len:
            raise W2VerificationError(
                f"{key} byte length {len(raw)} != expected {expected_len} ({path})"
            )
        sha = sha256_bytes(raw)
        if sha != expected_sha:
            raise W2VerificationError(f"{key} SHA-256 {sha} != expected {expected_sha} ({path})")
        checked[key] = sha
        if key == "dev_tokens_sha256":
            dev_hdr = parse_uort_header(raw[:UORT_HEADER_BYTES], expected_payload_tokens=DEV_PAYLOAD_TOKENS)
            checked["dev_uort_header"] = dev_hdr
        elif key == "train_tokens_sha256":
            train_hdr = parse_uort_header(raw[:UORT_HEADER_BYTES], expected_payload_tokens=TRAIN_PAYLOAD_TOKENS)
            checked["train_uort_header"] = train_hdr
        elif key == "d34_record_sha256":
            d34_obj = json.loads(raw.decode("utf-8"))
            checked["d34_matched_train_targets"] = d34_obj.get("matched_train_targets", MATCHED_TRAIN_TARGETS)

    checked["future_runtime_pins"] = {
        "original_source_sha256": PINNED_ORIGINAL_SOURCE_SHA256,
        "original_kernel_sha256": PINNED_ORIGINAL_KERNEL_SHA256,
        "original_operational_driver_sha256": PINNED_ORIGINAL_DRIVER_SHA256,
        "original_model_domain_a_sha256": PINNED_ORIGINAL_MODEL_DOMAIN_A_SHA256,
        "original_snapshot_domain_b_sha256": PINNED_ORIGINAL_SNAPSHOT_DOMAIN_B_SHA256,
        "target_compiled_source_sha256": "UNBOUND",
        "target_kernel_sha256": "UNBOUND",
        "target_binary_sha256": "UNBOUND",
        "runtime_manifest_sha256": "UNBOUND",
    }
    return checked


def build_uort_bytes(tokens: Sequence[int], vocabulary: int = VOCABULARY) -> bytes:
    """Builds a canonical 64-byte UORT header + little-endian u16 token payload for self-tests."""
    header = bytearray(UORT_HEADER_BYTES)
    header[0:4] = UORT_MAGIC
    header[4:8] = int(UORT_VERSION).to_bytes(4, "little")
    header[8:16] = int(len(tokens)).to_bytes(8, "little")
    header[16:20] = int(vocabulary).to_bytes(4, "little")
    payload = b"".join(int(t).to_bytes(2, "little") for t in tokens)
    return bytes(header) + payload


def make_synthetic_w2_record(
    *,
    T: int = 155,
    U: int = 1226,
    G: int = 5047,
    W: int = 149,
    b_U: Optional[int] = None,
    c_U: Optional[int] = None,
    b_G: Optional[int] = None,
    c_G: Optional[int] = None,
    b_W: Optional[int] = None,
    c_W: Optional[int] = None,
    coverage_rows: int = 13500,
    codes_with_count_ge_2: int = 2048,
    capture_mode: str = "q_only_2425925",
    smaller_table_bias_disclosed: bool = True,
) -> Dict[str, Any]:
    """Builds a valid synthetic W2 summary record satisfying c_X - b_X == T - X for preview/decision-table self-tests."""
    if b_U is None or c_U is None:
        diff_u = T - U
        b_U = max(100, 100 - min(0, diff_u))
        c_U = b_U + diff_u
    if b_G is None or c_G is None:
        diff_g = T - G
        b_G = max(100, 100 - min(0, diff_g))
        c_G = b_G + diff_g
    if b_W is None or c_W is None:
        diff_w = T - W
        b_W = max(50, 50 - min(0, diff_w))
        c_W = b_W + diff_w

    return {
        "schema": READ_CODE_CAPACITY_SCHEMA,
        "zero_apply": True,
        "applied_updates": 0,
        "updates_applied": 0,
        "snapshot_unchanged": True,
        "model_compute_executed": True,
        "epoch": Q4_EPOCH,
        "source_sha256": PINNED_ORIGINAL_SOURCE_SHA256,
        "checkpoint_sha256": PINNED_Q4_CHECKPOINT_SHA256,
        "panel_binding_sha256": PINNED_ORDERED_ROW_BINDING_SHA256,
        "original_to_target_bridge": {
            "original_source_sha256": PINNED_ORIGINAL_SOURCE_SHA256,
            "original_kernel_sha256": PINNED_ORIGINAL_KERNEL_SHA256,
            "target_compiled_source_sha256": "UNBOUND",
            "target_kernel_sha256": "UNBOUND",
            "target_binary_sha256": "UNBOUND",
            "runtime_manifest_sha256": "UNBOUND",
        },
        "instrument_validity": {
            "root_round_trip": {
                "sample_rows": 1024,
                "mismatches": 0,
                "absent_roots": 0,
            },
            "train_capture": {
                "mode": capture_mode,
                "sample_rows": 1024,
                "production_forward_disagreements": 0,
                "smaller_table_bias_disclosed": smaller_table_bias_disclosed,
            },
            "sealed_receipt_agreement": {
                "evaluation_sha256": PINNED_Q4_EVALUATION_SHA256,
                "dev_rows": DEV_ROWS,
                "row_disagreements": 0,
                "writer_top1": W,
                "writer_top8": Q4_WRITER_TOP8,
                "writer_admitted": Q4_WRITER_ADMITTED,
            },
            "count_table_conservation": {
                "matched_train_total": MATCHED_TRAIN_TARGETS,
                "full_train_total": FULL_TRAIN_TARGETS if capture_mode == "q_only_2425925" else None,
                "bigram_train_total": MATCHED_TRAIN_TARGETS,
                "invalid_tokens": 0,
                "negative_counts": 0,
            },
            "determinism": {
                "repeat_rows": 1000,
                "byte_identical": True,
            },
            "symmetry_action_non_roots": 0,
            "cuda_or_status_fault": False,
            "d23_d25_d32_ratchets_pass": True,
        },
        "evaluability": {
            "dev_rows": DEV_ROWS,
            "coverage_rows": coverage_rows,
            "codes_with_count_ge_2": codes_with_count_ge_2,
        },
        "aggregates": {
            "T_table_matched_top1": T,
            "U_unigram_matched_top1": U,
            "G_bigram_matched_top1": G,
            "W_writer_top1": W,
            "paired_discordance": {
                "U": {"b": b_U, "c": c_U},
                "G": {"b": b_G, "c": c_G},
                "W": {"b": b_W, "c": c_W},
            },
        },
    }


def run_self_test() -> Dict[str, Any]:
    """Executes deterministic self-tests across Precedence 0..5, canonical UORT 64-byte header + BOS
    eligibility, exact nibble order, target_admitted = admitted_cells - address_rivals vs nonempty
    admission set, K16 qualification graph 035b3c... rejection, and fail-closed runtime guards.
    """
    # 0. Summary-only records MUST be rejected in verified runtime mode (require_lossless_evidence=True):
    # (a) Unbound runtime pins -> Precedence 0 (INSTRUMENT_FAULT, runtime_verdict_authorized=False)
    rec_hrep = make_synthetic_w2_record(T=155, U=1226, G=5047, W=149, b_W=50, c_W=56)
    res_runtime_unbound = evaluate_w2_decision_table(rec_hrep, require_lossless_evidence=True)
    assert (
        res_runtime_unbound["precedence"] == 0
        and res_runtime_unbound["verdict"] == "INSTRUMENT_FAULT"
        and res_runtime_unbound["runtime_verdict_authorized"] is False
    ), res_runtime_unbound
    # (b) Bound pins with verified runtime_manifest but without lossless rows/corpus/stream joins -> raises W2VerificationError
    synth_manifest_bytes = json.dumps(
        {
            "original_source_sha256": PINNED_ORIGINAL_SOURCE_SHA256,
            "original_kernel_sha256": PINNED_ORIGINAL_KERNEL_SHA256,
            "original_operational_driver_sha256": PINNED_ORIGINAL_DRIVER_SHA256,
            "original_model_domain_a_sha256": PINNED_ORIGINAL_MODEL_DOMAIN_A_SHA256,
            "target_compiled_source_sha256": PINNED_ORIGINAL_SOURCE_SHA256,
            "target_kernel_sha256": PINNED_ORIGINAL_KERNEL_SHA256,
            "target_binary_sha256": "1" * 64,
        }
    ).encode("utf-8")
    synth_manifest = verify_runtime_manifest_bytes(synth_manifest_bytes)
    rec_bound_no_rows = make_synthetic_w2_record(T=155, U=1226, G=5047, W=149, b_W=50, c_W=56)
    rec_bound_no_rows["original_to_target_bridge"] = {
        "original_source_sha256": PINNED_ORIGINAL_SOURCE_SHA256,
        "original_kernel_sha256": PINNED_ORIGINAL_KERNEL_SHA256,
        "target_compiled_source_sha256": PINNED_ORIGINAL_SOURCE_SHA256,
        "target_kernel_sha256": PINNED_ORIGINAL_KERNEL_SHA256,
        "target_binary_sha256": "1" * 64,
        "runtime_manifest_sha256": synth_manifest["manifest_sha256"],
    }
    rec_bound_no_rows["producer_file_pins"] = {
        "dev_tokens_sha256": PINNED_DEV_TOKENS_SHA256,
        "dev_mask_sha256": PINNED_DEV_MASK_SHA256,
        "train_tokens_sha256": PINNED_TRAIN_TOKENS_SHA256,
        "train_mask_sha256": PINNED_TRAIN_MASK_SHA256,
        "tokenizer_sha256": PINNED_TOKENIZER_SHA256,
        "bos_token": PINNED_BOS_TOKEN,
        "bos_id": PINNED_BOS_ID,
        "compact_train_stream_sha256": "3" * 64,
        "compact_train_stream_bytes": COMPACT_TRAIN_FULL_BYTES,
    }
    try:
        evaluate_w2_decision_table(
            rec_bound_no_rows, require_lossless_evidence=True, runtime_manifest=synth_manifest
        )
        raise AssertionError("expected W2VerificationError when lossless rows/corpus/stream joins are omitted")
    except W2VerificationError:
        pass

    # (c) K16 qualification graph 035b3c... substitution MUST be rejected!
    bad_manifest_bytes = json.dumps(
        {
            "original_source_sha256": PINNED_ORIGINAL_SOURCE_SHA256,
            "original_kernel_sha256": PINNED_ORIGINAL_KERNEL_SHA256,
            "original_operational_driver_sha256": PINNED_ORIGINAL_DRIVER_SHA256,
            "original_model_domain_a_sha256": PINNED_ORIGINAL_MODEL_DOMAIN_A_SHA256,
            "target_compiled_source_sha256": FORBIDDEN_K16_QUAL_SOURCE_SHA256,
            "target_kernel_sha256": PINNED_ORIGINAL_KERNEL_SHA256,
            "target_binary_sha256": "1" * 64,
        }
    ).encode("utf-8")
    try:
        verify_runtime_manifest_bytes(bad_manifest_bytes)
        raise AssertionError("expected W2VerificationError when K16 qualification graph 035b3c... is substituted")
    except W2VerificationError:
        pass

    # 1. Precedence 4 (Summary Preview): H_rep (W - H <= T < W + H and T <= M + F)
    res_hrep = evaluate_w2_decision_table(rec_hrep, require_lossless_evidence=False)
    assert (
        res_hrep["precedence"] == 4
        and res_hrep["candidate_verdict"] == "H_rep"
        and res_hrep["verification_tier"] == "SUMMARY_PREVIEW_ONLY"
        and res_hrep["runtime_verdict_authorized"] is False
    ), res_hrep

    # Boundary T == W - H -> still H_rep (H = max(24, 2*sqrt(100)) = 24; W - H = 125)
    rec_bound_low = make_synthetic_w2_record(T=125, U=1226, G=5047, W=149, b_W=62, c_W=38)
    res_bound_low = evaluate_w2_decision_table(rec_bound_low, require_lossless_evidence=False)
    assert res_bound_low["precedence"] == 4 and res_bound_low["candidate_verdict"] == "H_rep", res_bound_low

    # 2. Precedence 5: WRITER_DOMINATES (T < W - H and T <= M + F)
    rec_wdom = make_synthetic_w2_record(T=124, U=1226, G=5047, W=149, b_W=62, c_W=37)
    res_wdom = evaluate_w2_decision_table(rec_wdom, require_lossless_evidence=False)
    assert res_wdom["precedence"] == 5 and res_wdom["candidate_verdict"] == "WRITER_DOMINATES", res_wdom

    # 3. Precedence 3: H_mixed (W + H <= T <= M + F)
    rec_hmix = make_synthetic_w2_record(T=173, U=1226, G=5047, W=149, b_W=38, c_W=62)
    res_hmix = evaluate_w2_decision_table(rec_hmix, require_lossless_evidence=False)
    assert res_hmix["precedence"] == 3 and res_hmix["candidate_verdict"] == "H_mixed", res_hmix

    # Boundary T == M + F -> still H_mixed (strict inequality required for H_law per §4.2/§4.7)
    rec_bound_high2 = make_synthetic_w2_record(
        T=5087, U=5047, G=5047, W=149, b_U=180, c_U=220, b_G=180, c_G=220
    )
    res_bound_high2 = evaluate_w2_decision_table(rec_bound_high2, require_lossless_evidence=False)
    assert res_bound_high2["precedence"] == 3 and res_bound_high2["candidate_verdict"] == "H_mixed", res_bound_high2

    # 4. Precedence 2: H_law (T > M + F)
    rec_hlaw = make_synthetic_w2_record(
        T=5200, U=1226, G=5047, W=149, b_U=20, c_U=3994, b_G=20, c_G=173
    )
    res_hlaw = evaluate_w2_decision_table(rec_hlaw, require_lossless_evidence=False)
    assert res_hlaw["precedence"] == 2 and res_hlaw["candidate_verdict"] == "H_law", res_hlaw

    # 5. Precedence 1: UNDERPOWERED
    rec_under_cov = make_synthetic_w2_record(coverage_rows=12858)
    res_under_cov = evaluate_w2_decision_table(rec_under_cov, require_lossless_evidence=False)
    assert res_under_cov["precedence"] == 1 and res_under_cov["candidate_verdict"] == "UNDERPOWERED", res_under_cov

    rec_under_mult = make_synthetic_w2_record(codes_with_count_ge_2=1023)
    res_under_mult = evaluate_w2_decision_table(rec_under_mult, require_lossless_evidence=False)
    assert res_under_mult["precedence"] == 1 and res_under_mult["candidate_verdict"] == "UNDERPOWERED", res_under_mult

    # 6. Precedence 0: INSTRUMENT_FAULT
    rec_fault = make_synthetic_w2_record()
    rec_fault["instrument_validity"]["root_round_trip"]["absent_roots"] = 1
    res_fault = evaluate_w2_decision_table(rec_fault, require_lossless_evidence=False)
    assert res_fault["precedence"] == 0 and res_fault["verdict"] == "INSTRUMENT_FAULT", res_fault

    # 7. Canonical 64-byte UORT + binary mask + BOS eligibility self-test (6255231630)
    uort_tokens = build_uort_bytes([0, 10, 42, 10, 42, 0, 99, 4095])
    uort_mask = bytes([0, 0, 1, 0, 1, 1, 0, 1])  # pos 5 has tok=0 (BOS) and mask=1 -> skipped by tok != bos_id!
    sample_stream = (
        (0x00FF).to_bytes(2, "little") + (42).to_bytes(2, "little")
        + (0x00FF).to_bytes(2, "little") + (42).to_bytes(2, "little")
        + (0x4321).to_bytes(2, "little") + (4095).to_bytes(2, "little")
    )
    sample_sidecar = (
        (10).to_bytes(2, "little") + (10).to_bytes(2, "little") + (99).to_bytes(2, "little")
    )
    stream_res = verify_compact_train_stream_bytes(
        sample_stream,
        expected_rows=3,
        allow_synthetic_length=True,
        query_sidecar_bytes=sample_sidecar,
        corpus_tokens_bytes=uort_tokens,
        corpus_mask_bytes=uort_mask,
    )
    assert (
        stream_res["rows_verified"] == 3
        and stream_res["codes_with_count_ge_2"] == 1
        and stream_res.get("corpus_join_verified") is True
        and stream_res.get("uort_header_verified") is True
        and stream_res.get("corpus_documents") == 2
        and stream_res.get("corpus_eligible_targets") == 3
        and stream_res.get("bigram_rows_verified") == 3
    ), stream_res

    # Raw u16 without 64-byte UORT header MUST be rejected (6255231630)
    raw_no_header = b"".join(t.to_bytes(2, "little") for t in (0, 10, 42, 10, 42, 0, 99, 4095))
    try:
        verify_compact_train_stream_bytes(
            sample_stream,
            expected_rows=3,
            allow_synthetic_length=True,
            corpus_tokens_bytes=raw_no_header,
            corpus_mask_bytes=uort_mask,
        )
        raise AssertionError("expected W2VerificationError on raw u16 tokens without 64-byte UORT header")
    except W2VerificationError:
        pass

    # 8. Exact nibble order & target_admitted = admitted_cells - address_rivals self-test (6255411191 item 3)
    valid_row = {
        "row_ordinal": 0,
        "document_id": 0,
        "record": (SPLIT_TAG_DEV << 62) | 0,
        "target_position": 2,
        "query_occurrence": 1,
        "query_token": 10,
        "target_token": 42,
        "code": 0x4321,
        "root_first": 1,
        "root_last": 4,
        "admitted_cells": 8,
        "address_rivals": 8,
        "admitted_cell_slots": [10, 11, 12, 13, 14, 15, 16, 17] + [SENTINEL_U16] * 8,
        "top8": [1, 2, 3, 4, 5, 6, 7, 8],
        "top1_correct": 0,
        "top8_correct": 0,
        "table_top1": 42,
        "table_top8": [42, 1, 2, 3, 4, 5, 6, 7],
        "unigram_1049856_top1": 10,
        "bigram_top1": 42,
        "table_hit1": True,
        "table_hit8": True,
        "unigram_1049856_hit1": False,
        "bigram_hit1": True,
        "seen_code": True,
    }
    row_res = verify_per_row_records([valid_row], expected_dev_rows=1)
    assert (
        row_res["table_hit1_sum"] == 1
        and row_res["target_admitted_sum"] == 0
        and row_res["admitted_cell_count_sum"] == 8
        and row_res["paired_discordance"]["U"] == {"b": 0, "c": 1}
    ), row_res

    reversed_nibble_row = dict(valid_row, root_first=4, root_last=1)
    try:
        verify_per_row_records([reversed_nibble_row], expected_dev_rows=1)
        raise AssertionError("expected W2VerificationError on reversed root_first/root_last nibbles")
    except W2VerificationError:
        pass

    return {
        "self_test": "PASS",
        "cases_verified": 18,
    }


def main(argv: Optional[Sequence[str]] = None) -> int:
    parser = argparse.ArgumentParser(
        description="Host-only D24-clean consumer verifier for W2 Read-Code Capacity Diagnostic (uor-r4.read-code-capacity/1)"
    )
    parser.add_argument("--self-test", action="store_true", help="Run deterministic Precedence 0..5 + UORT self-tests")
    parser.add_argument(
        "--verify-host-pins",
        action="store_true",
        help="Fail-closed verification of all host-pinned W2 artifacts and UORT corpus headers",
    )
    parser.add_argument("--record", type=pathlib.Path, help="Path to a uor-r4.read-code-capacity/1 JSON record to verify")
    parser.add_argument(
        "--sealed-evaluation",
        type=pathlib.Path,
        default=None,
        help="Path to sealed Q4 evaluation-16404.json (4fff1d3c...) for 224-batch per-row join in verified runtime mode",
    )
    parser.add_argument(
        "--runtime-manifest",
        type=pathlib.Path,
        default=None,
        help="Path to authenticated runtime manifest JSON file required in verified runtime mode",
    )
    parser.add_argument(
        "--compact-train-stream",
        type=pathlib.Path,
        default=None,
        help="Path to compact TRAIN (u16 code, u16 target) little-endian stream to verify",
    )
    parser.add_argument(
        "--capture-mode",
        type=str,
        default="q_only_2425925",
        choices=("q_only_2425925", "production_forward_matched_1049856_fallback", "matched_prefix_1049856"),
        help="Registered TRAIN capture mode determining exact expected stream length",
    )
    parser.add_argument(
        "--query-sidecar",
        type=pathlib.Path,
        default=None,
        help="Optional path to a bound u16 query-token sidecar (2 * rows bytes, little-endian)",
    )
    parser.add_argument(
        "--join-train-corpus",
        action="store_true",
        help="Verify zero-sidecar PreparedCorpus join against pinned TRAIN tokens.u16 (UORT) + response_mask.u8",
    )
    parser.add_argument(
        "--allow-summary-preview",
        action="store_true",
        help="Allow summary-only preview evaluation without lossless DEV/TRAIN row joins (marks runtime_verdict_authorized=false)",
    )
    parser.add_argument("--allow-non-k8-scope", action="store_true", help="Allow non-K8-Q4 checkpoint/evaluation pins")
    args = parser.parse_args(argv)

    if args.self_test:
        out = run_self_test()
        print(json.dumps(out, indent=2, sort_keys=True))
        return 0

    if args.verify_host_pins:
        out = verify_host_pinned_inputs()
        print(json.dumps(out, indent=2, sort_keys=True))
        return 0

    compact_summary: Optional[Dict[str, Any]] = None
    if args.compact_train_stream is not None:
        if not args.compact_train_stream.is_file():
            raise W2VerificationError(f"missing compact TRAIN stream file: {args.compact_train_stream}")
        raw = args.compact_train_stream.read_bytes()
        sidecar = None
        if args.query_sidecar is not None:
            if not args.query_sidecar.is_file():
                raise W2VerificationError(f"missing query sidecar file: {args.query_sidecar}")
            sidecar = args.query_sidecar.read_bytes()
        tok_bytes = None
        mask_bytes = None
        if args.join_train_corpus or sidecar is None:
            train_tokens = pathlib.Path(
                "/Users/casey.allard/uor-r4-local/codex-cf1-k16-rows-fastpath/qualification-prep/operator/data/mw-balp/train/tokens.u16"
            )
            train_mask = pathlib.Path(
                "/Users/casey.allard/uor-r4-local/codex-cf1-k16-rows-fastpath/qualification-prep/operator/data/mw-balp/train/response_mask.u8"
            )
            if not train_tokens.is_file() or not train_mask.is_file():
                raise W2VerificationError("missing pinned TRAIN tokens.u16 or response_mask.u8 on host")
            tok_bytes = train_tokens.read_bytes()
            mask_bytes = train_mask.read_bytes()
            if sha256_bytes(tok_bytes) != PINNED_TRAIN_TOKENS_SHA256:
                raise W2VerificationError("TRAIN tokens.u16 SHA-256 mismatch")
            if sha256_bytes(mask_bytes) != PINNED_TRAIN_MASK_SHA256:
                raise W2VerificationError("TRAIN response_mask.u8 SHA-256 mismatch")
        compact_summary = verify_compact_train_stream_bytes(
            raw,
            capture_mode=args.capture_mode,
            query_sidecar_bytes=sidecar,
            corpus_tokens_bytes=tok_bytes,
            corpus_mask_bytes=mask_bytes,
        )
        if args.record is None:
            print(json.dumps(compact_summary, indent=2, sort_keys=True))
            return 0

    if args.record is not None:
        if not args.record.is_file():
            raise W2VerificationError(f"missing W2 record file: {args.record}")
        raw = args.record.read_bytes()
        rec = json.loads(raw.decode("utf-8"))
        dev_rows_parsed: Optional[List[Dict[str, int]]] = None
        sealed_rows: Optional[List[Dict[str, Any]]] = None
        rt_manifest_parsed: Optional[Dict[str, str]] = None
        if not args.allow_summary_preview:
            dev_tokens = pathlib.Path(
                "/Users/casey.allard/uor-r4-local/codex-cf1-k16-rows-fastpath/qualification-prep/operator/data/ft/dev/tokens.u16"
            )
            dev_mask = pathlib.Path(
                "/Users/casey.allard/uor-r4-local/codex-cf1-k16-rows-fastpath/qualification-prep/operator/data/ft/dev/response_mask.u8"
            )
            if not dev_tokens.is_file() or not dev_mask.is_file():
                raise W2VerificationError("missing pinned DEV tokens.u16 or response_mask.u8 on host")
            dt_bytes = dev_tokens.read_bytes()
            dm_bytes = dev_mask.read_bytes()
            if sha256_bytes(dt_bytes) != PINNED_DEV_TOKENS_SHA256:
                raise W2VerificationError("DEV tokens.u16 SHA-256 mismatch")
            if sha256_bytes(dm_bytes) != PINNED_DEV_MASK_SHA256:
                raise W2VerificationError("DEV response_mask.u8 SHA-256 mismatch")
            dev_meta = parse_prepared_corpus_uort(
                dt_bytes,
                dm_bytes,
                bos_id=PINNED_BOS_ID,
                bos_token=PINNED_BOS_TOKEN,
                split_tag=SPLIT_TAG_DEV,
                context=CONTEXT_WINDOW,
                expected_payload_tokens=DEV_PAYLOAD_TOKENS,
                expected_documents=DEV_DOCUMENTS,
                expected_eligible_targets=DEV_ROWS,
            )
            dev_rows_parsed = dev_meta["rows"]
            if args.sealed_evaluation is None:
                raise W2VerificationError(
                    "verified runtime mode requires --sealed-evaluation <evaluation-16404.json>"
                )
            if not args.sealed_evaluation.is_file():
                raise W2VerificationError(f"missing sealed evaluation file: {args.sealed_evaluation}")
            se_bytes = args.sealed_evaluation.read_bytes()
            if not args.allow_non_k8_scope:
                if len(se_bytes) != PINNED_Q4_EVALUATION_BYTES or sha256_bytes(se_bytes) != PINNED_Q4_EVALUATION_SHA256:
                    raise W2VerificationError("sealed evaluation-16404.json size/SHA-256 mismatch")
            se_obj = json.loads(se_bytes.decode("utf-8"))
            sealed_rows = flatten_sealed_evaluation_batches(
                se_obj, dev_rows_parsed, strict_k8_q4_pins=not args.allow_non_k8_scope
            )
            if args.runtime_manifest is None:
                raise W2VerificationError(
                    "verified runtime mode requires --runtime-manifest <path>"
                )
            if not args.runtime_manifest.is_file():
                raise W2VerificationError(f"missing runtime manifest file: {args.runtime_manifest}")
            rt_manifest_parsed = verify_runtime_manifest_bytes(args.runtime_manifest.read_bytes())

        out = evaluate_w2_decision_table(
            rec,
            strict_k8_q4_pins=not args.allow_non_k8_scope,
            require_lossless_evidence=not args.allow_summary_preview,
            dev_corpus_rows=dev_rows_parsed,
            sealed_eval_rows=sealed_rows,
            compact_train_summary=compact_summary,
            runtime_manifest=rt_manifest_parsed,
        )
        out["record_sha256"] = sha256_bytes(raw)
        print(json.dumps(out, indent=2, sort_keys=True))
        return 0 if out["precedence"] != 0 else 2

    parser.print_help()
    return 0


if __name__ == "__main__":
    sys.exit(main())
