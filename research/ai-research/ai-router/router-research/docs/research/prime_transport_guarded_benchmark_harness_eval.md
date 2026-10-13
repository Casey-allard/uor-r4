# Prime Transport Guarded Benchmark Harness Evaluation

## Purpose

This note records the first guarded benchmark-harness experiment for the
prime-transport routing policy using the research-only wrapper boundary.

The policy under test is:

- stored state: `R_min = (b, phi, r, next_return_gap)`
- routing key: `base_gap = (b, r, next_return_gap)`
- fallback: `R_full = (b, spin_H)`

The controls used on the same bounded traces were:

- `static_only`
- `gap_only`

No live router seam was touched.

## Aggregate Comparison

### `static_only`

- unique route keys: `2233.0`
- unique emitted route keys: `2233.0`
- route reuse fraction: `0.0`
- promotion route fraction: `0.0`
- promotion step fraction: `0.0`
- effective resolved fraction: `1.0`
- route decision instability: `0.0`

Reading:

- exact but address-like
- not a useful reusable routing partition

### `gap_only`

- unique route keys: `75.5`
- unique emitted route keys: `280.0`
- route reuse fraction: `0.9153846153846155`
- promotion route fraction: `0.6687298741326874`
- promotion step fraction: `0.9532467532467532`
- effective resolved fraction: `0.9960039960039964`
- route decision instability: `0.0`

Reading:

- very coarse reuse
- resolves well only by promoting almost everything
- too fallback-heavy to be the preferred first policy

### `base_gap`

- unique route keys: `169.0`
- unique emitted route keys: `301.0`
- route reuse fraction: `0.8151848151848151`
- promotion route fraction: `0.2167907268170426`
- promotion step fraction: `0.45274725274725275`
- effective resolved fraction: `0.9813186813186814`
- route decision instability: `0.0`

Reading:

- much more reusable than `static_only`
- much less promotion-heavy than `gap_only`
- stable under repeated bounded harness use
- still leaves a nontrivial fallback burden, but not a pathological one

## Recommendation

`base_gap` is strong enough to justify a future guarded benchmark-side
integration step.

The smallest safe next hookup point is:

- the research-only benchmark wrapper boundary already built around the bounded
  trace-building path

That keeps the policy:

- outside the live MUDBench seam
- directly comparable against simple controls
- and observable in terms of route reuse, promotion, and fallback burden

## Remaining Risk

The main unresolved risk is not instability.

It is bounded fallback cost:

- promotion still occurs on `45.27%` of bounded trace steps

So before any live seam work, the next guarded step should remain:

- benchmark-side only
- explicit about fallback accounting
- and limited to the current bounded trace family or a very small extension of
  it
