# INC-0110: Product Phase Sparse Translation Dense Robust Hardening

## Status
Completed positive/explanatory on 2026-03-12.

## Trigger
`INC-0109` turned the repeated sparse translated evidence into a stable robust
reference. That reference keeps the upper-bank bounded backfill point promoted,
keeps the lower-bank bounded backfill point robust versus the continuous
translated product reference, and narrows the lower-bank soft sparse comparison
to a pruning-first read. The next honest question is whether the fixed sparse
translated points also keep the actual dense-exact systems claim under the same
kind of repeated robust timing treatment.

## Branch Contract
- keep the confirmed `INC-0065` product route law fixed
- keep the fixed sparse translated quality/reference points frozen
- keep the fixed bounded backfill sparse translated systems points frozen
- reopen translated work only through repeated dense-frontier hardening on the
  existing lower- and upper-bank `Q01` sparse translated anchors
- do not introduce new recovery heuristics, new sparse controllers, or new
  packet/bank mapping inside this branch

## Evidence
- Analyses:
  - `results/analysis/inc0104_product_phase_soft_sparse_translation_backfill_recovery_confirm.json`
  - `results/analysis/inc0105_product_phase_soft_sparse_translation_upper_bank_carry_forward_confirm.json`
  - `results/analysis/inc0108_product_phase_sparse_translation_repeated_timing_hardening_lower_r2.json`
  - `results/analysis/inc0108_product_phase_sparse_translation_repeated_timing_hardening_lower_r3.json`
  - `results/analysis/inc0108_product_phase_sparse_translation_repeated_timing_hardening_upper_r2.json`
  - `results/analysis/inc0108_product_phase_sparse_translation_repeated_timing_hardening_upper_r3.json`
  - `results/analysis/inc0110_product_phase_sparse_translation_dense_robust_hardening.json`
- Report:
  - `docs/reports/INC0110_PRODUCT_PHASE_SPARSE_TRANSLATION_DENSE_ROBUST_HARDENING.md`
- Tooling:
  - `tools/translated_robust_cost_reference.py`

## Read
- The lower-bank fixed soft sparse translated point does not keep a clean
  robust dense systems win:
  - amortized median `-0.004194s`
  - amortized trimmed mean `+0.003878s`
  - candidate-fraction median `-0.806804`
  - candidate-count median `-2017.0100`
  - top-1 median `-0.007200`
  - this is a pruning-first read, not a clean robust dense systems promotion
- The lower-bank bounded backfill sparse translated point does keep a robust
  dense systems promotion:
  - amortized median `-0.000506s`
  - amortized trimmed mean `-0.014058s`
  - candidate-fraction median `-0.811305`
  - candidate-count median `-2028.2628`
  - top-1 median `-0.006800`
- Both upper-bank sparse translated points keep robust dense systems
  promotions:
  - soft sparse reference:
    - amortized median `-7.207634s`
    - amortized trimmed mean `-7.318949s`
    - candidate-fraction median `-0.813011`
    - candidate-count median `-32520.4431`
    - top-1 median `-0.001100`
  - bounded backfill systems point:
    - amortized median `-7.080833s`
    - amortized trimmed mean `-7.397552s`
    - candidate-fraction median `-0.815110`
    - candidate-count median `-32604.3905`
    - top-1 median `-0.001100`
- The robust dense-frontier story is therefore real but narrow:
  - lower bank: only the bounded backfill point keeps a robust dense systems
    promotion
  - upper bank: both sparse translated points stay robustly below dense on
    systems cost
  - every sparse translated dense comparison still carries a robust top-1
    regression versus dense exact
- The dominant mechanism remains retrieval-search pruning, not route overhead:
  - all four dense comparisons show robust retrieval-search savings
  - route-index build and route-query remain robust penalties in every dense
    comparison
  - the dense systems win is therefore search-dominated rather than route-
    materialization dominated

## Decision
- Close `INC-0110` positive/explanatory.
- Keep `CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500` as the only
  robust lower-bank sparse translated dense systems point.
- Keep both upper-bank sparse translated points as robust dense systems
  promotions, with bounded backfill remaining the routed systems lead.
- Treat the dense claim as explicitly systems-first, not quality-matched:
  - lower-bank top-1 gap remains material
  - upper-bank top-1 gap remains small but still robustly negative
- Move next to a dense quality-frontier audit (`INC-0111`) rather than to more
  timing hardening or new retrieval heuristics.

## Acceptance
- produce a robust dense-frontier read for the fixed sparse translated points
- keep the hardware-side systems claim honest under repeated timing

## Resume Note
Resume from the completed `INC-0110` dense robust hardening audit. The next
branch is dense quality-frontier accounting on the fixed sparse translated
points, not another timing-only hardening pass.
