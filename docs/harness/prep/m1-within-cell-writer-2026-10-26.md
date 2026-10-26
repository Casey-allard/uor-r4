# Pre-Registration Sheet: Within-Cell Writer Error Decomposition & Zero-Update B0 (`m1-within-cell-writer`, `#3267`)

- **Date**: 2026-10-26
- **Lab**: `antigravity` (`antigravity-93` / `antigravity-94`, PR `#3280`)
- **Milestone**: M1 (`#2029`), Step 5 in [`PLAN.md`](../../../PLAN.md)
- **Roadmap Item**: [`docs/roadmap/items/m1-within-cell-writer.md`](../../roadmap/items/m1-within-cell-writer.md) (`#3267`, referencing `#3128` and `#3129`)
- **Order**: [`docs/loop/tasks/m1-within-cell-writer/antigravity-93/ORDER.md`](../../loop/tasks/m1-within-cell-writer/antigravity-93/ORDER.md)

## 1. Question & Scope

On Stage T (`#2836`), `T16_fz` admits `5,009 / 14,287` target cells (`35.06%`), yet `OuterL1V1` selects only `455` `top-1` (`1,484` `top-8`), while exact TRAIN-built count tables restricted to the same admitted cells score `2,140` (`T_restr(A_W)`) and `3,084` (`G_restr(A_W)`). This unit answers two pre-registered questions before any new training campaign:
1. **Stage T Error Decomposition (`Done when` bullet 1)**: How does the `455` vs `2,140` gap on `T16_fz` (and `371` vs `1,602` on `T16_rel`) decompose across cell admission, `top-8` writer candidate coverage, within-cell vs cross-cell L1 ranking, DEV read-code coverage, and a TRAIN-only within-cell reranker (`R_cell(c)`) vs its permuted-code null (`R_cell(c_perm)`)?
2. **Zero-Update Read Experiment `B0` (`Done when` bullets 2–4)**: On the re-derived epoch-0 `FullVectorV1` (`B = 4`) table under `OuterL1V1` and the `14,287` frozen DEV rows, does two-pass endpoint-output framing (`TwoPassEndpointOutputV1`, `#3129`) beat nearest-endpoint single-pass framing (`NearestEndpointSinglePassV1`) on exact 8-bit interior-key accuracy `(r_1, r_2)` over the pre-registered intersection of admitted rows under `ReadLawV1` or `ReadLawV2` (`#3128`), or does the within-cell read-code reranker capture the gap directly?

## 2. Arms (`B0` on Re-Derived Epoch-0 `FullVectorV1`, `B = 4`, `OuterL1V1`, Zero Updates)

All GPU writer arms evaluate the identical `4,096 x 32` `FullVectorV1` table (`2c886736c858778677d6a7b87253af188d8787cedc5fe5acd9b3c899a10a90e2`), `ActiveLeafV1` gates (`gates[b*5+0] = +1`, relation gates `0`), `kept_cells = 8`, `kept = 8`, `sweeps = 1`:
1. **`current_single_pass_v1` / `_v2`**: `f(0)=3, f(1)=0, f(2)=0, f(3)=0`, `fq` from cursor token (`ReadLawV1` / `ReadLawV2`, `RelativeFrameTransportV1`).
2. **`nearest_endpoint_single_pass_v1` / `_v2`**: `f(0)=3, f(1)=0, f(2)=3, f(3)=0`, `fq` from cursor token's nearest endpoint (`ReadLawV1` / `ReadLawV2`, `RelativeFrameTransportV1`).
3. **`twopass_endpoint_output_v1` / `_v2`**: Pass 1 executes `b in {0, 3}`; Pass 2 conditions `b = 1` on `out_0` (`f = 0, fq = out_0`) and `b = 2` on `out_3` (`f = 3, fq = out_3`) (`ReadLawV1` / `ReadLawV2`, `RelativeFrameTransportV1`).
4. **`frozen_constant_frame_v1` / `_v2`**: Interior blocks `b in {1, 2}` hold `fq = 0` constant (`frozen_frame_root = 0`).
5. **`permuted_frame_v1` / `_v2`**: Pass 2 conditions `b in {1, 2}` on a seeded 16-root derangement (`seed = 20261026`, zero fixed points) of Pass-1's `(out_0, out_3)`.
6. **`train_remap_16_v1` / `_v2`**: Single-pass interior blocks remap cursor endpoints `(r_0, r_3)` via a `16 -> 16` transition map fit on the first `900,000` TRAIN rows and validated on the remaining `149,856` holdout TRAIN rows.
7. **`twopass_global_writer_frame_v2`**: `ReadLawV2` + `TwoPassEndpointOutputV1` + `GlobalWriterFrameV1` (`#3128` Item 3: leaf carry in global codebook frame `g[IDENTITY]`).
8. **TRAIN-only lookup & reranker controls on the same admitted cells**: `lookup_t_frozen_restr_v1` (`T_frozen` from frozen `address-structured-v1` codes), `reranker_t_rebuilt_pooled_v1` / `_v2` (`T_rebuilt` from `FullVectorV1` codes), `reranker_t_rebuilt_top1_cell_only_v1` (strictly inside rank-1 cell `slots[0]`), `reranker_t_rebuilt_cell_prio_v1`, `reranker_code_permuted_null_v1` (`seed = 20261026`), `lookup_g_bigram_restr_v1` (`G`), and `lookup_u_unigram_restr_v1` (`U`).

