#!/usr/bin/env python3
"""Host-only deterministic consumer verifier for the Injective Attention runbook and pre-flight.

Verifies:
1. Runbook & static evidence contract (`docs/labs/injective-attention-runbook-2026-10-19/`,
   `docs/harness/prep/m1-attention-injective-init-DRAFT-2026-10-18.md`,
   `crates/uor-r4-discrete/src/attention_learning.rs`):
   - `evidence/SHA256SUMS` (no self-entry; digest `a6da44d0…`; 6 pinned files match byte-for-byte)
   - `prime-zeta-v4096-b4-c192-k8.json` (`83c1380a…`, inner `table_sha256` `e9e03bcd…`,
     131,072 words, 4,096 distinct vectors)
   - Single-key config diff against parent `rows_v1` configs (`codebook_initialization` only)
   - `scope` and D40 owner-release phrase derivation
   - Seven-item documentary contract (Codex `6257545219` / `6257642392`, closed in `#2730` + `#2731`)
2. Pre-flight GPU0 receipt directory (`injective-preflight-20261019/`):
   - `G-Init` + negative control (a) host refusal
   - `G-Occ` (`occupied_cells > 16`, zero within-cell tie mass, no split alias group)
   - `B3_init(P_succ) >= 500` and `net_g_cap > max(500, top1_control(cp0) + 24)` join
   - Exact partition `P_succ + complement == full` (`127 + 300 == 427` over `3,847 + 10,440 == 14,287`)
   - Truthful terminal classification:
     - `TERMINAL_PREFLIGHT_FAIL_INSTRUMENT_UNREACHABLE` when `B3_init < 500` or `net_g_cap <= max(500, top1_control + 24)`,
       with moot downstream controls (`g_init_neg_b`, `inherited_battery`, Step 5 plans) accepted as `NOT_RUN` / `WITHHELD`
       per sheet §3 and Codex `6257784262`
     - `PREFLIGHT_PASS_READY_FOR_PLAN` when `B3_init >= 500` and `net_g_cap > max(500, top1_control + 24)`
       and all remaining controls (`g_init_neg_b`, `inherited_battery`) are present and PASS.

Host-only; never creates a CUDA context, constructs a model, or alters a hold.
"""

from __future__ import annotations

import argparse
import copy
import hashlib
import json
import re
import struct
import sys
from pathlib import Path
from typing import Any

EXPECTED_RUNBOOK_SHA256SUMS_DIGEST = (
    "a6da44d0eaacd5f4f6bea2430c8418a9de185c4dcf09395b7e384669c687cab3"
)
EXPECTED_RUNBOOK_PINS = {
    "injective-frozen-zero-seed1.json": "568adfd0df9941f3c58aa6d102c394bddd41ec05c7015097381e06514605ee09",
    "injective-relation-seed1.json": "7927b1ddde3025d947ce56166c0a30e3ad8d310211aaab1df80851a4ae611bda",
    "parent-rows-frozen-zero-seed1.json": "6a035717130a5e7106e19ca585b4de4383506b425ce403d8a13d7ab44622e1cf",
    "parent-rows-relation-seed1.json": "52b32dc8747148e0cd385d309dbfb1c3cc750569d11ff3932745313e134cc670",
    "prime-zeta-v4096-b4-c192-k8.json": "83c1380a8f507e51d0342faa1b089dea6a20561a03552208c093fa640afc124b",
    "spec.json": "7994b8601a2c5523e94025d4f58de638906fe8cafc677b5af74ce0a0c3280052",
}
EXPECTED_TABLE_FILE_SHA256 = (
    "83c1380a8f507e51d0342faa1b089dea6a20561a03552208c093fa640afc124b"
)
EXPECTED_TABLE_WORDS_SHA256 = (
    "e9e03bcd95b78632a7f18d1b27c07d37f0ba82c52eee692c37da011eadac8804"
)
EXPECTED_SHEET_SHA256 = (
    "e47ce5afb3966bda13cd04bd9b99849b401c028e078de958fa98b5e8b7ed4efb"
)
EXPECTED_SHEET_TITLE = "# Learned attention link from an injective codebook"
EXPECTED_RELATION_CONFIG_SHA256 = (
    "0c3404d43109eafa4230e7245caf4219283417894f6a5adf77b862a326c0d5cf"
)
EXPECTED_FROZEN_ZERO_CONFIG_SHA256 = (
    "368f4285e9a2154fa344a006d86e6036809d1392b13b9402c7f688f788e3339d"
)
EXPECTED_RELATION_LINEAGE_SHA256 = (
    "de8077607683ded31e1f2ee5e7569238da7a28cea4101e62ceae7713f71973b0"
)
EXPECTED_FROZEN_ZERO_LINEAGE_SHA256 = (
    "9ec2a6adcd9ae4b30bc22bb7012fca4e01ce178427a249576297b13b6eb8c65d"
)

HEX64_RE = re.compile(r"^[0-9a-f]{64}$")


class VerificationError(ValueError):
    """Raised when an injective-attention runbook or pre-flight artifact fails verification."""


def require(condition: bool, message: str) -> None:
    if not condition:
        raise VerificationError(message)


def is_u64(value: Any) -> bool:
    return isinstance(value, int) and not isinstance(value, bool) and value >= 0


def is_hex64(value: Any) -> bool:
    return isinstance(value, str) and bool(HEX64_RE.match(value))


def derive_scope_and_phrase(config: dict[str, Any]) -> tuple[str, str]:
    """Replicate `attention_learning::scope` and Casey's D40 release phrase."""
    read_spec = config.get("read_spec", {})
    require(
        read_spec == {"kind": "relation", "kept": 8, "sweeps": 1},
        f"unexpected read_spec: {read_spec!r}",
    )
    policy = config.get("relation_learning_policy")
    if policy == "frozen_zero_v1":
        arm = "relation-frozen-zero"
    elif policy == "learn_v1":
        arm = "relation"
    else:
        raise VerificationError(f"unsupported relation_learning_policy: {policy!r}")

    seed = config.get("seed")
    require(seed in (1, 2), f"seed must be 1 or 2, got {seed!r}")
    cb_init = config.get("codebook_initialization")
    require(
        isinstance(cb_init, dict) and cb_init.get("kind") == "pinned_table_v1",
        "codebook_initialization must be pinned_table_v1",
    )
    table_pin = cb_init.get("table", {})
    table_sha = table_pin.get("sha256", "")
    require(is_hex64(table_sha), f"invalid table sha256: {table_sha!r}")
    prefix = table_sha[:12]
    scope = f"m1-attention-learning-injective-{prefix}-{arm}-k8-seed{seed}-batches0..16404"
    phrase = (
        f"I am Casey; release D4 for {scope} only through batch 16404 "
        "with the pinned DEV transfer and relation-off evaluations"
    )
    return scope, phrase


