# INC-0092: Product Phase Translation Warm Cache Q01 Floor Hardening

## Status
Completed at confirm stage and closed positive/explanatory.

## Trigger
`INC-0091` closes the tracked warm-cache single-query lower-bound search at
integer resolution:
- `DENSE_Q01_T2500`: `top1=0.052000`, `amortized=0.1103s`
- `H4XH4_FIELD_A150_CPX8_Q01_T2500`: `top1=0.044600`,
  `cand_frac=0.193328`, `amortized=0.1403s`
- `DENSE_Q01_T2501`: `top1=0.051800`, `amortized=0.1364s`
- `H4XH4_FIELD_A150_CPX8_Q01_T2501`: `top1=0.044600`,
  `cand_frac=0.193338`, `amortized=0.0779s`

That closes the integer bank gap. The next honest question is whether the exact
`T2501` warm-cache `Q01` floor is robust enough to treat as a stable systems
claim, not whether it can move one more unit earlier.

## Screen Outcome
- The tracked 4-seed warm-cache hardening screen already falsified the old
  exact-floor separation.
- Key screen read:
  - `DENSE_Q01_T2500`: `amortized=0.1412s`
  - `H4XH4_FIELD_A150_CPX8_Q01_T2500`: `cand_frac=0.189829`,
    `amortized=0.0771s`
  - `DENSE_Q01_T2501`: `amortized=0.0940s`
  - `H4XH4_FIELD_A150_CPX8_Q01_T2501`: `cand_frac=0.189837`,
    `amortized=0.0800s`
- Both `T2500` and `T2501` screened positive at `Q01`, so the hardening branch
  immediately became a boundary-correction test rather than a boundary-defense
  test.

## Confirm Outcome
- The tracked 8-seed warm-cache confirm keeps the same corrected read:
  - `DENSE_Q01_T2500`: `top1=0.050300`, `amortized=0.1078s`
  - `H4XH4_FIELD_A150_CPX8_Q01_T2500`: `top1=0.046300`,
    `cand_frac=0.198723`, `amortized=0.0741s`
  - `DENSE_Q01_T2501`: `top1=0.050300`, `amortized=0.1225s`
  - `H4XH4_FIELD_A150_CPX8_Q01_T2501`: `top1=0.046300`,
    `cand_frac=0.198731`, `amortized=0.0638s`
- `T2500` and `T2501` both now have first systems crossover at `Q01` on the
  expanded seed schedule.
- The routed search-work ratio stays pinned near `0.199` of dense on both
  banks.
- All routed warm runs kept exact persisted-bank reuse:
  - `chart_cache_hit=1.0`
  - `route_cache_hit=1.0`

## Reading
- `INC-0092` does not support the old exact `T2500` miss / `T2501` hit story.
- Under stronger hardening, the stable lower-bank warm-cache single-query floor
  collapses to `T2500`, not `T2501`.
- That means the lower-bound search is now fully closed:
  - earliest tracked confirmed warm-cache `Q01` crossover point =
    `H4XH4_FIELD_A150_CPX8_Q01_T2500`
  - earliest tracked confirmed warm-cache crossover at any repeat count =
    `H4XH4_FIELD_A150_CPX8_Q01_T2500`
- The next honest question is workload/cache-residency robustness, not more
  `T/Q` threshold search.

## Artifacts
- Configs:
  - `configs/proxy_transfer_inc0092_product_phase_translation_warm_cache_q01_floor_hardening_prewarm_screen.json`
  - `configs/proxy_transfer_inc0092_product_phase_translation_warm_cache_q01_floor_hardening_screen.json`
  - `configs/proxy_transfer_inc0092_product_phase_translation_warm_cache_q01_floor_hardening_prewarm_confirm.json`
  - `configs/proxy_transfer_inc0092_product_phase_translation_warm_cache_q01_floor_hardening_confirm.json`
- Analyses:
  - `results/analysis/inc0092_product_phase_translation_warm_cache_q01_floor_hardening_prewarm_screen.json`
  - `results/analysis/inc0092_product_phase_translation_warm_cache_q01_floor_hardening_screen.json`
  - `results/analysis/inc0092_product_phase_translation_warm_cache_q01_floor_hardening_prewarm_confirm.json`
  - `results/analysis/inc0092_product_phase_translation_warm_cache_q01_floor_hardening_confirm.json`
  - `results/analysis/inc0092_product_phase_translation_warm_cache_q01_floor_hardening_screen_profile.json`
  - `results/analysis/inc0092_product_phase_translation_warm_cache_q01_floor_hardening_confirm_profile.json`
- Reports:
  - `docs/reports/INC0092_PRODUCT_PHASE_TRANSLATION_WARM_CACHE_Q01_FLOOR_HARDENING_SCREEN.md`
  - `docs/reports/INC0092_PRODUCT_PHASE_TRANSLATION_WARM_CACHE_Q01_FLOOR_HARDENING_CONFIRM.md`
- Gate notes:
  - `docs/governance/gates/gate_20260312_064253.md`
  - `docs/governance/gates/gate_20260312_064320.md`
  - `docs/governance/gates/gate_20260312_064442.md`
  - `docs/governance/gates/gate_20260312_064531.md`

## Branch Contract
- keep the confirmed `INC-0065` route law fixed
- keep the confirmed `INC-0071` secondary-key law fixed
- keep the confirmed `INC-0083` persistent-cache implementation fixed
- keep the exact integer lower-bound bracket explicit:
  - `T2500`
  - `T2501`
- keep warm-cache operation explicit
- do not reopen geometry, retrieval scoring, or any new rescue surface
- do not widen the bank ladder again unless the hardening pass falsifies the
  current floor

## Minimal Scope
1. Carry only:
   - `DENSE`
   - `H4XH4_FIELD_A150_CPX8`
2. Probe only:
   - `Q01`
   - `Q02`
3. Harden only the exact integer floor bracket:
   - `T2500`
   - `T2501`
4. Increase confirm strength rather than bank breadth:
   - use an expanded seed schedule
   - keep the rest of the translated stack fixed

## Stop Rule
- If `T2501 Q01` collapses under hardening, demote the exact floor and reopen a
  narrow boundary explanation.
- If `T2501 Q01` survives while `T2500 Q01` still misses, close the lower-bound
  search and move on from `T/Q` refinement to the next hardware-side branch.

## Resume Note
Resume from the `INC-0092` confirm artifact and treat this branch as the
hardening correction that collapses the lower-bank warm-cache `Q01` floor to
`T2500`.
