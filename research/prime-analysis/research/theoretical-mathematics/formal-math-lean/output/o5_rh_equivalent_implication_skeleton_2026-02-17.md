# O5 RH-Equivalent Implication Skeleton

Generated: 2026-02-17T19:03:58.379511+00:00

## Fixed Endpoint Criterion

- `|psi(x)-x| <= C sqrt(x) (log x)^2`
- target exponent: `2.0`
- anchor: `AIM RH equivalence index (article 95a)`
- explicit-bound anchor: `Math. Comp. 30 (1976), Theorem 10`

## Mref Route

- `m_ref=512`
- `Delta_M=0` at `M=M_ref` (A2 exponent does not drive endpoint class here).
- `A_H=1.2`, `C_H=3.47471643488075`
- meets target exponent class: `True`
- exponent gap `A_H-target`: `-0.8`

## Theorem Skeleton

Assume, for x >= x0 and W in {30,210,2310,30030}, the M_ref transfer inequality |E(x)|/sqrt(x) <= C0_ref + |b_ref| + |a_ref||H_W^(M_ref)(x)|, and bridge growth |H_W^(M_ref)(x)| <= C_H (log x)^A_H. Then |E(x)| <= C sqrt(x) (log x)^A with A=max(A_H,0), C depending on C0_ref,b_ref,a_ref,C_H.

### Required Assumptions

- A1_ref: theorem-level uniform bound for reference residual at M_ref.
- A3_ref: theorem-level bound |H_W^(M_ref)(x)| <= C_H (log x)^A_H.
- A4_uniform: constants are uniform over W in {30,210,2310,30030}.

### Proof Outline

- Start from deterministic affine transfer inequality at M_ref.
- Substitute A3_ref bound for bridge term.
- Collect additive constants into C and exponent into A.
- Compare obtained endpoint class against fixed RH-equivalent target form.

## Remaining O5 Blockers

- Convert grid-validated M_ref transfer inequality into an asymptotic theorem with explicit hypotheses.
- Prove A3_ref asymptotically without empirical safety-factor dependence.
- Provide final implication write-up to a standard RH-equivalent statement.
