# INC-0114 Product Phase Sparse Translation Upper-Bank Dense Reference Selection

## Summary
- Carry-forward contract: `single_promoted_reference`
- Selection mode: `tie_break_within_tolerance`
- Promoted route: `CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000`
- Supporting route: `CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T40000`
- Read: CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000 is promoted as the single upper-bank dense-near routed reference. The remaining delta versus CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T40000 stays inside the configured top-1, amortized, and candidate-fraction tolerance band, so the supporting route can drop to comparator status.

## Tolerances
- Top-1 tolerance: `0.000100`
- Amortized tolerance: `0.050000s`
- Candidate-fraction tolerance: `0.002000`

## Upper-Bank Points
- Soft sparse `CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000`: `top1=0.047325`, `cand_frac=0.183764`, `amortized=3.426262s`, `classification=quality-near systems promotion`, `gap_bias=operationally_negligible`
- Bounded backfill `CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T40000`: `top1=0.047287`, `cand_frac=0.182003`, `amortized=3.470096s`, `classification=quality-near systems promotion`, `gap_bias=operationally_negligible`

## Pair Deltas
- Soft minus backfill top-1: `0.000038`
- Soft minus backfill candidate fraction: `0.001761`
- Soft minus backfill amortized: `-0.043834s`
