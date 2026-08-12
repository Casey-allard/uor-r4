# INC-0100: Product Phase Sparse Event Translation Pilot

## Status
Confirm completed positive/narrow on 2026-03-12.

## Trigger
`INC-0099` closed positive on the proxy harness:
- the fixed product route law can absorb a real soft event controller
- the promoted sparse point `H4XH4_FIELD_A150_EVT_T070` keeps shell health and
  slightly improves proxy MSE versus the continuous product reference
- the event result is meaningful on soft update mass
  (`event_gate_mean≈0.319`), but not yet on hard active fraction

That means the next honest branch is not more proxy threshold shaving. It is
carry-forward of the fixed sparse controller into the translated retrieval
reference stack.

## Branch Contract
- keep the confirmed `INC-0065` product route law fixed
- keep the confirmed `INC-0071` secondary-key law fixed wherever retrieval is
  used
- keep the confirmed `INC-0099` sparse event controller fixed:
  - `event_gate_mode=soft_error`
  - `event_gate_threshold=0.07`
  - `event_gate_tau=0.01`
- keep translated chart-resident evaluation as the downstream systems
  reference, not as a route-law tuning target
- do not reopen proxy-only threshold search or shell-law variants inside this
  branch

## Minimal Scope
1. Thread the sparse-event controller into the translated retrieval harness.
2. Compare the fixed sparse point against the fixed continuous product
   reference on the translated stack.
3. Measure at least:
   - top-1 / retrieval quality
   - candidate fraction
   - translated runtime
   - sparse-event cost proxy carried through training
4. Screen first before any larger hardware-side mapping.

## Stop Rule
- If the fixed sparse controller loses the translated systems story or destroys
  retrieval quality, close the branch negative and keep sparse-event claims at
  the proxy-only level.
- If the fixed sparse controller preserves the translated tradeoff, carry it
  forward as the first sparse-event retrieval reference.

## Resume Note
Resume from the `INC-0100` confirm artifact and treat the next honest branch as
hard threshold event activation on the fixed product route law (`INC-0101`),
with the soft-sparse translated point kept as the current sparse-event
retrieval reference.

## Evidence
- Configs:
  - `configs/proxy_transfer_inc0100_product_phase_sparse_event_translation_prewarm_screen.json`
  - `configs/proxy_transfer_inc0100_product_phase_sparse_event_translation_screen.json`
  - `configs/proxy_transfer_inc0100_product_phase_sparse_event_translation_prewarm_confirm.json`
  - `configs/proxy_transfer_inc0100_product_phase_sparse_event_translation_confirm.json`
- Analyses:
  - `results/analysis/inc0100_product_phase_sparse_event_translation_prewarm_screen.json`
  - `results/analysis/inc0100_product_phase_sparse_event_translation_screen.json`
  - `results/analysis/inc0100_product_phase_sparse_event_translation_prewarm_confirm.json`
  - `results/analysis/inc0100_product_phase_sparse_event_translation_confirm.json`
- Reports:
  - `docs/reports/INC0100_PRODUCT_PHASE_SPARSE_EVENT_TRANSLATION_SCREEN.md`
  - `docs/reports/INC0100_PRODUCT_PHASE_SPARSE_EVENT_TRANSLATION_CONFIRM.md`
- Gate notes:
  - `docs/governance/gates/gate_20260312_110218.md`
  - `docs/governance/gates/gate_20260312_110233.md`
  - `docs/governance/gates/gate_20260312_110306.md`
  - `docs/governance/gates/gate_20260312_110324.md`

## Screen Read
- The screen packet held the lower-bank chart-resident stack fixed at
  `T2500 Q01`.
- Relative to the continuous routed product reference, the sparse-event point
  already preserved retrieval signal and improved runtime:
  - `CHART_H4XH4_FIELD_A150_CPX8_Q01_T2500`
    - `top1=0.0444`
    - `cand_frac=0.189016`
    - `online=0.19856s`
    - `amortized=0.25265s`
  - `CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T2500`
    - `top1=0.0444`
    - `cand_frac=0.189016`
    - `online=0.14495s`
    - `amortized=0.19423s`
    - `event_gate_mean=0.3192`
    - `event_gate_active_frac=0.0`
- Dense exact still held a slight screen lead on amortized runtime, so the
  confirm question remained open.

## Confirm Read
- The 4-seed confirm closed the branch positively on the translated stack.
- Dense exact:
  - `DENSE_Q01_T2500`
    - `top1=0.0520`
    - `cand_frac=1.000000`
    - `online=0.16866s`
    - `amortized=0.16866s`
- Continuous routed product:
  - `CHART_H4XH4_FIELD_A150_CPX8_Q01_T2500`
    - `top1=0.0446`
    - `cand_frac=0.193328`
    - `online=0.27591s`
    - `amortized=0.33801s`
    - `event_gate_mean=1.0000`
- Sparse-event routed product:
  - `CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T2500`
    - `top1=0.0446`
    - `cand_frac=0.193328`
    - `online=0.11774s`
    - `amortized=0.16861s`
    - `event_gate_mean=0.3191`
    - `event_gate_active_frac=0.0`
- The sparse-event point therefore:
  - preserves translated retrieval signal relative to the continuous routed
    product reference
  - materially improves routed runtime relative to that continuous routed
    reference
  - reaches an almost exact lower-bank `Q01` amortized tie with dense exact
    while pruning about `80.7%` of candidates
- The remaining quality gap versus dense exact is unchanged from the routed
  product branch and remains the next task-quality constraint.

## Reading
- `INC-0100` does not prove hard event firing.
- It does prove something narrower and still important:
  - the fixed soft-sparse controller carries through the translated retrieval
    harness
  - the event controller still reduces effective training-update mass to about
    `31.9%` of the dense path
  - the downstream translated systems story survives that controller instead of
    collapsing
- This makes `CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T2500` the first
  sparse-event translated retrieval reference.

## Decision
- Close `INC-0100` confirm positive/narrow.
- Promote `CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T2500` as the sparse-event
  translated retrieval reference on the fixed product route law.
- Do not overclaim dense replacement from this branch:
  - the lower-bank dense amortized edge is effectively knife-edge
  - the top-1 gap to dense exact remains
- Move next to hard threshold event activation on the fixed product route law
  (`INC-0101`), beginning again on the proxy harness rather than reopening
  translated bank or cache mapping.
