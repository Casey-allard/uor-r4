# Pre-registration - M2 TRAIN-mask discrete-law sheet (`m2-pin-train-mask-sheet`)

- date: 2026-10-26; item `#2851`; base `bc5cad8da85c5b67de3af9734e9dff4c2bd4e439`
- milestone: M2 `#2030` - Grounded reply from exact memory: the native learner completes correct replies
- mechanism: discrete-geometric-learning (`TrunkAssemblyV1` + `BVG-K` `Stage T` + `OuterL1V1` + `D11` `uor-r4.lut-discrete/1`)
- mechanism brief: [`docs/mechanisms/discrete-geometric-learning.md`](../../mechanisms/discrete-geometric-learning.md)
- companion D37 sheet: [`m2-pin-train-mask-sheet-2026-10-23.md`](m2-pin-train-mask-sheet-2026-10-23.md) (retained for the offline-float D37 branch; sits at `TEST FITNESS: NOT_FIT` until `float-training-module` `#2823` and `d11-exporter-float-trunk` `#2820` land)
- historical antecedents: [`m1-rank-margin-rung0-2026-10-13.md`](m1-rank-margin-rung0-2026-10-13.md) (Amendments 1–6; frozen by Amendment 7 under D32) and [`m1-discrete-rung0-training-2026-10-13.md`](m1-discrete-rung0-training-2026-10-13.md) (14 Oct M1 design record on `e7d926fbe`, superseded here for M2 because its Gates 1–3 are now landed in `#3239`/`#3258`, its fresh transfer is now sealed in `#2849`/`#2844`, and this sheet carries the count-reference source SHAs and M2 bars)
- **owner scope ruling (2026-10-24T03:42Z, [`#2028` comment `6379430093`](https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6379430093)):** *"I want both because we do not know which we will ultimately need."* This sheet was intended to carry the discrete-law branch of [`m2-pin-train-mask-sheet`](../../roadmap/items/m2-pin-train-mask-sheet.md) (`#2851`) without gating the discrete M2 path ([`m2-train-grounded-replies`](../../roadmap/items/m2-train-grounded-replies.md), `#2853`) behind the offline-float modules. The #3303 review found that its execution chain is incomplete, so it does not authorize that path.
- **legal basis (CHARTER §3.1–§3.6, §4.5, D23, D24, D25, D28, D29, D31, D32):** pure integer/table geometry (`Cl(0,6)` signed-permutation actions, 16-root $E_8$ orbit $\mathcal{O}_{16}^4 \subset \{-2, 0, 2\}^{32}$, causal prefix holonomy `PositionHolonomyV1`, `OrbitRootSnapNormV1`, `DiscreteQuaternionRecurrenceV1`, `DiscreteHamiltonGateV1`, `OuterL1V1` writer, and `D11` `uor-r4.lut-discrete/1` integer artifact export with zero float/int export gap); no softmax or squashing (D23), no dense matmul or GEMM (D25), no floating point in discrete-model computation (D32), no CPU model compute (D24), and no neural teacher, comparator or grader — deterministic exact/membership checks only (D28).
- **run authority:** this sheet is currently **`TEST FITNESS: NOT_FIT`** and authorizes no compute. Future dispatch requires a valid FIT plus the lab's own D67 pre-run review inside the caps (CHARTER §4.5, §5.1). The registry's `training_hold: true` and `rental_hold: true` flags are stale after D67 retired the Training/M2/Stage B holds; its statement that this sheet is FIT is also stale and is not an authorization. This repair does not change the registry or rely on it for the classification.

## Bar (historical registration; NOT_FIT)

**Registration caveat from #3303:** Bar 1 below is a composite of different M1 instruments, not a faithfully preserved Amendment 5/6 or M2 bar. Its `admitted_strict_win_rows / 14,287` guard differs from the registered eligible-row metric; its Stage T DEV admission gate and #2844 `fresh_v2` panel are separate antecedents. Existing artifacts do not meet its top-8 threshold (`4,674` best Stage T table and `3,247` trunk Level 3, both below `4,775`). Do not use this composite as evidence or as an M2 acceptance bar. A future student sheet must re-register each exact bar, arm, panel and denominator. Bar 2 remains the registered M2 target text, but its candidate-specific GPU evaluator/artifact bridge is missing.

