# Pre-registration: M1 Fresh Panel Transfer (`m1-fresh-panel-transfer`, `#2841`)

- Date: 2026-10-26
- Author: `antigravity`
- Issue: `#2841` (`m1-fresh-panel-transfer`, Step 7 of `PLAN.md`)
- Milestone: M1 (`#2029`), closing bar (e) read with bar (d) (`CHARTER.md` §4.3, §4.5, §4.8, D39)
- Mechanism: [`discrete-attention`](../../mechanisms/discrete-attention.md) (`READY`) and [`bvg-k`](../../mechanisms/bvg-k.md) (`READY`)
- Parent sheets: [`m1-attention-a8-address-cells-learning-2026-10-19.md`](m1-attention-a8-address-cells-learning-2026-10-19.md), [`m1-export-gap-and-toks-DRAFT-2026-10-23.md`](m1-export-gap-and-toks-DRAFT-2026-10-23.md)
- Status: **EXECUTED — `PASS_BAR_E_FRESH_TRANSFER` (`antigravity-110`, `#3312`; `antigravity-111`, `#3318`)**

## 1. Question

When a discrete geometric language model (`crates/uor-r4-discrete`) has passed M1 bars (a)–(d) on the registered `14,287`-row `ft/dev` panel across both training seeds (`seed = 1` and `seed = 2`) through its exported D11 integer container (`UORLUT01`, verified at exact zero export gap `Δtop1 = 0`, `Δtop8 = 0`, `Δadm = 0`, `Δloss = 0` with `tokens/s` reported under `#2839`), does its top-1 gain over the frozen TRAIN Unigram (`U_train`) and TRAIN context-free next-word (`T2` bigram, `G_train`) count tables transfer to the sealed, pre-registered Fresh V2 panel (`0b7e754c…` / `a1cda529…`, `14,479` eligible targets, `3,594` `P_succ` copy-successor rows) on both seeds on the first and only opening of the fresh slice?

## 2. Bar and outcome decision table

Read with `CHARTER.md` §4.8 bar (d) (`D39`) and §4.3 (`>= 2` seeds, `>= 200` fresh rows, bars no narrower than measured noise), the fresh-panel transfer evaluation scores the two DEV-passing `UORLUT01` artifacts (`seed = 1` and `seed = 2`) once on the GPU via `d11-scorer fresh-score` (`docs/labs/m1-d11-artifact-scorer/instrument/`) against the frozen TRAIN count tables (`U_train` and `G_train` built strictly from the `1,049,856` matched TRAIN rows of `mw-balp/train`):

1. **G-Fresh-Gate (prospective admission gate before any fresh file is opened):** `d11-scorer fresh-score` (`inst::fresh_transfer::validate_fresh_transfer_gate`) verifies a sealed two-seed DEV decision receipt (`schema == "uor-r4.m1-fresh-transfer-gate/1"`, `dev_decision == "PASS_BARS_A_TO_D_TWO_SEEDS"`) showing that on `ft/dev` (`14,287` rows) both `seed = 1` and `seed = 2` achieved:
   - `d11_top1 > 1,226` (DEV unigram) and `d11_top1 >= 6,300 + max(24, |d11_top1(1) - d11_top1(2)|)` (DEV T2 successor table plus the two-seed DEV noise band);
   - `export_gap_b2_pass == true` (`14,287 / 14,287` exact row agreement with training-side `eval-corpus`), `planted_fault_b4_pass == true`, and `tokens_per_s_reported == true`;
   - distinct 64-hex `artifact_sha256` pins for `seed = 1` and `seed = 2`, plus posted `#2841` `fit_review_url` and `prerun_review_url`.
   Any call to `dev_rows`, `rows_from_corpus`, or `fresh-score` without a valid receipt is refused with `SEALED_FRESH_PANEL_REFUSED` before touching the fresh corpus bytes on disk.
2. **G-Fresh-Full (primary full-panel fresh transfer bar, `14,479` eligible targets of `fresh_v2`):** For each seed `s in {1, 2}`, let `M_fresh(s)` be the `UORLUT01` integer top-1 hit count on `fresh_v2`, `U_fresh` be the frozen TRAIN unigram top-1 hit count on `fresh_v2`, `T2_fresh` be the frozen TRAIN `T2` bigram top-1 hit count on `fresh_v2`, and `(b_U(s), c_U(s))` and `(b_T2(s), c_T2(s))` be the paired discordant row counts on `fresh_v2`. Both seeds must satisfy:
   - `M_fresh(s) - U_fresh >= max(24, ceil(2 * sqrt(b_U(s) + c_U(s))), |M_fresh(1) - M_fresh(2)|)`
   - `M_fresh(s) - T2_fresh >= max(24, ceil(2 * sqrt(b_T2(s) + c_T2(s))), |M_fresh(1) - M_fresh(2)|)`
   where `ceil(2 * sqrt(d))` is evaluated in exact integer arithmetic as the smallest integer `k >= 0` such that `k * k >= 4 * d`.
