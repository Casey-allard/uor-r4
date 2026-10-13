# Translated Hardware Profile

## Source Experiments
- `inc0079_product_phase_translation_large_bank_boundary_extension_screen`

## Group Summaries
- train=12000, q=8: dense=DENSE_Q08_T12000, crossover=False, quality=n/a, systems=n/a
- train=12000, q=12: dense=DENSE_Q12_T12000, crossover=True, quality=H4XH4_FIELD_A150_CPX8_Q12_T12000, systems=H4XH4_FIELD_A150_CPX8_Q12_T12000
  quality_delta_top1=0.000583, quality_amortized_margin=0.073s
  systems_amortized_margin=0.073s, systems_search_ratio=0.184, systems_cand_frac=0.184181
- train=12000, q=16: dense=DENSE_Q16_T12000, crossover=True, quality=H4XH4_FIELD_A150_CPX8_Q16_T12000, systems=H4XH4_FIELD_A150_CPX8_Q16_T12000
  quality_delta_top1=0.000583, quality_amortized_margin=0.253s
  systems_amortized_margin=0.253s, systems_search_ratio=0.184, systems_cand_frac=0.184181
- train=18000, q=8: dense=DENSE_Q08_T18000, crossover=True, quality=H4XH4_FIELD_A150_CPX8_Q08_T18000, systems=H4XH4_FIELD_A150_CPX8_Q08_T18000
  quality_delta_top1=-0.000167, quality_amortized_margin=0.567s
  systems_amortized_margin=0.567s, systems_search_ratio=0.183, systems_cand_frac=0.183255
- train=18000, q=12: dense=DENSE_Q12_T18000, crossover=True, quality=H4XH4_FIELD_A150_CPX8_Q12_T18000, systems=H4XH4_FIELD_A150_CPX8_Q12_T18000
  quality_delta_top1=-0.000167, quality_amortized_margin=1.187s
  systems_amortized_margin=1.187s, systems_search_ratio=0.183, systems_cand_frac=0.183255
- train=18000, q=16: dense=DENSE_Q16_T18000, crossover=True, quality=H4XH4_FIELD_A150_CPX8_Q16_T18000, systems=H4XH4_FIELD_A150_CPX8_Q16_T18000
  quality_delta_top1=-0.000167, quality_amortized_margin=1.428s
  systems_amortized_margin=1.428s, systems_search_ratio=0.183, systems_cand_frac=0.183255

## Bank Summaries
- train=12000: first_any=12, first_quality=12, first_systems=12, systems_family=H4XH4_FIELD_A150_CPX8
  systems_ratio_min=0.184, systems_ratio_max=0.184, systems_ratio_span=0.000, systems_margin_slope=0.069441
- train=18000: first_any=8, first_quality=8, first_systems=8, systems_family=H4XH4_FIELD_A150_CPX8
  systems_ratio_min=0.183, systems_ratio_max=0.183, systems_ratio_span=0.000, systems_margin_slope=0.107604

## Route Profiles
- DENSE_Q08_T12000: train=12000, q=8, top1=0.048333, cand_count=12000.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.971, online_per_repeat=1.424s, amortized=1.424s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q08_T12000: train=12000, q=8, top1=0.048917, cand_count=2210.172, cand_frac=0.184181, work_ratio_vs_dense=0.184, offline_share=0.739, online_share=0.218, online_per_repeat=0.394s, amortized=1.726s, amortized_margin_vs_dense=-0.302s
- DENSE_Q12_T12000: train=12000, q=12, top1=0.048333, cand_count=12000.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.980, online_per_repeat=1.344s, amortized=1.344s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q12_T12000: train=12000, q=12, top1=0.048917, cand_count=2210.172, cand_frac=0.184181, work_ratio_vs_dense=0.184, offline_share=0.656, online_share=0.307, online_per_repeat=0.405s, amortized=1.271s, amortized_margin_vs_dense=0.073s
- DENSE_Q16_T12000: train=12000, q=16, top1=0.048333, cand_count=12000.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.985, online_per_repeat=1.361s, amortized=1.361s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q16_T12000: train=12000, q=16, top1=0.048917, cand_count=2210.172, cand_frac=0.184181, work_ratio_vs_dense=0.184, offline_share=0.585, online_share=0.381, online_per_repeat=0.437s, amortized=1.108s, amortized_margin_vs_dense=0.253s
- DENSE_Q08_T18000: train=18000, q=8, top1=0.047500, cand_count=18000.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.986, online_per_repeat=3.239s, amortized=3.239s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q08_T18000: train=18000, q=8, top1=0.047333, cand_count=3298.590, cand_frac=0.183255, work_ratio_vs_dense=0.183, offline_share=0.697, online_share=0.269, online_per_repeat=0.744s, amortized=2.671s, amortized_margin_vs_dense=0.567s
- DENSE_Q12_T18000: train=18000, q=12, top1=0.047500, cand_count=18000.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.991, online_per_repeat=3.216s, amortized=3.216s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q12_T18000: train=18000, q=12, top1=0.047333, cand_count=3298.590, cand_frac=0.183255, work_ratio_vs_dense=0.183, offline_share=0.629, online_share=0.342, online_per_repeat=0.715s, amortized=2.029s, amortized_margin_vs_dense=1.187s
- DENSE_Q16_T18000: train=18000, q=16, top1=0.047500, cand_count=18000.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.993, online_per_repeat=3.172s, amortized=3.172s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q16_T18000: train=18000, q=16, top1=0.047333, cand_count=3298.590, cand_frac=0.183255, work_ratio_vs_dense=0.183, offline_share=0.548, online_share=0.426, online_per_repeat=0.763s, amortized=1.744s, amortized_margin_vs_dense=1.428s

