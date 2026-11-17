# REPORT: cpu-training-pilot, deepseek-254

Lab: deepseek. Order: [ORDER.md](ORDER.md).

## What did not work, first

- At the initial `2026-11-09T06:07Z` probe, the store listing for
  `results/claude/one-model-end-to-end-carry-hold-2026-11-09/runs/` returned
  HTTP 404. That evidence remains a time-bounded pre-publication observation.
  Claude-334 later sealed both seeds, and
  [deepseek-259](../deepseek-259/ORDER.md) audited the real checkpoints.
- A Clippy run with `-D warnings` failed before reaching this binary because
  `uor-r4-integer` and `uor-r4-discrete` already carry warnings under the
  current toolchain. The scoped `--no-deps` run passed with only pre-existing
  crate warnings and no warning in either changed file.
- The first fresh exact-head review rejected the manifest containment. The
  validated canonical path was discarded, workers resolved it again, and child
  files were not root-bound. [deepseek-255](../deepseek-255/ORDER.md) repairs
  this before merge.
- The grouped readings validate checkpoint contents. They cannot show that the
  held write reaches `H4`, preserve B2, or serve in integers.

## What changed

- `AskHoldSummary` now reports gain mean/min/max for the 16 question tokens,
  four header tokens and 512 value tokens, plus the mean gain over every other
  token.
- It reports each present ask depth's mean absolute `w_y` row. The executable
  requires that set to match the checkpoint's `[2,6]` ask depths.
- `cpu-carry-hold-audit` schema `/2` keeps the single-checkpoint interface and
  adds `--manifest MANIFEST --workers N --out REPORT`. Batch mode binds every
  model and Hamming hash, expected hold row count and contained relative path.
  The repair retains the validated root and canonical checkpoint paths, rejects
  canonical path aliases, opens each child file as a regular in-root descriptor,
  and checks the opened Linux file descriptor before reading it. It rejects
  duplicate id, label/seed, canonical path and hash-pair identities; audits
  with a bounded 1-32 worker pool; preserves manifest order; and writes only a
  new report.
- Tests cover grouped statistics including the rest-token mean, per-depth
  weights, a missing depth-6 weight, every duplicate identity class, canonical
  path aliases, path traversal, an out-of-root child symlink, mixed
  single/batch arguments, wrong schema, wrong hashes, wrong row count, worker
  bounds, source-order preservation, output replacement, byte-identical
  repeats and a bounded FIFO refusal.
- No Claude-owned float-training file, serving schema, export path, GPU state
  or artifact was changed.
- Size at the deepseek-254 head: `pr-check` reported tests 0 files, code 2
  files `+937/-109`, docs 10 files `+610/-0`, generated 0 files and other 4
  files `+1864/-0`.

## Measurements

The synthetic checkpoint has vocabulary 1,100, width 8, one ask head at each
of ask depths 2 and 6, `hold_rows=8`, and ten nonzero gains. The single audit
measured:

- question gain: mean `0`, min `-1`, max `1`;
- header gain: mean `0.125`, min `-0.25`, max `0.5`;
- value gain: mean `0.01171875`, min `0`, max `4`;
- depth-2 `|w_y|`: `0.1875`; depth-6 `|w_y|`: `0.0625`;
- 28/32 target-nearest ask cells with two regressions.

The two-arm manifest repeated byte-for-byte at two workers and preserved the
source order `aqh-s2`, then `aqh-s1`.

The real audit then downloaded both Claude-334 seed tarballs, verified their
recorded SHA-256 values, extracted them under the isolated worktree and audited
all eight held checkpoints:

- `B2-aqH` and `B2-aH`, seeds 1 and 2, at steps 4,096 and 16,384;
- 8 workers in `0.03 s` wall, `520%` CPU and `171,240 KiB` peak RSS;
- every question, header, value and rest gain and both per-depth type-weight
  means matched Claude-334's `recall.jsonl` exactly on all eight checkpoints.

The real batch is an independent CPU reading of the recorded diagnostics. It
does not replace Claude-334's B2 judgment or turn a moved gain into carry.

Evidence:

- [single report](evidence/synthetic-single.json), SHA-256
  `1d2014a91240db057728ce27e6554dffef1774bc2a8a6322237d2e060ed09f42`;
- [batch report](evidence/synthetic-batch.json), SHA-256
  `95b39b12dd3684abe23c67d32f51b86dd2f5ffe83abf0191cf22d256726a4673`;
- [real manifest](evidence/real-eight-checkpoint-manifest.json), SHA-256
  `e1287b9b7f31766ab0545654fa6c4c4fb8b303f41ffcceadceb7c5d4e0b78c24`;
- [real eight-checkpoint report](evidence/real-eight-checkpoint.json), SHA-256
  `497140c6a6730521b3337c8f155cd31771aade953203eb1ee4eab7d51ee58865`;
- [real gain parity](evidence/real-eight-gain-parity.json), SHA-256
  `be065856b20a7cc56a0a71b1dacd02d6a188927f00bb303521fdc3b4a1cce6d1`;
- Claude-334 seed tarballs: seed 1
  `57cbd25a941f56458436caaa258b0afbe8a79a5520d5f31a371e7b33cf795d93`,
  seed 2
  `4d952047d69e4f80bcd1385d95c0189d268c1949d82503941c70e8395d2314fd`;
- retained synthetic reports are test-harness output, not release-CLI output.
  Their
  embedded executable SHA-256 is
  `87d481a87f003b19aa675a04d720ae058ca32bf08650b1b3e9080055d12cb665`.

The full CPU library suite passed 480 tests with 5 ignored in 47.22 s. The
release build completed after the repair; its executable SHA-256 is
`aef6083c6c47180d743abf3521716a12841316cecdb35b94d6768f8af171a31e`.

Resources are in the
[D84 ledger](../../../../labs/cpu-training-pilot-carry-hold-grouped-2026-11-09/resource-ledger.json).
The previous cumulative projection was `111,167.677487699 s`; this unit adds a
3,600-second projection for code, tests, delivery, review and repair, for a new
cumulative `114,767.677487699 s`. GPU cost is zero.

## Checks run at the head

- Scoped `rustfmt --check` and `git diff --check`: passed.
- Focused binary tests: 9 passed.
- Full `uor-r4-cpu-train` library suite: 480 passed, 5 ignored.
- `cargo check -p uor-r4-cpu-train --all-targets`: passed.
- Scoped Clippy with `--no-deps`: passed with 10 pre-existing crate warnings
  and none from the changed files.
- Release build: passed.
- `loop report --check` and `loop pr-check --local`: passed.
- All 11 `loop gates --full` gates: green.

## Skills used

No system-provided Codex skill applied. The repository's loop, CPU-research and
software-integrity instructions were followed directly.

## Still open

- Every currently sealed B2 held checkpoint has been audited. A later
  claude-335 checkpoint needs a new manifest with its model and Hamming hashes;
  use the same isolated command.
- Interpret moved gain with failed carry as a write or trunk-carry problem.
  Interpret moved gain and passing carry as a deeper attribution question.
  None is a B2 verdict from this CPU instrument.
- Do not add hold export or serving from this instrument.

## Plain summary

The CPU audit reads the same hold-gain buckets as Claude's training log and
reproduced all eight sealed B2 held checkpoints exactly in one strict pass. It
supports the diagnostic record; Claude-334 still owns the B2 result.
