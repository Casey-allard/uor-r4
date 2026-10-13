# Translated Hardware Profile

## Source Experiments
- `inc0078_product_phase_translation_crossover_map_screen`

## Group Summaries
- train=3000, q=12: dense=DENSE_Q12_T3000, crossover=False, quality=n/a, systems=n/a
- train=3000, q=16: dense=DENSE_Q16_T3000, crossover=False, quality=n/a, systems=n/a
- train=3000, q=20: dense=DENSE_Q20_T3000, crossover=False, quality=n/a, systems=n/a
- train=3000, q=24: dense=DENSE_Q24_T3000, crossover=False, quality=n/a, systems=n/a
- train=6000, q=12: dense=DENSE_Q12_T6000, crossover=False, quality=n/a, systems=n/a
- train=6000, q=16: dense=DENSE_Q16_T6000, crossover=False, quality=n/a, systems=n/a
- train=6000, q=20: dense=DENSE_Q20_T6000, crossover=False, quality=n/a, systems=n/a
- train=6000, q=24: dense=DENSE_Q24_T6000, crossover=True, quality=H4XH4_FIELD_A150_CPX8_Q24_T6000, systems=H4XH4_FIELD_A150_CPX8_Q24_T6000
  quality_delta_top1=-0.002000, quality_amortized_margin=0.013s
  systems_amortized_margin=0.013s, systems_search_ratio=0.190, systems_cand_frac=0.190455
- train=12000, q=12: dense=DENSE_Q12_T12000, crossover=True, quality=H4XH4_FIELD_A150_CPX8_Q12_T12000, systems=H4XH4_FIELD_A150_CPX8_Q12_T12000
  quality_delta_top1=0.000583, quality_amortized_margin=0.059s
  systems_amortized_margin=0.059s, systems_search_ratio=0.184, systems_cand_frac=0.184181
- train=12000, q=16: dense=DENSE_Q16_T12000, crossover=True, quality=H4XH4_FIELD_A150_Q16_T12000, systems=H4XH4_FIELD_A150_CPX8_Q16_T12000
  quality_delta_top1=0.002583, quality_amortized_margin=0.199s
  systems_amortized_margin=0.369s, systems_search_ratio=0.184, systems_cand_frac=0.184181
- train=12000, q=20: dense=DENSE_Q20_T12000, crossover=True, quality=H4XH4_FIELD_A150_Q20_T12000, systems=H4XH4_FIELD_A150_CPX8_Q20_T12000
  quality_delta_top1=0.002583, quality_amortized_margin=0.366s
  systems_amortized_margin=0.479s, systems_search_ratio=0.184, systems_cand_frac=0.184181
- train=12000, q=24: dense=DENSE_Q24_T12000, crossover=True, quality=H4XH4_FIELD_A150_Q24_T12000, systems=H4XH4_FIELD_A150_CPX8_Q24_T12000
  quality_delta_top1=0.002583, quality_amortized_margin=0.381s
  systems_amortized_margin=0.504s, systems_search_ratio=0.184, systems_cand_frac=0.184181

## Bank Summaries
- train=3000: first_any=n/a, first_quality=n/a, first_systems=n/a, systems_family=n/a
- train=6000: first_any=24, first_quality=24, first_systems=24, systems_family=H4XH4_FIELD_A150_CPX8
  systems_ratio_min=0.190, systems_ratio_max=0.190, systems_ratio_span=0.000, systems_margin_slope=0.017516
- train=12000: first_any=12, first_quality=12, first_systems=12, systems_family=H4XH4_FIELD_A150_CPX8
  systems_ratio_min=0.184, systems_ratio_max=0.184, systems_ratio_span=0.000, systems_margin_slope=0.037100

