# Core Project Goals

Purpose: make the repo's actual mission impossible to forget during long
investigations, post-compaction recovery, or local branch churn.

## The Real Goal

This project is not primarily trying to find a slightly better router or a
cleaner evaluation packet.

The goal is to test whether fixed geometry itself can force useful routing and
cheap internal transport strongly enough to replace a meaningful portion of the
dense linear-lattice style routing used in current LLMs.

Target claim:
- geometry-native routing can reduce hardware cost by avoiding large portions
  of dense matrix-routing work
- if the mechanism survives software-side falsification, it could justify
  a later architecture shift beyond transformer-style routing
- long-range upside is on the order of `10x` to `100x` compute or hardware
  improvement, not a small proxy benchmark win

Short form:

`geometry -> routing -> phase / operator structure -> sparse event compute -> hardware savings`

## Architectural Thesis

The intended mathematical object is asymmetric `H^4 x H^4`.

- first `H^4` = routing geometry
- second `H^4` = coupled field
- Hopf angular structure supplies coarse routing geometry
- fiber/phase transport is supposed to be geometry-induced cheap motion
- sparse event routing is supposed to emerge on top of the geometry, not
  replace it

Routing manifold, transport/state field, and retrieval/key field must stay
conceptually separate unless a branch explicitly proves they should be merged.

## Kill-List Order

Every next step must map to one of these stages:

1. hyperbolic embedding stability
2. measure-consistent shell routing
3. Hopf angular correctness
4. phase transport usefulness
5. spectral / operator usefulness
6. sparse event-driven trainability
7. hardware-efficiency confirmation

If a later-stage branch is active while an earlier gate is still unresolved,
the branch must say exactly why that is still worthwhile.

## What Counts As Real Progress

Real progress means at least one of:

- stronger evidence that fixed geometry is the right routing substrate
- stronger evidence that phase transport is geometry-real rather than a tuning
  artifact
- stronger evidence that spectral/operator structure emerges from the geometry
- stronger evidence that sparse event routing can sit on top of the geometry
- stronger evidence that the routed architecture reduces software-side cost in
  a way that could matter for hardware

Helpful but secondary work:

- packet manifests
- carry-forward contracts
- comparison inheritance cleanup
- route-id selection cleanup
- report reshaping

Those are allowed only when they unblock one of the real goals above.

## Drift Warning Signs

Stop and course-correct if the next branch:

- only renames or re-encodes an existing comparison
- creates another extension/comparison/carry-forward artifact without new
  measured evidence
- optimizes packet scope without clarifying a kill-list question
- treats a proxy systems result as if it proved the whole geometric thesis
- forgets to state which mathematical object is actually being tested

## Current Honest State

As of 2026-03-12:

- The repo has meaningful positive evidence for geometry-driven routed
  computation on the product `H^4 x H^4` branch.
- Phase/coupled-field behavior is no longer inert.
- Translated retrieval has produced real software-side systems wins.
- But the project has not yet proven the full moonshot.
- We still do not have a final proof of:
  - learned hyperbolic embedding stability
  - fully measure-consistent shell law
  - end-to-end sparse event trainability as an architecture result
  - transformer replacement

## Dual-Anchor Clarification

`dual anchor` is an evaluation convenience, not a core theory object.

It means:
- one lower-bank routed operating point
- one upper-bank routed operating point

Those anchors are just fixed comparison points used to map systems behavior at
two scales. They are not the mathematical heart of the project.

## Mandatory Questions Before Queuing A New Branch

Before choosing a next step, answer these explicitly:

1. Which kill-list stage does this branch advance?
2. Which mathematical object is being tested?
   - routing manifold
   - shell law
   - Hopf angular law
   - coupled field
   - phase transport
   - spectral/operator structure
   - sparse event mechanism
   - hardware cost surface
3. What concrete result would count as success or falsification?
4. Why is this not just packaging or contract cleanup?

If those answers are weak, the branch is probably drift.

## Recovery Rule

After compaction, interruption, or obvious context drift:

1. read `docs/SESSION_BOOTSTRAP.md`
2. read this file
3. read `docs/PROJECT_CONTEXT.md`
4. read `geometric_routing_kill_tests.md`
5. read `GEOMETRIC_COMPUTATION_HYPOTHESIS.md`
6. read `docs/routes/ROUTE_MATRIX.md`
7. read `docs/DECISIONS.md`
8. then inspect `CURRENT_DIRECTION`, `HANDOFF_CURRENT`, and the latest
   increment docs

Do not pick a next branch until the proposed step can be justified against the
goal and kill list above.
