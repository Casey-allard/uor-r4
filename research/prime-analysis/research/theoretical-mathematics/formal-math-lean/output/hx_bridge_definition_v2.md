# Canonical H(x) Definition (Frozen v2, Kernelized)

Generated: 2026-02-17 (local)

This version supersedes v1 for current bridge experiments.

## 1) Kernelized Zero-Sum Manifold

For zero list `{gamma_j}` and kernel weight `K(gamma_j)`:

\[
F(u)=\sum_{j=1}^{M}K(\gamma_j)\frac{e^{i\gamma_j u}}{\tfrac12+i\gamma_j},\quad
F'(u)=\sum_{j=1}^{M}K(\gamma_j)\frac{i\gamma_j e^{i\gamma_j u}}{\tfrac12+i\gamma_j},
\]

with:

\[
u=\log n,\qquad
K(\gamma)=\exp\!\big(-(\gamma/\sigma)^2\big),\ \sigma=100.
\]

Define:

\[
X_n=(\Re F(\log n),\Im F(\log n),\Re F'(\log n),\Im F'(\log n))\in\mathbb{R}^4.
\]

## 2) Base-Filtered Sequence

For wheel base `W`, keep `n` with `gcd(n,W)=1`.

Channels used:
- `Y11s(n)`
- `Y20(n)`
- `su2_trace(n)`
- `phase_vel(n)`

## 3) Frozen Weights

Order `(Y11s, Y20, su2_trace, phase_vel)`:

\[
(w_1,w_2,w_3,w_4)=(-1,-1,0,-1).
\]

Thus:

\[
G_B^{(v2)}(n)=-Y11s(n)-Y20(n)-phase\_vel(n).
\]

## 4) Bridge Functional

\[
H(x)=\sum_{n\le x,\ \gcd(n,W)=1}G_B^{(v2)}(n),\qquad
E(x)=\psi(x)-x.
\]

Scale:

\[
H_s(x)=\frac{H(x)}{\sqrt{x}},\qquad E_s(x)=\frac{E(x)}{\sqrt{x}}.
\]

Diagnostics:
- `corr(H_s,E_s)`
- fit `E_s \approx a H_s + b`
- fit RMSE
- truncation sensitivity across zero cutoff `M`.

## 5) Scope

This is an empirical bridge definition for theorem-development.
Any change to kernel/weights/channels requires a new version.
