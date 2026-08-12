# Translated Cost Accounting Audit

## Source
- experiment: `inc0104_product_phase_soft_sparse_translation_backfill_recovery_confirm + inc0105_product_phase_soft_sparse_translation_upper_bank_carry_forward_confirm`
- config: `configs/proxy_transfer_inc0104_product_phase_soft_sparse_translation_backfill_recovery_confirm.json + configs/proxy_transfer_inc0105_product_phase_soft_sparse_translation_upper_bank_carry_forward_confirm.json`
- feature_dim: `128`
- bytes_per_float: `4`
- source_experiment: `inc0104_product_phase_soft_sparse_translation_backfill_recovery_confirm`
- source_experiment: `inc0105_product_phase_soft_sparse_translation_upper_bank_carry_forward_confirm`

## Pair Audits
- train=2500, q=1: dense=DENSE_Q01_T2500, routed=CHART_H4XH4_FIELD_A150_CPX8_Q01_T2500, crossover=False
  top1_delta=-0.007400, cand_frac=0.193328, search_ratio=0.193328, bytes_saved=0.806672
  offline_penalty_per_repeat=0.036s, online_gain_per_repeat=0.020s, search_gain_per_repeat=0.035s, route_lookup_penalty_per_repeat=0.016s, amortized_margin=-0.016s
  The offline penalty of 0.036s per repeat still exceeds the online savings of 0.020s.
- train=2500, q=1: dense=DENSE_Q01_T2500, routed=CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500, crossover=True
  top1_delta=-0.007600, cand_frac=0.189366, search_ratio=0.189366, bytes_saved=0.810634
  offline_penalty_per_repeat=0.026s, online_gain_per_repeat=0.046s, search_gain_per_repeat=0.062s, route_lookup_penalty_per_repeat=0.016s, amortized_margin=0.020s
  Online savings of 0.046s per repeat outpace the offline penalty of 0.026s.
- train=2500, q=1: dense=DENSE_Q01_T2500, routed=CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T2500, crossover=False
  top1_delta=-0.007400, cand_frac=0.193328, search_ratio=0.193328, bytes_saved=0.806672
  offline_penalty_per_repeat=0.032s, online_gain_per_repeat=0.005s, search_gain_per_repeat=0.025s, route_lookup_penalty_per_repeat=0.020s, amortized_margin=-0.027s
  The offline penalty of 0.032s per repeat still exceeds the online savings of 0.005s.
- train=40000, q=1: dense=DENSE_Q01_T40000, routed=CHART_H4XH4_FIELD_A150_CPX8_Q01_T40000, crossover=True
  top1_delta=-0.001525, cand_frac=0.183764, search_ratio=0.183764, bytes_saved=0.816236
  offline_penalty_per_repeat=0.319s, online_gain_per_repeat=8.404s, search_gain_per_repeat=8.566s, route_lookup_penalty_per_repeat=0.162s, amortized_margin=8.085s
  Online savings of 8.404s per repeat outpace the offline penalty of 0.319s.
- train=40000, q=1: dense=DENSE_Q01_T40000, routed=CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T40000, crossover=True
  top1_delta=-0.001563, cand_frac=0.182003, search_ratio=0.182003, bytes_saved=0.817997
  offline_penalty_per_repeat=0.353s, online_gain_per_repeat=8.644s, search_gain_per_repeat=8.815s, route_lookup_penalty_per_repeat=0.171s, amortized_margin=8.291s
  Online savings of 8.644s per repeat outpace the offline penalty of 0.353s.
- train=40000, q=1: dense=DENSE_Q01_T40000, routed=CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000, crossover=True
  top1_delta=-0.001525, cand_frac=0.183764, search_ratio=0.183764, bytes_saved=0.816236
  offline_penalty_per_repeat=0.307s, online_gain_per_repeat=8.535s, search_gain_per_repeat=8.788s, route_lookup_penalty_per_repeat=0.253s, amortized_margin=8.227s
  Online savings of 8.535s per repeat outpace the offline penalty of 0.307s.

