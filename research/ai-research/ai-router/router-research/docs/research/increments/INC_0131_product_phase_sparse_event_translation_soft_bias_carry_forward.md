# INC-0131: Product Phase Sparse Event Translation Soft-Bias Carry-Forward

## Status
Completed confirm, positive/explanatory.

## Trigger
`INC-0130` showed that translated sparse-event soft score bias is genuinely
downstream-live without pruning:
- `SBI030` improved lower-bank translated top-1 over uncoupled near-hard
  while staying faster than dense
- `SBI080` produced the strongest quality read, but with a much larger runtime
  penalty
- the original `INC-0130` packet also carried a cold-chart artifact on the
  first route, so the next branch needs an explicit prewarm path

## Branch Contract
- keep the fixed product route law unchanged
- keep the lower-bank translated bank size unchanged at `T2500`
- keep score bias as a reordering-only translated coupling
- evaluate only the smallest useful soft-bias subset:
  - uncoupled near-hard reference
  - balanced soft-bias candidate
  - quality-first soft-bias comparator
  - dense lower-bank baseline
  - current lower-bank bounded-backfill systems default
- require explicit prewarm before both screen and confirm

## Minimal Scope
1. Prewarm the chart cache explicitly for the lower-bank translated packet.
2. Re-run a focused 2-seed screen on:
   - `DENSE_Q01_T2500`
   - `CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500`
   - `CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_CPX8_Q01_T2500`
   - `CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_SBI030_CPX8_Q01_T2500`
   - `CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_SBI080_CPX8_Q01_T2500`
3. Carry `SBI030` and `SBI080` to 4-seed confirm only if the prewarmed screen
   preserves the same lower-bank quality/runtime ordering.

## Result
- Screen artifacts:
  - `results/analysis/inc0131_product_phase_sparse_event_translation_soft_bias_carry_forward_prewarm_screen.json`
  - `results/analysis/inc0131_product_phase_sparse_event_translation_soft_bias_carry_forward_screen.json`
  - gate notes:
    - `docs/governance/gates/gate_20260312_200530.md`
    - `docs/governance/gates/gate_20260312_200544.md`
- Confirm artifacts:
  - `results/analysis/inc0131_product_phase_sparse_event_translation_soft_bias_carry_forward_prewarm_confirm.json`
  - `results/analysis/inc0131_product_phase_sparse_event_translation_soft_bias_carry_forward_confirm.json`
  - gate notes:
    - `docs/governance/gates/gate_20260312_200642.md`
    - `docs/governance/gates/gate_20260312_200705.md`
- The confirm split is stable:
  - uncoupled near-hard remains the lower-bank systems reference:
    - `top1=0.0446`
    - `cand_frac=0.193328`
    - `amortized=0.0899s`
  - `SBI030` is the balanced quality-lift point:
    - `top1=0.0464`
    - `cand_frac=0.193328`
    - `amortized=0.0942s`
  - `SBI080` is the quality-first point:
    - `top1=0.0524`
    - `cand_frac=0.193328`
    - `amortized=0.1416s`
  - dense remains:
    - `top1=0.0520`
    - `amortized=0.1258s`
  - the old lower-bank bounded-backfill point did not hold as an active
    default on this focused prewarmed packet:
    - `top1=0.0452`
    - `amortized=1.9991s`

## Acceptance
- `SBI030` remains a real quality lift over uncoupled near-hard under prewarm
- at least one soft-bias point remains clearly faster than dense at `T2500`
- no soft-bias point is allowed to change candidate fraction materially
- the branch must close as one of:
  - balanced lower-bank quality comparator (`SBI030`)
  - quality-first comparator only (`SBI080`)
  - or negative if the screen lift disappears under prewarm

## Decision
- Close `INC-0131` positive/explanatory.
- Promote `CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_CPX8_Q01_T2500` as the
  lower-bank sparse-event translated systems reference.
- Promote `CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_SBI030_CPX8_Q01_T2500` as
  the balanced lower-bank quality comparator.
- Keep `CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_SBI080_CPX8_Q01_T2500` as a
  quality-first comparator only.
- Treat the old lower-bank bounded-backfill default as stale until an explicit
  lower-bank reference reselection branch resolves it.

## Next Step
- Queue `INC-0132`: product-phase sparse-event translated lower-bank
  reference reselection.
- Use the completed `INC-0131` confirm packet to decide whether the broader
  comparison and downstream default lower-bank route should move from the old
  bounded-backfill point to the uncoupled near-hard point.

## Resume Note
Resume from the completed `INC-0131` confirm. The next question is no longer
whether soft-bias survives prewarm. The next question is which lower-bank
sparse-event translated route should become the new default carry-forward
reference.
