# ORDER: native-space-state-contract, codex-6

Lab: codex. Item: [native-space-state-contract](../../../../roadmap/items/native-space-state-contract.md).
Parent: [codex-5](../codex-5/ORDER.md). Reviewed head: `d12c29a11`.

## Why this repair exists

The finalized `main` ORDER parser requires the item link to begin on the `Item:` line. Every
ORDER in this repair chain wraps the link onto the following line, so the active `codex-5` order
is not valid. PR #3968 has also been retargeted from the merged control-plane branch to `main`,
and its tracked PICKUP still describes the old stacked state.

## Goal

Make the existing CUDA repair chain valid under the finalized parser, record the exact checks at
the merged head, and refresh the next-step handoff without changing model code or Q31 arithmetic.

## Files in scope

- `docs/loop/tasks/native-space-state-contract/codex-1/ORDER.md`
- `docs/loop/tasks/native-space-state-contract/codex-2/ORDER.md`
- `docs/loop/tasks/native-space-state-contract/codex-3/ORDER.md`
- `docs/loop/tasks/native-space-state-contract/codex-4/ORDER.md`
- `docs/loop/tasks/native-space-state-contract/codex-5/ORDER.md`
- `docs/loop/tasks/native-space-state-contract/codex-5/PICKUP.md`
- `docs/loop/tasks/native-space-state-contract/codex-5/REPORT.md`
- `docs/loop/tasks/native-space-state-contract/codex-6/**`

## Size

About 30 to 70 documentation lines.

## Done when

1. Every ORDER in the chain names the same roadmap item on its `Item:` line.
2. `loop pr-check --local` passes the ORDER rule at the resulting head.
3. The tracked PICKUP names fresh `main`, PR #3968's `main` base, and integer-serving as the next
   task.
4. The codex-5 REPORT records the merged-head CUDA, discrete, harness, formatting, Clippy and
   qualification-example results.
5. `loop report --check`, `git diff --check` and the changed documentation checks pass.

## Stop

- Do not change Q31 arithmetic, CUDA source or serving code.
- Do not rewrite completed evidence.
- Do not add another harness layer or validation framework.
- STOP and report when the metadata repair is complete.
