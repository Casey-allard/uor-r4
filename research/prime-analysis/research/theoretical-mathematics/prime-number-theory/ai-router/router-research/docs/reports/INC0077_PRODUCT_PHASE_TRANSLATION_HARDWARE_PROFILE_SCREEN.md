# Translated Hardware Profile

## Source Experiments
- `inc0076_product_phase_translation_break_even_confirm`
- `inc0077_product_phase_translation_hardware_profile_screen`

## Group Summaries
- train=6000, q=16: dense=DENSE_Q16_T6000, crossover=False, quality=n/a, systems=n/a
- train=6000, q=24: dense=DENSE_Q24_T6000, crossover=True, quality=H4XH4_FIELD_A150_CPX8_Q24_T6000, systems=H4XH4_FIELD_A150_CPX8_Q24_T6000
  quality_delta_top1=-0.002000, quality_amortized_margin=0.043s
  systems_amortized_margin=0.043s, systems_search_ratio=0.190, systems_cand_frac=0.190455
- train=12000, q=8: dense=DENSE_Q08, crossover=False, quality=n/a, systems=n/a
- train=12000, q=16: dense=DENSE_Q16, crossover=True, quality=H4XH4_FIELD_A150_Q16, systems=H4XH4_FIELD_A150_CPX8_Q16
  quality_delta_top1=0.000000, quality_amortized_margin=0.159s
  systems_amortized_margin=0.286s, systems_search_ratio=0.190, systems_cand_frac=0.190318
- train=12000, q=24: dense=DENSE_Q24, crossover=True, quality=H4XH4_FIELD_A150_Q24, systems=H4XH4_FIELD_A150_CPX8_Q24
  quality_delta_top1=0.000000, quality_amortized_margin=0.397s
  systems_amortized_margin=0.497s, systems_search_ratio=0.190, systems_cand_frac=0.190318

## Route Profiles
- DENSE_Q16_T6000: train=6000, q=16, top1=0.049667, cand_count=6000.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.960, online_per_repeat=0.462s, amortized=0.462s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q16_T6000: train=6000, q=16, top1=0.047667, cand_count=1142.733, cand_frac=0.190455, work_ratio_vs_dense=0.190, offline_share=0.661, online_share=0.285, online_per_repeat=0.145s, amortized=0.481s, amortized_margin_vs_dense=-0.019s
- H4XH4_FIELD_A150_Q16_T6000: train=6000, q=16, top1=0.045667, cand_count=1888.372, cand_frac=0.314729, work_ratio_vs_dense=0.315, offline_share=0.648, online_share=0.300, online_per_repeat=0.164s, amortized=0.519s, amortized_margin_vs_dense=-0.057s
- DENSE_Q24_T6000: train=6000, q=24, top1=0.049667, cand_count=6000.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.968, online_per_repeat=0.388s, amortized=0.388s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q24_T6000: train=6000, q=24, top1=0.047667, cand_count=1142.733, cand_frac=0.190455, work_ratio_vs_dense=0.190, offline_share=0.580, online_share=0.370, online_per_repeat=0.134s, amortized=0.345s, amortized_margin_vs_dense=0.043s
- H4XH4_FIELD_A150_Q24_T6000: train=6000, q=24, top1=0.045667, cand_count=1888.372, cand_frac=0.314729, work_ratio_vs_dense=0.315, offline_share=0.526, online_share=0.428, online_per_repeat=0.170s, amortized=0.378s, amortized_margin_vs_dense=0.010s
- DENSE_Q08: train=12000, q=8, top1=0.049125, cand_count=12000.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.970, online_per_repeat=1.377s, amortized=1.377s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q08: train=12000, q=8, top1=0.048667, cand_count=2283.812, cand_frac=0.190318, work_ratio_vs_dense=0.190, offline_share=0.714, online_share=0.245, online_per_repeat=0.431s, amortized=1.691s, amortized_margin_vs_dense=-0.314s
- H4XH4_FIELD_A150_Q08: train=12000, q=8, top1=0.049125, cand_count=3776.986, cand_frac=0.314749, work_ratio_vs_dense=0.315, offline_share=0.692, online_share=0.271, online_per_repeat=0.518s, amortized=1.838s, amortized_margin_vs_dense=-0.461s
- DENSE_Q16: train=12000, q=16, top1=0.049125, cand_count=12000.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.985, online_per_repeat=1.322s, amortized=1.322s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q16: train=12000, q=16, top1=0.048667, cand_count=2283.812, cand_frac=0.190318, work_ratio_vs_dense=0.190, offline_share=0.608, online_share=0.358, online_per_repeat=0.384s, amortized=1.036s, amortized_margin_vs_dense=0.286s
- H4XH4_FIELD_A150_Q16: train=12000, q=16, top1=0.049125, cand_count=3776.986, cand_frac=0.314749, work_ratio_vs_dense=0.315, offline_share=0.548, online_share=0.422, online_per_repeat=0.506s, amortized=1.163s, amortized_margin_vs_dense=0.159s
- DENSE_Q24: train=12000, q=24, top1=0.049125, cand_count=12000.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.989, online_per_repeat=1.328s, amortized=1.328s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q24: train=12000, q=24, top1=0.048667, cand_count=2283.812, cand_frac=0.190318, work_ratio_vs_dense=0.190, offline_share=0.509, online_share=0.462, online_per_repeat=0.395s, amortized=0.831s, amortized_margin_vs_dense=0.497s
- H4XH4_FIELD_A150_Q24: train=12000, q=24, top1=0.049125, cand_count=3776.986, cand_frac=0.314749, work_ratio_vs_dense=0.315, offline_share=0.447, online_share=0.529, online_per_repeat=0.505s, amortized=0.931s, amortized_margin_vs_dense=0.397s

