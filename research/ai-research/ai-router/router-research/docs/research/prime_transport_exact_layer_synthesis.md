# Prime Transport Exact-Layer Synthesis

## Purpose

This note ties together the now-canonical exact-layer results for the prime
transport line.

It is a compact synthesis of how the following objects fit together:

1. phase-fiber-scale factorization
2. finite-horizon spin as predictive compression
3. immediate internal split vs delayed visible split
4. visible-threshold law class
5. minimal routing interpretation

This note stays entirely at the exact recursive-system layer. It does not treat
quotient geometry or transport backbones as primary explanatory objects.

## Exact-Layer Stack

### 1. Phase-Fiber-Scale Factorization

The exact recursive system lives on the transport orbit

- `n(j) = 5 + 6j`

with odometer update

- `j -> j + 1`

At finite depth, the exact state factors as

- `j -> (b, phi, r)`

where:

- `b` is the base phase
- `phi` is the refinement-layer fiber state
- `r` is wheel depth

This is the exact finite-depth state description of the admissibility system.
It is built from CRT-glued local stencils and recursive affine lift, not from
any downstream approximation.

### 2. Finite-Horizon Spin as Predictive Compression

The exact symbolic admissibility word along the orbit induces the finite-horizon
spin

- `spin_H(j)`

and the compressed predictive state

- `(b(j), spin_H(j))`

This is the correct internal predictive compression object at the exact layer.
It compresses future distinguishability without leaving the recursive system.

So the conceptual flow is:

- exact state `(b, phi, r)`
- symbolic admissibility dynamics
- predictive compression `(b, spin_H)`

not:

- exact state directly into downstream quotient geometry

### 3. Internal Split vs Visible Split

Wheel lift `W -> pW` activates new refinement in two stages:

- **internal split**:
  child lifts already diverge in future evolution while still sharing the same
  current predictive state
- **visible split**:
  the number of distinct predictive states `(b, spin_H)` on the child wheel
  becomes larger than on the parent wheel

This gives the exact delayed-visibility mechanism:

- hidden refinement becomes active first
- only later does that refinement become visible as predictive-state growth

## Threshold Mechanism Flow

The current exact-layer evidence supports the following mechanism flow:

1. the recursive lift adds new local stencil structure
2. that new structure becomes internally active immediately
3. hidden divergence accumulates inside the predictive classes
4. a first visible splitting event releases that hidden divergence into a
   larger `(b, spin_H)` state count

This is why internal split is immediate while visible split can be delayed by a
long horizon.

So the visible-threshold problem is not “when does the new layer exist?”

It is:

- when does hidden refinement become predictively visible?

## Current Canonical Threshold-Law Class

The strongest current exact-layer law class is:

- `density + first-splitting event`

The pieces now fit together as follows:

- parent admissible density measures the inherited coarse return grammar
- first-splitting-event statistics measure how much unresolved predictive
  structure is released at the threshold

The current evidence shows:

- density alone is insufficient
- burden or simple arrangement alone are insufficient
- simple global interaction averages are insufficient
- first-splitting-event structure is the key additional mechanism

On the current tightly matched evidence, arrangement is secondary rather than
leading.

## Exact Layer vs Downstream Layer

### Exact Layer

Primary objects:

- transport orbit
- phase-fiber-scale state `(b, phi, r)`
- symbolic admissibility dynamics
- predictive compression `(b, spin_H)`
- internal split / visible split
- threshold-law mechanism class

### Downstream Layer

Derived or approximation objects:

- quotient coordinates
- empirical low-dimensional transport laws
- packet-based or geometry-first explanations

These downstream objects may still be useful later, but they should inherit the
exact-layer mechanism rather than replace it.

## Minimal Routing Interpretation

The exact-layer results suggest a minimal routing principle:

- routing-relevant refinement should be organized around predictive
  distinguishability, not around raw hidden refinement alone

In exact-layer terms:

- hidden structure activates immediately
- but only part of it becomes predictively visible at a given horizon
- the key routing object is therefore the first visible predictive split, not
  just the existence of deeper hidden fiber structure

So the minimal routing interpretation is:

- use `(b, spin_H)` as the exact predictive compression object
- treat delayed visibility as the criterion for when deeper refinement becomes
  operationally relevant
- treat density plus first-splitting event structure as the current exact-layer
  guide for where predictive branching pressure actually appears

This is only an interpretation of the exact-layer math. It does not change
runtime code and it does not claim a finished routing algorithm.

## Canonical Standing

Future routing-theory work should inherit this order:

1. exact recursive system
2. predictive compression `(b, spin_H)`
3. delayed visibility and visible-threshold law class
4. only then downstream quotient or transport approximations

That ordering is the current continuity guard against drifting away from the
exact recursive object.
