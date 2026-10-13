# Prime Transport Minimal Predictive State

## Question

What is the smallest exact predictive state that:

- strictly improves on the static chart as a practical predictive abstraction
- remains meaningfully smaller than full spin
- stays structurally interpretable as a routing state

## Candidate State

The minimal state is taken in the form

- `S_min = (b, phi, r, residual_small)`

with `residual_small` restricted to already validated exact residual objects.

For practical predictive use, the best current choice is:

- `residual_small = next_return_gap`

So the current candidate is:

- `S_min = (b, phi, r, next_return_gap)`

The reason for this choice is not that it is the strongest raw predictor. It is
chosen because it is the smallest candidate that still captures a substantial
part of the predictive structure while remaining genuinely smaller than spin.

## Routing Interpretation

`S_min` should be interpreted as an exact routing state in the following
precise sense.

It partitions the orbit by:

- static location in the exact chart `(b, phi, r)`
- immediate forward return structure through `next_return_gap`

Its transition is induced by the odometer update `j -> j + 1`, which updates:

- the base phase `b`
- the fiber coordinate `phi`
- the next admissible-return distance

Its relation to first visible splitting is:

- states with the same static chart position class but different immediate
  return structure are candidates for different downstream predictive branch
  behavior
- so `next_return_gap` supplies the smallest currently validated dynamical tag
  that improves practical predictive discrimination beyond static chart reading

## Comparison

The exact comparison against the current reference states is:

- chart exact address `(b, phi, r)`:
  - membership purity `1.0`
  - split purity `1.0`
  - label ratio to spin `7.273714418110126`
  - interpretation: exact address, but far too detailed to count as a useful
    compressed predictive state

- `next_return_gap`:
  - membership purity `1.0`
  - split purity `0.69033629362971`
  - combined capture fraction of spin `0.7502981320146836`
  - label ratio to spin `0.6121906310452204`

- `prev_next_gap_pair`:
  - membership purity `1.0`
  - split purity `0.7699750264223661`
  - combined capture fraction of spin `0.8118207348148488`
  - label ratio to spin `1.539815778208287`
  - interpretation: stronger, but no longer smaller than spin

- full spin:
  - membership purity `1.0`
  - split purity `1.0`
  - label ratio `1.0`

So the practical tradeoff is:

- `next_return_gap` is the smallest genuine compressed state among the tested
  exact residual objects
- `prev_next_gap_pair` captures more but fails the “meaningfully smaller than
  spin” requirement

## Threshold-Law Reading

The current canonical law class is:

- `density + first-splitting event`

In `S_min`, this becomes:

- static part:
  chart location and inherited density context through `(b, phi, r)`
- dynamic part:
  immediate return-memory through `next_return_gap`

So `S_min` does not replace the full first-splitting event structure, but it
does provide a minimal exact state that approximates the dynamic side of the
threshold law with a small interpretable residual.

## Failure Modes

`S_min` diverges from spin mainly in the deeper-lift cases where:

- multiple distinct predictive branches share the same immediate return gap
- visible splitting depends on richer return-grammar structure than one-step
  forward return distance can encode

That is why its split purity stays at about `0.69` rather than approaching
full spin.

## Classification

The present result should be classified as:

- **(B) `S_min` is useful but materially weaker than spin**

This is the strongest conservative reading because:

- it clearly improves practical predictive structure beyond a weak baseline
- it remains genuinely smaller than spin
- but it still leaves a substantial predictive gap on first-splitting
  structure, especially at deeper lifts
