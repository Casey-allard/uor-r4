# Prime Transport Framework Enforcement Step 3

## Purpose

This step adds exactly one new lawful within-class move primitive under the
frozen framework.

It does not add learning.
It does not add fallback.
It does not add lifts.
It does not optimize scoring.

Its only purpose is to increase lawful move supply inside the strict class gate.

## New Primitive

The new primitive is:

- explicit within-class hold

Definition:

- keep the full native state unchanged for that side-step

So the primitive preserves by construction:

- `radial_class = r`
- `fiber_class = phi`
- `spin_class = native truncated spin_h`
- `composite_compat_class`

This is a real move primitive, not a fallback path:

- it is added to the candidate set explicitly
- it is checked by the same class gate as every other candidate
- it can be selected lawfully

## Was a new lawful within-class move primitive actually added?

yes

## Required Measurements

The verification CSV is:

- [prime_transport_framework_enforcement_step3.csv](/Users/adminamn/AI-Research/ai-router/router-research/results/prime_transport_recursive_system/prime_transport_framework_enforcement_step3.csv)

It reports:

- total candidates before gate
- total candidates after gate
- lawful retention
- side-steps with at least one lawful candidate
- side-steps with zero lawful candidates
- explicit `no_lawful_move` fraction
- average lawful candidates per side-step
- terminated sequence fraction
- class-failure breakdown

## Required Honesty Section

### Did lawful candidate supply materially improve under strict gate enforcement?

yes

Measured against `lock2` on the same bounded surface:

- total candidates before gate:
  - `3584`
- total candidates after gate:
  - `776`
- lawful retention:
  - `776 / 3584 = 21.65%`
- side-steps with at least one lawful candidate:
  - `512 / 512 = 100.00%`
- side-steps with zero lawful candidates:
  - `0 / 512 = 0.00%`
- explicit `no_lawful_move` side-transports:
  - `0 / 512 = 0.00%`
- average lawful candidates per side-step:
  - `776 / 512 = 1.5156`
- terminated sequences due to no lawful move:
  - `0 / 16 = 0.00%`

Selection split:

- selected within-class hold primitive:
  - `424 / 512 = 82.81%`
- selected ordinary lawful candidate:
  - `88 / 512 = 17.19%`

Class-failure breakdown:

- radial mismatch:
  - `1512 / 3584 = 42.19%`
- fiber mismatch:
  - `2124 / 3584 = 59.26%`
- spin mismatch:
  - `768 / 3584 = 21.43%`
- composite compatibility mismatch:
  - `1433 / 3584 = 39.98%`

Lawful candidate-count distribution:

- `1` lawful candidate:
  - `424 / 512 = 82.81%`
- `4` lawful candidates:
  - `88 / 512 = 17.19%`

## Single Next Recommendation After lock3

Resume native transport learning only inside the lawful move set by retraining
the existing transport selector over the gate-enforced candidates plus the
explicit within-class hold primitive, with all unlawful candidates removed
before scoring.
