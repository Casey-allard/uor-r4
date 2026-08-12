# Translated Hardware Profile

## Source Experiments
- `inc0087_product_phase_translation_warm_cache_q01_threshold_refine_confirm`

## Group Summaries
- train=2600, q=1: dense=DENSE_Q01_T2600, crossover=True, quality=H4XH4_FIELD_A150_CPX8_Q01_T2600, systems=H4XH4_FIELD_A150_CPX8_Q01_T2600
  quality_delta_top1=-0.006346, quality_amortized_margin=0.002s
  systems_amortized_margin=0.002s, systems_search_ratio=0.194, systems_cand_frac=0.193587
- train=2600, q=2: dense=DENSE_Q02_T2600, crossover=True, quality=H4XH4_FIELD_A150_CPX8_Q02_T2600, systems=H4XH4_FIELD_A150_CPX8_Q02_T2600
  quality_delta_top1=-0.006346, quality_amortized_margin=0.040s
  systems_amortized_margin=0.040s, systems_search_ratio=0.194, systems_cand_frac=0.193587
- train=2650, q=1: dense=DENSE_Q01_T2650, crossover=False, quality=n/a, systems=n/a
- train=2650, q=2: dense=DENSE_Q02_T2650, crossover=True, quality=H4XH4_FIELD_A150_CPX8_Q02_T2650, systems=H4XH4_FIELD_A150_CPX8_Q02_T2650
  quality_delta_top1=-0.006604, quality_amortized_margin=0.027s
  systems_amortized_margin=0.027s, systems_search_ratio=0.194, systems_cand_frac=0.193916
- train=2700, q=1: dense=DENSE_Q01_T2700, crossover=True, quality=H4XH4_FIELD_A150_CPX8_Q01_T2700, systems=H4XH4_FIELD_A150_CPX8_Q01_T2700
  quality_delta_top1=-0.007593, quality_amortized_margin=0.037s
  systems_amortized_margin=0.037s, systems_search_ratio=0.193, systems_cand_frac=0.193139
- train=2700, q=2: dense=DENSE_Q02_T2700, crossover=True, quality=H4XH4_FIELD_A150_CPX8_Q02_T2700, systems=H4XH4_FIELD_A150_CPX8_Q02_T2700
  quality_delta_top1=-0.007593, quality_amortized_margin=0.091s
  systems_amortized_margin=0.091s, systems_search_ratio=0.193, systems_cand_frac=0.193139

## Bank Summaries
- train=2600: first_any=1, first_quality=1, first_systems=1, systems_family=H4XH4_FIELD_A150_CPX8
  systems_ratio_min=0.194, systems_ratio_max=0.194, systems_ratio_span=0.000, systems_margin_slope=0.037838
- train=2650: first_any=2, first_quality=2, first_systems=2, systems_family=H4XH4_FIELD_A150_CPX8
  systems_ratio_min=0.194, systems_ratio_max=0.194, systems_ratio_span=0.000, systems_margin_slope=0.044012
- train=2700: first_any=1, first_quality=1, first_systems=1, systems_family=H4XH4_FIELD_A150_CPX8
  systems_ratio_min=0.193, systems_ratio_max=0.193, systems_ratio_span=0.000, systems_margin_slope=0.054019

## Route Profiles
- DENSE_Q01_T2600: train=2600, q=1, top1=0.051923, cand_count=2600.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.284, online_per_repeat=0.118s, amortized=0.118s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q01_T2600: train=2600, q=1, top1=0.045577, cand_count=503.327, cand_frac=0.193587, work_ratio_vs_dense=0.194, offline_share=0.011, online_share=0.226, online_per_repeat=0.111s, amortized=0.116s, amortized_margin_vs_dense=0.002s
- DENSE_Q02_T2600: train=2600, q=2, top1=0.051923, cand_count=2600.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.451, online_per_repeat=0.117s, amortized=0.117s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q02_T2600: train=2600, q=2, top1=0.045577, cand_count=503.327, cand_frac=0.193587, work_ratio_vs_dense=0.194, offline_share=0.010, online_share=0.285, online_per_repeat=0.075s, amortized=0.077s, amortized_margin_vs_dense=0.040s
- DENSE_Q01_T2650: train=2650, q=1, top1=0.051132, cand_count=2650.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.245, online_per_repeat=0.097s, amortized=0.097s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q01_T2650: train=2650, q=1, top1=0.044528, cand_count=513.877, cand_frac=0.193916, work_ratio_vs_dense=0.194, offline_share=0.011, online_share=0.226, online_per_repeat=0.109s, amortized=0.114s, amortized_margin_vs_dense=-0.017s
- DENSE_Q02_T2650: train=2650, q=2, top1=0.051132, cand_count=2650.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.445, online_per_repeat=0.118s, amortized=0.118s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q02_T2650: train=2650, q=2, top1=0.044528, cand_count=513.877, cand_frac=0.193916, work_ratio_vs_dense=0.194, offline_share=0.010, online_share=0.331, online_per_repeat=0.088s, amortized=0.091s, amortized_margin_vs_dense=0.027s
- DENSE_Q01_T2700: train=2700, q=1, top1=0.051852, cand_count=2700.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.294, online_per_repeat=0.122s, amortized=0.122s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q01_T2700: train=2700, q=1, top1=0.044259, cand_count=521.476, cand_frac=0.193139, work_ratio_vs_dense=0.193, offline_share=0.012, online_share=0.187, online_per_repeat=0.080s, amortized=0.085s, amortized_margin_vs_dense=0.037s
- DENSE_Q02_T2700: train=2700, q=2, top1=0.051852, cand_count=2700.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.491, online_per_repeat=0.147s, amortized=0.147s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q02_T2700: train=2700, q=2, top1=0.044259, cand_count=521.476, cand_frac=0.193139, work_ratio_vs_dense=0.193, offline_share=0.011, online_share=0.230, online_per_repeat=0.053s, amortized=0.056s, amortized_margin_vs_dense=0.091s

