# Resolution & Queries

Resolution is the core operation of the [PRISM](../pipeline/) pipeline's Resolve stage.
A {@class https://uor.foundation/query/Query} specifies what to resolve; a
{@class https://uor.foundation/resolver/Resolver} computes the answer by factorizing
the input under the dihedral group D_{2^n}.

## The Query-Resolver Pattern

The resolution process follows a structured pattern:

1. A **query** specifies the question: what coordinates does this element have?
   What is its canonical representation? What metrics apply?
2. A **resolver** takes the query and the input element, then computes the answer
   using ring arithmetic and dihedral group factorization.
3. A **refinement suggestion** is produced if the input is only partially constrained,
   guiding the next iteration toward a complete resolution.

The {@class https://uor.foundation/query/Query} class in the {@ns query} namespace
defines the query interface. The {@class https://uor.foundation/resolver/Resolver}
class in the {@ns resolver} namespace defines the resolver interface.

## Iterative Resolution

Not all inputs are fully constrained. When the {@concept site} budget is not yet
closed (some sites remain unpinned), the resolver cannot produce a complete answer
in one step. Instead, it enters an iterative loop:

1. Attempt resolution with current constraints.
2. If incomplete, produce a {@class https://uor.foundation/resolver/RefinementSuggestion}
   identifying which additional constraints would help.
3. Apply the refinement (pin more sites) and retry.

The {@class https://uor.foundation/resolver/CompletenessResolver} manages this loop,
tracking which constraints have been satisfied and which remain open.

## The Constraint Nerve

When constraints interact in complex ways, the
{@class https://uor.foundation/resolver/CechNerve} provides a topological
view of the constraint space. It is a simplicial complex whose simplices represent
compatible subsets of constraints. The topology of this nerve reveals whether
resolution will converge -- see {@concept homology} for the algebraic topology
analysis.

## Specialized Resolvers

The {@ns resolver} namespace includes several specialized resolvers:

- **WittLevelResolver** -- resolves which {@concept witt-levels} an identity
  holds at.
- **TypeSynthesisResolver** -- inverts the resolution pipeline, computing what type
  is needed given a constraint.
- **SessionResolver** -- resolves within a session scope, accumulating bindings
  across steps.
- **GeodesicValidator** -- verifies that a computation follows the shortest path
  in the derivation space.
- **MeasurementResolver** -- handles quantum measurement events that collapse
  superposed site states.

## Connection to the PRISM Pipeline

Resolution is the heart of the Resolve stage. It takes the definitions from
{@concept content-addressing} and {@concept partition}, applies queries, and
produces evidence that flows to the {@concept proof-system} for certification.
{@concept observables} are measured during resolution, recording the geometric
and algebraic properties of each computation step.
