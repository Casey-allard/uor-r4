# REPORT: one-model-end-to-end, deepseek-262

Lab: deepseek. Order: [ORDER.md](ORDER.md).

## What did not work, first

- A workspace-wide Rust formatter followed module declarations and rewrote
  unrelated stack files and pre-existing formatting. Every formatter-only
  change was reverted by exact path before the small implementation hunks were
  reapplied. The final diff touches only four source files.
- Strict package-wide Clippy is blocked by pre-existing warnings across the
  integer crate, including `format.rs`. Clippy reports no warning in this
  unit's new code after filtering the changed files; the remaining
  `format.rs:250` warning predates this work.
- The first full gate run hit the harness's known transient `Text file busy`
  race in its fake-`gh` test. The clean rerun passed all 11 gates.
- The first GitHub `ci` run failed before creating any job or log. Reopening
  the PR at the same reviewed head produced a clean run with ten successful
  checks, but the merge queue retained the stale pre-job suite and reported
  all five required checks as expected. This receipt-only commit gives the
  queue a fresh head and check suite; product code is byte-identical.
- The sealed Claude-335 t1 archive downloaded and verified at SHA-256
  `aa6fb485…`, but it contains training checkpoints and no exported
  `UORLUT01`. Producing an artifact from that checkpoint needs the CUDA
  snapshot path, so real-artifact ablation is `NOT_RUN` on this CPU-only host.
- The synthetic carry artifact proves artifact transformation and query
  semantics only. It does not show a language result or qualify `−C` on a
  trained model.

## What changed

- Added `drop_hamming_carry_uorlut01` to the stack format API.
- The function refuses an artifact without a complete Hamming read or without
  the readout carry.
- It removes `hamming.d{depth}.carry` and `carry_rows`, then rebuilds the data
  block from every remaining matrix and table in original order with fresh
  aligned offsets.
- It reparses the output and checks that the Hamming inventory and ask flags are
  unchanged, every ask table is byte-identical, and no carry remains.
- Added `fbs-toks ARTIFACT --drop-hamming-carry --out FILE`. The command writes
  only to a new path and reports input/output hashes, removed gains, removed
  gain bytes, carry reach and ask depths.
- Tests cover exact non-carry section preservation, ask preservation, later-row
  readout-query change, unchanged stream queries, no-carry refusal, no-read
  refusal and output-exists refusal.
- No artifact schema, serving arithmetic, model weight or training objective
  changed.
- Size before this REPORT: 4 code/test files, `+403/-4`. The ORDER gave no line
  estimate.

## Measurements

The synthetic carried artifact was **58,176 bytes** with SHA-256
`fdc706c0…`; the ablated artifact was **57,088 bytes** with SHA-256
`0e1484e4…`. The transform removed **256 gains / 1,024 gain bytes** while
retaining ask depths 1 and 2.

The byte-level test compares every matrix nibble/scales section and every table
except the carry table, and finds them identical. The Hamming trace test leaves
every stream ask query unchanged and finds at least one changed readout query
after row 0.

The sealed store archive was read only for the real-artifact probe:

- path:
  `results/claude/one-model-end-to-end-question-carry-2026-11-09/tests/t1/`;
- archive SHA-256: `aa6fb48538a4d3037785fce85e6b908a0eacc2f1f99f4682a83ab6b025eaa309`;
- exported `UORLUT01` files: 0.

## Checks run at the head

- Focused carry-ablation library tests: 2 passed.
- `cargo test --offline --locked -p uor-r4-integer --features
  serving-wasm,fbs-synthetic --bin fbs-toks`: 18 passed, including the
  write-once CLI control.
- Full integer library suite with `serving-wasm,fbs-synthetic`: 501 passed,
  1 environment-gated test ignored, 0 failed.
- `cargo check --offline --locked -p uor-r4-integer --features
  serving-wasm,fbs-synthetic --all-targets`: passed.
- Clippy on the library and `fbs-toks`: no warning in the new code; strict
  package-wide `-D warnings` remains blocked by pre-existing lints elsewhere.
- `git diff --check`: passed.
- `uor-harness loop pr-check --local`: passed; only the existing claim-scope,
  FILEMAP and pre-existing harness advisories remain.
- `uor-harness loop gates --full`: all 11 gates green in 7.97 s on the clean
  rerun.
- `uor-harness check`: 360 rules, 0 errors and 326 pre-existing warnings in
  96.28 s.
- GitHub Actions on the fresh check suite: 10 successful, 15 skipped, 0
  pending or failing, including all five required checks and the five-minute
  repository-wide harness job.

## Skills used

No system-provided skill was used. The repository's loop, CPU-research and
artifact-format instructions were followed directly.

## Still open

- Run `fbs-toks --drop-hamming-carry` on an exported Claude-339 language-carry
  artifact and score the carried and ablated artifacts on the same panel.
- After Claude-340 merges, enable the CPU rank modes in `served-reply-panel`;
  its current explicit refusal predates the merged CPU ask/carry parity.
- Treat ablation output as an evaluation control, not a replacement serving
  model.

## Plain summary

Claude needed a way to compare an asking model with and without its short
question carry. This unit adds that exact control: it removes the carry table
and its metadata from a model artifact while keeping every question table and
all other model data unchanged. It then proves the output reads back as a
valid asking model with no carry. The transformation is tested on a synthetic
model; a real exported Claude model is still needed for the model-level check.
