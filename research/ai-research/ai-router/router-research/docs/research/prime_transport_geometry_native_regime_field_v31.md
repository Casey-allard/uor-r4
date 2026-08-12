# Prime Transport Geometry-Native Regime Field V31

## Purpose

Test whether a small global learned chart field can recover more of the lost
v26 margin on the stronger shifted family than the current bounded regional
field family.

## Mechanism

The chosen mechanism is a tiny global chart field over coarse contiguous
blocks.

It keeps:

1. the same stronger shifted family as v27-v30
2. the same geometry-native local schema/bridge machinery
3. the same unchanged downstream geometry-native computation core

The structural change is:

- divide the sequence into a small number of coarse blocks
- score region variants locally for each block
- add a learned sequence-global low-rank chart map that modulates those
  blockwise scores jointly across the whole sequence
- decode the resulting field into a bounded contiguous assignment and apply
  the corresponding geometry-native local variant inside each region

This gives the field broader chart-scope than the regional variants from
v27-v30 without inserting any transformer block into the computation path.

## Result

On the stronger shifted family:

- v26 earlier-family learned field:
  - transfer test accuracy `0.654854893684`
  - transfer query accuracy `0.732780933380`
- v27 tiny learned field:
  - transfer test accuracy `0.540852844715`
  - transfer query accuracy `0.584938704967`
- v29 shallow structured field:
  - transfer test accuracy `0.549641907215`
  - transfer query accuracy `0.589491665363`
- v30 explicit contiguous chart field:
  - transfer test accuracy `0.543050110340`
  - transfer query accuracy `0.596061289310`
- v31 small global learned chart field:
  - transfer test accuracy `0.527669250965`
  - transfer query accuracy `0.581842005253`
  - test loss `6.738704204559`
  - parameter count `6914`
  - effective state size `22`
- v31 tiny transformer:
  - transfer test accuracy `0.381754547358`
  - transfer query accuracy `0.432125717402`
  - test loss `2.734487295151`

The honest reading is negative:

- v31 does not improve on v30 or v29
- it remains clearly geometry-native and still beats the tiny transformer
  baseline strongly
- but this particular small global chart field does not recover the lost v26
  stronger-family margin
- this suggests the next step is not simply broader scope by itself; the field
  structure would need to be richer or the shifted-family line may be near the
  limit of these bounded chart-field variants
