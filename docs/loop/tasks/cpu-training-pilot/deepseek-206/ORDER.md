# ORDER: cpu-training-pilot, deepseek-206 (final claim cleanup)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md)
(#3506). Base: `35d770b04`.

## Goal

Remove the final `deepseek/prompt-retrieval-release` coordination claim after
PR #3762 recorded and protected-merged the deepseek-204 delivery.

## Scope

- `docs/harness/claims.json`
- `docs/loop/tasks/cpu-training-pilot/deepseek-205/REPORT.md`
- `docs/loop/tasks/cpu-training-pilot/deepseek-206/**`

No source or runtime changes.

## Done when

1. The final prompt-retrieval release claim is absent.
2. PR #3762 and its protected merge are recorded.
3. The cleanup PR passes policy checks and merges without source changes.

## Stop rule

Stop on source, claim or unexpected-path drift.

Lab: deepseek
Order: docs/loop/tasks/cpu-training-pilot/deepseek-206/ORDER.md
