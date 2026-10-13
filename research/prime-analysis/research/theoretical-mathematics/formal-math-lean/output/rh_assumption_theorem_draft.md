# RH Assumption -> Theorem Draft (Bridge Program)

Generated: February 17, 2026

## Goal
State a minimal hypothesis set that, if proved, implies an RH-style endpoint bound
\[
|E(x)| \le C \sqrt{x}(\log x)^A,\quad E(x)=\psi(x)-x.
\]

This draft converts the current empirical pipeline into a theorem blueprint.

## Fixed Objects
- Wheel family: \(\mathcal W=\{30,210,2310,30030\}\).
- Truncated bridge process: \(H_W^{(M)}(x)\).
- Reference cutoff: \(M_{\mathrm{ref}}\) (currently 512 in calibrations).
- Frozen affine reference constants: \((a_{\mathrm{ref}},b_{\mathrm{ref}})\).
- Truncation gap: \(\Delta_M(x;W)=|H_W^{(M)}(x)-H_W^{(M_{\mathrm{ref}})}(x)|\).
- Endpoint residual:
\[
R_M(x;W)=\left|E(x)/\sqrt{x}-(a_{\mathrm{ref}}H_W^{(M)}(x)+b_{\mathrm{ref}})\right|.
\]

## Assumptions (to be proved analytically)
### A1. Reference residual control
There exist constants \(C_0\ge 0\), \(x_0\ge 3\) such that for all \(x\ge x_0\) and all \(W\in\mathcal W\),
\[
\left|E(x)/\sqrt{x}-(a_{\mathrm{ref}}H_W^{(M_{\mathrm{ref}})}(x)+b_{\mathrm{ref}})\right|\le C_0.
\]

Current finite-window candidate from `/Users/adminamn/Documents/New project/research/output/a1_reference_residual_probe_none_ref512.json`:
- \(a_{\mathrm{ref}}\approx -0.001347471211160521\),
- \(b_{\mathrm{ref}}\approx -0.05436128386957597\),
- \(C_0\approx 0.6334997534895853\) (max over tested windows up to \(x=5\times10^6\)).

Observed slow envelope indicator:
\[
C_0(x)\approx C(\log x)^\alpha,\quad \alpha\approx 0.4
\]
on tested scales (evidence only, not proof).

Smoothing-decomposition uplift (`/Users/adminamn/Documents/New project/research/output/a1_smoothing_uplift_note.md`):
- decompose reference residual into smoothing and link terms:
\[
|E/\sqrt{x}-(a_{ref}H_{ref}+b_{ref})|
\le R_{smooth}+R_{link},
\]
and calibrate \(C_0\) from split suprema.
- current uplifted constant:
\[
C_0^{uplift}=0.910288368768
\]
with zero held-out violations on tested splits.

### A2. Truncation decay
There exist constants \(C_\Delta\ge 0\), \(\beta\ge 0\), \(M_0\), \(x_0\) such that for \(M\ge M_0\), \(x\ge x_0\), all \(W\in\mathcal W\),
\[
\Delta_M(x;W)\le C_\Delta\,(\log x)^\beta\,\tau(M),
\]
with \(\tau(M)\to 0\) as \(M\to\infty\).

Current finite-window candidate (kernelized track, gaussian scale 100):
- \(\tau(M)\) chosen as tail\(_{Fp}\)-type sum
\[
\tau(M)=\sum_{j>M}\frac{\gamma_j K(\gamma_j)}{|1/2+i\gamma_j|},
\]
- \(\beta\approx 2.6\),
- \(C_\Delta\approx 5.598810038306168\times 10^{-4}\),
from `/Users/adminamn/Documents/New project/research/output/a2_truncation_decay_probe_gauss100_ref512.json`.

Asymptotic-tail candidate model from `/Users/adminamn/Documents/New project/research/output/a2_tau_asymptotic_note.md`:
\[
\tau(M)\approx \exp(c_0+c_1\gamma_M^2),
\]
with gaussian-track fit
\[
c_0\approx 2.4071685075911513,\quad c_1\approx -1.0187257998037246\times 10^{-4}.
\]

Fixed-beta non-regression constant pack (`/Users/adminamn/Documents/New project/research/output/a2_theorem_constant_pack_note.md`):
- form frozen as
\[
\Delta_M(x;W)\le C_\Delta(\log x)^\beta\tau(M),
\]
with \(\beta\) fixed in advance (\(2.6\)),
\[
\tau(M)=\sum_{j>M}^{M_{ref}}\frac{\gamma_j K(\gamma_j)}{|1/2+i\gamma_j|},
\]
and \(C_\Delta\) extracted as train-sup-ratio times safety factor.
- current theorem-scaffold value:
\[
C_\Delta=1.3231345976502436\times 10^{-3}
\]
(gaussian kernel scale 100, \(M\in\{64,128,192,256\}\), \(M_{ref}=512\)),
with zero held-out violations on tested splits.