## 3. Bar and Primary Comparison

- **Primary `B0` Comparison**: Paired comparison of `twopass_endpoint_output` vs `nearest_endpoint_single_pass` on **8-bit interior-key accuracy `(r_1, r_2)` over the pre-registered intersection of admitted DEV rows** using the strict paired McNemar rule:
  $$d = b - c > 24 \quad \text{and} \quad d^2 > 4(b + c)$$
  Plus strict non-inertness (`non_inert_rows > 0`) and beating both `frozen_constant_frame` and `permuted_frame` under the same rule.
- **Secondary Readouts (reported separately; do not create extra opportunities to pass `TwoPass`)**: Total `top-1`, intersection-admitted `top-1`, cell admission (`Full` and `P_succ`), TRAIN/DEV read-code coverage, non-inert row counts, cell/token comparison counts (`256` cells, `128` tokens), resident bytes, and GPU decode throughput (`rows/s`, `tok/s`).

## 4. Base artifact and SHA-256 Pins

| Input / Artifact | Path / Store Location | SHA-256 |
|---|---|---|
| TRAIN tokens (`7,512,797` payload, `2,425,925` eligible) | `mw-balp/train/tokens.u16` | `3dee858f84b750f507ceb3dc3f393711741950954735efaaf61bf704c017a4c8` |
| TRAIN response mask | `mw-balp/train/response_mask.u8` | `b5e7352d67d6883a833155a5fd397a03f6d4f1b2649e3d4ec3e2ea7d95b949e3` |
| DEV tokens (`42,856` payload, `14,287` eligible) | `ft/dev/tokens.u16` | `0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b` |
| DEV response mask | `ft/dev/response_mask.u8` | `a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078` |
| Pinned A8 `StructuredV1` table | `address-cells/address-structured-v1.json` | `6600bdb1306dbaea4d77f5bf021e717a6bef791e44c1ec7d7c9bb9f0e8cd3cd1` |
| Re-derived `FullVectorV1` (`B = 4`) raw LE table | Re-derived deterministically from TRAIN | `2c886736c858778677d6a7b87253af188d8787cedc5fe5acd9b3c899a10a90e2` |
| Sealed Stage T `T16_fz` Seed 1 checkpoint (`epoch = 420`) | `t16-frozen-zero-seed1/final-checkpoint.json` | `0831900fc6c04b4489215dc93876f87a173b2c6c16b6635dde2f2ff24c1015a4` |
| Sealed Stage T `T16_rel` Seed 1 checkpoint (`epoch = 678`) | `t16-relation-seed1/final-checkpoint.json` | `558b8810dc888d3acd2d5d135b9137062ce69f74e4148a822e1b59b9cfccb944` |
| Unchanged `driver_source_sha256` | `crates/uor-r4-discrete/src/training.rs` | `04c5165464a18eea6d4edadc6970007dbdc7908ead420f4a2f575e22ea43061d` |

## 5. Data split and Panel

- **Training split (`mw-balp/train`)**: `2,425,925` eligible response rows (`1,049,856` causal windows of length `C = 192` at stride `64`); first `900,000` windows used to fit `TrainRemap16V1`, remaining `149,856` holdout windows used for `TrainRemap16V1` validation; full `1,049,856` TRAIN windows used to build exact read-code count tables (`T_frozen`, `T_rebuilt`, `G`, `U`).
- **Frozen evaluation panel (`ft/dev`)**: `14,287` eligible response targets across `400` documents (`42,856` payload tokens, `224` batches of up to `64` windows of length `C = 192`), with `P_succ = 3,847` rows reported separately.

## 6. Negative Gates & Finite CUDA Instrument Controls (`within-cell-qualification.json`)

