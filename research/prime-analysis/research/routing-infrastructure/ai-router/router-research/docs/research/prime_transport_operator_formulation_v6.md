# Prime Transport Operator Formulation V6

## Purpose

This step refines the class-adjacent lift by replacing the trivial-spin fiber
lift with a nontrivial lawful spin-transport lift:

- `T_z' = fiber_phase_lift_spin_transport`

The updated operator is:

- `H_v6 = I + T_b + T_x + T_c + T_y + T_z'`

There is still:

- no candidate generation
- no filtering
- no scoring
- no learned weighting

The operator support itself defines lawful motion.

## Refined Lift

`T_z'` still changes fiber class:

- `phi -> (phi + 1) mod 3`

It also changes spin class through the explicit named map:

- `fiber_spin_transport_map_v6`

### Spin Transport Rule

For source `spin_h` with horizon `h = 4`:

1. rotate the admissibility word forward by one slot
2. compute
   - `compat_weight = popcount(composite_compat_class)`
   - `occlusion_index = (lifted_phi + twist + compat_weight) mod h`
3. set the transported word at `occlusion_index` to `0`

This preserves:

- fixed spin horizon
- radial class
- composite compatibility class
- admissible bounded lift structure

while making spin transport nontrivial on the bounded surface.

## Which Class Components `T_z'` Changes

`T_z'` changes:

- `fiber_class`
- `spin_class`

`T_z'` does not change:

- `radial_class`
- `composite_compat_class`

## Lawfulness

`T_z'` is lawful by construction because it is an explicit named fiber lift
with explicit spin transport. The legality check requires:

- `r` preserved
- `composite_compat_class` preserved
- spin horizon preserved
- `phi` advanced by exactly one fiber step

No illegal transitions are included in operator support.

## Measurements

The CSV is:

- [prime_transport_operator_formulation_v6.csv](/Users/adminamn/AI-Research/ai-router/router-research/results/prime_transport_recursive_system/prime_transport_operator_formulation_v6.csv)

It reports:

- reachable state count
- total nonzero transitions
- lawful transition fraction
- distinct class identities reached
- distinct spin classes reached
- class distribution
- spin distribution
- expansion versus `v5`
- refined-lift fraction

## Required Honesty Section

### Was the class-adjacent lift refined to include nontrivial lawful spin transport?

yes

### Does the operator now reach more than one spin class through lawful support?

yes

Observed bounded-surface measurements:

- reachable state count:
  - `461`
- total nonzero transitions:
  - `2766`
- lawful transition fraction:
  - `2766 / 2766 = 100.00%`
- illegal transition fraction:
  - `0 / 2766 = 0.00%`
- distinct class identities reached:
  - `27`
- distinct spin classes reached:
  - `13`
- reachable-state expansion versus `v5`:
  - `461 / 60 = 7.6833x`
- transition expansion versus `v5`:
  - `2766 / 360 = 7.6833x`
- fraction of transitions using the refined lift:
  - `461 / 2766 = 16.67%`

The refined lift now changes:

- fiber class: `yes`
- spin class: `yes`

Class distribution is no longer confined to the three `phi` classes from `v5`.
The bounded support now reaches 27 lawful target class identities:

- 9 at `phi = 0`
- 9 at `phi = 1`
- 9 at `phi = 2`

across the following target spin words:

- `0000`
- `0001`
- `0010`
- `0100`
- `0101`
- `0110`
- `0111`
- `1000`
- `1001`
- `1010`
- `1100`
- `1110`
- `1111`

So the lift is no longer just a fiber relabel. It is a lawful class-adjacent
map with explicit nontrivial spin transport.

## Single Next Step

Return to learned weighting over operator entries only, because the lift now
provides meaningful lawful spin-class reach rather than the trivial-spin
fiber-only structure of `v5`.