## Comparison Audits
- `lower_continuous_vs_soft`: baseline=`CHART_H4XH4_FIELD_A150_CPX8_Q01_T2500`, candidate=`CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T2500`
  top1_delta=0.000000, cand_frac_delta=0.000000, offline_penalty=-0.004s, online_penalty=0.015s, amortized_delta=0.011s
  route_index_build_penalty=-0.003s, route_query_penalty=0.005s, retrieval_search_penalty=0.011s, residual_penalty=-0.001s, dominant=retrieval_search
  CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T2500 relative to CHART_H4XH4_FIELD_A150_CPX8_Q01_T2500: route_index_build=-0.003s, route_query=0.005s, retrieval_search=0.011s, residual=-0.001s, amortized_delta=0.011s.
- `lower_soft_vs_backfill`: baseline=`CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T2500`, candidate=`CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500`
  top1_delta=-0.000200, cand_frac_delta=-0.003963, offline_penalty=-0.006s, online_penalty=-0.041s, amortized_delta=-0.047s
  route_index_build_penalty=-0.006s, route_query_penalty=-0.004s, retrieval_search_penalty=-0.037s, residual_penalty=-0.000s, dominant=residual
  CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500 relative to CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T2500: route_index_build=-0.006s, route_query=-0.004s, retrieval_search=-0.037s, residual=-0.000s, amortized_delta=-0.047s.
- `lower_continuous_vs_backfill`: baseline=`CHART_H4XH4_FIELD_A150_CPX8_Q01_T2500`, candidate=`CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500`
  top1_delta=-0.000200, cand_frac_delta=-0.003963, offline_penalty=-0.010s, online_penalty=-0.026s, amortized_delta=-0.036s
  route_index_build_penalty=-0.009s, route_query_penalty=0.000s, retrieval_search_penalty=-0.026s, residual_penalty=-0.001s, dominant=route_query
  CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500 relative to CHART_H4XH4_FIELD_A150_CPX8_Q01_T2500: route_index_build=-0.009s, route_query=0.000s, retrieval_search=-0.026s, residual=-0.001s, amortized_delta=-0.036s.
- `upper_continuous_vs_soft`: baseline=`CHART_H4XH4_FIELD_A150_CPX8_Q01_T40000`, candidate=`CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000`
  top1_delta=0.000000, cand_frac_delta=0.000000, offline_penalty=-0.012s, online_penalty=-0.131s, amortized_delta=-0.143s
  route_index_build_penalty=-0.011s, route_query_penalty=0.091s, retrieval_search_penalty=-0.222s, residual_penalty=-0.001s, dominant=route_query
  CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000 relative to CHART_H4XH4_FIELD_A150_CPX8_Q01_T40000: route_index_build=-0.011s, route_query=0.091s, retrieval_search=-0.222s, residual=-0.001s, amortized_delta=-0.143s.
- `upper_soft_vs_backfill`: baseline=`CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000`, candidate=`CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T40000`
  top1_delta=-0.000038, cand_frac_delta=-0.001761, offline_penalty=0.046s, online_penalty=-0.109s, amortized_delta=-0.063s
  route_index_build_penalty=0.046s, route_query_penalty=-0.082s, retrieval_search_penalty=-0.028s, residual_penalty=-0.000s, dominant=route_index_build
  CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T40000 relative to CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000: route_index_build=0.046s, route_query=-0.082s, retrieval_search=-0.028s, residual=-0.000s, amortized_delta=-0.063s.
- `upper_continuous_vs_backfill`: baseline=`CHART_H4XH4_FIELD_A150_CPX8_Q01_T40000`, candidate=`CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T40000`
  top1_delta=-0.000038, cand_frac_delta=-0.001761, offline_penalty=0.034s, online_penalty=-0.240s, amortized_delta=-0.206s
  route_index_build_penalty=0.035s, route_query_penalty=0.009s, retrieval_search_penalty=-0.249s, residual_penalty=-0.001s, dominant=route_index_build
  CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T40000 relative to CHART_H4XH4_FIELD_A150_CPX8_Q01_T40000: route_index_build=0.035s, route_query=0.009s, retrieval_search=-0.249s, residual=-0.001s, amortized_delta=-0.206s.

