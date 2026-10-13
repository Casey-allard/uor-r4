# Prime Transport Visible Threshold Canonical Law Class

## Purpose

This note consolidates the current exact-layer threshold-law result for the
prime admissibility system into one durable reference.

It is the canonical exact-layer statement of what is currently supported about

- `H_visible_first(A, W -> pW)`

inside the recursive admissibility system.

This note does **not** use quotient geometry, `C^2`, `A_*`, packet recovery, or
runtime routing claims.

## 1. Exact Recursive-System Facts

### Transport Orbit

The exact transport orbit is

- `n(j) = 5 + 6j`

with odometer update

- `j -> j + 1`

along the reduced wheel orbit.

### Exact State

The exact finite-depth state factors as

- `j -> (b, phi, r)`

where:

- `b` is the base phase
- `phi` collects the refinement-layer fiber phases
- `r` is wheel depth

Finite depth is torus-valued, and wheel lift adds new refinement-layer fiber
structure.

### Finite-Horizon Spin

For horizon `H`, let

- `spin_H(j)`

be the exact length-`H` admissibility future word.

The compressed predictive state is

- `(b(j), spin_H(j))`

This is the exact compressed predictive object used in the threshold analysis.

### Internal Split vs Visible Split

For a lift `W -> pW` and a tuplet pattern `A`:

- **internal split at horizon `H`** means child lifts can already diverge in
  one-step future evolution while still sharing the same current `spin_H`
  state
- **visible split at horizon `H`** means the child wheel has strictly more
  distinct predictive states `(b, spin_H)` than the parent wheel

So internal split is hidden activation, while visible split is actual predictive
state-count refinement.

## 2. Threshold-Law Facts Now Supported

### Immediate Internal Split

Across the current exact threshold table,

- `first_internal_split_H = 1`

for every tested lift and every tested tuplet.

So newly added layers become internally active immediately in the current exact
data.

### Delayed Visible Split

Visible split is the nontrivial threshold object.

It can be much later than internal split. In the current table:

- twins: visible `1, 2, 4, 6`
- triplet: visible `1, 6, 16, 26`
- quadruplet: visible `1, 21, 51, >60`

So delayed visibility is an exact observed phenomenon in the recursive system.

### What Is Insufficient

The current evidence rules out the following as complete descriptions on their
own:

- density alone
- lift prime alone
- tuplet burden alone
- simple local arrangement alone
- simple global interaction averages alone

In particular:

- density is the strongest single coarse control on the broad event table, but
  density-conditioned tests still leave strong residual structure
- arrangement matters in fixed-`p`, fixed-`nu_p(A)` slices, but simple
  arrangement statistics such as `gap_max` do not dominate the matched evidence

### Best Current Exact-Layer Law Class

The strongest current exact-layer law class is:

- `density + first-splitting event`

More explicitly:

- predictive visibility is governed by inherited parent-grammar density
  together with the size and multiplicity of the first visible splitting event

### Arrangement on Current Matched Evidence

Arrangement is currently secondary on the tight matched evidence.

In both tightly density-matched families tested so far, first-splitting-event
statistics dominate simple arrangement statistics like `gap_max`.

So the current evidence does **not** justify promoting arrangement correction to
the leading term of the law class.

## 3. Evidence Hierarchy

### Global Table

The global threshold table establishes:

- immediate internal split
- delayed visible split
- strong variation across lifts and tuplets

That table is recorded in
[threshold_law_summary.csv](/Users/adminamn/AI-Research/ai-router/router-research/results/prime_transport_recursive_system/threshold_law_summary.csv).

### Event-Level First-Splitting Table

The first-splitting-event analysis shows that visible threshold is better
organized by exact splitting events than by earlier global interaction
averages.

On the broad event table:

- `num_first_splitting_classes`: Spearman `0.820410093133918`
- `total_extra_child_classes_from_first_split`: `0.7982368473735418`

with parent density still the strongest single coarse control:

- parent admissible density: `-0.8593693199561495`

This establishes first-splitting-event structure as a core part of the law
class.

### Density-Conditioned Result

After coarse parent-density conditioning, the event statistics still carry
strong residual signal:

- `num_first_splitting_classes_residual`: `0.8144338729465652`
- `total_extra_child_classes_residual`: `0.7879242712955016`
- `fraction_first_splitting_classes_residual`: `0.7776149817645324`

So density alone is not enough.

### Tightly Density-Matched Family 1

On the exact `2310 -> 30030` matched family with fixed parent density
`0.05454545454545454`:

- `num_first_splitting_classes`: `0.9710083124552245`
- `total_extra_child_classes_from_first_split`: `0.9710083124552245`
- `fraction_first_splitting_classes`: `0.9710083124552245`
- `gap_max`: `0.2537596094612761`

### Tightly Density-Matched Family 2

On the exact `30030 -> 510510` matched family with fixed parent density
`0.03776223776223776`:

- `num_first_splitting_classes`: `1.0`
- `total_extra_child_classes_from_first_split`: `1.0`
- `fraction_first_splitting_classes`: `1.0`
- `gap_max`: `0.31622776601683794`

Taken together, these two matched families are the strongest current evidence
that the stable exact-layer law class is:

- `density + first-splitting event`

with arrangement secondary on the present matched slices.

## 4. Conservative Conclusion

The current result is:

- a canonical exact-layer law class / mechanism class

not:

- a closed exact formula for `H_visible_first`
- a theorem that arrangement never matters
- a finished universal recursion law

What should now be treated as canonical exact-layer knowledge is:

1. internal split is immediate
2. visible split is the true delayed threshold object
3. density alone is insufficient
4. burden and simple arrangement alone are insufficient
5. the best current exact-layer law class is
   `density + first-splitting event`
6. arrangement is secondary on current tightly matched evidence

This is the exact-layer result future routing-theory work should inherit before
attempting any downstream quotient or transport approximation.

## Stable Artifact References

- global threshold note:
  [prime_transport_threshold_law.md](/Users/adminamn/AI-Research/ai-router/router-research/docs/research/prime_transport_threshold_law.md)
- first-splitting note:
  [prime_transport_first_splitting_classes.md](/Users/adminamn/AI-Research/ai-router/router-research/docs/research/prime_transport_first_splitting_classes.md)
- density-conditioned note:
  [prime_transport_density_conditioned_first_splitting.md](/Users/adminamn/AI-Research/ai-router/router-research/docs/research/prime_transport_density_conditioned_first_splitting.md)
- first matched-family note:
  [prime_transport_tight_density_matched_first_splitting.md](/Users/adminamn/AI-Research/ai-router/router-research/docs/research/prime_transport_tight_density_matched_first_splitting.md)
- second matched-family note:
  [prime_transport_second_tight_density_matched_first_splitting.md](/Users/adminamn/AI-Research/ai-router/router-research/docs/research/prime_transport_second_tight_density_matched_first_splitting.md)
