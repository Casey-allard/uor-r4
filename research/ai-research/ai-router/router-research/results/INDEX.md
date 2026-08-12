# Results Index

Use this file to map each run batch to parsed summaries and decision records.

## Durable Research Artifacts

- Prime admissibility canonical framework:
  - canonical note:
    `docs/research/prime_transport_canonical_framework.md`
  - boundary note:
    `docs/research/prime_transport_canonical_vs_downstream.md`
  - exact-system companion:
    `docs/research/prime_transport_exact_recursive_system.md`
  - exact-state runner:
    `tools/prime_transport/run_recursive_state_visibility.py`
  - exact-state results:
    `results/prime_transport_recursive_system/recursive_state_visibility_detail.csv`
    `results/prime_transport_recursive_system/recursive_state_visibility_summary.csv`
  - next preferred step:
    formalize and test the law for first internal split horizon, first visible
    split horizon, and their lag as a function of wheel lift and tuplet pattern

- Prime admissibility `C^2` backbone:
  - note:
    `docs/research/prime_transport_c2_backbone.md`
  - script:
    `tools/prime_transport/c2_shared_astar_unseen_wheel_test.py`
  - artifacts:
    `results/prime_transport_c2_backbone/c2_shared_astar_unseen_9699690_summary.csv`
    `results/prime_transport_c2_backbone/c2_shared_astar_unseen_9699690_eigenvalues.csv`
    `results/prime_transport_c2_backbone/c2_shared_astar_unseen_9699690_note.md`
  - result:
    shared `A_*` trained on `W = 2310, 30030, 510510` generalized to unseen
    `W = 9699690` with `err_star / err_test = 1.013071972668`

- Prime admissibility grouped-packet hypothesis:
  - note:
    `docs/research/prime_transport_grouped_packet_hypothesis.md`
  - scaffold:
    `tools/prime_transport/layer_packet_c2_scaffold.py`
  - dataset builder:
    `tools/prime_transport/build_layer_packet_dataset.py`
  - dataset schema note:
    `results/prime_transport_grouped_packets/README.md`
  - built datasets:
    `results/prime_transport_grouped_packets/prime_transport_layer_packet_dataset_W30030.csv`
    `results/prime_transport_grouped_packets/prime_transport_layer_packet_dataset_W510510.csv`
  - question:
    can small per-layer complex packets composed across depth recover or
    explain part of the empirical shared `C^2` transport backbone?

- Prime admissibility grouped-packet recovery stage 2:
  - note:
    `docs/research/prime_transport_grouped_packet_recovery_stage2.md`
  - runner:
    `tools/prime_transport/run_grouped_packet_family_recovery.py`
  - detail table:
    `results/prime_transport_grouped_packets/grouped_packet_family_recovery_detail.csv`
  - summary table:
    `results/prime_transport_grouped_packets/grouped_packet_family_recovery_summary.csv`

- Prime admissibility spin-first reset and bridge test:
  - note:
    `docs/research/prime_transport_spin_first_reset.md`
  - runner:
    `tools/prime_transport/run_spin_h_c2_bridge.py`
  - detail table:
    `results/prime_transport_spin_bridge/spin_h_c2_bridge_detail.csv`
  - summary table:
    `results/prime_transport_spin_bridge/spin_h_c2_bridge_summary.csv`
  - question:
    is the empirical shared `C^2` backbone better understood as a quotient of
    finite-horizon spin dynamics than as a direct composition of per-layer
    phase packets?

- Prime admissibility exact recursive system:
  - note:
    `docs/research/prime_transport_exact_recursive_system.md`
  - runner:
    `tools/prime_transport/run_recursive_state_visibility.py`
  - detail table:
    `results/prime_transport_recursive_system/recursive_state_visibility_detail.csv`
  - summary table:
    `results/prime_transport_recursive_system/recursive_state_visibility_summary.csv`
  - question:
    how should the exact recursive system be formalized and measured at the
    `(b, spin_H)` layer before any downstream quotient geometry is attempted?

- Prime admissibility exact-layer consolidated note:
  - canonical note:
    `docs/research/prime_transport_exact_layer_consolidated.md`
  - purpose:
    compact summary of the strongest current exact-layer mathematical picture
    tying together static phase-fiber-scale structure, predictive spin,
    visible-threshold mechanism, and the current residual beyond the static
    chart

- Prime admissibility threshold law:
  - note:
    `docs/research/prime_transport_threshold_law.md`
  - runner:
    `tools/prime_transport/run_threshold_law_table.py`
  - summary table:
    `results/prime_transport_recursive_system/threshold_law_summary.csv`
  - detail table:
    `results/prime_transport_recursive_system/threshold_law_detail.csv`
  - question:
    how do first internal split horizon, first visible split horizon, and their
    lag vary with wheel lift and tuplet pattern at the exact `(b, spin_H)`
    layer?

- Prime admissibility phase-fiber first-split readability:
  - note:
    `docs/research/prime_transport_phase_fiber_first_split_readability.md`
  - runner:
    `tools/prime_transport/analyze_phase_fiber_first_split.py`
  - class table:
    `results/prime_transport_recursive_system/visible_threshold_phase_fiber_mapped_first_split_classes.csv`
  - summary table:
    `results/prime_transport_recursive_system/visible_threshold_phase_fiber_readability_summary.csv`
  - score table:
    `results/prime_transport_recursive_system/visible_threshold_phase_fiber_readability_scores.csv`
  - question:
    are first visible splitting events already partially readable in the exact
    phase-fiber-scale chart `(b, phi, r)`, or is full spin still necessary?

- Prime admissibility phase-fiber versus spin:
  - canonical note:
    `docs/research/prime_transport_phase_fiber_vs_spin_canonical.md`
  - question:
    what is the current exact-layer relationship between the static chart
    `(b, phi, r)`, finite-horizon spin, and visible first-splitting events?

- Prime admissibility spin-minus-phase-fiber residual:
  - note:
    `docs/research/prime_transport_spin_minus_phase_fiber_residual.md`
  - runner:
    `tools/prime_transport/analyze_spin_phase_fiber_residual.py`
  - residual table:
    `results/prime_transport_recursive_system/visible_threshold_spin_minus_phase_fiber_residuals.csv`
  - summary table:
    `results/prime_transport_recursive_system/visible_threshold_spin_minus_phase_fiber_summary.csv`
  - score table:
    `results/prime_transport_recursive_system/visible_threshold_spin_minus_phase_fiber_scores.csv`
  - question:
    can the predictive structure carried by spin beyond the static chart be
    compressed into a small exact return-memory object?

- Prime admissibility return-grammar compression:
  - note:
    `docs/research/prime_transport_return_grammar_compression.md`
  - runner:
    `tools/prime_transport/analyze_return_grammar_compression.py`
  - candidate comparison:
    `results/prime_transport_recursive_system/visible_threshold_return_grammar_comparison.csv`
  - score table:
    `results/prime_transport_recursive_system/visible_threshold_return_grammar_scores.csv`
  - question:
    does there exist a small exact return-grammar object that captures most of
    the predictive distinctions currently carried by spin beyond the static
    chart?

- Prime admissibility minimal predictive state:
  - note:
    `docs/research/prime_transport_minimal_predictive_state.md`
  - comparison table:
    `results/prime_transport_recursive_system/visible_threshold_minimal_predictive_state_comparison.csv`
  - question:
    what is the smallest exact predictive state that improves over static chart
    reading while remaining meaningfully smaller than full spin?

- Prime admissibility routing abstraction:
  - note:
    `docs/research/prime_transport_routing_abstraction.md`
  - candidate table:
    `results/prime_transport_recursive_system/prime_transport_routing_state_candidates.csv`
  - question:
    which exact-layer state should serve as the first routing prototype target:
    static chart, minimal hybrid predictive state, or full spin?

- Prime admissibility `R_min` prototype spec:
  - note:
    `docs/research/prime_transport_rmin_prototype_spec.md`
  - prototype table:
    `results/prime_transport_recursive_system/prime_transport_rmin_prototype_table.csv`
  - question:
    how should the first exact-layer routing prototype based on
    `(b, phi, r, next_return_gap)` be specified before any runtime integration?

- Prime admissibility `R_min` offline evaluation:
  - note:
    `docs/research/prime_transport_rmin_offline_eval.md`
  - summary table:
    `results/prime_transport_recursive_system/prime_transport_rmin_offline_eval_summary.csv`
  - runner:
    `tools/prime_transport/run_rmin_offline_eval.py`
  - question:
    is `R_min` strong enough to justify a first routing prototype before any
    runtime integration?

- Prime admissibility mock router module spec:
  - note:
    `docs/research/prime_transport_mock_router_module_spec.md`
  - interface table:
    `results/prime_transport_recursive_system/prime_transport_mock_router_interface_table.csv`
  - question:
    what is the smallest offline module/API shape that can host `R_static`,
    `R_min`, and selective promotion toward `R_full`?

- Prime admissibility mock router module implementation:
  - note:
    `docs/research/prime_transport_mock_router_module_implementation.md`
  - module:
    `tools/prime_transport/mock_router_module.py`
  - demo:
    `tools/prime_transport/run_mock_router_module_demo.py`
  - demo artifact:
    `results/prime_transport_recursive_system/prime_transport_mock_router_demo.csv`

- Prime admissibility larger real-signal trial:
  - note:
    `docs/research/prime_transport_larger_real_signal_trial.md`
  - runner:
    `tools/prime_transport/run_prime_transport_larger_real_signal_trial.py`
  - summary table:
    `results/prime_transport_recursive_system/prime_transport_larger_real_signal_trial.csv`
  - question:
    does natural route reuse emerge for `base_gap` on the paired larger replay
    slice, and does the guarded fallback advantage survive on that same
    real-signal boundary?

- Prime admissibility final real-signal status:
  - note:
    `docs/research/prime_transport_real_signal_status.md`
  - purpose:
    freeze the current real-signal conclusion: `base_gap` is the correct first
    guarded real-signal policy because the validated win is fallback
    efficiency, while natural route reuse is still not established on the
    current replay-based benchmark boundary

- Prime admissibility router-memory layer:
  - note:
    `docs/research/prime_transport_router_memory_layer.md`
  - module:
    `tools/prime_transport/router_memory_layer.py`
  - demo:
    `tools/prime_transport/run_router_memory_layer_demo.py`
  - result artifact:
    `results/prime_transport_recursive_system/prime_transport_router_memory_layer_demo.csv`
  - question:
    can the validated routing policy function as a small structured
    read/write memory machine, with explicit update, query, and promotion,
    before any larger router-native architecture experiment is attempted?

- Prime admissibility router-memory task loop:
  - note:
    `docs/research/prime_transport_router_memory_task_loop.md`
  - driver:
    `tools/prime_transport/run_router_memory_task_loop.py`
  - result artifact:
    `results/prime_transport_recursive_system/prime_transport_router_memory_task_loop.csv`
  - question:
    can the router-memory layer support real state carry and later retrieval in
    a bounded multi-step task loop, not only in replay-style routing analyses?

- Prime admissibility router-memory promoted-query reduction:
  - note:
    `docs/research/prime_transport_router_memory_promoted_query_reduction.md`
  - runner:
    `tools/prime_transport/run_router_memory_promoted_query_reduction.py`
  - result artifact:
    `results/prime_transport_recursive_system/prime_transport_router_memory_promoted_query_reduction.csv`
  - question:
    can one tiny read-side refinement reduce promoted-query burden without
    materially harming carried-state retrieval quality or stability?

- Prime admissibility router-memory status:
  - note:
    `docs/research/prime_transport_router_memory_status.md`
  - purpose:
    freeze the branch-level architectural reading: the router-memory path is
    viable as a stateful memory substrate, but deeper-family promoted-query
    burden remains the main unsolved cost and the first cheap refinement has
    already been rejected

- Prime admissibility larger router-memory experiment:
  - task-family note:
    `docs/research/prime_transport_router_memory_record_family.md`
  - experiment note:
    `docs/research/prime_transport_larger_router_memory_experiment.md`
  - runner:
    `tools/prime_transport/run_larger_router_memory_experiment.py`
  - result artifact:
    `results/prime_transport_recursive_system/prime_transport_larger_router_memory_experiment.csv`
  - question:
    does the current router-memory architecture remain coherent on a richer
    multi-field state-carry task family without any read-path redesign?

- Prime admissibility multi-entity router-memory experiment:
  - task-family note:
    `docs/research/prime_transport_multi_entity_router_memory_family.md`
  - experiment note:
    `docs/research/prime_transport_multi_entity_router_memory_experiment.md`
  - runner:
    `tools/prime_transport/run_multi_entity_router_memory_experiment.py`
  - result artifact:
    `results/prime_transport_recursive_system/prime_transport_multi_entity_router_memory_experiment.csv`
  - question:
    can the unchanged router-memory architecture support exact bounded
    multi-entity state coordination, not only single-record memory carry?

- Prime admissibility router-memory readiness:
  - readiness note:
    `docs/research/prime_transport_router_memory_readiness.md`
  - next-prototype spec:
    `docs/research/prime_transport_router_memory_systems_prototype.md`
  - purpose:
    freeze the branch-level readiness decision and define the first larger
    bounded router-native systems prototype to attempt next

- Prime admissibility visible-threshold predictors:
  - note:
    `docs/research/prime_transport_visible_threshold_predictors.md`
  - runner:
    `tools/prime_transport/analyze_visible_threshold_predictors.py`
  - row analysis:
    `results/prime_transport_recursive_system/visible_threshold_predictor_rows.csv`
  - score table:
    `results/prime_transport_recursive_system/visible_threshold_predictor_scores.csv`
  - monotonicity table:
    `results/prime_transport_recursive_system/visible_threshold_monotonicity.csv`
  - question:
    which coarse exact-layer variables are actually predictive of the visible
    split threshold, and which naive law classes are already ruled out?

- Prime admissibility visible-threshold extension:
  - note:
    `docs/research/prime_transport_visible_threshold_extension.md`
  - added tuplets:
    `double_twins_p13 = [0,2,26,28]`
    `double_twins_p17 = [0,2,34,36]`
    `double_twins_p19 = [0,2,38,40]`
  - updated threshold summary:
    `results/prime_transport_recursive_system/threshold_law_summary.csv`
  - updated predictor scores:
    `results/prime_transport_recursive_system/visible_threshold_predictor_scores.csv`
  - question:
    after breaking the `|A|` versus `nu_p(A)` degeneracy, which exact-layer law
    classes for visible threshold remain plausible?

- Prime admissibility arrangement statistics:
  - note:
    `docs/research/prime_transport_arrangement_statistics.md`
  - arrangement stats CSV:
    `results/prime_transport_recursive_system/visible_threshold_arrangement_stats.csv`
  - updated score table:
    `results/prime_transport_recursive_system/visible_threshold_predictor_scores.csv`
  - question:
    does a small arrangement-sensitive local stencil statistic explain visible
    threshold variation better than the current coarse predictors?

- Prime admissibility arrangement isolation:
  - note:
    `docs/research/prime_transport_arrangement_isolation.md`
  - runner:
    `tools/prime_transport/analyze_arrangement_isolation.py`
  - fixed-family rows:
    `results/prime_transport_recursive_system/visible_threshold_arrangement_isolation_rows.csv`
  - fixed-family scores:
    `results/prime_transport_recursive_system/visible_threshold_arrangement_isolation_scores.csv`
  - question:
    when `p` and `nu_p(A)` are held fixed, which simple arrangement statistic
    best explains residual visible-threshold variation?

## Batch Template
- Batch ID:
- Config:
- Logs:
- Parsed files:
- Summary row range:
- Decision note:

## Batch B001
- Batch ID: `B001_smoke_single`
- Config: ad hoc fast-dev smoke command (`run_tag=smoke_single`)
- Logs: `results/raw/smoke_single.log`
- Parsed files: `results/parsed/smoke_single.json`
- Summary row range: `results/summary.csv` row with `log_file=smoke_single.log`
- Decision note: n/a

## Batch B002
- Batch ID: `B002_sweep_20260305_123420`
- Config: `configs/route_sweep.yaml`
- Logs: `results/raw/sweep_*.log`
- Parsed files: `results/parsed/sweep_*.json`
- Summary row range: rows with `log_file` prefix `sweep_` in `results/summary.csv`
- Decision note: `docs/governance/gates/gate_20260305_123420.md`

## Batch B003
- Batch ID: `B003_wikitext2_proxy_baseline`
- Config: `tools/prepare_wikitext2.py` + `tasks/wikitext2_proxy.py` + `tasks/dense_baseline.py`
- Logs: `results/raw/dense_baseline_wikitext2_proxy.json`
- Parsed files: n/a (non-router artifact)
- Summary row range: n/a
- Decision note: captured in `docs/reports/REAL_TASK_COMPARISON.md`

## Batch B004
- Batch ID: `B004_phase4d_validation_20260305_125915`
- Config: `configs/route_sweep_phase4d_validation.json`
- Logs: `results/raw/sweep_R0_*_20260305_125*.log`, `results/raw/sweep_R5_*_20260305_125*.log`
- Parsed files: matching `results/parsed/*.json`
- Summary row range: `results/summary.csv` where `run_tag` contains `_20260305_125`
- Decision note: `docs/governance/gates/gate_20260305_125915.md`

## Batch B005
- Batch ID: `B005_lm_proxy_ptb_baseline`
- Config: `tools/prepare_wikitext2.py --dataset auto` (resolved to PTB) + proxy + dense baseline
- Logs: `results/raw/dense_baseline_lm_proxy.json`
- Parsed files: n/a (non-router artifact)
- Summary row range: n/a
- Decision note: reflected in `docs/reports/REAL_TASK_COMPARISON.md`

## Batch B006
- Batch ID: `B006_phase4d_validation_non_fast`
- Config: `configs/route_sweep_phase4d_validation.json`
- Logs: `results/raw/sweep_R0_*_20260305_125*.log`, `results/raw/sweep_R5_*_20260305_125*.log`
- Parsed files: matching `results/parsed/*.json`
- Summary row range: rows where `run_tag` contains `_20260305_125`
- Decision note: `docs/governance/gates/gate_20260305_125915.md`

## Batch B007
- Batch ID: `B007_phase4d_dims_fast`
- Config: `configs/route_sweep_phase4d_dims_fast.json`
- Logs: `results/raw/sweep_R0_*_20260305_130*.log`, `results/raw/sweep_R5A_*_20260305_130*.log`, `results/raw/sweep_R5B_*_20260305_130*.log`, `results/raw/sweep_R5C_*_20260305_130*.log`
- Parsed files: matching `results/parsed/*.json`
- Summary row range: rows where `run_tag` contains `_20260305_130`
- Decision note: `docs/governance/gates/gate_20260305_130139.md`

## Batch B008
- Batch ID: `B008_inc0003_r0_vs_r5b_non_fast`
- Config: `configs/route_sweep_inc0003_r0_vs_r5b.json`
- Logs: `results/raw/sweep_R0_*_20260305_1303-1306.log`, `results/raw/sweep_R5B_*_20260305_1306-1308.log`
- Parsed files: matching `results/parsed/*.json`
- Summary row range: rows where `run_tag` contains `_20260305_1303`..`_20260305_1308` for routes `R0` and `R5B`
- Decision note: `docs/governance/gates/gate_20260305_130833.md`

## Batch B009
- Batch ID: `B009_inc0004_r0_vs_r5b_finalize4`
- Config: `configs/route_sweep_inc0004_r0_vs_r5b_finalize4.json`
- Logs: `results/raw/sweep_R0_*_20260305_131*.log`, `results/raw/sweep_R5B_*_20260305_131*.log`
- Parsed files: matching `results/parsed/*.json`
- Summary row range: rows where `run_tag` contains `_20260305_131` for routes `R0` and `R5B`
- Decision note: `docs/governance/gates/gate_20260305_131933.md`

## Batch B010
- Batch ID: `B010_inc0005_r5b_timepressure`
- Config: `configs/route_sweep_inc0005_r5b_timepressure.json`
- Logs: `results/raw/sweep_R5B_L*_finalize_seed*_{20260305_1324..1330}.log`
- Parsed files: matching `results/parsed/*.json`
- Summary row range: rows where `run_tag` contains `R5B_L` and `_20260305_13`
- Decision note: `docs/governance/gates/gate_20260305_133115.md`

## Batch B011
- Batch ID: `B011_inc0006_r5b_robustness_large_n`
- Config: `configs/route_sweep_inc0006_r5b_robustness.json`
- Logs: `results/raw/sweep_R0_*_20260305_1348..1355.log`, `results/raw/sweep_R5B_*_20260305_1356..1401.log`
- Parsed files: matching `results/parsed/*.json`
- Summary row range: rows where `run_tag` contains `_20260305_134`..`_20260305_140`
- Decision note: `docs/governance/gates/gate_20260305_140216.md`

