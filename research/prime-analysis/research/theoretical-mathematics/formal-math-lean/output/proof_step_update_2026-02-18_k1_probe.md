# Proof Step Update (2026-02-18): K1 Numerical Probe

Ran:

`python3 research/explicit_formula_probe.py --zeros-file research/data/zeta_zeros_odlyzko100k_2026-02-18.json --zero-limit 256 --x-max 120000 --x-step 400 --moduli 30,210,2310 --radius-power 1.3 --output research/output/explicit_formula_probe_k1_2026-02-18.json`

Outputs:
- `research/output/explicit_formula_probe_k1_2026-02-18.json`
- `research/output/explicit_formula_probe_k1_2026-02-18.md`
- `research/output/explicit_formula_probe_k1_2026-02-18.svg`

Key metrics:
- `corr_exact_vs_estimate = 0.9999999067`
- `rmse = 15.0036`
- `scaled_residual_std = 0.05950`

Interpretation:
- Truncated explicit-formula reconstruction is extremely correlated with exact `psi(x)` on this finite range.
- This supports the decomposition strategy used in K1-style formulations, but remains numerical evidence (not a formal non-circular proof of K1).
