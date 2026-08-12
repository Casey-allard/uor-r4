# INC-0103 Confirm

- Fixed soft sparse translated reference:
  - `CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T2500`
  - `top1=0.0446`, `cand_frac=0.193328`, `amortized=0.10683s`
- Best rerank point:
  - `CHART_H4XH4_FIELD_A150_EVT_T070_R050_CPX8_Q01_T2500`
  - `top1=0.0446`, `cand_frac=0.193328`, `amortized=0.10469s`
- Read:
  - confirm killed the quality-recovery claim
  - `R050` only matched top-1 and shaved a tiny amount of amortized cost
  - branch closes negative on quality recovery

