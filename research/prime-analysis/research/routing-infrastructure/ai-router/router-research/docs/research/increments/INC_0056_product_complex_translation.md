# INC-0056: Product Complex-Key Translation

## Status
Closed.

## Trigger
`INC-0055` confirmed that discrete complex route-key storage on the second `H^4` is efficiency-positive:
- candidate fraction dropped materially
- runtime improved materially
- fallback stayed low

## Hypothesis
The complex-key law should be more valuable in a translated retrieval harness than in a pure proxy-regression harness, because its main signal is candidate addressing efficiency rather than average reconstruction quality.

## Minimal Scope
1. Keep the first `H^4` factor as the coarse routed position field.
2. Keep the second `H^4` factor as the retrieval / imaginary field.
3. Translate the discrete complex key into the routed retrieval harness.
4. Compare:
   - plain Hopf translated retrieval
   - product complex-key translated retrieval
5. Measure:
   - candidate fraction
   - fallback rate
   - online retrieval time
   - amortized retrieval time
   - quality / top-1 drift

## Acceptance
- improves translated candidate pruning or online/amortized retrieval cost materially
- keeps fallback low
- keeps quality loss bounded enough to remain decision-useful

## Artifacts
- Screen config:
  - `configs/proxy_transfer_inc0056_product_complex_translation_screen.json`
- Confirm config:
  - `configs/proxy_transfer_inc0056_product_complex_translation_confirm.json`
- Screen analysis:
  - `results/analysis/inc0056_product_complex_translation_screen.json`
- Confirm analysis:
  - `results/analysis/inc0056_product_complex_translation_confirm.json`
- Gate notes:
  - `docs/governance/gates/gate_20260306_131055.md`
  - `docs/governance/gates/gate_20260306_131507.md`

## Result
4-seed confirm means:
- `DENSE_Q24`: `mse=0.004321788`, `top1=0.04867`, `total=16.113s`, `amortized=0.6573s`, `cand_frac=1.0000`
- `HOPF_RET_P1_Q24`: `0.004324992`, `0.04683`, `16.679s`, `0.6685s`, `0.3511`
- `HOPF_RET_CPX_P1_Q24`: `0.004324266`, `0.04592`, `15.447s`, `0.6129s`, `0.2095`, `fallback=0.0000`, `secondary_keys=4.0`

## Reading
- The discrete complex key survives translation into the routed retrieval harness.
- It materially improves addressing efficiency against plain Hopf:
  - candidate fraction drops from about `0.3511` to `0.2095`
  - online cost drops from about `0.3245s` to `0.2976s` per repeat
  - amortized cost drops from about `0.6685s` to `0.6129s` per repeat
  - proxy MSE improves slightly
- The branch does pay a small top-1 penalty versus plain Hopf and dense exact retrieval.
- This is now a positive translated-addressing result, not just a product-state surrogate result.

## Decision
- Close `INC-0056` as positive.
- Promote the discrete complex / imaginary key law to the translated retrieval frontier.
- Queue a hierarchical backfill follow-up to recover top-1 without giving back the candidate-pruning gain.
