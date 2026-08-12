# Lemma B Note: Truncation Envelope Calibration

Generated: February 17, 2026
Program artifact: `/Users/adminamn/Documents/New project/research/output/lemma_b_truncation_program_gauss100_ref512.json`

## Statement Template
For wheel family `W in {30,210,2310,30030}`, define truncated bridge process `H_W^{(M)}(x)` and reference `H_W^{(M_ref)}(x)`.
Candidate finite-window envelope:
\[
B_M(X) := \max_{W}\sup_{x\le X}\left|H_W^{(M)}(x)-H_W^{(M_{ref})}(x)\right|.
\]

Goal for Lemma B (theorem form later): show `B_M(X)` decreases in `M` and tends to `0` with explicit rate/bounds.

## Run Configuration
- kernel = gaussian(scale=100)
- `M_ref = 512`
- tested `M in {64,128,192,256}`
- `X in {3e5, 1e6}`
- `x_step = 2000`
- weights `(-1,-1,0,-1)`

## Empirical Bounds (Global Worst Across Bases and Tested X)
- `B_64 <= 1.1427371695007338e-01`
- `B_128 <= 7.994631679153485e-04`
- `B_192 <= 8.81680755959735e-07`
- `B_256 <= 8.807621298956292e-11`

Monotonicity check (all bases, both X values): **True**.

## Interpretation
1. Strong decay in `M` is observed (roughly three sharp phases: coarse at 64, tight at 128, near-converged by 192, numerically frozen by 256).
2. This supports a practical working regime where `M>=192` gives near-reference stability for the tested windows.
3. These are still finite-window calibrations, not asymptotic proofs.

## Link To Analytic Tail Terms
For same run, analytic coordinate-tail terms at `M=256` are:
- `tail_F_bound <= 1.3546846701849444e-12`
- `tail_Fp_bound <= 6.643992170229376e-10`

These provide a candidate route to convert numeric `B_M(X)` calibration into theorem-level bounds.

## Next Step For Lemma B
Prove an explicit inequality of form
\[
B_M(X) \le C_1(X)\sum_{j>M}\frac{K(\gamma_j)}{|1/2+i\gamma_j|} + C_2(X)\sum_{j>M}\frac{\gamma_jK(\gamma_j)}{|1/2+i\gamma_j|},
\]
then control `C_1(X),C_2(X)` uniformly over wheel family.
