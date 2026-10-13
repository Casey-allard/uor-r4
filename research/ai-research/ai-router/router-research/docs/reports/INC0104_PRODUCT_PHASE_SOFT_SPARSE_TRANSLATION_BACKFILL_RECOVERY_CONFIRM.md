# INC-0104 Confirm

- Fixed soft sparse translated quality reference:
  - `CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T2500`
  - `top1=0.0446`, `cand_frac=0.193328`, `amortized=0.15271s`
- Promoted lower-bank sparse translated systems lead:
  - `CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500`
  - `top1=0.0444`, `cand_frac=0.189366`, `amortized=0.10572s`
  - `backfill_trigger_rate=0.0058`, `backfill_extra_candidates=0.0106`
- Read:
  - bounded backfill did not recover quality
  - it did create a clearly better sparse translated systems point at the
    lower bank

