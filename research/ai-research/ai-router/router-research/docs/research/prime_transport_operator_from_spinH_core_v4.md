# Prime Transport Operator From spin_H_core_v4

## Scope

This step rebuilds exactly one operator component:

- `T_c`

The rebuilt component is derived directly from:

- `spin_H_core_v4 = (theta, rho, sigma_direct, h)`

No other operator component is rebuilt in this step.
No benchmark or behavior comparison is run in this step.

## Rebuilt Component

The rebuilt coupled component is:

- `coupled_torus_kick_component_v16`

It uses:

- `theta`
  - for coupled base-step selection
- `rho`
  - for radial/unfolding contribution to coupled transport
- `sigma_direct`
  - via `sigma_update_v4(..., "coupled_torus_kick")`
  - reading `current_mode`, `fiber_mode`, `radial_mode`, and `regressive_phase`
- `h`
  - for recursive/holonomy-aware tau transport

So the coupled operator now depends on the direct sigma law rather than on the
older profile carrier.

## Bounded Structural Results

On the same bounded lawful surface:

- reachable state count: `125677`
- total nonzero transitions: `879739`
- lawful transition fraction: `1.0`
- illegal transition fraction: `0.0`
- distinct class identities reached: `34231`
- distinct spin classes reached: `16`
- distinct radial classes reached: `3`

Core-usage checks:

- rebuilt component depends explicitly on `spin_H_core_v4`: `yes`
- `sigma_direct` materially active: `yes`
- `theta` materially used: `yes`
- `rho` materially used: `yes`
- `h` materially used: `yes`

## Comparison Against Prior `T_c` Rebuild From `spin_H_core_v3`

Reference:

- `geometry_native_operator_model_v13.py`

Changes:

- state count change: `+8173`
- transition count change: `+57211`
- class diversity change: `+876`
- spin diversity change: `0`
- radial diversity change: `0`

So the rebuilt `T_c` shows meaningful structural change while preserving full
lawfulness.

## Honesty

Was one operator component actually rebuilt directly from `spin_H_core_v4`?
- `yes`

Was the rebuilt component lawful?
- `yes`

Did the rebuilt component materially use direct regressive sigma rather than
the older profile carrier?
- `yes`

## Next Step

Rebuild `T_z'` next from `spin_H_core_v4`.
