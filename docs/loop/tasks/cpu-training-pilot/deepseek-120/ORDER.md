# ORDER: cpu-training-pilot, deepseek-120 (correct cell-gate report)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md)
(#3506). Base: `24dc560285b12aea2b92682c00b485814534217d`.

## Finished before this repair

- Deepseek-119 added and FIT-reviewed the writer-cell termination gate.
- Both public seeds and the fresh seed-1 repeat completed.
- Fresh exact-head review of PR #3647 verified the mechanism and evidence and
  returned CHANGES for two report values.

## Findings

1. REPORT says source/ORDER/prep changes are `+647/-25`; the reviewed snapshot
   is `+738/-22`, while the full PR is 12 files, `+1087/-22`.
2. REPORT incorrectly says cell 1 produced six or seven additional terminal
   observations. The other six terminal cells hold exactly ten terminal rows.

## Goal

Correct only REPORT size and terminal-count prose. No source, experiment input,
arm, bar, result or evidence changes.

## Files in scope

- `docs/loop/tasks/cpu-training-pilot/deepseek-119/**`
- `docs/loop/tasks/cpu-training-pilot/deepseek-120/**`
- `docs/research/notebook/cpu-training.md`
- `docs/roadmap/items/cpu-training-pilot.md`

## Done when

1. REPORT states the exact source/ORDER/prep and full-PR size.
2. REPORT states that cells other than 1 hold ten terminal rows.
3. Report, formatting and loop gates pass.
4. Fresh exact-head review, protected merge and fresh-main verification pass.

## Stop

Stop if this repair requires source, evidence, experiment or claim changes.
