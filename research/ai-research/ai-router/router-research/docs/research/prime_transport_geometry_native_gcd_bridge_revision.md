# Prime Transport Geometry-Native GCD Bridge Revision

## Purpose

Test whether a bounded GCD-style factor-overlap bridge-revision rule can
outperform the current stronger-mismatch recovery line on the v12 setting.

This step keeps the same stronger reduced-alignment family, the same tiny
bridge family, and the same unchanged geometry-native engine. The only change
is how bridge revision is performed once contradictory evidence appears.

## Mechanism

The chosen mechanism is symbolic factor-overlap revision over prime-coded
bridge states.

For each token:

- represent the current bridge and an alternative bridge as semiprimes:
  `proxy_prime * partner_prime`
- use a GCD-style overlap operator to identify the shared factor
- if the overlap preserves the whole current bridge, retain it
- otherwise retain the shared proxy factor and replace only the conflicting
  partner factor

Operationally:

1. choose an initial bridge the same way as the earlier calibrated line
2. look for one disagreement-rich local window where an alternative bridge
   scores better
3. revise the suffix by retaining shared factors and replacing only the
   contradicted partner factor

This is distinct from v14/v15 because it does not switch whole bridge variants
for the suffix. It performs factor-level revision inside the bridge object.

## Comparison

Stronger reduced-alignment line:

- v11 fixed bridge:
  - geometry-native accuracy `0.883501827717`
  - geometry-native query accuracy `0.969032287598`
- v12 fixed stronger-mismatch bridge:
  - geometry-native accuracy `0.552269339561`
  - geometry-native query accuracy `0.584205031395`
- v13 per-sequence bridge calibration:
  - geometry-native accuracy `0.593377947807`
  - geometry-native query accuracy `0.633928596973`
- v14 midpoint bridge switch:
  - geometry-native accuracy `0.606119811535`
  - geometry-native query accuracy `0.660416662693`
- v15 coherence-triggered switch:
  - geometry-native accuracy `0.591052830219`
  - geometry-native query accuracy `0.637893617153`
- v16 GCD-style factor-overlap revision:
  - geometry-native accuracy `0.592540919781`
  - geometry-native query accuracy `0.651993811131`
  - geometry-native test loss `5.578721046448`
- v16 tiny transformer baseline:
  - accuracy `0.457589298487`
  - query accuracy `0.469704806805`
  - test loss `2.272322416306`

## Interpretation

The result is mixed.

What improved:

- v16 still beats the fixed v12 bridge
- v16 still beats the tiny transformer baseline
- v16 improves on v15 in both overall transfer accuracy and query accuracy

What did not improve:

- v16 does not beat the current best v14 midpoint switch
- v16 is also slightly below v13 on overall transfer accuracy, even though its
  query accuracy is higher than v13

So the honest read is that factor-overlap bridge revision changes the recovery
profile, but it does not become the new best stronger-mismatch result.

The likely meaning is:

- arithmetic retained-vs-replaced factor logic is useful
- but one coarse window-level contradiction event plus one-step partner
  replacement is not yet enough to outperform a simpler midpoint recalibration

## Answers

### Does GCD-style bridge revision improve on the current best stronger-mismatch recovery?

No.

It does not beat v14:

- v14 accuracy `0.606119811535` vs v16 accuracy `0.592540919781`
- v14 query accuracy `0.660416662693` vs v16 query accuracy `0.651993811131`

### Does it remain geometry-native in a meaningful sense?

Yes.

The mechanism stays entirely inside prime-coded bridge objects, semiprime
composition, and factor-overlap revision. No generic learned adapter or
transformer block is introduced.

### Does this support the idea that adaptive arithmetic geometry needs factor-level revision, not just bridge-level switching?

Partially.

It supports factor-level revision as a meaningful architectural direction,
because the result remains competitive and improves on some weaker adaptive
variants. But it does not establish factor-level revision as better than the
best current switching method on this setting.

### What is the next smallest stronger test if v16 works?

The next smallest stronger test should keep the same factor-overlap idea but
move from one-step suffix revision to one local revision window with explicit
query/binding conflict markers, so revision is triggered by arithmetic
contradiction structure rather than generic window disagreement alone.

## Bottom Line

GCD-style factor-overlap bridge revision is a meaningful arithmetic adaptation
mechanism, but it is not the new best result on the stronger mismatch setting.
It supports the factor-level revision idea directionally, while leaving the
current best stronger-mismatch recovery with the simpler v14 midpoint switch.
