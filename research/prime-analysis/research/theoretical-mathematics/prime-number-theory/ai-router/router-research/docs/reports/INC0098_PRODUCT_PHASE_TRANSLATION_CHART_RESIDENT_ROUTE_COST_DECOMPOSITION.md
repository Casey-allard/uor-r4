# Translated Cost Accounting Audit

## Source
- experiment: `inc0098_product_phase_translation_chart_resident_route_cost_decomposition_input`
- config: `derived from inc0096 focused confirm + inc0094 confirm + inc0093 warm confirm`
- feature_dim: `128`
- bytes_per_float: `4`
- source_experiment: `inc0098_product_phase_translation_chart_resident_route_cost_decomposition_input`

## Pair Audits
- train=2500, q=1: dense=DENSE_Q01_T2500, routed=CHART_H4XH4_FIELD_A150_CPX8_Q01_T2500, crossover=True
  top1_delta=-0.004000, cand_frac=0.198723, search_ratio=0.198723, bytes_saved=0.801277
  offline_penalty_per_repeat=0.021s, online_gain_per_repeat=0.055s, search_gain_per_repeat=0.065s, route_lookup_penalty_per_repeat=0.010s, amortized_margin=0.035s
  Online savings of 0.055s per repeat outpace the offline penalty of 0.021s.
- train=2500, q=1: dense=DENSE_Q01_T2500, routed=FULL_H4XH4_FIELD_A150_CPX8_Q01_T2500, crossover=True
  top1_delta=-0.005700, cand_frac=0.193328, search_ratio=0.193328, bytes_saved=0.806672
  offline_penalty_per_repeat=0.007s, online_gain_per_repeat=0.066s, search_gain_per_repeat=0.075s, route_lookup_penalty_per_repeat=0.008s, amortized_margin=0.059s
  Online savings of 0.066s per repeat outpace the offline penalty of 0.007s.
- train=40000, q=1: dense=DENSE_Q01_T40000, routed=CHART_H4XH4_FIELD_A150_CPX8_Q01_T40000, crossover=True
  top1_delta=-0.001525, cand_frac=0.183764, search_ratio=0.183764, bytes_saved=0.816236
  offline_penalty_per_repeat=0.249s, online_gain_per_repeat=7.008s, search_gain_per_repeat=7.130s, route_lookup_penalty_per_repeat=0.122s, amortized_margin=6.759s
  Online savings of 7.008s per repeat outpace the offline penalty of 0.249s.
- train=40000, q=1: dense=DENSE_Q01_T40000, routed=FULL_H4XH4_FIELD_A150_CPX8_Q01_T40000, crossover=True
  top1_delta=-0.001525, cand_frac=0.183764, search_ratio=0.183764, bytes_saved=0.816236
  offline_penalty_per_repeat=0.044s, online_gain_per_repeat=7.280s, search_gain_per_repeat=7.405s, route_lookup_penalty_per_repeat=0.124s, amortized_margin=7.236s
  Online savings of 7.280s per repeat outpace the offline penalty of 0.044s.

## Comparison Audits
- `chart_vs_full_lower`: baseline=`FULL_H4XH4_FIELD_A150_CPX8_Q01_T2500`, candidate=`CHART_H4XH4_FIELD_A150_CPX8_Q01_T2500`
  top1_delta=0.001700, cand_frac_delta=0.005395, offline_penalty=0.013s, online_penalty=0.011s, amortized_delta=0.025s
  route_index_build_penalty=0.014s, route_query_penalty=0.001s, retrieval_search_penalty=0.010s, residual_penalty=-0.000s, dominant=route_index_build
  CHART_H4XH4_FIELD_A150_CPX8_Q01_T2500 relative to FULL_H4XH4_FIELD_A150_CPX8_Q01_T2500: route_index_build=0.014s, route_query=0.001s, retrieval_search=0.010s, residual=-0.000s, amortized_delta=0.025s.
- `chart_vs_full_upper`: baseline=`FULL_H4XH4_FIELD_A150_CPX8_Q01_T40000`, candidate=`CHART_H4XH4_FIELD_A150_CPX8_Q01_T40000`
  top1_delta=0.000000, cand_frac_delta=0.000000, offline_penalty=0.204s, online_penalty=0.272s, amortized_delta=0.477s
  route_index_build_penalty=0.204s, route_query_penalty=-0.002s, retrieval_search_penalty=0.275s, residual_penalty=-0.000s, dominant=retrieval_search
  CHART_H4XH4_FIELD_A150_CPX8_Q01_T40000 relative to FULL_H4XH4_FIELD_A150_CPX8_Q01_T40000: route_index_build=0.204s, route_query=-0.002s, retrieval_search=0.275s, residual=-0.000s, amortized_delta=0.477s.

