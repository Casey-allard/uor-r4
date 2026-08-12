# A3 Offdiag Dynamic (Fixed A_eta) Note

Generated: February 17, 2026  
Artifact: `/Users/adminamn/Documents/New project/research/output/a3_offdiag_dynamic_majorant_eta5p0_sf3.json`

Constrained setup:
- fixed \(\eta\)-exponent: \(A_\eta=5.0\),
- \(\eta\)-safety: `3.0`.

Resulting envelopes:
- \(\eta_+(x)\le C_\eta (\log x)^{5.0}\), with
  \(C_\eta=8.883973385620043\times10^{-2}\),
- \(|H(x)|\le C_H(\log x)^{A_H}\), with
  \((A_H,C_H)=(1.7,\,0.7044246161530919)\).

Validation:
- held-out valid for deterministic dynamic-\(\eta\) upper target,
- held-out valid for final A3 envelope.

Leaderboard impact:
- on `/Users/adminamn/Documents/New project/research/output/a3_branch_leaderboard_2026-02-17.json`
  this branch is the current winner by both lowest \(A_H\) and smallest RHS at \(x=10^{12}\).
