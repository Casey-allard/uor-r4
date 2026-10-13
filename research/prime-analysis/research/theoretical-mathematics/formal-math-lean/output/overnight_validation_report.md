# Overnight Validation Report

Generated: 2026-02-16T22:50:57.439747+00:00

Passed gates: 5/6

- PASS `pair_mse_improves_with_more_zeros`: pair_mse 2k=1.046367, 8k=0.986724
- PASS `wigner_mse_improves_with_more_zeros`: wigner_mse 2k=0.014885, 8k=0.011937
- PASS `explicit_rmse_improves_with_more_zeros`: rmse z64=23.300617, z256=15.001274
- PASS `explicit_scaled_residual_std_improves`: scaled_std z64=0.092882, z256=0.061634
- FAIL `top_sparse_model_transfer_positive`: transfer_r2_min=-0.101162, features=['ent']
- PASS `top_invariant_stability_high`: top_invariant=ent stability=0.995541

Recommendation: promote top formulas/invariants to long-run validation
