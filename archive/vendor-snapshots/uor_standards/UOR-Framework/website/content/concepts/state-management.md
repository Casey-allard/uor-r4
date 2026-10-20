# State, Sessions, and Accumulation

The {@ns state} namespace models the mutable side of the UOR framework.
While the kernel is immutable and the bridge is purely computed, state
captures what happens when a resolver accumulates bindings across a
sequence of queries.

## Contexts and Bindings

A {@class https://uor.foundation/state/Context} is a bounded set of
populated UOR addresses — the parameter space for a resolution cycle.
Each {@class https://uor.foundation/state/Binding} associates a datum
value with an address, and a {@class https://uor.foundation/state/Frame}
determines which bindings are visible to a given resolution step.

State changes are modeled explicitly: a
{@class https://uor.foundation/state/Transition} records the
transformation of one context into another through binding or unbinding.

## Sessions

A {@class https://uor.foundation/state/Session} is a bounded sequence
of query/response pairs sharing a common context. As queries arrive,
the {@class https://uor.foundation/state/BindingAccumulator} appends
bindings to the session context, progressively narrowing the set of
free sites.

Sessions can end for several reasons, each classified by the
{@class https://uor.foundation/state/SessionBoundaryType} vocabulary:
explicit caller reset, convergence (no further progress possible),
or contradiction (a new binding conflicts with an existing one).

## Grounding

The goal of accumulation is grounding — the point at which every
site is pinned and no free coordinates remain. The
{@class https://uor.foundation/state/GroundedContext} represents a
context that has reached full grounding, with grounding degree
sigma = 1, freeRank = 0, and residual entropy S = 0.

The grounding process is tracked through
{@class https://uor.foundation/state/GroundingPhase} phases:
Open (sigma = 0), PartialGrounding (0 < sigma < 1), and
FullGrounding (sigma = 1). The
{@class https://uor.foundation/state/GroundingWitness} records
step-by-step evidence of how grounding was achieved.

## Shared State and Composition

Multiple sessions can share state. A
{@class https://uor.foundation/state/SharedContext} is visible to more
than one session simultaneously, with
{@class https://uor.foundation/state/ContextLease} providing exclusive
claims on site coordinates. Sessions themselves can compose: the
{@class https://uor.foundation/state/SessionComposition} records when
a session was formed by merging the binding sets of predecessors.

## Connection to Resolution

State is the bridge between the {@concept resolution} pipeline and the
certification layer. As a session accumulates bindings, the resolver
uses the current context to guide its search. When grounding is reached,
the result can be certified — connecting state management to the
{@concept certification} workflow.
