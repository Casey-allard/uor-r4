# #250 — fix(certify): clamp zero-quantized teacher-floor collisions and count them (#231)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-07-28T03:59:41Z
- **Updated:** 2026-10-09T13:09:11Z
- **Closed:** 2026-07-28T04:10:23Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/250
- **Merged:** 2026-07-28T04:10:23Z
- **Merge commit:** d7ee6830a6ddf5bfff551265adbe740c0937c606
- **Base / Head:** main / UOR-Foundation:issue-231-teacher-floor-clamp
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Closes #231 (legacy-path half; the HF-path ≈-uniform floor anomaly is the tokenizer defect, tracked in #242).

## What

- Extracts `recorded_next_prob()`: the corpus-recorded integer-percent probability for the observed next token; a **zero-quantized-weight collision** (next ∈ top-3 with recorded weight 0 — renormalized p < 0.5% at observation time, but still CDF-sampled as `next`) clamps to half a quantization step (0.005) instead of producing `-ln(0)` = +inf. Same clamp discipline the HF evaluate path already applies (`uor-r4-graph-cli/src/lib.rs:1444`).
- The floor output line now reports the collision count (`zero-quantized-weight collisions: N/ntest`) — the instrumentation this issue requested, aimed at the actual mechanism (quantization, not f32 underflow; both softmaxes are max-stabilized).
- 5 unit tests on the extracted function (recorded-weight hit, zero-weight clamp + finiteness, outside-top-3 fallback, non-collision zero in the table, mixed-position finiteness property).

## Why artifacts are safe

Observation records argmax/top-k **ids** (sort-based, quantization-independent) and integer weights; zero weights are dropped at store build (`if weight > 0`). The inf was purely an evaluation-display defect — the certified 1.5960 run simply had no collision in its corpus draw.

## Gates

All four green locally (macOS arm64, pinned 1.97.1, offline).

🤖 Generated with [Claude Code](https://claude.com/claude-code)

https://claude.ai/code/session_017tu2Y4AGZvcgM4vmoSz5x6


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

