# INC-0012: Convergence-Controlled Shell Geometry

## Hypothesis
If shell divergence is bounded by an overflow-based convergence controller, the route can keep radial structure open without drifting into the over-dispersed regime seen in `INC-0011`.

The expected win condition was:
- keep `eval_shells >= 2`
- keep `shell_pmax < 0.85`
- reduce unseen-route exposure versus the over-dispersed shell branch
- preserve or improve proxy `test_mse_after` and total runtime versus `R0`

## Motivation
`INC-0011` proved shell activation, but left two unsolved problems:
- `R5A_SG12` was healthy but only barely better than `R0`
- `R5A_SG16_SB10` opened the radial field too aggressively and paid for it in unseen-route growth

That suggested the next problem was not more shell repulsion by itself. The route needed a convergence law that clipped excessive shell widening only after the divergence field pushed beyond a target band.

## Code Changes
- Replaced the old shell convergence term in `hyperbolic_router_so8.py` with an overflow-based controller:
  - `shell_expand`
  - `shell_overflow`
  - `shell_converge`
- Added new controls:
  - `--adaptive_converge_target`
  - `--adaptive_converge_hysteresis`
- Added new diagnostics to `tasks/router_proxy_eval.py`:
  - `adaptive_shell_expand_mean|max`
  - `adaptive_shell_overflow_mean|max`
  - `adaptive_shell_converge_mean|max`
- Extended `tools/proxy_sweep.py` route-health gates with:
  - `max_unseen_rate`

## Configs
- Screen: `configs/proxy_transfer_inc0012_convergence_screen.json`
- Confirm: `configs/proxy_transfer_inc0012_convergence_confirm.json`

## Evidence
- Screen analysis: `results/analysis/inc0012_convergence_screen.json`
- Confirm analysis: `results/analysis/inc0012_convergence_confirm.json`
- Screen gate: `docs/governance/gates/gate_20260305_160251.md`
- Confirm gate: `docs/governance/gates/gate_20260305_160951.md`

## Screen Result
Single-seed screen on PTB proxy (`train=3000`, `test=1500`, seed `0`):

- `R5A_SG12_REF`
  - `test_mse_after=0.0039463`
  - `total_sec=34.853`
  - `eval_shells=3`
  - `shell_pmax=0.498`
  - `unseen_rate=0.000667`
- `R5A_SG12_C10`
  - `test_mse_after=0.0039262`
  - `total_sec=32.766`
  - `eval_shells=3`
  - `shell_pmax=0.537`
  - `unseen_rate=0.001333`
- `R5A_SG16_C10`
  - `test_mse_after=0.0039460`
  - `total_sec=36.758`
  - `eval_shells=5`
  - `shell_pmax=0.508`
  - `unseen_rate=0.004000`
- `R5A_SG16_C10_D35`
  - `test_mse_after=0.0039264`
  - `total_sec=34.318`
  - `eval_shells=2`
  - `shell_pmax=0.676`
  - `unseen_rate=0.000000`
- `R5A_SG16_C15`
  - `test_mse_after=0.0039037`
  - `total_sec=31.339`
  - `eval_shells=1`
  - `shell_pmax=1.000`
  - failed shell-health gate

Screen reading:
- the new convergence controller works mechanically
- `SG16` can be pulled back out of the worst over-dispersion regime
- too much convergence (`C15`) collapses shells back to `1`
- `delta_r=3.5` looked important: it kept the stronger-divergence branch healthy without erasing the quality/runtime gain

Decision after screen:
- carry `R0`, `R5A_SG12_REF`, `R5A_SG12_C10`, `R5A_SG16_C10`, and `R5A_SG16_C10_D35` into confirm
- kill `R5A_SG16_C15`

## Confirm Result
Two-seed confirm on PTB proxy (`train=3000`, `test=1500`, seeds `0,1`):

- `R0`
  - `test_mse_after=0.0039450`
  - `total_sec=32.666`
  - `eval_shells=1.0`
  - `shell_pmax=1.000`
- `R5A_SG12_REF`
  - `test_mse_after=0.0039451`
  - `total_sec=32.291`
  - `eval_shells=4.5`
  - `shell_pmax=0.442`
  - `unseen_rate=0.001667`
- `R5A_SG12_C10`
  - `test_mse_after=0.0039362`
  - `total_sec=31.231`
  - `eval_shells=2.5`
  - `shell_pmax=0.542`
  - `unseen_rate=0.000667`
- `R5A_SG16_C10`
  - `test_mse_after=0.0039431`
  - `total_sec=28.815`
  - `eval_shells=3.5`
  - `shell_pmax=0.639`
  - `unseen_rate=0.002000`
- `R5A_SG16_C10_D35`
  - `test_mse_after=0.0039278`
  - `total_sec=28.430`
  - `eval_shells=2.0`
  - `shell_pmax=0.792`
  - `unseen_rate=0.000333`

## Decision
- Promote `R5A_SG16_C10_D35` as the new stabilized proxy-transfer lead.
- Keep `R5A_SG12_C10` as the lower-concentration comparison branch.
- Keep `R5A_SG12_REF` as the no-convergence reference.
- Keep `R5A_SG16_C10` as the higher-shell comparison branch, not the lead.

## Interpretation
The important result is not just that convergence helped. The stronger result is that:
- stronger shell divergence can be useful
- but only if convergence clips the overflow region
- and `delta_r` is allowed to coarsen radial quantization enough to avoid shell chatter

This means `delta_r` is not just a bookkeeping constant. In the current proxy regime, it behaves like a radial merge prior.

What the evidence says:
- `SG12_C10` is a real improvement over `R0` and over the no-convergence `SG12_REF`
- `SG16_C10` shows strong divergence can run fast, but still carries more unseen exposure than necessary
- `SG16_C10_D35` is the first branch to combine:
  - better mean proxy MSE than `R0`
  - meaningfully better runtime than `R0`
  - shell activation that still passes the configured route-health gate

Constraint:
- the current lead is still close to the `shell_pmax` gate (`0.792` vs `0.85`)
- so this is a credible lead, not a finished law

## What Changed In The Project Direction
Before this increment:
- transfer lead = `R5A_SG12`
- open problem = convergence control after shell activation

After this increment:
- transfer lead = `R5A_SG16_C10_D35`
- open problem = explain and widen the safe operating band around strong shell divergence

## Next Increment
`INC-0013`: shell-control phase diagram

Priority questions:
- why does `delta_r=3.5` rescue the stronger-divergence branch?
- where is the phase boundary between shell collapse, healthy shell routing, and over-dispersion?
- can a local merge/hysteresis rule replace hand-tuned radial quantization?
