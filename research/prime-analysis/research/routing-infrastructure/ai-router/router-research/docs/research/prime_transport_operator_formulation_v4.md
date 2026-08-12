# Prime Transport Operator Formulation V4

## Purpose

This step extends `H_v3` by adding exactly one new lawful generator on a
different structural axis:

- `T_y = composite_twist`

The updated operator is:

- `H_v4 = I + T_b + T_x + T_c + T_y`

There is still:

- no candidate generation
- no filtering
- no scoring
- no learned weighting

Operator support itself defines lawful motion.

## New Generator

`T_y` acts on a relational twist coordinate, not on the torus base.

### Definition

State is extended with one extra structured coordinate:

- `twist in {0, 1}`

and

- `T_y(twist) = 1 - twist`

while preserving:

- `b`
- `r`
- `phi`
- `spin_h`
- `composite_compat_class`
- `query_semiprime`
- `binding_semiprime`
- `admissible_transition`

## Why `T_y` Is On a Different Structural Axis

`T_y` is not another base kick:

- it does not change `b`
- it does not change `phi`
- it does not change `spin_h`

It changes only the relational twist coordinate.

So it is:

- not identity
- not `T_b`
- not `T_x`
- not `T_c`

## Lawfulness

Every nonzero transition in `H_v4` preserves the frozen class tuple:

- `radial_class = r`
- `fiber_class = phi`
- `spin_class = spin_h`
- `composite_compat_class`

So the operator support remains 100% lawful.

## Measurements

The CSV is:

- [prime_transport_operator_formulation_v4.csv](/Users/adminamn/AI-Research/ai-router/router-research/results/prime_transport_recursive_system/prime_transport_operator_formulation_v4.csv)

It reports:

- reachable state count
- total nonzero transitions
- nonzero transitions per state
- lawful transition fraction
- hold / `T_b` / `T_x` / `T_c` / `T_y` fractions
- expansion versus `v3`
- class distribution
- whether `T_y` changes `phi`, `spin_h`, both, or neither

## Required Honesty Section

### Was a new lawful generator added on a genuinely different structural axis?

yes

### Did operator support expand beyond v3 in a way not reducible to extra base-coordinate connectivity?

yes

Observed bounded-surface measurements:

- reachable state count:
  - `20`
- total nonzero transitions:
  - `100`
- nonzero transitions per state:
  - `5`
- lawful transition fraction:
  - `100 / 100 = 100.00%`
- illegal transition fraction:
  - `0 / 100 = 0.00%`
- hold fraction:
  - `20 / 100 = 20.00%`
- `T_b` fraction:
  - `20 / 100 = 20.00%`
- `T_x` fraction:
  - `20 / 100 = 20.00%`
- `T_c` fraction:
  - `20 / 100 = 20.00%`
- `T_y` fraction:
  - `20 / 100 = 20.00%`
- reachable-state expansion versus `v3`:
  - `20 / 10 = 2.00x`
- transition expansion versus `v3`:
  - `100 / 40 = 2.50x`

`T_y` changes:

- `phi`: `no`
- `spin_h`: `no`
- both: `no`
- neither: `yes`

The support expansion is not reducible to extra base-coordinate connectivity
because `T_y` acts only on the twist axis while preserving `b`.

The bounded surface still remains inside one frozen class identity:

- `class_r0_phi0_spin1111_compat100_100: 100 / 100 = 100.00%`

## Single Next Step

Define the first explicit lawful lift or class-adjacent generator, because
`H_v4` now has substantial lawful support but is still trapped inside one
frozen class on the bounded surface.
