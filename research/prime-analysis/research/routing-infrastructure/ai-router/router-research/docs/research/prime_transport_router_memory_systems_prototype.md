# Prime Transport Router Memory Systems Prototype

## Purpose

This note defines the first larger bounded router-native systems prototype that
should be attempted next on top of the validated router-memory branch.

## Prototype Name

- bounded router-memory workspace prototype

## Why This Is The Right Next Step

The next systems-level step should be larger than the current multi-entity
coordination test, but it should still stay fully bounded and research-only.

The right next prototype is therefore:

- a small workspace-style state machine with several active entities and a
  shared task ledger

This is the right next step because it extends the already validated branch in
the smallest honest systems direction:

- from isolated carried records
- to coordinated records
- to a bounded shared working-memory system

without introducing:

- training
- live runtime integration
- or model replacement

## Smallest Larger System To Build

The prototype should maintain:

- `3-5` active entities
- one shared task/goal ledger
- one shared world-status ledger
- per-entity local records for progress, pressure, and pending work

So the system has both:

- local entity memory
- and shared coordination memory

## State It Maintains

Minimum state objects:

- per-entity record:
  - `goal`
  - `world`
  - `combat`
  - `pressure`
  - `revision`
- shared task ledger:
  - open tasks
  - claimed tasks
  - completed tasks
  - shared revision
- shared dependency summary:
  - which entity depends on which other entity or shared task item

These should all be carried through the current router-memory substrate, not
through a new read path.

## Updates / Queries It Must Support

Required updates:

- partial updates to per-entity state
- partial updates to the shared ledger
- cross-entity changes that depend on a shared task or shared world condition

Required queries:

- read the current state of one entity
- read the current shared ledger
- read a small joint snapshot over several entities plus the shared ledger
- resolve a bounded next-action query that depends on both local and shared
  carried state

Required promotion behavior:

- keep the current promotion logic unchanged
- record when deeper unresolved classes require promoted queries

## Success Metrics

The prototype should record:

- route reuse fraction
- query-hit fraction on reuse
- per-entity retrieval accuracy
- shared-ledger retrieval accuracy
- joint snapshot accuracy
- promoted-query fraction on reuse
- promotion step fraction
- route decision instability
- one bounded overall systems score:
  - effective joint-state resolution fraction

## Failure Conditions

The prototype should count as a failure if any of the following occur:

- carried local state becomes inaccurate
- shared-ledger state becomes inaccurate
- joint snapshot accuracy collapses
- route decision instability becomes nonzero
- promotion burden grows so much that bounded joint-state resolution degrades
  sharply

## Why It Is The Right Next Systems-Level Step

This prototype is the right next step because it tests the missing systems
property directly:

- whether the router-memory branch can support a bounded shared working memory,
  not only isolated or loosely coordinated records

It is still conservative because it does **not** require:

- training
- live evaluation infrastructure
- new benchmark families
- or a redesign of the current read path

So it is the smallest prototype that can honestly answer whether the current
router-memory architecture is ready to grow from bounded coordination into a
larger router-native systems object.
