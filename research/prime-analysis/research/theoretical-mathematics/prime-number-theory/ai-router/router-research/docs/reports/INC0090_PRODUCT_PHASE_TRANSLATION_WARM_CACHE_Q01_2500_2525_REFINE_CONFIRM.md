# Translated Hardware Profile

## Source Experiments
- `inc0090_product_phase_translation_warm_cache_q01_2500_2525_refine_confirm`

## Group Summaries
- train=2505, q=1: dense=DENSE_Q01_T2505, crossover=True, quality=H4XH4_FIELD_A150_CPX8_Q01_T2505, systems=H4XH4_FIELD_A150_CPX8_Q01_T2505
  quality_delta_top1=-0.007388, quality_amortized_margin=0.071s
  systems_amortized_margin=0.071s, systems_search_ratio=0.194, systems_cand_frac=0.193544
- train=2505, q=2: dense=DENSE_Q02_T2505, crossover=True, quality=H4XH4_FIELD_A150_CPX8_Q02_T2505, systems=H4XH4_FIELD_A150_CPX8_Q02_T2505
  quality_delta_top1=-0.007388, quality_amortized_margin=0.076s
  systems_amortized_margin=0.076s, systems_search_ratio=0.194, systems_cand_frac=0.193544
- train=2510, q=1: dense=DENSE_Q01_T2510, crossover=True, quality=H4XH4_FIELD_A150_CPX8_Q01_T2510, systems=H4XH4_FIELD_A150_CPX8_Q01_T2510
  quality_delta_top1=-0.007171, quality_amortized_margin=0.029s
  systems_amortized_margin=0.029s, systems_search_ratio=0.193, systems_cand_frac=0.193187
- train=2510, q=2: dense=DENSE_Q02_T2510, crossover=True, quality=H4XH4_FIELD_A150_CPX8_Q02_T2510, systems=H4XH4_FIELD_A150_CPX8_Q02_T2510
  quality_delta_top1=-0.007171, quality_amortized_margin=0.043s
  systems_amortized_margin=0.043s, systems_search_ratio=0.193, systems_cand_frac=0.193187
- train=2515, q=1: dense=DENSE_Q01_T2515, crossover=True, quality=H4XH4_FIELD_A150_CPX8_Q01_T2515, systems=H4XH4_FIELD_A150_CPX8_Q01_T2515
  quality_delta_top1=-0.007160, quality_amortized_margin=0.033s
  systems_amortized_margin=0.033s, systems_search_ratio=0.193, systems_cand_frac=0.192989
- train=2515, q=2: dense=DENSE_Q02_T2515, crossover=True, quality=H4XH4_FIELD_A150_CPX8_Q02_T2515, systems=H4XH4_FIELD_A150_CPX8_Q02_T2515
  quality_delta_top1=-0.007160, quality_amortized_margin=0.033s
  systems_amortized_margin=0.033s, systems_search_ratio=0.193, systems_cand_frac=0.192989
- train=2520, q=1: dense=DENSE_Q01_T2520, crossover=True, quality=H4XH4_FIELD_A150_CPX8_Q01_T2520, systems=H4XH4_FIELD_A150_CPX8_Q01_T2520
  quality_delta_top1=-0.006746, quality_amortized_margin=0.043s
  systems_amortized_margin=0.043s, systems_search_ratio=0.193, systems_cand_frac=0.193024
- train=2520, q=2: dense=DENSE_Q02_T2520, crossover=True, quality=H4XH4_FIELD_A150_CPX8_Q02_T2520, systems=H4XH4_FIELD_A150_CPX8_Q02_T2520
  quality_delta_top1=-0.006746, quality_amortized_margin=0.037s
  systems_amortized_margin=0.037s, systems_search_ratio=0.193, systems_cand_frac=0.193024

## Bank Summaries
- train=2505: first_any=1, first_quality=1, first_systems=1, systems_family=H4XH4_FIELD_A150_CPX8
  systems_ratio_min=0.194, systems_ratio_max=0.194, systems_ratio_span=0.000, systems_margin_slope=0.004407
- train=2510: first_any=1, first_quality=1, first_systems=1, systems_family=H4XH4_FIELD_A150_CPX8
  systems_ratio_min=0.193, systems_ratio_max=0.193, systems_ratio_span=0.000, systems_margin_slope=0.013750
