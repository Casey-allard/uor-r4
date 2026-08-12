# Real Task Comparison: LM Proxy

## Objective
Measure hardware-efficiency impact of geometric routing vs dense baseline.

## Metrics
- quality proxy (`mse`, optional top-1)
- runtime (`train_sec`, `eval_sec`, `total_sec`)
- memory estimate (`model_bytes`, `activation_bytes_est`)

## Current Status
- Proxy pipeline implemented and executed:
  - token prep: `tools/prepare_wikitext2.py`
  - tensorization: `tasks/wikitext2_proxy.py`
  - dense comparator: `tasks/dense_baseline.py`
  - route evaluator: `tasks/router_proxy_eval.py`
- `--dataset auto` currently resolves to PTB in this environment.

## Latest Sparse-Event Dual-Anchor Refreshed Comparison
- Refreshed comparison artifacts:
  - `results/analysis/inc0134_product_phase_sparse_event_translation_dual_anchor_real_task_refresh_comparison.json`
  - `docs/reports/INC0134_PRODUCT_PHASE_SPARSE_EVENT_TRANSLATION_DUAL_ANCHOR_REAL_TASK_REFRESH_COMPARISON.md`
- Current default lower-bank routed route:
  - `CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_CPX8_Q01_T2500`
  - read: `systems-only`
- Current lower-bank explicit comparators:
  - balanced quality comparator:
    `CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_SBI030_CPX8_Q01_T2500`
  - quality-first comparator:
    `CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_SBI080_CPX8_Q01_T2500`
- Current upper-bank routed default:
  - `CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000`
  - read: `quality-near systems promotion`
- Current upper-bank optional comparator:
  - `CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T40000`
- Historical-only lower-bank comparator:
  - `CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500`
- Current reading:
  - `TAU002` remains the fastest lower-bank systems point
  - `SBI030` remains the balanced lower-bank quality lift
  - `SBI080` is now the first lower-bank route to edge dense on top-1, but it
    gives back the lower-bank systems advantage
  - the next honest branch is the lower-bank quality/systems frontier, not
    another inheritance refresh

## Latest Baseline Snapshot
- Artifact: `results/raw/dense_baseline_lm_proxy.json`
- Metrics:
  - `val_mse`: `0.003853`
  - `test_mse`: `0.003851`
  - `val_top1`: `0.0719`
  - `test_top1`: `0.0737`
- Runtime:
  - `train`: `0.1191s`
  - `eval`: `0.0992s`
  - `total`: `0.2183s`
- Memory:
  - `weights`: `262144 bytes`
  - `train arrays`: `122880000 bytes`
  - `eval arrays`: `245760000 bytes`

## Data Source Note
- WikiText endpoints were unavailable in this environment.
- Source resolution fallback selected PTB successfully.
- Proven fallback chain for `--dataset auto`: `wikitext2 -> ptb -> tinyshakespeare -> synthetic`.

## First Route Transfer Smoke
- Artifacts:
  - `results/parsed/proxy_cmp_r0.json`
  - `results/parsed/proxy_cmp_r5b.json`
- Comparison on PTB proxy subset (`train=2000`, `test=1000`, `fast_dev=1`, `seed=0`)
  - `R0`:
    - `test_mse_after=0.0039455`
    - `total_sec=128.599`
    - `buckets=8`
    - `pmax_after=0.223`
  - `R5B`:
    - `test_mse_after=0.0039146`
    - `total_sec=127.130`
    - `buckets=2`
    - `pmax_after=0.875`

## Transfer Reading
- The first smoke favors `R5B` very slightly on quality and total runtime.
- The route collapses much more aggressively on proxy data (`2` buckets and high `pmax`), so this is not yet evidence of a healthy transfer story.

## Multi-Seed Larger-Subset Transfer
- Analysis:
  - `results/analysis/inc0008_proxy_transfer_multiseed.json`
  - `docs/governance/gates/gate_20260305_141648.md`
- Comparison on PTB proxy subset (`train=3000`, `test=1500`, `seeds=0,1`)
  - `R0` mean:
    - `test_mse_after=0.0039450`
    - `total_sec=26.112`
    - `buckets=8.0`
    - `pmax_after=0.205`
  - `R5B` mean:
    - `test_mse_after=0.0038858`
    - `total_sec=23.237`
    - `buckets=2.0`
    - `pmax_after=0.877`

## Current Transfer Reading
- `R5B` now has repeatable proxy-transfer wins over `R0` on both quality and runtime in this harness.
- The transfer mechanism is still unhealthy because the route remains near-degenerate in bucket usage.
- The dense baseline is still dramatically faster in absolute terms on this proxy harness, so this track currently supports relative router comparison, not a claim of end-to-end system efficiency.
- Next transfer step should be stabilization: widen route usage while preserving the current `R5B` edge.

## Current Large-Subset Frontier
- Increment docs:
  - `docs/research/increments/INC_0042_large_subset_ema_pressure.md`
  - `docs/research/increments/INC_0043_train_route_static.md`
- Analyses:
  - `results/analysis/inc0042_timing_diag.json`
  - `results/analysis/inc0043_train_route_static_confirm.json`
- Gate notes:
  - `docs/governance/gates/gate_20260306_094708.md`
  - `docs/governance/gates/gate_20260306_100530.md`
- Reading:
  - `INC-0042` showed that the large-subset EMA bottleneck was almost entirely per-step training rerouting, not EMA writes or growth-phase splitting
  - `INC-0043` replaced that dynamic training rerouting with final-route reuse and restored a routed quality/runtime win over `R0`
- Current 4-seed large-subset means:
  - `HOPF_PHI2_BAND_IT60_P4_STATIC`
    - `test_mse_after=0.003902306`
    - `total_sec=19.602`
    - widened healthy routed lead
  - `HOPF_K25_BASE_IT60_P4_STATIC`
    - `test_mse_after=0.003899506`
    - `total_sec=19.798`
    - quality-balanced routed lead
  - `R0`
    - `test_mse_after=0.003913707`
    - `total_sec=22.520`
    - still shell-collapsed and health-failing
- Current conclusion:
  - inside the proxy harness, the routed frontier is operationally alive again
  - the dominant remaining routed cost term is now `chart_opt`
  - the next responsible systems branch is chart-pressure on the new static frontier, not fresh geometry

## Cheap-Schedule Static Frontier
- Increment doc:
  - `docs/research/increments/INC_0044_static_chart_pressure.md`
- Analyses:
  - `results/analysis/inc0044_static_chart_pressure_screen.json`
  - `results/analysis/inc0044_static_chart_pressure_confirm.json`
- Gate notes:
  - `docs/governance/gates/gate_20260306_101427.md`
  - `docs/governance/gates/gate_20260306_102058.md`
- 4-seed confirm means:
  - `HOPF_PHI2_BAND_IT48_P3_STATIC`
    - `test_mse_after=0.003901257`
    - `total_sec=17.217`
    - health pass
  - `HOPF_K25_BASE_IT60_P4_STATIC`
    - `test_mse_after=0.003899506`
    - `total_sec=24.155`
    - runtime fail vs cheap `R0`
  - `R0`
    - `test_mse_after=0.003922779`
    - `total_sec=16.183`
    - shell-collapse fail
- Current conclusion:
  - the cheap widened static route is the healthiest quality branch under the common cheap schedule
  - it still trails cheap `R0` on absolute runtime
  - the next systems question is chart floor, not geometry

## Cheap-Schedule Routed Win
- Increment doc:
  - `docs/research/increments/INC_0045_static_chart_floor.md`
- Analyses:
  - `results/analysis/inc0045_static_chart_floor_screen.json`
  - `results/analysis/inc0045_static_chart_floor_confirm.json`
- Gate notes:
  - `docs/governance/gates/gate_20260306_103538.md`
  - `docs/governance/gates/gate_20260306_103811.md`
- 4-seed confirm means:
  - `HOPF_K25_BASE_IT40_P2_STATIC`
    - `test_mse_after=0.003895098`
    - `total_sec=6.800`
    - health pass
  - `HOPF_PHI2_BAND_IT40_P2_STATIC`
    - `test_mse_after=0.003903409`
    - `total_sec=7.176`
    - health pass
  - `R0`
    - `test_mse_after=0.003911417`
    - `total_sec=8.334`
    - shell-collapse fail
- Current conclusion:
  - the proxy harness now has a cheap routed winner over cheap `R0`
  - the next question is scale robustness, not route rescue

## Larger-Subset Cheap Routed Robustness
- Increment doc:
  - `docs/research/increments/INC_0046_static_scale_robustness.md`
- Analyses:
  - `results/analysis/inc0046_static_scale_robustness_screen.json`
  - `results/analysis/inc0046_static_scale_robustness_confirm.json`
- Gate notes:
  - `docs/governance/gates/gate_20260306_104728.md`
  - `docs/governance/gates/gate_20260306_105119.md`
- 4-seed confirm means:
  - `HOPF_K25_BASE_IT40_P2_STATIC`
    - `test_mse_after=0.003884370`
    - `total_sec=11.035`
    - health pass
  - `HOPF_PHI2_BAND_IT40_P2_STATIC`
    - `test_mse_after=0.003900404`
    - `total_sec=10.186`
    - health pass
  - `R0`
    - `test_mse_after=0.003892404`
    - `total_sec=18.872`
    - shell-collapse fail
- Current conclusion:
  - the cheap routed win held through the next larger-subset step
  - the next transfer question is near-full-proxy scale, not geometry rescue

## Translated Retrieval Screen
- Increment doc:
  - `docs/research/increments/INC_0048_integration_translation.md`
- Analysis:
  - `results/analysis/inc0048_retrieval_translation_screen.json`
- Gate:
  - `docs/governance/gates/gate_20260306_111959.md`
- Reading:
  - translated routed retrieval preserved a real candidate-pruning signal
  - `HOPF_PHI2_RET_P1` pruned to about `34.15%` of dense candidates
  - dense exact retrieval still dominated single-batch wall-clock

## Translated Retrieval Cost Rescue
- Increment doc:
  - `docs/research/increments/INC_0049_retrieval_cost_rescue.md`
- Analysis:
  - `results/analysis/inc0049_retrieval_cost_rescue_screen.json`
- Gate:
  - `docs/governance/gates/gate_20260306_113201.md`
- 2-seed screen means:
  - `DENSE`
    - `test_mse_after=0.004318726`
    - `total_sec=1.332`
    - `retrieval_offline_total_sec=0.000`
    - `retrieval_online_total_sec=0.879`
    - `retrieval_candidate_fraction_mean=1.0`
  - `HOPF_RET_P1`
    - `test_mse_after=0.004325216`
    - `total_sec=10.687`
    - `retrieval_offline_total_sec=9.694`
    - `retrieval_online_total_sec=0.401`
    - `retrieval_candidate_fraction_mean=0.348843`
  - `HOPF_PHI2_RET_P1`
    - `test_mse_after=0.004326332`
    - `total_sec=8.525`
    - `retrieval_offline_total_sec=7.664`
    - `retrieval_online_total_sec=0.299`
    - `retrieval_candidate_fraction_mean=0.341492`
