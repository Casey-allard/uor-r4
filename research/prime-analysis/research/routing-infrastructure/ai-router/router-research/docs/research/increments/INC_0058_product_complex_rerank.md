# INC-0058: Product Complex Exact-Bucket Rerank

## Status
Closed negative.

## Trigger
`INC-0057` showed that hierarchical backfill is the wrong recall repair path for the translated complex-key branch:
- broad coarse backfill is expensive
- low-margin backfill over-triggers and destroys cost
- small-bucket backfill is almost inert and does not recover top-1

## Hypothesis
The remaining translated top-1 gap is more likely a ranking problem than a recall problem.
A rerank term derived from the second `H^4` complex / imaginary field, applied inside the already-pruned exact complex bucket, may improve top-1 without expanding the candidate set.

## Minimal Scope
1. Keep the translated exact complex-key candidate set unchanged.
2. Add an optional rerank term inside routed retrieval:
   - base score: full routed chart similarity
   - rerank correction: local complex-plane similarity on the designated complex dims
3. Screen a small lambda family.
4. Compare against:
   - dense exact retrieval
   - plain Hopf translated retrieval
   - exact complex translated retrieval
5. Measure:
   - top-1
   - proxy MSE
   - candidate fraction
   - online retrieval time
   - amortized per-repeat time

## Acceptance
- improves top-1 versus `HOPF_RET_CPX_P1_Q24`
- preserves candidate fraction within a narrow tolerance
- does not materially worsen amortized per-repeat cost
- keeps fallback at zero

## Mathematical Rationale
If the translated complex key is already a strong address field, then the remaining error is likely within-bucket ordering, not bucket coverage.
That points toward using the second `H^4` as a local ordering field rather than a candidate-expansion field.
This is consistent with the broader project direction:
- global alignment from the hyperbolic / Poincare structure
- discrete addressing from the complex route key
- local repair from the same secondary geometric field, not from coarse candidate inflation

## Result
The simple complex-plane rerank did not rescue the translated branch cleanly.

Artifacts:
- `configs/proxy_transfer_inc0058_product_complex_rerank_screen.json`
- `results/analysis/inc0058_product_complex_rerank_screen.json`
- `docs/governance/gates/gate_20260306_140424.md`

2-seed screen means:
- `HOPF_RET_CPX_P1_Q24`
  - `mse=0.00432337`
  - `top1=0.04767`
  - `total=13.074s`
  - `amortized=0.5233s`
  - `cand_frac=0.20754`
- `HOPF_RET_CPX_R025_P1_Q24`
  - `mse=0.00432341`
  - `top1=0.04767`
  - `total=13.402s`
  - `amortized=0.5347s`
  - `cand_frac=0.20754`
- `HOPF_RET_CPX_R050_P1_Q24`
  - `mse=0.00432431`
  - `top1=0.04750`
  - `total=12.599s`
  - `amortized=0.5028s`
  - `cand_frac=0.20754`
- `HOPF_RET_CPX_R075_P1_Q24`
  - `mse=0.00432388`
  - `top1=0.04783`
  - `total=17.449s`
  - `amortized=0.7014s`
  - `cand_frac=0.20754`

## Decision
- Keep `HOPF_RET_CPX_P1_Q24` as the translated complex-key efficiency reference.
- Kill simple exact-bucket complex-plane reranking as a primary rescue path.
- Move next to a coupled `H^4 x H^4` polar-flow branch instead of more local retrieval patching.
