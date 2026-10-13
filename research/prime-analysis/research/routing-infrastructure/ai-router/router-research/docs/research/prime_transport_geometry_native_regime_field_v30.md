# Prime Transport Geometry-Native Regime Field V30

## Purpose

Test whether an explicit structured chart field over contiguous regions can
recover more of the lost v26 margin on the stronger shifted family than the
shallow structured split-field used in v29.

## Mechanism

The chosen mechanism is a tiny piecewise chart field over contiguous blocks.

It keeps:

1. the same stronger shifted family as v27-v29
2. the same geometry-native local schema/bridge machinery
3. the same unchanged downstream geometry-native computation core

The structural change is:

- divide the sequence into a small number of contiguous coarse blocks
- learn per-block chart/regime scores for the existing bounded region variants
- decode those scores under an explicit contiguity constraint with at most
  three contiguous regions
- collapse the decoded path into explicit regions and apply the corresponding
  geometry-native local variant inside each region

This makes the regional chart structure explicit rather than only scoring fixed
split candidates or widening an unstructured scorer.

## Result

On the stronger shifted family:

- v26 earlier-family learned field:
  - transfer test accuracy `0.654854893684`
  - transfer query accuracy `0.732780933380`
- v27 tiny learned field:
  - transfer test accuracy `0.540852844715`
  - transfer query accuracy `0.584938704967`
- v28 richer scorer:
  - transfer test accuracy `0.548014342785`
  - transfer query accuracy `0.584723412991`
- v29 shallow structured field:
  - transfer test accuracy `0.549641907215`
  - transfer query accuracy `0.589491665363`
- v30 explicit structured chart field:
  - transfer test accuracy `0.543050110340`
  - transfer query accuracy `0.596061289310`
  - test loss `6.582681179047`
  - parameter count `8078`
  - effective state size `19`
- v30 tiny transformer:
  - transfer test accuracy `0.391276031733`
  - transfer query accuracy `0.441137850285`
  - test loss `2.638148069382`

The honest reading is mixed:

- v30 does not materially improve on v29 overall; it is worse on transfer
  accuracy
- v30 does improve on v29 in query accuracy
- the explicit contiguous chart field remains clearly geometry-native and still
  beats the tiny transformer baseline strongly
- this supports structured chart fields as a meaningful branch, but not yet as
  a decisive recovery of the lost v26 stronger-family margin
