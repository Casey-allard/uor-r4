# Prime Transport Real-Signal Status

## Current Real-Signal Conclusion

This note freezes the current real-signal status of the prime-transport routing
policy on the guarded replay-based MUDBench benchmark boundary.

The policy under evaluation is unchanged:

- stored state: `R_min = (b, phi, r, next_return_gap)`
- routing key: `base_gap = (b, r, next_return_gap)`
- fallback: `R_full = (b, spin_H)`

## What Is Established on Real Signal

Across the current guarded replay-based checks:

- `base_gap` remains stable
- fallback advantage over guarded angular survives
- effective resolution remains full
- route-decision instability remains `0.0`

This now holds on both:

- the first single-replay guarded real-signal trial
- the paired larger replay slice selected for the next guarded reuse test

The current bounded aggregate numbers are:

### Single-replay trial

- `base_gap`:
  - route reuse fraction: `0.0`
  - fallback step fraction: `0.0`
  - effective resolved fraction: `1.0`
  - route decision instability: `0.0`
- `angular_hopf_guarded`:
  - route reuse fraction: `0.36380952380952375`
  - fallback step fraction: `0.5657142857142856`
  - effective resolved fraction: `1.0`
  - route decision instability: `0.0`

### Paired larger replay slice

- `base_gap`:
  - route reuse fraction: `0.0`
  - fallback step fraction: `0.0`
  - effective resolved fraction: `1.0`
  - route decision instability: `0.0`
- `angular_hopf_guarded`:
  - route reuse fraction: `0.3638095238095238`
  - fallback step fraction: `0.5657142857142856`
  - effective resolved fraction: `1.0`
  - route decision instability: `0.0`

So the currently validated real-signal win is:

- lower fallback burden at the same full effective resolution

## What Is Not Yet Established

Natural reusable route classes for `base_gap` do **not** emerge on the current
replay-based benchmark boundary.

That non-result is now stable across:

- the original single replay
- the tiny reuse-recovery comparison
- the paired larger replay slice

The current bounded reading is that this is not mainly a `next_return_gap`
granularity failure:

- gap bucketing changes nothing

And the first tiny base-phase coarsening is not good enough:

- it recovers reuse only by giving back the fallback advantage

So the present replay family still appears too short and too phase-specific to
judge natural real-signal reuse fairly.

## Practical Conclusion

`base_gap` is still the correct first guarded real-signal policy.

But the real-signal result that is actually validated right now is narrow and
practical:

- the policy wins on fallback efficiency
- the policy remains stable
- the policy preserves full effective resolution

What is **not** yet validated is:

- reusable real-signal route classes on the current replay boundary

So the correct practical reading is:

- keep `base_gap` as the guarded real-signal policy
- treat fallback efficiency, not route reuse, as the currently established win

## Recommendation

Yes: stop real-signal micro-refinement on the current replay family.

The correct next step is:

- **A. bank the result and transfer the winning routing discipline into broader benchmark work**

with one guard:

- only continue direct real-signal reuse work if **B. a materially richer real-signal slice is available**

The current replay family is already sufficient to establish the bounded
fallback-efficiency result, and it is not rich enough to justify more tiny
coarsening work.