def verify_runbook_dir(runbook_dir: Path, repo_root: Path | None = None) -> dict[str, Any]:
    """Verify the staged injective-attention runbook, evidence pins, table, configs, and 7-item contract."""
    evidence_dir = runbook_dir / "evidence"
    sha_sums_path = evidence_dir / "SHA256SUMS"
    require(sha_sums_path.is_file(), f"missing {sha_sums_path}")
    sha_sums_bytes = sha_sums_path.read_bytes()
    sha_sums_digest = hashlib.sha256(sha_sums_bytes).hexdigest()
    require(
        sha_sums_digest == EXPECTED_RUNBOOK_SHA256SUMS_DIGEST,
        f"runbook SHA256SUMS digest {sha_sums_digest} != {EXPECTED_RUNBOOK_SHA256SUMS_DIGEST}",
    )

    parsed_pins: dict[str, str] = {}
    for line in sha_sums_bytes.decode("utf-8").splitlines():
        line = line.strip()
        if not line:
            continue
        parts = line.split()
        require(len(parts) == 2, f"malformed SHA256SUMS line: {line!r}")
        digest, name = parts
        require(name != "SHA256SUMS", "SHA256SUMS must not contain a self-entry")
        require(is_hex64(digest), f"invalid SHA-256 in SHA256SUMS: {digest!r}")
        target = evidence_dir / name
        require(target.is_file(), f"missing pinned evidence file: {target}")
        actual = hashlib.sha256(target.read_bytes()).hexdigest()
        require(actual == digest, f"SHA-256 mismatch on {name}: {actual} != {digest}")
        parsed_pins[name] = digest

    require(
        parsed_pins == EXPECTED_RUNBOOK_PINS,
        f"runbook evidence pins mismatch: {parsed_pins}",
    )

    # Verify PrimeZeta table file and inner word digest
    table_doc = json.loads((evidence_dir / "prime-zeta-v4096-b4-c192-k8.json").read_text(encoding="utf-8"))
    require(
        set(table_doc.keys())
        == {"schema", "spec", "mode", "table", "table_sha256", "distinct_vectors"},
        f"table keys mismatch: {sorted(table_doc.keys())}",
    )
    require(table_doc.get("schema") == "uor-r4.codebook-table/1", "table schema mismatch")
    require(table_doc.get("mode") == "prime_zeta", "table mode must be prime_zeta")
    require(
        table_doc.get("spec") == {"vocabulary": 4096, "blocks": 4, "context": 192, "kept_cells": 8},
        f"table spec mismatch: {table_doc.get('spec')}",
    )
    words = table_doc.get("table")
    require(
        isinstance(words, list) and len(words) == 4096 * 32,
        "table must contain 4096 * 32 = 131072 i32 words",
    )
    word_bytes = struct.pack(f"<{len(words)}i", *words)
    computed_table_sha = hashlib.sha256(word_bytes).hexdigest()
    require(
        table_doc.get("table_sha256") == EXPECTED_TABLE_WORDS_SHA256
        and computed_table_sha == EXPECTED_TABLE_WORDS_SHA256,
        f"table_sha256 mismatch: declared={table_doc.get('table_sha256')}, computed={computed_table_sha}",
    )
    distinct_vectors = len({tuple(words[i : i + 32]) for i in range(0, len(words), 32)})
    require(
        table_doc.get("distinct_vectors") == 4096 and distinct_vectors == 4096,
        f"table distinct_vectors must be 4096, got {distinct_vectors}",
    )

    # Verify parent vs injective config single-key diff
    parent_rel = json.loads((evidence_dir / "parent-rows-relation-seed1.json").read_text(encoding="utf-8"))
    parent_fz = json.loads((evidence_dir / "parent-rows-frozen-zero-seed1.json").read_text(encoding="utf-8"))
    inj_rel = json.loads((evidence_dir / "injective-relation-seed1.json").read_text(encoding="utf-8"))
    inj_fz = json.loads((evidence_dir / "injective-frozen-zero-seed1.json").read_text(encoding="utf-8"))

    expected_pin_obj = {
        "kind": "pinned_table_v1",
        "table": {
            "path": "/workspace/uor-r4/local/driver/data/injective/prime-zeta-v4096-b4-c192-k8.json",
            "sha256": EXPECTED_TABLE_FILE_SHA256,
        },
    }
    for label, parent_cfg, inj_cfg in (
        ("relation", parent_rel, inj_rel),
        ("frozen-zero", parent_fz, inj_fz),
    ):
        require(
            "codebook_initialization" not in parent_cfg,
            f"parent {label} config must not have codebook_initialization",
        )
        require(
            inj_cfg.get("codebook_initialization") == expected_pin_obj,
            f"injective {label} codebook_initialization mismatch",
        )
        stripped = copy.deepcopy(inj_cfg)
        del stripped["codebook_initialization"]
        require(
            stripped == parent_cfg,
            f"injective {label} config differs from parent in keys other than codebook_initialization",
        )

    rel_scope, rel_phrase = derive_scope_and_phrase(inj_rel)
    fz_scope, fz_phrase = derive_scope_and_phrase(inj_fz)

    # Verify 7-item documentary contract in README.md
    readme_text = (runbook_dir / "README.md").read_text(encoding="utf-8")
    verify_runbook_readme_contract(readme_text, rel_scope, fz_scope, rel_phrase, fz_phrase)

    if repo_root is not None:
        sheet_path = repo_root / "docs" / "harness" / "prep" / "m1-attention-injective-init-DRAFT-2026-10-18.md"
        require(sheet_path.is_file(), f"missing sheet: {sheet_path}")
        sheet_bytes = sheet_path.read_bytes()
        sheet_sha = hashlib.sha256(sheet_bytes).hexdigest()
        require(
            sheet_sha == EXPECTED_SHEET_SHA256,
            f"sheet SHA-256 {sheet_sha} != {EXPECTED_SHEET_SHA256}",
        )
        sheet_text = sheet_bytes.decode("utf-8")
        require(
            sheet_text.startswith(EXPECTED_SHEET_TITLE),
            f"sheet must start with {EXPECTED_SHEET_TITLE!r}",
        )
        require(
            any(line.startswith("TEST FITNESS: FIT") for line in sheet_text.splitlines()),
            "sheet must carry TEST FITNESS: FIT",
        )

    return {
        "verdict": "PASS",
        "sha256sums_digest": sha_sums_digest,
        "table_file_sha256": EXPECTED_TABLE_FILE_SHA256,
        "table_words_sha256": computed_table_sha,
        "distinct_vectors": distinct_vectors,
        "scopes": {
            "relation": rel_scope,
            "frozen_zero": fz_scope,
        },
    }


