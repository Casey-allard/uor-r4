# INC-0032: Phi^2 Gated Widening

## Status
Screen completed. Closed as a negative operational rescue.

## Hypothesis
`INC-0031` proved that recurrence-constrained `phi^2` rung forcing can widen Hopf cleanly, but it applied the discrete lattice too globally.
The next branch should only activate the extra rung structure where local Hopf pressure is high enough to justify it.

## Core Idea
1. keep pure Hopf as the default coarse geometry
2. activate the `phi^2` rung jump only when local pressure clears a discrete gate
3. allow the route to fall back to the cheaper Hopf allocation elsewhere
4. preserve phase-shift geometry and shell health while reducing runtime and `chi` concentration

## Implemented Gates
- ungated reference:
  - `fib_rung_gate_threshold=0.0`
- phi-tied gate:
  - `fib_rung_gate_threshold=1 / phi ~= 0.618`
- strict gate:
  - `fib_rung_gate_threshold=0.85`

## Acceptance Signal
Keep the branch only if it preserves the main `INC-0031` geometry gain:
- more than `4` effective sectors
- active `chi` usage above `1`
- bounded quality loss vs pure Hopf

and fixes at least one operational failure:
- materially lower runtime than `HOPF_PHI2_K25`
- materially lower `chi_bin_pmax`

## Evidence
- config:
  - `configs/proxy_transfer_inc0032_phi2_gated_screen.json`
- analysis:
  - `results/analysis/inc0032_phi2_gated_screen.json`
- gate:
  - `docs/governance/gates/gate_20260306_014339.md`

## Screen Result
- `HOPF_K25_BASE`
  - `test_mse_after=0.003888756`
  - `total_sec=61.261`
  - `eval_sectors=4.0`
  - `adaptive_chi_bin_pmax=0.783`
- `HOPF_PHI2_K25`
  - `test_mse_after=0.003902407`
  - `total_sec=104.365`
  - `eval_sectors=10.5`
  - `adaptive_chi_bin_pmax=0.942`
- `HOPF_PHI2_G062`
  - `test_mse_after=0.003905332`
  - `total_sec=96.498`
  - `eval_sectors=8.0`
  - `adaptive_chi_bin_pmax=0.943`
- `HOPF_PHI2_G085`
  - `test_mse_after=0.003894006`
  - `total_sec=97.079`
  - `eval_sectors=5.5`
  - `adaptive_chi_bin_pmax=0.996`
- `PHASE_K25_C035`
  - `test_mse_after=0.003909488`
  - `total_sec=58.525`
- `R0`
  - `test_mse_after=0.003911258`
  - `total_sec=46.961`
  - shell-collapse health fail

## Reading
- threshold gating reduced `phi^2` cost relative to the ungated branch
- the reduction was not enough:
  - both gated variants stayed far slower than pure Hopf
  - both gated variants stayed about `2x` slower than `R0`
- the `1 / phi` gate did not recover quality
- the strict gate recovered some quality, but made `chi` concentration worse
- this means the problem is not only “too much widening everywhere”
- the problem is that per-point threshold gating is still the wrong operational form for the lattice

## Decision
- kill per-point threshold gating as the primary rescue for the `phi^2` rung family
- keep the `phi^2` widening result itself as valid geometry evidence
- keep `HOPF_K25_BASE` as the routed-quality lead
- keep `PHASE_K25_C035` as the widened routed-family comparator
- queue the next discrete geometry branch:
  - pre-quantized / banded `phi^2` lattice states that share a small number of rung decompositions instead of evaluating a local gate everywhere

## Why This Still Matters
`INC-0031` did not fail mathematically.
`INC-0032` showed that sparse activation by threshold is still too expensive and too concentrated.
The next branch should therefore simplify the lattice representation itself, not tune the gate harder.
