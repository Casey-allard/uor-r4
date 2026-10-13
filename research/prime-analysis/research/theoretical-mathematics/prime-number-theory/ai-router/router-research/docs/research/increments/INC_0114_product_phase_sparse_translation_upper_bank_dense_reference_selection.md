# INC-0114: Product Phase Sparse Translation Upper-Bank Dense Reference Selection

## Status
Completed positive/explanatory on 2026-03-12.

## Trigger
`INC-0113` closed the residual upper-bank dense gap as operationally
negligible:
- both fixed upper-bank sparse translated points remain quality-near systems
  promotions
- the net dense advantage rate stays inside the `0.002` operational band
- dense-only wins are overwhelmingly in-candidate rather than omission-led

The next honest question was no longer rescue. It was whether one of the two
fixed upper-bank sparse translated points should become the single promoted
dense-near routed reference for later carry-forward work, or whether the pair
should remain explicit as a quality/reference point plus systems lead.

## Branch Contract
- keep the confirmed `INC-0065` product route law fixed
- keep the lower-bank dense read frozen as systems-only
- keep the two fixed upper-bank sparse translated points frozen:
  - `CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000`
  - `CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T40000`
- reopen the upper-bank dense story only through reference-selection and
  carry-forward accounting
- do not reopen dense-gap rescue, lower-bank remapping, or new sparse
  controller search inside this branch

## Minimal Scope
1. Build a compact promotion audit on the completed `INC-0112` and `INC-0113`
   evidence.
2. Decide whether the upper-bank branch now has:
   - one promoted dense-near routed reference
   - or an irreducible quality/reference versus systems-lead pair
3. Record the resulting reference selection as the new upper-bank carry-forward
   contract for later broader hardware-side or task-side comparison.

## Acceptance
- stop treating upper-bank dense rescue as an active branch once reference
  selection is explicit
- keep the promoted read tied to the actual completed evidence, not to a new
  heuristic packet

## Evidence
- Analyses:
  - `results/analysis/inc0112_product_phase_sparse_translation_upper_bank_dense_quality_tolerance_hardening.json`
  - `results/analysis/inc0113_product_phase_sparse_translation_upper_bank_dense_gap_decomposition.json`
  - `results/analysis/inc0113_product_phase_sparse_translation_upper_bank_dense_gap_decomposition_confirm.json`
  - `results/analysis/inc0114_product_phase_sparse_translation_upper_bank_dense_reference_selection.json`
- Report:
  - `docs/reports/INC0114_PRODUCT_PHASE_SPARSE_TRANSLATION_UPPER_BANK_DENSE_REFERENCE_SELECTION.md`

## Reading
- Both upper-bank sparse translated points stayed eligible for carry-forward:
  - both are `quality-near systems promotion` under `INC-0112`
  - both are `operationally_negligible` under `INC-0113`
- Under the completed `INC-0113` confirm packet:
  - soft sparse `CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000`
    - `top1=0.047325`
    - `cand_frac=0.183764`
    - `amortized=3.426262s`
  - bounded backfill `CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T40000`
    - `top1=0.0472875`
    - `cand_frac=0.182003`
    - `amortized=3.470096s`
- Pair deltas stay inside the configured tie-break tolerance band:
  - top-1 tolerance `0.000100`, observed delta `+0.000038`
  - amortized tolerance `0.050000s`, observed delta `-0.043834s`
  - candidate-fraction tolerance `0.002000`, observed delta `+0.001761`
- The pair now collapses cleanly to a single promoted upper-bank dense-near
  routed reference:
  - promoted route:
    `CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000`
  - supporting comparator:
    `CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T40000`
  - selection mode: `tie_break_within_tolerance`
- This closes the branch positive/explanatory:
  - upper-bank dense rescue remains closed
  - the upper-bank pair no longer needs to stay live as a branch fork
  - future carry-forward work can use the promoted soft sparse upper-bank
    reference as the single dense-near routed representative

## Resume Note
Resume from the completed `INC-0114` upper-bank reference selection. The next
branch should carry the promoted upper-bank dense-near routed reference
forward, not reopen the old upper-bank pair as a live fork.
