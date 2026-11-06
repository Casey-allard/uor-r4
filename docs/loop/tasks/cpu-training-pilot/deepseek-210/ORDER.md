# ORDER: cpu-training-pilot, deepseek-210 (release VSA claim)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md)
(#3506). Base: `b8e01c13b`.

## Goal

Record protected delivery and fresh-main verification of deepseek-209, then
remove its completed coordination claim.

PR #3767 merged as `b8e01c13bb07c69c9484a9a850489ce3c245d631`. Fresh main
retains the reviewed VSA source and evidence, passes 474 serving library tests
with one expected ignore, 19 API tests, and 394 CPU tests with four expected
ignores. The persisted seed-1 VSA report reproduces after timing/path removal.

## Scope

- `docs/harness/claims.json`
- `docs/loop/tasks/cpu-training-pilot/deepseek-209/REPORT.md`
- `docs/loop/tasks/cpu-training-pilot/deepseek-210/**`

No source or runtime changes.

## Done when

1. Delivery and fresh-main evidence are recorded.
2. The `deepseek/prompt-vsa` claim is removed.
3. The bookkeeping PR passes policy and merges.

## Stop rule

Stop on source, evidence, test, merge-identity or unexpected-path drift.

Lab: deepseek
Order: docs/loop/tasks/cpu-training-pilot/deepseek-210/ORDER.md
