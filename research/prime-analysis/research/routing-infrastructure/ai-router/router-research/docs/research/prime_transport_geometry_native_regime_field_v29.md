# Prime Transport Geometry-Native Regime Field V29

## Purpose

Test whether a more structured learned regional field can recover more of the
lost v26 margin on the stronger shifted family, where simple scorer-width
increases have mostly stalled.

## Mechanism

The chosen mechanism is a structured field over the ordered candidate split
points themselves.

It keeps:

1. the same stronger shifted family as v27/v28
2. the same bounded candidate boundaries
3. the same geometry-native local schema/bridge machinery

The structural change is:

- treat the candidate boundaries as an ordered regional lattice
- run a tiny convolutional field over that ordered set
- let nearby candidate splits share evidence before selecting the final split

This adds contiguity bias and regional structure without inserting any
transformer block or generic sequence front-end into the computation path.
