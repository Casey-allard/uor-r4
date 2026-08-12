# Translated Robust Cost Reference

## Source
- source_experiment: `inc0105_product_phase_soft_sparse_translation_upper_bank_carry_forward_confirm`
- source_experiment: `inc0108_product_phase_sparse_translation_repeated_timing_hardening_upper_r2`
- source_experiment: `inc0108_product_phase_sparse_translation_repeated_timing_hardening_upper_r3`
- source_experiment: `inc0112_product_phase_sparse_translation_upper_bank_dense_quality_tolerance_hardening_r4`
- source_experiment: `inc0112_product_phase_sparse_translation_upper_bank_dense_quality_tolerance_hardening_r5`
- source_config: `configs/proxy_transfer_inc0105_product_phase_soft_sparse_translation_upper_bank_carry_forward_confirm.json`
- source_config: `configs/proxy_transfer_inc0108_product_phase_sparse_translation_repeated_timing_hardening_upper_r2.json`
- source_config: `configs/proxy_transfer_inc0108_product_phase_sparse_translation_repeated_timing_hardening_upper_r3.json`
- source_config: `/Users/adminamn/ai-router/router-research/configs/proxy_transfer_inc0112_product_phase_sparse_translation_upper_bank_dense_quality_tolerance_hardening_r4.json`
- source_config: `/Users/adminamn/ai-router/router-research/configs/proxy_transfer_inc0112_product_phase_sparse_translation_upper_bank_dense_quality_tolerance_hardening_r5.json`

## Overall Read
- Robust systems promotion survives for upper_dense_vs_soft_sparse, upper_dense_vs_backfill.

## Comparisons
- `upper_dense_vs_soft_sparse`: baseline=`DENSE_Q01_T40000`, candidate=`CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000`, observations=20, verdict=`promote`
  Amortized cost stays robustly favorable under both median and trimmed-mean summaries. Candidate-fraction and candidate-count reduction both remain robust. Retrieval-search cost remains a robust part of the advantage. Top-1 stays lower under the robust summaries.
  amortized: mean=-7.300349, median=-7.207634, trimmed_mean=-7.240882, min=-10.574590, max=-5.096514, std=1.379339, sign=stable_improvement, robust=robust_improvement
  route_index: mean=0.313473, median=0.285856, trimmed_mean=0.305281, min=0.228054, max=0.546345, std=0.076418, sign=stable_regression, robust=robust_regression
  route_query: mean=0.175568, median=0.148520, trimmed_mean=0.159724, min=0.115080, max=0.521233, std=0.084277, sign=stable_regression, robust=robust_regression
  retrieval_search: mean=-7.790743, median=-7.643340, trimmed_mean=-7.742861, min=-11.002872, max=-5.440481, std=1.357710, sign=stable_improvement, robust=robust_improvement
  cand_frac: mean=-0.816236, median=-0.813011, trimmed_mean=-0.815878, min=-0.837292, max=-0.801630, std=0.014945, sign=stable_improvement, robust=robust_improvement
  cand_count: mean=-32649.445100, median=-32520.443125, trimmed_mean=-32635.111547, min=-33491.679950, max=-32065.214200, std=597.789539, sign=stable_improvement, robust=robust_improvement
  top1: mean=-0.001525, median=-0.001100, trimmed_mean=-0.001478, min=-0.003050, max=-0.000850, std=0.000887, sign=stable_regression, robust=robust_regression
  Per-seed robust summaries:
    seed=0: repeats=5, amort_median=-8.276648, amort_trimmed=-8.974009, amort_robust=robust_improvement, search_robust=robust_improvement
    seed=1: repeats=5, amort_median=-6.502704, amort_trimmed=-6.682401, amort_robust=robust_improvement, search_robust=robust_improvement
    seed=2: repeats=5, amort_median=-7.402227, amort_trimmed=-7.367372, amort_robust=robust_improvement, search_robust=robust_improvement
    seed=3: repeats=5, amort_median=-6.089038, amort_trimmed=-6.146585, amort_robust=robust_improvement, search_robust=robust_improvement

- `upper_dense_vs_backfill`: baseline=`DENSE_Q01_T40000`, candidate=`CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T40000`, observations=20, verdict=`promote`
  Amortized cost stays robustly favorable under both median and trimmed-mean summaries. Candidate-fraction and candidate-count reduction both remain robust. Retrieval-search cost remains a robust part of the advantage. Top-1 stays lower under the robust summaries.
  amortized: mean=-7.335252, median=-7.129120, trimmed_mean=-7.277707, min=-10.790023, max=-4.916295, std=1.445600, sign=stable_improvement, robust=robust_improvement
  route_index: mean=0.303897, median=0.288901, trimmed_mean=0.301057, min=0.217861, max=0.441044, std=0.055217, sign=stable_regression, robust=robust_regression
  route_query: mean=0.155061, median=0.153017, trimmed_mean=0.153914, min=0.105791, max=0.224980, std=0.028821, sign=stable_regression, robust=robust_regression
  retrieval_search: mean=-7.795168, median=-7.563378, trimmed_mean=-7.739941, min=-11.340101, max=-5.244308, std=1.471568, sign=stable_improvement, robust=robust_improvement
  cand_frac: mean=-0.817997, median=-0.815110, trimmed_mean=-0.817676, min=-0.838390, max=-0.803379, std=0.014407, sign=stable_improvement, robust=robust_improvement
  cand_count: mean=-32719.876513, median=-32604.390525, trimmed_mean=-32707.044736, min=-33535.582500, max=-32135.142500, std=576.262405, sign=stable_improvement, robust=robust_improvement
  top1: mean=-0.001562, median=-0.001100, trimmed_mean=-0.001511, min=-0.003000, max=-0.001050, std=0.000831, sign=stable_regression, robust=robust_regression
  Per-seed robust summaries:
    seed=0: repeats=5, amort_median=-8.798516, amort_trimmed=-8.911005, amort_robust=robust_improvement, search_robust=robust_improvement
    seed=1: repeats=5, amort_median=-6.619893, amort_trimmed=-6.607642, amort_robust=robust_improvement, search_robust=robust_improvement
    seed=2: repeats=5, amort_median=-7.421791, amort_trimmed=-7.525773, amort_robust=robust_improvement, search_robust=robust_improvement
    seed=3: repeats=5, amort_median=-6.020392, amort_trimmed=-5.949976, amort_robust=robust_improvement, search_robust=robust_improvement
