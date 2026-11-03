# ORDER: cpu-training-pilot, deepseek-103 (correct superseded CPU readings)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Base: `6729e5d507bd39ef400c6515bc29b75f97e682d7`.

## Goal

Repair the final fresh-context CHANGES review of PR #3620 at
`fa4bb0b73de1169efd620f7cccb1c20d0ecdddc1`. The review found the technical
timing repair closed but identified three stale CPU-utilization ranges in the
superseded deepseek-100 report.

## Files in scope

- `docs/loop/tasks/cpu-training-pilot/deepseek-100/REPORT.md`
- `docs/loop/tasks/cpu-training-pilot/deepseek-102/**`
- `docs/loop/tasks/cpu-training-pilot/deepseek-103/**`

## Done when

1. The deepseek-100 CPU-utilization ranges exactly match all retained timing
   logs.
2. The active deepseek-103 REPORT records the corrected values and exact PR
   size.
3. Report check, exact-diff check, evidence checksums, local PR check and full
   gates pass.
4. A fresh exact-head review approves the resulting head.

## Stop and ask

Do not change timing evidence, code, the worker recommendation or any model
result.