- train=2515: first_any=1, first_quality=1, first_systems=1, systems_family=H4XH4_FIELD_A150_CPX8
  systems_ratio_min=0.193, systems_ratio_max=0.193, systems_ratio_span=0.000, systems_margin_slope=-0.000626
- train=2520: first_any=1, first_quality=1, first_systems=1, systems_family=H4XH4_FIELD_A150_CPX8
  systems_ratio_min=0.193, systems_ratio_max=0.193, systems_ratio_span=0.000, systems_margin_slope=-0.005444

## Route Profiles
- DENSE_Q01_T2505: train=2505, q=1, top1=0.052117, cand_count=2505.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.302, online_per_repeat=0.129s, amortized=0.129s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q01_T2505: train=2505, q=1, top1=0.044728, cand_count=484.829, cand_frac=0.193544, work_ratio_vs_dense=0.194, offline_share=0.013, online_share=0.133, online_per_repeat=0.053s, amortized=0.058s, amortized_margin_vs_dense=0.071s
- DENSE_Q02_T2505: train=2505, q=2, top1=0.052117, cand_count=2505.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.450, online_per_repeat=0.120s, amortized=0.120s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q02_T2505: train=2505, q=2, top1=0.044728, cand_count=484.829, cand_frac=0.193544, work_ratio_vs_dense=0.194, offline_share=0.012, online_share=0.195, online_per_repeat=0.042s, amortized=0.044s, amortized_margin_vs_dense=0.076s
- DENSE_Q01_T2510: train=2510, q=1, top1=0.051594, cand_count=2510.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.217, online_per_repeat=0.083s, amortized=0.083s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q01_T2510: train=2510, q=1, top1=0.044422, cand_count=484.899, cand_frac=0.193187, work_ratio_vs_dense=0.193, offline_share=0.013, online_share=0.126, online_per_repeat=0.049s, amortized=0.054s, amortized_margin_vs_dense=0.029s
- DENSE_Q02_T2510: train=2510, q=2, top1=0.051594, cand_count=2510.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.397, online_per_repeat=0.094s, amortized=0.094s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q02_T2510: train=2510, q=2, top1=0.044422, cand_count=484.899, cand_frac=0.193187, work_ratio_vs_dense=0.193, offline_share=0.012, online_share=0.218, online_per_repeat=0.049s, amortized=0.052s, amortized_margin_vs_dense=0.043s
- DENSE_Q01_T2515: train=2515, q=1, top1=0.051909, cand_count=2515.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.235, online_per_repeat=0.089s, amortized=0.089s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q01_T2515: train=2515, q=1, top1=0.044749, cand_count=485.369, cand_frac=0.192989, work_ratio_vs_dense=0.193, offline_share=0.013, online_share=0.124, online_per_repeat=0.051s, amortized=0.056s, amortized_margin_vs_dense=0.033s
- DENSE_Q02_T2515: train=2515, q=2, top1=0.051909, cand_count=2515.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.377, online_per_repeat=0.089s, amortized=0.089s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q02_T2515: train=2515, q=2, top1=0.044749, cand_count=485.369, cand_frac=0.192989, work_ratio_vs_dense=0.193, offline_share=0.011, online_share=0.240, online_per_repeat=0.054s, amortized=0.056s, amortized_margin_vs_dense=0.033s
- DENSE_Q01_T2520: train=2520, q=1, top1=0.051587, cand_count=2520.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.257, online_per_repeat=0.101s, amortized=0.101s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q01_T2520: train=2520, q=1, top1=0.044841, cand_count=486.419, cand_frac=0.193024, work_ratio_vs_dense=0.193, offline_share=0.013, online_share=0.132, online_per_repeat=0.053s, amortized=0.058s, amortized_margin_vs_dense=0.043s
- DENSE_Q02_T2520: train=2520, q=2, top1=0.051587, cand_count=2520.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.375, online_per_repeat=0.091s, amortized=0.091s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q02_T2520: train=2520, q=2, top1=0.044841, cand_count=486.419, cand_frac=0.193024, work_ratio_vs_dense=0.193, offline_share=0.011, online_share=0.221, online_per_repeat=0.051s, amortized=0.053s, amortized_margin_vs_dense=0.037s