## Batch B012
- Batch ID: `B012_inc0007_lm_proxy_smoke`
- Config: `tasks/router_proxy_eval.py` with `run_tag=proxy_cmp_r0` and `run_tag=proxy_cmp_r5b`
- Logs: `results/raw/proxy_cmp_r0.log`, `results/raw/proxy_cmp_r5b.log`
- Parsed files: `results/parsed/proxy_cmp_r0.json`, `results/parsed/proxy_cmp_r5b.json`
- Summary row range: rows with `run_tag in {proxy_cmp_r0, proxy_cmp_r5b}`
- Decision note: reflected in `docs/reports/REAL_TASK_COMPARISON.md`

## Batch B013
- Batch ID: `B013_inc0008_proxy_transfer_multiseed`
- Config: `configs/proxy_transfer_inc0008.json`
- Logs: `results/raw/inc0008_proxy_transfer_multiseed_R0_seed*.log`, `results/raw/inc0008_proxy_transfer_multiseed_R5B_seed*.log`
- Parsed files: matching `results/parsed/inc0008_proxy_transfer_multiseed_*.json`
- Analysis: `results/analysis/inc0008_proxy_transfer_multiseed.json`
- Summary row range: rows where `run_tag` contains `inc0008_proxy_transfer_multiseed`
- Decision note: `docs/governance/gates/gate_20260305_141648.md`

## Batch B014
- Batch ID: `B014_inc0009_proxy_stabilization_screen`
- Config: `configs/proxy_transfer_inc0009_screen.json`
- Logs: `results/raw/inc0009_proxy_stabilization_screen_*.log`
- Parsed files: matching `results/parsed/inc0009_proxy_stabilization_screen_*.json`
- Analysis: `results/analysis/inc0009_proxy_stabilization_screen.json`
- Summary row range: rows where `run_tag` contains `inc0009_proxy_stabilization_screen`
- Decision note: `docs/governance/gates/gate_20260305_144909.md`

## Batch B015
- Batch ID: `B015_inc0009_proxy_stabilization_confirm`
- Config: `configs/proxy_transfer_inc0009_confirm.json`
- Logs: `results/raw/inc0009_proxy_stabilization_confirm_*.log`
- Parsed files: matching `results/parsed/inc0009_proxy_stabilization_confirm_*.json`
- Analysis: `results/analysis/inc0009_proxy_stabilization_confirm.json`
- Summary row range: rows where `run_tag` contains `inc0009_proxy_stabilization_confirm`
- Decision note: `docs/governance/gates/gate_20260305_145544.md`

## Batch B016
- Batch ID: `B016_inc0010_adaptive_phase4d_confirm`
- Config: `configs/proxy_transfer_inc0010_adaptive_confirm.json`
- Logs: `results/raw/inc0010_adaptive_phase4d_confirm_*.log`
- Parsed files: matching `results/parsed/inc0010_adaptive_phase4d_confirm_*.json`
- Analysis: `results/analysis/inc0010_adaptive_phase4d_confirm.json`
- Summary row range: rows where `run_tag` contains `inc0010_adaptive_phase4d_confirm`
- Decision note: `docs/governance/gates/gate_20260305_151230.md`

## Batch B017
- Batch ID: `B017_inc0011_shell_activation_screen`
- Config: `configs/proxy_transfer_inc0011_shell_screen.json`
- Logs: `results/raw/inc0011_shell_activation_screen_*.log`
- Parsed files: matching `results/parsed/inc0011_shell_activation_screen_*.json`
- Analysis: `results/analysis/inc0011_shell_activation_screen.json`
- Summary row range: rows where `run_tag` contains `inc0011_shell_activation_screen`
- Decision note: `docs/governance/gates/gate_20260305_153422.md`

## Batch B018
- Batch ID: `B018_inc0011_shell_activation_confirm`
- Config: `configs/proxy_transfer_inc0011_shell_confirm.json`
- Logs: `results/raw/inc0011_shell_activation_confirm_*.log`
- Parsed files: matching `results/parsed/inc0011_shell_activation_confirm_*.json`
- Analysis: `results/analysis/inc0011_shell_activation_confirm.json`
- Summary row range: rows where `run_tag` contains `inc0011_shell_activation_confirm`
- Decision note: `docs/governance/gates/gate_20260305_153937.md`

## Batch B019
- Batch ID: `B019_inc0012_convergence_screen`
- Config: `configs/proxy_transfer_inc0012_convergence_screen.json`
- Logs: `results/raw/inc0012_convergence_screen_*.log`
- Parsed files: matching `results/parsed/inc0012_convergence_screen_*.json`
- Analysis: `results/analysis/inc0012_convergence_screen.json`
- Summary row range: rows where `run_tag` contains `inc0012_convergence_screen`
- Decision note: `docs/governance/gates/gate_20260305_160251.md`

## Batch B020
- Batch ID: `B020_inc0012_convergence_confirm`
- Config: `configs/proxy_transfer_inc0012_convergence_confirm.json`
- Logs: `results/raw/inc0012_convergence_confirm_*.log`
- Parsed files: matching `results/parsed/inc0012_convergence_confirm_*.json`
- Analysis: `results/analysis/inc0012_convergence_confirm.json`
- Summary row range: rows where `run_tag` contains `inc0012_convergence_confirm`
- Decision note: `docs/governance/gates/gate_20260305_160951.md`

## Batch B021
- Batch ID: `B021_inc0013_phase_diagram_screen`
- Config: `configs/proxy_transfer_inc0013_phase_diagram_screen.json`
- Logs: `results/raw/inc0013_phase_diagram_screen_*.log`
- Parsed files: matching `results/parsed/inc0013_phase_diagram_screen_*.json`
- Analysis: `results/analysis/inc0013_phase_diagram_screen.json`
- Summary row range: rows where `run_tag` contains `inc0013_phase_diagram_screen`
- Decision note: `docs/governance/gates/gate_20260305_163649.md`

## Batch B022
- Batch ID: `B022_inc0013_phase_diagram_confirm`
- Config: `configs/proxy_transfer_inc0013_phase_diagram_confirm.json`
- Logs: `results/raw/inc0013_phase_diagram_confirm_*.log`
- Parsed files: matching `results/parsed/inc0013_phase_diagram_confirm_*.json`
- Analysis: `results/analysis/inc0013_phase_diagram_confirm.json`
- Summary row range: rows where `run_tag` contains `inc0013_phase_diagram_confirm`
- Decision note: `docs/governance/gates/gate_20260305_164628.md`

## Batch B023
- Batch ID: `B023_inc0013_phase_diagram_confirm_strict_review`
- Config: strict seed-wise review of `results/analysis/inc0013_phase_diagram_confirm.json`
- Logs: reused `B022` logs
- Parsed files: reused `B022` parsed JSON
- Analysis: `results/analysis/inc0013_phase_diagram_confirm_strict.json`
- Summary row range: reused `B022` rows
- Decision note: `docs/governance/gates/gate_20260305_164628_strict.md`

## Batch B024
- Batch ID: `B024_inc0014_strict_robustness`
- Config: `configs/proxy_transfer_inc0014_strict_robustness.json`
- Logs: `results/raw/inc0014_strict_robustness_*.log`
- Parsed files: matching `results/parsed/inc0014_strict_robustness_*.json`
- Analysis: `results/analysis/inc0014_strict_robustness.json`
- Summary row range: rows where `run_tag` contains `inc0014_strict_robustness`
- Decision note: `docs/governance/gates/gate_20260305_171526.md`

## Batch B025
- Batch ID: `B025_inc0015_ridge_discrimination`
- Config: `configs/proxy_transfer_inc0015_ridge_discrimination.json`
- Logs: `results/raw/inc0015_ridge_discrimination_*.log`
- Parsed files: matching `results/parsed/inc0015_ridge_discrimination_*.json`
- Analysis: `results/analysis/inc0015_ridge_discrimination.json`
- Summary row range: rows where `run_tag` contains `inc0015_ridge_discrimination`
- Decision note: `docs/governance/gates/gate_20260305_174043.md`

## Batch B026
- Batch ID: `B026_inc0016_delta_confirm`
- Config: `configs/proxy_transfer_inc0016_delta_confirm.json`
- Logs: `results/raw/inc0016_delta_confirm_*.log`
- Parsed files: matching `results/parsed/inc0016_delta_confirm_*.json`
- Analysis: `results/analysis/inc0016_delta_confirm.json`
- Summary row range: rows where `run_tag` contains `inc0016_delta_confirm`
- Decision note: `docs/governance/gates/gate_20260305_175537.md`

## Batch B027
- Batch ID: `B027_inc0017_phi_ratio_screen`
- Config: `configs/proxy_transfer_inc0017_phi_ratio_screen.json`
- Logs: `results/raw/inc0017_phi_ratio_screen_*.log`
- Parsed files: matching `results/parsed/inc0017_phi_ratio_screen_*.json`
- Analysis: `results/analysis/inc0017_phi_ratio_screen.json`
- Summary row range: rows where `run_tag` contains `inc0017_phi_ratio_screen`
- Decision note: `docs/governance/gates/gate_20260305_181815.md`

## Batch B028
- Batch ID: `B028_inc0017_phi_ratio_confirm`
- Config: `configs/proxy_transfer_inc0017_phi_ratio_confirm.json`
- Logs: `results/raw/inc0017_phi_ratio_confirm_*.log`
- Parsed files: matching `results/parsed/inc0017_phi_ratio_confirm_*.json`
- Analysis: `results/analysis/inc0017_phi_ratio_confirm.json`
- Summary row range: rows where `run_tag` contains `inc0017_phi_ratio_confirm`
- Decision note: `docs/governance/gates/gate_20260305_183037.md`

## Batch B029
- Batch ID: `B029_inc0018_phi_delta_screen`
- Config: `configs/proxy_transfer_inc0018_phi_delta_screen.json`
- Logs: `results/raw/inc0018_phi_delta_screen_*.log`
- Parsed files: matching `results/parsed/inc0018_phi_delta_screen_*.json`
- Analysis: `results/analysis/inc0018_phi_delta_screen.json`
- Summary row range: rows where `run_tag` contains `inc0018_phi_delta_screen`
- Decision note: `docs/governance/gates/gate_20260305_184239.md`

## Batch B030
- Batch ID: `B030_inc0021_phi_ladder_screen`
- Config: `configs/proxy_transfer_inc0021_phi_ladder_screen.json`
- Logs: `results/raw/inc0021_phi_ladder_screen_*.log`
- Parsed files: matching `results/parsed/inc0021_phi_ladder_screen_*.json`
- Analysis: `results/analysis/inc0021_phi_ladder_screen.json`
- Summary row range: rows where `run_tag` contains `inc0021_phi_ladder_screen`
- Decision note: `docs/governance/gates/gate_20260305_202833.md`

## Batch B030
- Batch ID: `B030_inc0018_phi_delta_confirm`
- Config: `configs/proxy_transfer_inc0018_phi_delta_confirm.json`
- Logs: `results/raw/inc0018_phi_delta_confirm_*.log`
- Parsed files: matching `results/parsed/inc0018_phi_delta_confirm_*.json`
- Analysis: `results/analysis/inc0018_phi_delta_confirm.json`
- Summary row range: rows where `run_tag` contains `inc0018_phi_delta_confirm`
- Decision note: `docs/governance/gates/gate_20260305_185546.md`

## Batch B031
- Batch ID: `B031_inc0019_hybrid_screen`
- Config: `configs/proxy_transfer_inc0019_hybrid_screen.json`
- Logs: `results/raw/inc0019_hybrid_screen_*.log`
- Parsed files: matching `results/parsed/inc0019_hybrid_screen_*.json`
- Analysis: `results/analysis/inc0019_hybrid_screen.json`
- Summary row range: rows where `run_tag` contains `inc0019_hybrid_screen`
- Decision note: `docs/governance/gates/gate_20260305_191832.md`

## Batch B032
- Batch ID: `B032_ctrl0001_seedmajor_lead`
- Config: `configs/proxy_transfer_ctrl0001_seedmajor_lead.json`
- Logs: `results/raw/ctrl0001_seedmajor_lead_*.log`
- Parsed files: matching `results/parsed/ctrl0001_seedmajor_lead_*.json`
- Analysis: `results/analysis/ctrl0001_seedmajor_lead.json`
- Summary row range: rows where `run_tag` contains `ctrl0001_seedmajor_lead`
- Decision note: `docs/governance/gates/gate_20260305_192810.md`

## Batch B033
- Batch ID: `B033_inc0020_hybrid_rescue_screen`
- Config: `configs/proxy_transfer_inc0020_hybrid_rescue_screen.json`
- Logs: `results/raw/inc0020_hybrid_rescue_screen_*.log`
- Parsed files: matching `results/parsed/inc0020_hybrid_rescue_screen_*.json`
- Analysis: `results/analysis/inc0020_hybrid_rescue_screen.json`
- Summary row range: rows where `run_tag` contains `inc0020_hybrid_rescue_screen`
- Decision note: `docs/governance/gates/gate_20260305_195327.md`

## Batch B034
- Batch ID: `B034_inc0020_hybrid_rescue_confirm`
- Config: `configs/proxy_transfer_inc0020_hybrid_rescue_confirm.json`
- Logs: `results/raw/inc0020_hybrid_rescue_confirm_*.log`
- Parsed files: matching `results/parsed/inc0020_hybrid_rescue_confirm_*.json`
- Analysis: `results/analysis/inc0020_hybrid_rescue_confirm.json`
- Summary row range: rows where `run_tag` contains `inc0020_hybrid_rescue_confirm`
- Decision note: `docs/governance/gates/gate_20260305_200621.md`

## Batch B035
- Batch ID: `B035_inc0021_phi_ladder_screen`
- Config: `configs/proxy_transfer_inc0021_phi_ladder_screen.json`
- Logs: `results/raw/inc0021_phi_ladder_screen_*.log`
- Parsed files: matching `results/parsed/inc0021_phi_ladder_screen_*.json`
- Analysis: `results/analysis/inc0021_phi_ladder_screen.json`
- Summary row range: rows where `run_tag` contains `inc0021_phi_ladder_screen`
- Decision note: `docs/governance/gates/gate_20260305_202833.md`

## Batch B036
- Batch ID: `B036_inc0022_phi_log_screen`
- Config: `configs/proxy_transfer_inc0022_phi_log_screen.json`
- Logs: `results/raw/inc0022_phi_log_screen_*.log`
- Parsed files: matching `results/parsed/inc0022_phi_log_screen_*.json`
- Analysis: `results/analysis/inc0022_phi_log_screen.json`
- Summary row range: rows where `run_tag` contains `inc0022_phi_log_screen`
- Decision note: `docs/governance/gates/gate_20260305_205512.md`

## Batch B037
- Batch ID: `B037_inc0022_phi_log_confirm`
- Config: `configs/proxy_transfer_inc0022_phi_log_confirm.json`
- Logs: `results/raw/inc0022_phi_log_confirm_*.log`
- Parsed files: matching `results/parsed/inc0022_phi_log_confirm_*.json`
- Analysis: `results/analysis/inc0022_phi_log_confirm.json`
- Summary row range: rows where `run_tag` contains `inc0022_phi_log_confirm`
- Decision note: `docs/governance/gates/gate_20260305_210615.md`

## Batch B038
- Batch ID: `B038_inc0023_phi3_budget_screen`
- Config: `configs/proxy_transfer_inc0023_phi3_budget_screen.json`
- Logs: `results/raw/inc0023_phi3_budget_screen_*.log`
- Parsed files: matching `results/parsed/inc0023_phi3_budget_screen_*.json`
- Analysis: `results/analysis/inc0023_phi3_budget_screen.json`
- Summary row range: rows where `run_tag` contains `inc0023_phi3_budget_screen`
- Decision note: `docs/governance/gates/gate_20260305_212430.md`

## Batch B039
- Batch ID: `B039_inc0023_phi3_budget_confirm`
- Config: `configs/proxy_transfer_inc0023_phi3_budget_confirm.json`
- Logs: `results/raw/inc0023_phi3_budget_confirm_*.log`
- Parsed files: matching `results/parsed/inc0023_phi3_budget_confirm_*.json`
- Analysis: `results/analysis/inc0023_phi3_budget_confirm.json`
- Summary row range: rows where `run_tag` contains `inc0023_phi3_budget_confirm`
- Decision note: `docs/governance/gates/gate_20260305_213556.md`

## Batch B040
- Batch ID: `B040_ctrl0002_phi3_vs_r0_seedmajor`
- Config: `configs/proxy_transfer_ctrl0002_phi3_vs_r0_seedmajor.json`
- Logs: `results/raw/ctrl0002_phi3_vs_r0_seedmajor_*.log`
- Parsed files: matching `results/parsed/ctrl0002_phi3_vs_r0_seedmajor_*.json`
- Analysis: `results/analysis/ctrl0002_phi3_vs_r0_seedmajor.json`
- Summary row range: rows where `run_tag` contains `ctrl0002_phi3_vs_r0_seedmajor`
- Decision note: `docs/governance/gates/gate_20260305_214935.md`

## Batch B041
- Batch ID: `B041_inc0024_phase_shell_screen`
- Config: `configs/proxy_transfer_inc0024_phase_shell_screen.json`
- Logs: `results/raw/inc0024_phase_shell_screen_*.log`
- Parsed files: matching `results/parsed/inc0024_phase_shell_screen_*.json`
- Analysis: `results/analysis/inc0024_phase_shell_screen.json`
- Summary row range: rows where `run_tag` contains `inc0024_phase_shell_screen`
- Decision note: `docs/governance/gates/gate_20260305_221026.md`

## Batch B042
- Batch ID: `B042_inc0024_phase_shell_confirm`
- Config: `configs/proxy_transfer_inc0024_phase_shell_confirm.json`
- Logs: `results/raw/inc0024_phase_shell_confirm_*.log`
- Parsed files: matching `results/parsed/inc0024_phase_shell_confirm_*.json`
- Analysis: `results/analysis/inc0024_phase_shell_confirm.json`
- Summary row range: rows where `run_tag` contains `inc0024_phase_shell_confirm`
- Decision note: `docs/governance/gates/gate_20260305_222202.md`

## Batch B043
- Batch ID: `B043_inc0026_hopf_diag`
- Config: `configs/proxy_transfer_inc0026_hopf_diag.json`
- Logs: `results/raw/inc0026_hopf_diag_*.log`
- Parsed files: matching `results/parsed/inc0026_hopf_diag_*.json`
- Analysis: `results/analysis/inc0026_hopf_diag.json`
- Summary row range: rows where `run_tag` contains `inc0026_hopf_diag`
- Decision note: `docs/governance/gates/gate_20260305_230403.md`

## Batch B044
- Batch ID: `B044_inc0026_hopf_pilot_screen`
- Config: `configs/proxy_transfer_inc0026_hopf_pilot_screen.json`
- Logs: `results/raw/inc0026_hopf_pilot_screen_*.log`
- Parsed files: matching `results/parsed/inc0026_hopf_pilot_screen_*.json`
- Analysis: `results/analysis/inc0026_hopf_pilot_screen.json`
- Summary row range: rows where `run_tag` contains `inc0026_hopf_pilot_screen`
- Decision note: `docs/governance/gates/gate_20260305_231636.md`

## Batch B045
- Batch ID: `B045_inc0028_hopf_chi_screen`
- Config: `configs/proxy_transfer_inc0028_hopf_chi_screen.json`
- Logs: `results/raw/inc0028_hopf_chi_screen_*.log`
- Parsed files: matching `results/parsed/inc0028_hopf_chi_screen_*.json`
- Analysis: `results/analysis/inc0028_hopf_chi_screen.json`
- Summary row range: rows where `run_tag` contains `inc0028_hopf_chi_screen`
- Decision note: `docs/governance/gates/gate_20260305_235825.md`
- Note: ignore the earlier invalid first attempt at `gate_20260305_234515.md`; routed runs crashed before summary emission due an evaluator bug.

## Batch B046
- Batch ID: `B046_inc0030_hopf_confirm`
- Config: `configs/proxy_transfer_inc0030_hopf_confirm.json`
- Logs: `results/raw/inc0030_hopf_confirm_*.log`
- Parsed files: matching `results/parsed/inc0030_hopf_confirm_*.json`
- Analysis: `results/analysis/inc0030_hopf_confirm.json`
- Summary row range: rows where `run_tag` contains `inc0030_hopf_confirm`
- Decision note: `docs/governance/gates/gate_20260306_001608.md`

## Batch B047
- Batch ID: `B047_inc0029_fib_screen`
- Config: `configs/proxy_transfer_inc0029_fib_screen.json`
- Logs: `results/raw/inc0029_fib_screen_*.log`
- Parsed files: matching `results/parsed/inc0029_fib_screen_*.json`
- Analysis: `results/analysis/inc0029_fib_screen.json`
- Summary row range: rows where `run_tag` contains `inc0029_fib_screen`
- Decision note: `docs/governance/gates/gate_20260306_004144.md`

