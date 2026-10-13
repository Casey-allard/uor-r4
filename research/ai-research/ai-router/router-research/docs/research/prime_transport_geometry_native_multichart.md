# Prime Transport Geometry-Native Multi-Chart Selection

## Purpose

This note records the first bounded multi-chart selection test on the
reduced-schema-alignment transfer setting.

The question is:

- can bounded multi-chart selection recover more of the v7 loss than the
  single fixed chart-realignment rule from v8?

## Mechanism

The mechanism is:

- a fixed family of three local chart variants
- each variant uses the same v8 local chart rule plus an offset in `{0, 1, 2}`
- per sequence, score the candidate charts by internal predictive coherence:
  - mean max-softmax confidence from the existing geometry-native readout
- choose the highest-scoring chart
- run the unchanged geometry-native engine on that selected chart

This stays bounded and geometry-native:

- no transformer blocks
- no generic dense front-end
- no new learned adapter
- same main geometry-native engine after chart selection

## Comparison

Relevant aggregate comparison:

- v7 reduced-schema-alignment transfer
  - geometry-native accuracy `0.479204952717`
  - transfer query accuracy `0.544108569622`
- v8 single-rule chart realignment
  - geometry-native accuracy `0.568933844566`
  - transfer query accuracy `0.624918460846`
- v9 multi-chart selection
  - geometry-native accuracy `0.547449469566`
  - transfer query accuracy `0.597468376160`
  - transformer accuracy `0.480124086142`
  - transformer transfer query accuracy `0.491139233112`

## Result

Did bounded multi-chart selection recover more of the v7 loss than v8?

- no

What it did do:

- it remained better than the v7 collapse
- it reopened a clear gap over the transformer baseline

But relative to v8:

- accuracy dropped:
  - `0.568933844566 -> 0.547449469566`
- transfer query accuracy dropped:
  - `0.624918460846 -> 0.597468376160`
- test loss worsened slightly:
  - `6.270763874054 -> 6.499938011169`

## Interpretation

Does this support the stronger thesis of adaptive geometry rather than fixed
geometry?

- only weakly

Reason:

- selection among local charts does help relative to the unrecovered v7 case
- but it does not beat the simpler single-rule chart realignment

So the current evidence does **not** support the claim that this particular
multi-chart mechanism is the next architectural win.

Is the gain still geometry-native in a meaningful sense?

- yes

Because:

- chart family is fixed and small
- selection is based on internal coherence of the existing geometry-native
  engine
- the main computation engine remains unchanged

So this is not a disguised generic adapter. It is a negative-to-mixed
geometry-native result.

## Conclusion

The honest reading is:

- adaptive local charting still looks relevant
- but this simple bounded multi-chart selector is not better than the v8
  single-rule realignment

So the current best reduced-alignment recovery result remains:

- v8 single-rule chart realignment

## Next Stronger Test

The next smallest stronger test should be:

- a tiny calibration stage or coherence criterion that is more structured than
  raw confidence averaging

For example:

- chart scoring based on query-specific consistency rather than whole-sequence
  confidence
- or a very small calibration set to choose among the fixed chart family

That is the next honest step because v9 suggests the missing piece is not the
existence of multiple local charts, but how they are selected.
