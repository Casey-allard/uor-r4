# Prime Transport Research-Side Benchmark Validation

## Policy Under Validation

The currently validated prime-transport routing policy is:

- stored state: `R_min = (b, phi, r, next_return_gap)`
- routing key: `base_gap = (b, r, next_return_gap)`
- fallback: `R_full = (b, spin_H)`

This remains an exact-layer routing policy only. No downstream quotient
geometry is involved in this validation status.

## Validation Stages Completed

Two guarded research-side benchmark stages have now been completed.

### 1. Guarded Benchmark Trial

The first guarded trial was run on the original tiny bounded trace family
through the existing research-only wrapper boundary.

Aggregate `base_gap` result:

- unique route keys: `169.0`
- unique emitted route keys: `301.0`
- route reuse fraction: `0.8151848151848151`
- promotion route fraction: `0.2167907268170426`
- promotion step fraction: `0.45274725274725275`
- effective resolved fraction: `0.9813186813186814`
- mean unresolved among nonpromoted routes: `0.014803465594151983`
- route decision instability: `0.0`

### 2. Larger Research-Side Benchmark

The second stage reused the same guarded wrapper boundary on the broader
exact-row trace set derived from the exact-visible rows of
`threshold_law_summary.csv`.

Aggregate `base_gap` result:

- unique route keys: `153.13636363636363`
- unique emitted route keys: `411.5`
- route reuse fraction: `0.6488062739399638`
- promotion route fraction: `0.14382868963876608`
- promotion step fraction: `0.3311747076452959`
- effective resolved fraction: `0.9900083873345916`
- mean unresolved among nonpromoted routes: `0.010289706992288178`
- route decision instability: `0.0`

## Stable Findings

The following findings are now stable across the completed research-side
benchmark stages:

- `base_gap` remains stable under repeated guarded benchmark use
- route decision instability remains `0.0`
- fallback burden does not grow materially on the larger trace set
- `base_gap` remains the intended middle policy between the two controls:
  - `static_only` is too exact and address-like
  - `gap_only` is too coarse and too fallback-heavy
- `base_gap` continues to provide the best current balance between route reuse,
  selective promotion, and effective resolution

## Remaining Risk

The remaining explicit risk is still fallback burden.

Current evidence does **not** indicate:

- a correctness failure
- a transport-update failure
- a route-instability problem

It indicates only:

- a bounded fallback cost that remains visible and should stay explicitly
  accounted for in any future guarded work

## Recommendation

The policy is now sufficiently validated for a future guarded research-side
integration trial beyond the current wrapper boundary.

The smallest future next step should be:

- another research-side benchmark-facing integration trial that still stays
  outside the live MUDBench seam
- reuses the same policy and controls
- keeps fallback accounting explicit
- and only expands the benchmark-facing wrapper usage, not the runtime router
  path

This is enough validation for further guarded research-side integration work,
but not for live seam adoption.
