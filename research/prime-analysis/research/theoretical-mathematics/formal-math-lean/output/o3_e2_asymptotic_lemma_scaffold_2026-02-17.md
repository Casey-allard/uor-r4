# O3 E2 Asymptotic Lemma Scaffold

Generated: 2026-02-17T19:23:03.381317+00:00

## Quantified Statement

There exist x0>=3, C_E2>=0, A_E2<=A_E2_max such that for all x>=x0 and W in {30,210,2310,30030}, E2(x;W)/x <= C_E2 (log x)^A_E2.

## Target Thresholds

- `A_E2_max = 0.0`
- `C_E2_min_chain_compatibility = 5.751450514658472`
- `eps_sign_assumption = 0.0038168084184695417`
- `neg_over_abs_cap_assumption = 0.0019084042092348122`

## Proof Obligation Slices

- Split E2 into diagonal and offdiag components with explicit remainder control.
- Use quantified sign assumptions to bound offdiag without worst-case absolute loss.
- Derive final asymptotic exponent A_E2 and constant C_E2, uniform in wheel family.

Combined with eta_+ bound and |H|<=sqrt((1+eta_+)E2/x), this discharges O3 bridge-growth theorem target.
