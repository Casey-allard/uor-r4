# INC-0089: Product Phase Translation Warm Cache Q01 2500-2600 Refine

## Status
Confirm completed positive/narrow on 2026-03-12.

## Trigger
`INC-0087` confirmed that the fixed translated product stack already crosses
dense at `Q01` by `max_train=2600`, while `INC-0088` explained the local
`T2650` miss as a systems-cost pocket rather than a route-law failure:
- `DENSE_Q01_T2600`: `top1=0.051923`, `amortized=0.1180s`
- `H4XH4_FIELD_A150_CPX8_Q01_T2600`: `top1=0.045577`,
  `cand_frac=0.193587`, `amortized=0.1159s`
- `DENSE_Q01_T2500`: `top1=0.052000`, `amortized=0.1103s`
- `H4XH4_FIELD_A150_CPX8_Q01_T2500`: `top1=0.044600`,
  `cand_frac=0.193328`, `amortized=0.1403s`

That left a final tracked onset gap inside `2500-2600`.

## Branch Contract
- keep the confirmed `INC-0065` route law fixed
- keep the confirmed `INC-0071` secondary-key law fixed
- keep the confirmed `INC-0083` persistent-cache implementation fixed
- keep warm-cache operation explicit
- use the `INC-0088` cost read as interpretation guardrails
- change only the bank ladder inside the remaining `2500-2600` gap
- probe only the minimal `Q01/Q02` repeat bracket needed to locate the next
  onset boundary
- do not reopen geometry, retrieval scoring, or any new rescue surface

## Evidence
- Prewarm screen config:
  - `configs/proxy_transfer_inc0089_product_phase_translation_warm_cache_q01_2500_2600_refine_prewarm_screen.json`
- Screen config:
  - `configs/proxy_transfer_inc0089_product_phase_translation_warm_cache_q01_2500_2600_refine_screen.json`
- Prewarm confirm config:
  - `configs/proxy_transfer_inc0089_product_phase_translation_warm_cache_q01_2500_2600_refine_prewarm_confirm.json`
- Confirm config:
  - `configs/proxy_transfer_inc0089_product_phase_translation_warm_cache_q01_2500_2600_refine_confirm.json`
- Screen analysis:
  - `results/analysis/inc0089_product_phase_translation_warm_cache_q01_2500_2600_refine_screen.json`
- Confirm analysis:
  - `results/analysis/inc0089_product_phase_translation_warm_cache_q01_2500_2600_refine_confirm.json`
- Screen profile:
  - `results/analysis/inc0089_product_phase_translation_warm_cache_q01_2500_2600_refine_screen_profile.json`
- Confirm profile:
  - `results/analysis/inc0089_product_phase_translation_warm_cache_q01_2500_2600_refine_confirm_profile.json`
- Reports:
  - `docs/reports/INC0089_PRODUCT_PHASE_TRANSLATION_WARM_CACHE_Q01_2500_2600_REFINE_SCREEN.md`
  - `docs/reports/INC0089_PRODUCT_PHASE_TRANSLATION_WARM_CACHE_Q01_2500_2600_REFINE_CONFIRM.md`
- Gate notes:
  - `docs/governance/gates/gate_20260312_054028.md`
  - `docs/governance/gates/gate_20260312_054048.md`
  - `docs/governance/gates/gate_20260312_054156.md`
  - `docs/governance/gates/gate_20260312_054233.md`

## Screen Read
- The tracked 2-seed screen suggested that the floor might move again.
- `T2525 Q01`
  - `DENSE_Q01_T2525`: `top1=0.051902`, `amortized=0.1383s`
  - `H4XH4_FIELD_A150_CPX8_Q01_T2525`: `top1=0.045959`,
    `cand_frac=0.188632`, `amortized=0.0991s`
- `T2550 Q01`
  - `DENSE_Q01_T2550`: `top1=0.052157`, `amortized=0.0908s`
  - `H4XH4_FIELD_A150_CPX8_Q01_T2550`: `top1=0.045882`,
    `cand_frac=0.187976`, `amortized=0.0595s`
- `T2575 Q01`
  - `DENSE_Q01_T2575`: `top1=0.052059`, `amortized=0.1155s`
  - `H4XH4_FIELD_A150_CPX8_Q01_T2575`: `top1=0.045455`,
    `cand_frac=0.187734`, `amortized=0.1697s`
- Every routed screen run hit both caches exactly:
  - `chart_cache_hit=1.0`
  - `route_cache_hit=1.0`

## Confirm Read
- The 4-seed confirm cleaned up the remaining local ambiguity instead of
  preserving another pocket.
- `T2525 Q01`
  - `DENSE_Q01_T2525`: `top1=0.051902`, `amortized=0.1787s`
  - `H4XH4_FIELD_A150_CPX8_Q01_T2525`: `top1=0.045166`,
    `cand_frac=0.193195`, `amortized=0.0643s`
  - amortized margin vs dense: `+0.1145s`
- `T2550 Q01`
  - `DENSE_Q01_T2550`: `top1=0.051373`, `amortized=0.1225s`
  - `H4XH4_FIELD_A150_CPX8_Q01_T2550`: `top1=0.044902`,
    `cand_frac=0.192997`, `amortized=0.0563s`
  - amortized margin vs dense: `+0.0662s`
- `T2575 Q01`
  - `DENSE_Q01_T2575`: `top1=0.051476`, `amortized=0.1304s`
  - `H4XH4_FIELD_A150_CPX8_Q01_T2575`: `top1=0.044872`,
    `cand_frac=0.192759`, `amortized=0.0544s`
  - amortized margin vs dense: `+0.0761s`
- Every routed confirm run kept exact warm-cache hits:
  - `chart_cache_hit=1.0`
  - `route_cache_hit=1.0`

## Reading
- The fixed translated product stack now has a tracked confirmed warm-cache
  `Q01` crossover by `T2525`.
- The earliest tracked and confirmed warm-cache single-query crossover point is
  now:
  - `H4XH4_FIELD_A150_CPX8_Q01_T2525`
- The earlier `T2650` anomaly does not block the lower boundary from moving
  again once the bracket is re-centered inside `2500-2600`.
- Search work stayed pinned near `19%` of dense across the whole refined band:
  - `T2525`: `0.193195`
  - `T2550`: `0.192997`
  - `T2575`: `0.192759`

## Decision
- Close `INC-0089` confirm positive/narrow.
- Promote `H4XH4_FIELD_A150_CPX8_Q01_T2525` as the earliest tracked confirmed
  warm-cache single-query crossover point on the fixed translated stack.
- Keep `H4XH4_FIELD_A150_CPX8_Q02_T2500` as the earliest tracked confirmed
  warm-cache crossover point at any repeat count.
- Keep the route law, secondary-key law, and cache implementation fixed.
- Move next to a final threshold refinement inside `2500-2525`.

## Resume Note
Resume from the `INC-0089` confirm artifacts and treat the next branch as a
final warm-cache `Q01` threshold refinement inside `2500-2525`.
