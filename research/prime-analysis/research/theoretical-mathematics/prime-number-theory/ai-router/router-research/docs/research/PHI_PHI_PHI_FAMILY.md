# PHI_PHI_PHI Family

## Purpose
Give the `phi`-structured branch one coherent family name.

The implementation labels such as `PHILOG_D36_L065` are useful for exact reproducibility, but they hide the actual mechanism.
This file defines the mechanism-level name.

## Meaning
`PHI_PHI_PHI` means:
- first `PHI`: discrete branch-control ladder
- second `PHI`: `phi`-spaced shell geometry
- third `PHI`: `phi`-step convergence / hysteresis

This family still keeps:
- `pi` for angular geometry
- hyperbolic / exponential law for continuous radial expansion

So this is not an all-`phi` continuum.
It is a `phi`-structured discrete routing architecture built on top of a `pi` / hyperbolic manifold.

## Implementation Mapping
Current lead family instance:
- family name: `PHI_PHI_PHI v1`
- normalized artifact label: `PHI3_K25_D36_L065`
- historical equivalent artifact label: `PHILOG_D36_L065`
- sector mode: `phase4d_adaptive`
- controller: `adaptive_converge_mode=phi_ladder`
- shell mode: `phi_log`
- `delta_r=3.6`

Historical route labels should remain unchanged in logs, configs, parsed JSON, and gate notes.
Future research docs should prefer the family name first and then include the artifact label in parentheses.

Example:
- `PHI_PHI_PHI v1 (artifact: PHI3_K25_D36_L065)`

## Why This Naming Matters
The project is no longer testing isolated knobs.
It is testing coherent geometric families.

`PHI_PHI_PHI` is the first branch in this repo where:
- controller law
- shell metric
- convergence law

all live in the same multiplicative geometry.

That is a real research family, not just a flag bundle.

## Current Status
Confirmed across `INC-0022` and `INC-0023`:
- better quality than `R0`
- healthy shell usage
- lost the stricter `CTRL-0002` runtime control
- current status is quality/health lead, not runtime lead

After `INC-0024`:
- a phase-coupled shell descendant (`PHASE_K25_C035`) improved on the coarse family reference
- the coarse `PHI_PHI_PHI` family should now be treated as the routed-family reference point, not the top candidate

## Next Mechanism Question
A live user-originated hypothesis became the active branch at that stage:
- shells may need to be phase-coupled or phase-shifted, not just radial objects with angular modulation

That hypothesis proved real in `INC-0024`.
The next branch is now a narrower question:
- should the shell-phase law be sparse or quantized instead of continuously biased?
