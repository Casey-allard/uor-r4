# CTRL-0001: Seed-Major Lead Validation

## Hypothesis
If `PHI_D32_L120` is a real routed hardware-efficiency lead rather than a route-order artifact, it should survive a larger-subset control batch where routes are interleaved by seed instead of run in route-major blocks.

## Config
- `configs/proxy_transfer_ctrl0001_seedmajor_lead.json`
- Execution order: `seed_major`
- Same larger-subset regime as `INC-0018` confirm (`train=5000`, `test=2500`)

## Evidence
- Analysis: `results/analysis/ctrl0001_seedmajor_lead.json`
- Gate note: `docs/governance/gates/gate_20260305_192810.md`

## Result
4-seed larger-subset control:
- `R0`
  - `mse=0.003907888`
  - `total=44.125s`
  - collapsed baseline
- `D30_FIXED_SG16`
  - `mse=0.003918454`
  - `total=44.432s`
  - health pass
- `PHI_D32_L120`
  - `mse=0.003937115`
  - `total=43.990s`
  - `shells=6.0`
  - `shell_pmax=0.543`
  - `unseen=0.0020`
  - health pass

## Decision
- Retain `PHI_D32_L120` as the routed hardware-efficiency transfer lead.
- Do not call the runtime edge decisive or large.
- Treat the lead as controlled but narrow.

## Interpretation
The route-order control matters:
- `PHI_D32_L120` still ends up as the fastest healthy routed branch in the controlled larger-subset batch
- but the runtime edge vs `R0` is now small rather than dramatic
- `D30_FIXED_SG16` remains slower than `PHI_D32_L120`, so the fixed controller does not reclaim the lead

That means:
- the lead survives
- the lead is not yet a sweeping systems win
- future claims should describe this as a narrow, controlled hardware-efficiency lead rather than a decisive throughput win
