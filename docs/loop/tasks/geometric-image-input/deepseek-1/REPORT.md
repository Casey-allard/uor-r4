# REPORT: geometric-image-input, deepseek-1

Lab: deepseek. Item:
[geometric-image-input](../../../../roadmap/items/geometric-image-input.md)
(#3619). Order: [ORDER.md](ORDER.md). Base:
`ab8fc5b0c51d7b160e0520e738c85475ab662853`.

## What did not work, first

- The first mean-RGB implementation multiplied the raw channel sum by Q12 but
  omitted the 255-level denominator, so a white channel produced 1,044,480
  instead of 4096 and failed closed on conversion.
- The first per-pixel luma helper multiplied a `u16` by 587 and panicked before
  a patch could be returned. The retained tests caught both arithmetic defects.
- Rayon at its first one-task-per-patch granularity was slow: 32 logical workers
  took 230.4 microseconds per decomposition against 119.5 at 16. Grouping work
  into one ordered chunk per worker reduced the final pass to 91.7 and 71.2
  microseconds respectively.
- Thirty-two logical workers remain slower than the 16 physical cores on this
  small workload despite consuming about 25 CPU cores. More concurrency is real
  work, not automatically lower wall time.

## What changed

Added the internal `uor-r4-vision` crate and workspace membership. Its
`decompose_rgba(width, height, rgba, grid)` API:

- accepts at most 256 by 256 RGBA pixels and a non-empty grid of at most 32 by
  32 whose axes do not exceed the source;
- validates the exact RGBA byte count and partitions every pixel into exactly
  one non-empty patch;
- computes rounded Q12 mean RGB, integer Rec.601 luma, `R-luma` and `B-luma`
  chroma axes, and mean signed horizontal/vertical luma edges without float;
- exposes two signed Q12 R4 vectors, centered RGB plus centered luma and chroma
  plus edges, all held in `i16`;
- binds dimensions, grid, patch data and a Blake3 digest of the exact RGBA bytes
  in versioned `R4IMG001` canonical bytes; and
- evaluates independent patches through a bounded ordered Rayon collection, or
  serially with default features disabled.

The implementation includes no matmul, softmax, squashing, decoder, captioner,
model call, persisted image or visual-understanding claim.

## Measurements

Host: AMD Ryzen 9 9950X, 16 physical cores / 32 hardware threads. The synthetic
release fixture contains 256 by 256 RGBA pixels and a 32 by 32 grid (1,024
patches). Each row ran 10,000 decompositions.

| Rayon workers | Microseconds per decomposition | Wall | Average CPU | Canonical digest |
|---:|---:|---:|---:|---|
| 1 | 200.361 | 2.00 s | 100% | `83764218…e6d936` |
| 16 | 71.204 | 0.71 s | 1,116% | `83764218…e6d936` |
| 32 | 91.656 | 0.91 s | 2,569% | `83764218…e6d936` |

Sixteen workers gave a 2.814 times wall reduction against one with exact output.
Peak RSS stayed at or below 2.54 MiB. Raw rows are in
[parallel-workers.txt](evidence/parallel-workers.txt). These are component
preprocessing timings, not model or visual-quality results.

## Checks run at the head

- Default parallel and no-default-features serial `cargo test --locked -p
  uor-r4-vision --all-targets --no-fail-fast`: each passed 1 unit test and 6
  integration tests; both example targets compiled.
- Strict Clippy with `-D warnings` passed for both feature configurations.
- `cargo fmt -p uor-r4-vision --check` and `git diff --check` passed.
- The tests pin hand-computed pixel/patch values, exact canonical bytes,
  digest-change behavior, alpha-only binding, complete partition coverage,
  input immutability and all declared failure classes.
- `uor-harness loop gates --full` passed all 11 gates on the immediate rerun.
  The first attempt hit the known fake-gh `ETXTBSY` flake in `loop-drive`; both
  records are retained.
- `cargo check --locked --workspace --all-targets` is red on this branch and on
  exact `origin/main` at the base: `uor-r4-workbench` requests
  `uor-r4-api/learned-reference`, which activates the intentionally excluded
  `uor-r4-core` historical surface. The base reproduction is retained; this
  branch does not claim that workspace check passed.
- No GPU was present and no GPU result is claimed.

## Skills used

No available Codex skill applied. The harness's L-019 was read because the path
matched `crates/**`; this crate prepares input data and has no model-compute or
host-fallback path.

## Still open

The primitive is not wired into the model. #3619 still requires the bounded
`POST /v1/images/decompose` route, native `uor_r4_image` validation and preflight,
truthful 501/413 behavior, Canvas de/code and downsample, preview/removal and
memory-only browser blobs. Visual understanding and a useful multimodal model
are `NOT_RUN`.

## Plain summary

The deterministic integer image patch layer now exists and uses the CPU well.
It produces the same bytes at every worker count; 16 physical cores were the
best measured setting, while 32 logical threads paid more synchronization cost
than they returned. Native and browser image ingestion remain the next work.
