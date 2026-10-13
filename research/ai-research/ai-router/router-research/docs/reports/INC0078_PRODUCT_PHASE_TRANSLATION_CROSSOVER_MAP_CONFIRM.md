# Translated Hardware Profile

## Source Experiments
- `inc0078_product_phase_translation_crossover_map_confirm`

## Group Summaries
- train=3000, q=20: dense=DENSE_Q20_T3000, crossover=False, quality=n/a, systems=n/a
- train=3000, q=24: dense=DENSE_Q24_T3000, crossover=False, quality=n/a, systems=n/a
- train=6000, q=20: dense=DENSE_Q20_T6000, crossover=False, quality=n/a, systems=n/a
- train=6000, q=24: dense=DENSE_Q24_T6000, crossover=True, quality=H4XH4_FIELD_A150_CPX8_Q24_T6000, systems=H4XH4_FIELD_A150_CPX8_Q24_T6000
  quality_delta_top1=-0.001583, quality_amortized_margin=0.016s
  systems_amortized_margin=0.016s, systems_search_ratio=0.187, systems_cand_frac=0.187229
- train=12000, q=12: dense=DENSE_Q12_T12000, crossover=True, quality=H4XH4_FIELD_A150_CPX8_Q12_T12000, systems=H4XH4_FIELD_A150_CPX8_Q12_T12000
  quality_delta_top1=-0.000458, quality_amortized_margin=0.074s
  systems_amortized_margin=0.074s, systems_search_ratio=0.190, systems_cand_frac=0.190318
- train=12000, q=16: dense=DENSE_Q16_T12000, crossover=True, quality=H4XH4_FIELD_A150_Q16_T12000, systems=H4XH4_FIELD_A150_CPX8_Q16_T12000
  quality_delta_top1=0.000000, quality_amortized_margin=0.220s
  systems_amortized_margin=0.345s, systems_search_ratio=0.190, systems_cand_frac=0.190318

## Bank Summaries
- train=3000: first_any=n/a, first_quality=n/a, first_systems=n/a, systems_family=n/a
- train=6000: first_any=24, first_quality=24, first_systems=24, systems_family=H4XH4_FIELD_A150_CPX8
  systems_ratio_min=0.187, systems_ratio_max=0.187, systems_ratio_span=0.000, systems_margin_slope=0.010673
- train=12000: first_any=12, first_quality=12, first_systems=12, systems_family=H4XH4_FIELD_A150_CPX8
  systems_ratio_min=0.190, systems_ratio_max=0.190, systems_ratio_span=0.000, systems_margin_slope=0.067586

## Route Profiles
- DENSE_Q20_T3000: train=3000, q=20, top1=0.049833, cand_count=3000.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.898, online_per_repeat=0.142s, amortized=0.142s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q20_T3000: train=3000, q=20, top1=0.044833, cand_count=575.112, cand_frac=0.191704, work_ratio_vs_dense=0.192, offline_share=0.639, online_share=0.276, online_per_repeat=0.058s, amortized=0.193s, amortized_margin_vs_dense=-0.051s
- H4XH4_FIELD_A150_Q20_T3000: train=3000, q=20, top1=0.049000, cand_count=944.102, cand_frac=0.314701, work_ratio_vs_dense=0.315, offline_share=0.624, online_share=0.298, online_per_repeat=0.066s, amortized=0.205s, amortized_margin_vs_dense=-0.064s
- DENSE_Q24_T3000: train=3000, q=24, top1=0.049833, cand_count=3000.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.909, online_per_repeat=0.127s, amortized=0.127s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q24_T3000: train=3000, q=24, top1=0.044833, cand_count=575.112, cand_frac=0.191704, work_ratio_vs_dense=0.192, offline_share=0.608, online_share=0.309, online_per_repeat=0.058s, amortized=0.171s, amortized_margin_vs_dense=-0.043s
- H4XH4_FIELD_A150_Q24_T3000: train=3000, q=24, top1=0.049000, cand_count=944.102, cand_frac=0.314701, work_ratio_vs_dense=0.315, offline_share=0.602, online_share=0.321, online_per_repeat=0.063s, amortized=0.182s, amortized_margin_vs_dense=-0.055s
- DENSE_Q20_T6000: train=6000, q=20, top1=0.048667, cand_count=6000.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.961, online_per_repeat=0.380s, amortized=0.380s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q20_T6000: train=6000, q=20, top1=0.047083, cand_count=1123.376, cand_frac=0.187229, work_ratio_vs_dense=0.187, offline_share=0.620, online_share=0.328, online_per_repeat=0.141s, amortized=0.407s, amortized_margin_vs_dense=-0.027s
- H4XH4_FIELD_A150_Q20_T6000: train=6000, q=20, top1=0.044500, cand_count=1850.001, cand_frac=0.308333, work_ratio_vs_dense=0.308, offline_share=0.586, online_share=0.366, online_per_repeat=0.165s, amortized=0.430s, amortized_margin_vs_dense=-0.050s
- DENSE_Q24_T6000: train=6000, q=24, top1=0.048667, cand_count=6000.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.967, online_per_repeat=0.380s, amortized=0.380s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q24_T6000: train=6000, q=24, top1=0.047083, cand_count=1123.376, cand_frac=0.187229, work_ratio_vs_dense=0.187, offline_share=0.574, online_share=0.378, online_per_repeat=0.144s, amortized=0.364s, amortized_margin_vs_dense=0.016s
- H4XH4_FIELD_A150_Q24_T6000: train=6000, q=24, top1=0.044500, cand_count=1850.001, cand_frac=0.308333, work_ratio_vs_dense=0.308, offline_share=0.510, online_share=0.448, online_per_repeat=0.194s, amortized=0.414s, amortized_margin_vs_dense=-0.034s
- DENSE_Q12_T12000: train=12000, q=12, top1=0.049125, cand_count=12000.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.980, online_per_repeat=1.320s, amortized=1.320s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q12_T12000: train=12000, q=12, top1=0.048667, cand_count=2283.812, cand_frac=0.190318, work_ratio_vs_dense=0.190, offline_share=0.663, online_share=0.300, online_per_repeat=0.388s, amortized=1.245s, amortized_margin_vs_dense=0.074s
- H4XH4_FIELD_A150_Q12_T12000: train=12000, q=12, top1=0.049125, cand_count=3776.986, cand_frac=0.314749, work_ratio_vs_dense=0.315, offline_share=0.607, online_share=0.360, online_per_repeat=0.527s, amortized=1.416s, amortized_margin_vs_dense=-0.096s
- DENSE_Q16_T12000: train=12000, q=16, top1=0.049125, cand_count=12000.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.985, online_per_repeat=1.371s, amortized=1.371s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q16_T12000: train=12000, q=16, top1=0.048667, cand_count=2283.812, cand_frac=0.190318, work_ratio_vs_dense=0.190, offline_share=0.601, online_share=0.364, online_per_repeat=0.387s, amortized=1.026s, amortized_margin_vs_dense=0.345s
- H4XH4_FIELD_A150_Q16_T12000: train=12000, q=16, top1=0.049125, cand_count=3776.986, cand_frac=0.314749, work_ratio_vs_dense=0.315, offline_share=0.538, online_share=0.431, online_per_repeat=0.512s, amortized=1.151s, amortized_margin_vs_dense=0.220s

