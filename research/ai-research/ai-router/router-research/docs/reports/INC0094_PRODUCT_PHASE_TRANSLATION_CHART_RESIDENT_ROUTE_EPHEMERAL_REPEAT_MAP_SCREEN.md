# Translated Hardware Profile

## Source Experiments
- `inc0094_product_phase_translation_chart_resident_route_ephemeral_repeat_map_screen`

## Group Summaries
- train=2500, q=1: dense=DENSE_Q01_T2500, crossover=False, quality=n/a, systems=n/a
- train=2500, q=2: dense=DENSE_Q02_T2500, crossover=True, quality=CHART_H4XH4_FIELD_A150_CPX8_Q02_T2500, systems=CHART_H4XH4_FIELD_A150_CPX8_Q02_T2500
  quality_delta_top1=-0.007200, quality_amortized_margin=0.061s
  systems_amortized_margin=0.061s, systems_search_ratio=0.189, systems_cand_frac=0.189016
- train=2500, q=4: dense=DENSE_Q04_T2500, crossover=False, quality=n/a, systems=n/a
- train=40000, q=1: dense=DENSE_Q01_T40000, crossover=True, quality=CHART_H4XH4_FIELD_A150_CPX8_Q01_T40000, systems=CHART_H4XH4_FIELD_A150_CPX8_Q01_T40000
  quality_delta_top1=-0.002050, quality_amortized_margin=6.267s
  systems_amortized_margin=6.267s, systems_search_ratio=0.187, systems_cand_frac=0.187484
- train=40000, q=2: dense=DENSE_Q02_T40000, crossover=True, quality=CHART_H4XH4_FIELD_A150_CPX8_Q02_T40000, systems=CHART_H4XH4_FIELD_A150_CPX8_Q02_T40000
  quality_delta_top1=-0.002050, quality_amortized_margin=6.843s
  systems_amortized_margin=6.843s, systems_search_ratio=0.187, systems_cand_frac=0.187484
- train=40000, q=4: dense=DENSE_Q04_T40000, crossover=True, quality=CHART_H4XH4_FIELD_A150_CPX8_Q04_T40000, systems=CHART_H4XH4_FIELD_A150_CPX8_Q04_T40000
  quality_delta_top1=-0.002050, quality_amortized_margin=7.218s
  systems_amortized_margin=7.218s, systems_search_ratio=0.187, systems_cand_frac=0.187484

## Bank Summaries
- train=2500: first_any=2, first_quality=2, first_systems=2, systems_family=CHART_H4XH4_FIELD_A150_CPX8
  systems_ratio_min=0.189, systems_ratio_max=0.189, systems_ratio_span=0.000, systems_margin_slope=0.000311
- train=40000: first_any=1, first_quality=1, first_systems=1, systems_family=CHART_H4XH4_FIELD_A150_CPX8
  systems_ratio_min=0.187, systems_ratio_max=0.187, systems_ratio_span=0.000, systems_margin_slope=0.317140

## Route Profiles
- CHART_H4XH4_FIELD_A150_CPX8_Q01_T2500: train=2500, q=1, top1=0.044400, cand_count=472.540, cand_frac=0.189016, work_ratio_vs_dense=0.189, offline_share=0.066, online_share=0.221, online_per_repeat=0.110s, amortized=0.143s, amortized_margin_vs_dense=-0.032s
- DENSE_Q01_T2500: train=2500, q=1, top1=0.051600, cand_count=2500.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.268, online_per_repeat=0.111s, amortized=0.111s, amortized_margin_vs_dense=0.000s
- CHART_H4XH4_FIELD_A150_CPX8_Q02_T2500: train=2500, q=2, top1=0.044400, cand_count=472.540, cand_frac=0.189016, work_ratio_vs_dense=0.189, offline_share=0.038, online_share=0.206, online_per_repeat=0.050s, amortized=0.059s, amortized_margin_vs_dense=0.061s
- DENSE_Q02_T2500: train=2500, q=2, top1=0.051600, cand_count=2500.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.462, online_per_repeat=0.119s, amortized=0.119s, amortized_margin_vs_dense=0.000s
- CHART_H4XH4_FIELD_A150_CPX8_Q04_T2500: train=2500, q=4, top1=0.044400, cand_count=472.540, cand_frac=0.189016, work_ratio_vs_dense=0.189, offline_share=0.044, online_share=0.498, online_per_repeat=0.114s, amortized=0.125s, amortized_margin_vs_dense=-0.031s
- DENSE_Q04_T2500: train=2500, q=4, top1=0.051600, cand_count=2500.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.561, online_per_repeat=0.094s, amortized=0.094s, amortized_margin_vs_dense=0.000s
- CHART_H4XH4_FIELD_A150_CPX8_Q01_T40000: train=40000, q=1, top1=0.047825, cand_count=7499.373, cand_frac=0.187484, work_ratio_vs_dense=0.187, offline_share=0.069, online_share=0.616, online_per_repeat=2.445s, amortized=2.718s, amortized_margin_vs_dense=6.267s
- DENSE_Q01_T40000: train=40000, q=1, top1=0.049875, cand_count=40000.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.958, online_per_repeat=8.985s, amortized=8.985s, amortized_margin_vs_dense=0.000s
- CHART_H4XH4_FIELD_A150_CPX8_Q02_T40000: train=40000, q=2, top1=0.047825, cand_count=7499.373, cand_frac=0.187484, work_ratio_vs_dense=0.187, offline_share=0.041, online_share=0.739, online_per_repeat=1.998s, amortized=2.109s, amortized_margin_vs_dense=6.843s
- DENSE_Q02_T40000: train=40000, q=2, top1=0.049875, cand_count=40000.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.979, online_per_repeat=8.952s, amortized=8.952s, amortized_margin_vs_dense=0.000s
- CHART_H4XH4_FIELD_A150_CPX8_Q04_T40000: train=40000, q=4, top1=0.047825, cand_count=7499.373, cand_frac=0.187484, work_ratio_vs_dense=0.187, offline_share=0.029, online_share=0.841, online_per_repeat=1.891s, amortized=1.956s, amortized_margin_vs_dense=7.218s
- DENSE_Q04_T40000: train=40000, q=4, top1=0.049875, cand_count=40000.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.990, online_per_repeat=9.174s, amortized=9.174s, amortized_margin_vs_dense=0.000s

