# Translated Hardware Profile

## Source Experiments
- `inc0089_product_phase_translation_warm_cache_q01_2500_2600_refine_confirm`

## Group Summaries
- train=2525, q=1: dense=DENSE_Q01_T2525, crossover=True, quality=H4XH4_FIELD_A150_CPX8_Q01_T2525, systems=H4XH4_FIELD_A150_CPX8_Q01_T2525
  quality_delta_top1=-0.006735, quality_amortized_margin=0.114s
  systems_amortized_margin=0.114s, systems_search_ratio=0.193, systems_cand_frac=0.193195
- train=2525, q=2: dense=DENSE_Q02_T2525, crossover=True, quality=H4XH4_FIELD_A150_CPX8_Q02_T2525, systems=H4XH4_FIELD_A150_CPX8_Q02_T2525
  quality_delta_top1=-0.006735, quality_amortized_margin=0.069s
  systems_amortized_margin=0.069s, systems_search_ratio=0.193, systems_cand_frac=0.193195
- train=2550, q=1: dense=DENSE_Q01_T2550, crossover=True, quality=H4XH4_FIELD_A150_CPX8_Q01_T2550, systems=H4XH4_FIELD_A150_CPX8_Q01_T2550
  quality_delta_top1=-0.006471, quality_amortized_margin=0.066s
  systems_amortized_margin=0.066s, systems_search_ratio=0.193, systems_cand_frac=0.192997
- train=2550, q=2: dense=DENSE_Q02_T2550, crossover=True, quality=H4XH4_FIELD_A150_CPX8_Q02_T2550, systems=H4XH4_FIELD_A150_CPX8_Q02_T2550
  quality_delta_top1=-0.006471, quality_amortized_margin=0.083s
  systems_amortized_margin=0.083s, systems_search_ratio=0.193, systems_cand_frac=0.192997
- train=2575, q=1: dense=DENSE_Q01_T2575, crossover=True, quality=H4XH4_FIELD_A150_CPX8_Q01_T2575, systems=H4XH4_FIELD_A150_CPX8_Q01_T2575
  quality_delta_top1=-0.006605, quality_amortized_margin=0.076s
  systems_amortized_margin=0.076s, systems_search_ratio=0.193, systems_cand_frac=0.192759
- train=2575, q=2: dense=DENSE_Q02_T2575, crossover=True, quality=H4XH4_FIELD_A150_CPX8_Q02_T2575, systems=H4XH4_FIELD_A150_CPX8_Q02_T2575
  quality_delta_top1=-0.006605, quality_amortized_margin=0.052s
  systems_amortized_margin=0.052s, systems_search_ratio=0.193, systems_cand_frac=0.192759

## Bank Summaries
- train=2525: first_any=1, first_quality=1, first_systems=1, systems_family=H4XH4_FIELD_A150_CPX8
  systems_ratio_min=0.193, systems_ratio_max=0.193, systems_ratio_span=0.000, systems_margin_slope=-0.045003
- train=2550: first_any=1, first_quality=1, first_systems=1, systems_family=H4XH4_FIELD_A150_CPX8
  systems_ratio_min=0.193, systems_ratio_max=0.193, systems_ratio_span=0.000, systems_margin_slope=0.016298
- train=2575: first_any=1, first_quality=1, first_systems=1, systems_family=H4XH4_FIELD_A150_CPX8
  systems_ratio_min=0.193, systems_ratio_max=0.193, systems_ratio_span=0.000, systems_margin_slope=-0.024540

## Route Profiles
- DENSE_Q01_T2525: train=2525, q=1, top1=0.051902, cand_count=2525.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.379, online_per_repeat=0.179s, amortized=0.179s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q01_T2525: train=2525, q=1, top1=0.045166, cand_count=487.818, cand_frac=0.193195, work_ratio_vs_dense=0.193, offline_share=0.013, online_share=0.145, online_per_repeat=0.059s, amortized=0.064s, amortized_margin_vs_dense=0.114s
- DENSE_Q02_T2525: train=2525, q=2, top1=0.051902, cand_count=2525.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.466, online_per_repeat=0.125s, amortized=0.125s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q02_T2525: train=2525, q=2, top1=0.045166, cand_count=487.818, cand_frac=0.193195, work_ratio_vs_dense=0.193, offline_share=0.011, online_share=0.237, online_per_repeat=0.053s, amortized=0.055s, amortized_margin_vs_dense=0.069s
- DENSE_Q01_T2550: train=2550, q=1, top1=0.051373, cand_count=2550.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.295, online_per_repeat=0.122s, amortized=0.122s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q01_T2550: train=2550, q=1, top1=0.044902, cand_count=492.143, cand_frac=0.192997, work_ratio_vs_dense=0.193, offline_share=0.013, online_share=0.128, online_per_repeat=0.051s, amortized=0.056s, amortized_margin_vs_dense=0.066s
- DENSE_Q02_T2550: train=2550, q=2, top1=0.051373, cand_count=2550.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.477, online_per_repeat=0.137s, amortized=0.137s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q02_T2550: train=2550, q=2, top1=0.044902, cand_count=492.143, cand_frac=0.192997, work_ratio_vs_dense=0.193, offline_share=0.011, online_share=0.229, online_per_repeat=0.052s, amortized=0.054s, amortized_margin_vs_dense=0.083s
- DENSE_Q01_T2575: train=2575, q=1, top1=0.051476, cand_count=2575.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.300, online_per_repeat=0.130s, amortized=0.130s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q01_T2575: train=2575, q=1, top1=0.044872, cand_count=496.354, cand_frac=0.192759, work_ratio_vs_dense=0.193, offline_share=0.013, online_share=0.121, online_per_repeat=0.049s, amortized=0.054s, amortized_margin_vs_dense=0.076s
- DENSE_Q02_T2575: train=2575, q=2, top1=0.051476, cand_count=2575.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.441, online_per_repeat=0.115s, amortized=0.115s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q02_T2575: train=2575, q=2, top1=0.044872, cand_count=496.354, cand_frac=0.192759, work_ratio_vs_dense=0.193, offline_share=0.011, online_share=0.253, online_per_repeat=0.061s, amortized=0.063s, amortized_margin_vs_dense=0.052s

