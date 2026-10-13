# Translated Robust Cost Reference

## Source
- source_experiment: `inc0104_product_phase_soft_sparse_translation_backfill_recovery_confirm`
- source_experiment: `inc0105_product_phase_soft_sparse_translation_upper_bank_carry_forward_confirm`
- source_experiment: `inc0108_product_phase_sparse_translation_repeated_timing_hardening_lower_r2`
- source_experiment: `inc0108_product_phase_sparse_translation_repeated_timing_hardening_lower_r3`
- source_experiment: `inc0108_product_phase_sparse_translation_repeated_timing_hardening_upper_r2`
- source_experiment: `inc0108_product_phase_sparse_translation_repeated_timing_hardening_upper_r3`
- source_config: `configs/proxy_transfer_inc0104_product_phase_soft_sparse_translation_backfill_recovery_confirm.json`
- source_config: `configs/proxy_transfer_inc0105_product_phase_soft_sparse_translation_upper_bank_carry_forward_confirm.json`
- source_config: `configs/proxy_transfer_inc0108_product_phase_sparse_translation_repeated_timing_hardening_lower_r2.json`
- source_config: `configs/proxy_transfer_inc0108_product_phase_sparse_translation_repeated_timing_hardening_lower_r3.json`
- source_config: `configs/proxy_transfer_inc0108_product_phase_sparse_translation_repeated_timing_hardening_upper_r2.json`
- source_config: `configs/proxy_transfer_inc0108_product_phase_sparse_translation_repeated_timing_hardening_upper_r3.json`

## Overall Read
- Robust systems promotion survives for lower_dense_vs_backfill, upper_dense_vs_soft_sparse, upper_dense_vs_backfill. Stable pruning survives but wallclock stays mixed for lower_dense_vs_soft_sparse.

## Comparisons
- `lower_dense_vs_soft_sparse`: baseline=`DENSE_Q01_T2500`, candidate=`CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T2500`, observations=12, verdict=`pruning_only`
  Amortized cost does not keep a single robust sign once the repeated evidence is summarized by both median and trimmed mean. Candidate-fraction and candidate-count reduction both remain robust. Retrieval-search cost remains a robust part of the advantage. Top-1 stays lower under the robust summaries.
  amortized: mean=0.004257, median=-0.004194, trimmed_mean=0.003878, min=-0.086250, max=0.098558, std=0.051097, sign=mixed, robust=mixed
  route_index: mean=0.035291, median=0.032303, trimmed_mean=0.033295, min=0.013546, max=0.076996, std=0.019561, sign=stable_regression, robust=robust_regression
  route_query: mean=0.026116, median=0.017420, trimmed_mean=0.022911, min=0.005646, max=0.078632, std=0.021202, sign=stable_regression, robust=robust_regression
  retrieval_search: mean=-0.058069, median=-0.059844, trimmed_mean=-0.057192, min=-0.143382, max=0.018473, std=0.040817, sign=mixed, robust=robust_improvement
  cand_frac: mean=-0.806672, median=-0.806804, trimmed_mean=-0.806698, min=-0.811024, max=-0.802055, std=0.004318, sign=stable_improvement, robust=robust_improvement
  cand_count: mean=-2016.679400, median=-2017.010000, trimmed_mean=-2016.745520, min=-2027.560800, max=-2005.136800, std=10.794292, sign=stable_improvement, robust=robust_improvement
  top1: mean=-0.007400, median=-0.007200, trimmed_mean=-0.007360, min=-0.015200, max=0.000000, std=0.005495, sign=stable_regression, robust=robust_regression
  Per-seed robust summaries:
    seed=0: repeats=3, amort_median=0.062152, amort_trimmed=0.062152, amort_robust=robust_regression, search_robust=robust_improvement
    seed=1: repeats=3, amort_median=-0.012749, amort_trimmed=-0.012749, amort_robust=robust_improvement, search_robust=robust_improvement
    seed=2: repeats=3, amort_median=0.004360, amort_trimmed=0.004360, amort_robust=robust_regression, search_robust=robust_improvement
    seed=3: repeats=3, amort_median=-0.027273, amort_trimmed=-0.027273, amort_robust=robust_improvement, search_robust=robust_improvement

- `lower_dense_vs_backfill`: baseline=`DENSE_Q01_T2500`, candidate=`CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500`, observations=12, verdict=`promote`
  Amortized cost stays robustly favorable under both median and trimmed-mean summaries. Candidate-fraction and candidate-count reduction both remain robust. Retrieval-search cost remains a robust part of the advantage. Top-1 stays lower under the robust summaries.
  amortized: mean=-0.002025, median=-0.000506, trimmed_mean=-0.014058, min=-0.125557, max=0.241836, std=0.087748, sign=mixed, robust=robust_improvement
  route_index: mean=0.041309, median=0.028746, trimmed_mean=0.030636, min=0.012001, max=0.177352, std=0.042476, sign=stable_regression, robust=robust_regression
  route_query: mean=0.022117, median=0.014419, trimmed_mean=0.018975, min=0.006283, max=0.069365, std=0.017925, sign=stable_regression, robust=robust_regression
  retrieval_search: mean=-0.066463, median=-0.038671, trimmed_mean=-0.061976, min=-0.171748, max=-0.006048, std=0.053965, sign=stable_improvement, robust=robust_improvement
  cand_frac: mean=-0.810634, median=-0.811305, trimmed_mean=-0.810769, min=-0.815728, max=-0.804200, std=0.004146, sign=stable_improvement, robust=robust_improvement
  cand_count: mean=-2026.586200, median=-2028.262800, trimmed_mean=-2026.921520, min=-2039.319200, max=-2010.500000, std=10.365563, sign=stable_improvement, robust=robust_improvement
  top1: mean=-0.007600, median=-0.006800, trimmed_mean=-0.007440, min=-0.016800, max=0.000000, std=0.006158, sign=stable_regression, robust=robust_regression
  Per-seed robust summaries:
    seed=0: repeats=3, amort_median=0.000996, amort_trimmed=0.000996, amort_robust=robust_regression, search_robust=robust_improvement
    seed=1: repeats=3, amort_median=-0.009030, amort_trimmed=-0.009030, amort_robust=robust_improvement, search_robust=robust_improvement
    seed=2: repeats=3, amort_median=0.007826, amort_trimmed=0.007826, amort_robust=robust_regression, search_robust=robust_improvement
    seed=3: repeats=3, amort_median=0.008414, amort_trimmed=0.008414, amort_robust=robust_regression, search_robust=robust_improvement

