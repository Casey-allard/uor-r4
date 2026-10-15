# #658 — feat(#645): shared #603 trace-row reader beside the writer

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-13T03:15:05Z
- **Updated:** 2026-10-09T14:17:07Z
- **Closed:** 2026-08-13T05:37:31Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/658
- **Merged:** 2026-08-13T05:37:31Z
- **Merge commit:** 755789c0c34e7091270e3c947c8afb471c81c4a7
- **Base / Head:** main / UOR-Foundation:i645-trace-reader
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Closes #645.

## Summary

#603 documented a trace-row lane layout, but the row-width math was copied in three places and the row *decoder* existed only inside #605's `route_fit.rs` — a drift surface: a writer-side layout change would be caught only by whichever consumer's tests happened to notice. This adds one shared reader beside the writer.

## What's included

- **`TraceRowLayout`** (a pure function of `TraceProfile` + the oracle's capture geometry: lane counts, kv width, `row_bytes`) and **`TraceRow` + `read_row`** in `observation.rs`, beside the writer-side `TraceCapture` assembler. `read_row` is marker-aware (absent support slots dropped, never zero-filled) and length-checked.
- **`TraceCapture::new`** now derives `row_bytes` from `TraceRowLayout` (was an inline copy of the width formula).
- **`route_fit::load_route_trace_corpus`** replaces its inline `row_bytes` math and manual per-record offset decode with `TraceRowLayout::new` + `read_row`; the now-dead `read_f32`, the `SUPPORT_ABSENT_MARKER` import, and the `support` local are removed.

## Verification

- **Round-trip test**: a row assembled exactly as the writer does (front-loaded support entries, trailing absence markers, GQA kv width) decodes back to the same structured lanes; wrong-length rows are rejected.
- **Behaviour parity**: the existing observe + `route_fit` end-to-end tests (real capture → decode round-trips) are the regression gate and pass unchanged — **95 lib + 49 integration, 0 failed**.
- Workspace builds; graph-compiler `clippy` clean; register gates R1/R2/R3/R4/R6 pass.

## Breaking changes

None. `TraceRowLayout`/`TraceRow` are additive public API; the byte format and all behaviour are unchanged.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

