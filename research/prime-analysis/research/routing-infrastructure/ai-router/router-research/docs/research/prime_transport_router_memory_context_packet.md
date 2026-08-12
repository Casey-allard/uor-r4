# Prime Transport Router Memory Context Packet

## Purpose

This note records the first bounded context-packet experiment on top of the
unchanged router-memory workspace system.

## Minimal Context Packet

Each control decision now receives one bounded packet with:

- route / entity id
- intent: `read`, `update`, or `resolve`
- payload: the current local record snapshot
- context summary:
  - local record snapshot
  - relevant shared task-ledger fields
  - relevant shared world-ledger fields
  - bounded dependency summary

The memory layer, routing key, read path, and promotion logic all remain
unchanged. Only the controller’s view of retrieved state is structured more
explicitly.

## Aggregate Comparison

- baseline control policy:
  - action correctness: `0.5975357975357976`
  - joint control-loop correctness: `0.4287376200235855`
  - promoted-query fraction on reuse: `0.48843742442001076`
  - route reuse fraction: `0.8411588411588411`
  - instability: `0.0`
- context-packet control policy:
  - action correctness: `0.7577422577422577`
  - joint control-loop correctness: `0.4768426712341106`
  - promoted-query fraction on reuse: `0.48843742442001076`
  - route reuse fraction: `0.8411588411588411`
  - instability: `0.0`

## Reading

### Does introducing a minimal context packet improve control quality?

Yes.

The improvement is material while staying fully bounded:

- action correctness rises by about `0.16020646020646016`
- joint control-loop correctness rises by about `0.048105051210525084`
- effective joint-state resolution rises from `0.7133465916037847` to
  `0.7550088938871817`

So the first control weakness was at least partly a context-structuring
problem, not a memory-coherence problem.

### Does it reduce unnecessary re-query behavior?

Not directly in the current bounded implementation.

The comparison keeps the same memory query pattern, so:

- route reuse is unchanged
- query-hit fraction on reuse is unchanged
- promoted-query burden is unchanged

What the packet changes is not the number of reads, but the controller’s use of
retrieved state. The policy now acts on one bounded structured snapshot instead
of piecing together decision logic from looser per-object state.

### Does it change promoted-query burden?

No.

Both policies show:

- promoted-query fraction on reuse: `0.48843742442001076`
- promotion step fraction: `0.4595404595404595`

So this is a pure control-layer improvement, not a routing or memory-cost
change.

### Does it preserve the memory/workspace stability already demonstrated?

Yes.

The stable properties remain intact:

- per-entity retrieval accuracy: `1.0`
- shared-ledger retrieval accuracy: `1.0`
- route reuse fraction: `0.8411588411588411`
- route decision instability: `0.0`

## Conclusion

The minimal context packet improves bounded control quality without changing
memory/workspace stability, route reuse, or promoted-query burden.

The honest reading is:

- the router-memory substrate was already sound
- the controller benefited from a more explicit structured decision boundary
- promoted querying remains the main explicit efficiency cost

So the next systems step should keep the same memory architecture and treat
context-packet control as the new default bounded policy surface for future
router-native coordination experiments.
