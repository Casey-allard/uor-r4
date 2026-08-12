# Translated Component Stability Audit

## Source
- experiment: `inc0104_product_phase_soft_sparse_translation_backfill_recovery_confirm + inc0105_product_phase_soft_sparse_translation_upper_bank_carry_forward_confirm`
- config: `configs/proxy_transfer_inc0104_product_phase_soft_sparse_translation_backfill_recovery_confirm.json + configs/proxy_transfer_inc0105_product_phase_soft_sparse_translation_upper_bank_carry_forward_confirm.json`
- source_experiment: `inc0104_product_phase_soft_sparse_translation_backfill_recovery_confirm`
- source_experiment: `inc0105_product_phase_soft_sparse_translation_upper_bank_carry_forward_confirm`

## Comparisons
- `lower_soft_vs_backfill`: baseline=`CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T2500`, candidate=`CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500`, seeds=4
  lower_soft_vs_backfill: 2 seed wins, 2 seed losses on amortized cost. Amortized behavior is seed-fragile rather than sign-stable. Route-query deltas change sign across seeds, so they are not safe as optimization guidance yet. Retrieval-search savings dominate on average, but the sign is not fully stable.
  amortized: mean=-0.046992, min=-0.119572, max=0.003719, std=0.052783, sign=mixed
  route_index: mean=-0.005677, min=-0.024216, max=0.010162, std=0.012271, sign=mixed
  route_query: mean=-0.004347, min=-0.022598, max=0.007766, std=0.012592, sign=mixed
  retrieval_search: mean=-0.036931, min=-0.075958, max=0.001873, std=0.037457, sign=mixed
  cand_frac: mean=-0.003963, min=-0.008740, max=-0.000792, std=0.003140, sign=stable_improvement
  top1: mean=-0.000200, min=-0.001600, max=0.000800, std=0.000872, sign=mixed
  Seed rows:
    seed=0: amort=-0.075208, route_index=0.010162, route_query=-0.009510, search=-0.075958, cand_frac=-0.004784, top1=0.000800
    seed=1: amort=0.003719, route_index=-0.005753, route_query=0.007766, search=0.001873, cand_frac=-0.000792, top1=0.000000
    seed=2: amort=0.003092, route_index=-0.002902, route_query=0.006954, search=-0.000882, cand_frac=-0.008740, top1=-0.001600
    seed=3: amort=-0.119572, route_index=-0.024216, route_query=-0.022598, search=-0.072757, cand_frac=-0.001536, top1=0.000000

- `lower_continuous_vs_backfill`: baseline=`CHART_H4XH4_FIELD_A150_CPX8_Q01_T2500`, candidate=`CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500`, seeds=4
  lower_continuous_vs_backfill: 4 seed wins, 0 seed losses on amortized cost. Amortized behavior is sign-stable improvement across seeds. Route-query deltas change sign across seeds, so they are not safe as optimization guidance yet. Retrieval-search deltas are consistently favorable across seeds.
  amortized: mean=-0.035771, min=-0.111994, max=-0.002455, std=0.044285, sign=stable_improvement
  route_index: mean=-0.008892, min=-0.018258, max=0.005313, std=0.008677, sign=mixed
  route_query: mean=0.000234, min=-0.023354, max=0.014479, std=0.014321, sign=mixed
  retrieval_search: mean=-0.026382, min=-0.076483, max=-0.001736, std=0.029888, sign=stable_improvement
  cand_frac: mean=-0.003963, min=-0.008740, max=-0.000792, std=0.003140, sign=stable_improvement
  top1: mean=-0.000200, min=-0.001600, max=0.000800, std=0.000872, sign=mixed
  Seed rows:
    seed=0: amort=-0.002455, route_index=0.005313, route_query=0.014479, search=-0.021773, cand_frac=-0.004784, top1=0.000800
    seed=1: amort=-0.015797, route_index=-0.011292, route_query=0.001958, search=-0.005537, cand_frac=-0.000792, top1=0.000000
    seed=2: amort=-0.012838, route_index=-0.018258, route_query=0.007853, search=-0.001736, cand_frac=-0.008740, top1=-0.001600
    seed=3: amort=-0.111994, route_index=-0.011330, route_query=-0.023354, search=-0.076483, cand_frac=-0.001536, top1=0.000000

