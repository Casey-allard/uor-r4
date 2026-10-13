# K1 W36 Clean Math Path Conclusion (2026-02-24)

## Path executed (math only)
1. Lock theorem-grade Branch-A remainder constants from source TeX.
2. Propagate those constants through the existing A->B translation inequality.
3. Test near-critical `beta` regimes directly against the fixed target `eta = 0.01`.
4. Identify the exact remaining mathematical obstruction.

No Lean remapping/refactoring was done in this step.

## Source lock (theorem-level)
- Cully-Hugill & Johnston, arXiv:2111.10001:
  - explicit truncated formula shape:
    `E(x,T) = O^*(M * x * log x / T)` with admissible `(x_M, alpha, M)` tuples.
  - local source:
    - `/Users/adminamn/Documents/New project/research/external/papers/src/2111.10001/main.tex:128`
    - `/Users/adminamn/Documents/New project/research/external/papers/src/2111.10001/main.tex:851`
  - extracted table values include:
    - `(log x_M = 40, alpha = 1/2, M = 6.431)`
    - `(log x_M = 10^3, alpha = 1/2, M = 5.823)`

## Constant propagation results

### A) Fixed branch `beta_lower = 0.55` with theorem constant `C_A = 6.431`
Artifact:
- `/Users/adminamn/Documents/New project/research/output/k1_w36_branchA_translation_chj2024_M6431_beta055_2026-02-24.md`

Using the admissible range `theta <= alpha = 0.5`:
- best scanned point at `theta = 0.50` gives finite
  - `eta_raw = 0.05`
  - `C_total ~ 2188.22`.

So this branch is mathematically finite and consistent with the translation gate.

### B) Near-critical branch `beta_lower = 0.5001` with explicit Dudek-style `C_A = 2`
Artifact:
- `/Users/adminamn/Documents/New project/research/output/k1_w36_branchA_translation_dudek_C2_beta05001_2026-02-24.md`

Even though rows are pointwise feasible on finite `x` windows, the band contribution is the issue asymptotically.

## Hard obstruction identified (computed and asymptotic)
Artifact:
- `/Users/adminamn/Documents/New project/research/output/k1_w36_eta_band_necessity_probe_2026-02-24.md`

For fixed `eta = 0.01`, `theta = 0.5`, `x1 = 1e21`:
- `beta = 0.5001`: `C_band` explodes with `xmax` (`333 -> 2.85e6`).
- `beta = 0.51`: still grows strongly.
- `beta = 0.52`: stabilizes.
- `beta = 0.55`: stable.

Numerical transition scan (up to `xmax=1e240`) places practical stabilization around `beta ~ 0.517` in this setup.

This matches the analytic condition from the band envelope:
\[
C_{\text{band}} \sim \sup_{x \ge x_1} x^{\eta + 1/2 - \beta}\cdot \mathrm{polylog}(x),
\]
so finite global `x^{-eta}` control requires
\[
\beta > \tfrac12 + \eta \quad (\text{strict}).
\]
With `eta = 0.01`, this means `beta > 0.51`.

## What is now actually closed
- The unknown `C_A` constant is no longer the core blocker in the `beta ~ 0.55` branch.
- We now have theorem-grade source constants and complete numeric propagation for that branch.

## Remaining mathematical blocker (single sentence)
To close the hard gate with fixed `eta = 0.01`, one must prove a uniform lower-gap regime `beta > 0.51` in the right-half-zero forcing chain; without that, the global band majorant constant diverges.

This is the remaining math target, not a formatting or pipeline issue.
