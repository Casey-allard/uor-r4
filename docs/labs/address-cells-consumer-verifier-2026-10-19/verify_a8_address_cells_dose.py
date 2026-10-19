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
4. Per-Checkpoint Cell Census, `steps.jsonl` Slot Bijection, Step-175 Bifurcation & Two-Sided /
   4-Way Cross-Arm `P_succ` Admission Decomposition (`verify_a8_census_and_attribution`):
   - `runs.manifest.sha256` (`083d2444…`, 434 lines), all `65/65` `Relation` (`0..4096` step 64)
     and `258/258` `FrozenZero` (`0..16384` step 64 + `16404`) checkpoints, both `steps.jsonl`
     logs (`c0294d36…` / `645c3fa9…`), `q1.json` (`1362a783…` / `844eda30…`), and `report.json`
     (`b8f58bd6…` / `86367dac…`)
   - Kernel cell mapping (`discrete.cu:282-289`): `0` off-orbit `block_0..3` roots, `256/256`
     occupied cells at every checkpoint (`cp0` `16` tokens/cell matching GPU `token_cells.u16`
     `f249902d…` `4,096/4,096`), and `0` transient cell excursions across all `65 + 258` checkpoints
   - `0`-mismatch per-interval (`64`-batch) direct `VectorBlock` slot bijection (`discrete.cu:449`)
     across all `64 + 257 = 321` intervals (`token = index // 4, block = index % 4`)
   - Exact step-`175` (`Gate(21)`) bifurcation after identical steps `0..174` (`28` vector writes,
     `14` on `block in {0, 3}` over `10` tokens / `6` `P_succ` targets / `302` rows; `7` combine writes)
   - Two-sided (`init_b3` vs `q1_b3`/`final_b3`) and matched 4-way cross-arm Q1 cohort (`unchanged_both`,
     `moved_both`, `moved_rel_only`, `moved_fz_only`) `P_succ` admission decompositions.
5. Admission-Preserving Acceptance-Rule Pre-Registration Sheet, Unvisited-Tail TRAIN `P_succ`
   Anchor & Active Interval-Rejection Consumer Verification (`verify_admission_preserving_sheet_and_anchor`,
   `verify_train_psucc_tail_anchor_csv`, `verify_admission_preserving_guard_bundle`):
   - `docs/harness/prep/m1-attention-admission-preserving-DRAFT-2026-10-19.md` (`e9d4a4bf…` at draft,
     `17,156` B, `126` lines; top/bottom `TEST FITNESS` consistency, all 7 D22 headings, parent sheet
     `3e622beb…`, and anchor CSV `8c1b80e9…`)
   - Host-derived unvisited-tail TRAIN `P_succ` anchor (`3,847` genuine `is_copy_successor` rows in
     `C = 192` causal in-document window starting at eligible index `1,049,856`, actual indices
     `1,049,857 ..= 1,064,112`, canonical CSV SHA-256 `8c1b80e9b15e4e2bcc765037afa7800b785418be3e20b8cc80ce13ee2e32b3c1`)
   - §7 Registration checklist, D40/D31 preconditions, 15% readiness predicate (`R = added_gate_cost /
     ungated_two_arm_dose_cost <= 0.15`, `258` evaluations/arm = `516` total), synchronous-barrier interval
     audits (`0`, `64..16384`, and mandatory 20-batch tail `16404`), `margin == 24` rollback + schedule
     cursor advance, and §2.4 `RULE_EFFECTIVELY_NOOP` (`> 0.50` rejection rate or `3` consecutive
     full-interval rejections)
6. Integrated `#2782` Audited Source Contract, `TrainAnchorManifest`, Finite Qualification &
   Audited Dose Consumer Verification (`verify_integrated_audited_source_contract`,
   `verify_train_anchor_manifest`, `verify_audited_qualification_artifacts`,
   `verify_integrated_audited_dose_artifacts`; `64/64` self-tests).

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

HISTORICAL_DRIVER_SOURCE_FILES: tuple[str, ...] = (
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
    "attention_anchor.rs",
    "attention_audit.rs",
    "attention_anchor_device.rs",
    "attention_audited.rs",
    "attention_audit_qualification.rs",
    "driver_qualification.rs",
    "relation_qualification.rs",
    "snapshot.rs",
    "main.rs",
)

