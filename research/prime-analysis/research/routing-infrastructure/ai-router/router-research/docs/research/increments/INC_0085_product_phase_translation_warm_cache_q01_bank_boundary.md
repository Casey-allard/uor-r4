# INC-0085: Product Phase Translation Warm Cache Q01 Bank Boundary

## Status
Confirm completed positive/narrow on 2026-03-12.

## Trigger
`INC-0084` confirmed that the fixed translated product stack crosses dense
exact retrieval all the way down to `Q01` under warm-cache conditions at
`max_train=40000`:
- `DENSE_Q01_T40000`: `top1=0.048850`, `amortized=9.536s`
- `H4XH4_FIELD_A150_CPX8_Q01_T40000`: `top1=0.047325`,
  `cand_frac=0.183764`, `amortized=2.204s`

The next honest question was how early in bank size that warm-cache
single-query crossover begins on the fixed translated stack.

## Branch Contract
- keep the confirmed `INC-0065` route law fixed
- keep the confirmed `INC-0071` secondary-key law fixed
- keep the confirmed `INC-0083` persistent-cache implementation fixed
- keep warm-cache operation explicit
- change only the bank-size bracket and the minimal `Q01/Q02` repeat bracket
  needed to locate the onset
- do not reopen geometry, retrieval scoring, or any new rescue surface

## Evidence
- Prewarm screen config:
  - `configs/proxy_transfer_inc0085_product_phase_translation_warm_cache_q01_bank_boundary_prewarm_screen.json`
- Screen config:
  - `configs/proxy_transfer_inc0085_product_phase_translation_warm_cache_q01_bank_boundary_screen.json`
- Prewarm confirm config:
  - `configs/proxy_transfer_inc0085_product_phase_translation_warm_cache_q01_bank_boundary_prewarm_confirm.json`
- Confirm config:
  - `configs/proxy_transfer_inc0085_product_phase_translation_warm_cache_q01_bank_boundary_confirm.json`
- Screen analysis:
  - `results/analysis/inc0085_product_phase_translation_warm_cache_q01_bank_boundary_screen.json`
- Confirm analysis:
  - `results/analysis/inc0085_product_phase_translation_warm_cache_q01_bank_boundary_confirm.json`
- Screen profile:
  - `results/analysis/inc0085_product_phase_translation_warm_cache_q01_bank_boundary_screen_profile.json`
- Confirm profile:
  - `results/analysis/inc0085_product_phase_translation_warm_cache_q01_bank_boundary_confirm_profile.json`
- Reports:
  - `docs/reports/INC0085_PRODUCT_PHASE_TRANSLATION_WARM_CACHE_Q01_BANK_BOUNDARY_SCREEN.md`
  - `docs/reports/INC0085_PRODUCT_PHASE_TRANSLATION_WARM_CACHE_Q01_BANK_BOUNDARY_CONFIRM.md`
- Gate notes:
  - `docs/governance/gates/gate_20260312_045413.md`
  - `docs/governance/gates/gate_20260312_045437.md`
  - `docs/governance/gates/gate_20260312_045613.md`
  - `docs/governance/gates/gate_20260312_045657.md`

## Screen Read
- The tracked 2-seed screen already showed crossover at every tested bank in
  the narrowed ladder.
- `T3000 Q01`
  - `DENSE_Q01_T3000`: `top1=0.048000`, `amortized=0.1567s`
  - `H4XH4_FIELD_A150_CPX8_Q01_T3000`: `top1=0.044000`,
    `cand_frac=0.185411`, `amortized=0.0780s`
- `T4500 Q01`
  - `DENSE_Q01_T4500`: `top1=0.047333`, `amortized=0.2543s`
  - `H4XH4_FIELD_A150_CPX8_Q01_T4500`: `top1=0.047111`,
    `cand_frac=0.189224`, `amortized=0.1687s`
- `T6000 Q01`
  - `DENSE_Q01_T6000`: `top1=0.049667`, `amortized=0.5086s`
  - `H4XH4_FIELD_A150_CPX8_Q01_T6000`: `top1=0.047667`,
    `cand_frac=0.190455`, `amortized=0.1581s`
- All routed screen runs hit both caches exactly:
  - `chart_cache_hit=1.0`
  - `route_cache_hit=1.0`

## Confirm Read
- The 4-seed confirm preserved the same lower-bound result.
- `T3000 Q01`
  - `DENSE_Q01_T3000`: `top1=0.049833`, `amortized=0.1592s`
  - `H4XH4_FIELD_A150_CPX8_Q01_T3000`: `top1=0.044833`,
    `cand_frac=0.191704`, `amortized=0.0744s`
  - amortized margin vs dense: `+0.0849s`
- `T4500 Q01`
  - `DENSE_Q01_T4500`: `top1=0.045889`, `amortized=0.2826s`
  - `H4XH4_FIELD_A150_CPX8_Q01_T4500`: `top1=0.047333`,
    `cand_frac=0.193020`, `amortized=0.1877s`
  - amortized margin vs dense: `+0.0949s`
- `T6000 Q01`
  - `DENSE_Q01_T6000`: `top1=0.048667`, `amortized=0.3956s`
  - `H4XH4_FIELD_A150_CPX8_Q01_T6000`: `top1=0.047083`,
    `cand_frac=0.187229`, `amortized=0.2047s`
  - amortized margin vs dense: `+0.1910s`
- `Q02` stayed positive at every tracked bank as well.
- Every routed confirm run kept exact warm-cache hits:
  - `chart_cache_hit=1.0`
  - `route_cache_hit=1.0`

## Reading
- The fixed translated product stack does not need the upper-bank `T40000`
  regime to reach a warm-cache single-query systems crossover.
- The earliest tracked and confirmed `Q01` warm-cache crossover point is now:
  - `H4XH4_FIELD_A150_CPX8_Q01_T3000`
- Search work stays pinned near `19%` of dense across the whole tracked bank
  band:
  - `T3000`: `0.191704`
  - `T4500`: `0.193020`
  - `T6000`: `0.187229`
- The next honest question is the lower-bound gap below `T3000`, not a
  higher-bank extension and not another rescue surface.

## Decision
- Close `INC-0085` confirm positive/narrow.
- Promote `H4XH4_FIELD_A150_CPX8_Q01_T3000` as the earliest tracked confirmed
  warm-cache single-query crossover point on the fixed translated stack.
- Keep the route law, secondary-key law, and cache implementation fixed.
- Move next to a lower-bound refinement below `T3000`.

## Resume Note
Resume from the `INC-0085` confirm artifacts and treat the next branch as a
lower-bound refinement below `T3000` on the fixed translated warm-cache stack.
