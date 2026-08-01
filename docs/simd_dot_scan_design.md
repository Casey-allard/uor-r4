# Design: SIMD Shift-Add Dot Scan (Contract-Legal Serving Speedup)

*Issue #330. Status: DESIGN FOR REVIEW — no code authorized by this
document. Decision points marked ⚑ are open.
Claim language: Definitions, Objectives, and Empirical Criteria only.*

## Objective

Reduce steady-state serving latency by vectorizing the shift-add dot
scan — measured at ~95% of per-token op count — **without** weakening
the runtime contract (`docs/transformerless/INFERENCE_OPERATION_CONTRACT.md`):
no multiply, divide, or float in the deployed kernel; allocation-free
steady state; deterministic outputs identical to the scalar path.

## The hotspot, measured

Per token, assignment runs 4 stages × 256 classes × 288 dims ≈ 295,000
`dot_term_apply` evaluations (decode packed term → shift → conditional
negate → add). The op census on a certification run reports ~318k adds,
~318k shifts, ~313k table reads per token. After #329 (TLA7 residual
wiring) the scan is unchanged in size; the residual update adds <2%.

## Contract analysis (the load-bearing claim)

Contract §3 forbids SIMD multiply, `PMADDWD`/`UDOT`/`SDOT`, and FMA.
The candidate instruction set uses none of them:

- **Per-lane variable shifts** — AVX2 `vpsllvq`/`vpsravq`, NEON
  `vshl`/`vsshl` — are *shifts*, the same operation class as scalar
  `shl`/`shr` in §2. ⚑ NEON right-shift rounding: the truncating vs
  rounding shift instructions differ across platforms; the scalar path
  uses truncating arithmetic shifts, so the NEON form must be pinned to
  the truncating instruction and equality-witnessed per platform.
- **Lane-wise i64 add/sub** — adds, §2.
- **Sign fixup** — compare + bitwise ops + subtract (negate = subtract
  from zero); no multiply.
- **Table reads** — vector loads of packed terms/exponents are the same
  logical table reads, wider.

## Candidate design

**Layout**: transpose the po2 dot tables from K×D (row per class) to
D×K (row per dim): for each dim d, broadcast `work[d]` once, then for
each group of 4 (AVX2, i64×4) or 2 (NEON) classes: load exponents,
variable-shift, apply sign mask, accumulate into per-lane class scores.
Per stage: D × K/4 × ~6 vector ops ≈ 110k AVX2 ops replacing 295k
scalar term applications, at higher IPC. Two-term tables
(`DOT_TERMS=2`) double the accumulation, unchanged structure.

**Storage**: ⚑ (a) transposed copy stored in the artifact (era bump,
+layout duplication) vs (b) one-time transpose at artifact load
(startup allocation only — steady-state stays allocation-free).
Preference: (b), no format change.

**Structure**: mirror `transformerless/simd.rs` — safe scalar normative
form (the existing `dot_score_plain`, untouched), `unsafe` SIMD in an
isolated adapter module behind runtime feature detection, proptest
equality witness across randomized tables/work vectors, P-4 scan
extended to the adapter (it is contract-clean by construction).

## Empirical Criterion

Ship only if, on the serving wall-clock harness (the ns/token benchmark
called for in the same directive, prerequisite): **≥ 3× on the isolated
dot scan and ≥ 2× end-to-end serving**, at bit-identical outputs on the
fixture artifact (equality witness, existing discipline), zero
allocations preserved (`tests/allocation_census.rs`), and green P-4.
Anything less: the adapter is removed; the recorded benchmark stands as
the negative result.

## Phasing

- **Phase 0 (prerequisite)**: serving wall-clock harness (ns/token,
  no new deps) with the TLA6 and TLA7 paths as its first two data
  points — lands independently of this design.
- **Phase 1**: transposed layout + AVX2 adapter + proptest witness +
  microbenchmark; numbers posted to this issue.
- **Phase 2**: NEON adapter (after the ⚑ rounding semantics are
  pinned), wasm considerations if any (likely none — scalar there).

## Explicit non-goals

No change to assignment semantics, artifact content, or κ labels. No
SIMD multiply/dot-product instructions under any circumstance. No
allocation in steady state. No unsafe outside the isolated adapter.
TLA7 residual subtraction vectorization is out of scope (<2% of cost).

## Sign-off

- Casey: ____   - Ari: ____   - Alex: ____

⚑ decisions: NEON rounding pin · artifact-vs-load-time transpose ·
op-count budget for the adapter
