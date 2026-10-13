# Prime Transport First Splitting Classes

## Purpose

This note moves from global interaction averages to the exact event level.

The question is:

- which parent predictive classes are first split when visible threshold is
  reached?

and whether statistics built from those first splitting classes explain

- `H_visible_first(A, W -> pW)`

better than the previously-tested global interaction summaries.

The analysis remains entirely inside the exact recursive-system layer.

## Exact Event Definition

For a row with exact visible threshold

- `H_visible_first = H* > 1`

define the pre-threshold parent predictive classes at horizon `H* - 1` by

- `(b, spin_{H*-1})`

where `spin_{H*-1}` is the exact finite-horizon future word at the parent
wheel.

A parent class is called a **first splitting class** if, under the lift to the
child wheel, its descendants produce more than one distinct child predictive
class at horizon `H*`.

So this is the exact event that first makes the compressed predictive state
count visibly larger at the child wheel.

Saved first-splitting-class table:
  [visible_threshold_first_splitting_classes.csv](/Users/adminamn/AI-Research/ai-router/router-research/results/prime_transport_recursive_system/visible_threshold_first_splitting_classes.csv)

This class-level table is restricted to the stronger nontrivial exact rows with

- `H_visible_first >= 13`

so that the detailed split events stay interpretable.

Saved event-level row table:
  [visible_threshold_first_splitting_event_stats.csv](/Users/adminamn/AI-Research/ai-router/router-research/results/prime_transport_recursive_system/visible_threshold_first_splitting_event_stats.csv)

Saved ranking table:
  [visible_threshold_first_splitting_scores.csv](/Users/adminamn/AI-Research/ai-router/router-research/results/prime_transport_recursive_system/visible_threshold_first_splitting_scores.csv)

## Event Statistics Tested

The event-based statistics were kept local and interpretable:

- `num_first_splitting_classes`
- `fraction_first_splitting_classes`
- `fraction_parent_mass_in_first_splitting_classes`
- `max_first_splitting_class_size`
- `mean_first_splitting_class_size`
- `max_child_classes_from_first_split`
- `mean_child_classes_from_first_split`
- `total_extra_child_classes_from_first_split`
- `fraction_unsplit_parent_classes_pre_threshold`

These were compared against the previously-used controls:

- `new_prime`
- parent admissible density
- `local_allowed_count`
- `gap_max`
- `gap_entropy`

## Main Result

The strongest event-based statistics are:

- `num_first_splitting_classes`: Spearman `0.820410093133918`
- `total_extra_child_classes_from_first_split`: Spearman `0.7982368473735418`
- `fraction_first_splitting_classes`: Spearman `0.7464992739326641`
- `fraction_parent_mass_in_first_splitting_classes`: Spearman `0.7036309987959369`

These are much stronger than the previous global interaction averages:

- `weighted_gap_mean_forbidden_distance`: Spearman `0.17411086930415728`
- `most_common_gap_forbidden_distance`: Spearman `0.09922117949474808`
- `weighted_gap_forbidden_hit_rate`: Spearman `0.0658119856133263`
- `largest_gap_forbidden_distance`: Spearman `-0.19296925587412428`

The strongest control on the current exact event table is still:

- parent admissible density: Spearman `-0.8593693199561495`

So the first-splitting-event statistics materially outperform the weak global
interaction summaries, but they do not yet displace parent density as the
single strongest coarse predictor.

## What The First Split Looks Like

The first visible split is generally not a tiny exceptional event.

Representative rows:

- quadruplet, `30030 -> 510510`, `H_visible_first = 51`
  - pre-threshold classes: `665`
  - first splitting classes: `633`
  - split-class mass fraction: `0.9768231768231769`
  - max child classes from one first splitter: `10`

- `double_twins_p13`, `510510 -> 9699690`, `H_visible_first = 51`
  - pre-threshold classes: `1655`
  - first splitting classes: `1624`
  - split-class mass fraction: `0.9826291355703121`
  - max child classes from one first splitter: `19`

- triplet, `2310 -> 30030`, `H_visible_first = 6`
  - pre-threshold classes: `85`
  - first splitting classes: `58`
  - split-class mass fraction: `0.7532467532467533`
  - max child classes from one first splitter: `7`

So at the visible threshold, the decisive event usually looks like a broad
release of many unresolved parent predictive classes, not a single isolated
rare class.

## Interpretation

### 1. First splitting events are a better target than global interaction averages

The event-based statistics now clearly improve on the earlier global
parent-return/stencil interaction summaries.

So the threshold looks more like a **first-splitting-event** object than a law
captured by global residue-alignment averages.

### 2. The threshold still does not collapse to one finished law

Even on this better event-level framing, parent admissible density remains
slightly stronger than the tested first-splitting statistics.

So the current evidence supports:

- first-splitting-event structure is part of the correct law class

but does not yet support:

- a complete exact law in one event statistic alone

### 3. Current best reading

The visible threshold now looks most plausibly governed by:

- coarse lift scale
- inherited parent grammar density
- and the size / multiplicity of the first splitting event

rather than by local stencil shape alone or by global interaction averages.

## Conservative Conclusion

What is now supported:

- the threshold is better organized by first splitting event structure than by
  the weak global interaction averages tested earlier
- the decisive event is usually distributed across many pre-threshold parent
  classes, not a single rare outlier class

What is not yet supported:

- a finished exact formula for `H_visible_first`
- that any one first-splitting statistic already replaces parent density as the
  dominant predictor

## Next Exact-Layer Step

The next exact-layer step should stay bounded and discriminate the event-law
class more sharply:

1. hold `p` fixed where possible
2. compare rows with similar parent density but different first-splitting
   multiplicity
3. test whether first-splitting statistics still explain residual variation
   after conditioning on parent density

That is the cleanest next step if the goal is to decide whether predictive
visibility is fundamentally a first-splitting-event law.
