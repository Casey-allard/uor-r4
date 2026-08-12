# Prime Transport spin_H Extended v1

## Purpose

This step replaces the bounded split transport-side shadow state

`(phi, r, spin_h4, tau)`

with one coherent bounded transport object:

`spin_H_extended`

The objective is not to claim full exact `spin_H`. The objective is to stop treating predictive structure and recursive closure as two separate side fields and instead carry them inside one transport-side state.

## Extended Transport State

`spin_H_extended` is implemented as four coupled components:

1. `angular_identity = (base_angle, fiber_phase)`
2. `radial_unfolding = (radial_class, unfolding_load, radial_direction, radial_target, radial_target_phi)`
3. `predictive_shell = (current_spin, fiber_spin, radial_spin)`
4. `closure_shell = (current_tau, fiber_tau, radial_tau)`

## Component Roles

### 1. Angular identity

Angular identity is carried by:

- `base_angle = b`
- `fiber_phase = phi`

This preserves the primary pointing coordinate explicitly.

### 2. Radial / unfolding identity

Radial and unfolding structure are carried by:

- `radial_class = r`
- `unfolding_load = popcount(current_spin)`
- `radial_direction`
- `radial_target`
- `radial_target_phi`

This is richer than carrying `r` alone. It records both current radial class and the bounded radial transport intent induced by the current lawful radial law.

### 3. Predictive structure

Predictive structure is carried by:

- `current_spin`
- `fiber_spin`
- `radial_spin`

`spin_h4` does not remain a separate top-level field. It is absorbed into a predictive shell that records the current bounded predictive word and its lawful fiber/radial transport images.

### 4. Recursive closure structure

Recursive closure structure is carried by:

- `current_tau`
- `fiber_tau`
- `radial_tau`

`tau` does not remain a separate top-level field. It is absorbed into a closure shell that records the current recursive phase class and its lawful fiber/radial transport images.

## What Is Still Approximate

`spin_H_extended` is still bounded, not full exact `spin_H`.

The bounded parts are explicit:

- `current_spin`, `fiber_spin`, `radial_spin` are still built from bounded `spin_h4`
- `current_tau`, `fiber_tau`, `radial_tau` are still built from bounded `tau`

So this step reduces fragmentation, but it does not yet recover the full exact transport-side identity implied by the intended geometry.

## Bounded Audit

Audit surface:

- bounded lawful `H_v8` surface from the current lawful operator stack

Measured results:

- primary states examined: `45`
- distinct transport identities reached: `37682`
- collision count: `0`
- collision fraction: `0.0`
- ambiguity count: `45`
- ambiguity fraction: `1.0`
- recursive consistency rate: `0.986624913751924`
- canonical states under iteration: `37178`
- non-canonical branching states: `504`
- distinct spin classes represented: `511`
- distinct radial classes represented: `3`

Comparison vs `spin_H_candidate_v3 = (phi, r, spin_h4, tau)`:

- collision change: `0`
- ambiguity change: `+30`
- recursive consistency change: `-0.01337508624807604`

Interpretation:

- the extended object stays collision-free on this bounded lawful surface
- angular identity is preserved
- radial identity is preserved
- predictive structure is preserved
- recursive closure is not fully preserved

So this step improves transport-state faithfulness as a coherent object, but it does not yet produce full recursive closure at the extended-state level.

## Representation Checks

- angular identity preserved: `yes`
- radial identity preserved: `yes`
- predictive structure preserved: `yes`
- recursive closure preserved: `no`

## Why This Is Better Than `(phi, r, spin_h4, tau)`

`(phi, r, spin_h4, tau)` stores predictive structure and closure structure as separate bounded attachments.

`spin_H_extended` instead treats transport identity as one composed object with:

- an angular face
- a radial/unfolding face
- a predictive shell
- a closure shell

That is a more faithful bounded model of the intended transport chart than the previous fragmented tuple.

## Honesty

Was a fuller extended transport state implemented beyond `(phi, r, spin_h4, tau)`?

`yes`

Does the new transport state reduce reliance on truncated `spin_h4` and/or bounded `tau`?

`yes`

Is full exact `spin_H` now present?

`no`

## Single Next Step

Identify the next missing component of full exact `spin_H`, specifically the part needed to recover full recursive closure inside the unified extended transport object before any more weighting or local operator work.
