# Prime Transport Tight Density-Matched First Splitting

## Purpose

This note tightens the current law class by testing a small exact family in
which parent admissible density is held fixed as tightly as possible, while
first-splitting multiplicity and local arrangement still vary.

The goal is to decide whether, under tighter density matching, the better
description is:

- `density + first-splitting event`

or

- `density + first-splitting event + arrangement correction`

The analysis remains entirely inside the exact recursive-system layer.

## Matched Family Used

To avoid a broad sweep, a single exact lift family was used:

- `2310 -> 30030`
- new prime `p = 13`

Six size-4 tuplets were selected, all with exactly the same parent admissible
density:

- `0.05454545454545454`

Rows used:

- `quadruplet = [0,2,6,8]`
- `double_twins_p17 = [0,2,34,36]`
- `quad_alt_0618 = [0,2,6,18]`
- `quad_alt_0826 = [0,2,8,26]`
- `quad_alt_1836 = [0,2,18,36]`
- `quad_alt_3854 = [0,2,38,54]`

Saved matched rows:
  [visible_threshold_tight_density_matched_rows.csv](/Users/adminamn/AI-Research/ai-router/router-research/results/prime_transport_recursive_system/visible_threshold_tight_density_matched_rows.csv)

Saved class table:
  [visible_threshold_tight_density_matched_first_split_classes.csv](/Users/adminamn/AI-Research/ai-router/router-research/results/prime_transport_recursive_system/visible_threshold_tight_density_matched_first_split_classes.csv)

Saved matched ranking table:
  [visible_threshold_tight_density_matched_scores.csv](/Users/adminamn/AI-Research/ai-router/router-research/results/prime_transport_recursive_system/visible_threshold_tight_density_matched_scores.csv)

## Main Matched Result

Within this exact density-matched family, visible thresholds range from:

- `6`
- `16`
- `21`
- `26`

while the first-splitting statistics vary strongly:

- `num_first_splitting_classes`: `18` to `148`
- `total_extra_child_classes_from_first_split`: `22` to `244`
- `fraction_first_splitting_classes`: `0.36` to `0.822222...`

The matched-family ranking is:

- `num_first_splitting_classes`: Spearman `0.9710083124552245`
- `total_extra_child_classes_from_first_split`: Spearman `0.9710083124552245`
- `fraction_first_splitting_classes`: Spearman `0.9710083124552245`
- `gap_max`: Spearman `0.2537596094612761`
- `gap_span`: Spearman `-0.2537596094612761`

`new_prime` is constant in this matched family and therefore irrelevant here.

## Interpretation

### 1. First-splitting multiplicity survives tight density matching

This is the main result.

When parent admissible density is held exactly fixed in this family,
first-splitting-event multiplicity remains very strong, while the simple
arrangement statistic `gap_max` becomes weak.

So the current evidence favors:

- `density + first-splitting event`

over:

- `density + first-splitting event + gap_max correction`

at least on this exact matched slice.

### 2. Arrangement is not erased, but it is not the leading correction here

The matched rows do still vary in local arrangement:

- `gap_max` ranges from `4` to `9`

Yet that arrangement variation does not organize the visible threshold nearly
as well as the first-splitting statistics do.

So the present matched-family evidence does not support promoting `gap_max` to
the main residual correction term.

### 3. Current best exact-layer law class

The best current description is now:

- predictive visibility is controlled by parent density together with the size
  and multiplicity of the first visible splitting event

This is stronger than:

- density alone
- arrangement alone
- or density plus a simple `gap_max` correction

## Conservative Conclusion

What is now supported:

- under tight parent-density matching, first-splitting-event multiplicity
  remains much stronger than `gap_max`
- the best current exact-layer law class is
  `density + first-splitting event`

What is not yet supported:

- a closed exact formula for `H_visible_first`
- that no arrangement-sensitive correction will ever matter
- that the matched-family result alone settles the whole exact-layer law

## Next Exact-Layer Step

The next bounded step should be:

1. build one more tiny density-matched family at a different lift
2. keep parent density as tight as possible again
3. test whether first-splitting multiplicity still dominates simple
   arrangement statistics there

That is the cleanest next discrimination step before attempting any more
ambitious exact-law proposal.
