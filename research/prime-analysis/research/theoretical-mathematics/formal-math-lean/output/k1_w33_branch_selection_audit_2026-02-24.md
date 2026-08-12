# K1 W33 Branch Selection Audit (2026-02-24)

## Question
Which remainder normalization branch is currently required by the formal endpoint chain?

## Formal-side requirement (Lean)
From
- `/Users/adminamn/Documents/New project/research/formal/lean/PrimeRiemannBridgeSpinningTopFrontier.lean:672`
- `/Users/adminamn/Documents/New project/research/formal/lean/PrimeRiemannBridgeSpinningTopFrontier.lean:677`

the endpoint gate is directly:
\[
\left|\frac{R(x)}{x^\beta}\right| \le C x^{-\eta}.
\]

From
- `/Users/adminamn/Documents/New project/research/formal/lean/PrimeRiemannBridgeOscillatoryReduction.lean:1270`
- `/Users/adminamn/Documents/New project/research/formal/lean/PrimeRiemannBridgeOscillatoryReduction.lean:1284`

the existing formal assumptions already include normalized tail behavior:
- `Tendsto (R/x^beta) -> 0`,
- and residual-majorant shape `|residual x| <= C x^{-eta}`.

So the formal scaffolding is currently aligned with **Branch B** (normalized residual form).

## External-source shape
Johnston truncated explicit formula (TeX source):
- `/Users/adminamn/Documents/New project/research/external/papers/src/2411.13791/main.tex:287`

is given at `psi(x)` level:
\[
\psi(x)=x-\sum_{|\Im(\rho)|\le T} \frac{x^\rho}{\rho}+O\!\left(\frac{x(\log x)^2}{T}\right),
\]
which is naturally **Branch A** before dividing by `x^beta`.

## Mismatch
Current formal interface expects Branch B data, while the imported theorem statements are naturally Branch A unless an explicit normalization/translation lemma is added.

## Immediate consequence
The missing math is now precise:
1. Either prove a translation lemma from Branch A to Branch B in the target normalization.
2. Or import/prove an external result already in Branch B normalized form.

Until this translation is formalized, `theta` optimization and `C_256` ledgers remain conditional on branch choice.
