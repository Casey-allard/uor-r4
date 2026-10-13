# Prime Transport Operator Formulation V5

## Purpose

This step adds the first explicit lawful class-adjacent lift:

- `T_z = fiber_phase_lift`

The updated operator is:

- `H_v5 = I + T_b + T_x + T_c + T_y + T_z`

There is still:

- no candidate generation
- no filtering
- no scoring
- no learned weighting

The operator support itself defines lawful motion.

## New Lift

`T_z` is the first explicit class-adjacent operator.

### Definition

- `phi -> (phi + 1) mod 3`

while preserving:

- `b`
- `r`
- `query_semiprime`
- `binding_semiprime`
- `composite_compat_class`
- `admissible_transition`

Spin transport is governed by the explicit named map:

- `fiber_spin_transport_map_v5`

On the current bounded surface this map is the identity on `spin_h`, because
the admissibility word is constant on the transported states.

## Which Class Component `T_z` Changes

`T_z` changes:

- `fiber_class`

`T_z` does not change:

- `radial_class`
- `spin_class`
- `composite_compat_class`

So this is a lawful fiber-phase lift into a neighboring class.

## Lawfulness

`T_z` is lawful by construction because:

- it is explicitly named as a fiber-phase lift
- it preserves the required invariants for that lift on the bounded surface
- no illegal transitions are present in operator support

## Measurements

The CSV is:

- [prime_transport_operator_formulation_v5.csv](/Users/adminamn/AI-Research/ai-router/router-research/results/prime_transport_recursive_system/prime_transport_operator_formulation_v5.csv)

It reports:

- reachable state count
- total nonzero transitions
- nonzero transitions per state
- lawful transition fraction
- expansion versus `v4`
- class distribution
- distinct class identities reached
- fraction of transitions staying in the original class
- fraction entering a new lawful class
- which class component `T_z` changes

## Required Honesty Section

### Was a lawful class-adjacent generator or lift actually added?

yes

### Did operator support escape the single frozen class?

yes

Observed bounded-surface measurements:

- reachable state count:
  - `60`
- total nonzero transitions:
  - `360`
- nonzero transitions per state:
  - `6`
- lawful transition fraction:
  - `360 / 360 = 100.00%`
- illegal transition fraction:
  - `0 / 360 = 0.00%`
- reachable-state expansion versus `v4`:
  - `60 / 20 = 3.00x`
- transition expansion versus `v4`:
  - `360 / 100 = 3.60x`
- distinct class identities reached:
  - `3`
- fraction staying in original class:
  - `120 / 360 = 33.33%`
- fraction entering new lawful class:
  - `240 / 360 = 66.67%`

`T_z` changes:

- `radial_class`: `no`
- `fiber_class`: `yes`
- `spin_class`: `no`
- `composite_compat_class`: `no`

Class distribution across transitions:

- `class_r0_phi0_spin1111_compat100_100: 120 / 360 = 33.33%`
- `class_r0_phi1_spin1111_compat100_100: 120 / 360 = 33.33%`
- `class_r0_phi2_spin1111_compat100_100: 120 / 360 = 33.33%`

## Single Next Step

Proceed to learned weighting over operator entries only, because the operator
now reaches multiple lawful class identities through an explicit named lift.
