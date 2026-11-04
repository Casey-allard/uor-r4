# ORDER: cpu-training-pilot, deepseek-116 (repair premature-stop success)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md)
(#3506). Base: `f4192c14d9e1f2ed425667dbfdd40f729322b112`.

## Finished before this repair

- `1dd53c193` added the termination-point gate and controls.
- `951ce09c7` separated corpus EOS from decoder BOS.
- `5a31ab8cb` recorded both seeds and the seed-1 repeat.
- Fresh exact-head review of PR #3645 returned CHANGES.

## Findings

1. The paired success predicate used raw BOS-complete count, so a future
   candidate whose BOS gains were all premature could pass when exact count did
   not fall. This contradicts the registered rule that premature stops do not
   count as success.
2. The REPORT said the branch changed seven files while the reviewed diff had
   12.

## Goal

Count only non-premature BOS completions for candidate success while retaining
raw BOS and premature-stop readings. Require `premature_stop <= bos_complete`,
add a regression proving an all-premature candidate cannot pass, correct the
file count, rerun both seeds and the seed-1 repeat, and repeat FIT and
exact-head review.

The mechanism, training replay, selection, radius, gates, arms, cap and
candidate bar remain unchanged.

## Files in scope

- `crates/uor-r4-cpu-train/**`
- `docs/harness/prep/cpu-training-pilot-termination-gate-*`
- `docs/loop/tasks/cpu-training-pilot/deepseek-114/**`
- `docs/loop/tasks/cpu-training-pilot/deepseek-115/**`
- `docs/loop/tasks/cpu-training-pilot/deepseek-116/**`
- `docs/research/notebook/cpu-training.md`
- `docs/roadmap/items/cpu-training-pilot.md`

## Done when

1. Candidate success uses non-premature BOS completion; all-premature gain
   fails in a finite test.
2. The REPORT's file count and accounting fields match the final diff.
3. Fresh FIT clears the repaired comparison source.
4. Both seeds and the seed-1 repeat reproduce the original negative.
5. Tests, Clippy, formatting, evidence and loop gates pass.
6. Fresh exact-head review, protected merge and fresh-main verification pass.

## Stop

Stop on a changed empirical verdict without a code/input explanation, evaluation
leakage, post-observation tuning, hash drift, nondeterminism or counter
underflow.
