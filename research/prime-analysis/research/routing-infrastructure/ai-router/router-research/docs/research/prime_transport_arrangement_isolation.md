# Prime Transport Arrangement Isolation

## Purpose

This note isolates the effect of local forbidden-residue arrangement on the
visible threshold by holding

- lift prime `p` fixed
- forbidden-count `nu_p(A)` fixed

and varying only the local arrangement pattern.

The analysis remains entirely inside the exact recursive-system layer.

## Fixed Families Used

Two exact families were selected because they are clean and fully observed:

- `p = 13`, `nu_p(A) = 4`
- `p = 17`, `nu_p(A) = 4`

Each family contains three size-4 tuplets with distinct forbidden-residue
arrangements at the lift prime:

- ordinary quadruplet `[0,2,6,8]`
- two collision tuplets from the small extension set

Saved rows:
  [visible_threshold_arrangement_isolation_rows.csv](/Users/adminamn/AI-Research/ai-router/router-research/results/prime_transport_recursive_system/visible_threshold_arrangement_isolation_rows.csv)

Saved scores:
  [visible_threshold_arrangement_isolation_scores.csv](/Users/adminamn/AI-Research/ai-router/router-research/results/prime_transport_recursive_system/visible_threshold_arrangement_isolation_scores.csv)

## Arrangement Statistics Compared

Using the same exact forbidden residues at the lift prime, the following
statistics were compared on the fixed-family rows:

- `gap_max`
- `gap_second_max`
- `gap_span`
- `gap_num_distinct`
- `gap_entropy`

As a control, the same table also includes:

- parent admissible density

The response variable is the family-centered residual visible threshold:

- `H_visible_first - mean_family(H_visible_first)`

So the comparison asks which statistic explains residual variation after `p` and
`nu_p(A)` have already been fixed.

## Fixed-Family Rows

For `p = 13`, `nu_p(A) = 4`:

- `double_twins_p17`: `H_visible_first = 21`
  - gaps `1 3 3 6`
- `double_twins_p19`: `H_visible_first = 3`
  - gaps `2 2 2 7`
- `quadruplet`: `H_visible_first = 21`
  - gaps `1 1 3 8`

For `p = 17`, `nu_p(A) = 4`:

- `double_twins_p13`: `H_visible_first = 16`
  - gaps `1 4 6 6`
- `double_twins_p19`: `H_visible_first = 13`
  - gaps `1 5 5 6`
- `quadruplet`: `H_visible_first = 51`
  - gaps `1 1 5 10`

So arrangement is genuinely varying while `p` and `nu_p(A)` are held fixed.

## Main Result

Residual predictor ranking on the fixed families:

- `parent_admissible_density`: Spearman `-0.9411764705882353`
- `gap_entropy`: Spearman `-0.7826908981308054`
- `gap_max`: Spearman `0.6160411036336975`
- `gap_num_distinct`: Spearman `0.3985266984930429`
- `gap_span`: Spearman `-0.37317589626658254`
- `gap_second_max`: Spearman `0.0298540717013266`

## Interpretation

### 1. Arrangement is a real secondary driver

Because `p` and `nu_p(A)` are fixed inside each family, the observed threshold
variation cannot be attributed to those two quantities alone.

So the fixed-family rows directly support:

- arrangement matters

in the exact recursive system.

### 2. But no simple arrangement statistic dominates yet

The strongest arrangement-sensitive signal in this fixed-family analysis is:

- `gap_entropy`

followed by:

- `gap_max`

However, both still trail the control variable:

- parent admissible density

So the current data does **not** justify the claim that a single simple
arrangement statistic is already the main residual law.

### 3. `gap_max` remains the cleanest simple geometric candidate

Although `gap_entropy` scores better in this tiny residual table, `gap_max` is
still the clearest simple statistic to interpret:

- larger largest empty arc tends to accompany larger visible threshold in the
  current fixed families

`gap_entropy` may be useful, but it is less structurally transparent.

## Conservative Conclusion

What is now supported:

- arrangement has been isolated as a real secondary driver of visible threshold
- `p` and `nu_p(A)` do not exhaust the exact-layer story

What is not yet supported:

- that arrangement alone is the dominant residual law
- that any one simple arrangement statistic is already the correct formula

Current best simple arrangement candidates:

- `gap_entropy`
- `gap_max`

with `gap_max` the cleaner explanatory candidate and `gap_entropy` the stronger
raw residual correlator on this tiny family.

## Next Exact-Layer Step

The next exact-layer step should be:

1. keep `p` fixed and `nu_p(A)` fixed
2. add one more very small arrangement-variation family
3. test whether `gap_max` or a closely related largest-gap profile remains
   stable as the simplest arrangement-sensitive predictor once parent-density
   effects are better separated

That is the minimal next step needed before claiming any arrangement-sensitive
law class beyond coarse plausibility.
