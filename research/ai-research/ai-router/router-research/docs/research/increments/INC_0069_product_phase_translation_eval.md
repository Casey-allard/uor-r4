# INC-0069: Product Phase-Field Translation Evaluation

## Status
Confirm completed positive/narrow on 2026-03-11.

## Trigger
`INC-0065` confirmed that the explicit product `H^4 x H^4` branch is
mechanism-live, and `INC-0066` confirmed that it carries a distinct operator
signature. `INC-0067` and `INC-0068` then showed that neither raw label
projections nor routed residual/error probes beat the Hopf controls on the
proxy regression task. The next responsible check was therefore task
translation, not another proxy-spectral replay.

## Branch Contract
- keep the confirmed product phase-field routes fixed
- reuse the routed retrieval harness
- test whether the confirmed product routes preserve useful locality or pruning
  under task translation
- do not reopen route-law search or spectral-probe definitions here

## Evidence
- Config:
  - `configs/proxy_transfer_inc0069_product_phase_translation_screen.json`
  - `configs/proxy_transfer_inc0069_product_phase_translation_confirm.json`
- Analysis:
  - `results/analysis/inc0069_product_phase_translation_screen.json`
  - `results/analysis/inc0069_product_phase_translation_confirm.json`
- Gate note:
  - `docs/governance/gates/gate_20260311_124011.md`
  - `docs/governance/gates/gate_20260311_124616.md`

## Screen Read
- All routed variants stayed healthy across the 2-seed screen.
- The fixed product routes preserved real translated retrieval signal:
  - versus `HOPF_K25_BASE_PHI`, `H4XH4_FIELD_A100` reduced candidate fraction
    from `0.3488` to `0.3203` while slightly improving top-1 from `0.0475` to
    `0.0482`
  - versus `HOPF_K25_BASE_PHI`, `H4XH4_FIELD_A150` reduced candidate fraction
    to `0.3147` and amortized runtime from `0.4745s` to `0.4694s`, with a small
    top-1 drop to `0.0457`
- The screen was not a clean route-frontier win because `HOPF_BASE_K25_PHI`
  remained the strongest coarse-address translated comparator and the product
  gains split across quality and runtime.

## Confirm Read
- The 4-seed confirm preserved the same translated tradeoff.
- `H4XH4_FIELD_A100`
  - `top1=0.04625`
  - `cand_frac=0.31774`
  - `fallback=0.00058`
  - `amortized=0.4733s`
  - versus `HOPF_K25_BASE_PHI`:
    - top-1 delta `-0.00058`
    - candidate-fraction delta `-0.03333`
    - amortized delta `-0.0073s`
- `H4XH4_FIELD_A150`
  - `top1=0.04450`
  - `cand_frac=0.30833`
  - `fallback=0.00108`
  - `amortized=0.4688s`
  - versus `HOPF_K25_BASE_PHI`:
    - top-1 delta `-0.00233`
    - candidate-fraction delta `-0.04273`
    - amortized delta `-0.0119s`
- `HOPF_BASE_K25_PHI` remained the strongest coarse-address translated control:
  - `top1=0.04642`
  - `cand_frac=0.31126`
  - `amortized=0.4616s`
- `HOPF_PHI2_BAND_PHI` remained the best routed-quality / MSE branch under the
  default sweep read, but `H4XH4_FIELD_A100` still showed a better translated
  efficiency tradeoff than that widened Hopf control:
  - top-1 delta `+0.00008`
  - candidate-fraction delta `-0.02520`
  - amortized delta `-0.0101s`

## Decision
- Close `INC-0069` confirm positive/narrow as translated evaluation evidence.
- The fixed product branch does preserve useful translated retrieval structure.
- This is not a route-law promotion result:
  - pure Hopf still owns the main routed-quality lead
  - `HOPF_BASE_K25_PHI` still sets the best coarse-address translated
    comparator
- The remaining translated weakness is small recall / ordering loss on the
  fixed product routes, not collapse of locality or fallback stability.

## Next Step
- Move next to a fixed-route translated retrieval rescue branch (`INC-0070`):
  - keep the product route law fixed
  - work only on retrieval-side rescue surfaces such as secondary keys,
    selective rerank, or narrow backfill
  - try to recover the small top-1 loss without giving back the pruning/runtime
    gains

## Resume Note
Resume from the confirmed `INC-0069` artifacts, not from the pre-run queued
state. The next branch is retrieval-surface refinement on top of the fixed
product routes, not another route-law search.
