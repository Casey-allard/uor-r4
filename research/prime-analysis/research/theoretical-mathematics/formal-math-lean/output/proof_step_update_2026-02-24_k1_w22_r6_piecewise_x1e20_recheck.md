# Proof Step Update — 2026-02-24 (K1-W22 R6 Piecewise x1e20 Recheck)

## What was done
Ran a larger-range recheck (`x_max = 1e20`) and regenerated the R6 piecewise certificate to test stability of the finite-window majorant at a larger asymptotic scale.

## New research artifacts
- `/Users/adminamn/Documents/New project/research/output/k1_multimode_phase_probe_2026-02-24_x1e20_m12_beta055_recheck.json`
- `/Users/adminamn/Documents/New project/research/output/k1_multimode_phase_probe_2026-02-24_x1e20_m12_beta055_recheck.md`
- `/Users/adminamn/Documents/New project/research/output/r6_piecewise_majorant_certificate_2026-02-24_beta055_m12_x1e20_recheck.json`
- `/Users/adminamn/Documents/New project/research/output/r6_piecewise_majorant_certificate_2026-02-24_beta055_m12_x1e20_recheck.md`
- `/Users/adminamn/Documents/New project/research/output/r6_piecewise_majorant_certificate_2026-02-24_beta055_m12_x1e20_recheck_etafixed.json`
- `/Users/adminamn/Documents/New project/research/output/r6_piecewise_majorant_certificate_2026-02-24_beta055_m12_x1e20_recheck_etafixed.md`

## Key numerical interpretation
Using fixed `eta = 0.034301034952287375` (the earlier x1e18 certificate value):
- new window: `[x0, x1] = [2.530315076480206e17, 1.0e20]`
- certificate `C = 0.3105085436363839`
- window max ratio to bound: `0.9523809523809523`

Compared with the earlier x1e18 certificate:
- previous `C = 0.28804927883866127`
- increase factor `C_new / C_old = 1.0779702170693586`

This indicates the finite-window majorant remains stable under the larger-range recheck (moderate `C` inflation, same `eta` still certifies the window).

## Current math frontier
- Finite-window side is now repeatedly certified on two large windows.
- Remaining gate is unchanged: formal asymptotic tail theorem for `x >= x1` with shared `(C, eta)`.
