# REPORT: one-model-end-to-end, deepseek-16 (current-checkpoint centroid)

Lab: deepseek. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md)
(#3407). Order: [ORDER.md](ORDER.md). Sheet:
[one-model-end-to-end-centroid-2026-11-06](../../../../harness/prep/one-model-end-to-end-centroid-2026-11-06.md).
Base: `ffe65bcd3`.

## What did not work, first

- **The merged cache could not fit a centroid.** Its first capture path recorded DEV,
  Fresh V2 and requested panels, but not TRAIN. The evaluator's `--train train` would
  therefore fail after expensive compute. This unit adds a cache-only TRAIN capture.
- **The merged cache wrote both memory modes.** On D3 that is 27,416,554 TRAIN queries,
  about 29 GiB per mode. This unit adds `--query-cache-memory 0|1`; the registered run
  requests no-memory only.
- **The real capture cannot run on this workstation.** The private Hugging Face store
  returns HTTP 401 without an owner token, and there is no `uor-pod` config, Vast key,
  compute SSH key or running GPU. `uor-pod status` reports no free GPU or offer. The
  sheet is therefore PENDING, not FIT, and no panel was opened.

## What changed

- `score_rows` accepts the configured `--train-split` and captures it under the cache
  split name `train` when a query cache is requested.
- The TRAIN cache uses the same Q12/served-forward choice, memory state, seeds and
  `row_picks` decoder as evaluation splits. It is not added to normal scored outputs.
- `--query-cache-memory 0|1` selects one memory mode for caching and is refused unless
  `--query-cache-out` is present.
- The focus test checks both modes are captured by default and exactly one when selected.
- A current-checkpoint sheet pins D3, both n3S archives, all four held-out panels and a
  two-seed zero/centroid/shuffle decision rule.

No training, optimizer, serving or existing scoring decision changed.

## Measurements

Nothing model-level was measured. `uor-harness prep-check` reads the new sheet as
complete but `PENDING`; it is not cleared for compute.

Expected capture scale from published pins:

- TRAIN: 27,416,554 scored rows;
- panel: 1,460,697 scored rows across four roots;
- no-memory cache: about 29 GiB per seed before the seal;
- proposed run: two sequential RTX 5090 captures, estimated 2-4 hours and at most $5.

## Checks run at the head

- `cargo check -p uor-r4-float-train --bin train-in-space --bin
  fbs-query-cache --locked --offline`: passed.
- `cargo test -p uor-r4-float-train --lib --no-fail-fast --locked --offline`:
  **293 passed, 0 failed**.
- `uor-harness prep-check`: the sheet is structurally complete and `PENDING`.
- `git diff --check`: passed.
- Scoped Clippy and the full harness are run before delivery.

Not run: cache capture, centroid evaluation, panel scoring or any GPU work.

## Skills used

No system-provided Codex skill applied. The repository's D22 preparation, geometry,
task-report and compute rules were used directly.

## Still open

1. Supply a private-store token and Vast/pod credentials on this workstation.
2. Obtain a fresh non-author `TEST FITNESS: FIT` for the sheet.
3. Obtain the D67 pre-run review for the exact job.
4. Capture n3S at seeds 1 and 2, seal and upload both caches.
5. Run the CPU evaluator, pool the four panels and report the registered verdict.

Until those complete, the roadmap's centroid-refit step remains open.

## Plain summary

The readout experiment needs two things: queries from TRAIN to fit new output rows and
queries from the held-out panels to grade them. The merged cache captured only the
grading side. This change captures TRAIN as well and can record only the no-memory path,
which halves the storage. The experiment is now fully specified against the current
checkpoint, but the checkpoint is in a private store and this machine has no access or
GPU credential, so the actual model run has not happened.
