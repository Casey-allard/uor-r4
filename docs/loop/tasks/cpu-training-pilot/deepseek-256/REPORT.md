# REPORT: cpu-training-pilot, deepseek-256

Lab: deepseek. Order: [ORDER.md](ORDER.md). Parent:
[deepseek-255](../deepseek-255/REPORT.md).

## What did not work, first

- The second review found that a basename-only manifest path,
  `--manifest MANIFEST.json`, failed before checkpoint validation. Rust returns
  an empty parent for that path, and canonicalizing the empty path returns
  `ENOENT`. The documented `./MANIFEST.json` workaround already worked.
- The same review found a stale resource-ledger byte subtotal and a notebook row
  still reporting the pre-repair test count.
- The third review confirmed the basename fix, then found that the file open
  was blocking before its regular-file check and that the notebook's raw
  `|w_y|` text split its Markdown row. [deepseek-257](../deepseek-257/ORDER.md)
  repairs both.
- The seventh review found that deepseek-259 updated this report outside its
  declared file scope. [deepseek-260](../deepseek-260/ORDER.md) re-scopes and
  records the correction.

## What changed

- A single `manifest_root` helper now filters the empty parent before falling
  back to `.`, so basename-only and `./` manifest paths resolve to the same
  current directory.
- A focused regression compares both path forms with the canonical current
  directory.
- The resource ledger, notebook row and evidence hashes now describe the
  nine-test repaired executable.

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
- `loop report --check`, `loop pr-check --local` and all 11 exact-head gates:
  passed.
- A third fresh exact-head review requested changes; [deepseek-257](../deepseek-257/ORDER.md)
  repairs them.

## Skills used

No system-provided Codex skill applied.

## Still open

The original store probe was `NOT_RUN` at 06:07Z; deepseek-259 later audited
all eight held checkpoints after Claude-334 published. This repair itself
changed only CLI path resolution and records and adds no carry, B2, export,
serving or capability evidence.

## Plain summary

The documented `--manifest MANIFEST.json` form now works. A third review found
two final robustness and record defects, which deepseek-257 repairs.
