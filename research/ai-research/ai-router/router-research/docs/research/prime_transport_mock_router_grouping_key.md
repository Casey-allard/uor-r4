# Prime Transport Mock Router Grouping Key

## Purpose

This note records the next offline design step after the trace-level mock router
evaluation.

The trace-level result showed that `R_min = (b, phi, r, next_return_gap)` is a
valid exact-layer state, but the current literal routing key

- `(b, phi, r, next_return_gap)`

is too fine-grained to act as a reusable routing partition on real traces.

It behaves like a per-position address:

- update dynamics are exact
- but route classes are almost pointwise
- so unresolved ambiguity vanishes before promotion has any work to do

The problem is therefore not state validity. It is routing-key granularity.

## Candidate Family

The bounded comparison used the same real trace sources as the previous trace
evaluation and tested only a very small exact-layer family:

- `literal_rmin = (b, phi, r, next_return_gap)`
- `base_gap = (b, r, next_return_gap)`
- `phi_gap = (phi, r, next_return_gap)`
- `base_zero_gap = (b, r, zero_count(phi), next_return_gap)`
- `gap_only = (r, next_return_gap)`

No arbitrary clustering was used.

## Bounded Reading

Across the bounded traces:

- `literal_rmin` stays exact but gives one route key per position and no
  promotion at all
- `phi_gap` is still too fine-grained to serve as the first reusable routing
  class
- `gap_only` is too coarse and promotes almost everything
- `base_zero_gap` improves mean ambiguity slightly, but it is not meaningfully
  smaller than the average full-spin partition
- `base_gap` is the cleanest first compromise

On the aggregate bounded rows:

- `literal_rmin`
  - average unique route keys: `2233.0`
  - mean unresolved fraction within route: `0.0`
  - promotion fraction: `0.0`
- `base_gap`
  - average unique route keys: `169.0`
  - mean unresolved fraction within route: `0.12668687785676264`
  - promotion fraction: `0.45274725274725275`
- `phi_gap`
  - average unique route keys: `1898.3`
  - mean unresolved fraction within route: `0.12181115374698126`
  - promotion fraction: `0.3811788211788211`
- `base_zero_gap`
  - average unique route keys: `257.1`
  - mean unresolved fraction within route: `0.09857052066484633`
  - promotion fraction: `0.4415784215784216`
- `gap_only`
  - average unique route keys: `75.5`
  - mean unresolved fraction within route: `0.3899274722242755`
  - promotion fraction: `0.9532467532467532`

## Recommendation

The best first coarse routing key is:

- `base_gap = (b, r, next_return_gap)`

Reason:

- it is much coarser than literal `R_min`
- it remains smaller than the average full-spin partition on the bounded rows
- it produces meaningful unresolved ambiguity
- and it triggers promotion at a moderate rate rather than at `0` or almost
  `1`

So `base_gap` is the best current first routing class for the offline prototype.

## Conservative Conclusion

The current exact-layer reading is:

- `R_min` remains the right default state object
- but its literal identity should not be used as the routing partition
- the first practical routing partition should be a coarser key derived from
  `R_min`
- on current evidence, `base_gap = (b, r, next_return_gap)` is the best first
  candidate

The next offline design step should therefore be:

- keep `R_min` as the stored state
- switch the mock router's route partition to `base_gap`
- then rerun the trace-level promotion path with that coarser routing key
