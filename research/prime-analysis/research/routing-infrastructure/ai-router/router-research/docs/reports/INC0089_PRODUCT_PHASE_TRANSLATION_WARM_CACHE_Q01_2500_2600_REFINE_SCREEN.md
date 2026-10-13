# Translated Hardware Profile

## Source Experiments
- `inc0089_product_phase_translation_warm_cache_q01_2500_2600_refine_screen`

## Group Summaries
- train=2525, q=1: dense=DENSE_Q01_T2525, crossover=True, quality=H4XH4_FIELD_A150_CPX8_Q01_T2525, systems=H4XH4_FIELD_A150_CPX8_Q01_T2525
  quality_delta_top1=-0.005943, quality_amortized_margin=0.039s
  systems_amortized_margin=0.039s, systems_search_ratio=0.189, systems_cand_frac=0.188632
- train=2525, q=2: dense=DENSE_Q02_T2525, crossover=True, quality=H4XH4_FIELD_A150_CPX8_Q02_T2525, systems=H4XH4_FIELD_A150_CPX8_Q02_T2525
  quality_delta_top1=-0.005943, quality_amortized_margin=0.067s
  systems_amortized_margin=0.067s, systems_search_ratio=0.189, systems_cand_frac=0.188632
- train=2550, q=1: dense=DENSE_Q01_T2550, crossover=True, quality=H4XH4_FIELD_A150_CPX8_Q01_T2550, systems=H4XH4_FIELD_A150_CPX8_Q01_T2550
  quality_delta_top1=-0.006275, quality_amortized_margin=0.031s
  systems_amortized_margin=0.031s, systems_search_ratio=0.188, systems_cand_frac=0.187976
- train=2550, q=2: dense=DENSE_Q02_T2550, crossover=True, quality=H4XH4_FIELD_A150_CPX8_Q02_T2550, systems=H4XH4_FIELD_A150_CPX8_Q02_T2550
  quality_delta_top1=-0.006275, quality_amortized_margin=0.070s
  systems_amortized_margin=0.070s, systems_search_ratio=0.188, systems_cand_frac=0.187976
- train=2575, q=1: dense=DENSE_Q01_T2575, crossover=False, quality=n/a, systems=n/a
- train=2575, q=2: dense=DENSE_Q02_T2575, crossover=True, quality=H4XH4_FIELD_A150_CPX8_Q02_T2575, systems=H4XH4_FIELD_A150_CPX8_Q02_T2575
  quality_delta_top1=-0.006605, quality_amortized_margin=0.081s
  systems_amortized_margin=0.081s, systems_search_ratio=0.188, systems_cand_frac=0.187734

## Bank Summaries
- train=2525: first_any=1, first_quality=1, first_systems=1, systems_family=H4XH4_FIELD_A150_CPX8
  systems_ratio_min=0.189, systems_ratio_max=0.189, systems_ratio_span=0.000, systems_margin_slope=0.028132
- train=2550: first_any=1, first_quality=1, first_systems=1, systems_family=H4XH4_FIELD_A150_CPX8
  systems_ratio_min=0.188, systems_ratio_max=0.188, systems_ratio_span=0.000, systems_margin_slope=0.038948
- train=2575: first_any=2, first_quality=2, first_systems=2, systems_family=H4XH4_FIELD_A150_CPX8
  systems_ratio_min=0.188, systems_ratio_max=0.188, systems_ratio_span=0.000, systems_margin_slope=0.135343

## Route Profiles
- DENSE_Q01_T2525: train=2525, q=1, top1=0.051902, cand_count=2525.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.315, online_per_repeat=0.138s, amortized=0.138s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q01_T2525: train=2525, q=1, top1=0.045959, cand_count=476.296, cand_frac=0.188632, work_ratio_vs_dense=0.189, offline_share=0.011, online_share=0.202, online_per_repeat=0.094s, amortized=0.099s, amortized_margin_vs_dense=0.039s
- DENSE_Q02_T2525: train=2525, q=2, top1=0.051902, cand_count=2525.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.466, online_per_repeat=0.120s, amortized=0.120s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q02_T2525: train=2525, q=2, top1=0.045959, cand_count=476.296, cand_frac=0.188632, work_ratio_vs_dense=0.189, offline_share=0.011, online_share=0.213, online_per_repeat=0.050s, amortized=0.053s, amortized_margin_vs_dense=0.067s
- DENSE_Q01_T2550: train=2550, q=1, top1=0.052157, cand_count=2550.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.235, online_per_repeat=0.091s, amortized=0.091s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q01_T2550: train=2550, q=1, top1=0.045882, cand_count=479.340, cand_frac=0.187976, work_ratio_vs_dense=0.188, offline_share=0.012, online_share=0.137, online_per_repeat=0.055s, amortized=0.060s, amortized_margin_vs_dense=0.031s
- DENSE_Q02_T2550: train=2550, q=2, top1=0.052157, cand_count=2550.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.454, online_per_repeat=0.122s, amortized=0.122s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q02_T2550: train=2550, q=2, top1=0.045882, cand_count=479.340, cand_frac=0.187976, work_ratio_vs_dense=0.188, offline_share=0.011, online_share=0.228, online_per_repeat=0.050s, amortized=0.052s, amortized_margin_vs_dense=0.070s
- DENSE_Q01_T2575: train=2575, q=1, top1=0.052059, cand_count=2575.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.285, online_per_repeat=0.116s, amortized=0.116s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q01_T2575: train=2575, q=1, top1=0.045455, cand_count=483.416, cand_frac=0.187734, work_ratio_vs_dense=0.188, offline_share=0.010, online_share=0.296, online_per_repeat=0.164s, amortized=0.170s, amortized_margin_vs_dense=-0.054s
- DENSE_Q02_T2575: train=2575, q=2, top1=0.052059, cand_count=2575.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.484, online_per_repeat=0.139s, amortized=0.139s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q02_T2575: train=2575, q=2, top1=0.045455, cand_count=483.416, cand_frac=0.187734, work_ratio_vs_dense=0.188, offline_share=0.010, online_share=0.225, online_per_repeat=0.055s, amortized=0.058s, amortized_margin_vs_dense=0.081s

