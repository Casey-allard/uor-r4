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
- Robust systems promotion survives for lower_continuous_vs_backfill, upper_soft_vs_backfill, upper_continuous_vs_backfill. Stable pruning survives but wallclock stays mixed for lower_soft_vs_backfill.

## Comparisons
- `lower_soft_vs_backfill`: baseline=`CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T2500`, candidate=`CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500`, observations=12, verdict=`pruning_only`
  Amortized cost does not keep a single robust sign once the repeated evidence is summarized by both median and trimmed mean. Candidate-fraction and candidate-count reduction both remain robust. Retrieval-search cost still changes sign across the repeated evidence. Route-query also contributes robustly. Top-1 deltas remain small and mixed.
  amortized: mean=-0.006282, median=0.003406, trimmed_mean=-0.020222, min=-0.142274, max=0.269109, std=0.102754, sign=mixed, robust=mixed
  route_index: mean=0.006018, median=0.001877, trimmed_mean=-0.001781, min=-0.042276, max=0.132303, std=0.041420, sign=mixed, robust=mixed
  route_query: mean=-0.003999, median=-0.000430, trimmed_mean=-0.004131, min=-0.033784, max=0.027115, std=0.015623, sign=mixed, robust=robust_improvement
  retrieval_search: mean=-0.008394, median=0.000496, trimmed_mean=-0.012969, min=-0.080605, max=0.109566, std=0.052888, sign=mixed, robust=mixed
  cand_frac: mean=-0.003963, median=-0.003160, trimmed_mean=-0.003802, min=-0.008740, max=-0.000792, std=0.003140, sign=stable_improvement, robust=robust_improvement
  cand_count: mean=-9.906800, median=-7.899600, trimmed_mean=-9.505360, min=-21.848800, max=-1.979200, std=7.849994, sign=stable_improvement, robust=robust_improvement
  top1: mean=-0.000200, median=0.000000, trimmed_mean=-0.000160, min=-0.001600, max=0.000800, std=0.000872, sign=mixed, robust=mixed
  Per-seed robust summaries:
    seed=0: repeats=3, amort_median=-0.075208, amort_trimmed=-0.075208, amort_robust=robust_improvement, search_robust=robust_improvement
    seed=1: repeats=3, amort_median=0.003719, amort_trimmed=0.003719, amort_robust=robust_regression, search_robust=robust_regression
    seed=2: repeats=3, amort_median=0.003092, amort_trimmed=0.003092, amort_robust=robust_regression, search_robust=robust_improvement
    seed=3: repeats=3, amort_median=0.056764, amort_trimmed=0.056764, amort_robust=robust_regression, search_robust=robust_regression

- `lower_continuous_vs_backfill`: baseline=`CHART_H4XH4_FIELD_A150_CPX8_Q01_T2500`, candidate=`CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500`, observations=12, verdict=`promote`
  Amortized cost stays robustly favorable under both median and trimmed-mean summaries. Candidate-fraction and candidate-count reduction both remain robust. Retrieval-search cost still changes sign across the repeated evidence. Top-1 deltas remain small and mixed.
  amortized: mean=-0.011151, median=-0.011214, trimmed_mean=-0.014959, min=-0.234365, max=0.250147, std=0.105931, sign=mixed, robust=robust_improvement
  route_index: mean=-0.001452, median=-0.006679, trimmed_mean=-0.008046, min=-0.044209, max=0.107242, std=0.036775, sign=mixed, robust=robust_improvement
  route_query: mean=0.000874, median=0.002840, trimmed_mean=0.001813, min=-0.023354, max=0.015721, std=0.012192, sign=mixed, robust=robust_regression
  retrieval_search: mean=-0.010228, median=0.000006, trimmed_mean=-0.007274, min=-0.179480, max=0.129491, std=0.070326, sign=mixed, robust=mixed
  cand_frac: mean=-0.003963, median=-0.003160, trimmed_mean=-0.003802, min=-0.008740, max=-0.000792, std=0.003140, sign=stable_improvement, robust=robust_improvement
  cand_count: mean=-9.906800, median=-7.899600, trimmed_mean=-9.505360, min=-21.848800, max=-1.979200, std=7.849994, sign=stable_improvement, robust=robust_improvement
  top1: mean=-0.000200, median=0.000000, trimmed_mean=-0.000160, min=-0.001600, max=0.000800, std=0.000872, sign=mixed, robust=mixed
  Per-seed robust summaries:
    seed=0: repeats=3, amort_median=-0.009590, amort_trimmed=-0.009590, amort_robust=robust_improvement, search_robust=robust_improvement
    seed=1: repeats=3, amort_median=-0.015797, amort_trimmed=-0.015797, amort_robust=robust_improvement, search_robust=robust_regression
    seed=2: repeats=3, amort_median=-0.012838, amort_trimmed=-0.012838, amort_robust=robust_improvement, search_robust=robust_improvement
    seed=3: repeats=3, amort_median=0.042630, amort_trimmed=0.042630, amort_robust=robust_regression, search_robust=robust_regression