## Batch B048
- Batch ID: `B048_inc0031_phi2_rung_screen`
- Config: `configs/proxy_transfer_inc0031_phi2_rung_screen.json`
- Logs: `results/raw/inc0031_phi2_rung_screen_*.log`
- Parsed files: matching `results/parsed/inc0031_phi2_rung_screen_*.json`
- Analysis: `results/analysis/inc0031_phi2_rung_screen.json`
- Summary row range: rows where `run_tag` contains `inc0031_phi2_rung_screen`
- Decision note: `docs/governance/gates/gate_20260306_010724.md`

## Batch B049
- Batch ID: `B049_inc0032_phi2_gated_screen`
- Config: `configs/proxy_transfer_inc0032_phi2_gated_screen.json`
- Logs: `results/raw/inc0032_phi2_gated_screen_*.log`
- Parsed files: matching `results/parsed/inc0032_phi2_gated_screen_*.json`
- Analysis: `results/analysis/inc0032_phi2_gated_screen.json`
- Summary row range: rows where `run_tag` contains `inc0032_phi2_gated_screen`
- Decision note: `docs/governance/gates/gate_20260306_014339.md`

## Batch B050
- Batch ID: `B050_inc0033_phi2_band_screen`
- Config: `configs/proxy_transfer_inc0033_phi2_band_screen.json`
- Logs: `results/raw/inc0033_phi2_band_screen_*.log`
- Parsed files: matching `results/parsed/inc0033_phi2_band_screen_*.json`
- Analysis: `results/analysis/inc0033_phi2_band_screen.json`
- Summary row range: rows where `run_tag` contains `inc0033_phi2_band_screen`
- Decision note: `docs/governance/gates/gate_20260306_021036.md`

## Batch B051
- Batch ID: `B051_inc0034_blended_hopf_screen`
- Config: `configs/proxy_transfer_inc0034_blended_hopf_screen.json`
- Logs: `results/raw/inc0034_blended_hopf_screen_*.log`
- Parsed files: matching `results/parsed/inc0034_blended_hopf_screen_*.json`
- Analysis: `results/analysis/inc0034_blended_hopf_screen.json`
- Summary row range: rows where `run_tag` contains `inc0034_blended_hopf_screen`
- Decision note: `docs/governance/gates/gate_20260306_024928.md`

## Batch B052
- Batch ID: `B052_inc0035_alignment_diag_screen`
- Config: `configs/proxy_transfer_inc0035_alignment_diag_screen.json`
- Logs: `results/raw/inc0035_alignment_diag_screen_*.log`
- Parsed files: matching `results/parsed/inc0035_alignment_diag_screen_*.json`
- Analysis: `results/analysis/inc0035_alignment_diag_screen.json`
- Summary row range: rows where `run_tag` contains `inc0035_alignment_diag_screen`
- Decision note: `docs/governance/gates/gate_20260306_030909.md`

## Batch B053
- Batch ID: `B053_inc0035_shell_anchor_screen`
- Config: `configs/proxy_transfer_inc0035_shell_anchor_screen.json`
- Logs: `results/raw/inc0035_shell_anchor_screen_*.log`
- Parsed files: matching `results/parsed/inc0035_shell_anchor_screen_*.json`
- Analysis: `results/analysis/inc0035_shell_anchor_screen.json`
- Summary row range: rows where `run_tag` contains `inc0035_shell_anchor_screen`
- Decision note: `docs/governance/gates/gate_20260306_032618.md`

## Batch B054
- Batch ID: `B054_inc0036_chart_iso_screen`
- Config: `configs/proxy_transfer_inc0036_chart_iso_screen.json`
- Logs: `results/raw/inc0036_chart_iso_screen_*.log`
- Parsed files: matching `results/parsed/inc0036_chart_iso_screen_*.json`
- Analysis: `results/analysis/inc0036_chart_iso_screen.json`
- Summary row range: rows where `run_tag` contains `inc0036_chart_iso_screen`
- Decision note: `docs/governance/gates/gate_20260306_074531.md`

## Batch B055
- Batch ID: `B055_inc0037_isometric_band_screen`
- Config: `configs/proxy_transfer_inc0037_isometric_band_screen.json`
- Logs: `results/raw/inc0037_isometric_band_screen_*.log`
- Parsed files: matching `results/parsed/inc0037_isometric_band_screen_*.json`
- Analysis: `results/analysis/inc0037_isometric_band_screen.json`
- Summary row range: rows where `run_tag` contains `inc0037_isometric_band_screen`
- Decision note: `docs/governance/gates/gate_20260306_075923.md`

## Batch B056
- Batch ID: `B056_inc0038_bounded_band_screen`
- Config: `configs/proxy_transfer_inc0038_bounded_band_screen.json`
- Logs: `results/raw/inc0038_bounded_band_screen_*.log`
- Parsed files: matching `results/parsed/inc0038_bounded_band_screen_*.json`
- Analysis: `results/analysis/inc0038_bounded_band_screen.json`
- Summary row range: rows where `run_tag` contains `inc0038_bounded_band_screen`
- Decision note: `docs/governance/gates/gate_20260306_082106.md`

## Batch B057
- Batch ID: `B057_inc0039_route_memory_screen`
- Config: `configs/proxy_transfer_inc0039_route_memory_screen.json`
- Logs: `results/raw/inc0039_route_memory_screen_*.log`
- Parsed files: matching `results/parsed/inc0039_route_memory_screen_*.json`
- Analysis: `results/analysis/inc0039_route_memory_screen.json`
- Summary row range: rows where `run_tag` contains `inc0039_route_memory_screen`
- Decision note: `docs/governance/gates/gate_20260306_084204.md`

## Batch B058
- Batch ID: `B058_ctrl0003_hopf_frontier_confirm`
- Config: `configs/proxy_transfer_ctrl0003_hopf_frontier_confirm.json`
- Logs: `results/raw/ctrl0003_hopf_frontier_confirm_*.log`
- Parsed files: matching `results/parsed/ctrl0003_hopf_frontier_confirm_*.json`
- Analysis: `results/analysis/ctrl0003_hopf_frontier_confirm.json`
- Summary row range: rows where `run_tag` contains `ctrl0003_hopf_frontier_confirm`
- Decision note: `docs/governance/gates/gate_20260306_085323.md`

## Batch B059
- Batch ID: `B059_inc0040_cost_screen`
- Config: `configs/proxy_transfer_inc0040_cost_screen.json`
- Logs: `results/raw/inc0040_cost_screen_*.log`
- Parsed files: matching `results/parsed/inc0040_cost_screen_*.json`
- Analysis: `results/analysis/inc0040_cost_screen.json`
- Summary row range: rows where `run_tag` contains `inc0040_cost_screen`
- Decision note: `docs/governance/gates/gate_20260306_091429.md`

## Batch B060
- Batch ID: `B060_inc0040_cost_confirm`
- Config: `configs/proxy_transfer_inc0040_cost_confirm.json`
- Logs: `results/raw/inc0040_cost_confirm_*.log`
- Parsed files: matching `results/parsed/inc0040_cost_confirm_*.json`
- Analysis: `results/analysis/inc0040_cost_confirm.json`
- Summary row range: rows where `run_tag` contains `inc0040_cost_confirm`
- Decision note: `docs/governance/gates/gate_20260306_092503.md`

## Batch B061
- Batch ID: `B061_inc0041_cost_large_subset`
- Config: `configs/proxy_transfer_inc0041_cost_large_subset.json`
- Logs: `results/raw/inc0041_cost_large_subset_*.log`
- Parsed files: matching `results/parsed/inc0041_cost_large_subset_*.json`
- Analysis: `results/analysis/inc0041_cost_large_subset.json`
- Summary row range: rows where `run_tag` contains `inc0041_cost_large_subset`
- Decision note: `docs/governance/gates/gate_20260306_093641.md`

## Batch B050
- Batch ID: `B050_inc0042_timing_diag`
- Config: `configs/proxy_transfer_inc0042_timing_diag.json`
- Logs: `results/raw/inc0042_timing_diag_*.log`
- Parsed files: matching `results/parsed/inc0042_timing_diag_*.json`
- Analysis: `results/analysis/inc0042_timing_diag.json`
- Summary row range: rows where `run_tag` contains `inc0042_timing_diag`
- Decision note: `docs/governance/gates/gate_20260306_094708.md`

## Batch B051
- Batch ID: `B051_inc0043_train_route_static_screen`
- Config: `configs/proxy_transfer_inc0043_train_route_static_screen.json`
- Logs: `results/raw/inc0043_train_route_static_screen_*.log`
- Parsed files: matching `results/parsed/inc0043_train_route_static_screen_*.json`
- Analysis: `results/analysis/inc0043_train_route_static_screen.json`
- Summary row range: rows where `run_tag` contains `inc0043_train_route_static_screen`
- Decision note: `docs/governance/gates/gate_20260306_095825.md`

## Batch B052
- Batch ID: `B052_inc0043_train_route_static_confirm`
- Config: `configs/proxy_transfer_inc0043_train_route_static_confirm.json`
- Logs: `results/raw/inc0043_train_route_static_confirm_*.log`
- Parsed files: matching `results/parsed/inc0043_train_route_static_confirm_*.json`
- Analysis: `results/analysis/inc0043_train_route_static_confirm.json`
- Summary row range: rows where `run_tag` contains `inc0043_train_route_static_confirm`
- Decision note: `docs/governance/gates/gate_20260306_100530.md`

## Batch B053
- Batch ID: `B053_inc0044_static_chart_pressure_screen`
- Config: `configs/proxy_transfer_inc0044_static_chart_pressure_screen.json`
- Logs: `results/raw/inc0044_static_chart_pressure_screen_*.log`
- Parsed files: matching `results/parsed/inc0044_static_chart_pressure_screen_*.json`
- Analysis: `results/analysis/inc0044_static_chart_pressure_screen.json`
- Summary row range: rows where `run_tag` contains `inc0044_static_chart_pressure_screen`
- Decision note: `docs/governance/gates/gate_20260306_101427.md`

## Batch B054
- Batch ID: `B054_inc0044_static_chart_pressure_confirm`
- Config: `configs/proxy_transfer_inc0044_static_chart_pressure_confirm.json`
- Logs: `results/raw/inc0044_static_chart_pressure_confirm_*.log`
- Parsed files: matching `results/parsed/inc0044_static_chart_pressure_confirm_*.json`
- Analysis: `results/analysis/inc0044_static_chart_pressure_confirm.json`
- Summary row range: rows where `run_tag` contains `inc0044_static_chart_pressure_confirm`
- Decision note: `docs/governance/gates/gate_20260306_102058.md`

## Batch B055
- Batch ID: `B055_inc0045_static_chart_floor_screen`
- Config: `configs/proxy_transfer_inc0045_static_chart_floor_screen.json`
- Logs: `results/raw/inc0045_static_chart_floor_screen_*.log`
- Parsed files: matching `results/parsed/inc0045_static_chart_floor_screen_*.json`
- Analysis: `results/analysis/inc0045_static_chart_floor_screen.json`
- Summary row range: rows where `run_tag` contains `inc0045_static_chart_floor_screen`
- Decision note: `docs/governance/gates/gate_20260306_103538.md`

## Batch B056
- Batch ID: `B056_inc0045_static_chart_floor_confirm`
- Config: `configs/proxy_transfer_inc0045_static_chart_floor_confirm.json`
- Logs: `results/raw/inc0045_static_chart_floor_confirm_*.log`
- Parsed files: matching `results/parsed/inc0045_static_chart_floor_confirm_*.json`
- Analysis: `results/analysis/inc0045_static_chart_floor_confirm.json`
- Summary row range: rows where `run_tag` contains `inc0045_static_chart_floor_confirm`
- Decision note: `docs/governance/gates/gate_20260306_103811.md`

## Batch B057
- Batch ID: `B057_inc0046_static_scale_robustness_screen`
- Config: `configs/proxy_transfer_inc0046_static_scale_robustness_screen.json`
- Logs: `results/raw/inc0046_static_scale_robustness_screen_*.log`
- Parsed files: matching `results/parsed/inc0046_static_scale_robustness_screen_*.json`
- Analysis: `results/analysis/inc0046_static_scale_robustness_screen.json`
- Summary row range: rows where `run_tag` contains `inc0046_static_scale_robustness_screen`
- Decision note: `docs/governance/gates/gate_20260306_104728.md`

## Batch B058
- Batch ID: `B058_inc0046_static_scale_robustness_confirm`
- Config: `configs/proxy_transfer_inc0046_static_scale_robustness_confirm.json`
- Logs: `results/raw/inc0046_static_scale_robustness_confirm_*.log`
- Parsed files: matching `results/parsed/inc0046_static_scale_robustness_confirm_*.json`
- Analysis: `results/analysis/inc0046_static_scale_robustness_confirm.json`
- Summary row range: rows where `run_tag` contains `inc0046_static_scale_robustness_confirm`
- Decision note: `docs/governance/gates/gate_20260306_105119.md`

## Batch B059
- Batch ID: `B059_inc0047_near_full_proxy_scale_screen`
- Config: `configs/proxy_transfer_inc0047_near_full_proxy_scale_screen.json`
- Logs: `results/raw/inc0047_near_full_proxy_scale_screen_*.log`
- Parsed files: matching `results/parsed/inc0047_near_full_proxy_scale_screen_*.json`
- Analysis: `results/analysis/inc0047_near_full_proxy_scale_screen.json`
- Summary row range: rows where `run_tag` contains `inc0047_near_full_proxy_scale_screen`
- Decision note: `docs/governance/gates/gate_20260306_105627.md`

## Batch B060
- Batch ID: `B060_inc0047_near_full_proxy_scale_confirm`
- Config: `configs/proxy_transfer_inc0047_near_full_proxy_scale_confirm.json`
- Logs: `results/raw/inc0047_near_full_proxy_scale_confirm_*.log`
- Parsed files: matching `results/parsed/inc0047_near_full_proxy_scale_confirm_*.json`
- Analysis: `results/analysis/inc0047_near_full_proxy_scale_confirm.json`
- Summary row range: rows where `run_tag` contains `inc0047_near_full_proxy_scale_confirm`
- Decision note: `docs/governance/gates/gate_20260306_110140.md`

## Batch B063
- Batch ID: `B063_inc0048_retrieval_translation_screen`
- Config: `configs/proxy_transfer_inc0048_retrieval_translation_screen.json`
- Logs: `results/raw/inc0048_retrieval_translation_screen_*.log`
- Parsed files: matching `results/parsed/inc0048_retrieval_translation_screen_*.json`
- Analysis: `results/analysis/inc0048_retrieval_translation_screen.json`
- Summary row range: rows where `run_tag` contains `inc0048_retrieval_translation_screen`
- Decision note: `docs/governance/gates/gate_20260306_111959.md`

## Batch B064
- Batch ID: `B064_inc0049_retrieval_cost_rescue_screen`
- Config: `configs/proxy_transfer_inc0049_retrieval_cost_rescue_screen.json`
- Logs: `results/raw/inc0049_retrieval_cost_rescue_screen_*.log`
- Parsed files: matching `results/parsed/inc0049_retrieval_cost_rescue_screen_*.json`
- Analysis: `results/analysis/inc0049_retrieval_cost_rescue_screen.json`
- Summary row range: rows where `run_tag` contains `inc0049_retrieval_cost_rescue_screen`
- Decision note: `docs/governance/gates/gate_20260306_113201.md`

## Batch B065
- Batch ID: `B065_inc0051_retrieval_amortization_screen`
- Config: `configs/proxy_transfer_inc0051_retrieval_amortization_screen.json`
- Logs: `results/raw/inc0051_retrieval_amortization_screen_*.log`
- Parsed files: matching `results/parsed/inc0051_retrieval_amortization_screen_*.json`
- Analysis: `results/analysis/inc0051_retrieval_amortization_screen.json`
- Summary row range: rows where `run_tag` contains `inc0051_retrieval_amortization_screen`
- Decision note: `docs/governance/gates/gate_20260306_114654.md`

## Batch B066
- Batch ID: `B066_inc0052_retrieval_amortization_confirm`
- Config: `configs/proxy_transfer_inc0052_retrieval_amortization_confirm.json`
- Logs: `results/raw/inc0052_retrieval_amortization_confirm_*.log`
- Parsed files: matching `results/parsed/inc0052_retrieval_amortization_confirm_*.json`
- Analysis: `results/analysis/inc0052_retrieval_amortization_confirm.json`
- Summary row range: rows where `run_tag` contains `inc0052_retrieval_amortization_confirm`
- Decision note: `docs/governance/gates/gate_20260306_115931.md`

## Batch B096
- Batch ID: `B096_inc0050_dynamic_h4_screen`
- Config: `configs/proxy_transfer_inc0050_dynamic_h4_screen.json`
- Logs: `results/raw/inc0050_dynamic_h4_screen_*.log`
- Parsed files: matching `results/parsed/inc0050_dynamic_h4_screen_*.json`
- Analysis: `results/analysis/inc0050_dynamic_h4_screen.json`
- Summary row range: rows where `run_tag` contains `inc0050_dynamic_h4_screen`
- Decision note: `docs/governance/gates/gate_20260306_122447.md`

## Batch B097
- Batch ID: `B097_inc0050_dynamic_h4_confirm`
- Config: `configs/proxy_transfer_inc0050_dynamic_h4_confirm.json`
- Logs: `results/raw/inc0050_dynamic_h4_confirm_*.log`
- Parsed files: matching `results/parsed/inc0050_dynamic_h4_confirm_*.json`
- Analysis: `results/analysis/inc0050_dynamic_h4_confirm.json`
- Summary row range: rows where `run_tag` contains `inc0050_dynamic_h4_confirm`
- Decision note: `docs/governance/gates/gate_20260306_122733.md`

## Batch B098
- Batch ID: `B098_inc0054_tangent_flow_route_law_screen_invalid`
- Config: `configs/proxy_transfer_inc0054_tangent_flow_route_law_screen.json`
- Logs: `results/raw/inc0054_tangent_flow_route_law_screen_*.log`
- Analysis: `results/analysis/inc0054_tangent_flow_route_law_screen.json`
- Decision note: `docs/governance/gates/gate_20260306_124108.md`
- Note: first screen attempt was invalid because `STATIC_GLOBAL` failed to emit a summary after an evaluator bug.

## Batch B099
- Batch ID: `B099_inc0054_tangent_flow_route_law_screen`
- Config: `configs/proxy_transfer_inc0054_tangent_flow_route_law_screen.json`
- Logs: `results/raw/inc0054_tangent_flow_route_law_screen_*.log`
- Parsed files: matching `results/parsed/inc0054_tangent_flow_route_law_screen_*.json`
- Analysis: `results/analysis/inc0054_tangent_flow_route_law_screen.json`
- Summary row range: rows where `run_tag` contains `inc0054_tangent_flow_route_law_screen`
- Decision note: `docs/governance/gates/gate_20260306_124322.md`

## Batch B100
- Batch ID: `B100_inc0055_product_h4x4_retrieval_field_screen`
- Config: `configs/proxy_transfer_inc0055_product_h4x4_retrieval_field_screen.json`
- Logs: `results/raw/inc0055_product_h4x4_retrieval_field_screen_*.log`
- Parsed files: matching `results/parsed/inc0055_product_h4x4_retrieval_field_screen_*.json`
- Analysis: `results/analysis/inc0055_product_h4x4_retrieval_field_screen.json`
- Summary row range: rows where `run_tag` contains `inc0055_product_h4x4_retrieval_field_screen`
- Decision note: `docs/governance/gates/gate_20260306_125229.md`

## Batch B101
- Batch ID: `B101_inc0055_product_h4x4_retrieval_field_confirm`
- Config: `configs/proxy_transfer_inc0055_product_h4x4_retrieval_field_confirm.json`
- Logs: `results/raw/inc0055_product_h4x4_retrieval_field_confirm_*.log`
- Parsed files: matching `results/parsed/inc0055_product_h4x4_retrieval_field_confirm_*.json`
- Analysis: `results/analysis/inc0055_product_h4x4_retrieval_field_confirm.json`
- Summary row range: rows where `run_tag` contains `inc0055_product_h4x4_retrieval_field_confirm`
- Decision note: `docs/governance/gates/gate_20260306_125455.md`

## Batch B102
- Batch ID: `B102_inc0056_product_complex_translation_screen`
- Config: `configs/proxy_transfer_inc0056_product_complex_translation_screen.json`
- Logs: `results/raw/inc0056_product_complex_translation_screen_*.log`
- Parsed files: matching `results/parsed/inc0056_product_complex_translation_screen_*.json`
- Analysis: `results/analysis/inc0056_product_complex_translation_screen.json`
- Summary row range: rows where `run_tag` contains `inc0056_product_complex_translation_screen`
- Decision note: `docs/governance/gates/gate_20260306_131055.md`

