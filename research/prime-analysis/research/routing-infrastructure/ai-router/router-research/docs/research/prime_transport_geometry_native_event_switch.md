# Prime Transport Geometry-Native Event Switch

## Purpose

Test whether one bounded event-triggered bridge switch can recover more of the
stronger reduced-alignment loss than the fixed midpoint switch from v14.

This step keeps the same stronger v12 family, the same tiny bridge family, and
the same unchanged geometry-native engine. The only change is the switch rule.

## Mechanism

The chosen trigger is a local predictive-coherence drop.

For each sequence:

1. choose the initial bridge from the same small family used in v13/v14
2. scan later local windows
3. if the current bridge’s local score falls below its earlier baseline by a
   bounded amount and another bridge wins by a bounded margin, switch once at
   that event location
4. continue with the new bridge for the remainder of the sequence

The trigger is still geometry-native because it is based on bounded local
coherence from the existing readout over bridge-mediated geometric state, not
on a generic learned controller.

## Comparison

Stronger reduced-alignment line:

- v11 fixed bridge:
  - geometry-native accuracy `0.883501827717`
  - geometry-native query accuracy `0.969032287598`
- v12 fixed stronger-mismatch bridge:
  - geometry-native accuracy `0.552269339561`
  - geometry-native query accuracy `0.584205031395`
- v13 per-sequence calibration:
  - geometry-native accuracy `0.593377947807`
  - geometry-native query accuracy `0.633928596973`
- v14 fixed midpoint switch:
  - geometry-native accuracy `0.606119811535`
  - geometry-native query accuracy `0.660416662693`
- v15 event-triggered switch:
  - geometry-native accuracy `0.591052830219`
  - geometry-native query accuracy `0.637893617153`
  - geometry-native test loss `5.772159576416`
- v15 tiny transformer baseline:
  - accuracy `0.460844486952`
  - query accuracy `0.467969596386`
  - test loss `2.170051097870`

## Interpretation

This event-triggered switch rule does not improve on v14.

Relative to v14:

- transfer accuracy drops from `0.606119811535` to `0.591052830219`
- transfer query accuracy drops from `0.660416662693` to `0.637893617153`

Relative to v13:

- transfer accuracy also drops slightly from `0.593377947807` to
  `0.591052830219`
- transfer query accuracy rises only slightly from `0.633928596973` to
  `0.637893617153`

So the honest reading is:

- event-triggered switching still beats the fixed v12 bridge
- it still beats the tiny transformer baseline
- but this particular trigger does not outperform the simpler fixed midpoint
  switch, and it does not clearly outperform v13 overall

The most likely issue is that the chosen coherence-drop trigger is too noisy or
too late relative to the actual regime change, so it does not reliably place a
better switch point than the fixed midpoint heuristic.

## Answers

### Does event-triggered switching improve on v14 materially?

No.

It underperforms v14 on both overall transfer accuracy and transfer query
accuracy.

### Does it remain geometry-native in a meaningful sense?

Yes.

The mechanism still switches only among the same tiny bounded bridge family
using local predictive-coherence signals from the unchanged geometry-native
engine. No generic adapter is introduced.

### Does this further strengthen adaptive divisibility-mediated geometry as the right architecture direction?

Only weakly.

It does not strengthen the specific claim for this trigger rule. What it does
show is that adaptive switching remains plausible in principle, but switch
quality matters, and not every local trigger is better than a simpler fixed
recalibration point.

### What is the next smallest stronger test if v15 works?

The next smallest stronger test should keep the same bridge family and allow an
event-triggered switch based on an explicit bridge-contradiction or
query/binding conflict marker rather than a generic confidence-drop signal.

## Bottom Line

This bounded event-trigger rule is not a new best result. It remains better
than the fixed v12 bridge and the tiny transformer baseline, but it does not
beat the fixed midpoint switch from v14. So adaptive switching still looks
promising, but this particular regime-change detector is not yet the right one.