- Current conclusion:
  - routed translated retrieval is now online-faster than dense exact retrieval
  - offline chart/index build dominates the total cost
  - the next systems branch is amortization / repeated-query break-even, not fresh geometry

## Product Phase Dense Frontier
- Increment docs:
  - `docs/research/increments/INC_0074_product_phase_translation_dense_frontier.md`
- Analyses:
  - `results/analysis/inc0074_product_phase_translation_dense_frontier_screen.json`
  - `results/analysis/inc0074_product_phase_translation_dense_frontier_confirm.json`
- Gate notes:
  - `docs/governance/gates/gate_20260311_140824.md`
  - `docs/governance/gates/gate_20260311_141705.md`
- 4-seed confirm means:
  - `DENSE_Q24`
    - `test_mse_after=0.004319`
    - `test_top1_after=0.049125`
    - `retrieval_candidate_fraction_mean=1.000000`
    - `retrieval_online_total_per_repeat_sec=1.339524`
    - `retrieval_total_amortized_per_repeat_sec=1.339524`
  - `H4XH4_FIELD_A150_CPX8`
    - `test_mse_after=0.004320`
    - `test_top1_after=0.048667`
    - `retrieval_candidate_fraction_mean=0.190318`
    - `retrieval_online_total_per_repeat_sec=0.390383`
    - `retrieval_total_amortized_per_repeat_sec=0.830795`
  - `H4XH4_FIELD_A150`
    - `test_mse_after=0.004320`
    - `test_top1_after=0.049125`
    - `retrieval_candidate_fraction_mean=0.314749`

## Product Phase Break-Even
- Increment doc:
  - `docs/research/increments/INC_0076_product_phase_translation_break_even.md`
- Analyses:
  - `results/analysis/inc0076_product_phase_translation_break_even_screen.json`
  - `results/analysis/inc0076_product_phase_translation_break_even_confirm.json`
- Gate notes:
  - `docs/governance/gates/gate_20260311_145722.md`
  - `docs/governance/gates/gate_20260311_151013.md`
- 4-seed confirm means:
  - `DENSE_Q16`
    - `test_top1_after=0.049125`
    - `retrieval_candidate_fraction_mean=1.000000`
    - `retrieval_total_amortized_per_repeat_sec=1.322228`
  - `H4XH4_FIELD_A150_Q16`
    - `test_top1_after=0.049125`
    - `retrieval_candidate_fraction_mean=0.314749`
    - `retrieval_total_amortized_per_repeat_sec=1.163243`
  - `H4XH4_FIELD_A150_CPX8_Q16`
    - `test_top1_after=0.048667`
    - `retrieval_candidate_fraction_mean=0.190318`
    - `retrieval_total_amortized_per_repeat_sec=1.036462`
  - `DENSE_Q24`
    - `test_top1_after=0.049125`
    - `retrieval_candidate_fraction_mean=1.000000`
    - `retrieval_total_amortized_per_repeat_sec=1.328060`
  - `H4XH4_FIELD_A150_CPX8_Q24`
    - `test_top1_after=0.048667`
    - `retrieval_candidate_fraction_mean=0.190318`
    - `retrieval_total_amortized_per_repeat_sec=0.831087`
- Reading:
  - the fixed product branch now has a confirmed amortized crossover against
    dense exact retrieval
  - `H4XH4_FIELD_A150_Q16` is the first quality-matched break-even point
  - `H4XH4_FIELD_A150_CPX8_Q16` is the first stronger-pruning systems
    crossover point
  - `H4XH4_FIELD_A150_CPX8_Q24` remains the stabilized systems point

## Product Phase Hardware Profile
- Increment doc:
  - `docs/research/increments/INC_0077_product_phase_translation_hardware_profile.md`
- Analyses:
  - `results/analysis/inc0077_product_phase_translation_hardware_profile_screen.json`
  - `results/analysis/inc0077_product_phase_translation_hardware_profile_confirm.json`
  - `results/analysis/inc0077_product_phase_translation_hardware_profile_screen_profile.json`
  - `results/analysis/inc0077_product_phase_translation_hardware_profile_confirm_profile.json`
- Reports:
  - `docs/reports/INC0077_PRODUCT_PHASE_TRANSLATION_HARDWARE_PROFILE_SCREEN.md`
  - `docs/reports/INC0077_PRODUCT_PHASE_TRANSLATION_HARDWARE_PROFILE_CONFIRM.md`
- Gate notes:
  - `docs/governance/gates/gate_20260311_152632.md`
  - `docs/governance/gates/gate_20260311_153047.md`
- 4-seed confirm means at `max_train=6000`:
  - `DENSE_Q16_T6000`
    - `test_top1_after=0.048667`
    - `retrieval_candidate_fraction_mean=1.000000`
    - `retrieval_total_amortized_per_repeat_sec=0.380635`
  - `H4XH4_FIELD_A150_Q16_T6000`
    - `test_top1_after=0.044500`
    - `retrieval_candidate_fraction_mean=0.308333`
    - `retrieval_total_amortized_per_repeat_sec=0.489746`
  - `H4XH4_FIELD_A150_CPX8_Q16_T6000`
    - `test_top1_after=0.047083`
    - `retrieval_candidate_fraction_mean=0.187229`
    - `retrieval_total_amortized_per_repeat_sec=0.464347`
  - `DENSE_Q24_T6000`
    - `test_top1_after=0.048667`
    - `retrieval_candidate_fraction_mean=1.000000`
    - `retrieval_total_amortized_per_repeat_sec=0.382233`
  - `H4XH4_FIELD_A150_Q24_T6000`
    - `test_top1_after=0.044500`
    - `retrieval_candidate_fraction_mean=0.308333`
    - `retrieval_total_amortized_per_repeat_sec=0.394226`
  - `H4XH4_FIELD_A150_CPX8_Q24_T6000`
    - `test_top1_after=0.047083`
    - `retrieval_candidate_fraction_mean=0.187229`
    - `retrieval_total_amortized_per_repeat_sec=0.350687`
- Derived confirm profile:
  - `H4XH4_FIELD_A150_CPX8_Q24_T6000`
    - `search_work_ratio_vs_dense=0.187229`
    - `amortized_margin_vs_dense=+0.031546s`
    - `offline_share=0.5895`
    - `online_share=0.3630`
  - `H4XH4_FIELD_A150_Q16`
    - `search_work_ratio_vs_dense=0.314749`
    - `amortized_margin_vs_dense=+0.158985s`
  - `H4XH4_FIELD_A150_CPX8_Q16`
    - `search_work_ratio_vs_dense=0.190318`
    - `amortized_margin_vs_dense=+0.285766s`
- Reading:
  - search-work reduction is stable across bank size:
    - secondary-key branch stays near `19%` of dense scan work
    - plain product branch stays near `31%`
  - the crossover is scale-dependent rather than dead:
    - at `max_train=12000`, crossover begins by `Q16`
    - at `max_train=6000`, crossover survives only at `Q24`
  - the first confirmed smaller-bank crossover point is
    `H4XH4_FIELD_A150_CPX8_Q24_T6000`

## Product Phase Crossover Map
- Increment doc:
  - `docs/research/increments/INC_0078_product_phase_translation_crossover_map.md`
- Analyses:
  - `results/analysis/inc0078_product_phase_translation_crossover_map_screen.json`
  - `results/analysis/inc0078_product_phase_translation_crossover_map_confirm.json`
  - `results/analysis/inc0078_product_phase_translation_crossover_map_screen_profile.json`
  - `results/analysis/inc0078_product_phase_translation_crossover_map_confirm_profile.json`
- Reports:
  - `docs/reports/INC0078_PRODUCT_PHASE_TRANSLATION_CROSSOVER_MAP_SCREEN.md`
  - `docs/reports/INC0078_PRODUCT_PHASE_TRANSLATION_CROSSOVER_MAP_CONFIRM.md`
- Gate notes:
  - `docs/governance/gates/gate_20260311_155644.md`
  - `docs/governance/gates/gate_20260311_161119.md`
- 4-seed confirm means:
  - `DENSE_Q20_T3000`
    - `test_top1_after=0.049833`
    - `retrieval_candidate_fraction_mean=1.000000`
    - `retrieval_total_amortized_per_repeat_sec=0.141692`
  - `H4XH4_FIELD_A150_CPX8_Q24_T3000`
    - `test_top1_after=0.044833`
    - `retrieval_candidate_fraction_mean=0.191704`
    - `retrieval_total_amortized_per_repeat_sec=0.170653`
  - `DENSE_Q24_T6000`
    - `test_top1_after=0.048667`
    - `retrieval_candidate_fraction_mean=1.000000`
    - `retrieval_total_amortized_per_repeat_sec=0.379925`
  - `H4XH4_FIELD_A150_CPX8_Q24_T6000`
    - `test_top1_after=0.047083`
    - `retrieval_candidate_fraction_mean=0.187229`
    - `retrieval_total_amortized_per_repeat_sec=0.363767`
  - `DENSE_Q12_T12000`
    - `test_top1_after=0.049125`
    - `retrieval_candidate_fraction_mean=1.000000`
    - `retrieval_total_amortized_per_repeat_sec=1.319593`
  - `H4XH4_FIELD_A150_CPX8_Q12_T12000`
    - `test_top1_after=0.048667`
    - `retrieval_candidate_fraction_mean=0.190318`
    - `retrieval_total_amortized_per_repeat_sec=1.245288`
- Confirmed bank map:
  - `max_train=3000`
    - no routed crossover survived through `Q24`
  - `max_train=6000`
    - first systems crossover at `Q24`
  - `max_train=12000`
    - first systems crossover already at `Q12`
- Reading:
  - the crossover boundary improves monotonically with bank size on the fixed
    translated product law
  - the secondary-key search-work ratio stays essentially flat near `0.19`
    while the onset moves earlier
  - this is the clearest software-side hardware-efficiency read so far, but it
    remains narrow because the map still stops at `12000`

## Product Phase Larger-Bank Boundary Extension
- Increment doc:
  - `docs/research/increments/INC_0079_product_phase_translation_large_bank_boundary_extension.md`
- Analyses:
  - `results/analysis/inc0079_product_phase_translation_large_bank_boundary_extension_screen.json`
  - `results/analysis/inc0079_product_phase_translation_large_bank_boundary_extension_confirm.json`
  - `results/analysis/inc0079_product_phase_translation_large_bank_boundary_extension_screen_profile.json`
  - `results/analysis/inc0079_product_phase_translation_large_bank_boundary_extension_confirm_profile.json`
- Reports:
  - `docs/reports/INC0079_PRODUCT_PHASE_TRANSLATION_LARGE_BANK_BOUNDARY_EXTENSION_SCREEN.md`
  - `docs/reports/INC0079_PRODUCT_PHASE_TRANSLATION_LARGE_BANK_BOUNDARY_EXTENSION_CONFIRM.md`
- Gate notes:
  - `docs/governance/gates/gate_20260311_222501.md`
  - `docs/governance/gates/gate_20260311_223841.md`
