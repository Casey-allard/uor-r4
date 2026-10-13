# Prime Transport Geometry-Native Regime Field V32

## Purpose

Test one meaningfully different field architecture on the stronger shifted
family: a bounded multiscale chart field that preserves local structure while
adding nearby coarse coordination.

## Mechanism

The chosen mechanism is a two-level multiscale chart field.

It keeps:

1. the same stronger shifted family as v27-v31
2. the same geometry-native local schema/bridge machinery
3. the same unchanged downstream geometry-native computation core

The structural change is:

- divide the sequence into fine contiguous blocks
- score region variants locally at the fine-block level
- add a small coarse chart field over larger neighboring regions
- let that coarse field modulate only the nearby fine blocks
- decode the resulting multiscale scores into a bounded contiguous assignment

This is meaningfully different from the earlier line because it is not just a
wider scorer, not a pure split-candidate model, and not a single sequence-
global chart map.

## Result

On the stronger shifted family:

- v29 shallow structured field:
  - transfer test accuracy `0.549641907215`
  - transfer query accuracy `0.589491665363`
- v30 explicit contiguous chart field:
  - transfer test accuracy `0.543050110340`
  - transfer query accuracy `0.596061289310`
- v31 small global chart field:
  - transfer test accuracy `0.527669250965`
  - transfer query accuracy `0.581842005253`
- v32 multiscale chart field:
  - transfer test accuracy `0.547607421875`
  - transfer query accuracy `0.579540550709`
  - test loss `6.906696319580`
  - parameter count `6537`
  - effective state size `20`
- v32 tiny transformer:
  - transfer test accuracy `0.384521484375`
  - transfer query accuracy `0.444299966097`
  - test loss `2.868544340134`

The honest reading is mixed but mostly negative:

- v32 preserves local structure and recovers some overall accuracy relative to
  v30 and especially v31
- v32 does not beat the best shifted-family field result from v29 on overall
  accuracy
- v32 also does not beat v29 or v30 on query accuracy
- this suggests the shifted-family bottleneck is not solved by this bounded
  multiscale chart field, even though the mechanism remains clearly geometry-
  native and beats the tiny transformer baseline