COMPILED_SOURCE_FILES: tuple[str, ...] = (
    "lib.rs",
    "device.rs",
    "discrete.cu",
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
EXPECTED_INTEGRATED_DRIVER_SOURCE_SHA256 = (
    "0f8ce4061996acdcce3ad4ae01d9e267f280703a77536eb6ecc24bfdc6e6df1d"
)
EXPECTED_COMPILED_SOURCE_SHA256 = (
    "a5d59806d7aa12ce7bed05bacc8cffa700da717a172efbe787075cce78c60831"
)
EXPECTED_ATTENTION_LEARNING_RS_SHA256 = (
    "21e8b5282b3ef2956d359c9518e9f0bf68c0d2d75e4febcc59514c1cacd8709f"
)
EXPECTED_INTEGRATED_ATTENTION_LEARNING_RS_SHA256 = (
    "2dda0b543cbb957f275c1f1e952be18e09f9d3c094e6aac72db7bdd70b6c7c89"
)
EXPECTED_INTEGRATED_MODULE_PINS: dict[str, tuple[str, int, int]] = {
    "attention_anchor.rs": (
        "603a0f953d6aefa8216a7e6d25d4f080eecf8411a91db1d0b665fbdb74e11585",
        24921,
        658,
    ),
    "attention_audit.rs": (
        "aff42682b220fef11698566fb1055bd43773e9db2f2653c86806de9a088f4350",
        37194,
        1064,
    ),
    "attention_anchor_device.rs": (
        "9e89c266c6815b045e15a792812bfc33d0014133ef908f740d9f62b41aba08c8",
        28030,
        695,
    ),
    "attention_audited.rs": (
        "cd4c6e3d29708d0f93a348e1e352440009f8630cffb9c285d84a53631f22c909",
        70828,
        1718,
    ),
    "attention_audit_qualification.rs": (
        "dc9919134cdfecc614d8327c75fa91f4526c40f3c12644c46c76e39487212163",
        138269,
        3234,
    ),
    "snapshot.rs": (
        "c03dcb7bc0e0e966951285a52cf76bf672ee74bf136c42403b40d1a4b5da1170",
        5922,
        135,
    ),
}
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
EXPECTED_INTEGRATED_A8_RELATION_LINEAGE_SHA256 = (
    "f22ebd7129f194c7b3089a3b647b3d174d8c0c95fb18395611bba77036fa3cf3"
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
EXPECTED_INTEGRATED_A8_FROZEN_ZERO_LINEAGE_SHA256 = (
    "e4c3efc99f51bc3eb4c13844cc3e7dc12e9e42d32a2cde39603e6052d649e79a"
)
EXPECTED_TRAIN_ANCHOR_MANIFEST_SHA256 = (
    "f85466ff339d8fb848fb1151a7d672fba5687296b81b1ad6fb39c36ddaca27de"
)
EXPECTED_TRAIN_ANCHOR_ROW0_WINDOW_SHA256 = (
    "eaddc64951b2a1520df965ba1ad089d8572a2146c2cc6813b25b81a5662abb81"
)
EXPECTED_TRAIN_ANCHOR_ROW_LAST_WINDOW_SHA256 = (
    "6a53f3af61f225e226e1c3e25cbec97cd622ea7fafc364fba4886891efae5175"
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

# Verified vast-56670997 census, steps.jsonl, q1.json, and report.json digests (#2029)
EXPECTED_VAST_56670997_RUNS_MANIFEST_SHA256 = (
    "083d2444a1aecda46985b0ef086856bcae9dcd0cb60e41a133ecf2046b6f1767"
)
EXPECTED_VAST_56670997_RUNS_MANIFEST_BYTES = 53520
EXPECTED_VAST_56670997_RUNS_MANIFEST_LINES = 434
EXPECTED_VAST_56670997_CP0_TOKEN_CELLS_U16_SHA256 = (
    "f249902d932715a68ad3173dc933b3a4f75d52da95325aa5909a1fadcbf076cd"
)
EXPECTED_VAST_56670997_REL_CP00000_SHA256 = (
    "b2afa7fe98ac301cc9054e9297675efdca3a2b3a44ec9ffdee9c9b05c98716e9"
)
EXPECTED_VAST_56670997_REL_CP00000_BYTES = 6967383
EXPECTED_VAST_56670997_REL_CP04096_SHA256 = (
    "2e980f2d688877bb4d8f4c3587bcf15f2499b551ad4b613dd149ac4a77ab7e7e"
)
EXPECTED_VAST_56670997_REL_CP04096_BYTES = 6967576
EXPECTED_VAST_56670997_FZ_CP00000_SHA256 = (
    "0e5c9fb9bea891833e163971f9a90968ac5bba8f3c1515818cebdd8703ab060d"
)
EXPECTED_VAST_56670997_FZ_CP00000_BYTES = 6967471
EXPECTED_VAST_56670997_FZ_CP04096_SHA256 = (
    "59251bd70ff6b650779f46fe91f50abd8cf6f8d5def95914ed4cd38c90b72689"
)
EXPECTED_VAST_56670997_FZ_CP04096_BYTES = 6967653
EXPECTED_VAST_56670997_FZ_CP16404_SHA256 = (
    "14e693a08d9e717702f2181d7e6c6e86e28e815fefb0985223187dc6015bafce"
)
EXPECTED_VAST_56670997_FZ_CP16404_BYTES = 6967666
EXPECTED_VAST_56670997_REL_STEPS_JSONL_SHA256 = (
    "c0294d3639f5c2265874c4f63f91baeadd07bb26793edadd3eb8b3d419015a2c"
)
EXPECTED_VAST_56670997_REL_STEPS_JSONL_BYTES = 4008402
EXPECTED_VAST_56670997_REL_STEPS_JSONL_LINES = 4096
EXPECTED_VAST_56670997_FZ_STEPS_JSONL_SHA256 = (
    "645c3fa9592147969ff674cb4f2d74f1b421adedc8b8022739700275d937acc2"
)
EXPECTED_VAST_56670997_FZ_STEPS_JSONL_BYTES = 16992852
EXPECTED_VAST_56670997_FZ_STEPS_JSONL_LINES = 16404
EXPECTED_VAST_56670997_REL_Q1_JSON_SHA256 = (
    "1362a783b568e6d8a00775e834aff9e4d1b03bf649a76b7e6c0148925a88b404"
)
EXPECTED_VAST_56670997_REL_Q1_JSON_BYTES = 3262919
EXPECTED_VAST_56670997_FZ_Q1_JSON_SHA256 = (
    "844eda30a0e8a1f2b993999521c5bbc5e39c566ea4fcc7b864a3803aa87df4ac"
)
EXPECTED_VAST_56670997_FZ_Q1_JSON_BYTES = 3261947
EXPECTED_VAST_56670997_FZ_REPORT_JSON_SHA256 = (
    "b8f58bd6d72321ad82e4441e184d602f96a386d45fb411d9ef62a39e79289a89"
)
EXPECTED_VAST_56670997_FZ_REPORT_JSON_BYTES = 8176537
EXPECTED_VAST_56670997_REL_REPORT_JSON_SHA256 = (
    "86367dac9c283f4469293f4dcc4d894a29dbf5247738b5b34a280e4b920617c9"
)
EXPECTED_VAST_56670997_REL_REPORT_JSON_BYTES = 746

DEFAULT_VAST_56670997_ARCHIVE_DIR = Path(
    "/Users/casey.allard/uor-r4-local/a8-dose-archive-56670997/vast-archive/vast-56670997/uor-r4"
)
DEFAULT_VAST_56670997_RELATION_CHECKPOINTS_DIR = Path(
    "/Users/casey.allard/uor-r4-local/a8-census-relation/vast-archive/vast-56670997/uor-r4/runs/claude/a8-dose-relation-seed1"
)
DEFAULT_VAST_56670997_FROZEN_CHECKPOINTS_DIR = Path(
    "/Users/casey.allard/uor-r4-local/a8-census-frozen/vast-archive/vast-56670997/uor-r4/runs/claude/a8-dose-frozen-zero-seed1"
)
DEFAULT_VAST_56670997_STEPS_DIR = Path(
    "/Users/casey.allard/uor-r4-local/a8-steps-sample/vast-archive/vast-56670997/uor-r4"
)

ROOT_TUPLE_TO_ID: dict[tuple[int, ...], int] = {
    tuple(r): idx for idx, r in enumerate(ROOTS)
}
ROOT_BYTES_TO_ID: dict[bytes, int] = {
    struct.pack("<8i", *r): idx for idx, r in enumerate(ROOTS)
}

EXPECTED_ADMISSION_PRESERVING_SHEET_REL_PATH = (
    "docs/harness/prep/m1-attention-admission-preserving-DRAFT-2026-10-19.md"
)
EXPECTED_ADMISSION_PRESERVING_DRAFT_SHEET_SHA256 = (
    "e9d4a4bf1951986f683cf3914241de375a864d0e756d8b70ea9024fe50afa452"
)
EXPECTED_ADMISSION_PRESERVING_DRAFT_SHEET_BYTES = 17156
EXPECTED_ADMISSION_PRESERVING_DRAFT_SHEET_LINES = 126
EXPECTED_ADMISSION_PRESERVING_FIT_SHEET_SHA256 = (
    "fcb770e66d38f607d25e16dade14ac7a6223dd8af0ccc9ec22c80db71f2c3450"
)
EXPECTED_ADMISSION_PRESERVING_FIT_SHEET_BYTES = 17745
EXPECTED_ADMISSION_PRESERVING_FIT_SHEET_LINES = 126
EXPECTED_ADMISSION_PRESERVING_D22_HEADINGS: tuple[str, ...] = (
    "## Base artifact",
    "## Data split",
    "## 3. Bar",
    "## 4. Cost",
    "## Stop rule",
    "## Minimum test power (D22 section 3)",
    "## 5. Steps",
)
EXPECTED_TRAIN_PSUCC_TAIL_ANCHOR_CSV_SHA256 = (
    "8c1b80e9b15e4e2bcc765037afa7800b785418be3e20b8cc80ce13ee2e32b3c1"
)
EXPECTED_TRAIN_PSUCC_TAIL_ANCHOR_ROWS = 3847
EXPECTED_TRAIN_PSUCC_TAIL_START_ELIGIBLE_INDEX = 1049856
EXPECTED_TRAIN_PSUCC_TAIL_FIRST_ROW: tuple[int, int, int, int] = (1049857, 3251166, 714, 1)
EXPECTED_TRAIN_PSUCC_TAIL_LAST_ROW: tuple[int, int, int, int] = (1064112, 3295596, 310, 1)
EXPECTED_ADMISSION_PRESERVING_AUDIT_CADENCE_BATCHES = 64
EXPECTED_ADMISSION_PRESERVING_TAIL_BATCHES = 20
EXPECTED_ADMISSION_PRESERVING_EVALUATIONS_PER_ARM = 258
EXPECTED_ADMISSION_PRESERVING_TOTAL_EVALUATIONS_TWO_ARMS = 516
EXPECTED_ADMISSION_PRESERVING_MARGIN_ROWS = 24
EXPECTED_ADMISSION_PRESERVING_ANCHOR_MIN_BASELINE = 500
EXPECTED_ADMISSION_PRESERVING_MAX_REJECTION_RATE = 0.50
EXPECTED_ADMISSION_PRESERVING_MAX_CONSECUTIVE_FULL_REJECTIONS = 3
EXPECTED_ADMISSION_PRESERVING_MAX_GATE_COST_RATIO = 0.15
DEFAULT_LOCAL_MW_BALP_TRAIN_TOKENS_PATH = Path(
    "/Users/casey.allard/uor-r4-local/codex-attention-preflight-inputs-v1/mw-balp/train/tokens.u16"
)
DEFAULT_LOCAL_MW_BALP_TRAIN_MASK_PATH = Path(
    "/Users/casey.allard/uor-r4-local/codex-attention-preflight-inputs-v1/mw-balp/train/response_mask.u8"
)

EXPECTED_VAST_56670997_CENSUS_EXPECTATIONS: dict[str, Any] = {
    "manifest_sha256": EXPECTED_VAST_56670997_RUNS_MANIFEST_SHA256,
    "manifest_bytes": EXPECTED_VAST_56670997_RUNS_MANIFEST_BYTES,
    "manifest_lines": EXPECTED_VAST_56670997_RUNS_MANIFEST_LINES,
    "relation_checkpoint_count": 65,
    "frozen_zero_checkpoint_count": 258,
    "relation_intervals": 64,
    "frozen_zero_intervals": 257,
    "total_intervals": 321,
    "cp0_token_cells_u16_sha256": EXPECTED_VAST_56670997_CP0_TOKEN_CELLS_U16_SHA256,
    "relation_cp0_sha256": EXPECTED_VAST_56670997_REL_CP00000_SHA256,
    "relation_q1_cp_sha256": EXPECTED_VAST_56670997_REL_CP04096_SHA256,
    "frozen_zero_cp0_sha256": EXPECTED_VAST_56670997_FZ_CP00000_SHA256,
    "frozen_zero_q1_cp_sha256": EXPECTED_VAST_56670997_FZ_CP04096_SHA256,
    "frozen_zero_final_cp_sha256": EXPECTED_VAST_56670997_FZ_CP16404_SHA256,
    "relation_steps_sha256": EXPECTED_VAST_56670997_REL_STEPS_JSONL_SHA256,
    "relation_steps_bytes": EXPECTED_VAST_56670997_REL_STEPS_JSONL_BYTES,
    "relation_steps_lines": EXPECTED_VAST_56670997_REL_STEPS_JSONL_LINES,
    "frozen_zero_steps_sha256": EXPECTED_VAST_56670997_FZ_STEPS_JSONL_SHA256,
    "frozen_zero_steps_bytes": EXPECTED_VAST_56670997_FZ_STEPS_JSONL_BYTES,
    "frozen_zero_steps_lines": EXPECTED_VAST_56670997_FZ_STEPS_JSONL_LINES,
    "relation_q1_json_sha256": EXPECTED_VAST_56670997_REL_Q1_JSON_SHA256,
    "frozen_zero_q1_json_sha256": EXPECTED_VAST_56670997_FZ_Q1_JSON_SHA256,
    "frozen_zero_report_json_sha256": EXPECTED_VAST_56670997_FZ_REPORT_JSON_SHA256,
    "relation_report_json_sha256": EXPECTED_VAST_56670997_REL_REPORT_JSON_SHA256,
    "population_bounds": {
        "cp0": {"occupied_cells": 256, "min_pop": 16, "max_pop": 16},
        "relation_q1": {"occupied_cells": 256, "min_pop": 9, "max_pop": 20},
        "frozen_zero_q1": {"occupied_cells": 256, "min_pop": 8, "max_pop": 20},
        "frozen_zero_final": {"occupied_cells": 256, "min_pop": 4, "max_pop": 20},
    },
    "vector_writes": {
        "relation_q1": {
            "total": 192,
            "by_block": [58, 49, 52, 33],
            "block_0_or_3_writes": 91,
            "moved_tokens": 80,
            "moved_psucc_targets": 63,
            "moved_psucc_rows": 1461,
        },
        "frozen_zero_q1": {
            "total": 207,
            "by_block": [64, 49, 55, 39],
            "block_0_or_3_writes": 103,
            "moved_tokens": 91,
            "moved_psucc_targets": 71,
            "moved_psucc_rows": 1369,
        },
        "frozen_zero_final": {
            "total": 301,
            "by_block": [82, 75, 80, 64],
            "block_0_or_3_writes": 146,
            "moved_tokens": 130,
            "moved_psucc_targets": 89,
            "moved_psucc_rows": 1527,
        },
        "q1_moved_overlap": {
            "shared": {"tokens": 65, "psucc_targets": 52, "psucc_rows": 1150},
            "relation_only": {"tokens": 15, "psucc_targets": 11, "psucc_rows": 311},
            "frozen_zero_only": {"tokens": 26, "psucc_targets": 19, "psucc_rows": 219},
        },
    },
    "bifurcation": {
        "identical_prefix_steps": 175,
        "first_divergence_step": 175,
        "prefix_accepted_vector_writes": 28,
        "prefix_accepted_block_0_or_3_writes": 14,
        "prefix_moved_tokens": 10,
        "prefix_moved_psucc_targets": 6,
        "prefix_moved_psucc_rows": 302,
        "prefix_accepted_gate_combine_writes": 7,
        "prefix_accepted_gate_relation_writes": 0,
        "divergence_coordinate": {"kind": "gate", "index": 21},
        "divergence_candidate": 2,
        "divergence_loss_before": 55680,
        "divergence_loss_after": 55232,
        "relation_applied": True,
        "frozen_zero_applied": False,
        "frozen_zero_refused_by_policy": True,
    },
    "two_sided_decomposition": {
        "relation_q1": {
            "target_cell_unchanged": {
                "targets": 385,
                "rows": 2386,
                "init_admitted": 934,
                "end_admitted": 412,
                "kept": 295,
                "lost": 639,
                "gained": 117,
                "never": 1335,
                "net_change": -522,
            },
            "target_cell_moved": {
                "targets": 63,
                "rows": 1461,
                "init_admitted": 417,
                "end_admitted": 139,
                "kept": 44,
                "lost": 373,
                "gained": 95,
                "never": 949,
                "net_change": -278,
            },
            "total": {
                "targets": 448,
                "rows": 3847,
                "init_admitted": 1351,
                "end_admitted": 551,
                "kept": 339,
                "lost": 1012,
                "gained": 212,
                "never": 2284,
                "net_change": -800,
            },
        },
        "frozen_zero_q1": {
            "target_cell_unchanged": {
                "targets": 377,
                "rows": 2478,
                "init_admitted": 912,
                "end_admitted": 577,
                "kept": 437,
                "lost": 475,
                "gained": 140,
                "never": 1426,
                "net_change": -335,
            },
            "target_cell_moved": {
                "targets": 71,
                "rows": 1369,
                "init_admitted": 439,
                "end_admitted": 280,
                "kept": 102,
                "lost": 337,
                "gained": 178,
                "never": 752,
                "net_change": -159,
            },
            "total": {
                "targets": 448,
                "rows": 3847,
                "init_admitted": 1351,
                "end_admitted": 857,
                "kept": 539,
                "lost": 812,
                "gained": 318,
                "never": 2178,
                "net_change": -494,
            },
        },
        "frozen_zero_final": {
            "target_cell_unchanged": {
                "targets": 359,
                "rows": 2320,
                "init_admitted": 863,
                "end_admitted": 540,
                "kept": 385,
                "lost": 478,
                "gained": 155,
                "never": 1302,
                "net_change": -323,
            },
            "target_cell_moved": {
                "targets": 89,
                "rows": 1527,
                "init_admitted": 488,
                "end_admitted": 257,
                "kept": 89,
                "lost": 399,
                "gained": 168,
                "never": 871,
                "net_change": -231,
            },
            "total": {
                "targets": 448,
                "rows": 3847,
                "init_admitted": 1351,
                "end_admitted": 797,
                "kept": 474,
                "lost": 877,
                "gained": 323,
                "never": 2173,
                "net_change": -554,
            },
        },
    },
    "four_way_q1_cohorts": {
        "unchanged_both": {
            "targets": 366,
            "rows": 2167,
            "init_admitted": 833,
            "rel_q1_admitted": 366,
            "rel_net": -467,
            "rel_kept": 260,
            "rel_lost": 573,
            "rel_gained": 106,
            "rel_never": 1228,
            "fz_q1_admitted": 537,
            "fz_net": -296,
            "fz_kept": 404,
            "fz_lost": 429,
            "fz_gained": 133,
            "fz_never": 1201,
            "paired_rel_minus_fz": -171,
            "both_admitted": 189,
            "rel_only_admitted": 177,
            "fz_only_admitted": 348,
            "neither_admitted": 1453,
        },
        "moved_both": {
            "targets": 52,
            "rows": 1150,
            "init_admitted": 338,
            "rel_q1_admitted": 116,
            "rel_net": -222,
            "rel_kept": 34,
            "rel_lost": 304,
            "rel_gained": 82,
            "rel_never": 730,
            "fz_q1_admitted": 222,
            "fz_net": -116,
            "fz_kept": 62,
            "fz_lost": 276,
            "fz_gained": 160,
            "fz_never": 652,
            "paired_rel_minus_fz": -106,
            "both_admitted": 42,
            "rel_only_admitted": 74,
            "fz_only_admitted": 180,
            "neither_admitted": 854,
        },
        "moved_rel_only": {
            "targets": 11,
            "rows": 311,
            "init_admitted": 79,
            "rel_q1_admitted": 23,
            "rel_net": -56,
            "rel_kept": 10,
            "rel_lost": 69,
            "rel_gained": 13,
            "rel_never": 219,
            "fz_q1_admitted": 40,
            "fz_net": -39,
            "fz_kept": 33,
            "fz_lost": 46,
            "fz_gained": 7,
            "fz_never": 225,
            "paired_rel_minus_fz": -17,
            "both_admitted": 7,
            "rel_only_admitted": 16,
            "fz_only_admitted": 33,
            "neither_admitted": 255,
        },
        "moved_fz_only": {
            "targets": 19,
            "rows": 219,
            "init_admitted": 101,
            "rel_q1_admitted": 46,
            "rel_net": -55,
            "rel_kept": 35,
            "rel_lost": 66,
            "rel_gained": 11,
            "rel_never": 107,
            "fz_q1_admitted": 58,
            "fz_net": -43,
            "fz_kept": 40,
            "fz_lost": 61,
            "fz_gained": 18,
            "fz_never": 100,
            "paired_rel_minus_fz": -12,
            "both_admitted": 16,
            "rel_only_admitted": 30,
            "fz_only_admitted": 42,
            "neither_admitted": 131,
        },
    },
}

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

    # 2. Verify float-free constructor, 19-file driver_source_sha256, and 3-file compiled_source_sha256
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
        driver_sha == EXPECTED_INTEGRATED_DRIVER_SOURCE_SHA256,
        f"driver_source_sha256 {driver_sha} != {EXPECTED_INTEGRATED_DRIVER_SOURCE_SHA256}",
    )

    compiled_hasher = hashlib.sha256()
    for fname in COMPILED_SOURCE_FILES:
        compiled_hasher.update(read_bytes_checked(src_dir / fname, fname))
    compiled_sha = compiled_hasher.hexdigest()
    require(
        compiled_sha == EXPECTED_COMPILED_SOURCE_SHA256,
        f"compiled_source_sha256 {compiled_sha} != {EXPECTED_COMPILED_SOURCE_SHA256}",
    )

    attn_rs_bytes = read_bytes_checked(src_dir / "attention_learning.rs", "attention_learning.rs")
    attn_rs_sha = hashlib.sha256(attn_rs_bytes).hexdigest()
    require(
        attn_rs_sha == EXPECTED_INTEGRATED_ATTENTION_LEARNING_RS_SHA256,
        f"attention_learning.rs SHA {attn_rs_sha} != {EXPECTED_INTEGRATED_ATTENTION_LEARNING_RS_SHA256}",
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

    rel_cfg_sha, rel_lin_sha = compute_config_and_lineage_sha(
        a8_rel, EXPECTED_DRIVER_SOURCE_SHA256
    )
    fz_cfg_sha, fz_lin_sha = compute_config_and_lineage_sha(
        a8_fz, EXPECTED_DRIVER_SOURCE_SHA256
    )
    require(
        rel_cfg_sha == EXPECTED_A8_RELATION_CONFIG_SHA256
        and rel_lin_sha == EXPECTED_A8_RELATION_LINEAGE_SHA256,
        f"a8-relation historical config/lineage SHA mismatch: {rel_cfg_sha} / {rel_lin_sha}",
    )
    require(
        fz_cfg_sha == EXPECTED_A8_FROZEN_ZERO_CONFIG_SHA256
        and fz_lin_sha == EXPECTED_A8_FROZEN_ZERO_LINEAGE_SHA256,
        f"a8-frozen-zero historical config/lineage SHA mismatch: {fz_cfg_sha} / {fz_lin_sha}",
    )

    _, rel_int_lin_sha = compute_config_and_lineage_sha(a8_rel, driver_sha)
    _, fz_int_lin_sha = compute_config_and_lineage_sha(a8_fz, driver_sha)
    require(
        rel_int_lin_sha == EXPECTED_INTEGRATED_A8_RELATION_LINEAGE_SHA256
        and fz_int_lin_sha == EXPECTED_INTEGRATED_A8_FROZEN_ZERO_LINEAGE_SHA256,
        f"integrated A8 lineage SHA mismatch: {rel_int_lin_sha} / {fz_int_lin_sha}",
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

    admission_preserving_summary = None
    ap_sheet_path = repo_root / EXPECTED_ADMISSION_PRESERVING_SHEET_REL_PATH
    if ap_sheet_path.is_file():
        admission_preserving_summary = verify_admission_preserving_sheet_and_anchor(repo_root)

    integrated_source_summary = verify_integrated_audited_source_contract(repo_root)

    return {
        "status": "PASS",
        "sheet_sha256": sheet_sha,
        "driver_source_sha256": driver_sha,
        "historical_driver_source_sha256": EXPECTED_DRIVER_SOURCE_SHA256,
        "compiled_source_sha256": compiled_sha,
        "attention_learning_rs_sha256": attn_rs_sha,
        "relation": {
            "config_file_sha256": EXPECTED_A8_RELATION_CONFIG_FILE_SHA256,
            "config_sha256": rel_cfg_sha,
            "lineage_sha256": rel_lin_sha,
            "integrated_lineage_sha256": rel_int_lin_sha,
            "scope": rel_scope,
            "d40_phrase": rel_phrase,
        },
        "frozen_zero": {
            "config_file_sha256": EXPECTED_A8_FROZEN_ZERO_CONFIG_FILE_SHA256,
            "config_sha256": fz_cfg_sha,
            "lineage_sha256": fz_lin_sha,
            "integrated_lineage_sha256": fz_int_lin_sha,
            "scope": fz_scope,
            "d40_phrase": fz_phrase,
        },
        "neg_b_a8": negb_summary,
        "runbook_readme_sha256": rb_readme_sha,
        "executor_packet": packet_summary,
        "committed_a8_dose_receipt": committed_dose_summary,
        "admission_preserving_sheet": admission_preserving_summary,
        "integrated_audited_source": integrated_source_summary,
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
    """Run 56 positive and negative self-tests covering all A8 pre-run, Q1, D34, D40, decision, census/attribution, and admission-preserving active interval rejection rules."""
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

    # T41: Positive census, slot bijection, step-175 bifurcation, 0 transient excursions, and two-sided + 4-way P_succ decomposition
    base_census = build_synthetic_census_bundle(repo_root)
    res_census = verify_a8_census_and_attribution(base_census)
    record(
        "41_census_bijection_bifurcation_and_psucc_decomposition_pass",
        res_census["status"] == "PASS"
        and res_census["slot_bijection"]["total_mismatches"] == 0
        and res_census["occupancy"]["relation_transient_excursions"] == 0
        and res_census["occupancy"]["frozen_zero_transient_excursions"] == 0
        and res_census["bifurcation"]["first_divergence_step"] == 175
        and res_census["two_sided_decomposition"]["relation_q1"]["total"]["net_change"] == -800
        and res_census["four_way_q1_cohorts"]["unchanged_both"]["paired_rel_minus_fz"] == -171,
        f"intervals={res_census['slot_bijection']['total_intervals']}, rel_net={res_census['two_sided_decomposition']['relation_q1']['total']['net_change']}",
    )

    # T42: Negative — Checkpoint or steps.jsonl SHA-256 mismatch against runs.manifest.sha256 rejected
    c_bad_sha = copy.deepcopy(base_census)
    c_bad_sha["relation_checkpoints"][-1]["sha256"] = "0" * 64
    try:
        verify_a8_census_and_attribution(c_bad_sha)
        record("42_census_checkpoint_manifest_sha256_mismatch_rejected", False, "failed to reject SHA mismatch")
    except VerificationError as exc:
        record(
            "42_census_checkpoint_manifest_sha256_mismatch_rejected",
            "SHA-256" in str(exc),
            str(exc),
        )

    # T43: Negative — Off-orbit block_0 / block_3 root in checkpoint table rejected
    c_off_orbit = copy.deepcopy(base_census)
    c_off_orbit["relation_checkpoints"][1]["table"][:8] = [0, 2, 2, 0, 0, 0, 0, 0]
    try:
        verify_a8_census_and_attribution(c_off_orbit)
        record("43_census_off_orbit_root_rejected", False, "failed to reject off-orbit root")
    except VerificationError as exc:
        record("43_census_off_orbit_root_rejected", "off-orbit root" in str(exc), str(exc))

    # T44: Negative — Cell occupancy < 256 at any checkpoint rejected
    c_occ_collapse = copy.deepcopy(base_census)
    # Collapse all 4,096 tokens at checkpoint 1 into cell 0 (ROOTS[0] on block_0 and block_3)
    collapsed_tbl = list(c_occ_collapse["relation_checkpoints"][1]["table"])
    for t in range(4096):
        collapsed_tbl[t * 32 : t * 32 + 8] = list(ROOTS[0])
        collapsed_tbl[t * 32 + 24 : t * 32 + 32] = list(ROOTS[0])
    c_occ_collapse["relation_checkpoints"][1]["table"] = collapsed_tbl
    try:
        verify_a8_census_and_attribution(c_occ_collapse)
        record("44_census_cell_occupancy_below_256_rejected", False, "failed to reject occupancy < 256")
    except VerificationError as exc:
        record(
            "44_census_cell_occupancy_below_256_rejected",
            "cell occupancy collapse" in str(exc),
            str(exc),
        )

    # T45: Negative — steps.jsonl vector_block slot mismatch against checkpoint table diff rejected
    c_slot_mismatch = copy.deepcopy(base_census)
    # Mutate an extra unlogged vector_block slot (slot 3999, block 3 of token 999) in checkpoint 1
    bad_slot_tbl = list(c_slot_mismatch["relation_checkpoints"][1]["table"])
    cur_root_id = ROOT_TUPLE_TO_ID[tuple(bad_slot_tbl[3999 * 8 : 4000 * 8])]
    bad_slot_tbl[3999 * 8 : 4000 * 8] = list(ROOTS[(cur_root_id + 1) % 16])
    c_slot_mismatch["relation_checkpoints"][1]["table"] = bad_slot_tbl
    try:
        verify_a8_census_and_attribution(c_slot_mismatch)
        record("45_census_vector_block_slot_bijection_mismatch_rejected", False, "failed to reject slot bijection mismatch")
    except VerificationError as exc:
        record(
            "45_census_vector_block_slot_bijection_mismatch_rejected",
            "vector_block slot bijection mismatch" in str(exc),
            str(exc),
        )

    # T46: Negative — Transient cell excursion (token leaves cp0 cell at cp1 and returns to cp0 cell at Q1) rejected
    c_excursion = copy.deepcopy(base_census)
    # Step 4 in prefix_ops moved block 0 of prefix_b03_tokens[4] (and never touched its block 3).
    # Revert that slot back to its cp0 root in interval 175->400 with a matching applied step at batch 399
    # so slot bijection and interval cell-diff checks pass, isolating the transient cell excursion detector.
    exc_slot = c_excursion["relation_steps"][4]["coordinate"]["index"]
    cp0_slot_words = list(c_excursion["relation_checkpoints"][0]["table"][exc_slot * 8 : (exc_slot + 1) * 8])
    c_excursion["relation_checkpoints"][-1]["table"][exc_slot * 8 : (exc_slot + 1) * 8] = cp0_slot_words
    last_st = c_excursion["relation_steps"][399]
    acc_b = last_st["cursor_before"]["accepted_updates"]
    c_excursion["relation_steps"][399] = {
        "cursor_before": dict(last_st["cursor_before"]),
        "cursor_after": {
            "completed_batches": 400,
            "accepted_updates": acc_b + 1,
            "corpus_pass": 0,
            "next_target": 400 * 64,
            "next_coordinate": 400,
        },
        "coordinate": {"kind": "vector_block", "index": exc_slot},
        "target_positions": [19, 20],
        "candidate": 15,
        "loss_before": 60000,
        "loss_after": 59500,
        "model_epoch": acc_b + 1,
        "applied": True,
        "any_candidate_cost_differs_from_identity": True,
    }
    c_excursion["relation_checkpoints"][-1]["accepted_updates"] = acc_b + 1
    c_excursion["relation_checkpoints"][-1]["epoch"] = acc_b + 1
    try:
        verify_a8_census_and_attribution(c_excursion)
        record("46_census_transient_cell_excursion_rejected", False, "failed to reject transient cell excursion")
    except VerificationError as exc:
        record(
            "46_census_transient_cell_excursion_rejected",
            "transient cell excursion detected" in str(exc),
            str(exc),
        )

    # T47: Negative — Pre-bifurcation (0..174) mismatch or step-175 Gate(21) mismatch rejected
    c_bad_bif = copy.deepcopy(base_census)
    c_bad_bif["frozen_zero_steps"][174]["loss_after"] = 999999
    try:
        verify_a8_census_and_attribution(c_bad_bif)
        record("47_census_step175_bifurcation_or_prefix_mismatch_rejected", False, "failed to reject step 174 mismatch")
    except VerificationError as exc:
        c_bad_175 = copy.deepcopy(base_census)
        c_bad_175["relation_steps"][175]["coordinate"] = {"kind": "gate", "index": 22}
        try:
            verify_a8_census_and_attribution(c_bad_175)
            record("47_census_step175_bifurcation_or_prefix_mismatch_rejected", False, "failed to reject step 175 coordinate mismatch")
        except VerificationError as exc2:
            record(
                "47_census_step175_bifurcation_or_prefix_mismatch_rejected",
                "pre-bifurcation step 174" in str(exc) and "Gate(21) bifurcation mismatch" in str(exc2),
                f"prefix={exc}; step175={exc2}",
            )

    # T48: Negative — Two-sided or 4-way cross-arm P_succ decomposition arithmetic / row-key mismatch rejected
    c_bad_decomp = copy.deepcopy(base_census)
    # Flip one P_succ row's target_admitted in Relation Q1 without updating expectations
    for row in c_bad_decomp["relation_q1"]["q1_b3"]["rows"]:
        if row["is_psucc"]:
            row["target_admitted"] = not row["target_admitted"]
            break
    try:
        verify_a8_census_and_attribution(c_bad_decomp)
        record("48_census_two_sided_and_four_way_psucc_decomposition_mismatch_rejected", False, "failed to reject P_succ decomposition mismatch")
    except VerificationError as exc:
        c_bad_row_key = copy.deepcopy(base_census)
        c_bad_row_key["frozen_zero_q1"]["q1_b3"]["rows"][0]["target_position"] += 1
        try:
            verify_a8_census_and_attribution(c_bad_row_key)
            record("48_census_two_sided_and_four_way_psucc_decomposition_mismatch_rejected", False, "failed to reject row_key mismatch")
        except VerificationError as exc2:
            record(
                "48_census_two_sided_and_four_way_psucc_decomposition_mismatch_rejected",
                "two-sided P_succ decomposition mismatch" in str(exc) and "DEV row key mismatch" in str(exc2),
                f"decomp={exc}; row_key={exc2}",
            )

    # T49: Admission-preserving sheet & TRAIN tail P_succ anchor CSV verification
    ap_sheet_res = verify_admission_preserving_sheet_and_anchor(repo_root)
    syn_anchor_rows = [
        (EXPECTED_TRAIN_PSUCC_TAIL_START_ELIGIBLE_INDEX + 1 + i, 3251166 + i * 10, (i % 4095) + 1, 1)
        for i in range(EXPECTED_TRAIN_PSUCC_TAIL_ANCHOR_ROWS)
    ]
    syn_anchor_res = verify_train_psucc_tail_anchor_csv(
        syn_anchor_rows, require_exact_pinned_sha=False
    )
    record(
        "49_admission_preserving_sheet_and_train_tail_anchor_csv_pass",
        ap_sheet_res["status"] == "PASS"
        and (
            (
                ap_sheet_res["test_fitness"] == "PENDING"
                and ap_sheet_res["sheet_sha256"] == EXPECTED_ADMISSION_PRESERVING_DRAFT_SHEET_SHA256
            )
            or (
                ap_sheet_res["test_fitness"] == "FIT"
                and ap_sheet_res["sheet_sha256"] == EXPECTED_ADMISSION_PRESERVING_FIT_SHEET_SHA256
            )
        )
        and ap_sheet_res["anchor_csv_sha256"] == EXPECTED_TRAIN_PSUCC_TAIL_ANCHOR_CSV_SHA256
        and len(ap_sheet_res["d22_headings_verified"]) == 7
        and syn_anchor_res["status"] == "PASS"
        and (
            ap_sheet_res["anchor_uort_verification"] is None
            or ap_sheet_res["anchor_uort_verification"]["csv_sha256"]
            == EXPECTED_TRAIN_PSUCC_TAIL_ANCHOR_CSV_SHA256
        ),
        f"sheet={ap_sheet_res['sheet_sha256'][:12]} ({ap_sheet_res['test_fitness']}), anchor={ap_sheet_res['anchor_csv_sha256'][:12]}, uort={ap_sheet_res['anchor_uort_verification'] is not None}",
    )

    # T50: Valid 2-arm (258 evaluations/arm including baseline 0 and tail 16,404) active interval rejection bundle PASS
    base_ap = build_synthetic_admission_preserving_bundle()
    res_ap_ok = verify_admission_preserving_guard_bundle(base_ap)
    record(
        "50_admission_preserving_valid_two_arm_active_rejection_pass",
        res_ap_ok["status"] == "PASS"
        and res_ap_ok["verdict"]
        == "ADMISSION_PRESERVING_DIAGNOSTIC_PASS_DEFER_TO_PARENT_DECISION"
        and res_ap_ok["keep"] is False
        and res_ap_ok["mechanism_negative"] is False
        and res_ap_ok["arms"]["relation"]["evaluations_count"] == 258
        and res_ap_ok["arms"]["frozen_zero"]["evaluations_count"] == 258
        and res_ap_ok["arms"]["relation"]["rejected_intervals"] == 2
        and res_ap_ok["arms"]["frozen_zero"]["rejected_intervals"] == 2,
        f"verdict={res_ap_ok['verdict']}, rel_rej={res_ap_ok['arms']['relation']['rejected_intervals']}/257",
    )

    # T51: Qualification gate cost ratio R = added_gate_cost / ungated_two_arm_dose_cost > 0.15 -> STOP_BEFORE_TRAINING_REPLAN_GATE_COST_EXCEEDS_15PCT
    ap_high_cost = copy.deepcopy(base_ap)
    ap_high_cost["training_started"] = False
    ap_high_cost["qualification_cost"]["added_gate_cost"] = 2.28
    ap_high_cost["qualification_cost"]["ungated_two_arm_dose_cost"] = 11.40
    ap_high_cost["qualification_cost"]["cost_ratio_r"] = 0.20
    res_ap_cost = verify_admission_preserving_guard_bundle(ap_high_cost)
    ap_high_cost_started = copy.deepcopy(ap_high_cost)
    ap_high_cost_started["training_started"] = True
    res_ap_cost_started = verify_admission_preserving_guard_bundle(ap_high_cost_started)
    record(
        "51_admission_preserving_gate_cost_ratio_exceeds_15pct_stop",
        res_ap_cost["verdict"] == "STOP_BEFORE_TRAINING_REPLAN_GATE_COST_EXCEEDS_15PCT"
        and res_ap_cost["mechanism_negative"] is False
        and res_ap_cost["keep"] is False
        and res_ap_cost_started["verdict"] == "INSTRUMENT_UNREACHABLE_D31"
        and "gate_cost_exceeds_15pct_but_training_started" in res_ap_cost_started["faults"],
        f"pre_train={res_ap_cost['verdict']} (R={res_ap_cost['cost_ratio_r']:.2f}), started={res_ap_cost_started['verdict']}",
    )

    # T52: Anchor baseline < 500, wrong anchor CSV SHA-256, or training-prefix overlap (eligible_ordinal < 1,049,856) -> INSTRUMENT_UNREACHABLE_D31
    ap_low_base = copy.deepcopy(base_ap)
    ap_low_base["prerun_gates"]["relation"]["anchor_baseline_admitted"] = 499
    ap_low_base["prerun_gates"]["frozen_zero"]["anchor_baseline_admitted"] = 499
    res_ap_low_base = verify_admission_preserving_guard_bundle(ap_low_base)

    ap_bad_anchor_sha = copy.deepcopy(base_ap)
    ap_bad_anchor_sha["registration"]["anchor_csv_sha256"] = "0" * 64
    res_ap_bad_sha = verify_admission_preserving_guard_bundle(ap_bad_anchor_sha)

    ap_prefix_overlap = copy.deepcopy(base_ap)
    ap_prefix_overlap["registration"]["anchor_first_eligible_ordinal"] = (
        EXPECTED_TRAIN_PSUCC_TAIL_START_ELIGIBLE_INDEX - 1
    )
    ap_prefix_overlap["registration"]["training_prefix_overlap_rows"] = 1
    res_ap_overlap = verify_admission_preserving_guard_bundle(ap_prefix_overlap)

    overlap_csv_rejected = False
    bad_csv_rows = list(syn_anchor_rows)
    bad_csv_rows[0] = (EXPECTED_TRAIN_PSUCC_TAIL_START_ELIGIBLE_INDEX - 1, 3251160, 714, 1)
    try:
        verify_train_psucc_tail_anchor_csv(bad_csv_rows, require_exact_pinned_sha=False)
    except VerificationError as exc:
        overlap_csv_rejected = "overlaps training prefix" in str(exc)

    record(
        "52_admission_preserving_anchor_baseline_or_hash_or_prefix_overlap_d31",
        res_ap_low_base["verdict"] == "INSTRUMENT_UNREACHABLE_D31"
        and "relation_anchor_baseline_below_500" in res_ap_low_base["faults"]
        and res_ap_bad_sha["verdict"] == "INSTRUMENT_UNREACHABLE_D31"
        and "anchor_csv_sha256_mismatch" in res_ap_bad_sha["faults"]
        and res_ap_overlap["verdict"] == "INSTRUMENT_UNREACHABLE_D31"
        and "anchor_overlaps_training_prefix" in res_ap_overlap["faults"]
        and overlap_csv_rejected,
        f"low_base={res_ap_low_base['faults']}, bad_sha={res_ap_bad_sha['faults']}, overlap_csv_rejected={overlap_csv_rejected}",
    )

    # T53: Breach without restoring last passing model_storage_sha256/epoch/accepted_updates or without cursor advance or missing receipt -> INSTRUMENT_UNREACHABLE_D31
    ap_bad_restore = copy.deepcopy(base_ap)
    ap_bad_restore["arms"]["relation"]["audits"][10]["restored_model_storage_sha256"] = "9" * 64
    res_ap_bad_restore = verify_admission_preserving_guard_bundle(ap_bad_restore)

    ap_no_cursor = copy.deepcopy(base_ap)
    ap_no_cursor["arms"]["relation"]["audits"][10][
        "schedule_cursor_advanced_past_interval"
    ] = False
    res_ap_no_cursor = verify_admission_preserving_guard_bundle(ap_no_cursor)

    ap_no_receipt = copy.deepcopy(base_ap)
    ap_no_receipt["arms"]["relation"]["audits"][10]["rejection_receipts_count"] = 0
    res_ap_no_receipt = verify_admission_preserving_guard_bundle(ap_no_receipt)

    record(
        "53_admission_preserving_breach_unrestored_state_or_cursor_or_receipt_d31",
        res_ap_bad_restore["verdict"] == "INSTRUMENT_UNREACHABLE_D31"
        and "relation_audit_640_breach_failed_to_restore_last_passing_state"
        in res_ap_bad_restore["faults"]
        and res_ap_no_cursor["verdict"] == "INSTRUMENT_UNREACHABLE_D31"
        and "relation_audit_640_breach_missing_preservation_cursor_or_receipt"
        in res_ap_no_cursor["faults"]
        and res_ap_no_receipt["verdict"] == "INSTRUMENT_UNREACHABLE_D31"
        and "relation_audit_640_breach_missing_preservation_cursor_or_receipt"
        in res_ap_no_receipt["faults"],
        f"restore={res_ap_bad_restore['faults']}, cursor={res_ap_no_cursor['faults']}, receipt={res_ap_no_receipt['faults']}",
    )

    # T54: 3 consecutive full-interval rejections -> RULE_EFFECTIVELY_NOOP (mechanism_negative = False)
    ap_3consec = build_synthetic_admission_preserving_bundle()
    # Rebuild relation arm with 3 consecutive breaches at intervals 10, 11, 12 (stopping at batch 768 = index 12)
    ap_3consec_full = copy.deepcopy(base_ap)
    # Construct relation arm with breaches at {10, 11, 12} by mutating interval 11 and 12 in relation
    # Even cleaner: use build helper logic by setting breaches on intervals 1, 2, 3 and truncating at index 3 (batch 192)
    rel_a0 = copy.deepcopy(ap_3consec["arms"]["relation"]["audits"][0])
    s0_sha = rel_a0["model_storage_sha256"]
    sn0_sha = rel_a0["model_snapshot_sha256"]
    consec_audits = [rel_a0]
    for idx in (1, 2, 3):
        b = idx * 64
        consec_audits.append(
            {
                "audit_batch": b,
                "synchronous_pause": True,
                "concurrent_updates": 0,
                "model_storage_sha256": hashlib.sha256(f"c:{idx}".encode("utf-8")).hexdigest(),
                "model_snapshot_sha256": hashlib.sha256(f"cs:{idx}".encode("utf-8")).hexdigest(),
                "model_epoch": 1,
                "accepted_updates": 1,
                "driver_cursor": {
                    "completed_batches": b,
                    "accepted_updates": 1,
                    "corpus_pass": 0,
                    "next_target": b * 64,
                    "next_coordinate": b % 45,
                },
                "family_counters": {"vector": 1, "route": 0, "combine": 0, "relation": 0},
                "lineage_sha256": EXPECTED_A8_RELATION_LINEAGE_SHA256,
                "anchor_csv_sha256": EXPECTED_TRAIN_PSUCC_TAIL_ANCHOR_CSV_SHA256,
                "anchor_admitted": 1120 - 30,
                "breach": True,
                "audit_status": "AUDIT_BREACH",
                "action": "REJECT_INTERVAL",
                "rejection_mode": "whole_interval",
                "failed_checkpoint_preserved": True,
                "append_only_evidence_preserved": True,
                "schedule_cursor_advanced_past_interval": True,
                "rejection_receipts_count": 1,
                "restored_model_storage_sha256": s0_sha,
                "restored_model_snapshot_sha256": sn0_sha,
                "restored_model_epoch": 0,
                "restored_accepted_updates": 0,
                "restored_family_counters": {"vector": 0, "route": 0, "combine": 0, "relation": 0},
            }
        )
    ap_3consec_full["arms"]["relation"]["audits"] = consec_audits
    res_ap_3consec = verify_admission_preserving_guard_bundle(ap_3consec_full)
    record(
        "54_admission_preserving_three_consecutive_full_rejections_noop_stop",
        res_ap_3consec["verdict"] == "RULE_EFFECTIVELY_NOOP"
        and res_ap_3consec["mechanism_negative"] is False
        and res_ap_3consec["keep"] is False
        and res_ap_3consec["arms"]["relation"]["max_consecutive_full_rejections"] == 3
        and res_ap_3consec["arms"]["relation"]["noop_triggered_at_batch"] == 192,
        f"verdict={res_ap_3consec['verdict']}, reasons={res_ap_3consec['noop_reasons']}",
    )

    # T55: Rejection rate rejected_intervals / audited_intervals > 0.50 at Q1 (33 / 64, max_consecutive == 2 < 3) -> RULE_EFFECTIVELY_NOOP
    ap_q1_rate = copy.deepcopy(base_ap)
    # Build 64 Q1 intervals where 33 breach without 3 consecutive (e.g. pattern of 2 breaches, 1 pass for 11 blocks = 22 breaches in 33 intervals, plus 11 single breaches separated by passes = 33 breaches in 55 intervals, remaining 9 pass -> 33/64 breaches, max_consecutive = 2)
    q1_breach_set: set[int] = set()
    # Indices 1..64:
    # Put breaches at (1,2), (4,5), (7,8) -> 6 breaches; and odd indices 11,13,...,63 (27 indices) -> total 6 + 27 = 33 breaches out of 64, max consecutive = 2!
    q1_breach_set.update({1, 2, 4, 5, 7, 8})
    for odd_idx in range(11, 65, 2):
        q1_breach_set.add(odd_idx)
    assert len(q1_breach_set) == 33

    q1_rate_audits = [copy.deepcopy(base_ap["arms"]["relation"]["audits"][0])]
    cur_s = q1_rate_audits[0]["model_storage_sha256"]
    cur_sn = q1_rate_audits[0]["model_snapshot_sha256"]
    cur_ep = 0
    cur_acc = 0
    cur_fc = {"vector": 0, "route": 0, "combine": 0, "relation": 0}
    for idx in range(1, 65):
        b = idx * 64
        ns = hashlib.sha256(f"q1r:s:{idx}".encode("utf-8")).hexdigest()
        nsn = hashlib.sha256(f"q1r:sn:{idx}".encode("utf-8")).hexdigest()
        nfc = dict(cur_fc)
        nfc["vector"] += 1
        nacc = sum(nfc.values())
        if idx in q1_breach_set:
            q1_rate_audits.append(
                {
                    "audit_batch": b,
                    "synchronous_pause": True,
                    "concurrent_updates": 0,
                    "model_storage_sha256": ns,
                    "model_snapshot_sha256": nsn,
                    "model_epoch": nacc,
                    "accepted_updates": nacc,
                    "driver_cursor": {
                        "completed_batches": b,
                        "accepted_updates": nacc,
                        "corpus_pass": 0,
                        "next_target": b * 64,
                        "next_coordinate": b % 45,
                    },
                    "family_counters": nfc,
                    "lineage_sha256": EXPECTED_A8_RELATION_LINEAGE_SHA256,
                    "anchor_csv_sha256": EXPECTED_TRAIN_PSUCC_TAIL_ANCHOR_CSV_SHA256,
                    "anchor_admitted": 1120 - 28,
                    "breach": True,
                    "audit_status": "AUDIT_BREACH",
                    "action": "REJECT_INTERVAL",
                    "rejection_mode": "whole_interval",
                    "failed_checkpoint_preserved": True,
                    "append_only_evidence_preserved": True,
                    "schedule_cursor_advanced_past_interval": True,
                    "rejection_receipts_count": 1,
                    "restored_model_storage_sha256": cur_s,
                    "restored_model_snapshot_sha256": cur_sn,
                    "restored_model_epoch": cur_ep,
                    "restored_accepted_updates": cur_acc,
                    "restored_family_counters": dict(cur_fc),
                }
            )
        else:
            cur_s = ns
            cur_sn = nsn
            cur_ep = nacc
            cur_acc = nacc
            cur_fc = nfc
            q1_rate_audits.append(
                {
                    "audit_batch": b,
                    "synchronous_pause": True,
                    "concurrent_updates": 0,
                    "model_storage_sha256": cur_s,
                    "model_snapshot_sha256": cur_sn,
                    "model_epoch": cur_ep,
                    "accepted_updates": cur_acc,
                    "driver_cursor": {
                        "completed_batches": b,
                        "accepted_updates": cur_acc,
                        "corpus_pass": 0,
                        "next_target": b * 64,
                        "next_coordinate": b % 45,
                    },
                    "family_counters": dict(cur_fc),
                    "lineage_sha256": EXPECTED_A8_RELATION_LINEAGE_SHA256,
                    "anchor_csv_sha256": EXPECTED_TRAIN_PSUCC_TAIL_ANCHOR_CSV_SHA256,
                    "anchor_admitted": 1120 - 5,
                    "breach": False,
                    "action": "COMMIT",
                }
            )
    ap_q1_rate["arms"]["relation"]["audits"] = q1_rate_audits
    res_ap_q1_rate = verify_admission_preserving_guard_bundle(ap_q1_rate)
    record(
        "55_admission_preserving_q1_rejection_rate_above_50pct_noop_stop",
        res_ap_q1_rate["verdict"] == "RULE_EFFECTIVELY_NOOP"
        and res_ap_q1_rate["mechanism_negative"] is False
        and res_ap_q1_rate["keep"] is False
        and res_ap_q1_rate["arms"]["relation"]["rejected_intervals"] == 33
        and res_ap_q1_rate["arms"]["relation"]["audited_intervals"] == 64
        and res_ap_q1_rate["arms"]["relation"]["max_consecutive_full_rejections"] == 2,
        f"verdict={res_ap_q1_rate['verdict']}, rejected=33/64, max_consec=2",
    )

    # T56: Missing final 20-batch tail audit at 16,404 (257 instead of 258 evaluations) or FrozenZero accepted_relation > 0 -> INSTRUMENT_UNREACHABLE_D31
    ap_no_tail = copy.deepcopy(base_ap)
    ap_no_tail["arms"]["frozen_zero"]["audits"] = ap_no_tail["arms"]["frozen_zero"]["audits"][:257]
    res_ap_no_tail = verify_admission_preserving_guard_bundle(ap_no_tail)

    ap_fz_rel = copy.deepcopy(base_ap)
    ap_fz_rel["arms"]["frozen_zero"]["audits"][1]["family_counters"]["relation"] = 1
    ap_fz_rel["arms"]["frozen_zero"]["audits"][1]["accepted_updates"] += 1
    ap_fz_rel["arms"]["frozen_zero"]["audits"][1]["model_epoch"] += 1
    res_ap_fz_rel = verify_admission_preserving_guard_bundle(ap_fz_rel)

    record(
        "56_admission_preserving_missing_tail_16404_or_frozen_zero_relation_d31",
        res_ap_no_tail["verdict"] == "INSTRUMENT_UNREACHABLE_D31"
        and "frozen_zero_missing_mandatory_final_16404_tail_audit" in res_ap_no_tail["faults"]
        and res_ap_fz_rel["verdict"] == "INSTRUMENT_UNREACHABLE_D31"
        and "frozen_zero_audit_64_accepted_relation_nonzero" in res_ap_fz_rel["faults"],
        f"no_tail={res_ap_no_tail['faults']}, fz_rel={res_ap_fz_rel['faults']}",
    )

    # T57: Integrated #2782 19-file driver_source_sha256, 3-file compiled_source_sha256, 6 modules & lineage SHAs
    int_src = verify_integrated_audited_source_contract(repo_root)
    record(
        "57_integrated_audited_source_contract_and_lineage_pass",
        int_src["status"] == "PASS"
        and int_src["driver_source_sha256"] == EXPECTED_INTEGRATED_DRIVER_SOURCE_SHA256
        and int_src["compiled_source_sha256"] == EXPECTED_COMPILED_SOURCE_SHA256
        and len(int_src["modules"]) == 6
        and staged["relation"]["integrated_lineage_sha256"]
        == EXPECTED_INTEGRATED_A8_RELATION_LINEAGE_SHA256
        and staged["frozen_zero"]["integrated_lineage_sha256"]
        == EXPECTED_INTEGRATED_A8_FROZEN_ZERO_LINEAGE_SHA256,
        f"driver={int_src['driver_source_sha256'][:12]}, compiled={int_src['compiled_source_sha256'][:12]}",
    )

    # T58: TrainAnchorManifest (`uor-r4.train-copy-successor-anchor/v1`) & OriginalWindowPin verification + tamper refusal
    syn_manifest = build_synthetic_train_anchor_manifest()
    res_manifest_ok = verify_train_anchor_manifest(
        syn_manifest, require_canonical_3847=False
    )
    bad_manifest_csv = copy.deepcopy(syn_manifest)
    bad_manifest_csv["row_csv_sha256"] = "0" * 64
    manifest_tamper_rejected = False
    try:
        verify_train_anchor_manifest(bad_manifest_csv, require_canonical_3847=False)
    except VerificationError as exc:
        manifest_tamper_rejected = "row_csv_sha256 mismatch" in str(exc)

    bad_manifest_split = copy.deepcopy(syn_manifest)
    bad_manifest_split["corpus"]["source"]["split"] = "dev"
    split_tamper_rejected = False
    try:
        verify_train_anchor_manifest(bad_manifest_split, require_canonical_3847=False)
    except VerificationError as exc:
        split_tamper_rejected = "must be TRAIN" in str(exc)

    record(
        "58_train_anchor_manifest_and_original_window_pin_pass_and_tamper_rejected",
        res_manifest_ok["status"] == "PASS"
        and res_manifest_ok["rows"] == 4
        and manifest_tamper_rejected
        and split_tamper_rejected,
        f"manifest_sha={res_manifest_ok['manifest_sha256'][:12]}, tamper_rejected={manifest_tamper_rejected and split_tamper_rejected}",
    )

    # T59: Finite source qualification bundle (`attention_audit_qualification.rs`) valid PASS
    base_qual = build_synthetic_audited_qualification_bundle(repo_root)
    res_qual_ok = verify_audited_qualification_artifacts(base_qual)
    record(
        "59_audited_qualification_valid_bundle_pass",
        res_qual_ok["status"] == "PASS"
        and res_qual_ok["verdict"] == "QUALIFICATION_PASS_READY_FOR_SEPARATE_DOSE_D40_PLAN"
        and res_qual_ok["checks_passed"] == 5
        and res_qual_ok["mechanism_negative"] is False
        and res_qual_ok["keep"] is False,
        f"verdict={res_qual_ok['verdict']}, adm={res_qual_ok['admission_sha256'][:12]}",
    )

    # T60: Finite source qualification authorization actor/verdict tamper, limits warmup tamper, or check failure -> INSTRUMENT_UNREACHABLE_D31
    qual_bad_auth = copy.deepcopy(base_qual)
    qual_bad_auth["authorizations"]["qualification_fit"]["actor"] = "Casey"
    res_qual_bad_auth = verify_audited_qualification_artifacts(qual_bad_auth)

    qual_bad_lim = copy.deepcopy(base_qual)
    qual_bad_lim["plan"]["limits"]["throughput_warmup_attempts_per_arm"] = 1
    res_qual_bad_lim = verify_audited_qualification_artifacts(qual_bad_lim)

    qual_bad_chk = copy.deepcopy(base_qual)
    qual_bad_chk["report"]["qualification"]["checks"][4]["passed"] = False
    res_qual_bad_chk = verify_audited_qualification_artifacts(qual_bad_chk)

    record(
        "60_audited_qualification_auth_or_limits_or_battery_tamper_d31",
        res_qual_bad_auth["verdict"] == "INSTRUMENT_UNREACHABLE_D31"
        and "qualification_authorization_qualification_fit_invalid" in res_qual_bad_auth["faults"]
        and res_qual_bad_lim["verdict"] == "INSTRUMENT_UNREACHABLE_D31"
        and "qualification_plan_limits_invalid" in res_qual_bad_lim["faults"]
        and res_qual_bad_chk["verdict"] == "INSTRUMENT_UNREACHABLE_D31"
        and "qualification_five_check_battery_failed" in res_qual_bad_chk["faults"],
        f"auth={res_qual_bad_auth['faults']}, lim={res_qual_bad_lim['faults']}, chk={res_qual_bad_chk['faults']}",
    )

    # T61: Finite source qualification calibration > 15% gate cost -> STOP_OVER_15_PERCENT_GATE_COST, and arithmetic tamper -> INSTRUMENT_UNREACHABLE_D31
    qual_over_15 = copy.deepcopy(base_qual)
    qual_over_15["report"]["calibration"]["projected_ungated_two_arm_dose_ns"] = (
        qual_over_15["report"]["calibration"]["projected_added_gate_ns"] * 5
    )  # R = 0.20 > 0.15
    qual_over_15["report"]["calibration"]["passes_15_percent_gate"] = False
    qual_over_15["report"]["status"] = "STOP_OVER_15_PERCENT_GATE_COST"
    res_qual_over_15 = verify_audited_qualification_artifacts(qual_over_15)

    qual_bad_math = copy.deepcopy(base_qual)
    qual_bad_math["report"]["calibration"]["projected_added_gate_ns"] += 1
    res_qual_bad_math = verify_audited_qualification_artifacts(qual_bad_math)

    record(
        "61_audited_qualification_calibration_over_15pct_stop_and_formula_mismatch_d31",
        res_qual_over_15["verdict"] == "STOP_OVER_15_PERCENT_GATE_COST"
        and res_qual_over_15["mechanism_negative"] is False
        and res_qual_over_15["keep"] is False
        and res_qual_bad_math["verdict"] == "INSTRUMENT_UNREACHABLE_D31"
        and "qualification_calibration_arithmetic_or_gate_flag_mismatch"
        in res_qual_bad_math["faults"],
        f"over_15={res_qual_over_15['verdict']}, bad_math={res_qual_bad_math['faults']}",
    )

    # T62: Integrated #2782 audited dose bundle (`attention_audited.rs`) valid PASS
    base_aud_dose = build_synthetic_integrated_audited_dose_bundle(repo_root)
    res_aud_dose_ok = verify_integrated_audited_dose_artifacts(base_aud_dose)
    record(
        "62_integrated_audited_dose_valid_bundle_pass",
        res_aud_dose_ok["status"] == "PASS"
        and res_aud_dose_ok["verdict"]
        == "AUDITED_DOSE_COMPLETE_PENDING_PAIRED_DISCRIMINATOR"
        and res_aud_dose_ok["mechanism_negative"] is False
        and res_aud_dose_ok["keep"] is False,
        f"verdict={res_aud_dose_ok['verdict']}, rel_adm={res_aud_dose_ok['admission_shas']['relation'][:12]}",
    )

    # T63: Integrated audited dose cost attestation tamper, resource cap breach, or restore observation mismatch -> INSTRUMENT_UNREACHABLE_D31
    ad_bad_att = copy.deepcopy(base_aud_dose)
    ad_bad_att["cost_attestations"]["pm_dispatch"]["actor"] = "Claude"
    res_ad_bad_att = verify_integrated_audited_dose_artifacts(ad_bad_att)

    ad_over_cap = copy.deepcopy(base_aud_dose)
    ad_over_cap["arms"]["relation"]["resource_observation"]["observed_cost_cents"] = 99999
    res_ad_over_cap = verify_integrated_audited_dose_artifacts(ad_over_cap)

    ad_bad_rest = copy.deepcopy(base_aud_dose)
    ad_bad_rest["arms"]["relation"]["restore_observations"][0][
        "restored_model_storage_sha256"
    ] = "f" * 64
    res_ad_bad_rest = verify_integrated_audited_dose_artifacts(ad_bad_rest)

    record(
        "63_integrated_audited_dose_cost_attestation_or_resource_cap_or_restore_mismatch_d31",
        res_ad_bad_att["verdict"] == "INSTRUMENT_UNREACHABLE_D31"
        and "dose_cost_attestation_pm_dispatch_invalid" in res_ad_bad_att["faults"]
        and res_ad_over_cap["verdict"] == "INSTRUMENT_UNREACHABLE_D31"
        and "relation_resource_observation_invalid_or_exceeds_approved_cap"
        in res_ad_over_cap["faults"]
        and res_ad_bad_rest["verdict"] == "INSTRUMENT_UNREACHABLE_D31"
        and "relation_breach_artifact_0_restore_mismatch" in res_ad_bad_rest["faults"],
        f"att={res_ad_bad_att['faults']}, cap={res_ad_over_cap['faults']}, rest={res_ad_bad_rest['faults']}",
    )

    # T64: Integrated audited dose RULE_EFFECTIVELY_NOOP diagnostic stop and cross-arm directory/pin mismatch -> INSTRUMENT_UNREACHABLE_D31
    ad_noop = copy.deepcopy(base_aud_dose)
    ad_noop["arms"]["relation"]["report"]["status"] = "RULE_EFFECTIVELY_NOOP"
    res_ad_noop = verify_integrated_audited_dose_artifacts(ad_noop)

    ad_same_dir = copy.deepcopy(base_aud_dose)
    ad_same_dir["arms"]["frozen_zero"]["plan"]["run_state_directory"] = ad_same_dir["arms"][
        "relation"
    ]["plan"]["run_state_directory"]
    res_ad_same_dir = verify_integrated_audited_dose_artifacts(ad_same_dir)

    record(
        "64_integrated_audited_dose_noop_or_q1_stop_and_cross_arm_pin_mismatch",
        res_ad_noop["verdict"] == "RULE_EFFECTIVELY_NOOP"
        and res_ad_noop["mechanism_negative"] is False
        and res_ad_noop["keep"] is False
        and res_ad_same_dir["verdict"] == "INSTRUMENT_UNREACHABLE_D31"
        and "both_arms_share_same_run_state_directory" in res_ad_same_dir["faults"],
        f"noop={res_ad_noop['verdict']}, same_dir={res_ad_same_dir['faults']}",
    )

    return {
        "status": "PASS",
        "total_tests": len(results),
        "passed_tests": len(results),
        "tests": results,
    }


def _extract_checkpoint_root_ids_and_cells(
    table: list[int], label: str
) -> tuple[list[int], list[int], int, int, int]:
    """Validate 131,072-word table roots (`0` off-orbit blocks) and compute 4,096 token cells and occupancy."""
    require(
        isinstance(table, list) and len(table) == 4096 * 32,
        f"{label}: table must have 131,072 i32 words, got {len(table) if isinstance(table, list) else type(table)!r}",
    )
    try:
        packed = struct.pack("<131072i", *table)
    except (struct.error, TypeError) as exc:
        raise VerificationError(f"{label}: invalid i32 word in table: {exc}") from exc

    mv = memoryview(packed)
    root_ids: list[int] = [0] * 16384
    for slot in range(16384):
        rid = ROOT_BYTES_TO_ID.get(mv[slot * 32 : (slot + 1) * 32])
        if rid is None:
            tok = slot // 4
            blk = slot % 4
            raise VerificationError(
                f"{label}: off-orbit root at slot {slot} (token={tok}, block={blk}): {table[slot * 8 : (slot + 1) * 8]}"
            )
        root_ids[slot] = rid

    cells: list[int] = [0] * 4096
    pop: list[int] = [0] * 256
    for tok in range(4096):
        c = root_ids[tok * 4] * 16 + root_ids[tok * 4 + 3]
        cells[tok] = c
        pop[c] += 1

    occupied_cells = sum(1 for count in pop if count > 0)
    require(
        occupied_cells == 256,
        f"{label}: cell occupancy collapse: occupied_cells={occupied_cells} < 256",
    )
    return root_ids, cells, occupied_cells, min(pop), max(pop)


def _fast_load_checkpoint_json(
    raw: bytes, ref_base_eval_bytes: bytes | None, label: str
) -> tuple[dict[str, Any], bytes | None]:
    """Parse a checkpoint JSON file while verifying byte-identity of the repeated 6.68 MB `baseline_evaluation` slice."""
    marker_start = b'"baseline_evaluation":'
    marker_end = b',"baseline_snapshot_metadata":'
    i0 = raw.find(marker_start)
    i1 = raw.find(marker_end, i0 if i0 != -1 else 0)
    if i0 == -1 or i1 == -1:
        doc = json.loads(raw.decode("utf-8"))
        return doc, ref_base_eval_bytes
    be_slice = raw[i0:i1]
    if ref_base_eval_bytes is None:
        doc = json.loads(raw.decode("utf-8"))
        be = doc.get("baseline_evaluation", {})
        agg = be.get("aggregate", {}) if isinstance(be, dict) else {}
        require(
            agg.get("scored_rows") == CANONICAL_DEV_FULL_ROWS
            and agg.get("target_admitted_rows") == EXPECTED_A8_CP0_FULL_ADMITTED
            and agg.get("top1_correct") == EXPECTED_A8_CP0_FULL_TOP1
            and be.get("evaluation_mutates_model") is False
            and be.get("updates_applied_by_evaluation") == 0,
            f"{label}: cp0 baseline_evaluation aggregate mismatch: {agg}",
        )
        return doc, be_slice
    require(
        be_slice == ref_base_eval_bytes,
        f"{label}: baseline_evaluation slice differs from cp0",
    )
    stripped = raw[:i0] + b'"baseline_evaluation":null' + raw[i1:]
    return json.loads(stripped.decode("utf-8")), ref_base_eval_bytes


def load_vast_56670997_census_bundle(
    census_archive_dir: Path | None = None,
    relation_checkpoints_dir: Path | None = None,
    frozen_checkpoints_dir: Path | None = None,
    steps_dir: Path | None = None,
) -> dict[str, Any]:
    """Load and SHA-verify the real `vast-56670997` 65+258 checkpoints, `steps.jsonl` logs, `q1.json`, and `report.json`."""
    archive_dir = census_archive_dir or DEFAULT_VAST_56670997_ARCHIVE_DIR
    rel_cp_dir = relation_checkpoints_dir or DEFAULT_VAST_56670997_RELATION_CHECKPOINTS_DIR
    fz_cp_dir = frozen_checkpoints_dir or DEFAULT_VAST_56670997_FROZEN_CHECKPOINTS_DIR
    s_dir = steps_dir or DEFAULT_VAST_56670997_STEPS_DIR

    manifest_path = archive_dir / "runs.manifest.sha256"
    manifest_bytes = read_bytes_checked(manifest_path, "runs.manifest.sha256")
    manifest_sha = hashlib.sha256(manifest_bytes).hexdigest()
    require(
        manifest_sha == EXPECTED_VAST_56670997_RUNS_MANIFEST_SHA256
        and len(manifest_bytes) == EXPECTED_VAST_56670997_RUNS_MANIFEST_BYTES,
        f"runs.manifest.sha256 SHA/bytes mismatch: {manifest_sha} ({len(manifest_bytes)} B)",
    )
    manifest_entries: dict[str, str] = {}
    for line in manifest_bytes.decode("utf-8").splitlines():
        line = line.strip()
        if not line:
            continue
        parts = line.split()
        require(len(parts) == 2 and is_hex64(parts[0]), f"invalid manifest line: {line!r}")
        manifest_entries[parts[1]] = parts[0]
    require(
        len(manifest_entries) == EXPECTED_VAST_56670997_RUNS_MANIFEST_LINES,
        f"runs.manifest.sha256 line count {len(manifest_entries)} != {EXPECTED_VAST_56670997_RUNS_MANIFEST_LINES}",
    )

    def load_arm_checkpoints(cp_dir: Path, arm_dir_name: str, expected_count: int) -> list[dict[str, Any]]:
        cp_paths = sorted(cp_dir.glob("checkpoint-*.json"))
        require(
            len(cp_paths) == expected_count,
            f"{arm_dir_name}: expected {expected_count} checkpoint-*.json files in {cp_dir}, found {len(cp_paths)}",
        )
        out_cps: list[dict[str, Any]] = []
        ref_be: bytes | None = None
        for p in cp_paths:
            raw = read_bytes_checked(p, p.name)
            sha = hashlib.sha256(raw).hexdigest()
            doc, ref_be = _fast_load_checkpoint_json(raw, ref_be, f"{arm_dir_name}/{p.name}")
            cp_inner = doc.get("checkpoint", {})
            cursor = cp_inner.get("cursor", {})
            model = cp_inner.get("model", {})
            out_cps.append(
                {
                    "name": p.name,
                    "manifest_key": f"./claude/{arm_dir_name}/{p.name}",
                    "sha256": sha,
                    "bytes": len(raw),
                    "completed_batches": cursor.get("completed_batches"),
                    "accepted_updates": cursor.get("accepted_updates"),
                    "epoch": model.get("epoch"),
                    "gates": model.get("gates"),
                    "routes": model.get("routes"),
                    "table": model.get("table"),
                }
            )
        return out_cps

    rel_cps = load_arm_checkpoints(rel_cp_dir, "a8-dose-relation-seed1", 65)
    fz_cps = load_arm_checkpoints(fz_cp_dir, "a8-dose-frozen-zero-seed1", 258)

    def resolve_steps_path(arm_dir_name: str) -> Path:
        cand1 = s_dir / "runs" / "claude" / arm_dir_name / "steps.jsonl"
        if cand1.is_file():
            return cand1
        cand2 = s_dir / "claude" / arm_dir_name / "steps.jsonl"
        if cand2.is_file():
            return cand2
        return s_dir / arm_dir_name / "steps.jsonl"

    rel_steps_path = resolve_steps_path("a8-dose-relation-seed1")
    fz_steps_path = resolve_steps_path("a8-dose-frozen-zero-seed1")
    rel_steps_bytes = read_bytes_checked(rel_steps_path, "Relation steps.jsonl")
    fz_steps_bytes = read_bytes_checked(fz_steps_path, "FrozenZero steps.jsonl")
    rel_steps_sha = hashlib.sha256(rel_steps_bytes).hexdigest()
    fz_steps_sha = hashlib.sha256(fz_steps_bytes).hexdigest()
    rel_steps = [json.loads(line) for line in rel_steps_bytes.decode("utf-8").splitlines() if line.strip()]
    fz_steps = [json.loads(line) for line in fz_steps_bytes.decode("utf-8").splitlines() if line.strip()]

    rel_q1_path = (
        archive_dir
        / "runs"
        / "claude"
        / "a8-dose-custody-20261019"
        / "state"
        / "a8-relation-seed1"
        / "q1.json"
    )
    fz_q1_path = (
        archive_dir
        / "runs"
        / "claude"
        / "a8-dose-custody-20261019"
        / "state"
        / "a8-frozen-zero-seed1"
        / "q1.json"
    )
    fz_report_path = archive_dir / "runs" / "claude" / "a8-dose-frozen-zero-seed1" / "report.json"
    rel_report_path = archive_dir / "runs" / "claude" / "a8-dose-relation-seed1" / "report.json"

    rel_q1_bytes = read_bytes_checked(rel_q1_path, "Relation q1.json")
    fz_q1_bytes = read_bytes_checked(fz_q1_path, "FrozenZero q1.json")
    fz_report_bytes = read_bytes_checked(fz_report_path, "FrozenZero report.json")
    rel_report_bytes = read_bytes_checked(rel_report_path, "Relation report.json")

    return {
        "manifest_sha256": manifest_sha,
        "manifest_bytes": len(manifest_bytes),
        "manifest_entries": manifest_entries,
        "relation_checkpoints": rel_cps,
        "frozen_zero_checkpoints": fz_cps,
        "relation_steps_sha256": rel_steps_sha,
        "relation_steps_bytes": len(rel_steps_bytes),
        "relation_steps": rel_steps,
        "frozen_zero_steps_sha256": fz_steps_sha,
        "frozen_zero_steps_bytes": len(fz_steps_bytes),
        "frozen_zero_steps": fz_steps,
        "relation_q1_sha256": hashlib.sha256(rel_q1_bytes).hexdigest(),
        "relation_q1_bytes": len(rel_q1_bytes),
        "relation_q1": json.loads(rel_q1_bytes.decode("utf-8")),
        "frozen_zero_q1_sha256": hashlib.sha256(fz_q1_bytes).hexdigest(),
        "frozen_zero_q1_bytes": len(fz_q1_bytes),
        "frozen_zero_q1": json.loads(fz_q1_bytes.decode("utf-8")),
        "frozen_zero_report_sha256": hashlib.sha256(fz_report_bytes).hexdigest(),
        "frozen_zero_report_bytes": len(fz_report_bytes),
        "frozen_zero_report": json.loads(fz_report_bytes.decode("utf-8")),
        "relation_report_sha256": hashlib.sha256(rel_report_bytes).hexdigest(),
        "relation_report_bytes": len(rel_report_bytes),
        "relation_report": json.loads(rel_report_bytes.decode("utf-8")),
        "expected": EXPECTED_VAST_56670997_CENSUS_EXPECTATIONS,
    }


def verify_a8_census_and_attribution(
    bundle: dict[str, Any] | None = None,
    *,
    census_archive_dir: Path | None = None,
    relation_checkpoints_dir: Path | None = None,
    frozen_checkpoints_dir: Path | None = None,
    steps_dir: Path | None = None,
) -> dict[str, Any]:
    """Verify 65+258 checkpoints, steps.jsonl slot bijection, step-175 Gate(21) bifurcation,
    0 transient cell excursions, and two-sided + 4-way cross-arm P_succ admission decompositions.
    """
    if bundle is None:
        bundle = load_vast_56670997_census_bundle(
            census_archive_dir=census_archive_dir,
            relation_checkpoints_dir=relation_checkpoints_dir,
            frozen_checkpoints_dir=frozen_checkpoints_dir,
            steps_dir=steps_dir,
        )

    expected = bundle.get("expected", EXPECTED_VAST_56670997_CENSUS_EXPECTATIONS)
    manifest_entries = bundle.get("manifest_entries", {})
    require(
        isinstance(manifest_entries, dict)
        and bundle.get("manifest_sha256") == expected["manifest_sha256"]
        and len(manifest_entries) == expected["manifest_lines"],
        f"runs.manifest.sha256 verification failed: sha={bundle.get('manifest_sha256')}, lines={len(manifest_entries)}",
    )
    require(
        manifest_entries.get("./claude/a8-dose-gates-20261019/relation/cells/token_cells.u16")
        == expected["cp0_token_cells_u16_sha256"]
        and manifest_entries.get("./claude/a8-dose-gates-20261019/frozen-zero/cells/token_cells.u16")
        == expected["cp0_token_cells_u16_sha256"],
        "runs.manifest.sha256 cp0 token_cells.u16 SHA-256 mismatch",
    )

    # Verify non-checkpoint file pins against runs.manifest.sha256 and expected constants
    pin_checks = (
        (
            "./claude/a8-dose-relation-seed1/steps.jsonl",
            bundle.get("relation_steps_sha256"),
            expected["relation_steps_sha256"],
        ),
        (
            "./claude/a8-dose-frozen-zero-seed1/steps.jsonl",
            bundle.get("frozen_zero_steps_sha256"),
            expected["frozen_zero_steps_sha256"],
        ),
        (
            "./claude/a8-dose-custody-20261019/state/a8-relation-seed1/q1.json",
            bundle.get("relation_q1_sha256"),
            expected["relation_q1_json_sha256"],
        ),
        (
            "./claude/a8-dose-custody-20261019/state/a8-frozen-zero-seed1/q1.json",
            bundle.get("frozen_zero_q1_sha256"),
            expected["frozen_zero_q1_json_sha256"],
        ),
        (
            "./claude/a8-dose-frozen-zero-seed1/report.json",
            bundle.get("frozen_zero_report_sha256"),
            expected["frozen_zero_report_json_sha256"],
        ),
        (
            "./claude/a8-dose-relation-seed1/report.json",
            bundle.get("relation_report_sha256"),
            expected["relation_report_json_sha256"],
        ),
    )
    for m_key, actual_sha, exp_sha in pin_checks:
        require(
            actual_sha == exp_sha and manifest_entries.get(m_key) == exp_sha,
            f"manifest/file SHA-256 mismatch for {m_key}: actual={actual_sha}, manifest={manifest_entries.get(m_key)}, expected={exp_sha}",
        )

    rel_cps = bundle.get("relation_checkpoints", [])
    fz_cps = bundle.get("frozen_zero_checkpoints", [])
    rel_steps = bundle.get("relation_steps", [])
    fz_steps = bundle.get("frozen_zero_steps", [])
    require(
        len(rel_cps) == expected["relation_checkpoint_count"]
        and len(fz_cps) == expected["frozen_zero_checkpoint_count"],
        f"checkpoint count mismatch: rel={len(rel_cps)} (exp {expected['relation_checkpoint_count']}), fz={len(fz_cps)} (exp {expected['frozen_zero_checkpoint_count']})",
    )
    require(
        len(rel_steps) == expected["relation_steps_lines"]
        and len(fz_steps) == expected["frozen_zero_steps_lines"],
        f"steps.jsonl line count mismatch: rel={len(rel_steps)}, fz={len(fz_steps)}",
    )
    q1_batch_key = expected.get("q1_batch", CANONICAL_Q1_BATCHES)
    final_batch_key = expected.get("final_batch", CANONICAL_ATTENTION_BATCHES)

    def verify_arm_trajectory(
        arm_name: str,
        cps: list[dict[str, Any]],
        steps: list[dict[str, Any]],
        is_frozen_zero: bool,
    ) -> dict[str, Any]:
        prev_batch: int | None = None
        prev_root_ids: list[int] | None = None
        prev_cells: list[int] | None = None
        cp0_cells: list[int] | None = None
        q1_cells: list[int] | None = None
        ever_moved_tokens: set[int] = set()
        ever_moved_by_q1: set[int] = set()
        interval_mismatches = 0
        pop_by_batch: dict[int, dict[str, int]] = {}

        for idx, cp in enumerate(cps):
            m_key = cp.get("manifest_key", "")
            cp_sha = cp.get("sha256", "")
            require(
                is_hex64(cp_sha) and manifest_entries.get(m_key) == cp_sha,
                f"{arm_name} checkpoint {m_key} SHA-256 {cp_sha} != manifest {manifest_entries.get(m_key)}",
            )
            batch = cp.get("completed_batches")
            require(is_u64(batch), f"{arm_name} checkpoint {m_key} invalid completed_batches: {batch!r}")
            gates = cp.get("gates")
            require(
                isinstance(gates, list) and len(gates) == 45 and all(g in (-1, 0, 1) for g in gates),
                f"{arm_name} checkpoint {m_key} invalid 45-trit gates",
            )
            if is_frozen_zero:
                require(
                    gates[20:45] == [0] * 25,
                    f"{arm_name} checkpoint {m_key} has non-zero relation gate trit: {gates[20:45]}",
                )

            root_ids, cells, occ, min_pop, max_pop = _extract_checkpoint_root_ids_and_cells(
                cp.get("table", []), f"{arm_name}/{cp.get('name', idx)}"
            )
            pop_by_batch[batch] = {
                "occupied_cells": occ,
                "min_pop": min_pop,
                "max_pop": max_pop,
            }

            if idx == 0:
                require(batch == 0, f"{arm_name} first checkpoint must have batch 0, got {batch}")
                cp0_cells = cells
                cp0_word_sha = hashlib.sha256(
                    struct.pack("<131072i", *cp["table"])
                ).hexdigest()
                require(
                    cp0_word_sha == EXPECTED_A8_TABLE_WORDS_SHA256,
                    f"{arm_name} cp0 table_sha256 {cp0_word_sha} != {EXPECTED_A8_TABLE_WORDS_SHA256}",
                )
                cp0_u16_sha = hashlib.sha256(struct.pack("<4096H", *cells)).hexdigest()
                require(
                    cp0_u16_sha == expected["cp0_token_cells_u16_sha256"],
                    f"{arm_name} cp0 token_cells.u16 SHA-256 {cp0_u16_sha} != {expected['cp0_token_cells_u16_sha256']}",
                )
                require(
                    gates == EXPECTED_ACTIVE_LEAF_GATES,
                    f"{arm_name} cp0 gates mismatch",
                )
            else:
                assert prev_batch is not None and prev_root_ids is not None and prev_cells is not None
                require(
                    batch > prev_batch and batch <= len(steps),
                    f"{arm_name} non-monotonic or out-of-range checkpoint batch {prev_batch} -> {batch}",
                )
                interval_steps = steps[prev_batch:batch]
                for offset, st in enumerate(interval_steps):
                    exp_b = prev_batch + offset
                    cb = st.get("cursor_before", {}).get("completed_batches")
                    ca = st.get("cursor_after", {}).get("completed_batches")
                    require(
                        cb == exp_b and ca == exp_b + 1,
                        f"{arm_name} step {exp_b} cursor batch discontinuity: {cb} -> {ca}",
                    )
                    if is_frozen_zero and st.get("coordinate", {}).get("kind") == "gate":
                        g_idx = st["coordinate"].get("index", -1)
                        if g_idx >= 20:
                            require(
                                st.get("applied") is False,
                                f"FrozenZero applied a relation gate update at step {exp_b}: {st}",
                            )

                changed_slots = {
                    slot for slot in range(16384) if root_ids[slot] != prev_root_ids[slot]
                }
                applied_vec_steps = [
                    st
                    for st in interval_steps
                    if st.get("applied") is True
                    and st.get("coordinate", {}).get("kind") == "vector_block"
                ]
                step_slots: set[int] = set()
                for st in applied_vec_steps:
                    s_idx = st["coordinate"].get("index")
                    cand = st.get("candidate")
                    require(
                        is_u64(s_idx) and 0 <= s_idx < 16384 and is_u64(cand) and 1 <= cand < 16,
                        f"{arm_name} invalid applied vector_block step: {st}",
                    )
                    require(
                        s_idx not in step_slots,
                        f"{arm_name} duplicate applied vector_block slot {s_idx} in interval {prev_batch}->{batch}",
                    )
                    step_slots.add(s_idx)

                if changed_slots != step_slots:
                    interval_mismatches += 1
                    raise VerificationError(
                        f"{arm_name} interval {prev_batch}->{batch} vector_block slot bijection mismatch: "
                        f"table_diff={sorted(changed_slots)} vs steps_applied={sorted(step_slots)}"
                    )

                changed_cell_tokens = {
                    tok for tok in range(4096) if cells[tok] != prev_cells[tok]
                }
                expected_cell_moved_tokens = {
                    slot // 4 for slot in changed_slots if (slot % 4) in (0, 3)
                }
                require(
                    changed_cell_tokens == expected_cell_moved_tokens,
                    f"{arm_name} interval {prev_batch}->{batch} token cell movement mismatch: "
                    f"actual={sorted(changed_cell_tokens)} vs block_0_3={sorted(expected_cell_moved_tokens)}",
                )

            assert cp0_cells is not None
            curr_diff_from_cp0 = {tok for tok in range(4096) if cells[tok] != cp0_cells[tok]}
            ever_moved_tokens |= curr_diff_from_cp0
            if batch <= q1_batch_key:
                ever_moved_by_q1 |= curr_diff_from_cp0
            if batch == q1_batch_key:
                q1_cells = cells
            # Check 0 transient cell excursions at every checkpoint
            transient_now = ever_moved_tokens - curr_diff_from_cp0
            require(
                len(transient_now) == 0,
                f"{arm_name} transient cell excursion detected at batch {batch} for tokens {sorted(transient_now)}",
            )

            prev_batch = batch
            prev_root_ids = root_ids
            prev_cells = cells

        assert cp0_cells is not None and prev_cells is not None
        end_moved_tokens = {tok for tok in range(4096) if prev_cells[tok] != cp0_cells[tok]}
        q1_moved_tokens = (
            {tok for tok in range(4096) if q1_cells[tok] != cp0_cells[tok]}
            if q1_cells is not None
            else end_moved_tokens
        )
        return {
            "checkpoints_verified": len(cps),
            "intervals_verified": len(cps) - 1,
            "interval_mismatches": interval_mismatches,
            "transient_excursions": len(ever_moved_tokens - end_moved_tokens),
            "q1_transient_excursions": len(ever_moved_by_q1 - q1_moved_tokens),
            "pop_by_batch": pop_by_batch,
            "q1_moved_tokens": q1_moved_tokens,
            "end_moved_tokens": end_moved_tokens,
        }

    rel_traj = verify_arm_trajectory("relation", rel_cps, rel_steps, is_frozen_zero=False)
    fz_traj = verify_arm_trajectory("frozen_zero", fz_cps, fz_steps, is_frozen_zero=True)

    require(
        rel_cps[0]["sha256"] == expected["relation_cp0_sha256"]
        and rel_cps[-1]["sha256"] == expected["relation_q1_cp_sha256"]
        and fz_cps[0]["sha256"] == expected["frozen_zero_cp0_sha256"]
        and fz_cps[-1]["sha256"] == expected["frozen_zero_final_cp_sha256"],
        "endpoint checkpoint SHA-256 mismatch against expected pins",
    )
    fz_q1_matches = [cp for cp in fz_cps if cp["completed_batches"] == q1_batch_key]
    require(
        len(fz_q1_matches) == 1 and fz_q1_matches[0]["sha256"] == expected["frozen_zero_q1_cp_sha256"],
        "FrozenZero Q1 checkpoint SHA-256 mismatch",
    )

    exp_pop = expected["population_bounds"]
    require(
        rel_traj["pop_by_batch"][0] == exp_pop["cp0"]
        and fz_traj["pop_by_batch"][0] == exp_pop["cp0"]
        and rel_traj["pop_by_batch"][q1_batch_key] == exp_pop["relation_q1"]
        and fz_traj["pop_by_batch"][q1_batch_key] == exp_pop["frozen_zero_q1"]
        and fz_traj["pop_by_batch"][final_batch_key] == exp_pop["frozen_zero_final"],
        f"checkpoint population bounds mismatch: rel={rel_traj['pop_by_batch']}, fz={fz_traj['pop_by_batch']}",
    )

    # Verify row-key alignment across init_b3, Relation q1_b3, FrozenZero q1_b3, and FrozenZero final_b3
    rel_q1_doc = bundle["relation_q1"]
    fz_q1_doc = bundle["frozen_zero_q1"]
    fz_rep_doc = bundle["frozen_zero_report"]
    init_rows_rel = rel_q1_doc.get("init_b3", {}).get("rows", [])
    init_rows_fz = fz_q1_doc.get("init_b3", {}).get("rows", [])
    rel_q1_rows = rel_q1_doc.get("q1_b3", {}).get("rows", [])
    fz_q1_rows = fz_q1_doc.get("q1_b3", {}).get("rows", [])
    fz_final_rows = fz_rep_doc.get("final_b3", {}).get("rows", [])

    require(
        len(init_rows_rel) == CANONICAL_DEV_FULL_ROWS
        and init_rows_rel == init_rows_fz
        and len(rel_q1_rows) == CANONICAL_DEV_FULL_ROWS
        and len(fz_q1_rows) == CANONICAL_DEV_FULL_ROWS
        and len(fz_final_rows) == CANONICAL_DEV_FULL_ROWS,
        "DEV B3 row arrays must all have 14,287 rows with identical init_b3 across arms",
    )

    psucc_rows: list[dict[str, Any]] = []
    psucc_targets: set[int] = set()
    init_psucc_admitted = 0
    init_full_admitted = 0
    for idx in range(CANONICAL_DEV_FULL_ROWS):
        r0 = init_rows_rel[idx]
        key0 = (r0.get("row_ordinal"), r0.get("target_position"), r0.get("target_token"), r0.get("is_psucc"))
        require(r0.get("row_ordinal") == idx, f"non-sequential row_ordinal at index {idx}: {r0}")
        for label_r, other_rows in (
            ("relation_q1", rel_q1_rows),
            ("frozen_zero_q1", fz_q1_rows),
            ("frozen_zero_final", fz_final_rows),
        ):
            ro = other_rows[idx]
            key_o = (
                ro.get("row_ordinal"),
                ro.get("target_position"),
                ro.get("target_token"),
                ro.get("is_psucc"),
            )
            require(
                key_o == key0,
                f"DEV row key mismatch at index {idx} in {label_r}: {key_o} != {key0}",
            )
        if r0.get("target_admitted") is True:
            init_full_admitted += 1
        if r0.get("is_psucc") is True:
            psucc_rows.append(r0)
            psucc_targets.add(r0["target_token"])
            if r0.get("target_admitted") is True:
                init_psucc_admitted += 1

    require(
        len(psucc_rows) == CANONICAL_DEV_PSUCC_ROWS
        and len(psucc_targets) == 448
        and init_psucc_admitted == EXPECTED_A8_CP0_PSUCC_ADMITTED
        and init_full_admitted == EXPECTED_A8_CP0_FULL_ADMITTED,
        f"init_b3 invariant mismatch: psucc_rows={len(psucc_rows)}, targets={len(psucc_targets)}, psucc_adm={init_psucc_admitted}, full_adm={init_full_admitted}",
    )

    # Verify exact step-175 Gate(21) bifurcation
    exp_bif = expected["bifurcation"]
    bif_idx = exp_bif["first_divergence_step"]
    require(
        len(rel_steps) > bif_idx and len(fz_steps) > bif_idx,
        f"steps.jsonl shorter than bifurcation step {bif_idx}",
    )
    step_compare_keys = (
        "cursor_before",
        "cursor_after",
        "coordinate",
        "target_positions",
        "candidate",
        "loss_before",
        "loss_after",
        "model_epoch",
        "applied",
        "any_candidate_cost_differs_from_identity",
    )
    for s_idx in range(bif_idx):
        sr = rel_steps[s_idx]
        sf = fz_steps[s_idx]
        for k in step_compare_keys:
            require(
                sr.get(k) == sf.get(k),
                f"pre-bifurcation step {s_idx} field {k!r} differs between Relation and FrozenZero: {sr.get(k)!r} vs {sf.get(k)!r}",
            )

    prefix_vec = [
        st
        for st in rel_steps[:bif_idx]
        if st.get("applied") is True and st.get("coordinate", {}).get("kind") == "vector_block"
    ]
    prefix_b03 = [st for st in prefix_vec if (st["coordinate"]["index"] % 4) in (0, 3)]
    prefix_moved_toks = {st["coordinate"]["index"] // 4 for st in prefix_b03}
    prefix_moved_psucc_targets = prefix_moved_toks & psucc_targets
    prefix_moved_psucc_rows = sum(1 for r in psucc_rows if r["target_token"] in prefix_moved_toks)
    prefix_comb = [
        st
        for st in rel_steps[:bif_idx]
        if st.get("applied") is True
        and st.get("coordinate", {}).get("kind") == "gate"
        and st["coordinate"].get("index", 99) < 20
    ]
    prefix_rel = [
        st
        for st in rel_steps[:bif_idx]
        if st.get("applied") is True
        and st.get("coordinate", {}).get("kind") == "gate"
        and st["coordinate"].get("index", 0) >= 20
    ]
    require(
        len(prefix_vec) == exp_bif["prefix_accepted_vector_writes"]
        and len(prefix_b03) == exp_bif["prefix_accepted_block_0_or_3_writes"]
        and len(prefix_moved_toks) == exp_bif["prefix_moved_tokens"]
        and len(prefix_moved_psucc_targets) == exp_bif["prefix_moved_psucc_targets"]
        and prefix_moved_psucc_rows == exp_bif["prefix_moved_psucc_rows"]
        and len(prefix_comb) == exp_bif["prefix_accepted_gate_combine_writes"]
        and len(prefix_rel) == exp_bif["prefix_accepted_gate_relation_writes"],
        f"pre-bifurcation 0..{bif_idx - 1} summary mismatch",
    )

    sr_175 = rel_steps[bif_idx]
    sf_175 = fz_steps[bif_idx]
    require(
        sr_175.get("coordinate") == exp_bif["divergence_coordinate"]
        and sf_175.get("coordinate") == exp_bif["divergence_coordinate"]
        and sr_175.get("candidate") == exp_bif["divergence_candidate"]
        and sf_175.get("candidate") == exp_bif["divergence_candidate"]
        and sr_175.get("loss_before") == exp_bif["divergence_loss_before"]
        and sf_175.get("loss_before") == exp_bif["divergence_loss_before"]
        and sr_175.get("loss_after") == exp_bif["divergence_loss_after"]
        and sf_175.get("loss_after") == exp_bif["divergence_loss_after"]
        and sr_175.get("applied") is exp_bif["relation_applied"]
        and sf_175.get("applied") is exp_bif["frozen_zero_applied"]
        and sf_175.get("apply_refused_by_policy") is exp_bif["frozen_zero_refused_by_policy"],
        f"step-{bif_idx} Gate(21) bifurcation mismatch: rel={sr_175}, fz={sf_175}",
    )

    # Verify cumulative VectorBlock write counts and cell-moved token sets
    def summarize_vector_steps(steps_slice: list[dict[str, Any]], expected_moved_toks: set[int]) -> dict[str, Any]:
        vec = [
            st
            for st in steps_slice
            if st.get("applied") is True and st.get("coordinate", {}).get("kind") == "vector_block"
        ]
        by_block = [
            sum(1 for st in vec if st["coordinate"]["index"] % 4 == blk) for blk in range(4)
        ]
        b03 = [st for st in vec if (st["coordinate"]["index"] % 4) in (0, 3)]
        moved_toks = {st["coordinate"]["index"] // 4 for st in b03}
        require(
            moved_toks == expected_moved_toks,
            f"steps.jsonl moved tokens {sorted(moved_toks)} != checkpoint cell diff {sorted(expected_moved_toks)}",
        )
        moved_pt = moved_toks & psucc_targets
        moved_pr = sum(1 for r in psucc_rows if r["target_token"] in moved_toks)
        return {
            "total": len(vec),
            "by_block": by_block,
            "block_0_or_3_writes": len(b03),
            "moved_tokens": len(moved_toks),
            "moved_psucc_targets": len(moved_pt),
            "moved_psucc_rows": moved_pr,
            "_moved_tokens_set": moved_toks,
        }

    rel_q1_vec = summarize_vector_steps(rel_steps[:q1_batch_key], rel_traj["q1_moved_tokens"])
    fz_q1_vec = summarize_vector_steps(fz_steps[:q1_batch_key], fz_traj["q1_moved_tokens"])
    fz_fin_vec = summarize_vector_steps(fz_steps[:final_batch_key], fz_traj["end_moved_tokens"])

    rel_moved_q1 = rel_q1_vec.pop("_moved_tokens_set")
    fz_moved_q1 = fz_q1_vec.pop("_moved_tokens_set")
    fz_moved_fin = fz_fin_vec.pop("_moved_tokens_set")

    shared_q1_toks = rel_moved_q1 & fz_moved_q1
    rel_only_q1_toks = rel_moved_q1 - fz_moved_q1
    fz_only_q1_toks = fz_moved_q1 - rel_moved_q1
    q1_overlap = {
        "shared": {
            "tokens": len(shared_q1_toks),
            "psucc_targets": len(shared_q1_toks & psucc_targets),
            "psucc_rows": sum(1 for r in psucc_rows if r["target_token"] in shared_q1_toks),
        },
        "relation_only": {
            "tokens": len(rel_only_q1_toks),
            "psucc_targets": len(rel_only_q1_toks & psucc_targets),
            "psucc_rows": sum(1 for r in psucc_rows if r["target_token"] in rel_only_q1_toks),
        },
        "frozen_zero_only": {
            "tokens": len(fz_only_q1_toks),
            "psucc_targets": len(fz_only_q1_toks & psucc_targets),
            "psucc_rows": sum(1 for r in psucc_rows if r["target_token"] in fz_only_q1_toks),
        },
    }

    exp_vw = expected["vector_writes"]
    require(
        rel_q1_vec == exp_vw["relation_q1"]
        and fz_q1_vec == exp_vw["frozen_zero_q1"]
        and fz_fin_vec == exp_vw["frozen_zero_final"]
        and q1_overlap == exp_vw["q1_moved_overlap"],
        f"vector_writes summary mismatch: rel_q1={rel_q1_vec}, fz_q1={fz_q1_vec}, fz_fin={fz_fin_vec}, overlap={q1_overlap}",
    )

    # Compute and verify two-sided P_succ admission decomposition
    def compute_two_sided(end_rows: list[dict[str, Any]], moved_toks: set[int]) -> dict[str, Any]:
        out_sub: dict[str, Any] = {}
        for sub_name, pred in (
            ("target_cell_unchanged", lambda t: t not in moved_toks),
            ("target_cell_moved", lambda t: t in moved_toks),
            ("total", lambda t: True),
        ):
            t_set: set[int] = set()
            rows_cnt = init_adm = end_adm = kept = lost = gained = never = 0
            for r0, r1 in zip(init_rows_rel, end_rows):
                if not r0["is_psucc"] or not pred(r0["target_token"]):
                    continue
                t_set.add(r0["target_token"])
                rows_cnt += 1
                a0 = bool(r0["target_admitted"])
                a1 = bool(r1["target_admitted"])
                if a0:
                    init_adm += 1
                if a1:
                    end_adm += 1
                if a0 and a1:
                    kept += 1
                elif a0 and not a1:
                    lost += 1
                elif not a0 and a1:
                    gained += 1
                else:
                    never += 1
            net = end_adm - init_adm
            require(
                kept + lost == init_adm
                and kept + gained == end_adm
                and kept + lost + gained + never == rows_cnt
                and gained - lost == net,
                f"two-sided conservation failure in {sub_name}",
            )
            out_sub[sub_name] = {
                "targets": len(t_set),
                "rows": rows_cnt,
                "init_admitted": init_adm,
                "end_admitted": end_adm,
                "kept": kept,
                "lost": lost,
                "gained": gained,
                "never": never,
                "net_change": net,
            }
        for k in ("targets", "rows", "init_admitted", "end_admitted", "kept", "lost", "gained", "never", "net_change"):
            require(
                out_sub["target_cell_unchanged"][k] + out_sub["target_cell_moved"][k]
                == out_sub["total"][k],
                f"two-sided additive partition failure on {k}",
            )
        return out_sub

    two_sided = {
        "relation_q1": compute_two_sided(rel_q1_rows, rel_moved_q1),
        "frozen_zero_q1": compute_two_sided(fz_q1_rows, fz_moved_q1),
        "frozen_zero_final": compute_two_sided(fz_final_rows, fz_moved_fin),
    }
    require(
        two_sided == expected["two_sided_decomposition"],
        f"two-sided P_succ decomposition mismatch: actual={two_sided} vs expected={expected['two_sided_decomposition']}",
    )

    # Compute and verify matched 4-way cross-arm Q1 cohort decomposition
    cohorts_pred = {
        "unchanged_both": lambda t: (t not in rel_moved_q1) and (t not in fz_moved_q1),
        "moved_both": lambda t: (t in rel_moved_q1) and (t in fz_moved_q1),
        "moved_rel_only": lambda t: (t in rel_moved_q1) and (t not in fz_moved_q1),
        "moved_fz_only": lambda t: (t not in rel_moved_q1) and (t in fz_moved_q1),
    }
    four_way: dict[str, Any] = {}
    for c_name, pred in cohorts_pred.items():
        t_set: set[int] = set()
        rows_cnt = init_adm = rel_adm = fz_adm = 0
        r_kept = r_lost = r_gain = r_nev = 0
        f_kept = f_lost = f_gain = f_nev = 0
        both_adm = rel_only_adm = fz_only_adm = neither_adm = 0
        for r0, rr, rf in zip(init_rows_rel, rel_q1_rows, fz_q1_rows):
            if not r0["is_psucc"] or not pred(r0["target_token"]):
                continue
            t_set.add(r0["target_token"])
            rows_cnt += 1
            a0 = bool(r0["target_admitted"])
            ar = bool(rr["target_admitted"])
            af = bool(rf["target_admitted"])
            if a0:
                init_adm += 1
            if ar:
                rel_adm += 1
            if af:
                fz_adm += 1
            if a0 and ar:
                r_kept += 1
            elif a0 and not ar:
                r_lost += 1
            elif not a0 and ar:
                r_gain += 1
            else:
                r_nev += 1
            if a0 and af:
                f_kept += 1
            elif a0 and not af:
                f_lost += 1
            elif not a0 and af:
                f_gain += 1
            else:
                f_nev += 1
            if ar and af:
                both_adm += 1
            elif ar and not af:
                rel_only_adm += 1
            elif not ar and af:
                fz_only_adm += 1
            else:
                neither_adm += 1
        require(
            both_adm + rel_only_adm == rel_adm
            and both_adm + fz_only_adm == fz_adm
            and both_adm + rel_only_adm + fz_only_adm + neither_adm == rows_cnt
            and rel_only_adm - fz_only_adm == rel_adm - fz_adm,
            f"4-way cohort paired conservation failure in {c_name}",
        )
        four_way[c_name] = {
            "targets": len(t_set),
            "rows": rows_cnt,
            "init_admitted": init_adm,
            "rel_q1_admitted": rel_adm,
            "rel_net": rel_adm - init_adm,
            "rel_kept": r_kept,
            "rel_lost": r_lost,
            "rel_gained": r_gain,
            "rel_never": r_nev,
            "fz_q1_admitted": fz_adm,
            "fz_net": fz_adm - init_adm,
            "fz_kept": f_kept,
            "fz_lost": f_lost,
            "fz_gained": f_gain,
            "fz_never": f_nev,
            "paired_rel_minus_fz": rel_adm - fz_adm,
            "both_admitted": both_adm,
            "rel_only_admitted": rel_only_adm,
            "fz_only_admitted": fz_only_adm,
            "neither_admitted": neither_adm,
        }

    require(
        four_way == expected["four_way_q1_cohorts"],
        f"four_way_q1_cohorts mismatch: actual={four_way} vs expected={expected['four_way_q1_cohorts']}",
    )

    # Verify cross-identities between 4-way cohorts and two-sided Q1 decompositions
    require(
        four_way["unchanged_both"]["rows"] + four_way["moved_fz_only"]["rows"]
        == two_sided["relation_q1"]["target_cell_unchanged"]["rows"]
        and four_way["moved_both"]["rows"] + four_way["moved_rel_only"]["rows"]
        == two_sided["relation_q1"]["target_cell_moved"]["rows"]
        and four_way["unchanged_both"]["rows"] + four_way["moved_rel_only"]["rows"]
        == two_sided["frozen_zero_q1"]["target_cell_unchanged"]["rows"]
        and four_way["moved_both"]["rows"] + four_way["moved_fz_only"]["rows"]
        == two_sided["frozen_zero_q1"]["target_cell_moved"]["rows"]
        and sum(c["paired_rel_minus_fz"] for c in four_way.values())
        == two_sided["relation_q1"]["total"]["end_admitted"]
        - two_sided["frozen_zero_q1"]["total"]["end_admitted"],
        "4-way cohort vs two-sided Q1 cross-identity mismatch",
    )

    return {
        "status": "PASS",
        "manifest_sha256": bundle["manifest_sha256"],
        "manifest_lines": len(manifest_entries),
        "occupancy": {
            "all_checkpoints_occupied_256_of_256": True,
            "off_orbit_roots": 0,
            "cp0_token_cells_u16_sha256": expected["cp0_token_cells_u16_sha256"],
            "relation_transient_excursions": rel_traj["transient_excursions"],
            "frozen_zero_transient_excursions": fz_traj["transient_excursions"],
            "population_bounds": {
                "cp0": rel_traj["pop_by_batch"][0],
                "relation_q1": rel_traj["pop_by_batch"][q1_batch_key],
                "frozen_zero_q1": fz_traj["pop_by_batch"][q1_batch_key],
                "frozen_zero_final": fz_traj["pop_by_batch"][final_batch_key],
            },
        },
        "slot_bijection": {
            "relation_intervals": rel_traj["intervals_verified"],
            "frozen_zero_intervals": fz_traj["intervals_verified"],
            "total_intervals": rel_traj["intervals_verified"] + fz_traj["intervals_verified"],
            "total_mismatches": rel_traj["interval_mismatches"] + fz_traj["interval_mismatches"],
            "vector_writes": {
                "relation_q1": rel_q1_vec,
                "frozen_zero_q1": fz_q1_vec,
                "frozen_zero_final": fz_fin_vec,
                "q1_moved_overlap": q1_overlap,
            },
        },
        "bifurcation": exp_bif,
        "two_sided_decomposition": two_sided,
        "four_way_q1_cohorts": four_way,
    }


def build_synthetic_census_bundle(repo_root: Path | None = None) -> dict[str, Any]:
    """Construct a deterministic synthetic census, steps.jsonl, and B3 row bundle for self-tests 41..48."""
    repo_root = repo_root or Path(__file__).resolve().parents[3]
    negb_doc = read_json_checked(
        repo_root / "docs" / "labs" / "a8-dose-runbook-2026-10-19" / "evidence" / "neg-b-a8-nonroot-table.json"
    )
    a8_words = EXPECTED_A8_TOKEN0_BLOCK0 + negb_doc["table"][8:]
    base_root_ids, _, _, _, _ = _extract_checkpoint_root_ids_and_cells(a8_words, "synthetic_cp0")

    # Group tokens by their cp0 cell (256 cells x 16 tokens) so we can move specific tokens
    # while preserving 256/256 cell occupancy and hitting exact target/row counts.
    cell_to_tokens: dict[int, list[int]] = {c: [] for c in range(256)}
    for tok in range(4096):
        c = base_root_ids[tok * 4] * 16 + base_root_ids[tok * 4 + 3]
        cell_to_tokens[c].append(tok)

    # Pick 448 distinct P_succ target tokens (at most 2 per cell across cells 0..223)
    # and at most 1 moved non-P_succ token per cell so every cell retains >= 13 tokens.
    psucc_target_list: list[int] = []
    non_psucc_moved_pool: list[int] = []
    unmoved_non_pt: list[int] = []
    for c in range(256):
        toks = cell_to_tokens[c]
        if c < 224:
            psucc_target_list.extend(toks[:2])
            non_psucc_moved_pool.append(toks[2])
            unmoved_non_pt.extend(toks[3:])
        else:
            non_psucc_moved_pool.append(toks[0])
            unmoved_non_pt.extend(toks[1:])
    assert len(psucc_target_list) == 448

    # Partition the 448 P_succ targets into the 4 cross-arm Q1 cohorts:
    # - moved_both: 52 targets (first 6 are moved in prefix 0..174)
    # - moved_rel_only: 11 targets
    # - moved_fz_only: 19 targets
    # - unchanged_both: 366 targets (first 18 move in FrozenZero post-Q1 so final has 71+18=89 moved P_succ targets)
    moved_both_pt = psucc_target_list[:52]
    moved_rel_only_pt = psucc_target_list[52:63]
    moved_fz_only_pt = psucc_target_list[63:82]
    unchanged_both_pt = psucc_target_list[82:448]
    fz_post_q1_pt = unchanged_both_pt[:18]

    # Non-P_succ moved tokens to hit exact total moved token counts:
    # - prefix 0..174: 6 P_succ targets + 4 non-P_succ = 10 moved tokens
    # - shared Q1: 52 P_succ + 13 non-P_succ = 65 tokens
    # - rel_only Q1: 11 P_succ + 4 non-P_succ = 15 tokens (total Rel Q1 moved = 80)
    # - fz_only Q1: 19 P_succ + 7 non-P_succ = 26 tokens (total FZ Q1 moved = 91)
    # - fz post-Q1: 18 P_succ + 21 non-P_succ = 39 tokens (total FZ Final moved = 130)
    shared_non_pt = non_psucc_moved_pool[:13]
    rel_only_non_pt = non_psucc_moved_pool[13:17]
    fz_only_non_pt = non_psucc_moved_pool[17:24]
    fz_post_non_pt = non_psucc_moved_pool[24:45]

    prefix_b03_tokens = moved_both_pt[:6] + shared_non_pt[:4]  # 10 tokens (6 P_succ targets)
    post_prefix_shared_tokens = moved_both_pt[6:] + shared_non_pt[4:]  # 55 tokens
    rel_only_tokens = moved_rel_only_pt + rel_only_non_pt  # 15 tokens
    fz_only_tokens = moved_fz_only_pt + fz_only_non_pt  # 26 tokens
    fz_post_tokens = fz_post_q1_pt + fz_post_non_pt  # 39 tokens

    # Build step lists:
    # Relation: 192 steps (checkpoints at 0, 175, 192)
    # FrozenZero: 256 steps (checkpoints at 0, 175, 192, 256)
    def make_step(
        step_idx: int,
        acc_before: int,
        epoch_before: int,
        kind: str,
        coord_idx: int,
        candidate: int,
        applied: bool,
        loss_before: int = 60000,
        loss_after: int = 59500,
        refused_by_policy: bool = False,
    ) -> dict[str, Any]:
        acc_after = acc_before + (1 if applied else 0)
        epoch_after = epoch_before + (1 if applied else 0)
        st = {
            "cursor_before": {
                "completed_batches": step_idx,
                "accepted_updates": acc_before,
                "corpus_pass": 0,
                "next_target": step_idx * 64,
                "next_coordinate": step_idx,
            },
            "cursor_after": {
                "completed_batches": step_idx + 1,
                "accepted_updates": acc_after,
                "corpus_pass": 0,
                "next_target": (step_idx + 1) * 64,
                "next_coordinate": step_idx + 1,
            },
            "coordinate": {"kind": kind, "index": coord_idx},
            "target_positions": [19, 20],
            "candidate": candidate,
            "loss_before": loss_before,
            "loss_after": loss_after if (applied or refused_by_policy) else loss_before,
            "model_epoch": epoch_after,
            "applied": applied,
            "any_candidate_cost_differs_from_identity": True,
        }
        if refused_by_policy:
            st["apply_refused_by_policy"] = True
        return st

    # Prefix 0..174: 28 vector writes (14 on block 0/3 over 10 tokens: 10 on block 0 + 4 on block 3; 7 on block 1; 7 on block 2) + 7 gate_combine writes
    prefix_ops: list[tuple[str, int, int, bool]] = []
    for t in prefix_b03_tokens:
        prefix_ops.append(("vector_block", t * 4 + 0, 1, True))
    for t in prefix_b03_tokens[:4]:
        prefix_ops.append(("vector_block", t * 4 + 3, 1, True))
    for i in range(7):
        prefix_ops.append(("vector_block", unmoved_non_pt[i] * 4 + 1, 1, True))
        prefix_ops.append(("vector_block", unmoved_non_pt[i] * 4 + 2, 1, True))
    for i in range(7):
        prefix_ops.append(("gate", i, 2, True))
    while len(prefix_ops) < 175:
        prefix_ops.append(("vector_block", unmoved_non_pt[50 + (len(prefix_ops) % 100)] * 4 + 1, 0, False))

    # Interval 175..399 (Q1 at 400) and 400..511 (Final at 512):
    # Matches all exact moved-token counts (80, 91, 130, 65, 15, 26) and exact
    # 192 [58,49,52,33] / 207 [64,49,55,39] / 301 [82,75,80,64] vector write counts.
    rel_q1_slots: list[int] = []
    # Relation Q1 needs [58, 49, 52, 33] across blocks 0..3 (total 192; b0+b3 = 91 across 80 tokens):
    # Prefix already has 10 b0 + 4 b3 (over 10 tokens) + 7 b1 + 7 b2.
    # Add 48 b0 + 22 b3 over the remaining 70 Relation moved tokens (55 shared + 15 rel_only) + 7 extra b3 on already-moved tokens:
    rem_rel_moved = post_prefix_shared_tokens + rel_only_tokens  # 70 tokens
    for t in rem_rel_moved[:48]:
        rel_q1_slots.append(t * 4 + 0)
    for t in rem_rel_moved[48:70]:
        rel_q1_slots.append(t * 4 + 3)
    for t in rem_rel_moved[:7]:
        rel_q1_slots.append(t * 4 + 3)
    for i in range(42):
        rel_q1_slots.append(unmoved_non_pt[10 + i] * 4 + 1)
    for i in range(45):
        rel_q1_slots.append(unmoved_non_pt[10 + i] * 4 + 2)

    # FrozenZero Q1 needs [64, 49, 55, 39] across blocks 0..3 (total 207; b0+b3 = 103 across 91 tokens):
    # Prefix already has 10 b0 + 4 b3 (over 10 tokens) + 7 b1 + 7 b2.
    # Remaining 81 FZ Q1 moved tokens = 55 post_prefix_shared + 26 fz_only:
    rem_fz_moved = post_prefix_shared_tokens + fz_only_tokens  # 81 tokens
    fz_q1_slots: list[int] = []
    for t in rem_fz_moved[:54]:
        fz_q1_slots.append(t * 4 + 0)
    for t in rem_fz_moved[54:81]:
        fz_q1_slots.append(t * 4 + 3)
    for t in rem_fz_moved[:8]:
        fz_q1_slots.append(t * 4 + 3)
    for i in range(42):
        fz_q1_slots.append(unmoved_non_pt[60 + i] * 4 + 1)
    for i in range(48):
        fz_q1_slots.append(unmoved_non_pt[60 + i] * 4 + 2)

    # FrozenZero post-Q1 (to Final) needs [82-64=18, 75-49=26, 80-55=25, 64-39=25] = 94 writes (43 on b0/b3 across 39 new tokens + 4 existing):
    fz_post_slots: list[int] = []
    for t in fz_post_tokens[:18]:
        fz_post_slots.append(t * 4 + 0)
    for t in fz_post_tokens[18:39]:
        fz_post_slots.append(t * 4 + 3)
    for t in fz_post_tokens[:4]:
        fz_post_slots.append(t * 4 + 3)
    for i in range(26):
        fz_post_slots.append(unmoved_non_pt[120 + i] * 4 + 1)
    for i in range(25):
        fz_post_slots.append(unmoved_non_pt[120 + i] * 4 + 2)

    q1_syn_batch = 400
    fin_syn_batch = 512

    def build_arm_steps_and_cps(is_fz: bool) -> tuple[list[dict[str, Any]], list[dict[str, Any]]]:
        steps: list[dict[str, Any]] = []
        table = list(a8_words)
        gates = list(EXPECTED_ACTIVE_LEAF_GATES)
        acc = 0
        epoch = 0

        def snap(batch_num: int, name: str, sha_override: str) -> dict[str, Any]:
            return {
                "name": name,
                "manifest_key": f"./claude/{'a8-dose-frozen-zero-seed1' if is_fz else 'a8-dose-relation-seed1'}/{name}",
                "sha256": sha_override,
                "bytes": 6967500,
                "completed_batches": batch_num,
                "accepted_updates": acc,
                "epoch": epoch,
                "gates": list(gates),
                "routes": list(EXPECTED_MODEL_ROUTES),
                "table": list(table),
            }

        def apply_slot(slot: int) -> None:
            old_rid = ROOT_TUPLE_TO_ID[tuple(table[slot * 8 : (slot + 1) * 8])]
            table[slot * 8 : (slot + 1) * 8] = list(ROOTS[(old_rid + 1) % 16])

        cps: list[dict[str, Any]] = [
            snap(
                0,
                "checkpoint-00000.json",
                EXPECTED_VAST_56670997_FZ_CP00000_SHA256 if is_fz else EXPECTED_VAST_56670997_REL_CP00000_SHA256,
            )
        ]

        # 0..174
        for s_idx, (kind, c_idx, cand, app) in enumerate(prefix_ops):
            steps.append(make_step(s_idx, acc, epoch, kind, c_idx, cand, app))
            if app:
                acc += 1
                epoch += 1
                if kind == "vector_block":
                    apply_slot(c_idx)
                else:
                    gates[c_idx] = -1

        cps.append(snap(175, "checkpoint-00175.json", ("a" if not is_fz else "b") * 64))

        # Step 175: Gate(21) bifurcation
        if not is_fz:
            steps.append(make_step(175, acc, epoch, "gate", 21, 2, True, 55680, 55232))
            acc += 1
            epoch += 1
            gates[21] = -1
        else:
            steps.append(make_step(175, acc, epoch, "gate", 21, 2, False, 55680, 55232, refused_by_policy=True))

        # 176..q1_syn_batch-1
        q1_slots = fz_q1_slots if is_fz else rel_q1_slots
        for offset in range(q1_syn_batch - 176):
            s_idx = 176 + offset
            if offset < len(q1_slots):
                slot = q1_slots[offset]
                steps.append(make_step(s_idx, acc, epoch, "vector_block", slot, 1, True))
                acc += 1
                epoch += 1
                apply_slot(slot)
            else:
                steps.append(make_step(s_idx, acc, epoch, "vector_block", 0, 0, False))

        cps.append(
            snap(
                q1_syn_batch,
                "checkpoint-04096.json",
                EXPECTED_VAST_56670997_FZ_CP04096_SHA256 if is_fz else EXPECTED_VAST_56670997_REL_CP04096_SHA256,
            )
        )

        if is_fz:
            for offset in range(fin_syn_batch - q1_syn_batch):
                s_idx = q1_syn_batch + offset
                if offset < len(fz_post_slots):
                    slot = fz_post_slots[offset]
                    steps.append(make_step(s_idx, acc, epoch, "vector_block", slot, 1, True))
                    acc += 1
                    epoch += 1
                    apply_slot(slot)
                else:
                    steps.append(make_step(s_idx, acc, epoch, "vector_block", 0, 0, False))
            cps.append(snap(fin_syn_batch, "checkpoint-16404.json", EXPECTED_VAST_56670997_FZ_CP16404_SHA256))

        return steps, cps

    rel_steps, rel_cps = build_arm_steps_and_cps(False)
    fz_steps, fz_cps = build_arm_steps_and_cps(True)

    # Construct the 14,287 DEV B3 rows (3,847 P_succ rows + 10,440 complement rows)
    # matching the exact 4-way Q1 cohorts, 302 prefix rows, and FrozenZero Final two-sided decomposition!
    # Cohort row counts:
    # - moved_both (52 targets, 1,150 rows): first 6 targets (prefix) get 302 rows; remaining 46 targets get 848 rows
    # - moved_rel_only (11 targets, 311 rows)
    # - moved_fz_only (19 targets, 219 rows)
    # - unchanged_both (366 targets, 2,167 rows): first 18 targets (fz_post_q1_pt) get 158 rows (so FZ Final moved has 1369 + 158 = 1527 rows!); remaining 348 targets get 2,009 rows
    def distribute_rows(targets: list[int], total_rows: int) -> list[int]:
        base_c = total_rows // len(targets)
        rem_c = total_rows % len(targets)
        out_t: list[int] = []
        for idx, t in enumerate(targets):
            cnt = base_c + (1 if idx < rem_c else 0)
            out_t.extend([t] * cnt)
        assert len(out_t) == total_rows
        return out_t

    mb_tokens_per_row = distribute_rows(moved_both_pt[:6], 302) + distribute_rows(moved_both_pt[6:], 848)
    mro_tokens_per_row = distribute_rows(moved_rel_only_pt, 311)
    mfo_tokens_per_row = distribute_rows(moved_fz_only_pt, 219)
    ub_post_tokens_per_row = distribute_rows(fz_post_q1_pt, 158)
    ub_stay_tokens_per_row = distribute_rows(unchanged_both_pt[18:], 2009)

    def make_cohort_states(
        rows_cnt: int,
        r_kept: int,
        r_lost: int,
        r_gain: int,
        f_kept: int,
        f_lost: int,
        f_gain: int,
        both_adm: int,
    ) -> list[tuple[bool, bool, bool]]:
        # Construct (init, rel_q1, fz_q1) tuples satisfying all marginals and both_admitted
        init_adm = r_kept + r_lost
        assert f_kept + f_lost == init_adm
        init_0 = rows_cnt - init_adm
        # On init=1 (size init_adm): rel_q1=1 on r_kept, fz_q1=1 on f_kept. Overlap on init=1:
        ov1 = min(r_kept, f_kept, both_adm)
        # On init=0 (size init_0): rel_q1=1 on r_gain, fz_q1=1 on f_gain. Need overlap ov0 = both_adm - ov1:
        ov0 = both_adm - ov1
        if ov0 > min(r_gain, f_gain):
            ov0 = min(r_gain, f_gain)
            ov1 = both_adm - ov0
        states: list[tuple[bool, bool, bool]] = []
        # init=1 rows
        for i in range(init_adm):
            ar = i < r_kept
            af = (i < ov1) or (r_kept <= i < r_kept + (f_kept - ov1))
            states.append((True, ar, af))
        # init=0 rows
        for i in range(init_0):
            ar = i < r_gain
            af = (i < ov0) or (r_gain <= i < r_gain + (f_gain - ov0))
            states.append((False, ar, af))
        assert len(states) == rows_cnt
        return states

    ub_states = make_cohort_states(2167, 260, 573, 106, 404, 429, 133, 189)
    mb_states = make_cohort_states(1150, 34, 304, 82, 62, 276, 160, 42)
    mro_states = make_cohort_states(311, 10, 69, 13, 33, 46, 7, 7)
    mfo_states = make_cohort_states(219, 35, 66, 11, 40, 61, 18, 16)

    # Reorder ub_states so the first 158 rows (fz_post_q1_pt) have exact init_admitted = 488 - 439 = 49
    ub_init1 = [s for s in ub_states if s[0]]
    ub_init0 = [s for s in ub_states if not s[0]]
    ub_states_ordered = (
        ub_init1[:49]
        + ub_init0[: 158 - 49]
        + ub_init1[49:]
        + ub_init0[158 - 49 :]
    )
    ub_tokens_per_row = ub_post_tokens_per_row + ub_stay_tokens_per_row

    # Assign fz_final (a_fin) on each row:
    # - FZ Final moved rows = mb (1,150) + mfo (219) + ub_post (158) = 1,527 rows (init_adm = 338 + 101 + 49 = 488):
    #   needs kept=89, lost=399, gained=168, never=871
    # - FZ Final unchanged rows = ub_stay (2,009) + mro (311) = 2,320 rows (init_adm = (833-49) + 79 = 863):
    #   needs kept=385, lost=478, gained=155, never=1302
    all_psucc_spec: list[tuple[int, bool, bool, bool, bool]] = []
    moved_fin_pool = list(zip(mb_tokens_per_row, mb_states)) + list(zip(mfo_tokens_per_row, mfo_states)) + list(zip(ub_post_tokens_per_row, ub_states_ordered[:158]))
    unch_fin_pool = list(zip(ub_stay_tokens_per_row, ub_states_ordered[158:])) + list(zip(mro_tokens_per_row, mro_states))

    def assign_final(
        pool: list[tuple[int, tuple[bool, bool, bool]]], kept_target: int, gain_target: int
    ) -> None:
        k_cnt = g_cnt = 0
        for tok, (a0, ar, af) in pool:
            if a0:
                afin = k_cnt < kept_target
                if afin:
                    k_cnt += 1
            else:
                afin = g_cnt < gain_target
                if afin:
                    g_cnt += 1
            all_psucc_spec.append((tok, a0, ar, af, afin))
        assert k_cnt == kept_target and g_cnt == gain_target

    assign_final(moved_fin_pool, 89, 168)
    assign_final(unch_fin_pool, 385, 155)
    assert len(all_psucc_spec) == CANONICAL_DEV_PSUCC_ROWS

    init_rows: list[dict[str, Any]] = []
    rel_q1_rows: list[dict[str, Any]] = []
    fz_q1_rows: list[dict[str, Any]] = []
    fz_fin_rows: list[dict[str, Any]] = []

    for idx, (tok, a0, ar, af, afin) in enumerate(all_psucc_spec):
        pos = idx + 10
        init_rows.append({"row_ordinal": idx, "target_position": pos, "target_token": tok, "is_psucc": True, "target_admitted": a0})
        rel_q1_rows.append({"row_ordinal": idx, "target_position": pos, "target_token": tok, "is_psucc": True, "target_admitted": ar})
        fz_q1_rows.append({"row_ordinal": idx, "target_position": pos, "target_token": tok, "is_psucc": True, "target_admitted": af})
        fz_fin_rows.append({"row_ordinal": idx, "target_position": pos, "target_token": tok, "is_psucc": True, "target_admitted": afin})

    for offset in range(CANONICAL_DEV_COMP_ROWS):
        idx = CANONICAL_DEV_PSUCC_ROWS + offset
        pos = idx + 10
        tok = unmoved_non_pt[0]
        a0 = offset < EXPECTED_A8_CP0_COMP_ADMITTED
        ar = offset < 960
        af = offset < 1662
        afin = offset < 1220
        init_rows.append({"row_ordinal": idx, "target_position": pos, "target_token": tok, "is_psucc": False, "target_admitted": a0})
        rel_q1_rows.append({"row_ordinal": idx, "target_position": pos, "target_token": tok, "is_psucc": False, "target_admitted": ar})
        fz_q1_rows.append({"row_ordinal": idx, "target_position": pos, "target_token": tok, "is_psucc": False, "target_admitted": af})
        fz_fin_rows.append({"row_ordinal": idx, "target_position": pos, "target_token": tok, "is_psucc": False, "target_admitted": afin})

    # Compute actual synthetic population bounds at cp0, q1_syn_batch, and fin_syn_batch
    _, _, _, rel_q1_min, rel_q1_max = _extract_checkpoint_root_ids_and_cells(rel_cps[-1]["table"], "syn_rel_q1")
    _, _, _, fz_q1_min, fz_q1_max = _extract_checkpoint_root_ids_and_cells(fz_cps[-2]["table"], "syn_fz_q1")
    _, _, _, fz_fin_min, fz_fin_max = _extract_checkpoint_root_ids_and_cells(fz_cps[-1]["table"], "syn_fz_fin")

    syn_expected = copy.deepcopy(EXPECTED_VAST_56670997_CENSUS_EXPECTATIONS)
    syn_expected["relation_checkpoint_count"] = len(rel_cps)
    syn_expected["frozen_zero_checkpoint_count"] = len(fz_cps)
    syn_expected["relation_intervals"] = len(rel_cps) - 1
    syn_expected["frozen_zero_intervals"] = len(fz_cps) - 1
    syn_expected["total_intervals"] = (len(rel_cps) - 1) + (len(fz_cps) - 1)
    syn_expected["relation_steps_lines"] = len(rel_steps)
    syn_expected["frozen_zero_steps_lines"] = len(fz_steps)
    syn_expected["q1_batch"] = q1_syn_batch
    syn_expected["final_batch"] = fin_syn_batch
    syn_expected["population_bounds"] = {
        "cp0": {"occupied_cells": 256, "min_pop": 16, "max_pop": 16},
        "relation_q1": {"occupied_cells": 256, "min_pop": rel_q1_min, "max_pop": rel_q1_max},
        "frozen_zero_q1": {"occupied_cells": 256, "min_pop": fz_q1_min, "max_pop": fz_q1_max},
        "frozen_zero_final": {"occupied_cells": 256, "min_pop": fz_fin_min, "max_pop": fz_fin_max},
    }

    manifest_entries: dict[str, str] = {
        "./claude/a8-dose-gates-20261019/relation/cells/token_cells.u16": EXPECTED_VAST_56670997_CP0_TOKEN_CELLS_U16_SHA256,
        "./claude/a8-dose-gates-20261019/frozen-zero/cells/token_cells.u16": EXPECTED_VAST_56670997_CP0_TOKEN_CELLS_U16_SHA256,
        "./claude/a8-dose-relation-seed1/steps.jsonl": EXPECTED_VAST_56670997_REL_STEPS_JSONL_SHA256,
        "./claude/a8-dose-frozen-zero-seed1/steps.jsonl": EXPECTED_VAST_56670997_FZ_STEPS_JSONL_SHA256,
        "./claude/a8-dose-custody-20261019/state/a8-relation-seed1/q1.json": EXPECTED_VAST_56670997_REL_Q1_JSON_SHA256,
        "./claude/a8-dose-custody-20261019/state/a8-frozen-zero-seed1/q1.json": EXPECTED_VAST_56670997_FZ_Q1_JSON_SHA256,
        "./claude/a8-dose-frozen-zero-seed1/report.json": EXPECTED_VAST_56670997_FZ_REPORT_JSON_SHA256,
        "./claude/a8-dose-relation-seed1/report.json": EXPECTED_VAST_56670997_REL_REPORT_JSON_SHA256,
    }
    for cp in rel_cps + fz_cps:
        manifest_entries[cp["manifest_key"]] = cp["sha256"]
    syn_expected["manifest_lines"] = len(manifest_entries)

    return {
        "manifest_sha256": EXPECTED_VAST_56670997_RUNS_MANIFEST_SHA256,
        "manifest_bytes": EXPECTED_VAST_56670997_RUNS_MANIFEST_BYTES,
        "manifest_entries": manifest_entries,
        "relation_checkpoints": rel_cps,
        "frozen_zero_checkpoints": fz_cps,
        "relation_steps_sha256": EXPECTED_VAST_56670997_REL_STEPS_JSONL_SHA256,
        "relation_steps": rel_steps,
        "frozen_zero_steps_sha256": EXPECTED_VAST_56670997_FZ_STEPS_JSONL_SHA256,
        "frozen_zero_steps": fz_steps,
        "relation_q1_sha256": EXPECTED_VAST_56670997_REL_Q1_JSON_SHA256,
        "relation_q1": {
            "init_b3": {"rows": init_rows},
            "q1_b3": {"rows": rel_q1_rows},
        },
        "frozen_zero_q1_sha256": EXPECTED_VAST_56670997_FZ_Q1_JSON_SHA256,
        "frozen_zero_q1": {
            "init_b3": {"rows": copy.deepcopy(init_rows)},
            "q1_b3": {"rows": fz_q1_rows},
        },
        "frozen_zero_report_sha256": EXPECTED_VAST_56670997_FZ_REPORT_JSON_SHA256,
        "frozen_zero_report": {
            "final_b3": {"rows": fz_fin_rows},
        },
        "relation_report_sha256": EXPECTED_VAST_56670997_REL_REPORT_JSON_SHA256,
        "relation_report": {"status": "Q1_STOP_FOR_DIAGNOSIS"},
        "expected": syn_expected,
    }


_CACHED_TRAIN_PSUCC_TAIL_ANCHOR: dict[tuple[str, str, int, int], dict[str, Any]] = {}


def recompute_train_psucc_tail_anchor_from_uort(
    train_tokens_path: Path,
    train_mask_path: Path,
    start_eligible_index: int = EXPECTED_TRAIN_PSUCC_TAIL_START_ELIGIBLE_INDEX,
    target_rows: int = EXPECTED_TRAIN_PSUCC_TAIL_ANCHOR_ROWS,
) -> dict[str, Any]:
    """Recompute the 3,847-row unvisited-tail TRAIN P_succ anchor CSV directly from pinned UORT files."""
    cache_key = (
        str(train_tokens_path.resolve()),
        str(train_mask_path.resolve()),
        start_eligible_index,
        target_rows,
    )
    if cache_key in _CACHED_TRAIN_PSUCC_TAIL_ANCHOR:
        return copy.deepcopy(_CACHED_TRAIN_PSUCC_TAIL_ANCHOR[cache_key])

    raw_tokens = read_bytes_checked(train_tokens_path, "TRAIN tokens.u16")
    raw_mask = read_bytes_checked(train_mask_path, "TRAIN response_mask.u8")
    tok_sha = hashlib.sha256(raw_tokens).hexdigest()
    mask_sha = hashlib.sha256(raw_mask).hexdigest()
    require(
        tok_sha == EXPECTED_TRAIN_TOKENS_SHA256,
        f"TRAIN tokens.u16 SHA-256 mismatch: {tok_sha} != {EXPECTED_TRAIN_TOKENS_SHA256}",
    )
    require(
        mask_sha == EXPECTED_TRAIN_MASK_SHA256,
        f"TRAIN response_mask.u8 SHA-256 mismatch: {mask_sha} != {EXPECTED_TRAIN_MASK_SHA256}",
    )
    require(
        len(raw_tokens) >= 64 and raw_tokens[:4] == b"UORT",
        "TRAIN tokens.u16 missing canonical 64-byte UORT header",
    )
    version, count, vocab = struct.unpack_from("<IQI", raw_tokens, 4)
    require(
        version == 1
        and count == 7512797
        and vocab == 4096
        and all(b == 0 for b in raw_tokens[20:64])
        and len(raw_tokens) == 64 + count * 2
        and len(raw_mask) == count,
        f"TRAIN UORT header/inventory mismatch: version={version}, count={count}, vocab={vocab}",
    )

    tokens = struct.unpack_from(f"<{count}H", raw_tokens, 64)
    require(tokens[0] == 0, "TRAIN UORT must start with BOS token id 0")

    doc_start = 0
    docs = 0
    elig_idx = 0
    rows: list[tuple[int, int, int, int]] = []
    for pos in range(count):
        tok = tokens[pos]
        if tok == 0:
            doc_start = pos
            docs += 1
        elif pos > 0 and raw_mask[pos] == 1:
            if elig_idx >= start_eligible_index and len(rows) < target_rows:
                w_start = max(doc_start, pos - 192)
                require(w_start < pos, f"target at {pos} has no in-document predecessor")
                query = tokens[pos - 1]
                found = -1
                for p in range(pos - 2, w_start - 1, -1):
                    if tokens[p] == query:
                        found = p
                        break
                if found != -1 and tokens[found + 1] == tok:
                    rows.append((elig_idx, pos, tok, 1))
            elig_idx += 1

    require(
        docs == 60000 and elig_idx == 2425925,
        f"TRAIN documents/eligible_targets mismatch: docs={docs}, eligible={elig_idx}",
    )
    require(
        len(rows) == target_rows,
        f"expected {target_rows} unvisited-tail TRAIN P_succ rows, found {len(rows)}",
    )
    csv_text = "".join(f"{r[0]},{r[1]},{r[2]},{r[3]}\n" for r in rows)
    csv_sha = hashlib.sha256(csv_text.encode("utf-8")).hexdigest()
    if (
        start_eligible_index == EXPECTED_TRAIN_PSUCC_TAIL_START_ELIGIBLE_INDEX
        and target_rows == EXPECTED_TRAIN_PSUCC_TAIL_ANCHOR_ROWS
    ):
        require(
            csv_sha == EXPECTED_TRAIN_PSUCC_TAIL_ANCHOR_CSV_SHA256
            and rows[0] == EXPECTED_TRAIN_PSUCC_TAIL_FIRST_ROW
            and rows[-1] == EXPECTED_TRAIN_PSUCC_TAIL_LAST_ROW,
            f"recomputed TRAIN P_succ anchor mismatch: sha={csv_sha}, first={rows[0]}, last={rows[-1]}",
        )
    res = {
        "status": "PASS",
        "train_tokens_sha256": tok_sha,
        "train_mask_sha256": mask_sha,
        "payload_tokens": count,
        "documents": docs,
        "eligible_targets": elig_idx,
        "start_eligible_index": start_eligible_index,
        "anchor_rows": len(rows),
        "first_row": rows[0],
        "last_row": rows[-1],
        "csv_sha256": csv_sha,
        "csv_text": csv_text,
        "rows": rows,
    }
    _CACHED_TRAIN_PSUCC_TAIL_ANCHOR[cache_key] = copy.deepcopy(res)
    return res


def verify_train_psucc_tail_anchor_csv(
    csv_text_or_rows: str | list[Any],
    train_tokens_path: Path | None = None,
    train_mask_path: Path | None = None,
    require_exact_pinned_sha: bool = True,
) -> dict[str, Any]:
    """Verify the 3,847-line canonical TRAIN P_succ tail anchor CSV and optional UORT binary equivalence."""
    rows: list[tuple[int, int, int, int]] = []
    if isinstance(csv_text_or_rows, str):
        require(
            csv_text_or_rows.endswith("\n"),
            "anchor CSV must be newline-terminated UTF-8",
        )
        lines = csv_text_or_rows.splitlines()
        require(
            len(lines) == EXPECTED_TRAIN_PSUCC_TAIL_ANCHOR_ROWS,
            f"anchor CSV must have {EXPECTED_TRAIN_PSUCC_TAIL_ANCHOR_ROWS} lines, got {len(lines)}",
        )
        for idx, line in enumerate(lines):
            parts = line.split(",")
            require(
                len(parts) == 4 and all(p.isdigit() for p in parts),
                f"anchor CSV line {idx} malformed: {line!r}",
            )
            eo, tp, tt, ip = (int(parts[0]), int(parts[1]), int(parts[2]), int(parts[3]))
            require(
                f"{eo},{tp},{tt},{ip}" == line,
                f"anchor CSV line {idx} non-canonical formatting: {line!r}",
            )
            rows.append((eo, tp, tt, ip))
        csv_text = csv_text_or_rows
    elif isinstance(csv_text_or_rows, list):
        require(
            len(csv_text_or_rows) == EXPECTED_TRAIN_PSUCC_TAIL_ANCHOR_ROWS,
            f"anchor rows must have {EXPECTED_TRAIN_PSUCC_TAIL_ANCHOR_ROWS} entries, got {len(csv_text_or_rows)}",
        )
        for idx, item in enumerate(csv_text_or_rows):
            if isinstance(item, (tuple, list)) and len(item) == 4:
                eo, tp, tt, ip = item[0], item[1], item[2], item[3]
            elif isinstance(item, dict):
                eo = item.get("eligible_ordinal", item.get("row_ordinal"))
                tp = item.get("target_position")
                tt = item.get("target_token")
                ip = 1 if item.get("is_psucc") is True else item.get("is_psucc")
            else:
                raise VerificationError(f"anchor row {idx} invalid type: {item!r}")
            require(
                all(is_u64(v) for v in (eo, tp, tt, ip)),
                f"anchor row {idx} fields must be non-negative integers: {(eo, tp, tt, ip)!r}",
            )
            rows.append((int(eo), int(tp), int(tt), int(ip)))
        csv_text = "".join(f"{r[0]},{r[1]},{r[2]},{r[3]}\n" for r in rows)
    else:
        raise VerificationError(f"unsupported anchor input type: {type(csv_text_or_rows)!r}")

    prev_eo = -1
    prev_tp = -1
    for idx, (eo, tp, tt, ip) in enumerate(rows):
        require(
            eo >= EXPECTED_TRAIN_PSUCC_TAIL_START_ELIGIBLE_INDEX,
            f"anchor row {idx} eligible_ordinal {eo} overlaps training prefix (< {EXPECTED_TRAIN_PSUCC_TAIL_START_ELIGIBLE_INDEX})",
        )
        require(
            eo < 2425925 and 0 < tp < 7512797 and 0 < tt < 4096 and ip == 1,
            f"anchor row {idx} out of TRAIN bounds or is_psucc != 1: {(eo, tp, tt, ip)}",
        )
        require(
            eo > prev_eo and tp > prev_tp,
            f"anchor row {idx} violates strict ascending eligible_ordinal/target_position order: {(eo, tp)} after {(prev_eo, prev_tp)}",
        )
        prev_eo = eo
        prev_tp = tp

    csv_sha = hashlib.sha256(csv_text.encode("utf-8")).hexdigest()
    if require_exact_pinned_sha:
        require(
            csv_sha == EXPECTED_TRAIN_PSUCC_TAIL_ANCHOR_CSV_SHA256,
            f"anchor CSV SHA-256 mismatch: {csv_sha} != {EXPECTED_TRAIN_PSUCC_TAIL_ANCHOR_CSV_SHA256}",
        )
        require(
            rows[0] == EXPECTED_TRAIN_PSUCC_TAIL_FIRST_ROW
            and rows[-1] == EXPECTED_TRAIN_PSUCC_TAIL_LAST_ROW,
            f"anchor first/last row mismatch: first={rows[0]}, last={rows[-1]}",
        )

    uort_recomputed = False
    if train_tokens_path is not None or train_mask_path is not None:
        require(
            train_tokens_path is not None and train_mask_path is not None,
            "train_tokens_path and train_mask_path must be supplied together",
        )
        rec = recompute_train_psucc_tail_anchor_from_uort(train_tokens_path, train_mask_path)
        require(
            rec["csv_sha256"] == csv_sha and rec["rows"] == rows,
            "anchor CSV rows/SHA-256 disagree with UORT binary recomputation",
        )
        uort_recomputed = True

    return {
        "status": "PASS",
        "anchor_rows": len(rows),
        "first_row": rows[0],
        "last_row": rows[-1],
        "training_prefix_overlap_rows": 0,
        "dev_or_heldout_or_fresh_rows_in_anchor": 0,
        "csv_sha256": csv_sha,
        "uort_recomputed": uort_recomputed,
    }


def verify_admission_preserving_sheet_and_anchor(
    repo_root: Path,
    sheet_path: Path | None = None,
    train_tokens_path: Path | None = None,
    train_mask_path: Path | None = None,
) -> dict[str, Any]:
    """Verify `docs/harness/prep/m1-attention-admission-preserving-DRAFT-2026-10-19.md` and the TRAIN tail P_succ anchor."""
    target_sheet = sheet_path or (repo_root / EXPECTED_ADMISSION_PRESERVING_SHEET_REL_PATH)
    raw = read_bytes_checked(target_sheet, "admission-preserving sheet")
    sheet_sha = hashlib.sha256(raw).hexdigest()
    sheet_bytes = len(raw)
    try:
        text = raw.decode("utf-8")
    except UnicodeDecodeError as exc:
        raise VerificationError(f"invalid UTF-8 in {target_sheet}: {exc}") from exc
    sheet_lines = text.count("\n")
    lines = text.splitlines()

    top_verdict: str | None = None
    bottom_verdict: str | None = None
    for line in lines[:12]:
        if line.startswith("- **Status:**"):
            if "TEST FITNESS: PENDING" in line:
                top_verdict = "PENDING"
            elif "TEST FITNESS: FIT" in line:
                top_verdict = "FIT"
            break
    for line in reversed(lines):
        stripped = line.strip()
        if not stripped:
            continue
        if stripped.startswith("TEST FITNESS: PENDING"):
            bottom_verdict = "PENDING"
        elif stripped.startswith("TEST FITNESS: FIT"):
            bottom_verdict = "FIT"
        break

    require(
        top_verdict in ("PENDING", "FIT") and top_verdict == bottom_verdict,
        f"admission-preserving sheet top/bottom TEST FITNESS mismatch: top={top_verdict!r}, bottom={bottom_verdict!r}",
    )
    if top_verdict == "PENDING":
        require(
            sheet_sha == EXPECTED_ADMISSION_PRESERVING_DRAFT_SHEET_SHA256
            and sheet_bytes == EXPECTED_ADMISSION_PRESERVING_DRAFT_SHEET_BYTES
            and sheet_lines == EXPECTED_ADMISSION_PRESERVING_DRAFT_SHEET_LINES,
            f"admission-preserving DRAFT sheet SHA/bytes/lines mismatch: {sheet_sha} ({sheet_bytes} B, {sheet_lines} lines)",
        )
    else:
        require(
            sheet_sha == EXPECTED_ADMISSION_PRESERVING_FIT_SHEET_SHA256
            and sheet_bytes == EXPECTED_ADMISSION_PRESERVING_FIT_SHEET_BYTES
            and sheet_lines == EXPECTED_ADMISSION_PRESERVING_FIT_SHEET_LINES,
            f"admission-preserving FIT sheet SHA/bytes/lines mismatch: {sheet_sha} ({sheet_bytes} B, {sheet_lines} lines)",
        )

    line_set = {line.strip() for line in lines}
    for heading in EXPECTED_ADMISSION_PRESERVING_D22_HEADINGS:
        require(
            heading in line_set,
            f"admission-preserving sheet missing required D22 heading: {heading!r}",
        )

    for required_token in (
        "docs/harness/prep/m1-attention-a8-address-cells-learning-2026-10-19.md",
        EXPECTED_A8_SHEET_SHA256,
        EXPECTED_TRAIN_PSUCC_TAIL_ANCHOR_CSV_SHA256,
        "1,049,856",
        "1,049,857…1,064,112",
        "3,847",
        "margin = 24 rows",
        "anchor baseline must be >= 500",
        "RULE_EFFECTIVELY_NOOP",
        "rejected_intervals / audited_intervals > 0.50",
        "3 consecutive full-interval rejections",
        "258 per arm including baseline",
        "516 evaluations",
        "R = added_gate_cost / ungated_two_arm_dose_cost",
        "R <= 0.15",
        "R > 0.15",
    ):
        require(
            required_token in text,
            f"admission-preserving sheet missing required binding token: {required_token!r}",
        )

    tok_p = train_tokens_path or (
        DEFAULT_LOCAL_MW_BALP_TRAIN_TOKENS_PATH
        if DEFAULT_LOCAL_MW_BALP_TRAIN_TOKENS_PATH.is_file()
        else None
    )
    msk_p = train_mask_path or (
        DEFAULT_LOCAL_MW_BALP_TRAIN_MASK_PATH
        if DEFAULT_LOCAL_MW_BALP_TRAIN_MASK_PATH.is_file()
        else None
    )
    anchor_verification: dict[str, Any] | None = None
    if tok_p is not None and msk_p is not None:
        rec = recompute_train_psucc_tail_anchor_from_uort(tok_p, msk_p)
        anchor_verification = verify_train_psucc_tail_anchor_csv(
            rec["csv_text"],
            train_tokens_path=tok_p,
            train_mask_path=msk_p,
            require_exact_pinned_sha=True,
        )

    return {
        "status": "PASS",
        "sheet_path": str(target_sheet),
        "sheet_sha256": sheet_sha,
        "sheet_bytes": sheet_bytes,
        "sheet_lines": sheet_lines,
        "test_fitness": top_verdict,
        "d22_headings_verified": list(EXPECTED_ADMISSION_PRESERVING_D22_HEADINGS),
        "parent_sheet_sha256": EXPECTED_A8_SHEET_SHA256,
        "anchor_csv_sha256": EXPECTED_TRAIN_PSUCC_TAIL_ANCHOR_CSV_SHA256,
        "anchor_rows": EXPECTED_TRAIN_PSUCC_TAIL_ANCHOR_ROWS,
        "anchor_start_eligible_index": EXPECTED_TRAIN_PSUCC_TAIL_START_ELIGIBLE_INDEX,
        "anchor_first_row": list(EXPECTED_TRAIN_PSUCC_TAIL_FIRST_ROW),
        "anchor_last_row": list(EXPECTED_TRAIN_PSUCC_TAIL_LAST_ROW),
        "anchor_uort_verification": anchor_verification,
    }


def verify_admission_preserving_guard_bundle(bundle: dict[str, Any]) -> dict[str, Any]:
    """Consumer-verify an admission-preserving active interval rejection & no-op counter-metric bundle."""
    faults: list[str] = []
    if not isinstance(bundle, dict):
        return {
            "status": "FAIL",
            "verdict": "INSTRUMENT_UNREACHABLE_D31",
            "mechanism_negative": False,
            "keep": False,
            "faults": ["bundle_not_a_dict"],
        }

    reg = bundle.get("registration")
    if not isinstance(reg, dict):
        faults.append("missing_registration_section")
        reg = {}

    training_started = bool(bundle.get("training_started", True))

    # 1. §7 Registration Checklist & D40/D31 Preconditions
    if reg.get("sheet_sha256") not in (
        EXPECTED_ADMISSION_PRESERVING_FIT_SHEET_SHA256,
        EXPECTED_ADMISSION_PRESERVING_DRAFT_SHEET_SHA256,
    ):
        faults.append("invalid_or_missing_sheet_sha256")
    if training_started and (
        reg.get("sheet_verdict") != "FIT"
        or reg.get("sheet_sha256") != EXPECTED_ADMISSION_PRESERVING_FIT_SHEET_SHA256
    ):
        faults.append("sheet_verdict_not_fit_for_execution")
    if reg.get("parent_sheet_sha256") != EXPECTED_A8_SHEET_SHA256:
        faults.append("parent_sheet_sha256_mismatch")
    if reg.get("a8_table_sha256") != EXPECTED_A8_TABLE_FILE_SHA256:
        faults.append("a8_table_sha256_mismatch")
    if reg.get("train_tokens_sha256") != EXPECTED_TRAIN_TOKENS_SHA256:
        faults.append("train_tokens_sha256_mismatch")
    if reg.get("train_mask_sha256") != EXPECTED_TRAIN_MASK_SHA256:
        faults.append("train_mask_sha256_mismatch")
    if reg.get("anchor_csv_sha256") != EXPECTED_TRAIN_PSUCC_TAIL_ANCHOR_CSV_SHA256:
        faults.append("anchor_csv_sha256_mismatch")
    if reg.get("anchor_rows") != EXPECTED_TRAIN_PSUCC_TAIL_ANCHOR_ROWS:
        faults.append("anchor_rows_count_mismatch")
    first_eo = reg.get("anchor_first_eligible_ordinal")
    last_eo = reg.get("anchor_last_eligible_ordinal")
    if (
        not is_u64(first_eo)
        or first_eo < EXPECTED_TRAIN_PSUCC_TAIL_START_ELIGIBLE_INDEX
        or first_eo != EXPECTED_TRAIN_PSUCC_TAIL_FIRST_ROW[0]
    ):
        faults.append("anchor_first_eligible_ordinal_invalid_or_overlaps_training_prefix")
    if not is_u64(last_eo) or last_eo != EXPECTED_TRAIN_PSUCC_TAIL_LAST_ROW[0]:
        faults.append("anchor_last_eligible_ordinal_mismatch")
    if reg.get("dev_or_heldout_or_fresh_rows_in_anchor") != 0:
        faults.append("anchor_contains_dev_or_heldout_or_fresh_rows")
    if reg.get("training_prefix_overlap_rows") != 0:
        faults.append("anchor_overlaps_training_prefix")
    if not is_hex64(reg.get("evaluator_source_sha256")):
        faults.append("missing_or_invalid_evaluator_source_sha256")
    if reg.get("evaluator_reads_dev_or_fresh") is not False:
        faults.append("evaluator_must_never_read_dev_or_fresh")
    if reg.get("evaluator_accepts_post_update_checkpoint") is not True:
        faults.append("evaluator_must_accept_post_update_checkpoint")
    if reg.get("evaluator_reconstructs_original_context") is not True:
        faults.append("evaluator_must_reconstruct_original_context")
    if not is_hex64(reg.get("controller_source_sha256")):
        faults.append("missing_or_invalid_controller_source_sha256")
    if not isinstance(reg.get("pm_dispatch_id"), str) or not reg["pm_dispatch_id"].strip():
        faults.append("missing_pm_dispatch_id")
    if (
        not isinstance(reg.get("d40_release_record_id"), str)
        or not reg["d40_release_record_id"].strip()
        or str(reg.get("d40_owner", "")).lower() != "casey"
        or reg.get("d40_uor_lab_env_unset") is not True
    ):
        faults.append("invalid_or_missing_casey_d40_release")

    # 2. Qualification-stage 15% readiness predicate (§4)
    qcost = bundle.get("qualification_cost")
    computed_r: float | None = None
    if not isinstance(qcost, dict):
        faults.append("missing_qualification_cost_section")
    else:
        added_cost = qcost.get("added_gate_cost")
        ungated_cost = qcost.get("ungated_two_arm_dose_cost")
        if (
            not isinstance(added_cost, (int, float))
            or isinstance(added_cost, bool)
            or added_cost < 0
            or not isinstance(ungated_cost, (int, float))
            or isinstance(ungated_cost, bool)
            or ungated_cost <= 0
        ):
            faults.append("qualification_cost_values_invalid")
        else:
            computed_r = float(added_cost) / float(ungated_cost)
            decl_r = qcost.get("cost_ratio_r")
            if decl_r is not None and (
                not isinstance(decl_r, (int, float))
                or isinstance(decl_r, bool)
                or abs(float(decl_r) - computed_r) > 1e-9
            ):
                faults.append("qualification_cost_ratio_r_mismatch")
        if qcost.get("planned_evaluations_per_arm") != EXPECTED_ADMISSION_PRESERVING_EVALUATIONS_PER_ARM:
            faults.append("planned_evaluations_per_arm_must_equal_258")
        if qcost.get("planned_total_evaluations") != EXPECTED_ADMISSION_PRESERVING_TOTAL_EVALUATIONS_TWO_ARMS:
            faults.append("planned_total_evaluations_must_equal_516")
        if qcost.get("shared_setup_custody_close_excluded") is not True:
            faults.append("shared_setup_custody_close_must_be_excluded")
        if not qcost.get("hardware_and_rate_bound"):
            faults.append("qualification_hardware_and_rate_not_bound")

    if (
        not faults
        and computed_r is not None
        and computed_r > EXPECTED_ADMISSION_PRESERVING_MAX_GATE_COST_RATIO
    ):
        if not training_started:
            return {
                "status": "DIAGNOSTIC_STOP",
                "verdict": "STOP_BEFORE_TRAINING_REPLAN_GATE_COST_EXCEEDS_15PCT",
                "mechanism_negative": False,
                "keep": False,
                "cost_ratio_r": computed_r,
                "max_allowed_cost_ratio_r": EXPECTED_ADMISSION_PRESERVING_MAX_GATE_COST_RATIO,
                "faults": [],
            }
        faults.append("gate_cost_exceeds_15pct_but_training_started")

    # 3. Inherited cp0 pre-run gates & anchor baseline >= 500 on both arms
    prerun = bundle.get("prerun_gates")
    anchor_baselines: dict[str, int] = {}
    if not isinstance(prerun, dict):
        faults.append("missing_prerun_gates_section")
    else:
        for arm in ("relation", "frozen_zero"):
            pg = prerun.get(arm)
            if not isinstance(pg, dict):
                faults.append(f"{arm}_missing_prerun_gates")
                continue
            if (
                pg.get("b3_init") != EXPECTED_A8_CP0_PSUCC_ADMITTED
                or pg.get("full_admitted") != EXPECTED_A8_CP0_FULL_ADMITTED
                or pg.get("top1") != EXPECTED_A8_CP0_FULL_TOP1
                or pg.get("bar_i") != 86
                or pg.get("g_init_passed") is not True
                or pg.get("g_occ_passed") is not True
            ):
                faults.append(f"{arm}_inherited_cp0_prerun_gate_mismatch")
            ab = pg.get("anchor_baseline_admitted")
            if not is_u64(ab) or ab > EXPECTED_TRAIN_PSUCC_TAIL_ANCHOR_ROWS:
                faults.append(f"{arm}_anchor_baseline_invalid")
            elif ab < EXPECTED_ADMISSION_PRESERVING_ANCHOR_MIN_BASELINE:
                faults.append(f"{arm}_anchor_baseline_below_500")
            else:
                anchor_baselines[arm] = int(ab)
        if (
            "relation" in anchor_baselines
            and "frozen_zero" in anchor_baselines
            and anchor_baselines["relation"] != anchor_baselines["frozen_zero"]
        ):
            faults.append("cross_arm_cp0_anchor_baseline_mismatch")

    # 4. Per-Arm Synchronous Barrier & Active Interval Rejection Audit Sequence
    arms_doc = bundle.get("arms")
    if not isinstance(arms_doc, dict):
        faults.append("missing_arms_section")
        arms_doc = {}

    expected_audit_batches = [
        k * EXPECTED_ADMISSION_PRESERVING_AUDIT_CADENCE_BATCHES for k in range(257)
    ] + [CANONICAL_ATTENTION_BATCHES]
    assert len(expected_audit_batches) == EXPECTED_ADMISSION_PRESERVING_EVALUATIONS_PER_ARM

    arm_summaries: dict[str, Any] = {}
    noop_stops: list[str] = []
    q1_stops: list[str] = []

    for arm, expected_lin_sha in (
        ("relation", EXPECTED_A8_RELATION_LINEAGE_SHA256),
        ("frozen_zero", EXPECTED_A8_FROZEN_ZERO_LINEAGE_SHA256),
    ):
        arm_data = arms_doc.get(arm)
        if not isinstance(arm_data, dict):
            faults.append(f"{arm}_missing_arm_bundle")
            continue
        audits = arm_data.get("audits")
        if not isinstance(audits, list) or len(audits) == 0:
            faults.append(f"{arm}_missing_or_empty_audits")
            continue

        baseline_adm = anchor_baselines.get(arm, EXPECTED_ADMISSION_PRESERVING_ANCHOR_MIN_BASELINE)
        min_allowed_adm = baseline_adm - EXPECTED_ADMISSION_PRESERVING_MARGIN_ROWS

        # Audit 0 (baseline at batch 0)
        a0 = audits[0]
        if not isinstance(a0, dict) or a0.get("audit_batch") != 0:
            faults.append(f"{arm}_missing_baseline_audit_at_batch_0")
            continue
        if (
            a0.get("synchronous_pause") is not True
            or a0.get("concurrent_updates") != 0
            or not is_hex64(a0.get("model_storage_sha256"))
            or not is_hex64(a0.get("model_snapshot_sha256"))
            or a0.get("model_epoch") != 0
            or a0.get("accepted_updates") != 0
            or not isinstance(a0.get("driver_cursor"), dict)
            or a0["driver_cursor"].get("completed_batches") != 0
            or a0.get("family_counters") != {"vector": 0, "route": 0, "combine": 0, "relation": 0}
            or a0.get("lineage_sha256") != expected_lin_sha
            or a0.get("anchor_csv_sha256") != EXPECTED_TRAIN_PSUCC_TAIL_ANCHOR_CSV_SHA256
            or a0.get("anchor_admitted") != baseline_adm
            or a0.get("breach") is not False
        ):
            faults.append(f"{arm}_baseline_audit_0_barrier_or_pin_mismatch")

        last_pass_storage_sha = a0.get("model_storage_sha256", "")
        last_pass_snapshot_sha = a0.get("model_snapshot_sha256", "")
        last_pass_epoch = 0
        last_pass_accepted = 0
        last_pass_counters = {"vector": 0, "route": 0, "combine": 0, "relation": 0}

        audited_intervals = 0
        rejected_intervals = 0
        consecutive_full_rejections = 0
        max_consecutive_full_rejections = 0
        arm_noop_batch: int | None = None
        arm_noop_reason: str | None = None
        arm_q1_stop = False

        if len(audits) > EXPECTED_ADMISSION_PRESERVING_EVALUATIONS_PER_ARM:
            faults.append(f"{arm}_too_many_audits_{len(audits)}")
            continue

        for idx in range(1, len(audits)):
            ak = audits[idx]
            exp_batch = expected_audit_batches[idx]
            if not isinstance(ak, dict) or ak.get("audit_batch") != exp_batch:
                faults.append(f"{arm}_audit_{idx}_batch_schedule_mismatch_expected_{exp_batch}")
                break

            # Synchronous barrier & pin checks
            fc = ak.get("family_counters")
            dc = ak.get("driver_cursor")
            acc_u = ak.get("accepted_updates")
            m_ep = ak.get("model_epoch")
            if (
                ak.get("synchronous_pause") is not True
                or ak.get("concurrent_updates") != 0
                or not is_hex64(ak.get("model_storage_sha256"))
                or not is_hex64(ak.get("model_snapshot_sha256"))
                or not is_u64(m_ep)
                or not is_u64(acc_u)
                or not isinstance(dc, dict)
                or dc.get("completed_batches") != exp_batch
                or not isinstance(fc, dict)
                or set(fc.keys()) != {"vector", "route", "combine", "relation"}
                or not all(is_u64(fc[k]) for k in ("vector", "route", "combine", "relation"))
                or fc["route"] != 0
                or sum(fc.values()) != acc_u
                or ak.get("lineage_sha256") != expected_lin_sha
                or ak.get("anchor_csv_sha256") != EXPECTED_TRAIN_PSUCC_TAIL_ANCHOR_CSV_SHA256
            ):
                faults.append(f"{arm}_audit_{exp_batch}_synchronous_barrier_or_pin_invalid")
                break

            if arm == "frozen_zero" and fc["relation"] != 0:
                faults.append(f"frozen_zero_audit_{exp_batch}_accepted_relation_nonzero")
                break

            adm = ak.get("anchor_admitted")
            if not is_u64(adm) or adm > EXPECTED_TRAIN_PSUCC_TAIL_ANCHOR_ROWS:
                faults.append(f"{arm}_audit_{exp_batch}_anchor_admitted_invalid")
                break

            expected_breach = adm < min_allowed_adm
            if ak.get("breach") != expected_breach:
                faults.append(f"{arm}_audit_{exp_batch}_breach_flag_mismatch")
                break

            audited_intervals += 1

            if not expected_breach:
                if (
                    ak.get("action") not in (None, "COMMIT", "PASS")
                    or acc_u < last_pass_accepted
                    or m_ep != acc_u
                    or any(fc[k] < last_pass_counters[k] for k in ("vector", "route", "combine", "relation"))
                ):
                    faults.append(f"{arm}_audit_{exp_batch}_commit_state_monotonicity_violation")
                    break
                last_pass_storage_sha = ak["model_storage_sha256"]
                last_pass_snapshot_sha = ak["model_snapshot_sha256"]
                last_pass_epoch = m_ep
                last_pass_accepted = acc_u
                last_pass_counters = dict(fc)
                consecutive_full_rejections = 0
            else:
                mode = ak.get("rejection_mode", "whole_interval")
                rcount = ak.get("rejection_receipts_count")
                if (
                    ak.get("audit_status") != "AUDIT_BREACH"
                    or ak.get("action") not in ("REJECT_INTERVAL", "REJECT_SELECTIVE")
                    or ak.get("failed_checkpoint_preserved") is not True
                    or ak.get("append_only_evidence_preserved") is not True
                    or ak.get("schedule_cursor_advanced_past_interval") is not True
                    or dc.get("completed_batches") != exp_batch
                    or not is_u64(rcount)
                    or rcount < 1
                ):
                    faults.append(f"{arm}_audit_{exp_batch}_breach_missing_preservation_cursor_or_receipt")
                    break

                if mode == "whole_interval":
                    if (
                        ak.get("restored_model_storage_sha256") != last_pass_storage_sha
                        or ak.get("restored_model_snapshot_sha256", last_pass_snapshot_sha)
                        != last_pass_snapshot_sha
                        or ak.get("restored_model_epoch") != last_pass_epoch
                        or ak.get("restored_accepted_updates") != last_pass_accepted
                        or ak.get("restored_family_counters", last_pass_counters)
                        != last_pass_counters
                    ):
                        faults.append(f"{arm}_audit_{exp_batch}_breach_failed_to_restore_last_passing_state")
                        break
                    rejected_intervals += 1
                    consecutive_full_rejections += 1
                    if consecutive_full_rejections > max_consecutive_full_rejections:
                        max_consecutive_full_rejections = consecutive_full_rejections
                elif mode == "selective":
                    rest_ep = ak.get("restored_model_epoch")
                    rest_acc = ak.get("restored_accepted_updates")
                    rest_adm = ak.get("post_selective_restore_anchor_admitted")
                    rest_fc = ak.get("restored_family_counters")
                    if (
                        ak.get("per_update_isolation_proved") is not True
                        or not is_hex64(ak.get("restored_model_storage_sha256"))
                        or not is_hex64(ak.get("restored_model_snapshot_sha256"))
                        or not is_u64(rest_ep)
                        or not is_u64(rest_acc)
                        or rest_ep != rest_acc
                        or not (last_pass_accepted <= rest_acc < acc_u)
                        or not isinstance(rest_fc, dict)
                        or sum(rest_fc.values()) != rest_acc
                        or not is_u64(rest_adm)
                        or rest_adm < min_allowed_adm
                    ):
                        faults.append(f"{arm}_audit_{exp_batch}_selective_rejection_proof_invalid")
                        break
                    last_pass_storage_sha = ak["restored_model_storage_sha256"]
                    last_pass_snapshot_sha = ak["restored_model_snapshot_sha256"]
                    last_pass_epoch = rest_ep
                    last_pass_accepted = rest_acc
                    last_pass_counters = dict(rest_fc)
                    consecutive_full_rejections = 0
                else:
                    faults.append(f"{arm}_audit_{exp_batch}_unknown_rejection_mode_{mode!r}")
                    break

                if (
                    arm_noop_batch is None
                    and consecutive_full_rejections
                    >= EXPECTED_ADMISSION_PRESERVING_MAX_CONSECUTIVE_FULL_REJECTIONS
                ):
                    arm_noop_batch = exp_batch
                    arm_noop_reason = f"{arm}_three_consecutive_full_interval_rejections_at_batch_{exp_batch}"

            # Check Q1 boundary (batch 4,096 = audit index 64)
            if exp_batch == CANONICAL_Q1_BATCHES:
                q1g = arm_data.get("q1_gate")
                if not isinstance(q1g, dict):
                    faults.append(f"{arm}_missing_q1_gate_at_batch_4096")
                    break
                q1_psucc = q1g.get("q1_dev_psucc_admitted")
                comb_s = q1g.get("combine_sensitive")
                rel_s = q1g.get("relation_sensitive")
                if not (is_u64(q1_psucc) and is_u64(comb_s) and is_u64(rel_s)):
                    faults.append(f"{arm}_q1_gate_counts_invalid")
                    break
                if arm == "frozen_zero" and q1g.get("accepted_relation", 0) != 0:
                    faults.append("frozen_zero_q1_accepted_relation_nonzero")
                    break
                q1_dec = compute_q1_decision(
                    EXPECTED_A8_CP0_PSUCC_ADMITTED, int(q1_psucc), int(comb_s), int(rel_s)
                )
                if q1_dec["diagnostic_stop"]:
                    arm_q1_stop = True
                if (
                    arm_noop_batch is None
                    and rejected_intervals * 2 > audited_intervals
                ):
                    arm_noop_batch = CANONICAL_Q1_BATCHES
                    arm_noop_reason = (
                        f"{arm}_q1_rejection_rate_exceeds_50pct_{rejected_intervals}_of_{audited_intervals}"
                    )

        if faults:
            continue

        last_batch = audits[-1]["audit_batch"]
        if (
            arm_noop_batch is None
            and audited_intervals > 0
            and rejected_intervals * 2 > audited_intervals
        ):
            arm_noop_batch = last_batch
            arm_noop_reason = (
                f"{arm}_rejection_rate_exceeds_50pct_{rejected_intervals}_of_{audited_intervals}"
            )

        # Verify schedule completion / mandatory final 20-batch tail audit at 16,404
        if len(audits) != EXPECTED_ADMISSION_PRESERVING_EVALUATIONS_PER_ARM:
            # Allowed to stop early ONLY if stopped at a valid diagnostic stop boundary
            valid_early_stop = (
                (arm_noop_batch is not None and last_batch == arm_noop_batch)
                or (arm_q1_stop and last_batch == CANONICAL_Q1_BATCHES)
            )
            if not valid_early_stop:
                if len(audits) == 257 and last_batch == 16384:
                    faults.append(f"{arm}_missing_mandatory_final_16404_tail_audit")
                else:
                    faults.append(
                        f"{arm}_incomplete_audit_schedule_{len(audits)}_of_{EXPECTED_ADMISSION_PRESERVING_EVALUATIONS_PER_ARM}"
                    )
                continue

        if arm_q1_stop:
            q1_stops.append(arm)
        if arm_noop_batch is not None and arm_noop_reason is not None:
            noop_stops.append(arm_noop_reason)

        arm_summaries[arm] = {
            "evaluations_count": len(audits),
            "audited_intervals": audited_intervals,
            "rejected_intervals": rejected_intervals,
            "rejection_rate": (
                rejected_intervals / audited_intervals if audited_intervals > 0 else 0.0
            ),
            "max_consecutive_full_rejections": max_consecutive_full_rejections,
            "final_completed_batches": last_batch,
            "final_accepted_updates": last_pass_accepted,
            "final_model_epoch": last_pass_epoch,
            "final_model_storage_sha256": last_pass_storage_sha,
            "final_family_counters": last_pass_counters,
            "q1_diagnostic_stop": arm_q1_stop,
            "noop_triggered_at_batch": arm_noop_batch,
            "noop_reason": arm_noop_reason,
        }

    if faults:
        return {
            "status": "FAIL",
            "verdict": "INSTRUMENT_UNREACHABLE_D31",
            "mechanism_negative": False,
            "keep": False,
            "cost_ratio_r": computed_r,
            "faults": faults,
        }

    if noop_stops:
        return {
            "status": "DIAGNOSTIC_STOP",
            "verdict": "RULE_EFFECTIVELY_NOOP",
            "mechanism_negative": False,
            "keep": False,
            "cost_ratio_r": computed_r,
            "noop_reasons": noop_stops,
            "arms": arm_summaries,
            "faults": [],
        }

    if q1_stops:
        return {
            "status": "DIAGNOSTIC_STOP",
            "verdict": "Q1_STOP_FOR_DIAGNOSIS",
            "mechanism_negative": False,
            "keep": False,
            "cost_ratio_r": computed_r,
            "q1_stopped_arms": q1_stops,
            "arms": arm_summaries,
            "faults": [],
        }

    return {
        "status": "PASS",
        "verdict": "ADMISSION_PRESERVING_DIAGNOSTIC_PASS_DEFER_TO_PARENT_DECISION",
        "mechanism_negative": False,
        "keep": False,
        "cost_ratio_r": computed_r,
        "arms": arm_summaries,
        "faults": [],
    }


def build_synthetic_admission_preserving_bundle() -> dict[str, Any]:
    """Construct a valid 2-arm (258 evaluations/arm) active interval rejection bundle for self-tests 50..56."""
    audit_batches = [
        k * EXPECTED_ADMISSION_PRESERVING_AUDIT_CADENCE_BATCHES for k in range(257)
    ] + [CANONICAL_ATTENTION_BATCHES]
    baseline_adm = 1120

    def make_arm(is_fz: bool, breach_indices: set[int]) -> dict[str, Any]:
        lin_sha = (
            EXPECTED_A8_FROZEN_ZERO_LINEAGE_SHA256
            if is_fz
            else EXPECTED_A8_RELATION_LINEAGE_SHA256
        )
        prefix_ch = "b" if is_fz else "a"
        storage_sha = (prefix_ch + "0") * 32
        snap_sha = (prefix_ch + "1") * 32
        epoch = 0
        acc = 0
        counters = {"vector": 0, "route": 0, "combine": 0, "relation": 0}

        audits: list[dict[str, Any]] = [
            {
                "audit_batch": 0,
                "synchronous_pause": True,
                "concurrent_updates": 0,
                "model_storage_sha256": storage_sha,
                "model_snapshot_sha256": snap_sha,
                "model_epoch": 0,
                "accepted_updates": 0,
                "driver_cursor": {
                    "completed_batches": 0,
                    "accepted_updates": 0,
                    "corpus_pass": 0,
                    "next_target": 0,
                    "next_coordinate": 0,
                },
                "family_counters": dict(counters),
                "lineage_sha256": lin_sha,
                "anchor_csv_sha256": EXPECTED_TRAIN_PSUCC_TAIL_ANCHOR_CSV_SHA256,
                "anchor_admitted": baseline_adm,
                "breach": False,
                "action": "PASS",
            }
        ]

        for idx in range(1, len(audit_batches)):
            b = audit_batches[idx]
            cand_storage = hashlib.sha256(f"{prefix_ch}:storage:{idx}".encode("utf-8")).hexdigest()
            cand_snap = hashlib.sha256(f"{prefix_ch}:snap:{idx}".encode("utf-8")).hexdigest()
            cand_counters = dict(counters)
            cand_counters["vector"] += 1
            if idx % 4 == 0:
                cand_counters["combine"] += 1
            if not is_fz and idx % 8 == 0:
                cand_counters["relation"] += 1
            cand_acc = sum(cand_counters.values())
            cand_epoch = cand_acc

            if idx in breach_indices:
                audits.append(
                    {
                        "audit_batch": b,
                        "synchronous_pause": True,
                        "concurrent_updates": 0,
                        "model_storage_sha256": cand_storage,
                        "model_snapshot_sha256": cand_snap,
                        "model_epoch": cand_epoch,
                        "accepted_updates": cand_acc,
                        "driver_cursor": {
                            "completed_batches": b,
                            "accepted_updates": cand_acc,
                            "corpus_pass": 0,
                            "next_target": b * 64,
                            "next_coordinate": b % 45,
                        },
                        "family_counters": cand_counters,
                        "lineage_sha256": lin_sha,
                        "anchor_csv_sha256": EXPECTED_TRAIN_PSUCC_TAIL_ANCHOR_CSV_SHA256,
                        "anchor_admitted": baseline_adm - 25,  # breach (< baseline - 24)
                        "breach": True,
                        "audit_status": "AUDIT_BREACH",
                        "action": "REJECT_INTERVAL",
                        "rejection_mode": "whole_interval",
                        "failed_checkpoint_preserved": True,
                        "append_only_evidence_preserved": True,
                        "schedule_cursor_advanced_past_interval": True,
                        "rejection_receipts_count": cand_acc - acc,
                        "restored_model_storage_sha256": storage_sha,
                        "restored_model_snapshot_sha256": snap_sha,
                        "restored_model_epoch": epoch,
                        "restored_accepted_updates": acc,
                        "restored_family_counters": dict(counters),
                    }
                )
            else:
                storage_sha = cand_storage
                snap_sha = cand_snap
                epoch = cand_epoch
                acc = cand_acc
                counters = cand_counters
                audits.append(
                    {
                        "audit_batch": b,
                        "synchronous_pause": True,
                        "concurrent_updates": 0,
                        "model_storage_sha256": storage_sha,
                        "model_snapshot_sha256": snap_sha,
                        "model_epoch": epoch,
                        "accepted_updates": acc,
                        "driver_cursor": {
                            "completed_batches": b,
                            "accepted_updates": acc,
                            "corpus_pass": 0,
                            "next_target": b * 64,
                            "next_coordinate": b % 45,
                        },
                        "family_counters": dict(counters),
                        "lineage_sha256": lin_sha,
                        "anchor_csv_sha256": EXPECTED_TRAIN_PSUCC_TAIL_ANCHOR_CSV_SHA256,
                        "anchor_admitted": baseline_adm - 10,  # within 24-row margin
                        "breach": False,
                        "action": "COMMIT",
                    }
                )

        return {
            "q1_gate": {
                "q1_dev_psucc_admitted": 920 if not is_fz else 910,
                "combine_sensitive": 120,
                "relation_sensitive": 85 if not is_fz else 0,
                "accepted_relation": 8 if not is_fz else 0,
            },
            "audits": audits,
        }

    return {
        "schema": "uor-r4.admission-preserving-guard-bundle/1",
        "training_started": True,
        "registration": {
            "sheet_path": EXPECTED_ADMISSION_PRESERVING_SHEET_REL_PATH,
            "sheet_sha256": EXPECTED_ADMISSION_PRESERVING_FIT_SHEET_SHA256,
            "sheet_verdict": "FIT",
            "parent_sheet_sha256": EXPECTED_A8_SHEET_SHA256,
            "a8_table_sha256": EXPECTED_A8_TABLE_FILE_SHA256,
            "train_tokens_sha256": EXPECTED_TRAIN_TOKENS_SHA256,
            "train_mask_sha256": EXPECTED_TRAIN_MASK_SHA256,
            "anchor_csv_sha256": EXPECTED_TRAIN_PSUCC_TAIL_ANCHOR_CSV_SHA256,
            "anchor_rows": EXPECTED_TRAIN_PSUCC_TAIL_ANCHOR_ROWS,
            "anchor_first_eligible_ordinal": EXPECTED_TRAIN_PSUCC_TAIL_FIRST_ROW[0],
            "anchor_last_eligible_ordinal": EXPECTED_TRAIN_PSUCC_TAIL_LAST_ROW[0],
            "dev_or_heldout_or_fresh_rows_in_anchor": 0,
            "training_prefix_overlap_rows": 0,
            "evaluator_source_sha256": "e" * 64,
            "evaluator_reads_dev_or_fresh": False,
            "evaluator_accepts_post_update_checkpoint": True,
            "evaluator_reconstructs_original_context": True,
            "controller_source_sha256": "c" * 64,
            "pm_dispatch_id": "https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6263492322",
            "d40_release_record_id": "d40-casey-admission-preserving-20261019",
            "d40_owner": "Casey",
            "d40_uor_lab_env_unset": True,
        },
        "qualification_cost": {
            "added_gate_cost": 1.14,
            "ungated_two_arm_dose_cost": 11.40,
            "cost_ratio_r": 0.10,
            "planned_evaluations_per_arm": EXPECTED_ADMISSION_PRESERVING_EVALUATIONS_PER_ARM,
            "planned_total_evaluations": EXPECTED_ADMISSION_PRESERVING_TOTAL_EVALUATIONS_TWO_ARMS,
            "shared_setup_custody_close_excluded": True,
            "hardware_and_rate_bound": True,
        },
        "prerun_gates": {
            "relation": {
                "b3_init": EXPECTED_A8_CP0_PSUCC_ADMITTED,
                "full_admitted": EXPECTED_A8_CP0_FULL_ADMITTED,
                "top1": EXPECTED_A8_CP0_FULL_TOP1,
                "bar_i": 86,
                "g_init_passed": True,
                "g_occ_passed": True,
                "anchor_baseline_admitted": baseline_adm,
            },
            "frozen_zero": {
                "b3_init": EXPECTED_A8_CP0_PSUCC_ADMITTED,
                "full_admitted": EXPECTED_A8_CP0_FULL_ADMITTED,
                "top1": EXPECTED_A8_CP0_FULL_TOP1,
                "bar_i": 86,
                "g_init_passed": True,
                "g_occ_passed": True,
                "anchor_baseline_admitted": baseline_adm,
            },
        },
        "arms": {
            "relation": make_arm(False, {10, 42}),
            "frozen_zero": make_arm(True, {15, 88}),
        },
    }


TRAIN_ANCHOR_SCHEMA = "uor-r4.train-copy-successor-anchor/v1"
TRAIN_ANCHOR_SELECTION_POLICY = "first-copy-successor-at-or-after-original-eligible-index/v1"
TRAIN_ANCHOR_ROW_CSV_POLICY = (
    "original-eligible-index,target-absolute-position,target-token,1/headerless-decimal-lf/v1"
)
TRAIN_ANCHOR_WINDOW_HASH_POLICY = (
    "uor-r4.original-corpus-window/identities-u32-u64-u64-u64-le/v1"
)

QUAL_PLAN_SCHEMA = "uor-r4.attention-audit-source-qualification-plan/1"
QUAL_ADMISSION_SCHEMA = "uor-r4.attention-audit-source-qualification-admission/1"
QUAL_OWNER_SCHEMA = "uor-r4.attention-audit-source-qualification-owner-decision/1"
QUAL_OWNER_DECISION = "RELEASE_FINITE_AUDITED_SOURCE_QUALIFICATION_ONLY"
QUAL_SCOPE = "m1-attention-active-audit-finite-source-qualification-v1"
QUAL_PROTOCOL = "train61-synthetic1-interval64-two-arm-throughput-v1"
QUAL_AUTH_SCHEMA = "uor-r4.attention-audit-qualification-authorization/1"
QUAL_TIMING_PROTOCOL_SCHEMA = "uor-r4.attention-audit-measurement-protocol/1"
QUAL_BOOKKEEPING_SCHEMA = "uor-r4.attention-audit-host-bookkeeping-measurement/1"
QUAL_CALIBRATION_SCHEMA = "uor-r4.attention-audit-calibration/1"
QUAL_BATTERY_SCHEMA = "uor-r4.attention-audit-qualification/1"
QUAL_REPORT_SCHEMA = "uor-r4.attention-audit-finite-qualification-report/1"
QUAL_REQUIRED_CHECK_NAMES: list[str] = [
    "canonical_initlock_c2",
    "required_negatives",
    "g_occ",
    "sensitivity",
    "audited_restore_epoch_cursor",
]
QUAL_MAX_PROGRESS_HISTORY = 16922

AUDITED_DOSE_PLAN_SCHEMA = "uor-r4.attention-audit-owner-plan/1"
AUDITED_DOSE_ADMISSION_SCHEMA = "uor-r4.attention-audit-owner-admission/1"
AUDITED_DOSE_OWNER_SCHEMA = "uor-r4.attention-audit-owner-decision/1"
AUDITED_DOSE_SCOPE_PREFIX = "m1-attention-active-whole-interval-rejection-v1"
AUDITED_DOSE_OWNER_DECISION = "RELEASE_D40_AUDITED_WHOLE_INTERVAL_ARM_ONLY"
AUDITED_DOSE_REVIEW_VERDICT = "APPROVE_ACTIVE_WHOLE_INTERVAL_ADMISSION_PLAN"
AUDITED_DOSE_COST_DECISION_SCHEMA = "uor-r4.attention-audit-costed-dose-decision/1"
AUDITED_DOSE_COST_ATTESTATION_SCHEMA = "uor-r4.attention-audit-dose-cost-attestation/1"
AUDITED_DOSE_RESOURCE_OBSERVATION_SCHEMA = "uor-r4.attention-audit-resource-observation/1"
AUDITED_DOSE_DURABLE_RECORD_SCHEMA = "uor-r4.attention-whole-interval-audit/v1"
AUDITED_DOSE_EVALUATION_SCHEMA = "uor-r4.attention-audit-evaluation/1"
AUDITED_DOSE_RESTORE_INTENT_SCHEMA = "uor-r4.attention-audit-restore-intent/1"
AUDITED_DOSE_RESTORE_OBSERVATION_SCHEMA = "uor-r4.attention-audit-restore-observation/1"
AUDITED_DOSE_Q1_SCHEMA = "uor-r4.attention-audit-q1/1"
AUDITED_DOSE_CHECKPOINT_SCHEMA = "uor-r4.attention-audit-checkpoint/1"
AUDITED_DOSE_REPORT_SCHEMA = "uor-r4.attention-audit-report/1"
AUDITED_DOSE_FAILED_MODEL_SCHEMA = "uor-r4.attention-audit-failed-model/1"
EXPECTED_REGISTERED_AUDIT_POLICY: dict[str, Any] = {
    "cadence": 64,
    "max_batches": 16404,
    "final_tail_batches": 20,
    "margin": 24,
    "minimum_baseline": 500,
    "reject_whole_interval": True,
    "consecutive_full_rejections": 3,
    "audits_per_arm": 258,
}


def verify_integrated_audited_source_contract(repo_root: Path) -> dict[str, Any]:
    """Verify the integrated #2782 audited-learning Rust modules, float-free invariants, and CLI subcommands."""
    src_dir = repo_root / "crates" / "uor-r4-discrete" / "src"
    module_digests: dict[str, dict[str, Any]] = {}
    for fname, (exp_sha, exp_bytes, exp_lines) in EXPECTED_INTEGRATED_MODULE_PINS.items():
        fpath = src_dir / fname
        raw = read_bytes_checked(fpath, fname)
        actual_sha = hashlib.sha256(raw).hexdigest()
        text = raw.decode("utf-8")
        actual_lines = text.count("\n")
        require(
            actual_sha == exp_sha and len(raw) == exp_bytes and actual_lines == exp_lines,
            f"integrated module {fname} SHA/bytes/lines mismatch: {actual_sha} ({len(raw)} B, {actual_lines} lines) != {exp_sha} ({exp_bytes} B, {exp_lines} lines)",
        )
        require(
            not re.search(r"\b(f32|f64)\b", text),
            f"integrated module {fname} contains forbidden f32/f64 token",
        )
        module_digests[fname] = {
            "sha256": actual_sha,
            "bytes": len(raw),
            "lines": actual_lines,
        }

    main_text = read_text_checked(src_dir / "main.rs", "main.rs")
    for subcmd in (
        '"attention-audit-build-identity"',
        '"attention-audit-check-plan"',
        '"attention-audit-owner-release"',
        '"attention-audit"',
        '"attention-audit-qualification-check-plan"',
        '"attention-audit-qualification-owner-release"',
        '"attention-audit-qualify"',
    ):
        require(subcmd in main_text, f"main.rs missing required audited CLI subcommand: {subcmd}")
    anchor_text = read_text_checked(src_dir / "attention_anchor.rs", "attention_anchor.rs")
    require(
        "pub fn build(corpus: &PreparedCorpus, policy: AnchorPolicy)" in anchor_text,
        "attention_anchor.rs missing TrainAnchorManifest::build",
    )

    return {
        "status": "PASS",
        "driver_source_sha256": EXPECTED_INTEGRATED_DRIVER_SOURCE_SHA256,
        "compiled_source_sha256": EXPECTED_COMPILED_SOURCE_SHA256,
        "modules": module_digests,
    }


def compute_original_window_pin(window: list[dict[str, Any]]) -> dict[str, str]:
    """Replicate `OriginalWindowPin::from_window` (`attention_anchor.rs:47-65`)."""
    buf = bytearray(TRAIN_ANCHOR_WINDOW_HASH_POLICY.encode("utf-8"))
    buf.append(0)
    buf.extend(struct.pack("<Q", len(window)))
    for ident in window:
        buf.extend(
            struct.pack(
                "<IQQQ",
                int(ident["token"]),
                int(ident["occurrence"]),
                int(ident["record"]),
                int(ident["version"]),
            )
        )
    return {
        "encoding": TRAIN_ANCHOR_WINDOW_HASH_POLICY,
        "sha256": hashlib.sha256(buf).hexdigest(),
    }


def verify_train_anchor_manifest(
    manifest: dict[str, Any],
    train_tokens_path: Path | None = None,
    train_mask_path: Path | None = None,
    require_canonical_3847: bool = True,
) -> dict[str, Any]:
    """Verify a serialized `TrainAnchorManifest` (`uor-r4.train-copy-successor-anchor/v1`) matching `attention_anchor.rs`."""
    require(isinstance(manifest, dict), "TrainAnchorManifest must be a dict")
    require(
        manifest.get("schema") == TRAIN_ANCHOR_SCHEMA
        and manifest.get("selection_policy") == TRAIN_ANCHOR_SELECTION_POLICY
        and manifest.get("row_csv_policy") == TRAIN_ANCHOR_ROW_CSV_POLICY,
        "TrainAnchorManifest schema/policy header mismatch",
    )
    corpus = manifest.get("corpus")
    policy = manifest.get("policy")
    rows = manifest.get("rows")
    require(
        isinstance(corpus, dict) and isinstance(policy, dict) and isinstance(rows, list),
        "TrainAnchorManifest missing corpus, policy, or rows",
    )
    src = corpus.get("source", {})
    require(
        src.get("split") == "train"
        and corpus.get("vocabulary") == 4096
        and corpus.get("bos") == {"token": "<|bos|>", "id": 0}
        and corpus.get("window_policy")
        == "response-masked-targets/within-document-predecessor/sliding-prefix/ascending-once/v1"
        and corpus.get("identity_policy")
        == "corpus-SHA-bound/split-prefix2bits/document-ordinal62bits/absolute-token-occurrence/version0/v1",
        "TrainAnchorManifest corpus binding must be TRAIN with canonical V4096/BOS0 policies",
    )
    count = policy.get("count")
    start_el = policy.get("start_eligible_index")
    ctx = policy.get("context")
    require(
        is_u64(count)
        and count > 0
        and is_u64(start_el)
        and is_u64(ctx)
        and 1 <= ctx <= 243
        and len(rows) == count,
        "TrainAnchorManifest policy or row count invalid",
    )
    if require_canonical_3847:
        require(
            start_el == EXPECTED_TRAIN_PSUCC_TAIL_START_ELIGIBLE_INDEX
            and count == EXPECTED_TRAIN_PSUCC_TAIL_ANCHOR_ROWS
            and ctx == 192
            and src.get("tokens", {}).get("sha256") == EXPECTED_TRAIN_TOKENS_SHA256
            and src.get("mask", {}).get("sha256") == EXPECTED_TRAIN_MASK_SHA256
            and corpus.get("payload_tokens") == 7512797
            and corpus.get("eligible_targets") == 2425925
            and corpus.get("documents") == 60000,
            "TrainAnchorManifest does not match canonical 3,847-row mw-balp TRAIN anchor pins",
        )

    csv_lines: list[str] = []
    prev_el = -1
    prev_pos = -1
    canonical_rows: list[dict[str, Any]] = []
    for idx, r in enumerate(rows):
        require(isinstance(r, dict), f"row[{idx}] must be a dict")
        ao = r.get("anchor_ordinal")
        el = r.get("eligible_index")
        pos = r.get("target_position")
        tok = r.get("target_token")
        tid = r.get("target_identity")
        ow = r.get("original_window")
        require(
            ao == idx
            and is_u64(el)
            and el >= start_el
            and el > prev_el
            and is_u64(pos)
            and pos > prev_pos
            and is_u64(tok)
            and 0 < tok < 4096
            and isinstance(tid, dict)
            and tid.get("token") == tok
            and tid.get("occurrence") == pos
            and is_u64(tid.get("record"))
            and (int(tid["record"]) >> 62) == 0
            and tid.get("version") == 0
            and isinstance(ow, dict)
            and ow.get("encoding") == TRAIN_ANCHOR_WINDOW_HASH_POLICY
            and is_hex64(ow.get("sha256")),
            f"TrainAnchorManifest row[{idx}] identity/ordering/window pin invalid: {r!r}",
        )
        prev_el = int(el)
        prev_pos = int(pos)
        csv_lines.append(f"{el},{pos},{tok},1\n")
        canonical_rows.append(
            {
                "anchor_ordinal": ao,
                "eligible_index": el,
                "target_position": pos,
                "target_token": tok,
                "target_identity": {
                    "token": tid["token"],
                    "occurrence": tid["occurrence"],
                    "record": tid["record"],
                    "version": tid["version"],
                },
                "original_window": {
                    "encoding": ow["encoding"],
                    "sha256": ow["sha256"],
                },
            }
        )

    recomputed_csv_sha = hashlib.sha256("".join(csv_lines).encode("utf-8")).hexdigest()
    require(
        manifest.get("row_csv_sha256") == recomputed_csv_sha,
        f"TrainAnchorManifest row_csv_sha256 mismatch: {manifest.get('row_csv_sha256')} != {recomputed_csv_sha}",
    )
    if require_canonical_3847:
        require(
            recomputed_csv_sha == EXPECTED_TRAIN_PSUCC_TAIL_ANCHOR_CSV_SHA256
            and rows[0]["original_window"]["sha256"] == EXPECTED_TRAIN_ANCHOR_ROW0_WINDOW_SHA256
            and rows[-1]["original_window"]["sha256"] == EXPECTED_TRAIN_ANCHOR_ROW_LAST_WINDOW_SHA256,
            "TrainAnchorManifest canonical row_csv_sha256 or endpoint OriginalWindowPin SHA-256 mismatch",
        )

    ordered_corpus = {
        "source": {
            "split": src["split"],
            "tokens": {"path": src["tokens"]["path"], "sha256": src["tokens"]["sha256"]},
            "mask": {"path": src["mask"]["path"], "sha256": src["mask"]["sha256"]},
            "expected_payload_tokens": src["expected_payload_tokens"],
            "expected_eligible_targets": src["expected_eligible_targets"],
        },
        "vocabulary": corpus["vocabulary"],
        "bos": {"token": corpus["bos"]["token"], "id": corpus["bos"]["id"]},
        "payload_tokens": corpus["payload_tokens"],
        "mask_bytes": corpus["mask_bytes"],
        "mask_selected_positions": corpus["mask_selected_positions"],
        "eligible_targets": corpus["eligible_targets"],
        "documents": corpus["documents"],
        "window_policy": corpus["window_policy"],
        "identity_policy": corpus["identity_policy"],
    }
    ordered_payload = {
        "schema": manifest["schema"],
        "selection_policy": manifest["selection_policy"],
        "row_csv_policy": manifest["row_csv_policy"],
        "corpus": ordered_corpus,
        "policy": {
            "start_eligible_index": policy["start_eligible_index"],
            "count": policy["count"],
            "context": policy["context"],
        },
        "rows": canonical_rows,
        "row_csv_sha256": recomputed_csv_sha,
    }
    recomputed_manifest_sha = hashlib.sha256(
        json.dumps(ordered_payload, separators=(",", ":")).encode("utf-8")
    ).hexdigest()
    require(
        manifest.get("manifest_sha256") == recomputed_manifest_sha,
        f"TrainAnchorManifest manifest_sha256 mismatch: {manifest.get('manifest_sha256')} != {recomputed_manifest_sha}",
    )

    if train_tokens_path is not None and train_mask_path is not None:
        uort_rec = recompute_train_psucc_tail_anchor_from_uort(
            train_tokens_path, train_mask_path, int(start_el), int(count)
        )
        require(
            uort_rec["csv_sha256"] == recomputed_csv_sha,
            "TrainAnchorManifest differs from UORT recomputation",
        )

    return {
        "status": "PASS",
        "manifest_sha256": recomputed_manifest_sha,
        "row_csv_sha256": recomputed_csv_sha,
        "rows": len(rows),
    }


def _qual_plan_payload_sha256(plan: dict[str, Any]) -> str:
    """Replicate `attention_audit_qualification::payload_sha256` (serde_json::Value BTreeMap order with 4 auth keys removed)."""
    payload = copy.deepcopy(plan)
    for k in ("qualification_fit", "owner_scope_cost", "pm_dispatch", "independent_review"):
        payload.pop(k, None)
    payload["relation_config"] = canonical_driver_config(payload["relation_config"])
    payload["frozen_zero_config"] = canonical_driver_config(payload["frozen_zero_config"])
    return hashlib.sha256(
        json.dumps(payload, separators=(",", ":"), sort_keys=True).encode("utf-8")
    ).hexdigest()


def verify_audited_qualification_artifacts(bundle: dict[str, Any]) -> dict[str, Any]:
    """Consumer-verify the finite source qualification plan, admission, 4 authorizations, calibration, and 5-check report (`attention_audit_qualification.rs`)."""
    faults: list[str] = []
    if not isinstance(bundle, dict):
        return {
            "status": "FAIL",
            "verdict": "INSTRUMENT_UNREACHABLE_D31",
            "mechanism_negative": False,
            "keep": False,
            "faults": ["qualification_bundle_not_a_dict"],
        }

    plan = bundle.get("plan")
    admission = bundle.get("admission")
    owner = bundle.get("owner")
    auths = bundle.get("authorizations", {})
    report = bundle.get("report")
    if not (
        isinstance(plan, dict)
        and isinstance(admission, dict)
        and isinstance(owner, dict)
        and isinstance(auths, dict)
        and isinstance(report, dict)
    ):
        return {
            "status": "FAIL",
            "verdict": "INSTRUMENT_UNREACHABLE_D31",
            "mechanism_negative": False,
            "keep": False,
            "faults": ["missing_plan_admission_owner_authorizations_or_report"],
        }

    # 1. Verify QualificationPlan header, execution, configs, anchor pin, and Limits
    ex = plan.get("execution", {})
    if (
        plan.get("schema") != QUAL_PLAN_SCHEMA
        or plan.get("status") != "HELD_FINITE_QUALIFICATION_ADMISSION_REQUIRED"
        or plan.get("scope") != QUAL_SCOPE
        or ex.get("base_source") != EXPECTED_ATTENTION_BASE_SOURCE
        or not is_hex40(ex.get("reviewed_source_commit"))
        or ex.get("driver_source_sha256") != EXPECTED_INTEGRATED_DRIVER_SOURCE_SHA256
        or ex.get("compiled_source_sha256") != EXPECTED_COMPILED_SOURCE_SHA256
        or ex.get("sheet_sha256") != EXPECTED_A8_SHEET_SHA256
        or not is_hex64(ex.get("executable_sha256"))
        or plan.get("sheet", {}).get("sha256") != EXPECTED_A8_SHEET_SHA256
        or not is_hex64(plan.get("anchor", {}).get("sha256"))
    ):
        faults.append("qualification_plan_header_or_execution_identity_invalid")

    rel_cfg = plan.get("relation_config")
    fz_cfg = plan.get("frozen_zero_config")
    if not (isinstance(rel_cfg, dict) and isinstance(fz_cfg, dict)):
        faults.append("qualification_plan_configs_missing")
    else:
        r_csha, r_lsha = compute_config_and_lineage_sha(
            rel_cfg, EXPECTED_INTEGRATED_DRIVER_SOURCE_SHA256
        )
        f_csha, f_lsha = compute_config_and_lineage_sha(
            fz_cfg, EXPECTED_INTEGRATED_DRIVER_SOURCE_SHA256
        )
        if (
            r_csha != EXPECTED_A8_RELATION_CONFIG_SHA256
            or r_lsha != EXPECTED_INTEGRATED_A8_RELATION_LINEAGE_SHA256
            or f_csha != EXPECTED_A8_FROZEN_ZERO_CONFIG_SHA256
            or f_lsha != EXPECTED_INTEGRATED_A8_FROZEN_ZERO_LINEAGE_SHA256
        ):
            faults.append("qualification_plan_config_or_integrated_lineage_sha256_mismatch")

    lim = plan.get("limits", {})
    gs = lim.get("gate_samples", 0)
    if (
        not isinstance(lim, dict)
        or lim.get("protocol") != QUAL_PROTOCOL
        or lim.get("anchor_batches_per_sample") != 61
        or not is_u64(gs)
        or not (1 <= gs <= 3)
        or lim.get("synthetic_apply_calls") != 1
        or lim.get("synthetic_accepted_updates") != 1
        or lim.get("synthetic_stale_apply_refusals") != 1
        or lim.get("production_stale_apply_refusals_per_arm") != 1
        or lim.get("anchor_evaluations_per_arm") != gs + 1
        or lim.get("rejected_interval_attempts") != 64
        or lim.get("throughput_attempts_per_arm") != 384
        or lim.get("throughput_warmup_attempts_per_arm") != 0
        or lim.get("projected_audits") != 516
        or lim.get("projected_restore_events") != 514
        or lim.get("timer_scope") != "synchronized-device-snapshot-seal-fsync-readback-lifecycle-v1"
        or lim.get("cache_policy") != "fresh-device-per-arm-no-unreported-warmup-v1"
        or not is_u64(lim.get("max_runtime_seconds"))
        or lim["max_runtime_seconds"] <= 0
        or not is_u64(lim.get("max_cost_cents"))
        or lim["max_cost_cents"] <= 0
    ):
        faults.append("qualification_plan_limits_invalid")

    # 2. Verify 4 Authorization records binding payload_sha256(plan)
    expected_payload_sha = _qual_plan_payload_sha256(plan)
    auth_specs = (
        ("qualification_fit", "qualification_fit", {"Claude", "DeepSeek", "AGY"}, "TEST FITNESS: FIT"),
        ("owner_scope_cost", "owner_cost_scope", {"Casey"}, "APPROVE_EXACT_FINITE_QUALIFICATION_SCOPE_AND_COST"),
        ("pm_dispatch", "pm_dispatch", {"Kimi"}, "DISPATCH_FINITE_SOURCE_QUALIFICATION_ONLY"),
        ("independent_review", "source_plan_review", {"Claude", "DeepSeek", "AGY"}, "APPROVE_EXACT_QUALIFICATION_PLAN"),
    )
    for plan_key, exp_role, allowed_actors, exp_verdict in auth_specs:
        auth_obj = auths.get(plan_key)
        pin_obj = plan.get(plan_key, {})
        if not isinstance(auth_obj, dict) or not isinstance(pin_obj, dict):
            faults.append(f"qualification_missing_authorization_{plan_key}")
            continue
        auth_sha = hashlib.sha256(
            json.dumps(auth_obj, separators=(",", ":")).encode("utf-8")
        ).hexdigest()
        if (
            auth_obj.get("schema") != QUAL_AUTH_SCHEMA
            or auth_obj.get("role") != exp_role
            or auth_obj.get("actor") not in allowed_actors
            or auth_obj.get("verdict") != exp_verdict
            or auth_obj.get("scope") != QUAL_SCOPE
            or auth_obj.get("execution") != ex
            or auth_obj.get("reviewed_payload_sha256") != expected_payload_sha
            or not isinstance(auth_obj.get("receipt_url"), str)
            or not bool(RECEIPT_URL_RE.match(auth_obj["receipt_url"]))
            or pin_obj.get("sha256") != auth_sha
        ):
            faults.append(f"qualification_authorization_{plan_key}_invalid")

    # 3. Verify QualificationOwner & QualificationAdmission
    plan_sha = hashlib.sha256(
        json.dumps(plan, separators=(",", ":")).encode("utf-8")
    ).hexdigest()
    owner_sha = hashlib.sha256(
        json.dumps(owner, separators=(",", ":")).encode("utf-8")
    ).hexdigest()
    owner_pin = admission.get("owner_release", {})
    if (
        admission.get("schema") != QUAL_ADMISSION_SCHEMA
        or admission.get("plan") != plan
        or owner.get("schema") != QUAL_OWNER_SCHEMA
        or owner.get("owner") != "Casey"
        or owner.get("decision") != QUAL_OWNER_DECISION
        or owner.get("scope") != QUAL_SCOPE
        or owner.get("plan_sha256") != plan_sha
        or owner.get("receipt_url") != f"owner-terminal:{owner_pin.get('path')}"
        or owner_pin.get("sha256") != owner_sha
    ):
        faults.append("qualification_owner_or_admission_invalid")

    admission_sha = hashlib.sha256(
        json.dumps(admission, separators=(",", ":")).encode("utf-8")
    ).hexdigest()

    # 4. Verify Calibration, TimingProtocol, BookkeepingMeasurement, and 5-Check Qualification battery
    cal = report.get("calibration", {})
    tp = cal.get("timing_protocol", {}) if isinstance(cal, dict) else {}
    bk = cal.get("bookkeeping", {}) if isinstance(cal, dict) else {}
    qual = report.get("qualification", {})

    if (
        report.get("schema") != QUAL_REPORT_SCHEMA
        or report.get("scope") != QUAL_SCOPE
        or report.get("admission_sha256") != admission_sha
        or report.get("anchor_manifest_sha256") != plan.get("anchor", {}).get("sha256")
        or report.get("anchor_row_csv_sha256") != EXPECTED_TRAIN_PSUCC_TAIL_ANCHOR_CSV_SHA256
        or report.get("anchor_rows") != EXPECTED_TRAIN_PSUCC_TAIL_ANCHOR_ROWS
        or report.get("training") != "HELD"
        or report.get("full_dose_started") is not False
    ):
        faults.append("qualification_report_header_or_hold_invariant_invalid")

    if (
        not isinstance(tp, dict)
        or tp.get("schema") != QUAL_TIMING_PROTOCOL_SCHEMA
        or tp.get("timer_scope") != "synchronized-device-snapshot-seal-fsync-readback-lifecycle-v1"
        or tp.get("cache_policy") != "fresh-device-per-arm-no-unreported-warmup-v1"
        or tp.get("cuda_synchronized_before_and_after") is not True
        or tp.get("fresh_device_per_arm") is not True
        or tp.get("unreported_warmup_evaluations") != 0
        or tp.get("includes_anchor_batches_and_snapshot_seal") is not True
        or tp.get("includes_restore_fsync_and_readback") is not True
        or tp.get("includes_step_and_boundary_host_bookkeeping") is not True
    ):
        faults.append("qualification_timing_protocol_invalid")

    if (
        not isinstance(bk, dict)
        or bk.get("schema") != QUAL_BOOKKEEPING_SCHEMA
        or bk.get("progress_history_entries") != QUAL_MAX_PROGRESS_HISTORY
        or not is_u64(bk.get("step_bookkeeping_ns"))
        or bk["step_bookkeeping_ns"] <= 0
        or not is_u64(bk.get("boundary_bookkeeping_ns"))
        or bk["boundary_bookkeeping_ns"] <= 0
        or not is_u64(bk.get("breach_ack_bookkeeping_ns"))
        or bk["breach_ack_bookkeeping_ns"] <= 0
    ):
        faults.append("qualification_bookkeeping_measurement_invalid")

    passes_15pct = False
    if not isinstance(cal, dict) or cal.get("schema") != QUAL_CALIBRATION_SCHEMA:
        faults.append("qualification_calibration_missing_or_schema_invalid")
    else:
        g_eval = cal.get("gate_evaluation_ns", 0)
        rest_ns = cal.get("restore_and_FailedModel_preservation_ns", 0)
        ungated_ns = cal.get("projected_ungated_two_arm_dose_ns", 0)
        added_ns = cal.get("projected_added_gate_ns", 0)
        step_ns = bk.get("step_bookkeeping_ns", 0) if isinstance(bk, dict) else 0
        bound_ns = bk.get("boundary_bookkeeping_ns", 0) if isinstance(bk, dict) else 0
        ack_ns = bk.get("breach_ack_bookkeeping_ns", 0) if isinstance(bk, dict) else 0
        expected_added_ns = (
            516 * g_eval + 32808 * step_ns + 516 * bound_ns + 514 * (rest_ns + ack_ns)
        )
        expected_passes = (added_ns * 100) <= (ungated_ns * 15)
        if (
            cal.get("protocol") != QUAL_PROTOCOL
            or cal.get("anchor_rows") != EXPECTED_TRAIN_PSUCC_TAIL_ANCHOR_ROWS
            or cal.get("anchor_batches_per_evaluation") != 61
            or cal.get("projected_audits") != 516
            or cal.get("projected_restore_events") != 514
            or not is_u64(g_eval)
            or g_eval <= 0
            or not is_u64(rest_ns)
            or rest_ns <= 0
            or not is_u64(ungated_ns)
            or ungated_ns <= 0
            or added_ns != expected_added_ns
            or cal.get("passes_15_percent_gate") != expected_passes
        ):
            faults.append("qualification_calibration_arithmetic_or_gate_flag_mismatch")
        else:
            passes_15pct = bool(expected_passes)

    checks = qual.get("checks", []) if isinstance(qual, dict) else []
    if (
        not isinstance(qual, dict)
        or qual.get("schema") != QUAL_BATTERY_SCHEMA
        or qual.get("status") != "PASS"
        or [c.get("name") for c in checks if isinstance(c, dict)] != QUAL_REQUIRED_CHECK_NAMES
        or not all(isinstance(c, dict) and c.get("passed") is True for c in checks)
    ):
        faults.append("qualification_five_check_battery_failed")

    expected_report_status = (
        "QUALIFICATION_PASS_READY_FOR_SEPARATE_DOSE_D40_PLAN"
        if passes_15pct
        else "STOP_OVER_15_PERCENT_GATE_COST"
    )
    if report.get("status") != expected_report_status:
        faults.append("qualification_report_status_mismatch")

    if faults:
        return {
            "status": "FAIL",
            "verdict": "INSTRUMENT_UNREACHABLE_D31",
            "mechanism_negative": False,
            "keep": False,
            "faults": faults,
        }

    if not passes_15pct:
        return {
            "status": "DIAGNOSTIC_STOP",
            "verdict": "STOP_OVER_15_PERCENT_GATE_COST",
            "mechanism_negative": False,
            "keep": False,
            "admission_sha256": admission_sha,
            "projected_added_gate_ns": cal["projected_added_gate_ns"],
            "projected_ungated_two_arm_dose_ns": cal["projected_ungated_two_arm_dose_ns"],
            "faults": [],
        }

    return {
        "status": "PASS",
        "verdict": "QUALIFICATION_PASS_READY_FOR_SEPARATE_DOSE_D40_PLAN",
        "mechanism_negative": False,
        "keep": False,
        "admission_sha256": admission_sha,
        "projected_added_gate_ns": cal["projected_added_gate_ns"],
        "projected_ungated_two_arm_dose_ns": cal["projected_ungated_two_arm_dose_ns"],
        "checks_passed": len(checks),
        "faults": [],
    }


def _audited_dose_review_payload_sha256(plan: dict[str, Any]) -> str:
    """Replicate `attention_audited::review_payload_sha256`."""
    payload = copy.deepcopy(plan)
    payload.pop("independent_review", None)
    payload["config"] = canonical_driver_config(payload["config"])
    return hashlib.sha256(
        json.dumps(payload, separators=(",", ":"), sort_keys=True).encode("utf-8")
    ).hexdigest()


def _audited_dose_cost_payload_sha256(decision: dict[str, Any]) -> str:
    """Replicate `attention_audited::cost_payload_sha256`."""
    payload = copy.deepcopy(decision)
    payload.pop("owner_cost_approval", None)
    payload.pop("pm_dispatch", None)
    return hashlib.sha256(
        json.dumps(payload, separators=(",", ":"), sort_keys=True).encode("utf-8")
    ).hexdigest()


def verify_integrated_audited_dose_artifacts(bundle: dict[str, Any]) -> dict[str, Any]:
    """Consumer-verify an integrated #2782 audited dose bundle (`attention_audited.rs` & `attention_audit.rs`)."""
    faults: list[str] = []
    if not isinstance(bundle, dict):
        return {
            "status": "FAIL",
            "verdict": "INSTRUMENT_UNREACHABLE_D31",
            "mechanism_negative": False,
            "keep": False,
            "faults": ["audited_dose_bundle_not_a_dict"],
        }

    cost_dec = bundle.get("cost_decision")
    cost_atts = bundle.get("cost_attestations", {})
    arms = bundle.get("arms", {})
    if not (isinstance(cost_dec, dict) and isinstance(cost_atts, dict) and isinstance(arms, dict)):
        return {
            "status": "FAIL",
            "verdict": "INSTRUMENT_UNREACHABLE_D31",
            "mechanism_negative": False,
            "keep": False,
            "faults": ["missing_cost_decision_attestations_or_arms"],
        }

    # 1. Verify CostedDoseDecision & 2 DoseCostAttestations (Casey + Kimi)
    expected_cost_payload_sha = _audited_dose_cost_payload_sha256(cost_dec)
    if (
        cost_dec.get("schema") != AUDITED_DOSE_COST_DECISION_SCHEMA
        or cost_dec.get("status") != "COSTED_DOSE_READY_FOR_SEPARATE_D40_RELEASE"
        or cost_dec.get("scope") != AUDITED_DOSE_SCOPE_PREFIX
        or cost_dec.get("calibration_passes_15_percent_gate") is not True
        or cost_dec.get("projected_audits") != 516
        or cost_dec.get("projected_restore_events") != 514
        or not is_u64(cost_dec.get("projected_added_gate_ns"))
        or not is_u64(cost_dec.get("projected_ungated_two_arm_dose_ns"))
        or (cost_dec["projected_added_gate_ns"] * 100)
        > (cost_dec["projected_ungated_two_arm_dose_ns"] * 15)
        or not is_u64(cost_dec.get("max_runtime_seconds"))
        or cost_dec["max_runtime_seconds"] <= 0
        or not is_u64(cost_dec.get("max_cost_cents"))
        or cost_dec["max_cost_cents"] <= 0
    ):
        faults.append("costed_dose_decision_invalid_or_exceeds_15pct_gate")

    for key, exp_actor, exp_role, exp_verdict in (
        ("owner_cost_approval", "Casey", "owner_cost_scope", "APPROVE_EXACT_COSTED_DOSE_PLAN"),
        ("pm_dispatch", "Kimi", "pm_dispatch", "DISPATCH_EXACT_COSTED_DOSE_PLAN"),
    ):
        att = cost_atts.get(key)
        pin = cost_dec.get(key, {})
        if not isinstance(att, dict) or not isinstance(pin, dict):
            faults.append(f"missing_dose_cost_attestation_{key}")
            continue
        att_sha = hashlib.sha256(
            json.dumps(att, separators=(",", ":")).encode("utf-8")
        ).hexdigest()
        if (
            att.get("schema") != AUDITED_DOSE_COST_ATTESTATION_SCHEMA
            or att.get("actor") != exp_actor
            or att.get("role") != exp_role
            or att.get("verdict") != exp_verdict
            or att.get("scope") != AUDITED_DOSE_SCOPE_PREFIX
            or att.get("reviewed_cost_payload_sha256") != expected_cost_payload_sha
            or not isinstance(att.get("receipt_url"), str)
            or not bool(RECEIPT_URL_RE.match(att["receipt_url"]))
            or pin.get("sha256") != att_sha
        ):
            faults.append(f"dose_cost_attestation_{key}_invalid")

    cost_dec_sha = hashlib.sha256(
        json.dumps(cost_dec, separators=(",", ":")).encode("utf-8")
    ).hexdigest()

    # 2. Verify both arms (relation and frozen_zero)
    arm_Admission_shas: dict[str, str] = {}
    arm_state_dirs: list[str] = []
    arm_qual_pins: list[dict[str, Any]] = []
    arm_anchor_pins: list[dict[str, Any]] = []
    arm_verdicts: dict[str, str] = {}

    for arm_name, is_learn, exp_cfg_sha, exp_lin_sha in (
        (
            "relation",
            True,
            EXPECTED_A8_RELATION_CONFIG_SHA256,
            EXPECTED_INTEGRATED_A8_RELATION_LINEAGE_SHA256,
        ),
        (
            "frozen_zero",
            False,
            EXPECTED_A8_FROZEN_ZERO_CONFIG_SHA256,
            EXPECTED_INTEGRATED_A8_FROZEN_ZERO_LINEAGE_SHA256,
        ),
    ):
        arm_b = arms.get(arm_name)
        if not isinstance(arm_b, dict):
            faults.append(f"{arm_name}_missing_arm_bundle")
            continue
        plan = arm_b.get("plan")
        review = arm_b.get("review")
        admission = arm_b.get("admission")
        owner = arm_b.get("owner")
        res_obs = arm_b.get("resource_observation")
        progress = arm_b.get("progress")
        report = arm_b.get("report")
        if not all(
            isinstance(x, dict)
            for x in (plan, review, admission, owner, res_obs, progress, report)
        ):
            faults.append(f"{arm_name}_missing_required_plan_admission_progress_or_report_objects")
            continue

        exp_scope = f"{AUDITED_DOSE_SCOPE_PREFIX}-{'relation' if is_learn else 'frozen-zero'}-seed1-attempts0..16404"
        ex = plan.get("execution", {})
        if (
            plan.get("schema") != AUDITED_DOSE_PLAN_SCHEMA
            or plan.get("status") != "HELD_OWNER_ACTION_REQUIRED"
            or plan.get("scope") != exp_scope
            or ex.get("driver_source_sha256") != EXPECTED_INTEGRATED_DRIVER_SOURCE_SHA256
            or ex.get("config_sha256") != exp_cfg_sha
            or ex.get("lineage_sha256") != exp_lin_sha
            or ex.get("sheet_sha256") != EXPECTED_A8_SHEET_SHA256
            or plan.get("policy") != EXPECTED_REGISTERED_AUDIT_POLICY
            or plan.get("cost_decision", {}).get("sha256") != cost_dec_sha
        ):
            faults.append(f"{arm_name}_audited_plan_header_execution_or_policy_invalid")

        rev_payload_sha = _audited_dose_review_payload_sha256(plan)
        rev_sha = hashlib.sha256(
            json.dumps(review, separators=(",", ":")).encode("utf-8")
        ).hexdigest()
        if (
            review.get("reviewer") not in {"Claude", "DeepSeek", "AGY"}
            or review.get("verdict") != AUDITED_DOSE_REVIEW_VERDICT
            or review.get("scope") != exp_scope
            or review.get("execution") != ex
            or review.get("reviewed_plan_payload_sha256") != rev_payload_sha
            or not isinstance(review.get("receipt_url"), str)
            or not bool(RECEIPT_URL_RE.match(review["receipt_url"]))
            or plan.get("independent_review", {}).get("sha256") != rev_sha
        ):
            faults.append(f"{arm_name}_audited_plan_independent_review_invalid")

        plan_sha = hashlib.sha256(
            json.dumps(plan, separators=(",", ":")).encode("utf-8")
        ).hexdigest()
        owner_sha = hashlib.sha256(
            json.dumps(owner, separators=(",", ":")).encode("utf-8")
        ).hexdigest()
        owner_pin = admission.get("owner_release", {})
        if (
            admission.get("schema") != AUDITED_DOSE_ADMISSION_SCHEMA
            or admission.get("plan") != plan
            or owner.get("schema") != AUDITED_DOSE_OWNER_SCHEMA
            or owner.get("owner") != "Casey"
            or owner.get("decision") != AUDITED_DOSE_OWNER_DECISION
            or owner.get("scope") != exp_scope
            or owner.get("plan_sha256") != plan_sha
            or owner.get("receipt_url") != f"owner-terminal:{owner_pin.get('path')}"
            or owner_pin.get("sha256") != owner_sha
        ):
            faults.append(f"{arm_name}_audited_owner_or_admission_invalid")

        adm_sha = hashlib.sha256(
            json.dumps(admission, separators=(",", ":")).encode("utf-8")
        ).hexdigest()
        arm_Admission_shas[arm_name] = adm_sha
        arm_state_dirs.append(str(plan.get("run_state_directory", "")).rstrip("/"))
        arm_qual_pins.append(plan.get("qualification_report", {}))
        arm_anchor_pins.append(plan.get("anchor", {}))

        # ResourceObservation cap checks
        if (
            res_obs.get("schema") != AUDITED_DOSE_RESOURCE_OBSERVATION_SCHEMA
            or res_obs.get("scope") != exp_scope
            or res_obs.get("admission_sha256") != adm_sha
            or not is_u64(res_obs.get("observed_elapsed_seconds"))
            or res_obs["observed_elapsed_seconds"] > cost_dec.get("max_runtime_seconds", 0)
            or not is_u64(res_obs.get("observed_cost_cents"))
            or res_obs["observed_cost_cents"] > cost_dec.get("max_cost_cents", 0)
        ):
            faults.append(f"{arm_name}_resource_observation_invalid_or_exceeds_approved_cap")

        # ProgressCheckpoint, DurableRecords, Evaluations, RestoreIntents/Observations, FailedModels, and Report
        if (
            progress.get("schema") != AUDITED_DOSE_CHECKPOINT_SCHEMA
            or progress.get("scope") != exp_scope
            or progress.get("admission_sha256") != adm_sha
            or progress.get("anchor_manifest_sha256") != plan.get("anchor", {}).get("sha256")
            or progress.get("anchor_row_csv_sha256") != EXPECTED_TRAIN_PSUCC_TAIL_ANCHOR_CSV_SHA256
            or progress.get("policy") != EXPECTED_REGISTERED_AUDIT_POLICY
        ):
            faults.append(f"{arm_name}_progress_checkpoint_header_invalid")

        records = arm_b.get("durable_records", [])
        evals = arm_b.get("evaluations", [])
        intents = arm_b.get("restore_intents", [])
        observations = arm_b.get("restore_observations", [])
        failed_models = arm_b.get("failed_models", [])
        if not (isinstance(records, list) and len(records) >= 1 and len(evals) == len(records)):
            faults.append(f"{arm_name}_missing_or_mismatched_durable_records_and_evaluations")
            continue

        breach_count = 0
        for idx, (rec, ev) in enumerate(zip(records, evals)):
            if (
                not isinstance(rec, dict)
                or not isinstance(ev, dict)
                or rec.get("schema") != AUDITED_DOSE_DURABLE_RECORD_SCHEMA
                or ev.get("schema") != AUDITED_DOSE_EVALUATION_SCHEMA
                or rec.get("audit_index") != idx
                or ev.get("audit_index") != idx
                or ev.get("scope") != exp_scope
                or ev.get("admission_sha256") != adm_sha
            ):
                faults.append(f"{arm_name}_audit_record_or_evaluation_{idx}_invalid")
                break
            out_block = rec.get("outcome", {})
            if isinstance(out_block, dict) and out_block.get("kind") == "rejected_whole_interval":
                breach_count += 1

        if (
            len(intents) != breach_count
            or len(observations) != breach_count
            or len(failed_models) != breach_count
        ):
            faults.append(f"{arm_name}_breach_restore_intent_observation_or_failed_model_count_mismatch")
        else:
            for b_idx, (it, ob, fm) in enumerate(zip(intents, observations, failed_models)):
                if (
                    not isinstance(it, dict)
                    or not isinstance(ob, dict)
                    or not isinstance(fm, dict)
                    or it.get("schema") != AUDITED_DOSE_RESTORE_INTENT_SCHEMA
                    or ob.get("schema") != AUDITED_DOSE_RESTORE_OBSERVATION_SCHEMA
                    or fm.get("schema") != AUDITED_DOSE_FAILED_MODEL_SCHEMA
                    or it.get("scope") != exp_scope
                    or ob.get("scope") != exp_scope
                    or fm.get("scope") != exp_scope
                    or it.get("admission_sha256") != adm_sha
                    or ob.get("admission_sha256") != adm_sha
                    or fm.get("admission_sha256") != adm_sha
                    or ob.get("restored_model_storage_sha256") != it.get("last_passing_model_storage_sha256")
                    or ob.get("restored_model_snapshot_sha256") != it.get("last_passing_model_snapshot_sha256")
                    or ob.get("restored_model_epoch") != it.get("last_passing_model_epoch")
                ):
                    faults.append(f"{arm_name}_breach_artifact_{b_idx}_restore_mismatch")
                    break

        if (
            report.get("schema") != AUDITED_DOSE_REPORT_SCHEMA
            or report.get("scope") != exp_scope
            or report.get("admission_sha256") != adm_sha
            or report.get("anchor_manifest_sha256") != plan.get("anchor", {}).get("sha256")
            or report.get("anchor_row_csv_sha256") != EXPECTED_TRAIN_PSUCC_TAIL_ANCHOR_CSV_SHA256
            or report.get("audits_completed") != len(records)
            or report.get("rejected_intervals") != breach_count
            or report.get("training_other_scopes") != "HELD"
        ):
            faults.append(f"{arm_name}_audited_dose_report_invalid")

        if not is_learn and report.get("sensitivity", {}).get("accepted_relation", 0) != 0:
            faults.append("frozen_zero_audited_dose_accepted_relation_nonzero")

        arm_verdicts[arm_name] = str(report.get("status", ""))

    if len(arm_state_dirs) == 2 and arm_state_dirs[0] == arm_state_dirs[1]:
        faults.append("both_arms_share_same_run_state_directory")
    if len(arm_qual_pins) == 2 and arm_qual_pins[0] != arm_qual_pins[1]:
        faults.append("cross_arm_qualification_report_pin_mismatch")
    if len(arm_anchor_pins) == 2 and arm_anchor_pins[0] != arm_anchor_pins[1]:
        faults.append("cross_arm_anchor_pin_mismatch")

    if faults:
        return {
            "status": "FAIL",
            "verdict": "INSTRUMENT_UNREACHABLE_D31",
            "mechanism_negative": False,
            "keep": False,
            "faults": faults,
        }

    if any(v == "RULE_EFFECTIVELY_NOOP" for v in arm_verdicts.values()):
        return {
            "status": "DIAGNOSTIC_STOP",
            "verdict": "RULE_EFFECTIVELY_NOOP",
            "mechanism_negative": False,
            "keep": False,
            "arm_verdicts": arm_verdicts,
            "admission_shas": arm_Admission_shas,
            "faults": [],
        }

    if any(v == "Q1_STOP_FOR_DIAGNOSIS" for v in arm_verdicts.values()):
        return {
            "status": "DIAGNOSTIC_STOP",
            "verdict": "Q1_STOP_FOR_DIAGNOSIS",
            "mechanism_negative": False,
            "keep": False,
            "arm_verdicts": arm_verdicts,
            "admission_shas": arm_Admission_shas,
            "faults": [],
        }

    return {
        "status": "PASS",
        "verdict": "AUDITED_DOSE_COMPLETE_PENDING_PAIRED_DISCRIMINATOR",
        "mechanism_negative": False,
        "keep": False,
        "arm_verdicts": arm_verdicts,
        "admission_shas": arm_Admission_shas,
        "faults": [],
    }


def build_synthetic_train_anchor_manifest() -> dict[str, Any]:
    """Construct a valid 4-row synthetic `TrainAnchorManifest` for self-test 58."""
    win0 = [
        {"token": 10, "occurrence": 1, "record": 0, "version": 0},
        {"token": 20, "occurrence": 2, "record": 0, "version": 0},
        {"token": 10, "occurrence": 3, "record": 0, "version": 0},
    ]
    rows: list[dict[str, Any]] = []
    csv_lines: list[str] = []
    for idx in range(4):
        el = 1049856 + idx + 1
        pos = 3251166 + idx * 10
        tok = 700 + idx
        w = copy.deepcopy(win0)
        w[-1]["occurrence"] = pos - 1
        wpin = compute_original_window_pin(w)
        rows.append(
            {
                "anchor_ordinal": idx,
                "eligible_index": el,
                "target_position": pos,
                "target_token": tok,
                "target_identity": {
                    "token": tok,
                    "occurrence": pos,
                    "record": 25932 + idx,
                    "version": 0,
                },
                "original_window": wpin,
            }
        )
        csv_lines.append(f"{el},{pos},{tok},1\n")
    row_csv_sha = hashlib.sha256("".join(csv_lines).encode("utf-8")).hexdigest()
    corpus_binding = {
        "source": {
            "split": "train",
            "tokens": {
                "path": "/workspace/uor-r4/local/driver/data/mw-balp/train/tokens.u16",
                "sha256": EXPECTED_TRAIN_TOKENS_SHA256,
            },
            "mask": {
                "path": "/workspace/uor-r4/local/driver/data/mw-balp/train/response_mask.u8",
                "sha256": EXPECTED_TRAIN_MASK_SHA256,
            },
            "expected_payload_tokens": 7512797,
            "expected_eligible_targets": 2425925,
        },
        "vocabulary": 4096,
        "bos": {"token": "<|bos|>", "id": 0},
        "payload_tokens": 7512797,
        "mask_bytes": 7512797,
        "mask_selected_positions": 2425925,
        "eligible_targets": 2425925,
        "documents": 60000,
        "window_policy": "response-masked-targets/within-document-predecessor/sliding-prefix/ascending-once/v1",
        "identity_policy": "corpus-SHA-bound/split-prefix2bits/document-ordinal62bits/absolute-token-occurrence/version0/v1",
    }
    payload = {
        "schema": TRAIN_ANCHOR_SCHEMA,
        "selection_policy": TRAIN_ANCHOR_SELECTION_POLICY,
        "row_csv_policy": TRAIN_ANCHOR_ROW_CSV_POLICY,
        "corpus": corpus_binding,
        "policy": {"start_eligible_index": 1049856, "count": 4, "context": 192},
        "rows": rows,
        "row_csv_sha256": row_csv_sha,
    }
    m_sha = hashlib.sha256(json.dumps(payload, separators=(",", ":")).encode("utf-8")).hexdigest()
    out = dict(payload)
    out["manifest_sha256"] = m_sha
    return out


def build_synthetic_audited_qualification_bundle(repo_root: Path) -> dict[str, Any]:
    """Construct a valid finite source qualification bundle matching `attention_audit_qualification.rs` for self-tests 59..61."""
    cfg_dir = repo_root / "docs" / "labs" / "a8-training-configs-2026-10-19" / "evidence"
    rel_cfg = canonical_driver_config(read_json_checked(cfg_dir / "a8-relation-seed1.json"))
    fz_cfg = canonical_driver_config(read_json_checked(cfg_dir / "a8-frozen-zero-seed1.json"))
    ex = {
        "base_source": EXPECTED_ATTENTION_BASE_SOURCE,
        "reviewed_source_commit": "fd409fcdf3bc43d607e768bae6f8cf403aa75901",
        "compiled_source_sha256": EXPECTED_COMPILED_SOURCE_SHA256,
        "driver_source_sha256": EXPECTED_INTEGRATED_DRIVER_SOURCE_SHA256,
        "relation_config_sha256": EXPECTED_A8_RELATION_CONFIG_SHA256,
        "frozen_zero_config_sha256": EXPECTED_A8_FROZEN_ZERO_CONFIG_SHA256,
        "relation_lineage_sha256": EXPECTED_INTEGRATED_A8_RELATION_LINEAGE_SHA256,
        "frozen_zero_lineage_sha256": EXPECTED_INTEGRATED_A8_FROZEN_ZERO_LINEAGE_SHA256,
        "sheet_sha256": EXPECTED_A8_SHEET_SHA256,
        "executable_sha256": "e" * 64,
    }
    plan: dict[str, Any] = {
        "schema": QUAL_PLAN_SCHEMA,
        "status": "HELD_FINITE_QUALIFICATION_ADMISSION_REQUIRED",
        "scope": QUAL_SCOPE,
        "execution": ex,
        "relation_config": rel_cfg,
        "frozen_zero_config": fz_cfg,
        "sheet": {
            "path": "/workspace/uor-r4/docs/harness/prep/m1-attention-a8-address-cells-learning-2026-10-19.md",
            "sha256": EXPECTED_A8_SHEET_SHA256,
        },
        "anchor": {
            "path": "/workspace/uor-r4/local/anchor/train-anchor-manifest.json",
            "sha256": EXPECTED_TRAIN_ANCHOR_MANIFEST_SHA256,
        },
        "qualification_fit": {"path": "/workspace/qual/fit.json", "sha256": "0" * 64},
        "owner_scope_cost": {"path": "/workspace/qual/owner-cost.json", "sha256": "0" * 64},
        "pm_dispatch": {"path": "/workspace/qual/pm.json", "sha256": "0" * 64},
        "independent_review": {"path": "/workspace/qual/review.json", "sha256": "0" * 64},
        "output_directory": "/workspace/uor-r4/runs/qual-20261019",
        "limits": {
            "protocol": QUAL_PROTOCOL,
            "anchor_batches_per_sample": 61,
            "gate_samples": 2,
            "synthetic_apply_calls": 1,
            "synthetic_accepted_updates": 1,
            "synthetic_stale_apply_refusals": 1,
            "production_stale_apply_refusals_per_arm": 1,
            "anchor_evaluations_per_arm": 3,
            "rejected_interval_attempts": 64,
            "throughput_attempts_per_arm": 384,
            "throughput_warmup_attempts_per_arm": 0,
            "projected_audits": 516,
            "projected_restore_events": 514,
            "timer_scope": "synchronized-device-snapshot-seal-fsync-readback-lifecycle-v1",
            "cache_policy": "fresh-device-per-arm-no-unreported-warmup-v1",
            "max_runtime_seconds": 1800,
            "max_cost_cents": 400,
        },
    }
    payload_sha = _qual_plan_payload_sha256(plan)
    auths: dict[str, Any] = {}
    for key, role, actor, verdict in (
        ("qualification_fit", "qualification_fit", "AGY", "TEST FITNESS: FIT"),
        ("owner_scope_cost", "owner_cost_scope", "Casey", "APPROVE_EXACT_FINITE_QUALIFICATION_SCOPE_AND_COST"),
        ("pm_dispatch", "pm_dispatch", "Kimi", "DISPATCH_FINITE_SOURCE_QUALIFICATION_ONLY"),
        ("independent_review", "source_plan_review", "AGY", "APPROVE_EXACT_QUALIFICATION_PLAN"),
    ):
        a_obj = {
            "schema": QUAL_AUTH_SCHEMA,
            "actor": actor,
            "role": role,
            "verdict": verdict,
            "scope": QUAL_SCOPE,
            "execution": ex,
            "reviewed_payload_sha256": payload_sha,
            "receipt_url": "https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6266600001",
        }
        a_sha = hashlib.sha256(json.dumps(a_obj, separators=(",", ":")).encode("utf-8")).hexdigest()
        auths[key] = a_obj
        plan[key]["sha256"] = a_sha

    plan_sha = hashlib.sha256(json.dumps(plan, separators=(",", ":")).encode("utf-8")).hexdigest()
    owner_path = "/workspace/qual/owner-release.json"
    owner = {
        "schema": QUAL_OWNER_SCHEMA,
        "owner": "Casey",
        "decision": QUAL_OWNER_DECISION,
        "scope": QUAL_SCOPE,
        "plan_sha256": plan_sha,
        "receipt_url": f"owner-terminal:{owner_path}",
    }
    owner_sha = hashlib.sha256(json.dumps(owner, separators=(",", ":")).encode("utf-8")).hexdigest()
    admission = {
        "schema": QUAL_ADMISSION_SCHEMA,
        "plan": plan,
        "owner_release": {"path": owner_path, "sha256": owner_sha},
    }
    adm_sha = hashlib.sha256(json.dumps(admission, separators=(",", ":")).encode("utf-8")).hexdigest()

    g_eval_ns = 1_000_000
    step_ns = 1_000
    bound_ns = 10_000
    rest_ns = 200_000
    ack_ns = 5_000
    added_ns = 516 * g_eval_ns + 32808 * step_ns + 516 * bound_ns + 514 * (rest_ns + ack_ns)
    ungated_ns = added_ns * 10  # R = 0.10 <= 0.15

    report = {
        "schema": QUAL_REPORT_SCHEMA,
        "status": "QUALIFICATION_PASS_READY_FOR_SEPARATE_DOSE_D40_PLAN",
        "scope": QUAL_SCOPE,
        "admission_sha256": adm_sha,
        "execution": ex,
        "anchor_manifest_sha256": EXPECTED_TRAIN_ANCHOR_MANIFEST_SHA256,
        "anchor_row_csv_sha256": EXPECTED_TRAIN_PSUCC_TAIL_ANCHOR_CSV_SHA256,
        "anchor_rows": EXPECTED_TRAIN_PSUCC_TAIL_ANCHOR_ROWS,
        "calibration": {
            "schema": QUAL_CALIBRATION_SCHEMA,
            "protocol": QUAL_PROTOCOL,
            "timing_protocol": {
                "schema": QUAL_TIMING_PROTOCOL_SCHEMA,
                "timer_scope": "synchronized-device-snapshot-seal-fsync-readback-lifecycle-v1",
                "cache_policy": "fresh-device-per-arm-no-unreported-warmup-v1",
                "cuda_synchronized_before_and_after": True,
                "fresh_device_per_arm": True,
                "unreported_warmup_evaluations": 0,
                "includes_anchor_batches_and_snapshot_seal": True,
                "includes_restore_fsync_and_readback": True,
                "includes_step_and_boundary_host_bookkeeping": True,
            },
            "anchor_rows": EXPECTED_TRAIN_PSUCC_TAIL_ANCHOR_ROWS,
            "anchor_batches_per_evaluation": 61,
            "gate_evaluation_ns": g_eval_ns,
            "restore_and_FailedModel_preservation_ns": rest_ns,
            "bookkeeping": {
                "schema": QUAL_BOOKKEEPING_SCHEMA,
                "progress_history_entries": QUAL_MAX_PROGRESS_HISTORY,
                "step_bookkeeping_ns": step_ns,
                "boundary_bookkeeping_ns": bound_ns,
                "breach_ack_bookkeeping_ns": ack_ns,
            },
            "relation_step_throughput_ns_per_attempt": 100_000,
            "frozen_zero_step_throughput_ns_per_attempt": 100_000,
            "projected_audits": 516,
            "projected_restore_events": 514,
            "projected_added_gate_ns": added_ns,
            "projected_ungated_two_arm_dose_ns": ungated_ns,
            "passes_15_percent_gate": True,
        },
        "qualification": {
            "schema": QUAL_BATTERY_SCHEMA,
            "status": "PASS",
            "checks": [{"name": name, "passed": True} for name in QUAL_REQUIRED_CHECK_NAMES],
        },
        "training": "HELD",
        "full_dose_started": False,
    }
    return {
        "plan": plan,
        "authorizations": auths,
        "owner": owner,
        "admission": admission,
        "report": report,
    }


def build_synthetic_integrated_audited_dose_bundle(repo_root: Path) -> dict[str, Any]:
    """Construct a valid integrated #2782 audited dose bundle matching `attention_audited.rs` for self-tests 62..64."""
    cfg_dir = repo_root / "docs" / "labs" / "a8-training-configs-2026-10-19" / "evidence"
    rel_cfg = canonical_driver_config(read_json_checked(cfg_dir / "a8-relation-seed1.json"))
    fz_cfg = canonical_driver_config(read_json_checked(cfg_dir / "a8-frozen-zero-seed1.json"))

    cost_dec: dict[str, Any] = {
        "schema": AUDITED_DOSE_COST_DECISION_SCHEMA,
        "status": "COSTED_DOSE_READY_FOR_SEPARATE_D40_RELEASE",
        "scope": AUDITED_DOSE_SCOPE_PREFIX,
        "qualification_report": {
            "path": "/workspace/qual/report.json",
            "sha256": "9" * 64,
        },
        "calibration_passes_15_percent_gate": True,
        "projected_audits": 516,
        "projected_restore_events": 514,
        "projected_added_gate_ns": 655_068_000,
        "projected_ungated_two_arm_dose_ns": 6_550_680_000,
        "max_runtime_seconds": 7200,
        "max_cost_cents": 1600,
        "owner_cost_approval": {"path": "/workspace/dose/owner-cost.json", "sha256": "0" * 64},
        "pm_dispatch": {"path": "/workspace/dose/pm-dispatch.json", "sha256": "0" * 64},
    }
    cost_payload_sha = _audited_dose_cost_payload_sha256(cost_dec)
    cost_atts: dict[str, Any] = {}
    for key, actor, role, verdict in (
        ("owner_cost_approval", "Casey", "owner_cost_scope", "APPROVE_EXACT_COSTED_DOSE_PLAN"),
        ("pm_dispatch", "Kimi", "pm_dispatch", "DISPATCH_EXACT_COSTED_DOSE_PLAN"),
    ):
        att = {
            "schema": AUDITED_DOSE_COST_ATTESTATION_SCHEMA,
            "actor": actor,
            "role": role,
            "verdict": verdict,
            "scope": AUDITED_DOSE_SCOPE_PREFIX,
            "qualification_report_sha256": "9" * 64,
            "reviewed_cost_payload_sha256": cost_payload_sha,
            "receipt_url": "https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6266600002",
        }
        att_sha = hashlib.sha256(json.dumps(att, separators=(",", ":")).encode("utf-8")).hexdigest()
        cost_atts[key] = att
        cost_dec[key]["sha256"] = att_sha

    cost_dec_sha = hashlib.sha256(
        json.dumps(cost_dec, separators=(",", ":")).encode("utf-8")
    ).hexdigest()

    def make_arm(arm_name: str, is_learn: bool, cfg: dict[str, Any], cfg_sha: str, lin_sha: str) -> dict[str, Any]:
        scope = f"{AUDITED_DOSE_SCOPE_PREFIX}-{'relation' if is_learn else 'frozen-zero'}-seed1-attempts0..16404"
        ex = {
            "base_source": EXPECTED_ATTENTION_BASE_SOURCE,
            "reviewed_source_commit": "fd409fcdf3bc43d607e768bae6f8cf403aa75901",
            "source_sha256": EXPECTED_COMPILED_SOURCE_SHA256,
            "kernel_sha256": "8cc15ccb605364e8ae4239a1852fc71a08fcfb747c515269b8cca7eecdbf190e",
            "geometry_sha256": "a" * 64,
            "executable_sha256": "e" * 64,
            "driver_source_sha256": EXPECTED_INTEGRATED_DRIVER_SOURCE_SHA256,
            "config_sha256": cfg_sha,
            "lineage_sha256": lin_sha,
            "sheet_sha256": EXPECTED_A8_SHEET_SHA256,
        }
        plan: dict[str, Any] = {
            "schema": AUDITED_DOSE_PLAN_SCHEMA,
            "status": "HELD_OWNER_ACTION_REQUIRED",
            "scope": scope,
            "execution": ex,
            "config": cfg,
            "sheet": {
                "path": "/workspace/uor-r4/docs/harness/prep/m1-attention-a8-address-cells-learning-2026-10-19.md",
                "sha256": EXPECTED_A8_SHEET_SHA256,
            },
            "anchor": {
                "path": "/workspace/uor-r4/local/anchor/train-anchor-manifest.json",
                "sha256": EXPECTED_TRAIN_ANCHOR_MANIFEST_SHA256,
            },
            "qualification_report": {
                "path": "/workspace/qual/report.json",
                "sha256": "9" * 64,
            },
            "cost_decision": {
                "path": "/workspace/dose/cost-decision.json",
                "sha256": cost_dec_sha,
            },
            "independent_review": {
                "path": f"/workspace/dose/{arm_name}-review.json",
                "sha256": "0" * 64,
            },
            "run_state_directory": f"/workspace/uor-r4/runs/audited-{arm_name}-seed1",
            "policy": copy.deepcopy(EXPECTED_REGISTERED_AUDIT_POLICY),
        }
        rev_payload_sha = _audited_dose_review_payload_sha256(plan)
        review = {
            "reviewer": "AGY",
            "verdict": AUDITED_DOSE_REVIEW_VERDICT,
            "scope": scope,
            "execution": ex,
            "reviewed_plan_payload_sha256": rev_payload_sha,
            "receipt_url": "https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6266600003",
        }
        rev_sha = hashlib.sha256(
            json.dumps(review, separators=(",", ":")).encode("utf-8")
        ).hexdigest()
        plan["independent_review"]["sha256"] = rev_sha
        plan_sha = hashlib.sha256(
            json.dumps(plan, separators=(",", ":")).encode("utf-8")
        ).hexdigest()
        owner_path = f"/workspace/dose/{arm_name}-owner.json"
        owner = {
            "schema": AUDITED_DOSE_OWNER_SCHEMA,
            "owner": "Casey",
            "decision": AUDITED_DOSE_OWNER_DECISION,
            "scope": scope,
            "plan_sha256": plan_sha,
            "receipt_url": f"owner-terminal:{owner_path}",
        }
        owner_sha = hashlib.sha256(
            json.dumps(owner, separators=(",", ":")).encode("utf-8")
        ).hexdigest()
        admission = {
            "schema": AUDITED_DOSE_ADMISSION_SCHEMA,
            "plan": plan,
            "owner_release": {"path": owner_path, "sha256": owner_sha},
        }
        adm_sha = hashlib.sha256(
            json.dumps(admission, separators=(",", ":")).encode("utf-8")
        ).hexdigest()

        base_storage = ("1" if is_learn else "2") * 64
        base_snap = ("3" if is_learn else "4") * 64
        failed_storage = ("5" if is_learn else "6") * 64
        failed_snap = ("7" if is_learn else "8") * 64

        records = [
            {
                "schema": AUDITED_DOSE_DURABLE_RECORD_SCHEMA,
                "audit_index": 0,
                "outcome": {"kind": "baseline_committed", "admitted": 1120},
            },
            {
                "schema": AUDITED_DOSE_DURABLE_RECORD_SCHEMA,
                "audit_index": 1,
                "outcome": {"kind": "rejected_whole_interval", "admitted": 1090, "threshold": 1096},
            },
            {
                "schema": AUDITED_DOSE_DURABLE_RECORD_SCHEMA,
                "audit_index": 2,
                "outcome": {"kind": "interval_committed", "admitted": 1115, "threshold": 1096},
            },
        ]
        evals = [
            {
                "schema": AUDITED_DOSE_EVALUATION_SCHEMA,
                "scope": scope,
                "admission_sha256": adm_sha,
                "audit_index": i,
                "anchor_admitted": 1120 if i == 0 else (1090 if i == 1 else 1115),
            }
            for i in range(3)
        ]
        intents = [
            {
                "schema": AUDITED_DOSE_RESTORE_INTENT_SCHEMA,
                "scope": scope,
                "admission_sha256": adm_sha,
                "audit_index": 1,
                "last_passing_model_storage_sha256": base_storage,
                "last_passing_model_snapshot_sha256": base_snap,
                "last_passing_model_epoch": 0,
            }
        ]
        observations = [
            {
                "schema": AUDITED_DOSE_RESTORE_OBSERVATION_SCHEMA,
                "scope": scope,
                "admission_sha256": adm_sha,
                "audit_index": 1,
                "restored_model_storage_sha256": base_storage,
                "restored_model_snapshot_sha256": base_snap,
                "restored_model_epoch": 0,
            }
        ]
        failed_models = [
            {
                "schema": AUDITED_DOSE_FAILED_MODEL_SCHEMA,
                "scope": scope,
                "admission_sha256": adm_sha,
                "audit_index": 1,
                "failed_model_storage_sha256": failed_storage,
                "failed_model_snapshot_sha256": failed_snap,
            }
        ]
        progress = {
            "schema": AUDITED_DOSE_CHECKPOINT_SCHEMA,
            "scope": scope,
            "admission_sha256": adm_sha,
            "anchor_manifest_sha256": EXPECTED_TRAIN_ANCHOR_MANIFEST_SHA256,
            "anchor_row_csv_sha256": EXPECTED_TRAIN_PSUCC_TAIL_ANCHOR_CSV_SHA256,
            "policy": copy.deepcopy(EXPECTED_REGISTERED_AUDIT_POLICY),
        }
        res_obs = {
            "schema": AUDITED_DOSE_RESOURCE_OBSERVATION_SCHEMA,
            "scope": scope,
            "admission_sha256": adm_sha,
            "observed_elapsed_seconds": 1200,
            "observed_cost_cents": 300,
        }
        report = {
            "schema": AUDITED_DOSE_REPORT_SCHEMA,
            "status": "COMPLETE_PENDING_REGISTERED_DISCRIMINATOR",
            "scope": scope,
            "admission_sha256": adm_sha,
            "anchor_manifest_sha256": EXPECTED_TRAIN_ANCHOR_MANIFEST_SHA256,
            "anchor_row_csv_sha256": EXPECTED_TRAIN_PSUCC_TAIL_ANCHOR_CSV_SHA256,
            "audits_completed": 3,
            "rejected_intervals": 1,
            "sensitivity": {"accepted_relation": 4 if is_learn else 0},
            "training_other_scopes": "HELD",
        }
        return {
            "plan": plan,
            "review": review,
            "owner": owner,
            "admission": admission,
            "resource_observation": res_obs,
            "progress": progress,
            "durable_records": records,
            "evaluations": evals,
            "restore_intents": intents,
            "restore_observations": observations,
            "failed_models": failed_models,
            "report": report,
        }

    return {
        "cost_decision": cost_dec,
        "cost_attestations": cost_atts,
        "arms": {
            "relation": make_arm(
                "relation",
                True,
                rel_cfg,
                EXPECTED_A8_RELATION_CONFIG_SHA256,
                EXPECTED_INTEGRATED_A8_RELATION_LINEAGE_SHA256,
            ),
            "frozen_zero": make_arm(
                "frozen_zero",
                False,
                fz_cfg,
                EXPECTED_A8_FROZEN_ZERO_CONFIG_SHA256,
                EXPECTED_INTEGRATED_A8_FROZEN_ZERO_LINEAGE_SHA256,
            ),
        },
    }


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        description="Host-only consumer verifier for A8 address-cells learning dose, pre-run gates, 65+258 checkpoint census/attribution, and admission-preserving active interval rejection."
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
        "--verify-census",
        action="store_true",
        help="Verify the 65 Relation + 258 FrozenZero checkpoints, steps.jsonl slot bijection, step-175 Gate(21) bifurcation, and two-sided + 4-way P_succ admission decomposition.",
    )
    parser.add_argument(
        "--census-archive-dir",
        type=Path,
        default=None,
        help="Optional path to vast-56670997 uor-r4 archive directory containing runs.manifest.sha256, q1.json, and report.json.",
    )
    parser.add_argument(
        "--relation-checkpoints-dir",
        type=Path,
        default=None,
        help="Optional path to directory containing 65 Relation checkpoint-*.json files.",
    )
    parser.add_argument(
        "--frozen-checkpoints-dir",
        type=Path,
        default=None,
        help="Optional path to directory containing 258 FrozenZero checkpoint-*.json files.",
    )
    parser.add_argument(
        "--steps-dir",
        type=Path,
        default=None,
        help="Optional path to directory containing runs/claude/a8-dose-{relation,frozen-zero}-seed1/steps.jsonl.",
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
        "--verify-admission-preserving",
        type=Path,
        default=None,
        help="Optional JSON bundle of admission-preserving active interval rejection receipts to grade.",
    )
    parser.add_argument(
        "--verify-qualification-bundle",
        type=Path,
        default=None,
        help="Optional JSON bundle of finite audited source qualification receipts to grade.",
    )
    parser.add_argument(
        "--verify-audited-dose-bundle",
        type=Path,
        default=None,
        help="Optional JSON bundle of integrated #2782 audited dose receipts to grade.",
    )
    parser.add_argument(
        "--allow-fault-exit-zero",
        action="store_true",
        help="Return exit code 0 even when --prerun-bundle fails or --dose-bundle grades INSTRUMENT_FAULT.",
    )
    parser.add_argument(
        "--self-test",
        action="store_true",
        help="Run the 64-case positive and negative self-test suite.",
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

    if (
        args.verify_census
        or args.census_archive_dir is not None
        or args.relation_checkpoints_dir is not None
        or args.frozen_checkpoints_dir is not None
        or args.steps_dir is not None
    ):
        out["census_and_attribution"] = verify_a8_census_and_attribution(
            census_archive_dir=args.census_archive_dir,
            relation_checkpoints_dir=args.relation_checkpoints_dir,
            frozen_checkpoints_dir=args.frozen_checkpoints_dir,
            steps_dir=args.steps_dir,
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

    if args.verify_admission_preserving is not None:
        ap_doc = read_json_checked(
            args.verify_admission_preserving, "admission-preserving bundle"
        )
        ap_res = verify_admission_preserving_guard_bundle(ap_doc)
        out["admission_preserving_verification"] = ap_res
        if (
            ap_res.get("verdict") == "INSTRUMENT_UNREACHABLE_D31"
            and not args.allow_fault_exit_zero
        ):
            exit_code = 1

    if args.verify_qualification_bundle is not None:
        q_doc = read_json_checked(args.verify_qualification_bundle, "qualification bundle")
        q_res = verify_audited_qualification_artifacts(q_doc)
        out["qualification_verification"] = q_res
        if q_res.get("verdict") == "INSTRUMENT_UNREACHABLE_D31" and not args.allow_fault_exit_zero:
            exit_code = 1

    if args.verify_audited_dose_bundle is not None:
        ad_doc = read_json_checked(args.verify_audited_dose_bundle, "audited dose bundle")
        ad_res = verify_integrated_audited_dose_artifacts(ad_doc)
        out["audited_dose_verification"] = ad_res
        if (
            ad_res.get("verdict") == "INSTRUMENT_UNREACHABLE_D31"
            and not args.allow_fault_exit_zero
        ):
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