- 4-seed confirm means:
  - `DENSE_Q08_T12000`
    - `test_top1_after=0.049125`
    - `retrieval_candidate_fraction_mean=1.000000`
    - `retrieval_total_amortized_per_repeat_sec=1.317740`
  - `H4XH4_FIELD_A150_CPX8_Q12_T12000`
    - `test_top1_after=0.048667`
    - `retrieval_candidate_fraction_mean=0.190318`
    - `retrieval_total_amortized_per_repeat_sec=1.222917`
  - `DENSE_Q08_T18000`
    - `test_top1_after=0.048639`
    - `retrieval_candidate_fraction_mean=1.000000`
    - `retrieval_total_amortized_per_repeat_sec=3.056797`
  - `H4XH4_FIELD_A150_CPX8_Q08_T18000`
    - `test_top1_after=0.047667`
    - `retrieval_candidate_fraction_mean=0.189969`
    - `retrieval_total_amortized_per_repeat_sec=2.637824`
- Confirmed upper-bank onset:
  - `max_train=12000`
    - first systems crossover at `Q12`
  - `max_train=18000`
    - first systems crossover already at `Q08`
- Reading:
  - the onset keeps moving left at a larger bank without degrading the search
    work ratio
  - the secondary-key product systems family now has the earliest confirmed
    crossover at `H4XH4_FIELD_A150_CPX8_Q08_T18000`
  - this is stronger hardware-side software evidence again, but still narrow
    because only one bank above `12000` has been confirmed
    - `retrieval_total_amortized_per_repeat_sec=0.944737`
- Current conclusion:
  - the fixed product phase-field branch is now directly positive against
    dense exact retrieval on the repo’s translated real-task pipeline
  - `H4XH4_FIELD_A150_CPX8` is the strongest systems point
  - the current systems lead still gives back a very small amount of top-1
    versus dense exact, which motivated bounded quality recovery

## Product Phase Second Large-Bank Boundary Extension
- Increment doc:
  - `docs/research/increments/INC_0080_product_phase_translation_second_large_bank_boundary_extension.md`
- Analyses:
  - `results/analysis/inc0080_product_phase_translation_second_large_bank_boundary_extension_screen.json`
  - `results/analysis/inc0080_product_phase_translation_second_large_bank_boundary_extension_confirm.json`
  - `results/analysis/inc0080_product_phase_translation_second_large_bank_boundary_extension_screen_profile.json`
  - `results/analysis/inc0080_product_phase_translation_second_large_bank_boundary_extension_confirm_profile.json`
- Key results:
  - `max_train=24000`
    - `Q04`
      - `DENSE_Q04_T24000`: `top1=0.049354`, `amortized=5.152s`
      - `H4XH4_FIELD_A150_CPX8_Q04_T24000`: `0.048833`, `7.425s`
    - `Q08`
      - `DENSE_Q08_T24000`: `0.049354`, `5.491s`
      - `H4XH4_FIELD_A150_CPX8_Q08_T24000`: `0.048833`, `3.867s`
  - `max_train=30000`
    - `Q04`
      - `DENSE_Q04_T30000`: `top1=0.048017`, `amortized=7.901s`
      - `H4XH4_FIELD_A150_CPX8_Q04_T30000`: `0.047167`, `8.946s`
    - `Q08`
      - `DENSE_Q08_T30000`: `0.048017`, `7.560s`
      - `H4XH4_FIELD_A150_CPX8_Q08_T30000`: `0.047167`, `5.394s`
- Confirmed upper-bank onset:
  - `max_train=24000`: first systems crossover at `Q08`
  - `max_train=30000`: first systems crossover at `Q08`
- Reading:
  - the onset now clearly holds at `Q08` through `30000`
  - `Q04` still does not cross
  - the secondary-key search-work ratio remains stable near `19%`
  - this strengthens the hardware-side scaling story, but shifts the next
    question from generic bank extension to explicit `Q04` threshold search

## Product Phase Q04 Threshold Search
- Increment doc:
  - `docs/research/increments/INC_0081_product_phase_translation_q04_threshold_search.md`
- Analyses:
  - `results/analysis/inc0081_product_phase_translation_q04_threshold_search_screen.json`
  - `results/analysis/inc0081_product_phase_translation_q04_threshold_search_confirm.json`
  - `results/analysis/inc0081_product_phase_translation_q04_threshold_search_screen_profile.json`
  - `results/analysis/inc0081_product_phase_translation_q04_threshold_search_confirm_profile.json`
- Key results:
  - `max_train=36000`
    - `Q04`
      - `DENSE_Q04_T36000`: `top1=0.047903`, `amortized=12.149s`
      - `H4XH4_FIELD_A150_CPX8_Q04_T36000`: `0.047069`, `9.694s`
    - `Q08`
      - `DENSE_Q08_T36000`: `0.047903`, `11.093s`
      - `H4XH4_FIELD_A150_CPX8_Q08_T36000`: `0.047069`, `5.950s`
  - `max_train=40000`
    - `Q04`
      - `DENSE_Q04_T40000`: `top1=0.048850`, `amortized=9.024s`
      - `H4XH4_FIELD_A150_CPX8_Q04_T40000`: `0.047325`, `9.990s`
    - `Q08`
      - `DENSE_Q08_T40000`: `0.048850`, `9.642s`
      - `H4XH4_FIELD_A150_CPX8_Q08_T40000`: `0.047325`, `6.106s`
- Confirmed threshold read:
  - `max_train=36000`: first systems crossover at `Q04`
  - `max_train=40000`: first systems crossover at `Q08`
- Reading:
  - the first confirmed `Q04` crossover is real at `T36000`
  - the onset is not monotone, because `T40000` still starts at `Q08`
  - the search-work ratio remains stable, so the split is more likely a
    cost-composition issue than a routing-collapse issue

## Product Phase Cost Accounting Audit
- Increment doc:
  - `docs/research/increments/INC_0082_product_phase_translation_cost_accounting_audit.md`
- Artifacts:
  - `results/analysis/inc0082_product_phase_translation_cost_accounting_audit.json`
  - `docs/reports/INC0082_PRODUCT_PHASE_TRANSLATION_COST_ACCOUNTING_AUDIT.md`
- Key results:
  - `T36000 Q04`
    - online gain per repeat vs dense: `9.862s`
    - offline penalty per repeat vs dense: `7.407s`
    - amortized margin vs dense: `+2.455s`
  - `T40000 Q04`
    - online gain per repeat vs dense: `7.072s`
    - offline penalty per repeat vs dense: `8.038s`
    - amortized margin vs dense: `-0.966s`
  - `T40000 Q08`
    - online gain per repeat vs dense: `7.657s`
    - offline penalty per repeat vs dense: `4.121s`
    - amortized margin vs dense: `+3.536s`
- Reading:
  - the non-monotone `Q04/Q08` threshold is now explained directly by static
    offline route-build cost composition
  - the pruning signal itself stays stable:
    - search-work ratio `0.190206` at `T36000 Q04`
    - search-work ratio `0.183764` at `T40000 Q04`
    - bytes-saved proxy stays near `81%`
  - the next honest move is offline-cost rescue on the fixed translated stack,
    not more bank extension and not new geometry

## Product Phase Persistent Route Cache Rescue
- Increment doc:
  - `docs/research/increments/INC_0083_product_phase_translation_persistent_route_cache.md`
- Artifacts:
  - `results/analysis/inc0083_product_phase_translation_persistent_route_cache_screen_cold.json`
  - `results/analysis/inc0083_product_phase_translation_persistent_route_cache_screen_warm.json`
  - `results/analysis/inc0083_product_phase_translation_persistent_route_cache_screen_compare.json`
  - `docs/reports/INC0083_PRODUCT_PHASE_TRANSLATION_PERSISTENT_ROUTE_CACHE_SCREEN_COMPARE.md`
  - `results/analysis/inc0083_product_phase_translation_persistent_route_cache_confirm_cold.json`
  - `results/analysis/inc0083_product_phase_translation_persistent_route_cache_confirm_warm.json`
  - `results/analysis/inc0083_product_phase_translation_persistent_route_cache_confirm_compare.json`
  - `docs/reports/INC0083_PRODUCT_PHASE_TRANSLATION_PERSISTENT_ROUTE_CACHE_CONFIRM_COMPARE.md`
- Key confirm results:
  - `Q04 T40000`
    - cold:
      - `DENSE_Q04_T40000`: `top1=0.048850`, `amortized=9.347s`
      - `H4XH4_FIELD_A150_CPX8_Q04_T40000`: `0.047325`,
        `cand_frac=0.183764`, `amortized=10.389s`
    - warm:
      - `DENSE_Q04_T40000`: `top1=0.048850`, `amortized=9.246s`
      - `H4XH4_FIELD_A150_CPX8_Q04_T40000`: `0.047325`,
        `cand_frac=0.183764`, `amortized=1.972s`
      - routed cache hits: `chart=1.0`, `route=1.0`
  - `Q08 T40000`
    - cold:
      - `DENSE_Q08_T40000`: `0.048850`, `9.506s`
      - `H4XH4_FIELD_A150_CPX8_Q08_T40000`: `0.047325`,
        `0.183764`, `6.165s`
    - warm:
      - `DENSE_Q08_T40000`: `0.048850`, `9.113s`
      - `H4XH4_FIELD_A150_CPX8_Q08_T40000`: `0.047325`,
        `0.183764`, `1.891s`
      - routed cache hits: `chart=1.0`, `route=1.0`
- Reading:
  - `INC-0082` was correct: the blocking term was static offline build cost,
    not route failure
  - persistent cache reuse now removes almost all of that cost without changing
    top-1 or candidate fraction
  - `Q04 T40000` is now a strong warm-cache crossover on the fixed translated
    stack
  - the next honest move is a warm-cache onset map on the fixed `T40000` bank,
    not new geometry

## Product Phase Warm Cache Onset Map
- Increment doc:
  - `docs/research/increments/INC_0084_product_phase_translation_warm_cache_onset_map.md`
- Artifacts:
  - `results/analysis/inc0084_product_phase_translation_warm_cache_onset_map_screen.json`
  - `results/analysis/inc0084_product_phase_translation_warm_cache_onset_map_confirm.json`
  - `docs/reports/INC0084_PRODUCT_PHASE_TRANSLATION_WARM_CACHE_ONSET_MAP.md`
- Key confirm results:
  - dense:
    - `Q01`: `top1=0.048850`, `amortized=9.536s`
    - `Q02`: `0.048850`, `9.173s`
    - `Q04`: `0.048850`, `9.364s`
    - `Q08`: `0.048850`, `9.244s`
  - routed:
    - `Q01`: `top1=0.047325`, `cand_frac=0.183764`, `amortized=2.204s`
    - `Q02`: `0.047325`, `0.183764`, `2.022s`
    - `Q04`: `0.047325`, `0.183764`, `1.924s`
    - `Q08`: `0.047325`, `0.183764`, `1.868s`
  - routed cache hits:
    - `chart_cache_hit=1.0`
    - `route_cache_hit=1.0`
- Reading:
  - the fixed translated product stack is now confirm-stage system-positive at
    `Q01` under the persisted-bank assumption
  - this is still not a new geometry read; it is an operational reuse read on
    the fixed geometry/search law
  - the next honest move is to find the earliest bank where warm-cache `Q01`
    holds, not to reopen the route law

