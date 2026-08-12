# Translated Hardware Profile

## Source Experiments
- `inc0080_product_phase_translation_second_large_bank_boundary_extension_confirm`

## Group Summaries
- train=24000, q=4: dense=DENSE_Q04_T24000, crossover=False, quality=n/a, systems=n/a
- train=24000, q=8: dense=DENSE_Q08_T24000, crossover=True, quality=H4XH4_FIELD_A150_CPX8_Q08_T24000, systems=H4XH4_FIELD_A150_CPX8_Q08_T24000
  quality_delta_top1=-0.000521, quality_amortized_margin=1.624s
  systems_amortized_margin=1.624s, systems_search_ratio=0.193, systems_cand_frac=0.192929
- train=30000, q=4: dense=DENSE_Q04_T30000, crossover=False, quality=n/a, systems=n/a
- train=30000, q=8: dense=DENSE_Q08_T30000, crossover=True, quality=H4XH4_FIELD_A150_CPX8_Q08_T30000, systems=H4XH4_FIELD_A150_CPX8_Q08_T30000
  quality_delta_top1=-0.000850, quality_amortized_margin=2.165s
  systems_amortized_margin=2.165s, systems_search_ratio=0.189, systems_cand_frac=0.188863

## Bank Summaries
- train=24000: first_any=8, first_quality=8, first_systems=8, systems_family=H4XH4_FIELD_A150_CPX8
  systems_ratio_min=0.193, systems_ratio_max=0.193, systems_ratio_span=0.000, systems_margin_slope=0.974123
- train=30000: first_any=8, first_quality=8, first_systems=8, systems_family=H4XH4_FIELD_A150_CPX8
  systems_ratio_min=0.189, systems_ratio_max=0.189, systems_ratio_span=0.000, systems_margin_slope=0.802680

## Route Profiles
- DENSE_Q04_T24000: train=24000, q=4, top1=0.049354, cand_count=24000.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.982, online_per_repeat=5.152s, amortized=5.152s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q04_T24000: train=24000, q=4, top1=0.048833, cand_count=4630.298, cand_frac=0.192929, work_ratio_vs_dense=0.193, offline_share=0.798, online_share=0.171, online_per_repeat=1.311s, amortized=7.425s, amortized_margin_vs_dense=-2.272s
- DENSE_Q08_T24000: train=24000, q=8, top1=0.049354, cand_count=24000.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.992, online_per_repeat=5.491s, amortized=5.491s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q08_T24000: train=24000, q=8, top1=0.048833, cand_count=4630.298, cand_frac=0.192929, work_ratio_vs_dense=0.193, offline_share=0.677, online_share=0.296, online_per_repeat=1.175s, amortized=3.867s, amortized_margin_vs_dense=1.624s
- DENSE_Q04_T30000: train=30000, q=4, top1=0.048017, cand_count=30000.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.987, online_per_repeat=7.901s, amortized=7.901s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q04_T30000: train=30000, q=4, top1=0.047167, cand_count=5665.895, cand_frac=0.188863, work_ratio_vs_dense=0.189, offline_share=0.781, online_share=0.190, online_per_repeat=1.749s, amortized=8.946s, amortized_margin_vs_dense=-1.045s
- DENSE_Q08_T30000: train=30000, q=8, top1=0.048017, cand_count=30000.000, cand_frac=1.000000, work_ratio_vs_dense=1.000, offline_share=0.000, online_share=0.994, online_per_repeat=7.560s, amortized=7.560s, amortized_margin_vs_dense=0.000s
- H4XH4_FIELD_A150_CPX8_Q08_T30000: train=30000, q=8, top1=0.047167, cand_count=5665.895, cand_frac=0.188863, work_ratio_vs_dense=0.189, offline_share=0.623, online_share=0.350, online_per_repeat=1.941s, amortized=5.394s, amortized_margin_vs_dense=2.165s

