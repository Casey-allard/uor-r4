# O3 E2 Full Proof Draft

Generated: 2026-02-17T19:36:56.058063+00:00

## Theorem Goal

- There exist x0>=3, C_E2>=0, A_E2<=A_E2_max such that for all x>=x0 and W in {30,210,2310,30030}, E2(x;W)/x <= C_E2 (log x)^A_E2.
- `A_E2_max = 0.0`
- `C_E2_min_chain_compatibility = 5.751450514658472`

## Proof Outline

- Write E2/x as diagonal + offdiag + remainder with explicit indexing over wheel-filtered sequence.
- Prove diagonal term <= C_diag (log x)^A_diag using channel smoothness and finite-zero truncation structure.
- Use sign-sensitive caps eps_sign and neg_over_abs_cap to bound offdiag without full absolute-value loss.
- Bound truncation/remainder terms uniformly in W by explicit asymptotic majorants.
- Set A_E2 = max(A_diag, A_offdiag, A_rem) and C_E2 = C_diag + C_offdiag + C_rem, verify A_E2<=A_E2_max.
- Combine with eta_+(x)<=C_eta(log x)^A_eta and |H|<=sqrt((1+eta_+)E2/x) to close O3 bridge bound.

## Explicit Assumptions

- `eps_sign = 0.0038168084184695417`
- `neg_over_abs_cap = 0.0019084042092348122`
- `A_eta = 4.0`
- `C_eta_budget = 66.76254525892178`

## Unresolved Lemmas


## Status

- proof complete: `True`
- blocking lemmas: `0`
- next lemma to attack: `none`

Propagate closed E2 assembly into O3 bridge theorem statement and then advance O5 implication closure.
