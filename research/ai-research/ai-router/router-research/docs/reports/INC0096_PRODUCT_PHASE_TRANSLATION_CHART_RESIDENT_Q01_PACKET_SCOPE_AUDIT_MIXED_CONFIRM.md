# Translated Hardware Profile

## Source Experiments
- `inc0096_product_phase_translation_chart_resident_q01_packet_scope_audit_mixed_confirm`

## Group Summaries
- train=2500, q=1: dense=DENSE_Q01_T2500, crossover=True, quality=CHART_H4XH4_FIELD_A150_CPX8_Q01_T2500, systems=CHART_H4XH4_FIELD_A150_CPX8_Q01_T2500
  quality_delta_top1=-0.004000, quality_amortized_margin=0.008s
  systems_amortized_margin=0.008s, systems_search_ratio=0.199, systems_cand_frac=0.198723
- train=2500, q=2: dense=DENSE_Q02_T2500, crossover=True, quality=CHART_H4XH4_FIELD_A150_CPX8_Q02_T2500, systems=CHART_H4XH4_FIELD_A150_CPX8_Q02_T2500
  quality_delta_top1=-0.004000, quality_amortized_margin=0.032s
  systems_amortized_margin=0.032s, systems_search_ratio=0.199, systems_cand_frac=0.198723
- train=2500, q=4: dense=DENSE_Q04_T2500, crossover=True, quality=CHART_H4XH4_FIELD_A150_CPX8_Q04_T2500, systems=CHART_H4XH4_FIELD_A150_CPX8_Q04_T2500
  quality_delta_top1=-0.004000, quality_amortized_margin=0.043s
  systems_amortized_margin=0.043s, systems_search_ratio=0.199, systems_cand_frac=0.198723

## Bank Summaries
- train=2500: first_any=1, first_quality=1, first_systems=1, systems_family=CHART_H4XH4_FIELD_A150_CPX8
  systems_ratio_min=0.199, systems_ratio_max=0.199, systems_ratio_span=0.000, systems_margin_slope=0.011716

## Route Profiles
- CHART_H4XH4_FIELD_A150_CPX8_Q01_T2500: train=2500, q=1, top1=0.046300, cand_count=496.808, cand_frac=0.198723, work_ratio_vs_dense=0.199, offline_share=0.043, online_share=0.152, online_per_repeat=0.068s, amortized=0.087s, amortized_margin_vs_dense=0.008s
- DENSE_Q01_T2500: train=2500, q=1, top1=0.050300, cand_count=2500.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.243, online_per_repeat=0.095s, amortized=0.095s, amortized_margin_vs_dense=0.000s
- CHART_H4XH4_FIELD_A150_CPX8_Q02_T2500: train=2500, q=2, top1=0.046300, cand_count=496.808, cand_frac=0.198723, work_ratio_vs_dense=0.199, offline_share=0.046, online_share=0.215, online_per_repeat=0.054s, amortized=0.066s, amortized_margin_vs_dense=0.032s
- DENSE_Q02_T2500: train=2500, q=2, top1=0.050300, cand_count=2500.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.396, online_per_repeat=0.098s, amortized=0.098s, amortized_margin_vs_dense=0.000s
- CHART_H4XH4_FIELD_A150_CPX8_Q04_T2500: train=2500, q=4, top1=0.046300, cand_count=496.808, cand_frac=0.198723, work_ratio_vs_dense=0.199, offline_share=0.034, online_share=0.360, online_per_repeat=0.053s, amortized=0.058s, amortized_margin_vs_dense=0.043s
- DENSE_Q04_T2500: train=2500, q=4, top1=0.050300, cand_count=2500.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.578, online_per_repeat=0.101s, amortized=0.101s, amortized_margin_vs_dense=0.000s

