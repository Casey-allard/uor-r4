# INC-0070: Product Phase Translation Rescue

## Status
Closed negative at screen stage on 2026-03-11.

## Trigger
`INC-0069` confirmed that the fixed product phase-field routes preserve useful
translated retrieval structure:
- lower candidate fraction than the main Hopf routed controls
- slightly lower amortized runtime than `HOPF_K25_BASE_PHI`
- stable low fallback

The remaining weakness is a small recall / top-1 loss on the fixed product
routes. The next responsible step is to refine translated retrieval on top of
that fixed route law, not to reopen routing geometry.

## Branch Contract
- keep the confirmed `INC-0065` / `INC-0069` product route law fixed
- keep the routed retrieval harness fixed
- change only translated retrieval rescue surfaces:
  - secondary keys
  - selective rerank
  - narrow backfill if needed
- judge success by translated retrieval tradeoff, not proxy-route MSE alone

## Minimal Scope
1. carry forward the fixed route set:
   - `HOPF_BASE_K25_PHI`
   - `HOPF_K25_BASE_PHI`
   - `HOPF_PHI2_BAND_PHI`
   - `H4XH4_FIELD_A100`
   - `H4XH4_FIELD_A150`
2. test one restrained rescue layer on the fixed product routes
3. compare:
   - candidate fraction
   - fallback
   - top-1
   - amortized runtime

## Working Hypothesis
The fixed product routes already give the right translated locality signal, but
the retrieval surface still loses a small amount of ordering / recall quality.
A modest retrieval-side rescue may recover that loss without giving back the
product pruning gain.

## Acceptance
- routed health stays stable
- at least one product route retains its pruning/runtime gain
- top-1 recovers to within noise of the best Hopf translated comparator, or
  beats at least one Hopf control on a clearly useful tradeoff

## Implemented Surface
- Added a narrow low-margin rerank mode in
  `tasks/router_retrieval_eval.py`:
  - `complex_rerank_mode=complex_plane_low_margin`
  - `complex_rerank_margin_threshold`
- The rerank applies the existing complex-plane correction only when the
  top-2 base-score margin inside the routed candidate set is below the chosen
  threshold.
- Added regression coverage in:
  - `tests/test_router_retrieval_eval.py`
  - `tests/test_cli_contract.py`

## Evidence
- Config:
  - `configs/proxy_transfer_inc0070_product_phase_translation_rescue_screen.json`
- Analysis:
  - `results/analysis/inc0070_product_phase_translation_rescue_screen.json`
- Gate note:
  - `docs/governance/gates/gate_20260311_130226.md`

## Screen Read
- The screen stayed healthy across all routed variants.
- The rescue surface itself failed its purpose:
  - `H4XH4_FIELD_A100` remained better than both rerank variants on the actual
    translated tradeoff:
    - base: `top1=0.04817`, `cand_frac=0.32029`, `amortized=0.4346s`
    - `R025`: `0.04800`, `0.32029`, `0.6064s`
    - `R050`: `0.04783`, `0.32029`, `0.6108s`
  - `H4XH4_FIELD_A150_R050` improved top-1 slightly relative to `A150`, but
    still stayed slower and weaker than `H4XH4_FIELD_A100`:
    - base: `top1=0.04567`, `cand_frac=0.31473`, `amortized=0.4749s`
    - `R050`: `0.04650`, `0.31473`, `0.6001s`
- No rerank variant beat its corresponding fixed product baseline on the
  translated quality/runtime tradeoff.
- None of the rerank variants preserved the `INC-0069` runtime advantage versus
  `HOPF_K25_BASE_PHI`.

## Decision
- Close `INC-0070` negative at screen stage.
- Keep `H4XH4_FIELD_A100` as the translated product balanced reference from
  `INC-0069`.
- Do not promote low-margin complex-plane reranking as the next rescue path for
  the fixed product routes.

## Next Step
- Move next to `INC-0071`: test secondary-key retrieval on the fixed product
  routes.
- Keep the route law fixed and change only the translated addressing surface.

## Resume Note
Resume from the fixed `INC-0069` product routes and the `INC-0070` negative
screen result. Do not reopen route-law search here.
