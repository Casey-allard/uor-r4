# Translated Hardware Profile

## Source Experiments
- `inc0080_product_phase_translation_second_large_bank_boundary_extension_screen`

## Group Summaries
- train=24000, q=4: dense=DENSE_Q04_T24000, crossover=False, quality=n/a, systems=n/a
- train=24000, q=8: dense=DENSE_Q08_T24000, crossover=True, quality=H4XH4_FIELD_A150_CPX8_Q08_T24000, systems=H4XH4_FIELD_A150_CPX8_Q08_T24000
  quality_delta_top1=-0.000083, quality_amortized_margin=2.114s
  systems_amortized_margin=2.114s, systems_search_ratio=0.191, systems_cand_frac=0.191436
- train=24000, q=12: dense=DENSE_Q12_T24000, crossover=True, quality=H4XH4_FIELD_A150_CPX8_Q12_T24000, systems=H4XH4_FIELD_A150_CPX8_Q12_T24000
  quality_delta_top1=-0.000083, quality_amortized_margin=2.047s
  systems_amortized_margin=2.047s, systems_search_ratio=0.191, systems_cand_frac=0.191436
- train=30000, q=4: dense=DENSE_Q04_T30000, crossover=False, quality=n/a, systems=n/a
- train=30000, q=8: dense=DENSE_Q08_T30000, crossover=True, quality=H4XH4_FIELD_A150_CPX8_Q08_T30000, systems=H4XH4_FIELD_A150_CPX8_Q08_T30000
  quality_delta_top1=-0.000967, quality_amortized_margin=3.860s
  systems_amortized_margin=3.860s, systems_search_ratio=0.182, systems_cand_frac=0.182290
- train=30000, q=12: dense=DENSE_Q12_T30000, crossover=True, quality=H4XH4_FIELD_A150_CPX8_Q12_T30000, systems=H4XH4_FIELD_A150_CPX8_Q12_T30000
  quality_delta_top1=-0.000967, quality_amortized_margin=5.413s
  systems_amortized_margin=5.413s, systems_search_ratio=0.182, systems_cand_frac=0.182290

## Bank Summaries
- train=24000: first_any=8, first_quality=8, first_systems=8, systems_family=H4XH4_FIELD_A150_CPX8
  systems_ratio_min=0.191, systems_ratio_max=0.191, systems_ratio_span=0.000, systems_margin_slope=0.412196
- train=30000: first_any=8, first_quality=8, first_systems=8, systems_family=H4XH4_FIELD_A150_CPX8
  systems_ratio_min=0.182, systems_ratio_max=0.182, systems_ratio_span=0.000, systems_margin_slope=0.715374

## Route Profiles
- DENSE_Q04_T24000: train=24000, q=4, top1=0.049333, cand_count=24000.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.980, online_per_repeat=5.162s, amortized=5.162s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q04_T24000: train=24000, q=4, top1=0.049250, cand_count=4594.463, cand_frac=0.191436, work_ratio_vs_dense=0.191, offline_share=0.787, online_share=0.179, online_per_repeat=1.189s, amortized=6.413s, amortized_margin_vs_dense=-1.250s
- DENSE_Q08_T24000: train=24000, q=8, top1=0.049333, cand_count=24000.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.987, online_per_repeat=5.894s, amortized=5.894s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q08_T24000: train=24000, q=8, top1=0.049250, cand_count=4594.463, cand_frac=0.191436, work_ratio_vs_dense=0.191, offline_share=0.676, online_share=0.297, online_per_repeat=1.154s, amortized=3.781s, amortized_margin_vs_dense=2.114s
- DENSE_Q12_T24000: train=24000, q=12, top1=0.049333, cand_count=24000.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.994, online_per_repeat=4.988s, amortized=4.988s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q12_T24000: train=24000, q=12, top1=0.049250, cand_count=4594.463, cand_frac=0.191436, work_ratio_vs_dense=0.191, offline_share=0.552, online_share=0.424, online_per_repeat=1.278s, amortized=2.941s, amortized_margin_vs_dense=2.047s
- DENSE_Q04_T30000: train=30000, q=4, top1=0.047733, cand_count=30000.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.988, online_per_repeat=7.944s, amortized=7.944s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q04_T30000: train=30000, q=4, top1=0.046767, cand_count=5468.699, cand_frac=0.182290, work_ratio_vs_dense=0.182, offline_share=0.778, online_share=0.192, online_per_repeat=1.631s, amortized=8.254s, amortized_margin_vs_dense=-0.310s
- DENSE_Q08_T30000: train=30000, q=8, top1=0.047733, cand_count=30000.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.994, online_per_repeat=8.707s, amortized=8.707s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q08_T30000: train=30000, q=8, top1=0.046767, cand_count=5468.699, cand_frac=0.182290, work_ratio_vs_dense=0.182, offline_share=0.652, online_share=0.323, online_per_repeat=1.605s, amortized=4.847s, amortized_margin_vs_dense=3.860s
- DENSE_Q12_T30000: train=30000, q=12, top1=0.047733, cand_count=30000.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.996, online_per_repeat=9.101s, amortized=9.101s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q12_T30000: train=30000, q=12, top1=0.046767, cand_count=5468.699, cand_frac=0.182290, work_ratio_vs_dense=0.182, offline_share=0.556, online_share=0.421, online_per_repeat=1.590s, amortized=3.687s, amortized_margin_vs_dense=5.413s