## Product Phase Dense Quality Recovery
- Increment doc:
  - `docs/research/increments/INC_0075_product_phase_translation_dense_quality_recovery.md`
- Analyses:
  - `results/analysis/inc0075_product_phase_translation_dense_quality_recovery_screen.json`
  - `results/analysis/inc0075_product_phase_translation_dense_quality_recovery_confirm.json`
- Gate notes:
  - `docs/governance/gates/gate_20260311_142843.md`
  - `docs/governance/gates/gate_20260311_144445.md`
- 4-seed confirm means:
  - `DENSE_Q24`
    - `test_top1_after=0.049125`
    - `retrieval_total_amortized_per_repeat_sec=1.355`
  - `H4XH4_FIELD_A150`
    - `test_top1_after=0.049125`
    - `retrieval_candidate_fraction_mean=0.314749`
    - `retrieval_total_amortized_per_repeat_sec=0.935`
  - `H4XH4_FIELD_A150_CPX8`
    - `test_top1_after=0.048667`
    - `retrieval_candidate_fraction_mean=0.190318`
    - `retrieval_total_amortized_per_repeat_sec=0.893`
- Current conclusion:
  - bounded rerank quality rescue did not improve the fixed dense-frontier law
  - the frontier remains split:
    - `H4XH4_FIELD_A150` is the quality-matched routed point
    - `H4XH4_FIELD_A150_CPX8` is the strongest systems point
  - the next honest step is break-even mapping, not another retrieval tweak

## Product Phase Break-Even
- Increment doc:
  - `docs/research/increments/INC_0076_product_phase_translation_break_even.md`
- Analyses:
  - `results/analysis/inc0076_product_phase_translation_break_even_screen.json`
  - `results/analysis/inc0076_product_phase_translation_break_even_confirm.json`
- Gate notes:
  - `docs/governance/gates/gate_20260311_145722.md`
  - `docs/governance/gates/gate_20260311_151013.md`
- 4-seed confirm means at the crossover bracket:
  - `Q08`
    - `DENSE_Q08`: `top1=0.049125`, `amortized=1.377s`
    - `H4XH4_FIELD_A150_Q08`: `0.049125`, `1.838s`
    - `H4XH4_FIELD_A150_CPX8_Q08`: `0.048667`, `1.691s`
  - `Q16`
    - `DENSE_Q16`: `top1=0.049125`, `amortized=1.322s`
    - `H4XH4_FIELD_A150_Q16`: `0.049125`, `0.314749 cand_frac`, `1.163s`
    - `H4XH4_FIELD_A150_CPX8_Q16`: `0.048667`, `0.190318 cand_frac`, `1.036s`
  - `Q24`
    - `DENSE_Q24`: `top1=0.049125`, `amortized=1.328s`
    - `H4XH4_FIELD_A150_Q24`: `0.049125`, `0.314749 cand_frac`, `0.931s`
    - `H4XH4_FIELD_A150_CPX8_Q24`: `0.048667`, `0.190318 cand_frac`, `0.831s`
- Current conclusion:
  - the fixed product branch now has a confirmed amortized crossover against
    dense exact retrieval on the translated pipeline
  - `Q16` is the first practical confirmed crossover
  - `H4XH4_FIELD_A150_Q16` is the quality-matched break-even point
  - `H4XH4_FIELD_A150_CPX8_Q16` is the stronger-pruning systems crossover point
  - `H4XH4_FIELD_A150_CPX8_Q24` remains the stabilized dense-frontier systems
    point

## Translated Retrieval Amortization
- Increment doc:
  - `docs/research/increments/INC_0051_retrieval_amortization.md`
- Analysis:
  - `results/analysis/inc0051_retrieval_amortization_screen.json`
- Gate:
  - `docs/governance/gates/gate_20260306_114654.md`
- 2-seed screen means at the decisive repeat counts:
  - `DENSE_Q16`
    - `test_mse_after=0.004318726`
    - `retrieval_total_amortized_per_repeat_sec=0.474373`
  - `HOPF_RET_P1_Q16`
    - `test_mse_after=0.004325216`
    - `retrieval_total_amortized_per_repeat_sec=0.727979`
    - `retrieval_candidate_fraction_mean=0.348843`
  - `DENSE_Q24`
    - `test_mse_after=0.004318726`
    - `retrieval_total_amortized_per_repeat_sec=0.554539`
  - `HOPF_RET_P1_Q24`
    - `test_mse_after=0.004325216`
    - `retrieval_total_amortized_per_repeat_sec=0.539914`
    - `retrieval_candidate_fraction_mean=0.348843`
  - `HOPF_PHI2_RET_P1_Q24`
    - `test_mse_after=0.004326332`
    - `retrieval_total_amortized_per_repeat_sec=0.631121`
    - `retrieval_candidate_fraction_mean=0.341492`
- Current conclusion:
  - the translated retrieval path now has a first amortized crossover candidate
  - plain Hopf, not widened Hopf, is the live translated retrieval family
  - the crossover is narrow enough that the next responsible step is a targeted confirm, not a packaging claim

## Translated Retrieval Amortization Confirm
- Increment doc:
  - `docs/research/increments/INC_0052_retrieval_amortization_confirm.md`
- Analysis:
  - `results/analysis/inc0052_retrieval_amortization_confirm.json`
- Gate:
  - `docs/governance/gates/gate_20260306_115931.md`
- 4-seed confirm means:
  - `DENSE_Q24`
    - `test_mse_after=0.004321788`
    - `retrieval_total_amortized_per_repeat_sec=0.505093`
  - `HOPF_RET_P1_Q24`
    - `test_mse_after=0.004324992`
    - `retrieval_total_amortized_per_repeat_sec=0.593752`
    - `retrieval_candidate_fraction_mean=0.351066`
  - `DENSE_Q32`
    - `test_mse_after=0.004321788`
    - `retrieval_total_amortized_per_repeat_sec=0.558604`
  - `HOPF_RET_P1_Q32`
    - `test_mse_after=0.004324992`
    - `retrieval_total_amortized_per_repeat_sec=0.654360`
    - `retrieval_candidate_fraction_mean=0.351066`
- Current conclusion:
  - the screen-stage amortized crossover did not survive confirm
  - translated retrieval remains a useful evaluation harness, not a promoted systems answer
  - the next live branch should return to deeper dynamic geometry

## Gated Phi^2 Widening
- Increment doc:
  - `docs/research/increments/INC_0032_phi2_gated_widening.md`
- Analysis:
  - `results/analysis/inc0032_phi2_gated_screen.json`
- Gate:
  - `docs/governance/gates/gate_20260306_014339.md`
- Reading:
  - sparse threshold gating did not rescue the widened Hopf family operationally
  - `HOPF_PHI2_G062` and `HOPF_PHI2_G085` reduced runtime relative to ungated `HOPF_PHI2_K25`, but remained dramatically slower than `HOPF_K25_BASE` and `R0`
  - the strict gate improved quality somewhat, but worsened `chi` concentration
  - conclusion:
    - the `phi^2` lattice is still valid geometry evidence
    - per-point gating is not the right operational form
    - the next discrete branch should use a small number of shared rung states instead of thresholding every point

## Banded Phi^2 Lattice
- Increment doc:
  - `docs/research/increments/INC_0033_phi2_band_lattice.md`
- Analysis:
  - `results/analysis/inc0033_phi2_band_screen.json`
- Gate:
  - `docs/governance/gates/gate_20260306_021036.md`
- 2-seed screen means:
  - `HOPF_K25_BASE`
    - `test_mse_after=0.003888756`
    - `total_sec=62.389`
    - `eval_sectors=4.0`
  - `HOPF_PHI2_BAND`
    - `test_mse_after=0.003897103`
    - `total_sec=65.725`
    - `eval_sectors=10.5`
    - `adaptive_chi_bin_pmax=0.9418`
  - `HOPF_PHI2_K25`
    - `test_mse_after=0.003902407`
    - `total_sec=116.411`
    - `eval_sectors=10.5`
    - `adaptive_chi_bin_pmax=0.9424`
  - `R0`
    - `test_mse_after=0.003911258`
    - `total_sec=52.127`
- Reading:
  - banded shared states rescued the widened Hopf family substantially on runtime relative to ungated `phi^2`
  - the widening signal stayed intact
  - the branch still did not clear the operational gate vs `R0`
  - `chi` concentration did not materially improve
  - conclusion:
    - `HOPF_PHI2_BAND` is the current widened Hopf geometry candidate
    - the next viable route law at that stage was a blended Hopf-capacity branch, not more gating

## Blended Hopf-Capacity Law
- Increment doc:
  - `docs/research/increments/INC_0034_blended_hopf_capacity.md`
- Analysis:
  - `results/analysis/inc0034_blended_hopf_screen.json`
- Gate:
  - `docs/governance/gates/gate_20260306_024928.md`
- 2-seed screen means:
  - `HOPF_K25_BASE`
    - `test_mse_after=0.003888756`
    - `total_sec=62.885`
  - `HOPF_PHI2_BAND`
    - `test_mse_after=0.003897103`
    - `total_sec=62.094`
    - `adaptive_chi_bin_pmax=0.9418`
  - `HOPF_BLEND_L110_C15_S05`
    - `test_mse_after=0.003911125`
    - `total_sec=59.899`
    - `adaptive_chi_bin_pmax=0.7628`
  - `HOPF_BLEND_L080_C10_S05`
    - `test_mse_after=0.003914592`
    - `total_sec=68.800`
    - `adaptive_chi_bin_pmax=0.7720`
  - `R0`
    - `test_mse_after=0.003911258`
    - `total_sec=45.506`
- Reading:
  - the blended law did reduce `chi` concentration relative to `HOPF_PHI2_BAND`
  - it did not beat pure Hopf on quality
  - it did not beat `R0` on runtime
  - conclusion:
    - keep `HOPF_K25_BASE` as the routed-quality lead
    - keep `HOPF_PHI2_BAND` as the widened Hopf geometry candidate
    - move the next branch to stronger Poincare-ball global alignment rather than more local widening laws

## Poincare-Global Alignment Slices
- Increment doc:
  - `docs/research/increments/INC_0035_poincare_global_alignment.md`
- Analyses:
  - `results/analysis/inc0035_alignment_diag_screen.json`
  - `results/analysis/inc0035_shell_anchor_screen.json`
- Gate notes:
  - `docs/governance/gates/gate_20260306_030909.md`
  - `docs/governance/gates/gate_20260306_032618.md`
- Reading:
  - Slice A proved the Poincare alignment metric is live and informative in the proxy harness
  - `HOPF_PHI2_BAND` preserved global alignment best on the fast diagnostic screen
  - Slice B proved shell-only anchoring is not enough:
    - `HOPF_K25_BALL` improved MSE relative to pure Hopf
    - but worsened shell concentration and pairwise alignment
  - conclusion:
    - the next viable global branch is chart-isometry / shared-route-coordinate routing, not shell-only repair

