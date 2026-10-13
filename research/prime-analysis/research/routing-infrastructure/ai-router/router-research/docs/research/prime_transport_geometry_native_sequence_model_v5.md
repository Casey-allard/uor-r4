# Prime Transport Geometry-Native Sequence Model V5

## Purpose

This note records the fifth direct bounded test of the thesis:

- multi-axis held-out generalization for the geometry-native sequence engine on
  the discourse-style contextual sequence family

This is the strongest bounded out-of-distribution test in the current sequence
model line.

## Multi-Axis Held-Out Split

The v5 task stays inside the same v3/v4 discourse family, but now removes more
than one structural factor from training and restores them together at test.

Training constraints:

- excludes the held-out speaker-reference query mode from v4:
  - `style = 2`
  - discourse lens resolves to `speaker`
  - `speaker != topic`
- uses shorter sequences only:
  - train length `26`
- allows at most one `SHIFT_STYLE` event per training sequence

Test restores all of the following simultaneously:

- the removed speaker-reference query mode
- longer sequences:
  - test length `42`
- denser style dynamics:
  - at least `4` style shifts per sequence
- at least `3` held-out query events per sequence

So v5 is a real multi-axis shift:

- removed overloaded query mode
- removed discourse-dynamics regime
- removed context-length regime

## Models

The compared models remain:

- geometry-native discourse engine
- tiny causal transformer baseline

The geometry-native path is unchanged in kind:

- exact geometric state evolution is primary
- explicit discourse-role state is primary
- no transformer attention blocks inside the geometry-native path
- no MoE blocks

## Result

Aggregate held-out result:

- geometry-native v5
  - held-out test accuracy `0.996558785439`
  - held-out query accuracy `0.985419213772`
  - test loss `0.011096824892`
  - parameter count `5699`
  - effective state size `10`
  - training time `1.812054665992` seconds
- tiny transformer baseline v5
  - held-out test accuracy `0.559709846973`
  - held-out query accuracy `0.469015806913`
  - test loss `1.370441794395`
  - parameter count `70531`
  - effective state size `64`
  - training time `4.928033250035` seconds

## Interpretation

Does the geometry-native model still generalize under stronger multi-axis
structural shift?

- yes

Does it still compare favorably to the tiny transformer baseline?

- yes, strongly

What changed relative to v4:

- the geometry-native engine remains extremely stable under the stronger shift
- the transformer baseline degrades further on both the full held-out metric
  and the held-out query metric

Why this matters:

- this is no longer only a single removed discourse mode
- the model must handle the removed mode inside longer contexts and under
  denser style changes than it saw in training
- the geometry-native engine still behaves like a true evolving-state sequence
  processor rather than a brittle same-distribution memorizer

This still does **not** prove broad language generalization.

But it does strengthen the thesis beyond single-axis held-out competence.

## Conclusion

This v5 result is the strongest direct thesis result so far.

Reason:

- it combines multiple structural shifts inside the same discourse family
- the geometry-native engine keeps almost all of its performance
- the tiny transformer baseline does not keep pace

So the current bounded evidence now supports the stronger claim:

- geometry can serve as the primary sequence-processing engine under a stronger
  simultaneous structural shift, not just within one bounded family or under
  one removed mode

## Next Stronger Test

The next smallest stronger test should be:

- cross-family transfer

For example:

- train on one bounded discourse-style family and test on a nearby but distinct
  contextual family with different latent-role update rules
- or train on the discourse family and test on a mixed discourse-plus-command
  family that shares contextual dependence but changes the surface grammar

That is the next honest step because the current line has now pushed within-
family bounded generalization far enough that the next question is whether the
geometry-native engine transfers across closely related contextual families.
