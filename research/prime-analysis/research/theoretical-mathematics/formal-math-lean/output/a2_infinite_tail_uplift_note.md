# A2 Infinite-Tail Uplift Note

Generated: February 17, 2026  
Artifact: `/Users/adminamn/Documents/New project/research/output/a2_infinite_tail_uplift_refresh_2026-02-17_sf3p5.json`

## Goal
Replace finite-\(M_{ref}\) A2 tail with an analytic infinite-tail majorant:
\[
\tau_\infty(M)\ge \sum_{j>M}\frac{\gamma_j K(\gamma_j)}{|1/2+i\gamma_j|}.
\]

## Method
1. Keep finite observed tail through \(M_{ref}\):
\[
\tau_{fin}(M)=\sum_{M<j\le M_{ref}}\frac{\gamma_j K(\gamma_j)}{|1/2+i\gamma_j|}.
\]
2. Add explicit density-style tail integral beyond \(\gamma_{M_{ref}}\):
\[
\tau_{extra}(M)\le \int_{\gamma_{M_{ref}}}^{\infty}N'(t)_{up}\,\frac{t}{\sqrt{1/4+t^2}}\,K(t)\,dt,
\]
with \(N'(t)_{up}=a\log t+b\), \(a=\frac1{2\pi}\), \(b=0.5\).
3. Define \(\tau_\infty^{maj}(M)=\tau_{fin}(M)+\tau_{extra}(M)\) and recalibrate \(C_\Delta\) on train sup-ratio.

## Current result (gaussian kernel, scale 100)
- \(\beta=2.6\) (fixed),
- uplifted \(C_\Delta=1.3231345976502436\times10^{-3}\) (`safety-factor=3.5`),
- train split: holds, zero violations,
- held-out split: holds, zero violations.

## Interpretation
- For this gaussian-track regime, the computed extra tail term is numerically negligible versus \(\tau_{fin}(M)\), so uplifted constants match the finite-tail scaffold.
- This is a strong bridge step, but full rigor still needs:
  - a formally justified zero-density/zero-count bound replacing the heuristic \(a\log t+b\) envelope,
  - explicit error accounting from sum-to-integral replacement.