- `upper_soft_vs_backfill`: baseline=`CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000`, candidate=`CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T40000`, seeds=4
  upper_soft_vs_backfill: 3 seed wins, 1 seed losses on amortized cost. Amortized behavior is seed-fragile rather than sign-stable. Route-query deltas change sign across seeds, so they are not safe as optimization guidance yet. Retrieval-search savings dominate on average, but the sign is not fully stable. Route-index build remains a cost penalty on average.
  amortized: mean=-0.063274, min=-1.065731, max=1.334614, std=0.871795, sign=mixed
  route_index: mean=0.046031, min=-0.043424, max=0.161788, std=0.073917, sign=mixed
  route_query: mean=-0.081686, min=-0.296253, max=0.024283, std=0.126906, sign=mixed
  retrieval_search: mean=-0.027532, min=-0.969817, max=1.148578, std=0.768391, sign=mixed
  cand_frac: mean=-0.001761, min=-0.002898, max=-0.001098, std=0.000697, sign=stable_improvement
  top1: mean=-0.000037, min=-0.000200, max=0.000050, std=0.000096, sign=mixed
  Seed rows:
    seed=0: amort=1.334614, route_index=0.161788, route_query=0.024283, search=1.148578, cand_frac=-0.001748, top1=0.000000
    seed=1: amort=-0.222823, route_index=0.040202, route_query=-0.296253, search=0.033353, cand_frac=-0.001300, top1=0.000050
    seed=2: amort=-0.299157, route_index=0.025556, route_query=-0.002334, search=-0.322243, cand_frac=-0.001098, top1=0.000000
    seed=3: amort=-1.065731, route_index=-0.043424, route_query=-0.052438, search=-0.969817, cand_frac=-0.002898, top1=-0.000200

- `upper_continuous_vs_backfill`: baseline=`CHART_H4XH4_FIELD_A150_CPX8_Q01_T40000`, candidate=`CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T40000`, seeds=4
  upper_continuous_vs_backfill: 3 seed wins, 1 seed losses on amortized cost. Amortized behavior is seed-fragile rather than sign-stable. Route-query deltas change sign across seeds, so they are not safe as optimization guidance yet. Retrieval-search savings dominate on average, but the sign is not fully stable. Route-index build remains a cost penalty on average.
  amortized: mean=-0.205897, min=-0.775950, max=0.974804, std=0.693023, sign=mixed
  route_index: mean=0.035328, min=-0.118343, max=0.137861, std=0.103473, sign=mixed
  route_query: mean=0.009207, min=-0.066319, max=0.090800, std=0.055930, sign=mixed
  retrieval_search: mean=-0.249462, min=-0.811165, max=0.823734, std=0.648265, sign=mixed
  cand_frac: mean=-0.001761, min=-0.002898, max=-0.001098, std=0.000697, sign=stable_improvement
  top1: mean=-0.000037, min=-0.000200, max=0.000050, std=0.000096, sign=mixed
  Seed rows:
    seed=0: amort=0.974804, route_index=0.137861, route_query=0.014316, search=0.823734, cand_frac=-0.001748, top1=0.000000
    seed=1: amort=-0.599717, route_index=0.121689, route_query=0.090800, search=-0.811165, cand_frac=-0.001300, top1=0.000050
    seed=2: amort=-0.422724, route_index=-0.118343, route_query=-0.001967, search=-0.301593, cand_frac=-0.001098, top1=0.000000
    seed=3: amort=-0.775950, route_index=0.000105, route_query=-0.066319, search=-0.708823, cand_frac=-0.002898, top1=-0.000200
