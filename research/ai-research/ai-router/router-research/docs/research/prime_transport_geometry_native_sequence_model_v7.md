# Prime Transport Geometry-Native Sequence Model V7

## Purpose

This note records the seventh direct bounded test of the thesis:

- cross-family transfer under reduced shared abstract-state alignment

This is the first direct test where the target family is intentionally less
neatly expressible in the same latent schema used during training.

## Reduced-Alignment Transfer Setup

Training remains:

- the original discourse/reference family from v3

Test uses a related but more entangled target family:

- same surface vocabulary
- different latent binding semantics
- different latent query semantics
- target-family state is projected back into the old shared schema only through
  a lossy proxy

Concretely:

- true tag binding uses fused candidate entities rather than the original clean
  focus / speaker / topic roles
- true query resolution also uses a more entangled geometry-conditioned rule
- the geometry-native transfer path sees only a reduced proxy projection of
  that target state into the original v3 feature schema

So this is a real step beyond v6:

- same-family training
- related-family testing
- weaker schema alignment at transfer time

## Result

Aggregate transfer result:

- geometry-native v7
  - transfer test accuracy `0.479204952717`
  - transfer query accuracy `0.544108569622`
  - test loss `8.150774955750`
  - parameter count `5699`
  - effective state size `10`
  - training time `2.026054167072` seconds
- tiny transformer baseline v7
  - transfer test accuracy `0.480468750000`
  - transfer query accuracy `0.502776086330`
  - test loss `2.312074422836`
  - parameter count `70019`
  - effective state size `64`
  - training time `5.265222666902` seconds

## Interpretation

Does the geometry-native model still transfer under reduced shared-schema
alignment?

- only partially

Does it still compare favorably to the tiny transformer baseline?

- not on overall transfer accuracy
- yes, slightly, on the transfer query metric

This is the first direct thesis test in the sequence-model line that clearly
breaks the earlier strong advantage.

What it shows:

- once the stable shared abstract-state schema is weakened enough, the
  geometry-native transfer story is no longer automatic
- the model still retains some contextual advantage on the query-specific
  metric
- but the broad cross-family advantage collapses to near parity overall

So this result weakens the strongest form of the current thesis line.

The right reading is not:

- geometry fails

The right reading is:

- geometry-native transfer appears strong when related families share a stable
  abstract contextual state surface
- when that alignment is reduced, transfer becomes fragile and the earlier
  large advantage no longer holds

## Conclusion

This v7 result is a meaningful boundary result.

It does **not** extend the earlier strong thesis gains.

Instead it sharpens them:

- geometry-native sequence computation is strongly promising within-family,
  under held-out structural shifts, and across nearby families with shared
  abstract-state alignment
- but reduced schema alignment is the first place where the current transfer
  picture substantially weakens

## Next Stronger Test

The next smallest stronger test should be:

- adaptation under reduced alignment

For example:

- a very small low-shot adaptation budget on the reduced-alignment family
- or a shared intermediate abstraction layer that is learned jointly across two
  related families before evaluation on a third

That is the next honest step because v7 suggests the current bottleneck is not
basic geometry-native computation, but transfer across mismatched abstract
schemas.
