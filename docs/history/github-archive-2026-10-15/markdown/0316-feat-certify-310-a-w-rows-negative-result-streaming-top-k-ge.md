# #316 — feat(certify): #310 A-W rows (negative result) + streaming top-k gen speedup

- **Type:** Pull request
- **State:** closed
- **Author:** @auser
- **Created:** 2026-07-31T23:17:22Z
- **Updated:** 2026-10-09T14:17:15Z
- **Closed:** 2026-08-01T00:46:32Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/316
- **Merged:** 2026-08-01T00:46:32Z
- **Merge commit:** 700c307963501cc04c3fbdecbe295c6cee05b195
- **Base / Head:** main / UOR-Foundation:issue-310-aw-rows
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Refs #310. Companion to #311 (design note).

Two commits, one measurement outcome:

1. **`feat(certify)`: A-W(2)/A-W(4) rows** — margin-reliability weighted Hamming assignment rows in the #243 Phase A matrix, per `docs/learned_signature_weighting_design.md`. Measured on a fresh-era rows-only run: **A-W(2) 28.0/31.6, A-W(4) 28.3/32.0** vs sign-space baseline ~28 and A-f32 32.1/36.4 — ~5% of the gap against the note's ≥25% criterion. **Recorded negative result**: sign-space assignment is saturated; rows retained as the record. Full matrix in the #310 comment.

2. **`perf(compiler)`: streaming top-k in `softmax_top3_sample`/`softmax_top8_sample`** — replaces the per-token 32k-entry Vec + full sort with O(vocab·N) streaming selection (strict-`>` preserves stable-sort tie order).
   - Byte-identical corpus confirmed by old/new binary differential (recs + hidden prefixes; story-chunked determinism).
   - Equivalence witnessed by new `tests/softmax_topk.rs` (stable-sort reference, exact-tie cases).
   - **+20–25% teacher-corpus gen throughput**, interleaved A/B under load.
   - Also records the **rayon row-parallel matmul negative result** in `model-source::matmul` (bit-identical in principle, ~10× slower in practice via fork-join scheduling latency on loaded machines; reverted).

No runtime-kernel changes; no format changes; no κ impact.

Local gates: `cargo test --workspace --offline` ✓ · `cargo clippy --workspace --all-targets --all-features --offline -D warnings` ✓ · `cargo fmt --check` ✓ · no_std ladder (graph-format, default-off / +alloc) ✓


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

