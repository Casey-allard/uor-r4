# INC-0132: Product Phase Sparse Event Translation Lower-Bank Reference Reselection

## Status
Completed positive/explanatory.

## Trigger
`INC-0131` resolved the lower-bank soft-bias mechanism question, but it also
created a live carry-forward conflict:
- uncoupled near-hard is now the clean lower-bank sparse-event systems point
- `SBI030` is now the balanced quality comparator
- `SBI080` is now the quality-first comparator
- the old lower-bank bounded-backfill default did not hold on the focused
  prewarmed packet

## Branch Contract
- do not reopen the sparse-event mechanism itself
- use only the completed lower-bank confirm artifacts already on disk
- resolve which lower-bank route should be the default carry-forward route for
  broader and downstream comparison packets
- classify the other lower-bank routes explicitly as:
  - default systems reference
  - balanced quality comparator
  - quality-first comparator
  - or stale historical comparator

## Minimal Scope
1. Compare the completed `INC-0104`, `INC-0130`, and `INC-0131` lower-bank
   artifacts on one normalized surface.
2. Decide whether the old bounded-backfill default still belongs in the
   default packet.
3. If not, rewrite the lower-bank default contract once, then stop.

## Result
- Artifact:
  - `results/analysis/inc0132_product_phase_sparse_event_translation_lower_bank_reference_reselection.json`
- Report:
  - `docs/reports/INC0132_PRODUCT_PHASE_SPARSE_EVENT_TRANSLATION_LOWER_BANK_REFERENCE_RESELECTION.md`
- The normalized lower-bank read now resolves cleanly:
  - default systems reference:
    `CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_CPX8_Q01_T2500`
    - `top1=0.0446`
    - `cand_frac=0.193328`
    - `amortized=0.0899s`
  - balanced quality comparator:
    `CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_SBI030_CPX8_Q01_T2500`
    - `top1=0.0464`
    - `cand_frac=0.193328`
    - `amortized=0.0942s`
  - quality-first comparator:
    `CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_SBI080_CPX8_Q01_T2500`
    - `top1=0.0524`
    - `cand_frac=0.193328`
    - `amortized=0.1416s`
  - stale historical comparator:
    `CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500`
    - focused packet amortized inflation: `+1.8934s`
    - focused packet amortized ratio: `18.91x`

## Acceptance
- the repo ends with one explicit lower-bank default route
- the `SBI030` and `SBI080` roles are explicit and non-overlapping
- the old bounded-backfill default is either reaffirmed or explicitly demoted

## Decision
- Close `INC-0132` positive/explanatory.
- Promote `CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_CPX8_Q01_T2500` as the
  explicit lower-bank default carry-forward route.
- Promote `CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_SBI030_CPX8_Q01_T2500` as
  the balanced lower-bank quality comparator.
- Keep `CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_SBI080_CPX8_Q01_T2500` as the
  quality-first comparator.
- Demote `CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500` to stale
  historical comparator status.

## Next Step
- Queue `INC-0133`: product-phase sparse-event translated lower-bank contract
  refresh.
- Refresh the broader/task-side/downstream packet contracts so the completed
  `INC-0132` selection becomes the new explicit inherited lower-bank default.

## Resume Note
Resume from the completed `INC-0132` selection audit. The next task is no
longer deciding which lower-bank route wins. The next task is refreshing the
broader and downstream contracts to inherit the new lower-bank default once.
