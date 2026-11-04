# ORDER: cpu-training-pilot, deepseek-118 (repair final bookkeeping)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md)
(#3506). Base: `6b30d30d597eef15f57c5c7857f16c070c72347c`.

## Finished before this repair

- Deepseek-114 added the termination gate.
- Deepseek-115 separated corpus EOS from decoder BOS and ran the two-seed
  negative from fresh roots.
- Deepseek-116 repaired non-premature success accounting and repeated both
  seeds plus the seed-1 repeat.
- Deepseek-117 corrected FIT and warning-count provenance.
- Fresh exact-head review of PR #3645 verified code and evidence but found two
  bookkeeping mismatches.

## Findings

1. Deepseek-117's files-in-scope list omitted the notebook and roadmap files
   changed by its commit.
2. The final REPORT said the unit adds three ORDERS; the final unit contains
   five.

## Goal

Correct the final ORDER scope and REPORT count, then verify the exact head.
No source, experiment input, arm, bar, result or numeric evidence changes.

## Files in scope

- `docs/loop/tasks/cpu-training-pilot/deepseek-114/**`
- `docs/loop/tasks/cpu-training-pilot/deepseek-115/**`
- `docs/loop/tasks/cpu-training-pilot/deepseek-116/**`
- `docs/loop/tasks/cpu-training-pilot/deepseek-117/**`
- `docs/loop/tasks/cpu-training-pilot/deepseek-118/**`
- `docs/research/notebook/cpu-training.md`
- `docs/roadmap/items/cpu-training-pilot.md`

## Done when

1. The active ORDER names every task, notebook and roadmap path changed by the
   unit.
2. REPORT states that the final unit contains five ORDERS and keeps the final
   code and evidence identities unchanged.
3. Report, formatting and loop gates pass at the exact head.
4. Fresh exact-head review, protected merge and fresh-main verification pass.

## Stop

Stop if this repair requires a source, evidence, experiment, arm or bar change.
