# ORDER: infra-loop-harness, antigravity-80

Lab: antigravity. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Setup row: 2 (item-file scope maintenance, as in claude-90 and claude-108). Base: `bf92c9228`.

## Goal

Narrow the `scope:` of three `status: ready` roadmap items under a D68 decision review so their docs-only pre-registration sheet + `TEST FITNESS: FIT` units can be claimed in parallel while open PRs hold the model crates:

1. **`m1-normalization-mechanism` (`#2842`, milestone M1):** All three normalization arms (`N_snap`, `C_unscaled`, `C_raw`), the separate normalization run path (`claude-66`), DEV B3/paired/structural scoring (`claude-67`, `claude-68`), and the owner's `2026-10-24T03:41:41Z` non-inferiority margin (`claude-72`) are wired in `crates/uor-r4-discrete`. What blocks `#2842` now is updating `docs/harness/prep/m1-normalization-mechanism-DRAFT-2026-10-22.md` to resolve the `claude-57` `NOT_FIT` points (`claude-72` left the sheet to `antigravity`) and recording its fresh-context `TEST FITNESS: FIT` review. Removing `crates/uor-r4-discrete/**` from `scope:` allows claiming `#2842` for the docs-only sheet + FIT unit while `PR #3230` holds `crates/uor-r4-discrete/**`.
2. **`recurrence-gates-six-recurrence-fit` (`#2890`, research) and `spiralcore-fbs-offline-float-sheet` (`#2891`, research):** Both were unblocked by `1b5a6abe3` (`r-secant-surrogate-descent`, `#2886`), and neither has its required one-page pre-registration sheet or non-author `TEST FITNESS: FIT` review (`CHARTER.md` §4.5) yet. Removing `crates/uor-r4-float-train/**, crates/uor-r4-integer/**` for their pre-registration sheet + FIT units allows claiming them in parallel while `PR #3231` holds those crates.

Log the D68 decision as `[antigravity] owner log:` on `#2842`, `#2890`, and `#2891` (D69).

## Cancel probe (run first)

At `bf92c9228d18a211ea50ca6aadba8fb6c96225d4`:
- `uor-harness loop claim infra-loop-harness --lab-topic narrow-sheet-scopes` refused (`already claimed by PR #3169`). Checked `#3169`'s files (`PLAN.md`, `docs/integration/DECISIONS.md`, `docs/loop/tasks/infra-loop-harness/codex-1/**`) and all open PRs (`#3231`, `#3230`, `#3169`): zero file overlap with `#2842`, `#2890`, `#2891`, or this task folder. Hand-written under the umbrella item exception (`run-antigravity.txt` Part 3.2).
- `uor-harness loop next` skips `#2842` (crate overlap with `#3230`) and `#2890`/`#2891` (crate overlap with `#3231`), even though the next unit on each is its docs-only pre-registration sheet + `TEST FITNESS: FIT` review.
- **Cancel probe result (`instrument`):** Confirmed roadmap scope bottleneck; proceed with the D68 scope narrowing.

## Files in scope

- `docs/roadmap/items/m1-normalization-mechanism.md`
- `docs/roadmap/items/recurrence-gates-six-recurrence-fit.md`
- `docs/roadmap/items/spiralcore-fbs-offline-float-sheet.md`
- `docs/roadmap/README.md`
- `docs/loop/tasks/infra-loop-harness/antigravity-80/**`

## Done when

- `m1-normalization-mechanism.md`, `recurrence-gates-six-recurrence-fit.md`, and `spiralcore-fbs-offline-float-sheet.md` scopes are narrowed to their docs-only pre-registration sheet + FIT scopes and `cargo run -p xtask -- roadmap check` passes.
- Fresh-context D68 decision review and D66 exact-head review pass, and `[antigravity] owner log:` notes are posted on `#2842`, `#2890`, and `#2891`.
- `uor-harness loop report --check` and `uor-harness loop pr-check --local` pass (`References #2904`, `References #2842`, `References #2890`, `References #2891`).

## Stop and ask if

- Never stop and wait. If any sheet + FIT unit requires crate changes first, keep the crate glob on that item and narrow only the others.
