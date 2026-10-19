#!/usr/bin/env python3
"""Host-only deterministic consumer verifier for W3 BVG-K Stage C2 `bvgk-qualify`.

Verifies `bvgk-qualification.json` produced by `crates/uor-r4-discrete/src/bin/bvgk-qualify.rs`
(`crates/uor-r4-discrete/src/bvgk_qualification.rs`) against the registered Stage C2
pre-registration sheet `docs/harness/prep/m1-bvgk-stagec2-block-qualification-2026-10-18.md`.

Host-only; never creates a CUDA context, constructs a model, or alters a hold.
"""

from __future__ import annotations

import argparse
import copy
import hashlib
import json
import re
import sys
from pathlib import Path
from typing import Any

EXPECTED_SCHEMA = "uor-r4.discrete-spiralcore-fbs/2"
EXPECTED_SUITE = "bvgk-block-qualification"
EXPECTED_APPLIED_UPDATES_SCOPE = (
    "fixture devices only: block commits and the single applies of the V3/JR "
    "references; no corpus, checkpoint or artifact is written"
)
EXPECTED_TP_SHAPE = (
    "V4096/B4/C192/K16 Route objective A, seed 1 checkpoint 0, 64 rows, "
    "VectorBlock 0..K, all eligible, 1 warmup + 20 timed, median and nearest-rank p95"
)
EXPECTED_TP_SCOPE = (
    "host-callable Device::propose_block (rows-K scoring, choose, scratch clone + "
    "re-carry + joint score when |S_t| >= 2, decide, readback) vs Device::propose_rows"
)

DRIVER_SOURCE_FILES = (
    "corpus.rs",
    "training.rs",
    "controller.rs",
    "completion.rs",
    "completion_baseline.rs",
    "redirect_admission.rs",
    "attention_learning.rs",
    "attention_evidence.rs",
    "attention_transfer.rs",
    "attention_preflight.rs",
    "main.rs",
)

C1_QUALIFIED_ROWS_IMAGE_SHA256 = [
    "48d8e6e05f2de03d2f9931812f606e5f923c2416dd4205696a6228d858446318",
    "c03752ab4750394b4cb6b10897884622e5cf8150cd0b7e08326e89ad4a33e49a",
    "9d0f170b86961c4cc3900c29c75c65a75a9d8e0baa41e67aea613f9db313c5df",
    "3a3c620782d2cb97b3c1e99074e467ff5faad979636766edd368c02dd9710221",
]
C1_QUALIFIED_ROWSK_IMAGE_SHA256 = [
    "0b1373a02ec191ff173f10cfd6f293cc3355ed2022cde113c6dd17f41c7dea51",
    "9b320a75ce3a92d50ca81b1a95789cb3b01b24d2a3f48ede851003e90da1f408",
    "11d3cfd57356be8769d821f26389049dbcfa545126c5dc39b9f82faea8d7c62d",
    "c2d74425413da8a23ac8cb4f9791a3e19ffcb122daa90535ee2970d670f5be10",
]
EXPECTED_BVGK_SOURCE_SHA256 = (
    "e646f0160a49d22430c0d3fbd3f2b3764d2c6d332864549b13ddbcbd292ba050"
)

HEX64_RE = re.compile(r"^[0-9a-f]{64}$")
HEX40_RE = re.compile(r"^[0-9a-f]{40}$")


class VerificationError(ValueError):
    """Raised when a C2 qualification receipt or companion artifact fails verification."""


def require(condition: bool, message: str) -> None:
    if not condition:
        raise VerificationError(message)


def is_u64(value: Any) -> bool:
    return isinstance(value, int) and not isinstance(value, bool) and value >= 0


def is_hex64(value: Any) -> bool:
    return isinstance(value, str) and bool(HEX64_RE.match(value))


RUST_TOKEN_RE = re.compile(
    r'//[^\n]*|/\*.*?\*/|r(#*)\".*?\"\1|\"(?:\\.|[^\"\\])*\"',
    re.S,
)
SINGLE_APPLY_CALL_RE = re.compile(r"(?:\bDevice\s*::\s*apply|\.\s*apply)\s*\(")
APPLY_BLOCK_CALL_RE = re.compile(r"\.\s*apply_block\s*\(")


def strip_rust_comments_and_strings(source: str) -> str:
    """Replace Rust comments and string literals with spaces so lexical checks inspect code only."""
    return RUST_TOKEN_RE.sub(" ", source)


def verify_no_guarded_single_apply(
    source: str,
    label: str,
    require_apply_block: bool = False,
) -> None:
    """Verify code (after stripping comments/strings) never calls guarded single-move Device::apply."""
    stripped = strip_rust_comments_and_strings(source)
    require(
        SINGLE_APPLY_CALL_RE.search(stripped) is None,
        f"{label} must never call guarded single-move Device::apply",
    )
    if require_apply_block:
        require(
            APPLY_BLOCK_CALL_RE.search(stripped) is not None,
            f"{label} must commit fixture updates via apply_block",
        )