## Batch B103
- Batch ID: `B103_inc0056_product_complex_translation_confirm`
- Config: `configs/proxy_transfer_inc0056_product_complex_translation_confirm.json`
- Logs: `results/raw/inc0056_product_complex_translation_confirm_*.log`
- Parsed files: matching `results/parsed/inc0056_product_complex_translation_confirm_*.json`
- Analysis: `results/analysis/inc0056_product_complex_translation_confirm.json`
- Summary row range: rows where `run_tag` contains `inc0056_product_complex_translation_confirm`
- Decision note: `docs/governance/gates/gate_20260306_131507.md`

## Batch B104
- Batch ID: `B104_inc0057_product_complex_backfill_smallbucket_screen`
- Config: `configs/proxy_transfer_inc0057_product_complex_backfill_smallbucket_screen.json`
- Logs: `results/raw/inc0057_product_complex_backfill_smallbucket_screen_*.log`
- Parsed files: matching `results/parsed/inc0057_product_complex_backfill_smallbucket_screen_*.json`
- Analysis: `results/analysis/inc0057_product_complex_backfill_smallbucket_screen.json`
- Summary row range: rows where `run_tag` contains `inc0057_product_complex_backfill_smallbucket_screen`
- Decision note: `docs/governance/gates/gate_20260306_135217.md`
- Note: the earlier mixed selective screen was stopped after low-margin backfill showed pathological trigger frequency and cost; its partial logs remain in `results/raw/` for forensics only.

## Batch B105
- Batch ID: `B105_inc0058_product_complex_rerank_screen`
- Config: `configs/proxy_transfer_inc0058_product_complex_rerank_screen.json`
- Logs: `results/raw/inc0058_product_complex_rerank_screen_*.log`
- Parsed files: matching `results/parsed/inc0058_product_complex_rerank_screen_*.json`
- Analysis: `results/analysis/inc0058_product_complex_rerank_screen.json`
- Summary row range: rows where `run_tag` contains `inc0058_product_complex_rerank_screen`
- Decision note: `docs/governance/gates/gate_20260306_140424.md`

## Batch B106
- Batch ID: `B106_inc0063_phase_transport_screen`
- Config: `configs/proxy_transfer_inc0063_phase_transport_screen.json`
- Logs: `results/raw/inc0063_phase_transport_screen_*.log`
- Parsed files: matching `results/parsed/inc0063_phase_transport_screen_*.json`
- Analysis:
  - `results/analysis/inc0063_phase_transport_screen.json`
  - `results/analysis/inc0063_phase_transport_address_diff.json`
- Summary row range: rows where `run_tag` contains `inc0063_phase_transport_screen`
- Decision notes:
  - `docs/governance/gates/gate_20260310_235845.md`
  - `docs/governance/gates/gate_20260311_000712.md`
- Note: the first RR-063 screen failed because the harness was not forwarding `phase_transport_lambda` into `optimize_chart`; use the successful rerun plus the address-diff audit as canonical evidence.

## Batch B107
- Batch ID: `B107_inc0062_hopf_base_screen_corrected`
- Config: `configs/proxy_transfer_inc0062_hopf_base_screen_corrected.json`
- Logs: `results/raw/inc0062_hopf_base_screen_corrected_*.log`
- Parsed files: matching `results/parsed/inc0062_hopf_base_screen_corrected_*.json`
- Analysis: `results/analysis/inc0062_hopf_base_screen_corrected.json`
- Summary row range: rows where `run_tag` contains `inc0062_hopf_base_screen_corrected`
- Decision note: `docs/governance/gates/gate_20260311_101015.md`

## Batch B108
- Batch ID: `B108_inc0062_hopf_base_confirm_corrected`
- Config: `configs/proxy_transfer_inc0062_hopf_base_confirm_corrected.json`
- Logs: `results/raw/inc0062_hopf_base_confirm_corrected_*.log`
- Parsed files: matching `results/parsed/inc0062_hopf_base_confirm_corrected_*.json`
- Analysis: `results/analysis/inc0062_hopf_base_confirm_corrected.json`
- Summary row range: rows where `run_tag` contains `inc0062_hopf_base_confirm_corrected`
- Decision note: `docs/governance/gates/gate_20260311_101213.md`

## Batch B109
- Batch ID: `B109_inc0063_phase_transport_screen_corrected`
- Config: `configs/proxy_transfer_inc0063_phase_transport_screen_corrected.json`
- Logs: `results/raw/inc0063_phase_transport_screen_corrected_*.log`
- Parsed files: matching `results/parsed/inc0063_phase_transport_screen_corrected_*.json`
- Analysis:
  - `results/analysis/inc0063_phase_transport_screen_corrected.json`
  - `results/analysis/inc0063_phase_transport_address_diff_corrected.json`
- Summary row range: rows where `run_tag` contains `inc0063_phase_transport_screen_corrected`
- Decision note: `docs/governance/gates/gate_20260311_101344.md`

## Batch B110
- Batch ID: `B110_inc0064_coupled_complex_phase_screen_corrected`
- Config: `configs/proxy_transfer_inc0064_coupled_complex_phase_screen.json`
- Logs: `results/raw/inc0064_coupled_complex_phase_screen_corrected_*.log`
- Parsed files: matching `results/parsed/inc0064_coupled_complex_phase_screen_corrected_*.json`
- Analysis:
  - `results/analysis/inc0064_coupled_complex_phase_screen_corrected.json`
  - `results/analysis/inc0064_coupled_complex_phase_address_diff_corrected.json`
- Summary row range: rows where `run_tag` contains `inc0064_coupled_complex_phase_screen_corrected`
- Decision note: `docs/governance/gates/gate_20260311_101607.md`

## Batch B111
- Batch ID: `B111_inc0065_product_phase_field_screen`
- Config: `configs/proxy_transfer_inc0065_product_phase_field_screen.json`
- Logs: `results/raw/inc0065_product_phase_field_screen_*.log`
- Parsed files: matching `results/parsed/inc0065_product_phase_field_screen_*.json`
- Analysis:
  - `results/analysis/inc0065_product_phase_field_screen.json`
  - `results/analysis/inc0065_product_phase_field_address_diff.json`
- Summary row range: rows where `run_tag` contains `inc0065_product_phase_field_screen`
- Decision note: `docs/governance/gates/gate_20260311_105034.md`

## Batch B112
- Batch ID: `B112_inc0065_product_phase_field_confirm`
- Config: `configs/proxy_transfer_inc0065_product_phase_field_confirm.json`
- Logs: `results/raw/inc0065_product_phase_field_confirm_*.log`
- Parsed files: matching `results/parsed/inc0065_product_phase_field_confirm_*.json`
- Analysis:
  - `results/analysis/inc0065_product_phase_field_confirm.json`
  - `results/analysis/inc0065_product_phase_field_confirm_address_diff.json`
- Summary row range: rows where `run_tag` contains `inc0065_product_phase_field_confirm`
- Decision note: `docs/governance/gates/gate_20260311_110024.md`

## Batch B113
- Batch ID: `B113_inc0066_spectral_route_audit_seed0`
- Config: `configs/proxy_transfer_inc0065_product_phase_field_confirm.json` reused as route set for spectral audit
- Logs: none (tool writes analysis directly)
- Parsed files: n/a
- Analysis:
  - `results/analysis/inc0065_product_phase_field_confirm_spectral_seed0.json`
- Summary row range: n/a
- Decision note: reflected in `docs/DECISIONS.md`

## Batch B114
- Batch ID: `B114_inc0066_spectral_route_operator_screen`
- Config: `configs/spectral_route_inc0066_screen.json`
- Logs: `results/raw/inc0066_spectral_route_operator_screen_seed*.log`
- Parsed files: n/a
- Analysis:
  - `results/analysis/inc0066_spectral_route_operator_screen_seed0.json`
  - `results/analysis/inc0066_spectral_route_operator_screen_seed1.json`
  - `results/analysis/inc0066_spectral_route_operator_screen.json`
- Summary row range: n/a
- Decision note: `docs/governance/gates/gate_20260311_112010.md`

## Batch B115
- Batch ID: `B115_inc0066_spectral_route_operator_confirm`
- Config: `configs/spectral_route_inc0066_confirm.json`
- Logs: `results/raw/inc0066_spectral_route_operator_confirm_seed*.log`
- Parsed files: n/a
- Analysis:
  - `results/analysis/inc0066_spectral_route_operator_confirm_seed0.json`
  - `results/analysis/inc0066_spectral_route_operator_confirm_seed1.json`
  - `results/analysis/inc0066_spectral_route_operator_confirm_seed2.json`
  - `results/analysis/inc0066_spectral_route_operator_confirm_seed3.json`
  - `results/analysis/inc0066_spectral_route_operator_confirm.json`
- Summary row range: n/a
- Decision note: `docs/governance/gates/gate_20260311_112215.md`

## Batch B116
- Batch ID: `B116_inc0067_spectral_signal_probes_screen`
- Config: `configs/spectral_signal_inc0067_screen.json`
- Logs: `results/raw/inc0067_spectral_signal_probes_screen_seed*.log`
- Parsed files: n/a
- Analysis:
  - `results/analysis/inc0067_spectral_signal_probes_screen_seed0.json`
  - `results/analysis/inc0067_spectral_signal_probes_screen_seed1.json`
  - `results/analysis/inc0067_spectral_signal_probes_screen.json`
- Summary row range: n/a
- Decision note: `docs/governance/gates/gate_20260311_113729.md`

## Batch B117
- Batch ID: `B117_inc0067_spectral_signal_probes_confirm`
- Config: `configs/spectral_signal_inc0067_confirm.json`
- Logs: `results/raw/inc0067_spectral_signal_probes_confirm_seed*.log`
- Parsed files: n/a
- Analysis:
  - `results/analysis/inc0067_spectral_signal_probes_confirm_seed0.json`
  - `results/analysis/inc0067_spectral_signal_probes_confirm_seed1.json`
  - `results/analysis/inc0067_spectral_signal_probes_confirm_seed2.json`
  - `results/analysis/inc0067_spectral_signal_probes_confirm_seed3.json`
  - `results/analysis/inc0067_spectral_signal_probes_confirm.json`
- Summary row range: n/a
- Decision note: `docs/governance/gates/gate_20260311_114258.md`

## Batch B118
- Batch ID: `B118_inc0068_spectral_residual_task_signals_screen`
- Config: `configs/spectral_residual_inc0068_screen.json`
- Logs: `results/raw/inc0068_spectral_residual_task_signals_screen_seed*.log`
- Parsed files: n/a
- Analysis:
  - `results/analysis/inc0068_spectral_residual_task_signals_screen_seed0.json`
  - `results/analysis/inc0068_spectral_residual_task_signals_screen_seed1.json`
  - `results/analysis/inc0068_spectral_residual_task_signals_screen.json`
- Summary row range: n/a
- Decision note: `docs/governance/gates/gate_20260311_122236.md`

## Batch B119
- Batch ID: `B119_inc0069_product_phase_translation_screen`
- Config: `configs/proxy_transfer_inc0069_product_phase_translation_screen.json`
- Logs: `results/raw/inc0069_product_phase_translation_screen_*.log`
- Parsed files: matching `results/parsed/inc0069_product_phase_translation_screen_*.json`
- Analysis:
  - `results/analysis/inc0069_product_phase_translation_screen.json`
- Summary row range: rows where `run_tag` contains `inc0069_product_phase_translation_screen`
- Decision note: `docs/governance/gates/gate_20260311_124011.md`

## Batch B120
- Batch ID: `B120_inc0069_product_phase_translation_confirm`
- Config: `configs/proxy_transfer_inc0069_product_phase_translation_confirm.json`
- Logs: `results/raw/inc0069_product_phase_translation_confirm_*.log`
- Parsed files: matching `results/parsed/inc0069_product_phase_translation_confirm_*.json`
- Analysis:
  - `results/analysis/inc0069_product_phase_translation_confirm.json`
- Summary row range: rows where `run_tag` contains `inc0069_product_phase_translation_confirm`
- Decision note: `docs/governance/gates/gate_20260311_124616.md`

## Batch B121
- Batch ID: `B121_inc0070_product_phase_translation_rescue_screen`
- Config: `configs/proxy_transfer_inc0070_product_phase_translation_rescue_screen.json`
- Logs: `results/raw/inc0070_product_phase_translation_rescue_screen_*.log`
- Parsed files: matching `results/parsed/inc0070_product_phase_translation_rescue_screen_*.json`
- Analysis:
  - `results/analysis/inc0070_product_phase_translation_rescue_screen.json`
- Summary row range: rows where `run_tag` contains `inc0070_product_phase_translation_rescue_screen`
- Decision note: `docs/governance/gates/gate_20260311_130226.md`

## Batch B122
- Batch ID: `B122_inc0071_product_phase_translation_secondary_keys_screen`
- Config: `configs/proxy_transfer_inc0071_product_phase_translation_secondary_keys_screen.json`
- Logs: `results/raw/inc0071_product_phase_translation_secondary_keys_screen_*.log`
- Parsed files: matching `results/parsed/inc0071_product_phase_translation_secondary_keys_screen_*.json`
- Analysis:
  - `results/analysis/inc0071_product_phase_translation_secondary_keys_screen.json`
- Summary row range: rows where `run_tag` contains `inc0071_product_phase_translation_secondary_keys_screen`
- Decision note: `docs/governance/gates/gate_20260311_131450.md`

## Batch B123
- Batch ID: `B123_inc0071_product_phase_translation_secondary_keys_confirm`
- Config: `configs/proxy_transfer_inc0071_product_phase_translation_secondary_keys_confirm.json`
- Logs: `results/raw/inc0071_product_phase_translation_secondary_keys_confirm_*.log`
- Parsed files: matching `results/parsed/inc0071_product_phase_translation_secondary_keys_confirm_*.json`
- Analysis:
  - `results/analysis/inc0071_product_phase_translation_secondary_keys_confirm.json`
- Summary row range: rows where `run_tag` contains `inc0071_product_phase_translation_secondary_keys_confirm`
- Decision note: `docs/governance/gates/gate_20260311_132141.md`

## Batch B124
- Batch ID: `B124_inc0072_product_phase_translation_secondary_key_cost_rescue_screen`
- Config: `configs/proxy_transfer_inc0072_product_phase_translation_secondary_key_cost_rescue_screen.json`
- Logs: `results/raw/inc0072_product_phase_translation_secondary_key_cost_rescue_screen_*.log`
- Parsed files: matching `results/parsed/inc0072_product_phase_translation_secondary_key_cost_rescue_screen_*.json`
- Analysis:
  - `results/analysis/inc0072_product_phase_translation_secondary_key_cost_rescue_screen.json`
- Summary row range: rows where `run_tag` contains `inc0072_product_phase_translation_secondary_key_cost_rescue_screen`
- Decision note: `docs/governance/gates/gate_20260311_133356.md`

## Batch B125
- Batch ID: `B125_inc0072_product_phase_translation_secondary_key_cost_rescue_confirm`
- Config: `configs/proxy_transfer_inc0072_product_phase_translation_secondary_key_cost_rescue_confirm.json`
- Logs: `results/raw/inc0072_product_phase_translation_secondary_key_cost_rescue_confirm_*.log`
- Parsed files: matching `results/parsed/inc0072_product_phase_translation_secondary_key_cost_rescue_confirm_*.json`
- Analysis:
  - `results/analysis/inc0072_product_phase_translation_secondary_key_cost_rescue_confirm.json`
- Summary row range: rows where `run_tag` contains `inc0072_product_phase_translation_secondary_key_cost_rescue_confirm`
- Decision note: `docs/governance/gates/gate_20260311_133731.md`

## Batch B126
- Batch ID: `B126_inc0073_product_phase_translation_secondary_key_large_load_screen`
- Config: `configs/proxy_transfer_inc0073_product_phase_translation_secondary_key_large_load_screen.json`
- Logs: `results/raw/inc0073_product_phase_translation_secondary_key_large_load_screen_*.log`
- Parsed files: matching `results/parsed/inc0073_product_phase_translation_secondary_key_large_load_screen_*.json`
- Analysis:
  - `results/analysis/inc0073_product_phase_translation_secondary_key_large_load_screen.json`
- Summary row range: rows where `run_tag` contains `inc0073_product_phase_translation_secondary_key_large_load_screen`
- Decision note: `docs/governance/gates/gate_20260311_134522.md`

## Batch B127
- Batch ID: `B127_inc0073_product_phase_translation_secondary_key_large_load_confirm`
- Config: `configs/proxy_transfer_inc0073_product_phase_translation_secondary_key_large_load_confirm.json`
- Logs: `results/raw/inc0073_product_phase_translation_secondary_key_large_load_confirm_*.log`
- Parsed files: matching `results/parsed/inc0073_product_phase_translation_secondary_key_large_load_confirm_*.json`
- Analysis:
  - `results/analysis/inc0073_product_phase_translation_secondary_key_large_load_confirm.json`
- Summary row range: rows where `run_tag` contains `inc0073_product_phase_translation_secondary_key_large_load_confirm`
- Decision note: `docs/governance/gates/gate_20260311_135226.md`

## Batch B128
- Batch ID: `B128_inc0074_product_phase_translation_dense_frontier_screen`
- Config: `configs/proxy_transfer_inc0074_product_phase_translation_dense_frontier_screen.json`
- Logs: `results/raw/inc0074_product_phase_translation_dense_frontier_screen_*.log`
- Parsed files: matching `results/parsed/inc0074_product_phase_translation_dense_frontier_screen_*.json`
- Analysis:
  - `results/analysis/inc0074_product_phase_translation_dense_frontier_screen.json`
- Summary row range: rows where `run_tag` contains `inc0074_product_phase_translation_dense_frontier_screen`
- Decision note: `docs/governance/gates/gate_20260311_140824.md`

## Batch B129
- Batch ID: `B129_inc0074_product_phase_translation_dense_frontier_confirm`
- Config: `configs/proxy_transfer_inc0074_product_phase_translation_dense_frontier_confirm.json`
- Logs: `results/raw/inc0074_product_phase_translation_dense_frontier_confirm_*.log`
- Parsed files: matching `results/parsed/inc0074_product_phase_translation_dense_frontier_confirm_*.json`
- Analysis:
  - `results/analysis/inc0074_product_phase_translation_dense_frontier_confirm.json`
- Summary row range: rows where `run_tag` contains `inc0074_product_phase_translation_dense_frontier_confirm`
- Decision note: `docs/governance/gates/gate_20260311_141705.md`

## Batch B130
- Batch ID: `B130_inc0075_product_phase_translation_dense_quality_recovery_screen`
- Config: `configs/proxy_transfer_inc0075_product_phase_translation_dense_quality_recovery_screen.json`
- Logs: `results/raw/inc0075_product_phase_translation_dense_quality_recovery_screen_*.log`
- Parsed files: matching `results/parsed/inc0075_product_phase_translation_dense_quality_recovery_screen_*.json`
- Analysis:
  - `results/analysis/inc0075_product_phase_translation_dense_quality_recovery_screen.json`
- Summary row range: rows where `run_tag` contains `inc0075_product_phase_translation_dense_quality_recovery_screen`
- Decision note: `docs/governance/gates/gate_20260311_142843.md`

## Batch B131
- Batch ID: `B131_inc0075_product_phase_translation_dense_quality_recovery_confirm`
- Config: `configs/proxy_transfer_inc0075_product_phase_translation_dense_quality_recovery_confirm.json`
- Logs: `results/raw/inc0075_product_phase_translation_dense_quality_recovery_confirm_*.log`
- Parsed files: matching `results/parsed/inc0075_product_phase_translation_dense_quality_recovery_confirm_*.json`
- Analysis:
  - `results/analysis/inc0075_product_phase_translation_dense_quality_recovery_confirm.json`
- Summary row range: rows where `run_tag` contains `inc0075_product_phase_translation_dense_quality_recovery_confirm`
- Decision note: `docs/governance/gates/gate_20260311_144445.md`

## Batch B132
- Batch ID: `B132_inc0076_product_phase_translation_break_even_screen`
- Config: `configs/proxy_transfer_inc0076_product_phase_translation_break_even_screen.json`
- Logs: `results/raw/inc0076_product_phase_translation_break_even_screen_*.log`
- Parsed files: matching `results/parsed/inc0076_product_phase_translation_break_even_screen_*.json`
- Analysis:
  - `results/analysis/inc0076_product_phase_translation_break_even_screen.json`
- Summary row range: rows where `run_tag` contains `inc0076_product_phase_translation_break_even_screen`
- Decision note: `docs/governance/gates/gate_20260311_145722.md`

## Batch B133
- Batch ID: `B133_inc0076_product_phase_translation_break_even_confirm`
- Config: `configs/proxy_transfer_inc0076_product_phase_translation_break_even_confirm.json`
- Logs: `results/raw/inc0076_product_phase_translation_break_even_confirm_*.log`
- Parsed files: matching `results/parsed/inc0076_product_phase_translation_break_even_confirm_*.json`
- Analysis:
  - `results/analysis/inc0076_product_phase_translation_break_even_confirm.json`
- Summary row range: rows where `run_tag` contains `inc0076_product_phase_translation_break_even_confirm`
- Decision note: `docs/governance/gates/gate_20260311_151013.md`

