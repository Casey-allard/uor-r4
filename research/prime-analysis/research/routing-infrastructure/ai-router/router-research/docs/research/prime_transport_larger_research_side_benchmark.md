# Prime Transport Larger Research-Side Benchmark

## Purpose

This note records the first larger research-side benchmark experiment executed
through the same guarded wrapper boundary as the earlier trial, but on the next
larger reproducible exact-row trace set.

The policy under test remained fixed:

- stored state: `R_min = (b, phi, r, next_return_gap)`
- routing key: `base_gap = (b, r, next_return_gap)`
- fallback: `R_full = (b, spin_H)`

The controls also remained fixed:

1. `static_only`
2. `gap_only`
3. `base_gap + fallback`

## Larger Trace Set

The larger research-side trace set was built from the exact-visible rows of:

- `results/prime_transport_recursive_system/threshold_law_summary.csv`

using the same research-only wrapper boundary and the same exact trace
reconstruction path already used in the smaller guarded trial.

This produced:

- `66` trace-policy rows
- aggregate trace length `372890`

So the experiment is larger, but still fully guarded and reproducible.

## Aggregate Result

### `static_only`

- unique route keys: `16949.545454545456`
- unique emitted route keys: `16949.545454545456`
- route reuse fraction: `0.0`
- promotion route fraction: `0.0`
- promotion step fraction: `0.0`
- effective resolved fraction: `1.0`
- mean unresolved among nonpromoted routes: `0.0`
- route decision instability: `0.0`

### `gap_only`

- unique route keys: `41.0`
- unique emitted route keys: `373.27272727272725`
- route reuse fraction: `0.8670313643575676`
- promotion route fraction: `0.8125899411970444`
- promotion step fraction: `0.946935951748786`
- effective resolved fraction: `0.9988647715920446`
- mean unresolved among nonpromoted routes: `0.00937114356232003`
- route decision instability: `0.0`

### `base_gap`

- unique route keys: `153.13636363636363`
- unique emitted route keys: `411.5`
- route reuse fraction: `0.6488062739399638`
- promotion route fraction: `0.14382868963876608`
- promotion step fraction: `0.3311747076452959`
- effective resolved fraction: `0.9900083873345916`
- mean unresolved among nonpromoted routes: `0.010289706992288178`
- route decision instability: `0.0`

## Reading

`base_gap` remains stable on the larger trace set:

- route decision instability stays at `0.0`
- it still occupies the intended middle position between the two controls
- and fallback burden does not grow materially

In fact, on this larger exact-row set:

- promotion step fraction falls from the earlier bounded-trial value of
  `0.4527` to `0.3312`

So the larger research-side benchmark does not weaken the current policy
reading. It strengthens it.

## Conclusion

The current policy remains the right candidate for any future guarded
benchmark-side integration beyond the current research boundary.

The remaining guarded risk is still fallback burden, but on this larger exact
set it stays acceptable and does not show instability or collapse toward the
`gap_only` regime.

So the next safe step is:

- a larger research-side benchmark experiment or wrapper-level integration
  exercise using the same guarded boundary and the same explicit fallback
  accounting

It is still **not** a live seam step.
