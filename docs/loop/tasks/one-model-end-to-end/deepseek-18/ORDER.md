# ORDER: one-model-end-to-end, deepseek-18 (centroid capture delivery)

Lab: deepseek. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md)
(#3407). Base: `3a9c2f11d`.

## Goal

Deliver an exact, reviewable execution package for the current-checkpoint centroid
experiment without running it or touching either busy existing pod. The package must
capture the registered TRAIN plus four-panel cache at both seeds, preserve all evidence,
restore it on the D84 CPU host, and bind the evaluation to the captured checkpoint/cache
identity. This is the n3S development slice; it does not complete the roadmap's other
sealed cells or substitute for them.

## Cancel probe

Class: evaluation package. The D84 evaluator and the scorer's `--query-cache-dev 0`
inventory repair are already merged. The checkpoint and panel pins are available, but the
sheet is PENDING and both existing pods are busy. Preparation can finish without compute;
the capture remains blocked on FIT and the D67 pre-run review.

## Done when

1. The sheet uses the merged evaluator CLI, separate seed reports and the aggregate.
2. The capture job names the verified source pin, uses a clean bootstrap checkout or
   creates an exact job-owned clone, uses an immutable remote run id, and refuses an
   existing remote output.
3. GPU, disk, compression and upload/readback resources are checked; watchers renew
   healthy leases and no self-set wall clock cancels the job.
4. Both seed caches are explicitly limited to exactly five no-memory splits: TRAIN plus
   the four registered panels.
5. CPU restore verifies the checkpoint archive, cache archive, manifest and checkpoint
   hashes before evaluation.
6. `bash -n`, `git diff --check`, `prep-check` structure and the loop report pass.

## Stop and ask if

Ask means decide, record and continue.

- Any existing busy pod would be touched.
- A no-memory cache would contain DEV, Fresh V2, memory-on data or an extra panel.
- Disk, time or spend cannot stay within the registered limits.
- A cache or checkpoint cannot be tied back to its pinned seed and archive.

## Files in scope

- `docs/labs/one-model-end-to-end/centroid-cache-2026-11-06/**`
- `docs/harness/prep/one-model-end-to-end-centroid-*`
- `docs/loop/tasks/one-model-end-to-end/deepseek-16/**`
- `docs/loop/tasks/one-model-end-to-end/deepseek-18/**`
- `docs/research/notebook/float-training.md`
