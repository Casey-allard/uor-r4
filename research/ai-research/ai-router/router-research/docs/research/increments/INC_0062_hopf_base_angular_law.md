# INC-0062: Hopf-Base Angular Route Law

## Status
Complete, with corrected reruns recorded on 2026-03-11.

## Trigger
`INC-0061` showed that shell-only measure correction is not enough:
- raw equal-mass `H^4` shells fail
- bounded `H^4`-mass shells also fail

The local theory corpus points to a stronger structural split:
- coarse routing should live on the Hopf base
- common fiber phase should remain separate

## Hypothesis
The current route law is using the wrong angular variables for coarse routing.

Instead of routing directly on `(theta1, theta2)`, the coarse route should use the Hopf-base coordinates:
- `eta` / `chi`
- `delta`

and keep
- `alpha`

as the fiber phase.

If that is right, then coarse routing on the Hopf base should:
- improve angular measure behavior
- preserve or improve route health
- create a cleaner foundation for later phase-necessity testing

## Minimal Scope
1. Add a Hopf-base sector mode that routes on coarse base coordinates.
2. Keep the current cheap routed training schedule fixed.
3. Screen against:
   - `phase4d_hopf`
   - `phase4d_hopf_fib_band`
   - the new Hopf-base angular law
4. Measure:
   - proxy quality/runtime
   - Hopf angular-mass diagnostics
   - geodesic neighborhood preservation
   - route health

## Why This Was Reopened
The first `INC-0062` closeout was directionally useful, but it still leaned on
diagnostics that were too chart-distribution-heavy to separate same-chart Hopf
route laws cleanly.

The corrected rerun added route-law-sensitive Hopf sector diagnostics:
- within-sector `chi` spread
- sector-level `alpha` entropy gap
- cleaner base/fiber occupancy contrast

That does not turn `INC-0062` into a final proof of full angular closure, but it
does make the branch materially more informative than the original writeup.

## Corrected Screen
- Config:
  - `configs/proxy_transfer_inc0062_hopf_base_screen_corrected.json`
- Analysis:
  - `results/analysis/inc0062_hopf_base_screen_corrected.json`
- Gate:
  - `docs/governance/gates/gate_20260311_101015.md`

2-seed means:
- `HOPF_BASE_K25_PHI`
  - `mse=0.003900`
  - `total=6.767s`
  - `buckets=22.5`
  - `shell_pmax=0.522`
  - `hopf_base_mass=1.0218`
  - `hopf_sector_chi_std=0.0093`
  - `hopf_sector_alpha_gap=0.6209`
- `HOPF_K25_BASE_PHI`
  - `mse=0.003903`
  - `total=6.541s`
  - `hopf_sector_chi_std=0.2810`
- `HOPF_PHI2_BAND_PHI`
  - `mse=0.003905`
  - `total=7.580s`
  - `hopf_sector_chi_std=0.2536`
- `R0`
  - `mse=0.003916`
  - `total=7.304s`
  - shell-collapse fail

Reading:
- the Hopf-base route law is viable and healthy
- it improved quality and runtime vs `R0`
- the new sector-sensitive diagnostics sharply separate it from the same-chart
  routed references
- the branch became the clearest base/fiber separation control and justified a
  corrected confirm

## Corrected Confirm
- Config:
  - `configs/proxy_transfer_inc0062_hopf_base_confirm_corrected.json`
- Analysis:
  - `results/analysis/inc0062_hopf_base_confirm_corrected.json`
- Gate:
  - `docs/governance/gates/gate_20260311_101213.md`

4-seed means:
- `HOPF_K25_BASE_PHI`
  - `mse=0.003895`
  - `total=6.093s`
  - `hopf_base_mass=1.0168`
  - `hopf_sector_chi_std=0.2815`
- `HOPF_PHI2_BAND_PHI`
  - `mse=0.003903`
  - `total=6.061s`
  - `hopf_base_mass=0.9963`
  - `hopf_sector_chi_std=0.2491`
- `HOPF_BASE_K25_PHI`
  - `mse=0.003906`
  - `total=6.035s`
  - `buckets=21.0`
  - `shell_pmax=0.5378`
  - `hopf_base_mass=1.0234`
  - `hopf_sector_chi_std=0.0107`
  - `hopf_sector_alpha_gap=0.6212`
- `R0`
  - `mse=0.003911`
  - `total=6.687s`
  - shell-collapse fail

## Conclusion
- `phase4d_hopf_base` is real, healthy, and now canonically corrected.
- The corrected diagnostics show something the original writeup could not show
  cleanly:
  - within-sector `chi` spread is dramatically lower than for pure Hopf,
    widened Hopf, and `R0`
  - the branch preserves a strong base/fiber separation signature while staying
    inside the routed quality/runtime band
- Pure `phase4d_hopf` still kept the best confirm MSE.
- So the corrected durable read is:
  - `phase4d_hopf_base` is the canonical no-fiber-phase coarse-address control
  - it provides positive evidence that the coarse address should live on the
    Hopf base rather than on raw fiber coordinates
  - it still does **not** close the entire measure-consistent route-law program
    by itself

## Acceptance
- route-health preserved: yes
- stayed inside the routed quality/runtime band: yes
- cleaner base/fiber interpretation: yes
- strengthened the angular-law evidence materially: yes
- proved the full angular-correction hypothesis outright: no
