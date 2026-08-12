# INC-0098: Product Phase Translation Chart-Resident Route-Cost Decomposition

## Status
Completed positive/explanatory on 2026-03-12.

## Trigger
`INC-0097` reopens shell-side geometry honestly and closes negative at screen:
- sparse / gated shell controllers are mechanism-live
- both sparse candidates fail shell health by over-concentrating shell mass
- the fixed continuous product route remains the only healthy product branch

That means the next honest question is no longer shell-law tuning. It is where
the remaining routed cost lives on the fixed translated chart-resident stack.

## Branch Contract
- keep the confirmed `INC-0065` product route law fixed
- keep the confirmed `INC-0071` secondary-key law fixed
- keep the confirmed `INC-0083` persistent-cache implementation fixed
- keep the confirmed chart-resident lower-bank point fixed:
  - `CHART_H4XH4_FIELD_A150_CPX8_Q01_T2500`
- keep the confirmed chart-resident upper-bank point fixed:
  - `CHART_H4XH4_FIELD_A150_CPX8_Q01_T40000`
- use the exact lower-bank full-warm point only as a reference:
  - `FULL_H4XH4_FIELD_A150_CPX8_Q01_T2500`
- do not reopen shell-law tuning, packet-scope auditing, or bank-threshold
  search inside this branch

## Minimal Scope
1. Audit routed cost composition at the fixed chart-resident anchors.
2. Break routed time into at least:
   - `route_index_build`
   - `query_route`
   - `retrieval_search`
   - residual overhead
3. Compare chart-resident against:
   - dense exact
   - the lower-bank full-warm floor
4. Treat this as a cost-accounting branch, not a new geometry branch.

## Stop Rule
- If route materialization dominates the remaining lower-bank gap, future
  rescue should target route-query/build implementation or precomputed route
  coordinates rather than shell-law tuning.
- If search dominates or chart-resident margins are already robust at both
  anchors, freeze the translated systems branch and return to the broader
  sparse/event or real-task kill tests.

## Resume Note
Resume from the `INC-0097` negative screen artifact and use the fixed
chart-resident `T2500/T40000 Q01` points as the canonical hardware-side
reference surfaces.

## Evidence
- Tool:
  - `tools/translated_cost_accounting.py`
- Test:
  - `tests/test_translated_cost_accounting.py`
- Input merge:
  - `results/analysis/inc0098_product_phase_translation_chart_resident_route_cost_decomposition_input.json`
- Audit artifacts:
  - `results/analysis/inc0098_product_phase_translation_chart_resident_route_cost_decomposition.json`
  - `docs/reports/INC0098_PRODUCT_PHASE_TRANSLATION_CHART_RESIDENT_ROUTE_COST_DECOMPOSITION.md`

## Audit Read
- The fixed chart-resident stack is already positive against dense at both
  anchors:
  - lower bank, hardened chart-resident point:
    - `DENSE_Q01_T2500`: `0.1155s`
    - `CHART_H4XH4_FIELD_A150_CPX8_Q01_T2500`: `0.0807s`
    - margin vs dense: `+0.0348s`
  - upper bank, chart-resident point:
    - `DENSE_Q01_T40000`: `9.2545s`
    - `CHART_H4XH4_FIELD_A150_CPX8_Q01_T40000`: `2.4952s`
    - margin vs dense: `+6.7593s`
- Lower-bank chart versus full-warm floor:
  - `FULL_H4XH4_FIELD_A150_CPX8_Q01_T2500`: `0.0562s`
  - chart penalty vs full-warm: `+0.0245s`
  - decomposition:
    - route-index build penalty: `+0.0138s`
    - route-query penalty: `+0.0014s`
    - retrieval-search penalty: `+0.0098s`
    - residual penalty: approximately `0`
  - dominant component: `route_index_build`, but only narrowly
- Upper-bank chart versus full-warm reference:
  - `FULL_H4XH4_FIELD_A150_CPX8_Q01_T40000`: `2.0185s`
  - chart penalty vs full-warm: `+0.4767s`
  - decomposition:
    - route-index build penalty: `+0.2045s`
    - route-query delta: `-0.0023s`
    - retrieval-search penalty: `+0.2748s`
    - residual penalty: approximately `0`
  - dominant component: `retrieval_search`
- The routed signal itself stays fixed on the upper bank:
  - top-1 delta between chart-resident and full-warm upper reference: `0.0000`
  - candidate-fraction delta: `0.0000`

## Reading
- The translated systems branch is no longer blocked on a hidden route-query or
  offline-residual bug.
- The remaining chart-resident gap to full warm is not a single
  route-materialization story:
  - lower bank is split between route-index build and retrieval search
  - upper bank is dominated by retrieval-search overhead with route-index build
    as the secondary term
- Because chart-resident margins are already positive against dense at both
  anchors, more translated cost shaving is no longer the highest-value kill
  test.
- The honest next move is to freeze the translated systems branch as the
  current hardware-side reference and return to the broader sparse event-driven
  trainability question from the kill ladder.

## Decision
- Close `INC-0098` positive/explanatory.
- Freeze the fixed chart-resident translated stack as the current software-side
  hardware reference.
- Do not reopen shell-law tuning, packet-scope auditing, or more translated
  bank/cache refinement from this point.
- Move next to a sparse event-driven proxy trainability pilot (`INC-0099`)
  rather than another translated cost rescue surface.
