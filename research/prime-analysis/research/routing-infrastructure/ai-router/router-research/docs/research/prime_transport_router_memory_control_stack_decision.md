# Prime Transport Router Memory Control Stack Decision

## Current Decision

This note freezes the current control-stack conclusion for the router-memory
branch.

## What Is Now Established

The controller-side sequence is now clear:

- the packet surface improves controller quality materially over the baseline
  retrieved-state controller
- the improved packet controller improves it further through better bounded
  action ranking
- the attractor-identity layer improves controller quality materially again
  while preserving exact retrieval and zero instability

At the same time, one fact remains unchanged:

- attractor identity does **not** reduce promoted-query burden on the current
  bounded stack

So the current best stack is now:

- unchanged router-memory substrate
- unchanged promotion / query behavior
- packet plus attractor identity as the controller surface

## Frozen Reading

What this stack now validates:

- memory coherence is stable
- shared workspace behavior is stable
- controller quality can improve materially without touching routing, memory,
  packet query pattern, or promotion logic
- convergence identity is useful as a controller-side aid

What it does **not** yet validate:

- a reduction in the main explicit efficiency cost

That cost remains:

- promoted-query burden on deeper lift families

## Decision

The router-memory branch should now carry forward the following control stack
as the primary bounded controller surface:

- router-memory workspace substrate unchanged
- packet surface unchanged
- attractor-identity controller surface enabled

The older controller variants should now be treated as comparison baselines,
not as the primary path.

## Consequence

The next systems-level step should no longer ask:

- whether the packet surface is useful
- whether controller quality can improve without memory redesign

Those questions are now answered positively.

The next step should instead ask:

- whether this best current controller surface scales to a harder bounded
  coordination-logic task while the same promoted-query cost remains explicit
