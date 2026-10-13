# Prime Transport Geometry-Native Hybrid Boundary

## Purpose

Test whether a lightly induced split boundary can improve on the fixed-midpoint
hybrid by keeping the same two-region-plus-local-schema architecture while
moving only the region boundary.

## Mechanism

The chosen mechanism is a bounded split-boundary selector over a small
candidate set.

It works as follows:

1. keep the same two-region hybrid architecture as v22
2. consider a small bounded set of split candidates around the midpoint
3. for each candidate boundary:
   - fit regional support-window centroids on a calibration split
   - infer a local bridge regime separately in each region
4. at test time, select the boundary whose two regional summaries fit the
   learned prototypes best
5. run the unchanged geometry-native engine on the resulting two-region
   configuration

This remains geometry-native in purpose because the induction stage only
chooses boundary placement and local bridge regimes among bounded existing
geometry-native options. No generic dense front-end or attention block is
introduced.

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
  - geometry-native test loss `4.979188442230`
- v23 tiny transformer baseline:
  - accuracy `0.452194929123`
  - query accuracy `0.465191572905`
  - test loss `2.128999710083`

## Interpretation

This is a small but real improvement over v22.

What improved:

- v23 beats v22 on overall transfer accuracy
- v23 also beats v22 on transfer query accuracy
- v23 remains well above both v14 and v21

Numerically relative to v22:

- accuracy gain: `0.000930070877`
- query-accuracy gain: `0.005796968937`

So the honest reading is that the main hybrid architecture from v22 was
already doing most of the work, but lightly inducing the split boundary still
adds a measurable gain. That supports the idea that boundary placement matters
after regime logic is already in place.

## Answers

### Does v23 improve on v22 materially?

Only modestly.

The gain is real on both metrics, but it is incremental rather than a large
new jump.

### Does it remain geometry-native in a meaningful sense?

Yes.

The only added capability is choosing the split point from a small bounded set
using regional geometry/conflict summaries. The downstream geometry-native
engine and divisibility-bridge structure are unchanged.

### Does this strengthen the case that the remaining gap is boundary placement rather than regime logic?

Yes, moderately.

Since v23 improves on the already-strong v22 hybrid without adding more local
schema complexity, the most plausible explanation is better regime placement
rather than richer regime logic.

### What is the next smallest stronger test if v23 works?

The next smallest stronger test is a slightly richer but still bounded
boundary-induction rule, such as a light contradiction-aware boundary score or
one-step regional boundary refinement rather than a larger regime model.

## Bottom Line

The induced-boundary hybrid is the new best stronger-mismatch result, but by a
small margin over v22 rather than a new phase change. That is still useful:
it suggests the hybrid regime logic is largely right, and the next gains are
likely to come from better boundary placement rather than more local bridge
machinery.
