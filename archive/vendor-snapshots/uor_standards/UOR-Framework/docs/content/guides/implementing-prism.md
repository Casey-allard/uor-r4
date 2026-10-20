# Implementing PRISM

PRISM (Polymorphic Resolution and Isometric Symmetry Machine) is the reference
implementation pattern for a UOR-compliant resolver.

## Overview

A PRISM implementation must:
1. Accept a {@class https://uor.foundation/query/Query}
2. Use a {@class https://uor.foundation/resolver/Resolver} to factorize the input
3. Produce a {@class https://uor.foundation/partition/Partition}
4. Measure {@class https://uor.foundation/observable/Observable} properties
5. Issue a {@class https://uor.foundation/cert/Certificate}
6. Record a {@class https://uor.foundation/trace/ComputationTrace}

## Step 1: Define the Context

```rust
// Establish the evaluation context at Witt level 8
let context = Context {
    witt_length: 8,
    capacity: 256,
};
```

The corresponding ontology individual:
```turtle
<my:ctx>
    a               state:Context ;
    state:wittLength "8"^^xsd:nonNegativeInteger ;
    state:capacity  "256"^^xsd:nonNegativeInteger .
```

## Step 2: Create a Query

Choose the appropriate query type:
- {@class https://uor.foundation/query/CoordinateQuery} for spatial resolution
- {@class https://uor.foundation/query/MetricQuery} for metric measurement
- {@class https://uor.foundation/query/RepresentationQuery} for canonical form

```turtle
<my:query>
    a               query:RepresentationQuery ;
    query:inputType <my:target-address> .
```

## Step 3: Apply the Resolver

The {@class https://uor.foundation/resolver/DihedralFactorizationResolver}
performs the core computation:

```turtle
<my:resolver>
    a                   resolver:DihedralFactorizationResolver ;
    resolver:inputType  <my:type-u8> ;
    resolver:strategy   "dihedral-factorization" .
```

## Step 4: Inspect the Partition

The resolver produces a {@class https://uor.foundation/partition/Partition}:

```turtle
<my:partition>
    a                   partition:Partition ;
    partition:irreducibles  <my:irred-set> ;
    partition:reducibles    <my:red-set> ;
    partition:units         <my:unit-set> ;
    partition:exterior      <my:ext-set> .
```

## Step 5: Issue a Certificate

```turtle
<my:cert>
    a               cert:Certificate ;
    cert:certifies  <my:partition> .
```

Certificate types available:
- {@class https://uor.foundation/cert/TransformCertificate}
- {@class https://uor.foundation/cert/IsometryCertificate}
- {@class https://uor.foundation/cert/InvolutionCertificate}

## Step 6: Record the Trace

```turtle
<my:trace>
    a                   trace:ComputationTrace ;
    trace:certifiedBy   <my:cert> .
```

## Iterative Resolution (Amendment 11)

The single-pass pipeline above works when the type is fully determined. For
partially-constrained types, PRISM supports an iterative resolution loop:

1. **Declare** — create a {@class https://uor.foundation/type/ConstrainedType} with initial constraints
2. **Resolve** — run the resolver to produce a partition with a {@class https://uor.foundation/partition/FreeRank}
3. **Observe** — check {@prop https://uor.foundation/partition/isClosed} on the budget
4. **Refine** — if not closed, apply a {@class https://uor.foundation/resolver/RefinementSuggestion} to pin more sites
5. **Iterate** — repeat until the budget closes or convergence stalls

The {@class https://uor.foundation/resolver/ResolutionState} tracks iteration count,
site deficit, and {@prop https://uor.foundation/resolver/convergenceRate}. Each
iteration produces a {@class https://uor.foundation/derivation/RefinementStep} recording
the applied constraint and sites closed.

## Completeness Certification (Amendment 25)

To certify that a type resolves in O(1), promote it to a
{@class https://uor.foundation/type/CompletenessCandidate} and run the ψ
pipeline via a {@class https://uor.foundation/resolver/CompletenessResolver}:

1. **Promote** — associate the type with a ResolutionState and CechNerve
   via {@prop https://uor.foundation/type/candidateNerve}.
2. **Accumulate witnesses** — each site-closing step produces a
   {@class https://uor.foundation/type/CompletenessWitness} recording the
   applied constraint and {@prop https://uor.foundation/type/sitesClosed}.
3. **Check IT\_7d** — the resolver reads the cached
   {@prop https://uor.foundation/resolver/nerveEulerCharacteristic}. If
   χ(N(C)) = n and all β\_k = 0, the kernel issues a
   {@class https://uor.foundation/cert/CompletenessCertificate}.
4. **Audit** — the certificate's
   {@prop https://uor.foundation/cert/auditTrail} links to a
   {@class https://uor.foundation/cert/CompletenessAuditTrail} with the
   full witness sequence and {@prop https://uor.foundation/cert/witnessCount}.

## W16 Resolver (Amendment 26)

To run any resolver at Witt level W16 instead of W8, use a
{@class https://uor.foundation/resolver/WittLevelResolver} with
{@prop https://uor.foundation/resolver/quantumLevel} set to
`schema:W16`. The W16 ring is {@class https://uor.foundation/schema/W16Ring}:
Z/(2^16)Z with {@prop https://uor.foundation/schema/W16bitWidth} = 16 and
65,536 elements.

Identities marked `op:universallyValid true` (such as the critical identity
and all QL\_ individuals) hold at W16 and every higher level without
re-verification.

## Session Lifecycle (Amendment 27)

Multi-turn Prism deployments use a
{@class https://uor.foundation/resolver/SessionResolver} that maintains a
{@class https://uor.foundation/state/BindingAccumulator} across queries:

1. **Open session** — create a {@class https://uor.foundation/state/Session}
   with an empty {@prop https://uor.foundation/state/sessionBindings} context.
2. **Submit queries** — each {@class https://uor.foundation/query/SessionQuery}
   declares {@prop https://uor.foundation/query/sessionMembership}; resolved
   bindings are appended to the accumulator.
3. **Monitor deficit** — the
   {@prop https://uor.foundation/state/aggregateSiteDeficit} on the
   accumulator decreases monotonically (SR\_1 invariant).
4. **Handle boundaries** — when convergence stalls or a contradiction arises,
   a {@class https://uor.foundation/state/SessionBoundary} resets the context.
   The {@prop https://uor.foundation/state/boundaryType} is one of
   `state:ExplicitReset`, `state:ConvergenceBoundary`, or
   `state:ContradictionBoundary`.

## Composition (Amendment 12)

Transforms compose categorically via {@class https://uor.foundation/morphism/Composition}.
The critical composition law {@ind https://uor.foundation/morphism/criticalComposition}
asserts that `neg ∘ bnot = succ`, connecting the two involutions to cyclic rotation.

Use {@class https://uor.foundation/morphism/Identity} for identity transforms and
{@prop https://uor.foundation/morphism/composesWith} to declare composability.

## Complete Example

See SHACL test `test7_end_to_end` in `conformance/src/tests/fixtures/test7_end_to_end.rs`
for a complete single-pass pipeline, and `test12_factorization` for a full PRISM pipeline
with free rank and certification using {@prop https://uor.foundation/cert/certifies}
and {@prop https://uor.foundation/trace/certifiedBy}.
