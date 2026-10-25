# Finite Relational Head Stage M2 Synthesis & Pre-Registration (`r-finite-relational-head`, `#2877`)

- **Lab:** `antigravity` ([`#2877`](https://github.com/UOR-Foundation/uor-r4/issues/2877), Draft PR [`#3210`](https://github.com/UOR-Foundation/uor-r4/pull/3210), ORDER [`antigravity-65`](../../loop/tasks/r-finite-relational-head/antigravity-65/ORDER.md))
- **Milestone / Line:** M2 (`#2030`) / `m1-discrete-attention` — mechanism [`relation-read`](../../mechanisms/relation-read.md) (`ACTIVE`)
- **Pre-Registration Sheet:** [`../../harness/prep/r-finite-relational-head-2026-10-25.md`](../../harness/prep/r-finite-relational-head-2026-10-25.md) (**TEST FITNESS: FIT**, **D68 DECISION REVIEW: PASS** at [`#2877` comment `6396409286`](https://github.com/UOR-Foundation/uor-r4/issues/2877#issuecomment-6396409286))
- **Current Item Status:** **`held`** (held on `#2836` `m1-bvg-k-stage-t-training`, Step 4 of [`PLAN.md`](../../../PLAN.md), and scope widening to include `crates/uor-r4-discrete/**` when `#2836` lands)

---

## 1. Executive Summary & Cancel Probe Classification

When [`m1-a8-full-dose-rerun`](../m1-a8-full-dose-rerun/README.md) (`#2831`, PR `#3192`) reached `status: done`, `uor-harness loop next` unlocked `r-finite-relational-head` (`#2877`, whose sole dependency in `docs/roadmap/items/r-finite-relational-head.md` was `depends_on: [m1-a8-full-dose-rerun]`). Running the cancel probe on `origin/main` at `8cc6d7964cc6350eecba47d25576138eadcdf902` established four facts:

1. **Missing Pre-Registration Sheet (`instrument`):** No `docs/harness/prep/r-finite-relational-head*` sheet or `docs/labs/r-finite-relational-head/` directory existed yet (`CHARTER.md` §4.5 and `docs/mechanisms/relation-read.md:81-87`).
2. **What `#2831` Proved and Why It Stopped Early (`instrument` cadence-granularity coupling):**
   - In `#2831` ([`docs/labs/m1-a8-full-dose-rerun/README.md`](../m1-a8-full-dose-rerun/README.md)), the paired `Relation` (`LearnV1`, seed 1) and `FrozenZero` (`FrozenZeroV1`, seed 1) arms ran on `2 x RTX 5090` (`vast-57627077`) under the active D40 whole-interval admission gate (`cadence = 64`, `margin = 24`, `floor = 1,187 / 3,847` on the 3,847-row `TRAIN` `P_succ` anchor) with the legacy `K = 1` single-coordinate greedy schedule (`rows_v1`).
   - Protecting admission worked: `FrozenZero` passed `Q1` (`4,096` batches, `epoch = 258`) with `DEV b_3(P_succ) = 1,106 / 3,847` (`+249` rows over the ungated `#2831` parent run's `857`), `DEV b_3(Full) = 4,292 / 14,287` (`+48` over initial `4,244`), and **`DEV top1_correct = 871 / 14,287`** (`+738` rows / `6.55x` over initial `133`, `+722` rows over the sealed Q4 writer's `149`, and `+408` rows over the best K16 run's `463`).
   - Meanwhile, `Relation` (`LearnV1`) committed `22` relation-gate updates (`42` sensitive relation proposals, `82` vector updates, `28` combine updates; `epoch = 132`) and reached `TRAIN anchor b_3(P_succ) = 1,600 / 3,847` at batch `256` before hitting `STOP_FOR_DIAGNOSIS: consecutive_rejections` at batch `1,600` (`FrozenZero` hit the same `consecutive_rejections` stop at batch `5,248`, `epoch = 274`).
   - As diagnosed in [`docs/labs/m1-a8-full-dose-rerun/README.md`](../m1-a8-full-dose-rerun/README.md) §3.1, under `K = 1` greedy single-batch updates, each 64-batch audit window bundles `4–11` local 64-row updates before a global 3,847-row `TRAIN` anchor check runs; once `TRAIN` anchor admission settles near `1,232–1,249` (`45–62` rows above the `1,187` floor), a single locally greedy coordinate update inside a 64-batch window that drops global admission by `> 45–62` rows forces the whole 64-batch window to roll back.
3. **Why `#2836` (`m1-bvg-k-stage-t-training`) Is the Immediate Prerequisite:**
   - Step 4 of [`PLAN.md`](../../../PLAN.md) (`m1-bvg-k-stage-t-training`, `#2836`) replaces the `K = 1` single-coordinate schedule with the `BVG-K` (`K = 16`) batched-coordinate Stage T training law qualified in C1 (`#2847`) and C2 (`#2835`). Running `r-finite-relational-head` before `#2836` lands would repeat the `K = 1` cadence-granularity stop rather than testing the finite relational head at full dose.
4. **Docs-Only Item Scope:**
   - `docs/roadmap/items/r-finite-relational-head.md` line 10 restricts `scope:` to `[docs/harness/prep/r-finite-relational-head*, docs/labs/r-finite-relational-head/**, docs/loop/tasks/r-finite-relational-head/**]`.

---

## 2. Cumulative Evidence Chain for the Finite Relational Head (`relation-read`)

| Stage / Run | Artifact / Source | Key Admission Measurements (`b_3(P_succ)` / `b_3(Full)`) | Key Ranking & Control Measurements (`DEV` `n = 14,287`) | Classification / Status |
| :--- | :--- | :--- | :--- | :--- |
| **Stage 1 Preflight (`2026-10-18`)** | [`attention-preflight-2026-10-18`](../attention-preflight-2026-10-18/README.md) (`669395d88`) | Untrained level-0 sequential-successor gate ($g[\text{succ}, 0] = +1$): **`2,034 / 3,847`** (`52.87%`) vs `Flock` ($g \equiv 0$): **`1,351 / 3,847`** (`35.12%`, **`+683` rows**) | `9/9` structural controls pass (`Flock` collapse, `Route` collapse, `25/25` gate sensitivity, determinism) | Finite host & structural qualification **PASS (`9/9`)** |
| **Ungated A8 Q1 Stop (`2026-10-19`)** | [`a8-dose-2026-10-19`](../a8-dose-2026-10-19/README.md) (`be71b24dc`) | Without D40 anchor gating, `DEV b_3(P_succ)` eroded at `Q1` (`4,096` batches) from `1,351` to `551` (`Relation`) and `857` (`FrozenZero`) | `Relation` `DEV top-1`: `133 -> 244`; `FrozenZero` `DEV top-1`: `133 -> 287` (`11` relation updates accepted) | **Instrument stop** (local 64-row loss eroded global `b_3(P_succ)` admission; motivated D40) |
| **W2 Read-Code Capacity (`#2845`, `2026-10-25`)** | [`m1-w2-capacity-capture`](../m1-w2-capacity-capture/README.md) (`a0cd1972a`) | Sealed Q4 checkpoint (`seed 1`, `epoch 407`): `DEV b_3(Full) = 1,920 / 14,287` (`13.44%`) | Unconditional: `U = 1,226`, `T_code = 3,414`, `T2 = 6,300`, `W = 149`. Admission-matched (`n = 1,920`): **`T_restr(code) = 1,138 / 1,920`** (`59.27%`), `T_restr(T2) = 1,482 / 1,920`, `T_restr(U) = 754 / 1,920`, `W = 149 / 1,920` | **`H_mixed`**: exact read code carries `3,414` hits (`1,138 / 1,920` on admitted rows), proving massive head/writer headroom once admission is preserved |
| **Stage 1 CUDA Qualification (`#2830`, `2026-10-25`)** | [`m1-a8-finite-cuda-qualification`](../m1-a8-finite-cuda-qualification/README.md) (`98424f353`) | CUDA `c4_copy_p_succ`: **`2,034 / 3,847`** vs `1,351 / 3,847` (`+683`); `TRAIN` anchor `1,211 -> 1,092 < 1,187` rejected & bit-identically restored to `1,211` | `5/5` CUDA controls (`c1_flock_collapse`..`c5_determinism`) + D40 rollback/restore proven on RTX 5090 (`3,205 ms`) | Finite CUDA qualification **PASS (`5/5`)** (`status: done`) |
| **Gated A8 Rerun (`#2831`, `2026-10-25`)** | [`m1-a8-full-dose-rerun`](../m1-a8-full-dose-rerun/README.md) (`4cd9cb1db`) | `Relation`: peaked at `TRAIN anchor = 1,607 / 3,847` (batch `64`) and `1,600 / 3,847` (batch `256`), restored to `1,232 / 3,847` at batch `1,600`. `FrozenZero`: passed `Q1` (`4,096` batches) with **`DEV b_3(P_succ) = 1,106 / 3,847`** and **`DEV b_3(Full) = 4,292 / 14,287`** (`+2,372` admitted rows over Q4's `1,920`!) | `FrozenZero` at `Q1` (`4,096` batches): **`DEV top1_correct = 871 / 14,287`** (`+738` over init `133`, `5.85x` Q4 writer's `149`, `1.88x` best K16's `463`), **`DEV top8_correct = 1,455 / 14,287`**, `admitted_strict_win_rows = 679` | **Instrument stop** at batch `1,600` (`Relation`) / `5,248` (`FrozenZero`) from `K = 1` 64-batch cadence coupling (`mechanism_negative: false`, `status: done`) |

---

## 3. Baseline Admission & Ranking Reference Table (Against Count Tables)

Per the item's `Done when` (*"A registered run of the finite relational head with admission and ranking reported against the count tables"*), the table below consolidates the exact measured admission and ranking numbers from the sealed A8 and W2 artifacts on the 14,287-row `DEV` panel (`dev_panel_v1.bin`, `29e536d2d8ff1371a5194b838dc0791dd47df7a42a165fa226a99d95620ec288`) and its 3,847-row `P_succ` sub-panel (`p_succ_dev_v1.txt`, `e165f039c30a0a6bf5e74f9730e1c9d03f827c9d450201f31b7d318d18ff5b07`), establishing the baseline reference that the Stage M2 `BVG-K` (`K = 16`) 4-arm run (`docs/harness/prep/r-finite-relational-head-2026-10-25.md`) will extend across two seeds:

| Predictor / Checkpoint | `DEV b_3(P_succ)` (`n = 3,847`) | `DEV b_3(Full)` (`n = 14,287`) | `DEV top-1` (`n = 14,287`) | `DEV top-8` (`n = 14,287`) | Admission-Matched `top-1` (`n = b_3(Full)`) | Source Artifact |
| :--- | ---: | ---: | ---: | ---: | ---: | :--- |
| **`U` (Unigram Count Table)** | — (unrestricted) | `14,287 / 14,287` | **`1,226 / 14,287`** (`8.58%`) | `4,060 / 14,287` (`28.42%`) | `754 / 1,920` (on Q4 admitted) | [`m1-w2-capacity-capture`](../m1-w2-capacity-capture/README.md) |
| **`T2` / `G` (Query-Token Bigram Table)** | — (unrestricted) | `14,287 / 14,287` | **`6,300 / 14,287`** (`44.10%`) | `11,263 / 14,287` (`78.83%`) | `1,482 / 1,920` (on Q4 admitted) | [`m1-w2-capacity-capture`](../m1-w2-capacity-capture/README.md) |
| **`T_code` (W2 Exact Read-Code Table)** | — (unrestricted) | `14,287 / 14,287` | **`3,414 / 14,287`** (`23.90%`) | `7,558 / 14,287` (`52.90%`) | **`1,138 / 1,920`** (`59.27%` on Q4 admitted) | [`m1-w2-capacity-capture`](../m1-w2-capacity-capture/README.md) |
| **Untrained Init (`epoch = 0`, `Flock` / `Relation` $g \equiv 0$)** | `1,351 / 3,847` (`35.12%`) | `4,244 / 14,287` (`29.71%`) | `133 / 14,287` (`0.93%`) | `714 / 14,287` (`5.00%`) | `133 / 4,244` (`3.13%`) | [`dose-summary.json`](../m1-a8-full-dose-rerun/dose-summary.json) |
| **Untrained Init (`SuccessorOnly`, $g[\text{succ}, 0] = +1$)** | **`2,034 / 3,847`** (`52.87%`) | — (`P_succ` probe) | — | — | — | [`qual-summary.json`](../m1-a8-finite-cuda-qualification/qual-summary.json) |
| **Sealed Q4 Writer (`seed 1`, `epoch 407`, pre-D40)** | — | `1,920 / 14,287` (`13.44%`) | `149 / 14,287` (`1.04%`) | `747 / 14,287` (`5.23%`) | `149 / 1,920` (`7.76%`) | [`m1-w2-capacity-capture`](../m1-w2-capacity-capture/README.md) |
| **D40 `FrozenZero` at `Q1` (`batch = 4,096`, `epoch = 258`, `#2831`)** | **`1,106 / 3,847`** (`28.75%`) | **`4,292 / 14,287`** (`30.04%`) | **`871 / 14,287`** (`6.10%`) | **`1,455 / 14,287`** (`10.18%`) | **`871 / 4,292`** (`20.29%`) | [`dose-summary.json`](../m1-a8-full-dose-rerun/dose-summary.json) |
| **D40 `Relation` (`LearnV1`) at Batch `1,600` Stop (`epoch = 132`, `#2831`)** | `TRAIN`: `1,232 / 3,847` (peak `1,607` at b64, `1,600` at b256) | — (`stopped before Q1`) | — (`stopped before Q1`) | — (`stopped before Q1`) | — (`22` relation + `82` vec + `28` comb updates committed) | [`dose-summary.json`](../m1-a8-full-dose-rerun/dose-summary.json) |

---

## 4. Re-Entry Protocol Once `#2836` (`m1-bvg-k-stage-t-training`) Completes

When `#2836` (`m1-bvg-k-stage-t-training`) lands on `origin/main`:
1. Widen `docs/roadmap/items/r-finite-relational-head.md` `scope:` via D68 decision review to include `crates/uor-r4-discrete/**` if a dedicated 4-arm runner subcommand is needed, or dispatch the existing `discrete-stack` binary under the Stage T `BVG-K` (`K = 16`) schedule.
2. Post a fresh-context D67 pre-run review on `#2877` citing [`../../harness/prep/r-finite-relational-head-2026-10-25.md`](../../harness/prep/r-finite-relational-head-2026-10-25.md) and launch the 4-arm $\times$ 2-seed GPU run (`H_relation`, `C_frozen_zero`, `C_route`, `C_successor_only`, `~$5.05` across `~2.5h` on `1 x 2-GPU RTX 5090` pod).
3. Record all 8 arm-seed rows (unconditional `DEV` admission/ranking + admission-matched `T_restr` count-table comparisons + `fresh_v2` transfer) in this directory and in [`docs/research/notebook/m1-discrete-attention.md`](../../research/notebook/m1-discrete-attention.md), and close `#2877`.
