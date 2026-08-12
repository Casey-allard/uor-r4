# Prime Transport: Canonical vs Downstream

## Canonical

The canonical project object is the exact recursive admissibility system:

- transport orbit `n(j) = 5 + 6j`
- odometer update `j -> j + 1`
- exact state `(b, phi, r)`
- CRT-glued local stencils
- recursive affine lift law
- fiber evolution by direct CRT product
- symbolic admissibility dynamics
- predictive compression `(b, spin_H)`
- delayed visibility

## Downstream

The following are downstream and should not be treated as primary in current
exact-layer work:

- quotient coordinates
- empirical low-dimensional transport laws
- packet-family compression proposals
- any geometry-first explanatory framing

## Practical Rule

When a new task is ambiguous, default to:

1. exact recursive system
2. predictive compression `(b, spin_H)`
3. delayed-visibility thresholds
4. only then downstream approximation

This ordering is the intended continuity guard against research drift.
