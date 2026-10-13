# Translated Hardware Profile

## Source Experiments
- `inc0090_product_phase_translation_warm_cache_q01_2500_2525_refine_screen`

## Group Summaries
- train=2505, q=1: dense=DENSE_Q01_T2505, crossover=True, quality=H4XH4_FIELD_A150_CPX8_Q01_T2505, systems=H4XH4_FIELD_A150_CPX8_Q01_T2505
  quality_delta_top1=-0.007188, quality_amortized_margin=0.072s
  systems_amortized_margin=0.072s, systems_search_ratio=0.189, systems_cand_frac=0.189414
- train=2505, q=2: dense=DENSE_Q02_T2505, crossover=True, quality=H4XH4_FIELD_A150_CPX8_Q02_T2505, systems=H4XH4_FIELD_A150_CPX8_Q02_T2505
  quality_delta_top1=-0.007188, quality_amortized_margin=0.042s
  systems_amortized_margin=0.042s, systems_search_ratio=0.189, systems_cand_frac=0.189414
- train=2510, q=1: dense=DENSE_Q01_T2510, crossover=True, quality=H4XH4_FIELD_A150_CPX8_Q01_T2510, systems=H4XH4_FIELD_A150_CPX8_Q01_T2510
  quality_delta_top1=-0.006773, quality_amortized_margin=0.040s
  systems_amortized_margin=0.040s, systems_search_ratio=0.189, systems_cand_frac=0.188774
- train=2510, q=2: dense=DENSE_Q02_T2510, crossover=True, quality=H4XH4_FIELD_A150_CPX8_Q02_T2510, systems=H4XH4_FIELD_A150_CPX8_Q02_T2510
  quality_delta_top1=-0.006773, quality_amortized_margin=0.035s
  systems_amortized_margin=0.035s, systems_search_ratio=0.189, systems_cand_frac=0.188774
- train=2515, q=1: dense=DENSE_Q01_T2515, crossover=True, quality=H4XH4_FIELD_A150_CPX8_Q01_T2515, systems=H4XH4_FIELD_A150_CPX8_Q01_T2515
  quality_delta_top1=-0.007160, quality_amortized_margin=0.036s
  systems_amortized_margin=0.036s, systems_search_ratio=0.189, systems_cand_frac=0.188558
- train=2515, q=2: dense=DENSE_Q02_T2515, crossover=True, quality=H4XH4_FIELD_A150_CPX8_Q02_T2515, systems=H4XH4_FIELD_A150_CPX8_Q02_T2515
  quality_delta_top1=-0.007160, quality_amortized_margin=0.000s
  systems_amortized_margin=0.000s, systems_search_ratio=0.189, systems_cand_frac=0.188558
- train=2520, q=1: dense=DENSE_Q01_T2520, crossover=True, quality=H4XH4_FIELD_A150_CPX8_Q01_T2520, systems=H4XH4_FIELD_A150_CPX8_Q01_T2520
  quality_delta_top1=-0.006349, quality_amortized_margin=0.077s
  systems_amortized_margin=0.077s, systems_search_ratio=0.188, systems_cand_frac=0.188411
- train=2520, q=2: dense=DENSE_Q02_T2520, crossover=False, quality=n/a, systems=n/a

## Bank Summaries
- train=2505: first_any=1, first_quality=1, first_systems=1, systems_family=H4XH4_FIELD_A150_CPX8
  systems_ratio_min=0.189, systems_ratio_max=0.189, systems_ratio_span=0.000, systems_margin_slope=-0.029170
- train=2510: first_any=1, first_quality=1, first_systems=1, systems_family=H4XH4_FIELD_A150_CPX8
  systems_ratio_min=0.189, systems_ratio_max=0.189, systems_ratio_span=0.000, systems_margin_slope=-0.005081
- train=2515: first_any=1, first_quality=1, first_systems=1, systems_family=H4XH4_FIELD_A150_CPX8
  systems_ratio_min=0.189, systems_ratio_max=0.189, systems_ratio_span=0.000, systems_margin_slope=-0.036326
