#!/usr/bin/env python3
"""Host-only deterministic consumer verifier for the A8 Address-Cells Learned-Attention
pre-registration sheet, staged dose runbook, executor packet, pre-run gates, Q1/full-dose
trajectory, D34 paired-DEV reference scorer, and Precedence 0..6 decision table.

Verifies:
1. Staged Runbook, Configs, Sheet & Source Contract (`verify_staged_contract`):
   - `docs/harness/prep/m1-attention-a8-address-cells-learning-2026-10-19.md`
     (`3e622bebd7c468c6cd3d15a38486768c7902807998f71cae039a4341ab55da2b`, 12,697 B;
     `TEST FITNESS: FIT`, canonical title prefix, float-free constructor check)
   - `crates/uor-r4-discrete/src/attention_learning.rs` (`21e8b528…`) and 11-file
     `driver_source_sha256` (`f673edb5d3bfea8bae44594adf122fbe937626c60c5be838d59a3ffc42f55ccf`),
     plus exact `registered_sheet(config)` mapping (`SeededDeviceV1`, `PrimeZeta` `83c1380a…`,
     `A8` `6600bdb1…`, and refusal of any unregistered `PinnedTableV1` such as `S8` `a185bdca…`)
   - `docs/labs/a8-training-configs-2026-10-19/evidence/SHA256SUMS` (`8a72d6fd…`, 181 B;
     no self-entry) and staged configs:
     - `a8-relation-seed1.json` (`469a2ff0…`, 1,924 B; `config_sha256 = f223d78f…`,
       `lineage_sha256 = 265c91d0…`, byte-identical to pre-flight `a8.json`)
     - `a8-frozen-zero-seed1.json` (`cf7ebcc8…`, 1,930 B; `config_sha256 = c3fbffe0…`,
       `lineage_sha256 = 3f3f4816…`)
     - Single-key diffs against `injective-{relation,frozen-zero}-seed1.json` and
       `parent-rows-{relation,frozen-zero}-seed1.json`, and between the two A8 arms
   - `docs/labs/a8-dose-runbook-2026-10-19/` (`README.md` `9fcf48cf…`, `evidence/SHA256SUMS`
     `4293db2e…`, and `evidence/neg-b-a8-nonroot-table.json` `e35bb48d…` with inner word
     digest `f9e1ea54…`, 4,096 distinct vectors, token-0 block-0 `[0,2,2,0,0,0,0,0]` not in
     `ROOTS`, and exact 131,064-word tail reconstructing A8's `4122cbab…` when block 0 is
     restored to `[0,0,0,0,0,-2,2,0]`)
   - Optional executor packet (`--packet-dir`, e.g. `/Users/casey.allard/uor-r4-local/a8-dose-executor-packet-20261019`):
     verifies `bound-inputs/SHA256SUMS` (`7/7` entries, including `d34-successor-table-record.json`
     `2717d34b…`, 5,519,089 B) and `INHERITED-CHECKS.md` / `STATUS.md` / `OWNER-RELEASE-READINESS.md`.
2. Pre-Run Gate & Plan/Release Consumer Verification (`verify_prerun_bundle`):
   - Pod-rebuilt A8 table (`6600bdb1…` / `4122cbab…` / `counts_sha256 = 755a5a81…`) and
     `attention-build-identity` (`cuda_context_created = false`, `training = "HELD"`, no `scope`)
   - `G-Init` on both arms (`relation` and `frozen_zero`): checkpoint-0 `cursor == 0`,
     131,072-word `model.table` digest `4122cbab…`, active-leaf 45-gate layout (`[0,5,10,15] == +1`),
     cross-arm checkpoint-0 bit-identity, **hard unpinned `SeededDeviceV1` comparison**
     (`routes`, `gates`, `seed`, `epoch` equal to `parent-rows-{relation,frozen-zero}-seed1`
     while unpinned `table` differs from A8), negative (a) host refusal, and negative (b)
     GPU status-6 refusal on `neg-b-a8-nonroot-table.json`
   - `G-Occ` on both arms (`occupied_cells == 256 > 16`, zero within-cell tie mass,
     `distinct_vectors == 4096`, `alias_group_size_histogram == {"1": 4096}`,
     `cell_occupancy_histogram == {"16": 256}`, `cell_collision_pairs == 30,720`)
   - Reproduction gate (`D31`) on **both** arms: `B3_init(P_succ) == 1,351 / 3,847`,
     `B3_init(full) == 4,244 / 14,287` (`complement == 2,893 / 10,440`), `top1_cp0(full) == 133`;
     `Bar (i) = top1_control(P_succ, cp0) + 24`, and `net_g_cap (1,351) > max(500, Bar (i))`
   - Inherited controls:
     - `both_probes_any_active_candidate_cost_differs_v1` on Gate 22 and Gate 0 for both arms
       (`any_candidate_cost_differs_from_identity == true`, `epoch == 0`, `applied == false`,
       16 `candidate_losses`, `before`/`after`/`candidate` retained verbatim)
     - Canonical `qualify-relation` `relation-controls.json` (`status == "PASS"`, `9/9` controls,
       C2 ACTIVE `relation_copy_successor_fixture`, C2 INERT + `init_locks[]` in
       `active_leaf_initialization_and_matched_off_path` for seeds 1 and 2)
     - Reference battery (`attention-cell-reference-scorer preflight-dev`): accepts explicit
       `NOT_RUN_DEFERRED_TO_DOSE` (with supplied D34 `2717d34b…` bound and capture deferred)
       or full `VALIDATED_PREFLIGHT_DEV_CAPTURE_AND_SEALED_D34_SCORE`, refusing any metadata-only
       claim of full reference `PASS`
   - `AttentionPlan/2`, `AttentionReview` (`review_payload_sha256`), `attention-check-plan`,
     and optional D40 `AttentionAdmission/2` + `AttentionOwner` (`Casey` personal release).
3. Dose Trajectory, Q1 Gate, D34 Paired-DEV Scorer & Precedence 0..6 Decision Table (`verify_dose_bundle`):
   - 45-coordinate interleaved schedule & control-lock ledger proof (`Counters::validate`,
     `1,130` refused relation proposals and `0` accepted relation updates on `FrozenZero` at
     16,404 batches, `275` at Q1 4,096 batches, and `gates[20:45] == 0`)
   - Q1 checkpoint (`4,096` batches) `Q1Decision` (`psucc_below_500`, `psucc_less_than_half_init`,
     `no_combine_or_relation_sensitivity`, `diagnostic_stop`) and non-resumable `Q1_DIAGNOSTIC_STOP`
   - Full-dose (`16,404` batches, `1,049,856` TRAIN target exposures) checkpoints, within-checkpoint
     zero-relation ablation (`uor-r4.attention-relation-off/1`), post-training `G-Occ` and `G-Adm`
   - D34 `uor-r4.attention-paired-dev-reference-score/1` verification (`B3_R`, `B3_C`, `B3_I >= 200`,
     `MDE_rel = 24 / B3_I`, paired 2x2 McNemar floors `F_I`, `F_{T,R}`, `F_{T,I}`, `F_A`, `F_C`, `F_ITT`,
     and same-cell `T2_R` / `T2_I` / `U_succ` conservation)
   - Strict Precedence 0..6 Decision Table (`INSTRUMENT_FAULT`, `UNDERPOWERED_OR_INVALID`,
     `H_null`, `H_control`, `H_mixed_INCONCLUSIVE`, `H_geom_DEV_PENDING_PAIRED_FRESH_TRANSFER`,
     `H_geom`, `DEV_ONLY_DIAGNOSTIC_FRESH_FAILED`) and conditional fresh-transfer gate requiring
     both DEV `H_geom` and `uor-r4.attention-transfer-decision/2` PM authorization.

Host-only; never creates a CUDA context, constructs a model, or alters a hold.
"""

from __future__ import annotations

import argparse
import contextlib
import copy
import hashlib
import io
import json
import math
import re
import struct
import sys
import tempfile
from pathlib import Path
from typing import Any

