# ORDER: one-model-end-to-end, deepseek-19 (checkpoint-parameterized centroid inputs)

Lab: deepseek. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md)
(#3407). Base: `3f41847a3`.

## Goal

Remove the hard-coded n3S checkpoint identity from the merged centroid capture package,
and bind its reviewed input model to the current human-text champion. At this base, that
champion is Claude-301's (b64L): width 256, step 16,384, batch 64 built as four
micro-batches of 16, at seeds 1 and 2.

The package must keep the scorer and CPU evaluator fixed. This unit changes how an exact
checkpoint and archive are described and verified, not what either program computes.
It does not run the capture, evaluation, a GPU job or a training job.

## Cancel probe

Class: execution-package repair. Claude-302's width-1,024 diagnostic is merged and did
not produce a competing checkpoint; the report explicitly keeps (b64L) at width 256 as
the current human-text recipe. Claude-301's two b64L archives are sealed in
`results/claude/one-model-end-to-end-d3-accum-2026-11-06/runs-2/s{1,2}/`.

The ordinary claim command is blocked by the preserved local worktree
`.worktrees/deepseek-centroid-cache` on `deepseek/centroid-cache-current`. That branch's
PR, #3804, is merged, but its local worktree holds ignored evidence and is not touched.
The owner explicitly approved this retarget, so this unit starts in a new worktree from
fresh `main` and records that the claim helper was bypassed rather than overriding or
deleting the preserved worktree.

## Done when

1. A separate machine-readable input specification names each seed's sealed archive,
   archive hash, run case, checkpoint step, run config and three checkpoint files.
2. The values in that specification are computed from the two (b64L) archives and checked
   against their inner `SHA256SUMS.files`. Byte-identical model-config files may share the
   old hash, but no selected value is accepted without matching the b64L archive.
3. The capture job requires the specification, validates its schema and selected seed,
   and uses it for archive download, extraction, hash checks and configuration checks.
4. The controller copies and hashes the specification beside the job, and the CPU restore
   extracts the same selected files through the same specification.
5. The sheet identifies (b64L) as the current target, retains the n3S package as history,
   and leaves `TEST FITNESS: PENDING` with capture blocked.
6. `bash -n`, `shellcheck`, `git diff --check`, `prep-check`, the loop report and the local
   PR checks pass. No model, cache, pod or GPU run occurs.

## Stop and ask if

Ask means decide, record and continue.

- The selected step-16,384 checkpoint is absent, duplicated, or disagrees with its
  registered config or file hashes. The archive also contains the training trace's earlier
  checkpoints and the b64S arm; selecting step 16,384 from b64L is intended.
- Its run or model config differs from width 256, context 192, six layers, four Hamming
  heads, objective `nearest`, or `train_as_served: true`.
- The same checkpoint file hashes unlike its inner manifest.
- Retargeting would touch Claude's active sources, an existing pod or another lab's files.

## Files in scope

- `docs/labs/one-model-end-to-end/centroid-cache-2026-11-06/**`
- `docs/harness/prep/one-model-end-to-end-centroid-*`
- `docs/loop/tasks/one-model-end-to-end/deepseek-19/**`
