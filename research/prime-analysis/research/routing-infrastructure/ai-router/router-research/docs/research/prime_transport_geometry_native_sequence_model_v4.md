# Prime Transport Geometry-Native Sequence Model V4

## Purpose

This note records the fourth direct bounded test of the thesis:

- held-out generalization for the geometry-native sequence engine on the
  discourse-style contextual sequence family

This is the first direct out-of-distribution check in the current sequence
model line.

## Held-Out Structural Shift

The v4 task keeps the same bounded discourse-style family as v3, but changes
the split.

Training excludes one explicit overloaded query mode:

- query steps where:
  - `style = 2`
  - the discourse lens resolves to the `speaker` role
  - `speaker != topic`

Test uses a held-out set in which this removed mode is present and required.

So the shift is not a random split. It is a targeted held-out discourse mode:

- a speaker-reference query under a specific style regime with distinct
  speaker/topic assignments

This is a small but real structural generalization test.

## Models

The compared models remain:

- geometry-native sequence model v3-style engine
- tiny causal transformer baseline

The geometry-native path is unchanged in kind:

- exact geometric state evolution is primary
- explicit discourse-role state is primary
- no transformer attention blocks appear inside the geometry-native path
- no MoE blocks appear inside the geometry-native path

## Result

Aggregate held-out result:

- geometry-native v4
  - held-out test accuracy `0.996438443661`
  - held-out query accuracy `0.979522168636`
  - test loss `0.017076831311`
  - parameter count `5699`
  - effective state size `10`
  - training time `2.037606374943` seconds
- tiny transformer baseline v4
  - held-out test accuracy `0.620404422283`
  - held-out query accuracy `0.453924924135`
  - test loss `0.954713940620`
  - parameter count `70019`
  - effective state size `64`
  - training time `5.364191082888` seconds

## Interpretation

Does the geometry-native model generalize on the held-out discourse-style
task?

- yes

Does it still compare favorably to the tiny transformer baseline under the
structural shift?

- yes, strongly

What changed relative to v3:

- the result is no longer just same-distribution competence
- the geometry-native engine keeps nearly all of its performance on the
  removed discourse mode
- the tiny transformer baseline degrades much more sharply, especially on the
  held-out query metric

Why this matters:

- the held-out mode requires the model to apply the same contextual computation
  under a discourse configuration it did not see during training
- the geometry-native engine appears to generalize through explicit evolving
  state rather than only memorizing seen token-context mixtures

This still does **not** prove broad language generalization.

But it does strengthen the thesis beyond same-distribution bounded competence.

## Conclusion

This v4 result is the strongest direct thesis result so far.

Reason:

- it is still bounded and interpretable
- it tests real structural shift rather than another random split
- the geometry-native engine keeps strong performance under that shift
- the tiny transformer baseline does not

So the current bounded evidence now supports the stronger claim:

- geometry can serve as the primary sequence-processing engine not only within
  a bounded contextual family, but also under at least one meaningful held-out
  structural shift inside that family

## Next Stronger Test

The next smallest stronger test should be:

- multi-axis held-out generalization on the same discourse family

Examples:

- longer held-out context windows plus held-out discourse modes
- unseen style schedules plus unseen speaker/topic assignment templates
- train on one operation mixture and test on a shifted mixture together with a
  held-out role-resolution mode

That is the next honest step because it asks whether the geometry-native engine
continues to generalize when more than one contextual axis shifts at once.
