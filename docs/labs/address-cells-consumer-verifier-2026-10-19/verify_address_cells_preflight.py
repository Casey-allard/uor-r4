#!/usr/bin/env python3
"""Host-only deterministic consumer verifier for the Address-Structured Injective Cells
(`A8` / `S8` / `P8` / `P16` / `A16`) codebook constructor, staged arm configs, and pre-flight.

Verifies:
1. Staged configs & registration contract (`docs/labs/address-cells-preflight-configs-2026-10-19/`,
   `docs/harness/prep/m1-attention-address-structured-cells-DRAFT-2026-10-19.md`,
   `docs/labs/address-cells-codebook-2026-10-19/README.md`,
   `crates/uor-r4-discrete/src/`):
   - `evidence/SHA256SUMS` (`895ec8fb…`, 372 B; no self-entry; 5 arm configs match byte-for-byte)
   - Single/two-field diffs of `p8.json`, `a8.json`, `s8.json`, `p16.json`, `a16.json` against
     `docs/labs/injective-attention-runbook-2026-10-19/evidence/injective-relation-seed1.json` (`7927b1dd…`)
   - Exact `driver_source_sha256` (`44f3b50a…`), `config_sha256`, and `lineage_sha256` across all 5 arms
   - `attention_learning::scope` and D40 owner phrases for K=8 arms (`P8`, `A8`, `S8`), and verification
     that K=16 arms (`P16`, `A16`) pass `DriverConfig::validate` (`init-corpus`/`eval-corpus`) while
     being outside `attention_learning::validate_config` (`kept_cells == 8`)
   - Registration bindings (#2738 / #2742 / #2746 / #2748) and `TEST FITNESS: FIT`
2. Host data-preparation constructor & table pins (`crates/uor-r4-discrete/src/address_cells.rs`,
   `crates/uor-r4-discrete/src/bin/address-cells-codebook.rs`):
   - Reconstructs `BigramCounts` from TRAIN (`3dee858f…` / `b5e7352d…`) at the 2,425,925 eligible
     response-masked target positions (`counts_sha256` `755a5a81…`, `train_tokens_sha256` `3082a50c…`)
   - Reconstructs `A8` (`address_structured_v1`, file `6600bdb1…`, `table_sha256` `4122cbab…`) and
     `S8` (`address_shuffled_v1_seed_20261019`, file `a185bdca…`, `table_sha256` `d2fb8680…`) from
     `BigramCounts` and `PrimeZeta` fallback cells (`83c1380a…`), verifying 4,096 distinct vectors,
     256 occupied cells, exact 16-per-cell quota, 16 orbit roots, zero within-cell tie mass, and
     TRAIN predecessor L1 pull (`22,374,488` on A8 vs `29,881,384` on S8 and `33,871,344` on P8)
3. Five-arm zero-update GPU pre-flight receipt bundle (`--preflight-dir` and `--self-test`):
   - P8 exactness gate: checkpoint 0 must reproduce `427` full-panel and `127` `P_succ` target-admitted
     rows (`300` complement). Any difference is an instrument fault (`INSTRUMENT_FAULT_D31`), never a negative
   - Pod table rebuild identity check (`P8` `83c1380a…`, `A8` `6600bdb1…`, `S8` `a185bdca…`)
   - K=8 arms (`P8`, `A8`, `S8`): `G-Init` + negatives (a) and (b), `G-Occ` (`occupied_cells > 16`,
     256 expected, zero within-cell tie mass, no split alias group, `cell_occupancy_histogram == {"16": 256}`,
     `cell_collision_pairs == 30,720`), `B3` denominator and admission partitions
   - Paired one-sided McNemar discordance on the 3,847 `P_succ` rows (`F = max(24, 2*sqrt(b+c))`, checked
     via exact integer inequality `b - c > 24` and `(b - c)^2 > 4*(b + c)`):
     - `A8 vs P8`: whether placement moved admission
     - `A8 vs S8`: whether the gain is attributed to the predecessor relation vs frequency packing
   - K=16 bracket join precondition & formula check (`P8`, `P16`, `A16`):
     - Every `eval-corpus` row has non-null `target_position` and `target_token`, unique keys across
       14,287 rows, `admitted_cells - address_rivals in {0, 1}`, and exactly 3,847 `is_psucc == True` joins
     - Counting `admitted_cells - address_rivals == 1` on `P8` reproduces `427` overall and `127` on `P_succ`
       before the formula is used for `P16` / `A16` (any miss is `INSTRUMENT_FAULT_D31`)
     - Bracket arms are diagnostic only (`bracket_can_promote = False`) and may be dropped as diagnostics
       if `eval-corpus` refuses K=16 at the pod (#2748 Registration bindings)
   - `A8` promotion battery (`B3_init(P_succ) >= 500` and `net_g_cap > max(500, top1_control(cp0) + 24)`).

Host-only; never creates a CUDA context, constructs a model, or alters a hold.
"""

from __future__ import annotations

import argparse
from collections import defaultdict
import copy
import hashlib
import json
import math
import re
import struct
import sys
from pathlib import Path
from typing import Any

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