## Chart-Isometry and Bounded-Isometry Routing
- Increment docs:
  - `docs/research/increments/INC_0036_chart_isometry_route.md`
  - `docs/research/increments/INC_0037_isometric_band_route.md`
  - `docs/research/increments/INC_0038_bounded_isometry_band_route.md`
- Analyses:
  - `results/analysis/inc0036_chart_iso_screen.json`
  - `results/analysis/inc0037_isometric_band_screen.json`
  - `results/analysis/inc0038_bounded_band_screen.json`
- Reading:
  - pure isometry restored exact Poincare alignment, but stayed too compressed and too slow
  - isometric band routing restored exact alignment and widened capacity, but was still too slow
  - bounded isometry produced a clean alignment/runtime interpolation
  - no bounded point passed the operational screen
- Conclusion:
  - alignment and widening are both real
  - the next likely missing ingredient is route/memory coordinate separation, not another bounded-scale sweep

## Route/Memory Separation and Frontier Confirm
- Increment/control docs:
  - `docs/research/increments/INC_0039_route_memory_separation.md`
  - `docs/research/controls/CTRL_0003_hopf_frontier_confirm.md`
- Analyses:
  - `results/analysis/inc0039_route_memory_screen.json`
  - `results/analysis/ctrl0003_hopf_frontier_confirm.json`
- Gate notes:
  - `docs/governance/gates/gate_20260306_084204.md`
  - `docs/governance/gates/gate_20260306_085323.md`
- Reading:
  - route/memory separation improved geometry but did not recover enough runtime to replace the live Hopf frontier
  - strict 4-seed confirm resolved the frontier:
    - `HOPF_K25_BASE`
      - `test_mse_after=0.003937984`
      - `total_sec=44.838`
      - `eval_shells=2.25`
      - `eval_sectors=4.0`
      - health pass
    - `HOPF_PHI2_BAND`
      - `test_mse_after=0.003921230`
      - `total_sec=51.541`
      - `eval_shells=2.25`
      - `eval_sectors=9.25`
      - health fail on runtime
    - `R0`
      - `test_mse_after=0.003946853`
      - `total_sec=42.409`
      - shell-collapse health fail
- Timing diagnosis from the same confirm:
  - `chart_opt` dominates every branch:
    - `HOPF_K25_BASE`: `39.959s`
    - `HOPF_PHI2_BAND`: `40.658s`
    - `R0`: `40.634s`
  - widened Hopf pays a much larger EMA cost:
    - `HOPF_K25_BASE`: `4.143s`
    - `HOPF_PHI2_BAND`: `10.133s`
    - `R0`: `0.856s`
- Conclusion:
  - pure Hopf is now the healthiest routed branch on the proxy
  - widened Hopf remains the best widened-quality candidate, but is not operationally promotable yet
  - the next responsible branch is cost decomposition, not more geometry invention

## Cost Frontier Rescue
- Increment doc:
  - `docs/research/increments/INC_0040_hopf_cost_decomposition.md`
- Analyses:
  - `results/analysis/inc0040_cost_screen.json`
  - `results/analysis/inc0040_cost_confirm.json`
- Gate notes:
  - `docs/governance/gates/gate_20260306_091429.md`
  - `docs/governance/gates/gate_20260306_092503.md`
- Reading:
  - chart schedule was the real runtime bottleneck at this stage
  - reduced schedule (`chart_iters=60`, `so8_candidates=4`, `scale_candidates=4`) rescued both pure Hopf and widened Hopf
  - 4-seed confirm:
    - `HOPF_K25_BASE_IT60_P4`
      - `test_mse_after=0.003919349`
      - `total_sec=19.905`
      - `chart_opt=12.421`
      - health pass
    - `HOPF_PHI2_BAND_IT60_P4`
      - `test_mse_after=0.003928139`
      - `total_sec=18.270`
      - `chart_opt=10.387`
      - health pass
    - `R0`
      - `test_mse_after=0.003946853`
      - `total_sec=44.240`
      - shell-collapse health fail
- Conclusion:
  - the current operational routed lead is now cost-reduced pure Hopf
  - the current widened efficient lead is now cost-reduced widened Hopf
  - the next question is robustness under larger subset load, not another route-law search

## Large-Subset Cost Frontier
- Increment doc:
  - `docs/research/increments/INC_0041_cost_frontier_large_subset.md`
- Analysis:
  - `results/analysis/inc0041_cost_large_subset.json`
- Gate note:
  - `docs/governance/gates/gate_20260306_093641.md`
- Reading:
  - the smaller-subset cost rescue did not survive as an operational runtime win
  - 4-seed larger-subset means:
    - `HOPF_K25_BASE_IT60_P4`
      - `test_mse_after=0.003895705`
      - `total_sec=37.090`
      - `chart_opt=19.655`
      - `training_ema=16.140`
    - `HOPF_PHI2_BAND_IT60_P4`
      - `test_mse_after=0.003904061`
      - `total_sec=47.604`
      - `chart_opt=25.087`
      - `training_ema=21.057`
    - `R0`
      - `test_mse_after=0.003913707`
      - `total_sec=27.271`
      - shell-collapse health fail
- Conclusion:
  - reduced-schedule pure Hopf remains the best large-subset quality candidate
  - widened Hopf is clearly behind pure Hopf under larger load
  - the next branch should attack large-subset EMA/chart pressure, not reopen geometry yet

## Fixed Phase4D Stabilization
- Analyses:
  - `results/analysis/inc0009_proxy_stabilization_screen.json`
  - `results/analysis/inc0009_proxy_stabilization_confirm.json`
- Best fixed candidate after confirm: `R5B_K25`
  - `test_mse_after=0.0038885`
  - `total_sec=37.588`
  - `eval_sectors=4.0`
  - `pmax_after=0.675`
- Reading:
  - fixed `K` widening helped somewhat, but the branch still failed the route-health gate
  - shell diversity stayed at `1.0`

## Adaptive Phase4D Transfer
- Math spec:
  - `docs/research/ADAPTIVE_PHASE4D_SPEC.md`
- Analysis:
  - `results/analysis/inc0010_adaptive_phase4d_confirm.json`
  - `docs/governance/gates/gate_20260305_151230.md`
- Best adaptive candidate: `R5A_K25_M3`
  - `test_mse_after=0.0039009`
  - `total_sec=32.664`
  - `buckets=11.0`
  - `eval_sectors=11.0`
  - `pmax_after=0.598`
- Reading:
  - `phase4d_adaptive` is the first route in this repo to satisfy the configured proxy transfer-health gate
  - it gives up a small amount of quality versus collapsed `R5B`, but remains better than `R0` while becoming much healthier geometrically
  - shell diversity is still absent, so the adaptive branch is a transfer lead, not an end-state

## Shell Activation
- Increment doc:
  - `docs/research/increments/INC_0011_shell_activation.md`
- Analyses:
  - `results/analysis/inc0011_shell_activation_screen.json`
  - `results/analysis/inc0011_shell_activation_confirm.json`
- Gate notes:
  - `docs/governance/gates/gate_20260305_153422.md`
  - `docs/governance/gates/gate_20260305_153937.md`
- Confirm means:
  - `R5A_REF`
    - `test_mse_after=0.0039009`
    - `total_sec=33.887`
    - `eval_shells=1.0`
    - `shell_pmax=1.000`
  - `R5A_SG12`
    - `test_mse_after=0.0039451`
    - `total_sec=33.330`
    - `eval_shells=4.5`
    - `shell_pmax=0.442`
    - `buckets=38.5`
  - `R5A_SG16_SB10`
    - `test_mse_after=0.0039636`
    - `total_sec=35.384`
    - `eval_shells=40.5`
    - `shell_pmax=0.190`
    - `buckets=167.5`
- Reading:
  - shell divergence works; radial collapse is no longer a hard blocker
  - `R5A_SG12` is the first shell-active route that remains effectively tied with `R0` on proxy MSE while running slightly faster
  - `R5A_SG16_SB10` proves the radial field can be widened much further, but the unseen-rate penalty shows that divergence alone is not enough
  - the next transfer question is controlled convergence, not whether shells can open at all

## Convergence-Controlled Shell Geometry
- Increment doc:
  - `docs/research/increments/INC_0012_convergence_control.md`
- Analyses:
  - `results/analysis/inc0012_convergence_screen.json`
  - `results/analysis/inc0012_convergence_confirm.json`
- Gate notes:
  - `docs/governance/gates/gate_20260305_160251.md`
  - `docs/governance/gates/gate_20260305_160951.md`
- Confirm means:
  - `R0`
    - `test_mse_after=0.0039450`
    - `total_sec=32.666`
    - `eval_shells=1.0`
    - `shell_pmax=1.000`
  - `R5A_SG12_REF`
    - `test_mse_after=0.0039451`
    - `total_sec=32.291`
    - `eval_shells=4.5`
    - `shell_pmax=0.442`
    - `unseen_rate=0.001667`
  - `R5A_SG12_C10`
    - `test_mse_after=0.0039362`
    - `total_sec=31.231`
    - `eval_shells=2.5`
    - `shell_pmax=0.542`
    - `unseen_rate=0.000667`
  - `R5A_SG16_C10`
    - `test_mse_after=0.0039431`
    - `total_sec=28.815`
    - `eval_shells=3.5`
    - `shell_pmax=0.639`
    - `unseen_rate=0.002000`
  - `R5A_SG16_C10_D35`
    - `test_mse_after=0.0039278`
    - `total_sec=28.430`
    - `eval_shells=2.0`
    - `shell_pmax=0.792`
    - `unseen_rate=0.000333`
- Reading:
  - controlled convergence works; the project no longer needs to choose between shell collapse and shell explosion
  - `R5A_SG12_C10` is a cleaner version of the moderate shell branch
  - the mean-gate lead after this increment was `R5A_SG16_C10_D35`, which beat `R0` on both quality and runtime while staying inside the configured mean shell-health gate
  - `delta_r` now looks like part of the routing law because slightly coarser shell quantization materially improved the stronger-divergence branch
  - the remaining transfer question is not activation or first-pass convergence, but the size of the safe operating band around this lead

## Shell-Control Phase Diagram
- Increment doc:
  - `docs/research/increments/INC_0013_phase_diagram.md`
- Analyses:
  - `results/analysis/inc0013_phase_diagram_screen.json`
  - `results/analysis/inc0013_phase_diagram_confirm.json`
  - `results/analysis/inc0013_phase_diagram_confirm_strict.json`
- Gate notes:
  - `docs/governance/gates/gate_20260305_163649.md`
  - `docs/governance/gates/gate_20260305_164628.md`
  - `docs/governance/gates/gate_20260305_164628_strict.md`
- Strict-review reading:
  - `PD_CENTER` (`R5A_SG16_C10_D35`) and `PD_SG18` both fail seed-wise review because one confirm seed crosses `shell_pmax=0.85`
  - `PD_C12` also fails strict review because one seed collapses to one shell even though the route mean looked healthy
  - `PD_D40` is the collapse-side anchor
  - `PD_D30` is the best current strict-health candidate:
    - `test_mse_after=0.0039431`
    - `total_sec=29.851`
    - `eval_shells=3.5`
    - `shell_pmax=0.639`
    - `unseen_rate=0.0020`
  - `PD_T080` is the conservative interior comparator
  - `PD_C06` is the stable high-dispersion comparator
