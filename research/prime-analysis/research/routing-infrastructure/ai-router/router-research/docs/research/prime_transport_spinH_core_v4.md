# Prime Transport spin_H Core v4

## Purpose

This step replaces the sigma parent carrier with the direct regressive update
law:

- `sigma' = R_G(sigma)`

for each lawful generator `G`.

The parent transport state is now:

- `spin_H_core_v4 = (theta, rho, sigma_direct, h)`

The old profile tables are no longer the sigma carrier.
They survive only as derived diagnostics of the direct law.

## Direct Sigma Carrier

The sigma carrier is now:

- `SigmaDirectV4(current_mode, fiber_mode, radial_mode, regressive_phase)`

This is the transported sigma state itself.

It carries:

- `current_mode`
  - current predictive projection of sigma
- `fiber_mode`
  - sigma's lawful fiber-transport mode
- `radial_mode`
  - sigma's lawful radial-transport mode
- `regressive_phase`
  - sigma's native regressive phase coordinate

So sigma is no longer represented by sampled generator tables.
It is represented by a direct transported mode state.

## Direct Regressive Update Maps

The implemented direct maps are:

- `R_I`
  - preserve sigma
- `R_Tb`
  - rotate all sigma modes forward by one slot and increment regressive phase
- `R_Tx`
  - swap current/fiber modes, update radial mode by xor with current mode, and
    advance regressive phase
- `R_Tc`
  - update all three sigma modes by direct coupled mixing and rotate the radial
    mode by a phase-controlled step
- `R_Ty`
  - reverse and rotate sigma modes, then advance regressive phase by the twist
    increment
- `R_Tz`
  - promote `fiber_mode` to `current_mode`, advance fiber mode, and mix the
    radial mode against the old current mode
- `R_Tr`
  - promote `radial_mode` to `current_mode`, mix fiber against radial, and
    advance radial mode directly

Operationally:

- `sigma_update_v4(sigma, component)` dispatches to the direct `R_G` map
- sigma is updated natively by those maps
- no generator profile table is used to reconstruct sigma

## Diagnostics Only

The following objects may still be derived:

- `seed_orbit`
- `generator_projection_profile`
- `generator_orbit_profile`
- `generator_composition_profile`

But they are now diagnostics only.

They are computed by:

- applying `R_G` directly to `sigma`
- projecting the resulting sigma states

So they no longer define the parent carrier.

## Projection Hierarchy

The hierarchy remains intact:

- `Pi_pred(spin_H_core_v4) -> spin_h4 = sigma.current_mode`
- `Pi_rec(spin_H_core_v4) -> tau`
- `Pi_hol(spin_H_core_v4) -> kappa`

So:

- `spin_h4` remains derived
- `tau` remains derived
- `kappa` remains derived

None of them is promoted back into foundational state.

## Bounded Audit

On the same bounded lawful surface:

- primary states examined: `45`
- distinct parent states reached: `38315`
- collision count at parent-state level: `0`
- recursive consistency rate: `1.0`
- sigma updated directly by `R_G`: `yes`
- profile fields derived diagnostics only: `yes`
- `spin_h4` derivable: `yes`
- `tau` derivable: `yes`
- `kappa` derivable: `yes`

Comparison against `spin_H_core_v3`:

- collision change: `0`
- recursive-consistency change: `0`
- distinct-state-count change: `0`

So the change is representational and architectural:

- the parent carrier is now the direct law
- the bounded lawful surface partition stays unchanged relative to `v3`

## Honesty

Was sigma replaced with a direct regressive update law `sigma' = R_G(sigma)`?
- `yes`

Are the profile fields now diagnostics rather than the parent carrier?
- `yes`

Is full exact `spin_H` now present?
- `no`

## Next Step

Rebuild one operator component from `spin_H_core_v4`.
