# A1 Smoothing Uplift Note

Generated: February 17, 2026  
Artifact: `/Users/adminamn/Documents/New project/research/output/a1_smoothing_uplift_pack_refresh_2026-02-17.json`

## Goal
Upgrade A1 into a split-validated decomposition form using explicit-formula surrogate smoothing:
\[
\left|E/\sqrt{x}-(a_{ref}H_{ref}+b_{ref})\right|
\le C_{smooth}+C_{link}.
\]

## Construction
- Train split fits:
  - \(a_{ref},b_{ref}\) for \(H_{ref}\to E/\sqrt{x}\),
  - \(a_{se},b_{se}\) for surrogate \(S_{ref}\to E/\sqrt{x}\).
- Decompose residual:
  - \(R_{smooth}=|E/\sqrt{x}-(a_{se}S_{ref}+b_{se})|\),
  - \(R_{link}=|(a_{ref}H_{ref}+b_{ref})-(a_{se}S_{ref}+b_{se})|\).
- Set
\[
C_0^{train}= \sup R_{smooth} + \sup R_{link},\qquad
C_0^{uplift}=sf\cdot C_0^{train}.
\]

## Current result
- \(C_{smooth}=0.137996254549\),
- \(C_{link}=0.620577386091\),
- \(C_0^{uplift}=0.910288368768\) (`sf=1.2`).

Checks:
- train split: holds, zero violations,
- held-out split: holds, zero violations.

## Interpretation
- This yields a more structured A1 scaffold than direct one-shot residual maxima.
- Remaining analytic gap:
  1. prove the smoothing approximation term \(R_{smooth}\) with explicit formula tools,
  2. prove deterministic comparison term \(R_{link}\) from channel/surrogate coupling bounds.
