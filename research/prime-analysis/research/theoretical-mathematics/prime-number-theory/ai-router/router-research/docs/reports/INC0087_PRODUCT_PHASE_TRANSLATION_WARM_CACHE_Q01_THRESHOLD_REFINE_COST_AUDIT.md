# Translated Cost Accounting Audit

## Source
- experiment: `inc0087_product_phase_translation_warm_cache_q01_threshold_refine_confirm`
- config: `configs/proxy_transfer_inc0087_product_phase_translation_warm_cache_q01_threshold_refine_confirm.json`
- feature_dim: `128`
- bytes_per_float: `4`

## Pair Audits
- train=2600, q=1: dense=DENSE_Q01_T2600, routed=H4XH4_FIELD_A150_CPX8_Q01_T2600, crossover=True
  top1_delta=-0.006346, cand_frac=0.193587, search_ratio=0.193587, bytes_saved=0.806413
  offline_penalty_per_repeat=0.005s, online_gain_per_repeat=0.007s, search_gain_per_repeat=0.026s, route_lookup_penalty_per_repeat=0.019s, amortized_margin=0.002s
  Online savings of 0.007s per repeat outpace the offline penalty of 0.005s.
- train=2650, q=1: dense=DENSE_Q01_T2650, routed=H4XH4_FIELD_A150_CPX8_Q01_T2650, crossover=False
  top1_delta=-0.006604, cand_frac=0.193916, search_ratio=0.193916, bytes_saved=0.806084
  offline_penalty_per_repeat=0.005s, online_gain_per_repeat=-0.012s, search_gain_per_repeat=0.007s, route_lookup_penalty_per_repeat=0.019s, amortized_margin=-0.017s
  The offline penalty of 0.005s per repeat still exceeds the online savings of -0.012s.
- train=2700, q=1: dense=DENSE_Q01_T2700, routed=H4XH4_FIELD_A150_CPX8_Q01_T2700, crossover=True
  top1_delta=-0.007593, cand_frac=0.193139, search_ratio=0.193139, bytes_saved=0.806861
  offline_penalty_per_repeat=0.005s, online_gain_per_repeat=0.042s, search_gain_per_repeat=0.061s, route_lookup_penalty_per_repeat=0.019s, amortized_margin=0.037s
  Online savings of 0.042s per repeat outpace the offline penalty of 0.005s.

