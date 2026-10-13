# Prime Transport Interaction Statistics

## Purpose

This note tests whether a small, interpretable family of interaction-sensitive
statistics between

- the inherited parent return grammar
- and the new local forbidden-residue stencil at the lift prime

explains visible-threshold variation better than the current leading exact-layer
predictors.

The analysis stays entirely inside the exact recursive-system layer. The target
object remains

- `H_visible_first(A, W -> pW)`

with no downstream quotient or compression target.

## Interaction Statistics Tested

For each threshold-table row:

1. build the exact parent admissibility word at the parent wheel
2. compute the cyclic parent return-gap multiset
3. reduce each parent return gap modulo the lift prime `p`
4. compare those residues to the exact forbidden-residue set at `p`

The following small interaction statistics were computed:

- `weighted_gap_forbidden_hit_rate`
  - fraction of parent return-gap mass whose residue mod `p` lands exactly in
    the forbidden set
- `weighted_gap_mean_forbidden_distance`
  - mean circular distance from return-gap residues to the forbidden set,
    weighted by gap multiplicity
- `largest_gap_forbidden_distance`
  - circular distance between the residue of the largest parent return gap and
    the forbidden set
- `most_common_gap_forbidden_distance`
  - circular distance between the residue of the most frequent parent return
    gap and the forbidden set

Saved row table:
  [visible_threshold_interaction_stats.csv](/Users/adminamn/AI-Research/ai-router/router-research/results/prime_transport_recursive_system/visible_threshold_interaction_stats.csv)

Saved ranking table:
  [visible_threshold_interaction_scores.csv](/Users/adminamn/AI-Research/ai-router/router-research/results/prime_transport_recursive_system/visible_threshold_interaction_scores.csv)

## Main Comparison

On the current exact threshold table, the leading previously-established coarse
predictors remain:

- `new_prime`: Spearman `0.8125966644585016`
- `parent_admissible_density`: Spearman `-0.7346753721413163`
- `local_allowed_count`: Spearman `0.7213299617065603`
- `gap_max`: Spearman `0.5951493353290387`

The tested interaction-sensitive statistics score:

- `largest_gap_forbidden_distance`: Spearman `-0.19296925587412428`
- `weighted_gap_mean_forbidden_distance`: Spearman `0.17411086930415728`
- `most_common_gap_forbidden_distance`: Spearman `0.09922117949474808`
- `weighted_gap_forbidden_hit_rate`: Spearman `0.0658119856133263`

## Interpretation

### 1. The tested interaction proxies do not beat the current organizers

None of the small parent-grammar/stencil interaction statistics outperforms:

- `new_prime`
- `parent_admissible_density`
- `local_allowed_count`
- or `gap_max`

So this bounded experiment does **not** support the claim that the visible
threshold is already explained best by these simple interaction proxies.

### 2. The current interaction family is probably too coarse

The interaction scores used here only compare parent return gaps to forbidden
residues through residue-level alignment and circular distance.

That leaves out more structured exact-layer effects such as:

- which parent return classes first split under lift
- whether the largest parent return structures are the ones destabilized first
- how the parent return-gap distribution is partitioned by forbidden-residue
  classes rather than only globally averaged against them

So the negative result is informative, but it does not rule out the broader
law class:

- parent-grammar / new-stencil interaction matters

It only rules out this first small family of global residue-alignment proxies as
the leading explanation.

### 3. Current best reading

The present exact-layer picture is:

- `p` remains the strongest coarse organizer
- parent admissible density remains stronger than the tested simple interaction
  features
- local arrangement matters once `p` and `nu_p(A)` are fixed
- but the current globally averaged interaction scores are too weak to replace
  the coarse predictors

So the best current law class is still:

- coarse scale from the lift prime, refined by parent-grammar and local-stencil
  structure

but the refinement has not yet been captured by the simple interaction
statistics tested here.

## Conservative Conclusion

What is now ruled out:

- that these first simple parent-return/stencil alignment scores are the main
  explanatory law
- that a single global residue-alignment average already captures predictive
  visibility

What remains plausible:

- the visible threshold depends on interaction between inherited parent return
  grammar and the new stencil
- but that interaction likely has to be measured at a more structured level
  than the four coarse summary scores tested here

## Next Exact-Layer Step

The next exact-layer step should stay small and target the first visible split
more directly:

1. keep the current threshold rows
2. identify which parent return-gap classes are split first under each lift
3. measure whether first visible split is better predicted by a
   largest-return-class splitting statistic than by global residue-alignment
   averages

That is the cleanest next probe of the parent-grammar/stencil interaction law
class without leaving the exact recursive-system layer.
