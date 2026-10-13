# K1 Phase-Majorant Grid Convergence (2026-02-19)

Generated: 2026-02-19T08:40:16.556248+00:00

- series: beta=0.74 tau=first_zero x=[1e7,1e20] tail=0.05 zeros=100k
- strict sub-1 count: 3
- borderline/above-1 count: 2
- min ratio: 0.996728014
- max ratio: 1.001150421

| grid_size | tail_ratio_sup_to_amp | decay_exp | interpretation | file |
|---:|---:|---:|---|---|
| 1536 | 0.996728014 | 0.150317 | near_strict_single_mode_tail_dominance_finite_range | `research/output/k1_source_shape_probe_2026-02-19_phase_majorant_beta074_z100000_xmin1e7_xmax1e20_tail005_g1536_recheck.json` |
| 3072 | 0.998374433 | 0.146867 | near_strict_single_mode_tail_dominance_finite_range | `research/output/k1_source_shape_probe_2026-02-19_phase_majorant_beta074_z100000_xmin1e7_xmax1e20_tail005_g3072_recheck.json` |
| 6144 | 0.999392510 | 0.140123 | near_strict_single_mode_tail_dominance_finite_range | `research/output/k1_source_shape_probe_2026-02-19_phase_majorant_beta074_z100000_xmin1e7_xmax1e20_tail005_g6144_recheck.json` |
| 7168 | 1.001150421 | 0.143662 | borderline_single_mode_tail_dominance_finite_range | `research/output/k1_source_shape_probe_2026-02-19_phase_majorant_beta074_z100000_xmin1e7_xmax1e20_tail005_g7168_recheck.json` |
| 8192 | 1.001139904 | 0.142994 | borderline_single_mode_tail_dominance_finite_range | `research/output/k1_source_shape_probe_2026-02-19_phase_majorant_beta074_z100000_xmin1e7_xmax1e20_tail005_g8192_recheck.json` |

Interpretation: finite-range near-threshold regime; strict crossing is sensitive to grid density and is not yet robustly stable.
