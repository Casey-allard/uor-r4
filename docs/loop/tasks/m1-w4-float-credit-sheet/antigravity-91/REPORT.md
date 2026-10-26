# REPORT: m1-w4-float-credit-sheet, antigravity-91

Lab: antigravity. Order: [ORDER.md](ORDER.md). Decision review: [DECISION.md](DECISION.md) ([owner log on #2846](https://github.com/UOR-Foundation/uor-r4/issues/2846#issuecomment-6414439464), fresh-context `FIT` and `D68` `PASS`). Item: [m1-w4-float-credit-sheet](../../../../roadmap/items/m1-w4-float-credit-sheet.md) (Issue [`#2846`](https://github.com/UOR-Foundation/uor-r4/issues/2846)).

References #2846

## What did not work

- **The conditional GPU run (`$5–$10`) is held at the cancel probe under `PLAN.md:58, 93` and `D68` (`classification: instrument / sequence`):**
  - `PLAN.md:58, 93` and [`docs/roadmap/items/m1-w4-float-credit-sheet.md:19`](../../../../roadmap/items/m1-w4-float-credit-sheet.md) condition `W4` on `W2 = H_law` and `W3 (BVG-K)` failing.
  - In [`m1-w2-capacity-capture`](../../../labs/m1-w2-capacity-capture/README.md) (`#2845`, `claude-87`), the exact read-code count table scored `T = 3,414 / 14,287` against `M + F = 6,435`, resolving **`H_mixed`** (`3,021` rows below `H_law`) and explicitly recording (`README.md:54`): *"The W4 float-credit sheet runs only on H_law, so it does not run."*
  - In [`m1-w2-code-utilization`](../../../labs/m1-w2-code-utilization/README.md) (`#3179`, `claude-95`), the stride-subsample projection `T_proj = 3,552` remained `2,883` rows below `H_law` (`6,435`), declining `G1` (`QB = B2`).
  - In [`m1-bvg-k-stage-t-training`](../../../labs/m1-bvg-k-stage-t-training/README.md) (`#2836`, `antigravity-84`), discrete `BVG-K` (`K = 16`) raised full-DEV target-cell admission to `5,009 / 14,287` (`G-Adm` PASS) and lifted the checkpoint-restricted count tables on those exact `5,009` admitted cells to `T_restr(A_W) = 2,140` and `G_restr(A_W) = 3,084` (both above `U = 1,226`), localizing the `455` vs `2,140` / `3,084` gap to parameterless `OuterL1V1` within-cell selection ([`m1-within-cell-writer`](../../../../roadmap/items/m1-within-cell-writer.md), `#3267`) and carrier capacity ([`m1-wider-code-carrier`](../../../../roadmap/items/m1-wider-code-carrier.md), `#2847`), with both [`claude-107/DECISION.md:34`](../../m1-bvg-k-stage-t-training/claude-107/DECISION.md) and [`antigravity-84/DECISION.md:17`](../../m1-bvg-k-stage-t-training/antigravity-84/DECISION.md) dropping `#2846` from Stage T's fallback routing.
  - Because `D77` (`claude-82`) had batch-flipped `#2846` to `status: ready` before `#2845`, `#3179`, and `#2836` ran, `uor-harness loop next` offered `#2846` ahead of `#2847` (`m1-wider-code-carrier`) and `#3267` (`m1-within-cell-writer`), locking `crates/uor-r4-discrete/**`.

## What changed

- **One-page W4 pre-registration sheet (`docs/harness/prep/m1-w4-float-credit-sheet-DRAFT-2026-10-26.md`):**
  - Completed the non-compute preparation deliverable (`m1-w4-float-credit-sheet.md:19-24, 38`) and stamped `TEST FITNESS: FIT` after fresh-context non-author review (`454cc9f1-3f1d-4b16-b291-4df23b2b23eb`, [`#2846#issuecomment-6414439464`](https://github.com/UOR-Foundation/uor-r4/issues/2846#issuecomment-6414439464)).
  - Fixed the D39 item 5 disqualifying export-gap rule (`B1`: `\Delta_{\text{top1}} \le 398`, `\Delta_{\text{top8}} \le 748`, `\Delta_{\text{adm}} \le 930` on the `14,287`-row DEV panel, `0` post-snap `UORLUT01` mismatches), the mandatory `256`-batch Stage-1 export-gap probe before any full `16,404`-batch Stage-2 run, the 5 matched arms (`W4-Float-Exported`, `W3-BVG-K`, `W1-K1`, `W4-RandomRefit`, `W4-ZeroStep`), and the Stage 2 / §4.8 integer quality, discrete non-inferiority, admission, and zero-multiply serving bars (`B2..B5`).
- **D68 decision card & lab synthesis (`DECISION.md`, `docs/labs/m1-w4-float-credit-sheet/README.md`):**
  - Recorded the Stage 2 comparative baseline table across `#2840`, `#2831`, `#2836`, `#2845`, and `#3179`, and passed a fresh-context D68 whole-project decision review (`454cc9f1-3f1d-4b16-b291-4df23b2b23eb`).
- **D68 status transition (`docs/roadmap/items/m1-w4-float-credit-sheet.md`, `docs/roadmap/README.md`):**
  - Transitioned `m1-w4-float-credit-sheet.md` from `status: ready` to `status: held` (`status_only_edit` per `crates/uor-r4-harness/src/loopprcheck.rs:2352`) and re-rendered `docs/roadmap/README.md` via `cargo run -p xtask -- roadmap render`.

## Measurements

- **Registered `W4` Stage 2 & D39 item 5 bars (`docs/harness/prep/m1-w4-float-credit-sheet-DRAFT-2026-10-26.md`):**
  - Frozen `14,287`-row DEV panel (`ft/dev/tokens.u16` `0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b`, `ft/dev/response_mask.u8` `a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078`).
  - `B1` disqualifying export-gap band: `\Delta_{\text{top1}} \le 398` rows (`2.79%`), `\Delta_{\text{top8}} \le 748` rows (`5.24%`), `\Delta_{\text{adm}} \le 930` rows (`6.51%`).
  - Sealed discrete comparator baselines on pinned `A8` (`6600bdb1…`): checkpoint-0 `top1 = 133`, `admitted = 4,244` (`81,533 tokens/s` direct writer in `#2840`); `W1-K1` (`#2831`) `T1_fz` `Q1` `top1 = 871`, `admitted = 4,292`; `W3-BVG-K` (`#2836`) `T16_fz` `top1 = 455`, `top8 = 1,484`, `admitted = 5,009`, `T_restr(A_W) = 2,140`, `G_restr(A_W) = 3,084`; `W2` (`#2845`) `T(Q4) = 3,414`, `U = 1,226`, `G = 6,300` (`M + F = 6,435`).
- **GPU spend:** **`$0.00`** (`0` pods rented).

## Checks run at the head

- `CARGO_TARGET_DIR=local/target ~/.cargo/bin/cargo run --offline -p xtask -- roadmap render`: `PASS` (`94` items rendered).
- `CARGO_TARGET_DIR=local/target ~/.cargo/bin/cargo run --offline -p xtask -- roadmap check`: `PASS` (`94 items OK`).
- `uor-harness check`: `PASS` (`0` errors on `docs/harness/prep/m1-w4-float-credit-sheet-DRAFT-2026-10-26.md`).
- `uor-harness loop report --check --order docs/loop/tasks/m1-w4-float-credit-sheet/antigravity-91/ORDER.md`: `PASS`.
- `uor-harness loop pr-check --local`: `PASS`.
- `uor-harness loop gates --full`: `PASS` (`11 / 11` gates `GREEN`).

## Skills used

- **solution-integrity:** Evaluated the whole-project Stage 2 evidence chain (`#2845`, `#3179`, `#2831`, `#2836`, `#3267`, `#2820`) before spending any GPU budget, confirming that `W2 = H_mixed` rules out the `H_law` trigger for `W4`.
- **scope-guard:** Restricted the roadmap item edit on `docs/roadmap/items/m1-w4-float-credit-sheet.md` to a `status_only_edit` (`status: ready -> held`) and left `crates/uor-r4-discrete` untouched.

## Still open

- **Active discrete Stage 1/2 M1 items now unblocked in `uor-harness loop next`:**
  - [`m1-wider-code-carrier`](../../../../roadmap/items/m1-wider-code-carrier.md) (`#2847`)
  - [`m1-within-cell-writer`](../../../../roadmap/items/m1-within-cell-writer.md) (`#3267`)
- **Conditional `W4` execution (`#2846`):** Stays `status: held` with its pre-registration sheet (`docs/harness/prep/m1-w4-float-credit-sheet-DRAFT-2026-10-26.md`) FIT-cleared and ready if the discrete within-cell (`#3267`) and carrier (`#2847`) paths fail to close the ranking gap and `#2820` (`d11-exporter-float-trunk`) clears the float export-gap bar.

## Plain summary

Item `#2846` (`m1-w4-float-credit-sheet`, W4) is a fallback training experiment that uses decimal (floating-point) numbers during offline training—and then converts the result back into whole numbers for serving—only if earlier tests (`W2` and `W3`) showed that the model's discrete learning rule was the main bottleneck. However, the completed `W2` test (`#2845` and `#3179`) and `W3` test (`#2836`) showed that the discrete model already finds the right word drawer on `5,009` out of `14,287` test examples (where simple lookup tables inside those drawers score `2,140` and `3,084` correct predictions), and that the real bottleneck is picking the right word *inside* the chosen drawer (`#3267` `m1-within-cell-writer`) and expanding the code carrier (`#2847` `m1-wider-code-carrier`). In this unit, we wrote and FIT-approved the complete one-page W4 test plan and export-gap rule so it is ready on the shelf if ever needed, and set `#2846` to `held` (`$0` GPU spent) so the loop immediately moves to `#2847` and `#3267`.
