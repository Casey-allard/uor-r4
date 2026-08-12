# Prime Transport Radial Law V1

## Purpose

This step replaces the adjacent radial placeholder

- `T_r : r -> (r + 1) mod 3`

with a fuller bounded radial transport law:

- `T_r* = radial_transport_unfolding`

This is a root-geometry step, not a weighting step.

## Geometric Meaning

The new radial law treats radial motion as:

- recursive unfolding / refolding depth transport

not just adjacency in the static radial class label.

Concretely:

- radial motion still acts on `r`
- but direction now depends on predictive unfolding load and recursive phase
- fiber phase may precess during radial transport
- spin and tau are transported as part of the radial law itself

So radial motion is now coupled to:

- unfolding / depth
- fiber phase
- predictive spin structure
- recursive transport phase `tau`

## Law

The new bounded radial law is:

1. compute unfolding charge from:
   - popcount of `spin_h4`
   - `tau.coupled_phase`
   - `tau.lift_phase`
2. choose radial direction:
   - expand if charge parity is even
   - contract if charge parity is odd
3. update radial class:
   - `r' = (r + direction) mod 3`
4. update fiber phase with radial precession:
   - `phi' = phi + tau.twist_phase + 1_{direction<0} mod 3`
5. transport `spin_h4` with recursive-unfolding spin map
6. transport `tau` with recursive-unfolding tau map

So the law is no longer:

- fixed adjacent radial stepping

It is now:

- bounded expansion/contraction with coupled fiber, spin, and tau transport

## Results

Bounded-surface measurements:

- reachable state count:
  - `38315`
- total nonzero transitions:
  - `268205`
- lawful transition fraction:
  - `100%`
- illegal transition fraction:
  - `0%`

Reach:

- distinct radial classes reached:
  - `3`
- distinct class identities reached:
  - `29901`
- distinct spin classes reached:
  - `16`

Transition fractions:

- fraction changing radial class:
  - `38315 / 268205 = 0.1429`
- fraction changing spin class lawfully:
  - `71716 / 268205 = 0.2674`
- fraction changing tau lawfully:
  - `189188 / 268205 = 0.7054`

Comparison vs old radial primitive:

- state-space expansion vs `T_r`:
  - `38315 / 14032 = 2.7305x`
- class diversity vs `T_r`:
  - `29901 / 15287 = 1.9560x`
- spin diversity vs `T_r`:
  - `16 / 16 = 1.0000x`

So the new law does not create more distinct bounded spin words than the old
radial primitive, but it does create far more lawful class diversity and much
stronger tau transport.

## Required Honesty Section

### Was the adjacent radial placeholder replaced or materially refined?

yes

### Does the new radial law couple radial motion to the intended transport geometry more faithfully than `r -> (r + 1) mod 3`?

yes

## Output

The CSV is:

- [prime_transport_radial_law_v1.csv](/Users/adminamn/AI-Research/ai-router/router-research/results/prime_transport_recursive_system/prime_transport_radial_law_v1.csv)

## Single Next Step

Resume operator weighting using the new radial law, because the bounded radial
transport is now structurally richer than the adjacent placeholder and the next
question is whether that richer law improves lawful operator use.
