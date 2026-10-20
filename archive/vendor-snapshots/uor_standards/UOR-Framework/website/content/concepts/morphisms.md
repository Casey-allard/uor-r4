# Morphisms and Transformations

The {@ns morphism} namespace defines the maps between UOR objects. Where the
kernel namespaces declare *what* objects are, and bridge namespaces compute
*how* to resolve them, morphisms specify *how objects relate to each other*
through structure-preserving transformations.

## The Transform Hierarchy

Every morphism is a {@class https://uor.foundation/morphism/Transform} — a map
with a source, a target, and a declaration of what structure it preserves.
Specialisations include:

- {@class https://uor.foundation/morphism/Isometry} — preserves metric
  distances with respect to a named metric.
- {@class https://uor.foundation/morphism/Embedding} — an injective,
  structure-preserving map across Witt levels.
- {@class https://uor.foundation/morphism/Action} — the mechanism by which a
  group applies transforms systematically to a set.

## Composition

Transforms compose. The {@class https://uor.foundation/morphism/Composition}
class records the sequential application of two or more transforms, while the
{@class https://uor.foundation/morphism/CompositionLaw} governs how operations
combine. The critical composition law — `neg compose bnot = succ` — is the
algebraic backbone of the entire framework.

## Grounding and Projection

Two special transforms connect the abstract algebra to concrete representation:

- {@class https://uor.foundation/morphism/GroundingMap} — maps a surface
  symbol to its ring datum via the content-addressing bijection.
- {@class https://uor.foundation/morphism/ProjectionMap} — maps a resolved
  partition back to an ordered sequence of surface symbols.

Together they form a round-trip: ground, resolve, then project. The
{@class https://uor.foundation/morphism/GroundingCertificate} attests that
a grounding round-trip satisfied the shared-frame condition.

## Computational Morphisms

Higher-order transforms treat computation itself as data. A
{@class https://uor.foundation/morphism/ComputationDatum} is a datum whose
ring value is the content address of a certificate; an
{@class https://uor.foundation/morphism/ApplicationMorphism} applies such a
datum to an input, and
{@class https://uor.foundation/morphism/PartialApplication} fixes some inputs
while leaving others free.

## Connection to Sites

Morphisms interact tightly with the {@concept site} decomposition. An
embedding across Witt levels maps sites in the source type to sites in the
target, and the {@concept partition} decomposition must respect the structure
that morphisms preserve.
