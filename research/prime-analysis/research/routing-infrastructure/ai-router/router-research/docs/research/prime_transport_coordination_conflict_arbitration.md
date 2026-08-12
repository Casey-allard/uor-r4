# Prime Transport Coordination Conflict Arbitration

## Bounded Failure Mode

The most likely remaining bounded coordination failure mode on the current
stack is:

- weak conflict and dependency arbitration across entity actions

The current improved local controller is already reasonably good at:

- local stage progression
- transfer timing under direct dependency visibility

and the rejected one-step lookahead showed that shallow replanning does not
recover more quality.

That points to a more structural controller gap:

- multiple entities can sit in the same shared conflict surface
- the controller may choose an action that is locally valid but not the one
  that best relieves the currently hottest shared blockage

So the next bounded step is to test explicit shared conflict arbitration on the
same unchanged packet + attractor surface.

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
- conflict / dependency arbitration policy:
  - action correctness: `0.8162837162837163`
  - reassignment / handoff correctness: `1.0`
  - joint coordination-loop correctness: `0.45384747564191574`
  - per-entity retrieval accuracy: `1.0`
  - shared-ledger retrieval accuracy: `1.0`
  - promoted-query fraction on reuse: `0.5047922252391286`
  - route reuse fraction: `0.8425859854431283`
  - instability: `0.0`

## Reading

### Does explicit conflict/dependency arbitration improve coordination quality materially on the same unchanged stack?

No.

On this bounded comparison, the explicit conflict / dependency arbitration
controller does not beat the current improved local policy:

- action correctness falls slightly from `0.8164585414585415` to
  `0.8162837162837163`
- joint coordination-loop correctness falls slightly from `0.4549983440244225`
  to `0.45384747564191574`
- reassignment / handoff correctness stays `1.0`

So this arbitration variant does **not** improve coordination quality
materially on the unchanged stack.

### Does it preserve the stability and retrieval quality already validated?

Yes.

All validated substrate-side properties remain fixed:

- per-entity retrieval accuracy remains `1.0`
- shared-ledger retrieval accuracy remains `1.0`
- promoted-query fraction on reuse remains `0.5047922252391286`
- route reuse remains `0.8425859854431283`
- instability remains `0.0`

### Does it outperform both the improved local policy and the rejected one-step lookahead variant?

No.

It does not beat the improved local coordination policy, and on this bounded
comparison it matches the rejected one-step lookahead variant exactly on the
aggregate row.

That is a useful negative result:

- the remaining controller headroom is not unlocked by this style of explicit
  arbitration score on the current packet surface
- another small controller tweak in the same family is unlikely to be the next
  productive move

### Is the branch then strong enough to justify a larger bounded router-native systems prototype with harder coordination logic?

Yes, conservatively.

The branch is still strong enough because:

- the stack remains coherent and exact on retrieval
- the improved local controller remains the best bounded controller tested so
  far
- these negative controller variants help bound what is not worth pushing
  further on the current surface

## Conclusion

The tested conflict / dependency arbitration controller does **not** improve
coordination quality materially on the same unchanged router-native stack.

It preserves all validated substrate behavior, but it does not outperform the
current improved local coordination policy and it collapses to the same
aggregate outcome as the rejected one-step lookahead variant.

The honest branch-level reading is:

- the substrate remains stable
- the improved local policy is still the best bounded coordination controller
  on this stack
- the next larger step should not be another small arbitration or shallow
  replanning tweak on the same controller surface
