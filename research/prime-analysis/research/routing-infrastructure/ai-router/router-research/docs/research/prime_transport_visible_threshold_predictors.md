# Prime Transport Visible Threshold Predictors

## Purpose

This note analyzes candidate predictors for the exact-layer object

- `H_visible_first(A, W -> pW)`

using the current threshold table as the core dataset.

The framing remains:

- internal split is immediate in all tested cases
- visible split is the actual threshold object
- the main question is the law of predictive visibility, not internal activation

This note does not propose a closed formula. It only narrows which candidate
law classes are plausible and which are already ruled out by the current exact
table.

## Source Data

Primary inputs:

- canonical framework:
  [prime_transport_canonical_framework.md](/Users/adminamn/AI-Research/ai-router/router-research/docs/research/prime_transport_canonical_framework.md)
- threshold-law note:
  [prime_transport_threshold_law.md](/Users/adminamn/AI-Research/ai-router/router-research/docs/research/prime_transport_threshold_law.md)
- threshold summary:
  [threshold_law_summary.csv](/Users/adminamn/AI-Research/ai-router/router-research/results/prime_transport_recursive_system/threshold_law_summary.csv)

Derived analysis artifacts:

- runner:
  [analyze_visible_threshold_predictors.py](/Users/adminamn/AI-Research/ai-router/router-research/tools/prime_transport/analyze_visible_threshold_predictors.py)
- row-level analysis:
  [visible_threshold_predictor_rows.csv](/Users/adminamn/AI-Research/ai-router/router-research/results/prime_transport_recursive_system/visible_threshold_predictor_rows.csv)
- predictor scores:
  [visible_threshold_predictor_scores.csv](/Users/adminamn/AI-Research/ai-router/router-research/results/prime_transport_recursive_system/visible_threshold_predictor_scores.csv)
- monotonicity checks:
  [visible_threshold_monotonicity.csv](/Users/adminamn/AI-Research/ai-router/router-research/results/prime_transport_recursive_system/visible_threshold_monotonicity.csv)

## Candidate Variables Compared

For each threshold-table row, the analysis computes:

- new prime `p`
- tuplet size `|A|`
- local forbidden count `nu_p(A)`
- local allowed count `p - nu_p(A)`
- local allowed fraction `(p - nu_p(A)) / p`
- parent phase length `L_parent = W_parent / 6`
- parent admissible density on the exact parent wheel
- mixed scale `p * |A|`
- mixed scale `p * nu_p(A)`

The current tuplets satisfy `nu_p(A) = |A|` for the tested primes, so those two
counts coincide in the current table.

## What Is Clearly Ruled Out

### 1. `H_visible_first` is not a small correction to internal split

Internal split is always at `H = 1`, but visible split ranges from immediate to
very delayed. So the visible threshold is genuinely nontrivial.

### 2. A law in `p` alone is unsupported

At fixed `p`, thresholds vary strongly by tuplet:

- `p = 19`: visible thresholds are `6`, `26`, and `> 60`

So the new prime alone cannot be the law.

### 3. A law in `|A|` alone is unsupported

At fixed tuplet, thresholds grow materially across lifts:

- twins: `1, 2, 4, 6`
- triplet: `1, 6, 16, 26`
- quadruplet: `1, 21, 51, > 60`

So tuplet size alone also cannot be the law.

### 4. A law in local allowed fraction alone is weak

Among the tested simple variables, local allowed fraction has the weakest rank
association with visible threshold:

- Spearman correlation `0.1103521175692087`

So a law driven mainly by allowed fraction is not supported by the current
table.

## What Appears Most Predictive So Far

Among the coarse candidate predictors tested on the exact visible-threshold
rows, the strongest rank association is:

- `p * |A|` with Spearman `0.8414348964652163`

Since `nu_p(A) = |A|` in the current table, `p * nu_p(A)` ties exactly with the
same score.

Other coarse signals:

- `p`: Spearman `0.7465796629458695`
- `L_parent`: Spearman `0.7465796629458695`
- parent admissible density: Spearman `-0.63452467602295`
- `|A|`: Spearman `0.38971105112598486`

These numbers should be interpreted cautiously because the dataset is still
small and highly structured.

## Coarse Structural Trends Supported

### 1. Monotonicity in the new prime within each tested tuplet

Across exact visible-threshold rows, the visible threshold is nondecreasing in
the new prime within each tuplet:

- monotonicity score: `8 / 8`

The censored quadruplet row at `p = 19` is also consistent with this.

### 2. Monotonicity in tuplet size within each tested prime

Across exact visible-threshold rows, the visible threshold is nondecreasing in
tuplet size within each tested prime:

- monotonicity score: `7 / 7`

Again, the censored quadruplet row at `p = 19` is consistent with the same
ordering.

### 3. Parent admissible density is plausibly relevant

Parent admissible density has a moderate negative association with visible
threshold:

- Spearman `-0.63452467602295`

This is plausible from the exact recursive-system perspective: sparser parent
admissibility may require longer horizon before new-layer distinctions become
visible in compressed predictive states.

But the present table is too small to separate this cleanly from tuplet size and
lift depth.

## Conservative Interpretation

The current exact table supports the following narrowed law class:

- `H_visible_first` is governed by a mixed complexity measure involving both
  the new lift prime and the tuplet’s local stencil burden
- monotonic growth in visible threshold with prime and with tuplet density is
  supported at the coarse table level
- internal split is not the hard part; predictive visibility is

What remains unearned:

- any closed-form formula
- any theorem in `p` alone
- any theorem in `|A|` alone
- any claim that one of the tested mixed scales is the true law

At this stage, `p * |A|` should be treated only as a useful candidate law class
anchor, not as a discovered formula.

## Next Exact-Layer Step

The next preferred exact-layer step should be:

1. extend the exact threshold table with a few additional admissible tuplets of
   different stencil density
2. retain the same internal/visible split definitions
3. test whether `H_visible_first` is better organized by:
   - `p * |A|`
   - `p * nu_p(A)`
   - parent admissible density
   - or a composite lower-bound class using both lift prime and local stencil
     burden

That is the minimal next step that can discriminate between currently plausible
exact-layer law classes without drifting into quotient geometry or overfitting.
