# REPORT: repair CPU M2 evidence semantics, deepseek-2

Lab: deepseek. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md). Base: `440aeae14`. Follow-up to [deepseek-1](../deepseek-1/REPORT.md).

## What did not work, first

- The merged first pass included each document's BOS in the first prompt run. That disagreed with the canonical census and first-eight-token turn-memory key.
- The UORT embedded vocabulary field was not compared with the manifest or artifact vocabulary.
- Full independent parity was not exercised for the four-table family that the CPU query path does support.
- The evidence-only graph and panel-writer graph shared all dependencies, making the new public smoke command harder to use independently.

## What changed

- Every document's BOS is excluded before prompt/reply run construction.
- UORT vocabulary, manifest vocabulary and artifact vocabulary must agree.
- The tokenizer is parsed through the supported byte-level BPE reader, and corpora with no prompt or response runs are refused.
- `cpu_engine_parity` checks query coordinates, admitted cells, top-8 and top-1 with `CpuDiscreteProposalScorer` for supported four-table artifacts. The 19-table M1 artifact retains an independent writer check and reports full CPU-trunk parity as unavailable.
- `cpu-parity` and `panel-writer` are separate features, so each evidence graph can be selected explicitly.
- The public M1 panel/control smoke and a separate four-table parity smoke were regenerated after the semantic fixes.

## Measurements

Public 19-table M1 seed-1 smoke:

- 616 response targets.
- Admission `365`, top-1 `293`, top-8 `306`.
- Openings: `7/58` top-1; continuations: `286/558` top-1.
- All `58` prompts and first-eight-token keys were absent from the generated control.
- All `58` value-masked prompt templates were absent at K=256 and K=1024.
- `19/58` reference replies were verbatim in the generated control.
- Independent writer parity: `616/616` rows with zero admitted-cell, top-8 or token mismatches.
- Full independent trunk parity is unavailable for this 19-table family.
- Repeated reports were byte-identical at SHA-256
  `2fc34ee636a24481cda96fdc85dfa627c81032f4297c0ae70766c303a4ab7a2b`.

Public four-table seed-1 parity smoke:

- Artifact SHA-256 `fc31bcc7a88250f3758d07a8b3d12480a5332acb5437b0c0f0e5c71ae88939ca`.
- 128 development targets.
- Independent CPU-engine parity checked 4,096 Q12 query coordinates.
- Query, admitted-cell, top-8 and token mismatches were all zero.
- Report SHA-256 `e3e4e3b868dffc4953b018cc7afcbc643622e4493a3f8f835a4c507a9bbdc5ea`.

These are instrument readings against public data, not M2 capability results. The reference index is a generated control rather than canonical TRAIN, and no model-generated replies were supplied.

## Checks run at the head

- `cargo test --offline --features cpu-parity --lib --bin m2-evidence -j1`: 13 evidence tests and 3 CLI tests passed.
- `cargo test --offline --features panel-writer --bin heldout-panel -j1`: the panel-writer test passed after the repository's known duplicate Cargo artifact race was cleared.
- `cargo clippy --offline --features cpu-parity --all-targets -- -D warnings`: passed.
- `cargo clippy --offline --features panel-writer --bin heldout-panel -- -D warnings`: passed after the known duplicate artifact race was cleared.
- `cargo fmt -- --check`: passed.
- `git diff --check`: passed.
- Two 19-table smoke executions produced the same report SHA-256.

## Skills used

No repository skill was invoked. Independent subagents reviewed the evidence semantics and the
parity boundary.

## Still open

- The 19-table M1 trunk does not have a full independent CPU query implementation; only the writer boundary is checked.
- Generated complete replies, generated novelty, canonical TRAIN overlap and count-table/memory-off/no-read arms remain unavailable without those inputs.
- This instrument does not qualify M2.

## Plain summary

The CPU evidence report now follows the same BOS and value-mask conventions as the project census,
binds every relevant corpus identity, and distinguishes new prompt templates from known reply
templates. It independently reproduces the full CPU discrete path for a four-table artifact and
clearly limits the current 19-table M1 report to writer parity.
