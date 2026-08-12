# INC-0130: Product Phase Sparse Event Translation Route-Coupled Soft-Bias Pilot

## Status
Completed screen, positive/explanatory.

## Trigger
`INC-0128` and `INC-0129` together showed:
- translated sparse-event behavior can be made genuinely downstream-live
- hard train-bank omission via train-gate pruning is too blunt
- tiny pruning preserves quality but does not improve runtime
- useful pruning collapses translated top-1

## Branch Contract
- keep the fixed product route law unchanged
- retire train-gate pruning as the active carry-forward candidate
- add one softer translated coupling that changes downstream retrieval
  behavior without deleting translated train items
- stay on the lower-bank translated screen surface first
- do not reopen rerank, packet, or unrelated downstream inheritance work

## Minimal Scope
1. Choose one soft translated coupling surface, such as score bias or route-
   local weighting, that can change downstream behavior without item omission.
2. Compare it against the fixed lower-bank translated sparse references.
3. Stop at screen stage unless one point is clearly promotable.

## Screen Result
- Artifact:
  - `results/analysis/inc0130_product_phase_sparse_event_translation_route_coupled_soft_bias_screen.json`
- Gate note:
  - `docs/governance/gates/gate_20260312_195638.md`
- The new score-bias surface is genuinely downstream-live:
  - every `SBI` point keeps
    `event_gate_retrieval_surface_active_mean=1.0`
  - `event_gate_translation_score_bias_abs_mean≈0.258245`
- Candidate fraction does not change on this branch:
  - every soft-bias point keeps `cand_frac=0.189016`
  - the mechanism is reordering-only, not prune-driven
- The balanced quality-positive screen point is:
  - `CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_SBI030_CPX8_Q01_T2500`
  - `top1=0.0464`
  - `online=0.0996s`
  - `amortized=0.1330s`
- The strongest quality-first screen point is:
  - `CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_SBI080_CPX8_Q01_T2500`
  - `top1=0.0548`
  - `online=0.1954s`
  - `amortized=0.2443s`
- The score-bias branch is therefore real, but not a direct systems promotion
  at screen:
  - `SBI030` improves lower-bank translated top-1 over uncoupled near-hard
    with only a small runtime penalty
  - `SBI080` crosses dense on top-1 but pays too much runtime to carry
    forward as a systems point

## Decision
- Close `INC-0130` positive/explanatory.
- Keep `CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_CPX8_Q01_T2500` as the fixed
  lower-bank near-hard sparse-event systems reference.
- Treat `SBI030` as the balanced soft-bias carry-forward candidate.
- Treat `SBI080` as a quality-first comparator rather than a systems point.
- Do not replace the existing lower-bank bounded-backfill default from
  `INC-0104` on the basis of this screen alone.

## Next Step
- Queue `INC-0131`: product-phase sparse-event translated soft-bias
  carry-forward.
- Re-run the lower-bank packet under explicit prewarm to remove the cold-chart
  artifact and confirm whether the `SBI030` quality lift survives while
  `SBI080` remains quality-first only.

## Acceptance
- sparse-event settings become selectively downstream-live under the new soft
  coupling
- the best soft-coupled point preserves translated quality near the current
  lower-bank sparse references
- the branch can be read honestly as either:
  - a viable soft translated sparse-event surface
  - or a negative result that closes the current sparse-event translated
    reopen more broadly

## Resume Note
Resume from the completed `INC-0130` screen. The next question is no longer
whether soft score bias is downstream-live. The next question is whether the
balanced `SBI030` lift survives a prewarmed lower-bank confirm strongly enough
to carry as a new sparse-event translated quality comparator.
