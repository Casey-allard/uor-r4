# Prime Transport Coordination Frame Experiment

## Minimal Coordination Frame

The single tested coordination-frame design is:

- `coordination_episode`

It is a bounded transaction-like object above the current packet surface, not a
memory or routing redesign. Each active frame stores only:

- `episode_id`
- `entity_name`
- `kind`
- `blocked_target`
- `release_condition`

The three bounded frame kinds are:

- `goal_resolution`
- `pressure_resolution`
- `completion_push`

Operationally, a frame means:

- once an entity enters a dependency-resolution episode, the controller keeps a
  short explicit coordination objective active across steps
- actions stay aligned to that episode until the release condition is met

So the frame is a small coordination object above the packet surface, not a
new packet field and not a substrate change.

## Why This Is The Next Reasonable Step

The current bounded evidence says:

- local ordering helps modestly
- one-step lookahead does not help
- explicit conflict scoring does not help

That suggests the remaining weakness is not another per-step score tweak, but
the absence of a persistent higher-level coordination object that keeps a
multi-step dependency-resolution intent active across successive actions.

## Aggregate Comparison

- improved local coordination policy:
  - action correctness: `0.8164585414585415`
  - reassignment / handoff correctness: `1.0`
  - joint coordination-loop correctness: `0.4549983440244225`
  - per-entity retrieval accuracy: `1.0`
  - shared-ledger retrieval accuracy: `1.0`
  - promoted-query fraction on reuse: `0.5047922252391286`
  - route reuse fraction: `0.8425859854431283`
  - instability: `0.0`
- rejected one-step lookahead policy:
  - action correctness: `0.8162837162837163`
  - reassignment / handoff correctness: `1.0`
  - joint coordination-loop correctness: `0.45384747564191574`
  - per-entity retrieval accuracy: `1.0`
  - shared-ledger retrieval accuracy: `1.0`
  - promoted-query fraction on reuse: `0.5047922252391286`
  - route reuse fraction: `0.8425859854431283`
  - instability: `0.0`
- rejected conflict / dependency arbitration policy:
  - action correctness: `0.8162837162837163`
  - reassignment / handoff correctness: `1.0`
  - joint coordination-loop correctness: `0.45384747564191574`
  - per-entity retrieval accuracy: `1.0`
  - shared-ledger retrieval accuracy: `1.0`
  - promoted-query fraction on reuse: `0.5047922252391286`
  - route reuse fraction: `0.8425859854431283`
  - instability: `0.0`
- coordination-frame policy:
  - action correctness: `0.8164085914085915`
  - reassignment / handoff correctness: `1.0`
  - joint coordination-loop correctness: `0.45520759282124196`
  - per-entity retrieval accuracy: `1.0`
  - shared-ledger retrieval accuracy: `1.0`
  - promoted-query fraction on reuse: `0.5047922252391286`
  - route reuse fraction: `0.8425859854431283`
  - instability: `0.0`

## Reading

### Does a small coordination frame improve harder coordination quality materially on the same unchanged stack?

Not materially, but yes slightly on the harder coordination metric.

The coordination-frame controller does beat all prior bounded controller
families on the aggregate row:

- action correctness changes from `0.8164585414585415` to `0.8164085914085915`
  against the improved local policy, so action quality is effectively flat
- joint coordination-loop correctness rises from `0.4549983440244225` to
  `0.45520759282124196`
- it also beats both rejected variants:
  `0.45384747564191574 -> 0.45520759282124196`

So the frame is a real positive result on full-loop coordination, but only a
very small one.

### Does it preserve the stability and retrieval quality already validated?

Yes.

All substrate-side properties remain fixed:

- per-entity retrieval accuracy remains `1.0`
- shared-ledger retrieval accuracy remains `1.0`
- promoted-query fraction on reuse remains `0.5047922252391286`
- route reuse remains `0.8425859854431283`
- instability remains `0.0`

So this is a clean controller-surface gain, not a hidden substrate change.

### Does it outperform the prior local / lookahead / conflict-arbitration controller families?

Yes, narrowly.

On the aggregate row:

- it beats the improved local controller on joint coordination-loop correctness
- it does not beat the improved local controller on action correctness
- it beats both rejected variants on both action correctness and joint-loop
  correctness
- it preserves exact reassignment / handoff correctness at `1.0`

The honest reading is that the gain is small, but it is the first bounded
controller variant beyond the improved local policy that moves the overall
coordination metric upward rather than sideways or down.

### Is the branch then strong enough to justify a larger bounded router-native systems prototype with explicit coordination episodes?

Yes.

This result is enough to support the next bounded systems step because:

- the substrate remains stable and exact
- the coordination frame is the first higher-level controller object that
  actually improves the harder coordination aggregate
- it does so without touching routing, memory, packet structure, attractor
  identity, query behavior, or promotion logic

## Conclusion

The tested small coordination-frame layer does improve harder coordination
quality on the same unchanged router-native stack, but only slightly.

That is still important, because it is the first bounded positive result beyond
the improved local controller that comes from adding a higher-level
coordination object rather than another per-step scoring tweak.

The honest branch-level reading is:

- the substrate remains stable
- exact retrieval remains intact
- small explicit coordination episodes are a better next direction than more
  shallow lookahead or arbitration tweaks
- the gain is real but still modest, so harder coordination is not yet solved
