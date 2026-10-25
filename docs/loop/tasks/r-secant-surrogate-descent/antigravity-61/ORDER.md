# ORDER: r-secant-surrogate-descent, antigravity-61

Lab: antigravity. Item: [r-secant-surrogate-descent](../../../../roadmap/items/r-secant-surrogate-descent.md). Base: `c95883c5e`.

## Goal

The item's done-when reads:

> A recorded descent or no-descent result for each of the recurrence gates and SpiralCore/FBS, with instrument versus mechanism classified.

## Cancel probe (run first)

Ran the repository state and dependency probe at `c95883c5ea464d4defbf3f154a18abcd7ae462d4`:
1. `ls -d docs/harness/prep/r-secant-surrogate-descent* docs/labs/r-secant-surrogate-descent` -> neither a pre-registration sheet nor a lab note exists yet for `r-secant-surrogate-descent` (`#2886`).
2. `git show origin/main:docs/roadmap/items/float-training-module.md` -> `status: ready` (`crates/uor-r4-float-train` has not been created yet; `CHARTER.md` §3.7).
3. Inspected `crates/uor-r4-training/src/cuda_stack.cu:259-291` (`rate_select_bwd`) and `crates/uor-r4-training/src/spiralcore_fbs.cu:124-183` (`spiralcore_fbs_bwd`): both secant surrogates live only inside `crates/uor-r4-training` until extracted into `crates/uor-r4-float-train` (`#2823`).

- **Cancel-probe result:** **DID NOT CANCEL** (`instrument`). The item lacks its one-page pre-registration sheet (`docs/harness/prep/r-secant-surrogate-descent-2026-10-25.md`), non-author `TEST FITNESS: FIT` review (`CHARTER.md` §4.5), and lab protocol record (`docs/labs/r-secant-surrogate-descent/README.md`), and its GPU execution is gated on `float-training-module` (`#2823`). This unit stages the pre-registration sheet, obtains its fresh-context `TEST FITNESS: FIT` review, writes the lab protocol note, and records a fresh-context D68 decision review setting `r-secant-surrogate-descent` to `status: held` until `#2823` lands.

## Plan (expected files > 3)

1. `docs/harness/prep/r-secant-surrogate-descent-2026-10-25.md`: write the one-page pre-registration sheet defining the two-mechanism secant-surrogate descent test (`rate_select_bwd` and `spiralcore_fbs_bwd`) against zero-credit and random-refit controls across two seeds (`seed = 1`, `seed = 2`), instrument checks, and decision table.
2. `docs/labs/r-secant-surrogate-descent/README.md`: write the lab design and readiness record documenting the exact secant equations, the instrument vs. mechanism classification, the `TEST FITNESS: FIT` review, and the dependency hold on `float-training-module` (`#2823`).
3. Fresh-context subagent reviews (`CHARTER.md` §4.5, §5.1, D68): run a fresh-context `TEST FITNESS: FIT` review and a fresh-context D68 decision review to transition `r-secant-surrogate-descent` to `status: held` on `float-training-module` (`#2823`), logging both on `#2886`.
4. `docs/roadmap/items/r-secant-surrogate-descent.md` and `docs/roadmap/README.md`: set `status: held` and re-render `docs/roadmap/README.md`.

## Files in scope

- `docs/harness/prep/r-secant-surrogate-descent*`
- `docs/labs/r-secant-surrogate-descent/**`
- `docs/loop/tasks/r-secant-surrogate-descent/**`

## Done when

`docs/harness/prep/r-secant-surrogate-descent-2026-10-25.md` is staged and reaches non-author `TEST FITNESS: FIT` (`uor-harness check` reports zero `prep.fit_before_run` warnings on `r-secant-surrogate-descent`); `docs/labs/r-secant-surrogate-descent/README.md` documents the protocol and instrument/mechanism classification; `r-secant-surrogate-descent.md` is set to `status: held` under a logged D68 decision review until `float-training-module` (`#2823`) lands; and `uor-harness loop gates --full` passes.

## Stop and ask if

- Never wait on Casey or another lab (`CHARTER.md` §5.1, D65–D69). Stop only if a change requires editing an owner-only path.