def compute_host_source_hashes(repo_root: Path) -> dict[str, str]:
    """Recompute driver_source_sha256 and generated CUDA source hashes from repo files."""
    src_dir = repo_root / "crates" / "uor-r4-discrete" / "src"
    require(src_dir.is_dir(), f"missing discrete src dir: {src_dir}")

    driver_hasher = hashlib.sha256()
    for filename in DRIVER_SOURCE_FILES:
        path = src_dir / filename
        require(path.is_file(), f"missing driver source file: {path}")
        driver_hasher.update(path.read_bytes())

    # Check static source guard in bvgk_qualification.rs and bin/bvgk-qualify.rs
    qual_src = (src_dir / "bvgk_qualification.rs").read_text(encoding="utf-8")
    verify_no_guarded_single_apply(
        qual_src,
        "bvgk_qualification.rs",
        require_apply_block=True,
    )
    bin_qual_path = src_dir / "bin" / "bvgk-qualify.rs"
    require(bin_qual_path.is_file(), f"missing qualifier binary source: {bin_qual_path}")
    verify_no_guarded_single_apply(
        bin_qual_path.read_text(encoding="utf-8"),
        "bin/bvgk-qualify.rs",
        require_apply_block=False,
    )

    # Reconstruct generated CUDA sources (rows.cu, rowsk.cu, bvgk.cu) from source files
    discrete_cu = (src_dir / "discrete.cu").read_text(encoding="utf-8")
    rows_rs = (src_dir / "rows_source.rs").read_text(encoding="utf-8")
    rowsk_rs = (src_dir / "rowsk_source.rs").read_text(encoding="utf-8")
    bvgk_rs = (src_dir / "bvgk_source.rs").read_text(encoding="utf-8")

    rows_m = re.search(r'const ROWS: &str = r#"(.*?)"#;', rows_rs, re.S)
    rowsk_m = re.search(r'const ROWSK: &str = r#"(.*?)"#;', rowsk_rs, re.S)
    bvgk_m = re.search(r'const BVGK: &str = r#"(.*?)"#;', bvgk_rs, re.S)
    require(
        rows_m is not None and rowsk_m is not None and bvgk_m is not None,
        "failed to extract ROWS/ROWSK/BVGK CUDA source constants",
    )
    writer_end = discrete_cu.find("// Statuses added")
    require(writer_end > 0, "discrete.cu writer end anchor missing")
    rows_cu = f"{discrete_cu[:writer_end]}\n{rows_m.group(1)}"
    rowsk_cu = f"{rows_cu}\n{rowsk_m.group(1)}"
    bvgk_cu = f"{rowsk_cu}\n{bvgk_m.group(1)}"
    require(
        rowsk_cu.startswith(rows_cu) and bvgk_cu.startswith(rowsk_cu),
        "V0 host check failed: bvgk.cu must extend rowsk.cu and rowsk.cu must extend rows.cu",
    )

    rows_src_sha = hashlib.sha256(rows_cu.encode("utf-8")).hexdigest()
    rowsk_src_sha = hashlib.sha256(rowsk_cu.encode("utf-8")).hexdigest()
    bvgk_src_sha = hashlib.sha256(bvgk_cu.encode("utf-8")).hexdigest()
    require(
        rows_src_sha == C1_QUALIFIED_ROWS_IMAGE_SHA256[0]
        and rowsk_src_sha == C1_QUALIFIED_ROWSK_IMAGE_SHA256[0]
        and bvgk_src_sha == EXPECTED_BVGK_SOURCE_SHA256,
        "reconstructed rows/rowsk/bvgk CUDA source SHA-256 mismatch against pinned digests",
    )

    return {
        "driver_source_sha256": driver_hasher.hexdigest(),
        "rows_source_sha256": rows_src_sha,
        "rowsk_source_sha256": rowsk_src_sha,
        "bvgk_source_sha256": bvgk_src_sha,
    }


def verify_tally_fields(record: dict[str, Any], name: str) -> None:
    required_tally_keys = (
        "cases",
        "proposals",
        "multi_member_proposals",
        "empty",
        "single",
        "joint_wins",
        "joint_loses_or_ties",
        "refused_blocks",
        "commits",
        "v3_cases",
        "v3_joint_cases",
        "lanes",
        "racy_status",
        "refusals",
        "applied_updates",
    )
    for key in required_tally_keys:
        require(key in record, f"{name}: missing tally field {key!r}")
        require(is_u64(record[key]), f"{name}: tally field {key!r} must be u64")


