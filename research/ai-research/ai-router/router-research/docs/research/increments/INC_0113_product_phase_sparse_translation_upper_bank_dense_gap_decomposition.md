# INC-0113: Product Phase Sparse Translation Upper-Bank Dense Gap Decomposition

## Status
Completed positive/explanatory on 2026-03-12.

## Trigger
`INC-0112` hardened the upper-bank sparse translated dense claim and kept both
fixed sparse translated points inside the near-frontier quality band:
- strong dense systems gains remain robust
- the residual dense top-1 gap stays small
- but that gap is still robustly negative

The next honest question was not another rescue heuristic. It was whether the
remaining upper-bank dense gap came mostly from candidate omission or from
in-candidate ordering loss.

## Branch Contract
- keep the confirmed `INC-0065` product route law fixed
- keep the fixed upper-bank sparse translated quality/reference point frozen
- keep the fixed upper-bank bounded backfill sparse translated systems point
  frozen
- reopen dense comparison only through residual top-1 gap decomposition on the
  completed upper-bank evidence
- do not reopen lower-bank rescue, new sparse controllers, or fresh bank
  mapping inside this branch

## Scope
1. Build a query-level audit on the fixed upper-bank dense packet comparing
   dense exact with the two fixed sparse translated points.
2. Partition the residual top-1 gap into:
   - target absent from routed candidate set
   - target present but ranked below the routed top-1
3. Decide whether the next branch, if any, should target:
   - candidate recovery
   - in-candidate ordering
   - or no further rescue because the remaining gap is already operationally
     negligible

## Acceptance
- keep the upper-bank near-frontier claim mechanism-specific
- avoid guessing whether the residual dense gap is pruning loss or ranking loss

## Evidence
- Configs:
  - `configs/proxy_transfer_inc0113_product_phase_sparse_translation_upper_bank_dense_gap_decomposition_prewarm_confirm.json`
  - `configs/proxy_transfer_inc0113_product_phase_sparse_translation_upper_bank_dense_gap_decomposition_confirm.json`
- Analyses:
  - `results/analysis/inc0113_product_phase_sparse_translation_upper_bank_dense_gap_decomposition_confirm.json`
  - `results/analysis/inc0113_product_phase_sparse_translation_upper_bank_dense_gap_decomposition.json`
- Report:
  - `docs/reports/INC0113_PRODUCT_PHASE_SPARSE_TRANSLATION_UPPER_BANK_DENSE_GAP_DECOMPOSITION.md`

## Reading
- `DENSE_Q01_T40000` remained the exact upper-bank dense reference:
  - `top1=0.048850`
  - `cand_frac=1.000000`
  - `amortized=16.770385s`
- `CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000` stayed the upper-bank
  quality/reference point:
  - `top1=0.047325`
  - `cand_frac=0.183764`
  - `amortized=3.426262s`
- `CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T40000` stayed the
  upper-bank systems lead:
  - `top1=0.0472875`
  - `cand_frac=0.182003`
  - `amortized=3.470096s`
- The residual dense gap is operationally negligible under the completed
  upper-bank quality tolerance:
  - soft sparse mean net dense advantage rate `0.001525`
  - bounded backfill mean net dense advantage rate `0.001562`
  - both remain inside the `0.002000` operational-negligibility band
- Dense-only wins are overwhelmingly not omission-led:
  - omission share within dense-only wins is only `0.011749` for soft sparse
    and `0.014207` for bounded backfill
  - present-but-not-top1 explains `0.988251` and `0.985793` of dense-only wins
  - most of that residual is still outside routed top-k rather than target
    absence
- This closes the branch positive/explanatory:
  - the upper-bank sparse translated dense claim no longer needs a rescue queue
  - the next branch should promote or carry forward the accepted upper-bank
    near-frontier points rather than reopen quality repair

## Resume Note
Resume from the completed `INC-0113` upper-bank dense gap decomposition. The
next branch is upper-bank dense reference selection/carry-forward on the fixed
sparse translated points, not another dense-gap rescue.
