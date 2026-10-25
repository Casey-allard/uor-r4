# ORDER: r-finite-relational-head, antigravity-65

Lab: antigravity. Item: [r-finite-relational-head](../../../../roadmap/items/r-finite-relational-head.md). Base: `8cc6d7964`.

## Goal

Author and FIT-review the one-page pre-registration sheet (`docs/harness/prep/r-finite-relational-head-2026-10-25.md`, `CHARTER.md` §4.5) and lab evidence synthesis (`docs/labs/r-finite-relational-head/README.md`) for `r-finite-relational-head` (`#2877`, M2 `#2030`, line `m1-discrete-attention`) — the Stage M2 finite relational head discriminator (`ReadSpec::Relation { kept: 8, sweeps: 1 }` with 25 learned trits $g[e, \ell] \in \{-1, 0, +1\}$ across 5 link types $\times$ 5 levels vs `FrozenZero` $g \equiv 0 \equiv \text{Flock}$, `Route` `ReadSpec::Route { kept: 8 }`, and `SuccessorOnly` level-0 sequential-successor ablation) under the `BVG-K` (`K = 16`) + D40 admission-anchor law against the lawful count tables (`unigram = 1,226 / 14,287`, `T2 = 6,300 / 14,287`, and the W2 admission-matched count tables `T_restr` and `T_code = 3,414 / 14,287`) — and transition `r-finite-relational-head` to `status: held` under a fresh-context D68 decision review until `#2836` (`m1-bvg-k-stage-t-training`, Step 4 of `PLAN.md`) completes.

## Cancel probe (run first)

Ran on `origin/main` at `8cc6d7964cc6350eecba47d25576138eadcdf902`:

1. `ls docs/harness/prep/r-finite-relational-head* docs/labs/r-finite-relational-head` -> no pre-registration sheet or lab directory exists (`instrument` missing).
2. `docs/labs/m1-a8-full-dose-rerun/README.md` (`#2831`, PR `#3192`): the paired A8 admission-preserving rerun under `K = 1` greedy single-coordinate updates + 64-batch whole-interval rollback (`cadence = 64`, `margin = 24`, `floor = 1,187 / 3,847`) hit `STOP_FOR_DIAGNOSIS: consecutive_rejections` at batch `1,600` on `Relation` (`25` audits, `132` committed updates = `82` vector + `28` combine + `22` relation, restored `TRAIN anchor = 1,232 / 3,847`, peak `1,607 / 3,847` at batch `64` and `1,600 / 3,847` at batch `256`) and at batch `5,248` on `FrozenZero` (`82` audits, `274` committed updates, restored `TRAIN anchor = 1,249 / 3,847`, after `FrozenZero` passed `Q1` at batch `4,096` with `DEV b_3(P_succ) = 1,106 / 3,847`, `b_3(Full) = 4,292 / 14,287`, and `DEV top1_correct = 871 / 14,287`).
3. Under `K = 1`, each 64-batch window bundles `4–11` local 64-row updates before a global 3,847-row `TRAIN` anchor check runs, so one admission-eroding coordinate move rolls back the entire 64-batch window (`instrument` cadence-granularity coupling, `mechanism_negative: false`). Moving to `BVG-K` (`K = 16`) batched-coordinate Stage T updates is `#2836` (`m1-bvg-k-stage-t-training`, Step 4 of `PLAN.md`).
4. `docs/roadmap/items/r-finite-relational-head.md` line 10 restricts `scope:` to `[docs/harness/prep/r-finite-relational-head*, docs/labs/r-finite-relational-head/**, docs/loop/tasks/r-finite-relational-head/**]`, excluding `crates/uor-r4-discrete/**`.

- **Classification:** **`instrument`** (pre-registration sheet and lab directory not yet authored; `#2836` Stage T still open; scope excludes `crates/uor-r4-discrete/**`).

## Files in scope

- `docs/harness/prep/r-finite-relational-head*`
- `docs/labs/r-finite-relational-head/**`
- `docs/loop/tasks/r-finite-relational-head/**`
- Always-allowed paths (`docs/loop/README.md` lines 103, 190–194): `docs/roadmap/items/r-finite-relational-head.md` (`status:` line only) and `docs/roadmap/README.md`.

## Done when

- `docs/harness/prep/r-finite-relational-head-2026-10-25.md` and `docs/labs/r-finite-relational-head/README.md` are committed, a fresh-context non-author `TEST FITNESS: FIT` + D68 decision review (`status: ready -> status: held` until `#2836` completes) is posted on `#2877`, and `uor-harness loop gates --full` + `uor-harness loop pr-check --local` pass.

## Stop and ask if

- Never stop to wait; decide via D68 review and log on `#2877`.
