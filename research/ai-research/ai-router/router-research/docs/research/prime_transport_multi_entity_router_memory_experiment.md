# Prime Transport Multi-Entity Router Memory Experiment

## Purpose

This note records the first bounded multi-entity router-native systems
experiment using the current router-memory layer unchanged.

## Aggregate Result

Across the bounded multi-entity bundles:

- entity count: `3.0`
- route reuse fraction: `0.8411588411588411`
- query-hit fraction on reuse: `1.0`
- multi-record retrieval accuracy: `1.0`
- coordination snapshot accuracy: `1.0`
- promoted-query fraction on reuse: `0.48843742442001076`
- promotion step fraction: `0.4595404595404595`
- effective multi-entity resolution fraction: `0.8468198468198469`
- route decision instability: `0.0`

## Reading

### Does the unchanged router-memory architecture remain coherent on multi-entity state coordination?

Yes.

The unchanged architecture remains coherent even when several active structured
records must be maintained and queried together:

- all reused-address reads hit
- multi-record retrieval remains exact
- coordinated snapshot reads remain exact
- instability remains `0.0`

So the memory layer now supports bounded multi-entity state coordination, not
just single-record carry.

### Does promoted-query burden remain acceptable enough to continue this path?

Yes, with the same explicit caution that already existed on the single-entity
branch.

The burden remains real:

- promoted-query fraction on reuse: `0.48843742442001076`

and the deeper-lift bundle still carries most of that cost.

But that burden does not break correctness or stability. It remains an
architectural efficiency cost, not a coherence failure.

### Is the branch now strong enough to justify a future larger router-native systems prototype?

Yes.

The branch has now validated:

- single-record memory carry
- richer structured-record maintenance
- multi-entity coordinated retrieval

all under the same unchanged memory architecture.

## Conclusion

The router-memory branch is now strong enough to justify a future larger
router-native systems prototype.

The correct conservative reading is:

- coherence is validated
- exact multi-record retrieval is validated
- exact coordinated snapshot behavior is validated
- promoted-query burden remains the main explicit cost

So the next larger architectural step can proceed with the current read path
unchanged unless a deliberately different refinement design is chosen for cost
reduction rather than for correctness repair.
