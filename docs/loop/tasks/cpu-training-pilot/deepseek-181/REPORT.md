# REPORT: cpu-training-pilot, deepseek-181

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Base: `e94afaae8`; implementation pin:
`1ba1620a0`.

## Status

Complete as a read-only, fail-closed audit for anchored writer artifacts.
No writer-admission or model-quality result is claimed.

## What did not work

No artifact from Claude's enabled `train-in-space --writer-anchor 1` path has
been published to this worktree yet. The positive control is therefore a
synthetic export, not evidence that a trained writer run preserved its ports.

The two public dual-c131 artifacts predate writer anchoring and correctly fail
the new gate:

- seed 1: **65,505 / 65,536** port coordinates mismatch across all **4,096**
  rows;
- seed 2: **65,526 / 65,536** port coordinates mismatch across all **4,096**
  rows.

Those are expected negative controls, not model results.

## What changed

- Added `CpuFbsForward::audit_writer_port_anchors`, using the existing exact
  `UORLUT01` parser and parsed SpiralCore roots.
- For token `t`, the audit derives cell `t mod 256`, reads roots `cell / 16`
  and `cell mod 16`, and checks Q12 columns `0..8` and `W-8..W`.
- The report binds the artifact SHA-256 and records width, vocabulary, checked
  tokens and coordinates, distinct cells, mismatch coordinates, affected
  tokens and rows, and up to 32 first mismatches.
- Added `cpu-writer-anchor-audit --artifact PATH
  [--expected-artifact-sha256 HEX] [--output PATH]`. Hash drift, malformed
  artifacts, short widths and port mismatches fail closed. An output path that
  resolves to the input artifact is refused.
- Added synthetic positive, moved-coordinate, wrong-cell, malformed,
  short-width, hash, output-alias and Q12 boundary tests.

No training, serving, CUDA, float-inference, exporter or Claude-owned source
changed.

## Measurements

- reviewed delivery head:
  `1052f92555ce0ce3402d0cc96a9b4c22eca7f1c0`;
- PR: #3732;
- protected merge: `5bd24915e92f434d902c55b0db46063f5ab40851`;
- Synthetic positive: **256** tokens, width **32**, **4,096** checked port
  coordinates over **256** distinct cells, zero mismatches.
- One moved coordinate: **1** mismatch, **1** token, **1** row.
- One token assigned another cell's roots: only the changed second port
  mismatches; token and row counts remain **1**.
- Q12 boundary: float `-2, 0, 2` exports and reads back as
  `-8192, 0, 8192`.
- Public negative controls are recorded in
  [evidence/public-fbs-dual-negative-controls.json](evidence/public-fbs-dual-negative-controls.json)
  with their sealed source hashes.
- CPU package suite: **340 passed**, 4 expected ignores, and **101** binary
  tests passed.
- Focused writer-port tests: **5 passed**; CLI tests: **4 passed**.

## Checks run at the head

- `cargo fmt --package uor-r4-cpu-train`;
- focused `writer_port` tests;
- the CLI hash parsing test;
- full `cargo test --locked -p uor-r4-cpu-train`;
- strict Clippy with `--all-targets --no-deps -- -D warnings`;
- direct audit runs on both public unanchored artifacts;
- artifact SHA-256 agreement with the public `SHA256SUMS`;
- fresh-context review, with the output-alias and hash-branch findings fixed;
- exact-head review, all required CI checks and protected merge;
- fresh-main CPU package: **340 passed**, 4 expected ignores;
- fresh-main binary tests: **101 passed**; strict Clippy passed;
- `git diff --check`.

## Size

The implementation adds one audit API, one binary and focused tests in the
existing CPU training crate. No corpus or model artifact is committed; the
compact negative-control audit evidence is.

## Skills used

No system-provided Codex skill applied. Repository ORDER, claim, report and
delivery procedures were used.

## Still open

Run the audit against Claude's first exported anchored training artifact and
bind its hash/result to that run's evidence. A zero-mismatch audit establishes
export/read-back preservation only; it does not establish improved writer
admission or language quality.

## Plain summary

The new tool checks the two 8-coordinate port blocks of every token after
round-trip export. It accepts a synthetic anchored table, reports precise
coordinate and row mismatches, rejects malformed or hash-drifted inputs, and
rejects both known unanchored public artifacts as expected. The next useful
positive is an actual anchored artifact from Claude's training path.
