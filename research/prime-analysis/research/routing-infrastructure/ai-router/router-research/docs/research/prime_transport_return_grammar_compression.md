# Prime Transport Return-Grammar Compression

## Residual Question

The residual question is:

- what predictive distinctions does `(b, spin_H)` carry that are not already
  practically captured by a smaller exact dynamical object?

Because `j -> (b, phi, r)` is already an exact coordinatization of the orbit,
this must be treated as a **compressive predictive** problem, not raw
information subtraction.

## Candidate Family

The bounded candidate family was kept intentionally small:

- `prev_next_gap_pair`
- `next_return_gap`
- `return_gap_seq_2`
- `return_gap_seq_3`
- `transition_memory_3`
- `branch_identity_min`

These are all exact return-grammar or transition-memory labels built on the
same existing tightly matched first-splitting rows.

## Evaluation

Each candidate was compared against the spin-side target in two ways:

1. how well it predicts first-splitting membership
2. how much of the predictive partition on first-splitting support it captures

The comparison also tracked whether the candidate remained compressed relative
to spin, via its label-count ratio to the spin partition.

## Result

The strongest raw capture came from:

- `return_gap_seq_3`

with:

- mean combined capture fraction of spin: `0.973799550914556`
- mean split label ratio to spin: `2.9235293321401783`

So `return_gap_seq_3` reproduces most of the spin-side distinctions, but it
does so by using substantially **more** labels than the spin partition itself.
That is not a genuine compression.

The next strongest family members were:

- `return_gap_seq_2`: mean combined capture `0.9675504681489274`, label ratio
  `1.7124951811424984`
- `branch_identity_min`: same score as `return_gap_seq_2`

These also remain label-expansive rather than compressive.

Among the genuinely smaller candidates, the best was still:

- `prev_next_gap_pair`

with:

- mean combined capture fraction of spin: `0.8118207348148487`
- mean split label ratio to spin: `1.539815778208287`

The simplest compressed candidate was:

- `next_return_gap`

with:

- mean combined capture fraction of spin: `0.7502981320146836`
- mean split label ratio to spin: `0.6121906310452204`

Short local transition memory remained weak:

- `transition_memory_3`: mean combined capture `0.11687144795266549`

## Interpretation

The current exact-layer reading is:

- return-grammar structure is clearly relevant
- short local transition memory is too weak
- small return-memory objects capture a substantial part of the residual
- but the best high-capture return-grammar candidates are not yet true
  compressions, because they expand the label space beyond spin

So the bounded result should be classified as:

- **(B) partially compressible but still significantly weaker than spin**

This is the right classification because:

- high-capture candidates exist
- but they are not yet small compressive objects
- and the genuinely smaller candidates remain materially weaker than full spin

## Current Best Reading

The residual beyond phase-fiber-scale now looks most plausibly like a
return-grammar object, not a short local bit-memory object.

But the present evidence does not yet justify replacing full spin with a small
exact return-grammar object.

The next exact-layer question is therefore:

- can one find a return-grammar object that keeps most of the spin-side capture
  while avoiding the label explosion seen in `return_gap_seq_2` and
  `return_gap_seq_3`?
