# A3 Channel-Energy Uplift Note

Generated: February 17, 2026  
Artifact: `/Users/adminamn/Documents/New project/research/output/a3_channel_energy_uplift_refresh_2026-02-17.json`

## Goal
Refine A3 with a deterministic bridge and split-validated envelope.

## Deterministic bridge
For wheel-event partial sums \(S_k=\sum_{i\le k} g_i\), define
\[
B_k=\frac{|S_k|}{\sqrt{k}},\qquad H(x)=\frac{S_k}{\sqrt{x}},\quad k=k(x;W).
\]
Then
\[
|H(x)|=B_k\sqrt{\frac{k}{x}}\le B_k.
\]
So any bound on \(B_k\) immediately yields a bound on \(|H(x)|\).

## Envelope construction
Fit a held-out-valid minimal RHS envelope on split data (train `n={3e5,1e6}`, valid `n={2e6,5e6}`):
\[
|H(x)| \le C_H (\log x)^{A_H},
\]
with constrained search over \(A_H\in[0,10]\) and safety uplift.

Current constants:
- \(C_H^{uplift}=4.0835159936256787\times10^{-4}\),
- \(A_H=4.4\),
- valid split max ratio \(\max |H|/\mathrm{RHS}=0.9517193744055934\).

Checks:
- deterministic bridge check: holds on train and held-out,
- log-envelope check for \(|H(x)|\): holds on train and held-out.

## Interpretation
- This keeps the exact deterministic bridge \( |H|\le B_k \) as a structural guardrail and uses split-validated fitting directly on \(|H|\) to reduce envelope slack.
- Analytic-style variant now available via density-transfer majorant:
  `/Users/adminamn/Documents/New project/research/output/a3_density_transfer_majorant_refresh_2026-02-17.json`,
  using \( |H|=B\sqrt{k/x} \) and analytic base-uniform guardrail
  \(k/x \le \phi(W)/W + \phi(W)/x_0\).
- Remaining analytic gap:
  1. derive the \(|H|\) (or \(B_k\)) envelope from analytic/ergodic cancellation arguments rather than sampled supremum,
  2. make constants independent of sampled windows.
