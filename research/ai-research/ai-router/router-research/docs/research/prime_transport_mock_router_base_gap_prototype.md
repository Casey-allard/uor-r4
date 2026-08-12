# Prime Transport Mock Router Base-Gap Prototype

## Purpose

This note records the first full offline routing prototype using:

- stored state: `R_min = (b, phi, r, next_return_gap)`
- routing partition: `base_gap = (b, r, next_return_gap)`
- promotion fallback: `R_full = (b, spin_H)`

The goal was to test whether the first coarse exact-layer routing key identified
in the grouping-key comparison is strong enough to serve as the first serious
offline prototype.

## Prototype Flow

On the bounded real traces:

1. initialize `R_min`
2. derive a `base_gap` routing key
3. route by that grouped key
4. promote only those grouped cases whose unresolved fraction exceeds the
   existing promotion threshold
5. resolve promoted cases by falling back to `R_full`

The same traces were also compared against:

- literal `R_min` routing
- `gap_only`
- full-spin baseline

## Aggregate Reading

### Literal `R_min`

- average unique route keys: `2233.0`
- route reuse fraction: `0.0`
- promotion fraction: `0.0`
- post-promotion resolution fraction: `1.0`

This remains exact, but it is not a usable routing partition because it behaves
like a pointwise address.

### `base_gap = (b, r, next_return_gap)`

- average unique route keys: `169.0`
- route reuse fraction: `0.8151848151848151`
- pre-promotion mean unresolved fraction: `0.12668687785676264`
- promotion fraction: `0.45274725274725275`
- post-promotion mean unresolved fraction: `0.007475236437475645`
- post-promotion resolution fraction: `0.9925247635625244`

This is the first candidate that behaves like a real routing partition:

- substantial route reuse
- meaningful but not near-total promotion
- near-complete effective resolution after promotion

### `gap_only`

- average unique route keys: `75.5`
- route reuse fraction: `0.9153846153846155`
- pre-promotion mean unresolved fraction: `0.3899274722242755`
- promotion fraction: `0.9532467532467532`
- post-promotion resolution fraction: `0.9937146288676437`

This resolves well only because it promotes almost everything. So it is too
coarse to be the first practical prototype key.

### Full-spin baseline

- average unique route keys: `255.0`
- route reuse fraction: `0.7908091908091908`
- promotion fraction: `0.0`
- post-promotion resolution fraction: `1.0`

This remains the exact fallback reference.

## Recommendation

`base_gap` is strong enough to be the first serious offline routing prototype.

Why:

- it is much coarser than literal `R_min`
- it preserves meaningful route reuse
- it exposes real unresolved ambiguity
- and it resolves that ambiguity well after selective promotion, without
  collapsing into almost universal fallback

## Failure / Pathology Reading

The main remaining limitation is not transport correctness.

It is that `base_gap` still leaves a nontrivial promoted fraction:

- promotion fraction: `0.45274725274725275`

So the next offline step before any live integration should be:

- keep `R_min` as stored state
- keep `base_gap` as the first routing partition
- inspect the promoted cases specifically and test whether one minimal
  exact-layer refinement on top of `base_gap` reduces promotion materially
  without losing reuse

## Conservative Conclusion

The current offline exact-layer result is:

- `base_gap` is good enough to be the first serious routing prototype
- it is not final
- and any next refinement should stay small, explicit, and grounded in the same
  exact-layer objects rather than expanding toward full spin by default
