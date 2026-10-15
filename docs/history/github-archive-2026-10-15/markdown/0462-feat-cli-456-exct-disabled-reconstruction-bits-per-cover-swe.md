# #462 — feat(cli): #456 — EXCT-disabled reconstruction bits per cover-sweep point (schema 3)

- **Type:** Pull request
- **State:** closed
- **Author:** @auser
- **Created:** 2026-08-06T13:36:04Z
- **Updated:** 2026-10-09T14:17:15Z
- **Closed:** 2026-08-06T14:39:21Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/462
- **Merged:** 2026-08-06T14:39:21Z
- **Merge commit:** 47f3115b1a4729a877a3c734778447f8edb12eed
- **Base / Head:** main / UOR-Foundation:issue-456-recon-objective
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## What

First slice of #456: the cover sweep (`r4 transformerless cover-sweep`) now reports the EXCT-disabled reconstruction metric — held-out top-1 / bits-per-token with exact-context lookup disabled (`rule1_chain`, TF1 D-4's ρ̂) — on **every grid point**, alongside the existing with-EXCT `gate_c_rule12` agreement block.

- `SWEEP_REPORT_SCHEMA` 2 → 3 (module docs updated)
- Per-point `reconstruction: {positions, top1_agreement, bits_per_token}` from the same `evaluate_gate_c` run (no extra pipeline cost)
- Recommendation block gains `reconstruction_bits_per_token` for the recommended point
- Knee rule **unchanged** — reporting only, per the issue's scope; making reconstruction bits a split/stop criterion is the gated follow-up (item 3)
- Unit test pins the schema-3 serialization

## Why

Gate C's pinned report shows the graph slice at ~1% top-1 while EXCT memory carries the headline number. Until reconstruction error is visible at decision time, cover configuration choices optimize the wrong axis. This makes the rate–distortion frontier see what each byte buys in *reconstructability*.

## Verification

Four local gates green (rustup-pinned toolchain): `cargo test --workspace --offline`, `clippy --all-targets --all-features -D warnings`, `fmt --check`, `no_std` ladder (both feature configs). Full sweep pipeline not re-run (expensive; the metrics come from the same `evaluate_gate_c` call already executed per point).

Refs #456. Follow-ups: #457 (IPF consistency operator), #458 (interaction-information block search), #459 (estimation ladder).


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

