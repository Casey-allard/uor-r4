# INC-0095: Product Phase Translation Chart-Resident Q01 Bank Boundary

## Status
Confirm completed positive/explanatory on 2026-03-12.

## Trigger
`INC-0094` confirms that chart-persistent / route-ephemeral operation is now a
real translated systems regime on the fixed product stack:
- `T2500` crosses by `Q02`
- `T40000` already crosses by `Q01`

That means the next honest question is the earliest bank where the
chart-resident workload already survives at `Q01`, without route-cache reuse.

## Branch Contract
- keep the confirmed `INC-0065` route law fixed
- keep the confirmed `INC-0071` secondary-key law fixed
- keep the confirmed `INC-0083` persistent-cache implementation fixed
- keep the `INC-0094` chart-resident repeat-map read fixed
- hold cache state to chart-resident / route-ephemeral only:
  - `cache_chart=1`
  - `cache_routes=0`
- hold query repeats fixed at:
  - `Q01`
- do not reopen geometry, retrieval scoring, or full-warm threshold search

## Minimal Scope
1. Carry only:
   - `DENSE`
   - `CHART_H4XH4_FIELD_A150_CPX8`
2. Search a coarse bank ladder above `T2500` for the first chart-resident
   `Q01` crossover.
3. Refine only if the screen shows a clean boundary.

## Stop Rule
- If a manageable lower/medium bank already crosses at chart-resident `Q01`,
  promote the single-query chart-persistent claim and stop pushing repeat-map
  analysis.
- If `Q01` still does not cross until very high banks, narrow the chart-only
  claim to `Q02` lower-bank sessions and `Q01` upper-bank sessions.

## Resume Note
Resume from the `INC-0095` confirm artifact and compare it against the
`INC-0094` mixed-repeat confirm artifact; treat the next branch as a packet-
scope audit rather than more bank search or full-warm refinement.

## Evidence
- Configs:
  - `configs/proxy_transfer_inc0095_product_phase_translation_chart_resident_q01_bank_boundary_prewarm_screen.json`
  - `configs/proxy_transfer_inc0095_product_phase_translation_chart_resident_q01_bank_boundary_screen.json`
  - `configs/proxy_transfer_inc0095_product_phase_translation_chart_resident_q01_bank_boundary_prewarm_confirm.json`
  - `configs/proxy_transfer_inc0095_product_phase_translation_chart_resident_q01_bank_boundary_confirm.json`
- Analyses:
  - `results/analysis/inc0095_product_phase_translation_chart_resident_q01_bank_boundary_screen.json`
  - `results/analysis/inc0095_product_phase_translation_chart_resident_q01_bank_boundary_screen_profile.json`
  - `results/analysis/inc0095_product_phase_translation_chart_resident_q01_bank_boundary_confirm.json`
  - `results/analysis/inc0095_product_phase_translation_chart_resident_q01_bank_boundary_confirm_profile.json`
- Reports:
  - `docs/reports/INC0095_PRODUCT_PHASE_TRANSLATION_CHART_RESIDENT_Q01_BANK_BOUNDARY_SCREEN.md`
  - `docs/reports/INC0095_PRODUCT_PHASE_TRANSLATION_CHART_RESIDENT_Q01_BANK_BOUNDARY_CONFIRM.md`
- Gate notes:
  - `docs/governance/gates/gate_20260312_090117.md`
  - `docs/governance/gates/gate_20260312_090210.md`
  - `docs/governance/gates/gate_20260312_090502.md`
  - `docs/governance/gates/gate_20260312_090530.md`

## Screen Read
- The 2-seed coarse ladder did not come back monotone.
- Screen outcomes:
  - `T2500`: crossover already appeared on the focused `Q01` packet
  - `T2750`: local miss on screen
  - `T3000`: slight positive margin
  - `T4000`, `T6000`, `T10000`, `T40000`: all positive
- That meant the bank search itself was not clean enough to stop on screen.
- The honest confirm was therefore the contradictory lower-bank slice:
  - `T2500`
  - `T2750`
  - `T3000`
  - `T4000`

## Confirm Read
- The 4-seed confirm resolved the lower slice in favor of a stronger
  chart-resident single-query claim.
- Dense versus chart-resident routed:
  - `T2500`
    - `DENSE_Q01_T2500`: `top1=0.052000`, `amortized=0.1240s`
    - `CHART_H4XH4_FIELD_A150_CPX8_Q01_T2500`: `top1=0.044600`,
      `cand_frac=0.193328`, `amortized=0.0849s`
    - margin vs dense: `+0.0391s`
  - `T2750`
    - `DENSE_Q01_T2750`: `top1=0.050182`, `amortized=0.1265s`
    - `CHART_H4XH4_FIELD_A150_CPX8_Q01_T2750`: `top1=0.044545`,
      `cand_frac=0.192894`, `amortized=0.0903s`
    - margin vs dense: `+0.0363s`
  - `T3000`
    - `DENSE_Q01_T3000`: `top1=0.049833`, `amortized=0.1184s`
    - `CHART_H4XH4_FIELD_A150_CPX8_Q01_T3000`: `top1=0.044833`,
      `cand_frac=0.191704`, `amortized=0.0949s`
    - margin vs dense: `+0.0236s`
  - `T4000`
    - `DENSE_Q01_T4000`: `top1=0.048875`, `amortized=0.2063s`
    - `CHART_H4XH4_FIELD_A150_CPX8_Q01_T4000`: `top1=0.045000`,
      `cand_frac=0.192636`, `amortized=0.1971s`
    - margin vs dense: `+0.0091s`
- Cache state stayed exact for the routed branch on confirm:
  - `chart_cache_hit=1.0`
  - `route_cache_hit=0.0`
- Search work stayed pinned near `19.2%-19.3%` of dense across the full
  confirmed lower slice.

## Reading
- The focused chart-resident `Q01` bank search does not support an onset above
  `T2500`.
- Under the narrower packet that carries only single-query `Q01` routes,
  chart-resident / route-ephemeral operation already survives at the lower
  anchor:
  - `CHART_H4XH4_FIELD_A150_CPX8_Q01_T2500`
- That broadens the operational claim beyond the older `INC-0094` mixed-repeat
  read, but it also exposes packet-scope sensitivity:
  - `INC-0094` mixed-repeat confirm still had `T2500 Q01` slightly negative
  - `INC-0095` focused `Q01` confirm now has `T2500 Q01` positive
- The next honest question is therefore not bank threshold location anymore.
  It is whether the lower-bank single-query claim is stable across packet
  composition.

## Decision
- Close `INC-0095` confirm positive/explanatory.
- Promote `CHART_H4XH4_FIELD_A150_CPX8_Q01_T2500` as the focused single-query
  chart-resident lower-bank point on the fixed translated stack.
- Keep `CHART_H4XH4_FIELD_A150_CPX8_Q02_T2500` as the mixed-repeat lower-bank
  point from `INC-0094`.
- Retire the chart-resident `Q01` bank-boundary search as an active branch.
- Move next to a packet-scope audit (`INC-0096`) rather than more bank
  refinement.
