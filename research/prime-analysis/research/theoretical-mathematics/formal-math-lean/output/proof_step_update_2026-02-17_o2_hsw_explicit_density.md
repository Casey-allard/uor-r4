# Proof Step Update (O2 HSW Explicit Density)

Generated: 2026-02-17

## Obligation

- Target: `O2` only

## Removed assumption

- Removed ad hoc placeholder dependence (`rvm_c0`, `rvm_c1`) in canonical O2 density model.

## What changed

- Added explicit zero-count-error model option in A2 uplift:
  - `density_model = n_diff_explicit`
  - finite-difference `N'(t)` upper control from an explicit `|N(T)-M(T)|` bound.
- Added explicit constants in config:
  - `nbound_c1=0.1038`, `nbound_c2=0.2573`, `nbound_c3=9.3675`, `nbound_h=1.0`
- Built and validated canonical O2 artifact:
  - `/Users/adminamn/Documents/New project/research/output/a2_infinite_tail_uplift_2026-02-17_n_diff_explicit_hsw2022_sf3p5.json`
  - `/Users/adminamn/Documents/New project/research/output/a2_tail_majorant_checker_2026-02-17_n_diff_explicit_hsw2022.json`

## Current O2 status

- Train/valid checks: pass on canonical grid.
- Canonical manifest `a2` pointer now targets the HSW explicit-density artifact.

## Remaining O2 work

1. Lock theorem-side citation assumptions in the O2 lemma statement directly.
2. Upgrade from finite-window validation wording to asymptotic theorem form (`forall x>=x0, M>=M0`).
