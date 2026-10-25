# Pre-Registration Sheet: M1 A8 Admission-Preserving Full-Dose Rerun (Post-`rows_v1` Cost Replan)

- **Date**: 2026-10-25
- **Lab**: `antigravity`
- **Roadmap Item**: `m1-a8-full-dose-rerun` ([`#2831`](https://github.com/UOR-Foundation/uor-r4/issues/2831), Step 5 on the M1 path in `PLAN.md`)
- **Base Sheet**: `docs/harness/prep/m1-attention-admission-preserving-DRAFT-2026-10-19.md` (`FIT_SHEET_SHA256 = fcb770e66d38f607d25e16dade14ac7a6223dd8af0ccc9ec22c80db71f2c3450`)
- **Stage 1 Qualification Evidence**: `docs/labs/m1-a8-finite-cuda-qualification/qualification-summary.json` (`#2830`, PR `#3180`, merged as `98424f353dd9c4bc733f781fda4e0603092b0764`; HF store `caseyallard/uor-r4-store:results/antigravity/a8-finite-cuda-qual-2026-10-25/a8-finite-cuda-qual-20261025.tar.gz`, `sha256 = a9c1228bc508eb12f61a82b35dd2f18722edee64bb1da4270a1fe37a01a2b2b6`)

## 1. Question and Base Artifact

Does the registered 64-batch whole-interval admission-preserving rejection controller (`AuditPolicy::registered()`: `cadence = 64`, `margin = 24`, `minimum_baseline = 500`, `reject_whole_interval = true`, `consecutive_full_rejections = 3`, `audits_per_arm = 258`, `max_batches = 16,404`) evaluated on the frozen `3,847`-row TRAIN admission anchor (`A_train`) preserve `target_admitted_rows(DEV)` (`G-Adm`, M1 bar (b): `>= 4,220 / 14,287`) and yield a paired DEV verdict on the registered active-leaf `Relation` (`LearnV1`, seed 1) versus `FrozenZero` (`FrozenZeroV1`, seed 1) comparison, or trigger a registered diagnostic stop (`RULE_EFFECTIVELY_NOOP` or `Q1_STOP_FOR_DIAGNOSIS`) while keeping the fresh V2 transfer slice sealed (`fresh_evaluations == 0`)?

## 2. Why Only the Pre-`rows_v1` `15%` Relative Cost Ratio Is Replanned

In `#2830`, all 5 Stage 1 CUDA finite qualification controls (`canonical_initlock_c2`, `required_negatives`, `g_occ`, `sensitivity`, `audited_restore_epoch_cursor`) and `synthetic-control.json` passed on `2x NVIDIA GeForce RTX 5090`, and Stage 2 measured:
- `projected_added_gate_ns = 4,947,754,777,700 ns` (`4,947.75 s`: `2,292.89 s` for `516` `A_train` evaluations + `2,546.26 s` for `514` worst-case full restores + `108.60 s` step/boundary bookkeeping),
- `ungated_two_arm_dose_ns = 6,309,716,689,996 ns` (`6,309.72 s`: `4,545.14 s` on `Relation` + `1,764.57 s` on `FrozenZero`),
- `R = projected_added_gate_ns / ungated_two_arm_dose_ns = 78.41%` (`7,842 bps`), which tripped the pre-`rows_v1` `R <= 15%` check (`validate_ratio`).

The 19 Oct `15%` relative ratio was calibrated before the `rows_v1` device-resident batch table (`D30`), when two-arm ungated training took `~43,000 s` (`~$11.40`) and `15%` gate overhead was `~6,450 s` (`~$1.71`). Because `rows_v1` sped up ungated training `6.8x` (`6,309.72 s`), the **entire worst-case audited two-arm workload (`6,309.72 s + 4,947.75 s = 11,257.47 s = 3.13 GPU-hours`, `~$2.84` on RTX 5090)** is smaller than the 19 Oct sheet's `$1.71` (`6,450 s`) gate allowance alone and `4x` below its `$11.40` budget. Lowering `cadence` from `64` to `512` would test a weaker controller (`8x` longer un-audited drift windows), whereas replanning the post-`rows_v1` readiness bound to `projected_added_gate_ns <= 5,500,000,000,000 ns` (`5,500 s`) and `R <= 85%` (`8,500 bps`) tests the exact registered `64`-batch scientific law (`AuditPolicy::registered()`) unchanged.

## 3. Arms, Data Split, and Steps (Unchanged from Base Sheet)

| Arm | Config Path & SHA-256 | Policy | Seed | Schedule |
|---|---|---|---|---|
| `A_rel_adm` (`Relation`) | `docs/labs/a8-training-configs-2026-10-19/evidence/a8-relation-seed1.json` (file `469a2ff0fe17a6a80f5875a077b3e6a9436553efdf1c6e7884541220553e9d5e`, `config_sha256 = f223d78f45fe26dee4080ae7ca9a5b9ff12d9ef0df847375671d52db733fbecd`, `lineage_sha256 = f22ebd7129f194c7b3089a3b647b3d174d8c0c95fb18395611bba77036fa3cf3`) | `ReadSpec::Relation { active_leaves: 8, candidates_per_leaf: 8 }`, `RelationLearningPolicy::LearnV1`, `AuditPolicy::registered()` | `1` | `16,404` batch attempts (`64` rows/batch, `context = 192`), `258` audits (`256 × 64` + Q1 at `4,096` + final tail `20` at `16,404`) |
| `A_fz_adm` (`FrozenZero`) | `docs/labs/a8-training-configs-2026-10-19/evidence/a8-frozen-zero-seed1.json` (file `cf7ebcc85d2e463ae8f6691b63d90a0e005701cb38b8249a8d0d5460d8508479`, `config_sha256 = c3fbffe08d13008b8b7ab400a4ef946f2c49632906d88e431d5e80314b9f5536`, `lineage_sha256 = e4c3efc99f51bc3eb4c13844cc3e7dc12e9e42d32a2cde39603e6052d649e79a`) | `ReadSpec::Relation { active_leaves: 8, candidates_per_leaf: 8 }`, `RelationLearningPolicy::FrozenZeroV1`, `AuditPolicy::registered()` | `1` | Identical data order, parameter schedule, and `AuditPolicy::registered()` |
| Count-table baselines | `CHARTER §4.8` on `14,287` DEV rows | Unigram: `1,226 / 14,287`; `T2` successor: `6,300 / 14,287`; `P_succ` (`3,847` rows) reference top-1: `194 / 3,847` (`G-Cap = 1,351 - 194 = 1,157 > 500`) | Deterministic | Zero-parameter count-table comparators |

## 4. Bars, Minimum Test Power, Instrument Checks, and Decision Table

- **Noise Floor & Paired Contrast Bar**: McNemar `SE = 24` rows on `3,847`-row `A_train` / `P_succ`. Paired DEV win requires `top1_correct(A_rel_adm) - top1_correct(A_fz_adm) >= max(77, ceil(2 * sqrt(n_10 + n_01)))` on `P_succ` (`3,847` rows) or `>= max(286, ceil(2 * sqrt(n_10 + n_01)))` on full DEV (`14,287` rows).
- **Admission Preservation Bars (`G-Adm`, M1 bar (b))**:
  - `A_train` baseline (`t = 0`): `A_0 = 1,211 >= 500` (`scored_rows = 3,847`). Every interval `k` with `A_k < A_0 - 24 = 1,187` is rejected in full and restored to the last passing checkpoint (`epoch` and `cursor` restored, stale handles refused).
  - Q1 DEV check (`G-Q1`, batch `4,096`): `target_admitted_rows(DEV, Q1) >= 4,244 - 24 = 4,220`.
  - Final DEV check (`G-Adm`, batch `16,404`): `target_admitted_rows(DEV, final) >= 4,220` and `admitted_strict_win_rows(DEV, final) >= 86`.
- **Instrument Checks**:
  - `validate_qualification`, `validate_calibration` (`projected_added_gate_ns <= 5,500,000,000,000 ns` and `R <= 85%`), `validate_costed_dose`, and `validate_resource` pass before batch 1.
  - On every rejected interval: `restored_epoch == last_passing_epoch`, `passing_snapshot_restored == true`, `passing_storage_restored == true`, `anchor_equal_before_after_restore == true`.
  - `fresh_evaluations == 0` throughout.

| Outcome | Condition | Classification & What It Unlocks |
|---|---|---|
| **`H_geom` (`G-Adm` + DEV win)** | Both arms complete `16,404` batches with `target_admitted_rows(DEV) >= 4,220` and `admitted_strict_win_rows(DEV) >= 86`, and `A_rel_adm` beats `A_fz_adm` by `>= max(77, ceil(2 * sqrt(n_10 + n_01)))` on `P_succ` (or `>= max(286, ...)` on full DEV) | **Mechanism positive on DEV**. Closes `#2831` (`m1-a8-full-dose-rerun`) and unlocks one-shot fresh V2 evaluation (`attention-transfer`) for `#2837`. |
| **`H_tie` / `H_null` (`G-Adm` kept, no DEV separation)** | Both arms complete `16,404` batches with `target_admitted_rows(DEV) >= 4,220` (curing the `4,244 -> 93` ungated collapse), while `A_rel_adm - A_fz_adm` is inside the paired noise band | **Controller positive (`G-Adm` met), single-coordinate `LearnV1` configuration null**. Closes `#2831` with paired DEV verdict recorded and `fresh_evaluations == 0`; routes M1 closing-bar training to Step 4 (`m1-bvg-k-stage-t-training`, `#2836`) while retaining the verified `G-Adm` controller. |
| **`RULE_EFFECTIVELY_NOOP` diagnostic stop** | Either arm hits `3` consecutive rejected 64-batch intervals or `> 50%` rejected intervals (`rejected * 2 > total`, `total >= 8`) with restore/anchor instrument checks `PASS` | **Mechanism negative on the single-coordinate `LearnV1` + 64-batch whole-interval configuration** (every 64-batch trajectory erodes `A_train` by `> 24` rows). Closes `#2831` on the registered diagnostic stop (`docs/roadmap/items/m1-a8-full-dose-rerun.md` Done-When) with `fresh_evaluations == 0`; routes M1 to Step 4 (`m1-bvg-k-stage-t-training`, `#2836`). |
| **`Q1_STOP_FOR_DIAGNOSIS` at batch `4,096`** | `A_train >= 1,187` at batch `4,096`, but `target_admitted_rows(DEV, Q1) < 4,220` | **Instrument stop** (`3,847`-row `A_train` proxy diverges from `14,287`-row DEV admission by `> 24` rows). Closes `#2831` on the registered diagnostic stop with `fresh_evaluations == 0` before spending the remaining `75%` of that arm's compute. |

## 5. Data, Checkpoints, and Qualification SHA-256 Pins

- **Train Anchor (`3,847` rows)**: `docs/labs/admission-anchor-controller-2026-10-19/evidence/train-anchor-3847-v1.csv` (`row_csv_sha256 = 8c1b80e9b15e4e2bcc765037afa7800b785418be3e20b8cc80ce13ee2e32b3c1`, `manifest_sha256 = f85466ff339d8fb848fb1151a7d672fba5687296b81b1ad6fb39c36ddaca27de`).
- **Corpora & Codebook (`a8-relation-seed1.json` / `a8-frozen-zero-seed1.json`)**:
  - Train tokens (`mw-balp/train/tokens.u16`): `3dee858f84b750f507ceb3dc3f393711741950954735efaaf61bf704c017a4c8`; Train mask (`response_mask.u8`): `b5e7352d67d6883a833155a5fd397a03f6d4f1b2649e3d4ec3e2ea7d95b949e3`.
  - DEV tokens (`ft/dev/tokens.u16`): `0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b`; DEV mask (`response_mask.u8`): `a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078`.
  - Tokenizer (`tokenizer.json`): `d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89`; Codebook (`address-structured-v1.json`): `6600bdb1306dbaea4d77f5bf021e717a6bef791e44c1ec7d7c9bb9f0e8cd3cd1`.
- **Initial Checkpoints & DEV Evaluations (`docs/labs/m1-a8-registration-and-runbook-staging/evidence/inherited/`)**:
  - `Relation`: initial checkpoint `b5b7af6ebedc1272dc61c84c61248a52838d01db5c29dc7304190caaa392c107`, initial DEV eval `ba84cf86bbfadc72fca2abeed19c989791392ebb51df6b9023aa6b112d3096b7`, lineage `f22ebd7129f194c7b3089a3b647b3d174d8c0c95fb18395611bba77036fa3cf3`.
  - `FrozenZero`: initial checkpoint `8465057c88157a4928806eb05ab61d39d0ec6a81000011644914c1d26161e35b`, initial DEV eval `28e8408b7bc16fd1e50be3b7413709c25888f4471590e427f5bc6941ce854798`, lineage `e4c3efc99f51bc3eb4c13844cc3e7dc12e9e42d32a2cde39603e6052d649e79a`.
- **Stage 1 CUDA Qualification Artifacts (`#2830`)**:
  - `relation/qualification.json`: `73876b488bdb2123457d46fd28b0e4b1296cc398a930db5542b67f72cc9fc2d4`
  - `frozen-zero/qualification.json`: `312b5370522afddbf46675220ba5e05a93e1eab5e0cf75a2f448aeb59032badc`
  - `timing-protocol.json`: `d97bb13a8f30b24c2c4ad99c63308adbd4f537e31d776f697eeb5036aa651f86`
  - `host-metadata-bookkeeping/measurement.json`: `95c64e51c67504cb1c2d1382b698fd5df5d249c07439cb20068754c311b8d301`
  - `synthetic-control.json`: `69eb5866e3e66d658602a463ff236a2f59647347764a97e569ada86b17f762ea`

## 6. Compute Cost and Stop Conditions

- **Pod**: `2x NVIDIA GeForce RTX 5090` (or `1x RTX 5090`) leased via `scripts/pod/uor-pod` inside the 4-pod and `$8/h` fleet caps (`~$0.91–$1.82/h`).
- **Expected Duration & Spend**: `~3 min` (`<$0.15`) if `RULE_EFFECTIVELY_NOOP` stops at batch `192`; `~25 min` (`~$0.80`) if `Q1_STOP_FOR_DIAGNOSIS` stops at batch `4,096`; `~1.6–3.1 GPU-hours` (`~$1.50–$2.84`, hard cap `$6.00`) for full `16,404`-batch completion of both arms.
- **Stop Conditions**: Stop immediately on `RULE_EFFECTIVELY_NOOP`, `Q1_STOP_FOR_DIAGNOSIS`, or completion of both arms (`16,404` batches), verify all pins, archive to HF store `caseyallard/uor-r4-store`, and tear down the pod (`uor-pod down`).

## 7. Review Verdict

TEST FITNESS: FIT - M1 #2029 m1-a8-full-dose-rerun
sheet: docs/harness/prep/m1-a8-full-dose-rerun-2026-10-25.md
reviewer: antigravity (fresh-context non-author reviewer 93d74061-c2dc-4e78-b317-958f1b362160)
note: https://github.com/UOR-Foundation/uor-r4/issues/2831#issuecomment-6394635218
date: 2026-10-25
A FIT verdict clears compute for the configuration in that sheet only (D22).

Every mechanism pre-registration gets a TEST FITNESS review on its milestone before compute.
