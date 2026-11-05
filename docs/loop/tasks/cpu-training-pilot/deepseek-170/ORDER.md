# ORDER: cpu-training-pilot, deepseek-170 (release lookahead claim)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506). Base: `4ec22a118`.

## Goal

Close bookkeeping for the completed deepseek-169 exact-copy lookahead
experiment.

## Fixed facts

- implementation and control-fault evidence merged through PR #3715 as
  `4ec22a118`;
- fresh-main CPU library: 305 passed, 4 expected ignores;
- fresh-main CPU binary tests: 83 passed;
- published evidence manifest and no-`.u16` boundary pass on fresh main;
- the registered control was invalid because target-shuffled lookahead changed
  zero decisions;
- the primary lookahead was also strongly negative;
- no serving, CUDA, float-training, termination or Claude-owned source changed.

## Files in scope

- `docs/harness/claims.json`
- `docs/loop/tasks/cpu-training-pilot/deepseek-170/**`
- `docs/loop/tasks/cpu-training-pilot/deepseek-169/REPORT.md`

## Done when

1. Remove `deepseek/copy-lookahead` from the claims registry.
2. Record merge and fresh-main verification in the deepseek-169 REPORT.
3. Obtain fresh exact-head review and merge through the protected queue.
4. Verify on fresh main and remove the task worktree.
