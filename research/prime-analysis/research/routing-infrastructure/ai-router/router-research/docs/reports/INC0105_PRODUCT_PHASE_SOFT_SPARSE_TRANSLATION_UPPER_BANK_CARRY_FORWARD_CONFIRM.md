# INC-0105 Confirm

- Fixed soft sparse translated upper-bank quality/reference point:
  - `CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000`
  - `top1=0.047325`, `cand_frac=0.183764`, `amortized=3.53342s`
- Promoted upper-bank sparse translated systems lead:
  - `CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T40000`
  - `top1=0.0472875`, `cand_frac=0.182003`, `amortized=3.47015s`
- Read:
  - the bounded backfill systems gain survives at the upper bank
  - the quality delta is negligible and slightly negative, so this remains a
    systems lead, not a quality lead

