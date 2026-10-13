# A3 Offdiag Dynamic Majorant Note

Generated: February 17, 2026  
Artifact: `/Users/adminamn/Documents/New project/research/output/a3_offdiag_dynamic_majorant_refresh_2026-02-17.json`

Model:
\[
\eta_+(x):=\max\!\left(0,\frac{\mathrm{OffDiag}}{E_2}\right),\quad
\eta_+(x)\le C_\eta (\log x)^{A_\eta},
\]
\[
|H(x)| \le \sqrt{\frac{(1+\eta_+(x))E_2}{x}}
\le C_H(\log x)^{A_H}.
\]

Current calibrated constants:
- \(\eta\)-envelope: \(A_\eta=6.6\), \(C_\eta=8.991144645550242\times10^{-4}\),
- A3 envelope: \(A_H=2.5\), \(C_H=7.086646136830307\times10^{-2}\).

Checks:
- held-out valid for both dynamic-\(\eta\) deterministic upper target and final A3 envelope,
- stricter exponent than current primary density-transfer branch (\(2.5\) vs \(4.4\)),
- but larger constant, so finite-window RHS is still conservative.
- piecewise-constant \(\eta(x)\) bin model (8 bins) was tested and failed held-out deterministically, so global-log \(\eta\) is retained as default.
- tight-constant variant (`h_safety=1.0`) was also tested and stayed valid, but moved to \(A_H=2.8\) (higher than the dynamic baseline exponent).

Update:
- constrained fixed-\(A_\eta\) run (`A_eta=5.0`, `eta_safety=3.0`) now yields
  a held-out-valid branch with \((A_H,C_H)=(1.7, 0.7044246161530919)\).
- further constrained run (`A_eta=4.0`, `eta_safety=3.0`) yields
  held-out-valid \((A_H,C_H)=(1.2, 2.607225675383175)\), now top-ranked at \(x=10^{12}\);
  see `/Users/adminamn/Documents/New project/research/output/a3_offdiag_dynamic_eta4_fixed_note.md`.

Interpretation:
- This is a promising asymptotic-shape branch (lower exponent) with deterministic offdiag handling.
- Next tightening target is reducing \(C_H\) by sharpening \(E_2/x\) scaling and reducing slack in the fitted \(\eta_+(x)\) envelope.