3. **G-Fresh-Psucc (copy-successor fresh transfer bar, `3,594` `P_succ` rows of `fresh_v2`):** Carrying forward the A8 fresh transfer bar (`m1-attention-a8-address-cells-learning-2026-10-19.md:73,120`), on the `3,594` copy-successor rows (`inst::fresh_transfer::is_prefix_copy_successor`: `prefix[..prefix.len() - 1].iter().rposition(|&tok| tok == query).is_some_and(|index| prefix[index + 1] == target)`, with the `4,900`-row `windows(2).any(...)` variant also audited), both seeds must satisfy:
   - `M_psucc(s) - T2_psucc >= max(20, ceil(2 * sqrt(b_psucc(s) + c_psucc(s))), |M_psucc(1) - M_psucc(2)|)`
4. **G-Fresh-Integrity (clean-room integer parity and planted-fault control on `fresh_v2`):** On both seeds:
   - `refused_rows == 0` across all `14,479` rows;
   - `reference_mismatches == 0` (`14,479 / 14,479` exact row agreement between the GPU `d11_score_rows` kernel and clean-room `IntegerDiscreteModel::observe_target`);
   - `b4_pass == true` (`planted_fault_control` rejects tampered container headers/tables and produces `> 0` row diffs on in-memory `cells` and `routes`/`gates` bit flips).

| Outcome | Condition | What it unlocks / decides |
|---|---|---|
| `PASS_BAR_E_FRESH_TRANSFER` | `G-Fresh-Gate`, `G-Fresh-Full`, `G-Fresh-Psucc`, and `G-Fresh-Integrity` all pass on both `seed = 1` and `seed = 2` (`bar_e_pass == true`). | Satisfies M1 closing bar (e), closes `#2841` (`status: done`), and unblocks Step 8 (`m1-close-saved-model`, `#2837`) to close Milestone M1 (`#2029`). |
| `FAIL_BAR_E_CLOSE_CONFIGURATION` | `G-Fresh-Gate` and `G-Fresh-Integrity` pass, but either seed misses `G-Fresh-Full` or `G-Fresh-Psucc` (`bar_e_pass == false`). | Closes the evaluated model configuration (`CHARTER.md` §4.1; does not close the mechanism), records the fresh transfer negative in `docs/research/notebook/m1-discrete-attention.md`, and prohibits re-scoring or re-drawing `fresh_v2` for tuned variants of the same configuration. |
| `INSTRUMENT_FAULT_REPAIR` | `G-Fresh-Integrity` fails (`refused_rows > 0`, `reference_mismatches > 0`, or `b4_pass == false`). | Classed as an instrument fault (`CHARTER.md` §4.2, L-007); no model capability verdict is called until the scorer/export fault is repaired. |

## 3. Base artifact and frozen baselines

- **Scorer instrument (`docs/labs/m1-d11-artifact-scorer/instrument/`):**
  - Crate: `m1-d11-artifact-scorer-instrument`, binary `d11-scorer`, subcommand `fresh-score SEED1_LUT SEED2_LUT FRESH_CONFIG_JSON DECISION_RECEIPT_JSON REPORT_JSON`.
  - GPU kernel: `src/scorer.cu` (`d11_score_rows`, integer-only PTX admitted by `build.rs` with zero float/mma/transcendental instructions) and shared row body `src/kernel_core.h`.
  - Clean-room reference: `uor_r4_integer::discrete_d11::IntegerDiscreteModel::observe_target` (`crates/uor-r4-integer/src/discrete_d11.rs`, `#![forbid(unsafe_code)]`).
