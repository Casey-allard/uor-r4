# Proof Step Update (2026-02-19): K1 Phase-Majorant Refresh 2

## Goal
Push the single live frontier (`K1-SOURCE`) by stress-testing the finite-range source-shape witness under deeper zero truncation and wider x-windows, while preserving hot/cold cache usage.

## Runs executed (cached probe path)
Using `research/k1_source_shape_probe.py` with cache at `research/cache/k1_source_shape_probe`.

Representative commands:
- `python3 research/k1_source_shape_probe.py --max-zeta-zeros 4096 --tau-candidate-count 1 --beta-grid 0.60 --x-min 5000 --x-max 1e11,1e12,1e13,1e14 --grid-size 2048 --tail-frac 0.35 ...`
- `python3 research/k1_source_shape_probe.py --max-zeta-zeros 4096 --tau-candidate-count 1 --beta-grid 0.52..0.90 --x-min 5000 --x-max 1e13 --grid-size 2048 ...`
- `python3 research/k1_source_shape_probe.py --max-zeta-zeros 20000/50000/100000 --tau-candidate-count 1 --beta-grid 0.68 --x-min 5000 --x-max 1e13..1e14 --grid-size 1024/1536 ...`
- `python3 research/k1_source_shape_probe.py --max-zeta-zeros 20000 --tau-candidate-count 64 --beta-grid 0.68 --x-min 5000 --x-max 1e13 --grid-size 1536 ...`

## New artifacts
- `research/output/k1_source_shape_probe_2026-02-19_phase_majorant_xmax_100000000000.json`
- `research/output/k1_source_shape_probe_2026-02-19_phase_majorant_xmax_1000000000000.json`
- `research/output/k1_source_shape_probe_2026-02-19_phase_majorant_xmax_10000000000000.json`
- `research/output/k1_source_shape_probe_2026-02-19_phase_majorant_xmax_100000000000000.json`
- `research/output/k1_source_shape_probe_2026-02-19_phase_majorant_beta_scan_xmax_10000000000000.json`
- `research/output/k1_source_shape_probe_2026-02-19_phase_majorant_beta_scan2_xmax_10000000000000.json`
- `research/output/k1_source_shape_probe_2026-02-19_phase_majorant_beta_scan_xmin_10000000_xmax_10000000000000.json`
- `research/output/k1_source_shape_probe_2026-02-19_phase_majorant_beta_scan_nodecay_xmax_10000000000000.json`
- `research/output/k1_source_shape_probe_2026-02-19_phase_majorant_z20000_beta068_xmax_10000000000000.json`
- `research/output/k1_source_shape_probe_2026-02-19_phase_majorant_z50000_beta068_xmax_10000000000000_g1536.json`
- `research/output/k1_source_shape_probe_2026-02-19_phase_majorant_z50000_beta068_xmax_100000000000000_g1536.json`
- `research/output/k1_source_shape_probe_2026-02-19_phase_majorant_z100000_beta068_xmax_100000000000000_g1024.json`
- `research/output/k1_source_shape_probe_2026-02-19_phase_majorant_tau_scan64_z20000_beta068_xmax_10000000000000.json`
- `research/output/k1_phase_majorant_candidate_2026-02-19_refresh2.json`
- `research/output/k1_phase_majorant_candidate_2026-02-19_refresh2.md`

## Key numerical findings
- Best finite-range run in this refresh:
  - `beta = 0.68`
  - `tau = 14.134725142` (first nontrivial zeta zero)
  - `zeros_used = 20000`
  - `x-window = [5e3, 1e13]`
  - `tail_ratio_sup_to_amp = 1.337001854`
  - `eta = 0.034981039`
- Across the aggregated 14 runs:
  - tail-ratio range: `[1.337001854, 2.452595171]`
  - eta range: `[0.034981039, 0.079904818]`

## Interpretation
- This materially tightens the finite-range candidate compared with earlier `>2` ratios.
- The dominant mode remains locked at the first zeta zero across scans.
- Single-mode dominance is still not strict (`ratio > 1`), so this does not close `K1-SOURCE`.

## Frontier status
- Formal state unchanged: one remaining open kernel (`K1-SOURCE` / concrete non-circular source theorem term).
