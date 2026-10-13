# Translated Hardware Profile

## Source Experiments
- `inc0091_product_phase_translation_warm_cache_q01_2500_2505_refine_screen`

## Group Summaries
- train=2501, q=1: dense=DENSE_Q01_T2501, crossover=True, quality=H4XH4_FIELD_A150_CPX8_Q01_T2501, systems=H4XH4_FIELD_A150_CPX8_Q01_T2501
  quality_delta_top1=-0.007200, quality_amortized_margin=0.049s
  systems_amortized_margin=0.049s, systems_search_ratio=0.189, systems_cand_frac=0.189033
- train=2501, q=2: dense=DENSE_Q02_T2501, crossover=True, quality=H4XH4_FIELD_A150_CPX8_Q02_T2501, systems=H4XH4_FIELD_A150_CPX8_Q02_T2501
  quality_delta_top1=-0.007200, quality_amortized_margin=0.001s
  systems_amortized_margin=0.001s, systems_search_ratio=0.189, systems_cand_frac=0.189033
- train=2502, q=1: dense=DENSE_Q01_T2502, crossover=True, quality=H4XH4_FIELD_A150_CPX8_Q01_T2502, systems=H4XH4_FIELD_A150_CPX8_Q01_T2502
  quality_delta_top1=-0.007194, quality_amortized_margin=0.098s
  systems_amortized_margin=0.098s, systems_search_ratio=0.189, systems_cand_frac=0.189269
- train=2502, q=2: dense=DENSE_Q02_T2502, crossover=True, quality=H4XH4_FIELD_A150_CPX8_Q02_T2502, systems=H4XH4_FIELD_A150_CPX8_Q02_T2502
  quality_delta_top1=-0.007194, quality_amortized_margin=0.065s
  systems_amortized_margin=0.065s, systems_search_ratio=0.189, systems_cand_frac=0.189269
- train=2503, q=1: dense=DENSE_Q01_T2503, crossover=True, quality=H4XH4_FIELD_A150_CPX8_Q01_T2503, systems=H4XH4_FIELD_A150_CPX8_Q01_T2503
  quality_delta_top1=-0.007194, quality_amortized_margin=0.011s
  systems_amortized_margin=0.011s, systems_search_ratio=0.189, systems_cand_frac=0.189285
- train=2503, q=2: dense=DENSE_Q02_T2503, crossover=False, quality=n/a, systems=n/a
- train=2504, q=1: dense=DENSE_Q01_T2504, crossover=True, quality=H4XH4_FIELD_A150_CPX8_Q01_T2504, systems=H4XH4_FIELD_A150_CPX8_Q01_T2504
  quality_delta_top1=-0.006789, quality_amortized_margin=0.019s
  systems_amortized_margin=0.019s, systems_search_ratio=0.189, systems_cand_frac=0.189397
- train=2504, q=2: dense=DENSE_Q02_T2504, crossover=True, quality=H4XH4_FIELD_A150_CPX8_Q02_T2504, systems=H4XH4_FIELD_A150_CPX8_Q02_T2504
  quality_delta_top1=-0.006789, quality_amortized_margin=0.072s
  systems_amortized_margin=0.072s, systems_search_ratio=0.189, systems_cand_frac=0.189397

## Bank Summaries
- train=2501: first_any=1, first_quality=1, first_systems=1, systems_family=H4XH4_FIELD_A150_CPX8
  systems_ratio_min=0.189, systems_ratio_max=0.189, systems_ratio_span=0.000, systems_margin_slope=-0.048005
- train=2502: first_any=1, first_quality=1, first_systems=1, systems_family=H4XH4_FIELD_A150_CPX8
  systems_ratio_min=0.189, systems_ratio_max=0.189, systems_ratio_span=0.000, systems_margin_slope=-0.032581
- train=2503: first_any=1, first_quality=1, first_systems=1, systems_family=H4XH4_FIELD_A150_CPX8
  systems_ratio_min=0.189, systems_ratio_max=0.189, systems_ratio_span=0.000, systems_margin_slope=-0.257656
