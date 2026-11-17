# REPORT: cpu-training-pilot, deepseek-257

Lab: deepseek. Order: [ORDER.md](ORDER.md). Parent:
[deepseek-256](../deepseek-256/REPORT.md).

## What did not work, first

- The third review found that Linux opened checkpoint paths before checking
  whether they were regular files. A FIFO under the manifest root could block
  a worker indefinitely.
- The review also found that two raw `|w_y|` spans split the notebook row into
  ten cells; the harness could no longer read the intended Number, Bar and
  Verdict columns.
- The fourth exact-head review passed the code, controls and bounded claims,
  then found that the final size commit added one byte to this report without
  refreshing the resource ledger. Its exact receipt was stale by one byte.
  [deepseek-258](../deepseek-258/ORDER.md) repairs it.

## What changed

- Linux batch reads now open the canonical child path with `O_NONBLOCK |
  O_NOFOLLOW`, validate the pinned descriptor as a regular file, verify its
  `/proc/self/fd` target remains beneath the root, and only then read it.
- Non-Linux batch mode fails before any child-file open.
- A focused control creates a directory at `model.safetensors` and requires the
  non-regular-file refusal.
- Both notebook occurrences use `\|w_y\|`. The row now parses as six cells.
- The synthetic checkpoint wording now says one ask head at each depth.

## Measurements

- Focused binary tests: 9 passed.
- Full CPU library suite: 480 passed, 5 ignored in 47.22 s.
- Full-suite process: 47.30 s, 2,112% CPU, 707,820 KiB peak RSS.
- Single evidence SHA-256:
  `1d2014a91240db057728ce27e6554dffef1774bc2a8a6322237d2e060ed09f42`.
- Batch evidence SHA-256:
  `95b39b12dd3684abe23c67d32f51b86dd2f5ffe83abf0191cf22d256726a4673`.
- Test executable SHA-256:
  `87d481a87f003b19aa675a04d720ae058ca32bf08650b1b3e9080055d12cb665`.
- Release executable SHA-256:
  `aef6083c6c47180d743abf3521716a12841316cecdb35b94d6768f8af171a31e`.

## Checks run at the head

- Scoped `rustfmt --check` and `git diff --check`: passed.
- Focused binary tests: 9 passed.
- Full `uor-r4-cpu-train` library suite: 480 passed, 5 ignored.
- `cargo check -p uor-r4-cpu-train --all-targets`: passed.
- Scoped Clippy with `--no-deps`: passed with only pre-existing warnings.
- Release build: passed.
- Report policy, local policy and all 11 gates passed before the fourth review.
  That review requested changes only for the one-byte stale resource receipt;
  deepseek-258 repairs it.

## Skills used

No system-provided Codex skill applied.

## Still open

The original store probe was `NOT_RUN` at 06:07Z. After Claude-334 published,
deepseek-259 audited all eight real held checkpoints and reproduced every
recorded group gain and type-weight mean. This unit still adds no B2, carry,
export, serving or capability evidence.

## Plain summary

Batch validation now refuses FIFOs and other non-regular files without a
blocking open, and the notebook row is structurally valid again.
