# Prime Transport Router Memory Coordination Experiment

## Purpose

This note records the first harder bounded coordination-logic prototype on the
current validated stack:

- unchanged router-memory substrate
- unchanged promotion / query behavior
- packet plus attractor identity as the controller surface

## Aggregate Result

Across the bounded four-entity coordination bundles:

- entity count: `4.0`
- action correctness: `0.8075674325674326`
- reassignment / handoff correctness: `0.9285714285714286`
- per-entity retrieval accuracy: `1.0`
- shared-ledger retrieval accuracy: `1.0`
- joint coordination-loop correctness: `0.4391505191725942`
- promoted-query fraction on reuse: `0.5047922252391286`
- route reuse fraction: `0.8425859854431283`
- promotion step fraction: `0.47580990438133297`
- effective coordinated-state resolution fraction: `0.783246579321687`
- route decision instability: `0.0`

## Reading

### Does the current best stack remain coherent under harder coordination logic?

Yes on the substrate side.

The stable properties remain intact even under the harder reassignment and
handoff loop:

- per-entity retrieval accuracy remains `1.0`
- shared-ledger retrieval accuracy remains `1.0`
- route reuse stays high at `0.8425859854431283`
- route decision instability remains `0.0`

So the current best stack remains coherent as a bounded coordination system.

### Does the packet plus attractor identity controller surface still scale meaningfully?

Yes, but only partially.

The controller surface still supports:

- strong action correctness: `0.8075674325674326`
- strong reassignment / handoff correctness: `0.9285714285714286`

But the harder full-loop coordination metric is much weaker:

- joint coordination-loop correctness: `0.4391505191725942`

The split is informative:

- the shallower bundle remains much weaker on full-loop coordination
- the deeper bundle remains much stronger on full-loop coordination

So the controller surface scales meaningfully beyond the earlier agent loop,
but not yet uniformly across the bounded coordination task.

### Does promoted-query burden remain acceptable enough to justify a future larger router-native systems prototype?

Yes, with the same explicit caution already established on the branch.

The burden remains real:

- promoted-query fraction on reuse: `0.5047922252391286`
- promotion step fraction: `0.47580990438133297`

That is slightly higher than the earlier control-loop runs, but it does not
break retrieval or stability. It remains an explicit efficiency cost, not a
coherence failure.

### Is the branch now strong enough to justify a bounded system-level architecture paper/demo inside the research tree?

Yes, as a bounded internal architecture demo.

The branch now supports a credible bounded systems story:

- exact carried-state retrieval
- exact shared-ledger retrieval
- stable route behavior
- materially improved controller quality from packet plus attractor identity
- nontrivial reassignment and handoff behavior under harder coordination logic

But the current evidence does **not** support a stronger claim that bounded
coordination is uniformly solved. The full-loop coordination metric is still
mixed enough that this should be treated as:

- a bounded internal system-level architecture demo

not as:

- a broad mature coordination result

## Conclusion

The current best stack remains coherent under harder bounded coordination logic
and is strong enough to justify a bounded system-level architecture demo inside
the research tree.

The conservative branch-level reading is:

- the substrate is stable
- the best controller surface still scales meaningfully
- promoted-query burden remains the main explicit efficiency cost
- harder full-loop coordination is now the main controller-side challenge

So the next larger step can proceed, but it should focus on bounded
coordination-quality improvement rather than on memory or routing repair.
