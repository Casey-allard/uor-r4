# ORDER: one-model-end-to-end, deepseek-16 (current-checkpoint centroid)

Lab: deepseek. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md)
(#3407). Base: `a9a0f75ac`.

## Goal

Complete the roadmap's frozen-trunk centroid-refit step on the stable D3/8,192-step
checkpoint at seeds 1 and 2. Capture exact no-memory teacher-forced queries and
decisions once, then evaluate the original table, a TRAIN mean-centroid table and a
matched shuffled-label control on CPU over the same 1,460,697 human-panel rows.

## Cancel probe

Class: evaluation. The query-cache instrument is merged. The existing scorer exposes
the exact Q12 query and table; Claude-297 sealed both checkpoints and published the
TRAIN and panel hashes. The cache must also capture TRAIN, which the scorer did not do
in its first unit; this ORDER adds that bounded capture and no-memory selection.

The private store and Vast credentials are installed. The local unit can finish code,
tests, pre-registration and the exact execution commands without touching either busy
existing pod.

## Done when

1. `--query-cache-out` captures the configured TRAIN split as cache split `train`,
   while leaving all normal scored outputs and forward decisions unchanged.
2. `--query-cache-memory 0|1` captures only the requested memory mode and refuses use
   without `--query-cache-out`.
3. `--query-cache-dev 0` omits the built-in DEV split from the cache while leaving its
   ordinary scored output unchanged, so the registered `train + four panels` inventory
   can be produced.
4. The sheet pins D3, its four held-out panels, both archive hashes and the exact
   n3S checkpoint paths/hashes required by the run.
5. A non-author FIT and a fresh D67 pre-run review are recorded before capture.
6. Two seed caches are captured on one leased RTX 5090, sealed and uploaded.
7. The CPU evaluator reproduces every frozen panel decision and reports original,
   centroid and shuffled control by panel and pooled across all four panels.
8. The result lands in a REPORT and notebook row, with zeros exact and a mechanism
   verdict that does not claim model capability.

## Stop and ask if

Ask means decide, record and continue.

- A private-store token, Vast credential or `uor-pod` configuration is absent. Stop
  before any external action and leave the exact command for the owner.
- A checkpoint archive lacks exactly one n3S step-008192 checkpoint or its inner
  `SHA256SUMS` fails.
- The original cached decisions differ from the frozen scorer; status is
  `CONTROL_FAULT`, not a centroid result.
- Disk headroom is below the projected raw cache, archive, readback and reserve:
  stop before capture or sealing rather than deleting another run.

## Files in scope

- `crates/uor-r4-float-train/src/in_space_long.rs`
- `crates/uor-r4-float-train/src/bin/train-in-space.rs`
- `docs/harness/prep/one-model-end-to-end-centroid-*`
- `docs/labs/one-model-end-to-end/centroid-cache-2026-11-06/**`
- `docs/loop/tasks/one-model-end-to-end/deepseek-16/**`
- `docs/research/notebook/float-training.md`
- `docs/roadmap/items/one-model-end-to-end.md`