- Direction update:
  - multi-seed transfer promotion now requires seed-wise health, not just mean health
  - after `INC-0013`, the strict-health transfer lead was `R5A_SG16_C10_D30`
  - the next transfer problem is robustness of the low-`delta_r` healthy band

## Larger-Subset Strict-Health Robustness
- Increment doc:
  - `docs/research/increments/INC_0014_strict_robustness.md`
- Analysis:
  - `results/analysis/inc0014_strict_robustness.json`
- Gate note:
  - `docs/governance/gates/gate_20260305_171526.md`
- 4-seed larger-subset means (`train=5000`, `test=2500`):
  - `R0`
    - `test_mse_after=0.0039079`
    - `total_sec=75.653`
    - `eval_shells=1.0`
    - `shell_pmax=1.000`
  - `R5A_SG16_C10_D30`
    - `test_mse_after=0.0039185`
    - `total_sec=50.461`
    - `eval_shells=3.0`
    - `shell_pmax=0.579`
    - `unseen_rate=0.0005`
    - strict-health pass
  - `R5A_SG18_C10_D35`
    - `test_mse_after=0.0039183`
    - `total_sec=50.966`
    - `eval_shells=3.0`
    - `shell_pmax=0.658`
    - `unseen_rate=0.0007`
    - strict-health pass
  - `R5A_SG16_C10_T080_D35`
    - `test_mse_after=0.0039128`
    - `total_sec=58.827`
    - failed strict review on multiple seeds
  - `R5A_SG16_C06_D35`
    - `test_mse_after=0.0039326`
    - `total_sec=52.589`
    - `eval_shells=7.0`
    - `shell_pmax=0.482`
    - `unseen_rate=0.0009`
    - strict-health pass
- Reading:
  - `R0` still wins raw proxy MSE, so the adaptive branch is not the quality leader on this benchmark
  - the adaptive branch now owns the healthy hardware-efficiency frontier
  - `R5A_SG16_C10_D30` is the hardware-efficiency lead because it is fastest among the healthy routes while staying within the configured MSE tolerance vs `R0`

## Discrete Phi-Ladder Controller Screen
- Increment doc:
  - `docs/research/increments/INC_0021_phi_ladder_screen.md`
- Analysis:
  - `results/analysis/inc0021_phi_ladder_screen.json`
- Gate note:
  - `docs/governance/gates/gate_20260305_202833.md`
- 2-seed seed-major larger-subset means:
  - `R0`
    - `test_mse_after=0.0039113`
    - `total_sec=42.542`
    - collapsed
  - `PHI_D32_L120`
    - `test_mse_after=0.0039543`
    - `total_sec=68.881`
    - health fail on runtime ratio
  - `LADDER_D32_L045`
    - `test_mse_after=0.0039383`
    - `total_sec=50.493`
    - health fail on runtime ratio
  - `LADDER_D32_L055`
    - `test_mse_after=0.0039335`
    - `total_sec=50.592`
    - health fail on runtime ratio
  - `LADDER_D32_L065`
    - `test_mse_after=0.0039419`
    - `total_sec=47.407`
    - health pass
- Reading:
  - the discrete `phi` ladder is a real improvement over the continuous `phi_ratio` controller
  - `LADDER_D32_L065` is now the healthiest routed controller candidate
  - but no healthy routed branch beats `R0` on runtime, so the main bottleneck is no longer just controller law
  - the next likely fix is shell metric geometry:
    - current shell control is logarithmic / multiplicative
    - shell indexing is still linear in `delta_r`
    - the next branch should test `phi`-spaced log shells
  - `R5A_SG18_C10_D35` is close enough to matter, but not cleaner or faster than `D30`
  - transfer evaluation now needs a Pareto reading, not a single raw-MSE scalar

## Ridge Discrimination
- Increment doc:
  - `docs/research/increments/INC_0015_ridge_discrimination.md`
- Analysis:
  - `results/analysis/inc0015_ridge_discrimination.json`
- Gate note:
  - `docs/governance/gates/gate_20260305_174043.md`
- Reading:
  - at fixed `delta_r`, `SG16` and `SG18` are effectively identical on quality, shell count, shell concentration, unseen-rate, and `adaptive_shell_mult_mean`
  - `adaptive_shell_mult_mean ≈ 2.4596` across the healthy ridge, which matches `exp(c_target + c_hyst)`
  - the current controller is saturating away the shell-growth axis
  - `delta_r`, not `shell_growth`, is the only live radial control variable in the current law

## Delta-Only Confirm
- Increment doc:
  - `docs/research/increments/INC_0016_delta_confirm.md`
- Analysis:
  - `results/analysis/inc0016_delta_confirm.json`
- Gate note:
  - `docs/governance/gates/gate_20260305_175537.md`
- 4-seed means:
  - `D28_SG18`
    - `mse=0.003915777`
    - `total=54.294s`
    - `shells=2.0`
    - `shell_pmax=0.580`
  - `D30_SG18`
    - `mse=0.003915140`
    - `total=49.334s`
    - `shells=3.0`
    - `shell_pmax=0.579`
  - `D35_SG18`
    - `mse=0.003918287`
    - `total=49.549s`
    - `shells=3.0`
    - `shell_pmax=0.658`
- Reading:
  - `D28` does not survive confirm as a lead-replacement candidate
  - `D30` remains the best current radial law
  - `D35` remains more concentrated and slightly worse
  - the next serious branch is no longer another `delta_r` or `shell_growth` sweep
  - the next serious branch is a new cap/merge controller

## Phi-Ratio Controller
- Increment doc:
  - `docs/research/increments/INC_0017_phi_ratio_controller.md`
- Analyses:
  - `results/analysis/inc0017_phi_ratio_screen.json`
  - `results/analysis/inc0017_phi_ratio_confirm.json`
- Gate notes:
  - `docs/governance/gates/gate_20260305_181815.md`
  - `docs/governance/gates/gate_20260305_183037.md`
- 4-seed larger-subset means:
  - `R0`
    - `mse=0.003907888`
    - `total=41.442s`
    - `shells=1.0`
    - `shell_pmax=1.000`
  - `D30_FIXED_SG16`
    - `mse=0.003918454`
    - `total=42.805s`
    - `shells=3.0`
    - `shell_pmax=0.579`
    - `unseen=0.0005`
  - `D30_PHI_L120`
    - `mse=0.003914423`
    - `total=44.637s`
    - `shells=3.25`
    - `shell_pmax=0.555`
    - `unseen=0.0002`
  - `D30_PHI_L100`
    - `mse=0.003932333`
    - `total=45.349s`
    - `shells=7.75`
    - `shell_pmax=0.486`
    - `unseen=0.0036`
- Reading:
  - the `phi` controller is a real mechanism improvement because it reopens the shell controller as a live axis
  - `D30_PHI_L120` is the best healthy `phi` branch and slightly improves routed-branch MSE vs the fixed `D30` controller
  - `D30_FIXED_SG16` remains faster than `D30_PHI_L120`, so the fixed controller stays operationally preferred for hardware-efficiency
  - both routed branches remain slower than `R0` in this confirm batch, so the transfer track still has a quality baseline / routed-lead split rather than a single promoted winner
  - the next mathematically justified step is to retune `delta_r` under the live `phi` controller instead of assuming the fixed-regime `D30` optimum still applies

## Phi Delta Retune
- Increment doc:
  - `docs/research/increments/INC_0018_phi_delta_retune.md`
- Analyses:
  - `results/analysis/inc0018_phi_delta_screen.json`
  - `results/analysis/inc0018_phi_delta_confirm.json`
- Gate notes:
  - `docs/governance/gates/gate_20260305_184239.md`
  - `docs/governance/gates/gate_20260305_185546.md`
- 4-seed larger-subset means:
  - `R0`
    - `mse=0.003907888`
    - `total=45.985s`
    - `shells=1.0`
    - `shell_pmax=1.000`
  - `D30_FIXED_SG16`
    - `mse=0.003918454`
    - `total=47.476s`
    - `shells=3.0`
    - `shell_pmax=0.579`
    - `unseen=0.0005`
  - `PHI_D30_L120`
    - `mse=0.003914423`
    - `total=50.308s`
    - `shells=3.25`
    - `shell_pmax=0.555`
    - `unseen=0.0002`
  - `PHI_D32_L120`
    - `mse=0.003937115`
    - `total=40.801s`
    - `shells=6.0`
    - `shell_pmax=0.543`
    - `unseen=0.0020`
- Reading:
  - `INC-0018` shows the `phi` controller can win the routed hardware-efficiency objective once `delta_r` is retuned for the unsaturated regime
  - `PHI_D32_L120` becomes the transfer hardware-efficiency lead because it is materially faster than both `R0` and `D30_FIXED_SG16` while remaining inside the configured quality tolerance
  - `PHI_D30_L120` remains the better routed-branch quality point and therefore stays relevant as a quality-first comparator
  - `R0` still owns raw transfer quality, so the transfer track remains a Pareto frontier rather than a single scalar winner
  - the next justified branch is hybrid `phase4d_adaptive -> complex2` local zoom, with discrete complex-multiplication / imaginary-field routing reserved as the local mechanism candidate

## Hybrid Local Zoom Screen
- Increment doc:
  - `docs/research/increments/INC_0019_hybrid_local_zoom.md`
- Analysis:
  - `results/analysis/inc0019_hybrid_screen.json`
- Gate note:
  - `docs/governance/gates/gate_20260305_191832.md`
- 2-seed screen means:
  - `PHI_D32_L120`
    - `mse=0.003937783`
    - `total=29.284s`
    - `unseen=0.00033`
  - `HYB_L4_R4`
    - `mse=0.003957661`
    - `total=31.961s`
    - `unseen=0.01467`
  - `HYB_L9_R4`
    - `mse=0.003978375`
    - `total=34.356s`
    - `unseen=0.03367`
- Reading:
  - the first hybrid local-zoom branch increases address richness, but it does so by fragmenting the route space too aggressively
  - the next viable hybrid branch needs local convergence, not more local bins

## Seed-Major Lead Control
- Control doc:
  - `docs/research/controls/CTRL_0001_seedmajor_lead.md`
- Analysis:
  - `results/analysis/ctrl0001_seedmajor_lead.json`
- Gate note:
  - `docs/governance/gates/gate_20260305_192810.md`
- 4-seed larger-subset means:
  - `R0`
    - `mse=0.003907888`
    - `total=44.125s`
  - `D30_FIXED_SG16`
    - `mse=0.003918454`
    - `total=44.432s`
  - `PHI_D32_L120`
    - `mse=0.003937115`
    - `total=43.990s`
- Reading:
  - `PHI_D32_L120` survives the seed-major control as the fastest healthy routed branch
  - the runtime edge is now narrow, so the lead is retained but should be described cautiously

## Hybrid Local Rescue Screen
- Increment doc:
  - `docs/research/increments/INC_0020_hybrid_local_rescue.md`
- Analysis:
  - `results/analysis/inc0020_hybrid_rescue_screen.json`
