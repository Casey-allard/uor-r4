# INC-0028: Hopf Chi-Axis Pilot

## Status
Completed.

## Hypothesis
The first pure `phase4d_hopf` pilot improved quality because the global `H^4` shell-capacity signal is real, but it over-compressed the angular field because `chi` was only implicit.
A low-cardinality explicit `chi` axis, or a blended shell-capacity law, should preserve the geometric quality gain while reopening enough angular capacity to recover runtime.

## Why This Is Next
`INC-0026` showed:
- stable `chi_mean ~= 0.33`
- capped Hopf shell capacity near `9`
- direct shell-capacity coupling reduced effective sectors to `4`
- proxy MSE improved, but runtime regressed badly

That pattern says the route is missing a third angular coordinate, not that the geometry branch is wrong.

## Minimal Scope
1. add a low-cardinality `chi` axis (`k_chi in {2,3}`) or a blended shell-capacity law
2. keep shell controller fixed
3. do not add shell-phase coupling to the first `chi` pilot
4. compare against:
   - `PHASE_K25_C035`
   - `PHI3_K25_D36_L065`
   - `R0`

## Acceptance Signal
Keep the branch only if it shows at least one of:
- better runtime than `HOPF_K25_BASE` without losing the quality gain
- lower concentration than `HOPF_K25_BASE` at similar MSE
- a plausible path back toward the routed-family runtime frontier

## Implementation
- added `sector_mode=phase4d_hopf_chi`
- added `--hopf_chi_bins`
- used a measure-aware `chi` coordinate:
  - `u_chi = sin^2(chi)`
  - uniform binning in `u_chi`, not raw `chi`
- added chi diagnostics:
  - `adaptive_chi_bins_used`
  - `adaptive_chi_bin_pmax`
  - `adaptive_chi_bin_entropy`

## Run Notes
- first screen attempt was invalidated by a proxy-evaluator bug:
  - routed branches crashed before emitting `__JSON_SUMMARY__`
  - cause: `tasks/router_proxy_eval.py` passed `hopf_chi_bins` into `phase4d_adaptive_components()`
- bug fixed and the screen was rerun cleanly

## Evidence
- config:
  - `configs/proxy_transfer_inc0028_hopf_chi_screen.json`
- valid analysis:
  - `results/analysis/inc0028_hopf_chi_screen.json`
- valid gate:
  - `docs/governance/gates/gate_20260305_235825.md`
- invalid gate to ignore:
  - `docs/governance/gates/gate_20260305_234515.md`

## Valid Screen Result
- `HOPF_K25_BASE`
  - `test_mse_after=0.0038888`
  - `total_sec=57.498`
  - `eval_sectors=4.0`
  - `eval_shells=3.0`
  - `sector_pmax=0.697`
- `HOPF_CHI3_K25`
  - `test_mse_after=0.0039025`
  - `total_sec=59.118`
  - `eval_sectors=11.0`
  - `eval_shells=2.5`
  - `sector_pmax=0.612`
- `HOPF_CHI2_K25`
  - `test_mse_after=0.0039296`
  - `total_sec=63.581`
  - `eval_sectors=8.0`
  - `eval_shells=2.0`
  - `sector_pmax=0.608`
- `PHASE_K25_C035`
  - `test_mse_after=0.0039095`
  - `total_sec=56.723`
- `PHI3_K25_D36_L065`
  - `test_mse_after=0.0039171`
  - `total_sec=56.528`

## Reading
- explicit `chi` did reopen angular capacity
- but neither `k_chi=2` nor `k_chi=3` improved on pure `HOPF_K25_BASE`
- `chi` as a simple low-cardinality axis is therefore not the missing lead mechanism by itself
- the deeper `H^4` signal is still real because pure Hopf remained the best quality route in the screen

## Decision
- keep `HOPF_K25_BASE` alive and promote it to 4-seed confirm
- kill the first standalone `chi`-axis branch as an immediate lead-replacement path
- promote the phi/Fibonacci lattice branch as the next live geometry branch if pure Hopf confirm does not recover runtime
