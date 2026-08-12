# K1 W39 Contradiction Route Research Note (2026-02-24)

## Objective
Continue the math-first contradiction route on a fixed canonical error:

\[
E_\*(x) := \psi(x) - x,\quad
\text{(right-half zero)} \Rightarrow \text{oscillation lower bound} \Rightarrow \text{conflict with endpoint upper bound}.
\]

## Route skeleton (mathematical form)

Target contradiction template:

1. Lower (oscillation) side:
\[
\exists \beta > \tfrac12,\ \exists c_\beta>0,\ \forall X\ \exists x\ge X:\ |E_\*(x)| \ge c_\beta x^\beta.
\]

2. Endpoint upper side (von-Koch type class):
\[
\exists C_{\mathrm{vk}}<\infty,\ \forall x\ge x_0:\ |E_\*(x)| \le C_{\mathrm{vk}} x^{1/2}(\log x)^2.
\]

3. If both hold with \(\beta>\frac12\), then for sufficiently large \(x\):
\[
c_\beta x^\beta > C_{\mathrm{vk}}x^{1/2}(\log x)^2,
\]
which is impossible. This is the contradiction gate.

## New finite-window empirical probe (canonical function only)

Script:
- `/Users/adminamn/Documents/New project/research/fixed_error_psi_contradiction_probe.py`

Artifacts:
- `/Users/adminamn/Documents/New project/research/output/k1_w39_fixed_error_psi_contradiction_probe_2026-02-24.json`
- `/Users/adminamn/Documents/New project/research/output/k1_w39_fixed_error_psi_contradiction_probe_2026-02-24.md`
- `/Users/adminamn/Documents/New project/research/output/k1_w39_fixed_error_psi_contradiction_probe_2026-02-24_x3e7.json`
- `/Users/adminamn/Documents/New project/research/output/k1_w39_fixed_error_psi_contradiction_probe_2026-02-24_x3e7.md`

### Window A (`x in [10^4, 10^7]`)
- `C_endpoint_sup_window = 6.490395126e-03` for empirical envelope
  \[
  |E_\*(x)| \le C_{\text{endpoint}}\,x^{1/2}(\log x)^2
  \]
  on sampled points.
- Tail-peak lower-envelope constants `c_beta` (q10 over tail local peaks):
  - `beta=0.55`: `c_beta≈4.49e-02`
  - `beta=0.58`: `c_beta≈2.89e-02`
  - `beta=0.60`: `c_beta≈2.16e-02`
  - `beta=0.62`: `c_beta≈1.64e-02`
- Corresponding crossover scales from
  \[
  c_\beta x^\beta = C_{\text{endpoint}}x^{1/2}(\log x)^2
  \]
  (q10 constants):
  - `beta=0.55`: `x≈9.28e71`
  - `beta=0.58`: `x≈2.31e41`
  - `beta=0.60`: `x≈1.48e32`
  - `beta=0.62`: `x≈2.32e26`

### Window B (`x in [10^4, 3*10^7]`)
- `C_endpoint_sup_window = 7.686628693e-03`
- Tail-peak q10 constants:
  - `beta=0.55`: `c_beta≈4.33e-02`
  - `beta=0.58`: `c_beta≈2.68e-02`
  - `beta=0.60`: `c_beta≈1.95e-02`
  - `beta=0.62`: `c_beta≈1.44e-02`
- q10 crossover scales:
  - `beta=0.55`: `x≈2.05e74`
  - `beta=0.58`: `x≈1.43e43`
  - `beta=0.60`: `x≈5.92e33`
  - `beta=0.62`: `x≈7.20e27`

## Interpretation

1. The canonical fixed-function dataset shows strong oscillatory behavior (`psi(x)-x` sign changes are frequent in sample windows) and persistent tail local-peak lower constants for each tested `beta>1/2`.
2. The empirical contradiction map behaves as expected: larger `beta` gives earlier crossover scales.
3. Crossovers occur far beyond sampled windows, which is normal for this growth-vs-log-squared conflict.
4. This is still finite-window evidence. It does not yet provide theorem quantifiers (`forall X exists x>=X`) or theorem-grade constants.

## Exact math obligations still needed for theorem closure of this route

The route now has explicit missing ingredients:

1. **Lower theorem constant (`c_beta`)**:
   derive from explicit formula + right-half-zero hypothesis (fixed canonical `E*`), with non-circular remainder control.

2. **Upper theorem constant (`C_vk`)**:
   import/prove endpoint class at theorem level for the same fixed canonical error function and hypothesis set.

3. **Quantifier upgrade**:
   replace sampled tail-peak claims with an infinite-sequence or eventual-oscillation statement.

Once (1)-(3) are theorem-grade, the contradiction gate is algebraically immediate.

