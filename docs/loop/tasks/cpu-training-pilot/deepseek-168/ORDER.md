# ORDER: cpu-training-pilot, deepseek-168 (release learned-copy claim)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506). Base: `f579da597`.

## Goal

Close the bookkeeping for the completed deepseek-167 learned geometric copy
read experiment.

## Fixed facts

- implementation and evidence merged through PR #3712 as `f579da597`;
- fresh-main CPU library: 302 passed, 4 expected ignores;
- fresh-main CPU binary tests: 83 passed;
- published evidence manifest and no-`.u16` boundary pass on fresh main;
- the experiment is a bounded **REJECT**;
- no model, serving, CUDA, float-training, termination or Claude-owned source
  changed.

## Files in scope

- `docs/harness/claims.json`
- `docs/loop/tasks/cpu-training-pilot/deepseek-168/**`
- `docs/loop/tasks/cpu-training-pilot/deepseek-167/REPORT.md`

## Done when

1. Remove `deepseek/copy-learned-read` from the claims registry.
2. Record merge and fresh-main verification in the deepseek-167 REPORT.
3. Obtain fresh exact-head review and merge through the protected queue.
4. Verify the release on fresh main and remove the task worktree.
