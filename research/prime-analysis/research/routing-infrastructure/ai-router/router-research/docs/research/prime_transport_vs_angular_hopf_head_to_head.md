# Prime Transport vs Angular Hopf Head-to-Head

## Purpose

This note records the first guarded research-side head-to-head comparison
between:

1. the existing canonical angular Hopf routing baseline
2. the prime-transport `base_gap + fallback` policy
3. the trivial `gap_only` control

The comparison stays fully research-only. No live MUDBench seam was touched.

## Comparison Setup

The fixed prime-transport policy was:

- stored state: `R_min = (b, phi, r, next_return_gap)`
- routing key: `base_gap = (b, r, next_return_gap)`
- fallback: `R_full = (b, spin_H)`

The angular baseline used the established MUDBench canonical path:

- `router_core.canonical_angular_router.build_canonical_angular_prompt_plan`
- default variant: `angular-hopf-trans`

For a fair research-only comparison at the same wrapper boundary, each exact
trace position was projected into the angular contract by a minimal adapter:

- first angular pair: exact base phase `b mod 35`
- second angular pair: first fiber phase when present, otherwise bounded
  next-return-gap

The comparison ran on the same larger reproducible exact-row trace set used by
the guarded research-side benchmark:

- exact-visible rows from `threshold_law_summary.csv`

## Aggregate Result

### `angular_hopf`

- unique route keys: `7.818181818181818`
- unique emitted route keys: `7.818181818181818`
- route reuse fraction: `0.9367370597317118`
- promotion route fraction: `0.0`
- promotion step fraction: `0.0`
- fallback step fraction: `0.0`
- effective resolved fraction: `0.16810432050297877`
- mean unresolved among nonpromoted routes: `0.8318956794970213`
- route decision instability: `0.0`

### `gap_only`

- unique route keys: `41.0`
- unique emitted route keys: `373.27272727272725`
- route reuse fraction: `0.8670313643575676`
- promotion route fraction: `0.8125899411970444`
- promotion step fraction: `0.946935951748786`
- fallback step fraction: `0.946935951748786`
- effective resolved fraction: `0.9988647715920446`
- mean unresolved among nonpromoted routes: `0.00937114356232003`
- route decision instability: `0.0`

### `base_gap`

- unique route keys: `153.13636363636363`
- unique emitted route keys: `411.5`
- route reuse fraction: `0.6488062739399638`
- promotion route fraction: `0.14382868963876608`
- promotion step fraction: `0.3311747076452959`
- fallback step fraction: `0.3311747076452959`
- effective resolved fraction: `0.9900083873345916`
- mean unresolved among nonpromoted routes: `0.010289706992288178`
- route decision instability: `0.0`

## Reading

The direct head-to-head is decisive on this guarded benchmark boundary.

`angular_hopf` does win on raw route-key reuse:

- far fewer route keys
- very high reuse
- zero fallback burden

But it loses the predictive-routing tradeoff that matters on this benchmark:

- effective resolved fraction collapses to only `0.1681`
- mean unresolved mass inside the nonpromoted angular classes is `0.8319`

So on the same bounded exact traces, the canonical angular Hopf partition is
much too coarse to serve as the stronger practical routing policy by itself.

`base_gap` is the better middle policy:

- much higher effective resolution than `angular_hopf`
- much lower fallback burden than `gap_only`
- zero instability

`gap_only` still resolves well only by promoting almost everything, so it
remains a useful coarse control rather than the preferred policy.

## Direct Answers

### Does prime-transport `base_gap` outperform angular Hopf on a meaningful bounded routing tradeoff?

Yes.

It substantially outperforms the angular Hopf baseline on the key practical
tradeoff between reuse and retained predictive resolution:

- `base_gap` effective resolved fraction: `0.9900`
- `angular_hopf` effective resolved fraction: `0.1681`

The cost is explicit fallback burden, but that burden remains bounded at
`0.3312` step fraction and does not create instability.

### Does angular Hopf remain the better practical policy?

No, not on this bounded research-side setup.

The existing angular Hopf baseline remains attractive as a very compact static
routing partition, but in this directly comparable experiment it is too coarse
to preserve enough predictive structure.

### What should transfer back into the angular Hopf path?

Even if the angular path is revisited later, three ideas now look worth
transferring back:

- delayed refinement rather than one-shot coarse partitioning
- explicit fallback accounting rather than implicit unresolved mixing
- predictive partitioning judged against downstream unresolved burden, not just
  route-key reuse

## Conclusion

On the current guarded research-side benchmark boundary, `base_gap + fallback`
is the stronger practical policy.

The angular Hopf path does not win the first direct head-to-head. Its main
remaining value here is conceptual:

- it shows that extremely compact static routing is possible
- but the prime-transport branch now gives the stronger bounded policy because
  it couples compact routing with selective predictive refinement

So the current recommendation is:

- keep `base_gap + fallback` as the leading research-side routing candidate
- treat angular Hopf as the compact static comparison baseline
- carry the prime-side lessons about delayed refinement and explicit fallback
  accounting into any future angular-side practical routing work