Every negative gate is proven failing when its check is set aside (`bypass_refusal_mask`) and passing with exact refusal when active:
1. **`G-Refuse-Blocks`**: Refuses `s[1] != 4` (`status = 15`) with zero state mutation.
2. **`G-Refuse-InertPosition`**: Refuses inert position representation `s[14] >= 3` (`status = 16`).
3. **`G-Refuse-IdentityBypass`**: Refuses `R-Off` route identity bypass `s[13] != 0` (`status = 17`).
4. **`G-Refuse-RelationBound`**: Refuses `|rel[i]| > max_rel_bound` (`status = 14`).
5. **`G-Refuse-InvalidGateTrit`**: Refuses gate values outside `-1..=1` on host and GPU (`status = 10`).
6. **`G-Refuse-ZeroNonInertness`**: Refuses any two-pass configuration with `0` non-inert rows.
- **Finite CUDA Controls `C1–C6`**: `C1` (`256/256` root-pair orbit check: `129/256` V1 vs `256/256` V2 positive vs `0/256` V2 negative), `C2` (multi-level active carry `terms = 5` & antipodal `-1` gates), `C3` (`s[12] = 1` unit-snap & `s[12] = 2` raw residual), `C4` (recent-8 relation cap `rel = 64` & single-copy succession `[3, 1, 2, 3, 1] -> 2`), `C5` (frame-aligned copy succession `[1, 2, 3, 1] -> 2` across distinct frames & 16-token single-cell `FullVectorV1` separation), and `C6` (`16 -> 16` validation remap & bit-exact old-lineage GPU parity against `Device::forward` and `Device::evaluate`).

## 7. Steps

1. Run `within-cell-writer-run qualify` on GPU across two independent release builds (`Build 1` and `Build 2`), verifying all 6 negative gates (`fails_when_set_aside && passes_when_active`) and finite controls `C1–C6`.
2. Run `within-cell-writer-run decompose-stage-t` on the sealed `T16_fz` (`epoch = 420`) and `T16_rel` (`epoch = 678`) artifacts, verifying exact Gate R reproduction (`5,009 / 455 / 1,484` and `3,827 / 371 / 1,124`) and decomposing the `455` vs `2,140` gap into admission, `top-8` coverage, within-cell vs cross-cell L1 ranking, DEV read-code coverage, and TRAIN-only within-cell reranker controls.
3. Run `within-cell-writer-run b0` on the re-derived epoch-0 `FullVectorV1` (`2c886736...`) table under `OuterL1V1` across all 21 writer, frame-ablation, lookup, and within-cell reranker arms on both builds, archive receipts to `caseyallard/uor-r4-store:results/antigravity/m1-within-cell-writer-2026-10-26/`, and tear down the pod.

## 8. Stop conditions and Decision Table

- **Stop rule**: Terminate immediately if any negative gate or `C1–C6` control fails, if Gate R fails to reproduce the sealed Stage T `final-dev.json` / `w2-summary.json` numbers (`5,009 / 455 / 1,484` and `3,827 / 371 / 1,124`), or if `G-Occ` fails on `FullVectorV1`.
- **Decision Table**:
  - If `TwoPass` clears the primary bar over `Nearest` and beats `FrozenConstantFrame` and `PermutedFrame`: register a component-wise `ReadLawV2` + `TwoPass` training sheet with ablations before any D11 work.
  - If `TwoPass` fails to beat `Nearest` (or ties `FrozenConstantFrame` / `PermutedFrame`), while the TRAIN-only within-cell read-code reranker beats both the matched writer and `R_cell(c_perm)` by `d > 24 && d^2 > 4(b+c)`: close only the tested epoch-0 `TwoPass` interior-key framing configuration and advance the exact-memory within-cell writer lookup/reranker mechanism (`#3267` How-to-start step 5).

## 9. Cost

- **Pod**: `1x NVIDIA GeForce RTX 5090` (`sm_120`, `<= $1.50/h`, leased via `scripts/pod/uor-pod`, well inside the 4-pod / `$8/h` fleet cap).
- **Projected Wall Time & Cost**: `~10–15 minutes` total (`< $0.40`).

## 10. Minimum test power

- **Rows**: `14,287` frozen `ft/dev` eligible response rows (`>= 200` fresh rows under D22 §3), with `3,334` commonly admitted intersection rows for the paired interior-key test and `5,009` (`T16_fz`) / `3,827` (`T16_rel`) admitted rows for the Stage T decomposition.
- **Replication**: Two independent release builds (`Build 1` and `Build 2`) verified to have `0` metric diffs across `qualify`, `decompose-stage-t`, and `b0`.

## 11. Verdict

```text
TEST FITNESS: FIT - M1 #2029 m1-within-cell-writer
sheet: docs/harness/prep/m1-within-cell-writer-2026-10-26.md
reviewer: antigravity (non-author; fresh-context adversarial FIT reviewer 0153428c-fcde-40c0-ae3d-ab2d4935455a for #3267 in antigravity-93, posted on #3267 comment 4315460788)
note: Both Stage T error decomposition and B0 zero-update read arms are cleanly specified with exact SHA-256 pins, strict paired McNemar rule d > 24 && d^2 > 4(b+c) on 8-bit interior-key accuracy over the pre-registered admitted intersection, all 6 negative refusal gates proven failing when bypassed and passing when active, finite CUDA controls C1-C6, and an explicit stop rule closing only the epoch-0 TwoPass interior-key framing configuration under D22 if negative while advancing the exact-memory within-cell reranker.
date: 2026-10-26
A FIT verdict clears compute for the configuration in that sheet only (D22).
```
