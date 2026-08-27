# Design: SIMD Shift-Add Dot Scan (Contract-Legal Serving Speedup)

> **Historical optimization proposal.** This design belongs to the preserved
> dot-scan runtime and is not the current #963 optimization target. Profile the
> accepted route-native product path before reviving it. See the
> [Geometric Intelligence Programme](geometric_intelligence_programme.md).

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
  points — implemented as `cargo bench -p uor-r4-core
  --bench transformerless_dot -- <ARTIFACT> <ITERATIONS>` (the default
  artifact is `tests/fixtures/tless_artifacts.bin`).
  The harness reports the scalar dot scan and complete runtime assignment,
  and prints checksums that an adapter must preserve.
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

## Outcome: the assign path (issue #469 lever B, 2026-08-07)

Phases 1 and 2 landed the adapters; this records where they are now
consumed, because the answer was not "everywhere" and the gap was not
obvious from the code.

`Runtime::dot_argmax` had a cached `DotTables` and used the adapter. The
free-function assign path — `assign_for_bundle` / `assign_code_for_bundle`,
which is what every corpus-scale code pass actually calls — did not, and
could not: `DotTables::from_packed` is a **load-time** expansion of the
packed u16 tables, and a free function has nowhere to cache it. Measured on
the pinned TLA7 artifact, that expansion costs 2,337 us against ~487 us for
one whole scalar assignment. Calling the adapter per bundle would therefore
have been roughly a **4x regression**, not a speedup — the adapter is only a
win once the decode is amortized.

`runtime::AssignTables` closes that gap by making the amortization explicit:
build once per artifact, share across rayon workers, call
`assign_code_for_bundle_with` / `code_plain_with` per bundle. On the pinned
TLA7 artifact the corpus pass measures **310.6 us/call against 496.3 us/call
(1.60x)**, and the one-time table build pays for itself after 7.5 calls
against the 500,000 in a corpus pass. Consumers wired: the Gate C left-code
pass (on a sidecar miss) and `derive_right_codes` (uncached, so every scored
run).

Two results worth keeping because they redirected the work:

- **The membership beam is not the cost.** `assign_for_bundle` builds the
  full top-M beam and discards everything but the primary code, which looked
  like an obvious waste. Removing it measured **0.95x** — nothing. The
  `K x D` scan dominates so completely that the beam is inside the noise. The
  hypothesis was cheap to test and would have been an expensive assumption.
- **The non-goal above still holds.** TLA7 residual subtraction remains
  unvectorized; the residual path gains here because its per-stage *scan* now
  uses the adapter, not because the subtraction changed.

The equality obligation is discharged in `tests/assign_prepared.rs`
(prepared == scalar over both artifact shapes, the sign-metric fallback, the
short-table stage arity, and 1,024 real corpus positions on the committed
artifact fixture) and in `tests/kappa_reproduction.rs` on a fresh compile.
The fixture-based test fails rather than skips when its inputs are missing,
because the κ test skips wherever the pinned checkpoint is absent and a
κ-pinned change cannot rest on a gate that silently does not run (issue #354).
