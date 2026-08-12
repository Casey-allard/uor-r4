# Translated Hardware Profile

## Source Experiments
- `inc0092_product_phase_translation_warm_cache_q01_floor_hardening_confirm`

## Group Summaries
- train=2500, q=1: dense=DENSE_Q01_T2500, crossover=True, quality=H4XH4_FIELD_A150_CPX8_Q01_T2500, systems=H4XH4_FIELD_A150_CPX8_Q01_T2500
  quality_delta_top1=-0.004000, quality_amortized_margin=0.034s
  systems_amortized_margin=0.034s, systems_search_ratio=0.199, systems_cand_frac=0.198723
- train=2500, q=2: dense=DENSE_Q02_T2500, crossover=True, quality=H4XH4_FIELD_A150_CPX8_Q02_T2500, systems=H4XH4_FIELD_A150_CPX8_Q02_T2500
  quality_delta_top1=-0.004000, quality_amortized_margin=0.047s
  systems_amortized_margin=0.047s, systems_search_ratio=0.199, systems_cand_frac=0.198723
- train=2501, q=1: dense=DENSE_Q01_T2501, crossover=True, quality=H4XH4_FIELD_A150_CPX8_Q01_T2501, systems=H4XH4_FIELD_A150_CPX8_Q01_T2501
  quality_delta_top1=-0.004000, quality_amortized_margin=0.059s
  systems_amortized_margin=0.059s, systems_search_ratio=0.199, systems_cand_frac=0.198731
- train=2501, q=2: dense=DENSE_Q02_T2501, crossover=True, quality=H4XH4_FIELD_A150_CPX8_Q02_T2501, systems=H4XH4_FIELD_A150_CPX8_Q02_T2501
  quality_delta_top1=-0.004000, quality_amortized_margin=0.002s
  systems_amortized_margin=0.002s, systems_search_ratio=0.199, systems_cand_frac=0.198731

## Bank Summaries
- train=2500: first_any=1, first_quality=1, first_systems=1, systems_family=H4XH4_FIELD_A150_CPX8
  systems_ratio_min=0.199, systems_ratio_max=0.199, systems_ratio_span=0.000, systems_margin_slope=0.013457
- train=2501: first_any=1, first_quality=1, first_systems=1, systems_family=H4XH4_FIELD_A150_CPX8
  systems_ratio_min=0.199, systems_ratio_max=0.199, systems_ratio_span=0.000, systems_margin_slope=-0.057007

## Route Profiles
- DENSE_Q01_T2500: train=2500, q=1, top1=0.050300, cand_count=2500.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.267, online_per_repeat=0.108s, amortized=0.108s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q01_T2500: train=2500, q=1, top1=0.046300, cand_count=496.808, cand_frac=0.198723, work_ratio_vs_dense=0.199, offline_share=0.013, online_share=0.158, online_per_repeat=0.068s, amortized=0.074s, amortized_margin_vs_dense=0.034s
- DENSE_Q02_T2500: train=2500, q=2, top1=0.050300, cand_count=2500.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.431, online_per_repeat=0.111s, amortized=0.111s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q02_T2500: train=2500, q=2, top1=0.046300, cand_count=496.808, cand_frac=0.198723, work_ratio_vs_dense=0.199, offline_share=0.011, online_share=0.259, online_per_repeat=0.061s, amortized=0.064s, amortized_margin_vs_dense=0.047s
- DENSE_Q01_T2501: train=2501, q=1, top1=0.050300, cand_count=2501.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.291, online_per_repeat=0.122s, amortized=0.122s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q01_T2501: train=2501, q=1, top1=0.046300, cand_count=497.027, cand_frac=0.198731, work_ratio_vs_dense=0.199, offline_share=0.013, online_share=0.142, online_per_repeat=0.058s, amortized=0.064s, amortized_margin_vs_dense=0.059s
- DENSE_Q02_T2501: train=2501, q=2, top1=0.050300, cand_count=2501.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.395, online_per_repeat=0.095s, amortized=0.095s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q02_T2501: train=2501, q=2, top1=0.046300, cand_count=497.027, cand_frac=0.198731, work_ratio_vs_dense=0.199, offline_share=0.010, online_share=0.326, online_per_repeat=0.091s, amortized=0.094s, amortized_margin_vs_dense=0.002s

