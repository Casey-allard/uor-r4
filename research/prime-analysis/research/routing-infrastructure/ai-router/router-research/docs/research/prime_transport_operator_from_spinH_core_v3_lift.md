# Prime Transport Operator From spin_H Core v3 Lift

## Purpose

This step rebuilds exactly one more operator component directly from:

- `spin_H_core_v3 = (theta, rho, sigma**, h)`

The rebuilt component is:

- `T_z'`

No new generators are added.
No learning is run.
No other operator component is rebuilt in this step.

## Rebuilt Component

### Target chosen

The rebuilt component is:

- `fiber_phase_lift_spin_transport`

This is the preferred next target because it is the lawful lift and is tightly
tied to:

- cross-class structure
- spin transport
- phase identity
- recursive closure

### What changed

The old `T_z'` inherited its action from earlier bounded lift surrogates.

The new `T_z'` now reads:

- `theta`
- `rho`
- `sigma**`
- `h`

from `active_transport_lift_core_v3(state)`.

In the rebuilt law:

- `theta` contributes to the lift phase-step
- `rho` contributes radial class, radial direction, target phi, and unfolding
  load
- `sigma**` contributes:
  - `global_regressive_mode_index.generator_projection_profile`
  - `global_regressive_mode_index.generator_orbit_profile`
  - derived mode orbit
- `h` contributes recursive and holonomy structure to the lift tau update

So the rebuilt lift is explicitly derived from the canonical parent transport
state rather than the older lift surrogate pieces.

## Rebuilt T_z' Law

Mechanical rule:

1. lift source operator state into `spin_H_core_v3`
2. read the lift projection and lift successor orbit from
   `sigma**.global_regressive_mode_index`
3. compute a lift phase step from:
   - `theta`
   - `rho`
   - sigma residues / orbit size
   - recursive / holonomy content in `h`
4. set:
   - `b' = b`
   - `phi'` by the lift phase step
   - `r' = r`
   - `spin_h'` to the lift projection word
   - `tau'` by an explicit lift update using `h`, `rho`, and the lift orbit size

This keeps the rebuild bounded and lawful while making `sigma**` materially
active inside the lift law.

## Lawfulness

The rebuilt component remains lawful by construction.

`_is_lawful` for `fiber_phase_lift_spin_transport` now checks exact equality
against the explicit `spin_H_core_v3`-derived target and preserves:

- operator support only
- fixed `b`
- fixed `r`
- composite compatibility
- fixed horizon

No illegal transitions are introduced.

## Bounded Structural Audit

On the same bounded lawful surface:

- reachable state count: `154897`
- total nonzero transitions: `1084279`
- lawful transition fraction: `100%`
- distinct class identities reached: `34008`
- distinct spin classes reached: `16`
- distinct radial classes reached: `3`

Material core usage checks:

- rebuilt component depends explicitly on `spin_H_core_v3`: `yes`
- `sigma**` materially active: `yes`
- `theta` materially used: `yes`
- `rho` materially used: `yes`
- `h` materially used: `yes`

Comparison vs prior operator with older `T_z'`:

- state count change: `+37393`
- transition count change: `+261751`
- class diversity change: `+653`
- spin diversity change: `0`
- radial diversity change: `0`

This step does not run a bounded task benchmark, to stay within the requested
single-component operator rebuild scope.

## Honesty

Was one operator component actually rebuilt directly from `spin_H_core_v3`?

`yes`

Was the rebuilt component lawful?

`yes`

Did the rebuilt component materially use the canonical sigma parent mode identity?

`yes`

## Single Next Step

If this structural lift rebuild is sufficient, rebuild `T_r*` next from
`spin_H_core_v3`; otherwise identify the next missing ingredient of full exact
`spin_H` before rebuilding more operators.
