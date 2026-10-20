# #330 — Design: SIMD shift-add dot scan (contract-legal serving speedup)

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @auser
- **Created:** 2026-08-01T18:15:14Z
- **Updated:** 2026-08-03T13:26:11Z
- **Closed:** 2026-08-03T13:26:11Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/330
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Follow-up to the execution-performance directive from the #318 review conversation: the shift-add dot scan is ~95% of steady-state serving cost (4 stages × 256 classes × 288 dims ≈ 295k term applications per token, op-census measured) and is contract-legally SIMD-able — per-lane variable shifts and lane-wise integer adds are shifts and adds, not multiplies.

Design note to follow. Scope: `dot_score_plain`'s per-stage K-scan (and its TLA7 residual variant, #329) behind the repo's established SIMD pattern (safe scalar normative, unsafe isolated adapter, proptest-witnessed equality, runtime dispatch — `transformerless/simd.rs` precedent for AVX2/NEON Hamming).

Key design points to be written up:

1. **Contract analysis**: AVX2 `vpsllvq`/`vpsravq` and NEON `vshl` are shifts (legal); no SIMD multiply, no `PMADDWD`/`UDOT`/`SDOT`, no FMA (contract §3 stays honored). Sign fixup via compare/blend/subtract. ⚑ NEON right-shift rounding semantics (truncating vs rounding shift instructions) must be pinned and witnessed per platform.
2. **Layout**: transposed D×K table access (per dim, broadcast `work[d]`, variable-shift across 4-8 class lanes, accumulate) vs the shipped K×D row layout — either artifact-era transposed storage or one-time transpose at load (outside the hot path; allocation at startup only).
3. **Empirical Criterion**: ≥3× on a dot-scan microbenchmark and ≥2× end-to-end serving at unchanged outputs (equality witness over the fixture artifact), zero-allocation serving path preserved (allocation census), P-4 scan extended to the new adapter module.

Depends on: #329 (TLA7 residual path) landing, and the serving wall-clock benchmark harness (ns/token regression number) that the same directive called for.


## Comments (2)

### comment 5153618549 — 2026-08-01T21:53:56Z — auser

## Reconciling parallel adapter measurements (from the #318 work thread)

While #334 was in flight, a parallel implementation of this design (same note, same load-time-transpose decision) was built and measured on an arm64 M4 Max host. Recording its numbers here so the issue has both data points; the parallel branch was abandoned unmerged once #334 landed (duplicate work, not a competing verdict).

| Metric (M4 Max, release, min-floor under load) | scalar | NEON | ratio |
|---|---|---|---|
| Isolated per-stage scan | 87,027 ns | 67,902 ns | 1.28× |
| TLA6 assign end-to-end | 354,740 ns/tok | 272,325 ns/tok | 1.30× |
| TLA7 assign end-to-end | 407,136 ns/tok | 272,991 ns/tok | 1.49× |

vs #334's arm64 number: ~1.56× end-to-end on the TLA6 fixture. Same order; #334's inner loop is the better one (the parallel implementation's widen/broadcast overhead was ~32% of its NEON budget — op-count analysis in the work log). Analytic ceiling on this layout/lane-width estimated at ~2× (dim-outer register tiling); ≥3× isolated judged out of reach on i64×2 lanes without a storage-design change.

Two engineering notes of record:

1. **Rounding semantics**: both implementations independently pinned `vshlq_s64` (SSHL, truncating) over the rounding variants — the ⚑ from the design note is resolved consistently. Verified in #334's merged `simd.rs`.
2. **Dispatch-default bug class**: the parallel implementation initially had its env opt-out default the adapter OFF when unset (the census silently exercised scalar). Worth checking any future opt-out flag for the same failure mode — the repo convention is `map(|v| v != "0").unwrap_or(true)`.

The serving wall-clock harness from the parallel work (TLA5/TLA6/TLA7 ns/token as a regression-test idiom) may still be worth adopting alongside #334's cargo bench — its TLA7 residual-path numbers above are the first wall-clock measurement of the residual wiring's cost (<2% of assignment, consistent with the design note's op-count estimate).


### comment 5166914266 — 2026-08-03T13:26:10Z — Casey-allard

## Closing: design executed and measured; residual recorded

Executed across #331 (design note), #333 (scalar baseline), #334 (vectorized shift-add), #336 (one-term specialization), #351 (compact layout), #352 (isolated measurement). Against the issue's Empirical Criterion: **≥3× isolated — met** (3.82×, #352); **≥2× end-to-end — not met** (~1.3–1.56× measured on arm64 across two independent implementations; analytic ceiling ~2× on this layout without a storage-design change).

Closing anyway as a maintainer decision rather than chasing the storage redesign: the #364 audit measured the emission-residual machinery this scan feeds at accuracy-break-even against the unigram prior, so the remaining e2e headroom accelerates a component not currently paying for itself. Residual recorded here (storage-era transposed layout for the full ≥2× e2e) — reopen or re-file if the cover-induction line restores graph-path accuracy value.


---
_Comment/review ids in this file: 5153618549, 5166914266_

