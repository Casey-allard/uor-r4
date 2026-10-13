# Prime Transport Density-Conditioned First Splitting

## Purpose

This note tests whether first-splitting-event multiplicity still organizes
visible threshold after conditioning coarsely on parent admissible density.

The motivation is straightforward:

- parent admissible density is currently the strongest single coarse control
- first-splitting-event statistics are the strongest event-based candidates
- so the next exact-layer question is whether those event statistics still
  matter once density is held roughly fixed

The analysis remains entirely inside the exact recursive-system layer.

## Grouping Scheme

The current exact event rows were partitioned into three simple density bands:

- `low_density_le_0p08`
- `mid_density_0p08_to_0p18`
- `high_density_gt_0p18`

This keeps the conditioning interpretable and avoids fitting narrow ad hoc bins.
No new rows were added for this step.

Saved conditioned row table:
  [visible_threshold_density_conditioned_rows.csv](/Users/adminamn/AI-Research/ai-router/router-research/results/prime_transport_recursive_system/visible_threshold_density_conditioned_rows.csv)

Saved band summary table:
  [visible_threshold_density_conditioned_band_scores.csv](/Users/adminamn/AI-Research/ai-router/router-research/results/prime_transport_recursive_system/visible_threshold_density_conditioned_band_scores.csv)

Saved residual ranking table:
  [visible_threshold_density_conditioned_residual_scores.csv](/Users/adminamn/AI-Research/ai-router/router-research/results/prime_transport_recursive_system/visible_threshold_density_conditioned_residual_scores.csv)

## Event Statistics Tested

Within the density-conditioned comparison, the main event-based predictors were:

- `num_first_splitting_classes`
- `total_extra_child_classes_from_first_split`
- `fraction_first_splitting_classes`

The conditioned comparison was done by subtracting each density band's mean
visible threshold and each band's mean predictor value, then comparing the
within-band residuals.

## Main Result

The density-conditioned residual ranking is:

- `num_first_splitting_classes_residual`: Spearman `0.8144338729465652`
- `total_extra_child_classes_residual`: Spearman `0.7879242712955016`
- `gap_max_residual`: Spearman `0.7841852240636016`
- `fraction_first_splitting_classes_residual`: Spearman `0.7776149817645324`
- `new_prime_residual`: Spearman `0.6393790552461432`

So after coarse conditioning on parent density, first-splitting multiplicity
still carries strong association with visible-threshold variation.

## Band-Level Reading

The bands are small, so they should be treated as descriptive rather than
decisive. Still, the within-band associations are consistent:

- low-density band (`9` rows):
  - visible vs `num_first_splitting_classes`: Spearman `0.7815402003388966`
  - visible vs `total_extra_child_classes`: Spearman `0.7815402003388966`
  - visible vs `fraction_first_splitting_classes`: Spearman `0.7395219099980958`

- mid-density band (`4` rows):
  - all three event statistics give perfect rank order on this tiny sample

- high-density band (`3` rows):
  - all three event statistics also give perfect rank order on this tiny sample

The tiny `4`-row and `3`-row bands are too small to support a strong standalone
claim, but they do not contradict the pooled conditioned result.

## Interpretation

### 1. First-splitting multiplicity still matters after density conditioning

This is the main new result.

Parent density is not the whole story. Once density is held roughly fixed,
event-based quantities tied to the first visible split still organize the
remaining variation well.

### 2. The current best exact-layer law class is now clearer

The visible threshold is best described, at present, as an interaction between:

- coarse parent-grammar density
- and first-splitting-event multiplicity

That is a stronger and more specific law class than:

- prime alone
- local stencil size alone
- forbidden-count alone
- arrangement alone
- or global residue-alignment averages

### 3. Still no finished exact formula

The current table is still too small for a serious exact formula claim.

What is now supported is not a closed law, but a narrowed explanatory class:

- predictive visibility is controlled by density together with the size and
  multiplicity of the first visible splitting event

## Conservative Conclusion

What is now supported:

- first-splitting multiplicity still matters after coarse density conditioning
- visible threshold is better thought of as a density-plus-event law than as a
  density-only law

What is not yet supported:

- a closed formula for `H_visible_first`
- a claim that any one first-splitting statistic is the exact law by itself

## Next Exact-Layer Step

The cleanest next step is:

1. add only a few exact rows chosen to sharpen density-controlled contrasts
2. keep parent density roughly matched
3. vary first-splitting multiplicity and split mass as much as possible
4. test whether the event statistics remain stronger than `gap_max` under that
   tighter conditioning

That is the minimal next step needed to decide whether predictive visibility is
best described by a density-plus-first-splitting-event law.
