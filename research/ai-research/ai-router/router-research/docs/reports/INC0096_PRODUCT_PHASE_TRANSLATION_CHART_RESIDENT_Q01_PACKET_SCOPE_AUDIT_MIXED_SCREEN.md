# Translated Hardware Profile

## Source Experiments
- `inc0096_product_phase_translation_chart_resident_q01_packet_scope_audit_mixed_screen`

## Group Summaries
- train=2500, q=1: dense=DENSE_Q01_T2500, crossover=True, quality=CHART_H4XH4_FIELD_A150_CPX8_Q01_T2500, systems=CHART_H4XH4_FIELD_A150_CPX8_Q01_T2500
  quality_delta_top1=-0.007200, quality_amortized_margin=0.057s
  systems_amortized_margin=0.057s, systems_search_ratio=0.189, systems_cand_frac=0.189016
- train=2500, q=2: dense=DENSE_Q02_T2500, crossover=True, quality=CHART_H4XH4_FIELD_A150_CPX8_Q02_T2500, systems=CHART_H4XH4_FIELD_A150_CPX8_Q02_T2500
  quality_delta_top1=-0.007200, quality_amortized_margin=0.063s
  systems_amortized_margin=0.063s, systems_search_ratio=0.189, systems_cand_frac=0.189016
- train=2500, q=4: dense=DENSE_Q04_T2500, crossover=True, quality=CHART_H4XH4_FIELD_A150_CPX8_Q04_T2500, systems=CHART_H4XH4_FIELD_A150_CPX8_Q04_T2500
  quality_delta_top1=-0.007200, quality_amortized_margin=0.057s
  systems_amortized_margin=0.057s, systems_search_ratio=0.189, systems_cand_frac=0.189016

## Bank Summaries
- train=2500: first_any=1, first_quality=1, first_systems=1, systems_family=CHART_H4XH4_FIELD_A150_CPX8
  systems_ratio_min=0.189, systems_ratio_max=0.189, systems_ratio_span=0.000, systems_margin_slope=-0.000058

## Route Profiles
- CHART_H4XH4_FIELD_A150_CPX8_Q01_T2500: train=2500, q=1, top1=0.044400, cand_count=472.540, cand_frac=0.189016, work_ratio_vs_dense=0.189, offline_share=0.037, online_share=0.117, online_per_repeat=0.049s, amortized=0.065s, amortized_margin_vs_dense=0.057s
- DENSE_Q01_T2500: train=2500, q=1, top1=0.051600, cand_count=2500.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.244, online_per_repeat=0.122s, amortized=0.122s, amortized_margin_vs_dense=0.000s
- CHART_H4XH4_FIELD_A150_CPX8_Q02_T2500: train=2500, q=2, top1=0.044400, cand_count=472.540, cand_frac=0.189016, work_ratio_vs_dense=0.189, offline_share=0.046, online_share=0.220, online_per_repeat=0.053s, amortized=0.065s, amortized_margin_vs_dense=0.063s
- DENSE_Q02_T2500: train=2500, q=2, top1=0.051600, cand_count=2500.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.394, online_per_repeat=0.127s, amortized=0.127s, amortized_margin_vs_dense=0.000s
- CHART_H4XH4_FIELD_A150_CPX8_Q04_T2500: train=2500, q=4, top1=0.044400, cand_count=472.540, cand_frac=0.189016, work_ratio_vs_dense=0.189, offline_share=0.042, online_share=0.327, online_per_repeat=0.045s, amortized=0.051s, amortized_margin_vs_dense=0.057s
- DENSE_Q04_T2500: train=2500, q=4, top1=0.051600, cand_count=2500.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.589, online_per_repeat=0.107s, amortized=0.107s, amortized_margin_vs_dense=0.000s

