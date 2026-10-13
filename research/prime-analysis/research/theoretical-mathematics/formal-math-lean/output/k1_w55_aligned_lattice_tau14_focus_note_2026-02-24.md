# K1 W55 Tau14 Aligned-Lattice Focus Note (2026-02-24)

## Scope
Refined T2 evidence using structured phase-locked alignment (not sparse logspace coincidence), with `beta=0.62`.

## What changed
- Added aligned-lattice probe:
  - `research/fixed_error_psi_aligned_lattice_probe.py`
- Added multi-window lattice aggregator:
  - `research/k1_aligned_lattice_multiwindow_summary.py`
- Added cache optimization for `psi_events`:
  - exact `xmax` cache can be materialized by truncating the smallest available larger cache file.

## Core results (finite-window)
- Lattice five-window summary (`x=1e7..5e7`, taus `14,21,30,32,40,49`):
  - `research/output/k1_w54_aligned_lattice_five_window_summary_2026-02-24.md`
  - Robust tau set (`rr<a0` and `delta_tri>0` in all windows, `a0=0.98`):
    - **`tau = 14.134725142` only**
- Tau14 sensitivity (`a0=0.95, 0.97, 0.98`) across same five windows:
  - `research/output/k1_w55_tau14_a0sensitivity_summary_2026-02-24_a095.md`
  - `research/output/k1_w55_tau14_a0sensitivity_summary_2026-02-24_a097.md`
  - `research/output/k1_w55_tau14_a0sensitivity_summary_2026-02-24_a098.md`
  - Same robust profile for all three `a0` values (phase-locked points have `|cos|~1`).

## Conservative tau14 finite-window constants (current)
Using W55 (`a0=0.98`):
- `rr_cofinal_max over windows ≈ 0.897808`
- `delta_min = a0 - rr_cofinal_max ≈ 0.102192`
- `A_min over windows ≈ 2.906284e-02`
- `c_cons = A_min * delta_min ≈ 2.9696e-03`

These are empirical finite-window constants, not theorem constants.

## Interpretation for the remaining math gate (T2)
This narrows the theorem-facing target to a single candidate mode:
- Prove aligned remainder ratio bound for `tau=14.134725142` on a cofinal phase-locked subsequence.
- Current numeric evidence is now substantially less alias-sensitive than prior logspace-gated scans.

## Next exact step
Build a theorem-facing decomposition for tau14 on phase-locked points:
1. Isolate tau14 mode with explicit residual expression.
2. Split residual into principal tail + truncation/error terms.
3. Prove each term admits a uniform ratio bound against `A_tau14` on the phase-locked subsequence.
4. Combine to obtain a symbolic `q < a0`.
