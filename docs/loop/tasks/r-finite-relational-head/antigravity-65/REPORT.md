# REPORT: r-finite-relational-head, antigravity-65

Lab: antigravity. Item: [r-finite-relational-head](../../../../roadmap/items/r-finite-relational-head.md). Order: [ORDER.md](ORDER.md). Issue: References #2877.

## What did not work

- **Cancel probe finding (`instrument`):** When `m1-a8-full-dose-rerun` (`#2831`, PR `#3192`) reached `status: done`, `uor-harness loop next` unlocked `r-finite-relational-head` (`#2877`), whose sole listed dependency in `docs/roadmap/items/r-finite-relational-head.md` was `depends_on: [m1-a8-full-dose-rerun]`. Running the cancel probe on `origin/main` at `8cc6d7964cc6350eecba47d25576138eadcdf902` identified three `instrument` blockers to immediately launching the 4-arm $\times$ 2-seed GPU run:
  1. **Missing pre-registration sheet (`instrument`):** No `docs/harness/prep/r-finite-relational-head*` pre-registration sheet or `docs/labs/r-finite-relational-head/` lab directory existed (`CHARTER.md` §4.5, `docs/mechanisms/relation-read.md:81-87`).
  2. **`K = 1` 64-batch cadence-granularity stop in `#2831` (`instrument`, `mechanism_negative: false`):** In [`docs/labs/m1-a8-full-dose-rerun/README.md`](../../../labs/m1-a8-full-dose-rerun/README.md), the paired `Relation` (`LearnV1`, seed 1) and `FrozenZero` (`FrozenZeroV1`, seed 1) arms ran under the active D40 64-batch whole-interval rejection gate (`floor = 1,187 / 3,847` on the 3,847-row `TRAIN` `P_succ` anchor) with the `K = 1` single-coordinate greedy schedule (`rows_v1`). Protecting admission succeeded — `FrozenZero` passed `Q1` (`4,096` batches, `epoch = 258`) with `DEV b_3(P_succ) = 1,106 / 3,847`, `DEV b_3(Full) = 4,292 / 14,287`, and `DEV top1_correct = 871 / 14,287` (`+738` over init `133`, `5.85x` Q4 writer's `149`, `1.88x` best K16's `463`), while `Relation` committed `22` relation-gate updates (`42` sensitive) and reached `1,600 / 3,847` `TRAIN` anchor admission at batch `256` — before both arms halted on `STOP_FOR_DIAGNOSIS: consecutive_rejections` (`Relation` at batch `1,600`, `FrozenZero` at batch `5,248`) because each 64-batch audit window under `K = 1` bundles `4–11` local 64-row updates before a global 3,847-row `TRAIN` anchor check runs. Replacing `K = 1` with the `BVG-K` (`K = 16`) batched-coordinate Stage T learning law is `#2836` (`m1-bvg-k-stage-t-training`, Step 4 of `PLAN.md`).
  3. **Docs-only item scope (`instrument`):** `docs/roadmap/items/r-finite-relational-head.md` line 10 restricts `scope:` to `[docs/harness/prep/r-finite-relational-head*, docs/labs/r-finite-relational-head/**, docs/loop/tasks/r-finite-relational-head/**]`, excluding `crates/uor-r4-discrete/**`.

## What changed

