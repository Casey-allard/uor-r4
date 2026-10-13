# Translated Hardware Profile

## Source Experiments
- `inc0092_product_phase_translation_warm_cache_q01_floor_hardening_screen`

## Group Summaries
- train=2500, q=1: dense=DENSE_Q01_T2500, crossover=True, quality=H4XH4_FIELD_A150_CPX8_Q01_T2500, systems=H4XH4_FIELD_A150_CPX8_Q01_T2500
  quality_delta_top1=-0.007400, quality_amortized_margin=0.064s
  systems_amortized_margin=0.064s, systems_search_ratio=0.193, systems_cand_frac=0.193328
- train=2500, q=2: dense=DENSE_Q02_T2500, crossover=True, quality=H4XH4_FIELD_A150_CPX8_Q02_T2500, systems=H4XH4_FIELD_A150_CPX8_Q02_T2500
  quality_delta_top1=-0.007400, quality_amortized_margin=0.020s
  systems_amortized_margin=0.020s, systems_search_ratio=0.193, systems_cand_frac=0.193328
- train=2501, q=1: dense=DENSE_Q01_T2501, crossover=True, quality=H4XH4_FIELD_A150_CPX8_Q01_T2501, systems=H4XH4_FIELD_A150_CPX8_Q01_T2501
  quality_delta_top1=-0.007200, quality_amortized_margin=0.014s
  systems_amortized_margin=0.014s, systems_search_ratio=0.193, systems_cand_frac=0.193338
- train=2501, q=2: dense=DENSE_Q02_T2501, crossover=True, quality=H4XH4_FIELD_A150_CPX8_Q02_T2501, systems=H4XH4_FIELD_A150_CPX8_Q02_T2501
  quality_delta_top1=-0.007200, quality_amortized_margin=0.044s
  systems_amortized_margin=0.044s, systems_search_ratio=0.193, systems_cand_frac=0.193338

## Bank Summaries
- train=2500: first_any=1, first_quality=1, first_systems=1, systems_family=H4XH4_FIELD_A150_CPX8
  systems_ratio_min=0.193, systems_ratio_max=0.193, systems_ratio_span=0.000, systems_margin_slope=-0.043633
- train=2501: first_any=1, first_quality=1, first_systems=1, systems_family=H4XH4_FIELD_A150_CPX8
  systems_ratio_min=0.193, systems_ratio_max=0.193, systems_ratio_span=0.000, systems_margin_slope=0.029672

## Route Profiles
- DENSE_Q01_T2500: train=2500, q=1, top1=0.052000, cand_count=2500.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.290, online_per_repeat=0.123s, amortized=0.123s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q01_T2500: train=2500, q=1, top1=0.044600, cand_count=483.321, cand_frac=0.193328, work_ratio_vs_dense=0.193, offline_share=0.013, online_share=0.136, online_per_repeat=0.054s, amortized=0.059s, amortized_margin_vs_dense=0.064s
- DENSE_Q02_T2500: train=2500, q=2, top1=0.052000, cand_count=2500.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.464, online_per_repeat=0.125s, amortized=0.125s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q02_T2500: train=2500, q=2, top1=0.044600, cand_count=483.321, cand_frac=0.193328, work_ratio_vs_dense=0.193, offline_share=0.009, online_share=0.346, online_per_repeat=0.102s, amortized=0.104s, amortized_margin_vs_dense=0.020s
- DENSE_Q01_T2501: train=2501, q=1, top1=0.051800, cand_count=2501.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.294, online_per_repeat=0.120s, amortized=0.120s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q01_T2501: train=2501, q=1, top1=0.044600, cand_count=483.539, cand_frac=0.193338, work_ratio_vs_dense=0.193, offline_share=0.011, online_share=0.214, online_per_repeat=0.101s, amortized=0.106s, amortized_margin_vs_dense=0.014s
- DENSE_Q02_T2501: train=2501, q=2, top1=0.051800, cand_count=2501.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.415, online_per_repeat=0.105s, amortized=0.105s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q02_T2501: train=2501, q=2, top1=0.044600, cand_count=483.539, cand_frac=0.193338, work_ratio_vs_dense=0.193, offline_share=0.011, online_share=0.251, online_per_repeat=0.059s, amortized=0.061s, amortized_margin_vs_dense=0.044s

