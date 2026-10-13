# R6 Piecewise Majorant Growth Scan (eta fixed)

Fixed eta:
- `eta = 0.034301034952287375`

Model config (per run):
- `beta = 0.55`
- `max_modes = 12`
- `x_min = 1e7`
- `grid_size = 4096`
- `tau_candidate_count = 64`
- `max_zeta_zeros = 20000`

## Results

- x_max=1e18: C=0.28063040574211057, mode_count=12, tail_ratio_sup_to_amp_total=1.141028149290865
- x_max=1e19: C=0.2907788028664586, mode_count=12, tail_ratio_sup_to_amp_total=1.1276617238025803
- x_max=1e20: C=0.3105085436363839, mode_count=10, tail_ratio_sup_to_amp_total=1.374599931385677
- x_max=1e21: C=0.30439193236635864, mode_count=12, tail_ratio_sup_to_amp_total=1.2189595874990642
- x_max=1e22: C=0.2963388677670072, mode_count=12, tail_ratio_sup_to_amp_total=1.223439334278111

## Interpretation

- Finite-window bound remains certifiable with the fixed eta across all tested windows.
- C stays in a narrow band (~0.280630 to ~0.310509) across tested windows.
- This supports stability of the finite-window side; it does not itself prove the all-x asymptotic tail theorem.