## Batch B134
- Batch ID: `B134_inc0077_product_phase_translation_hardware_profile_screen`
- Config: `configs/proxy_transfer_inc0077_product_phase_translation_hardware_profile_screen.json`
- Logs: `results/raw/inc0077_product_phase_translation_hardware_profile_screen_*.log`
- Parsed files: matching `results/parsed/inc0077_product_phase_translation_hardware_profile_screen_*.json`
- Analysis:
  - `results/analysis/inc0077_product_phase_translation_hardware_profile_screen.json`
  - `results/analysis/inc0077_product_phase_translation_hardware_profile_screen_profile.json`
- Summary row range: rows where `run_tag` contains `inc0077_product_phase_translation_hardware_profile_screen`
- Decision note: `docs/governance/gates/gate_20260311_152632.md`

## Batch B135
- Batch ID: `B135_inc0077_product_phase_translation_hardware_profile_confirm`
- Config: `configs/proxy_transfer_inc0077_product_phase_translation_hardware_profile_confirm.json`
- Logs: `results/raw/inc0077_product_phase_translation_hardware_profile_confirm_*.log`
- Parsed files: matching `results/parsed/inc0077_product_phase_translation_hardware_profile_confirm_*.json`
- Analysis:
  - `results/analysis/inc0077_product_phase_translation_hardware_profile_confirm.json`
  - `results/analysis/inc0077_product_phase_translation_hardware_profile_confirm_profile.json`
- Summary row range: rows where `run_tag` contains `inc0077_product_phase_translation_hardware_profile_confirm`
- Decision note: `docs/governance/gates/gate_20260311_153047.md`

## Batch B136
- Batch ID: `B136_inc0078_product_phase_translation_crossover_map_screen`
- Config: `configs/proxy_transfer_inc0078_product_phase_translation_crossover_map_screen.json`
- Logs: `results/raw/inc0078_product_phase_translation_crossover_map_screen_*.log`
- Parsed files: matching `results/parsed/inc0078_product_phase_translation_crossover_map_screen_*.json`
- Analysis:
  - `results/analysis/inc0078_product_phase_translation_crossover_map_screen.json`
  - `results/analysis/inc0078_product_phase_translation_crossover_map_screen_profile.json`
- Summary row range: rows where `run_tag` contains `inc0078_product_phase_translation_crossover_map_screen`
- Decision note: `docs/governance/gates/gate_20260311_155644.md`

## Batch B137
- Batch ID: `B137_inc0078_product_phase_translation_crossover_map_confirm`
- Config: `configs/proxy_transfer_inc0078_product_phase_translation_crossover_map_confirm.json`
- Logs: `results/raw/inc0078_product_phase_translation_crossover_map_confirm_*.log`
- Parsed files: matching `results/parsed/inc0078_product_phase_translation_crossover_map_confirm_*.json`
- Analysis:
  - `results/analysis/inc0078_product_phase_translation_crossover_map_confirm.json`
  - `results/analysis/inc0078_product_phase_translation_crossover_map_confirm_profile.json`
- Summary row range: rows where `run_tag` contains `inc0078_product_phase_translation_crossover_map_confirm`
- Decision note: `docs/governance/gates/gate_20260311_161119.md`

## Batch B138
- Batch ID: `B138_inc0079_product_phase_translation_large_bank_boundary_extension_screen`
- Config: `configs/proxy_transfer_inc0079_product_phase_translation_large_bank_boundary_extension_screen.json`
- Logs: `results/raw/inc0079_product_phase_translation_large_bank_boundary_extension_screen_*.log`
- Parsed files: matching `results/parsed/inc0079_product_phase_translation_large_bank_boundary_extension_screen_*.json`
- Analysis:
  - `results/analysis/inc0079_product_phase_translation_large_bank_boundary_extension_screen.json`
  - `results/analysis/inc0079_product_phase_translation_large_bank_boundary_extension_screen_profile.json`
- Summary row range: rows where `run_tag` contains `inc0079_product_phase_translation_large_bank_boundary_extension_screen`
- Decision note: `docs/governance/gates/gate_20260311_222501.md`

## Batch B139
- Batch ID: `B139_inc0079_product_phase_translation_large_bank_boundary_extension_confirm`
- Config: `configs/proxy_transfer_inc0079_product_phase_translation_large_bank_boundary_extension_confirm.json`
- Logs: `results/raw/inc0079_product_phase_translation_large_bank_boundary_extension_confirm_*.log`
- Parsed files: matching `results/parsed/inc0079_product_phase_translation_large_bank_boundary_extension_confirm_*.json`
- Analysis:
  - `results/analysis/inc0079_product_phase_translation_large_bank_boundary_extension_confirm.json`
  - `results/analysis/inc0079_product_phase_translation_large_bank_boundary_extension_confirm_profile.json`
- Summary row range: rows where `run_tag` contains `inc0079_product_phase_translation_large_bank_boundary_extension_confirm`
- Decision note: `docs/governance/gates/gate_20260311_223841.md`

## Batch B140
- Batch ID: `B140_inc0080_product_phase_translation_second_large_bank_boundary_extension_screen`
- Config: `configs/proxy_transfer_inc0080_product_phase_translation_second_large_bank_boundary_extension_screen.json`
- Logs: `results/raw/inc0080_product_phase_translation_second_large_bank_boundary_extension_screen_*.log`
- Parsed files: matching `results/parsed/inc0080_product_phase_translation_second_large_bank_boundary_extension_screen_*.json`
- Analysis:
  - `results/analysis/inc0080_product_phase_translation_second_large_bank_boundary_extension_screen.json`
  - `results/analysis/inc0080_product_phase_translation_second_large_bank_boundary_extension_screen_profile.json`
- Summary row range: rows where `run_tag` contains `inc0080_product_phase_translation_second_large_bank_boundary_extension_screen`
- Decision note: `docs/governance/gates/gate_20260311_230536.md`

## Batch B141
- Batch ID: `B141_inc0080_product_phase_translation_second_large_bank_boundary_extension_confirm`
- Config: `configs/proxy_transfer_inc0080_product_phase_translation_second_large_bank_boundary_extension_confirm.json`
- Logs: `results/raw/inc0080_product_phase_translation_second_large_bank_boundary_extension_confirm_*.log`
- Parsed files: matching `results/parsed/inc0080_product_phase_translation_second_large_bank_boundary_extension_confirm_*.json`
- Analysis:
  - `results/analysis/inc0080_product_phase_translation_second_large_bank_boundary_extension_confirm.json`
  - `results/analysis/inc0080_product_phase_translation_second_large_bank_boundary_extension_confirm_profile.json`
- Summary row range: rows where `run_tag` contains `inc0080_product_phase_translation_second_large_bank_boundary_extension_confirm`
- Decision note: `docs/governance/gates/gate_20260311_232657.md`

## Batch B142
- Batch ID: `B142_inc0081_product_phase_translation_q04_threshold_search_screen`
- Config: `configs/proxy_transfer_inc0081_product_phase_translation_q04_threshold_search_screen.json`
- Logs: `results/raw/inc0081_product_phase_translation_q04_threshold_search_screen_*.log`
- Parsed files: matching `results/parsed/inc0081_product_phase_translation_q04_threshold_search_screen_*.json`
- Analysis:
  - `results/analysis/inc0081_product_phase_translation_q04_threshold_search_screen.json`
  - `results/analysis/inc0081_product_phase_translation_q04_threshold_search_screen_profile.json`
- Summary row range: rows where `run_tag` contains `inc0081_product_phase_translation_q04_threshold_search_screen`
- Decision note: `docs/governance/gates/gate_20260311_234946.md`

## Batch B143
- Batch ID: `B143_inc0081_product_phase_translation_q04_threshold_search_confirm`
- Config: `configs/proxy_transfer_inc0081_product_phase_translation_q04_threshold_search_confirm.json`
- Logs: `results/raw/inc0081_product_phase_translation_q04_threshold_search_confirm_*.log`
- Parsed files: matching `results/parsed/inc0081_product_phase_translation_q04_threshold_search_confirm_*.json`
- Analysis:
  - `results/analysis/inc0081_product_phase_translation_q04_threshold_search_confirm.json`
  - `results/analysis/inc0081_product_phase_translation_q04_threshold_search_confirm_profile.json`
- Summary row range: rows where `run_tag` contains `inc0081_product_phase_translation_q04_threshold_search_confirm`
- Decision note: `docs/governance/gates/gate_20260312_002000.md`

## Analysis INC-0082
- Source analysis: `results/analysis/inc0081_product_phase_translation_q04_threshold_search_confirm.json`
- Source profile: `results/analysis/inc0081_product_phase_translation_q04_threshold_search_confirm_profile.json`
- Audit tool: `tools/translated_cost_accounting.py`
- Audit outputs:
  - `results/analysis/inc0082_product_phase_translation_cost_accounting_audit.json`
  - `docs/reports/INC0082_PRODUCT_PHASE_TRANSLATION_COST_ACCOUNTING_AUDIT.md`
- Reading:
  - `Q04 T36000` crosses because online savings exceed the offline penalty
  - `Q04 T40000` misses because the offline penalty exceeds the online gain
  - `Q08 T40000` still crosses because the same static offline cost is spread
    across more repeats

## Analysis INC-0083
- Source analysis:
  - `results/analysis/inc0083_product_phase_translation_persistent_route_cache_confirm_cold.json`
  - `results/analysis/inc0083_product_phase_translation_persistent_route_cache_confirm_warm.json`
- Compare tool: `tools/translated_cache_compare.py`
- Compare outputs:
  - `results/analysis/inc0083_product_phase_translation_persistent_route_cache_confirm_compare.json`
  - `docs/reports/INC0083_PRODUCT_PHASE_TRANSLATION_PERSISTENT_ROUTE_CACHE_CONFIRM_COMPARE.md`
- Gate notes:
  - `docs/governance/gates/gate_20260312_012911.md`
  - `docs/governance/gates/gate_20260312_013859.md`
- Reading:
  - persistent cache reuse removes almost all static offline chart/train-route
    build cost on the fixed translated product stack
  - `Q04 T40000` flips from a cold miss to a strong warm-cache crossover
  - `Q08 T40000` also strengthens materially under warm-cache reuse
  - top-1 and candidate fraction stay unchanged between cold and warm routed
    runs

## Analysis INC-0084
- Source analysis:
  - `results/analysis/inc0084_product_phase_translation_warm_cache_onset_map_screen.json`
  - `results/analysis/inc0084_product_phase_translation_warm_cache_onset_map_confirm.json`
- Report:
  - `docs/reports/INC0084_PRODUCT_PHASE_TRANSLATION_WARM_CACHE_ONSET_MAP.md`
- Gate notes:
  - `docs/governance/gates/gate_20260312_015936.md`
  - `docs/governance/gates/gate_20260312_021141.md`
- Reading:
  - the fixed translated product stack crosses dense all the way down to
    `Q01` under warm-cache conditions at `T40000`
  - all routed runs keep `chart_cache_hit=1.0` and `route_cache_hit=1.0`
  - top-1 and candidate fraction stay unchanged across the warm-cache repeat
    bracket

## Analysis INC-0085
- Source analysis:
  - `results/analysis/inc0085_product_phase_translation_warm_cache_q01_bank_boundary_screen.json`
  - `results/analysis/inc0085_product_phase_translation_warm_cache_q01_bank_boundary_confirm.json`
- Profiles:
  - `results/analysis/inc0085_product_phase_translation_warm_cache_q01_bank_boundary_screen_profile.json`
  - `results/analysis/inc0085_product_phase_translation_warm_cache_q01_bank_boundary_confirm_profile.json`
- Reports:
  - `docs/reports/INC0085_PRODUCT_PHASE_TRANSLATION_WARM_CACHE_Q01_BANK_BOUNDARY_SCREEN.md`
  - `docs/reports/INC0085_PRODUCT_PHASE_TRANSLATION_WARM_CACHE_Q01_BANK_BOUNDARY_CONFIRM.md`
- Gate notes:
  - `docs/governance/gates/gate_20260312_045413.md`
  - `docs/governance/gates/gate_20260312_045437.md`
  - `docs/governance/gates/gate_20260312_045613.md`
  - `docs/governance/gates/gate_20260312_045657.md`
- Reading:
  - the fixed translated product stack now has an earliest tracked confirmed
    warm-cache `Q01` crossover at `T3000`
  - `T3000/T4500/T6000` all stayed positive on the tracked `Q01/Q02` bracket
  - routed cache hits stayed exact across the full ladder
  - search work stayed pinned near `19%` of dense across banks

## Analysis INC-0086
- Source analysis:
  - `results/analysis/inc0086_product_phase_translation_warm_cache_q01_lower_boundary_refine_screen.json`
  - `results/analysis/inc0086_product_phase_translation_warm_cache_q01_lower_boundary_refine_confirm.json`
- Profiles:
  - `results/analysis/inc0086_product_phase_translation_warm_cache_q01_lower_boundary_refine_screen_profile.json`
  - `results/analysis/inc0086_product_phase_translation_warm_cache_q01_lower_boundary_refine_confirm_profile.json`
- Reports:
  - `docs/reports/INC0086_PRODUCT_PHASE_TRANSLATION_WARM_CACHE_Q01_LOWER_BOUNDARY_REFINE_SCREEN.md`
  - `docs/reports/INC0086_PRODUCT_PHASE_TRANSLATION_WARM_CACHE_Q01_LOWER_BOUNDARY_REFINE_CONFIRM.md`
- Gate notes:
  - `docs/governance/gates/gate_20260312_051358.md`
  - `docs/governance/gates/gate_20260312_051419.md`
  - `docs/governance/gates/gate_20260312_051526.md`
  - `docs/governance/gates/gate_20260312_051652.md`
- Reading:
  - the fixed translated product stack now has an earlier tracked confirmed
    warm-cache `Q01` crossover at `T2750`
  - `T2500` still misses at `Q01`, but already crosses at `Q02`
  - routed cache hits stayed exact across the full `T2500/T2750/T3000`
    `Q01/Q02` bracket
  - search work stayed pinned near `19%` of dense across the refined bank band

## Analysis INC-0087
- Source analysis:
  - `results/analysis/inc0087_product_phase_translation_warm_cache_q01_threshold_refine_screen.json`
  - `results/analysis/inc0087_product_phase_translation_warm_cache_q01_threshold_refine_confirm.json`
- Profiles:
  - `results/analysis/inc0087_product_phase_translation_warm_cache_q01_threshold_refine_screen_profile.json`
  - `results/analysis/inc0087_product_phase_translation_warm_cache_q01_threshold_refine_confirm_profile.json`
- Reports:
  - `docs/reports/INC0087_PRODUCT_PHASE_TRANSLATION_WARM_CACHE_Q01_THRESHOLD_REFINE_SCREEN.md`
  - `docs/reports/INC0087_PRODUCT_PHASE_TRANSLATION_WARM_CACHE_Q01_THRESHOLD_REFINE_CONFIRM.md`
- Gate notes:
  - `docs/governance/gates/gate_20260312_052842.md`
  - `docs/governance/gates/gate_20260312_052915.md`
  - `docs/governance/gates/gate_20260312_053034.md`
  - `docs/governance/gates/gate_20260312_053113.md`
- Reading:
  - the fixed translated product stack now has an earlier tracked confirmed
    warm-cache `Q01` crossover at `T2600`
  - `T2650` is a local non-monotone pocket: it misses at `Q01` and crosses at
    `Q02`
  - routed cache hits stayed exact across the full `T2600/T2650/T2700`
    `Q01/Q02` bracket
  - search work stayed pinned near `19%` of dense across the refined band

## Analysis INC-0088
- Source confirm analysis:
  - `results/analysis/inc0087_product_phase_translation_warm_cache_q01_threshold_refine_confirm.json`
- Source confirm profile:
  - `results/analysis/inc0087_product_phase_translation_warm_cache_q01_threshold_refine_confirm_profile.json`
- Cost audit:
  - `results/analysis/inc0087_product_phase_translation_warm_cache_q01_threshold_refine_cost_audit.json`
- Cost report:
  - `docs/reports/INC0087_PRODUCT_PHASE_TRANSLATION_WARM_CACHE_Q01_THRESHOLD_REFINE_COST_AUDIT.md`
- Reading:
  - the local `T2600/T2650/T2700` split is explained by route-query versus
    dense-search cost balance
  - `T2600` stays positive because search gain still beats the fixed routed
    penalties
  - `T2650` misses because dense search time dips locally while routed
    route-query cost stays almost unchanged
  - `T2700` crosses again once the dense search time rises back up

## Analysis INC-0089
- Source analysis:
  - `results/analysis/inc0089_product_phase_translation_warm_cache_q01_2500_2600_refine_screen.json`
  - `results/analysis/inc0089_product_phase_translation_warm_cache_q01_2500_2600_refine_confirm.json`
- Profiles:
  - `results/analysis/inc0089_product_phase_translation_warm_cache_q01_2500_2600_refine_screen_profile.json`
  - `results/analysis/inc0089_product_phase_translation_warm_cache_q01_2500_2600_refine_confirm_profile.json`
- Reports:
  - `docs/reports/INC0089_PRODUCT_PHASE_TRANSLATION_WARM_CACHE_Q01_2500_2600_REFINE_SCREEN.md`
  - `docs/reports/INC0089_PRODUCT_PHASE_TRANSLATION_WARM_CACHE_Q01_2500_2600_REFINE_CONFIRM.md`
- Gate notes:
  - `docs/governance/gates/gate_20260312_054028.md`
  - `docs/governance/gates/gate_20260312_054048.md`
  - `docs/governance/gates/gate_20260312_054156.md`
  - `docs/governance/gates/gate_20260312_054233.md`
- Reading:
  - the fixed translated product stack now has an earlier tracked confirmed
    warm-cache `Q01` crossover at `T2525`
  - all three banks in the `2525/2550/2575` bracket held at `Q01` on confirm
  - routed cache hits stayed exact across the full `Q01/Q02` bracket
  - search work stayed pinned near `19%` of dense across the refined band

## Analysis INC-0090
- Source analysis:
  - `results/analysis/inc0090_product_phase_translation_warm_cache_q01_2500_2525_refine_screen.json`
  - `results/analysis/inc0090_product_phase_translation_warm_cache_q01_2500_2525_refine_confirm.json`
- Profiles:
  - `results/analysis/inc0090_product_phase_translation_warm_cache_q01_2500_2525_refine_screen_profile.json`
  - `results/analysis/inc0090_product_phase_translation_warm_cache_q01_2500_2525_refine_confirm_profile.json`
- Reports:
  - `docs/reports/INC0090_PRODUCT_PHASE_TRANSLATION_WARM_CACHE_Q01_2500_2525_REFINE_SCREEN.md`
  - `docs/reports/INC0090_PRODUCT_PHASE_TRANSLATION_WARM_CACHE_Q01_2500_2525_REFINE_CONFIRM.md`
- Gate notes:
  - `docs/governance/gates/gate_20260312_060106.md`
  - `docs/governance/gates/gate_20260312_060131.md`
  - `docs/governance/gates/gate_20260312_060246.md`
  - `docs/governance/gates/gate_20260312_060331.md`
- Reading:
  - the fixed translated product stack now has an earlier tracked confirmed
    warm-cache `Q01` crossover at `T2505`
  - all four banks in the `2505/2510/2515/2520` bracket held at `Q01` on
    confirm
  - routed cache hits stayed exact across the full `Q01/Q02` bracket
  - search work stayed pinned near `19%` of dense across the refined band

## Analysis INC-0091
- Source analysis:
  - `results/analysis/inc0091_product_phase_translation_warm_cache_q01_2500_2505_refine_screen.json`
  - `results/analysis/inc0091_product_phase_translation_warm_cache_q01_2500_2505_refine_confirm.json`
- Profiles:
  - `results/analysis/inc0091_product_phase_translation_warm_cache_q01_2500_2505_refine_screen_profile.json`
  - `results/analysis/inc0091_product_phase_translation_warm_cache_q01_2500_2505_refine_confirm_profile.json`
- Reports:
  - `docs/reports/INC0091_PRODUCT_PHASE_TRANSLATION_WARM_CACHE_Q01_2500_2505_REFINE_SCREEN.md`
  - `docs/reports/INC0091_PRODUCT_PHASE_TRANSLATION_WARM_CACHE_Q01_2500_2505_REFINE_CONFIRM.md`
- Gate notes:
  - `docs/governance/gates/gate_20260312_062958.md`
  - `docs/governance/gates/gate_20260312_063025.md`
  - `docs/governance/gates/gate_20260312_063141.md`
  - `docs/governance/gates/gate_20260312_063230.md`
- Reading:
  - the fixed translated product stack now has an exact tracked confirmed
    warm-cache `Q01` crossover at `T2501`
  - all four banks in the `2501/2502/2503/2504` bracket held at `Q01` on
    confirm
  - the screen-only `T2503 Q02` pocket disappeared on confirm
  - routed cache hits stayed exact across the full `Q01/Q02` bracket
  - search work stayed pinned near `19%` of dense across the refined band

