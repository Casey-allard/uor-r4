# O3 E2 Offdiag Sign Symbolic Draft

Generated: 2026-02-17T19:25:54.335359+00:00

## Lemma Target

- `E2-OFFDIAG-SIGN`
- There exist x0>=3, C_E2>=0, A_E2<=A_E2_max such that for all x>=x0 and W in {30,210,2310,30030}, E2(x;W)/x <= C_E2 (log x)^A_E2.

## Assumptions

- `eps_sign = 0.0038168084184695417`
- `neg_over_abs_cap = 0.0019084042092348122`
- `pos_over_abs_floor = 0.9961831915815305`

## Proposed Symbolic Inequality

|Offdiag(x;W)| <= k_pos * Offdiag_pos(x;W) + k_neg * Offdiag_neg(x;W) <= k_abs * Offdiag_abs(x;W)

## Coefficients

- `k_pos = 0.0038168084184695417`
- `k_neg = 0.0019084042092348122`
- `k_abs = 0.005725212627704354`

This is a theorem-draft placeholder preserving sign structure by tracking positive/negative offdiag channels before fallback to absolute mass.

## Remaining To Formalize

- Provide explicit definitions of Offdiag_pos, Offdiag_neg, Offdiag_abs in the E2 decomposition.
- Prove asymptotic bounds for these components uniformly in W and x>=x0.
- Derive an explicit offdiag exponent A_offdiag compatible with A_E2 target.
