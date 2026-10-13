# Prime Transport Inner Representation Rebuild V1

## Purpose

Start the first direct rebuild of the inner representation so the
sequence-model line moves at least partway beyond:

- snapshot -> one-hot feature vector -> MLP

and instead carries native persistent transport state in the computation path.

## Mechanism

This first rebuild stays on the v3 discourse-style task family and changes the
inner state, not the outer wrapper.

It introduces three things directly into the actual state update path:

1. persistent composite transport state
   - `query_semiprime`
   - `binding_semiprime`

2. native chart / spin-style coordinates
   - direct cyclic chart coordinates for `(b, phi, r, next_return_gap)`
   - a persistent length-4 spin/admissibility register

3. admissibility-constrained transitions
   - each step proposes new composite transport state
   - if the proposal breaks the local admissibility condition, it is projected
     back to a nearby admissible semiprime state before readout features are
     formed

This is still bounded and still uses a small learned readout. So it is not a
full inner-architecture completion. But the representation itself is no longer
only a temporary wrapper rewrite.

## Bounded Experiment

Task:
- reuse the v3 discourse / query task so the rebuild can be compared against
  the prior best approximate geometry-native line on the same bounded problem

Compared models:
- `geometry_native_sequence_model_v3_reference`
- `geometry_native_sequence_model_r1`
- `tiny_transformer_baseline_r1`

## Results

| model | test loss | test accuracy | query accuracy | params | effective state |
|---|---:|---:|---:|---:|---:|
| v3 reference | 0.0077 | 0.9978 | 0.9878 | 5699 | 10 |
| r1 rebuild | 0.4953 | 0.7361 | 0.7009 | 7427 | 16 |
| tiny transformer | 0.7199 | 0.6958 | 0.6707 | 69763 | 64 |

## Architecture Check

### Did the rebuild move the intended architecture into the actual computation path?

Partially yes.

- persistent composite transport is now native evolving state:
  `query_semiprime` and `binding_semiprime` are updated every step and fed to
  readout features every step
- chart coordinates are now in the computation path as cyclic coordinates, not
  only discrete labels
- admissibility is now active: inadmissible composite proposals are projected
  back into the admissible semiprime set before the next state is committed

### What remains approximate?

The main approximation is still large.

- the final learned computation is still a feature vector plus MLP readout
- orbital / spin structure is present only as a small fixed register and cyclic
  coordinates, not as a genuine orbit-space transport law
- admissibility acts as local projection, not as a richer constrained transport
  geometry over a native bundle state
- composite transport is persistent, but still represented as two semiprime
  slots rather than a fuller compositional algebra

## Interpretation

This first rebuild clarifies the situation more than it improves performance.

- it successfully moves the intended architecture into the real computation
  path enough to make the audit actionable
- but it hurts performance badly relative to the previous approximate v3 model
- it still beats the tiny transformer baseline, which means the rebuilt state
  is not useless, but the current readout/update pairing is much weaker than
  the old hand-shaped snapshot encoding on this task

The blunt read is:
- the old line was performant partly because the snapshot encoding was an easy
  target for the MLP
- once composite state, chart coordinates, and admissibility are made more
  native, the current readout is no longer strong enough to exploit them well

## Direct Answers

- Did the rebuilt prototype successfully move the intended architecture into
  the actual computation path?
  - yes, partially and concretely
- What remains approximate?
  - the readout and much of the update law are still feature-engineered rather
    than native transport operators
- Does the rebuild improve, hurt, or merely clarify behavior on the bounded
  task?
  - it hurts performance strongly, but clarifies the representational gap
- What is the next smallest rebuild step after this?
  - replace the feature-vector readout with a native composite/chart state
    update-and-readout block that operates directly on persistent semiprime and
    chart coordinates instead of flattening them immediately into an MLP input
