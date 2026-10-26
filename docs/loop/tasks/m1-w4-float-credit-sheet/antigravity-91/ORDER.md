# ORDER: m1-w4-float-credit-sheet, antigravity-91

Lab: antigravity. Item: [m1-w4-float-credit-sheet](../../../../roadmap/items/m1-w4-float-credit-sheet.md). Base: `5a104c6f8`.

## Goal

The item's done-when reads:

> Sheet FIT'd and run once; integer top-1 through the D11 export reported against both count tables; export gap within the two-seed band on the same evaluator.

And `What is missing` (`docs/roadmap/items/m1-w4-float-credit-sheet.md:19`, `PLAN.md:58, 93`) states:

> W4 is conditional: only if W2 = H_law and W3 (BVG-K) fails. It is one decimal-training run in a separate scanned module (float is legal in offline training only, D37), about $5 to $10 GPU plus build work, with a disqualifying export-gap rule fixed per sheet at FIT time (D39 item 5)... Preparation that runs no compute (code, finite CPU tests, the pre-registration sheet) can proceed now.

In this unit (`antigravity-91`):
1. Complete the non-compute preparation (`docs/harness/prep/m1-w4-float-credit-sheet-DRAFT-2026-10-26.md` and `docs/labs/m1-w4-float-credit-sheet/README.md`) defining the W4 sheet, Stage-1 export-gap probe, D39 item 5 disqualifying export-gap rule (`<= 398` top-1 rows on the `14,287`-row DEV panel), and matched arms against `#2831` (`K = 1`) and `#2836` (`BVG-K` `T16`), with a fresh-context non-author `TEST FITNESS: FIT` review.
2. Run a fresh-context D68 whole-project decision review (`DECISION.md`, logged as `[antigravity] owner log:` on `#2846`) weighing `#2845` (`W2 = H_mixed`), `#3179` (`QB = B2`, `G1` declined), `#2836` (`Stage T`, routing to `#3267` and `#2847` and dropping `#2846` in `antigravity-84/DECISION.md:17`), and `#2820`, and set `docs/roadmap/items/m1-w4-float-credit-sheet.md` to `status: held` (`status_only_edit`).

## Cancel probe (run first)

Ran on `/Users/casey.allard/uor-r4/.worktrees/antigravity-m1-w4-float-credit-sheet` (`b59e06ea7453ddc28d1d9fe1b97cb875b421362a`):
1. `ls docs/harness/prep/m1-w4-float-credit-sheet* docs/labs/m1-w4-float-credit-sheet/`: both absent.
2. Checked the conditional run gate (`PLAN.md:58, 93`, `m1-w4-float-credit-sheet.md:19`, *"only if W2 = H_law and W3 (BVG-K) fails"*):
   - `W2` (`#2845`, `docs/labs/m1-w2-capacity-capture/README.md:5, 27-29, 54`) returned **`H_mixed`** (`T = 3,414 <= M + F = 6,435`): *"The W4 float-credit sheet runs only on H_law, so it does not run."*
   - `W2` utilization (`#3179`, `claude-95/REPORT.md:71-73`) returned `QB = B2` (`T_proj = 3,552 < 6,435`), declining `G1`.
   - `Stage T` (`#2836`, `claude-107/DECISION.md:34`, `antigravity-84/DECISION.md:17`) dropped `#2846` routing and routed to `#3267` (`m1-within-cell-writer`) and `#2847` (`m1-wider-code-carrier`).
   - Classification: **mechanism** preparation (FIT the sheet + D39 item 5 export-gap rule) + **instrument/sequence** D68 status transition (`status: ready -> held`, `$0` GPU spent).

## Files in scope

- `crates/uor-r4-discrete/**`
- `docs/harness/prep/m1-w4-float-credit-sheet*`
- `docs/labs/m1-w4-float-credit-sheet/**`
- `docs/loop/tasks/m1-w4-float-credit-sheet/**`
- `docs/roadmap/items/m1-w4-float-credit-sheet.md` (`status_only_edit` + `docs/roadmap/README.md`)

## Done when

- `docs/harness/prep/m1-w4-float-credit-sheet-DRAFT-2026-10-26.md` passes `uor-harness check` (`prep.sheet_shape`) and holds a fresh-context non-author `TEST FITNESS: FIT` verdict fixing the D39 item 5 disqualifying export-gap rule and Stage-1 probe.
- `docs/loop/tasks/m1-w4-float-credit-sheet/antigravity-91/DECISION.md` passes a fresh-context D68 decision review (`[antigravity] owner log:` on `#2846`), `docs/labs/m1-w4-float-credit-sheet/README.md` records the synthesis and bars, and `docs/roadmap/items/m1-w4-float-credit-sheet.md` is set to `status: held` (`References #2846`).

## Stop and ask if

- Fresh-context D68 decision review disagrees with holding GPU execution while `W2 = H_mixed` (`T = 3,414 <= M + F = 6,435`) and `#3267` (`m1-within-cell-writer`) / `#2820` (`d11-exporter-float-trunk`) are pending.
