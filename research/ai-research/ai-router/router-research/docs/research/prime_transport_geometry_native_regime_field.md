# Prime Transport Geometry-Native Regime Field

## Purpose

Test whether a very small learned regional boundary/regime field can beat the
current handcrafted two-region hybrid family while leaving the
geometry-native computation core intact.

## Mechanism

The chosen mechanism is a tiny learned boundary field over the existing split
candidate set.

It works as follows:

1. keep the same two-region hybrid architecture and same local schema
   induction used in v22-v25
2. build a compact feature vector for each candidate split from:
   - split position
   - regional prototype distances
   - contradiction objectives near the split
   - prefix/suffix support summaries
3. use a tiny learned scorer to choose the best split from that bounded set
4. run the unchanged geometry-native engine under the resulting two-region
   decomposition

This remains geometry-native in purpose because the learned field only selects
among bounded geometry-native regional decompositions and still feeds the same
local schema/bridge machinery.

## Comparison

Stronger reduced-alignment line:

- v22 fixed-midpoint hybrid:
  - geometry-native accuracy `0.646577358246`
  - geometry-native query accuracy `0.722751319408`
- v23 induced-boundary hybrid:
  - geometry-native accuracy `0.647507429123`
  - geometry-native query accuracy `0.728548288345`
- v24 contradiction-aware hybrid:
  - geometry-native accuracy `0.649925589561`
  - geometry-native query accuracy `0.719008266926`
- v25 Pareto hybrid:
  - geometry-native accuracy `0.634393572807`
  - geometry-native query accuracy `0.697614133358`
- v26 learned regional boundary/regime field:
  - geometry-native accuracy `0.654854893684`
  - geometry-native query accuracy `0.732780933380`
  - geometry-native test loss `4.916808128357`
  - geometry-native parameter count `6516`
  - geometry-native effective state size `15`
- v26 tiny transformer baseline:
  - accuracy `0.461774557829`
  - query accuracy `0.456240296364`
  - test loss `2.194231033325`

## Interpretation

This is the first bounded learned-field result that clearly beats the
handcrafted hybrid family.

What improved:

- v26 beats the best handcrafted overall-accuracy result from v24
- v26 also beats the best handcrafted query-accuracy result from v23
- the gain is achieved with a very small learned field layered on top of the
  same geometry-native hybrid core

Numerically:

- accuracy gain over v24: `0.004929304123`
- query-accuracy gain over v23: `0.004232645035`
- accuracy gain over v23: `0.007347464561`

So the honest reading is that the handcrafted two-region hybrid family was not
fully saturated after all. A small learned regional field does recover more of
the remaining headroom while still leaving the geometry-native computation core
intact.

## Answers

### Does v26 improve on v23/v24 materially?

Yes.

The gains are not enormous, but they are clean on the two strongest reference
points: v26 beats v24 on overall accuracy and beats v23 on query accuracy,
while also exceeding both on the complementary metric.

### Does it remain geometry-native in a meaningful sense?

Yes.

The learned component only scores bounded regional decompositions and feeds the
same local geometry-native schema/bridge machinery. It is not a transformer
front-end or a replacement sequence model.

### Does this justify moving from handcrafted bounded segmentation to a learned regional regime field?

Yes, at least for this task family.

This is the strongest evidence so far that adaptive geometry benefits from a
small learned regional field once the bounded handcrafted hybrid family has
been exhausted.

### If it fails, should this family be frozen and the next step move to a stronger task family instead?

It did not fail, so freezing is not yet forced.

The next smallest stronger test is likely either:

- a slightly richer learned regional field with the same bounded candidate set,
  or
- transfer of this learned-field hybrid to a stronger or shifted family to
  check whether the gain is robust rather than family-specific

## Bottom Line

v26 is the strongest stronger-mismatch result so far. It shows that a very
small learned regional field can beat the handcrafted two-region hybrid family
while preserving geometry-native computation as the core engine. That is the
clearest support yet for moving from handcrafted bounded segmentation to a
learned regional regime field.
