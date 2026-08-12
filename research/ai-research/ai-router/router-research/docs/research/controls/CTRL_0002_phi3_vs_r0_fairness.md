# CTRL-0002: PHI_PHI_PHI vs R0 Fairness Control

## Purpose
Re-check the runtime relation between `R0` and `PHI_PHI_PHI v1` under a cleaner control batch.

## Why This Exists
`INC-0022` and `INC-0023` agree on route health and quality, but they differ materially on runtime magnitude.
That means the lead family deserves one tighter control pass before its runtime claim is treated as locked.

## Config
- `configs/proxy_transfer_ctrl0002_phi3_vs_r0_seedmajor.json`

## Design Choice
This control intentionally lists `PHI3_K25_D36_L065` first and `R0` second in seed-major order.
If the `PHI_PHI_PHI` lead survives that ordering, the runtime claim is much harder to dismiss as favorable sequencing.

## Success Condition
- `PHI3_K25_D36_L065` remains healthy
- `PHI3_K25_D36_L065` stays within the configured MSE gate vs `R0`
- runtime relation remains favorable or at least close enough that the lead can still be described as operationally competitive

## Result
- Analysis:
  - `results/analysis/ctrl0002_phi3_vs_r0_seedmajor.json`
- Gate note:
  - `docs/governance/gates/gate_20260305_214935.md`
- 4-seed seed-major means:
  - `R0`
    - `mse=0.003907888`
    - `total=44.916s`
    - collapsed
  - `PHI3_K25_D36_L065`
    - `mse=0.003901309`
    - `total=52.077s`
    - `runtime_ratio_vs_r0=1.159`
    - healthy on route structure
    - failed the configured runtime gate only

## Decision
- Keep `PHI_PHI_PHI v1` as the transfer quality/health lead.
- Keep `R0` as the operational runtime preference.
- Promote phase-coupled shells as the next geometry branch rather than claiming the coarse-shell family is finished.
