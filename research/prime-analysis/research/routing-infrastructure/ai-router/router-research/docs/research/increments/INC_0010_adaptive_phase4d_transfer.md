# INC-0010: Adaptive Phase4D Transfer Branch

## Hypothesis
A time-expanded adaptive `phase4d` route can widen proxy sector usage fluidly enough to pass the route-health gate while remaining competitive on MSE and runtime.

## Configuration
- Math spec: `docs/research/ADAPTIVE_PHASE4D_SPEC.md`
- Config: `configs/proxy_transfer_inc0010_adaptive_confirm.json`
- Runner: `tools/proxy_sweep.py`
- Dataset: PTB proxy via `tools/prepare_wikitext2.py --dataset auto`
- Subset: `train=3000`, `test=1500`
- Seeds: `0,1`

## Evidence
- Analysis: `results/analysis/inc0010_adaptive_phase4d_confirm.json`
- Gate: `docs/governance/gates/gate_20260305_151230.md`

## Results
- `R5A_K25_M3` (`phase4d_adaptive`, `K=25`, `adaptive_min_pair_bins=3`, `adaptive_time_growth=1.4`, `adaptive_balance=1.2`, `adaptive_angle_growth=0.5`)
  - `test_mse_after=0.0039009`
  - `total_sec=32.664`
  - `buckets=11.0`
  - `eval_sectors=11.0`
  - `pmax_after=0.598`
- `R5B_K25` fixed comparison
  - `test_mse_after=0.0038885`
  - `total_sec=37.588`
  - `buckets=4.0`
  - `eval_sectors=4.0`
  - `pmax_after=0.675`
- `R0`
  - `test_mse_after=0.0039450`
  - `total_sec=35.537`
  - `buckets=8.0`
  - `eval_sectors=8.0`
  - `pmax_after=0.205`

## Decision
- `phase4d_adaptive` is the first route to pass the configured proxy transfer-health gate in this repo.
- The adaptive branch widened sectors materially beyond the fixed route without giving back the relative quality/runtime advantage over `R0`.
- Shell diversity is still unresolved because `eval_shells` remained `1.0`.
- Decision: promote `R5A_K25_M3` as the current stabilized proxy-transfer candidate, while keeping `R5B` as the synthetic lead.

## Next Increment
- `INC-0011`: larger-subset and stricter-seed confirmation for `R5A_K25_M3`, plus explicit shell-activation experiments in the time-expanded hyperbolic field.