def verify_runbook_readme_contract(
    readme_text: str,
    rel_scope: str,
    fz_scope: str,
    rel_phrase: str,
    fz_phrase: str,
) -> None:
    """Check the 7-item documentary contract (Codex 6257545219 / 6257642392)."""
    require(rel_scope in readme_text and fz_scope in readme_text, "README missing derived scopes")
    require(rel_phrase in readme_text and fz_phrase in readme_text, "README missing D40 owner phrases")
    # 1. SHA256SUMS self-entry rule
    require(
        "every evidence file except itself" in readme_text,
        "README contract item 1 missing: SHA256SUMS except-itself statement",
    )
    # 2. Fresh output dir per attempt + Q1 stop not resumable + external review
    require(
        "fresh, never-used output directory" in readme_text
        and "diagnostic Q1 stop is **not** resumable" in readme_text
        and "external review" in readme_text,
        "README contract item 2 missing: fresh output dir / Q1 non-resumable / external review",
    )
    # 3. Transfer requires genuine paired DEV H_geom decision AND typed PM transfer authorization
    require(
        "H_geom" in readme_text and "typed PM transfer authorization" in readme_text,
        "README contract item 3 missing: paired DEV H_geom + typed PM transfer authorization",
    )
    # 4. G-Occ > 16 (256 expected, not gate), tie mass 0, no split alias group
    require(
        "`occupied_cells` must exceed 16; 256 is the expected value, not the gate" in readme_text
        and "within-cell tie mass must be 0, with no split alias group" in readme_text,
        "README contract item 4 missing: G-Occ > 16 / tie mass 0 / no split alias group",
    )
    # 5. attention-build-identity does not emit scope
    require(
        "(`attention-build-identity` does not emit it)" in readme_text
        and "including the scope above" not in readme_text,
        "README contract item 5 missing: attention-build-identity does not emit scope",
    )
    # 6. Net G-Cap explicit join of raw full-DEV alias ceiling to zero-update B3/P_succ and control cp0 top-1
    require(
        "net G-Cap > max(500, top1_control(cp0) + 24)" in readme_text
        and "join the raw full-DEV alias ceiling to the actual zero-update `B3`/`P_succ` and the control `cp0` top-1"
        in readme_text,
        "README contract item 6 missing: explicit net G-Cap join",
    )
    # 7. Inherited contract explicitly retained
    for token in ("Gate22", "Gate0", "ACTIVE/INERT", "INIT LOCK", "reference battery"):
        require(
            token in readme_text,
            f"README contract item 7 missing inherited battery token {token!r}",
        )


