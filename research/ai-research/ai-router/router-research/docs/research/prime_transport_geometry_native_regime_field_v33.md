# Prime Transport Geometry-Native Regime Field V33

## Purpose

Test whether sparse region-to-region interaction can improve the stronger
shifted-family result beyond the bounded regional-field line.

## Mechanism

The chosen mechanism is a sparse region-interaction field on top of a
v30-style contiguous regional decomposition.

It keeps:

1. the same stronger shifted family as v29-v32
2. the same geometry-native local schema/bridge machinery
3. the same unchanged downstream geometry-native computation core

The structural change is:

- first decode a small contiguous regional decomposition, as in the stronger
  regional field line
- treat regions as graph nodes
- pass one bounded interaction round only between adjacent regions
- build region messages from simple compositional operators:
  neighbor average, local-neighbor difference, and local-neighbor product
- use a tiny selector to choose each region's geometry-native variant from
  those sparse interaction features

This tests coordination directly rather than only changing region
representation or chart smoothness.

## Result

On the stronger shifted family:

- v29 structured field:
  - transfer test accuracy `0.549641907215`
  - transfer query accuracy `0.589491665363`
- v30 explicit contiguous chart field:
  - transfer test accuracy `0.543050110340`
  - transfer query accuracy `0.596061289310`
- v32 multiscale chart field:
  - transfer test accuracy `0.547607421875`
  - transfer query accuracy `0.579540550709`
- v33 sparse region-interaction field:
  - transfer test accuracy `0.539062500000`
  - transfer query accuracy `0.578022003174`
  - test loss `6.073722839355`
  - parameter count `10289`
  - effective state size `18`
- v33 tiny transformer:
  - transfer test accuracy `0.386962890625`
  - transfer query accuracy `0.443076908588`
  - test loss `2.301008701324`

The honest reading is negative:

- adding sparse region-to-region interaction does not improve over v29 or v30
- the result remains clearly above the tiny transformer baseline
- but the gain does not appear to come from useful coordination beyond the
  bounded representation line
- this does not support sparse structured interaction, in this small form, as
  the missing piece on the stronger shifted family
