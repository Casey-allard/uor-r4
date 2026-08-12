# K1 W30 Missing Math Dossier (2026-02-24)

## 1) Exact unresolved math target
The unresolved payload remains:
- `/Users/adminamn/Documents/New project/research/formal/lean/PrimeRiemannBridgeSpinningTopFrontier.lean:881`
- `R6DualBandTailRepresentationCandidateShapeTerm`

Core hard clause (for all admissible `E`, all right-half zeros `s`):
\[
\frac{R(x)}{x^{\beta}} = \sum_{j=1}^{256} \kappa_j x^{-\eta_j}\sin(\omega_j \log x + \theta_j)
\quad \forall x \ge 10^{21}
\]
with fixed `w.eta = 0.01`, fixed `w.x1 = 10^21`, and exactly `256` modes.

## 2) What is mathematically missing
Not pipeline glue. Not provider wiring. The missing mathematics is:

1. A theorem that converts explicit-formula tails into an **exact finite 256-mode identity** on a half-line.
2. Equivalently: a theorem that the omitted spectrum beyond those 256 modes contributes **identically zero** on `x >= 10^21`.
3. Uniform quantification over all `E` and all right-half zeros in that exact shape.

Current repo math already supports a weaker (and realistic) shape:
- finite modes + decaying residual majorant
- see `/Users/adminamn/Documents/New project/research/formal/lean/PrimeRiemannBridgeOscillatoryReduction.lean:1256`

