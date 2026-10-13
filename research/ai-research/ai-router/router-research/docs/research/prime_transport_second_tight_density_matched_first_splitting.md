# Prime Transport Second Tight Density-Matched First Splitting

## Purpose

This note replicates the tight density-matched first-splitting comparison on a
second exact lift family.

The specific question is whether the law class

- `density + first-splitting event`

still outperforms simple arrangement statistics when parent admissible density
is tightly controlled on a different lift.

The analysis remains entirely inside the exact recursive-system layer.

## Matched Family Used

The second matched slice uses:

- lift `30030 -> 510510`
- new prime `p = 17`

To keep the extension small, only four size-4 tuplets were used, all with
exactly the same parent admissible density:

- `0.03776223776223776`

Rows used:

- `quad_alt_0406 = [0,2,4,6]`
- `double_twins_p17 = [0,2,34,36]`
- `quad_alt_0618 = [0,2,6,18]`
- `quadruplet = [0,2,6,8]`

Saved matched rows:
  [visible_threshold_second_tight_density_matched_rows.csv](/Users/adminamn/AI-Research/ai-router/router-research/results/prime_transport_recursive_system/visible_threshold_second_tight_density_matched_rows.csv)

Saved matched ranking table:
  [visible_threshold_second_tight_density_matched_scores.csv](/Users/adminamn/AI-Research/ai-router/router-research/results/prime_transport_recursive_system/visible_threshold_second_tight_density_matched_scores.csv)

## Main Matched Result

Within this exact matched slice:

- visible thresholds are `36`, `41`, `46`, `51`
- `num_first_splitting_classes` rises `223 -> 368 -> 538 -> 633`
- `total_extra_child_classes_from_first_split` rises `467 -> 792 -> 1805 -> 1879`

The matched-family ranking is:

- `num_first_splitting_classes`: Spearman `1.0`
- `total_extra_child_classes_from_first_split`: Spearman `1.0`
- `fraction_first_splitting_classes`: Spearman `1.0`
- `gap_max`: Spearman `0.31622776601683794`
- `gap_span`: Spearman `-0.31622776601683794`

So the exact matched ordering is again carried by first-splitting multiplicity,
not by the simple arrangement statistics.

## Interpretation

### 1. The tight matched result now replicates across two lift families

The first matched family on `2310 -> 30030` already favored

- `density + first-splitting event`

over a simple `gap_max` correction.

This second matched family on `30030 -> 510510` shows the same pattern more
strongly.

So the current evidence now supports that law class across more than one exact
matched slice.

### 2. Arrangement remains secondary on the matched evidence so far

The matched rows do vary in arrangement:

- `gap_max` takes values `6`, `10`, `11`

but that variation does not organize the thresholds nearly as strongly as the
first-splitting statistics do.

So the current exact evidence still does not justify promoting `gap_max` to the
leading correction term.

## Conservative Conclusion

What is now supported:

- across two tightly density-matched exact families, first-splitting
  multiplicity remains the stronger organizer
- the best current exact-layer law class is robustly
  `density + first-splitting event`

What is not yet supported:

- a closed exact formula for `H_visible_first`
- a claim that arrangement never matters
- a claim that the present matched slices are already sufficient to settle the
  full recursive-system law

## Next Exact-Layer Step

The next bounded step should be:

1. add one final tiny matched family if it can be built cleanly on the `19`
   lift
2. if that is too expensive or too censored, stop extension and formalize the
   current law class as the stable working hypothesis
3. then shift from discrimination to cautious bound-form proposals for
   predictive visibility
