# A3 Offdiag Symbolic Chain Note

Generated: February 17, 2026  
Artifact: `/Users/adminamn/Documents/New project/research/output/a3_offdiag_symbolic_chain_2026-02-17.json`

Deterministic chain tested:
\[
\eta_+(x)\le \eta_{\text{sym}}(x):=
\frac{\left(\sum_{i\le k}|g_i|\right)^2-\sum_{i\le k}g_i^2}{\sum_{i\le k}g_i^2}.
\]

Result:
- Chain is valid on sampled grid (zero violations).
- But normalized constants are too loose:
  - \(C_{\text{sym,train}} \approx 9.821\),
  - \(C_{\text{sym,valid}} \approx 31.991\),
  - with safety `3.0`, \(C_{\text{sym,safe}} \approx 29.464\),
  - still under-covers held-out (`valid_ratio_over_safe ≈ 1.086`).
- Relative to current selected \(C_\eta\) budget (`1.217...`), symbolic absolute chain is about `24.21x` larger.

Interpretation:
- Absolute bilinear majorization destroys cancellation structure and is not competitive for the current A3 theorem path.
- Next theorem target must be a sign-sensitive/offdiag-structured bound, not pure absolute-value expansion.
