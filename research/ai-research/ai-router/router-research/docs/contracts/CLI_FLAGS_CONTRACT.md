# CLI Flags Contract

## Required existing flags
- `--sector_mode {kmeans,phase2,phase4d,phase4d_adaptive,phase4d_hopf,phase4d_hopf_iso,phase4d_hopf_ball,phase4d_hopf_chi,phase4d_hopf_fib,phase4d_hopf_fib_rung,phase4d_hopf_fib_band,phase4d_hopf_fib_band_iso,phase4d_hopf_fib_band_bound,phase4d_hopf_blend,phase4d_complex_local,complex2}`
- `--phase_dims i,j`
- `--phase4_dims i,j,k,l`
- `--complex_dims i,j`
- `--hopf_chi_bins`
- `--hopf_blend_lambda`
- `--hopf_blend_chi_weight`
- `--hopf_blend_shell_weight`
- `--hybrid_local_k`
- `--hybrid_complex_roots`
- `--hybrid_local_min_k`
- `--hybrid_local_target`
- `--hybrid_local_hysteresis`
- `--hybrid_local_converge_lambda`
- `--adaptive_min_pair_bins`
- `--adaptive_time_growth`
- `--adaptive_balance`
- `--adaptive_angle_growth`
- `--adaptive_shell_growth`
- `--adaptive_shell_balance`
- `--adaptive_converge_lambda`
- `--adaptive_converge_target`
- `--adaptive_converge_hysteresis`
- `--adaptive_converge_mode {fixed,phi_ratio,phi_ladder}`
- `--fib_rung_gate_threshold`
- `--route_scale_lambda`
- `--memory_coord_mode {route_chart,full_chart}`
- `--shell_mode {linear,phi_log,phi_phase}`
- `--shell_phase_coupling`
- `--time_pressure_lambda`
- `--train_route_mode {dynamic,final_static}`
- `--recluster_after_chart`

## Runtime controls
- `--fast_dev {0,1}`
- `--early_stop_patience`
- `--early_stop_min_delta`
- `--cache_dir`
- `--cache_chart {0,1}`
- `--cache_routes {0,1}`
- `--run_tag`

## Compatibility
- Existing baseline scripts remain valid.
- New flags are additive and have safe defaults.
- `train_route_mode=dynamic` is the compatibility default.
- `phase4d_hopf`, `phase4d_hopf_iso`, `phase4d_hopf_ball`, `phase4d_hopf_chi`, `phase4d_hopf_fib`, `phase4d_hopf_fib_rung`, `phase4d_hopf_fib_band`, `phase4d_hopf_fib_band_iso`, `phase4d_hopf_fib_band_bound`, and `phase4d_hopf_blend` are active research modes.
- `phase4d_complex_local` remains experimental, but `INC-0020` rescued it into a healthy routed-quality branch.
- `shell_mode=linear` is the compatibility default.