def verify_preflight_bundle(bundle: dict[str, Any]) -> dict[str, Any]:
    """Verify a parsed pre-flight bundle (either loaded from disk or synthetic in self-test).

    Follows sheet §3 and Codex 6257784262:
    - If `B3_init(P_succ) < 500` or `net_g_cap <= max(500, top1_control_cp0 + 24)`,
      returns `TERMINAL_PREFLIGHT_FAIL_INSTRUMENT_UNREACHABLE`, verifying that
      `G-Init`, `G-Init` neg (a), and `G-Occ` passed with 0 updates and `training == "HELD"`,
      and accepting remaining moot controls (`g_init_neg_b`, `inherited_battery`, `step5_plans`)
      as `NOT_RUN` / `WITHHELD`.
    - If `B3_init(P_succ) >= 500` and `net_g_cap > max(500, top1_control_cp0 + 24)`,
      requires `g_init_neg_b` and `inherited_battery` to be present and PASS, returning
      `PREFLIGHT_PASS_READY_FOR_PLAN`.
    """
    table_shas = bundle.get("table_shas")
    require(
        isinstance(table_shas, list)
        and len(table_shas) == 2
        and all(s == EXPECTED_TABLE_FILE_SHA256 for s in table_shas),
        f"table-sha.txt must bind both rebuilt and pinned table to {EXPECTED_TABLE_FILE_SHA256}",
    )

    build_id = bundle.get("build_identity")
    rel_id = bundle.get("relation_identity")
    fz_id = bundle.get("frozen_zero_identity")
    require(
        isinstance(build_id, dict) and isinstance(rel_id, dict) and isinstance(fz_id, dict),
        "missing build/arm identity objects",
    )
    driver_sha = build_id.get("driver_source_sha256")
    exe_sha = build_id.get("executable_sha256")
    require(is_hex64(driver_sha) and is_hex64(exe_sha), "invalid build_identity SHA-256")
    for label, ident, expected_cfg_sha, expected_lin_sha in (
        ("build", build_id, None, None),
        ("relation", rel_id, EXPECTED_RELATION_CONFIG_SHA256, EXPECTED_RELATION_LINEAGE_SHA256),
        (
            "frozen_zero",
            fz_id,
            EXPECTED_FROZEN_ZERO_CONFIG_SHA256,
            EXPECTED_FROZEN_ZERO_LINEAGE_SHA256,
        ),
    ):
        require(ident.get("training") == "HELD", f"{label} identity training must be HELD")
        require(
            ident.get("cuda_context_created") is False,
            f"{label} identity cuda_context_created must be false",
        )
        require(
            "scope" not in ident,
            f"{label} identity must not emit 'scope' (scope comes from scope/check-plan)",
        )
        require(
            ident.get("driver_source_sha256") == driver_sha
            and ident.get("executable_sha256") == exe_sha
            and ident.get("compiled") == build_id.get("compiled"),
            f"{label} identity compiled/driver/executable mismatch against build-identity",
        )
        if expected_cfg_sha is not None:
            require(
                ident.get("config_sha256") == expected_cfg_sha,
                f"{label} config_sha256 mismatch: {ident.get('config_sha256')}",
            )
            require(
                ident.get("lineage_sha256") == expected_lin_sha,
                f"{label} lineage_sha256 mismatch: {ident.get('lineage_sha256')}",
            )

    # G-Init checkpoint-0 summaries + negative (a)
    rel_init = bundle.get("relation_init")
    fz_init = bundle.get("frozen_zero_init")
    require(isinstance(rel_init, dict) and isinstance(fz_init, dict), "missing init summaries")
    expected_cursor = {
        "accepted_updates": 0,
        "completed_batches": 0,
        "corpus_pass": 0,
        "next_coordinate": 0,
        "next_target": 0,
    }
    model_sha = rel_init.get("model_sha256")
    require(is_hex64(model_sha), "invalid init model_sha256")
    for label, init_doc in (("relation", rel_init), ("frozen_zero", fz_init)):
        require(init_doc.get("cursor") == expected_cursor, f"{label} init cursor not zeroed")
        require(init_doc.get("model_sha256") == model_sha, f"{label} checkpoint-0 model_sha256 mismatch")
        require(
            init_doc.get("lineage_driver") == driver_sha,
            f"{label} init lineage_driver != driver_source_sha256",
        )

    neg_a = bundle.get("neg_a_stdout", "")
    require(
        isinstance(neg_a, str) and "invalid discrete model: content hash differs" in neg_a,
        "G-Init negative (a) must refuse edited table with 'invalid discrete model: content hash differs'",
    )

    # G-Occ and raw G-Cap via coverage-census
    census = bundle.get("relation_census")
    require(isinstance(census, dict), "missing relation census.json")
    require(
        census.get("schema") == "uor-r4.coverage-census/1" and census.get("status") == "COMPLETE",
        "census schema/status invalid",
    )
    require(
        census.get("accepted_updates") == 0
        and census.get("completed_batches") == 0
        and census.get("host_cell_reconstruction") is False
        and census.get("model_execution") is False
        and census.get("model_sha256") == model_sha,
        "census zero-update / provenance invariants failed",
    )
    require(
        census.get("vocabulary") == 4096
        and census.get("blocks") == 4
        and census.get("distinct_vectors") == 4096
        and census.get("alias_group_size_histogram") == {"1": 4096},
        "census injectivity invariants failed (expected 4096 distinct vectors)",
    )
    raw_g_cap = census.get("dev_alias_ceiling_rows")
    require(
        census.get("dev_rows") == 14287
        and raw_g_cap == 14287
        and census.get("dev_distinct_targets") == 674
        and census.get("dev_targets_alias_representative") == 674,
        "census DEV alias ceiling invariants failed",
    )
    tie_mass = census.get("within_cell_tie_mass")
    require(
        tie_mass
        == {
            "dev_rows_targeting_non_representative": 0,
            "tokens_in_alias_groups": 0,
        },
        f"G-Occ within_cell_tie_mass must be 0, got {tie_mass}",
    )
    cells = census.get("cells")
    require(
        isinstance(cells, dict)
        and cells.get("status") == "FROM_GPU_TOKEN_CELL_RECEIPT"
        and is_u64(cells.get("occupied_cells"))
        and cells["occupied_cells"] > 16
        and cells.get("tokens_in_crowded_cells") == 4096,
        f"G-Occ cells check failed (occupied_cells must exceed 16): {cells}",
    )

    # Zero-update DEV eval aggregates and B3 summaries
    rel_eval = bundle.get("relation_eval")
    fz_eval = bundle.get("frozen_zero_eval")
    rel_b3 = bundle.get("relation_b3")
    fz_b3 = bundle.get("frozen_zero_b3")
    require(
        all(isinstance(x, dict) for x in (rel_eval, fz_eval, rel_b3, fz_b3)),
        "missing eval-aggregate or b3/summary objects",
    )

    for label, ev, b3_doc, expected_cfg_file_sha, expected_cfg_sha, expected_lin_sha in (
        (
            "relation",
            rel_eval,
            rel_b3,
            EXPECTED_RUNBOOK_PINS["injective-relation-seed1.json"],
            EXPECTED_RELATION_CONFIG_SHA256,
            EXPECTED_RELATION_LINEAGE_SHA256,
        ),
        (
            "frozen_zero",
            fz_eval,
            fz_b3,
            EXPECTED_RUNBOOK_PINS["injective-frozen-zero-seed1.json"],
            EXPECTED_FROZEN_ZERO_CONFIG_SHA256,
            EXPECTED_FROZEN_ZERO_LINEAGE_SHA256,
        ),
    ):
        require(
            ev.get("updates_applied_by_evaluation") == 0
            and ev.get("evaluation_mutates_model") is False
            and ev.get("accepted_updates_in_model") == 0
            and ev.get("model_storage_sha256") == model_sha,
            f"{label} eval-aggregate zero-update / model_storage_sha256 invariant failed",
        )
        agg = ev.get("aggregate", {})
        require(
            agg.get("scored_rows") == 14287
            and agg.get("occupied_cells") == cells["occupied_cells"]
            and is_u64(agg.get("target_admitted_rows"))
            and is_u64(agg.get("top1_correct")),
            f"{label} eval-aggregate counts invalid",
        )

        require(
            b3_doc.get("schema") == "uor-r4.attention-preflight-dev-b3/1"
            and b3_doc.get("status") == "VALIDATED_RETAINED_DEV_OBSERVATIONS"
            and b3_doc.get("training") == "HELD"
            and b3_doc.get("q4") == "NOT_EXECUTABLE"
            and b3_doc.get("release") == "STILL_REQUIRED"
            and b3_doc.get("updates_applied") == 0
            and b3_doc.get("completed_batches") == 0
            and b3_doc.get("model_epoch") == 0
            and b3_doc.get("evaluation_mutates_model") is False
            and b3_doc.get("cuda_context_created") is False
            and b3_doc.get("model_constructed") is False
            and b3_doc.get("scoring_performed_by_this_command") is False
            and b3_doc.get("compiled_identity_checked") is True
            and b3_doc.get("config_sha256") == expected_cfg_sha
            and b3_doc.get("lineage_sha256") == expected_lin_sha
            and b3_doc.get("model_storage_sha256") == model_sha
            and b3_doc.get("config_file", {}).get("sha256") == expected_cfg_file_sha,
            f"{label} b3/summary metadata/hold/identity check failed",
        )
        b3 = b3_doc.get("b3", {})
        full = b3.get("full", {})
        psucc = b3.get("psucc", {})
        comp = b3.get("complement", {})
        require(
            full.get("rows") == 14287
            and psucc.get("rows") == 3847
            and comp.get("rows") == 10440
            and psucc["rows"] + comp["rows"] == full["rows"],
            f"{label} B3 row denominator partition 3847 + 10440 == 14287 failed: {b3}",
        )
        require(
            is_u64(psucc.get("target_admitted"))
            and is_u64(comp.get("target_admitted"))
            and is_u64(full.get("target_admitted"))
            and psucc["target_admitted"] + comp["target_admitted"] == full["target_admitted"]
            and full["target_admitted"] == agg["target_admitted_rows"],
            f"{label} B3 target_admitted partition psucc + complement == full == eval_aggregate failed: {b3}",
        )

    require(
        rel_b3["checkpoint_file"]["sha256"] == census.get("checkpoint_raw_sha256")
        and rel_b3["driver_checkpoint_sha256"] == census.get("driver_checkpoint_sha256"),
        "relation B3 summary checkpoint hashes do not match relation census.json",
    )

    rel_b3_psucc = rel_b3["b3"]["psucc"]["target_admitted"]
    fz_b3_psucc = fz_b3["b3"]["psucc"]["target_admitted"]
    control_top1_cp0 = fz_eval["aggregate"]["top1_correct"]
    bar_i = control_top1_cp0 + 24
    net_g_cap_threshold = max(500, bar_i)
    # Because distinct_vectors == 4096 and dev_rows_targeting_non_representative == 0,
    # every DEV row targets its alias representative, so net G-Cap on P_succ equals B3_init(P_succ).
    net_g_cap = rel_b3_psucc

    b3_init_pass = (rel_b3_psucc >= 500) and (fz_b3_psucc >= 500)
    net_g_cap_pass = net_g_cap > net_g_cap_threshold

    if not b3_init_pass or not net_g_cap_pass:
        # Terminal pre-flight stop: remaining controls are moot and must not be fabricated
        require(
            bundle.get("owner_plans_issued", False) is False,
            "owner plans must remain WITHHELD / NOT_ISSUED when B3_init or net G-Cap fails",
        )
        return {
            "verdict": "TERMINAL_PREFLIGHT_FAIL_INSTRUMENT_UNREACHABLE",
            "classification": "INSTRUMENT_UNREACHABLE",
            "mechanism_negative": False,
            "dispatch_decision": "DO_NOT_DISPATCH",
            "owner_plans": "WITHHELD",
            "training": "HELD",
            "gates": {
                "g_init": "PASS",
                "g_init_neg_a_host": "PASS",
                "g_occ": "PASS",
                "g_cap_raw": raw_g_cap,
                "b3_init_psucc_relation": rel_b3_psucc,
                "b3_init_psucc_frozen_zero": fz_b3_psucc,
                "b3_init_floor": 500,
                "b3_init_pass": b3_init_pass,
                "control_top1_cp0": control_top1_cp0,
                "bar_i": bar_i,
                "net_g_cap": net_g_cap,
                "net_g_cap_threshold": net_g_cap_threshold,
                "net_g_cap_pass": net_g_cap_pass,
                "g_init_neg_b_gpu_status6": "NOT_RUN_MOOT_AFTER_B3_FAIL",
                "inherited_battery": "NOT_RUN_MOOT_AFTER_B3_FAIL",
            },
            "b3_partition": {
                "psucc": {"rows": 3847, "target_admitted": rel_b3_psucc},
                "complement": {
                    "rows": 10440,
                    "target_admitted": rel_b3["b3"]["complement"]["target_admitted"],
                },
                "full": {
                    "rows": 14287,
                    "target_admitted": rel_b3["b3"]["full"]["target_admitted"],
                },
            },
        }

    # If B3_init and net G-Cap both pass, G-Init neg (b) and the inherited battery are required
    require(
        bundle.get("g_init_neg_b_status6") == "PASS",
        "when B3_init and net G-Cap pass, G-Init negative (b) GPU status-6 check is required",
    )
    require(
        bundle.get("inherited_battery") == "PASS",
        "when B3_init and net G-Cap pass, inherited Gate22/Gate0/ACTIVE-INERT/INIT LOCK battery is required",
    )
    return {
        "verdict": "PREFLIGHT_PASS_READY_FOR_PLAN",
        "classification": "PREFLIGHT_GATES_PASSED",
        "mechanism_negative": False,
        "dispatch_decision": "READY_FOR_STEP5_PLANS_AND_OWNER_RELEASE",
        "training": "HELD",
        "gates": {
            "g_init": "PASS",
            "g_init_neg_a_host": "PASS",
            "g_init_neg_b_gpu_status6": "PASS",
            "g_occ": "PASS",
            "g_cap_raw": raw_g_cap,
            "b3_init_psucc_relation": rel_b3_psucc,
            "b3_init_psucc_frozen_zero": fz_b3_psucc,
            "b3_init_pass": True,
            "control_top1_cp0": control_top1_cp0,
            "bar_i": bar_i,
            "net_g_cap": net_g_cap,
            "net_g_cap_threshold": net_g_cap_threshold,
            "net_g_cap_pass": True,
            "inherited_battery": "PASS",
        },
    }