- **Pre-registration sheet ([`docs/harness/prep/r-finite-relational-head-2026-10-25.md`](../../../harness/prep/r-finite-relational-head-2026-10-25.md)):** Authored the one-page pre-registration sheet (`CHARTER.md` §4.5) for the Stage M2 finite relational head discriminator (`H_relation` `ReadSpec::Relation { kept: 8, sweeps: 1 }` with 25 learned trits $g[e, \ell] \in \{-1, 0, +1\}$ across 5 link types $\times$ 5 tree levels vs `C_frozen_zero` $g \equiv 0 \equiv \text{Flock}$, `C_route` `ReadSpec::Route { kept: 8 }`, and `C_successor_only` frozen $g[\text{succ}, 0] = +1$) under the Stage T `BVG-K` (`K = 16`) + D40 admission-anchor law across two seeds (`seed = 1, 2`) against both the unconditional count tables (`U = 1,226 / 14,287`, `T2 = 6,300 / 14,287`, `T_code = 3,414 / 14,287`) and the W2 admission-matched count tables (`T_restr`), with non-author `TEST FITNESS: FIT` and `D68 DECISION REVIEW: PASS` posted at [`#2877` comment `6396409286`](https://github.com/UOR-Foundation/uor-r4/issues/2877#issuecomment-6396409286).
- **Lab synthesis ([`docs/labs/r-finite-relational-head/README.md`](../../../labs/r-finite-relational-head/README.md)):** Consolidated the complete `relation-read` evidence chain (`attention-preflight` `9/9` controls and `2,034 / 3,847` init `P_succ`, `a8-dose` ungated Q1 stop, W2 `H_mixed` read-code capacity `T_code = 3,414 / 14,287` and `T_restr = 1,138 / 1,920`, `#2830` CUDA qualification `5/5`, and `#2831` gated rerun `FrozenZero` `Q1` `DEV b_3(P_succ) = 1,106 / 3,847`, `b_3(Full) = 4,292 / 14,287`, `DEV top-1 = 871 / 14,287`, and `Relation` `1,600 / 3,847` `TRAIN` anchor peak at batch `256`) alongside the baseline count-table comparison table and the Stage T re-entry protocol.
- **Roadmap status (`docs/roadmap/items/r-finite-relational-head.md` and `docs/roadmap/README.md`):** Transitioned `r-finite-relational-head` from `status: ready` to `status: held` under the D68 decision review (`#2877` comment `6396409286`) until `#2836` (`m1-bvg-k-stage-t-training`) completes, and re-rendered `docs/roadmap/README.md`.
- **Size:** `6` files (`0` code, `0` tests, `5` docs `+209/-1`, `1` generated roadmap table `+3/-3`), `+212 / -4` lines against `origin/main` (`8cc6d7964cc6350eecba47d25576138eadcdf902`).

## Measurements

- **Consolidated baseline admission and ranking against the lawful count tables (`DEV` `n = 14,287` rows, `P_succ` `n = 3,847` rows; from sealed artifacts `a8-full-dose-rerun-20261025.tar.gz` `4a744188f14c934f89e57a1be83ecc8f2cb211620290a6539217837f809bae55` and W2 `#2845`):**
  - `U` (Unigram count table): `DEV top-1 = 1,226 / 14,287` (`8.58%`), `DEV top-8 = 4,060 / 14,287` (`28.42%`), admission-matched on Q4 (`n = 1,920`): `754 / 1,920` (`39.27%`).
  - `T2` / `G` (Query-token bigram count table): `DEV top-1 = 6,300 / 14,287` (`44.10%`), `DEV top-8 = 11,263 / 14,287` (`78.83%`), admission-matched on Q4 (`n = 1,920`): `1,482 / 1,920` (`77.19%`).
  - `T_code` (W2 exact read-code table): `DEV top-1 = 3,414 / 14,287` (`23.90%`), `DEV top-8 = 7,558 / 14,287` (`52.90%`), admission-matched `T_restr(code)` on Q4 (`n = 1,920`): `1,138 / 1,920` (`59.27%`).
  - Untrained init (`epoch = 0`, `Flock` / `Relation` $g \equiv 0$): `DEV b_3(P_succ) = 1,351 / 3,847` (`35.12%`), `DEV b_3(Full) = 4,244 / 14,287` (`29.71%`), `DEV top-1 = 133 / 14,287` (`0.93%`), `DEV top-8 = 714 / 14,287` (`5.00%`).
  - Untrained `SuccessorOnly` ($g[\text{succ}, 0] = +1$): `DEV b_3(P_succ) = 2,034 / 3,847` (`52.87%`, `+683` rows over `Flock`).
  - Sealed Q4 writer (`seed 1`, `epoch 407`, pre-D40): `DEV b_3(Full) = 1,920 / 14,287` (`13.44%`), `DEV top-1 = 149 / 14,287` (`1.04%`), `DEV top-8 = 747 / 14,287` (`5.23%`).
  - D40 `FrozenZero` at `Q1` (`batch = 4,096`, `epoch = 258`, `#2831`): `DEV b_3(P_succ) = 1,106 / 3,847` (`28.75%`), `DEV b_3(Full) = 4,292 / 14,287` (`30.04%`), `DEV top-1 = 871 / 14,287` (`6.10%`, `20.29%` of admitted rows), `DEV top-8 = 1,455 / 14,287` (`10.18%`).
  - D40 `Relation` (`LearnV1`) at batch `1,600` stop (`epoch = 132`, `#2831`): `TRAIN anchor b_3(P_succ) = 1,232 / 3,847` (peaked at `1,607 / 3,847` at batch `64` and `1,600 / 3,847` at batch `256`; `22` relation + `82` vector + `28` combine updates committed).

## Checks run at the head

- `CARGO_TARGET_DIR=local/target ~/.cargo/bin/cargo run -p xtask -- roadmap render` — PASS
- `CARGO_TARGET_DIR=local/target ~/.cargo/bin/cargo run -p xtask -- roadmap check` — PASS
- `uor-harness loop gates --full` — PASS (`11/11` gates green)
- `uor-harness loop report --check --order docs/loop/tasks/r-finite-relational-head/antigravity-65/ORDER.md` — PASS
- `uor-harness loop pr-check --local` — PASS (`0` FAIL, `0` WARN)

## Skills used

- `uor-research-architect`, `scope-guard`, `solution-integrity`, `gates`.

## Still open

- Executing the 4-arm $\times$ 2-seed GPU run (`H_relation`, `C_frozen_zero`, `C_route`, `C_successor_only`) under `docs/harness/prep/r-finite-relational-head-2026-10-25.md` once `#2836` (`m1-bvg-k-stage-t-training`, Step 4 of `PLAN.md`) completes and `r-finite-relational-head` scope includes `crates/uor-r4-discrete/**`.

## Plain summary

The finite relational head (`ReadSpec::Relation`) is a 25-trit discrete routing table ($g[e, \ell] \in \{-1, 0, +1\}$) that scores five exact structural link types (parent-child, sibling, sequential successor, same token, and same frame) across the five levels of the base-3 context tree without using floating-point numbers, matrix multiplication, or softmax. When the previous A8 full-dose rerun (`#2831`) finished, it showed that protecting cell admission (`b_3`, the fraction of evaluation rows where the target token's geometric cell is kept in the top 3 cells) raises the baseline model's exact next-token accuracy (`top-1`) nearly sixfold (`149 -> 871 / 14,287`), but that proposing one coordinate at a time (`K = 1`) with a 64-batch rollback window halts training early when a single coordinate move inside a 64-batch window dips below the admission floor. This unit writes and reviews the one-page pre-registration sheet (`docs/harness/prep/r-finite-relational-head-2026-10-25.md`), consolidates the baseline admission and ranking numbers against the unconditional and admission-matched count tables (`docs/labs/r-finite-relational-head/README.md`), and places `r-finite-relational-head` (`#2877`) in `held` status until the batched-coordinate Stage T training law (`BVG-K`, `K = 16`, `#2836`) completes.
