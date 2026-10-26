# Pre-Registration Sheet: M1 BVG-K Stage T Batched-Coordinate Training (`#2836`)

- **Date**: 2026-10-25
- **Lab**: `antigravity`
- **Roadmap Item**: `m1-bvg-k-stage-t-training` ([`#2836`](https://github.com/UOR-Foundation/uor-r4/issues/2836), Step 4 in `PLAN.md`)
- **Predecessor Sheets**: supersedes `docs/harness/prep/m1-bvgk-staget-learning-law-DRAFT-2026-10-19.md` (`sha256 = 0d2775e1aa197f0cff577f01f00fc80ae9072cd7cbe11bb79558e35fb0f32e32`) and updates `docs/harness/prep/m1-bvgk-stage-t-training-2026-10-25.md` (`#2835`) by weighing `#2834` (`m1-address-key-full-vector`, `claude-110`) and dropping the unmet `W4` (`#2846`) fallback routing per [`claude-107/DECISION.md`](../../loop/tasks/m1-bvg-k-stage-t-training/claude-107/DECISION.md) and [`antigravity-84/DECISION.md`](../../loop/tasks/m1-bvg-k-stage-t-training/antigravity-84/DECISION.md)
- **Stage C1 & C2 Qualification Receipts**:
  - Stage C1 (`K = 16` scoring parity): `docs/labs/bvgk-rowsk-scoring-2026-10-18/evidence/run1/clean/rowsk-qualification.json` (`sha256 = d5ec5348d64e2abd74a334319d4f78e6ed7b174b5a2fc0bb381505dc46e09e56`, `5,130 / 5,130` parity across 4 cells, `16.25x` scoring speedup)
  - Stage C2 (`propose_block` + `apply_block` exactness): `docs/labs/injective-preflight-and-c2-qualify-2026-10-19/evidence/bvgk-c2-qualify-20261019/report/bvgk-qualification.json` (`sha256 = 2fff942fa2b802e180c9bd87336692730d6beaf22f2c0bac8867afb0dea345f3`, `8 / 8` controls `V0`, `RC`, `UT`, `JR`, `DC`, `V3`, `RF`, `C1R` pass across `3,419` fixture updates, `~8x` effective throughput with joint verify)

## 1. Question, Geometric Construction, and Base Artifact

**Question**: On the qualified A8 address-structured base lineage (`#2831`) under active D40 64-batch whole-interval admission-preserving rollback (`AuditPolicy::registered()`) with straddle-aware block evidence accounting (`crates/uor-r4-discrete/src/stage_t.rs`), does replacing single-coordinate (`K = 1`) proposal/apply with the qualified `BVG-K` (`K = 16`) batched-coordinate block law (`T16`) preserve DEV admission (`>= 4,220 / 14,287`), beat the matched `K = 1` control (`T1`), and beat the unigram table (`1,226 / 14,287`) and then the W2 admission-matched next-word table (`1,138 / 14,287` restricted code table and `1,482 / 14,287` restricted bigram on the baseline, plus the checkpoint-admitted restricted tables `T_restr(A_W)` and `G_restr(A_W)`) on two seeds (`seed 1`, `seed 2`)?

**Geometric construction (CHARTER §3.1–§3.3 compliant)**:
- Each batch draws `K = 16` distinct coordinates of one parameter kind (`Route`, `Combine`, or `Relation`) by advancing the deterministic cursor up to `K` times inside the current kind phase (`crates/uor-r4-discrete/src/{bvgk_block.rs,bvgk_device.rs,bvgk_source.rs}`).
- **One-launch scoring (`score_rows_k`)**: Evaluates all `16` candidate moves (`3` for `Relation` trits `{0, -1, +1}`, `16` E8 orbit / SpiralCore actions for `Route` and `Combine`) across all `K = 16` coordinates on the 64-row carried batch using exact integer rank-margin hinge loss `(8*T + w*A)/(8*w)` (`ObjectivePolicy::NearestAddressMarginV1`). Uses only integer L1/Hamming distances, SpiralCore table lookups, and E8 root actions — zero softmax, log-softmax, softplus, sigmoid, tanh, GELU, or normalized exponential (§3.2), and zero dense matmul or GEMM (§3.3).
- **Candidate set & joint verification (`propose_block`)**: `discrete_bvgk_choose` forms `S_t = { c in [0, K) : eligible[c] and b_c != 0 and L(c, b_c) < L_0 }` and identifies `best = argmin_{c in S_t} L(c, b_c)` (ties to lowest ordinal). When `|S_t| == 0`, no move is committed; when `|S_t| == 1`, `{best}` is selected with no verify forward; when `|S_t| >= 2`, `S_t` is applied to a device scratch clone (`scratch_clone`), the 64 batch windows are re-carried, and `L(S_t)` is evaluated on the device.
- **Monotone block decision & commit (`apply_block`)**: `discrete_bvgk_decide` selects `S_t` iff `L(S_t) < L({best})`, else `{best}`. `discrete_bvgk_apply` commits all selected moves in one epoch increment (`epoch += 1`), guaranteeing `committed_loss <= min_{c in S_t} L(c, b_c) < L_0`.

**Why this test is right now (whole-project chain & weighing `#2834`)**:
1. In `#2834` (`m1-address-key-full-vector`, `claude-110`, PR `#3235`), zero-update `FV8-Env` (`cell_envelope_l1_v1` on `FullVectorV1` table `2c886736…`) passed the Stage 1 admission bar (`1,980 / 3,847` `P_succ` rows vs `A8-Outer`'s `1,351 / 3,847`, `5,709 / 14,287` full-DEV admitted vs `4,244 / 14,287`, `top-8 = 1,126` vs `513`), while checkpoint-0 `top-1` fell from `133` to `118 / 14,287` (`claude-110/REPORT.md:8-11,89-91`), and training and D11 serving under a non-outer metric are refused in code (`zero_update_only`). Meanwhile, on `A8-Outer` (`address-structured-v1.json`, `6600bdb1…`), `4,244 / 14,287` DEV rows at checkpoint 0 (`4,292 / 14,287` at `#2831` `Q1`) already admit the target cell (`~3x` higher than the Stage 2 bars `U = 1,226`, `T_restr(Q4) = 1,138`, `G_restr(Q4) = 1,482`).
2. In `#2831` (`m1-a8-full-dose-rerun`, merged as `dc5f409b0`), `A8-Outer` (`FrozenZeroV1`) under active D40 rollback passed `B3_init` and `Q1` (`batch = 4,096`, `DEV b_3(Full) = 4,292 / 14,287`, `DEV top1_correct = 871 / 14,287`, `6.55x` over initial `133`), then stopped on `consecutive_rejections` because `K = 1` single-coordinate 64-row greedy updates coupled with 64-batch whole-interval rollback discard an entire 64-batch window whenever one move breaches the global 3,847-row anchor floor. Scoring `K = 16` coordinates per batch and committing verified joint blocks directly tests whether `16x` broader per-batch coordinate coverage and multi-coordinate joint moves overcome this `K = 1` bottleneck and clear the Stage 2 bars on two seeds.

## 2. Arms, Seeds, Data Split, and Steps

All arms use the identical A8 address-structured base codebook (`address-structured-v1.json`, `256` occupied cells, `4,096` distinct vectors), `ReadSpec::Relation { active_leaves: 8, candidates_per_leaf: 8 }`, `ObjectivePolicy::NearestAddressMarginV1`, `K_admit = 8`, `64` rows/batch, `context = 192`, `16,404` batches per full dose (`1,049,856` scored target positions), and the active D40 admission-preserving controller (`AuditPolicy::registered()`: `cadence = 64`, `margin = 24`, `minimum_baseline = 500`, `reject_whole_interval = true`, `consecutive_full_rejections = 3`, `3,847`-row TRAIN anchor floor `1,211 - 24 = 1,187`) driven by `crates/uor-r4-discrete/src/stage_t.rs` (`stage-t-run`) with straddle-aware block evidence accounting (`VectorBlock`, `Combine` `0..20`, `Relation` `20..45`, including the straddling read block `16..32` at `o_read % 45 == 16`).

| Arm | Block Size & Law | Relation Policy | Seeds | Role |
|---|---|---|---|---|
| **`T16_fz` (Primary treatment, FrozenZero)** | `K = 16` (`transition_block` via `propose_block` + `apply_block`) | `FrozenZeroV1` | `seed 1`, `seed 2` | Tests `K = 16` block learning on the A8 lineage that passed `Q1` (`871 / 14,287`) in `#2831` |
| **`T16_rel` (Primary treatment, Relation)** | `K = 16` (`transition_block` via `propose_block` + `apply_block`) | `LearnV1` | `seed 1`, `seed 2` | Tests `K = 16` block learning with active relation-gate updates (`LearnV1`) |
| **`T1_fz` / `T1_rel` (Matched `K = 1` controls)** | `K = 1` (`transition` via `rows_v1`) | `FrozenZeroV1` / `LearnV1` | `seed 1` (from `#2831` sealed artifacts), `seed 2` (run if needed for paired contrast) | Isolates the effect of `K = 16` block scoring and joint commit against `K = 1` |
| **Registered non-neural count-table comparators** | Deterministic count tables on `1,049,856` TRAIN targets (`#2845` / `#3179`) | — | Deterministic | Full-panel unigram `U = 1,226 / 14,287`; W2 baseline admission-matched code table `T_restr(Q4) = 1,138 / 14,287` and admission-matched bigram `G_restr(Q4) = 1,482 / 14,287`; plus checkpoint-matched `T_restr(A_W)` and `G_restr(A_W)` restricted to each Stage T checkpoint's admitted rows `A_W` (full-panel `T2 = 6,300 / 14,287` reported for M1 §4.8 reference) |

**Execution steps**:
1. **Step 0 (First-batch GPU instrument qualification)**: Run the five driver/device controls (`AKd`, `V1d`, `KId`, `V8d`, `V9d` in Section 4) via `stage-t-run step0` on the GPU pod before reading any training metric. Any control failure halts immediately as an `instrument` defect (`0` training claims).
2. **Step 1 (Seed-1 paired run to `Q1 = 4,096` and `Q4 = 16,404`)**: Run `T16_fz` and `T16_rel` on `seed 1` with `Q1` evaluation at batch `4,096` and full dose at batch `16,404`.
3. **Step 2 (Seed-2 replication)**: Run `T16_fz` and `T16_rel` on `seed 2` (`16,404` batches) to test two-seed replication against the Stage 2 bars.
4. **Step 3 (Admission-matched table evaluation)**: On each completed seed's final (or last passing) checkpoint, evaluate DEV `top1_correct`, `top8_correct`, `target_admitted_rows`, and the W2 admission-matched comparators (`T_restr(A_W)` and `G_restr(A_W)` restricted to the checkpoint's admitted candidate sets `A_W`, alongside the fixed `U = 1,226`, `T_restr(Q4) = 1,138`, `G_restr(Q4) = 1,482`, and `T2 = 6,300` tables). Keep `fresh_v2` sealed (`fresh_evaluations == 0`) unless a two-seed Stage 2 pass unlocks Step 7 (`#2841`).

## 3. Stage 2 Bars, Minimum Test Power, and Decision Table

- **Minimum Test Power (CHARTER §4.3)**: Two independent seeds (`seed 1`, `seed 2`) on the `14,287`-row DEV panel (`3,847` `P_succ` rows).
- **Noise Bands**:
  - Admission anchor / `P_succ` (`3,847` rows): `SE = 24` rows (`TRAIN anchor` floor `1,211 - 24 = 1,187`; `DEV` full-panel admission bar `G-Adm = 4,244 - 24 = 4,220 / 14,287`; `DEV b_3(P_succ) >= 676 / 3,847`).
  - Paired DEV top-1 contrast (`14,287` rows): `Delta_dev = max(286, ceil(2 * sqrt(n_10 + n_01)))` rows on full DEV (or `>= max(77, ceil(2 * sqrt(n_10 + n_01)))` on `3,847`-row `P_succ`), plus two-seed noise band `2 * max(59, |top1_s1 - top1_s2|)`.
- **Stage 2 Bar (`docs/roadmap/items/m1-bvg-k-block-apply-c2.md` & `m1-bvg-k-stage-t-training.md`)**:
  On **each of two seeds (`seed 1` and `seed 2`)**, the `T16` arm (`T16_fz` or `T16_rel`) achieves:
  1. **Admission kept (`G-Adm`)**: `target_admitted_rows(DEV) >= 4,220 / 14,287` and `b_3(P_succ, DEV) >= 676 / 3,847`.
  2. **Beat the unigram table (`B_unigram`)**: `top1_correct(DEV) > 1,226 / 14,287` (and `>= 1,226 + 118 = 1,344 / 14,287` outside the `2 * SE` binomial half-width on `14,287` rows).
  3. **Beat the admission-matched next-word table (`B_adm_table`)**: `top1_correct(DEV)` beats the W2 admission-matched next-word table on both seeds — specifically beating the W2 baseline admission-matched code table (`1,138 / 14,287`) and admission-matched bigram (`1,482 / 14,287`), and beating the checkpoint-matched restricted code table `T_restr(A_W)` (and tested against `G_restr(A_W)`) on the checkpoint's admitted rows `A_W` (not the `6,300 / 14,287` unrestricted full-panel `T2` table).
  4. **Beat `T1` (`B_block_gain`)**: `top1_correct(T16) - top1_correct(T1) >= max(286, ceil(2 * sqrt(n_10 + n_01)))` on DEV (or `T16` completes with `B_unigram` + `B_adm_table` met where `T1` halted at `871 / 14,287` on `consecutive_rejections`).

| Outcome | Condition Across Two Seeds | Classification & What It Unlocks |
|---|---|---|
| **`PASS_STAGE2_ADM_TABLE` (Stage 2 Pass)** | On both seeds, `G-Adm` (`>= 4,220`) holds, `top1_correct(DEV)` beats unigram (`> 1,226`, `>= 1,344`) and beats the W2 admission-matched tables (`> 1,138` restricted code table, `> 1,482` restricted bigram, and `> T_restr(A_W)`), and beats `T1` | **Mechanism positive for `BVG-K` (`K = 16`) on Stage 2**. Closes `#2836` (`m1-bvg-k-stage-t-training`) and unlocks Step 6 (`m1-export-gap-and-toks`, `#2839`) and Step 7 (`m1-fresh-panel-transfer`, `#2841`). If `top1_correct(DEV) > 6,300` on both seeds, also unlocks Step 8 (`m1-close-saved-model`, `#2837`). |
| **`PARTIAL_UNIGRAM_ONLY` (Beats unigram & `T1`, below admission-matched bigram)** | On both seeds, `G-Adm` holds and `top1_correct(DEV)` beats unigram (`1,226`) and W2 restricted code table (`1,138`) and `T1` (`871`), but falls below the admission-matched bigram (`1,482` or `G_restr(A_W)`) | **Positive step over `K = 1` (`871 -> >1,226`), Stage 2 admission-matched bigram bar not yet met**. Closes `#2836` with exact gap recorded (`fresh_evaluations == 0`); routes next work to registering training on `#2834`'s `FV8-Env` full-vector lineage (`5,709 / 14,287` admitted) and `m1-wider-code-carrier` (`#2847`) (`W4` `#2846` is not unlocked because W2 returned `H_mixed`, not `H_law`). |
| **`DIAGNOSTIC_STOP_CONSECUTIVE_REJECTIONS` or `H_TIE`** | `T16` hits `consecutive_full_rejections = 3` before `Q4` or finishes inside the paired noise band of `T1` (`<= 871 + 286 = 1,157`) with all Step 0 controls `PASS` | **Configuration negative on `K = 16` greedy 64-row block proposals under 64-batch rollback** (`mechanism_negative: false` for discrete attention as a whole). Closes `#2836` on its recorded two-seed measurement (`fresh_evaluations == 0`) and routes next work to registering training on `#2834`'s `FV8-Env` lineage and `m1-wider-code-carrier` (`#2847`) (`W4` `#2846` remains gated on `H_law` per `PLAN.md:58,93`). |

## 4. First-Batch Instrument Controls (Covering C2 Uncovered Items)

Stage C2 (`2fff942fa2b802e180c9bd87336692730d6beaf22f2c0bac8867afb0dea345f3`) left three items outside its fixture scope (`docs/labs/injective-preflight-and-c2-qualify-2026-10-19/README.md` §"What this does not prove"). Stage T covers all three as **mandatory Step 0 GPU controls before any training result is read**:

1. **`AKd` (BVG-K `apply_block` vs ordinary single-move `Device::apply` kernel)**: Across the first `32` single-move commits (`|commit_ordinals| == 1`) of `T16`, a shadow device restored from the pre-commit snapshot commits the same coordinate through the ordinary `K = 1` `Device::apply` (`discrete.cu`, via `driver.transition`); `table`, `routes`, `gates`, `epoch`, and snapshot SHA-256 match byte-for-byte (`0` mismatches).
2. **`V1d` (`K = 1` identity with today's `transition` law)**: Across the first `24` batches on the production A8 configuration, `transition_block` at `K = 1` on a shadow device matches the `T1` arm's `transition` bitwise in state hash, `epoch`, `accepted_updates`, and schedule cursor (`0` mismatches).
3. **Driver-level controls (`KId`, `V8d`, `V9d`)**:
   - **`KId` (block-transition accounting)**: On every `T16` batch, `model.epoch() == accepted_updates`, `committed_moves == sum(moves)`, `|commit_ordinals| == moves`, a fresh device score on the post-commit state equals `committed_loss < l_before` on every commit, and an all-ineligible batch is an exact no-op (`0` moves, state and epoch unchanged).
   - **`V8d` (`FrozenZeroV1` mask integrity)**: On `T16_fz`, all relation trits remain `0` at every audit and checkpoint, and `0` relation ordinals are ever committed.
   - **`V9d` (checkpoint and resume bitwise parity)**: Restoring the batch-`64` checkpoint of `T16` and running `64` additional batches to batch `128` matches an uninterrupted `128`-batch run bitwise in snapshot SHA-256, `epoch`, cursor, and `committed_moves`.

## 5. Data, Checkpoints, and Qualification SHA-256 Pins

- **Base Codebook (`A8` address-structured cells)**: `address-structured-v1.json` (built deterministically from TRAIN via `address-cells-codebook` and pinned in `crates/uor-r4-discrete/src/attention_learning.rs` and `docs/labs/m1-a8-registration-and-runbook-staging/README.md`; `file_sha256 = 6600bdb1306dbaea4d77f5bf021e717a6bef791e44c1ec7d7c9bb9f0e8cd3cd1`, `table_sha256 = 4122cbab44bd88658743984fc1ab79059bf032c3e2fe526ffcf4ea5f95d9cd74`, `B3_init(P_succ) = 1,351 / 3,847`, `B3_init(Full) = 4,244 / 14,287`, `TRAIN anchor = 1,211 / 3,847`).
- **Train & DEV Corpora**:
  - TRAIN tokens (`mw-balp/train/tokens.u16`): `3dee858f84b750f507ceb3dc3f393711741950954735efaaf61bf704c017a4c8`; TRAIN mask (`response_mask.u8`): `b5e7352d67d6883a833155a5fd397a03f6d4f1b2649e3d4ec3e2ea7d95b949e3`.
  - DEV tokens (`ft/dev/tokens.u16`): `0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b`; DEV mask (`response_mask.u8`): `a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078` (`14,287` rows, `3,847` `P_succ` rows).
  - Tokenizer (`tokenizer.json`): `d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89`.
- **TRAIN Admission Anchor (`3,847` rows)**: Canonical `uor-r4.train-copy-successor-anchor/v1` (`crates/uor-r4-discrete/src/attention_anchor.rs` and `docs/labs/admission-anchor-controller-2026-10-19/README.md`; `row_csv_sha256 = 8c1b80e9b15e4e2bcc765037afa7800b785418be3e20b8cc80ce13ee2e32b3c1`, `manifest_sha256 = f85466ff339d8fb848fb1151a7d672fba5687296b81b1ad6fb39c36ddaca27de`).
- **Inherited `#2831` `K = 1` Seed-1 Baselines & Checkpoints**:
  - `Relation` seed-1 config `f223d78f45fe26dee4080ae7ca9a5b9ff12d9ef0df847375671d52db733fbecd`, initial checkpoint `b5b7af6ebedc1272dc61c84c61248a52838d01db5c29dc7304190caaa392c107`, `#2831` report `eb02da3b46c72591e062025a7f1ecb0fecc8e493947d8efc7fd4e3e1e92ecbc1`.
  - `FrozenZero` seed-1 config `c3fbffe08d13008b8b7ab400a4ef946f2c49632906d88e431d5e80314b9f5536`, initial checkpoint `8465057c88157a4928806eb05ab61d39d0ec6a81000011644914c1d26161e35b`, `#2831` `q1-dev.json` (`top1 = 871 / 14,287`) `6b924afa04103467814b6c71773f7df2c7e6df342f045d83434d3dafe8372943`, `#2831` report `ca5f946945763142fa45e461014a7cc765cb4b53b5850143013a61ed8049a421`.
- **W2 Capacity & Utilization Baselines (`#2845` / `#3179`) & `#2834` Pre-Flight Receipt**:
  - `caseyallard/uor-r4-store:results/claude/w2-capacity-2026-10-25/record.json` (`sha256 = 69717971a6f4e6f4a6dede6ad5eee630c4abdbda13e4c8dd434ef126a74c1abc`, summarized in `docs/labs/m1-w2-capacity-capture/README.md`).
  - `caseyallard/uor-r4-store:results/claude/w2-utilization-2026-10-25/record.json` (`sha256 = d9aa6cd1a9a46876d4e50f1143d481e810fdcfb216beeccf6422ce1b7d70db90`, summarized in `docs/labs/m1-w2-code-utilization/README.md`).
  - `docs/labs/m1-address-key-full-vector/grade.json` (`claude-110`, `#2834` zero-update `PASS`: `FV8-Env` `1,980 / 3,847` `P_succ`, `5,709 / 14,287` full-DEV admitted, `top-1 = 118`, `top-8 = 1,126`).
- **Fresh V2 Transfer Slice**: Remains sealed (`fresh_evaluations == 0`) throughout Stage T.

## 6. Compute Cost and Stop Conditions

- **Pod & Fleet Caps**: `2x NVIDIA GeForce RTX 5090` (or `2x` single RTX 5090 pods) leased via `scripts/pod/uor-pod` inside the project caps (`<= 4` pods and `<= $8/h` across all labs, typically `~$1.80–$2.10/h` for `2x RTX 5090`).
- **Expected Duration & Cost Estimate (`~$12` for two seeds)**:
  - At `~150–260 ms/batch` for `K = 16` (`score_rows_k` + scratch re-carry/verify when `|S_t| >= 2`) plus `258` 64-batch `A_train` audits (`~4.4 s` each), one `16,404`-batch arm takes `~1.0–1.5 GPU-hours`.
  - Running `T16` (`T16_fz` and `T16_rel`) across **two seeds** (`4` arm-seeds total, run in parallel across `2` RTX 5090 GPUs) plus Step 0 controls and DEV / W2-restricted-table evaluations takes `~4–6 GPU-hours` (`~2–3` wall-clock hours on `2x RTX 5090`), or **about `$6–$12` total (`~$12` carried estimate, hard budget cap `$15.00`)**.
- **Stop Conditions**:
  1. **Step 0 instrument failure**: Stop immediately before batch 1 if any of `AKd`, `V1d`, `KId`, `V8d`, or `V9d` fails (`instrument` fix required; `0` training claims).
  2. **`Q1` admission or consecutive-rejection diagnostic stop**: Stop an arm if `DEV target_admitted_rows < 4,220` or `DEV b_3(P_succ) < 676` at `Q1` (`batch = 4,096`), or if `consecutive_full_rejections = 3` (`AuditPolicy::registered()`) triggers.
  3. **Completion**: Stop on completion of Seed 1 and Seed 2 (`batch = 16,404`), seal all checkpoints and reports to `caseyallard/uor-r4-store`, and tear down the pod (`uor-pod down`).

## 7. What This Leaves Uncovered

- **Full-vector metric training (`#2834` follow-up)**: Stage T trains on the registered `A8-Outer` (`OuterL1V1`) lineage (`4,244–4,292 / 14,287` DEV admitted); training under `cell_envelope_l1_v1` (`FV8-Env`, `5,709 / 14,287` DEV admitted) requires its own registration and exporter binding.
- **Full-panel `T2` (`6,300 / 14,287`) vs Stage 2 admission-matched bar**: Beating the Stage 2 bar (`unigram = 1,226` and the W2 admission-matched next-word table `1,138` / `1,482`) proves `BVG-K` (`K = 16`) learns effective within-shortlist ranking on two seeds, whereas closing M1 (`#2837`, CHARTER §4.8) requires beating the full-panel `T2` successor table (`6,300 / 14,287`).
- **Cross-kind joint blocks**: `BVG-K` scores and commits `K = 16` coordinates of a **single parameter kind** per batch (`Route`, `Combine`, or `Relation`); joint blocks combining a `Route` move and a `Combine` vector move in the same epoch remain outside `BVG-K` v1.

## 8. Review Verdict

TEST FITNESS: FIT - M1 #2029 m1-bvg-k-stage-t-training
sheet: docs/harness/prep/m1-bvg-k-stage-t-training-2026-10-25.md
reviewer: antigravity (fresh-context non-author reviewer f96e37ea-4794-4e07-b3f4-2ad7b21a5f3c)
note: https://github.com/UOR-Foundation/uor-r4/issues/2836#issuecomment-6401853483
date: 2026-10-25
A FIT verdict clears compute for the configuration in that sheet only (D22).

Every mechanism pre-registration gets a TEST FITNESS review on its milestone before compute.
