# ORDER: r-ranking-with-admission-intact, antigravity-68

Lab: antigravity. Item: [r-ranking-with-admission-intact](../../../../roadmap/items/r-ranking-with-admission-intact.md). Base: `c364b6c66`.

## Goal

The item's done-when reads:

> A run with admission demonstrably intact reports ranking results (top-1 among admitted rows) against the count tables.

Under the item's `docs/**` scope, this unit runs the cancel probe first, authors the one-page pre-registration sheet (`docs/harness/prep/r-ranking-with-admission-intact-2026-10-25.md`) and lab record (`docs/labs/r-ranking-with-admission-intact/README.md`) reporting the exact `#2831` `Q1` (`4,096`-batch) admission-intact ranking decomposition (`871 / 4,292` conditional `top-1`, `1,455 / 4,292` conditional `top-8`, `256 / 256` cells, $\bar{n} = 126.49$) alongside `#2845` (`W2`) admission-matched count tables and pre-registering the full `16,404`-batch 2-seed `BVG-K` Stage T run, obtains the fresh-context non-author `TEST FITNESS: FIT` review (CHARTER §4.5) and `D68` decision review (`status: ready -> status: held` until `#2836` `m1-bvg-k-stage-t-training` completes), and posts the review on `#2884`.

## Cancel probe (run first)

- **Class:** `instrument` (missing pre-registration sheet + `#2831` halted at batch `1,600` on `Relation` and batch `5,248` on `FrozenZero` due to `K = 1` cadence-granularity coupling before reaching `16,404` batches across `2` seeds; gated on `#2836` `m1-bvg-k-stage-t-training`).
- **Command & Evidence:** Inspected `docs/labs/m1-a8-full-dose-rerun/{README.md,dose-summary.json}`, sealed `#2831` GPU evaluations `baseline-dev.json` (`884b9a8f...`) and `q1-dev.json` (`6b924afa...`) in `a8-full-dose-rerun-20261025.tar.gz` (`4a744188...`), `docs/labs/m1-w2-capacity-capture/README.md`, and `docs/mechanisms/admission.md`.
- **Findings:**
  1. No pre-registration sheet `docs/harness/prep/r-ranking-with-admission-intact*` or lab record `docs/labs/r-ranking-with-admission-intact/**` exists yet (`instrument` missing).
  2. In `#2831` (`m1-a8-full-dose-rerun`), `FrozenZero` reached `Q1` (`4,096` batches, `epoch = 258`) on RTX 5090 with admission intact: `occupied_cells = 256 / 256`, $\bar{n} = 1,807,135 / 14,287 = 126.49$ tokens/row, `TRAIN anchor b_3(P_succ) = 1,258 / 3,847` (`>= 1,187` floor), `DEV b_3(P_succ) = 1,106 / 3,847`, and `DEV b_3(Full) = 4,292 / 14,287` (`+48` over `4,244`).
  3. With admission intact at `Q1`, `FrozenZero` raised `DEV top-1` from `133 / 14,287` (`3.13%` of `4,244` admitted rows) to **`871 / 14,287`** (**`20.29%` conditional `top-1` among `4,292` admitted rows**, `679` strict wins + `192` tie wins; `1.88x` `K16-A`'s `463 / 14,287` and `71.04%` of `1,226` Unigram at `1/4` dose), and raised `DEV top-8` from `513 / 14,287` (`12.09%`) to **`1,455 / 14,287`** (**`33.90%` conditional `top-8`**), leaving `3,421 / 4,292` admitted rows (`79.71%`) unranked at `top-1`.
  4. Both `#2831` arms stopped early (`Relation` at `1,600`, `FrozenZero` at `5,248`) on `STOP_FOR_DIAGNOSIS: consecutive_rejections` (`mechanism_negative: false`) from `K = 1` updates + `64`-batch rollback. Completing a full `16,404`-batch, 2-seed (`CHARTER §4.3`) run requires `m1-bvg-k-stage-t-training` (`#2836`, Step 4 of `PLAN.md`).

## Files in scope

- `docs/harness/prep/r-ranking-with-admission-intact*`
- `docs/labs/r-ranking-with-admission-intact/**`
- `docs/loop/tasks/r-ranking-with-admission-intact/**`
- Always-allowed paths: `docs/roadmap/items/r-ranking-with-admission-intact.md`, `docs/roadmap/README.md`

## Done when

- Cancel probe committed first in `ORDER.md`.
- `docs/harness/prep/r-ranking-with-admission-intact-2026-10-25.md` and `docs/labs/r-ranking-with-admission-intact/README.md` authored and reviewed (`TEST FITNESS: FIT` + `D68` decision review posted on `#2884`).
- `r-ranking-with-admission-intact` set to `status: held`, `docs/roadmap/README.md` rendered, and `REPORT.md` passes all gates.

## Stop and ask if

- Never stop to wait; decide via `D68` decision review and log `[antigravity] owner log:` on `#2884`.
