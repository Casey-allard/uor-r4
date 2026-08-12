# INC-0022: PHI_PHI_PHI Shells

## Hypothesis
Once shell convergence is quantized in `log(phi)` steps, shell indexing itself should stop being linear.

A coherent branch should therefore use:
- `pi` for angular geometry
- hyperbolic / exponential law for continuous radial expansion
- `phi` for discrete branch-control ratios
- `phi`-spaced shells plus `log(phi)` convergence steps for shell activation and merge pressure

If that is correct, replacing linear shells with `phi`-spaced shells should improve the routed branch relative to the linear-ladder comparator without losing route health.

## Family Naming
This branch is the first coherent `PHI_PHI_PHI` family instance:
- first `PHI`: discrete ladder controller
- second `PHI`: `phi`-spaced shell geometry
- third `PHI`: `phi`-step convergence / hysteresis

Implementation artifact label for this increment:
- `PHILOG_D36_L065`

Narratively, this should now be treated as `PHI_PHI_PHI v1`.

## Screen
- Config: `configs/proxy_transfer_inc0022_phi_log_screen.json`
- Analysis: `results/analysis/inc0022_phi_log_screen.json`
- Gate: `docs/governance/gates/gate_20260305_205512.md`

Screen reading:
- `PHILOG_D32_L065`
  - best narrow-slice quality/runtime tradeoff
  - failed screen on shell concentration
- `PHILOG_D36_L065`
  - beat the linear-ladder comparator on mean runtime and mean quality
  - passed the route-health gate
  - promoted to 4-seed confirm

## Confirm
- Config: `configs/proxy_transfer_inc0022_phi_log_confirm.json`
- Analysis: `results/analysis/inc0022_phi_log_confirm.json`
- Gate: `docs/governance/gates/gate_20260305_210615.md`

4-seed larger-subset means:
- `R0`
  - `mse=0.003907888`
  - `total=47.751s`
  - `shells=1.0`
  - `shell_pmax=1.000`
- `LINEAR_D32_L065`
  - `mse=0.003924776`
  - `total=52.032s`
  - `shells=4.25`
  - `shell_pmax=0.584`
- `PHILOG_D36_L065` (`PHI_PHI_PHI v1`)
  - `mse=0.003901309`
  - `total=50.566s`
  - `shells=2.75`
  - `shell_pmax=0.676`

## Decision
- Keep `PHI_PHI_PHI v1` as the current transfer quality lead.
- Keep `R0` as the transfer control baseline and current absolute runtime baseline.
- Kill `PHILOG_D32_L065` as a promotion candidate because shell concentration remained too high.
- Demote `LINEAR_D32_L065` to a comparator; the shell-metric correction beat it on both quality and runtime.

## Mechanistic Reading
`INC-0022` proves the shell metric matters.

The controller was not enough by itself.
Once the shell geometry moved into the same multiplicative family as the controller, the route improved materially:
- better quality than `R0`
- better quality and runtime than the linear-ladder comparator
- healthy shell usage

What it did not prove:
- it did not recover an absolute runtime lead over `R0`
- it did not make the route cheap enough yet to claim hardware-efficiency leadership

## Next Increment
`INC-0023`: budget compression inside the `PHI_PHI_PHI` family.

Hypothesis:
- once shell geometry is corrected, the angular budget should be reducible without collapsing route health
- if so, the remaining runtime gap can be attacked by lowering route complexity rather than changing the geometry family again