- **Frozen count-table baselines (`U_train` and `G_train`):**
  - Built by `inst::fresh_transfer::build_train_count_tables` strictly from the `1,049,856` matched TRAIN rows (`16,404` batches of `64` rows at `context = 192`, `vocabulary = 4,096`, `bos = ("<|bos|>", 0)`) of `mw-balp/train` using `uor_r4_discrete::w2_capacity::{build_tables, predict}`:
    - `U_train`: unconditional target-token frequency table on `mw-balp/train`, ranked by `(highest count, lowest token id)` (scores `1,226 / 14,287` on `ft/dev`).
    - `G_train` (`T2`): query-token `(q = prefix.last() -> target_token)` successor frequency table on `mw-balp/train`, ranked by `(highest count, lowest token id)` with fallback to `U_train` only when `q` is unseen in `mw-balp/train` (scores `6,300 / 14,287` on `ft/dev`).
  - Zero fresh or DEV tokens ever enter `U_train` or `G_train`.
- **Candidate `UORLUT01` model artifacts (`seed = 1` and `seed = 2`):**
  - Exported 19-table `UORLUT01` (`uor-r4.lut-discrete-trunk/1`) artifacts of `trunk-joint-pointer-v1-d11` (`T16_fz` `FrozenZeroV1` Seeds `1` and `2`, passing M1 Bars (a)–(d) on `ft/dev` with `d11_dev_top1 = 7,002` and `7,001 / 14,287 >= 6,324`, `psucc_d11_top1 = 1,779` and `1,770 / 3,847 >= 1,265`, `0` export gap, and `B4` pass):
    - Seed 1 (`t16-frozen-zero-seed1-trunk.d11`, `86,536,132` bytes): SHA-256 `04652a3de8e966a23539b04fbd77c7d9872d9f13a541fcb224d48e766e2c147d`.
    - Seed 2 (`t16-frozen-zero-seed2-trunk.d11`, `84,943,328` bytes): SHA-256 `1b49e8bd823a1021e31de55c767111f9cf43807d3e6d007644e49281935877de`.

## 4. Data split and seal verification without opening

All corpus splits are pinned by SHA-256 and verified against the stored preservation archives without extracting, reading, or scoring the fresh rows prior to `G-Fresh-Gate`:

- **TRAIN split (`mw-balp/train`, count-table source only):**
  - `tokens.u16`: SHA-256 `3dee858f84b750f507ceb3dc3f393711741950954735efaaf61bf704c017a4c8` (`26,214,400` payload tokens, `10,975,281` eligible targets; first `16,404` batches = `1,049,856` matched TRAIN rows).
  - `response_mask.u8`: SHA-256 `b5e7352d67d6883a833155a5fd397a03f6d4f1b2649e3d4ec3e2ea7d95b949e3`.
- **DEV split (`ft/dev`, gate prerequisite only, never reported as fresh):**
  - `tokens.u16`: SHA-256 `0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b` (`44,300` payload tokens, `14,287` eligible targets, `3,847` `P_succ` rows).
  - `response_mask.u8`: SHA-256 `a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078`.
- **Sealed Fresh V2 transfer panel (`fresh_v2`, opened once only after `G-Fresh-Gate` passes):**
  - Preservation archive: `icloud:UOR-R4/results/codex/codex-attention-dose1-fresh-v2-preservation.tar` (`5,614,080` bytes, MD5 `318fb430c9726c9600a897eb1e3d767a`, SHA-256 `2cd13ef672990a01f7a9e63023e03afacecb9ba8883cd9d7d844ca12cfb60143`), stream-verified on 2026-10-26 without extracting or opening row contents:
    - `tokens.u16` (`89,420` bytes = 64-byte `UORT` header + `44,678 * 2` payload bytes across `400` retained `milestone_world_v2` documents with `0` exact full TRAIN/DEV document overlap): SHA-256 `0b7e754cfd242bf53145fe9cbca563be8503e6cd75df5449abb325b5c51ee1b8`.
    - `response_mask.u8` (`44,678` bytes, selecting `14,479` eligible response targets and `3,594` `P_succ` copy-successor rows): SHA-256 `a1cda5295547bb641b1ea71ba4ebabb58d58c40e6c048cda91f35a878fecbf5e`.
    - Seal report `report.json` (`4,047` bytes): SHA-256 `df5d074ae09c743aa586e3463ffe87faca5703f44f7802bff8ac49ee2990395f`.
    - Ordered private row binding `ordered-row-binding.private.jsonl` (`325,399` bytes): SHA-256 `c2d4c7e512379650ae3d72703c1affc9a0e85d754da7b73703dc5aad2cee6c06`.
    - Manifest `manifest.json` (`4,098` bytes): SHA-256 `b41b8344cd4ca162ab2ea7d6749b826041d06b660546ed69c8205a5f64caca44`.
