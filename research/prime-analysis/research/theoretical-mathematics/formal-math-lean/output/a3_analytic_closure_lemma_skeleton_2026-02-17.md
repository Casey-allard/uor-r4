# A3 Analytic-Closure Lemma Skeleton

Generated: 2026-02-17T16:57:05.666956+00:00

## Candidate Statement

For wheel family W and all x >= x0: |H_W(x)| <= C_H (log x)^A_H, obtained via eta_+(x;W) <= C_eta (log x)^A_eta and |H| <= sqrt((1+eta_+) E2/x).

## Proof-Primary Branch

- `offdiag_dynamic_eta_fixed_Aeta4`

## Frozen Constants

- `A_eta`: `4.0`
- `C_eta_uplifted`: `1.2170134478356474`
- `A_H_primary`: `1.2`
- `C_H_primary`: `2.607225675383175`
- `A_H_stress`: `1.0999999999999999`
- `C_H_stress`: `4.526748350185781`
- `eta_safety`: `3.0`
- `m_zero`: `128`
- `zero_kernel`: `none`
- `kernel_scale`: `0.0`
- `bases`: `[30, 210, 2310, 30030]`

## Validation Snapshot

- `primary_train_holds`: `True`
- `primary_valid_holds`: `True`
- `primary_valid_ratio_max`: `0.945541199909`
- `primary_valid_max_gap_h_minus_rhs`: `-3.78542946895`
- `stress_valid_holds`: `True`
- `stress_valid_ratio_max`: `0.80328355081`
- `stress_valid_max_gap_h_minus_rhs`: `-18.5460812574`
- `eta4_just_valid_ratio_over_c_eta`: `0.69321408041`

## Analytic Obligations (O3)

- Replace empirical eta safety uplift by explicit sign-sensitive offdiag inequality constants.
- Prove base-uniform eta_+(x;W) <= C_eta(log x)^A_eta for all x>=x0 over W in {30,210,2310,30030}.
- Bind E2(x;W)/x analytically from Lemma A/B ingredients, independent of sampled grids.
- Derive asymptotic x0 and final C_H,A_H from theorem constants (not fitted envelopes).

## Sources

- `research/output/a3_offdiag_dynamic_majorant_eta4p0_sf3.json`
- `research/output/a3_offdiag_dynamic_majorant_eta4p0_sf3_stress_2026-02-17.json`
- `research/output/a3_eta_exponent_probe_2026-02-17.json`
- `research/output/a3_eta4_justification_probe_2026-02-17_sf3.json`
