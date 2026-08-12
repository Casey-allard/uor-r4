# Prime Transport spin_H Core v3

## Purpose

This step refines:

`spin_H_core_v2 = (theta, rho, sigma*, h)`

into:

`spin_H_core_v3 = (theta, rho, sigma**, h)`

where `sigma**` installs the missing nonlocal parent mode identity:

- `global_regressive_mode_index`

The parent-object hierarchy remains unchanged:

- parent state first
- projections second
- operators derived afterward

## Sigma Refinement

### What `global_regressive_mode_index` is

`global_regressive_mode_index` is the bounded canonical generator-action profile
of the current sigma mode.

It indexes:

- the seed orbit of bounded current/fiber/radial spin observables
- the current projected spin observable under each lawful generator
- the successor local orbit under each lawful generator

So it is not just the currently visible orbit.
It is the bounded nonlocal parent identity of that orbit across the full lawful
generator family.

### Why it is the missing nonlocal mode carrier

The prior `sigma*` knew only:

- the local orbit
- and the current/fiber/radial placements inside that orbit

The new `sigma**` knows:

- which bounded mode presentation is seen locally
- and how that mode presents itself under all lawful generator actions

That generator-action profile is the smallest bounded parent identity that
survives lawful composition more globally than a single local orbit snapshot.

## Sigma Structure

`sigma**` now contains:

- `global_regressive_mode_index`
- `current_residue`
- `fiber_residue`
- `radial_residue`

The bounded orbit members are children of the parent mode identity.

## Derived Children Of The Parent Mode

Derived from `sigma**`:

- `mode_orbit = derive_mode_orbit_v3(global_regressive_mode_index)`
- `current_residue`
  - coordinate of the current observable relative to the derived orbit
- `fiber_residue`
  - coordinate of the fiber observable relative to the derived orbit
- `radial_residue`
  - coordinate of the radial observable relative to the derived orbit

Predictive projection:

- `Pi_pred(spin_H_core_v3) -> spin_h4`
- `spin_h4 = mode_orbit[current_residue]`

Recursive/holonomy projections remain:

- `Pi_rec(spin_H_core_v3) -> tau`
- `Pi_hol(spin_H_core_v3) -> kappa`

So `spin_h4`, `tau`, and `kappa` remain derived observables of the parent
transport state.

## Update-Law Role For `global_regressive_mode_index`

The update law is explicit and mechanical.

- `I`
  - preserve `global_regressive_mode_index`
- `T_b`
  - preserve `global_regressive_mode_index`
- `T_x`
  - recompute the bounded generator-action profile from the swap successor state
- `T_c`
  - recompute the bounded generator-action profile from the coupled successor
    state
- `T_y`
  - recompute the bounded generator-action profile from the twist successor
    state
- `T_z'`
  - recompute the bounded generator-action profile from the fiber-lift successor
    state
- `T_r*`
  - recompute the bounded generator-action profile from the radial-unfolding
    successor state

This is still bounded and approximate, but it is no longer only local-orbit
based. The parent sigma identity now carries an explicit nonlocal generator
profile.

## Bounded Audit

On the same bounded lawful surface:

- primary states examined: `45`
- distinct `spin_H_core_v3` states reached: `38315`
- collision count at parent-object level: `0`
- recursive consistency rate at parent-object level: `1.0`

Projection checks:

- `spin_h4` derivable from parent: `yes`
- `tau` derivable from parent: `yes`
- `kappa` derivable from parent: `yes`
- `mode_orbit` derived from `global_regressive_mode_index`: `yes`
- sigma slot carries nonlocal parent mode identity: `yes`

Explicit-component checks:

- `theta` explicit: `yes`
- `rho` explicit: `yes`
- `sigma**` explicit: `yes`
- `h` explicit: `yes`

Comparison vs `spin_H_core_v2`:

- parent-state collision change: `0`
- recursive-consistency change: `0`
- distinct state count change: `0`

So this step improves the sigma slot semantically and hierarchically without
changing the bounded audit counts on the present lawful surface.

## What Remains Approximate

`spin_H_core_v3` is still not full exact `spin_H`.

Remaining approximation:

- `global_regressive_mode_index` is still built from bounded generator-action
  profiles rather than a full exact regressive mode law
- `rho` remains bounded by the current radial transport law
- `h` remains bounded by the current recursive/holonomy carrier

So sigma now has a real parent mode identity, but that identity is still a
bounded approximation of exact canonical sigma.

## Honesty

Was `global_regressive_mode_index` implemented inside sigma?

`yes`

Is sigma still just a bounded orbit-plus-residue presentation of local observables?

`no`

Is full exact `spin_H` now present?

`no`

## Single Next Step

Rebuild one operator component directly in terms of `spin_H_core_v3`, because
sigma now carries an explicit nonlocal parent mode identity rather than only a
local orbit presentation.
