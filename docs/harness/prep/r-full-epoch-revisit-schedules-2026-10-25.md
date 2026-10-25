# Pre-Registration: Full-Epoch and Frequent-Token Revisit Schedules (`r-full-epoch-revisit-schedules`, `#2878`)

- **Date:** 2026-10-25
- **Author:** `antigravity` (ORDER [`docs/loop/tasks/r-full-epoch-revisit-schedules/antigravity-66/ORDER.md`](../../loop/tasks/r-full-epoch-revisit-schedules/antigravity-66/ORDER.md))
- **Non-Author FIT & D68 Reviewer:** `antigravity` fresh-context reviewer `fe902a56-7dcd-4627-b147-ea82df35b5bc` ([`#2878` comment `6396559418`](https://github.com/UOR-Foundation/uor-r4/issues/2878#issuecomment-6396559418))
- **Verdict:** **TEST FITNESS: FIT** | **D68 DECISION REVIEW: PASS** (`status: held` until `#2836` `m1-bvg-k-stage-t-training` completes)
- **Milestone / Line:** `research` / `m1-discrete-attention` — item [`r-full-epoch-revisit-schedules`](../../roadmap/items/r-full-epoch-revisit-schedules.md) (`#2878`), mechanism [`discrete-learning`](../../mechanisms/discrete-learning.md) (`ACTIVE`)

## 1. Question and Base Artifact

In `crates/uor-r4-discrete/src/completion.rs` (`COMPLETION_BATCHES = 16_404`) and `crates/uor-r4-discrete/src/training.rs` (`SchedulePolicy::CanonicalVectorThenReadV1`, `SchedulePolicy::InterleavedReadEvery8V1`), one canonical dose runs `16,404` batches of `64` rows, consuming `16,404 * 64 = 1,049,856` of the `2,425,925` `TRAIN` response targets (**`43.28%`**, leaving `1,376,069` `TRAIN` targets / `56.72%` unseen; a full corpus epoch requires $\lceil 2,425,925 / 64 \rceil = 37,906$ batches, or $2.3108\times$ the `16,404`-batch dose). Simultaneously, under a flat round-robin coordinate cycle over the `16,384` vector blocks (`4,096 tokens x 4 blocks`) plus read coordinates (`20` or `45`), every token's vector block receives the exact same coordinate visit frequency (`1` visit per `16,404` batches under `K = 1`, or `16` visits per `16,404` batches under `BVG-K` `K = 16`), regardless of whether the token is a top-256 frequent piece (`> 75%` of `TRAIN` targets) or a tail token.
- **Base artifact & prior diagnostic evidence:**
  - **W2 Read-Code Capacity Capture (`#2845`, [`docs/labs/m1-w2-capacity-capture/README.md`](../m1-w2-capacity-capture/README.md)):** On the `1,049,856` `TRAIN` targets of the `16,404`-batch slice, the model-matched unigram table (`U = 1,226 / 14,287`) equals the full-`TRAIN` (`2,425,925`-target) unigram table (`1,226 / 14,287`), and the exact read-code count table built on those `1,049,856` targets already covers `14,175 / 14,287` (`99.22%`) of `DEV` rows (`41,806` codes with count $\ge 2$) and scores **`T_code = 3,414 / 14,287`** (`T_restr = 1,138 / 1,920 = 59.27%` on admitted rows) vs `149 / 14,287` for the Q4 writer. Thus the `1,049,856`-target first slice already contains `99.22%` read-code coverage on `DEV`; the primary coverage bottleneck is **coordinate revisit frequency on high-frequency target tokens** (`256` total rows seen per token across `4` block visits under `K = 1`), compounded by the `56.72%` unseen tail of `TRAIN`.
  - **A8 Gated Rerun (`#2831`, [`docs/labs/m1-a8-full-dose-rerun/README.md`](../m1-a8-full-dose-rerun/README.md)):** Under `K = 1` + D40 64-batch whole-interval rollback (`floor = 1,187 / 3,847`, `kernel_sha256 = d600957a1ebf5f981db2e07b1d8534c3c981f4a1ae0abf9837ce110d586c75a6`, `geometry_sha256 = 46ddbb76b7fbe6ac6222ea24779fc10055b1d6cea6a8da786895676fd29fe327`, `driver_source_sha256 = 31789abea8b955cb0e137257fc9e8b3e5eb42865fec7728396fa377c9d4228ea`), `FrozenZero` reached **`DEV top-1 = 871 / 14,287`** and `DEV b_3(Full) = 4,292 / 14,287` at `Q1` (`4,096` batches) before `K = 1` cadence-granularity coupling stopped `Relation` at batch `1,600` and `FrozenZero` at batch `5,248`.
- **Question:** Under the Stage T `BVG-K` (`K = 16`) batched-coordinate learning law (`#2836`) and D40 admission gating (`floor = 1,187 / 3,847`), does (a) extending corpus coverage from the 1-pass `16,404`-batch baseline (`1,049,856 / 2,425,925 = 43.28%` `TRAIN` targets) to a **full corpus epoch (`37,906` batches, `2,425,925 / 2,425,925 = 100%` `TRAIN` targets)** and/or (b) reallocating coordinate visits via a **Zipf-stratified frequent-token revisit schedule (`FrequentTokenRevisitV1`)** improve `DEV` `top-1`, `DEV` `top-8`, and `DEV` admission (`b_3(P_succ)`, `b_3(Full)`) over the flat 1-pass baseline across two seeds?

## 2. Arms, Data Split, and Steps

All four arms run on the GPU (`discrete-stack`, `crates/uor-r4-discrete`, `CHARTER.md` §3.5) with identical A8 initialization (`SeededDeviceV1` + A8 cell layout), identical Stage T `BVG-K` (`K = 16`) block-apply kernel, identical D40 admission anchor (`floor = 1,187 / 3,847` on `p_succ_train_v1.txt`), and identical two seeds (`seed = 1, 2`):

| Arm | Target Coverage (`TRAIN` Targets) | Coordinate Schedule (`BVG-K`, `K = 16`) | Scientific Role |
| :--- | :--- | :--- | :--- |
| **`C_one_pass` (Baseline Control)** | `16,404` batches (`1,049,856 / 2,425,925 = 43.28%` `TRAIN` targets) | Flat round-robin (`InterleavedReadEvery8V1`, `~16` visits per vector block) | Matched 1-pass Stage T baseline |
| **`H_freq_16k` (Matched-Budget Frequent-Token Revisit)** | `16,404` batches (`1,049,856 / 2,425,925 = 43.28%` `TRAIN` targets) | Zipf-stratified 3-tier revisit (`FrequentTokenRevisitV1`: `50%` of coordinate blocks allocated to top-`256` `TRAIN`-frequency tokens, `30%` to ranks `257..1,024`, `20%` to ranks `1,025..4,096`, read coordinates interleaved every `8`th batch) | Isolates **coordinate revisit allocation** at identical `16,404`-batch compute and identical `43.28%` corpus slice |
| **`H_full_epoch_flat` (100% Target Coverage, Flat Cycle)** | `37,906` batches (`2,425,925 / 2,425,925 = 100.0%` `TRAIN` targets; final partial batch padded deterministically to `64` rows) | Flat round-robin (`InterleavedReadEvery8V1`, `~37` visits per vector block across full `TRAIN`) | Isolates **100% corpus target coverage** (`2.31x` batches) with uniform coordinate revisits |
| **`H_full_epoch_freq` (100% Target Coverage + Frequent-Token Revisit)** | `37,906` batches (`2,425,925 / 2,425,925 = 100.0%` `TRAIN` targets) | Zipf-stratified 3-tier revisit (`FrequentTokenRevisitV1` across all `37,906` batches) | Tests the joint effect of **100% target coverage + Zipf-stratified coordinate revisits** |

- **Data split and SHA-256 pins (`caseyallard/uor-r4-store:inputs/m1-a8-2026-10-25/` and `artifacts/corpus-v1/`):**
  - Full `TRAIN` split (`2,425,925` eligible response targets across `37,906` 64-row batches; first `16,404` batches = `train_stream.bin` `7c74a4ef45f93b02963ff8ca1fa1d61d7bb1138eb79d292dc2b2ed43cf3376bb`).
  - `p_succ_train_v1.txt`: `f85466ff339d8fb848fb1151a7d672fba5687296b81b1ad6fb39c36ddaca27de` (`3,847` rows, baseline `1,211 / 3,847`, active floor `1,187 / 3,847`).
  - `dev_panel_v1.bin`: `29e536d2d8ff1371a5194b838dc0791dd47df7a42a165fa226a99d95620ec288` (`14,287` rows) + `p_succ_dev_v1.txt` `e165f039c30a0a6bf5e74f9730e1c9d03f827c9d450201f31b7d318d18ff5b07` (`3,847` rows).
  - `fresh_panel_v2.bin`: `a643be37f1f007891a59dc42705158c3db52c4cf2322585cb8e3336e82b4cf30` (`2,095` rows; evaluated once at terminal checkpoint).
  - **Count-table references (`CHARTER.md` §4.4, §4.8):** `U = 1,226 / 14,287` (unigram), `T2 = 6,300 / 14,287` (bigram), `T_code(16k) = 3,414 / 14,287`, `T_code(full_epoch)` (exact read-code table built on all `2,425,925` `TRAIN` targets), and admission-matched `T_restr`.
- **Execution steps:**
  1. **Step 0 (Host & GPU schedule invariants):** Verify exact target coverage (`1,049,856` at batch `16,404`; `2,425,925 / 2,425,925` unique `TRAIN` target positions at batch `37,906`) and Zipf tier frequencies (`50% / 30% / 20%`, deterministic from `TRAIN` unigram counts only, `0` `DEV` leakage).
  2. **Step 1 (`16,404`-batch matched comparison):** Run `C_one_pass` and `H_freq_16k` on `seed = 1, 2` and score `DEV` at `Q1` (`4,096`) and `16,404`.
  3. **Step 2 (`37,906`-batch full-epoch continuation):** Continue `C_one_pass -> H_full_epoch_flat` and `H_freq_16k -> H_full_epoch_freq` from batch `16,404` to batch `37,906` (`100%` `TRAIN` targets; checkpoints at `16,404`, `24,606`, `32,808`, `37,906`), so `C_one_pass` and `H_freq_16k` are exact prefixes of the full-epoch arms with zero redundant compute.

## 3. Bar, Minimum Test Power, and Decision Table

- **Minimum test power (`CHARTER.md` §4.3):** Two seeds (`seed = 1, 2`) on `14,287` `DEV` rows (`3,847` `P_succ` rows) and `2,095` fresh rows (`fresh_panel_v2.bin`). Paired McNemar noise floor on `DEV` `Full`: $F_{\text{full}} = \max(24, \lceil 2\sqrt{b + c} \rceil)$ rows.
- **Instrument gates:** `Gate Cov` (exact `2,425,925 / 2,425,925` `TRAIN` targets visited by batch `37,906`, `0` `DEV` tokens read when building the `FrequentTokenRevisitV1` rank table); `Gate D40` (`TRAIN` anchor admission $\ge 1,187 / 3,847$ at every accepted checkpoint, `DEV b_3(P_succ) >= 676 / 3,847`).

| Outcome | Condition (Must Hold on Each of `seed = 1` and `seed = 2`) | Meaning & What It Unlocks |
| :--- | :--- | :--- |
| **`PASS_REVISIT_AND_EPOCH`** | `H_freq_16k` beats `C_one_pass` by $\ge F_{\text{full}}$ on `DEV top1_correct` at `16,404` batches **and** `H_full_epoch_freq` beats `H_freq_16k` by $\ge F_{\text{full}}$ at `37,906` batches (with non-inferior `DEV b_3(P_succ)`) | Closes `#2878` as `done`; promotes `FrequentTokenRevisitV1` + `37,906`-batch full-epoch coverage as the default discrete training schedule |
| **`PASS_FREQUENT_REVISIT_ONLY`** | `H_freq_16k` beats `C_one_pass` by $\ge F_{\text{full}}$ at `16,404` batches, while `37,906` vs `16,404` stays within $\pm F_{\text{full}}$ | Closes `#2878` as `done`; confirms W2's finding that `1,049,856` targets already saturate corpus coverage (`99.22%`) and promotes `FrequentTokenRevisitV1` at the cheaper `16,404`-batch dose |
| **`PASS_FULL_EPOCH_ONLY`** | `H_full_epoch_flat` beats `C_one_pass` by $\ge F_{\text{full}}$ at `37,906` batches, while `H_freq_16k` stays within $\pm F_{\text{full}}$ of `C_one_pass` | Closes `#2878` as `done`; promotes `37,906`-batch full-epoch coverage with flat coordinate cycling |
| **`NULL_COVERAGE_SATURATED`** | Both `H_freq_16k` and `H_full_epoch_freq` stay within $\pm F_{\text{full}}$ of `C_one_pass` on both seeds (with `Gate Cov` and `Gate D40` passing) | Closes `#2878` as `done`; proves `16,404` batches under `BVG-K` (`K = 16`) already saturates discrete table updates and closes the revisit-schedule hypothesis (`CHARTER.md` §4.1) |

## 4. Cost and Stop Conditions

- **Cost:** Because `C_one_pass` (`16,404` batches) is the exact prefix of `H_full_epoch_flat` (`37,906` batches) and `H_freq_16k` (`16,404` batches) is the exact prefix of `H_full_epoch_freq` (`37,906` batches), all four arms run as **two `37,906`-batch trajectories per seed** (`4` trajectories total across `seed = 1, 2`). On `1` pod (`2 x RTX 5090`, `~$2.02/h` via `scripts/pod/uor-pod`), `2` parallel trajectories take `~70 min` per seed (`~2.3 hours` total), estimated spend **`~$4.65`** (inside `<= 4` pods and `<= $8.00/h` fleet caps).
- **Stop conditions:** (1) Immediate stop if `Gate Cov` or `Gate D40` fails (`0` mechanism claims). (2) Hold pod launch until `#2836` (`m1-bvg-k-stage-t-training`, Step 4 of `PLAN.md`) completes.

## 5. Review Verdict

TEST FITNESS: FIT - research #2878 r-full-epoch-revisit-schedules
sheet: docs/harness/prep/r-full-epoch-revisit-schedules-2026-10-25.md
reviewer: antigravity (fresh-context non-author reviewer fe902a56-7dcd-4627-b147-ea82df35b5bc; https://github.com/UOR-Foundation/uor-r4/issues/2878#issuecomment-6396559418)
date: 2026-10-25
A FIT verdict clears compute for the configuration in that sheet only (D22).
