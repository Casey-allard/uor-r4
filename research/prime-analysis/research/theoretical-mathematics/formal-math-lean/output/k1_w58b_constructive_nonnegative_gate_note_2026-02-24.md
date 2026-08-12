# K1 W58b Constructive Nonnegative-Gate Note (2026-02-24)

## Objective
Advance the remaining T2 math blocker by replacing unstable sample-coincidence gating with an explicit constructive subsequence rule.

## Constructive rule
For each tau14 phase anchor \(k\), choose one integer near the anchor such that:
1. \(|\cos(\tau_{14}\log x + \phi_{14})| \ge a_1\), with \(a_1=0.98\),
2. \(\cos(\tau_{21}\log x + \phi_{21}) \ge 0\),
3. selection uses **phase-only criterion** (minimize \(|\cos(\tau_{21}\log x+\phi_{21})|\)); no residual-based optimization.

Then for two-mode decomposition
\[
Y_\beta(x)=A_1\cos_1(x)+A_2\cos_2(x)+R_2(x),
\]
on selected points:
\[
Y_\beta(x)\ge A_1 a_1 - |R_2(x)|
\]
because \(A_2\cos_2(x)\ge 0\).

So the effective cofinal margin is governed by
\[
q_{\text{cofinal}}=\text{cofinal ratio of }|R_2|/A_1,\qquad
\delta_{\text{eff}}=a_1-q_{\text{cofinal}}.
\]

## Artifacts
- Probe script:
  - `research/fixed_error_psi_tau14_tau21_constructive_gate_probe.py`
- Multi-window summary:
  - `research/k1_tau14_tau21_constructive_gate_multiwindow_summary.py`
- Nonnegative gate, five-window summary:
  - `research/output/k1_w58b_tau14_tau21_constructive_gate_nonneg_five_window_summary_2026-02-24.md`
- Suppress gate comparison (fails robustly):
  - `research/output/k1_w57b_tau14_tau21_constructive_gate_suppress_five_window_summary_2026-02-24.md`

## Key finite-window findings
Windows: \(x\in\{10^7,2\cdot10^7,3\cdot10^7,4\cdot10^7,5\cdot10^7\}\)

- **Suppress mode** (`q = alpha*b2 + rr`): no robust search window.
- **Nonnegative mode** (`q = rr`): robust positive margins for all tested search windows
  \(W\in\{100,200,500,1000,2000,5000,10000\}\).

Best low-variance window in current sweep:
- \(W=100\): \(\delta_{\min}\approx 0.9053\), \(q_{\max}\approx 0.0747\), \(rr_{\max}\approx 0.0747\).

## Interpretation
This gives a concrete math target that is narrower and more structurally defensible than prior ad hoc alignment scans:
- prove cofinal existence of the nonnegative-phase constructive subsequence,
- then prove a theorem-grade bound for \( |R_2|/A_1 \) along that subsequence.

Current evidence remains finite-window numeric, not theorem closure.
