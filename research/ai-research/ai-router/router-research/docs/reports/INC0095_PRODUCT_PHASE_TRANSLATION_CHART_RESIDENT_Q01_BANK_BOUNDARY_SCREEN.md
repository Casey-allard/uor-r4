# Translated Hardware Profile

## Source Experiments
- `inc0095_product_phase_translation_chart_resident_q01_bank_boundary_screen`

## Group Summaries
- train=2500, q=1: dense=DENSE_Q01_T2500, crossover=True, quality=CHART_H4XH4_FIELD_A150_CPX8_Q01_T2500, systems=CHART_H4XH4_FIELD_A150_CPX8_Q01_T2500
  quality_delta_top1=-0.007200, quality_amortized_margin=0.037s
  systems_amortized_margin=0.037s, systems_search_ratio=0.189, systems_cand_frac=0.189016
- train=2750, q=1: dense=DENSE_Q01_T2750, crossover=False, quality=n/a, systems=n/a
- train=3000, q=1: dense=DENSE_Q01_T3000, crossover=True, quality=CHART_H4XH4_FIELD_A150_CPX8_Q01_T3000, systems=CHART_H4XH4_FIELD_A150_CPX8_Q01_T3000
  quality_delta_top1=-0.004000, quality_amortized_margin=0.004s
  systems_amortized_margin=0.004s, systems_search_ratio=0.185, systems_cand_frac=0.185411
- train=4000, q=1: dense=DENSE_Q01_T4000, crossover=True, quality=CHART_H4XH4_FIELD_A150_CPX8_Q01_T4000, systems=CHART_H4XH4_FIELD_A150_CPX8_Q01_T4000
  quality_delta_top1=-0.010750, quality_amortized_margin=0.076s
  systems_amortized_margin=0.076s, systems_search_ratio=0.189, systems_cand_frac=0.188669
- train=6000, q=1: dense=DENSE_Q01_T6000, crossover=True, quality=CHART_H4XH4_FIELD_A150_CPX8_Q01_T6000, systems=CHART_H4XH4_FIELD_A150_CPX8_Q01_T6000
  quality_delta_top1=-0.002000, quality_amortized_margin=0.155s
  systems_amortized_margin=0.155s, systems_search_ratio=0.190, systems_cand_frac=0.190455
- train=10000, q=1: dense=DENSE_Q01_T10000, crossover=True, quality=CHART_H4XH4_FIELD_A150_CPX8_Q01_T10000, systems=CHART_H4XH4_FIELD_A150_CPX8_Q01_T10000
  quality_delta_top1=-0.000200, quality_amortized_margin=0.611s
  systems_amortized_margin=0.611s, systems_search_ratio=0.190, systems_cand_frac=0.190432
- train=40000, q=1: dense=DENSE_Q01_T40000, crossover=True, quality=CHART_H4XH4_FIELD_A150_CPX8_Q01_T40000, systems=CHART_H4XH4_FIELD_A150_CPX8_Q01_T40000
  quality_delta_top1=-0.002050, quality_amortized_margin=6.520s
  systems_amortized_margin=6.520s, systems_search_ratio=0.187, systems_cand_frac=0.187484

## Bank Summaries
- train=2500: first_any=1, first_quality=1, first_systems=1, systems_family=CHART_H4XH4_FIELD_A150_CPX8
  systems_ratio_min=0.189, systems_ratio_max=0.189, systems_ratio_span=0.000, systems_margin_slope=n/a
- train=2750: first_any=n/a, first_quality=n/a, first_systems=n/a, systems_family=n/a
- train=3000: first_any=1, first_quality=1, first_systems=1, systems_family=CHART_H4XH4_FIELD_A150_CPX8
  systems_ratio_min=0.185, systems_ratio_max=0.185, systems_ratio_span=0.000, systems_margin_slope=n/a
- train=4000: first_any=1, first_quality=1, first_systems=1, systems_family=CHART_H4XH4_FIELD_A150_CPX8
  systems_ratio_min=0.189, systems_ratio_max=0.189, systems_ratio_span=0.000, systems_margin_slope=n/a
- train=6000: first_any=1, first_quality=1, first_systems=1, systems_family=CHART_H4XH4_FIELD_A150_CPX8
  systems_ratio_min=0.190, systems_ratio_max=0.190, systems_ratio_span=0.000, systems_margin_slope=n/a
