# INC-0112: Product Phase Sparse Translation Upper-Bank Dense Quality Tolerance Hardening

## Status
Completed positive/explanatory on 2026-03-12.

## Trigger
`INC-0111` resolved the dense quality/system frontier on completed evidence:
- lower-bank sparse translated dense replacement remains systems-only
- both fixed upper-bank sparse translated points now sit in the
  quality-near systems band
- the next honest question is whether that upper-bank near-frontier read
  survives focused paired hardening, not whether lower-bank rescue can be
  tuned again

## Branch Contract
- keep the confirmed `INC-0065` product route law fixed
- keep the fixed sparse translated upper-bank quality/reference point frozen
- keep the fixed bounded backfill upper-bank sparse translated systems point
  frozen
- reopen dense comparison only through focused upper-bank quality-tolerance
  hardening on those exact fixed points
- do not reopen lower-bank rescue, new sparse controllers, packet remapping,
  or bank-threshold search inside this branch

## Evidence
- Configs:
  - `configs/proxy_transfer_inc0112_product_phase_sparse_translation_upper_bank_dense_quality_tolerance_hardening_prewarm_r4.json`
  - `configs/proxy_transfer_inc0112_product_phase_sparse_translation_upper_bank_dense_quality_tolerance_hardening_r4.json`
  - `configs/proxy_transfer_inc0112_product_phase_sparse_translation_upper_bank_dense_quality_tolerance_hardening_prewarm_r5.json`
  - `configs/proxy_transfer_inc0112_product_phase_sparse_translation_upper_bank_dense_quality_tolerance_hardening_r5.json`
- Analyses:
  - `results/analysis/inc0112_product_phase_sparse_translation_upper_bank_dense_quality_tolerance_hardening_prewarm_r4.json`
  - `results/analysis/inc0112_product_phase_sparse_translation_upper_bank_dense_quality_tolerance_hardening_r4.json`
  - `results/analysis/inc0112_product_phase_sparse_translation_upper_bank_dense_quality_tolerance_hardening_prewarm_r5.json`
  - `results/analysis/inc0112_product_phase_sparse_translation_upper_bank_dense_quality_tolerance_hardening_r5.json`
  - `results/analysis/inc0112_product_phase_sparse_translation_upper_bank_dense_robust_hardening.json`
  - `results/analysis/inc0112_product_phase_sparse_translation_upper_bank_dense_quality_tolerance_hardening.json`
- Reports:
  - `docs/reports/INC0112_PRODUCT_PHASE_SPARSE_TRANSLATION_UPPER_BANK_DENSE_ROBUST_HARDENING.md`
  - `docs/reports/INC0112_PRODUCT_PHASE_SPARSE_TRANSLATION_UPPER_BANK_DENSE_QUALITY_TOLERANCE_HARDENING.md`
- Gate notes:
  - `docs/governance/gates/gate_20260312_145113.md`
  - `docs/governance/gates/gate_20260312_145244.md`
  - `docs/governance/gates/gate_20260312_145601.md`
  - `docs/governance/gates/gate_20260312_145727.md`

## Read
- Two fresh paired upper-bank confirm repeats (`r4` and `r5`) reproduced the
  same fixed sparse translated dense split:
  - `DENSE_Q01_T40000`
    - `top1=0.048850`
  - `CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000`
    - `top1=0.047325`
    - `cand_frac=0.183764`
    - fresh amortized confirms:
      - `r4=2.958464s`
      - `r5=2.801974s`
  - `CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T40000`
    - `top1=0.0472875`
    - `cand_frac=0.182003`
    - fresh amortized confirms:
      - `r4=2.932045s`
      - `r5=2.859189s`
- The upper-bank robust dense systems read strengthens under the additional
  repeats:
  - soft sparse versus dense:
    - robust top-1 gap max abs `0.001478`
    - robust amortized median `-7.207634s`
    - robust amortized trimmed mean `-7.240882s`
  - bounded backfill versus dense:
    - robust top-1 gap max abs `0.001511`
    - robust amortized median `-7.129120s`
    - robust amortized trimmed mean `-7.277707s`
- Both fixed upper-bank sparse translated points remain
  `quality-near systems promotion` under the completed upper-bank-only hardening.
- The branch does not justify a tighter dense replacement statement because the
  top-1 deficit remains robustly negative, even though it stays small.

## Decision
- Close `INC-0112` positive/explanatory.
- Keep both upper-bank sparse translated points as near-frontier dense systems
  promotions.
- Keep bounded backfill as the upper-bank routed systems lead.
- Do not overclaim dense replacement beyond the completed upper-bank
  tolerance band.
- Move next to residual upper-bank dense top-1 gap decomposition (`INC-0113`)
  rather than opening another rescue heuristic immediately.

## Acceptance
- kept the upper-bank dense claim anchored to paired hard evidence
- avoided overclaiming lower-bank sparse translated points as quality-near

## Resume Note
Resume from the completed `INC-0112` upper-bank dense quality-tolerance
hardening branch. The live next branch is residual top-1 gap decomposition on
the fixed upper-bank sparse translated points, not another broad heuristic
rescue pass.
