# Translated Hardware Profile

## Source Experiments
- `inc0086_product_phase_translation_warm_cache_q01_lower_boundary_refine_confirm`

## Group Summaries
- train=2500, q=1: dense=DENSE_Q01_T2500, crossover=False, quality=n/a, systems=n/a
- train=2500, q=2: dense=DENSE_Q02_T2500, crossover=True, quality=H4XH4_FIELD_A150_CPX8_Q02_T2500, systems=H4XH4_FIELD_A150_CPX8_Q02_T2500
  quality_delta_top1=-0.007400, quality_amortized_margin=0.017s
  systems_amortized_margin=0.017s, systems_search_ratio=0.193, systems_cand_frac=0.193328
- train=2750, q=1: dense=DENSE_Q01_T2750, crossover=True, quality=H4XH4_FIELD_A150_CPX8_Q01_T2750, systems=H4XH4_FIELD_A150_CPX8_Q01_T2750
  quality_delta_top1=-0.005636, quality_amortized_margin=0.009s
  systems_amortized_margin=0.009s, systems_search_ratio=0.193, systems_cand_frac=0.192894
- train=2750, q=2: dense=DENSE_Q02_T2750, crossover=True, quality=H4XH4_FIELD_A150_CPX8_Q02_T2750, systems=H4XH4_FIELD_A150_CPX8_Q02_T2750
  quality_delta_top1=-0.005636, quality_amortized_margin=0.037s
  systems_amortized_margin=0.037s, systems_search_ratio=0.193, systems_cand_frac=0.192894
- train=3000, q=1: dense=DENSE_Q01_T3000, crossover=True, quality=H4XH4_FIELD_A150_CPX8_Q01_T3000, systems=H4XH4_FIELD_A150_CPX8_Q01_T3000
  quality_delta_top1=-0.005000, quality_amortized_margin=0.059s
  systems_amortized_margin=0.059s, systems_search_ratio=0.192, systems_cand_frac=0.191704
- train=3000, q=2: dense=DENSE_Q02_T3000, crossover=True, quality=H4XH4_FIELD_A150_CPX8_Q02_T3000, systems=H4XH4_FIELD_A150_CPX8_Q02_T3000
  quality_delta_top1=-0.005000, quality_amortized_margin=0.089s
  systems_amortized_margin=0.089s, systems_search_ratio=0.192, systems_cand_frac=0.191704

## Bank Summaries
- train=2500: first_any=2, first_quality=2, first_systems=2, systems_family=H4XH4_FIELD_A150_CPX8
  systems_ratio_min=0.193, systems_ratio_max=0.193, systems_ratio_span=0.000, systems_margin_slope=0.047175
- train=2750: first_any=1, first_quality=1, first_systems=1, systems_family=H4XH4_FIELD_A150_CPX8
  systems_ratio_min=0.193, systems_ratio_max=0.193, systems_ratio_span=0.000, systems_margin_slope=0.028268
- train=3000: first_any=1, first_quality=1, first_systems=1, systems_family=H4XH4_FIELD_A150_CPX8
  systems_ratio_min=0.192, systems_ratio_max=0.192, systems_ratio_span=0.000, systems_margin_slope=0.030180

## Route Profiles
- DENSE_Q01_T2500: train=2500, q=1, top1=0.052000, cand_count=2500.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.273, online_per_repeat=0.110s, amortized=0.110s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q01_T2500: train=2500, q=1, top1=0.044600, cand_count=483.321, cand_frac=0.193328, work_ratio_vs_dense=0.193, offline_share=0.010, online_share=0.259, online_per_repeat=0.135s, amortized=0.140s, amortized_margin_vs_dense=-0.030s
- DENSE_Q02_T2500: train=2500, q=2, top1=0.052000, cand_count=2500.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.427, online_per_repeat=0.108s, amortized=0.108s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q02_T2500: train=2500, q=2, top1=0.044600, cand_count=483.321, cand_frac=0.193328, work_ratio_vs_dense=0.193, offline_share=0.009, online_share=0.321, online_per_repeat=0.089s, amortized=0.091s, amortized_margin_vs_dense=0.017s
- DENSE_Q01_T2750: train=2750, q=1, top1=0.050182, cand_count=2750.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.270, online_per_repeat=0.108s, amortized=0.108s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q01_T2750: train=2750, q=1, top1=0.044545, cand_count=530.458, cand_frac=0.192894, work_ratio_vs_dense=0.193, offline_share=0.012, online_share=0.201, online_per_repeat=0.094s, amortized=0.100s, amortized_margin_vs_dense=0.009s
- DENSE_Q02_T2750: train=2750, q=2, top1=0.050182, cand_count=2750.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.480, online_per_repeat=0.131s, amortized=0.131s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q02_T2750: train=2750, q=2, top1=0.044545, cand_count=530.458, cand_frac=0.192894, work_ratio_vs_dense=0.193, offline_share=0.010, online_share=0.316, online_per_repeat=0.091s, amortized=0.094s, amortized_margin_vs_dense=0.037s
- DENSE_Q01_T3000: train=3000, q=1, top1=0.049833, cand_count=3000.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.309, online_per_repeat=0.132s, amortized=0.132s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q01_T3000: train=3000, q=1, top1=0.044833, cand_count=575.112, cand_frac=0.191704, work_ratio_vs_dense=0.192, offline_share=0.014, online_share=0.162, online_per_repeat=0.067s, amortized=0.073s, amortized_margin_vs_dense=0.059s
- DENSE_Q02_T3000: train=3000, q=2, top1=0.049833, cand_count=3000.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.508, online_per_repeat=0.150s, amortized=0.150s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q02_T3000: train=3000, q=2, top1=0.044833, cand_count=575.112, cand_frac=0.191704, work_ratio_vs_dense=0.192, offline_share=0.012, online_share=0.242, online_per_repeat=0.057s, amortized=0.060s, amortized_margin_vs_dense=0.089s

