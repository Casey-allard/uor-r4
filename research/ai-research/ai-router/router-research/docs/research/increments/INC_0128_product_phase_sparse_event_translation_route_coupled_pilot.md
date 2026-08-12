# INC-0128: Product Phase Sparse Event Translation Route-Coupled Pilot

## Status
Completed screen, positive/explanatory.

## Trigger
`INC-0127` closed the translated systems-cost rescue reopen negative:
- translated near-hard and translated soft sparse differ only in
  `event_gate_tau`
- the current translated harness treats sparse-event knobs as audit-only
- so the translated sparse-event branch cannot move on route/query/search
  behavior without a route-coupled implementation change

## Implementation
- Kept the fixed product route law unchanged.
- Kept the lower-bank translated surface fixed:
  - `CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T2500`
  - `CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_CPX8_Q01_T2500`
- Added one explicit translated coupling point:
  - `event_gate_translation_coupling=train_gate_prune`
  - prune the translated train bank from per-sample sparse-event gate means
  - preserve at least one item per coarse translated bucket
- Fixed the harness edge case exposed by the first screen run:
  - post-prune label coherence must use the pruned effective train coordinate
    view instead of the full unpruned `v_tr`

## Screen Result
- Artifact:
  - `results/analysis/inc0128_product_phase_sparse_event_translation_route_coupled_screen.json`
- Gate note:
  - `docs/governance/gates/gate_20260312_193527.md`
- Soft sparse plus prune at threshold `0.02` is still effectively inert:
  - `CHART_H4XH4_FIELD_A150_EVT_T070_EGP020_CPX8_Q01_T2500`
  - `event_gate_translation_keep_frac=1.0`
  - candidate fraction stays `0.189016`
  - top-1 stays `0.0444`
- Near-hard plus prune at threshold `0.02` is genuinely downstream-live:
  - `CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_EGP020_CPX8_Q01_T2500`
  - `event_gate_translation_keep_frac≈0.745`
  - `retrieval_train_items_effective≈1863`
  - candidate fraction drops from `0.189016` to `0.131042`
  - online time drops from `0.1782s` to `0.1307s`
- But that same near-hard coupled point is not promotable:
  - top-1 collapses from `0.0444` to `0.0212`
  - this is a real selective downstream effect, not an audit-only no-op
  - this specific prune law is too blunt for translated carry-forward

## Decision
- Close `INC-0128` positive/explanatory.
- The translated sparse-event reopen is no longer blocked on “audit-only”
  behavior.
- Do not promote the current route-coupled prune point:
  - soft sparse at `0.02` does not actually prune
  - near-hard at `0.02` prunes too aggressively and destroys quality
- Keep the existing translated sparse-event references unchanged:
  - quality/reference:
    `CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T2500`
  - lower-bank systems lead:
    `CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500`

## Next Step
- Queue `INC-0129`: product-phase sparse-event translated route-coupled
  threshold map.
- Hold the coupling mechanism fixed and search for the first selective point
  that:
  - actually prunes the translated train bank
  - preserves lower-bank translated quality well enough to remain a real
    candidate
  - otherwise closes train-gate-prune as a dead translated sparse-event
    surface

## Resume Note
Resume from the completed `INC-0128` screen. The next question is no longer
whether sparse-event behavior can be made real on the translated surface at
all. The next question is whether any threshold on the same train-gate-prune
coupling can preserve quality while remaining selectively downstream-live.
