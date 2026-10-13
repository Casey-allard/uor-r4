# Prime Transport Operator From spin_H_core_v4 Lift

## Scope

This step rebuilds exactly one operator component:

- `T_z'`

The rebuilt component is derived directly from:

- `spin_H_core_v4 = (theta, rho, sigma_direct, h)`

No other operator component is rebuilt in this step.
No benchmark or behavior comparison is run in this step.

## Rebuilt Component

The rebuilt lift component is:

- `fiber_phase_lift_component_v17`

It uses:

- `theta`
  - for lift phase-step selection
- `rho`
  - for radial/unfolding contribution to lift transport
- `sigma_direct`
  - via `sigma_update_v4(..., "fiber_phase_lift_spin_transport")`
  - reading `current_mode`, `fiber_mode`, `radial_mode`, and `regressive_phase`
- `h`
  - for recursive and holonomy-aware tau transport

So the lift operator now depends on the direct sigma law rather than on the old
profile-encoded sigma carrier.

## Bounded Structural Results

On the same bounded lawful surface:

- reachable state count: `171832`
- total nonzero transitions: `1202824`
- lawful transition fraction: `1.0`
- illegal transition fraction: `0.0`
- distinct class identities reached: `34514`
- distinct spin classes reached: `16`
- distinct radial classes reached: `3`

Core-usage checks:

- rebuilt component depends explicitly on `spin_H_core_v4`: `yes`
- `sigma_direct` materially active: `yes`
- `theta` materially used: `yes`
- `rho` materially used: `yes`
- `h` materially used: `yes`

## Comparison Against Prior `T_z'` Rebuild From `spin_H_core_v3`

Reference:

- `geometry_native_operator_model_v14.py`

Changes:

- state count change: `+16935`
- transition count change: `+118545`
- class diversity change: `+506`
- spin diversity change: `0`
- radial diversity change: `0`

So the rebuilt `T_z'` shows meaningful structural change while preserving full
lawfulness.

## Honesty

Was one operator component rebuilt directly from `spin_H_core_v4`?
- `yes`

Was the rebuilt component lawful?
- `yes`

Did it materially use direct regressive sigma rather than the older profile
carrier?
- `yes`

## Next Step

Rebuild `T_r*` next from `spin_H_core_v4`.
