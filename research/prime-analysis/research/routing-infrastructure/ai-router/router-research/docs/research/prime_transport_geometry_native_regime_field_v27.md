# Prime Transport Geometry-Native Regime Field V27

## Purpose

Test whether the v26 learned regional-field hybrid remains superior under a
stronger shifted task family rather than only on the current stronger
reduced-alignment family.

## Mechanism

The chosen mechanism keeps the same small learned regional field as v26 and
changes only the transfer family.

The stronger shifted family adds:

1. longer sequences
2. stronger latent-role entanglement in binding and query rules
3. a harsher lossy proxy role/entity surface
4. slightly denser style/query/tag dynamics

The geometry-native computation core, divisibility-bridge structure, and tiny
learned regional field all remain the same in architectural role.

## Comparison

Reference points:

- best handcrafted hybrid result on prior family, v24:
  - geometry-native accuracy `0.649925589561`
  - geometry-native query accuracy `0.719008266926`
- learned regional-field result on prior family, v26:
  - geometry-native accuracy `0.654854893684`
  - geometry-native query accuracy `0.732780933380`

Stronger shifted family result:

- v27 learned regional-field hybrid:
  - geometry-native accuracy `0.540852844715`
  - geometry-native query accuracy `0.584938704967`
  - geometry-native test loss `6.327512264252`
  - geometry-native parameter count `6516`
  - geometry-native effective state size `15`
- v27 tiny transformer baseline:
  - accuracy `0.383219391108`
  - query accuracy `0.435639232397`
  - test loss `2.621042013168`

## Interpretation

This is a clear weakening under stronger shift, but not a collapse.

What held up:

- v27 still beats the tiny transformer baseline by a substantial margin on
  both overall accuracy and query accuracy
- the learned regional field remains functional on the stronger shifted family

What weakened:

- v27 loses a large amount of the v26 advantage from the prior family
- the gain does not look fully robust under the stronger shifted family

Numerically:

- relative to v26:
  - accuracy change: `-0.114002048969`
  - query-accuracy change: `-0.147842228413`
- relative to the v27 transformer baseline:
  - accuracy gain: `0.157633453608`
  - query-accuracy gain: `0.149299472570`

So the honest reading is that the learned regional field advantage survives in
direction but weakens sharply under a stronger shifted family. That means the
architecture still looks promising, but the current tiny field is not yet a
robust cross-family solution.

## Answers

### Does v27 preserve or lose the v26 advantage under stronger transfer?

It loses a substantial amount of the v26 advantage.

The model still beats the transformer baseline clearly, but it does not hold
up anywhere near the prior-family level.

### Does it remain geometry-native in a meaningful sense?

Yes.

The same small learned regional field is only selecting among bounded regional
decompositions, and the same geometry-native local schema/bridge machinery
still performs the computation.

### If it holds up, does that justify treating the learned regional-field hybrid as the current leading architecture?

Partially.

It remains the current leading bounded architecture in the sense that it still
wins under stronger shift, but this result shows the current learned field is
not yet robust enough to treat as a solved regional-structure mechanism.

### If it fails, is the next step to enrich the learned field or to revisit the transfer family design?

The next step should probably be to enrich the learned field before revisiting
the transfer family design.

The stronger shifted family is still coherent and useful because the learned
field did not collapse completely; it just weakened enough to show that the
current tiny field is underpowered for this shift magnitude.

## Bottom Line

v27 shows that the learned regional-field hybrid retains a real advantage
under stronger shift, but the advantage weakens sharply. That supports the
architecture direction in a qualified way: small learned regime fields help,
but the current tiny field is not yet robust enough to treat as the final
solution for stronger family transfer.
