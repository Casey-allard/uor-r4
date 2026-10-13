# INC-0103: Product Phase Soft Sparse Translation Quality Recovery

## Status
Confirm completed negative on 2026-03-12.

## Trigger
`INC-0102` closed negative at screen:
- the confirmed near-hard proxy point did not survive as a translated systems
  improvement
- the translated sparse-event story remained explicitly soft
- the next honest question was whether the fixed soft sparse translated point
  could recover some quality without giving back its lower-bank systems
  position

## Branch Contract
- keep the confirmed `INC-0065` product route law fixed
- keep the confirmed `INC-0100` soft sparse translated point fixed as the
  translated sparse-event reference
- keep near-hard event activation frozen as a proxy-only result
- reopen translated work only through bounded quality recovery on the fixed
  soft sparse translated point
- do not reopen bank, cache, packet, or event-threshold mapping inside this
  branch

## Evidence
- Configs:
  - `configs/proxy_transfer_inc0103_product_phase_soft_sparse_translation_quality_recovery_prewarm_screen.json`
  - `configs/proxy_transfer_inc0103_product_phase_soft_sparse_translation_quality_recovery_screen.json`
  - `configs/proxy_transfer_inc0103_product_phase_soft_sparse_translation_quality_recovery_prewarm_confirm.json`
  - `configs/proxy_transfer_inc0103_product_phase_soft_sparse_translation_quality_recovery_confirm.json`
- Analyses:
  - `results/analysis/inc0103_product_phase_soft_sparse_translation_quality_recovery_prewarm_screen.json`
  - `results/analysis/inc0103_product_phase_soft_sparse_translation_quality_recovery_screen.json`
  - `results/analysis/inc0103_product_phase_soft_sparse_translation_quality_recovery_prewarm_confirm.json`
  - `results/analysis/inc0103_product_phase_soft_sparse_translation_quality_recovery_confirm.json`
- Reports:
  - `docs/reports/INC0103_PRODUCT_PHASE_SOFT_SPARSE_TRANSLATION_QUALITY_RECOVERY_SCREEN.md`
  - `docs/reports/INC0103_PRODUCT_PHASE_SOFT_SPARSE_TRANSLATION_QUALITY_RECOVERY_CONFIRM.md`
- Gate notes:
  - `docs/governance/gates/gate_20260312_120256.md`
  - `docs/governance/gates/gate_20260312_120306.md`
  - `docs/governance/gates/gate_20260312_120447.md`
  - `docs/governance/gates/gate_20260312_120504.md`

## Screen Read
- The screen was strong enough to justify confirm:
  - the fixed soft sparse translated reference stayed at
    `top1=0.0444`, `cand_frac=0.189016`, `amortized=0.13237s`
  - `R025` improved screen top-1 to `0.0448` and cut amortized cost to
    `0.10680s`
  - `R050` improved screen top-1 to `0.0452` with amortized cost
    `0.11437s`
- That made confirm worthwhile because the rerank surface was no longer
  obviously dead on the branch contract.

## Confirm Read
- The quality-recovery claim did not survive hardening.
- Dense exact remained the quality ceiling:
  - `DENSE_Q01_T2500`
    - `top1=0.0520`
    - `amortized=0.17059s`
- Continuous translated product reference:
  - `CHART_H4XH4_FIELD_A150_CPX8_Q01_T2500`
    - `top1=0.0446`
    - `cand_frac=0.193328`
    - `online=0.08323s`
    - `amortized=0.11168s`
- Fixed soft sparse translated reference:
  - `CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T2500`
    - `top1=0.0446`
    - `cand_frac=0.193328`
    - `online=0.07415s`
    - `amortized=0.10683s`
- Bounded rerank variants:
  - `CHART_H4XH4_FIELD_A150_EVT_T070_R025_CPX8_Q01_T2500`
    - `top1=0.0444`
    - `cand_frac=0.193328`
    - `online=0.12653s`
    - `amortized=0.15753s`
  - `CHART_H4XH4_FIELD_A150_EVT_T070_R050_CPX8_Q01_T2500`
    - `top1=0.0446`
    - `cand_frac=0.193328`
    - `online=0.08076s`
    - `amortized=0.10469s`
- `R050` matched the soft sparse translated reference on top-1 and trimmed a
  small amount of amortized cost, but it did not improve quality and it did
  not improve online latency.

## Reading
- `INC-0103` is negative on the thing it set out to prove:
  bounded reranking did not recover translated quality on top of the fixed
  soft sparse point.
- The confirm is still useful:
  - the rerank surface is not catastrophic
  - `R050` behaves like a tiny implementation-side systems trim
  - but it is not a true quality rescue and is not strong enough to promote as
    the new translated sparse-event reference

## Decision
- Close `INC-0103` confirm negative on quality recovery.
- Keep `CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T2500` as the translated
  sparse-event quality reference.
- Do not promote low-margin reranking as the next rescue path for sparse-event
  translation.
- Move next to bounded small-bucket backfill on the same fixed soft sparse
  translated point (`INC-0104`).

