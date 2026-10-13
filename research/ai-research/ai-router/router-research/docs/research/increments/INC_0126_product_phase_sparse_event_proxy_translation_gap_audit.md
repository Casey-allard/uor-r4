# INC-0126: Product Phase Sparse Event Proxy Translation Gap Audit

## Status
Completed positive/explanatory on 2026-03-12.

## Trigger
`INC-0125` hardened the sparse-event proxy story:
- the near-hard controller `H4XH4_FIELD_A150_EVT_T070_TAU002` is now a
  confirmed healthy sparse proxy point under a materially harder load
- the true hard gate remains mostly-on

But `INC-0102` had already closed translated near-hard carry-forward negative
at screen. This branch asked why the sharpened controller survives on proxy
trainability but not on translated systems.

## Evidence
- Audit tool:
  `tools/sparse_event_proxy_translation_gap_audit.py`
- Audit analysis:
  `results/analysis/inc0126_product_phase_sparse_event_proxy_translation_gap_audit.json`
- Report:
  `docs/reports/INC0126_PRODUCT_PHASE_SPARSE_EVENT_PROXY_TRANSLATION_GAP_AUDIT.md`
- Source analyses:
  - `results/analysis/inc0125_product_phase_sparse_event_proxy_trainability_hardening_confirm.json`
  - `results/analysis/inc0100_product_phase_sparse_event_translation_confirm.json`
  - `results/analysis/inc0102_product_phase_near_hard_event_translation_screen.json`

## Reading
- The near-hard controller remains a real sparse proxy mechanism result.
- The translated near-hard failure is not a quality or candidate-set failure:
  - near-hard vs translated soft sparse top-1 delta = `0.000000`
  - near-hard vs translated soft sparse candidate-fraction delta = `0.000000`
- The translated failure is systems-cost-only:
  - near-hard vs translated soft sparse online delta = `+0.098602s`
  - near-hard vs translated soft sparse amortized delta = `+0.117689s`
  - near-hard vs translated continuous online delta = `+0.134220s`
- The dominant cost driver is retrieval search, not candidate omission:
  - retrieval-search delta vs soft sparse = `+0.087495s`
  - route-index-build delta vs soft sparse = `+0.018890s`
  - route-query delta vs soft sparse = `+0.011108s`
  - omission-supported = `False`
  - in-candidate-ordering-loss-supported = `False`
- This means the sharpened event gate keeps the same translated answer surface
  but makes the lower-bank translated stack slower.

## Decision
- Close `INC-0126` positive/explanatory.
- Keep the near-hard result as a valid proxy mechanism win.
- Reclassify the translated near-hard failure from “near-hard does not carry”
  to “near-hard carries quality but loses on translated systems cost.”
- Move next to a narrow translated systems-cost rescue branch, not to quality
  recovery, candidate recovery, or more downstream packet work.

## Next Honest Branch
- `INC-0127`: Product Phase Sparse Event Translation Systems Cost Rescue
  - keep the fixed product route law and near-hard controller unchanged
  - target only translated route-query, route-index-build, and retrieval-search
    overhead on the lower-bank chart-resident surface
  - stop immediately if cost rescue requires reopening quality/candidate-set
    behavior

## Resume Note
Resume from the completed `INC-0126` gap audit, not from the old downstream
packet-manifest loop. The live question is now whether translated near-hard can
be rescued on systems cost alone.
