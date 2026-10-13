# Translated Hardware Profile

## Source Experiments
- `inc0081_product_phase_translation_q04_threshold_search_screen`

## Group Summaries
- train=36000, q=4: dense=DENSE_Q04_T36000, crossover=True, quality=H4XH4_FIELD_A150_CPX8_Q04_T36000, systems=H4XH4_FIELD_A150_CPX8_Q04_T36000
  quality_delta_top1=-0.000833, quality_amortized_margin=1.362s
  systems_amortized_margin=1.362s, systems_search_ratio=0.185, systems_cand_frac=0.185074
- train=36000, q=8: dense=DENSE_Q08_T36000, crossover=True, quality=H4XH4_FIELD_A150_CPX8_Q08_T36000, systems=H4XH4_FIELD_A150_CPX8_Q08_T36000
  quality_delta_top1=-0.000833, quality_amortized_margin=5.082s
  systems_amortized_margin=5.082s, systems_search_ratio=0.185, systems_cand_frac=0.185074
- train=40000, q=4: dense=DENSE_Q04_T40000, crossover=False, quality=n/a, systems=n/a
- train=40000, q=8: dense=DENSE_Q08_T40000, crossover=True, quality=H4XH4_FIELD_A150_CPX8_Q08_T40000, systems=H4XH4_FIELD_A150_CPX8_Q08_T40000
  quality_delta_top1=-0.002050, quality_amortized_margin=3.239s
  systems_amortized_margin=3.239s, systems_search_ratio=0.187, systems_cand_frac=0.187484

## Bank Summaries
- train=36000: first_any=4, first_quality=4, first_systems=4, systems_family=H4XH4_FIELD_A150_CPX8
  systems_ratio_min=0.185, systems_ratio_max=0.185, systems_ratio_span=0.000, systems_margin_slope=0.930090
- train=40000: first_any=8, first_quality=8, first_systems=8, systems_family=H4XH4_FIELD_A150_CPX8
  systems_ratio_min=0.187, systems_ratio_max=0.187, systems_ratio_span=0.000, systems_margin_slope=1.095991

## Route Profiles
- DENSE_Q04_T36000: train=36000, q=4, top1=0.047778, cand_count=36000.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.991, online_per_repeat=11.109s, amortized=11.109s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q04_T36000: train=36000, q=4, top1=0.046944, cand_count=6662.650, cand_frac=0.185074, work_ratio_vs_dense=0.185, offline_share=0.753, online_share=0.218, online_per_repeat=2.188s, amortized=9.748s, amortized_margin_vs_dense=1.362s
- DENSE_Q08_T36000: train=36000, q=8, top1=0.047778, cand_count=36000.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.995, online_per_repeat=10.982s, amortized=10.982s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q08_T36000: train=36000, q=8, top1=0.046944, cand_count=6662.650, cand_frac=0.185074, work_ratio_vs_dense=0.185, offline_share=0.614, online_share=0.361, online_per_repeat=2.183s, amortized=5.900s, amortized_margin_vs_dense=5.082s
- DENSE_Q04_T40000: train=40000, q=4, top1=0.049875, cand_count=40000.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.989, online_per_repeat=8.892s, amortized=8.892s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q04_T40000: train=40000, q=4, top1=0.047825, cand_count=7499.373, cand_frac=0.187484, work_ratio_vs_dense=0.187, offline_share=0.781, online_share=0.191, online_per_repeat=1.972s, amortized=10.037s, amortized_margin_vs_dense=-1.145s
- DENSE_Q08_T40000: train=40000, q=8, top1=0.049875, cand_count=40000.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.995, online_per_repeat=9.149s, amortized=9.149s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q08_T40000: train=40000, q=8, top1=0.047825, cand_count=7499.373, cand_frac=0.187484, work_ratio_vs_dense=0.187, offline_share=0.672, online_share=0.303, online_per_repeat=1.838s, amortized=5.910s, amortized_margin_vs_dense=3.239s

