# ORDER: one-model-end-to-end, deepseek-41 (served-panel product parity)

Lab: deepseek. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Base: `59494646c`.

## Goal

Make the reply-panel scorer decode an integer artifact through the same
`serve_step` path as `uor-chat`, including an optional artifact-bound content
readout, so a successful asking artifact is not measured through a different
product path.

This is CPU-only support for the active language-carry run. It does not train,
start a pod, score a sealed result, change a bar or touch claude-339's files.

## Cancel probe

- claude-338 added `--engine integer`, but its decoder calls
  `IntegerStackSession::step_native`.
- `uor-chat` and `StackChat` decode through
  `IntegerStackSession::serve_step`, which applies a content readout's vote.
- The scorer has no content-readout sidecar input, and its turn-memory path is
  named but not pinned against the product loop.
- No open DeepSeek PR or claim touches this unit.

**Result (instrument, at claim):** pass. The gap is present and CPU-only.

## Design

- Separate generation from teacher-forced ranking:
  - generation asks a decoder for the served token only;
  - CPU-port generation continues to use its nearest head;
  - integer generation uses `serve_step`.
- Keep the CPU-specific teacher-forced and paired-first paths unchanged.
- Add `--content-readout PATH` for the integer engine. Bind it to the artifact
  hash and vocabulary with the existing `ContentReadout::read_sidecar`, attach
  it before any session, and record its SHA-256 in the report.
- Refuse `--content-readout` on the CPU engine and keep teacher-forced ranking
  refused on asking artifacts.
- Pin the product path with tests:
  - an attached content readout changes the integer panel decoder's first token;
  - a turn-memory artifact decodes through the same reply-start sequence;
  - a no-content artifact remains equal to the CPU port.

## Files in scope

- `docs/labs/one-model-end-to-end/instrument/src/bin/served-reply-panel.rs`
- `docs/loop/tasks/one-model-end-to-end/deepseek-41/**`
- `docs/research/notebook/float-training.md`

## Done when

1. Integer panel generation calls `serve_step`; teacher-forced ranking remains
   on the CPU port.
2. `--content-readout` is validated, attached and identified in the report;
   the CPU engine refuses it.
3. Focused tests pass, and mutations that restore `step_native`, omit the
   content-readout attachment, or ignore the sidecar hash fail them.
4. Existing claude-322 parity still passes when no content readout is supplied.
5. REPORT, notebook row, report/policy checks, full gates and a fresh exact-head
   review pass; one PR merges.

## Stop and ask

Do not touch `crates/uor-r4-integer`, claude-339's branch or files, any pod,
dataset, panel label or sealed result. A product-path mismatch outside this
instrument is reported as open, not repaired here.