- **Secondary preserved K16 2,000-row `Split::Fresh` slice (stream-verified, remains unopened):**
  - Preservation archive: `icloud:UOR-R4/results/codex/admission-redirect-transfer-20261015-codex-v2.tar` (`3,276,288` bytes, MD5 `8fb6a9e0d9282ef9ac869a595ce8ff33`, SHA-256 `004523df17711fc5eed4afa4ebadd2ce27a3fa001d3c327873fdb6cbdb3212fb`): `binding.json` `4bd108ad4b1ee4673f87e3b87ef89a01a563a556330fa577d06e675f6be7f460` (`4,084` bytes), `fresh/tokens.u16` `82ead8db9584544a4cc461c394aaf9b00fa6c9fd8fdbf82e316392a71ecabd7b` (`1,010,234` bytes, `505,085` payload tokens across `200` documents), `fresh/response_mask.u8` `8287d20d2c76d388ab7ed9becda6f209d224574a933fa2d14c7bdda3663f5ce4` (`505,085` bytes, `2,000` eligible targets).

## 5. Steps

1. **Pre-run gate verification (before any pod allocation):** Confirm that a saved discrete geometric model has passed M1 bars (a)–(d) on `ft/dev` (`14,287` rows) across both `seed = 1` and `seed = 2` via `m1-export-gap-and-toks` (`#2839`), write the two-seed `DECISION_RECEIPT_JSON` (`schema: "uor-r4.m1-fresh-transfer-gate/1"`), and pass a fresh-context D67 pre-run review posted on `#2841`.
2. **Pod allocation and ahead-of-time CUDA build:** Lease 1x RTX 5090 (`compute_120`) via `scripts/pod/uor-pod up --count 1 --gpu 5090` inside the `$8/h` / 4-pod fleet caps. Build `d11-scorer` with ahead-of-time `nvcc`:
   `UOR_NVCC_ARCH=compute_120 CARGO_TARGET_DIR=/root/build/target-sm120 cargo build --release --locked --manifest-path docs/labs/m1-d11-artifact-scorer/instrument/Cargo.toml --features cuda`.
3. **Single-shot GPU fresh-panel scoring:** Stage `mw-balp/train` (`3dee858f…` / `b5e7352d…`), the sealed `fresh_v2` corpus (`0b7e754c…` / `a1cda529…`), the two exported `UORLUT01` artifacts (`seed1.uorlut` and `seed2.uorlut`), and `DECISION_RECEIPT_JSON`, then execute once:
   `d11-scorer fresh-score seed1.uorlut seed2.uorlut fresh-v2-config.json decision-receipt.json fresh-transfer-report.json`
   on two independent builds (`build-a` and `build-b`) to confirm byte-identical deterministic output.
4. **Custody, teardown, and closeout:** Archive the receipts to `results/antigravity/m1-fresh-panel-transfer-<date>/` in `caseyallard/uor-r4-store` with `SHA256SUMS`, release and tear down the pod (`uor-pod release` + `uor-pod down`), record the notebook row in `docs/research/notebook/m1-discrete-attention.md`, and close `#2841` if `PASS_BAR_E_FRESH_TRANSFER` is met.

## 6. Stop rule

- **Do not start before DEV bars (a)–(d) pass on two seeds:** While no saved model has beaten both `1,226 / 14,287` (unigram) and `6,300 / 14,287` (T2) on `ft/dev` across `seed = 1` and `seed = 2`, `fresh_v2` remains sealed and unopened (`UNAVAILABLE_NOT_RUN`).
- **Score once; never re-draw or re-open for tuning:** Once `d11-scorer fresh-score` opens `fresh_v2` for a DEV-passing model configuration, the outcome (`PASS_BAR_E_FRESH_TRANSFER` or `FAIL_BAR_E_CLOSE_CONFIGURATION`) is final for that configuration (`CHARTER.md` §4.1, §4.3).
- **Abort immediately on instrument or parity fault:** If `refused_rows > 0`, `reference_mismatches > 0`, or `b4_pass == false`, halt and classify as `INSTRUMENT_FAULT_REPAIR` (`CHARTER.md` §4.2).

## 7. Cost

