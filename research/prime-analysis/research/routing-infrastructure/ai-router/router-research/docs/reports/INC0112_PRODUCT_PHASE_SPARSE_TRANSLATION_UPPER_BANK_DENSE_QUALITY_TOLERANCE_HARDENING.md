# Product Phase Sparse Translation Upper-Bank Dense Quality Tolerance Hardening

## Summary
- Overall dense-frontier claim: `upper-bank near-frontier`
- Lower bank read: `not_evaluated`
- Upper bank read: `near_frontier`
- Quality-near tolerance on robust top-1 gap: `0.002000`

## Comparison Reads
### upper_dense_vs_soft_sparse
- Classification: `quality-near systems promotion`
- Systems verdict: `promote`
- Quality read: `quality_near`
- Baseline `DENSE_Q01_T40000`: `top1=0.048850`, `cand_frac=1.000000`, `amortized=9.733566s`
- Candidate `CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000`: `top1=0.047325`, `cand_frac=0.183764`, `amortized=2.801974s`
- Robust top-1 delta: `median=-0.001100`, `trimmed_mean=-0.001478`, `max_abs=0.001478`
- Robust candidate-fraction delta: `median=-0.813011`, `trimmed_mean=-0.815878`
- Robust amortized delta: `median=-7.207634s`, `trimmed_mean=-7.240882s`
- Read: Amortized cost stays robustly favorable under both median and trimmed-mean summaries. Candidate-fraction and candidate-count reduction both remain robust. Retrieval-search cost remains a robust part of the advantage. Top-1 stays lower under the robust summaries.

### upper_dense_vs_backfill
- Classification: `quality-near systems promotion`
- Systems verdict: `promote`
- Quality read: `quality_near`
- Baseline `DENSE_Q01_T40000`: `top1=0.048850`, `cand_frac=1.000000`, `amortized=9.733566s`
- Candidate `CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T40000`: `top1=0.047287`, `cand_frac=0.182003`, `amortized=2.859189s`
- Robust top-1 delta: `median=-0.001100`, `trimmed_mean=-0.001511`, `max_abs=0.001511`
- Robust candidate-fraction delta: `median=-0.815110`, `trimmed_mean=-0.817676`
- Robust amortized delta: `median=-7.129120s`, `trimmed_mean=-7.277707s`
- Read: Amortized cost stays robustly favorable under both median and trimmed-mean summaries. Candidate-fraction and candidate-count reduction both remain robust. Retrieval-search cost remains a robust part of the advantage. Top-1 stays lower under the robust summaries.

## Interpretation
- Upper-bank sparse translated dense replacement is now near-frontier on quality while staying systems-positive.
- The hardware-side sparse translated claim remains honest: strong systems gains survive, but dense exact still owns the absolute quality ceiling.
