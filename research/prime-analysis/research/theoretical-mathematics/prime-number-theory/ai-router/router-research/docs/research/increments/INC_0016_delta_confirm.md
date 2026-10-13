# INC-0016: Delta-Only Confirm

## Hypothesis
If `INC-0015` is correct and `shell_growth` is saturated away, then a delta-only confirm should show:
- `D30` remains the best healthy radial law
- `D28` was only a 2-seed screen curiosity, not a real lead
- `D35` remains a more concentrated trailing branch

## Config
- `configs/proxy_transfer_inc0016_delta_confirm.json`
- `seeds=0,1,2,3`
- larger-subset proxy (`train=5000`, `test=2500`)
- fixed `adaptive_shell_growth=1.8`

## Evidence
- Analysis: `results/analysis/inc0016_delta_confirm.json`
- Gate note: `docs/governance/gates/gate_20260305_175537.md`

## Confirm Result
- `R0`
  - `mse=0.003907888`
  - `total=48.976s`
  - collapsed baseline
- `D28_SG18`
  - `mse=0.003915777`
  - `total=54.294s`
  - `shells=2.0`
  - `shell_pmax=0.580`
  - `unseen=0.0`
- `D30_SG18`
  - `mse=0.003915140`
  - `total=49.334s`
  - `shells=3.0`
  - `shell_pmax=0.579`
  - `unseen=0.0005`
- `D35_SG18`
  - `mse=0.003918287`
  - `total=49.549s`
  - `shells=3.0`
  - `shell_pmax=0.658`
  - `unseen=0.0007`

## Decision
- Kill `D28` as a lead-replacement candidate.
- Keep `D30` as the best healthy `delta_r` law in the current capped regime.
- Keep `D35` as a trailing comparison branch, not the lead.
- Do not change the project lead from this increment.

## Interpretation
`INC-0015` and `INC-0016` together say:
- `shell_growth` is saturated away
- `delta_r` is the active radial control axis
- the best current point is still the `D30` band

`D28` did not survive 4-seed confirmation:
- it is slower than `D30`
- it is slightly worse on MSE
- it opens fewer shells than `D30`

`D35` also stays behind:
- slightly worse quality
- more concentrated shell field

So the search space is now materially smaller and cleaner.

## Important Constraint
This confirm batch did not reproduce the large runtime gap vs `R0` seen in `INC-0014`.
That does not invalidate the `D30` route law, but it does mean:
- absolute wall-clock is still sensitive to batch conditions
- route decisions should continue to rely on within-batch comparisons and route-health first
- the systems track should reduce order/load bias in future sweeps

## Next Increment
`INC-0017`: replace the fixed cap with a ratio-based merge/cap law.

Recommended branch:
- keep `pi` for angular/time normalization
- test `phi` in the controller ratios:
  - split/merge hysteresis
  - convergence cap
  - local radial merge pressure
