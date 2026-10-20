# Session Resolution

## Definition

**Session resolution** is the multi-turn inference protocol in which a sequence
of {@class https://uor.foundation/query/RelationQuery} evaluations shares a
common {@class https://uor.foundation/state/Context}. Each resolved query
appends a {@class https://uor.foundation/state/Binding} to a
{@class https://uor.foundation/state/BindingAccumulator}, monotonically
reducing the aggregate free site space for subsequent queries.

A {@class https://uor.foundation/state/Session} is the bounded collection of
query/response pairs that share this accumulator. Sessions are the unit of
coherent multi-turn reasoning in Prism deployments.

## Monotonicity Invariant

The binding monotonicity invariant (GR\_1) guarantees:

> freeRank(B\_{i+1}) ≤ freeRank(B\_i) for all i in a Session S.

The {@prop https://uor.foundation/state/aggregateSiteDeficit} on the
BindingAccumulator tracks the total remaining free sites across all accumulated
bindings. This value can only decrease as the session progresses — accumulated
knowledge cannot increase uncertainty.

## SessionResolver

A {@class https://uor.foundation/resolver/SessionResolver} is the top-level
resolver for multi-turn Prism deployments. It maintains the BindingAccumulator
across evaluations via the
{@prop https://uor.foundation/resolver/sessionAccumulator} property.

## SessionQuery

A {@class https://uor.foundation/query/SessionQuery} extends RelationQuery
by explicitly declaring session membership via the
{@prop https://uor.foundation/query/sessionMembership} property. This
allows the conformance suite to validate session-scoped site reduction.

## Session Boundaries

A {@class https://uor.foundation/state/SessionBoundary} marks a context-reset
event within a session stream. The {@class https://uor.foundation/state/SessionBoundaryType}
vocabulary enum classifies the reason:

| Individual | Meaning |
|------------|---------|
| `state:ExplicitReset` | Caller requested context reset |
| `state:ConvergenceBoundary` | No further queries can reduce the aggregate site deficit |
| `state:ContradictionBoundary` | New query produced a type contradiction with an accumulated binding |

Each boundary records the {@prop https://uor.foundation/state/priorContext}
(the state before reset) and the {@prop https://uor.foundation/state/freshContext}
(the clean state for subsequent queries).

## Session Identity Algebra (GR_ series)

Amendment 27 adds five GR\_ identity individuals formalizing the session
resolution algebra:

| Identity | Statement |
|----------|-----------|
| GR\_1 | Binding monotonicity: freeRank(B\_{i+1}) ≤ freeRank(B\_i) |
| GR\_2 | Empty session is identity: freeRank(B\_0) = total site space |
| GR\_3 | Convergence: session terminates iff freeRank reaches minimum |
| GR\_4 | Disjoint bindings compose without site conflict |
| GR\_5 | ContradictionBoundary fires iff ∃ conflicting bindings at same address |

Amendment 44 adds two more GR\_ identities for session-level coherence:

| Identity | Statement |
|----------|-----------|
| GR\_6 | Session join monotonicity: freeRank(join(S\_A, S\_B)) ≤ freeRank(S\_A) + freeRank(S\_B) |
| GR\_7 | Session inclusion: S\_A ⊆ S\_B implies freeRank(S\_A) ≥ freeRank(S\_B) |

Amendment 48 adds three axiomatic and eight derivational identities for multi-session coordination:

| Identity | Statement |
|----------|-----------|
| GR\_8 | Composition validity: compose(S\_A, S\_B) valid at Q\_k iff datum agreement holds at every tower level Q\_0…Q\_k |
| GR\_9 | ContextLease disjointness: leasedSites(L\_A) ∩ leasedSites(L\_B) = ∅ for distinct leases on the same SharedContext |
| GR\_10 | ExecutionPolicy confluence: different policies produce the same final resolved state (Church–Rosser) |
| MC\_1 | Lease partition conserves budget: Σᵢ freeRank(L\_i) = freeRank(C) |
| MC\_2 | Per-lease binding monotonicity: GR\_1 holds within each leased sub-domain |
| MC\_3 | Composition freeRank via inclusion-exclusion |
| MC\_4 | Disjoint-lease additivity: freeRank(compose(S\_A, S\_B)) = freeRank(S\_A) + freeRank(S\_B) when GR\_9 holds |
| MC\_5 | Policy-invariant final binding set |
| MC\_6 | Full lease coverage implies composed grounding (σ = 1) |
| MC\_7 | Distributed O(1) resolution: stepCount = 0 on a fully grounded composed context |
| MC\_8 | Parallelism bound: per-session work ≤ ⌈n/k⌉ for k uniform-partition leases |

## Multi-Session Coordination (Amendment 48)

### SharedContext and ContextLease

A {@class https://uor.foundation/state/SharedContext} is a Context visible to more
than one Session simultaneously. Concurrent write conflicts are prevented by
partitioning its sites across {@class https://uor.foundation/state/ContextLease}
instances.

Each ContextLease holds:
- {@prop https://uor.foundation/state/leasedSites} — exclusive site sub-domain (a {@class https://uor.foundation/partition/FreeRank})
- {@prop https://uor.foundation/state/leaseHolder} — the owning Session

The SharedContext links active leases via
{@prop https://uor.foundation/state/leaseSet}. Lease disjointness (GR\_9)
is enforced structurally: each lease claims a non-overlapping FreeRank.

### SessionComposition

A {@class https://uor.foundation/state/SessionComposition} records that a Session
was formed by merging the binding sets of two or more predecessor sessions:

- {@prop https://uor.foundation/state/composedFrom} — predecessor sessions (non-functional, multiple allowed)
- {@prop https://uor.foundation/state/compositionCompatible} — `true` iff datum agreement holds at all tower levels (GR\_8)
- {@prop https://uor.foundation/state/compositionResult} — the merged Context produced on success
- {@prop https://uor.foundation/state/towerConsistencyVerified} — `true` iff tower check was run for Q\_1+ sessions

An invalid composition (compositionCompatible = false) triggers a ContradictionBoundary on
the target session.

### ExecutionPolicy

A {@class https://uor.foundation/resolver/ExecutionPolicy} defines how a
{@class https://uor.foundation/resolver/SessionResolver} orders pending
RelationQuery instances, linked via
{@prop https://uor.foundation/resolver/executionPolicy}.

The {@class https://uor.foundation/resolver/ExecutionPolicyKind} enum provides
four scheduling strategies:

| Individual | Strategy |
|------------|----------|
| `resolver:FifoPolicy` | Arrival order (pre-Amendment 48 implicit default) |
| `resolver:MinFreeRankFirst` | Cheapest resolutions first — favours early grounding gain |
| `resolver:MaxFreeRankFirst` | Hardest resolutions first — maximises information gain per step |
| `resolver:DisjointFirst` | Site-disjoint queries first — minimises SharedContext contention |

All four strategies are policy-confluent (GR\_10): they converge to identical final binding sets.

## Related

- {@class https://uor.foundation/state/Session}
- {@class https://uor.foundation/state/BindingAccumulator}
- {@class https://uor.foundation/state/SessionBoundary}
- {@class https://uor.foundation/resolver/SessionResolver}
- [Type Completeness](type-completeness.html)