## Analysis INC-0092
- Source analysis:
  - `results/analysis/inc0092_product_phase_translation_warm_cache_q01_floor_hardening_screen.json`
  - `results/analysis/inc0092_product_phase_translation_warm_cache_q01_floor_hardening_confirm.json`
- Profiles:
  - `results/analysis/inc0092_product_phase_translation_warm_cache_q01_floor_hardening_screen_profile.json`
  - `results/analysis/inc0092_product_phase_translation_warm_cache_q01_floor_hardening_confirm_profile.json`
- Reports:
  - `docs/reports/INC0092_PRODUCT_PHASE_TRANSLATION_WARM_CACHE_Q01_FLOOR_HARDENING_SCREEN.md`
  - `docs/reports/INC0092_PRODUCT_PHASE_TRANSLATION_WARM_CACHE_Q01_FLOOR_HARDENING_CONFIRM.md`
- Gate notes:
  - `docs/governance/gates/gate_20260312_064253.md`
  - `docs/governance/gates/gate_20260312_064320.md`
  - `docs/governance/gates/gate_20260312_064442.md`
  - `docs/governance/gates/gate_20260312_064531.md`
- Reading:
  - the fixed translated product stack no longer supports the old exact
    `T2500` miss / `T2501` hit story under hardening
  - both `T2500` and `T2501` now hold at `Q01`
  - the lower-bank warm-cache floor therefore collapses to `T2500`
  - routed cache hits stayed exact across the full `Q01/Q02` bracket
  - search work stayed pinned near `20%` of dense across the hardened band

## Analysis INC-0093
- Source analysis:
  - `results/analysis/inc0093_product_phase_translation_cache_residency_mix_screen_cold.json`
  - `results/analysis/inc0093_product_phase_translation_cache_residency_mix_screen_warm.json`
  - `results/analysis/inc0093_product_phase_translation_cache_residency_mix_confirm_cold.json`
  - `results/analysis/inc0093_product_phase_translation_cache_residency_mix_confirm_warm.json`
- Compare analyses:
  - `results/analysis/inc0093_product_phase_translation_cache_residency_mix_screen_compare_chart.json`
  - `results/analysis/inc0093_product_phase_translation_cache_residency_mix_screen_compare_route.json`
  - `results/analysis/inc0093_product_phase_translation_cache_residency_mix_screen_compare_full.json`
  - `results/analysis/inc0093_product_phase_translation_cache_residency_mix_confirm_compare_chart.json`
  - `results/analysis/inc0093_product_phase_translation_cache_residency_mix_confirm_compare_route.json`
  - `results/analysis/inc0093_product_phase_translation_cache_residency_mix_confirm_compare_full.json`
- Reports:
  - `docs/reports/INC0093_PRODUCT_PHASE_TRANSLATION_CACHE_RESIDENCY_MIX_SCREEN_COMPARE_CHART.md`
  - `docs/reports/INC0093_PRODUCT_PHASE_TRANSLATION_CACHE_RESIDENCY_MIX_SCREEN_COMPARE_ROUTE.md`
  - `docs/reports/INC0093_PRODUCT_PHASE_TRANSLATION_CACHE_RESIDENCY_MIX_SCREEN_COMPARE_FULL.md`
  - `docs/reports/INC0093_PRODUCT_PHASE_TRANSLATION_CACHE_RESIDENCY_MIX_CONFIRM_COMPARE_CHART.md`
  - `docs/reports/INC0093_PRODUCT_PHASE_TRANSLATION_CACHE_RESIDENCY_MIX_CONFIRM_COMPARE_ROUTE.md`
  - `docs/reports/INC0093_PRODUCT_PHASE_TRANSLATION_CACHE_RESIDENCY_MIX_CONFIRM_COMPARE_FULL.md`
  - `docs/reports/INC0093_PRODUCT_PHASE_TRANSLATION_CACHE_RESIDENCY_MIX_CONFIRM.md`
- Gate notes:
  - `docs/governance/gates/gate_20260312_071450.md`
  - `docs/governance/gates/gate_20260312_071837.md`
  - `docs/governance/gates/gate_20260312_072037.md`
  - `docs/governance/gates/gate_20260312_072941.md`
  - `docs/governance/gates/gate_20260312_073711.md`
  - `docs/governance/gates/gate_20260312_074111.md`
- Reading:
  - chart residency carries almost all of the operational rescue on the fixed
    translated product stack
  - route-only residency stays negative at both anchor operating points
  - the upper-bank `T40000 Q01` systems win survives under chart-only
    residency
  - the exact lower-bank `T2500 Q01` floor still requires full warm residency

## Analysis INC-0094
- Source analysis:
  - `results/analysis/inc0094_product_phase_translation_chart_resident_route_ephemeral_repeat_map_screen.json`
  - `results/analysis/inc0094_product_phase_translation_chart_resident_route_ephemeral_repeat_map_confirm.json`
- Profiles:
  - `results/analysis/inc0094_product_phase_translation_chart_resident_route_ephemeral_repeat_map_screen_profile.json`
  - `results/analysis/inc0094_product_phase_translation_chart_resident_route_ephemeral_repeat_map_confirm_profile.json`
- Reports:
  - `docs/reports/INC0094_PRODUCT_PHASE_TRANSLATION_CHART_RESIDENT_ROUTE_EPHEMERAL_REPEAT_MAP_SCREEN.md`
  - `docs/reports/INC0094_PRODUCT_PHASE_TRANSLATION_CHART_RESIDENT_ROUTE_EPHEMERAL_REPEAT_MAP_CONFIRM.md`
- Gate notes:
  - `docs/governance/gates/gate_20260312_080917.md`
  - `docs/governance/gates/gate_20260312_081226.md`
  - `docs/governance/gates/gate_20260312_081526.md`
  - `docs/governance/gates/gate_20260312_082126.md`
- Reading:
  - chart-resident / route-ephemeral sessions now survive at `T2500` by `Q02`
  - chart-resident `T40000` already survives by `Q01`
  - the remaining gap is the earliest bank where chart-only `Q01` survives

## Analysis INC-0095
- Source analysis:
  - `results/analysis/inc0095_product_phase_translation_chart_resident_q01_bank_boundary_screen.json`
  - `results/analysis/inc0095_product_phase_translation_chart_resident_q01_bank_boundary_confirm.json`
- Profiles:
  - `results/analysis/inc0095_product_phase_translation_chart_resident_q01_bank_boundary_screen_profile.json`
  - `results/analysis/inc0095_product_phase_translation_chart_resident_q01_bank_boundary_confirm_profile.json`
- Reports:
  - `docs/reports/INC0095_PRODUCT_PHASE_TRANSLATION_CHART_RESIDENT_Q01_BANK_BOUNDARY_SCREEN.md`
  - `docs/reports/INC0095_PRODUCT_PHASE_TRANSLATION_CHART_RESIDENT_Q01_BANK_BOUNDARY_CONFIRM.md`
- Gate notes:
  - `docs/governance/gates/gate_20260312_090117.md`
  - `docs/governance/gates/gate_20260312_090210.md`
  - `docs/governance/gates/gate_20260312_090502.md`
  - `docs/governance/gates/gate_20260312_090530.md`
- Reading:
  - the focused chart-resident `Q01` packet now survives already at `T2500`
  - the earlier `INC-0094` mixed-repeat `T2500 Q01` miss is now a packet-scope
    sensitivity issue, not a bank-location issue

## Analysis INC-0096
- Source analysis:
  - `results/analysis/inc0096_product_phase_translation_chart_resident_q01_packet_scope_audit_focused_screen.json`
  - `results/analysis/inc0096_product_phase_translation_chart_resident_q01_packet_scope_audit_mixed_screen.json`
  - `results/analysis/inc0096_product_phase_translation_chart_resident_q01_packet_scope_audit_focused_confirm.json`
  - `results/analysis/inc0096_product_phase_translation_chart_resident_q01_packet_scope_audit_mixed_confirm.json`
- Profiles:
  - `results/analysis/inc0096_product_phase_translation_chart_resident_q01_packet_scope_audit_focused_screen_profile.json`
  - `results/analysis/inc0096_product_phase_translation_chart_resident_q01_packet_scope_audit_mixed_screen_profile.json`
  - `results/analysis/inc0096_product_phase_translation_chart_resident_q01_packet_scope_audit_focused_confirm_profile.json`
  - `results/analysis/inc0096_product_phase_translation_chart_resident_q01_packet_scope_audit_mixed_confirm_profile.json`
- Compare artifacts:
  - `results/analysis/inc0096_product_phase_translation_chart_resident_q01_packet_scope_audit_screen_compare.json`
  - `results/analysis/inc0096_product_phase_translation_chart_resident_q01_packet_scope_audit_confirm_compare.json`
  - `docs/reports/INC0096_PRODUCT_PHASE_TRANSLATION_CHART_RESIDENT_Q01_PACKET_SCOPE_AUDIT_SCREEN_COMPARE.md`
  - `docs/reports/INC0096_PRODUCT_PHASE_TRANSLATION_CHART_RESIDENT_Q01_PACKET_SCOPE_AUDIT_CONFIRM_COMPARE.md`
- Gate notes:
  - `docs/governance/gates/gate_20260312_092114.md`
  - `docs/governance/gates/gate_20260312_092121.md`
  - `docs/governance/gates/gate_20260312_092137.md`
  - `docs/governance/gates/gate_20260312_092238.md`
  - `docs/governance/gates/gate_20260312_092252.md`
  - `docs/governance/gates/gate_20260312_092334.md`
- Reading:
  - both focused and mixed packets stay positive at `T2500 Q01` on hardening
  - packet scope changes the margin size, but not the retrieval signal or the
  sign of the systems claim

## Analysis INC-0097
- Source analysis:
  - `results/analysis/inc0097_product_phase_sparse_gated_shell_screen.json`
- Reports:
  - `docs/reports/INC0097_PRODUCT_PHASE_SPARSE_GATED_SHELL_SCREEN.md`
- Gate notes:
  - `docs/governance/gates/gate_20260312_095747.md`
- Reading:
  - the continuous product reference remains the only healthy product route on
    the sparse-shell screen
  - gated and banded shell control are mechanism-live but both collapse shell
    balance (`shell_pmax≈0.985`)
  - the shell pilot closes negative and does not justify translated carry-
    forward

## Analysis INC-0098
- Source analysis:
  - `results/analysis/inc0098_product_phase_translation_chart_resident_route_cost_decomposition_input.json`
  - `results/analysis/inc0098_product_phase_translation_chart_resident_route_cost_decomposition.json`
- Reports:
  - `docs/reports/INC0098_PRODUCT_PHASE_TRANSLATION_CHART_RESIDENT_ROUTE_COST_DECOMPOSITION.md`
- Reading:
  - the fixed chart-resident translated stack is already positive against dense
    at both lower and upper anchors
  - the remaining chart-versus-full-warm gap is split between route-index
    build and retrieval-search overhead
  - the translated systems branch can now be frozen as a reference instead of
    continued as an active cost-rescue branch

## Analysis INC-0099
- Source analysis:
  - `results/analysis/inc0099_product_phase_sparse_event_proxy_screen.json`
  - `results/analysis/inc0099_product_phase_sparse_event_proxy_confirm.json`
- Reports:
  - `docs/reports/INC0099_PRODUCT_PHASE_SPARSE_EVENT_PROXY_SCREEN.md`
  - `docs/reports/INC0099_PRODUCT_PHASE_SPARSE_EVENT_PROXY_CONFIRM.md`
- Gate notes:
  - `docs/governance/gates/gate_20260312_104037.md`
  - `docs/governance/gates/gate_20260312_104345.md`
- Reading:
  - the fixed product route law now supports a healthy soft-sparse proxy
    controller
  - promoted point `H4XH4_FIELD_A150_EVT_T070` holds with
    `event_gate_mean≈0.319`
  - the positive sparse-event read is about soft update mass, not yet hard
    firing
  - the next honest branch is translated carry-forward of that fixed sparse
    point

## Analysis INC-0100
- Source analysis:
  - `results/analysis/inc0100_product_phase_sparse_event_translation_screen.json`
  - `results/analysis/inc0100_product_phase_sparse_event_translation_confirm.json`
- Reports:
  - `docs/reports/INC0100_PRODUCT_PHASE_SPARSE_EVENT_TRANSLATION_SCREEN.md`
  - `docs/reports/INC0100_PRODUCT_PHASE_SPARSE_EVENT_TRANSLATION_CONFIRM.md`
- Gate notes:
  - `docs/governance/gates/gate_20260312_110233.md`
  - `docs/governance/gates/gate_20260312_110324.md`
- Reading:
  - the fixed soft-sparse controller now carries through the translated
    chart-resident retrieval harness
  - the sparse translated point preserves routed retrieval signal relative to
    the continuous product reference while materially improving runtime
  - the lower-bank `Q01` dense-amortized comparison becomes an almost exact tie
    rather than a routed miss
  - the branch is still soft-sparse, not hard-firing, so hard threshold
    activation becomes the next honest mechanism test

## Analysis INC-0101
- Source analysis:
  - `results/analysis/inc0101_product_phase_hard_event_proxy_screen.json`
  - `results/analysis/inc0101_product_phase_hard_event_proxy_confirm.json`
- Reports:
  - `docs/reports/INC0101_PRODUCT_PHASE_HARD_EVENT_PROXY_SCREEN.md`
  - `docs/reports/INC0101_PRODUCT_PHASE_HARD_EVENT_PROXY_CONFIRM.md`
- Gate notes:
  - `docs/governance/gates/gate_20260312_113952.md`
  - `docs/governance/gates/gate_20260312_114258.md`
- Reading:
  - the fixed product route law now supports a stable near-hard proxy event
    point
  - promoted point `H4XH4_FIELD_A150_EVT_T070_TAU002` holds with
    `event_gate_mean≈0.0205`
  - the true hard controller stays healthy only in a mostly-on regime and does
    not justify the same discreteness claim
  - the next honest branch is translated carry-forward of the near-hard point,
    not more hard-threshold shaving

## Analysis INC-0102
- Source analysis:
  - `results/analysis/inc0102_product_phase_near_hard_event_translation_prewarm_screen.json`
  - `results/analysis/inc0102_product_phase_near_hard_event_translation_screen.json`
- Reports:
  - `docs/reports/INC0102_PRODUCT_PHASE_NEAR_HARD_EVENT_TRANSLATION_SCREEN.md`
- Gate notes:
  - `docs/governance/gates/gate_20260312_115408.md`
  - `docs/governance/gates/gate_20260312_115418.md`
- Reading:
  - the translated near-hard candidate preserves the same routed retrieval
    signal as the continuous and soft sparse translated references
  - but it becomes slower than both translated references and therefore loses
    the translated systems story
  - translated sparse-event claims remain explicitly soft
  - the next honest branch is bounded quality recovery on the fixed soft sparse
    translated point

## Analysis INC-0103
- Source analysis:
  - `results/analysis/inc0103_product_phase_soft_sparse_translation_quality_recovery_screen.json`
  - `results/analysis/inc0103_product_phase_soft_sparse_translation_quality_recovery_confirm.json`
- Reports:
  - `docs/reports/INC0103_PRODUCT_PHASE_SOFT_SPARSE_TRANSLATION_QUALITY_RECOVERY_SCREEN.md`
  - `docs/reports/INC0103_PRODUCT_PHASE_SOFT_SPARSE_TRANSLATION_QUALITY_RECOVERY_CONFIRM.md`
- Gate notes:
  - `docs/governance/gates/gate_20260312_120306.md`
  - `docs/governance/gates/gate_20260312_120504.md`
- Reading:
  - bounded low-margin reranking did not recover translated quality on confirm
  - the best rerank point only matched top-1 and trimmed amortized cost
    slightly
  - the fixed soft sparse translated point remains the sparse translated
    quality/reference point

## Analysis INC-0104
- Source analysis:
  - `results/analysis/inc0104_product_phase_soft_sparse_translation_backfill_recovery_screen.json`
  - `results/analysis/inc0104_product_phase_soft_sparse_translation_backfill_recovery_confirm.json`
- Reports:
  - `docs/reports/INC0104_PRODUCT_PHASE_SOFT_SPARSE_TRANSLATION_BACKFILL_RECOVERY_SCREEN.md`
  - `docs/reports/INC0104_PRODUCT_PHASE_SOFT_SPARSE_TRANSLATION_BACKFILL_RECOVERY_CONFIRM.md`
- Gate notes:
  - `docs/governance/gates/gate_20260312_121232.md`
  - `docs/governance/gates/gate_20260312_121339.md`
- Reading:
  - bounded small-bucket backfill did not recover translated quality
  - but it created a materially better lower-bank sparse translated systems
    point
  - the branch closes negative on quality recovery and positive on systems
    refinement

## Analysis INC-0105
- Source analysis:
  - `results/analysis/inc0105_product_phase_soft_sparse_translation_upper_bank_carry_forward_screen.json`
  - `results/analysis/inc0105_product_phase_soft_sparse_translation_upper_bank_carry_forward_confirm.json`
- Reports:
  - `docs/reports/INC0105_PRODUCT_PHASE_SOFT_SPARSE_TRANSLATION_UPPER_BANK_CARRY_FORWARD_SCREEN.md`
  - `docs/reports/INC0105_PRODUCT_PHASE_SOFT_SPARSE_TRANSLATION_UPPER_BANK_CARRY_FORWARD_CONFIRM.md`
- Gate notes:
  - `docs/governance/gates/gate_20260312_121941.md`
  - `docs/governance/gates/gate_20260312_122439.md`
- Reading:
  - the lower-bank sparse translated systems lead survives at the upper bank
  - the branch is positive/narrow on systems carry-forward, not on quality
    recovery
  - the next honest branch is sparse translated systems cost decomposition

## Analysis INC-0106
- Source analysis:
  - `results/analysis/inc0104_product_phase_soft_sparse_translation_backfill_recovery_confirm.json`
  - `results/analysis/inc0105_product_phase_soft_sparse_translation_upper_bank_carry_forward_confirm.json`
  - `results/analysis/inc0106_product_phase_sparse_translation_systems_cost_decomposition.json`
- Reports:
  - `docs/reports/INC0106_PRODUCT_PHASE_SPARSE_TRANSLATION_SYSTEMS_COST_DECOMPOSITION.md`
- Reading:
  - lower-bank bounded backfill gain is search-dominated on average
  - upper-bank gain remains real on mean but mixes search, route-query, and
    route-index effects
  - no hidden accounting regression surfaced

## Analysis INC-0107
- Source analysis:
  - `results/analysis/inc0104_product_phase_soft_sparse_translation_backfill_recovery_confirm.json`
  - `results/analysis/inc0105_product_phase_soft_sparse_translation_upper_bank_carry_forward_confirm.json`
  - `results/analysis/inc0107_product_phase_sparse_translation_component_stability_audit.json`
- Reports:
  - `docs/reports/INC0107_PRODUCT_PHASE_SPARSE_TRANSLATION_COMPONENT_STABILITY_AUDIT.md`
- Reading:
  - lower-bank backfill versus the continuous translated product reference is
    stable at `4/4` seed wins
  - upper-bank bounded backfill remains mean-positive but not seed-uniform
  - candidate-fraction reduction is stable, while `route_query` changes sign
    across seeds
  - the next honest branch is repeated timing hardening on the fixed sparse
    translated pairs

## Analysis INC-0108
- Source analysis:
  - `results/analysis/inc0104_product_phase_soft_sparse_translation_backfill_recovery_confirm.json`
  - `results/analysis/inc0105_product_phase_soft_sparse_translation_upper_bank_carry_forward_confirm.json`
  - `results/analysis/inc0108_product_phase_sparse_translation_repeated_timing_hardening_lower_r2.json`
  - `results/analysis/inc0108_product_phase_sparse_translation_repeated_timing_hardening_lower_r3.json`
  - `results/analysis/inc0108_product_phase_sparse_translation_repeated_timing_hardening_upper_r2.json`
  - `results/analysis/inc0108_product_phase_sparse_translation_repeated_timing_hardening_upper_r3.json`
  - `results/analysis/inc0108_product_phase_sparse_translation_repeated_timing_hardening.json`
- Reports:
  - `docs/reports/INC0108_PRODUCT_PHASE_SPARSE_TRANSLATION_REPEATED_TIMING_HARDENING.md`
- Gate notes:
  - `docs/governance/gates/gate_20260312_130158.md`
  - `docs/governance/gates/gate_20260312_130648.md`
  - `docs/governance/gates/gate_20260312_130722.md`
  - `docs/governance/gates/gate_20260312_131452.md`
- Reading:
  - repeated timing hardening does not recover a clean seed-stable wallclock
    story for the sparse translated systems leads
  - candidate-fraction reduction remains stable across every repeated
    comparison
  - the bounded backfill points remain valid on mean, but microtiming remains
    too noisy for direct optimization guidance
  - the next honest branch is a robust cost-reference audit on the completed
    sparse translated evidence

