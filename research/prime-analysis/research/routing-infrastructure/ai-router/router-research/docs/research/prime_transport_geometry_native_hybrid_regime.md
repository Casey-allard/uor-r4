# Prime Transport Geometry-Native Hybrid Regional Regime

## Purpose

Test whether a bounded hybrid mechanism can combine the strongest parts of the
current line:

- coarse two-region segmentation from the v14 family
- local schema induction from the v21 family

The aim is to see whether regional segmentation plus local schema selection
can outperform both the current best overall stronger-mismatch recovery and
the current best query-accuracy result on the same stronger reduced-alignment
setting.

## Mechanism

The chosen mechanism is a two-region hybrid selector.

It works as follows:

1. split each sequence into two coarse contiguous regions
2. build a short geometry/conflict summary separately for each region
3. fit small support-window centroids on a calibration split for each region
4. let each region independently choose among the existing bounded bridge
   variants:
   - `v11_like`
   - `v12_base`
   - `hybrid`
5. run the unchanged geometry-native engine on the concatenated regional
   result

This remains geometry-native because the learned part only selects among
existing bounded divisibility-bridge regimes inside a coarse geometry-native
segmentation. It does not add a generic front-end or attention module.

## Comparison

Stronger reduced-alignment line:

- v14 midpoint switch:
  - geometry-native accuracy `0.606119811535`
  - geometry-native query accuracy `0.660416662693`
- v20 contradiction-grown repair:
  - geometry-native accuracy `0.604437589645`
  - geometry-native query accuracy `0.643454015255`
- v21 regional schema induction:
  - geometry-native accuracy `0.605003714561`
  - geometry-native query accuracy `0.686405777931`
- v22 hybrid regional segmentation + local schema induction:
  - geometry-native accuracy `0.646577358246`
  - geometry-native query accuracy `0.722751319408`
  - geometry-native test loss `4.806250572205`
- v22 tiny transformer baseline:
  - accuracy `0.449311763048`
  - query accuracy `0.455555558205`
  - test loss `2.245343446732`

## Interpretation

This is the first clear stronger-mismatch win over both reference lines.

What improved:

- v22 beats the current best overall stronger-mismatch result from v14
- v22 also beats the current best query-accuracy result from v21
- v22 improves sharply over the handcrafted repair line and the transformer
  baseline

Numerically:

- accuracy gain over v14: `0.040457546711`
- query-accuracy gain over v21: `0.036345541477`
- query-accuracy gain over v14: `0.062334656715`

So the honest reading is that the missing capability was not purely more
repair or purely more local schema selection. The strongest result comes from
combining a coarse regional split with local schema induction inside each
region.

## Answers

### Does the hybrid improve on both v14 and v21?

Yes.

It beats v14 on overall transfer accuracy and beats v21 on transfer query
accuracy, while also exceeding both on the complementary metric.

### Does it remain geometry-native in a meaningful sense?

Yes.

The hybrid only performs a bounded regional split and selects among existing
divisibility-bridge regimes inside each region. The core geometry-native
computation and bridge logic remain intact.

### Does this strengthen the case for adaptive regional geometry rather than fixed charts or local patch rules?

Yes, clearly.

This is the first stronger-mismatch result showing that adaptive regional
geometry beats both the best fixed coarse switch and the best purely local
schema selector. That is stronger evidence for regional regime structure as
the right architecture direction.

### What is the next smallest stronger test if v22 works?

The next smallest stronger test is a bounded variable-boundary regional split,
or a two-region split whose boundary is itself lightly induced rather than
fixed at midpoint.

## Bottom Line

The v22 hybrid is the strongest stronger-mismatch recovery so far. It is the
first result to beat both the best overall midpoint-switch baseline and the
best query-accuracy schema-induction baseline, which strongly supports
adaptive regional geometry over fixed charts or local patch rules alone.
