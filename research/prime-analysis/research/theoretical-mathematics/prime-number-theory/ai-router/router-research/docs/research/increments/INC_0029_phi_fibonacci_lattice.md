# INC-0029: Phi/Fibonacci Lattice Pilot

## Status
First screen completed. Needs corrected follow-up.

## Hypothesis
If continuous Hopf-capacity geometry plus a small `chi` axis still fails, the route may need to be discretized more explicitly.
A tri-axial lattice built from Fibonacci counts can preserve geometric self-similarity while making routing tables and split/merge logic more hardware-friendly.

## Core Idea
Use Fibonacci building blocks as the integerization of `phi` ratios across three routing axes:
- shell ladder
- `chi` ladder
- phase-bin ladders (`theta1`, `theta2`)

Convergence/divergence becomes `n -> n ± 1` ladder moves rather than free continuous controller tuning.

Refined geometric reading after `INC-0028` and `INC-0030`:
- keep the continuous manifold hyperbolic (`H^4` in the `B^4` Poincare ball model)
- keep angular geometry on `pi`
- use geodesic shell steps in `log(phi)`
- integerize shell / `chi` / phase capacity on Fibonacci rungs
- use rung moves, not continuous controller drift, for widening and contraction

## Minimal Scope
1. `phi_log` shell ladder remains the shell coordinate
2. allocate shell / `chi` / phase budgets from Fibonacci rungs
3. use `log(phi)` or rung transitions for split/merge hysteresis
4. compare against the best surviving `H4` branch

## Acceptance Signal
Keep the branch only if Fibonacci discretization improves either:
- runtime at comparable quality/health
- stability by reducing controller chatter and seed sensitivity

## Promotion Reason
- explicit `chi` reopened angular capacity but did not beat pure Hopf
- pure Hopf survived 4-seed confirm as the routed-quality lead, but remained over-compressed and slower than `R0`
- the next serious question is whether a discrete phi/Fibonacci lattice can widen capacity without washing out the Hopf quality signal

## First Implementation
- added `sector_mode=phase4d_hopf_fib`
- preserved existing phase-shift geometry:
  - `theta_shift`
  - golden-angle rotation
  - shell-phase machinery remains available outside this first screen
- first allocator design:
  - `hopf_shell_capacity` snapped upward to Fibonacci rungs
  - `chi` bins selected from Fibonacci candidates
  - pair bins selected from Fibonacci candidates under the same total budget

## Evidence
- config:
  - `configs/proxy_transfer_inc0029_fib_screen.json`
- analysis:
  - `results/analysis/inc0029_fib_screen.json`
- gate:
  - `docs/governance/gates/gate_20260306_004144.md`

## First Screen Result
- `HOPF_K25_BASE`
  - `test_mse_after=0.0038888`
  - `total_sec=72.469`
  - `eval_sectors=4.0`
- `HOPF_FIB_K25`
  - `test_mse_after=0.0038888`
  - `total_sec=104.563`
  - `eval_sectors=4.0`
  - `adaptive_chi_bins_used=1.0`
- `PHASE_K25_C035`
  - `test_mse_after=0.0039095`
  - `total_sec=52.871`

## Reading
- the first lattice implementation did **not** actually widen Hopf
- with `K=25` and `adaptive_min_pair_bins=3`, the greedy Fibonacci allocator collapsed back to the same effective Hopf pair budget
- `kchi` stayed at `1`, so the `phi/Fibonacci` branch never expressed the intended tri-axial widening
- the result is still informative:
  - the idea is not falsified
  - the first allocator law was too conservative

## Decision
- do not promote the first `phase4d_hopf_fib` implementation
- keep the phi/Fibonacci direction alive
- reframe the next branch around explicit recurrence-constrained rung forcing instead of greedy product fit
