# Prime Transport Arrangement Statistics

## Purpose

This note tests whether a small arrangement-sensitive statistic at the lift
prime explains visible threshold variation better than the current coarse exact
predictors.

The analysis stays entirely inside the exact recursive-system layer and treats

- `H_visible_first(A, W -> pW)`

as the threshold object of interest.

## Arrangement Statistics Tested

For each threshold-table row, let the forbidden residues at the lift prime be
the exact local stencil set.

The following small arrangement-sensitive statistics were computed:

- `gap_multiset`:
  the sorted circular gap multiset between consecutive forbidden residues mod
  `p`
- `gap_min`
- `gap_max`
- `gap_span = p - gap_max`
  which is the covered arc length complementary to the largest empty gap
- `gap_num_distinct`
- `gap_entropy`
- normalized forms:
  `gap_max / p` and `gap_span / p`

Saved arrangement CSV:
  [visible_threshold_arrangement_stats.csv](/Users/adminamn/AI-Research/ai-router/router-research/results/prime_transport_recursive_system/visible_threshold_arrangement_stats.csv)

Updated ranking table:
  [visible_threshold_predictor_scores.csv](/Users/adminamn/AI-Research/ai-router/router-research/results/prime_transport_recursive_system/visible_threshold_predictor_scores.csv)

## Main Comparison

The strongest coarse predictors in the current exact table are:

- `new_prime`: Spearman `0.8125966644585016`
- `parent_phase_length`: Spearman `0.8125966644585016`
- `new_prime_times_tuplet_size`: Spearman `0.7970636785738106`
- `parent_admissible_density`: Spearman `-0.7346753721413163`
- `local_allowed_count`: Spearman `0.7213299617065603`

The strongest arrangement-sensitive statistics are:

- `gap_max`: Spearman `0.5951493353290387`
- `gap_span`: Spearman `0.5388689914244414`

The weaker arrangement-sensitive statistics are:

- `gap_entropy`: Spearman `-0.18994257106087795`
- `gap_num_distinct`: Spearman `0.16960308636006222`
- `gap_max / p`: Spearman `0.12722597852810427`
- `gap_span / p`: Spearman `-0.12722597852810427`
- `gap_min`: Spearman `0.04301084926412601`

## What Is Ruled Out

### 1. Arrangement alone is not the main coarse organizer

No tested arrangement-sensitive statistic beats:

- `new_prime`
- `parent_phase_length`
- `new_prime_times_tuplet_size`
- `parent_admissible_density`
- or `local_allowed_count`

So the current exact table does not support a law driven mainly by arrangement
alone.

### 2. Forbidden-count alone remains too weak

`local_forbidden_count` remains essentially uninformative on the extended table:

- Spearman `-0.009361178696031551`

So arrangement has materially more signal than forbidden-count alone, but it is
not yet dominant.

## What Looks Promising

### 1. Arrangement does add real information beyond forbidden-count

The strongest arrangement-sensitive statistics:

- `gap_max`
- `gap_span`

substantially outperform `local_forbidden_count`.

So some local arrangement information is clearly relevant.

### 2. Largest-gap structure is the most promising simple arrangement signal

Among the tested arrangement statistics, `gap_max` is the strongest:

- Spearman `0.5951493353290387`

This suggests that the size of the largest empty arc between forbidden residues
may be a more useful arrangement feature than entropy, gap count, or minimum
gap.

### 3. Arrangement should probably be used as a secondary factor

The current picture is:

- prime still provides the strongest coarse scale
- arrangement helps refine cases that coarse counts cannot separate

That matches the extension rows where size-4 tuplets with different local
arrangements have visibly different thresholds under the same lift prime.

## Conservative Conclusion

The current exact table supports the following narrowed view:

- arrangement information materially improves on forbidden-count alone
- but no tested arrangement statistic beats `p` as the main coarse organizer
- the best simple arrangement candidates are `gap_max` and `gap_span`

So the remaining plausible law class is:

- a mixed exact-layer law combining lift prime with a small
  arrangement-sensitive stencil statistic

not:

- prime alone
- forbidden-count alone
- or arrangement alone

## Next Exact-Layer Step

The next exact-layer step should be:

1. build one very small follow-on family where `p` is fixed and
   `nu_p(A)` is fixed
2. vary only the local arrangement of forbidden residues
3. test whether `gap_max`, `gap_span`, or a closely related spacing profile
   predicts the residual visible-threshold variation better than parent density

That is the cleanest next discrimination step now that size and forbidden-count
have both been weakened.
