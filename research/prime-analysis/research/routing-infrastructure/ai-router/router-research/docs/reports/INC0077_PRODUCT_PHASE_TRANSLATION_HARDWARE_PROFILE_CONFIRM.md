# Translated Hardware Profile

## Source Experiments
- `inc0076_product_phase_translation_break_even_confirm`
- `inc0077_product_phase_translation_hardware_profile_confirm`

## Group Summaries
- train=6000, q=16: dense=DENSE_Q16_T6000, crossover=False, quality=n/a, systems=n/a
- train=6000, q=24: dense=DENSE_Q24_T6000, crossover=True, quality=H4XH4_FIELD_A150_CPX8_Q24_T6000, systems=H4XH4_FIELD_A150_CPX8_Q24_T6000
  quality_delta_top1=-0.001583, quality_amortized_margin=0.032s
  systems_amortized_margin=0.032s, systems_search_ratio=0.187, systems_cand_frac=0.187229
- train=12000, q=8: dense=DENSE_Q08, crossover=False, quality=n/a, systems=n/a
- train=12000, q=16: dense=DENSE_Q16, crossover=True, quality=H4XH4_FIELD_A150_Q16, systems=H4XH4_FIELD_A150_CPX8_Q16
  quality_delta_top1=0.000000, quality_amortized_margin=0.159s
  systems_amortized_margin=0.286s, systems_search_ratio=0.190, systems_cand_frac=0.190318
- train=12000, q=24: dense=DENSE_Q24, crossover=True, quality=H4XH4_FIELD_A150_Q24, systems=H4XH4_FIELD_A150_CPX8_Q24
  quality_delta_top1=0.000000, quality_amortized_margin=0.397s
  systems_amortized_margin=0.497s, systems_search_ratio=0.190, systems_cand_frac=0.190318

## Route Profiles
- DENSE_Q16_T6000: train=6000, q=16, top1=0.048667, cand_count=6000.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.952, online_per_repeat=0.381s, amortized=0.381s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q16_T6000: train=6000, q=16, top1=0.047083, cand_count=1123.376, cand_frac=0.187229, work_ratio_vs_dense=0.187, offline_share=0.671, online_share=0.276, online_per_repeat=0.135s, amortized=0.464s, amortized_margin_vs_dense=-0.084s
- H4XH4_FIELD_A150_Q16_T6000: train=6000, q=16, top1=0.044500, cand_count=1850.001, cand_frac=0.308333, work_ratio_vs_dense=0.308, offline_share=0.629, online_share=0.319, online_per_repeat=0.165s, amortized=0.490s, amortized_margin_vs_dense=-0.109s
- DENSE_Q24_T6000: train=6000, q=24, top1=0.048667, cand_count=6000.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.967, online_per_repeat=0.382s, amortized=0.382s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q24_T6000: train=6000, q=24, top1=0.047083, cand_count=1123.376, cand_frac=0.187229, work_ratio_vs_dense=0.187, offline_share=0.590, online_share=0.363, online_per_repeat=0.134s, amortized=0.351s, amortized_margin_vs_dense=0.032s
- H4XH4_FIELD_A150_Q24_T6000: train=6000, q=24, top1=0.044500, cand_count=1850.001, cand_frac=0.308333, work_ratio_vs_dense=0.308, offline_share=0.553, online_share=0.403, online_per_repeat=0.166s, amortized=0.394s, amortized_margin_vs_dense=-0.012s
- DENSE_Q08: train=12000, q=8, top1=0.049125, cand_count=12000.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.970, online_per_repeat=1.377s, amortized=1.377s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q08: train=12000, q=8, top1=0.048667, cand_count=2283.812, cand_frac=0.190318, work_ratio_vs_dense=0.190, offline_share=0.714, online_share=0.245, online_per_repeat=0.431s, amortized=1.691s, amortized_margin_vs_dense=-0.314s
- H4XH4_FIELD_A150_Q08: train=12000, q=8, top1=0.049125, cand_count=3776.986, cand_frac=0.314749, work_ratio_vs_dense=0.315, offline_share=0.692, online_share=0.271, online_per_repeat=0.518s, amortized=1.838s, amortized_margin_vs_dense=-0.461s
- DENSE_Q16: train=12000, q=16, top1=0.049125, cand_count=12000.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.985, online_per_repeat=1.322s, amortized=1.322s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q16: train=12000, q=16, top1=0.048667, cand_count=2283.812, cand_frac=0.190318, work_ratio_vs_dense=0.190, offline_share=0.608, online_share=0.358, online_per_repeat=0.384s, amortized=1.036s, amortized_margin_vs_dense=0.286s
- H4XH4_FIELD_A150_Q16: train=12000, q=16, top1=0.049125, cand_count=3776.986, cand_frac=0.314749, work_ratio_vs_dense=0.315, offline_share=0.548, online_share=0.422, online_per_repeat=0.506s, amortized=1.163s, amortized_margin_vs_dense=0.159s
- DENSE_Q24: train=12000, q=24, top1=0.049125, cand_count=12000.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.989, online_per_repeat=1.328s, amortized=1.328s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q24: train=12000, q=24, top1=0.048667, cand_count=2283.812, cand_frac=0.190318, work_ratio_vs_dense=0.190, offline_share=0.509, online_share=0.462, online_per_repeat=0.395s, amortized=0.831s, amortized_margin_vs_dense=0.497s
- H4XH4_FIELD_A150_Q24: train=12000, q=24, top1=0.049125, cand_count=3776.986, cand_frac=0.314749, work_ratio_vs_dense=0.315, offline_share=0.447, online_share=0.529, online_per_repeat=0.505s, amortized=0.931s, amortized_margin_vs_dense=0.397s