- train=2504: first_any=1, first_quality=1, first_systems=1, systems_family=H4XH4_FIELD_A150_CPX8
  systems_ratio_min=0.189, systems_ratio_max=0.189, systems_ratio_span=0.000, systems_margin_slope=0.053048

## Route Profiles
- DENSE_Q01_T2501: train=2501, q=1, top1=0.051600, cand_count=2501.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.301, online_per_repeat=0.133s, amortized=0.133s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q01_T2501: train=2501, q=1, top1=0.044400, cand_count=472.770, cand_frac=0.189033, work_ratio_vs_dense=0.189, offline_share=0.012, online_share=0.180, online_per_repeat=0.078s, amortized=0.084s, amortized_margin_vs_dense=0.049s
- DENSE_Q02_T2501: train=2501, q=2, top1=0.051600, cand_count=2501.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.458, online_per_repeat=0.126s, amortized=0.126s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q02_T2501: train=2501, q=2, top1=0.044400, cand_count=472.770, cand_frac=0.189033, work_ratio_vs_dense=0.189, offline_share=0.008, online_share=0.377, online_per_repeat=0.122s, amortized=0.125s, amortized_margin_vs_dense=0.001s
- DENSE_Q01_T2502: train=2502, q=1, top1=0.051559, cand_count=2502.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.344, online_per_repeat=0.159s, amortized=0.159s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q01_T2502: train=2502, q=1, top1=0.044365, cand_count=473.551, cand_frac=0.189269, work_ratio_vs_dense=0.189, offline_share=0.013, online_share=0.145, online_per_repeat=0.056s, amortized=0.061s, amortized_margin_vs_dense=0.098s
- DENSE_Q02_T2502: train=2502, q=2, top1=0.051559, cand_count=2502.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.446, online_per_repeat=0.123s, amortized=0.123s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q02_T2502: train=2502, q=2, top1=0.044365, cand_count=473.551, cand_frac=0.189269, work_ratio_vs_dense=0.189, offline_share=0.011, online_share=0.231, online_per_repeat=0.055s, amortized=0.057s, amortized_margin_vs_dense=0.065s
- DENSE_Q01_T2503: train=2503, q=1, top1=0.051559, cand_count=2503.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.265, online_per_repeat=0.105s, amortized=0.105s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q01_T2503: train=2503, q=1, top1=0.044365, cand_count=473.781, cand_frac=0.189285, work_ratio_vs_dense=0.189, offline_share=0.012, online_share=0.189, online_per_repeat=0.089s, amortized=0.094s, amortized_margin_vs_dense=0.011s
- DENSE_Q02_T2503: train=2503, q=2, top1=0.051559, cand_count=2503.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.392, online_per_repeat=0.092s, amortized=0.092s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q02_T2503: train=2503, q=2, top1=0.044365, cand_count=473.781, cand_frac=0.189285, work_ratio_vs_dense=0.189, offline_share=0.055, online_share=0.512, online_per_repeat=0.306s, amortized=0.339s, amortized_margin_vs_dense=-0.247s
- DENSE_Q01_T2504: train=2504, q=1, top1=0.051118, cand_count=2504.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.222, online_per_repeat=0.085s, amortized=0.085s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q01_T2504: train=2504, q=1, top1=0.044329, cand_count=474.251, cand_frac=0.189397, work_ratio_vs_dense=0.189, offline_share=0.013, online_share=0.148, online_per_repeat=0.061s, amortized=0.066s, amortized_margin_vs_dense=0.019s
- DENSE_Q02_T2504: train=2504, q=2, top1=0.051118, cand_count=2504.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.459, online_per_repeat=0.125s, amortized=0.125s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q02_T2504: train=2504, q=2, top1=0.044329, cand_count=474.251, cand_frac=0.189397, work_ratio_vs_dense=0.189, offline_share=0.011, online_share=0.222, online_per_repeat=0.051s, amortized=0.054s, amortized_margin_vs_dense=0.072s