def verify_throughput(tp: Any) -> dict[str, Any]:
    require(isinstance(tp, dict), "throughput must be a JSON object")
    require(
        tp.get("qualified") is False,
        "throughput.qualified must be false (diagnostic only, never a gate)",
    )
    require(
        tp.get("shape") == EXPECTED_TP_SHAPE,
        f"throughput.shape mismatch: {tp.get('shape')!r}",
    )
    require(
        tp.get("scope") == EXPECTED_TP_SCOPE,
        f"throughput.scope mismatch: {tp.get('scope')!r}",
    )

    single_raw = tp.get("single_propose_rows_raw_samples_ns")
    require(
        isinstance(single_raw, list)
        and len(single_raw) == 20
        and all(is_u64(x) and x > 0 for x in single_raw),
        "throughput.single_propose_rows_raw_samples_ns must be 20 positive u64 samples",
    )
    sorted_single = sorted(single_raw)
    expected_single_median = sorted_single[10]
    expected_single_p95 = sorted_single[18]
    require(
        tp.get("single_propose_rows_median_ns") == expected_single_median,
        "throughput.single_propose_rows_median_ns does not match sorted raw[10]",
    )
    require(
        tp.get("single_propose_rows_p95_ns") == expected_single_p95,
        "throughput.single_propose_rows_p95_ns does not match nearest-rank p95 sorted raw[18]",
    )

    cases = tp.get("cases")
    require(isinstance(cases, list) and len(cases) == 3, "throughput.cases must have 3 entries")
    expected_ks = [1, 4, 16]
    per_k_summary: dict[int, dict[str, Any]] = {}
    for idx, (case, expected_k) in enumerate(zip(cases, expected_ks)):
        require(isinstance(case, dict), f"throughput.cases[{idx}] must be an object")
        require(
            case.get("k") == expected_k,
            f"throughput.cases[{idx}].k expected {expected_k}, got {case.get('k')!r}",
        )
        raw = case.get("raw_samples_ns")
        require(
            isinstance(raw, list) and len(raw) == 20 and all(is_u64(x) and x > 0 for x in raw),
            f"throughput.cases[{idx}].raw_samples_ns must be 20 positive u64 samples",
        )
        sorted_raw = sorted(raw)
        median_ns = case.get("block_median_ns")
        p95_ns = case.get("block_p95_ns")
        ns_per_coord = case.get("ns_per_coordinate")
        require(
            median_ns == sorted_raw[10],
            f"throughput.cases[{idx}].block_median_ns != sorted_raw[10]",
        )
        require(
            p95_ns == sorted_raw[18],
            f"throughput.cases[{idx}].block_p95_ns != sorted_raw[18]",
        )
        require(
            ns_per_coord == median_ns // expected_k,
            f"throughput.cases[{idx}].ns_per_coordinate != block_median_ns // k",
        )
        s_len = case.get("s_len")
        require(
            is_u64(s_len) and 0 <= s_len <= expected_k,
            f"throughput.cases[{idx}].s_len must be in 0..={expected_k}",
        )
        joint_verify_ran = case.get("joint_verify_ran")
        require(
            isinstance(joint_verify_ran, bool) and joint_verify_ran == (s_len >= 2),
            f"throughput.cases[{idx}].joint_verify_ran must equal (s_len >= 2)",
        )
        joint_committed = case.get("joint_committed")
        require(
            isinstance(joint_committed, bool),
            f"throughput.cases[{idx}].joint_committed must be bool",
        )
        if s_len < 2:
            require(
                joint_committed is False,
                f"throughput.cases[{idx}].joint_committed must be false when s_len < 2",
            )
        per_k_summary[expected_k] = {
            "block_median_ns": median_ns,
            "ns_per_coordinate": ns_per_coord,
            "s_len": s_len,
            "joint_verify_ran": joint_verify_ran,
            "joint_committed": joint_committed,
        }
    return {
        "single_propose_rows_median_ns": expected_single_median,
        "cases": per_k_summary,
    }


