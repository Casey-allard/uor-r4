# INC-0093: Product Phase Translation Cache Residency Mix

## Status
Confirm completed positive/explanatory on 2026-03-12.

## Trigger
`INC-0092` closes the lower-bank warm-cache search more strongly than expected:
under the expanded seed schedule, both `T2500` and `T2501` survive at `Q01`.
That means the lower-bound bank search is done. The next honest question is no
longer bank threshold location. It is whether the systems win survives outside
the idealized fully warm cache state.

## Branch Contract
- keep the confirmed `INC-0065` route law fixed
- keep the confirmed `INC-0071` secondary-key law fixed
- keep the confirmed `INC-0083` persistent-cache implementation fixed
- keep the bank/repeat points fixed:
  - `T2500 Q01` lower-bank floor
  - `T40000 Q01` stabilized warm-cache point
- use existing cache-compare tooling where possible
- do not reopen geometry, retrieval scoring, or bank threshold search

## Minimal Scope
1. Carry only:
   - `DENSE`
   - `H4XH4_FIELD_A150_CPX8`
2. Compare operational cache states:
   - cold
   - full warm
   - any partial residency state already supported by the existing tools
3. Evaluate only the two anchor operating points:
   - `T2500 Q01`
   - `T40000 Q01`
4. Treat this as workload/operational robustness, not route-law tuning.

## Stop Rule
- If the systems win survives only under fully warm cache, narrow the hardware
  claim to persisted-session reuse.
- If the win survives broader cache-residency conditions, promote the branch to
  a stronger operational hardware-side result and move on from cache-threshold
  analysis.

## Resume Note
Resume from the `INC-0093` confirm artifact and treat the next branch as a
chart-resident / route-ephemeral repeat map on the same fixed operating points
rather than more bank refinement or any new route-law work.

## Evidence
- Configs:
  - `configs/proxy_transfer_inc0093_product_phase_translation_cache_residency_mix_screen_cold.json`
  - `configs/proxy_transfer_inc0093_product_phase_translation_cache_residency_mix_screen_prewarm.json`
  - `configs/proxy_transfer_inc0093_product_phase_translation_cache_residency_mix_screen_warm.json`
  - `configs/proxy_transfer_inc0093_product_phase_translation_cache_residency_mix_confirm_cold.json`
  - `configs/proxy_transfer_inc0093_product_phase_translation_cache_residency_mix_confirm_prewarm.json`
  - `configs/proxy_transfer_inc0093_product_phase_translation_cache_residency_mix_confirm_warm.json`
- Analyses:
  - `results/analysis/inc0093_product_phase_translation_cache_residency_mix_screen_cold.json`
  - `results/analysis/inc0093_product_phase_translation_cache_residency_mix_screen_prewarm.json`
  - `results/analysis/inc0093_product_phase_translation_cache_residency_mix_screen_warm.json`
  - `results/analysis/inc0093_product_phase_translation_cache_residency_mix_screen_compare_chart.json`
  - `results/analysis/inc0093_product_phase_translation_cache_residency_mix_screen_compare_route.json`
  - `results/analysis/inc0093_product_phase_translation_cache_residency_mix_screen_compare_full.json`
  - `results/analysis/inc0093_product_phase_translation_cache_residency_mix_confirm_cold.json`
  - `results/analysis/inc0093_product_phase_translation_cache_residency_mix_confirm_prewarm.json`
  - `results/analysis/inc0093_product_phase_translation_cache_residency_mix_confirm_warm.json`
  - `results/analysis/inc0093_product_phase_translation_cache_residency_mix_confirm_compare_chart.json`
  - `results/analysis/inc0093_product_phase_translation_cache_residency_mix_confirm_compare_route.json`
  - `results/analysis/inc0093_product_phase_translation_cache_residency_mix_confirm_compare_full.json`
- Reports:
  - `docs/reports/INC0093_PRODUCT_PHASE_TRANSLATION_CACHE_RESIDENCY_MIX_SCREEN_COMPARE_CHART.md`
  - `docs/reports/INC0093_PRODUCT_PHASE_TRANSLATION_CACHE_RESIDENCY_MIX_SCREEN_COMPARE_ROUTE.md`
  - `docs/reports/INC0093_PRODUCT_PHASE_TRANSLATION_CACHE_RESIDENCY_MIX_SCREEN_COMPARE_FULL.md`
  - `docs/reports/INC0093_PRODUCT_PHASE_TRANSLATION_CACHE_RESIDENCY_MIX_CONFIRM_COMPARE_CHART.md`
  - `docs/reports/INC0093_PRODUCT_PHASE_TRANSLATION_CACHE_RESIDENCY_MIX_CONFIRM_COMPARE_ROUTE.md`
  - `docs/reports/INC0093_PRODUCT_PHASE_TRANSLATION_CACHE_RESIDENCY_MIX_CONFIRM_COMPARE_FULL.md`
  - `docs/reports/INC0093_PRODUCT_PHASE_TRANSLATION_CACHE_RESIDENCY_MIX_CONFIRM.md`
- Gate notes:
  - `docs/governance/gates/gate_20260312_071450.md`
  - `docs/governance/gates/gate_20260312_071837.md`
  - `docs/governance/gates/gate_20260312_072037.md`
  - `docs/governance/gates/gate_20260312_072941.md`
  - `docs/governance/gates/gate_20260312_073711.md`
  - `docs/governance/gates/gate_20260312_074111.md`

