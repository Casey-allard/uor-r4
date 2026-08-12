# Prime Transport spin_H Core v2

## Purpose

This step refines:

`spin_H_core_v1 = (theta, rho, sigma, h)`

into:

`spin_H_core_v2 = (theta, rho, sigma*, h)`

where `sigma*` is the first explicit canonical regressive sigma mode.

The architectural hierarchy is unchanged:

- parent state first
- projections second
- operators derived afterward

## Sigma Canonicalization

### What `sigma*` is

`sigma*` is a canonical orbit-plus-residue mode object:

- `mode_orbit`
- `current_residue`
- `fiber_residue`
- `radial_residue`

`mode_orbit` is the canonical sorted orbit of the bounded current, fiber, and
radial spin observables.

The residue fields locate the current, fiber, and radial predictive words
inside that shared orbit.

### What `sigma*` carries that `sigma` did not

`sigma_v1` stored three separate bounded words directly:

- `regressive_word`
- `fiber_mode_word`
- `radial_mode_word`

`sigma*` instead carries:

- one parent bounded mode orbit
- plus transport residues inside that orbit

So the parent sigma slot is no longer “three local words”.
It is “one transport mode with explicit positions under lawful fiber and radial
transport”.

### Why `sigma*` is more regressive / canonical

`sigma*` is more canonical because the spin-side parent object is now the orbit,
and the bounded predictive words are recovered from the orbit by projection.

This makes the sigma slot mode-like:

- one parent bounded mode carrier
- multiple derived predictive placements

rather than merely storing the predictive observables themselves.

### Why this is not just another predictive shell

The predictive words are no longer the parent sigma state.
They are orbit members.

The parent sigma state is:

- the canonical bounded orbit
- and the lawful residue positions inside that orbit

That is still bounded, but it is no longer just a renamed future-word tuple.

## Parent Object

`spin_H_core_v2` contains:

- `theta = (b, phi)`
- `rho = (radial_class, unfolding_load, radial_direction, radial_target, radial_target_phi)`
- `sigma* = (mode_orbit, current_residue, fiber_residue, radial_residue)`
- `h = (recursive_phase, fiber_recursive_phase, radial_recursive_phase, holonomy_bit)`

## Projection Relationships

Implemented projections:

- `Pi_pred(spin_H_core_v2) -> spin_h4`
  - `spin_h4 = sigma*.mode_orbit[sigma*.current_residue]`
- `Pi_rec(spin_H_core_v2) -> tau`
  - `tau = h.recursive_phase`
- `Pi_hol(spin_H_core_v2) -> kappa`
  - `kappa = h.holonomy_bit`

So `spin_h4`, `tau`, and `kappa` remain projections of the parent object.

## Recursive Update Law For `sigma*`

The bounded update law remains explicit and mechanical.

For each lawful generator:

- `I`
  - preserve `sigma*`
- `T_b`
  - preserve `sigma*`
- `T_x`
  - recompute the bounded current/fiber/radial spin observables after swap, then
    canonicalize them into the new orbit-plus-residue sigma mode
- `T_c`
  - recompute the bounded current/fiber/radial spin observables after coupled
    transport, then canonicalize them into the new orbit-plus-residue sigma mode
- `T_y`
  - recompute the bounded current/fiber/radial spin observables after twist
    transport, then canonicalize them into the new orbit-plus-residue sigma mode
- `T_z'`
  - recompute the bounded current/fiber/radial spin observables after fiber lift,
    then canonicalize them into the new orbit-plus-residue sigma mode
- `T_r*`
  - recompute the bounded current/fiber/radial spin observables after radial
    unfolding transport, then canonicalize them into the new orbit-plus-residue
    sigma mode

This law is still bounded and approximate, but it is now mode-oriented rather
than word-storage oriented.

## Bounded Audit

On the same bounded lawful surface:

- primary states examined: `45`
- distinct `spin_H_core_v2` states reached: `38315`
- collision count at parent-object level: `0`
- recursive consistency rate at parent-object level: `1.0`

Projection checks:

- `spin_h4` derivable from parent: `yes`
- `tau` derivable from parent: `yes`
- `kappa` derivable from parent: `yes`

Explicit-component checks:

- `theta` explicit: `yes`
- `rho` explicit: `yes`
- `sigma*` explicit: `yes`
- `h` explicit: `yes`

Comparison vs `spin_H_core_v1`:

- parent-state collision change: `0`
- recursive-consistency change: `0`
- distinct state count change: `0`

So this step improves the parent-state semantics of the sigma slot without
changing bounded audit counts on the current lawful surface.

## What Remains Approximate

`spin_H_core_v2` is still not full exact `spin_H`.

Remaining approximation:

- `sigma*` is still bounded because its orbit is built from bounded current,
  fiber, and radial spin observables
- `h` remains a bounded recursive/holonomy carrier
- `rho` remains tied to the bounded radial law

So `sigma*` is materially better than the prior sigma surrogate, but still not
the final exact regressive mode.

## Honesty

Was sigma replaced or materially refined into a canonical regressive mode?

`yes`

Is sigma still just a bounded predictive surrogate?

`no`

Is full exact `spin_H` now present?

`no`

## Single Next Step

Rebuild one operator component directly in terms of `spin_H_core_v2`, because
the weakest parent-state surrogate has now been materially refined into an
explicit canonical sigma mode.
