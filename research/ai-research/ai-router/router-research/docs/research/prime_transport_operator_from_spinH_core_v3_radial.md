# Prime Transport Operator From spin_H_core_v3 Radial

## Scope

This step rebuilds exactly one operator component:

- `T_r*`

The rebuilt component is derived directly from:

- `spin_H_core_v3 = (theta, rho, sigma**, h)`

No new generator was added.
No weighting or benchmark step was run.
No other operator component was rebuilt in this step.

## Rebuilt radial law

The rebuilt radial transport component is:

- `radial_transport_component_v15`

It uses the canonical parent transport state as active state:

- `theta = (b, phi)` for radial phase transport and spin injection
- `rho` for the radial target class, radial target phase, radial direction, and unfolding load
- `sigma**.global_regressive_mode_index` for the radial generator projection, radial generator orbit, and residue-relative orbit selection
- `h` for recursive-phase and holonomy-aware tau/spin transport

The rebuilt law therefore treats radial transport as a parent-state-driven transport update, not as the older bounded surrogate radial placeholder.

## Structural results on the bounded lawful surface

- reachable state count: `236969`
- total nonzero transitions: `1658783`
- lawful transition fraction: `1.0`
- illegal transition fraction: `0.0`
- distinct class identities reached: `34560`
- distinct spin classes reached: `16`
- distinct radial classes reached: `3`

## Core dependency audit

- rebuilt component depends explicitly on `spin_H_core_v3`: `yes`
- `sigma**` materially active in rebuilt radial logic: `yes`
- `theta` materially used: `yes`
- `rho` materially used: `yes`
- `h` materially used: `yes`

## Comparison against prior operator with older `T_r*`

Prior reference:

- `geometry_native_operator_model_v14.py`

Changes:

- state count change: `+82072`
- transition count change: `+574504`
- class diversity change: `+552`
- spin diversity change: `0`
- radial diversity change: `0`

This is a meaningful structural gain in reachable support and class diversity, while bounded spin/radial class cardinality stays unchanged.

## Honesty

Was one operator component actually rebuilt directly from `spin_H_core_v3`?
- `yes`

Was the rebuilt radial component lawful?
- `yes`

Did the rebuilt component materially use the canonical sigma parent mode identity?
- `yes`

## Next step

Run the first bounded behavior comparison using the three rebuilt components:

- `T_c`
- `T_z'`
- `T_r*`
