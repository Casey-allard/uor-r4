# Pipeline: Staged Sweep

## Input
- `configs/route_sweep.yaml` for synthetic staged sweeps
- `configs/proxy_transfer_*.json` for LM-proxy transfer sweeps with route-health gates

## Execution Order
- `run_order=route_major` (default): complete all seeds for one route before moving to the next
- `run_order=seed_major`: interleave routes by seed for timing-fairness controls

## Steps
1. Screen stage (1-2 seeds depending on branch cost)
2. Confirm stage (2 seeds)
3. Finalize stage (4 seeds)
4. Write gate note with recommended branch behavior
5. For proxy sweeps, enforce transfer-health thresholds such as shell count, shell concentration, and unseen-route rate
6. Use `seed_major` for control batches when runtime fairness is part of the hypothesis

## Output
- batch logs in `results/raw/`
- analysis JSON in `results/analysis/`
- gate note in `docs/governance/gates/`
