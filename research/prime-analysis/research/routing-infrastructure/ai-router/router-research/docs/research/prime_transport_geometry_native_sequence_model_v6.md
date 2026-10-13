# Prime Transport Geometry-Native Sequence Model V6

## Purpose

This note records the sixth direct bounded test of the thesis:

- cross-family transfer for the geometry-native sequence engine

This is the first direct test that crosses a bounded task-family boundary
rather than staying inside one family with train/test shifts.

## Transfer Setup

The setup is:

- train on the original bounded discourse/reference family from v3
- test on a different but related bounded contextual family

The test family is a delegation-style family that reuses the same surface
vocabulary but changes the latent control semantics:

- `SET_TOPIC` updates topic differently
- `SHIFT_STYLE` advances style differently
- `TAG` tokens bind through a different latent role lens
- `REFER` / `ASK` resolve through a different geometry-conditioned query lens

So this is not another split of the same family. It is a related contextual
family with changed latent-role semantics.

## Models

The compared models remain:

- geometry-native sequence engine
- tiny causal transformer baseline

Important clarification:

- the geometry-native engine uses the same learned readout as before
- transfer happens because the new family is expressed through the same
  abstract geometry-plus-context state schema
- so this is transfer of the learned readout over a stable abstract state
  surface, not raw-token zero-shot without structured state mediation

That is still a meaningful thesis test, because the claim being probed is that
geometry can be the primary sequence-processing engine.

## Result

Aggregate transfer result:

- geometry-native v6
  - transfer test accuracy `0.994750976562`
  - transfer query accuracy `0.971504330635`
  - test loss `0.014314385131`
  - parameter count `5699`
  - effective state size `10`
  - training time `2.029429665999` seconds
- tiny transformer baseline v6
  - transfer test accuracy `0.634033203125`
  - transfer query accuracy `0.661365151405`
  - test loss `1.114169836044`
  - parameter count `69891`
  - effective state size `64`
  - training time `5.248430415988` seconds

## Interpretation

Does the geometry-native model transfer across bounded task families?

- yes

Does it still compare favorably to the tiny transformer baseline?

- yes, strongly

What is strongest here:

- the geometry-native side was trained on one family and tested on another
  related family with changed latent semantics
- the learned readout still works because the geometry-native engine exposes a
  stable abstract contextual state
- the transformer baseline transfers much less cleanly across that family
  boundary

So this does strengthen the thesis beyond within-family generalization.

The correct limit is:

- this is still bounded transfer between closely related symbolic families
- it is not broad natural-language transfer
- it is not raw token-only transfer with no family-specific state engine

But that does not make the result trivial. The family boundary is real, and
the geometry-native engine is still doing the main sequence computation.

## Conclusion

This v6 result is the strongest direct thesis result so far.

Reason:

- it crosses a bounded task-family boundary
- the geometry-native engine keeps very strong performance
- the tiny transformer baseline does not keep pace

So the current bounded evidence now supports the stronger claim:

- geometry can act as the primary sequence-processing engine not only within
  one bounded contextual family, but also across a nearby contextual family
  when the family is expressed through the same abstract geometry-native state
  schema

## Next Stronger Test

The next smallest stronger test should be:

- reduced-structure transfer

For example:

- weaken the geometry-native abstract schema available at transfer time
- or transfer into a related family whose latent state is only partially
  aligned with the original schema

That is the next honest step because the current result is strongest precisely
where the stable abstract state surface is shared across families. The next
question is how robust the thesis remains when that alignment is only partial.
