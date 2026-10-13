# INC-0094: Product Phase Translation Chart-Resident Route-Ephemeral Repeat Map

## Status
Confirm completed positive/narrow on 2026-03-12.

## Trigger
`INC-0093` decomposed the cache-residency story on the fixed translated product
stack:
- chart-only residency already preserves the high-bank `T40000 Q01` systems win
- route-only residency is negative at both anchor banks
- the exact lower-bank `T2500 Q01` floor still requires full warm residency

That means the next honest question is no longer whether partial residency can
work at all. It is whether the remaining lower-bank gap closes under the
practical chart-resident / route-ephemeral workload that rebuilds routes but
reuses the chart across a small number of repeated queries.

## Branch Contract
- keep the confirmed `INC-0065` route law fixed
- keep the confirmed `INC-0071` secondary-key law fixed
- keep the confirmed `INC-0083` persistent-cache implementation fixed
- keep the `INC-0093` residency decomposition fixed
- hold cache state to chart-resident / route-ephemeral only:
  - `cache_chart=1`
  - `cache_routes=0`
- keep the same anchor banks:
  - `T2500`
  - `T40000`
- do not reopen geometry, retrieval scoring, or bank-size search

## Minimal Scope
1. Carry only:
   - `DENSE`
   - `H4XH4_FIELD_A150_CPX8`
2. Evaluate only chart-resident / route-ephemeral workloads.
3. Map the minimal repeat ladder needed at fixed banks:
   - `Q01`
   - `Q02`
   - `Q04`
4. Treat this as operational reuse analysis, not another warm-cache frontier
   search.

## Stop Rule
- If `T2500` crosses by `Q02` under chart-only residency, strengthen the
  hardware claim to chart-persistent sessions rather than fully warm sessions.
- If `T2500` still does not cross by `Q04`, narrow the lower-bank claim to
  fully warm reuse and stop pushing this surface.

## Resume Note
Resume from the `INC-0094` confirm artifact and treat the next branch as a
chart-resident `Q01` bank-boundary search rather than another repeat-map or
full-warm threshold branch.

## Evidence
- Configs:
  - `configs/proxy_transfer_inc0094_product_phase_translation_chart_resident_route_ephemeral_repeat_map_prewarm_screen.json`
  - `configs/proxy_transfer_inc0094_product_phase_translation_chart_resident_route_ephemeral_repeat_map_screen.json`
  - `configs/proxy_transfer_inc0094_product_phase_translation_chart_resident_route_ephemeral_repeat_map_prewarm_confirm.json`
  - `configs/proxy_transfer_inc0094_product_phase_translation_chart_resident_route_ephemeral_repeat_map_confirm.json`
- Analyses:
  - `results/analysis/inc0094_product_phase_translation_chart_resident_route_ephemeral_repeat_map_screen.json`
  - `results/analysis/inc0094_product_phase_translation_chart_resident_route_ephemeral_repeat_map_screen_profile.json`
  - `results/analysis/inc0094_product_phase_translation_chart_resident_route_ephemeral_repeat_map_confirm.json`
  - `results/analysis/inc0094_product_phase_translation_chart_resident_route_ephemeral_repeat_map_confirm_profile.json`
- Reports:
  - `docs/reports/INC0094_PRODUCT_PHASE_TRANSLATION_CHART_RESIDENT_ROUTE_EPHEMERAL_REPEAT_MAP_SCREEN.md`
  - `docs/reports/INC0094_PRODUCT_PHASE_TRANSLATION_CHART_RESIDENT_ROUTE_EPHEMERAL_REPEAT_MAP_CONFIRM.md`
- Gate notes:
  - `docs/governance/gates/gate_20260312_080917.md`
  - `docs/governance/gates/gate_20260312_081226.md`
  - `docs/governance/gates/gate_20260312_081526.md`
  - `docs/governance/gates/gate_20260312_082126.md`

## Screen Read
- The 2-seed screen already answered the branch question.
- `T2500`
  - `Q01`: still negative under chart-only residency
  - `Q02`: first crossover at `CHART_H4XH4_FIELD_A150_CPX8_Q02_T2500`
  - `Q04`: still healthy, with larger margin
- `T40000`
  - `Q01`: already positive under chart-only residency
  - `Q02` and `Q04`: remain positive with increasing margin
- The screen justified confirm because the low-bank `Q02` crossover was real
  enough to carry and matched the stop rule exactly.

## Confirm Read
- The 4-seed confirm kept the same structure.
- `T2500`
  - dense:
    - `DENSE_Q01_T2500`: `top1=0.052000`, `amortized=0.1041s`
    - `DENSE_Q02_T2500`: `0.052000`, `0.1049s`
    - `DENSE_Q04_T2500`: `0.052000`, `0.0896s`
  - chart-only routed:
    - `CHART_H4XH4_FIELD_A150_CPX8_Q01_T2500`: `top1=0.044600`,
      `cand_frac=0.193328`, `amortized=0.1241s`
    - `CHART_H4XH4_FIELD_A150_CPX8_Q02_T2500`: `0.044600`,
      `0.193328`, `0.0964s`
    - `CHART_H4XH4_FIELD_A150_CPX8_Q04_T2500`: `0.044600`,
      `0.193328`, `0.0543s`
  - low-bank read:
    - `Q01` still misses slightly: margin `-0.0199s`
    - `Q02` crosses: margin `+0.0085s`
    - `Q04` strengthens: margin `+0.0353s`
- `T40000`
  - dense:
    - `DENSE_Q01_T40000`: `top1=0.048850`, `amortized=9.2545s`
    - `DENSE_Q02_T40000`: `0.048850`, `9.3027s`
    - `DENSE_Q04_T40000`: `0.048850`, `9.0216s`
  - chart-only routed:
    - `CHART_H4XH4_FIELD_A150_CPX8_Q01_T40000`: `top1=0.047325`,
      `cand_frac=0.183764`, `amortized=2.4952s`
    - `CHART_H4XH4_FIELD_A150_CPX8_Q02_T40000`: `0.047325`,
      `0.183764`, `2.1109s`
    - `CHART_H4XH4_FIELD_A150_CPX8_Q04_T40000`: `0.047325`,
      `0.183764`, `1.9548s`
  - upper-bank read:
    - `Q01` stays strongly positive: margin `+6.7593s`
    - `Q02` and `Q04` remain strongly positive
- Cache state stayed exact for the routed branch:
  - `chart_cache_hit=1.0`
  - `route_cache_hit=0.0`

## Reading
- The hardware-side translated claim is now stronger than “fully warm only.”
- A chart-persistent / route-ephemeral session already works:
  - at `T2500` by `Q02`
  - at `T40000` already by `Q01`
- The remaining gap is now specific and narrow:
  - chart-resident `T2500 Q01` still misses slightly
  - full warm still gives the stronger single-query lower-bank point

## Decision
- Close `INC-0094` confirm positive/narrow.
- Promote chart-persistent sessions as the stronger operational claim relative
  to the old fully-warm-only read.
- Keep `CHART_H4XH4_FIELD_A150_CPX8_Q02_T2500` as the earliest confirmed
  chart-resident lower-bank crossover point.
- Keep `CHART_H4XH4_FIELD_A150_CPX8_Q01_T40000` as the chart-resident
  single-query upper-bank point.
- Move next to a chart-resident `Q01` bank-boundary search (`INC-0095`).
