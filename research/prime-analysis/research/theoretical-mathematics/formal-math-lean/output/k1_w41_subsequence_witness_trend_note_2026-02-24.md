# K1 W41 Subsequence Witness Trend Note (2026-02-24)

## Objective
Stress-test the phase-aligned subsequence witness channel across increasing windows for fixed
`beta = 0.62`, using the W40 ledger method.

## Runs
Temporary trend artifacts:
- `/Users/adminamn/Documents/New project/research/output/tmp_k1_w41_beta062_witness_trend_x1000000.json`
- `/Users/adminamn/Documents/New project/research/output/tmp_k1_w41_beta062_witness_trend_x3000000.json`
- `/Users/adminamn/Documents/New project/research/output/tmp_k1_w41_beta062_witness_trend_x10000000.json`
- `/Users/adminamn/Documents/New project/research/output/tmp_k1_w41_beta062_witness_trend_x30000000.json`

Common settings:
- `xmin=1e4`, `samples=2200`, `tail_frac=0.5`, `|cos|>=0.98`,
- witness grid: `40` thresholds up to `0.95*xmax`.

## Numeric trend (beta = 0.62)

| xmax | tau_best | rho_sup | w_obs_grid | w_tri_grid | tri_pos_frac | x_cross(w_obs) | x_cross(w_tri) |
|---:|---:|---:|---:|---:|---:|---:|---:|
| 1e6 | 14.134725 | 3.736178 | 6.106087e-02 | 2.785901e-02 | 0.5448 | 2.628380e+20 | 4.083214e+24 |
| 3e6 | 14.134725 | 3.803631 | 3.630360e-02 | 3.243456e-02 | 0.5704 | 2.027220e+23 | 7.855551e+23 |
| 1e7 | 14.134725 | 4.178262 | 6.676215e-02 | 2.427575e-02 | 0.5664 | 8.236734e+19 | 2.054893e+25 |
| 3e7 | 14.134725 | 3.323375 | 2.197168e-02 | 1.632886e-02 | 0.6809 | 7.395372e+24 | 2.324954e+26 |

## Interpretation
1. Dominant phase frequency is stable (`tau_best = 14.134725142`) over all tested windows.
2. Subsequence witness constants remain positive in every window:
   - `w_obs_grid > 0`
   - `w_tri_grid > 0`
3. Uniform remainder domination remains out of reach (`rho_sup > 3`), consistent with W40.
4. The data favors a **subsequence/limsup** contradiction route rather than a global uniform remainder route.

## Next math step
Move from numeric witness to theorem shape:
- derive a subsequence remainder lemma that yields infinitely many phase-aligned `x_n` with
  \[
  A|\cos(\tau\log x_n+\phi)|-|R(x_n)| \ge c > 0,
  \]
  for fixed canonical `E*(x)=psi(x)-x`, then combine with endpoint upper bound.

