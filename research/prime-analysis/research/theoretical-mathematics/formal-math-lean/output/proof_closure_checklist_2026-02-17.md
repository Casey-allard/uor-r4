# Proof Closure Checklist (Lemma A-E + A1-A4)

Generated: 2026-02-17
Tracker source: `research/output/proof_closure_tracker_2026-02-17.json`

## O1. A1 Residual Bound Analytic Uplift
Goal: prove `sup_{x>=x0,W in W}|E/sqrt(x)-(a_ref H_ref+b_ref)| <= C0` with explicit constants.
Done when:
- `a_ref,b_ref` are no longer regression-defined in the theorem statement (only in optional calibration appendix).
- proof gives explicit `(x0,C0)` and uses no sampled supremum.
- finite-window script check remains a consistency check, not a premise.

## O2. A2 Infinite-Tail Truncation Proof
Goal: prove `Delta_M(x;W) <= C_delta (log x)^beta tau_infty(M)` with explicit `tau_infty(M)->0`.
Done when:
- tail bound beyond `M_ref` is fully analytic and explicit.
- constants are independent of tested grid (`n_values`, `x_step`).
- theorem uses this bound directly, not finite-window uplift replacement.

## O3. A3 Bridge Growth / Offdiag Analytic Closure
Goal: prove `|H_W^{(M)}(x)| <= C_H (log x)^{A_H}` (or equivalent endpoint-safe bound) analytically.
Done when:
- sign-sensitive offdiag/energy control is proven as lemma(s), not calibrated.
- chosen branch (channel-energy or offdiag-dynamic) has theorem-level derivation for `(C_H,A_H)`.
- no fitted envelope is required for theorem statement.

## O4. A4 Base-Uniform Asymptotic Constants
Goal: prove A1-A3 constants are uniform over `W={30,210,2310,30030}` asymptotically.
Done when:
- proof supplies explicit base-uniform constants and admissible `x0`.
- finite-window uniformity probes only validate numerics, not assumptions.

## O5. RH-Equivalent Endpoint Implication
Goal: close implication from A1-A4 to a standard RH-equivalent theorem.
Done when:
- final theorem states full hypotheses with explicit constants and quantifiers.
- reference to RH-equivalent criterion is explicit and complete.
- proof document contains no empirical premise in the implication chain.

## Execution Discipline
- Performance work only when a proof task is blocked by runtime.
- New scripts must keep cache compatibility (`--cache-dir`, shared `--cache-read-dirs`) and reuse existing zero lists/artifacts.
- Every proof-run artifact should label whether it is: `analytic`, `computational check`, or `calibration`.

