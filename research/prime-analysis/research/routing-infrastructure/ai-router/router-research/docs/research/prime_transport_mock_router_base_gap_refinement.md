# Prime Transport Mock Router Base-Gap Refinement

## Purpose

This note records the next bounded offline prototype step after the first
`base_gap` routing prototype.

The question was:

- can one minimal exact-layer ingredient reduce `base_gap` promotion materially
  without collapsing back toward full-spin complexity?

## Promoted-Case Reading

The promoted `base_gap` cases were inspected on the same bounded real traces.

The missing distinction was most often phi-side rather than a new return-memory
 label:

- on the one-layer `2310 -> 30030` family, the promoted cases are completely
  separated by the first fiber coordinate
- on the two-layer `30030 -> 510510` family, the first fiber coordinate still
  reduces ambiguity substantially, though not completely

So the single refinement candidate used here was:

- `base_gap_phi0 = (b, r, next_return_gap, phi0)`

where `phi0` is the first fiber-phase coordinate.

## Comparison

Aggregate bounded trace results:

### `base_gap`

- unique route keys: `169.0`
- route reuse fraction: `0.8151848151848151`
- promotion fraction: `0.45274725274725275`
- post-promotion resolution fraction: `0.9925247635625244`

### `base_gap_phi0`

- unique route keys: `672.5`
- route reuse fraction: `0.31178821178821176`
- promotion fraction: `0.22417582417582418`
- post-promotion resolution fraction: `0.9938265671792292`

### Full-spin baseline

- unique route keys: `255.0`
- route reuse fraction: `0.7908091908091908`
- promotion fraction: `0.0`
- post-promotion resolution fraction: `1.0`

## Reading

`base_gap_phi0` does reduce promotion materially:

- from `0.4527` down to `0.2242`

But it pays for that by making the routing partition much finer:

- unique route keys jump from `169.0` to `672.5`
- route reuse drops from `0.8152` to `0.3118`

So the refinement is real, but the tradeoff is not attractive for the first
prototype.

It improves ambiguity only by moving much closer to an address-like partition.

## Recommendation

`base_gap_phi0` is **not** better enough to replace `base_gap` as the first
offline routing prototype.

The current recommendation remains:

- keep `base_gap = (b, r, next_return_gap)` as the first prototype target

because it preserves the better routing tradeoff:

- strong reuse
- meaningful unresolved signal
- moderate promotion
- near-complete post-promotion resolution

## Conservative Conclusion

The current offline exact-layer result is:

- the promoted `base_gap` cases do contain a real phi-side missing distinction
- but adding `phi0` immediately makes the routing partition much finer
- so `base_gap` remains the right first prototype target

The next offline step, if any, should be smaller than a direct phi coordinate
add-on:

- either inspect promoted `base_gap` cases for a still coarser refinement
- or stop here and use `base_gap` as the last offline design before any
  implementation-side prototype wiring
