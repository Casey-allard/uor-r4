# INC-0001: Phase4D vs KMeans Validation

## Hypothesis
A 4D polar sectorization (`sector_mode=phase4d`) can outperform or match kmeans sectoring (`sector_mode=kmeans`) under richer non-fast-dev settings while retaining competitive runtime.

## Configuration
- Sweep config: `configs/route_sweep_phase4d_validation.json`
- Stage policy: screen(1), confirm(2), finalize(4)
- Core settings: `N=6000`, `chart_iters=300`, `fast_dev=0`, radial scale on

## Evidence
- Gate note: `docs/governance/gates/gate_20260305_125915.md`
- Summary table: `results/summary.csv` rows where `run_tag` contains `_20260305_125`

## Results (mean over stage)
- R0 (kmeans)
  - screen: `mse_after=0.835581`, `total_sec=34.996`
  - confirm: `mse_after=0.840253`, `total_sec=15.637`
  - finalize: `mse_after=0.829354`, `total_sec=19.527`
- R5 (phase4d)
  - screen: `mse_after=0.824565`, `total_sec=24.365`
  - confirm: `mse_after=0.827565`, `total_sec=17.031`
  - finalize: `mse_after=0.837396`, `total_sec=16.339`

## Decision
- `phase4d` remains promising but is currently unstable across finalize seeds.
- Keep R5 alive as a research branch; do not promote over R0 yet.

## Next Increment
- `INC-0002`: Phase4D dimension geometry study
  - Compare `phase4_dims` variants (`0,1,2,3`, `0,2,4,6`, `1,3,5,7`) to test whether finalize degradation is dimension-selection driven.
