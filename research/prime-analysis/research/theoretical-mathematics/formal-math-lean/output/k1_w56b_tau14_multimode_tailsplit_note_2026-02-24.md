# K1 W56b Tau14 Multimode Tail-Split Note (2026-02-24)

## Goal
Refine the tau14 aligned-remainder route into a finite-mode + tail structure that is less noisy than single-mode closure.

## Method
- Script:
  - `research/fixed_error_psi_tau14_multimode_lattice_decompose.py`
- Fit coefficients on dense global samples.
- Evaluate residual ratio on tau14 phase-locked aligned lattice points.
- Sweep extra nearby zero-modes (`K=0..10`) and aggregate across five windows.

## Key artifacts
- Window runs:
  - `research/output/k1_w56b_tau14_multimode_lattice_decompose_2026-02-24_x10000000.json`
  - `research/output/k1_w56b_tau14_multimode_lattice_decompose_2026-02-24_x20000000.json`
  - `research/output/k1_w56b_tau14_multimode_lattice_decompose_2026-02-24_x3e7.json`
  - `research/output/k1_w56b_tau14_multimode_lattice_decompose_2026-02-24_x40000000.json`
  - `research/output/k1_w56b_tau14_multimode_lattice_decompose_2026-02-24_x5e7.json`
- Five-window K robustness summary:
  - `research/output/k1_w56b_tau14_multimode_k_five_window_summary_2026-02-24.json`
  - `research/output/k1_w56b_tau14_multimode_k_five_window_summary_2026-02-24.md`

## Findings (finite-window)
- Robust K values (`rr<a0` and `delta_tri>0` in all five windows, `a0=0.98`):
  - `K in {0,1,4,5,7,9,10}`
- Best low-complexity candidate:
  - **`K=1`** (tau list `[14.134725142, 21.022039639]`)
  - Five-window range: `rr_max ≈ 0.616540`, `delta_min ≈ 0.363460`.
- Baseline single-mode (`K=0`) remains robust but weaker worst-case margin:
  - `rr_max ≈ 0.896382`, `delta_min ≈ 0.083618`.

## Interpretation for T2
This suggests a practical theorem route:
1. Treat `(tau14, tau21)` as explicit finite principal block.
2. Prove the residual tail ratio bound relative to `A_tau14`.
3. Use the improved margin (`delta_min` evidence from `K=1`) to target symbolic `q<a0`.

The numeric evidence is still finite-window; theorem closure still requires non-empirical tail inequalities.