def verify_bvgk_qualification_payload(
    payload: Any,
    expected_driver_sha256: str | None = None,
) -> dict[str, Any]:
    """Verify a parsed `bvgk-qualification.json` object against Stage C2 requirements."""
    require(isinstance(payload, dict), "root payload must be a JSON object")
    require(
        payload.get("schema") == EXPECTED_SCHEMA,
        f"schema mismatch: expected {EXPECTED_SCHEMA!r}, got {payload.get('schema')!r}",
    )
    require(
        payload.get("suite") == EXPECTED_SUITE,
        f"suite mismatch: expected {EXPECTED_SUITE!r}, got {payload.get('suite')!r}",
    )
    require(
        payload.get("status") == "PASS",
        f"status must be 'PASS', got {payload.get('status')!r}",
    )
    require(
        payload.get("training") == "HELD",
        f"training must remain 'HELD', got {payload.get('training')!r}",
    )
    require(
        payload.get("applied_updates_scope") == EXPECTED_APPLIED_UPDATES_SCOPE,
        f"applied_updates_scope mismatch: {payload.get('applied_updates_scope')!r}",
    )

    driver_sha = payload.get("driver_source_sha256")
    require(is_hex64(driver_sha), "driver_source_sha256 must be 64 lowercase hex chars")
    if expected_driver_sha256 is not None:
        require(
            driver_sha == expected_driver_sha256,
            f"driver_source_sha256 {driver_sha} != expected {expected_driver_sha256}",
        )

    for field, expected_prefix_list in (
        ("rows_image_sha256", C1_QUALIFIED_ROWS_IMAGE_SHA256),
        ("rowsk_image_sha256", C1_QUALIFIED_ROWSK_IMAGE_SHA256),
    ):
        arr = payload.get(field)
        require(
            isinstance(arr, list) and len(arr) == 4 and all(is_hex64(x) for x in arr),
            f"{field} must be a 4-tuple of 64-hex SHA-256 digests",
        )
        require(
            arr == expected_prefix_list,
            f"{field} differs from C1-qualified 4-tuple: {arr} vs {expected_prefix_list}",
        )

    bvgk_arr = payload.get("bvgk_image_sha256")
    require(
        isinstance(bvgk_arr, list)
        and len(bvgk_arr) == 4
        and all(is_hex64(x) for x in bvgk_arr),
        "bvgk_image_sha256 must be a 4-tuple of 64-hex SHA-256 digests",
    )
    require(
        bvgk_arr[0] == EXPECTED_BVGK_SOURCE_SHA256,
        f"bvgk_image_sha256[0] (source) {bvgk_arr[0]} != {EXPECTED_BVGK_SOURCE_SHA256}",
    )
    require(
        len(set(bvgk_arr)) == 4,
        "bvgk_image_sha256 4-tuple entries must be distinct (source, ptx, cubin, admit_ptx)",
    )

    controls = payload.get("controls")
    require(
        isinstance(controls, list) and len(controls) == 5,
        "controls must be a 5-element array [V0, RC, UT/JR/DC/V3, RF, C1R]",
    )

    expected_names = [
        "v0_source_identity",
        "rc_recarry_parity",
        "ut_jr_dc_v3_block_sweep",
        "rf_refusals",
        "c1r_v2_scoring_parity",
    ]
    actual_names = [c.get("name") if isinstance(c, dict) else None for c in controls]
    require(
        actual_names == expected_names,
        f"control names/order mismatch: expected {expected_names}, got {actual_names}",
    )

    # 1. V0 source identity
    v0 = controls[0]
    require(v0.get("status") == "PASS", "v0_source_identity status must be PASS")
    require(
        v0.get("bvgk_extends_rowsk_source") is True,
        "v0_source_identity.bvgk_extends_rowsk_source must be true",
    )
    require(
        v0.get("rowsk_extends_rows_source") is True,
        "v0_source_identity.rowsk_extends_rows_source must be true",
    )

    # 2. RC re-carry parity
    rc = controls[1]
    verify_tally_fields(rc, "rc_recarry_parity")
    require(rc.get("status") == "PASS" and "error" not in rc, "rc_recarry_parity must PASS")
    require(rc["cases"] == 64, f"rc_recarry_parity.cases expected 64, got {rc['cases']}")
    require(rc["lanes"] > 0, "rc_recarry_parity.lanes must be > 0")
    require(
        rc["applied_updates"] == 0 and rc["commits"] == 0,
        "rc_recarry_parity must apply 0 updates and 0 commits",
    )

    # 3. UT / JR / DC / V3 block sweep
    sweep = controls[2]
    verify_tally_fields(sweep, "ut_jr_dc_v3_block_sweep")
    require(
        sweep.get("status") == "PASS" and "error" not in sweep,
        "ut_jr_dc_v3_block_sweep must PASS without error",
    )
    sub_controls = sweep.get("controls")
    expected_sub_controls = [
        {"name": "ut_original_untouched", "status": "PASS"},
        {"name": "jr_joint_reference", "status": "PASS"},
        {"name": "dc_decision_coverage_v4_v5", "status": "PASS"},
        {"name": "v3_block_equals_sequential", "status": "PASS"},
    ]
    require(
        sub_controls == expected_sub_controls,
        f"ut_jr_dc_v3_block_sweep sub-controls mismatch: {sub_controls}",
    )
    require(
        sweep["cases"] == 804
        and sweep["refused_blocks"] == 0
        and sweep["proposals"] + sweep["refused_blocks"] == sweep["cases"],
        f"ut_jr_dc_v3_block_sweep expected 804 cases == proposals and 0 refused_blocks, got cases={sweep['cases']} proposals={sweep['proposals']} refused_blocks={sweep['refused_blocks']}",
    )
    # Minimum test power / UNDERPOWERED rules (sheet §Bar DC & §Minimum test power)
    for cat in ("empty", "single", "joint_wins", "joint_loses_or_ties"):
        require(
            sweep[cat] > 0,
            f"DC: UNDERPOWERED: decision category {cat!r} has count 0 (instrument blocker)",
        )
    require(
        sweep["multi_member_proposals"] > 0,
        "JR: UNDERPOWERED: multi_member_proposals must be > 0",
    )
    require(
        sweep["v3_joint_cases"] > 0,
        "V3: UNDERPOWERED: v3_joint_cases must be > 0",
    )
    # Partition identities
    require(
        sweep["empty"] + sweep["single"] + sweep["multi_member_proposals"]
        == sweep["proposals"],
        "sweep proposal partition empty + single + multi_member_proposals != proposals",
    )
    require(
        sweep["joint_wins"] + sweep["joint_loses_or_ties"] == sweep["multi_member_proposals"],
        "sweep multi-member partition joint_wins + joint_loses_or_ties != multi_member_proposals",
    )
    require(
        sweep["commits"]
        == sweep["single"] + sweep["joint_wins"] + sweep["joint_loses_or_ties"],
        "sweep commits != single + joint_wins + joint_loses_or_ties",
    )
    require(
        sweep["v3_cases"] == sweep["commits"],
        "sweep v3_cases must equal commits",
    )
    require(
        sweep["v3_joint_cases"] == sweep["joint_wins"],
        "sweep v3_joint_cases must equal joint_wins",
    )
    min_composite_updates = (
        2 * sweep["single"]
        + 4 * sweep["joint_loses_or_ties"]
        + 5 * sweep["joint_wins"]
    )
    require(
        sweep["applied_updates"] >= min_composite_updates,
        f"sweep applied_updates {sweep['applied_updates']} < composite floor {min_composite_updates}",
    )

    # 4. RF refusals
    rf = controls[3]
    verify_tally_fields(rf, "rf_refusals")
    require(rf.get("status") == "PASS" and "error" not in rf, "rf_refusals must PASS")
    require(
        rf["cases"] == 9 and rf["refusals"] == 9,
        f"rf_refusals expected 9 cases and 9 refusals, got {rf['cases']}/{rf['refusals']}",
    )
    require(
        rf["applied_updates"] == 1 and rf["commits"] == 0,
        f"rf_refusals expected applied_updates == 1 and commits == 0, got {rf['applied_updates']}/{rf['commits']}",
    )

    # 5. C1R V2 scoring parity
    c1r = controls[4]
    verify_tally_fields(c1r, "c1r_v2_scoring_parity")
    require(
        c1r.get("status") == "PASS" and "error" not in c1r,
        "c1r_v2_scoring_parity must PASS",
    )
    detail = c1r.get("detail")
    require(isinstance(detail, dict), "c1r_v2_scoring_parity.detail must be an object")
    require(
        detail.get("v2_blocks") == 2028
        and detail.get("v2_successes") == 5130
        and detail.get("v2_racy_refusals") == 0,
        f"c1r_v2_scoring_parity.detail unexpected counts: {detail}",
    )
    require(
        c1r["cases"] == detail["v2_successes"] + c1r["refusals"],
        "c1r_v2_scoring_parity.cases != v2_successes + refusals",
    )
    require(
        c1r["applied_updates"] == 0 and c1r["commits"] == 0,
        "c1r_v2_scoring_parity must apply 0 updates",
    )

    # Total applied_updates sum check
    summed_updates = sum(c["applied_updates"] for c in controls[1:])
    require(
        is_u64(payload.get("applied_updates"))
        and payload["applied_updates"] == summed_updates,
        f"top-level applied_updates {payload.get('applied_updates')} != control sum {summed_updates}",
    )

    tp_summary = verify_throughput(payload.get("throughput"))

    return {
        "verdict": "PASS",
        "suite": EXPECTED_SUITE,
        "training": payload["training"],
        "driver_source_sha256": driver_sha,
        "bvgk_image_sha256": bvgk_arr,
        "applied_updates": payload["applied_updates"],
        "sweep_counts": {
            "cases": sweep["cases"],
            "empty": sweep["empty"],
            "single": sweep["single"],
            "joint_wins": sweep["joint_wins"],
            "joint_loses_or_ties": sweep["joint_loses_or_ties"],
            "commits": sweep["commits"],
            "v3_joint_cases": sweep["v3_joint_cases"],
        },
        "throughput": tp_summary,
    }


