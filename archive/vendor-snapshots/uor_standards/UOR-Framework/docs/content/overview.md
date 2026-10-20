# UOR Framework Overview

The Universal Object Reference (UOR) Framework is a formal ontology and mathematical
framework for content-addressed object spaces. It provides a unified model for
representing, resolving, and transforming any computable object using an algebraic
substrate based on ring theory and group symmetry.

## Core Ideas

**Content addressing** means an object is identified by *what it is*, not *where it is*.
The UOR framework formalizes this via {@class https://uor.foundation/u/Element}: every
object has a canonical address derived from its content, not from an external naming system.

**The ring substrate** {@class https://uor.foundation/schema/Ring} is the algebraic
foundation: Z/(2^n)Z — integers modulo 2^n. At Witt level n=8 this is the byte ring
(Z/256Z), familiar from computer arithmetic.

**Two involutions** generate the structure:
- {@ind https://uor.foundation/op/neg}: ring negation (reflection)
- {@ind https://uor.foundation/op/bnot}: bitwise complement (hypercube reflection)

These generate the dihedral group D_{2^n}, captured by {@class https://uor.foundation/op/DihedralGroup}.

**The critical identity** {@ind https://uor.foundation/op/criticalIdentity}:
`neg(bnot(x)) = succ(x) for all x ∈ R_n` — successor is the composition of the two involutions.
This is the foundational theorem proved by {@class https://uor.foundation/proof/CriticalIdentityProof}.

## Namespace Layers

The {@count:namespaces} namespaces are organized into three **space** classifications:

**Kernel** (mathematical core):
- {@class https://uor.foundation/u/Element} — universal addressing
- {@class https://uor.foundation/schema/Ring} — ring substrate
- {@class https://uor.foundation/op/Operation} — operations

**Bridge** (resolution infrastructure):
- {@class https://uor.foundation/query/Query} — queries
- {@class https://uor.foundation/resolver/Resolver} — resolvers
- {@class https://uor.foundation/partition/Partition} — partitions
- {@class https://uor.foundation/observable/Observable} — observables
- {@class https://uor.foundation/proof/Proof} — proofs
- {@class https://uor.foundation/derivation/Derivation} — derivations
- {@class https://uor.foundation/trace/ComputationTrace} — traces
- {@class https://uor.foundation/cert/Certificate} — certificates
- {@class https://uor.foundation/homology/Simplex} — homology (simplicial complexes, chain homology)
- {@class https://uor.foundation/cohomology/Sheaf} — cohomology (sheaf cohomology, local-to-global)

**User** (application layer):
- {@class https://uor.foundation/type/TypeDefinition} — types
- {@class https://uor.foundation/morphism/Transform} — morphisms
- {@class https://uor.foundation/state/Context} — state

## How It Works

The resolution pipeline transforms a typed value into a certified, traceable result:

1. A value has a **type** ({@class https://uor.foundation/type/TypeDefinition}) that declares
   what constraints apply — residue classes, carry patterns, depth bounds — each pinning
   sites of the Z/2Z fibration. See [Type System](concepts/type-system.html).

2. A **query** ({@class https://uor.foundation/query/Query}) specifies what to resolve:
   coordinates, metrics, or canonical representation. See [Resolution](concepts/resolution.html).

3. A **resolver** ({@class https://uor.foundation/resolver/Resolver}) factorizes the value
   in the ring using the dihedral group structure. For partially-constrained types, the
   resolver iterates, applying {@class https://uor.foundation/resolver/RefinementSuggestion}
   until the {@class https://uor.foundation/partition/FreeRank} closes.
   See [Factorization](concepts/factorization.html) and [Iterative Resolution](concepts/iterative-resolution.html).

4. The **partition** ({@class https://uor.foundation/partition/Partition}) decomposes the
   ring into four disjoint sets — irreducible, reducible, units, exterior — classifying
   every element. See [Partition](concepts/partition.html).

5. **Observables** ({@class https://uor.foundation/observable/Observable}) measure geometric
   properties: ring distance, Hamming distance, curvature, holonomy, and more.
   See [Observables](concepts/observables.html).

6. A **certificate** ({@class https://uor.foundation/cert/Certificate}) attests that the
   result is correct and the transform preserves the claimed isometry properties.

7. A **trace** ({@class https://uor.foundation/trace/ComputationTrace}) records every step,
   linking the certificate to the derivation that produced it.

8. **State** ({@class https://uor.foundation/state/Context}) maintains the evaluation
   context across resolution steps: Witt level, active bindings, and frame transitions.
   See [State Model](concepts/state-model.html).

## Structural Reasoning

When constraints interact in complex ways, the resolution pipeline may stall or produce
incomplete results. Amendments 21–22 add an algebraic topology layer that diagnoses these
situations:

The **Cech nerve** ({@class https://uor.foundation/resolver/CechNerve}) is a
{@class https://uor.foundation/homology/SimplicialComplex} whose simplices represent
compatible subsets of constraints. Its topological structure reveals whether resolution
will converge smoothly:

- **Trivial homology** (all Betti numbers zero except β_0) means the constraint space is
  contractible — resolution converges without obstruction.
- **Nontrivial β_1** means the constraints contain loops — cyclic dependencies that may
  cause resolution to stall.
- **Nontrivial higher Betti numbers** detect higher-dimensional voids in the constraint
  structure.

The **sheaf cohomology** layer ({@class https://uor.foundation/cohomology/Sheaf}) detects
when local constraint satisfaction fails to globalize. A
{@class https://uor.foundation/cohomology/GluingObstruction} in H^1 pinpoints exactly where
local solutions cannot be assembled into a global resolution.

See [Homology](concepts/homology.html), [Cohomology](concepts/cohomology.html),
[Sheaf Semantics](concepts/sheaf-semantics.html), and the
[ψ-Pipeline Guide](guides/psi-pipeline.html) for the full structural reasoning pipeline.

## Algebraic Verification

The framework includes **{@count:identities} named algebraic identities** spanning 7 core algebras (with additional identity families from Amendments 23–53):
Ring, Boolean, Cross-Structure, Dihedral, Unit, Affine, and Carry. Each identity is a named
{@class https://uor.foundation/op/Identity} individual with `lhs`, `rhs`, and `forAll`
properties specifying the equation and its domain.

Every identity carries typed grounding properties:
- {@prop https://uor.foundation/op/verificationDomain}: one or more references to a
  {@class https://uor.foundation/op/VerificationDomain} individual identifying the
  mathematical discipline (Enumerative, Algebraic, Geometric, Analytical, Thermodynamic,
  Topological, Pipeline, IndexTheoretic, SuperpositionDomain, QuantumThermodynamic,
  or ArithmeticValuation).
- {@prop https://uor.foundation/proof/dependsOn}: a formal dependency on other identities
  used as lemmas in the proof (Amendment 87).

Of the {@count:identities} identities, those with `universallyValid = true` are derivable
from axioms; the remainder are verifiable by exhaustive enumeration at Q0.
See [Algebraic Laws](concepts/algebraic-laws.html) for the core algebras and
their identities.
