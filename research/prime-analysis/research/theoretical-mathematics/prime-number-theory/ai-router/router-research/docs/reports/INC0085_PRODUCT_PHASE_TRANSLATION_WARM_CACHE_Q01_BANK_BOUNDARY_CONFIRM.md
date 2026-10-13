# Translated Hardware Profile

## Source Experiments
- `inc0085_product_phase_translation_warm_cache_q01_bank_boundary_confirm`

## Group Summaries
- train=3000, q=1: dense=DENSE_Q01_T3000, crossover=True, quality=H4XH4_FIELD_A150_CPX8_Q01_T3000, systems=H4XH4_FIELD_A150_CPX8_Q01_T3000
  quality_delta_top1=-0.005000, quality_amortized_margin=0.085s
  systems_amortized_margin=0.085s, systems_search_ratio=0.192, systems_cand_frac=0.191704
- train=3000, q=2: dense=DENSE_Q02_T3000, crossover=True, quality=H4XH4_FIELD_A150_CPX8_Q02_T3000, systems=H4XH4_FIELD_A150_CPX8_Q02_T3000
  quality_delta_top1=-0.005000, quality_amortized_margin=0.082s
  systems_amortized_margin=0.082s, systems_search_ratio=0.192, systems_cand_frac=0.191704
- train=4500, q=1: dense=DENSE_Q01_T4500, crossover=True, quality=H4XH4_FIELD_A150_CPX8_Q01_T4500, systems=H4XH4_FIELD_A150_CPX8_Q01_T4500
  quality_delta_top1=0.001444, quality_amortized_margin=0.095s
  systems_amortized_margin=0.095s, systems_search_ratio=0.193, systems_cand_frac=0.193020
- train=4500, q=2: dense=DENSE_Q02_T4500, crossover=True, quality=H4XH4_FIELD_A150_CPX8_Q02_T4500, systems=H4XH4_FIELD_A150_CPX8_Q02_T4500
  quality_delta_top1=0.001444, quality_amortized_margin=0.116s
  systems_amortized_margin=0.116s, systems_search_ratio=0.193, systems_cand_frac=0.193020
- train=6000, q=1: dense=DENSE_Q01_T6000, crossover=True, quality=H4XH4_FIELD_A150_CPX8_Q01_T6000, systems=H4XH4_FIELD_A150_CPX8_Q01_T6000
  quality_delta_top1=-0.001583, quality_amortized_margin=0.191s
  systems_amortized_margin=0.191s, systems_search_ratio=0.187, systems_cand_frac=0.187229
- train=6000, q=2: dense=DENSE_Q02_T6000, crossover=True, quality=H4XH4_FIELD_A150_CPX8_Q02_T6000, systems=H4XH4_FIELD_A150_CPX8_Q02_T6000
  quality_delta_top1=-0.001583, quality_amortized_margin=0.226s
  systems_amortized_margin=0.226s, systems_search_ratio=0.187, systems_cand_frac=0.187229

## Bank Summaries
- train=3000: first_any=1, first_quality=1, first_systems=1, systems_family=H4XH4_FIELD_A150_CPX8
  systems_ratio_min=0.192, systems_ratio_max=0.192, systems_ratio_span=0.000, systems_margin_slope=-0.003134
- train=4500: first_any=1, first_quality=1, first_systems=1, systems_family=H4XH4_FIELD_A150_CPX8
  systems_ratio_min=0.193, systems_ratio_max=0.193, systems_ratio_span=0.000, systems_margin_slope=0.020755
- train=6000: first_any=1, first_quality=1, first_systems=1, systems_family=H4XH4_FIELD_A150_CPX8
  systems_ratio_min=0.187, systems_ratio_max=0.187, systems_ratio_span=0.000, systems_margin_slope=0.034874

## Route Profiles
- DENSE_Q01_T3000: train=3000, q=1, top1=0.049833, cand_count=3000.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.352, online_per_repeat=0.159s, amortized=0.159s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q01_T3000: train=3000, q=1, top1=0.044833, cand_count=575.112, cand_frac=0.191704, work_ratio_vs_dense=0.192, offline_share=0.014, online_share=0.165, online_per_repeat=0.069s, amortized=0.074s, amortized_margin_vs_dense=0.085s
- DENSE_Q02_T3000: train=3000, q=2, top1=0.049833, cand_count=3000.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.506, online_per_repeat=0.145s, amortized=0.145s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q02_T3000: train=3000, q=2, top1=0.044833, cand_count=575.112, cand_frac=0.191704, work_ratio_vs_dense=0.192, offline_share=0.012, online_share=0.253, online_per_repeat=0.060s, amortized=0.063s, amortized_margin_vs_dense=0.082s
- DENSE_Q01_T4500: train=4500, q=1, top1=0.045889, cand_count=4500.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.493, online_per_repeat=0.283s, amortized=0.283s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q01_T4500: train=4500, q=1, top1=0.047333, cand_count=868.588, cand_frac=0.193020, work_ratio_vs_dense=0.193, offline_share=0.013, online_share=0.301, online_per_repeat=0.180s, amortized=0.188s, amortized_margin_vs_dense=0.095s
- DENSE_Q02_T4500: train=4500, q=2, top1=0.045889, cand_count=4500.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.649, online_per_repeat=0.279s, amortized=0.279s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q02_T4500: train=4500, q=2, top1=0.047333, cand_count=868.588, cand_frac=0.193020, work_ratio_vs_dense=0.193, offline_share=0.011, online_share=0.423, online_per_repeat=0.159s, amortized=0.163s, amortized_margin_vs_dense=0.116s
- DENSE_Q01_T6000: train=6000, q=1, top1=0.048667, cand_count=6000.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.574, online_per_repeat=0.396s, amortized=0.396s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q01_T6000: train=6000, q=1, top1=0.047083, cand_count=1123.376, cand_frac=0.187229, work_ratio_vs_dense=0.187, offline_share=0.017, online_share=0.296, online_per_repeat=0.194s, amortized=0.205s, amortized_margin_vs_dense=0.191s
- DENSE_Q02_T6000: train=6000, q=2, top1=0.048667, cand_count=6000.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.724, online_per_repeat=0.406s, amortized=0.406s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q02_T6000: train=6000, q=2, top1=0.047083, cand_count=1123.376, cand_frac=0.187229, work_ratio_vs_dense=0.187, offline_share=0.013, online_share=0.438, online_per_repeat=0.175s, amortized=0.180s, amortized_margin_vs_dense=0.226s

