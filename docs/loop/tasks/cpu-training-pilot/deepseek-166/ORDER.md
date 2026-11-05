# ORDER: cpu-training-pilot, deepseek-166 (release geometric-copy claim)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Base: `38ab13578`.

## Goal

Close the bookkeeping for the completed deepseek-165 geometric-copy
experiment.

## Fixed facts

- implementation and evidence merged through PR #3710 as `38ab13578`;
- fresh-main CPU library: 295 passed, 4 expected ignores;
- fresh-main CPU binary tests: 82 passed;
- published evidence manifest and no-`.u16` boundary pass on fresh main;
- the experiment is a bounded **REJECT**;
- no model, serving, CUDA, float-training, termination or Claude-owned source
  changed.

## Files in scope

- `docs/harness/claims.json`
- `docs/loop/tasks/cpu-training-pilot/deepseek-166/**`
- `docs/loop/tasks/cpu-training-pilot/deepseek-165/REPORT.md`

## Done when

1. Remove `deepseek/copy-geometric` from the claims registry.
2. Record merge and fresh-main verification in the deepseek-165 REPORT.
3. Obtain fresh exact-head review and merge through the protected queue.
4. Verify the release on fresh main and remove the task worktree.
