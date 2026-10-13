# A3 Quadratic-Energy Majorant Note

Generated: February 17, 2026  
Artifact: `/Users/adminamn/Documents/New project/research/output/a3_quadratic_energy_majorant_refresh_2026-02-17.json`

## Deterministic structure
For event sums \(S_k=\sum_{i\le k} g_i\):
\[
B_k=\frac{|S_k|}{\sqrt{k}},\qquad
|S_k|^2 \le k\sum_{i\le k} g_i^2
\Rightarrow
B_k \le \sqrt{\sum_{i\le k} g_i^2}.
\]
Also
\[
|H(x)|=\frac{|S_k|}{\sqrt{x}}=B_k\sqrt{\frac{k}{x}}
\le
\sqrt{\sum_{i\le k} g_i^2}\sqrt{\frac{k}{x}}.
\]

Using analytic density guardrail
\[
\frac{k}{x}\le \frac{\phi(W)}{W}+\frac{\phi(W)}{x_0},
\]
this yields an A3 transfer envelope candidate.

## Current calibrated constants (finite-window)
Current run uses direct deterministic target
\[
|H(x)| \le \sqrt{\Big(\sum_{i\le k} g_i^2\Big)\frac{k}{x}}
\le C_H(\log x)^{A_H}.
\]

- \(A_H=5.2\), \(C_H=2.089011392050098\times10^{-4}\),
- deterministic checks hold on train and held-out for:
  - \(B_k\le\sqrt{\sum g_i^2}\),
  - \(|H(x)|\le\sqrt{(\sum g_i^2)k/x}\),
  - \(|H(x)|\le C_H(\log x)^{A_H}\).

## Interpretation
- This branch is mathematically cleaner in inequality structure than direct max-fit routes.
- It is substantially looser on the tested grid than the current density-transfer A3 branch because the fitted exponent increases.
- Use it as a rigorous fallback/guardrail while tightening the energy envelope step.