def load_preflight_bundle_from_dir(preflight_dir: Path) -> dict[str, Any]:
    table_sha_lines = [
        line.strip().split()[0]
        for line in (preflight_dir / "table-sha.txt").read_text(encoding="utf-8").splitlines()
        if line.strip()
    ]
    return {
        "table_shas": table_sha_lines,
        "build_identity": json.loads(
            (preflight_dir / "build-identity.json").read_text(encoding="utf-8")
        ),
        "relation_identity": json.loads(
            (preflight_dir / "relation" / "identity.json").read_text(encoding="utf-8")
        ),
        "frozen_zero_identity": json.loads(
            (preflight_dir / "frozen-zero" / "identity.json").read_text(encoding="utf-8")
        ),
        "relation_init": json.loads(
            (preflight_dir / "relation" / "init-summary.json").read_text(encoding="utf-8")
        ),
        "frozen_zero_init": json.loads(
            (preflight_dir / "frozen-zero" / "init-summary.json").read_text(encoding="utf-8")
        ),
        "neg_a_stdout": (preflight_dir / "neg" / "a.stdout").read_text(encoding="utf-8"),
        "relation_census": json.loads(
            (preflight_dir / "relation" / "census.json").read_text(encoding="utf-8")
        ),
        "relation_eval": json.loads(
            (preflight_dir / "relation" / "eval-aggregate.json").read_text(encoding="utf-8")
        ),
        "frozen_zero_eval": json.loads(
            (preflight_dir / "frozen-zero" / "eval-aggregate.json").read_text(encoding="utf-8")
        ),
        "relation_b3": json.loads(
            (preflight_dir / "relation" / "b3" / "summary.json").read_text(encoding="utf-8")
        ),
        "frozen_zero_b3": json.loads(
            (preflight_dir / "frozen-zero" / "b3" / "summary.json").read_text(encoding="utf-8")
        ),
    }


