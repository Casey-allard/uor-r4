# K1 W33 L2A Normalization Gate (2026-02-24)

## Why this gate matters
Current L2A constants depend heavily on which truncation remainder normalization is mathematically valid for the target quantity
\[
\left|\frac{R(x)}{x^\beta}\right|.
\]

## Two normalization branches

### Branch A (E-level truncated explicit formula)
If the remainder is imported from a formula of shape
\[
E(x)=\text{(truncated zero sum)}+O\!\left(\frac{x(\log x)^2}{T}\right),
\]
then after dividing by `x^beta`:
\[
\left|\frac{R_{\text{rem}}(x)}{x^\beta}\right|
\lesssim
\frac{x^{1-\beta}(\log x)^2}{T}.
\]
For schedule `T(x)=x^theta`, target `eta` requires
\[
\frac{x^{1-\beta}(\log x)^2}{x^\theta}\le Cx^{-\eta}
\quad\Longrightarrow\quad
\theta \ge 1-\beta+\eta
\]
(up to logarithmic slack).

Numerical threshold at locked values:
- `beta = 0.55`, `eta = 0.01` gives `theta >= 0.46`.
- worst-case `beta -> 0.5+` gives `theta >= 0.51`.

### Branch B (phase-residual / already-normalized remainder)
If imported theorem already controls the normalized residual at phase level:
\[
|residual(x)|\le C x^{-\eta},
\]
then no extra `x^(1-beta)` factor appears, and smaller `theta` schedules are admissible.

## Current repo alignment evidence
Formal target in:
- `/Users/adminamn/Documents/New project/research/formal/lean/PrimeRiemannBridgeSpinningTopFrontier.lean:672`
- `/Users/adminamn/Documents/New project/research/formal/lean/PrimeRiemannBridgeSpinningTopFrontier.lean:677`
is explicitly on `|R(x)/x^beta|`, i.e. normalized residual.

But imported external explicit-formula lemmas (e.g. Johnston truncated RVM formula)
are naturally written at the `psi(x)-x` level before this normalization.

Hence this is a real gate: we must prove which branch the current chain legitimately lands in.

## Implication for current theta candidates
- `theta = 0.19` is best under current model-A ledger if Branch B-style normalization is permitted in the imported chain.
- If Branch A is the correct imported normalization, then `theta=0.19` is not asymptotically admissible for `eta=0.01`; we need `theta >= 0.46` (or larger depending on beta regime).

## Next mathematical obligation
Prove a branch-selection lemma:
1. either map external truncated formula remainder into normalized `R/x^beta` with exact factor and show resulting admissible `theta`;
2. or import/derive a theorem already in Branch B normalized form.

Until this branch is fixed, `C_256, eta_256` is not theorem-final.