- train=2520: first_any=1, first_quality=1, first_systems=1, systems_family=H4XH4_FIELD_A150_CPX8
  systems_ratio_min=0.188, systems_ratio_max=0.188, systems_ratio_span=0.000, systems_margin_slope=-0.080459

## Route Profiles
- DENSE_Q01_T2505: train=2505, q=1, top1=0.051518, cand_count=2505.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.270, online_per_repeat=0.124s, amortized=0.124s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q01_T2505: train=2505, q=1, top1=0.044329, cand_count=474.481, cand_frac=0.189414, work_ratio_vs_dense=0.189, offline_share=0.013, online_share=0.115, online_per_repeat=0.047s, amortized=0.053s, amortized_margin_vs_dense=0.072s
- DENSE_Q02_T2505: train=2505, q=2, top1=0.051518, cand_count=2505.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.425, online_per_repeat=0.109s, amortized=0.109s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q02_T2505: train=2505, q=2, top1=0.044329, cand_count=474.481, cand_frac=0.189414, work_ratio_vs_dense=0.189, offline_share=0.011, online_share=0.264, online_per_repeat=0.064s, amortized=0.066s, amortized_margin_vs_dense=0.042s
- DENSE_Q01_T2510: train=2510, q=1, top1=0.050598, cand_count=2510.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.251, online_per_repeat=0.096s, amortized=0.096s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q01_T2510: train=2510, q=1, top1=0.043825, cand_count=473.823, cand_frac=0.188774, work_ratio_vs_dense=0.189, offline_share=0.011, online_share=0.118, online_per_repeat=0.051s, amortized=0.056s, amortized_margin_vs_dense=0.040s
- DENSE_Q02_T2510: train=2510, q=2, top1=0.050598, cand_count=2510.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.375, online_per_repeat=0.086s, amortized=0.086s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q02_T2510: train=2510, q=2, top1=0.043825, cand_count=473.823, cand_frac=0.188774, work_ratio_vs_dense=0.189, offline_share=0.011, online_share=0.220, online_per_repeat=0.048s, amortized=0.051s, amortized_margin_vs_dense=0.035s
- DENSE_Q01_T2515: train=2515, q=1, top1=0.051710, cand_count=2515.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.264, online_per_repeat=0.105s, amortized=0.105s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q01_T2515: train=2515, q=1, top1=0.044551, cand_count=474.223, cand_frac=0.188558, work_ratio_vs_dense=0.189, offline_share=0.014, online_share=0.156, online_per_repeat=0.063s, amortized=0.069s, amortized_margin_vs_dense=0.036s
- DENSE_Q02_T2515: train=2515, q=2, top1=0.051710, cand_count=2515.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.377, online_per_repeat=0.089s, amortized=0.089s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q02_T2515: train=2515, q=2, top1=0.044551, cand_count=474.223, cand_frac=0.188558, work_ratio_vs_dense=0.189, offline_share=0.009, online_share=0.320, online_per_repeat=0.087s, amortized=0.089s, amortized_margin_vs_dense=0.000s
- DENSE_Q01_T2520: train=2520, q=1, top1=0.051190, cand_count=2520.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.310, online_per_repeat=0.132s, amortized=0.132s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q01_T2520: train=2520, q=1, top1=0.044841, cand_count=474.796, cand_frac=0.188411, work_ratio_vs_dense=0.188, offline_share=0.012, online_share=0.125, online_per_repeat=0.051s, amortized=0.056s, amortized_margin_vs_dense=0.077s
- DENSE_Q02_T2520: train=2520, q=2, top1=0.051190, cand_count=2520.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.374, online_per_repeat=0.089s, amortized=0.089s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q02_T2520: train=2520, q=2, top1=0.044841, cand_count=474.796, cand_frac=0.188411, work_ratio_vs_dense=0.188, offline_share=0.009, online_share=0.320, online_per_repeat=0.090s, amortized=0.092s, amortized_margin_vs_dense=-0.004s

