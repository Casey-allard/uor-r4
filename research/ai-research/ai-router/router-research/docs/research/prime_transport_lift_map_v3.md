# Prime Transport Lift Map V3

## Purpose

This step tests whether a longer adaptive predictive horizon reduces lift
collisions materially enough to support later faithful operator derivation.

The refined lift is an adaptive bounded truncation:

- `L_h^* : (b, phi, r) -> (b, r, spin_h)`

with:

- minimum horizon `h_min = 4`
- maximum cap `h_cap = 12`

This is still a lawful truncation, not full exact `spin_H`.

## Adaptive Lift Definition

### Source space

The source state remains:

- `(b, phi, r)`

### Target space

The target state is:

- `(b, r, spin_h)`

where:

- `b` is preserved exactly
- `r` is preserved exactly
- `spin_h` is a variable-length prefix of the exact future admissibility word

### Adaptive stopping rule

For each exact bounded observation:

1. start at horizon `h = 4`
2. compare the prefix `(b, r, spin_H[:h])` against all observed primary states
3. stop at the first `h <= 12` for which that prefix identifies a unique
   primary state inside the observed `(b, r)` transport fiber
4. if no such `h` exists by the cap, use `h = 12` or the available spin length,
   whichever is smaller

So the realized horizon is explicit and part of the transport identity through
the `NativeSpinHV1.horizon` field.

## Mechanical Semantics

### 1. What is preserved exactly

The adaptive lift preserves exactly:

- `b`
- `r`

### 2. How `phi` contributes

`phi` is not copied into the target tuple, but it contributes through the exact
future-word selection problem inside fixed `(b, r)`.

### 3. How predictive unfolding is encoded

Predictive unfolding is encoded explicitly as:

- variable-length `spin_h`

So this step tests whether longer bounded future words recover more of the
primary chart identity.

### 4. What is still lost

The adaptive lift still discards:

- all future admissibility bits after the realized horizon
- any distinctions that require horizon greater than `12`
- any exact transport structure outside the bounded trace surface

## Required Honesty Section

### Was the horizon extended beyond h=4?

yes

### Did the refined lift materially reduce collisions versus v2?

no

### Is full spin_H now present?

no

### Is the current lift still a lawful truncation?

yes

Observed bounded-surface measurements:

- primary states examined:
  - `5390`
- distinct transport states reached:
  - `230`
- collision count:
  - `5160`
- collision fraction:
  - `0.9573283858998145`
- ambiguous primary states:
  - `4532`
- ambiguous primary fraction:
  - `0.8408163265306122`
- skipped short-spin observations:
  - `0`

Direct comparison:

- collision reduction vs `v1`:
  - `179`
- collision-fraction reduction vs `v1`:
  - `0.033209647495361705`
- collision reduction vs `v2`:
  - `136`
- collision-fraction reduction vs `v2`:
  - `0.02523191094619659`
- ambiguity reduction vs `v1`:
  - `-2187`
- ambiguity reduction vs `v2`:
  - `-2187`

Representation checks:

- `phi` represented:
  - `yes`
- `r` represented under the bounded discrimination test:
  - `no`
- predictive unfolding represented explicitly:
  - `yes`

Adaptive horizon statistics:

- realized horizon min:
  - `5`
- realized horizon mean:
  - `11.879041648007165`
- realized horizon max:
  - `12`

So the horizon was extended substantially, but the refined lift still remains
too lossy to count as materially improved for operator derivation because the
collision rate is still about `95.73%` and ambiguity becomes much worse.

## Results

The adaptive lift is:

- `L_h^* : (b, phi, r) -> (b, r, spin_h)`

with a variable realized horizon chosen by the first bounded prefix that
separates the primary state inside the observed `(b, r)` transport fiber, up to
cap `12`.

The main effect is:

1. collisions do decrease relative to both `v1` and `v2`
2. distinct transport states increase substantially
3. ambiguity rises sharply because the same primary state now realizes many
   different bounded transport identities across observations

That means the adaptive horizon is not enough by itself. It improves
expressivity, but it also exposes that truncated transport identity remains
structurally unstable on the bounded surface.

## Required Outputs

The CSV is:

- [prime_transport_lift_map_v3.csv](/Users/adminamn/AI-Research/ai-router/router-research/results/prime_transport_recursive_system/prime_transport_lift_map_v3.csv)

It reports:

- number of primary states examined
- number of distinct transport states reached
- collision count and collision fraction
- ambiguous primary states and fraction
- direct comparisons against `v1` and `v2`
- representation checks for `phi`, `r`, and predictive unfolding
- adaptive horizon min / mean / max

## Single Next Step

Implement the next missing piece of transport identity, most likely fuller
`spin_H`, before any new operator work, because adaptive bounded truncation
still leaves collisions too high and ambiguity too unstable.