- `upper_dense_vs_soft_sparse`: baseline=`DENSE_Q01_T40000`, candidate=`CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000`, observations=12, verdict=`promote`
  Amortized cost stays robustly favorable under both median and trimmed-mean summaries. Candidate-fraction and candidate-count reduction both remain robust. Retrieval-search cost remains a robust part of the advantage. Top-1 stays lower under the robust summaries.
  amortized: mean=-7.405050, median=-7.207634, trimmed_mean=-7.318949, min=-10.574590, max=-5.096514, std=1.634745, sign=stable_improvement, robust=robust_improvement
  route_index: mean=0.289763, median=0.282602, trimmed_mean=0.289487, min=0.228054, max=0.354229, std=0.033271, sign=stable_regression, robust=robust_regression
  route_query: mean=0.189114, median=0.153076, trimmed_mean=0.163306, min=0.115080, max=0.521233, std=0.103824, sign=stable_regression, robust=robust_regression
  retrieval_search: mean=-7.884915, median=-7.643340, trimmed_mean=-7.817563, min=-11.002872, max=-5.440481, std=1.635414, sign=stable_improvement, robust=robust_improvement
  cand_frac: mean=-0.816236, median=-0.813011, trimmed_mean=-0.815591, min=-0.837292, max=-0.801630, std=0.014945, sign=stable_improvement, robust=robust_improvement
  cand_count: mean=-32649.445100, median=-32520.443125, trimmed_mean=-32623.644705, min=-33491.679950, max=-32065.214200, std=597.789539, sign=stable_improvement, robust=robust_improvement
  top1: mean=-0.001525, median=-0.001100, trimmed_mean=-0.001440, min=-0.003050, max=-0.000850, std=0.000887, sign=stable_regression, robust=robust_regression
  Per-seed robust summaries:
    seed=0: repeats=3, amort_median=-10.426297, amort_trimmed=-10.426297, amort_robust=robust_improvement, search_robust=robust_improvement
    seed=1: repeats=3, amort_median=-6.397070, amort_trimmed=-6.397070, amort_robust=robust_improvement, search_robust=robust_improvement
    seed=2: repeats=3, amort_median=-7.676699, amort_trimmed=-7.676699, amort_robust=robust_improvement, search_robust=robust_improvement
    seed=3: repeats=3, amort_median=-6.369328, amort_trimmed=-6.369328, amort_robust=robust_improvement, search_robust=robust_improvement

- `upper_dense_vs_backfill`: baseline=`DENSE_Q01_T40000`, candidate=`CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T40000`, observations=12, verdict=`promote`
  Amortized cost stays robustly favorable under both median and trimmed-mean summaries. Candidate-fraction and candidate-count reduction both remain robust. Retrieval-search cost remains a robust part of the advantage. Top-1 stays lower under the robust summaries.
  amortized: mean=-7.473487, median=-7.080833, trimmed_mean=-7.397552, min=-10.790023, max=-4.916295, std=1.608028, sign=stable_improvement, robust=robust_improvement
  route_index: mean=0.296961, median=0.288586, trimmed_mean=0.290463, min=0.217861, max=0.441044, std=0.059006, sign=stable_regression, robust=robust_regression
  route_query: mean=0.161549, median=0.161815, trimmed_mean=0.160782, min=0.105791, max=0.224980, std=0.034121, sign=stable_regression, robust=robust_regression
  retrieval_search: mean=-7.932925, median=-7.531740, trimmed_mean=-7.861069, min=-11.340101, max=-5.244308, std=1.660205, sign=stable_improvement, robust=robust_improvement
  cand_frac: mean=-0.817997, median=-0.815110, trimmed_mean=-0.817419, min=-0.838390, max=-0.803379, std=0.014407, sign=stable_improvement, robust=robust_improvement
  cand_count: mean=-32719.876513, median=-32604.390525, trimmed_mean=-32696.779315, min=-33535.582500, max=-32135.142500, std=576.262405, sign=stable_improvement, robust=robust_improvement
  top1: mean=-0.001562, median=-0.001100, trimmed_mean=-0.001470, min=-0.003000, max=-0.001050, std=0.000831, sign=stable_regression, robust=robust_regression
  Per-seed robust summaries:
    seed=0: repeats=3, amort_median=-9.239975, amort_trimmed=-9.239975, amort_robust=robust_improvement, search_robust=robust_improvement
    seed=1: repeats=3, amort_median=-6.619893, amort_trimmed=-6.619893, amort_robust=robust_improvement, search_robust=robust_improvement
    seed=2: repeats=3, amort_median=-8.011888, amort_trimmed=-8.011888, amort_robust=robust_improvement, search_robust=robust_improvement
    seed=3: repeats=3, amort_median=-6.020392, amort_trimmed=-6.020392, amort_robust=robust_improvement, search_robust=robust_improvement
