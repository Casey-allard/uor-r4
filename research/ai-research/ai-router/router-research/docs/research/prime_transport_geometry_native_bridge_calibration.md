# Prime Transport Geometry-Native Bridge Calibration

## Purpose

Test whether a bounded support-window bridge calibration mechanism can recover
 more of the stronger v12 reduced-alignment loss than the fixed divisibility
 bridge rule.

This step keeps the stronger v12 family unchanged and keeps the main
geometry-native sequence engine unchanged. The only change is how the bridge is
chosen from a small bounded family before committing to it for the rest of the
sequence.

## Mechanism

The chosen calibration mechanism is short support-window bridge-family
selection.

The bridge family has three bounded members:

- `v11_like`
- `v12_base`
- `hybrid`

For each candidate bridge:

1. reconstruct the same v12 transfer sequence
2. apply that bridge variant to produce the local recovered role/referent path
3. score only an early support window using:
   - query-step confidence
   - overall confidence
   - a disagreement penalty over repeated recovered referent groups
4. commit to the best bridge for the rest of the sequence

This remains geometry-native because the calibration happens over a tiny fixed
family of divisibility-bridge rules using the existing geometry-native readout.
There is no generic dense adapter and no transformer block in the path.

## Comparison

Reduced-alignment line:

- v7 failure:
  - geometry-native accuracy `0.479204952717`
  - geometry-native query accuracy `0.544108569622`
- v8 single-rule chart realignment:
  - geometry-native accuracy `0.568933844566`
  - geometry-native query accuracy `0.624918460846`
- v11 divisibility bridge:
  - geometry-native accuracy `0.883501827717`
  - geometry-native query accuracy `0.969032287598`
- v12 stronger-mismatch fixed bridge:
  - geometry-native accuracy `0.552269339561`
  - geometry-native query accuracy `0.584205031395`
  - tiny transformer accuracy `0.452380955219`
  - tiny transformer query accuracy `0.464435160160`
- v13 support-window-calibrated bridge:
  - geometry-native accuracy `0.593377947807`
  - geometry-native query accuracy `0.633928596973`
  - geometry-native test loss `5.709022045135`
  - tiny transformer accuracy `0.459077388048`
  - tiny transformer query accuracy `0.459033608437`
  - tiny transformer test loss `2.079487323761`

## Interpretation

This is a real partial recovery on the stronger v12 family.

What improved relative to v12:

- transfer accuracy rises from `0.552269339561` to `0.593377947807`
- transfer query accuracy rises from `0.584205031395` to `0.633928596973`

What improved relative to v8:

- transfer accuracy rises from `0.568933844566` to `0.593377947807`
- transfer query accuracy rises from `0.624918460846` to `0.633928596973`

What did not happen:

- v13 still does not approach the strong v11 recovery level
- so support-window calibration helps materially, but it does not fully solve
  stronger-mismatch robustness

The result is still important because it shows that the v12 weakening was not
just a dead end for divisibility-mediated transition structure. A small local
bridge calibration step can recover a meaningful part of the loss and reopen a
margin over both v12 and the older chart line.

## Answers

### Does bridge calibration improve on v12 materially?

Yes.

Relative to v12:

- transfer accuracy improves by `0.041108608246`
- transfer query accuracy improves by `0.049723565578`

### Does it approach or exceed v11-level recovery?

No.

It remains far below v11:

- v11 accuracy `0.883501827717` vs v13 accuracy `0.593377947807`
- v11 query accuracy `0.969032287598` vs v13 query accuracy `0.633928596973`

### Is the gain still geometry-native in a meaningful sense?

Yes.

The gain comes from selecting among a tiny fixed family of arithmetic bridge
rules using early support-window evidence from the existing geometry-native
readout. The main engine is unchanged, and no generic learned adapter is
introduced.

### What is the next smallest stronger test if v13 works?

The next smallest stronger test should keep the same v12 family and move from
per-sequence bridge selection to bounded within-sequence bridge switching or
support-query bridge recalibration over one or two local windows. That would
test whether the remaining weakness comes from a single bridge decision being
too coarse for the stronger mismatch.

## Bottom Line

Support-window bridge calibration does improve on v12 materially and slightly
beats the older v8 chart recovery result on the stronger mismatch. It does not
recover the large v11 gain, but it strengthens the case for adaptive
divisibility-mediated geometry over fixed bridge rules.
