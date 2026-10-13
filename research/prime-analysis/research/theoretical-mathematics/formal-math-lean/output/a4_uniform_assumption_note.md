# A4 Uniform Assumption Note (Unified Finite-Window Constant Pack)

Generated: February 17, 2026
Artifact: `/Users/adminamn/Documents/New project/research/output/a4_uniform_assumption_check_none_z128_ref512.json`

## Unified Setup
- kernel=`none`
- `M=128`, `M_ref=512`
- bases `W in {30,210,2310,30030}`
- scales up to `x=5e6`
- exponents fixed: `beta=2.6`, `A_H=7.2`

## Extracted Uniform Constants
- `a_ref = -0.001347471211160521`
- `b_ref = -0.05436128386957597`
- `C0_ref = 0.6334997534895853`
- `tau_M = 383.99980444354367` (tail_Fp, from `M=128` to `M_ref=512`)
- `C_delta = 4.674970279117053e-05`
- `C_H = 8.45323261007565e-07`

## Combined Theorem-RHS Check
Checked bound:
\[
|E(x)|/\sqrt{x} \le |a_{ref}|C_H(\log x)^{A_H}+|b_{ref}|+C0_{ref}+|a_{ref}|C_\Delta(\log x)^\beta\tau_M.
\]

Result on tested grid:
- holds: `True`
- violations: `0`
- max gap `(lhs-rhs)`: `-0.18056509535525345`
- ratio range `lhs/rhs`: `[~1.1e-05, 0.774694]`

## Interpretation
1. A single uniform constant pack exists that covers all tested bases/scales.
2. The bound is conservative (significant slack), which is expected at this stage.
3. This is the first complete finite-window realization of A1-A4 working together.

## Next Proof Task
Replace this calibrated pack by analytic asymptotic constants with explicit threshold `x0`, and reduce conservativeness while preserving uniformity in base.
