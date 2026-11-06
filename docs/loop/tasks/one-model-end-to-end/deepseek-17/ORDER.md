# ORDER: one-model-end-to-end, deepseek-17 (registered cache inventory)

Lab: deepseek. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md)
(#3407). Base: `a9a0f75ac`.

## Goal

Make the current-checkpoint centroid capture produce the cache inventory required by the
merged registered CPU evaluator. The scorer must keep its ordinary DEV output but be able
to omit DEV from a query cache, leaving exactly TRAIN plus the four registered panels.

## Cancel probe

Class: instrument repair. The capture path starts a cache split for every scored split,
including its built-in DEV split. The registered CPU evaluator requires exactly five
no-memory splits: `train` plus `oasst2`, `wiki-00000`, `wiki-00020` and `wiki-00040`.
Without this option every GPU capture would be rejected after capture. No model, panel,
bar or decision changes.

## Done when

1. `--query-cache-dev 0` omits only the DEV cache split while preserving the ordinary
   scored DEV files and every model decision.
2. The option defaults to the prior behavior and is refused without `--query-cache-out`.
3. A focused test pins the split-selection rule.
4. The 294-test float library suite, scoped Clippy and the D23/D25/D32 ratchets pass.

## Stop and ask if

Ask means decide, record and continue.

- Omitting the DEV cache split would change any scored file or forward decision.
- The registered evaluator would need a relaxed inventory to accept the capture.

## Files in scope

- `crates/uor-r4-float-train/src/in_space_long.rs`
- `crates/uor-r4-float-train/src/bin/train-in-space.rs`
- `docs/loop/tasks/one-model-end-to-end/deepseek-16/**`
- `docs/loop/tasks/one-model-end-to-end/deepseek-17/**`
