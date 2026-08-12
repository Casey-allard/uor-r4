# Prime Transport Operator Formulation V2

## Purpose

This step extends `H_v1` by adding exactly one new independent lawful operator
generator:

- `T_x = composite_swap`

The updated operator is:

- `H_v2 = I + T_b + T_x`

There is still no candidate generation, no filtering, and no scoring.
Allowed motion is exactly the nonzero support of the operator.

## New Generator

`T_x` acts on a different structural aspect of state than `T_b`.

### Definition

- `T_x(query_semiprime, binding_semiprime) = (binding_semiprime, query_semiprime)`

while preserving:

- `b`
- `r`
- `phi`
- `spin_h`
- `composite_compat_class`
- `admissible_transition`

This is lawful in the current bounded state family because swapping the two
persistent composites leaves the frozen class tuple unchanged.

## Why `T_x` Is Independent

`T_b` changes only the torus base coordinate:

- `b -> (b + 1) mod 5`

`T_x` changes only composite orientation:

- `(query_semiprime, binding_semiprime) -> (binding_semiprime, query_semiprime)`

So `T_x` is:

- not identity
- not equivalent to `T_b`
- acting on a different structural component of state

## Lawfulness

Every nonzero transition in `H_v2` preserves:

- `radial_class = r`
- `fiber_class = phi`
- `spin_class = spin_h`
- `composite_compat_class`

So the operator support remains 100% lawful.

## Measurements

The CSV is:

- [prime_transport_operator_formulation_v2.csv](/Users/adminamn/AI-Research/ai-router/router-research/results/prime_transport_recursive_system/prime_transport_operator_formulation_v2.csv)

It reports:

- reachable state count
- nonzero transitions per state
- total nonzero transitions
- lawful transition fraction
- class distribution
- orbit/connectivity change versus `v1`

## Required Honesty Section

### Was a new independent lawful operator generator added?

yes

### Did operator support expand beyond the v1 single-orbit structure?

yes

Observed bounded-surface measurements:

- reachable state count:
  - `10`
- total nonzero transitions:
  - `30`
- lawful transition fraction:
  - `30 / 30 = 100.00%`
- illegal transition fraction:
  - `0 / 30 = 0.00%`
- nonzero transitions per state:
  - `3`
- hold fraction:
  - `10 / 30 = 33.33%`
- `T_b` fraction:
  - `10 / 30 = 33.33%`
- `T_x = composite_swap` fraction:
  - `10 / 30 = 33.33%`
- reachable state expansion versus `v1`:
  - `10 / 5 = 2.00x`
- nonzero transition expansion versus `v1`:
  - `30 / 10 = 3.00x`

The operator still stays inside one frozen class identity:

- `class_r0_phi0_spin1111_compat100_100: 30 / 30 = 100.00%`

So the support expansion is real, but it is expansion inside the same lawful
class, not a cross-class lift.

## Single Next Step

Proceed to learned weighting over the lawful operator entries only, since the
operator now has meaningful sparse support beyond the `v1` degenerate orbit.
