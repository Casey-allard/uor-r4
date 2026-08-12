# INC-0113 Product Phase Sparse Translation Upper-Bank Dense Gap Decomposition

## Summary
- Operational-negligibility threshold: `0.002000`

## Comparisons
### upper_dense_vs_soft_sparse
- Baseline: `DENSE_Q01_T40000`
- Candidate: `CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000`
- Mean dense-only gap rate: `0.030488`
- Mean net dense advantage rate: `0.001525`
- Mean omission share within dense-only wins: `0.011749`
- Mean present-but-not-top1 share within dense-only wins: `0.988251`
- Mean present-outside-topk share within dense-only wins: `0.774163`
- Mean present-inside-topk-not-top1 share within dense-only wins: `0.214088`
- Next-branch bias: `operationally_negligible`
- Read: The net dense advantage rate stays within the configured operational-negligibility band (0.001525 <= 0.002000), so the residual gap is too small to justify another rescue branch.

### upper_dense_vs_backfill
- Baseline: `DENSE_Q01_T40000`
- Candidate: `CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T40000`
- Mean dense-only gap rate: `0.030525`
- Mean net dense advantage rate: `0.001562`
- Mean omission share within dense-only wins: `0.014207`
- Mean present-but-not-top1 share within dense-only wins: `0.985793`
- Mean present-outside-topk share within dense-only wins: `0.772386`
- Mean present-inside-topk-not-top1 share within dense-only wins: `0.213407`
- Next-branch bias: `operationally_negligible`
- Read: The net dense advantage rate stays within the configured operational-negligibility band (0.001562 <= 0.002000), so the residual gap is too small to justify another rescue branch.

## Seed Detail
### upper_dense_vs_soft_sparse
- seed0: dense_only=601, candidate_only=580, omission=4, present_not_top1=597, present_outside_topk=467, present_inside_topk_not_top1=130
- seed1: dense_only=653, candidate_only=592, omission=12, present_not_top1=641, present_outside_topk=503, present_inside_topk_not_top1=138
- seed2: dense_only=590, candidate_only=567, omission=8, present_not_top1=582, present_outside_topk=454, present_inside_topk_not_top1=128
- seed3: dense_only=595, candidate_only=578, omission=5, present_not_top1=590, present_outside_topk=464, present_inside_topk_not_top1=126

### upper_dense_vs_backfill
- seed0: dense_only=601, candidate_only=580, omission=6, present_not_top1=595, present_outside_topk=466, present_inside_topk_not_top1=129
- seed1: dense_only=653, candidate_only=593, omission=13, present_not_top1=640, present_outside_topk=502, present_inside_topk_not_top1=138
- seed2: dense_only=590, candidate_only=567, omission=8, present_not_top1=582, present_outside_topk=454, present_inside_topk_not_top1=128
- seed3: dense_only=598, candidate_only=577, omission=8, present_not_top1=590, present_outside_topk=464, present_inside_topk_not_top1=126
