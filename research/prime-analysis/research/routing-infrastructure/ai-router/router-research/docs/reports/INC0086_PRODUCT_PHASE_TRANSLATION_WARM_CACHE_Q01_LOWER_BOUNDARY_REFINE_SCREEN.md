# Translated Hardware Profile

## Source Experiments
- `inc0086_product_phase_translation_warm_cache_q01_lower_boundary_refine_screen`

## Group Summaries
- train=2500, q=1: dense=DENSE_Q01_T2500, crossover=True, quality=H4XH4_FIELD_A150_CPX8_Q01_T2500, systems=H4XH4_FIELD_A150_CPX8_Q01_T2500
  quality_delta_top1=-0.007200, quality_amortized_margin=0.057s
  systems_amortized_margin=0.057s, systems_search_ratio=0.189, systems_cand_frac=0.189016
- train=2500, q=2: dense=DENSE_Q02_T2500, crossover=True, quality=H4XH4_FIELD_A150_CPX8_Q02_T2500, systems=H4XH4_FIELD_A150_CPX8_Q02_T2500
  quality_delta_top1=-0.007200, quality_amortized_margin=0.086s
  systems_amortized_margin=0.086s, systems_search_ratio=0.189, systems_cand_frac=0.189016
- train=2750, q=1: dense=DENSE_Q01_T2750, crossover=True, quality=H4XH4_FIELD_A150_CPX8_Q01_T2750, systems=H4XH4_FIELD_A150_CPX8_Q01_T2750
  quality_delta_top1=-0.008000, quality_amortized_margin=0.052s
  systems_amortized_margin=0.052s, systems_search_ratio=0.188, systems_cand_frac=0.187798
- train=2750, q=2: dense=DENSE_Q02_T2750, crossover=True, quality=H4XH4_FIELD_A150_CPX8_Q02_T2750, systems=H4XH4_FIELD_A150_CPX8_Q02_T2750
  quality_delta_top1=-0.008000, quality_amortized_margin=0.070s
  systems_amortized_margin=0.070s, systems_search_ratio=0.188, systems_cand_frac=0.187798
- train=3000, q=1: dense=DENSE_Q01_T3000, crossover=True, quality=H4XH4_FIELD_A150_CPX8_Q01_T3000, systems=H4XH4_FIELD_A150_CPX8_Q01_T3000
  quality_delta_top1=-0.004000, quality_amortized_margin=0.078s
  systems_amortized_margin=0.078s, systems_search_ratio=0.185, systems_cand_frac=0.185411
- train=3000, q=2: dense=DENSE_Q02_T3000, crossover=True, quality=H4XH4_FIELD_A150_CPX8_Q02_T3000, systems=H4XH4_FIELD_A150_CPX8_Q02_T3000
  quality_delta_top1=-0.004000, quality_amortized_margin=0.098s
  systems_amortized_margin=0.098s, systems_search_ratio=0.185, systems_cand_frac=0.185411

## Bank Summaries
- train=2500: first_any=1, first_quality=1, first_systems=1, systems_family=H4XH4_FIELD_A150_CPX8
  systems_ratio_min=0.189, systems_ratio_max=0.189, systems_ratio_span=0.000, systems_margin_slope=0.028900
- train=2750: first_any=1, first_quality=1, first_systems=1, systems_family=H4XH4_FIELD_A150_CPX8
  systems_ratio_min=0.188, systems_ratio_max=0.188, systems_ratio_span=0.000, systems_margin_slope=0.017535
- train=3000: first_any=1, first_quality=1, first_systems=1, systems_family=H4XH4_FIELD_A150_CPX8
  systems_ratio_min=0.185, systems_ratio_max=0.185, systems_ratio_span=0.000, systems_margin_slope=0.019529

## Route Profiles
- DENSE_Q01_T2500: train=2500, q=1, top1=0.051600, cand_count=2500.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.278, online_per_repeat=0.113s, amortized=0.113s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q01_T2500: train=2500, q=1, top1=0.044400, cand_count=472.540, cand_frac=0.189016, work_ratio_vs_dense=0.189, offline_share=0.012, online_share=0.129, online_per_repeat=0.052s, amortized=0.056s, amortized_margin_vs_dense=0.057s
- DENSE_Q02_T2500: train=2500, q=2, top1=0.051600, cand_count=2500.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.494, online_per_repeat=0.139s, amortized=0.139s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q02_T2500: train=2500, q=2, top1=0.044400, cand_count=472.540, cand_frac=0.189016, work_ratio_vs_dense=0.189, offline_share=0.010, online_share=0.224, online_per_repeat=0.051s, amortized=0.054s, amortized_margin_vs_dense=0.086s
- DENSE_Q01_T2750: train=2750, q=1, top1=0.051636, cand_count=2750.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.279, online_per_repeat=0.116s, amortized=0.116s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q01_T2750: train=2750, q=1, top1=0.043636, cand_count=516.444, cand_frac=0.187798, work_ratio_vs_dense=0.188, offline_share=0.013, online_share=0.145, online_per_repeat=0.059s, amortized=0.064s, amortized_margin_vs_dense=0.052s
- DENSE_Q02_T2750: train=2750, q=2, top1=0.051636, cand_count=2750.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.459, online_per_repeat=0.120s, amortized=0.120s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q02_T2750: train=2750, q=2, top1=0.043636, cand_count=516.444, cand_frac=0.187798, work_ratio_vs_dense=0.188, offline_share=0.012, online_share=0.216, online_per_repeat=0.048s, amortized=0.051s, amortized_margin_vs_dense=0.070s
- DENSE_Q01_T3000: train=3000, q=1, top1=0.048000, cand_count=3000.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.347, online_per_repeat=0.157s, amortized=0.157s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q01_T3000: train=3000, q=1, top1=0.044000, cand_count=556.234, cand_frac=0.185411, work_ratio_vs_dense=0.185, offline_share=0.014, online_share=0.173, online_per_repeat=0.072s, amortized=0.078s, amortized_margin_vs_dense=0.078s
- DENSE_Q02_T3000: train=3000, q=2, top1=0.048000, cand_count=3000.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.530, online_per_repeat=0.159s, amortized=0.159s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q02_T3000: train=3000, q=2, top1=0.044000, cand_count=556.234, cand_frac=0.185411, work_ratio_vs_dense=0.185, offline_share=0.012, online_share=0.245, online_per_repeat=0.058s, amortized=0.061s, amortized_margin_vs_dense=0.098s

