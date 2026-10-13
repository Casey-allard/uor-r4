# Prime Transport Lift Map V1

## Purpose

This step implements the missing cross-space primitive:

- an explicit primary-to-transport lift map

Because full exact `spin_H` is not yet carried natively in the rebuilt line,
this step implements the declared lawful truncation:

- `L_h : (b, phi, r) -> (b, spin_h)`

with fixed:

- `h = 4`

The map is built directly from bounded exact trace data, not from operator
heuristics.

## Lift Definition

### Source space

The source state is the exact primary chart key:

- `(b, phi, r)`

where:

- `b` is preserved exactly
- `phi` is the exact refinement fiber tuple from the bounded trace chart
- `r` is the exact wheel depth from the bounded trace family

### Target space

The target transport state is:

- `(b, spin_h)`

where:

- `spin_h` is the first `h = 4` bits of the exact future admissibility word
  `spin_H`

So the implemented map is:

- preserve `b` exactly
- read exact `spin_H` from bounded trace observations
- truncate to the first four bits
- choose a deterministic representative transport image for each primary chart
  state by majority frequency on the bounded surface

## Mechanical Semantics

### 1. What is preserved exactly

The lift preserves exactly:

- base angle `b`

### 2. How `phi` contributes

`phi` contributes through the observed exact chart sector.

It is not copied into the target state, but it changes the transport image by
changing which future admissibility words are observed at fixed `b`.

### 3. How `r` contributes

`r` contributes through depth-conditioned unfolding.

Deeper `r` values come from deeper trace families and can change the observed
future admissibility word even at fixed `b` and `phi`.

### 4. How predictive unfolding is encoded

Predictive unfolding is encoded explicitly as:

- `spin_h = spin_H[:4]`

So the lift target really is a transport-word state, not a generic proxy
feature.

### 5. What is lost by truncation

The lift discards:

- all future admissibility bits after horizon `4`
- deeper return-grammar distinctions beyond that horizon
- delayed visible-splitting structure that only appears in longer `spin_H`

### 6. Injectivity

This lift is not guaranteed to be injective.

There are two different failure modes:

1. ambiguity:
   the same primary state is observed with multiple exact truncated transport
   words on the bounded surface
2. collision:
   different primary states map to the same representative transport state

Both are reported explicitly in the CSV.

## Bounded Surface

The bounded surface uses all currently available bounded exact traces from:

- `visible_threshold_tight_density_matched_rows.csv`
- `visible_threshold_second_tight_density_matched_rows.csv`

On the current tree this is:

- `6` traces from the 2310 family
- `4` traces from the 30030 family

## Required Honesty Section

### Was the explicit primary-to-transport lift map actually implemented?

yes

### Is full spin_H now present?

no

### Was a lawful explicit truncation L_h implemented instead?

yes

Observed bounded-surface measurements:

- primary states examined:
  - `5390`
- distinct transport states reached:
  - `51`
- collision count:
  - `5339`
- collision fraction:
  - `0.9905380333951762`
- ambiguous primary states:
  - `2345`
- ambiguous primary fraction:
  - `0.43506493506493504`
- skipped short-spin observations:
  - `0`

Representation checks:

- `phi` represented in transport identity:
  - `yes`
- `r` represented in transport identity:
  - `no`
- predictive unfolding represented explicitly:
  - `yes`

So this step does implement the missing primitive in bounded truncated form,
but it also shows that `L_h` at `h = 4` is highly lossy on the bounded exact
surface.

## Results

The implemented lift is:

- `L_h : (b, phi, r) -> (b, spin_h)`
- with `spin_h = spin_H[:4]`

and a deterministic representative image chosen by majority frequency from the
exact bounded trace observations.

What the bounded report shows:

1. `b` is preserved exactly.
2. `phi` contributes materially to the transport identity.
3. predictive unfolding is represented explicitly, because the target state is
   a truncated future admissibility word.
4. `r` does not survive distinctly enough inside this truncated lift.
5. the lift is far from injective:
   - only `51` distinct transport states for `5390` distinct primary states
   - collision fraction about `99.05%`

So the lift primitive now exists explicitly, but it is not sufficiently
informative to support faithful operator derivation beyond a very compressed
shadow of the intended transport geometry.

## Required Outputs

The CSV is:

- [prime_transport_lift_map_v1.csv](/Users/adminamn/AI-Research/ai-router/router-research/results/prime_transport_recursive_system/prime_transport_lift_map_v1.csv)

It reports:

- number of primary states examined
- number of distinct transport states reached
- collision count and collision fraction
- primary-state ambiguity count
- distribution by primary states
- distribution by lifted transport states
- whether `phi` and `r` are represented in transport identity
- whether predictive unfolding is represented explicitly

## Single Next Step

Implement the next missing part of full `spin_H`, because the explicit lift now
exists but the current `h = 4` truncation is too lossy and does not preserve
radial / unfolding depth strongly enough for further operator construction.
