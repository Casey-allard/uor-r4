# ORDER: recurrence-gates-six-recurrence-fit, antigravity-82

Lab: antigravity. Item: [recurrence-gates-six-recurrence-fit](../../../../roadmap/items/recurrence-gates-six-recurrence-fit.md). Base: `e5f276619`.

## Goal

The item's done-when reads:

> The six-recurrence fit runs after surrogate descent is shown and reports integer DEV metrics against a control.

This unit (`antigravity-82`) delivers the mandatory pre-execution preparation (`CHARTER.md` §4.5, `recurrence-gates-six-recurrence-fit.md` lines 36–37, and `docs/mechanisms/recurrence-gates.md` reopen condition 1):
1. Author the one-page D37 offline-float pre-registration sheet `docs/harness/prep/recurrence-gates-six-recurrence-fit-2026-10-25.md` with all 8 `REQUIRED_BLOCKS` (`bar`, `base`, `split`, `steps`, `stop`, `cost`, `power`, `fit`), pinning the 4 matched six-recurrence geometric arms (`Rec_Geometric_S3_Active`, `Rec_Geometric_U1_AbelianControl`, `Rec_Geometric_IdentityTransport_Control`, `Rec_FrozenRates_Null`) plus the GPU train-split count-table baselines across seeds `1` and `2`, standalone CUDA device qualification of `rate_select_fwd`/`bwd` and `quaternion_scan_fwd`/`bwd`, the D11 integer table export binding (`(rate_index: u4, transport_action)` + Q12 `i32` `NativeVectorWriter`) on all `14,287` `ft/dev` targets, `tokens/s` (`CHARTER.md` §1), and fresh V2 transfer (`14,479` targets).
2. Run a fresh-context non-author `TEST FITNESS` review (`CHARTER.md` §4.5) and D68 decision review (`CHARTER.md` §5.1) of the sheet, record `TEST FITNESS: FIT`, and log `[antigravity] owner log:` on `#2890`.
3. Update `docs/mechanisms/recurrence-gates.md` with the registered sheet and `TEST FITNESS: FIT` pointer.

## Cancel probe (run first)

Verify on host (`instrument` class) before authoring the sheet that:
1. `r-secant-surrogate-descent` (`#2886`, the sole `depends_on` of `recurrence-gates-six-recurrence-fit`) is `status: done` on `main` with `G-Rec-Descent = PASS` recorded in `docs/labs/r-secant-surrogate-descent/README.md`.
2. `crates/uor-r4-float-train/src/{geometric_stack_native.rs,geometric_stack_native_cuda.rs,cuda_stack.cu}` exist on `main` with `RecGates::Geometric`, `RATE_TABLE`, `KEEP_TABLE`, `rate_select_fwd`/`bwd`, and `quaternion_scan_fwd`/`bwd` and zero banned tokens (`d23`/`d25`).

**Cancel probe result (`2026-10-25`, class: `instrument` — PASS):**
- `r-secant-surrogate-descent.md` is `status: done` on `main` (`G-Rec-Descent = PASS` at `1b5a6abe3`).
- `geometric_stack_native.rs` (`03aec162…`), `geometric_stack_native_cuda.rs` (`03cc8a11…`), and `cuda_stack.cu` (`c577fce9…`) are present on `main` and pass `d23`/`d25` ratchets.

## Files in scope

- `docs/mechanisms/recurrence-gates.md`
- `docs/harness/prep/recurrence-gates-six-recurrence-fit*`
- `docs/labs/recurrence-gates-six-recurrence-fit/**`
- `docs/loop/tasks/recurrence-gates-six-recurrence-fit/**`

## Done when

1. `docs/harness/prep/recurrence-gates-six-recurrence-fit-2026-10-25.md` exists, stays within the one-page prep-sheet budget, and passes `uor-harness check` and `cargo test --no-fail-fast -p uor-r4-harness --lib preprules` with all 8 `REQUIRED_BLOCKS` (`bar`, `base`, `split`, `steps`, `stop`, `cost`, `power`, `fit`).
2. A fresh-context non-author reviewer (`CHARTER.md` §4.5, D68) issues `TEST FITNESS: FIT` and `DECISION REVIEW: APPROVE`, recorded in the sheet and logged on `#2890` via `[antigravity] owner log:`.
3. `docs/mechanisms/recurrence-gates.md` cites the registered sheet, and `uor-harness loop gates --full` and `uor-harness loop pr-check --local` pass.

## Stop and ask if

- The fresh-context reviewer returns `TEST FITNESS: NOT_FIT` for a reason that cannot be fixed within two revision rounds: fix all structural/specification findings in-unit and re-verify before closing.
