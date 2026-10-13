# Prime Transport spin_H Canonical Specification

## Purpose

This step canonicalizes the transport identity.

The current bounded transport-side state was assembled incrementally:

- `spin_h4`
- then `tau`
- then `kappa`
- then extended shells around them

That is useful for bounded closure, but architecturally backward.

The correct direction is:

- define canonical regressive `spin_H`
- treat `spin_h4`, `tau`, and `kappa` as projections or bounded observables of it

This document defines that parent object.

## 1. What `spin_h4` Actually Is

`spin_h4` currently measures:

- the first four bits of bounded future admissibility behavior attached to the current orbit point

Operationally:

- it is a depth-4 predictive observation
- it reflects only a short bounded horizon of future transport grammar

So `spin_h4` is not the true spin state because:

- it only sees a finite local prefix
- it does not by itself determine recursive closure
- it does not by itself determine holonomy
- it is one bounded observable of transport behavior, not the full transport object

## 2. What `tau` Actually Is

`tau` currently carries:

- recursive stabilizer-path residue needed to keep bounded transport identity canonical under lawful iteration

Operationally it records:

- swap-phase residue
- coupled-phase residue
- twist-phase residue
- lift-phase residue

Why `tau` should be derived from canonical `spin_H`:

- it is not a separate physical coordinate of the transport chart
- it is recursive phase information of the same transport identity
- it exists only because the bounded predictive observation did not absorb recursive closure internally

So `tau` is a recursive projection of canonical transport state, not a peer state object beside it.

## 3. What `kappa` Actually Is

`kappa` currently carries:

- the bounded transport holonomy bit
- in the current lawful implementation, exactly hidden twist parity

Why `kappa` should be derived from canonical `spin_H`:

- it is not an independent transport chart axis
- it is the residual holonomy class of the same recursive transport identity
- it appears only because the current bounded object did not internalize that holonomy in its parent state

So `kappa` is a holonomy projection of canonical `spin_H`, not a foundational patch variable.

## 4. Smallest Canonical Regressive `spin_H`

The smallest canonical parent transport object should be:

`spin_H = (theta, rho, sigma, h)`

with the following components.

### 4.1 Angular identity component: `theta`

`theta` carries:

- base angle `b`
- fiber refinement `phi`

So:

- `theta = (b, phi)`

This is the canonical angular address in transport space.

### 4.2 Radial / unfolding component: `rho`

`rho` carries:

- radial class `r`
- unfolding mode / depth state beyond static radius

So:

- `rho` is not just `r`
- `rho` is the radial-unfolding mode of the transport chart

This is the parent object from which bounded radial target/load summaries should be derived.

### 4.3 Regressive spin mode component: `sigma`

`sigma` carries:

- the canonical recursive transport mode
- the parent predictive state from which bounded future admissibility observations are projected

This is the actual regressive spin state.

It must be:

- recursively updated under lawful operator action
- rich enough to generate bounded predictive words such as `spin_h4`
- not itself just a bounded future prefix

### 4.4 Holonomy / closure component: `h`

`h` carries:

- recursive closure phase
- holonomy/twist parity
- stabilizer-path residue needed to make transport identity canonical

This is the parent object from which both:

- recursive closure summaries
- holonomy bits

should be projected.

### 4.5 Canonical form

So the smallest canonical regressive transport object is:

`spin_H = ((b, phi), rho, sigma, h)`

This is the smallest parent state that explicitly accounts for:

- angular identity
- radial/unfolding identity
- recursive closure
- holonomy/twist parity
- predictive observables as derived projections

## 5. Projection Relationships

The current bounded objects should be projections of canonical `spin_H`.

### 5.1 Predictive projection

`Pi_pred(spin_H) -> spin_h4`

Meaning:

- read the first bounded predictive observation emitted by the canonical regressive spin mode `sigma`

So `spin_h4` is:

- a depth-4 predictive observable of `sigma`

### 5.2 Recursive-closure projection

`Pi_rec(spin_H) -> tau`

Meaning:

- read the bounded recursive phase coordinates of the canonical holonomy/closure component `h`

So `tau` is:

- a bounded recursive closure summary of `h`

### 5.3 Holonomy projection

`Pi_hol(spin_H) -> kappa`

Meaning:

- read the parity/bit-level holonomy class of `h`

So `kappa` is:

- a one-bit holonomy observable of the canonical closure component

### 5.4 Radial projection

`Pi_rad(spin_H) -> r`

Meaning:

- read the bounded radial class from the radial-unfolding component `rho`

This makes clear that plain `r` is also a projection, not the full radial transport state.

## 6. Generator Law vs State

The operator/generator laws are things like:

- hold
- base advance
- swap
- coupled move
- twist
- fiber lift
- radial transport

These are update laws.

They are not the spin state itself.

Why this matters:

- a generator law tells how transport state changes
- canonical `spin_H` is the state being changed
- bounded observables such as `spin_h4`, `tau`, and `kappa` are read from that state

So:

- the generator algebra is not the identity
- the projections are not the identity
- canonical `spin_H` is the recursively updated parent transport state the operator acts on

## 7. Single Next Implementation Primitive

The single next primitive to build is:

- canonical `spin_H` core state object

Reason:

- it is the smallest code step that begins replacing the patched bounded identity with the parent object plus projection maps
- once the core object exists, `spin_h4`, `tau`, and `kappa` can be recast as explicit projections instead of treated as foundational state

## Honesty

Is `spin_h4` the canonical regressive spin state?

`no`

What is `spin_h4` actually?

`spin_h4` is a bounded four-step predictive observable of the canonical recursive transport mode, not the transport mode itself.`

What should be the parent object instead?

`The parent object should be canonical regressive spin_H = ((b, phi), rho, sigma, h), whose predictive, recursive, and holonomy summaries are projected as spin_h4, tau, and kappa.`

## Single Next Step

Implement the canonical `spin_H` core state object as native transport-side state, with explicit projection maps to `spin_h4`, `tau`, and `kappa`.