- `upper_soft_vs_backfill`: baseline=`CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000`, candidate=`CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T40000`, observations=12, verdict=`promote`
  Amortized cost stays robustly favorable under both median and trimmed-mean summaries. Candidate-fraction and candidate-count reduction both remain robust. Retrieval-search cost remains a robust part of the advantage. Route-query also contributes robustly. Top-1 deltas remain small and mixed.
  amortized: mean=-0.068437, median=-0.232755, trimmed_mean=-0.109013, min=-1.065731, max=1.334614, std=0.553402, sign=mixed, robust=robust_improvement
  route_index: mean=0.007198, median=-0.009013, trimmed_mean=-0.000456, min=-0.070849, max=0.161788, std=0.058217, sign=mixed, robust=robust_improvement
  route_query: mean=-0.027565, median=-0.015338, trimmed_mean=-0.008894, min=-0.296253, max=0.054412, std=0.087818, sign=mixed, robust=robust_improvement
  retrieval_search: mean=-0.048010, median=-0.084093, trimmed_mean=-0.075488, min=-0.969817, max=1.148578, std=0.497891, sign=mixed, robust=robust_improvement
  cand_frac: mean=-0.001761, median=-0.001524, trimmed_mean=-0.001713, min=-0.002898, max=-0.001098, std=0.000697, sign=stable_improvement, robust=robust_improvement
  cand_count: mean=-70.431413, median=-60.956300, trimmed_mean=-68.536390, min=-115.910500, max=-43.902550, std=27.895685, sign=stable_improvement, robust=robust_improvement
  top1: mean=-0.000037, median=0.000000, trimmed_mean=-0.000030, min=-0.000200, max=0.000050, std=0.000096, sign=mixed, robust=mixed
  Per-seed robust summaries:
    seed=0: repeats=3, amort_median=0.220772, amort_trimmed=0.220772, amort_robust=robust_regression, search_robust=robust_regression
    seed=1: repeats=3, amort_median=0.032830, amort_trimmed=0.032830, amort_robust=robust_regression, search_robust=robust_regression
    seed=2: repeats=3, amort_median=-0.335189, amort_trimmed=-0.335189, amort_robust=robust_improvement, search_robust=robust_improvement
    seed=3: repeats=3, amort_median=-0.242687, amort_trimmed=-0.242687, amort_robust=robust_improvement, search_robust=robust_improvement

- `upper_continuous_vs_backfill`: baseline=`CHART_H4XH4_FIELD_A150_CPX8_Q01_T40000`, candidate=`CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T40000`, observations=12, verdict=`promote`
  Amortized cost stays robustly favorable under both median and trimmed-mean summaries. Candidate-fraction and candidate-count reduction both remain robust. Retrieval-search cost remains a robust part of the advantage. Top-1 deltas remain small and mixed.
  amortized: mean=-0.212149, median=-0.270871, trimmed_mean=-0.274464, min=-0.775950, max=0.974804, std=0.477172, sign=mixed, robust=robust_improvement
  route_index: mean=0.004095, median=-0.002224, trimmed_mean=0.005003, min=-0.138743, max=0.137861, std=0.079419, sign=mixed, robust=mixed
  route_query: mean=0.008779, median=0.006301, trimmed_mean=0.008086, min=-0.066319, max=0.090800, std=0.035907, sign=mixed, robust=robust_regression
  retrieval_search: mean=-0.224346, median=-0.277300, trimmed_mean=-0.270472, min=-0.811165, max=0.823734, std=0.455362, sign=mixed, robust=robust_improvement
  cand_frac: mean=-0.001761, median=-0.001524, trimmed_mean=-0.001713, min=-0.002898, max=-0.001098, std=0.000697, sign=stable_improvement, robust=robust_improvement
  cand_count: mean=-70.431413, median=-60.956300, trimmed_mean=-68.536390, min=-115.910500, max=-43.902550, std=27.895685, sign=stable_improvement, robust=robust_improvement
  top1: mean=-0.000037, median=0.000000, trimmed_mean=-0.000030, min=-0.000200, max=0.000050, std=0.000096, sign=mixed, robust=mixed
  Per-seed robust summaries:
    seed=0: repeats=3, amort_median=-0.048675, amort_trimmed=-0.048675, amort_robust=robust_improvement, search_robust=robust_improvement
    seed=1: repeats=3, amort_median=-0.004773, amort_trimmed=-0.004773, amort_robust=robust_improvement, search_robust=robust_regression
    seed=2: repeats=3, amort_median=-0.422724, amort_trimmed=-0.422724, amort_robust=robust_improvement, search_robust=robust_improvement
    seed=3: repeats=3, amort_median=-0.596176, amort_trimmed=-0.596176, amort_robust=robust_improvement, search_robust=robust_improvement