Infinite-tail uplift candidate (`/Users/adminamn/Documents/New project/research/output/a2_infinite_tail_uplift_note.md`):
- define
\[
\tau_\infty^{maj}(M)=\tau_{fin}(M)+\tau_{extra}(M),
\]
where \(\tau_{extra}\) is an explicit density-envelope integral beyond \(\gamma_{M_{ref}}\).
- current gaussian-track uplifted constant:
\[
C_\Delta^{(\infty\text{-uplift})}=1.3231345976502436\times10^{-3},
\]
with train and held-out finite-window checks both holding.

### A3. Bridge growth control
There exist constants \(C_H\ge 0\), \(A_H\ge 0\), \(x_0\) such that for all \(x\ge x_0\), all \(W\in\mathcal W\),
\[
|H_W^{(M)}(x)|\le C_H(\log x)^{A_H}.
\]

Current finite-window candidate from `/Users/adminamn/Documents/New project/research/output/a3_bridge_growth_probe_none_z128_v2.json`:
- \(A_H\approx 7.2\),
- \(C_H\approx 1.9424205508648547\times 10^{-7}\),
for `M=128`, tested scales up to \(x=5\times 10^6\), with zero anchor-scale envelope violations.

Channel-energy uplift (`/Users/adminamn/Documents/New project/research/output/a3_channel_energy_uplift_note.md`):
- exact bridge:
\[
|H(x)|\le B_k,\quad B_k=\frac{|S_k|}{\sqrt{k}},
\]
with \(k=k(x;W)\).
- split-validated uplifted envelope:
\[
B_k \le C_B(\log x)^{A_H},
\]
hence
\[
|H(x)|\le C_B(\log x)^{A_H}.
\]
Current uplift constants:
- \(C_H=C_B=4.0835159936256787\times10^{-4}\),
- \(A_H=4.4\),
with train and held-out checks holding on tested grids.

### A4. Base-uniform transfer constants
The constants in A1-A3 are uniform over \(W\in\mathcal W\).

## Deterministic Lemma (already structurally established)
For any \(M<M_{\mathrm{ref}}\), any \(x,W\):
\[
R_M(x;W)\le
\left|E/\sqrt{x}-(a_{\mathrm{ref}}H_W^{(M_{\mathrm{ref}})}+b_{\mathrm{ref}})\right|
+
|a_{\mathrm{ref}}|\,\Delta_M(x;W).
\]
Therefore under A1-A2:
\[
R_M(x;W)\le C_0+|a_{\mathrm{ref}}|\,C_\Delta(\log x)^\beta\tau(M).
\]

## Theorem Candidate
If A1-A4 hold, then for fixed \(M\ge M_0\) and \(x\ge x_0\):
\[
|E(x)|/\sqrt{x}
\le
|a_{\mathrm{ref}}|\,|H_W^{(M)}(x)|+|b_{\mathrm{ref}}|+R_M(x;W)
\]
\[
\le
|a_{\mathrm{ref}}|C_H(\log x)^{A_H}
+|b_{\mathrm{ref}}|
+C_0
+|a_{\mathrm{ref}}|C_\Delta(\log x)^\beta\tau(M).
\]
Hence
\[
|E(x)|\le C\sqrt{x}(\log x)^A
\]
for
\[
A=\max(A_H,\beta),\quad
C=|a_{\mathrm{ref}}|C_H+|a_{\mathrm{ref}}|C_\Delta\tau(M)+|b_{\mathrm{ref}}|+C_0.
\]

This is an RH-compatible endpoint shape.

## RH-equivalent direction
To move from RH-compatible bound to RH-equivalent conclusion, one must prove the endpoint with the standard sharp class required by a known equivalence theorem (or directly show the equivalent criterion in the chosen framework).

## Immediate proof tasks
1. Upgrade A2 from finite-\(M_{ref}\) to true infinite-tail bound \(\tau_\infty(M)\), with explicit analytic constants.
2. Replace sampled-sup \(C_\Delta\) by analytic channel perturbation majorant.
3. Prove A1 uniformly from explicit-formula smoothing residual analysis (Lemma C uplift).
4. Prove A3 (polylog growth of \(H_W^{(M)}\)) uniformly in \(W\).
5. Bind all constants independent of sampled windows and confirm threshold \(x_0\).