## Route Profiles
- DENSE_Q12_T3000: train=3000, q=12, top1=0.048000, cand_count=3000.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.854, online_per_repeat=0.139s, amortized=0.139s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q12_T3000: train=3000, q=12, top1=0.044000, cand_count=556.234, cand_frac=0.185411, work_ratio_vs_dense=0.185, offline_share=0.647, online_share=0.268, online_per_repeat=0.105s, amortized=0.359s, amortized_margin_vs_dense=-0.220s
- H4XH4_FIELD_A150_Q12_T3000: train=3000, q=12, top1=0.047000, cand_count=922.849, cand_frac=0.307616, work_ratio_vs_dense=0.308, offline_share=0.708, online_share=0.213, online_per_repeat=0.082s, amortized=0.353s, amortized_margin_vs_dense=-0.214s
- DENSE_Q16_T3000: train=3000, q=16, top1=0.048000, cand_count=3000.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.879, online_per_repeat=0.142s, amortized=0.142s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q16_T3000: train=3000, q=16, top1=0.044000, cand_count=556.234, cand_frac=0.185411, work_ratio_vs_dense=0.185, offline_share=0.687, online_share=0.220, online_per_repeat=0.055s, amortized=0.229s, amortized_margin_vs_dense=-0.087s
- H4XH4_FIELD_A150_Q16_T3000: train=3000, q=16, top1=0.047000, cand_count=922.849, cand_frac=0.307616, work_ratio_vs_dense=0.308, offline_share=0.663, online_share=0.253, online_per_repeat=0.069s, amortized=0.249s, amortized_margin_vs_dense=-0.107s
- DENSE_Q20_T3000: train=3000, q=20, top1=0.048000, cand_count=3000.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.905, online_per_repeat=0.143s, amortized=0.143s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q20_T3000: train=3000, q=20, top1=0.044000, cand_count=556.234, cand_frac=0.185411, work_ratio_vs_dense=0.185, offline_share=0.671, online_share=0.246, online_per_repeat=0.055s, amortized=0.204s, amortized_margin_vs_dense=-0.061s
- H4XH4_FIELD_A150_Q20_T3000: train=3000, q=20, top1=0.047000, cand_count=922.849, cand_frac=0.307616, work_ratio_vs_dense=0.308, offline_share=0.625, online_share=0.297, online_per_repeat=0.070s, amortized=0.216s, amortized_margin_vs_dense=-0.073s
- DENSE_Q24_T3000: train=3000, q=24, top1=0.048000, cand_count=3000.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.924, online_per_repeat=0.158s, amortized=0.158s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q24_T3000: train=3000, q=24, top1=0.044000, cand_count=556.234, cand_frac=0.185411, work_ratio_vs_dense=0.185, offline_share=0.623, online_share=0.296, online_per_repeat=0.054s, amortized=0.168s, amortized_margin_vs_dense=-0.010s
- H4XH4_FIELD_A150_Q24_T3000: train=3000, q=24, top1=0.047000, cand_count=922.849, cand_frac=0.307616, work_ratio_vs_dense=0.308, offline_share=0.597, online_share=0.321, online_per_repeat=0.064s, amortized=0.183s, amortized_margin_vs_dense=-0.026s
- DENSE_Q12_T6000: train=6000, q=12, top1=0.049667, cand_count=6000.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.936, online_per_repeat=0.387s, amortized=0.387s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q12_T6000: train=6000, q=12, top1=0.047667, cand_count=1142.733, cand_frac=0.190455, work_ratio_vs_dense=0.190, offline_share=0.713, online_share=0.228, online_per_repeat=0.141s, amortized=0.584s, amortized_margin_vs_dense=-0.197s
- H4XH4_FIELD_A150_Q12_T6000: train=6000, q=12, top1=0.045667, cand_count=1888.372, cand_frac=0.314729, work_ratio_vs_dense=0.315, offline_share=0.687, online_share=0.254, online_per_repeat=0.165s, amortized=0.612s, amortized_margin_vs_dense=-0.225s
- DENSE_Q16_T6000: train=6000, q=16, top1=0.049667, cand_count=6000.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.952, online_per_repeat=0.389s, amortized=0.389s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q16_T6000: train=6000, q=16, top1=0.047667, cand_count=1142.733, cand_frac=0.190455, work_ratio_vs_dense=0.190, offline_share=0.655, online_share=0.289, online_per_repeat=0.147s, amortized=0.481s, amortized_margin_vs_dense=-0.092s
- H4XH4_FIELD_A150_Q16_T6000: train=6000, q=16, top1=0.045667, cand_count=1888.372, cand_frac=0.314729, work_ratio_vs_dense=0.315, offline_share=0.631, online_share=0.317, online_per_repeat=0.168s, amortized=0.501s, amortized_margin_vs_dense=-0.112s
- DENSE_Q20_T6000: train=6000, q=20, top1=0.049667, cand_count=6000.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.961, online_per_repeat=0.380s, amortized=0.380s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q20_T6000: train=6000, q=20, top1=0.047667, cand_count=1142.733, cand_frac=0.190455, work_ratio_vs_dense=0.190, offline_share=0.629, online_share=0.320, online_per_repeat=0.144s, amortized=0.426s, amortized_margin_vs_dense=-0.046s
- H4XH4_FIELD_A150_Q20_T6000: train=6000, q=20, top1=0.045667, cand_count=1888.372, cand_frac=0.314729, work_ratio_vs_dense=0.315, offline_share=0.573, online_share=0.379, online_per_repeat=0.174s, amortized=0.438s, amortized_margin_vs_dense=-0.058s
- DENSE_Q24_T6000: train=6000, q=24, top1=0.049667, cand_count=6000.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.969, online_per_repeat=0.400s, amortized=0.400s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q24_T6000: train=6000, q=24, top1=0.047667, cand_count=1142.733, cand_frac=0.190455, work_ratio_vs_dense=0.190, offline_share=0.613, online_share=0.342, online_per_repeat=0.138s, amortized=0.387s, amortized_margin_vs_dense=0.013s
- H4XH4_FIELD_A150_Q24_T6000: train=6000, q=24, top1=0.045667, cand_count=1888.372, cand_frac=0.314729, work_ratio_vs_dense=0.315, offline_share=0.544, online_share=0.407, online_per_repeat=0.169s, amortized=0.395s, amortized_margin_vs_dense=0.005s
- DENSE_Q12_T12000: train=12000, q=12, top1=0.048333, cand_count=12000.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.979, online_per_repeat=1.276s, amortized=1.276s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q12_T12000: train=12000, q=12, top1=0.048917, cand_count=2210.172, cand_frac=0.184181, work_ratio_vs_dense=0.184, offline_share=0.661, online_share=0.301, online_per_repeat=0.381s, amortized=1.217s, amortized_margin_vs_dense=0.059s
- H4XH4_FIELD_A150_Q12_T12000: train=12000, q=12, top1=0.050917, cand_count=3683.316, cand_frac=0.306943, work_ratio_vs_dense=0.307, offline_share=0.606, online_share=0.360, online_per_repeat=0.509s, amortized=1.365s, amortized_margin_vs_dense=-0.090s
- DENSE_Q16_T12000: train=12000, q=16, top1=0.048333, cand_count=12000.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.985, online_per_repeat=1.374s, amortized=1.374s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q16_T12000: train=12000, q=16, top1=0.048917, cand_count=2210.172, cand_frac=0.184181, work_ratio_vs_dense=0.184, offline_share=0.604, online_share=0.360, online_per_repeat=0.375s, amortized=1.005s, amortized_margin_vs_dense=0.369s
- H4XH4_FIELD_A150_Q16_T12000: train=12000, q=16, top1=0.050917, cand_count=3683.316, cand_frac=0.306943, work_ratio_vs_dense=0.307, offline_share=0.528, online_share=0.442, online_per_repeat=0.535s, amortized=1.175s, amortized_margin_vs_dense=0.199s
- DENSE_Q20_T12000: train=12000, q=20, top1=0.048333, cand_count=12000.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.987, online_per_repeat=1.362s, amortized=1.362s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q20_T12000: train=12000, q=20, top1=0.048917, cand_count=2210.172, cand_frac=0.184181, work_ratio_vs_dense=0.184, offline_share=0.553, online_share=0.415, online_per_repeat=0.379s, amortized=0.883s, amortized_margin_vs_dense=0.479s
- H4XH4_FIELD_A150_Q20_T12000: train=12000, q=20, top1=0.050917, cand_count=3683.316, cand_frac=0.306943, work_ratio_vs_dense=0.307, offline_share=0.492, online_share=0.480, online_per_repeat=0.492s, amortized=0.996s, amortized_margin_vs_dense=0.366s
- DENSE_Q24_T12000: train=12000, q=24, top1=0.048333, cand_count=12000.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.989, online_per_repeat=1.294s, amortized=1.294s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q24_T12000: train=12000, q=24, top1=0.048917, cand_count=2210.172, cand_frac=0.184181, work_ratio_vs_dense=0.184, offline_share=0.508, online_share=0.463, online_per_repeat=0.377s, amortized=0.790s, amortized_margin_vs_dense=0.504s
- H4XH4_FIELD_A150_Q24_T12000: train=12000, q=24, top1=0.050917, cand_count=3683.316, cand_frac=0.306943, work_ratio_vs_dense=0.307, offline_share=0.442, online_share=0.531, online_per_repeat=0.498s, amortized=0.913s, amortized_margin_vs_dense=0.381s

