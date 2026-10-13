# Prime Transport Operator Formulation V3

## Purpose

This step extends `H_v2` by adding exactly one new lawful coupled generator:

- `T_c = coupled_torus_kick`

The updated operator is:

- `H_v3 = I + T_b + T_x + T_c`

There is still:

- no candidate generation
- no filtering
- no scoring
- no learned weighting

The operator support itself is the allowed motion.

## New Coupled Generator

`T_c` depends on the ordered composite pair:

- `(query_semiprime, binding_semiprime)`

### Definition

Let

- `g = gcd(query_semiprime, binding_semiprime)`
- `k = (query_semiprime + 2 * binding_semiprime + g) mod 5`

Then

- `T_c(b, query_semiprime, binding_semiprime) = (b + k mod 5, query_semiprime, binding_semiprime)`

with the remaining structured state preserved:

- `r`
- `phi`
- `spin_h`
- `composite_compat_class`
- `admissible_transition`

## Why `T_c` Is Genuinely Coupled

`T_c` is not unary because its step size depends on both composites jointly:

- `query_semiprime`
- `binding_semiprime`
- their interaction through `gcd(query_semiprime, binding_semiprime)`

It is not swap-equivalent because:

- `T_x` changes composite orientation and leaves `b` fixed
- `T_c` leaves composite orientation fixed and changes `b` by an
  interaction-derived amount

It is not equivalent to `T_b` because:

- `T_b` always uses `+1 mod 5`
- `T_c` uses an ordered composite-pair-dependent step

## Lawfulness

Every nonzero transition in `H_v3` preserves the frozen class tuple:

- `radial_class = r`
- `fiber_class = phi`
- `spin_class = spin_h`
- `composite_compat_class`

So the operator support remains 100% lawful.

## Measurements

The CSV is:

- [prime_transport_operator_formulation_v3.csv](/Users/adminamn/AI-Research/ai-router/router-research/results/prime_transport_recursive_system/prime_transport_operator_formulation_v3.csv)

It reports:

- reachable state count
- total nonzero transitions
- nonzero transitions per state
- lawful transition fraction
- hold / `T_b` / `T_x` / `T_c` fractions
- expansion versus `v2`
- class distribution

## Required Honesty Section

### Was a new lawful coupled generator actually added?

yes

### Is the new generator genuinely coupled rather than unary or swap-equivalent?

yes

### Did operator support expand beyond v2?

yes

Observed bounded-surface measurements:

- reachable state count:
  - `10`
- total nonzero transitions:
  - `40`
- lawful transition fraction:
  - `40 / 40 = 100.00%`
- illegal transition fraction:
  - `0 / 40 = 0.00%`
- nonzero transitions per state:
  - `4`
- hold fraction:
  - `10 / 40 = 25.00%`
- `T_b` fraction:
  - `10 / 40 = 25.00%`
- `T_x` fraction:
  - `10 / 40 = 25.00%`
- `T_c` fraction:
  - `10 / 40 = 25.00%`
- reachable state expansion versus `v2`:
  - `10 / 10 = 1.00x`
- transition expansion versus `v2`:
  - `40 / 30 = 1.3333x`

So `T_c` does not add new reachable states on this bounded surface, but it does
add a new lawful transition family and increases operator connectivity.

Class distribution remains:

- `class_r0_phi0_spin1111_compat100_100: 40 / 40 = 100.00%`

## Single Next Step

Add one more lawful generator before learned weighting, because `H_v3` now has
meaningful multi-generator support but still does not expand beyond the single
frozen class on the bounded surface.
