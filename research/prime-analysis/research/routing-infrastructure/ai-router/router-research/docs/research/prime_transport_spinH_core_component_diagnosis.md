# Prime Transport spin_H Core Component Diagnosis

## Purpose

This step performs a component-level diagnosis of:

`spin_H_core_v1 = (theta, rho, sigma, h)`

The task is to identify the single weakest bounded surrogate inside the
canonical parent object and name the next refinement primitive.

No weighting, benchmarking, or operator expansion is performed here.

## Component Role Audit

### 1. `theta`

Intended role:

- carry angular identity
- fix the cross-space pointing coordinate

Current content:

- `theta = (b, phi)`

Assessment:

- explicit
- structurally correct
- likely already canonical on the bounded surface

Reason:

`theta` is not acting as a summary patch. It carries the declared angular chart
coordinates directly and does not depend on predictive truncation or recursive
repair variables.

### 2. `rho`

Intended role:

- carry radial / unfolding identity
- encode radial target structure under lawful transport

Current content:

- `radial_class`
- `unfolding_load`
- `radial_direction`
- `radial_target`
- `radial_target_phi`

Assessment:

- structurally meaningful
- still bounded / surrogate
- not the weakest current component

Reason:

`rho` is still driven by the bounded radial law and unfolding summary already
implemented, but it is at least organized as a radial transport object rather
than a short predictive word. Its boundedness is secondary to the weaker
predictive construction inside `sigma`.

### 3. `sigma`

Intended role:

- carry the regressive recursive spin mode
- be the parent spin-side mode from which predictive observables are projected

Current content:

- `regressive_word`
- `fiber_mode_word`
- `radial_mode_word`

Assessment:

- explicit
- still bounded
- still surrogate
- weakest current component

Reason:

All three fields are local bounded words. `regressive_word` is exactly the
source of `spin_h4`, and the fiber/radial fields are companion bounded word
images under current transport maps. This means `sigma` is still fundamentally
a predictive-shell object, not a canonical regressive spin mode.

### 4. `h`

Intended role:

- carry recursive closure
- carry holonomy / twist residue

Current content:

- `recursive_phase`
- `fiber_recursive_phase`
- `radial_recursive_phase`
- `holonomy_bit`

Assessment:

- bounded
- surrogate
- but structurally closer to canonical closure than `sigma`

Reason:

`h` is still summarized in bounded phase form, but it already absorbs the
recursive closure and holonomy distinctions needed for bounded canonicity.
On the bounded lawful surface it closes cleanly. Its remaining limitation is
bounded depth, not misidentification of its role.

## Weakest Component

Weakest component:

- `sigma`

Why this is the next component that must be refined:

- `theta` is already direct angular state
- `rho` is bounded but organized as radial/unfolding transport structure
- `h` is bounded but already functions as the closure/holonomy carrier
- `sigma` is still built out of short mode words and therefore remains the most
  obvious predictive surrogate inside the parent object

So the main architectural weakness is not that `spin_H_core_v1` lacks a parent
shape. It is that the parent spin-mode slot is still populated by bounded local
predictive words instead of a true regressive mode.

## Sigma Test

Is `sigma` currently a true regressive mode?

- `no`

What `sigma` currently is:

- a bounded predictive surrogate assembled from the current, fiber, and radial
  transport words on the bounded surface

What exact information it is still missing:

- the non-bounded recursive spin-mode carrier whose lawful transport generates
  those observed words as projections rather than storing the words themselves

What should replace or refine it:

- a canonical regressive sigma mode that evolves under lawful transport first,
  with bounded predictive words recovered from it as observables

## Update-Law Adequacy

For the weakest component `sigma`, the current update law is:

- structurally correct but bounded

Reason:

The current generator deltas show that `sigma` responds in the expected places:

- preserved under `I`
- preserved under `T_b`
- updated under `T_x`
- updated under `T_c`
- updated under `T_y`
- updated under `T_z'`
- updated under `T_r*`

So the law is not structurally wrong. The weakness is that the updated object is
still a bounded word-level shell rather than the canonical regressive mode
itself.

## Single Next Primitive

Next primitive to implement:

- `canonical_regressive_sigma_mode`

This is the smallest code-level refinement that directly targets the weakest
surrogate inside `spin_H_core_v1` without adding more patches around it.

## Honesty

Which component of `spin_H_core_v1` is currently the weakest bounded surrogate?

`sigma`

Is `sigma` currently a true regressive spin mode?

`no`

What is `sigma` currently instead?

`sigma` is currently a bounded predictive surrogate made of current/fiber/radial mode words rather than a canonical regressive spin mode.`

## Single Next Step

Implement `canonical_regressive_sigma_mode` as the native replacement for the
current word-based `sigma` component, then derive bounded predictive words from
that mode as projections.
