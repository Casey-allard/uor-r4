# Prime Transport Inner Representation Rebuild V2

## Purpose

Replace the remaining `native state -> flattened feature vector -> generic MLP`
core from `r1` with a structured update/readout path that operates directly on
native composite/chart state objects.

## Mechanism

`r2` keeps the bounded native state introduced in `r1`:

- persistent composite transport state
  - `query_semiprime`
  - `binding_semiprime`
- persistent chart state
  - `b`
  - `phi`
  - `r`
  - `next_return_gap`
- persistent discourse anchor state
  - `focus`
  - `speaker`
  - `topic`
  - `style`
  - `tags`
- persistent spin / admissibility state
  - `spin_bits`
  - `admissible_transition`

The key change is the learned core:

- no flattened global feature vector is passed to an MLP
- the readout now operates as structured table lookups and compositions over:
  - token id
  - chart coordinates
  - semiprime state
  - semiprime overlap
  - admissibility bit
  - spin bits
- role scores are then mapped natively into entity logits or tag logits using
  the current `(focus, speaker, topic, tags)` state

This is still bounded and still not a full exact-layer implementation, but the
main learned path is now tied to structured state objects rather than a flat
snapshot vector.

## Bounded Experiment

Task:
- same bounded v3 discourse/query task used in `r1`

Compared models:
- `geometry_native_sequence_model_v3_reference`
- `geometry_native_sequence_model_r1`
- `geometry_native_sequence_model_r2`
- `tiny_transformer_baseline_r2`

## Results

| model | test loss | test accuracy | query accuracy | params | effective state |
|---|---:|---:|---:|---:|---:|
| v3 reference | 0.0077 | 0.9978 | 0.9878 | 5699 | 10 |
| r1 rebuild | 0.4953 | 0.7361 | 0.7009 | 7427 | 16 |
| r2 rebuild | 0.5985 | 0.7426 | 0.7468 | 219 | 16 |
| tiny transformer | 0.7199 | 0.6958 | 0.6707 | 69763 | 64 |

## Was the core flatten-to-MLP path actually removed?

Yes.

`r2` does not flatten the native global state into a feature vector and then
feed that vector to an MLP.

The learned core now consists of structured parameter tables and direct
compositions over native state objects.

## Exact Structured Objects Used In The Update Path

The update path still uses the native structured state introduced in `r1`,
with no flattened learned interface:

- chart coordinates:
  - `b`
  - `phi`
  - `r`
  - `next_return_gap`
- discourse anchors:
  - `focus`
  - `speaker`
  - `topic`
  - `style`
  - `tags`
- persistent composite transport:
  - `query_semiprime`
  - `binding_semiprime`
- admissibility / spin state:
  - `admissible_transition`
  - `spin_bits`

These objects are updated natively by the same bounded semiprime proposal plus
admissibility-projection mechanism from `r1`.

## Exact Structured Objects Used In The Readout Path

The readout path operates directly on:

- token identity
- chart indices:
  - `b`
  - `phi`
  - `r`
  - `gap`
- composite state:
  - `query_semiprime`
  - `binding_semiprime`
  - semiprime overlap `gcd(query_semiprime, binding_semiprime)`
- admissibility state:
  - `admissible_transition`
- spin state:
  - each bit in `spin_bits`
- dynamic discourse geometry:
  - `(focus, speaker, topic)` for role-to-entity mapping
  - `tags` for role-to-tag mapping on `ASK`

The model first builds role logits from structured lookup tables over those
state objects, then converts those role logits into entity logits or tag logits
through the live discourse mapping. That conversion is also structured, not an
MLP over a flattened snapshot.

## Interpretation

`r2` is still far worse than the old approximate v3 reference, but it is a
real improvement over `r1` in the architecture-honest direction.

- test accuracy improves from `0.7361` to `0.7426`
- query accuracy improves more clearly from `0.7009` to `0.7468`
- parameter count drops from `7427` to `219`

So `r2` does not restore the lost v3 performance, but it does clarify that the
generic MLP was not necessary to beat the tiny transformer baseline on this
task. A small structured readout can do slightly better than `r1` while being
much more faithful to the intended architecture.

## What Remains Approximate After R2?

- the update law is still partly hand-built rather than learned as a native
  transport operator
- composite state is still represented as two semiprime slots, not a richer
  factor algebra or bundle object
- orbital / spin structure is still coarse and manually parameterized
- admissibility is still local projection, not a full constrained transport
  operator over a richer inner state

## Direct Answers

- Is the flatten-to-MLP bottleneck actually gone?
  - yes
- What exact structured objects are now used in the update path?
  - chart coordinates, discourse anchors, semiprime pair state, tags, and
    admissibility / spin state
- What exact structured objects are now used in the readout path?
  - token, chart indices, semiprime pair, semiprime overlap, admissibility,
    spin bits, and the live role-to-entity / role-to-tag mappings
- What remains approximate after `r2`?
  - the update law and transport algebra remain only partially native
- Does `r2` clarify or improve behavior relative to `r1`?
  - yes; it improves accuracy modestly, improves query accuracy materially, and
    removes the main architectural dishonesty from the learned core

## Next Smallest Rebuild Step

Learn the composite/chart transition operator itself over native state, rather
than keeping the `r1` hand-built semiprime proposal/projection update and only
rebuilding the readout.
