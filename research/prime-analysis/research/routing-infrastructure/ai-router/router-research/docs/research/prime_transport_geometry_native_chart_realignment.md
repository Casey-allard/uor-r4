# Prime Transport Geometry-Native Chart Realignment

## Purpose

This note records the first direct geometry-native chart-realignment test on the
reduced-schema-alignment transfer setting exposed by v7.

The question is:

- can a small local geometry-native chart recovery step recover any of the v7
  loss without changing the main geometry-native computation engine?

## Realignment Mechanism

The chosen mechanism is:

- local angular role-frame rotation plus candidate-basis rebasing

More concretely:

- keep the main geometry-native engine unchanged
- keep the same reduced-alignment v7 transfer family unchanged
- at the interface, choose one of three small local role charts using:
  - `chart_rotation = (b + r + style) mod 3`
- use that rotation to:
  - rotate the local role frame
  - rebase the referent onto the candidate entangled basis rather than the raw
    clean role basis

This is deliberately small:

- no transformer blocks
- no generic dense adapter front-end
- no learned embedding replacement
- just one bounded geometry-native chart-selection rule

## Comparison

Relevant aggregate comparison:

- v6 shared-schema transfer
  - geometry-native accuracy `0.994750976562`
  - transfer query accuracy `0.971504330635`
- v7 reduced-schema-alignment transfer
  - geometry-native accuracy `0.479204952717`
  - transfer query accuracy `0.544108569622`
  - transformer accuracy `0.480468750000`
  - transformer transfer query accuracy `0.502776086330`
- v8 chart realignment
  - geometry-native accuracy `0.568933844566`
  - transfer query accuracy `0.624918460846`
  - transformer accuracy `0.477711409330`
  - transformer transfer query accuracy `0.527071118355`

## Result

Did chart realignment recover any of the v7 loss?

- yes

Recovered gain relative to v7:

- accuracy:
  - `0.479204952717 -> 0.568933844566`
- transfer query accuracy:
  - `0.544108569622 -> 0.624918460846`
- test loss:
  - `8.150774955750 -> 6.270763874054`

Did it restore the v6 regime?

- no

The shared-schema transfer result remains much stronger than the reduced-schema
realignment result.

## Interpretation

Is the recovered gain geometry-native in a meaningful sense?

- yes, in the bounded sense tested here

Reason:

- the gain comes from a small local chart-selection rule derived from route
  state
- the main geometry-native engine is unchanged
- there is no generic dense front-end absorbing the problem

So the recovery is not just a disguised non-geometric adapter.

What this means:

- v7 was not simply evidence that adaptive geometry fails
- part of the loss did come from presenting the target family in the wrong
  local chart
- a small bounded geometry-native realignment can recover some of that loss

What it does **not** mean:

- that the reduced-alignment problem is solved
- that adaptive geometry has already closed the whole gap

## Conclusion

This v8 result supports a stronger version of the thesis than v7 alone:

- adaptive local geometry helps

But only partially.

The current honest reading is:

- fixed aligned geometry works very strongly
- reduced alignment breaks the strong transfer story
- small geometry-native chart realignment recovers a meaningful part of the
  loss, but not the whole shared-schema advantage

## Next Stronger Test

The next smallest stronger test should be:

- bounded multi-chart selection or low-shot geometry-native realignment

For example:

- choose among a tiny family of two or three candidate local chart transforms
  instead of one fixed rule
- or allow a very small family-specific realignment calibration stage before
  transfer evaluation

That is the next honest step because v8 suggests the missing capability is
indeed adaptive geometry, but that one fixed local realignment rule is not yet
enough.
