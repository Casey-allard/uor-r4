# Pre-Registration: Ranking With Admission Intact (`r-ranking-with-admission-intact`)

- **Issue / Roadmap Item:** [`#2884`](https://github.com/UOR-Foundation/uor-r4/issues/2884) (`docs/roadmap/items/r-ranking-with-admission-intact.md`)
- **Milestone / Area:** `research` / `training` (`docs/research/notebook/m1-discrete-attention.md`)
- **Author Lab:** `antigravity`
- **Date:** 2026-10-25
- **Upstream Prerequisites:**
  - `m1-a8-full-dose-rerun` (`#2831`, PR `#3192`, merged `cb2b5748905dc77b122325101b62dd7c29fc1413` — `status: done`)
  - `m1-w2-capacity-capture` (`#2845`, PR `#3187`, merged `9dd74d80f077e1d1df523c97dc4a5a0bf5fa1a7b` — `status: done`)
  - `m1-bvg-k-stage-t-training` (`#2836`, Step 4 of `PLAN.md` — `BVG-K` `K = 16` batched-coordinate training)

---

## 1. Question and Base Artifact

**Question:** When cell admission is held intact across a full `16,404`-batch dose on two independent seeds (`seed in {1, 2}`) using the `PrimeZeta` injective codebook (`4,096 / 4,096`), `AddressCellsA8` (`256 / 256` occupied cells at `16` tokens/cell at init), and the active `D40` admission-anchor controller (`floor = 1,187 / 3,847`), what is the trained model's **conditional ranking accuracy among admitted rows** ($\text{top-1}_{\text{cond}} = \text{top-1} / N_{\text{adm}}$ and $\text{top-8}_{\text{cond}} = \text{top-8} / N_{\text{adm}}$) and full-panel `DEV top-1` compared against the full-panel and admission-matched count-table baselines (`Unigram` and `T2` query-token bigram), and does within-cell ranking or cell admission remain the dominant M1 bottleneck?

- **What `#2831` (`m1-a8-full-dose-rerun`) and `#2845` (`m1-w2-capacity-capture`) already proved at `Q1` (`4,096` batches):**
  1. In `#2831` (`docs/labs/m1-a8-full-dose-rerun/README.md`, sealed RTX 5090 archive `a8-full-dose-rerun-20261025.tar.gz`, SHA-256 `4a744188f14c934f89e57a1be83ecc8f2cb211620290a6539217837f809bae55`), the `FrozenZero` arm reached `Q1` (`batch = 4,096`, `epoch = 258`, `q1-dev.json` SHA-256 `6b924afa04103467814b6c71773f7df2c7e6df342f045d83434d3dafe8372943`) with **admission demonstrably intact**:
     - `occupied_cells = 256 / 256` (`mean_admitted_token_count = 1,807,135 / 14,287 = 126.49` tokens/row vs `1,828,736 / 14,287 = 128.00` at `batch = 0`).
     - `TRAIN anchor b_3(P_succ) = 1,258 / 3,847` (`+47` over initial `1,211`, `+71` above the `1,187` floor).
     - `DEV b_3(P_succ) = 1,106 / 3,847` (`28.75%`, vs `857` ungated).
     - `DEV b_3(Full) = 4,292 / 14,287` (`30.04%`, `+48` rows above initial `4,244 / 14,287`).
  2. With admission intact at `Q1` (`4,096` batches, `1/4` of a full dose), **conditional within-cell ranking improved `6.48x`**:
     - `DEV top-1` rose from `133 / 14,287` (`133 / 4,244 = 3.13%` of admitted rows at `batch = 0`) to **`871 / 14,287` (`871 / 4,292 = 20.29%` of admitted rows**, `679` strict wins + `192` tie-resolved wins; `1.88x` the best `16,404`-batch `K16-A` score of `463 / 14,287` and `71.04%` of the full-panel `Unigram` baseline `1,226 / 14,287`).
     - `DEV top-8` rose from `513 / 14,287` (`513 / 4,244 = 12.09%` of admitted rows at `batch = 0`) to **`1,455 / 14,287` (`1,455 / 4,292 = 33.90%` of admitted rows**, `2.84x`).
     - Even with admission held intact at `4,292 / 14,287`, `3,421 / 4,292` admitted targets (`79.71%`) were not ranked first inside the `126.49`-token admitted candidate pool (and `9,995 / 14,287` targets were outside the $K = 8$ admitted cells).
  3. Why a full `16,404`-batch 2-seed run is still required: in `#2831`, `Relation` stopped at batch `1,600` (`epoch = 132`) and `FrozenZero` stopped at batch `5,248` (`epoch = 274`) on `STOP_FOR_DIAGNOSIS: consecutive_rejections` (`mechanism_negative: false`) because `K = 1` single-coordinate greedy updates coupled with `64`-batch whole-interval rollback rejected whole intervals whenever one local 64-row move dipped below the global anchor floor.
- **Base Artifact & Pinned Inputs:**
  - **Codebook & Cell Layout:** `PrimeZeta` injective codebook (`4,096 / 4,096` unique vectors, $B = 4$, $d = 32$) + `AddressCellsA8` (`256 / 256` cells at `16` tokens/cell at init, `b_3(P_succ) = 1,351 / 3,847`, `b_3(Full) = 4,244 / 14,287` at $K = 8$; `b_3(P_succ) = 2,178 / 3,847` at $K = 16$).
  - **Active Admission Controller (`D40`):** Fixed 3,847-row TRAIN anchor manifest `f85466ff339d8fb848fb1151a7d672fba5687296b81b1ad6fb39c36ddaca27de` (`margin = 24`, `minimum_baseline = 500`, `floor = 1,187 / 3,847` at $K = 8$).
  - **Pinned Corpus Shards (BPE `d36d3e87`, $V = 4,096$, $L = 128$):**
    - `train.bin`: SHA-256 `49263fe24ecc3ad2d8bfabbc6dc1fbd95fb5fbecbefdfbe0fa0ee6bfefcf0d6f` (`19,165` sequences, `2,425,925` targets)
    - `val.bin` (`DEV` panel): SHA-256 `53dfca085fa4be04875e424eefeb43952d0b44ab6d5d6ec2d590ae7cf5ff7e71` (`14,287` evaluation positions; `P_succ = 3,847` rows)
    - `fresh_v2` transfer slice (`crates/uor-r4-discrete/fixtures/fresh_panel_v2.bin`): SHA-256 `c1f40ab0578d20ca337282b9efbe12b29f21f99a11cb17689708ee46f2181167` (`384` rows; sealed until a DEV gate passes)

---

## 2. Arms, Data Split, and Steps

All arms run on a single NVIDIA GeForce RTX 5090 (`sm_120`) under the Stage T `BVG-K` (`K = 16` coordinates/step) + `AddressCellsA8` + `D40` admission-anchor recipe across two deterministic seeds (`seed in {1, 2}`) for `16,404` batches (`64` rows/batch):

| Arm ID | Read Policy | Kept Cells ($K = s[3]$) | Admission Anchor Floor | Evaluators Scored at `Q1` (`4,096`) and `Q4` (`16,404`) |
| :--- | :--- | ---: | :--- | :--- |
| **`Arm-FZ-K8`** (`FrozenZeroV1`, $K = 8$) | `FrozenZeroV1` (trit-locked control) | `8` (`~128` tokens/row) | `1,187 / 3,847` (`1,211 - 24`) | Full-panel + conditional admitted `top-1`/`top-8` vs full & admission-matched `Unigram`, `T2 Bigram`, `Code Table` |
| **`Arm-Rel-K8`** (`Relation` `LearnV1`, $K = 8$) | `Relation` (`LearnV1`) | `8` (`~128` tokens/row) | `1,187 / 3,847` (`1,211 - 24`) | Full-panel + conditional admitted `top-1`/`top-8` vs full & admission-matched `Unigram`, `T2 Bigram`, `Code Table` |
| **`Arm-Rel-K16`** (`Relation` `LearnV1`, $K = 16$) | `Relation` (`LearnV1`) | `16` (`~256` tokens/row) | $B_0(K=16) - 24$ | Full-panel + conditional admitted `top-1`/`top-8` vs full & admission-matched `Unigram`, `T2 Bigram`, `Code Table` |

- **Data Split:**
  - **Training:** `train.bin` (`19,165` sequences, `2,425,925` targets) + 3,847-row `D40` TRAIN anchor manifest `f85466ff...`.
  - **Development (`DEV`):** `val.bin` (`14,287` positions; `P_succ = 3,847` rows), scored at `batch = 0`, `Q1 = 4,096`, `Q2 = 8,192`, and `Q4 = 16,404`, plus admission-matched count-table scoring (`w2-assemble` over the $N_{\text{adm}}$ admitted rows of each checkpoint).
  - **Fresh Transfer (`fresh_v2`):** `384`-row sealed slice, evaluated once at `Q4` if and only if an arm passes its DEV admission and ranking bars.
- **Execution Steps (Probe Ladder):**
  1. **Step 1 — Finite Control & Admission-Matched Scorer Check (~3 min):** Verify on RTX 5090 that `w2-assemble` / `discrete-stack` reproduces the exact `#2831` `Q1` `FrozenZero` numbers (`b_3(Full) = 4,292`, `b_3(P_succ) = 1,106`, `top1 = 871`, `top8 = 1,455`) and computes the exact admission-matched `Unigram` and `T2 Bigram` scores on those `4,292` admitted rows.
  2. **Step 2 — 256-Batch `BVG-K` + `D40` Throughput Probe (~5 min):** Run `256` batches of `Arm-Rel-K8` (seed 1) on RTX 5090 under Stage T `BVG-K`, confirming zero `consecutive_rejections` stops and projected wall time $\le 35$ min/run.
  3. **Step 3 — Paired 3-Arm $\times$ 2-Seed Full-Dose GPU Run (`6` runs):** Execute `Arm-FZ-K8`, `Arm-Rel-K8`, and `Arm-Rel-K16` on seeds `1` and `2` through `16,404` batches, recording full-panel and conditional admitted `top-1` / `top-8` against both full-panel and admission-matched count tables, plus decode `tokens/s`.

---

## 3. Bar, Minimum Test Power, and Decision Table

- **Minimum Test Power (CHARTER §4.3):**
  - `2` independent seeds (`seed in {1, 2}`) per arm across all `14,287` DEV positions (`3,847` `P_succ` rows, $\ge 4,244$ admitted rows at $K = 8$) and `384` `fresh_v2` transfer rows.
- **Registered Bars:**
  1. **Bar `G-Intact` (Admission Intact Bar):** On **both** seeds (`1` and `2`) at `Q1` and `Q4`, `occupied_cells >= 192 / 256`, $\bar{n} \le 256$ ($K = 8$) or $\le 512$ ($K = 16$), DEV $b_3(P_{\text{succ}}) \ge 1,106 / 3,847$ ($K = 8$) or $\ge 1,750 / 3,847$ ($K = 16$), and DEV $b_3(\text{Full}) \ge 4,244 / 14,287$ ($K = 8$) or $\ge 6,500 / 14,287$ ($K = 16$).
  2. **Bar `G-Cond-Rank` (Conditional Admitted Ranking Bar vs Count Tables):** On **both** seeds at `Q4`, conditional top-1 among admitted rows ($\text{top-1} / N_{\text{adm}}$) exceeds the `#2831` `Q1` mark (`20.29% = 871 / 4,292`) and beats the **admission-matched `Unigram` baseline** on the same admitted row set, while full-panel `DEV top-1` beats full-panel `Unigram` (`1,226 / 14,287`, CHARTER §4.8) with decode `tokens/s` reported.
  3. **Bar `G-Rel-Lift` (Relation Read Attribution Bar):** `Arm-Rel-K8` (or `Arm-Rel-K16`) beats `Arm-FZ-K8` on mean `DEV top-1` by $\ge +192$ rows across seeds `1` and `2`, with positive transfer on `fresh_v2`.
- **Decision Table:**

| Outcome | Condition | Decision & What It Unlocks |
| :--- | :--- | :--- |
| **`PASS_RANKING_BEATS_UNIGRAM`** | `G-Intact` holds and full-panel `DEV top-1 > 1,226 / 14,287` (plus conditional `top-1` beats admission-matched `Unigram`) on both seeds | Close `#2884` (`status: done`); confirms admission protection + `BVG-K` unlocks above-unigram ranking and feeds the winning checkpoint into `m1-export-gap-and-toks` (`#2839`). |
| **`RANK_BOTTLENECK_ISOLATED`** | `G-Intact` holds ($N_{\text{adm}} \ge 4,244$ at $K=8$ or $\ge 6,500$ at $K=16$), conditional `top-1` improves over `20.29%`, but full-panel `DEV top-1 <= 1,226` | Close `#2884` (`status: done` on its diagnostic contract: admission-intact ranking reported against count tables) and prioritize within-cell ranking capacity (`r-finite-relational-head` `#2877`, `r-full-epoch-revisit-schedules` `#2878`). |
| **`ADMISSION_CEILING_DOMINATES`** | Conditional `top-1` among admitted rows exceeds `50%` (approaching the W2 code table's `59.27%`), while $N_{\text{adm}}$ caps full-panel `top-1` | Close `#2884` (`status: done`) and prioritize `m1-address-key-full-vector` (`#2834`) and `r-rank-objective-keeps-cells` (`#2883`) to raise $N_{\text{adm}}$. |
| **`INSTRUMENT_STOP`** | Step 1 or Step 2 fails or `consecutive_rejections` fires under `BVG-K` | Classify as `instrument` (CHARTER §4.2) and repair the anchor cadence/granularity before Step 3. |

---

## 4. Cost and Stop Conditions

- **Estimated Compute & Cost:**
  - `1 x NVIDIA GeForce RTX 5090` (`~$0.65/h`–`$1.05/h` on Vast.ai, inside the 4-pod / `$8/h` fleet cap).
  - Step 1 (Q1 admission-matched count-table check) + Step 2 (`256`-batch probe): `~8 min`.
  - Step 3 (`6` runs $\times$ `~22 min/run` under Stage T `BVG-K` `K = 16`): `~2.2 hours` (`~$1.50`–`$2.40` total).
- **Stop Conditions:**
  - **Immediate Instrument Stop:** Abort before Step 3 if Step 1 fails to reproduce `#2831` `Q1` (`871 / 4,292`) or if Step 2 triggers `consecutive_rejections`.
  - **Dependency Gate:** Execution of Steps 1–3 is held (`status: held`) until `m1-bvg-k-stage-t-training` (`#2836`, Step 4 of `PLAN.md`) completes.

---

## 5. Review Verdict

- **Reviewer:** Fresh-context non-author `research` subagent `0d7e3ab3-18b0-4f86-9020-0fbbd27df5f4` (`antigravity`, CHARTER §4.5, §5.1, D68)
- **Posted Record:** Issue [`#2884` comment `6396898060`](https://github.com/UOR-Foundation/uor-r4/issues/2884#issuecomment-6396898060)

TEST FITNESS: FIT
