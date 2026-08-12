# A3 Offdiag Dynamic (Fixed A_eta=4) Note

Generated: February 17, 2026  
Artifact: `/Users/adminamn/Documents/New project/research/output/a3_offdiag_dynamic_majorant_eta4p0_sf3.json`

Constrained setup:
- fixed \(\eta\)-exponent: \(A_\eta=4.0\),
- \(\eta\)-safety: `3.0`.

Resulting envelopes:
- \(\eta_+(x)\le C_\eta (\log x)^{4.0}\), with
  \(C_\eta=1.2170134478356474\),
- \(|H(x)|\le C_H(\log x)^{A_H}\), with
  \((A_H,C_H)=(1.2,\,2.607225675383175)\).

Validation:
- held-out valid for deterministic dynamic-\(\eta\) upper target,
- held-out valid for final A3 envelope.

Exponent-constraint support:
- `/Users/adminamn/Documents/New project/research/output/a3_eta_exponent_probe_2026-02-17.md`
  shows required safety factors for fixed \(A_\eta\in\{4.0,4.5,5.0,5.5,6.0,6.5\}\);
  fixed-\(A_\eta=4.0\) is feasible under safety budget `3.0`.
- `/Users/adminamn/Documents/New project/research/output/a3_eta4_justification_probe_2026-02-17_sf3.md`
  confirms held-out validity for \(A_\eta=4.0\) with `safety=3.0` on stress split.

Structural caveat:
- normalized trend \( \eta_+(x)/(\log x)^4 \) is not monotone on current sampled windows,
  so this remains a calibrated guardrail (not yet a proved asymptotic monotonic theorem).

Leaderboard impact:
- on `/Users/adminamn/Documents/New project/research/output/a3_branch_leaderboard_2026-02-17.json`
  this is the current winner by both lowest \(A_H\) and smallest RHS at \(x=10^{12}\).

Stress validation:
- extended split (`valid` up to `10^7`) with denser `x_step=2500` is documented in
  `/Users/adminamn/Documents/New project/research/output/a3_offdiag_dynamic_stress_note.md`.
