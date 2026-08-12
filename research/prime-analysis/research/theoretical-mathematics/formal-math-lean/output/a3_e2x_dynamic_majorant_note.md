# A3 E2/x Dynamic Majorant Note

Generated: February 17, 2026  
Artifact: `/Users/adminamn/Documents/New project/research/output/a3_e2x_dynamic_majorant_refresh_2026-02-17.json`

Model:
\[
\eta_+(x)\le C_\eta(\log x)^{A_\eta},\qquad
\frac{E_2(x)}{x}\le C_q(\log x)^{A_q},
\]
\[
|H(x)|\le \sqrt{(1+\eta_+(x))\frac{E_2(x)}{x}}
\le C_H(\log x)^{A_H}.
\]

Current calibrated constants:
- \(\eta\)-envelope: \(A_\eta=6.6\), \(C_\eta=8.991144645550242\times10^{-4}\),
- \(E_2/x\)-envelope: \(A_q=0\), \(C_q=1.4084530153302305\times10^{-1}\),
- A3 envelope: \(A_H=3.3\), \(C_H=1.1257801357429618\times10^{-2}\).

Checks:
- held-out valid for the deterministic composite upper target and final A3 envelope.

Interpretation:
- This branch reduces \(C_H\) substantially versus offdiag-dynamic default,
  but raises exponent (\(3.3\) vs \(2.5\)); leaderboard at \(x=10^{12}\) still favors the \(A_H=2.5\) branch.
