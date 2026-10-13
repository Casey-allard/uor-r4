# INC-0090: Product Phase Translation Warm Cache Q01 2500-2525 Refine

## Status
Completed at confirm stage and closed positive/narrow.

## Trigger
`INC-0089` confirmed that the fixed translated product stack already crosses
dense at `Q01` by `max_train=2525`, while `max_train=2500` still misses at
`Q01` and only crosses at `Q02`:
- `DENSE_Q01_T2525`: `top1=0.051902`, `amortized=0.1787s`
- `H4XH4_FIELD_A150_CPX8_Q01_T2525`: `top1=0.045166`,
  `cand_frac=0.193195`, `amortized=0.0643s`
- `DENSE_Q01_T2500`: `top1=0.052000`, `amortized=0.1103s`
- `H4XH4_FIELD_A150_CPX8_Q01_T2500`: `top1=0.044600`,
  `cand_frac=0.193328`, `amortized=0.1403s`

That leaves a final tracked onset gap inside `2500-2525`.

## Screen Outcome
- The tracked 2-seed warm-cache screen stayed positive across the full
  `2505/2510/2515/2520` bracket at `Q01`.
- Key screen read:
  - `DENSE_Q01_T2505`: `top1=0.051518`, `amortized=0.1241s`
  - `H4XH4_FIELD_A150_CPX8_Q01_T2505`: `top1=0.044329`,
    `cand_frac=0.189414`, `amortized=0.0525s`
  - `DENSE_Q01_T2520`: `top1=0.051190`, `amortized=0.1324s`
  - `H4XH4_FIELD_A150_CPX8_Q01_T2520`: `top1=0.044841`,
    `cand_frac=0.188411`, `amortized=0.0557s`
- The only weak screen pocket was `T2520 Q02`, which slipped slightly negative,
  but that did not change the single-query onset read, so the exact same bank
  bracket carried to confirm.

## Confirm Outcome
- The tracked 4-seed warm-cache confirm also stayed positive across the full
  `2505/2510/2515/2520` bracket at `Q01`.
- Key confirm read:
  - `DENSE_Q01_T2505`: `top1=0.052117`, `amortized=0.1291s`
  - `H4XH4_FIELD_A150_CPX8_Q01_T2505`: `top1=0.044728`,
    `cand_frac=0.193544`, `amortized=0.0579s`
  - `DENSE_Q01_T2510`: `top1=0.051594`, `amortized=0.0834s`
  - `H4XH4_FIELD_A150_CPX8_Q01_T2510`: `top1=0.044422`,
    `cand_frac=0.193187`, `amortized=0.0544s`
  - `DENSE_Q01_T2515`: `top1=0.051909`, `amortized=0.0890s`
  - `H4XH4_FIELD_A150_CPX8_Q01_T2515`: `top1=0.044749`,
    `cand_frac=0.192989`, `amortized=0.0559s`
  - `DENSE_Q01_T2520`: `top1=0.051587`, `amortized=0.1010s`
  - `H4XH4_FIELD_A150_CPX8_Q01_T2520`: `top1=0.044841`,
    `cand_frac=0.193024`, `amortized=0.0581s`
- Every confirmed bank now has first systems crossover at `Q01`, and the
  secondary-key search-work ratio stays pinned near `0.193`.
- All routed warm runs kept exact persisted-bank reuse:
  - `chart_cache_hit=1.0`
  - `route_cache_hit=1.0`

## Reading
- `INC-0090` moves the earliest tracked confirmed warm-cache single-query
  crossover point from `T2525` down to `T2505`.
- `H4XH4_FIELD_A150_CPX8_Q02_T2500` still remains the earliest tracked
  confirmed warm-cache crossover at any repeat count.
- The remaining honest threshold question is now only the final `2500-2505`
  gap, not another route-law, retrieval, or cache rescue surface.

## Artifacts
- Configs:
  - `configs/proxy_transfer_inc0090_product_phase_translation_warm_cache_q01_2500_2525_refine_prewarm_screen.json`
  - `configs/proxy_transfer_inc0090_product_phase_translation_warm_cache_q01_2500_2525_refine_screen.json`
  - `configs/proxy_transfer_inc0090_product_phase_translation_warm_cache_q01_2500_2525_refine_prewarm_confirm.json`
  - `configs/proxy_transfer_inc0090_product_phase_translation_warm_cache_q01_2500_2525_refine_confirm.json`
- Analyses:
  - `results/analysis/inc0090_product_phase_translation_warm_cache_q01_2500_2525_refine_prewarm_screen.json`
  - `results/analysis/inc0090_product_phase_translation_warm_cache_q01_2500_2525_refine_screen.json`
  - `results/analysis/inc0090_product_phase_translation_warm_cache_q01_2500_2525_refine_prewarm_confirm.json`
  - `results/analysis/inc0090_product_phase_translation_warm_cache_q01_2500_2525_refine_confirm.json`
  - `results/analysis/inc0090_product_phase_translation_warm_cache_q01_2500_2525_refine_screen_profile.json`
  - `results/analysis/inc0090_product_phase_translation_warm_cache_q01_2500_2525_refine_confirm_profile.json`
- Reports:
  - `docs/reports/INC0090_PRODUCT_PHASE_TRANSLATION_WARM_CACHE_Q01_2500_2525_REFINE_SCREEN.md`
  - `docs/reports/INC0090_PRODUCT_PHASE_TRANSLATION_WARM_CACHE_Q01_2500_2525_REFINE_CONFIRM.md`
- Gate notes:
  - `docs/governance/gates/gate_20260312_060106.md`
  - `docs/governance/gates/gate_20260312_060131.md`
  - `docs/governance/gates/gate_20260312_060246.md`
  - `docs/governance/gates/gate_20260312_060331.md`

## Branch Contract
- keep the confirmed `INC-0065` route law fixed
- keep the confirmed `INC-0071` secondary-key law fixed
- keep the confirmed `INC-0083` persistent-cache implementation fixed
- keep warm-cache operation explicit
- keep the `INC-0088` local cost audit as interpretation guardrails
- change only the bank ladder inside the remaining `2500-2525` gap
- probe only the minimal `Q01/Q02` repeat bracket needed to locate the next
  onset boundary
- do not reopen geometry, retrieval scoring, or any new rescue surface

## Minimal Scope
1. Carry only:
   - `DENSE`
   - `H4XH4_FIELD_A150_CPX8`
2. Probe only:
   - `Q01`
   - `Q02`
3. Search only the threshold interval between the now-confirmed endpoints:
   - `T2500` negative at `Q01`
   - `T2525` positive at `Q01`
4. Promote a new earliest `Q01` onset only if the tracked sweep, not an ad hoc
   pilot, closes the gap.

## Stop Rule
- If no tracked bank between `2500` and `2525` crosses at `Q01`, keep `T2525`
  as the earliest tracked confirmed warm-cache single-query onset.
- If the refined bracket exposes another local systems pocket, explain it with
  cost composition before widening the search again.

## Resume Note
Resume from the `INC-0090` confirm artifact and treat this branch as the
resolved `2500-2525` close-out: `T2505` is now the earliest tracked confirmed
warm-cache `Q01` onset, and the only remaining threshold gap is `2500-2505`.
