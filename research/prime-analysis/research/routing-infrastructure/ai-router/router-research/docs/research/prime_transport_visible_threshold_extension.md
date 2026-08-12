# Prime Transport Visible Threshold Extension

## Purpose

This note records the small exact-layer extension added to discriminate between
candidate law classes for

- `H_visible_first(A, W -> pW)`

The extension stays entirely inside the recursive admissibility framework and is
designed specifically to break the earlier degeneracy between:

- tuplet size `|A|`
- local forbidden-count / stencil burden `nu_p(A)`

## Added Tuplets

Three additional admissible tuplets were added:

- `double_twins_p13 = [0,2,26,28]`
- `double_twins_p17 = [0,2,34,36]`
- `double_twins_p19 = [0,2,38,40]`

These were chosen because each is a size-4 tuplet, but for one targeted tested
prime the local forbidden count collapses:

- `nu_13([0,2,26,28]) = 2`
- `nu_17([0,2,34,36]) = 2`
- `nu_19([0,2,38,40]) = 2`

while the ordinary quadruplet `[0,2,6,8]` keeps `nu_p(A) = 4` across the same
tested primes.

So this extension cleanly separates:

- same raw tuplet size
- different local stencil burden at the lift prime

## Updated Threshold Table Highlights

Updated summary:
  [threshold_law_summary.csv](/Users/adminamn/AI-Research/ai-router/router-research/results/prime_transport_recursive_system/threshold_law_summary.csv)

Examples showing the new discrimination:

- at `p = 13`:
  - ordinary quadruplet `[0,2,6,8]` has visible threshold `21`
  - `double_twins_p13 [0,2,26,28]` has visible threshold `6`
- at `p = 17`:
  - ordinary quadruplet `[0,2,6,8]` has visible threshold `51`
  - `double_twins_p17 [0,2,34,36]` has visible threshold `41`
  - `double_twins_p13 [0,2,26,28]` has visible threshold `16`
- at `p = 19`:
  - ordinary quadruplet `[0,2,6,8]` is still censored at `H <= 60`
  - `double_twins_p19 [0,2,38,40]` has visible threshold `34`
  - `double_twins_p13 [0,2,26,28]` has visible threshold `51`
  - `double_twins_p17 [0,2,34,36]` is also censored at `H <= 60`

Internal split remains immediate in every tested row.

## What Is Now Ruled Out More Strongly

### 1. Tuplet size alone is ruled out more decisively

We now have multiple size-4 tuplets with very different visible thresholds under
the same lift prime.

For example at `p = 19`:

- `[0,2,6,8]` gives `> 60`
- `[0,2,38,40]` gives `34`

So raw size `|A| = 4` clearly does not determine the threshold.

### 2. A law in `p * |A|` alone is no longer the best coarse organizer

Before the extension, `p * |A|` was a plausible leading coarse candidate. After
the extension, that candidate weakens because equal-size tuplets separate.

Updated score table:
  [visible_threshold_predictor_scores.csv](/Users/adminamn/AI-Research/ai-router/router-research/results/prime_transport_recursive_system/visible_threshold_predictor_scores.csv)

Top coarse rank signal is now:

- `new_prime` with Spearman `0.839002048918404`

while:

- `new_prime_times_tuplet_size` falls to Spearman `0.8169044933264241`

So the extension successfully breaks the earlier appearance that `p * |A|` was
the best coarse organizer.

### 3. `nu_p(A)` alone is also not enough

The newly added size-4 collision tuplets have `nu_p(A) = 2` at their targeted
prime, but their visible thresholds still differ across lifts and from twins.

So local forbidden count alone does not appear to determine the threshold
either.

## What Remains Plausible

The current exact table is most consistent with:

- visible threshold growing with lift prime in a coarse monotone sense
- visible threshold depending materially on local stencil structure
- visible threshold also depending on more than the single scalar `nu_p(A)`

The updated monotonicity table supports:

- within each tested tuplet, visible threshold is nondecreasing in the new
  prime: `27 / 27`
- within each tested prime, visible threshold is usually but not perfectly
  nondecreasing in raw tuplet size: `26 / 29`

This is exactly what the extension was meant to show:

- prime matters
- stencil structure matters
- raw size is too crude

## Conservative Conclusion

After the extension:

- laws in `|A|` alone are clearly ruled out
- laws in `p * |A|` alone are weakened and should no longer be treated as the
  leading candidate
- laws in `nu_p(A)` alone are also not sufficient
- the remaining plausible law class is a mixed exact-layer law involving:
  - lift prime
  - local stencil burden
  - and finer stencil arrangement information beyond a single count

This is still not a formula. It is only a narrowing of candidate law classes.

## Next Exact-Layer Step

The next exact-layer step should be:

1. add a very small second extension family whose tuplets have the same
   `nu_p(A)` at a tested prime but different local arrangement pattern
2. keep the same threshold definitions
3. test whether visible threshold is better organized by:
   - `p`
   - `nu_p(A)`
   - parent admissible density
   - or a small arrangement-sensitive stencil statistic

The main new question is no longer “size or count?” but:

- which exact local arrangement statistics matter beyond forbidden-count alone?
