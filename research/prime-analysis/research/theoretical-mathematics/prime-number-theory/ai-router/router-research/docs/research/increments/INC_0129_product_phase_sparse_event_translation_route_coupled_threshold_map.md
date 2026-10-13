# INC-0129: Product Phase Sparse Event Translation Route-Coupled Threshold Map

## Status
Completed screen, negative/explanatory.

## Trigger
`INC-0128` made translated sparse-event behavior real on the downstream
surface, but the first train-gate-prune point split cleanly:
- soft sparse at threshold `0.02` did not actually prune
- near-hard at threshold `0.02` pruned materially but collapsed top-1

## Implementation
- Kept the fixed product route law unchanged.
- Kept the translated coupling mechanism unchanged:
  - `event_gate_translation_coupling=train_gate_prune`
- Varied only the near-hard prune threshold on the lower-bank translated
  screen surface.
- Carried the fixed lower-bank translated references:
  - `CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T2500`
  - `CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_CPX8_Q01_T2500`

## Screen Result
- Artifact:
  - `results/analysis/inc0129_product_phase_sparse_event_translation_route_coupled_threshold_map_screen.json`
- Gate note:
  - `docs/governance/gates/gate_20260312_194544.md`
- Threshold `0.010` is the only near-hard point that keeps quality near the
  existing translated sparse references:
  - `CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_EGP010_CPX8_Q01_T2500`
  - `keep_frac=0.992`
  - `top1=0.0448`
  - `cand_frac=0.187042`
- But that point is still not promotable:
  - online rises from `0.1139s` to `0.1255s` versus uncoupled near-hard
  - amortized rises from `0.1452s` to `0.1723s`
- Stronger thresholds are selectively live but degrade too sharply:
  - `0.015`: `top1=0.0356`, `cand_frac=0.161352`
  - `0.018`: `top1=0.0324`, `cand_frac=0.143105`
  - `0.020`: `top1=0.0212`, `cand_frac=0.131042`
  - `0.022`: `top1=0.0136`, `cand_frac=0.103121`

## Decision
- Close `INC-0129` negative/explanatory.
- The current train-gate-prune coupling is genuinely downstream-live.
- But no threshold window satisfies the branch contract:
  - tiny pruning preserves quality but worsens runtime
  - useful pruning collapses translated quality
- Do not promote any thresholded train-gate-prune point.
- Keep the existing translated sparse-event references unchanged.

## Next Step
- Queue `INC-0130`: product-phase sparse-event translated route-coupled
  soft-bias pilot.
- Move off hard train-bank omission and test a softer downstream coupling that
  can change translated retrieval behavior without deleting train items.

## Resume Note
Resume from the completed `INC-0129` screen. The next question is no longer
whether train-gate pruning has a viable threshold window. The next question is
whether a softer translated coupling can become downstream-live without paying
the omission-driven quality collapse.
