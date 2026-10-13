# Prime Transport Phase-Fiber Readability

## Question

Can first visible splitting be detected or approximated directly in the exact
phase-fiber-scale coordinates `j -> (b, phi, r)`, without reconstructing the
full finite-horizon spin state first?

This is an exact-layer question. It does not treat downstream quotient objects
as primary.

## Setup

The bounded experiment used the already-constructed tightly density-matched
families where visible threshold and first-splitting behavior are known exactly:

- `2310 -> 30030`
- `30030 -> 510510`

For each row:

1. Take the known first visible split horizon `H_visible_first`.
2. Form the parent predictive classes at horizon `H_visible_first - 1`.
3. Identify the parent classes whose child lifts first branch into multiple
   child predictive classes at `H_visible_first`.
4. Map the parent positions belonging to those first-splitting classes back to
   phase-fiber coordinates using the currently implemented exact chart:
   - `b = j mod 35`
   - `t = floor(j / 35)`
   - `phi = (t mod q_1, ..., t mod q_r)` from the prime-factor decomposition of
     the fiber modulus

The tested phase-fiber features were intentionally small and interpretable:

- concentration of first splitters in a dominant `phi` tuple
- concentration of first splitters in a dominant base phase `b`
- support fraction of `phi` tuples represented among first splitters
- support fraction of base phases represented among first splitters
- a simple boundary/fringe flag on `phi`

## Result

The mapped first splitters are not distributed uniformly in phase-fiber space.
The strongest simple phase-fiber signals on the bounded matched-family table
were:

- `top_split_phi_tuple_share`: Spearman `-0.9817255676437577`
- `top_split_base_share`: Spearman `-0.9817255676437577`
- `split_base_support_fraction`: Spearman `0.8139881369630423`

The boundary/fringe heuristic was weak:

- `boundary_enrichment_ratio`: Spearman `-0.18293022999573125`

For comparison, the already-established full-spin event control remains
stronger:

- `num_first_splitting_classes`: Spearman `0.9939209163101398`

So the current evidence supports the following interpretation:

- first visible splitting is partially readable in `(b, phi, r)`
- the strongest simple readability signal is concentration versus dispersion of
  first splitters across `phi` tuples and base phases
- but these simple phase-fiber features do not yet replace the full spin-side
  event description

## Interpretation

This suggests that the first visible split is not arbitrary with respect to the
exact phase-fiber chart. The splitting classes appear to condense into specific
phase-fiber sectors before visible predictive refinement occurs.

However, the present result does not justify the stronger claim that
phase-fiber coordinates alone already determine the first-splitting event by a
simple local rule. The weak performance of the boundary/fringe statistic is a
warning against overreading the chart geometrically.

The conservative conclusion is:

- the exact phase-fiber chart carries real first-splitting signal
- that signal is currently best described as partial readability
- full finite-horizon spin is still the more reliable exact predictor of the
  first-splitting event on the current evidence

## Status

This should be treated as a bounded exact-layer refinement of the canonical
threshold-law picture:

- visible threshold is governed by `density + first-splitting event`
- first-splitting events are not fully opaque in `(b, phi, r)`
- but no simple phase-fiber rule has yet displaced the spin-based event view