## 3) Why this is hard (and likely the true frontier)
Known modern results provide explicit error/zero-density control, not exact finite collapse:
- Bellotti 2024: explicit log-free density `N(σ,T) <= A T^{B(1-σ)}` [arXiv:2405.12545](https://arxiv.org/abs/2405.12545)
- Johnston 2024/2025: zero-density -> near-optimal PNT error transfer [arXiv:2411.13791](https://arxiv.org/abs/2411.13791)
- Bellotti 2025: new density near 1-line and optimal PNT error form [arXiv:2508.02041](https://arxiv.org/abs/2508.02041)
- Fiori–Kadiri–Swidinsky 2022/2023: sharper explicit bounds for `psi(x)-x` [arXiv:2204.02588](https://arxiv.org/abs/2204.02588)
- Schlage-Puchta 2019: right-half zero forces oscillations [arXiv:1912.00853](https://arxiv.org/abs/1912.00853)

These support tail majorants, not exact finite identity on all large `x`.

Exact theorem-line extraction artifact:
- `/Users/adminamn/Documents/New project/research/output/k1_w31_external_theorem_constants_2026-02-24.md`

## 4) New empirical math evidence from this run

### 4.1 Truncation-residual probe (directly on explicit-formula surrogate)
Script:
- `/Users/adminamn/Documents/New project/research/r6_truncation_residual_probe.py`

Main run:
- `/Users/adminamn/Documents/New project/research/output/r6_truncation_residual_probe_2026-02-24_x1e22_m20000_g8192.md`

For `M=20000` zeros total and finite head `N` modes retained, omitted-tail residual at `x>=1e21`:
- `N=256`: tail residual sup ratio `~0.337`
- `N=512`: `~0.226`
- `N=1024`: `~0.195`
- `N=2048`: `~0.173`

This is consistent with decaying residual, not exact vanishing residual.

Additional runs:
- `M=50000`: `/Users/adminamn/Documents/New project/research/output/.tmp_r6_truncation_residual_probe_x1e22_m50000_g4096.md`
- `M=100000`: `/Users/adminamn/Documents/New project/research/output/.tmp_r6_truncation_residual_probe_x1e22_m100000_g4096.md`

For `M=100000`:
- `N=256`: tail residual sup ratio `~0.332`
- `N=8192`: tail residual sup ratio `~0.070`

Residual shrinks with `N`, but does not exhibit finite-`N` exact collapse.

### 4.2 Data-quality correction from earlier runs
Earlier `k=256` fit with `grid=4096` at `x1=1e21` had underdetermined tail regression (`273` tail samples vs `513` coefficients). Dense-grid and out-of-sample checks degrade the apparent near-fit.
(Details in `/Users/adminamn/Documents/New project/research/output/k1_w29_local_research_audit_2026-02-24.md`.)

## 5) Concrete math sublemmas to derive next (no remapping)

### L1: Explicit truncation identity with remainder
Derive a theorem of form:
\[
\frac{R(x)}{x^\beta} = S_N(x) + \mathcal{E}_N(x),\quad x\ge x_1
\]
where `S_N` is finite mode sum (target `N=256`) and `E_N` is omitted-spectrum remainder.

### L2: Omitted-spectrum upper bound from zero-density + zero-free region
Prove explicit:
\[
|\mathcal{E}_N(x)| \le C_N x^{-\eta_N}, \quad x\ge x_1
\]
using modern explicit zero-density inputs (Bellotti/Johnston/FKS chain).

### L3: Parameter lock problem
Show whether one can force:
- `eta_N >= 0.01`
- `x1 = 10^21`
- explicit `C_N` compatible with existing R6 constants
for a fixed finite `N=256`.

### L4: Decide exact-vs-majorant target
If L2 succeeds but residual is nonzero, exact clause at candidate-shape level remains unproved; mathematically, the proved object is finite-mode-plus-majorant, not exact finite identity.

## 6) Immediate next research step
Start L2A derivation (still math-first, no Lean plumbing):
- write explicit 3-piece omitted-tail split `E_N = E_{N,<=T} + E_{>T} + R_T` for `N=256`
- substitute TeX-locked constants from Bellotti 2024/2025, Johnston 2024/2025, FKS 2022/2023
- derive a symbolic `C_N, eta_N` candidate at `x>=10^21`
- run a pass/fail check on `eta_N >= 0.01` and record the exact failing component if not met.

W32 update:
- with the source-backed cutoff policy currently used (`T=max(gamma_256, exp(2*omega))`), finite-band term is empty on `x in [1e21,1e30]`; see:
  - `/Users/adminamn/Documents/New project/research/output/k1_w32_l2a_cutoff_diagnostic_2026-02-24.md`

Therefore the current exact failing component is:
- cutoff mechanism, not yet the exponent target itself.

W32 candidate:
- endpoint-valid schedule `T(x)=x^0.13` identified from local scan as the smallest tested `x^p` with non-empty finite band at `x1=1e21`.
- artifacts:
  - `/Users/adminamn/Documents/New project/research/output/k1_w32_cutoff_xpow_scan_2026-02-24.md`
  - `/Users/adminamn/Documents/New project/research/output/k1_w32_l2a_split_ledger_xpow013_2026-02-24.md`

W33 update:
- explicit-only ledger now produced (band term no longer empirical-phase-fitted):
  - `/Users/adminamn/Documents/New project/research/output/k1_w32_l2a_explicit_ledger_2026-02-24.md`
- current candidate constants at `eta=0.01`:
  - `C_band_explicit ~ 0.401`
  - `C_high_model ~ 0.967`
  - remainder uncertainty dominates:
    - model A: `C_rem ~ 7.06` -> total `~8.43`
    - model B: `C_rem ~ 0.629` -> total `~2.00`
- remaining open math is now localized to the exact remainder normalization/theorem chain.

W33 schedule optimization:
- theta scan artifact:
  - `/Users/adminamn/Documents/New project/research/output/k1_w33_l2a_theta_schedule_optimization_2026-02-24.md`
- current best explicit schedule candidate:
  - `T(x)=x^0.19` with `C_total_A ~ 2.52` at `eta=0.01`.

W33 normalization gate:
- `/Users/adminamn/Documents/New project/research/output/k1_w33_l2a_normalization_gate_2026-02-24.md`
- key unresolved mathematical fork:
  - if remainder comes from E-level truncation, normalized remainder carries `x^(1-beta)` and forces `theta >= 1-beta+eta`;
  - if remainder is already in normalized phase-residual form, low-theta schedules remain admissible.
- this gate now controls whether the current `theta=0.19` candidate can be theorem-final.

W34 hard-item closure (A->B translation package):
- `/Users/adminamn/Documents/New project/research/output/k1_w34_branch_translation_lemma_package_2026-02-24.md`
- translation ledger runs:
  - `/Users/adminamn/Documents/New project/research/output/k1_w34_branchA_to_B_translation_beta055_2026-02-24.md`
  - `/Users/adminamn/Documents/New project/research/output/k1_w34_branchA_to_B_translation_beta051_2026-02-24.md`
- key formula now explicit:
  - Branch-A remainder can be translated to Branch-B with
    `C_rem = C_A * M(x1, delta)` and `delta = (theta + beta_lower - 1) - eta_target`.
- mathematically admissible theta condition now fixed:
  - `theta >= 1 - beta_lower + eta_target`.