## Analysis INC-0109
- Source analysis:
  - `results/analysis/inc0104_product_phase_soft_sparse_translation_backfill_recovery_confirm.json`
  - `results/analysis/inc0105_product_phase_soft_sparse_translation_upper_bank_carry_forward_confirm.json`
  - `results/analysis/inc0108_product_phase_sparse_translation_repeated_timing_hardening_lower_r2.json`
  - `results/analysis/inc0108_product_phase_sparse_translation_repeated_timing_hardening_lower_r3.json`
  - `results/analysis/inc0108_product_phase_sparse_translation_repeated_timing_hardening_upper_r2.json`
  - `results/analysis/inc0108_product_phase_sparse_translation_repeated_timing_hardening_upper_r3.json`
  - `results/analysis/inc0109_product_phase_sparse_translation_robust_cost_reference.json`
- Reports:
  - `docs/reports/INC0109_PRODUCT_PHASE_SPARSE_TRANSLATION_ROBUST_COST_REFERENCE.md`
- Reading:
  - upper-bank bounded backfill remains a clean robust sparse translated
    systems lead versus both routed baselines
  - lower-bank bounded backfill remains robust versus the continuous
    translated product reference
  - lower-bank bounded backfill versus the fixed soft sparse translated
    reference narrows to a pruning-first read rather than a clean robust
    wallclock promotion
  - candidate-fraction and candidate-count reduction are now the clean robust
    sparse translated signals
  - the next honest branch is repeated dense-frontier hardening on the fixed
    sparse translated anchors

## Analysis INC-0110
- Source analysis:
  - `results/analysis/inc0104_product_phase_soft_sparse_translation_backfill_recovery_confirm.json`
  - `results/analysis/inc0105_product_phase_soft_sparse_translation_upper_bank_carry_forward_confirm.json`
  - `results/analysis/inc0108_product_phase_sparse_translation_repeated_timing_hardening_lower_r2.json`
  - `results/analysis/inc0108_product_phase_sparse_translation_repeated_timing_hardening_lower_r3.json`
  - `results/analysis/inc0108_product_phase_sparse_translation_repeated_timing_hardening_upper_r2.json`
  - `results/analysis/inc0108_product_phase_sparse_translation_repeated_timing_hardening_upper_r3.json`
  - `results/analysis/inc0110_product_phase_sparse_translation_dense_robust_hardening.json`
- Reports:
  - `docs/reports/INC0110_PRODUCT_PHASE_SPARSE_TRANSLATION_DENSE_ROBUST_HARDENING.md`
- Reading:
  - lower-bank soft sparse versus dense narrows to a pruning-first read rather
    than a clean robust systems promotion
  - lower-bank bounded backfill remains the only robust lower-bank dense
    systems promotion
  - both upper-bank sparse translated points remain robust dense systems
    promotions
  - every dense comparison still carries a robust top-1 deficit versus dense
    exact
  - the next honest branch is dense quality-frontier accounting on the fixed
    sparse translated points

## Analysis INC-0111
- Source analysis:
  - `results/analysis/inc0104_product_phase_soft_sparse_translation_backfill_recovery_confirm.json`
  - `results/analysis/inc0105_product_phase_soft_sparse_translation_upper_bank_carry_forward_confirm.json`
  - `results/analysis/inc0110_product_phase_sparse_translation_dense_robust_hardening.json`
  - `results/analysis/inc0111_product_phase_sparse_translation_dense_quality_frontier.json`
- Reports:
  - `docs/reports/INC0111_PRODUCT_PHASE_SPARSE_TRANSLATION_DENSE_QUALITY_FRONTIER.md`
- Reading:
  - lower-bank soft sparse versus dense is `pruning-only`
  - lower-bank bounded backfill versus dense is `systems-only`
  - both upper-bank sparse translated points are `quality-near systems
    promotion`
  - the dense claim is now explicit:
    - lower bank = systems-only
    - upper bank = near-frontier
  - the next honest branch is focused upper-bank dense quality-tolerance
    hardening on the fixed sparse translated points

## Analysis INC-0112
- Source analysis:
  - `results/analysis/inc0105_product_phase_soft_sparse_translation_upper_bank_carry_forward_confirm.json`
  - `results/analysis/inc0108_product_phase_sparse_translation_repeated_timing_hardening_upper_r2.json`
  - `results/analysis/inc0108_product_phase_sparse_translation_repeated_timing_hardening_upper_r3.json`
  - `results/analysis/inc0112_product_phase_sparse_translation_upper_bank_dense_quality_tolerance_hardening_r4.json`
  - `results/analysis/inc0112_product_phase_sparse_translation_upper_bank_dense_quality_tolerance_hardening_r5.json`
  - `results/analysis/inc0112_product_phase_sparse_translation_upper_bank_dense_robust_hardening.json`
  - `results/analysis/inc0112_product_phase_sparse_translation_upper_bank_dense_quality_tolerance_hardening.json`
- Reports:
  - `docs/reports/INC0112_PRODUCT_PHASE_SPARSE_TRANSLATION_UPPER_BANK_DENSE_ROBUST_HARDENING.md`
  - `docs/reports/INC0112_PRODUCT_PHASE_SPARSE_TRANSLATION_UPPER_BANK_DENSE_QUALITY_TOLERANCE_HARDENING.md`
- Reading:
  - both upper-bank sparse translated points remain quality-near dense systems
    promotions under two fresh paired repeats
  - the upper-bank top-1 gap stays small but robustly negative
  - the upper-bank dense claim remains near-frontier, not fully
    quality-matched
  - the next honest branch is residual upper-bank dense top-1 gap
    decomposition on the fixed sparse translated points

## Analysis INC-0113
- Source analysis:
  - `results/analysis/inc0113_product_phase_sparse_translation_upper_bank_dense_gap_decomposition_confirm.json`
  - `results/analysis/inc0113_product_phase_sparse_translation_upper_bank_dense_gap_decomposition.json`
- Reports:
  - `docs/reports/INC0113_PRODUCT_PHASE_SPARSE_TRANSLATION_UPPER_BANK_DENSE_GAP_DECOMPOSITION.md`
- Reading:
  - both fixed upper-bank sparse translated points remain large systems wins
    versus dense exact
  - the residual upper-bank dense top-1 gap is operationally negligible on
    both fixed sparse translated points:
    - soft sparse mean net dense advantage rate `0.001525`
    - bounded backfill mean net dense advantage rate `0.001562`
  - omission explains only about `1.2%-1.4%` of dense-only wins
  - dense-only wins are overwhelmingly present-but-not-top1 and therefore not
    a candidate-recovery surface worth reopening
  - the next honest branch is upper-bank dense reference selection/carry-
    forward, not another quality rescue

## Analysis INC-0114
- Source analysis:
  - `results/analysis/inc0112_product_phase_sparse_translation_upper_bank_dense_quality_tolerance_hardening.json`
  - `results/analysis/inc0113_product_phase_sparse_translation_upper_bank_dense_gap_decomposition.json`
  - `results/analysis/inc0113_product_phase_sparse_translation_upper_bank_dense_gap_decomposition_confirm.json`
  - `results/analysis/inc0114_product_phase_sparse_translation_upper_bank_dense_reference_selection.json`
- Reports:
  - `docs/reports/INC0114_PRODUCT_PHASE_SPARSE_TRANSLATION_UPPER_BANK_DENSE_REFERENCE_SELECTION.md`
- Reading:
  - both upper-bank sparse translated points remain eligible dense-near systems
    promotions
  - the completed pair delta stays inside the explicit carry-forward
    tolerance band:
    - top-1 `+0.000038`
    - candidate fraction `+0.001761`
    - amortized `-0.043834s`
  - `CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000` is now the promoted
    upper-bank dense-near routed reference
  - `CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T40000` drops to a
    supporting comparator
  - the next honest branch is promoted upper-bank carry-forward, not another
    upper-bank fork

## Analysis INC-0115
- Source analysis:
  - `results/analysis/inc0104_product_phase_soft_sparse_translation_backfill_recovery_confirm.json`
  - `results/analysis/inc0113_product_phase_sparse_translation_upper_bank_dense_gap_decomposition_confirm.json`
  - `results/analysis/inc0114_product_phase_sparse_translation_upper_bank_dense_reference_selection.json`
  - `results/analysis/inc0115_product_phase_sparse_translation_promoted_upper_bank_carry_forward.json`
- Reports:
  - `docs/reports/INC0115_PRODUCT_PHASE_SPARSE_TRANSLATION_PROMOTED_UPPER_BANK_CARRY_FORWARD.md`
- Reading:
  - the default broader-comparison packet is now explicit:
    - lower bank: `DENSE_Q01_T2500`,
      `CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500`
    - upper bank: `DENSE_Q01_T40000`,
      `CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000`
  - the lower-bank soft sparse route is now excluded by default
  - the upper-bank bounded-backfill route is now comparator-only
  - the next honest branch is the dual-anchor broader-comparison packet,
    not another upper-bank or lower-bank fork

## Analysis INC-0116
- Source analysis:
  - `results/analysis/inc0115_product_phase_sparse_translation_promoted_upper_bank_carry_forward.json`
  - `configs/proxy_transfer_inc0104_product_phase_soft_sparse_translation_backfill_recovery_confirm.json`
  - `configs/proxy_transfer_inc0113_product_phase_sparse_translation_upper_bank_dense_gap_decomposition_confirm.json`
  - `results/analysis/inc0116_product_phase_sparse_translation_dual_anchor_broader_comparison_packet.json`
  - `configs/packet_inc0116_product_phase_sparse_translation_dual_anchor_broader_comparison.json`
- Reports:
  - `docs/reports/INC0116_PRODUCT_PHASE_SPARSE_TRANSLATION_DUAL_ANCHOR_BROADER_COMPARISON_PACKET.md`
- Reading:
  - the default broader-comparison packet now exists as one reusable packet
    manifest with resolved inherited args
  - the four default routes remain fixed
  - the upper-bank bounded-backfill route stays optional comparator-only
  - the next honest branch is the first broader comparison that inherits this
    packet directly

## Analysis INC-0117
- Source analysis:
  - `results/analysis/inc0111_product_phase_sparse_translation_dense_quality_frontier.json`
  - `results/analysis/inc0116_product_phase_sparse_translation_dual_anchor_broader_comparison_packet.json`
  - `results/analysis/inc0117_product_phase_sparse_translation_dual_anchor_broader_comparison.json`
- Reports:
  - `docs/reports/INC0117_PRODUCT_PHASE_SPARSE_TRANSLATION_DUAL_ANCHOR_BROADER_COMPARISON.md`
- Reading:
  - lower-bank default routed point remains `systems-only`
  - upper-bank default routed point remains `quality-near systems promotion`
  - the upper-bank bounded-backfill route remains explicit but optional
  - the next honest branch is the task-side extension of the same packet

## Analysis INC-0118
- Source analysis:
  - `results/analysis/inc0116_product_phase_sparse_translation_dual_anchor_broader_comparison_packet.json`
  - `results/analysis/inc0117_product_phase_sparse_translation_dual_anchor_broader_comparison.json`
  - `results/analysis/inc0118_product_phase_sparse_translation_dual_anchor_task_side_extension.json`
- Reports:
  - `docs/reports/INC0118_PRODUCT_PHASE_SPARSE_TRANSLATION_DUAL_ANCHOR_TASK_SIDE_EXTENSION.md`
- Reading:
  - the dual-anchor packet now extends directly onto the real-task side
  - lower bank stays systems-only by default
  - upper bank stays quality-near systems promotion by default
  - the upper-bank bounded-backfill route remains optional comparator-only
  - the next honest branch is `INC-0119` explicit dual-anchor real-task
    comparison

## Analysis INC-0119
- Source analysis:
  - `results/analysis/inc0116_product_phase_sparse_translation_dual_anchor_broader_comparison_packet.json`
  - `results/analysis/inc0118_product_phase_sparse_translation_dual_anchor_task_side_extension.json`
  - `results/analysis/inc0119_product_phase_sparse_translation_dual_anchor_real_task_comparison.json`
- Reports:
  - `docs/reports/INC0119_PRODUCT_PHASE_SPARSE_TRANSLATION_DUAL_ANCHOR_REAL_TASK_COMPARISON.md`
- Reading:
  - the first explicit LM-proxy real-task comparison now inherits the exact
    dual-anchor packet
  - lower bank remains systems-only by default
  - upper bank remains quality-near systems promotion by default
  - the upper-bank bounded-backfill route remains optional comparator-only
  - the next honest branch is `INC-0120` downstream real-task carry-forward

## Analysis INC-0120
- Source analysis:
  - `results/analysis/inc0119_product_phase_sparse_translation_dual_anchor_real_task_comparison.json`
  - `results/analysis/inc0120_product_phase_sparse_translation_dual_anchor_real_task_carry_forward.json`
- Reports:
  - `docs/reports/INC0120_PRODUCT_PHASE_SPARSE_TRANSLATION_DUAL_ANCHOR_REAL_TASK_CARRY_FORWARD.md`
- Reading:
  - the explicit LM-proxy real-task comparison now has one downstream
    carry-forward contract
  - lower bank remains systems-only by default
  - upper bank remains quality-near systems promotion by default
  - the upper-bank bounded-backfill route remains optional comparator-only
  - the next honest branch is `INC-0121` downstream packet-manifest

## Analysis INC-0121
- Source analysis:
  - `results/analysis/inc0120_product_phase_sparse_translation_dual_anchor_real_task_carry_forward.json`
  - `results/analysis/inc0121_product_phase_sparse_translation_dual_anchor_real_task_packet_manifest.json`
- Reports:
  - `docs/reports/INC0121_PRODUCT_PHASE_SPARSE_TRANSLATION_DUAL_ANCHOR_REAL_TASK_PACKET_MANIFEST.md`
- Reading:
  - the downstream LM-proxy real-task packet now exists as one exact reusable
    manifest
  - default downstream route ids remain fixed
  - the upper-bank bounded-backfill route remains optional comparator-only
  - the next honest branch is `INC-0122` downstream real-task extension

## Analysis INC-0122
- Source analysis:
  - `results/analysis/inc0120_product_phase_sparse_translation_dual_anchor_real_task_carry_forward.json`
  - `results/analysis/inc0121_product_phase_sparse_translation_dual_anchor_real_task_packet_manifest.json`
  - `results/analysis/inc0122_product_phase_sparse_translation_dual_anchor_real_task_downstream_extension.json`
- Reports:
  - `docs/reports/INC0122_PRODUCT_PHASE_SPARSE_TRANSLATION_DUAL_ANCHOR_REAL_TASK_DOWNSTREAM_EXTENSION.md`
- Reading:
  - downstream LM-proxy real-task inheritance is now explicit from the exact
    packet manifest
  - lower bank remains systems-only by default
  - upper bank remains quality-near systems promotion by default
  - the upper-bank bounded-backfill route remains optional comparator-only
  - the next honest branch is `INC-0123` downstream explicit real-task
    comparison

## Analysis INC-0123
- Source analysis:
  - `results/analysis/inc0121_product_phase_sparse_translation_dual_anchor_real_task_packet_manifest.json`
  - `results/analysis/inc0122_product_phase_sparse_translation_dual_anchor_real_task_downstream_extension.json`
  - `results/analysis/inc0123_product_phase_sparse_translation_dual_anchor_real_task_downstream_comparison.json`
- Reports:
  - `docs/reports/INC0123_PRODUCT_PHASE_SPARSE_TRANSLATION_DUAL_ANCHOR_REAL_TASK_DOWNSTREAM_COMPARISON.md`
- Reading:
  - the explicit downstream LM-proxy real-task comparison is now fixed from
    the completed downstream extension artifact
  - lower bank remains systems-only by default
  - upper bank remains quality-near systems promotion by default
  - the upper-bank bounded-backfill route remains optional comparator-only
  - the next honest branch is `INC-0124` downstream real-task carry-forward

## Analysis INC-0124
- Source analysis:
  - `results/analysis/inc0123_product_phase_sparse_translation_dual_anchor_real_task_downstream_comparison.json`
  - `results/analysis/inc0124_product_phase_sparse_translation_dual_anchor_real_task_downstream_carry_forward.json`
- Reports:
  - `docs/reports/INC0124_PRODUCT_PHASE_SPARSE_TRANSLATION_DUAL_ANCHOR_REAL_TASK_DOWNSTREAM_CARRY_FORWARD.md`
- Reading:
  - the explicit downstream LM-proxy real-task comparison now has one
    downstream carry-forward contract
  - lower bank remains systems-only by default
  - upper bank remains quality-near systems promotion by default
  - the upper-bank bounded-backfill route remains optional comparator-only
  - the next honest branch is `INC-0125` downstream packet manifest

## Analysis INC-0125
- Source analysis:
  - `results/analysis/inc0125_product_phase_sparse_event_proxy_trainability_hardening_screen.json`
  - `results/analysis/inc0125_product_phase_sparse_event_proxy_trainability_hardening_confirm.json`
- Reports:
  - `docs/reports/INC0125_PRODUCT_PHASE_SPARSE_EVENT_PROXY_TRAINABILITY_HARDENING.md`
- Reading:
  - sparse-event proxy trainability survives the harder proxy load on the
    fixed product route law
  - `H4XH4_FIELD_A150_EVT_T070_TAU002` is now the hardened near-hard proxy
    reference
  - `H4XH4_FIELD_A150_EVT_T070` remains the healthy soft-sparse comparator
  - `H4XH4_FIELD_A150_HARD_T062` remains mostly-on and is not a clean hard
    sparse result
  - the next honest branch is `INC-0126` proxy/translation gap audit

## Analysis INC-0126
- Source analysis:
  - `results/analysis/inc0126_product_phase_sparse_event_proxy_translation_gap_audit.json`
- Reports:
  - `docs/reports/INC0126_PRODUCT_PHASE_SPARSE_EVENT_PROXY_TRANSLATION_GAP_AUDIT.md`
- Reading:
  - translated near-hard preserves the same top-1 and candidate fraction as
    translated soft sparse and continuous
  - the translated near-hard failure is systems-cost-only, not a quality or
    candidate-set failure
  - retrieval search is the dominant extra cost term, with route-index build
    second
  - the next honest branch is `INC-0127` translated systems-cost rescue

## Analysis INC-0127
- Source analysis:
  - `results/analysis/inc0127_product_phase_sparse_event_translation_systems_cost_rescue.json`
- Reports:
  - `docs/reports/INC0127_PRODUCT_PHASE_SPARSE_EVENT_TRANSLATION_SYSTEMS_COST_RESCUE.md`
- Reading:
  - translated soft sparse and translated near-hard differ only on
    `event_gate_tau`
  - sparse-event knobs are audit-only on the current translated surface
  - the old translated systems-cost rescue queue is therefore invalid
  - the next honest branch is `INC-0128` route-coupled sparse-event
    translated pilot

## Analysis INC-0128
- Source analysis:
  - `results/analysis/inc0128_product_phase_sparse_event_translation_route_coupled_screen.json`
- Reports:
  - `docs/reports/INC0128_PRODUCT_PHASE_SPARSE_EVENT_TRANSLATION_ROUTE_COUPLED_SCREEN.md`
- Reading:
  - translated sparse-event behavior is no longer audit-only once train-gate
    pruning is wired into the translated train bank
  - soft sparse at threshold `0.02` stays inert with `keep_frac=1.0`
  - near-hard at threshold `0.02` prunes materially but collapses top-1
  - the next honest branch is `INC-0129` route-coupled threshold mapping

## Analysis INC-0129
- Source analysis:
  - `results/analysis/inc0129_product_phase_sparse_event_translation_route_coupled_threshold_map_screen.json`
- Reports:
  - `docs/reports/INC0129_PRODUCT_PHASE_SPARSE_EVENT_TRANSLATION_ROUTE_COUPLED_THRESHOLD_MAP_SCREEN.md`
- Reading:
  - train-gate pruning is genuinely downstream-live
  - threshold `0.010` best preserves translated quality, but still regresses
    runtime versus uncoupled near-hard
  - stronger thresholds reduce candidate fraction further but collapse top-1
  - the next honest branch is `INC-0130` soft translated route-coupling

## Analysis INC-0130
- Source analysis:
  - `results/analysis/inc0130_product_phase_sparse_event_translation_route_coupled_soft_bias_screen.json`
- Reports:
  - `docs/reports/INC0130_PRODUCT_PHASE_SPARSE_EVENT_TRANSLATION_ROUTE_COUPLED_SOFT_BIAS_SCREEN.md`
- Reading:
  - soft score bias is genuinely downstream-live without train-item deletion
  - `SBI030` screened as the balanced lower-bank quality-lift point
  - `SBI080` screened as the strongest quality-first point
  - the next honest branch is `INC-0131` prewarmed lower-bank carry-forward

