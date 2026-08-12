# INC-0087: Product Phase Translation Warm Cache Q01 Threshold Refine

## Status
Confirm completed positive/narrow on 2026-03-12.

## Trigger
`INC-0086` confirmed that the fixed translated product stack already crosses
dense at `Q01` by `max_train=2750`, while `max_train=2500` still misses at
`Q01` and only crosses at `Q02`:
- `DENSE_Q01_T2750`: `top1=0.050182`, `amortized=0.1084s`
- `H4XH4_FIELD_A150_CPX8_Q01_T2750`: `top1=0.044545`,
  `cand_frac=0.192894`, `amortized=0.0999s`
- `DENSE_Q01_T2500`: `top1=0.052000`, `amortized=0.1103s`
- `H4XH4_FIELD_A150_CPX8_Q01_T2500`: `top1=0.044600`,
  `cand_frac=0.193328`, `amortized=0.1403s`

That left a clean remaining threshold question inside the `2500-2750` gap.

## Branch Contract
- keep the confirmed `INC-0065` route law fixed
- keep the confirmed `INC-0071` secondary-key law fixed
- keep the confirmed `INC-0083` persistent-cache implementation fixed
- keep warm-cache operation explicit
- change only the bank ladder inside the remaining `2500-2750` gap
- probe only the minimal `Q01/Q02` repeat bracket needed to locate the next
  onset boundary
- do not reopen geometry, retrieval scoring, or any new rescue surface

## Evidence
- Prewarm screen config:
  - `configs/proxy_transfer_inc0087_product_phase_translation_warm_cache_q01_threshold_refine_prewarm_screen.json`
- Screen config:
  - `configs/proxy_transfer_inc0087_product_phase_translation_warm_cache_q01_threshold_refine_screen.json`
- Prewarm confirm config:
  - `configs/proxy_transfer_inc0087_product_phase_translation_warm_cache_q01_threshold_refine_prewarm_confirm.json`
- Confirm config:
  - `configs/proxy_transfer_inc0087_product_phase_translation_warm_cache_q01_threshold_refine_confirm.json`
- Screen analysis:
  - `results/analysis/inc0087_product_phase_translation_warm_cache_q01_threshold_refine_screen.json`
- Confirm analysis:
  - `results/analysis/inc0087_product_phase_translation_warm_cache_q01_threshold_refine_confirm.json`
- Screen profile:
  - `results/analysis/inc0087_product_phase_translation_warm_cache_q01_threshold_refine_screen_profile.json`
- Confirm profile:
  - `results/analysis/inc0087_product_phase_translation_warm_cache_q01_threshold_refine_confirm_profile.json`
- Reports:
  - `docs/reports/INC0087_PRODUCT_PHASE_TRANSLATION_WARM_CACHE_Q01_THRESHOLD_REFINE_SCREEN.md`
  - `docs/reports/INC0087_PRODUCT_PHASE_TRANSLATION_WARM_CACHE_Q01_THRESHOLD_REFINE_CONFIRM.md`
- Gate notes:
  - `docs/governance/gates/gate_20260312_052842.md`
  - `docs/governance/gates/gate_20260312_052915.md`
  - `docs/governance/gates/gate_20260312_053034.md`
  - `docs/governance/gates/gate_20260312_053113.md`

## Screen Read
- The tracked 2-seed screen suggested that the threshold might move below
  `T2750` immediately.
- `T2600 Q01`
  - `DENSE_Q01_T2600`: `top1=0.052692`, `amortized=0.1180s`
  - `H4XH4_FIELD_A150_CPX8_Q01_T2600`: `top1=0.046154`,
    `cand_frac=0.188787`, `amortized=0.0983s`
- `T2650 Q01`
  - `DENSE_Q01_T2650`: `top1=0.051698`, `amortized=0.0885s`
  - `H4XH4_FIELD_A150_CPX8_Q01_T2650`: `top1=0.045660`,
    `cand_frac=0.189479`, `amortized=0.0644s`
- `T2700 Q01`
  - `DENSE_Q01_T2700`: `top1=0.052222`, `amortized=0.1672s`
  - `H4XH4_FIELD_A150_CPX8_Q01_T2700`: `top1=0.044815`,
    `cand_frac=0.187953`, `amortized=0.1310s`
- Every routed screen run hit both caches exactly:
  - `chart_cache_hit=1.0`
  - `route_cache_hit=1.0`

## Confirm Read
- The 4-seed confirm moved the onset earlier again, but it also exposed a new
  local non-monotone pocket.
- `T2600 Q01`
  - `DENSE_Q01_T2600`: `top1=0.051923`, `amortized=0.1180s`
  - `H4XH4_FIELD_A150_CPX8_Q01_T2600`: `top1=0.045577`,
    `cand_frac=0.193587`, `amortized=0.1159s`
  - amortized margin vs dense: `+0.0022s`
- `T2650 Q01`
  - `DENSE_Q01_T2650`: `top1=0.051132`, `amortized=0.0970s`
  - `H4XH4_FIELD_A150_CPX8_Q01_T2650`: `top1=0.044528`,
    `cand_frac=0.193916`, `amortized=0.1143s`
  - amortized margin vs dense: `-0.0173s`
- `T2650 Q02`
  - `DENSE_Q02_T2650`: `top1=0.051132`, `amortized=0.1178s`
  - `H4XH4_FIELD_A150_CPX8_Q02_T2650`: `top1=0.044528`,
    `cand_frac=0.193916`, `amortized=0.0910s`
  - amortized margin vs dense: `+0.0268s`
- `T2700 Q01`
  - `DENSE_Q01_T2700`: `top1=0.051852`, `amortized=0.1217s`
  - `H4XH4_FIELD_A150_CPX8_Q01_T2700`: `top1=0.044259`,
    `cand_frac=0.193139`, `amortized=0.0850s`
  - amortized margin vs dense: `+0.0367s`
- Every routed confirm run kept exact warm-cache hits:
  - `chart_cache_hit=1.0`
  - `route_cache_hit=1.0`

## Reading
- The fixed translated product stack does reach a tracked confirmed `Q01`
  crossover by `T2600`.
- The earliest tracked and confirmed warm-cache single-query crossover point is
  now:
  - `H4XH4_FIELD_A150_CPX8_Q01_T2600`
- The local threshold is not monotone inside the refined band:
  - `T2600`: positive at `Q01`
  - `T2650`: misses at `Q01`, crosses at `Q02`
  - `T2700`: positive at `Q01`
- Search work stayed pinned near `19%` of dense across the whole refined band:
  - `T2600`: `0.193587`
  - `T2650`: `0.193916`
  - `T2700`: `0.193139`
- That means the new split should be read as a local systems-threshold question,
  not as a geometric routing collapse.

## Decision
- Close `INC-0087` confirm positive/narrow.
- Promote `H4XH4_FIELD_A150_CPX8_Q01_T2600` as the earliest tracked confirmed
  warm-cache single-query crossover point on the fixed translated stack.
- Record the `T2650` miss as a local non-monotone pocket that needs direct
  cost-accounting rather than another route-law change.
- Keep the route law, secondary-key law, and cache implementation fixed.
- Move next to a local cost audit around `T2600/T2650/T2700`.

## Resume Note
Resume from the `INC-0087` confirm artifacts and treat the next branch as a
local systems-cost audit inside the `2500-2700` threshold band.
