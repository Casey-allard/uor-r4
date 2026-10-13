# Translated Hardware Profile

## Source Experiments
- `inc0095_product_phase_translation_chart_resident_q01_bank_boundary_confirm`

## Group Summaries
- train=2500, q=1: dense=DENSE_Q01_T2500, crossover=True, quality=CHART_H4XH4_FIELD_A150_CPX8_Q01_T2500, systems=CHART_H4XH4_FIELD_A150_CPX8_Q01_T2500
  quality_delta_top1=-0.007400, quality_amortized_margin=0.039s
  systems_amortized_margin=0.039s, systems_search_ratio=0.193, systems_cand_frac=0.193328
- train=2750, q=1: dense=DENSE_Q01_T2750, crossover=True, quality=CHART_H4XH4_FIELD_A150_CPX8_Q01_T2750, systems=CHART_H4XH4_FIELD_A150_CPX8_Q01_T2750
  quality_delta_top1=-0.005636, quality_amortized_margin=0.036s
  systems_amortized_margin=0.036s, systems_search_ratio=0.193, systems_cand_frac=0.192894
- train=3000, q=1: dense=DENSE_Q01_T3000, crossover=True, quality=CHART_H4XH4_FIELD_A150_CPX8_Q01_T3000, systems=CHART_H4XH4_FIELD_A150_CPX8_Q01_T3000
  quality_delta_top1=-0.005000, quality_amortized_margin=0.024s
  systems_amortized_margin=0.024s, systems_search_ratio=0.192, systems_cand_frac=0.191704
- train=4000, q=1: dense=DENSE_Q01_T4000, crossover=True, quality=CHART_H4XH4_FIELD_A150_CPX8_Q01_T4000, systems=CHART_H4XH4_FIELD_A150_CPX8_Q01_T4000
  quality_delta_top1=-0.003875, quality_amortized_margin=0.009s
  systems_amortized_margin=0.009s, systems_search_ratio=0.193, systems_cand_frac=0.192636

## Bank Summaries
- train=2500: first_any=1, first_quality=1, first_systems=1, systems_family=CHART_H4XH4_FIELD_A150_CPX8
  systems_ratio_min=0.193, systems_ratio_max=0.193, systems_ratio_span=0.000, systems_margin_slope=n/a
- train=2750: first_any=1, first_quality=1, first_systems=1, systems_family=CHART_H4XH4_FIELD_A150_CPX8
  systems_ratio_min=0.193, systems_ratio_max=0.193, systems_ratio_span=0.000, systems_margin_slope=n/a
- train=3000: first_any=1, first_quality=1, first_systems=1, systems_family=CHART_H4XH4_FIELD_A150_CPX8
  systems_ratio_min=0.192, systems_ratio_max=0.192, systems_ratio_span=0.000, systems_margin_slope=n/a
- train=4000: first_any=1, first_quality=1, first_systems=1, systems_family=CHART_H4XH4_FIELD_A150_CPX8
  systems_ratio_min=0.193, systems_ratio_max=0.193, systems_ratio_span=0.000, systems_margin_slope=n/a

## Route Profiles
- CHART_H4XH4_FIELD_A150_CPX8_Q01_T2500: train=2500, q=1, top1=0.044600, cand_count=483.321, cand_frac=0.193328, work_ratio_vs_dense=0.193, offline_share=0.047, online_share=0.148, online_per_repeat=0.064s, amortized=0.085s, amortized_margin_vs_dense=0.039s
- DENSE_Q01_T2500: train=2500, q=1, top1=0.052000, cand_count=2500.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.300, online_per_repeat=0.124s, amortized=0.124s, amortized_margin_vs_dense=0.000s
- CHART_H4XH4_FIELD_A150_CPX8_Q01_T2750: train=2750, q=1, top1=0.044545, cand_count=530.458, cand_frac=0.192894, work_ratio_vs_dense=0.193, offline_share=0.054, online_share=0.152, online_per_repeat=0.067s, amortized=0.090s, amortized_margin_vs_dense=0.036s
- DENSE_Q01_T2750: train=2750, q=1, top1=0.050182, cand_count=2750.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.300, online_per_repeat=0.127s, amortized=0.127s, amortized_margin_vs_dense=0.000s
- CHART_H4XH4_FIELD_A150_CPX8_Q01_T3000: train=3000, q=1, top1=0.044833, cand_count=575.112, cand_frac=0.191704, work_ratio_vs_dense=0.192, offline_share=0.046, online_share=0.168, online_per_repeat=0.075s, amortized=0.095s, amortized_margin_vs_dense=0.024s
- DENSE_Q01_T3000: train=3000, q=1, top1=0.049833, cand_count=3000.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.287, online_per_repeat=0.118s, amortized=0.118s, amortized_margin_vs_dense=0.000s
- CHART_H4XH4_FIELD_A150_CPX8_Q01_T4000: train=4000, q=1, top1=0.045000, cand_count=770.545, cand_frac=0.192636, work_ratio_vs_dense=0.193, offline_share=0.049, online_share=0.277, online_per_repeat=0.167s, amortized=0.197s, amortized_margin_vs_dense=0.009s
- DENSE_Q01_T4000: train=4000, q=1, top1=0.048875, cand_count=4000.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.412, online_per_repeat=0.206s, amortized=0.206s, amortized_margin_vs_dense=0.000s

