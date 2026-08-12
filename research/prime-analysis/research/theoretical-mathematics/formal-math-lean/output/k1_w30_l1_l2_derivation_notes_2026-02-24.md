# K1 W30 L1/L2 Derivation Notes (2026-02-24)

## L1 target (decomposition identity)
Work in the normalized variable
\[
Y(x) := \frac{R(x)}{x^\beta}.
\]
For a truncated explicit-formula surrogate with `M` zeros:
\[
Y_M(x) = -2x^{\frac12-\beta}
\sum_{n=1}^{M}
\frac{\frac12\cos(\gamma_n\log x)+\gamma_n\sin(\gamma_n\log x)}{\frac14+\gamma_n^2}
-\log(2\pi)x^{-\beta}.
\]
Split at finite head size `N`:
\[
Y_M(x)=S_N(x)+\mathcal E_{N,M}(x),
\]
where `S_N` is first `N` modes and `E_{N,M}` is omitted tail (`n>N`).

This L1 identity is immediate algebraically for finite `M`.

## L2 target (majorant inequality)
Desired theorem shape:
\[
|\mathcal E_{N}(x)| \le C_N x^{-\eta_N},\qquad x\ge x_1.
\]
For the current target path, key locked values are:
- `N=256`
- `x1=10^21`
- need at least `\eta_N \ge 0.01`.

## Why exact finite identity is the hard gate
Current unresolved candidate-shape clause is stronger than L2:
\[
Y(x)=S_{256}(x)\quad \forall x\ge 10^{21}
\]
(i.e. `\mathcal E_{256}(x)\equiv 0` on a half-line).

So the missing math is not “tail is small”; it is “tail is identically zero”.
Existing explicit-formula / zero-density literature gives bounds, not exact vanishing.

## Data-backed calibration from this run
Using `/Users/adminamn/Documents/New project/research/r6_truncation_residual_probe.py`:

- `M=100000`, `x in [1e7,1e22]`, `grid=8192`, `x1=1e21`
- `N=256`:
  - tail sup residual: `1.8457e-02`
  - tail sup ratio to total signal tail: `0.3712`
  - fitted majorant: `eta_eff ≈ 0.1984`, `C_tail ≈ 2.97e2`
- `N=8192`:
  - tail sup ratio: `0.09445`
  - fitted majorant: `eta_eff ≈ 0.1173`, `C_tail ≈ 1.72`

Residual decays as `N` increases, but finite-`N` exact collapse is not observed.

## TeX-exact external constants lock (W31)
New source-locked extraction:
- `/Users/adminamn/Documents/New project/research/output/k1_w31_external_theorem_constants_2026-02-24.md`

Core imported theorem inputs now pinned to exact TeX lines:
1. Bellotti 2024 log-free density:
\[
N(\sigma,T)\le C T^{B(1-\sigma)}.
\]
2. Johnston transfer theorem:
\[
\Delta_i(x)\ll
\exp(-\omega(x))
\exp\!\left(2A\omega(x)\left(\frac{\omega(x)}{\log x}\right)^B\right)
\omega(x)^C.
\]
3. Bellotti 2025 near-edge corollary:
\[
N(\sigma,T)\ll e^{55A}
\]
in the KV-edge regime, plus
\[
\Delta(x)\ll \exp(55A_0)\exp(-\omega(x)).
\]
4. FKS global explicit:
\[
E_\psi(x)<9.22022(\log x)^{3/2}\exp(-0.8476836\sqrt{\log x}),\ x>2.
\]
5. Schlage-Puchta oscillation forcing:
right-half zeros imply explicit lower-bound oscillations of `Delta(x)`.

These are valid L2 majorant ingredients; they still do not imply exact finite tail vanishing.

## L2A concrete derivation target (next immediate step)
Split blueprint:
- `/Users/adminamn/Documents/New project/research/output/k1_w31_l2a_split_blueprint_2026-02-24.md`
Constant ledger:
- `/Users/adminamn/Documents/New project/research/output/k1_w31_l2a_constant_ledger_2026-02-24.md`
- `/Users/adminamn/Documents/New project/research/output/k1_w31_l2a_constant_ledger_2026-02-24.json`

For `N=256`, define a cutoff `T_N` from the `N`-th head mode and derive:
\[
\mathcal E_N(x)=\mathcal E_{N,\le T}(x)+\mathcal E_{>T}(x)+\mathcal R_T(x),
\]
then prove explicit bounds for each piece using:
- density on `N(\sigma,T)` (Bellotti 2024/2025),
- zero-free/transfer error forms (Johnston, FKS),
- truncated explicit formula remainder term.

Deliverable for L2A:
- a symbolic closed-form `C_N, eta_N` candidate at `N=256`, `x1=10^21`,
- and a clear pass/fail check for `eta_N >= 0.01`.

## Structural obstacle identified (W31)
Tail-divergence note:
- `/Users/adminamn/Documents/New project/research/output/k1_w31_l2a_tail_divergence_note_2026-02-24.md`

