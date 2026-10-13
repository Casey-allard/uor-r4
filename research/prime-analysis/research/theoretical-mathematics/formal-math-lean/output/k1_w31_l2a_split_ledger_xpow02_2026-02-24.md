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
- cutoff_policy: xpow
- xpow: 0.2

## Locked Values
- gamma_n: 478.942181535
- omega_threshold_for_tcut_eq_gamma_n: 3.085789941758885
- x_where_exp2omega_hits_gamma_n: 1.3371924055358497e+64

## Cutoff Summary
- t_raw_min: 15848.931924611137
- t_raw_max: 1000000.0000000003
- t_cut_min: 15848.931924611137
- t_cut_max: 1000000.0000000003
- k_cut_min: 17236
- k_cut_median: 100000
- k_cut_max: 100000
- band_count_min: 16980
- band_count_median: 99744
- band_count_max: 99744
- band_count_nonzero_fraction: 1.0

## Component Majorants
- e_total: sup=1.583544e-02, rmse=3.538861e-03, eta_fit=0.056611, C_fit=2.905526e-01, C_eta0.01=2.579155e-02
- e_band: sup=1.370519e-02, rmse=3.504021e-03, eta_fit=0.054747, C_fit=2.614307e-01, C_eta0.01=2.232197e-02
- e_high: sup=2.606316e-03, rmse=3.549716e-04, eta_fit=2.422129, C_fit=1.043033e+55, C_eta0.01=4.256163e-03
- eta_target_pass_empirical: True

## External Envelopes At Eta Target
- c_fks: 1.384904e+01
- c_bellotti_2025_model: 9.669252e-01

## Interpretation
- Split-ledger uses finite 100k-zero surrogate plus external explicit envelopes; it diagnoses where cutoff choice blocks or enables L2A constant propagation.
