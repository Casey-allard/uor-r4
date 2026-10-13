# Prime Transport Coordination Policy Improvement

## Bounded Failure Mode

The most likely bounded coordination-policy failure mode on the current stack
is:

- over-eager transfer under partial dependency visibility

More concretely, the current coordination controller promotes:

- `reassign_claim`
- `handoff_dependency`

too early whenever dependency count is high, even in states where the current
entity still has directly progressable local work:

- `sync_world`
- `engage_combat`
- `stabilize_pressure`
- `mark_complete`

That makes the controller more brittle to slightly stale shared context than it
needs to be, especially on the shallower bounded bundle where promoted-query
pressure is lower but full-loop coordination is weaker.

## Improved Policy

The tested improvement keeps the full stack unchanged:

- same router-memory substrate
- same packet surface
- same attractor identity
- same query / promotion behavior

The only change is controller ordering:

- direct local dependency-resolution actions are chosen before transfer actions
- reassignment and handoff are delayed until local progress paths are exhausted

So this is a controller-only change, not a stack redesign.

## Aggregate Comparison

- current coordination policy:
  - action correctness: `0.8075674325674326`
  - reassignment / handoff correctness: `0.9285714285714286`
  - per-entity retrieval accuracy: `1.0`
  - shared-ledger retrieval accuracy: `1.0`
  - joint coordination-loop correctness: `0.4391505191725942`
  - promoted-query fraction on reuse: `0.5047922252391286`
  - route reuse fraction: `0.8425859854431283`
  - instability: `0.0`
- improved coordination policy:
  - action correctness: `0.8164585414585415`
  - reassignment / handoff correctness: `1.0`
  - per-entity retrieval accuracy: `1.0`
  - shared-ledger retrieval accuracy: `1.0`
  - joint coordination-loop correctness: `0.4549983440244225`
  - promoted-query fraction on reuse: `0.5047922252391286`
  - route reuse fraction: `0.8425859854431283`
  - instability: `0.0`

## Reading

### Can coordination quality be improved materially without changing the current stack?

Not materially overall on this bounded step, but yes directionally.

The improved policy still raises:

- action correctness from `0.8075674325674326` to `0.8164585414585415`
- reassignment / handoff correctness from `0.9285714285714286` to `1.0`
- joint coordination-loop correctness from `0.4391505191725942` to
  `0.4549983440244225`

The important bounded reading is:

- both bundles move in the right direction
- reassignment / handoff decisions become exact on this comparison
- but full-loop coordination quality only improves modestly

So the controller bottleneck is real and policy-only improvement is possible,
but harder coordination is still not solved by this one bounded change.

### Does the improved policy preserve the stability and retrieval quality already validated?

Yes.

The substrate-side properties remain fixed:

- per-entity retrieval accuracy stays `1.0`
- shared-ledger retrieval accuracy stays `1.0`
- promoted-query fraction on reuse stays `0.5047922252391286`
- route reuse stays `0.8425859854431283`
- route decision instability stays `0.0`

So the gain is a controller-quality gain, not a hidden architecture change.

### Is the branch then strong enough to justify a future larger bounded router-native systems prototype beyond the current coordination demo?

Yes, conservatively.

The current branch-level reading is now:

- the unchanged stack remains coherent
- the best current controller surface supports modestly better bounded
  coordination quality
- promoted-query burden remains explicit but unchanged

That is strong enough to justify a future larger bounded router-native systems
prototype beyond the current coordination demo.

## Conclusion

The bounded coordination controller can be improved modestly on the current
unchanged router-native stack by reconciling local record stage against stale
shared-ledger flags and delaying transfer actions until direct local
dependency-resolution paths are exhausted.

That improves:

- coordination decisions
- reassignment / handoff quality
- full-loop coordination quality

while preserving exact retrieval and zero instability.

So the branch is ready to move forward on the same stack, with future work
still focused on coordination-policy quality rather than on substrate repair.
