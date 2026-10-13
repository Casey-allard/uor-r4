# INC-0057: Product Complex-Key Hierarchical Backfill

## Status
Closed negative.

## Trigger
`INC-0056` confirmed that discrete complex / imaginary route-key storage survives translation into the routed retrieval harness:
- candidate fraction improved materially
- online and amortized cost improved materially
- fallback stayed at zero
- top-1 regressed slightly

## Hypothesis
The current translated complex key is a strong address field but a slightly incomplete recall field.
A small coarse Hopf backfill on top of the exact complex key should recover top-1 without giving back most of the candidate-pruning win.

## Minimal Scope
1. Keep exact complex key lookup as the first candidate stage.
2. Add a bounded coarse Hopf backfill stage only when retrieving from the translated harness.
3. Compare:
   - plain Hopf translated retrieval
   - exact complex-key translated retrieval
   - complex-key plus coarse-backfill translated retrieval
4. Measure:
   - candidate fraction
   - fallback rate
   - online retrieval time
   - amortized retrieval time
   - top-1 recovery
   - proxy MSE drift

## Acceptance
- preserves most of the complex-key pruning gain
- improves top-1 versus exact complex-key retrieval
- keeps fallback low
- keeps translated runtime materially below plain Hopf translated retrieval

## Result
The branch did not clear the bar.

Completed artifacts:
- `configs/proxy_transfer_inc0057_product_complex_backfill_smallbucket_screen.json`
- `results/analysis/inc0057_product_complex_backfill_smallbucket_screen.json`
- `docs/governance/gates/gate_20260306_135217.md`

Important killed subpaths:
- broad coarse backfill (`BF4/BF8`) remained materially expensive even after precomputing coarse extra pools
- low-margin selective backfill was pathologically expensive in live screening
- small-bucket selective backfill stayed cheap, but was nearly inert and did not repair top-1

## Quantitative Reading
2-seed small-bucket screen means:
- `HOPF_RET_CPX_P1_Q24`
  - `mse=0.00432337`
  - `top1=0.04767`
  - `total=14.123s`
  - `amortized=0.5652s`
  - `cand_frac=0.20754`
- `HOPF_RET_CPX_SB1_BF2_P1_Q24`
  - `mse=0.00432294`
  - `top1=0.04767`
  - `total=13.767s`
  - `amortized=0.5482s`
  - `cand_frac=0.20754`
  - `backfill_trigger=0.0005`
- `HOPF_RET_CPX_SB2_BF2_P1_Q24`
  - `mse=0.00432261`
  - `top1=0.04767`
  - `total=13.245s`
  - `amortized=0.5261s`
  - `cand_frac=0.20754`
  - `backfill_trigger=0.0008`

Low-margin failure note from the interrupted selective screen:
- `HOPF_RET_CPX_M002_BF2_P1_Q24` seed 0
  - `amortized=5.3099s`
  - `cand_frac=0.2059`
  - `backfill_trigger=0.7170`

## Decision
- Keep `HOPF_RET_CPX_P1_Q24` as the translated complex-key efficiency reference.
- Kill coarse-backfill rescue as the preferred recall path.
- The next recall hypothesis should not add candidates.
- Move next to exact-bucket reranking or another no-expansion repair path inside the complex-addressed candidate set.
