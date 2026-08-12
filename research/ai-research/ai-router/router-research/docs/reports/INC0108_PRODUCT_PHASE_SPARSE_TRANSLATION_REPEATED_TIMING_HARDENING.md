# Translated Repeated Timing Hardening Audit

## Source
- source_experiment: `inc0104_product_phase_soft_sparse_translation_backfill_recovery_confirm`
- source_experiment: `inc0105_product_phase_soft_sparse_translation_upper_bank_carry_forward_confirm`
- source_experiment: `inc0108_product_phase_sparse_translation_repeated_timing_hardening_lower_r2`
- source_experiment: `inc0108_product_phase_sparse_translation_repeated_timing_hardening_lower_r3`
- source_experiment: `inc0108_product_phase_sparse_translation_repeated_timing_hardening_upper_r2`
- source_experiment: `inc0108_product_phase_sparse_translation_repeated_timing_hardening_upper_r3`

## Comparisons
- `lower_soft_vs_backfill`: baseline=`CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T2500`, candidate=`CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500`, observations=12
  Amortized sign flips across repeats for seeds 0, 1, 2, 3, so timing noise is still present. Route-query remains repeat-mixed for seeds 1, 3. Retrieval-search remains repeat-mixed for seeds 0, 1, 2, 3.
  aggregate amortized: mean=-0.006282, min=-0.142274, max=0.269109, std=0.102754, sign=mixed
  aggregate route_index: mean=0.006018, min=-0.042276, max=0.132303, std=0.041420, sign=mixed
  aggregate route_query: mean=-0.003999, min=-0.033784, max=0.027115, std=0.015623, sign=mixed
  aggregate retrieval_search: mean=-0.008394, min=-0.080605, max=0.109566, std=0.052888, sign=mixed
  aggregate cand_frac: mean=-0.003963, min=-0.008740, max=-0.000792, std=0.003140, sign=stable_improvement
  aggregate top1: mean=-0.000200, min=-0.001600, max=0.000800, std=0.000872, sign=mixed
  Per-seed repeat summaries:
    seed=0: repeats=3, amort_mean=-0.067161, amort_std=0.064865, amort_sign=mixed, route_query_sign=stable_improvement, search_sign=mixed
    seed=1: repeats=3, amort_mean=-0.021692, amort_std=0.050563, amort_sign=mixed, route_query_sign=mixed, search_sign=mixed
    seed=2: repeats=3, amort_mean=-0.005044, amort_std=0.025317, amort_sign=mixed, route_query_sign=stable_regression, search_sign=mixed
    seed=3: repeats=3, amort_mean=0.068767, amort_std=0.158905, amort_sign=mixed, route_query_sign=mixed, search_sign=mixed

- `lower_continuous_vs_backfill`: baseline=`CHART_H4XH4_FIELD_A150_CPX8_Q01_T2500`, candidate=`CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500`, observations=12
  Amortized sign flips across repeats for seeds 1, 2, 3, so timing noise is still present. Route-query remains repeat-mixed for seeds 0, 2, 3. Retrieval-search remains repeat-mixed for seeds 0, 1, 2, 3.
  aggregate amortized: mean=-0.011151, min=-0.234365, max=0.250147, std=0.105931, sign=mixed
  aggregate route_index: mean=-0.001452, min=-0.044209, max=0.107242, std=0.036775, sign=mixed
  aggregate route_query: mean=0.000874, min=-0.023354, max=0.015721, std=0.012192, sign=mixed
  aggregate retrieval_search: mean=-0.010228, min=-0.179480, max=0.129491, std=0.070326, sign=mixed
  aggregate cand_frac: mean=-0.003963, min=-0.008740, max=-0.000792, std=0.003140, sign=stable_improvement
  aggregate top1: mean=-0.000200, min=-0.001600, max=0.000800, std=0.000872, sign=mixed
  Per-seed repeat summaries:
    seed=0: repeats=3, amort_mean=-0.082137, amort_std=0.107681, amort_sign=stable_improvement, route_query_sign=mixed, search_sign=mixed
    seed=1: repeats=3, amort_mean=-0.007998, amort_std=0.023905, amort_sign=mixed, route_query_sign=stable_regression, search_sign=mixed
    seed=2: repeats=3, amort_mean=-0.014729, amort_std=0.023340, amort_sign=mixed, route_query_sign=mixed, search_sign=mixed
    seed=3: repeats=3, amort_mean=0.060261, amort_std=0.148368, amort_sign=mixed, route_query_sign=mixed, search_sign=mixed

