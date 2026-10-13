# INC-0106: Product Phase Sparse Translation Systems Cost Decomposition

## Status
Completed positive/explanatory on 2026-03-12.

## Trigger
`INC-0104` and `INC-0105` produced a clean split:
- bounded backfill did not recover translated quality
- but it did create stable sparse translated systems leads at both anchors:
  - lower bank:
    `CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500`
  - upper bank:
    `CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T40000`

The next honest question is not another rescue heuristic. It is where the new
systems gain comes from and whether it is mostly search-side, route-query-side,
or route-index-side.

## Branch Contract
- keep the confirmed `INC-0065` product route law fixed
- keep the confirmed sparse translated quality references fixed:
  - `CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T2500`
  - `CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000`
- keep the confirmed sparse translated systems leads fixed:
  - `CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500`
  - `CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T40000`
- reopen translated work only through cost decomposition on the fixed anchor
  points
- do not reopen `T/Q` threshold search or new recovery heuristics inside this
  branch

## Minimal Scope
1. Audit lower and upper sparse translated anchors against:
   - dense exact
   - continuous translated product reference
   - fixed soft sparse translated reference
   - bounded backfill sparse translated systems lead
2. Decompose at least:
   - route-index build
   - query-route time
   - retrieval-search time
   - offline total
   - amortized total
3. Stop at audit/report stage unless the decomposition exposes a new hidden
   regression surface.

## Evidence
- Analyses:
  - `results/analysis/inc0104_product_phase_soft_sparse_translation_backfill_recovery_confirm.json`
  - `results/analysis/inc0105_product_phase_soft_sparse_translation_upper_bank_carry_forward_confirm.json`
  - `results/analysis/inc0106_product_phase_sparse_translation_systems_cost_decomposition.json`
- Report:
  - `docs/reports/INC0106_PRODUCT_PHASE_SPARSE_TRANSLATION_SYSTEMS_COST_DECOMPOSITION.md`
- Tooling:
  - `tools/translated_cost_accounting.py`

## Read
- Lower bank versus dense:
  - `CHART_H4XH4_FIELD_A150_CPX8_Q01_T2500` stays negative on amortized cost
  - `CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T2500` also stays negative
  - `CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500` becomes positive
    because `0.046s` online gain exceeds the `0.026s` offline penalty
- Lower-bank sparse translated systems gain is mostly search-side:
  - soft sparse to bounded backfill amortized delta: `-0.047s`
  - retrieval-search delta: `-0.037s`
  - route-index build delta: `-0.006s`
  - route-query delta: `-0.004s`
- Lower-bank bounded backfill also beats the continuous translated product
  reference on all seeds on average, again mainly through search reduction.
- Upper bank stays strongly positive against dense for all routed points, but
  the component mix is less clean:
  - continuous to soft sparse: average gain comes from search reduction
    despite worse route-query cost
  - soft sparse to bounded backfill: average gain comes from route-query plus
    search, while route-index build gets worse
  - continuous to bounded backfill: average gain is still search-dominated
- No hidden offline or accounting regression surfaced. The bounded backfill
  systems gain is real, but the upper-bank component story needs a seed-level
  stability check before it is used as optimization guidance.

## Decision
- Close `INC-0106` positive/explanatory.
- Keep the bounded backfill points as the sparse translated systems leads at
  `T2500 Q01` and `T40000 Q01`.
- Treat lower-bank gain as a real search-dominated systems improvement.
- Do not treat upper-bank route-query savings as stable guidance yet.
- Move next to per-seed component stability hardening (`INC-0107`) instead of
  another sparse translated rescue heuristic.

## Acceptance
- produce a direct explanation of the `BF2_SB1` systems gain at `T2500` and
  `T40000`
- show whether the gain is dominated by search reduction or by some other
  component
- leave the route law and retrieval heuristics unchanged

## Resume Note
Resume from the fixed sparse translated quality/system split created by
`INC-0104` and `INC-0105`. This branch is now closed; the next branch is
seed-level component stability hardening, not another translated quality
rescue.
