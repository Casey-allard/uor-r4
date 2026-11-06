# REPORT: one-model-end-to-end, deepseek-16 (current-checkpoint centroid)

Lab: deepseek. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md)
(#3407). Order: [ORDER.md](ORDER.md). Sheet:
[one-model-end-to-end-centroid-2026-11-06](../../../../harness/prep/one-model-end-to-end-centroid-2026-11-06.md).
Base: `a9a0f75ac`.

## What did not work, first

- **The merged cache could not fit a centroid.** Its first capture path recorded DEV,
  Fresh V2 and requested panels, but not TRAIN. The evaluator's `--train train` would
  therefore fail after expensive compute. This unit adds a cache-only TRAIN capture.
- **The merged cache wrote both memory modes.** On D3 that is 27,416,554 TRAIN queries,
  about 29 GiB per mode. This unit adds `--query-cache-memory 0|1`; the registered run
  requests no-memory only.
- **The first capture inventory could not reach the registered evaluator.** `score_rows`
  always cached DEV as well as TRAIN and the requested panels, but the evaluator requires
  exactly TRAIN plus its four registered panels. This unit adds explicit DEV omission;
  ordinary DEV scoring is unchanged.
- **No capture was available when the execution package was prepared.** The private
  store and Vast credentials are now installed, but both existing pods are busy and
  must not be touched. The sheet remains PENDING, not FIT, and no panel was opened.

## What changed

- `score_rows` accepts the configured `--train-split` and captures it under the cache
  split name `train` when a query cache is requested.
- The TRAIN cache uses the same Q12/served-forward choice, memory state, seeds and
  `row_picks` decoder as evaluation splits. It is not added to normal scored outputs.
- `--query-cache-memory 0|1` selects one memory mode for caching and is refused unless
  `--query-cache-out` is present.
- `--query-cache-dev 0|1` defaults to `1` for compatibility. `0` omits only the DEV cache
  split while leaving DEV in the ordinary scored outputs.
- Focused tests check both memory modes and the DEV omission rule.
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
- proposed run: one RTX 5090 capture per seed, estimated 2-4 hours each, with the
  package's hard wall and price caps.

## Checks run at the head

- `cargo check -p uor-r4-float-train --bin train-in-space --bin
  fbs-query-cache --locked --offline`: passed.
- `CARGO_TARGET_DIR=target cargo test -p uor-r4-float-train --lib
  --no-fail-fast --locked --offline`:
  **294 passed, 0 failed**.
- `cargo clippy -p uor-r4-float-train --all-targets --no-deps --locked
  --offline`: passed with the crate's pre-existing warnings.
- D23, D25 and D32 ratchets: passed.
- `uor-harness prep-check`: the sheet is structurally complete and `PENDING`.
- `git diff --check`: passed.
- Scoped Clippy and the full harness are run before delivery.

Not run: cache capture, centroid evaluation, panel scoring or any GPU work.

## Skills used

No system-provided Codex skill applied. The repository's D22 preparation, geometry,
task-report and compute rules were used directly.

## Still open

1. Obtain a fresh non-author `TEST FITNESS: FIT` for the sheet.
2. Obtain the D67 pre-run review for the exact job.
3. Lease a Codex-owned pod without touching either busy existing pod.
4. Capture n3S at seeds 1 and 2, seal and upload both caches.
5. Run the CPU evaluator, pool the four panels and report the registered verdict.

Until those complete, the roadmap's centroid-refit step remains open.

## Plain summary

The readout experiment needs two things: queries from TRAIN to fit new output rows and
queries from the held-out panels to grade them. The merged cache captured only the
grading side. This change captures TRAIN as well and can record only the no-memory path,
which halves the storage. The experiment is now fully specified against the current
checkpoint and can produce the exact registered split inventory. Credentials are now
available, but the sheet still needs fitness and pre-run review and both existing GPUs
remain busy, so the actual model run has not happened.
