# Translated Hardware Profile

## Source Experiments
- `inc0081_product_phase_translation_q04_threshold_search_confirm`

## Group Summaries
- train=36000, q=4: dense=DENSE_Q04_T36000, crossover=True, quality=H4XH4_FIELD_A150_CPX8_Q04_T36000, systems=H4XH4_FIELD_A150_CPX8_Q04_T36000
  quality_delta_top1=-0.000833, quality_amortized_margin=2.455s
  systems_amortized_margin=2.455s, systems_search_ratio=0.190, systems_cand_frac=0.190206
- train=36000, q=8: dense=DENSE_Q08_T36000, crossover=True, quality=H4XH4_FIELD_A150_CPX8_Q08_T36000, systems=H4XH4_FIELD_A150_CPX8_Q08_T36000
  quality_delta_top1=-0.000833, quality_amortized_margin=5.143s
  systems_amortized_margin=5.143s, systems_search_ratio=0.190, systems_cand_frac=0.190206
- train=40000, q=4: dense=DENSE_Q04_T40000, crossover=False, quality=n/a, systems=n/a
- train=40000, q=8: dense=DENSE_Q08_T40000, crossover=True, quality=H4XH4_FIELD_A150_CPX8_Q08_T40000, systems=H4XH4_FIELD_A150_CPX8_Q08_T40000
  quality_delta_top1=-0.001525, quality_amortized_margin=3.536s
  systems_amortized_margin=3.536s, systems_search_ratio=0.184, systems_cand_frac=0.183764

## Bank Summaries
- train=36000: first_any=4, first_quality=4, first_systems=4, systems_family=H4XH4_FIELD_A150_CPX8
  systems_ratio_min=0.190, systems_ratio_max=0.190, systems_ratio_span=0.000, systems_margin_slope=0.672131
- train=40000: first_any=8, first_quality=8, first_systems=8, systems_family=H4XH4_FIELD_A150_CPX8
  systems_ratio_min=0.184, systems_ratio_max=0.184, systems_ratio_span=0.000, systems_margin_slope=1.125558

## Route Profiles
- DENSE_Q04_T36000: train=36000, q=4, top1=0.047903, cand_count=36000.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.991, online_per_repeat=12.149s, amortized=12.149s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q04_T36000: train=36000, q=4, top1=0.047069, cand_count=6847.432, cand_frac=0.190206, work_ratio_vs_dense=0.190, offline_share=0.743, online_share=0.229, online_per_repeat=2.287s, amortized=9.694s, amortized_margin_vs_dense=2.455s
- DENSE_Q08_T36000: train=36000, q=8, top1=0.047903, cand_count=36000.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.995, online_per_repeat=11.093s, amortized=11.093s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q08_T36000: train=36000, q=8, top1=0.047069, cand_count=6847.432, cand_frac=0.190206, work_ratio_vs_dense=0.190, offline_share=0.612, online_share=0.362, online_per_repeat=2.212s, amortized=5.950s, amortized_margin_vs_dense=5.143s
- DENSE_Q04_T40000: train=40000, q=4, top1=0.048850, cand_count=40000.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.989, online_per_repeat=9.024s, amortized=9.024s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q04_T40000: train=40000, q=4, top1=0.047325, cand_count=7350.555, cand_frac=0.183764, work_ratio_vs_dense=0.184, offline_share=0.780, online_share=0.189, online_per_repeat=1.952s, amortized=9.990s, amortized_margin_vs_dense=-0.966s
- DENSE_Q08_T40000: train=40000, q=8, top1=0.048850, cand_count=40000.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.995, online_per_repeat=9.642s, amortized=9.642s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q08_T40000: train=40000, q=8, top1=0.047325, cand_count=7350.555, cand_frac=0.183764, work_ratio_vs_dense=0.184, offline_share=0.659, online_share=0.317, online_per_repeat=1.985s, amortized=6.106s, amortized_margin_vs_dense=3.536s