def verify_evidence_dir(evidence_dir: Path) -> dict[str, Any]:
    """Verify SHA256SUMS and D29/source-head files in an evidence directory."""
    sha_sums_path = evidence_dir / "SHA256SUMS"
    require(sha_sums_path.is_file(), f"missing SHA256SUMS: {sha_sums_path}")
    lines = [
        line.strip()
        for line in sha_sums_path.read_text(encoding="utf-8").splitlines()
        if line.strip()
    ]
    require(len(lines) > 0, "SHA256SUMS is empty")
    verified_files = 0
    for line in lines:
        parts = line.split()
        require(len(parts) == 2, f"malformed SHA256SUMS line: {line!r}")
        digest, rel_path = parts
        require(is_hex64(digest), f"invalid digest in SHA256SUMS: {digest!r}")
        clean_rel = rel_path[2:] if rel_path.startswith("./") else rel_path
        require(
            clean_rel != "SHA256SUMS",
            "SHA256SUMS must not list a self-entry",
        )
        target = evidence_dir / clean_rel
        require(target.is_file(), f"missing file listed in SHA256SUMS: {target}")
        actual = hashlib.sha256(target.read_bytes()).hexdigest()
        require(
            actual == digest,
            f"SHA-256 mismatch for {clean_rel}: expected {digest}, got {actual}",
        )
        verified_files += 1

    c2_dir = evidence_dir / "bvgk-c2-qualify-20261019"
    if c2_dir.is_dir():
        d29_txt = (c2_dir / "d29.txt").read_text(encoding="utf-8")
        require(
            d29_txt.count("PASS:") == 6 and "Static scope PASS." in d29_txt,
            "d29.txt must contain all 6 PASS lines and 'Static scope PASS.'",
        )
        d29_self = (c2_dir / "d29-selftest.txt").read_text(encoding="utf-8")
        require(
            d29_self.startswith("PASS: missing, renamed-extra"),
            "d29-selftest.txt did not pass",
        )
        head_txt = (c2_dir / "source-head.txt").read_text(encoding="utf-8").strip()
        require(bool(HEX40_RE.match(head_txt)), f"invalid source-head.txt: {head_txt!r}")

    return {
        "sha256sums_sha256": hashlib.sha256(sha_sums_path.read_bytes()).hexdigest(),
        "verified_files": verified_files,
    }


