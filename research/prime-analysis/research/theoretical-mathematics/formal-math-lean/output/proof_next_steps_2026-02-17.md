# Proof Next Steps

Generated: 2026-02-17

## What was advanced now

- O3 obligation #1 partially advanced:
  - sign-sensitive lag-tail path now supports deterministic mode (`k_tail=1`) in
    `/Users/adminamn/Documents/New project/research/a3_offdiag_sign_sensitive_lagbound.py`
  - deterministic fallback artifact is held-out valid:
    `/Users/adminamn/Documents/New project/research/output/a3_sign_sensitive_constant_replacement_pack_2026-02-17_deterministic_k1.md`

## What is left to do

1. O3 (still open): replace finite-grid envelope fit with asymptotic theorem constants (`x>=x0`) and derive analytic `E2/x` bound.
2. O2 (open): replace heuristic density model in `tau_infty(M)` with explicit theorem-side zero-density/count bounds.
3. O1 (open): prove uniform residual bound `C0` analytically (not calibration-only).
4. O4 (open): prove base-uniform asymptotic constants over `{30,210,2310,30030}`.
5. O5 (open): map final theorem to a standard RH-equivalent endpoint implication.

## Highest-value next action

- Build an O3 asymptotic handoff note that translates deterministic fallback constants into an explicit
  `forall x>=x0` theorem candidate and quantifies exactly which sub-lemmas are missing for `E2/x`.
