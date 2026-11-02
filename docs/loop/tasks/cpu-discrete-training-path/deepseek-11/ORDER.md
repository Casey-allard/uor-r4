# ORDER: exact BVG-K joint decision and batch apply, deepseek-11

Lab: deepseek. Item: [cpu-discrete-training-path](../../../../roadmap/items/cpu-discrete-training-path.md) (#3501). Base: `f1405cc10`.

## Goal

Reproduce the exact BVG-K member summary, joint-vs-best commit decision and atomic multi-coordinate apply over disjoint proposal records. Keep cursor persistence/replay disabled.

## Scope

- `crates/uor-r4-cpu-train/**`
- `docs/loop/tasks/cpu-discrete-training-path/deepseek-11/**`
- `docs/research/notebook/cpu-training.md`
- `docs/roadmap/items/cpu-discrete-training-path.md`

No serving edit, corpus/GPU run or language claim.

## Done when

- Members are exactly the records with valid status, eligibility, nonzero improving lane and in-range index.
- Summary count, best ordinal and best selected loss match CUDA.
- Joint commits all members only when its loss is strictly below the best member; ties commit the single best member.
- Missing joint loss with at least two members refuses status 17.
- Batch apply validates every member before writes, refusing kind mismatch 14, stale epoch 3, mode 9, inventory/lane 4, non-improving 5 and duplicate indices 15.
- A valid batch applies all disjoint moves and increments epoch exactly once; empty commit refuses 16.
- Full release tests, clippy, formatting, ratchets and loop gates pass.
- REPORT/notebook classify cursor persistence as remaining.
