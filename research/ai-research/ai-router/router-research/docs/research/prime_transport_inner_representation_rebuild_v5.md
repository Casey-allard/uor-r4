# Prime Transport Inner Representation Rebuild V5

## Purpose

Replace the remaining hand-designed anchor choice from `r4` with a learned
native anchor-selection mechanism while preserving:

- native persistent composite state
- native chart/spin coordinates
- native structured readout
- native learned partner-factor transport
- admissibility by construction

## Mechanism

`r5` keeps the `r4` learned partner-factor transport kernel, but the retained
anchor factor is no longer chosen by a hand-coded parity rule.

Instead, `r5` learns anchor logits over the two current factors in each
semiprime from native structured state pieces:

- mode: query vs binding
- token id
- left factor candidate
- right factor candidate
- opposite composite left/right factors
- chart turn
- discourse turn
- spin turn
- tag turn
- query-token flag
- tag-token flag

The selected anchor is one of the two current factors, and the learned partner
transport then operates on that chosen anchor directly in factor space.

## Was anchor selection actually learned without reintroducing flattening or projection?

Yes.

## Bounded Experiment

Task:
- same bounded v3 discourse/query task used in `r1`-`r4`

Compared models:
- `geometry_native_sequence_model_v3_reference`
- `geometry_native_sequence_model_r1`
- `geometry_native_sequence_model_r2`
- `geometry_native_sequence_model_r3`
- `geometry_native_sequence_model_r4`
- `geometry_native_sequence_model_r5`
- `tiny_transformer_baseline_r5`

## Results

| model | test loss | test accuracy | query accuracy | params | effective state |
|---|---:|---:|---:|---:|---:|
| v3 reference | 0.0077 | 0.9978 | 0.9878 | 5699 | 10 |
| r1 rebuild | 0.4953 | 0.7361 | 0.7009 | 7427 | 16 |
| r2 rebuild | 0.5985 | 0.7426 | 0.7468 | 219 | 16 |
| r3 rebuild | 0.5963 | 0.7424 | 0.7504 | 219 | 16 |
| r4 rebuild | 0.5144 | 0.7500 | 0.7862 | 423 | 16 |
| r5 rebuild | 0.4760 | 0.7503 | 0.7927 | 583 | 16 |
| tiny transformer | 0.7199 | 0.6958 | 0.6707 | 69763 | 64 |

## Anchor-Selection Rule

`r5` learns anchor selection as additive logits over the two current factors in
 each semiprime. For each candidate anchor position (`left` or `right`), the
 selector sums contributions from:

- mode: query vs binding
- token id
- candidate factor
- other factor in the same semiprime
- opposite composite left factor
- opposite composite right factor
- chart turn
- discourse turn
- spin turn
- tag turn
- query-token flag
- tag-token flag

The higher-logit factor is retained as anchor. The learned partner-factor
transport from `r4` then operates on that chosen anchor directly in factor
space.

## Direct Answers

- Is anchor selection now learned?
  - yes
- Is anchor selection still native to factor/composite/chart state?
  - yes
- How is admissibility preserved by construction?
  - the selected anchor is always one of the two current factors, so the next
    semiprime necessarily shares a factor with the previous semiprime
- What remains approximate after `r5`?
  - the anchor selector and transport kernel are both still shallow additive
    structured models learned from bounded transition statistics rather than a
    richer learned transport algebra

## Admissibility Check

On the bounded evaluation sample:

- `admissible_mean = 1.0`
- `admissible_min = 1`
- `admissible_max = 1`

So learned anchor selection did not break the by-construction admissibility
guarantee.

## Interpretation

`r5` is a small but real improvement over `r4`.

- test accuracy improves slightly
  - `0.7500 -> 0.7503`
- query accuracy improves more clearly
  - `0.7862 -> 0.7927`
- loss improves
  - `0.5144 -> 0.4760`

This is not a dramatic jump, but it does replace the next major hand-designed
piece in the rebuilt line without giving up the native architectural gains from
`r2`-`r4`.

## What Remains Approximate?

- anchor selection is learned, but still via a shallow additive structured
  selector
- partner-factor transport is learned, but also still shallow and additive
- chart/orbit structure is still coarse discrete state rather than a richer
  bundle geometry
- readout remains structured lookup/composition rather than a fuller exact
  transport/readout algebra

## Next Smallest Rebuild Step

Learn anchor selection and partner transport jointly as a richer
chart-conditioned native kernel, while keeping the no-flatten and
no-projection constraints intact.
