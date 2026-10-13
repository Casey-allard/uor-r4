# INC-0071: Product Phase Translation Secondary Keys

## Status
Confirm completed positive/narrow on 2026-03-11.

## Trigger
`INC-0070` showed that low-margin reranking inside the fixed product candidate
set is the wrong rescue surface:
- candidate fraction did not improve
- top-1 did not beat the fixed `H4XH4_FIELD_A100` baseline
- online and amortized cost both regressed sharply

The next smallest retrieval-side intervention is to test whether the second
`H^4` can help as a discrete translated-addressing field rather than as a local
score correction.

## Branch Contract
- keep the confirmed `INC-0065` / `INC-0069` product route law fixed
- keep the routed retrieval harness fixed
- change only the translated key surface
- reuse the existing `route_key_mode=hopf_plus_complex` support
- judge success by translated retrieval tradeoff, not by proxy-route MSE alone

## Minimal Scope
1. Carry forward the fixed routed controls and fixed product baselines:
   - `HOPF_BASE_K25_PHI`
   - `HOPF_K25_BASE_PHI`
   - `HOPF_PHI2_BAND_PHI`
   - `H4XH4_FIELD_A100`
   - `H4XH4_FIELD_A150`
2. Add restrained secondary-key variants on the fixed product routes.
3. Screen only the existing complex-key surface:
   - `route_key_mode=hopf_plus_complex`
   - default `complex_dims=1,3`
   - small radius-bin family
4. Compare:
   - candidate fraction
   - fallback
   - top-1
   - amortized runtime

## Working Hypothesis
The fixed product routes may already place queries into the right coarse routed
bucket, but the second `H^4` may still carry a discrete addressing signal that
can sharpen candidate selection more cleanly than reranking did. If that field
is real at the retrieval layer, a modest complex-key refinement may improve the
quality/pruning tradeoff without route-law changes.

## Acceptance
- routed health stays stable
- fallback remains low
- at least one product secondary-key variant keeps a useful pruning/runtime
  advantage
- and either:
  - recovers top-1 relative to the fixed product baseline within noise, or
  - beats at least one Hopf translated comparator on a clearly useful tradeoff

## Evidence
- Screen config:
  - `configs/proxy_transfer_inc0071_product_phase_translation_secondary_keys_screen.json`
- Confirm config:
  - `configs/proxy_transfer_inc0071_product_phase_translation_secondary_keys_confirm.json`
- Screen analysis:
  - `results/analysis/inc0071_product_phase_translation_secondary_keys_screen.json`
- Confirm analysis:
  - `results/analysis/inc0071_product_phase_translation_secondary_keys_confirm.json`
- Gate notes:
  - `docs/governance/gates/gate_20260311_131450.md`
  - `docs/governance/gates/gate_20260311_132141.md`

## Screen Read
- The first useful secondary-key point was already visible on the 2-seed screen:
  - `H4XH4_FIELD_A150_CPX8`
    - `top1=0.04767`
    - `cand_frac=0.19046`
    - `amortized=0.4406s`
    - `fallback=0.00317`
- This beat the fixed `H4XH4_FIELD_A150` baseline cleanly on both top-1 and
  pruning while staying healthy.
- It also beat `HOPF_K25_BASE_PHI` on top-1 at the screen stage, though the
  main systems read still needed confirm.

## Confirm Read
- The 4-seed confirm preserved the main addressing result.
- `H4XH4_FIELD_A150_CPX8`
  - `top1=0.04708`
  - `cand_frac=0.18723`
  - `fallback=0.00333`
  - `amortized=0.4999s`
- Versus the fixed `H4XH4_FIELD_A150` baseline:
  - top-1 delta `+0.00258`
  - candidate-fraction delta `-0.12110`
  - amortized delta `+0.0044s`
- Versus `HOPF_K25_BASE_PHI`:
  - top-1 delta `+0.00025`
  - candidate-fraction delta `-0.16384`
  - amortized delta `+0.0416s`
- Reading:
  - the second `H^4` is now evidence-positive as a discrete translated
    addressing field on top of the fixed product geometry
  - the branch is not yet a translated systems frontier, because the current
    implementation overhead still prevents the pruning gain from turning into a
    runtime win against the main Hopf translated control

## Decision
- Close `INC-0071` confirm positive/narrow.
- Promote `H4XH4_FIELD_A150_CPX8` as the translated secondary-key product
  reference.
- Do not over-promote it as a systems lead yet.

## Next Step
- Move next to `INC-0072`: keep the fixed product route law and the `A150_CPX8`
  secondary-key law fixed, and rescue only the retrieval implementation cost.

## Resume Note
Resume from the confirmed `INC-0071` secondary-key artifacts, not from the
queued state. Do not reopen routing geometry here.