DRIVER_SOURCE_FILES: tuple[str, ...] = (
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

RELATION_QUALIFICATION_NAMES: list[str] = [
    "relation_off_flock_parity",
    "relation_copy_successor_fixture",
    "relation_irrelevant_change_invariance",
    "relation_conflicting_successors",
    "relation_suffix_and_target_isolation",
    "relation_endpoint_causal_boundary",
    "relation_deterministic_tie_order",
    "relation_bounded_sweeps_and_refusals",
    "active_leaf_initialization_and_matched_off_path",
]

EXPECTED_DRIVER_SOURCE_SHA256 = (
    "f673edb5d3bfea8bae44594adf122fbe937626c60c5be838d59a3ffc42f55ccf"
)
EXPECTED_ATTENTION_LEARNING_RS_SHA256 = (
    "21e8b5282b3ef2956d359c9518e9f0bf68c0d2d75e4febcc59514c1cacd8709f"
)
EXPECTED_ATTENTION_BASE_SOURCE = "ef9d30999700be11e103f96590e9e3b83b6807c7"
EXPECTED_SEEDED_SHEET_SHA256 = (
    "844814009ecc7368c0a3ecabf62434bae7d72850363e19f67c0126810fee7ffc"
)
EXPECTED_SEEDED_SHEET_TITLE = "# Pre-registration — M1 learned-attention link"
EXPECTED_INJECTIVE_SHEET_SHA256 = (
    "e47ce5afb3966bda13cd04bd9b99849b401c028e078de958fa98b5e8b7ed4efb"
)
EXPECTED_INJECTIVE_SHEET_TITLE = "# Learned attention link from an injective codebook"
EXPECTED_A8_SHEET_SHA256 = (
    "3e622bebd7c468c6cd3d15a38486768c7902807998f71cae039a4341ab55da2b"
)
EXPECTED_A8_SHEET_BYTES = 12697
EXPECTED_A8_SHEET_TITLE = (
    "# Learned attention link from an address-structured injective codebook"
)

EXPECTED_PRIME_ZETA_TABLE_FILE_SHA256 = (
    "83c1380a8f507e51d0342faa1b089dea6a20561a03552208c093fa640afc124b"
)
EXPECTED_PRIME_ZETA_TABLE_WORDS_SHA256 = (
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
EXPECTED_COUNTS_SHA256 = (
    "755a5a81ddbf0f9d81b4d0bd88e54435b96f5485f87ad92a22d235651fe6ca44"
)

EXPECTED_A8_TOKEN0_BLOCK0: list[int] = [0, 0, 0, 0, 0, -2, 2, 0]
EXPECTED_NEGB_A8_TOKEN0_BLOCK0: list[int] = [0, 2, 2, 0, 0, 0, 0, 0]
EXPECTED_A8_TAIL_131064_WORDS_SHA256 = (
    "bbfa92244b530398f30d6abb4b9916191f8397e9eb8ec957450e6a35feaba769"
)
EXPECTED_NEGB_A8_FILE_SHA256 = (
    "e35bb48d800cd41e5e6b0b48d4b41b8b61fc147f4fde98c54e8dff919d15f90e"
)
EXPECTED_NEGB_A8_BYTES = 278778
EXPECTED_NEGB_A8_WORDS_SHA256 = (
    "f9e1ea54345d6e8528801484062a84c4db4747888529f05a278f0776c2cd2db5"
)

EXPECTED_TRAINING_CONFIGS_SHA256SUMS_DIGEST = (
    "8a72d6fd2a98b4ca52672d2e6595eefe1233cd2350599bd97038905b2d0560d7"
)
EXPECTED_RUNBOOK_SHA256SUMS_DIGEST = (
    "4293db2ea743201de13edf8ebf0b6eb0dfec78e11dbb3abd98955293ee1e95d5"
)
EXPECTED_RUNBOOK_README_SHA256 = (
    "9fcf48cf3c110b7f3c1523edad24dcb8a6ea4bc311bf0ed7ab4c63e4135d304a"
)
EXPECTED_RUNBOOK_README_BYTES = 17670

EXPECTED_A8_RELATION_CONFIG_FILE_SHA256 = (
    "469a2ff0fe17a6a80f5875a077b3e6a9436553efdf1c6e7884541220553e9d5e"
)
EXPECTED_A8_RELATION_CONFIG_BYTES = 1924
EXPECTED_A8_RELATION_CONFIG_SHA256 = (
    "f223d78f45fe26dee4080ae7ca9a5b9ff12d9ef0df847375671d52db733fbecd"
)
EXPECTED_A8_RELATION_LINEAGE_SHA256 = (
    "265c91d07cb59031a766efff92535772957c92556acca2a2bce2082a8a6f44de"
)

EXPECTED_A8_FROZEN_ZERO_CONFIG_FILE_SHA256 = (
    "cf7ebcc85d2e463ae8f6691b63d90a0e005701cb38b8249a8d0d5460d8508479"
)
EXPECTED_A8_FROZEN_ZERO_CONFIG_BYTES = 1930
EXPECTED_A8_FROZEN_ZERO_CONFIG_SHA256 = (
    "c3fbffe08d13008b8b7ab400a4ef946f2c49632906d88e431d5e80314b9f5536"
)
EXPECTED_A8_FROZEN_ZERO_LINEAGE_SHA256 = (
    "3f3f4816816b2c529882f6b3edbb0a68c7f16d51e8c228ba04e346f9ed439cf5"
)

EXPECTED_UNPINNED_RELATION_CONFIG_FILE_SHA256 = (
    "52b32dc8747148e0cd385d309dbfb1c3cc750569d11ff3932745313e134cc670"
)
EXPECTED_UNPINNED_RELATION_CONFIG_BYTES = 1659
EXPECTED_UNPINNED_FROZEN_ZERO_CONFIG_FILE_SHA256 = (
    "6a035717130a5e7106e19ca585b4de4383506b425ce403d8a13d7ab44622e1cf"
)
EXPECTED_UNPINNED_FROZEN_ZERO_CONFIG_BYTES = 1665

EXPECTED_INJECTIVE_RELATION_CONFIG_FILE_SHA256 = (
    "7927b1ddde3025d947ce56166c0a30e3ad8d310211aaab1df80851a4ae611bda"
)
EXPECTED_INJECTIVE_FROZEN_ZERO_CONFIG_FILE_SHA256 = (
    "568adfd0df9941f3c58aa6d102c394bddd41ec05c7015097381e06514605ee09"
)

EXPECTED_D34_RECORD_SHA256 = (
    "2717d34bcfcdd1486991925ecab066266a6b56ba34b89727c37f78c821fe3bec"
)
EXPECTED_D34_RECORD_BYTES = 5519089
EXPECTED_ORDERED_ROW_BINDING_SHA256 = (
    "e4ef44fb81d6c3a833212e4857117db71cafb8259b81c286e5b1391129d74446"
)

EXPECTED_TRAIN_TOKENS_SHA256 = (
    "3dee858f84b750f507ceb3dc3f393711741950954735efaaf61bf704c017a4c8"
)
EXPECTED_TRAIN_MASK_SHA256 = (
    "b5e7352d67d6883a833155a5fd397a03f6d4f1b2649e3d4ec3e2ea7d95b949e3"
)
EXPECTED_DEV_TOKENS_SHA256 = (
    "0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b"
)
EXPECTED_DEV_MASK_SHA256 = (
    "a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078"
)
EXPECTED_TOKENIZER_SHA256 = (
    "d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89"
)
EXPECTED_FRESH_V2_TOKENS_SHA256 = (
    "0b7e754cfd242bf53145fe9cbca563be8503e6cd75df5449abb325b5c51ee1b8"
)
EXPECTED_FRESH_V2_MASK_SHA256 = (
    "a1cda5295547bb641b1ea71ba4ebabb58d58c40e6c048cda91f35a878fecbf5e"
)

EXPECTED_PACKET_BOUND_INPUTS: dict[str, tuple[str, int]] = {
    "sheet.md": (EXPECTED_A8_SHEET_SHA256, EXPECTED_A8_SHEET_BYTES),
    "a8-frozen-zero-seed1.json": (
        EXPECTED_A8_FROZEN_ZERO_CONFIG_FILE_SHA256,
        EXPECTED_A8_FROZEN_ZERO_CONFIG_BYTES,
    ),
    "a8-relation-seed1.json": (
        EXPECTED_A8_RELATION_CONFIG_FILE_SHA256,
        EXPECTED_A8_RELATION_CONFIG_BYTES,
    ),
    "d34-successor-table-record.json": (
        EXPECTED_D34_RECORD_SHA256,
        EXPECTED_D34_RECORD_BYTES,
    ),
    "neg-b-a8-nonroot-table.json": (
        EXPECTED_NEGB_A8_FILE_SHA256,
        EXPECTED_NEGB_A8_BYTES,
    ),
    "unpinned-frozen-zero-seed1.json": (
        EXPECTED_UNPINNED_FROZEN_ZERO_CONFIG_FILE_SHA256,
        EXPECTED_UNPINNED_FROZEN_ZERO_CONFIG_BYTES,
    ),
    "unpinned-relation-seed1.json": (
        EXPECTED_UNPINNED_RELATION_CONFIG_FILE_SHA256,
        EXPECTED_UNPINNED_RELATION_CONFIG_BYTES,
    ),
}

EXPECTED_A8_CP0_PSUCC_ADMITTED = 1351
EXPECTED_A8_CP0_FULL_ADMITTED = 4244
EXPECTED_A8_CP0_COMP_ADMITTED = 2893
EXPECTED_A8_CP0_FULL_TOP1 = 133
CANONICAL_DEV_FULL_ROWS = 14287
CANONICAL_DEV_PSUCC_ROWS = 3847
CANONICAL_DEV_COMP_ROWS = 10440
CANONICAL_ATTENTION_BATCHES = 16404
CANONICAL_Q1_BATCHES = 4096
CANONICAL_TRAIN_MATCHED_TARGETS = 1049856

EXPECTED_ACTIVE_LEAF_GATES: list[int] = [1, 0, 0, 0, 0] * 4 + [0] * 25
EXPECTED_MODEL_ROUTES: list[int] = [0] * 20
U64_MAX = (1 << 64) - 1

# Verified vast-56670997 pre-run gate, plan, review, and D40 release digests (#2029)
EXPECTED_VAST_56670997_GATES_LOG_SHA256 = (
    "4ec8411f728c9076d7d9cf214a3a5ee81b0036ba947605a56057e9cb362c4d83"
)
EXPECTED_VAST_56670997_GATES_LOG_BYTES = 9746
EXPECTED_VAST_56670997_RELATION_CONTROLS_SHA256 = (
    "e753e0031420d6e924ce42783b43123e4cfac83e8f172cc3dd4c83a5887caf35"
)
EXPECTED_VAST_56670997_RELATION_CONTROLS_BYTES = 5165
EXPECTED_VAST_56670997_FREEZE_COMMIT = "2c015db9f01d3ba3a7d79889a052b1345d9da48f"
EXPECTED_VAST_56670997_ELF_SHA256 = (
    "ea2e13933fbf61b43b42c948bef91eac70ac76352c24019c64913cf72475f4e5"
)
EXPECTED_VAST_56670997_SOURCE_SHA256 = (
    "d962ec06c5d71827e63c668646cd77befce992d9d979350e690719833bf37a5c"
)
EXPECTED_VAST_56670997_KERNEL_SHA256 = (
    "5cd0e73241fd895f2dd0004c83af3d1eb9f89e00ec2e933c1acc44fc7127bfd6"
)
EXPECTED_VAST_56670997_GEOMETRY_SHA256 = (
    "46ddbb76b7fbe6ac6222ea24779fc10055b1d6cea6a8da786895676fd29fe327"
)
EXPECTED_VAST_56670997_REL_REVIEW_PAYLOAD_SHA256 = (
    "29a5264d9ac18ba219536c1d08d7cf7f2520f5fd5fc902692eb528a2ce61665e"
)
EXPECTED_VAST_56670997_FZ_REVIEW_PAYLOAD_PREFIX_SHA256 = (
    "bd6b1481b0d32603a5ab5f33fc57164eb92001fa2c1e0d61787408172d9c8a66"
)
EXPECTED_VAST_56670997_FZ_REVIEW_PAYLOAD_SHA256 = (
    "d03089912cdedbd7c03c81a2d49d9a310627a736eaa75f4bad7983e259ba80f5"
)
EXPECTED_VAST_56670997_REL_REVIEW_FILE_SHA256 = (
    "17f4abce7579abf13d87a8b802efaefb1316210ddd9fa3297d8d32c10ac277e4"
)
EXPECTED_VAST_56670997_FZ_REVIEW_FILE_SHA256 = (
    "0d03541233c7fe04922cb1ac39d8371b12d703b23f52dbf41c8700edc72784aa"
)
EXPECTED_VAST_56670997_REL_PLAN_RAW_SHA256 = (
    "c97649e388b13f6a542c165b3bcdd0bbe37715787abfece6f392d9d230bf230d"
)
EXPECTED_VAST_56670997_FZ_PLAN_RAW_SHA256 = (
    "faed2c0d07cea6ba23070e6bd348426c038ebd0006fa3f55b13fe058a80f179d"
)
EXPECTED_VAST_56670997_REL_PLAN_CANONICAL_SHA256 = (
    "ac858a31a0af6d562bec883318240ba3ffeacd96773249150c1a69ed4ac10c26"
)
EXPECTED_VAST_56670997_FZ_PLAN_CANONICAL_SHA256 = (
    "7777bc0c37a8f91407fae18e5fa778da895e963c998b78a7b1b7189f56f99067"
)
EXPECTED_VAST_56670997_REL_OWNER_SHA256 = (
    "f392a260064d7462b9971330d741999f945cc8d55824954973cabddc360638c3"
)
EXPECTED_VAST_56670997_FZ_OWNER_SHA256 = (
    "1e74a6e52acde9d006816e276f26ed11a07603e6816fc6739a2766b250d61c8d"
)
EXPECTED_VAST_56670997_REL_ADMISSION_SHA256 = (
    "d06174f68cff121b5271b01c5fed962a4b30ed6ba978a3ec795b5aba6f00f57f"
)
EXPECTED_VAST_56670997_FZ_ADMISSION_SHA256 = (
    "b8a2845a41f0eda97d27173a7c41dfc9fb2456b7daaf99d7c8972fae7ffc7505"
)

CLAIM_SCOPE_H_GEOM = (
    "learned_copy_successor_link@Relation+InterleavedReadEvery8V1+ActiveLeafV1+V4096/B4/C192"
)

HEX64_RE = re.compile(r"^[0-9a-f]{64}$")
HEX40_RE = re.compile(r"^[0-9a-f]{40}$")
RECEIPT_URL_RE = re.compile(
    r"^https://github\.com/UOR-Foundation/uor-r4/(issues|pull)/\d+#issuecomment-\d+$"
)
PM_TRANSFER_RECEIPT_URL_RE = re.compile(
    r"^https://github\.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-\d+$"
)


class VerificationError(ValueError):
    """Raised when an A8 runbook, packet, pre-run gate, or dose artifact fails verification."""


def require(condition: bool, message: str) -> None:
    if not condition:
        raise VerificationError(message)


def is_u64(value: Any) -> bool:
    return isinstance(value, int) and not isinstance(value, bool) and value >= 0


def is_hex64(value: Any) -> bool:
    return isinstance(value, str) and bool(HEX64_RE.match(value))


def is_hex40(value: Any) -> bool:
    return isinstance(value, str) and bool(HEX40_RE.match(value))


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


def paired_integer_floor(mde_floor: int, discordant_sum: int) -> int:
    """Exact integer ceiling of `max(mde_floor, 2 * sqrt(b + c))` matching `attention_reference.rs`."""
    require(is_u64(mde_floor) and is_u64(discordant_sum), "floor inputs must be non-negative ints")
    target_sq = 4 * discordant_sum
    k = mde_floor
    while k * k < target_sq:
        k += 1
    return k


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


def registered_sheet_for_config(config: dict[str, Any]) -> tuple[str, str]:
    """Replicate `attention_learning::registered_sheet` (#2755)."""
    cb_init = config.get("codebook_initialization", {"kind": "seeded_device_v1"})
    kind = cb_init.get("kind") if isinstance(cb_init, dict) else None
    if kind in (None, "seeded_device_v1"):
        return EXPECTED_SEEDED_SHEET_SHA256, EXPECTED_SEEDED_SHEET_TITLE
    if kind == "pinned_table_v1":
        table = cb_init.get("table", {})
        sha = table.get("sha256") if isinstance(table, dict) else None
        if sha == EXPECTED_PRIME_ZETA_TABLE_FILE_SHA256:
            return EXPECTED_INJECTIVE_SHEET_SHA256, EXPECTED_INJECTIVE_SHEET_TITLE
        if sha == EXPECTED_A8_TABLE_FILE_SHA256:
            return EXPECTED_A8_SHEET_SHA256, EXPECTED_A8_SHEET_TITLE
        raise VerificationError(
            "attention binds a sheet only for a registered pinned table (PrimeZeta or A8)"
        )
    raise VerificationError(f"unsupported codebook_initialization.kind: {kind!r}")


def derive_scope_and_phrase(config: dict[str, Any]) -> tuple[str, str]:
    """Replicate `attention_learning::validate_config` + `attention_learning::scope` + D40 phrase."""
    spec = config.get("spec", {})
    require(
        spec == {"vocabulary": 4096, "blocks": 4, "context": 192, "kept_cells": 8},
        f"attention requires V4096/B4/C192/K8 spec, got {spec!r}",
    )
    read_spec = config.get("read_spec", {})
    if read_spec == {"kind": "flock", "kept": 8}:
        arm = "flock"
    elif read_spec == {"kind": "relation", "kept": 8, "sweeps": 1}:
        policy = config.get("relation_learning_policy", "learn_v1")
        if policy == "frozen_zero_v1":
            arm = "relation-frozen-zero"
        elif policy == "learn_v1":
            arm = "relation"
        else:
            raise VerificationError(f"unsupported relation_learning_policy: {policy!r}")
    else:
        raise VerificationError(f"attention admits only Flock8 or Relation8/sweep1, got {read_spec!r}")

    require(
        config.get("objective_policy", "nearest_address_margin_v1") == "nearest_address_margin_v1"
        and config.get("schedule") == "interleaved_read_every8_v1"
        and config.get("read_initialization") == "active_leaf_v1",
        "attention requires nearest_address_margin_v1, active_leaf_v1, and interleaved_read_every8_v1",
    )
    seed = config.get("seed")
    require(seed in (1, 2), f"attention requires seed 1 or 2, got {seed!r}")

    cb_init = config.get("codebook_initialization", {"kind": "seeded_device_v1"})
    kind = cb_init.get("kind") if isinstance(cb_init, dict) else None
    if kind in (None, "seeded_device_v1"):
        lineage = ""
    elif kind == "pinned_table_v1":
        table_sha = cb_init.get("table", {}).get("sha256", "")
        require(is_hex64(table_sha), f"invalid pinned table sha256: {table_sha!r}")
        lineage = f"injective-{table_sha[:12]}-"
    else:
        raise VerificationError(f"unsupported codebook_initialization: {cb_init!r}")

    scope = f"m1-attention-learning-{lineage}{arm}-k8-seed{seed}-batches0..16404"
    phrase = (
        f"I am Casey; release D4 for {scope} only through batch 16404 "
        "with the pinned DEV transfer and relation-off evaluations"
    )
    return scope, phrase


def expected_cost_contract(config: dict[str, Any]) -> dict[str, int]:
    """Replicate `AttentionCostContract::expected` in `attention_learning.rs`."""
    read_spec = config.get("read_spec", {})
    is_relation = isinstance(read_spec, dict) and read_spec.get("kind") == "relation"
    is_learn = config.get("relation_learning_policy", "learn_v1") == "learn_v1"
    return {
        "max_batches": CANONICAL_ATTENTION_BATCHES,
        "checkpoint_every_batches": 64,
        "baseline_dev_evaluations": 1,
        "q1_dev_evaluations": 1,
        "final_dev_evaluations": 1,
        "relation_off_dev_evaluations": 1 if (is_relation and is_learn) else 0,
        "final_transfer_evaluations": 1 if is_relation else 0,
        "relation_off_transfer_evaluations": 0,
    }


def review_payload_sha256(plan: dict[str, Any]) -> str:
    """Replicate `attention_learning::review_payload_sha256` (serde_json::Value BTreeMap key order)."""
    payload = copy.deepcopy(plan)
    payload.pop("independent_review", None)
    payload["config"] = canonical_driver_config(payload["config"])
    encoded = json.dumps(payload, separators=(",", ":"), sort_keys=True).encode("utf-8")
    return hashlib.sha256(encoded).hexdigest()


def canonical_plan_dict(plan: dict[str, Any]) -> dict[str, Any]:
    """Return `AttentionPlan` dict in exact Rust struct declaration order (`controller.rs:81-93` & `attention_learning.rs:104-118`)."""
    return {
        "schema": plan["schema"],
        "status": plan["status"],
        "scope": plan["scope"],
        "execution": {
            "base_source": plan["execution"]["base_source"],
            "reviewed_source_commit": plan["execution"]["reviewed_source_commit"],
            "source_sha256": plan["execution"]["source_sha256"],
            "kernel_sha256": plan["execution"]["kernel_sha256"],
            "geometry_sha256": plan["execution"]["geometry_sha256"],
            "executable_sha256": plan["execution"]["executable_sha256"],
            "driver_source_sha256": plan["execution"]["driver_source_sha256"],
            "config_sha256": plan["execution"]["config_sha256"],
            "lineage_sha256": plan["execution"]["lineage_sha256"],
            "sheet_sha256": plan["execution"]["sheet_sha256"],
        },
        "config": canonical_driver_config(plan["config"]),
        "sheet": {"path": plan["sheet"]["path"], "sha256": plan["sheet"]["sha256"]},
        "independent_review": {
            "path": plan["independent_review"]["path"],
            "sha256": plan["independent_review"]["sha256"],
        },
        "transfer": {
            "split": plan["transfer"]["split"],
            "tokens": {
                "path": plan["transfer"]["tokens"]["path"],
                "sha256": plan["transfer"]["tokens"]["sha256"],
            },
            "mask": {
                "path": plan["transfer"]["mask"]["path"],
                "sha256": plan["transfer"]["mask"]["sha256"],
            },
            "expected_payload_tokens": plan["transfer"]["expected_payload_tokens"],
            "expected_eligible_targets": plan["transfer"]["expected_eligible_targets"],
        },
        "run_state_directory": plan["run_state_directory"],
        "cost_contract": {
            "max_batches": plan["cost_contract"]["max_batches"],
            "checkpoint_every_batches": plan["cost_contract"]["checkpoint_every_batches"],
            "baseline_dev_evaluations": plan["cost_contract"]["baseline_dev_evaluations"],
            "q1_dev_evaluations": plan["cost_contract"]["q1_dev_evaluations"],
            "final_dev_evaluations": plan["cost_contract"]["final_dev_evaluations"],
            "relation_off_dev_evaluations": plan["cost_contract"]["relation_off_dev_evaluations"],
            "final_transfer_evaluations": plan["cost_contract"]["final_transfer_evaluations"],
            "relation_off_transfer_evaluations": plan["cost_contract"]["relation_off_transfer_evaluations"],
        },
    }


def canonical_plan_sha256(plan: dict[str, Any]) -> str:
    """Compute `sha256(encode(&plan))` matching `AttentionPlan` struct field order."""
    ordered = canonical_plan_dict(plan)
    return hashlib.sha256(json.dumps(ordered, separators=(",", ":")).encode("utf-8")).hexdigest()


def canonical_admission_sha256(admission: dict[str, Any]) -> str:
    """Compute `sha256(encode(&admission))` (`manifest_sha256`) matching `AttentionAdmission` in `attention_learning.rs:121-125`."""
    ordered = {
        "schema": admission["schema"],
        "plan": canonical_plan_dict(admission["plan"]),
        "owner_release": {
            "path": admission["owner_release"]["path"],
            "sha256": admission["owner_release"]["sha256"],
        },
    }
    return hashlib.sha256(json.dumps(ordered, separators=(",", ":")).encode("utf-8")).hexdigest()


def parse_sha256sums_file(sha_sums_path: Path, expected_digest: str | None = None) -> dict[str, str]:
    """Parse and verify a `SHA256SUMS` manifest (forbidding self-entries)."""
    raw_bytes = read_bytes_checked(sha_sums_path, "SHA256SUMS")
    digest = hashlib.sha256(raw_bytes).hexdigest()
    if expected_digest is not None:
        require(
            digest == expected_digest,
            f"{sha_sums_path} digest {digest} != {expected_digest}",
        )
    entries: dict[str, str] = {}
    base_dir = sha_sums_path.parent
    for line in raw_bytes.decode("utf-8").splitlines():
        line = line.strip()
        if not line:
            continue
        parts = line.split()
        require(len(parts) == 2, f"malformed SHA256SUMS line in {sha_sums_path}: {line!r}")
        file_sha, rel_name = parts
        require(rel_name != "SHA256SUMS", f"{sha_sums_path} must not contain a self-entry")
        require(is_hex64(file_sha), f"invalid SHA-256 in {sha_sums_path}: {file_sha!r}")
        require(rel_name not in entries, f"duplicate entry in {sha_sums_path}: {rel_name}")
        target = base_dir / rel_name
        actual_bytes = read_bytes_checked(target, f"pinned file {rel_name}")
        actual_sha = hashlib.sha256(actual_bytes).hexdigest()
        require(
            actual_sha == file_sha,
            f"SHA-256 mismatch for {target}: {actual_sha} != {file_sha}",
        )
        entries[rel_name] = file_sha
    return entries


def verify_neg_b_a8_table(negb_path: Path) -> dict[str, Any]:
    """Verify `neg-b-a8-nonroot-table.json` is built from A8 with token-0 block-0 mutated to a non-root."""
    raw = read_bytes_checked(negb_path, "neg-b-a8-nonroot-table.json")
    file_sha = hashlib.sha256(raw).hexdigest()
    require(
        file_sha == EXPECTED_NEGB_A8_FILE_SHA256 and len(raw) == EXPECTED_NEGB_A8_BYTES,
        f"neg-b-a8-nonroot-table.json file SHA/bytes mismatch: {file_sha} ({len(raw)} B)",
    )
    doc = json.loads(raw.decode("utf-8"))
    require(doc.get("schema") == "uor-r4.codebook-table/1", "neg-b schema mismatch")
    require(doc.get("mode") == "address_structured_v1", "neg-b mode must be address_structured_v1")
    require(
        doc.get("spec") == {"vocabulary": 4096, "blocks": 4, "context": 192, "kept_cells": 8},
        "neg-b spec mismatch",
    )
    words = doc.get("table")
    require(
        isinstance(words, list) and len(words) == 4096 * 32,
        "neg-b table must have 131,072 words",
    )
    computed_word_sha = hashlib.sha256(struct.pack(f"<{len(words)}i", *words)).hexdigest()
    require(
        doc.get("table_sha256") == EXPECTED_NEGB_A8_WORDS_SHA256
        and computed_word_sha == EXPECTED_NEGB_A8_WORDS_SHA256,
        f"neg-b table_sha256 mismatch: declared={doc.get('table_sha256')}, computed={computed_word_sha}",
    )
    distinct = len({tuple(words[i : i + 32]) for i in range(0, len(words), 32)})
    require(
        doc.get("distinct_vectors") == 4096 and distinct == 4096,
        f"neg-b distinct_vectors must be 4096, got {distinct}",
    )
    block0 = words[:8]
    require(
        block0 == EXPECTED_NEGB_A8_TOKEN0_BLOCK0 and block0 not in ROOTS,
        f"neg-b token-0 block-0 must be non-root {EXPECTED_NEGB_A8_TOKEN0_BLOCK0}, got {block0}",
    )
    tail_sha = hashlib.sha256(struct.pack("<131064i", *words[8:])).hexdigest()
    require(
        tail_sha == EXPECTED_A8_TAIL_131064_WORDS_SHA256,
        f"neg-b tail words[8:] SHA-256 mismatch against A8: {tail_sha}",
    )
    reconstructed_a8 = EXPECTED_A8_TOKEN0_BLOCK0 + words[8:]
    reconstructed_a8_sha = hashlib.sha256(
        struct.pack("<131072i", *reconstructed_a8)
    ).hexdigest()
    require(
        reconstructed_a8_sha == EXPECTED_A8_TABLE_WORDS_SHA256,
        f"restoring A8 block-0 failed to reproduce A8 table_sha256: {reconstructed_a8_sha}",
    )
    return {
        "file_sha256": file_sha,
        "bytes": len(raw),
        "table_sha256": computed_word_sha,
        "distinct_vectors": distinct,
        "token0_block0": block0,
        "reconstructed_a8_table_sha256": reconstructed_a8_sha,
    }


def verify_executor_packet(packet_dir: Path, repo_root: Path | None = None) -> dict[str, Any]:
    """Verify `/Users/casey.allard/uor-r4-local/a8-dose-executor-packet-20261019` bound inputs, docs, and preserved pre-run artifacts."""
    sha_sums_path = packet_dir / "bound-inputs" / "SHA256SUMS"
    pins = parse_sha256sums_file(sha_sums_path)
    require(
        set(pins.keys()) == set(EXPECTED_PACKET_BOUND_INPUTS.keys()),
        f"packet bound-inputs keys mismatch: {sorted(pins.keys())}",
    )
    byte_counts: dict[str, int] = {}
    for name, (expected_sha, expected_bytes) in EXPECTED_PACKET_BOUND_INPUTS.items():
        require(
            pins[name] == expected_sha,
            f"packet pin {name} SHA mismatch: {pins[name]} != {expected_sha}",
        )
        fpath = packet_dir / "bound-inputs" / name
        actual_size = fpath.stat().st_size
        require(
            actual_size == expected_bytes,
            f"packet pin {name} byte size {actual_size} != {expected_bytes}",
        )
        byte_counts[name] = actual_size

    inherited_md = read_text_checked(packet_dir / "INHERITED-CHECKS.md", "INHERITED-CHECKS.md")
    status_md = read_text_checked(packet_dir / "STATUS.md", "STATUS.md")
    owner_md = read_text_checked(
        packet_dir / "OWNER-RELEASE-READINESS.md", "OWNER-RELEASE-READINESS.md"
    )
    for required_token in (
        "both_probes_any_active_candidate_cost_differs_v1",
        "qualify-relation",
        "active_leaf_initialization_and_matched_off_path",
        "attention-cell-reference-scorer preflight-dev",
        "attention-cell-reference-scorer paired-dev",
        EXPECTED_D34_RECORD_SHA256,
    ):
        require(
            required_token in inherited_md,
            f"packet INHERITED-CHECKS.md missing required token: {required_token}",
        )
    require(
        "DEFERRED_TO_DOSE" in status_md and EXPECTED_D34_RECORD_SHA256 in status_md,
        "packet STATUS.md must bind DEFERRED_TO_DOSE and D34 SHA-256",
    )
    rel_cfg = read_json_checked(
        packet_dir / "bound-inputs" / "a8-relation-seed1.json", "packet a8-relation-seed1.json"
    )
    fz_cfg = read_json_checked(
        packet_dir / "bound-inputs" / "a8-frozen-zero-seed1.json",
        "packet a8-frozen-zero-seed1.json",
    )
    _, rel_phrase = derive_scope_and_phrase(rel_cfg)
    _, fz_phrase = derive_scope_and_phrase(fz_cfg)
    require(
        rel_phrase in owner_md and fz_phrase in owner_md,
        "packet OWNER-RELEASE-READINESS.md missing exact treatment/control D40 phrases",
    )
    vast_prerun = None
    gates_log_path = packet_dir / "init-lock-selector-diagnosis" / "original-gates.log"
    rel_ctrl_path = packet_dir / "init-lock-selector-diagnosis" / "original-relation-controls.json"
    if repo_root is not None and gates_log_path.is_file() and rel_ctrl_path.is_file():
        vast_prerun = verify_vast_56670997_prerun_artifacts(
            gates_log_path=gates_log_path,
            relation_controls_path=rel_ctrl_path,
            repo_root=repo_root,
        )
    return {
        "status": "PASS",
        "bound_inputs_verified": len(pins),
        "pins": pins,
        "byte_counts": byte_counts,
        "vast_56670997_prerun": vast_prerun,
    }


def verify_staged_contract(
    repo_root: Path,
    runbook_dir: Path | None = None,
    configs_dir: Path | None = None,
    packet_dir: Path | None = None,
) -> dict[str, Any]:
    """Verify the FIT sheet, source registration, staged configs, runbook, and optional executor packet."""
    runbook_dir = runbook_dir or (repo_root / "docs" / "labs" / "a8-dose-runbook-2026-10-19")
    configs_dir = configs_dir or (repo_root / "docs" / "labs" / "a8-training-configs-2026-10-19")

    # 1. Verify sheet
    sheet_path = (
        repo_root / "docs" / "harness" / "prep" / "m1-attention-a8-address-cells-learning-2026-10-19.md"
    )
    sheet_bytes = read_bytes_checked(sheet_path, "A8 learning sheet")
    sheet_sha = hashlib.sha256(sheet_bytes).hexdigest()
    require(
        sheet_sha == EXPECTED_A8_SHEET_SHA256 and len(sheet_bytes) == EXPECTED_A8_SHEET_BYTES,
        f"A8 sheet SHA/size mismatch: {sheet_sha} ({len(sheet_bytes)} B)",
    )
    sheet_text = sheet_bytes.decode("utf-8")
    require(
        sheet_text.startswith(EXPECTED_A8_SHEET_TITLE)
        and not sheet_text.startswith(EXPECTED_INJECTIVE_SHEET_TITLE),
        "A8 sheet title prefix mismatch",
    )
    require(
        any(line.startswith("TEST FITNESS: FIT") for line in sheet_text.splitlines()),
        "A8 sheet missing TEST FITNESS: FIT line",
    )

    # 2. Verify float-free constructor and 11-file driver_source_sha256
    for rel_path in (
        "crates/uor-r4-discrete/src/address_cells.rs",
        "crates/uor-r4-discrete/src/bin/address-cells-codebook.rs",
    ):
        code_text = read_text_checked(repo_root / rel_path, rel_path)
        require(
            not re.search(r"\b(f32|f64)\b", code_text),
            f"{rel_path} contains forbidden f32/f64 token",
        )

    src_dir = repo_root / "crates" / "uor-r4-discrete" / "src"
    driver_hasher = hashlib.sha256()
    for fname in DRIVER_SOURCE_FILES:
        driver_hasher.update(read_bytes_checked(src_dir / fname, fname))
    driver_sha = driver_hasher.hexdigest()
    require(
        driver_sha == EXPECTED_DRIVER_SOURCE_SHA256,
        f"driver_source_sha256 {driver_sha} != {EXPECTED_DRIVER_SOURCE_SHA256}",
    )

    attn_rs_bytes = read_bytes_checked(src_dir / "attention_learning.rs", "attention_learning.rs")
    attn_rs_sha = hashlib.sha256(attn_rs_bytes).hexdigest()
    require(
        attn_rs_sha == EXPECTED_ATTENTION_LEARNING_RS_SHA256,
        f"attention_learning.rs SHA {attn_rs_sha} != {EXPECTED_ATTENTION_LEARNING_RS_SHA256}",
    )
    attn_rs_text = attn_rs_bytes.decode("utf-8")
    for token in (
        f'pub const A8_SHEET_SHA256: &str =\n    "{EXPECTED_A8_SHEET_SHA256}";',
        f'const A8_TABLE_FILE_SHA256: &str =\n    "{EXPECTED_A8_TABLE_FILE_SHA256}";',
        f'const PRIME_ZETA_TABLE_FILE_SHA256: &str =\n    "{EXPECTED_PRIME_ZETA_TABLE_FILE_SHA256}";',
        '"attention binds a sheet only for a registered pinned table (PrimeZeta or A8)"',
    ):
        require(token in attn_rs_text, f"attention_learning.rs missing expected binding: {token}")

    # 3. Verify staged training configs & single-key diffs
    cfg_pins = parse_sha256sums_file(
        configs_dir / "evidence" / "SHA256SUMS",
        EXPECTED_TRAINING_CONFIGS_SHA256SUMS_DIGEST,
    )
    require(
        cfg_pins
        == {
            "a8-frozen-zero-seed1.json": EXPECTED_A8_FROZEN_ZERO_CONFIG_FILE_SHA256,
            "a8-relation-seed1.json": EXPECTED_A8_RELATION_CONFIG_FILE_SHA256,
        },
        f"staged training config pins mismatch: {cfg_pins}",
    )
    a8_rel = read_json_checked(
        configs_dir / "evidence" / "a8-relation-seed1.json", "a8-relation-seed1.json"
    )
    a8_fz = read_json_checked(
        configs_dir / "evidence" / "a8-frozen-zero-seed1.json", "a8-frozen-zero-seed1.json"
    )

    # Check byte-identity of a8-relation-seed1.json with pre-flight a8.json
    preflight_a8_path = (
        repo_root
        / "docs"
        / "labs"
        / "address-cells-preflight-configs-2026-10-19"
        / "evidence"
        / "a8.json"
    )
    require(
        read_bytes_checked(preflight_a8_path, "pre-flight a8.json")
        == read_bytes_checked(
            configs_dir / "evidence" / "a8-relation-seed1.json", "a8-relation-seed1.json"
        ),
        "a8-relation-seed1.json must be byte-identical to pre-flight a8.json",
    )

    rel_cfg_sha, rel_lin_sha = compute_config_and_lineage_sha(a8_rel, driver_sha)
    fz_cfg_sha, fz_lin_sha = compute_config_and_lineage_sha(a8_fz, driver_sha)
    require(
        rel_cfg_sha == EXPECTED_A8_RELATION_CONFIG_SHA256
        and rel_lin_sha == EXPECTED_A8_RELATION_LINEAGE_SHA256,
        f"a8-relation config/lineage SHA mismatch: {rel_cfg_sha} / {rel_lin_sha}",
    )
    require(
        fz_cfg_sha == EXPECTED_A8_FROZEN_ZERO_CONFIG_SHA256
        and fz_lin_sha == EXPECTED_A8_FROZEN_ZERO_LINEAGE_SHA256,
        f"a8-frozen-zero config/lineage SHA mismatch: {fz_cfg_sha} / {fz_lin_sha}",
    )

    # Single-key diffs against parent seeded and PrimeZeta injective configs
    inj_ev_dir = repo_root / "docs" / "labs" / "injective-attention-runbook-2026-10-19" / "evidence"
    parent_rel = read_json_checked(
        inj_ev_dir / "parent-rows-relation-seed1.json", "parent-rows-relation-seed1.json"
    )
    parent_fz = read_json_checked(
        inj_ev_dir / "parent-rows-frozen-zero-seed1.json", "parent-rows-frozen-zero-seed1.json"
    )
    inj_rel = read_json_checked(
        inj_ev_dir / "injective-relation-seed1.json", "injective-relation-seed1.json"
    )
    inj_fz = read_json_checked(
        inj_ev_dir / "injective-frozen-zero-seed1.json", "injective-frozen-zero-seed1.json"
    )

    expected_a8_cb_init = {
        "kind": "pinned_table_v1",
        "table": {
            "path": "/workspace/uor-r4/local/driver/data/address-cells/address-structured-v1.json",
            "sha256": EXPECTED_A8_TABLE_FILE_SHA256,
        },
    }
    for label, a8_cfg, par_cfg, inj_cfg in (
        ("relation", a8_rel, parent_rel, inj_rel),
        ("frozen-zero", a8_fz, parent_fz, inj_fz),
    ):
        require(
            a8_cfg.get("codebook_initialization") == expected_a8_cb_init,
            f"A8 {label} codebook_initialization mismatch",
        )
        stripped_a8 = copy.deepcopy(a8_cfg)
        del stripped_a8["codebook_initialization"]
        require(
            stripped_a8 == par_cfg,
            f"A8 {label} differs from parent-rows-{label}-seed1.json outside codebook_initialization",
        )
        rebound_inj = copy.deepcopy(inj_cfg)
        rebound_inj["codebook_initialization"] = expected_a8_cb_init
        require(
            rebound_inj == a8_cfg,
            f"A8 {label} differs from injective-{label}-seed1.json outside codebook_initialization",
        )
        require(
            registered_sheet_for_config(a8_cfg) == (EXPECTED_A8_SHEET_SHA256, EXPECTED_A8_SHEET_TITLE),
            f"registered_sheet mismatch for A8 {label}",
        )
        require(
            registered_sheet_for_config(par_cfg)
            == (EXPECTED_SEEDED_SHEET_SHA256, EXPECTED_SEEDED_SHEET_TITLE),
            f"registered_sheet mismatch for parent {label}",
        )
        require(
            registered_sheet_for_config(inj_cfg)
            == (EXPECTED_INJECTIVE_SHEET_SHA256, EXPECTED_INJECTIVE_SHEET_TITLE),
            f"registered_sheet mismatch for injective {label}",
        )

    # Between-arm single-key diff
    rel_as_fz = copy.deepcopy(a8_rel)
    rel_as_fz["relation_learning_policy"] = "frozen_zero_v1"
    require(
        rel_as_fz == a8_fz,
        "a8-relation-seed1.json and a8-frozen-zero-seed1.json must differ only in relation_learning_policy",
    )

    # S8 unregistered table refusal check
    s8_cfg = copy.deepcopy(a8_rel)
    s8_cfg["codebook_initialization"]["table"]["sha256"] = EXPECTED_S8_TABLE_FILE_SHA256
    try:
        registered_sheet_for_config(s8_cfg)
        raise VerificationError("S8 unregistered table must be refused by registered_sheet")
    except VerificationError as exc:
        require("registered pinned table" in str(exc), f"unexpected error: {exc}")

    # 4. Verify runbook directory, neg-b table, and README contract
    rb_pins = parse_sha256sums_file(
        runbook_dir / "evidence" / "SHA256SUMS",
        EXPECTED_RUNBOOK_SHA256SUMS_DIGEST,
    )
    require(
        rb_pins == {"neg-b-a8-nonroot-table.json": EXPECTED_NEGB_A8_FILE_SHA256},
        f"runbook evidence pins mismatch: {rb_pins}",
    )
    negb_summary = verify_neg_b_a8_table(runbook_dir / "evidence" / "neg-b-a8-nonroot-table.json")

    rb_readme_bytes = read_bytes_checked(runbook_dir / "README.md", "runbook README.md")
    rb_readme_sha = hashlib.sha256(rb_readme_bytes).hexdigest()
    require(
        rb_readme_sha == EXPECTED_RUNBOOK_README_SHA256
        and len(rb_readme_bytes) == EXPECTED_RUNBOOK_README_BYTES,
        f"runbook README.md SHA/bytes mismatch: {rb_readme_sha} ({len(rb_readme_bytes)} B)",
    )
    rb_readme_text = rb_readme_bytes.decode("utf-8")
    rel_scope, rel_phrase = derive_scope_and_phrase(a8_rel)
    fz_scope, fz_phrase = derive_scope_and_phrase(a8_fz)
    for token in (
        EXPECTED_A8_SHEET_SHA256,
        EXPECTED_A8_RELATION_CONFIG_FILE_SHA256,
        EXPECTED_A8_FROZEN_ZERO_CONFIG_FILE_SHA256,
        EXPECTED_UNPINNED_RELATION_CONFIG_FILE_SHA256,
        EXPECTED_UNPINNED_FROZEN_ZERO_CONFIG_FILE_SHA256,
        EXPECTED_PRIME_ZETA_TABLE_FILE_SHA256,
        EXPECTED_NEGB_A8_FILE_SHA256,
        EXPECTED_NEGB_A8_WORDS_SHA256,
        EXPECTED_D34_RECORD_SHA256,
        rel_scope,
        fz_scope,
        rel_phrase,
        fz_phrase,
        "both_probes_any_active_candidate_cost_differs_v1",
        "active_leaf_initialization_and_matched_off_path",
        "DEFERRED_TO_DOSE",
        "NOT_RUN pre-run",
        "input (b) is supplied by the executor packet",
    ):
        require(token in rb_readme_text, f"runbook README.md missing required token: {token!r}")

    packet_summary = None
    if packet_dir is not None:
        packet_summary = verify_executor_packet(packet_dir, repo_root=repo_root)

    committed_dose_summary = None
    dose_ev_dir = repo_root / "docs" / "labs" / "a8-dose-2026-10-19" / "evidence"
    if (dose_ev_dir / "SHA256SUMS").is_file():
        committed_dose_summary = verify_committed_a8_dose_receipt(repo_root)

    return {
        "status": "PASS",
        "sheet_sha256": sheet_sha,
        "driver_source_sha256": driver_sha,
        "attention_learning_rs_sha256": attn_rs_sha,
        "relation": {
            "config_file_sha256": EXPECTED_A8_RELATION_CONFIG_FILE_SHA256,
            "config_sha256": rel_cfg_sha,
            "lineage_sha256": rel_lin_sha,
            "scope": rel_scope,
            "d40_phrase": rel_phrase,
        },
        "frozen_zero": {
            "config_file_sha256": EXPECTED_A8_FROZEN_ZERO_CONFIG_FILE_SHA256,
            "config_sha256": fz_cfg_sha,
            "lineage_sha256": fz_lin_sha,
            "scope": fz_scope,
            "d40_phrase": fz_phrase,
        },
        "neg_b_a8": negb_summary,
        "runbook_readme_sha256": rb_readme_sha,
        "executor_packet": packet_summary,
        "committed_a8_dose_receipt": committed_dose_summary,
    }


def verify_probe_receipt(probe: dict[str, Any], expected_gate_index: int, arm_label: str) -> dict[str, Any]:
    """Verify Gate22 or Gate0 sensitivity probe receipt under `both_probes_any_active_candidate_cost_differs_v1`."""
    require(isinstance(probe, dict), f"{arm_label} gate {expected_gate_index} probe must be a dict")
    if "parameter" in probe:
        require(
            probe["parameter"] in (
                {"kind": "gate", "index": expected_gate_index},
                f"gate:{expected_gate_index}",
            ),
            f"{arm_label} gate {expected_gate_index} parameter mismatch: {probe['parameter']!r}",
        )
    require(
        probe.get("epoch") == 0 and probe.get("applied", False) is False,
        f"{arm_label} gate {expected_gate_index} probe requires epoch == 0 and applied == false",
    )
    cand_losses = probe.get("candidate_losses")
    require(
        isinstance(cand_losses, list)
        and len(cand_losses) == 16
        and all(is_u64(x) for x in cand_losses),
        f"{arm_label} gate {expected_gate_index} candidate_losses must be 16 u64 values",
    )
    before = probe.get("before")
    after = probe.get("after")
    cand = probe.get("candidate")
    require(
        is_u64(before)
        and is_u64(after)
        and is_u64(cand)
        and 0 <= cand < 16
        and before != U64_MAX
        and cand_losses[0] == before,
        f"{arm_label} gate {expected_gate_index} requires valid u64 before (== cand_losses[0]), after, and candidate in 0..16",
    )
    active_losses = [loss for loss in cand_losses if loss != U64_MAX]
    require(
        len(active_losses) >= 3,
        f"{arm_label} gate {expected_gate_index} requires at least 3 active (non-u64::MAX) candidate losses",
    )
    any_diff = probe.get("any_candidate_cost_differs_from_identity")
    computed_any_diff = any(loss != before for loss in active_losses)
    require(
        isinstance(any_diff, bool) and any_diff == computed_any_diff,
        f"{arm_label} gate {expected_gate_index} any_candidate_cost_differs_from_identity ({any_diff}) != computed ({computed_any_diff})",
    )
    literal_best_improves = (after < before) or (cand != 0)
    return {
        "gate_index": expected_gate_index,
        "before": before,
        "after": after,
        "candidate": cand,
        "active_candidates": len(active_losses),
        "any_candidate_cost_differs_from_identity": any_diff,
        "grandparent_literal_best_candidate_differs": literal_best_improves,
    }


def _is_zero_cursor(cursor: Any) -> bool:
    """Accept canonical 5-field `DriverCursor` (`attention_reference.rs:372-378`) or 3-field shorthand at checkpoint 0."""
    return cursor in (
        {
            "completed_batches": 0,
            "accepted_updates": 0,
            "corpus_pass": 0,
            "next_target": 0,
            "next_coordinate": 0,
        },
        {
            "completed_batches": 0,
            "accepted_updates": 0,
            "next_target_ordinal": 0,
        },
    )


def _extract_census_cells(census: dict[str, Any]) -> tuple[Any, Any, Any, bool]:
    """Extract `(occupied_cells, cell_occupancy_histogram, cell_collision_pairs, cells_status_ok)` from canonical `uor-r4.coverage-census/1` (`coverage_census.rs:97-173`) or flat shorthand."""
    if isinstance(census.get("cells"), dict):
        cells = census["cells"]
        status_ok = cells.get("status") in (None, "FROM_GPU_TOKEN_CELL_RECEIPT")
        return (
            cells.get("occupied_cells"),
            cells.get("cell_occupancy_histogram"),
            cells.get("cell_collision_pairs"),
            status_ok,
        )
    return (
        census.get("occupied_cells"),
        census.get("cell_occupancy_histogram"),
        census.get("cell_collision_pairs"),
        True,
    )


def verify_relation_controls(rel_controls: dict[str, Any]) -> dict[str, Any]:
    """Verify `qualify-relation` `relation-controls.json` (9/9 PASS, C2 ACTIVE, C2 INERT, and INIT LOCK)."""
    require(
        rel_controls.get("schema") == "uor-r4.discrete-spiralcore-fbs/2"
        and rel_controls.get("suite") == "relation"
        and rel_controls.get("status") == "PASS"
        and rel_controls.get("registered") == 9
        and rel_controls.get("attempted") == 9
        and rel_controls.get("passed") == 9
        and rel_controls.get("failed") == 0
        and rel_controls.get("not_run") == 0
        and rel_controls.get("applied_updates") == 0
        and rel_controls.get("training") == "HELD",
        "relation-controls.json header fields failed verification",
    )
    controls = rel_controls.get("controls")
    require(
        isinstance(controls, list) and len(controls) == 9,
        "relation-controls.json must contain 9 control records",
    )
    for idx, (ctrl, expected_name) in enumerate(zip(controls, RELATION_QUALIFICATION_NAMES), start=1):
        require(
            ctrl.get("index") == idx
            and ctrl.get("name") == expected_name
            and ctrl.get("status") == "PASS",
            f"relation-controls[{idx}] ({expected_name}) failed: {ctrl!r}",
        )
    ctrl9_details = controls[8].get("details")
    require(
        isinstance(ctrl9_details, dict) and isinstance(ctrl9_details.get("init_locks"), list),
        "control 9 (active_leaf_initialization_and_matched_off_path) missing details.init_locks",
    )
    init_locks = ctrl9_details["init_locks"]
    require(len(init_locks) == 2, "control 9 init_locks must have 2 seed records (seeds 1 and 2)")
    seen_seeds: list[int] = []
    for lock in init_locks:
        require(
            lock.get("control") == "active_leaf_initialization_and_matched_off_path"
            and lock.get("part") == "INIT LOCK"
            and lock.get("spec")
            == {"vocabulary": 4096, "blocks": 4, "context": 192, "kept_cells": 8}
            and lock.get("read_spec") == {"kind": "relation", "kept": 8, "sweeps": 1}
            and lock.get("initialization") == "active_leaf_v1"
            and lock.get("plus_one_combine_indices") == [0, 5, 10, 15]
            and lock.get("gates") == EXPECTED_ACTIVE_LEAF_GATES
            and lock.get("treatment_control_snapshot_bit_identical") is True
            and lock.get("reload_bit_identical") is True
            and lock.get("applied_updates") == 0
            and is_hex64(lock.get("snapshot_sha256"))
            and is_hex64(lock.get("model_storage_sha256"))
            and is_hex64(lock.get("model_snapshot_sha256")),
            f"invalid INIT LOCK record in relation-controls.json: {lock!r}",
        )
        seen_seeds.append(lock.get("seed"))
    require(seen_seeds == [1, 2], f"INIT LOCK seeds must be [1, 2], got {seen_seeds}")
    return {
        "status": "PASS",
        "registered": 9,
        "passed": 9,
        "c2_active": "PASS",
        "c2_inert": "PASS",
        "init_lock_seeds": seen_seeds,
    }


def verify_plans_and_releases(
    bundle: dict[str, Any],
    driver_sha: str,
    require_owner_release: bool = False,
) -> dict[str, Any]:
    """Verify `AttentionPlan/2`, `AttentionReview`, `check-plan`, and D40 `AttentionAdmission/2`."""
    plans_block = bundle.get("plans")
    require(isinstance(plans_block, dict), "bundle missing plans block")
    out: dict[str, Any] = {}
    state_dirs: list[str] = []

    for arm_key, expected_cfg_sha, expected_lin_sha in (
        ("relation", EXPECTED_A8_RELATION_CONFIG_SHA256, EXPECTED_A8_RELATION_LINEAGE_SHA256),
        ("frozen_zero", EXPECTED_A8_FROZEN_ZERO_CONFIG_SHA256, EXPECTED_A8_FROZEN_ZERO_LINEAGE_SHA256),
    ):
        arm_entry = plans_block.get(arm_key)
        require(isinstance(arm_entry, dict), f"missing plans.{arm_key}")
        plan = arm_entry.get("plan")
        review = arm_entry.get("review")
        check_plan = arm_entry.get("check_plan")
        require(
            isinstance(plan, dict) and isinstance(review, dict) and isinstance(check_plan, dict),
            f"plans.{arm_key} requires plan, review, and check_plan objects",
        )
        cfg = plan.get("config")
        require(isinstance(cfg, dict), f"plans.{arm_key}.plan.config must be a dict")
        scope, d40_phrase = derive_scope_and_phrase(cfg)
        reg_sha, _ = registered_sheet_for_config(cfg)
        require(reg_sha == EXPECTED_A8_SHEET_SHA256, f"plans.{arm_key} must bind A8 sheet")
        comp_cfg_sha, comp_lin_sha = compute_config_and_lineage_sha(cfg, driver_sha)
        require(
            comp_cfg_sha == expected_cfg_sha and comp_lin_sha == expected_lin_sha,
            f"plans.{arm_key} config/lineage SHA mismatch",
        )
        ex = plan.get("execution", {})
        for h_key in (
            "source_sha256",
            "kernel_sha256",
            "geometry_sha256",
            "executable_sha256",
            "driver_source_sha256",
            "config_sha256",
            "lineage_sha256",
            "sheet_sha256",
        ):
            require(is_hex64(ex.get(h_key)), f"plans.{arm_key} invalid execution.{h_key}")
        require(
            plan.get("schema") == "uor-r4.attention-learning-owner-plan/2"
            and plan.get("status") == "HELD_OWNER_ACTION_REQUIRED"
            and plan.get("scope") == scope
            and ex.get("base_source") == EXPECTED_ATTENTION_BASE_SOURCE
            and is_hex40(ex.get("reviewed_source_commit"))
            and ex.get("driver_source_sha256") == driver_sha
            and ex.get("config_sha256") == expected_cfg_sha
            and ex.get("lineage_sha256") == expected_lin_sha
            and ex.get("sheet_sha256") == EXPECTED_A8_SHEET_SHA256
            and plan.get("sheet", {}).get("sha256") == EXPECTED_A8_SHEET_SHA256
            and plan.get("cost_contract") == expected_cost_contract(cfg),
            f"plans.{arm_key} header/execution/cost_contract mismatch",
        )
        run_state_dir = plan.get("run_state_directory", "")
        require(
            isinstance(run_state_dir, str) and run_state_dir.startswith("/"),
            f"plans.{arm_key} run_state_directory must be absolute",
        )
        state_dirs.append(run_state_dir.rstrip("/"))

        # Transfer source validation
        transfer = plan.get("transfer", {})
        t_tok_sha = transfer.get("tokens", {}).get("sha256")
        t_mask_sha = transfer.get("mask", {}).get("sha256")
        require(
            transfer.get("split") == "dev"
            and is_hex64(t_tok_sha)
            and is_hex64(t_mask_sha)
            and is_u64(transfer.get("expected_eligible_targets"))
            and transfer["expected_eligible_targets"] >= 2000
            and is_u64(transfer.get("expected_payload_tokens"))
            and transfer["expected_eligible_targets"] <= transfer["expected_payload_tokens"]
            and t_tok_sha not in (EXPECTED_TRAIN_TOKENS_SHA256, EXPECTED_DEV_TOKENS_SHA256)
            and t_mask_sha not in (EXPECTED_TRAIN_MASK_SHA256, EXPECTED_DEV_MASK_SHA256),
            f"plans.{arm_key} transfer source invalid or overlaps train/dev",
        )

        # Review validation
        expected_rev_payload_sha = review_payload_sha256(plan)
        require(
            review.get("reviewer")
            in {"AGY", "Antigravity", "antigravity", "Claude", "claude", "DeepSeek", "deepseek"}
            and review.get("verdict") == "APPROVE_ATTENTION_LEARNING_ADMISSION_DIFF"
            and review.get("scope") == scope
            and review.get("execution") == ex
            and review.get("reviewed_plan_payload_sha256") == expected_rev_payload_sha
            and isinstance(review.get("receipt_url"), str)
            and bool(RECEIPT_URL_RE.match(review["receipt_url"])),
            f"plans.{arm_key} independent review failed validation",
        )

        # check-plan validation
        plan_sha = canonical_plan_sha256(plan)
        require(
            check_plan.get("status") == "PLAN_METADATA_VALID"
            and check_plan.get("scope") == scope
            and check_plan.get("plan_sha256") == plan_sha
            and check_plan.get("cuda_context_created") is False
            and check_plan.get("fresh_content_read") is False
            and check_plan.get("owner_release") == "STILL_REQUIRED"
            and check_plan.get("training") == "HELD",
            f"plans.{arm_key} check_plan output failed validation",
        )

        # D40 owner release / admission validation (optional pre-run, mandatory for post-update dose bundles)
        owner_release_status = "STILL_REQUIRED"
        manifest_sha: str | None = None
        if "admission" in arm_entry or "owner" in arm_entry:
            admission = arm_entry.get("admission")
            owner = arm_entry.get("owner")
            require(
                isinstance(admission, dict) and isinstance(owner, dict),
                f"plans.{arm_key} must provide both admission and owner when released",
            )
            owner_pin = admission.get("owner_release", {})
            owner_bytes = json.dumps(
                {
                    "owner": owner.get("owner"),
                    "decision": owner.get("decision"),
                    "scope": owner.get("scope"),
                    "plan_sha256": owner.get("plan_sha256"),
                    "receipt_url": owner.get("receipt_url"),
                },
                separators=(",", ":"),
            ).encode("utf-8")
            owner_sha = hashlib.sha256(owner_bytes).hexdigest()
            require(
                admission.get("schema") == "uor-r4.attention-learning-owner-admission/2"
                and canonical_plan_dict(admission.get("plan", {})) == canonical_plan_dict(plan)
                and owner.get("owner") == "Casey"
                and owner.get("decision") == "RELEASE_D4_FOR_THIS_ATTENTION_LEARNING_ARM_SEED_ONLY"
                and owner.get("scope") == scope
                and owner.get("plan_sha256") == plan_sha
                and isinstance(owner_pin.get("path"), str)
                and owner_pin["path"].startswith("/")
                and owner.get("receipt_url") == f"owner-terminal:{owner_pin.get('path')}"
                and owner_pin.get("sha256") == owner_sha,
                f"plans.{arm_key} D40 owner release / admission failed validation",
            )
            owner_release_status = "RELEASED_BY_CASEY_D40"
            manifest_sha = canonical_admission_sha256(admission)

        if require_owner_release:
            require(
                owner_release_status == "RELEASED_BY_CASEY_D40" and is_hex64(manifest_sha),
                f"plans.{arm_key} requires RELEASED_BY_CASEY_D40 owner admission for post-update dose bundles",
            )

        out[arm_key] = {
            "scope": scope,
            "d40_phrase": d40_phrase,
            "plan_sha256": plan_sha,
            "reviewed_plan_payload_sha256": expected_rev_payload_sha,
            "owner_release_status": owner_release_status,
            "manifest_sha256": manifest_sha,
        }

    require(
        len(state_dirs) == 2
        and state_dirs[0] != state_dirs[1]
        and not state_dirs[0].startswith(state_dirs[1] + "/")
        and not state_dirs[1].startswith(state_dirs[0] + "/"),
        f"treatment and control run_state_directory paths must be disjoint: {state_dirs}",
    )
    return out


def verify_prerun_bundle(bundle: dict[str, Any]) -> dict[str, Any]:
    """Verify the complete A8 pre-run gate receipt bundle (sheet §3 & runbook steps 1-6).

    Returns a structured verdict dict:
    - Any gate miss (table, unpinned G-Init, neg-a/neg-b, G-Occ, reproduction 1351/4244/133,
      Bar (i), Gate22/Gate0 probe sensitivity, or `qualify-relation` C2/INERT/INIT-LOCK)
      returns `verdict = "INSTRUMENT_FAULT_D31"` (`mechanism_negative = False`), never a
      mechanism negative.
    """
    faults: list[str] = []

    driver_sha = bundle.get("driver_source_sha256", EXPECTED_DRIVER_SOURCE_SHA256)
    require(is_hex64(driver_sha), f"invalid driver_source_sha256: {driver_sha!r}")

    # 1. Table rebuild check
    table_rebuild = bundle.get("table_rebuild", {})
    if (
        table_rebuild.get("file_sha256") != EXPECTED_A8_TABLE_FILE_SHA256
        or table_rebuild.get("table_sha256") != EXPECTED_A8_TABLE_WORDS_SHA256
        or table_rebuild.get("counts_sha256") != EXPECTED_COUNTS_SHA256
    ):
        faults.append("table_rebuild_pin_mismatch")

    # 2. Both arms G-Init, unpinned comparison, G-Occ, and reproduction gate (1351 / 4244 / 133)
    arms = bundle.get("arms", {})
    require(
        isinstance(arms, dict) and "relation" in arms and "frozen_zero" in arms,
        "pre-run bundle requires arms.relation and arms.frozen_zero",
    )

    arm_models: dict[str, dict[str, Any]] = {}
    arm_psucc_top1: dict[str, int] = {}
    probe_summaries: dict[str, Any] = {}

    for arm_key, expected_cfg_sha, expected_lin_sha in (
        ("relation", EXPECTED_A8_RELATION_CONFIG_SHA256, EXPECTED_A8_RELATION_LINEAGE_SHA256),
        ("frozen_zero", EXPECTED_A8_FROZEN_ZERO_CONFIG_SHA256, EXPECTED_A8_FROZEN_ZERO_LINEAGE_SHA256),
    ):
        arm = arms[arm_key]
        ident = arm.get("build_identity", {})
        if (
            ident.get("driver_source_sha256") != driver_sha
            or ident.get("config_sha256") != expected_cfg_sha
            or ident.get("lineage_sha256") != expected_lin_sha
            or ident.get("cuda_context_created") is not False
            or ident.get("training") != "HELD"
            or "scope" in ident
        ):
            faults.append(f"{arm_key}_build_identity_mismatch")

        init_rep = arm.get("init_checkpoint", {})
        cursor = init_rep.get("cursor")
        model = init_rep.get("model", {})
        arm_models[arm_key] = model
        if not _is_zero_cursor(cursor):
            faults.append(f"{arm_key}_g_init_cursor_nonzero")

        words = model.get("table")
        if not (isinstance(words, list) and len(words) == 4096 * 32):
            faults.append(f"{arm_key}_g_init_table_length_invalid")
        else:
            word_sha = hashlib.sha256(struct.pack(f"<{len(words)}i", *words)).hexdigest()
            if word_sha != EXPECTED_A8_TABLE_WORDS_SHA256:
                faults.append(f"{arm_key}_g_init_table_sha256_mismatch")

        if (
            model.get("seed") != 1
            or model.get("epoch") != 0
            or model.get("read_initialization") != "active_leaf_v1"
            or model.get("gates") != EXPECTED_ACTIVE_LEAF_GATES
            or model.get("routes") != EXPECTED_MODEL_ROUTES
        ):
            faults.append(f"{arm_key}_g_init_storage_or_active_leaf_gates_mismatch")

        # Hard pre-run gate: unpinned SeededDeviceV1 comparison (sheet §3)
        unpinned = arm.get("unpinned_init_checkpoint")
        if not isinstance(unpinned, dict) or not isinstance(unpinned.get("model"), dict):
            faults.append(f"{arm_key}_g_init_unpinned_missing")
        else:
            u_model = unpinned["model"]
            u_words = u_model.get("table")
            u_word_sha = (
                hashlib.sha256(struct.pack(f"<{len(u_words)}i", *u_words)).hexdigest()
                if isinstance(u_words, list) and len(u_words) == 4096 * 32
                else None
            )
            if (
                u_model.get("routes") != model.get("routes")
                or u_model.get("gates") != model.get("gates")
                or u_model.get("seed") != model.get("seed")
                or u_model.get("epoch") != model.get("epoch")
                or u_word_sha == EXPECTED_A8_TABLE_WORDS_SHA256
            ):
                faults.append(f"{arm_key}_g_init_unpinned_routes_gates_seed_epoch_mismatch")

        # G-Occ check (supports canonical nested `cells` block from `coverage_census.rs` or flat shorthand)
        census = arm.get("census", {})
        occ_cells, cell_hist, coll_pairs, cells_status_ok = _extract_census_cells(census)
        if (
            not cells_status_ok
            or occ_cells != 256
            or census.get("distinct_vectors") != 4096
            or census.get("alias_group_size_histogram") != {"1": 4096}
            or census.get("within_cell_tie_mass")
            != {"dev_rows_targeting_non_representative": 0, "tokens_in_alias_groups": 0}
            or cell_hist != {"16": 256}
            or coll_pairs != 30720
            or census.get("dev_alias_ceiling_rows") != CANONICAL_DEV_FULL_ROWS
            or census.get("host_cell_reconstruction") is not False
            or census.get("model_execution") is not False
        ):
            faults.append(f"{arm_key}_g_occ_mismatch")

        # Reproduction gate (D31): B3_init(P_succ) == 1351, B3_init(full) == 4244, top1(full) == 133
        b3 = arm.get("b3", {})
        eval_agg = arm.get("eval_aggregate", {})
        psucc = b3.get("psucc", {})
        comp = b3.get("complement", {})
        full = b3.get("full", {})
        if (
            psucc.get("rows") != CANONICAL_DEV_PSUCC_ROWS
            or comp.get("rows") != CANONICAL_DEV_COMP_ROWS
            or full.get("rows") != CANONICAL_DEV_FULL_ROWS
            or psucc.get("target_admitted") != EXPECTED_A8_CP0_PSUCC_ADMITTED
            or comp.get("target_admitted") != EXPECTED_A8_CP0_COMP_ADMITTED
            or full.get("target_admitted") != EXPECTED_A8_CP0_FULL_ADMITTED
            or eval_agg.get("scored_rows") != CANONICAL_DEV_FULL_ROWS
            or eval_agg.get("target_admitted_rows") != EXPECTED_A8_CP0_FULL_ADMITTED
            or eval_agg.get("top1_correct") != EXPECTED_A8_CP0_FULL_TOP1
            or eval_agg.get("updates_applied_by_evaluation") != 0
            or eval_agg.get("evaluation_mutates_model") is not False
        ):
            faults.append(f"{arm_key}_reproduction_gate_miss")

        psucc_top1 = arm.get("psucc_top1_cp0")
        if not (is_u64(psucc_top1) and psucc_top1 <= EXPECTED_A8_CP0_FULL_TOP1):
            faults.append(f"{arm_key}_psucc_top1_cp0_invalid")
        else:
            arm_psucc_top1[arm_key] = psucc_top1

        # Gate22 and Gate0 probes
        probes = arm.get("probes", {})
        try:
            g22 = verify_probe_receipt(probes.get("gate_22", {}), 22, arm_key)
            g0 = verify_probe_receipt(probes.get("gate_0", {}), 0, arm_key)
            both_active = (
                g22["any_candidate_cost_differs_from_identity"]
                and g0["any_candidate_cost_differs_from_identity"]
            )
            if not both_active:
                faults.append(f"{arm_key}_probe_inert_under_bound_predicate")
            probe_summaries[arm_key] = {
                "predicate": "both_probes_any_active_candidate_cost_differs_v1",
                "passed": both_active,
                "gate_22": g22,
                "gate_0": g0,
            }
        except VerificationError:
            faults.append(f"{arm_key}_probe_schema_invalid")

    # Cross-arm checkpoint-0 bit-identity
    if arm_models.get("relation") != arm_models.get("frozen_zero"):
        faults.append("cross_arm_checkpoint0_model_mismatch")

    # 3. Negative controls (a) and (b)
    neg_a = bundle.get("neg_a", {})
    if (
        neg_a.get("rc", 0) == 0
        or neg_a.get("cuda_context_created") is not False
        or "content hash differs" not in str(neg_a.get("error", ""))
    ):
        faults.append("g_init_neg_a_host_refusal_failed")

    neg_b = bundle.get("neg_b", {})
    if (
        neg_b.get("rc", 0) == 0
        or neg_b.get("table_file_sha256") != EXPECTED_NEGB_A8_FILE_SHA256
        or neg_b.get("table_sha256") != EXPECTED_NEGB_A8_WORDS_SHA256
        or not re.search(r"status[^0-9]*6", str(neg_b.get("error", "")), re.IGNORECASE)
    ):
        faults.append("g_init_neg_b_gpu_status6_refusal_failed")

    # 4. Bar (i) & net G-Cap check (sheet §3)
    fz_psucc_top1 = arm_psucc_top1.get("frozen_zero", 0)
    bar_i_psucc = fz_psucc_top1 + 24
    bar_i_full_upper_bound = EXPECTED_A8_CP0_FULL_TOP1 + 24
    net_g_cap_threshold = max(500, bar_i_psucc)
    net_g_cap = EXPECTED_A8_CP0_PSUCC_ADMITTED
    if net_g_cap <= net_g_cap_threshold:
        faults.append("net_g_cap_below_threshold")

    # 5. qualify-relation C2 / INERT / INIT LOCK
    rel_controls_summary = None
    try:
        rel_controls_summary = verify_relation_controls(bundle.get("relation_controls", {}))
    except VerificationError:
        faults.append("qualify_relation_controls_failed")

    # 6. Reference battery (`attention-cell-reference-scorer preflight-dev`)
    ref_battery = bundle.get("reference_battery", {})
    ref_mode = ref_battery.get("mode")
    ref_status = "INVALID"
    if ref_mode == "NOT_RUN_DEFERRED_TO_DOSE":
        if (
            ref_battery.get("d34_sha256") == EXPECTED_D34_RECORD_SHA256
            and ref_battery.get("d34_supplied_in_packet") is True
            and ref_battery.get("capture_released") is False
            and ref_battery.get("full_reference_pass_claimed") is False
        ):
            ref_status = "NOT_RUN_DEFERRED_TO_DOSE"
        else:
            faults.append("reference_battery_deferred_contract_violation")
    elif ref_mode == "SCORED_PREFLIGHT_DEV":
        score_doc = ref_battery.get("score", {})
        same_cell = score_doc.get("same_cell_candidate_reranking", {})
        if (
            score_doc.get("schema") == "uor-r4.attention-preflight-reference-score/1"
            and score_doc.get("status") == "VALIDATED_PREFLIGHT_DEV_CAPTURE_AND_SEALED_D34_SCORE"
            and score_doc.get("pins", {}).get("d34_sha256") == EXPECTED_D34_RECORD_SHA256
            and score_doc.get("pins", {}).get("ordered_row_binding_sha256")
            == EXPECTED_ORDERED_ROW_BINDING_SHA256
            and score_doc.get("model_compute_executed_by_scorer") is False
            and score_doc.get("updates_applied") == 0
            and same_cell.get("status")
            == "SCORED_FROM_GPU_TOKEN_CELLS_AND_REGISTERED_CORPUS_TABLES"
            and same_cell.get("corpus_pins", {}).get("conservation_ok") is True
        ):
            ref_status = "PASS_FULL_PREFLIGHT_REFERENCE_SCORED"
        else:
            faults.append("reference_battery_scored_preflight_invalid")
    else:
        faults.append("reference_battery_mode_invalid")

    # 7. Plans & reviews
    plans_summary = None
    if "plans" in bundle:
        try:
            plans_summary = verify_plans_and_releases(bundle, driver_sha, require_owner_release=False)
        except VerificationError:
            faults.append("plans_or_reviews_validation_failed")

    if faults:
        return {
            "status": "FAIL",
            "verdict": "INSTRUMENT_FAULT_D31",
            "mechanism_negative": False,
            "ready_for_owner_release": False,
            "faults": faults,
            "reference_battery_status": ref_status,
        }

    return {
        "status": "PASS",
        "verdict": "PRERUN_GATES_PASS_READY_FOR_OWNER_RELEASE",
        "mechanism_negative": False,
        "ready_for_owner_release": True,
        "faults": [],
        "reproduction_gate": {
            "psucc_admitted": EXPECTED_A8_CP0_PSUCC_ADMITTED,
            "full_admitted": EXPECTED_A8_CP0_FULL_ADMITTED,
            "complement_admitted": EXPECTED_A8_CP0_COMP_ADMITTED,
            "full_top1_cp0": EXPECTED_A8_CP0_FULL_TOP1,
        },
        "bar_i_and_g_cap": {
            "frozen_zero_psucc_top1_cp0": fz_psucc_top1,
            "bar_i_psucc": bar_i_psucc,
            "bar_i_full_panel_upper_bound": bar_i_full_upper_bound,
            "net_g_cap_threshold": net_g_cap_threshold,
            "net_g_cap": net_g_cap,
            "passed": True,
        },
        "probes": probe_summaries,
        "relation_controls": rel_controls_summary,
        "reference_battery_status": ref_status,
        "plans": plans_summary,
    }


def validate_arm_counters(
    counters: dict[str, Any],
    completed_batches: int,
    accepted_updates: int,
    is_frozen_zero: bool,
    arm_label: str,
) -> None:
    """Replicate `Counters::validate` in `attention_learning.rs` + §7.8 item 2 45-coordinate proof."""
    require(
        0 <= completed_batches <= CANONICAL_ATTENTION_BATCHES,
        f"{arm_label}: completed_batches {completed_batches} out of bounds",
    )
    reads = completed_batches // 8
    combines = (reads // 45) * 20 + min(reads % 45, 20)
    relations = reads - combines
    vectors = completed_batches - reads
    expected_attempts = {
        "vector": vectors,
        "route": 0,
        "gate_combine": combines,
        "gate_relation": relations,
    }
    expected_refused = relations if is_frozen_zero else 0
    require(
        counters.get("refused_relation_proposals") == expected_refused,
        f"{arm_label}: refused_relation_proposals {counters.get('refused_relation_proposals')} != {expected_refused}",
    )
    total_accepted = 0
    for fam, exp_att in expected_attempts.items():
        fc = counters.get(fam, {})
        att = fc.get("attempted")
        sens = fc.get("any_candidate_cost_differs_from_identity")
        acc = fc.get("accepted_updates")
        require(
            is_u64(att)
            and is_u64(sens)
            and is_u64(acc)
            and att == exp_att
            and acc <= sens <= att,
            f"{arm_label} counter {fam} invalid: {fc!r} (expected attempted={exp_att})",
        )
        total_accepted += acc
    if is_frozen_zero:
        require(
            counters["gate_relation"]["accepted_updates"] == 0,
            f"{arm_label}: FrozenZero arm cannot have accepted relation updates",
        )
    require(
        total_accepted == accepted_updates,
        f"{arm_label}: accepted counter sum {total_accepted} != cursor.accepted_updates {accepted_updates}",
    )


def compute_q1_decision(
    init_psucc: int,
    q1_psucc: int,
    combine_sensitive: int,
    relation_sensitive: int,
) -> dict[str, Any]:
    """Replicate `attention_evidence::decide_counts`."""
    psucc_below_500 = q1_psucc < 500
    psucc_less_than_half_init = (2 * q1_psucc) < init_psucc
    no_sens = combine_sensitive == 0 and relation_sensitive == 0
    return {
        "diagnostic_stop": psucc_below_500 or psucc_less_than_half_init or no_sens,
        "psucc_below_500": psucc_below_500,
        "psucc_less_than_half_init": psucc_less_than_half_init,
        "no_combine_or_relation_sensitivity": no_sens,
        "init_psucc_target_admitted": init_psucc,
        "q1_psucc_target_admitted": q1_psucc,
        "combine_sensitive": combine_sensitive,
        "relation_sensitive": relation_sensitive,
    }


def verify_2x2_block(block: dict[str, Any], expected_mde_floor: int, label: str) -> dict[str, Any]:
    """Verify a `compute_paired_2x2` dictionary from `attention-cell-reference-scorer paired-dev`."""
    require(isinstance(block, dict), f"{label} must be a dict")
    n = block.get("n")
    a_hits = block.get("a_hits")
    b_hits = block.get("b_hits")
    n11 = block.get("n11_both_correct")
    b_only = block.get("b_a_only")
    c_only = block.get("c_b_only")
    n00 = block.get("n00_neither")
    disc = block.get("discordant_b_plus_c")
    diff = block.get("difference_a_minus_b")
    mde_floor = block.get("mde_floor")
    req_bar = block.get("required_integer_bar_max_mde_2sqrt_b_plus_c")
    meets = block.get("meets_or_exceeds_bar")

    require(
        all(is_u64(x) for x in (n, a_hits, b_hits, n11, b_only, c_only, n00, disc, mde_floor, req_bar))
        and isinstance(diff, int)
        and not isinstance(diff, bool)
        and isinstance(meets, bool),
        f"{label}: invalid field types in 2x2 block",
    )
    expected_req = paired_integer_floor(expected_mde_floor, b_only + c_only)
    require(
        n11 + b_only + c_only + n00 == n
        and a_hits == n11 + b_only
        and b_hits == n11 + c_only
        and disc == b_only + c_only
        and diff == a_hits - b_hits == b_only - c_only
        and mde_floor == expected_mde_floor
        and req_bar == expected_req
        and meets == (diff >= req_bar),
        f"{label}: 2x2 conservation or integer floor mismatch: {block!r}",
    )
    return block


def _is_valid_file_pin(pin: Any) -> bool:
    return (
        isinstance(pin, dict)
        and isinstance(pin.get("path"), str)
        and pin["path"].startswith("/")
        and is_hex64(pin.get("sha256"))
    )


def _validate_transfer_decision_pins(
    decision: dict[str, Any], plans_summary: dict[str, Any] | None
) -> bool:
    """Validate all 9 required `FilePin` bindings on `TransferDecision` (`attention_transfer.rs:47-63`)."""
    t_arm = decision.get("treatment")
    c_arm = decision.get("control")
    if not (isinstance(t_arm, dict) and isinstance(c_arm, dict)):
        return False
    pins = [
        t_arm.get("admission"),
        t_arm.get("final_checkpoint"),
        t_arm.get("final_dev"),
        c_arm.get("admission"),
        c_arm.get("final_checkpoint"),
        c_arm.get("final_dev"),
        decision.get("treatment_ablation_checkpoint"),
        decision.get("treatment_ablation_dev"),
        decision.get("paired_discriminator"),
    ]
    if not all(_is_valid_file_pin(p) for p in pins):
        return False
    if (
        t_arm["admission"]["path"] == c_arm["admission"]["path"]
        or t_arm["admission"]["sha256"] == c_arm["admission"]["sha256"]
        or t_arm["final_checkpoint"]["path"] == c_arm["final_checkpoint"]["path"]
        or t_arm["final_dev"]["path"] == c_arm["final_dev"]["path"]
    ):
        return False
    if plans_summary is not None:
        rel_man = plans_summary.get("relation", {}).get("manifest_sha256")
        fz_man = plans_summary.get("frozen_zero", {}).get("manifest_sha256")
        if (
            is_hex64(rel_man)
            and t_arm["admission"]["sha256"] != rel_man
        ) or (
            is_hex64(fz_man)
            and c_arm["admission"]["sha256"] != fz_man
        ):
            return False
    return True


def verify_dose_bundle(bundle: dict[str, Any]) -> dict[str, Any]:
    """Verify a Q1 or full 16,404-batch paired A8 dose bundle and apply the §7.7 Decision Table."""
    faults: list[str] = []
    arms = bundle.get("arms", {})
    if not (isinstance(arms, dict) and "relation" in arms and "frozen_zero" in arms):
        return {
            "verdict": "INSTRUMENT_FAULT",
            "precedence": 0,
            "mechanism_negative": False,
            "faults": ["missing_relation_or_frozen_zero_arm"],
        }

    # Precedence 0: Post-update dose bundles MUST carry verified D40 owner admission records
    # (`owner_release_status == "RELEASED_BY_CASEY_D40"`) for both arms.
    driver_sha = bundle.get("driver_source_sha256", EXPECTED_DRIVER_SOURCE_SHA256)
    plans_summary: dict[str, Any] | None = None
    if "plans" not in bundle:
        faults.append("missing_or_unreleased_d40_owner_admission_in_dose_bundle")
    else:
        try:
            plans_summary = verify_plans_and_releases(
                bundle, driver_sha, require_owner_release=True
            )
        except VerificationError as exc:
            faults.append(f"missing_or_unreleased_d40_owner_admission_in_dose_bundle:{exc}")

    # Check Q1 records on both arms (must exist for any run that reached >= 4096 batches)
    q1_decisions: dict[str, dict[str, Any]] = {}
    arm_completed: dict[str, int] = {}

    for arm_key, is_fz in (("relation", False), ("frozen_zero", True)):
        arm = arms[arm_key]
        rep = arm.get("report", {})
        completed = rep.get("completed_batches")
        arm_completed[arm_key] = completed if is_u64(completed) else -1
        q1_rec = arm.get("q1_record")
        if not isinstance(q1_rec, dict):
            faults.append(f"{arm_key}_missing_q1_record")
            continue

        q1_cursor = q1_rec.get("checkpoint", {}).get("cursor")
        q1_counters = q1_rec.get("counters", {})
        if isinstance(q1_cursor, dict):
            q1_batches = q1_cursor.get("completed_batches")
            q1_acc = q1_cursor.get("accepted_updates", -1)
        else:
            q1_batches = CANONICAL_Q1_BATCHES
            q1_acc = sum(
                q1_counters.get(fam, {}).get("accepted_updates", 0)
                for fam in ("vector", "route", "gate_combine", "gate_relation")
                if isinstance(q1_counters.get(fam), dict)
            )
        if q1_batches != CANONICAL_Q1_BATCHES:
            faults.append(f"{arm_key}_q1_completed_batches_not_4096")
        try:
            validate_arm_counters(
                q1_counters,
                CANONICAL_Q1_BATCHES,
                q1_acc,
                is_fz,
                f"{arm_key}.q1",
            )
        except VerificationError as exc:
            faults.append(f"{arm_key}_q1_counter_fault:{exc}")

        init_b3 = q1_rec.get("init_b3", {})
        q1_b3 = q1_rec.get("q1_b3", {})
        if (
            init_b3.get("psucc", {}).get("rows") != CANONICAL_DEV_PSUCC_ROWS
            or init_b3.get("psucc", {}).get("target_admitted") != EXPECTED_A8_CP0_PSUCC_ADMITTED
            or init_b3.get("full", {}).get("rows") != CANONICAL_DEV_FULL_ROWS
            or init_b3.get("full", {}).get("target_admitted") != EXPECTED_A8_CP0_FULL_ADMITTED
            or q1_b3.get("psucc", {}).get("rows") != CANONICAL_DEV_PSUCC_ROWS
            or q1_b3.get("full", {}).get("rows") != CANONICAL_DEV_FULL_ROWS
            or q1_b3.get("psucc", {}).get("target_admitted", 0)
            + q1_b3.get("complement", {}).get("target_admitted", 0)
            != q1_b3.get("full", {}).get("target_admitted", -1)
        ):
            faults.append(f"{arm_key}_q1_b3_partition_mismatch")
        else:
            expected_q1_dec = compute_q1_decision(
                init_b3["psucc"]["target_admitted"],
                q1_b3["psucc"]["target_admitted"],
                q1_counters.get("gate_combine", {}).get(
                    "any_candidate_cost_differs_from_identity", 0
                ),
                q1_counters.get("gate_relation", {}).get(
                    "any_candidate_cost_differs_from_identity", 0
                ),
            )
            if q1_rec.get("decision") != expected_q1_dec:
                faults.append(f"{arm_key}_q1_decision_mismatch")
            q1_decisions[arm_key] = expected_q1_dec

    # If either arm triggered Q1 diagnostic stop, enforce non-resumable stop at 4096
    any_q1_stop = any(d.get("diagnostic_stop") for d in q1_decisions.values())
    if any_q1_stop:
        for arm_key, dec in q1_decisions.items():
            rep = arms[arm_key].get("report", {})
            rep_adm_sha = rep.get("admission_sha256") or rep.get("manifest_sha256")
            if (
                plans_summary is not None
                and rep_adm_sha is not None
                and rep_adm_sha != plans_summary[arm_key]["manifest_sha256"]
            ):
                faults.append(f"{arm_key}_report_manifest_sha256_mismatch")
            if dec.get("diagnostic_stop"):
                if (
                    rep.get("status") != "Q1_STOP_FOR_DIAGNOSIS"
                    or rep.get("completed_batches") != CANONICAL_Q1_BATCHES
                    or rep.get("mechanism_negative") is not False
                    or rep.get("fresh_transfer_evaluations") != 0
                ):
                    faults.append(f"{arm_key}_q1_stop_bypassed_or_malformed")
            elif rep.get("completed_batches") == CANONICAL_ATTENTION_BATCHES:
                is_fz = arm_key == "frozen_zero"
                counters = rep.get("sensitivity", {})
                acc_sum = sum(
                    counters.get(fam, {}).get("accepted_updates", 0)
                    for fam in ("vector", "route", "gate_combine", "gate_relation")
                    if isinstance(counters.get(fam), dict)
                )
                try:
                    validate_arm_counters(
                        counters,
                        CANONICAL_ATTENTION_BATCHES,
                        acc_sum,
                        is_fz,
                        f"{arm_key}.final",
                    )
                except VerificationError as exc:
                    faults.append(f"{arm_key}_final_counter_fault:{exc}")
        if faults:
            return {
                "verdict": "INSTRUMENT_FAULT",
                "precedence": 0,
                "mechanism_negative": False,
                "faults": faults,
            }
        return {
            "verdict": "Q1_DIAGNOSTIC_STOP",
            "precedence": "Q1_GATE",
            "mechanism_negative": False,
            "configuration_closed_d22": False,
            "q1_decisions": q1_decisions,
            "plans": plans_summary,
            "next_action": "Diagnose Q1 stop (D31); non-resumable; never a mechanism negative.",
        }

    # Full-dose (16,404 batches) verification on both arms
    for arm_key, is_fz, expected_scope_arm in (
        ("relation", False, "relation"),
        ("frozen_zero", True, "relation-frozen-zero"),
    ):
        arm = arms[arm_key]
        rep = arm.get("report", {})
        expected_scope = f"m1-attention-learning-injective-6600bdb1306d-{expected_scope_arm}-k8-seed1-batches0..16404"
        if (
            rep.get("schema") != "uor-r4.attention-learning-report/1"
            or rep.get("status") != "COMPLETE_PENDING_REGISTERED_DISCRIMINATOR"
            or rep.get("learned_attention_result") != "NOT_YET_SCORED"
            or rep.get("scope") != expected_scope
            or rep.get("completed_batches") != CANONICAL_ATTENTION_BATCHES
            or rep.get("eligible_train_target_exposures") != CANONICAL_TRAIN_MATCHED_TARGETS
            or rep.get("fresh_transfer", {}).get("status")
            != "SEALED_PENDING_PAIRED_DEV_DECISION"
            or rep.get("fresh_transfer", {}).get("evaluations") != 0
            or rep.get("training_other_scopes") != "HELD"
        ):
            faults.append(f"{arm_key}_full_report_envelope_mismatch")

        rep_adm_sha = rep.get("admission_sha256") or rep.get("manifest_sha256")
        if (
            plans_summary is not None
            and rep_adm_sha != plans_summary[arm_key]["manifest_sha256"]
        ):
            faults.append(f"{arm_key}_report_manifest_sha256_mismatch")

        final_ckpt = arm.get("final_checkpoint", {})
        cursor = final_ckpt.get("cursor", {})
        model = final_ckpt.get("model", {})
        counters = rep.get("sensitivity", {})
        if cursor.get("completed_batches") != CANONICAL_ATTENTION_BATCHES:
            faults.append(f"{arm_key}_final_cursor_completed_batches_mismatch")
        try:
            validate_arm_counters(
                counters,
                CANONICAL_ATTENTION_BATCHES,
                cursor.get("accepted_updates", -1),
                is_fz,
                f"{arm_key}.final",
            )
        except VerificationError as exc:
            faults.append(f"{arm_key}_final_counter_fault:{exc}")

        gates = model.get("gates")
        if not (isinstance(gates, list) and len(gates) == 45):
            faults.append(f"{arm_key}_final_gates_length_invalid")
        elif is_fz and any(g != 0 for g in gates[20:45]):
            faults.append("frozen_zero_nonzero_relation_trit_in_final_snapshot")

        # Post-training G-Occ check (supports canonical nested `cells` block or flat shorthand)
        if "final_census" in arm:
            fc = arm["final_census"]
            occ_cells, _, _, cells_status_ok = _extract_census_cells(fc)
            if (
                not cells_status_ok
                or not is_u64(occ_cells)
                or occ_cells <= 16
                or fc.get("within_cell_tie_mass")
                != {"dev_rows_targeting_non_representative": 0, "tokens_in_alias_groups": 0}
            ):
                faults.append(f"{arm_key}_final_g_occ_failed")

        # Zero-relation ablation check
        rel_off = rep.get("relation_off", {})
        if is_fz:
            if rel_off.get("status") != "NOT_APPLICABLE_MATCHED_CONTROL_RELATION_PINNED_ZERO":
                faults.append("frozen_zero_relation_off_status_invalid")
        else:
            if (
                rel_off.get("status") != "OBSERVED_PENDING_REGISTERED_DISCRIMINATOR"
                or rel_off.get("transfer_evaluations") != 0
            ):
                faults.append("relation_ablation_status_invalid")
            abl_ckpt = arm.get("ablation_checkpoint_record", {})
            abl_model = abl_ckpt.get("checkpoint", {}).get("model", {})
            if isinstance(gates, list) and len(gates) == 45:
                expected_abl_gates = list(gates[:20]) + [0] * 25
                if (
                    abl_ckpt.get("schema") != "uor-r4.attention-relation-off/1"
                    or abl_ckpt.get("modified_storage") != "only relation gate trits set to zero"
                    or abl_model.get("gates") != expected_abl_gates
                    or abl_model.get("table") != model.get("table")
                    or abl_model.get("routes") != model.get("routes")
                    or abl_model.get("epoch") != model.get("epoch")
                ):
                    faults.append("relation_ablation_snapshot_mismatch")

    # D34 paired-dev reference score verification
    paired_score = bundle.get("paired_dev_score")
    if not isinstance(paired_score, dict):
        faults.append("missing_paired_dev_score")
    else:
        pins = paired_score.get("pins", {})
        if (
            paired_score.get("schema") != "uor-r4.attention-paired-dev-reference-score/1"
            or paired_score.get("status") != "VALIDATED_PAIRED_DEV_CAPTURE_AND_SEALED_D34_SCORE"
            or paired_score.get("model_compute_executed_by_scorer") is not False
            or paired_score.get("updates_applied") != 0
            or pins.get("d34_sha256") != EXPECTED_D34_RECORD_SHA256
            or pins.get("ordered_row_binding_sha256") != EXPECTED_ORDERED_ROW_BINDING_SHA256
            or pins.get("completed_batches") != CANONICAL_ATTENTION_BATCHES
            or pins.get("relation", {}).get("config_sha256") != EXPECTED_A8_RELATION_CONFIG_SHA256
            or pins.get("control", {}).get("config_sha256")
            != EXPECTED_A8_FROZEN_ZERO_CONFIG_SHA256
        ):
            faults.append("paired_dev_score_header_or_pins_mismatch")

    if faults:
        return {
            "verdict": "INSTRUMENT_FAULT",
            "precedence": 0,
            "mechanism_negative": False,
            "faults": faults,
        }

    # Validate admission_summary_psucc & paired_comparisons
    adm = paired_score["admission_summary_psucc"]
    b3_r = adm.get("b3_r")
    b3_c = adm.get("b3_c")
    b3_i = adm.get("b3_i_intersection")
    r_only = adm.get("r_only")
    c_only = adm.get("c_only")
    neither = adm.get("neither")
    if (
        not all(is_u64(x) for x in (b3_r, b3_c, b3_i, r_only, c_only, neither))
        or adm.get("psucc_rows") != CANONICAL_DEV_PSUCC_ROWS
        or b3_i + r_only + c_only + neither != CANONICAL_DEV_PSUCC_ROWS
        or b3_r != b3_i + r_only
        or b3_c != b3_i + c_only
        or adm.get("b3_i_meets_minimum_200") != (b3_i >= 200)
        or adm.get("mde_rel_24_over_b3_i") != {"numerator": 24, "denominator": b3_i}
    ):
        return {
            "verdict": "INSTRUMENT_FAULT",
            "precedence": 0,
            "mechanism_negative": False,
            "faults": ["admission_summary_psucc_conservation_mismatch"],
        }

    comps = paired_score.get("paired_comparisons", {})
    same_cell = comps.get("same_cell_reranked", {})
    ablation_block = comps.get("ablation", {})
    try:
        rel_vs_ctrl = verify_2x2_block(
            comps.get("relation_vs_control_on_b3_i_top1", {}), 24, "rel_vs_ctrl_b3_i"
        )
        itt_top1 = verify_2x2_block(
            comps.get("relation_vs_control_psucc_itt_top1", {}), 24, "rel_vs_ctrl_itt_top1"
        )
        itt_top8 = verify_2x2_block(
            comps.get("relation_vs_control_psucc_itt_top8", {}), 39, "rel_vs_ctrl_itt_top8"
        )
        comp_top1 = verify_2x2_block(
            comps.get("relation_vs_control_complement_top1", {}), 22, "rel_vs_ctrl_comp_top1"
        )
        comp_top8 = verify_2x2_block(
            comps.get("relation_vs_control_complement_top8", {}), 63, "rel_vs_ctrl_comp_top8"
        )
        require(ablation_block.get("status") == "SCORED", "ablation block must be SCORED")
        rel_vs_abl_b3_r = verify_2x2_block(
            ablation_block.get("relation_vs_ablation_on_b3_r_top1", {}), 24, "rel_vs_abl_b3_r"
        )
        rel_vs_abl_b3_i = verify_2x2_block(
            ablation_block.get("relation_vs_ablation_on_b3_i_top1", {}), 24, "rel_vs_abl_b3_i"
        )
        require(
            same_cell.get("status")
            == "SCORED_FROM_GPU_TOKEN_CELLS_AND_REGISTERED_CORPUS_TABLES",
            "same_cell_reranked must be SCORED_FROM_GPU_TOKEN_CELLS_AND_REGISTERED_CORPUS_TABLES",
        )
        corpus_pins = same_cell.get("corpus_pins", {})
        require(
            corpus_pins.get("conservation_ok") is True
            and corpus_pins.get("train_matched_targets") == CANONICAL_TRAIN_MATCHED_TARGETS
            and corpus_pins.get("dev_psucc_rows") == CANONICAL_DEV_PSUCC_ROWS
            and corpus_pins.get("ordered_row_binding_sha256")
            == EXPECTED_ORDERED_ROW_BINDING_SHA256,
            "same_cell_reranked corpus_pins conservation failed",
        )
        rel_vs_t2_r = verify_2x2_block(
            same_cell.get("relation_vs_t2_r_same_cell_on_b3_r_top1", {}), 24, "rel_vs_t2_r_b3_r"
        )
        rel_vs_t2_i = verify_2x2_block(
            same_cell.get("relation_vs_t2_r_same_cell_on_b3_i_top1", {}), 24, "rel_vs_t2_r_b3_i"
        )
        ctrl_vs_t2_i = verify_2x2_block(
            same_cell.get("control_vs_t2_c_same_cell_on_b3_i_top1", {}), 24, "ctrl_vs_t2_c_b3_i"
        )
        require(
            rel_vs_ctrl["n"] == b3_i
            and rel_vs_t2_r["n"] == b3_r
            and rel_vs_t2_i["n"] == b3_i
            and ctrl_vs_t2_i["n"] == b3_i
            and itt_top1["n"] == CANONICAL_DEV_PSUCC_ROWS
            and comp_top1["n"] == CANONICAL_DEV_COMP_ROWS
            and rel_vs_abl_b3_r["n"] == b3_r
            and rel_vs_abl_b3_i["n"] == b3_i,
            "paired 2x2 denominator mismatch against B3_I / B3_R / panel counts",
        )
    except VerificationError as exc:
        return {
            "verdict": "INSTRUMENT_FAULT",
            "precedence": 0,
            "mechanism_negative": False,
            "faults": [f"paired_2x2_or_same_cell_fault:{exc}"],
        }

    # Check complement B3 non-regression (floor = 87 rows on P_comp)
    rel_b3_comp = (
        arms["relation"]
        .get("report", {})
        .get("final_b3", {})
        .get("complement", {})
        .get("target_admitted", 0)
    )
    ctrl_b3_comp = (
        arms["frozen_zero"]
        .get("report", {})
        .get("final_b3", {})
        .get("complement", {})
        .get("target_admitted", 0)
    )
    comp_b3_diff = rel_b3_comp - ctrl_b3_comp

    # Precedence 1: Evaluability (`B3_I >= 200`)
    if b3_i < 200:
        if "fresh_transfer" in bundle:
            return {
                "verdict": "INSTRUMENT_FAULT",
                "precedence": 0,
                "mechanism_negative": False,
                "faults": ["premature_fresh_transfer_on_underpowered_dev"],
            }
        return {
            "verdict": "UNDERPOWERED_OR_INVALID",
            "precedence": 1,
            "mechanism_negative": False,
            "configuration_closed_d22": False,
            "b3_i": b3_i,
            "mde_rel": 24.0 / b3_i if b3_i > 0 else None,
            "plans": plans_summary,
        }

    # Precedence 2: H_null on common B3_I cells (`T2_I >= Relation + F_{T,I}` AND `T2_I >= control + F_{T,I}`)
    f_t_i = max(
        rel_vs_t2_i["required_integer_bar_max_mde_2sqrt_b_plus_c"],
        ctrl_vs_t2_i["required_integer_bar_max_mde_2sqrt_b_plus_c"],
    )
    t2_minus_rel_on_b3_i = -rel_vs_t2_i["difference_a_minus_b"]
    t2_minus_ctrl_on_b3_i = -ctrl_vs_t2_i["difference_a_minus_b"]

    # Determine DEV verdict across Precedence 2, 3, 4, 5
    f_i = rel_vs_ctrl["required_integer_bar_max_mde_2sqrt_b_plus_c"]
    diff_rel_ctrl = rel_vs_ctrl["difference_a_minus_b"]

    f_t_r = rel_vs_t2_r["required_integer_bar_max_mde_2sqrt_b_plus_c"]
    diff_rel_t2_r = rel_vs_t2_r["difference_a_minus_b"]

    f_a = rel_vs_abl_b3_r["required_integer_bar_max_mde_2sqrt_b_plus_c"]
    diff_rel_abl = rel_vs_abl_b3_r["difference_a_minus_b"]

    f_c_top1 = comp_top1["required_integer_bar_max_mde_2sqrt_b_plus_c"]
    diff_comp_top1 = comp_top1["difference_a_minus_b"]
    f_c_top8 = comp_top8["required_integer_bar_max_mde_2sqrt_b_plus_c"]
    diff_comp_top8 = comp_top8["difference_a_minus_b"]

    f_itt_top1 = itt_top1["required_integer_bar_max_mde_2sqrt_b_plus_c"]
    diff_itt_top1 = itt_top1["difference_a_minus_b"]

    comp_regressed = (
        (diff_comp_top1 < -f_c_top1)
        or (diff_comp_top8 < -f_c_top8)
        or (comp_b3_diff < -87)
    )
    itt_regressed = diff_itt_top1 < -f_itt_top1

    if t2_minus_rel_on_b3_i >= f_t_i and t2_minus_ctrl_on_b3_i >= f_t_i:
        dev_verdict = "H_null"
        dev_precedence = 2
    elif (-diff_rel_ctrl >= f_i) or (diff_rel_abl < f_a):
        dev_verdict = "H_control"
        dev_precedence = 3
    elif (
        diff_rel_ctrl >= f_i
        and diff_rel_t2_r >= f_t_r
        and not comp_regressed
        and not itt_regressed
        and diff_rel_abl >= f_a
    ):
        dev_verdict = "H_geom"
        dev_precedence = 5
    else:
        dev_verdict = "H_mixed_INCONCLUSIVE"
        dev_precedence = 4

    # Conditional Fresh-Transfer Gate (§7.12.3 & `attention_transfer.rs`)
    fresh_block = bundle.get("fresh_transfer")
    if fresh_block is not None:
        if dev_verdict != "H_geom":
            return {
                "verdict": "INSTRUMENT_FAULT",
                "precedence": 0,
                "mechanism_negative": False,
                "faults": [f"unauthorized_fresh_transfer_executed_when_dev_verdict_is_{dev_verdict}"],
            }
        decision = fresh_block.get("decision")
        result = fresh_block.get("result")
        if not (isinstance(decision, dict) and isinstance(result, dict)):
            return {
                "verdict": "INSTRUMENT_FAULT",
                "precedence": 0,
                "mechanism_negative": False,
                "faults": ["fresh_transfer_missing_decision_or_result"],
            }
        pm_url = decision.get("pm_receipt_url", "")
        if (
            decision.get("schema") != "uor-r4.attention-transfer-decision/2"
            or decision.get("verdict") != "RELEASE_PAIRED_FRESH_AFTER_DEV_H_GEOM"
            or decision.get("dev_verdict") != "H_geom"
            or not isinstance(pm_url, str)
            or not PM_TRANSFER_RECEIPT_URL_RE.match(pm_url)
            or not _validate_transfer_decision_pins(decision, plans_summary)
        ):
            return {
                "verdict": "INSTRUMENT_FAULT",
                "precedence": 0,
                "mechanism_negative": False,
                "faults": ["fresh_transfer_pm_decision_header_or_pins_invalid"],
            }
        pf = result.get("paired_fresh", {})
        b_fresh = pf.get("b_fresh")
        c_fresh = pf.get("c_fresh")
        both_corr = pf.get("both_correct")
        both_wrong = pf.get("both_wrong")
        b3_r_fresh = pf.get("b3_r")
        b3_c_fresh = pf.get("b3_c")
        b3_i_fresh = pf.get("b3_i")
        psucc_fresh = pf.get("psucc_rows")
        if (
            result.get("schema") != "uor-r4.attention-transfer-result/2"
            or result.get("status") != "PAIRED_FRESH_OBSERVED_PENDING_PM_DISCRIMINATOR"
            or result.get("snapshot_unchanged") is not True
            or result.get("updates_applied") != 0
            or result.get("treatment_fresh_evaluations") != 1
            or result.get("control_fresh_evaluations") != 1
            or result.get("gpu_fresh_evaluations") != 2
            or result.get("ablation_fresh_evaluations") != 0
            or not all(
                is_u64(x)
                for x in (
                    b_fresh,
                    c_fresh,
                    both_corr,
                    both_wrong,
                    b3_r_fresh,
                    b3_c_fresh,
                    b3_i_fresh,
                    psucc_fresh,
                )
            )
            or psucc_fresh < 500
            or b3_i_fresh > min(b3_r_fresh, b3_c_fresh)
            or max(b3_r_fresh, b3_c_fresh) > psucc_fresh
            or b_fresh + c_fresh + both_corr + both_wrong != b3_i_fresh
        ):
            return {
                "verdict": "INSTRUMENT_FAULT",
                "precedence": 0,
                "mechanism_negative": False,
                "faults": ["fresh_transfer_result_envelope_or_conservation_invalid"],
            }
        expected_fresh_floor = paired_integer_floor(20, b_fresh + c_fresh)
        expected_fresh_met = (c_fresh - b_fresh) >= expected_fresh_floor
        if (
            pf.get("floor") != expected_fresh_floor
            or pf.get("treatment_direction_and_floor_met") != expected_fresh_met
        ):
            return {
                "verdict": "INSTRUMENT_FAULT",
                "precedence": 0,
                "mechanism_negative": False,
                "faults": ["fresh_transfer_floor_or_direction_mismatch"],
            }
        if b3_i_fresh >= 200 and expected_fresh_met:
            return {
                "verdict": "H_geom",
                "dev_verdict": "H_geom",
                "precedence": 5,
                "keep": True,
                "mechanism_negative": False,
                "configuration_closed_d22": False,
                "claim_scope": CLAIM_SCOPE_H_GEOM,
                "b3_r": b3_r,
                "b3_c": b3_c,
                "b3_i": b3_i,
                "mde_rel": 24.0 / b3_i,
                "paired_fresh": pf,
                "plans": plans_summary,
            }
        return {
            "verdict": "DEV_ONLY_DIAGNOSTIC_FRESH_FAILED",
            "dev_verdict": "H_geom",
            "precedence": 6,
            "keep": False,
            "underpowered_fresh_b3_i": b3_i_fresh < 200,
            "mechanism_negative": False,
            "configuration_closed_d22": False,
            "b3_r": b3_r,
            "b3_c": b3_c,
            "b3_i": b3_i,
            "mde_rel": 24.0 / b3_i,
            "paired_fresh": pf,
            "plans": plans_summary,
        }

    # No fresh transfer executed yet
    if dev_verdict == "H_geom":
        return {
            "verdict": "H_geom_DEV_PENDING_PAIRED_FRESH_TRANSFER",
            "dev_verdict": "H_geom",
            "precedence": 5,
            "keep": False,
            "mechanism_negative": False,
            "configuration_closed_d22": False,
            "b3_r": b3_r,
            "b3_c": b3_c,
            "b3_i": b3_i,
            "mde_rel": 24.0 / b3_i,
            "plans": plans_summary,
            "next_action": "Await typed PM TransferDecision (uor-r4.attention-transfer-decision/2) and run paired attention-transfer.",
        }

    return {
        "verdict": dev_verdict,
        "dev_verdict": dev_verdict,
        "precedence": dev_precedence,
        "keep": False,
        "mechanism_negative": False,
        "configuration_closed_d22": (dev_verdict == "H_control"),
        "b3_r": b3_r,
        "b3_c": b3_c,
        "b3_i": b3_i,
        "mde_rel": 24.0 / b3_i,
        "plans": plans_summary,
    }


def _make_2x2(n11: int, b: int, c: int, n00: int, mde_floor: int) -> dict[str, Any]:
    req = paired_integer_floor(mde_floor, b + c)
    diff = b - c
    return {
        "n": n11 + b + c + n00,
        "a_hits": n11 + b,
        "b_hits": n11 + c,
        "n11_both_correct": n11,
        "b_a_only": b,
        "c_b_only": c,
        "n00_neither": n00,
        "discordant_b_plus_c": b + c,
        "difference_a_minus_b": diff,
        "mde_floor": mde_floor,
        "required_integer_bar_max_mde_2sqrt_b_plus_c": req,
        "meets_or_exceeds_bar": diff >= req,
    }


def build_vast_56670997_plan_bundle(repo_root: Path, arm_key: str) -> dict[str, Any]:
    """Construct and cryptographically verify the exact `vast-56670997` plan/review/owner/admission records."""
    if arm_key == "relation":
        cfg = read_json_checked(
            repo_root
            / "docs"
            / "labs"
            / "a8-training-configs-2026-10-19"
            / "evidence"
            / "a8-relation-seed1.json"
        )
        cfg_sha = EXPECTED_A8_RELATION_CONFIG_SHA256
        lin_sha = EXPECTED_A8_RELATION_LINEAGE_SHA256
        arm_slug = "relation"
        rev_file_sha = EXPECTED_VAST_56670997_REL_REVIEW_FILE_SHA256
        exp_rev_payload_sha = EXPECTED_VAST_56670997_REL_REVIEW_PAYLOAD_SHA256
        rev_receipt_url = "https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6259900886"
        exp_raw_plan_sha = EXPECTED_VAST_56670997_REL_PLAN_RAW_SHA256
        exp_canon_plan_sha = EXPECTED_VAST_56670997_REL_PLAN_CANONICAL_SHA256
        exp_owner_sha = EXPECTED_VAST_56670997_REL_OWNER_SHA256
        exp_adm_sha = EXPECTED_VAST_56670997_REL_ADMISSION_SHA256
    elif arm_key == "frozen_zero":
        cfg = read_json_checked(
            repo_root
            / "docs"
            / "labs"
            / "a8-training-configs-2026-10-19"
            / "evidence"
            / "a8-frozen-zero-seed1.json"
        )
        cfg_sha = EXPECTED_A8_FROZEN_ZERO_CONFIG_SHA256
        lin_sha = EXPECTED_A8_FROZEN_ZERO_LINEAGE_SHA256
        arm_slug = "frozen-zero"
        rev_file_sha = EXPECTED_VAST_56670997_FZ_REVIEW_FILE_SHA256
        exp_rev_payload_sha = EXPECTED_VAST_56670997_FZ_REVIEW_PAYLOAD_SHA256
        rev_receipt_url = "https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6259913207"
        exp_raw_plan_sha = EXPECTED_VAST_56670997_FZ_PLAN_RAW_SHA256
        exp_canon_plan_sha = EXPECTED_VAST_56670997_FZ_PLAN_CANONICAL_SHA256
        exp_owner_sha = EXPECTED_VAST_56670997_FZ_OWNER_SHA256
        exp_adm_sha = EXPECTED_VAST_56670997_FZ_ADMISSION_SHA256
    else:
        raise VerificationError(f"unknown arm_key: {arm_key}")

    scope, _ = derive_scope_and_phrase(cfg)
    ex = {
        "base_source": EXPECTED_ATTENTION_BASE_SOURCE,
        "reviewed_source_commit": EXPECTED_VAST_56670997_FREEZE_COMMIT,
        "source_sha256": EXPECTED_VAST_56670997_SOURCE_SHA256,
        "kernel_sha256": EXPECTED_VAST_56670997_KERNEL_SHA256,
        "geometry_sha256": EXPECTED_VAST_56670997_GEOMETRY_SHA256,
        "executable_sha256": EXPECTED_VAST_56670997_ELF_SHA256,
        "driver_source_sha256": EXPECTED_DRIVER_SOURCE_SHA256,
        "config_sha256": cfg_sha,
        "lineage_sha256": lin_sha,
        "sheet_sha256": EXPECTED_A8_SHEET_SHA256,
    }
    plan = {
        "schema": "uor-r4.attention-learning-owner-plan/2",
        "status": "HELD_OWNER_ACTION_REQUIRED",
        "scope": scope,
        "execution": ex,
        "config": cfg,
        "sheet": {
            "path": f"/root/build/src-{EXPECTED_VAST_56670997_FREEZE_COMMIT}/docs/harness/prep/m1-attention-a8-address-cells-learning-2026-10-19.md",
            "sha256": EXPECTED_A8_SHEET_SHA256,
        },
        "independent_review": {
            "path": f"/workspace/a8-dose/authorization/a8-{arm_slug}-seed1.review.json",
            "sha256": rev_file_sha,
        },
        "transfer": {
            "split": "dev",
            "tokens": {
                "path": "/workspace/a8-dose/fresh-sealed-v2/tokens.u16",
                "sha256": EXPECTED_FRESH_V2_TOKENS_SHA256,
            },
            "mask": {
                "path": "/workspace/a8-dose/fresh-sealed-v2/response_mask.u8",
                "sha256": EXPECTED_FRESH_V2_MASK_SHA256,
            },
            "expected_payload_tokens": 44678,
            "expected_eligible_targets": 14479,
        },
        "run_state_directory": f"/workspace/a8-dose/state/a8-{arm_slug}-seed1",
        "cost_contract": expected_cost_contract(cfg),
    }
    rev_payload_sha = review_payload_sha256(plan)
    require(
        rev_payload_sha == exp_rev_payload_sha,
        f"{arm_key} vast-56670997 review_payload_sha256 {rev_payload_sha} != {exp_rev_payload_sha}",
    )
    raw_plan_sha = hashlib.sha256((json.dumps(plan, indent=2) + "\n").encode("utf-8")).hexdigest()
    require(
        raw_plan_sha == exp_raw_plan_sha,
        f"{arm_key} vast-56670997 raw plan SHA-256 {raw_plan_sha} != {exp_raw_plan_sha}",
    )
    plan_sha = canonical_plan_sha256(plan)
    require(
        plan_sha == exp_canon_plan_sha,
        f"{arm_key} vast-56670997 canonical plan_sha256 {plan_sha} != {exp_canon_plan_sha}",
    )
    review = {
        "reviewer": "Claude",
        "verdict": "APPROVE_ATTENTION_LEARNING_ADMISSION_DIFF",
        "scope": scope,
        "execution": ex,
        "reviewed_plan_payload_sha256": rev_payload_sha,
        "receipt_url": rev_receipt_url,
    }
    computed_rev_file_sha = hashlib.sha256(
        (json.dumps(review, indent=2) + "\n").encode("utf-8")
    ).hexdigest()
    require(
        computed_rev_file_sha == rev_file_sha,
        f"{arm_key} vast-56670997 review file SHA-256 {computed_rev_file_sha} != {rev_file_sha}",
    )
    owner_path = f"/workspace/a8-dose/release/a8-{arm_slug}-seed1/owner.json"
    owner = {
        "owner": "Casey",
        "decision": "RELEASE_D4_FOR_THIS_ATTENTION_LEARNING_ARM_SEED_ONLY",
        "scope": scope,
        "plan_sha256": plan_sha,
        "receipt_url": f"owner-terminal:{owner_path}",
    }
    owner_sha = hashlib.sha256(
        json.dumps(owner, separators=(",", ":")).encode("utf-8")
    ).hexdigest()
    require(
        owner_sha == exp_owner_sha,
        f"{arm_key} vast-56670997 owner.json SHA-256 {owner_sha} != {exp_owner_sha}",
    )
    admission = {
        "schema": "uor-r4.attention-learning-owner-admission/2",
        "plan": plan,
        "owner_release": {"path": owner_path, "sha256": owner_sha},
    }
    adm_sha = canonical_admission_sha256(admission)
    require(
        adm_sha == exp_adm_sha,
        f"{arm_key} vast-56670997 admission.json SHA-256 {adm_sha} != {exp_adm_sha}",
    )
    check_plan = {
        "status": "PLAN_METADATA_VALID",
        "scope": scope,
        "plan_sha256": plan_sha,
        "cuda_context_created": False,
        "fresh_content_read": False,
        "owner_release": "STILL_REQUIRED",
        "training": "HELD",
    }
    return {
        "plan": plan,
        "review": review,
        "check_plan": check_plan,
        "owner": owner,
        "admission": admission,
        "raw_plan_sha256": raw_plan_sha,
        "review_file_sha256": computed_rev_file_sha,
        "manifest_sha256": adm_sha,
    }


def build_synthetic_prerun_bundle(repo_root: Path) -> dict[str, Any]:
    """Construct a valid pre-run gate receipt bundle for `--self-test` using canonical producer schemas."""
    negb_doc = read_json_checked(
        repo_root / "docs" / "labs" / "a8-dose-runbook-2026-10-19" / "evidence" / "neg-b-a8-nonroot-table.json"
    )
    a8_words = EXPECTED_A8_TOKEN0_BLOCK0 + negb_doc["table"][8:]
    unpinned_words = list(a8_words)
    unpinned_words[0] = 2  # distinct from A8

    def make_arm(cfg_sha: str, lin_sha: str) -> dict[str, Any]:
        return {
            "build_identity": {
                "driver_source_sha256": EXPECTED_DRIVER_SOURCE_SHA256,
                "config_sha256": cfg_sha,
                "lineage_sha256": lin_sha,
                "cuda_context_created": False,
                "training": "HELD",
            },
            "init_checkpoint": {
                "cursor": {
                    "completed_batches": 0,
                    "accepted_updates": 0,
                    "corpus_pass": 0,
                    "next_target": 0,
                    "next_coordinate": 0,
                },
                "model": {
                    "seed": 1,
                    "epoch": 0,
                    "read_initialization": "active_leaf_v1",
                    "routes": list(EXPECTED_MODEL_ROUTES),
                    "gates": list(EXPECTED_ACTIVE_LEAF_GATES),
                    "table": list(a8_words),
                },
            },
            "unpinned_init_checkpoint": {
                "cursor": {
                    "completed_batches": 0,
                    "accepted_updates": 0,
                    "corpus_pass": 0,
                    "next_target": 0,
                    "next_coordinate": 0,
                },
                "model": {
                    "seed": 1,
                    "epoch": 0,
                    "read_initialization": "active_leaf_v1",
                    "routes": list(EXPECTED_MODEL_ROUTES),
                    "gates": list(EXPECTED_ACTIVE_LEAF_GATES),
                    "table": list(unpinned_words),
                },
            },
            "census": {
                "cells": {
                    "status": "FROM_GPU_TOKEN_CELL_RECEIPT",
                    "occupied_cells": 256,
                    "cell_occupancy_histogram": {"16": 256},
                    "cell_collision_pairs": 30720,
                },
                "distinct_vectors": 4096,
                "alias_group_size_histogram": {"1": 4096},
                "within_cell_tie_mass": {
                    "dev_rows_targeting_non_representative": 0,
                    "tokens_in_alias_groups": 0,
                },
                "dev_alias_ceiling_rows": CANONICAL_DEV_FULL_ROWS,
                "host_cell_reconstruction": False,
                "model_execution": False,
            },
            "b3": {
                "psucc": {"rows": CANONICAL_DEV_PSUCC_ROWS, "target_admitted": EXPECTED_A8_CP0_PSUCC_ADMITTED},
                "complement": {"rows": CANONICAL_DEV_COMP_ROWS, "target_admitted": EXPECTED_A8_CP0_COMP_ADMITTED},
                "full": {"rows": CANONICAL_DEV_FULL_ROWS, "target_admitted": EXPECTED_A8_CP0_FULL_ADMITTED},
            },
            "eval_aggregate": {
                "scored_rows": CANONICAL_DEV_FULL_ROWS,
                "target_admitted_rows": EXPECTED_A8_CP0_FULL_ADMITTED,
                "top1_correct": EXPECTED_A8_CP0_FULL_TOP1,
                "updates_applied_by_evaluation": 0,
                "evaluation_mutates_model": False,
            },
            "psucc_top1_cp0": 62,
            "probes": {
                "gate_22": {
                    "parameter": {"kind": "gate", "index": 22},
                    "before": 59136,
                    "after": 59136,
                    "candidate": 0,
                    "any_candidate_cost_differs_from_identity": True,
                    "candidate_losses": [59136, 59872, 60928] + [U64_MAX] * 13,
                    "epoch": 0,
                    "applied": False,
                },
                "gate_0": {
                    "parameter": {"kind": "gate", "index": 0},
                    "before": 59136,
                    "after": 59136,
                    "candidate": 0,
                    "any_candidate_cost_differs_from_identity": True,
                    "candidate_losses": [59136, 61312, 62400] + [U64_MAX] * 13,
                    "epoch": 0,
                    "applied": False,
                },
            },
        }

    controls = []
    for idx, name in enumerate(RELATION_QUALIFICATION_NAMES, start=1):
        details = None
        if idx == 9:
            details = {
                "init_locks": [
                    {
                        "control": "active_leaf_initialization_and_matched_off_path",
                        "part": "INIT LOCK",
                        "spec": {"vocabulary": 4096, "blocks": 4, "context": 192, "kept_cells": 8},
                        "read_spec": {"kind": "relation", "kept": 8, "sweeps": 1},
                        "seed": s,
                        "initialization": "active_leaf_v1",
                        "plus_one_combine_indices": [0, 5, 10, 15],
                        "gates": list(EXPECTED_ACTIVE_LEAF_GATES),
                        "treatment_control_snapshot_bit_identical": True,
                        "reload_bit_identical": True,
                        "snapshot_sha256": (
                            "4fd9d9665964a5d448dd74a9f15c288c86b051a5f3d472ebaa90abcfa5cff423"
                            if s == 1
                            else "83387d14ef50f113238a88226fa75df82a7f718fd90a594519e6fb520eabc12d"
                        ),
                        "model_storage_sha256": (
                            "c65446ba7b53835a42667a11efff5be6db0d35a4a93b30cdeaec6cf95d22dce3"
                            if s == 1
                            else "53653efb2a934756b22d1b79b9daea6862baf63656d882d3fda7d9522bf5d03e"
                        ),
                        "model_snapshot_sha256": (
                            "4fd9d9665964a5d448dd74a9f15c288c86b051a5f3d472ebaa90abcfa5cff423"
                            if s == 1
                            else "83387d14ef50f113238a88226fa75df82a7f718fd90a594519e6fb520eabc12d"
                        ),
                        "applied_updates": 0,
                    }
                    for s in (1, 2)
                ]
            }
        controls.append({"index": idx, "name": name, "status": "PASS", "details": details})

    return {
        "driver_source_sha256": EXPECTED_DRIVER_SOURCE_SHA256,
        "table_rebuild": {
            "file_sha256": EXPECTED_A8_TABLE_FILE_SHA256,
            "table_sha256": EXPECTED_A8_TABLE_WORDS_SHA256,
            "counts_sha256": EXPECTED_COUNTS_SHA256,
        },
        "arms": {
            "relation": make_arm(
                EXPECTED_A8_RELATION_CONFIG_SHA256,
                EXPECTED_A8_RELATION_LINEAGE_SHA256,
            ),
            "frozen_zero": make_arm(
                EXPECTED_A8_FROZEN_ZERO_CONFIG_SHA256,
                EXPECTED_A8_FROZEN_ZERO_LINEAGE_SHA256,
            ),
        },
        "neg_a": {
            "rc": 2,
            "cuda_context_created": False,
            "error": "invalid discrete model: content hash differs: /workspace/uor-r4/runs/claude/a8-dose-gates-20261019/neg/a.json",
        },
        "neg_b": {
            "rc": 2,
            "table_file_sha256": EXPECTED_NEGB_A8_FILE_SHA256,
            "table_sha256": EXPECTED_NEGB_A8_WORDS_SHA256,
            "error": "invalid discrete model: discrete CUDA snapshot validation refused with status 6",
        },
        "relation_controls": {
            "schema": "uor-r4.discrete-spiralcore-fbs/2",
            "suite": "relation",
            "status": "PASS",
            "registered": 9,
            "attempted": 9,
            "passed": 9,
            "failed": 0,
            "not_run": 0,
            "controls": controls,
            "training": "HELD",
            "applied_updates": 0,
        },
        "reference_battery": {
            "mode": "NOT_RUN_DEFERRED_TO_DOSE",
            "d34_sha256": EXPECTED_D34_RECORD_SHA256,
            "d34_supplied_in_packet": True,
            "capture_released": False,
            "full_reference_pass_claimed": False,
        },
        "plans": {
            "relation": build_vast_56670997_plan_bundle(repo_root, "relation"),
            "frozen_zero": build_vast_56670997_plan_bundle(repo_root, "frozen_zero"),
        },
    }


def verify_vast_56670997_prerun_artifacts(
    gates_log_path: Path,
    relation_controls_path: Path,
    repo_root: Path,
) -> dict[str, Any]:
    """Ingest and verify the preserved `vast-56670997` pre-run `original-gates.log` and `original-relation-controls.json`."""
    gates_bytes = read_bytes_checked(gates_log_path, "vast-56670997 original-gates.log")
    gates_sha = hashlib.sha256(gates_bytes).hexdigest()
    require(
        gates_sha == EXPECTED_VAST_56670997_GATES_LOG_SHA256
        and len(gates_bytes) == EXPECTED_VAST_56670997_GATES_LOG_BYTES,
        f"original-gates.log SHA/bytes mismatch: {gates_sha} ({len(gates_bytes)} B)",
    )
    rel_ctrl_bytes = read_bytes_checked(
        relation_controls_path, "vast-56670997 original-relation-controls.json"
    )
    rel_ctrl_sha = hashlib.sha256(rel_ctrl_bytes).hexdigest()
    require(
        rel_ctrl_sha == EXPECTED_VAST_56670997_RELATION_CONTROLS_SHA256
        and len(rel_ctrl_bytes) == EXPECTED_VAST_56670997_RELATION_CONTROLS_BYTES,
        f"original-relation-controls.json SHA/bytes mismatch: {rel_ctrl_sha} ({len(rel_ctrl_bytes)} B)",
    )
    rel_ctrl_doc = json.loads(rel_ctrl_bytes.decode("utf-8"))
    gates_text = gates_bytes.decode("utf-8")
    lines = gates_text.splitlines()

    require(
        EXPECTED_VAST_56670997_FREEZE_COMMIT in lines,
        "original-gates.log missing source freeze commit 2c015db9f01d3ba3a7d79889a052b1345d9da48f",
    )
    require(
        f"{EXPECTED_VAST_56670997_ELF_SHA256}  /root/build/target-sm120/release/discrete-stack" in lines,
        "original-gates.log missing discrete-stack ELF SHA-256 line",
    )

    required_gate_passes = (
        "GATE DATA_PINS: PASS",
        "GATE TABLE_REBUILD: PASS",
        "GATE relation_IDENTITY: PASS",
        "GATE relation_GOCC: PASS",
        "GATE relation_GINIT_TABLE: PASS",
        "GATE relation_INIT_LOCK_GATES: PASS",
        "GATE relation_GINIT_UNPINNED_routes: PASS",
        "GATE relation_GINIT_UNPINNED_gates: PASS",
        "GATE relation_GINIT_UNPINNED_seed: PASS",
        "GATE relation_GINIT_UNPINNED_epoch: PASS",
        "GATE relation_REPRODUCTION: PASS",
        "GATE relation_GATE22_SENSITIVE: PASS",
        "GATE relation_GATE0_SENSITIVE: PASS",
        "GATE frozen-zero_IDENTITY: PASS",
        "GATE frozen-zero_GOCC: PASS",
        "GATE frozen-zero_GINIT_TABLE: PASS",
        "GATE frozen-zero_INIT_LOCK_GATES: PASS",
        "GATE frozen-zero_GINIT_UNPINNED_routes: PASS",
        "GATE frozen-zero_GINIT_UNPINNED_gates: PASS",
        "GATE frozen-zero_GINIT_UNPINNED_seed: PASS",
        "GATE frozen-zero_GINIT_UNPINNED_epoch: PASS",
        "GATE frozen-zero_REPRODUCTION: PASS",
        "GATE frozen-zero_GATE22_SENSITIVE: PASS",
        "GATE frozen-zero_GATE0_SENSITIVE: PASS",
        "GATE CROSS_ARM_MODEL: PASS",
        "GATE NET_GCAP: PASS",
        "GATE B3_INIT: PASS",
        "GATE NEG_A: PASS",
        "GATE NEG_B: PASS",
        "GATE QUALIFY_RELATION: PASS",
    )
    for gline in required_gate_passes:
        require(gline in lines, f"original-gates.log missing required PASS gate line: {gline}")

    # Verify the preserved INIT_LOCK_RECORD jq selector false-FAIL is documented and resolved by canonical JSON
    require(
        '{"part":null,"treatment_control_snapshot_bit_identical":null,"reload_bit_identical":null,"plus_one_combine_indices":null,"snapshot_sha256":null,"applied_updates":null}'
        in lines
        and "GATE INIT_LOCK_RECORD: FAIL" in lines,
        "original-gates.log missing preserved INIT_LOCK_RECORD selector false-FAIL evidence",
    )

    # Verify compact jq -c hashes of routes, gates, seed, epoch in unpinned G-Init check
    expected_jq_hashes = {
        "routes": hashlib.sha256(
            (json.dumps(EXPECTED_MODEL_ROUTES, separators=(",", ":")) + "\n").encode("utf-8")
        ).hexdigest(),
        "gates": hashlib.sha256(
            (json.dumps(EXPECTED_ACTIVE_LEAF_GATES, separators=(",", ":")) + "\n").encode("utf-8")
        ).hexdigest(),
        "seed": hashlib.sha256(b"1\n").hexdigest(),
        "epoch": hashlib.sha256(b"0\n").hexdigest(),
    }
    for arm_label in ("relation", "frozen-zero"):
        for field_name, exp_hash in expected_jq_hashes.items():
            expected_line = f"{arm_label} {field_name} pinned={exp_hash} unpinned={exp_hash}"
            require(
                expected_line in lines,
                f"original-gates.log missing unpinned equality line: {expected_line}",
            )
        require(
            f"{arm_label} b3[psucc,full]=[1351,4244] top1_full=133" in lines,
            f"original-gates.log missing {arm_label} reproduction line",
        )
        require(
            f"{arm_label} psucc_top1_cp0=62" in lines,
            f"original-gates.log missing {arm_label} psucc_top1_cp0=62 line",
        )
        require(
            f"58cd6f1d4ec3a6dd35c536957e693f58ee41794535110a96e5ecd5896677e703  /workspace/uor-r4/runs/claude/a8-dose-gates-20261019/{arm_label}/snapshot.json"
            in lines,
            f"original-gates.log missing {arm_label} snapshot.json SHA-256",
        )
        require(
            f"c919ac5f0ac442983f3dd89e457956dd5c6e8b7a5542c171854f4c58d4940ac6  /workspace/uor-r4/runs/claude/a8-dose-gates-20261019/{arm_label}/request.json"
            in lines,
            f"original-gates.log missing {arm_label} request.json SHA-256",
        )

    require(
        "bar_i=86 ceiling=14287 b3_psucc=1351 net_gcap=1351 threshold=500" in lines,
        "original-gates.log missing Bar (i) / net G-Cap summary line",
    )

    # Parse embedded JSON receipts from original-gates.log (table rebuild + 4 gate probes)
    table_rebuild_doc = json.loads(lines[19])
    rel_g22 = json.loads(lines[55])
    rel_g0 = json.loads(lines[59])
    fz_g22 = json.loads(lines[95])
    fz_g0 = json.loads(lines[99])

    # Build full pre-run bundle using live vast-56670997 receipts + original-relation-controls.json + plans/releases
    bundle = build_synthetic_prerun_bundle(repo_root)
    bundle["table_rebuild"] = table_rebuild_doc
    bundle["arms"]["relation"]["probes"]["gate_22"] = rel_g22
    bundle["arms"]["relation"]["probes"]["gate_0"] = rel_g0
    bundle["arms"]["frozen_zero"]["probes"]["gate_22"] = fz_g22
    bundle["arms"]["frozen_zero"]["probes"]["gate_0"] = fz_g0
    bundle["neg_a"] = {
        "rc": 2,
        "cuda_context_created": False,
        "error": lines[111],
    }
    bundle["neg_b"] = {
        "rc": 2,
        "table_file_sha256": EXPECTED_NEGB_A8_FILE_SHA256,
        "table_sha256": EXPECTED_NEGB_A8_WORDS_SHA256,
        "error": lines[117],
    }
    bundle["relation_controls"] = rel_ctrl_doc

    prerun_res = verify_prerun_bundle(bundle)
    require(
        prerun_res["status"] == "PASS"
        and prerun_res["verdict"] == "PRERUN_GATES_PASS_READY_FOR_OWNER_RELEASE"
        and prerun_res["bar_i_and_g_cap"]["bar_i_psucc"] == 86
        and prerun_res["bar_i_and_g_cap"]["net_g_cap"] == 1351
        and prerun_res["faults"] == []
        and prerun_res["plans"]["relation"]["owner_release_status"] == "RELEASED_BY_CASEY_D40"
        and prerun_res["plans"]["frozen_zero"]["owner_release_status"] == "RELEASED_BY_CASEY_D40",
        f"vast-56670997 pre-run verification failed: {prerun_res}",
    )

    # Also verify that the pre-fix frozen-zero plan (relation_off_dev_evaluations=1) reproduces bd6b1481...
    fz_prefix_plan = copy.deepcopy(bundle["plans"]["frozen_zero"]["plan"])
    fz_prefix_plan["cost_contract"]["relation_off_dev_evaluations"] = 1
    require(
        review_payload_sha256(fz_prefix_plan) == EXPECTED_VAST_56670997_FZ_REVIEW_PAYLOAD_PREFIX_SHA256,
        "failed to reproduce pre-fix frozen-zero review_payload_sha256 bd6b1481...",
    )

    return {
        "status": "PASS",
        "gates_log_sha256": gates_sha,
        "relation_controls_sha256": rel_ctrl_sha,
        "freeze_commit": EXPECTED_VAST_56670997_FREEZE_COMMIT,
        "discrete_stack_elf_sha256": EXPECTED_VAST_56670997_ELF_SHA256,
        "cp0_snapshot_sha256": "58cd6f1d4ec3a6dd35c536957e693f58ee41794535110a96e5ecd5896677e703",
        "cp0_request_sha256": "c919ac5f0ac442983f3dd89e457956dd5c6e8b7a5542c171854f4c58d4940ac6",
        "bar_i": prerun_res["bar_i_and_g_cap"]["bar_i_psucc"],
        "net_g_cap": prerun_res["bar_i_and_g_cap"]["net_g_cap"],
        "plans": prerun_res["plans"],
    }


def verify_committed_a8_dose_receipt(repo_root: Path) -> dict[str, Any]:
    """Verify the committed `#2767` `docs/labs/a8-dose-2026-10-19/` receipt and `vast-56670997` Q1 diagnostic stop."""
    ev_dir = repo_root / "docs" / "labs" / "a8-dose-2026-10-19" / "evidence"
    sha_manifest = parse_sha256sums_file(ev_dir / "SHA256SUMS")
    expected_ev_shas = {
        "a8-gates.sh": "a11ae977996d8012a60f830928e979387d26a1619858c466413da6c9fe0e9392",
        "a8-plan-payload.rs": "cbf28735bae46238daa867cc0ce370b97854693e71a0ec541a4929191c40eb29",
        "a8-plans.sh": "e38864aa361b5357e5ac31ef089582c54ccde3cc7099206faa19837f92bd7931",
        "a8-seal.sh": "988fc158a8bbb956da5120803f00ee46cd1cd9e35d51879ecf437e5df4aa2ca8",
        "frozen-zero-q1-summary.json": "92fcc94c38da0017e31fb3eabf88d4edf66d9ed0d50666c0a931e38f1ded607e",
        "frozen-zero-report.json": "91531de94af5039ca1ce44615e27f2802294ea09232f26f6f42c300c49fabd28",
        "gates-job.log": EXPECTED_VAST_56670997_GATES_LOG_SHA256,
        "relation-q1-summary.json": "12bff673bf7a3c2541e07da1de41c4457217673bd9ee3747ec3100bd62a9fb68",
    }
    require(
        sha_manifest == expected_ev_shas,
        f"a8-dose-2026-10-19/evidence/SHA256SUMS mismatch: {sha_manifest}",
    )
    rel_q1 = read_json_checked(ev_dir / "relation-q1-summary.json")
    fz_q1 = read_json_checked(ev_dir / "frozen-zero-q1-summary.json")
    fz_rep = read_json_checked(ev_dir / "frozen-zero-report.json")

    real_dose_bundle = {
        "driver_source_sha256": EXPECTED_DRIVER_SOURCE_SHA256,
        "plans": {
            "relation": build_vast_56670997_plan_bundle(repo_root, "relation"),
            "frozen_zero": build_vast_56670997_plan_bundle(repo_root, "frozen_zero"),
        },
        "arms": {
            "relation": {
                "q1_record": rel_q1,
                "report": rel_q1["report"],
            },
            "frozen_zero": {
                "q1_record": fz_q1,
                "report": fz_rep,
            },
        },
    }
    dose_res = verify_dose_bundle(real_dose_bundle)
    require(
        dose_res["verdict"] == "Q1_DIAGNOSTIC_STOP"
        and dose_res["mechanism_negative"] is False
        and dose_res["configuration_closed_d22"] is False
        and dose_res["q1_decisions"]["relation"]["diagnostic_stop"] is True
        and dose_res["q1_decisions"]["relation"]["psucc_less_than_half_init"] is True
        and dose_res["q1_decisions"]["relation"]["q1_psucc_target_admitted"] == 551
        and dose_res["q1_decisions"]["frozen_zero"]["diagnostic_stop"] is False
        and dose_res["q1_decisions"]["frozen_zero"]["q1_psucc_target_admitted"] == 857,
        f"committed #2767 A8 dose verification failed: {dose_res}",
    )
    return {
        "status": "PASS",
        "verdict": dose_res["verdict"],
        "mechanism_negative": dose_res["mechanism_negative"],
        "configuration_closed_d22": dose_res["configuration_closed_d22"],
        "relation_q1_psucc": 551,
        "relation_q1_full": 1511,
        "relation_q1_sha256": rel_q1["report"]["q1"]["sha256"],
        "frozen_zero_q1_psucc": 857,
        "frozen_zero_q1_full": 2519,
        "frozen_zero_q1_sha256": fz_rep["q1"]["sha256"],
        "frozen_zero_completed_batches": fz_rep["completed_batches"],
        "frozen_zero_refused_relation_proposals": fz_rep["sensitivity"]["refused_relation_proposals"],
    }


def build_synthetic_dose_bundle(repo_root: Path | None = None) -> dict[str, Any]:
    """Construct a valid synthetic full-dose (16,404-batch) `H_geom` + fresh-transfer bundle."""
    repo_root = repo_root or Path(__file__).resolve().parents[3]
    rel_gates = [1, 0, 0, 0, 0] * 4 + [1, 0, -1, 0, 0] + [0] * 20
    fz_gates = list(EXPECTED_ACTIVE_LEAF_GATES)

    def make_q1(is_fz: bool) -> dict[str, Any]:
        rel_sens = 12 if not is_fz else 10
        rel_acc = 4 if not is_fz else 0
        acc_total = 80 + 6 + rel_acc
        return {
            "checkpoint": {
                "cursor": {
                    "completed_batches": CANONICAL_Q1_BATCHES,
                    "accepted_updates": acc_total,
                    "corpus_pass": 0,
                    "next_target": CANONICAL_Q1_BATCHES * 64,
                    "next_coordinate": 0,
                }
            },
            "counters": {
                "vector": {
                    "attempted": 3584,
                    "any_candidate_cost_differs_from_identity": 400,
                    "accepted_updates": 80,
                },
                "route": {
                    "attempted": 0,
                    "any_candidate_cost_differs_from_identity": 0,
                    "accepted_updates": 0,
                },
                "gate_combine": {
                    "attempted": 237,
                    "any_candidate_cost_differs_from_identity": 30,
                    "accepted_updates": 6,
                },
                "gate_relation": {
                    "attempted": 275,
                    "any_candidate_cost_differs_from_identity": rel_sens,
                    "accepted_updates": rel_acc,
                },
                "refused_relation_proposals": 275 if is_fz else 0,
            },
            "init_b3": {
                "psucc": {"rows": CANONICAL_DEV_PSUCC_ROWS, "target_admitted": EXPECTED_A8_CP0_PSUCC_ADMITTED},
                "complement": {"rows": CANONICAL_DEV_COMP_ROWS, "target_admitted": EXPECTED_A8_CP0_COMP_ADMITTED},
                "full": {"rows": CANONICAL_DEV_FULL_ROWS, "target_admitted": EXPECTED_A8_CP0_FULL_ADMITTED},
            },
            "q1_b3": {
                "psucc": {"rows": CANONICAL_DEV_PSUCC_ROWS, "target_admitted": 1100},
                "complement": {"rows": CANONICAL_DEV_COMP_ROWS, "target_admitted": 2500},
                "full": {"rows": CANONICAL_DEV_FULL_ROWS, "target_admitted": 3600},
            },
            "decision": compute_q1_decision(EXPECTED_A8_CP0_PSUCC_ADMITTED, 1100, 30, rel_sens),
        }

    rel_counters = {
        "vector": {
            "attempted": 14354,
            "any_candidate_cost_differs_from_identity": 1500,
            "accepted_updates": 310,
        },
        "route": {
            "attempted": 0,
            "any_candidate_cost_differs_from_identity": 0,
            "accepted_updates": 0,
        },
        "gate_combine": {
            "attempted": 920,
            "any_candidate_cost_differs_from_identity": 110,
            "accepted_updates": 18,
        },
        "gate_relation": {
            "attempted": 1130,
            "any_candidate_cost_differs_from_identity": 95,
            "accepted_updates": 14,
        },
        "refused_relation_proposals": 0,
    }
    fz_counters = {
        "vector": {
            "attempted": 14354,
            "any_candidate_cost_differs_from_identity": 1480,
            "accepted_updates": 305,
        },
        "route": {
            "attempted": 0,
            "any_candidate_cost_differs_from_identity": 0,
            "accepted_updates": 0,
        },
        "gate_combine": {
            "attempted": 920,
            "any_candidate_cost_differs_from_identity": 108,
            "accepted_updates": 17,
        },
        "gate_relation": {
            "attempted": 1130,
            "any_candidate_cost_differs_from_identity": 88,
            "accepted_updates": 0,
        },
        "refused_relation_proposals": 1130,
    }

    b3_i = 900
    r_only = 100
    c_only = 80
    b3_r = b3_i + r_only
    b3_c = b3_i + c_only
    neither = CANONICAL_DEV_PSUCC_ROWS - (b3_i + r_only + c_only)

    return {
        "plans": {
            "relation": build_vast_56670997_plan_bundle(repo_root, "relation"),
            "frozen_zero": build_vast_56670997_plan_bundle(repo_root, "frozen_zero"),
        },
        "arms": {
            "relation": {
                "q1_record": make_q1(False),
                "final_checkpoint": {
                    "cursor": {
                        "completed_batches": CANONICAL_ATTENTION_BATCHES,
                        "accepted_updates": 342,
                        "corpus_pass": 0,
                        "next_target": CANONICAL_TRAIN_MATCHED_TARGETS,
                        "next_coordinate": 0,
                    },
                    "model": {
                        "epoch": 342,
                        "routes": list(EXPECTED_MODEL_ROUTES),
                        "gates": rel_gates,
                        "table": [2, 2, 0, 0, 0, 0, 0, 0] * 16384,
                    },
                },
                "ablation_checkpoint_record": {
                    "schema": "uor-r4.attention-relation-off/1",
                    "modified_storage": "only relation gate trits set to zero",
                    "checkpoint": {
                        "model": {
                            "epoch": 342,
                            "routes": list(EXPECTED_MODEL_ROUTES),
                            "gates": list(rel_gates[:20]) + [0] * 25,
                            "table": [2, 2, 0, 0, 0, 0, 0, 0] * 16384,
                        }
                    },
                },
                "final_census": {
                    "cells": {
                        "status": "FROM_GPU_TOKEN_CELL_RECEIPT",
                        "occupied_cells": 256,
                        "cell_occupancy_histogram": {"16": 256},
                        "cell_collision_pairs": 30720,
                    },
                    "within_cell_tie_mass": {
                        "dev_rows_targeting_non_representative": 0,
                        "tokens_in_alias_groups": 0,
                    },
                },
                "report": {
                    "schema": "uor-r4.attention-learning-report/1",
                    "status": "COMPLETE_PENDING_REGISTERED_DISCRIMINATOR",
                    "learned_attention_result": "NOT_YET_SCORED",
                    "scope": "m1-attention-learning-injective-6600bdb1306d-relation-k8-seed1-batches0..16404",
                    "manifest_sha256": EXPECTED_VAST_56670997_REL_ADMISSION_SHA256,
                    "completed_batches": CANONICAL_ATTENTION_BATCHES,
                    "eligible_train_target_exposures": CANONICAL_TRAIN_MATCHED_TARGETS,
                    "sensitivity": rel_counters,
                    "final_b3": {
                        "psucc": {"rows": CANONICAL_DEV_PSUCC_ROWS, "target_admitted": b3_r},
                        "complement": {"rows": CANONICAL_DEV_COMP_ROWS, "target_admitted": 2400},
                        "full": {"rows": CANONICAL_DEV_FULL_ROWS, "target_admitted": b3_r + 2400},
                    },
                    "relation_off": {
                        "status": "OBSERVED_PENDING_REGISTERED_DISCRIMINATOR",
                        "transfer_evaluations": 0,
                    },
                    "fresh_transfer": {
                        "status": "SEALED_PENDING_PAIRED_DEV_DECISION",
                        "evaluations": 0,
                    },
                    "training_other_scopes": "HELD",
                },
            },
            "frozen_zero": {
                "q1_record": make_q1(True),
                "final_checkpoint": {
                    "cursor": {
                        "completed_batches": CANONICAL_ATTENTION_BATCHES,
                        "accepted_updates": 322,
                        "corpus_pass": 0,
                        "next_target": CANONICAL_TRAIN_MATCHED_TARGETS,
                        "next_coordinate": 0,
                    },
                    "model": {
                        "epoch": 322,
                        "routes": list(EXPECTED_MODEL_ROUTES),
                        "gates": fz_gates,
                        "table": [2, 2, 0, 0, 0, 0, 0, 0] * 16384,
                    },
                },
                "final_census": {
                    "cells": {
                        "status": "FROM_GPU_TOKEN_CELL_RECEIPT",
                        "occupied_cells": 256,
                        "cell_occupancy_histogram": {"16": 256},
                        "cell_collision_pairs": 30720,
                    },
                    "within_cell_tie_mass": {
                        "dev_rows_targeting_non_representative": 0,
                        "tokens_in_alias_groups": 0,
                    },
                },
                "report": {
                    "schema": "uor-r4.attention-learning-report/1",
                    "status": "COMPLETE_PENDING_REGISTERED_DISCRIMINATOR",
                    "learned_attention_result": "NOT_YET_SCORED",
                    "scope": "m1-attention-learning-injective-6600bdb1306d-relation-frozen-zero-k8-seed1-batches0..16404",
                    "manifest_sha256": EXPECTED_VAST_56670997_FZ_ADMISSION_SHA256,
                    "completed_batches": CANONICAL_ATTENTION_BATCHES,
                    "eligible_train_target_exposures": CANONICAL_TRAIN_MATCHED_TARGETS,
                    "sensitivity": fz_counters,
                    "final_b3": {
                        "psucc": {"rows": CANONICAL_DEV_PSUCC_ROWS, "target_admitted": b3_c},
                        "complement": {"rows": CANONICAL_DEV_COMP_ROWS, "target_admitted": 2390},
                        "full": {"rows": CANONICAL_DEV_FULL_ROWS, "target_admitted": b3_c + 2390},
                    },
                    "relation_off": {
                        "status": "NOT_APPLICABLE_MATCHED_CONTROL_RELATION_PINNED_ZERO",
                    },
                    "fresh_transfer": {
                        "status": "SEALED_PENDING_PAIRED_DEV_DECISION",
                        "evaluations": 0,
                    },
                    "training_other_scopes": "HELD",
                },
            },
        },
        "paired_dev_score": {
            "schema": "uor-r4.attention-paired-dev-reference-score/1",
            "status": "VALIDATED_PAIRED_DEV_CAPTURE_AND_SEALED_D34_SCORE",
            "pins": {
                "d34_sha256": EXPECTED_D34_RECORD_SHA256,
                "ordered_row_binding_sha256": EXPECTED_ORDERED_ROW_BINDING_SHA256,
                "completed_batches": CANONICAL_ATTENTION_BATCHES,
                "relation": {"config_sha256": EXPECTED_A8_RELATION_CONFIG_SHA256},
                "control": {"config_sha256": EXPECTED_A8_FROZEN_ZERO_CONFIG_SHA256},
            },
            "admission_summary_psucc": {
                "psucc_rows": CANONICAL_DEV_PSUCC_ROWS,
                "b3_r": b3_r,
                "b3_c": b3_c,
                "b3_i_intersection": b3_i,
                "r_only": r_only,
                "c_only": c_only,
                "neither": neither,
                "b3_i_meets_minimum_200": True,
                "mde_rel_24_over_b3_i": {"numerator": 24, "denominator": b3_i},
            },
            "paired_comparisons": {
                "relation_vs_control_on_b3_i_top1": _make_2x2(120, 60, 20, b3_i - 200, 24),
                "relation_vs_control_psucc_itt_top1": _make_2x2(
                    130, 65, 22, CANONICAL_DEV_PSUCC_ROWS - 217, 24
                ),
                "relation_vs_control_psucc_itt_top8": _make_2x2(
                    310, 90, 40, CANONICAL_DEV_PSUCC_ROWS - 440, 39
                ),
                "relation_vs_control_complement_top1": _make_2x2(
                    200, 30, 28, CANONICAL_DEV_COMP_ROWS - 258, 22
                ),
                "relation_vs_control_complement_top8": _make_2x2(
                    650, 50, 48, CANONICAL_DEV_COMP_ROWS - 748, 63
                ),
                "ablation": {
                    "status": "SCORED",
                    "relation_vs_ablation_on_b3_r_top1": _make_2x2(115, 58, 18, b3_r - 191, 24),
                    "relation_vs_ablation_on_b3_i_top1": _make_2x2(110, 55, 17, b3_i - 182, 24),
                },
                "same_cell_reranked": {
                    "status": "SCORED_FROM_GPU_TOKEN_CELLS_AND_REGISTERED_CORPUS_TABLES",
                    "corpus_pins": {
                        "conservation_ok": True,
                        "train_matched_targets": CANONICAL_TRAIN_MATCHED_TARGETS,
                        "dev_psucc_rows": CANONICAL_DEV_PSUCC_ROWS,
                        "ordered_row_binding_sha256": EXPECTED_ORDERED_ROW_BINDING_SHA256,
                    },
                    "relation_vs_t2_r_same_cell_on_b3_r_top1": _make_2x2(
                        105, 62, 20, b3_r - 187, 24
                    ),
                    "relation_vs_t2_r_same_cell_on_b3_i_top1": _make_2x2(
                        100, 58, 19, b3_i - 177, 24
                    ),
                    "control_vs_t2_c_same_cell_on_b3_i_top1": _make_2x2(
                        95, 22, 55, b3_i - 172, 24
                    ),
                },
            },
            "model_compute_executed_by_scorer": False,
            "updates_applied": 0,
        },
        "fresh_transfer": {
            "decision": {
                "schema": "uor-r4.attention-transfer-decision/2",
                "verdict": "RELEASE_PAIRED_FRESH_AFTER_DEV_H_GEOM",
                "dev_verdict": "H_geom",
                "treatment": {
                    "admission": {
                        "path": "/workspace/a8-dose/release/a8-relation-seed1/admission.json",
                        "sha256": EXPECTED_VAST_56670997_REL_ADMISSION_SHA256,
                    },
                    "final_checkpoint": {
                        "path": "/workspace/a8-dose/state/a8-relation-seed1/checkpoint.json",
                        "sha256": "1" * 64,
                    },
                    "final_dev": {
                        "path": "/workspace/a8-dose/state/a8-relation-seed1/final-evaluation.json",
                        "sha256": "2" * 64,
                    },
                },
                "control": {
                    "admission": {
                        "path": "/workspace/a8-dose/release/a8-frozen-zero-seed1/admission.json",
                        "sha256": EXPECTED_VAST_56670997_FZ_ADMISSION_SHA256,
                    },
                    "final_checkpoint": {
                        "path": "/workspace/a8-dose/state/a8-frozen-zero-seed1/checkpoint.json",
                        "sha256": "3" * 64,
                    },
                    "final_dev": {
                        "path": "/workspace/a8-dose/state/a8-frozen-zero-seed1/final-evaluation.json",
                        "sha256": "4" * 64,
                    },
                },
                "treatment_ablation_checkpoint": {
                    "path": "/workspace/uor-r4/runs/claude/a8-dose-relation-seed1/relation-off-checkpoint.json",
                    "sha256": "5" * 64,
                },
                "treatment_ablation_dev": {
                    "path": "/workspace/uor-r4/runs/claude/a8-dose-relation-seed1/relation-off-dev.json",
                    "sha256": "6" * 64,
                },
                "paired_discriminator": {
                    "path": "/workspace/uor-r4/runs/claude/a8-dose-paired-dev-score.json",
                    "sha256": "7" * 64,
                },
                "pm_receipt_url": "https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6259999999",
            },
            "result": {
                "schema": "uor-r4.attention-transfer-result/2",
                "status": "PAIRED_FRESH_OBSERVED_PENDING_PM_DISCRIMINATOR",
                "snapshot_unchanged": True,
                "updates_applied": 0,
                "treatment_fresh_evaluations": 1,
                "control_fresh_evaluations": 1,
                "gpu_fresh_evaluations": 2,
                "ablation_fresh_evaluations": 0,
                "paired_fresh": {
                    "psucc_rows": 860,
                    "b3_r": 290,
                    "b3_c": 280,
                    "b3_i": 250,
                    "b_fresh": 8,
                    "c_fresh": 34,
                    "both_correct": 40,
                    "both_wrong": 168,
                    "floor": 20,
                    "treatment_direction_and_floor_met": True,
                },
            },
        },
    }


def run_self_tests(repo_root: Path, packet_dir: Path | None = None) -> dict[str, Any]:
    """Run 40 positive and negative self-tests covering all A8 pre-run, Q1, D34, D40, and decision rules."""
    results: list[dict[str, Any]] = []

    def record(name: str, passed: bool, detail: str) -> None:
        require(passed, f"SELF-TEST FAILED [{name}]: {detail}")
        results.append({"test": name, "status": "PASS", "detail": detail})

    # T1: Staged contract verification on actual repository + optional executor packet
    staged = verify_staged_contract(repo_root, packet_dir=packet_dir)
    record("01_staged_contract_and_sheet_fit", staged["status"] == "PASS", staged["sheet_sha256"])

    # T2: Unregistered pinned table refusal (S8 `a185bdca…`)
    a8_rel = read_json_checked(
        repo_root / "docs" / "labs" / "a8-training-configs-2026-10-19" / "evidence" / "a8-relation-seed1.json"
    )
    bad_s8 = copy.deepcopy(a8_rel)
    bad_s8["codebook_initialization"]["table"]["sha256"] = EXPECTED_S8_TABLE_FILE_SHA256
    try:
        registered_sheet_for_config(bad_s8)
        record("02_unregistered_pinned_table_refused", False, "failed to refuse S8")
    except VerificationError as exc:
        record("02_unregistered_pinned_table_refused", True, str(exc))

    # T3: Valid pre-run gate bundle PASS
    base_prerun = build_synthetic_prerun_bundle(repo_root)
    res_prerun = verify_prerun_bundle(base_prerun)
    record(
        "03_prerun_bundle_valid_pass",
        res_prerun["verdict"] == "PRERUN_GATES_PASS_READY_FOR_OWNER_RELEASE"
        and res_prerun["ready_for_owner_release"] is True,
        res_prerun["verdict"],
    )

    # T4: Pre-run unpinned G-Init missing -> INSTRUMENT_FAULT_D31
    b_no_unp = copy.deepcopy(base_prerun)
    del b_no_unp["arms"]["relation"]["unpinned_init_checkpoint"]
    res_no_unp = verify_prerun_bundle(b_no_unp)
    record(
        "04_prerun_missing_unpinned_g_init_fault",
        res_no_unp["verdict"] == "INSTRUMENT_FAULT_D31"
        and "relation_g_init_unpinned_missing" in res_no_unp["faults"],
        str(res_no_unp["faults"]),
    )

    # T5: Pre-run unpinned G-Init routes/gates mismatch -> INSTRUMENT_FAULT_D31
    b_bad_unp = copy.deepcopy(base_prerun)
    b_bad_unp["arms"]["frozen_zero"]["unpinned_init_checkpoint"]["model"]["gates"][0] = 0
    res_bad_unp = verify_prerun_bundle(b_bad_unp)
    record(
        "05_prerun_unpinned_gates_mismatch_fault",
        res_bad_unp["verdict"] == "INSTRUMENT_FAULT_D31"
        and "frozen_zero_g_init_unpinned_routes_gates_seed_epoch_mismatch" in res_bad_unp["faults"],
        str(res_bad_unp["faults"]),
    )

    # T6: Pre-run unpinned G-Init masquerading A8 table -> INSTRUMENT_FAULT_D31
    b_dup_tbl = copy.deepcopy(base_prerun)
    b_dup_tbl["arms"]["relation"]["unpinned_init_checkpoint"]["model"]["table"] = list(
        b_dup_tbl["arms"]["relation"]["init_checkpoint"]["model"]["table"]
    )
    res_dup_tbl = verify_prerun_bundle(b_dup_tbl)
    record(
        "06_prerun_unpinned_same_a8_table_rejected",
        res_dup_tbl["verdict"] == "INSTRUMENT_FAULT_D31"
        and "relation_g_init_unpinned_routes_gates_seed_epoch_mismatch" in res_dup_tbl["faults"],
        str(res_dup_tbl["faults"]),
    )

    # T7: Pre-run reproduction gate miss (B3_init(P_succ) != 1351 or top1 != 133) -> INSTRUMENT_FAULT_D31
    b_repro = copy.deepcopy(base_prerun)
    b_repro["arms"]["frozen_zero"]["b3"]["psucc"]["target_admitted"] = 1350
    res_repro = verify_prerun_bundle(b_repro)
    record(
        "07_prerun_reproduction_gate_miss_fault",
        res_repro["verdict"] == "INSTRUMENT_FAULT_D31"
        and "frozen_zero_reproduction_gate_miss" in res_repro["faults"]
        and res_repro["mechanism_negative"] is False,
        str(res_repro["faults"]),
    )

    # T8: Pre-run Gate22 probe inert (`any_candidate_cost_differs_from_identity == False`) -> INSTRUMENT_FAULT_D31
    b_inert = copy.deepcopy(base_prerun)
    b_inert["arms"]["relation"]["probes"]["gate_22"]["any_candidate_cost_differs_from_identity"] = False
    b_inert["arms"]["relation"]["probes"]["gate_22"]["candidate_losses"] = [59136, 59136, 59136] + [U64_MAX] * 13
    res_inert = verify_prerun_bundle(b_inert)
    record(
        "08_prerun_gate22_probe_inert_fault",
        res_inert["verdict"] == "INSTRUMENT_FAULT_D31"
        and "relation_probe_inert_under_bound_predicate" in res_inert["faults"],
        str(res_inert["faults"]),
    )

    # T9: Pre-run `qualify-relation` INIT LOCK tamper (`gates[22] = 1` or `gates[3] = 1`) -> INSTRUMENT_FAULT_D31
    b_lock = copy.deepcopy(base_prerun)
    b_lock["relation_controls"]["controls"][8]["details"]["init_locks"][0]["gates"][22] = 1
    res_lock = verify_prerun_bundle(b_lock)
    record(
        "09_prerun_init_lock_mutant_gate22_fault",
        res_lock["verdict"] == "INSTRUMENT_FAULT_D31"
        and "qualify_relation_controls_failed" in res_lock["faults"],
        str(res_lock["faults"]),
    )

    # T10: Pre-run neg-b reusing PrimeZeta SHA instead of A8 neg-b -> INSTRUMENT_FAULT_D31
    b_negb = copy.deepcopy(base_prerun)
    b_negb["neg_b"]["table_file_sha256"] = (
        "070478a09b25fbe230055d3fff44f799c42b270e8fc89ff795705271970aa806"
    )
    res_negb = verify_prerun_bundle(b_negb)
    record(
        "10_prerun_neg_b_wrong_table_pin_fault",
        res_negb["verdict"] == "INSTRUMENT_FAULT_D31"
        and "g_init_neg_b_gpu_status6_refusal_failed" in res_negb["faults"],
        str(res_negb["faults"]),
    )

    # T11: Pre-run metadata-only reference battery claiming full PASS -> INSTRUMENT_FAULT_D31
    b_ref_claim = copy.deepcopy(base_prerun)
    b_ref_claim["reference_battery"]["full_reference_pass_claimed"] = True
    res_ref_claim = verify_prerun_bundle(b_ref_claim)
    record(
        "11_prerun_metadata_only_full_reference_claim_refused",
        res_ref_claim["verdict"] == "INSTRUMENT_FAULT_D31"
        and "reference_battery_deferred_contract_violation" in res_ref_claim["faults"],
        str(res_ref_claim["faults"]),
    )

    # T12: Pre-run full `SCORED_PREFLIGHT_DEV` reference battery PASS
    b_ref_scored = copy.deepcopy(base_prerun)
    b_ref_scored["reference_battery"] = {
        "mode": "SCORED_PREFLIGHT_DEV",
        "score": {
            "schema": "uor-r4.attention-preflight-reference-score/1",
            "status": "VALIDATED_PREFLIGHT_DEV_CAPTURE_AND_SEALED_D34_SCORE",
            "pins": {
                "d34_sha256": EXPECTED_D34_RECORD_SHA256,
                "ordered_row_binding_sha256": EXPECTED_ORDERED_ROW_BINDING_SHA256,
            },
            "model_compute_executed_by_scorer": False,
            "updates_applied": 0,
            "same_cell_candidate_reranking": {
                "status": "SCORED_FROM_GPU_TOKEN_CELLS_AND_REGISTERED_CORPUS_TABLES",
                "corpus_pins": {"conservation_ok": True},
            },
        },
    }
    res_ref_scored = verify_prerun_bundle(b_ref_scored)
    record(
        "12_prerun_scored_preflight_reference_pass",
        res_ref_scored["verdict"] == "PRERUN_GATES_PASS_READY_FOR_OWNER_RELEASE"
        and res_ref_scored["reference_battery_status"] == "PASS_FULL_PREFLIGHT_REFERENCE_SCORED",
        res_ref_scored["reference_battery_status"],
    )

    # T13: Pre-run plan/review payload SHA tamper -> INSTRUMENT_FAULT_D31
    b_rev = copy.deepcopy(base_prerun)
    b_rev["plans"]["relation"]["review"]["reviewed_plan_payload_sha256"] = "0" * 64
    res_rev = verify_prerun_bundle(b_rev)
    record(
        "13_prerun_review_payload_sha_tamper_fault",
        res_rev["verdict"] == "INSTRUMENT_FAULT_D31"
        and "plans_or_reviews_validation_failed" in res_rev["faults"],
        str(res_rev["faults"]),
    )

    # T14: Pre-run D40 owner release non-Casey tamper -> INSTRUMENT_FAULT_D31
    b_owner = copy.deepcopy(base_prerun)
    b_owner["plans"]["frozen_zero"]["owner"]["owner"] = "claude"
    res_owner = verify_prerun_bundle(b_owner)
    record(
        "14_prerun_d40_owner_non_casey_refused",
        res_owner["verdict"] == "INSTRUMENT_FAULT_D31"
        and "plans_or_reviews_validation_failed" in res_owner["faults"],
        str(res_owner["faults"]),
    )

    # T15: Dose Q1 diagnostic stop (admission collapse `< 0.5 * 1351`, i.e. `675 <= 675.5`)
    base_dose = build_synthetic_dose_bundle(repo_root)
    d_q1_stop = copy.deepcopy(base_dose)
    d_q1_stop["arms"]["relation"]["q1_record"]["q1_b3"]["psucc"]["target_admitted"] = 675
    d_q1_stop["arms"]["relation"]["q1_record"]["q1_b3"]["full"]["target_admitted"] = 675 + 2500
    d_q1_stop["arms"]["relation"]["q1_record"]["decision"] = compute_q1_decision(
        EXPECTED_A8_CP0_PSUCC_ADMITTED, 675, 30, 12
    )
    d_q1_stop["arms"]["relation"]["report"] = {
        "status": "Q1_STOP_FOR_DIAGNOSIS",
        "completed_batches": CANONICAL_Q1_BATCHES,
        "mechanism_negative": False,
        "fresh_transfer_evaluations": 0,
    }
    res_q1_stop = verify_dose_bundle(d_q1_stop)
    record(
        "15_dose_q1_diagnostic_stop_admission_collapse",
        res_q1_stop["verdict"] == "Q1_DIAGNOSTIC_STOP"
        and res_q1_stop["mechanism_negative"] is False,
        res_q1_stop["verdict"],
    )

    # T16: Dose Q1 stop bypassed (continued to 16,404 despite `diagnostic_stop == True`) -> INSTRUMENT_FAULT
    d_q1_bypass = copy.deepcopy(d_q1_stop)
    d_q1_bypass["arms"]["relation"]["report"]["completed_batches"] = CANONICAL_ATTENTION_BATCHES
    res_q1_bypass = verify_dose_bundle(d_q1_bypass)
    record(
        "16_dose_q1_stop_bypass_fault",
        res_q1_bypass["verdict"] == "INSTRUMENT_FAULT"
        and "relation_q1_stop_bypassed_or_malformed" in res_q1_bypass["faults"],
        str(res_q1_bypass["faults"]),
    )

    # T17: Dose Q1 zero gate sensitivity (`combine_sensitive == 0 and relation_sensitive == 0`) -> Q1_DIAGNOSTIC_STOP
    d_q1_nosens = copy.deepcopy(base_dose)
    q1_c = d_q1_nosens["arms"]["relation"]["q1_record"]["counters"]
    q1_c["gate_combine"]["any_candidate_cost_differs_from_identity"] = 0
    q1_c["gate_combine"]["accepted_updates"] = 0
    q1_c["gate_relation"]["any_candidate_cost_differs_from_identity"] = 0
    q1_c["gate_relation"]["accepted_updates"] = 0
    d_q1_nosens["arms"]["relation"]["q1_record"]["checkpoint"]["cursor"]["accepted_updates"] = 80
    d_q1_nosens["arms"]["relation"]["q1_record"]["decision"] = compute_q1_decision(
        EXPECTED_A8_CP0_PSUCC_ADMITTED, 1100, 0, 0
    )
    d_q1_nosens["arms"]["relation"]["report"] = {
        "status": "Q1_STOP_FOR_DIAGNOSIS",
        "completed_batches": CANONICAL_Q1_BATCHES,
        "mechanism_negative": False,
        "fresh_transfer_evaluations": 0,
    }
    res_q1_nosens = verify_dose_bundle(d_q1_nosens)
    record(
        "17_dose_q1_zero_gate_sensitivity_stop",
        res_q1_nosens["verdict"] == "Q1_DIAGNOSTIC_STOP"
        and res_q1_nosens["q1_decisions"]["relation"]["no_combine_or_relation_sensitivity"] is True,
        res_q1_nosens["verdict"],
    )

    # T18: Precedence 0 — FrozenZero control lock fault (`refused_relation_proposals != 1130`)
    d_fz_lock = copy.deepcopy(base_dose)
    d_fz_lock["arms"]["frozen_zero"]["report"]["sensitivity"]["refused_relation_proposals"] = 1129
    res_fz_lock = verify_dose_bundle(d_fz_lock)
    record(
        "18_dose_prec0_frozen_zero_refusal_count_fault",
        res_fz_lock["verdict"] == "INSTRUMENT_FAULT" and res_fz_lock["precedence"] == 0,
        str(res_fz_lock["faults"]),
    )

    # T19: Precedence 0 — FrozenZero nonzero relation trit in final snapshot -> INSTRUMENT_FAULT
    d_fz_trit = copy.deepcopy(base_dose)
    d_fz_trit["arms"]["frozen_zero"]["final_checkpoint"]["model"]["gates"][22] = 1
    res_fz_trit = verify_dose_bundle(d_fz_trit)
    record(
        "19_dose_prec0_frozen_zero_nonzero_relation_trit_fault",
        res_fz_trit["verdict"] == "INSTRUMENT_FAULT"
        and "frozen_zero_nonzero_relation_trit_in_final_snapshot" in res_fz_trit["faults"],
        str(res_fz_trit["faults"]),
    )

    # T20: Precedence 0 — Zero-relation ablation snapshot didn't zero relation trits -> INSTRUMENT_FAULT
    d_abl_bad = copy.deepcopy(base_dose)
    d_abl_bad["arms"]["relation"]["ablation_checkpoint_record"]["checkpoint"]["model"]["gates"][20] = 1
    res_abl_bad = verify_dose_bundle(d_abl_bad)
    record(
        "20_dose_prec0_ablation_nonzero_relation_trit_fault",
        res_abl_bad["verdict"] == "INSTRUMENT_FAULT"
        and "relation_ablation_snapshot_mismatch" in res_abl_bad["faults"],
        str(res_abl_bad["faults"]),
    )

    # T21: Precedence 1 — `B3_I < 200` -> `UNDERPOWERED_OR_INVALID`
    d_under = copy.deepcopy(base_dose)
    del d_under["fresh_transfer"]
    b3_i_low = 180
    r_only_u = 820
    c_only_u = 800
    neither_u = CANONICAL_DEV_PSUCC_ROWS - (b3_i_low + r_only_u + c_only_u)
    d_under["paired_dev_score"]["admission_summary_psucc"] = {
        "psucc_rows": CANONICAL_DEV_PSUCC_ROWS,
        "b3_r": 1000,
        "b3_c": 980,
        "b3_i_intersection": b3_i_low,
        "r_only": r_only_u,
        "c_only": c_only_u,
        "neither": neither_u,
        "b3_i_meets_minimum_200": False,
        "mde_rel_24_over_b3_i": {"numerator": 24, "denominator": b3_i_low},
    }
    pc_u = d_under["paired_dev_score"]["paired_comparisons"]
    pc_u["relation_vs_control_on_b3_i_top1"] = _make_2x2(20, 40, 10, b3_i_low - 70, 24)
    pc_u["ablation"]["relation_vs_ablation_on_b3_i_top1"] = _make_2x2(20, 40, 10, b3_i_low - 70, 24)
    pc_u["same_cell_reranked"]["relation_vs_t2_r_same_cell_on_b3_i_top1"] = _make_2x2(
        20, 40, 10, b3_i_low - 70, 24
    )
    pc_u["same_cell_reranked"]["control_vs_t2_c_same_cell_on_b3_i_top1"] = _make_2x2(
        20, 10, 40, b3_i_low - 70, 24
    )
    res_under = verify_dose_bundle(d_under)
    record(
        "21_dose_prec1_b3_i_below_200_underpowered",
        res_under["verdict"] == "UNDERPOWERED_OR_INVALID"
        and res_under["precedence"] == 1
        and res_under["mechanism_negative"] is False,
        res_under["verdict"],
    )

    # T22: Precedence 2 — `H_null` (`T2_I >= Relation + F_{T,I}` and `T2_I >= control + F_{T,I}`)
    d_null = copy.deepcopy(base_dose)
    del d_null["fresh_transfer"]
    sc_n = d_null["paired_dev_score"]["paired_comparisons"]["same_cell_reranked"]
    sc_n["relation_vs_t2_r_same_cell_on_b3_i_top1"] = _make_2x2(80, 15, 60, 900 - 155, 24)
    sc_n["control_vs_t2_c_same_cell_on_b3_i_top1"] = _make_2x2(80, 12, 58, 900 - 150, 24)
    res_null = verify_dose_bundle(d_null)
    record(
        "22_dose_prec2_h_null_frequency_explains_panel",
        res_null["verdict"] == "H_null"
        and res_null["precedence"] == 2
        and res_null["configuration_closed_d22"] is False,
        res_null["verdict"],
    )

    # T23: Precedence 3 — `H_control` via ablation drop `< F_A` (within-checkpoint zero-relation ablation)
    d_ctrl_abl = copy.deepcopy(base_dose)
    del d_ctrl_abl["fresh_transfer"]
    d_ctrl_abl["paired_dev_score"]["paired_comparisons"]["ablation"][
        "relation_vs_ablation_on_b3_r_top1"
    ] = _make_2x2(150, 12, 8, 1000 - 170, 24)
    res_ctrl_abl = verify_dose_bundle(d_ctrl_abl)
    record(
        "23_dose_prec3_h_control_ablation_within_floor",
        res_ctrl_abl["verdict"] == "H_control"
        and res_ctrl_abl["precedence"] == 3
        and res_ctrl_abl["configuration_closed_d22"] is True
        and res_ctrl_abl["mechanism_negative"] is False,
        res_ctrl_abl["verdict"],
    )

    # T24: Precedence 3 — `H_control` via `control >= Relation + F_I`
    d_ctrl_win = copy.deepcopy(base_dose)
    del d_ctrl_win["fresh_transfer"]
    d_ctrl_win["paired_dev_score"]["paired_comparisons"]["relation_vs_control_on_b3_i_top1"] = (
        _make_2x2(100, 15, 55, 900 - 170, 24)
    )
    res_ctrl_win = verify_dose_bundle(d_ctrl_win)
    record(
        "24_dose_prec3_h_control_beats_relation",
        res_ctrl_win["verdict"] == "H_control" and res_ctrl_win["precedence"] == 3,
        res_ctrl_win["verdict"],
    )

    # T25: Precedence 4 — `H_mixed_INCONCLUSIVE` (`Relation >= control + F_I`, but `< T2_R + F_{T,R}`)
    d_mixed_t2 = copy.deepcopy(base_dose)
    del d_mixed_t2["fresh_transfer"]
    d_mixed_t2["paired_dev_score"]["paired_comparisons"]["same_cell_reranked"][
        "relation_vs_t2_r_same_cell_on_b3_r_top1"
    ] = _make_2x2(120, 25, 15, 1000 - 160, 24)
    res_mixed_t2 = verify_dose_bundle(d_mixed_t2)
    record(
        "25_dose_prec4_h_mixed_below_t2_r_floor",
        res_mixed_t2["verdict"] == "H_mixed_INCONCLUSIVE" and res_mixed_t2["precedence"] == 4,
        res_mixed_t2["verdict"],
    )

    # T26: Precedence 4 — `H_mixed_INCONCLUSIVE` via complement regression beyond `F_C`
    d_mixed_comp = copy.deepcopy(base_dose)
    del d_mixed_comp["fresh_transfer"]
    d_mixed_comp["paired_dev_score"]["paired_comparisons"][
        "relation_vs_control_complement_top1"
    ] = _make_2x2(180, 10, 45, CANONICAL_DEV_COMP_ROWS - 235, 22)
    res_mixed_comp = verify_dose_bundle(d_mixed_comp)
    record(
        "26_dose_prec4_h_mixed_complement_regression",
        res_mixed_comp["verdict"] == "H_mixed_INCONCLUSIVE" and res_mixed_comp["precedence"] == 4,
        res_mixed_comp["verdict"],
    )

    # T27: Precedence 5 (DEV only, before fresh transfer) -> `H_geom_DEV_PENDING_PAIRED_FRESH_TRANSFER`
    d_hgeom_dev = copy.deepcopy(base_dose)
    del d_hgeom_dev["fresh_transfer"]
    res_hgeom_dev = verify_dose_bundle(d_hgeom_dev)
    record(
        "27_dose_prec5_h_geom_dev_pending_fresh_transfer",
        res_hgeom_dev["verdict"] == "H_geom_DEV_PENDING_PAIRED_FRESH_TRANSFER"
        and res_hgeom_dev["keep"] is False,
        res_hgeom_dev["verdict"],
    )

    # T28: Unauthorized fresh transfer when DEV verdict is not `H_geom` -> `INSTRUMENT_FAULT`
    d_unauth_fresh = copy.deepcopy(d_mixed_t2)
    d_unauth_fresh["fresh_transfer"] = copy.deepcopy(base_dose["fresh_transfer"])
    res_unauth_fresh = verify_dose_bundle(d_unauth_fresh)
    record(
        "28_dose_unauthorized_fresh_transfer_on_non_h_geom_fault",
        res_unauth_fresh["verdict"] == "INSTRUMENT_FAULT"
        and any("unauthorized_fresh_transfer" in f for f in res_unauth_fresh["faults"]),
        str(res_unauth_fresh["faults"]),
    )

    # T29: Precedence 5 (DEV `H_geom` + valid PM `TransferDecision` + passing paired fresh transfer) -> `H_geom` (`keep = True`)
    res_hgeom_full = verify_dose_bundle(base_dose)
    record(
        "29_dose_prec5_h_geom_confirmed_on_fresh_transfer",
        res_hgeom_full["verdict"] == "H_geom"
        and res_hgeom_full["keep"] is True
        and res_hgeom_full["claim_scope"] == CLAIM_SCOPE_H_GEOM,
        res_hgeom_full["verdict"],
    )

    # T30: Precedence 6 (DEV `H_geom` + valid PM `TransferDecision`, but fresh transfer misses floor) -> `DEV_ONLY_DIAGNOSTIC_FRESH_FAILED`
    d_fresh_fail = copy.deepcopy(base_dose)
    d_fresh_fail["fresh_transfer"]["result"]["paired_fresh"] = {
        "psucc_rows": 860,
        "b3_r": 290,
        "b3_c": 280,
        "b3_i": 250,
        "b_fresh": 18,
        "c_fresh": 26,
        "both_correct": 40,
        "both_wrong": 166,
        "floor": 20,
        "treatment_direction_and_floor_met": False,
    }
    res_fresh_fail = verify_dose_bundle(d_fresh_fail)
    record(
        "30_dose_prec6_dev_only_diagnostic_when_fresh_fails",
        res_fresh_fail["verdict"] == "DEV_ONLY_DIAGNOSTIC_FRESH_FAILED"
        and res_fresh_fail["precedence"] == 6
        and res_fresh_fail["keep"] is False
        and res_fresh_fail["mechanism_negative"] is False,
        res_fresh_fail["verdict"],
    )

    # T31 & T32: CLI file bundle exit-code checks (--prerun-bundle and --dose-bundle)
    with tempfile.TemporaryDirectory() as tmpdir:
        tmp_path = Path(tmpdir)
        prerun_ok_path = tmp_path / "prerun_ok.json"
        prerun_bad_path = tmp_path / "prerun_bad.json"
        dose_ok_path = tmp_path / "dose_ok.json"
        dose_fault_path = tmp_path / "dose_fault.json"
        prerun_ok_path.write_text(json.dumps(base_prerun), encoding="utf-8")
        prerun_bad_path.write_text(json.dumps(b_rev), encoding="utf-8")
        dose_ok_path.write_text(json.dumps(base_dose), encoding="utf-8")
        dose_fault_path.write_text(json.dumps(d_fz_lock), encoding="utf-8")

        with contextlib.redirect_stdout(io.StringIO()):
            rc_prerun_ok = main(["--repo-root", str(repo_root), "--prerun-bundle", str(prerun_ok_path)])
            rc_prerun_bad = main(["--repo-root", str(repo_root), "--prerun-bundle", str(prerun_bad_path)])
            rc_prerun_bad_allow = main(
                [
                    "--repo-root",
                    str(repo_root),
                    "--prerun-bundle",
                    str(prerun_bad_path),
                    "--allow-fault-exit-zero",
                ]
            )
            rc_dose_ok = main(["--repo-root", str(repo_root), "--dose-bundle", str(dose_ok_path)])
            rc_dose_fault = main(["--repo-root", str(repo_root), "--dose-bundle", str(dose_fault_path)])

        record(
            "31_cli_prerun_bundle_exit_codes",
            rc_prerun_ok == 0 and rc_prerun_bad == 1 and rc_prerun_bad_allow == 0,
            f"ok={rc_prerun_ok}, bad={rc_prerun_bad}, allow={rc_prerun_bad_allow}",
        )
        record(
            "32_cli_dose_bundle_exit_codes",
            rc_dose_ok == 0 and rc_dose_fault == 1,
            f"ok={rc_dose_ok}, fault={rc_dose_fault}",
        )

    # T33: ExecutionIdentity struct field order & vast-56670997 12-digest reproduction
    rel_plan_b = base_prerun["plans"]["relation"]
    fz_plan_b = base_prerun["plans"]["frozen_zero"]
    record(
        "33_execution_identity_order_and_vast_56670997_digests",
        canonical_plan_sha256(rel_plan_b["plan"]) == EXPECTED_VAST_56670997_REL_PLAN_CANONICAL_SHA256
        and canonical_plan_sha256(fz_plan_b["plan"]) == EXPECTED_VAST_56670997_FZ_PLAN_CANONICAL_SHA256
        and rel_plan_b["manifest_sha256"] == EXPECTED_VAST_56670997_REL_ADMISSION_SHA256
        and fz_plan_b["manifest_sha256"] == EXPECTED_VAST_56670997_FZ_ADMISSION_SHA256,
        f"rel={rel_plan_b['manifest_sha256'][:12]}, fz={fz_plan_b['manifest_sha256'][:12]}",
    )

    # T34: Gate probe u64::MAX sentinel slot filtering rejects inert gate (both forged any_diff=True and any_diff=False)
    b_u64max_forged = copy.deepcopy(base_prerun)
    b_u64max_forged["arms"]["relation"]["probes"]["gate_22"]["candidate_losses"] = (
        [59136, 59136, 59136] + [U64_MAX] * 13
    )
    b_u64max_forged["arms"]["relation"]["probes"]["gate_22"][
        "any_candidate_cost_differs_from_identity"
    ] = True
    res_u64max_forged = verify_prerun_bundle(b_u64max_forged)
    b_u64max_inert = copy.deepcopy(b_u64max_forged)
    b_u64max_inert["arms"]["relation"]["probes"]["gate_22"][
        "any_candidate_cost_differs_from_identity"
    ] = False
    res_u64max_inert = verify_prerun_bundle(b_u64max_inert)
    record(
        "34_prerun_probe_u64_max_sentinel_inert_gate_rejected",
        res_u64max_forged["verdict"] == "INSTRUMENT_FAULT_D31"
        and "relation_probe_schema_invalid" in res_u64max_forged["faults"]
        and res_u64max_inert["verdict"] == "INSTRUMENT_FAULT_D31"
        and "relation_probe_inert_under_bound_predicate" in res_u64max_inert["faults"],
        f"forged={res_u64max_forged['faults']}, inert={res_u64max_inert['faults']}",
    )

    # T35: Nested coverage-census `cells.status != FROM_GPU_TOKEN_CELL_RECEIPT` rejected
    b_bad_census = copy.deepcopy(base_prerun)
    b_bad_census["arms"]["relation"]["census"]["cells"]["status"] = "HOST_RECONSTRUCTED"
    res_bad_census = verify_prerun_bundle(b_bad_census)
    record(
        "35_prerun_nested_census_status_tamper_rejected",
        res_bad_census["verdict"] == "INSTRUMENT_FAULT_D31"
        and "relation_g_occ_mismatch" in res_bad_census["faults"],
        str(res_bad_census["faults"]),
    )

    # T36: Dose bundle missing D40 `RELEASED_BY_CASEY_D40` (`STILL_REQUIRED`) -> INSTRUMENT_FAULT
    d_no_d40 = copy.deepcopy(base_dose)
    del d_no_d40["plans"]["relation"]["owner"]
    del d_no_d40["plans"]["relation"]["admission"]
    res_no_d40 = verify_dose_bundle(d_no_d40)
    record(
        "36_dose_missing_d40_owner_release_fault",
        res_no_d40["verdict"] == "INSTRUMENT_FAULT"
        and any(
            f.startswith("missing_or_unreleased_d40_owner_admission_in_dose_bundle")
            for f in res_no_d40["faults"]
        ),
        str(res_no_d40["faults"]),
    )

    # T37: Dose bundle report `manifest_sha256` mismatch against `admission.json` -> INSTRUMENT_FAULT
    d_bad_manifest = copy.deepcopy(base_dose)
    d_bad_manifest["arms"]["frozen_zero"]["report"]["manifest_sha256"] = "0" * 64
    res_bad_manifest = verify_dose_bundle(d_bad_manifest)
    record(
        "37_dose_report_manifest_sha256_mismatch_fault",
        res_bad_manifest["verdict"] == "INSTRUMENT_FAULT"
        and "frozen_zero_report_manifest_sha256_mismatch" in res_bad_manifest["faults"],
        str(res_bad_manifest["faults"]),
    )

    # T38: Dose bundle `TransferDecision` missing required FilePin -> INSTRUMENT_FAULT
    d_missing_pin = copy.deepcopy(base_dose)
    del d_missing_pin["fresh_transfer"]["decision"]["treatment_ablation_checkpoint"]
    res_missing_pin = verify_dose_bundle(d_missing_pin)
    record(
        "38_dose_fresh_transfer_missing_file_pin_fault",
        res_missing_pin["verdict"] == "INSTRUMENT_FAULT"
        and "fresh_transfer_pm_decision_header_or_pins_invalid" in res_missing_pin["faults"],
        str(res_missing_pin["faults"]),
    )

    # T39: Dose bundle fresh transfer `b3_i < 200` underpowered -> DEV_ONLY_DIAGNOSTIC_FRESH_FAILED (never `H_geom`)
    d_fresh_under = copy.deepcopy(base_dose)
    d_fresh_under["fresh_transfer"]["result"]["paired_fresh"] = {
        "psucc_rows": 860,
        "b3_r": 290,
        "b3_c": 280,
        "b3_i": 199,
        "b_fresh": 8,
        "c_fresh": 34,
        "both_correct": 40,
        "both_wrong": 117,
        "floor": 20,
        "treatment_direction_and_floor_met": True,
    }
    res_fresh_under = verify_dose_bundle(d_fresh_under)
    record(
        "39_dose_fresh_transfer_b3_i_below_200_underpowered_fault",
        res_fresh_under["verdict"] == "DEV_ONLY_DIAGNOSTIC_FRESH_FAILED"
        and res_fresh_under.get("underpowered_fresh_b3_i") is True
        and res_fresh_under["keep"] is False,
        f"verdict={res_fresh_under['verdict']}, underpowered={res_fresh_under.get('underpowered_fresh_b3_i')}",
    )

    # T40: Live vast-56670997 pre-run artifacts & committed #2767 A8 dose receipt verification
    committed_dose_res = verify_committed_a8_dose_receipt(repo_root)
    default_pkt = packet_dir or Path("/Users/casey.allard/uor-r4-local/a8-dose-executor-packet-20261019")
    gates_log_p = default_pkt / "init-lock-selector-diagnosis" / "original-gates.log"
    rel_ctrl_p = default_pkt / "init-lock-selector-diagnosis" / "original-relation-controls.json"
    if gates_log_p.is_file() and rel_ctrl_p.is_file():
        vast_res = verify_vast_56670997_prerun_artifacts(gates_log_p, rel_ctrl_p, repo_root)
        record(
            "40_live_vast_56670997_prerun_and_committed_dose_receipt_pass",
            vast_res["status"] == "PASS"
            and vast_res["bar_i"] == 86
            and committed_dose_res["status"] == "PASS"
            and committed_dose_res["verdict"] == "Q1_DIAGNOSTIC_STOP",
            f"gates_log={vast_res['gates_log_sha256'][:12]}, bar_i={vast_res['bar_i']}, dose={committed_dose_res['verdict']}",
        )
    else:
        record(
            "40_live_vast_56670997_prerun_and_committed_dose_receipt_pass",
            res_prerun["bar_i_and_g_cap"]["bar_i_psucc"] == 86
            and res_prerun["status"] == "PASS"
            and committed_dose_res["status"] == "PASS"
            and committed_dose_res["verdict"] == "Q1_DIAGNOSTIC_STOP",
            f"dose={committed_dose_res['verdict']} (external packet dir not mounted)",
        )

    return {
        "status": "PASS",
        "total_tests": len(results),
        "passed_tests": len(results),
        "tests": results,
    }


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        description="Host-only consumer verifier for A8 address-cells learning dose & pre-run gates."
    )
    parser.add_argument(
        "--repo-root",
        type=Path,
        default=Path("."),
        help="Repository root directory (default: current directory).",
    )
    parser.add_argument(
        "--runbook-dir",
        type=Path,
        default=None,
        help="Path to docs/labs/a8-dose-runbook-2026-10-19.",
    )
    parser.add_argument(
        "--configs-dir",
        type=Path,
        default=None,
        help="Path to docs/labs/a8-training-configs-2026-10-19.",
    )
    parser.add_argument(
        "--packet-dir",
        type=Path,
        default=None,
        help="Optional path to external executor packet (/Users/casey.allard/uor-r4-local/a8-dose-executor-packet-20261019).",
    )
    parser.add_argument(
        "--gates-log",
        type=Path,
        default=None,
        help="Optional path to vast-56670997 original-gates.log.",
    )
    parser.add_argument(
        "--relation-controls",
        type=Path,
        default=None,
        help="Optional path to vast-56670997 original-relation-controls.json.",
    )
    parser.add_argument(
        "--prerun-bundle",
        type=Path,
        default=None,
        help="Optional JSON bundle of pre-run gate receipts to grade.",
    )
    parser.add_argument(
        "--dose-bundle",
        type=Path,
        default=None,
        help="Optional JSON bundle of Q1/full-dose + D34 paired-dev receipts to grade.",
    )
    parser.add_argument(
        "--allow-fault-exit-zero",
        action="store_true",
        help="Return exit code 0 even when --prerun-bundle fails or --dose-bundle grades INSTRUMENT_FAULT.",
    )
    parser.add_argument(
        "--self-test",
        action="store_true",
        help="Run the 40-case positive and negative self-test suite.",
    )
    args = parser.parse_args(argv)

    out: dict[str, Any] = {
        "schema": "uor-r4.a8-address-cells-dose-consumer-verifier/1",
        "cuda_context_created": False,
        "model_constructed": False,
        "training": "HELD",
    }

    staged = verify_staged_contract(
        repo_root=args.repo_root,
        runbook_dir=args.runbook_dir,
        configs_dir=args.configs_dir,
        packet_dir=args.packet_dir,
    )
    out["staged_contract"] = staged

    if args.gates_log is not None or args.relation_controls is not None:
        require(
            args.gates_log is not None and args.relation_controls is not None,
            "--gates-log and --relation-controls must be supplied together",
        )
        out["vast_56670997_prerun"] = verify_vast_56670997_prerun_artifacts(
            gates_log_path=args.gates_log,
            relation_controls_path=args.relation_controls,
            repo_root=args.repo_root,
        )

    if args.self_test:
        out["self_tests"] = run_self_tests(args.repo_root, packet_dir=args.packet_dir)

    exit_code = 0
    if args.prerun_bundle is not None:
        prerun_doc = read_json_checked(args.prerun_bundle, "prerun bundle")
        prerun_res = verify_prerun_bundle(prerun_doc)
        out["prerun_verification"] = prerun_res
        if prerun_res.get("status") != "PASS" and not args.allow_fault_exit_zero:
            exit_code = 1

    if args.dose_bundle is not None:
        dose_doc = read_json_checked(args.dose_bundle, "dose bundle")
        dose_res = verify_dose_bundle(dose_doc)
        out["dose_verification"] = dose_res
        if dose_res.get("verdict") == "INSTRUMENT_FAULT" and not args.allow_fault_exit_zero:
            exit_code = 1

    print(json.dumps(out, indent=2))
    return exit_code


if __name__ == "__main__":
    try:
        sys.exit(main())
    except VerificationError as err:
        print(
            json.dumps(
                {
                    "schema": "uor-r4.a8-address-cells-dose-consumer-verifier/1",
                    "status": "FAIL",
                    "error": str(err),
                    "cuda_context_created": False,
                    "model_constructed": False,
                    "training": "HELD",
                },
                indent=2,
            ),
            file=sys.stderr,
        )
        sys.exit(1)
