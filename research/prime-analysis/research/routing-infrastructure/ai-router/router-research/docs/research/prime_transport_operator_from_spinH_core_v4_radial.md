# Prime Transport Operator From spin_H_core_v4 Radial

## Scope

This step rebuilds exactly one operator component:

- `T_r*`

The rebuilt component is derived directly from:

- `spin_H_core_v4 = (theta, rho, sigma_direct, h)`

No other operator component is rebuilt in this step.
No benchmark or behavior comparison is run in this step.

## Rebuilt Component

The rebuilt radial component is:

- `radial_transport_component_v18`

It uses:

- `theta`
  - for radial phase transport and spin injection
- `rho`
  - for radial target, radial target phase, radial direction, and unfolding load
- `sigma_direct`
  - via `sigma_update_v4(..., "radial_transport_unfolding")`
  - reading `current_mode`, `fiber_mode`, `radial_mode`, and `regressive_phase`
- `h`
  - for recursive and holonomy-aware tau transport

So the radial operator now depends on the direct sigma law rather than on the
old profile-encoded sigma carrier.

## Bounded Structural Results

On the same bounded lawful surface:

- reachable state count: `222618`
- total nonzero transitions: `1558326`
- lawful transition fraction: `1.0`
- illegal transition fraction: `0.0`
- distinct class identities reached: `34560`
- distinct spin classes reached: `16`
- distinct radial classes reached: `3`

Core-usage checks:

- rebuilt component depends explicitly on `spin_H_core_v4`: `yes`
- `sigma_direct` materially active: `yes`
- `theta` materially used: `yes`
- `rho` materially used: `yes`
- `h` materially used: `yes`

## Comparison Against Prior `T_r*` Rebuild From `spin_H_core_v3`

Reference:

- `geometry_native_operator_model_v15.py`

Changes:

- state count change: `-14351`
- transition count change: `-100457`
- class diversity change: `0`
- spin diversity change: `0`
- radial diversity change: `0`

So the rebuilt `T_r*` stays lawful and sigma-direct, but it does not show
meaningful structural gain over the prior radial rebuild from `spin_H_core_v3`.

## Honesty

Was one operator component rebuilt directly from `spin_H_core_v4`?
- `yes`

Was it lawful?
- `yes`

Did it materially use direct regressive sigma rather than the older profile
carrier?
- `yes`

## Next Step

Identify the next missing ingredient of the direct sigma law before any
behavior run.