1. **Rung-0 discrete trunk rank-margin bars on `ft/dev` (`14,287` response-masked targets) and `fresh_v2` (`14,287` sealed held-out targets), across both paired seeds (`Seed 1`, `Seed 2`) — historical composite, not validated as unchanged from Amendment 5/6:**
   - **(i) Raw-score `top-1` vs the response-masked TRAIN count predictor:** `top-1` accuracy over the `14,287` scored positions beats the response-masked TRAIN count predictor (`1,226 / 14,287` = `8.58%` on `ft/dev`) by **`>= 5` percentage points (`>= 1,941 / 14,287` = `13.59%`) on both seeds** (`Seed 1` and `Seed 2`), with deterministic smallest-token-ID tie breaking (`token_id asc`).
   - **(ii) `recall@8` (`top-8`) over the declared 8-token kept list vs the response-masked TRAIN count predictor:** `top-8` accuracy over the writer's 8-token shortlist beats the response-masked TRAIN count predictor (`4,060 / 14,287` = `28.42%` on `ft/dev`) by **`>= 5` percentage points (`>= 4,775 / 14,287` = `33.42%`) on both seeds**.
   - **Reference (Amendment 5/6 masked TRAIN count predictor):** token-frequency count table computed exclusively over the **response-masked target positions of the TRAIN split** (`mw-balp/train/tokens.u16`: `7,512,797` tokens; `mw-balp/train/response_mask.u8`: `7,512,797` bytes, **`2,425,925` eligible response-masked targets**; ties by smallest token ID; top-8 `[1, 16, 14, 435, 340, 351, 3, 302]`), never reading DEV or fresh targets; the DEV-response oracle is diagnostic only.
   - **Anti-vacuity & admission guards (discrete integer form):** strict distance-win rate `admitted_strict_win_rows / 14,287 >= max(1%, min(5u, u + 5pp))` on both seeds (where `u` is the same-seed checkpoint-0 rate before any update), target cell admission `G-Adm >= 4,220 / 14,287` (`P_succ >= 676 / 3,847`), and exact `0` float/int export gap (`D11` `uor-r4.lut-discrete/1` round-trip verified).
2. **M2 grounded-reply milestone acceptance bars (`#2030`, [`m2-reply512.json`](../panels/m2-reply512.json), [`m2-fresh256.json`](../panels/m2-fresh256.json)), graded by the existing deterministic complete-reply evaluator unchanged (D28: no neural grader):**
   - **(a) Frozen development panel (`m2-reply512`, `b9661606…`, `512` rows / `256` histories):** **`complete >= 256 / 512` (`50.0%`)** complete correct replies through `uor-r4-core::native_geometric` (compiler $\to$ exact `StackStore` $\to$ emitter).
   - **(b) Sealed 256-row fresh held-out panel (`m2-fresh256`, `#2849`, frozen at HF commit `fb8c5e4fcd3568cc59be8be3073151295b98069c`):** **`pairs_both_complete_distinct >= 52 / 128` (`40.625%` of the `128` opposite-role history pairs, with single-row `complete >= 104 / 256`)** under `crates/uor-r4-core/examples/native-bank-generalization.rs` (`uor-r4.literal-role-dialogue/2`, `FrozenAnswers` + `ByteBpeTokenizer` + `pair_id` grader), where all four question-blind and role-swap controls (`copy_first_stated`, `copy_last_stated`, `role_swap_distractor`, `best_constant_reply`) score `0 / 128` pairs (`0 / 256` pair-verified rows).

## Base artifact

