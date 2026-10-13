# Canonical H(x) Definition (Frozen v1)

Generated: 2026-02-17 (local)

This file freezes the current bridge functional used in experiments.

## 1) R^4 Manifold

For integer `n >= 2` and zero list `{gamma_j}_{j=1..M}`:

\[
F(u) = \sum_{j=1}^{M}\frac{e^{i\gamma_j u}}{\tfrac12+i\gamma_j},\quad
F'(u) = \sum_{j=1}^{M}\frac{i\gamma_j e^{i\gamma_j u}}{\tfrac12+i\gamma_j},\quad
u=\log n.
\]

\[
X_n=(\Re F(\log n),\Im F(\log n),\Re F'(\log n),\Im F'(\log n))\in\mathbb{R}^4.
\]

## 2) Base-Filtered Sequence

For wheel base `W`, keep integers `n` with `gcd(n,W)=1`.

From manifold trajectory, define channels:
- `Y11s(n)` (real spherical harmonic component)
- `Y20(n)` (zonal harmonic component)
- `su2_trace(n)` (trace of local SU(2) transport)
- `phase_vel(n)` (first phase difference)

## 3) Frozen Weights

Current frozen weights (order: `Y11s,Y20,su2_trace,phase_vel`):

\[
(w_1,w_2,w_3,w_4)=(-1,-1,0,-1).
\]

Hence:

\[
G_B(n)=-Y11s(n)-Y20(n)-phase\_vel(n).
\]

## 4) Bridge Functional

\[
H(x)=\sum_{n\le x,\ \gcd(n,W)=1}G_B(n).
\]

Primary comparison target:

\[
E(x)=\psi(x)-x,\qquad
H_s(x)=\frac{H(x)}{\sqrt{x}},\quad
E_s(x)=\frac{E(x)}{\sqrt{x}}.
\]

Bridge diagnostics are based on:
- `corr(H_s, E_s)`
- linear fit `E_s ≈ a H_s + b`
- fit residual scale (`RMSE`).

## 5) Scope

This is a frozen empirical definition for theorem-development work.
Changing channel map, weights, or scaling requires a new versioned definition file.
