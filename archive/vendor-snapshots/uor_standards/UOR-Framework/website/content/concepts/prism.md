# The PRISM Pipeline

PRISM is the three-stage operational pipeline of the UOR Foundation ontology:
**Define → Resolve → Certify**. Each stage corresponds to a space in the three-space
hierarchy, and together they form a complete circuit from raw structure to certified truth.

## Overview

Every computable identity in UOR travels the PRISM pipeline. A claim about the
{@concept ring} is first defined in the Kernel, then resolved by the Bridge, and
finally certified in the certification layer. The pipeline is not a metaphor: it is
implemented in the {@count:namespaces} namespaces assembled in dependency order, with
the {@ns cert} namespace sitting at the apex.

The name PRISM captures the dual role of the pipeline: it refracts abstract structure into
computable forms (the dispersive function of a prism), and it certifies identities with
cryptographic precision (the certification function).

## Stage 1 — Define (Kernel)

The Kernel space comprises the three foundational namespaces: {@ns u}, {@ns schema},
and {@ns op}.

- **{@ns u}**: The {@class https://uor.foundation/u/UniversalAddress} class establishes the
  address space. The {@class https://uor.foundation/u/ContentAddressed} and
  {@class https://uor.foundation/u/Identifiable} classes ground every UOR object in a
  content-addressable location. See {@concept content-addressing}.

- **{@ns schema}**: The {@class https://uor.foundation/schema/Ring} and
  {@class https://uor.foundation/schema/WittLevel} classes define the algebraic substrate.
  This is where W8--W32 are declared and the ring structure is encoded formally in OWL.
  See {@concept ring} and {@concept witt-levels}.

- **{@ns op}**: The {@class https://uor.foundation/op/Operation} hierarchy defines what
  operations are possible over ring elements. The {@count:identities} named algebraic
  identities ({@class https://uor.foundation/op/Identity} individuals) live here — these
  are the theorems that the pipeline must eventually certify.

The Define stage answers: *what exists and what are its algebraic laws?*

## Stage 2 — Resolve (Bridge)

The Bridge space comprises 10 namespaces: {@ns query}, {@ns resolver}, `type`,
{@ns partition}, {@ns observable}, {@ns homology}, {@ns cohomology}, {@ns proof},
{@ns derivation}, and {@ns trace}.

These namespaces implement the computational resolution of algebraic claims:

- **{@ns query} / {@ns resolver}**: The query-resolver pair implements type-directed search
  and incremental completeness resolution. See {@concept resolution}.

- **`type` / {@ns partition}**: The type namespace implements the {@concept site} bundle
  structure — types are sites over addresses. The {@concept partition} decomposes the
  address space.

- **{@ns observable} / {@ns homology} / {@ns cohomology}**: These namespaces observe
  algebraic invariants: spectral sequences, holonomy groups, monodromy classes, and Euler
  characteristics. See {@concept observables} and {@concept homology}.

- **{@ns proof} / {@ns derivation} / {@ns trace}**: The {@concept proof-system} defines
  `ComputationCertificate` and `AxiomaticDerivation` — the two fundamental proof species.

The Resolve stage answers: *how do we compute this and what evidence does the computation produce?*

## Stage 3 — Certify (cert)

The {@ns cert} namespace stands alone in the certification layer. Despite being in the Bridge
space (it imports Bridge namespaces), it plays the Certify role: it takes the evidence
produced by the Resolve stage and issues formal certificates.

Key certificate classes:

- {@class https://uor.foundation/cert/CompletenessAuditTrail} — certifies that a
  completeness resolution is exhaustive
- {@class https://uor.foundation/cert/GeodesicCertificate} — certifies that a computation
  follows the geodesic (shortest) path in the space of derivations
- {@class https://uor.foundation/cert/GroundingCertificate} — certifies that the context
  is fully grounded

The Certify stage answers: *has this been verified to the required standard?*

## The {@count:amendments} Amendments

The UOR ontology has evolved through {@count:amendments} amendments, each adding or refining
structure. The amendments are not arbitrary patches — each one closes a gap identified through
conformance testing.

## Conformance as Truth

The {@count:conformance_checks} conformance checks are not tests — they are the specification.
Every check defines a property that the ontology must have. When a check passes, it is a
proof that the pipeline is correctly implemented in that dimension.

The website you are reading was generated from the ontology itself. Every namespace page,
every identity count, every class hierarchy diagram is derived from `Ontology::full()` at
generation time. The site is not documentation of the pipeline — it is an instance of it.
