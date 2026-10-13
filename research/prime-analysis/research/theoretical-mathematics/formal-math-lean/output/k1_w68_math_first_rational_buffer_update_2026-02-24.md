# K1 W68 Math-First Update (2026-02-24): Rational Branch Buffer Without Fitted Phase

## Scope
Advance remaining math lemmas directly, with no Lean/plumbing work.

## New Artifacts
- `/Users/adminamn/Documents/New project/research/output/k1_w68_tau12_phase_lock_tied_fit_2026-02-24.json`
- `/Users/adminamn/Documents/New project/research/output/k1_w68_tau12_phase_lock_tied_fit_2026-02-24.md`
- `/Users/adminamn/Documents/New project/research/tau12_rational_uniform_buffer_certificate.py`
- `/Users/adminamn/Documents/New project/research/output/k1_w68_tau12_rational_uniform_buffer_2026-02-24.json`
- `/Users/adminamn/Documents/New project/research/output/k1_w68_tau12_rational_uniform_buffer_2026-02-24.md`
- `/Users/adminamn/Documents/New project/research/output/k1_w68_tau12_rational_uniform_buffer_conservative_2026-02-24.json`
- `/Users/adminamn/Documents/New project/research/output/k1_w68_tau12_rational_uniform_buffer_conservative_2026-02-24.md`
- `/Users/adminamn/Documents/New project/research/output/k1_w68_c2_dichotomy_closure_draft_2026-02-24.md`

## What We Ran
1. Tied-phase estimator stress test (`phi2 = rho*phi1 + beta0`) across five windows.
2. New denominator-based rational-buffer certificate that does not use fitted `beta0`.

## Key Results
1. Tied-fit phase-lock did not resolve drift in a theorem-useful way:
   - `beta0` collapsed into two clusters separated by `4.188790e-02`.
   - Drift fit gave near-zero exponent (`eta_hat ~ -4.1e-16`) with this cluster artifact.
   - Conclusion: this estimator does not provide the needed asymptotic-phase object.

2. Rational branch can be buffered structurally without fitted `beta0`:
   - For reduced `rho=p/q`, the periodic orbit has `q` equally spaced points, so
     `max_j cos(2*pi*p*j/q + beta0) >= cos(pi/q)` for any `beta0`.
   - Therefore if a denominator lower bound `q >= q_min` is certified, rational-branch C2 holds with
     `c0 = cos(pi/q_min)` and infinite repetition by periodicity.

3. Interval-based denominator exclusions from current tau inputs:
   - With `tau` absolute errors `1e-6` each:
     - `rho interval = [1.4872618279, 1.4872621798]`
     - no `p/q` hits for any `q <= 1000` (scan result),
     - so conditional bound `q >= 1001` and `c0 >= cos(pi/1001)`.
   - With conservative `tau` absolute errors `0.05` each:
     - `rho interval = [1.4784946080, 1.4960916473]`
     - first denominator hit at `q=25`,
     - still `q=2` excluded and `c0 >= cos(pi/25)`.

4. Robust exclusion of `q=2` (critical case):
   - distance from the conservative interval to `3/2` is `3.908352e-03`.
   - Since `rho in (1,2)`, excluding `3/2` removes the only denominator-2 possibility.
   - This removes dependence on fitted `cos(beta0)` for the rational C2 branch.

5. C2 dichotomy packet drafted at fixed positive buffer:
   - New draft proposition closes `C2(1/2)` by irrational/rational case split,
     using equidistribution on the irrational branch and periodic geometry on the rational branch.
   - This moves C2 from fit-stability concerns to a clean denominator-certification assumption.

## Mathematical Impact on Open Lemmas
1. C2 buffered-anchor branch:
   - Rational side is now reduced to interval-certified denominator exclusion (especially `q=2`).
   - No phase-fit lock of `beta0` is needed for this branch.
2. Rounding-preservation:
   - unchanged structurally (already reduced to symbolic thresholds in W66).
3. One-sided tail `C*x^{-eta}`:
   - unchanged; still the principal hard analytic gate.

## Immediate Next Math Step
Write the final C2 dichotomy proposition in one theorem packet:
- irrational case via equidistribution,
- rational case via the new periodic geometry + denominator certificate,
then move the frontier to one-sided tail constants as the main remaining blocker.
