# ORDER: cpu-training-pilot, deepseek-208 (release serving claim)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md)
(#3506). Base: `a8d9a2e16`.

## Goal

Record protected delivery and fresh-main verification of deepseek-207, then
remove its completed coordination claim.

PR #3765 merged as `a8d9a2e16dc551f09a74c233e6fe640753db97f9`. Fresh main
passes 473 serving library tests with one expected ignore, 19 OpenAI loopback
tests, and the 394-test CPU suite with four expected ignores. The persisted
seed-1 serving report reproduces after timing and absolute-path removal.

## Scope

- `docs/harness/claims.json`
- `docs/loop/tasks/cpu-training-pilot/deepseek-207/REPORT.md`
- `docs/loop/tasks/cpu-training-pilot/deepseek-208/**`

No source or runtime changes.

## Done when

1. The reviewed head, protected merge and fresh-main checks are recorded.
2. The `deepseek/prompt-retrieval-serving` claim is removed.
3. The release PR passes policy checks and merges.

## Stop rule

Stop on source, evidence, test, merge-identity or unexpected-path drift.

Lab: deepseek
Order: docs/loop/tasks/cpu-training-pilot/deepseek-208/ORDER.md
