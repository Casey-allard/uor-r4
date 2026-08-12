# Mathlib PR Plan: Gamma Logarithmic Derivative and Digamma

## Status Update (Dec 2025)

The original plan to prove `Gamma_mem_slitPlane_of_re_pos` (that $\Gamma(z)$ never takes negative real values for $Re(z) > 0$) was found to be **incorrect**.
- **Counter-evidence**: Asymptotically, $\arg \Gamma(z) \approx y \ln y - y$. For large $y$, the argument winds around the origin infinitely many times. Thus, $\Gamma(z)$ crosses the negative real axis (and the branch cut of the principal logarithm) infinitely many times in the right half-plane.
- **Consequence**: The definition `digamma z = deriv (fun w => log (Gamma w)) z` using the principal branch of `log` is ill-defined (or at least discontinuous/non-holomorphic) at points where $\Gamma(z)$ is negative real.

## Revised Approach

We have redefined `digamma` algebraically:
```lean
def digamma (z : ℂ) : ℂ := deriv Gamma z / Gamma z
```
This definition is:
1.  Holomorphic everywhere on $\mathbb{C} \setminus \{0, -1, -2, \dots\}$.
2.  Consistent with the standard definition $\psi(z) = \Gamma'(z)/\Gamma(z)$.
3.  Avoids dependence on `Complex.log` and its branch cuts.

## Next Steps

### 1. Digamma Properties (In Progress)
- ✅ `digamma_add_one`: Proven using algebraic properties of $\Gamma$.
- ✅ `digamma_nat`: Proven using `deriv_Gamma_nat`.
- ⏳ `digamma_series`: Requires establishing the convergence of the Euler sequence derivatives.
  - Strategy: Show locally uniform convergence of `logGammaSeq` (or its derivative directly) on $Re(z) > 0$.
- ⏳ `digamma_gauss_integral`: Prove the integral representation.

### 2. Binet Kernel and Stirling Formula
- The Binet integral representation for `log Gamma` needs to be stated carefully to avoid branch cut issues.
- `logGamma_eq_stirling_plus_J` should arguably use a continuous branch of `log Gamma` on the right half-plane (which exists since $\Gamma$ is non-zero), rather than `Complex.log (Gamma z)`.
- Alternatively, state the result for $\psi(z)$ (Stirling for digamma) first, which is single-valued.
  $\psi(z) \approx \ln z - 1/(2z) - ...$

## Legacy Plans (Discarded)
- `Gamma_mem_slitPlane_of_re_pos`: Discarded.
- Holomorphic Logarithm infrastructure: Still useful generally, but not needed for the basic definition of `digamma`.
