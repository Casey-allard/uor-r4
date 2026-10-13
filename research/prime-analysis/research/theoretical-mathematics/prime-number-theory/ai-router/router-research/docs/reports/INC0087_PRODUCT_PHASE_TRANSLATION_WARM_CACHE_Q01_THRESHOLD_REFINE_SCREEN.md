# Translated Hardware Profile

## Source Experiments
- `inc0087_product_phase_translation_warm_cache_q01_threshold_refine_screen`

## Group Summaries
- train=2600, q=1: dense=DENSE_Q01_T2600, crossover=True, quality=H4XH4_FIELD_A150_CPX8_Q01_T2600, systems=H4XH4_FIELD_A150_CPX8_Q01_T2600
  quality_delta_top1=-0.006538, quality_amortized_margin=0.020s
  systems_amortized_margin=0.020s, systems_search_ratio=0.189, systems_cand_frac=0.188787
- train=2600, q=2: dense=DENSE_Q02_T2600, crossover=True, quality=H4XH4_FIELD_A150_CPX8_Q02_T2600, systems=H4XH4_FIELD_A150_CPX8_Q02_T2600
  quality_delta_top1=-0.006538, quality_amortized_margin=0.059s
  systems_amortized_margin=0.059s, systems_search_ratio=0.189, systems_cand_frac=0.188787
- train=2650, q=1: dense=DENSE_Q01_T2650, crossover=True, quality=H4XH4_FIELD_A150_CPX8_Q01_T2650, systems=H4XH4_FIELD_A150_CPX8_Q01_T2650
  quality_delta_top1=-0.006038, quality_amortized_margin=0.024s
  systems_amortized_margin=0.024s, systems_search_ratio=0.189, systems_cand_frac=0.189479
- train=2650, q=2: dense=DENSE_Q02_T2650, crossover=True, quality=H4XH4_FIELD_A150_CPX8_Q02_T2650, systems=H4XH4_FIELD_A150_CPX8_Q02_T2650
  quality_delta_top1=-0.006038, quality_amortized_margin=0.093s
  systems_amortized_margin=0.093s, systems_search_ratio=0.189, systems_cand_frac=0.189479
- train=2700, q=1: dense=DENSE_Q01_T2700, crossover=True, quality=H4XH4_FIELD_A150_CPX8_Q01_T2700, systems=H4XH4_FIELD_A150_CPX8_Q01_T2700
  quality_delta_top1=-0.007407, quality_amortized_margin=0.036s
  systems_amortized_margin=0.036s, systems_search_ratio=0.188, systems_cand_frac=0.187953
- train=2700, q=2: dense=DENSE_Q02_T2700, crossover=True, quality=H4XH4_FIELD_A150_CPX8_Q02_T2700, systems=H4XH4_FIELD_A150_CPX8_Q02_T2700
  quality_delta_top1=-0.007407, quality_amortized_margin=0.056s
  systems_amortized_margin=0.056s, systems_search_ratio=0.188, systems_cand_frac=0.187953

## Bank Summaries
- train=2600: first_any=1, first_quality=1, first_systems=1, systems_family=H4XH4_FIELD_A150_CPX8
  systems_ratio_min=0.189, systems_ratio_max=0.189, systems_ratio_span=0.000, systems_margin_slope=0.039649
- train=2650: first_any=1, first_quality=1, first_systems=1, systems_family=H4XH4_FIELD_A150_CPX8
  systems_ratio_min=0.189, systems_ratio_max=0.189, systems_ratio_span=0.000, systems_margin_slope=0.069229
- train=2700: first_any=1, first_quality=1, first_systems=1, systems_family=H4XH4_FIELD_A150_CPX8
  systems_ratio_min=0.188, systems_ratio_max=0.188, systems_ratio_span=0.000, systems_margin_slope=0.019462

## Route Profiles
- DENSE_Q01_T2600: train=2600, q=1, top1=0.052692, cand_count=2600.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.285, online_per_repeat=0.118s, amortized=0.118s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q01_T2600: train=2600, q=1, top1=0.046154, cand_count=490.845, cand_frac=0.188787, work_ratio_vs_dense=0.189, offline_share=0.011, online_share=0.204, online_per_repeat=0.093s, amortized=0.098s, amortized_margin_vs_dense=0.020s
- DENSE_Q02_T2600: train=2600, q=2, top1=0.052692, cand_count=2600.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.445, online_per_repeat=0.113s, amortized=0.113s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q02_T2600: train=2600, q=2, top1=0.046154, cand_count=490.845, cand_frac=0.188787, work_ratio_vs_dense=0.189, offline_share=0.010, online_share=0.216, online_per_repeat=0.052s, amortized=0.054s, amortized_margin_vs_dense=0.059s
- DENSE_Q01_T2650: train=2650, q=1, top1=0.051698, cand_count=2650.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.225, online_per_repeat=0.089s, amortized=0.089s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q01_T2650: train=2650, q=1, top1=0.045660, cand_count=502.120, cand_frac=0.189479, work_ratio_vs_dense=0.189, offline_share=0.013, online_share=0.146, online_per_repeat=0.059s, amortized=0.064s, amortized_margin_vs_dense=0.024s
- DENSE_Q02_T2650: train=2650, q=2, top1=0.051698, cand_count=2650.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.505, online_per_repeat=0.149s, amortized=0.149s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q02_T2650: train=2650, q=2, top1=0.045660, cand_count=502.120, cand_frac=0.189479, work_ratio_vs_dense=0.189, offline_share=0.011, online_share=0.235, online_per_repeat=0.053s, amortized=0.056s, amortized_margin_vs_dense=0.093s
- DENSE_Q01_T2700: train=2700, q=1, top1=0.052222, cand_count=2700.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.359, online_per_repeat=0.167s, amortized=0.167s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q01_T2700: train=2700, q=1, top1=0.044815, cand_count=507.473, cand_frac=0.187953, work_ratio_vs_dense=0.188, offline_share=0.010, online_share=0.238, online_per_repeat=0.126s, amortized=0.131s, amortized_margin_vs_dense=0.036s
- DENSE_Q02_T2700: train=2700, q=2, top1=0.052222, cand_count=2700.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.461, online_per_repeat=0.139s, amortized=0.139s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q02_T2700: train=2700, q=2, top1=0.044815, cand_count=507.473, cand_frac=0.187953, work_ratio_vs_dense=0.188, offline_share=0.010, online_share=0.315, online_per_repeat=0.080s, amortized=0.083s, amortized_margin_vs_dense=0.056s