## Screen Read
- The 2-seed screen immediately separated the residency states.
- `T2500 Q01`
  - dense:
    - `DENSE_Q01_T2500`: `top1=0.052500`, `amortized=0.1183s`
  - chart-only warm:
    - `CHART_H4XH4_FIELD_A150_CPX8_Q01_T2500`: `amortized=0.0667s`
    - warm margin vs dense: `+0.0516s`
    - cache hits: `chart=1.0`, `route=0.0`
  - route-only warm:
    - `ROUTE_H4XH4_FIELD_A150_CPX8_Q01_T2500`: `amortized=2.2888s`
    - warm margin vs dense: `-2.1705s`
    - cache hits: `chart=0.0`, `route=1.0`
  - full warm:
    - `FULL_H4XH4_FIELD_A150_CPX8_Q01_T2500`: `amortized=0.0628s`
    - warm margin vs dense: `+0.0555s`
    - cache hits: `chart=1.0`, `route=1.0`
- `T40000 Q01`
  - dense:
    - `DENSE_Q01_T40000`: `top1=0.049875`, `amortized=9.2134s`
  - chart-only warm:
    - `CHART_H4XH4_FIELD_A150_CPX8_Q01_T40000`: `amortized=2.8010s`
    - warm margin vs dense: `+6.4124s`
    - cache hits: `chart=1.0`, `route=0.0`
  - route-only warm:
    - `ROUTE_H4XH4_FIELD_A150_CPX8_Q01_T40000`: `amortized=34.8566s`
    - warm margin vs dense: `-25.6432s`
    - cache hits: `chart=0.0`, `route=1.0`
  - full warm:
    - `FULL_H4XH4_FIELD_A150_CPX8_Q01_T40000`: `amortized=2.0354s`
    - warm margin vs dense: `+7.1780s`
    - cache hits: `chart=1.0`, `route=1.0`
- The screen justified confirm because the same pattern was already clear:
  chart residency carried almost all of the rescue, route-only residency did
  not, and top-1 / candidate fraction stayed unchanged across state changes.

## Confirm Read
- The 4-seed confirm kept the same structure and made the lower-bank boundary
  more precise.
- `T2500 Q01`
  - dense:
    - `DENSE_Q01_T2500`: `top1=0.052000`, `amortized=0.1035s`
  - chart-only warm:
    - `CHART_H4XH4_FIELD_A150_CPX8_Q01_T2500`: `top1=0.044600`,
      `cand_frac=0.193328`, `amortized=0.1326s`
    - warm margin vs dense: `-0.0292s`
    - cache hits: `chart=1.0`, `route=0.0`
  - route-only warm:
    - `ROUTE_H4XH4_FIELD_A150_CPX8_Q01_T2500`: `top1=0.044600`,
      `cand_frac=0.193328`, `amortized=2.5345s`
    - warm margin vs dense: `-2.4310s`
    - cache hits: `chart=0.0`, `route=1.0`
  - full warm:
    - `FULL_H4XH4_FIELD_A150_CPX8_Q01_T2500`: `top1=0.044600`,
      `cand_frac=0.193328`, `amortized=0.0562s`
    - warm margin vs dense: `+0.0473s`
    - cache hits: `chart=1.0`, `route=1.0`
- `T40000 Q01`
  - dense:
    - `DENSE_Q01_T40000`: `top1=0.048850`, `amortized=9.5814s`
  - chart-only warm:
    - `CHART_H4XH4_FIELD_A150_CPX8_Q01_T40000`: `top1=0.047325`,
      `cand_frac=0.183764`, `amortized=2.3856s`
    - warm margin vs dense: `+7.1959s`
    - cache hits: `chart=1.0`, `route=0.0`
  - route-only warm:
    - `ROUTE_H4XH4_FIELD_A150_CPX8_Q01_T40000`: `top1=0.047325`,
      `cand_frac=0.183764`, `amortized=34.7146s`
    - warm margin vs dense: `-25.1332s`
    - cache hits: `chart=0.0`, `route=1.0`
  - full warm:
    - `FULL_H4XH4_FIELD_A150_CPX8_Q01_T40000`: `top1=0.047325`,
      `cand_frac=0.183764`, `amortized=2.0185s`
    - warm margin vs dense: `+7.5629s`
    - cache hits: `chart=1.0`, `route=1.0`
- Stability checks all passed:
  - top-1 stayed unchanged between cold and warm for every routed state
  - candidate fraction stayed unchanged between cold and warm for every routed
    state
  - the change is operational cost only, not a retrieval-quality shift

## Reading
- Route-only residency is not enough on this harness at either anchor point.
- Chart residency is the dominant operational lever:
  - it already preserves the `T40000 Q01` systems win without route-cache reuse
  - it recovers almost all of the full-warm benefit at the upper bank
- The exact lower-bank `T2500 Q01` floor is still a full-warm claim:
  - chart-only is close, but still slightly negative on confirm
  - route-only is nowhere near the crossover
- The hardware-side story therefore broadens, but only partially:
  - upper-bank operational wins survive outside the idealized fully warm state
  - the strict lower-bank single-query floor still depends on both caches

## Decision
- Close `INC-0093` confirm positive/explanatory.
- Promote chart residency as the dominant operational reuse surface on the
  fixed translated product stack.
- Keep `FULL_H4XH4_FIELD_A150_CPX8_Q01_T2500` as the exact lower-bank
  single-query floor reference.
- Keep `CHART_H4XH4_FIELD_A150_CPX8_Q01_T40000` as the first partial-residency
  upper-bank operational point.
- Treat route-only residency as insufficient on the fixed `Q01` anchors.
- Move next to a chart-resident / route-ephemeral repeat map on the same fixed
  operating points (`INC-0094`).
