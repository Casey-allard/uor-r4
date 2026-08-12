# INC-0011: Shell Activation In The Time-Expanded Hyperbolic Field

## Hypothesis
If shell geometry expands with the same divergence field as the adaptive 4D phase route, the router can open radial structure instead of remaining trapped in a single shell.

The expected win condition was:
- activate `eval_shells > 1`
- keep proxy `test_mse_after` near `R0`
- avoid route-health collapse

## Motivation
`INC-0010` showed that adaptive `phase4d` fixed sector collapse but left radial routing flat:
- `R5A_K25_M3`: `eval_shells=1.0`, `shell_pmax=1.0`

That meant the geometry was only widening angularly. The shell branch tests whether the time-expanded field should repel radially by default unless an explicit convergence term is applied.

## Code Changes
- Added divergence-aware shell controls to `hyperbolic_router_so8.py`:
  - `--adaptive_shell_growth`
  - `--adaptive_shell_balance`
  - `--adaptive_converge_lambda`
- Added shell diagnostics to `tasks/router_proxy_eval.py`:
  - `adaptive_shell_drive_mean`
  - `adaptive_shell_drive_max`
  - `adaptive_shell_mult_mean`
  - `adaptive_shell_mult_max`
- Added shell-aware sweep health gates in `tools/proxy_sweep.py`:
  - `min_eval_shells`
  - `max_shell_pmax`

## Configs
- Screen: `configs/proxy_transfer_inc0011_shell_screen.json`
- Confirm: `configs/proxy_transfer_inc0011_shell_confirm.json`

## Evidence
- Screen analysis: `results/analysis/inc0011_shell_activation_screen.json`
- Confirm analysis: `results/analysis/inc0011_shell_activation_confirm.json`
- Screen gate: `docs/governance/gates/gate_20260305_153422.md`
- Confirm gate: `docs/governance/gates/gate_20260305_153937.md`

## Screen Result
Single-seed screen on PTB proxy (`train=3000`, `test=1500`, seed `0`):

- `R5A_REF`
  - `test_mse_after=0.0039037`
  - `total_sec=35.026`
  - `eval_shells=1`
  - `shell_pmax=1.000`
- `R5A_SG08`
  - same routing health as `R5A_REF`
  - interpretation: shell multiplier increased, but not enough to cross shell boundaries
- `R5A_SG12`
  - `test_mse_after=0.0039463`
  - `total_sec=41.506`
  - `eval_shells=3`
  - `shell_pmax=0.498`
- `R5A_SG12_SB08`
  - `test_mse_after=0.0039678`
  - `total_sec=34.247`
  - `eval_shells=18`
  - `shell_pmax=0.266`
- `R5A_SG16_SB10`
  - `test_mse_after=0.0039625`
  - `total_sec=31.026`
  - `eval_shells=36`
  - `shell_pmax=0.215`

Screen reading:
- There is a real threshold effect.
- `shell_growth=0.8` did not activate shells.
- `shell_growth=1.2` did.
- stronger repulsion created much richer shell structure, but also increased unseen-route exposure.

Decision after screen:
- kill `R5A_SG08`
- carry `R5A_SG12` and `R5A_SG16_SB10` into confirm

## Confirm Result
Two-seed confirm on PTB proxy (`train=3000`, `test=1500`, seeds `0,1`):

- `R0`
  - `test_mse_after=0.0039450`
  - `total_sec=34.208`
  - `eval_shells=1.0`
  - `shell_pmax=1.000`
- `R5A_REF`
  - `test_mse_after=0.0039009`
  - `total_sec=33.887`
  - `eval_shells=1.0`
  - `shell_pmax=1.000`
  - failed shell gate
- `R5A_SG12`
  - `test_mse_after=0.0039451`
  - `total_sec=33.330`
  - `eval_shells=4.5`
  - `shell_pmax=0.442`
  - `buckets=38.5`
  - `unseen_rate=0.00167`
  - passed shell gate
- `R5A_SG16_SB10`
  - `test_mse_after=0.0039636`
  - `total_sec=35.384`
  - `eval_shells=40.5`
  - `shell_pmax=0.190`
  - `buckets=167.5`
  - `unseen_rate=0.0160`
  - passed shell gate

## Decision
- Promote `R5A_SG12` as the new proxy-transfer lead.
- Keep `R5A_REF` as the best raw-MSE shell-collapsed reference.
- Keep `R5A_SG16_SB10` as an exploratory over-dispersed branch, not the lead.

## Interpretation
The shell-divergence idea is correct.

What the evidence says:
- radial collapse is not structural; it can be broken by the geometry
- moderate shell divergence is enough to activate shells while staying effectively tied with `R0` on mean proxy MSE and slightly ahead on total runtime
- stronger shell divergence continues to improve route health, but begins paying for it with higher unseen-rate and slightly worse MSE

This is the first increment where the time-expanded field is behaving as both:
- angular divergence
- radial divergence

## What Changed In The Project Direction
Before this increment:
- transfer lead = `R5A_K25_M3`
- open problem = shell activation

After this increment:
- transfer lead = `R5A_SG12`
- open problem = controlled convergence after shell activation

## Next Increment
`INC-0012`: convergence-controlled shell geometry

Priority questions:
- can `adaptive_converge_lambda` hold onto the `SG12` shell win while reducing unseen-rate further?
- can the over-dispersed `SG16` branch be pulled back into a better quality/runtime regime?
- is there a stable hysteresis rule between no-shell and over-shell regimes?
