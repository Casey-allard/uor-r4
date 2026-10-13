# Prime Transport spin_H Core v1

## Purpose

This step installs the native canonical parent transport object:

`spin_H_core_v1 = (theta, rho, sigma, h)`

The bounded patch variables are no longer treated as the transport identity.
They are now explicit projections of the parent object:

- `Pi_pred(spin_H_core_v1) -> spin_h4`
- `Pi_rec(spin_H_core_v1) -> tau`
- `Pi_hol(spin_H_core_v1) -> kappa`

## Core Object

`spin_H_core_v1` is implemented with four explicit components.

### 1. `theta`

`theta = (b, phi)`

This is the explicit angular identity component.

### 2. `rho`

`rho` carries:

- `radial_class = r`
- `unfolding_load`
- `radial_direction`
- `radial_target`
- `radial_target_phi`

This is the explicit radial/unfolding component.

### 3. `sigma`

`sigma` carries the bounded recursive spin-mode component:

- `regressive_word`
- `fiber_mode_word`
- `radial_mode_word`

`regressive_word` is the parent predictive mode from which bounded
`spin_h4` is projected.

### 4. `h`

`h` carries the recursive closure / holonomy component:

- `recursive_phase`
- `fiber_recursive_phase`
- `radial_recursive_phase`
- `holonomy_bit`

This is the parent recursive/holonomy component from which both:

- `tau`
- `kappa`

are projected.

## Projection Maps

Implemented projections:

- `Pi_pred(spin_H_core_v1) -> spin_h4`
  - `spin_h4 = sigma.regressive_word`
- `Pi_rec(spin_H_core_v1) -> tau`
  - `tau = h.recursive_phase`
- `Pi_hol(spin_H_core_v1) -> kappa`
  - `kappa = h.holonomy_bit`

These are now explicit functions in code, not informal interpretation.

## Generator Update Sketch

The parent object is still built from the existing lawful bounded generator
stack, so the update law is explicit and mechanical on the bounded surface.

Dominant component deltas on the lawful surface:

- `I`
  - `theta0 rho0 sigma0 h0`
  - no parent component changes
- `T_b`
  - `theta1 rho0 sigma0 h0`
  - base angle changes; radial, spin-mode, and closure/holonomy components are preserved
- `T_x`
  - dominant `theta0 rho0 sigma1 h1`
  - angular identity is preserved; sigma and h update
- `T_c`
  - dominant `theta1 rho1 sigma1 h1`
  - coupled move changes angular base and induces rho/sigma/h update
- `T_y`
  - dominant `theta0 rho1 sigma1 h1`
  - twist does not change angle directly, but it changes radial/unfolding, spin-mode, and closure/holonomy state
- `T_z'`
  - dominant `theta1 rho1 sigma1 h1`
  - fiber lift changes fiber angle and lawfully transports rho/sigma/h
- `T_r*`
  - dominant `theta1 rho1 sigma1 h1`
  - radial law changes the parent transport state across all nontrivial components except compatibility payload

This is not yet full exact `spin_H`. It is the smallest explicit parent object
that matches the canonical decomposition and supports the existing lawful
generator stack.

## Bounded Audit

On the same bounded lawful surface:

- primary states examined: `45`
- distinct `spin_H_core_v1` states reached: `38315`
- collision count at parent-object level: `0`
- recursive consistency rate at parent-object level: `1.0`

Projection checks:

- `spin_h4` derivable from parent: `yes`
- `tau` derivable from parent: `yes`
- `kappa` derivable from parent: `yes`

Explicit-component checks:

- `theta` explicit: `yes`
- `rho` explicit: `yes`
- `sigma` explicit: `yes`
- `h` explicit: `yes`

Comparison vs `spin_H_extended_v2`:

- collision change: `0`
- recursive consistency change: `0`

So this step does not improve the bounded audit beyond `v2`.
Its purpose is architectural:

- install the parent object
- demote the prior bounded variables to projections

## What Remains Approximate

`spin_H_core_v1` is still bounded and not full exact `spin_H`.

Weak points remain:

- `sigma` still uses bounded depth-4 predictive words
- `h` still uses bounded recursive phase summaries
- `rho` is still driven by the bounded radial law already implemented

So this is the smallest canonical parent object, not the final exact one.

## Honesty

Was a native canonical `spin_H` core object actually implemented?

`yes`

Are `spin_h4`, `tau`, and `kappa` now treated as projections of that parent object?

`yes`

Is full exact `spin_H` now present?

`no`

## Single Next Step

Rebuild one operator component directly in terms of `spin_H_core_v1`, because the canonical parent object now exists explicitly and the old bounded patches have been demoted to projections.