- train=10000: first_any=1, first_quality=1, first_systems=1, systems_family=CHART_H4XH4_FIELD_A150_CPX8
  systems_ratio_min=0.190, systems_ratio_max=0.190, systems_ratio_span=0.000, systems_margin_slope=n/a
- train=40000: first_any=1, first_quality=1, first_systems=1, systems_family=CHART_H4XH4_FIELD_A150_CPX8
  systems_ratio_min=0.187, systems_ratio_max=0.187, systems_ratio_span=0.000, systems_margin_slope=n/a

## Route Profiles
- CHART_H4XH4_FIELD_A150_CPX8_Q01_T2500: train=2500, q=1, top1=0.044400, cand_count=472.540, cand_frac=0.189016, work_ratio_vs_dense=0.189, offline_share=0.058, online_share=0.134, online_per_repeat=0.059s, amortized=0.084s, amortized_margin_vs_dense=0.037s
- DENSE_Q01_T2500: train=2500, q=1, top1=0.051600, cand_count=2500.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.289, online_per_repeat=0.121s, amortized=0.121s, amortized_margin_vs_dense=0.000s
- CHART_H4XH4_FIELD_A150_CPX8_Q01_T2750: train=2750, q=1, top1=0.043636, cand_count=516.444, cand_frac=0.187798, work_ratio_vs_dense=0.188, offline_share=0.065, online_share=0.282, online_per_repeat=0.168s, amortized=0.206s, amortized_margin_vs_dense=-0.061s
- DENSE_Q01_T2750: train=2750, q=1, top1=0.051636, cand_count=2750.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.339, online_per_repeat=0.146s, amortized=0.146s, amortized_margin_vs_dense=0.000s
- CHART_H4XH4_FIELD_A150_CPX8_Q01_T3000: train=3000, q=1, top1=0.044000, cand_count=556.234, cand_frac=0.185411, work_ratio_vs_dense=0.185, offline_share=0.051, online_share=0.231, online_per_repeat=0.126s, amortized=0.154s, amortized_margin_vs_dense=0.004s
- DENSE_Q01_T3000: train=3000, q=1, top1=0.048000, cand_count=3000.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.355, online_per_repeat=0.157s, amortized=0.157s, amortized_margin_vs_dense=0.000s
- CHART_H4XH4_FIELD_A150_CPX8_Q01_T4000: train=4000, q=1, top1=0.042750, cand_count=754.678, cand_frac=0.188669, work_ratio_vs_dense=0.189, offline_share=0.058, online_share=0.200, online_per_repeat=0.107s, amortized=0.138s, amortized_margin_vs_dense=0.076s
- DENSE_Q01_T4000: train=4000, q=1, top1=0.053500, cand_count=4000.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.425, online_per_repeat=0.214s, amortized=0.214s, amortized_margin_vs_dense=0.000s
- CHART_H4XH4_FIELD_A150_CPX8_Q01_T6000: train=6000, q=1, top1=0.047667, cand_count=1142.733, cand_frac=0.190455, work_ratio_vs_dense=0.190, offline_share=0.063, online_share=0.269, online_per_repeat=0.174s, amortized=0.215s, amortized_margin_vs_dense=0.155s
- DENSE_Q01_T6000: train=6000, q=1, top1=0.049667, cand_count=6000.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.535, online_per_repeat=0.370s, amortized=0.370s, amortized_margin_vs_dense=0.000s
- CHART_H4XH4_FIELD_A150_CPX8_Q01_T10000: train=10000, q=1, top1=0.048800, cand_count=1904.323, cand_frac=0.190432, work_ratio_vs_dense=0.190, offline_share=0.052, online_share=0.372, online_per_repeat=0.343s, amortized=0.392s, amortized_margin_vs_dense=0.611s
- DENSE_Q01_T10000: train=10000, q=1, top1=0.049000, cand_count=10000.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.758, online_per_repeat=1.003s, amortized=1.003s, amortized_margin_vs_dense=0.000s
- CHART_H4XH4_FIELD_A150_CPX8_Q01_T40000: train=40000, q=1, top1=0.047825, cand_count=7499.373, cand_frac=0.187484, work_ratio_vs_dense=0.187, offline_share=0.065, online_share=0.617, online_per_repeat=2.274s, amortized=2.514s, amortized_margin_vs_dense=6.520s
- DENSE_Q01_T40000: train=40000, q=1, top1=0.049875, cand_count=40000.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.959, online_per_repeat=9.034s, amortized=9.034s, amortized_margin_vs_dense=0.000s

