# Prime Transport Inner Representation Rebuild V6

## Purpose

Replace the separate learned anchor selector and learned partner transport from
`r5` with one joint native transition operator over factor/composite/chart
state.

## Mechanism

`r6` learns one structured kernel over admissible `(anchor choice, partner
factor)` pairs.

For each mode (`query` or `binding`), the operator scores six native transition
options:

- retain left factor, partner = 0
- retain left factor, partner = 1
- retain left factor, partner = 2
- retain right factor, partner = 0
- retain right factor, partner = 1
- retain right factor, partner = 2

The logits are built directly from native structured state pieces:

- token id
- current left/right factors
- opposite composite left/right factors
- chart turn
- discourse turn
- spin turn
- tag turn
- query-token flag
- tag-token flag

The chosen `(anchor, partner)` pair is then recomposed directly into the next
semiprime, preserving admissibility by construction because the anchor must be
one of the two current factors.

## Was the transition decision actually learned jointly without reintroducing flattening or projection?

Yes.

## Bounded Experiment

Task:
- same bounded v3 discourse/query task used in `r1`-`r5`

Compared models:
- `geometry_native_sequence_model_v3_reference`
- `geometry_native_sequence_model_r1`
- `geometry_native_sequence_model_r2`
- `geometry_native_sequence_model_r3`
- `geometry_native_sequence_model_r4`
- `geometry_native_sequence_model_r5`
- `geometry_native_sequence_model_r6`
- `tiny_transformer_baseline_r6`

## Results

| model | test loss | test accuracy | query accuracy | params | effective state |
|---|---:|---:|---:|---:|---:|
| v3 reference | 0.0077 | 0.9978 | 0.9878 | 5699 | 10 |
| r1 rebuild | 0.4953 | 0.7361 | 0.7009 | 7427 | 16 |
| r2 rebuild | 0.5985 | 0.7426 | 0.7468 | 219 | 16 |
| r3 rebuild | 0.5963 | 0.7424 | 0.7504 | 219 | 16 |
| r4 rebuild | 0.5144 | 0.7500 | 0.7862 | 423 | 16 |
| r5 rebuild | 0.4760 | 0.7503 | 0.7927 | 583 | 16 |
| r6 rebuild | 0.6617 | 0.7382 | 0.7310 | 735 | 16 |
| tiny transformer | 0.7199 | 0.6958 | 0.6707 | 69763 | 64 |

## Joint Operator

`r6` learns one structured kernel over six admissible transition options:

- retain left factor, partner = 0
- retain left factor, partner = 1
- retain left factor, partner = 2
- retain right factor, partner = 0
- retain right factor, partner = 1
- retain right factor, partner = 2

The logits are built directly from native structured state pieces:

- token id
- current left/right factors
- opposite composite left/right factors
- chart turn
- discourse turn
- spin turn
- tag turn
- query-token flag
- tag-token flag

The chosen `(anchor, partner)` pair is then recomposed directly into the next
semiprime.

## Direct Answers

- Is anchor-plus-partner transport now learned jointly?
  - yes
- Is the joint operator still native to factor/composite/chart state?
  - yes
- How is admissibility preserved by construction?
  - every scored option retains either the left or right current factor as
    anchor, so the next semiprime necessarily shares a factor with the previous
    one
- What remains approximate after `r6`?
  - the joint kernel is still a shallow additive structured model learned from
    bounded transition statistics, and the chart/orbit state is still coarse
    discrete state rather than a richer bundle geometry

## Admissibility Check

On the bounded evaluation sample:

- `admissible_mean = 1.0`
- `admissible_min = 1`
- `admissible_max = 1`

So the joint operator still preserves admissibility by construction.

## Interpretation

`r6` is a clean negative relative to `r5`.

- test accuracy drops
  - `0.7503 -> 0.7382`
- query accuracy drops
  - `0.7927 -> 0.7310`
- loss worsens
  - `0.4760 -> 0.6617`

So the joint transition decision is learned and native, but this smallest
bounded joint operator is worse than the separate learned anchor-plus-transport
decomposition from `r5`.

## What Remains Approximate?

- the joint operator is still shallow and additive
- the factor/composite/chart representation is still coarse
- readout remains structured lookup/composition rather than a richer exact
  transport algebra
- a better joint model may need richer factor interaction terms, not just
  collapsing the two decisions into one additive table

## Next Smallest Rebuild Step

Add richer native interaction terms inside the joint factor-pair kernel, rather
than returning to separate learned stages or to wrapper tuning.
