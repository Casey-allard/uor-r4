# ORDER: cpu-training-pilot, deepseek-82

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506). Base: `de9cfc84a`.

## Goal

Complete the BVG-K table-coordinate pivot after deepseek-81's matched
four-Q12 control correctly stopped on admission loss. Compare one coordinate
and four coordinates at the same one-Q12 per-coordinate amplitude, with fresh
zero-update controls.

This isolates spatial breadth from the larger per-coordinate step already
tested by deepseek-9.

## Work completed

- deepseek-81 implemented and tested the opt-in K4 table block, checkpoint
  identity, coordinate telemetry and fail-closed driver gate at `8a0defae5`.
- Its seed-1 four-Q12 K1 control changed 278/384/396
  top-1/top-8/admitted to 278/383/395 and exited 2 under the registered rule.
- K4 and seed 2 were therefore not run.

## Scope

- `docs/harness/prep/cpu-training-pilot-table-bvgk-same-amplitude-2026-11-03.md`
- `docs/loop/tasks/cpu-training-pilot/deepseek-82/**`
- `docs/research/notebook/cpu-training.md`
- `docs/research/cpu-training-program.md`
- `docs/roadmap/items/cpu-training-pilot.md`

## Done when

1. A new FIT sheet fixes baseline, K1-step-1 and K4-step-1 arms on both seeds,
   the exact 3-sigma top-1 rule, admission, export and stop outcomes.
2. Six bounded CPU runs execute, with K1 admission loss reported but not used
   as a reason to skip the K4 mechanism.
3. K4's accepted-coordinate telemetry proves at least one multi-coordinate
   accepted block and reports its histogram.
4. The notebook and roadmap record a replicated gain or the pivot negative.
5. A fresh exact-head review approves the PR.

## Stop if

An artifact, sample, export or K4-admission instrument fails; in that case keep
the arm and classify instrument against mechanism before retrying.
