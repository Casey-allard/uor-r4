# Prime Transport Framework Enforcement Step 2

## Purpose

This step turns the `lock1` scaffold gate into an actual enforced gate before:

- candidate scoring
- transport selection

It does not add lifts.
It does not add fallbacks.
It does not repair the move geometry.

Its only job is to reveal whether the current rebuilt move space can function
inside the frozen framework when unlawful candidates are removed before
scoring.

## What Was Enforced

For every side-transport candidate:

1. compute the full class tuple:
   - `radial_class = r`
   - `fiber_class = phi`
   - `spin_class = explicit native spin_h`
   - `composite_compat_class = ordered shared-prime mask pair`
2. run the direct-comparison gate from the framework lock
3. if the candidate fails the gate:
   - do not score it
   - do not allow it into the selectable pool
4. if no lawful candidate remains:
   - emit explicit `no_lawful_move`
   - do not fall back

## Was the class gate actually enforced before scoring/selection?

yes

Illegal candidates are not scored and cannot be selected in `lock2`.

## Required Measurements

The verification CSV is:

- [prime_transport_framework_enforcement_step2.csv](/Users/adminamn/AI-Research/ai-router/router-research/results/prime_transport_recursive_system/prime_transport_framework_enforcement_step2.csv)

It reports:

- total candidate count before gate
- total candidate count after gate
- fraction of side-steps with at least one lawful candidate
- fraction of side-steps with zero lawful candidates
- fraction of side-transports that become explicit `no_lawful_move`
- class-specific failure breakdown
- average lawful candidates per side-step
- lawful candidate count distribution

## Required Honesty Section

### Do enough lawful candidates remain for the current rebuilt move geometry to function?

no

Observed counts on the bounded evaluation surface:

- total candidate count before gate:
  - `276`
- total candidate count after gate:
  - `42`
- lawful candidate retention:
  - `42 / 276 = 15.22%`
- side-steps with at least one lawful candidate:
  - `14 / 46 = 30.43%`
- side-steps with zero lawful candidates:
  - `32 / 46 = 69.57%`
- explicit `no_lawful_move` side-transports:
  - `32 / 46 = 69.57%`
- average lawful candidates per side-step:
  - `42 / 46 = 0.9130`
- terminated sequences due to no lawful move:
  - `16 / 16 = 100.00%`

Class-specific failure breakdown:

- radial mismatch:
  - `144 / 276 = 52.17%`
- fiber mismatch:
  - `180 / 276 = 65.22%`
- spin mismatch:
  - `0 / 276 = 0.00%`
- composite compatibility mismatch:
  - `127 / 276 = 46.01%`

Lawful candidate-count distribution:

- `0` lawful candidates:
  - `32 / 46 = 69.57%`
- `3` lawful candidates:
  - `14 / 46 = 30.43%`

## Single Next Recommendation After lock2

Rebuild the native move geometry to add one explicit lawful within-class move
primitive that preserves `(r, phi, spin_h, composite_compat_class)` by
construction, before any further transport learning or selection logic.
