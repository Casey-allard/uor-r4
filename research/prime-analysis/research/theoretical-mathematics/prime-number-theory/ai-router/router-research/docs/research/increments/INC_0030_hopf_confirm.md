# INC-0030: Pure Hopf Confirm

## Status
Completed.

## Hypothesis
If the `H^4` shell-capacity signal is genuinely the right global geometry, the pure Hopf route should survive a 4-seed larger-subset confirm even if the first explicit `chi` pilot does not become the lead.

## Why This Was Next
`INC-0028` showed:
- pure `HOPF_K25_BASE` beat both `HOPF_CHI2_K25` and `HOPF_CHI3_K25`
- explicit `chi` widened sectors, but gave back both quality and runtime
- the correct next question was therefore not more `chi` tuning
- the correct next question was whether pure Hopf survives a stricter confirm

## Evidence
- config:
  - `configs/proxy_transfer_inc0030_hopf_confirm.json`
- analysis:
  - `results/analysis/inc0030_hopf_confirm.json`
- gate:
  - `docs/governance/gates/gate_20260306_001608.md`

## Confirm Result
- `HOPF_K25_BASE`
  - `test_mse_after=0.0038966`
  - `total_sec=63.244`
  - `eval_sectors=4.0`
  - `eval_shells=3.0`
  - `sector_pmax=0.692`
  - health pass
- `PHASE_K25_C035`
  - `test_mse_after=0.0039044`
  - `total_sec=60.888`
  - `eval_sectors=11.75`
  - `eval_shells=3.5`
  - `sector_pmax=0.545`
  - health pass
- `PHI3_K25_D36_L065`
  - `test_mse_after=0.0039169`
  - `total_sec=63.745`
  - health pass
- `R0`
  - `test_mse_after=0.0039079`
  - `total_sec=57.833`
  - shell-collapse health fail

## Reading
- pure Hopf survives the 4-seed confirm as the healthiest routed-quality branch
- it does not survive as a hardware-efficiency lead
- the route is still over-compressed globally:
  - only `4` effective sectors on average
- but the quality gain is stable enough that the branch must now be treated as real, not as a fluke screen
- `PHASE_K25_C035` remains the better widened/healthier routed family branch, but not the best raw quality route

## Decision
- promote `HOPF_K25_BASE` to routed-quality lead status
- keep `PHASE_K25_C035` as the widened routed-family comparator
- do not promote any routed runtime lead from this confirm
- move the next live geometry branch to `INC-0029`:
  - use a phi/Fibonacci lattice to widen shell / `chi` / phase budgets without discarding the Hopf quality signal