Key finding:
- unsmoothed omitted-mode absolute control scales like `sum_{n>N} 1/gamma_n`, which diverges.

Consequence:
- L2A must use truncated-explicit-formula remainder machinery, smoothing weights, or explicit signed cancellation;
- raw absolute-tail summation cannot close the bound.

## Cutoff obstacle identified (W32)
Cutoff diagnostic:
- `/Users/adminamn/Documents/New project/research/output/k1_w32_l2a_cutoff_diagnostic_2026-02-24.md`

Key blocker:
- with source-backed cutoff `T(x)=max(gamma_256, exp(2*omega(x)))`, we get `k_cut=256` for all sampled `x in [1e21,1e30]`;
- finite omitted band is empty (`E_{256,<=T} == 0`) and the split does not reduce the endpoint problem.

Crossing point:
- `exp(2*omega(x))` reaches `gamma_256` only near `x ~ 1.337e64` in the current model.

Implication:
- to close L2 at `x1=1e21`, we must replace this cutoff strategy or directly bound full omitted tail `E_256`.

Endpoint-valid candidate (W32):
- power-cutoff scan: `/Users/adminamn/Documents/New project/research/output/k1_w32_cutoff_xpow_scan_2026-02-24.md`
- selected candidate: `T(x)=x^0.13`
- nondegenerate split artifact: `/Users/adminamn/Documents/New project/research/output/k1_w32_l2a_split_ledger_xpow013_2026-02-24.md`
- first constant propagation candidate:
  - `/Users/adminamn/Documents/New project/research/output/k1_w32_l2a_constant_propagation_candidate_2026-02-24.md`

## Explicit-only L2A ledger (W33)
Artifact:
- `/Users/adminamn/Documents/New project/research/output/k1_w32_l2a_explicit_ledger_2026-02-24.md`

This step removes empirical phase-fit dependence from the band term:
- band bound built from explicit zero-count envelope `U(T)` and
  \[
  S_1(\gamma_N,b)\le U(b)/b+\int_{\gamma_N}^b U(t)/t^2\,dt.
  \]
- amplitude inequality used: per-mode contribution `<= 2/gamma`.

At `eta=0.01`, `theta=0.13`, `beta=0.55`, `x>=1e21`:
- `C_band_explicit ≈ 0.40113`
- `C_high_selected (Bellotti model) ≈ 0.96693`
- remainder candidate A (`(log x)^2/x^theta`): `C_rem_A ≈ 7.06106`
- remainder candidate B (`x^(1/2-beta)(log x)^2/x^theta`): `C_rem_B ≈ 0.62932`
- combined:
  - `C_total_A ≈ 8.42912`
  - `C_total_B ≈ 1.99737`

Status:
- `eta=0.01` remains feasible numerically with explicit band control.
- remaining theorem task is normalization-accurate justification of the remainder term and exact high-envelope chain.

Theta schedule optimization (W33):
- `/Users/adminamn/Documents/New project/research/output/k1_w33_l2a_theta_schedule_optimization_2026-02-24.md`
- best scanned schedule under explicit model-A ledger:
  - `T(x)=x^0.19`
  - `C_total_A ≈ 2.519704`
- high-resolution run:
  - `/Users/adminamn/Documents/New project/research/output/k1_w33_l2a_explicit_ledger_theta019_2026-02-24.md`

Normalization gate (W33):
- `/Users/adminamn/Documents/New project/research/output/k1_w33_l2a_normalization_gate_2026-02-24.md`
- branch-selection audit:
  - `/Users/adminamn/Documents/New project/research/output/k1_w33_branch_selection_audit_2026-02-24.md`
- unresolved branch choice:
  - Branch A (E-level truncation) induces factor `x^(1-beta)` after normalization;
  - Branch B (already-normalized phase residual) does not.
- this branch decides whether low-theta schedules (like `0.19`) are mathematically admissible for `eta=0.01`.

Branch-translation package (W34):
- `/Users/adminamn/Documents/New project/research/output/k1_w34_branch_translation_lemma_package_2026-02-24.md`
- executable ledger:
  - `/Users/adminamn/Documents/New project/research/k1_l2a_branchA_translation_ledger.py`
- main Branch-A consequence now explicit:
  \[
  \theta \ge 1-\beta_{lower}+\eta_{target}.
  \]
- with `beta_lower=0.55`, `eta_target=0.01`:
  - `theta_min=0.46`, best scanned at `C_A=1` is `theta=0.62`.

## Research-grade derivation route (math only)
1. Prove L1 at theorem level (explicit finite-head + omitted-tail decomposition).
2. Derive L2 from explicit zero-density + zero-free input:
   - Bellotti 2024/2025, Johnston 2024, FKS 2022 provide candidate inputs.
3. Extract explicit `C_N, eta_N` for `N=256`, `x1=10^21`.
4. Compare theorem constants against empirical envelopes as a sanity check.

## Practical implication
If L2 is proved but `\mathcal E_{256}` is nonzero, then the exact candidate-shape clause is still open.
At that point the mathematically proved object is finite-mode-plus-majorant, not exact finite identity.
