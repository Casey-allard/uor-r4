# INC-0006: Large-N Robustness for R5B

## Hypothesis
`R5B` remains superior to `R0` when the synthetic workload is scaled up materially.

## Configuration
- Sweep config: `configs/route_sweep_inc0006_r5b_robustness.json`
- `N=12000`
- non-fast mode
- stage policy: screen(1), confirm(2), finalize(4)

## Evidence
- Gate note: `docs/governance/gates/gate_20260305_140216.md`
- Logs:
  - `results/raw/sweep_R0_*_20260305_1348..1355.log`
  - `results/raw/sweep_R5B_*_20260305_1356..1401.log`

## Results
- `R0` finalize mean:
  - `test_mse_after=0.835950`
  - `total_sec=69.786`
- `R5B` finalize mean:
  - `test_mse_after=0.807154`
  - `total_sec=53.679`

## Decision
- `R5B` survives the larger-`N` robustness gate.
- This materially strengthens the thesis that the route is not just a small-scale artifact.

## Next Increment
- `INC-0007`: LM proxy transfer smoke and health check.
