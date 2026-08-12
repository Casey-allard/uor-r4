# A3 Offdiag-Energy Majorant Note

Generated: February 17, 2026  
Artifacts:
- `/Users/adminamn/Documents/New project/research/output/a3_offdiag_energy_majorant_refresh_2026-02-17.json`
- `/Users/adminamn/Documents/New project/research/output/a3_offdiag_energy_majorant_refresh_2026-02-17_sf5.json`

Model:
\[
S_k^2 = E_2 + \mathrm{OffDiag},\qquad
|H| = \frac{|S_k|}{\sqrt{x}}
\le \sqrt{\frac{(1+\eta)E_2}{x}},
\]
where \(\eta\) upper-bounds positive \(\mathrm{OffDiag}/E_2\).

## Outcome
- Split-trained \(\eta\) (default safety) fails held-out (`eta_valid_holds=false`).
- Conservative held-out-valid run (`offdiag_safety=5.0`) succeeds but is too loose:
  - \(A_H=0\),
  - \(C_H \approx 1.0584794493532736\times10^2\).

## Interpretation
- Off-diagonal positive ratios grow significantly out-of-split on this grid.
- This branch is currently useful as a stress-test / guardrail diagnostic, not as the preferred A3 theorem constant path.
