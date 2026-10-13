# INC-0088: Product Phase Translation Warm Cache Q01 Local Cost Audit

## Status
Completed positive/explanatory on 2026-03-12.

## Trigger
`INC-0087` confirmed an earlier `Q01` onset at `T2600`, but also exposed a
local non-monotone pocket at `T2650`:
- `T2600 Q01`: routed amortized `0.1159s` vs dense `0.1180s`
- `T2650 Q01`: routed amortized `0.1143s` vs dense `0.0970s`
- `T2700 Q01`: routed amortized `0.0850s` vs dense `0.1217s`

Search work stayed effectively fixed near `0.193` of dense across the whole
band, so the honest next question was cost composition, not geometry.

## Branch Contract
- keep the confirmed `INC-0065` route law fixed
- keep the confirmed `INC-0071` secondary-key law fixed
- keep the confirmed `INC-0083` persistent-cache implementation fixed
- reuse the tracked `INC-0087` confirm artifact as the measurement source
- explain the `T2600/T2650/T2700` local split in direct systems terms
- do not reopen geometry or reroute the branch from the data already on disk

## Evidence
- Source confirm analysis:
  - `results/analysis/inc0087_product_phase_translation_warm_cache_q01_threshold_refine_confirm.json`
- Source confirm profile:
  - `results/analysis/inc0087_product_phase_translation_warm_cache_q01_threshold_refine_confirm_profile.json`
- Cost audit:
  - `results/analysis/inc0087_product_phase_translation_warm_cache_q01_threshold_refine_cost_audit.json`
- Cost report:
  - `docs/reports/INC0087_PRODUCT_PHASE_TRANSLATION_WARM_CACHE_Q01_THRESHOLD_REFINE_COST_AUDIT.md`

## Reading
- `T2600` crosses because the routed search gain still beats the fixed
  route-query plus residual offline penalty:
  - search gain per repeat vs dense: `+0.0264s`
  - route-query penalty per repeat: `0.0189s`
  - offline penalty per repeat: `0.0053s`
  - amortized margin vs dense: `+0.0022s`
- `T2650` misses because the dense search time dips locally while the routed
  route-query penalty stays almost unchanged:
  - search gain per repeat vs dense: `+0.0074s`
  - route-query penalty per repeat: `0.0191s`
  - offline penalty per repeat: `0.0055s`
  - amortized margin vs dense: `-0.0173s`
- `T2700` crosses again because the dense search time rises back up while the
  routed work ratio remains unchanged:
  - search gain per repeat vs dense: `+0.0611s`
  - route-query penalty per repeat: `0.0192s`
  - offline penalty per repeat: `0.0053s`
  - amortized margin vs dense: `+0.0367s`
- The local `T2650` miss is therefore explained by systems-cost balance, not
  by route degeneration:
  - routed candidate fraction stays near `0.194`
  - routed cache hits stay exact
  - routed top-1 stays in the same narrow band

## Decision
- Close `INC-0088` positive/explanatory.
- Keep `H4XH4_FIELD_A150_CPX8_Q01_T2600` as the earliest tracked confirmed
  warm-cache single-query crossover point.
- Treat the `T2650` miss as a local cost-composition pocket, not as a reason
  to reopen geometry or to revoke the earlier onset.
- Move next to a final threshold refinement inside `2500-2600`.

## Resume Note
Resume from the `INC-0087` confirm artifact and the `INC-0088` cost audit, and
treat the next branch as a narrow `2500-2600` warm-cache `Q01` refinement.
