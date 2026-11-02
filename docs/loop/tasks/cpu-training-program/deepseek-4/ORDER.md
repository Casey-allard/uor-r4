# ORDER: reconcile the CPU training program with merged evidence, deepseek-4

Lab: deepseek. Item: [cpu-training-program](../../../../roadmap/items/cpu-training-program.md) (#3507). Base: `45540f256`.

## Goal

Bring the durable CPU-training program, roadmap summaries, notebook links and lab claims back into agreement with the work merged through `45540f256`, without changing code or claiming an untested training result.

## Scope

- `docs/research/cpu-training-program.md`
- `docs/roadmap/README.md`
- `docs/roadmap/items/cpu-training-program.md`
- `docs/roadmap/items/cpu-discrete-training-path.md`
- `docs/roadmap/items/cpu-quantization-exact-memory.md`
- `docs/roadmap/items/cpu-shared-vector-train.md`
- `docs/roadmap/items/cpu-training-pilot.md`
- `docs/loop/tasks/cpu-training-program/deepseek-4/**`
- `docs/harness/claims.json`

No serving edit, code edit, corpus-scale training or language claim.

## Done when

- Every merged CPU result through `45540f256` is linked once and described with its actual scope.
- The historical-family matrix retains one supported/repaired or explicitly rejected disposition per family.
- The completed discrete-path item keeps the roadmap parser's required section while stating that no component is missing.
- The child-item table records #3501/#3502/#3504/#3510 as closed or done, #3503 as finite mechanisms delivered with trained-float and complete-reply outcomes not run, #3505 as trained-artifact/corpus-driver components delivered with the pilot not run, and #3506 as not admitted.
- Stale DeepSeek claims for branches whose work is already on `origin/main` are removed; other labs' claims are preserved.
- Harness documentation checks applicable to the touched files, formatting and a fresh exact-head review pass.

## Cancel if

The reconciliation would require inventing evidence, changing a project bar, editing another lab's active work or changing an executable path.
