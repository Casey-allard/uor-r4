# K1 W66 Math Lemma Derivation Package (2026-02-24)

## Goal
Close the three remaining math obligations on the W64 track in a theorem-oriented way:
1. buffered C2 existence,
2. eventual rounding preservation,
3. one-sided M4 split to `q < a1`.

This note is math-first (no new proof-plumbing requirements).

## Definitions
Fix:
- `tau1 = 14.134725142`
- `tau2 = 21.022039639`
- `rho = tau2/tau1`
- `a1 = 0.98`
- normalized signal:
  `Y_beta(x) = A1*cos(phi1(x)) + A2*cos(phi2(x)) + R2(x)` with `A1 > 0`, `A2 >= 0`.

Anchor model:
- exact tau1 anchors `x*_k` satisfy `phi1(x*_k) = 2*pi*k`.
- tau2 phase on anchors:
  `Theta_k = 2*pi*rho*k + beta0`.

## Lemma C2.1 (Buffered anchor existence, case split)
For fixed `c0 in (0,1)`:

1. If `rho` is irrational, Weyl/Kronecker equidistribution of `k*rho mod 1` gives
   `dens{ k : cos(Theta_k) >= c0 } = arccos(c0)/pi > 0`,
   so infinitely many anchors satisfy `cos(Theta_k) >= c0`.

2. If `rho = p/q in Q` (reduced), the sequence is periodic mod `q`.
   Then `cos(Theta_k) >= c0` infinitely often iff at least one residue class has
   `cos(2*pi*p*j/q + beta0) >= c0`.

So buffered C2 existence is fully reduced to:
- irrational-rotation theorem (clean route), or
- finite periodic witness check (rational route).

### Numeric support (not proof)
From `/Users/adminamn/Documents/New project/research/output/k1_w66_tau12_buffered_rotation_density_2026-02-24.json`:
- `c0 in {0,0.2,0.4,0.6,0.8}`,
- `N up to 500000`,
- max absolute gap to `arccos(c0)/pi` across 5 window-fits: `6.672353e-04`.

This strongly supports the irrational-rotation branch in practice.

## Lemma R1 (Anchor-to-integer log bound)
Let `n_k = round(x*_k)` with `x*_k > 1/2`.
Then:
- `|n_k - x*_k| <= 1/2`,
- `|log n_k - log x*_k| <= 0.5 / (x*_k - 0.5)`.

Hence phase errors satisfy:
- `|delta1_k| <= tau1 * 0.5/(x*_k - 0.5)`,
- `|delta2_k| <= tau2 * 0.5/(x*_k - 0.5)`.

## Lemma R2 (Rounding-preservation thresholds)
If for an anchor:
- `cos(phi2(x*_k)) >= c0`,
- `|delta2_k| <= c0`,
then by `|cos(u+v)-cos u| <= |v|`:
- `cos(phi2(n_k)) >= 0`.

If additionally:
- `|delta1_k| <= arccos(a1)`,
then:
- `|cos(phi1(n_k))| >= a1`.

So for each `c0 > 0`, it is enough to impose:
- `x*_k >= X_round(c0)`,
where
- `X_round(c0) = 0.5 + max(tau1/(2*arccos(a1)), tau2/(2*c0))`.

### Numeric instantiation (not proof)
From `/Users/adminamn/Documents/New project/research/output/k1_w66_tau12_rounding_eventual_threshold_2026-02-24.json`:
- `X_round(0.1)=105.610198`,
- `X_round(0.2)=53.055099`,
- `X_round(0.3)=35.777750`,
- `X_round(0.4)=35.777750`,
- `X_round(0.5)=35.777750`,
and zero failures across all five windows for `c0 in [0.1,0.5]`.

## Lemma S1 (One-sided cofinal q-threshold)
If one-sided tail bound has form:
- `R2^-(x)/A1 <= C * x^(-eta)` with `eta > 0`,
then for any `q_target in (0,a1)`:
- choosing `X_q = max(x1, (C/q_target)^(1/eta))` gives
  `R2^-(x)/A1 <= q_target < a1` for all `x >= X_q`.

This is exactly the required cofinal `q < a1` closure step.

### Numeric instantiation from explicit constants
Using `/Users/adminamn/Documents/New project/research/output/k1_w66_l2a_explicit_ledger_beta062_theta019_2026-02-24.json`
with `beta=0.62`, `eta=0.01`, `theta=0.19`:
- `C_total_A = 1.38771794406`,
- `C_total_B = 1.00085562213`.

From `/Users/adminamn/Documents/New project/research/output/k1_w66_q_cofinal_thresholds_2026-02-24.json` (with `a1=0.98`):
- for `q_target = 0.882 (0.9*a1)`:
  - model A: threshold `1.0e21`,
  - model B: threshold `1.0e21`.
- for `q_target = 0.49 (0.5*a1)`:
  - model A: `1.623725e45`,
  - model B: `1.041203e31`.

Therefore cofinal `q<a1` follows once the corresponding `C*x^{-eta}` theorem bound is established.

## Proposition P (Math closure blueprint)
Assume:
1. Buffered C2 existence at some `c0 in (0,1)` (Lemma C2.1 route).
2. Eventual rounding bounds from Lemmas R1-R2.
3. One-sided tail majorant `R2^-/A1 <= C*x^{-eta}` with `eta>0` (Lemma S1).

Then there exists `q<a1` and a cofinal sequence `x -> inf` such that:
- `cos(phi1(x)) >= a1`,
- `cos(phi2(x)) >= 0`,
- `R2^-(x)/A1 <= q`,
hence:
- `Y_beta(x) >= A1*(a1-q) > 0` on that cofinal sequence.

This is the exact math target needed by the current contradiction endpoint.

## Remaining strictly mathematical gaps
1. Prove/instantiate the buffered C2 existence theorem branch unconditionally
   (or explicitly fix the irrational-rotation premise in the theorem contract).
2. Replace model-level one-sided constants with theorem-grade `C, eta` in the same shape.

Everything else is now reduced to these two analytic obligations.

