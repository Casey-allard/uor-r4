# Prime Transport Geometry-Native Hybrid Boundary V24

## Purpose

Test whether contradiction-aware boundary induction can improve on the current
best two-region hybrid by keeping the same architecture and changing only the
boundary score.

## Mechanism

The chosen mechanism is contradiction-aware boundary scoring.

It works as follows:

1. keep the same two-region hybrid and the same local schema induction as v23
2. score each candidate split with:
   - regional prototype fit
   - a contradiction/conflict bonus in a small window around the boundary
3. optionally refine one step around the best candidate
4. run the unchanged geometry-native engine under the chosen two-region split

This remains geometry-native because the added term measures local bridge
contradiction among bounded divisibility-bridge regimes near the boundary,
rather than adding a generic learned front-end.

## Comparison

Stronger reduced-alignment line:

- v14 midpoint switch:
  - geometry-native accuracy `0.606119811535`
  - geometry-native query accuracy `0.660416662693`
- v21 regional schema induction:
  - geometry-native accuracy `0.605003714561`
  - geometry-native query accuracy `0.686405777931`
- v22 fixed-midpoint hybrid:
  - geometry-native accuracy `0.646577358246`
  - geometry-native query accuracy `0.722751319408`
- v23 induced-boundary hybrid:
  - geometry-native accuracy `0.647507429123`
  - geometry-native query accuracy `0.728548288345`
- v24 contradiction-aware induced-boundary hybrid:
  - geometry-native accuracy `0.649925589561`
  - geometry-native query accuracy `0.719008266926`
  - geometry-native test loss `4.979606151581`
- v24 tiny transformer baseline:
  - accuracy `0.446056544781`
  - query accuracy `0.462293386459`
  - test loss `2.306171178818`

## Interpretation

This is a mixed but useful result.

What improved:

- v24 sets the best overall transfer accuracy so far on the stronger mismatch
  line
- v24 still stays well above v14, v21, and the transformer baseline

What regressed:

- v24 does not beat v23 on transfer query accuracy
- the contradiction-aware boundary term appears to trade some query quality
  for slightly better global placement

Numerically relative to v23:

- overall-accuracy gain: `0.002418160439`
- query-accuracy change: `-0.009540021420`

So the honest reading is that contradiction-aware boundary scoring helps place
the coarse regional split a bit better for overall sequence performance, but
it does not improve the best query-sensitive behavior. That looks like a real
boundary-placement gain, but not a clean across-the-board win.

## Answers

### Does v24 improve on v23 materially?

Only partially.

It improves overall transfer accuracy, but it does not improve query accuracy.

### If only slightly, does that indicate the hybrid architecture is near its bounded optimum on this family?

Probably yes.

The hybrid architecture already appears to be doing most of the work, and
further bounded boundary-scoring refinements now produce small metric trades
rather than large uniform gains.

### Does it remain geometry-native in a meaningful sense?

Yes.

The new term is still an interpretable contradiction measure over bounded
divisibility-bridge disagreement near the candidate split. The core
geometry-native engine and local schema logic are unchanged.

### What is the next smallest stronger test if v24 works or stalls?

The next smallest stronger test is likely a Pareto-style boundary selector or
a tiny two-objective boundary scorer that explicitly balances global transfer
accuracy against query-side correctness, rather than a more complex regional
model.

## Bottom Line

v24 is the new best stronger-mismatch result on overall accuracy, but not on
query accuracy. That suggests the hybrid regional architecture is likely near
its bounded optimum on this family, and the remaining gains are now about
which objective the boundary score should prioritize.
