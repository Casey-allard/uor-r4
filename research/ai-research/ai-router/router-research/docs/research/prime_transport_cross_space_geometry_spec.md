# Prime Transport Cross-Space Geometry Specification

## Purpose

This document freezes the intended correspondence between:

- the primary torus space
- the transformed transport subspace
- angular identity
- radial / depth / unfolding coordinate
- spin identity

Its role is to make future operator algebra derive from the geometry rather
than from local lawful-generator invention.

## 1. Primary Torus Coordinates

The exact primary orbit is:

- `j -> j + 1`

with exact finite-depth chart:

- `j -> (b, phi, r)`

The primary torus coordinates are:

- `b`
  - base phase
  - discrete angular coordinate on the coarse torus
- `phi`
  - refinement fiber phase
  - discrete fiber coordinate over the same base angle
- `r`
  - wheel depth / radial class
  - scale layer of the exact chart, not a torus coordinate itself

Current operator-line correspondence:

- `b` is present directly
- `phi` is present directly
- `r` is present directly
- `twist` is not a primary torus coordinate
- `query_semiprime`, `binding_semiprime`, and `composite_compat_class` are
  coupled payload state attached to the chart, not primary torus coordinates

So the primary space is best described as:

- base torus angle `b`
- with fiber refinement coordinate `phi`
- stratified by radial layer `r`

## 2. Transport Subspace Coordinates

The transformed transport subspace is not the same object as the static primary
chart.

The best exact source-of-truth transport coordinate currently available is:

- `(b, spin_H)`

This is not purely polar and not purely toroidal. It is a hybrid chart:

- `b`
  - stable angular anchor inherited from the primary torus
- `spin_H`
  - finite-horizon predictive transport word
  - dynamical transport coordinate

For the implemented bounded line, the available shadow is:

- `(b, spin_h, composite_compat_class)`

where:

- `spin_h` is an explicit truncation of `spin_H`
- `composite_compat_class` is the coupled transport-compatibility tag carried
  by the composite pair

Current variables that are only proxies for the transformed transport space:

- `spin_h`
  - truncation / shadow of full `spin_H`
- `next_return_gap`
  - small residual proxy for transport grammar, not the transformed space
- `twist`
  - relational auxiliary coordinate, not part of the exact transport chart
- `phi`
  - exact primary fiber coordinate, but not itself the transport subspace

Missing variables that prevent full realization:

- full native `spin_H`
- explicit primary-to-transport lift map
- explicit transport-space radial / unfolding coordinate beyond static `r`

## 3. Angular Correspondence Law

This is the central cross-space identity rule.

### 3.1 Same-angle definition

Two states have the same cross-space angle if and only if they share:

- the same base phase `b`

This is the stable pointing identity across both spaces.

### 3.2 Refined angular address

There is also a stricter refined angular address:

- `(b, phi)`

This is not the primary cross-space angle itself. It is the base angle plus
fiber refinement inside the same angular sector.

So:

- same base angle:
  - same `b`
- same refined angle:
  - same `(b, phi)`

### 3.3 What angle preservation means

Under lawful cross-space correspondence, preserving angle means preserving:

- `b`

The following may change while angle remains the same:

- `phi`
- `r`
- `spin_H` or `spin_h`
- composite compatibility state

provided the change occurs through a named lawful lift or transport map.

### 3.4 Angular type

Angle is:

- discrete
- modular
- fiber-refined

It is not a continuous Euclidean angle in the current architecture.

## 4. Radial / Depth / Unfolding Law

### 4.1 Primary-space radial meaning

In the primary chart:

- `radial_class = r`

This means:

- wheel depth
- multiplicative layering
- exact scale level in the recursive admissibility chart

### 4.2 Why radial is more than static radius

In the intended architecture, radial coordinate also carries unfolding meaning.

Reason:

- visible splitting is delayed relative to internal splitting
- deeper layers expose later predictive branch distinctions

So radial coordinate is not only geometric depth. It is also:

- depth of predictive unfolding

### 4.3 Transport-space radial meaning

In the transformed transport subspace, radial meaning is not fully captured by
static `r` alone.

The transport-space radial notion should include:

- primary wheel depth `r`
- plus how much future admissibility structure is unfolded in `spin_H`

So the current `r` is:

- exact as a primary radial class
- only a weak proxy for transformed radial / unfolding depth

### 4.4 Consequence

A faithful radial transport operator cannot be derived from `r` alone.
It needs either:

- full `spin_H`
- or an explicit lawful unfolding map that states how transport-space depth
  changes while preserving cross-space angle

