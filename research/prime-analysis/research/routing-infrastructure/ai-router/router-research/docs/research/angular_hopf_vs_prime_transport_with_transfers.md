# Angular Hopf vs Prime Transport With Transferred Routing Discipline

## Purpose

This note records the first guarded research-side comparison among:

1. the current angular Hopf baseline
2. angular Hopf with the smallest transferred routing discipline
3. prime-transport `base_gap + fallback`

The transfer was intentionally minimal. The angular geometry itself was left
unchanged.

Transferred ideas were restricted to:

- delayed refinement
- explicit fallback accounting
- predictive partitioning

## Comparison Setup

The bounded trace family was the same larger guarded set used in the previous
head-to-head:

- exact-visible rows from `threshold_law_summary.csv`

The compared policies were:

### `angular_hopf`

- existing canonical angular Hopf routing baseline
- no promotion
- no fallback

### `angular_hopf_guarded`

- same canonical angular Hopf route key
- same angular geometry
- route-class unresolved burden measured explicitly
- route class promoted to full fallback when unresolved fraction exceeds the
  same bounded threshold used in the guarded control style

### `base_gap`

- stored state: `R_min = (b, phi, r, next_return_gap)`
- routing key: `base_gap = (b, r, next_return_gap)`
- fallback: `R_full = (b, spin_H)`

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

### `angular_hopf_guarded`

- unique route keys: `7.818181818181818`
- unique emitted route keys: `365.6363636363636`
- route reuse fraction: `0.9367370597317118`
- promotion route fraction: `0.9456168831168832`
- promotion step fraction: `0.9883116883116884`
- fallback step fraction: `0.9883116883116884`
- effective resolved fraction: `1.0`
- mean unresolved among nonpromoted routes: `0.0`
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

The transferred routing discipline does recover the angular Hopf baseline’s
main weakness:

- once delayed refinement and explicit fallback are added, the angular path no
  longer fails on effective resolution

But it recovers that gap only by promoting almost everything:

- `angular_hopf_guarded` fallback step fraction: `0.9883`
- `base_gap` fallback step fraction: `0.3312`

So the transfer experiment cleanly separates two effects:

### What belongs to routing discipline

- the baseline angular failure was partly a discipline failure
- once unresolved route classes are explicitly detected and promoted, the
  angular path becomes predictively correct in this bounded sense

### What belongs to underlying routing structure

- the cost of that correction is structural
- angular Hopf remains too coarse a partition on these traces, so guarded use
  collapses toward near-total fallback
- `base_gap` keeps much more predictive structure inside the nonpromoted
  partition, so it needs far less fallback to stay effective

## Direct Answers

### Does angular Hopf recover most of the gap once delayed refinement, fallback accounting, and predictive partitioning are added?

Yes in resolution, but no in cost.

It recovers the predictive-resolution gap almost completely, but only by
promoting essentially the entire workload.

### Does prime-transport still remain the stronger practical policy even after that transfer?

Yes.

`base_gap` remains the stronger practical policy because it preserves the same
zero-instability behavior while carrying far more of the predictive burden
inside the routed partition:

- effective resolved fraction remains near-complete
- fallback burden is much lower

### Which parts of the win belong to routing discipline versus underlying routing structure?

Both matter, but they matter differently.

- routing discipline explains why the original angular baseline underperformed
  so badly
- underlying routing structure explains why `base_gap` still wins after the
  discipline transfer

The prime-transport advantage now looks like:

- discipline advantage removed
- structural advantage retained

## Conclusion

The angular Hopf path does benefit from the prime-side routing discipline, and
those discipline ideas are worth keeping.

But after that transfer, `base_gap + fallback` still remains the stronger
practical policy on the guarded research-side benchmark boundary.

So the current conservative reading is:

- delayed refinement and fallback accounting should be treated as generally
  useful routing discipline
- the remaining win of prime transport is not just discipline
- it reflects a genuinely stronger bounded predictive routing structure on the
  current trace family
