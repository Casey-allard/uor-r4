# Translated Hardware Profile

## Source Experiments
- `inc0079_product_phase_translation_large_bank_boundary_extension_confirm`

## Group Summaries
- train=12000, q=8: dense=DENSE_Q08_T12000, crossover=False, quality=n/a, systems=n/a
- train=12000, q=12: dense=DENSE_Q12_T12000, crossover=True, quality=H4XH4_FIELD_A150_CPX8_Q12_T12000, systems=H4XH4_FIELD_A150_CPX8_Q12_T12000
  quality_delta_top1=-0.000458, quality_amortized_margin=0.085s
  systems_amortized_margin=0.085s, systems_search_ratio=0.190, systems_cand_frac=0.190318
- train=18000, q=8: dense=DENSE_Q08_T18000, crossover=True, quality=H4XH4_FIELD_A150_CPX8_Q08_T18000, systems=H4XH4_FIELD_A150_CPX8_Q08_T18000
  quality_delta_top1=-0.000972, quality_amortized_margin=0.419s
  systems_amortized_margin=0.419s, systems_search_ratio=0.190, systems_cand_frac=0.189969
- train=18000, q=12: dense=DENSE_Q12_T18000, crossover=True, quality=H4XH4_FIELD_A150_CPX8_Q12_T18000, systems=H4XH4_FIELD_A150_CPX8_Q12_T18000
  quality_delta_top1=-0.000972, quality_amortized_margin=1.316s
  systems_amortized_margin=1.316s, systems_search_ratio=0.190, systems_cand_frac=0.189969

## Bank Summaries
- train=12000: first_any=12, first_quality=12, first_systems=12, systems_family=H4XH4_FIELD_A150_CPX8
  systems_ratio_min=0.190, systems_ratio_max=0.190, systems_ratio_span=0.000, systems_margin_slope=0.099282
- train=18000: first_any=8, first_quality=8, first_systems=8, systems_family=H4XH4_FIELD_A150_CPX8
  systems_ratio_min=0.190, systems_ratio_max=0.190, systems_ratio_span=0.000, systems_margin_slope=0.224295

## Route Profiles
- DENSE_Q08_T12000: train=12000, q=8, top1=0.049125, cand_count=12000.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.970, online_per_repeat=1.318s, amortized=1.318s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q08_T12000: train=12000, q=8, top1=0.048667, cand_count=2283.812, cand_frac=0.190318, work_ratio_vs_dense=0.190, offline_share=0.731, online_share=0.226, online_per_repeat=0.385s, amortized=1.630s, amortized_margin_vs_dense=-0.313s
- DENSE_Q12_T12000: train=12000, q=12, top1=0.049125, cand_count=12000.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.979, online_per_repeat=1.307s, amortized=1.307s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q12_T12000: train=12000, q=12, top1=0.048667, cand_count=2283.812, cand_frac=0.190318, work_ratio_vs_dense=0.190, offline_share=0.659, online_share=0.302, online_per_repeat=0.384s, amortized=1.223s, amortized_margin_vs_dense=0.085s
- DENSE_Q08_T18000: train=18000, q=8, top1=0.048639, cand_count=18000.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.986, online_per_repeat=3.057s, amortized=3.057s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q08_T18000: train=18000, q=8, top1=0.047667, cand_count=3419.438, cand_frac=0.189969, work_ratio_vs_dense=0.190, offline_share=0.701, online_share=0.266, online_per_repeat=0.725s, amortized=2.638s, amortized_margin_vs_dense=0.419s
- DENSE_Q12_T18000: train=18000, q=12, top1=0.048639, cand_count=18000.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.991, online_per_repeat=3.281s, amortized=3.281s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q12_T18000: train=18000, q=12, top1=0.047667, cand_count=3419.438, cand_frac=0.189969, work_ratio_vs_dense=0.190, offline_share=0.615, online_share=0.356, online_per_repeat=0.720s, amortized=1.965s, amortized_margin_vs_dense=1.316s

