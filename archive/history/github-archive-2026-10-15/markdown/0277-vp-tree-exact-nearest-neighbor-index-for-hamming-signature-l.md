# #277 — VP-tree exact nearest-neighbor index for Hamming signature lookups (replace linear scans)

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-07-29T03:31:55Z
- **Updated:** 2026-08-03T13:26:09Z
- **Closed:** 2026-08-03T13:26:09Z
- **Labels:** enhancement
- **Assignees:** auser, Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/277
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Problem — measured tonight

The #244 C-row evaluation runs at ~0.4s/position (macOS sample profile: all frames in R4G1Runtime::predict_distribution), driven largely by the ROUT Hamming-fallback path scanning nodes linearly per query (engine.rs:568-619). The same linear-scan shape appears anywhere we resolve nearest signatures: class assignment scans all K=256 class signatures per stage (runtime::assign_memberships_plain), and any future NN store would inherit it.

## Proposal

A vantage-point tree over signatures. Properties that fit this repo:

- EXACT nearest neighbor (not approximate) under any metric satisfying the triangle inequality — Hamming over 288-bit signatures qualifies; so would Design P's cyclic distance (#276).
- O(log n) average query; ~87× over brute force at 100k points in the reference implementation's benchmarks.
- Deterministic construction possible (fixed vantage-point selection rule → κ-pinnable index bytes, consistent with store_bytes discipline).
- Runtime ops: tree traversal is compares + table-reads; distance is XOR+popcount. Mul-free contract §4 holds. Construction at compile time is unrestricted.

## Reference, not a dependency

sanity/renegade (Rust) demonstrates the mechanics well (src/vptree.rs: exact search, incremental growth with brute-force tail between rebuilds, metric-agnostic). It is LGPL-3.0 — use as a design/benchmark reference only; implement independently. Its isotonic-regression effect-space metric learning (per-feature weights, no gradients, LOO-validated fallback) is separately interesting for compile-time signature-bit weighting and could ship as runtime table-reads — out of scope here, noted for the record.

## Scope

1. Deterministic VP-tree over class signatures for assign_memberships (K=256 per stage — small; measure whether it beats the SIMD scan before adopting).
2. VP-tree over ROUT node signatures in the graph runtime's Hamming fallback — the measured hot path; primary target.
3. Bench harness comparing scan vs tree at matched exactness on the #244 corpus; adopt only where measured faster.

Stands on its own regardless of which addressing design (#243 R/G, #276 P) wins. Related: #232, #244, #276.

## Comments (2)

### comment 5123548909 — 2026-07-29T21:28:13Z — Casey-allard

Scope update after #280/#288: the **primary target named here rescopes**. Item 2's measured hot path (`R4G1Runtime` ROUT Hamming fallback, engine.rs linear scan) belonged to the certify scaffold, which is now demoted out of the measurement matrix — a VP-tree over it would accelerate a non-functional readout.

What stands, re-ranked:
1. `assign_memberships_plain` class-signature scans (K=256 per stage) — unchanged, still the small-but-everywhere case; measure vs the SIMD scan before adopting.
2. **New primary**: the serving engine's membership/widen paths (`uor-r4-api::engine`, score_runtime scorer) — the surface the new C row measures. WidenOnce probes at WIDENED_TOP_M are the expensive shape when Novel traffic appears.
3. The bench-at-matched-exactness discipline and the #276 cyclic-distance compatibility argument carry over unchanged.

No action needed until the #243 Phase B decision lands (it may change the signature family the tree indexes — thermometer-graded widths change the metric's bit layout).

### comment 5166913818 — 2026-08-03T13:26:08Z — Casey-allard

## Closing: delivered for the surface that matters; remainder overtaken by evidence

Delivered: exact VP-tree over ROUT signatures (#339) with the small-graph skip guard (#348) — the deterministic, κ-pinnable, mul-free construction this issue specified, on the lane where linear scans were measured.

The 2026-07-29 rescope's remaining targets (assign_memberships_plain K=256 scans, serving membership/widen paths) are overtaken: the #330 SIMD dot-scan line addressed the class-scan cost directly (3.82× isolated, #352), and the #364 audit measured the emission machinery at accuracy-break-even against the unigram prior — further NN-index acceleration there buys latency on a component not currently paying for itself. If cover-induction work (#364 follow-on) restores the graph path's accuracy value, a narrow re-scoped issue can pick this up with the then-current signature family (the #276 cyclic-distance compatibility argument recorded here carries over).


---
_Comment/review ids in this file: 5123548909, 5166913818_

