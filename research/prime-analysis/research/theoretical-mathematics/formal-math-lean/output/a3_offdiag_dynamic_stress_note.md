# A3 Offdiag Dynamic Stress Validation

Generated: February 17, 2026

Stress artifacts:
- A3 stress run: `/Users/adminamn/Documents/New project/research/output/a3_offdiag_dynamic_majorant_eta4p0_sf3_stress_2026-02-17.json`
- A4 stress run: `/Users/adminamn/Documents/New project/research/output/a4_uniform_assumption_check_stress_2026-02-17.json`
- Full stress pack: `/Users/adminamn/Documents/New project/research/output/uplift_theorem_pack_refresh_2026-02-17_stress_full.json`

Settings:
- train `n={3e5,1e6,2e6}`, valid `n={5e6,1e7}`,
- denser sampling `x_step=2500`.

Results:
- A3 stress constants: \((A_H,C_H)=(1.1,4.526748350185781)\), held-out valid.
- A4 stress theorem RHS check: held on grid up to `n=1e7`.
- Full stress pack held with strict margin:
  - max ratio `lhs/rhs = 0.008230738837252427`,
  - min gap `rhs-lhs = 74.81059379161977`.

Selection note:
- At reference asymptotic scale (`x=1e12`), non-stress winner
  `/Users/adminamn/Documents/New project/research/output/a3_offdiag_dynamic_majorant_eta4p0_sf3.json`
  remains better by `C_H (log x)^{A_H}` than stress constants, so it stays default.
