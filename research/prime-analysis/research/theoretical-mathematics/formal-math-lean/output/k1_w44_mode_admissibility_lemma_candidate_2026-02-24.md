# K1 W44 Mode-Admissibility Lemma Candidate (2026-02-24)

## Objective
Turn tau-scan results into a clean theorem-facing hypothesis class for the subsequence contradiction route.

## Inputs
- Tau scan at `beta=0.62`, window `x<=3e7`:
  - `/Users/adminamn/Documents/New project/research/output/k1_w44_fixed_error_psi_subsequence_tau_scan_2026-02-24_x3e7.json`
- Tau scan at `beta=0.62`, window `x<=1e7`:
  - `/Users/adminamn/Documents/New project/research/output/k1_w44_fixed_error_psi_subsequence_tau_scan_2026-02-24_x1e7_beta062.json`
- Robustness summary:
  - `/Users/adminamn/Documents/New project/research/output/k1_w44_tau_scan_robustness_summary_2026-02-24.json`

## Empirical robust set (finite-window)
With alignment gate `a0=0.98`, `delta_tri>0` in both windows for 7 tau values:
\[
\{14.1347,\ 21.0220,\ 25.0109,\ 30.4249,\ 32.9351,\ 40.9187,\ 43.3271\}.
\]

For these 7 modes:
- worst observed cofinal remainder ratio (across both windows):
  \[
  q_{\max}^{\text{robust}} \approx 0.703592 < a_0=0.98,
  \]
  hence positive margin
  \[
  \delta_{\min}^{\text{robust}} \approx a_0-q_{\max}^{\text{robust}} \approx 0.276408.
  \]
- worst observed triangle-cofinal margin:
  \[
  \min \delta_{\mathrm{tri}} \approx 0.284575.
  \]

## Candidate lemma shape (mode-admissibility form)
Fix `E*(x)=psi(x)-x`, `beta>1/2`, and `a0 in (0,1)`.

Assume there exists a tau-mode with amplitude `A>0` and phase `phi` such that:
1. infinitely many aligned points satisfy `|cos(tau log x + phi)| >= a0`;
2. cofinally on aligned thresholds, remainder ratio obeys
   \[
   \frac{|R_\beta(x)|}{A} \le q < a_0;
   \]
3. therefore `delta := a0 - q > 0`.

Then:
\[
\forall X\ \exists x\ge X:\ |E_\*(x)| \ge (\delta A)\,x^\beta.
\]

This is the lower-envelope contradiction input needed against endpoint bounds.

## Why this is better than the previous target
- It avoids requiring global uniform sup remainder domination over all large `x`.
- It matches observed finite-window behavior (positive cofinal margins on an admissible tau family).
- It isolates the true theorem obligation: prove mode-admissibility analytically from explicit formula assumptions.

## Remaining hard theorem step
Replace finite-window robust evidence with an asymptotic theorem proving:
\[
\exists \tau,\ A,\phi,\ q<a_0\ \text{such that aligned cofinal remainder ratio bound holds}.
\]

Once this is proved, the contradiction route closes at the lower-envelope gate.

