# Prime Transport Larger Router Memory Experiment

## Purpose

This note records the first larger bounded router-native architecture
experiment using the current router-memory layer with:

- the current memory state
- the current read path
- the current promotion logic
- no refinement redesign

## Experiment

The larger task family is bounded structured record maintenance.

Each reusable route address carries a small record with multiple fields that is
partially edited over time. At every step the experiment:

1. queries the current record at the active memory address
2. checks correctness against the expected carried record
3. applies one partial update
4. writes the updated record back
5. promotes only when the current route class remains unresolved

This is the smallest larger step above the earlier symbolic-payload loop while
still remaining deterministic, bounded, and fully research-only.

## Aggregate Result

Across the bounded larger family:

- memory slots: `169.0`
- route reuse fraction: `0.8151848151848151`
- query hit fraction on reuse: `1.0`
- structured record accuracy: `1.0`
- promoted-query fraction on reuse: `0.48887856062924245`
- promotion step fraction: `0.45274725274725275`
- effective structured resolution fraction: `0.7736263736263738`
- route decision instability: `0.0`

## Reading

### Does the current router-memory architecture still behave coherently on the larger task family?

Yes.

The unchanged architecture remains coherent on the richer task family:

- reads on reused addresses still hit perfectly
- structured record retrieval stays exact
- partial updates do not corrupt carried state
- instability remains `0.0`

So the current memory branch survives the first honest scale-up.

### Does promoted-query burden remain acceptable enough to justify continuing this architecture path?

Yes, with the same explicit caution already seen in the smaller task family.

The burden remains real:

- promoted-query fraction on reuse: `0.48887856062924245`

and the deeper lift family is still the main source of cost.

But the burden does not destroy coherence, retrieval, or stability. So it is
still acceptable as an architectural cost for the next stage of bounded
research work.

### Is this branch now strong enough to justify a future larger router-native systems experiment?

Yes.

The branch is now stronger than it was after the earlier task loop because the
memory layer has demonstrated:

- state carry across repeated revisits
- correct structured-record retrieval
- partial update support
- stable promotion behavior

without changing the read path or adding any new refinement trick.

## Conclusion

The current router-memory architecture is now strong enough to justify a future
larger router-native systems experiment.

The honest framing is:

- coherence is validated
- structured memory behavior is validated
- the main remaining cost is still promoted-query burden on deeper lifts

So the branch should continue forward with the current read path unchanged
unless a qualitatively different refinement design is chosen on purpose.
