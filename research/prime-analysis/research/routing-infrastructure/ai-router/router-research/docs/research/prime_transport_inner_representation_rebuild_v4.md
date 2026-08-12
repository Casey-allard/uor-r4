# Prime Transport Inner Representation Rebuild V4

## Purpose

Replace the hand-designed factor transport rule from `r3` with a learned native
transport law while keeping:

- native persistent composite state
- native chart/spin state
- native structured readout
- admissibility by construction

## Mechanism

`r4` keeps the same factor-space transition form as `r3`:

- retain one anchor factor from the current semiprime
- learn the partner-factor motion directly in factor space
- recompose the next semiprime from `(anchor, learned_partner)`

The learned operator is not a flattened neural scorer. It is an additive
transport kernel over native structured state pieces:

- mode: query vs binding
- token id
- retained anchor factor
- coupled factor from the opposite composite state
- chart turn
- discourse turn
- spin turn
- tag turn
- query-token flag
- tag-token flag

Each component contributes learned partner-factor logits, and the logits are
summed to choose the transported partner factor.

This keeps the operator:

- native to factor/composite/chart state
- bounded and interpretable
- admissible by construction, because one previous factor is still retained in
  every next semiprime

## Was the transport law actually learned without reintroducing flattening or projection?

Yes.

## Bounded Experiment

Task:
- same bounded v3 discourse/query task used in `r1`-`r3`

Compared models:
- `geometry_native_sequence_model_v3_reference`
- `geometry_native_sequence_model_r1`
- `geometry_native_sequence_model_r2`
- `geometry_native_sequence_model_r3`
- `geometry_native_sequence_model_r4`
- `tiny_transformer_baseline_r4`

## Results

| model | test loss | test accuracy | query accuracy | params | effective state |
|---|---:|---:|---:|---:|---:|
| v3 reference | 0.0077 | 0.9978 | 0.9878 | 5699 | 10 |
| r1 rebuild | 0.4953 | 0.7361 | 0.7009 | 7427 | 16 |
| r2 rebuild | 0.5985 | 0.7426 | 0.7468 | 219 | 16 |
| r3 rebuild | 0.5963 | 0.7424 | 0.7504 | 219 | 16 |
| r4 rebuild | 0.5144 | 0.7500 | 0.7862 | 219 | 16 |
| tiny transformer | 0.7199 | 0.6958 | 0.6707 | 69763 | 64 |

## Transport Operator

The learned transport law is an additive structured kernel over native state
pieces. For each mode (`query` or `binding`), partner-factor logits are built
as:

- base partner prior
- plus token-specific partner logits
- plus retained-anchor partner logits
- plus coupled-opposite-factor partner logits
- plus chart-turn partner logits
- plus discourse-turn partner logits
- plus spin-turn partner logits
- plus tag-turn partner logits
- plus query-token flag logits
- plus tag-token flag logits

The argmax partner factor is then recomposed with the retained anchor factor to
form the next semiprime.

This is learned from training trajectories as transport statistics over native
contexts. It is not a flattened neural scorer and it is not a proposal-plus-fix
scheme.

## Direct Answers

- Is the transport law now learned?
  - yes
- Is the transport law still native to factor/composite/chart state?
  - yes
- How is admissibility preserved by construction?
  - one factor from the previous semiprime is always retained as the anchor, so
    each next semiprime shares a factor with the previous one
- What remains approximate after `r4`?
  - the learned law is still a shallow additive transport kernel learned from
    bounded empirical transition statistics rather than a richer learned bundle
    transport algebra

## Admissibility Check

On the bounded evaluation sample:

- `admissible_mean = 1.0`
- `admissible_min = 1`
- `admissible_max = 1`

So admissibility remains preserved by construction under the learned transport
operator.

## Interpretation

`r4` is the first real positive step after the architecture-honest rebuilds.

- it improves test accuracy over `r3`
  - `0.7424 -> 0.7500`
- it improves query accuracy more clearly
  - `0.7504 -> 0.7862`
- it improves loss substantially
  - `0.5963 -> 0.5144`

This still does not recover the old approximate `v3` line, but it does replace
the last major hand-designed piece in the rebuilt architecture and does so
without reintroducing flattening or projection.

## What Remains Approximate?

- the learned transport operator is still shallow and additive
- anchor selection remains hand-designed rather than learned
- the chart/orbit structure is still coarse discrete state, not a fuller
  continuous bundle geometry
- readout is still a structured lookup/composition block rather than an exact
  transport/readout algebra

## Next Smallest Rebuild Step

Learn anchor selection or a richer chart-conditioned factor transport kernel
while keeping the no-flatten and no-projection constraints intact.