- Gate note:
  - `docs/governance/gates/gate_20260305_195327.md`
- 2-seed screen means:
  - `HYB4_M2_T010_C005`
    - `mse=0.003936500`
    - `total=29.562s`
    - `unseen=0.000667`
  - `PHI_D32_L120`
    - `mse=0.003937783`
    - `total=29.962s`
    - `unseen=0.000333`
- Reading:
  - local convergence rescue made the hybrid branch viable
  - the screen was good enough to justify a 4-seed larger-subset confirm

## Hybrid Local Rescue Confirm
- Increment doc:
  - `docs/research/increments/INC_0020_hybrid_local_rescue.md`
- Analysis:
  - `results/analysis/inc0020_hybrid_rescue_confirm.json`
- Gate note:
  - `docs/governance/gates/gate_20260305_200621.md`
- 4-seed larger-subset means:
  - `R0`
    - `mse=0.003907888`
    - `total=43.329s`
  - `HYB4_M2_T010_C005`
    - `mse=0.003920267`
    - `total=46.116s`
  - `HYB4_M2_T005_C005`
    - `mse=0.003923083`
    - `total=46.062s`
  - `PHI_D32_L120`
    - `mse=0.003937115`
    - `total=45.038s`
- Reading:
  - the rescued hybrid branch is now the healthiest routed-quality branch
  - `PHI_D32_L120` remains the fastest healthy routed branch
  - no healthy route beat `R0` on runtime in this confirm

## PHI_PHI_PHI Coarse Shell Confirm
- Increment doc:
  - `docs/research/increments/INC_0022_phi_phi_phi_shells.md`
- Family doc:
  - `docs/research/PHI_PHI_PHI_FAMILY.md`
- Analysis:
  - `results/analysis/inc0022_phi_log_confirm.json`
- Gate note:
  - `docs/governance/gates/gate_20260305_210615.md`
- 4-seed larger-subset means:
  - `R0`
    - `mse=0.003907888`
    - `total=47.751s`
    - `shells=1.0`
  - `LINEAR_D32_L065`
    - `mse=0.003924776`
    - `total=52.032s`
    - `shells=4.25`
  - `PHI_PHI_PHI v1` (`PHILOG_D36_L065`)
    - `mse=0.003901309`
    - `total=50.566s`
    - `shells=2.75`
- Reading:
  - shell-metric correction is real and confirmed
  - this is now the best healthy transfer-quality route in the repo
  - the branch beats the linear-shell comparator on both quality and runtime
  - the branch still does not beat `R0` on absolute runtime
  - next transfer question is budget compression inside the `PHI_PHI_PHI` family

## PHI_PHI_PHI Budget Compression Confirm
- Increment doc:
  - `docs/research/increments/INC_0023_phi3_budget_compression.md`
- Analysis:
  - `results/analysis/inc0023_phi3_budget_confirm.json`
- Gate note:
  - `docs/governance/gates/gate_20260305_213556.md`
- 4-seed larger-subset means:
  - `R0`
    - `mse=0.003907888`
    - `total=53.154s`
  - `PHI3_K25_D36_L065`
    - `mse=0.003901309`
    - `total=48.472s`
  - `PHI3_K20_D36_L065`
    - `mse=0.003893818`
    - `total=50.990s`
- Reading:
  - simple budget compression did not replace the `K25` lead
  - `PHI_PHI_PHI v1` remains the best current family instance
  - the latest confirm now shows a runtime win over `R0`, but that claim still deserves one tighter fairness control because earlier batches differed materially on timing

## PHI_PHI_PHI Fairness Control
- Control doc:
  - `docs/research/controls/CTRL_0002_phi3_vs_r0_fairness.md`
- Analysis:
  - `results/analysis/ctrl0002_phi3_vs_r0_seedmajor.json`
- Gate note:
  - `docs/governance/gates/gate_20260305_214935.md`
- 4-seed seed-major means:
  - `R0`
    - `mse=0.003907888`
    - `total=44.916s`
    - `shells=1.0`
  - `PHI3_K25_D36_L065`
    - `mse=0.003901309`
    - `total=52.077s`
    - `shells=2.75`
    - `runtime_ratio_vs_r0=1.159`
- Reading:
  - `PHI_PHI_PHI v1` keeps the best quality/health position in the routed family
  - the runtime-win claim does not survive the stricter seed-major fairness control
  - `R0` remains the operational runtime preference until a new geometry branch clears that control
  - the next live mechanism question is whether shells need phase coupling or phase shifts

## Phase-Coupled Shell Confirm
- Increment doc:
  - `docs/research/increments/INC_0024_phase_coupled_shells.md`
- Analysis:
  - `results/analysis/inc0024_phase_shell_confirm.json`
- Gate note:
  - `docs/governance/gates/gate_20260305_222202.md`
- 4-seed seed-major means:
  - `R0`
    - `mse=0.003907888`
    - `total=46.405s`
    - `shells=1.0`
  - `PHASE_K25_C035`
    - `mse=0.003916993`
    - `total=53.423s`
    - `shells=2.5`
    - `runtime_ratio_vs_r0=1.151`
  - `PHI3_K25_D36_L065`
    - `mse=0.003917867`
    - `total=57.203s`
    - `shells=2.75`
- Reading:
  - continuous shell-phase coupling is a real routed-family improvement over the coarse `PHI_PHI_PHI` reference
  - the branch still misses the strict runtime gate vs `R0`
  - current best interpretation is that shell phase matters, but the phase law should become sparser or more discrete before the next confirm-worthy claim

## Hopf Chi-Axis Pilot
- Increment doc:
  - `docs/research/increments/INC_0028_hopf_chi_axis.md`
- Analysis:
  - `results/analysis/inc0028_hopf_chi_screen.json`
- Gate note:
  - `docs/governance/gates/gate_20260305_235825.md`
- Reading:
  - explicit `chi` reopened Hopf angular capacity
  - but neither `HOPF_CHI2_K25` nor `HOPF_CHI3_K25` beat pure `HOPF_K25_BASE`
  - `chi` alone is therefore not the missing route law

## Pure Hopf Confirm
- Increment doc:
  - `docs/research/increments/INC_0030_hopf_confirm.md`
- Analysis:
  - `results/analysis/inc0030_hopf_confirm.json`
- Gate note:
  - `docs/governance/gates/gate_20260306_001608.md`
- 4-seed means:
  - `HOPF_K25_BASE`
    - `mse=0.003896580`
    - `total=63.244s`
    - `shells=3.0`
    - `sectors=4.0`
  - `PHASE_K25_C035`
    - `mse=0.003904390`
    - `total=60.888s`
    - `shells=3.5`
    - `sectors=11.75`
  - `R0`
    - `mse=0.003907888`
    - `total=57.833s`
    - health fail on shell collapse
- Reading:
  - pure Hopf is now the routed-quality lead on the proxy
  - no healthy routed branch is yet the runtime lead
  - the next live geometry branch is a phi/Fibonacci lattice intended to widen Hopf capacity without washing out the quality signal

## First Phi/Fibonacci Screen
- Increment doc:
  - `docs/research/increments/INC_0029_phi_fibonacci_lattice.md`
- Analysis:
  - `results/analysis/inc0029_fib_screen.json`
- Gate note:
  - `docs/governance/gates/gate_20260306_004144.md`
- Reading:
  - first lattice implementation collapsed back to the same effective Hopf pair
  - no widening occurred in practice (`kchi=1`, `sectors=4`)
  - runtime got much worse
  - the phi/Fibonacci direction remains open, but the allocator must be recurrence-constrained rather than greedily fit

## Phi^2 Rung Forcing Screen
- Increment doc:
  - `docs/research/increments/INC_0031_phi2_rung_forcing.md`
- Analysis:
  - `results/analysis/inc0031_phi2_rung_screen.json`
- Gate note:
  - `docs/governance/gates/gate_20260306_010724.md`
- Screen means:
  - `HOPF_K25_BASE`
    - `test_mse_after=0.0038888`
    - `total_sec=81.824`
    - `eval_sectors=4.0`
  - `HOPF_PHI2_K25`
    - `test_mse_after=0.0039024`
    - `total_sec=115.481`
    - `eval_sectors=10.5`
    - `buckets=20.0`
    - `adaptive_chi_bins_used=2.0`
    - `adaptive_chi_bin_pmax=0.942`
  - `PHASE_K25_C035`
    - `test_mse_after=0.0039095`
    - `total_sec=71.515`
    - `eval_sectors=11.5`
  - `R0`
    - `test_mse_after=0.0039113`
    - `total_sec=59.113`
- Reading:
  - recurrence-constrained `phi^2` widening is a real geometry branch
  - it can widen Hopf substantially without collapsing quality
  - the current law is still too expensive and too `chi`-concentrated to promote

## Chart-Isometry Routing Pilot
- Increment doc:
  - `docs/research/increments/INC_0036_chart_isometry_route.md`
- Analysis:
  - `results/analysis/inc0036_chart_iso_screen.json`
- Gate:
  - `docs/governance/gates/gate_20260306_074531.md`
- 2-seed screen means:
  - `HOPF_PHI2_BAND`
    - `test_mse_after=0.003910130`
    - `total_sec=30.702`
    - `align_pair_mae=0.103762`
  - `PHASE_K25_C035`
    - `test_mse_after=0.003923775`
    - `total_sec=29.103`
    - `align_pair_mae=0.231601`
  - `HOPF_K25_ISO`
    - `test_mse_after=0.003926004`
    - `total_sec=42.004`
    - `align_pair_mae=0.000000`
  - `HOPF_K25_BASE`
    - `test_mse_after=0.003934246`
    - `total_sec=33.220`
    - `align_pair_mae=0.118315`
  - `R0`
    - `test_mse_after=0.003946221`
    - `total_sec=31.958`
- Reading:
  - pure chart isometry recovered global alignment exactly
  - pure chart isometry still over-compressed the route and paid too much runtime
  - the next credible transfer branch is an isometric base coordinate plus explicit widening, not pure isometry by itself

## Translated Complex-Key Translation
- Increment doc:
  - `docs/research/increments/INC_0056_product_complex_translation.md`
- Analyses:
  - `results/analysis/inc0056_product_complex_translation_screen.json`
  - `results/analysis/inc0056_product_complex_translation_confirm.json`
- Gate notes:
  - `docs/governance/gates/gate_20260306_131055.md`
  - `docs/governance/gates/gate_20260306_131507.md`
- 4-seed confirm means:
  - `DENSE_Q24`
    - `test_mse_after=0.004321788`
    - `test_top1_after=0.04867`
    - `retrieval_total_amortized_per_repeat_sec=0.6573`
    - `retrieval_candidate_fraction_mean=1.0`
  - `HOPF_RET_P1_Q24`
    - `test_mse_after=0.004324992`
    - `test_top1_after=0.04683`
    - `retrieval_total_amortized_per_repeat_sec=0.6685`
    - `retrieval_candidate_fraction_mean=0.3511`
  - `HOPF_RET_CPX_P1_Q24`
    - `test_mse_after=0.004324266`
    - `test_top1_after=0.04592`
    - `retrieval_total_amortized_per_repeat_sec=0.6129`
    - `retrieval_candidate_fraction_mean=0.2095`
    - `retrieval_bucket_fallback_rate=0.0000`
