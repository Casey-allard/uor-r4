# INC-0086: Product Phase Translation Warm Cache Q01 Lower Boundary Refine

## Status
Confirm completed positive/narrow on 2026-03-12.

## Trigger
`INC-0085` confirmed that the fixed translated product stack already crosses
dense at `Q01` under warm-cache conditions by `max_train=3000`:
- `DENSE_Q01_T3000`: `top1=0.049833`, `amortized=0.1592s`
- `H4XH4_FIELD_A150_CPX8_Q01_T3000`: `top1=0.044833`,
  `cand_frac=0.191704`, `amortized=0.0744s`

That made `T3000` the earliest tracked confirmed warm-cache onset. The next
honest question was whether the same fixed stack already crosses somewhere
below `T3000`.

## Branch Contract
- keep the confirmed `INC-0065` route law fixed
- keep the confirmed `INC-0071` secondary-key law fixed
- keep the confirmed `INC-0083` persistent-cache implementation fixed
- keep warm-cache operation explicit
- change only the lower bank ladder below `T3000`
- do not reopen geometry, scoring, or any new rescue surface

## Evidence
- Prewarm screen config:
  - `configs/proxy_transfer_inc0086_product_phase_translation_warm_cache_q01_lower_boundary_refine_prewarm_screen.json`
- Screen config:
  - `configs/proxy_transfer_inc0086_product_phase_translation_warm_cache_q01_lower_boundary_refine_screen.json`
- Prewarm confirm config:
  - `configs/proxy_transfer_inc0086_product_phase_translation_warm_cache_q01_lower_boundary_refine_prewarm_confirm.json`
- Confirm config:
  - `configs/proxy_transfer_inc0086_product_phase_translation_warm_cache_q01_lower_boundary_refine_confirm.json`
- Screen analysis:
  - `results/analysis/inc0086_product_phase_translation_warm_cache_q01_lower_boundary_refine_screen.json`
- Confirm analysis:
  - `results/analysis/inc0086_product_phase_translation_warm_cache_q01_lower_boundary_refine_confirm.json`
- Screen profile:
  - `results/analysis/inc0086_product_phase_translation_warm_cache_q01_lower_boundary_refine_screen_profile.json`
- Confirm profile:
  - `results/analysis/inc0086_product_phase_translation_warm_cache_q01_lower_boundary_refine_confirm_profile.json`
- Reports:
  - `docs/reports/INC0086_PRODUCT_PHASE_TRANSLATION_WARM_CACHE_Q01_LOWER_BOUNDARY_REFINE_SCREEN.md`
  - `docs/reports/INC0086_PRODUCT_PHASE_TRANSLATION_WARM_CACHE_Q01_LOWER_BOUNDARY_REFINE_CONFIRM.md`
- Gate notes:
  - `docs/governance/gates/gate_20260312_051358.md`
  - `docs/governance/gates/gate_20260312_051419.md`
  - `docs/governance/gates/gate_20260312_051526.md`
  - `docs/governance/gates/gate_20260312_051652.md`

## Screen Read
- The tracked 2-seed screen came back stronger than the earlier ad hoc pilots.
- `T2500 Q01`
  - `DENSE_Q01_T2500`: `top1=0.051600`, `amortized=0.1131s`
  - `H4XH4_FIELD_A150_CPX8_Q01_T2500`: `top1=0.044400`,
    `cand_frac=0.189016`, `amortized=0.0564s`
- `T2750 Q01`
  - `DENSE_Q01_T2750`: `top1=0.051636`, `amortized=0.1165s`
  - `H4XH4_FIELD_A150_CPX8_Q01_T2750`: `top1=0.043636`,
    `cand_frac=0.187798`, `amortized=0.0645s`
- `T3000 Q01`
  - `DENSE_Q01_T3000`: `top1=0.048000`, `amortized=0.1570s`
  - `H4XH4_FIELD_A150_CPX8_Q01_T3000`: `top1=0.044000`,
    `cand_frac=0.185411`, `amortized=0.0785s`
- Every routed screen run hit both caches exactly:
  - `chart_cache_hit=1.0`
  - `route_cache_hit=1.0`

## Confirm Read
- The 4-seed confirm narrowed the story cleanly instead of preserving the full
  screen optimism.
- `T2500 Q01`
  - `DENSE_Q01_T2500`: `top1=0.052000`, `amortized=0.1103s`
  - `H4XH4_FIELD_A150_CPX8_Q01_T2500`: `top1=0.044600`,
    `cand_frac=0.193328`, `amortized=0.1403s`
  - amortized margin vs dense: `-0.0300s`
- `T2500 Q02`
  - `DENSE_Q02_T2500`: `top1=0.052000`, `amortized=0.1083s`
  - `H4XH4_FIELD_A150_CPX8_Q02_T2500`: `top1=0.044600`,
    `cand_frac=0.193328`, `amortized=0.0911s`
  - amortized margin vs dense: `+0.0172s`
- `T2750 Q01`
  - `DENSE_Q01_T2750`: `top1=0.050182`, `amortized=0.1084s`
  - `H4XH4_FIELD_A150_CPX8_Q01_T2750`: `top1=0.044545`,
    `cand_frac=0.192894`, `amortized=0.0999s`
  - amortized margin vs dense: `+0.0085s`
- `T3000 Q01`
  - `DENSE_Q01_T3000`: `top1=0.049833`, `amortized=0.1319s`
  - `H4XH4_FIELD_A150_CPX8_Q01_T3000`: `top1=0.044833`,
    `cand_frac=0.191704`, `amortized=0.0726s`
  - amortized margin vs dense: `+0.0593s`
- Every routed confirm run kept exact warm-cache hits:
  - `chart_cache_hit=1.0`
  - `route_cache_hit=1.0`

## Reading
- The fixed translated product stack does cross below `T3000`, but not all the
  way down to `T2500` at `Q01`.
- The earliest tracked and confirmed warm-cache single-query crossover point is
  now:
  - `H4XH4_FIELD_A150_CPX8_Q01_T2750`
- The earliest tracked and confirmed warm-cache any-repeat crossover point is
  now:
  - `H4XH4_FIELD_A150_CPX8_Q02_T2500`
- Search work stayed pinned near `19%` of dense across the whole refined bank
  band:
  - `T2500`: `0.193328`
  - `T2750`: `0.192894`
  - `T3000`: `0.191704`
- The next honest question is the remaining `T2500` to `T2750` threshold gap,
  not a wider bank sweep and not another retrieval rescue surface.

## Decision
- Close `INC-0086` confirm positive/narrow.
- Promote `H4XH4_FIELD_A150_CPX8_Q01_T2750` as the earliest tracked confirmed
  warm-cache single-query crossover point on the fixed translated stack.
- Record `H4XH4_FIELD_A150_CPX8_Q02_T2500` as the earliest tracked confirmed
  warm-cache crossover point at any repeat count.
- Keep the route law, secondary-key law, and cache implementation fixed.
- Move next to a threshold refinement inside the remaining `2500-2750` bank
  gap.

## Resume Note
Resume from the `INC-0086` confirm artifacts and treat the next branch as a
warm-cache `Q01` threshold refinement inside the `2500-2750` bank gap.