## 5. Spin Identity Law

### 5.1 What spin represents

Spin is the predictive transport identity attached to an orbit point.

Operationally:

- `spin_H(j)` = length-`H` future admissibility word along the exact orbit

So spin is not an arbitrary tag. It is:

- finite-horizon future transport grammar

### 5.2 Relation to angle and radial structure

Spin refines the static chart.

Given the same:

- `b`
- `phi`
- `r`

different `spin_H` values represent different dynamical branch identities.

So spin is:

- not the angle
- not the radial coordinate
- but the dynamical identity layered on top of them

### 5.3 Current implementation status

Current `spin_h` is:

- an explicit truncation
- a projection / shadow of intended `spin_H`

It is not the full intended spin identity.

### 5.4 What full `spin_H` carries that current `spin_h` does not

Full `spin_H` should carry:

- longer predictive branch distinctions
- deeper return-grammar structure
- delayed visible splitting information beyond the truncated horizon
- more faithful transport-space unfolding depth

Current truncated `spin_h` discards all information beyond the chosen horizon.

## 6. Comparison / Transport Law

### 6.1 Direct comparison

Two states may be directly compared only if they match on:

- `radial_class = r`
- `fiber_class = phi`
- declared spin horizon
- `spin_class = spin_H or spin_h`
- `composite_compat_class`

### 6.2 Comparison after lawful lift

Comparison is lawful after normalization only through an explicit named lift:

- `radial_lift`
- `fiber_refinement_lift`
- `spin_truncation_lift`
- `compatibility_normalization_lift`

Each lawful lift must preserve the invariants frozen in
[prime_transport_framework_lock.md](/Users/adminamn/AI-Research/ai-router/router-research/docs/research/prime_transport_framework_lock.md).

### 6.3 Forbidden comparison

Comparison is forbidden if class components differ and no named lift is
declared.

Forbidden means:

- do not score
- do not rank
- do not pool together

### 6.4 Within-class transport

Transport is within-class when it preserves:

- `r`
- `phi`
- spin class at the declared horizon
- `composite_compat_class`

### 6.5 Cross-class transport

Transport is cross-class only when the change is licensed by a named
cross-space law.

At present the only partially instantiated one is:

- fiber lift with spin transport

The missing ones are:

- explicit radial lift
- explicit primary-to-transport lift

## 7. Operator Derivation Guidance

Existing generators should be classified as follows.

### `I`

- status: geometry-derived
- reason:
  - identity component exists in any lawful discrete operator algebra

### `T_b`

- status: geometry-derived
- reason:
  - direct shadow of exact odometer base advance on the primary torus

### `T_x`

- status: proxy-derived
- reason:
  - composite swap reflects a bounded symmetry of the coupled payload pair, but
    no explicit cross-space geometric law currently derives it

### `T_c`

- status: ad hoc / under-specified
- reason:
  - coupled torus kick depends on both composites, but there is no explicit
    cross-space geometric law from which the kick is derived

### `T_y`

- status: ad hoc / under-specified
- reason:
  - composite twist introduces a relational axis, but that axis is not yet part
    of the frozen primary-to-transport geometry

### `T_z'`

- status: proxy-derived
- reason:
  - it is the first explicit lawful lift, but it still uses truncated `spin_h`
    and a locally declared transport map rather than a full primary-to-subspace
    geometric lift

## 8. Single Most Important Missing Primitive

The single most important missing primitive is:

- explicit primary-to-transport lift map

Definition:

- a typed map sending exact primary chart state
  - `(b, phi, r)`
  - plus coupled composite payload
  - into transformed transport coordinates
  - `(b, spin_H)` or explicit declared truncation thereof

Why this is first:

- it is the missing object that makes operator entries consequences of the
  geometry rather than local lawful constructions
- without it, fiber lift, spin transport, and future radial transport remain
  only partially grounded

## Required Honesty Section

### Have recent operator branches been deriving motion from the full cross-space geometry?

no

### What exact part of the intended geometry is still missing from implementation?

The implementation still lacks an explicit primary-to-transport lift map that
turns exact chart coordinates `(b, phi, r)` into transformed transport
coordinates `(b, spin_H)` with declared preserved invariants.

## Single Next Implementation Step After Geometry Spec

Implement one typed `primary_to_transport_lift` module that takes native
primary-chart state `(b, phi, r)` plus composite payload and returns explicit
transport-space coordinates with declared preserved invariants, before adding
any further operator or weighting changes.
