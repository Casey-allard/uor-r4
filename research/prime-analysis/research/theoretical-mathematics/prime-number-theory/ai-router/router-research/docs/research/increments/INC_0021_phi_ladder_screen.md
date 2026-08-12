# INC-0021: Discrete Phi-Ladder Controller Screen

## Hypothesis
The continuous `phi_ratio` shell controller is still too soft and too sensitive in the larger-subset regime.
Quantizing shell convergence in `log(phi)` steps should reduce controller chatter, preserve healthy shell usage, and recover runtime within the routed branch family.

## Mechanism Change
- Added `adaptive_converge_mode=phi_ladder`.
- Quantized shell overflow into integer ladder steps:
  - `overflow = max(shell_expand - target_band, 0)`
  - `ladder_steps = ceil(overflow / log(phi))`
  - `shell_converge = lambda_conv * ladder_steps * log(phi)`
- Added diagnostics:
  - `adaptive_shell_ladder_steps_mean`
  - `adaptive_shell_ladder_steps_max`

Interpretation:
- `phi_ratio` keeps a continuous post-threshold slope.
- `phi_ladder` turns shell convergence into multiplicative down-steps in `phi` units.
- This matches the intended `+phi / -phi` branch-control intuition more directly.

## Evidence
- Screen config:
  - `configs/proxy_transfer_inc0021_phi_ladder_screen.json`
- Analysis:
  - `results/analysis/inc0021_phi_ladder_screen.json`
- Gate note:
  - `docs/governance/gates/gate_20260305_202833.md`

## Screen Result
2-seed seed-major larger-subset means:
- `R0`
  - `mse=0.003911258`
  - `total=42.542s`
  - collapsed on shells
- `PHI_D32_L120`
  - `mse=0.003954300`
  - `total=68.881s`
  - health fail on runtime ratio
- `LADDER_D32_L045`
  - `mse=0.003938289`
  - `total=50.493s`
  - health fail on runtime ratio
- `LADDER_D32_L055`
  - `mse=0.003933478`
  - `total=50.592s`
  - health fail on runtime ratio
- `LADDER_D32_L065`
  - `mse=0.003941923`
  - `total=47.407s`
  - health pass

## Reading
- The discrete `phi` ladder is a real controller improvement over `phi_ratio` in this regime.
- `L065` is the best healthy controller candidate from the screen:
  - materially faster than `PHI_D32_L120`
  - healthier than the more aggressive ladder settings
  - still within the configured MSE tolerance vs `R0`
- But the key negative result is more important than the positive one:
  - no healthy routed branch beat `R0` on runtime
  - the controller law improved, but the geometry still did not cross the practical efficiency bar

## Decision
- Close `INC-0021` at the screen stage.
- Do not spend a 4-seed confirm on `LADDER_D32_L065` yet.
- Track `LADDER_D32_L065` as the healthiest routed controller candidate.
- Put `PHI_D32_L120` under review rather than treating it as the current operational lead.

## Mechanistic Interpretation
The result supports a sharper decomposition of constants and geometry:
- `pi` should remain in the continuous angular manifold and time-normalized divergence field.
- `phi` is useful in the discrete branch controller.
- `log(phi)` is the natural additive unit for split/merge and convergence hysteresis.

The remaining mismatch is likely radial metric, not controller choice:
- the controller is now multiplicative / logarithmic
- shell indexing is still linear: `shell = floor(r_eff / delta_r)`
- that is probably the wrong shell law for an expanding hyperbolic field

## Next Increment
- `INC-0022`: replace linear shell indexing with `phi`-spaced log shells while keeping:
  - `pi` for angular normalization
  - hyperbolic time expansion for the radial field
  - `phi_ladder` as the discrete convergence controller
- Only confirm the next branch if it beats the linear-ladder baseline on runtime without losing route health.
