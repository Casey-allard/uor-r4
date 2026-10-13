# Translated Hardware Profile

## Source Experiments
- `inc0091_product_phase_translation_warm_cache_q01_2500_2505_refine_confirm`

## Group Summaries
- train=2501, q=1: dense=DENSE_Q01_T2501, crossover=True, quality=H4XH4_FIELD_A150_CPX8_Q01_T2501, systems=H4XH4_FIELD_A150_CPX8_Q01_T2501
  quality_delta_top1=-0.007200, quality_amortized_margin=0.058s
  systems_amortized_margin=0.058s, systems_search_ratio=0.193, systems_cand_frac=0.193338
- train=2501, q=2: dense=DENSE_Q02_T2501, crossover=True, quality=H4XH4_FIELD_A150_CPX8_Q02_T2501, systems=H4XH4_FIELD_A150_CPX8_Q02_T2501
  quality_delta_top1=-0.007200, quality_amortized_margin=0.046s
  systems_amortized_margin=0.046s, systems_search_ratio=0.193, systems_cand_frac=0.193338
- train=2502, q=1: dense=DENSE_Q01_T2502, crossover=True, quality=H4XH4_FIELD_A150_CPX8_Q01_T2502, systems=H4XH4_FIELD_A150_CPX8_Q01_T2502
  quality_delta_top1=-0.007394, quality_amortized_margin=0.038s
  systems_amortized_margin=0.038s, systems_search_ratio=0.194, systems_cand_frac=0.193503
- train=2502, q=2: dense=DENSE_Q02_T2502, crossover=True, quality=H4XH4_FIELD_A150_CPX8_Q02_T2502, systems=H4XH4_FIELD_A150_CPX8_Q02_T2502
  quality_delta_top1=-0.007394, quality_amortized_margin=0.044s
  systems_amortized_margin=0.044s, systems_search_ratio=0.194, systems_cand_frac=0.193503
- train=2503, q=1: dense=DENSE_Q01_T2503, crossover=True, quality=H4XH4_FIELD_A150_CPX8_Q01_T2503, systems=H4XH4_FIELD_A150_CPX8_Q01_T2503
  quality_delta_top1=-0.007194, quality_amortized_margin=0.048s
  systems_amortized_margin=0.048s, systems_search_ratio=0.194, systems_cand_frac=0.193513
- train=2503, q=2: dense=DENSE_Q02_T2503, crossover=True, quality=H4XH4_FIELD_A150_CPX8_Q02_T2503, systems=H4XH4_FIELD_A150_CPX8_Q02_T2503
  quality_delta_top1=-0.007194, quality_amortized_margin=0.039s
  systems_amortized_margin=0.039s, systems_search_ratio=0.194, systems_cand_frac=0.193513
- train=2504, q=1: dense=DENSE_Q01_T2504, crossover=True, quality=H4XH4_FIELD_A150_CPX8_Q01_T2504, systems=H4XH4_FIELD_A150_CPX8_Q01_T2504
  quality_delta_top1=-0.007188, quality_amortized_margin=0.073s
  systems_amortized_margin=0.073s, systems_search_ratio=0.194, systems_cand_frac=0.193565
- train=2504, q=2: dense=DENSE_Q02_T2504, crossover=True, quality=H4XH4_FIELD_A150_CPX8_Q02_T2504, systems=H4XH4_FIELD_A150_CPX8_Q02_T2504
  quality_delta_top1=-0.007188, quality_amortized_margin=0.085s
  systems_amortized_margin=0.085s, systems_search_ratio=0.194, systems_cand_frac=0.193565

## Bank Summaries
- train=2501: first_any=1, first_quality=1, first_systems=1, systems_family=H4XH4_FIELD_A150_CPX8
  systems_ratio_min=0.193, systems_ratio_max=0.193, systems_ratio_span=0.000, systems_margin_slope=-0.012894
- train=2502: first_any=1, first_quality=1, first_systems=1, systems_family=H4XH4_FIELD_A150_CPX8
  systems_ratio_min=0.194, systems_ratio_max=0.194, systems_ratio_span=0.000, systems_margin_slope=0.005788
