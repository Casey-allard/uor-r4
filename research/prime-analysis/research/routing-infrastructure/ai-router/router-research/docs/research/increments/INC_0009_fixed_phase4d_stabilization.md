# INC-0009: Fixed Phase4D Transfer Stabilization

## Hypothesis
The unhealthy PTB proxy collapse in `R5B` can be corrected inside the fixed `phase4d` route by adjusting `K`, `delta_r`, chart bucket penalty, and growth budget.

## Configuration
- Screen config: `configs/proxy_transfer_inc0009_screen.json`
- Confirm config: `configs/proxy_transfer_inc0009_confirm.json`
- Runner: `tools/proxy_sweep.py`
- Dataset: PTB proxy via `tools/prepare_wikitext2.py --dataset auto`
- Subset: `train=3000`, `test=1500`
- Seeds:
  - screen: `0`
  - confirm: `0,1`

## Evidence
- Screen analysis: `results/analysis/inc0009_proxy_stabilization_screen.json`
- Screen gate: `docs/governance/gates/gate_20260305_144909.md`
- Confirm analysis: `results/analysis/inc0009_proxy_stabilization_confirm.json`
- Confirm gate: `docs/governance/gates/gate_20260305_145544.md`

## Results
- Screen finding:
  - `chart_beta`, `delta_r`, and extra growth budget did not resolve the collapse.
  - Increasing `K` was the only lever that widened active sectors.
- Best fixed candidate after confirm: `R5B_K25`
  - `test_mse_after=0.0038885`
  - `total_sec=37.588`
  - `buckets=4.0`
  - `eval_sectors=4.0`
  - `pmax_after=0.675`
- Control `R0`
  - `test_mse_after=0.0039450`
  - `total_sec=38.564`
  - `buckets=8.0`
  - `eval_sectors=8.0`
  - `pmax_after=0.205`

## Decision
- Fixed `phase4d` can partially widen sectors by raising `K`, but it saturates at `4` active sectors and still fails the concentration gate.
- Shell diversity remained `1` across the whole confirm batch, so this branch did not open the radial structure at all.
- Decision: fixed-`K` stabilization is insufficient. Open an adaptive time-expanded branch.

## Next Increment
- `INC-0010`: adaptive `phase4d` with time-expanded widening, phi-balanced pair allocation, and golden-angle divergence.
