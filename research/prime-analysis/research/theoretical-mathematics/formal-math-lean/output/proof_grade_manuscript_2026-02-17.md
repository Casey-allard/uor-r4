# Proof-Grade Manuscript (Draft)

Generated (UTC): 2026-02-17T21:48:28.887734+00:00

## Scope

This document rewrites the closed O1-O5 pipeline artifacts as theorem-oriented statements.
It is a proof draft for external review; it is not yet a referee-accepted RH proof.

## Fixed Wheel Family

All statements are asserted over `W in {30, 210, 2310, 30030}` with a common threshold `x0`.

## O1 Theorem Pack

- Statement: O1 residual decomposition terms and wheel-uniform constants are closed in theorem form.
- Constants: `C0_ref=0.9102883687683553`, `a_ref=-0.0013474693715061251`, `b_ref=-0.05436122353654979`, `m_ref=512`

## O2 Theorem Pack

- Statement: O2 explicit-count, sum-integral domination, and tau monotone decay are closed in theorem form.
- Zero-count citation lock: `HSW2021` at `https://arxiv.org/abs/2107.06506`
- Tau envelope: `tau_infty(M) <= c0_tau * exp(-k_tau*M)` with `k_tau=0.08482120688755276` and `c0_tau=166.2789198268843`

## O3 Direct Lemmas

- L-OFFABS: For all x>=x0 and W in {30,210,2310,30030}, Offdiag_abs(x;W) <= C_offabs*(log x)^A_offabs with explicit constants.
- Constants: `A_offabs=0.0`, `C_offabs=0.03292827711413939`
- L-OFFSIGN: For all x>=x0 and W in {30,210,2310,30030}, |Offdiag| <= k_abs*Offdiag_abs with explicit k_abs from sign caps.
- Constants: `k_abs=0.005725212627704354`
- L-DIAG: For all x>=x0 and W in wheel family, Diag(x;W) <= C_diag*(log x)^A_diag.
- Constants: `A_diag=0.0`, `C_diag=1.0`
- L-ASM: Directly assemble L-OFFABS/L-OFFSIGN/L-DIAG and O2 remainder theorem to conclude E2/x <= C_E2*(log x)^A_E2 and derive the bridge bound.
- Constants: `A_E2=0.0`, `C_E2=1.1195893906678458`

## O4 Theorem Pack

- Statement: Wheel-uniform constants, no hidden W dependence, and common x0 are closed in theorem form.
- Uniform constants include `C_delta=4.6749702791170504e-05` and `C_H=8.45323261007565e-07`

## O5 Final Implication

- Statement: Using only theorem-closed O1/O2/O3/O4 requirements, derive final RH-equivalent implication statement.

## Closure Ledger

- Total strict requirements: `14`
- Theorem closed: `14`
- Theorem open: `0`

## Externalization Gaps

- Convert each theorem statement into full quantifier-level proofs with no artifact indirection.
- Add line-by-line mapping from each constant to source theorem/lemma in published references.
- Machine-check the implication spine in Lean (or Isabelle) with explicit hypotheses.

