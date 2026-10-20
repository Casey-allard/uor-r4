# Resolution

## Definition

**Resolution** is the process of computing the canonical representation of an object
in the ring substrate. The {@class https://uor.foundation/resolver/Resolver} hierarchy
implements this process.

## Resolver Hierarchy

| Class | Role |
|-------|------|
| {@class https://uor.foundation/resolver/Resolver} | Abstract resolver base |
| {@class https://uor.foundation/resolver/DihedralFactorizationResolver} | Factorizes in D_{2^n} |
| {@class https://uor.foundation/resolver/CanonicalFormResolver} | Computes canonical form |
| {@class https://uor.foundation/resolver/EvaluationResolver} | Evaluates the canonical form |

## Resolution Process

1. A {@class https://uor.foundation/query/Query} specifies what to resolve
2. The resolver uses its strategy ({@prop https://uor.foundation/resolver/strategy})
3. The input type ({@prop https://uor.foundation/resolver/inputType}) is the source
4. Resolution produces a {@class https://uor.foundation/partition/Partition}
   (output type is `partition:Partition` via {@prop https://uor.foundation/resolver/outputType})

## Query Types

Three specialized queries correspond to the three resolver strategies:

| Query | Description |
|-------|-------------|
| {@class https://uor.foundation/query/CoordinateQuery} | Resolves spatial coordinates |
| {@class https://uor.foundation/query/MetricQuery} | Resolves metric properties |
| {@class https://uor.foundation/query/RepresentationQuery} | Resolves canonical representation |

## Complexity

The property {@prop https://uor.foundation/resolver/hasComplexityClass} declares the
computational complexity of the resolver (e.g., `"O(n)"` or `"O(log n)"`).

## Output

The final output of resolution is:
- A {@class https://uor.foundation/partition/Partition} of R_n
- A {@class https://uor.foundation/trace/ComputationTrace} recording the steps
- Optionally, a {@class https://uor.foundation/cert/Certificate} attesting the result

## The Full Pipeline

The resolution pipeline flows through several stages, each tracked by the
{@class https://uor.foundation/resolver/ResolutionState}:

1. **Query**: A {@class https://uor.foundation/query/Query} specifies the
   target — what element or property to resolve and which coordinate system
   to use.
2. **Type check**: The resolver reads the input's
   {@class https://uor.foundation/type/TypeDefinition} to determine which
   constraints apply (residue classes, carry patterns, depth bounds).
3. **Factorization**: The
   {@class https://uor.foundation/resolver/DihedralFactorizationResolver}
   decomposes the element in the dihedral group D_{2^n}, producing the
   canonical neg/bnot factorization.
4. **Partition**: The factorization result classifies the element into one of
   the four {@class https://uor.foundation/partition/Partition} components
   (irreducible, reducible, unit, exterior).
5. **Observation**: {@class https://uor.foundation/observable/Observable}
   instances measure properties of the resolved element — Hamming weight,
   spectral gap, entropy.
6. **Certification**: A {@class https://uor.foundation/cert/Certificate}
   records the resolution result with a verification status and hash.

## Iterative vs Single-Pass

The factorization pipeline (steps 1–6 above) is a **single pass** through
the resolver hierarchy. When constraints conflict or the type's site budget
is not fully resolved, the system enters an **iterative resolution loop**
that refines the solution. Each iteration applies constraints, re-evaluates
observables, and checks convergence. See
[Iterative Resolution](iterative-resolution.html) for convergence conditions
and [Factorization](factorization.html) for the single-pass dihedral
decomposition.

## Topological Diagnostics

When iterative resolution stalls, the
{@class https://uor.foundation/resolver/CechNerve} provides structural
diagnostics. Its Betti numbers reveal whether stalls arise from topological
obstructions (constraint loops) or from insufficient constraint coverage.
See [Analytical Completeness](analytical-completeness.html) for the nerve
construction and [ψ-Pipeline](../guides/psi-pipeline.html) for the full
structural reasoning workflow.