## Analysis INC-0131
- Source analysis:
  - `results/analysis/inc0131_product_phase_sparse_event_translation_soft_bias_carry_forward_screen.json`
  - `results/analysis/inc0131_product_phase_sparse_event_translation_soft_bias_carry_forward_confirm.json`
- Reports:
  - `docs/reports/INC0131_PRODUCT_PHASE_SPARSE_EVENT_TRANSLATION_SOFT_BIAS_CARRY_FORWARD_CONFIRM.md`
- Reading:
  - uncoupled near-hard is now the lower-bank sparse-event translated systems
    reference
  - `SBI030` is now the balanced lower-bank quality comparator
  - `SBI080` is now the quality-first comparator
  - the old lower-bank bounded-backfill default did not hold on the focused
    prewarmed packet and now requires explicit reselection
  - the next honest branch is `INC-0132` lower-bank reference reselection

## Analysis INC-0132
- Source analysis:
  - `results/analysis/inc0132_product_phase_sparse_event_translation_lower_bank_reference_reselection.json`
- Reports:
  - `docs/reports/INC0132_PRODUCT_PHASE_SPARSE_EVENT_TRANSLATION_LOWER_BANK_REFERENCE_RESELECTION.md`
- Reading:
  - uncoupled near-hard is now the explicit lower-bank sparse-event
    translated default route
  - `SBI030` is now the balanced lower-bank quality comparator
  - `SBI080` is now the quality-first comparator
  - the old lower-bank bounded-backfill route is now stale historical-only
  - the next honest branch is `INC-0133` lower-bank contract refresh

## Analysis INC-0133
- Source analysis:
  - `results/analysis/inc0133_product_phase_sparse_event_translation_lower_bank_contract_refresh.json`
- Reports:
  - `docs/reports/INC0133_PRODUCT_PHASE_SPARSE_EVENT_TRANSLATION_LOWER_BANK_CONTRACT_REFRESH.md`
- Reading:
  - the `INC-0132` lower-bank selection is now inherited consistently across
    broader, task-side, real-task, and downstream contract surfaces
  - default lower-bank route:
    `CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_CPX8_Q01_T2500`
  - explicit lower-bank comparators:
    `CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_SBI030_CPX8_Q01_T2500`,
    `CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_SBI080_CPX8_Q01_T2500`
  - historical-only lower-bank comparator:
    `CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500`
  - the next honest branch is `INC-0134` refreshed dual-anchor real-task
    comparison

## Analysis INC-0134
- Source analysis:
  - `results/analysis/inc0134_product_phase_sparse_event_translation_dual_anchor_real_task_refresh_comparison.json`
- Reports:
  - `docs/reports/INC0134_PRODUCT_PHASE_SPARSE_EVENT_TRANSLATION_DUAL_ANCHOR_REAL_TASK_REFRESH_COMPARISON.md`
- Reading:
  - refreshed real-task comparison reaffirms
    `CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_CPX8_Q01_T2500` as the
    lower-bank systems-only default
  - `SBI030` remains the balanced lower-bank quality comparator
  - `SBI080` remains the quality-first lower-bank comparator and is the first
    lower-bank route to edge dense on top-1
  - upper bank remains unchanged with
    `CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000` as the promoted default
  - the next honest branch is `INC-0135` lower-bank quality/systems frontier

## Prime Transport Recursive System
- Canonical notes:
  - `docs/research/prime_transport_canonical_framework.md`
  - `docs/research/prime_transport_canonical_vs_downstream.md`
  - `docs/research/prime_transport_exact_recursive_system.md`
- Threshold and visibility notes:
  - `docs/research/prime_transport_threshold_law.md`
  - `docs/research/prime_transport_visible_threshold_predictors.md`
  - `docs/research/prime_transport_visible_threshold_extension.md`
  - `docs/research/prime_transport_arrangement_statistics.md`
  - `docs/research/prime_transport_arrangement_isolation.md`
  - `docs/research/prime_transport_interaction_statistics.md`
  - `docs/research/prime_transport_first_splitting_classes.md`
  - `docs/research/prime_transport_density_conditioned_first_splitting.md`
  - `docs/research/prime_transport_tight_density_matched_first_splitting.md`
  - `docs/research/prime_transport_second_tight_density_matched_first_splitting.md`
  - `docs/research/prime_transport_visible_threshold_canonical_law_class.md`
  - `docs/research/prime_transport_exact_layer_synthesis.md`
- Core exact-layer results:
  - `results/prime_transport_recursive_system/threshold_law_summary.csv`
  - `results/prime_transport_recursive_system/threshold_law_detail.csv`
  - `results/prime_transport_recursive_system/visible_threshold_predictor_rows.csv`
  - `results/prime_transport_recursive_system/visible_threshold_predictor_scores.csv`
  - `results/prime_transport_recursive_system/visible_threshold_arrangement_stats.csv`
  - `results/prime_transport_recursive_system/visible_threshold_arrangement_isolation_rows.csv`
  - `results/prime_transport_recursive_system/visible_threshold_arrangement_isolation_scores.csv`
  - `results/prime_transport_recursive_system/visible_threshold_interaction_stats.csv`
  - `results/prime_transport_recursive_system/visible_threshold_interaction_scores.csv`
  - `results/prime_transport_recursive_system/visible_threshold_first_splitting_classes.csv`
  - `results/prime_transport_recursive_system/visible_threshold_first_splitting_event_stats.csv`
  - `results/prime_transport_recursive_system/visible_threshold_first_splitting_scores.csv`
  - `results/prime_transport_recursive_system/visible_threshold_density_conditioned_rows.csv`
  - `results/prime_transport_recursive_system/visible_threshold_density_conditioned_band_scores.csv`
  - `results/prime_transport_recursive_system/visible_threshold_density_conditioned_residual_scores.csv`
  - `results/prime_transport_recursive_system/visible_threshold_tight_density_matched_rows.csv`
  - `results/prime_transport_recursive_system/visible_threshold_tight_density_matched_first_split_classes.csv`
  - `results/prime_transport_recursive_system/visible_threshold_tight_density_matched_scores.csv`
  - `results/prime_transport_recursive_system/visible_threshold_second_tight_density_matched_rows.csv`
  - `results/prime_transport_recursive_system/visible_threshold_second_tight_density_matched_scores.csv`
  - `results/prime_transport_recursive_system/prime_transport_mock_router_trace_eval_summary.csv`
  - `results/prime_transport_recursive_system/prime_transport_mock_router_grouping_eval.csv`
  - `results/prime_transport_recursive_system/prime_transport_mock_router_base_gap_prototype.csv`
  - `results/prime_transport_recursive_system/prime_transport_mock_router_base_gap_refinement.csv`
  - `results/prime_transport_recursive_system/prime_transport_base_gap_routing_loop_eval.csv`
  - `results/prime_transport_recursive_system/prime_transport_base_gap_routing_adapter_demo.csv`
  - `results/prime_transport_recursive_system/prime_transport_base_gap_benchmark_wrapper.csv`
  - `results/prime_transport_recursive_system/prime_transport_guarded_benchmark_harness_eval.csv`
  - `results/prime_transport_recursive_system/prime_transport_base_gap_fallback_burden.csv`
  - `results/prime_transport_recursive_system/prime_transport_guarded_benchmark_hookup_checklist.csv`
  - `results/prime_transport_recursive_system/prime_transport_larger_research_side_benchmark.csv`
  - `results/prime_transport_recursive_system/prime_transport_vs_angular_hopf_head_to_head.csv`
  - `results/prime_transport_recursive_system/angular_hopf_vs_prime_transport_with_transfers.csv`
  - `results/prime_transport_recursive_system/prime_transport_guarded_real_signal_trial.csv`
  - `results/prime_transport_recursive_system/prime_transport_real_signal_reuse_recovery.csv`
  - `results/prime_transport_recursive_system/prime_transport_next_real_signal_slice_checklist.csv`
- Validation Notes:
  - `docs/research/prime_transport_research_side_benchmark_validation.md`
  - `docs/research/prime_transport_vs_angular_hopf_head_to_head.md`
  - `docs/research/angular_hopf_vs_prime_transport_with_transfers.md`
  - `docs/research/prime_transport_routing_decision.md`
  - `docs/research/prime_transport_guarded_real_signal_trial.md`
  - `docs/research/prime_transport_real_signal_reuse_recovery.md`
  - `docs/research/prime_transport_real_signal_reuse_decision.md`
- Reading:
  - `first_internal_split_H = 1` across the tested lifts and tuplets
  - visible split is the nontrivial threshold object
  - lift prime is still the strongest coarse organizer in the current table
  - arrangement matters when `p` and `nu_p(A)` are fixed
  - the first global parent-return/stencil interaction statistics are too weak
    to replace the coarse predictors
  - first-splitting-event statistics are much stronger than those global
    interaction averages
  - first-splitting multiplicity still carries strong signal after coarse
    parent-density conditioning
  - on the first tightly density-matched exact family, first-splitting
    multiplicity stays strong while `gap_max` becomes weak
  - the second tightly matched family reproduces the same pattern on a
    different lift
  - the current best law class is robustly `density + first-splitting event`
    across two exact matched families
  - the exact-layer threshold result is now canonically consolidated as
    `density + first-splitting event`
  - the research-side benchmark validation status is now frozen: `base_gap`
    remains stable across both guarded benchmark stages, while fallback burden
    remains the main explicit guarded cost
  - in the first direct head-to-head against the existing canonical angular
    Hopf routing baseline, `base_gap + fallback` is the stronger bounded
    practical policy; the angular baseline is compact but far too coarse
  - after delayed refinement, fallback accounting, and predictive partitioning
    are transferred back into the unchanged angular Hopf path, the angular side
    recovers resolution only by promoting almost everything, so `base_gap`
    retains a structural advantage rather than just a discipline advantage
  - the current guarded research-side routing decision is now frozen:
    `base_gap + fallback` is the primary candidate going forward, and angular
    Hopf is retained as the practical comparison baseline
  - on the first real MUDBench replay-signal trial, the prime policy retains
    its fallback/resolution advantage, but its current real-signal keying is
    too fine to show route reuse on the tiny bounded traces
  - on the bounded real-signal reuse-recovery test, the zero-reuse issue is
    traced mainly to exact base phase on short traces; the first tiny
    coarsening recovers reuse only by losing the fallback advantage
  - the current real-signal reuse decision is now frozen: keep `base_gap`
    unchanged for now, and move next to the paired `timing_mode_eval_v1`
    baseline replay slice as the smallest fair larger reuse test
  - the exact-layer synthesis note now ties phase-fiber-scale state,
    predictive compression, delayed visibility, and the canonical threshold-law
    class into one reference
  - the offline mock router updates now match the exact bounded traces
  - literal `R_min` route keys are too fine-grained to expose delayed
    refinement in trace operation
  - the best current first coarse routing partition is
    `base_gap = (b, r, next_return_gap)`
  - the first full offline prototype confirms that `base_gap` is strong enough
    to serve as the first serious routing prototype, with near-complete
    post-promotion resolution and materially less fallback than `gap_only`
  - adding `phi0` to `base_gap` reduces promotion but makes the partition much
    finer, so it is not currently a better enough tradeoff to replace
    `base_gap` as the first prototype target
  - the cached base-gap routing loop remains stable across the bounded traces
    and is now mature enough for a guarded non-runtime integration experiment
  - the selected `base_gap` policy is now packaged as a guarded research-side
    adapter boundary suitable for the first future benchmark-harness hookup
  - the same policy is now wrapped around the existing bounded trace-building
    path as the first research-only benchmark boundary, still without touching
    the live MUDBench seam
  - in the first guarded benchmark-harness comparison, `base_gap` sits in the
    intended middle position between the exact-but-address-like static control
    and the overfallback-heavy `gap_only` control
  - the promoted `base_gap` burden is moderately structured overall but not
    concentrated enough to justify one obvious cheap refinement before the
    first guarded benchmark-side integration step
  - the first guarded benchmark-side hookup plan is now specified explicitly
    around the existing research-only wrapper boundary, with fixed controls and
    explicit fallback-accounting requirements
  - the current policy is now explicitly consolidated as ready for a
    research-side integration trial through the same guarded wrapper boundary
  - the first research-side integration trial has now been executed through
    that wrapper boundary and reproduces the prior guarded benchmark result
  - on the next larger reproducible exact-row trace set, `base_gap` remains
    stable and its fallback burden does not grow materially
  - the unchanged router-memory branch now supports a bounded workspace-style
    system with exact per-entity retrieval, exact shared-ledger retrieval,
    exact joint snapshots, and zero instability; promoted querying remains the
    main explicit cost
  - the first bounded router-memory agent loop preserves exact retrieval and
    zero instability, but action correctness is only moderate, so the next
    systems step should improve bounded control policy rather than redesign the
    memory substrate
  - a minimal router-memory context packet now improves bounded control quality
    materially while leaving route reuse, promoted-query burden, and stability
    unchanged
  - a stronger bounded ranking controller now improves packet-surface control
    quality again without changing routing, memory, promotion, packet
    structure, or stability
  - a minimal attractor-identity layer improves bounded control quality again
    without harming retrieval, but it does not reduce promoted-query burden, so
    it currently behaves as a convergence aid rather than a cost fix
  - the current best bounded controller stack is now frozen as unchanged
    router-memory plus unchanged packet surface plus attractor identity, and
    the next prototype is a harder bounded reassignment / dependency
    coordination loop on that same stack
  - the first harder coordination-logic prototype keeps exact retrieval and
    zero instability on the same stack, while exposing mixed full-loop
    coordination quality as the next main controller-side problem
  - the router-native branch now has a compact internal demo package with one
    overview note, one manifest, the canonical stack, the key drivers and CSVs,
    the open costs, and the immediate next experiment fixed for restartability
  - a first controller-only coordination-policy improvement on the same packet
    plus attractor surface raises reassignment / handoff correctness to `1.0`
    and modestly improves full-loop coordination quality without changing
    retrieval, route reuse, promoted-query burden, or instability
  - a small one-step lookahead / replanning controller on the same unchanged
    stack does not beat the improved local coordination policy, while all
    substrate-side metrics remain fixed
  - an explicit conflict / dependency arbitration controller on the same
    unchanged stack also does not beat the improved local coordination policy
    and matches the rejected lookahead aggregate on the bounded bundles
  - a small explicit coordination-frame / transaction layer on the same
    unchanged stack slightly improves the harder full-loop coordination metric
    over the improved local controller and now stands as the best bounded
    higher-level controller variant on this branch
  - a first explicit geometry-native controller signal based on delayed-
    visibility / branch-depth zone preserves the coordination-frame result
    exactly but does not improve it further
  - a richer phase-fiber-aware geometry controller based on a bounded
    `(r_band, b_sector, phi_bucket, gap_band)` basin tag degrades coordination
    quality materially while leaving all substrate-side metrics fixed
  - the first direct geometry-native sequence-model test now shows that a
    bounded explicit geometry engine over `(b, phi, r, next_return_gap)` can
    outperform a tiny causal transformer on a small phase-fiber-controlled
    symbolic sequence task, especially on the state-carrying query steps
  - the second direct geometry-native sequence-model test now shows that the
    same geometry-native style also beats a tiny transformer on a stronger
    bounded stack-rewrite compositional task that is less directly aligned to
    the phase-fiber variables
  - the third direct geometry-native sequence-model test now shows that the
    geometry-native style also beats a tiny transformer on a bounded
    discourse-style context-dependent task with overloaded token meaning and
    evolving latent role state
  - the fourth direct geometry-native sequence-model test now shows that the
    geometry-native discourse engine also keeps a strong advantage under a
    held-out structural shift inside that discourse family, rather than only
    on random same-distribution splits
  - the fifth direct geometry-native sequence-model test now shows that the
    geometry-native discourse engine also keeps that advantage under a
    stronger multi-axis held-out shift combining removed query mode, longer
    contexts, and denser style dynamics
  - the sixth direct geometry-native sequence-model test now shows that the
    geometry-native engine also keeps a strong advantage across a related
    bounded task-family boundary, not only within one family and its held-out
    splits
  - the seventh direct geometry-native sequence-model test now shows the first
    clear boundary of the current thesis line: when shared abstract-state
    alignment is reduced enough, the large cross-family advantage collapses to
    near parity overall, though a small query-specific edge remains
  - the first geometry-native chart-realignment test now shows that a small
    local chart recovery rule can reclaim a meaningful part of the v7 loss and
    restore a clear advantage over the tiny transformer, though not the full
    shared-schema transfer strength seen in v6
  - the first bounded multi-chart selector remains better than the v7 collapse
    and better than the tiny transformer, but it does not materially improve on
    the simpler v8 single-rule chart realignment
  - a short prefix-calibrated chart selector also remains better than the v7
    collapse and better than the tiny transformer baseline, but it still does
    not beat the simpler v8 single-rule chart realignment, so v8 remains the
    best reduced-alignment recovery result so far
  - a bounded divisibility-bridge realignment now materially outperforms the
    earlier chart-based recovery line on the same reduced-alignment transfer
    setting, strongly suggesting that arithmetic transition infrastructure is a
    more powerful missing ingredient than better chart selection alone
  - under a stronger reduced-alignment mismatch, the divisibility bridge still
    beats the unrecovered v7 case and the tiny transformer baseline, but its
    large v11 gain weakens sharply and falls back below v8, so fixed
    divisibility hubs are promising but not yet robust
  - a short support-window bridge-family selector now recovers part of that
    v12 loss, materially beating the fixed v12 bridge and slightly beating the
    older v8 chart recovery, though it still remains far below the v11 peak
  - allowing one bounded within-sequence bridge switch improves the stronger-
    mismatch line again beyond v13, which supports local temporal bridge
    recalibration as the next useful adaptive-geometry capability, though the
    result still remains far below the v11 peak
  - a one-shot event-triggered switch based on local coherence drop does not
    beat the simpler fixed midpoint switch from v14, so adaptive switching
    still looks plausible but the trigger criterion remains open
  - a GCD-style factor-overlap bridge revision remains competitive and beats
    the fixed v12 bridge plus the tiny transformer baseline, but it still does
    not beat the current best v14 midpoint switch
  - a query/binding-conflict-triggered factor revision improves on weaker
    adaptive variants such as the generic event-triggered switch, but it still
    does not beat the current best v14 midpoint switch
  - a very small conflict-centered repair window does not improve on the
    broader factor-revision line and also does not beat the current best v14
    midpoint switch
  - a moderate contradiction-centered repair region improves on the
    micro-window variant but still does not beat the current best v14 midpoint
    switch, so repair scale alone remains insufficient
  - contradiction-grown regional repair becomes the strongest factor-repair
    variant so far and gets close to the current best v14 midpoint switch on
    overall accuracy, but it still does not beat it and remains clearly behind
    on query accuracy
  - a bounded regional schema-induction stage becomes the first semi-learned
    stronger-mismatch mechanism to beat the handcrafted repair line and set
    the best query accuracy so far, but it still misses the v14 midpoint
    switch slightly on overall transfer accuracy
  - a bounded two-region hybrid combining coarse segmentation with local
    schema induction becomes the first stronger-mismatch mechanism to beat
    both the v14 overall leader and the v21 query leader on the same setup
  - a lightly induced split-boundary version of that hybrid improves it again
    slightly on both overall accuracy and query accuracy, suggesting boundary
    placement is now the more relevant remaining lever
  - a contradiction-aware boundary score improves the hybrid line further on
    overall accuracy but not on query accuracy, suggesting remaining gains are
    now boundary-objective tradeoffs rather than missing architecture pieces
  - a bounded Pareto / two-objective boundary scorer does not improve the
    hybrid tradeoff and instead falls below both v23 and v24, suggesting this
    bounded hybrid family is now near its local optimum
  - a very small learned regional boundary/regime field then beats the
    handcrafted hybrid family, showing that bounded learned regional structure
    can recover additional headroom without replacing the geometry-native core
  - under a stronger shifted family, that learned regional-field hybrid still
    beats the tiny transformer baseline but loses much of the v26 margin,
    suggesting the architecture direction holds while the current tiny field
    remains underpowered for stronger transfer
  - a slightly richer learned regional field recovers some overall accuracy on
    that stronger shifted family but does not improve query accuracy, which
    suggests the next bottleneck is field structure rather than small scorer
    width increases
  - a shallow structured field over ordered split candidates then improves on
    both v27 and v28, suggesting topology helps more than simple width
    increases
  - an explicit structured chart field over contiguous regions improves query
    accuracy further but gives back some overall accuracy relative to v29, so
    explicit chart structure looks promising without yet becoming the best
    shifted-family solution
  - a small global learned chart field then falls below both v29 and v30 on
    the shifted family, suggesting broader chart scope alone is not enough to
    recover the lost margin
  - a bounded multiscale chart field then recovers some overall accuracy
    relative to the global-field variant, but still fails to beat the better
    shifted-family structured-field results from v29/v30
  - a sparse region-interaction field then fails to improve on the better
    shifted-family regional/structured results, so small region-level routing
    coordination does not appear to unlock the remaining gap
