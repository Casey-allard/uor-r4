# INC-0111: Product Phase Sparse Translation Dense Quality Frontier

## Status
Completed positive/explanatory on 2026-03-12.

## Trigger
`INC-0110` hardened the dense systems claim and showed that it is real but
explicitly systems-first. The lower-bank bounded backfill point is the only
robust lower-bank dense systems promotion, both upper-bank sparse translated
points stay robustly below dense on systems cost, and every sparse translated
dense comparison still carries a robust top-1 deficit. The next honest
question is not more timing hardening; it is whether any fixed sparse
translated point sits inside an acceptable dense-quality tolerance.

## Branch Contract
- keep the confirmed `INC-0065` product route law fixed
- keep the fixed sparse translated quality/reference points frozen
- keep the fixed bounded backfill sparse translated systems points frozen
- reopen dense comparison only through quality/system frontier accounting on
  the completed lower- and upper-bank evidence
- do not introduce new retrieval heuristics, new sparse controllers, or new
  packet/bank mapping inside this branch

## Evidence
- Analyses:
  - `results/analysis/inc0104_product_phase_soft_sparse_translation_backfill_recovery_confirm.json`
  - `results/analysis/inc0105_product_phase_soft_sparse_translation_upper_bank_carry_forward_confirm.json`
  - `results/analysis/inc0110_product_phase_sparse_translation_dense_robust_hardening.json`
  - `results/analysis/inc0111_product_phase_sparse_translation_dense_quality_frontier.json`
- Report:
  - `docs/reports/INC0111_PRODUCT_PHASE_SPARSE_TRANSLATION_DENSE_QUALITY_FRONTIER.md`
- Tooling:
  - `tools/translated_dense_quality_frontier.py`

## Read
- The fixed lower-bank soft sparse translated point stays `pruning-only`
  against dense exact:
  - robust top-1 gap max abs `0.007360`
  - robust candidate-fraction delta median `-0.806804`
  - robust amortized read remains mixed
- The fixed lower-bank bounded backfill sparse translated point stays
  `systems-only` against dense exact:
  - robust top-1 gap max abs `0.007440`
  - robust candidate-fraction delta median `-0.811305`
  - robust amortized median `-0.000506s`
  - robust amortized trimmed mean `-0.014058s`
- Both fixed upper-bank sparse translated points now classify as
  `quality-near systems promotion` under the completed robust summaries:
  - soft sparse upper-bank point:
    - robust top-1 gap max abs `0.001440`
    - robust candidate-fraction delta median `-0.813011`
    - robust amortized median `-7.207634s`
  - bounded backfill upper-bank point:
    - robust top-1 gap max abs `0.001470`
    - robust candidate-fraction delta median `-0.815110`
    - robust amortized median `-7.080833s`
- With the completed dense-quality tolerance read:
  - lower-bank dense claim remains systems-only
  - upper-bank dense claim becomes near-frontier
  - the project can now distinguish narrow hardware-side sparse wins from
    actual quality-near dense replacement pressure

## Decision
- Close `INC-0111` positive/explanatory.
- Keep `CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500` as the
  lower-bank sparse translated dense systems point, but do not overclaim it as
  quality-near.
- Keep both upper-bank sparse translated points as near-frontier dense
  systems promotions, with bounded backfill still the routed systems lead.
- Move next to focused upper-bank dense quality-tolerance hardening
  (`INC-0112`) rather than reopening lower-bank rescue, new sparse
  controllers, or fresh frontier heuristics.

## Acceptance
- produced an explicit dense quality/system frontier read for the fixed sparse
  translated points
- kept the moonshot hardware claim honest about where quality still trails

## Resume Note
Resume from the completed `INC-0111` dense quality-frontier audit. The next
branch is focused upper-bank dense quality-tolerance hardening on the fixed
sparse translated points, not more lower-bank threshold work or new recovery
heuristics.
