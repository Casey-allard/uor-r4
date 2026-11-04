# ORDER: cpu-training-pilot, deepseek-117 (repair report provenance)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md)
(#3506). Base: `f4e8ee7bbaecb027be6d34793e5de0fbb4e29964`.

## Finished before this repair

- Deepseek-114 added the termination gate.
- Deepseek-115 separated corpus EOS from decoder BOS and ran the two-seed
  negative from fresh roots.
- Deepseek-116 repaired candidate success accounting and reran both seeds plus
  the seed-1 repeat.
- Fresh exact-head review of PR #3645 returned CHANGES for report provenance.

## Findings

1. The final REPORT cites FIT at the superseded `951ce09c…` source without
   naming the final `8af603b0…` accounting repair.
2. The REPORT records a volatile `uor-harness check` warning count that changed
   from 123 to 124 during coordination.

## Goal

Correct only REPORT provenance: name both FITs and replace the volatile warning
count with the stable zero-error result. No source, experiment input, arm,
bar, result or evidence changes.

## Files in scope

- `docs/loop/tasks/cpu-training-pilot/deepseek-116/**`
- `docs/loop/tasks/cpu-training-pilot/deepseek-117/**`

## Done when

1. REPORT names FIT at `951ce09c…` for EOS repair and at `8af603b0…` for
   non-premature accounting.
2. REPORT records zero `uor-harness check` errors without a volatile warning
   count.
3. Report, formatting and loop gates pass at the exact head.
4. Fresh exact-head review, protected merge and fresh-main verification pass.

## Stop

Stop if the repair requires a source, evidence, experiment, arm or bar change.
