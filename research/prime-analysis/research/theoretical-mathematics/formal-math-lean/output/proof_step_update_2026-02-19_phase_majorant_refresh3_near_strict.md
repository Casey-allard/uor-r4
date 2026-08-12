# Proof Step Update (2026-02-19): K1 Phase-Majorant Refresh 3 (Near-Strict Tail)

## Goal
Test whether the improved finite-range K1 source-shape candidate can cross into strict tail dominance (`tail_ratio_sup_to_amp < 1`) on very high x-windows.

## What was run
Focused cached probes around the previously best band with deeper windows:
- `x_max = 1e20`
- `x_min = 1e7`
- `max_zeta_zeros = 5e4` and `1e5`
- `beta` near `0.74..0.76`
- tail windows including `tail_frac = 0.05`

## Key outputs
- Aggregated evidence:
  - `research/output/k1_phase_majorant_candidate_2026-02-19_refresh3.json`
  - `research/output/k1_phase_majorant_candidate_2026-02-19_refresh3.md`
- Representative strict-tail runs:
  - `research/output/k1_source_shape_probe_2026-02-19_phase_majorant_beta_scan_z100000_xmin1e7_xmax1e20_tail005.json`
  - `research/output/k1_source_shape_probe_2026-02-19_phase_majorant_beta074_z100000_xmin1e7_xmax1e20_tail005_g3072.json`
  - `research/output/k1_source_shape_probe_2026-02-19_phase_majorant_beta076_z50000_xmin1e7_xmax1e20_tail005.json`
  - `research/output/k1_source_shape_probe_2026-02-19_phase_majorant_beta076_z50000_xmin1e7_xmax1e20_tail005_g4096.json`

## Result summary
- Best run in refresh3:
  - `beta = 0.74`
  - `tau = 14.134725142`
  - `zeros_used = 100000`
  - `tail_frac = 0.05`
  - `tail_ratio_sup_to_amp = 0.996728` (strictly below 1)
  - interpretation: `near_strict_single_mode_tail_dominance_finite_range`
- Sub-1 runs found: `4`.
- Broader-tail windows (`tail_frac = 0.10`) remain above 1 (`~1.02+`).

## Interpretation
- This is the strongest finite-range evidence so far for the K1 source-shape route, and aligns with the spinning-top phase-lock narrative in the far tail.
- It is still finite-range and tail-window sensitive, so it does **not** close the non-circular theorem term by itself.

## Frontier status
- Remaining formal blocker is unchanged: one source theorem-term witness (`K1-SOURCE`).