## Current Unified Finite-Window Pack (A1-A4 together)
Latest uplift-consolidated pack:
- `/Users/adminamn/Documents/New project/research/output/uplift_theorem_pack_refresh_2026-02-17.json`
- `/Users/adminamn/Documents/New project/research/output/uplift_theorem_pack_refresh_2026-02-17.md`

Using uplift constants from A1/A2/A3 and A4 grid rows:
- \(C_0^{\mathrm{uplift}}=0.9102883687683553\),
- \(C_\Delta^{\mathrm{uplift}}=0.0013231345976502436\), \(\beta=2.6\),
- \(C_H^{\mathrm{uplift}}=0.00040835159936256787\), \(A_H=4.4\).

Unified checks on tested windows (up to \(x=5\times10^6\), all four bases):
- all component valid checks hold (A1, A2, A3, A4),
- unified combined inequality holds on all rows,
- max tested ratio \(\max(\mathrm{LHS}/\mathrm{RHS})=0.02045996411947228\),
- min tested margin \(\min(\mathrm{RHS}-\mathrm{LHS})=29.724092445561407\).

Analytic-style A3 transfer variant:
- artifact: `/Users/adminamn/Documents/New project/research/output/a3_density_transfer_majorant_refresh_2026-02-17.json`,
- transfer identity: \(|H| = B\sqrt{k/x}\), with \(B\)-envelope and analytic density guardrail
  \(k/x \le \phi(W)/W+\phi(W)/x_0\),
- unified pack check: `/Users/adminamn/Documents/New project/research/output/uplift_theorem_pack_refresh_2026-02-17_a3density.json` (holds on tested windows).

Quadratic-energy A3 guardrail branch:
- artifact: `/Users/adminamn/Documents/New project/research/output/a3_quadratic_energy_majorant_refresh_2026-02-17.json`,
- deterministic inequality core: \(B_k\le \sqrt{\sum_{i\le k} g_i^2}\), and
  \(|H|\le \sqrt{(\sum_{i\le k} g_i^2)\,k/x}\),
- unified pack check: `/Users/adminamn/Documents/New project/research/output/uplift_theorem_pack_refresh_2026-02-17_a3quad.json` (holds, but currently much looser).

Offdiag-energy A3 diagnostic branch:
- artifact note: `/Users/adminamn/Documents/New project/research/output/a3_offdiag_energy_majorant_note.md`,
- finding: split-trained offdiag guard fails held-out unless made very conservative,
- status: diagnostic/stress-test branch, not selected for primary A3 constants.

Offdiag-dynamic A3 branch:
- artifact note: `/Users/adminamn/Documents/New project/research/output/a3_offdiag_dynamic_majorant_note.md`,
- model: \(\eta_+(x)\) is log-envelope controlled and fed into
  \(|H|\le \sqrt{(1+\eta_+(x))E_2/x}\),
- current calibrated result: held-out valid with \(A_H=2.5\), \(C_H=0.07086646136830307\),
- status: promising lower-exponent branch; constant tightening remains open,
- note: piecewise-constant \(\eta(x)\) test failed held-out, so global-log \(\eta\) remains selected.
- note: tighter-constant run (`h_safety=1.0`) produced valid \((A_H,C_H)=(2.8,0.0313)\), but branch leaderboard still prefers \((2.5,0.0709)\) at reference asymptotic scale.
- update: constrained fixed-\(A_\eta\) run
  `/Users/adminamn/Documents/New project/research/output/a3_offdiag_dynamic_eta4_fixed_note.md`
  now provides held-out-valid \((A_H,C_H)=(1.2,2.607225675383175)\),
  currently the top-ranked A3 branch at \(x=10^{12}\).
- support artifact for exponent choice:
  `/Users/adminamn/Documents/New project/research/output/a3_eta_exponent_probe_2026-02-17.md`
  (fixed-\(A_\eta\) safety scan over train/held-out splits).
- support artifact for fixed-\(A_\eta=4\) stress-fit:
  `/Users/adminamn/Documents/New project/research/output/a3_eta4_justification_probe_2026-02-17_sf3.md`.
- stress validation artifact:
  `/Users/adminamn/Documents/New project/research/output/a3_offdiag_dynamic_stress_note.md`
  (denser sampling and valid split up to \(10^7\), with full A3+A4 stress pack still holding).

Important caveat for proof work:
- observed normalized sequence \(\eta_+(x)/(\log x)^4\) is not monotone on sampled windows;
  therefore current \(A_\eta=4\) support is still calibration evidence, not yet an analytic monotone-majorant proof.

Active A3 analytic handoff memo:
- `/Users/adminamn/Documents/New project/research/output/a3_eta_lemma_scaffold_2026-02-17.md`
defines the current candidate lemma statement, calibrated constant budget, and concrete proof obligations.

