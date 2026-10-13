# Translated Hardware Profile

## Source Experiments
- `inc0094_product_phase_translation_chart_resident_route_ephemeral_repeat_map_confirm`

## Group Summaries
- train=2500, q=1: dense=DENSE_Q01_T2500, crossover=False, quality=n/a, systems=n/a
- train=2500, q=2: dense=DENSE_Q02_T2500, crossover=True, quality=CHART_H4XH4_FIELD_A150_CPX8_Q02_T2500, systems=CHART_H4XH4_FIELD_A150_CPX8_Q02_T2500
  quality_delta_top1=-0.007400, quality_amortized_margin=0.009s
  systems_amortized_margin=0.009s, systems_search_ratio=0.193, systems_cand_frac=0.193328
- train=2500, q=4: dense=DENSE_Q04_T2500, crossover=True, quality=CHART_H4XH4_FIELD_A150_CPX8_Q04_T2500, systems=CHART_H4XH4_FIELD_A150_CPX8_Q04_T2500
  quality_delta_top1=-0.007400, quality_amortized_margin=0.035s
  systems_amortized_margin=0.035s, systems_search_ratio=0.193, systems_cand_frac=0.193328
- train=40000, q=1: dense=DENSE_Q01_T40000, crossover=True, quality=CHART_H4XH4_FIELD_A150_CPX8_Q01_T40000, systems=CHART_H4XH4_FIELD_A150_CPX8_Q01_T40000
  quality_delta_top1=-0.001525, quality_amortized_margin=6.759s
  systems_amortized_margin=6.759s, systems_search_ratio=0.184, systems_cand_frac=0.183764
- train=40000, q=2: dense=DENSE_Q02_T40000, crossover=True, quality=CHART_H4XH4_FIELD_A150_CPX8_Q02_T40000, systems=CHART_H4XH4_FIELD_A150_CPX8_Q02_T40000
  quality_delta_top1=-0.001525, quality_amortized_margin=7.192s
  systems_amortized_margin=7.192s, systems_search_ratio=0.184, systems_cand_frac=0.183764
- train=40000, q=4: dense=DENSE_Q04_T40000, crossover=True, quality=CHART_H4XH4_FIELD_A150_CPX8_Q04_T40000, systems=CHART_H4XH4_FIELD_A150_CPX8_Q04_T40000
  quality_delta_top1=-0.001525, quality_amortized_margin=7.067s
  systems_amortized_margin=7.067s, systems_search_ratio=0.184, systems_cand_frac=0.183764

## Bank Summaries
- train=2500: first_any=2, first_quality=2, first_systems=2, systems_family=CHART_H4XH4_FIELD_A150_CPX8
  systems_ratio_min=0.193, systems_ratio_max=0.193, systems_ratio_span=0.000, systems_margin_slope=0.018406
- train=40000: first_any=1, first_quality=1, first_systems=1, systems_family=CHART_H4XH4_FIELD_A150_CPX8
  systems_ratio_min=0.184, systems_ratio_max=0.184, systems_ratio_span=0.000, systems_margin_slope=0.102492

## Route Profiles
- CHART_H4XH4_FIELD_A150_CPX8_Q01_T2500: train=2500, q=1, top1=0.044600, cand_count=483.321, cand_frac=0.193328, work_ratio_vs_dense=0.193, offline_share=0.088, online_share=0.157, online_per_repeat=0.079s, amortized=0.124s, amortized_margin_vs_dense=-0.020s
- DENSE_Q01_T2500: train=2500, q=1, top1=0.052000, cand_count=2500.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.259, online_per_repeat=0.104s, amortized=0.104s, amortized_margin_vs_dense=0.000s
- CHART_H4XH4_FIELD_A150_CPX8_Q02_T2500: train=2500, q=2, top1=0.044600, cand_count=483.321, cand_frac=0.193328, work_ratio_vs_dense=0.193, offline_share=0.047, online_share=0.294, online_per_repeat=0.083s, amortized=0.096s, amortized_margin_vs_dense=0.009s
- DENSE_Q02_T2500: train=2500, q=2, top1=0.052000, cand_count=2500.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.422, online_per_repeat=0.105s, amortized=0.105s, amortized_margin_vs_dense=0.000s
- CHART_H4XH4_FIELD_A150_CPX8_Q04_T2500: train=2500, q=4, top1=0.044600, cand_count=483.321, cand_frac=0.193328, work_ratio_vs_dense=0.193, offline_share=0.047, online_share=0.331, online_per_repeat=0.048s, amortized=0.054s, amortized_margin_vs_dense=0.035s
- DENSE_Q04_T2500: train=2500, q=4, top1=0.052000, cand_count=2500.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.549, online_per_repeat=0.090s, amortized=0.090s, amortized_margin_vs_dense=0.000s
- CHART_H4XH4_FIELD_A150_CPX8_Q01_T40000: train=40000, q=1, top1=0.047325, cand_count=7350.555, cand_frac=0.183764, work_ratio_vs_dense=0.184, offline_share=0.067, online_share=0.607, online_per_repeat=2.247s, amortized=2.495s, amortized_margin_vs_dense=6.759s
- DENSE_Q01_T40000: train=40000, q=1, top1=0.048850, cand_count=40000.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.959, online_per_repeat=9.255s, amortized=9.255s, amortized_margin_vs_dense=0.000s
- CHART_H4XH4_FIELD_A150_CPX8_Q02_T40000: train=40000, q=2, top1=0.047325, cand_count=7350.555, cand_frac=0.183764, work_ratio_vs_dense=0.184, offline_share=0.045, online_share=0.732, online_per_repeat=1.988s, amortized=2.111s, amortized_margin_vs_dense=7.192s
- DENSE_Q02_T40000: train=40000, q=2, top1=0.048850, cand_count=40000.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.980, online_per_repeat=9.303s, amortized=9.303s, amortized_margin_vs_dense=0.000s
- CHART_H4XH4_FIELD_A150_CPX8_Q04_T40000: train=40000, q=4, top1=0.047325, cand_count=7350.555, cand_frac=0.183764, work_ratio_vs_dense=0.184, offline_share=0.028, online_share=0.837, online_per_repeat=1.892s, amortized=1.955s, amortized_margin_vs_dense=7.067s
- DENSE_Q04_T40000: train=40000, q=4, top1=0.048850, cand_count=40000.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.990, online_per_repeat=9.022s, amortized=9.022s, amortized_margin_vs_dense=0.000s