- `upper_soft_vs_backfill`: baseline=`CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000`, candidate=`CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T40000`, observations=12
  Amortized sign flips across repeats for seeds 0, 1, 3, so timing noise is still present. Route-query remains repeat-mixed for seeds 0, 1, 3. Retrieval-search remains repeat-mixed for seeds 0, 3.
  aggregate amortized: mean=-0.068437, min=-1.065731, max=1.334614, std=0.553402, sign=mixed
  aggregate route_index: mean=0.007198, min=-0.070849, max=0.161788, std=0.058217, sign=mixed
  aggregate route_query: mean=-0.027565, min=-0.296253, max=0.054412, std=0.087818, sign=mixed
  aggregate retrieval_search: mean=-0.048010, min=-0.969817, max=1.148578, std=0.497891, sign=mixed
  aggregate cand_frac: mean=-0.001761, min=-0.002898, max=-0.001098, std=0.000697, sign=stable_improvement
  aggregate top1: mean=-0.000037, min=-0.000200, max=0.000050, std=0.000096, sign=mixed
  Per-seed repeat summaries:
    seed=0: repeats=3, amort_mean=0.397220, amort_std=0.704481, amort_sign=mixed, route_query_sign=mixed, search_sign=mixed
    seed=1: repeats=3, amort_mean=-0.003258, amort_std=0.166508, amort_sign=mixed, route_query_sign=mixed, search_sign=stable_regression
    seed=2: repeats=3, amort_mean=-0.347883, amort_std=0.045854, amort_sign=stable_improvement, route_query_sign=stable_improvement, search_sign=stable_improvement
    seed=3: repeats=3, amort_mean=-0.319827, amort_std=0.580105, amort_sign=mixed, route_query_sign=mixed, search_sign=mixed

- `upper_continuous_vs_backfill`: baseline=`CHART_H4XH4_FIELD_A150_CPX8_Q01_T40000`, candidate=`CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T40000`, observations=12
  Amortized sign flips across repeats for seeds 0, 1, so timing noise is still present. Route-query remains repeat-mixed for seeds 1, 2, 3. Retrieval-search remains repeat-mixed for seeds 0, 1.
  aggregate amortized: mean=-0.212149, min=-0.775950, max=0.974804, std=0.477172, sign=mixed
  aggregate route_index: mean=0.004095, min=-0.138743, max=0.137861, std=0.079419, sign=mixed
  aggregate route_query: mean=0.008779, min=-0.066319, max=0.090800, std=0.035907, sign=mixed
  aggregate retrieval_search: mean=-0.224346, min=-0.811165, max=0.823734, std=0.455362, sign=mixed
  aggregate cand_frac: mean=-0.001761, min=-0.002898, max=-0.001098, std=0.000697, sign=stable_improvement
  aggregate top1: mean=-0.000037, min=-0.000200, max=0.000050, std=0.000096, sign=mixed
  Per-seed repeat summaries:
    seed=0: repeats=3, amort_mean=0.257642, amort_std=0.508902, amort_sign=mixed, route_query_sign=stable_regression, search_sign=mixed
    seed=1: repeats=3, amort_mean=-0.087469, amort_std=0.388910, amort_sign=mixed, route_query_sign=mixed, search_sign=mixed
    seed=2: repeats=3, amort_mean=-0.503307, amort_std=0.154675, amort_sign=stable_improvement, route_query_sign=mixed, search_sign=stable_improvement
    seed=3: repeats=3, amort_mean=-0.515461, amort_std=0.252183, amort_sign=stable_improvement, route_query_sign=mixed, search_sign=stable_improvement
