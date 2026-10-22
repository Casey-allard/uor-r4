# REPORT: legal-construction-exact-solve, claude-47 (the exact solver and its finite controls)

Lab: claude. Base: `18c03cebc`. Order: [ORDER.md](ORDER.md), committed first (`de63c7569`). The cancel probe found no solver under `docs/labs/legal-construction-exact-solve/`, so the task stands.

## What did not work, first

- **The tableau will not scale to the saved case as written.** The LP is a dense two-phase tableau, with upper bounds as explicit rows. At 1,920 variables it would be about 4,300 rows by 8,600 columns of big rationals. It is correct, and verified against brute force at small size, but unit 3 needs a bounded-variable or revised simplex, or a reduction first.
  - **A useful reduction:** each of the 1,900 on-grid coefficients enters only through `k/4`. Unit 3 should time one root LP before running branch-and-bound.
- **On-grid masters with |4m| > 7 follow the spec literally:** `q0` is clamped to ±7 and `δ` stays `k/4`. Whether the saved case has any is unit 3's first check.
- **Under a planted fault, cargo stops at the first failing test binary.** So the 240-seed random suite was not run under the faults; the 11 control tests caught all three.
- **The solver was built by a builder subagent.** I re-ran its gates myself.

## What changed

| Commit | What |
|---|---|
| `de63c7569` | This ORDER, alone. |
| `e38c4a9d5` | See the list below. |
| this REPORT's commit | This REPORT. |

What `e38c4a9d5` adds, as `docs/labs/legal-construction-exact-solve/solver/`: a standalone crate detached from the workspace, with offline dependencies only (`num-bigint`/`num-rational` 0.4).
- **`exact`:** bit-exact f32 to dyadic `BigRational`, which refuses NaN and infinity, and the only f32 in the crate.
- **`lp`:** the two-phase tableau with Bland's rule.
- **`bb`:** depth-first branch-and-bound.
  - It branches on fractional `l`/`r` before `k`, down branch first.
  - Pruning is strict, so `δ = 0` is returned when no move strictly improves.
  - It has a node cap.
- **`check`:** an independent exact evaluation of the guards and the objective, recomputing `q0` its own way.
- **`brute`:** exhaustive enumeration over legal codes, as the oracle.

## Measurements

These are host controls, not a result about the model.

- **The 240-seed random suite:** the solver's optimum equals brute force on every seed.
  - 203 seeds have an improving move.
  - 133 seeds use a fractional move.
  - 157 seeds improve with guards active.
- **Search:** 1,967 branch-and-bound nodes in total.
- **Timing:** 11.7–45 s in a debug build, mostly brute force. One N = 5 root LP (11 variables, 11 rows) takes 16.7 ms.

## Checks run at the head

- `cargo test --locked --offline`: 14 passed (11 controls, the no-floats check, 2 random and timing).
- `cargo clippy --locked --offline --all-targets -D warnings`: exit 0.
- **Planted faults,** each caught (the builder's run):
  - a flipped guard sign fails 4 control tests;
  - the `k` upper bound off by one (`8 − q0`) fails 3;
  - the dropped disjunction row `k ≤ −l + (7−q0)r` fails `hand_fractional_higher_to_endpoint`.
  - After each, the file was restored byte-identical (`cmp`), and the suite re-ran green.
- **No floats:** f32 and f64 appear only in `src/exact.rs`, confirmed by grep.
- `loop report --check` and `loop pr-check --local`: see the PR body.

## Skills used

scope-guard: one new crate under the item's lab folder. The saved case is not read.

## Still open (unit 3)

- **The inputs:** settle which saved files are `g` and `m`, their family order and their hashes, from the original source at commit `ebd6ecaa` (in the local object store).
- **The LP:** replace or reduce it for N = 1,920, and time one root LP.
- **The run:** branch-and-bound with the recorded 4,096-node cap, then `check`, the notebook row on the new page (#3004), and `Closes #2828`.

## Plain summary

The exact solver now exists and gives the same answers as trying every option on hundreds of small puzzles. It is not yet fast enough for the real 1,920-variable puzzle, which is the next step.