def make_synthetic_preflight_bundle() -> dict[str, Any]:
    driver_sha = "f4509049add30b7b756cf5a9ddf97d5633ad4aa790028d8b1b210986945d09e9"
    exe_sha = "6280a695de8d0788e5e0f1f0a85079a36050dbc7194dc2d998bade7946d587e2"
    model_sha = "134fd6636d5e256fb0d1320d88de527a5384582b6effe3b54214eb0c32332031"
    rel_ckpt_raw = "8fbbc3a3183d915dfcad1c7cf3a52ec16238dfe2f30f3fd5b797e8fc1d9f8fd3"
    rel_ckpt_drv = "a32f1258f00f12ab7968f45452418f0d38aee4785105ac1b05a0a6e624bac09d"
    compiled = {"source_sha256": "e120906e80f0a6e776a88ea662e94589e3381947f206ea1bdf14637cf0e2e7ef"}
    cursor = {
        "accepted_updates": 0,
        "completed_batches": 0,
        "corpus_pass": 0,
        "next_coordinate": 0,
        "next_target": 0,
    }
    eval_doc = {
        "updates_applied_by_evaluation": 0,
        "evaluation_mutates_model": False,
        "accepted_updates_in_model": 0,
        "model_storage_sha256": model_sha,
        "aggregate": {
            "scored_rows": 14287,
            "occupied_cells": 256,
            "target_admitted_rows": 427,
            "top1_correct": 38,
            "top8_correct": 95,
        },
    }
    return {
        "table_shas": [EXPECTED_TABLE_FILE_SHA256, EXPECTED_TABLE_FILE_SHA256],
        "build_identity": {
            "compiled": compiled,
            "cuda_context_created": False,
            "driver_source_sha256": driver_sha,
            "executable_sha256": exe_sha,
            "training": "HELD",
        },
        "relation_identity": {
            "compiled": compiled,
            "config_sha256": EXPECTED_RELATION_CONFIG_SHA256,
            "cuda_context_created": False,
            "driver_source_sha256": driver_sha,
            "executable_sha256": exe_sha,
            "lineage_sha256": EXPECTED_RELATION_LINEAGE_SHA256,
            "training": "HELD",
        },
        "frozen_zero_identity": {
            "compiled": compiled,
            "config_sha256": EXPECTED_FROZEN_ZERO_CONFIG_SHA256,
            "cuda_context_created": False,
            "driver_source_sha256": driver_sha,
            "executable_sha256": exe_sha,
            "lineage_sha256": EXPECTED_FROZEN_ZERO_LINEAGE_SHA256,
            "training": "HELD",
        },
        "relation_init": {
            "cursor": dict(cursor),
            "model_sha256": model_sha,
            "lineage_driver": driver_sha,
        },
        "frozen_zero_init": {
            "cursor": dict(cursor),
            "model_sha256": model_sha,
            "lineage_driver": driver_sha,
        },
        "neg_a_stdout": "invalid discrete model: content hash differs: neg/a.json\n",
        "relation_census": {
            "schema": "uor-r4.coverage-census/1",
            "status": "COMPLETE",
            "accepted_updates": 0,
            "completed_batches": 0,
            "host_cell_reconstruction": False,
            "model_execution": False,
            "model_sha256": model_sha,
            "checkpoint_raw_sha256": rel_ckpt_raw,
            "driver_checkpoint_sha256": rel_ckpt_drv,
            "vocabulary": 4096,
            "blocks": 4,
            "distinct_vectors": 4096,
            "alias_group_size_histogram": {"1": 4096},
            "dev_rows": 14287,
            "dev_alias_ceiling_rows": 14287,
            "dev_distinct_targets": 674,
            "dev_targets_alias_representative": 674,
            "within_cell_tie_mass": {
                "dev_rows_targeting_non_representative": 0,
                "tokens_in_alias_groups": 0,
            },
            "cells": {
                "status": "FROM_GPU_TOKEN_CELL_RECEIPT",
                "occupied_cells": 256,
                "tokens_in_crowded_cells": 4096,
            },
        },
        "relation_eval": copy.deepcopy(eval_doc),
        "frozen_zero_eval": copy.deepcopy(eval_doc),
        "relation_b3": {
            "schema": "uor-r4.attention-preflight-dev-b3/1",
            "status": "VALIDATED_RETAINED_DEV_OBSERVATIONS",
            "training": "HELD",
            "q4": "NOT_EXECUTABLE",
            "release": "STILL_REQUIRED",
            "updates_applied": 0,
            "completed_batches": 0,
            "model_epoch": 0,
            "evaluation_mutates_model": False,
            "cuda_context_created": False,
            "model_constructed": False,
            "scoring_performed_by_this_command": False,
            "compiled_identity_checked": True,
            "config_sha256": EXPECTED_RELATION_CONFIG_SHA256,
            "lineage_sha256": EXPECTED_RELATION_LINEAGE_SHA256,
            "model_storage_sha256": model_sha,
            "checkpoint_file": {"path": "/rel/init.json", "sha256": rel_ckpt_raw},
            "driver_checkpoint_sha256": rel_ckpt_drv,
            "config_file": {
                "path": "/rel/cfg.json",
                "sha256": EXPECTED_RUNBOOK_PINS["injective-relation-seed1.json"],
            },
            "b3": {
                "psucc": {"rows": 3847, "target_admitted": 127},
                "complement": {"rows": 10440, "target_admitted": 300},
                "full": {"rows": 14287, "target_admitted": 427},
            },
        },
        "frozen_zero_b3": {
            "schema": "uor-r4.attention-preflight-dev-b3/1",
            "status": "VALIDATED_RETAINED_DEV_OBSERVATIONS",
            "training": "HELD",
            "q4": "NOT_EXECUTABLE",
            "release": "STILL_REQUIRED",
            "updates_applied": 0,
            "completed_batches": 0,
            "model_epoch": 0,
            "evaluation_mutates_model": False,
            "cuda_context_created": False,
            "model_constructed": False,
            "scoring_performed_by_this_command": False,
            "compiled_identity_checked": True,
            "config_sha256": EXPECTED_FROZEN_ZERO_CONFIG_SHA256,
            "lineage_sha256": EXPECTED_FROZEN_ZERO_LINEAGE_SHA256,
            "model_storage_sha256": model_sha,
            "checkpoint_file": {"path": "/fz/init.json", "sha256": "0" * 64},
            "driver_checkpoint_sha256": "1" * 64,
            "config_file": {
                "path": "/fz/cfg.json",
                "sha256": EXPECTED_RUNBOOK_PINS["injective-frozen-zero-seed1.json"],
            },
            "b3": {
                "psucc": {"rows": 3847, "target_admitted": 127},
                "complement": {"rows": 10440, "target_admitted": 300},
                "full": {"rows": 14287, "target_admitted": 427},
            },
        },
    }


