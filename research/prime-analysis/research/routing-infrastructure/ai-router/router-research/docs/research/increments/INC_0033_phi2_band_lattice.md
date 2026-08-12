# INC-0033: Phi^2 Band Lattice

## Status
Completed.

## Hypothesis
`INC-0031` proved the `phi^2` lattice can widen Hopf.
`INC-0032` proved that per-point threshold gating is not a good operational rescue.
The next branch should replace pointwise gate decisions with a small number of shared discrete rung states.

## Core Idea
1. keep the continuous manifold:
   - `H^4` / Poincare-ball / Hopf
2. keep the discrete lattice idea:
   - recurrence-constrained `phi^2` rung moves
3. stop allocating the rung per point
4. instead quantize local pressure into a few band states:
   - base Hopf state
   - moderate widened state
   - full widened state
5. let many points share the same decomposition so routing is cheaper and less fragmented

## Mathematical Motivation
- `phi^2 = phi + 1` remains the discrete widening law
- the failure of `INC-0032` suggested the geometry signal was real, but the control representation was too fine-grained
- banded states are closer to a real lattice than thresholded free-form local gating
- this also aligns with the user’s earlier intuition that divergence/convergence may work best as small integer `+phi` / `-phi` moves rather than continuous control

## Run
- Config:
  - `configs/proxy_transfer_inc0033_phi2_band_screen.json`
- Analysis:
  - `results/analysis/inc0033_phi2_band_screen.json`
- Gate:
  - `docs/governance/gates/gate_20260306_021036.md`

## Result
2-seed screen means:
- `HOPF_K25_BASE`
  - `mse=0.003888756`
  - `total=62.389s`
  - `sectors=4.0`
  - `shells=3.0`
  - `chi_bin_pmax=0.783`
- `HOPF_PHI2_BAND`
  - `mse=0.003897103`
  - `total=65.725s`
  - `sectors=10.5`
  - `shells=2.0`
  - `chi_bin_pmax=0.942`
  - `fib_band_states_used=3.0`
- `HOPF_PHI2_K25`
  - `mse=0.003902407`
  - `total=116.411s`
  - `sectors=10.5`
  - `shells=2.5`
  - `chi_bin_pmax=0.942`
- `PHASE_K25_C035`
  - `mse=0.003909488`
  - `total=69.983s`
- `R0`
  - `mse=0.003911258`
  - `total=52.127s`
  - fails health on shell collapse

## Interpretation
- The banded shared-state rescue worked partially.
- It preserved almost all of the widening signal from ungated `phi^2`.
- It cut runtime sharply relative to ungated `HOPF_PHI2_K25`.
- It did not clear the operational gate:
  - runtime still exceeded the configured `1.15x` bar vs `R0`
  - `chi` concentration remained severe
- This means the problem is no longer "global vs local gating."
- The problem is now the capacity law itself: the route still widens into too-concentrated `chi` usage even with a cheaper shared-state representation.

## Decision
- Keep `HOPF_K25_BASE` as the routed-quality lead.
- Promote `HOPF_PHI2_BAND` over ungated `HOPF_PHI2_K25` as the widened Hopf geometry candidate.
- Kill banded `phi^2` as an operational rescue path at the screen stage.
- Promote blended Hopf-capacity control as the next live branch.

## Next Branch
- `INC_0034_blended_hopf_capacity.md`

## Fallback
If blended Hopf-capacity control still fails to keep the widened signal while reducing runtime or `chi` concentration, move to:
- explicit route-cost decomposition, or
- a more explicit global-alignment-preserving geometry revision
