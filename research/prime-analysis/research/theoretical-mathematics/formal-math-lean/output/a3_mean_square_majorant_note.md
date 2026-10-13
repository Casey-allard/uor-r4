# A3 Mean-Square Majorant Note

Generated: February 17, 2026  
Artifact: `/Users/adminamn/Documents/New project/research/output/a3_mean_square_majorant_refresh_2026-02-17.json`

Deterministic inequality chain:
\[
|H(x)| \le \sqrt{\left(\sum_{i\le k} g_i^2\right)\frac{k}{x}}
\le C_H(\log x)^{A_H}.
\]

Current constants:
- \(A_H=5.2\),
- \(C_H=2.089011392050098\times10^{-4}\).

Checks:
- deterministic \(|H|\)-upper inequality holds on train and held-out grids,
- log-envelope holds on train and held-out grids.

Interpretation:
- This branch is deterministic and valid, but currently looser than the density-transfer branch with \(A_H=4.4\).
- It effectively matches the quadratic-energy direct-\(|H|\) fitting branch on the current grid.
