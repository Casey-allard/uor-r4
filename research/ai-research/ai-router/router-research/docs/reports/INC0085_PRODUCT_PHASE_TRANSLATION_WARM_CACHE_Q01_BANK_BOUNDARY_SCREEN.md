# Translated Hardware Profile

## Source Experiments
- `inc0085_product_phase_translation_warm_cache_q01_bank_boundary_screen`

## Group Summaries
- train=3000, q=1: dense=DENSE_Q01_T3000, crossover=True, quality=H4XH4_FIELD_A150_CPX8_Q01_T3000, systems=H4XH4_FIELD_A150_CPX8_Q01_T3000
  quality_delta_top1=-0.004000, quality_amortized_margin=0.079s
  systems_amortized_margin=0.079s, systems_search_ratio=0.185, systems_cand_frac=0.185411
- train=3000, q=2: dense=DENSE_Q02_T3000, crossover=True, quality=H4XH4_FIELD_A150_CPX8_Q02_T3000, systems=H4XH4_FIELD_A150_CPX8_Q02_T3000
  quality_delta_top1=-0.004000, quality_amortized_margin=0.106s
  systems_amortized_margin=0.106s, systems_search_ratio=0.185, systems_cand_frac=0.185411
- train=4500, q=1: dense=DENSE_Q01_T4500, crossover=True, quality=H4XH4_FIELD_A150_CPX8_Q01_T4500, systems=H4XH4_FIELD_A150_CPX8_Q01_T4500
  quality_delta_top1=-0.000222, quality_amortized_margin=0.086s
  systems_amortized_margin=0.086s, systems_search_ratio=0.189, systems_cand_frac=0.189224
- train=4500, q=2: dense=DENSE_Q02_T4500, crossover=True, quality=H4XH4_FIELD_A150_CPX8_Q02_T4500, systems=H4XH4_FIELD_A150_CPX8_Q02_T4500
  quality_delta_top1=-0.000222, quality_amortized_margin=0.126s
  systems_amortized_margin=0.126s, systems_search_ratio=0.189, systems_cand_frac=0.189224
- train=6000, q=1: dense=DENSE_Q01_T6000, crossover=True, quality=H4XH4_FIELD_A150_CPX8_Q01_T6000, systems=H4XH4_FIELD_A150_CPX8_Q01_T6000
  quality_delta_top1=-0.002000, quality_amortized_margin=0.351s
  systems_amortized_margin=0.351s, systems_search_ratio=0.190, systems_cand_frac=0.190455
- train=6000, q=2: dense=DENSE_Q02_T6000, crossover=True, quality=H4XH4_FIELD_A150_CPX8_Q02_T6000, systems=H4XH4_FIELD_A150_CPX8_Q02_T6000
  quality_delta_top1=-0.002000, quality_amortized_margin=0.175s
  systems_amortized_margin=0.175s, systems_search_ratio=0.190, systems_cand_frac=0.190455

## Bank Summaries
- train=3000: first_any=1, first_quality=1, first_systems=1, systems_family=H4XH4_FIELD_A150_CPX8
  systems_ratio_min=0.185, systems_ratio_max=0.185, systems_ratio_span=0.000, systems_margin_slope=0.026797
- train=4500: first_any=1, first_quality=1, first_systems=1, systems_family=H4XH4_FIELD_A150_CPX8
  systems_ratio_min=0.189, systems_ratio_max=0.189, systems_ratio_span=0.000, systems_margin_slope=0.040317
- train=6000: first_any=1, first_quality=1, first_systems=1, systems_family=H4XH4_FIELD_A150_CPX8
  systems_ratio_min=0.190, systems_ratio_max=0.190, systems_ratio_span=0.000, systems_margin_slope=-0.175272

## Route Profiles
- DENSE_Q01_T3000: train=3000, q=1, top1=0.048000, cand_count=3000.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.357, online_per_repeat=0.157s, amortized=0.157s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q01_T3000: train=3000, q=1, top1=0.044000, cand_count=556.234, cand_frac=0.185411, work_ratio_vs_dense=0.185, offline_share=0.013, online_share=0.168, online_per_repeat=0.072s, amortized=0.078s, amortized_margin_vs_dense=0.079s
- DENSE_Q02_T3000: train=3000, q=2, top1=0.048000, cand_count=3000.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.533, online_per_repeat=0.172s, amortized=0.172s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q02_T3000: train=3000, q=2, top1=0.044000, cand_count=556.234, cand_frac=0.185411, work_ratio_vs_dense=0.185, offline_share=0.012, online_share=0.269, online_per_repeat=0.064s, amortized=0.066s, amortized_margin_vs_dense=0.106s
- DENSE_Q01_T4500: train=4500, q=1, top1=0.047333, cand_count=4500.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.473, online_per_repeat=0.254s, amortized=0.254s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q01_T4500: train=4500, q=1, top1=0.047111, cand_count=851.508, cand_frac=0.189224, work_ratio_vs_dense=0.189, offline_share=0.013, online_share=0.267, online_per_repeat=0.161s, amortized=0.169s, amortized_margin_vs_dense=0.086s
- DENSE_Q02_T4500: train=4500, q=2, top1=0.047333, cand_count=4500.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.619, online_per_repeat=0.231s, amortized=0.231s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q02_T4500: train=4500, q=2, top1=0.047111, cand_count=851.508, cand_frac=0.189224, work_ratio_vs_dense=0.189, offline_share=0.013, online_share=0.328, online_per_repeat=0.101s, amortized=0.105s, amortized_margin_vs_dense=0.126s
- DENSE_Q01_T6000: train=6000, q=1, top1=0.049667, cand_count=6000.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.628, online_per_repeat=0.509s, amortized=0.509s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q01_T6000: train=6000, q=1, top1=0.047667, cand_count=1142.733, cand_frac=0.190455, work_ratio_vs_dense=0.190, offline_share=0.018, online_share=0.253, online_per_repeat=0.148s, amortized=0.158s, amortized_margin_vs_dense=0.351s
- DENSE_Q02_T6000: train=6000, q=2, top1=0.049667, cand_count=6000.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.706, online_per_repeat=0.363s, amortized=0.363s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q02_T6000: train=6000, q=2, top1=0.047667, cand_count=1142.733, cand_frac=0.190455, work_ratio_vs_dense=0.190, offline_share=0.012, online_share=0.452, online_per_repeat=0.183s, amortized=0.188s, amortized_margin_vs_dense=0.175s

