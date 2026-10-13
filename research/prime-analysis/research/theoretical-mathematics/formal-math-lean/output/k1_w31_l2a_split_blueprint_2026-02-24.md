# K1 W31 L2A Split Blueprint (2026-02-24)

## Goal
Produce an explicit theorem-grade bound
\[
|\mathcal E_{256}(x)|\le C_{256}x^{-\eta_{256}},\qquad x\ge 10^{21},
\]
with transparent dependence on imported explicit constants.

## Locked inputs
- `N = 256`
- `gamma_256 = 478.942181535`
  (from `/Users/adminamn/Documents/New project/research/data/zeta_zeros_odlyzko_100k.json`)

## Proposed decomposition
For a truncation height `T >= gamma_256`:
\[
\mathcal E_{256}(x)=\mathcal E_{256,\le T}(x)+\mathcal E_{>T}(x)+\mathcal R_T(x),
\]
where:
1. `E_{256,<=T}` is the omitted mode band `256 < |gamma| <= T`.
2. `E_{>T}` is the high-frequency contribution beyond `T`.
3. `R_T` is the truncation remainder from the explicit formula.

This avoids the blocked raw-absolute form `sum_{n>256} 1/gamma_n`.

## Parameter choice for high tail
Adopt
\[
T=\exp(2\omega(x))
\]
as in Bellotti 2025 proof strategy
(`/Users/adminamn/Documents/New project/research/external/papers/src/2508.02041/semi-explicit.tex:620`).

Then
\[
\mathcal R_T(x)=O((\log x)^2/T)
=O(\exp(-\omega(x))\cdot (\log x)^2/\exp(\omega(x))),
\]
which is absorbed into an `exp(-omega(x))` envelope once `omega(x) >= 2 log log x`.

## Source-backed envelope options
1. Bellotti 2025:
\[
\Delta(x)\ll \exp(55A_0)\exp(-\omega(x)).
\]
2. FKS:
\[
E_\psi(x)<9.22022(\log x)^{3/2}\exp(-0.8476836\sqrt{\log x}).
\]
3. Johnston transfer (log-free/VK):
\[
\Delta_i(x)\ll\exp(-\omega(x))\times\text{explicit correction factor}.
\]

These provide explicit high-tail envelopes and remainder control.

## Remaining technical subproblems (L2A)
1. Derive a concrete bound for `E_{256,<=T}` using `N(sigma,T)` and zero-free shape.
2. Propagate constants to a single `C_256, eta_256`.
3. Evaluate whether `eta_256 >= 0.01` is actually achievable at `x >= 10^21`.
4. If not achievable, record exact bottleneck term and required target relaxation.

## Immediate execution plan
1. Build a symbolic constant ledger for each split term (`E_{256,<=T}`, `E_{>T}`, `R_T`).
2. Instantiate with Bellotti 2024/2025 + Johnston + FKS constants.
3. Run numerical sanity checks at `x = 10^21, 10^24, 10^30`.