def make_synthetic_valid_payload() -> dict[str, Any]:
    single_raw = [131880000 + i * 100 for i in range(20)]
    k1_raw = [129650000 + i * 100 for i in range(20)]
    k4_raw = [259970000 + i * 100 for i in range(20)]
    k16_raw = [259130000 + i * 100 for i in range(20)]
    return {
        "schema": EXPECTED_SCHEMA,
        "suite": EXPECTED_SUITE,
        "status": "PASS",
        "training": "HELD",
        "applied_updates": 3419,
        "applied_updates_scope": EXPECTED_APPLIED_UPDATES_SCOPE,
        "driver_source_sha256": "f4509049add30b7b756cf5a9ddf97d5633ad4aa790028d8b1b210986945d09e9",
        "rows_image_sha256": list(C1_QUALIFIED_ROWS_IMAGE_SHA256),
        "rowsk_image_sha256": list(C1_QUALIFIED_ROWSK_IMAGE_SHA256),
        "bvgk_image_sha256": [
            EXPECTED_BVGK_SOURCE_SHA256,
            "e3fb57ef0ce1ff0c2ff8d1fbb9c2347672dc5e71da04806f77ff35fe082931f5",
            "85ed1da8ed444eee2560a440aec65b951a17d32d1a74a775446843ba2ca81157",
            "16abad4a9a2f8a44b1bc7537c5e642d57b53a9917d486d82b98786c12701193b",
        ],
        "controls": [
            {
                "name": "v0_source_identity",
                "status": "PASS",
                "bvgk_extends_rowsk_source": True,
                "rowsk_extends_rows_source": True,
            },
            {
                "name": "rc_recarry_parity",
                "status": "PASS",
                "cases": 64,
                "proposals": 0,
                "multi_member_proposals": 0,
                "empty": 0,
                "single": 0,
                "joint_wins": 0,
                "joint_loses_or_ties": 0,
                "refused_blocks": 0,
                "commits": 0,
                "v3_cases": 0,
                "v3_joint_cases": 0,
                "lanes": 14080,
                "racy_status": 0,
                "refusals": 0,
                "applied_updates": 0,
                "detail": {},
            },
            {
                "name": "ut_jr_dc_v3_block_sweep",
                "status": "PASS",
                "cases": 804,
                "proposals": 804,
                "multi_member_proposals": 319,
                "empty": 140,
                "single": 345,
                "joint_wins": 260,
                "joint_loses_or_ties": 59,
                "refused_blocks": 0,
                "commits": 664,
                "v3_cases": 664,
                "v3_joint_cases": 260,
                "lanes": 0,
                "racy_status": 0,
                "refusals": 0,
                "applied_updates": 3418,
                "detail": {},
                "controls": [
                    {"name": "ut_original_untouched", "status": "PASS"},
                    {"name": "jr_joint_reference", "status": "PASS"},
                    {"name": "dc_decision_coverage_v4_v5", "status": "PASS"},
                    {"name": "v3_block_equals_sequential", "status": "PASS"},
                ],
            },
            {
                "name": "rf_refusals",
                "status": "PASS",
                "cases": 9,
                "proposals": 0,
                "multi_member_proposals": 0,
                "empty": 0,
                "single": 0,
                "joint_wins": 0,
                "joint_loses_or_ties": 0,
                "refused_blocks": 0,
                "commits": 0,
                "v3_cases": 0,
                "v3_joint_cases": 0,
                "lanes": 0,
                "racy_status": 0,
                "refusals": 9,
                "applied_updates": 1,
                "detail": {},
            },
            {
                "name": "c1r_v2_scoring_parity",
                "status": "PASS",
                "cases": 5130,
                "proposals": 0,
                "multi_member_proposals": 0,
                "empty": 0,
                "single": 0,
                "joint_wins": 0,
                "joint_loses_or_ties": 0,
                "refused_blocks": 0,
                "commits": 0,
                "v3_cases": 0,
                "v3_joint_cases": 0,
                "lanes": 0,
                "racy_status": 0,
                "refusals": 0,
                "applied_updates": 0,
                "detail": {
                    "v2_blocks": 2028,
                    "v2_successes": 5130,
                    "v2_racy_refusals": 0,
                },
            },
        ],
        "throughput": {
            "shape": EXPECTED_TP_SHAPE,
            "scope": EXPECTED_TP_SCOPE,
            "qualified": False,
            "single_propose_rows_raw_samples_ns": single_raw,
            "single_propose_rows_median_ns": sorted(single_raw)[10],
            "single_propose_rows_p95_ns": sorted(single_raw)[18],
            "cases": [
                {
                    "k": 1,
                    "raw_samples_ns": k1_raw,
                    "block_median_ns": sorted(k1_raw)[10],
                    "block_p95_ns": sorted(k1_raw)[18],
                    "ns_per_coordinate": sorted(k1_raw)[10] // 1,
                    "s_len": 1,
                    "joint_verify_ran": False,
                    "joint_committed": False,
                },
                {
                    "k": 4,
                    "raw_samples_ns": k4_raw,
                    "block_median_ns": sorted(k4_raw)[10],
                    "block_p95_ns": sorted(k4_raw)[18],
                    "ns_per_coordinate": sorted(k4_raw)[10] // 4,
                    "s_len": 2,
                    "joint_verify_ran": True,
                    "joint_committed": False,
                },
                {
                    "k": 16,
                    "raw_samples_ns": k16_raw,
                    "block_median_ns": sorted(k16_raw)[10],
                    "block_p95_ns": sorted(k16_raw)[18],
                    "ns_per_coordinate": sorted(k16_raw)[10] // 16,
                    "s_len": 8,
                    "joint_verify_ran": True,
                    "joint_committed": False,
                },
            ],
        },
    }


