# Prime Transport Router Memory Workspace Experiment

## Purpose

This note records the first bounded router-memory workspace-system prototype
using the current memory read path and promotion logic unchanged.

## Aggregate Result

Across the bounded workspace bundles:

- entity count: `3.0`
- route reuse fraction: `0.8411588411588411`
- query-hit fraction on reuse: `1.0`
- per-entity retrieval accuracy: `1.0`
- shared-ledger retrieval accuracy: `1.0`
- joint snapshot accuracy: `1.0`
- promoted-query fraction on reuse: `0.48843742442001076`
- promotion step fraction: `0.4595404595404595`
- effective joint-state resolution fraction: `0.8851148851148851`
- route decision instability: `0.0`

## Reading

### Does the unchanged router-memory architecture remain coherent at bounded workspace-system scope?

Yes.

The current architecture stays coherent when it must jointly maintain:

- per-entity local records
- a shared task ledger
- a shared world-status ledger
- shared dependency state

under repeated writes, repeated reads, and bounded carried updates.

The key stability facts are:

- all reused-address reads still hit
- per-entity retrieval remains exact
- shared-ledger retrieval remains exact
- joint workspace snapshots remain exact
- route decision instability remains `0.0`

So this branch now supports a bounded shared workspace system, not just a
memory substrate in isolation.

### Does promoted-query burden remain acceptable enough to keep this branch moving?

Yes, with the same explicit caution already established on the branch.

The burden remains real:

- promoted-query fraction on reuse: `0.48843742442001076`
- promotion step fraction: `0.4595404595404595`

and the deeper-lift bundle continues to carry most of that cost.

But that burden does not break correctness, carried-state coherence, or
stability. At this stage it remains an efficiency cost, not a failure mode.

### Is the branch now strong enough to justify a future larger router-native systems experiment beyond this bounded workspace prototype?

Yes.

The branch has now validated, under one unchanged memory architecture:

- structured read/write memory behavior
- bounded stateful task-loop success
- richer record-maintenance success
- bounded multi-entity coordination success
- bounded workspace-style local-plus-shared state maintenance

all with `0.0` instability.

## Conclusion

The router-memory branch is now strong enough to justify a future larger
router-native systems experiment beyond this bounded workspace prototype.

The conservative reading is:

- workspace-scope coherence is validated
- local and shared retrieval are exact on reused addresses
- promoted-query burden remains the main explicit architectural cost
- no read-path redesign is currently required just to keep the branch moving

So the next larger systems step can proceed on the current architecture
unchanged unless the next question is specifically about reducing promoted-query
cost rather than testing larger-scale coherence.
