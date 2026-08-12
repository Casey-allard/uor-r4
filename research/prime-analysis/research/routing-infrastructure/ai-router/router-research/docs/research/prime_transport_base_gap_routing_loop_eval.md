# Prime Transport Base-Gap Routing Loop Evaluation

## Purpose

This note records a slightly more realistic offline routing-loop evaluation for
 the selected exact-layer prototype:

- stored state: `R_min = (b, phi, r, next_return_gap)`
- routing partition: `base_gap = (b, r, next_return_gap)`
- fallback: `R_full = (b, spin_H)`

The goal was to test whether `base_gap` remains stable when used as a repeated
route class over real trace sequences, rather than only as a static partition.

## Loop Model

For each bounded trace:

1. build `R_min` at each exact position
2. derive the `base_gap` route key
3. cache the routing decision for each encountered route key
4. reuse that cached decision on later hits of the same route class
5. promote only those route classes whose unresolved fraction exceeds the
   existing threshold
6. resolve promoted steps through `R_full`

This keeps the partition fixed and tests whether the route classes behave
stably under repeated use.

## Aggregate Result

Across the bounded traces:

- unique `base_gap` route classes: `169.0`
- route reuse fraction: `0.8151848151848151`
- promotion route fraction: `0.2167907268170426`
- promotion step fraction: `0.45274725274725275`
- average fallback burden in steps: `1600.0`
- mean promoted route-class size: `22.412142857142857`
- mean nonpromoted route-class size: `4.637273084672466`
- effective resolved fraction: `0.9813186813186814`
- mean unresolved fraction among nonpromoted routes: `0.014803465594151983`
- max unresolved fraction among nonpromoted routes: `0.2857142857142857`
- route decision instability: `0.0`

## Reading

The loop-level result is encouraging in the bounded offline setting:

- route classes are reused heavily
- promotion remains selective rather than universal
- promoted classes tend to be larger and therefore worth treating separately
- route decisions remain stable once cached

The effective resolved fraction is slightly below the post-promotion partition
summary because nonpromoted classes are allowed to remain approximate rather
than being forced to exact resolution.

That is the intended routing behavior:

- use the coarse route when it is good enough
- promote only the unresolved classes that need richer prediction

## Recommendation

`base_gap` now looks mature enough to be the first candidate for a guarded
non-runtime integration experiment.

Reason:

- the partition is no longer only theoretically plausible
- it behaves stably as a reusable routing policy on real bounded traces
- and the fallback burden remains selective rather than overwhelming

## Conservative Next Step

No live integration is justified yet.

But one guarded implementation-side experiment is now justified:

- wire the same `base_gap` policy into a non-runtime research-side prototype
  module boundary
- keep promotion explicit and observable
- keep `R_full` as the only fallback
- and verify that the same reuse/promotion pattern survives outside the current
  trace-evaluation scripts
