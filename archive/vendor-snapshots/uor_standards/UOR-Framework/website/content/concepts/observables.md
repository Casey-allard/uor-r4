# Observables & Measurement

The {@ns observable} namespace defines what can be measured during resolution.
Observables are bridge-space objects that quantify geometric, topological, and
algebraic properties of ring elements and their types.

## What Is an Observable?

An {@class https://uor.foundation/observable/Observable} is a measurable quantity
attached to a computation. During {@concept resolution}, the resolver computes not
just the answer to a query but also a collection of observable measurements:
ring distance, Hamming distance, curvature, spectral data, and more.

These measurements serve two purposes:

1. **Diagnostic** -- they reveal the geometric structure of the computation,
   helping identify obstructions or convergence properties.
2. **Evidential** -- they provide the quantitative evidence that proofs and
   certificates reference when attesting to a result.

## Metric Axes

The {@class https://uor.foundation/observable/MetricAxis} enum class defines the
named measurement dimensions. Each axis represents a different way of measuring
distance or similarity in the ring:

- Ring distance (arithmetic difference modulo 2^n)
- Hamming distance (bitwise difference count)
- Curvature (local geometric bending)

These axes parameterize the multi-metric structure of UOR -- the same pair of
elements can have different distances depending on which metric axis is used.

## Holonomy and Monodromy

When a {@concept site} is transported around a loop in the base space, it may
return to a different state. This transformation is the **holonomy**.

The {@class https://uor.foundation/observable/HolonomyGroup} captures the group
of all holonomy transformations at a base point. The
{@class https://uor.foundation/observable/Monodromy} class records the
representation of the fundamental group in the holonomy group.

If all holonomies are trivial, the bundle is flat. Otherwise, it is twisted --
a distinction formalized by `type:FlatType` and `type:TwistedType` in the
{@ns type} namespace.

## Measurement and State Collapse

The {@class https://uor.foundation/trace/MeasurementEvent} class records when an
observable is measured during resolution. For superposed site states (where a
site is a quantum superposition of types), measurement collapses the
superposition into a definite type.

The {@class https://uor.foundation/observable/AchievabilityStatus} enum classifies
targets as either Achievable or Forbidden -- determining whether a morphospace
target is reachable from the current state.

## Connection to the PRISM Pipeline

Observables are computed during the Resolve stage by the {@concept resolution}
machinery. They produce the quantitative evidence that feeds into the
{@concept proof-system} certification pathway. The topological observables
(holonomy, monodromy, spectral sequences) connect directly to the
{@concept homology} layer that diagnoses convergence obstructions.

See also {@concept witt-levels} for how observables are scoped to specific
ring scales.
