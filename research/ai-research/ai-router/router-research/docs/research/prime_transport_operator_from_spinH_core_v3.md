# Prime Transport Operator From spin_H Core v3

## Purpose

This step rebuilds exactly one operator component directly from:

- `spin_H_core_v3 = (theta, rho, sigma**, h)`

The rebuilt component is:

- `T_c`

No new generators are added.
No learning is run.
No other operator component is rebuilt in this step.

## Rebuilt Component

### Target chosen

The rebuilt component is:

- `coupled_torus_kick`

This is the smallest honest core-v3 rebuild because it is the most coupled
operator component and the most natural place to test whether the new sigma
parent mode identity is actually being used.

### What changed

The old `T_c` inherited its action from earlier bounded surrogate pieces.

The new `T_c` now reads:

- `theta`
- `rho`
- `sigma**`
- `h`

from `active_transport_lift_core_v3(state)`.

In the rebuilt law:

- `theta` contributes to coupled base-step selection
- `rho` contributes radial/unfolding load and target structure
- `sigma**` contributes:
  - `global_regressive_mode_index.generator_projection_profile`
  - `global_regressive_mode_index.generator_orbit_profile`
  - derived mode orbit
- `h` contributes recursive and holonomy structure to the tau update

So the rebuilt component is explicitly derived from the canonical parent
transport state rather than the older local surrogate pieces.

## Rebuilt T_c Law

Mechanical rule:

1. lift source operator state into `spin_H_core_v3`
2. read the coupled successor projection and coupled successor orbit from
   `sigma**.global_regressive_mode_index`
3. compute a coupled base step from:
   - `theta`
   - `rho`
   - sigma residues / orbit size
   - recursive / holonomy content in `h`
4. set:
   - `b'` by the coupled base step
   - `phi' = phi`
   - `r' = r`
   - `spin_h'` to the coupled projection word
   - `tau'` by an explicit coupled update using `h` and the coupled orbit size

This keeps the rebuild bounded and lawful while making `sigma**` materially
active.

## Lawfulness

The rebuilt component remains lawful by construction.

`_is_lawful` for `coupled_torus_kick` now checks exact equality against the
explicit `spin_H_core_v3`-derived target and preserves:

- operator support only
- composite compatibility
- fixed horizon

No illegal transitions are introduced.

## Bounded Structural Audit

On the same bounded lawful surface:

- reachable state count: `117504`
- total nonzero transitions: `822528`
- lawful transition fraction: `100%`

Material core usage checks:

- rebuilt component depends explicitly on `spin_H_core_v3`: `yes`
- `sigma**` materially active: `yes`
- `theta` materially used: `yes`
- `rho` materially used: `yes`
- `h` materially used: `yes`

Comparison vs prior operator:

- state count change vs prior operator with older `T_c`: `+79189`
- transition count change vs prior operator with older `T_c`: `+554323`
- class diversity change: `+3454`
- spin diversity change: `0`

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

Rebuild the next most mode-dependent operator component from `spin_H_core_v3`
if this structural gain is sufficient; otherwise identify the next missing
ingredient of full exact `spin_H` before rebuilding more operators.