def run_self_tests(repo_root: Path | None = None) -> dict[str, Any]:
    base = make_synthetic_valid_payload()
    passed = 0
    total = 0

    def check_pass(label: str, doc: dict[str, Any]) -> None:
        nonlocal passed, total
        total += 1
        verify_bvgk_qualification_payload(doc)
        passed += 1

    def check_fail(label: str, mutator: Any, expected_substr: str) -> None:
        nonlocal passed, total
        total += 1
        doc = copy.deepcopy(base)
        mutator(doc)
        try:
            verify_bvgk_qualification_payload(doc)
        except VerificationError as exc:
            if expected_substr not in str(exc):
                raise AssertionError(
                    f"self-test {label!r} failed with {str(exc)!r}, expected {expected_substr!r}"
                ) from exc
            passed += 1
            return
        raise AssertionError(f"self-test {label!r} unexpectedly passed")

    check_pass("valid_synthetic_receipt", base)
    check_fail("wrong_schema", lambda d: d.__setitem__("schema", "wrong/1"), "schema mismatch")
    check_fail("training_not_held", lambda d: d.__setitem__("training", "RELEASED"), "training must remain 'HELD'")
    check_fail("wrong_scope", lambda d: d.__setitem__("applied_updates_scope", "corpus"), "applied_updates_scope mismatch")
    check_fail(
        "v0_false",
        lambda d: d["controls"][0].__setitem__("bvgk_extends_rowsk_source", False),
        "bvgk_extends_rowsk_source",
    )
    check_fail(
        "rc_zero_lanes",
        lambda d: d["controls"][1].__setitem__("lanes", 0),
        "rc_recarry_parity.lanes must be > 0",
    )
    check_fail(
        "rc_nonzero_updates",
        lambda d: d["controls"][1].__setitem__("applied_updates", 1),
        "rc_recarry_parity must apply 0 updates",
    )
    check_fail(
        "dc_underpowered_empty",
        lambda d: d["controls"][2].__setitem__("empty", 0),
        "DC: UNDERPOWERED",
    )
    check_fail(
        "dc_underpowered_single",
        lambda d: d["controls"][2].__setitem__("single", 0),
        "DC: UNDERPOWERED",
    )
    check_fail(
        "dc_underpowered_joint_wins",
        lambda d: d["controls"][2].__setitem__("joint_wins", 0),
        "DC: UNDERPOWERED",
    )
    check_fail(
        "dc_underpowered_joint_loses_or_ties",
        lambda d: d["controls"][2].__setitem__("joint_loses_or_ties", 0),
        "DC: UNDERPOWERED",
    )
    check_fail(
        "v3_underpowered_zero_joint",
        lambda d: d["controls"][2].__setitem__("v3_joint_cases", 0),
        "V3: UNDERPOWERED",
    )
    check_fail(
        "sweep_partition_mismatch",
        lambda d: d["controls"][2].__setitem__("commits", 663),
        "sweep commits !=",
    )
    check_fail(
        "rf_wrong_refusal_count",
        lambda d: d["controls"][3].__setitem__("refusals", 8),
        "rf_refusals expected 9",
    )
    check_fail(
        "c1r_wrong_blocks",
        lambda d: d["controls"][4]["detail"].__setitem__("v2_blocks", 2027),
        "c1r_v2_scoring_parity.detail",
    )
    check_fail(
        "applied_updates_sum_mismatch",
        lambda d: d.__setitem__("applied_updates", 3420),
        "top-level applied_updates",
    )
    check_fail(
        "tp_qualified_true",
        lambda d: d["throughput"].__setitem__("qualified", True),
        "throughput.qualified must be false",
    )
    check_fail(
        "tp_ns_per_coordinate_mismatch",
        lambda d: d["throughput"]["cases"][2].__setitem__("ns_per_coordinate", 123),
        "ns_per_coordinate != block_median_ns // k",
    )
    check_fail(
        "sweep_nonzero_refused_blocks",
        lambda d: (
            d["controls"][2].__setitem__("refused_blocks", 1),
            d["controls"][2].__setitem__("empty", 139),
        ),
        "0 refused_blocks",
    )
    check_fail(
        "sweep_applied_updates_below_composite_floor",
        lambda d: (
            d["controls"][2].__setitem__("applied_updates", 2225),
            d.__setitem__("applied_updates", 2226),
        ),
        "composite floor 2226",
    )

    # Static Device::apply guard self-tests (comments/strings allowed; no-amp and whitespace calls rejected)
    total += 1
    verify_no_guarded_single_apply(
        '// Device::apply(&p) in line comment\n/* dev.apply(p) in block comment */\nlet msg = "Device::apply(p)";\ndev.apply_block(&proposal);',
        "synthetic_comment_and_string",
        require_apply_block=True,
    )
    passed += 1

    for bad_label, bad_src in (
        ("static_apply_no_ampersand", "single.apply(proposal);\nsingle.apply_block(&proposal);"),
        ("static_apply_whitespace", "Device :: apply ( &proposal );\nsingle.apply_block(&proposal);"),
    ):
        total += 1
        try:
            verify_no_guarded_single_apply(bad_src, bad_label, require_apply_block=True)
            raise AssertionError(f"self-test {bad_label!r} unexpectedly passed")
        except VerificationError:
            passed += 1

    host_hashes = None
    if repo_root is not None:
        total += 1
        host_hashes = compute_host_source_hashes(repo_root)
        verify_bvgk_qualification_payload(
            base,
            expected_driver_sha256=host_hashes["driver_source_sha256"],
        )
        passed += 1

    return {
        "self_test": "PASS",
        "passed": passed,
        "total": total,
        "host_hashes": host_hashes,
    }


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Deterministic host-only consumer verifier for W3 BVG-K Stage C2 bvgk-qualify."
    )
    parser.add_argument("--self-test", action="store_true", help="Run positive and negative self-tests")
    parser.add_argument("--receipt", type=Path, help="Path to bvgk-qualification.json")
    parser.add_argument("--evidence-dir", type=Path, help="Path to evidence directory with SHA256SUMS and D29 logs")
    parser.add_argument("--repo-root", type=Path, help="Repository root for host source hash and static apply guard checks")
    args = parser.parse_args()

    if not args.self_test and args.receipt is None and args.evidence_dir is None:
        parser.error("specify at least one of --self-test, --receipt, or --evidence-dir")

    out: dict[str, Any] = {}
    expected_driver_sha = None
    if args.repo_root is not None:
        host_hashes = compute_host_source_hashes(args.repo_root)
        out["host_source_hashes"] = host_hashes
        expected_driver_sha = host_hashes["driver_source_sha256"]

    if args.self_test:
        out["self_test"] = run_self_tests(args.repo_root)

    if args.receipt is not None:
        raw_bytes = args.receipt.read_bytes()
        payload = json.loads(raw_bytes.decode("utf-8"))
        res = verify_bvgk_qualification_payload(payload, expected_driver_sha256=expected_driver_sha)
        res["receipt_sha256"] = hashlib.sha256(raw_bytes).hexdigest()
        res["receipt_bytes"] = len(raw_bytes)
        out["receipt_verification"] = res

    if args.evidence_dir is not None:
        out["evidence_verification"] = verify_evidence_dir(args.evidence_dir)

    print(json.dumps(out, indent=2, sort_keys=True))
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except VerificationError as err:
        print(json.dumps({"status": "FAIL", "error": str(err)}, sort_keys=True), file=sys.stderr)
        sys.exit(1)
