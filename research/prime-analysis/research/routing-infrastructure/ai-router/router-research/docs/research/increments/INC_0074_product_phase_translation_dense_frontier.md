# INC-0074: Product Phase Translation Dense Frontier

## Status
Confirm completed positive/narrow on 2026-03-11.

## Trigger
`INC-0071` is the confirm-stage translated-addressing result for the second
`H^4`, and `INC-0072` plus `INC-0073` show that the same fixed law can survive
as a translated systems/pruning result under both the original and larger-load
retrieval settings.

The next kill-list-consistent question is not fresh geometry. It is whether the
fixed `H^4 x H^4` route plus secondary-key law remains decision-useful when
measured directly against dense exact retrieval on the repo’s real-task
translation pipeline.

## Branch Contract
- keep the confirmed `INC-0065` product route law fixed
- keep the confirmed `INC-0071` secondary-key law fixed
- keep the confirmed `INC-0072/0073` translated systems rescue fixed
- change only the comparison surface:
  - add dense exact retrieval back into the active frontier check
- do not reopen geometry, phase coupling, or key-law search here

## Minimal Scope
1. Carry forward:
   - `DENSE_Q24`
   - `HOPF_BASE_K25_PHI`
   - `HOPF_K25_BASE_PHI`
   - `H4XH4_FIELD_A150`
   - `H4XH4_FIELD_A150_CPX8`
2. Reuse the larger-load translated setting from `INC-0073`.
3. Judge:
   - `top1`
   - `cand_frac`
   - `online`
   - `amortized`
4. Keep the evaluation in the existing WikiText/PTB proxy translation pipeline.

## Working Hypothesis
The fixed geometric route plus secondary-key field has already passed the routed
control comparisons that matter. The next honest question is whether the same
fixed law begins to cash out against dense exact retrieval on the translated
pipeline, which is the repo’s nearest live proxy for the hardware-efficiency
story.

## Acceptance
- `H4XH4_FIELD_A150_CPX8` materially reduces candidate fraction versus dense
- and reduces online or amortized retrieval cost versus dense
- while keeping top-1 close enough to remain decision-useful

## Evidence
- Screen config:
  - `configs/proxy_transfer_inc0074_product_phase_translation_dense_frontier_screen.json`
- Confirm config:
  - `configs/proxy_transfer_inc0074_product_phase_translation_dense_frontier_confirm.json`
- Screen analysis:
  - `results/analysis/inc0074_product_phase_translation_dense_frontier_screen.json`
- Confirm analysis:
  - `results/analysis/inc0074_product_phase_translation_dense_frontier_confirm.json`
- Gate notes:
  - `docs/governance/gates/gate_20260311_140824.md`
  - `docs/governance/gates/gate_20260311_141705.md`

## Screen Read
- The 2-seed screen was clearly positive against dense exact retrieval.
- `DENSE_Q24`
  - `top1=0.04833`
  - `cand_frac=1.00000`
  - `online=1.4961s`
  - `amortized=1.4961s`
- `H4XH4_FIELD_A150_CPX8`
  - `top1=0.04892`
  - `cand_frac=0.18418`
  - `online=0.4596s`
  - `amortized=0.9261s`
  - `fallback=0.00325`
- Versus dense:
  - top-1 delta `+0.00058`
  - candidate-fraction delta `-0.81582`
  - online delta `-1.0365s`
  - amortized delta `-0.5700s`
- The screen therefore justified a 4-seed confirm on the exact same fixed law.

## Confirm Read
- The 4-seed confirm preserved the dense-frontier systems result, but narrowed
  the quality story.
- `DENSE_Q24`
  - `top1=0.04912`
  - `cand_frac=1.00000`
  - `online=1.3395s`
  - `amortized=1.3395s`
- `H4XH4_FIELD_A150_CPX8`
  - `top1=0.04867`
  - `cand_frac=0.19032`
  - `online=0.3904s`
  - `amortized=0.8308s`
  - `fallback=0.00338`
- Versus dense:
  - top-1 delta `-0.00046`
  - candidate-fraction delta `-0.80968`
  - online delta `-0.9491s`
  - amortized delta `-0.5087s`
- The quality-matched routed reference on the same confirm was `H4XH4_FIELD_A150`:
  - `top1=0.04912`
  - `cand_frac=0.31475`
  - `amortized=0.9447s`
- `HOPF_BASE_K25_PHI` remained the highest-top1 routed control on the dense
  confirm:
  - `top1=0.05104`
  - `cand_frac=0.30414`
  - `amortized=0.9374s`

## Reading
- The fixed product phase-field branch is now confirm-stage positive directly
  against dense exact retrieval on the translated pipeline.
- The strongest systems point is still the secondary-key product branch:
  - `H4XH4_FIELD_A150_CPX8` keeps the major pruning and runtime gain
  - but it gives back a very small amount of top-1 relative to dense exact and
    to the best routed quality controls
- The dense-frontier claim is therefore narrow:
  - the geometry-plus-field law is now directly relevant on the repo’s real-task
    translation pipeline
  - but the highest-pruning systems point is not yet the highest-quality point

## Decision
- Close `INC-0074` confirm positive/narrow.
- Promote `H4XH4_FIELD_A150_CPX8` as the translated dense-frontier systems lead.
- Keep `H4XH4_FIELD_A150` as the quality-matched routed point on the same
  confirm load.

## Stop Rule
- If dense exact retrieval still dominates both quality and amortized cost on
  the larger-load translated setting, do not reopen geometry.
- Instead isolate whether the remaining gap is:
  - exact-search implementation overhead
  - candidate-ordering loss
  - or memory-bank/layout mismatch in the translated harness

## Resume Note
Resume from the confirmed `INC-0074` dense-frontier artifacts, not from the
queued state. Treat the kill list as the directional constraint and
`INC-0071` as the translated-addressing reference point for the second `H^4`.
