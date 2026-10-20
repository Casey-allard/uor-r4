# Observables

## Definition

An **observable** is a measurable quantity computed by the UOR kernel. The class
{@class https://uor.foundation/observable/Observable} is the root of a taxonomy
that classifies measurements by what geometric aspect they capture.

## Observable Taxonomy

The observable hierarchy is organized into six categories:

| Class | Description |
|-------|-------------|
| {@class https://uor.foundation/observable/StratumObservable} | Layer position within the ring |
| {@class https://uor.foundation/observable/MetricObservable} | Geometric distance under a specific metric |
| {@class https://uor.foundation/observable/PathObservable} | Properties of paths through the ring |
| {@class https://uor.foundation/observable/ReductionObservable} | Operation sequence measurements |
| {@class https://uor.foundation/observable/CatastropheObservable} | Qualitative partition changes |
| {@class https://uor.foundation/observable/CurvatureObservable} | Gap between ring and Hamming isometry |

## Tri-Metric Classification

The three metric observables correspond to the three axes of UOR geometry:

| Metric | Axis | Description |
|--------|------|-------------|
| {@class https://uor.foundation/observable/RingMetric} | {@ind https://uor.foundation/type/verticalAxis} | d_R(x, y) = \|x - y\| mod 2^n |
| {@class https://uor.foundation/observable/HammingMetric} | {@ind https://uor.foundation/type/horizontalAxis} | Number of differing bit positions |
| {@class https://uor.foundation/observable/IncompatibilityMetric} | {@ind https://uor.foundation/type/diagonalAxis} | Divergence between ring and Hamming distances |

The {@class https://uor.foundation/observable/CurvatureObservable} measures
the gap between ring-isometry and Hamming-isometry for a given transform.
Its subclasses include {@class https://uor.foundation/observable/Commutator}
and {@class https://uor.foundation/observable/CurvatureFlux}.

## Measurement Properties

All observables share a common measurement interface:

| Property | Range | Description |
|----------|-------|-------------|
| {@prop https://uor.foundation/observable/value} | xsd:decimal | Numeric measurement value |
| {@prop https://uor.foundation/observable/hasUnit} | xsd:string | Unit of measurement |
| {@prop https://uor.foundation/observable/source} | owl:Thing | Source object of the measurement |
| {@prop https://uor.foundation/observable/target} | owl:Thing | Target object (for pairwise metrics) |

## Holonomy Observables

The {@class https://uor.foundation/observable/HolonomyObservable} category
measures path-dependent transformations:

| Class | Description |
|-------|-------------|
| {@class https://uor.foundation/observable/Monodromy} | Net transformation from traversing a loop |
| {@class https://uor.foundation/observable/ParallelTransport} | Canonical lift to the tangent bundle |
| {@class https://uor.foundation/observable/DihedralElement} | Element of D_{2^n} acting on type space |

## Role in Resolution

Observables are consumed by the resolution pipeline. The
{@class https://uor.foundation/resolver/RefinementSuggestion} uses metric
axis information ({@prop https://uor.foundation/resolver/suggestedAxis}) to
guide which constraints to apply next, informed by observable measurements.

The {@class https://uor.foundation/observable/Jacobian} is a curvature
observable that decomposes the incompatibility metric site by site — see
[Differential Calculus](differential-calculus.html) for the Jacobian's
definition and its role in curvature-weighted constraint selection (DC_10).
