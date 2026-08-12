# Prime Transport spin_H Residual Closure Spec

## Purpose

This step diagnoses the remaining recursive failure inside `spin_H_extended_v1`.

The current bounded lawful transport state is already close:

- canonical recursively closed transport identities: `37178`
- non-canonical branching transport identities: `504`
- recursive consistency rate: `0.986624913751924`

So the remaining failure is not broad. It is a small residual invariant that the current extended transport object still does not absorb.

## 1. Partition: Canonical vs Branching

Bounded lawful transport identities on the current surface split into:

- canonical recursively closed cases: `37178`
- non-canonical branching cases: `504`

Branching is defined mechanically:

- a transport identity is branching if at least one lawful operator component maps it to more than one next transport identity

Observed branching components:

- `fiber_phase_lift_spin_transport` only: `125`
- `radial_transport_unfolding` only: `88`
- both `fiber_phase_lift_spin_transport` and `radial_transport_unfolding`: `291`

No residual branching is caused by:

- `I`
- `T_b`
- `T_x`
- `T_c`
- `T_y`

## 2. Observable Difference Between the Two Sets

### Angular identity

No structural split appears here.

- both canonical and branching cases occur across all `15` angular identities `(b, phi)`

Angular identity is not the residual problem.

### Radial / unfolding structure

The branching set is concentrated in low unfolding load:

- branching unfolding loads:
  - `0: 65`
  - `1: 306`
  - `2: 133`
- canonical unfolding loads:
  - `0: 2126`
  - `1: 9758`
  - `2: 14873`
  - `3: 8701`
  - `4: 1720`

So branching does not span the whole bounded shell. It is concentrated in the lower unfolding part of the lawful surface.

Radial direction itself does not separate the sets:

- branching `radial_direction = +1`: `257`
- branching `radial_direction = -1`: `247`

### Predictive shell

The branching set is restricted to a small predictive subset:

- branching uses `8` current spin classes
- canonical uses `16` current spin classes

Top branching current spins:

- `0010: 118`
- `0100: 65`
- `0000: 65`
- `1000: 63`
- `0001: 60`

So the residual problem is localized to a subset of bounded predictive shells rather than the whole predictive space.

### Closure shell

The current closure shell already records:

- `current_tau`
- `fiber_tau`
- `radial_tau`

But that still does not prevent branching.

The reason is concrete:

- the closure shell stores first-order transported tau images
- it does not store the hidden present twist parity that selects which lawful future fiber/radial continuation is active

### Hidden lawful state features

This is the decisive split.

For all `504` branching transport identities:

- preimage count is exactly `2`
- the two preimages are identical on:
  - `b`
  - `phi`
  - `r`
  - `spin_h`
  - `tau`
  - `composite_compat_class`
  - `query_semiprime`
  - `binding_semiprime`
  - `admissible_transition`
- the two preimages differ only in:
  - `twist ∈ {0, 1}`

Also:

- branching cases with varying composite pair: `0`
- branching cases with varying compatibility mask: `0`
- branching cases with varying admissibility flag: `0`
- branching cases with varying twist: `504`

So the residual branching is not caused by composite identity, compatibility class, or admissibility. It is caused by a hidden twist residue.

## 3. Smallest Residual Invariant

The smallest missing closure-bearing component is:

- `kappa`, the current transport holonomy bit

Operational meaning:

- `kappa` is the present twist parity carried into transport state
- in the current bounded implementation, `kappa` is exactly the hidden operator-side `twist` bit

Why this is the smallest missing invariant:

- every branching transport identity hides exactly one unresolved binary split
- that split is always the pair `{twist = 0, twist = 1}`
- no other hidden lawful state feature varies in the branching set

So the residual invariant is not another large shell. It is one binary holonomy bit.

## 4. Why spin_H_extended_v1 Still Misses It

`spin_H_extended_v1` carries:

- angular identity
- radial/unfolding identity
- predictive shell
- closure shell

But it still omits the current twist parity itself.

That omission matters because:

- `fiber_phase_lift_spin_transport` depends on `twist` directly
- `radial_transport_unfolding` preserves `twist`, and the next lifted image of its target therefore still depends on `twist`
- two operator states with different `twist` can share the same:
  - current spin
  - fiber spin
  - radial spin
  - current tau
  - fiber tau
  - radial tau
- but still diverge at the next lawful continuation

So `spin_H_extended_v1` stores first-order transported shells, but not the present holonomy bit that disambiguates future lawful continuation.

## 5. Required Update Law For The Missing Invariant

Define the residual invariant as `kappa`.

Bounded rule:

- `kappa = twist`

Required recursive update law:

- under `I`:
  - `kappa' = kappa`
- under `T_b`:
  - `kappa' = kappa`
- under `T_x`:
  - `kappa' = kappa`
- under `T_c`:
  - `kappa' = kappa`
- under `T_y`:
  - `kappa' = 1 - kappa`
- under `T_z'`:
  - `kappa' = kappa`
- under `T_r*`:
  - `kappa' = kappa`

This matches the current lawful operator implementation:

- only `T_y` flips the hidden twist axis
- all other generators preserve it

## Why This Is The Right Next Primitive

This is more justified than:

- more weighting:
  - because the residual failure occurs before weighting, at the transport identity level
- more operator generators:
  - because the branching already occurs inside the current lawful support
- more bounded shells:
  - because the missing information is not another predictive prefix or another one-step transported shell

The remaining failure is:

- binary
- local
- explicit
- already present in lawful operator state

So the smallest correct augmentation is to absorb this holonomy bit into the transport identity.

## Honesty

Is the remaining recursive failure now small and specific rather than broad and structural?

`yes`

What exact residual invariant is still missing?

The missing residual invariant is the current transport holonomy bit `kappa`, i.e. the hidden twist parity that all `504` branching transport identities still omit.

## Single Next Step

Implement the smallest augmentation to `spin_H_extended` that adds the holonomy bit `kappa` as native transport-side state, with the generator update law specified above.
