# INC-0105: Product Phase Soft Sparse Translation Upper-Bank Carry-Forward

## Status
Confirm completed positive/narrow on 2026-03-12.

## Trigger
`INC-0104` did not recover quality, but it did produce a better sparse
translated systems point at the lower bank:
- `CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500`
  became the best lower-bank sparse translated systems point
- the next honest question was whether that systems improvement survives at
  the upper bank instead of staying a lower-bank-only effect

## Branch Contract
- keep the confirmed `INC-0065` product route law fixed
- keep the confirmed `INC-0100` sparse translated point fixed as the quality
  reference
- keep the confirmed `INC-0104` bounded backfill point fixed as the lower-bank
  systems lead
- reopen translated work only through upper-bank carry-forward of that fixed
  systems point
- do not reopen new recovery heuristics or new event controllers inside this
  branch

## Evidence
- Configs:
  - `configs/proxy_transfer_inc0105_product_phase_soft_sparse_translation_upper_bank_carry_forward_prewarm_screen.json`
  - `configs/proxy_transfer_inc0105_product_phase_soft_sparse_translation_upper_bank_carry_forward_screen.json`
  - `configs/proxy_transfer_inc0105_product_phase_soft_sparse_translation_upper_bank_carry_forward_prewarm_confirm.json`
  - `configs/proxy_transfer_inc0105_product_phase_soft_sparse_translation_upper_bank_carry_forward_confirm.json`
- Analyses:
  - `results/analysis/inc0105_product_phase_soft_sparse_translation_upper_bank_carry_forward_prewarm_screen.json`
  - `results/analysis/inc0105_product_phase_soft_sparse_translation_upper_bank_carry_forward_screen.json`
  - `results/analysis/inc0105_product_phase_soft_sparse_translation_upper_bank_carry_forward_prewarm_confirm.json`
  - `results/analysis/inc0105_product_phase_soft_sparse_translation_upper_bank_carry_forward_confirm.json`
- Reports:
  - `docs/reports/INC0105_PRODUCT_PHASE_SOFT_SPARSE_TRANSLATION_UPPER_BANK_CARRY_FORWARD_SCREEN.md`
  - `docs/reports/INC0105_PRODUCT_PHASE_SOFT_SPARSE_TRANSLATION_UPPER_BANK_CARRY_FORWARD_CONFIRM.md`
- Gate notes:
  - `docs/governance/gates/gate_20260312_121839.md`
  - `docs/governance/gates/gate_20260312_121941.md`
  - `docs/governance/gates/gate_20260312_122232.md`
  - `docs/governance/gates/gate_20260312_122439.md`

## Screen Read
- The upper-bank screen immediately justified confirm.
- Dense exact:
  - `DENSE_Q01_T40000`
    - `top1=0.049875`
    - `online=11.5419s`
    - `amortized=11.5419s`
- Continuous translated product reference:
  - `CHART_H4XH4_FIELD_A150_CPX8_Q01_T40000`
    - `top1=0.047825`
    - `cand_frac=0.187484`
    - `online=3.25881s`
    - `amortized=3.58754s`
- Fixed soft sparse translated reference:
  - `CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000`
    - `top1=0.047825`
    - `cand_frac=0.187484`
    - `online=2.90606s`
    - `amortized=3.17472s`
- Bounded backfill systems point:
  - `CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T40000`
    - `top1=0.047850`
    - `cand_frac=0.185960`
    - `online=2.77118s`
    - `amortized=3.09168s`
- That was enough to justify confirm because the same fixed lower-bank
  systems point was now also positive on the upper-bank screen.

## Confirm Read
- Dense exact remained the quality ceiling:
  - `DENSE_Q01_T40000`
    - `top1=0.048850`
    - `amortized=11.76079s`
- Continuous translated product reference:
  - `CHART_H4XH4_FIELD_A150_CPX8_Q01_T40000`
    - `top1=0.047325`
    - `cand_frac=0.183764`
    - `online=3.35715s`
    - `amortized=3.67605s`
- Fixed soft sparse translated reference:
  - `CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000`
    - `top1=0.047325`
    - `cand_frac=0.183764`
    - `online=3.22611s`
    - `amortized=3.53342s`
- Bounded backfill systems point:
  - `CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T40000`
    - `top1=0.0472875`
    - `cand_frac=0.182003`
    - `online=3.11690s`
    - `amortized=3.47015s`
- The confirm read is narrow and stable:
  - the bounded backfill point no longer has a screen quality edge
  - but it still improves pruning and runtime versus both routed references
  - and it remains far below dense exact on systems cost

## Reading
- `INC-0105` is positive/narrow as an upper-bank systems carry-forward.
- The right interpretation is the same split already exposed by `INC-0104`:
  - the fixed soft sparse translated point remains the sparse translated
    quality/reference point
  - the bounded backfill point is the sparse translated systems lead
- The systems gain survives from `T2500 Q01` to `T40000 Q01`, so it is not a
  lower-bank accident.

## Decision
- Close `INC-0105` confirm positive/narrow.
- Promote `CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T40000` as the
  upper-bank sparse translated systems lead.
- Keep `CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000` as the upper-bank
  sparse translated quality/reference point.
- Move next to cost decomposition of the new sparse translated systems leads
  (`INC-0106`) rather than to more quality-recovery heuristics.

