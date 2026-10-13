# K1 L2A Split Ledger (2026-02-24)

## Config
- zeta_zeros_file: research/data/zeta_zeros_odlyzko_100k.json
- max_zeta_zeros: 100000
- n_head: 256
- beta: 0.55
- x_min: 1e+21
- x_max: 1e+30
- grid_size: 1024
- zero_chunk: 512
- eta_target: 0.01
- a0: 0.020802216684209868
- cutoff_policy: omega
- xpow: 0.5

## Locked Values
- gamma_n: 478.942181535
- omega_threshold_for_tcut_eq_gamma_n: 3.085789941758885
- x_where_exp2omega_hits_gamma_n: 1.3371924055358497e+64

## Cutoff Summary
- t_raw_min: 27.732299389308576
- t_raw_max: 57.03625147059352
- t_cut_min: 478.942181535
- t_cut_max: 478.942181535
- k_cut_min: 256
- k_cut_median: 256
- k_cut_max: 256
- band_count_min: 0
- band_count_median: 0
- band_count_max: 0
- band_count_nonzero_fraction: 0.0

## Component Majorants
- e_total: sup=1.583544e-02, rmse=3.538861e-03, eta_fit=0.056611, C_fit=2.905526e-01, C_eta0.01=2.579155e-02
- e_band: sup=0.000000e+00, rmse=0.000000e+00, eta_fit=0.000000, C_fit=1.000000e-300, C_eta0.01=0.000000e+00
- e_high: sup=1.583544e-02, rmse=3.538861e-03, eta_fit=0.056611, C_fit=2.905526e-01, C_eta0.01=2.579155e-02
- eta_target_pass_empirical: True

## External Envelopes At Eta Target
- c_fks: 1.384904e+01
- c_bellotti_2025_model: 9.669252e-01

## Interpretation
- Split-ledger uses finite 100k-zero surrogate plus external explicit envelopes; it diagnoses where cutoff choice blocks or enables L2A constant propagation.
