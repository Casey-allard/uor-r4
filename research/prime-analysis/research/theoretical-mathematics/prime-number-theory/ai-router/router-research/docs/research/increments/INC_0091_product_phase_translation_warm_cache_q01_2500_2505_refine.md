# INC-0091: Product Phase Translation Warm Cache Q01 2500-2505 Refine

## Status
Completed at confirm stage and closed positive/narrow.

## Trigger
`INC-0090` confirmed that the fixed translated product stack already crosses
dense at `Q01` by `max_train=2505`, while `max_train=2500` still misses at
`Q01` and only crosses at `Q02`:
- `DENSE_Q01_T2505`: `top1=0.052117`, `amortized=0.1291s`
- `H4XH4_FIELD_A150_CPX8_Q01_T2505`: `top1=0.044728`,
  `cand_frac=0.193544`, `amortized=0.0579s`
- `DENSE_Q01_T2500`: `top1=0.052000`, `amortized=0.1103s`
- `H4XH4_FIELD_A150_CPX8_Q01_T2500`: `top1=0.044600`,
  `cand_frac=0.193328`, `amortized=0.1403s`

That leaves one final tracked onset gap inside `2500-2505`.

## Screen Outcome
- The tracked 2-seed warm-cache screen stayed positive across the full
  `2501/2502/2503/2504` bracket at `Q01`.
- Key screen read:
  - `DENSE_Q01_T2501`: `top1=0.051600`, `amortized=0.1328s`
  - `H4XH4_FIELD_A150_CPX8_Q01_T2501`: `top1=0.044400`,
    `cand_frac=0.189033`, `amortized=0.0840s`
  - `DENSE_Q01_T2504`: `top1=0.051118`, `amortized=0.0846s`
  - `H4XH4_FIELD_A150_CPX8_Q01_T2504`: `top1=0.044329`,
    `cand_frac=0.189397`, `amortized=0.0661s`
- The only noisy screen pocket was `T2503 Q02`, which went locally negative,
  but the single-query onset read remained coherent enough to carry the same
  bank bracket to confirm.

## Confirm Outcome
- The tracked 4-seed warm-cache confirm also stayed positive across the full
  `2501/2502/2503/2504` bracket at `Q01`.
- Key confirm read:
  - `DENSE_Q01_T2501`: `top1=0.051800`, `amortized=0.1364s`
  - `H4XH4_FIELD_A150_CPX8_Q01_T2501`: `top1=0.044600`,
    `cand_frac=0.193338`, `amortized=0.0779s`
  - `DENSE_Q01_T2502`: `top1=0.052158`, `amortized=0.1212s`
  - `H4XH4_FIELD_A150_CPX8_Q01_T2502`: `top1=0.044764`,
    `cand_frac=0.193503`, `amortized=0.0827s`
  - `DENSE_Q01_T2503`: `top1=0.051958`, `amortized=0.1126s`
  - `H4XH4_FIELD_A150_CPX8_Q01_T2503`: `top1=0.044764`,
    `cand_frac=0.193513`, `amortized=0.0644s`
  - `DENSE_Q01_T2504`: `top1=0.051917`, `amortized=0.1324s`
  - `H4XH4_FIELD_A150_CPX8_Q01_T2504`: `top1=0.044728`,
    `cand_frac=0.193565`, `amortized=0.0592s`
- Every confirmed bank now has first systems crossover at `Q01`, and the
  secondary-key search-work ratio stays pinned near `0.193-0.194`.
- The screen-only `T2503 Q02` pocket disappeared on confirm.
- All routed warm runs kept exact persisted-bank reuse:
  - `chart_cache_hit=1.0`
  - `route_cache_hit=1.0`

## Reading
- `INC-0091` closes the lower-bank threshold search at integer resolution.
- `T2500` still misses at `Q01`, while `T2501` now survives on the tracked
  4-seed confirm, so `H4XH4_FIELD_A150_CPX8_Q01_T2501` is the earliest tracked
  confirmed warm-cache single-query crossover point.
- `H4XH4_FIELD_A150_CPX8_Q02_T2500` remains the earliest tracked confirmed
  warm-cache crossover at any repeat count.
- The next honest move is to harden the exact `T2501` floor as a systems
  result, not to keep shaving `T`.

## Artifacts
- Configs:
  - `configs/proxy_transfer_inc0091_product_phase_translation_warm_cache_q01_2500_2505_refine_prewarm_screen.json`
  - `configs/proxy_transfer_inc0091_product_phase_translation_warm_cache_q01_2500_2505_refine_screen.json`
  - `configs/proxy_transfer_inc0091_product_phase_translation_warm_cache_q01_2500_2505_refine_prewarm_confirm.json`
  - `configs/proxy_transfer_inc0091_product_phase_translation_warm_cache_q01_2500_2505_refine_confirm.json`
- Analyses:
  - `results/analysis/inc0091_product_phase_translation_warm_cache_q01_2500_2505_refine_prewarm_screen.json`
  - `results/analysis/inc0091_product_phase_translation_warm_cache_q01_2500_2505_refine_screen.json`
  - `results/analysis/inc0091_product_phase_translation_warm_cache_q01_2500_2505_refine_prewarm_confirm.json`
  - `results/analysis/inc0091_product_phase_translation_warm_cache_q01_2500_2505_refine_confirm.json`
  - `results/analysis/inc0091_product_phase_translation_warm_cache_q01_2500_2505_refine_screen_profile.json`
  - `results/analysis/inc0091_product_phase_translation_warm_cache_q01_2500_2505_refine_confirm_profile.json`
- Reports:
  - `docs/reports/INC0091_PRODUCT_PHASE_TRANSLATION_WARM_CACHE_Q01_2500_2505_REFINE_SCREEN.md`
  - `docs/reports/INC0091_PRODUCT_PHASE_TRANSLATION_WARM_CACHE_Q01_2500_2505_REFINE_CONFIRM.md`
- Gate notes:
  - `docs/governance/gates/gate_20260312_062958.md`
  - `docs/governance/gates/gate_20260312_063025.md`
  - `docs/governance/gates/gate_20260312_063141.md`
  - `docs/governance/gates/gate_20260312_063230.md`

## Branch Contract
- keep the confirmed `INC-0065` route law fixed
- keep the confirmed `INC-0071` secondary-key law fixed
- keep the confirmed `INC-0083` persistent-cache implementation fixed
- keep warm-cache operation explicit
- keep the `INC-0088` local cost audit as interpretation guardrails
- change only the bank ladder inside the remaining `2500-2505` gap
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
3. Search only the remaining threshold interval between the now-confirmed
   endpoints:
   - `T2500` negative at `Q01`
   - `T2505` positive at `Q01`
4. Promote a new earliest `Q01` onset only if the tracked sweep, not an ad hoc
   pilot, closes the gap.

## Stop Rule
- If no tracked bank between `2500` and `2505` crosses at `Q01`, keep `T2505`
  as the earliest tracked confirmed warm-cache single-query onset.
- If the refined bracket exposes another local systems pocket, explain it with
  cost composition before widening the search again.

## Resume Note
Resume from the `INC-0091` confirm artifact and treat this branch as the
resolved integer lower-bound close-out: `T2500` misses at `Q01`, `T2501`
survives at `Q01`, and the next branch should harden that exact floor rather
than refine `T` further.