Symbolic-chain checkpoint:
- `/Users/adminamn/Documents/New project/research/output/a3_offdiag_symbolic_chain_note.md`
confirms pure absolute bilinear offdiag majorant is valid but too loose for current \(C_\eta\) budget,
so the analytic route must preserve cancellation/sign structure.

Lag-structured next-step scaffold:
- `/Users/adminamn/Documents/New project/research/output/a3_offdiag_lag_decomposition_probe_2026-02-17.md`
prioritizes low-lag bands for a sign-sensitive offdiag lemma refinement.

Composite \(E_2/x\) A3 branch:
- artifact note: `/Users/adminamn/Documents/New project/research/output/a3_e2x_dynamic_majorant_note.md`,
- model combines \(\eta_+(x)\) and \(E_2(x)/x\) envelopes into
  \(|H|\le \sqrt{(1+\eta_+(x))E_2/x}\),
- current result: held-out valid with \((A_H,C_H)=(3.3,0.0113)\),
- status: useful constant-reduction branch, but not better than current fixed-\(A_\eta\) winner \((A_H,C_H)=(1.7,0.7044)\) on asymptotic leaderboard.

A3 branch ranking snapshot:
- `/Users/adminamn/Documents/New project/research/output/a3_branch_leaderboard_2026-02-17.md`
currently ranks offdiag-dynamic first by asymptotic exponent and by reference-scale RHS (`x=10^12`).

This remains finite-window calibration evidence; asymptotic proof obligations are unchanged.

Previous baseline finite-window pack snapshot:
From `/Users/adminamn/Documents/New project/research/output/a4_uniform_assumption_check_none_z128_ref512.json`:
- \(a_{\mathrm{ref}}=-0.001347471211160521\)
- \(b_{\mathrm{ref}}=-0.05436128386957597\)
- \(C_0=0.6334997534895853\)
- \(C_\Delta=4.674970279117053\times 10^{-5}\), \(\beta=2.6\), \(\tau_M=383.99980444354367\)
- \(C_H=8.45323261007565\times 10^{-7}\), \(A_H=7.2\)

On tested windows (up to \(x=5\times10^6\), all four bases), the combined theorem RHS check has zero violations.
This is calibration evidence only; asymptotic proof remains open.

Extended consistency check (`/Users/adminamn/Documents/New project/research/output/a4_uniform_assumption_check_none_z128_ref512_n10m.json`) up to \(x=10^7\):
- still zero violations on sampled grid,
- strict negative margin in max gap.

Working finite-window threshold candidate:
- \(x_0^{(\mathrm{cand})}=10^6\) (empirical, not proved).

Kernel robustness support:
- `/Users/adminamn/Documents/New project/research/output/a4_uniform_assumption_check_gauss100_z128_ref512.json`
also shows zero-violation unified check (with different constants), supporting structural robustness of the theorem form.

## Conservative Constant Replacement Pack (Train vs Held-Out)
From `/Users/adminamn/Documents/New project/research/output/analytic_constant_replacement_pack_none_z128_ref512_fast.json`:
- split: train `n in {3e5, 1e6}`, validation `n in {2e6, 5e6}`, all `W in {30,210,2310,30030}`.
- fitted-on-train:
  - \(a_{\mathrm{ref}}=-0.004573715456439612\),
  - \(b_{\mathrm{ref}}=-0.08301746463150639\),
  - \(C_0=0.6066357305845828\),
  - \(C_\Delta=2.8757332534599567\times10^{-5}\),
  - \(C_H=6.132322375874561\times10^{-7}\).
- conservative (safety factor 1.2 on \(C_0,C_\Delta,C_H\)):
  - \(C_0^{\mathrm{cons}}=0.7279628767014993\),
  - \(C_\Delta^{\mathrm{cons}}=3.4508799041519476\times10^{-5}\),
  - \(C_H^{\mathrm{cons}}=7.358786851049473\times10^{-7}\),
  - \(\tau_M=383.99980444354367\) (`M=128`, `M_ref=512`).

Both train and held-out checks have:
- zero theorem-RHS violations,
- strict negative max gap (\(\max(\text{LHS}-\text{RHS})=-0.4023940044...\)).

This is still finite-window evidence, but it is a stricter non-overfit calibration path than single-grid fitting.

Extended held-out check:
- `/Users/adminamn/Documents/New project/research/output/analytic_constant_replacement_pack_none_z128_ref512_valid10m_v3.json`
with validation up to \(x=10^7\) (coarser sample step \(2\times10^4\)) also has zero violations and strict margin.