def run_self_tests() -> dict[str, Any]:
    base = make_synthetic_preflight_bundle()
    passed = 0
    total = 0

    # 1. Terminal B3 fail (127 < 500) -> TERMINAL_PREFLIGHT_FAIL_INSTRUMENT_UNREACHABLE
    total += 1
    res_fail = verify_preflight_bundle(base)
    require(
        res_fail["verdict"] == "TERMINAL_PREFLIGHT_FAIL_INSTRUMENT_UNREACHABLE"
        and res_fail["classification"] == "INSTRUMENT_UNREACHABLE"
        and res_fail["mechanism_negative"] is False
        and res_fail["owner_plans"] == "WITHHELD",
        "synthetic B3-fail bundle did not classify as TERMINAL_PREFLIGHT_FAIL_INSTRUMENT_UNREACHABLE",
    )
    passed += 1

    # 2. Passing preflight bundle (B3 = 600 >= 500, net_g_cap = 600 > 500, neg_b + inherited PASS)
    total += 1
    passing_bundle = copy.deepcopy(base)
    for key in ("relation_b3", "frozen_zero_b3"):
        passing_bundle[key]["b3"]["psucc"]["target_admitted"] = 600
        passing_bundle[key]["b3"]["complement"]["target_admitted"] = 1400
        passing_bundle[key]["b3"]["full"]["target_admitted"] = 2000
    for key in ("relation_eval", "frozen_zero_eval"):
        passing_bundle[key]["aggregate"]["target_admitted_rows"] = 2000
    passing_bundle["g_init_neg_b_status6"] = "PASS"
    passing_bundle["inherited_battery"] = "PASS"
    res_pass = verify_preflight_bundle(passing_bundle)
    require(
        res_pass["verdict"] == "PREFLIGHT_PASS_READY_FOR_PLAN",
        "synthetic passing bundle did not return PREFLIGHT_PASS_READY_FOR_PLAN",
    )
    passed += 1

    def check_fail(label: str, doc_source: dict[str, Any], mutator: Any, expected_substr: str) -> None:
        nonlocal passed, total
        total += 1
        doc = copy.deepcopy(doc_source)
        mutator(doc)
        try:
            verify_preflight_bundle(doc)
        except VerificationError as exc:
            if expected_substr not in str(exc):
                raise AssertionError(
                    f"self-test {label!r} failed with {str(exc)!r}, expected {expected_substr!r}"
                ) from exc
            passed += 1
            return
        raise AssertionError(f"self-test {label!r} unexpectedly passed")

    check_fail(
        "table_sha_mismatch",
        base,
        lambda d: d["table_shas"].__setitem__(0, "0" * 64),
        "table-sha.txt must bind",
    )
    check_fail(
        "identity_emits_scope",
        base,
        lambda d: d["relation_identity"].__setitem__("scope", "bad"),
        "must not emit 'scope'",
    )
    check_fail(
        "init_nonzero_updates",
        base,
        lambda d: d["relation_init"]["cursor"].__setitem__("accepted_updates", 1),
        "init cursor not zeroed",
    )
    check_fail(
        "neg_a_missing_refusal",
        base,
        lambda d: d.__setitem__("neg_a_stdout", "ok"),
        "G-Init negative (a)",
    )
    check_fail(
        "g_occ_occupied_cells_16",
        base,
        lambda d: d["relation_census"]["cells"].__setitem__("occupied_cells", 16),
        "G-Occ cells check failed",
    )
    check_fail(
        "g_occ_nonzero_tie_mass",
        base,
        lambda d: d["relation_census"]["within_cell_tie_mass"].__setitem__(
            "tokens_in_alias_groups", 2
        ),
        "within_cell_tie_mass must be 0",
    )
    check_fail(
        "b3_partition_mismatch",
        base,
        lambda d: d["relation_b3"]["b3"]["full"].__setitem__("target_admitted", 428),
        "B3 target_admitted partition",
    )
    check_fail(
        "b3_denominator_partition_mismatch",
        base,
        lambda d: d["relation_b3"]["b3"]["psucc"].__setitem__("rows", 3848),
        "B3 row denominator partition",
    )
    check_fail(
        "owner_plans_issued_on_failed_b3",
        base,
        lambda d: d.__setitem__("owner_plans_issued", True),
        "owner plans must remain WITHHELD",
    )
    check_fail(
        "passing_b3_missing_neg_b",
        passing_bundle,
        lambda d: d.__setitem__("g_init_neg_b_status6", "NOT_RUN"),
        "G-Init negative (b) GPU status-6 check is required",
    )
    check_fail(
        "passing_b3_missing_inherited_battery",
        passing_bundle,
        lambda d: d.__setitem__("inherited_battery", "NOT_RUN"),
        "inherited Gate22/Gate0/ACTIVE-INERT/INIT LOCK battery is required",
    )
    check_fail(
        "net_g_cap_fails_high_control_top1",
        passing_bundle,
        lambda d: d["frozen_zero_eval"]["aggregate"].__setitem__("top1_correct", 580),
        "G-Init negative (b)",  #Wait: when net_g_cap <= 580+24=604, it classifies as TERMINAL_PREFLIGHT_FAIL!
    ) if False else None

    # Test net G-Cap > max(500, top1_control + 24) when B3_init >= 500 but control_top1 + 24 >= net_g_cap
    total += 1
    high_ctrl = copy.deepcopy(base)
    for key in ("relation_b3", "frozen_zero_b3"):
        high_ctrl[key]["b3"]["psucc"]["target_admitted"] = 550
        high_ctrl[key]["b3"]["complement"]["target_admitted"] = 1450
        high_ctrl[key]["b3"]["full"]["target_admitted"] = 2000
    for key in ("relation_eval", "frozen_zero_eval"):
        high_ctrl[key]["aggregate"]["target_admitted_rows"] = 2000
    high_ctrl["frozen_zero_eval"]["aggregate"]["top1_correct"] = 530  # bar_i = 554 >= 550
    res_high_ctrl = verify_preflight_bundle(high_ctrl)
    require(
        res_high_ctrl["verdict"] == "TERMINAL_PREFLIGHT_FAIL_INSTRUMENT_UNREACHABLE"
        and res_high_ctrl["gates"]["b3_init_pass"] is True
        and res_high_ctrl["gates"]["net_g_cap_pass"] is False,
        "net G-Cap fail when B3 >= 500 but net_g_cap <= top1_control + 24 must be caught",
    )
    passed += 1

    # Test scope & D40 phrase derivation (positive + negative)
    total += 1
    rel_cfg = {
        "read_spec": {"kind": "relation", "kept": 8, "sweeps": 1},
        "relation_learning_policy": "learn_v1",
        "seed": 1,
        "codebook_initialization": {
            "kind": "pinned_table_v1",
            "table": {"sha256": EXPECTED_TABLE_FILE_SHA256},
        },
    }
    fz_cfg = copy.deepcopy(rel_cfg)
    fz_cfg["relation_learning_policy"] = "frozen_zero_v1"
    rel_scope, rel_phrase = derive_scope_and_phrase(rel_cfg)
    fz_scope, fz_phrase = derive_scope_and_phrase(fz_cfg)
    require(
        rel_scope == "m1-attention-learning-injective-83c1380a8f50-relation-k8-seed1-batches0..16404"
        and fz_scope
        == "m1-attention-learning-injective-83c1380a8f50-relation-frozen-zero-k8-seed1-batches0..16404",
        "derived scope mismatch",
    )
    passed += 1

    total += 1
    bad_cfg = copy.deepcopy(rel_cfg)
    bad_cfg["seed"] = 3
    try:
        derive_scope_and_phrase(bad_cfg)
        raise AssertionError("seed=3 unexpectedly passed derive_scope_and_phrase")
    except VerificationError:
        passed += 1

    # Test 7-item README documentary contract (positive + negative)
    valid_readme = (
        f"{rel_scope}\n{fz_scope}\n{rel_phrase}\n{fz_phrase}\n"
        "every evidence file except itself\n"
        "fresh, never-used output directory\n"
        "diagnostic Q1 stop is **not** resumable\n"
        "external review\n"
        "H_geom and typed PM transfer authorization\n"
        "`occupied_cells` must exceed 16; 256 is the expected value, not the gate\n"
        "within-cell tie mass must be 0, with no split alias group\n"
        "(`attention-build-identity` does not emit it)\n"
        "net G-Cap > max(500, top1_control(cp0) + 24): join the raw full-DEV alias ceiling "
        "to the actual zero-update `B3`/`P_succ` and the control `cp0` top-1\n"
        "Gate22 Gate0 ACTIVE/INERT INIT LOCK reference battery\n"
    )
    total += 1
    verify_runbook_readme_contract(valid_readme, rel_scope, fz_scope, rel_phrase, fz_phrase)
    passed += 1

    total += 1
    try:
        verify_runbook_readme_contract(
            valid_readme.replace("(`attention-build-identity` does not emit it)", "including the scope above"),
            rel_scope,
            fz_scope,
            rel_phrase,
            fz_phrase,
        )
        raise AssertionError("README contract item 5 regression unexpectedly passed")
    except VerificationError:
        passed += 1

    return {
        "self_test": "PASS",
        "passed": passed,
        "total": total,
    }


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Deterministic host-only consumer verifier for Injective Attention runbook and pre-flight."
    )
    parser.add_argument("--self-test", action="store_true", help="Run positive and negative self-tests")
    parser.add_argument("--runbook-dir", type=Path, help="Path to docs/labs/injective-attention-runbook-2026-10-19")
    parser.add_argument("--preflight-dir", type=Path, help="Path to injective-preflight-20261019 evidence directory")
    parser.add_argument("--repo-root", type=Path, help="Path to repository root")
    args = parser.parse_args()

    if not args.self_test and args.runbook_dir is None and args.preflight_dir is None:
        parser.error("specify at least one of --self-test, --runbook-dir, or --preflight-dir")

    out: dict[str, Any] = {}
    if args.self_test:
        out["self_test"] = run_self_tests()

    if args.runbook_dir is not None:
        out["runbook_verification"] = verify_runbook_dir(args.runbook_dir, repo_root=args.repo_root)

    if args.preflight_dir is not None:
        bundle = load_preflight_bundle_from_dir(args.preflight_dir)
        out["preflight_verification"] = verify_preflight_bundle(bundle)

    print(json.dumps(out, indent=2, sort_keys=True))
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except VerificationError as err:
        print(json.dumps({"status": "FAIL", "error": str(err)}, sort_keys=True), file=sys.stderr)
        sys.exit(1)
