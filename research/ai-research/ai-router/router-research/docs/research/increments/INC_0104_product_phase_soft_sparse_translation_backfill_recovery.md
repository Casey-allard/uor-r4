# INC-0104: Product Phase Soft Sparse Translation Backfill Recovery

## Status
Confirm completed negative on quality recovery and positive/narrow on systems
refinement on 2026-03-12.

## Trigger
`INC-0103` showed that low-margin reranking was not the right recovery path:
- the best rerank point only matched the fixed soft sparse translated
  reference on top-1
- the branch did not improve translated quality
- the next honest rescue surface was bounded small-bucket backfill using the
  already-implemented complex backfill hooks

## Branch Contract
- keep the confirmed `INC-0065` product route law fixed
- keep the confirmed `INC-0100` soft sparse translated point fixed as the
  translated sparse-event quality reference
- reopen translated work only through tightly bounded small-bucket backfill
- do not reopen low-margin backfill, packet mapping, bank mapping, or event
  threshold search inside this branch

## Evidence
- Configs:
  - `configs/proxy_transfer_inc0104_product_phase_soft_sparse_translation_backfill_recovery_prewarm_screen.json`
  - `configs/proxy_transfer_inc0104_product_phase_soft_sparse_translation_backfill_recovery_screen.json`
  - `configs/proxy_transfer_inc0104_product_phase_soft_sparse_translation_backfill_recovery_prewarm_confirm.json`
  - `configs/proxy_transfer_inc0104_product_phase_soft_sparse_translation_backfill_recovery_confirm.json`
- Analyses:
  - `results/analysis/inc0104_product_phase_soft_sparse_translation_backfill_recovery_prewarm_screen.json`
  - `results/analysis/inc0104_product_phase_soft_sparse_translation_backfill_recovery_screen.json`
  - `results/analysis/inc0104_product_phase_soft_sparse_translation_backfill_recovery_prewarm_confirm.json`
  - `results/analysis/inc0104_product_phase_soft_sparse_translation_backfill_recovery_confirm.json`
- Reports:
  - `docs/reports/INC0104_PRODUCT_PHASE_SOFT_SPARSE_TRANSLATION_BACKFILL_RECOVERY_SCREEN.md`
  - `docs/reports/INC0104_PRODUCT_PHASE_SOFT_SPARSE_TRANSLATION_BACKFILL_RECOVERY_CONFIRM.md`
- Gate notes:
  - `docs/governance/gates/gate_20260312_121217.md`
  - `docs/governance/gates/gate_20260312_121232.md`
  - `docs/governance/gates/gate_20260312_121314.md`
  - `docs/governance/gates/gate_20260312_121339.md`

## Screen Read
- The small-bucket surface was worth carrying:
  - fixed soft sparse translated reference:
    - `top1=0.0444`
    - `cand_frac=0.189016`
    - `online=0.15993s`
    - `amortized=0.20460s`
  - `BF1_SB1`:
    - `top1=0.0448`
    - `cand_frac=0.186226`
    - `online=0.16739s`
    - `amortized=0.20674s`
  - `BF1_SB2`:
    - `top1=0.0448`
    - `cand_frac=0.186227`
    - `online=0.16205s`
    - `amortized=0.19534s`
  - `BF2_SB1`:
    - `top1=0.0448`
    - `cand_frac=0.186228`
    - `online=0.27773s`
    - `amortized=0.32600s`
- Screen outcome:
  - the branch was still alive on its own contract
  - `BF1_SB2` looked like the balanced screen point
  - `BF2_SB1` looked too expensive on the screen packet

## Confirm Read
- The quality-recovery claim failed on confirm.
- Continuous translated product reference:
  - `CHART_H4XH4_FIELD_A150_CPX8_Q01_T2500`
    - `top1=0.0446`
    - `cand_frac=0.193328`
    - `online=0.10568s`
    - `amortized=0.14149s`
- Fixed soft sparse translated reference:
  - `CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T2500`
    - `top1=0.0446`
    - `cand_frac=0.193328`
    - `online=0.12081s`
    - `amortized=0.15271s`
- Backfill variants:
  - `CHART_H4XH4_FIELD_A150_EVT_T070_BF1_SB1_CPX8_Q01_T2500`
    - `top1=0.0444`
    - `cand_frac=0.189364`
    - `online=0.11337s`
    - `amortized=0.15852s`
  - `CHART_H4XH4_FIELD_A150_EVT_T070_BF1_SB2_CPX8_Q01_T2500`
    - `top1=0.0444`
    - `cand_frac=0.189364`
    - `online=0.08402s`
    - `amortized=0.11595s`
  - `CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500`
    - `top1=0.0444`
    - `cand_frac=0.189366`
    - `online=0.07954s`
    - `amortized=0.10572s`
- The confirm flipped the screen read:
  - none of the backfill variants recovered quality
  - but `BF2_SB1` became the strongest routed systems point on the packet

## Reading
- `INC-0104` is negative on quality recovery.
- `INC-0104` is positive/narrow on systems refinement:
  - `BF2_SB1` prunes more than the fixed soft sparse translated reference
  - it is materially faster than both the fixed soft sparse translated
    reference and the continuous translated product reference
  - the backfill trigger stays tiny (`0.58%`) and the mean added-candidate
    load stays tiny (`0.0106`)
- The right read is therefore a split:
  - quality reference stays the fixed soft sparse translated point
  - systems lead becomes the bounded backfill point

## Decision
- Close `INC-0104` confirm negative on quality recovery.
- Promote `CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500` as the
  lower-bank sparse translated systems lead.
- Keep `CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T2500` as the lower-bank
  sparse translated quality/reference point.
- Move next to upper-bank carry-forward of the new systems point
  (`INC-0105`), not to more rescue heuristics at `T2500`.

