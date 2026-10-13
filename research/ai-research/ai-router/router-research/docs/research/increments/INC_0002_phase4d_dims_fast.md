# INC-0002: Phase4D Dimension Selection (Fast Staged)

## Hypothesis
Phase4D performance sensitivity is largely driven by which 4 dimensions define the polar chart sectors.

## Configuration
- Sweep config: `configs/route_sweep_phase4d_dims_fast.json`
- Stage policy: screen(1), confirm(2), finalize(2)
- Fast profile enabled (`fast_dev=1`) to select candidate dimensions quickly.

## Evidence
- Gate note: `docs/governance/gates/gate_20260305_130139.md`
- Summary rows: `results/summary.csv` where `run_tag` contains `_20260305_130`

## Results (finalize mean)
- R0 (`kmeans`): `0.947021`
- R5A (`phase4_dims=0,1,2,3`): `0.944876`
- R5B (`phase4_dims=0,2,4,6`): `0.936811`
- R5C (`phase4_dims=1,3,5,7`): `0.943956`

## Decision
- Promote `phase4_dims=0,2,4,6` as the current phase4d candidate default.
- Keep testing against R0 in non-fast-dev finalize runs before promotion.

## Next Increment
- `INC-0003`: Non-fast-dev confirm/finalize for R0 vs R5B with cache disabled for pure runtime comparison.
