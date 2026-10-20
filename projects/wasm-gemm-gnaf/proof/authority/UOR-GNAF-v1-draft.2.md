# UOR Global Non-Adjacent Form (UOR-GNAF) v1

**Status:** Normative Draft 0.2  
**Provisional specification identifier:** `uor-gnaf/1-draft.2`  
**Reserved stable identifier:** `uor-gnaf/1` — unassigned  
**Date:** 2026-08-11

## Abstract

UOR-GNAF specifies a machine process for accumulating typed content, kinds,
operations, semantic warrants, realizations, and cost facts into an exact,
versioned state-space, and for extracting transformations certified as globally
optimal over a complete declared universe when the required existence and
coverage obligations are discharged.

For arbitrary admitted inputs and use-cases, the specification binds the
correctness domain separately from evaluation, derives an exact observation
problem, models one complete nonanticipating system over a joint workload, and
quantifies optimality over the complete competitor class. “Arbitrary” is a
quantifier over every well-formed descriptor admitted by a sealed use-case
class; it is never an inference of undeclared meaning or a free pointwise
selector.

The process has two inseparable layers:

1. a typed semantic quotient that preserves exact meaning, together with a
   separately retained provenance/equivalence-witness graph;
2. a proof-carrying operational envelope that retains the context-qualified
   nondominated realizations of that meaning.

The accumulated object is therefore not one physical representation alleged to
be fastest for every possible operation. It is a canonical semantic state plus
a query-indexed optimality envelope. When feasibility, existence, attainment,
and coverage are proved, a query may return a certified scalar/Pareto member or,
with identity-complete coverage, the complete argmin/Pareto frontier.
Comparison-bound claims use their own result branch. Otherwise the process
returns an exact negative/incomplete status, or an explicitly authorized
weaker positive claim with its true scope.

UOR-GNAF is universal by admission protocol, global over a complete sealed
snapshot, and optimal only under an exact observation, machine, candidate
universe, resource envelope, and objective. New kinds and operations may be
admitted indefinitely. Every exact optimality certificate remains bound to the
snapshot whose complete universe it covers.

---

## 1. Purpose and scope

### 1.1 Purpose

This specification defines:

- the typed contracts by which any kind, type, content value, operation,
  realization, machine, cost model, or proof system may participate;
- the exact semantic state accumulated from admitted declarations and evidence;
- the generalized, finite-arity meaning of adjacency and non-adjacency;
- the closure and sealing laws required for insertion-order-independent state;
- the conditions under which incremental extension is equivalent to a rebuild;
- the complete-system execution and accounting boundary;
- scalar, Pareto, joint-workload, online/comparator, input-total,
  use-case-class, and revision-scoped optimality claims;
- constructive refinement/completion and exact unbounded analytic-transfer
  requirements;
- immutable snapshot, transactional update, deployment, runtime-effect, and
  maintained-class state boundaries;
- the evidence, lower-bound, coverage, and receipt requirements for those claims;
- capability-scoped conformance requirements and rejection behavior.

The specification is construction-independent. It does not require a registry,
repository, package ecosystem, graph database, e-graph, theorem prover, virtual
machine, or particular content-addressing scheme. Such systems may implement a
profile of this specification; none defines its semantics.

### 1.2 Exact meaning of the name

In **Global Non-Adjacent Form**:

- **global** means quantified over every admitted candidate in the complete
  universe fixed by a sealed snapshot and query;
- **non-adjacent** means no exact, warranted, strictly improving finite-context
  replacement is enabled under the declared generalized adjacency relation;
- **form** means a canonical semantic quotient together with its operational
  optimality envelope, not necessarily one materialized byte string;
- **any kind, any type, any operation** means every object admitted through the
  typed contracts of this specification, including future profiles. It does not
  mean that undeclared semantics are inferred or that unspecified future
  candidates are already covered by a present certificate.

### 1.3 Fundamental claim boundary

The following claims are distinct:

1. **semantic canonicality** — one accumulated state denotes one exact semantic
   object or admitted equivalence class;
2. **representation minimality** — one representation minimizes a declared
   representation objective over a declared representation universe;
3. **operation optimality** — one complete transformation system minimizes a
   declared execution objective for a declared request or workload.

No one of these claims implies another. In particular, non-adjacency is local
normality; it becomes global minimality only through exhaustive no-improver
coverage, proof-complete reduction, or an attained matching lower bound. A
minimum representation theorem becomes an execution theorem only through a
machine-specific bridge.

### 1.4 Normative language

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHALL**, **SHALL NOT**,
**SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **NOT RECOMMENDED**, **MAY**, and
**OPTIONAL** in this document are to be interpreted as described in BCP 14 when,
and only when, they appear in all capitals.

Mathematical definitions are normative. Prose examples are normative only when
marked **Normative fixture** or **Normative rejection**.

---

## 2. Terminology and required non-conflation

### 2.1 Core terms

**Kind**  
A parametric semantic family with an exact carrier, validity relation,
equivalence, observations, and admission obligations.

**Type instance**  
A kind paired with exact parameters. If `k` is a kind and `theta` is a valid
parameter, the type instance is written `k[theta]`.

**Content**  
A value inhabiting exactly one type instance in a declared context.

**Raw semantic term**  
A typed syntactic representation whose evaluation denotes content in one type
instance. It is not its evaluated semantic value. Operation/plan syntax and
machine transitions are separate objects and do not enter this quotient.

**Semantic subject**  
An element of a declared quotient of valid typed content under exact warranted
equivalence.

**Realization**  
A typed executable construction implementing an operation profile under a
declared machine contract.

**Operation profile**  
The exact input/output, observation, state, effect, failure, and composition
meaning that realizations must implement.

**Configuration**  
The complete typed memory, resource, capability, effect, control, and relevant
history state from which execution proceeds.

**Plan**  
A finite admitted derivation or transition system over a declared machine
grammar. A plan is not the operation profile it realizes.

**Query**  
A fully bound optimization request: correctness and evaluation domains, derived
observation problem, machine, resources, universe, objective, aggregation,
claim request, and selection policy.

**Problem**  
The exact typed behavior that a query requires a candidate to implement after
observation, while retaining every mandatory state, effect, failure, and
resource obligation of the base operation profile.

**Use-case profile**  
A typed joint input, update, query, environment, information, state, and cost
protocol against which one complete system or policy is compared.

**Uniform system**  
One selector-plus-executor policy, including dispatch, retained state, and
update behavior, that operates across every scenario in a use-case under its
information rule. A family of externally chosen per-scenario systems is not one
uniform system.

**Deployment configuration**  
A mutable, versioned runtime configuration pinned to an immutable sealed
snapshot. It is not part of the snapshot's semantic closure and cannot mutate
that snapshot.

**Declaration base**  
The immutable admitted declarations and evidence from which closure is derived.

**Snapshot**  
An immutable commitment to a declaration base, its exact closure, semantics,
machine and objective revisions, and completeness boundary.

**Seal**  
Evidence that a snapshot satisfies the closure, coverage, consistency, and
determinism obligations needed by its claim class.

**Address**  
An identity or lookup commitment derived from canonical bytes under an explicit
address profile. An address is never a semantic warrant.

**Trajectory**  
An ordered sequence of configurations and transitions. A trajectory is not an
unordered declaration accumulation.

### 2.2 Objects that MUST remain distinct

A conforming implementation MUST keep the following identities and propositions
distinct even when their encodings happen to be equal:

- kind descriptor, type instance, content, and semantic quotient class;
- raw syntax, evaluated operator, machine state, and execution trajectory;
- operation profile, realization, plan, invocation, result, and receipt;
- base operation profile, query-induced problem, workload, and use-case policy;
- declaration base, derived closure, sealed snapshot, and mutable deployment;
- immutable knowledge snapshot, seal layer, result certificate, and mutable
  deployment configuration;
- content address, artifact address, snapshot address, and certificate address;
- identity, domain membership, eligibility, exactness, feasibility, cost, and
  optimality;
- discovery, registration, admission, trust, evidence, and proof;
- semantic canonicality, representation minimality, and execution optimality;
- algebraic equality, topological equality or limit, observational equivalence,
  and finite numerical approximation;
- a finite matrix realization and an unbounded operator it approximates;
- a complete outer transformation system and an internal plan selected by it;
- an old snapshot and any extension derived from it.

Equality of bytes, length, hash, address, name, repository location, or
observation under one profile MUST NOT authorize substitution across these
boundaries.

### 2.3 No authority by location or identity

No repository, registry, resolver, API response, fixture, index, cache, address,
or implementation behavior is the semantic authority for UOR-GNAF. Authority
comes from this specification and explicitly identified conforming profiles.

A cryptographic digest or UOR address establishes identity only to the degree
guaranteed by its address profile. It does not establish type membership,
semantic equality, refinement, eligibility, correctness, trust, cost, or
optimality.

---

## 3. Typed semantic universe

### 3.1 Kind descriptors

A kind descriptor `K` MUST bind the tuple

```text
K = (
  KindId, Param, ParamValid, TypeId,
  Val, Valid, Eq, EqVerify,
  Obs, Constructors, Destructors,
  CanonicalProfile?, InterchangeProfile?
).
```

For every valid parameter `theta in Param`, the descriptor induces a type
instance

```text
T = K[theta]
```

with carrier `Val_T`, validity predicate `Valid_T`, semantic equivalence
`Eq_T`, and declared observation family `Obs_T`.

`Eq_T` MUST be an equivalence relation on valid values. If equivalence is
accepted by proof rather than decided directly, `EqVerify_T(x,y,w)` MUST be a
total verifier on bounded certificate bytes and MUST be sound:

```text
EqVerify_T(x,y,w) = accept  implies  Eq_T(x,y).
```

Verifier rejection proves only that `w` is not accepted. It does not prove
`not Eq_T(x,y)` unless the kind separately provides a complete decision
procedure.

A change to parameter interpretation, validity, equivalence, observation,
canonicalization, or strict parsing defines a new kind identity. A later bridge
between revisions does not mutate either identity.

### 3.2 Typed content

Typed content is a pair `(T,x)` with `Valid_T(x)`. The universe of content is a
disjoint union:

```text
C = coproduct over admitted T of {x in Val_T | Valid_T(x)}.
```

Consequently, equal host-language values in different type instances remain
different typed content. Cross-type use requires an explicit admitted operation,
equivalence bridge, or refinement warrant.

### 3.3 Proof-carrying semantic quotient

For every admitted type instance `T`, let `Raw_(S,T)` be its valid raw semantic
terms and let

```text
eval_(S,T) : Raw_(S,T) -> Val_T / Eq_T.
```

Let `~_(S,T)` be the least equivalence and congruence relation on
`Raw_(S,T)` generated by:

1. reflexivity, symmetry, and transitivity;
2. every equivalence warrant accepted by an admitted verifier;
3. every axiom explicitly declared and warranted as semantic equality by an
   admitted kind or equality profile;
4. congruence under constructors and operations only after each has
   independently been proved extensional over the already fixed input/output
   kind equivalences.

An ordinary operation equation such as `f(x) = y` specifies evaluation or a
state transition; it contributes an operational edge and does not identify `x`
and `y` in the quotient. A directed refinement likewise does not become equality
unless a separate symmetric equivalence warrant at the exact quotient
observation is admitted. This staging prevents an operation from using
congruence generated by its own unproved extensionality.

The canonical semantic state-space is the abstract quotient

```text
Q_(S,T) = Raw_(S,T) / ~_(S,T)
Q_S     = coproduct over admitted T of Q_(S,T).
```

The quotient relation MUST be semantically sound:

```text
r ~_(S,T) t
implies
eval_(S,T)(r) = eval_(S,T)(t).
```

Every generating equality warrant and every admitted congruence rule MUST
discharge this law. A relation that identifies distinct evaluated semantic
values is invalid even when no canonical representative is requested.

The state denoted by `r in Raw_(S,T)` is its class `[r]_(S,T)`. The quotient is
well-defined even when no canonical byte representative is defined. Cross-type
bridges remain typed morphisms between quotient components; they do not merge
carriers unless a separate common quotient type is explicitly declared.

A profile that emits a canonical representative

```text
Can_(S,T) : Raw_(S,T) -> Raw_(S,T)
```

MUST prove, on its claimed domain:

```text
evaluation:     eval_(S,T)(Can_(S,T)(r)) = eval_(S,T)(r)
normality:      Normal_(S,T)(Can_(S,T)(r))
idempotence:    Can_(S,T)(Can_(S,T)(r)) = Can_(S,T)(r)
completeness:   r ~_(S,T) t
                iff Can_(S,T)(r) = Can_(S,T)(t).
```

If only soundness and idempotence are proved, the claim class is `normal-form`,
not `canonical`.

The quotient is deliberately proof-carrying. UOR-GNAF does not require a total
procedure that discovers every true semantic equivalence of arbitrary programs.
Its exact closure contains every equivalence admitted by the sealed proof system
and no unwarranted one.

### 3.4 Observations and refinement

An observation profile `O` is a typed total function or total relation on its
declared domain. Two values are interchangeable for a request only when the
request's exact required-observation relation says so.

A refinement warrant from `T` to `T'` MUST bind:

- its source and target type identities;
- its exact valid domain;
- the construction or relation producing target content;
- the observation relation preserved;
- any reconstruction or error law;
- any effect, failure, and resource conditions;
- the verifier and evidence context.

Refinement is directional unless an inverse theorem is also supplied. Identity
of addresses or encodings MUST NOT be used in place of refinement evidence.

### 3.5 Algebraic and analytic profiles

A profile using algebraic language MUST provide the corresponding laws. For
example, a vector-space claim MUST bind a scalar field, addition, scalar action,
zero, and all vector-space axioms. A module claim MUST bind its ring and module
axioms.

A norm claim MUST bind its scalar absolute value and prove nonnegativity,
definiteness, absolute homogeneity, and the triangle inequality. An inner-product
claim MUST bind the scalar field, chosen linearity convention, conjugate
symmetry (symmetry over the reals), linearity/conjugate-linearity, and positive
definiteness. Any induced norm or topology MUST be derived explicitly. A
Hilbert-space claim MUST additionally prove completeness in the induced norm.

A profile using Hilbert-space or operator language MUST additionally bind the
completion and domain facts required by its claims. For an unbounded operator

```text
A : D(A) -> H
```

the profile MUST state `D(A)`, prove or warrant density when used, define the
graph, distinguish closed from closable, bind the adjoint domain and action,
identify any closure, and separately warrant self-adjointness, essential
self-adjointness, and spectral claims. An operation MUST NOT be extended beyond
its admitted domain merely by invoking continuity; continuity itself and the
applicable completion theorem MUST be present.

When linear-operator terminology is used, `D(A)` MUST be a declared linear
subspace and `A` MUST be linear. An adjoint claim requires a densely defined
operator under the selected scalar and linearity convention. Closedness,
closability, closure, self-adjointness, essential self-adjointness, resolvent,
and spectrum remain separate propositions.

Finite-prefix or finite-dimensional evidence MUST NOT be promoted to a genuine
infinite-dimensional, Hilbert, unbounded-operator, adjoint, closure, or spectrum
claim.

### 3.6 Constructive limits and exact analytic transfer

An analytic profile that constructs an unbounded or completed subject from
finite, discrete, or otherwise simpler stages MUST bind a typed directed
refinement diagram:

```text
RefinementDiagram = (
  DiagramId, IndexKind, DirectedOrder,
  StageKind_i, Refine_(i,j),
  StageObservation_i, ErrorRelation_(i,j)?,
  LimitKind?, Embed_i?,
  ConvergenceProfile, CoherenceWarrant
).
```

For every `i <= j <= k`, the warrant MUST prove on the complete declared
domain:

```text
Refine_(i,i) = identity
Refine_(j,k) o Refine_(i,j) = Refine_(i,k)
```

and MUST prove that every map or relation is typed, valid, extensional, and
observation-preserving under the exact declared error law. When a `LimitKind`
is bound, embeddings MUST commute with refinement:

```text
Embed_j(Refine_(i,j)(x)) = Embed_i(x).
```

Any cofinality, composite-error, convergence, permutation, or regrouping claim
MUST be separately proved. Agreement on sampled stages, shared prefixes,
names, encodings, or addresses is not diagram coherence.

A claimed completion MUST bind the constructive profile

```text
CompletionProfile = (
  CompletionId,
  BaseKind, CompletedKind,
  UniformityOrMetric,
  DenseEmbed,
  CauchyObjectClass,
  ApproximationNameKind, InterpretName,
  CauchyPredicate, CauchyWitnessSchema,
  NameEquivalence,
  Limit,
  ConvergenceObservation,
  DensityWarrant,
  NameCoverageWarrant,
  SeparationWarrant,
  CompletenessWarrant
).
```

It MUST prove that `DenseEmbed` is typed, injective, and isometric or uniformly
embedding exactly as claimed; that its range is dense in the bound topology;
that `CauchyObjectClass` is the exact net/filter/sequence class selected by the
uniformity; that `CauchyPredicate(n)` holds iff the interpreted name is Cauchy
under that uniformity; and that `NameEquivalence(n,m)` holds iff their
interpreted Cauchy objects are equivalent under the uniformity-induced
completion relation;
that `CauchyWitnessSchema` supplies the modulus, entourage response, filter
witness, or other constructive evidence appropriate to the declared
uniformity and name kind;
that `Limit` is total on the admitted Cauchy-name domain; that equivalent names
have equal limits; that constant names recover embedded base values; and that
admitted limits are unique. A constructive representability claim MUST supply
an admitted approximation name or a certificate constructing one. A sequence
MUST NOT silently replace a net or filter unless the profile proves the exact
sequentiality or countability theorem that permits it.

`NameCoverageWarrant` MUST prove both that every Cauchy object in the declared
class has an equivalent admitted approximation name and that every value of
`CompletedKind` claimed by the completion has an admitted name converging to
it. `CompletenessWarrant` quantifies over every object satisfying the exact
induced Cauchy predicate, not only a convenient named subset. A deliberately
smaller computable/effective subcompletion is permitted only as a separately
identified kind and explicitly restricted completion claim.

An exact unbounded-operator profile MUST bind

```text
AnalyticOperatorProfile = (
  OperatorId,
  AmbientSpaceKind,
  DomainKind, DomainInclusion,
  Action, Graph,
  ScalarConvention,
  LinearityWarrant,
  DensityStatus,
  ClosedStatus,
  ClosableStatus,
  Closure?, Adjoint?,
  SymmetryStatus,
  SelfAdjointStatus,
  EssentialSelfAdjointStatus
).
```

`DomainKind` is a distinct subtype kind and `DomainInclusion` MUST be an
injective typed map into the ambient space. Operator equality includes the
ambient space, exact domain, inclusion, and action; equality of formulas or
bytes is insufficient. Closedness means graph closedness in the bound product
topology. Closability means the graph closure is single-valued. A closure binds
its exact domain and action and MUST prove

```text
Graph(Closure(A)) = TopologicalClosure(Graph(A))
A subseteq Closure(A)
```

where operator inclusion means domain inclusion and equality of action on the
smaller domain. A declared adjoint MUST use the profile's inner-product
convention. If the inner product is linear in its first argument, then

```text
D(A*) = { y in H | exists a unique z in H,
                       for every x in D(A), <A x,y> = <x,z> }
A* y = that unique z.
```

If it is linear in its second argument, the defining equation is
`<y,A x> = <z,x>`. Density of `D(A)` and nondegeneracy MUST discharge existence
of a single-valued adjoint operator where that claim is made. A relation merely
labeled “adjoint” or an arbitrary extension of `A` is insufficient.
Symmetry is not self-adjointness;
self-adjointness requires equality with the adjoint including equality of
domains, and essential self-adjointness requires the proved closure to be
self-adjoint.

If a coordinate or weighted realization is used, its domain MUST be defined by
the exact summability or integrability predicate. Graph-norm completeness,
closedness, density, and any passage between coordinate and abstract domains
MUST be independently warranted. A finite support core alone does not establish
the domain or closure.

A property may pass from a refinement diagram or approximation family to its
limit only through

```text
LimitTransferWarrant = (
  TransferId,
  RefinementDiagramId,
  SourcePropositions,
  ExactTargetProposition,
  ConvergenceMode,
  QuantifiersAndCofinality,
  StabilityOrErrorHypotheses,
  SpectralApproximationWarrantId?,
  VerifierRevision
).
```

The warrant MUST prove the exact direction claimed. Refinement structure alone
does not transfer closedness, invertibility, kernel dimension, adjoints,
self-adjointness, resolvents, or spectra. Reuse across a conservative extension
also requires preservation or reflection of the topology, dense embeddings,
domains, graphs, convergence, and each transferred proposition.

A spectral claim MUST bind

```text
SpectralProfile = (
  SpectralProfileId, OperatorAndDomainId, ScalarField,
  AmbientTopologyAndNorm, IdentityOperator,
  Shift(lambda) = A - lambda I on D(A),
  ResolventPredicate, SpectrumPredicate,
  PointSpectrumPredicate,
  ContinuousSpectrumPredicate,
  ResidualSpectrumPredicate,
  ApproximatePointSpectrumPredicate?,
  PartitionOrInclusionTheorems?, MultiplicityProfile?
).
```

For the usual closed-operator profile on a complex Banach/Hilbert space, the
exact predicates are

```text
lambda in resolvent(A)
  iff Shift(lambda) is bijective D(A) -> H and its inverse H -> H
      is everywhere-defined and bounded

spectrum(A) = ScalarField \ resolvent(A)

lambda in point-spectrum(A)
  iff ker(Shift(lambda)) != {0}

lambda in continuous-spectrum(A)
  iff Shift(lambda) is injective, has dense range, and is not surjective

lambda in residual-spectrum(A)
  iff Shift(lambda) is injective and its range is not dense

lambda in approximate-point-spectrum(A)
  iff there is an admitted unit-vector net x_i in D(A) with
      Shift(lambda)x_i -> 0.
```

A different spectral convention MUST replace these definitions explicitly and
MUST NOT reuse their claim names. Any claimed partition or inclusion among the
classes requires its own theorem; the definitions alone do not assert one.
The profile MUST keep the resolvent, full spectrum, each spectrum class, kernel,
surjectivity, and existence of an everywhere-defined bounded inverse as
separate propositions. In particular,

```text
ker(A) != {0}
0 is an eigenvalue of A
0 in spectrum(A)
A is not surjective
A has no bounded everywhere-defined inverse
```

MUST NOT be conflated.

When spectra are transferred from stages or approximations, the profile MUST
bind

```text
SpectralApproximationWarrant = (
  WarrantId, RefinementDiagramId,
  StageOperatorAndSpectralSet_i, TargetOperatorAndSpectralSet,
  SpectralRegion, ClaimScope, RegionCoverageProposition,
  SetTopologyOrConvergenceMode,
  CofinalIndexQuantifiers,
  NoLossPredicate?, NoPollutionPredicate?,
  MultiplicityComparison?, Hypotheses, Proof
).
```

`NoLossPredicate` means that every target spectral point in the bound region is
the declared limit of stage spectral points along the required cofinal
subnet/sequence. `NoPollutionPredicate` means that every declared convergent
cofinal net/subnet of stage spectral points in that region has its limit in the
target spectral set. Multiplicity comparison is absent unless multiplicity is
defined on both sides and the exact preservation law is proved. Neither
predicate is implied by operator convergence without the bound theorem.
`RegionCoverageProposition` proves the exact relationship between
`SpectralRegion`, the target spectrum, and every stage set used. For a
full/global spectral claim it MUST cover the entire target spectrum and the
entire advertised stage scope. An empty or proper region can support only an
explicitly restricted-region claim and MUST NOT warrant full-spectrum loss or
pollution statements.

Every semantic-to-spectral zero claim MUST bind

```text
ZeroCorrespondenceWarrant = (
  CorrespondenceId,
  SourceZeroPredicate,
  TargetSpectralPredicate,
  Direction,
  WitnessMaps?,
  OperatorAndDomainIds,
  ApplicableRefinementCompletionAndTransferIds,
  SpectralApproximationWarrantId?, MultiplicityProfileId?,
  Hypotheses,
  Proof
).
```

`ApplicableRefinementCompletionAndTransferIds` is an exact dependency set and
MAY be empty for a direct correspondence. Every refinement, completion, or
limit-transfer fact actually used MUST occur in it; unused machinery is not a
mandatory assumption.

`Direction` is `forward`, `reverse`, or `iff`; an `iff` claim requires both
directions. The source predicate and construction of the operator MUST be
defined independently of the target spectral conclusion, so the proof cannot
be made true by defining either side in terms of the other. A determinant,
finite compression, numerical eigenvalue, small residual, interval containing
zero, or address identity never supplies this correspondence by itself.

---

## 4. Declarations, facts, and admission

### 4.1 Declaration classes

A declaration base MAY contain:

- kind and type-instance descriptors;
- typed content and provenance;
- operation profiles and primitive realizations;
- composition, equivalence, and refinement warrants;
- machine, resource, observation, and cost profiles;
- candidate-generation and closure rules;
- proof-verifier descriptors and trust roots;
- lower-bound, completeness, dominance, and extension-sufficiency certificates;
- prior snapshot references and exact deltas.

Every declaration MUST have a canonical identity under its own profile. The
identity MUST commit to every field that can alter its semantic interpretation.
Mutable names and resolver results MAY be metadata but MUST NOT be identity.

Whenever this specification displays an `...Id` inside the tuple it names, the
normative construction is nonrecursive:

```text
ObjectBody = the complete displayed semantic tuple with ObjectId omitted
ObjectId   = Identity(ObjectDomain, ObjectBody).
```

`ObjectDomain` is a distinct typed identity domain and includes any parent or
context identity on which interpretation depends. The identifier field is a
check/projection of this construction, never an input to itself. Profiles MUST
state which display-name or diagnostic fields, if any, are nonsemantic; no
unlisted field may be silently excluded.

### 4.2 Admission result

Admission is total over bounded declaration input and returns exactly one of:

```text
admitted       declaration is valid and its required warrants verify
rejected       declaration is definitively invalid under the bound profile
unresolved     a referenced identity or verifier result is unavailable
unsupported    the implementation lacks the claimed optional capability
incoherent     it conflicts with already sealed immutable meaning
```

`unresolved` and `unsupported` MUST NOT be interpreted as false. A rejected or
unresolved declaration MUST NOT affect exact closure except as an immutable
diagnostic record outside the admitted base.

### 4.3 Fact values

Unless a profile supplies a complete decision procedure, exact facts have three
knowledge states:

```text
true(warrant) | false(warrant) | unknown
```

Absence, lookup failure, timeout, index miss, unrecognized certificate, and
verifier unavailability produce `unknown`, not `false`.

Every semantic fact and eligibility proposition MUST be invariant under the
declared equivalence of its typed subject. A representation-layout fact MUST NOT
be promoted to a semantic fact.

### 4.4 Consistency

Two declarations are incoherent if they assign different immutable meanings to
the same semantic identity. A snapshot containing an unresolved incoherence
MUST NOT seal.

If accepted evidence roots derive both a proposition `f` and its exact negation
`not f`, the dependent fact context is `incoherent`. The implementation MUST NOT
use either derivation to admit or prune a realization. A snapshot MAY seal only
after excluding that context or
applying an exact trust-policy resolution whose identity and consequences are
bound into the snapshot.

New evidence MAY relate existing immutable objects; it MUST NOT retroactively
change their kinds, bytes, observations, or declared semantics. A changed
descriptor receives a new identity.

### 4.5 No proof cycles

An artifact MUST NOT establish the soundness of the verifier that is the sole
basis for accepting that artifact. Proof and trust dependencies MUST form an
explicitly accepted foundation or a well-founded dependency graph. Cyclic
self-warrant without an independently trusted fixed-point rule is invalid.

---

## 5. Operations and the abstract machine

### 5.1 Operation profiles

An operation profile `P` MUST bind:

```text
P = (
  OpId, InputSig, OutputSig, ValidInvocation,
  InvocationEquivalence, TargetBehavior,
  TargetEquivalence, RequiredObservation,
  StateSemantics, EffectSemantics,
  FailureSemantics, ChoiceSemantics,
  BehaviorConformance, CompletionContract,
  CompositionLaw
).
```

Input and output signatures are finite ordered families of type patterns and may
be nullary or multi-output. `ValidInvocation` is exact. A partial operation MUST
represent out-of-domain invocation by its declared refusal/result semantics;
undefined behavior is not an admitted observation.

For each valid invocation `z` and initial semantic state `sigma`,

```text
TargetBehavior_P(z,sigma)
```

is the exact nonempty relation or allowed-result object over ordered outputs,
observable transitions, final state, effects, refusal, and—when declared—the
complete outcome distribution. A deterministic functional operation is the
special case whose target relation is a singleton. Stateful target behavior
MUST bind initial state, ordered observable transitions, and final state.
`RequiredObservation_P` is the operation profile's declared deterministic total
base projection function on this target behavior; its value MAY itself be a
relation-, distribution-, or refinement-valued object. It is not an undefined
observation inferred from a realization. A query may compose only a compatible
total projection over this base observation as specified in §8.1.

`ChoiceSemantics_P` MUST state whether the target is deterministic, exact
relational may/must behavior, probabilistic law, demonic or angelic refinement,
or another exact typed behavior object. `BehaviorConformance_P` is the total
relation by which a complete realized behavior is compared with that target.
Per-run membership is sufficient only for an explicitly declared may-set
semantics. It does not establish relation equality, a must behavior,
availability of every result, fairness, or equality of an outcome law.

`CompletionContract_P` distinguishes successful termination, permitted refusal
or failure, divergence, and productive infinite behavior. A productive
infinite behavior MUST bind its exact prefix/productivity/fairness observation
and infinite-trace cost domain; it is neither silently accepted through a
finite-run implication nor rejected merely because it does not terminate.
An executable productive profile additionally binds a total next-observation-
boundary/continuation contract: within each declared run resource slice it
returns a verified finite prefix, a typed resource/partial status, or a terminal
outcome. A merely coinductive behavior with no productive boundary machine may
be analyzed but cannot satisfy the total execution capability of Appendix A.

The existence of two kinds or a path of names between them does not imply an
operation. Operation meaning MUST NOT be inferred from proximity, shared bytes,
common addresses, common parameters, or registry membership.

### 5.2 Realization declarations

A primitive realization `r` MUST bind:

```text
r = (
  RealizationId, ProfileId,
  InputPattern, OutputPattern,
  SemanticDomain, SemanticEligibility,
  Transition, RealizedBehavior,
  ExactnessWarrant, ResourceTransformer,
  CostEvents, FailureAndRefusal,
  CompositionEvidence
).
```

`SemanticEligibility_R(z,sigma,S)` depends on a typed invocation, its semantic
initial state, and admitted semantic facts. `Feasible_R(q,z,s,X)` is the
separate machine, capacity, resource, capability, and effect proposition.
Eligibility, exactness, feasibility, and cost are separate propositions. No one
MAY substitute for another.

`SemanticDomain_R` is the exact query- and machine-independent domain on which
the realization declares behavior. `RealizedBehavior_R(z,sigma)` is the exact
nonempty semantic relation/law over outputs, state, effects, failures, refusal,
and completion induced by `Transition_R` on that domain. It contains no cost or
resource-feasibility assertion. An implementation under a machine must
separately prove that its complete `Exec_X` realizes this declared behavior.
`RealizedBehavior_R` is total and nonempty exactly when
`SemanticDomain_R(z,sigma)` holds, and every accepted eligibility proof MUST
satisfy

```text
SemanticEligibility_R(z,sigma,S)
  implies SemanticDomain_R(z,sigma).
```

A realization with an optimized path that can refuse is not a complete compared
system unless refusal is the requested result. Otherwise its exact selector,
dispatcher, validator, fallback, and all failure paths MUST be included and
admitted.

### 5.3 Configurations

For a sealed universe `U`, a machine configuration is

```text
s = (mu, rho, chi) in Cfg_U
```

where:

- `mu` is finite typed memory, `mu : Location partial-> C`;
- `rho` is the complete resource and capacity state;
- `chi` is the complete capability, control, effect, randomness, cache, advice,
  and history state that can affect later eligibility, behavior, or cost.

Every occupied memory location is bound to exactly one typed value. If execution
history affects any later transition or charge, the relevant history MUST be
represented in `chi`. A state-local recurrence MUST NOT be used after omitting
cost-relevant or behavior-relevant state.

A realization instance induces a typed transition

```text
s --(r, omega, trace-event)-->_U s'
```

where `omega` belongs to the declared outcome set. Deterministic execution has a
singleton outcome set.

### 5.4 Complete configuration semantics

Plans operate on configurations, not isolated scalar values. The model therefore
supports:

- multiple inputs and outputs;
- destructive and persistent updates;
- DAG sharing and reuse of intermediates;
- fusion and common-subexpression retention;
- parallel or nondeterministic schedules;
- persistent caches and prepared state;
- effects, capabilities, and failures.

A profile MUST NOT silently replace this machine by a tree expression language,
single-output model, stateless model, or no-sharing model. Such restrictions are
permitted only when explicitly bound by the machine grammar and candidate
universe.

For a query `q`, complete system `R`, invocation `z`, and initial configuration
`s`, define

```text
Exec_X(q,R,z,s)
```

as the exact nonempty object of all maximal executions permitted by `X`. It is a
trace relation, may/must object, probability law, adversarial strategy outcome,
or other behavior object selected by the bound machine and operation profiles.
It MUST include every machine, environment, scheduler, randomness, oracle,
failure, refusal, and divergence choice permitted by `X`; it MUST exclude every
impossible choice. Every actual trace denotes exactly one represented outcome,
or the profile MUST bind an explicit exact quotient law. A stochastic object
MUST bind a normalized measure on a declared measurable space. An empty,
sampled, or selectively truncated execution object invalidates the machine
profile and cannot make a universal obligation vacuously true.

### 5.5 Exactness

For every admitted realization `r` of `P`, every semantic state `sigma`, and
every `z` with `SemanticDomain_r(z,sigma)`, its exactness warrant MUST establish

```text
BehaviorConformance_P(
  RealizedBehavior_r(z,sigma),
  TargetBehavior_P(z,sigma))
```

together with the declared semantic state, effect, failure, and completion
obligations on the complete realized behavior. It MUST NOT assume a query,
machine, resource envelope, feasibility, or cost. A **full-profile** realization
additionally proves `SemanticDomain_r(z,sigma)` for every
`z in ValidInvocation_P` and every initial semantic state admitted by `P`.

For a well-formed query with `z in CorrectnessDomain_q`, a complete system under
`X_q` MUST separately prove that `Exec_X(q,R,z,Init_q(z))` exactly realizes its
declared composed semantic behavior under the bound choice/conformance profile.
This execution-realization bridge and feasibility/resource proofs belong to
admissibility in §8.4, not to the realization's semantic exactness warrant. The
behavior comparison always covers the complete relation/law, not one selected
`omega`.

For a query-induced `Problem_q` from §8.1, a complete candidate `R` MAY
implement the observed problem directly without constructing unobserved base
outputs. It is exact exactly when, for every `z in CorrectnessDomain_q`,

```text
Accept_q(
  z, SemanticState(Init_q(z)),
  Exec_X(q,R,z,Init_q(z)))
```

holds and every mandatory state, effect, failure, and completion obligation
retained by `Problem_q` holds on every maximal execution. Projection MUST NOT
hide or weaken those mandatory obligations. A full-profile realization is one
permitted special case.

If an exact operation is represented by a deterministic function, this reduces
to equality with the reference value on the complete valid domain. Approximate,
probabilistic, interval, or refinement-valued operations MUST use their exact
declared relation and aggregation; they MUST NOT be described as exact equality.
For a distribution-valued target, the warrant proves the declared exact
equality/refinement of the complete induced law and separately proves every
maximal semantic behavior meets the declared support, state, effect, failure,
and completion conditions. Machine feasibility and resources remain separate
§8.4 obligations. Support inclusion or finite sampling is insufficient.

### 5.6 Extensionality

Let `(z,sigma) ~=_P (z',sigma')` mean that corresponding input values satisfy
the full declared kind-level `Eq_T` relations—not merely the currently known
proof closure—and that the initial semantic states are equivalent under the
exact state observation bound by `P`. Every admitted operation profile MUST
prove:

```text
(z,sigma) ~=_P (z',sigma')
implies
  ValidInvocation_P(z) = ValidInvocation_P(z')
  and TargetBehavior_P(z,sigma)
      ~=Targets_P
      TargetBehavior_P(z',sigma').
```

Separately, every admitted realization `R` MUST prove semantic-eligibility
saturation:

```text
(z,sigma) ~=_P (z',sigma')
implies
SemanticEligibility_R(z,sigma,S)
  = SemanticEligibility_R(z',sigma',S).
```

For a relational, multi-output, stateful, or probabilistic operation,
`~=Targets_P` means equivalence of the complete related-result set or exact
outcome law under the declared output, state, effect, and observation
equivalences. A function-only equality proof is insufficient for such an
operation.

An operation that intentionally distinguishes two raw spellings MUST take a
finer syntax-bearing type as input. It MUST NOT be admitted as an operation on a
quotient that identifies those spellings.

### 5.7 Composition

For every connected plan boundary, the producer output and consumer input MUST
be the same type instance or be joined by an explicit typed composition or
refinement warrant. Equality of host values or bytes is insufficient.

Composition evidence MUST cover intermediate validity, capacity, effects,
failure, reconstruction, and observation. End-to-end exactness is not inferred
from component names or from untyped reachability.

For partial or unbounded operators, the natural domain of every composite is
part of the operation identity. In particular,

```text
D(A o B) = { x in D(B) | B(x) in D(A) }.
```

Sums, products, powers, restrictions, extensions, adjoints, and closures MUST
likewise bind their exact domains and inclusion maps. Agreement of formulas on
a common core does not establish equality of operators or validity on a larger
domain.

### 5.8 Syntax, operators, states, and trajectories

Profiles MUST distinguish:

- a term encoding an operation;
- the evaluated mathematical or machine operator;
- a configuration on which it acts;
- the transition produced by one invocation;
- the ordered trajectory produced by repeated invocations;
- any content or artifact address recording those objects.

Unordered declaration accumulation MUST NOT erase semantically relevant
trajectory order. If order is semantic, it is explicit typed content.

Higher-order, dependent, variadic, streaming, interactive, and coinductive
operations are admitted through explicit kinds and trajectories. Each machine
transition has a finite typed interface; an unbounded family or stream is one
typed content object whose productivity, observation, and resource semantics
are declared. A finite port list MUST NOT be interpreted as a restriction to
finite semantic inputs, nor may an infinite arity be silently materialized as a
finite prefix.

---

## 6. Derivation space and generalized non-adjacency

### 6.1 Typed derivation hypergraph

For snapshot `S`, concrete execution is the typed transition graph

```text
G_S = (Cfg_S, Transition_S),
```

whose edges are complete configuration transitions from §5.3. A trajectory is
a path in `G_S`.

An optional symbolic planning representation is the typed directed hypergraph

```text
H_S = (Port_S, Hyperedge_S),
```

whose vertices/ports are typed component values and whose hyperedges consume
and produce finite ordered component families plus symbolic resource, effect,
and control interfaces. A symbolic derivation MUST denote the same complete
configuration transitions as `G_S` under an exact realization map.

A plan is a finite descriptor over these transitions. Its control graph MAY
contain cycles, recursion, iteration, feedback, sharing, or scheduling choices;
each admitted run MUST nevertheless meet the query's exact termination/refusal
and resource contract. A completed run has a finite trace unless the operation
profile explicitly makes an infinite behavior its semantic result.

The hypergraph MAY be represented explicitly, symbolically, lazily, by weighted
deduction, by equality saturation, or by another exact construction. The
representation is conforming only if its claimed results equal the abstract
definitions in this specification.

### 6.2 Plan contexts

A plan context `C[-]` is a well-typed admitted plan with one finite typed hole.
Plans `p` and `p'` have the same contextual boundary when either can fill that
hole without changing its declared input/output, state, effect, capability, and
resource interface.

For an admitted complete realization or plan `p`, define

```text
Beh_q(p) : CorrectnessDomain_q
  -> outcome-indexed required observations and observable final state.
```

`EquivalentFor_q(Beh_q(p'),Beh_q(p))` is the exact query-defined behavioral
relation. It may require equal behaviors, equivalent distributions, refinement,
or merely membership in the same allowed target relation. It is fixed by `q`
and is not inferred from plan syntax.

Write `a preceq_q b` for the objective's corresponding non-worse relation. Its
strict part is

```text
a prec_q b  iff  a preceq_q b and not (b preceq_q a).
```

For a scalar total order this is ordinary strict improvement; for a vector or
partial order it is strict dominance. A profile exposing a primitive strict
comparison MUST prove that it is exactly this strict part on the claimed
domain.

### 6.3 Generalized improving adjacency

For a sealed snapshot `S`, query `q`, and admitted context `C[-]`, define

```text
p' <_(S,q,C) p
```

if and only if:

```text
C[p]  in Adm_S(q,X)
C[p'] in Adm_S(q,X)
EquivalentFor_q(Beh_q(C[p']), Beh_q(C[p]))
J_q(C[p']) prec_q J_q(C[p]).
```

The replaced occurrence may contain any finite number of inputs, outputs,
states, or transitions. Generalized adjacency is therefore a hyperrelation, not
pairwise physical proximity.

### 6.4 GNAF normality

A plan or retained operational configuration `p` is **GNAF-normal for q** when
no finite admitted occurrence in it has a replacement satisfying §6.3.

Normality is relative to `S`, `q`, the context class, exact observation, machine,
and objective. It is not an unqualified property of bytes or graphs.

Local GNAF normality is necessary for global scalar optimality but is not
sufficient. A seal MUST NOT derive a global claim from the absence of known
local reductions unless the reduction family is proved complete and the global
minimum or no-dominator result is established by complete comparison,
proof-complete reduction, or lower-bound attainment.

### 6.5 Contextual dominance

Every query that permits operational pruning MUST bind

```text
ContinuationClass_q = (
  ContinuationClassId,
  ContextGrammar, BoundaryTypes,
  MembershipSemantics,
  InformationAndHistoryInterface,
  CompositionClosure,
  CoverageScope
).
```

For every boundary on which dominance is used, the class MUST be nonempty and
contain both the identity continuation and the current complete-system context.
A dominance certificate binds the class identity and covers every context
generated by its grammar. Extending the class requires a new theorem or exact
reconstruction; an empty class, current-query sample, or unspecified future
class cannot authorize pruning.

For two same-boundary alternatives, define

```text
p' <=^ctx_S p
```

when, for every allowed query `q` and every allowed continuation context `C[-]`,

```text
C[p] in Adm_S(q,X)
implies
  C[p'] in Adm_S(q,X)
  and EquivalentFor_q(Beh_q(C[p']), Beh_q(C[p]))
  and J_q(C[p']) preceq_q J_q(C[p]).
```

Dominance is **uniformly strict** when `J_q(C[p']) prec_q J_q(C[p])` for every
applicable pair `(q,C[-])`. It is **nontrivially weak** when it is never worse
and is strict for at least one applicable pair.

The retained operational envelope MUST be an antichain under uniformly strict
contextual dominance. Equal-cost distinct identities remain distinct unless a
declared identity quotient merges them.

An alternative MAY be irreversibly pruned only when:

1. a contextual-dominance certificate covers every continuation allowed by the
   seal and proves the strength required by the claim being preserved; or
2. the alternative is exactly reconstructible, and reconstruction plus all
   future uses and costs are covered by the same certificate.

Nontrivially weak dominance is sufficient to preserve a scalar minimum value
when the dominating alternative remains retained. It is not by itself
sufficient to preserve an identity-complete argmin, deterministic tie result, or
Pareto frontier in contexts where the two alternatives are equal. Those claims
require uniform strictness on the covered contexts, a declared identity
quotient, or exact reconstruction of the omitted identity.

Dominance observed only in current queries, on one machine, or under one cost
model is insufficient for extension-safe pruning. If future contexts are
unrestricted, operational alternatives generally MUST remain recoverable.

### 6.6 Representation-retention sub-invariant

The representation-retention sub-invariant is:

```text
RepresentationRetentionInvariant_S = (
  TypedSemanticQuotient_S,
  OperationalBasisMap_S)

OperationalBasisMap_S
  = (b |-> OperationalBasis_S(b))
    over every seal-covered boundary class b.
```

This pair is a required projection of the full `GNAFSpace_S` defined in §10.3;
it is not the complete accumulated state. Canonical semantic spellings MAY be merged because all admitted operations on
that quotient are extensional. Operational realizations MUST remain separate
until contextual dominance or exact reconstructibility authorizes pruning.

Provenance is retained alongside, not inside, the quotient:

```text
ProvenanceEnvelope_S([r]_(S,T))
  = admitted source, equality-witness, refinement, and derivation records
    whose exact typed semantic subject is [r]_(S,T).
```

Every provenance record has its own immutable identity and typed subject. Merging
semantic representatives MUST NOT erase the record or pretend that two proof
paths are identical. Provenance pruning requires its own retention or exact
reconstruction policy and never changes semantic equality.

For each sealed boundary class `b`, the retained operational state is

```text
OperationalBasis_S(b) = (
  Retained_S(b),
  ReconstructionManifest_S(b),
  RetentionCoverage_S(b)
).
```

`RetentionCoverage_S(b)` MUST prove, for every alternative `p` in the complete
covered plan universe, at least one of:

1. `p` and its identity are represented in `Retained_S(b)`;
2. an applicable retained alternative contextually dominates `p` with the
   strength required by every preserved claim; or
3. `p`, its identity, behavior, relevant retained state, provenance, and
   complete reconstruction cost are exactly reconstructible from objects whose
   retention is itself committed by the manifest.

The proof scope binds the problem/query class, continuation class, machine,
cost models, objectives, result modes, and identity-sensitive policies.
`Envelope_S(q)` is derived from this basis; it is not the basis. No retained
object or reconstruction dependency may be garbage-collected while a seal
depends on it. Loss of both an alternative and its reconstruction basis makes
the affected deployment unresolved and prevents execution under that seal.

---

## 7. Accumulation, closure, and sealing

### 7.1 Knowledge accumulation

Let `B` be an immutable set of admitted declarations and warrants. Knowledge
accumulation uses the typed declaration join `sqcup_D`:

```text
B sqcup_D Delta = B union Delta.
```

This join is associative, commutative, and idempotent. These laws apply to
declarative knowledge. They do not imply that every semantic operation on
content is commutative or idempotent.

When sequence is semantically significant, the sequence is typed content and is
not reordered by knowledge accumulation.

### 7.2 Consequence operator

Fix one semantic, rule, admission, proof-verifier, and trust universe `U`. Let
`Decl_U` be its typed admitted-declaration carrier, let `D_U` be its distinct
set of scoped derivable judgments, and bind a union-preserving seed map

```text
j_U    : Decl_U -> P(D_U)
Seed_U : P(Decl_U) -> P(D_U)
Seed_U(B) = union { j_U(d) | d in B }.
```

`j_U(d)` contains exactly the initial judgments asserted by admitted declaration
`d` under `U`; it is not an identity coercion. Let

```text
I_U : P(D_U) -> P(D_U)
```

be the fixed sound consequence operator containing every consequence required
by admitted:

- equivalence and congruence rules;
- kind constructors and destructors;
- operations and composition rules;
- eligibility and resource rules;
- candidate-generation grammars;
- cost composition rules;
- dominance and lower-bound rule schemas.

The last two categories contain only pre-identity schemas and
snapshot-independent consequences. A snapshot-bound dominance, feasibility,
lower-bound, optimum, or frontier instance is forbidden here and belongs to the
post-identity seal layer below.

Define

```text
Phi_(U,K)(S) = K union I_U(S)                 for K subseteq D_U
ClBar_U(K)   = least S such that Phi_(U,K)(S) = S
Cl_U(B)      = ClBar_U(Seed_U(B))             for B subseteq Decl_U.
```

These laws are scoped to one fixed semantic, admission, proof-verifier, and
trust context. A trust-root change, verifier-policy change, revocation, or
exclusion creates a new active declaration base and requires rebuild or a
transition theorem. Immutable historical records MAY remain append-only, but
the new active admitted base need not contain the old one.

`I_U` MUST be monotone on the complete lattice `P(D_U)`:

```text
S subseteq T  implies  I_U(S) subseteq I_U(T).
```

Changing a rule, verifier, or trust context defines a new `U`; closure equations
MUST NOT be mixed across those universes.

`ClBar_U` is the closure operator on judgment sets and MUST satisfy:

```text
K subseteq ClBar_U(K)                                extensivity
K subseteq L implies ClBar_U(K) subseteq ClBar_U(L) monotonicity
ClBar_U(ClBar_U(K)) = ClBar_U(K)                    idempotence
ClBar_U(K union L)
  = ClBar_U(ClBar_U(K) union ClBar_U(L))            merge law

Cl_U(A sqcup_D B)
  = ClBar_U(Cl_U(A) union Cl_U(B))                  declaration merge.
```

The merge law makes exact closure independent of insertion, batching, worker,
enumeration, and saturation order. An implementation MAY be incremental or
demand-driven, but every claimed answer MUST equal the least-fixed-point
semantics.

The declaration closure committed into `SnapshotBody` contains only facts whose
subjects are available before `SnapshotId` exists. Such a fact MUST bind

```text
DeclarationScopeId = Identity(
  U, active declaration base, closure rules, trust context
).
```

Snapshot-bound dominance, lower-bound, feasibility, optimum, and frontier
judgments MUST NOT enter `Cl_U(B)`. They are constructed only after
`SnapshotId` exists and bind the exact
`(UniverseContextId, q, X, M, objective)` scope. Evidence needed to authorize a
seal resides in the post-identity evidence layer committed by
`SealCertificate`. A later query-result certificate MAY be derived only from a
query class and coverage basis already committed by that seal, and MUST bind
that `SealId`; otherwise the same `SnapshotCandidate` must receive a new seal
and `SealId` before the stronger claim is used. New candidates create a new
scope whose current envelope MUST be recomputed. An unscoped “currently
optimal” or “currently dominated” fact MUST NOT enter either evidence layer.

### 7.3 Materialization

`Cl_U(B)` may be finite or infinite. UOR-GNAF does not require eager
materialization. A finite index, cache, or saturation prefix is not the closure
unless completeness is proved.

A global claim over an infinite symbolic closure requires a theorem, complete
reduction, or certified cutoff that covers every admitted candidate relevant to
the claim. If the least declaration closure itself is unresolved, construction
remains a `PendingUpdate` or unpublished checkpoint. If closure is exact but
only candidate/optimization coverage is incomplete, an exact candidate may be
`UnsealedSnapshot` or a sealed snapshot may answer with an honest weaker class
per its seal basis.

A theorem-complete symbolic closure artifact MUST bind

```text
SymbolicClosure = (
  SymbolicClosureId,
  SymbolicLanguage,
  Denotation,
  MembershipAndAbsenceInterface,
  LeastClosureProof,
  ProofVerifierId
).
```

It MUST prove

```text
Seed_U(B) subseteq Denotation
I_U(Denotation) subseteq Denotation

for every T,
  (Seed_U(B) subseteq T and I_U(T) subseteq T)
  implies Denotation subseteq T.
```

These obligations establish the least prefixed point and hence the declared
least closure. Fixed-point equality or rule closure alone is insufficient,
because a larger fixed point may contain unwarranted judgments. Every
query-specific membership, absence, coverage, or optimization claim over the
symbolic closure MUST have a total verifier for the exact proposition consumed
by that claim.

Every bounded closure or seal API MUST terminate in exactly one of these
semantic outcome categories:

```text
success(value, certificate)
incomplete(checkpoint? , outstanding obligations)
terminal-rejected-or-incoherent-or-unsupported(details)
unresolved(dependency identities)
conflict(current identity)
resource-exhausted(ResourceReport)
internal-failure(FailureReport).
```

Procedure-specific result tags MUST map one-to-one to these categories.
`materialized-complete` and `symbolic-complete` are closure-success variants;
`checkpoint` is closure-incomplete. `sealed` is seal-success;
`SealResult.incomplete` has no closure checkpoint because the candidate already
exists. A tag name does not merge categories or change their evidence meaning.

An in-progress computation is pinned to

```text
CheckpointTarget =
    pre-candidate(TransactionId)
  | post-candidate(SnapshotId).
```

New deltas MUST NOT move either target. A checkpoint binds the
applicable target identity, rule/trust universe, stage, processed and pending
work roots, and implementation/verifier revisions. Resumption under a different
identity requires restart or an exact transition theorem. Eventual-completion
claims additionally require a finite
bound or well-founded progress measure and a fair scheduler theorem; otherwise
only safety and honest `incomplete` results are claimed.

### 7.4 Snapshot state

A snapshot is constructed without identity recursion. First define

```text
SemanticRoot = Identity(
  extensional semantic projections of active declarations,
  Cl_U(B), ClosureLeastnessEvidenceRoot, semantic quotients,
  bound evidence/rule/trust context
)

HistoryRoot = Identity(
  ParentSnapshotIds, ParentHistoryRoots,
  TransitionProfileId, TransactionId,
  DeltaId, DeltaInputDescriptorId, AdmissionReportId, PendingRoot,
  RejectionLog, LogicalParentOrderProfileId
)

SnapshotBody_v = (
  SemanticRoot, HistoryRoot,
  KindSet, ContentScope, OperationSet,
  ProofAndTrustContext, DeclarationBase,
  SemanticQuotient, DerivationHypergraph,
  ClosureRules, MachineSet, CostModelSet,
  QueryClass, UseCaseClassProfileSet, ProvenanceEnvelope,
  AccumulationProfileSet,
  ContributionIndexRoot, OccurrenceIdentityProfile,
  MultiplicityProfileRoot, AccumulatedSubjectRoot,
  CanonicalAccumulationRoot, AccumulationProofRoot,
  OperationalAlternativeRoot, ReconstructionArtifactRoot,
  BodyDependencyManifestRoot, BodyUnresolvedObligationRoot
)

SnapshotId_v = Identity(SnapshotBody_v).
```

`Identity(body)` excludes any field intended to contain that resulting identity.
Two histories MAY have the same `SemanticRoot` and different `HistoryRoot`.
Insertion/order independence applies to the semantic root, not to the historical
record.

For every snapshot claiming `gnaf-state(G,U)`, the accumulation fields bind the
exact `G`, every contributing occurrence, its type, quotient class, and
multiplicity treatment. `AccumulatedSubjectRoot` and
`CanonicalAccumulationRoot` MUST be deterministic projections of that committed
contribution index under `G`; `AccumulationProofRoot` binds the applicable
homomorphism, absorption, totality, and safety evidence. An
accumulation-dependent certificate or receipt MUST bind these roots. Different
accumulation profiles or occurrence interpretations therefore cannot share the
same `SnapshotId` accidentally.

`OperationalAlternativeRoot` commits the pre-identity candidate/derivation
artifacts; `ReconstructionArtifactRoot` commits the recipes and dependencies
available to reconstruct them. Neither asserts snapshot-scoped dominance,
retention sufficiency, an optimum, or a frontier. After `SnapshotId` exists,
`OperationalBasis_S(b)` and its exact coverage proofs are constructed from
these roots and committed only by `SealCertificate`, avoiding an identity cycle.

`LogicalParentOrderProfileId` describes semantic parent order when it matters
for a merge and is fixed before identity computation. Actual publication time,
head sequence, and compare-and-swap result belong to the update receipt and
deployment head, not `SnapshotBody`; otherwise preparation would depend on its
future linearization event.

For a theorem-complete symbolic closure, `SemanticRoot` commits to the exact
rule/profile identities, symbolic definition, soundness/fixed-point/leastness
evidence, and coverage warrant rather than pretending to enumerate an infinite
set. For a materialized closure it likewise commits the carrier and its
leastness/fixed-point evidence. Neither `SnapshotBody` nor `VERIFY_SEAL` may
recover closure evidence from an ambient proof cache. `Identity` is an abstract typed
commitment here; canonical bytes and address domains exist only under a selected
interchange/address profile.

The pre-seal candidate and its universe context are

```text
SnapshotCandidate_v = (SnapshotBody_v, SnapshotId_v)

UniverseContext_(v,Problem,X) = (
  SnapshotId_v, Problem, X,
  CandidateGrammarProjection(SnapshotBody_v)
)

UniverseContextId_(v,Problem,X)
  = Identity(UniverseContext_(v,Problem,X)).
```

`Body(C_v)` projects `SnapshotBody_v` from a candidate; `Body(S_v)` makes the
same projection from a sealed snapshot. `U_sys(Body(C_v),Problem,X)` is derived
only from this pre-seal body, the bound problem, and the machine contract.
It MUST NOT depend on a seal certificate, `SealId`, operational envelope,
selected realization, query result, or the certificate currently being
verified. This staging prevents a candidate universe or its coverage proof from
depending on itself.

Coverage evidence is attached only after the body identity exists:

```text
SealCertificate_v = (
  SealSchemaRevision, SnapshotId_v,
  QueryAndUseCaseCoverageBasis,
  SymbolicClosureProof?, OperationalBasisRoots, RetentionCoverage,
  UniverseCoverage, OperationalEnvelopes,
  LowerBounds, CoverageWitnesses,
  SealDependencyManifestRoot, SealUnresolvedObligationRoot,
  verifier manifest and revisions
)

SealId_v = Identity(seal-certificate-domain, SealCertificate_v)
SealedSnapshot_v = (
  SnapshotCandidate_v, SealCertificate_v, SealId_v
)

UnsealedSnapshot_v = (
  SnapshotCandidate_v, ExactIncompleteSealStatus
).
```

The notation `S_v` is reserved for `SealedSnapshot_v` wherever admission,
extraction, a certified envelope, or a global claim is defined. An
`UnsealedSnapshot_v` is a distinct type and MUST NOT be passed where a sealed
snapshot is required. It may be inspected, repaired, or extended, but it cannot
authorize execution or an exact optimization claim until a seal succeeds.

Every semantic, admission, candidate, cost, and optimization function written
with an `S` subscript is extensionally a function of
`SnapshotCandidate(S)` plus its explicitly displayed arguments; it MUST ignore
`SealCertificate` and `SealId`. During seal verification the same functions are
evaluated with the pre-seal candidate subscript `C`. After a successful seal,
notation such as `Adm_S`, `Argmin_S`, and `Envelope_S` abbreviates the result for
that candidate. The seal supplies evidence about those results and never changes
them. Thus no lower bound, envelope, or certificate can become an input to the
universe or proposition it certifies.

An object inside `SnapshotBody_v` MUST NOT depend on `SnapshotId_v`. Receipts bind
`SealId_v`. A certificate's own identity is likewise computed from a body that
does not contain that identity.

`QueryClass` and `UseCaseClassProfileSet` in the body are grammars/profiles
independent of `SnapshotId_v`. A use-case request-constructor profile is a total
function parameterized by a future candidate context; it does not close over an
ambient snapshot. Concrete queries, use-case class instances/requests,
`SystemUniverseId`/`PolicyUniverseId` commitments, certificates, and receipts
that bind the resulting snapshot are constructed only after `SnapshotId_v`
exists, and their instantiated roots belong to the seal layer.

### 7.5 Sealing

A seal MUST fix:

- the exact snapshot and declaration closure;
- admitted kinds, type parameters, and content scope;
- operation and realization grammars;
- semantic equivalence and required observations;
- machine, resources, outcome set, and information boundary;
- cost model, aggregation, objective, result modes, tie policy, and preference
  policy domain;
- query and continuation classes;
- admitted use-case classes, invocation scopes, workload protocols, information
  rules, competitor classes, and quantifier prefixes;
- accumulation profiles, occurrence roots, canonical accumulation, and
  pre-identity alternative/reconstruction roots plus the snapshot-scoped
  operational retention/reconstruction basis;
- completeness boundary and certificate verifier revisions;
- every unresolved item that prevents a stronger claim.

For its claim class, a seal MUST prove or certificate-check:

1. semantic soundness of every included fact and rewrite;
2. consistency of immutable identities;
3. closure completeness or a theorem-complete symbolic representation;
4. extensionality of admitted operations over merged semantic classes;
5. frontier coverage for every pruned operational alternative;
6. lower-bound or no-dominator coverage for claimed optima;
7. deterministic complete-set extraction independent of discovery order when
   `execute-selected` is supported; a member/bound answer instead binds its
   exact witness identity without claiming canonical selection;
8. exact totality of every consumed cost, aggregation, objective, and behavior
   comparison;
9. dependency-manifest coverage including quantified-domain and negative
   assumptions.

Semantic inconsistency or incomplete semantic normalization prevents a
`canonical` claim. Incomplete candidate, cost, dominance, or lower-bound
coverage prevents `global-optimal` and `frontier-complete`. A snapshot MAY carry
a weaker semantic claim even when operational coverage remains incomplete, but
it MUST NOT exceed the obligations actually discharged.

### 7.6 Closure absorption within one universe

For one fixed rule/trust universe `U`, the closure laws imply:

```text
Cl_U(A sqcup_D B)
  = ClBar_U(Cl_U(A) union Seed_U(B))
  = ClBar_U(Cl_U(A) union Cl_U(B)).
```

This declaration-closure absorption is separate from non-idempotent semantic
accumulation in §10.7. It authorizes reuse of a closed consequence set while
computing the new closure. The final `SemanticRoot` still commits to the
original active declaration base and the resulting closure; no equality between
roots with different active bases is implied. It does not erase or normalize
`HistoryRoot`, snapshot identity, rejection logs, or provenance.

### 7.7 Incremental update

For an append-only admitted delta `Delta_v` under a preserving admission/trust
transition, the abstract update is

```text
B_(v+1)       = B_v sqcup_D Delta_v
C_(v+1)       = Cl_(U_(v+1))(B_(v+1))
SnapshotBody  = BuildSnapshotBody(C_(v+1), parent, delta, profiles)
SnapshotId    = Identity(SnapshotBody)
SealAttempt   = VerifySeal(SnapshotId, requested claim obligations).
```

`VerifySeal` returns a separate seal certificate on success and an exact
incomplete/failure status otherwise; it is not a closure or normalization
operator.

Batch admission is

```text
AdmitBatch_U(B, Pending, Delta)
  -> (AdmittedDelta, AdmissionReport, Pending').
```

It MUST be a deterministic function of the complete typed batch and fixed
admission/trust universe. Its admitted set and per-declaration statuses are
independent of enumeration, batching suborder, worker order, and scheduling.
Dependencies within one batch may satisfy one another only through a
well-founded proof-dependency graph or an independently warranted fixed-point
rule.

Every profile MUST choose one unresolved-item policy:

- `explicit-resubmission` — reconsider only when the declaration identity is
  resubmitted; or
- `committed-pending` — commit unresolved declarations to `PendingRoot` and
  reconsider them deterministically after relevant dependency additions.

Pending declarations do not affect closure before admission. Unsupported,
rejected, and incoherent declarations retain their exact diagnostics and are
never silently promoted.

If the rule, verifier, or trust transition revokes/excludes active declarations,
`B_(v+1)` is produced by that exact transition profile and need not equal a
union. The remaining closure, body-construction, and seal obligations still
apply, and old monotonicity theorems MUST NOT be reused across the changed `U`.

Every derived artifact MUST expose a dependency manifest covering direct
semantic and proof dependencies; verifier, machine, cost, objective, and policy
dependencies; candidate and quantified-domain roots; negative/completeness
assumptions; and retained-state/reconstruction dependencies. Define

```text
TransitionDiff = ExactDiff(
  old candidate, prospective pre-seal candidate,
  old/new active base, pending set, and admission report,
  old/new rule, verifier, trust, and transition universe,
  old/new closure, quotient, accumulation/contribution/multiplicity roots,
  old/new candidate grammars, machines, costs, workloads, classes, and profiles
).
```

`Impact(TransitionDiff)` is a sound transitive over-approximation of every
artifact whose dependency, quantified domain, interpretation, or hypothesis
changes. It therefore covers removals, revocations, pending-item promotion, and
profile changes even when they are not syntactically present in the submitted
delta. It is computed from the old body manifests, the prospective body's
manifests, and—when the old base is sealed—the old seal manifests. Reuse outside
`Impact` requires a preservation theorem. Invalidation never mutates an old
certificate; it marks that immutable statement inapplicable to the new
candidate unless a transition warrant proves reuse.

A conforming incremental implementation MUST produce the same result as this
definition. It MUST:

1. strictly decode, resolve, and type the complete delta;
2. run deterministic batch admission, preserve every per-declaration status,
   and update the chosen pending policy;
3. materialize every newly enabled operation and composition instance required
   by the requested seal, or bind a theorem-complete symbolic representation;
4. propagate equivalence, congruence, refinement, eligibility, and resource
   consequences;
5. update the contribution index, accumulated subject, canonical accumulation,
   and their proofs under the bound `G`, checking every intermediate before
   commit;
6. build the pre-identity operational-alternative, reconstruction-artifact, and
   body-dependency roots;
7. construct the exact `SnapshotBody` and `SnapshotId` after reaching the least
   closure required for a candidate;
8. compute `TransitionDiff` and its sound transitive impact set;
9. construct snapshot-scoped operational bases and either recertify every
   affected value, lower bound, frontier, use-case answer, and dependency or
   mark the requested seal incomplete;
10. publish the new immutable snapshot and update receipt at the transaction's
   single linearization point.

A partially validated update MUST NOT become visible as an exact sealed
snapshot. If the least declaration closure or quotient cannot be completed or
theorem-covered, the update remains `PendingUpdate` or unpublished. If those
semantic objects are exact but requested operational recertification is
incomplete, the candidate MAY publish only as an explicitly typed
`UnsealedSnapshot` or remain unpublished. Neither case may fabricate
completion.

### 7.8 Conservative extension

Let `C` and `C'` be pre-seal `SnapshotCandidate` contexts governed by rule/trust
universes `U` and `U'`, respectively. Write `Type_C` for the admitted type
instances committed by `Body(C)` and use the candidate-indexed §3.3 semantic
objects. A conservative extension MUST supply typed embeddings

```text
iota_T   : Type_C -> Type_C'
iota_R,T : Raw_(C,T) -> Raw_(C',iota_T(T))
iota_S,T : (Val_T / Eq_T) -> (Val_(iota_T(T)) / Eq_(iota_T(T)))
iota_Q,T : Q_(C,T) -> Q_(C',iota_T(T)).
```

and is semantically conservative when, for old raw states:

```text
iota_S,T(eval_(C,T)(x))
  = eval_(C',iota_T(T))(iota_R,T(x))

iota_S,T is injective on old semantic classes.

x ~_(C,T) y
  implies
iota_R,T(x) ~_(C',iota_T(T)) iota_R,T(y)

iota_Q,T([x]_(C,T)) = [iota_R,T(x)]_(C',iota_T(T)).
```

It is additionally **proof-quotient conservative** when

```text
x ~_(C,T) y
  iff
iota_R,T(x) ~_(C',iota_T(T)) iota_R,T(y).
```

New equality evidence between old terms may preserve underlying semantics while
failing proof-quotient conservativity; it then creates a new canonical proof
state and requires the applicable recomputation/sufficiency proof.

Old observations MUST be preserved, and every old operation MUST remain
extensional with the same exact meaning on old inputs.

An analytic extension additionally binds exact preservation or reflection maps
for every reused topology, refinement diagram, completion embedding, convergence
notion, operator domain, graph, closure, adjoint, and spectral proposition. A
finite-stage or old-universe proposition transfers only through its bound
`LimitTransferWarrant` or a proposition-specific conservative-extension
theorem.

For full accumulation profiles `G` and `G'`, a replay-free semantic-state
extension MUST additionally provide

```text
iota_GR : Raw_G -> Raw_G'
iota_GS : Sem_G -> Sem_G'

Eval_G'(iota_GR(x)) = iota_GS(Eval_G(x))
iota_GR(zeroRaw_G) = zeroRaw_G'
iota_GS(zeroSem_G) = zeroSem_G'
iota_GR(mergeRaw_G(x,z))
  = mergeRaw_G'(iota_GR(x),iota_GR(z))
iota_GS(mergeSem_G(a,b))
  = mergeSem_G'(iota_GS(a),iota_GS(b))
iota_GS is injective on old accumulated semantic states

iota_GS(Lift_(G,T)(a))
  = Lift_(G',iota_T(T))(iota_Q,T(a))

Project_G'(
  mergeRaw_G'(iota_GR(x), y))
=
Project_G'(
  mergeRaw_G'(iota_GR(Project_G(x)), y)).
```

The final equation is cross-universe absorption and is required for every old
contribution `x` and admissible new contribution `y`, unless an equivalent
sufficient-state theorem is supplied. Semantic conservativity alone does not
establish replay-free operational pruning.

Every retained operation transition MUST also be natural under the embedding.
If `SemStep_P^G(a)` is its complete target-state relation, the profile MUST
prove, with relation image/equality under `iota_GS`,

```text
iota_GS(SemStep_P^G(a))
  = SemStep_P^G'(iota_GS(a)).
```

For probabilistic or relational transitions this equation ranges over the
complete target law/relation, not a selected result.

If a new operation distinguishes old raw spellings merged by `Can_(C,T)` or
`Project_G`,
the extension is not extensional over the old quotient. It MUST either:

- take a new, finer syntax-bearing input kind;
- retain and replay the necessary raw evidence under a new semantic universe;
- or be rejected.

### 7.9 Candidate-only extension

If an extension changes only the admitted candidate set while preserving old
semantics, `Problem_q`, `InvocationScope_q`, complete behavior, machine,
resources, costs, objective, and observations, compare a transported query pair
`q_v` and `q_(v+1)`. Their request,
machine, cost, aggregation, objective, continuation, and tie-policy fields are
identical, as are result mode and any preference-policy fields, but each has a
distinct `QueryId` and binds the `SystemUniverseId` of its own snapshot. Any
restricted universe is likewise transported by an exact candidate embedding.
Then

```text
ApplicableAdm_(S_v)(q_v,X)
  subseteq ApplicableAdm_(S_(v+1))(q_(v+1),X).
```

If both minima are attained under one preserved lower-is-better scalar
objective, then

```text
OPT_(S_(v+1),M)(q_(v+1),X) <= OPT_(S_v,M)(q_v,X).
```

The same inequality holds for infima taken in one declared preserved order
completion even when a minimum is not attained.

Accumulated evidence is monotone; the selected optimum is not. A newly admitted
candidate may replace the old selection, and the Pareto frontier may gain and
lose members.

### 7.10 Revision binding

A certificate for `S_v` remains true only as a statement about `S_v`. It MUST
NOT be relabeled as a certificate for `S_(v+1)`.

Reuse at `S_(v+1)` requires a transition warrant proving at least:

1. the selected realization remains admitted and exact;
2. every observation, machine, resource, cost, aggregation, result-mode, and policy
   dependency is preserved by an exact map;
3. for one scalar/Pareto member, every new candidate is no better or fails to
   dominate that member; for `frontier-complete`, every new nondominated identity
   is included, every omitted identity is strictly dominated by a retained
   member, and newly dominated old members are removed from the frontier view;
4. semantic normalization preserves the exact subject;
5. equal-cost new identities are incorporated before deterministic selection.

The reused semantic request is represented by the transported snapshot-bound
query pair of §7.9; the old `QueryId` or `SystemUniverseId` MUST NOT be copied
into the new certificate.

No certificate quantifies over unspecified future additions. A future-stability
claim MUST bind a declared extension grammar and a theorem covering every
extension in that grammar.

### 7.11 Transactions, runtime state, and maintained coverage

An update request is

```text
UpdateRequest = (
  TransactionId,
  ExpectedKnowledgeHead,
  ExpectedRuntimeState?,
  TransitionProfileId,
  DeltaId, DeltaInputDescriptorId,
  RequestedSealObligations,
  UpdateResourceContractId
).

DeltaInputDescriptor = (
  InputKindAndInterchangeProfileId,
  Source = immutable-object(ObjectId,ContentLength,ContentRoot) |
           retained-stream(StreamId,FramingProfileId,DeclaredLengthOrBound,
                           RetentionAndFaultDomainWarrantId),
  StrictConsumeAllRule,
  StreamingDecodeAndHashProfileId,
  ResumeCursorAndStateGrammar,
  CheckpointRetentionLifetime,
  IdentityEqualityAndAvailabilityProof
).

DeltaInputDescriptorId
  = Identity(delta-input-descriptor-domain, DeltaInputDescriptor).

RequestedSealObligations = (
  RequestedSealObligationRoot,
  RequestedSealObligationObjectId,
  ObligationGrammarAndIdentityProfileId
).
```

Let `UpdateRequestBody` be this tuple without `TransactionId`; then

```text
TransactionId = Identity(update-transaction-domain, UpdateRequestBody).
```

The transaction ledger records this exact body identity. Replay with the same
identity and body is idempotent; the same identifier with nonidentical bytes or
semantics is malformed/collision and MUST NOT execute. Strictly decoded delta
identity MUST equal `DeltaId` before admission, and the resolved transition
profile identity MUST equal `TransitionProfileId`.
Before decode, the descriptor identity, source availability, framing, and
retention warrant are checked within the update resource contract. A decode
checkpoint is legal only when its cursor, incremental hash state, unread source,
and retention lifetime are sufficient to resume exactly. Otherwise exhaustion
is nonresumable and returns `resource-exhausted`; no transient caller buffer is
an admissible descriptor.

`ExpectedRuntimeState`, when present, is

```text
(
  DeploymentLineageId, DeploymentId, SnapshotId, SealId,
  ConfigurationRoot, ConfigurationEpoch,
  RetainedStateRoot, EffectLedgerRoot, RuntimePolicyStateRoot
).
```

It is present when an update consumes or persists a value from mutable
deployment state; the admitted occurrence and provenance then bind this entire
read set. It is absent for a declaration-only update. A change to any bound
runtime field before linearization produces conflict rather than a snapshot
derived from an uncommitted or partially matched read.

It is evaluated against exactly `ExpectedKnowledgeHead` and, when present,
the complete `ExpectedRuntimeState`, and publishes at one linearization point.
The knowledge-publication head is distinct from a runtime deployment:

```text
KnowledgeHead =
    empty
  | published(SnapshotId, PublishedSealId?).
```

`ExpectedKnowledgeHead` has exactly this type; its candidate component is the
semantic parent recorded in `HistoryRoot` when it is nonempty. A changed published-seal component
also conflicts, so an update cannot overwrite or ignore a concurrent reseal.

If the knowledge head or any expected runtime-state field changed, the machine
MUST either return `update-conflict` without publication or construct an
explicit rebase or merge request that binds every parent and delta and
recomputes admission,
closure, accumulation, invalidation, body identity, and seal. A body, closure,
accumulation state, retention basis, envelope, or certificate computed from
different bases MUST NOT be spliced into one snapshot.

Replaying one `TransactionId` returns the same committed receipt or a
deterministic duplicate status and MUST NOT create another occurrence. Equal
content under a new `OccurrenceId` is an intentional distinct occurrence and
follows `Multiplicity_G`.

The state planes are distinct:

```text
immutable SnapshotCandidate
immutable SealCertificate and SealId
immutable query/use-case result certificate
mutable versioned DeploymentConfiguration
occurrence delta producing a new SnapshotCandidate.
```

A sealed snapshot and `GNAFSpace_S` are immutable. An operation over `Sem_G`
changes a value in a runtime configuration; it does not mutate the snapshot.
Persisting that output as accumulated content requires an admitted occurrence
delta and a new snapshot. A runtime configuration is

```text
DeploymentConfiguration = (
  DeploymentLineageId, DeploymentId,
  SnapshotId, SealId,
  ConfigurationRoot, ConfigurationEpoch,
  RetainedStateRoot, EffectLedgerRoot, RuntimePolicyStateRoot
).
```

`DeploymentLineageId` is the stable lineage chosen or created by an activation
profile. `DeploymentId` is the version identity of the entire remaining tuple
under §4.1 and changes whenever any snapshot, seal, configuration, epoch,
retained-state, effect-ledger, or runtime-policy field changes.
`ConfigurationEpoch` is strictly monotone and never reused within one lineage;
creation allocates its first epoch and every head-changing transition allocates
the unique successor. Thus a full deployment-head mismatch cannot undergo an
ABA transition back to the same expected value.

`RuntimePolicyStateRoot` commits the exact executable/quarantined status of
every realization-machine binding and the policy governing investigation,
replacement, and recertification. It participates in `DeploymentId`, every
runtime read set, activation/migration, and CAS. No execution may begin through
a quarantined binding. A violation transition atomically commits both all
effects already observed and the quarantine state before another run can
interleave; a receipt alone is not enforcement.

Every initial or continuation execution is an identity-bearing transaction:

```text
ExecutionSubject =
    query(QueryId, InvocationIdentity)
  | workload(UseCaseRequestId, WorkloadRunInputId)
  | continuation(ContinuationId, ContinuationStateRoot,
                 ContinuationStepNumber)

ExecutionRequest = (
  ExecutionTransactionId,
  ExecutionIngressProfileId,
  ExpectedDeploymentConfiguration,
  ExecutionSubject,
  AnswerIdentity, CertificateId,
  RunResourceContractId,
  RecoveryPolicyId,RecoveryPolicyCoreId,
  RecoveryResourceContractId
)

ExecutionIngressProfile = (
  ExecutionIngressProfileId,
  FixedCanonicalRequestHeaderGrammar,
  MaximumHeaderSize,
  BootstrapResourceEnvelope,
  FreshExecutionAttemptAndResourceGrantIssuerProfileId,
  NoRunPublicationSliceConstructorAndSufficiencyWarrant,
  StreamingRequestBodyRootProcedure,
  NoEffectProofConstructorId
).

ExecutionIngressAllocationReceipt = (
  ExecutionIngressProfileId,
  FreshExecutionAttemptAndResourceGrantIssuerProfileId,
  TrustedIssuerEpoch,PriorIngressCapabilitySerial,
  TrustedIngressCapabilitySerial,ExecutionTransactionId,
  ExecutionRequestBodyRoot,CoordinatorStateVersionBeforeAndAfter
).

NoEffectProofConstructorCapability = (
  PrimaryNoEffectProofToken=live-no-effect-constructor(
    ExecutionIngressProfileId,ExecutionTransactionId,ExecutionRequestBodyRoot,
    TrustedIngressCapabilitySerial,primary,NoEffectProofConstructorId),
  RaceReplacementNoEffectProofToken=live-no-effect-constructor(
    ExecutionIngressProfileId,ExecutionTransactionId,ExecutionRequestBodyRoot,
    TrustedIngressCapabilitySerial,race-replacement,NoEffectProofConstructorId)
).

NoRunPublicationSliceId = Identity(
  no-run-publication-slice-domain,
  ExecutionIngressProfileId,ExecutionTransactionId,ExecutionRequestBodyRoot,
  TrustedIngressCapabilitySerial).

NoRunPublicationToken = live-no-run-publication(
  ExecutionIngressProfileId,ExecutionTransactionId,ExecutionRequestBodyRoot,
  TrustedIngressCapabilitySerial,NoRunPublicationSliceId).

These are primitive affine bootstrap capabilities. The primary token constructs
at most one proposed terminal `NoEffectProof`; the disjoint race-replacement
token constructs at most one replacement proof from the atomic `RECORD_NO_RUN`
loss observation. Unused siblings are disposed on return. Publishing/disposing
one no-run branch consumes its token, and immutable profile/capability IDs cannot
recreate live values.

ExecutionInvocationAttemptId = Identity(
  execution-invocation-attempt-domain,
  TrustedIssuerProfileId,TrustedIssuerEpoch,TrustedIssuerInvocationSerial,
  ExecutionTransactionId,ExecutionRequestBodyRoot).

`TrustedIssuerInvocationSerial` is atomically allocated and never repeated
within or across issuer epochs by the foundation-bound fresh-capability issuer;
its allocation receipt names the prior/new serial and issuer profile. Two calls
with identical transaction/request bodies therefore cannot share an attempt ID.

ExecutionResourceGrantReceipt = (
  ExecutionInvocationAttemptId,ExecutionTransactionId,
  ExecutionRequestBodyRoot,RunResourceContractId,RecoveryResourceContractId,
  RecoveryPolicyCoreId,TrustedIssuerProfileId,
  PriorIssuerSerial,NewIssuerSerial,
  FreshCapabilityFamilyIds,IssuerAllocationTransitionProofId,
  IssuerFreshnessAndDisjointnessProofId
).

ExecutionResourceGrantReceiptId = Identity(
  execution-resource-grant-receipt-domain,ExecutionResourceGrantReceipt).

FreshCapabilityFamily = (
  ExecutionInvocationAttemptId,ExecutionResourceGrantReceiptId,
  OrderedFreshCapabilityIds,PairwiseNonaliasingAndBoundsStatementId
).

SingleUsePartitionConstructionToken = live(
  ExecutionInvocationAttemptId,ExecutionResourceGrantReceiptId,
  PartitionConstructionCapabilityId).

ConsumedExecutionResourceGrantProof = consumed(
  ExecutionInvocationAttemptId,ExecutionResourceGrantReceiptId,
  PartitionConstructionCapabilityId,ExecutionResourcePartitionCoreRoot,
  AtomicConsumeTransitionStatementId).

ExecutionResourceGrantDispositionProof = disposed(
  ExecutionInvocationAttemptId,ExecutionResourceGrantReceiptId,
  PartitionConstructionCapabilityId,ExactFailureBranch,
  NoPartitionCapabilityEscapedStatementId).

SingleUseAffineDispositionToken = live-disposition(
  ExecutionInvocationAttemptId,IssuerAllocatedDispositionSerial).

UnreservedPartitionDispositionCapabilityId = Identity(
  unreserved-partition-disposition-capability-domain,
  ExecutionInvocationAttemptId,IssuerAllocatedDispositionSerial).

NoReservationOwnershipTransferProof = no-transfer(
  ExecutionInvocationAttemptId,ExecutionResourcePartitionCoreRoot,
  ExactCoordinatorObservationId,AffineClosureStatementId).

The `live`, `consumed`, `disposed`, `live-disposition`, and `no-transfer`
constructors above are primitive affine capability/proof sorts supplied by the
trusted issuers. A given capability ID has exactly one legal terminal transition:
consume into the displayed committed successor or dispose into the displayed
no-escape/no-transfer proof. Neither proof constructor can recreate a live
token, and a live token is not serializable or copyable merely because its ID
appears in an immutable receipt.

ExecutionResourceGrant = (
  ExecutionInvocationAttemptId,ExecutionResourceGrantReceipt,
  FreshCapabilityFamily,SingleUsePartitionConstructionToken
).

ExecutionResourceGrantResult =
    granted(ExecutionInvocationAttemptId,ExecutionResourceGrantReceipt,
            ExecutionResourceGrant)
  | rejected(Reason) | unresolved(DependencyIds) | unsupported(FeatureIds)
  | resource-exhausted(ResourceReport) | internal-failure(FailureReport).

After the fixed ingress and coherent absent-key/head preflight,
`BoundedAcquireFreshExecutionResourceGrant` returns exactly this algebra. Each
call receives the full expected coordinator/issuer-map state, atomically
advances the named issuer serial together with `CoordinatorStateVersion`, and
receives a fresh attempt-keyed capability family; identical concurrent
transaction bodies never share linear validation, reservation, effect, or
finalization tokens. A losing attempt affinely closes its private remainder.
The immutable request commits only the generic recovery policy/core and resource
contract IDs, not a call-specific partition or liveness instance.

ExecutionResourcePartitionResult =
    complete(ExecutionResourcePartition,
             ExecutionResourcePartitionObjectId,
             PartitionAndSufficiencyWarrant,
             ConsumedExecutionResourceGrantProof,
             LiveUnreservedPartitionDispositionCapability)
  | rejected(Reason,ExecutionResourceGrantDispositionProof)
  | incoherent(ConflictProof,ExecutionResourceGrantDispositionProof)
  | unresolved(DependencyIds,ExecutionResourceGrantDispositionProof)
  | unsupported(FeatureIds,ExecutionResourceGrantDispositionProof)
  | resource-exhausted(ResourceReport,ExecutionResourceGrantDispositionProof)
  | internal-failure(FailureReport,ExecutionResourceGrantDispositionProof).

`BoundedVerifyAndPartitionExecutionResource` consumes the grant's unique
partition-construction token exactly once. Its complete branch binds every
partition capability to that attempt/grant receipt; every other constructor
contains the exact affine disposal proof. Replaying an immutable receipt without
the live single-use grant cannot construct a partition.

LiveUnreservedPartitionDispositionCapability = (
  UnreservedPartitionDispositionCapabilityId,
  ExecutionInvocationAttemptId,SingleUseAffineDispositionToken
).

ExecutionIngressResult =
    complete(ExecutionTransactionId,PreReservationActionTraceRoot,
             NoEffectProofConstructorCapability,NoRunPublicationToken,
             DeploymentHeadState,ExecutionIngressAllocationReceipt)
  | terminal-no-run(ExecutionResult).

`BoundedValidateFixedExecutionIngressHeader` returns exactly this algebra. A
`terminal-no-run` value contains its exact malformed/unsupported/unresolved/
resource/internal diagnostic and a bootstrap-scoped no-effect proof; no caller
projects an undeclared `ExactNoRunExecutionResult` from a generic failure tag.
The helper is an operation on the supplied full `CoordinatorState`: before
exposing either live bootstrap capability it atomically advances the exact
execution issuer-map serial and state version and returns the allocation receipt.
Every terminal branch consumes or disposes that allocation internally.

`BoundedObserveExecutionKeyAndHead` returns one
`ExactExecutionKeyHeadObservation` built under the ingress envelope. It performs
the key/head read in one coordinator snapshot, constructs the displayed
observation body/ID, and exposes no separate free entry or head value.

The canonical `ExecutionRequest` header is fixed-size under this profile: every
variable-size invocation, workload input, answer, certificate, or resource body
is represented there only by its typed identity/root. The bootstrap envelope is
an implementation-foundation resource committed by the machine/implementation
profile. It permits only strict header decoding, streaming request-body identity,
transaction-ledger lookup, run-contract lookup, and construction of a scoped
`NoEffectProof`. It also yields one disjoint affine `NoRunPublicationSlice`
sufficient to hash/store one no-run result and perform its ledger CAS when no
execution partition can be obtained. That slice is consumed only by
`RECORD_NO_RUN`, disposed with a no-write proof on replay/identity-reuse or after
a successful reservation, and cannot execute a realization or emit an effect.

`RequestedSealObligations` in every update or reseal request is the displayed
fixed-size root/object reference, never an inline obligation carrier. Any later
`Root(RequestedSealObligations(...))` denotes the committed
`RequestedSealObligationRoot` accessor without rescanning input. The strict full
obligation object is resolved only within the request's bound resource contract.

InvocationIdentity
  = Identity(invocation-domain,
             QueryId, StrictTypedInvocationBodyWithoutIdentity)

WorkloadRunInputId
  = Identity(workload-run-input-domain,
             UseCaseRequestId, FullTaggedWorkloadRunInputBodyWithoutIdentity)

AnswerIdentity
  = Identity(answer-domain,
             ExactAnswerBranchWithoutIdentityIncludingScopePredicateAndSystem)

ExecutionTransactionId
  = Identity(execution-transaction-domain,
             ExecutionRequest excluding ExecutionTransactionId).

UniversalFenceProtocolStatement = (
  SinkEnforcementProfileId,
  LeaseExpiryOrExplicitQuiescenceRule,
  DeduplicationAndInFlightIntentRule,
  EffectCommitModelClassAndAuthorizationLaw,
  UniversalQuantificationOverWellTypedExecutionAndRecoveryFences
).

UniversalFenceProtocolStatementId = Identity(
  universal-fence-protocol-domain,UniversalFenceProtocolStatement).

UniversalFenceProtocolWarrant = (
  UniversalFenceProtocolStatement,
  UniversalFenceProtocolStatementId,
  VerifierResult=accept(VerifiedStatementIds containing
                        UniversalFenceProtocolStatementId)
).

ExecutionFenceToken = Identity(
  execution-fence-domain,
  ExecutionTransactionId,
  OriginalTransactionKey,
  ExecutionRequestBodyRoot,
  ExpectedDeploymentConfiguration.DeploymentId,
  ExpectedDeploymentConfiguration.ConfigurationEpoch,
  RecoveryBundleRoot).

ExecutionFenceSafetyStatement = (
  UniversalFenceProtocolStatementId,
  ExecutionFenceToken,
  SinkEnforcementProfileId,
  LeaseExpiryOrExplicitQuiescenceRule,
  DeduplicationAndInFlightIntentRule,
  EffectCommitModelId,
  RecoveryPolicyId
).

ExecutionFenceSafetyStatementId = Identity(
  execution-fence-safety-domain,ExecutionFenceSafetyStatement).

LeaseOrQuiescenceWarrant = (
  ExecutionFenceToken,
  SinkEnforcementProfileId,
  LeaseExpiryOrExplicitQuiescenceRule,
  DeduplicationAndInFlightIntentRule,
  VerifierResult=accept(VerifiedStatementIds containing
                        ExecutionFenceSafetyStatementId)
).

The token-specific statement is valid only as an exact proved instantiation of
the displayed `UniversalFenceProtocolStatementId` committed by the recovery
material. The universal statement contains no reservation token, bundle root, or
execution identity; the token-specific statement is constructed only after the
bundle and `ExecutionFenceToken` exist. This staging is acyclic.

ContinuationStepTransactionId
  = ExecutionTransactionId for a continuation subject.

RecoveryRequest = (
  RecoveryTransactionId,
  DeploymentLineageId,
  ExpectedReservedHeadRef,
  OriginalExecutionTransactionId,
  EffectIntentLedgerRoot,
  RecoveryPolicyId,RecoveryPolicyCoreId,
  RecoveryResourceContractId
)

RecoveryTransactionId
  = Identity(recovery-transaction-domain,
             RecoveryRequest excluding RecoveryTransactionId)

ExpectedReservedHeadRef = (
  DeploymentLineageId, ExecutionReservationId, ReservationStateRoot
).

RecoveryCoordinatorIngressProfile = (
  FixedRecoveryRequestGrammarAndStreamingIdentityEnvelope,
  TrustedFreshInvocationCapabilityIssuerProfileId,
  AtomicLedgerAndHeadObservationBound,
  ImmutableReplayTupleValidationBound,
  RecoveryOriginAndReservationGraphValidationBound,
  ExactTotalityDisjointnessAndMaximumObjectGraphWarrantId
).

RecoveryCoordinatorIngressProfileId is fixed by the implementation foundation
and resolves without consulting a request-selected object. Execution admission
MUST prove that the maximum request/result/receipt/reservation/origin graph it
can create fits this profile, including replay for a terminal result. The profile
contains bounds, not reusable slice capabilities. Its trusted issuer creates and
internally consumes a fresh per-call ingress partition for identity, observation,
replay, and origin validation; early branches dispose its remainder. That
partition is disjoint from every request recovery-resource slice.

RecoveryCoordinatorIngressResult =
    complete(
      ExactRecoveryRequestBody,RecoveryTransactionKey,
      ExactTwoKeyHeadObservation,
      OriginalExecutionReservation,CurrentExecutionReservation,
      RecoveringAlready:Boolean,ResolvedRecoveryOriginObjectGraph?,
      RecoveryInvocationAttemptId,RecoveryBootstrapAllocationReceipt,
      RecoveryResourceContractHeader,
      RecoveryInvocationBootstrapPartition)
  | duplicate(OriginalRecoveryResultId,FinalizedRecoveryResult)
  | rejected(Reason)
  | incoherent(ConflictProof)
  | conflict(CurrentDeploymentHeadState)
  | unresolved(DependencyIds)
  | unsupported(FeatureIds)
  | resource-exhausted(ResourceReport)
  | internal-failure(FailureReport).

`BoundedRecoveryCoordinatorIngress` returns exactly this algebra. Under the
fixed profile a trusted issuer first grants a fresh non-state-mutating ingress
capability. The helper strictly consumes and hashes the recovery request, makes one
serializable coordinator observation of the recovery key, the companion key
named by a reservation or winning recovery receipt, and the lineage head,
validates any terminal replay tuple, and validates the current reservation. For
an already-recovering head it strictly resolves the exact retained
`RecoveryOriginObjectGraph` and its accepted warrant, rederives the original
running reservation and request body, and proves `ReservationDescendsFrom`.
No caller performs any of those variable-size resolutions before this bounded
ingress. Only after a validated live reservation is established does it
atomically allocate one collision-free invocation attempt and derive the
attempt-keyed bootstrap partition. Duplicate/rejected/noncomplete branches do
not advance `RecoveryInvocationEpochMap`, publish state, or expose a
recovery-budget token.

RecoveryPreFenceResult =
    live(ExactTwoKeyHeadObservation)
  | validated-recovery-winner(
      OriginalRecoveryResultId,FinalizedRecoveryResult)
  | validated-normal-execution-winner(CurrentDeploymentHeadState)
  | identity-reuse(Reason)
  | conflict(CurrentDeploymentHeadState)
  | integrity-failure(FailureReport)
  | unresolved(DependencyIds)
  | unsupported(FeatureIds)
  | resource-exhausted(ResourceReport)
  | internal-failure(FailureReport).

`BoundedAtomicRecoveryPreFenceReobservation` returns exactly this algebra. It
performs all conditional result/receipt/companion-entry validation within the
fresh pre-fence slice. Only `live` exposes an observation in which both keys are
absent and the head is exactly the supplied reservation; callers never repeat a
winning-tuple validation after the slice is consumed.

RecoveryBundleLoadResult =
    complete(ResolvedRecoveryExecutionMaterial)
  | completion-warrant-violation(FailureReport)
  | rejected(Reason)
  | incoherent(ConflictProof)
  | unresolved(DependencyIds)
  | unsupported(FeatureIds)
  | resource-exhausted(ResourceReport)
  | internal-failure(FailureReport).

RecoveryEmergencyTemplateLoadResult =
    complete(ResolvedRecoveryExecutionMaterial,
             EmergencySafeQuarantineTemplate)
  | completion-warrant-violation(FailureReport)
  | rejected(Reason)
  | incoherent(ConflictProof)
  | unresolved(DependencyIds)
  | unsupported(FeatureIds)
  | resource-exhausted(ResourceReport)
  | internal-failure(FailureReport).

The primary and independent fallback loaders return exactly these algebras.
Their fixed bootstrap slices are noncheckpointing; a would-be computational
checkpoint is lifted to `completion-warrant-violation`, never fabricated as a
recovery/update checkpoint. The fallback `complete` constructor is the sole
source of its two typed values.

RecoveryInvocationBootstrapTemplateCore = (
  PrimaryBundleLoadBound,EmergencyTemplateFallbackBound,
  CurrentScheduleStateIngressBound,TakeoverProofAndCandidateCasAttemptBound,
  PreFenceReobservationAndReplayValidationBound,
  FencePreparationAndCasAttemptBound,TailAcquisitionPreparationAndCasAttemptBound
).

RecoveryInvocationBootstrapTemplateCoreId = Identity(
  recovery-invocation-bootstrap-template-core-domain,
  RecoveryInvocationBootstrapTemplateCore).

RecoveryInvocationBootstrapCompletionStatement = (
  RecoveryInvocationBootstrapTemplateCoreId,
  ExactSequentialPrimaryFallbackScheduleIngressTakeoverAndPrefenceStrategyId,
  PairwiseDisjointnessFromRecoveryBudgetPartitionStatementId,
  AffineDispositionForEveryUntakenBranchStatementId
).

RecoveryInvocationBootstrapCompletionStatementId = Identity(
  recovery-invocation-bootstrap-completion-domain,
  RecoveryInvocationBootstrapCompletionStatement).

RecoveryInvocationBootstrapCompletionWarrant = (
  RecoveryInvocationBootstrapCompletionStatement,
  RecoveryInvocationBootstrapCompletionStatementId,
  ProofId,VerifierResult=accept(VerifiedStatementIds containing
                                RecoveryInvocationBootstrapCompletionStatementId)
).

RecoveryInvocationBootstrapCompletionWarrantId = Identity(
  recovery-invocation-bootstrap-completion-warrant-domain,
  RecoveryInvocationBootstrapCompletionWarrant).

RecoveryInvocationBootstrapTemplate = (
  RecoveryInvocationBootstrapTemplateCore,
  RecoveryInvocationBootstrapTemplateCoreId,
  RecoveryInvocationBootstrapCompletionStatementId
).

RecoveryInvocationBootstrapTemplateId = Identity(
  recovery-invocation-bootstrap-template-domain,
  RecoveryInvocationBootstrapTemplate).

RecoveryBootstrapAllocationReceipt = (
  RecoveryInvocationAttemptId,TrustedIssuerEpoch,
  RecoveryCoordinatorIngressProfileId,RecoveryResourceContractId,
  RecoveryInvocationBootstrapTemplateId,
  OrderedFreshCapabilityIds,IssuerFreshnessAndDisjointnessProofId
).

The receipt excludes the final bootstrap-partition wrapper/root. It is therefore
acyclic even though the partition embeds it.

RecoveryBootstrapAllocationStatement = (
  RecoveryInvocationAttemptId,RecoveryInvocationBootstrapTemplateCoreId,
  OrderedFreshCapabilityIds,
  ExactCapabilityBoundsAndPairwiseDisjointnessStatementId,
  ExactSequentialCompletionAndAffineDispositionStatementId
).

RecoveryBootstrapAllocationStatementId = Identity(
  recovery-bootstrap-allocation-statement-domain,
  RecoveryBootstrapAllocationStatement).

RecoveryBootstrapAllocationWarrant = (
  RecoveryBootstrapAllocationStatement,
  RecoveryBootstrapAllocationStatementId,
  ProofId,VerifierResult=accept(VerifiedStatementIds containing
                                RecoveryBootstrapAllocationStatementId)
).

RecoveryBootstrapAllocationWarrantId = Identity(
  recovery-bootstrap-allocation-warrant-domain,
  RecoveryBootstrapAllocationWarrant).

RecoveryInvocationBootstrapPartition = (
  RecoveryInvocationAttemptId,
  PrimaryBundleLoadSlice,EmergencyTemplateFallbackSlice,
  CurrentScheduleStateIngressSlice,TakeoverAttemptPartition,
  PreFenceReobservationAndReplayValidationSlice,
  FencePreparationAndCasAttemptSlice,
  TailAcquisitionPreparationAndCasAttemptSlice,
  RecoveryBootstrapAllocationReceipt,
  RecoveryBootstrapAllocationWarrantId
).

RecoveryInvocationBootstrapPartitionRoot = Identity(
  recovery-invocation-bootstrap-partition-domain,
  RecoveryInvocationBootstrapPartition).

RecoveryBootstrapCapabilityState = live | consumed | disposed.

RecoveryBootstrapCapabilityTag =
    primary-bundle-load
  | emergency-template-fallback
  | current-schedule-state-ingress
  | takeover-proof
  | takeover-candidate-cas
  | pre-fence-reobservation
  | fence-preparation-cas
  | tail-acquisition-preparation-cas.

RecoveryBootstrapNoActionReason =
    primary-complete-no-fallback
  | fresh-entry-no-takeover
  | recovering-entry-no-fresh-fence
  | same-owner-no-takeover
  | active-other-owner-no-takeover.

RecoveryBootstrapRemainderState = (
  RecoveryInvocationAttemptId,RecoveryInvocationBootstrapPartitionRoot,
  PrimaryBundleLoadState,EmergencyTemplateFallbackState,
  CurrentScheduleStateIngressState,TakeoverProofState,
  TakeoverCandidateCasState,PreFenceReobservationState,
  FencePreparationCasState,TailAcquisitionPreparationCasState
).

RecoveryBootstrapRemainderStateRoot = Identity(
  recovery-bootstrap-remainder-state-domain,RecoveryBootstrapRemainderState).

SingleUseCheckedOutCapability = checked-out-bootstrap-capability(
  RecoveryInvocationAttemptId,RecoveryInvocationBootstrapPartitionRoot,
  RecoveryBootstrapCapabilityTag,UnderlyingCapabilityId,
  BeforeRecoveryBootstrapRemainderStateRoot,
  AfterRecoveryBootstrapRemainderStateRoot).

CheckedOutRecoveryBootstrapCapability = (
  RecoveryInvocationAttemptId,RecoveryInvocationBootstrapPartitionRoot,
  RecoveryBootstrapCapabilityTag,UnderlyingCapabilityId,
  BeforeRecoveryBootstrapRemainderStateRoot,
  AfterRecoveryBootstrapRemainderStateRoot,
  SingleUseCheckedOutCapability
).

RecoveryBootstrapTakeResult =
    checked-out(CheckedOutRecoveryBootstrapCapability,
                RecoveryBootstrapRemainderState,
                RecoveryBootstrapRemainderStateRoot)
  | warrant-violation(FailureReport).

RecoveryBootstrapStateTransitionResult =
    advanced(RecoveryBootstrapRemainderState,
             RecoveryBootstrapRemainderStateRoot)
  | warrant-violation(FailureReport).

InitialRecoveryBootstrapRemainderState(partition) is the unique state whose
attempt/root equal that exact partition and whose eight displayed state fields
are `live` in displayed field order.

RecoveryBootstrapDispositionProof = (
  RecoveryInvocationAttemptId,RecoveryInvocationBootstrapPartitionRoot,
  InitialRecoveryBootstrapRemainderStateRoot,
  FinalRecoveryBootstrapRemainderStateRoot,
  FinalStateContainsNoLiveCapability=true,
  NoBootstrapCapabilityEscapeOrReconstruction=true
).

`ConsumeRecoveryBootstrapCapability(state,partition,tag,capability)` is a total
linear checkout transition returning `RecoveryBootstrapTakeResult`. It requires
the tagged state field to be `live`, equality-checks the capability against the
exact tagged partition projection, changes only that field to `consumed`, and
returns the unique successor plus one `CheckedOutRecoveryBootstrapCapability`.
Only that checked-out value—not the original partition projection—may be passed
to the immediately associated bounded helper, which consumes it exactly once.
The embedded primitive affine value binds the exact before/after state roots;
an immutable state, partition, or checkout tuple cannot reconstruct it.
`DisposeRecoveryBootstrapCapability(state,partition,tag,capability,
reason:RecoveryBootstrapNoActionReason)` instead changes the same live field to
`disposed` and returns `RecoveryBootstrapStateTransitionResult`; it exposes no
capability. A non-live or mismatched input returns a typed bootstrap-disposition
warrant violation. The Appendix assigns every successor before matching or
returning from the associated operation.

The initial remainder state maps every capability in the displayed bootstrap
partition—including both takeover capabilities—to `live`. Passing one exact
slice to its named bounded helper changes only that entry to `consumed`; an
explicit affine no-action close changes it to `disposed`. A consumed or disposed
entry cannot become live. `CloseRecoveryBootstrapRemainderAndReturn` validates
the partition root and allocation warrant, disposes every still-live entry,
constructs the displayed exact proof, and returns the supplied typed
`RecoveryResult`. If the state or warrant is invalid it returns only
`internal-failure(FailureReport(
recovery-bootstrap-affine-disposition-warrant-violation,report))` and exposes no
capability. Every helper invocation
or explicit disposal using a bootstrap projection is a linear state transition
on this remainder state; bare immutable partition data never recreates a slice.

TakeoverAttemptPartition = (
  TakeoverProofSlice,TakeoverCandidateAndCasAttemptSlice
).

RecoveryResourceContractHeader = (
  RecoveryResourceContractId,
  FixedHeaderGrammarAndMaximumSize,
  RecoveryBundleObjectLocatorGrammar,
  RecoveryInvocationBootstrapTemplate,
  RecoveryInvocationBootstrapCompletionWarrantId
).

`RecoveryResourceContractId` is resolved by bounded coordinator ingress as a
fixed-size header without loading the recovery bundle. The trusted issuer derives
one fresh `RecoveryInvocationBootstrapPartition` from the template and allocated
attempt identity. Its capability tokens are unique to that attempt; resolving
the same immutable header later cannot recreate or consume them. The accepted
completion warrant proves the exact sequential strategy: the primary slice loads
and verifies the complete `ResolvedRecoveryExecutionMaterial` control graph;
on any noncomplete primary outcome the independent fallback slice loads and
verifies that same typed graph plus its safe-quarantine profile; the schedule
ingress slice then resolves the complete retained `RecoveryWorkObjectGraph`
(when recovering), including every transitive processed/pending/prior-output/
progress object, and all five current content-addressed schedule-state bodies
before any next token or bundle-acquisition slice is exposed. The warrant binds
a finite maximum for that graph and those canonical bodies.
For an already-recovering head, the attempt's `TakeoverProofSlice` proves and
verifies the exact prior-owner quiescence/takeover statement before a resume
token is consumed; on a fresh-fence, coherent same-owner, or pre-proof return it
is affinely disposed with a no-action proof. The
`PreFenceReobservationAndReplayValidationSlice` funds the later atomic re-read
and any conditional winning-tuple validation after bundle work. The
`TakeoverCandidateAndCasAttemptSlice` stores the accepted evidence/candidate and
attempts the lease CAS. A losing contender consumes no persisted resume token;
the token is winner-consumed atomically with that CAS.
`FencePreparationAndCasAttemptSlice` funds origin/work object construction and
the caller's compare-and-swap attempt; it does not consume the shared persisted
fence permit on a losing compare.
`TailAcquisitionPreparationAndCasAttemptSlice` funds selection validation,
candidate construction, the tail-acquisition CAS attempt, and classified loss;
the persisted tail key is winner-consumed only. These eight attempt slices are pairwise
disjoint and separate from every `RecoveryBudgetPartition`
slice. A conforming
bootstrap emits no checkpoint, performs no ambient transitive fetch, never asks
a hidden token to pay for resolving the object that contains it, and never
reuses a spent capability. Every early-return path affinely closes its unused
attempt capabilities.

RecoveryRequestTemplate = (
  DeploymentLineageId,
  ExpectedReservedHeadRef,
  OriginalExecutionTransactionId,
  EffectIntentLedgerRoot,
  RecoveryPolicyId,RecoveryPolicyCoreId,
  RecoveryResourceContractId
)

BuildRecoveryRequestTemplate(running r) = (
  r.ExpectedDeploymentConfiguration.DeploymentLineageId,
  ExpectedReservedHeadRef(
    r.ExpectedDeploymentConfiguration.DeploymentLineageId,
    Identity(execution-reservation-domain,r),r.ReservationStateRoot),
  r.ExecutionTransactionId,
  r.EffectIntentLedgerRoot, r.RecoveryPolicyId,r.RecoveryPolicyCoreId,
  r.RecoveryResourceContractId
).

BuildRecoveryRequestTemplate(recovering r)
  = the exact RecoveryRequestTemplate committed by r.RecoveryOrigin;
    it is never reconstructed from descendant effect/work roots.

MaterializeRecoveryRequest(template)
  = prepend Identity(recovery-transaction-domain,template) to template.

ExecutionReservationId
  = Identity(execution-reservation-domain, ExecutionReservation).

FinalizedRecoveryResult =
    recovered(FinalizedExecutionResult,RecoveryReceipt).

RecoveryResult =
    recovered(FinalizedExecutionResult, RecoveryReceipt)
  | duplicate(OriginalRecoveryResultId, FinalizedRecoveryResult)
  | checkpoint(RecoveryCheckpoint, ObligationIds)
  | rejected(Reason)
  | incoherent(ConflictProof)
  | conflict(CurrentDeploymentHeadState)
  | unresolved(DependencyIds)
  | unsupported(FeatureIds)
  | resource-exhausted(ResourceReport)
  | internal-failure(FailureReport).

ExactTwoKeyHeadObservationBody = (
  OriginalTransactionKey, OriginalTransactionLedgerEntry?,
  RecoveryTransactionKey, RecoveryTransactionLedgerEntry?,
  DeploymentLineageId, DeploymentHeadState,
  CoordinatorObservationVersion
).

ExactTwoKeyHeadObservationId = Identity(
  recovery-two-key-head-observation-domain,
  ExactTwoKeyHeadObservationBody).

ExactTwoKeyHeadObservation = (
  ExactTwoKeyHeadObservationBody,ExactTwoKeyHeadObservationId
).

RecoveryFinalizeResult =
    committed(FinalizedRecoveryResult)
  | same-body-winner(OriginalRecoveryResultId,FinalizedRecoveryResult)
  | normal-execution-winner(CurrentDeploymentHeadState)
  | identity-reuse(Reason)
  | conflict(CurrentDeploymentHeadState)
  | integrity-failure(FailureReport)
  | warrant-violation(FailureReport,ExactTwoKeyHeadObservation).

RecoveryReservationMutationResult =
    committed(ExecutionReservation)
  | same-body-winner(OriginalRecoveryResultId,FinalizedRecoveryResult)
  | normal-execution-winner(CurrentDeploymentHeadState)
  | identity-reuse(Reason)
  | conflict(CurrentDeploymentHeadState)
  | integrity-failure(FailureReport)
  | warrant-violation(FailureReport,ExactTwoKeyHeadObservation).

RecoveryCheckpointPublicationResult =
    committed(ExecutionReservation,RecoveryCheckpoint)
  | same-body-winner(OriginalRecoveryResultId,FinalizedRecoveryResult)
  | normal-execution-winner(CurrentDeploymentHeadState)
  | identity-reuse(Reason)
  | conflict(CurrentDeploymentHeadState)
  | integrity-failure(FailureReport)
  | warrant-violation(FailureReport,ExactTwoKeyHeadObservation).

Every fresh-fence, resume/takeover, stage-attempt, stage-advance, and tail-attempt
reservation CAS returns its closed result algebra with the exact
`RecoveryReservationMutationResult` loss constructors; checkpoint publication returns the
same classified loss constructors through
`RecoveryCheckpointPublicationResult`, whose committed branch also carries the
already built immutable checkpoint. On loss its bounded
helper makes one atomic two-key-and-head observation and, within that operation's
attempt slice, validates a same-body recovery winner, a normally finalized
execution, an identity collision, a same-origin descendant, or an impossible
torn tuple. Only `committed` consumes the persisted winner-only permit/token;
a losing attempt cannot spend it. Thus no recovery mutation caller reads an
ambient current head or overlooks a terminal ledger winner racing the CAS.

`BoundedBuildEntriesAndAtomicallyFinalizeRecovery` returns exactly
`RecoveryFinalizeResult`.
Only `committed` performed this attempt's two-key/head transition. On compare-
and-swap loss the helper uses its one exact atomic observation and the mutually
exclusive loss-validation portion of the same tail publication capability to
validate the complete winning recovery tuple, a normally finalized original
execution, identity reuse, or a same-origin descendant. It returns the matching
classified constructor; no caller re-resolves a result/receipt, reads a free
ledger entry, or performs winner validation after that capability is consumed.
`same-body-winner` contains only a fully validated immutable recovered tuple;
any one-sided entry, mismatched body/result/receipt/successor, or impossible
reservation coexistence is `integrity-failure`. `warrant-violation` retains the
exact reservation and observation and denotes a contradiction of the admitted
noncheckpointing tail-completion/sufficiency warrant, not an unbounded retry.

RecoveryCheckpointBody = (
  RecoveryTransactionId, DeploymentLineageId,
  RecoveryOriginRoot, CurrentRecoveringReservationRoot,
  RecoverySubstage, ProcessedWorkRoot, PendingWorkRoot,
  ImmutablePriorOutputRoots,
  RemainingStageResourceStateRoot,
  RemainingStageAdvancePublicationScheduleRoot,
  RemainingCheckpointPublicationScheduleRoot,
  ProgressMeasureAndWarrant
)

RecoveryCheckpointCandidate = (
  RecoveryTransactionId, RecoveryOriginRoot, RecoverySubstage,
  ResolvedRecoveryWorkObjectGraph,
  CheckpointScopeAndStrictProgressProof
).

Every bounded recovery-stage `checkpoint(P,ids)` carries exactly a
`RecoveryCheckpointCandidate`. `P.ProcessedWorkRoot`, `P.PendingWorkRoot`,
`P.ImmutablePriorOutputRoots`, the three remaining schedule roots, and
`P.ProgressMeasureAndWarrant` are defined projections of
`P.ResolvedRecoveryWorkObjectGraph.RecoveryWorkBody`. The corresponding
`P.ExactProcessedWorkObjects`, `P.ExactPendingWorkObjects`,
`P.ExactImmutablePriorOutputObjects`, and
`P.ExactProgressMeasureAndWarrantObject` are projections of that same graph.
Thus checkpoint publication never resolves an ambient root or invents a
preimage under its CAS-only token.

RecoveryWorkBody = (
  RecoveryTransactionId, RecoveryOriginRoot, RecoverySubstage,
  ProcessedWorkRoot, PendingWorkRoot, ImmutablePriorOutputRoots,
  RemainingStageResourceStateRoot,
  RemainingStageAdvancePublicationScheduleRoot,
  RemainingCheckpointPublicationScheduleRoot,
  ProgressMeasureAndWarrant
)

RecoveryWorkRoot = Identity(recovery-work-domain,RecoveryWorkBody).

RecoveryWorkPayload = (
  CanonicalProcessedWorkObjectGraph,
  CanonicalPendingWorkObjectGraph,
  CanonicalImmutablePriorOutputObjectGraph,
  ProgressMeasureAndWarrantObject,
  TransitiveRecoveryWorkDependencyGraphRoot
).

RecoveryWorkObjectGraph = (RecoveryWorkBody,RecoveryWorkPayload).

RecoveryWorkObjectGraphRoot = Identity(
  recovery-work-object-graph-domain,RecoveryWorkObjectGraph).

RecoveryWorkObjectId = Identity(
  recovery-work-object-domain,RecoveryWorkObjectGraphRoot).

RecoveryWorkObjectGraphRetentionStatement = (
  RecoveryWorkRoot,RecoveryWorkObjectGraphRoot,RecoveryWorkObjectId,
  RequiredCheckpointAndReservationLifetime,DurableFaultDomain,
  ImmutableObjectLocatorAndReplicationProfileId
).

RecoveryWorkObjectGraphRetentionStatementId = Identity(
  recovery-work-object-graph-retention-domain,
  RecoveryWorkObjectGraphRetentionStatement).

RecoveryWorkObjectGraphRetentionWarrant = (
  RecoveryWorkObjectGraphRetentionStatement,
  RecoveryWorkObjectGraphRetentionStatementId,
  ProofId,
  VerifierResult=accept(VerifiedStatementIds containing
                        RecoveryWorkObjectGraphRetentionStatementId)
).

RecoveryWorkObjectGraphRetentionWarrantId = Identity(
  recovery-work-object-graph-retention-warrant-domain,
  RecoveryWorkObjectGraphRetentionWarrant).

RecoveryWorkObjectGraphRetentionWarrantObjectId is the typed immutable-object
identifier resolving that exact warrant.

ResolvedRecoveryWorkObjectGraph = (
  RecoveryWorkObjectGraph,RecoveryWorkObjectGraphRoot,RecoveryWorkObjectId,
  RecoveryWorkObjectGraphRetentionWarrant,
  RecoveryWorkObjectGraphRetentionWarrantId,
  RecoveryWorkObjectGraphRetentionWarrantObjectId
).

The roots in `RecoveryWorkBody` MUST equal the identities of the corresponding
processed, pending, prior-output, and progress objects in this graph. Strict
resolution of `RecoveryWorkObjectId` yields the entire graph, not only the small
work-body tuple, and rederives both graph and work roots. Every stage advance,
stage-attempt acquisition, and checkpoint atomically stores the new graph and an
accepted retention warrant before publishing its references. An unavailable or
root-mismatched transitive work object makes resume an integrity failure.

RecoveryStageCursor =
    fresh(RecoveryWorkBody)
  | resume(RecoveryWorkBody).

RecoveryStageCompletion = (
  CompletedRecoverySubstage, ProcessedWorkRoot, PendingWorkRoot,
  NewlyCommittedImmutableOutputRoots,
  SuccessorRecoveryWorkPayload : RecoveryWorkPayload,
  RemainingStageResourceStateRoot,
  RemainingStageAdvancePublicationScheduleRoot,
  RemainingCheckpointPublicationScheduleRoot,
  ProgressMeasureAndWarrant
).

RecoveryStageFailure =
    rejected(Reason)
  | incoherent(ConflictProof)
  | unresolved(DependencyIds)
  | unsupported(FeatureIds)
  | resource-exhausted(ResourceReport)
  | internal-failure(FailureReport).

RecoveryQuiescenceResult =
    complete(DescendantReservation,QuiescenceProof,RecoveryStageCompletion)
  | checkpoint(RecoveryCheckpointCandidate,ObligationIds)
  | conflict(CurrentDeploymentHeadState)
  | failure(RecoveryStageFailure).

OriginalExecutionIdentityBinding = (
  ExactOriginalExecutionRequestBody,OriginalExecutionRequestBodyRoot,
  OriginalExecutionRequestBodyObjectId,OriginalExecutionSubject,
  OriginalInvocationOrWorkloadInputOrContinuationIdentity,
  OriginalAnswerIdentity,OriginalCertificateId,
  OriginalSelectedSystemOrPolicyId,
  OriginalObjectSetRootAndRetentionWarrantId
).

RecoveryValidationInput =
    validated(OriginalExecutionIdentityBinding,
              RecoveryValidationEvidence,ValidatedOriginalObjects)
  | failed-validation(OriginalExecutionIdentityBinding,
                      RecoveryStageFailure,FailedValidationEvidence).

RecoveryValidationInputRoot = Identity(
  recovery-validation-input-domain,RecoveryValidationInput).

Both constructors carry the exact identity binding available from the retained
original-object graph even when semantic validation fails. A resumed
reconciliation/outcome stage restores this tagged value and can therefore build
the exact recovery runtime statement without a local
`validatedOriginalObjects` variable. The request-body object ID strictly
resolves `ExactOriginalExecutionRequestBody`, its displayed root rederives, and
both are projections covered by the same original-object-graph retention
warrant; a naked request root is not sufficient to rebuild the companion ledger
entry.

RecoveryValidationResult =
    complete(OriginalExecutionIdentityBinding,
             RecoveryValidationEvidence,ValidatedOriginalObjects,
             RecoveryStageCompletion)
  | checkpoint(RecoveryCheckpointCandidate,ObligationIds)
  | failure(RecoveryStageFailure,OriginalExecutionIdentityBinding,
            FailedValidationEvidence,
            RecoveryStageCompletion).

RecoveryReconciliationResult =
    complete(ReconstructedEvent,SuccessorDeploymentConfiguration,
             DescendantReservation,RecoveryStageCompletion)
  | exact-counterexample(CounterexampleProof,ReconstructedViolationEvent,
                         QuarantinedSuccessorDeploymentConfiguration,
                         DescendantReservation,RecoveryStageCompletion)
  | failure(RecoveryStageFailure,ReconstructedInconclusiveEvent,
            QuarantinedPartialSuccessorDeploymentConfiguration,
            DescendantReservation,RecoveryStageCompletion)
  | checkpoint(RecoveryCheckpointCandidate,ObligationIds,
               DescendantReservation)
  | same-body-winner(OriginalRecoveryResultId,FinalizedRecoveryResult)
  | normal-execution-winner(CurrentDeploymentHeadState)
  | identity-reuse(Reason)
  | integrity-failure(FailureReport)
  | warrant-violation(FailureReport,ExactTwoKeyHeadObservation)
  | conflict(CurrentDeploymentHeadState).

The three bounded recovery helpers return exactly these algebras. Every
`RecoveryStageCompletion` carries the complete successor work payload required
for crash-safe advance; a failure constructor is data, not an untyped early
return.

`LiftRecoveryStageFailure` is the total projection from
`RecoveryStageFailure` to the identically tagged `RecoveryResult` branch:
`rejected`, `incoherent`, `unresolved`, `unsupported`, `resource-exhausted`, or
`internal-failure`. It performs no publication. A helper match MUST first bind
the single value `failure(stageFailure,...)`; prose such as “the matching exact
failure” is never a free constructor or an unbound variable.

RecoveryCursorAdvanceResult =
    complete(DescendantReservation,RecoveryStageCursor,
             StageAdvancePublicationScheduleState,
             ResolvedRecoveryWorkObjectGraph)
  | same-body-winner(OriginalRecoveryResultId,FinalizedRecoveryResult)
  | normal-execution-winner(CurrentDeploymentHeadState)
  | identity-reuse(Reason)
  | integrity-failure(FailureReport)
  | warrant-violation(FailureReport,ExactTwoKeyHeadObservation)
  | rejected(Reason)
  | incoherent(ConflictProof)
  | conflict(CurrentDeploymentHeadState)
  | unresolved(DependencyIds)
  | unsupported(FeatureIds)
  | resource-exhausted(ResourceReport)
  | internal-failure(FailureReport).

AdvanceRecoveryCursor(Request,currentReservation,recoveryOrigin,currentCursor,
                      currentResolvedRecoveryWorkObjectGraph,
                      validatedCurrentStageAdvanceScheduleState,
                      completion,nextSubstage,
                      StageAdvancePreparationAndCasAttemptSlice)
  validates that `completion.CompletedRecoverySubstage` is the cursor's current
  substage; the supplied current resolved work graph's body/root/object/warrant
  references equal `currentReservation` and `currentCursor`; the next tag is its
  unique normative successor; all prior immutable
  outputs are preserved, `completion.SuccessorRecoveryWorkPayload` contains the
  exact processed/pending/prior-output/progress preimages and matches every
  displayed root, all three remaining resource/schedule roots are the
  exact monotone successor states returned by that completed stage, and progress
  is strict. It requires the supplied ingress-validated/current successor map's
  identity equal the cursor's remaining advance-schedule root. It derives
  `StageAdvancePublicationKey` from the completed
  substage, accepted progress ordinal, current serialized recovery fault count,
  and next advance ordinal; consumes that
  exact branch-indexed token from the resolved
  `RemainingStageAdvancePublicationScheduleRoot`; atomically stores the
  successor map/consumed-set body under its derived root; builds and stores the
  next `RecoveryWorkObjectGraph` and accepted retention warrant, and CASes
  `currentReservation` to the
  same-attempt/epoch descendant naming that work body and the remaining advance
  schedule. That CAS also reconstructs `RecoveryInvocationLease` with the new
  `RecoveryWorkRoot` while preserving its attempt, epoch, origin, attempt-fence
  token, and takeover statement; a lease carrying the old work root is not a
  valid descendant, and clears `RecoveryStageAttemptMarker`.
  It returns exactly `RecoveryCursorAdvanceResult`; success is
  `complete(descendant,resume(nextBody),successorScheduleState,
  successorResolvedWorkGraph)`. A CAS loss is classified from one atomic
  two-key/head observation into the displayed same-body winner, normal winner,
  identity reuse, conflict, integrity, or warrant branch; validation failure is
  mapped to the displayed rejected/incoherent/failure branch. No completed stage's resource or effect
  state is usable by its successor until this CAS succeeds, so a crash resumes
  the last committed program counter rather than replaying an uncommitted
completion. The returned successor graph references MUST equal those atomically
  published by `descendant` and the returned cursor; an unrelated retained graph
  cannot authorize or satisfy an advance.

`RecoveryStageCursor(cursor,expectedSubstage)` is well-typed only when the fresh
or resumed body's substage equals `expectedSubstage`; all other uses are rejected
rather than coerced or restarted. A bare substage tag is never a cursor and
cannot authorize resource use, checkpoint publication, or resumption.

RecoveryStageAttemptAcquisitionResult =
    acquired(DescendantReservation,RecoveryStageAttemptBundle,
             RecoveryStageCursor,RecoveryStageAttemptScheduleState,
             ResolvedRecoveryWorkObjectGraph,
             MaterializedRecoveryEffectProtocolSchedule?)
  | exhausted-within-declared-bound(SufficiencyWarrantId)
  | declared-fault-bound-exceeded(ObservedFaultCount,DeclaredBound)
  | same-body-winner(OriginalRecoveryResultId,FinalizedRecoveryResult)
  | normal-execution-winner(CurrentDeploymentHeadState)
  | identity-reuse(Reason)
  | integrity-failure(FailureReport)
  | warrant-violation(FailureReport,ExactTwoKeyHeadObservation)
  | rejected(Reason)
  | incoherent(ConflictProof)
  | conflict(CurrentDeploymentHeadState)
  | unresolved(DependencyIds)
  | unsupported(FeatureIds)
  | resource-exhausted(ResourceReport)
  | internal-failure(FailureReport).

AcquireRecoveryStageAttempt(Request,currentReservation,cursor,
                            currentResolvedRecoveryWorkObjectGraph,
                            validatedCurrentStageAttemptScheduleState,substage,
                            DeclaredCrashAndTakeoverFaultBound)
  requires the supplied resolved work graph's body/root/object/warrant references
  equal the current reservation and cursor, and the supplied ingress-validated
  schedule state's identity equal
  `cursor.RecoveryWorkBody.RemainingStageResourceStateRoot`; derives the current
  `RecoveryProgressOrdinal` from the cursor's accepted progress warrant; selects
  the unique unconsumed ordinal key; resolves the complete embedded bundle/value;
  requires its exact substage, progress ordinal, root, and accepted
  completion/disjointness statement; then in one CAS stores and publishes the
  advanced schedule state, stores the updated same-substage work graph and
  accepted retention warrant,
  updates the lease's work root, and installs `RecoveryStageAttemptMarker` for
  the current attempt/epoch/token/bundle plus the accepted progress ordinal and
  actual progress-state root. Only `acquired` exposes `StageWorkSlice`
  or `EmergencyCheckpointSlice`; all other branches expose no stage resource.
  Completion/advance or checkpoint publication clears the marker. A takeover
  abandons it, increments the persisted fault count, and can acquire only a fresh
  bundle.
  A losing CAS consumes no selected stage token and classifies its one exact
  two-key/head observation into the displayed winner/conflict/integrity
  constructors before returning.
  In `acquired`, the cursor MUST be `resume(the exact RecoveryWorkBody stored by
  DescendantReservation)` and the returned `ResolvedRecoveryWorkObjectGraph`
  MUST be the exact graph stored by that descendant; no other cursor constructor,
  body, or ambient work preimage is permitted.
  The optional materialized effect schedule is present exactly for
  `effect-reconciliation` and is the deterministic projection bound to the
  descendant's committed attempt, epoch, fence, and stage marker. Every effect
  statement, intent, and permit binds its selected materialized token identity.

RecoveryCheckpointRoot
  = Identity(recovery-checkpoint-domain, RecoveryCheckpointBody).

RecoveryCheckpoint = (RecoveryCheckpointBody, RecoveryCheckpointRoot).

BuildRecoveryCheckpoint(Request,recoveringReservation,recoveryWorkBody)
  = let body = RecoveryCheckpointBody(
      Request.RecoveryTransactionId,
      Request.DeploymentLineageId,
      Root(recoveringReservation.RecoveryOrigin),
      Root(recoveringReservation),
      recoveryWorkBody.RecoverySubstage,
      recoveryWorkBody.ProcessedWorkRoot,
      recoveryWorkBody.PendingWorkRoot,
      recoveryWorkBody.ImmutablePriorOutputRoots,
      recoveryWorkBody.RemainingStageResourceStateRoot,
      recoveryWorkBody.RemainingStageAdvancePublicationScheduleRoot,
      recoveryWorkBody.RemainingCheckpointPublicationScheduleRoot,
      recoveryWorkBody.ProgressMeasureAndWarrant);
    (body, Identity(recovery-checkpoint-domain,body)).

RecoverySubstage = fence-quiescence | input-validation |
                   effect-reconciliation | outcome-staging.

Runtime conformance and terminal finalization are the noncheckpointing tail of
one pre-acquired `RecoveryTailAttemptBundle`; they are not recovery substages and
do not mutate the reserved head between tail acquisition and the terminal CAS.
An external checkpoint tagged as either is malformed.

A `checkpoint` recovery result does not finalize `recovery(RecoveryTransactionId)`
in the transaction ledger. It is returned only after the exact recovering
descendant and `RecoveryWorkRoot` have been committed. Replaying the same body
therefore resumes that descendant; only a terminal two-key finalization creates
the immutable recovery ledger entry.

RecoveryCommitPreCore = (
  RecoveryRequestBodyRoot, PriorRecoveringReservationRoot,
  CompanionOriginalTransactionKey,
  SuccessorDeploymentId,
  ExpectedBeforeHeadRoot, ExpectedAfterHeadRoot,
  CoordinatorAtomicityProfileId,
  RecoveryReconciliationEvidenceRoot
)

BuildPublishedRecoveryCommitPreCore(evaluatedPreCore,publishedSuccessor,
                                    priorReservation,publishedEvidenceRoot)
  copies the request-body root, prior-reservation root, companion key, and
  coordinator-atomicity profile exactly; sets `SuccessorDeploymentId` and
  `ExpectedAfterHeadRoot` from `publishedSuccessor`; rechecks
  `ExpectedBeforeHeadRoot=Root(reserved(priorReservation))`; and sets the exact
  published reconciliation/transition evidence root. No other field may be
  inherited from an evaluated successor that will not be published.

RecoveryCommitCore = (
  RecoveryCommitPreCore,
  CompanionOriginalTransactionLedgerEntryId,
  OriginalExecutionResultId, OriginalExecutionReceiptLikeId
)

OriginalExecutionReceiptLikeId =
    terminal(ReceiptTypeTag,ReceiptId)
  | productive(ReceiptTypeTag,ReceiptId)
  | partial(ReceiptTypeTag,ReceiptId)
  | violation(ReceiptTypeTag,ReceiptId).

RecoveryReceiptBody = (RecoveryCommitCore)
RecoveryReceiptId = Identity(recovery-receipt-domain,RecoveryReceiptBody)
RecoveryReceipt = (RecoveryReceiptBody,RecoveryReceiptId).

`RecoveryCommitPreCore` is the exact object that runtime conformance checks. It
contains no runtime-verification record, final execution result/receipt, companion
ledger-entry identity, recovery receipt, or recovery result. After conformance,
the original final execution receipt/result and its ledger entry are constructed;
their identities extend the pre-core to `RecoveryCommitCore`. The recovery
receipt is then constructed, `RecoveryResult` embeds it, and only then is the
recovery ledger entry identified. `ValidateCommittedRecoveryTupleFromLedger`
checks the receipt's immutable commit core and both companion entries; it never
infers atomicity from one key or from a later live head. This ordering is
acyclic.

`FinalizedRecoveryResult` is inhabited only when the embedded original is one
of the four receipt-bearing `FinalizedExecutionResult` branches, the nonoptional
`OriginalExecutionReceiptLikeId` selects that same branch and exact receipt,
and the original result ID, receipt ID, and
`CompanionOriginalTransactionLedgerEntryId` all rederive from the one embedded
companion entry. `no-run`, `duplicate`, `recovery-required`, and
`integrity-failure` cannot be recovered terminal payloads. A recovery duplicate
likewise contains only the exact prior `FinalizedRecoveryResult` written by a
terminal two-key publication.

OriginalExecutionObjectSet =
    query(
      ExecutionRequestBodyRoot, QueryId, InvocationIdentity,
      AnswerIdentity, CertificateId, SelectedSystemId,
      ExecProfileId, OutcomeModelId, CompletionProfileId,
      EffectDescriptorRoot)
  | workload(
      ExecutionRequestBodyRoot, UseCaseRequestId, WorkloadRunInputId,
      AnswerIdentity, CertificateId, SelectedPolicyId,
      ExecProfileId, OutcomeModelId, CompletionProfileId,
      EffectDescriptorRoot)
  | continuation(
      ExecutionRequestBodyRoot, ContinuationId, ContinuationStateRoot,
      ContinuationStepNumber, AnswerIdentity, CertificateId,
      SelectedSystemOrPolicyId, ExecProfileId, OutcomeModelId,
      CompletionProfileId, EffectDescriptorRoot).

OriginalExecutionObjectSetRoot
  = Identity(original-execution-object-set-domain,OriginalExecutionObjectSet).

OriginalExecutionObjectGraphRetentionStatement = (
  OriginalExecutionObjectSetRoot,
  OriginalExecutionObjectSetObjectId,
  TransitiveReferencedObjectGraphRoot,
  RequiredReservationAndRecoveryLifetime,
  DurableFaultDomain,
  ImmutableObjectLocatorAndReplicationProfileId
).

OriginalExecutionObjectGraphRetentionStatementId = Identity(
  original-execution-object-graph-retention-domain,
  OriginalExecutionObjectGraphRetentionStatement).

OriginalExecutionObjectGraphRetentionWarrant = (
  OriginalExecutionObjectGraphRetentionStatement,
  OriginalExecutionObjectGraphRetentionStatementId,
  RetentionProofId,
  VerifierResult=accept(VerifiedStatementIds containing
                        OriginalExecutionObjectGraphRetentionStatementId)
).

InitialRecoveryScheduleRoots = (
  ResumeAcquisitionScheduleRoot,
  RecoveryStageAttemptScheduleRoot,
  StageAdvancePublicationScheduleRoot,
  CheckpointPublicationScheduleRoot,
  RecoveryTailAttemptScheduleRoot
).

RecoveryScheduleRoots = InitialRecoveryScheduleRoots.

ResumeStageAttemptStageAdvanceCheckpointAndTailAttemptSchedulesRoot = Identity(
  recovery-schedule-roots-domain,RecoveryScheduleRoots).

This tuple is the exact **initial** projection of the admitted immutable
`RecoveryBudgetPartition`. Policy preflight and bundle loading MUST recompute it
from those five initial schedules and equality-check the liveness statement's
field. A resume does not equate consumed current roots with this initial tuple;
it resolves the current roots from the reservation/work body and proves, under
the accepted persistence core and monotone transition relation, that their
joint state is reachable from this exact initial tuple. An opaque, unrelated,
or rolled-back current root is not a schedule sufficiency premise.

CurrentRecoveryScheduleRoots(recoveringReservation,recoveryWorkBody) = (
  recoveringReservation.RecoveryResumeAcquisitionScheduleRoot,
  recoveryWorkBody.RemainingStageResourceStateRoot,
  recoveryWorkBody.RemainingStageAdvancePublicationScheduleRoot,
  recoveryWorkBody.RemainingCheckpointPublicationScheduleRoot,
  recoveringReservation.RecoveryTailAttemptScheduleRoot
).

ResolvedCurrentRecoveryScheduleStates = (
  ResumeAcquisitionScheduleState, ResumeAcquisitionScheduleRoot,
  RecoveryStageAttemptScheduleState, RecoveryStageAttemptScheduleRoot,
  StageAdvancePublicationScheduleState, StageAdvancePublicationScheduleRoot,
  CheckpointPublicationScheduleState, CheckpointPublicationScheduleRoot,
  RecoveryTailAttemptScheduleState, RecoveryTailAttemptScheduleRoot
).

CurrentRecoveryScheduleIngressResult =
    complete(ResolvedCurrentRecoveryScheduleStates,
             ResolvedRecoveryWorkObjectGraph?)
  | checkpoint(RecoveryCheckpointCandidate,ObligationIds)
  | rejected(Reason)
  | incoherent(ConflictProof)
  | unresolved(DependencyIds)
  | unsupported(FeatureIds)
  | resource-exhausted(ResourceReport)
  | internal-failure(FailureReport).

`BoundedResolveCurrentRecoveryScheduleStates` returns exactly this algebra.

Every schedule-consuming helper receives the applicable already ingress-validated
state body from this carrier. A successful CAS returns the exact successor body
and root, and the caller replaces that component locally. No helper may fetch or
invent a successor. Each persisted schedule-state body is a content-addressed
node that commits its schedule genesis identity, optional exact parent-state
root, and consumed key/token. Trusted coordinator atomicity and the accepted
transition relation validate the one store-and-publish step that produced it.
Initial nodes have no parent. Consequently monotone reachability from
`InitialRecoveryScheduleRoots` is rederived by resolving these finite parent
chains under `RecoveryScheduleStatePersistenceWarrant`; it is not a mutable
witness field that can become stale when one component is replaced locally.
No helper may fetch or
parse the schedule/map using an acquisition slice hidden inside the bundle or
token that the unresolved schedule contains.

RecoverySchedulePersistenceCore = (
  RecoveryScheduleStorageProfileId,
  InitialRecoveryScheduleRoots,
  AdmissibleMonotoneScheduleTransitionRelationId,
  RequiredReservationAndRecoveryLifetime,
  DurableFaultDomain
).

RecoverySchedulePersistenceCoreRoot = Identity(
  recovery-schedule-persistence-core-domain,
  RecoverySchedulePersistenceCore).

RecoveryScheduleStatePersistenceStatement = (
  RecoverySchedulePersistenceCoreRoot
).

RecoveryScheduleStatePersistenceStatementId = Identity(
  recovery-schedule-state-persistence-domain,
  RecoveryScheduleStatePersistenceStatement).

RecoveryScheduleStatePersistenceWarrant = (
  RecoveryScheduleStatePersistenceStatement,
  RecoveryScheduleStatePersistenceStatementId,
  ProofId,
  VerifierResult=accept(VerifiedStatementIds containing
                        RecoveryScheduleStatePersistenceStatementId)
).

Every displayed recovery-schedule root is simultaneously the typed,
content-addressed immutable-object identifier for its exact schedule-state body
under `RecoveryScheduleStorageProfileId`; it is not a digest with an ambient
preimage. Before any CAS publishes a successor schedule root, that same atomic
operation MUST store the strictly encoded successor body under the derived root
and preserve it for the displayed lifetime/fault domain. Resume MUST resolve the
body from the root and rederive the root before selecting a token. The persistence
warrant covers the initial five bodies and every state reachable through the
displayed monotone transition relation. The enclosing partition-and-sufficiency
warrant equality-checks the core's initial roots and storage profile against the
five schedules it encloses; the persistence statement deliberately does not
contain the final partition root, avoiding a partition/warrant identity cycle.
A missing body, mismatched preimage, or
unretained successor is an integrity failure, never permission to reconstruct a
consumed set from the initial partition or to reuse a resource.

BoundedSafeTerminalLivenessStatement = (
  RecoveryPolicyCoreId, RecoveryResourceContractId, EffectCommitModelId,
  ExecutionResourcePartitionCoreRoot,
  ExactReachableRecoveryStateTransitionRelationId,
  SafeTerminalOrQuarantinedSuccessorPredicateId,
  RecoveryCheckpointProgressMeasureAndStrictDecreaseRelationId,
  ResumeStageAttemptStageAdvanceCheckpointAndTailAttemptSchedulesRoot,
  DeclaredCrashAndTakeoverFaultBound in NaturalNumber,
  FiniteStageAndAtomicPublicationBudgetSufficiencyStatementId,
  FairnessPremiseId?
).

BoundedSafeTerminalLivenessStatementId = Identity(
  bounded-safe-terminal-liveness-domain,
  BoundedSafeTerminalLivenessStatement).

BoundedSafeTerminalLivenessWarrant = (
  BoundedSafeTerminalLivenessStatement,
  BoundedSafeTerminalLivenessStatementId,
  ProofId,
  VerifierResult=accept(VerifiedStatementIds containing
                        BoundedSafeTerminalLivenessStatementId)
).

The statement quantifies every recovery state reachable under the displayed
effect model and exact resource partition. It proves that the transition either
publishes a safe terminal/quarantined successor within the reserved finite
stage and atomic-publication budgets or commits a strictly decreasing resumable
checkpoint supported by the remaining schedules. `FairnessPremiseId` is absent
for unconditional termination; when present it is a declared hypothesis of the
liveness claim, never an ambient scheduler assumption.
Every proved interruption or lease takeover that abandons the initial fresh
recovery-fence owner or an acquired resume token, stage-attempt bundle, or tail
bundle increments the exact natural-number fault counter used by
`DeclaredCrashAndTakeoverFaultBound`; the reachable-transition relation and all
schedule sufficiency proofs use that same counter.

RecoveryPolicyCore = (
  RecoveryResourceContractId,EffectCommitModelId,
  EffectAuthorizationProfileId,LeaseExpiryOrExplicitQuiescenceRule,
  UniversalFenceProtocolStatementId,
  PermittedReconcileDeduplicateCompensateRetryAndQuarantineTransitionsRoot
).

RecoveryPolicyCoreId = Identity(
  recovery-policy-core-domain,RecoveryPolicyCore).

RecoveryPolicyBody = (
  RecoveryPolicyCore,RecoveryPolicyCoreId
).

RecoveryPolicyId = Identity(recovery-policy-domain,RecoveryPolicyBody).

ResolvedRecoveryPolicy = (
  RecoveryPolicyBody,RecoveryPolicyId,RecoveryPolicyCoreId,
  ResolvedEffectAuthorizationProfile,
  ResolvedLeaseQuiescenceAndRecoveryTransitionRules
).

VerifiedRecoveryPolicyResolution = (
  ResolvedRecoveryPolicy,
  Identity=RecoveryPolicyId,
  VerifiedRecoveryBundle,
  AcceptedPolicyBundlePartitionOriginalObjectAndLivenessEvidence
).

`BoundedResolveAndVerifyRecoveryPolicy` returns
`complete(VerifiedRecoveryPolicyResolution)` or the exact bounded failure tags.
Every projected `EffectAuthorizationProfileId`, lease/quiescence rule, identity,
and verified bundle in the execution machine is a dependent projection of this
carrier; no ambient `RecoveryPolicy` record exists.

Identity construction is ordered and acyclic: first build
`RecoveryPolicyCore` and its outer generic policy ID, neither of which names a
partition or liveness instance. A fresh execution-attempt grant then builds
`ExecutionResourcePartitionCore`; the exact bounded-liveness statement is
instantiated over that partition **core** root and policy-core ID; finally build
recovery material and its bundle. Neither policy carrier names a bundle/material
or partition root, and the liveness statement never names the final partition
wrapper root. A profile that introduces any reverse edge has no conforming
identity.

RecoveryExecutionMaterial = (
  RecoveryPolicyId,RecoveryPolicyCoreId,
  RecoveryResourceContractId,
  EffectCommitModelId,
  OriginalExecutionObjectSetRoot,
  OriginalExecutionObjectSetObjectId,
  OriginalExecutionObjectGraphRetentionWarrant,
  EffectAuthorizationProfileId,
  UniversalFenceProtocolStatementId,
  BoundedSafeTerminalLivenessStatementId,
  ExecutionResourcePartitionRoot,
  ExecutionResourcePartitionObjectId,
  PartitionAndSufficiencyWarrantId
).

RecoveryExecutionMaterialRoot
  = Identity(recovery-execution-material-domain,RecoveryExecutionMaterial).

RecoveryExecutionMaterialGraphRetentionStatement = (
  RecoveryExecutionMaterialRoot,
  TransitiveRecoveryPolicyEffectModelAuthorizationProfileStatementAndPartitionGraphRoot,
  ImmutableObjectLocatorAndReplicationProfileId,
  RequiredReservationAndRecoveryLifetime,
  DurableFaultDomain
).

RecoveryExecutionMaterialGraphRetentionStatementId = Identity(
  recovery-execution-material-graph-retention-domain,
  RecoveryExecutionMaterialGraphRetentionStatement).

RecoveryExecutionMaterialGraphRetentionWarrant = (
  RecoveryExecutionMaterialGraphRetentionStatement,
  RecoveryExecutionMaterialGraphRetentionStatementId,
  RetentionProofId,
  VerifierResult=accept(VerifiedStatementIds containing
                        RecoveryExecutionMaterialGraphRetentionStatementId)
).

RecoveryExecutionMaterialGraphRetentionWarrantId = Identity(
  recovery-execution-material-graph-retention-warrant-domain,
  RecoveryExecutionMaterialGraphRetentionWarrant).

ResolvedRecoveryExecutionMaterial = (
  RecoveryExecutionMaterial,
  RecoveryExecutionMaterialRoot,
  RecoveryExecutionMaterialGraphRetentionWarrant,
  ResolvedRecoveryPolicy,
  ResolvedEffectCommitModel,
  ResolvedEffectAuthorizationProfile,
  ResolvedFenceSafetyAndBoundedLivenessStatements,
  ResolvedExecutionResourcePartition
).

ResolvedFenceSafetyAndBoundedLivenessStatements = (
  UniversalFenceProtocolWarrant,
  BoundedSafeTerminalLivenessWarrant
).

EmergencySafeQuarantineTemplateBody = (
  RecoveryExecutionMaterialRoot,
  RecoveryExecutionMaterial,
  RecoveryExecutionMaterialGraphRetentionWarrant,
  SafeQuarantineTransitionProfileId,
  EmergencyTemplateObjectLocatorAndReplicaProfileId,
  EmergencyTemplateIndependentRetentionProofId
).

EmergencySafeQuarantineTemplateRoot = Identity(
  emergency-safe-quarantine-template-domain,
  EmergencySafeQuarantineTemplateBody).

EmergencySafeQuarantineTemplate = (
  EmergencySafeQuarantineTemplateBody,
  EmergencySafeQuarantineTemplateRoot,
  EmergencySafeQuarantineTemplateId
).

EmergencySafeQuarantineTemplateRetentionStatement = (
  EmergencySafeQuarantineTemplateId,
  EmergencySafeQuarantineTemplateRoot,
  RecoveryExecutionMaterialRoot,
  EmergencyTemplateObjectLocatorAndReplicaProfileId,
  RequiredReservationAndRecoveryLifetime,
  DurableFaultDomain
).

EmergencySafeQuarantineTemplateRetentionStatementId = Identity(
  emergency-template-retention-statement-domain,
  EmergencySafeQuarantineTemplateRetentionStatement).

EmergencySafeQuarantineTemplateRetentionWarrant = (
  EmergencySafeQuarantineTemplateRetentionStatement,
  EmergencySafeQuarantineTemplateRetentionStatementId,
  ProofId,
  VerifierResult=accept(VerifiedStatementIds containing
                        EmergencySafeQuarantineTemplateRetentionStatementId)
).

EmergencySafeQuarantineTemplateRetentionWarrantId = Identity(
  emergency-template-retention-warrant-domain,
  EmergencySafeQuarantineTemplateRetentionWarrant).

EmergencySafeQuarantineTemplateRetentionWarrantObjectId is the independently
retained typed object identifier resolving that exact warrant.

`EmergencySafeQuarantineTemplateId` is the typed object identifier that resolves
exactly `EmergencySafeQuarantineTemplateBody`; its identity/equality profile MUST
bind `EmergencySafeQuarantineTemplateRoot`. The template is a self-contained,
independently retained replica of the exact recovery material plus only the
transition needed to publish a safe quarantined successor. It never depends on
loading `RecoveryBundleBody`.

RecoveryBundleBody = (
  RecoveryExecutionMaterialRoot,
  RecoveryExecutionMaterial,
  RecoveryExecutionMaterialGraphRetentionWarrant,
  EmergencySafeQuarantineTemplateId,
  EmergencySafeQuarantineTemplateRoot,
  EmergencySafeQuarantineTemplateRetentionWarrantId,
  EmergencySafeQuarantineTemplateRetentionWarrantObjectId,
  RetentionProfileId
)

RecoveryBundleRoot
  = Identity(recovery-bundle-domain, RecoveryBundleBody).

RecoveryBundle = (
  RecoveryBundleBody,
  RecoveryBundleRoot,
  RecoveryBundleObjectId,
  RecoveryBundleRetentionWarrantId,
  RecoveryBundleRetentionWarrantObjectId
).

VerifiedRecoveryBundle = (
  RecoveryBundle,
  ResolvedRecoveryExecutionMaterial,
  RecoveryBundleRetentionWarrant,
  AcceptedBundleMaterialPolicyPartitionAndRetentionEvidence
).

Its `Body`, `Root`, `ObjectId`, `RecoveryExecutionMaterialRoot`, emergency
template references, and retention-warrant references are dependent projections
of the embedded `RecoveryBundle`. Strict object resolution MUST rederive the
body/root; the resolved material MUST equal the body's exact material/root; and
the warrant accept set MUST contain the statement ID that binds this bundle,
material, emergency template, lifetime, and fault domain. These conditions are
checked before `VerifiedRecoveryPolicyResolution` can be inhabited.

RecoveryBundleRetentionStatement = (
  RecoveryBundleRoot, RecoveryBundleObjectId,
  RecoveryExecutionMaterialRoot,
  RecoveryExecutionMaterialGraphRetentionWarrantId,
  RetentionProfileId,
  DurableFaultDomain, RequiredReservationAndRecoveryLifetime,
  PrimaryAndRedundantObjectLocatorSemantics,
  EmergencySafeQuarantineTemplateId,
  EmergencySafeQuarantineTemplateRoot,
  EmergencySafeQuarantineTemplateRetentionWarrantId,
  EmergencySafeQuarantineTemplateRetentionWarrantObjectId,
  EmergencyTemplateIndependentRetentionProofId
).

RecoveryBundleRetentionStatementId
  = Identity(recovery-bundle-retention-domain,
             RecoveryBundleRetentionStatement).

RecoveryBundleRetentionWarrant = (
  RecoveryBundleRetentionStatement,
  RecoveryBundleRetentionStatementId,
  RetentionProofId,
  VerifierResult=accept(VerifiedStatementIds containing
                        RecoveryBundleRetentionStatementId)
).

RecoveryBundleRetentionWarrantId = Identity(
  recovery-bundle-retention-warrant-domain,
  RecoveryBundleRetentionWarrant).

RetainedEffectObjectKind = payload | authorization-proof-input.

EffectObjectRetentionStatement = (
  RetainedEffectObjectKind,ExactEffectObjectRoot,EffectObjectId,
  RequiredReservationAndRecoveryLifetime,DurableFaultDomain,
  ImmutableObjectLocatorAndReplicationProfileId
).

EffectObjectRetentionStatementId = Identity(
  effect-object-retention-statement-domain,EffectObjectRetentionStatement).

EffectObjectRetentionWarrant = (
  EffectObjectRetentionStatement,EffectObjectRetentionStatementId,
  ProofId,VerifierResult=accept(VerifiedStatementIds containing
                                EffectObjectRetentionStatementId)
).

EffectObjectRetentionWarrantId = Identity(
  effect-object-retention-warrant-domain,EffectObjectRetentionWarrant).

RetainedEffectObject = (
  RetainedEffectObjectKind,ExactEffectObjectBody,ExactEffectObjectRoot,
  EffectObjectId,EffectObjectRetentionWarrant,
  EffectObjectRetentionWarrantId,EffectObjectRetentionWarrantObjectId
).

For a `RetainedEffectObject`, `ExactEffectObjectRoot` is the canonical typed
identity of `ExactEffectObjectBody`, `EffectObjectId` strictly resolves that
same body, and the warrant object strictly resolves the displayed accepted
warrant for the same kind/root/object and the reservation-through-recovery
lifetime. The warrant statement excludes every proposed-effect descriptor,
intent, permit, reservation, and warrant identity, so this retention graph is
acyclic. Admission bounds both bodies; neither verifier nor sink accepts a bare
root or fetches an object outside the charged slice.

ProposedEffectDescriptor = (
  EffectTarget,RetainedEffectPayloadObject:RetainedEffectObject,
  CommitMode,DeduplicationKey,ChargedCost,EffectOutcomeAndRecoveryClass,
  SinkIdentity,SinkDeduplicationOrLeaseToken,ExpiryOrQuiescenceCondition,
  RetainedEffectAuthorizationProofInputObject:RetainedEffectObject
).

ProposedEffectDescriptorRoot = Identity(
  proposed-effect-descriptor-domain,ProposedEffectDescriptor).

The payload object's kind is `payload`; the proof-input object's kind is
`authorization-proof-input`. `ExactPayloadRoot` and
`EffectAuthorizationProofInputRoot` below are dependent projections of those
two exact retained objects, never independently supplied roots.

ReservationFenceToken =
    execution(ExecutionFenceToken)
  | recovery(RecoveryAttemptFenceToken).

EffectAuthorizationStatement = (
  ExecProfileId,RecoveryPolicyId,EffectAuthorizationProfileId,
  ProposedEffectDescriptorRoot,ReservationFenceToken,
  EffectTarget,ExactPayloadRoot,CommitMode,DeduplicationKey,
  ChargedCost,EffectOutcomeAndRecoveryClass
).

EffectAuthorizationStatementId
  = Identity(effect-authorization-domain, EffectAuthorizationStatement).

EffectAuthorizationProofBody = (
  EffectAuthorizationStatementId,EffectAuthorizationProfileId,
  ProposedEffectDescriptorRoot,EffectAuthorizationProofInputRoot,
  VerifierResult=accept(VerifiedStatementIds containing
                        EffectAuthorizationStatementId)
).

EffectAuthorizationProofId = Identity(
  accepted-effect-authorization-proof-domain,EffectAuthorizationProofBody).

EffectProtocolTokenBindingId =
    execution(MaterializedExecutionEffectProtocolTokenId)
  | recovery(RecoveryEffectProtocolTokenId).

EffectIntentBody = (
  EffectAuthorizationStatementId, EffectAuthorizationProofId,
  EffectProtocolTokenBindingId,
  ReservationFenceToken, EffectTarget, ExactPayloadRoot, CommitMode,
  DeduplicationKey
).

IntentId = Identity(effect-intent-domain, EffectIntentBody).

EffectPermitBody = (
  IntentId, EffectAuthorizationStatementId, ReservationFenceToken,
  SinkIdentity, SinkDeduplicationOrLeaseToken, ExpiryOrQuiescenceCondition
).

PermitId = Identity(effect-permit-domain, EffectPermitBody).

EffectPermit = (EffectPermitBody, PermitId).

EffectIntentEntry = (
  ProposedEffectDescriptor,
  EffectAuthorizationStatement,EffectAuthorizationStatementId,
  EffectAuthorizationProofBody,EffectAuthorizationProofId,
  EffectIntentBody,IntentId,PermitId,
  IntentStatus, ObservedEffectEvidenceRoot?
).

Every entry equality-rederives the descriptor root, statement ID, accepted proof
ID, intent ID, and permit ID. Because the descriptor embeds both retained effect
objects and their acyclic warrants, recovery resolves the complete authorized
payload/proof/sink metadata from the ledger entry itself; no naked hash or
ambient sidecar is an authorization or recovery input.

EffectAuthorizationResult =
    authorized(EffectAuthorizationStatementId,EffectAuthorizationProofBody,
               EffectAuthorizationProofId,VerifiedStatementIds)
  | exact-unauthorized(EffectAuthorizationCounterexample)
  | inconclusive(VerifierResult).

ExecutionEffectProtocolTokenTemplate = (
  EffectOrdinal,
  EffectAuthorizationSlice,
  IntentAndPermitPublicationSlice,
  SinkMembershipAndOutcomeCaptureSlice,
  StatusPublicationAndLossValidationSlice
).

ExecutionEffectProtocolTokenTemplateRoot = Identity(
  execution-effect-protocol-token-template-domain,
  ExecutionEffectProtocolTokenTemplate).

EffectProtocolSchedule = ExactFiniteOrderedFamilyOf(
  EffectOrdinal -> (
    ExecutionEffectProtocolTokenTemplate,
    ExecutionEffectProtocolTokenTemplateRoot)).

EffectProtocolScheduleRoot = Identity(
  execution-effect-protocol-template-schedule-domain,EffectProtocolSchedule).

MaterializedExecutionEffectProtocolBinding = (
  EffectProtocolScheduleRoot,
  ExecutionInvocationAttemptId,ExecutionTransactionId,
  ExecutionFenceToken,ReservationOwnershipPreCoreRoot
).

MaterializedExecutionEffectProtocolBindingRoot = Identity(
  materialized-execution-effect-protocol-binding-domain,
  MaterializedExecutionEffectProtocolBinding).

MaterializedExecutionEffectProtocolTokenBody = (
  MaterializedExecutionEffectProtocolBindingRoot,
  EffectOrdinal,ExecutionEffectProtocolTokenTemplateRoot
).

MaterializedExecutionEffectProtocolTokenId = Identity(
  materialized-execution-effect-protocol-token-domain,
  MaterializedExecutionEffectProtocolTokenBody).

MaterializedExecutionEffectProtocolToken = (
  ExecutionEffectProtocolTokenTemplate,
  MaterializedExecutionEffectProtocolTokenBody,
  MaterializedExecutionEffectProtocolTokenId
).

MaterializedExecutionEffectProtocolScheduleState = (
  EffectProtocolSchedule,
  MaterializedExecutionEffectProtocolBinding,
  NextUnconsumedEffectOrdinal,
  ConsumedEffectPrefixRoot
).

MaterializedExecutionEffectProtocolScheduleStateRoot = Identity(
  materialized-execution-effect-protocol-schedule-state-domain,
  MaterializedExecutionEffectProtocolScheduleState).

ExecutionEffectProtocolTokenSelectionResult =
    selected(MaterializedExecutionEffectProtocolToken,
             MaterializedExecutionEffectProtocolScheduleState,
             MaterializedExecutionEffectProtocolScheduleStateRoot)
  | exhausted-before-declared-run-bound(SufficiencyWarrantId)
  | malformed-schedule(Reason)
  | internal-failure(FailureReport).

LastCompletedEffectProtocolPhase =
    selected
  | authorization
  | intent-publication
  | sink-membership-and-outcome
  | status-publication.

ExactTerminalOrRecoveryOwnedReason =
    public-result(ExecutionResult)
  | finalization(
      ExecutionTransactionId,ExpectedReservationStateRoot,
      PublishedExecutionResultCoreRoot)
  | pre-intent-stop(ExactObservedEventRoot).

`ConsumedEffectPhaseSliceIds(token,phase)` and
`DisposedEffectSuffixSliceIds(token,phase)` are the exact complementary sets:

| `phase` | consumed | disposed suffix |
|---|---|---|
| `selected` | none | authorization, intent/permit, sink/outcome, status/loss |
| `authorization` | authorization | intent/permit, sink/outcome, status/loss |
| `intent-publication` | authorization, intent/permit | sink/outcome, status/loss |
| `sink-membership-and-outcome` | authorization, intent/permit, sink/outcome | status/loss |
| `status-publication` | all four | none |

Each entry denotes the exact `CapabilityId` of that field in the supplied
materialized token. Their canonical set roots are
`ConsumedEffectPhaseSliceIdSetRoot` and
`DisposedEffectSuffixSliceIdSetRoot`. The sets are disjoint, their union is the
token's four phase-slice IDs, and neither contains a slice from another token.

ConsumedEffectPhaseSliceIdSetRoot(token,phase) = Identity(
  consumed-effect-phase-slice-set-domain,
  token.MaterializedExecutionEffectProtocolTokenId,phase,
  ConsumedEffectPhaseSliceIds(token,phase)).

DisposedEffectSuffixSliceIdSetRoot(token,phase) = Identity(
  disposed-effect-suffix-slice-set-domain,
  token.MaterializedExecutionEffectProtocolTokenId,phase,
  DisposedEffectSuffixSliceIds(token,phase)).

NoEffectAfterDispositionStatement = (
  MaterializedExecutionEffectProtocolTokenId,LastCompletedEffectProtocolPhase,
  ExactTerminalOrRecoveryOwnedReason,ConsumedEffectPhaseSliceIdSetRoot,
  DisposedEffectSuffixSliceIdSetRoot,
  NoCallerOwnedIntentSinkOrStatusCapabilityRemains=true
).

NoEffectAfterDispositionStatementId = Identity(
  no-effect-after-token-disposition-domain,
  NoEffectAfterDispositionStatement).

`RemainingEffectTemplateAndSliceIds(state)` is exactly the template and four
slice IDs at ordinals at or after `state.NextUnconsumedEffectOrdinal`, excluding
the consumed prefix. This definition is total for the initial state, a selected
successor, and an exhausted or malformed selection that returned no token. Its
canonical root is:

DisposedRemainingEffectTemplateAndSliceIdSetRoot(state) = Identity(
  disposed-remaining-effect-template-slice-set-domain,
  Identity(materialized-execution-effect-protocol-schedule-state-domain,state),
  RemainingEffectTemplateAndSliceIds(state)).

NoScheduleCapabilityEscapeOrReconstructionStatement = (
  MaterializedExecutionEffectProtocolScheduleStateRoot,
  ExactTerminalOrRecoveryOwnedReason,
  DisposedRemainingEffectTemplateAndSliceIdSetRoot,
  NextUnconsumedEffectOrdinal,ConsumedEffectPrefixRoot,
  NoTemplateIdentifierOrStaleStateRecreatesCapability=true
).

NoScheduleCapabilityEscapeOrReconstructionStatementId = Identity(
  no-effect-schedule-capability-escape-domain,
  NoScheduleCapabilityEscapeOrReconstructionStatement).

ExecutionEffectProtocolTokenDispositionProof = (
  MaterializedExecutionEffectProtocolTokenId,LastCompletedEffectProtocolPhase,
  ExactTerminalOrRecoveryOwnedReason,
  ConsumedEffectPhaseSliceIdSetRoot,DisposedEffectSuffixSliceIdSetRoot,
  NoEffectAfterDispositionStatementId
).

ExecutionEffectProtocolScheduleDispositionProof = (
  MaterializedExecutionEffectProtocolScheduleStateRoot,
  ExactTerminalOrRecoveryOwnedReason,
  DisposedRemainingEffectTemplateAndSliceIdSetRoot,
  NoScheduleCapabilityEscapeOrReconstructionStatementId
).

The token proof is inhabited only when its two set roots equal
`ConsumedEffectPhaseSliceIdSetRoot(token,phase)` and
`DisposedEffectSuffixSliceIdSetRoot(token,phase)` and its statement ID
recomputes from those same values and reason. The schedule proof is inhabited
only when its disposed-set root equals
`DisposedRemainingEffectTemplateAndSliceIdSetRoot(state)` and its statement ID
recomputes from that exact state, prefix, next ordinal, reason, and set root.

ExecutionEffectSinkResult =
    complete(SinkConsumptionEvidence,ExactOutcomeEvidence,
             StatusPublicationAndLossValidationSlice)
  | recovery-owned(DescendantExecutionReservation,EffectIntentEntry,
                   EffectPermit,ObservedEffectEvidenceRoot?)
  | same-body-winner(OriginalExecutionResultId,FinalizedExecutionResult)
  | integrity-failure(ExecutionIntegrityReport,
                      ObservedEffectLedgerState,CurrentDeploymentHeadState)
  | warrant-violation(FailureReport,ExactExecutionKeyHeadObservation).

`MaterializeExecutionEffectProtocolSchedule` constructs only the acyclic binding
and initial local schedule state after the reservation CAS succeeds. The binding
names the preexisting template-schedule root, invocation attempt, transaction,
running fence, and acyclic reservation-ownership pre-core; it contains neither
the final reservation/state root nor its own identity. `SelectNextExecutionEffectToken`
is total over that state. Its `selected` branch materializes exactly the next
template, returns the exact monotone prefix successor, and transfers the four
phase slices to that token; every other branch exposes no phase slice.
Field notation on a materialized execution token is the flattened projection
through its displayed template and body; every projected slice and identifier
therefore belongs to that exact selected token and cannot be supplied by a bare
template or a different materialization.
`CloseExecutionEffectTokenSuffix` consumes every selected but untaken phase slice
and returns the displayed token-disposition proof bound to the exact terminal,
recovery-owned, or pre-intent-stop reason. `CloseExecutionEffectSchedule`
affinely disposes every unselected template/slice in the supplied successor state
and returns the displayed schedule-disposition proof. Neither close operation may
recreate a live slice from an immutable template, identifier, or earlier state.
`BoundedConsumeExecutionEffectPermit` returns exactly `ExecutionEffectSinkResult`
and receives the exact retained proposed descriptor, payload and proof-input
objects plus both the sink/outcome slice and the status-publication/loss slice.
It rederives their bodies, roots, object IDs, warrants, statement ID, intent, and
permit and requires the verifier-accepted descriptor root to bind the exact sink,
deduplication/lease, expiry/quiescence, target, and payload used by the sink.
`complete` alone exposes outcome evidence and returns that same still-unconsumed
status slice. Any sink rejection, in-flight observation, timeout, exhaustion, or
other noncomplete outcome consumes the status slice to append the exact evidence
or in-flight classification to a retained descendant reservation before returning
`recovery-owned`. A losing evidence-publication CAS is classified from its one
atomic key/head observation into a fully validated same-body winner, recovering
descendant, integrity failure, or warrant violation. No branch drops local sink
evidence or authorizes an untracked retry.

IntentStatus =
    authorized-unconsumed
  | sink-rejected(ExactReason)
  | sink-consumed(ConsumptionEvidenceRoot)
  | outcome-observed(ExactOutcomeEvidenceRoot)
  | compensated(CompensationEvidenceRoot)
  | deduplicated(DeduplicationEvidenceRoot)
  | exposed-declared-partial(EffectEvidenceRoot).

The only permitted intent-status changes follow the displayed order from
`authorized-unconsumed` to one sink decision and then, where applicable, one
observed/reconciled terminal status. No transition may erase or replace an
authorization, permit, sink decision, or observed-effect fact.

EffectIntentLedger = (
  OrderedEffectIntentEntries,
  OrderedEffectPermits,
  StatusTransitionEvidence,
  SinkConsumptionEvidence
).

EffectIntentLedgerRoot
  = Identity(effect-intent-ledger-domain, EffectIntentLedger).

EffectIntentLedgerObjectId is the typed immutable-object identifier that resolves
exactly the `EffectIntentLedger` whose identity is `EffectIntentLedgerRoot`.

EffectIntentLedgerRetentionStatement = (
  EffectIntentLedgerRoot, EffectIntentLedgerObjectId,
  TransitiveIntentPermitAuthorizationAndSinkEvidenceGraphRoot,
  RequiredReservationAndRecoveryLifetime,
  DurableFaultDomain, ImmutableObjectLocatorAndReplicationProfileId
).

EffectIntentLedgerRetentionStatementId = Identity(
  effect-intent-ledger-retention-domain,
  EffectIntentLedgerRetentionStatement).

EffectIntentLedgerRetentionWarrant = (
  EffectIntentLedgerRetentionStatement,
  EffectIntentLedgerRetentionStatementId,
  RetentionProofId,
  VerifierResult=accept(VerifiedStatementIds containing
                        EffectIntentLedgerRetentionStatementId)
).

EffectIntentLedgerRetentionWarrantId = Identity(
  effect-intent-ledger-retention-warrant-domain,
  EffectIntentLedgerRetentionWarrant).

EffectIntentLedgerRetentionWarrantObjectId is the typed immutable-object
identifier resolving that exact warrant. Every intent/status/recovery-effect CAS
MUST atomically store the new ledger object and its transitive retention warrant,
then publish all four new root/object/warrant fields in the reservation. A root
without those resolvable retained objects is not a valid persisted ledger.

`ReservationDescendsFrom(r2,r1)` means that both reservations have the same
original transaction key/body, recovery material/bundle, independently retained
emergency template, and their recovery-bundle/original-material retention-warrant
identities; `r2` preserves every entry,
permit, and status fact committed by `r1`; and exactly one of these cases holds:
(a) `r1` is running and `r2` is the one legal running-to-recovering fence
transition whose `RecoveryOrigin` names `r1`, optionally followed by permitted
descendants, or (b) both are under that identical recovering fence and `r2`
differs only by permitted append-only intent/status/recovery-work/schedule-root
transitions, including monotone lease-epoch takeover after accepted prior-owner
quiescence and release of that lease at a checkpoint or terminal commit. No
second recovery fence replacement, lease-epoch-counter rollback, or fault-count
rollback is a descendant. A stage-attempt marker may only be installed by its
exact schedule CAS and may be cleared only by stage advance, checkpoint, or an
accepted fencing takeover that increments the fault counter.
Every takeover descendant installs the exact retained
`AcceptedRecoveryTakeoverEvidenceRef` both in its active lease and in the
reservation-level `LatestAcceptedRecoveryTakeoverEvidenceRef`; later descendants
preserve the reservation-level reference across checkpoint lease release, and a
higher epoch replaces it only with a newly accepted retained evidence object for
that transition. A missing, erased, or unresolvable takeover object/warrant is
not a valid descendant.
Resume acquisition is linear: a descendant contains only the exact remaining
suffix proved from its ancestor. Stage-attempt, stage-advance, checkpoint, and
tail-attempt schedules are authenticated branch-indexed maps: their immutable
map root is preserved and their consumed-key set can only grow by the one exact
CAS-selected key. No descendant can restore or reuse a consumed token/bundle or
switch to an unproved branch.
Every changed effect-ledger root in a descendant MUST be accompanied by its
matching immutable object and retention-warrant identities; preservation is
verified over the resolved typed ledgers, not inferred from unrelated roots.
Those four effect-ledger reference fields advance together and are deliberately
excluded from the invariant recovery-material retention-identity set above.
Likewise, a changed `RecoveryWorkRoot` MUST atomically publish the matching
`RecoveryWorkObjectId` and both accepted work-graph retention-warrant references;
an unchanged work root preserves all three references. The lease work root,
cursor body, and resolved work graph MUST name that same object. No descendant
may mix work roots, graph objects, or warrants from different stages.

FinalizationBudgetCore = (
  OutcomeStagingSlice, ConformanceCheckSlice,
  OutcomeStagingFailureSlice, ConformanceFailureSlice,
  AtomicPublicationSlice
).

FinalizationLinearUseAndSufficiencyStatement = (
  Root(FinalizationBudgetCore),ExactPairwiseDisjointnessStatementId,
  NoncheckpointingOutcomeCheckBuildCasAndClassifiedLossBoundStatementId,
  SafeQuarantineFailureCompletionStatementId,
  ExactMutuallyExclusiveBranchAffineDispositionStatementId
).

FinalizationLinearUseAndSufficiencyStatementId = Identity(
  finalization-linear-sufficiency-statement-domain,
  FinalizationLinearUseAndSufficiencyStatement).

FinalizationLinearUseAndSufficiencyWarrant = (
  FinalizationLinearUseAndSufficiencyStatement,
  FinalizationLinearUseAndSufficiencyStatementId,
  ProofId,VerifierResult=accept(VerifiedStatementIds containing
    FinalizationLinearUseAndSufficiencyStatementId)
).

FinalizationBudgetPartition = (
  FinalizationBudgetCore,FinalizationLinearUseAndSufficiencyWarrant
).

FencePublicationPermitBody = (
  RecoveryPolicyCoreId,RecoveryResourceContractId,
  ExecutionTransactionId,ExecutionRequestBodyRoot,
  AuthorizedRunningToRecoveringTransitionProfileId,
  WinnerOnlyCoordinatorConsumptionRuleId
).

FencePublicationPermitId = Identity(
  recovery-fence-publication-permit-domain,FencePublicationPermitBody).

CoordinatorPermitStateKey = Identity(
  coordinator-permit-state-key-domain,FencePublicationPermitId).

SingleUseCoordinatorCapability = stable-permit-reference(
  CoordinatorPermitStateKey,AuthorizedRunningToRecoveringTransitionProfileId).

FencePublicationPermit = (
  FencePublicationPermitBody,FencePublicationPermitId,
  SingleUseCoordinatorCapability
).

The permit is unique to one admitted execution partition. It authorizes only the
exact running-to-recovering transition for that partition and is consumed in the
same coordinator CAS only on success. Preparation, hashing, object storage, and
a losing compare cannot spend or duplicate it. The serialized permit contains
only the stable coordinator state key; authoritative spendability is the map's
`unspent` value, atomically changed to `consumed(CoordinatorCommitId)` with the
winning head transition. Reloading an old partition cannot restore it.

RecoveryBudgetCore = (
  FencePublicationPermit, ResumeAcquisitionSchedule,
  RecoveryStageAttemptSchedule,
  StageAdvancePublicationSchedule,
  CheckpointPublicationSchedule,
  RecoveryTailAttemptSchedule,
  RecoveryScheduleStatePersistenceWarrant
).

RecoveryLinearUseProgressAndSufficiencyStatement = (
  Root(RecoveryBudgetCore),ExactPairwiseDisjointnessStatementId,
  ResumeStageAdvanceCheckpointTailProgressAndFaultBoundStatementId,
  WinnerOnlyPermitAndClassifiedCasLossStatementId
).

RecoveryLinearUseProgressAndSufficiencyStatementId = Identity(
  recovery-linear-progress-sufficiency-statement-domain,
  RecoveryLinearUseProgressAndSufficiencyStatement).

RecoveryLinearUseProgressAndSufficiencyWarrant = (
  RecoveryLinearUseProgressAndSufficiencyStatement,
  RecoveryLinearUseProgressAndSufficiencyStatementId,
  ProofId,VerifierResult=accept(VerifiedStatementIds containing
    RecoveryLinearUseProgressAndSufficiencyStatementId)
).

RecoveryBudgetPartition = (
  RecoveryBudgetCore,RecoveryLinearUseProgressAndSufficiencyWarrant
).

Field notation on either budget partition is the flattened projection through
its displayed core. Each warrant binds only the core that excludes that warrant,
so both identities are acyclic.

RecoveryStageAttemptResourceCore = (
  StageAttemptTokenId, RecoverySubstage,
  StageAttemptAcquisitionSlice,
  StageWorkSlice,StageAdvancePreparationAndCasAttemptSlice,
  CheckpointPreparationAndCasAttemptSlice?,EmergencyCheckpointSlice?,
  RecoveryEffectProtocolSchedule?,
  DeclaredCrashAndTakeoverFaultBound
).

RecoveryEffectProtocolTokenTemplate = (
  EffectOrdinal,EffectAuthorizationSlice,IntentAndPermitPublicationSlice,
  SinkMembershipAndOutcomeCaptureSlice,StatusPublicationAndLossValidationSlice
).

RecoveryEffectProtocolSchedule = AuthenticatedFiniteOrderedFamilyOf(
  EffectOrdinal -> RecoveryEffectProtocolTokenTemplate).

RecoveryEffectProtocolTokenBody = (
  Root(RecoveryEffectProtocolTokenTemplate),RecoveryTransactionId,
  RecoveryInvocationAttemptId,RecoveryLeaseEpoch,ActiveRecoveryAttemptFence,
  RecoveryStageAttemptMarkerRoot
).

RecoveryEffectProtocolTokenId = Identity(
  materialized-recovery-effect-protocol-token-domain,
  RecoveryEffectProtocolTokenBody).

RecoveryEffectProtocolToken = (
  RecoveryEffectProtocolTokenTemplate,RecoveryEffectProtocolTokenBody,
  RecoveryEffectProtocolTokenId
).

MaterializedRecoveryEffectProtocolSchedule = AuthenticatedFiniteOrderedFamilyOf(
  EffectOrdinal -> (RecoveryEffectProtocolTokenTemplate,
                    RecoveryEffectProtocolToken)).

MaterializeRecoveryEffectProtocolSchedule(
  templateSchedule,recoveryTx,attempt,epoch,activeFence,stageMarkerRoot)
  deterministically pairs every template with the displayed materialized token;
  its identity binds all six dynamic inputs and is computed within the
  stage-acquisition slice. Every recovery `EffectIntentBody` binds
  `recovery(RecoveryEffectProtocolTokenId)` from that exact selected token; its
  permit binds the same value transitively through `IntentId`. Every recovery
  proposal also constructs the complete retained `ProposedEffectDescriptor`,
  uses the exact `ExecProfileId` from the retained
  `OriginalExecutionObjectSet`, verifies the authorization statement containing
  that descriptor root, and embeds the descriptor, exact statement, accepted
  proof body/IDs, intent body/ID, and permit ID in `EffectIntentEntry` exactly as
  the execution protocol does.

RecoveryEffectProtocolResult =
    committed(DescendantReservation,EffectIntentEntry,EffectPermit,
              SinkConsumptionEvidence,ExactOutcomeEvidence)
  | recovery-owned(DescendantReservation,EffectIntentEntry,
                   EffectPermit,ObservedEffectEvidenceRoot?)
  | same-body-winner(OriginalRecoveryResultId,FinalizedRecoveryResult)
  | normal-execution-winner(CurrentDeploymentHeadState)
  | identity-reuse(Reason)
  | conflict(CurrentDeploymentHeadState)
  | integrity-failure(FailureReport)
  | warrant-violation(FailureReport,ExactTwoKeyHeadObservation).

The optional attempt-neutral template schedule is present exactly for
`effect-reconciliation`. The stage-acquisition CAS materializes each selected
template against the current recovery transaction, attempt, lease epoch, active
fence, and marker; those dynamic values are not precommitted in the execution
partition. Each proposal consumes one materialized token across authorization, intent/permit publication,
sink membership/consumption, outcome capture, status publication, and classified
loss. Once an intent is committed, no unresolved/timeout/exhausted path may drop
it or return an unqualified failure: it yields `recovery-owned` with the exact
descendant and sink/in-flight evidence. A later stage failure/checkpoint embeds
that carrier in its successor work graph; loss of the lease returns only an
exact `conflict(reserved(descendant))` whose ledger contains the intent/permit,
so resumed reconciliation can deterministically complete it.

RecoveryStageAttemptResourceCoreRoot = Identity(
  recovery-stage-attempt-resource-core-domain,
  RecoveryStageAttemptResourceCore).

StageAttemptCompletionAndDisjointnessStatement = (
  RecoveryStageAttemptResourceCoreRoot,
  ExactDisjointnessFromResumeAdvanceCheckpointTailAndEffectSchedulesStatementId,
  BoundedStageTotalityAndCheckpointSafetyStatementId,
  ExactStageExitPresenceAndAffineDispositionStatementId,
  DeclaredCrashAndTakeoverFaultBound
).

StageAttemptCompletionAndDisjointnessStatementId = Identity(
  recovery-stage-attempt-completion-disjointness-domain,
  StageAttemptCompletionAndDisjointnessStatement).

StageAttemptCompletionAndDisjointnessWarrant = (
  StageAttemptCompletionAndDisjointnessStatement,
  StageAttemptCompletionAndDisjointnessStatementId,
  ProofId,
  VerifierResult=accept(VerifiedStatementIds containing
                        StageAttemptCompletionAndDisjointnessStatementId)
).

StageAttemptCompletionAndDisjointnessWarrantId = Identity(
  recovery-stage-attempt-completion-disjointness-warrant-domain,
  StageAttemptCompletionAndDisjointnessWarrant).

RecoveryStageAttemptBundle = (
  RecoveryStageAttemptResourceCore,
  RecoveryStageAttemptResourceCoreRoot,
  StageAttemptCompletionAndDisjointnessWarrantId,
  StageAttemptCompletionAndDisjointnessWarrantObjectId
).

RecoveryStageAttemptBundleRoot = Identity(
  recovery-stage-attempt-bundle-domain,RecoveryStageAttemptBundle).

RecoveryProgressOrdinal is a natural number projected by the exact
`ProgressMeasureAndWarrant` committed in `RecoveryWorkBody`. The warrant binds the
actual processed, pending, and progress-state roots to this ordinal and proves
strict descent/advance. Resource allocation uses the ordinal, which is known from
the admitted finite progress bound; runtime work roots never have to be enumerable
when the partition is constructed.

RecoveryStageAttemptKey = (
  RecoverySubstage, RecoveryProgressOrdinal,
  RecoveryFaultCount, StageAttemptOrdinal
).

RecoveryScheduleGenesisCore = (
  ScheduleKind, ImmutableResourceMapOrOrderedFamilyRoot,
  ExactKeyOrOrdinalDomain, AdmittedCardinalityBound,
  RecoveryResourceContractId,ScheduleAllocationProfileId
).

ScheduleGenesisId = Identity(
  recovery-schedule-genesis-domain,RecoveryScheduleGenesisCore).

`ScheduleAllocationProfileId` identifies a pre-partition allocation law and
contains no schedule-state or partition-wrapper root. The genesis core excludes
every mutable schedule-state node, consumed set, next ordinal, parent root, and
final partition wrapper. It can therefore be
identified before the initial state is built. The initial and every successor
node MUST carry the same exact `ScheduleGenesisId` derived from the immutable
resource family that the state exposes; an unrelated genesis has no inhabitant.

RecoveryScheduleStateNode = (
  ScheduleKind, ScheduleGenesisId,
  ParentScheduleStateRoot?, ConsumedSelector?
).

An initial schedule state has `ParentScheduleStateRoot=none`,
`ConsumedSelector=none`. Every successor names its exact parent and the one
consumed key/token/ordinal. Trusted coordinator atomicity requires the successor
body to be stored before that same root is published; no post-CAS receipt is
embedded in the successor identity. The immutable
genesis identity and schedule kind never change. These fields are inside every
schedule-state identity below, so parent-chain reachability is committed data,
not explanatory metadata.

RecoveryStageAttemptScheduleState = (
  RecoveryScheduleStateNode,
  AuthenticatedExactMapFromRecoveryStageAttemptKeyTo(
    RecoveryStageAttemptBundle,RecoveryStageAttemptBundleRoot),
  MonotoneConsumedStageAttemptKeySetRoot
).

RecoveryStageAttemptScheduleRoot = Identity(
  recovery-stage-attempt-schedule-domain,RecoveryStageAttemptScheduleState).

RecoveryStageAttemptSchedule = RecoveryStageAttemptScheduleState.

The authenticated map carries each complete bundle preimage together with its
root inside the retained `ResolvedExecutionResourcePartition`; a map value is not
an unresolved naked hash. Its state root is the content-addressed resolver
specified by `RecoveryScheduleStatePersistenceWarrant`, and every consumed-set
successor is atomically stored before publication.

The named stage tag, token, acquisition/work slices, and optional
emergency-checkpoint slice are projections of the acyclic resource core. Before
any checkpointable recovery substage, one CAS selects the exact key determined
by the committed program counter/progress, fault count, and next attempt ordinal,
uses only its acquisition slice, adds that key to the monotone consumed set, and
binds the bundle to the current attempt/lease/work root. A key already consumed
cannot be selected again. A crash/takeover uses a fresh fault/attempt key; a
checkpoint uses its new progress key; and advancing uses the next substage key.
No branch inference, suffix assumption, or re-presentation is permitted. The map
covers the checkpoint progress bound, declared crash/takeover bound, and one
final safe completion per reachable stage.

RecoveryTailAttemptResourceCore = (
  TailAttemptTokenId,
  TailAttemptAcquisitionSlice,
  OutcomeStagingSlice, OutcomeStagingFailureSlice,
  ConformanceCheckSlice, ConformanceFailureSlice,
  AtomicPublicationSlice,
  DeclaredCrashAndTakeoverFaultBound
).

RecoveryTailAttemptResourceCoreRoot = Identity(
  recovery-tail-attempt-resource-core-domain,
  RecoveryTailAttemptResourceCore).

TailCompletionAndDisjointnessStatement = (
  RecoveryTailAttemptResourceCoreRoot,
  ExactPairwiseDisjointnessStatementId,
  NoncheckpointingOutcomeFailureConformanceAndFinalizationTotalityStatementId,
  SafeQuarantineConversionSufficiencyStatementId,
  AtomicTwoKeyAndHeadPublicationSufficiencyStatementId,
  ExactMutuallyExclusiveTailBranchAffineDispositionStatementId,
  DeclaredCrashAndTakeoverFaultBound
).

TailCompletionAndDisjointnessStatementId = Identity(
  recovery-tail-completion-disjointness-domain,
  TailCompletionAndDisjointnessStatement).

TailCompletionAndDisjointnessWarrant = (
  TailCompletionAndDisjointnessStatement,
  TailCompletionAndDisjointnessStatementId,
  ProofId,
  VerifierResult=accept(VerifiedStatementIds containing
                        TailCompletionAndDisjointnessStatementId)
).

TailCompletionAndDisjointnessWarrantId = Identity(
  recovery-tail-completion-disjointness-warrant-domain,
  TailCompletionAndDisjointnessWarrant).

RecoveryTailAttemptBundle = (
  RecoveryTailAttemptResourceCore,
  RecoveryTailAttemptResourceCoreRoot,
  TailCompletionAndDisjointnessWarrantId,
  TailCompletionAndDisjointnessWarrantObjectId
).

RecoveryTailSliceState = unspent | consumed.

RecoveryTailProgressState = (
  TailAttemptBundleRoot,
  TailAttemptAcquisitionSliceState=consumed,
  OutcomeStagingSliceState,
  OutcomeStagingFailureSliceState,
  ConformanceCheckSliceState,
  ConformanceFailureSliceState,
  AtomicPublicationSliceState
).

RecoveryTailProgressStateRoot = Identity(
  recovery-tail-progress-state-domain,RecoveryTailProgressState).

The `acquired` tail result creates the unique progress state with every field
after `TailAttemptAcquisitionSliceState` equal to `unspent`. Passing a slice to
its displayed bounded tail operation changes exactly that field to `consumed`;
the conformance helper consumes both conformance fields on every branch, using
the failure slice either to stage the required conversion or to construct its
no-action disposition. No transition restores an `unspent` field.
`CloseRecoveryTailAttemptAndReturn(bundle,progress,result)` validates the bundle
and progress roots, affinely disposes every field still marked `unspent`, builds
the exact `BranchResourceDispositionProof`, and returns the supplied typed
`RecoveryResult`. If validation or closure fails it returns only
`internal-failure(FailureReport(recovery-tail-disposition-warrant-violation,
report))`; it never exposes a remainder. Terminal recovery finalization receives
the same progress state, consumes the still-unspent atomic-publication slice,
and disposes every other unspent sibling inside its classified result algebra.

BranchResourceDispositionProof = (
  OwningResourceCoreRoot,SelectedExitTag,
  ExactlyConsumedCapabilityIds,ExactlyDisposedSiblingCapabilityIds,
  NoCapabilityEscapeOrReconstructionStatementId
).

RecoveryStageExitSelectionResult =
    advance(StageAdvancePreparationAndCasAttemptSlice,
            BranchResourceDispositionProof)
  | checkpoint(CheckpointPreparationAndCasAttemptSlice,
               BranchResourceDispositionProof)
  | emergency-checkpoint(EmergencyCheckpointSlice,
                         CheckpointPreparationAndCasAttemptSlice,
                         BranchResourceDispositionProof)
  | no-publication(BranchResourceDispositionProof)
  | warrant-violation(FailureReport).

`SelectAndCloseRecoveryStageExit` returns exactly this algebra and may be called
once for an acquired stage bundle. It selects only the constructor matching the
actual stage result and affinely disposes every sibling; its
`warrant-violation` branch exposes no slice.

`CloseRecoveryStageNoPublication(bundle,result)` consumes the bundle's
`no-publication` exit, disposes every work/publication sibling, and returns the
supplied already-typed `RecoveryResult`; if closing fails it returns only
`internal-failure(FailureReport(
stage-exit-disposition-warrant-violation,report))`.
`DisposeSelectedRecoveryStageExit(slice,result)` similarly disposes a selected
but unused exit slice before returning the supplied failure. These are total
affine operations, not unchecked `require` shorthands.

For every finalization core, stage-attempt core, and tail-attempt core, the
accepted statement defines a total presence matrix and an affine close rule for
every result tag. A selected work/CAS slice is consumed at most once; every
untaken sibling is disposed exactly once with `BranchResourceDispositionProof`,
or its ownership is atomically transferred into the one permitted persisted
checkpoint/reservation successor. Absence of a slice required by a reachable
tag is a typed warrant violation, never a naked `require`. `FINISH_RESERVED`,
stage advance/checkpoint/emergency-checkpoint helpers, and recovery terminal
finalization own this close operation and return no live remainder. Thus normal
success also closes failure slices, failure closes success siblings, and a
recovery-required or classified CAS-loss return cannot leak or replay an
attempt-owned capability.

TailAttemptBundleRoot = Identity(
  recovery-tail-attempt-bundle-domain,RecoveryTailAttemptBundle).

RecoveryTailAttemptKey = (RecoveryFaultCount,TailAttemptOrdinal).

RecoveryTailAttemptScheduleState = (
  RecoveryScheduleStateNode,
  AuthenticatedExactMapFromRecoveryTailAttemptKeyTo(
      (RecoveryTailAttemptBundle,TailAttemptBundleRoot)),
  MonotoneConsumedTailAttemptKeySetRoot
).

RecoveryTailAttemptScheduleRoot = Identity(
  recovery-tail-attempt-schedule-domain,RecoveryTailAttemptScheduleState).

RecoveryTailAttemptSchedule = RecoveryTailAttemptScheduleState.

Every map value contains the complete retained bundle preimage and its checked
root. A successor adds exactly the current fault-generation/attempt key to the
consumed set, is content-addressed and stored under the persistence warrant, and
can never restore an earlier key. After a crash/takeover the serialized fault
count selects a disjoint generation, so a bundle whose acquisition work was
abandoned before CAS is never re-presented.

RecoveryScheduleTokenBody = (
  ScheduleKind, TokenOrdinal, FixedResourceSlice,
  AuthorizedCoordinatorTransitionProfileId
).

RecoveryScheduleTokenId = Identity(
  recovery-schedule-token-domain,RecoveryScheduleTokenBody).

ResumeAcquisitionTokenId is a `RecoveryScheduleTokenId` whose body has
`ScheduleKind=resume-acquisition`.

ResumeAcquisitionScheduleState = (
  RecoveryScheduleStateNode,
  ExactFiniteOrderedFamilyOf(
    ResumeOrdinal -> (RecoveryScheduleTokenBody,ResumeAcquisitionTokenId)),
  NextUnconsumedResumeOrdinal,
  ConsumedResumePrefixRoot
).

ResumeAcquisitionScheduleRoot = Identity(
  recovery-resume-acquisition-schedule-domain,
  ResumeAcquisitionScheduleState).

ResumeAcquisitionSchedule = ResumeAcquisitionScheduleState.

StageAdvancePublicationKey = (
  CompletedRecoverySubstage, RecoveryProgressOrdinal,
  RecoveryFaultCount, StageAdvanceOrdinal
).

CheckpointPublicationKey = (
  RecoverySubstage, RecoveryProgressOrdinal,
  RecoveryFaultCount, CheckpointOrdinal
).

StageAdvancePublicationScheduleState = (
  RecoveryScheduleStateNode,
  AuthenticatedExactMapFromStageAdvancePublicationKeyTo(
    RecoveryScheduleTokenBody,RecoveryScheduleTokenId),
  MonotoneConsumedStageAdvanceKeySetRoot
).

StageAdvancePublicationScheduleRoot = Identity(
  recovery-stage-advance-publication-schedule-domain,
  StageAdvancePublicationScheduleState).

StageAdvancePublicationSchedule = StageAdvancePublicationScheduleState.

CheckpointPublicationScheduleState = (
  RecoveryScheduleStateNode,
  AuthenticatedExactMapFromCheckpointPublicationKeyTo(
    RecoveryScheduleTokenBody,RecoveryScheduleTokenId),
  MonotoneConsumedCheckpointKeySetRoot
).

CheckpointPublicationScheduleRoot = Identity(
  recovery-checkpoint-publication-schedule-domain,
  CheckpointPublicationScheduleState).

CheckpointPublicationSchedule = CheckpointPublicationScheduleState.

CheckpointPublicationSelectionResult =
    selected(RecoveryScheduleTokenBody,RecoveryScheduleTokenId,
             CheckpointPublicationScheduleState,
             CheckpointPublicationScheduleRoot)
  | exhausted-within-declared-bound(SufficiencyWarrantId)
  | malformed-schedule(Reason)
  | internal-failure(FailureReport).

`SelectCheckpointPublicationToken` returns exactly this nonconsuming algebra
over an ingress-validated state. Only the atomic checkpoint helper may add the
selected key and publish the returned successor state/root; its persisted token
is winner-only publication authorization, while the current stage-attempt
bundle funds preparation, the CAS attempt, and classified loss.

The named token and six slices are exact projections of
`RecoveryTailAttemptResourceCore`. The warrant object is strictly resolved and
must accept its exact statement before bundle acquisition. Because the statement
binds only the preexisting resource-core root—not the bundle or warrant identity—
the identity construction is acyclic.

RecoveryTailAttemptAcquisitionResult =
    acquired(DescendantReservation,RecoveryTailAttemptBundle,
             RecoveryTailAttemptScheduleState)
  | exhausted-within-declared-bound(SufficiencyWarrantId)
  | declared-fault-bound-exceeded(ObservedFaultCount,DeclaredBound)
  | same-body-winner(OriginalRecoveryResultId,FinalizedRecoveryResult)
  | normal-execution-winner(CurrentDeploymentHeadState)
  | identity-reuse(Reason)
  | integrity-failure(FailureReport)
  | warrant-violation(FailureReport,ExactTwoKeyHeadObservation)
  | rejected(Reason)
  | incoherent(ConflictProof)
  | conflict(CurrentDeploymentHeadState)
  | unresolved(DependencyIds)
  | unsupported(FeatureIds)
  | resource-exhausted(ResourceReport)
  | internal-failure(FailureReport).

AcquireRecoveryTailAttempt(
  RecoveryRequest,RecoveryTransactionKey,CurrentRecoveringReservation,
  RecoveryInvocationAttemptId,RecoveryLeaseEpoch,
  ActiveRecoveryAttemptFence,ResolvedRecoveryWorkObjectGraph,
  RecoveryTailAttemptScheduleState,RecoveryTailAttemptScheduleRoot,
  RecoveryFaultCount,TailAcquisitionPreparationAndCasAttemptSlice,
  DeclaredCrashAndTakeoverFaultBound)
  returns `RecoveryTailAttemptAcquisitionResult` and equality-checks every
  request/key/attempt/epoch/fence/work/schedule projection against the current
  reservation before constructing a candidate.

`RecoveryTailAttemptSchedule` is the exact state/body above: a finite,
single-use family of disjoint
tail bundles sized by the recovery contract's explicit crash/takeover fault
bound. Before outcome staging, selection derives the exact
`RecoveryTailAttemptKey(currentReservation.RecoveryFaultCount,
nextTailAttemptOrdinal)`. One coordinator CAS adds that key to the authoritative
consumed set and binds the selected bundle to the current attempt/lease. The
CAS helper classifies its one exact two-key/head loss observation; only
`acquired` consumes the persisted key or exposes the bundle's work slices.
owning attempt performs no intermediate head mutation between that acquisition
and its terminal two-key CAS. The only other permitted mutation is a proved
fencing takeover, after which the old attempt's sink and coordinator tokens are
invalid and the new attempt must acquire a fresh bundle.
If a crash or proved lease takeover occurs, the new attempt consumes a fresh
bundle and rebuilds every core against its new acquired reservation. The schedule
contains at least the declared maximum interrupted attempts plus one proved
safe-terminal attempt. Exhaustion before a terminal commit contradicts the
admitted fault-bound premise and is reported as an exact fault-domain violation;
no undefined extra resource is assumed. A profile that can strand a reservation
within its declared fault bound is inadmissible.

`RecoveryWorkBody.RemainingStageResourceStateRoot` is exactly the remaining
`RecoveryStageAttemptSchedule` root. It separately commits the stage-advance and
checkpoint schedule roots; `ReservationStateBody` commits resume and tail-attempt
roots. Together they are equality/disjointness-checked against the original
partition warrant on fresh entry and every resume. No aggregate root may continue
to represent a separately advanced token as unspent.

`CheckpointPublicationSchedule` is an exact ordinal-indexed family of
single-use checkpoint-CAS/object-identity tokens represented as an authenticated
map plus monotone consumed-key set keyed by `CheckpointPublicationKey`. Each map
value contains the complete token body and its checked identity.
`RecoveryWorkRoot` commits the
remaining family after each checkpoint. Its sufficiency proof covers every
checkpoint allowed by the well-founded measure. It is disjoint from
`RecoveryTailAttemptSchedule` and from every atomic-publication or other subslice
inside every tail bundle; no checkpoint may consume any of them.

`StageAdvancePublicationSchedule` is the disjoint finite family of single-use
tokens, represented by the same authenticated-map/monotone-consumed-set form and
keyed by `StageAdvancePublicationKey`, for committing each completed recovery substage's next program counter,
remaining resource state, immutable outputs, and progress. Its remaining root is
committed by `RecoveryWorkRoot`. It cannot be substituted for a checkpoint,
resume-ingress, effect, or terminal-publication token.

`ResumeAcquisitionSchedule` is the exact ordered family above, sized from the
declared checkpoint and crash/takeover bounds, of
single-use fixed-size resume-ingress tokens. Its current remaining root is a
field of every reservation and `ReservationStateRoot`. A fresh caller uses only
its attempt-keyed `FencePreparationAndCasAttemptSlice` to construct objects and
attempt the compare; the shared `FencePublicationPermit` is consumed only by the
winning running-to-recovering CAS. Each newly issued or takeover invocation attempt
that enters an already-`recovering` reservation first CAS-advances exactly one
remaining resume token before it dispatches, acts on, or exposes a stage/effect
resource from the work. The fixed header's read-only schedule/work ingress MAY
strictly resolve and validate the retained work graph and schedules before that
CAS so the token is not hidden inside its own unresolved carrier; those objects
remain unusable until the lease CAS succeeds. An already
acquired marker is usable only inside that
same live attempt/lease epoch; no later call may reuse it, a prior caller's
preparation capability or the already-consumed `FencePublicationPermit`,
or an earlier resume token. Concurrent contenders may peek the same next token,
but each charges takeover proof/candidate construction to its own fresh
`TakeoverProofSlice` and `TakeoverCandidateAndCasAttemptSlice`; the
persisted token's `FixedResourceSlice` is consumed
only by the winning lease CAS, and a loser never spends or exposes it. Its
sufficiency warrant
covers every checkpoint permitted by the same well-founded progress measure plus
every resume/takeover invocation entering an already-recovering reservation
allowed by
`DeclaredCrashAndTakeoverFaultBound`, including the final proved terminal
attempt. Premature exhaustion is an exact fault-domain/sufficiency-warrant
violation, never permission to reuse a token.

ResumeTokenPeekResult =
    next(ResumeAcquisitionTokenId,ResumeAcquisitionScheduleState,
         PostConsumptionResumeScheduleRoot)
  | exhausted-within-declared-bound(SufficiencyWarrantId)
  | declared-fault-bound-exceeded(ObservedFaultCount,DeclaredBound)
  | malformed-schedule(Reason)
  | internal-failure(FailureReport).

`PeekExactNextResumeToken` is total over the already ingress-validated schedule
body and returns this algebra without consuming a token. Only
`next(id,postState,postRoot)` may enter the one atomic consume-and-lease CAS,
which must consume that same `id`, store that exact successor body under
`postRoot`, and
publish that same root or conflict.

`EffectProtocolSchedule` is the exact attempt-neutral template family displayed
above. The successful reservation acquisition materializes one acyclic local
schedule binding against the allocated execution attempt, transaction, committed
running fence, and reservation-ownership pre-core. Before each proposed effect,
`SelectNextExecutionEffectToken` returns one materialized token and the unique
successor schedule state. Its authorization, intent/permit publication, sink/
outcome, and status/loss slices are pairwise disjoint and are passed only to the
corresponding bounded operation. `RunWorkSlice` bounds the number and maximum
encoded size of proposals. A pre-intent stop disposes the token suffix; after an
intent CAS, any noncomplete sink/status outcome preserves the intent and yields
`recovery-owned` for the exact actual reservation. Every public return or terminal
finalization affinely closes the unselected schedule remainder. No phase reuses a
prior slice, and neither a template nor a stale schedule root recreates a live
materialized token.

ExecutionResourcePartitionCore = (
  ExecutionInvocationAttemptId,ExecutionResourceGrantReceipt,
  RunResourceContractId, RecoveryResourceContractId,
  UnreservedPartitionDispositionCapabilityId,
  InputValidationSlice, RecoveryPolicyVerificationSlice,
  ReservationAcquisitionSlice, RunWorkSlice, EffectProtocolSchedule,
  FinalizationBudgetPartition, RecoveryBudgetPartition
).

ExecutionResourcePartitionCoreRoot = Identity(
  execution-resource-partition-core-domain,
  ExecutionResourcePartitionCore).

FinalizationAndRecoverySafeTerminalSufficiencyStatement = (
  ExecutionResourcePartitionCoreRoot,
  FinalizationLinearUseAndSufficiencyStatementId,
  RecoveryLinearUseProgressAndSufficiencyStatementId,
  BoundedSafeTerminalLivenessStatementId
).

FinalizationAndRecoverySafeTerminalSufficiencyStatementId = Identity(
  finalization-recovery-safe-terminal-sufficiency-domain,
  FinalizationAndRecoverySafeTerminalSufficiencyStatement).

PartitionAndSufficiencyStatement = (
  ExecutionResourcePartitionCoreRoot,
  ExactPairwiseLinearDisjointnessStatementId,
  FinitePerStageSizeAndWorkBoundStatementId,
  RecoveryScheduleStatePersistenceStatementId,
  FinalizationAndRecoverySafeTerminalSufficiencyStatementId
).

PartitionAndSufficiencyStatementId = Identity(
  partition-and-sufficiency-statement-domain,
  PartitionAndSufficiencyStatement).

PartitionAndSufficiencyWarrant = (
  PartitionAndSufficiencyStatement,
  PartitionAndSufficiencyStatementId,
  ProofId,
  VerifierResult=accept(VerifiedStatementIds containing
                        PartitionAndSufficiencyStatementId)
).

PartitionAndSufficiencyWarrantId = Identity(
  partition-and-sufficiency-warrant-domain,
  PartitionAndSufficiencyWarrant).

ExecutionResourcePartition = (
  ExecutionResourcePartitionCore,
  PartitionAndSufficiencyWarrant
).

ExecutionResourcePartitionRoot = Identity(
  execution-resource-partition-domain,ExecutionResourcePartition).

ExecutionResourcePartitionObjectId = Identity(
  execution-resource-partition-object-domain,ExecutionResourcePartitionRoot).

`ExecutionResourcePartitionObjectId` is a typed content-addressed object
identifier whose strict resolution yields exactly `ExecutionResourcePartition`
and rederives `ExecutionResourcePartitionRoot`. Field notation such as
`resourcePartition.InputValidationSlice` is the flattened projection through
`ExecutionResourcePartitionCore`; it never denotes an ambient second body.

ResolvedExecutionResourcePartition = (
  ExecutionResourcePartition,
  ExecutionResourcePartitionRoot,
  ExecutionResourcePartitionObjectId,
  PartitionAndSufficiencyWarrant,
  PartitionAndSufficiencyWarrantId
).

The partition warrant is outside the core it certifies. Its statement binds the
acyclic core root, while the final partition root binds the warrant; no warrant
embedded in the final partition names that final root.

ExecutionPartitionDispositionProof = (
  ExecutionInvocationAttemptId,ExecutionResourcePartitionCoreRoot,
  ExactPreReservationReturnReason,
  AffinelyClosedRemainingCapabilityRoots,
  NoReservationOwnershipTransferProof
).

ExecutionPartitionOwnershipState =
    unreserved-transfer-intent(UnreservedPartitionDispositionCapabilityId)
  | reserved(ReservedExecutionPartitionOwnershipProof).

ReservedExecutionPartitionOwnershipProof = (
  ExecutionInvocationAttemptId,ExecutionResourcePartitionCoreRoot,
  ExecutionTransactionId,ExecutionRequestBodyRoot,
  ReservationOwnershipPreCoreRoot,
  ReservationOwnershipTransitionStatementId,
  ConsumedUnreservedDispositionCapabilityId
).

ReservationStatePreCore = (
  ReservationPhase,EffectIntentLedgerRoot,EffectIntentLedgerObjectId,
  EffectIntentLedgerRetentionWarrantId,
  EffectIntentLedgerRetentionWarrantObjectId,
  RecoveryWorkRoot?,RecoveryWorkObjectId?,
  RecoveryWorkObjectGraphRetentionWarrantId?,
  RecoveryWorkObjectGraphRetentionWarrantObjectId?,
  RecoveryResumeAcquisitionScheduleRoot,RecoveryResumeAcquisitionMarker?,
  RecoveryInvocationLease?,LatestAcceptedRecoveryTakeoverEvidenceRef?,
  RecoveryLeaseEpochCounter,RecoveryFaultCount,RecoveryStageAttemptMarker?,
  RecoveryTailAttemptScheduleRoot,RecoveryTailAttemptMarker?
).

ReservationOwnershipPreCore = (
  ExecutionTransactionId,OriginalTransactionKey,ExecutionRequestBodyRoot,
  ExactExecutionRequestBodyOrResolvableObjectId,
  ExpectedDeploymentConfiguration,ReservationStatePreCore,
  ExecutionResourcePartitionRoot,
  RecoveryPolicyId,RecoveryPolicyCoreId,RecoveryResourceContractId,
  RecoveryExecutionMaterialRoot,RecoveryBundleRoot,RecoveryBundleObjectId,
  EmergencySafeQuarantineTemplateId,EmergencySafeQuarantineTemplateRoot,
  EmergencySafeQuarantineTemplateRetentionWarrantId,
  EmergencySafeQuarantineTemplateRetentionWarrantObjectId,
  RecoveryBundleRetentionWarrantId,RecoveryBundleRetentionWarrantObjectId
).

ReservationOwnershipPreCoreRoot = Identity(
  reservation-ownership-precore-domain,
  ReservationOwnershipPreCore).

The ownership proof binds this acyclic draft/pre-core, not the final state root.
The final `ReservationStateRoot` then hashes the proof once. No proof preimage
contains that enclosing root.

ReservationOwnershipTransitionStatement = (
  BeforeDeploymentHeadState,TransactionKey,ExecutionRequestBodyRoot,
  ReservationOwnershipPreCoreRoot,
  ConsumedUnreservedDispositionCapabilityId,
  CoordinatorAtomicityProfileId
).

ReservationOwnershipTransitionStatementId = Identity(
  reservation-ownership-transition-domain,
  ReservationOwnershipTransitionStatement).

This is a pre-CAS authorization statement, not a receipt containing the final
reservation/state root. The successful reservation CAS atomically consumes the
live disposition capability, instantiates this exact statement/proof, and
publishes the reserved ownership state.

Before the reservation CAS succeeds, every return after partition construction
MUST call `DisposeUnreservedExecutionPartition` using the grant's unique live
disposition capability and retain this proof. A successful reservation CAS
instead consumes it, constructs `ReservedExecutionPartitionOwnershipProof`, and
publishes that proof in the final reservation/state identity. The pre-CAS
reservation draft carries only `unreserved-transfer-intent`; it is not a valid
published reservation. No branch may both dispose and reserve, or return while
leaving an attempt-private capability live.

QuiescenceStatement = (
  RecoveryFenceToken, EffectIntentLedgerRoot,
  CompleteIntentAndPermitCarrierRoot, SinkObservationRoot,
  EffectCommitModelId, QuiescenceBoundary
).

QuiescenceStatementId
  = Identity(recovery-quiescence-domain, QuiescenceStatement).
```

The run resource contract is a finite host-call/checkpoint budget distinct from
the semantic machine resource envelope, though it may be a checked restriction
of it. Before reservation it MUST be partitioned as
`ExecutionResourcePartition`. Every sub-slice is a linear budget token: it is
consumed exactly once by its named stage and cannot be reused by a later stage.
The finalization and recovery partitions MUST NOT be consumed by ordinary
preflight, execution, search, or effect authorization; their atomic-publication
slices remain untouched until the coordinator CAS. The partition warrant MUST
prove that every event reachable
after reservation can be staged, checked, and atomically committed as an exact
terminal, productive, safe partial, or quarantined result within those slices.
If that proof is absent, execution is rejected before reservation.

Before reservation, `ResolveAndVerifyRecoveryPolicy` MUST prove that for
every effect-intent state reachable under that run contract, the identity-bound
recovery contract either (a) completes, compensates, or deduplicates it; (b)
commits a safe partial or quarantined terminal state within its fixed finite
bound; or (c) commits a resumable `RecoveryCheckpoint` whose well-founded
progress measure strictly decreases. Eventual-completion claims additionally
bind the fairness premise and prove that checkpoint descent reaches (a) or (b).
A contract that can exhaust while retaining a fence without either a safe
terminal transition or such a checkpoint is rejected before execution. The verified recovery policy
and contract, their fence-safety warrant, their bounded safe-terminal liveness
warrant, the resource partition, every immutable original execution object,
and an object-retention proof are committed by the typed `RecoveryBundleBody`
and `RecoveryBundleRoot` in the reservation. The retention warrant MUST state
the durable fault domain and prove that the bundle remains resolvable for the
complete reservation/recovery lifetime. If that guarantee is lost, a redundant
identity-bound emergency transition MUST still be able to commit a quarantined
safe state; otherwise the execution profile is not admissible. Recovery does
not assume a new ambient resolution event.
Identical replay returns the immutable original execution result without
repeating effects. Reuse with a changed body is rejected before execution.

Before any run action, the coordinator atomically changes
`available(ExpectedDeploymentConfiguration)` to
`reserved(ExecutionReservation)`. The reservation commits the exact
effect-intent ledger, recovery policy, and a `running` fence; it is serialized state, not an
in-memory lock. Every normal return atomically replaces it with
`available(successor DeploymentConfiguration)` and records the immutable result
under the execution transaction key.
`RECOVER_EXECUTION(CoordinatorState,RecoveryRequest)` equality-checks the exact
reserved head, effect-intent root, recovery policy, and finite recovery resource
contract, then first compare-and-swaps `running` to a transaction-bound
`recovering` fence. It performs no recovery-visible action before that fence.
The executor proves possession of the current `running` fence at every effect
boundary. Every effect is mediated by a persisted sink-enforced permit binding
the running fence and intent identity: the sink either atomically rejects a
stale fence, or the profile proves lease quiescence/deduplication/declared
at-least-once handling. Thus a coordinator intent CAS alone never authorizes an
unguarded later effect. The typed `EffectIntentEntry` commits the exact
authorization statement and proof; the typed `EffectPermit` is consumed by the
declared sink. These same types and checks govern retry, compensation, and
recovery effects. An intent already in flight at fencing is handled only
through that same proved mechanism, and recovery waits for its declared
quiescence boundary before finalization. After acquiring the recovery fence, a
bounded fence-specific quiescence stage MUST observe the persisted sink permits
and either prove that every pre-fence permit is rejected, quiesced, deduplicated,
or classified by the declared at-least-once rule, or return a resumable recovery
checkpoint. A precomputed universal warrant alone is not evidence that this
particular fence has reached quiescence. Recovery
then reconciles every effect according to
`EffectCommitModel_X`. It completes, compensates, marks a partial result, or
quarantines before releasing the reservation. One CAS writes the original
execution result under its execution/continuation key, the recovery receipt
under `recovery(RecoveryTransactionId)`, and the available successor deployment.
Same-body replay returns the original recovery result; a changed body is
rejected as identity reuse, while a changed head conflicts. No other run
may bypass an unresolved reservation. Publication of local runtime/configuration state is atomic. External
effects follow the exact `EffectCommitModel` in `X`; they may be partially
visible only when that behavior is declared in `Exec_X`, costed, and recorded in
the successor effect ledger and receipt. A conflict MUST NOT hide an effect that
already occurred. Concurrent executions MUST be
serializable in a bound order, or every permitted interleaving MUST appear in
`X` and be covered by exactness, feasibility, outcome aggregation, and cost.
Fallback or retry after an observable effect requires proved compensation,
idempotence, deduplication, or the explicitly required at-least-once or
exactly-once semantics.

An identical execution replay that encounters its own matching persisted
reservation returns `recovery-required` with an identity-bound recovery-request
template. It MUST NOT record a `no-run`, assert `NoEffectProof`, repeat the
system, or overwrite the original execution key. Only `RECOVER_EXECUTION` may
finalize that reservation. A result under the original key and the same head
still being reserved is an atomicity-integrity violation, because conforming
finalization publishes the result and releases the head in one transition.

Publishing a candidate updates only `KnowledgeHead`. An unsealed candidate MUST
NOT become executable. Resealing and publishing a seal is a separate
full-head-CAS transition:

```text
ResealRequest = (
  ResealTransactionId, ExpectedKnowledgeHead,
  SnapshotId, RequestedSealObligations,
  ResealWork,
  ResealResourceContractId
)

ResealWork =
    fresh(PostIdEvidenceRoot)
  | resume(OriginalResealTransactionId, SealCheckpointRoot,
           ResourceContinuationTokenId)

ResealResult =
    published-reseal(published(SnapshotId,new SealId),
                     SealCertificate,ResealReceipt)
  | duplicate(ResultId,original nonduplicate ResealResult)
  | rejected(Reason)
  | checkpoint(SealCheckpoint,ObligationIds)
  | incomplete(ObligationIds)
  | incoherent(ConflictProof)
  | unresolved(DependencyIds)
  | conflict(CurrentKnowledgeHead)
  | unsupported(FeatureIds)
  | resource-exhausted(ResourceReport)
  | internal-failure(FailureReport).

PUBLISH_RESEAL(CoordinatorState, ResealRequest, SnapshotCandidate)
  -> (successor CoordinatorState, ResealResult).
```

Here and in `ACTIVATE_SEAL`, `head-unchanged successor CoordinatorState` means
the semantic knowledge/deployment heads are byte-for-byte unchanged while the
transaction ledger may advance exactly once with the immutable terminal result.
It never means the entire `CoordinatorState` value is unchanged.
For `published-reseal` and `activated`, the returned coordinator is the one exact
successor containing the new semantic head and immutable ledger entry. Every
other `ResealResult` or `ActivationResult` preserves the semantic head and may
only add its permitted idempotency entry. Thus both procedures uniformly return
one `(successor CoordinatorState,result)` pair.

`ResealTransactionId` is the identity of the remaining request body. The
procedure verifies the same exact `SnapshotId`, invokes the one
normative `VERIFY_SEAL` path, leaves `SnapshotBody`/`SnapshotId` unchanged, and
atomically compares/replaces the entire `KnowledgeHead`. Reuse with a changed
request is rejected; replay returns the immutable original nonduplicate result
and its stored identity, including a checkpoint result when that was the
original outcome. Seal
publication does not activate a deployment.

Activating a sealed snapshot is an identity-bearing, idempotent transaction:

```text
ActivationRequest = (
  ActivationTransactionId,
  ExpectedDeploymentConfiguration?,
  SnapshotId, SealId, MigrationProfileId,
  TargetDeploymentLineageOrCreatePolicy,
  ActivationResourceContractId
)

ActivationTransactionId
  = Identity(activation-transaction-domain,
             ActivationRequest excluding ActivationTransactionId)

ActivationResult =
    activated(new DeploymentConfiguration,ActivationReceipt)
  | duplicate(ResultId,original nonduplicate ActivationResult)
  | rejected(Reason)
  | incoherent(ConflictProof)
  | unresolved(DependencyIds)
  | conflict(CurrentDeploymentHeadState)
  | incomplete(ObligationIds)
  | unsupported(FeatureIds)
  | resource-exhausted(ResourceReport)
  | internal-failure(FailureReport).

ACTIVATE_SEAL(CoordinatorState, ActivationRequest,
              SealedSnapshot, MigrationProfile)
  -> (successor CoordinatorState, ActivationResult).
```

Successful knowledge, reseal, and activation transitions use the following
acyclic receipt carriers. A receipt body binds the exact request and before/after
state; it never contains its own identity or the result that embeds it.

```text
UpdateReceiptBody = (
  TransactionId, UpdateRequestBodyRoot,
  BeforeKnowledgeHead, AfterKnowledgeHead,
  ExpectedRuntimeStateRoot?, SnapshotId, PublishedSealId?,
  AdmissionReportId, CoordinatorAtomicityProfileId
)
UpdateReceiptId = Identity(update-receipt-domain,UpdateReceiptBody)
UpdateReceipt = (UpdateReceiptBody,UpdateReceiptId).

ResealReceiptBody = (
  ResealTransactionId, ResealRequestBodyRoot,
  BeforeKnowledgeHead, AfterKnowledgeHead,
  SnapshotId, SealId, SealCertificateId,
  CoordinatorAtomicityProfileId
)
ResealReceiptId = Identity(reseal-receipt-domain,ResealReceiptBody)
ResealReceipt = (ResealReceiptBody,ResealReceiptId).

MigrationReceiptBody = (
  ActivationTransactionId, MigrationProfileId,
  BeforeDeploymentHeadState, AfterDeploymentConfiguration,
  StateMigrationEvidenceRoot, RetainedArtifactMigrationEvidenceRoot,
  EffectLedgerMigrationEvidenceRoot, RuntimePolicyMigrationEvidenceRoot,
  ChargedMigrationTraceRoot
)
MigrationReceiptId = Identity(migration-receipt-domain,MigrationReceiptBody)
MigrationReceipt = (MigrationReceiptBody,MigrationReceiptId).

ActivationReceiptBody = (
  ActivationTransactionId, ActivationRequestBodyRoot,
  BeforeDeploymentHeadState, AfterDeploymentConfiguration,
  SnapshotId, SealId, MigrationReceiptId,
  CoordinatorAtomicityProfileId
)
ActivationReceiptId = Identity(activation-receipt-domain,ActivationReceiptBody)
ActivationReceipt = (ActivationReceiptBody,ActivationReceiptId).

CoordinatorAtomicityProfile = (
  FullValueCompareExchangeSemantics,
  HeadLedgerAndImmutableObjectSingleLinearizationDomain,
  DurabilityAndCrashVisibilitySemantics,
  SameBodyReplayAndDifferentBodyCollisionSemantics
)
CoordinatorAtomicityProfileId
  = Identity(coordinator-atomicity-profile-domain,CoordinatorAtomicityProfile).

CoordinatorTransactionIngressProfile = (
  CoordinatorTransactionIngressProfileId,
  FixedUpdateResealActivationHeaderGrammars,
  StrictStreamingIdentityAndLedgerLookupBootstrapEnvelope,
  TrustedFreshCoordinatorTransactionCapabilityIssuerProfileId,
  PublicationSliceSufficiencyAndLinearUseWarrant
).

CoordinatorTransactionAttemptId = Identity(
  coordinator-transaction-attempt-domain,
  TrustedFreshCoordinatorTransactionCapabilityIssuerProfileId,
  TrustedIssuerEpoch,AtomicallyUniqueMonotoneAttemptSerial,
  RequestKind,ExactRequestBodyRoot).

CoordinatorTransactionAllocationReceipt = (
  CoordinatorTransactionAttemptId,
  TrustedFreshCoordinatorTransactionCapabilityIssuerProfileId,TrustedIssuerEpoch,
  PriorAttemptSerial,SuccessorAttemptSerial,
  RequestKind,ExactRequestBodyRoot,
  FreshnessAndNonaliasingStatementId
).

CoordinatorPublicationCapabilityId = Identity(
  coordinator-publication-capability-domain,
  CoordinatorTransactionAttemptId,IssuerAllocatedPublicationSerial).

CoordinatorPublicationSlice = live-coordinator-publication(
  CoordinatorTransactionAttemptId,IssuerAllocatedPublicationSerial,
  CoordinatorPublicationCapabilityId).

CoordinatorPublicationDispositionAndSufficiencyStatement = (
  CoordinatorTransactionAttemptId,CoordinatorPublicationCapabilityId,
  ExactMaximumRequestResultReceiptAndAtomicObservationBounds,
  ExactlyOnePublishOrAffineDisposeTransitionStatementId
).

CoordinatorPublicationDispositionAndSufficiencyStatementId = Identity(
  coordinator-publication-disposition-sufficiency-domain,
  CoordinatorPublicationDispositionAndSufficiencyStatement).

AffineDispositionAndSufficiencyWarrant = (
  CoordinatorPublicationDispositionAndSufficiencyStatement,
  CoordinatorPublicationDispositionAndSufficiencyStatementId,
  ProofId,VerifierResult=accept(VerifiedStatementIds containing
    CoordinatorPublicationDispositionAndSufficiencyStatementId)
).

`live-coordinator-publication` is a primitive affine capability. Its only legal
terminal transition is the one atomic success/terminal-result publication or an
exact no-publication disposition proof; the immutable warrant/receipt cannot
recreate it.

CoordinatorTransactionLedgerObservation = (
  TransactionKey,TransactionLedgerEntry?,CoordinatorObservationVersion
).

CoordinatorTransactionResourcePartition = (
  CoordinatorTransactionAttemptId,
  CoordinatorPublicationSlice,
  CoordinatorTransactionAllocationReceipt,
  AffineDispositionAndSufficiencyWarrant
).

CoordinatorTransactionIngressResult =
    complete(ExactRequestBody,RecomputedTransactionId,
             CoordinatorTransactionLedgerObservation,
             CoordinatorTransactionResourcePartition)
  | rejected(Reason)
  | incoherent(ConflictProof)
  | unresolved(DependencyIds)
  | unsupported(FeatureIds)
  | resource-exhausted(ResourceReport)
  | internal-failure(FailureReport).
```

Every success path MUST construct these exact bodies from the values participating
in the winning atomic transition, recompute the displayed identity, and pass the
body—not a type name or ambient object—to the transaction-ledger builder.
`CoordinatorAtomicityProfileId` is resolved from and equality-checked against the
implementation dependency manifest before any coordinator transition.
`CoordinatorTransactionIngressProfile` is in that same foundation. Its bounded
fixed-header ingress strictly streams/recomputes the request-body identity and
performs the first ledger observation before any variable-size object is decoded.
`BoundedCoordinatorTransactionIngress` returns exactly
`CoordinatorTransactionIngressResult`. On `complete`, its trusted generative
issuer operates on the supplied full `CoordinatorState`, atomically advances
the exact coordinator-transaction issuer-map serial and state version, and returns a fresh
affine resource partition; the immutable allocation receipt proves freshness
but cannot recreate its live capability. Concurrent identical calls therefore
receive distinct publication capabilities. The slice is affine: exactly one new
success or terminal-result publication consumes it; an early malformed,
duplicate, collision, or no-write return disposes it with an exact no-publication
proof. Work-resource exhaustion cannot consume it, and no branch may copy,
persist, reconstruct, or reuse it. The issuer's private serial advance is only
capability-allocation metadata, not a knowledge/deployment/transaction-ledger
publication. Every result branch not carrying `complete` has already disposed
the ingress attempt internally and exposes no live publication authority.

The migration profile proves or performs all typed state, retained-artifact,
capability, effect-ledger, and runtime-policy/quarantine migration and binds
their costs. The activation verifies every request/profile identity and source
seal scope, computes the complete successor, then compare-and-swaps the entire
expected deployment value or the exact absence required by a create policy. It
MUST compute migration as a pure or isolated staged transition before CAS;
every observable allocation/effect becomes visible only atomically with the
winning deployment-head/receipt commit, or the activation profile is invalid.
There is no unreported pre-CAS migration effect. The winning transition
atomically creates the new deployment version and immutable receipt in the
transaction ledger; an identical replay returns `duplicate`. A conflict or any
other non-success branch publishes no deployment, migration, effect, or
quarantine state and preserves every concurrent semantic change. It may
atomically append only its exact immutable idempotency-ledger result, as
specified above. It never silently inherits state from a different snapshot or
seal.
An activation may clear an existing quarantine only when its migration profile
contains an accepted exact investigation, replacement, or recertification
statement naming that quarantined binding and proving the successor executable.
The full deployment-head CAS then removes or supersedes the quarantine entry.
Ordinary execution, crash recovery, reseal, and knowledge publication cannot
clear it.

Lifecycle objects are also distinct:

```text
PendingUpdateBody = (
  CheckpointTarget,
  UpdateRequestBody,
  BaseSnapshotIdOrGenesis,
  StageTag,
  StageStateBinding,
  ProcessedWorkRoot, PendingWorkRoot,
  ImmutableStageOutputRoots,
  ImplementationId, VerifierId,
  ProgressMeasureAndWarrant
)

StageTag =
    decode
  | base-and-runtime-validation
  | transition-resolution
  | dependency-resolution
  | admission
  | least-closure
  | quotient
  | accumulation
  | derivation-artifacts
  | pre-identity-body
  | post-identity-impact
  | post-identity-seal.

StageStateBinding =
    decode(ExpectedDeltaId, DeltaInputDescriptorId,
           StreamingDecodeAndHashStateRoot?)
  | base-and-runtime-validation(
      ExpectedKnowledgeHead, ExpectedRuntimeStateRoot?)
  | transition-resolution(
      TransitionProfileId, DeltaId, BaseRoot, BasePendingRoot)
  | dependency-resolution(
      TransitionProfileId, RuleTrustUniverseId, DeltaId,
      TransitionedBaseRoot, TransitionedPendingRoot)
  | admission(
      TransitionProfileId, RuleTrustUniverseId, DeltaId,
      TransitionedBaseRoot, TransitionedPendingRoot,
      ResolvedDependencyRoot)
  | semantic-stage(
      SemanticSubstage,
      TransitionProfileId, RuleTrustUniverseId, DeltaId,
      AdmissionReportId, PendingDeclarationRoot,
      ActiveBaseRoot, PriorSemanticOutputRoots)
  | post-candidate-stage(
      PostCandidateSubstage,
      SnapshotId, TransitionDiffRoot?, ImpactRoot?,
      RequestedSealObligationRoot, PriorPostIdentityOutputRoots)

SemanticSubstage =
    closure-completion
  | quotient-construction
  | contribution-indexing
  | accumulation-fold
  | derivation-space-construction
  | artifact-dependency-commit
  | snapshot-body-construction.

PostCandidateSubstage =
    impact-analysis
  | post-id-evidence-construction
  | seal-verification(SealSubstage)
  | sealed-prepare-proof
  | unsealed-prepare-proof.

CheckpointRoot
  = Identity(pending-checkpoint-domain, PendingUpdateBody)

PendingUpdate = (PendingUpdateBody, CheckpointRoot)
  // no exact closure claim

ProgressMeasureAndWarrant =
    safe-progress(StrictStateAdvanceProof)
  | terminating-progress(WellFoundedMeasure, StrictDecreaseProof,
                         FairnessAndBoundProof)

UnsealedSnapshot
  = exact SnapshotCandidate with fixed least closure and quotient,
    but incomplete requested operational seal obligations

SealedSnapshot
  = SnapshotCandidate plus successful SealCertificate and SealId.
```

`StageTag` follows the normative order decode, base-and-runtime-validation,
transition-resolution, dependency-resolution, admission,
least-closure, quotient, accumulation, derivation-artifacts,
pre-identity-body, post-identity-impact, or post-identity-seal.
The `StageStateBinding` constructor is determined by `StageTag`: `decode`,
`base-and-runtime-validation`, `transition-resolution`,
`dependency-resolution`, and `admission` use their like-named constructors;
`least-closure` through `pre-identity-body` use `semantic-stage`; and the two
post-identity tags use `post-candidate-stage`. Within a broad tag, the substage
is mandatory and unique: `accumulation` admits only `contribution-indexing` then
`accumulation-fold`; `derivation-artifacts` admits only
`derivation-space-construction` then `artifact-dependency-commit`; and
`post-identity-seal` admits post-ID evidence, the exact `SealSubstage`, sealed
prepare-proof, or unsealed prepare-proof as applicable. `RESUME_UPDATE` and
`ValidateCheckpoint` dispatch exhaustively on `(StageTag,substage)` and its
unique successor; roots or prose labels cannot substitute for that program
counter. A field that does not yet exist
at a stage is therefore absent by type, not filled from an ambient default.
Every stage through `pre-identity-body` requires a `pre-candidate(TransactionId)`
target; `post-identity-impact` and `post-identity-seal` require
`post-candidate(SnapshotId)`. The request
body's transaction identity and any prospective snapshot identity MUST equal
the target payload exactly.
`ImmutableStageOutputRoots` contains every exact output required to reproduce or
continue that stage and nothing asserted by a later stage. `CheckpointRoot`
commits the body without self-inclusion. `RESUME_UPDATE` MUST revalidate it and may advance only
through the normative next-stage transition; an omitted required root makes the
checkpoint malformed rather than an invitation to consult ambient mutable
state. `safe-progress` forbids a stuttering checkpoint but makes no eventual
completion claim. Only `terminating-progress` supports a liveness or bounded
completion claim, and only under its committed fairness and resource premises.

A partial saturation prefix is never placed in a `SnapshotBody` field claiming
`Cl_U(B)`. `GENESIS`, `UPDATE`, and `RESEAL` MUST preserve these distinctions;
`RESEAL` changes only the seal layer when the candidate body is unchanged.

A `maintained-use-case-class` seal binds a nonempty `UseCaseClassId`. Each
published successor either recertifies the exact answer for every use-case in
that class—by materialization or one class-wide theorem—or does not carry that
maintenance claim. Incremental maintenance MUST equal full recomputation for
the new candidate. Evidence accumulation may be monotone; selected policies,
frontiers, and objective values are not.

---

## 8. Queries, admission, and execution

### 8.1 Query tuple

An operation query is

```text
q = (
  QueryId, ProfileId, ProblemId,
  InvocationScope,
  TargetObservation, CandidateObservation,
  QueryTargetRelation,
  Init, MachineId,
  ResourceEnvelope, SystemUniverseId,
  RestrictedUniverseId?,
  CostModelId, CostParameters,
  ContinuationClass, ClaimRequest,
  ResultMode,
  TiePolicy?, PreferencePolicy?
).
```

Every field that can change admissibility, correctness, cost, or selection is
semantic. Correctness and evaluation are distinct:

```text
InvocationScope_q = (
  CorrectnessDomain_q,
  EvaluationDomain_q,
  EvaluationProfile_q
).
```

`CorrectnessDomain_q` is a nonempty exact subset of `ValidInvocation_P`.
Exactness, eligibility, hard-resource, state, effect, failure, and completion
obligations quantify over every `z` in that domain and every maximal execution.
`EvaluationDomain_q` is a nonempty exact indexed subfamily used only for the
declared cost/workload evaluation and MUST lie in the correctness domain.
`EvaluationProfile_q` is exactly one of

```text
point(z)
indexed-family(I, z_I)
probability-space(D, Sigma, mu).
```

The mode's carrier is not independent metadata:

```text
point(z) implies EvaluationDomain_q = singleton(z)

indexed-family(I,z_I) implies EvaluationDomain_q is exactly that indexed
  family, including declared order, repeated occurrences, and multiplicity

probability-space(D,Sigma,mu) implies D is exactly the underlying carrier of
  EvaluationDomain_q and Sigma is a sigma-algebra on that carrier.
```

A probability profile binds its measurable space, normalized exact measure,
conditioning and correlation rules, measurability of every evaluated cost, and
integrability or an explicit extended-cost convention. Sampling, support, or
an almost-sure qualifier MUST NOT narrow universal correctness silently. A
content-specific query uses a singleton correctness and evaluation domain.

The query-induced problem is

```text
Problem_q = (
  ProblemId, ProfileId,
  CorrectnessDomainId,
  TargetObservation_q,
  CandidateObservation_q,
  QueryTargetRelation_q,
  MandatoryStateEffectFailureAndCompletion_P
).

TargetObs_q(z,sigma)
  = MapBehavior(
      TargetObservation_q o RequiredObservation_P,
      TargetBehavior_P(z,sigma))

CandidateObs_q(B)
  = MapBehavior(CandidateObservation_q,B)

Accept_q(z,sigma,B)
  = QueryTargetRelation_q(
      CandidateObs_q(B),
      TargetObs_q(z,sigma)).
```

Both observation functions are well-typed deterministic total functions and
respect the exact target equivalence. Their values MAY be relation-,
distribution-, interval-, or refinement-valued objects.
`QueryTargetRelation_q` is a total typed proposition between the candidate's
complete observed behavior and the target observed behavior. For every
invocation its acceptable result class MUST be nonempty. The problem retains
all mandatory state, effect, failure, and completion semantics; projection
cannot hide them.

`ProblemId` commits to exactly these correctness fields and excludes machine
costs, objective, result policy, discovery state, and selected candidate. A
complete candidate MAY implement `Problem_q` directly; it need not materialize
unobserved output of `P`. A full `P` realization is a special case.

`ClaimRequest_q` is

```text
ClaimRequest_q = (
  RequiredClaimClass,
  RequiredClaimPredicate,
  PermittedFallbackClaims,
  OnOptimizationIncomplete
).
```

`RequiredClaimPredicate` is the exact typed proposition schema requested, not a
label chosen after optimization. For member/complete-set claims it fixes the
problem/workload objective and requested membership or set-equality statement.
For a comparison claim it binds

```text
ComparisonRequest = (
  ComparisonKind,
  ComparatorAndAdversaryProfileIds,
  CandidateQuantifier =
    system-free | find-one | specific(CandidateId) | forall-systems,
  ExactInequalityOrOrderTemplate,
  Thresholds, AlphaBetaOrConstants,
  Size/Horizon/Statistic/LimitConvention,
  ExtendedValueAndUnitsPolicy
).
```

Only fields applicable to the named comparison kind are populated, and their
schema proves total interpretation. The returned statement MUST be exactly this
predicate with only an authorized `find-one` candidate variable instantiated;
`system-free` and `forall-systems` statements carry no selected realization;
the resolver cannot choose a weaker threshold, favorable constants, smaller
domain, or different asymptotic convention post hoc. `execute-certified` for a
comparison request requires the verified returned statement to name exactly one
admitted complete system.

```text
OneSystemComparisonBoundClass =
  competitive-bound | instance-optimal(alpha,beta) | asymptotic-bound.

AttainedComparisonOptimumClass =
  competitive-optimal | asymptotic-optimal.
```

These are the canonical members, not a closed-world ban on admitted comparison
profiles. Every query-local comparison profile allowed below MUST register

```text
ComparisonResultShape =
    one-system-bound | attained-scalar-optimum | system-free-theorem
```

and its new base class is treated as a member of the corresponding set above or
as `comparison-theorem`. The same candidate-quantifier, result-shape,
certificate, and execution-cardinality rules then apply. A profile with no
registered shape is invalid.

Every class in either set requires `find-one` or `specific(CandidateId)` and the
result names one admitted complete system. Only the first set has
`ComparisonStatement.one-system-bound` shape; the second has
scalar-optimal-member shape.
`comparison-theorem` requires
`system-free` or `forall-systems`; it cannot be labeled as a one-system bound or
attained optimum and is never executable unless a separate verified
specialization creates a new one-system request/certificate. All other
class/quantifier combinations are invalid.

For a workload/use-case request, `ComparisonKind`, comparator/adversary profile
identities, size/horizon/statistic/limit conventions, units, extended-value
policy, quantifier order, and objective/order MUST equality-check as projections
of `W` and its `ComparatorProfile`; they cannot modify `WorkloadId` semantics.
Only candidate quantification and thresholds/constants that the workload
explicitly declares request-parameterized may be supplied by `u`. Any other
change defines a new workload. Query comparisons obey the analogous explicit
query-comparison profile rule above.

`PermittedFallbackClaims` is an exact set of `(ActualClaimClass,ClaimPredicate)`
pairs, not bare labels. Every fallback must satisfy one of those complete pairs.

Claim kind and universe scope are orthogonal:

```text
UniverseScope =
    complete(SystemUniverseId | PolicyUniverseId,
             derivation = direct |
               omission-bridge(restricted id,
                               BranchSpecificOmissionBridgeId))
  | restricted(RestrictedUniverseId | RestrictedPolicyUniverseId)

RequestedUniverseScope =
    complete(SystemUniverseId | PolicyUniverseId)
  | restricted(RestrictedUniverseId | RestrictedPolicyUniverseId)

EvidenceScope =
    discovered(DiscoveredSetId, ExactMembershipAndIdentityProfileId)
  | tested(TestedSetId, TestCoverageAndMeasurementProfileId)
  | heuristic(HeuristicPolicyId, EligibleCandidateSetId).

ActualClaimScope =
    universe(UniverseScope)
  | evidence(EvidenceScope).

RequestedClaimScope =
    universe(RequestedUniverseScope).

ActualClaimClass = (BaseClaimClass, ActualClaimScope).
RequiredClaimClass = (RequestableBaseClaimClass, RequestedClaimScope).
```

Both complete-universe derivations satisfy a request for the named complete universe;
the bridge form records how coverage was proved. Restricted scope never
satisfies a complete-scope request. Scope identity, base class, and exact claim
predicate must all match in `ValidQueryAnswers`, `ValidAnswers`, and fallback
authorization.

An evidence scope satisfies only an identical precommitted fallback pair. It is
not a legal requested scope and never satisfies or promotes to a complete or
restricted universe claim. In particular a discovered/tested candidate set is
not silently converted into `U_sys`, `U_policy`, `U_D`, or `U_PD`.

Thus a restricted Pareto member, frontier-complete set, scalar member, and
argmin-complete set retain different base claim classes even if they share one
restricted carrier. `restricted-universe-optimal` is only a legacy input alias
normalized before well-formedness checking to
`(global-optimal,universe(restricted(id)))`; it is not a `BaseClaimClass`, is never
emitted as one, and cannot be paired with complete scope. It is not a
replacement label for every restricted result shape. `ClaimRequest` and
permitted fallbacks match both components.

`OnOptimizationIncomplete` is `do-not-execute` or an exact fallback selection
policy over an explicitly identified `EvidenceScope`. A weaker execution is
forbidden unless its actual claim class and the identical discovered, tested, or
heuristic evidence scope are permitted here; that evidence carrier MUST NOT be
presented as a complete or restricted universe.

`ResultMode` is one of

```text
return-certified-member | return-scalar-member | return-argmin |
return-pareto-member | return-frontier |
return-bound | execute-certified | execute-selected.
```

The mode and requested claim MUST be type-correct:

- `return-certified-member` requests one admitted complete system satisfying an
  exact nonoptimal member predicate (`exact` or a compatible registered
  profile-defined class); it asserts only that predicate and operational scope;
- `return-scalar-member` requests one admitted scalar-optimal member and its
  membership proof; it does not assert the complete argmin;
- `return-argmin` requests the identity-complete scalar argmin;
- `return-pareto-member` requests one admitted nondominated member and its
  no-dominator proof; it does not assert the complete frontier;
- `return-frontier` requests the identity-complete nonempty frontier;
- `return-bound` requests an exact comparison statement such as a competitive,
  instance, or asymptotic bound and any system named by that statement;
- `execute-certified` executes the exact member/system already present in a
  certified member or bound answer and makes no complete-set selection claim;
- `execute-selected` requires a complete nonempty argmin or frontier and applies
  the bound selection policy to that complete identity set.

All nonselection return modes and `execute-certified` MUST omit both policies.
Scalar `execute-selected` MUST bind a deterministic total `TiePolicy` on every
complete nonempty argmin in its domain and omit `PreferencePolicy`. Partial
`execute-selected` MUST bind a deterministic total `PreferencePolicy` on every
complete nonempty frontier and omit `TiePolicy`. Every other combination is an
invalid query. Preference chooses among nondominated identities and does not
change the objective or frontier; it is not a scalar tie-break. A member answer
may be globally/Pareto optimal without proving which member a complete-set
policy would select.

Well-formedness computes one unambiguous `RequestedAnswerShape` from
`(ResultMode, RequiredClaimClass, ObjectiveOrder)`:

```text
ComparisonStatement =
    one-system-bound(R, ExactInequalityOrOrderStatement)
  | system-free-theorem(ExactUniversalOrSystemFreeStatement).

return-certified-member                      -> CertifiedMember
return-scalar-member                         -> ScalarMember
return-argmin                                -> CompleteScalarArgmin
return-pareto-member                         -> ParetoMember
return-frontier                              -> CompletePartialFrontier
return-bound + find-one/specific candidate   -> ComparisonStatement.one-system-bound
return-bound + system-free/forall-systems    -> ComparisonStatement.system-free-theorem

execute-certified + exact/registered nonoptimal member class
                                              -> CertifiedMember
execute-certified + scalar optimum class     -> ScalarMember
execute-certified + Pareto member class      -> ParetoMember
execute-certified + OneSystemComparisonBoundClass
                                              -> ComparisonStatement.one-system-bound

execute-selected + scalar order/complete class
                                              -> CompleteScalarArgmin
execute-selected + partial order/complete class
                                              -> CompletePartialFrontier.
```

“Scalar optimum class” includes `global-optimal`, `use-case-global-optimal`,
`competitive-optimal`, `asymptotic-optimal`, and another explicitly scalar
attained-optimum class;
“one-system comparison-bound class” includes `competitive-bound`,
`instance-optimal(alpha,beta)`, and `asymptotic-bound`;
`comparison-theorem` has only the system-free-theorem tag. `argmin-complete`, `frontier-complete`, and
their workload forms are the complete classes. An unsupported combination is
an invalid request, not an optimizer choice.

Claim classes are kinded by the carrier quantified by their proposition:

```text
BaseClaimClass =
    QueryResultBaseClaim
  | WorkloadResultBaseClaim
  | OperationalWeakerBaseClaim
  | profile-defined-comparison(ProfileId,ClassId,ComparisonResultShape).

QueryResultBaseClaim =
  global-optimal | argmin-complete | Pareto-optimal | frontier-complete.

WorkloadResultBaseClaim =
  use-case-global-optimal | workload-argmin-complete |
  workload-Pareto-optimal | workload-frontier-complete |
  competitive-bound | competitive-optimal |
  instance-optimal(alpha,beta) | asymptotic-bound | asymptotic-optimal |
  family-optimal.

OperationalWeakerBaseClaim =
  exact | comparison-theorem | best-known | measured-best-among-tested |
  heuristic-selected.

RequestableBaseClaimClass =
    QueryResultBaseClaim
  | WorkloadResultBaseClaim
  | exact
  | comparison-theorem
  | profile-defined-comparison(ProfileId,ClassId,ComparisonResultShape).

FallbackOnlyBaseClaimClass =
  best-known | measured-best-among-tested | heuristic-selected.

ArtifactClaimClass = normal-form | canonical | representation-minimal.

ArtifactClaimScope =
    semantic-universe(SemanticUniverseId,SemanticSubjectId?)
  | representation-fiber(RepresentationUniverseId,SemanticSubjectId,
                         RepresentationObjectiveId?).

query-result classes:
  global-optimal, argmin-complete, Pareto-optimal, frontier-complete

workload/use-case classes:
  use-case-global-optimal, workload-argmin-complete,
  workload-Pareto-optimal, workload-frontier-complete,
  competitive-bound, competitive-optimal,
  instance-optimal(alpha,beta), asymptotic-bound, asymptotic-optimal,
  family-optimal

carrier-polymorphic operational exact/weaker classes:
  exact, comparison-theorem, best-known,
  measured-best-among-tested, heuristic-selected

standalone artifact-certificate classes (not BaseClaimClass):
  normal-form, canonical, representation-minimal

family/class capabilities:
  input-total, query-family-answer-complete, pointwise-envelope-complete,
  use-case-class-answer-complete, use-case-class-complete,
  maintained-use-case-class

transition-certificate class:
  revision-preserved
```

The artifact classes, family/class capability names, and `revision-preserved` are not
`BaseClaimClass` inhabitants; they type quantified capability or transition
certificates. Artifact certificates bind `ArtifactClaimScope` and cannot be
smuggled into `ActualClaimClass` or promoted to operation optimality without the
separate machine-specific bridge required by §1.3.
`FallbackOnlyBaseClaimClass` inhabitants may occur only in an exact
`PermittedFallbackClaims` pair with `ActualClaimScope=evidence(...)`; they are
not legal `RequiredClaimClass` values.

A request MUST use a class whose carrier kind matches the request. In
particular, competitive, instance, asymptotic, and family comparisons require a
bound workload/use-case comparison profile; they are invalid as bare operation
queries. A query-local comparison class may be added only by an explicit typed
query-comparison profile that supplies the same comparator, quantifier, and
extended-value obligations as §9.5. Family/class capabilities are certified by
their own quantified certificate branch and are never returned as a single
query or workload member. Cross-carrier combinations are rejected before
admission or optimization.

`Init_q` is a total map from each correctness-domain invocation to its exact
initial configuration. Let `ResolveMachine(MachineId_q,ResourceEnvelope_q)` be
the machine profile's deterministic total restriction operation. A well-formed
query requires it to return exactly one valid machine contract; write that
contract as `X_q`. Unsupported or invalid resource combinations make the query
unsupported or invalid, not a conveniently smaller execution model. Write
`M_q` for the cost profile resolved by `CostModelId_q`. `CostParameters`
supplies only parameter values whose schema and permitted domain are fixed by
`M_q`. Define

```text
(OutcomeAggregation_q, InvocationAggregation_q, Objective_q)
  = ResolveCostSemantics(M_q, CostParameters_q).
```

These resolved functions are projections of one bound cost model; the query
MUST NOT independently override them. Changing an aggregation, objective, or a
semantic parameter creates a new cost-model/query identity. `SystemUniverseId`
MUST equal `SystemUniverseId(Body(S),Problem_q,X_q)` constructed in §8.3; it is
a commitment/check, not a query-selected candidate list. Notation such as
`Adm_S(q,X)` is well-formed only when
`X = X_q`; `Adm_S(q)` abbreviates that form. The same convention applies to
`M_q` in cost and optimum notation.

The correctness and evaluation domains and every per-invocation acceptable
result class MUST be nonempty for an exact optimization claim. A nullary
operation has one empty input tuple, not an empty domain. Vacuous truth MUST NOT
establish exactness, coverage, dominance, or optimality.

Cost staging is fixed: `TraceCompose` first converts each maximal trace to its
trace cost; `OutcomeAggregation_q` then aggregates the complete outcome object
for one evaluation occurrence; `InvocationAggregation_q` finally aggregates
exactly the occurrences/index/law bound by `EvaluationProfile_q`; and
`Objective_q` maps that result into its objective order. A profile that needs a
joint stateful or adaptive aggregation MUST use the workload semantics of §9.4
instead of reordering these stages implicitly.

### 8.2 Machine contract `X`

`X` MUST bind:

- the complete abstract machine and primitive semantics;
- initial memory, cache, persistent, accumulated, and prepared state;
- hard capacity and resource envelopes;
- preprocessing, advice, lookup, communication, parallelism, scheduling, and
  randomness permissions;
- `Exec_X`/`OutcomeModel_X`: the complete outcome domain, trace relation or
  exact law, scheduler/adversary interpretation, controller information,
  fairness assumptions, and maximality convention;
- information available during validation, selection, and execution;
- retained-state and update behavior;
- effect commit, retry, cancellation, compensation, external-effect identity,
  and configuration-concurrency semantics;
- side-channel and effect restrictions;
- the end-to-end accounting boundary.

Unit-cost treatment for an unbounded arithmetic, storage, communication, lookup,
oracle, or verification action is nonconforming unless `X` proves the applicable
bound or the cost model explicitly declares and scopes the abstraction.

### 8.3 Complete candidate universe

For a pre-seal candidate context `C`—or sealed `S` through
`C = SnapshotCandidate(S)`—problem `Problem_q`, and machine `X`, the universe
descriptor MUST bind

```text
SystemUniverseBody = (
  CandidateCarrier,
  CandidateIdentityAndEquality,
  MembershipSemantics,
  GenerationGrammar,
  CompletenessProposition
)

SystemUniverseId
  = Identity(
      system-universe-domain,
      SnapshotId(C), ProblemId_q, Identity(X), SystemUniverseBody)

SystemUniverse = (SystemUniverseId, SystemUniverseBody)

U_sys(Body(C),Problem_q,X)
  = { R in CandidateCarrier | MembershipSemantics(R) }.
```

as the complete outer universe of transformation systems generated by the
grammar and closure rules bound by those three objects. The carrier is an exact
set/type, not a proper-class-sized phrase. `U_sys` is fixed
independently of candidate discovery, selection, and the optimizer's requested
search restriction. It is not the set returned by an index, the set currently
cached, or the candidates encountered by one search, and MUST NOT exclude a
candidate because its cost is unknown, inconvenient, or unfavorable.

For a sealed snapshot, `U_sys(Body(S),Problem_q,X)` abbreviates this candidate
construction. It MAY be finite, infinite, enumerated, symbolically
generated, or structurally defined. A global claim requires complete enumeration, a
proof-complete reduction, or a theorem covering every admitted member.

`CompletenessProposition` MUST prove that the displayed membership set equals
the systems generated by the exact grammar/closure semantics; it cannot cite
`SystemUniverseId`, an optimizer result, or the proposition being proved as its
own premise. The query's `SystemUniverseId` is only an equality check against
this independently constructed descriptor.

A proof-carrying profile MAY define `U_sys` to contain only systems with
certificates accepted by a total verifier. Globality then ranges over that exact
certificate-admitted universe and MUST NOT be restated as a claim over all
semantically correct systems. A deliberately narrow grammar is legal only when
every external claim displays that scope.

A query MAY additionally bind an independently resolved decision subuniverse:

```text
DecisionUniverseBody = (
  DecisionCarrierOrGrammar,
  CandidateIdentityAndEquality,
  DecisionMembershipSemantics,
  InclusionProof,
  DecisionCoverageProposition
)

RestrictedUniverseId
  = Identity(decision-universe-domain,
             QuerySemanticRequestBodyWithoutQueryIdOrRestrictedUniverseId,
             SystemUniverseId, DecisionUniverseBody)

U_D(q)
  = { R in DecisionCarrierOrGrammar |
      DecisionMembershipSemantics(R) }
  subseteq U_sys(Body(S),Problem_q,X).
```

The body is resolved from `RestrictedUniverseId` and equality-checked before
optimization. It is fixed independently of candidate discovery and the result.
The displayed request body contains the independent problem, invocation,
observation, initialization, machine/resource, system-universe, cost,
continuation, claim, result-mode, and policy fields, but omits both `QueryId`
and `RestrictedUniverseId`; after constructing and inserting the restricted ID,
the implementation computes `QueryId` by §4.1. Thus neither identity contains
itself directly or through the outer query identity.
`InclusionProof` covers every member; `DecisionCoverageProposition` states
exactly which grammar/set the restriction represents. An unresolved,
self-referential, optimizer-produced, or merely cached carrier makes the request
invalid or incomplete, never conveniently restricted.

If the inclusion is proper, a result proved only over `U_D(q)` retains its base
member/set/Pareto/bound class and has `UniverseScope=restricted(U_D(q))`; the
scalar shorthand is `restricted-universe-optimal`. This remains true even if
`U_D(q)` is complete under its own grammar. A scalar member becomes
`global-optimal` with complete scope only when
`U_D(q) = U_sys(Body(S),Problem_q,X)` or a
bridge theorem covers every omitted complete system and proves none can improve
or dominate the selection.

### 8.4 Admissibility

A complete system `R` belongs to `Adm_S(q,X)` exactly when:

1. `R in U_sys(Body(S),Problem_q,X)`;
2. `q` and `Problem_q` are valid and, for every
   `z in CorrectnessDomain_q`,
   `SemanticEligibility_R(z,SemanticState(Init_q(z)),S)` is proved true;
3. `Exec_X(q,R,z,Init_q(z))` is the exact nonempty complete execution object for
   every such `z`, and an accepted execution-realization bridge proves that its
   semantic behavior equals/refines the complete declared composed behavior of
   `R` exactly as required by `ChoiceSemantics_P` and `BehaviorConformance_P`;
4. `Accept_q(z,SemanticState(Init_q(z)),Exec_X(...))` is proved and every
   mandatory state, effect, failure, completion, and observation condition holds
   on the complete behavior;
5. every intermediate and final value and productive prefix is well-typed and
   valid;
6. `Feasible_R(q,z,Init_q(z),X)` and every hard capacity, resource, capability,
   schedule, effect-prefix, and intermediate-safety condition are proved for
   every correctness-domain invocation and maximal execution;
7. execution uses only information, facts, state, primitives, and advice allowed
   by `S`, `Problem_q`, and `X`;
8. retained-state and external effects conform to both the problem and `X`,
   including retry, cancellation, compensation, and concurrency semantics;
9. every in-boundary action has a declared cost event or is explicitly free.

Unknown eligibility makes the realization unavailable for an exact selection;
it does not prove ineligibility.

An unknown or undefined cost does not remove an otherwise exact candidate from
this admission set and does not assign it infinity. It prevents an exact
optimization result until `J_q` is total for that candidate. If the target is a
probability distribution, admission compares the complete induced law under
`BehaviorConformance_P` or `QueryTargetRelation_q`; finite samples or support
membership are measured evidence, not exact distributional equality.

### 8.5 Outer system and internal plans

At an end-to-end comparison boundary, the competing realization is the complete
selector-plus-executor beginning from the exact initial state in `X`.
Discovery, resolution, validation, certificate checking, normalization,
planning, dispatch, fallback, execution, output production, receipt generation,
and retained-state effects inside that boundary belong to the run.

An internally selected leaf plan is not a competing complete system unless the
candidate universe explicitly defines it as one with the same starting
information and costs.

`return-argmin` and `return-frontier` are analytic result modes. Their mapping
may vary by query or input and is not itself an executable system. For
`execute-selected`, if selection depends on runtime input, history, machine
observation, or workload state, the compared candidate is the complete
nonanticipating selector-plus-executor and selection is charged. A certificate
of the form `for every z, there exists R_z` MUST NOT be reported as existence of
one uniform executable system.

Candidate-specific prepared artifacts may lie outside this boundary only under
the common prepared-state conditions of §10.8.

### 8.6 Receipts

Every execution carrying an exact claim MUST emit or make reproducible a receipt
binding:

- snapshot; query/operation or use-case-request/workload; and selected complete
  realization/policy identities;
- exact resolved inputs and valid-domain evidence;
- initial and final configuration, retained-state, effect-ledger, and
  runtime-policy/quarantine roots;
- consumed kind, refinement, eligibility, and exactness warrants;
- internal plan identity and complete-system boundary;
- outcome, required observation, trace or certified trace summary;
- realized trace cost, certified aggregated cost, certified objective value,
  and aggregation/objective/policy revisions;
- claim class and optimality/coverage certificate identity;
- implementation and verifier revisions.

For a workload, “inputs” and “trace” mean the complete scenario/environment
interaction and joint trajectory under its filtration and horizon. The receipt
binds the uniform policy, initial/successor deployment states, scenario or
environment-law identity, revealed-information sequence, quantifier prefix, and
whole-workload value evidence; it does not decompose the claim into post-hoc
pointwise query selections.

The receipt records a claim; it is not its own proof.

If a certified trace summary replaces a full trace as evidence, its profile MUST
bind an exact summary grammar, trace commitment, summarized predicates, and a
sound verifier proving that every accepted summary corresponds to a trace with
those predicates. An implementation-defined digest alone is not a certified
summary.

### 8.7 Arbitrary input classification

An input-total profile `I` binds an exact nonempty raw or already-typed input
domain with identity, equality, membership/coverage proposition, and a
deterministic total classifier

```text
Classify_I : RawInput_I ->
  valid(Scenario_I) |
  invalid(reason) |
  unresolved(dependency identities) |
  unsupported(capability).
```

The identity-bearing profile carrier is closed:

```text
InputProfileBody =
    query-family-input(
      SnapshotId,SealId,RawOrTypedInputDomainId,
      InputEqualityAndMembershipProfileId,NonemptyCoverageStatementId,
      TotalClassifierId,FourWayDisjointCoverAndTerminationStatementId,
      InitializationProfileId,AdmittedQueryConstructorId,
      QueryFamilyId,FamilySolverProfileId,
      CombinedClassifierConstructorSolverResourceEnvelopeId,
      ImplementationId,VerifierId)
  | use-case-class-input(
      SnapshotId,SealId,RawOrTypedInputDomainId,
      InputEqualityAndMembershipProfileId,NonemptyCoverageStatementId,
      TotalClassifierId,FourWayDisjointCoverAndTerminationStatementId,
      InitializationProfileId,AdmittedUseCaseConstructorId,
      UseCaseClassId,ClassSolverProfileId,
      CombinedClassifierConstructorSolverResourceEnvelopeId,
      ImplementationId,VerifierId).

InputProfileId = Identity(input-profile-domain,InputProfileBody).
InputProfile = (InputProfileBody,InputProfileId).
```

All field notation on `InputProfile` is a projection from its selected tagged
body. A profile cannot replace the tagged query-family/use-case-class scope,
domain, classifier, constructor, resource, implementation, or verifier after
its identity is computed.

The declared domain MUST contain at least one input; a nullary input domain is
the singleton containing the empty tuple, not the empty set. Empty or
unspecified raw carriers cannot establish `input-total` vacuously.

For a bound input size/resource domain, classification terminates within its
declared resource contract and the four result classes form an exact disjoint
cover. Every `valid` result supplies type membership, a correctness-domain
invocation or workload scenario, and exact initialization. Invalid syntax,
invalid typed content, missing dependencies, and unsupported capabilities MUST
remain distinct.

When `input-total` is combined with a workload or use-case-class capability,
the input profile also binds a deterministic total, snapshot-parameterized
constructor

```text
AdmittedUseCaseOf_(I,K,S) : valid(Scenario_I) ->
  (W, u, scenario) such that
    W is in the exact admitted subset of K,
    u = UseCaseRequestConstructor_(K,S)(W),
    scenario in ValidScenario_W.
```

Its identity, coverage, and equality behavior are certificate fields. Returning
a scenario without determining its workload, objective, machine, policy
universe, and request does not establish that every valid input reaches an
optimal use-case answer.

When combined with `query-family-answer-complete` or
`pointwise-envelope-complete`, the analogous bound constructor is

```text
AdmittedQueryOf_(I,F,S) : valid(Scenario_I) ->
  (q, z, initialization proof) such that
    q in QueryFamily_S,
    z in CorrectnessDomain_q,
    initialization = Init_q(z).
```

It is deterministic, total, identity-bearing, and coverage-complete for valid
inputs. A bare invocation without its query/problem/objective/universe cannot
establish an arbitrary-query family claim.

An `input-total` claim covers every input in that exact raw/typed domain. A
`use-case-class-complete` claim additionally requires that every valid scenario
in the bound class reaches a certified optimum/frontier or an exact proved
negative result; it permits neither `optimization-incomplete` nor an
unauthorized weaker execution. When no interchange profile is bound, this
contract begins at the typed in-memory input boundary and makes no byte-level
claim.

---

## 9. Cost and objective models

### 9.1 Cost model `M`

A cost model MUST bind:

```text
M = (
  EventCostType, Units, EventCost, TraceCompose,
  OutcomeCostType, OutcomeAggregation,
  InvocationCostType, InvocationAggregation,
  ObjectiveType, Objective, ObjectiveOrder,
  PreparationBoundary, AccountingBoundary
).
```

Every trace action permitted by `X` MUST have a typed cost or be explicitly
declared free. The model MUST state operand-size treatment, resource failures,
cold/warm/prepared state, amortization horizon, online/offline work, randomness,
and scheduling treatment where applicable.

For a bound query, `Cost_M(R;q,S,X)` composes event costs and applies exactly
`OutcomeAggregation_q` and `InvocationAggregation_q` resolved in §8.1. No
separate query-local aggregation semantics may be substituted in the value
function, objective, certificate, or receipt.

`EventCost`, both aggregations, and `Objective` MUST be deterministic and total
on their declared domains. Infinite traces, infinite invocation families,
nonintegrable laws, divergence, and hard failure use explicitly declared
extended values or make the cost profile invalid for that query; they never
silently disappear. Probability aggregation binds a measurable space, exact
law, measurability, and integrability/extended-value theorem. Worst-case or
adversarial aggregation binds a nonempty exact outcome set and proves the
required supremum exists in the declared completion. Conditional convergence
preserves order as typed content unless a permutation/regrouping theorem is
bound.

`ObjectiveOrder` is an order on `ObjectiveType` and MUST prove the advertised
total-order, partial-order, or preorder laws. A preorder is quotiented by mutual
comparison before a partial-order or identity-complete frontier claim. For
every `R in Adm_S(q,X)`, exactly one `J_q(R)` MUST exist. Unknown or undefined
cost yields `optimization-incomplete` or an invalid cost profile; it MUST NOT
remove `R` from comparison or assign a convenient infinity.

Costs from different snapshots, machines, units, preparation boundaries,
required observations, or objectives are incomparable unless an explicit
comparison embedding proves their meaning is preserved.

### 9.2 Scalar objective

For complete realization `R`, define

```text
Cost_M(R;q,S,X) in InvocationCostType_M
J_q(R) = Objective_q(Cost_M(R;q,S,X)).
```

The domain of `Objective_q` is exactly `InvocationCostType_M` (or an explicitly
bound total embedding from it into the declared objective input type); no
undefined intermediate `CostType_M` is inferred.

A scalar objective MUST be a deterministic total map into a declared
lower-is-better totally ordered codomain. Exact extraction additionally requires
that comparison is effective or that a proof certificate bypasses search. If
the profile uses an infimum, it MUST also bind an order completion and embedding
in which that infimum exists.

### 9.3 Vector and partial objectives

A vector objective MUST bind a finite ordered component index, typed units, and
an exact lower-is-better order per component. For cost vectors `c` and `d`,

```text
c strictly-dominates d
iff
  c_i <= d_i for every i
  and c_j < d_j for at least one j.
```

Independent per-coordinate lower bounds do not establish joint attainability.
Coordinates may be minimized by different candidates.

### 9.4 Family and workload objectives

A stateful or multi-operation use-case is a joint protocol, not an aggregation
of isolated query values:

```text
W = (
  WorkloadId,
  ScenarioKind, ValidScenario,
  EventAndQueryGrammar,
  InitialDeploymentConfiguration,
  MachineId, ResourceEnvelope,
  CostModelId, CostParameters,
  EnvironmentStrategiesOrLaw,
  ScenarioEnvironmentSelectionProfile,
  InformationFiltration,
  QuantifierPrefix,
  UniformSystemActionGrammar,
  JointTransitionSemantics,
  WorkloadTargetBehavior,
  WorkloadTargetObservation,
  WorkloadCandidateObservation,
  WorkloadAcceptanceOrConformance,
  StopRuleOrHorizon,
  CompletionAndEffectContract,
  TraceCostFunctional,
  ObjectiveAndOrder,
  ComparatorProfile =
      not-applicable
    | comparator(ComparatorBody)
    | absolute-asymptotic(AbsoluteScoreProfile)
).
```

`X_W = ResolveMachine(MachineId_W,ResourceEnvelope_W)` and `M_W` with its
resolved workload cost parameters are part of `WorkloadId` by §4.1. They MUST
be valid and total on the complete workload behavior. The displayed
`QuantifierPrefix` fixes the order of system choice, scenario/environment
choice or law, randomness, outcome/invocation aggregation, horizon, objective,
and comparison. No field may be supplied later by the optimizer.

The displayed cost functional and objective/order are checked projections:

```text
(TraceCostFunctional_W, ObjectiveAndOrder_W)
  = ResolveWorkloadCostSemantics(
      M_W, CostParameters_W, JointTransitionSemantics_W,
      StopRuleOrHorizon_W, QuantifierPrefix_W).
```

They cannot override `M_W`. If this resolution is undefined or nontotal on any
complete competitor behavior, the workload is not a valid exact optimization
profile.

`ValidScenario_W` is nonempty. A scenario may contain arbitrary admitted typed
inputs, operation changes, content additions, queries, failures, schedules, and
environment events. The information filtration specifies exactly what a system
knows before each action. If two histories are indistinguishable under that
filtration, a deterministic policy chooses the same action and a randomized
policy induces the same action law. This is the nonanticipation requirement.

`EnvironmentStrategiesOrLaw_W` is an exact tagged profile. It is either an
exact nonempty typed strategy carrier with legal moves and information at every
history, or a normalized probability law/kernel on a declared measurable
environment space with exact conditioning, correlation, and filtration rules.
It induces the nonempty tagged carrier

```text
EnvContext_W =
    adversarial(strategy in EnvironmentStrategyCarrier_W)
  | stochastic(LawOrKernelContext_W).

BoundScenarioEnvironment_W
  = { (a,c) | ValidScenario_W(a), c in EnvContext_W,
              CompatibleScenarioEnvironment_W(a,c) }.
```

Only the tag admitted by the workload profile is legal. A stochastic context
is the identity-bearing law/kernel and conditioning state, not one sampled
trajectory. An empty adversary set, subnormalized law, sampled trace set, or
unspecified scheduler is invalid for an exact workload claim.
`BoundScenarioEnvironment_W` is exact and nonempty, and every valid scenario
has a nonempty compatible context fiber. This relation—not an assumed Cartesian
product—binds scenario-dependent legal strategies and conditional laws.

`ScenarioEnvironmentSelectionProfile_W` is an identity-bearing exact tag over
that relation:

```text
adversarial(exact nonempty joint choice/strategy carrier)
| deterministic-or-indexed(exact nonempty family and aggregation rule)
| stochastic(normalized joint law/kernel on BoundScenarioEnvironment_W,
             marginal/conditioning/correlation/measurability warrants).
```

It supplies every scenario marginal, joint distribution, or adversarial carrier
consumed by `QuantifierPrefix_W`. The quantifier prefix fixes order but MUST NOT
invent a missing probability measure or silently replace the joint profile with
independent marginals. The selection profile, environment profile, and bound
relation MUST be mutually coherent.

A uniform competitor is

```text
UniformSystem = (
  SystemId, SupportedProblemClass,
  DispatchOrStrategy,
  PersistentStateSemantics,
  UpdateSemantics,
  CompleteBehaviorAndCostSemantics
).
```

It includes validation, selection, dispatch, storage, migration, rebuilding,
switching, execution, failure, and retained-state behavior. Choosing unrelated
precomputed systems externally for each scenario is not uniform unless the
selector, every retained representation, and all switching/maintenance costs
belong to this system and obey the information rule.

Define

```text
WorkloadBehavior_W(R,a,c) = (
  PossibleMaximalTraceRelation,
  StochasticTraceLawOrKernel?,
  ProductivityAndStopEvidence,
  StateAndEffectProjection
)
  = exact nonempty complete joint behavior of R under X_W
    for (a,c) in BoundScenarioEnvironment_W

CompleteWorkloadBehavior_W(R)
  = exact indexed/measure/adversarial behavior object containing
    WorkloadBehavior_W(R,a,c) for every bound pair, aggregated with exactly
    ScenarioEnvironmentSelectionProfile_W and its declared correlations

WorkloadAdm_S(W)
  = { R in U_policy(Body(S),W,X_W) |
      for every (a,c) in BoundScenarioEnvironment_W,
        WorkloadAcceptanceOrConformance_W(
          MapBehavior(WorkloadCandidateObservation_W,
                      WorkloadBehavior_W(R,a,c)),
          MapBehavior(WorkloadTargetObservation_W,
                      WorkloadTargetBehavior_W(a,c)))
      and every trace in PossibleMaximalTraceRelation is well-typed, feasible,
        nonanticipating, effect-conforming, completion-conforming, and fully
        charged,
      and any stochastic observation/objective is computed from the exact
        normalized StochasticTraceLawOrKernel according to QuantifierPrefix_W,
        never by universal quantification over sampled paths }

WorkloadValue_W(R)
  = Objective_W(TraceCostFunctional_W(CompleteWorkloadBehavior_W(R)))

WorkloadArgmin_S(W)
  = { R in WorkloadAdm_S(W) |
      for every R' in WorkloadAdm_S(W),
      WorkloadValue_W(R) preceq_W WorkloadValue_W(R') }.
```

For a partial objective, `WorkloadFrontier_S(W)` is defined by the strict part
of `preceq_W`. `U_policy` has the same exact carrier, identity, membership, and
completeness obligations as §8.3. Changing the horizon, environment law,
scenario/environment selection profile, adversary, information pattern,
initial state, update rule, comparator class, or
functional creates a new `WorkloadId`.

`WorkloadBehavior_W` MUST be derived from the exact
`InitialDeploymentConfiguration`, joint transition semantics, environment
profile, action/randomness law, stop/productivity rule, and effect contract; it
cannot be supplied as an unrelated favorable behavior object. The two workload
observations are total and extensional, the target behavior is
nonempty on every `(a,c) in BoundScenarioEnvironment_W`, and the acceptance/conformance relation is
total on complete behavior objects. As with §5.1, allowed path membership alone
does not establish equality/refinement of a must-set or probability law.

The policy universe is not an informal substitution. For candidate context `C`
it is the exact independently constructed descriptor

```text
PolicyUniverseBody = (
  UniformPolicyCarrier,
  UniformPolicyIdentityAndEquality,
  PolicyMembershipSemantics,
  UniformPolicyGenerationGrammar,
  PolicyCompletenessProposition
)

PolicyUniverseId
  = Identity(
      policy-universe-domain,
      SnapshotId(C), WorkloadId_W, Identity(X_W), PolicyUniverseBody)

PolicyUniverse = (PolicyUniverseId, PolicyUniverseBody)

U_policy(Body(C),W,X_W)
  = { R in UniformPolicyCarrier | PolicyMembershipSemantics(R) }.
```

`PolicyCompletenessProposition` proves equality with every uniform policy
generated by the exact grammar/closure semantics under `W` and `X_W`; it is
noncircular and independent of discovery, optimization, or the returned
answer. The identity, equality, membership, proper-set/type, and coverage rules
of §8.3 apply verbatim to this displayed policy descriptor.

A use-case request MUST bind and equality-check this independently constructed
identity. A proper requested policy subuniverse is restricted exactly as in
§8.3 unless an omission bridge covers every excluded uniform policy.

The former reduction to values `(J_q(R))_(q in F)` is permitted only under a
separability theorem proving reset independence, no shared state or adaptation,
the required outcome independence/correlation law, and exact decomposability of
the whole functional.

### 9.5 Quantifier and comparison modes

Every use-case profile commits its exact quantifier prefix. The following are
different propositions and MUST NOT be interchanged:

```text
pointwise envelope:        for every z, there exists R_z
uniform pointwise system:  there exists R*, for every z, for every R, ...
distributional system:    argmin_R Expectation_mu[Cost(R,z)]
worst-case system:         argmin_R sup_z Cost(R,z)
online/adversarial:        argmin over nonanticipating R of sup_A Cost(R,A).
```

Expectation, `inf`, `sup`, minimization, selection, and input revelation are
generally noncommutative. Reordering them requires an exact minimax,
interchange, measurability, compactness, or other applicable theorem.

For a workload with no comparison claim, `ComparatorProfile_W` MUST be
`not-applicable`. Competitive, instance, and comparator-relative asymptotic
claims require the exact `comparator(ComparatorBody)` tag; an absolute asymptotic claim
requires `absolute-asymptotic(AbsoluteScoreProfile)` and states that no
comparator is used. For competitive, instance, or comparator-relative
asymptotic analysis, the workload MUST bind the exact profile

```text
ComparatorBody_W = (
  ComparatorClass_W,
  ComparatorIdentityAndEquality,
  ComparatorInformationBoundary,
  ComparatorBehaviorAndFeasibility,
  Compare_W,
  ComparisonScoreProfile,
  ExtendedValueCompletionAndExtremaPolicy
).

ComparisonScoreProfile_W =
    competitive(CompetitiveScoreSpecification)
  | instance(InstanceScoreSpecification)
  | relative-asymptotic(RelativeAsymptoticScoreProfile).

CompetitiveScoreSpecification = (
  AdversaryProjection, OfflineInfimumLaw,
  CompetitiveScoreLaw, ScoreCodomainAndOrder
).

InstanceScoreSpecification = (
  CompatibleScenarioFiberAggregation,
  AlphaBetaParameterSchema,
  InstanceComparisonLaw, ScoreCodomainAndOrder
).

RelativeAsymptoticScoreProfile = (
  ExactNonemptySizeOrIndexFamily,
  SizeMapAndDirectedOrder,
  StatisticAndAggregation,
  ComparatorRelativeScoreDefinition,
  Limit/Limsup/EventualOrderConvention,
  ConstantsAndThresholds,
  TotalScoreCodomainAndOrder,
  UnitsAndExtendedValuePolicy,
  ExtremaExistencePolicy,
  CheckedProjectionToQuantifierPrefixHorizonObjectiveAndComparisonRequest
).

AbsoluteScoreProfile_W = (
  ExactNonemptySizeOrIndexFamily,
  SizeMapAndDirectedOrder,
  StatisticAndAggregation,
  Limit/Limsup/EventualOrderConvention,
  ConstantsAndThresholds,
  TotalScoreCodomainAndOrder,
  UnitsAndExtendedValuePolicy,
  ExtremaExistencePolicy,
  CheckedProjectionToQuantifierPrefixHorizonAndObjective
).
```

Every field of `AbsoluteScoreProfile_W` is part of `WorkloadId`. Its checked
projection proves exact agreement with `QuantifierPrefix_W`, the stop/horizon
contract, and `ObjectiveAndOrder_W`; it cannot introduce a second ambient
asymptotic semantics.
The relative profile has the same identity and projection obligations and
additionally fixes how the system/comparator costs form its score. A comparison
request must equality-check the exact score-profile tag and every applicable
field; it cannot add an asymptotic convention absent from `ComparatorBody_W`.

`ComparatorClass_W` is an exact nonempty carrier. Each comparator has its own
explicit information boundary, which MAY be stronger than the online policy's
boundary (as for a clairvoyant offline comparator), but no unbound advice,
state, oracle, or future information is permitted. Every comparator uses the
same target behavior, initial-state identity, horizon, effect contract, and
accounting boundary unless an explicit comparison embedding proves a declared
difference harmless. `ComparatorBehaviorAndFeasibility` proves semantic
admission and total cost under that profile; an infeasible or semantically
incorrect zero-cost comparator cannot enter an exact bound.

For competitive analysis, the workload additionally binds

```text
AdversaryClass_W
  = the exact nonempty carrier of joint adversary objects projected from
    the adversarial ScenarioEnvironmentSelectionProfile_W and
    EnvironmentStrategiesOrLaw_W, each containing a legal strategy and an
    exact nonempty compatible scenario fiber wholly contained in
    BoundScenarioEnvironment_W

Cost_W(R,A)
  = the total extended-value cost obtained by applying
    TraceCostFunctional_W and QuantifierPrefix_W to R's complete behavior over
    exactly A's compatible scenario fiber

Compare_W
  = the deterministic total unit/extended-value comparison functional projected
    from ComparatorBody_W.
```

Every online policy and comparator has complete behavior and one total
extended-value cost under its bound information boundary. A competitive claim
is invalid unless `EnvironmentStrategiesOrLaw_W` has the adversarial tag and
supplies this carrier; a distribution-only workload cannot silently invent or
ambiently import an adversary class.
With those bindings,

```text
Offline_W(A)
  = inf { Cost_W(C,A) | C in ComparatorClass_W }

CompetitiveScore_W(R)
  = sup { Compare_W(Cost_W(R,A), Offline_W(A))
          | A in AdversaryClass_W }.
```

`ExtendedValueCompletionAndExtremaPolicy` MUST prove that every displayed
offline infimum and competitive supremum exists with the declared units and
zero/infinity conventions, or the comparison request is invalid/incomplete.
The optimizer may not replace a missing extremum with a favorable default.

A `competitive-bound` proves a bound on that score. A
`competitive-optimal` claim additionally proves that one admitted online system
attains the minimum score among all admitted online systems.

Because a scalar-member answer is membership in
`ApplicableWorkloadArgmin_S(u)`, every attained comparison-optimum class binds

```text
AttainedComparisonObjective_W = (ClaimScore, ClaimOrder, ObjectiveBridge)

ObjectiveBridge:
  phi is a total order isomorphism from WorkloadValue_W's codomain onto the
    ClaimScore codomain, and for every R in WorkloadAdm_S(W),
      ClaimScore(R) = phi(WorkloadValue_W(R)),
  so x preceq_W y iff phi(x) ClaimOrder phi(y).
```

For `competitive-optimal`, `ClaimScore=CompetitiveScore_W`. For
`asymptotic-optimal`, it is the exact bound asymptotic score/order after all
size-family, statistic, limit, and threshold quantifiers. Definitional equality
is the special case `phi=id`; otherwise the displayed total bijective,
order-preserving and order-reflecting isomorphism is required.
The scalar `asymptotic-optimal` member shape is legal only when this score has a
declared total lower-is-better order (after any required preorder quotient). A
partial asymptotic order uses workload Pareto member/frontier semantics or a
separately typed least-element proposition, not the scalar-member constructor.
Without it, a workload-value optimum cannot inhabit a competitive/asymptotic
optimal-member branch, even if the same realization happens to have a proved
comparison bound.

For instance comparison define, rather than silently dropping the environment
or randomness quantifiers,

```text
InstanceCost_W(R,a)
  = the total cost obtained by aggregating every context in a's exact
    nonempty BoundScenarioEnvironment_W fiber, every outcome,
    random choice, effect, and horizon contribution for scenario a in exactly
    QuantifierPrefix_W order.
```

If `a` itself carries the entire environment context, the profile MUST prove
that equivalence. An `instance-optimal(alpha,beta)` claim binds an exact
nonnegative comparison law and proves, for every valid scenario `a`,

```text
InstanceCost_W(R*,a)
  <= alpha * inf { InstanceCost_W(C,a) | C in ComparatorClass_W }
     + beta(a).
```

The arithmetic, units, extended values, and attainment/extraction status of the
offline infimum MUST be declared. `alpha=1`, `beta=0` is still only exact
comparator-relative instance optimality. It promotes to pointwise or full
workload global optimality only when a coverage/equality embedding relates
every applicable policy to the comparator semantics under the same information
scope, and the cost, aggregation, and order maps preserve the comparison; full
global scope additionally requires the complete policy universe. Attainment of
one narrow comparator infimum alone is insufficient.

An asymptotic claim binds the size/index map, directed input family,
worst/expected/other statistic, limit/limsup/eventual-order convention,
constants, thresholds, and uniform quantifier order. It does not imply an exact
finite-instance `global-optimal` claim.

Without a bound workload or functional, no single scalar meaning of “the
representation optimal for arbitrary operations” exists. UOR-GNAF instead
supplies the exact query envelope and the workload-policy optimization above.

---

## 10. Global optimality and the GNAF envelope

### 10.1 Scalar optimum

For a sealed snapshot `S`, query `q`, and machine `X`, first define the one
admission carrier used by every positive, negative, and incomplete result:

```text
ApplicableAdm_S(q,X) =
  Adm_S(q,X)                         when q binds no U_D
  Adm_S(q,X) intersect U_D(q)        when q binds U_D.

OptimizationScope_q =
  complete-system-universe           when no U_D is bound or U_D=U_sys is proved
  restricted(U_D(q))                 only for a proved proper inclusion.
```

The minimum set is then defined directly:

```text
Argmin_(S,M)(q,X)
  = { R in ApplicableAdm_S(q,X) |
      for every R' in ApplicableAdm_S(q,X), J_q(R) <= J_q(R') }.
```

These definitions range over the complete outer
`U_sys(Body(S),Problem_q,X)` exactly when `OptimizationScope_q` is complete.
If `q` binds a proper decision subuniverse, every formula in this section uses
the displayed intersection and defines only the restricted result unless the
branch-specific omission bridge required by §8.3 is proved. A member bridge
must cover every omitted improver or dominator; an identity-complete-set bridge
must also account for every omitted equal or nondominated identity; an
infeasibility or nonattainment bridge must prove the corresponding full-universe
negative proposition. One bridge kind MUST NOT be silently reused for another.

When `Argmin` is nonempty, all its members have the same objective value; define
that value as `OPT_(S,M)(q,X)`. To distinguish nonattainment from incomplete
evidence, a profile MAY additionally define

```text
Inf_(S,M)(q,X)
  = inf { J_q(R) | R in ApplicableAdm_S(q,X) }
```

in an explicitly bound order completion. If no greatest lower bound exists in
that completion, the profile has not defined `Inf` for the query.

If proved coverage establishes that `ApplicableAdm_S(q,X)` is empty, the result is
`infeasible`. If the set is nonempty and `Argmin` is proved empty, the result is
`unattained(scalar-no-minimum)`, whether or not the objective codomain contains
an infimum. When a declared infimum exists but is not attained, the more precise
reason is `unattained(scalar-infimum-not-attained, Inf)`. Failure merely to find
an admitted member or a minimum yields `optimization-incomplete` or
`unresolved`, not either theorem-level status. None of these cases has a
selected exact scalar optimum.

Membership of one `R*` in this set is a complete `global-optimal` member claim;
it does not imply that every equal-valued identity has been enumerated. When a
complete `Argmin` is proved and scalar `execute-selected` is requested,
deterministic extraction is

```text
Extract_(S,M)(q,X)
  = TiePolicy_q-least member of Argmin_(S,M)(q,X).
```

For every such complete-set extraction, the tie policy MUST be a total deterministic
selection function or an order proved to have one least member on that exact
`Argmin`. The tie policy selects only among already optimal candidates. It is
not evidence of optimality and MUST NOT select a higher-cost candidate.

### 10.2 Pareto frontier

For a partial objective order, define

```text
Frontier_(S,M)(q,X)
  = { R in ApplicableAdm_S(q,X) |
      no R' in ApplicableAdm_S(q,X) has J_q(R') prec_q J_q(R) }.
```

A `Pareto-optimal` certificate for one realization MUST cover every candidate
that could dominate it. A returned set `F` is `frontier-complete` exactly when

```text
every r in F is admitted and nondominated
and
every admitted nondominated r belongs to F.
```

This is exact identity-set equality with `Frontier_(S,M)(q,X)`. Equal cost
vectors do not erase distinct identities. Physical storage MAY reconstruct a
frontier identity through the committed operational basis, but the returned
mathematical frontier still contains it.

The stronger pruning property

```text
CofinalDominanceCover(K)
  iff for every r in ApplicableAdm_S(q,X) \ K,
      some k in K has J_q(k) prec_q J_q(r)
```

is separate. It may authorize a retention strategy but is neither necessary
for frontier completeness nor guaranteed to exist in an infinite partial
order.

A nonempty feasible universe need not have a Pareto member under an arbitrary
partial order. `unattained(partial-no-nondominated)` requires a proof that no
nondominated admitted member exists. Mere failure to prove or find a frontier yields
`optimization-incomplete`. Neither case permits a fabricated frontier.

Choosing one frontier member requires a separate preference policy. That policy
is not a tie-break and does not turn its selection into a scalar optimum. A
scalarization defines a new objective and MUST bind exact weights, units,
normalization, and order.

A `ParetoMember(R,proof)` answer needs only admission plus complete
no-dominator coverage for that `R`; it need not construct other nondominated
identities. Preference-based selection is defined only from a complete frontier.

### 10.3 The global non-adjacent space

For a sealed snapshot, define the total optimization answer

```text
OptimalityAnswer_S(q) =
    CertifiedMember(R, ActualClaimClass, ActualClaimPredicate,
                    exact scoped member proof)
  | ScalarOptimalMember(R, ActualClaimClass, ActualClaimPredicate,
                        membership proof)
  | ScalarSolution(complete Argmin, selected?,
                   ActualClaimClass, ActualClaimPredicate, set proof)
  | ParetoMember(R, ActualClaimClass, ActualClaimPredicate,
                 no-dominator proof)
  | PartialSolution(complete Frontier, selected?,
                    ActualClaimClass, ActualClaimPredicate, set proof)
  | CertifiedComparisonStatement(
      one-system-bound(kind,R,statement) |
      system-free-theorem(kind,statement),
      ActualClaimClass, ActualClaimPredicate, proof)
  | AuthorizedFallback(R, EvidenceScope,
                       ActualClaimClass, ActualClaimPredicate,
                       FallbackPolicyProof, outstanding obligations)
  | Infeasible(ActualClaimScope, PropositionId, proof)
  | NoAttainer(reason, ActualClaimScope, PropositionId, proof)
  | ComparisonRefuted(RequiredClaimPredicateId, ActualClaimScope,
                      exact negation proof)
  | Incomplete(RequestedClaimScope, outstanding obligations).

QueryPositiveAnswerShapeOf(a) =
    CertifiedMember                 when a is CertifiedMember(...)
  | ScalarMember                    when a is ScalarOptimalMember(...)
  | CompleteScalarArgmin            when a is ScalarSolution(...)
  | ParetoMember                    when a is ParetoMember(...)
  | CompletePartialFrontier         when a is PartialSolution(...)
  | ComparisonStatement.one-system-bound
      when a is CertifiedComparisonStatement(one-system-bound(...),...)
  | ComparisonStatement.system-free-theorem
      when a is CertifiedComparisonStatement(system-free-theorem(...),...).

ValidQueryAnswers_S(q)
  = { a in OptimalityAnswer_S(q) |
      QueryShapeCompatible(a,q),
      every nonfallback embedded admission, optimum, set-equality, bound, or
        negative proposition verifies over ApplicableAdm_S(q,X_q),
      any fallback system is exactly admitted while its weaker selection
        predicate verifies only over its committed evidence scope and the
        stronger applicable-universe obligations remain explicit,
      every nonfallback positive/member/complete-set/bound branch has a
        non-fallback-only base class and directly satisfies RequiredClaimClass
        and RequiredClaimPredicate,
      and only AuthorizedFallback may carry a FallbackOnlyBaseClaimClass; its
        exact ActualClaimClass, predicate, and evidence scope form a member of
        PermittedFallbackClaims and it carries optimization-incomplete status
        plus the stronger request's outstanding obligations }.

ExactActualQueryAnswers_S(q)
  = ValidQueryAnswers_S(q) minus Incomplete.

ExactRequestedQueryAnswers_S(q)
  = { a in ExactActualQueryAnswers_S(q) minus AuthorizedFallback |
      AnswerPredicateAndScopeSatisfyRequest(a,q) }.

Envelope_S(q)
  = the Argmin/Frontier projection of the two complete-set solution branches
    only.

AccumulationStateMap_S
  = (G |-> (Contrib_(S,G),
            AccumulatedSubject_(S,G),
            CanonicalAccumulation_(S,G)))
    for every committed AccumulationProfile G

OperationalBasisMap_S
  = (b |-> OperationalBasis_S(b))
    for every boundary class b covered by SealCertificate(S).

GNAFSpace_S = (
  Q_S,
  AccumulationStateMap_S,
  ProvenanceEnvelope_S,
  OperationalBasisMap_S,
  OptimalityAnswer_S
).
```

`QueryShapeCompatible(a,q)` holds for a positive branch exactly when
`QueryPositiveAnswerShapeOf(a)=RequestedAnswerShape(q)`; otherwise it is the
applicable exact infeasibility or
nonattainment branch, `ComparisonRefuted` only for a comparison shape, precise
`Incomplete`, or an explicitly authorized fallback. Exact negatives and
incomplete answers have no fabricated positive claim binding. Every negative
nevertheless binds the exact carrier scope and proposition it proves;
`AnswerPredicateAndScopeSatisfyRequest` accepts it for a complete-universe
request only under a direct full proof or the applicable branch-specific
omission bridge. A restricted negative may be an exact actual statement while
the stronger requested answer remains `Incomplete`.
For every negative constructor,
`ActualClaimScope = ClaimScopeProvedBy(PropositionId,proof)` is independently
verified; it is never inferred from the request or optimizer's desired label.

For each bound accumulation profile `G`, these map components are

```text
Contrib_(S,G) = ((OccurrenceId_i, T_i, a_i))_(i in I_S)
  where a_i in Q_(S,T_i)

AccumulatedSubject_(S,G)
  = fold mergeSem_G over
    (Lift_(G,T_i)(a_i))_(i in I_S)

CanonicalAccumulation_(S,G)
  = Normalize_G(AccumulatedSubject_(S,G)).
```

`OccurrenceId_i` is an exact typed identity, distinct from the identity of the
kind or content value. Declaration-set union deduplicates only the same
occurrence declaration; it MUST NOT merge distinct occurrence identities merely
because their values are equal. `Multiplicity_(G,T_i)` determines how each such
occurrence contributes.

The displayed fold requires a finite `I_S`. It is order-independent only when
the profile proves the commutative monoid laws; otherwise the exact semantic
sequence is typed input. An infinite or symbolic contribution family MUST
instead bind an exact family-merge operator, its directed-limit/completion and
convergence laws, and a coverage warrant proving the declared result. Without
those obligations, `AccumulatedSubject` is undefined for that family and the
full state cannot seal. The complete `Project_G` operation remains part of the
profile and maintains the canonical state under admitted additions.

The infinite-family profile MUST bind its index object, semantic order, finite
approximants, convergence notion, and uniqueness theorem. Moving
`mergeSem_G`, `Normalize_G`, a participating operation, a cost functional, or
an objective through the limit requires its own continuity/interchange
theorem. Conditional convergence preserves declared order as typed content;
commutativity of finite merge does not prove permutation or regrouping
invariance of an infinite family.

The name `GNAFSpace` is reserved for a snapshot satisfying the full
`gnaf-state(G,U)` capability: typed quotient, §10.7 accumulation projection and
kind lifts, provenance retention, operation closure, and proof-covered envelope.
A component implementing only semantic closure or query optimization uses its
component capability name and MUST NOT imply the full state invariant.

An `unattained`, `infeasible`, `optimization-incomplete`, or other weaker status
is retained by `OptimalityAnswer_S` but has no solution projection in
`Envelope_S`. The
space need not be eagerly materialized. A symbolic or demand-driven
implementation conforms only when every defined envelope value and status
equals this abstract definition and carries matching evidence.

`AuthorizedFallback` likewise has no envelope projection. It proves the
fallback realization exact and its stated weaker claim only over the exact
discovered, tested, or heuristic evidence carrier named by
`PermittedFallbackClaims`; it carries
the stronger claim's outstanding obligations and always reports
`OptimizationStatus = optimization-incomplete`. It never turns the discovered
set into `U_sys`/`U_policy` or a restricted universe unless that set has the
separate exact universe descriptor and coverage evidence.

The universal object is `GNAFSpace_S`, not one operation-independent physical
layout.
An exact query family binds

```text
QueryFamily_S = (
  QueryFamilyId, nonempty QueryCarrier,
  QueryIdentityAndEquality, MembershipSemantics,
  CoverageProposition,
  FamilySolverProfile, FamilyResourceEnvelope,
  FamilyUniverseScopeProfile
).
```

`FamilyUniverseScopeProfile` is exactly one of
`member-scoped(q |-> RequestedClaimScope_q)` or
`complete-universe(q |-> complete(SystemUniverseId_q))`. The first preserves
each member's declared restricted/complete scope; the second advertises full
outer-universe envelopes and therefore requires a branch-specific omission
bridge for every proper member restriction.

The solver profile binds the one terminating resolver/certificate constructor
and verifier; the resource envelope bounds it. If the family accepts raw input,
the profile also binds its input classifier and snapshot-parameterized query
constructor identities and coverage. The coverage proposition proves that membership equals the advertised family;
an empty family cannot establish a completeness claim. A
`query-family-answer-complete` capability permits arbitrary well-formed result
shapes and proves

```text
for every q in QueryFamily_S,
  the machine terminates within the family resource bound and returns
  an answer in ExactRequestedQueryAnswers_S(q).
```

A stronger **pointwise-envelope-complete** family additionally requires every
member to be seal-covered and to have `RequestedAnswerShape` equal
`CompleteScalarArgmin` or `CompletePartialFrontier`; member and comparison-bound
queries are invalid members of this stronger family. Its claim has
the form

```text
for every q in QueryFamily_S,
  OptimalityAnswer_S(q) is either
    the complete Argmin, with a selected member iff q requests selection,
    the complete nonempty Frontier, with a selected member iff q requests selection,
    exact proved infeasibility, or exact proved nonattainment,
  over ApplicableAdm_S(q,X_q), and is never Incomplete.
```

Under `member-scoped`, any proper `U_D` remains displayed as restricted. Under
`complete-universe`, every member requires either no proper subuniverse or its
branch-specific full-universe omission bridge. Both variants require exact
complete-set shape and `ExactRequestedQueryAnswers` at the profile's stated
scope; neither silently promotes a restricted member.

This permits different selected realizations for different queries and is not a
`family-optimal` claim about one uniform executable system. The pointwise
mapping is an analytic envelope and is not itself one executable realization.
A family claim MUST instead construct a workload `W_Q` whose valid scenarios
and event grammar cover exactly `QueryFamily_S`, and prove for one admitted
uniform system `R*` that

```text
R* in WorkloadAdm_S(W_Q)
and
WorkloadValue_(W_Q)(R*) preceq_(W_Q) WorkloadValue_(W_Q)(R)
  for every R in WorkloadAdm_S(W_Q),
```

with all storage, update, selection, switching, and execution costs and the
quantifier/information order required by §9.4. A separability theorem may then
justify the narrower `family-optimal` name.

### 10.4 Value-function realization

Define a value function extensionally before using a recurrence. For a scalar
query and configuration `s`, let

```text
PolicyAdm_q(s)
  = admitted complete policies whose allowed runs start at s and
    terminate/refuse exactly as required at Goal_q

V_q(s)
  = inf { AggregatedRunCost_q(policy,s) |
          policy in PolicyAdm_q(s) }
```

in the declared order completion, with argmin/frontier existence handled as in
§§10.1–10.2.

When cost composition is exact, the goal/stopping predicate is state-local,
every relevant fact is in the configuration, and the required fixed-point
theorem holds, an implementation MAY derive the Bellman equation

```text
V_q(s) = inf over admitted choices r of
  OutcomeAggregation_q(
      { edgeCost_M(s,r,omega,s'_omega)
        Compose_M V_q(s'_omega)
      | omega in OneStepOutcomes(OutcomeModel_X(q,r,s)) }),
```

with an additional stopping-cost candidate exactly when stopping is admitted.
For a vector objective, replace scalar minimization by pointwise cost
composition followed by `ParetoMin` and prove frontier existence/coverage.
`OneStepOutcomes` MUST be a nonempty exact decomposition of `OutcomeModel_X`;
using it requires a theorem that composing the local models yields the complete
maximal execution object. A sampled or Markov approximation is insufficient.

The outer `inf` MAY be written as `min`, and an optimizing choice or policy MAY
be extracted, only when the applicable attainment theorem is proved. Outcome
aggregation does not supply attainment of the outer choice infimum.

The choice of `r` occurs before the outcome unless `X` explicitly gives the
controller that outcome information. `Aggregate_(X,M,q)` MUST be the query's
declared expectation, worst-case, distributional, adversarial, or other exact
outcome aggregation; an expectation MUST bind an exact measure. For singleton
or explicitly controller-selected outcomes this reduces to the ordinary edge
recurrence.

A recurrence with improving cycles, zero-cost non-goal loops, unbounded descent,
omitted history, favorable-outcome selection, or incomplete reachability does
not establish the extensional value without a matching fixed-point theorem. If
family queries couple invocations through shared preparation, state, or
aggregation, the Bellman state MUST be the joint information state; pointwise
per-invocation minimization is invalid.

### 10.5 Lower bounds and attainment

A scalar lower-bound certificate `LB` is sound only when it binds the exact:

```text
(S, q, Problem_q, required observation,
 X, M, objective, U_sys(Body(S),Problem_q,X), U_D(q) if any, admission context,
 execution boundary, verifier revision)
```

and proves

```text
LB(S,q,X,M) <= J_q(R)
```

for every `R in ApplicableAdm_S(q,X)`.

If one `R* in ApplicableAdm_S(q,X)` satisfies

```text
J_q(R*) = LB(S,q,X,M),
```

then `R*` belongs to the request-scoped argmin. It is `global-optimal` only when
`OptimizationScope_q` is the complete system universe or a branch-specific
omission bridge proves the same member proposition over it. Complete
enumeration is unnecessary when the lower-bound theorem genuinely covers the
applicable carrier; the claim label still displays that carrier's scope.

A lower bound over one internal plan set, restricted primitive grammar, output
coordinate, cost component, tree language, no-sharing model, or no-preprocessing
model proves nothing beyond that scope.

### 10.6 Representation-to-execution bridge

Let `RepMetric` be a representation metric. The profile MUST bind the exact
representation subject `a_q` denoted by query `q` and a same-subject
representation `normal_q` proved minimum under `RepMetric`. For a §10.7
instantiation with `a_q in Sem_G` and `RepMetric = RepCost_G`, this is

```text
normal_q = Normalize_G(a_q).
```

To use that representation theorem as an execution lower bound, the profile
MUST bind a typed order-preserving bridge

```text
Bridge_(q,M) : RepMetricCodomain -> ObjectiveCodomain

LB_q = Bridge_(q,M)(RepMetric(normal_q))
```

and prove

```text
LB_q <= J_q(R)  for every R in ApplicableAdm_S(q,X)
```

together with a complete realization `R* in ApplicableAdm_S(q,X)` satisfying
`J_q(R*) = LB_q`. The bridge may be affine, nonlinear, or mediated by further
machine invariants; its type, units, monotonicity properties, and proof domain
MUST be explicit.

In the §10.7 instantiation, if the proof uses a representation witness `w_R`,
it MUST establish `w_R in Fiber_G(a_q)` for the same exact semantic subject,
not merely equality under a coarser observation. A common-codomain embedding
`phi` may then prove

```text
phi(RepMetric(normal_q))
  <= phi(RepMetric(w_R))
  <= J_q(R).
```

Minimum digit weight, term count, support, stored bytes, path length, or local
non-adjacency alone does not supply this bridge.

### 10.7 Canonical accumulation profile

A component profile MAY define a single exact accumulated semantic object and a
globally minimum representation of that object. Any implementation claiming the
full `gnaf-state(G,U)` capability or calling its maintained object a
`GNAFSpace` MUST supply this profile in addition to `core` and `state(U)`.
The profile binds:

```text
Raw_G, Sem_G
zeroRaw_G, mergeRaw_G
zeroSem_G, mergeSem_G
Eval_G : Raw_G -> Sem_G
AdmRep_G
RepCost_G with well-founded lower-is-better order
SelectCan_G selecting one member of each nonempty minimum set
Lift_G : (coproduct over participating T of Q_(S,T)) -> Sem_G
OccurrenceId_G and Contrib_(S,G)
Multiplicity_(G,T)
Safe_G(input, capacity, trace).
```

Write `Lift_(G,T)` for the restriction of `Lift_G` to component `T`. `Lift_G`
MUST respect every type equivalence and MUST be injective on the complete typed
coproduct, including across different kinds, unless an explicit cross-kind
quotient and required observation authorize each collision.
`Multiplicity_(G,T)` states whether repeated equal values represent duplicate
knowledge, distinct occurrences, counted multiplicity, or an idempotent event.
When occurrences matter, each occurrence is typed content and declaration-set
union MUST NOT silently deduplicate it.

If the profile claims insertion, partition, or reduction-order invariance, both
`(Raw_G, mergeRaw_G, zeroRaw_G)` and
`(Sem_G, mergeSem_G, zeroSem_G)` MUST be commutative monoids and `Eval_G` MUST
be a homomorphism:

```text
Eval_G(zeroRaw_G) = zeroSem_G
Eval_G(mergeRaw_G(x,y))
  = mergeSem_G(Eval_G(x), Eval_G(y)).
```

The semantic operations in this law are exact. A bounded physical realization
MUST prove that its range contains every intermediate value, or it MUST refuse
before committing the update. Overflow, underflow, wrapping, saturation,
rounding, truncation, and reduction-order dependence are nonconforming unless
they are themselves the explicitly declared semantic operation.

If order is semantic, the profile instead MUST encode order as typed content and
MUST NOT claim permutation invariance.

For `a in Sem_G`, define

```text
Fiber_G(a)
  = { r in Raw_G | AdmRep_G(r) and Eval_G(r) = a }

Best_G(a)
  = { r in Fiber_G(a) |
      for every t in Fiber_G(a),
      RepCost_G(r) <= RepCost_G(t) }

Normalize_G(a)
  = SelectCan_G(Best_G(a))

Project_G(r)
  = Normalize_G(Eval_G(r)).
```

Because `Normalize_G` and `Project_G` are total at their displayed types, a
profile instantiating them MUST prove, for every `a in Sem_G`, that `Fiber_G(a)`
is nonempty, `Best_G(a)` is nonempty, and the minimum is attained.
`SelectCan_G` MUST be total and deterministic on every such `Best_G(a)`, return
exactly one member, and be independent of discovery or scheduling order. If an
implementation supports only a subdomain, that exact subdomain MUST replace
`Sem_G` in the profile and MUST be closed under `zeroSem_G`, `mergeSem_G`, every
participating lift, and every participating operation; the total functions have
no implicit out-of-domain case. A well-order with a proved least member is one
permitted construction. A `representation-minimal` profile then MUST satisfy:

```text
Eval_G(Normalize_G(a)) = a                         soundness
Normalize_G(Eval_G(Normalize_G(a)))
  = Normalize_G(a)                                 idempotence
Project_G(r) = Project_G(t)
  iff Eval_G(r) = Eval_G(t)                        complete quotient
RepCost_G(Normalize_G(a)) <= RepCost_G(r)
  for every r in Fiber_G(a),                       minimum
```

where the complete-quotient equation quantifies only over admitted raw
representations.

Representational generalized adjacency in `G` is any admitted finite
same-semantic replacement `r -> t` with strictly lower `RepCost_G`, or equal cost
and an earlier canonical selection rank when the profile exposes such a rank.
`Project_G(r)` is non-adjacent under this relation because it is the selected
global minimum of the complete fiber. A local rewrite implementation MUST still
prove that its irreducibles coincide with `Project_G`; termination alone is
insufficient.

Define the canonical carrier and merge by

```text
CanRaw_G
  = { r in Raw_G | AdmRep_G(r) and Project_G(r) = r }
  = image(Project_G)

boxplus_G : CanRaw_G x CanRaw_G -> CanRaw_G

x boxplus_G y
  = Project_G(mergeRaw_G(x,y))
  = Normalize_G(mergeSem_G(Eval_G(x), Eval_G(y)))

unit_boxplus_G = Normalize_G(zeroSem_G).
```

The normalized merge MUST satisfy raw absorption:

```text
Project_G(mergeRaw_G(x,y))
  = Project_G(mergeRaw_G(Project_G(x),y))
  = Project_G(mergeRaw_G(x,Project_G(y)))
  = Project_G(mergeRaw_G(Project_G(x),Project_G(y))).
```

On `CanRaw_G`, this operation is associative, commutative, and unital with
`unit_boxplus_G` when the declared semantic merge is. For arbitrary raw `x`,
the corresponding unit law is normalization,

```text
Project_G(mergeRaw_G(x,unit_boxplus_G)) = Project_G(x),
```

not raw equality with `x`. Canonical merge is not generally idempotent. A
profile MUST NOT assume `x boxplus_G x = x` unless semantic merge itself is
proved idempotent.

Every operation participating in `gnaf-state(G,U)` MUST bind how its exact
target behavior updates `Sem_G` and MUST return each allowed semantic result
through `Normalize_G`. For a deterministic operation `f` this gives the lifted
operation

```text
f#_G(r_1,...,r_n)
  = Normalize_G(
      SemResult_f(Eval_G(r_1),...,Eval_G(r_n))).
```

It MUST satisfy normalization compatibility:

```text
f#_G(r_1,...,r_n)
  = f#_G(Project_G(r_1),...,Project_G(r_n)).
```

For relational, multi-output, stateful, or probabilistic operations, the same
law applies to the complete set/law of allowed canonical target states and
observations, not to one arbitrarily selected outcome. An operation changes a
`Sem_G` value in a runtime configuration; it does not mutate immutable
`GNAFSpace_S` and is not an equality rewrite. Persisting that value requires an
admitted occurrence update and a new snapshot.

Every bounded physical accumulation run MUST prove
`Safe_G(input,capacity,trace)` for every admitted partition, order, schedule, and
outcome covered by its claim, or refuse before commit. `Safe_G` MUST cover every
intermediate, not only the final value.

This profile gives the precise UOR-NAF-like representation layer for an
arbitrary admitted accumulation algebra. It still does not imply operation
execution optimality without §10.6.

### 10.8 Complete-system optimum

A complete-system global claim compares complete outer realizations beginning
from the same exact information and state. If a system performs search,
validation, proof checking, dispatch, fallback, state migration, or rebuilding,
those actions are included. They MAY be excluded only when the exact prepared
artifacts, plans, indexes, certificates, layouts, and their identities are bound
in the common initial `X` for every competitor and the comparison is explicitly
labeled `prepared-state` or `prepared-plan`. Otherwise candidate-specific
generation, verification, search, migration, and preparation remain charged;
merely describing an exclusion as symmetric is insufficient.

An implementation MUST NOT compare its selected internal plan cost with the
complete end-to-end cost of a competitor, or conversely.

### 10.9 Globally optimal arbitrary use-cases

For every well-formed use-case `W` admitted by a sealed snapshot, UOR-GNAF
defines the complete competitor set `WorkloadAdm_S(W)`, joint behavior space,
information rule, evaluation functional, objective order, and resulting
`WorkloadArgmin` or `WorkloadFrontier`. A scalar use-case result is globally
optimal exactly when it belongs to the attained `WorkloadArgmin`; a partial
result is exact exactly when it equals the complete workload frontier or is a
proved member as claimed.

Workload semantics and result selection are separated. A request is

```text
UseCaseRequest = (
  UseCaseRequestId, WorkloadId,
  PolicyUniverseId, RestrictedPolicyUniverseId?,
  ClaimRequest, ResultMode,
  TiePolicy?, PreferencePolicy?
).
```

It uses the result-mode and policy typing rules of §8.1. A scalar workload
member/argmin uses the scalar modes; a partial workload member/frontier uses the
partial modes; a competitive, instance, or asymptotic statement may use
`return-bound` or `execute-certified`. `execute-selected` requires the complete
workload argmin/frontier and its applicable total policy. Changing only result
selection creates a new `UseCaseRequestId`, not a new `WorkloadId`.
For a sealed `S`, well-formedness requires
`PolicyUniverseId_u = PolicyUniverseId(Body(S),W,X_W)`; any restricted policy
universe resolves the identity-bearing descriptor

```text
PolicyDecisionUniverseBody = (
  PolicyCarrierOrGrammar,
  PolicyIdentityAndEquality,
  PolicyMembershipSemantics,
  InclusionProofIntoU_policy,
  PolicyDecisionCoverageProposition
)

RestrictedPolicyUniverseId
  = Identity(policy-decision-universe-domain,
             UseCaseRequestBodyWithoutUseCaseRequestIdOrRestrictedPolicyUniverseId,
             PolicyUniverseId, PolicyDecisionUniverseBody)

U_PD(u) = { R in PolicyCarrierOrGrammar |
            PolicyMembershipSemantics(R) }.
```

It is resolved and equality-checked independently of discovery and the answer,
and its inclusion into `U_policy(Body(S),W,X_W)` is proved. The body in this
identity omits both outer `UseCaseRequestId` and the restricted
ID; after inserting the latter, `UseCaseRequestId` is computed by §4.1.
Define
`WorkloadOptimizationScope_u` as `complete-policy-universe` when no proper
restriction is bound, otherwise `restricted(U_PD(u))`; a branch-specific
omission bridge may promote only the proposition it actually proves.
Define one carrier for every result branch:

```text
ApplicableWorkloadAdm_S(u) =
  WorkloadAdm_S(W)                         when u binds no U_PD
  WorkloadAdm_S(W) intersect U_PD(u)       when u binds U_PD.

ApplicableWorkloadArgmin_S(u)
  = { R in ApplicableWorkloadAdm_S(u) |
      for every R' in ApplicableWorkloadAdm_S(u),
        WorkloadValue_W(R) preceq_W WorkloadValue_W(R') }.

ApplicableWorkloadFrontier_S(u)
  = { R in ApplicableWorkloadAdm_S(u) |
      no R' in ApplicableWorkloadAdm_S(u)
        has WorkloadValue_W(R') prec_W WorkloadValue_W(R) }.
```

Every positive, bound, negative, and incomplete use-case formula uses this same
carrier. When `U_PD` is proper, the answer retains its base member/set/Pareto/
bound claim class and carries `UniverseScope=restricted(U_PD(u))` unless a
branch-specific omission bridge proves the corresponding complete-policy
proposition. A member bridge does not prove
identity-complete argmin/frontier coverage; and a no-improver bridge does not
prove full-universe infeasibility or nonattainment.

The result carrier is total:

```text
UseCaseAnswerType_S(u) =
    WorkloadCertifiedMember(R, ActualClaimClass, ActualClaimPredicate,
                            exact scoped member proof)
  | WorkloadScalarOptimalMember(R, ActualClaimClass, ActualClaimPredicate,
                                membership proof)
  | WorkloadScalarSolution(complete Argmin, selected?,
                           ActualClaimClass, ActualClaimPredicate, set proof)
  | WorkloadParetoMember(R, ActualClaimClass, ActualClaimPredicate,
                         no-dominator proof)
  | WorkloadPartialSolution(complete Frontier, selected?,
                            ActualClaimClass, ActualClaimPredicate, set proof)
  | WorkloadCertifiedComparisonStatement(
      one-system-bound(kind,R,statement) |
      system-free-theorem(kind,statement),
      ActualClaimClass, ActualClaimPredicate, proof)
  | WorkloadAuthorizedFallback(R, EvidenceScope,
                               ActualClaimClass, ActualClaimPredicate,
                               FallbackPolicyProof, outstanding obligations)
  | WorkloadInfeasible(ActualClaimScope, PropositionId, proof)
  | WorkloadNoAttainer(reason, ActualClaimScope, PropositionId, proof)
  | WorkloadComparisonRefuted(RequiredClaimPredicateId, ActualClaimScope,
                              exact negation/no-witness proof)
  | WorkloadIncomplete(RequestedClaimScope, outstanding obligations).

WorkloadPositiveAnswerShapeOf(a) =
    CertifiedMember                 when a is WorkloadCertifiedMember(...)
  | ScalarMember                    when a is WorkloadScalarOptimalMember(...)
  | CompleteScalarArgmin            when a is WorkloadScalarSolution(...)
  | ParetoMember                    when a is WorkloadParetoMember(...)
  | CompletePartialFrontier         when a is WorkloadPartialSolution(...)
  | ComparisonStatement.one-system-bound
      when a is WorkloadCertifiedComparisonStatement(one-system-bound(...),...)
  | ComparisonStatement.system-free-theorem
      when a is WorkloadCertifiedComparisonStatement(system-free-theorem(...),...).
```

Because member and bound requests may have more than one correct witness, the
mathematical result is a validity set, not one assumed unique value:

```text
ValidAnswers_S(u)
  = { a in UseCaseAnswerType_S(u) |
      ShapeCompatible(a,u),
      every nonfallback admission/optimality/bound/negative proposition in a
        verifies over ApplicableWorkloadAdm_S(u),
      any fallback system is exactly workload-admitted, while its weaker
        selection predicate verifies only over its committed evidence scope and
        the stronger applicable-universe obligations remain explicit,
      every nonfallback positive/member/complete-set/bound branch has a
        non-fallback-only base class and directly satisfies RequiredClaimClass
        and RequiredClaimPredicate,
      and only WorkloadAuthorizedFallback may carry a FallbackOnlyBaseClaimClass;
        its exact ActualClaimClass, predicate, and evidence scope form a member
        of PermittedFallbackClaims and it carries optimization-incomplete status
        plus the stronger request's outstanding obligations }.

ExactActualClaimAnswers_S(u)
  = { a in ValidAnswers_S(u) | a is not WorkloadIncomplete }.

ExactRequestedAnswers_S(u)
  = { a in ExactActualClaimAnswers_S(u) |
      a is not WorkloadAuthorizedFallback,
      AnswerPredicateAndScopeSatisfyRequest(a,u) }.
```

`ShapeCompatible(a,u)` holds when a positive branch has
`WorkloadPositiveAnswerShapeOf(a)=RequestedAnswerShape(u)`; or when `a` is proved infeasible for any optimization
shape; or when scalar/partial no-attainment matches the requested objective
shape; or when a comparison request has an exact `WorkloadComparisonRefuted`
proof; or when `a` is an exactly authorized fallback; or when `a` is
`WorkloadIncomplete` with the precise obligations blocking that shape. Negative
and incomplete branches have no fabricated `ActualClaimClass`.
They still bind their exact `UniverseScope` and proposition identity.
`AnswerPredicateAndScopeSatisfyRequest` applies the complete/restricted scope
law of §8.1; a restricted emptiness, nonattainment, or comparison refutation is
not an exact requested complete-universe answer without its own omission bridge.

A complete-set branch has a unique mathematical identity set even if its
serialization is not yet assigned; a member branch may contain any correctly
certified member. “The answer is exact” means membership in this validity set,
not equality to an arbitrarily chosen witness.

where `W` is resolved exactly by `WorkloadId_u`. The negative branches mean

```text
WorkloadInfeasible
  iff ApplicableWorkloadAdm_S(u) is proved empty

WorkloadNoAttainer(workload-no-minimum, actual UniverseScope, PropositionId, proof)
  iff ApplicableWorkloadAdm_S(u) is proved nonempty
      and ApplicableWorkloadArgmin_S(u) is proved empty

WorkloadNoAttainer(workload-no-nondominated, actual UniverseScope, PropositionId, proof)
  iff ApplicableWorkloadAdm_S(u) is proved nonempty
      and ApplicableWorkloadFrontier_S(u) is proved empty.
```

Failure to establish one of these quantified propositions is
`WorkloadIncomplete`, not a proved negative.

An analytic pointwise envelope of independently selected systems is not a
workload solution. If the input or history selects an implementation at
runtime, that selector is part of the one admitted nonanticipating system and
its state, information, failure, and cost are inside `WorkloadBehavior_W`.

### 10.10 Use-case-class completeness

A use-case class descriptor binds

```text
UseCaseClass = (
  UseCaseClassId,
  DescriptorCarrier,
  WellFormedAndAdmissionPredicate,
  DescriptorIdentityAndEquality,
  UseCaseRequestConstructorProfile,
  CoverageProposition,
  ClassSolverProfile,
  SolverResourceEnvelope
).
```

`ClassSolverProfile` binds the one classifier, request-constructor binding,
uniform solver/extractor, transition semantics, result-certificate constructor,
and verifier identities used by a class capability. It and the resource
envelope are semantic fields of `UseCaseClassId`; a certificate cannot swap in
a different solver or larger bound.

For candidate context `C`, the stored constructor profile instantiates the
total function

```text
UseCaseRequestConstructor_(K,C) : W in K -> UseCaseRequest
```

and computes `PolicyUniverseId(Body(C),W,X_W)` rather than reading ambient
state. A sealed instance writes `UseCaseRequestConstructor_(K,S)` for
`C=SnapshotCandidate(S)`; its instantiated identity and outputs are committed
by the seal/class certificate, not `SnapshotBody`.

The descriptor carrier and, independently, its admitted subset

```text
AdmittedDescriptors_(K,S)
  = { W in DescriptorCarrier_K |
      WellFormedAndAdmissionPredicate_(K,S)(W) }
```

MUST each be exact and nonempty. `CoverageProposition` proves that this admitted
subset, not merely the outer carrier, equals the advertised class. All displayed
`W in K` quantifiers below mean `W in AdmittedDescriptors_(K,S)`; a nonempty
carrier with no admitted descriptor cannot satisfy a class capability. A
`use-case-class-answer-complete(K)` capability proves that the machine
terminates within its bound for every admitted `W in K`, constructs
`u = UseCaseRequestConstructor_(K,S)(W)`, and returns a member of
`ExactRequestedAnswers_S(u)`, including a requested exact comparison bound where
applicable. It never returns
`optimization-incomplete`, `unresolved`, or an unauthorized weaker execution
for a valid admitted member. Invalid raw inputs may still return the exact
classification status of §8.7.

A `use-case-class-complete(K)` capability is stronger: its request constructor
MUST request only complete-universe scalar/Pareto optimal-member or
identity-complete argmin/frontier claims (including attained
competitive/asymptotic optima), never a mere comparison bound, proper
restricted-universe result, `best-known`, measured, or heuristic claim. It
returns a member of `ExactRequestedAnswers_S(u)` or an exact infeasible/unattained
branch for every class member.

This is the strongest conforming meaning of global optimality for arbitrary
inputs/use-cases:

```text
for every W in K,
  let u = UseCaseRequestConstructor_(K,S)(W);
  the returned answer belongs to ExactRequestedAnswers_S(u),
  and every selected system is compared with all complete competitors
  under W's exact quantifier prefix and information boundary.
```

When an `InputTotalCapabilityBinding` couples an input profile `I` to this
`use-case-class-complete(K)` capability, the quantifier begins at the exact raw
input boundary:

```text
for every x in RawInput_I,
  Classify_I(x) = valid(scenario)
    implies AdmittedUseCaseOf_(I,K,S)(scenario) = (W,u,scenario),
            W in AdmittedDescriptors_(K,S),
            and the returned answer is the complete-universe exact answer
            required above;
  Classify_I(x) = invalid | unresolved | unsupported
    implies only that exact nonoptimization request status is returned.
```

Thus arbitrary-input globality is a proved conjunction of input totality and
class-global capability, not a promotion of an instance certificate or an
assumption that every byte string denotes a valid optimization problem.

The request constructor MUST be deterministic, total, and type-correct for
every admitted class member. The class certificate binds one uniform solver,
the input/classifier and request-constructor identities, transition semantics,
result-certificate constructor, proof verifier, and resource bound, and proves
termination and exact output for every member. A per-instance collection of
unbound existence statements is insufficient.

The class may be extensible by the admission protocol, but every certificate is
closed over the exact `UseCaseClassId` in its sealed snapshot. Adding a new
descriptor creates a new snapshot/class identity and requires class-wide
recertification or a transition theorem. This quantified capability is not an
omniscient claim over undeclared semantics or arbitrary programs outside the
class.

---

## 11. Normative theorems

The theorems in this section are construction-independent. Their hypotheses are
mandatory whenever a conformance claim invokes the conclusion.

### Theorem 11.1 — Lower-bound attainment

Let `A = ApplicableAdm_S(q,X)`. Suppose `R* in A`, and suppose `LB <= J_q(R)` for every
`R in A`, with `J_q(R*) = LB`. Then `R* in Argmin_(S,M)(q,X)`.

**Proof.** For every admitted `R`,

```text
J_q(R*) = LB <= J_q(R).
```

Thus no realization in the exact request carrier has lower objective value than
`R*`; since `R*` is in that carrier, it attains its minimum. The conclusion is
full-universe global only under the scope rule of §10.1. QED.

### Theorem 11.2 — Extensional factorization

Let `~` be an equivalence relation on `V`. If `F : V -> W` is extensional,
meaning `x ~ y` implies `F(x) = F(y)`, then there is a unique function

```text
F_bar : V / ~ -> W
```

such that `F_bar([x]) = F(x)`.

**Proof.** Define `F_bar([x]) = F(x)`. Extensionality makes the definition
independent of the representative. Any factorization satisfying the equation
has the same value on every quotient class, so it is unique. QED.

This theorem is the semantic basis for discarding alternate spellings from the
canonical quotient. It does not authorize discarding operational realizations
with different future resource behavior.

### Theorem 11.3 — Closure merge law

Let `Cl` be an extensive, monotone, and idempotent closure operator on one
carrier—instantiated by `ClBar_U` on judgment sets in §7.2. Then

```text
Cl(A union B) = Cl(Cl(A) union Cl(B)).
```

**Proof.** By extensivity, `A union B` is contained in
`Cl(A) union Cl(B)`; monotonicity gives the forward inclusion after applying
`Cl`. Conversely, monotonicity gives both `Cl(A)` and `Cl(B)` contained in
`Cl(A union B)`. Apply `Cl` to their union and use idempotence for the reverse
inclusion. QED.

### Theorem 11.4 — Candidate-extension monotonicity

Suppose an extension preserves the semantic request, semantics, machine,
resources, cost, aggregation, objective, and either the absence of a decision
restriction or an exact transported decision-universe embedding. Let `q` and
`q'` be the transported queries for `S` and `S'`: they agree on those preserved
fields but carry their respective query, system-universe, and any restricted-
universe identities. Suppose

```text
ApplicableAdm_S(q,X) subseteq ApplicableAdm_S'(q',X).
```

Then, for a lower-is-better scalar objective with attained minima,

```text
OPT_S'(q',X) <= OPT_S(q,X).
```

**Proof.** Every old admitted realization remains a candidate in the enlarged
set. A minimum over a superset cannot exceed the minimum over its subset. QED.

### Theorem 11.5 — Replay-free extension under absorption

Let `(iota_GR,iota_GS)` be the accumulation embeddings of §7.8. Suppose

```text
Project_G'(
  mergeRaw_G'(iota_GR(x), y))
=
Project_G'(
  mergeRaw_G'(iota_GR(Project_G(x)), y))
```

for every old contribution `x` and admissible new contribution `y`. Then the old
projected state `Project_G(x)` is a sufficient input for computing the new projected state
after adding `y`; replay of the raw spelling of `x` is unnecessary.

**Proof.** The right-hand side uses only `Project_G(x)` and `y`, and equals by
hypothesis the normalization of the complete old contribution with `y`. QED.

The theorem says nothing about old operational derivations unless a separate
extension-sufficiency or contextual-dominance warrant covers them.

### Theorem 11.6 — Safe contextual pruning

Let `p' <=^ctx_S p`, and suppose `p'` remains retained. Removing `p` from every
candidate set covered by that dominance certificate preserves the minimum
objective value. If `p'` strictly dominates `p` under the partial order in every
covered context, removing `p` also preserves the Pareto frontier's cost image.

**Proof.** In each covered context where `p` is admissible, the definition of
contextual dominance supplies an admitted `p'` with query-equivalent behavior
and no higher cost. Thus `p` cannot be the sole witness of a lower scalar value. Under the
stated strict condition it cannot be a nondominated frontier member. QED.

This theorem does not preserve an identity-complete argmin/frontier or
identity-sensitive tie selection unless the identity is reconstructible, the
policy quotient merges it, or a separate policy warrant permits its removal.

### Theorem 11.7 — Global optimum implies GNAF normality

If `R*` is globally scalar-optimal for `q`, then it contains no occurrence with
a strictly improving replacement under §6.3.

**Proof.** Such a replacement would produce an admitted same-observation
realization with strictly lower objective, contradicting global optimality. QED.

The converse is false without complete comparison, proof-complete reduction, or
lower-bound attainment; see fixture `GNAF-VEC-04`.

### Theorem 11.8 — Pareto member implies partial-order GNAF normality

If `R*` is Pareto-optimal for `q`, it contains no occurrence whose admitted
same-behavior replacement strictly dominates it under `prec_q`.

**Proof.** Such a replacement would produce an admitted realization whose cost
vector strictly dominates `R*`, contradicting membership in the Pareto frontier.
QED.

As in the scalar case, partial-order normality alone does not prove Pareto
membership or frontier completeness.

### Theorem 11.9 — Workload lower-bound attainment

Let `A = WorkloadAdm_S(W)`. Suppose `R* in A`,

```text
LB_W preceq_W WorkloadValue_W(R)
```

for every `R in A`, and

```text
WorkloadValue_W(R*) = LB_W.
```

Then `R* in WorkloadArgmin_S(W)`.

**Proof.** The equality identifies the value of `R*` with a lower bound applying
to every complete admitted uniform competitor. Therefore `R*` is no worse than
every member of `A` and, being admitted, attains the minimum. QED.

The certificate for this theorem MUST bind the complete workload protocol,
competitor class, quantifier prefix, information filtration, environment
law/adversary, comparator, horizon, functional, objective, and complete-system
boundary. A bound over pointwise queries, an offline selector, or a proper
competitor subset does not satisfy the premise.

---

## 12. Evidence, certificates, claims, and outcomes

### 12.1 Separate evidence propositions

UOR-GNAF recognizes at least these distinct warrant types:

- kind and parameter validity;
- typed content membership;
- semantic equivalence or refinement;
- canonical decoding and reconstruction;
- operation exactness and extensionality;
- complete behavior/outcome-model conformance;
- invocation eligibility;
- machine and resource feasibility;
- capacity and intermediate-safety coverage;
- cost derivation and accounting coverage;
- closure and candidate-universe completeness;
- contextual dominance and extension sufficiency;
- lower-bound, no-dominator, and optimality attainment;
- snapshot transition preservation.
- input-domain classification and use-case/workload admission;
- refinement-diagram coherence and completion/density;
- operator domain, graph closedness, closability, closure, adjoint, symmetry,
  self-adjointness, and essential self-adjointness;
- resolvent, each spectrum class, limit transfer, no-loss/no-pollution, and
  zero correspondence.

A certificate field MUST NOT be omitted merely because another field uses the
same address or proof artifact.

### 12.2 Evidence classes

Evidence classes are:

- **proved** — a theorem or sound certificate whose hypotheses match exactly;
- **realized** — evidence that a concrete implementation implements a declared
  construction;
- **measured** — empirical evidence under a fully bound machine and workload;
- **hypothesized** — an unproved and unmeasured proposition.

Measured evidence MAY calibrate a cost model or rank the exact measured set. It
cannot enlarge a correctness, universe-completeness, lower-bound, or global
optimality theorem.

### 12.3 Evidence gates

An exact globally selected realization passes these gates in order:

1. **Specified** — kinds, operation, problem, invocation scope, observation,
   state, failure, universe, machine, resources, costs, workload, quantifier
   prefix, and objective are complete.
2. **Classified** — every input in the claimed input domain has the exact
   valid/invalid/unresolved/unsupported classification and every valid case maps
   to a typed invocation or scenario.
3. **Resolved** — every consumed descriptor, value, fact, rule, and warrant is
   strictly decoded and bound to its typed subject and snapshot.
4. **Warranted** — membership, equivalence, eligibility, complete-behavior
   exactness, composition,
   normalization, accumulator, intermediate-safety, and reconstruction
   obligations verify.
5. **Realized** — the implementation matches the declared construction under
   malformed, extremal, adversarial, differential, schedule, and retained-state
   tests. Realization evidence does not replace universal proof.
6. **Accounted** — every in-boundary action is charged or explicitly free and
   every compared candidate has one total objective value.
7. **Covered** — exhaustive enumeration, proof-complete reduction, lower-bound,
   or no-dominator evidence covers the complete relevant universe.
8. **Selected** — the exact scalar, Pareto, workload, or authorized weaker policy is
   applied after preserving the corresponding mathematical set.
9. **Revision-bound** — all semantics, evidence, universe, machine, cost,
   objective, policy, and verifier revisions are fixed.
10. **Receipted** — the execution receipt binds the actual run to the claim.

Failure at a gate produces rejection, `unknown`, or an already admitted
fallback. It MUST NOT silently reinterpret content or weaken the required
observation.

### 12.4 Claim classes

Permitted claim classes are:

- `exact` — required observation is satisfied; no optimality is asserted;
- `normal-form` — irreducible under the exact declared reductions; uniqueness
  and minimality are not asserted;
- `canonical` — normalization termination, evaluation soundness, normal-image
  membership, idempotence, and complete-quotient uniqueness are proved for the
  declared semantic universe;
- `representation-minimal` — an admitted representation attains the declared
  global representation minimum;
- `comparison-theorem` — the exact precommitted system-free or universal
  comparison proposition is proved; no one-system optimum or executable
  witness is implied;
- `profile-defined-comparison(ProfileId,ClassId,ComparisonResultShape)` — an
  admitted extension class registered under §8.1; it is permitted only with the
  same carrier kind, candidate quantifier, result shape/cardinality,
  certificate, and execution rules as its registered shape;
- `input-total` — every input in a bound raw/typed domain is exactly classified;
- `global-optimal` — a complete realization belongs to the scalar argmin over
  the complete bound universe;
- `argmin-complete` — the returned identity set equals the full admitted scalar
  argmin;
- `Pareto-optimal` — one realization is proved nondominated;
- `frontier-complete` — the returned identity set equals the full admitted
  Pareto frontier;
- `pointwise-envelope-complete` — every query in a bound class has its exact
  solution set or exact proved-negative answer, with no incomplete member and
  without asserting one executable system;
- `query-family-answer-complete` — every query in a bound nonempty family
  receives an exact requested-shape answer or exact negative result, with no
  incomplete member, but member/bound shapes need not materialize an envelope;
- `use-case-global-optimal` — one uniform system belongs to the exact
  `WorkloadArgmin` for a bound `W`;
- `workload-argmin-complete` — the returned identity set equals the complete
  `WorkloadArgmin` for a bound `W`;
- `workload-Pareto-optimal` — one uniform policy is proved nondominated under
  the exact workload partial order over the applicable policy universe;
- `workload-frontier-complete` — the returned identity set equals the complete
  `WorkloadFrontier` for a bound `W`;
- `family-optimal` — the separable-family specialization of
  `use-case-global-optimal` under a proved separability theorem;
- `instance-optimal(alpha,beta)` — one uniform system satisfies the exact
  per-scenario comparator inequality of §9.5;
- `competitive-bound` — one online system has a certified competitive score
  bound;
- `competitive-optimal` — one admitted online system attains the minimum
  competitive score;
- `asymptotic-bound` — one exact precommitted asymptotic comparison predicate
  is proved, without asserting attainment of an optimum;
- `asymptotic-optimal` — one system is optimal under the exact asymptotic order
  and quantifier convention bound by the profile;
- `use-case-class-complete` — every admitted member of a nonempty bound use-case
  class receives a certified solution or exact negative result within the
  declared solver bound;
- `use-case-class-answer-complete` — every admitted member receives the exact
  requested member, complete set, comparison bound, fallback-free negative, or
  other `ExactRequestedAnswers` branch, but the class may request bounds rather than
  optima and therefore does not carry the stronger global class meaning;
- `maintained-use-case-class` — a sealed snapshot and every claimed successor
  satisfy `use-case-class-complete` for the same transported class or a bound
  transition theorem;
- `restricted-universe-optimal` — legacy input alias for base class
  `global-optimal` with `UniverseScope=restricted(id)`; other restricted result
  shapes retain their own base class and the same orthogonal scope; conforming
  answers/certificates emit only the normalized pair, never this alias as a base;
- `revision-preserved` — a prior exact claim has a matching transition warrant;
- `best-known` — no discovered admitted candidate is known to improve it, but
  comparison coverage is incomplete;
- `measured-best-among-tested` — best exact measurement in a named tested set;
- `heuristic-selected` — selected without an exact optimum/no-dominator proof.

`universal` describes the admission protocol and is not an optimality claim.
`universal-global-optimal`, `optimal for all future additions`, and unscoped
`optimal for arbitrary operations` are nonconforming labels.

Only proved evidence can establish `canonical`, `representation-minimal`,
`input-total`, `global-optimal`, `argmin-complete`, `Pareto-optimal`,
`frontier-complete`, either query-family class, either workload-complete class, either use-case-class
complete class, any
workload/instance/competitive/asymptotic optimum, either use-case-class claim,
or `revision-preserved`.

### 12.5 Exact result statuses

A result has total, separate request, runtime, and optimization status
projections. A nonexecuting analytic, negative, incomplete, or request-status
answer has `RuntimeStatus=not-run`; a request not asking optimization has
`OptimizationStatus=not-requested`. Positive/member/set/bound/fallback answers
carry exactly one strongest honest `ActualClaimClass`. Exact negatives,
incomplete answers, and invalid/unresolved/unsupported requests carry no
fabricated positive claim class; their exact proposition/status or obligation
is the result. Thus “claim class” below is optional precisely on those branches:

```text
RequestStatus =
  valid | invalid | unresolved | unsupported |
  unadmitted | incoherent | unsealed

RuntimeStatus =
  not-run | succeeded | permitted-refusal(FailureTag) |
  permitted-failure(FailureTag) | productive |
  cancelled | resource-exhausted | execution-conflict |
  aborted-before-commit | unresolved-runtime |
  implementation-violation(ViolationTag)

OptimizationStatus =
  not-requested | certified | infeasible |
  unattained(reason) |
  comparison-refuted(RequiredClaimPredicateId) |
  optimization-incomplete
```

`infeasible`, `unattained`, and `comparison-refuted` are exact proved negative results as defined in
§10; absence of evidence yields `optimization-incomplete`. Statuses and the
strongest justified claim class MUST be reported independently. If a query
requests `global-optimal` but lacks global coverage, execution is prohibited
unless `ClaimRequest_q` explicitly authorizes a weaker class and fallback
policy. An authorized fallback reports `RuntimeStatus = succeeded` only after
the run succeeds, `OptimizationStatus = optimization-incomplete`, and the
actual weaker claim class. It MUST NOT report the requested global claim as
accepted or its discovered set as complete.

An exact negative result MUST carry a status certificate binding the same
`SnapshotId`, `SealId`; query or workload/use-case request; corresponding
`U_sys` or `U_policy`; machine, cost, quantifier prefix, and coverage context as
§12.6, together with the emptiness or nonattainment proof. It is subject to the
same seal-coverage-basis rule. `optimization-incomplete` reports the precise
undischarged obligation and MUST NOT masquerade as a proof of infeasibility or
nonattainment.

`comparison-refuted(P)` requires a closed proof of the exact negation or
no-witness form fixed by `RequiredClaimPredicate=P`; it is not inferred merely
because a bound search failed. It is compatible only with a comparison-answer
shape and carries no selected system or positive `ActualClaimClass`.

`unattained(reason)` uses at least

```text
scalar-no-minimum
scalar-infimum-not-attained(Inf)
partial-no-nondominated
workload-no-minimum
workload-no-nondominated.
```

The reason and its exact quantified proof are certificate fields. Runtime
resource exhaustion is not `unattained`; it normally yields
`optimization-incomplete` plus the exact runtime status.

If an actual completed behavior fails its certified execution-realization or
query/workload acceptance check, the runtime status is
`implementation-violation`. Every already visible effect and successor-ledger
fact MUST be retained and receipted, the run carries no exact actual claim
class, and the affected deployment/realization-machine binding MUST be
quarantined until an explicit investigation, replacement, or recertification
transition succeeds. The observation does not mutate the old immutable
certificate or by itself prove its mathematical proposition false; it proves
that this execution cannot use that certificate as its conformance warrant.

### 12.6 Certificate binding

Certificate bodies are a dependent tagged sum, not one flat tuple that forces a
class certificate to pretend it has one query, machine, or answer:

```text
CommonCertificateBinding = (
  CertificateSchemaRevision,
  SnapshotId, SealId, ParentTransitionIds,
  Accumulation/Contribution/Multiplicity/Subject/CanonicalRoots,
  OperationalBasisAndDependencyRoots,
  Semantic/ExtensionRevisions,
  AnalyticProfilesAndPropertySpecificWarrantsActuallyUsed,
  CertificateHypotheses,
  VerifierFoundation/Implementation/Configuration/Revision
)

ScopeBinding =
    QueryBinding(
      q, Problem_q, InvocationScope_q, OperationProfile,
      CorrectnessAndEvaluationProfiles, observations, Accept_q,
      CompletionAndCompleteBehaviorConformance,
      X_q, M_q, aggregations/objective/order/accounting/effects/concurrency,
      U_sys, U_D?, ApplicableAdm, OptimizationScope,
      ClaimRequest, ResultMode, selection policy?,
      answer in ValidQueryAnswers_S(q), selected complete system?,
      exact branch proposition and all coverage/lower-bound/no-dominator proofs)

  | WorkloadBinding(
      u, W, Scenario/Environment/BoundRelation/SelectionProfiles,
      InformationFiltration,
      InitialDeploymentAndJointBehavior, completion/effects,
      X_W, M_W, QuantifierPrefix_W, objective/order/comparator/accounting,
      U_policy, U_PD?, ApplicableWorkloadAdm, WorkloadOptimizationScope,
      ClaimRequest, ResultMode, selection policy?,
      answer in ValidAnswers_S(u), selected uniform system?,
      exact branch proposition and all workload/coverage/bound proofs)

  | QueryFamilyBinding(
      exact nonempty QueryFamily_S and its equality/membership/coverage,
      CapabilityClass = query-family-answer-complete |
                        pointwise-envelope-complete,
      exact FamilySolverProfile, FamilyResourceEnvelope,
        and FamilyUniverseScopeProfile,
      AdmittedQueryOf_(I,F,S)? when raw-input totality is claimed,
      for every q in the carrier: QueryBinding(q) and an answer in
        ExactRequestedQueryAnswers_S(q) whose scope satisfies
        FamilyUniverseScopeProfile,
      for pointwise-envelope-complete: complete-set shape and exact advertised-
        scope obligations; full bridges only for complete-universe profile,
      quantified coverage proof and one uniform-solver termination/resource theorem)

  | InputTotalBinding(
      exact nonempty raw/typed domain,
      classifier identity and four-way disjoint-cover/termination proof,
      initialization and, when combined with a family/class claim,
      AdmittedQueryOf_(I,F,S) or AdmittedUseCaseOf_(I,K,S)
        with its totality/coverage proof)

  | InputTotalCapabilityBinding(
      exact InputTotalBinding I,
      CapabilityBinding = QueryFamilyBinding F |
                          UseCaseClassBinding K,
      exact CertificationRequestId and accepted
        UnderlyingCapabilityCertificateId whose standard §12.6 body projection
        is CapabilityBinding,
      exact shared SnapshotId/SealId/implementation/verifier/resource identities,
      for every raw input: the bound classifier terminates and returns exactly
        one of valid/invalid/unresolved/unsupported;
      on valid, the exact admitted-query/use-case constructor terminates and
        yields a member of F/K and the *same* uniform capability solver returns
        the exact answer/certificate promised by CapabilityBinding;
      on invalid/unresolved/unsupported, the exact nonoptimization status and
        diagnostics are returned and no optimality claim is made,
      combined resource envelope and one quantified termination/coverage/
        compatibility proof over classifier, constructor, solver, certificate
        constructor, and verifier)

  | UseCaseClassBinding(
      K, exact nonempty AdmittedDescriptors_(K,S), coverage proposition,
      CapabilityClass = use-case-class-answer-complete |
                        use-case-class-complete,
      instantiated UseCaseRequestConstructor_(K,S),
      exact ClassSolverProfile with its one uniform classifier/solver/
        certificate-constructor/verifier identity,
      SolverResourceEnvelope,
      for every admitted W: u_W, its dependent WorkloadBinding, and proof that
        the returned answer belongs to ExactRequestedAnswers_S(u_W),
      for use-case-class-complete: full-scope optimum/Pareto constructor
        restrictions and proof that no bound/fallback/weaker mode is generated,
      one termination-and-correctness proof quantified over the whole class)

  | ArtifactBinding(
      ArtifactClaimClass, ArtifactClaimScope,
      exact typed subject/representation carrier and identity/equality,
      normalization or representation objective/order when applicable,
      exact claim proposition and complete proof/coverage basis)

  | MaintainedTransitionBinding(
      predecessor and successor snapshot/seal identities,
      transported family/class descriptor, exact transition grammar,
      dependency impact, recertification or preservation proof,
      and the quantified successor capability statement).

CertificateBody = (CommonCertificateBinding, ScopeBinding).
```

The proof material embedded by `InputTotalCapabilityBinding` is its acyclic
`InputTotalCapabilityProofCore`: exact theorem/proposition identities and their
evidence object, excluding `InputTotalCapabilityStatementBody`,
`RequiredInputTotalCapabilityStatementId`, every input-total certificate
body/ID, and the final verifier result. The standard `CertificateBody` is built
from this closed scope binding first; only then may the custom conjunction
statement bind that body and its derived standard certificate ID. This ordering
forbids either the proof or binding from containing the statement that later
commits it.

Only fields in the selected branch are required, but every premise used by its
statement is mandatory. A heterogeneous class branch carries a dependent
per-workload binding; it MUST NOT substitute one ambient `X`, `M`, quantifier
prefix, universe, query, or answer for all members. Every positive, bound, or
fallback answer binds its `ActualClaimClass` and exact predicate. Exact
negative/incomplete branches bind only their exact status proposition and scope;
a member or bound is never promoted to a complete-set claim.

`InputTotalBinding` alone proves classification/constructor totality but no
optimization capability. `QueryFamilyBinding` or `UseCaseClassBinding` alone
proves its admitted descriptor carrier but no claim about arbitrary raw inputs.
Only `InputTotalCapabilityBinding` proves their conjunction. Its valid-input
projection inherits the capability branch's exact global/restricted scope; the
other three classifier outcomes are total request statuses, not weak optimum
claims. Two unrelated certificates cannot be silently conjoined without the
displayed shared-identity and compatibility proof.

For `AuthorizedFallback`, the certificate separately binds
`RequestedClaimOutstandingObligations` for the unproved stronger request and
the proof obligations of the actual weaker fallback claim. The former are
diagnostic scope and are not premises of the weaker proof. The fallback's own
`CertificateHypotheses.UndischargedObligations` MUST be empty before its exact
weaker `ActualClaimClass` can verify; the overall `OptimizationStatus` remains
`optimization-incomplete` because the requested stronger predicate is open.

The displayed dependent sum forms `CertificateBody`, and
`CertificateId = Identity(result-certificate-domain, CertificateBody)`.

```text
RequiredCertificateStatementId(cert,a)
  = Identity(certificate-statement-domain,
             ExactProposition(cert.ScopeBinding,a)).
```

`ExactProposition` is the branch's fully instantiated typed proposition,
including quantifiers, carrier/universe scope, objective/order, negative or
positive result shape, and all identity-set equality claimed. It excludes proof
bytes and the statement identifier itself. The verifier MUST accept this exact
identity, not a syntactically related lemma.

`CertificateId` is not a self-included input. The body binds `SnapshotId` and
the already established `SealId`. It MUST be derivable from the query class,
coverage basis, and verifier revisions committed by that seal. If it is not,
the same `SnapshotCandidate` requires a new `SealCertificate` and `SealId`
before the result certificate can carry the stronger claim. The result
certificate never mutates the sealed snapshot it cites.

The optimizer's own assertion is not evidence for its result. An independent
verifier MUST check exact enumeration, proof-complete reduction, lower-bound
attainment, or the applicable no-dominator theorem.

### 12.7 Hypothesis closure and verifier results

Every certificate binds an explicit proof foundation and a closed hypothesis
set:

```text
CertificateHypotheses = (
  FoundationId,
  LogicAndKernelRevision,
  FoundationalAxiomIds,
  ProvedTheoremIds,
  VerifiedExternalWarrantIds,
  ConditionalAssumptionIds,
  DependencyGraph,
  DischargedObligations,
  UndischargedObligations
).
```

Every premise used by the certified statement MUST occur in this structure.
`FoundationalAxiomIds` are members of a foundation fixed independently before
the certificate-specific objects and admitted by that foundation's explicit
policy. `ProvedTheoremIds` have accepted derivations from earlier nodes in the
same well-founded dependency graph. `VerifiedExternalWarrantIds` have a typed
proposition and an accepted independent verification procedure.
`ConditionalAssumptionIds` are neither proved nor warrants.

An exact unqualified positive or proved-negative result requires both
`ConditionalAssumptionIds = empty` and `UndischargedObligations = empty`.
Otherwise the statement is explicitly conditional and cannot carry an exact
claim class. Merely naming an unproved premise as an axiom, evidence item,
profile, oracle, conjecture, or obligation does not discharge it. Ambient
mathematical, physical, probabilistic, machine, compiler, library, or data
assumptions are forbidden.

The dependency graph MUST be well-founded after the separately accepted
foundation roots. A proposition, a definitionally equivalent restatement, or a
certificate whose proof depends on the current conclusion cannot warrant that
conclusion. In particular, a `ZeroCorrespondenceWarrant` MUST prove that neither
its target correspondence nor a definitionally identical proposition occurs in
its foundational-axiom, external-warrant, theorem, operator-construction, or
source-zero dependency closure.

A nonconstructive existence theorem does not authorize execution. Execution
requires a bound realization witness or a total certified extractor whose
termination and output membership are proved for the exact query. Likewise, a
class-total claim requires a uniform extractor or decision procedure, not one
existence proof per instance.

Verification returns exactly one of:

```text
VerifierResult =
    accept(VerifiedStatementIds)
  | reject-malformed(reason)
  | reject-invalid-proof(reason)
  | unresolved(ObligationIds)
  | unsupported(FeatureIds)
  | resource-exhausted(ResourceReport)
  | internal-failure(FailureReport).
```

Only `accept` warrants the listed statements. Rejection or failure of one proof
does not prove the proposition false, and exhaustion does not prove
nonattainment or infeasibility. The verifier descriptor binds its accepted proof
grammar, foundation, code identity, immutable configuration, dependency set,
resource limits, deterministic result semantics, and any trusted hardware or
runtime root. Verification MUST NOT depend on an uncommitted mutable fetch.

“Independent verifier” means that the optimizer's claimed conclusion is not a
premise of verification. Physical co-location is permitted; circular evidence
is not.

---

## 13. Computability and exact machine behavior

### 13.1 No omniscience requirement

The base protocol does not require an omniscient optimizer. For unrestricted
executable descriptions, semantic equivalence, totality, and exact refinement
are not generally decidable. Exact facts MUST therefore come from decidable
domain rules, accepted proof certificates, structurally complete theorems, or
explicitly bounded exhaustive computation.

A machine MUST NOT assume it can automatically classify arbitrary code merely
because the admission protocol accepts arbitrary future typed profiles.

Conversely, a generic undecidability, hardness, or open-world result MUST NOT be
used to weaken or dismiss a bounded typed profile whose exact admission,
refinement, coverage, and lower-bound obligations can be discharged.

### 13.2 Infinite and nonterminating spaces

Normalization or equality closure over an arbitrary rewrite system may not
terminate and may have infinitely many reachable forms. Exact termination MUST
be based on at least one of:

- a finite complete boundary;
- a well-founded reduction measure;
- a proof-complete quotient or symbolic reduction;
- a certified cutoff and global lower bound;
- a theorem establishing the required fixed point.

A saturation limit, timeout, queue exhaustion, or absence of a newly observed
candidate yields a weaker status unless coverage is independently proved.

### 13.3 Well-founded optimization

A `global-optimal` scalar claim MUST prove that an admitted minimum is attained.
A valid profile without a minimum may prove and return `unattained`; incomplete
evidence returns `optimization-incomplete`. Operational recurrence MUST exclude
improving cycles or provide a theorem that resolves them. A Pareto or
frontier-complete claim MUST prove existence of the claimed nondominated set.

Computational difficulty does not weaken exactness. A machine MAY return an
honest weaker claim; it MUST NOT return an unproved global label.

### 13.4 Total verifier boundary

Every verifier used during bounded admission or certificate checking MUST have
defined behavior for every byte string in its declared input bound and MUST
terminate within its declared resource contract. A theorem prover or external
oracle MAY produce evidence; exact admission depends on the bounded verifier,
not on trusting the producer.

### 13.5 Constructive and class-total boundaries

The base protocol is universally extensible by admission, not universally
decisive by inference. For an individual well-formed query it may return the
total answer algebra in §10.9, including an exact proved-negative or
`optimization-incomplete` result. A claim over an input or use-case class is
stronger: it MUST bind one uniform terminating classifier/solver and prove that,
for every member of the exact nonempty class, it returns the advertised exact
answer. Merely postulating an oracle, restating the desired result as a profile
obligation, or assuming termination is not a class-completeness proof.

When the class contains infinite objects or interactions, the profile MUST
define the finite observation protocol, productive response/liveness contract,
and the sense in which a result is delivered. No machine can finish reading an
unpresented infinite input; an infinite-domain claim is therefore about the
bound representation and interaction semantics, never an implicit completed
byte string.

---

## 14. Canonical interchange and addressing boundary

### 14.1 No universal wire format in Draft 0.2

This draft defines the mathematical and machine contracts of UOR-GNAF. It does
not assign a universal byte grammar, media type, multicodec, UOR address profile,
certificate grammar, or snapshot wire identifier.

The reserved stable identifier `uor-gnaf/1` MUST NOT be emitted. Implementations
MAY use private or experimental encodings only when they are explicitly labeled
non-stable and do not claim universal interchange.

`uor-gnaf/1-draft.2` identifies this development document/profile revision. It
is not a wire identifier, media type, multicodec, address-domain assignment, or
permission to emit stable serialized objects.

### 14.2 Requirements for an interchange profile

A conforming interchange profile MUST define:

- a canonical grammar for every serialized object class;
- exact integer, length, ordering, optional-field, and union-tag encodings;
- duplicate, unknown-field, and extension behavior;
- strict decoders with complete input consumption;
- an accepted-image law: decode succeeds exactly on canonical encodings;
- malformed, noncanonical, truncation, overflow, and trailing-byte rejection;
- canonical ordering independent of map, worker, and discovery order;
- object-specific domain separation for addresses;
- normative byte vectors and a rejection corpus;
- parser and resource limits;
- version transition and freeze rules.

If `Enc_T` and `Dec_T` are the canonical encoder and strict decoder for type
`T`, `Enc_T` accepts canonical typed objects and the equality below is exact
canonical-object equality. The profile MUST prove or exhaustively establish on
its bound domain:

```text
Dec_T(Enc_T(x)) = x

Dec_T(b) = x  implies  b = Enc_T(x)

Dec_T(Enc_T(x) || trailing) = reject
  whenever trailing is nonempty.
```

### 14.3 Address separation

Content, kind, operation, realization, plan, snapshot, certificate, trajectory,
and receipt addresses MUST use distinct typed domains or provably injective
domain separation. An artifact identity that depends on a parent snapshot MUST
bind that parent identity.

Address equality never substitutes for the evidence propositions in §12.1.

---

## 15. Capability-scoped conformance

### 15.1 Conformance form

A conformance statement MUST name:

```text
UOR-GNAF draft revision
implemented capability groups
kind, operation, machine, cost, and interchange profiles
maximum declared resource bounds
supported claim classes
verifier and implementation revisions
known unresolved obligations
```

Base conformance is modular. Implementing one kind or one operation does not
claim support for all admitted profiles.

### 15.2 Capability groups

- `core` — typed admission, non-conflation, evidence, snapshot, and status laws;
- `kind(K)` — one exact kind/type family;
- `operation(P)` — one exact operation profile;
- `state(U)` — closure and sealing for one universe profile;
- `extension(U,U')` — one conservative or explicitly rebuilding extension;
- `execution(P,U,X)` — complete realization execution and receipts;
- `optimality(P,U,X,M,C)` — one exact claim class `C`;
- `input(I)` — input-total classification for one exact interchange or typed
  input domain;
- `query-answer-family(F)` — bounded exact requested-answer totality over one
  nonempty query family;
- `pointwise-envelope(F)` — bounded identity-complete argmin/frontier or exact
  negative envelopes over the scopes advertised by one nonempty query family;
- `workload(W)` — one joint stateful workload and uniform-policy universe;
- `use-case-answer-class(K)` — exact requested-answer totality, including bound
  requests, over one nonempty class;
- `use-case-class(K)` — one total exact answer contract over a bound nonempty
  use-case class whose requests require global/Pareto optima or exact negatives;
- `maintained-class(K)` — transactional maintenance and recertification for one
  exact use-case class across committed extensions;
- `analytic(A)` — one constructive refinement/completion/operator/spectrum
  profile under §3.6;
- `accumulation(G)` — one canonical accumulation algebra under §10.7;
- `gnaf-state(G,U)` — `core + state(U) + accumulation(G)`, participating kind
  lifts/multiplicity, provenance, operation closure, and the certified envelope;
- `interchange(W)` — one fully specified canonical wire/address profile.

### 15.3 Core requirements

| Requirement | Normative condition |
|---|---|
| `GNAF-CORE-001` | The implementation MUST preserve every identity and proposition separation in §2. |
| `GNAF-CORE-002` | Identity, address, location, or discovery MUST NOT be used as semantic warrant. |
| `GNAF-CORE-003` | Unknown facts MUST remain unknown and MUST NOT be treated as false. |
| `GNAF-CORE-004` | Declaration and certificate dependencies MUST be explicit and non-self-warranting. |
| `GNAF-CORE-005` | Exact operations MUST preserve the requested observation on their complete valid scope. |
| `GNAF-CORE-006` | Syntax, evaluated operators, configurations, trajectories, plans, and addresses MUST remain typed separately. |
| `GNAF-CORE-007` | Every result MUST have separate total request/runtime/optimization status projections; every positive or fallback result MUST carry exactly one honest strongest actual claim class, while exact negative, incomplete, and request-error branches MUST carry none. |
| `GNAF-CORE-008` | The stable `uor-gnaf/1` identifier MUST NOT be emitted before assignment. |
| `GNAF-CORE-009` | Exact query domains and required-result relations MUST be nonempty; vacuous coverage MUST NOT establish an exact claim. |
| `GNAF-CORE-010` | `Problem_q`, both observation maps, and `Accept_q` MUST be total, extensional, and type-correct; a query MAY require a derived observation without computing the operation's entire base output. |
| `GNAF-CORE-011` | Correctness domain, evaluation domain, and evaluation profile MUST remain separate; sampling or a probability law MUST NOT narrow exact correctness. |
| `GNAF-CORE-012` | Every exact result or proved-negative result MUST have a closed explicit hypothesis set under §12.7; an ambient or renamed assumption is not discharged. |
| `GNAF-CORE-013` | Requested claim, actual claim, authorized fallbacks, and downgrade behavior MUST be explicit; an unauthorized weaker result MUST NOT execute. |
| `GNAF-CORE-014` | Input-total and use-case-class-complete claims MUST use the exact total classifiers and quantified contracts of §§8.7, 10.9, and 10.10. |

### 15.4 Kind requirements

| Requirement | Normative condition |
|---|---|
| `GNAF-KIND-001` | A kind MUST bind exact parameters, carrier, validity, equivalence, observations, and identity. |
| `GNAF-KIND-002` | Equivalence MUST be an equivalence relation on valid values. |
| `GNAF-KIND-003` | Accepted equivalence certificates MUST be sound; verifier failure is not semantic inequality. |
| `GNAF-KIND-004` | Cross-kind substitution MUST use an exact typed bridge. |
| `GNAF-KIND-005` | A semantic change MUST create a new kind identity. |
| `GNAF-KIND-006` | Algebraic or analytic terminology MUST be backed by all laws required in §3.5. |
| `GNAF-KIND-007` | A computable canonical claim MUST prove termination, soundness, idempotence, and completeness on its domain. |

### 15.5 Operation requirements

| Requirement | Normative condition |
|---|---|
| `GNAF-OP-001` | An operation MUST bind exact signatures, valid invocations, observations, state/effect behavior, failures, and composition. |
| `GNAF-OP-002` | Eligibility, exactness, feasibility, cost, and optimality MUST be separately warranted. |
| `GNAF-OP-003` | Every operation on a quotient MUST be extensional over that quotient. |
| `GNAF-OP-004` | Representation-sensitive operations MUST consume a finer syntax-bearing kind. |
| `GNAF-OP-005` | Every connected plan boundary MUST have exact typed composition/refinement evidence. |
| `GNAF-OP-006` | Partial operations MUST have exact refusal/result semantics. |
| `GNAF-OP-007` | Fallback, validation, dispatch, and all outcome paths MUST be declared and charged. |
| `GNAF-OP-008` | Multi-input/output, sharing, effects, and retained state MUST follow the declared machine, not an implicit tree model. |
| `GNAF-OP-009` | For nondeterministic, probabilistic, or relational behavior, a complete realization MUST reproduce the exact allowed relation/law or its explicitly declared refinement order; support inclusion alone is insufficient. |
| `GNAF-OP-010` | A composite's natural correctness domain MUST be derived from every precondition, failure, state, and effect boundary; it MUST NOT be inherited from only its first or last operation. |
| `GNAF-OP-011` | Higher-order, streaming, and infinite behavior MUST be carried by admitted typed syntax/observations and a productive completion contract, never by an untyped meta-level exception. |

### 15.6 State and extension requirements

| Requirement | Normative condition |
|---|---|
| `GNAF-STATE-001` | Knowledge accumulation MUST be associative, commutative, and idempotent set/join accumulation. |
| `GNAF-STATE-002` | Semantic sequence order MUST be explicit content and MUST NOT be erased by accumulation. |
| `GNAF-STATE-003` | Closure MUST be extensive, monotone, idempotent, and satisfy the merge law. |
| `GNAF-STATE-004` | Every equality rewrite MUST preserve semantic state; every semantic merge MUST implement its declared operation; every quotient operation MUST be extensional. |
| `GNAF-STATE-005` | Local non-adjacency MUST NOT be labeled global without exhaustive/no-improver coverage, proof-complete reduction, or sound lower-bound/no-dominator attainment. |
| `GNAF-STATE-006` | Operational pruning MUST have contextual-dominance or exact-reconstructibility evidence. |
| `GNAF-STATE-007` | A seal MUST bind the complete semantics, universe, machine, objective, continuation, and verifier boundary. |
| `GNAF-STATE-008` | Incremental semantic results MUST equal full closure; every successful incremental seal MUST equal full obligation verification. |
| `GNAF-STATE-009` | A post-seal result certificate MUST derive from the seal's committed query-coverage basis and bind its `SealId`, or the candidate MUST be resealed first. |
| `GNAF-STATE-010` | `SnapshotBody` MUST commit accumulation profiles and contribution, occurrence/multiplicity, semantic-subject, canonical-state, pre-identity operational-alternative/reconstruction, dependency, and unresolved-obligation roots. Snapshot-scoped operational-basis evidence belongs only to the seal. |
| `GNAF-STATE-011` | Symbolic closure MUST carry base inclusion, rule closure, and leastness/completeness evidence; satisfying the closure equations alone is insufficient. |
| `GNAF-STATE-012` | Operational retention, reconstruction, contextual dominance, and garbage collection MUST be justified by the committed `OperationalBasis`; a selected envelope alone does not license deletion. |
| `GNAF-STATE-013` | Batch admission MUST be deterministic and report every declaration as admitted, rejected, or pending; unresolved declarations MUST NOT enter the active base. |
| `GNAF-STATE-014` | Publication MUST linearize against the exact complete expected `KnowledgeHead` and optional runtime-state read set; conflict, retry, rebase, and transaction replay MUST obey §7.11. |
| `GNAF-STATE-015` | Transaction identity MUST make identical replay idempotent and MUST reject identity reuse with different content. |
| `GNAF-STATE-016` | Dependency manifests and impact analysis MUST cover domains, negative assumptions, use-case classes, analytic warrants, and transitive dependents; omission invalidates incremental recertification. |
| `GNAF-STATE-017` | Closure and sealing procedures MUST return the explicit complete, checkpoint/incomplete, unsupported, conflict, and failure statuses defined by their result algebras. |
| `GNAF-STATE-018` | A resumed checkpoint MUST match its exact base, request, universe, work roots, and verifier identities; retargeting requires an explicit rebase/new request. |
| `GNAF-STATE-019` | Knowledge publication MUST NOT activate an unsealed candidate. A new sealed snapshot becomes executable only through an exact, costed, atomic activation/migration into a new `DeploymentConfiguration`. |
| `GNAF-STATE-020` | Reseal, activation, update, and execution MUST operate on one explicit `CoordinatorState`, use full-value CAS, and record idempotent transaction receipts. |
| `GNAF-STATE-021` | Every potentially unbounded update/proof stage MUST be resource-bounded and return a complete result, exact stage checkpoint, or typed terminal status. |
| `GNAF-STATE-022` | `VERIFY_SEAL` and every result verifier acceptance MUST include the exact required statement identity, not merely an `accept` tag. |
| `GNAF-STATE-023` | Runtime quarantine MUST be committed in `RuntimePolicyStateRoot`, participate in deployment identity/CAS, and prevent another run until an explicit investigation, replacement, or recertification transition; only a conforming full-head-CAS activation under §7.11 may clear it. |
| `GNAF-STATE-024` | Terminal success, permitted refusal/failure, incomplete partial execution, productive prefix, and implementation violation MUST be classified from the declared complete outcome contract before status assignment. |
| `GNAF-STATE-025` | Productive infinite behavior MUST return finite certified prefixes and exact continuation state; no external procedure may block indefinitely after a declared observation boundary. |
| `GNAF-STATE-026` | Every restricted query/workload result and negative status MUST use one identity-bound applicable carrier; only a branch-specific omission bridge may promote its scope. |
| `GNAF-STATE-027` | Query, workload, family, input-total, and use-case-class certificates MUST use the matching dependent scope binding; heterogeneous classes MUST NOT use one ambient machine, objective, or universe. |
| `GNAF-STATE-028` | A comparison request MUST precommit its exact predicate, comparator/information boundary, quantifier order, extrema policy, units, and candidate quantifier; false and unknown predicates MUST remain distinct. |
| `GNAF-EXT-001` | Every extension MUST create a new immutable snapshot and bind parent, delta, and rejection identities. |
| `GNAF-EXT-002` | Conservative extension MUST preserve old typed semantics and operation observations. |
| `GNAF-EXT-003` | Replay-free semantic update MUST prove cross-universe absorption or an equivalent sufficient-state theorem. |
| `GNAF-EXT-004` | Replay-free operational pruning MUST prove extension-sufficiency over the declared continuation class. |
| `GNAF-EXT-005` | Old optimality certificates MUST NOT be relabeled for a new snapshot. |
| `GNAF-EXT-006` | A changed old quotient or representation-sensitive new operation MUST use a new universe, finer kind, or exact replay. |
| `GNAF-EXT-007` | A maintained-class claim MUST prove that every committed extension is classified, all affected class members are recertified before publication, and nonconservative changes create explicit invalidation or migration boundaries. |

### 15.7 Execution and optimality requirements

| Requirement | Normative condition |
|---|---|
| `GNAF-EXEC-001` | `X` MUST bind the complete starting state, primitives, outcomes, resources, information, effects, and accounting boundary. |
| `GNAF-EXEC-002` | Every admitted run MUST be well-typed, exact, feasible, and conforming on every allowed outcome. |
| `GNAF-EXEC-003` | Complete-system comparisons MUST use one common outer boundary; excluded preparation MUST be identically bound in common initial `X` and labeled prepared. |
| `GNAF-EXEC-004` | Every exact run MUST produce or reproducibly bind the receipt fields in §8.6. |
| `GNAF-EXEC-005` | `Exec_X` MUST be nonempty and complete for each admitted system/configuration/input; one observed run or a subset of outcomes is not the execution semantics. |
| `GNAF-EXEC-006` | Runtime execution MUST pin one deployment configuration and record effects, partial commit, cancellation, retry, and fallback under its declared atomicity/idempotence model. |
| `GNAF-EXEC-007` | Runtime success, optimization status, and claim class MUST be committed separately; a successful fallback is not a successful global-optimal result. |
| `GNAF-OPT-001` | `M` MUST type and charge or explicitly free every in-boundary trace event. |
| `GNAF-OPT-002` | Query global/Pareto claims MUST cover `U_sys` and workload global/Pareto claims MUST cover `U_policy` by enumeration, proof-complete reduction, or theorem; proper `U_D`/`U_PD` carriers remain restricted unless the exact branch-specific omission bridge applies. |
| `GNAF-OPT-003` | A scalar global claim MUST prove admission, attainment, and complete no-improver coverage by exhaustive comparison, proof-complete reduction, or a universal lower bound attained. |
| `GNAF-OPT-004` | A Pareto claim MUST prove no-dominator coverage; frontier completeness MUST cover every omission. |
| `GNAF-OPT-005` | Tie selection MUST occur only after preserving the exact argmin or frontier. |
| `GNAF-OPT-006` | Infeasible, unattained, and incomplete cases MUST NOT emit a global optimum. |
| `GNAF-OPT-007` | Representation minimality MUST NOT be promoted to execution optimality without the bridge in §10.6. |
| `GNAF-OPT-008` | Family optimality MUST compare one uniform system under a bound workload functional. |
| `GNAF-OPT-009` | Every result certificate MUST bind the common fields and exactly the dependent query, workload, family, input-total, class, or maintained-transition `ScopeBinding` required by §12.6. |
| `GNAF-OPT-010` | Every query/use-case request MUST bind a type-correct member, complete-set, bound, or execution result mode; only selection from a complete set binds the applicable tie/preference policy. |
| `GNAF-OPT-011` | Every cost and objective stage MUST be total on every admitted candidate, with exact units, aggregation, quantifier order, and order laws; unknown cost MUST NOT remove a candidate. |
| `GNAF-OPT-012` | `frontier-complete` MUST certify exact set equality with the full nondominated set; a cofinal dominance cover MUST use its distinct weaker claim. |
| `GNAF-OPT-013` | A pointwise envelope MUST NOT be called one executable uniform family optimum unless a single admitted selector/policy, including switching and information costs, realizes it. |
| `GNAF-OPT-014` | Every workload binds its exact quantifier prefix. Competitive, instance, and comparator-relative asymptotic claims use `comparator(ComparatorBody)`; absolute asymptotic claims use `absolute-asymptotic(AbsoluteScoreProfile)`; ordinary noncomparison workloads use `not-applicable`. |
| `GNAF-OPT-015` | A use-case-class-complete claim MUST bind its classifier, request constructor, uniform terminating exact solver/extractor, transition/result-certificate semantics, verifier, and resource envelope, and prove exact totality over the whole declared class. |
| `GNAF-OPT-016` | Every selected result obtained from a frontier MUST include the preference/selector itself and its computation, information, dispatch, and switching costs inside the compared boundary. |
| `GNAF-OPT-017` | A certified optimal/Pareto member or comparison bound MUST use its distinct answer branch and MUST NOT be reported as an identity-complete argmin/frontier or attained optimum unless the stronger evidence is present. |
| `GNAF-OPT-018` | `argmin-complete`, `frontier-complete`, `workload-argmin-complete`, and `workload-frontier-complete` MUST prove exact identity-set equality with the applicable mathematical set. |
| `GNAF-OPT-019` | `use-case-class-answer-complete` MAY cover exact comparison-bound requests; `use-case-class-complete` MUST exclude mere bounds, proper restricted universes, and heuristic/measured claims and require complete-universe optimal/Pareto answers or exact negatives for every member. |
| `GNAF-OPT-020` | Query-family answer completeness MUST prove uniform bounded termination and `ExactRequestedQueryAnswers` membership for every exact nonempty family member; pointwise-envelope completeness additionally restricts members to full complete-set shapes. |
| `GNAF-OPT-021` | A claim covering arbitrary raw inputs MUST use one `InputTotalCapabilityBinding` over an exact nonempty input profile and a compatible accepted family/class capability certificate: the classifier MUST form a terminating four-way disjoint cover, every `valid` input MUST map to the bound admitted query/use-case and receive the exact capability answer, and `invalid`, `unresolved`, or `unsupported` inputs MUST return only their exact nonoptimization status. The accepted custom conjunction statement MUST bind the exact standard `CommonCertificateBinding` plus `InputTotalCapabilityBinding`, their `CertificateBody`, and its derived `CertificateId`; acceptance of an unbound theorem ID cannot authorize the conjunction certificate. |

### 15.8 Accumulation requirements

| Requirement | Normative condition |
|---|---|
| `GNAF-ACC-001` | A canonical accumulation profile MUST bind exact raw/semantic domains, merge, evaluation, admissible fibers, representation cost, and a total canonical selector. |
| `GNAF-ACC-002` | Claimed order-invariant accumulation MUST prove both monoids and the evaluation homomorphism. |
| `GNAF-ACC-003` | Representation minimality MUST prove nonempty fibers, attainment, and the minimum law; a canonical selection additionally proves one deterministic selected representative. |
| `GNAF-ACC-004` | Normalized merge MUST prove absorption; idempotence MUST NOT be assumed unless semantic merge is idempotent. |
| `GNAF-ACC-005` | Bounded exact accumulators MUST prove `Safe_G` for every intermediate in every covered run or refuse before commit. |
| `GNAF-ACC-006` | Every participating kind MUST have an equivalence-respecting lift, explicit quotient/injectivity law, and occurrence/multiplicity semantics. |
| `GNAF-ACC-007` | Every participating operation MUST bind its `Sem_G` update, normalize every allowed result, and prove normalization compatibility. |
| `GNAF-ACC-008` | Infinite or limit accumulation MUST bind its completion, convergence, grouping/order law, continuity or interchange theorem, and productive machine representation; finite-prefix testing is insufficient. |

### 15.9 Analytic requirements

| Requirement | Normative condition |
|---|---|
| `GNAF-AN-001` | A claim derived from stages MUST bind a directed typed `RefinementDiagram` with total declared refinement maps and identity, composition, validity, extensionality, observation, and coherence laws. Limit embeddings are required when used; projections are required only when declared and then need their own laws. |
| `GNAF-AN-002` | A completion claim MUST bind a constructive `CompletionProfile`, dense embedding, exact uniformity-induced Cauchy/name-equivalence predicates, admitted witnesses, name/carrier coverage, a total limit operator, and existence/uniqueness proofs. |
| `GNAF-AN-003` | Every weighted or unbounded operator MUST bind an exact domain, graph, action, codomain, domain-membership evidence, and `DensityStatus`; density MUST be proved for adjoints and every other density-dependent claim. Behavior outside the domain MUST remain undefined or explicitly extended. |
| `GNAF-AN-004` | Closed, closable, closure, adjoint, symmetric, and self-adjoint claims MUST use their exact graph/domain predicates; none may be inferred from finite tests or from another property. |
| `GNAF-AN-005` | Passing a property through a limit MUST use a property-specific `LimitTransferWarrant` whose hypotheses are proved for that exact diagram and topology. |
| `GNAF-AN-006` | Spectrum claims MUST distinguish point, continuous, residual, approximate-point, and resolvent classes whenever the profile can distinguish them. |
| `GNAF-AN-007` | A zero/spectrum correspondence MUST be noncircular and prove each claimed direction. Multiplicity is required only when compared; no-loss/no-pollution is required when the correspondence is transferred from approximations. |
| `GNAF-AN-008` | A finite compression, truncation, numerical sample, or convergent value sequence MUST NOT establish a limit claim without the property-specific theorem in `GNAF-AN-005`; a spectral approximation additionally requires the exact §3.6 spectral-approximation warrant, and a zero correspondence additionally requires `GNAF-AN-007`. |
| `GNAF-AN-009` | A full/global spectral approximation claim MUST prove its region covers the complete target spectrum and advertised stage scope; empty or proper-region warrants carry only an explicitly restricted claim. |

### 15.10 Interchange requirements

| Requirement | Normative condition |
|---|---|
| `GNAF-WIRE-001` | A claimed interchange profile MUST define a canonical grammar and strict complete-consumption parser. |
| `GNAF-WIRE-002` | A wire profile MUST satisfy round-trip and accepted-image laws. |
| `GNAF-WIRE-003` | A wire profile MUST publish canonical vectors and malformed/noncanonical rejection cases. |
| `GNAF-WIRE-004` | Addresses MUST use typed domain separation and MUST NOT act as warrants. |

---

## 16. Normative fixtures and rejection corpus

The fixtures in this section test the abstract laws. They do not privilege a
programming language, repository, registry, or implementation construction.

### 16.1 Fixture `GNAF-VEC-01` — finite global path and extension

Declare distinct kinds `A`, `B`, and `C`, values `a0`, `b0`, and `c0`, and exact
operations:

| Realization | Transition | Scalar cost |
|---|---:|---:|
| `rAB` | `a0 -> b0` | 2 |
| `rBC` | `b0 -> c0` | 3 |
| `rAC6` | `a0 -> c0` | 6 |

Let query `q0` require exact final value `c0` from `a0`, bind the system-universe
identity derived from `S0`, let plan cost be the sum of operation costs, and let
the complete `S0` grammar contain precisely the three operations and their
well-typed finite compositions.

Each well-typed finite composition denotes one complete `prepared-plan` system
beginning from the same exact `X`;
`U_sys(Body(S0),Problem_q0,X)` is exactly those systems.
Validation, selection, dispatch, receipt, and output actions are absent or
explicitly free under this fixture's `M`. Thus the displayed plan identity names
the complete prepared executor rather than an uncharged internal leaf.

Required result:

```text
Extract_S0(q0) = [rAB, rBC]
certified cost = 5.
```

Now create `S1` by admitting exact realization `rAC4 : a0 -> c0` at cost `4`
without changing any other semantic contract. Let `q1` be the transported query
with the same request, machine, cost, aggregation, objective, continuation,
result mode, tie policy, and any preference policy as `q0`, a new `QueryId`, and
the system-universe identity derived from `S1`.

Required result:

```text
Extract_S1(q1) = [rAC4]
certified cost = 4.
```

The `S0` certificate remains valid for `S0` and is invalid as an `S1` global
certificate unless recertified. An implementation that retains cost `5` as the
`S1` optimum fails `GNAF-EXT-005` and `GNAF-OPT-003`.

### 16.2 Fixture `GNAF-VEC-02` — Pareto envelope

For one exact same-observation query, admit distinct realizations with costs:

```text
rA = (1,3)
rB = (2,2)
rC = (3,1)
rD = (3,3).
```

These four identities are exactly `Adm_S(q,X)`. Each is one complete
`prepared-plan` outer system beginning from the same exact `X`; the displayed
vector is its exact whole-boundary cost under one `M`. Validation, selection,
dispatch, receipt, and output actions are absent or explicitly free in this
fixture. The query binds the resulting complete system-universe identity, so no
unlisted fifth candidate or uncharged internal leaf is in scope.

Under componentwise lower-is-better dominance, the complete frontier is exactly

```text
{ rA, rB, rC }.
```

`rD` is dominated. A preference MAY choose one of `rA`, `rB`, or `rC` for
execution but does not alter the certified frontier. A frontier member MUST NOT
be removed from a `frontier-complete` result without a newly declared scalarized
objective, strict dominance by a retained member, or exact identity
reconstruction permitted by the claim.

### 16.3 Fixture `GNAF-VEC-03` — unattained infimum

Let the complete admitted candidate family be `{r_n | n >= 1}` with costs in
the exact real numbers

```text
J(r_n) = 1 + 1/n.
```

The infimum in the declared complete ordered real codomain is `1`, but no
candidate attains it. Required result:

```text
status = unattained
claim != global-optimal.
```

### 16.4 Fixture `GNAF-VEC-04` — local normality is not globality

Let same-observation representations be `a`, `b`, and `c` with costs `2`, `1`,
and `0`. Let the declared local rewrite set contain only `a -> b`.

`b` is irreducible but is not globally minimum because `c` exists. A proof based
only on termination and local irreducibility MUST be rejected. A conforming
global proof MUST cover `c` through complete reductions, enumeration, or a lower
bound attained at cost `0`.

### 16.5 Fixture `GNAF-VEC-05` — extensionality rejection

Let `x,y in Raw_(S,T)` satisfy `x ~_(S,T) y`. Propose an operation `spell` that
returns `0` on `x` and `1` on `y` while claiming to consume the quotient type.

Required result:

```text
AdmissionStatus(spell) = rejected
reason = non-extensional over the declared quotient.
```

The operation MAY instead be declared over a new syntax-bearing kind in which
`x` and `y` are not equivalent.

### 16.6 Fixture `GNAF-VEC-06` — extension can change spelling

Let `S0` instantiate the §10.7 representation-minimal canonical profile with
selected representation `r3` for semantic value `3`. Let `S1` conservatively
add the sole new representation `tau` for that value with strictly lower
declared representation cost; the canonical selector minimizes that cost before
its tie policy.

Required result:

```text
Eval_G'(Project_G'(iota_GR(r3))) = 3
Project_G'(iota_GR(r3)) = [tau].
```

Meaning is preserved; old physical spelling is not. A profile claiming the old
spelling remains canonical fails unless it separately proves optimum stability.

### 16.7 Fixture `GNAF-VEC-07` — current-cost pruning is unsafe

Let same-semantic derivations have the same external boundary types and
capability interface but different retained capability-state values:

```text
p = (current cost 1, retained capability 0)
q = (current cost 2, retained capability 1).
```

An allowed future continuation requires capability `1`, costs `0` from `q`, and
costs `100` to reconstruct from `p`.

Required result: `q` MUST remain retained or reconstructible. Current scalar
cost does not prove `p <=^ctx q`.

### 16.8 Fixture `GNAF-VEC-08` — enlarged machine invalidates a bound

Let machine grammar `X` be a proper subset of `X'`. Let `LB_X = 3` be a proved
lower bound over every realization admitted by `X`, attained by `r`. Admit in
`X'` one exact same-observation realization `r'` with cost `2` that is not in
`X`.

Required result: the `X` certificate remains valid for `X` but MUST NOT certify
`X'`; the `X'` optimum is at most `2`.

**Informative UOR-NAF witness.** In the restricted rank-one contraction tree
grammar with no shifts of intermediates or sharing, coefficient `45` has NAF
weight `4` and costs `3` additions. An enlarged state machine that retains and
shifts an intermediate can use `t = a + 8a; result = t + 4t` at `2` additions.
This witness is informative, not a construction imported by the core fixture.

### 16.9 Fixture `GNAF-VEC-09` — non-idempotent accumulation

In an exact integer-sum accumulation profile:

```text
Project_G([1]) boxplus_G Project_G([1]) = Project_G([2]),
```

not `Project_G([1])`. Any implementation that assumes declaration-join
idempotence for semantic integer addition MUST be rejected.

### 16.10 Fixture `GNAF-VEC-10` — concrete batching and absorption

Bind `Raw_G` to finite integer multisets, `Sem_G` to exact integers,
`mergeRaw_G` to multiset union, and `Eval_G` to exact summation. Bracket notation
below denotes a multiset. Let `RepCost_G` be multiset cardinality; the canonical
selector chooses the empty multiset for zero and the unique singleton containing
the exact sum otherwise. Thus

```text
Project_G(r) = []                 if Eval_G(r) = 0
Project_G(r) = [Eval_G(r)]        otherwise.
```

Let `x=[3]`, `y=[5]`, and `z=[-3]`. Required results are:

```text
Project_G(mergeRaw_G(x,y))                              = [8]
Project_G(mergeRaw_G(Project_G(x),y))                   = [8]
Project_G(mergeRaw_G(x,Project_G(y)))                   = [8]
Project_G(mergeRaw_G(Project_G(x),Project_G(y)))        = [8]
(x boxplus_G y) boxplus_G z                             = [5]
x boxplus_G (y boxplus_G z)                             = [5]
x boxplus_G x                                           = [6]
x boxplus_G [-3]                                        = [].
```

All permutations agree because this fixture claims exact commutative integer
addition. The last two equations test non-idempotence and the canonical unit.

### 16.11 Fixture `GNAF-VEC-11` — cross-universe absorption

Use the integer profile of `GNAF-VEC-10`. Let old raw contribution multiset
`x=[1,2]`, so
`Project_G(x)=[3]`. Extend conservatively to `G'` by adding a new raw token
`tau` with exact value `3`. In `G'`, each ordinary integer atom has
representation cost `2`, `tau` has cost `1`, and multiset cost is additive, so
`[tau]` improves `[3]` while `[8]` improves `[tau,5]`.
Let the new contribution be `y=[5]` and let `iota_GR` embed old multisets.

Required result:

```text
Project_G'(mergeRaw_G'(iota_GR([1,2]), [5]))
  = Project_G'(mergeRaw_G'(iota_GR([3]), [5]))
  = [8].
```

`Project_G'(iota_GR([3])) = [tau]` is permitted and expected under the declared
cost policy. If the first equality fails, the extension MUST NOT claim
replay-free semantic update.

### 16.12 Fixture `GNAF-VEC-12` — occurrence identity and snapshot accumulation

Let two declarations `d1` and `d2` carry equal semantic integer contribution
`1` but distinct occurrence identities. Under the integer-sum profile of
`GNAF-VEC-10`, a committed update containing both MUST bind:

```text
ContributionRoot = Identity({Occurrence(d1,1), Occurrence(d2,1)})
Multiplicity(1) = 2
SemanticSubject = 2
CanonicalState = [2].
```

Deduplicating by value gives `1` and MUST fail. Replaying the same transaction
identity with the same batch returns the already committed snapshot; reusing it
with only `d1` is a transaction-identity conflict.

### 16.13 Fixture `GNAF-VEC-13` — concurrent linearization

Let sealed snapshot `S0` have two concurrent requests `tA` and `tB`, both with
`ExpectedKnowledgeHead = published(SnapshotId(S0),SealId(S0))`, adding distinct
declarations `a` and `b`. If `tA` commits `S1`, direct commit of the unrevised `tB` MUST return
`conflict(actualParent = SnapshotId(S1))`. A deterministic rebase of `tB` over
`S1` creates a new transition whose parent is exactly `S1`; if it succeeds, its
snapshot contains both admitted deltas. Publishing two children as the unique
linear successor, silently dropping either delta, or changing the parent field
after identity computation MUST fail.

### 16.14 Fixture `GNAF-VEC-14` — complete probabilistic behavior

Let `Pcoin` return `0` or `1` with the exact law
`Pr(0)=Pr(1)=1/2` on every invocation. Candidate `fair` has that law; candidate
`zero` always returns `0`. Both observed supports are subsets of `{0,1}`, and
every realized output of both candidates is individually permitted. Nevertheless:

```text
BehaviorConforms(fair,Pcoin) = true
BehaviorConforms(zero,Pcoin) = false.
```

No finite sample can replace the exact law warrant. If the query intentionally
asks only for support refinement, that is a different `Problem_q` and claim.

### 16.15 Fixture `GNAF-VEC-15` — symbolic least closure

Let the active declaration base be `{d_a}` with `Seed_U({d_a})={a}`, and let the
sole closure rule be `a -> b`. Both
`{a,b}` and `{a,b,c}` contain the base and are rule-closed, but the least closure
is exactly `{a,b}`. A symbolic closure certificate returning `{a,b,c}` without a
proof that `c` is derivable MUST fail its leastness obligation even though it is
a fixed point. A checkpoint containing `{a}` is sound partial progress but is
not a complete seal.

### 16.16 Fixture `GNAF-VEC-16` — verifier result separation

For proposition `2+2=4`, submit respectively a valid proof, malformed bytes, an
invalid derivation, a proof using an unsupported rule, and a valid proof whose
declared resource budget is too small. Required verifier results are
`accept`, `reject-malformed`, `reject-invalid-proof`, `unsupported`, and
`resource-exhausted`, with the first specifically
`accept({StatementId(2+2=4)})`. Only that listed statement is warranted; an
`accept` for an unrelated lemma is insufficient. None of the last
four warrants `2+2 != 4`, and none may be converted to `infeasible`.

### 16.17 Fixture `GNAF-VEC-17` — pointwise envelope is not a uniform policy

Let a hidden environment bit `h` be chosen before execution and remain outside
the system's information filtration. Systems `R0` and `R1` cost `0` when their
index equals `h` and `10` otherwise. The pointwise envelope, computed with
post-hoc knowledge, has value `0` for each `h`. Every deterministic uniform
system has worst-case value `10`; a fair randomized system has expected value
`5` under the declared fair law. Therefore no executable system may be
certified with the pointwise value `0`. Revealing `h` before action defines a
different workload and may change the optimum.

### 16.18 Fixture `GNAF-VEC-18` — effects, failure, and retry

Let a two-stage realization first append durable effect `e1` and then either
produce the target with `e2` or fail. With no rollback and non-idempotent append,
retry after failure can duplicate `e1`. The realization is inadmissible for a
problem requiring exactly-once atomic effects unless it supplies a proved
transaction/idempotency protocol. A receipt after stage-one failure MUST report
the partial effect and failure; it MUST NOT report semantic success, zero cost,
or a safe retry. Declaring a compensating action changes both behavior and cost
and MUST be included in `Exec_X` and `M`.

### 16.19 Fixture `GNAF-VEC-19` — quantified dependency invalidation

Let certificate `c` prove optimality for every integer input using the premise
“primitive `r` is exact on all integers.” Extend the active machine declaration
so that `r` is exact only on nonnegative integers, without changing its display
name or its behavior on the test set `{0,1}`. The dependency/impact cone for `c`
MUST include the changed correctness-domain proposition and invalidate `c`.
Test-set agreement, unchanged addresses of unrelated artifacts, or an unchanged
selected plan cannot preserve the old input-total claim.

### 16.20 Fixture `GNAF-VEC-20` — derived observation without full output

Let base operation `P(x)` produce the pair `(f(x),g(x))`. Query `q` requires only
the first projection and defines:

```text
Problem_q(x) = f(x)
TargetObservation_q(P(x)) = f(x)
CandidateObservation_q(y) = y
Accept_q(u,v) iff u = v.
```

Candidate `Rdirect` computes `f(x)` exactly at cost `1`; `Rfull` computes both
components at cost `3`. Both are complete realizations of `Problem_q`, and
`Rdirect` is optimal in the complete two-system universe. Requiring `Rdirect`
to materialize `g(x)` would silently replace `Problem_q` by a stronger problem
and MUST fail query identity matching.

### 16.21 Fixture `GNAF-VEC-21` — correctness and evaluation scopes

Let exact correctness require `R(x)=x^2` for every integer. Let the evaluation
law place probability one on `x=0`. Candidate `Rzero(x)=0` has expected
evaluation error zero under that law but fails exact correctness at `x=1`.
It is inadmissible for the exact problem, irrespective of its evaluation score.
It may compete only in a separately identified approximate/statistical problem
whose acceptance relation permits that behavior.

### 16.22 Fixture `GNAF-VEC-22` — constructive analytic refinement

Let `H_n = span_Q(i){e_0,...,e_n}` with its exact Gaussian-rational
inner-product structure, coherent isometric inclusions, and finite-support union
`H_fin`. Bind a constructive completion whose Cauchy names complete `H_fin` to
the complex Hilbert space `l2(N_0;C)` and whose canonical embedding is dense.
Define the number operator

```text
(A x)_k = k x_k
Dom(A) = {x in l2(N_0;C) : sum_k k^2 |x_k|^2 < infinity}.
```

An `analytic(A)` implementation of this fixture MUST separately verify the
dense domain, exact graph, closedness, adjoint domain/action, and
self-adjointness. Its finite compression `A_n` has spectrum `{0,...,n}`; the
limit profile MUST prove that `spectrum(A)=N_0` and that the compressions cause
neither spectral pollution nor loss in the declared convergence sense.

For a zero-correspondence claim, bind the independently defined entire function
`Z(z)=1/Gamma(-z)` and a proof from its analytic definition that its zeros are
exactly `N_0`, then prove `Z(lambda)=0 iff lambda in spectrum(A)`. Defining `Z`
as “zero exactly on the spectrum,” importing the desired equivalence as an
axiom, or extrapolating from finitely many `A_n` MUST be rejected as circular or
incomplete.

### 16.23 Fixtures `GNAF-VEC-23` through `GNAF-VEC-33` — transaction and runtime hardening

- **`GNAF-VEC-23`, reseal/update race.** Start at
  `KnowledgeHead=published(C,s1)`. Prepare an update against that full head;
  concurrently publish reseal `s2` for the same `C`. Committing the prepared
  update MUST conflict and preserve `published(C,s2)`; comparing only `C` fails.
- **`GNAF-VEC-24`, pre-candidate checkpoint.** Exhaust the closure budget before
  a body exists. The immutable pending object MUST bind
  `pre-candidate(TransactionId)`, stage/work/output roots, and emit no
  `SnapshotId`. Resuming the identical checkpoint is deterministic; changing
  the rule universe, verifier, or request conflicts.
- **`GNAF-VEC-25`, activation/execution race.** Prepare activation from `D0`;
  concurrently execute and publish `D1` with effect `e`. Activation MUST fail
  its full-value CAS and preserve `D1/e`; it MUST NOT migrate stale state.
- **`GNAF-VEC-26`, corrupt prepared input.** Alter any stored request-body,
  candidate body/id, history/report root, incomplete status, seal certificate,
  seal id, or prepare-proof field while retaining the transaction identity.
  `COMMIT_UPDATE` MUST reject the exact integrity mismatch/reuse and publish
  nothing.
- **`GNAF-VEC-27`, seal staging.** Snapshot-scoped dominance or envelope evidence
  constructed before `SnapshotId`, or included in `SnapshotBody`, MUST fail.
  Post-ID evidence with an altered retention/dependency root MUST be rejected by
  `VERIFY_SEAL`.
- **`GNAF-VEC-28`, runtime conformance violation.** After effect `e`, produce an
  output outside `Problem_q`. The machine MUST atomically preserve `e`, publish
  a deployment version whose `RuntimePolicyStateRoot` quarantines the binding,
  and return `violation`; it MUST NOT return success or any exact actual claim.
- **`GNAF-VEC-29`, post-closure resource checkpoint.** Exhaust during
  accumulation, derivation construction, exact impact, or post-ID evidence.
  The procedure MUST publish no incomplete body/seal, return the exact
  stage-tagged checkpoint, and resume to the same result as full recomputation.
- **`GNAF-VEC-30`, productive execution.** For an admitted infinite stream,
  reaching a finite certified observation boundary MUST return `productive`
  with committed effects/state and a bound continuation. It MUST NOT block,
  report terminal acceptance, or resume against a changed deployment.
- **`GNAF-VEC-31`, lost execution response.** An execution transaction appends
  one non-idempotent effect and commits, but its response is lost. Replaying the
  identical request/body MUST return the original immutable result/receipt and
  MUST NOT rerun the effect; the same id with a different invocation is rejected.
- **`GNAF-VEC-32`, reservation crash.** Crash once after reservation but before
  an effect and once after an effect becomes visible. The exact recovery request
  MUST reconcile the committed intent/effect ledger, return a partial/terminal/
  quarantine result as the profile dictates, release the reservation, and
  neither deadlock nor duplicate the effect.
- **`GNAF-VEC-33`, concurrent checkpoint resume.** Two identical resumptions of
  one immutable `CheckpointRoot` and continuation token MUST produce the same
  deterministic next checkpoint/prepared value. If both attempt publication,
  full-head COMMIT permits at most one and the other duplicates or conflicts;
  no mutable hidden checkpoint index may fork or drop progress.

### 16.24 Normative rejection corpus

| ID | Input condition | Required result |
|---|---|---|
| `GNAF-REJ-01` | Same kind identity, different immutable equivalence or observation | `incoherent`; snapshot does not seal |
| `GNAF-REJ-02` | Rewrite changes required semantic observation | declaration `rejected` or `unadmitted` |
| `GNAF-REJ-03` | Unknown eligibility treated as false or true | nonconforming |
| `GNAF-REJ-04` | Cost-improving cycle with no well-founded resolution | `unsealed` or `optimization-incomplete` |
| `GNAF-REJ-05` | Global claim over current index results without universe coverage | reject global claim |
| `GNAF-REJ-06` | Old certificate reused after machine or primitive extension | reject as snapshot mismatch |
| `GNAF-REJ-07` | Non-extensional operation consumes a quotient type | `unadmitted` or require finer kind |
| `GNAF-REJ-08` | Operational alternative pruned without contextual dominance/reconstruction | frontier incomplete; snapshot cannot make covering claim |
| `GNAF-REJ-09` | Infinite declaration closure truncated by timeout with no cutoff/leastness theorem | `PendingUpdate` or unpublished; no snapshot candidate |
| `GNAF-REJ-10` | Specialized path can refuse and fallback is omitted | realization inadmissible as complete system |
| `GNAF-REJ-11` | Equal bytes across distinct kinds used as equality | reject typed composition |
| `GNAF-REJ-12` | Trailing bytes accepted by a claimed strict interchange parser | reject parser conformance |
| `GNAF-REJ-13` | Bounded accumulator wraps, saturates, or truncates an exact intermediate | reject realization; no state commit |
| `GNAF-REJ-14` | Componentwise minima from different candidates asserted as one vector optimum | reject optimality proof |
| `GNAF-REJ-15` | Finite-prefix operator evidence promoted to unbounded spectral claim | reject analytic claim |
| `GNAF-REJ-16` | Registry, repository, hash, or fixture cited as the sole semantic warrant | reject warrant |
| `GNAF-REJ-17` | Equivalent inputs receive different semantic eligibility | reject operation/realization extensionality |
| `GNAF-REJ-18` | Accepted roots derive both `f` and `not f` with no bound resolution | `incoherent`; dependent seal fails |
| `GNAF-REJ-19` | Cross-universe absorption fails but update is labeled replay-free | reject extension claim |
| `GNAF-REJ-20` | Stable `uor-gnaf/1` wire/address identifier emitted before assignment | reject interchange conformance |
| `GNAF-REJ-21` | Equal-cost identity omitted before applying an identity-sensitive tie policy | argmin/frontier identity coverage fails |
| `GNAF-REJ-22` | Relational/stateful operation supplies only function-output extensionality | reject operation admission |
| `GNAF-REJ-23` | Raw semantic or operational history discarded without sufficient-state/dominance/reconstruction evidence | replay-free/pruning claim fails |
| `GNAF-REJ-24` | Correctness is proved only on an evaluation sample or probability support | realization inadmissible for the exact correctness domain |
| `GNAF-REJ-25` | Per-run outcomes are allowed but the complete relation/law is a strict undeclared subset | reject behavior-conformance claim |
| `GNAF-REJ-26` | A rule-closed symbolic set lacks leastness/derivability evidence | closure remains incomplete; no complete seal |
| `GNAF-REJ-27` | Concurrent update commits against a stale parent/configuration root | transaction `conflict`; no publication |
| `GNAF-REJ-28` | Pointwise query choices are reported as one uniform executable family system | reject family/workload claim |
| `GNAF-REJ-29` | Frontier response omits one distinct nondominated identity | reject `frontier-complete`; at most a separately proved cofinal cover |
| `GNAF-REJ-30` | Unknown or divergent candidate cost causes candidate omission | `optimization-incomplete` or invalid cost profile |
| `GNAF-REJ-31` | Nonconstructive existence proof is used to execute with no witness/extractor | execution prohibited |
| `GNAF-REJ-32` | Partial effect is hidden or retry safety is assumed | reject receipt/execution conformance |
| `GNAF-REJ-33` | Query asks a derived observation but universe is restricted to full-output implementations | reject universe-completeness claim |
| `GNAF-REJ-34` | Restricted-carrier infeasibility/nonattainment is returned for a complete-scope request without its negative omission bridge | exact restricted diagnostic plus requested-scope `Incomplete`; reject full negative claim |
| `GNAF-REJ-35` | Verifier returns `accept(ids)` but the required certificate/seal statement identity is absent | reject proof/scope; no seal, result, or run authorization |
| `GNAF-REJ-36` | Adversarial/stochastic run-input tag, scenario-context compatibility, or law conditioning disagrees with the workload | invalid run input; no execution/effect |
| `GNAF-REJ-37` | Derivation/impact/class proof search exceeds its bound without an exact checkpoint | resource/internal failure; no publication or class claim |
| `GNAF-REJ-38` | Violation emits a receipt but fails to commit quarantine into deployment state | nonconforming; run remains a violation and binding must not execute again |
| `GNAF-REJ-39` | Use-case request changes comparator, quantifier, horizon, asymptotic convention, or units without a new workload identity | invalid request identity/scope |
| `GNAF-REJ-40` | Input-family classifier returns a bare invocation/scenario with no bound query or use-case constructor | reject input-total family/class completeness claim |
| `GNAF-REJ-41` | An arbitrary-input global claim cites separate input-total and family/class certificates without the shared identities, valid-input constructor theorem, uniform solver, and combined resource proof of one `InputTotalCapabilityBinding`, or its custom accepted statement does not bind the exact standard common/scope body and derived certificate ID | reject the arbitrary-input conjunction claim; retain only the independently verified component claims |

Analytic capability profiles additionally use this rejection corpus:

| ID | Input condition | Required result |
|---|---|---|
| `GNAF-AN-REJ-01` | Refinement maps lack identity, composition, or coherence | reject refinement diagram |
| `GNAF-AN-REJ-02` | “Completion” has no constructive Cauchy names/equivalence/limit | reject completion claim |
| `GNAF-AN-REJ-03` | Dense embedding is asserted from finite-dimensional inclusion alone | unresolved density obligation |
| `GNAF-AN-REJ-04` | Unbounded operator omits its exact domain or graph | reject operator profile |
| `GNAF-AN-REJ-05` | Symmetry is promoted to self-adjointness without adjoint-domain equality | reject self-adjoint claim |
| `GNAF-AN-REJ-06` | Pointwise operator convergence is used as graph/resolvent convergence | reject limit transfer |
| `GNAF-AN-REJ-07` | Eigenvalues are reported as the whole spectrum without excluding continuous/residual spectrum | reject spectrum-complete claim |
| `GNAF-AN-REJ-08` | Finite compressions introduce an unexcluded spurious limit point | reject no-pollution/global spectral claim |
| `GNAF-AN-REJ-09` | A true limiting spectral point is not approximated under a claimed complete scheme | reject no-loss/global spectral claim |
| `GNAF-AN-REJ-10` | Zero object is defined by the desired spectrum | reject circular correspondence |
| `GNAF-AN-REJ-11` | Multiplicity is compared across zero/spectrum sides without a bound definition and proof | correspondence incomplete |
| `GNAF-AN-REJ-12` | Numerical tolerance or finitely many samples are treated as exact zero membership | reject exact analytic claim |
| `GNAF-AN-REJ-13` | Empty/proper spectral region is used to certify full-spectrum no-loss/no-pollution | reject full claim; at most restricted-region evidence |

### 16.25 Hardened mutation requirements

Profiles claiming hardened conformance MUST demonstrate non-vacuity by testing
mutations including:

- delete one semantic carry, rewrite, or composition rule;
- replace exact accumulation with wrapping or saturating arithmetic;
- make tie selection depend on worker completion order;
- merge equal bytes across distinct kinds;
- admit a representation-sensitive operation over a quotient;
- treat semantic merge as idempotent without a theorem;
- preserve an old normal spelling after adding a better representation;
- prune the capability-bearing derivation in `GNAF-VEC-07`;
- stop closure before one enabled operation is processed;
- claim execution optimality from representation minimality alone;
- omit selector, verification, fallback, or rebuild costs;
- reuse a certificate after changing `S`, `X`, `M`, or required observation;
- replace a nonempty correctness domain by the empty set and accept a vacuous
  exactness or optimality proof;
- replace a complete outcome law by one allowed deterministic outcome;
- return a non-least fixed point as symbolic closure;
- omit one occurrence identity or collapse multiplicity by semantic value;
- commit a transaction against a stale parent or configuration root;
- reuse a transaction identity with a different request body;
- hide a partial effect or treat a non-idempotent retry as safe;
- replace one uniform workload policy by post-hoc pointwise selectors;
- swap two noncommuting quantifiers in a workload objective;
- drop one nondominated identity from an exact frontier;
- narrow correctness to an evaluation sample or probability support;
- omit a transitive domain or negative-premise dependency from impact analysis;
- replace a constructive dense completion with finite-prefix agreement;
- erase an unbounded operator's domain, graph, or adjoint-domain condition;
- promote symmetry to self-adjointness;
- introduce one spectral-pollution or spectral-loss case; or
- replace a full spectral region by an empty/proper region while retaining the
  global label;
- define the zero object using the spectrum it is meant to characterize.

Each mutation MUST fail at least one named requirement, verifier check, or
fixture expectation.

---

## 17. Security, resource, and integrity requirements

### 17.1 Parser and verifier safety

Implementations MUST bound allocation, recursion, integer width, nesting,
certificate size, proof steps, and dependency traversal before processing
untrusted input. A profile MUST define behavior at every bound.

Strict rejection MUST be atomic: invalid declarations, certificates, or deltas
MUST NOT partially mutate a sealed snapshot.

### 17.2 Closure denial of service

Admission of one declaration may enable unbounded compositions or equivalence
closure. Implementations MUST separate semantic admission from a claim that
closure has completed. Resource exhaustion before the least declaration closure
and quotient are exact yields `PendingUpdate` or no publication. Exhaustion
after those objects exist but before operational seal obligations are proved may
yield `UnsealedSnapshot` or an honest weaker sealed result, never a partial
global claim.

### 17.3 Proof and trust roots

Verifier identity, revision, accepted proof language, trust root, and resource
contract are semantic inputs to admission. Upgrading a verifier or trust root
creates a new evidence context and MAY require a new snapshot.

Verifier failure modes MUST remain separated as in §12.7. Implementations MUST
defend against cyclic imports, dependency confusion, proof bombs, mutable
external evidence, rollback of verifier configuration, and inconsistent trust
roots. Acceptance under one foundation MUST NOT be replayed under another.

### 17.4 State integrity and rollback

Snapshot publication MUST be atomic and compare the exact complete expected
knowledge head and optional runtime-state read set. Parent, transition, transaction, delta,
admission-report, and pending-set identities MUST allow detection of omitted,
reordered, duplicated, conflicting, or replayed updates. Compare-and-swap
failure publishes nothing. A rollback to an older snapshot MUST be visible as
use of that older identity; it MUST NOT masquerade as the latest state.

Runtime effect commits MUST use the atomicity, idempotency, compensation,
cancellation, and retry rules bound by `X` and the deployment configuration.
Optimization certificates authorize only the bound execution; they do not make
an external effect transactional. Receipts MUST expose partial commits and the
exact configuration against which they occurred.

### 17.5 Side channels and hidden state

If side-channel behavior, nondeterministic scheduling, caches, mutable external
services, or hidden advice can affect required observation or cost, they MUST be
inside `X`, prohibited, or bounded by the outcome aggregation. An optimizer MUST
NOT exploit information excluded from competitors.

For online or workload claims, information release and observation timing MUST
follow the bound filtration. Post-hoc access, lookahead, training/test leakage,
or selectively revealed failures define a stronger information model and MUST
NOT be used to certify a policy in the weaker model.

---

## 18. Explicit non-claims

UOR-GNAF does not claim:

- that one stored representation is optimal for every operation, workload,
  machine, resource envelope, or objective;
- that one realization is fastest across hardware, preparation states, or
  accounting boundaries;
- that semantic canonicality implies compression or execution optimality;
- that non-adjacency, irreducibility, sparsity, support size, or term count alone
  implies a global minimum;
- that a representation theorem supplies an execution lower bound without a
  bridge theorem;
- that content addressing, common bytes, names, lengths, or locations establish
  cross-kind semantic equality;
- that registration, discovery, index presence, or provenance alone proves
  eligibility, correctness, trust, feasibility, cost, or optimality;
- that an index miss or missing fact proves ineligibility;
- that a finite snapshot necessarily has tractable closure or optimization;
- that incomplete saturation, search, autotuning, benchmarking, or learned
  selection proves an optimum;
- that an internal plan optimum proves the containing complete system globally
  optimal;
- that a restricted-universe optimum applies to an omitted parent universe;
- that a Pareto-optimal realization is unique, scalar-optimal, or the complete
  frontier;
- that a cofinal dominance cover is the identity-complete Pareto frontier;
- that independent component lower bounds are jointly attainable;
- that a collection of pointwise query optima is one executable uniform
  workload policy;
- that changing quantifier order, information filtration, comparator class, or
  workload horizon preserves an optimum;
- that correctness on a sample, benchmark set, probability support, or
  almost-everywhere evaluation domain establishes correctness on the declared
  exact domain;
- that every observed outcome being allowed establishes the complete required
  nondeterministic relation or probability law;
- that canonical semantic state permits all operational alternatives to be
  discarded;
- that adding facts or realizations preserves the previously selected optimum;
- that an old certificate applies to a later snapshot without a transition
  warrant;
- that later evidence mutates an immutable semantic identity;
- that accumulation must materialize every possible transformation path;
- that every pair of kinds has an admitted transformation;
- that arbitrary code admits automatic decisions of equality, termination,
  refinement, or optimality;
- that universal extensibility quantifies over unspecified future semantics;
- that a claim for one arbitrary admitted instance is input-total or
  use-case-class-complete;
- that a fixed point is the least derivation closure;
- that a nonconstructive existence theorem supplies an executable witness;
- that a successful runtime fallback carries the stronger requested claim;
- that an immutable semantic snapshot is mutable runtime, cache, deployment,
  or transaction state;
- that a sampled trace, hidden partial effect, or stale transaction represents
  the committed execution semantics;
- that finite-dimensional or finite-prefix evidence establishes unbounded
  operator, adjoint, closure, self-adjointness, or spectrum claims;
- that pointwise, weak, or strong convergence alone transfers closedness,
  self-adjointness, resolvents, spectra, or zero correspondences;
- that correctness may be weakened because complete optimization is expensive.

---

## 19. Obligation ledger and freeze status

### 19.1 Definitions fixed by this draft

| Item | Status |
|---|---|
| Typed separation of kind, type, content, syntax, operation, state, trajectory, plan, and address | fixed |
| Proof-carrying semantic quotient | fixed |
| Two-layer canonical semantic state plus operational envelope | fixed |
| Finite-arity contextual improving adjacency | fixed |
| Knowledge accumulation and closure laws | fixed |
| Sealed immutable snapshot boundary | fixed |
| Cross-universe semantic conservativity and absorption | fixed |
| Complete-system admissibility and accounting | fixed |
| Derived problem, invocation scope, complete behavior, and input-total boundary | fixed |
| Scalar argmin, identity-complete Pareto frontier, and total query answer | fixed |
| Joint workload, uniform policy, quantifier, comparator, and use-case-class boundary | fixed |
| Immutable semantic snapshot, operational basis, transaction, deployment, and runtime-state separation | fixed |
| Constructive analytic refinement, completion, unbounded-operator, limit-transfer, and zero/spectrum interfaces | fixed |
| Honest claim and result-status classes | fixed |
| Closed hypotheses, verifier result algebra, lower-bound attainment, and representation bridge | fixed |

### 19.2 Construction-independent theorems discharged here

| Theorem | Status |
|---|---|
| Lower-bound attainment | proved in §11.1 |
| Extensional factorization through a quotient | proved in §11.2 |
| Closure merge law | proved in §11.3 |
| Candidate-extension optimum monotonicity | proved in §11.4 |
| Replay-free semantic update under absorption | proved in §11.5 |
| Contextual-dominance pruning safety | proved in §11.6 |
| Global scalar optimum implies GNAF normality | proved in §11.7 |
| Pareto member implies partial-order GNAF normality | proved in §11.8 |
| Workload lower-bound attainment | proved in §11.9 |

### 19.3 Profile-supplied obligations

Every conforming profile supplies, as applicable:

| Obligation | Required evidence |
|---|---|
| Kind validity and equivalence laws | proof or complete decision procedure |
| Canonical representative | soundness, idempotence, completeness, termination |
| Operation semantics | exactness, extensionality, state/effect/failure laws |
| Composition | typed boundary, intermediate validity, reconstruction, effects |
| Accumulation algebra | monoid/order laws, homomorphism, absorption, capacity |
| Derivation closure | soundness, monotonicity, leastness/completeness, fixed-point theorem, and liveness/checkpoint semantics |
| Operational basis | retained-set coverage, contextual dominance or exact reconstruction, and garbage-collection safety |
| Conservative extension | semantic embedding and cross-universe absorption |
| Transactional maintenance | admission totality, parent/configuration linearization, idempotent replay, dependency impact, and recertification |
| Complete behavior | nonempty exact execution relation/law, outcome completeness, productive completion, and effect/failure semantics |
| Global scalar optimum | complete admission, attainment, and exhaustive/proof-complete/no-improver coverage or a universal lower bound attained |
| Pareto result | exact nondominated-set equality or explicitly weaker cofinal dominance coverage |
| Workload/family optimum | one uniform policy, filtration, scenario semantics, whole-workload functional, quantifier prefix, comparator, and workload lower bound |
| Input/use-case-class totality | one uniform terminating exact classifier/solver over the entire bound nonempty class |
| Analytic kind/operation | coherent refinement, constructive completion/density, exact operator domain/graph/adjoint, property-specific limit transfer, and noncircular no-loss/no-pollution zero/spectrum laws |
| Proof closure | explicit foundation, axiom/evidence imports, empty undischarged set, constructive witness/extractor where execution is claimed |
| Wire/address profile | canonical grammar, accepted image, vectors, rejection corpus |
| Hardened implementation | realization, mutation, adversarial, and differential evidence |

### 19.4 Open items before stable `uor-gnaf/1`

The following are deliberately not assigned by Draft 0.2:

- universal canonical wire grammars and media types;
- object-specific UOR address and multicodec assignments;
- canonical certificate and receipt byte schemas;
- a mandatory proof language or verifier foundation;
- a mandatory machine, cost, or operation universe;
- implementation-specific incremental algorithms;
- a formal mechanization of every construction-independent theorem and profile
  interface in a named proof assistant;
- governance and compatibility rules for assigning the stable identifier.

An implementation MUST NOT fill these gaps privately and present the result as
the stable base specification.

### 19.5 Freeze rule

The stable identifier `uor-gnaf/1` MUST NOT be assigned until:

1. all objects that cross an interchange boundary have canonical grammars;
2. strict parsers, accepted-image laws, normative byte vectors, and rejection
   corpora exist for those grammars;
3. requirement IDs and outcome semantics are independently reviewed;
4. at least one nontrivial kind, operation, extension, scalar optimum, Pareto,
   workload, input-total, transactional maintenance, and constructive analytic
   profile passes the applicable normative fixtures;
5. the construction-independent core is mechanized or independently proved to
   the publication standard adopted by UOR Foundation;
6. no open issue changes the meaning of semantic quotient, generalized
   adjacency, closure, seal, extension, admission, problem, complete behavior,
   workload, use-case class, transaction, analytic transfer, or global
   optimality.

---

## 20. Dependency boundary

The mathematical core has no normative serialization, addressing, repository,
registry, package-ecosystem, or implementation dependency. An interchange,
addressing, proof-verifier, or execution capability depends only on the exact
profiles named by that capability's dependency manifest.

Informatively, UOR-NAF is a motivating specialization: its signed-radix normal form and scoped
machine lower-bound bridge may instantiate UOR-GNAF under an exact declared
profile. UOR-NAF's digit algebra, rank-one contraction grammar, and machine
restrictions are not axioms of UOR-GNAF.

Any implementation or companion specification claiming to instantiate
UOR-GNAF MUST publish a dependency manifest identifying:

- the exact revision of this draft or stable successor;
- every kind, operation, machine, cost, proof, interchange, and address profile;
- whether each dependency is semantic, verification-only, realization-only, or
  measured;
- every restriction of the admitted universe;
- the strongest honest claim class actually supported.

A profile MUST NOT silently import candidate sets, cost assumptions, equality
rules, or proof obligations from a fixture, repository, or implementation.

---

## 21. Normative references

- RFC 2119, *Key words for use in RFCs to Indicate Requirement Levels*.
- RFC 8174, *Ambiguity of Uppercase vs Lowercase in RFC 2119 Key Words*.

These references define BCP 14 requirement language only. They do not supply
UOR-GNAF semantics, kinds, operations, candidate universes, or proof warrants.

---

## Appendix A. Compact abstract machine

The abstract machine uses an append-only immutable `ObjectStore` for decoded
declarations, checkpoints, candidates, seals, result certificates, deployment
versions, evidence, and receipts. Its only mutable serialized coordination
object is

```text
CoordinatorState = (
  CoordinatorStateVersion : NaturalNumber,
  KnowledgeHead,
  DeploymentHeadMap : DeploymentLineageId -> DeploymentHeadState,
  TransactionLedgerMap : TransactionKey partial-> TransactionLedgerEntry,
  RecoveryInvocationEpochMap : DeploymentLineageId -> NaturalNumber,
  ExecutionCapabilityIssuerStateMap : TrustedIssuerProfileId ->
    (TrustedIssuerEpoch,NextInvocationSerial),
  CoordinatorTransactionIssuerStateMap : TrustedIssuerProfileId ->
    (TrustedIssuerEpoch,NextAttemptSerial),
  CoordinatorPermitStateMap : CoordinatorPermitStateKey ->
    unspent | consumed(CoordinatorCommitId)
).

`CoordinatorObservationVersion` is the exact `CoordinatorStateVersion` read in
the same atomic observation. Every coordinator or issuer-state transition
increments it exactly once and no other operation chooses it. Observation
objects therefore contain a state-derived monotone value, not an ambient clock
or nonce.

The two issuer maps are the serialized source of every fresh execution grant and
coordinator update/reseal/activation publication capability. Allocation
atomically advances the applicable serial, returns the exact prior/successor
issuer-state receipt, and changes no knowledge, deployment, effect, or
transaction-ledger field. Issuer epoch rotation is itself an atomic transition
that carries the last serial and cannot reuse an earlier `(epoch,serial)` pair.
No ambient process-local counter or random nonce is a freshness proof.

AcquireRecoveryInvocationAttempt(CoordinatorState,lineage,recoveryTx)
  atomically reads epoch `n`, writes `n+1`, and returns
  `RecoveryInvocationAttemptId = Identity(recovery-invocation-attempt-domain,
  recoveryTx,lineage,n+1)` plus the exact coordinator transition receipt. It is
  the sole constructor of recovery attempt identities. Monotonic allocation and
  the identity domain prove collision freedom; a caller cannot choose, reuse, or
  forge an epoch. This coordination-only advance changes no knowledge,
  deployment, effect, or transaction-ledger head.

DeploymentHeadState =
    absent
  | available(DeploymentConfiguration)
  | reserved(ExecutionReservation).

ExecutionReservation = (
  ExecutionTransactionId,
  OriginalTransactionKey,
  ExecutionRequestBodyRoot,
  ExactExecutionRequestBodyOrResolvableObjectId,
  ExpectedDeploymentConfiguration,
  ReservationPhase,
  ReservationStateRoot,
  EffectIntentLedgerRoot,
  EffectIntentLedgerObjectId,
  EffectIntentLedgerRetentionWarrantId,
  EffectIntentLedgerRetentionWarrantObjectId,
  RecoveryWorkRoot?,
  RecoveryWorkObjectId?,
  RecoveryWorkObjectGraphRetentionWarrantId?,
  RecoveryWorkObjectGraphRetentionWarrantObjectId?,
  RecoveryResumeAcquisitionScheduleRoot,
  RecoveryResumeAcquisitionMarker?,
  RecoveryInvocationLease?,
  LatestAcceptedRecoveryTakeoverEvidenceRef?,
  RecoveryLeaseEpochCounter,
  RecoveryFaultCount,
  RecoveryStageAttemptMarker?,
  RecoveryTailAttemptScheduleRoot,
  RecoveryTailAttemptMarker?,
  ExecutionResourcePartitionRoot,
  ExecutionPartitionOwnershipState,
  RecoveryPolicyId,RecoveryPolicyCoreId,
  RecoveryResourceContractId,
  RecoveryExecutionMaterialRoot,
  RecoveryBundleRoot,
  RecoveryBundleObjectId,
  EmergencySafeQuarantineTemplateId,
  EmergencySafeQuarantineTemplateRoot,
  EmergencySafeQuarantineTemplateRetentionWarrantId,
  EmergencySafeQuarantineTemplateRetentionWarrantObjectId,
  RecoveryBundleRetentionWarrantId,
  RecoveryBundleRetentionWarrantObjectId
).

ReservationStateBody = (
  ReservationPhase, EffectIntentLedgerRoot, EffectIntentLedgerObjectId,
  EffectIntentLedgerRetentionWarrantId,
  EffectIntentLedgerRetentionWarrantObjectId,
  RecoveryWorkRoot?, RecoveryWorkObjectId?,
  RecoveryWorkObjectGraphRetentionWarrantId?,
  RecoveryWorkObjectGraphRetentionWarrantObjectId?,
  RecoveryResumeAcquisitionScheduleRoot,
  RecoveryResumeAcquisitionMarker?, RecoveryInvocationLease?,
  LatestAcceptedRecoveryTakeoverEvidenceRef?,
  RecoveryLeaseEpochCounter, RecoveryFaultCount,
  RecoveryStageAttemptMarker?,
  RecoveryTailAttemptScheduleRoot, RecoveryTailAttemptMarker?,
  ExecutionPartitionOwnershipState
).

RecoveryResumeAcquisitionMarker = (
  RecoveryTransactionId, RecoveryWorkRoot,
  RecoveryInvocationAttemptId, RecoveryLeaseEpoch,
  ConsumedResumeTokenId, PostConsumptionScheduleRoot
).

RecoveryInvocationAttemptId = Identity(
  recovery-invocation-attempt-domain,
  RecoveryTransactionId, DeploymentLineageId, RecoveryInvocationEpoch).

RecoveryInvocationLease = (
  RecoveryInvocationAttemptId, RecoveryLeaseEpoch,
  RecoveryTransactionId, RecoveryOriginRoot, RecoveryWorkRoot,
  LeaseExpiryOrExplicitQuiescenceRule,
  SinkAndCoordinatorFenceToken: RecoveryAttemptFenceToken,
  AcceptedRecoveryTakeoverEvidenceRef?
).

RecoveryTakeoverStatement = (
  RecoveryTransactionId, RecoveryOriginRoot,
  PriorRecoveringReservationRoot, PriorRecoveryInvocationLease?,
  NewRecoveryInvocationAttemptId, NewRecoveryLeaseEpoch,
  PriorRecoveryFaultCount, NewRecoveryFaultCount,
  ConsumedResumeAcquisitionTokenId,
  PreConsumptionResumeScheduleRoot, PostConsumptionResumeScheduleRoot,
  AbandonedRecoveryStageAttemptMarker?, AbandonedRecoveryTailAttemptMarker?,
  PriorEffectIntentLedgerRoot,
  CompleteNonterminalPriorAttemptFenceTokenSetRoot,
  PriorOwnerSinkAndCoordinatorQuiescenceEvidenceRoot
).

RecoveryTakeoverStatementId = Identity(
  recovery-takeover-statement-domain,RecoveryTakeoverStatement).

RecoveryTakeoverEvidenceObject = (
  RecoveryTakeoverStatement,RecoveryTakeoverStatementId,
  ProofId,VerifierResult=accept(VerifiedStatementIds containing
                               RecoveryTakeoverStatementId)
).

RecoveryTakeoverEvidenceObjectRoot = Identity(
  recovery-takeover-evidence-domain,RecoveryTakeoverEvidenceObject).

RecoveryTakeoverEvidenceObjectId = Identity(
  recovery-takeover-evidence-object-domain,
  RecoveryTakeoverEvidenceObjectRoot).

RecoveryTakeoverEvidenceRetentionStatement = (
  RecoveryTakeoverEvidenceObjectRoot,RecoveryTakeoverEvidenceObjectId,
  RequiredRecoveringReservationLifetime,DurableFaultDomain,
  ImmutableObjectLocatorAndReplicationProfileId
).

RecoveryTakeoverEvidenceRetentionStatementId = Identity(
  recovery-takeover-evidence-retention-domain,
  RecoveryTakeoverEvidenceRetentionStatement).

RecoveryTakeoverEvidenceRetentionWarrant = (
  RecoveryTakeoverEvidenceRetentionStatement,
  RecoveryTakeoverEvidenceRetentionStatementId,
  ProofId,
  VerifierResult=accept(VerifiedStatementIds containing
                        RecoveryTakeoverEvidenceRetentionStatementId)
).

RecoveryTakeoverEvidenceRetentionWarrantId = Identity(
  recovery-takeover-evidence-retention-warrant-domain,
  RecoveryTakeoverEvidenceRetentionWarrant).

RecoveryTakeoverEvidenceRetentionWarrantObjectId is the typed immutable-object
identifier resolving that warrant.

AcceptedRecoveryTakeoverEvidenceRef = (
  RecoveryTakeoverStatementId,RecoveryTakeoverEvidenceObjectId,
  RecoveryTakeoverEvidenceRetentionWarrantId,
  RecoveryTakeoverEvidenceRetentionWarrantObjectId
).

RecoveryTakeoverEvidenceResult =
    accepted(RecoveryTakeoverEvidenceObject,
             AcceptedRecoveryTakeoverEvidenceRef)
  | rejected(Reason)
  | incoherent(ConflictProof)
  | conflict(CurrentDeploymentHeadState)
  | unresolved(DependencyIds)
  | unsupported(FeatureIds)
  | resource-exhausted(ResourceReport)
  | internal-failure(FailureReport).

`BoundedProveRecoveryLeaseTakeover` returns exactly this algebra. `accepted` is
legal only when the evidence object's accept set contains the displayed exact
statement ID and every field equals the derived token/schedule roots, fault counter,
markers, attempt, epoch, origin, and current reservation. A generic verifier
`accept` that omits that ID is lifted to
`rejected(accepted-wrong-takeover-statement)`; no nonaccepted constructor may
enter the consume-and-lease CAS. Under `TakeoverProofSlice`, the accepted branch
only constructs and verifies the canonical evidence object plus the proposed
content-addressed reference and retention-warrant draft; it performs no object
store or coordinator mutation. `TakeoverCandidateAndCasAttemptSlice` alone
stores that exact object/warrant, equality-checks the proposed reference, builds
the descendant, and commits `AcceptedRecoveryTakeoverEvidenceRef` with the lease
CAS. A losing CAS may leave only harmless immutable content-addressed objects;
it exposes no live lease/token, and a naked statement digest is not crash-
verifiable evidence.

When `PriorRecoveryInvocationLease=none` because a checkpoint released it, the
helper MUST resolve the exact retained `PriorEffectIntentLedgerRoot`, enumerate
every nonterminal intent/permit's prior attempt-fence token, and prove each token
quiesced, rejected, or deduplication/in-flight-classified before accepting a new
lease. The complete token-set root and evidence root are statement fields. A
missing lease is never interpreted as proof that no old sink action exists.

RecoveryAttemptFenceToken = Identity(
  recovery-attempt-fence-domain,
  RecoveryFenceToken, RecoveryInvocationAttemptId, RecoveryLeaseEpoch).

NextRecoveryLeaseEpoch(RecoveryLeaseEpochCounter=e) = e + 1.

`SinkAndCoordinatorFenceToken` in a recovery lease MUST equal this attempt token.
Every recovery authorization statement, intent, permit, sink membership check,
status CAS, checkpoint, and terminal CAS binds it. A takeover creates a new token;
the accepted takeover statement proves the sink rejects the prior attempt token
before any new recovery effect is authorized.

RecoveryTailAttemptMarker = (
  RecoveryTransactionId, RecoveryInvocationAttemptId, RecoveryLeaseEpoch,
  RecoveryWorkRoot, TailAttemptTokenId, TailAttemptBundleRoot,
  PostConsumptionTailAttemptScheduleRoot
).

RecoveryStageAttemptMarker = (
  RecoveryTransactionId, RecoveryInvocationAttemptId, RecoveryLeaseEpoch,
  RecoveryWorkRoot, RecoverySubstage, RecoveryProgressOrdinal,
  RecoveryProgressStateRoot, StageAttemptTokenId,
  RecoveryStageAttemptBundleRoot,
  PostConsumptionStageAttemptScheduleRoot
).

ReservationStateRoot
  = Identity(execution-reservation-state-domain, ReservationStateBody).

For every published reservation `r`, its `ReservationStateRoot` MUST equal the
displayed identity of exactly `r.ReservationPhase`,
`r.EffectIntentLedgerRoot`, its exact ledger object and retention-warrant
identities, `r.RecoveryWorkRoot?`, and
`r.RecoveryWorkObjectId?`, both exact recovery-work graph retention-warrant
references, `r.RecoveryResumeAcquisitionScheduleRoot`, and
`r.RecoveryLeaseEpochCounter`, `r.RecoveryFaultCount`, and
`r.RecoveryTailAttemptScheduleRoot`, plus the exact
`r.ExecutionPartitionOwnershipState`.
The optional `RecoveryResumeAcquisitionMarker` and `RecoveryInvocationLease` are
included in that same state identity, as is the exact optional
`LatestAcceptedRecoveryTakeoverEvidenceRef`; `RecoveryStageAttemptMarker?` and
`RecoveryTailAttemptMarker?` are included as well.
Every constructor and
every running/recovering, intent/status, checkpoint, or recovery-work CAS MUST
recompute this root; a stale root makes the descendant invalid and the CAS must
reject it.

`LatestAcceptedRecoveryTakeoverEvidenceRef` is `none` before the first
takeover. A takeover CAS replaces it with the newly accepted retained evidence
reference, and the active lease MUST carry that same reference. Stage work,
effect, advance, checkpoint, and lease-release descendants preserve this
reservation-level history field even when `RecoveryInvocationLease` becomes
`none`; a later takeover may replace it only with evidence whose statement
names the prior reservation and higher epoch. Thus checkpointing never erases
the crash-verifiable takeover chain.

`RecoveryLeaseEpochCounter` and `RecoveryFaultCount` are natural numbers. The
lease counter is initialized to zero in the running reservation and increases by
exactly one whenever a recovery invocation lease is installed; releasing a
lease never resets it. `RecoveryFaultCount` is initialized to zero and increases
by exactly one in the same takeover CAS whenever a prior owner is proved
abandoned after the fresh recovery fence or after acquiring a resume, stage, or
tail token/bundle. It never decreases. All schedule exhaustion and liveness
checks read this serialized field from the exact current reservation.

All five recovery-schedule root fields in `RecoveryWorkBody` or
`ReservationStateBody` are typed content-addressed state-object identifiers as
defined in §7.11. Every schedule-consuming reservation/work CAS atomically stores
the exact successor schedule body before publishing its root. Recovery resumes
by resolving and checking those bodies under the admitted
`RecoveryScheduleStatePersistenceWarrant`; retaining only a digest is invalid.

Only the invocation attempt named by the current nonexpired
`RecoveryInvocationLease` may consume a recovery stage, effect, checkpoint, or
terminal-publication token. A competing caller either observes the active lease
and returns `conflict`, or, after the exact expiry/quiescence rule is proved,
atomically installs a higher lease epoch with an accepted statement proving the
old owner can perform no further sink/coordinator action. Every recovery-visible
sink operation and coordinator CAS checks the attempt ID, epoch, and fence.
Checkpoint publication releases the lease; crash takeover is therefore explicit
and deterministic. A resume marker is reusable only by its named attempt and
lease epoch, never by another same-body caller.

`RecoveryExecutionMaterialRoot`, `EmergencySafeQuarantineTemplateId/Root`, and
`RecoveryBundleRoot` MUST equal the corresponding fields in the exact
`RecoveryBundleBody` in §7.11; `RecoveryBundleObjectId` MUST resolve that body,
and `RecoveryBundleRetentionWarrantId/ObjectId` MUST resolve the exact warrant
whose accepted statement verifies its declared durable fault domain.
`EffectIntentLedgerRoot` commits an ordered typed collection of
`EffectIntentEntry` values and the corresponding consumed/rejected
`EffectPermit` evidence. `ExecutionResourcePartitionRoot` commits the exact
partition and sufficiency warrant used before reservation.

ReservationPhase =
    running(ExecutionFenceToken, LeaseOrQuiescenceWarrant)
  | recovering(RecoveryTransactionId, RecoveryFenceToken, RecoveryOrigin).

RecoveryOrigin = (
  RecoveryRequestBodyRoot,
  ExactRecoveryRequestBodyOrResolvableObjectId,
  OriginalExpectedReservedHeadRef,
  OriginalExecutionReservationObjectId,
  RecoveryOriginObjectGraphRoot,
  RecoveryOriginObjectGraphObjectId,
  RecoveryOriginObjectGraphRetentionWarrantId,
  RecoveryOriginObjectGraphRetentionWarrantObjectId
).

RecoveryOriginObjectGraph = (
  ExactRecoveryRequestBody, OriginalExecutionReservation
).

RecoveryOriginObjectGraphRoot = Identity(
  recovery-origin-object-graph-domain,RecoveryOriginObjectGraph).

RecoveryOriginObjectGraphObjectId is the typed immutable identifier resolving
that exact graph. `OriginalExecutionReservationObjectId` and
`ExactRecoveryRequestBodyOrResolvableObjectId` MUST resolve the graph's exact
two projections and rederive their displayed roots.

RecoveryOriginObjectGraphRetentionStatement = (
  RecoveryOriginObjectGraphRoot,RecoveryOriginObjectGraphObjectId,
  ExactRecoveryRequestBodyOrResolvableObjectId,
  OriginalExecutionReservationObjectId,
  RequiredRecoveringReservationLifetime,DurableFaultDomain,
  ImmutableObjectLocatorAndReplicationProfileId
).

RecoveryOriginObjectGraphRetentionStatementId = Identity(
  recovery-origin-object-graph-retention-domain,
  RecoveryOriginObjectGraphRetentionStatement).

RecoveryOriginObjectGraphRetentionWarrant = (
  RecoveryOriginObjectGraphRetentionStatement,
  RecoveryOriginObjectGraphRetentionStatementId,
  ProofId,VerifierResult=accept(VerifiedStatementIds containing
                                RecoveryOriginObjectGraphRetentionStatementId)
).

RecoveryOriginObjectGraphRetentionWarrantId = Identity(
  recovery-origin-object-graph-retention-warrant-domain,
  RecoveryOriginObjectGraphRetentionWarrant).

RecoveryOriginObjectGraphRetentionWarrantObjectId is the typed immutable object
identifier resolving that exact warrant. The graph and accepted warrant are
created and durably stored in the same atomic running-to-recovering fence
transition. They retain both the recovery request body and exact running
reservation for the complete recovering-reservation lifetime; a descendant
that cannot strictly resolve and rederive either projection is invalid and may
not resume.

ResolvedRecoveryOriginObjectGraph = (
  RecoveryOriginObjectGraph,RecoveryOriginObjectGraphRoot,
  RecoveryOriginObjectGraphObjectId,
  RecoveryOriginObjectGraphRetentionWarrant,
  RecoveryOriginObjectGraphRetentionWarrantId,
  RecoveryOriginObjectGraphRetentionWarrantObjectId
).

This dependent carrier is present in coordinator ingress exactly for an
already-recovering head and absent for a running head. Every root/object/warrant
field MUST equal the corresponding `RecoveryOrigin` projection, both graph
members MUST rederive the separately committed request/reservation identities,
and the warrant's accept set MUST contain its exact retention statement ID.

TransactionKey =
    update(TransactionId)
  | reseal(ResealTransactionId)
  | activation(ActivationTransactionId)
  | execution(ExecutionTransactionId)
  | continuation-step(ContinuationStepTransactionId)
  | recovery(RecoveryTransactionId).

TransactionLedgerEntry = (
  TransactionKind,
  RequestBodyRoot,
  ExactRequestBodyOrResolvableImmutableObjectId,
  ResultTypeTag,
  ResultId,
  ImmutableNonduplicateResult,
  ReceiptId?
).

TransactionLedgerEntryId
  = Identity(transaction-ledger-entry-domain,TransactionLedgerEntry).

BuildTransactionLedgerEntry(key,exactRequestBody,nonduplicateResult,
                            exactReceiptBody?)
  = TransactionLedgerEntry(
      TransactionKindFrom(key),
      Identity(TransactionRequestBodyDomainFrom(key),exactRequestBody),
      StoreImmutableExactBody(exactRequestBody),
      ExactResultTypeTag(nonduplicateResult),
      Identity(ResultIdentityDomainFrom(key),nonduplicateResult),
      StoreImmutableNonduplicateResult(nonduplicateResult),
      none when exactReceiptBody is absent, otherwise
        Identity(ReceiptIdentityDomainFrom(
                   ExactResultTypeTag(nonduplicateResult),exactReceiptBody),
                 exactReceiptBody)).

When a receipt is present, the builder also stores the exact immutable receipt
body under that derived identity in the same atomic object/ledger transition.
For `execution` or `continuation-step`, `nonduplicateResult` MUST be a
`LedgerRecordableExecutionResult`; ephemeral `recovery-required`,
`integrity-failure`, or `duplicate` control-flow results have no ledger entry.
For `recovery`, it MUST be a `FinalizedRecoveryResult`. These are dependent
input constraints of the builder, not post-hoc validation conventions.

The domain projections are closed typed maps, not implementation choices:

key kind          TransactionRequestBodyDomainFrom  ResultIdentityDomainFrom
update            update-request-body-domain        update-result-domain
reseal            reseal-request-body-domain        reseal-result-domain
activation        activation-request-body-domain    activation-result-domain
execution         execution-request-body-domain     execution-result-domain
continuation-step execution-request-body-domain     execution-result-domain
recovery          recovery-request-body-domain      recovery-result-domain

result receipt tag             ReceiptIdentityDomainFrom
execution terminal             execution-receipt-domain
productive prefix              prefix-receipt-domain
declared/inconclusive partial   partial-receipt-domain
violation/quarantine            quarantine-receipt-domain
update                          update-receipt-domain
reseal                          reseal-receipt-domain
activation                      activation-receipt-domain
recovery                        recovery-receipt-domain
no receipt                      absent

`TransactionKindFrom` is the corresponding first-column tag. A mismatched key,
result tag, or receipt body is rejected before the atomic write.

Every later phrase that appends or publishes a transaction result is shorthand
only for this complete builder; no shorter tuple is a conforming ledger entry.

ValidateExecutionReplayObservation(key,exactRequestBody,entry?,head) =
    coherent-duplicate(entry)
      when entry has the identical body and `head` is not a reservation whose
      OriginalTransactionKey is `key`
  | identity-reuse(entry)
      when entry exists with a different body
  | atomicity-integrity-failure(entry,head)
      when an entry exists and `head` is still a reservation whose
      OriginalTransactionKey is `key`
  | absent(entry,head)
      when no entry exists.

Every execution-path phrase below that returns `duplicate` MUST first obtain
`coherent-duplicate` from this exact validator over one coordinator observation.
`atomicity-integrity-failure` returns the declared `integrity-failure` branch and
never a duplicate; `identity-reuse` returns the exact invalid/no-run branch.
```

`RequestBodyRoot`, `ResultId`, and any `ReceiptId` are recomputed under their
typed identity domains before lookup or publication. The referenced exact body,
result, and receipt are immutable and resolvable from `ObjectStore`. The whole
entry is written in the same coordinator transition as its head change. Thus a
replay can distinguish an identical body from identifier reuse and can return
the exact original result without consulting an ambient cache.

An update, reseal, activation, or execution consumes explicitly expected head
values and atomically returns a successor `CoordinatorState` plus immutable
objects/receipts. A `PendingUpdate` is immutable, stored in `ObjectStore`, and
returned to its caller; it is not a hidden mutable coordinator slot. No
procedure relies on an ambient mutable head. External
procedures are total under their bound resource contracts and return tagged
results:

```text
PreparedUpdate =
    PreparedSealed(
      UpdateRequestBody, RequestIdentity,
      SnapshotCandidate, AdmissionReport, SealCertificate, SealId,
      PrepareProof)
  | PreparedUnsealed(
      UpdateRequestBody, RequestIdentity,
      SnapshotCandidate, AdmissionReport, ExactIncompleteSealStatus,
      PrepareProof)

ExactIncompleteSealStatus = (
  IncompleteSealStatusId,
  SnapshotId,
  RequestedSealObligationRoot,
  OutstandingObligationIds,
  PostIdEvidenceRoot,
  SealVerifierContextId,
  ExactSealResultId
)

IncompleteSealStatusId = Identity(
  incomplete-seal-status-domain,
  ExactIncompleteSealStatus excluding IncompleteSealStatusId).

UpdateResult =
    committed-sealed(SealedSnapshot, UpdateReceipt)
  | committed-unsealed(UnsealedSnapshot, UpdateReceipt)
  | pending(PendingUpdate, ObligationIds)
  | rejected(UpdateRejectionReport)
  | incoherent(ConflictProof)
  | unresolved(DependencyIds)
  | conflict(UpdateConflictReport)
  | unsupported(FeatureIds)
  | resource-exhausted(ResourceReport)
  | internal-failure(FailureReport)

UpdateRejectionReport =
    admission(AdmissionReport)
  | malformed-transaction-identity(details)
  | delta-identity-mismatch(expected,actual)
  | delta-input-descriptor-mismatch
  | corrupt-checkpoint(details)
  | corrupt-prepared(details)
  | other-exact-update-rejection(kind,details)

ConflictKind =
    seal-conflict
  | stale-commit-read-set
  | profile-defined-update-conflict(ProfileId,ConflictTag).

UpdateConflictReport = (
  ConflictKind,
  ExpectedKnowledgeAndRuntimeState?,
  ActualKnowledgeAndRuntimeState?,
  TransactionKey?, Details
)

UpdateCommitPublicationResult =
    published(UpdateResult)
  | recorded-conflict(UpdateResult)
  | same-body-winner(original nonduplicate UpdateResult)
  | identity-reuse(UpdateRejectionReport)
  | warrant-violation(FailureReport).

`BoundedAtomicPublishOrRecordUpdate` returns exactly this algebra and consumes
one coordinator-publication token. In one coordinator transaction it rechecks
the update key and the complete expected knowledge/runtime read set. If the key
is absent and the read set matches, it publishes the prepared candidate and its
success entry. If the key is absent and the read set is stale, it leaves all
heads unchanged and appends the exact immutable `conflict` result with no
receipt. A same-body winner returns the original immutable result; a different-body winner returns
`identity-reuse`. Thus no stale-read-set result is transient or capable of later
publishing under the same transaction identity.

ResolutionResult =
    resolved-query(RequestStatus=valid,
                   answer in ValidQueryAnswers_S(q),
                   EvidenceForQueryAnswer(q,answer))
  | resolved-use-case(RequestStatus=valid,
                      answer in ValidAnswers_S(u),
                      EvidenceForUseCaseAnswer(u,answer))
  | request-result(RequestStatus, Diagnostics)
  | verifier-result(VerifierResult)
  | resource-exhausted(ResourceReport)
  | internal-failure(FailureReport)

NoRunReport = (
  Reason,
  OptimizationStatus,
  ActualClaimClass?,
  NoEffectProof
)

NoEffectProof = (
  ExecutionRequestBodyRoot,
  ExpectedDeploymentHeadState,
  ObservedDeploymentHeadState,
  PreReservationActionTraceRoot,
  ExactProofNoEffectIntentOrExternalEffectWasEmitted
)

ExactExecutionKeyHeadObservationBody = (
  TransactionKey, TransactionLedgerEntry?,
  DeploymentLineageId, DeploymentHeadState,
  CoordinatorObservationVersion
).

ExactExecutionKeyHeadObservationId = Identity(
  execution-key-head-observation-domain,
  ExactExecutionKeyHeadObservationBody).

ExactExecutionKeyHeadObservation = (
  ExactExecutionKeyHeadObservationBody,ExactExecutionKeyHeadObservationId
).

ExecutionIntegrityReportBody =
    atomicity-violation(ExpectedTransactionKey,ExpectedRequestBodyRoot,
                        ExactExecutionKeyHeadObservation)
  | reservation-cas-invariant-violation(ExpectedReservationRoot,
                                        ExactExecutionKeyHeadObservation)
  | intent-cas-invariant-violation(IntentId?,ExpectedReservationRoot,
                                   ExactExecutionKeyHeadObservation)
  | status-cas-invariant-violation(IntentId,ExpectedReservationRoot,
                                   ExactExecutionKeyHeadObservation)
  | finalization-warrant-violation(ExpectedReservationRoot,FailureReport).

ExecutionIntegrityReportId = Identity(
  execution-integrity-report-domain,ExecutionIntegrityReportBody).

ExecutionIntegrityReport = (
  ExecutionIntegrityReportBody,ExecutionIntegrityReportId
).

BuildExecutionIntegrityReport(body) =
  (body,Identity(execution-integrity-report-domain,body)).

`ExecutionIntegrityReport(body)` in the abstract machines is shorthand only for
this exact builder; the bare carrier name or an arity-varying tuple is invalid.

ExecutionFinalizeResult =
    committed(FinalizedExecutionResult)
  | same-body-winner(OriginalExecutionResultId,
                     FinalizedExecutionResult)
  | recovery-owned(ExecutionReservationId,RecoveryRequestTemplate)
  | integrity-failure(ExecutionIntegrityReport,
                      ObservedEffectLedgerState,CurrentDeploymentHeadState)
  | warrant-violation(FailureReport,ExactExecutionKeyHeadObservation).

ExecutionReservationMutationLoss =
    same-body-winner(OriginalExecutionResultId,
                     LedgerRecordableExecutionResult)
  | recovery-owned(ExecutionReservationId,RecoveryRequestTemplate)
  | identity-reuse(CurrentDeploymentHeadState)
  | ordinary-head-conflict(CurrentDeploymentHeadState)
  | integrity-failure(ExecutionIntegrityReport,
                      ObservedEffectLedgerState,CurrentDeploymentHeadState).

ExecutionReservationMutationResult =
    committed(ExecutionReservation)
  | classified-loss(ExecutionReservationMutationLoss,
                    LiveUnreservedPartitionDispositionCapability)
  | warrant-violation(FailureReport,ExactExecutionKeyHeadObservation,
                      LiveUnreservedPartitionDispositionCapability).

PostReservationMutationLoss =
    same-body-winner(OriginalExecutionResultId,
                     FinalizedExecutionResult)
  | recovery-owned(ExecutionReservationId,RecoveryRequestTemplate)
  | integrity-failure(ExecutionIntegrityReport,
                      ObservedEffectLedgerState,CurrentDeploymentHeadState).

PostReservationMutationResult =
    committed(ExecutionReservation)
  | classified-loss(PostReservationMutationLoss)
  | warrant-violation(FailureReport,ExactExecutionKeyHeadObservation).

Every execution reservation, intent, status, or finalization helper makes one
atomic `ExactExecutionKeyHeadObservation` on loss and, within the helper's
reserved mutually exclusive loss-validation budget, strictly validates the
winning entry/reservation and returns the corresponding classified constructor.
Only an integrity constructor exposes its report/ledger/head projection; a bare
report, separately reread entry, free “actual head,” or caller-side winner fetch
is not a value. Finalization construction, CAS, and losing-winner validation are
one single-use bounded operation.

BuildNoEffectProof(Request,expected,observed,trace)
  MUST construct this tuple under the bound `ExecutionIngressProfile` bootstrap
  envelope or the later bounded execution-input contract; a bare type name is
  not a witness.

ObservedEffectLedgerState =
    matching-reservation(
      EffectIntentLedgerRoot,EffectIntentLedgerObjectId,
      EffectIntentLedgerRetentionWarrantId,
      EffectIntentLedgerRetentionWarrantObjectId)
  | attempted-reservation(EffectIntentLedgerRoot).

`matching-reservation` is constructed only by projecting all four references
from the reservation contained in the same exact coordinator observation.
`attempted-reservation` records the attempted reservation's ledger root when
the observation contains no matching reservation. A partial
`EffectIntentLedgerRootFrom(head)` function is not a value of this type.

ObservedEffectLedgerStateFrom(observation,attemptedReservation) =
    matching-reservation(
      r.EffectIntentLedgerRoot,r.EffectIntentLedgerObjectId,
      r.EffectIntentLedgerRetentionWarrantId,
      r.EffectIntentLedgerRetentionWarrantObjectId)
      when observation.DeploymentHeadState=reserved(r) and
           r.OriginalTransactionKey=observation.TransactionKey
  | attempted-reservation(attemptedReservation.EffectIntentLedgerRoot)
      otherwise.

This projection is total and uses only the one exact observation; callers never
guess which tagged branch applies.

FinalizedExecutionResult =
    executed(RuntimeStatus,OptimizationStatus,ActualClaimClass,
             new DeploymentConfiguration,Receipt)
  | partial(RuntimeStatus,OptimizationStatus,ActualClaimClass?,
            new DeploymentConfiguration,PartialReceipt)
  | productive(productive,OptimizationStatus,ActualClaimClass,
               new DeploymentConfiguration,
               ContinuationId,ContinuationStateRoot,PrefixReceipt)
  | violation(implementation-violation(ViolationTag),
              optimization-incomplete,ActualClaimClass=none,
              new DeploymentConfiguration,ViolationReport,QuarantineReceipt).

LedgerRecordableExecutionResult = FinalizedExecutionResult | no-run(NoRunReport).

ExecutionResult =
    duplicate(OriginalExecutionResultId, LedgerRecordableExecutionResult)
  | recovery-required(ExecutionReservationId, RecoveryRequestTemplate)
  | integrity-failure(unresolved-runtime, optimization-incomplete,
                      ActualClaimClass=none,
                      ExecutionIntegrityReport, ObservedEffectLedgerState,
                      CurrentDeploymentHeadState)
  | executed(RuntimeStatus, OptimizationStatus, ActualClaimClass,
             new DeploymentConfiguration, Receipt)
  | partial(RuntimeStatus, OptimizationStatus, ActualClaimClass?,
            new DeploymentConfiguration, PartialReceipt)
  | productive(productive, OptimizationStatus, ActualClaimClass,
               new DeploymentConfiguration,
               ContinuationId, ContinuationStateRoot, PrefixReceipt)
  | violation(implementation-violation(ViolationTag), optimization-incomplete,
              ActualClaimClass=none, new DeploymentConfiguration,
              ViolationReport, QuarantineReceipt)
  | no-run(NoRunReport)

For `violation`, `ViolationTag` MUST equal the exact classification proved by
`ViolationReport`; neither the tag nor report may be chosen independently.

RuntimeConformanceResult =
    accept(VerifiedStatementIds)
  | exact-counterexample(CounterexampleProof)
  | undeclared-event(ExactOutcomeClassificationProof)
  | reject-malformed(Reason)
  | reject-invalid-proof(Reason)
  | unresolved(DependencyIds)
  | unsupported(FeatureIds)
  | resource-exhausted(ResourceReport)
  | internal-failure(FailureReport)

ViolationReportBody = (
  ExecutionTransactionId, ExecutionSubject,
  ExactCounterexampleOrUndeclaredEventProof,
  ViolationTag, AffectedRealizationMachineBindingId,
  ExactObservedEventRoot, StateAndEffectRoot,
  BeforeRuntimePolicyStateRoot, QuarantinedRuntimePolicyStateRoot
)
ViolationReportId = Identity(violation-report-domain,ViolationReportBody)
ViolationReport = (ViolationReportBody,ViolationReportId).

PartialEffectReport =
    declared-partial(RuntimeStatus, StateAndEffectRoot,
                     CompletionContractEvidence)
  | inconclusive-runtime-conformance(
      RuntimeConformanceResult,
      StateAndEffectRoot, QuarantinedRuntimePolicyStateRoot)

PartialReceiptCore = (
  ExecutionTransactionId, ExecutionSubject,
  BeforeDeploymentConfiguration, AfterDeploymentConfiguration,
  RuntimeStatus, OptimizationStatus, ActualClaimClass?,
  SelectedSystemOrPolicyId?, CertificateId,
  BeforeEffectLedgerRoot, AfterEffectLedgerRoot,
  ChargedExecutionTraceRoot,
  PartialEffectReport
)
PartialReceiptBody = (PartialReceiptCore,RuntimeVerificationRecord)
PartialReceiptId = Identity(partial-receipt-domain,PartialReceiptBody)
PartialReceipt = (PartialReceiptBody,PartialReceiptId).

ExecutionReceiptCore = (
  ExecutionTransactionId, ExecutionSubject,
  BeforeDeploymentConfiguration, AfterDeploymentConfiguration,
  RuntimeStatus, OptimizationStatus, ActualClaimClass,
  SelectedSystemOrPolicyId, ExactOutcomeRoot,
  BeforeEffectLedgerRoot, AfterEffectLedgerRoot,
  ChargedExecutionTraceRoot
)
ExecutionReceiptBody = (ExecutionReceiptCore,RuntimeVerificationRecord)
ExecutionReceiptId = Identity(execution-receipt-domain,ExecutionReceiptBody)
Receipt = (ExecutionReceiptBody,ExecutionReceiptId).

PrefixReceiptCore = (
  ExecutionTransactionId, ExecutionSubject,
  BeforeDeploymentConfiguration, AfterDeploymentConfiguration,
  OptimizationStatus, ActualClaimClass,
  SelectedSystemOrPolicyId, ExactPrefixObservationRoot,
  ContinuationId, ContinuationStateRoot, ContinuationStepNumber,
  BeforeEffectLedgerRoot, AfterEffectLedgerRoot,
  ChargedPrefixTraceRoot
)
PrefixReceiptBody = (PrefixReceiptCore,RuntimeVerificationRecord)
PrefixReceiptId = Identity(prefix-receipt-domain,PrefixReceiptBody)
PrefixReceipt = (PrefixReceiptBody,PrefixReceiptId).

QuarantineReceiptCore = (
  ExecutionTransactionId, ExecutionSubject,
  BeforeDeploymentConfiguration, QuarantinedDeploymentConfiguration,
  ViolationTag, ViolationReportId,
  BeforeEffectLedgerRoot, AfterEffectLedgerRoot,
  RuntimePolicyStateRoot, ChargedTraceRoot
)
QuarantineReceiptBody = (QuarantineReceiptCore,RuntimeVerificationRecord)
QuarantineReceiptId = Identity(quarantine-receipt-domain,QuarantineReceiptBody)
QuarantineReceipt = (QuarantineReceiptBody,QuarantineReceiptId).

RequiredRuntimeStatementBody =
    execution(
      ExecutionRequestBodyRoot, ExecutionSubject,
      ExactInvocationOrWorkloadInputOrContinuationIdentity,
      AnswerIdentity, CertificateId, SelectedSystemOrPolicyId,
      EvaluatedReservationRoot, ExactObservedEventRoot,
      SuccessorDeploymentConfigurationRoot,
      EvaluatedExecutionResultCoreRoot, EvaluatedReceiptCoreRoot,
      RuntimeStatus, OptimizationStatus, ActualClaimClass?,
      ChargedTraceRoot, ContinuationAndPrefixFields?)
  | recovery(
      RecoveryRequestBodyRoot, OriginalExecutionRequestBodyRoot,
      OriginalExecutionSubject,
      OriginalInvocationOrWorkloadInputOrContinuationIdentity,
      OriginalAnswerIdentity, OriginalCertificateId,
      OriginalSelectedSystemOrPolicyId,
      RecoveryValidationInputRoot, EvaluatedReservationRoot,
      SuccessorDeploymentConfigurationRoot,
      EvaluatedExecutionResultCoreRoot, EvaluatedReceiptCoreRoot,
      EvaluatedRecoveryCommitPreCoreRoot,
      RuntimeStatus, OptimizationStatus, ActualClaimClass?,
      ChargedTraceRoot).

RequiredRuntimeStatementId = Identity(
  runtime-conformance-statement-domain,RequiredRuntimeStatementBody).

The two constructors are closed and domain-separated by their tags. Every
runtime verifier receives the exact body and its displayed identity; a
variadic hash preimage, an untagged tuple, or an identity with no retained body
is not a required runtime statement.

RuntimeVerificationEvidenceObjectGraph = (
  RequiredRuntimeStatementBody,
  EvaluatedExecutionResultCore, PublishedExecutionResultCore,
  EvaluatedReceiptCore, PublishedReceiptCore,
  EvaluatedRecoveryCommitPreCore?, PublishedRecoveryCommitPreCore?,
  RuntimeConformanceResult, ExactVerifierEvidenceObject,
  PostCheckTransitionEvidenceObject
).

RuntimeVerificationEvidenceObjectGraphRoot = Identity(
  runtime-verification-evidence-graph-domain,
  RuntimeVerificationEvidenceObjectGraph).

RuntimeVerificationEvidenceObjectGraphObjectId = Identity(
  runtime-verification-evidence-object-domain,
  RuntimeVerificationEvidenceObjectGraphRoot).

RuntimeVerificationEvidenceRetentionStatement = (
  RuntimeVerificationEvidenceObjectGraphRoot,
  RuntimeVerificationEvidenceObjectGraphObjectId,
  RequiredResultReceiptAndRecoveryReceiptLifetime,
  DurableFaultDomain,ImmutableObjectLocatorAndReplicationProfileId
).

RuntimeVerificationEvidenceRetentionStatementId = Identity(
  runtime-verification-evidence-retention-domain,
  RuntimeVerificationEvidenceRetentionStatement).

RuntimeVerificationEvidenceRetentionWarrant = (
  RuntimeVerificationEvidenceRetentionStatement,
  RuntimeVerificationEvidenceRetentionStatementId,
  ProofId,
  VerifierResult=accept(VerifiedStatementIds containing
                        RuntimeVerificationEvidenceRetentionStatementId)
).

RuntimeVerificationEvidenceRetentionWarrantId = Identity(
  runtime-verification-evidence-retention-warrant-domain,
  RuntimeVerificationEvidenceRetentionWarrant).

RuntimeVerificationEvidenceRetentionWarrantObjectId is the typed immutable
identifier resolving that exact warrant.

RuntimeVerificationRecordBody = (
  RequiredRuntimeStatementId,
  ReceiptCoreTypeBinding,
  EvaluatedExecutionResultCoreRoot,
  PublishedExecutionResultCoreRoot,
  EvaluatedReceiptCoreRoot,
  PublishedReceiptCoreRoot,
  EvaluatedRecoveryCommitPreCoreRoot?,
  PublishedRecoveryCommitPreCoreRoot?,
  RuntimeConformanceResult,
  ExactVerifierEvidenceRoot,
  PostCheckTransitionEvidenceRoot,
  RuntimeVerificationEvidenceObjectGraphRoot,
  RuntimeVerificationEvidenceObjectGraphObjectId,
  RuntimeVerificationEvidenceRetentionWarrantId,
  RuntimeVerificationEvidenceRetentionWarrantObjectId
)
RuntimeVerificationRecordId = Identity(
  runtime-verification-record-domain,RuntimeVerificationRecordBody).
RuntimeVerificationRecord = (
  RuntimeVerificationRecordBody,RuntimeVerificationRecordId).

ReceiptCoreTypeTag = execution | partial | prefix | quarantine.

ReceiptCoreTypeBinding =
    same(ReceiptCoreTypeTag)
  | published-quarantine-conversion(
      EvaluatedReceiptCoreTypeTag,PublishedReceiptCoreTypeTag,
      ExactAllowedConversionStatementId).

The required runtime statement is constructed over the applicable receipt core,
and its paired execution-result core, never over a final receipt or result body.
Only after the check returns is
`RuntimeVerificationRecord` constructed; the final receipt then embeds that
exact `(RuntimeVerificationRecordBody,RuntimeVerificationRecordId)` beside the
unchanged core. Neither the record nor final receipt is an
input to its own required statement, so the construction is acyclic.
The evidence graph likewise contains only the pre-record statement, cores, and
evidence—not this record, a final receipt, or a final result. It and its accepted
retention warrant MUST be stored in the same bounded finalization operation that
constructs the record. Thus evaluated cores remain rederivable after a published
quarantine conversion; no verifier relies on a naked hash preimage.

ExecutionResultCore =
    executed-core(ExecutionReceiptCore)
  | partial-core(PartialReceiptCore)
  | productive-core(PrefixReceiptCore)
  | violation-core(ViolationReport,QuarantineReceiptCore).

ReceiptCoreOfExecutionResultCore(core) =
    execution(core.ExecutionReceiptCore) when core is executed-core
  | partial(core.PartialReceiptCore) when core is partial-core
  | prefix(core.PrefixReceiptCore) when core is productive-core
  | quarantine(core.QuarantineReceiptCore) when core is violation-core.

ExecutionResultCoreTypeTag(core) is respectively
`execution | partial | prefix | quarantine`. Runtime status, optimization
status, actual claim, successor deployment, selected system/policy,
continuation, effect roots, and charged trace are dependent projections of that
single receipt core; the productive runtime status is fixed to `productive`, and
the violation optimization/claim fields are fixed to
`optimization-incomplete/none`. For `violation-core`, the report ID, transaction,
subject, violation tag, observed state/effect roots, and quarantined successor
MUST equal the corresponding `QuarantineReceiptCore` fields. A constructor with
any mismatch has no inhabitant.

`ValidExecutionResultCoreFor(Request,core)` additionally requires the projected
`ExecutionTransactionId` and `ExecutionSubject` to equal the exact request and
requires every branch-specific receipt-core invariant above. Runtime staging,
recovery staging, and finalization accept only this dependent subtype; duplicated
outer status or successor fields do not exist.

`FinalizeExecutionResult(core,RuntimeVerificationRecord)` is the total typed
projection on `ValidExecutionResultCoreFor(Request,core)` that pairs the core's
receipt core with that exact full record (whose included ID rederives) and
constructs the corresponding final
`ExecutionResult` constructor entirely from the dependent projections. It cannot
introduce or change any runtime status, optimization status, claim, successor,
continuation, violation, or receipt-core field.

ExecutionOutcomeEvaluationCore = (
  ExactObservedEvent,EvaluatedSuccessorDeploymentConfiguration,
  EvaluatedExecutionResultCore,EvaluatedReceiptCore,
  ChargedTraceRoot,ContinuationAndPrefixFields?
).

ProposedExecutionOutcome = (
  ExecutionOutcomeEvaluationCore,
  PublishedSuccessorDeploymentConfiguration,
  PublishedExecutionResultCore,PublishedReceiptCore,
  IdentityPostCheckTransitionEvidence
).

StagedExecutionOutcome = (
  ExecutionOutcomeEvaluationCore,
  PublishedSuccessorDeploymentConfiguration,
  PublishedExecutionResultCore,PublishedReceiptCore,
  PostCheckTransitionEvidence
).

For a proposed outcome the published successor/cores equal the evaluated
successor/cores and the transition evidence is the exact identity proof.
`StagedExecutionOutcome` has the same carrier but may replace the published
successor/cores only with the exact permitted violation or inconclusive
quarantine conversion and its retained post-check evidence. In both carriers the
evaluated and published receipt cores MUST equal
`ReceiptCoreOfExecutionResultCore` of their paired result cores; runtime status,
optimization status, actual claim, selected system/policy, continuation, effect
roots, and trace are dependent projections, never duplicated independent
fields. Legacy projections such as `ImmutableExecutionResultCore`,
`ImmutableReceiptCore`, `SuccessorDeploymentConfiguration`, `RuntimeStatus`, and
`ActualClaimClass` denote these exact dependent projections.

ExecutionOutcomeStageResult =
    complete(ProposedExecutionOutcome)
  | rejected(Reason) | incoherent(ConflictProof)
  | unresolved(DependencyIds) | unsupported(FeatureIds)
  | resource-exhausted(ResourceReport) | internal-failure(FailureReport).

ExecutionPostCheckStageResult =
    complete(StagedExecutionOutcome)
  | rejected(Reason) | incoherent(ConflictProof)
  | unresolved(DependencyIds) | unsupported(FeatureIds)
  | resource-exhausted(ResourceReport) | internal-failure(FailureReport).

`BoundedStageProposedExecutionOutcome` returns exactly
`ExecutionOutcomeStageResult`. Each safe-partial, violation, and inconclusive
post-check stage returns exactly `ExecutionPostCheckStageResult`; only its
`complete` value may enter finalization.

ProposedRecoveryOutcome = (
  RecoveryValidationInputRoot,ReconstructedEventRoot,
  EvaluatedSuccessorDeploymentConfiguration,
  EvaluatedExecutionResultCore,EvaluatedReceiptCore,
  RecoveryCommitPreCore,ChargedTraceRoot
).

PublishedRecoveryOutcome = (
  PublishedSuccessorDeploymentConfiguration,
  PublishedExecutionResultCore,PublishedReceiptCore,
  PublishedRecoveryCommitPreCore,PostCheckTransitionEvidence
).

RecoveryRuntimeStageResult =
    complete(RuntimeConformanceResult,PublishedRecoveryOutcome)
  | checkpoint(RecoveryCheckpointCandidate,ObligationIds)
  | rejected(Reason) | incoherent(ConflictProof)
  | unresolved(DependencyIds) | unsupported(FeatureIds)
  | resource-exhausted(ResourceReport) | internal-failure(FailureReport).

The recovery proposed carrier's receipt core is the exact projection of its
result core, and its pre-core's successor/before-head fields equal that evaluated
successor and acquired tail reservation. The published carrier obeys the same
result/receipt projection and its published pre-core equals
`BuildPublishedRecoveryCommitPreCore` for its actual successor. Runtime status,
optimization status, claim, selected system/policy, and trace are dependent
projections. `BoundedVerifyAndStageRecoveryRuntimeOutcome` returns exactly
`RecoveryRuntimeStageResult`; no untyped `proposedRecovery` or
`publishedRecovery` record is accepted by finalization.

RecoveryOutcomeStageResult =
    complete(ProposedRecoveryOutcome)
  | checkpoint(RecoveryCheckpointCandidate,ObligationIds)
  | rejected(Reason) | incoherent(ConflictProof)
  | unresolved(DependencyIds) | unsupported(FeatureIds)
  | resource-exhausted(ResourceReport) | internal-failure(FailureReport).

`BoundedStageRecoveredOutcome` and
`BoundedStageEmergencyRecoveredQuarantine` return exactly this algebra; the
tail warrant makes its checkpoint branch a typed warrant violation rather than
a resumable publication.

`RuntimeVerificationRecordBody` is well-formed only when its evaluated and
published receipt-core roots equal the roots returned by
`ReceiptCoreOfExecutionResultCore` on the corresponding result cores; its
`ReceiptCoreTypeBinding` is `same` with both branch tags equal, or the exact
proved `published-quarantine-conversion`; and the evaluated/published branch relation is
the one named by `PostCheckTransitionEvidenceRoot`. Both recovery-pre-core roots
are present exactly for recovery finalization and absent otherwise. A record that
mixes an independent type tag, receipt root, result core, or one-sided recovery
pre-core has no inhabitant.

In addition, its `RuntimeVerificationEvidenceObjectGraphObjectId` MUST strictly
resolve the exact `RuntimeVerificationEvidenceObjectGraph`; recomputing the
displayed graph root MUST equal the record field. The graph's required-statement
body, evaluated/published result cores, receipt cores, optional recovery
pre-cores, runtime-check value, verifier evidence, and post-check transition
evidence MUST project to and rederive every corresponding ID/root in the record.
Recovery pre-cores are both present exactly for the `recovery` statement
constructor and absent exactly for `execution`. The retention-warrant object
MUST resolve an accepted warrant whose exact statement names this graph root,
object ID, lifetime, and fault domain. An unrelated retained graph or naked
digest cannot inhabit a verification record.

PartialEffectReportId
  = Identity(partial-effect-report-domain,PartialEffectReport).

ReceiptLikeBodyOfExecutionResult(result) =
    some(ExecutionReceiptBody) when result is executed
  | some(PrefixReceiptBody) when result is productive
  | some(PartialReceiptBody) when result is partial
  | some(QuarantineReceiptBody) when result is violation
  | none otherwise.

ReceiptCoreOfExecutionResult(result) =
    ExecutionReceiptCore when result is executed
  | PrefixReceiptCore when result is productive
  | PartialReceiptCore when result is partial
  | QuarantineReceiptCore when result is violation.

`ReceiptOrPrefixReceiptBody(result)` is the legacy name for the nonempty value
of this exact projection. It MUST NOT be invoked on a branch whose projection is
`none`, and it MUST NOT accept a bare receipt type name.

InconclusiveRuntimeConformanceReport(check,stateAndEffectRoot,
                                     quarantinedRuntimePolicyStateRoot)
  = inconclusive-runtime-conformance(
      check,stateAndEffectRoot,quarantinedRuntimePolicyStateRoot)

SealResult =
    sealed(SealCertificate, SealId)
  | checkpoint(SealCheckpoint, ObligationIds)
  | incomplete(ObligationIds)
  | rejected(SealRejectionReport)
  | incoherent(ConflictProof)
  | unresolved(DependencyIds)
  | conflict(CurrentIdentity)
  | unsupported(FeatureIds)
  | resource-exhausted(ResourceReport)
  | internal-failure(FailureReport)

SealRejectionReport =
    malformed-seal-input(Details)
  | malformed-seal-checkpoint(Details)
  | invalid-seal-scope(Details)
  | stage-rejected(SealSubstage,Details).

SealCheckpointBody = (
  SealOperationScope,
  SnapshotId, RequestedSealObligationRoot, ResourceContractId,
  SealSubstage,
  ProcessedWorkRoot, PendingWorkRoot, ImmutablePriorOutputRoots,
  ImplementationId, VerifierId, ProgressMeasureAndWarrant
)

SealCheckpointRoot = Identity(seal-checkpoint-domain,SealCheckpointBody)
SealCheckpoint = (SealCheckpointBody,SealCheckpointRoot)

SealOperationScope =
    update(TransactionId)
  | reseal(OriginResealTransactionId, CurrentResealTransactionId)

SealSubstage = evidence-resolution | conjunction | certificate-build |
               exact-proof-verify

ClassCertificationResult =
    certified(ClassCertificate)
  | incomplete(PendingClassProof?, ObligationIds)
  | request-result(RequestStatus, Diagnostics)
  | verifier-result(VerifierResult)
  | incoherent(ConflictProof)
  | unresolved(DependencyIds)
  | unsupported(FeatureIds)
  | resource-exhausted(ResourceReport)
  | internal-failure(FailureReport).

FamilyCertificationResult =
    certified(FamilyCertificate)
  | incomplete(PendingFamilyProof?, ObligationIds)
  | request-result(RequestStatus, Diagnostics)
  | verifier-result(VerifierResult)
  | incoherent(ConflictProof)
  | unresolved(DependencyIds)
  | unsupported(FeatureIds)
  | resource-exhausted(ResourceReport)
  | internal-failure(FailureReport).

EvidenceForQueryAnswer(q,a) =
    matching ExactIncompleteReport(q,a)       when a is Incomplete
  | VerifiedResultCertificate(q,a)            otherwise

EvidenceForUseCaseAnswer(u,a) =
    matching ExactIncompleteReport(u,a)       when a is WorkloadIncomplete
  | VerifiedResultCertificate(u,a)            otherwise

VerifiedResultCertificate(scope,a)
  = a §12.6 CertificateBody whose verifier accepted the exact statement of a

ResolverStageTag =
    classify | derive | universe-validation | seal-coverage | admission-coverage |
    answer-construction | certificate-construction

ResolverCheckpointBody = (
  ResolutionOperationScope, SnapshotId, SealId, ResolutionScope,
  ResolverStageTag,
  ProcessedWorkRoot, PendingWorkRoot,
  ImmutablePriorOutputRoots,
  ImplementationId, VerifierId, ProgressMeasureAndWarrant
)

ResolverCheckpointRoot = Identity(
  resolver-checkpoint-domain,ResolverCheckpointBody)

ResolverCheckpoint = (ResolverCheckpointBody,ResolverCheckpointRoot)

ResolutionOperationScope = (
  OriginResolutionRequestId,
  CurrentResolutionRequestId
).

ResolverContinuation =
    no-checkpoint(ExactTerminalIncompleteReason)
  | checkpoint(ResolverCheckpoint)

ExactIncompleteReport = (
  IncompleteReportId,
  ResolutionRequestId,
  SnapshotId, SealId,
  QueryIdOrUseCaseRequestId,
  AnswerIdentity,
  RequestedClaimScope,
  OutstandingObligationIds,
  DependencyManifestRoot,
  ResolverContinuation
)

IncompleteReportId = Identity(
  incomplete-report-domain,
  ExactIncompleteReport excluding IncompleteReportId).

BuildExactIncompleteReport MUST populate this complete tuple and prove that the
answer's obligations, scope, and continuation are exactly those returned by the
bounded stage; a bare type name or ambient checkpoint is not a value.

ClassCertificate = verified §12.6 UseCaseClassBinding certificate
FamilyCertificate = verified §12.6 QueryFamilyBinding certificate
`PendingClassProof` and `PendingFamilyProof` are the exact resource-bounded,
scope-bound quantified-proof checkpoints defined with the certification request
machine below; they are not opaque implementation tokens.

PrepareProof = a closed verifier-checkable statement binding the exact
  UpdateRequestBody/RequestIdentity, candidate body/id, HistoryRoot request and
  parent fields, validated base identity/integrity, complete runtime read set or
  declaration-only proof, AdmissionReport identity, pending/unresolved roots, and either
  the seal certificate/id/requested obligations or the exact unsealed status.
```

`PREPARE_UPDATE` returns either `PreparedUpdate` or a nonpublication
`UpdateResult` branch. `COMMIT_UPDATE` consumes only `PreparedUpdate` and
returns `UpdateResult`. `RESUME_UPDATE` consumes a validated `PendingUpdate`.
`RESOLVE_QUERY` and `RESOLVE_USE_CASE` return `ResolutionResult`;
`CERTIFY_QUERY_FAMILY` returns `FamilyCertificationResult` and
`CERTIFY_USE_CASE_CLASS` returns `ClassCertificationResult`;
`CERTIFY_INPUT_TOTAL_CAPABILITY` returns
`InputTotalCapabilityCertificationResult`;
`ACTIVATE_SEAL` returns the activation algebra in §7.11; and
`EXECUTE_CERTIFIED`/`EXECUTE_USE_CASE_CERTIFIED` return `ExecutionResult`. Thus malformed, invalid,
unsupported, unresolved, conflict, verifier rejection, exhaustion, and
internal-failure paths have typed terminal results rather than host-language
exceptions. Every “require” below is a typed check whose failure returns the
listed exact branch; it is never a host-language abort.

`GENESIS` is `PREPARE_UPDATE` and `COMMIT_UPDATE` with a distinguished empty
parent identity, an empty active base/pending set, and an empty expected
`KnowledgeHead`. The empty objects and transition profile are identity-bearing;
they are not ambient defaults.

Every stage whose carrier or proof search may be unbounded—including base and
runtime-read-set validation, dependency resolution, closure, quotient construction, occurrence indexing, accumulation,
derivation-space construction, dependency manifests, exact diff/impact, and
post-identity evidence—uses the common bounded stage algebra

```text
StageResult(A) =
    complete(A)
  | checkpoint(PendingUpdate, ObligationIds)
  | rejected(Reason)
  | incoherent(ConflictProof)
  | unresolved(DependencyIds)
  | unsupported(FeatureIds)
  | resource-exhausted(ResourceReport)
  | internal-failure(FailureReport).
```

The bound `ResourceContract` fixes finite work/space/verification budgets and a
checkpoint policy. A stage MUST return one tag within that contract; it may not
search an arbitrary derivation or theorem space indefinitely. Only `complete`
advances. Every checkpoint commits its stage tag, target, processed/pending work
roots, immutable prior outputs, and progress warrant as §7.11 requires.
An update contract partitions or otherwise proves non-overlapping availability
for prepare/resume work and the final commit-validation slice; prepare cannot
consume the budget reserved for validating a `PreparedUpdate` at publication.
Every bounded call below receives an exact `CheckpointContext` constructed from
the corresponding `PendingUpdateBody` fields. In every call, without exception,
`checkpoint(P,ids)` maps to `UpdateResult.pending(P,ids)` after verifying
`P.StageTag`, `P.CheckpointTarget`, prior-output roots, and progress warrant;
verification failure maps to `rejected(corrupt-checkpoint(details))`. The caller
does not synthesize missing state after a stage exhausts its budget.
Every `complete` stage returns the charged, verified roots and identities of all
its outputs. Later `Root(...)` or `Identity(...)` notation references those
returned commitments and MUST NOT rescan or rehash an arbitrary carrier.
Likewise, construction of snapshot-bound queries/use-case requests occurs
inside `BoundedBuildSnapshotScopedSealEvidence`; eager argument evaluation is
not an unbounded caller-side operation.

`LiftStageToUpdate` is the total, type-preserving map used by every phrase
“matching exact stage branch” below:

```text
checkpoint(P,ids)       -> pending(ValidateCheckpoint(P),ids)
rejected(reason)        -> rejected(other-exact-update-rejection(
                                      stage-rejected,reason))
incoherent(proof)       -> incoherent(proof)
unresolved(ids)         -> unresolved(ids)
unsupported(ids)        -> unsupported(ids)
resource-exhausted(r)   -> resource-exhausted(r)
internal-failure(r)     -> internal-failure(r).
```

For a `SealResult`, `checkpoint` first converts the exact seal checkpoint to a
post-candidate `PendingUpdate`; `conflict(x)` maps to
`conflict(UpdateConflictReport(seal-conflict,expected,actual,...))`;
`rejected(s)` maps to
`rejected(other-exact-update-rejection(seal-rejected,s))`;
`incomplete` is handled only by constructing `ExactIncompleteSealStatus`; and
all remaining tags map homonymously. No raw `Reason`, `CurrentIdentity`, or
checkpoint is returned where `UpdateResult` requires a structured report.

```text
PREPARE_UPDATE(Base, Request, TypedDeltaOrBytes):
  requestIdentity := Identity(update-transaction-domain,
                              Request excluding TransactionId)
  if requestIdentity != Request.TransactionId:
    return rejected(malformed-transaction-identity(details))

  resourceResult := ResolveResourceContract(Request.UpdateResourceContractId)
  on rejected/unresolved/unsupported/exhausted/failure:
    return the matching exact nonpublication branch
  ResourceContract := resourceResult.value

  decodeResult := BoundedStrictDecodeAndConsumeAll(TypedDeltaOrBytes,
                    ResourceContract,
                    CheckpointContext(
                      pre-candidate(requestIdentity),
                      decode(Request.DeltaId,
                             Request.DeltaInputDescriptorId,
                             none),
                      no-prior-stage-outputs))
                  or BoundedValidateTypedInMemoryDelta(
                    TypedDeltaOrBytes, ResourceContract, the same context)
  match decodeResult:
    complete(delta,rawInputRoot,deltaIdentity,inputDescriptorIdentity): continue
    checkpoint(P,ids): return pending(ValidateCheckpoint(P,decode),ids)
    rejected(reason): return LiftStageToUpdate(rejected(reason))
    incoherent(proof): return LiftStageToUpdate(incoherent(proof))
    unresolved(ids): return LiftStageToUpdate(unresolved(ids))
    unsupported(ids): return LiftStageToUpdate(unsupported(ids))
    resource-exhausted(report): return LiftStageToUpdate(resource-exhausted(report))
    internal-failure(report): return LiftStageToUpdate(internal-failure(report))
  if inputDescriptorIdentity != Request.DeltaInputDescriptorId:
    return rejected(delta-input-descriptor-mismatch)
  if deltaIdentity != Request.DeltaId:
    return rejected(delta-identity-mismatch(Request.DeltaId,deltaIdentity))
  // RawInputRoot and deltaIdentity are streaming outputs of the bounded stage;
  // neither the raw nor decoded carrier is rehashed outside that stage.

  baseValidationResult := BoundedValidateBaseAndResolveRuntimeReference(
    Base,Request.ExpectedKnowledgeHead,Request.ExpectedRuntimeState,
    ResourceContract,
    CheckpointContext(
      pre-candidate(requestIdentity),
      base-and-runtime-validation(
        Request.ExpectedKnowledgeHead,
        RootOfExpectedRuntimeStateReference(Request.ExpectedRuntimeState)),
      exact decoded-delta/raw-input/input-descriptor roots))
  match baseValidationResult:
    complete(validatedBase,resolvedExpectedRuntimeState?,
             baseIdentityAndIntegrityProof,runtimeReferenceProof):
      Base := validatedBase
    checkpoint(P,ids):
      return pending(ValidateCheckpoint(P,base-and-runtime-validation),ids)
    rejected/incoherent/unresolved/unsupported/resource-exhausted/internal-failure:
      return the matching exact nonpublication branch

  transitionResult := BoundedResolveVerifyAndPrepareTransition(
    Request.TransitionProfileId, Base, delta,
    resolvedExpectedRuntimeState?,
    baseIdentityAndIntegrityProof,ResourceContract,
    CheckpointContext(
      pre-candidate(requestIdentity),
      transition-resolution(Request.TransitionProfileId,Request.DeltaId,
                            Root(Base),Root(PendingDeclarations(Base))),
      exact base/runtime-reference proof and decoded-delta roots))
  match transitionResult:
    complete(transition,U',transitionedBase,transitionedPending,
             runtimeReadSet,runtimeUseProof,declarationOnlyProof?):
      require runtimeUseProof proves that Request.ExpectedRuntimeState is
        present exactly when the transition/admission/body consumes mutable
        runtime state; when it is absent, require declarationOnlyProof to prove
        the entire prospective update is declaration-only
      continue
    checkpoint(P,ids):
      return pending(ValidateCheckpoint(P,transition-resolution),ids)
    rejected/incoherent/unresolved/unsupported/resource-exhausted/internal-failure:
      return the matching exact branch
  resolvedResult := BoundedResolveTypedDependencies(
    delta, transitionedBase, transitionedPending, U', ResourceContract,
    CheckpointContext(
      pre-candidate(requestIdentity),
      dependency-resolution(Request.TransitionProfileId,Identity(U'),
                            Request.DeltaId,Root(transitionedBase),
                            Root(transitionedPending)),
      exact decode/transition output roots))
  match resolvedResult:
    complete(resolvedDependencies): continue
    checkpoint(P,ids):
      return pending(ValidateCheckpoint(P,dependency-resolution),ids)
    rejected/incoherent/unresolved/unsupported/resource-exhausted/internal-failure:
      return the matching exact branch

  admissionResult := BoundedAdmitAndFinalizeBatch_(U')(
    transitionedBase, transitionedPending, resolvedDependencies,
    transition,
    ResourceContract,
    CheckpointContext(
      pre-candidate(requestIdentity),
      admission(Request.TransitionProfileId,Identity(U'),Request.DeltaId,
                Root(transitionedBase),Root(transitionedPending),
                Root(resolvedDependencies)),
      exact decode/transition/dependency output roots))
  match admissionResult:
    complete(activeDelta,report,pending',activeBase'): continue
    checkpoint(P,ids): return pending(ValidateCheckpoint(P,admission),ids)
    rejected/incoherent/unresolved/unsupported/resource-exhausted/internal-failure:
      return the matching exact branch
  if report contains a batch-fatal incoherence:
    return incoherent(report.conflictProof)

  semanticCheckpointContext(stage,substage,priorRoots) := CheckpointContext(
    pre-candidate(requestIdentity),
    semantic-stage(substage,Request.TransitionProfileId,Identity(U'),Request.DeltaId,
                   Identity(report),Root(pending'),Root(activeBase'),priorRoots),
    exact processed/pending work roots, implementation/verifier/progress roots)
  closureResult := BoundedCompleteLeastClosure(
    activeBase', U', ResourceContract,
    semanticCheckpointContext(least-closure,closure-completion,
      exact decode/transition/dependency/admission output roots))
  match closureResult:
    complete(materialized(C',leastnessProof)):
      completeClosure := C'; closureLeastnessEvidence := leastnessProof
    complete(symbolic(SymbolicC',soundnessClosureLeastnessProof)):
      completeClosure := SymbolicC'
      closureLeastnessEvidence := soundnessClosureLeastnessProof
    checkpoint(P,ids): return pending(ValidateCheckpoint(
      P,(least-closure,closure-completion)),ids)
    rejected/incoherent/unresolved/unsupported/resource-exhausted/internal-failure:
      return the matching exact status

  quotientResult := BoundedBuildTypedProofQuotients(
    completeClosure, ResourceContract,
    semanticCheckpointContext(quotient,quotient-construction,
      exact prior roots plus completeClosure and closureLeastnessEvidence roots))
  match quotientResult:
    complete(quotients'): continue
    checkpoint(P,ids): return pending(ValidateCheckpoint(
      P,(quotient,quotient-construction)),ids)
    rejected/incoherent/unresolved/unsupported/resource-exhausted/internal-failure:
      return the matching exact nonpublication branch

  contributionResult := BoundedIndexOccurrencesAndMultiplicityForEveryProfile(
    activeBase', transition, U', ResourceContract,
    semanticCheckpointContext(accumulation,contribution-indexing,
      exact prior roots plus complete-closure/quotient roots))
  match contributionResult:
    complete(contributionMaps'): continue
    checkpoint(P,ids): return pending(ValidateCheckpoint(
      P,(accumulation,contribution-indexing)),ids)
    rejected/incoherent/unresolved/unsupported/resource-exhausted/internal-failure:
      return the matching exact nonpublication branch
  accumulationResult := BoundedBuildAccumulationMap(
    every G' in AccumulationProfiles(U',activeBase'), ResourceContract,
    using RecomputeFullAccumulation(contributionMaps'[G'],G')
      or IncrementalAccumulationWithEqualityProof(
           Base,transition,contributionMaps'[G'],G'),
    semanticCheckpointContext(accumulation,accumulation-fold,
      exact prior roots plus contribution/multiplicity roots))
  match accumulationResult:
    complete(accumulationMap',accumulationCompletenessEvidence):
      // By the stage result type, this evidence covers every committed G with
      // exact subject, canonical minimum, intermediate safety, and every
      // required limit/interchange proof.
      continue
    checkpoint(P,ids): return pending(ValidateCheckpoint(
      P,(accumulation,accumulation-fold)),ids)
    rejected/incoherent/unresolved/unsupported/resource-exhausted/internal-failure:
      return the matching exact nonpublication branch

  derivationResult := BoundedBuildCompleteMaterializedOrSymbolicDerivationSpace(
    completeClosure, quotients', accumulationMap', ResourceContract,
    semanticCheckpointContext(derivation-artifacts,
      derivation-space-construction,
      exact prior roots plus accumulation roots))
  match derivationResult:
    complete(derivations'): continue
    checkpoint(P,ids):
      return pending(ValidateCheckpoint(
        P,(derivation-artifacts,derivation-space-construction)),ids)
    rejected/incoherent/unresolved/unsupported/resource-exhausted/internal-failure:
      return the matching exact nonpublication branch
  artifactResult := BoundedCommitAlternativesReconstructionAndDependencies(
    derivations', completeClosure, quotients', accumulationMap',
    ResourceContract,
    semanticCheckpointContext(derivation-artifacts,
      artifact-dependency-commit,
      exact prior roots plus derivation-space roots))
  match artifactResult:
    complete(alternativeRoot',reconstructionRoot',bodyDependencies'): continue
    checkpoint(P,ids):
      return pending(ValidateCheckpoint(
        P,(derivation-artifacts,artifact-dependency-commit)),ids)
    rejected/incoherent/unresolved/unsupported/resource-exhausted/internal-failure:
      return the matching exact nonpublication branch

  bodyResult := BoundedBuildSnapshotBodyAndIdentity(
    exact parent/history/transition/transaction/report/pending roots plus
      baseIdentityAndIntegrityProof/runtimeReferenceProof/runtimeReadSet/
      runtimeUseProof/declarationOnlyProof roots,
    active base,completeClosure,closureLeastnessEvidence roots,
    quotient and analytic-profile roots,
    accumulation/contribution/multiplicity/subject/canonical roots plus
      accumulationCompletenessEvidence root,
    derivation/alternative/reconstruction/body-dependency/unresolved roots,
    machine/cost/use-case/extension/verifier roots,
    ResourceContract,
    semanticCheckpointContext(pre-identity-body,snapshot-body-construction,
      every exact prior output and dependency root))
  match bodyResult:
    complete(body',sid',candidate'):
      require candidate'=SnapshotCandidate(body',sid') and the bounded identity
        proof establishes sid'=Identity(body'); continue
    checkpoint(P,ids):
      return pending(ValidateCheckpoint(
        P,(pre-identity-body,snapshot-body-construction)),ids)
    rejected/incoherent/unresolved/unsupported/resource-exhausted/internal-failure:
      return the matching exact nonpublication branch

  impactResult := BoundedExactDiffAndSoundTransitiveImpact(
    Base, candidate', old/new active base and pending/report,
    old/new U/rules/trust/profiles, closure/quotient/accumulation roots,
    candidate/machine/cost/workload/class roots,
    BodyDependencyManifests(Base),
    SealDependencyManifests(Base) if Base is sealed,
    bodyDependencies', ResourceContract,
    CheckpointContext(
      post-candidate(sid'),
      post-candidate-stage(impact-analysis,sid',none,none,
                           Root(RequestedSealObligations(Request)),
                           exact candidate/body-dependency roots),
      exact prior stage roots))
  match impactResult:
    complete(transitionDiff,impact): continue
    checkpoint(P,ids):
      return pending(ValidateCheckpoint(
        P,(post-identity-impact,impact-analysis)),ids)
    rejected/incoherent/unresolved/unsupported/resource-exhausted/internal-failure:
      return the matching exact nonpublication branch

  postIdResult := BoundedBuildSnapshotScopedSealEvidence(
    candidate', ConstructSnapshotBoundQueriesAndUseCaseRequests(candidate'),
    ContinuationClasses(Request), alternativeRoot', reconstructionRoot',
    impact, RequestedSealObligations(Request), ResourceContract,
    CheckpointContext(
      post-candidate(sid'),
      post-candidate-stage(post-id-evidence-construction,
                           sid',Root(transitionDiff),Root(impact),
                           Root(RequestedSealObligations(Request)),
                           exact candidate/impact roots),
      exact prior stage roots))
  match postIdResult:
    complete(postIdEvidence):
      require postIdEvidence contains the exact set of any unsatisfied optional
        seal obligations and its SealVerifierContextId; continue
    checkpoint(P,obligations):
      return pending(ValidateCheckpoint(
        P,(post-identity-seal,post-id-evidence-construction)),obligations)
    rejected/incoherent/unresolved/unsupported/resource-exhausted/internal-failure:
      return the matching exact nonpublication branch
  // That exact optional-obligation set is part of postIdEvidence and is passed
  // to the sole verifier; it may justify only an unsealed result, never a
  // completed seal claim.

  sealResult := VERIFY_SEAL(update(requestIdentity),candidate', postIdEvidence,
                            RequestedSealObligations(Request),
                            ResourceContract)
  if sealResult = sealed(cert, sealId):
    prepareProofResult := BoundedBuildAndVerifyPrepareProof(
      Request excluding TransactionId,requestIdentity,candidate',report,
      sealed(cert,sealId),runtimeReadSet,baseIdentityAndIntegrityProof,
      runtimeReferenceProof,runtimeUseProof,declarationOnlyProof?,ResourceContract,
      CheckpointContext(
        post-candidate(sid'),
        post-candidate-stage(
          sealed-prepare-proof,sid',Root(transitionDiff),Root(impact),
          Root(RequestedSealObligations(Request)),
          exact candidate/postIdEvidence/cert/seal roots),
        exact prior stage roots))
    on checkpoint(P,ids):
      return pending(ValidateCheckpoint(
        P,(post-identity-seal,sealed-prepare-proof)),ids)
    on rejected/incoherent/unresolved/unsupported/exhausted/failure:
      return the matching exact nonpublication branch
    return PreparedSealed(Request excluding TransactionId, requestIdentity,
                          candidate', report, cert, sealId,
                          prepareProofResult.value)
  if sealResult = checkpoint(sealCheckpoint,obligations):
    conversion := ConvertSealCheckpointToPendingUpdate(
      sealCheckpoint,
      checkpointTarget=post-candidate(candidate'.SnapshotId),
      substage=seal-verification(sealCheckpoint.SealSubstage),
      exact Request/base/candidate/postIdEvidence/impact/prior-output roots)
    match conversion:
      valid(pendingUpdate): return pending(pendingUpdate,obligations)
      malformed/incoherent/unresolved/unsupported/resource-exhausted/internal-failure:
        return the matching exact nonpublication UpdateResult branch
  if sealResult = incomplete(obligations):
    incompleteStatus := BuildExactIncompleteSealStatus(
      candidate'.SnapshotId,Root(RequestedSealObligations(Request)),
      obligations,Root(postIdEvidence),postIdEvidence.SealVerifierContextId,
      Identity(seal-result-domain,sealResult))
    prepareProofResult := BoundedBuildAndVerifyPrepareProof(
      Request excluding TransactionId,requestIdentity,candidate',report,
      unsealed(incompleteStatus),runtimeReadSet,baseIdentityAndIntegrityProof,
      runtimeReferenceProof,runtimeUseProof,declarationOnlyProof?,ResourceContract,
      CheckpointContext(
        post-candidate(sid'),
        post-candidate-stage(
          unsealed-prepare-proof,sid',Root(transitionDiff),Root(impact),
          Root(RequestedSealObligations(Request)),
          exact candidate/postIdEvidence/incomplete-status roots),
        exact prior stage roots))
    on checkpoint(P,ids):
      return pending(ValidateCheckpoint(
        P,(post-identity-seal,unsealed-prepare-proof)),ids)
    on rejected/incoherent/unresolved/unsupported/exhausted/failure:
      return the matching exact nonpublication branch
    return PreparedUnsealed(Request excluding TransactionId, requestIdentity,
                            candidate', report, incompleteStatus,
                            prepareProofResult.value)
  return the matching rejected/incoherent/unresolved/unsupported/conflict/
         exhausted/failure result
```

The sole sealing path is total and independent of optimizer assertions:

```text
VERIFY_SEAL(sealOperationScope, candidate, postIdEvidence, requestedObligations,
            ResourceContract):
  checks := BoundedVerifyConjunction(ResourceContract,
    strict consume-all body/evidence decoding and bounded identity recomputation,
    exact sealOperationScope binding,
    recomputed SnapshotId equals candidate.SnapshotId,
    every postIdEvidence item binds that SnapshotId, the exact requested
      obligation, its dependency manifest, and no pre-ID circular conclusion,
    parent/transition identities,
    least closure soundness/completeness and fixed-point/merge laws,
    typed quotient equivalence/extensionality and analytic profile obligations,
    contribution/multiplicity/accumulation/canonical-state equations,
    operational-alternative and reconstruction roots,
    for every seal-covered boundary: RetentionCoverage and contextual warrants,
    every claimed universe/admission/lower-bound/frontier/class envelope,
    exact dependency/unresolved roots and closed noncircular hypotheses,
    verifier foundation/configuration and requested-obligation coverage)
  match checks:
    complete(checkProofs,recomputedSid,
             verifiedOutstandingRequestedSealObligations): continue
    checkpoint(P,ids):
      checkpointValidation := ValidateSealCheckpoint(
        P,sealOperationScope,candidate,
        requestedObligations,ResourceContract)
      on valid(sealCheckpoint): return checkpoint(sealCheckpoint,ids)
      on malformed/incoherent/unresolved/unsupported/exhausted/failure:
        return the matching exact SealResult
    rejected/incoherent/unresolved/unsupported/resource-exhausted/internal-failure:
      return the matching exact SealResult
  if verifiedOutstandingRequestedSealObligations is nonempty:
    return incomplete(verifiedOutstandingRequestedSealObligations)

  sealBuild := BoundedBuildSealCertificateBodyAndStatement(
    candidate,postIdEvidence,checkProofs,requestedObligations,
    remaining ResourceContract)
  match sealBuild:
    complete(sealBody,requiredStatement,proposedSealId): continue
    checkpoint(P,ids):
      checkpointValidation := ValidateSealCheckpoint(
        P,sealOperationScope,candidate,
        requestedObligations,ResourceContract)
      on valid(sealCheckpoint): return checkpoint(sealCheckpoint,ids)
      on malformed/incoherent/unresolved/unsupported/exhausted/failure:
        return the matching exact SealResult
    rejected/incoherent/unresolved/unsupported/resource-exhausted/internal-failure:
      return the matching exact SealResult
  verifierResult := BoundedVerifyExactStatement(
    sealBody.proof,requiredStatement,remaining ResourceContract)
  match verifierResult:
    accept(ids) when requiredStatement in ids: continue
    accept(ids): return incoherent(verifier-accepted-wrong-statement(ids))
    reject-malformed/reject-invalid-proof:
      return incoherent(verifierResult)
    unresolved/unsupported/resource-exhausted/internal-failure:
      return the matching exact SealResult

  require proposedSealId is the bounded canonical identity of sealBody
    established by sealBuild
  return sealed(SealCertificate=sealBody, proposedSealId)
```

`RESUME_VERIFY_SEAL` is the same state machine beginning at the exact
`SealSubstage` and work/proof roots committed by `SealCheckpoint`. It first
validates the checkpoint body/root, origin operation, candidate, obligation,
resource-continuation, implementation, verifier, and strict progress warrant;
then it rewrites only `CurrentResealTransactionId` to the bound resume request.
It returns `SealResult` and may advance to a new checkpoint, proved
`incomplete`, or `sealed`. Restarting an exhausted conjunction or certificate
build from `postIdEvidence` is not conforming resumption.

`RESEAL` MUST invoke exactly this procedure with the unchanged candidate and a
new requested obligation set; no private weaker verifier or detached seal
constructor is conforming.

`IncrementalAccumulationWithEqualityProof` and affected-region closure are
permitted only when their result is proved equal to the full definitions for
this transition. A partial saturation is retained only in `PendingUpdate`; it
does not populate a body field claiming the least closure.

```text
RESUME_UPDATE(Pending, ResourceContinuationToken):
  validate Pending's base/request/U/checkpoint/processed/pending-work/
    implementation/verifier identities
  validate the token is authorized by the same UpdateResourceContractId and
    does not silently enlarge its declared total bound
  if any identity changed: return conflict or unsupported; publish nothing
  continue the same `PREPARE_UPDATE` state machine from the committed checkpoint
  return PreparedUpdate, a new PendingUpdate with a proved progress step,
    or the exact rejected/incoherent/unresolved/unsupported/exhausted/failure
    branch.
```

Resumption does not retarget a checkpoint to a new head. Rebase, profile
change, or a larger resource contract is a new update request and identity.

```text
COMMIT_UPDATE(Prepared, CoordinatorState C):
  updateIngressResult : CoordinatorTransactionIngressResult :=
    BoundedCoordinatorTransactionIngress(
    update,Prepared.UpdateRequestBody,Prepared.RequestIdentity,
    CoordinatorTransactionIngressProfile,C)
  match updateIngressResult:
    complete(preparedBody,recomputedTransactionId,ledgerObservation,
             coordinatorTxResources): continue
    rejected(reason): return rejected(other-exact-update-rejection(
      coordinator-ingress,reason))
    incoherent(proof): return incoherent(proof)
    unresolved(ids): return unresolved(ids)
    unsupported(ids): return unsupported(ids)
    resource-exhausted(report): return resource-exhausted(report)
    internal-failure(report): return internal-failure(report)
    // No noncomplete constructor exposes a publication capability.
  if recomputedTransactionId != Prepared.RequestIdentity:
    dispose the publication slice with its no-write proof
    return rejected(corrupt-prepared(request-identity-mismatch))
  if ledgerObservation contains update(Prepared.RequestIdentity)
     as ledgerEntry with an identical preparedBody:
    dispose the publication slice with its no-write proof
    return ledgerEntry.ImmutableNonduplicateResult
  if ledgerObservation contains update(Prepared.RequestIdentity)
     as ledgerEntry with a different UpdateRequestBody:
    dispose the publication slice with its no-write proof
    return rejected(other-exact-update-rejection(
      transaction-identity-reuse,details))
  commitResourceResult := ResolveReservedCommitValidationSlice(
    Prepared.UpdateRequestBody.UpdateResourceContractId)
  on rejected/unresolved/unsupported/resource-exhausted/internal-failure:
    dispose the coordinator publication slice with its no-write proof and return
      the matching exact nonpublication UpdateResult branch
  validation := BoundedValidatePreparedForCommit(
    Prepared,commitResourceResult.value):
    strictly decode every carried object and recompute
      SnapshotId = Identity(SnapshotBody)
    require HistoryRoot's parent, transaction, delta, transition, admission,
      pending, and rejection roots equal Prepared.UpdateRequestBody and report
    require candidate/PrepareProof commit the validated base and the exact
      ExpectedRuntimeState read-set, or the declaration-only proof when absent
    require Identity(AdmissionReport)=the report identity committed by HistoryRoot
    verify PrepareProof and require accept(ids) contains the exact statement
      binding every carried field
    for PreparedSealed:
      require cert.SnapshotId=candidate.SnapshotId,
        SealId=Identity(seal-certificate-domain,cert), requested-obligation
        coverage matches the request, and the exact seal statement was accepted
    for PreparedUnsealed:
      require the exact incomplete status is scoped to the candidate and the
        request's seal obligations and contains no completed seal claim
  match validation:
    valid: continue
    malformed/invalid-proof/field-mismatch:
      dispose the coordinator publication slice with its no-write proof; return
      rejected(corrupt-prepared(validation details)); publish nothing
    incoherent/unresolved/unsupported/resource-exhausted/internal-failure:
      dispose the coordinator publication slice with its no-write proof and return
      the matching exact nonpublication UpdateResult branch
  afterHead :=
    published(candidate SnapshotId, SealId) for PreparedSealed
    published(candidate SnapshotId, none)   for PreparedUnsealed
  updateReceiptBody := UpdateReceiptBody(
    Prepared.RequestIdentity,Root(Prepared.UpdateRequestBody),
    Prepared.UpdateRequestBody.ExpectedKnowledgeHead,afterHead,
    Root(Prepared.UpdateRequestBody.ExpectedRuntimeState) when present,
    candidate.SnapshotId,SealId for sealed or none,
    Identity(admission-report-domain,Prepared.AdmissionReport),
    CoordinatorAtomicityProfileId)
  updateReceipt := (updateReceiptBody,
    Identity(update-receipt-domain,updateReceiptBody))
  committedResult := committed-sealed(SealedSnapshot,updateReceipt) or
    committed-unsealed(UnsealedSnapshot,updateReceipt), matching Prepared
  updateEntry := BuildTransactionLedgerEntry(
    update(Prepared.RequestIdentity),Prepared.UpdateRequestBody,
    committedResult,updateReceiptBody)
  updatePublication : UpdateCommitPublicationResult :=
    BoundedAtomicPublishOrRecordUpdate(
      update(Prepared.RequestIdentity),Prepared.UpdateRequestBody,
      committedResult,updateEntry,afterHead,
      Prepared.UpdateRequestBody.ExpectedKnowledgeHead,
      Prepared.UpdateRequestBody.ExpectedRuntimeState,
      coordinatorTxResources.CoordinatorPublicationSlice;
      in one coordinator transition recheck the key and the complete
        knowledge/runtime read set; when both match, publish exactly the
        immutable candidate and any PreparedSealed seal, set KnowledgeHead :=
        afterHead, append updateEntry, and do not create, replace, or activate a
        DeploymentConfiguration; when the key remains absent but either head is
        stale, leave every head unchanged, construct conflictResult :=
        conflict(UpdateConflictReport(
          stale-commit-read-set,expected,the exact atomic actual state,
          update(Prepared.RequestIdentity),details)), and append
        BuildTransactionLedgerEntry(
          update(Prepared.RequestIdentity),Prepared.UpdateRequestBody,
          conflictResult,none) in that same transition)
  match updatePublication:
    published(result): return result
    recorded-conflict(result): return result
    same-body-winner(original): return original
    identity-reuse(report): return rejected(report)
    warrant-violation(report): return internal-failure(FailureReport(
      update-atomic-publication-warrant-violation,report))
```

An explicit rebase is a new request binding the actual parent, original
transition/delta identities, and a new deterministic rebase/merge transition.
It reruns every step above. Reseal and activation use these exact transitions:

```text
PUBLISH_RESEAL(CoordinatorState C, Request, candidate):
  resealIngressResult : CoordinatorTransactionIngressResult :=
    BoundedCoordinatorTransactionIngress(
    reseal,Request,Request.ResealTransactionId,
    CoordinatorTransactionIngressProfile,C)
  match resealIngressResult:
    complete(resealBody,recomputedTransactionId,ledgerObservation,
             coordinatorTxResources): continue
    rejected/incoherent/unresolved/unsupported/resource-exhausted/internal-failure:
      return the matching exact no-publication ResealResult; this branch exposes
        no publication capability
  if Request.ResealTransactionId != recomputedTransactionId:
    dispose the publication slice with its no-write proof
    return rejected(malformed-reseal-transaction-identity)
  if ledgerObservation contains reseal(Request.ResealTransactionId)
     as ledgerEntry with the same body: return duplicate(
       ledgerEntry.ResultId,ledgerEntry.ImmutableNonduplicateResult)
    after disposing the publication slice with its no-write proof
  if ledgerObservation contains that key as ledgerEntry with a different body:
    dispose the publication slice with its no-write proof; return rejected(identity-reuse)
  RECORD_RESEAL_TERMINAL(result,receiptBody?):
    consume coordinatorTxResources.CoordinatorPublicationSlice
    atomically require reseal(Request.ResealTransactionId) remains absent,
      validate every head precondition embedded by result; set committedResult
      to result if they hold and otherwise to
      conflict(the exact winning KnowledgeHead); leave KnowledgeHead unchanged,
      and append
      BuildTransactionLedgerEntry(
        reseal(Request.ResealTransactionId),resealBody,
        committedResult,receiptBody only when committedResult=result)
    on a same-body winning entry `winningEntry` return duplicate(
      winningEntry.ResultId,winningEntry.ImmutableNonduplicateResult)
    on a different-body winner return rejected(identity-reuse)
    return committedResult
  // After the replay check, every terminal nonpublication return below is
  // routed through RECORD_RESEAL_TERMINAL. Thus incomplete, rejected,
  // incoherent, unresolved, unsupported, exhausted, internal-failure, and
  // conflict results are idempotent rather than recomputed on replay.
  if C.KnowledgeHead != Request.ExpectedKnowledgeHead:
    return RECORD_RESEAL_TERMINAL(conflict(C.KnowledgeHead),none)
  if candidate.SnapshotId != Request.SnapshotId or
     C.KnowledgeHead's SnapshotId != Request.SnapshotId:
    return RECORD_RESEAL_TERMINAL(rejected(snapshot-scope-mismatch),none)
  resealResourceResult := ResolveExactResealResourceContract(
    Request.ResealResourceContractId)
  on rejected/unresolved/unsupported/resource-exhausted/internal-failure:
    return RECORD_RESEAL_TERMINAL(
      the matching exact ResealResult branch,none)
  resolved ResourceContract := resealResourceResult.value
  match Request.ResealWork:
    fresh(root):
      evidenceResult := BoundedResolveImmutableObject(
        root,resolved ResourceContract,
        SealCheckpointContext(
          reseal(Request.ResealTransactionId,Request.ResealTransactionId),
          candidate.SnapshotId,Root(Request.RequestedSealObligations),
          Request.ResealResourceContractId,evidence-resolution,
          exact processed/pending/prior-output/implementation/verifier/progress
            roots))
      match evidenceResult:
        complete(postIdEvidence):
          sealResult := VERIFY_SEAL(
            reseal(Request.ResealTransactionId,Request.ResealTransactionId),
            candidate,postIdEvidence,Request.RequestedSealObligations,
            resolved ResourceContract)
        checkpoint(P,ids): sealResult := checkpoint(P,ids)
        malformed/incoherent/unresolved/unsupported/exhausted/failure:
          return RECORD_RESEAL_TERMINAL(
            the matching exact ResealResult branch,none)
    resume(originalTx,checkpointRoot,continuationToken):
      sealResult := RESUME_VERIFY_SEAL(
        checkpointRoot,continuationToken,
        expectedOriginResealTransactionId=originalTx,
        newScope=reseal(originalTx,Request.ResealTransactionId),
        candidate,Request.SnapshotId,Request.RequestedSealObligations,
        resolved ResourceContract)
      // It resumes exactly the checkpoint's SealSubstage and committed work,
      // proof, prior-output, verifier, and progress roots; it never restarts
      // from postIdEvidence.
  if sealResult is checkpoint(P,ids):
    return RECORD_RESEAL_TERMINAL(checkpoint(P,ids),none)
  if sealResult is not sealed:
    return RECORD_RESEAL_TERMINAL(the matching exact branch,none)
  newHead := published(candidate.SnapshotId, sealResult.SealId)
  resealReceiptBody := ResealReceiptBody(
    Request.ResealTransactionId,Root(resealBody),
    Request.ExpectedKnowledgeHead,newHead,candidate.SnapshotId,
    sealResult.SealId,Identity(seal-certificate-domain,sealResult.Certificate),
    CoordinatorAtomicityProfileId)
  resealReceipt := (resealReceiptBody,
    Identity(reseal-receipt-domain,resealReceiptBody))
  publishedResult := published-reseal(
    newHead,sealResult.Certificate,resealReceipt)
  consume coordinatorTxResources.CoordinatorPublicationSlice in one indivisible
    AtomicPublishOrRecordReseal operation:
      if the key is absent and C.KnowledgeHead=Request.ExpectedKnowledgeHead,
        set C.KnowledgeHead:=newHead and append BuildTransactionLedgerEntry(
          reseal(Request.ResealTransactionId),resealBody,
          publishedResult,resealReceiptBody), then return publishedResult
      if a same-body entry `winningEntry` won, return duplicate(
        winningEntry.ResultId,winningEntry.ImmutableNonduplicateResult)
      if a different-body entry won, return rejected(identity-reuse)
      otherwise leave the head unchanged, append the exact conflict result under
        the still-absent key, and return conflict(the exact winning head)

ACTIVATE_SEAL(CoordinatorState C, Request, sealedSnapshot, migrationProfile):
  activationIngressResult : CoordinatorTransactionIngressResult :=
    BoundedCoordinatorTransactionIngress(
    activation,Request,Request.ActivationTransactionId,
    CoordinatorTransactionIngressProfile,C)
  match activationIngressResult:
    complete(activationBody,recomputedTransactionId,ledgerObservation,
             coordinatorTxResources): continue
    rejected/incoherent/unresolved/unsupported/resource-exhausted/internal-failure:
      return the matching exact no-publication ActivationResult; this branch
        exposes no publication capability
  if recomputedTransactionId != Request.ActivationTransactionId:
    dispose the publication slice with its no-write proof
    return rejected(malformed-activation-transaction-identity)
  if ledgerObservation contains
       activation(Request.ActivationTransactionId)
     as ledgerEntry with activationBody: return duplicate(
       ledgerEntry.ResultId,ledgerEntry.ImmutableNonduplicateResult)
    after disposing the publication slice with its no-write proof
  if ledgerObservation contains that key as ledgerEntry with a different body:
    dispose the publication slice with its no-write proof; return rejected(identity-reuse)
  RECORD_ACTIVATION_TERMINAL(result,receiptBody?):
    consume coordinatorTxResources.CoordinatorPublicationSlice
    atomically require activation(Request.ActivationTransactionId) remains
      absent, observe the exact current lineage head in this same transition,
      and validate every mutable-head predicate used to construct `result`;
      when `result` is conflict or any such mutable-head predicate fails,
      discard its advisory precheck projection and construct
      `committedResult := conflict(the exact atomic current head)`; otherwise
      set `committedResult := result`; leave every deployment head unchanged and append
      BuildTransactionLedgerEntry(
        activation(Request.ActivationTransactionId),activationBody,
        committedResult,
        receiptBody only when committedResult=result)
    on same-body race with exact `winningEntry` return duplicate(
      winningEntry.ResultId,winningEntry.ImmutableNonduplicateResult); on
      different-body race return rejected(identity-reuse)
    on append success return committedResult
  // Every later nonpublication terminal branch is routed through this helper.
  // Only an uncommitted transaction resolves mutable or externally supplied
  // dependencies; replay above never depends on their continued availability.
  activationResourceResult := ResolveExactActivationResourceContract(
    Request.ActivationResourceContractId)
  on rejected/unresolved/unsupported/exhausted/failure:
    return RECORD_ACTIVATION_TERMINAL(
      the matching exact nonpublication branch,none)
  resolved ResourceContract := activationResourceResult.value
  activationValidation := BoundedValidateActivationInputs(
    Request,sealedSnapshot,migrationProfile,resolved ResourceContract,
    strict consume-all decoding, canonical identities,
    exact Request.SnapshotId/SealId/MigrationProfileId,
    accepted exact seal statement and migration profile scope)
  match activationValidation:
    complete(validatedSealedSnapshot,validatedMigrationProfile): continue
    malformed/incomplete/incoherent/unresolved/unsupported/resource-exhausted/internal-failure:
      return RECORD_ACTIVATION_TERMINAL(
        the matching exact nonpublication branch,none)
  resolve the target lineage/create policy and current deployment head
  expectedHead := available(Request.ExpectedDeploymentConfiguration), with exact
    equality including configuration epoch, retained/effect roots, and
    RuntimePolicyStateRoot; for a create policy expectedHead := absent
  if current head != expectedHead: return RECORD_ACTIVATION_TERMINAL(
    conflict(the exact current head),none)
  activationPlanResult := BoundedBuildAndVerifyActivation(
    Request,validatedSealedSnapshot,validatedMigrationProfile,
    the exact current head,resolved ResourceContract;
    seal executable for the target capability,
    every migration semantic/state/effect/quarantine obligation,
    complete charged successor Dnew and exact MigrationReceipt,
    pure/isolated staging with no observable pre-CAS effect)
  match activationPlanResult:
    complete(Dnew,migrationReceipt): continue
    rejected/incomplete/incoherent/unresolved/unsupported/resource-exhausted/internal-failure:
      return RECORD_ACTIVATION_TERMINAL(
        the matching exact nonpublication branch,none)
  require migrationReceipt has exactly `MigrationReceipt`'s typed body and
    identity and binds Request, current head, Dnew, and every verified migration
    evidence/charge root
  activationReceiptBody := ActivationReceiptBody(
    Request.ActivationTransactionId,Root(activationBody),
    current head,Dnew,Request.SnapshotId,Request.SealId,
    migrationReceipt.MigrationReceiptId,CoordinatorAtomicityProfileId)
  activationReceipt := (activationReceiptBody,
    Identity(activation-receipt-domain,activationReceiptBody))
  activatedResult := activated(Dnew,activationReceipt)
  consume coordinatorTxResources.CoordinatorPublicationSlice in one indivisible
    AtomicPublishOrRecordActivation operation:
      if the key is absent and the full deployment head equals expectedHead,
        publish Dnew, append BuildTransactionLedgerEntry(
          activation(Request.ActivationTransactionId),activationBody,
          activatedResult,activationReceiptBody), and store the exact
          migrationReceipt body under its `migration-receipt-domain` identity;
        return activatedResult
      if a same-body entry `winningEntry` won, return duplicate(
        winningEntry.ResultId,winningEntry.ImmutableNonduplicateResult)
      if a different-body entry won, return rejected(identity-reuse)
      otherwise preserve the concurrent deployment/effects, append the exact
        nonpublication conflict under the still-absent key, and return
        conflict(the exact winning deployment head)
```

A `PreparedUnsealed` commit is knowledge publication only and can never activate
execution. A reseal never changes the candidate body or `SnapshotId`; activation
never changes the knowledge head.

Query resolution is non-executing and returns the total answer algebra:

```text
ResolutionScope = query(QueryId) | use-case(UseCaseRequestId)

ResolutionRequestBody = (
  SnapshotId, SealId, ResolutionScope, ResolutionResourceContractId,
  ResolutionWork
)

ResolutionWork =
    fresh
  | resume(OriginResolutionRequestId,
           ResolverCheckpointRoot,
           ResolverContinuationToken).

ResolutionRequestId = Identity(
  resolution-request-domain, ResolutionRequestBody)

ResolutionRequest = (ResolutionRequestId, ResolutionRequestBody).

CertificationScope =
    query-family(QueryFamilyId, CapabilityClass)
  | use-case-class(UseCaseClassId, CapabilityClass)
  | input-total-capability(
      InputProfileId,
      UnderlyingCapabilityScope : query-family(QueryFamilyId,CapabilityClass) |
                                  use-case-class(UseCaseClassId,CapabilityClass),
      UnderlyingCapabilityCertificateId)

CertificationRequestBody = (
  SnapshotId, SealId, CertificationScope,
  CertificationResourceContractId
)

CertificationRequestId = Identity(
  certification-request-domain, CertificationRequestBody)

CertificationRequest = (CertificationRequestId, CertificationRequestBody).

ResolutionStageResult(A) =
    complete(A)
  | checkpoint(ResolverCheckpoint, ObligationIds)
  | rejected(Reason)
  | incoherent(ConflictProof)
  | unresolved(DependencyIds)
  | unsupported(FeatureIds)
  | resource-exhausted(ResourceReport)
  | internal-failure(FailureReport)

CertificationCheckpointBody = (
  CertificationRequestId, SnapshotId, SealId, CertificationScope,
  CertificationStageTag, ProcessedWorkRoot, PendingWorkRoot,
  ImmutablePriorOutputRoots, ImplementationId, VerifierId,
  ProgressMeasureAndWarrant
)

CertificationStageTag =
    scope-validation | quantified-proof-verification |
    certificate-construction

CertificationCheckpointRoot = Identity(
  certification-checkpoint-domain, CertificationCheckpointBody)

PendingFamilyProof = family(
  CertificationCheckpointBody, CertificationCheckpointRoot)

PendingClassProof = class(
  CertificationCheckpointBody, CertificationCheckpointRoot)

PendingInputTotalCapabilityProof = input-total-capability(
  CertificationCheckpointBody,CertificationCheckpointRoot,
  InputProfileId,UnderlyingCapabilityCertificateId)

UnderlyingCapabilityCertificateId =
    family(FamilyCertificate.CertificateId)
  | use-case-class(ClassCertificate.CertificateId).

InputTotalCapabilityProofCore = (
  SharedSolverCertificateConstructorResourceImplementationAndVerifierIds,
  FourWayClassifierDisjointCoverAndTerminationStatementId,
  ValidConstructorCarrierCoverageAndCapabilityTheoremId,
  InvalidUnresolvedUnsupportedExactStatusTheoremId,
  CombinedResourceAndUniformTerminationTheoremId,
  ExactClassifierConstructorSolverCertificateAndVerifierProofObject,
  ProofObjectId
).

InputTotalCapabilityProof = InputTotalCapabilityProofCore.

InputTotalCapabilityStatementBody = (
  ExactCommonCertificateBinding,
  ExactInputTotalCapabilityBinding,
  ExactInputTotalCapabilityCertificateBody =
    (ExactCommonCertificateBinding,ExactInputTotalCapabilityBinding),
  ExactInputTotalCapabilityCertificateId = Identity(
    result-certificate-domain,ExactInputTotalCapabilityCertificateBody),
  CertificationRequestId,SnapshotId,SealId,InputProfileId,
  UnderlyingCapabilityCertificateId,
  UnderlyingAcceptedCapabilityStatementId,
  SharedSolverCertificateConstructorResourceImplementationAndVerifierIds,
  FourWayClassifierDisjointCoverAndTerminationStatementId,
  ValidConstructorCarrierCoverageAndCapabilityTheoremId,
  InvalidUnresolvedUnsupportedExactStatusTheoremId,
  CombinedResourceAndUniformTerminationTheoremId
).

`InputTotalCapabilityStatementBody` is the exact custom proposition of its
displayed standard certificate body. Its common/scope projections MUST equal
that body's two members; the request, snapshot, seal, input profile, underlying
capability scope/certificate/accepted statement, and every displayed shared-
identity/theorem field MUST equal the corresponding projections of that exact scope
binding and its embedded `InputTotalCapabilityProofCore`. A body with a changed
hypothesis set, universe scope, capability class, solver, resource envelope, or
proof core has no inhabitant.

ExactInputTotalCapabilityProposition(body) = body
  when body is an `InputTotalCapabilityStatementBody` satisfying all displayed
  dependent equalities.

RequiredInputTotalCapabilityStatementId = Identity(
  input-total-capability-statement-domain,
  ExactInputTotalCapabilityProposition(InputTotalCapabilityStatementBody)).

InputTotalCapabilityCertificate = (
  InputTotalCapabilityStatementBody,
  CertificateBody =
    InputTotalCapabilityStatementBody.ExactInputTotalCapabilityCertificateBody,
  CertificateId =
    InputTotalCapabilityStatementBody.ExactInputTotalCapabilityCertificateId =
      Identity(result-certificate-domain,CertificateBody),
  RequiredInputTotalCapabilityStatementId = Identity(
    input-total-capability-statement-domain,
    ExactInputTotalCapabilityProposition(InputTotalCapabilityStatementBody)),
  VerifierResult=accept(VerifiedStatementIds containing
                        RequiredInputTotalCapabilityStatementId)
).

InputTotalCapabilityCertificateDraft = (
  InputTotalCapabilityStatementBody,
  CertificateBody =
    InputTotalCapabilityStatementBody.ExactInputTotalCapabilityCertificateBody,
  CertificateId =
    InputTotalCapabilityStatementBody.ExactInputTotalCapabilityCertificateId =
      Identity(result-certificate-domain,CertificateBody),
  RequiredInputTotalCapabilityStatementId = Identity(
    input-total-capability-statement-domain,
    ExactInputTotalCapabilityProposition(InputTotalCapabilityStatementBody))
).

ResolvedInputTotalCertificationInputs = (
  InputProfile,UnderlyingCapabilityCertificate : FamilyCertificate | ClassCertificate,
  UnderlyingAcceptedCapabilityStatementId,
  ExactSharedScopeAndIdentityValidationEvidence,
  RemainingCertificationResourceContract
).

InputTotalCapabilityCertificateId = InputTotalCapabilityCertificate.CertificateId.

InputTotalCapabilityCertificationResult =
    certified(InputTotalCapabilityCertificate)
  | incomplete(PendingInputTotalCapabilityProof?,ObligationIds)
  | request-result(RequestStatus,Diagnostics)
  | verifier-result(VerifierResult)
  | incoherent(ConflictProof)
  | unresolved(DependencyIds)
  | unsupported(FeatureIds)
  | resource-exhausted(ResourceReport)
  | internal-failure(FailureReport).

CertificationStageResult(A) =
    complete(A)
  | checkpoint(PendingFamilyProof | PendingClassProof |
               PendingInputTotalCapabilityProof, ObligationIds)
  | rejected(Reason)
  | incoherent(ConflictProof)
  | unresolved(DependencyIds)
  | unsupported(FeatureIds)
  | resource-exhausted(ResourceReport)
  | internal-failure(FailureReport).
```

The resolution resource contract is a finite host proof-search/decoding budget,
not `X`, `M`, or a competitor resource envelope. It fixes checkpoint policy and
is bound to the snapshot, seal, and semantic request. Every classification,
coverage construction, optimization proof, complete-set extraction, policy
selection, certificate construction, identity computation, and proof check in
the two procedures below consumes the remaining contract through a `Bounded*`
interface. After semantic request validity has been established, a resumable
exhaustion returns the applicable `Incomplete` answer and an
`ExactIncompleteReport` committing the resolution-request identity and
checkpoint root. Before validity is established, it returns
`request-result(unresolved,resolver-checkpoint(...))`, because no well-typed
answer exists yet. Nonresumable exhaustion returns `resource-exhausted`. No
helper performs an unbounded search outside that orchestration.
`CertificationRequest` provides the corresponding bootstrap bound for a family,
class, or input-total-capability conjunction before that object is decoded. Its
contract MUST equal or be a
proved restriction of the family/class solver resource envelope; a larger
caller-supplied budget cannot strengthen the certified capability.
Every bounded proof-search, construction, or admission call in query/use-case
resolution returns `ResolutionStageResult`; the corresponding quantified
certification calls return `CertificationStageResult`, while an exact proof
checker returns `VerifierResult`. Thus `checkpoint` never ambiguously denotes an
update `PendingUpdate`, and every checkpoint is pinned to the exact request and
scope that can resume it.

`LiftResolutionStage` is total: before request validity, `rejected`,
`incoherent`, `unresolved`, and `unsupported` become `request-result` with the
corresponding `RequestStatus` and exact diagnostics; after validity, a resumable
checkpoint becomes the typed incomplete answer/report described above, while
the same terminal tags become the matching `ResolutionResult` request or
verifier branch. `resource-exhausted` and `internal-failure` map to their
homonymous `ResolutionResult` branches at either phase. A raw stage tag is never
returned as a resolution result.

For `ResolutionWork=fresh`, the origin and current request identities are both
the fresh `ResolutionRequestId`. For `resume(origin,checkpointRoot,token)`, the
resolver strictly decodes the checkpoint and requires its origin identity,
snapshot, seal, query/use-case scope, stage tag, implementation, verifier,
immutable prior outputs, and progress warrant to match. The token authorizes
only the next transition of that exact checkpoint under the newly bound current
request and resource continuation; it cannot retarget another snapshot, scope,
or solver. A resumed checkpoint carries the same origin and the new current
request identity.

`RESUME_RESOLUTION(S,Request,scopeObject)` is the same stage machine as the
corresponding fresh resolver, beginning at the checkpoint's exact
`ResolverStageTag`, processed/pending work, and prior-output roots. It does not
repeat earlier stages or discard their charged work. It returns the same
`ResolutionResult` algebra and may emit a strictly advancing checkpoint. A
different scope, profile, verifier, or resource semantics requires a fresh
request; a larger budget may be supplied only through the checkpoint's declared
resource-continuation rule.

```text
RESOLVE_QUERY(S : SealedSnapshot, Request : ResolutionRequest, q):
  verify the fixed-size ResolutionRequestId and require Request body binds
    S.SnapshotId, S.SealId, and query(q.QueryId)
  resolve exactly Request.ResolutionResourceContractId
  on invalid/unresolved/unsupported/resource-exhausted/internal-failure:
    return the matching ResolutionResult branch
  if Request.ResolutionWork is resume(...):
    return RESUME_RESOLUTION(S,Request,q)
  require Request.ResolutionWork=fresh
  classificationResult := BoundedClassifyCommittedQuery(S,q,remaining contract)
  match classificationResult:
    complete(classification): continue
    checkpoint(P,ids):
      return request-result(unresolved,resolver-checkpoint(P,ids))
    rejected/incoherent/unresolved/unsupported:
      return request-result(the exact RequestStatus,diagnostics)
    resource-exhausted/internal-failure: return the matching result branch

  derivationResult := BoundedDeriveAndIdentityCheck(
                  Problem_q, entire InvocationScope_q, X_q, M_q,
                  ClaimRequest_q, RequestedAnswerShape(q),remaining contract)
  match derivationResult:
    complete(derivation): continue
    checkpoint(P,ids):
      return request-result(unresolved,resolver-checkpoint(P,ids))
    rejected/incoherent/unresolved/unsupported/resource-exhausted/internal-failure:
      return the matching tagged branch
  if q.SystemUniverseId != derivation.ExpectedSystemUniverseId:
    return request-result(invalid, system-universe-identity-mismatch)
  sealCoverageResult := BoundedCheckSealCoverage(
    S,q,its exact requested claim,remaining contract)
  match sealCoverageResult:
    complete(sealCoverage): continue
    checkpoint(P,ids):
      answer := Incomplete(RequestedClaimScope(q),ids)
      return resolved-query(valid,answer,
        BuildExactIncompleteReport(
          Request,S,q,answer,checkpoint(P),ids))
    rejected/incoherent/unresolved/unsupported/resource-exhausted/internal-failure:
      return the matching exact ResolutionResult branch
  if sealCoverage proves not covered:
    answer := Incomplete(RequestedClaimScope(q),
                         reseal-required or exact obligation ids)
    return resolved-query(valid,answer,
      BuildExactIncompleteReport(
        Request,S,q,answer,no-checkpoint(reseal-required),
        answer.OutstandingObligationIds))

  admResult := BoundedCompleteApplicableAdmissionAndBehaviorCoverage(
                S, q, ApplicableAdm_S(q,X_q), entire InvocationScope_q,
                classification/derivation/sealCoverage proof roots,
                remaining contract)
  match admResult:
    complete(admProof): continue
    checkpoint(P,ids):
      answer := Incomplete(RequestedClaimScope(q),ids)
      return resolved-query(valid,answer,
        BuildExactIncompleteReport(
          Request,S,q,answer,checkpoint(P),ids))
    rejected/incoherent/unresolved/unsupported/resource-exhausted/internal-failure:
      return the matching exact ResolutionResult branch
  // An invocation supplied later to EXECUTE cannot narrow this family proof.
  answerResult := BoundedConstructAndAuthorizeQueryAnswer(
    S,q,admProof,M_q,RequestedAnswerShape(q),remaining contract,
    exact empty-carrier/negative semantics,
    scalar-member/complete-argmin/Pareto-member/complete-frontier/
      comparison-bound dispatcher,
    total charged selection policy when requested,
    ClaimRequest_q,UniverseScope satisfaction, ValidQueryAnswers and
      ExactRequestedQueryAnswers,
    exact authorized-fallback policy and discovered-set identity)
  match answerResult:
    complete(answer,answerContinuation): continue
    checkpoint(P,ids):
      answer := Incomplete(RequestedClaimScope(q),ids)
      return resolved-query(valid,answer,
        BuildExactIncompleteReport(Request,S,q,answer,checkpoint(P),ids))
    rejected/incoherent/unresolved/unsupported/resource-exhausted/internal-failure:
      return the matching exact ResolutionResult branch
  if answer is Incomplete:
    return resolved-query(valid,answer,
      BuildExactIncompleteReport(Request,S,q,answer,answerContinuation,
                                 answer.OutstandingObligationIds))
  require answer in ValidQueryAnswers_S(q)
  certResult := BoundedBuildClosedCertificateBodyAndStatement(
    CommonCertificateBinding(S),
    QueryBinding(q,answer,classification/derivation/sealCoverage proof roots),
    answer,
    remaining contract)
  match certResult:
    complete(cert,requiredStatement): continue
    checkpoint(P,ids):
      incompleteAnswer := Incomplete(RequestedClaimScope(q),ids)
      return resolved-query(valid,incompleteAnswer,
        BuildExactIncompleteReport(
          Request,S,q,incompleteAnswer,checkpoint(P),ids))
    rejected/incoherent/unresolved/unsupported/resource-exhausted/internal-failure:
      return the matching exact ResolutionResult branch
  match BoundedVerifyExactStatement(cert,requiredStatement,remaining contract):
    accept(ids) when requiredStatement in ids:
      return resolved-query(valid, answer, cert)
    accept(ids):
      return verifier-result(reject-invalid-proof(
        accepted-wrong-statement(ids,requiredStatement)))
    reject-malformed/reject-invalid-proof/unresolved/unsupported:
      return verifier-result(the exact VerifierResult)
    resource-exhausted(report): return resource-exhausted(report)
    internal-failure(report): return internal-failure(report)
```

`RESOLVE_USE_CASE` is the dependent workload analogue, not an ambient prose
shortcut:

```text
RESOLVE_USE_CASE(S : SealedSnapshot, Request : ResolutionRequest, u):
  verify the fixed-size ResolutionRequestId and require Request body binds
    S.SnapshotId, S.SealId, and use-case(u.UseCaseRequestId)
  resolve exactly Request.ResolutionResourceContractId
  on invalid/unresolved/unsupported/resource-exhausted/internal-failure:
    return the matching ResolutionResult branch
  if Request.ResolutionWork is resume(...):
    return RESUME_RESOLUTION(S,Request,u)
  require Request.ResolutionWork=fresh
  classificationResult := BoundedClassifyAndDeriveUseCase(
    S,u,remaining contract; resolve exactly W,X_W,M_W,QuantifierPrefix_W,
    ComparatorProfile_W,RequestedAnswerShape(u))
  match classificationResult:
    complete(classification,W,X_W,M_W): continue
    checkpoint(P,ids):
      return request-result(unresolved,resolver-checkpoint(P,ids))
    rejected/incoherent/unresolved/unsupported/resource-exhausted/internal-failure:
      return the matching tagged ResolutionResult
  universeValidationResult := BoundedVerifyUseCaseUniverseScope(
    u,classification,remaining contract;
    u.PolicyUniverseId equals the expected identity and every restricted-policy
    identity/membership predicate verifies)
  match universeValidationResult:
    complete(universeScopeProof): continue
    checkpoint(P,ids):
      answer := WorkloadIncomplete(RequestedClaimScope(u),ids)
      return resolved-use-case(valid,answer,
        BuildExactIncompleteReport(
          Request,S,u,answer,checkpoint(P),ids))
    rejected/incoherent/unresolved/unsupported/resource-exhausted/internal-failure:
      return LiftResolutionStage(the exact result,post-validity)
  sealCoverageResult := BoundedCheckSealCoverage(
    S,u,its exact requested claim,remaining contract)
  match sealCoverageResult:
    complete(sealCoverage): continue
    checkpoint(P,ids):
      answer := WorkloadIncomplete(RequestedClaimScope(u),ids)
      return resolved-use-case(valid,answer,
        BuildExactIncompleteReport(
          Request,S,u,answer,checkpoint(P),ids))
    rejected/incoherent/unresolved/unsupported/resource-exhausted/internal-failure:
      return the matching exact ResolutionResult branch
  if sealCoverage proves not covered:
    answer := WorkloadIncomplete(RequestedClaimScope(u),reseal-required)
    return resolved-use-case(valid,answer,
      BuildExactIncompleteReport(
        Request,S,u,answer,no-checkpoint(reseal-required),
        answer.OutstandingObligationIds))

  admResult := BoundedCompleteApplicableWorkloadAdmissionAndBehaviorCoverage(
    S,u,ApplicableWorkloadAdm_S(u),
    every (a,c) in BoundScenarioEnvironment_W,
    ScenarioEnvironmentSelectionProfile_W,
    complete trace relation/law, filtration and effects,
    classification/universeScope/sealCoverage proof roots,remaining contract)
  match admResult:
    complete(admProof): continue
    checkpoint(P,ids):
      answer := WorkloadIncomplete(RequestedClaimScope(u),ids)
      return resolved-use-case(valid,answer,
        BuildExactIncompleteReport(
          Request,S,u,answer,checkpoint(P),ids))
    rejected/incoherent/unresolved/unsupported/resource-exhausted/internal-failure:
      return the matching exact ResolutionResult branch
  answerResult := BoundedConstructAndAuthorizeWorkloadAnswer(
    S,u,W,M_W,admProof,RequestedAnswerShape(u),remaining contract,
    exact empty-carrier/negative semantics,
    scalar-member/complete-argmin/Pareto-member/complete-frontier/
      comparison-bound dispatcher,
    total charged workload-selection policy when requested,
    ClaimRequest_u,UniverseScope satisfaction, ValidAnswers and
      ExactRequestedAnswers,
    exact authorized-fallback policy and discovered-policy-set identity)
  match answerResult:
    complete(answer,answerContinuation): continue
    checkpoint(P,ids):
      answer := WorkloadIncomplete(RequestedClaimScope(u),ids)
      return resolved-use-case(valid,answer,
        BuildExactIncompleteReport(Request,S,u,answer,checkpoint(P),ids))
    rejected/incoherent/unresolved/unsupported/resource-exhausted/internal-failure:
      return the matching exact ResolutionResult branch
  if answer is WorkloadIncomplete:
    return resolved-use-case(valid,answer,
      BuildExactIncompleteReport(Request,S,u,answer,answerContinuation,
                                 answer.OutstandingObligationIds))
  certResult := BoundedBuildClosedCertificateBodyAndStatement(
    CommonCertificateBinding(S),
    WorkloadBinding(u,W,answer,
                    classification/universeScope/sealCoverage proof roots),answer,
    remaining contract)
  match certResult:
    complete(cert,requiredStatement): continue
    checkpoint(P,ids):
      incompleteAnswer := WorkloadIncomplete(RequestedClaimScope(u),ids)
      return resolved-use-case(valid,incompleteAnswer,
        BuildExactIncompleteReport(
          Request,S,u,incompleteAnswer,checkpoint(P),ids))
    rejected/incoherent/unresolved/unsupported/resource-exhausted/internal-failure:
      return the matching exact ResolutionResult branch
  match BoundedVerifyExactStatement(cert,requiredStatement,remaining contract):
    accept(ids) when requiredStatement in ids:
      return resolved-use-case(valid,answer,cert)
    accept(ids): return verifier-result(reject-invalid-proof(
      accepted-wrong-statement(ids,requiredStatement)))
    reject-malformed/reject-invalid-proof/unresolved/unsupported:
      return verifier-result(the exact VerifierResult)
    resource-exhausted/internal-failure: return the matching exact branch

CERTIFY_QUERY_FAMILY(S,Request : CertificationRequest,F,CapabilityClass,
                     SolverProfile,CandidateProofOrCheckpoint):
  verify fixed-size CertificationRequestId and scope =
    query-family(requested QueryFamilyId,CapabilityClass), bound to
    S.SnapshotId/SealId
  ResourceContract := ResolveFiniteCertificationResourceContract(
    Request.CertificationResourceContractId)
  on invalid/unresolved/unsupported/resource-exhausted/internal-failure:
    return the matching FamilyCertificationResult
  scopeValidationResult := BoundedValidateQueryFamilyCertificationScope(
    Request,F,CapabilityClass,SolverProfile,S,ResourceContract;
    strict consume-all decoding,
    capability is query-family-answer-complete or pointwise-envelope-complete,
    exact nonempty carrier/equality/membership/coverage and seal scope,
    exact solver/resource/universe-scope identities,
    F.QueryFamilyId=requested QueryFamilyId,
    SolverProfile=F.FamilySolverProfile, and ResourceContract equals or is a
      proved restriction of F.FamilyResourceEnvelope)
  match scopeValidationResult:
    complete(scopeProof): continue
    checkpoint(pendingFamilyProof,ids):
      return incomplete(pendingFamilyProof,ids)
    rejected/incoherent/unresolved/unsupported/resource-exhausted/internal-failure:
      return the matching FamilyCertificationResult
  proofResult := BoundedVerifyQueryFamilyCapabilityProof(
    CandidateProofOrCheckpoint,F,S,CapabilityClass,SolverProfile,
    scopeProof,remaining ResourceContract;
    every q's requested/returned scope satisfies F.FamilyUniverseScopeProfile,
    complete-universe advertising has every branch-specific omission bridge,
    pointwise-envelope-complete has complete-argmin/frontier shape,
    the same solver terminates within the envelope and returns
      ExactRequestedQueryAnswers_S(q),
    and the §10.3 complete-set/negative statement when applicable)
  match proofResult:
    complete(proof): continue
    checkpoint(pendingFamilyProof,ids):
      return incomplete(pendingFamilyProof,ids)
    rejected/incoherent/unresolved/unsupported/resource-exhausted/internal-failure:
      return the matching FamilyCertificationResult
  quantifiedClaim := ExactQueryFamilyCapabilityStatement(
    S,F,CapabilityClass,SolverProfile,proof)
  familyCertResult := BoundedBuildClosedCertificateBodyAndStatement(
    CommonCertificateBinding(S),
    QueryFamilyBinding(F,S,CapabilityClass,SolverProfile,scopeProof,proof),
    quantifiedClaim,remaining ResourceContract)
  match familyCertResult:
    complete(familyCert,requiredStatement): continue
    checkpoint(pendingFamilyProof,ids):
      return incomplete(pendingFamilyProof,ids)
    rejected/incoherent/unresolved/unsupported/resource-exhausted/internal-failure:
      return the matching FamilyCertificationResult
  match BoundedVerifyExactStatement(
          familyCert,requiredStatement,remaining ResourceContract):
    accept(ids) when requiredStatement in ids: return certified(familyCert)
    accept(ids): return verifier-result(reject-invalid-proof(
      accepted-wrong-statement(ids,requiredStatement)))
    reject-malformed/reject-invalid-proof/unresolved/unsupported:
      return verifier-result(the exact VerifierResult)
    resource-exhausted/internal-failure: return the matching exact branch

CERTIFY_USE_CASE_CLASS(S,Request : CertificationRequest,K,CapabilityClass,
                       SolverProfile,CandidateProofOrCheckpoint):
  verify fixed-size CertificationRequestId and scope =
    use-case-class(requested UseCaseClassId,CapabilityClass), bound to
    S.SnapshotId/SealId
  ResourceContract := ResolveFiniteCertificationResourceContract(
    Request.CertificationResourceContractId)
  on invalid/unresolved/unsupported/resource-exhausted/internal-failure:
    return the matching ClassCertificationResult
  scopeValidationResult := BoundedValidateUseCaseClassCertificationScope(
    Request,K,CapabilityClass,SolverProfile,S,ResourceContract;
    strict consume-all decoding,
    capability is use-case-class-answer-complete or use-case-class-complete,
    exact nonempty AdmittedDescriptors_(K,S),CoverageProposition,seal scope,
    UseCaseRequestConstructor_(K,S),K.UseCaseClassId=requested UseCaseClassId,
    SolverProfile=K.ClassSolverProfile and its bound classifier/constructor/
      solver/extractor/certificate/verifier/transition identities,
    ResourceContract equals or is a proved restriction of
      K.SolverResourceEnvelope)
  match scopeValidationResult:
    complete(scopeProof): continue
    checkpoint(pendingClassProof,ids):
      return incomplete(pendingClassProof,ids)
    rejected/incoherent/unresolved/unsupported/resource-exhausted/internal-failure:
      return the matching ClassCertificationResult
  proofResult := BoundedVerifyUseCaseClassCapabilityProof(
    CandidateProofOrCheckpoint,K,S,CapabilityClass,SolverProfile,
    scopeProof,remaining ResourceContract;
    for strong capability the constructor emits only full-scope attained
      scalar/Pareto members or identity-complete sets, never bounds/fallbacks/
      restricted/measured/best-known/heuristic requests;
    for every W in AdmittedDescriptors_(K,S), the same solver terminates within
      the envelope, constructs u=UseCaseRequestConstructor_(K,S)(W), and returns
      ExactRequestedAnswers_S(u) with a valid dependent WorkloadBinding)
  match proofResult:
    complete(proof): continue
    checkpoint(pendingClassProof,ids):
      return incomplete(pendingClassProof,ids)
    rejected/incoherent/unresolved/unsupported/resource-exhausted/internal-failure:
      return the matching ClassCertificationResult
  quantifiedClaim := ExactUseCaseClassCapabilityStatement(
    S,K,CapabilityClass,SolverProfile,proof)
  classCertResult := BoundedBuildClosedCertificateBodyAndStatement(
    CommonCertificateBinding(S),
    UseCaseClassBinding(K,S,CapabilityClass,SolverProfile,scopeProof,proof),
    quantifiedClaim,remaining ResourceContract)
  match classCertResult:
    complete(classCert,requiredStatement): continue
    checkpoint(pendingClassProof,ids):
      return incomplete(pendingClassProof,ids)
    rejected/incoherent/unresolved/unsupported/resource-exhausted/internal-failure:
      return the matching ClassCertificationResult
  match BoundedVerifyExactStatement(
          classCert,requiredStatement,remaining ResourceContract):
    accept(ids) when requiredStatement in ids: return certified(classCert)
    accept(ids): return verifier-result(reject-invalid-proof(
      accepted-wrong-statement(ids,requiredStatement)))
    reject-malformed/reject-invalid-proof/unresolved/unsupported:
      return verifier-result(the exact VerifierResult)
    resource-exhausted/internal-failure: return the matching exact branch

CERTIFY_INPUT_TOTAL_CAPABILITY(
    S,Request : CertificationRequest,InputProfile,
    capabilityCert : FamilyCertificate | ClassCertificate,
    CandidateProofOrCheckpoint):
  underlyingCertificateId :=
    family(capabilityCert.CertificateId) when capabilityCert is FamilyCertificate
    or use-case-class(capabilityCert.CertificateId) when it is ClassCertificate
  require Request.CertificationScope = input-total-capability(
    InputProfile.InputProfileId,
    the exact query-family/use-case-class scope of capabilityCert,
    underlyingCertificateId)
  inputValidationResult :
    CertificationStageResult(ResolvedInputTotalCertificationInputs) :=
    BoundedValidateInputTotalCertificationInputs(
      S,Request,InputProfile,capabilityCert,
      Request.CertificationResourceContractId;
      strictly validate Request/S/seal identities, the exact capability
      certificate and its accepted statement, and InputProfile's raw/typed
      domain, classifier, initialization, constructor, resource,
      implementation, and verifier roots)
  match inputValidationResult:
    complete(resolvedInputs):
      InputProfile := resolvedInputs.InputProfile
      capabilityCert := resolvedInputs.UnderlyingCapabilityCertificate
      remaining ResourceContract :=
        resolvedInputs.RemainingCertificationResourceContract
      continue
    checkpoint(pending,ids):
      require pending is the exact `PendingInputTotalCapabilityProof` for Request
      return incomplete(pending,ids)
    rejected(reason): return request-result(invalid,reason)
    incoherent(proof): return incoherent(proof)
    unresolved(ids): return unresolved(ids)
    unsupported(ids): return unsupported(ids)
    resource-exhausted(report): return resource-exhausted(report)
    internal-failure(report): return internal-failure(report)
  conjunctionResult : CertificationStageResult(InputTotalCapabilityProof) :=
    BoundedVerifyInputTotalCapabilityConjunction(
      Request,InputProfile,capabilityCert,
      resolvedInputs.UnderlyingAcceptedCapabilityStatementId,
      CandidateProofOrCheckpoint,remaining ResourceContract):
    prove the four-way classifier is total, terminating, and disjoint on every
      admitted raw input; every valid branch's exact constructor terminates,
      lands in capabilityCert's carrier, and invokes the same solver/certificate
      constructor/verifier within the combined envelope; every other branch
      returns its exact nonoptimization request status
  match conjunctionResult:
    complete(proof): continue
    checkpoint(pending,ids):
      require pending exactly pattern-matches
        PendingInputTotalCapabilityProof(
          checkpointBody,checkpointRoot,InputProfile.InputProfileId,
          the tagged exact certificate identity of capabilityCert),
        checkpointRoot=Identity(certification-checkpoint-domain,checkpointBody),
        and checkpointBody binds Request, the conjunction scope, current stage,
        work/prior-output roots, implementation/verifier, and strict progress
      return incomplete(pending,ids)
    rejected(reason): return request-result(invalid,reason)
    incoherent(proof): return incoherent(proof)
    unresolved(ids): return unresolved(ids)
    unsupported(ids): return unsupported(ids)
    resource-exhausted(report): return resource-exhausted(report)
    internal-failure(report): return internal-failure(report)
  require proof is an `InputTotalCapabilityProofCore` and contains no
    InputTotalCapabilityStatementBody, RequiredInputTotalCapabilityStatementId,
    input-total certificate body/ID, or verifier result
  inputTotalBinding := InputTotalBinding(
    InputProfile.InputProfileBody,proof's exact four-way totality,
    initialization and exact tagged admitted constructor)
  conjunctionBinding := InputTotalCapabilityBinding(
    inputTotalBinding,capabilityCert.CertificateBody.ScopeBinding,
    Request.CertificationRequestId,underlyingCertificateId,proof)
  exactCommonBinding := CommonCertificateBinding(S)
  conjunctionCertificateBody := (exactCommonBinding,conjunctionBinding)
  conjunctionCertificateId := Identity(
    result-certificate-domain,conjunctionCertificateBody)
  buildResult : CertificationStageResult(InputTotalCapabilityCertificateDraft) :=
    BoundedBuildInputTotalCapabilityCertificate(
      Request,exactCommonBinding,inputTotalBinding,conjunctionBinding,
      conjunctionCertificateBody,conjunctionCertificateId,capabilityCert,
      underlyingCertificateId,proof,remaining ResourceContract;
      construct the exact `InputTotalCapabilityStatementBody` whose displayed
      common binding, scope binding, standard body, and standard ID equal the
      four supplied values; project every other field exactly from Request,
      InputProfile, capabilityCert and proof; derive
      `RequiredInputTotalCapabilityStatementId`, and retain the complete custom
      proposition preimage)
  match buildResult:
    complete(conjunctionDraft):
      require conjunctionDraft.CertificateBody=conjunctionCertificateBody,
        conjunctionDraft.CertificateId=conjunctionCertificateId,
        conjunctionDraft.InputTotalCapabilityStatementBody.
          ExactCommonCertificateBinding=exactCommonBinding,
        conjunctionDraft.InputTotalCapabilityStatementBody.
          ExactInputTotalCapabilityBinding=conjunctionBinding,
        conjunctionDraft.InputTotalCapabilityStatementBody.
          ExactInputTotalCapabilityCertificateBody=conjunctionCertificateBody,
        conjunctionDraft.InputTotalCapabilityStatementBody.
          ExactInputTotalCapabilityCertificateId=conjunctionCertificateId,
        and Identity(result-certificate-domain,
                     conjunctionDraft.CertificateBody)=conjunctionCertificateId,
        and conjunctionDraft.RequiredInputTotalCapabilityStatementId=Identity(
          input-total-capability-statement-domain,
          ExactInputTotalCapabilityProposition(
            conjunctionDraft.InputTotalCapabilityStatementBody))
      requiredStatement := (
        conjunctionDraft.InputTotalCapabilityStatementBody,
        conjunctionDraft.RequiredInputTotalCapabilityStatementId)
      continue
    checkpoint(pending,ids):
      require pending is the exact `PendingInputTotalCapabilityProof` for
        Request,InputProfile.InputProfileId,underlyingCertificateId
      return incomplete(pending,ids)
    rejected(reason): return request-result(invalid,reason)
    incoherent(proof): return incoherent(proof)
    unresolved(ids): return unresolved(ids)
    unsupported(ids): return unsupported(ids)
    resource-exhausted(report): return resource-exhausted(report)
    internal-failure(report): return internal-failure(report)
  match BoundedVerifyExactStatement(
          conjunctionDraft,requiredStatement,remaining ResourceContract;
          recompute the standard CertificateBody/CertificateId and require the
          custom proposition to bind those exact values, including the complete
          common hypotheses and exact global/restricted capability scope):
    accept(ids) when
      conjunctionDraft.RequiredInputTotalCapabilityStatementId in ids and
      conjunctionDraft.RequiredInputTotalCapabilityStatementId = Identity(
        input-total-capability-statement-domain,
        ExactInputTotalCapabilityProposition(
          conjunctionDraft.InputTotalCapabilityStatementBody)) and
      conjunctionDraft.CertificateBody =
        conjunctionDraft.InputTotalCapabilityStatementBody.
          ExactInputTotalCapabilityCertificateBody and
      conjunctionDraft.CertificateId =
        conjunctionDraft.InputTotalCapabilityStatementBody.
          ExactInputTotalCapabilityCertificateId and
      conjunctionDraft.CertificateId = Identity(
        result-certificate-domain,conjunctionDraft.CertificateBody):
      conjunctionCert := InputTotalCapabilityCertificate(
        conjunctionDraft.InputTotalCapabilityStatementBody,
        conjunctionDraft.CertificateBody,conjunctionDraft.CertificateId,
        conjunctionDraft.RequiredInputTotalCapabilityStatementId,accept(ids))
      return certified(conjunctionCert)
    accept(ids): return verifier-result(reject-invalid-proof(
      accepted-wrong-statement(
        ids,conjunctionDraft.RequiredInputTotalCapabilityStatementId)))
    reject-malformed/reject-invalid-proof/unresolved/unsupported:
      return verifier-result(the exact VerifierResult)
    resource-exhausted(report): return resource-exhausted(report)
    internal-failure(report): return internal-failure(report)
```

Execution consumes, but never strengthens, a certified result:

```text
EXECUTE_CERTIFIED(CoordinatorState C, ExecutionRequest Request,
                  DeploymentConfiguration D, q, z, answer, cert):
  ingressResult : ExecutionIngressResult :=
    BoundedValidateFixedExecutionIngressHeader(
    C,Request,ResolveCommittedExecutionIngressProfile(
              Request.ExecutionIngressProfileId);
    strict consume-all fixed header, streaming request-body root and
      ExecutionTransactionId, ledger-key projection, run-contract locator,
      empty pre-reservation trace, and scoped NoEffectProof constructor)
  match ingressResult:
    complete(requestIdentity,preReservationTrace,noEffectConstructor,
             noRunPublicationToken,initialObservedHead,
             executionIngressAllocationReceipt): continue
    terminal-no-run(result): return result
  NO_EFFECT(observedHead) := BuildNoEffectProof(
    Request,available(Request.ExpectedDeploymentConfiguration),observedHead,
    preReservationTrace) using
      noEffectConstructor.PrimaryNoEffectProofToken and the remaining bootstrap
    envelope
  NO_EFFECT_RACE(observedHead) := BuildNoEffectProof(
    Request,available(Request.ExpectedDeploymentConfiguration),observedHead,
    preReservationTrace) using
      noEffectConstructor.RaceReplacementNoEffectProofToken and only the atomic
      `RECORD_NO_RUN` loss observation
  if requestIdentity != Request.ExecutionTransactionId:
    dispose noRunPublicationToken with its exact no-write proof
    return no-run(NoRunReport(
      request-result(invalid,malformed-execution-transaction-identity),
      optimization-incomplete,none,
      NO_EFFECT(initialObservedHead)))

  key := continuation-step(Request.ExecutionTransactionId)
           when Request.ExecutionSubject is continuation(...)
         else execution(Request.ExecutionTransactionId)
  preflightObservation : ExactExecutionKeyHeadObservation :=
    BoundedObserveExecutionKeyAndHead(
      key,Request.ExpectedDeploymentConfiguration.DeploymentLineageId,
      the remaining fixed execution-ingress observation envelope)
  replayValidation := ValidateExecutionReplayObservation(
    key,Request body,preflightObservation.TransactionLedgerEntry?,
    preflightObservation.DeploymentHeadState)
  match replayValidation:
    coherent-duplicate(entry): dispose noRunPublicationToken with no-write proof;
      return duplicate(
      entry.ResultId,entry.ImmutableNonduplicateResult)
    identity-reuse(entry): dispose noRunPublicationToken with no-write proof;
      return no-run(NoRunReport(
      request-result(invalid,execution-transaction-identity-reuse),
      optimization-incomplete,none,
      NO_EFFECT(preflightObservation.DeploymentHeadState)))
    atomicity-integrity-failure(entry,reserved(observedReservation)):
      dispose noRunPublicationToken with no-write proof; return integrity-failure(
      unresolved-runtime,optimization-incomplete,none,
      ExecutionIntegrityReport(atomicity-violation(
        key,Root(Request body),preflightObservation)),
      matching-reservation(
        observedReservation.EffectIntentLedgerRoot,
        observedReservation.EffectIntentLedgerObjectId,
        observedReservation.EffectIntentLedgerRetentionWarrantId,
        observedReservation.EffectIntentLedgerRetentionWarrantObjectId),
      reserved(observedReservation))
    absent(noEntry,head): continue

  preflightHead := preflightObservation.DeploymentHeadState
  if preflightHead is reserved(r) and
     r.OriginalTransactionKey = key and
     ExactRequestBodyEquals(r,Request excluding ExecutionTransactionId):
    dispose noRunPublicationToken with no-write proof; return recovery-required(
      Identity(execution-reservation-domain,r),
      BuildRecoveryRequestTemplate(r))
    // This check precedes all fallible validation: an existing run can never
    // be overwritten by a newly recorded NoEffectProof.
  if preflightHead is reserved(r) and r.OriginalTransactionKey = key:
    dispose noRunPublicationToken with no-write proof
    return no-run(NoRunReport(
      request-result(invalid,execution-transaction-identity-reuse),
      optimization-incomplete,none,NO_EFFECT(preflightHead)))

  RECORD_NO_RUN(result):
    consume noRunPublicationToken exactly once
    let noRunLossObservation denote the single
      `ExactExecutionKeyHeadObservation` returned by this atomic append attempt
      on any noncommit branch
    atomically require key is still absent and the target deployment head is
      not a reservation with the same transaction key; leave every deployment head
      unchanged and append
        BuildTransactionLedgerEntry(key,Request body,result,none)
      to TransactionLedgerMap
    if a same-key exact-body reservation exists or won the race, return
      recovery-required(Identity(execution-reservation-domain,the reservation),
                        BuildRecoveryRequestTemplate(the reservation))
      and write no ledger entry
    if a same-key different-body reservation exists or won the race, return
      no-run(NoRunReport(
        request-result(invalid,execution-transaction-identity-reuse),
        optimization-incomplete,none,
        NO_EFFECT_RACE(noRunLossObservation.DeploymentHeadState)))
      and write no ledger entry
    on a same-body concurrent ledger entry, call
      ValidateExecutionReplayObservation over that entry and the same atomic
      head observation; return duplicate only for coherent-duplicate, otherwise
      return the exact integrity-failure
    on a different-body concurrent entry return no-run(NoRunReport(
      request-result(invalid,execution-transaction-identity-reuse),
      optimization-incomplete,none,
      NO_EFFECT_RACE(noRunLossObservation.DeploymentHeadState)))
    return result

  runResource := ResolveFiniteRunResourceContract(Request.RunResourceContractId)
  on invalid/unresolved/unsupported/resource-exhausted/internal-failure:
    result := no-run(NoRunReport(the exact reason,
                                 optimization-incomplete,none,
                                 NO_EFFECT(preflightHead)))
    return RECORD_NO_RUN(result)

  executionGrantResult := BoundedAcquireFreshExecutionResourceGrant(
    C,C.ExecutionCapabilityIssuerStateMap,
    Request.ExecutionTransactionId,Root(Request body),
    Request.RunResourceContractId,Request.RecoveryResourceContractId,
    Request.RecoveryPolicyCoreId,
    ResolveCommittedExecutionIngressProfile(
      Request.ExecutionIngressProfileId).
      FreshExecutionAttemptAndResourceGrantIssuerProfileId)
  match executionGrantResult:
    granted(executionInvocationAttemptId,executionResourceGrantReceipt,
            executionResourceGrant): continue
    rejected/unresolved/unsupported/resource-exhausted/internal-failure:
      result := no-run(NoRunReport(
        ExactExecutionResourceGrantFailure(executionGrantResult),
        optimization-incomplete,none,NO_EFFECT(preflightHead)))
      return RECORD_NO_RUN(result)

  partitionResult : ExecutionResourcePartitionResult :=
    BoundedVerifyAndPartitionExecutionResource(
    runResource,Request.RecoveryResourceContractId,
    Request.ExecutionTransactionId,Root(Request body),
    Request.RecoveryPolicyCoreId,
    executionInvocationAttemptId,executionResourceGrantReceipt,
    executionResourceGrant)
  match partitionResult:
    complete(resourcePartition : ExecutionResourcePartition,
             resourcePartitionObjectId,partitionAndSufficiencyWarrant,
             consumedGrantProof,
             livePartitionDispositionCapability): continue
    any noncomplete branch(exactFailure,grantDispositionProof):
      result := no-run(NoRunReport(
        ExactResourcePartitionFailure(partitionResult),optimization-incomplete,
        none,NO_EFFECT(preflightHead)))
      return RECORD_NO_RUN(result)
  // FinalizationBudgetPartition and RecoveryBudgetPartition are now sequestered;
  // no ordinary stage below may draw from them.
  executionFinalizationPartition :=
    resourcePartition.FinalizationBudgetPartition
  DISPOSE_UNRESERVED_PARTITION(reason,capability):
    partitionDispositionProof := DisposeUnreservedExecutionPartition(
      resourcePartition,executionInvocationAttemptId,reason,
      capability whose id equals
        resourcePartition.UnreservedPartitionDispositionCapabilityId)
    require the proof affinely closes every still-unspent partition capability
      and proves no reservation acquired ownership; return the proof

  inputValidation := BoundedValidateExecutionInputs(
    Request,D,q,z,answer,cert,resourcePartition.InputValidationSlice,
    strict consume-all decoding and canonical typed identities,
    Request.ExpectedDeploymentConfiguration = D,
    Request.ExecutionSubject =
      query(q.QueryId,InvocationIdentity(q.QueryId,z)),
    Request.AnswerIdentity = AnswerIdentity(answer),
    Request.CertificateId = Identity(result-certificate-domain,cert.body),
    exactly one executable admitted complete selected system,
    Verify(cert)=accept(ids) containing RequiredCertificateStatementId(cert,answer),
    exact certificate binding to D.SnapshotId/D.SealId/q/X_q/M_q/system/mode,
    z in CorrectnessDomain_q and exact Init_q(z) runtime projection,
    requested-or-explicitly-authorized actual claim,
    nonquarantined realization-machine binding,
    return the exact retained `ExecProfileId` bound by the certificate and
      `OriginalExecutionObjectSet` for this selected system/policy)
  match inputValidation:
    complete(validatedInputs):
      (selected system,X_q,M_q,exactExecProfileId,actual claim,
       certificate OptimizationStatus) := validatedInputs
    rejected/incoherent/unresolved/unsupported/resource-exhausted/internal-failure:
      DISPOSE_UNRESERVED_PARTITION(
        inputValidation,livePartitionDispositionCapability)
      result := no-run(NoRunReport(
        ExactExecutionInputFailure(inputValidation),optimization-incomplete,
        none,NO_EFFECT(preflightHead)))
      return RECORD_NO_RUN(result)

  recoveryPolicyResult := BoundedResolveAndVerifyRecoveryPolicy(
    Request.RecoveryPolicyId,Request.RecoveryResourceContractId,
    X_q.EffectCommitModel,
    Request.RunResourceContractId,
    Root(resourcePartition),resourcePartitionObjectId,
    Identity(partition-and-sufficiency-warrant-domain,
             partitionAndSufficiencyWarrant),
    ExactOriginalExecutionObjectSet(
      Request,q,z,answer,cert,selected system,X_q,exactExecProfileId,
      every execution/outcome/completion/effect descriptor),
    resourcePartition.RecoveryPolicyVerificationSlice;
    construct and verify the exact OriginalExecutionObjectSet root/object and
      transitive retention warrant; require the resolved RecoveryBundleBody and
      RecoveryExecutionMaterial fields equal this original graph, this actual
      partition root/object/warrant, the run/recovery contracts, policy, effect
      model, authorization profile, and accepted fence/liveness statements;
    recompute RecoveryScheduleRoots from the actual RecoveryBudgetPartition and
      require its displayed identity equals the liveness statement's
      ResumeStageAttemptStageAdvanceCheckpointAndTailAttemptSchedulesRoot;
    resolve and accept the exact RecoveryScheduleStatePersistenceWarrant and
      require its acyclic core names these initial roots, the admitted storage
      profile, transition relation, lifetime, and durable fault domain)
  match recoveryPolicyResult:
    complete(recoveryPolicy):
      require recoveryPolicy.ResolvedRecoveryPolicy.RecoveryPolicyCoreId =
        Request.RecoveryPolicyCoreId and
        recoveryPolicy.ResolvedRecoveryPolicy.RecoveryPolicyId =
          Request.RecoveryPolicyId;
      require recoveryPolicy.VerifiedRecoveryBundle's material and every retained
        original-object/partition identity equal the exact values supplied above;
      continue
    rejected/incoherent/unresolved/unsupported/resource-exhausted/internal-failure:
      DISPOSE_UNRESERVED_PARTITION(
        recoveryPolicyResult,livePartitionDispositionCapability)
      result := no-run(NoRunReport(
        ExactRecoveryPolicyFailure(recoveryPolicyResult),
        optimization-incomplete,none,NO_EFFECT(preflightHead)))
      return RECORD_NO_RUN(result)

  FINISH_RESERVED(stagedOutcome,requiredRuntimeStatementBody,
                  requiredRuntimeStatement,runtimeCheck,
                  currentEffectProtocolScheduleState):
    finishOutcome : ExecutionFinalizeResult :=
      BoundedBuildAndAtomicallyFinalizeExecution(
      Request,key,currentReservation,stagedOutcome,
      requiredRuntimeStatementBody,requiredRuntimeStatement,runtimeCheck,
      currentEffectProtocolScheduleState,executionFinalizationPartition;
      require stagedOutcome's evaluated core is the one named by the exact
        statement and its published core is identical or the proved
        quarantine/partial conversion; construct/store the acyclic
        RuntimeVerificationEvidenceObjectGraph whose
        first projection is this exact `requiredRuntimeStatementBody`, and its
        accepted retention warrant; construct the record, final receipt embedding
        that complete record, `FinalizedExecutionResult`, and complete ledger
        entry; then atomically require the key absent and head exactly
        reserved(currentReservation), replace it by the staged successor and
        append the entry; affinely close the exact remaining materialized effect
        schedule and every untaken outcome/conformance sibling under their exact
        disposition warrants; if the
        compare loses, use the same single-use partition's
        mutually exclusive loss-validation budget to validate the one atomic
        observation and return an already classified winner/recovery/integrity
        constructor)
    match finishOutcome:
      committed(finalResult): return finalResult
      same-body-winner(originalId,original): return duplicate(
        originalId,original)
      recovery-owned(reservationId,recoveryTemplate): return recovery-required(
        reservationId,recoveryTemplate)
      integrity-failure(report,observedLedgerState,currentHead): return
        integrity-failure(unresolved-runtime,optimization-incomplete,none,
                          report,observedLedgerState,currentHead)
      warrant-violation(report,observation): return integrity-failure(
        unresolved-runtime,optimization-incomplete,none,
        ExecutionIntegrityReport(finalization-warrant-violation(
          Root(currentReservation),report)),
        ObservedEffectLedgerStateFrom(observation,currentReservation),
        observation.DeploymentHeadState)
    // No failure branch writes the execution key or asserts NoEffectProof.
    no state/effect/result field may be published outside this transition except
      external effects already governed by the committed EffectIntentLedger

  currentReservation := BuildExecutionReservation(
    Request,key,selected system,resourcePartition,X_q.EffectCommitModel,
    empty EffectIntentLedgerRoot,empty EffectIntentLedgerObjectId,
    empty EffectIntentLedgerRetentionWarrantId/ObjectId,
    recoveryPolicy.Identity,Request.RecoveryPolicyCoreId,
    Request.RecoveryResourceContractId,
    recoveryPolicy.VerifiedRecoveryBundle.Body/Root/ObjectId/
      RecoveryExecutionMaterialRoot/EmergencyTemplateId/EmergencyTemplateRoot/
      EmergencySafeQuarantineTemplateRetentionWarrantId/
      EmergencySafeQuarantineTemplateRetentionWarrantObjectId/
      RetentionWarrantId/RetentionWarrantObjectId;
    set ExecutionPartitionOwnershipState = unreserved-transfer-intent(
      resourcePartition.UnreservedPartitionDispositionCapabilityId);
    set RecoveryWorkRoot = none;
    set RecoveryWorkObjectId = none;
    set RecoveryWorkObjectGraphRetentionWarrantId = none;
    set RecoveryWorkObjectGraphRetentionWarrantObjectId = none;
    set RecoveryResumeAcquisitionScheduleRoot =
      Root(resourcePartition.RecoveryBudgetPartition.ResumeAcquisitionSchedule);
    set RecoveryResumeAcquisitionMarker = none;
    set RecoveryInvocationLease = none;
    set LatestAcceptedRecoveryTakeoverEvidenceRef = none;
    set RecoveryLeaseEpochCounter = 0;
    set RecoveryFaultCount = 0;
    set RecoveryStageAttemptMarker = none;
    set RecoveryTailAttemptScheduleRoot =
      Root(resourcePartition.RecoveryBudgetPartition.RecoveryTailAttemptSchedule);
    set RecoveryTailAttemptMarker = none;
    set ReservationPhase=running(
      the exact ExecutionFenceToken displayed in §7.11,
      the accepted LeaseOrQuiescenceWarrant whose projected statement equals
        ExecutionFenceSafetyStatement for that token))
  reservationCas : ExecutionReservationMutationResult :=
    BoundedAcquireExecutionReservation(
      key,Request body,D,currentReservation,
      livePartitionDispositionCapability,
      resourcePartition.ReservationAcquisitionSlice;
      construct/store every reservation/bundle/warrant identity, atomically
        require the key absent and head=available(D), and on loss validate the
        one observation into `ExecutionReservationMutationLoss`)
  match reservationCas:
    committed(committedReservation):
      require committedReservation is the unique successor of the draft whose
        ExecutionPartitionOwnershipState is reserved(the exact ownership proof)
        binding `ReservationOwnershipPreCoreRoot`, and whose
        ReservationStateRoot is recomputed;
      currentReservation := committedReservation;
      require currentReservation.ReservationPhase exactly pattern-matches
        running(currentExecutionFence,currentLeaseOrQuiescenceWarrant) and
        currentReservation.ExecutionPartitionOwnershipState exactly
          pattern-matches reserved(currentPartitionOwnershipProof), whose
        ExecutionInvocationAttemptId=executionInvocationAttemptId and
        ExecutionResourcePartitionCoreRoot=
          Root(resourcePartition.ExecutionResourcePartitionCore)
      currentEffectProtocolScheduleState :=
        MaterializeExecutionEffectProtocolSchedule(
          resourcePartition.EffectProtocolSchedule,
          executionInvocationAttemptId,Request.ExecutionTransactionId,
          currentExecutionFence,
          currentPartitionOwnershipProof.ReservationOwnershipPreCoreRoot)
      require its template-schedule root, attempt, transaction, fence, and
        ownership-pre-core projections equal those exact supplied values
      dispose noRunPublicationToken with its no-write proof;
      transfer every remaining partition capability to currentReservation;
      continue
    classified-loss(loss,returnedDispositionCapability):
      DISPOSE_UNRESERVED_PARTITION(loss,returnedDispositionCapability)
      match loss:
        same-body-winner(originalId,original):
          dispose noRunPublicationToken with no-write proof;
          return duplicate(originalId,original)
        recovery-owned(reservationId,recoveryTemplate):
          dispose noRunPublicationToken with no-write proof;
          return recovery-required(reservationId,recoveryTemplate)
        identity-reuse(currentHead):
          dispose noRunPublicationToken with no-write proof;
          return no-run(NoRunReport(
            request-result(invalid,execution-transaction-identity-reuse),
            optimization-incomplete,none,NO_EFFECT(currentHead)))
        ordinary-head-conflict(currentHead):
          return RECORD_NO_RUN(no-run(NoRunReport(
            conflict(currentHead),certificate OptimizationStatus,
            actual claim,NO_EFFECT(currentHead))))
        integrity-failure(report,observedLedgerState,currentHead):
          dispose noRunPublicationToken with no-write proof;
          return integrity-failure(
            unresolved-runtime,optimization-incomplete,none,
            report,observedLedgerState,currentHead)
    warrant-violation(report,observation,returnedDispositionCapability):
      DISPOSE_UNRESERVED_PARTITION(report,returnedDispositionCapability)
      dispose noRunPublicationToken with no-write proof
      return integrity-failure(
        unresolved-runtime,optimization-incomplete,none,
        ExecutionIntegrityReport(reservation-cas-invariant-violation(
          Root(currentReservation),observation)),
        ObservedEffectLedgerStateFrom(observation,currentReservation),
        observation.DeploymentHeadState)

  CLOSE_EXECUTION_EFFECT_SCHEDULE_AND_RETURN(result):
    scheduleDispositionProof := CloseExecutionEffectSchedule(
      currentEffectProtocolScheduleState,public-result(result))
    require the proof's schedule-state root equals
      Identity(materialized-execution-effect-protocol-schedule-state-domain,
               currentEffectProtocolScheduleState)
    return result

  CLOSE_SELECTED_EFFECT_AND_RETURN(
      selectedEffectToken,lastCompletedPhase,result):
    tokenDispositionProof := CloseExecutionEffectTokenSuffix(
      selectedEffectToken,lastCompletedPhase,public-result(result))
    require tokenDispositionProof names the exact selected token and every
      unconsumed suffix slice, and its `ExactTerminalOrRecoveryOwnedReason`
      equals `public-result(result)`
    return CLOSE_EXECUTION_EFFECT_SCHEDULE_AND_RETURN(result)

  execute the complete selected outer system under Exec_X and OutcomeModel_X
    within resourcePartition.RunWorkSlice; before every potentially visible
    effect:
    effectTokenSelection : ExecutionEffectProtocolTokenSelectionResult :=
      SelectNextExecutionEffectToken(currentEffectProtocolScheduleState)
    match effectTokenSelection:
      selected(selectedEffectToken,successorEffectScheduleState,
               successorEffectScheduleStateRoot):
        require successorEffectScheduleStateRoot equals the identity of that
          exact successor and the selected token's binding equals the current
          attempt, transaction, running fence, and ownership pre-core;
        currentEffectProtocolScheduleState := successorEffectScheduleState
      exhausted-before-declared-run-bound(warrant):
        return CLOSE_EXECUTION_EFFECT_SCHEDULE_AND_RETURN(recovery-required(
          Identity(execution-reservation-domain,currentReservation),
          BuildRecoveryRequestTemplate(currentReservation)))
      malformed-schedule(reason) | internal-failure(reason):
        return CLOSE_EXECUTION_EFFECT_SCHEDULE_AND_RETURN(recovery-required(
          Identity(execution-reservation-domain,currentReservation),
          BuildRecoveryRequestTemplate(currentReservation)))
    bind proposedEffectDescriptor : ProposedEffectDescriptor to the selected
      outer system's exact next proposed-effect output
    require Root(proposedEffectDescriptor)=Identity(
      proposed-effect-descriptor-domain,proposedEffectDescriptor) and every
      descriptor field is canonical and charged to the current RunWorkSlice;
      require its retained payload/proof-input objects have respectively
      `payload`/`authorization-proof-input` kind, their exact bodies rederive
      their roots and object IDs, and both accepted retention warrants bind the
      admitted reservation-through-recovery lifetime and fault domain
    effectAuthorizationStatement := EffectAuthorizationStatement(
      exactExecProfileId,
      recoveryPolicy.ResolvedRecoveryPolicy.RecoveryPolicyId,
      recoveryPolicy.EffectAuthorizationProfileId,
      Root(proposedEffectDescriptor),
      execution(currentExecutionFence),
      proposedEffectDescriptor.EffectTarget,
      proposedEffectDescriptor.RetainedEffectPayloadObject.
        ExactEffectObjectRoot,
      proposedEffectDescriptor.CommitMode,
      proposedEffectDescriptor.DeduplicationKey,
      proposedEffectDescriptor.ChargedCost,
      proposedEffectDescriptor.EffectOutcomeAndRecoveryClass)
    effectAuthorizationStatementId := Identity(
      effect-authorization-domain,effectAuthorizationStatement)
    authResult := BoundedVerifyEffectAuthorization(
      proposedEffectDescriptor,
      proposedEffectDescriptor.RetainedEffectPayloadObject,
      proposedEffectDescriptor.RetainedEffectAuthorizationProofInputObject,
      recoveryPolicy.EffectAuthorizationProfileId,
      effectAuthorizationStatement,effectAuthorizationStatementId,
      require the statement's EffectAuthorizationProfileId equals that exact
        committed profile; on acceptance construct/store the exact
        `EffectAuthorizationProofBody` and return its recomputed identity,
      selectedEffectToken.EffectAuthorizationSlice)
    match authResult:
      authorized(id,authorizationProofBody,authorizationProofId,ids)
          when id=effectAuthorizationStatementId and id in ids and
            authorizationProofBody.EffectAuthorizationStatementId=id and
            authorizationProofBody.EffectAuthorizationProfileId=
              recoveryPolicy.EffectAuthorizationProfileId and
            authorizationProofBody.ProposedEffectDescriptorRoot=
              Root(proposedEffectDescriptor) and
            authorizationProofBody.EffectAuthorizationProofInputRoot=
              proposedEffectDescriptor.
                RetainedEffectAuthorizationProofInputObject.
                  ExactEffectObjectRoot and
            authorizationProofBody.VerifierResult=accept(ids) and
            authorizationProofId=Identity(
              accepted-effect-authorization-proof-domain,
              authorizationProofBody):
        continue
      exact-unauthorized(proof):
        authorizationStopEvent := effect-authorization-violation(proof)
        tokenDispositionProof := CloseExecutionEffectTokenSuffix(
          selectedEffectToken,authorization,
          pre-intent-stop(Root(authorizationStopEvent)))
        set the exact observed event to authorizationStopEvent,
        retain all prior effects, emit none of this proposed effect, terminate
        the selected system, and jump to CLASSIFY_OBSERVED_EVENT; no private
        result builder or direct FINISH call is permitted
      authorized(other,otherProofBody,otherProofId,ids):
        authorizationStopEvent := effect-authorization-inconclusive(
          nonmatching-authorization-acceptance(
            other,otherProofBody,otherProofId,ids))
        tokenDispositionProof := CloseExecutionEffectTokenSuffix(
          selectedEffectToken,authorization,
          pre-intent-stop(Root(authorizationStopEvent)))
        set the exact observed event to authorizationStopEvent,
        retain all prior effects, emit none of this
        proposed effect, terminate the selected system, and jump to
        CLASSIFY_OBSERVED_EVENT; an accept set lacking the required identity is
        inconclusive, never authorization, and no private result builder is used
      inconclusive(verifierResult):
        authorizationStopEvent := effect-authorization-inconclusive(verifierResult)
        tokenDispositionProof := CloseExecutionEffectTokenSuffix(
          selectedEffectToken,authorization,
          pre-intent-stop(Root(authorizationStopEvent)))
        set the exact observed event to authorizationStopEvent,
        retain all prior effects, emit none of this proposed
        effect, terminate the selected system, and jump to
        CLASSIFY_OBSERVED_EVENT; no private result builder is used
    require currentReservation.ReservationPhase exactly pattern-matches
      running(currentExecutionFence,currentLeaseOrQuiescenceWarrant)
    effectIntentBody := EffectIntentBody(
      effectAuthorizationStatementId,authorizationProofId,
      execution(selectedEffectToken.
        MaterializedExecutionEffectProtocolTokenId),
      execution(currentExecutionFence),proposedEffectDescriptor.EffectTarget,
      proposedEffectDescriptor.RetainedEffectPayloadObject.
        ExactEffectObjectRoot,
      proposedEffectDescriptor.CommitMode,
      proposedEffectDescriptor.DeduplicationKey)
    intentId := Identity(effect-intent-domain,effectIntentBody)
    effectPermitBody := EffectPermitBody(
      intentId,effectAuthorizationStatementId,execution(currentExecutionFence),
      proposedEffectDescriptor.SinkIdentity,
      proposedEffectDescriptor.SinkDeduplicationOrLeaseToken,
      proposedEffectDescriptor.ExpiryOrQuiescenceCondition)
    permitId := Identity(effect-permit-domain,effectPermitBody)
    effectPermit := EffectPermit(effectPermitBody,permitId)
    effectIntentEntry := EffectIntentEntry(
      proposedEffectDescriptor,effectAuthorizationStatement,
      effectAuthorizationStatementId,authorizationProofBody,
      authorizationProofId,effectIntentBody,intentId,permitId,
      authorized-unconsumed,none)
    require the bound intent's `EffectProtocolTokenBindingId` equals
      execution(selectedEffectToken.
        MaterializedExecutionEffectProtocolTokenId); the bound permit carries
      that same value transitively through its exact `intentId`
    construct nextReservation by atomically appending the intent entry and
      unconsumed permit together, storing the new complete EffectIntentLedger
      object and retention warrant whose transitive graph contains the exact
      `proposedEffectDescriptor`, both complete retained effect objects and their
      accepted warrant objects, `effectAuthorizationStatement`,
      `authorizationProofBody`, `effectIntentBody`, and `effectPermitBody`, and
      replacing its exact
      root/object/warrant fields while preserving that fence; neither intent nor
      permit is active or externally consumable before this reservation CAS
      succeeds
    intentCas : PostReservationMutationResult :=
      BoundedAppendAuthorizedIntentAndPermit(
        key,Request body,currentReservation,nextReservation,intentId,
        selectedEffectToken.IntentAndPermitPublicationSlice;
        atomically store the successor ledger/reservation and, on compare loss,
          validate the one key/head observation into the closed loss algebra)
    match intentCas:
      committed(committedReservation):
        currentReservation := committedReservation; continue
      classified-loss(loss):
        match loss:
          same-body-winner(originalId,original): return
            CLOSE_SELECTED_EFFECT_AND_RETURN(
              selectedEffectToken,intent-publication,
              duplicate(originalId,original))
          recovery-owned(reservationId,recoveryTemplate): return
            CLOSE_SELECTED_EFFECT_AND_RETURN(
              selectedEffectToken,intent-publication,
              recovery-required(reservationId,recoveryTemplate))
          integrity-failure(report,observedLedgerState,currentHead): return
            CLOSE_SELECTED_EFFECT_AND_RETURN(
              selectedEffectToken,intent-publication,integrity-failure(
                unresolved-runtime,optimization-incomplete,none,
                report,observedLedgerState,currentHead))
      warrant-violation(report,observation): return
        CLOSE_SELECTED_EFFECT_AND_RETURN(
          selectedEffectToken,intent-publication,integrity-failure(
            unresolved-runtime,optimization-incomplete,none,
            ExecutionIntegrityReport(intent-cas-invariant-violation(
              intentId,Root(currentReservation),observation)),
            ObservedEffectLedgerStateFrom(observation,currentReservation),
            observation.DeploymentHeadState))
    sinkResult : ExecutionEffectSinkResult :=
      BoundedConsumeExecutionEffectPermit(
        currentReservation,effectIntentEntry,effectPermit,
        proposedEffectDescriptor,
        proposedEffectDescriptor.RetainedEffectPayloadObject,
        proposedEffectDescriptor.RetainedEffectAuthorizationProofInputObject,
        selectedEffectToken.SinkMembershipAndOutcomeCaptureSlice,
        selectedEffectToken.StatusPublicationAndLossValidationSlice;
        atomically validate that the exact current reservation contains this
        still-unconsumed intent/permit pair before the sink may consume it or emit
        the effect; capture the exact observed outcome in the same bounded phase;
        on every noncomplete sink outcome atomically persist the exact evidence or
        in-flight classification in a descendant using the status slice before
        returning)
    match sinkResult:
      complete(sinkConsumptionEvidence,exactOutcomeEvidence,
               statusPublicationSlice): continue
      recovery-owned(recoveryReservation,retainedIntent,retainedPermit,
                     observedEffectEvidenceRoot):
        require recoveryReservation is the exact validated descendant whose
          retained ledger contains retainedIntent,retainedPermit, and the exact
          evidence/in-flight classification;
        currentReservation := recoveryReservation;
        return CLOSE_EXECUTION_EFFECT_SCHEDULE_AND_RETURN(recovery-required(
          Identity(execution-reservation-domain,recoveryReservation),
          BuildRecoveryRequestTemplate(recoveryReservation)))
      same-body-winner(originalId,original): return
        CLOSE_EXECUTION_EFFECT_SCHEDULE_AND_RETURN(
          duplicate(originalId,original))
      integrity-failure(report,observedLedgerState,currentHead): return
        CLOSE_EXECUTION_EFFECT_SCHEDULE_AND_RETURN(integrity-failure(
          unresolved-runtime,optimization-incomplete,none,
          report,observedLedgerState,currentHead))
      warrant-violation(report,observation): return
        CLOSE_EXECUTION_EFFECT_SCHEDULE_AND_RETURN(integrity-failure(
          unresolved-runtime,optimization-incomplete,none,
          ExecutionIntegrityReport(status-cas-invariant-violation(
            intentId,Root(currentReservation),observation)),
          ObservedEffectLedgerStateFrom(observation,currentReservation),
          observation.DeploymentHeadState))
    construct outcomeReservation with that exact `effectIntentEntry`/
      `effectPermit` pair's
      sink-consumed and outcome-observed status/evidence, a newly stored complete
      ledger object, and retention warrant
    statusCas : PostReservationMutationResult :=
      BoundedRecordEffectOutcomeStatus(
        key,Request body,currentReservation,outcomeReservation,intentId,
        statusPublicationSlice;
        atomically publish that exact descendant and classify a losing
          key/head observation within the token)
    match statusCas:
      committed(committedReservation):
        currentReservation := committedReservation; continue execution with
          currentEffectProtocolScheduleState
      classified-loss(loss):
        match loss:
          same-body-winner(originalId,original): return
            CLOSE_EXECUTION_EFFECT_SCHEDULE_AND_RETURN(
              duplicate(originalId,original))
          recovery-owned(reservationId,recoveryTemplate): return
            CLOSE_EXECUTION_EFFECT_SCHEDULE_AND_RETURN(
              recovery-required(reservationId,recoveryTemplate))
          integrity-failure(report,observedLedgerState,currentHead): return
            CLOSE_EXECUTION_EFFECT_SCHEDULE_AND_RETURN(integrity-failure(
              unresolved-runtime,optimization-incomplete,none,
              report,observedLedgerState,currentHead))
      warrant-violation(report,observation): return
        CLOSE_EXECUTION_EFFECT_SCHEDULE_AND_RETURN(integrity-failure(
          unresolved-runtime,optimization-incomplete,none,
          ExecutionIntegrityReport(status-cas-invariant-violation(
            intentId,Root(currentReservation),observation)),
          ObservedEffectLedgerStateFrom(observation,currentReservation),
          observation.DeploymentHeadState))
    // After intent publication every sink/status noncompletion above either
    // retained the exact reservation for recovery or returned a classified
    // terminal winner. No path loses the intent, retries the effect, or leaves a
    // selected phase/schedule capability live.
  CLASSIFY_OBSERVED_EVENT:
  commit, compensate, deduplicate, or expose effects exactly as X declares
  classify the observed finite event under X's OutcomeModel and
    CompletionContract_P before proposing a RuntimeStatus

  proposedOutcomeResult := BoundedStageProposedExecutionOutcome(
    D,currentReservation,cert,q,z,selected system,actual claim,
    certificate OptimizationStatus,the exact observed event,effect/trace roots,
    resourcePartition.FinalizationBudgetPartition.OutcomeStagingSlice;
    build the complete successor DeploymentConfiguration and exactly one of:
      productive(prefix status,ContinuationId,ContinuationStateRoot),
      declared partial(RuntimeStatus,CompletionContractEvidence), or
      terminal success/permitted-refusal/permitted-failure;
    build the complete immutable ExecutionResultCore and receipt core before
      verification, excluding every RuntimeVerificationRecordId and final wrapper;
    bind every before/after configuration, retained-state, effect-ledger and
      runtime-policy root, SnapshotId, SealId, QueryId, invocation,
      CertificateId, actual claim, RuntimeStatus, OptimizationStatus, charged
      trace, outcome, continuation/prefix fields, and receipt fields)
  match proposedOutcomeResult:
    complete(proposedOutcome): continue
    rejected/incoherent/unresolved/unsupported/resource-exhausted/internal-failure:
      // The sufficiency warrant makes bare post-effect exhaustion impossible.
      // Any such result is converted within the emergency slice to the safe
      // quarantined partial outcome below; failure to do so is nonconforming.
      emergencyStage := BoundedStageSafeQuarantinedPartial(
        D,currentReservation,proposedOutcomeResult,
        resourcePartition.FinalizationBudgetPartition.OutcomeStagingFailureSlice)
      match emergencyStage:
        complete(proposedOutcome): continue
        otherwise: return CLOSE_EXECUTION_EFFECT_SCHEDULE_AND_RETURN(
          recovery-required(
            Identity(execution-reservation-domain,currentReservation),
            BuildRecoveryRequestTemplate(currentReservation)))

  requiredRuntimeStatementBody := RequiredRuntimeStatementBody.execution(
    Root(Request body),Request.ExecutionSubject,
    ExactInvocationOrWorkloadInputOrContinuationIdentity(Request),
    ExactAnswerIdentity(answer),Request.CertificateId,
    ExactSelectedSystemOrPolicyId(selected system),
    Root(currentReservation),Root(proposedOutcome.ExactObservedEvent),
    Root(proposedOutcome.SuccessorDeploymentConfiguration),
    Root(proposedOutcome.ImmutableExecutionResultCore),
    Root(proposedOutcome.ImmutableReceiptCore),
    proposedOutcome.RuntimeStatus,proposedOutcome.OptimizationStatus,
    proposedOutcome.ActualClaimClass,proposedOutcome.ChargedTraceRoot,
    proposedOutcome.ContinuationAndPrefixFields?)
  requiredRuntimeStatement := Identity(
    runtime-conformance-statement-domain,requiredRuntimeStatementBody)
  runtimeCheck := BoundedVerifyRuntimeConformance(
    requiredRuntimeStatementBody,requiredRuntimeStatement,
    resourcePartition.FinalizationBudgetPartition.ConformanceCheckSlice)
  match runtimeCheck:
    accept(ids) when requiredRuntimeStatement in ids:
      // No core field may be rebuilt after acceptance. FINISH adds only the
      // acyclic verification record and final wrapper.
      return FINISH_RESERVED(
        proposedOutcome,requiredRuntimeStatementBody,
        requiredRuntimeStatement,runtimeCheck,
        currentEffectProtocolScheduleState)
    exact-counterexample(proof) or undeclared-event(proof):
      violationStage := BoundedStageViolationAndQuarantine(
        D,currentReservation,proposedOutcome,proof,
        resourcePartition.FinalizationBudgetPartition.ConformanceFailureSlice;
        retain every observed state/effect fact, set the binding's quarantine
          entry, and build the complete violation result/receipt cores plus the
          exact post-check transition evidence)
      match violationStage:
        complete(violationOutcome): continue
        otherwise: return CLOSE_EXECUTION_EFFECT_SCHEDULE_AND_RETURN(
          recovery-required(
            Identity(execution-reservation-domain,currentReservation),
            BuildRecoveryRequestTemplate(currentReservation)))
      return FINISH_RESERVED(
        violationOutcome,requiredRuntimeStatementBody,
        requiredRuntimeStatement,runtimeCheck,
        currentEffectProtocolScheduleState)
    accept(ids) | reject-malformed | reject-invalid-proof |
    unresolved | unsupported | resource-exhausted | internal-failure:
      // Inconclusive checking is not a semantic counterexample and creates no
      // exact run claim. The emergency slice still commits every effect and a
      // blocking quarantine before releasing the reservation.
      blockedStage := BoundedStageInconclusiveQuarantinedPartial(
        D,currentReservation,proposedOutcome,runtimeCheck,
        requiredRuntimeStatement,
        resourcePartition.FinalizationBudgetPartition.ConformanceFailureSlice)
      match blockedStage:
        complete(blockedOutcome): continue
        otherwise: return CLOSE_EXECUTION_EFFECT_SCHEDULE_AND_RETURN(
          recovery-required(
            Identity(execution-reservation-domain,currentReservation),
            BuildRecoveryRequestTemplate(currentReservation)))
      return FINISH_RESERVED(
        blockedOutcome,requiredRuntimeStatementBody,
        requiredRuntimeStatement,runtimeCheck,
        currentEffectProtocolScheduleState)
```

Crash recovery uses the persisted reservation, never ambient process memory:

```text
RECOVER_EXECUTION(CoordinatorState C, RecoveryRequest Request):
  coordinatorIngress := BoundedRecoveryCoordinatorIngress(
    C,Request,RecoveryCoordinatorIngressProfileId;
    strictly recompute the recovery body/transaction identity, obtain the one
      exact two-key/head observation, validate a same-body terminal replay or
      different-body reuse, and for a live head perform every running/recovering
      scope, origin-graph, retained-reservation, effect-reference, and
      `ReservationDescendsFrom` check specified by §7.11)
  match coordinatorIngress:
    complete(recoveryBody,recoveryKey,recoveryReplayObservation,
             originalReservation,reservation,true,
             some(resolvedRecoveryOriginGraph),recoveryInvocationAttemptId,
             bootstrapAllocationReceipt,recoveryHeader,
             recoveryBootstrapResources):
      recoveringAlready := true; continue
    complete(recoveryBody,recoveryKey,recoveryReplayObservation,
             originalReservation,reservation,false,none,
             recoveryInvocationAttemptId,bootstrapAllocationReceipt,
             recoveryHeader,recoveryBootstrapResources):
      recoveringAlready := false; continue
    complete(_,_,_,_,_,recoveringFlag,originOption,
             mismatchedAttemptId,_,_,mismatchedBootstrapResources):
      mismatchedBootstrapState := InitialRecoveryBootstrapRemainderState(
        mismatchedBootstrapResources)
      return CloseRecoveryBootstrapRemainderAndReturn(
        mismatchedBootstrapResources,mismatchedBootstrapState,
        internal-failure(FailureReport(
          recovery-coordinator-ingress-origin-graph-presence-mismatch,
          recoveringFlag,originOption,mismatchedAttemptId)))
    duplicate(originalId,original): return duplicate(originalId,original)
    rejected(reason): return rejected(reason)
    incoherent(proof): return incoherent(proof)
    conflict(currentHead): return conflict(currentHead)
    unresolved(ids): return unresolved(ids)
    unsupported(ids): return unsupported(ids)
    resource-exhausted(report): return resource-exhausted(report)
    internal-failure(report): return internal-failure(report)
  recoveryBootstrapRemainderState :=
    InitialRecoveryBootstrapRemainderState(recoveryBootstrapResources)
  require recoveryBootstrapRemainderState.RecoveryInvocationAttemptId =
    recoveryInvocationAttemptId and its partition root equals Identity(
      recovery-invocation-bootstrap-partition-domain,
      recoveryBootstrapResources)
  RETURN_RECOVERY(result):
    procedure-return CloseRecoveryBootstrapRemainderAndReturn(
      recoveryBootstrapResources,recoveryBootstrapRemainderState,result)
  TAKE_BOOTSTRAP_CAPABILITY(tag,capability)
      returns CheckedOutRecoveryBootstrapCapability:
    priorBootstrapStateRoot := Identity(
      recovery-bootstrap-remainder-state-domain,
      recoveryBootstrapRemainderState)
    bootstrapTransition : RecoveryBootstrapTakeResult :=
      ConsumeRecoveryBootstrapCapability(
        recoveryBootstrapRemainderState,recoveryBootstrapResources,
        tag,capability)
    match bootstrapTransition:
      checked-out(checkedOutCapability,successorState,successorRoot):
        require successorRoot=Identity(
          recovery-bootstrap-remainder-state-domain,successorState) and
          checkedOutCapability.RecoveryInvocationAttemptId =
            recoveryInvocationAttemptId and
          checkedOutCapability.RecoveryInvocationBootstrapPartitionRoot =
            Identity(recovery-invocation-bootstrap-partition-domain,
                     recoveryBootstrapResources) and
          checkedOutCapability.RecoveryBootstrapCapabilityTag=tag and
          checkedOutCapability.BeforeRecoveryBootstrapRemainderStateRoot=
            priorBootstrapStateRoot and
          checkedOutCapability.AfterRecoveryBootstrapRemainderStateRoot=
            successorRoot
        recoveryBootstrapRemainderState := successorState
        macro-yield checkedOutCapability
      warrant-violation(report): RETURN_RECOVERY(internal-failure(
        FailureReport(
          recovery-bootstrap-capability-transition-warrant-violation,
          tag,report)))
  DISPOSE_BOOTSTRAP_CAPABILITY(tag,capability,noActionReason):
    bootstrapTransition : RecoveryBootstrapStateTransitionResult :=
      DisposeRecoveryBootstrapCapability(
        recoveryBootstrapRemainderState,recoveryBootstrapResources,
        tag,capability,noActionReason)
    match bootstrapTransition:
      advanced(successorState,successorRoot):
        require successorRoot=Identity(
          recovery-bootstrap-remainder-state-domain,successorState)
        recoveryBootstrapRemainderState := successorState
      warrant-violation(report): RETURN_RECOVERY(internal-failure(
        FailureReport(
          recovery-bootstrap-capability-disposition-warrant-violation,
          tag,report)))
  // Every subsequent public procedure `return result` in this
  // RECOVER_EXECUTION body, including the terminal finalizer match, is the
  // canonical shorthand `RETURN_RECOVERY(result)`. Internal `macro-yield`
  // transfers a local macro value without exiting the procedure and is exempt.
  // Thus every helper use or explicit no-action disposal advances the exact
  // remainder map, and no public branch can bypass closure of a later bootstrap
  // capability.
  primaryBundleLoadSlice := recoveryBootstrapResources.PrimaryBundleLoadSlice
  emergencyTemplateFallbackSlice :=
    recoveryBootstrapResources.EmergencyTemplateFallbackSlice
  currentScheduleStateIngressSlice :=
    recoveryBootstrapResources.CurrentScheduleStateIngressSlice
  takeoverAttemptResources :=
    recoveryBootstrapResources.TakeoverAttemptPartition
  takeoverProofSlice := takeoverAttemptResources.TakeoverProofSlice
  takeoverCandidateCasSlice :=
    takeoverAttemptResources.TakeoverCandidateAndCasAttemptSlice
  preFenceValidationSlice :=
    recoveryBootstrapResources.PreFenceReobservationAndReplayValidationSlice
  fencePreparationSlice :=
    recoveryBootstrapResources.FencePreparationAndCasAttemptSlice
  tailAcquisitionAttemptSlice :=
    recoveryBootstrapResources.TailAcquisitionPreparationAndCasAttemptSlice
  // This conditional header slice is consumed exactly once by the takeover
  // proof below or affinely disposed with a no-action proof on every branch that
  // reaches no takeover proof.
  forceEmergencyQuarantine := false
  primaryBundleLoadCheckout := TAKE_BOOTSTRAP_CAPABILITY(
    primary-bundle-load,primaryBundleLoadSlice)
  bundleResult : RecoveryBundleLoadResult :=
    BoundedLoadAndVerifyRecoveryBundle(
    reservation.RecoveryBundleObjectId,reservation.RecoveryBundleRoot,
    reservation.RecoveryBundleRetentionWarrantId,
    reservation.RecoveryBundleRetentionWarrantObjectId,
    Request,primaryBundleLoadCheckout;
    strict consume-all decode, typed RecoveryBundleBody identity,
    policy/contract/effect-model/original-object identities,
    accepted universal-fence-protocol and safe-terminal-liveness statement identities,
    ExecutionResourcePartitionRoot/ObjectId and
      PartitionAndSufficiencyWarrantId equal to the partition committed by the
      reservation, the header's accepted bootstrap-completion and disjointness
      warrants, complete material-graph resolution/identity/retention, and
      durable-fault-domain
      retention or independently retained EmergencySafeQuarantineTemplateId)
  match bundleResult:
    complete(resolvedRecoveryMaterial : ResolvedRecoveryExecutionMaterial):
      DISPOSE_BOOTSTRAP_CAPABILITY(
        emergency-template-fallback,emergencyTemplateFallbackSlice,
        primary-complete-no-fallback)
      emergencySafeQuarantineTransitionProfile := none
    completion-warrant-violation(report):
      retain the running reservation and return internal-failure(
        FailureReport(recovery-bootstrap-completion-warrant-violation,report))
    rejected/incoherent/unresolved/unsupported/resource-exhausted/internal-failure:
      emergencyTemplateFallbackCheckout := TAKE_BOOTSTRAP_CAPABILITY(
        emergency-template-fallback,emergencyTemplateFallbackSlice)
      emergencyResult : RecoveryEmergencyTemplateLoadResult :=
        BoundedLoadIndependentEmergencySafeQuarantineTemplate(
        reservation.EmergencySafeQuarantineTemplateId,
        reservation.EmergencySafeQuarantineTemplateRoot,
        reservation.EmergencySafeQuarantineTemplateRetentionWarrantId,
        reservation.EmergencySafeQuarantineTemplateRetentionWarrantObjectId,
        reservation.RecoveryExecutionMaterialRoot,
        emergencyTemplateFallbackCheckout)
      match emergencyResult:
        complete(resolvedRecoveryMaterial,emergencyTemplate): continue
        completion-warrant-violation(report) |
        rejected(report) | incoherent(report) | unresolved(report) |
        unsupported(report) | resource-exhausted(report) |
        internal-failure(report):
          retain the reservation and return internal-failure(
            FailureReport(
              fault-outside-declared-recovery-retention-domain,
              bundleResult,emergencyResult,report))
      require the template body/root/object identity, material-graph warrant,
        and independently loaded template-retention warrant are exact and its
        RecoveryExecutionMaterialRoot equals the reservation's exact material
        root; do not consult the unavailable bundle body or bundle-retention
        warrant on this fallback path
      emergencySafeQuarantineTransitionProfile :=
        emergencyTemplate.SafeQuarantineTransitionProfileId
      forceEmergencyQuarantine := true
  recoveryMaterial := resolvedRecoveryMaterial.RecoveryExecutionMaterial
  require Identity(recovery-execution-material-domain,recoveryMaterial) equals
    the validated RecoveryExecutionMaterialRoot
  require resolvedRecoveryMaterial.RecoveryExecutionMaterialGraphRetentionWarrant
    verifies the exact material root and complete control graph under the declared
    lifetime/fault domain
  recoveryPolicy := resolvedRecoveryMaterial.ResolvedRecoveryPolicy
  effectCommitModel := resolvedRecoveryMaterial.ResolvedEffectCommitModel
  effectAuthorizationProfile :=
    resolvedRecoveryMaterial.ResolvedEffectAuthorizationProfile
  (fenceSafetyWarrant : UniversalFenceProtocolWarrant,
   recoveryLivenessWarrant) :=
    resolvedRecoveryMaterial.ResolvedFenceSafetyAndBoundedLivenessStatements
  resourcePartition :=
    resolvedRecoveryMaterial.ResolvedExecutionResourcePartition
  recoveryScheduleStatePersistenceWarrant :=
    resourcePartition.RecoveryBudgetPartition.
      RecoveryScheduleStatePersistenceWarrant
  require all these values' identities equal the corresponding fields of
    recoveryMaterial and the partition root/warrant
  require Identity(recovery-schedule-roots-domain,
                   RecoveryScheduleRoots(resourcePartition)) equals
    recoveryLivenessWarrant.BoundedSafeTerminalLivenessStatement.
      ResumeStageAttemptStageAdvanceCheckpointAndTailAttemptSchedulesRoot
  require recoveryScheduleStatePersistenceWarrant is accepted for
    the exact initial roots/storage profile/transition relation projected from
    resourcePartition and can resolve every current schedule root below
  currentScheduleStateIngressCheckout := TAKE_BOOTSTRAP_CAPABILITY(
    current-schedule-state-ingress,currentScheduleStateIngressSlice)
  scheduleIngressResult := BoundedResolveCurrentRecoveryScheduleStates(
    reservation, reservation.RecoveryWorkRoot?,
    reservation.RecoveryWorkObjectId?, resourcePartition,
    recoveryScheduleStatePersistenceWarrant,
    currentScheduleStateIngressCheckout;
    for a running reservation use the initial stage/advance/checkpoint states and
      the reservation's current resume/tail states; for a recovering reservation
      strictly resolve and verify the complete retained RecoveryWorkObjectGraph,
      including every transitive processed/pending/prior-output/progress object,
      and its three current states plus the reservation's current resume/tail
      states; rederive every root and prove the
      joint current state is a monotone descendant of InitialRecoveryScheduleRoots)
  match scheduleIngressResult:
    complete(resolvedCurrentScheduleStates,some(resolvedRecoveryWorkGraph))
      when recoveringAlready: continue
    complete(resolvedCurrentScheduleStates,none)
      when not recoveringAlready: continue
    complete(_,mismatchedOptionalWorkGraph):
      retain reservation and return internal-failure(
        FailureReport(
          recovery-schedule-ingress-work-graph-presence-mismatch,
          mismatchedOptionalWorkGraph))
    checkpoint(P,ids): retain reservation and return internal-failure(
      FailureReport(
        recovery-schedule-ingress-completion-warrant-violation,P,ids))
    rejected(reason):
      if recoveringAlready, retain the fenced reservation and return
        internal-failure(FailureReport(
          recovery-schedule-persistence-or-bootstrap-warrant-violation,
          scheduleIngressResult))
      otherwise retain the still-running reservation and return rejected(reason)
        without acquiring a fence
    incoherent(proof):
      if recoveringAlready, retain the fenced reservation and return
        internal-failure(FailureReport(
          recovery-schedule-persistence-or-bootstrap-warrant-violation,
          scheduleIngressResult))
      otherwise retain the still-running reservation and return incoherent(proof)
        without acquiring a fence
    unresolved(ids):
      if recoveringAlready, retain the fenced reservation and return
        internal-failure(FailureReport(
          recovery-schedule-persistence-or-bootstrap-warrant-violation,
          scheduleIngressResult))
      otherwise retain the still-running reservation and return unresolved(ids)
        without acquiring a fence
    unsupported(ids):
      if recoveringAlready, retain the fenced reservation and return
        internal-failure(FailureReport(
          recovery-schedule-persistence-or-bootstrap-warrant-violation,
          scheduleIngressResult))
      otherwise retain the still-running reservation and return unsupported(ids)
        without acquiring a fence
    resource-exhausted(report):
      if recoveringAlready, retain the fenced reservation and return
        internal-failure(FailureReport(
          recovery-schedule-persistence-or-bootstrap-warrant-violation,
          scheduleIngressResult))
      otherwise retain the still-running reservation and return
        resource-exhausted(report) without acquiring a fence
    internal-failure(report):
      if recoveringAlready, retain the fenced reservation and return
        internal-failure(FailureReport(
          recovery-schedule-persistence-or-bootstrap-warrant-violation,
          scheduleIngressResult))
      otherwise retain the still-running reservation and return
        internal-failure(report) without acquiring a fence
  originalObjectGraphRef := (
    recoveryMaterial.OriginalExecutionObjectSetRoot,
    recoveryMaterial.OriginalExecutionObjectSetObjectId,
    recoveryMaterial.OriginalExecutionObjectGraphRetentionWarrant)
  preFenceValidationCheckout := TAKE_BOOTSTRAP_CAPABILITY(
    pre-fence-reobservation,preFenceValidationSlice)
  preFenceResult : RecoveryPreFenceResult :=
    BoundedAtomicRecoveryPreFenceReobservation(
    recoveryKey,originalReservation.OriginalTransactionKey,
    Request.DeploymentLineageId,reservation,preFenceValidationCheckout;
    atomically re-read both keys and the lineage deployment head after bootstrap
      resolution, and within that same slice validate any conditionally observed
      winning recovery tuple or normally finalized original tuple)
  match preFenceResult:
    live(preFenceObservation):
      require preFenceObservation.DeploymentHeadState=reserved(reservation)
      continue to fence acquisition
    validated-recovery-winner(originalId,original):
      return duplicate(originalId,original)
    validated-normal-execution-winner(currentHead):
      return conflict(currentHead)
    identity-reuse(reason): return rejected(reason)
    conflict(currentHead): return conflict(currentHead)
    integrity-failure(report): return internal-failure(report)
    unresolved(ids): return unresolved(ids)
    unsupported(ids): return unsupported(ids)
    resource-exhausted(report): return resource-exhausted(report)
    internal-failure(report): return internal-failure(report)
  if recoveringAlready:
    require reservation.ReservationPhase exactly pattern-matches
      recovering(Request.RecoveryTransactionId,recoveryFence,recoveryOrigin),
      `recoveryFence` equals the deterministic fence committed by that phase,
      and `recoveryOrigin` equals the origin strictly resolved and validated by
      `resolvedRecoveryOriginGraph`; this pattern is the sole binding of those
      values on the resume/takeover path
  if not recoveringAlready:
    DISPOSE_BOOTSTRAP_CAPABILITY(
      takeover-proof,takeoverProofSlice,fresh-entry-no-takeover)
    DISPOSE_BOOTSTRAP_CAPABILITY(
      takeover-candidate-cas,takeoverCandidateCasSlice,
      fresh-entry-no-takeover)
    fencePreparationCheckout := TAKE_BOOTSTRAP_CAPABILITY(
      fence-preparation-cas,fencePreparationSlice)
    under this invocation's just-checked-out `fencePreparationCheckout`, prepare the
      following exact candidate and pass that same single-use capability to
      `BoundedAcquireRecoveryFence` to
      set freshLeaseEpoch := NextRecoveryLeaseEpoch(
        reservation.RecoveryLeaseEpochCounter),
      compute recoveryFence := Identity(recovery-fence-domain,
                                Request.RecoveryTransactionId,Root(reservation)),
      construct and retain recoveryOriginGraph := RecoveryOriginObjectGraph(
        recoveryBody,reservation) and
        recoveryOriginRetentionWarrant :=
          the exact accepted RecoveryOriginObjectGraphRetentionWarrant, then
      construct recoveryOrigin := RecoveryOrigin(
        Root(recoveryBody),StoreImmutableExactBody(recoveryBody),
        Request.ExpectedReservedHeadRef,
        StoreImmutableExactBody(reservation),
        Identity(recovery-origin-object-graph-domain,recoveryOriginGraph),
        StoreImmutableExactBody(recoveryOriginGraph),
        Identity(recovery-origin-object-graph-retention-warrant-domain,
                 recoveryOriginRetentionWarrant),
        StoreImmutableExactBody(recoveryOriginRetentionWarrant)),
      construct initialRecoveryWorkBody := RecoveryWorkBody(
        Request.RecoveryTransactionId,Root(recoveryOrigin),fence-quiescence,
        EmptyProcessedWorkRoot,
        Root(the complete pending fence-specific quiescence work),
        Root(recoveryMaterial,resourcePartition,fenceSafetyWarrant,
             recoveryLivenessWarrant),
        Root(resourcePartition.RecoveryBudgetPartition.
             RecoveryStageAttemptSchedule),
        Root(resourcePartition.RecoveryBudgetPartition.
             StageAdvancePublicationSchedule),
        Root(resourcePartition.RecoveryBudgetPartition.
             CheckpointPublicationSchedule),
        InitialRecoveryProgressMeasureAndWarrant),
      construct and retain initialRecoveryWorkGraph :=
        BuildRecoveryWorkObjectGraph(
          initialRecoveryWorkBody,the exact empty processed work,
          the complete pending fence-specific quiescence work,
          the exact immutable material/partition/fence/liveness prior outputs,
          InitialRecoveryProgressMeasureAndWarrant),
      construct/recompute recoveringReservation with ReservationPhase =
        recovering(Request.RecoveryTransactionId,recoveryFence,recoveryOrigin),
        RecoveryWorkRoot=Identity(recovery-work-domain,initialRecoveryWorkBody),
        RecoveryWorkObjectId=initialRecoveryWorkGraph.RecoveryWorkObjectId,
        RecoveryWorkObjectGraphRetentionWarrantId=
          initialRecoveryWorkGraph.RecoveryWorkObjectGraphRetentionWarrantId,
        RecoveryWorkObjectGraphRetentionWarrantObjectId=
          initialRecoveryWorkGraph.
            RecoveryWorkObjectGraphRetentionWarrantObjectId,
        RecoveryResumeAcquisitionMarker=none,
        RecoveryStageAttemptMarker=none,
        RecoveryTailAttemptMarker=none,
        RecoveryLeaseEpochCounter=freshLeaseEpoch,
        RecoveryInvocationLease=RecoveryInvocationLease(
          recoveryInvocationAttemptId,freshLeaseEpoch,
          Request.RecoveryTransactionId,Root(recoveryOrigin),
          Identity(recovery-work-domain,initialRecoveryWorkBody),
          recoveryPolicy.LeaseExpiryOrExplicitQuiescenceRule,
          RecoveryAttemptFenceToken(
            recoveryFence,recoveryInvocationAttemptId,freshLeaseEpoch),
          none),
        LatestAcceptedRecoveryTakeoverEvidenceRef=none
    fenceCasResult : RecoveryReservationMutationResult :=
      BoundedAcquireRecoveryFence(
        recoveryKey,reservation,recoveringReservation,
        resourcePartition.RecoveryBudgetPartition.FencePublicationPermit,
        fencePreparationCheckout;
        before any recovery-visible effect atomically compare the reservation
        and both ledger keys; consume the still-unspent persisted fence permit
        only with a successful reserved(reservation) to
        reserved(recoveringReservation) transition, and classify one exact loss
        observation within the attempt slice)
    match fenceCasResult:
      committed(recoveringReservation):
        resolvedRecoveryWorkGraph := initialRecoveryWorkGraph
      same-body-winner(originalId,original): return duplicate(originalId,original)
      normal-execution-winner(currentHead): return conflict(currentHead)
      identity-reuse(reason): return rejected(reason)
      conflict(currentHead): return conflict(currentHead)
      integrity-failure(report): return internal-failure(report)
      warrant-violation(report,observation): return internal-failure(
        FailureReport(
          recovery-fence-publication-warrant-violation,report,observation))
  else:
    DISPOSE_BOOTSTRAP_CAPABILITY(
      fence-preparation-cas,fencePreparationSlice,
      recovering-entry-no-fresh-fence)
    currentLeaseOption := reservation.RecoveryInvocationLease
    if currentLeaseOption is some(currentLease) and
       reservation.RecoveryResumeAcquisitionMarker exactly binds
       (Request.RecoveryTransactionId,reservation.RecoveryWorkRoot,
        recoveryInvocationAttemptId,
        currentLease.RecoveryLeaseEpoch,
        alreadyConsumedResumeTokenId,
        reservation.RecoveryResumeAcquisitionScheduleRoot)
       and currentLease names that same attempt, epoch,
       origin, work root, and still-valid sink/coordinator fence:
      DISPOSE_BOOTSTRAP_CAPABILITY(
        takeover-proof,takeoverProofSlice,same-owner-no-takeover)
      DISPOSE_BOOTSTRAP_CAPABILITY(
        takeover-candidate-cas,takeoverCandidateCasSlice,
        same-owner-no-takeover)
      acquisitionReservation := reservation
    else if currentLeaseOption is some(currentLease) and
            currentLease names another attempt whose
            lease is not proved expired and quiescent:
      DISPOSE_BOOTSTRAP_CAPABILITY(
        takeover-proof,takeoverProofSlice,active-other-owner-no-takeover)
      DISPOSE_BOOTSTRAP_CAPABILITY(
        takeover-candidate-cas,takeoverCandidateCasSlice,
        active-other-owner-no-takeover)
      return conflict(reserved(reservation)) without consuming a resume token or
        performing a recovery action
    else:
      preResumeScheduleRoot := reservation.RecoveryResumeAcquisitionScheduleRoot
      abandonedPriorOwner := currentLeaseOption is some(...)
      nextFaultCount := reservation.RecoveryFaultCount +
        (1 if abandonedPriorOwner else 0)
      resumePeek := PeekExactNextResumeToken(
        resolvedCurrentScheduleStates.ResumeAcquisitionScheduleState,
        preResumeScheduleRoot,nextFaultCount,
        recoveryLivenessWarrant.DeclaredCrashAndTakeoverFaultBound)
      match resumePeek:
        next(nextResumeToken,postResumeScheduleState,postResumeScheduleRoot):
          continue
        exhausted-within-declared-bound(warrant): return internal-failure(
          FailureReport(
            recovery-resume-schedule-sufficiency-warrant-violation,warrant))
        declared-fault-bound-exceeded(observed,bound): return rejected(
          declared-recovery-fault-domain-exceeded(observed,bound))
        malformed-schedule(reason) | internal-failure(reason):
          return internal-failure(reason)
      nextLeaseEpoch := NextRecoveryLeaseEpoch(
        reservation.RecoveryLeaseEpochCounter)
      takeoverProofCheckout := TAKE_BOOTSTRAP_CAPABILITY(
        takeover-proof,takeoverProofSlice)
      takeoverEvidence := BoundedProveRecoveryLeaseTakeover(
        priorReservation=reservation,
        priorReservationRoot=Root(reservation),
        recoveryOrigin=reservation.RecoveryOrigin,
        priorLease=currentLeaseOption,
        newAttempt=recoveryInvocationAttemptId,
        newLeaseEpoch=nextLeaseEpoch,
        priorFaultCount=reservation.RecoveryFaultCount,
        newFaultCount=nextFaultCount,
        reservation.RecoveryWorkRoot,nextResumeToken,
        preResumeScheduleRoot,postResumeScheduleRoot,
        reservation.RecoveryStageAttemptMarker?,
        reservation.RecoveryTailAttemptMarker?,
        reservation.RecoveryTailAttemptScheduleRoot,
        reservation.EffectIntentLedgerRoot,
        takeoverProofCheckout;
        require sink and coordinator rejection of every prior-owner action and
          derive the complete retained nonterminal prior-attempt fence-token
          set from that exact effect-ledger root, bind its root in the statement,
          prove every previously committed consumed selector remains consumed;
          any fault-keyed stage/advance/checkpoint/tail selector merely selected
          before a losing CAS remains unconsumed but becomes permanently
          unreachable when this CAS increments the serialized fault generation;
          a losing resume peek leaves its linear next token unconsumed and
          reselectable by the winning contender while only the loser's private
          attempt slices are spent; no committed consumption can be restored)
      match takeoverEvidence:
        accepted(takeoverEvidenceObject,takeoverEvidenceRef):
          takeoverStatement := takeoverEvidenceObject.RecoveryTakeoverStatement
          takeoverStatementId :=
            takeoverEvidenceObject.RecoveryTakeoverStatementId
          require takeoverStatementId is the exact statement derived from this
            reservation, nextResumeToken, pre/post schedule roots, markers, next fault
            count, attempt, and epoch, the evidence object's accept set contains
            that exact ID, and takeoverEvidenceRef resolves this exact evidence
            object and accepted retention warrant;
          continue
        rejected(reason): return rejected(reason)
        incoherent(proof): return incoherent(proof)
        conflict(currentHead): return conflict(currentHead)
        unresolved(ids): return unresolved(ids)
        unsupported(ids): return unsupported(ids)
        resource-exhausted(report): return resource-exhausted(report)
        internal-failure(report): return internal-failure(report)
      // Every nonaccepted branch above returns before any resume/stage token is
      // consumed or any recovery action is performed.
      takeoverCandidateCasCheckout := TAKE_BOOTSTRAP_CAPABILITY(
        takeover-candidate-cas,takeoverCandidateCasSlice)
      under the just-checked-out `takeoverCandidateCasCheckout`, store the exact accepted
        evidence and build the candidate same-origin descendant
        acquisitionReservation
        whose RecoveryResumeAcquisitionScheduleRoot is postResumeScheduleRoot
        and whose RecoveryLeaseEpochCounter=nextLeaseEpoch and
        RecoveryFaultCount=nextFaultCount
        and whose RecoveryStageAttemptMarker=none and
        RecoveryTailAttemptMarker=none
        and whose RecoveryResumeAcquisitionMarker equals
        (Request.RecoveryTransactionId,reservation.RecoveryWorkRoot,
         recoveryInvocationAttemptId,nextLeaseEpoch,nextResumeToken,
         postResumeScheduleRoot), and whose RecoveryInvocationLease binds that
        attempt/epoch/work/origin,
        RecoveryAttemptFenceToken(recoveryFence,recoveryInvocationAttemptId,
                                  nextLeaseEpoch), and takeoverEvidenceRef,
        and whose LatestAcceptedRecoveryTakeoverEvidenceRef =
          takeoverEvidenceRef
      takeoverCasResult : RecoveryReservationMutationResult :=
        BoundedAcquireRecoveryResumeLease(
          recoveryKey,reservation,acquisitionReservation,nextResumeToken,
          preResumeScheduleRoot,postResumeScheduleState,
          resolvedRecoveryWorkGraph,resolvedCurrentScheduleStates,
          takeoverCandidateCasCheckout;
          store the accepted evidence and successor schedule-state body, then
          atomically require the next unique persisted token equals
          nextResumeToken and CAS reserved(reservation) to the candidate; consume
          that persisted token only with CAS success and classify the exact
          two-key/head observation on loss; before returning `committed`,
          equality-check the already ingress-resolved complete work graph and
          all five schedule bodies against the committed descendant, using the
          returned post-resume body for the consumed component)
      match takeoverCasResult:
        committed(acquisitionReservation): continue
        same-body-winner(originalId,original): return duplicate(originalId,original)
        normal-execution-winner(currentHead): return conflict(currentHead)
        identity-reuse(reason): return rejected(reason)
        conflict(currentHead): return conflict(currentHead)
        integrity-failure(report): return internal-failure(report)
        warrant-violation(report,observation): return internal-failure(
          FailureReport(
            recovery-resume-publication-warrant-violation,report,observation))
      resolvedCurrentScheduleStates.ResumeAcquisitionScheduleState :=
        postResumeScheduleState
      resolvedCurrentScheduleStates.ResumeAcquisitionScheduleRoot :=
        postResumeScheduleRoot
    recoveringReservation := acquisitionReservation
    require the committed takeover helper, or on same-owner reentry the already
      accepted marker plus current schedule-ingress validation, established the
      exact `ResolvedRecoveryWorkObjectGraph`, RecoveryTransactionId/origin/
      substage, and all five schedule-body equalities; no consumed resume token
      is reused for caller-side work. The reservation's
      post-CAS `RecoveryResumeAcquisitionScheduleRoot` is the sole authoritative
      remaining resume schedule and is not duplicated in RecoveryWorkBody;
      equality-check the already resolved remaining resume, stage-resource,
      stage-advance, checkpoint, and tail schedule state bodies (with the returned
      post-resume body replacing only the consumed resume component),
      immutable prior outputs, and the strict progress warrant; prove their
      `CurrentRecoveryScheduleRoots` is a monotone reachable descendant of the
      liveness statement's exact `InitialRecoveryScheduleRoots`, never equal it
      after a consumption unless the transition relation proves no change
    recoveryCursor := resume(resolvedRecoveryWorkGraph.RecoveryWorkBody)
  if not recoveringAlready:
    recoveryCursor := resume(initialRecoveryWorkBody)
  latestRecoveringReservation := recoveringReservation
  require recoveringReservation.ReservationPhase exactly pattern-matches
    recovering(Request.RecoveryTransactionId,activeRecoveryFence,recoveryOrigin)
    and recoveringReservation.RecoveryInvocationLease exactly pattern-matches
    some(activeRecoveryLease), whose attempt/origin/work/epoch equal this
    acquired reservation
  activeRecoveryAttemptFence :=
    activeRecoveryLease.SinkAndCoordinatorFenceToken

  before every recovery stage, effect-boundary operation, checkpoint CAS, and
    terminal publication below, atomically require the current head still names
    a descendant whose RecoveryInvocationLease has
    RecoveryInvocationAttemptId=recoveryInvocationAttemptId and the exact
    acquired lease epoch/fence; a loss to another valid epoch returns conflict
    before further action, and any non-descendant state returns internal-failure

  ACQUIRE_RECOVERY_STAGE(substage):
    stageAcquisition := AcquireRecoveryStageAttempt(
      Request,latestRecoveringReservation,recoveryCursor,
      resolvedRecoveryWorkGraph,
      resolvedCurrentScheduleStates.RecoveryStageAttemptScheduleState,substage,
      recoveryLivenessWarrant.DeclaredCrashAndTakeoverFaultBound)
    match stageAcquisition:
      acquired(descendant,bundle,nextCursor,successorStageScheduleState,
               successorResolvedWorkGraph,
               materializedEffectScheduleOption):
        latestRecoveringReservation := descendant
        recoveringReservation := descendant
        recoveryCursor := nextCursor
        resolvedCurrentScheduleStates.RecoveryStageAttemptScheduleState :=
          successorStageScheduleState
        resolvedCurrentScheduleStates.RecoveryStageAttemptScheduleRoot :=
          nextCursor.RecoveryWorkBody.RemainingStageResourceStateRoot
        resolvedRecoveryWorkGraph := successorResolvedWorkGraph
        stageAttemptBundle := bundle
        materializedRecoveryEffectScheduleOption :=
          materializedEffectScheduleOption
        if substage=effect-reconciliation and
           materializedEffectScheduleOption is not some(_): return
          CloseRecoveryStageNoPublication(stageAttemptBundle,internal-failure(
            missing-materialized-recovery-effect-schedule))
        if substage!=effect-reconciliation and
           materializedEffectScheduleOption is not none: return
          CloseRecoveryStageNoPublication(stageAttemptBundle,internal-failure(
            unexpected-materialized-recovery-effect-schedule))
      exhausted-within-declared-bound(warrant): return internal-failure(
        FailureReport(
          recovery-stage-attempt-sufficiency-warrant-violation,warrant))
      declared-fault-bound-exceeded(observed,bound): return rejected(
        declared-recovery-fault-domain-exceeded(observed,bound))
      same-body-winner(originalId,original): return duplicate(originalId,original)
      normal-execution-winner(currentHead): return conflict(currentHead)
      identity-reuse(reason): return rejected(reason)
      integrity-failure(report): return internal-failure(report)
      warrant-violation(report,observation): return internal-failure(
        FailureReport(
          recovery-stage-acquisition-warrant-violation,report,observation))
      rejected(reason): return rejected(reason)
      incoherent(proof): return incoherent(proof)
      conflict(currentHead): return conflict(currentHead)
      unresolved(ids): return unresolved(ids)
      unsupported(ids): return unsupported(ids)
      resource-exhausted(report): return resource-exhausted(report)
      internal-failure(report): return internal-failure(report)

  ADVANCE_RECOVERY_STAGE(completionState,nextSubstage):
    stageExitSelection : RecoveryStageExitSelectionResult :=
      SelectAndCloseRecoveryStageExit(stageAttemptBundle,advance)
    match stageExitSelection:
      advance(stageAdvanceAttemptSlice,stageExitDispositionProof): continue
      warrant-violation(report): return internal-failure(
        FailureReport(
          recovery-stage-exit-disposition-warrant-violation,report))
      otherwise: return internal-failure(
        FailureReport(recovery-stage-exit-tag-mismatch,stageExitSelection))
    advanceResult := AdvanceRecoveryCursor(
      Request,latestRecoveringReservation,
      latestRecoveringReservation.RecoveryOrigin,recoveryCursor,
      resolvedRecoveryWorkGraph,
      resolvedCurrentScheduleStates.StageAdvancePublicationScheduleState,
      completionState,nextSubstage,
      stageAdvanceAttemptSlice)
    match advanceResult:
      complete(descendant,nextCursor,successorAdvanceScheduleState,
               successorResolvedWorkGraph):
        latestRecoveringReservation := descendant
        recoveringReservation := descendant
        recoveryCursor := nextCursor
        resolvedCurrentScheduleStates.StageAdvancePublicationScheduleState :=
          successorAdvanceScheduleState
        resolvedCurrentScheduleStates.StageAdvancePublicationScheduleRoot :=
          nextCursor.RecoveryWorkBody.
            RemainingStageAdvancePublicationScheduleRoot
        resolvedRecoveryWorkGraph := successorResolvedWorkGraph
      conflict(currentHead): return conflict(currentHead)
      same-body-winner(originalId,original): return duplicate(originalId,original)
      normal-execution-winner(currentHead): return conflict(currentHead)
      identity-reuse(reason): return rejected(reason)
      integrity-failure(report): return internal-failure(report)
      warrant-violation(report,observation): return internal-failure(
        FailureReport(
          recovery-stage-advance-warrant-violation,report,observation))
      rejected(reason): return rejected(reason)
      incoherent(proof): return incoherent(proof)
      unresolved(ids): return unresolved(ids)
      unsupported(ids): return unsupported(ids)
      resource-exhausted(report): return resource-exhausted(report)
      internal-failure(report): return internal-failure(report)

  DISPATCH_RECOVERY_CURSOR(recoveryCursor):
    fresh(body) or resume(body) when body.RecoverySubstage=fence-quiescence:
      enter FENCE_QUIESCENCE with the exact pending/processed state
    fresh(body) or resume(body) when body.RecoverySubstage=input-validation:
      restore the accepted quiescence proof from ImmutablePriorOutputRoots, set
      latestRecoveringReservation to the current `recoveringReservation` while
      proving it descends from the recorded ancestor, skip fence observation,
      and enter INPUT_VALIDATION
    fresh(body) or resume(body) when
      body.RecoverySubstage=effect-reconciliation:
      restore quiescenceProof and validationInput, skip both completed stages,
      set latestRecoveringReservation to the current `recoveringReservation`
      with the proved ancestor relation, and enter EFFECT_RECONCILIATION with the
      exact pending effect work
    fresh(body) or resume(body) when body.RecoverySubstage=outcome-staging:
      restore quiescenceProof, validationInput, reconstructedEvent, and
      Dcandidate; set latestRecoveringReservation to the current
      `recoveringReservation` with the proved ancestor relation, skip completed
      effect work, and enter OUTCOME_STAGING with the exact pending staging work
    any missing, mismatched, later-stage, or non-progressing cursor:
      return rejected(malformed-recovery-checkpoint) without a recovery effect

  COMMIT_RECOVERY_CHECKPOINT(
      baseReservation,substage,P,ids,preselectedCheckpointAttemptSlice?=none):
    require baseReservation.RecoveryInvocationLease exactly pattern-matches
      some(checkpointLease), and
      checkpointLease.RecoveryInvocationAttemptId =
        recoveryInvocationAttemptId and
      checkpointLease.RecoveryLeaseEpoch =
        activeRecoveryLease.RecoveryLeaseEpoch and
      checkpointLease.SinkAndCoordinatorFenceToken =
        activeRecoveryAttemptFence and
      checkpointLease.RecoveryTransactionId = Request.RecoveryTransactionId and
      checkpointLease.RecoveryWorkRoot = baseReservation.RecoveryWorkRoot
    require P binds Request.RecoveryTransactionId, the exact RecoveryOrigin,
      substage, immutable prior-output roots, and a strict well-founded progress
      step
    require P starts from the validated RecoveryStageCursor's exact remaining
      stage-resource, stage-advance-publication, and checkpoint-publication roots
    if preselectedCheckpointAttemptSlice is some(checkpointAttemptSlice):
      continue; the emergency stage-exit selector already closed all siblings
    else:
      stageExitSelection : RecoveryStageExitSelectionResult :=
        SelectAndCloseRecoveryStageExit(stageAttemptBundle,checkpoint)
      match stageExitSelection:
        checkpoint(checkpointAttemptSlice,stageExitDispositionProof): continue
        warrant-violation(report): return internal-failure(
          FailureReport(
            recovery-stage-exit-disposition-warrant-violation,report))
        otherwise: return internal-failure(
          FailureReport(recovery-stage-exit-tag-mismatch,stageExitSelection))
    require the identity of
      resolvedCurrentScheduleStates.CheckpointPublicationScheduleState equals
      RecoveryStageCursor(recoveryCursor,substage).RecoveryWorkBody.
        RemainingCheckpointPublicationScheduleRoot; derive
      `progressOrdinal` from P's accepted ProgressMeasureAndWarrant, require
      it binds P's exact progress-state root, and form the unique
      checkpointKey := CheckpointPublicationKey(
        substage,progressOrdinal,baseReservation.RecoveryFaultCount,
        next CheckpointOrdinal)
    checkpointSelection : CheckpointPublicationSelectionResult :=
      SelectCheckpointPublicationToken(
        resolvedCurrentScheduleStates.CheckpointPublicationScheduleState,
        checkpointKey)
    match checkpointSelection:
      selected(checkpointTokenBody,checkpointTokenId,
               successorCheckpointScheduleState,
               remainingCheckpointScheduleRoot): continue
      exhausted-within-declared-bound(warrant): return
        DisposeSelectedRecoveryStageExit(checkpointAttemptSlice,
          internal-failure(
            FailureReport(
              recovery-checkpoint-schedule-sufficiency-warrant-violation,
              warrant)))
      malformed-schedule(reason) | internal-failure(reason):
        return DisposeSelectedRecoveryStageExit(
          checkpointAttemptSlice,internal-failure(reason))
    selection is nonconsuming and the key/state advance occurs only
      inside the successful CAS below; never select from the original full
      resource-partition schedule
    recoveryWorkBody := RecoveryWorkBody(
      Request.RecoveryTransactionId,Root(baseReservation.RecoveryOrigin),substage,
      P.ProcessedWorkRoot,P.PendingWorkRoot,P.ImmutablePriorOutputRoots,
      P.RemainingStageResourceStateRoot,
      P.RemainingStageAdvancePublicationScheduleRoot,
      remainingCheckpointScheduleRoot,
      P.ProgressMeasureAndWarrant)
    recoveryWorkGraph := BuildRecoveryWorkObjectGraph(
      recoveryWorkBody,
      P.ResolvedRecoveryWorkObjectGraph.RecoveryWorkPayload;
      require every body/root equals P and all transitive objects are retained)
    checkpointReservation := a `recovering` descendant of baseReservation whose
      RecoveryWorkRoot = Identity(recovery-work-domain,recoveryWorkBody) and
      RecoveryWorkObjectId = recoveryWorkGraph.RecoveryWorkObjectId and
      RecoveryWorkObjectGraphRetentionWarrantId =
        recoveryWorkGraph.RecoveryWorkObjectGraphRetentionWarrantId and
      RecoveryWorkObjectGraphRetentionWarrantObjectId =
        recoveryWorkGraph.RecoveryWorkObjectGraphRetentionWarrantObjectId and
      RecoveryResumeAcquisitionMarker = none and RecoveryInvocationLease = none
      and LatestAcceptedRecoveryTakeoverEvidenceRef =
        baseReservation.LatestAcceptedRecoveryTakeoverEvidenceRef
      and RecoveryStageAttemptMarker = none and RecoveryTailAttemptMarker = none
    checkpointCasResult : RecoveryCheckpointPublicationResult :=
      BoundedCommitRecoveryCheckpoint(
        recoveryKey,baseReservation,checkpointReservation,
        recoveryWorkBody,recoveryWorkGraph,checkpointTokenBody,checkpointTokenId,
        successorCheckpointScheduleState,remainingCheckpointScheduleRoot,
        checkpointAttemptSlice;
        build and store the exact `RecoveryCheckpoint` identity/object,
        atomically store the successor work graph and schedule-state body, CAS
        reserved(baseReservation) to reserved(checkpointReservation), consume
        the persisted checkpoint selector/permit only on success, and classify the one
        exact two-key/head observation on loss)
    match checkpointCasResult:
      committed(checkpointReservation,checkpointObject):
        return checkpoint(checkpointObject,ids)
      same-body-winner(originalId,original): return duplicate(originalId,original)
      normal-execution-winner(currentHead): return conflict(currentHead)
      identity-reuse(reason): return rejected(reason)
      conflict(currentHead): return conflict(currentHead)
      integrity-failure(report): return internal-failure(
        FailureReport(recovery-checkpoint-head-integrity-violation,report))
      warrant-violation(report,observation): return internal-failure(
        FailureReport(
          recovery-checkpoint-publication-warrant-violation,
          report,observation))

  invoke DISPATCH_RECOVERY_CURSOR(recoveryCursor) exactly once here. The selected
    arm performs an immediate, non-fallthrough transfer to exactly one of
    FENCE_QUIESCENCE, INPUT_VALIDATION, EFFECT_RECONCILIATION, or OUTCOME_STAGING;
    no textual label below is entered merely by fallthrough. The dispatcher uses
    `recoveringReservation` as the observed current descendant and establishes
    `latestRecoveringReservation` before the target stage can acquire a bundle.
  FENCE_QUIESCENCE:
  ACQUIRE_RECOVERY_STAGE(fence-quiescence)
  require fenceSafetyWarrant is the accepted universal protocol premise proving
    how any intent/permit is fenced, quiesced, deduplicated, or classified as
    declared at-least-once; it does not mention or prove the newly created
    activeRecoveryFence
  require recoveryLivenessWarrant is the accepted exact statement that every
    reachable recovery state reaches a safe terminal successor within the
    reserved finite recovery/emergency budget or commits a strictly decreasing
    resumable RecoveryCheckpoint; any eventual-completion claim also binds and
    proves the declared fairness premise
  quiescenceResult := BoundedObserveFenceSpecificSinkQuiescence(
    recoveringReservation,activeRecoveryFence,
    RecoveryStageCursor(recoveryCursor,fence-quiescence),
    boundedly resolve and strictly verify the exact EffectIntentLedger object and
      transitive graph named by the reservation's root/object/retention-warrant
      fields, then every persisted EffectIntentEntry and EffectPermit,
    effectCommitModel,fenceSafetyWarrant;
    construct QuiescenceStatement over this activeRecoveryFence, the current
      EffectIntentLedgerRoot, the complete intent/permit carrier, sink
      observations, effect model, and claimed boundary; return complete only
      when the verifier accepts its exact QuiescenceStatementId,
    stageAttemptBundle.StageWorkSlice)
  match quiescenceResult:
    complete(latestRecoveringReservation,quiescenceProof,completionState):
      require completionState's exact immutable output graph contains
        quiescenceProof and its accepted statement/preimage
      ADVANCE_RECOVERY_STAGE(completionState,input-validation)
      goto INPUT_VALIDATION; do not fall through or reacquire fence-quiescence
    checkpoint(P,ids):
      return COMMIT_RECOVERY_CHECKPOINT(
        recoveringReservation,fence-quiescence,P,ids)
    conflict(currentHead): return CloseRecoveryStageNoPublication(
      stageAttemptBundle,conflict(currentHead))
    failure(exactFailure):
      emergencyExitSelection : RecoveryStageExitSelectionResult :=
        SelectAndCloseRecoveryStageExit(stageAttemptBundle,emergency-checkpoint)
      match emergencyExitSelection:
        emergency-checkpoint(emergencyCheckpointSlice,
                             emergencyCheckpointAttemptSlice,
                             stageExitDispositionProof): continue
        warrant-violation(report): return internal-failure(
          FailureReport(
            recovery-stage-exit-disposition-warrant-violation,report))
        otherwise: return internal-failure(
          FailureReport(
            recovery-stage-exit-tag-mismatch,emergencyExitSelection))
      emergencyCheckpointResult := BoundedBuildRecoveryEmergencyCheckpoint(
        recoveringReservation,exactFailure,recoveryLivenessWarrant,
        emergencyCheckpointSlice)
      match emergencyCheckpointResult:
        complete(P,emergencyIds):
          return COMMIT_RECOVERY_CHECKPOINT(
            recoveringReservation,fence-quiescence,P,emergencyIds,
            some(emergencyCheckpointAttemptSlice))
        otherwise:
          retain the recovering reservation and return
            DisposeSelectedRecoveryStageExit(
              emergencyCheckpointAttemptSlice,internal-failure(
                FailureReport(
                  verified-recovery-liveness-warrant-violation,
                  emergencyCheckpointResult)))

  INPUT_VALIDATION:
  ACQUIRE_RECOVERY_STAGE(input-validation)
  recoveryValidation := BoundedValidateRecoveryInputs(
    originalObjectGraphRef and every effect-intent object,
    Request,latestRecoveringReservation,quiescenceProof,
    RecoveryStageCursor(recoveryCursor,input-validation),
    forceEmergencyQuarantine,emergencySafeQuarantineTransitionProfile,
    require the nonempty profile identity and material root equal the independently
      retained template and forbid ordinary recovery when emergency mode is set,
    stageAttemptBundle.StageWorkSlice)
  match recoveryValidation:
    complete(originalIdentityBinding,recoveryValidationEvidence,
             validatedOriginalObjects,completionState):
      validationInput := validated(
        originalIdentityBinding,recoveryValidationEvidence,
        validatedOriginalObjects)
      require completionState's exact immutable output graph contains this
        `validationInput` and all of its retained preimages
      ADVANCE_RECOVERY_STAGE(completionState,effect-reconciliation)
      goto EFFECT_RECONCILIATION; do not fall through
    checkpoint(P,ids): return COMMIT_RECOVERY_CHECKPOINT(
      latestRecoveringReservation,input-validation,P,ids)
    failure(stageFailure,originalIdentityBinding,failedValidationEvidence,
            completionState):
      validationInput := failed-validation(
        originalIdentityBinding,stageFailure,failedValidationEvidence)
      require
        completionState's exact immutable output graph contains it
      ADVANCE_RECOVERY_STAGE(completionState,effect-reconciliation)
      // This exact failure is data for the recovery policy; it is not silently
      // treated as successful validation or allowed to leave an unowned fence.
      goto EFFECT_RECONCILIATION; do not fall through
  EFFECT_RECONCILIATION:
  ACQUIRE_RECOVERY_STAGE(effect-reconciliation)
  match materializedRecoveryEffectScheduleOption:
    some(materializedRecoveryEffectSchedule): continue
    none: return CloseRecoveryStageNoPublication(
      stageAttemptBundle,internal-failure(
        missing-materialized-recovery-effect-schedule))
  reconcileResult := BoundedReconcileRecoveryEffects(
    latestRecoveringReservation,effectCommitModel,recoveryPolicy,
    effectAuthorizationProfile,
    emergencySafeQuarantineTransitionProfile,
    RecoveryStageCursor(recoveryCursor,effect-reconciliation),
    stageAttemptBundle.StageWorkSlice,
    materializedRecoveryEffectSchedule,
    validationInput,
    in emergency mode perform only the template's exact safe-quarantine
      reconciliation/transition and no normal retry or compensation,
    require every compensation/retry/recovery effect to construct and verify an
      exact typed EffectAuthorizationStatement, append a typed EffectIntentEntry,
      and use a sink-consumed typed EffectPermit bound to
      `ReservationFenceToken=recovery(activeRecoveryAttemptFence)` (not merely
      the transaction-wide recovery fence), with the identical
      CAS/deduplication/quiescence protocol;
    exhaustively consume each `RecoveryEffectProtocolResult`: committed updates
      the current descendant; recovery-owned is embedded in the exact failure/
      checkpoint completion graph for resumed reconciliation; a terminal winner,
      identity reuse, integrity, warrant, or lease conflict is lifted to the
      identically tagged `RecoveryReconciliationResult`)
  // Every recovery intent/status update CASes the exact current recovering
  // reservation and advances only its committed effect-ledger root/object/warrant
  // fields while preserving RecoveryWorkRoot and all work-graph references. The
  // subsequent successful ADVANCE_RECOVERY_STAGE is the sole publisher of the
  // completed same-stage work graph. After intent publication, every noncomplete
  // sink/status path returns a retained recovery-owned carrier; a lease loss
  // exposes only the exact reserved descendant containing that intent.
  match reconcileResult:
    complete(reconstructedEvent,Dcandidate,latestRecoveringReservation,
             completionState):
      require completionState's exact immutable output graph contains
        reconstructedEvent,Dcandidate, and all reconciliation evidence
      ADVANCE_RECOVERY_STAGE(completionState,outcome-staging)
      goto OUTCOME_STAGING; do not fall through
    exact-counterexample(proof,reconstructedEvent,Dcandidate,
                         latestRecoveringReservation,completionState):
      require reconstructedEvent=violation-event(proof), Dcandidate is the exact
        quarantined successor retaining every known effect, and completionState's
        immutable output graph contains both
      ADVANCE_RECOVERY_STAGE(completionState,outcome-staging)
      goto OUTCOME_STAGING; do not fall through
    failure(stageFailure,reconstructedEvent,Dcandidate,
            latestRecoveringReservation,completionState):
      // The policy's reserved emergency slice must still make this a safe
      // quarantined partial terminal; bare fenced exhaustion is nonconforming.
      require reconstructedEvent=inconclusive-recovery(stageFailure),
        Dcandidate is the exact quarantined partial successor retaining every
        known effect, and completionState's immutable output graph contains both
      ADVANCE_RECOVERY_STAGE(completionState,outcome-staging)
      goto OUTCOME_STAGING; do not fall through
    checkpoint(P,ids,latestRecoveringReservation):
      return COMMIT_RECOVERY_CHECKPOINT(
        latestRecoveringReservation,effect-reconciliation,P,ids)
    same-body-winner(originalId,original): return
      CloseRecoveryStageNoPublication(
        stageAttemptBundle,duplicate(originalId,original))
    normal-execution-winner(currentHead): return
      CloseRecoveryStageNoPublication(stageAttemptBundle,conflict(currentHead))
    identity-reuse(reason): return CloseRecoveryStageNoPublication(
      stageAttemptBundle,rejected(reason))
    integrity-failure(report): return CloseRecoveryStageNoPublication(
      stageAttemptBundle,internal-failure(report))
    warrant-violation(report,observation): return
      CloseRecoveryStageNoPublication(stageAttemptBundle,internal-failure(
        FailureReport(
          recovery-effect-protocol-warrant-violation,report,observation)))
    conflict(currentHead):
      return CloseRecoveryStageNoPublication(
        stageAttemptBundle,conflict(currentHead)) and perform no further
        recovery effect

  OUTCOME_STAGING:
  tailAcquisitionCheckout := TAKE_BOOTSTRAP_CAPABILITY(
    tail-acquisition-preparation-cas,tailAcquisitionAttemptSlice)
  tailAcquisition : RecoveryTailAttemptAcquisitionResult :=
    AcquireRecoveryTailAttempt(
    Request,recoveryKey,latestRecoveringReservation,
    recoveryInvocationAttemptId,
    activeRecoveryLease.RecoveryLeaseEpoch,
    activeRecoveryAttemptFence,resolvedRecoveryWorkGraph,
    resolvedCurrentScheduleStates.RecoveryTailAttemptScheduleState,
    latestRecoveringReservation.RecoveryTailAttemptScheduleRoot,
    latestRecoveringReservation.RecoveryFaultCount,
    tailAcquisitionCheckout,
    recoveryLivenessWarrant.DeclaredCrashAndTakeoverFaultBound;
    require every request/key/attempt/epoch/fence/work value equals the current
      reservation and the supplied ingress-validated schedule state's identity
      equals the current root; select its full embedded next bundle and accept the bundle's
      exact TailCompletionAndDisjointnessStatement using only its
      TailAttemptAcquisitionSlice, then derive `postTailScheduleState` and
      `postTailScheduleRoot` from that selected current schedule, atomically
      consume that exact bundle, store that exact successor body under its root,
      and CAS
    reserved(latestRecoveringReservation) to reserved(tailReservation), where
    tailReservation is the same-lease descendant with
    RecoveryTailAttemptScheduleRoot=`postTailScheduleRoot` and a
    RecoveryTailAttemptMarker binding
    (Request.RecoveryTransactionId,recoveryInvocationAttemptId,
     activeRecoveryLease.RecoveryLeaseEpoch,
     latestRecoveringReservation.RecoveryWorkRoot,
     tailBundle.TailAttemptTokenId,
     Identity(recovery-tail-attempt-bundle-domain,tailBundle),
     postTailScheduleRoot); require `acquired` returns
     `postTailScheduleState` and the marker's post-schedule root equals that exact
     root)
  match tailAcquisition:
    acquired(tailReservation,tailBundle,successorTailScheduleState):
      latestRecoveringReservation := tailReservation
      recoveringReservation := tailReservation
      tailProgressState := RecoveryTailProgressState(
        Identity(recovery-tail-attempt-bundle-domain,tailBundle),
        consumed,unspent,unspent,unspent,unspent,unspent)
      resolvedCurrentScheduleStates.RecoveryTailAttemptScheduleState :=
        successorTailScheduleState
      resolvedCurrentScheduleStates.RecoveryTailAttemptScheduleRoot :=
        tailReservation.RecoveryTailAttemptScheduleRoot
    conflict(currentHead): return conflict(currentHead)
    exhausted-within-declared-bound(warrant):
      retain the reservation and return internal-failure(
        FailureReport(
          recovery-tail-schedule-sufficiency-warrant-violation,warrant))
    declared-fault-bound-exceeded(observed,bound): return rejected(
      declared-recovery-fault-domain-exceeded(observed,bound))
    same-body-winner(originalId,original): return duplicate(originalId,original)
    normal-execution-winner(currentHead): return conflict(currentHead)
    identity-reuse(reason): return rejected(reason)
    integrity-failure(report): return internal-failure(report)
    warrant-violation(report,observation): return internal-failure(
      FailureReport(
        recovery-tail-acquisition-warrant-violation,report,observation))
    rejected(reason): return rejected(reason)
    incoherent(proof): return incoherent(proof)
    unresolved(ids): return unresolved(ids)
    unsupported(ids): return unsupported(ids)
    resource-exhausted(report): return resource-exhausted(report)
    internal-failure(report): return internal-failure(report)
  // TailCompletionAndDisjointnessWarrant makes outcome staging, its emergency
  // conversion, runtime conformance, and terminal publication noncheckpointing.
  // This owning attempt performs no reserved-head mutation after acquisition
  // except the final two-key-and-head CAS. A proved fencing takeover invalidates
  // it; the new owner consumes another bundle and rebuilds all cores against its
  // newly acquired reservation.
  proposedRecoveryResult := BoundedStageRecoveredOutcome(
    validationInput and the fixed reservation request/body identities,
    reconstructedEvent,Dcandidate,latestRecoveringReservation,
    emergencySafeQuarantineTransitionProfile,
    RecoveryStageCursor(recoveryCursor,outcome-staging),
    original certified status/claim,Request,
    tailBundle.OutcomeStagingSlice;
    build before verification the exact successor deployment, evaluated
      ExecutionResultCore, evaluated receipt core, and RecoveryCommitPreCore;
      do not construct a RuntimeVerificationRecord, final execution receipt or
      result, companion ledger entry, RecoveryCommitCore, RecoveryReceipt, or
      RecoveryResult; bind every before/after state/effect/runtime-policy root,
      status, claim, charged trace, effect-reconciliation fact, continuation
      field, receipt-core field, and coordinator before/after-head root)
  tailProgressState.OutcomeStagingSliceState := consumed
  match proposedRecoveryResult:
    complete(proposedRecovery): continue
    checkpoint(P,ids): retain latestRecoveringReservation and return
      CloseRecoveryTailAttemptAndReturn(
        tailBundle,tailProgressState,internal-failure(FailureReport(
          noncheckpointing-tail-bundle-warrant-violation,P,ids)))
    rejected/incoherent/unresolved/unsupported/resource-exhausted/internal-failure:
      emergencyRecoveryStage := BoundedStageEmergencyRecoveredQuarantine(
        proposedRecoveryResult,latestRecoveringReservation,
        tailBundle.OutcomeStagingFailureSlice)
      tailProgressState.OutcomeStagingFailureSliceState := consumed
      match emergencyRecoveryStage:
        complete(proposedRecovery): continue
        checkpoint(P,ids): retain latestRecoveringReservation and return
          CloseRecoveryTailAttemptAndReturn(
            tailBundle,tailProgressState,internal-failure(FailureReport(
              noncheckpointing-tail-bundle-warrant-violation,P,ids)))
        otherwise:
          retain latestRecoveringReservation and return
            CloseRecoveryTailAttemptAndReturn(
              tailBundle,tailProgressState,internal-failure(FailureReport(
                verified-recovery-liveness-warrant-violation,
                emergencyRecoveryStage)))

  // `proposedRecovery` contains only acyclic result/receipt/pre-commit cores
  // whose before-head root is exactly the acquired tail reservation. Final
  // wrapper and ledger identities do not yet exist.

  requiredRecoveryStatementBody := RequiredRuntimeStatementBody.recovery(
    Root(recoveryBody),
    validationInput.OriginalExecutionIdentityBinding.
      OriginalExecutionRequestBodyRoot,
    validationInput.OriginalExecutionIdentityBinding.OriginalExecutionSubject,
    validationInput.OriginalExecutionIdentityBinding.
      OriginalInvocationOrWorkloadInputOrContinuationIdentity,
    validationInput.OriginalExecutionIdentityBinding.OriginalAnswerIdentity,
    validationInput.OriginalExecutionIdentityBinding.OriginalCertificateId,
    validationInput.OriginalExecutionIdentityBinding.
      OriginalSelectedSystemOrPolicyId,
    Root(validationInput),Root(latestRecoveringReservation),
    Root(proposedRecovery.EvaluatedSuccessorDeploymentConfiguration),
    Root(proposedRecovery.EvaluatedExecutionResultCore),
    Root(proposedRecovery.EvaluatedReceiptCore),
    Root(proposedRecovery.RecoveryCommitPreCore),
    proposedRecovery.RuntimeStatus,proposedRecovery.OptimizationStatus,
    proposedRecovery.ActualClaimClass,proposedRecovery.ChargedTraceRoot)
  requiredRecoveryStatement := Identity(
    runtime-conformance-statement-domain,requiredRecoveryStatementBody)
  runtimeStageResult := BoundedVerifyAndStageRecoveryRuntimeOutcome(
    requiredRecoveryStatementBody,requiredRecoveryStatement,
    proposedRecovery,latestRecoveringReservation,
    tailBundle.ConformanceCheckSlice,
    tailBundle.ConformanceFailureSlice;
    accept(ids) containing the required statement preserves the evaluated cores;
    exact-counterexample or undeclared-event constructs the exact quarantined
      violation result/receipt cores and a published RecoveryCommitPreCore;
    accept missing the ID or any malformed/invalid/inconclusive verifier result
      constructs the exact quarantined partial cores and published pre-core;
    every changed pre-core is built by BuildPublishedRecoveryCommitPreCore and
      binds the actually published successor/effect roots; return no checkpoint)
  tailProgressState.ConformanceCheckSliceState := consumed
  tailProgressState.ConformanceFailureSliceState := consumed
  match runtimeStageResult:
    complete(runtimeCheck,publishedRecovery):
      publishedRecoveryCommitPreCore :=
        publishedRecovery.PublishedRecoveryCommitPreCore
      postCheckTransitionEvidence :=
        publishedRecovery.PostCheckTransitionEvidence
      continue
    checkpoint(P,ids):
      retain latestRecoveringReservation and return
        CloseRecoveryTailAttemptAndReturn(
          tailBundle,tailProgressState,internal-failure(FailureReport(
            noncheckpointing-runtime-stage-warrant-violation,P,ids)))
    rejected/incoherent/unresolved/unsupported/resource-exhausted/internal-failure:
      retain latestRecoveringReservation and return
        CloseRecoveryTailAttemptAndReturn(
          tailBundle,tailProgressState,internal-failure(FailureReport(
            verified-recovery-liveness-warrant-violation,runtimeStageResult)))

  finalizeRecovery := BoundedBuildEntriesAndAtomicallyFinalizeRecovery(
    reservation.OriginalTransactionKey,
    validationInput.OriginalExecutionIdentityBinding.
      ExactOriginalExecutionRequestBody,
    proposedRecovery.EvaluatedExecutionResultCore,
    proposedRecovery.EvaluatedReceiptCore,
    publishedRecovery.PublishedExecutionResultCore,
    publishedRecovery.PublishedReceiptCore,
    proposedRecovery.RecoveryCommitPreCore,
    publishedRecoveryCommitPreCore,
    requiredRecoveryStatementBody,requiredRecoveryStatement,
    runtimeCheck,postCheckTransitionEvidence,
    recoveryKey,recoveryBody,
    expectedHead=reserved(latestRecoveringReservation),
    requiredAbsentKeys={reservation.OriginalTransactionKey,recoveryKey},
    successorHead=available(
      publishedRecovery.PublishedSuccessorDeploymentConfiguration),
    tailBundle,tailProgressState;
    require tailProgressState.TailAttemptBundleRoot=Identity(
      recovery-tail-attempt-bundle-domain,tailBundle), its acquisition/outcome/
      conformance fields exactly reflect the calls above, and its
      AtomicPublicationSliceState=unspent;
    construct/store the exact RuntimeVerificationEvidenceObjectGraph whose
      projections are `requiredRecoveryStatementBody`, the evaluated and
      published result/receipt/recovery-pre-cores, exact runtimeCheck, verifier
      evidence, and post-check transition evidence; construct/store and accept
      its exact retention warrant; then construct RuntimeVerificationRecord over
      those identical graph projections/roots and references;
    use FinalizeExecutionResult to construct the final original execution
      receipt/result embedding that complete record without changing a core
      field, then construct its complete
      companion TransactionLedgerEntry and identity;
    extend the published RecoveryCommitPreCore with that companion entry,
      original-result, and original-receipt identities to construct
      RecoveryCommitCore, then construct
      RecoveryReceipt, recovered RecoveryResult, and its complete recovery entry;
    build every object/result/receipt identity within this single-use bundle's
      atomic publication slice, affinely close every untaken outcome/conformance
      sibling under its exact branch-disposition warrant, then
      perform one atomic two-key-and-head coordinator transition; on CAS loss
      validate the complete winning tuple or live descendant under this same
      slice and return the exact classified `RecoveryFinalizeResult` branch)
  match finalizeRecovery:
    committed(recoveryResult): return recoveryResult
    same-body-winner(originalId,recoveryResult):
      return duplicate(originalId,recoveryResult)
    normal-execution-winner(currentHead): return conflict(currentHead)
    identity-reuse(reason): return rejected(reason)
    conflict(currentHead): return conflict(currentHead)
    integrity-failure(report):
      retain all visible state and return internal-failure(
        FailureReport(
          atomic-recovery-finalization-integrity-violation,report))
    warrant-violation(report,finalizeObservation):
      retain latestRecoveringReservation and return internal-failure(
        FailureReport(
          recovery-tail-finalization-warrant-violation,
          report,finalizeObservation))
```

Continuation resume is itself a typed execution request. It binds the original
certificate, query, invocation, selected system, `ContinuationId`, exact
`ContinuationStateRoot`, expected `DeploymentConfiguration`, and a resource
contract.
`RESUME_PRODUCTIVE(C,ExecutionRequest,D,original scope/answer/cert)` requires
`ExecutionSubject=continuation(ContinuationId,ContinuationStateRoot,StepNumber)`,
validates `ContinuationStepTransactionId`, and uses
`continuation-step(id)` for replay/different-body detection. It repeats the same verifier-statement, quarantine,
full-head-CAS, prefix/terminal conformance, effect, conflict, partial, and
violation rules. A finite productive prefix is never reported as final
`Problem_q` acceptance or a completed workload value.

For workloads define

```text
WorkloadRunInput_W = (
  scenario in ValidScenario_W,
  environment =
      adversarial(strategy identity, legal-move/information warrant)
    | stochastic(normalized law/kernel identity,
                 conditioning/correlation identity),
  InitialOrContinuationState,
  RunResourceContractId
).
```

`EXECUTE_USE_CASE_CERTIFIED(C,ExecutionRequest,D,u,WorkloadRunInput_W,answer,cert)` applies the same
exact verifier-statement, authorization, quarantine, complete-runtime-read-set,
effect, continuation, successor-state, and outcome-classification rules. It
requires the request subject to equal
`workload(u.UseCaseRequestId,WorkloadRunInputId)` and equality-checks the full
tagged input, answer, certificate, expected deployment, and run-resource
identities after the same-body replay lookup and before reservation. It
rejects any answer that does not contain exactly one admitted executable
uniform policy under an execution result mode. It
also equality-checks certificate bindings for `u`, `W`, `X_W`, `M_W`,
`QuantifierPrefix_W`, policy universe/scope, result mode, and the one selected
uniform policy; verifies the tagged environment mode, normalized law/kernel,
conditioning/correlation, and scenario membership; and requires
the `(scenario,environment)` pair to belong to
`BoundScenarioEnvironment_W`; and requires
`RuntimeProjection(D)` to equal `InitialDeploymentConfiguration_W` or the exact
certified continuation projection. It executes that one policy under
`JointTransitionSemantics_W` and `InformationFiltration_W`, verifies the
complete workload acceptance/conformance object at a terminal boundary, and
emits the workload receipt of §8.6. It MUST NOT narrow the workload, choose a
different policy with post-hoc information, or treat one sample path as the
complete stochastic behavior.

Observed runtime cost is not retroactively the certified aggregate objective.
Persisting an output into accumulated semantic state requires a new admitted
occurrence and update transaction.

---

## Appendix B. Design invariant

The shortest faithful statement of UOR-GNAF is:

> Preserve the exact typed semantic quotient and provenance; retain the
> canonical accumulated subject and proof-covered operational basis; derive the
> exact observed problem, behavior, machine, objective, quantifier order, and
> use-case boundary; return an attained optimum, identity-complete frontier,
> exact proved-negative result, or honest incompleteness; and certify every
> statement against the complete outer system or uniform-policy universe of one
> immutable snapshot and closed hypothesis set.

This invariant permits unbounded future admission without pretending that a
present machine knows undeclared semantics or can freeze a final optimum against
future candidates. “Any input/use-case” therefore means every member of an
explicit nonempty admitted class under one total class solver and bound machine
semantics; it never means an untyped, assumption-free oracle over unspecified
future meaning.
