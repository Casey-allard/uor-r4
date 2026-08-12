# INC-0031: Phi^2 Rung Forcing

## Status
Screen completed.

## Hypothesis
The first Fibonacci lattice failed because it greedily fit the budget and collapsed back to the same `3x3` Hopf pair.
The next branch should force recurrence-constrained widening directly from the golden-ratio identity:

- `phi^2 = phi + 1`

In integer form:
- `F(n+2) = F(n+1) + F(n)`

The allocator should therefore move by rung construction, not by greedy local fit.

## Core Idea
1. choose a shell/radial rung
2. assign total angular budget from a forced next Fibonacci rung
3. decompose that budget by recurrence, not by free product search
4. use phase / chi pressure only to choose **which rung**, not the raw counts themselves

## Candidate Decompositions
- consecutive-rung triple:
  - shell / `chi` / phase each live on neighboring Fibonacci levels
- forced next-rung angular budget:
  - if Hopf wants about `9`, do not fall back to `3x3`
  - jump to the next valid discrete decomposition
- phase-gated activation:
  - use existing phase shifts to decide when `chi` or phase rung increments activate

## Acceptance Signal
Keep the branch only if it produces a route that is materially different from pure Hopf:
- more than `4` effective sectors
- no major quality collapse
- no extreme runtime blowup

## Why This Was Next
`INC-0029` did not falsify the phi/Fibonacci idea.
It falsified the **first allocator law**.

## Implementation
- Added `sector_mode=phase4d_hopf_fib_rung`.
- Preserved the existing phase-shift geometry:
  - `theta_shift`
  - golden-angle rotation
  - Hopf `chi` coordinate
- Replaced the old greedy fit with a recurrence-constrained rung law:
  - snap Hopf shell capacity to the next Fibonacci rung
  - initialize `chi/theta1/theta2` on predecessor Fibonacci rungs
  - reduce only by discrete rung steps until the product fits `K`
  - keep phase / chi pressure responsible for choosing which rung survives, not the raw counts themselves

## Evidence
- config:
  - `configs/proxy_transfer_inc0031_phi2_rung_screen.json`
- analysis:
  - `results/analysis/inc0031_phi2_rung_screen.json`
- gate:
  - `docs/governance/gates/gate_20260306_010724.md`

## Screen Result
- `HOPF_K25_BASE`
  - `test_mse_after=0.0038888`
  - `total_sec=81.824`
  - `eval_sectors=4.0`
- `HOPF_FIB_K25`
  - `test_mse_after=0.0038888`
  - `total_sec=120.002`
  - `eval_sectors=4.0`
  - `adaptive_chi_bins_used=1.0`
- `HOPF_PHI2_K25`
  - `test_mse_after=0.0039024`
  - `total_sec=115.481`
  - `eval_sectors=10.5`
  - `buckets=20.0`
  - `adaptive_chi_bins_used=2.0`
  - `adaptive_chi_bin_pmax=0.942`
  - `adaptive_fib_forced_total_mean=15.196`
- `PHASE_K25_C035`
  - `test_mse_after=0.0039095`
  - `total_sec=71.515`
  - `eval_sectors=11.5`
- `R0`
  - `test_mse_after=0.0039113`
  - `total_sec=59.113`
  - shell-collapse health fail

## Reading
- the recurrence-constrained branch did what `INC-0029` failed to do:
  - it widened Hopf from `4` effective sectors to about `10.5`
  - it preserved active `chi` usage at `2` bins instead of collapsing to `1`
- the geometry result is real:
  - quality stayed close to pure Hopf
  - quality stayed slightly better than `R0`
  - widening reproduced across both seeds
- the operational result is not good enough:
  - runtime regressed badly
  - `chi` mass stayed highly concentrated (`chi_bin_pmax ~= 0.94`)
  - the branch fails the runtime gate by a large margin

## Decision
- keep `HOPF_PHI2_K25` as a valid geometry candidate
- do not promote it as a transfer lead or confirm candidate
- kill the idea of **global** ungated rung forcing as an operational answer
- queue a narrower follow-up:
  - sparse / gated phi^2 widening that only turns on when local Hopf pressure justifies the extra lattice capacity