- **Discrete trainer & Stage 3 trunk (`crates/uor-r4-discrete` on `main`):**
  - **Stage 2 `BVG-K` (`K = 16`) coordinate-block trainer (`#2836`, PR `#3239` merge `c695ad21a72c5b91def22458eebbe796e07d5392`):** `crates/uor-r4-discrete/src/stage_t.rs` and binary `crates/uor-r4-discrete/src/bin/stage-t-run.rs` (`T16_fz` `FrozenZeroV1` and `T16_rel` `LearnV1`, `Relation { kept: 8, sweeps: 1 }`, `CodebookInit::PinnedTableV1` `address-structured-v1.json` sha256 `6600bdb1306dbaea4d77f5bf021e717a6bef791e44c1ec7d7c9bb9f0e8cd3cd1`, active D40 64-batch rollback on the `3,847`-row TRAIN copy-successor anchor `anchor-manifest.json` sha256 `885789a71420751660c69c82ec01df7c6bfc285dc4a87e28648e1ead050e909b`).
  - **Stage 3 single-vector geometric trunk & `D11` exporter (`#2844`, PR `#3258` merge `6f38dd393865a0b4320a2e28bee3be675130dc16`):** `TrunkAssemblyV1` in `crates/uor-r4-discrete/src/trunk.rs` and binary `crates/uor-r4-discrete/src/bin/trunk-run.rs`, composing GPU causal read + `PositionHolonomyV1` (`#2843`) + `OrbitRootSnapNormV1` (`#2842`) + `DiscreteQuaternionRecurrenceV1` + `DiscreteHamiltonGateV1` + unmodified `OuterL1V1` writer + `D11` (`uor-r4.lut-discrete/1`) integer artifact export (`trunk-model.d11`, `542,260` bytes; `Level 3` `T16_fz` already reaches `2,988 / 2,989` `top-1` on `ft/dev` and `3,163 / 14,287` on `fresh_v2` at `34,325–37,853 context tok/s`).
  - **Deterministic compiled digests pinned by the discrete driver:** `driver_source_sha256 = 04c5165464a18eea6d4edadc6970007dbdc7908ead420f4a2f575e22ea43061d` (`crates/uor-r4-discrete/src/training.rs`), `geometry_sha256 = 46ddbb76b7fbe6ac6222ea24779fc10055b1d6cea6a8da786895676fd29fe327` (`uor-r4-core` geometry tables), and `sm_120` CUDA `kernel_sha256 = 0f9ccfa7b4748292f83a40227e30ea7b69c893a1649fa8afa3f7d547802429d6` (`discrete.ptx`).
  - **Literal frozen count-reference source SHAs (carried over from Amendment 6):** frozen qualification source `2fd6358bbc52d2a202a872ad0954ebb12894606f` (`11` finite controls PASS: `9` metadata, `2` CUDA; [`archive/labs/count-reference-2026-10-13/README.md`](../../../archive/labs/count-reference-2026-10-13/README.md)) and protected PR [`#2328`](https://github.com/UOR-Foundation/uor-r4/pull/2328) merge commit `9e904a83448c451f738d5c1d45224dfdf73eb84e` (verified ancestor of `main`).

## Data split and pins

1. **TRAIN split (`data/ft-balp.tar`, `326,860,800` bytes, md5 `45fbd266ecab1e5c113e10ec7ae1e918`, sha256 `b1d2211377a2e99d5977cc95d46875027023b08867488b1372185b3f62a1473f`):**
   - `mw-balp/train/tokens.u16`: `7,512,797` `u16` payload tokens (`expected_payload_tokens = 7512797`), sha256 `3dee858f84b750f507ceb3dc3f393711741950954735efaaf61bf704c017a4c8`.
   - `mw-balp/train/response_mask.u8`: **`7,512,797` bytes, sha256 `b5e7352d67d6883a833155a5fd397a03f6d4f1b2649e3d4ec3e2ea7d95b949e3`**, **`expected_train_targets=2425925`** (`expected_eligible_targets = 2425925`), enforced fail-closed by `crates/uor-r4-discrete/src/controller.rs:184-191` and `docs/labs/m1-bvg-k-stage-t-training/configs/t16-{frozen-zero,relation}-seed{1,2}.json`.
   - Visited discrete training prefix (`Q1` = `4,096` batches / `262,144` rows; `Q4` = `16,404` batches $\times$ `64` rows = `1,049,856` eligible targets at `context = 192`).
2. **DEV split (`data/ft-dev.tar`, `143,360` bytes, md5 `0a1d0a05020a5d80b5d008fd5b731134`, sha256 `45e93e32918f8ad5c977ad3a98926b561173049aacd8bfce2b6e2e49a0390b5c`):**
   - `ft/dev/tokens.u16`: `42,856` `u16` payload tokens (`expected_payload_tokens = 42856`), sha256 `0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b`.
   - `ft/dev/response_mask.u8`: `42,856` bytes, sha256 `a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078`, **`expected_dev_targets=14287`** (`expected_eligible_targets = 14287` across `400` documents, including `3,847` `P_succ` copy-successor rows).
3. **Sealed rung-0 fresh transfer slice (`fresh_v2`, `#2844`):**
   - Deterministic cut `cut_fresh_slice(train, 192, 1_049_856, 14_287)` strictly past the `1,049,856`-target visited TRAIN prefix (`0` duplicate `(window, target)` rows): `14,287` eligible targets (`3,932` `P_succ` rows), `tokens` sha256 `1f8bf88d8deee51574c68ffd8976ce4415fc8aed5602213d6905a37a138517ea`, `mask` sha256 `fe87363920e19bf072e9fe3a7289e6cd2f1abde6c2431caae8f5c8494d3829d5`.
4. **Tokenizer (`ByteBpeTokenizer`):**
   - `tokenizer.json` (`vocab = 4096`, `bos = 0` `<|bos|>`, `eos = 1`, `unk = 2`), sha256 `d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89`.
5. **M2 grounded-reply panels (`#2030`, `#2849`, never included in training data per D22 §4):**
   - **Frozen development panel (`m2-reply512`):** `512` rows / `256` histories (`b9661606…`; [`docs/harness/panels/m2-reply512.json`](../panels/m2-reply512.json)).
   - **Sealed 256-row fresh held-out panel (`m2-fresh256`):** [`docs/harness/panels/m2-fresh256.json`](../panels/m2-fresh256.json), stored at `hf://datasets/caseyallard/uor-r4-store/data/m2-fresh256/` at commit `fb8c5e4fcd3568cc59be8be3073151295b98069c` (`plan.json` `122,268` B sha256 `0ed1ab98ab50bbad6c484c4e7e4ffeeb8bb7ad17d162251826302efe86d75715`; unscored `inputs.json` `524,850` B sha256 `1de2fc28607eff6bc412eee41a259d806ee1479b7c925059f00e9d3b54afa99a`; sealed `labels.json` `68,000` B sha256 `3b5c54771b8dc62e3c30aef2b30de28c09dfe79625f333854af046e6e354e2be`; `manifest.json` `1,384` B sha256 `7c33f3f7e19edcb4b1a7aa5e5eab3bb0056db9351067d809f62f97b4a8880964`; `0` exact/word/2-gram overlaps against all `74` historical curriculum and transfer literals).

## Steps

**NOT_FIT: no dispatch command is registered and no compute is authorized.** The superseded recipe and FIT block are preserved byte-identical in [`docs/labs/m2-pin-train-mask-sheet/superseded-recipe-2026-10-26.md`](../../labs/m2-pin-train-mask-sheet/superseded-recipe-2026-10-26.md). Post-merge review #3303 found these instrument defects at the bound source:

- `stage-t-run step0` requires `--t16-fz`, `--t16-rel`, `--t1-fz` and `--out`; `train` additionally requires `--anchor-manifest`, `--sheet` and `--out-dir`. The rejected recipe omitted them, as well as the anchor-manifest build and Stage T grade.
- `trunk-run step0` requires `--fz-config`, `--fz-checkpoint` and `--out-dir`; each arm requires `--name`, `--config`, `--checkpoint` and `--out-dir`; grade requires all four report paths and `--out`. The rejected recipe omitted these and ran trunk Step 0 before Stage T had written its checkpoint.
- The invoked `/workspace/bin` paths had no binary staging step from the build output. The four-arm loop repeated the already sealed M1 Stage T/trunk recipe rather than training an M2 student, and Step 4 named no command or registered GPU scorer for complete replies and opposite-role pairs.
- Bar 1 is the earlier M1/trunk rung-0 bar, not an M2 result or the M1-close bar. Passing it would not resolve the trunk's `STAGE3_DIAGNOSTIC_HOLD`. Bar 2 names the native M2 grader, but no discrete-D11/native-bank bridge or registered GPU complete-reply/pair evaluator exists for this recipe.

Before a future FIT, `m2-train-grounded-replies` (#2853) must provide under its own ORDER and scope: (a) the discrete M2 student trainer and its saved D11 artifact; (b) a deterministic GPU evaluation command for `m2-reply512` and sealed `m2-fresh256`, including the 256/512 and 52/128 metrics plus the four 0/128 controls; and (c) the D11/native-bank artifact bridge, or a separately reviewed artifact-family decision. Its scope and dependencies must name the evaluator/bridge work. Do not add CLI aliases, rerun the M1 recipe, or restore FIT by editing flags alone.

- Pinned driver config digests (`docs/labs/m1-bvg-k-stage-t-training/configs/`), retained as identities only:
  - `t16-frozen-zero-seed1.json`: sha256 `77d61451125b04827821216e221d548885af188bad6d9d301234d126fefa3b7d`
  - `t16-frozen-zero-seed2.json`: sha256 `d169d4ef347f28f4a289d131c3664e56a40a62a29774bf87cf0313ab358592cf`
  - `t16-relation-seed1.json`: sha256 `ab41788a5e72a1f11370f875b090653e0d1b36be091393861b0e147688e9d746`
  - `t16-relation-seed2.json`: sha256 `a0bfa6c5fe7497d85d2a3d8a68d86157a8dcaa72a5d5b7eca193d904c480f263`

## Stop rule

- **Enforced preflight & Step 0 aborts (`controller.rs:174-204`, `stage_t.rs`, `trunk.rs`):** the discrete driver refuses before model allocation if `mw-balp/train/response_mask.u8` is missing or its sha256 differs from `b5e7352d67d6883a833155a5fd397a03f6d4f1b2649e3d4ec3e2ea7d95b949e3`, if `expected_eligible_targets != 2425925` or `expected_payload_tokens != 7512797`, if `ft/dev` tokens/mask digests or `expected_eligible_targets != 14287` mismatch, if `tokenizer.json` sha256 differs from `d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89`, or if any Step 0 control (`AKd`, `V1d`, `KId`, `V8d`, `V9d` in `stage-t-run step0`; `G-T1`..`G-T5` in `trunk-run step0`) fails.
- **In-run diagnostic stops (D40):** 64-batch anchor audit rollback on the `3,847`-row TRAIN copy-successor anchor (`consecutive_full_rejections = 3` triggers `DIAG_STOP_CONSEC_REJ`, classified as a configuration diagnostic stop, never a mechanism negative under CHARTER §4.1 / D22).
- **Fresh-panel seal protection:** `m2-fresh256` (`1de2fc28…` / `3b5c5477…`) is never opened or scored unless the candidate first clears its development bars across both seeds.
- **Self-set budgets:** wall-time and dollar figures below are estimates, not stop rules; a healthy run inside the global owner caps (`<= 4` pods, `<= $8/h`) is extended and its lease renewed (CHARTER §5.4).

## Cost

- **M2 student cost:** **UNDEFINED** until #2853 defines the actual student, seed/arm design and evaluator. The figures below are M1 antecedent records, not an M2 estimate or authorization.
- **preparation:** `$0` host-only verification and this #3303 instrument repair; no pod was leased.
- **M1 antecedent only:** `~5.5h` (`$11.94`) for four-arm Stage T (#2836) and `~4.4h` (`$2.48`) for four-arm Stage 3 trunk assembly (#2844). Repeating them would not produce an M2 result.
- **M1 antecedent controls:** Step 0 `~33 s`; `ft/dev` and `fresh_v2` evaluation `32–158 s` per arm at `8,326–37,853 context tok/s`. No M2 complete-reply/pair evaluator throughput is registered.
- **storage:** the M1 `542,260`-byte `trunk-model.d11` is an antecedent artifact, not the missing M2 student artifact.

## Minimum test power

- **M2 student power:** **UNDEFINED** until #2853 registers its actual seed/arm design and candidate-specific evaluator. The paired Stage T seeds are trajectory-identical antecedents, not independent evidence for the M2 student.
- **M1 antecedent only:** the two `T16_fz`/`T16_rel` seed bindings and `1,049,856`-row `BVG-K`/`TrunkAssemblyV1` fitting describe #2836/#2844, not the missing M2 student.
- **Panel noise references:** on `ft/dev` and `fresh_v2` (`n = 14,287`), worst-case Wilson 95% half-width is `< 0.83` percentage points (`119` rows); the historical `+5 pp` (`715`-row) threshold is `> 6x` that interval. On `m2-fresh256` (`256` rows / `128` pairs), measured 16-block cluster-sampling SE is `6.24` rows (`244 bps`) at `p = 85.35%` and `8.64` rows (`338 bps`, `4.32` pairs) at the 40% bar. These references do not define the missing student's arms or prove its power.

## TEST FITNESS line

```text
TEST FITNESS: NOT_FIT - M2 #2030 discrete-geometric-learning
sheet: docs/harness/prep/m2-pin-train-mask-sheet-discrete-2026-10-26.md
reviewer: fresh non-author agent-48 (DeepSeek/deepseek-flash), TEST FITNESS: NOT_FIT confirmed after corrections at 3db39b50405ba1135f5eace9d32ca12011fe2f5c; this line records that review identity only.
reason: instrument - the registered Stage T/trunk recipe is not executable at the bound head (required CLI flags omitted; trunk Step 0 precedes the Stage T checkpoint; no anchor-manifest build; no Stage T grade; no binary staging; no M2 student training; no candidate-specific GPU M2 reply evaluation or D11/native-bank bridge for this recipe). The lawful-pointer forced-copy oracle is a different mechanism and is not a substitute.
scope: closes no mechanism; the discrete learner stays ACTIVE; pins are retained and historical bar text is qualified by the composite-bar caveat, not treated as an authorization; no compute is authorized by this sheet.
prior: the 2026-10-26 FIT (antigravity-104, reviewer d929c424-0c35-4319-9260-6c9e41913c06) was a pins/shape review, not an executability review; its recipe and verdict are preserved byte-identical in docs/labs/m2-pin-train-mask-sheet/superseded-recipe-2026-10-26.md.
decision: D68 fresh review agent-47 at 29d3b99651057c44c35d3a21cfa17f378af42ad2; #3303 comment 6426348540.
date: 2026-10-26
```