- Current reading:
  - the discrete complex / imaginary key survives translation into the routed retrieval harness
  - the branch materially improves translated addressing efficiency
  - the remaining weakness is a small top-1 penalty, which now looks like a recall/backfill problem rather than a routing-collapse problem

## Sparse Translated Dual-Anchor Broader Comparison
- Increment docs:
  - `docs/research/increments/INC_0116_product_phase_sparse_translation_dual_anchor_broader_comparison_packet.md`
  - `docs/research/increments/INC_0117_product_phase_sparse_translation_dual_anchor_broader_comparison.md`
- Artifacts:
  - `configs/packet_inc0116_product_phase_sparse_translation_dual_anchor_broader_comparison.json`
  - `results/analysis/inc0116_product_phase_sparse_translation_dual_anchor_broader_comparison_packet.json`
  - `results/analysis/inc0117_product_phase_sparse_translation_dual_anchor_broader_comparison.json`
  - `docs/reports/INC0116_PRODUCT_PHASE_SPARSE_TRANSLATION_DUAL_ANCHOR_BROADER_COMPARISON_PACKET.md`
  - `docs/reports/INC0117_PRODUCT_PHASE_SPARSE_TRANSLATION_DUAL_ANCHOR_BROADER_COMPARISON.md`
- Current reading:
  - lower-bank default routed point:
    `CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500`
    - `systems-only`
  - upper-bank default routed point:
    `CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000`
    - `quality-near systems promotion`
  - upper-bank bounded-backfill comparator remains explicit but optional-only:
    `CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T40000`
- Current conclusion:
  - the broader sparse translated hardware-side read is now explicit and
    reusable
  - lower bank remains systems-first
  - upper bank remains the near-frontier pressure point
  - downstream real-task work should inherit this packet instead of rebuilding
    route forks

## Lower-Bank Sparse-Event Update
- Increment docs:
  - `docs/research/increments/INC_0132_product_phase_sparse_event_translation_lower_bank_reference_reselection.md`
- Artifacts:
  - `results/analysis/inc0132_product_phase_sparse_event_translation_lower_bank_reference_reselection.json`
  - `docs/reports/INC0132_PRODUCT_PHASE_SPARSE_EVENT_TRANSLATION_LOWER_BANK_REFERENCE_RESELECTION.md`
- Current reading:
  - explicit lower-bank default route:
    `CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_CPX8_Q01_T2500`
    - `systems-only`
  - balanced lower-bank quality comparator:
    `CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_SBI030_CPX8_Q01_T2500`
  - quality-first lower-bank comparator:
    `CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_SBI080_CPX8_Q01_T2500`
  - historical bounded-backfill lower-bank route is now stale:
    `CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500`
- Current conclusion:
  - the old lower-bank default from the broader/task-side lineage should no
    longer be treated as the active carry-forward route
  - the next honest branch is contract refresh, not another lower-bank sparse
    translated mechanism search

## Sparse Translated Dual-Anchor Task-Side Extension
- Increment docs:
  - `docs/research/increments/INC_0118_product_phase_sparse_translation_dual_anchor_task_side_extension.md`
- Artifacts:
  - `results/analysis/inc0118_product_phase_sparse_translation_dual_anchor_task_side_extension.json`
  - `docs/reports/INC0118_PRODUCT_PHASE_SPARSE_TRANSLATION_DUAL_ANCHOR_TASK_SIDE_EXTENSION.md`
- Current reading:
  - the exact dual-anchor packet now extends directly onto the real-task side
  - default task-side routes stay fixed:
    - `DENSE_Q01_T2500`
    - `CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500`
    - `DENSE_Q01_T40000`
    - `CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000`
  - lower bank stays systems-only by default
  - upper bank stays quality-near systems promotion by default
  - the upper-bank bounded-backfill route remains optional comparator-only
- Current conclusion:
  - later task-side comparisons should now start from this fixed packet and
    report contract
  - the next honest branch is an explicit dual-anchor real-task comparison,
    not another packet rebuild

## Sparse Translated Dual-Anchor Explicit Real-Task Comparison
- Increment docs:
  - `docs/research/increments/INC_0119_product_phase_sparse_translation_dual_anchor_real_task_comparison.md`
- Artifacts:
  - `results/analysis/inc0119_product_phase_sparse_translation_dual_anchor_real_task_comparison.json`
  - `docs/reports/INC0119_PRODUCT_PHASE_SPARSE_TRANSLATION_DUAL_ANCHOR_REAL_TASK_COMPARISON.md`
- Current reading:
  - the first explicit LM-proxy real-task comparison now inherits the exact
    dual-anchor packet
  - lower-bank default routed point:
    `CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500`
    - `systems-only`
    - recommendation: carry as systems-only default
  - upper-bank default routed point:
    `CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000`
    - `quality-near systems promotion`
    - recommendation: carry as promoted real-task default
  - upper-bank bounded-backfill comparator remains explicit but optional-only:
    `CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T40000`
- Current conclusion:
  - the explicit real-task comparison is now fixed and reusable
  - future downstream task-side branches should inherit this comparison by
    default instead of rebuilding sparse translated route forks

## Sparse Translated Dual-Anchor Real-Task Carry-Forward
- Increment docs:
  - `docs/research/increments/INC_0120_product_phase_sparse_translation_dual_anchor_real_task_carry_forward.md`
- Artifacts:
  - `results/analysis/inc0120_product_phase_sparse_translation_dual_anchor_real_task_carry_forward.json`
  - `docs/reports/INC0120_PRODUCT_PHASE_SPARSE_TRANSLATION_DUAL_ANCHOR_REAL_TASK_CARRY_FORWARD.md`
- Current reading:
  - the explicit LM-proxy real-task comparison now has one downstream
    carry-forward contract
  - lower-bank downstream default:
    `CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500`
    - `systems-only`
    - recommendation: carry as systems-only default
  - upper-bank downstream default:
    `CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000`
    - `quality-near systems promotion`
    - recommendation: carry as promoted real-task default
  - upper-bank bounded-backfill comparator remains explicit but optional-only:
    `CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T40000`
- Current conclusion:
  - downstream real-task work now has one fixed carry-forward contract
  - the next honest branch is to turn that contract into one reusable packet
    manifest, not to rebuild sparse translated route forks again

## Sparse Translated Dual-Anchor Real-Task Packet Manifest
- Increment docs:
  - `docs/research/increments/INC_0121_product_phase_sparse_translation_dual_anchor_real_task_packet_manifest.md`
- Artifacts:
  - `results/analysis/inc0121_product_phase_sparse_translation_dual_anchor_real_task_packet_manifest.json`
  - `docs/reports/INC0121_PRODUCT_PHASE_SPARSE_TRANSLATION_DUAL_ANCHOR_REAL_TASK_PACKET_MANIFEST.md`
- Current reading:
  - the downstream LM-proxy real-task packet now exists as one exact reusable
    manifest
  - default route ids remain fixed:
    - `DENSE_Q01_T2500`
    - `CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500`
    - `DENSE_Q01_T40000`
    - `CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000`
  - upper-bank bounded-backfill comparator remains explicit but optional-only:
    `CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T40000`
  - lower-bank soft sparse route remains excluded by default:
    `CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T2500`
- Current conclusion:
  - downstream real-task work now has one exact packet manifest to inherit
  - the next honest branch is to use that packet on the next downstream
    real-task question, not to rebuild route forks again

## Sparse Translated Dual-Anchor Real-Task Downstream Extension
- Increment docs:
  - `docs/research/increments/INC_0122_product_phase_sparse_translation_dual_anchor_real_task_downstream_extension.md`
- Artifacts:
  - `results/analysis/inc0122_product_phase_sparse_translation_dual_anchor_real_task_downstream_extension.json`
  - `docs/reports/INC0122_PRODUCT_PHASE_SPARSE_TRANSLATION_DUAL_ANCHOR_REAL_TASK_DOWNSTREAM_EXTENSION.md`
- Current reading:
  - the downstream LM-proxy real-task packet now extends into one explicit
    downstream inheritance artifact
  - lower-bank downstream default routed point:
    `CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500`
    - `systems-only`
    - recommendation: carry as systems-only default
  - upper-bank downstream default routed point:
    `CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000`
    - `quality-near systems promotion`
    - recommendation: carry as promoted real-task default
  - upper-bank bounded-backfill comparator remains explicit but optional-only:
    `CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T40000`
- Current conclusion:
  - later downstream real-task branches should now start from this
    manifest-backed extension artifact
  - the next honest branch is an explicit downstream real-task comparison, not
    another packet rebuild

## Sparse Translated Dual-Anchor Real-Task Downstream Comparison
- Increment docs:
  - `docs/research/increments/INC_0123_product_phase_sparse_translation_dual_anchor_real_task_downstream_comparison.md`
- Artifacts:
  - `results/analysis/inc0123_product_phase_sparse_translation_dual_anchor_real_task_downstream_comparison.json`
  - `docs/reports/INC0123_PRODUCT_PHASE_SPARSE_TRANSLATION_DUAL_ANCHOR_REAL_TASK_DOWNSTREAM_COMPARISON.md`
- Current reading:
  - the explicit downstream LM-proxy real-task comparison is now fixed from
    the completed downstream extension artifact
  - lower-bank downstream default routed point:
    `CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500`
    - `systems-only`
    - recommendation: carry as systems-only default
  - upper-bank downstream default routed point:
    `CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000`
    - `quality-near systems promotion`
    - recommendation: carry as promoted real-task default
  - upper-bank bounded-backfill comparator remains explicit but optional-only:
    `CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T40000`
- Current conclusion:
  - downstream work now has one explicit comparison recommendation to inherit
  - the next honest branch is downstream carry-forward, not another extension
    or packet rebuild

## Sparse Translated Dual-Anchor Real-Task Downstream Carry-Forward
- Increment docs:
  - `docs/research/increments/INC_0124_product_phase_sparse_translation_dual_anchor_real_task_downstream_carry_forward.md`
- Artifacts:
  - `results/analysis/inc0124_product_phase_sparse_translation_dual_anchor_real_task_downstream_carry_forward.json`
  - `docs/reports/INC0124_PRODUCT_PHASE_SPARSE_TRANSLATION_DUAL_ANCHOR_REAL_TASK_DOWNSTREAM_CARRY_FORWARD.md`
- Current reading:
  - the explicit downstream LM-proxy real-task comparison now has one
    downstream carry-forward contract
  - lower-bank downstream default:
    `CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500`
    - `systems-only`
    - recommendation: carry as systems-only default
  - upper-bank downstream default:
    `CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000`
    - `quality-near systems promotion`
    - recommendation: carry as promoted real-task default
  - upper-bank bounded-backfill comparator remains explicit but optional-only:
    `CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T40000`
- Current conclusion:
  - downstream real-task work now has one explicit carry-forward contract
  - the next honest branch is to turn that contract into one reusable packet
    manifest, not to rebuild route forks again
