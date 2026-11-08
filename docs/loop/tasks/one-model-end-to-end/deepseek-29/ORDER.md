# ORDER: one-model-end-to-end, deepseek-29 (repair and run the merged extension)

Lab: deepseek. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md)
(#3407). Base: `56ef8a4feed0fec94b5954ebbfbfd6dca403af68`.

## Goal

Repair the post-merge runtime record and the D84 CPU-evaluation identity, then
execute the registered 2,048-step reply-SFT dose at one new root. Score every
checkpoint, apply the registered selector and outcome rows, seal and read back
the evidence, and delete the owned pod.

## Cancel probe

- PR #3845 is merged on `main` at `56ef8a4f`.
- The only diff after the reviewed head and before the merge is Claude's
  claude-315 documentation and one notebook row.
- The 1,024-step source seal remains immutable.
- `uor-pod status` must retain room inside four pods and $8/hour without
  touching Claude's pod.
- The new `s1-2048` store root must be absent.

## Files in scope

- `docs/loop/tasks/one-model-end-to-end/deepseek-29/**`
- `docs/harness/prep/one-model-end-to-end-reply-sft-2048-2026-11-07.md`
- `docs/labs/one-model-end-to-end/reply-sft-2026-11-07/**`
- `docs/research/notebook/float-training.md`

## Done when

1. The merged-runtime pin and exact normalized identity are recorded after fresh
   FIT and D67 reviews.
2. The sealed run identifies the CPU model, architecture and logical processor
   count used by D84 grading, and every grade carries the D84 substrate labels.
3. Checkpoints 256 through 2,048 are exported and scored by the unchanged
   selector; the job applies the original outcome rows.
4. The run seals within 100 MiB, reads back, and the owned pod is deleted.
5. The REPORT and notebook record the result without a capability, mechanism,
   M2, seed-reproducibility or useful-chat claim.

Never touch Claude's pods, branches, leases or files. Never weaken a hash, gate,
selector, stop, seal, CPU-identity check or geometry constraint. Never cancel a
healthy run for a self-set timer.
