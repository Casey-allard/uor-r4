# Translated Cost Accounting Audit

## Source
- experiment: `inc0081_product_phase_translation_q04_threshold_search_confirm`
- config: `configs/proxy_transfer_inc0081_product_phase_translation_q04_threshold_search_confirm.json`
- feature_dim: `128`
- bytes_per_float: `4`

## Pair Audits
- train=36000, q=4: dense=DENSE_Q04_T36000, routed=H4XH4_FIELD_A150_CPX8_Q04_T36000, crossover=True
  top1_delta=-0.000833, cand_frac=0.190206, search_ratio=0.190206, bytes_saved=0.809794
  offline_penalty_per_repeat=7.407s, online_gain_per_repeat=9.862s, search_gain_per_repeat=9.893s, route_lookup_penalty_per_repeat=0.031s, amortized_margin=2.455s
  Online savings of 9.862s per repeat outpace the offline penalty of 7.407s.
- train=40000, q=4: dense=DENSE_Q04_T40000, routed=H4XH4_FIELD_A150_CPX8_Q04_T40000, crossover=False
  top1_delta=-0.001525, cand_frac=0.183764, search_ratio=0.183764, bytes_saved=0.816236
  offline_penalty_per_repeat=8.038s, online_gain_per_repeat=7.072s, search_gain_per_repeat=7.104s, route_lookup_penalty_per_repeat=0.031s, amortized_margin=-0.966s
  The offline penalty of 8.038s per repeat still exceeds the online savings of 7.072s.
- train=40000, q=8: dense=DENSE_Q08_T40000, routed=H4XH4_FIELD_A150_CPX8_Q08_T40000, crossover=True
  top1_delta=-0.001525, cand_frac=0.183764, search_ratio=0.183764, bytes_saved=0.816236
  offline_penalty_per_repeat=4.121s, online_gain_per_repeat=7.657s, search_gain_per_repeat=7.673s, route_lookup_penalty_per_repeat=0.015s, amortized_margin=3.536s
  Online savings of 7.657s per repeat outpace the offline penalty of 4.121s.

## Threshold Finding
- `Q04` crossing route: `H4XH4_FIELD_A150_CPX8_Q04_T36000` (train=36000, margin=2.455s)
- `Q04` miss route: `H4XH4_FIELD_A150_CPX8_Q04_T40000` (train=40000, margin=-0.966s)
- Q04 crosses at train=36000 because online savings (9.862s/repeat) exceed the offline penalty (7.407s/repeat), but misses at train=40000 because the offline penalty (8.038s/repeat) stays larger than the online savings (7.072s/repeat).

