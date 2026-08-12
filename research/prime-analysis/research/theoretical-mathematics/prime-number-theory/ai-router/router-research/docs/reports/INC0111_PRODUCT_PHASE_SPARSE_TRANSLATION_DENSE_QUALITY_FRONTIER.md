# Product Phase Sparse Translation Dense Quality Frontier

## Summary
- Overall dense-frontier claim: `upper-bank near-frontier; lower-bank systems-only`
- Lower bank read: `systems_only`
- Upper bank read: `near_frontier`
- Quality-near tolerance on robust top-1 gap: `0.002000`

## Comparison Reads
### lower_dense_vs_soft_sparse
- Classification: `pruning-only`
- Systems verdict: `pruning_only`
- Quality read: `quality_negative`
- Baseline `DENSE_Q01_T2500`: `top1=0.052000`, `cand_frac=1.000000`, `amortized=0.125365s`
- Candidate `CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T2500`: `top1=0.044600`, `cand_frac=0.193328`, `amortized=0.152708s`
- Robust top-1 delta: `median=-0.007200`, `trimmed_mean=-0.007360`, `max_abs=0.007360`
- Robust candidate-fraction delta: `median=-0.806804`, `trimmed_mean=-0.806698`
- Robust amortized delta: `median=-0.004194s`, `trimmed_mean=0.003878s`
- Read: Amortized cost does not keep a single robust sign once the repeated evidence is summarized by both median and trimmed mean. Candidate-fraction and candidate-count reduction both remain robust. Retrieval-search cost remains a robust part of the advantage. Top-1 stays lower under the robust summaries.

### lower_dense_vs_backfill
- Classification: `systems-only`
- Systems verdict: `promote`
- Quality read: `quality_negative`
- Baseline `DENSE_Q01_T2500`: `top1=0.052000`, `cand_frac=1.000000`, `amortized=0.125365s`
- Candidate `CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500`: `top1=0.044400`, `cand_frac=0.189366`, `amortized=0.105716s`
- Robust top-1 delta: `median=-0.006800`, `trimmed_mean=-0.007440`, `max_abs=0.007440`
- Robust candidate-fraction delta: `median=-0.811305`, `trimmed_mean=-0.810769`
- Robust amortized delta: `median=-0.000506s`, `trimmed_mean=-0.014058s`
- Read: Amortized cost stays robustly favorable under both median and trimmed-mean summaries. Candidate-fraction and candidate-count reduction both remain robust. Retrieval-search cost remains a robust part of the advantage. Top-1 stays lower under the robust summaries.

### upper_dense_vs_soft_sparse
- Classification: `quality-near systems promotion`
- Systems verdict: `promote`
- Quality read: `quality_near`
- Baseline `DENSE_Q01_T40000`: `top1=0.048850`, `cand_frac=1.000000`, `amortized=11.760794s`
- Candidate `CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000`: `top1=0.047325`, `cand_frac=0.183764`, `amortized=3.533423s`
- Robust top-1 delta: `median=-0.001100`, `trimmed_mean=-0.001440`, `max_abs=0.001440`
- Robust candidate-fraction delta: `median=-0.813011`, `trimmed_mean=-0.815591`
- Robust amortized delta: `median=-7.207634s`, `trimmed_mean=-7.318949s`
- Read: Amortized cost stays robustly favorable under both median and trimmed-mean summaries. Candidate-fraction and candidate-count reduction both remain robust. Retrieval-search cost remains a robust part of the advantage. Top-1 stays lower under the robust summaries.

### upper_dense_vs_backfill
- Classification: `quality-near systems promotion`
- Systems verdict: `promote`
- Quality read: `quality_near`
- Baseline `DENSE_Q01_T40000`: `top1=0.048850`, `cand_frac=1.000000`, `amortized=11.760794s`
- Candidate `CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T40000`: `top1=0.047287`, `cand_frac=0.182003`, `amortized=3.470149s`
- Robust top-1 delta: `median=-0.001100`, `trimmed_mean=-0.001470`, `max_abs=0.001470`
- Robust candidate-fraction delta: `median=-0.815110`, `trimmed_mean=-0.817419`
- Robust amortized delta: `median=-7.080833s`, `trimmed_mean=-7.397552s`
- Read: Amortized cost stays robustly favorable under both median and trimmed-mean summaries. Candidate-fraction and candidate-count reduction both remain robust. Retrieval-search cost remains a robust part of the advantage. Top-1 stays lower under the robust summaries.

## Interpretation
- Lower-bank sparse translated dense replacement remains systems-first.
- Upper-bank sparse translated dense replacement is now near-frontier on quality while staying systems-positive.
- The hardware-side sparse translated claim remains honest: strong systems gains survive, but dense exact still owns the absolute quality ceiling.
