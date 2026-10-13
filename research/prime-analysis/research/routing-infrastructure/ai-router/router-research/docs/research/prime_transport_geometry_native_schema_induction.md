# Prime Transport Geometry-Native Regional Schema Induction

## Purpose

Test whether a bounded regional schema-induction / calibration stage can beat
both the current best stronger-mismatch recovery and the strongest handcrafted
repair result on the same stronger reduced-alignment transfer setting.

This step keeps the same stronger v12 family, the same divisibility-bridge
structure, and the same unchanged geometry-native engine. The only new piece
is a tiny calibration stage that infers which local regime is active from a
short support window before choosing among existing bounded strategies.

## Mechanism

The chosen mechanism is a tiny support-window regional schema selector.

It works as follows:

1. compute a short support-window summary from:
   - local geometry state statistics
   - query/binding density
   - proxy-role histogram
   - pairwise bridge-conflict rates
   - support-window selection scores for the three base bridge variants
2. generate a small calibration split from the same stronger v12 mismatch
   family
3. for each calibration sequence, label the best bounded strategy by actual
   downstream sequence performance among:
   - `v11_like`
   - `v12_base`
   - `hybrid`
   - `midpoint_switch`
   - `grownrepair`
4. fit one prototype centroid per strategy in the support-summary space
5. at test time, choose the nearest centroid and run the unchanged
   geometry-native engine under the selected strategy

This remains geometry-native in purpose because the calibrator is only
choosing among existing geometry/divisibility regimes. It does not add a
generic dense front-end or a transformer block.

## Comparison

Stronger reduced-alignment line:

- v12 fixed bridge:
  - geometry-native accuracy `0.552269339561`
  - geometry-native query accuracy `0.584205031395`
- v13 per-sequence calibration:
  - geometry-native accuracy `0.593377947807`
  - geometry-native query accuracy `0.633928596973`
- v14 midpoint switch:
  - geometry-native accuracy `0.606119811535`
  - geometry-native query accuracy `0.660416662693`
- v16 GCD-style revision:
  - geometry-native accuracy `0.592540919781`
  - geometry-native query accuracy `0.651993811131`
- v17 conflict revision:
  - geometry-native accuracy `0.597052812576`
  - geometry-native query accuracy `0.643229186535`
- v20 contradiction-grown repair:
  - geometry-native accuracy `0.604437589645`
  - geometry-native query accuracy `0.643454015255`
- v21 regional schema induction:
  - geometry-native accuracy `0.605003714561`
  - geometry-native query accuracy `0.686405777931`
  - geometry-native test loss `5.769324779510`
- v21 tiny transformer baseline:
  - accuracy `0.438523054123`
  - query accuracy `0.456230700016`
  - test loss `2.415073633194`

## Interpretation

This is a partial win rather than a clean sweep.

What improved:

- v21 beats the fixed v12 bridge clearly
- v21 beats the strongest handcrafted repair result v20 on both overall
  accuracy and query accuracy
- v21 sets the strongest query-accuracy result so far on the stronger mismatch
  line
- v21 still beats the tiny transformer baseline by a wide margin

What did not improve:

- v21 does not beat v14 on overall transfer accuracy
- the overall-accuracy gap to v14 is small, but still real

Numerically relative to v14:

- overall-accuracy gap: `0.001116096973` lower than v14
- query-accuracy gain: `0.025989115238` higher than v14

So the honest reading is that bounded regional schema induction helps in a way
the handcrafted repair rules did not: it learns a useful local regime signal
before downstream bridge selection and produces the best query-side recovery
so far. But it still does not fully displace the simpler midpoint switch as
the strongest result on the headline overall-accuracy metric.

## Answers

### Does regional schema induction improve on the current best stronger-mismatch recovery?

Partially.

It improves on v20 and sets the best query accuracy so far, but it does not
quite beat v14 on overall transfer accuracy.

### Does it remain geometry-native in a meaningful sense?

Yes.

The new stage only summarizes support-window geometry/conflict statistics and
selects among existing bounded bridge/repair regimes. The main
geometry-native engine and divisibility-bridge machinery remain unchanged.

### Does this strengthen the adaptive geometry architecture direction?

Yes, moderately.

This is the first stronger-mismatch result where a bounded semi-learned schema
calibration stage outperforms the handcrafted repair line and materially
improves the hard query metric. That is real evidence for adaptive geometry,
even though it falls just short of a full replacement of v14 on overall
accuracy.

### If it fails, does that imply the next step is a more serious learned chart-induction field rather than more bounded calibration tricks?

Not strictly, but it points that way.

Since v21 already surpasses the handcrafted repair line yet still misses v14
overall, the next smallest stronger test should likely be a somewhat richer
regional schema-induction field or regime map, rather than further manual
window/repair heuristics.

## Bottom Line

Bounded regional schema induction is the strongest adaptive query-recovery
result so far on the stronger mismatch line, and it beats the handcrafted
repair line. But it still misses the v14 midpoint switch slightly on overall
accuracy, so the evidence now favors adaptive geometry with light schema
induction, not yet a fully sufficient learned regional regime model.
