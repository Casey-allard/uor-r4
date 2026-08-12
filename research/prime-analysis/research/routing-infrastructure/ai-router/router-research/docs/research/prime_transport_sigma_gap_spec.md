# Prime Transport Sigma Gap Specification

## Purpose

This step isolates the exact remaining gap in:

`spin_H_core_v2 = (theta, rho, sigma*, h)`

The target is not operator work.
The target is the missing content that prevents `sigma*` from being a true
canonical regressive spin mode.

## What `sigma*` Currently Is

`sigma*` is currently:

- `mode_orbit`
- `current_residue`
- `fiber_residue`
- `radial_residue`

### `mode_orbit`

`mode_orbit` carries:

- the canonical sorted orbit built from the bounded current, fiber, and radial
  spin observables available on the present lawful surface

Concretely, it is the deduplicated bounded set formed from:

- current bounded spin word
- fiber-transported bounded spin word
- radial-transported bounded spin word

So `mode_orbit` is a canonicalized local observable orbit.
It is not yet a global regressive mode carrier.

### `current_residue`

`current_residue` carries:

- the index of the current bounded predictive observable inside `mode_orbit`

So it identifies which orbit member is currently observed as `Pi_pred`.

### `fiber_residue`

`fiber_residue` carries:

- the index of the fiber-transported bounded observable inside `mode_orbit`

So it records the local fiber transport placement of the current predictive
observation.

### `radial_residue`

`radial_residue` carries:

- the index of the radial-transported bounded observable inside `mode_orbit`

So it records the local radial transport placement of the current predictive
observation.

## What `sigma*` Still Lacks

Primary missing ingredient:

- `global_regressive_mode_index`

This is the smallest missing content that prevents `sigma*` from becoming a
true canonical regressive spin mode.

What it must carry:

- the nonlocal parent mode identity that persists across lawful generator
  composition
- not just the currently visible orbit members
- not just their local residue placements

Why this is the missing ingredient:

`sigma*` currently knows:

- which bounded words are visible locally
- how the current/fiber/radial views sit inside that local orbit

But it does not know:

- which global regressive mode this local orbit belongs to across repeated lawful
  transport and composition

So two states can share the same bounded orbit-plus-residue presentation while
still lacking an explicit parent mode identity above that presentation.

## Why `sigma*` Is Still Bounded

The boundedness comes mainly from:

- bounded predictive observables feeding sigma
- bounded orbit construction
- missing recursive/global mode carrier

The decisive cause is the third item:

- missing recursive/global mode carrier

Reason:

Even though the inputs are bounded observables, the real weakness is not merely
that they are short.
The real weakness is that `sigma*` is formed by canonicalizing those local
observables after the fact, rather than being driven by a parent regressive mode
that generates them as projections.

## Why `sigma*` Is Better Than `spin_h4` But Still Not Canonical

`sigma*` is better than `spin_h4` because:

- `spin_h4` is one bounded predictive observable
- `sigma*` organizes multiple lawful transport views of that observable into one
  parent bounded mode presentation

But `sigma*` is still not full canonical sigma because:

- it stores a canonicalized local orbit of observables
- it does not store the nonlocal regressive mode identity that those observables
  come from

So `sigma*` is a bounded mode presentation.
It is not yet the canonical regressive mode itself.

## Next Sigma Primitive

Next primitive to build:

- `global_regressive_mode_index`

This is the smallest direct refinement because it adds the missing parent mode
identity without adding another shell or reverting to observable-first state.

## Intended Update-Law Role

The missing primitive should interact with lawful updates as follows:

- `I`
  - preserve `global_regressive_mode_index`
- `T_b`
  - preserve `global_regressive_mode_index`
- `T_x`
  - update `global_regressive_mode_index` by the lawful swap action on the
    canonical regressive mode
- `T_c`
  - update `global_regressive_mode_index` by the lawful coupled transport action
    on the canonical regressive mode
- `T_y`
  - update `global_regressive_mode_index` by the lawful twist action on the
    canonical regressive mode
- `T_z'`
  - update `global_regressive_mode_index` by the lawful lift action on the
    canonical regressive mode
- `T_r*`
  - update `global_regressive_mode_index` by the lawful radial-unfolding action
    on the canonical regressive mode

Operationally:

- local bounded orbit members should be derived from the updated global mode
  index
- residues should become placements of projections from that parent mode
- not the defining data from which the parent mode is inferred

## Honesty

Is `sigma*` already a full canonical regressive spin mode?

`no`

What exact thing is `sigma*` still missing?

`sigma*` is still missing a global regressive mode index that carries the nonlocal parent mode identity across lawful generator composition.`

What is `sigma*` currently instead?

`sigma*` is currently a bounded canonicalized orbit-plus-residue presentation of local predictive observables rather than the global regressive spin mode itself.`

## Single Next Step

Implement `global_regressive_mode_index` inside the sigma slot so that
`sigma*` becomes a parent regressive mode carrier first and the bounded orbit
and `spin_h4` become derived projections from it.
