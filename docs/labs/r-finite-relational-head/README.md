# Finite Relational Head Two-Seed Stage T & Stage 3 Trunk Evaluation (`r-finite-relational-head`, `#2877`)

- **Lab:** `antigravity` ([`#2877`](https://github.com/UOR-Foundation/uor-r4/issues/2877), PR [`#3210`](https://github.com/UOR-Foundation/uor-r4/pull/3210) `antigravity-65` -> PR [`#3288`](https://github.com/UOR-Foundation/uor-r4/pull/3288) [`antigravity-96`](../../loop/tasks/r-finite-relational-head/antigravity-96/ORDER.md))
- **Milestone / Line:** M2 (`#2030`) / `m1-discrete-attention` — mechanism [`relation-read`](../../mechanisms/relation-read.md) (`ACTIVE`)
- **Pre-Registration Sheet:** [`../../harness/prep/r-finite-relational-head-2026-10-25.md`](../../harness/prep/r-finite-relational-head-2026-10-25.md) (**TEST FITNESS: FIT** at [`#2877` comment `6396409286`](https://github.com/UOR-Foundation/uor-r4/issues/2877#issuecomment-6396409286); **D68 Reconciliation: APPROVE** at [`#2877` comment `6421371461`](https://github.com/UOR-Foundation/uor-r4/issues/2877#issuecomment-6421371461))
- **Sealed GPU Archives (`caseyallard/uor-r4-store`):**
  - Stage T (`#2836`, `vast-57710022`, `2x RTX 5090`, `$11.94`): `results/m1-bvg-k-stage-t-training/stage-t-20261025.tar.gz` (`c004db506bf64903231c5c225a07e9969d5e3344c51a67eaec9429265e5f44b6`, [`grade.json`](../m1-bvg-k-stage-t-training/grade.json) `7917ec85941d919db14ab67567fe824c87d6d2edf18bbd54138bf320b6ea4c81`, [`step0-receipt.json`](../m1-bvg-k-stage-t-training/step0-receipt.json) `0ca7d95568adf216ba3f2abda16fbcbe3fc73bd72b96c7d0665c694000d5be50`)
  - Stage 3 Trunk (`#2844`, `vast-57764453`, `1x RTX 5090`, `$2.48`): `results/m1-trunk-assembly-and-training/trunk-20261025.tar.gz` (`4f4d4817e2750565f65bdb8a6c6f8ed07b44389f0757702cf6eba11b64e23d35`, [`grade.json`](../m1-trunk-assembly-and-training/grade.json) `8413be77ae4253318536a5df26e054032b768086544f2b7ed5f91dfb92f1cce9`)
  - Within-Cell Reranker Decomposition (`#3267`, `vast-57824058`, `1x RTX 5090`): [`stage-t-decomposition.json`](../m1-within-cell-writer/stage-t-decomposition.json) (`e9dd940c0f58a1c494bb5d8f38a485528b6693bcaa67b23df3f1d867bec76cc6`)
- **Graded Receipt:** [`relation-head-grade.json`](./relation-head-grade.json) (`stage-t-run grade-relation-head --out docs/labs/r-finite-relational-head/relation-head-grade.json`)
- **Verdict:** **`RELATIONAL_WITHIN_ARM_GAIN_TRAILS_FROZEN_ZERO_ADMISSION`** (`mechanism_negative: false`, `status: done`)

---

## 1. Executive Summary & Outcome-Invariance Decision (`CHARTER.md` §3.5, §4.1, §5.10 D67/D68)

1. **Instrument Gates (`Gate C1–C5` / `Step 0`, `Gate D40`, `Gate S`) — ALL PASS (`2 / 2` seeds):**
   - **Step 0 GPU controls (`#2830` & `#2836`):** `c1_flock_collapse`..`c5_determinism` (`5/5 PASS`, `a9c1228b...`) and Stage T `Step 0` (`AKd` `32/32`, `V1d` `24/24`, `KId` `128/128`, `V8d` `128/128`, `V9d` bitwise resume `3545fe65...`, `57,156 context tok/s`) all pass with `0` mismatches.
   - **Gate D40 (`2 / 2` seeds):** On both `H_relation` (`T16_rel` Seeds 1 & 2) and `C_frozen_zero` (`T16_fz` Seeds 1 & 2), the 3,847-row `TRAIN` copy-successor anchor stayed strictly above `floor = 1,187 / 3,847` (`1,211 -> peak 1,849 -> final 1,286` on `T16_rel`; `1,211 -> peak 1,823 -> final 1,342` on `T16_fz`), and `DEV b_3(P_succ)` stayed well above `676 / 3,847` (`1,144` on `T16_rel`, `1,216` on `T16_fz`).
   - **Gate S (`2 / 2` seeds):** `H_relation` (`T16_rel`) committed **`186 >= 16`** relation-gate block updates and **`425 >= 16`** relation coordinate moves per seed (`1,505` total moves = `545` vector + `535` combine + `425` relation across `678` committed updates and `199` joint commits at `11,562` / `11,201 context tok/s`), while `C_frozen_zero` committed `0` relation moves (`V8d PASS`).
2. **Positive Within-Arm Relational Learning (`H_relation` = `T16_rel`, Seeds 1 & 2):**
   - Training the 25-trit relational head under `BVG-K` (`K = 16`) + D40 reaches `Q1` (`4,096` batches, `+2,496` batches past `K = 1` `T1_rel`'s batch-`1,600` stop), lifting `DEV top-1` **`2.79x`** over `T1_rel` (`133 -> 371`, `+238` rows) and reaching a higher `TRAIN` copy-successor peak (`1,849 / 3,847` vs `1,823 / 3,847` on `T16_fz`) and higher `admitted_strict_win_rows` (`172` vs `93`).
   - Zeroing the learned 25 relation trits on the trained `T16_rel` checkpoint (`relation_off` ablation) degrades every metric on both seeds: `DEV b_3(Full)` drops `3,827 -> 3,753` (`-74` rows), `DEV b_3(P_succ)` drops `1,144 -> 1,129` (`-15` rows), `DEV top-1` drops `371 -> 327` (`-44` rows), `DEV top-8` drops `1,124 -> 1,063` (`-61` rows), and `admitted_strict_win_rows` drops `172 -> 123` (`-49` rows).
   - Learned relation gates expand distinct read codes to **`27,132` TRAIN / `4,429` DEV** (vs `19,385` / `4,067` on `T16_fz`, `+39.96%` TRAIN code diversity) and produce a larger Stage 3 `Level 2` causal quaternion recurrence lift over `C_rec_null` (`+67` / `+53` rows on `ft/dev` vs `+33` on `T16_fz`, and `+35` / `+43` on `fresh_v2` vs `+20` / `+38`, beating `T_restr(A_W) = 1,602` on `ft/dev`!).
3. **Between-Arm Comparison Against `C_frozen_zero` (`T16_fz`) and Count Tables (`CHARTER.md` §4.1, D22):**
   - Because modifying the 25 relation trits during shared outer-L1 cell shortlist selection perturbs the predecessor-aligned `A8-Outer` query anchor $x_{L-1}$ (consistent with `#2885` `r-read-off-vs-read-on`), `H_relation` (`T16_rel`) admits **`3,827 / 14,287`** `DEV` rows (`1,144 / 3,847` `P_succ`) vs **`5,009 / 14,287`** (`1,216 / 3,847` `P_succ`) on `C_frozen_zero` (`T16_fz`, $\Delta = -1,182$ `Full`, $-72$ `P_succ`).
   - Across all downstream ranking and count-table metrics on both seeds, this `-1,182`-row outer-L1 admission gap causes `H_relation` to trail `C_frozen_zero`: raw `OuterL1V1` `DEV top-1` is `371` vs `455` ($\Delta = -84$, `top-8` `1,124` vs `1,484`), full-panel read-code table $T(c(x))$ is `3,680` vs `4,085`, admission-matched `T_restr(A_W)` is `1,602` (`top-8 = 3,007`) vs `2,140` (`top-8 = 4,006`), `G_restr(A_W)` is `2,364` (`top-8 = 3,529`) vs `3,084` (`top-8 = 4,674`), and Stage 3 Trunk `Level 3` `top-1` is `2,227 / 2,226` vs `2,988 / 2,989` on `ft/dev` and `2,427` vs `3,163` on `fresh_v2` (`14,287` rows).
   - Under `CHARTER.md` §4.1 (D22), this closes only the **uncoupled single-pass `Relation + A8-Outer` shortlist configuration** where relation-perturbed $q^{\text{read}}(x)$ drives outer-L1 cell shortlist admission, while keeping `relation-read` `ACTIVE` for decoupled admission (anchoring cell shortlist admission on $x_{L-1}$ / `FrozenZero` while using the richer `27,132`-code `Relation` representation for within-cell / trunk selection).
4. **Why Re-Running `C_route` or `C_successor_only` on a Pod Is Refused (`CHARTER.md` §3.5, §5.10 D67/D68):**
   - Both positive rows of [`r-finite-relational-head-2026-10-25.md`](../../harness/prep/r-finite-relational-head-2026-10-25.md) §3 require $\text{H\_relation} \ge \max(\text{C\_frozen\_zero}, \text{C\_route}) + F_{\text{full}}$ on both seeds. Because $\max(\text{C\_frozen\_zero}, \text{C\_route}) \ge \text{C\_frozen\_zero} > \text{H\_relation}$ already holds strictly across both seeds on every metric, no outcome of a `$5.05` GPU training run on `C_route` or `C_successor_only` can change the decision table verdict (`outcome_invariance_verified: true`).

---

## 2. Two-Seed Admission, Ablation & Count-Table Comparison (`n = 14,287` DEV, `n = 3,847` `P_succ`)

| Arm / Comparator | Seed | Batches / Updates / Moves | `TRAIN` Anchor (`init -> peak -> final`, floor `1,187`) | Distinct Read Codes (`TRAIN` / `DEV`) | `DEV b_3(P_succ)` (`n = 3,847`) | `DEV b_3(Full)` (`n = 14,287`) | `DEV top-1` (`P_succ`) | `DEV top-8` (`P_succ`) | Strict Wins |
| :--- | :---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| **`H_relation` (`T16_rel`, `LearnV1`)** | `1` | `4,096` / `678` / `1,505` (`425` rel in `186` b) | `1,211 -> 1,849 -> 1,286` | **`27,132` / `4,429`** | `1,144` (`29.74%`) | `3,827` (`26.79%`) | **`371`** (`78`) | **`1,124`** (`243`) | **`172`** |
| **`H_relation` (`T16_rel`, `LearnV1`)** | `2` | `4,096` / `678` / `1,505` (`425` rel in `186` b) | `1,211 -> 1,849 -> 1,286` | **`27,132` / `4,429`** | `1,144` (`29.74%`) | `3,827` (`26.79%`) | **`371`** (`78`) | **`1,124`** (`243`) | **`172`** |
| **`H_relation` `relation_off` Ablation ($g \gets 0$)** | `1, 2` | `4,096` / `678` / `1,505` (eval $g \equiv 0$) | — | — | `1,129` (`-15`) | `3,753` (`-74`) | `327` (`-44`) | `1,063` (`-61`) | `123` (`-49`) |
| **`C_frozen_zero` (`T16_fz`, `FrozenZeroV1`)** | `1` | `3,008` / `420` / `854` (`0` rel) | `1,211 -> 1,823 -> 1,342` | `19,385` / `4,067` | **`1,216`** (`31.61%`) | **`5,009`** (`35.06%`) | **`455`** (`107`) | **`1,484`** (`318`) | `93` |
| **`C_frozen_zero` (`T16_fz`, `FrozenZeroV1`)** | `2` | `3,008` / `420` / `854` (`0` rel) | `1,211 -> 1,823 -> 1,342` | `19,385` / `4,067` | **`1,216`** (`31.61%`) | **`5,009`** (`35.06%`) | **`455`** (`107`) | **`1,484`** (`318`) | `93` |
| **`C_route` (`ReadSpec::Route`, cp0 `#2885`)** | `1` | `0` (zero-update GPU control) | — | — | `408` (`10.61%`) | `748` (`5.24%`) | `0` | `74` | `0` |
| **`C_successor_only` ($g[\text{succ},0]=+1$, cp0 `#2830`)** | `1` | `0` (zero-update GPU control) | `1,211` | — | **`2,034`** (`52.87%`) | — | — | — | — |

---

## 3. Unconditional & Admission-Matched Count Tables + Within-Cell Rerankers (`#2836`, `#3267`)

| Metric / Predictor (`DEV` `n = 14,287`, `P_succ` `n = 3,847`) | `H_relation` (`T16_rel` Seeds 1 & 2) | `C_frozen_zero` (`T16_fz` Seeds 1 & 2) | Delta (`H_relation - C_frozen_zero`) | Reference / W2 Q4 Baseline (`n = 1,920` admitted) |
| :--- | ---: | ---: | ---: | ---: |
| **Unconditional Unigram `U` (`top-1` / `top-8`)** | `1,226` / `4,060` | `1,226` / `4,060` | `0` | `1,226` (`8.58%`, `CHARTER.md` §4.8) |
| **Unconditional Query-Bigram `T2` = `G` (`top-1` / `top-8`)** | `6,300` / `11,263` | `6,300` / `11,263` | `0` | `6,300` (`44.10%`, `CHARTER.md` §4.8) |
| **Unconditional Exact Read-Code Table $T(c(x))$ (`top-1`)** | `3,680` (`25.76%`) | **`4,085`** (`28.59%`) | `-405` | `3,414` (`23.90%` on Q4; both Stage T arms beat Q4) |
| **Admission-Matched Unigram `U_restr(A_W)` (`top-1`)** | `942` | **`1,127`** | `-185` | `754` (on Q4 admitted) |
| **Admission-Matched Symmetry Table `T_sym_restr(A_W)` (`top-1`)** | `1,377` | **`1,909`** | `-532` | `1,004` (on Q4 admitted) |
| **Admission-Matched Read-Code Table `T_restr(A_W)` (`top-1` / `top-8` / `P_succ`)** | **`1,602`** / `3,007` / `327` | **`2,140`** / `4,006` / `376` | `-538` / `-999` / `-49` | `1,138` (`top-8 = 1,806` on Q4; both beat Q4 & `U = 1,226`) |
| **Admission-Matched Bigram Table `G_restr(A_W)` (`top-1` / `top-8` / `P_succ`)** | **`2,364`** / `3,529` / `496` | **`3,084`** / `4,674` / `517` | `-720` / `-1,145` / `-21` | `1,482` (`top-8 = 1,879` on Q4; both beat Q4 & `U = 1,226`) |
| **Within-Cell Missed `T_restr(A_W)` Hits Lost Inside Target's Own Cell (`#3267`)** | `1,190 / 1,291` (**`92.18%`**) | `1,597 / 1,765` (**`90.48%`**) | `+1.70 pp` | Isolates `OuterL1V1` loss to within-cell tie-breaking |
| **Within-Cell Read-Code Reranker (`top1_cell_only` / `cell_prio` / `pooled`, `#3267`)** | `1,095` / `1,597` / **`1,602`** ($d = +1,231 > 74$) | `1,706` / `2,142` / **`2,140`** ($d = +1,685 > 86$) | `-611` / `-545` / `-538` | Permuted-code null: `554` (`T16_rel`) / `753` (`T16_fz`) |

---

## 4. Stage 3 Trunk Ladder (`Level 0..3`) on `ft/dev` (`14,287`) and `fresh_v2` (`14,287`, `#2844`)

| Trunk Arm (`TrunkAssemblyV1` + `D11` Export) | Throughput (`TRAIN` / `DEV` / `fresh_v2` `context tok/s`) | `ft/dev` `L0 -> L1 -> L2 -> L3` `top-1` (`L3 top-8`) | `ft/dev` Recurrence Lift (`L2 - L1`) | `fresh_v2` `L0 -> L1 -> L2 -> L3` `top-1` (`L3 top-8`) | `fresh_v2` Recurrence Lift (`L2 - L1`) |
| :--- | ---: | ---: | ---: | ---: | ---: |
| **`Trunk-Rel-S1` (`H_relation` Seed 1)** | `16,609` / `8,530` / `8,326` | `371 -> 1,557 -> 1,624 ->` **`2,227`** (`2,497`) | **`+67`** (`> T_restr = 1,602`) | `439 -> 1,559 -> 1,594 ->` **`2,427`** (`2,693`) | **`+35`** (`L3 > G_restr = 2,364`) |
| **`Trunk-Rel-S2` (`H_relation` Seed 2)** | `16,608` / `8,542` / `8,336` | `371 -> 1,557 -> 1,610 ->` **`2,226`** (`2,496`) | **`+53`** (`> T_restr = 1,602`) | `439 -> 1,559 -> 1,602 ->` **`2,427`** (`2,693`) | **`+43`** (`L3 > G_restr = 2,364`) |
| **`Trunk-FZ-S1` (`C_frozen_zero` Seed 1)** | `76,316` / `34,325` / `37,838` | `455 -> 2,054 -> 2,087 ->` **`2,988`** (`3,247`) | `+33` | `457 -> 2,187 -> 2,207 ->` **`3,163`** (`3,415`) | `+20` (`L3 > G_restr = 3,084`) |
| **`Trunk-FZ-S2` (`C_frozen_zero` Seed 2)** | `76,343` / `34,337` / `37,853` | `455 -> 2,054 -> 2,087 ->` **`2,989`** (`3,248`) | `+33` | `457 -> 2,187 -> 2,225 ->` **`3,163`** (`3,416`) | `+38` (`L3 > G_restr = 3,084`) |