- train=2503: first_any=1, first_quality=1, first_systems=1, systems_family=H4XH4_FIELD_A150_CPX8
  systems_ratio_min=0.194, systems_ratio_max=0.194, systems_ratio_span=0.000, systems_margin_slope=-0.008770
- train=2504: first_any=1, first_quality=1, first_systems=1, systems_family=H4XH4_FIELD_A150_CPX8
  systems_ratio_min=0.194, systems_ratio_max=0.194, systems_ratio_span=0.000, systems_margin_slope=0.011771

## Route Profiles
- DENSE_Q01_T2501: train=2501, q=1, top1=0.051800, cand_count=2501.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.319, online_per_repeat=0.136s, amortized=0.136s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q01_T2501: train=2501, q=1, top1=0.044600, cand_count=483.539, cand_frac=0.193338, work_ratio_vs_dense=0.193, offline_share=0.011, online_share=0.162, online_per_repeat=0.073s, amortized=0.078s, amortized_margin_vs_dense=0.058s
- DENSE_Q02_T2501: train=2501, q=2, top1=0.051800, cand_count=2501.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.471, online_per_repeat=0.125s, amortized=0.125s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q02_T2501: train=2501, q=2, top1=0.044600, cand_count=483.539, cand_frac=0.193338, work_ratio_vs_dense=0.193, offline_share=0.010, online_share=0.291, online_per_repeat=0.077s, amortized=0.080s, amortized_margin_vs_dense=0.046s
- DENSE_Q01_T2502: train=2502, q=1, top1=0.052158, cand_count=2502.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.283, online_per_repeat=0.121s, amortized=0.121s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q01_T2502: train=2502, q=1, top1=0.044764, cand_count=484.145, cand_frac=0.193503, work_ratio_vs_dense=0.194, offline_share=0.011, online_share=0.168, online_per_repeat=0.078s, amortized=0.083s, amortized_margin_vs_dense=0.038s
- DENSE_Q02_T2502: train=2502, q=2, top1=0.052158, cand_count=2502.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.422, online_per_repeat=0.123s, amortized=0.123s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q02_T2502: train=2502, q=2, top1=0.044764, cand_count=484.145, cand_frac=0.193503, work_ratio_vs_dense=0.194, offline_share=0.010, online_share=0.292, online_per_repeat=0.076s, amortized=0.079s, amortized_margin_vs_dense=0.044s
- DENSE_Q01_T2503: train=2503, q=1, top1=0.051958, cand_count=2503.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.276, online_per_repeat=0.113s, amortized=0.113s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q01_T2503: train=2503, q=1, top1=0.044764, cand_count=484.364, cand_frac=0.193513, work_ratio_vs_dense=0.194, offline_share=0.012, online_share=0.141, online_per_repeat=0.059s, amortized=0.064s, amortized_margin_vs_dense=0.048s
- DENSE_Q02_T2503: train=2503, q=2, top1=0.051958, cand_count=2503.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.385, online_per_repeat=0.092s, amortized=0.092s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q02_T2503: train=2503, q=2, top1=0.044764, cand_count=484.364, cand_frac=0.193513, work_ratio_vs_dense=0.194, offline_share=0.011, online_share=0.210, online_per_repeat=0.050s, amortized=0.052s, amortized_margin_vs_dense=0.039s
- DENSE_Q01_T2504: train=2504, q=1, top1=0.051917, cand_count=2504.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.304, online_per_repeat=0.132s, amortized=0.132s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q01_T2504: train=2504, q=1, top1=0.044728, cand_count=484.688, cand_frac=0.193565, work_ratio_vs_dense=0.194, offline_share=0.013, online_share=0.134, online_per_repeat=0.054s, amortized=0.059s, amortized_margin_vs_dense=0.073s
- DENSE_Q02_T2504: train=2504, q=2, top1=0.051917, cand_count=2504.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.468, online_per_repeat=0.133s, amortized=0.133s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q02_T2504: train=2504, q=2, top1=0.044728, cand_count=484.688, cand_frac=0.193565, work_ratio_vs_dense=0.194, offline_share=0.012, online_share=0.207, online_per_repeat=0.046s, amortized=0.048s, amortized_margin_vs_dense=0.085s