- **Current preparation and seal-verification unit (`antigravity-99`):** `$0.00` (`0` GPU hours; host unit tests and stream SHA-256 verification of the sealed archives only).
- **Future single-shot GPU fresh-panel evaluation (once unlocked by a DEV bar-(a)–(d)-passing model):** 1x RTX 5090 pod for `<= 0.25` GPU-hours at `<= $1.00/h` (estimated `$0.15`–`$0.25` total, well within the 4-pod / `$8/h` fleet cap). Based on `#3287`'s measured `d11-scorer score` time of `62.2 ms` per `14,287`-row panel on RTX 5090 (`sm_120`), scoring both seeds plus planted-fault `B4` controls and building the `1,049,856`-row TRAIN count tables takes under `2` minutes of active execution plus `~2` minutes of ahead-of-time `nvcc` compilation.

## 8. Minimum test power

- **Sample size (`CHARTER.md` §4.3):** `2` independent training seeds (`seed = 1` and `seed = 2`) evaluated across `14,479` eligible fresh targets (`72.4x` the `CHARTER.md` §4.3 `200`-row minimum and `7.2x` the A8 sheet's `2,000`-row minimum) and `3,594` copy-successor (`P_succ`) fresh rows (`7.18x` the A8 sheet's `500`-row `P_succ` minimum).
- **Noise floor and minimum detectable effect:**
  - Full-panel threshold (`14,479` rows): `max(24, ceil(2 * sqrt(b + c)), |M_fresh(1) - M_fresh(2)|)` (`>= 2.0` paired standard errors and at least the `24`-row floor and empirical two-seed spread).
  - Copy-successor threshold (`3,594` `P_succ` rows): `max(20, ceil(2 * sqrt(b_psucc + c_psucc)), |M_psucc(1) - M_psucc(2)|)` (`>= 2.0` paired standard errors and at least the `20`-row `P_succ` floor and empirical two-seed spread).

## 9. Verdict

- **Initial Reviewer:** Fresh-context non-author subagent `27655669-ec58-40ba-bcd0-b66880e8f91e` (`gemini-4-argon-high`, 2026-10-26, `#issuecomment-6423732020`)
- **Pre-run & FIT Reviewer (`trunk-joint-pointer-v1-d11`):** Fresh-context non-author subagent `f3238439-9ed8-4ca8-bb97-348f599481d1` (`gemini-4-argon-high`, 2026-10-26, `#issuecomment-6432026082`)
- **Execution (`1x RTX 5090` `vast-57903382`, `Build A` `ce5843b00dee006cf89a1028f80b5125221393dbfb549b09c2273f3e6d3660f4` and `Build B` `0357abf8ca70cd463b7976eebef5ddea2dbfb7d9128ef1246246f665e25fed17`, `ptx_sha256` `5eca26218bde5a74aee90b30d5c5270bbe0146a3ad2a87cf22c8bf770b30dce7`, `cubin_sha256` `c513559a64fc7fcd83e6ee90e893bc91b2f6452d29cf4605cd8b7e3585235cd1`, archive `m1-close-saved-model-20261026.tar` SHA-256 `a758ca0962d68c43d2796fcf7e43e3af57dccf0d82849f3b11a9f4e2294d1506`):**
  - `G-Fresh-Gate`: **PASS** (`trunk-joint-pointer-v1-d11-fresh-gate-receipt.json`)
  - `G-Fresh-Full` (`14,479` rows, `U_fresh = 1,279`, `T2_fresh = 6,582`): **PASS** (`seed_1 = 7,215` [`+5,936` vs `U_fresh`, `+633` vs `T2_fresh`, floor `82`]; `seed_2 = 7,228` [`+5,949` vs `U_fresh`, `+646` vs `T2_fresh`, floor `82`]; `two_seed_spread_full = 13`)
  - `G-Fresh-Psucc` (`3,594` `rposition` `P_succ` rows, `U_psucc = 229`, `T2_psucc = 1,240`): **PASS** (`seed_1 = 1,713` [`+473` vs `T2_psucc`, floor `52`]; `seed_2 = 1,711` [`+471` vs `T2_psucc`, floor `52`]; `two_seed_spread_psucc = 2`; `windows(2)` `4,900`-row variant: `2,333` and `2,329` vs `1,863` [`+470` / `+466`, floor `53`])
  - `G-Fresh-Integrity`: **PASS** (`refused_rows = 0`, `reference_mismatches = 0`, `b4_pass = true`)
  - Outcome: **`PASS_BAR_E_FRESH_TRANSFER`** (`bar_e_pass = true`)

TEST FITNESS: FIT
