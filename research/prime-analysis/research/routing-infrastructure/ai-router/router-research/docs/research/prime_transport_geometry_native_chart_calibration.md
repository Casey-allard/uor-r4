# Prime Transport Geometry-Native Chart Calibration

## Purpose

Test whether bounded chart calibration can recover more reduced-schema-alignment
transfer performance than the v8 fixed realignment rule while keeping the same
geometry-native computation engine intact.

This step reuses the v7/v8/v9 reduced-alignment transfer setting exactly. The
only change is how the local chart is chosen before the main geometry-native
sequence engine runs.

## Mechanism

The chosen mechanism is short prefix-based chart calibration.

It keeps the same small three-chart family spirit as v9, but changes the
selection rule:

- build three candidate local chart views using the v8-style realignment plus
  offsets `{0, 1, 2}`
- score charts only on a short calibration prefix
- combine:
  - query-step confidence on the prefix
  - overall predictive confidence on the prefix
  - an inconsistency penalty when the same recovered referent group leads to
    unstable predicted labels
- commit to one chart for the full sequence

This is still geometry-native in spirit because it selects among bounded local
geometric charts using internal predictive coherence from the existing
geometry-native readout, rather than inserting a generic learned adapter or a
transformer front-end.

## Comparison

Reduced-alignment transfer results:

- v7 unrecovered transfer:
  - geometry-native accuracy `0.479204952717`
  - geometry-native query accuracy `0.544108569622`
  - transformer accuracy `0.480468750000`
  - transformer query accuracy `0.502776086330`
- v8 single-rule chart realignment:
  - geometry-native accuracy `0.568933844566`
  - geometry-native query accuracy `0.624918460846`
  - transformer accuracy `0.477711409330`
  - transformer query accuracy `0.527071118355`
- v9 multi-chart selection:
  - geometry-native accuracy `0.547449469566`
  - geometry-native query accuracy `0.597468376160`
  - transformer accuracy `0.480124086142`
  - transformer query accuracy `0.491139233112`
- v10 chart calibration:
  - geometry-native accuracy `0.539407193661`
  - geometry-native query accuracy `0.569453179836`
  - geometry-native test loss `6.897306919098`
  - tiny transformer accuracy `0.485064327717`
  - tiny transformer query accuracy `0.500942826271`
  - tiny transformer test loss `2.247161388397`

## Interpretation

The result is mixed and should be read conservatively.

What improved:

- v10 remains better than the unrecovered v7 geometry-native result
- v10 remains better than the tiny transformer baseline on both overall
  transfer accuracy and query accuracy

What did not improve:

- v10 does not beat v8
- v10 also does not beat v9 on query accuracy
- so better chart selection did not recover more of the reduced-alignment loss
  than the simpler fixed v8 realignment rule

The likely reason is that the calibration rule is still too local and too
dependent on confidence from a readout already trained in the aligned source
chart. It helps avoid total collapse, but it still does not recover a
sufficiently good local basis on the harder mismatched target family.

## Answers

### Does chart calibration / better selection improve on v8?

No. It improves on v7, but not on v8.

Relative to v8:

- transfer accuracy drops from `0.568933844566` to `0.539407193661`
- transfer query accuracy drops from `0.624918460846` to `0.569453179836`

### Is the gain still geometry-native in a meaningful sense?

Yes.

The gain over v7 still comes from bounded local chart choice over a fixed small
family of geometry-native transforms. The main computation engine is unchanged,
and no generic dense adapter or transformer block is introduced.

### Does this support the thesis of adaptive local geometry?

Partially.

It supports the weaker claim that adaptive local geometry is directionally
useful under reduced alignment, because v8, v9, and v10 all outperform the
unrecovered v7 case. But it does not strengthen the claim beyond v8. The best
adaptive result is still the simpler fixed realignment rule.

### What is the next smallest stronger test if v10 works?

The next smallest stronger test should not enlarge the chart family. It should
test a very small geometry-native support-query or calibration-then-commit
scheme where chart choice is evaluated on held-back support steps that are more
diagnostic of target-family role recovery than raw prefix confidence alone.

## Bottom Line

Bounded chart calibration does not beat the best current reduced-alignment
recovery result. The evidence still favors adaptive geometry over unrecovered
fixed-geometry transfer, but the present best recovery remains the simpler v8
single-rule chart realignment rather than a more elaborate chart-selection
criterion.
