# Prime Transport Router Memory Coordination Prototype

## Purpose

This note defines the next larger bounded coordination-logic prototype for the
router-memory branch using the current best controller surface unchanged.

## Fixed Stack

The prototype should use exactly:

- unchanged router-memory substrate
- unchanged routing and promotion behavior
- unchanged packet query pattern and packet fields
- attractor-identity controller surface on top of the packet

## Smallest Harder Task Family

The smallest next task family should be:

- bounded multi-entity dependency-resolution and reassignment loop

Why this is the right next step:

- it is harder than the current agent loop because actions must sometimes
  reassign work, not just advance local stage
- it still stays inside the current workspace objects
- it does not require a new benchmark family
- it stresses coordinated action rather than just local next-step choice

## Entities, Ledgers, And Actions

The prototype should maintain:

- `4` active entities
- one shared task / goal ledger
- one shared world-status ledger
- one shared dependency ledger
- per-entity local records

The prototype should support bounded actions such as:

- `claim_goal`
- `sync_world`
- `engage_combat`
- `stabilize_pressure`
- `mark_complete`
- `hold_claim`
- `hold_combat`
- `reassign_claim`
- `handoff_dependency`
- `idle`

The added difficulty comes from:

- conflict between multiple eligible claimers
- reassignment when one entity is blocked
- dependency handoff across entities
- preserving shared-ledger consistency after reassignment

## What It Must Test

This prototype should test whether the current best controller surface can
support:

- repeated local decisions
- repeated shared-state reads
- bounded reassignment logic
- dependency-aware action coordination
- exact local and shared carried-state maintenance under harder control flow

## Success Metrics

The prototype should record:

- action correctness
- reassignment correctness
- shared-ledger consistency accuracy
- joint coordination-loop correctness
- route reuse
- promoted-query fraction on reuse
- local retrieval accuracy
- shared-ledger retrieval accuracy
- instability
- one bounded overall score:
  - effective coordinated-state resolution fraction

## Failure Conditions

The prototype should count as a failure if any of the following happen:

- local retrieval degrades
- shared-ledger retrieval degrades
- reassignment logic collapses shared consistency
- joint coordination correctness falls back toward the earlier weak-controller
  regime
- instability becomes nonzero

Promoted-query burden remaining high is **not** by itself a failure here,
unless it accompanies degraded coordinated-state correctness.

## Why This Is The Right Next Systems-Level Step

This is the right next prototype because it increases coordination difficulty
without changing the underlying architecture:

- the memory substrate is already validated
- the packet surface is already validated
- the attractor identity is already validated as a control aid

So the missing question is now:

- can the current best controller surface handle bounded reassignment and
  dependency coordination, not just bounded stepwise control

That makes this the smallest honest next prototype before any broader
router-native systems expansion.
