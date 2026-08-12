# INC-0055: Product `H^4 x H^4` Retrieval Field Pilot

## Status
Closed.

## Trigger
`INC-0050` Slice A confirm showed that the product surrogate did not win the main MSE objective, but it did keep a top-1 advantage over static `H^4`.

## Hypothesis
The second hyperbolic factor may be more retrieval-like than regression-like.
It may carry a dynamic transport or imaginary field that helps top-1 discrimination or candidate pruning more than average target reconstruction.

## Minimal Scope
1. Keep this branch secondary only in systems risk, not in mathematical seriousness.
2. Test the second `H^4` as a retrieval or candidate-pruning field.
3. Prefer translated retrieval metrics or routed-locality metrics over pure proxy regression metrics.
4. Explicitly test whether route keys want discrete storage in a complex / imaginary field attached to the second `H^4`.
5. Keep the full object as `H^4 x H^4` in hyperbolic polar structure on both factors.

## Acceptance
- improves top-1 or candidate pruning materially without blowing up total cost
- explains why the product branch helps discrete decision quality more than proxy MSE

## First Working Theory
- The first `H^4` carries routing position.
- The second `H^4` carries retrieval / imaginary transport state.
- A good first surrogate may not be another continuous regression metric.
- A better first surrogate may be:
  - product-state distance for neighborhood ordering
  - plus discrete complex / imaginary route-key storage for candidate addressing
- This is the branch where the user's complex discrete key suggestion belongs.

## Artifacts
- Screen config:
  - `configs/proxy_transfer_inc0055_product_h4x4_retrieval_field_screen.json`
- Confirm config:
  - `configs/proxy_transfer_inc0055_product_h4x4_retrieval_field_confirm.json`
- Screen analysis:
  - `results/analysis/inc0055_product_h4x4_retrieval_field_screen.json`
- Confirm analysis:
  - `results/analysis/inc0055_product_h4x4_retrieval_field_confirm.json`
- Gate notes:
  - `docs/governance/gates/gate_20260306_125229.md`
  - `docs/governance/gates/gate_20260306_125455.md`

## Result
4-seed confirm means:
- `H4XH4_BUCKET_W025`: `mse=0.004318471`, `top1=0.03333`, `total=7.729s`, `cand_frac=0.3344`
- `H4XH4_CPX13_W025`: `0.004336934`, `0.03167`, `7.088s`, `0.2672`, `fallback=0.0070`, `secondary_keys=7.25`
- `STATIC_BUCKET`: `0.004327840`, `0.02558`, `7.794s`, `0.3344`

## Reading
- The product branch is now clearly retrieval/discrete-key positive.
- Discrete complex route-key storage on the second `H^4`:
  - reduced candidate fraction materially
  - reduced total runtime materially
  - kept fallback low
  - paid a bounded quality cost
- Plain product bucket remains the quality/top-1 reference.

## Decision
- Close `INC-0055` as a positive pilot.
- Promote translation/integration of the product complex-key law as the next slice.