ROOTS: list[list[int]] = [
    [2, 2, 0, 0, 0, 0, 0, 0],
    [-2, -2, 0, 0, 0, 0, 0, 0],
    [-2, 2, 0, 0, 0, 0, 0, 0],
    [0, 0, -2, 0, -2, 0, 0, 0],
    [0, 0, -2, 0, 2, 0, 0, 0],
    [0, 0, 0, -2, 0, 0, 0, -2],
    [0, 0, 0, -2, 0, 0, 0, 2],
    [0, 0, 0, 0, 0, -2, -2, 0],
    [0, 0, 0, 0, 0, -2, 2, 0],
    [0, 0, 0, 0, 0, 2, -2, 0],
    [0, 0, 0, 0, 0, 2, 2, 0],
    [0, 0, 0, 2, 0, 0, 0, -2],
    [0, 0, 0, 2, 0, 0, 0, 2],
    [0, 0, 2, 0, -2, 0, 0, 0],
    [0, 0, 2, 0, 2, 0, 0, 0],
    [2, -2, 0, 0, 0, 0, 0, 0],
]
ROOT_INDEX: dict[tuple[int, ...], int] = {tuple(r): idx for idx, r in enumerate(ROOTS)}
ROOT_L1: list[list[int]] = [
    [sum(abs(ROOTS[a][i] - ROOTS[b][i]) for i in range(8)) for b in range(16)]
    for a in range(16)
]
CELL_L1: list[list[int]] = [
    [ROOT_L1[a // 16][b // 16] + ROOT_L1[a % 16][b % 16] for b in range(256)]
    for a in range(256)
]

EXPECTED_DRIVER_SOURCE_SHA256 = (
    "44f3b50ab80a98931996c07dd1ab2f661b11ed0a3e9493b5e10ae9fd90632bdc"
)
EXPECTED_ADDRESS_CELLS_RS_SHA256 = (
    "22dec8e967f7d2bfd410f83067ee69646993bffe0aa3be4996375896e33fa75e"
)
EXPECTED_ADDRESS_CELLS_BIN_SHA256 = (
    "691349582b563dde327c0613cb72a282ce84fd96ed0dc226a409d0a25c19744c"
)
EXPECTED_CONFIGS_SHA256SUMS_DIGEST = (
    "895ec8fb8bd875d7009a0d91e82c838f5a88e636556c4e19614ed6530928c4fb"
)
EXPECTED_STAGED_CONFIG_FILE_PINS: dict[str, str] = {
    "a16.json": "204733c9a21c370eaa9d725d7bb54da7d1d4a7d36877409dc97aef4291bf11a5",
    "a8.json": "469a2ff0fe17a6a80f5875a077b3e6a9436553efdf1c6e7884541220553e9d5e",
    "p16.json": "656fe13e4658e42446a79f44aa1da88fa1cb166e63a2caac81918e1afb03c0ea",
    "p8.json": "7927b1ddde3025d947ce56166c0a30e3ad8d310211aaab1df80851a4ae611bda",
    "s8.json": "b22c6c04ff160b741f1c6c2e8f7806f10ac8e5e3c2027ff72fef9d432f99bfad",
}
EXPECTED_ARM_CONFIG_SHA256: dict[str, str] = {
    "p8": "0c3404d43109eafa4230e7245caf4219283417894f6a5adf77b862a326c0d5cf",
    "a8": "f223d78f45fe26dee4080ae7ca9a5b9ff12d9ef0df847375671d52db733fbecd",
    "s8": "16e37144c966925b36477efee1ee2bbedeb8a18b500454cd50a924ce5d743db5",
    "p16": "0d7438c845fdc8eb4589ca61e707f21f7d4525081e4bc5041a3bd44938929c2d",
    "a16": "93623108cbbf11bb1e96263c89702ce259e060093102e38f89f83aad8c80480c",
}
EXPECTED_ARM_LINEAGE_SHA256: dict[str, str] = {
    "p8": "3849a4bdf93c847baef835c60253421fa14283fb15c47dbcd1c18fca23916025",
    "a8": "cf6cf50f76eb9f6950fc518258a878df4cf8e44c9ec0c010e85ceb50c8cb8940",
    "s8": "fda6bd48d0d655631e17c284297b5c7d4b3cc8f8b23990f8be97cb92e246f761",
    "p16": "8cbb8df7a73477a01a96698340a44a69432cb9140a7a71e03321a7e5537852c1",
    "a16": "0340e25ff976491d5f01ef3bc824980812c49a84ff922450cee58ae95855ddf5",
}

EXPECTED_P8_TABLE_FILE_SHA256 = (
    "83c1380a8f507e51d0342faa1b089dea6a20561a03552208c093fa640afc124b"
)
EXPECTED_P8_TABLE_WORDS_SHA256 = (
    "e9e03bcd95b78632a7f18d1b27c07d37f0ba82c52eee692c37da011eadac8804"
)
EXPECTED_A8_TABLE_FILE_SHA256 = (
    "6600bdb1306dbaea4d77f5bf021e717a6bef791e44c1ec7d7c9bb9f0e8cd3cd1"
)
EXPECTED_A8_TABLE_WORDS_SHA256 = (
    "4122cbab44bd88658743984fc1ab79059bf032c3e2fe526ffcf4ea5f95d9cd74"
)
EXPECTED_S8_TABLE_FILE_SHA256 = (
    "a185bdca07077fe7157e947ff1fa80919e6f99165a4dd5a04061a97873c92718"
)
EXPECTED_S8_TABLE_WORDS_SHA256 = (
    "d2fb8680a16e8017f6f0e6518df6810f6206023b0d14388dc076d4cfde2ebc5b"
)
EXPECTED_COUNTS_SHA256 = (
    "755a5a81ddbf0f9d81b4d0bd88e54435b96f5485f87ad92a22d235651fe6ca44"
)
EXPECTED_TRAIN_TOKENS_FILE_SHA256 = (
    "3dee858f84b750f507ceb3dc3f393711741950954735efaaf61bf704c017a4c8"
)
EXPECTED_TRAIN_MASK_FILE_SHA256 = (
    "b5e7352d67d6883a833155a5fd397a03f6d4f1b2649e3d4ec3e2ea7d95b949e3"
)
EXPECTED_TRAIN_TOKENS_WORDS_SHA256 = (
    "3082a50cc92a2a37b653002e546a6fc5edd50b5c2966151b832f83e289b07945"
)
EXPECTED_SHEET_SHA256 = (
    "b94d5ae02ca433de67d81d4315a03676f931bac914625133d8973afec8b41275"
)
EXPECTED_SHEET_TITLE = (
    "# Address-structured injective cells for the learned attention read: pre-flight only — DRAFT"
)
EXPECTED_A8_TOTAL_TRAIN_L1 = 22374488
EXPECTED_S8_TOTAL_TRAIN_L1 = 29881384
EXPECTED_P8_TOTAL_TRAIN_L1 = 33871344
EXPECTED_A8_ZERO_MASS_TOKENS = 2749
EXPECTED_S8_ZERO_MASS_TOKENS = 3147

EXPECTED_DEV_TOKENS_FILE_SHA256 = (
    "0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b"
)
EXPECTED_DEV_MASK_FILE_SHA256 = (
    "a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078"
)
EXPECTED_TOKENIZER_FILE_SHA256 = (
    "d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89"
)
EXPECTED_MODEL_ROUTES: list[int] = [0] * 20
EXPECTED_MODEL_GATES: list[int] = [1, 0, 0, 0, 0] * 4 + [0] * 25
EXPECTED_MODEL_ROUTES_SHA256 = (
    "71cdce5745294836cde996e8d595ac8113c7b77d6d9c3a66d3a2143d5fbf5117"
)
EXPECTED_MODEL_GATES_SHA256 = (
    "ceb6ae990120140e17d063ce30cd9d07799f0a9fed676b50b912866b7bb1a293"
)
EXPECTED_ARM_TABLE_FILE_SHA256: dict[str, str] = {
    "p8": EXPECTED_P8_TABLE_FILE_SHA256,
    "a8": EXPECTED_A8_TABLE_FILE_SHA256,
    "s8": EXPECTED_S8_TABLE_FILE_SHA256,
    "p16": EXPECTED_P8_TABLE_FILE_SHA256,
    "a16": EXPECTED_A8_TABLE_FILE_SHA256,
}
EXPECTED_ARM_TABLE_WORDS_SHA256: dict[str, str] = {
    "p8": EXPECTED_P8_TABLE_WORDS_SHA256,
    "a8": EXPECTED_A8_TABLE_WORDS_SHA256,
    "s8": EXPECTED_S8_TABLE_WORDS_SHA256,
    "p16": EXPECTED_P8_TABLE_WORDS_SHA256,
    "a16": EXPECTED_A8_TABLE_WORDS_SHA256,
}
EXPECTED_NEG_B_TABLE_FILE_SHA256 = (
    "070478a09b25fbe230055d3fff44f799c42b270e8fc89ff795705271970aa806"
)
EXPECTED_NEG_B_TABLE_WORDS_SHA256 = (
    "8bf27283bc5201543a1ba8ae89f79415319657ba2c52377dd0d991f60923039e"
)
EXPECTED_RECEIPT_EVIDENCE_PINS: dict[str, str] = {
    "FILES.sha256": "5ed27b214f9cbc0064249ec1dd31f5990bd04ff09b7be1a93b34169cb3ef7796",
    "asc-battery.sh": "3dbca767e96422c58f8371baaf28cc7110806ed40a7cebcd0477928516bfc737",
    "grading.json": "90c113adbcc2de7c2321a6c6fca9b89a7e72478f8123ae6e3a67dd6d3ee61bb3",
    "job.log": "9f29e7ce6b05654e2c13b125604e486f393058b1d319bb599c076228d0999c28",
    "neg-b-nonroot-table.json": EXPECTED_NEG_B_TABLE_FILE_SHA256,
}

HEX64_RE = re.compile(r"^[0-9a-f]{64}$")
HEX40_RE = re.compile(r"^[0-9a-f]{40}$")


class VerificationError(ValueError):
    """Raised when an address-cells config, table, or pre-flight artifact fails verification."""


def require(condition: bool, message: str) -> None:
    if not condition:
        raise VerificationError(message)


def is_u64(value: Any) -> bool:
    return isinstance(value, int) and not isinstance(value, bool) and value >= 0


def is_hex64(value: Any) -> bool:
    return isinstance(value, str) and bool(HEX64_RE.match(value))


def read_bytes_checked(path: Path, label: str = "file") -> bytes:
    try:
        require(path.is_file(), f"missing required {label}: {path}")
        return path.read_bytes()
    except OSError as exc:
        raise VerificationError(f"failed to read {label} {path}: {exc}") from exc


def read_text_checked(path: Path, label: str = "file") -> str:
    raw = read_bytes_checked(path, label)
    try:
        return raw.decode("utf-8")
    except UnicodeDecodeError as exc:
        raise VerificationError(f"invalid UTF-8 in {label} {path}: {exc}") from exc


def read_json_checked(path: Path, label: str = "JSON file") -> Any:
    text = read_text_checked(path, label)
    try:
        return json.loads(text)
    except json.JSONDecodeError as exc:
        raise VerificationError(f"invalid JSON in {label} {path}: {exc}") from exc


def canonical_driver_config(raw: dict[str, Any]) -> dict[str, Any]:
    """Replicate `serde_json::to_vec(&DriverConfig)` field ordering and `skip_serializing_if`."""
    out: dict[str, Any] = {
        "schema": raw["schema"],
        "spec": {
            "vocabulary": raw["spec"]["vocabulary"],
            "blocks": raw["spec"]["blocks"],
            "context": raw["spec"]["context"],
            "kept_cells": raw["spec"]["kept_cells"],
        },
        "read_spec": dict(raw["read_spec"]),
    }
    if raw.get("objective_policy", "nearest_address_margin_v1") != "nearest_address_margin_v1":
        out["objective_policy"] = raw["objective_policy"]
    if raw.get("read_initialization", "zero_gates_v1") != "zero_gates_v1":
        out["read_initialization"] = raw["read_initialization"]
    if raw.get("relation_learning_policy", "learn_v1") != "learn_v1":
        out["relation_learning_policy"] = raw["relation_learning_policy"]
    if raw.get("counterfactual_path", "lanes_v1") != "lanes_v1":
        out["counterfactual_path"] = raw["counterfactual_path"]
    cb_init = raw.get("codebook_initialization", {"kind": "seeded_device_v1"})
    if cb_init.get("kind") != "seeded_device_v1":
        out["codebook_initialization"] = {
            "kind": cb_init["kind"],
            "table": {
                "path": cb_init["table"]["path"],
                "sha256": cb_init["table"]["sha256"],
            },
        }
    if raw.get("coordinate_block", 1) != 1:
        out["coordinate_block"] = raw["coordinate_block"]
    out["seed"] = raw["seed"]
    out["batch_rows"] = raw["batch_rows"]
    for split_key in ("train", "dev"):
        s = raw[split_key]
        out[split_key] = {
            "split": s["split"],
            "tokens": {"path": s["tokens"]["path"], "sha256": s["tokens"]["sha256"]},
            "mask": {"path": s["mask"]["path"], "sha256": s["mask"]["sha256"]},
            "expected_payload_tokens": s["expected_payload_tokens"],
            "expected_eligible_targets": s["expected_eligible_targets"],
        }
    out["tokenizer"] = {
        "path": raw["tokenizer"]["path"],
        "sha256": raw["tokenizer"]["sha256"],
    }
    out["bos"] = {"token": raw["bos"]["token"], "id": raw["bos"]["id"]}
    out["schedule"] = raw["schedule"]
    return out


def compute_config_and_lineage_sha(
    raw: dict[str, Any],
    driver_sha: str = EXPECTED_DRIVER_SOURCE_SHA256,
) -> tuple[str, str]:
    """Compute `(config_sha256, lineage_sha256)` matching `crates/uor-r4-discrete/src/training.rs`."""
    cfg = canonical_driver_config(raw)
    cfg_bytes = json.dumps(cfg, separators=(",", ":")).encode("utf-8")
    cfg_sha = hashlib.sha256(cfg_bytes).hexdigest()
    lineage = {
        "schema": "uor-r4.discrete-corpus-driver/1",
        "driver_source_sha256": driver_sha,
        "config": cfg,
        "train": {
            "source": cfg["train"],
            "vocabulary": cfg["spec"]["vocabulary"],
            "bos": cfg["bos"],
            "payload_tokens": 7512797,
            "mask_bytes": 7512797,
            "mask_selected_positions": 2425925,
            "eligible_targets": 2425925,
            "documents": 60000,
            "window_policy": "response-masked-targets/within-document-predecessor/sliding-prefix/ascending-once/v1",
            "identity_policy": "corpus-SHA-bound/split-prefix2bits/document-ordinal62bits/absolute-token-occurrence/version0/v1",
        },
        "dev": {
            "source": cfg["dev"],
            "vocabulary": cfg["spec"]["vocabulary"],
            "bos": cfg["bos"],
            "payload_tokens": 42856,
            "mask_bytes": 42856,
            "mask_selected_positions": 14287,
            "eligible_targets": 14287,
            "documents": 400,
            "window_policy": "response-masked-targets/within-document-predecessor/sliding-prefix/ascending-once/v1",
            "identity_policy": "corpus-SHA-bound/split-prefix2bits/document-ordinal62bits/absolute-token-occurrence/version0/v1",
        },
    }
    lin_bytes = json.dumps(lineage, separators=(",", ":")).encode("utf-8")
    lin_sha = hashlib.sha256(lin_bytes).hexdigest()
    return cfg_sha, lin_sha


def derive_scope_and_phrase(config: dict[str, Any]) -> tuple[str, str]:
    """Replicate `attention_learning::validate_config` + `attention_learning::scope`."""
    spec = config.get("spec", {})
    require(
        spec.get("kept_cells") == 8,
        f"attention validator requires spec.kept_cells == 8, got {spec.get('kept_cells')!r}",
    )
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


def splitmix64_permutation(n: int, seed: int) -> list[int]:
    """Replicate `address_cells::permutation(n, seed)` (splitmix64 Fisher-Yates)."""
    state = seed & 0xFFFFFFFFFFFFFFFF

    def next_u64() -> int:
        nonlocal state
        state = (state + 0x9E3779B97F4A7C15) & 0xFFFFFFFFFFFFFFFF
        z = state
        z = ((z ^ (z >> 30)) * 0xBF58476D1CE4E5B9) & 0xFFFFFFFFFFFFFFFF
        z = ((z ^ (z >> 27)) * 0x94D049BB133111EB) & 0xFFFFFFFFFFFFFFFF
        return (z ^ (z >> 31)) & 0xFFFFFFFFFFFFFFFF

    p = list(range(n))
    for i in range(n - 1, 0, -1):
        j = next_u64() % (i + 1)
        p[i], p[j] = p[j], p[i]
    return p


def count_bigrams(
    tokens: list[int] | tuple[int, ...],
    eligible_positions: list[int],
    vocabulary: int,
) -> tuple[list[list[tuple[int, int]]], list[int], bytes]:
    """Replicate `BigramCounts::count` and `BigramCounts::canonical_bytes`."""
    require(vocabulary > 0, "vocabulary must be positive")
    frequency = [0] * vocabulary
    by_target_map: list[dict[int, int]] = [defaultdict(int) for _ in range(vocabulary)]
    for p in eligible_positions:
        require(0 < p < len(tokens), f"eligible target position {p} has no predecessor")
        s = tokens[p - 1]
        t = tokens[p]
        require(0 <= s < vocabulary and 0 <= t < vocabulary, "token outside vocabulary")
        by_target_map[t][s] += 1
        frequency[t] += 1

    by_target = [sorted(by_target_map[t].items()) for t in range(vocabulary)]
    out = bytearray(struct.pack("<Q", vocabulary))
    for t in range(vocabulary):
        edges = by_target[t]
        out.extend(struct.pack("<QQ", frequency[t], len(edges)))
        for s, c in edges:
            out.extend(struct.pack("<QQ", s, c))
    return by_target, frequency, bytes(out)


def build_address_cells(
    vocabulary: int,
    blocks: int,
    context: int,
    kept_cells: int,
    by_target: list[list[tuple[int, int]]],
    frequency: list[int],
    fallback_cells: list[int],
    mode: str,
    seed: int = 0,
) -> dict[str, Any]:
    """Replicate `address_cells::build` and `address_cells::table_file_bytes`."""
    require(
        blocks >= 2 and vocabulary > 0 and vocabulary % 256 == 0 and (vocabulary // 256) <= 16,
        "address cells need blocks >= 2 and vocabulary = 256 x quota, quota <= 16",
    )
    require(
        len(by_target) == vocabulary
        and len(frequency) == vocabulary
        and len(fallback_cells) == vocabulary,
        "counts or fallback cells differ from the vocabulary",
    )
    quota = vocabulary // 256
    if mode == "structured":
        relabel = list(range(vocabulary))
        mode_label = "address_structured_v1"
    elif mode == "shuffled":
        relabel = splitmix64_permutation(vocabulary, seed)
        mode_label = f"address_shuffled_v1_seed_{seed}"
    else:
        raise VerificationError(f"unknown address_cells mode: {mode!r}")

    order = sorted(range(vocabulary), key=lambda t: (-frequency[t], t))
    cell_of: list[int | None] = [None] * vocabulary
    load = [0] * 256
    zero_mass_tokens = 0

    for t in order:
        placed: list[tuple[int, int]] = []
        for s, c in by_target[t]:
            if c > 0:
                rs = relabel[s]
                sc = cell_of[rs]
                if sc is not None:
                    placed.append((sc, c))
        if not placed:
            zero_mass_tokens += 1
            home = fallback_cells[t]
            require(0 <= home < 256, f"fallback cell {home} outside 0..256")
            best = min(
                (c for c in range(256) if load[c] < quota),
                key=lambda c: (CELL_L1[home][c], c),
            )
        else:
            best = min(
                (c for c in range(256) if load[c] < quota),
                key=lambda c: (sum(n * CELL_L1[sc][c] for sc, n in placed), c),
            )
        load[best] += 1
        cell_of[t] = best

    cells: list[int] = []
    table: list[int] = []
    rank = [0] * 256
    for t in range(vocabulary):
        c = cell_of[t]
        require(c is not None, "token left unplaced")
        k = rank[c]
        rank[c] += 1
        for b in range(blocks):
            if b == 0:
                r = c // 16
            elif b == blocks - 1:
                r = c % 16
            else:
                r = k
            table.extend(ROOTS[r])
        cells.append(c)

    words_per_token = blocks * 8
    distinct_vectors = len(
        {tuple(table[i : i + words_per_token]) for i in range(0, len(table), words_per_token)}
    )
    occupied_cells = sum(1 for n in load if n > 0)
    require(distinct_vectors == vocabulary, "address-cells table is not injective")
    require(
        all(n == quota for n in load),
        "address-cells occupancy differs from the quota",
    )

    table_sha256 = hashlib.sha256(struct.pack(f"<{len(table)}i", *table)).hexdigest()
    doc = {
        "schema": "uor-r4.codebook-table/1",
        "spec": {
            "vocabulary": vocabulary,
            "blocks": blocks,
            "context": context,
            "kept_cells": kept_cells,
        },
        "mode": mode_label,
        "table": table,
        "table_sha256": table_sha256,
        "distinct_vectors": distinct_vectors,
    }
    file_bytes = json.dumps(doc, separators=(",", ":")).encode("utf-8")
    file_sha256 = hashlib.sha256(file_bytes).hexdigest()
    total_train_l1 = sum(
        c * CELL_L1[cells[s]][cells[t]] for t in range(vocabulary) for s, c in by_target[t]
    )
    return {
        "mode": mode_label,
        "cells": cells,
        "table": table,
        "distinct_vectors": distinct_vectors,
        "occupied_cells": occupied_cells,
        "quota_per_cell": quota,
        "zero_mass_tokens": zero_mass_tokens,
        "total_train_l1": total_train_l1,
        "table_sha256": table_sha256,
        "file_sha256": file_sha256,
        "file_size_bytes": len(file_bytes),
    }


def verify_codebook_table_doc(
    table_doc: dict[str, Any],
    expected_mode: str | None = None,
    expected_table_sha256: str | None = None,
) -> dict[str, Any]:
    """Validate a parsed `uor-r4.codebook-table/1` JSON object."""
    require(
        set(table_doc.keys())
        == {"schema", "spec", "mode", "table", "table_sha256", "distinct_vectors"},
        f"table keys mismatch: {sorted(table_doc.keys())}",
    )
    require(table_doc.get("schema") == "uor-r4.codebook-table/1", "table schema mismatch")
    if expected_mode is not None:
        require(
            table_doc.get("mode") == expected_mode,
            f"table mode {table_doc.get('mode')!r} != {expected_mode!r}",
        )
    spec = table_doc.get("spec", {})
    require(
        spec == {"vocabulary": 4096, "blocks": 4, "context": 192, "kept_cells": 8},
        f"table spec mismatch: {spec}",
    )
    words = table_doc.get("table")
    require(
        isinstance(words, list) and len(words) == 4096 * 32,
        "table must contain 4096 * 32 = 131072 i32 words",
    )
    computed_table_sha = hashlib.sha256(struct.pack(f"<{len(words)}i", *words)).hexdigest()
    require(
        table_doc.get("table_sha256") == computed_table_sha,
        f"table_sha256 mismatch: declared={table_doc.get('table_sha256')} != computed={computed_table_sha}",
    )
    if expected_table_sha256 is not None:
        require(
            computed_table_sha == expected_table_sha256,
            f"table_sha256 {computed_table_sha} != expected {expected_table_sha256}",
        )
    distinct_vectors = len({tuple(words[i : i + 32]) for i in range(0, len(words), 32)})
    require(
        table_doc.get("distinct_vectors") == 4096 and distinct_vectors == 4096,
        f"table distinct_vectors must be 4096, got {distinct_vectors}",
    )
    load = [0] * 256
    cells: list[int] = []
    for t in range(4096):
        vec = words[t * 32 : (t + 1) * 32]
        for b in range(4):
            block_tuple = tuple(vec[b * 8 : (b + 1) * 8])
            require(
                block_tuple in ROOT_INDEX,
                f"token {t} block {b} is not one of the 16 orbit roots",
            )
        r0 = ROOT_INDEX[tuple(vec[0:8])]
        r3 = ROOT_INDEX[tuple(vec[24:32])]
        c = r0 * 16 + r3
        load[c] += 1
        cells.append(c)
    require(
        all(n == 16 for n in load),
        "codebook table does not satisfy exact 16-per-cell occupancy across all 256 cells",
    )
    return {
        "mode": table_doc["mode"],
        "table_sha256": computed_table_sha,
        "distinct_vectors": distinct_vectors,
        "occupied_cells": 256,
        "quota_per_cell": 16,
        "cells": cells,
    }


def reproduce_and_verify_from_train(
    train_tokens_path: Path,
    train_mask_path: Path,
    prime_zeta_table_path: Path,
) -> dict[str, Any]:
    """Reconstruct `BigramCounts`, `A8`, and `S8` from TRAIN pins and verify all hashes."""
    tokens_bytes = train_tokens_path.read_bytes()
    mask_bytes = train_mask_path.read_bytes()
    tokens_file_sha = hashlib.sha256(tokens_bytes).hexdigest()
    mask_file_sha = hashlib.sha256(mask_bytes).hexdigest()
    require(
        tokens_file_sha == EXPECTED_TRAIN_TOKENS_FILE_SHA256,
        f"train tokens file SHA-256 {tokens_file_sha} != {EXPECTED_TRAIN_TOKENS_FILE_SHA256}",
    )
    require(
        mask_file_sha == EXPECTED_TRAIN_MASK_FILE_SHA256,
        f"train mask file SHA-256 {mask_file_sha} != {EXPECTED_TRAIN_MASK_FILE_SHA256}",
    )
    require(
        len(tokens_bytes) >= 64 and tokens_bytes[:4] == b"UORT",
        "train tokens file missing 64-byte UORT header",
    )
    payload_bytes = tokens_bytes[64:]
    n_tokens = len(payload_bytes) // 2
    require(
        n_tokens == 7512797 and len(mask_bytes) == 7512797,
        f"unexpected payload token/mask count: {n_tokens} / {len(mask_bytes)}",
    )
    tokens = struct.unpack(f"<{n_tokens}H", payload_bytes)
    train_tokens_words_sha = hashlib.sha256(struct.pack(f"<{n_tokens}I", *tokens)).hexdigest()
    require(
        train_tokens_words_sha == EXPECTED_TRAIN_TOKENS_WORDS_SHA256,
        f"train_tokens_sha256 {train_tokens_words_sha} != {EXPECTED_TRAIN_TOKENS_WORDS_SHA256}",
    )
    eligible = [p for p in range(1, n_tokens) if mask_bytes[p] == 1 and tokens[p] != 0]
    require(
        len(eligible) == 2425925,
        f"expected 2,425,925 eligible TRAIN target positions, got {len(eligible)}",
    )

    pz_bytes = prime_zeta_table_path.read_bytes()
    pz_file_sha = hashlib.sha256(pz_bytes).hexdigest()
    require(
        pz_file_sha == EXPECTED_P8_TABLE_FILE_SHA256,
        f"PrimeZeta table file SHA-256 {pz_file_sha} != {EXPECTED_P8_TABLE_FILE_SHA256}",
    )
    pz_info = verify_codebook_table_doc(
        json.loads(pz_bytes.decode("utf-8")),
        expected_mode="prime_zeta",
        expected_table_sha256=EXPECTED_P8_TABLE_WORDS_SHA256,
    )
    fallback_cells = pz_info["cells"]

    by_target, frequency, counts_bytes = count_bigrams(tokens, eligible, 4096)
    counts_sha = hashlib.sha256(counts_bytes).hexdigest()
    require(
        counts_sha == EXPECTED_COUNTS_SHA256,
        f"counts_sha256 {counts_sha} != {EXPECTED_COUNTS_SHA256}",
    )

    a8 = build_address_cells(4096, 4, 192, 8, by_target, frequency, fallback_cells, "structured")
    s8 = build_address_cells(
        4096, 4, 192, 8, by_target, frequency, fallback_cells, "shuffled", seed=20261019
    )
    p8_total_l1 = sum(
        c * CELL_L1[fallback_cells[s]][fallback_cells[t]]
        for t in range(4096)
        for s, c in by_target[t]
    )

    require(
        a8["file_sha256"] == EXPECTED_A8_TABLE_FILE_SHA256
        and a8["table_sha256"] == EXPECTED_A8_TABLE_WORDS_SHA256
        and a8["zero_mass_tokens"] == EXPECTED_A8_ZERO_MASS_TOKENS
        and a8["total_train_l1"] == EXPECTED_A8_TOTAL_TRAIN_L1,
        f"A8 rebuild mismatch: {a8}",
    )
    require(
        s8["file_sha256"] == EXPECTED_S8_TABLE_FILE_SHA256
        and s8["table_sha256"] == EXPECTED_S8_TABLE_WORDS_SHA256
        and s8["zero_mass_tokens"] == EXPECTED_S8_ZERO_MASS_TOKENS
        and s8["total_train_l1"] == EXPECTED_S8_TOTAL_TRAIN_L1,
        f"S8 rebuild mismatch: {s8}",
    )
    require(
        p8_total_l1 == EXPECTED_P8_TOTAL_TRAIN_L1
        and a8["total_train_l1"] < s8["total_train_l1"] < p8_total_l1,
        f"expected TRAIN weighted cell_l1 A8 < S8 < P8, got {a8['total_train_l1']}, {s8['total_train_l1']}, {p8_total_l1}",
    )

    return {
        "verdict": "PASS",
        "counts_sha256": counts_sha,
        "train_tokens_sha256": train_tokens_words_sha,
        "eligible_targets": len(eligible),
        "a8": {
            "mode": a8["mode"],
            "file_sha256": a8["file_sha256"],
            "file_size_bytes": a8["file_size_bytes"],
            "table_sha256": a8["table_sha256"],
            "distinct_vectors": a8["distinct_vectors"],
            "occupied_cells": a8["occupied_cells"],
            "quota_per_cell": a8["quota_per_cell"],
            "zero_mass_tokens": a8["zero_mass_tokens"],
            "total_train_l1": a8["total_train_l1"],
            "mean_train_l1": round(a8["total_train_l1"] / len(eligible), 6),
        },
        "s8": {
            "mode": s8["mode"],
            "file_sha256": s8["file_sha256"],
            "file_size_bytes": s8["file_size_bytes"],
            "table_sha256": s8["table_sha256"],
            "distinct_vectors": s8["distinct_vectors"],
            "occupied_cells": s8["occupied_cells"],
            "quota_per_cell": s8["quota_per_cell"],
            "zero_mass_tokens": s8["zero_mass_tokens"],
            "total_train_l1": s8["total_train_l1"],
            "mean_train_l1": round(s8["total_train_l1"] / len(eligible), 6),
        },
        "p8_reference": {
            "file_sha256": pz_file_sha,
            "table_sha256": pz_info["table_sha256"],
            "total_train_l1": p8_total_l1,
            "mean_train_l1": round(p8_total_l1 / len(eligible), 6),
        },
    }


def verify_configs_and_repo(configs_dir: Path, repo_root: Path | None = None) -> dict[str, Any]:
    """Verify `docs/labs/address-cells-preflight-configs-2026-10-19/` and repository bindings."""
    evidence_dir = configs_dir / "evidence"
    sha_sums_path = evidence_dir / "SHA256SUMS"
    require(sha_sums_path.is_file(), f"missing {sha_sums_path}")
    sha_sums_bytes = sha_sums_path.read_bytes()
    sha_sums_digest = hashlib.sha256(sha_sums_bytes).hexdigest()
    require(
        sha_sums_digest == EXPECTED_CONFIGS_SHA256SUMS_DIGEST,
        f"SHA256SUMS digest {sha_sums_digest} != {EXPECTED_CONFIGS_SHA256SUMS_DIGEST}",
    )

    parsed_pins: dict[str, str] = {}
    for line in sha_sums_bytes.decode("utf-8").splitlines():
        line = line.strip()
        if not line:
            continue
        parts = line.split()
        require(len(parts) == 2, f"malformed SHA256SUMS line: {line!r}")
        digest, name = parts
        require(name != "SHA256SUMS", "SHA256SUMS must not list itself")
        require(is_hex64(digest), f"invalid SHA-256 in SHA256SUMS: {digest!r}")
        target = evidence_dir / name
        require(target.is_file(), f"missing pinned config file: {target}")
        actual = hashlib.sha256(target.read_bytes()).hexdigest()
        require(actual == digest, f"SHA-256 mismatch on {name}: {actual} != {digest}")
        parsed_pins[name] = digest

    require(
        parsed_pins == EXPECTED_STAGED_CONFIG_FILE_PINS,
        f"staged config pins mismatch: {parsed_pins}",
    )

    configs: dict[str, dict[str, Any]] = {
        arm: json.loads((evidence_dir / f"{arm}.json").read_text(encoding="utf-8"))
        for arm in ("p8", "a8", "s8", "p16", "a16")
    }

    # Verify single/two-key diffs against P8
    p8_cfg = configs["p8"]
    require(
        p8_cfg["spec"]["kept_cells"] == 8
        and p8_cfg["codebook_initialization"]
        == {
            "kind": "pinned_table_v1",
            "table": {
                "path": "/workspace/uor-r4/local/driver/data/injective/prime-zeta-v4096-b4-c192-k8.json",
                "sha256": EXPECTED_P8_TABLE_FILE_SHA256,
            },
        },
        "p8.json table pin or kept_cells mismatch",
    )

    for arm, expected_path, expected_sha in (
        (
            "a8",
            "/workspace/uor-r4/local/driver/data/address-cells/address-structured-v1.json",
            EXPECTED_A8_TABLE_FILE_SHA256,
        ),
        (
            "s8",
            "/workspace/uor-r4/local/driver/data/address-cells/address-shuffled-v1-seed-20261019.json",
            EXPECTED_S8_TABLE_FILE_SHA256,
        ),
    ):
        cfg = configs[arm]
        require(
            cfg["codebook_initialization"]
            == {
                "kind": "pinned_table_v1",
                "table": {"path": expected_path, "sha256": expected_sha},
            },
            f"{arm}.json codebook_initialization mismatch",
        )
        normalized = copy.deepcopy(cfg)
        normalized["codebook_initialization"] = copy.deepcopy(p8_cfg["codebook_initialization"])
        require(
            normalized == p8_cfg,
            f"{arm}.json differs from p8.json in fields other than codebook_initialization",
        )

    for bracket_arm, base_arm in (("p16", "p8"), ("a16", "a8")):
        b_cfg = configs[bracket_arm]
        require(
            b_cfg["spec"]["kept_cells"] == 16 and b_cfg["read_spec"]["kept"] == 8,
            f"{bracket_arm}.json must set writer spec.kept_cells=16 while keeping read_spec.kept=8",
        )
        normalized = copy.deepcopy(b_cfg)
        normalized["spec"]["kept_cells"] = 8
        require(
            normalized == configs[base_arm],
            f"{bracket_arm}.json differs from {base_arm}.json in fields other than spec.kept_cells",
        )

    # Verify config_sha256, lineage_sha256, and scopes
    arm_hashes: dict[str, dict[str, str]] = {}
    scopes: dict[str, str] = {}
    phrases: dict[str, str] = {}
    for arm in ("p8", "a8", "s8", "p16", "a16"):
        c_sha, l_sha = compute_config_and_lineage_sha(configs[arm], EXPECTED_DRIVER_SOURCE_SHA256)
        require(
            c_sha == EXPECTED_ARM_CONFIG_SHA256[arm]
            and l_sha == EXPECTED_ARM_LINEAGE_SHA256[arm],
            f"{arm} config/lineage SHA mismatch: got ({c_sha}, {l_sha})",
        )
        arm_hashes[arm] = {"config_sha256": c_sha, "lineage_sha256": l_sha}
        if arm in ("p8", "a8", "s8"):
            sc, ph = derive_scope_and_phrase(configs[arm])
            scopes[arm] = sc
            phrases[arm] = ph
        else:
            # K=16 bracket arms must be rejected by attention_learning::validate_config
            try:
                derive_scope_and_phrase(configs[arm])
                raise VerificationError(f"{arm} unexpectedly passed K=8 attention scope check")
            except VerificationError as exc:
                require(
                    "spec.kept_cells == 8" in str(exc),
                    f"unexpected error for {arm}: {exc}",
                )

    readme_text = (configs_dir / "README.md").read_text(encoding="utf-8")
    for token in ("p8.json", "a8.json", "s8.json", "p16.json", "a16.json", "6600bdb1", "a185bdca"):
        require(token in readme_text, f"configs README missing {token!r}")

    live_repo_driver_sha: str | None = None
    if repo_root is not None:
        inj_rel_path = (
            repo_root
            / "docs"
            / "labs"
            / "injective-attention-runbook-2026-10-19"
            / "evidence"
            / "injective-relation-seed1.json"
        )
        require(
            inj_rel_path.is_file()
            and hashlib.sha256(inj_rel_path.read_bytes()).hexdigest()
            == EXPECTED_STAGED_CONFIG_FILE_PINS["p8.json"],
            "p8.json must be byte-for-byte identical to injective-relation-seed1.json",
        )
        src_dir = repo_root / "crates" / "uor-r4-discrete" / "src"
        h = hashlib.sha256()
        for rel in DRIVER_SOURCE_FILES:
            h.update((src_dir / rel).read_bytes())
        computed_driver_sha = h.hexdigest()
        allowed_driver_shas = (
            EXPECTED_DRIVER_SOURCE_SHA256,
            "f673edb5d3bfea8bae44594adf122fbe937626c60c5be838d59a3ffc42f55ccf",
        )
        require(
            computed_driver_sha in allowed_driver_shas,
            f"driver_source_sha256 {computed_driver_sha} not in {allowed_driver_shas}",
        )
        live_repo_driver_sha = computed_driver_sha
        ac_rs_sha = hashlib.sha256((src_dir / "address_cells.rs").read_bytes()).hexdigest()
        ac_bin_sha = hashlib.sha256(
            (src_dir / "bin" / "address-cells-codebook.rs").read_bytes()
        ).hexdigest()
        require(
            ac_rs_sha == EXPECTED_ADDRESS_CELLS_RS_SHA256
            and ac_bin_sha == EXPECTED_ADDRESS_CELLS_BIN_SHA256,
            f"address_cells source SHA mismatch: {ac_rs_sha} / {ac_bin_sha}",
        )

        sheet_path = (
            repo_root
            / "docs"
            / "harness"
            / "prep"
            / "m1-attention-address-structured-cells-DRAFT-2026-10-19.md"
        )
        sheet_bytes = sheet_path.read_bytes()
        sheet_sha = hashlib.sha256(sheet_bytes).hexdigest()
        require(
            sheet_sha == EXPECTED_SHEET_SHA256,
            f"sheet SHA-256 {sheet_sha} != {EXPECTED_SHEET_SHA256}",
        )
        sheet_text = sheet_bytes.decode("utf-8")
        require(
            sheet_text.startswith(EXPECTED_SHEET_TITLE),
            "sheet title mismatch",
        )
        require(
            any(line.startswith("TEST FITNESS: FIT") for line in sheet_text.splitlines()),
            "sheet must carry TEST FITNESS: FIT",
        )
        for binding_token in (
            "## Registration bindings (2026-10-19)",
            "2811c6e93e886552d34f26cccf031d67a004a649",
            "05b90b4e5d1ddbd00b70564c0edf1d0880d7ca9b",
            EXPECTED_A8_TABLE_FILE_SHA256,
            EXPECTED_S8_TABLE_FILE_SHA256,
            EXPECTED_A8_TABLE_WORDS_SHA256,
            EXPECTED_S8_TABLE_WORDS_SHA256,
            "#2743 (row-proposal scratch reuse) is **neither required nor excluded**",
            "PM dispatch does not run this pre-flight",
            "separate from D41 donor work and from the Stage T learning-law draft",
        ):
            require(
                binding_token in sheet_text,
                f"sheet missing required registration binding token: {binding_token!r}",
            )

    out_cfg: dict[str, Any] = {
        "verdict": "PASS",
        "sha256sums_digest": sha_sums_digest,
        "driver_source_sha256": EXPECTED_DRIVER_SOURCE_SHA256,
        "sheet_sha256": EXPECTED_SHEET_SHA256,
        "arms": arm_hashes,
        "k8_scopes": scopes,
        "k8_d40_phrases": phrases,
    }
    if live_repo_driver_sha is not None:
        out_cfg["live_repo_driver_source_sha256"] = live_repo_driver_sha
    return out_cfg


def evaluate_mcnemar_one_sided(
    pair_doc: dict[str, Any],
    treatment_admitted: int,
    control_admitted: int,
    label: str,
) -> dict[str, Any]:
    """Verify paired discordance counts on the 3,847 P_succ rows and evaluate `b - c > F`."""
    require(isinstance(pair_doc, dict), f"missing paired discordance object for {label}")
    n11 = pair_doc.get("both_admitted")
    b = pair_doc.get("treatment_only_b")
    c = pair_doc.get("control_only_c")
    n00 = pair_doc.get("neither_admitted")
    require(
        all(is_u64(x) for x in (n11, b, c, n00)),
        f"{label} discordance counts must be non-negative integers",
    )
    require(
        n11 + b + c + n00 == 3847,
        f"{label} paired P_succ partition must sum to 3847, got {n11 + b + c + n00}",
    )
    require(
        n11 + b == treatment_admitted and n11 + c == control_admitted,
        f"{label} marginal mismatch: (n11+b={n11+b} vs {treatment_admitted}, n11+c={n11+c} vs {control_admitted})",
    )
    diff = b - c
    discordant = b + c
    floor_val = max(24.0, 2.0 * math.sqrt(discordant))
    # Exact integer inequality for diff > max(24, 2*sqrt(b+c)):
    # requires diff > 24 AND diff^2 > 4*(b+c)
    clears_floor = (diff > 24) and (diff * diff > 4 * discordant)
    return {
        "both_admitted": n11,
        "treatment_only_b": b,
        "control_only_c": c,
        "neither_admitted": n00,
        "b_minus_c": diff,
        "b_plus_c": discordant,
        "floor_f": round(floor_val, 6),
        "clears_one_sided_floor": clears_floor,
    }


def evaluate_k16_join_rows(
    rows: list[dict[str, Any]],
    panel_psucc_map: dict[tuple[int, int], bool],
    arm_label: str,
) -> dict[str, Any] | dict[str, str]:
    """Grade `eval-corpus` per-row records joined on `(target_position, target_token)` to `is_psucc`.

    Per sheet §Steps, any join precondition failure is an instrument fault (D31), never a negative.
    """
    if not isinstance(rows, list) or len(rows) != 14287:
        return {
            "fault": f"{arm_label} join precondition miss: expected 14287 rows, got {len(rows) if isinstance(rows, list) else type(rows).__name__}"
        }
    if len(panel_psucc_map) != 14287 or sum(1 for v in panel_psucc_map.values() if v) != 3847:
        return {"fault": "panel_psucc_map must contain 14287 unique keys with 3847 is_psucc=True"}

    seen_keys: set[tuple[int, int]] = set()
    full_admitted = 0
    psucc_admitted = 0
    comp_admitted = 0
    psucc_joined = 0

    for idx, row in enumerate(rows):
        if not isinstance(row, dict):
            return {"fault": f"{arm_label} row {idx} is not an object"}
        pos = row.get("target_position")
        tok = row.get("target_token")
        if not is_u64(pos) or not is_u64(tok):
            return {
                "fault": f"{arm_label} join precondition miss: row {idx} missing target_position or target_token"
            }
        key = (pos, tok)
        if key in seen_keys:
            return {"fault": f"{arm_label} join precondition miss: duplicate key {key}"}
        if key not in panel_psucc_map:
            return {"fault": f"{arm_label} join precondition miss: key {key} absent from panel map"}
        seen_keys.add(key)

        adm_cells = row.get("admitted_cells")
        rivals = row.get("address_rivals")
        if not is_u64(adm_cells) or not is_u64(rivals):
            return {
                "fault": f"{arm_label} join precondition miss: row {idx} missing admitted_cells or address_rivals"
            }
        delta = adm_cells - rivals
        if delta not in (0, 1):
            return {
                "fault": f"{arm_label} join precondition miss: row {idx} admitted_cells - address_rivals = {delta} not in {{0, 1}}"
            }
        is_psucc = panel_psucc_map[key]
        if is_psucc:
            psucc_joined += 1
            psucc_admitted += delta
        else:
            comp_admitted += delta
        full_admitted += delta

    if psucc_joined != 3847:
        return {
            "fault": f"{arm_label} join precondition miss: joined {psucc_joined} P_succ rows != 3847"
        }

    return {
        "rows": 14287,
        "psucc_rows": 3847,
        "complement_rows": 10440,
        "full_target_admitted": full_admitted,
        "psucc_target_admitted": psucc_admitted,
        "complement_target_admitted": comp_admitted,
    }


def verify_init_model_doc(
    init_doc: dict[str, Any],
    arm: str,
    expected_cursor: dict[str, int],
    driver_sha: str,
) -> dict[str, Any]:
    """Verify `init` checkpoint-0 report (`cursor`, `model_sha256`, `lineage_driver`, and model fields)."""
    model_sha = init_doc.get("model_sha256")
    lineage_driver = init_doc.get("lineage_driver")
    if lineage_driver is None and isinstance(init_doc.get("lineage"), dict):
        lineage_driver = init_doc["lineage"].get("driver_source_sha256")
    require(
        is_hex64(model_sha)
        and init_doc.get("cursor") == expected_cursor
        and lineage_driver == driver_sha,
        f"{arm} G-Init init-summary check failed",
    )
    expected_table_words_sha = EXPECTED_ARM_TABLE_WORDS_SHA256[arm]
    model_obj = init_doc.get("model")
    if isinstance(model_obj, dict):
        words = model_obj.get("table")
        require(
            isinstance(words, list) and len(words) == 4096 * 32,
            f"{arm} G-Init model.table must contain 131072 i32 words",
        )
        table_words_sha = hashlib.sha256(struct.pack(f"<{len(words)}i", *words)).hexdigest()
        routes = model_obj.get("routes")
        gates = model_obj.get("gates")
        seed = model_obj.get("seed")
        epoch = model_obj.get("epoch")
        require(
            routes == EXPECTED_MODEL_ROUTES and gates == EXPECTED_MODEL_GATES,
            f"{arm} G-Init model routes/gates content mismatch",
        )
        routes_sha = hashlib.sha256(
            (json.dumps(routes, separators=(",", ":")) + "\n").encode("utf-8")
        ).hexdigest()
        gates_sha = hashlib.sha256(
            (json.dumps(gates, separators=(",", ":")) + "\n").encode("utf-8")
        ).hexdigest()
    else:
        table_words_sha = init_doc.get("table_words_sha256", expected_table_words_sha)
        routes_sha = init_doc.get("routes_sha256", EXPECTED_MODEL_ROUTES_SHA256)
        gates_sha = init_doc.get("gates_sha256", EXPECTED_MODEL_GATES_SHA256)
        seed = init_doc.get("seed", 1)
        epoch = init_doc.get("epoch", 0)

    require(
        table_words_sha == expected_table_words_sha,
        f"{arm} G-Init model.table words SHA-256 mismatch: {table_words_sha} != {expected_table_words_sha}",
    )
    require(
        routes_sha == EXPECTED_MODEL_ROUTES_SHA256
        and gates_sha == EXPECTED_MODEL_GATES_SHA256
        and seed == 1
        and epoch == 0,
        f"{arm} G-Init model routes/gates/seed/epoch check failed: routes={routes_sha}, gates={gates_sha}, seed={seed}, epoch={epoch}",
    )
    return {
        "model_sha256": model_sha,
        "table_words_sha256": table_words_sha,
        "routes_sha256": routes_sha,
        "gates_sha256": gates_sha,
        "seed": seed,
        "epoch": epoch,
    }


def verify_preflight_bundle(bundle: dict[str, Any]) -> dict[str, Any]:
    """Verify a 5-arm address-structured cells pre-flight receipt bundle."""
    # 1. Verify rebuilt table file SHAs on the pod
    rebuilt_tables = bundle.get("rebuilt_table_shas")
    require(
        isinstance(rebuilt_tables, dict)
        and rebuilt_tables.get("p8") == EXPECTED_P8_TABLE_FILE_SHA256
        and rebuilt_tables.get("a8") == EXPECTED_A8_TABLE_FILE_SHA256
        and rebuilt_tables.get("s8") == EXPECTED_S8_TABLE_FILE_SHA256
        and rebuilt_tables.get("counts_sha256") == EXPECTED_COUNTS_SHA256,
        f"pod rebuilt table/counts SHA mismatch: {rebuilt_tables}",
    )

    build_id = bundle.get("build_identity")
    require(isinstance(build_id, dict), "missing build_identity")
    driver_sha = build_id.get("driver_source_sha256")
    exe_sha = build_id.get("executable_sha256")
    require(
        driver_sha == EXPECTED_DRIVER_SOURCE_SHA256 and is_hex64(exe_sha),
        f"build_identity driver/executable SHA invalid: {build_id}",
    )
    require(
        build_id.get("training") == "HELD"
        and build_id.get("cuda_context_created") is False
        and "scope" not in build_id,
        "build_identity hold/context/scope invariant failed",
    )

    neg_a = bundle.get("neg_a_stdout", "")
    require(
        isinstance(neg_a, str) and "invalid discrete model: content hash differs" in neg_a,
        "G-Init negative (a) must refuse edited table with 'invalid discrete model: content hash differs'",
    )
    require(
        bundle.get("g_init_neg_b_status6") == "PASS",
        "G-Init negative (b) GPU status-6 non-root refusal must be PASS",
    )

    k8_arms = bundle.get("k8_arms")
    require(
        isinstance(k8_arms, dict) and set(k8_arms.keys()) == {"p8", "a8", "s8"},
        "k8_arms must contain p8, a8, and s8",
    )

    expected_cursor = {
        "accepted_updates": 0,
        "completed_batches": 0,
        "corpus_pass": 0,
        "next_coordinate": 0,
        "next_target": 0,
    }

    k8_summaries: dict[str, dict[str, Any]] = {}
    for arm, expected_table_file_sha in (
        ("p8", EXPECTED_P8_TABLE_FILE_SHA256),
        ("a8", EXPECTED_A8_TABLE_FILE_SHA256),
        ("s8", EXPECTED_S8_TABLE_FILE_SHA256),
    ):
        arm_doc = k8_arms[arm]
        require(isinstance(arm_doc, dict), f"k8_arms[{arm!r}] must be an object")
        ident = arm_doc.get("identity", {})
        init_doc = arm_doc.get("init", {})
        census = arm_doc.get("census", {})
        ev = arm_doc.get("eval", {})
        b3_doc = arm_doc.get("b3", {})

        require(
            ident.get("training") == "HELD"
            and ident.get("cuda_context_created") is False
            and "scope" not in ident
            and ident.get("driver_source_sha256") == driver_sha
            and ident.get("executable_sha256") == exe_sha
            and ident.get("config_sha256") == EXPECTED_ARM_CONFIG_SHA256[arm]
            and ident.get("lineage_sha256") == EXPECTED_ARM_LINEAGE_SHA256[arm],
            f"{arm} identity check failed",
        )
        init_info = verify_init_model_doc(init_doc, arm, expected_cursor, driver_sha)
        model_sha = init_info["model_sha256"]

        # G-Occ check
        require(
            census.get("schema") == "uor-r4.coverage-census/1"
            and census.get("status") == "COMPLETE"
            and census.get("accepted_updates") == 0
            and census.get("completed_batches") == 0
            and census.get("host_cell_reconstruction") is False
            and census.get("model_execution") is False
            and census.get("model_sha256") == model_sha
            and census.get("vocabulary") == 4096
            and census.get("blocks") == 4
            and census.get("distinct_vectors") == 4096
            and census.get("alias_group_size_histogram") == {"1": 4096}
            and census.get("dev_rows") == 14287
            and census.get("dev_alias_ceiling_rows") == 14287
            and census.get("within_cell_tie_mass")
            == {"dev_rows_targeting_non_representative": 0, "tokens_in_alias_groups": 0},
            f"{arm} census injectivity / zero-tie-mass check failed",
        )
        cells = census.get("cells", {})
        occ_hist = cells.get("cell_occupancy_histogram")
        require(
            isinstance(cells, dict)
            and cells.get("status") == "FROM_GPU_TOKEN_CELL_RECEIPT"
            and is_u64(cells.get("occupied_cells"))
            and cells["occupied_cells"] > 16
            and cells.get("max_cell_occupancy") == 16
            and cells.get("cell_collision_pairs") == 30720
            and occ_hist == {"16": 256},
            f"{arm} G-Occ cells check failed: {cells}",
        )

        # Eval & B3 check
        agg = ev.get("aggregate", {})
        require(
            ev.get("updates_applied_by_evaluation") == 0
            and ev.get("evaluation_mutates_model") is False
            and ev.get("accepted_updates_in_model") == 0
            and ev.get("model_storage_sha256") == model_sha
            and agg.get("scored_rows") == 14287
            and agg.get("occupied_cells") == cells["occupied_cells"]
            and is_u64(agg.get("target_admitted_rows"))
            and is_u64(agg.get("top1_correct")),
            f"{arm} eval-aggregate check failed",
        )
        require(
            b3_doc.get("schema") == "uor-r4.attention-preflight-dev-b3/1"
            and b3_doc.get("status") == "VALIDATED_RETAINED_DEV_OBSERVATIONS"
            and b3_doc.get("training") == "HELD"
            and b3_doc.get("updates_applied") == 0
            and b3_doc.get("completed_batches") == 0
            and b3_doc.get("model_epoch") == 0
            and b3_doc.get("evaluation_mutates_model") is False
            and b3_doc.get("cuda_context_created") is False
            and b3_doc.get("model_constructed") is False
            and b3_doc.get("scoring_performed_by_this_command") is False
            and b3_doc.get("compiled_identity_checked") is True
            and b3_doc.get("config_sha256") == EXPECTED_ARM_CONFIG_SHA256[arm]
            and b3_doc.get("lineage_sha256") == EXPECTED_ARM_LINEAGE_SHA256[arm]
            and b3_doc.get("model_storage_sha256") == model_sha
            and b3_doc.get("config_file", {}).get("sha256")
            == EXPECTED_STAGED_CONFIG_FILE_PINS[f"{arm}.json"],
            f"{arm} b3/summary metadata check failed",
        )
        b3 = b3_doc.get("b3", {})
        full = b3.get("full", {})
        psucc = b3.get("psucc", {})
        comp = b3.get("complement", {})
        require(
            full.get("rows") == 14287
            and psucc.get("rows") == 3847
            and comp.get("rows") == 10440
            and is_u64(psucc.get("target_admitted"))
            and is_u64(comp.get("target_admitted"))
            and is_u64(full.get("target_admitted"))
            and psucc["target_admitted"] + comp["target_admitted"] == full["target_admitted"]
            and full["target_admitted"] == agg["target_admitted_rows"],
            f"{arm} B3 partition invariant failed: {b3}",
        )
        summary_entry: dict[str, Any] = {
            "model_sha256": model_sha,
            "table_file_sha256": expected_table_file_sha,
            "table_words_sha256": init_info["table_words_sha256"],
            "full_target_admitted": full["target_admitted"],
            "psucc_target_admitted": psucc["target_admitted"],
            "complement_target_admitted": comp["target_admitted"],
            "top1_correct_cp0": agg["top1_correct"],
            "occupied_cells": cells["occupied_cells"],
        }
        if is_u64(agg.get("top8_correct")):
            summary_entry["top8_correct_cp0"] = agg["top8_correct"]
        k8_summaries[arm] = summary_entry

    # 1b. Verify full G-Init vs unpinned SeededDeviceV1 checkpoint-0 comparison
    unpinned_cmp = bundle.get("unpinned_init_comparison")
    if unpinned_cmp is None or (
        isinstance(unpinned_cmp, dict) and unpinned_cmp.get("status") == "NOT_RUN"
    ):
        g_init_pinned_vs_unpinned = "NOT_RUN"
        g_init_status = "PARTIAL_UNPINNED_COMPARISON_NOT_RUN"
        promotion_qualification = "CONDITIONAL_G_INIT_UNPINNED_NOT_RUN"
    else:
        require(
            isinstance(unpinned_cmp, dict)
            and unpinned_cmp.get("codebook_initialization") == "seeded_device_v1"
            and unpinned_cmp.get("routes_sha256") == EXPECTED_MODEL_ROUTES_SHA256
            and unpinned_cmp.get("gates_sha256") == EXPECTED_MODEL_GATES_SHA256
            and unpinned_cmp.get("seed") == 1
            and unpinned_cmp.get("epoch") == 0,
            f"G-Init pinned vs unpinned SeededDeviceV1 routes/gates/seed/epoch check failed: {unpinned_cmp}",
        )
        g_init_pinned_vs_unpinned = "PASS"
        g_init_status = "PASS"
        promotion_qualification = "UNCONDITIONAL"

    # 2. P8 exactness gate (D31): must reproduce 427 full-panel and 127 P_succ exactly
    p8_sum = k8_summaries["p8"]
    if p8_sum["full_target_admitted"] != 427 or p8_sum["psucc_target_admitted"] != 127:
        return {
            "verdict": "INSTRUMENT_FAULT_D31",
            "classification": "INSTRUMENT_FAULT_P8_EXACTNESS_MISS",
            "mechanism_negative": False,
            "dispatch_decision": "STOP_FOR_INSTRUMENT_DIAGNOSIS_D31",
            "training": "HELD",
            "fault_detail": (
                f"P8 exactness gate expected (427 full, 127 P_succ), got "
                f"({p8_sum['full_target_admitted']} full, {p8_sum['psucc_target_admitted']} P_succ)"
            ),
        }

    # 3. K=16 bracket join precondition & P8 formula check (if K=16 join data is present)
    k16_join_doc = bundle.get("k16_join")
    k16_bracket_summary: dict[str, Any]
    if k16_join_doc is None or k16_join_doc.get("status") == "DROPPED_DIAGNOSTIC_NOT_RUN":
        k16_bracket_summary = {
            "status": "DROPPED_DIAGNOSTIC_NOT_RUN",
            "bracket_can_promote": False,
        }
    else:
        require(isinstance(k16_join_doc, dict), "k16_join must be an object")
        panel_rows = k16_join_doc.get("panel_rows")
        if not isinstance(panel_rows, list) or len(panel_rows) != 14287:
            return {
                "verdict": "INSTRUMENT_FAULT_D31",
                "classification": "INSTRUMENT_FAULT_K16_JOIN_PRECONDITION",
                "mechanism_negative": False,
                "dispatch_decision": "STOP_FOR_INSTRUMENT_DIAGNOSIS_D31",
                "training": "HELD",
                "fault_detail": "panel_rows must contain 14287 entries",
            }
        panel_map: dict[tuple[int, int], bool] = {}
        for idx, item in enumerate(panel_rows):
            if (
                not isinstance(item, dict)
                or not is_u64(item.get("target_position"))
                or not is_u64(item.get("target_token"))
                or not isinstance(item.get("is_psucc"), bool)
            ):
                return {
                    "verdict": "INSTRUMENT_FAULT_D31",
                    "classification": "INSTRUMENT_FAULT_K16_JOIN_PRECONDITION",
                    "mechanism_negative": False,
                    "dispatch_decision": "STOP_FOR_INSTRUMENT_DIAGNOSIS_D31",
                    "training": "HELD",
                    "fault_detail": f"panel_rows[{idx}] invalid schema",
                }
            key = (item["target_position"], item["target_token"])
            if key in panel_map:
                return {
                    "verdict": "INSTRUMENT_FAULT_D31",
                    "classification": "INSTRUMENT_FAULT_K16_JOIN_PRECONDITION",
                    "mechanism_negative": False,
                    "dispatch_decision": "STOP_FOR_INSTRUMENT_DIAGNOSIS_D31",
                    "training": "HELD",
                    "fault_detail": f"duplicate panel_rows key {key}",
                }
            panel_map[key] = item["is_psucc"]

        p8_join = evaluate_k16_join_rows(k16_join_doc.get("p8_rows"), panel_map, "p8")
        if "fault" in p8_join:
            return {
                "verdict": "INSTRUMENT_FAULT_D31",
                "classification": "INSTRUMENT_FAULT_K16_JOIN_PRECONDITION",
                "mechanism_negative": False,
                "dispatch_decision": "STOP_FOR_INSTRUMENT_DIAGNOSIS_D31",
                "training": "HELD",
                "fault_detail": p8_join["fault"],
            }
        if (
            p8_join["full_target_admitted"] != 427
            or p8_join["psucc_target_admitted"] != 127
        ):
            return {
                "verdict": "INSTRUMENT_FAULT_D31",
                "classification": "INSTRUMENT_FAULT_K16_FORMULA_CHECK",
                "mechanism_negative": False,
                "dispatch_decision": "STOP_FOR_INSTRUMENT_DIAGNOSIS_D31",
                "training": "HELD",
                "fault_detail": (
                    f"P8 formula check admitted_cells - address_rivals == 1 expected (427, 127), "
                    f"got ({p8_join['full_target_admitted']}, {p8_join['psucc_target_admitted']})"
                ),
            }

        p16_join = evaluate_k16_join_rows(k16_join_doc.get("p16_rows"), panel_map, "p16")
        if "fault" in p16_join:
            return {
                "verdict": "INSTRUMENT_FAULT_D31",
                "classification": "INSTRUMENT_FAULT_K16_JOIN_PRECONDITION",
                "mechanism_negative": False,
                "dispatch_decision": "STOP_FOR_INSTRUMENT_DIAGNOSIS_D31",
                "training": "HELD",
                "fault_detail": p16_join["fault"],
            }
        a16_join = evaluate_k16_join_rows(k16_join_doc.get("a16_rows"), panel_map, "a16")
        if "fault" in a16_join:
            return {
                "verdict": "INSTRUMENT_FAULT_D31",
                "classification": "INSTRUMENT_FAULT_K16_JOIN_PRECONDITION",
                "mechanism_negative": False,
                "dispatch_decision": "STOP_FOR_INSTRUMENT_DIAGNOSIS_D31",
                "training": "HELD",
                "fault_detail": a16_join["fault"],
            }
        k16_bracket_summary = {
            "status": "VERIFIED",
            "p8_formula_check": "PASS",
            "p16": p16_join,
            "a16": a16_join,
            "bracket_can_promote": False,
        }

    # 4. Paired McNemar discordance readings on P_succ (3,847 rows)
    paired_doc = bundle.get("paired_psucc_discordance", {})
    require(isinstance(paired_doc, dict), "missing paired_psucc_discordance")
    a8_psucc = k8_summaries["a8"]["psucc_target_admitted"]
    s8_psucc = k8_summaries["s8"]["psucc_target_admitted"]
    a8_vs_p8 = evaluate_mcnemar_one_sided(
        paired_doc.get("a8_vs_p8"), a8_psucc, 127, "a8_vs_p8"
    )
    a8_vs_s8 = evaluate_mcnemar_one_sided(
        paired_doc.get("a8_vs_s8"), a8_psucc, s8_psucc, "a8_vs_s8"
    )
    placement_moved = a8_vs_p8["clears_one_sided_floor"]
    attributed_to_predecessor = placement_moved and a8_vs_s8["clears_one_sided_floor"]
    if attributed_to_predecessor:
        attribution = "PREDECESSOR_RELATION"
    elif placement_moved:
        attribution = "FREQUENCY_PACKING_ONLY"
    else:
        attribution = "PLACEMENT_DID_NOT_MOVE_ADMISSION"

    # 5. Optional P8 q-distance proxy distribution check (not a gate if omitted)
    q_dist = bundle.get("p8_q_distance_distribution")
    q_dist_status = "NOT_REPORTED"
    if q_dist is not None:
        require(
            isinstance(q_dist, dict) and len(q_dist) > 0,
            "p8_q_distance_distribution must be a non-empty histogram dict when provided",
        )
        total_q_rows = 0
        for dist_k, count_v in q_dist.items():
            require(
                isinstance(dist_k, str) and dist_k.isdigit() and is_u64(count_v),
                f"invalid p8_q_distance_distribution entry: {dist_k!r}: {count_v!r}",
            )
            total_q_rows += count_v
        require(
            total_q_rows in (14287, 3847),
            f"p8_q_distance_distribution counts must sum to 14287 (DEV) or 3847 (P_succ), got {total_q_rows}",
        )
        q_dist_status = f"REPORTED_{total_q_rows}_ROWS"

    # 6. A8 promotion battery (sheet §Bar)
    control_top1_cp0 = bundle.get(
        "control_top1_cp0", k8_summaries["p8"]["top1_correct_cp0"]
    )
    require(is_u64(control_top1_cp0), "control_top1_cp0 must be a non-negative integer")
    bar_i = control_top1_cp0 + 24
    net_g_cap_threshold = max(500, bar_i)
    net_g_cap = a8_psucc
    b3_init_pass = a8_psucc >= 500
    net_g_cap_pass = net_g_cap > net_g_cap_threshold

    gates = {
        "p8_exactness_427_127": "PASS",
        "g_init": g_init_status,
        "g_init_table_match": "PASS",
        "g_init_cross_arm_routes_gates_seed_epoch": "PASS",
        "g_init_pinned_vs_unpinned_routes_gates_epoch": g_init_pinned_vs_unpinned,
        "g_init_neg_a_host": "PASS",
        "g_init_neg_b_gpu_status6": "PASS",
        "g_occ": "PASS",
        "a8_b3_init_psucc": a8_psucc,
        "a8_b3_init_floor": 500,
        "a8_b3_init_pass": b3_init_pass,
        "p8_control_top1_cp0": k8_summaries["p8"]["top1_correct_cp0"],
        "a8_top1_cp0": k8_summaries["a8"]["top1_correct_cp0"],
        "control_top1_cp0": control_top1_cp0,
        "net_g_cap": net_g_cap,
        "net_g_cap_threshold": net_g_cap_threshold,
        "net_g_cap_pass": net_g_cap_pass,
        "placement_moved_admission": placement_moved,
        "attributed_to_predecessor_relation": attributed_to_predecessor,
        "attribution": attribution,
        "p8_q_distance_status": q_dist_status,
    }

    if b3_init_pass and net_g_cap_pass:
        out_res: dict[str, Any] = {
            "verdict": "A8_PROMOTED_TO_TRAINING_SHEET_DRAFT",
            "promotion_qualification": promotion_qualification,
            "classification": "A8_PREFLIGHT_GATES_PASSED",
            "mechanism_negative": False,
            "dispatch_decision": "DRAFT_A8_TRAINING_SHEET",
            "training": "HELD",
            "gates": gates,
            "k8_arms": k8_summaries,
            "paired_psucc_discordance": {
                "a8_vs_p8": a8_vs_p8,
                "a8_vs_s8": a8_vs_s8,
            },
            "k16_bracket": k16_bracket_summary,
        }
        if "archive_audit" in bundle:
            out_res["archive_audit"] = bundle["archive_audit"]
        return out_res

    return {
        "verdict": "TERMINAL_PREFLIGHT_FAIL_A8_BELOW_GATE",
        "promotion_qualification": "NONE",
        "classification": "CONFIGURATION_CLOSED_D22",
        "mechanism_negative": False,
        "dispatch_decision": "DO_NOT_DISPATCH_POST_SUCCESSOR_DECISION",
        "training": "HELD",
        "gates": gates,
        "k8_arms": k8_summaries,
        "paired_psucc_discordance": {
            "a8_vs_p8": a8_vs_p8,
            "a8_vs_s8": a8_vs_s8,
        },
        "k16_bracket": k16_bracket_summary,
    }


def make_synthetic_preflight_bundle() -> dict[str, Any]:
    """Construct a complete, valid synthetic 5-arm pre-flight bundle for `--self-test`."""
    exe_sha = "6280a695de8d0788e5e0f1f0a85079a36050dbc7194dc2d998bade7946d587e2"
    cursor = {
        "accepted_updates": 0,
        "completed_batches": 0,
        "corpus_pass": 0,
        "next_coordinate": 0,
        "next_target": 0,
    }

    def make_k8_arm(
        arm: str,
        model_sha: str,
        full_adm: int,
        psucc_adm: int,
        top1: int,
    ) -> dict[str, Any]:
        comp_adm = full_adm - psucc_adm
        return {
            "identity": {
                "config_sha256": EXPECTED_ARM_CONFIG_SHA256[arm],
                "cuda_context_created": False,
                "driver_source_sha256": EXPECTED_DRIVER_SOURCE_SHA256,
                "executable_sha256": exe_sha,
                "lineage_sha256": EXPECTED_ARM_LINEAGE_SHA256[arm],
                "training": "HELD",
            },
            "init": {
                "cursor": dict(cursor),
                "model_sha256": model_sha,
                "lineage_driver": EXPECTED_DRIVER_SOURCE_SHA256,
                "table_words_sha256": EXPECTED_ARM_TABLE_WORDS_SHA256[arm],
                "routes_sha256": EXPECTED_MODEL_ROUTES_SHA256,
                "gates_sha256": EXPECTED_MODEL_GATES_SHA256,
                "seed": 1,
                "epoch": 0,
            },
            "census": {
                "schema": "uor-r4.coverage-census/1",
                "status": "COMPLETE",
                "accepted_updates": 0,
                "completed_batches": 0,
                "host_cell_reconstruction": False,
                "model_execution": False,
                "model_sha256": model_sha,
                "vocabulary": 4096,
                "blocks": 4,
                "distinct_vectors": 4096,
                "alias_group_size_histogram": {"1": 4096},
                "dev_rows": 14287,
                "dev_alias_ceiling_rows": 14287,
                "within_cell_tie_mass": {
                    "dev_rows_targeting_non_representative": 0,
                    "tokens_in_alias_groups": 0,
                },
                "cells": {
                    "status": "FROM_GPU_TOKEN_CELL_RECEIPT",
                    "occupied_cells": 256,
                    "max_cell_occupancy": 16,
                    "cell_collision_pairs": 30720,
                    "cell_occupancy_histogram": {"16": 256},
                },
            },
            "eval": {
                "updates_applied_by_evaluation": 0,
                "evaluation_mutates_model": False,
                "accepted_updates_in_model": 0,
                "model_storage_sha256": model_sha,
                "aggregate": {
                    "scored_rows": 14287,
                    "occupied_cells": 256,
                    "target_admitted_rows": full_adm,
                    "top1_correct": top1,
                },
            },
            "b3": {
                "schema": "uor-r4.attention-preflight-dev-b3/1",
                "status": "VALIDATED_RETAINED_DEV_OBSERVATIONS",
                "training": "HELD",
                "updates_applied": 0,
                "completed_batches": 0,
                "model_epoch": 0,
                "evaluation_mutates_model": False,
                "cuda_context_created": False,
                "model_constructed": False,
                "scoring_performed_by_this_command": False,
                "compiled_identity_checked": True,
                "config_sha256": EXPECTED_ARM_CONFIG_SHA256[arm],
                "lineage_sha256": EXPECTED_ARM_LINEAGE_SHA256[arm],
                "model_storage_sha256": model_sha,
                "config_file": {
                    "path": f"/cfg/{arm}.json",
                    "sha256": EXPECTED_STAGED_CONFIG_FILE_PINS[f"{arm}.json"],
                },
                "b3": {
                    "psucc": {"rows": 3847, "target_admitted": psucc_adm},
                    "complement": {"rows": 10440, "target_admitted": comp_adm},
                    "full": {"rows": 14287, "target_admitted": full_adm},
                },
            },
        }

    panel_rows = [
        {
            "target_position": i + 1,
            "target_token": (i % 4095) + 1,
            "is_psucc": i < 3847,
        }
        for i in range(14287)
    ]

    def make_eval_rows(psucc_adm: int, comp_adm: int) -> list[dict[str, Any]]:
        rows: list[dict[str, Any]] = []
        for i in range(14287):
            if i < 3847:
                admitted = 1 if i < psucc_adm else 0
            else:
                admitted = 1 if (i - 3847) < comp_adm else 0
            rows.append(
                {
                    "target_position": i + 1,
                    "target_token": (i % 4095) + 1,
                    "admitted_cells": 8,
                    "address_rivals": 8 - admitted,
                }
            )
        return rows

    return {
        "rebuilt_table_shas": {
            "p8": EXPECTED_P8_TABLE_FILE_SHA256,
            "a8": EXPECTED_A8_TABLE_FILE_SHA256,
            "s8": EXPECTED_S8_TABLE_FILE_SHA256,
            "counts_sha256": EXPECTED_COUNTS_SHA256,
        },
        "build_identity": {
            "cuda_context_created": False,
            "driver_source_sha256": EXPECTED_DRIVER_SOURCE_SHA256,
            "executable_sha256": exe_sha,
            "training": "HELD",
        },
        "neg_a_stdout": "invalid discrete model: content hash differs: neg/a.json\n",
        "g_init_neg_b_status6": "PASS",
        "unpinned_init_comparison": {
            "codebook_initialization": "seeded_device_v1",
            "routes_sha256": EXPECTED_MODEL_ROUTES_SHA256,
            "gates_sha256": EXPECTED_MODEL_GATES_SHA256,
            "seed": 1,
            "epoch": 0,
        },
        "k8_arms": {
            "p8": make_k8_arm("p8", "1" * 64, 427, 127, 38),
            "a8": make_k8_arm("a8", "2" * 64, 1850, 620, 95),
            "s8": make_k8_arm("s8", "3" * 64, 920, 280, 52),
        },
        "paired_psucc_discordance": {
            "a8_vs_p8": {
                "both_admitted": 80,
                "treatment_only_b": 540,
                "control_only_c": 47,
                "neither_admitted": 3180,
            },
            "a8_vs_s8": {
                "both_admitted": 150,
                "treatment_only_b": 470,
                "control_only_c": 130,
                "neither_admitted": 3097,
            },
        },
        "k16_join": {
            "panel_rows": panel_rows,
            "p8_rows": make_eval_rows(127, 300),
            "p16_rows": make_eval_rows(254, 600),
            "a16_rows": make_eval_rows(1050, 2100),
        },
        "p8_q_distance_distribution": {
            "0": 1200,
            "8": 4500,
            "16": 5587,
            "24": 3000,
        },
    }


def run_self_tests() -> dict[str, Any]:
    """Run deterministic positive and negative self-tests."""
    base = make_synthetic_preflight_bundle()
    passed = 0
    total = 0

    # 1. Passing A8 bundle -> A8_PROMOTED_TO_TRAINING_SHEET_DRAFT
    total += 1
    res_pass = verify_preflight_bundle(base)
    require(
        res_pass["verdict"] == "A8_PROMOTED_TO_TRAINING_SHEET_DRAFT"
        and res_pass["gates"]["a8_b3_init_pass"] is True
        and res_pass["gates"]["net_g_cap_pass"] is True
        and res_pass["gates"]["placement_moved_admission"] is True
        and res_pass["gates"]["attributed_to_predecessor_relation"] is True
        and res_pass["gates"]["attribution"] == "PREDECESSOR_RELATION"
        and res_pass["k16_bracket"]["bracket_can_promote"] is False,
        f"synthetic passing bundle failed: {res_pass}",
    )
    passed += 1

    # 2. Terminal fail when A8 B3_init < 500 (e.g. 320 < 500) -> CONFIGURATION_CLOSED_D22
    total += 1
    fail_b3 = copy.deepcopy(base)
    fail_b3["k8_arms"]["a8"]["b3"]["b3"]["psucc"]["target_admitted"] = 320
    fail_b3["k8_arms"]["a8"]["b3"]["b3"]["complement"]["target_admitted"] = 680
    fail_b3["k8_arms"]["a8"]["b3"]["b3"]["full"]["target_admitted"] = 1000
    fail_b3["k8_arms"]["a8"]["eval"]["aggregate"]["target_admitted_rows"] = 1000
    fail_b3["paired_psucc_discordance"]["a8_vs_p8"] = {
        "both_admitted": 80,
        "treatment_only_b": 240,
        "control_only_c": 47,
        "neither_admitted": 3480,
    }
    fail_b3["paired_psucc_discordance"]["a8_vs_s8"] = {
        "both_admitted": 150,
        "treatment_only_b": 170,
        "control_only_c": 130,
        "neither_admitted": 3397,
    }
    res_fail_b3 = verify_preflight_bundle(fail_b3)
    require(
        res_fail_b3["verdict"] == "TERMINAL_PREFLIGHT_FAIL_A8_BELOW_GATE"
        and res_fail_b3["classification"] == "CONFIGURATION_CLOSED_D22"
        and res_fail_b3["mechanism_negative"] is False
        and res_fail_b3["gates"]["a8_b3_init_pass"] is False,
        f"expected TERMINAL_PREFLIGHT_FAIL_A8_BELOW_GATE, got {res_fail_b3}",
    )
    passed += 1

    # 3. Terminal fail when B3_init >= 500 (540) but net_g_cap <= max(500, control_top1_cp0 + 24)
    total += 1
    fail_gcap = copy.deepcopy(base)
    fail_gcap["control_top1_cp0"] = 600  # threshold = 624 >= 620
    res_fail_gcap = verify_preflight_bundle(fail_gcap)
    require(
        res_fail_gcap["verdict"] == "TERMINAL_PREFLIGHT_FAIL_A8_BELOW_GATE"
        and res_fail_gcap["gates"]["a8_b3_init_pass"] is True
        and res_fail_gcap["gates"]["net_g_cap_pass"] is False,
        "expected net_g_cap failure when net_g_cap <= control_top1_cp0 + 24",
    )
    passed += 1

    # 4. Attribution = FREQUENCY_PACKING_ONLY when A8 vs P8 clears floor but A8 vs S8 does not
    total += 1
    freq_only = copy.deepcopy(base)
    freq_only["k8_arms"]["s8"]["b3"]["b3"]["psucc"]["target_admitted"] = 605
    freq_only["k8_arms"]["s8"]["b3"]["b3"]["complement"]["target_admitted"] = 1195
    freq_only["k8_arms"]["s8"]["b3"]["b3"]["full"]["target_admitted"] = 1800
    freq_only["k8_arms"]["s8"]["eval"]["aggregate"]["target_admitted_rows"] = 1800
    freq_only["paired_psucc_discordance"]["a8_vs_s8"] = {
        "both_admitted": 590,
        "treatment_only_b": 30,
        "control_only_c": 15,
        "neither_admitted": 3212,
    }
    res_freq = verify_preflight_bundle(freq_only)
    require(
        res_freq["gates"]["placement_moved_admission"] is True
        and res_freq["gates"]["attributed_to_predecessor_relation"] is False
        and res_freq["gates"]["attribution"] == "FREQUENCY_PACKING_ONLY",
        f"expected FREQUENCY_PACKING_ONLY, got {res_freq['gates']}",
    )
    passed += 1

    # 5. Attribution = PLACEMENT_DID_NOT_MOVE_ADMISSION when A8 vs P8 does not clear floor
    total += 1
    no_move = copy.deepcopy(fail_b3)
    no_move["k8_arms"]["a8"]["b3"]["b3"]["psucc"]["target_admitted"] = 140
    no_move["k8_arms"]["a8"]["b3"]["b3"]["complement"]["target_admitted"] = 300
    no_move["k8_arms"]["a8"]["b3"]["b3"]["full"]["target_admitted"] = 440
    no_move["k8_arms"]["a8"]["eval"]["aggregate"]["target_admitted_rows"] = 440
    no_move["paired_psucc_discordance"]["a8_vs_p8"] = {
        "both_admitted": 120,
        "treatment_only_b": 20,
        "control_only_c": 7,
        "neither_admitted": 3700,
    }
    no_move["paired_psucc_discordance"]["a8_vs_s8"] = {
        "both_admitted": 100,
        "treatment_only_b": 40,
        "control_only_c": 180,
        "neither_admitted": 3527,
    }
    res_no_move = verify_preflight_bundle(no_move)
    require(
        res_no_move["gates"]["placement_moved_admission"] is False
        and res_no_move["gates"]["attribution"] == "PLACEMENT_DID_NOT_MOVE_ADMISSION",
        f"expected PLACEMENT_DID_NOT_MOVE_ADMISSION, got {res_no_move['gates']}",
    )
    passed += 1

    # 6. P8 exactness gate miss on full (428 != 427) -> INSTRUMENT_FAULT_D31
    total += 1
    p8_miss_full = copy.deepcopy(base)
    p8_miss_full["k8_arms"]["p8"]["b3"]["b3"]["complement"]["target_admitted"] = 301
    p8_miss_full["k8_arms"]["p8"]["b3"]["b3"]["full"]["target_admitted"] = 428
    p8_miss_full["k8_arms"]["p8"]["eval"]["aggregate"]["target_admitted_rows"] = 428
    res_p8_full = verify_preflight_bundle(p8_miss_full)
    require(
        res_p8_full["verdict"] == "INSTRUMENT_FAULT_D31"
        and res_p8_full["classification"] == "INSTRUMENT_FAULT_P8_EXACTNESS_MISS"
        and res_p8_full["mechanism_negative"] is False,
        f"expected INSTRUMENT_FAULT_P8_EXACTNESS_MISS on full=428, got {res_p8_full}",
    )
    passed += 1

    # 7. P8 exactness gate miss on P_succ (126 != 127) -> INSTRUMENT_FAULT_D31
    total += 1
    p8_miss_psucc = copy.deepcopy(base)
    p8_miss_psucc["k8_arms"]["p8"]["b3"]["b3"]["psucc"]["target_admitted"] = 126
    p8_miss_psucc["k8_arms"]["p8"]["b3"]["b3"]["complement"]["target_admitted"] = 301
    res_p8_psucc = verify_preflight_bundle(p8_miss_psucc)
    require(
        res_p8_psucc["verdict"] == "INSTRUMENT_FAULT_D31"
        and res_p8_psucc["classification"] == "INSTRUMENT_FAULT_P8_EXACTNESS_MISS",
        f"expected INSTRUMENT_FAULT_P8_EXACTNESS_MISS on psucc=126, got {res_p8_psucc}",
    )
    passed += 1

    # 8. K=16 join precondition miss: missing target_position -> INSTRUMENT_FAULT_D31
    total += 1
    k16_no_pos = copy.deepcopy(base)
    k16_no_pos["k16_join"]["p16_rows"][0]["target_position"] = None
    res_k16_no_pos = verify_preflight_bundle(k16_no_pos)
    require(
        res_k16_no_pos["verdict"] == "INSTRUMENT_FAULT_D31"
        and res_k16_no_pos["classification"] == "INSTRUMENT_FAULT_K16_JOIN_PRECONDITION",
        f"expected INSTRUMENT_FAULT_K16_JOIN_PRECONDITION, got {res_k16_no_pos}",
    )
    passed += 1

    # 9. K=16 join precondition miss: duplicate key -> INSTRUMENT_FAULT_D31
    total += 1
    k16_dup = copy.deepcopy(base)
    k16_dup["k16_join"]["a16_rows"][1]["target_position"] = k16_dup["k16_join"]["a16_rows"][0][
        "target_position"
    ]
    k16_dup["k16_join"]["a16_rows"][1]["target_token"] = k16_dup["k16_join"]["a16_rows"][0][
        "target_token"
    ]
    res_k16_dup = verify_preflight_bundle(k16_dup)
    require(
        res_k16_dup["verdict"] == "INSTRUMENT_FAULT_D31"
        and res_k16_dup["classification"] == "INSTRUMENT_FAULT_K16_JOIN_PRECONDITION",
        f"expected duplicate key INSTRUMENT_FAULT_D31, got {res_k16_dup}",
    )
    passed += 1

    # 10. K=16 join precondition miss: P_succ join count != 3847 -> INSTRUMENT_FAULT_D31
    total += 1
    k16_bad_psucc = copy.deepcopy(base)
    k16_bad_psucc["k16_join"]["panel_rows"][0]["is_psucc"] = False
    res_k16_bad_psucc = verify_preflight_bundle(k16_bad_psucc)
    require(
        res_k16_bad_psucc["verdict"] == "INSTRUMENT_FAULT_D31"
        and res_k16_bad_psucc["classification"] == "INSTRUMENT_FAULT_K16_JOIN_PRECONDITION",
        f"expected P_succ join count INSTRUMENT_FAULT_D31, got {res_k16_bad_psucc}",
    )
    passed += 1

    # 11. K=16 join precondition miss: admitted_cells - address_rivals == 2 -> INSTRUMENT_FAULT_D31
    total += 1
    k16_bad_delta = copy.deepcopy(base)
    k16_bad_delta["k16_join"]["p16_rows"][0]["address_rivals"] = 6  # 8 - 6 = 2
    res_k16_bad_delta = verify_preflight_bundle(k16_bad_delta)
    require(
        res_k16_bad_delta["verdict"] == "INSTRUMENT_FAULT_D31"
        and res_k16_bad_delta["classification"] == "INSTRUMENT_FAULT_K16_JOIN_PRECONDITION",
        f"expected delta outside {{0,1}} INSTRUMENT_FAULT_D31, got {res_k16_bad_delta}",
    )
    passed += 1

    # 12. K=16 formula check miss on P8 (admitted_cells - address_rivals == 1 != 427/127) -> INSTRUMENT_FAULT_D31
    total += 1
    k16_bad_p8_formula = copy.deepcopy(base)
    k16_bad_p8_formula["k16_join"]["p8_rows"][0]["address_rivals"] = 8  # drops P_succ from 127 to 126
    res_k16_formula = verify_preflight_bundle(k16_bad_p8_formula)
    require(
        res_k16_formula["verdict"] == "INSTRUMENT_FAULT_D31"
        and res_k16_formula["classification"] == "INSTRUMENT_FAULT_K16_FORMULA_CHECK",
        f"expected INSTRUMENT_FAULT_K16_FORMULA_CHECK, got {res_k16_formula}",
    )
    passed += 1

    # 13. K=16 bracket dropped as diagnostic (#2748 Registration bindings)
    total += 1
    k16_dropped = copy.deepcopy(base)
    k16_dropped["k16_join"] = {"status": "DROPPED_DIAGNOSTIC_NOT_RUN"}
    res_k16_dropped = verify_preflight_bundle(k16_dropped)
    require(
        res_k16_dropped["verdict"] == "A8_PROMOTED_TO_TRAINING_SHEET_DRAFT"
        and res_k16_dropped["k16_bracket"]["status"] == "DROPPED_DIAGNOSTIC_NOT_RUN"
        and res_k16_dropped["k16_bracket"]["bracket_can_promote"] is False,
        f"expected DROPPED_DIAGNOSTIC_NOT_RUN to pass K=8 gates, got {res_k16_dropped}",
    )
    passed += 1

    def check_fail(
        label: str,
        doc_source: dict[str, Any],
        mutator: Any,
        expected_substr: str,
    ) -> None:
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

    # 14–22. Tampering tests on verify_preflight_bundle
    check_fail(
        "rebuilt_a8_table_sha_mismatch",
        base,
        lambda d: d["rebuilt_table_shas"].__setitem__("a8", "0" * 64),
        "pod rebuilt table/counts SHA mismatch",
    )
    check_fail(
        "rebuilt_s8_table_sha_mismatch",
        base,
        lambda d: d["rebuilt_table_shas"].__setitem__("s8", "0" * 64),
        "pod rebuilt table/counts SHA mismatch",
    )
    check_fail(
        "driver_source_sha_mismatch",
        base,
        lambda d: d["build_identity"].__setitem__("driver_source_sha256", "0" * 64),
        "build_identity driver/executable SHA invalid",
    )
    check_fail(
        "a8_nonzero_updates",
        base,
        lambda d: d["k8_arms"]["a8"]["init"]["cursor"].__setitem__("accepted_updates", 1),
        "a8 G-Init init-summary check failed",
    )
    check_fail(
        "neg_a_missing_refusal",
        base,
        lambda d: d.__setitem__("neg_a_stdout", "ok"),
        "G-Init negative (a)",
    )
    check_fail(
        "neg_b_missing_status6",
        base,
        lambda d: d.__setitem__("g_init_neg_b_status6", "FAIL"),
        "G-Init negative (b)",
    )
    check_fail(
        "a8_g_occ_nonzero_tie_mass",
        base,
        lambda d: d["k8_arms"]["a8"]["census"]["within_cell_tie_mass"].__setitem__(
            "tokens_in_alias_groups", 2
        ),
        "a8 census injectivity / zero-tie-mass check failed",
    )
    check_fail(
        "s8_g_occ_histogram_mismatch",
        base,
        lambda d: d["k8_arms"]["s8"]["census"]["cells"].__setitem__(
            "cell_occupancy_histogram", {"16": 255, "15": 1}
        ),
        "s8 G-Occ cells check failed",
    )
    check_fail(
        "a8_b3_partition_mismatch",
        base,
        lambda d: d["k8_arms"]["a8"]["b3"]["b3"]["full"].__setitem__("target_admitted", 1851),
        "a8 B3 partition invariant failed",
    )
    check_fail(
        "mcnemar_marginal_mismatch",
        base,
        lambda d: d["paired_psucc_discordance"]["a8_vs_p8"].__setitem__("treatment_only_b", 541),
        "a8_vs_p8 paired P_succ partition must sum to 3847",
    )
    check_fail(
        "q_distance_bad_sum",
        base,
        lambda d: d["p8_q_distance_distribution"].__setitem__("0", 1201),
        "p8_q_distance_distribution counts must sum to 14287",
    )

    # 25. Partial G-Init when unpinned SeededDeviceV1 comparison is NOT_RUN / omitted (#2751 audit)
    total += 1
    partial_ginit = copy.deepcopy(base)
    del partial_ginit["unpinned_init_comparison"]
    res_partial = verify_preflight_bundle(partial_ginit)
    require(
        res_partial["verdict"] == "A8_PROMOTED_TO_TRAINING_SHEET_DRAFT"
        and res_partial["promotion_qualification"] == "CONDITIONAL_G_INIT_UNPINNED_NOT_RUN"
        and res_partial["gates"]["g_init"] == "PARTIAL_UNPINNED_COMPARISON_NOT_RUN"
        and res_partial["gates"]["g_init_table_match"] == "PASS"
        and res_partial["gates"]["g_init_cross_arm_routes_gates_seed_epoch"] == "PASS"
        and res_partial["gates"]["g_init_pinned_vs_unpinned_routes_gates_epoch"] == "NOT_RUN",
        f"expected PARTIAL_UNPINNED_COMPARISON_NOT_RUN when unpinned comparison omitted, got {res_partial['gates']}",
    )
    passed += 1

    # 26–29. Tampering tests on G-Init table_words_sha256, routes, gates, and unpinned comparison
    check_fail(
        "a8_table_words_sha_mismatch",
        base,
        lambda d: d["k8_arms"]["a8"]["init"].__setitem__("table_words_sha256", "0" * 64),
        "a8 G-Init model.table words SHA-256 mismatch",
    )
    check_fail(
        "a8_routes_sha_mismatch",
        base,
        lambda d: d["k8_arms"]["a8"]["init"].__setitem__("routes_sha256", "0" * 64),
        "a8 G-Init model routes/gates/seed/epoch check failed",
    )
    check_fail(
        "a8_gates_sha_mismatch",
        base,
        lambda d: d["k8_arms"]["a8"]["init"].__setitem__("gates_sha256", "0" * 64),
        "a8 G-Init model routes/gates/seed/epoch check failed",
    )
    check_fail(
        "unpinned_init_comparison_mismatch",
        base,
        lambda d: d["unpinned_init_comparison"].__setitem__("routes_sha256", "0" * 64),
        "G-Init pinned vs unpinned SeededDeviceV1 routes/gates/seed/epoch check failed",
    )

    # 30. Boundary test on McNemar one-sided floor F = max(24, 2*sqrt(b+c))
    total += 1
    eq_24 = evaluate_mcnemar_one_sided(
        {
            "both_admitted": 100,
            "treatment_only_b": 24,
            "control_only_c": 0,
            "neither_admitted": 3723,
        },
        124,
        100,
        "boundary_24_equal",
    )
    gt_24 = evaluate_mcnemar_one_sided(
        {
            "both_admitted": 100,
            "treatment_only_b": 25,
            "control_only_c": 0,
            "neither_admitted": 3722,
        },
        125,
        100,
        "boundary_25_gt",
    )
    eq_sqrt = evaluate_mcnemar_one_sided(
        {
            "both_admitted": 100,
            "treatment_only_b": 220,
            "control_only_c": 180,
            "neither_admitted": 3347,
        },
        320,
        280,
        "boundary_sqrt_equal",
    )
    require(
        eq_24["clears_one_sided_floor"] is False
        and gt_24["clears_one_sided_floor"] is True
        and eq_sqrt["clears_one_sided_floor"] is False,
        "McNemar strict inequality boundary test failed",
    )
    passed += 1

    # 31. Synthetic constructor test: BigramCounts + build_address_cells + B=2 non-injective refusal
    total += 1
    synth_tokens: list[int] = []
    for t in range(1, 512):
        for _ in range(1 + (512 - t) // 64):
            synth_tokens.append(t // 2)
            synth_tokens.append(t)
    synth_eligible = list(range(1, len(synth_tokens), 2))
    by_t, freq, _ = count_bigrams(synth_tokens, synth_eligible, 512)
    fallback_512 = [t % 256 for t in range(512)]
    built_struct = build_address_cells(512, 4, 8, 8, by_t, freq, fallback_512, "structured")
    built_shuf = build_address_cells(512, 4, 8, 8, by_t, freq, fallback_512, "shuffled", seed=8)
    require(
        built_struct["distinct_vectors"] == 512
        and built_struct["occupied_cells"] == 256
        and built_struct["quota_per_cell"] == 2
        and built_struct["total_train_l1"] < built_shuf["total_train_l1"],
        "synthetic address_cells constructor check failed",
    )
    try:
        build_address_cells(512, 2, 8, 8, by_t, freq, fallback_512, "structured")
        raise AssertionError("two-block non-injective table unexpectedly passed build_address_cells")
    except VerificationError as exc:
        require("not injective" in str(exc), f"unexpected error on B=2 table: {exc}")
    passed += 1

    # 32. Missing pre-flight directory raises VerificationError (never uncaught FileNotFoundError)
    total += 1
    try:
        load_preflight_bundle_from_dir(Path("/nonexistent/address-cells-preflight-dir"))
        raise AssertionError("missing preflight_dir unexpectedly succeeded")
    except VerificationError:
        passed += 1

    return {
        "self_test": "PASS",
        "passed": passed,
        "total": total,
    }


def parse_sha256_lines(text: str, label: str) -> list[tuple[str, str]]:
    """Parse `sha256  path` lines into `(digest, relpath)` pairs."""
    entries: list[tuple[str, str]] = []
    for raw_line in text.splitlines():
        line = raw_line.strip()
        if not line:
            continue
        parts = line.split(None, 1)
        require(len(parts) == 2, f"malformed line in {label}: {raw_line!r}")
        digest, rel = parts[0].strip(), parts[1].strip()
        if rel.startswith("*"):
            rel = rel[1:]
        require(is_hex64(digest), f"invalid SHA-256 digest in {label}: {digest!r}")
        entries.append((digest, rel))
    require(len(entries) > 0, f"{label} contains no SHA-256 entries")
    return entries


def verify_nonroot_negative_table_doc(table_doc: dict[str, Any], raw_bytes: bytes) -> dict[str, Any]:
    """Verify G-Init negative (b) non-root table (`070478a0…`, `table_sha256 = 8bf27283…`)."""
    file_sha = hashlib.sha256(raw_bytes).hexdigest()
    require(
        file_sha == EXPECTED_NEG_B_TABLE_FILE_SHA256,
        f"neg-b table file SHA-256 {file_sha} != {EXPECTED_NEG_B_TABLE_FILE_SHA256}",
    )
    words = table_doc.get("table")
    require(
        isinstance(words, list) and len(words) == 4096 * 32,
        "neg-b table must contain 131072 i32 words",
    )
    words_sha = hashlib.sha256(struct.pack(f"<{len(words)}i", *words)).hexdigest()
    require(
        table_doc.get("table_sha256") == words_sha
        and words_sha == EXPECTED_NEG_B_TABLE_WORDS_SHA256,
        f"neg-b table_sha256 mismatch: {table_doc.get('table_sha256')} / {words_sha}",
    )
    tok0_block0 = tuple(words[0:8])
    require(
        tok0_block0 not in ROOT_INDEX,
        f"neg-b token 0 block 0 {tok0_block0} is unexpectedly an orbit root",
    )
    return {
        "file_sha256": file_sha,
        "table_sha256": words_sha,
        "token0_block0": list(tok0_block0),
    }


def load_raw_pod_preflight_dir(preflight_dir: Path) -> dict[str, Any]:
    """Load and verify a raw pod pre-flight archive directory (e.g. `vast-56660582/.../address-cells-preflight-20261019`)."""
    files_sha_path = preflight_dir / "FILES.sha256"
    verified_manifest_files = 0
    if files_sha_path.is_file():
        entries = parse_sha256_lines(
            read_text_checked(files_sha_path, "FILES.sha256"), "FILES.sha256"
        )
        seen_rels: set[str] = set()
        for digest, rel in entries:
            while rel.startswith("./"):
                rel = rel[2:]
            require(rel not in seen_rels, f"duplicate path in FILES.sha256: {rel}")
            seen_rels.add(rel)
            target = preflight_dir / rel
            actual = hashlib.sha256(read_bytes_checked(target, f"manifest entry {rel}")).hexdigest()
            require(
                actual == digest,
                f"FILES.sha256 mismatch for {rel}: declared={digest} != actual={actual}",
            )
            verified_manifest_files += 1

    source_head = None
    source_head_path = preflight_dir / "source-head.txt"
    if source_head_path.is_file():
        source_head = read_text_checked(source_head_path, "source-head.txt").strip()
        require(
            bool(HEX40_RE.match(source_head)),
            f"invalid git commit SHA in source-head.txt: {source_head!r}",
        )

    inputs_sha_path = preflight_dir / "inputs-sha.txt"
    if inputs_sha_path.is_file():
        input_entries = parse_sha256_lines(
            read_text_checked(inputs_sha_path, "inputs-sha.txt"), "inputs-sha.txt"
        )
        by_basename = {Path(rel).name: digest for digest, rel in input_entries}
        by_suffix = {
            "/".join(Path(rel).parts[-2:]): digest for digest, rel in input_entries
        }
        require(
            by_suffix.get("train/tokens.u16") == EXPECTED_TRAIN_TOKENS_FILE_SHA256
            and by_suffix.get("train/response_mask.u8") == EXPECTED_TRAIN_MASK_FILE_SHA256
            and by_suffix.get("dev/tokens.u16") == EXPECTED_DEV_TOKENS_FILE_SHA256
            and by_suffix.get("dev/response_mask.u8") == EXPECTED_DEV_MASK_FILE_SHA256
            and by_basename.get("tokenizer.json") == EXPECTED_TOKENIZER_FILE_SHA256
            and by_basename.get("prime-zeta-v4096-b4-c192-k8.json")
            == EXPECTED_P8_TABLE_FILE_SHA256,
            f"inputs-sha.txt pin mismatch: {input_entries}",
        )

    a8_stdout = read_json_checked(preflight_dir / "tables" / "a8.stdout", "tables/a8.stdout")
    s8_stdout = read_json_checked(preflight_dir / "tables" / "s8.stdout", "tables/s8.stdout")
    require(
        a8_stdout.get("schema") == "uor-r4.codebook-table/1"
        and a8_stdout.get("mode") == "address_structured_v1"
        and a8_stdout.get("model_execution") is False
        and a8_stdout.get("file_sha256") == EXPECTED_A8_TABLE_FILE_SHA256
        and a8_stdout.get("table_sha256") == EXPECTED_A8_TABLE_WORDS_SHA256
        and a8_stdout.get("counts_sha256") == EXPECTED_COUNTS_SHA256
        and a8_stdout.get("train_tokens_sha256") == EXPECTED_TRAIN_TOKENS_WORDS_SHA256
        and a8_stdout.get("distinct_vectors") == 4096
        and a8_stdout.get("occupied_cells") == 256,
        f"tables/a8.stdout verification failed: {a8_stdout}",
    )
    require(
        s8_stdout.get("schema") == "uor-r4.codebook-table/1"
        and s8_stdout.get("mode") == "address_shuffled_v1_seed_20261019"
        and s8_stdout.get("model_execution") is False
        and s8_stdout.get("file_sha256") == EXPECTED_S8_TABLE_FILE_SHA256
        and s8_stdout.get("table_sha256") == EXPECTED_S8_TABLE_WORDS_SHA256
        and s8_stdout.get("counts_sha256") == EXPECTED_COUNTS_SHA256
        and s8_stdout.get("train_tokens_sha256") == EXPECTED_TRAIN_TOKENS_WORDS_SHA256
        and s8_stdout.get("distinct_vectors") == 4096
        and s8_stdout.get("occupied_cells") == 256,
        f"tables/s8.stdout verification failed: {s8_stdout}",
    )

    rebuilt_tables_verified = False
    a8_rebuilt_path = preflight_dir / "tables" / "a8-rebuilt.json"
    s8_rebuilt_path = preflight_dir / "tables" / "s8-rebuilt.json"
    if a8_rebuilt_path.is_file() and s8_rebuilt_path.is_file():
        a8_raw = read_bytes_checked(a8_rebuilt_path, "tables/a8-rebuilt.json")
        s8_raw = read_bytes_checked(s8_rebuilt_path, "tables/s8-rebuilt.json")
        require(
            hashlib.sha256(a8_raw).hexdigest() == EXPECTED_A8_TABLE_FILE_SHA256
            and hashlib.sha256(s8_raw).hexdigest() == EXPECTED_S8_TABLE_FILE_SHA256,
            "rebuilt table file SHA-256 mismatch",
        )
        verify_codebook_table_doc(
            json.loads(a8_raw.decode("utf-8")),
            expected_mode="address_structured_v1",
            expected_table_sha256=EXPECTED_A8_TABLE_WORDS_SHA256,
        )
        verify_codebook_table_doc(
            json.loads(s8_raw.decode("utf-8")),
            expected_mode="address_shuffled_v1_seed_20261019",
            expected_table_sha256=EXPECTED_S8_TABLE_WORDS_SHA256,
        )
        rebuilt_tables_verified = True

    build_identity = read_json_checked(preflight_dir / "build-identity.json", "build-identity.json")
    neg_a_stdout = read_text_checked(preflight_dir / "neg" / "a.stdout", "neg/a.stdout")
    neg_a_report_path = preflight_dir / "neg" / "a-run" / "report.json"
    if neg_a_report_path.is_file():
        neg_a_rep = read_json_checked(neg_a_report_path, "neg/a-run/report.json")
        require(
            neg_a_rep.get("status") == "FAILED"
            and neg_a_rep.get("training") == "HELD"
            and "invalid discrete model: content hash differs" in neg_a_rep.get("error", ""),
            f"neg/a-run/report.json check failed: {neg_a_rep}",
        )

    neg_b_stdout = read_text_checked(preflight_dir / "neg" / "b.stdout", "neg/b.stdout")
    require(
        "invalid discrete model: discrete CUDA snapshot validation refused with status 6"
        in neg_b_stdout,
        f"neg/b.stdout missing GPU status 6 refusal: {neg_b_stdout!r}",
    )
    neg_b_report_path = preflight_dir / "neg" / "b-run" / "report.json"
    if neg_b_report_path.is_file():
        neg_b_rep = read_json_checked(neg_b_report_path, "neg/b-run/report.json")
        require(
            neg_b_rep.get("status") == "FAILED"
            and neg_b_rep.get("training") == "HELD"
            and "invalid discrete model: discrete CUDA snapshot validation refused with status 6"
            in neg_b_rep.get("error", ""),
            f"neg/b-run/report.json check failed: {neg_b_rep}",
        )
    neg_b_json_path = preflight_dir / "neg" / "b.json"
    if neg_b_json_path.is_file():
        neg_b_bytes = read_bytes_checked(neg_b_json_path, "neg/b.json")
        verify_nonroot_negative_table_doc(json.loads(neg_b_bytes.decode("utf-8")), neg_b_bytes)

    k8_arms: dict[str, Any] = {}
    panel_rows_by_arm: dict[str, list[dict[str, Any]]] = {}
    eval_rows_by_arm: dict[str, list[dict[str, Any]]] = {}
    token_cells_verified_arms: list[str] = []

    for arm in ("p8", "a8", "s8"):
        arm_dir = preflight_dir / arm
        ident = read_json_checked(arm_dir / "identity.json", f"{arm}/identity.json")
        init_doc = read_json_checked(arm_dir / "init" / "report.json", f"{arm}/init/report.json")
        census = read_json_checked(arm_dir / "census.json", f"{arm}/census.json")
        ev = read_json_checked(arm_dir / "eval" / "report.json", f"{arm}/eval/report.json")
        b3_doc = read_json_checked(arm_dir / "b3" / "report.json", f"{arm}/b3/report.json")
        panel_doc = read_json_checked(arm_dir / "panel" / "report.json", f"{arm}/panel/report.json")
        cells_doc = read_json_checked(arm_dir / "cells" / "report.json", f"{arm}/cells/report.json")
        cells_u16_bytes = read_bytes_checked(
            arm_dir / "cells" / "token_cells.u16", f"{arm}/cells/token_cells.u16"
        )
        require(
            len(cells_u16_bytes) == 8192,
            f"{arm}/cells/token_cells.u16 must be 8192 bytes, got {len(cells_u16_bytes)}",
        )
        cells_u16_sha = hashlib.sha256(cells_u16_bytes).hexdigest()
        require(
            cells_doc.get("schema") == "uor-r4.attention-token-cells/1"
            and cells_doc.get("status") == "DEVICE_TOKEN_CELL_MAP_EXPORTED_ZERO_APPLY"
            and cells_doc.get("token_cells_file", {}).get("sha256") == cells_u16_sha
            and cells_doc.get("model_storage_sha256") == init_doc.get("model_sha256")
            and cells_doc.get("host_cell_reconstruction") is False
            and cells_doc.get("updates_applied") == 0
            and cells_doc.get("training") == "HELD"
            and cells_doc.get("token_count") == 4096
            and cells_doc.get("byte_count") == 8192,
            f"{arm}/cells/report.json verification failed",
        )
        # Cross-verify GPU token_cells.u16 against checkpoint-0 model.table orbit roots
        words = init_doc.get("model", {}).get("table", [])
        require(len(words) == 4096 * 32, f"{arm} init model.table length != 131072")
        u16_cells = struct.unpack("<4096H", cells_u16_bytes)
        expected_cells = [
            ROOT_INDEX[tuple(words[t * 32 : t * 32 + 8])] * 16
            + ROOT_INDEX[tuple(words[t * 32 + 24 : t * 32 + 32])]
            for t in range(4096)
        ]
        require(
            list(u16_cells) == expected_cells,
            f"{arm} GPU token_cells.u16 does not match model.table block-0/block-3 cells",
        )
        token_cells_verified_arms.append(arm)

        require(
            panel_doc.get("schema") == "uor-r4.attention-preflight-panel/1"
            and panel_doc.get("status") == "PINNED_PANEL_AND_PROBE_METADATA_ONLY"
            and panel_doc.get("cuda_context_created") is False
            and panel_doc.get("model_constructed") is False
            and panel_doc.get("scoring_performed") is False
            and panel_doc.get("training") == "HELD"
            and panel_doc.get("updates_applied") == 0
            and isinstance(panel_doc.get("rows"), list)
            and len(panel_doc["rows"]) == 14287,
            f"{arm}/panel/report.json verification failed",
        )
        panel_rows_by_arm[arm] = panel_doc["rows"]
        ev_rows = ev.get("rows")
        if ev_rows is None and isinstance(ev.get("batches"), list):
            ev_rows = [
                r
                for batch in ev["batches"]
                if isinstance(batch, dict) and isinstance(batch.get("rows"), list)
                for r in batch["rows"]
            ]
        require(
            isinstance(ev_rows, list) and len(ev_rows) == 14287,
            f"{arm}/eval/report.json must contain 14287 rows across batches",
        )
        eval_rows_by_arm[arm] = ev_rows

        k8_arms[arm] = {
            "identity": ident,
            "init": init_doc,
            "census": census,
            "eval": ev,
            "b3": b3_doc,
        }

    # Verify panel (target_position, target_token, is_psucc) is identical across p8, a8, s8
    p8_panel_keys = [
        (r["target_position"], r["target_token"], r["is_psucc"]) for r in panel_rows_by_arm["p8"]
    ]
    for other_arm in ("a8", "s8"):
        other_keys = [
            (r["target_position"], r["target_token"], r["is_psucc"])
            for r in panel_rows_by_arm[other_arm]
        ]
        require(
            other_keys == p8_panel_keys,
            f"{other_arm} panel (target_position, target_token, is_psucc) keys differ from p8",
        )

    # Compute paired P_succ discordance from raw rows
    psucc_map = {
        (r["target_position"], r["target_token"]): r["is_psucc"] for r in panel_rows_by_arm["p8"]
    }
    admitted_on_psucc: dict[str, dict[tuple[int, int], bool]] = {}
    for arm in ("p8", "a8", "s8"):
        adm_map: dict[tuple[int, int], bool] = {}
        for r in eval_rows_by_arm[arm]:
            key = (r["target_position"], r["target_token"])
            if psucc_map.get(key) is True:
                adm_map[key] = (r["admitted_cells"] - r["address_rivals"]) == 1
        require(len(adm_map) == 3847, f"{arm} P_succ row count != 3847")
        admitted_on_psucc[arm] = adm_map

    def compute_pair_counts(treat_arm: str, ctrl_arm: str) -> dict[str, int]:
        t_map = admitted_on_psucc[treat_arm]
        c_map = admitted_on_psucc[ctrl_arm]
        n11 = sum(1 for k in t_map if t_map[k] and c_map[k])
        b = sum(1 for k in t_map if t_map[k] and not c_map[k])
        c = sum(1 for k in t_map if (not t_map[k]) and c_map[k])
        n00 = sum(1 for k in t_map if (not t_map[k]) and (not c_map[k]))
        return {
            "both_admitted": n11,
            "treatment_only_b": b,
            "control_only_c": c,
            "neither_admitted": n00,
        }

    paired_discordance = {
        "a8_vs_p8": compute_pair_counts("a8", "p8"),
        "a8_vs_s8": compute_pair_counts("a8", "s8"),
    }

    # Verify K=16 bracket arms (p16, a16)
    expected_cursor = {
        "accepted_updates": 0,
        "completed_batches": 0,
        "corpus_pass": 0,
        "next_coordinate": 0,
        "next_target": 0,
    }
    driver_sha = build_identity.get("driver_source_sha256", "")
    k16_eval_rows: dict[str, list[dict[str, Any]]] = {}
    k16_summaries: dict[str, dict[str, Any]] = {}
    for arm in ("p16", "a16"):
        arm_dir = preflight_dir / arm
        ident_text = read_text_checked(arm_dir / "identity.json", f"{arm}/identity.json")
        require(
            "invalid discrete model: attention requires K8" in ident_text,
            f"{arm}/identity.json expected K8 attention-build-identity refusal, got {ident_text!r}",
        )
        init_doc = read_json_checked(arm_dir / "init" / "report.json", f"{arm}/init/report.json")
        ev = read_json_checked(arm_dir / "eval" / "report.json", f"{arm}/eval/report.json")
        init_info = verify_init_model_doc(init_doc, arm, expected_cursor, driver_sha)
        agg = ev.get("aggregate", {})
        ev_rows = ev.get("rows")
        if ev_rows is None and isinstance(ev.get("batches"), list):
            ev_rows = [
                r
                for batch in ev["batches"]
                if isinstance(batch, dict) and isinstance(batch.get("rows"), list)
                for r in batch["rows"]
            ]
        require(
            ev.get("updates_applied_by_evaluation") == 0
            and ev.get("evaluation_mutates_model") is False
            and ev.get("accepted_updates_in_model") == 0
            and ev.get("model_storage_sha256") == init_info["model_sha256"]
            and ev.get("evaluation_lineage_sha256") == EXPECTED_ARM_LINEAGE_SHA256[arm]
            and agg.get("scored_rows") == 14287
            and agg.get("occupied_cells") == 256
            and isinstance(ev_rows, list)
            and len(ev_rows) == 14287,
            f"{arm}/eval/report.json verification failed",
        )
        k16_eval_rows[arm] = ev_rows
        k16_summaries[arm] = {
            "model_sha256": init_info["model_sha256"],
            "table_words_sha256": init_info["table_words_sha256"],
            "evaluation_lineage_sha256": ev.get("evaluation_lineage_sha256"),
            "attention_build_identity_k8_refusal": "PASS",
            "top1_correct_cp0": agg.get("top1_correct"),
            "top8_correct_cp0": agg.get("top8_correct"),
            "target_admitted_rows": agg.get("target_admitted_rows"),
        }

    return {
        "rebuilt_table_shas": {
            "p8": EXPECTED_P8_TABLE_FILE_SHA256,
            "a8": a8_stdout["file_sha256"],
            "s8": s8_stdout["file_sha256"],
            "counts_sha256": a8_stdout["counts_sha256"],
        },
        "build_identity": build_identity,
        "neg_a_stdout": neg_a_stdout,
        "g_init_neg_b_status6": "PASS",
        "k8_arms": k8_arms,
        "paired_psucc_discordance": paired_discordance,
        "k16_join": {
            "panel_rows": panel_rows_by_arm["p8"],
            "p8_rows": eval_rows_by_arm["p8"],
            "p16_rows": k16_eval_rows["p16"],
            "a16_rows": k16_eval_rows["a16"],
        },
        "archive_audit": {
            "source_head": source_head,
            "files_sha256_verified_entries": verified_manifest_files,
            "rebuilt_tables_verified": rebuilt_tables_verified,
            "token_cells_u16_verified_arms": token_cells_verified_arms,
            "k16_arms_init_eval_verified": k16_summaries,
        },
    }


def load_preflight_bundle_from_dir(preflight_dir: Path) -> dict[str, Any]:
    """Load a 5-arm pre-flight receipt bundle from either a raw pod archive or synthetic bundle layout."""
    require(preflight_dir.is_dir(), f"preflight directory does not exist: {preflight_dir}")
    if (preflight_dir / "p8" / "init" / "report.json").is_file() or (
        preflight_dir / "tables" / "a8.stdout"
    ).is_file():
        return load_raw_pod_preflight_dir(preflight_dir)

    rebuilt_shas = read_json_checked(
        preflight_dir / "rebuilt-table-shas.json", "rebuilt-table-shas.json"
    )
    build_identity = read_json_checked(
        preflight_dir / "build-identity.json", "build-identity.json"
    )
    neg_a_stdout = read_text_checked(preflight_dir / "neg" / "a.stdout", "neg/a.stdout")
    neg_b_status = read_text_checked(preflight_dir / "neg" / "b.status", "neg/b.status").strip()

    k8_arms: dict[str, Any] = {}
    for arm in ("p8", "a8", "s8"):
        arm_dir = preflight_dir / arm
        k8_arms[arm] = {
            "identity": read_json_checked(arm_dir / "identity.json", f"{arm}/identity.json"),
            "init": read_json_checked(arm_dir / "init-summary.json", f"{arm}/init-summary.json"),
            "census": read_json_checked(arm_dir / "census.json", f"{arm}/census.json"),
            "eval": read_json_checked(arm_dir / "eval-aggregate.json", f"{arm}/eval-aggregate.json"),
            "b3": read_json_checked(arm_dir / "b3" / "summary.json", f"{arm}/b3/summary.json"),
        }

    paired_discordance = read_json_checked(
        preflight_dir / "paired-psucc-discordance.json", "paired-psucc-discordance.json"
    )
    bundle: dict[str, Any] = {
        "rebuilt_table_shas": rebuilt_shas,
        "build_identity": build_identity,
        "neg_a_stdout": neg_a_stdout,
        "g_init_neg_b_status6": neg_b_status,
        "k8_arms": k8_arms,
        "paired_psucc_discordance": paired_discordance,
    }
    k16_join_path = preflight_dir / "k16-join.json"
    if k16_join_path.is_file():
        bundle["k16_join"] = read_json_checked(k16_join_path, "k16-join.json")
    q_dist_path = preflight_dir / "p8" / "q-distance.json"
    if q_dist_path.is_file():
        bundle["p8_q_distance_distribution"] = read_json_checked(q_dist_path, "p8/q-distance.json")
    return bundle


def verify_committed_receipt_dir(
    receipt_dir: Path,
    preflight_verdict: dict[str, Any] | None = None,
) -> dict[str, Any]:
    """Verify `docs/labs/address-cells-preflight-2026-10-19` committed receipt directory (#2751)."""
    require(receipt_dir.is_dir(), f"receipt directory does not exist: {receipt_dir}")
    evidence_dir = receipt_dir / "evidence"
    sha_sums_path = evidence_dir / "SHA256SUMS"
    sha_sums_bytes = read_bytes_checked(sha_sums_path, "evidence/SHA256SUMS")
    sha_sums_digest = hashlib.sha256(sha_sums_bytes).hexdigest()
    entries = parse_sha256_lines(sha_sums_bytes.decode("utf-8"), "evidence/SHA256SUMS")
    parsed_pins: dict[str, str] = {}
    for digest, name in entries:
        require(name != "SHA256SUMS", "evidence/SHA256SUMS must not list itself")
        target = evidence_dir / name
        actual = hashlib.sha256(read_bytes_checked(target, f"evidence/{name}")).hexdigest()
        require(
            actual == digest,
            f"evidence/SHA256SUMS mismatch for {name}: declared={digest} != actual={actual}",
        )
        parsed_pins[name] = actual

    for req_name in ("FILES.sha256", "asc-battery.sh", "grading.json", "job.log", "neg-b-nonroot-table.json"):
        require(req_name in parsed_pins, f"evidence/SHA256SUMS missing {req_name}")

    files_manifest_entries = parse_sha256_lines(
        read_text_checked(evidence_dir / "FILES.sha256", "evidence/FILES.sha256"),
        "evidence/FILES.sha256",
    )
    require(
        len(files_manifest_entries) == 72,
        f"evidence/FILES.sha256 expected 72 entries, got {len(files_manifest_entries)}",
    )

    neg_b_bytes = read_bytes_checked(
        evidence_dir / "neg-b-nonroot-table.json", "evidence/neg-b-nonroot-table.json"
    )
    neg_b_info = verify_nonroot_negative_table_doc(
        json.loads(neg_b_bytes.decode("utf-8")), neg_b_bytes
    )

    job_log = read_text_checked(evidence_dir / "job.log", "evidence/job.log")
    for req_token in (
        "build_rc=0",
        "build_identity_rc=0",
        "a8_rc=0",
        "s8_rc=0",
        "TABLE_REBUILD: PASS",
        "neg_a_rc=2",
        "neg_b_rc=2",
        "p8 identity_rc=0",
        "p8 init_rc=0",
        "p8 eval_rc=0",
        "p8 b3_rc=0",
        "p8 panel_rc=0",
        "p8 cells_rc=0",
        "p8 census_rc=0",
        "a8 identity_rc=0",
        "a8 init_rc=0",
        "a8 eval_rc=0",
        "a8 b3_rc=0",
        "a8 panel_rc=0",
        "a8 cells_rc=0",
        "a8 census_rc=0",
        "s8 identity_rc=0",
        "s8 init_rc=0",
        "s8 eval_rc=0",
        "s8 b3_rc=0",
        "s8 panel_rc=0",
        "s8 cells_rc=0",
        "s8 census_rc=0",
        "p16 identity_rc=2",
        "p16 init_rc=0",
        "p16 eval_rc=0",
        "a16 identity_rc=2",
        "a16 init_rc=0",
        "a16 eval_rc=0",
        "BATTERY_DONE",
    ):
        require(req_token in job_log, f"evidence/job.log missing required token {req_token!r}")

    grading = read_json_checked(evidence_dir / "grading.json", "evidence/grading.json")
    require(
        grading.get("schema") == "uor-r4.address-cells-preflight-grading/1"
        and grading.get("source_commit") == "93b54f48c4e1983e278140f77a3377041ddd65e4",
        f"grading.json header mismatch: {grading}",
    )
    arms = grading.get("arms", {})
    require(
        arms.get("p8", {}).get("b3")
        == {"full": 427, "psucc": 127, "psucc_rows": 3847, "complement": 300}
        and arms.get("p8", {}).get("top1_cp0") == 38
        and arms.get("p8", {}).get("top8_cp0") == 95
        and arms.get("a8", {}).get("b3")
        == {"full": 4244, "psucc": 1351, "psucc_rows": 3847, "complement": 2893}
        and arms.get("a8", {}).get("top1_cp0") == 133
        and arms.get("a8", {}).get("top8_cp0") == 513
        and arms.get("s8", {}).get("b3")
        == {"full": 1733, "psucc": 476, "psucc_rows": 3847, "complement": 1257}
        and arms.get("s8", {}).get("top1_cp0") == 99
        and arms.get("s8", {}).get("top8_cp0") == 185,
        f"grading.json arms summary mismatch: {arms}",
    )

    k16_j = grading.get("k16_join", {})
    require(
        k16_j.get("p8_formula_check")
        == {
            "rows": 14287,
            "keyed": 14287,
            "keys_unique": True,
            "joined_psucc": 3847,
            "unjoined": 0,
            "outside_0_1": 0,
            "admitted_full": 427,
            "admitted_psucc": 127,
        }
        and k16_j.get("p16", {}).get("admitted_full") == 872
        and k16_j.get("p16", {}).get("admitted_psucc") == 246
        and k16_j.get("a16", {}).get("admitted_full") == 6023
        and k16_j.get("a16", {}).get("admitted_psucc") == 2178
        and k16_j.get("top1_cp0") == {"p16": 37, "a16": 133},
        f"grading.json k16_join mismatch: {k16_j}",
    )

    paired = grading.get("paired_psucc", {})
    require(
        paired.get("a8_vs_p8", {}).get("n11") == 49
        and paired.get("a8_vs_p8", {}).get("b") == 1302
        and paired.get("a8_vs_p8", {}).get("c") == 78
        and paired.get("a8_vs_p8", {}).get("n00") == 2418
        and paired.get("a8_vs_p8", {}).get("b_minus_c") == 1224
        and paired.get("a8_vs_p8", {}).get("clears") is True
        and paired.get("a8_vs_s8", {}).get("n11") == 93
        and paired.get("a8_vs_s8", {}).get("b") == 1258
        and paired.get("a8_vs_s8", {}).get("c") == 383
        and paired.get("a8_vs_s8", {}).get("n00") == 2113
        and paired.get("a8_vs_s8", {}).get("b_minus_c") == 875
        and paired.get("a8_vs_s8", {}).get("clears") is True,
        f"grading.json paired_psucc mismatch: {paired}",
    )

    net_g_cap_doc = grading.get("net_g_cap_a8", {})
    declared_ctrl_top1 = net_g_cap_doc.get("control_top1_cp0")
    require(
        declared_ctrl_top1 in (38, 133)
        and net_g_cap_doc.get("alias_ceiling") == 14287
        and net_g_cap_doc.get("b3_psucc") == 1351
        and net_g_cap_doc.get("net") == 1351
        and net_g_cap_doc.get("threshold") == 500,
        f"grading.json net_g_cap_a8 mismatch: {net_g_cap_doc}",
    )

    reconciled_with_raw_archive = False
    if preflight_verdict is not None:
        k8_v = preflight_verdict.get("k8_arms", {})
        for arm in ("p8", "a8", "s8"):
            require(
                arms[arm]["b3"]["full"] == k8_v[arm]["full_target_admitted"]
                and arms[arm]["b3"]["psucc"] == k8_v[arm]["psucc_target_admitted"]
                and arms[arm]["b3"]["complement"] == k8_v[arm]["complement_target_admitted"]
                and arms[arm]["top1_cp0"] == k8_v[arm]["top1_correct_cp0"],
                f"grading.json vs raw archive mismatch on {arm}",
            )
        reconciled_with_raw_archive = True

    return {
        "verdict": "PASS",
        "sha256sums_digest": sha_sums_digest,
        "evidence_pins": parsed_pins,
        "files_sha256_entry_count": len(files_manifest_entries),
        "neg_b_nonroot_table": neg_b_info,
        "control_top1_cp0_audit": {
            "declared_in_grading_json": declared_ctrl_top1,
            "p8_control_top1_cp0": 38,
            "a8_top1_cp0": 133,
            "threshold_under_p8_38": max(500, 38 + 24),
            "threshold_under_a8_133": max(500, 133 + 24),
            "net_g_cap_a8": 1351,
            "gate_outcome_unchanged": True,
        },
        "g_init_audit": {
            "g_init_table_match": "PASS",
            "g_init_neg_a_host": "PASS",
            "g_init_neg_b_gpu_status6": "PASS",
            "g_init_cross_arm_routes_gates_seed_epoch": "PASS",
            "g_init_pinned_vs_unpinned_routes_gates_epoch": "NOT_RUN",
            "promotion_qualification": "CONDITIONAL_G_INIT_UNPINNED_NOT_RUN",
        },
        "reconciled_with_raw_archive": reconciled_with_raw_archive,
    }


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Deterministic host-only consumer verifier for Address-Structured Injective Cells."
    )
    parser.add_argument(
        "--self-test",
        action="store_true",
        help="Run 32 positive and negative self-tests",
    )
    parser.add_argument(
        "--configs-dir",
        type=Path,
        help="Path to docs/labs/address-cells-preflight-configs-2026-10-19",
    )
    parser.add_argument(
        "--repo-root",
        type=Path,
        help="Path to repository root",
    )
    parser.add_argument(
        "--train-tokens",
        type=Path,
        help="Path to TRAIN tokens.u16 (3dee858f…) to reproduce A8 and S8 tables on host CPU",
    )
    parser.add_argument(
        "--train-mask",
        type=Path,
        help="Path to TRAIN response_mask.u8 (b5e7352d…) to reproduce A8 and S8 tables on host CPU",
    )
    parser.add_argument(
        "--preflight-dir",
        type=Path,
        help="Path to executed 5-arm address-cells pre-flight pod archive or bundle directory",
    )
    parser.add_argument(
        "--receipt-dir",
        type=Path,
        help="Path to docs/labs/address-cells-preflight-2026-10-19 committed receipt directory (#2751)",
    )
    args = parser.parse_args()

    if (
        not args.self_test
        and args.configs_dir is None
        and args.train_tokens is None
        and args.preflight_dir is None
        and args.receipt_dir is None
    ):
        parser.error(
            "specify at least one of --self-test, --configs-dir, --train-tokens, --preflight-dir, or --receipt-dir"
        )

    out: dict[str, Any] = {}
    if args.self_test:
        out["self_test"] = run_self_tests()

    if args.configs_dir is not None:
        out["configs_verification"] = verify_configs_and_repo(
            args.configs_dir, repo_root=args.repo_root
        )

    if args.train_tokens is not None or args.train_mask is not None:
        require(
            args.train_tokens is not None
            and args.train_mask is not None
            and args.repo_root is not None,
            "--train-tokens and --train-mask require each other and --repo-root",
        )
        pz_path = (
            args.repo_root
            / "docs"
            / "labs"
            / "injective-attention-runbook-2026-10-19"
            / "evidence"
            / "prime-zeta-v4096-b4-c192-k8.json"
        )
        out["train_table_reproduction"] = reproduce_and_verify_from_train(
            args.train_tokens, args.train_mask, pz_path
        )

    preflight_verdict: dict[str, Any] | None = None
    if args.preflight_dir is not None:
        if (args.preflight_dir / "evidence" / "grading.json").is_file() and not (
            args.preflight_dir / "p8"
        ).is_dir():
            out["receipt_verification"] = verify_committed_receipt_dir(args.preflight_dir)
        else:
            bundle = load_preflight_bundle_from_dir(args.preflight_dir)
            preflight_verdict = verify_preflight_bundle(bundle)
            out["preflight_verification"] = preflight_verdict

    if args.receipt_dir is not None:
        out["receipt_verification"] = verify_committed_receipt_dir(
            args.receipt_dir, preflight_verdict=preflight_verdict
        )

    print(json.dumps(out, indent=2, sort_keys=True))
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except VerificationError as err:
        print(json.dumps({"status": "FAIL", "error": str(err)}, sort_keys=True), file=sys.stderr)
        sys.exit(1)
