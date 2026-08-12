# Prime Transport Base-Gap Fallback Burden

## Purpose

This note profiles only the promoted/fallback cases of the current guarded
`base_gap` routing policy in order to answer one bounded implementation-side
question:

- is fallback burden concentrated enough to justify one cheap extra refinement
  before guarded integration?

The policy under analysis remains:

- stored state: `R_min = (b, phi, r, next_return_gap)`
- routing key: `base_gap = (b, r, next_return_gap)`
- fallback: `R_full = (b, spin_H)`

## Aggregate Reading

From
`results/prime_transport_recursive_system/prime_transport_base_gap_fallback_burden.csv`:

- promotion step fraction: `0.45274725274725275`
- promotion route fraction: `0.2167907268170426`
- top-1 promoted route share: `0.0384707044864271`
- top-5 promoted route share: `0.19235352243213544`
- top-1 promoted gap share: `0.06652322457039239`
- top-5 promoted gap share: `0.332616122851962`
- top-1 promoted base share: `0.045654463035810736`
- top-5 promoted base share: `0.2282723151790537`

On the bounded aggregate, fallback burden is best classified as:

- `B_moderately_structured`

## Trace-Family Reading

The one-layer `2310 -> 30030` family is more structured:

- promoted route burden sits in relatively small repeated class sets
- typical top-5 route share is around `0.3333`
- promotion route fractions stay in the `0.125` to `0.24` range

The deeper `30030 -> 510510` family is less concentrated:

- top-5 route share drops to about `0.072` to `0.097`
- promotion route fractions range from `0.1842` up to `0.5263`
- the largest burden is spread over many promoted classes rather than a tiny
  handful of dominant hard cases

So the current fallback burden is not a small isolated hotspot. It is
structured, but on the deeper family it is clearly spread out.

## Recommendation

The right recommendation is:

- **(B) `base_gap` is already the right first guarded integration policy and
  fallback burden should be accepted**

Reason:

- the burden is not concentrated enough to justify one obvious cheap bounded
  refinement
- the deeper-family promoted cases are too spread out for a tiny targeted patch
  to plausibly remove much of the cost
- and the current policy already occupies the right middle ground between the
  address-like static control and the overfallback-heavy `gap_only` control

## Conservative Implementation Reading

This does **not** mean the fallback burden is unimportant.

It means only:

- the current bounded evidence does not point to one clear, cheap additional
  refinement that would be worth inserting before the first guarded benchmark
  integration step

So the next safe step remains:

- keep `base_gap` unchanged
- move into the guarded benchmark-side integration path
- keep fallback accounting explicit so the cost remains observable
