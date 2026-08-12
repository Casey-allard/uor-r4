# Prime Transport Lift Map V2

## Purpose

This step refines the bounded primary-to-transport lift so that radial /
unfolding information is preserved explicitly in the transport identity.

The refined lift is:

- `L_h^+ : (b, phi, r) -> (b, r, spin_h)`

with fixed:

- `h = 4`

This remains a declared lawful truncation, not full exact `spin_H`.

## Refined Lift Definition

### Source space

The source state is the exact primary chart key:

- `(b, phi, r)`

### Target space

The refined target transport state is:

- `(b, r, spin_h)`

where:

- `b` is preserved exactly
- `r` is preserved exactly
- `spin_h = spin_H[:4]`

The map is still built from bounded exact trace observations and uses a
deterministic representative image per primary state by majority frequency on
the bounded surface.

## Mechanical Semantics

### 1. What is preserved exactly

The refined lift preserves exactly:

- `b`
- `r`

### 2. How `phi` contributes

`phi` is not copied into the target state, but it still contributes through the
observed chart sector. At fixed `(b, r)`, different `phi` values may map to
different transport words.

### 3. How `r` is now preserved

`r` is part of the transport identity directly:

- transport state includes explicit `r`

So radial/unfolding depth is no longer left implicit inside the short spin word
alone.

### 4. How predictive unfolding is encoded

Predictive unfolding remains encoded explicitly as:

- `spin_h = spin_H[:4]`

### 5. What is still lost

The refined lift still discards:

- all future admissibility bits after horizon `4`
- deeper return-grammar distinctions beyond that horizon
- delayed visible-splitting structure that only appears in longer `spin_H`

### 6. Injectivity and collisions

This refined lift may still be non-injective because:

- multiple primary states can share the same `(b, r, spin_h)`
- the same primary state can still be observed with multiple truncated spin
  words on the bounded surface

The CSV reports both collisions and ambiguity explicitly, and compares them
directly against `lift_map_v1`.

## Required Honesty Section

### Was radial/unfolding information preserved explicitly in the refined lift?

yes

### Is full spin_H now present?

no

### Is the current transport identity still a lawful truncation?

yes

Observed bounded-surface measurements:

- primary states examined:
  - `5390`
- distinct transport states reached:
  - `94`
- collision count:
  - `5296`
- collision fraction:
  - `0.9825602968460111`
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
  - `no` under the bounded discrimination test
- predictive unfolding represented explicitly:
  - `yes`

Direct comparison against `lift_map_v1`:

- collision reduction vs `v1`:
  - `43`
- collision-fraction reduction vs `v1`:
  - `0.007977736549165115`
- ambiguity reduction vs `v1`:
  - `0`

So `L_h^+` does preserve `r` syntactically in the target tuple, but the
bounded surface still does not show meaningful additional radial
discrimination.

## Results

The refined lift is:

- `L_h^+ : (b, phi, r) -> (b, r, spin_h)`
- with `spin_h = spin_H[:4]`

This is strictly less lossy than `lift_map_v1`, but only modestly:

- distinct transport states increase from `51` to `94`
- collision fraction drops from about `0.9905` to about `0.9826`
- ambiguity does not improve at all

So the refinement helps, but not enough to treat the bounded transport
identity as faithful yet.

The honest reading is:

1. preserving `r` explicitly was necessary
2. preserving `r` explicitly was not sufficient
3. the dominant remaining loss is still the short predictive horizon rather
   than the absence of an `r` field alone

## Required Outputs

The CSV is:

- [prime_transport_lift_map_v2.csv](/Users/adminamn/AI-Research/ai-router/router-research/results/prime_transport_recursive_system/prime_transport_lift_map_v2.csv)

It reports:

- number of primary states examined
- number of distinct transport states reached
- collision count and collision fraction
- ambiguous primary states and fraction
- representation checks for `phi`, `r`, and predictive unfolding
- direct collision and ambiguity reduction versus `v1`

## Single Next Step

Extend the predictive horizon or refine transport identity further before any
new operator work, because the refined lift reduces collisions only slightly
and still leaves ambiguity unchanged.
