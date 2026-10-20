# Canonical Form

## Definition

A **canonical form** is the unique representative of an equivalence class of
terms under the rewrite rules of the UOR framework. The
{@class https://uor.foundation/resolver/CanonicalFormResolver} computes
canonical forms by applying the critical identity and normalization rules until
no further rewrites are possible.

## The Resolver

The {@class https://uor.foundation/resolver/CanonicalFormResolver} is a subclass
of {@class https://uor.foundation/resolver/Resolver}. It reduces terms to their
unique canonical representatives using the critical identity as the primary
rewrite rule:

| Property | Description |
|----------|-------------|
| {@prop https://uor.foundation/resolver/inputType} | The type declaration to resolve |
| {@prop https://uor.foundation/resolver/outputType} | The resulting partition |
| {@prop https://uor.foundation/resolver/strategy} | Term rewriting via critical identity |

## Derivation Witness

The canonical form computation is witnessed by a
{@class https://uor.foundation/derivation/Derivation}, which records the full
rewrite sequence:

| Property | Range | Description |
|----------|-------|-------------|
| {@prop https://uor.foundation/derivation/originalTerm} | Term | The term before rewriting |
| {@prop https://uor.foundation/derivation/canonicalTerm} | Term | The canonical form |
| {@prop https://uor.foundation/derivation/result} | Datum | The evaluated datum value |
| {@prop https://uor.foundation/derivation/step} | RewriteStep | Individual rewrite steps |

Each {@class https://uor.foundation/derivation/RewriteStep} records a single
rule application via {@prop https://uor.foundation/derivation/hasRewriteRule}:

```turtle
<https://uor.foundation/instance/step-1>
    a                   derivation:RewriteStep ;
    derivation:from     <term-neg-bnot-x> ;
    derivation:to       <term-succ-x> ;
    derivation:hasRewriteRule  derivation:CriticalIdentityRule .
```

## Term Metrics

The {@class https://uor.foundation/derivation/TermMetrics} class tracks the
size and complexity of canonical forms:

| Property | Description |
|----------|-------------|
| {@prop https://uor.foundation/derivation/stepCount} | Total rewrite steps performed |
| {@prop https://uor.foundation/derivation/termSize} | Nodes in the canonical term's syntax tree |

## Iterative Refinement

With Amendment 11, canonical form computation can also proceed iteratively via
{@class https://uor.foundation/derivation/RefinementStep}. Each refinement
step applies a {@class https://uor.foundation/type/Constraint} and records how
many sites it closes ({@prop https://uor.foundation/derivation/sitesClosed}).

## Relation to Content Addressing

The canonical form is the foundation of content addressing: the
{@class https://uor.foundation/u/Element} of an object is derived from its
canonical form. Two objects with the same canonical form receive the same
address. See [Addressing](addressing.html) for the Braille-encoded glyph
scheme that maps canonical forms to content-addressable identifiers.
