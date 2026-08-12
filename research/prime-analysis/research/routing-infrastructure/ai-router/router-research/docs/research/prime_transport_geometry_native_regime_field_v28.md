# Prime Transport Geometry-Native Regime Field V28

## Purpose

Test whether a slightly richer learned regional regime field can recover more
of the lost v26 margin on the stronger shifted family while keeping the
geometry-native computation core intact.

## Mechanism

The chosen mechanism is a slightly richer but still small learned boundary
field over the same bounded candidate split set used in v27.

It keeps:

1. the same stronger shifted family as v27
2. the same bounded split candidates
3. the same local geometry-native schema/bridge machinery

The only change is a slightly richer scorer:

- wider hidden projection
- one additional learned mixing stage
- still no transformer blocks
- still no generic sequence front-end

## Comparison

Reference points:

- best handcrafted hybrid result on earlier family, v24:
  - geometry-native accuracy `0.649925589561`
  - geometry-native query accuracy `0.719008266926`
- learned regional-field result on earlier family, v26:
  - geometry-native accuracy `0.654854893684`
  - geometry-native query accuracy `0.732780933380`

Stronger shifted family results:

- v27 tiny learned field:
  - geometry-native accuracy `0.540852844715`
  - geometry-native query accuracy `0.584938704967`
  - parameter count `6516`
- v28 slightly richer learned field:
  - geometry-native accuracy `0.548014342785`
  - geometry-native query accuracy `0.584723412991`
  - geometry-native test loss `6.508901119232`
  - parameter count `7316`
  - effective state size `15`
- v28 tiny transformer baseline:
  - accuracy `0.369710296392`
  - query accuracy `0.438542574644`
  - test loss `2.805939912796`

## Interpretation

This is only a partial recovery.

What improved:

- v28 recovers some overall transfer accuracy over v27
- v28 still beats the tiny transformer baseline clearly

What did not improve:

- v28 does not recover query accuracy; it is slightly worse than v27 on that
  metric
- the richer field does not come close to restoring the earlier-family v26
  margin

Numerically relative to v27:

- overall-accuracy gain: `0.007161498070`
- query-accuracy change: `-0.000215291977`
- parameter increase: `800`

So the honest reading is that a slightly richer scorer helps a bit on overall
regional modeling, but it does not fix the main robustness problem. That
suggests the next bottleneck is likely field structure or representation, not
just a small increase in scorer capacity.

## Answers

### Does v28 materially improve on v27?

Only partially.

It improves overall accuracy modestly, but it does not improve query
accuracy.

### Does it remain geometry-native in a meaningful sense?

Yes.

The richer component still only scores bounded regional decompositions and
feeds the same geometry-native local schema/bridge machinery. It is still far
smaller than a generic sequence model and does not replace the computation
core.

### Does it justify enriching the learned regime field as the next architecture branch?

Yes, but cautiously.

The result suggests there is some benefit from increasing field expressiveness,
but the benefit is too limited to justify blind scaling. The next branch
should likely enrich field structure, not just width.

### If it fails, should this line be frozen and the next work move to a different stronger family or a more global learned chart field?

Freezing is not necessary yet, but simple scorer widening is not enough.

The next strongest step is probably a more structured learned regional field
or a more global learned chart field, rather than another small MLP capacity
increase on the same feature bundle.

## Bottom Line

v28 shows that a slightly richer learned field helps only modestly under the
stronger shifted family. That keeps the learned-field direction alive, but it
also shows that the next bottleneck is probably field structure rather than
just raw scorer capacity.
