# Expanded-Grid Tight Sweep Note (n<=2e7)

## Grid
- n values: `300000, 1000000, 2000000, 5000000, 10000000, 20000000`
- bases: `30,210,2310,30030`

## Winner on expanded grid
- A1 artifact: `research/output/a1_smoothing_uplift_pack_2026-02-17_n20m_sf1.00.json`
- A3 artifact: `research/output/a3_channel_energy_uplift_2026-02-17_n20m_h_abs_sf1.05.json`
- A4 artifact: `research/output/a4_uniform_assumption_check_2026-02-17_n20m.json`
- Unified pack: `research/output/uplift_theorem_pack_2026-02-17_n20m_best.json`

## Key constants/metrics
- `C0_uplifted = 0.7775344079136917`
- `C_H_uplifted = 7.130682255903078e-04`
- `A_H = 4.3`
- `unified_ratio_max = 0.015395621934824343`
- `unified_pack_holds = true`

## Runtime observations
- First expanded A4 run was cold (new max_n cache build): `~240s`.
- Warm rerun times with caches present:
  - A4 expanded: `~1.02s`
  - A1 expanded: `~1.05s`
  - A3 expanded: `~2.64s`

