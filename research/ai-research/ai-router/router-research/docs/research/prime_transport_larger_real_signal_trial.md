# Prime Transport Larger Real-Signal Trial

## Purpose

This note records the next guarded real-signal benchmark step after the first
single-replay MUDBench trial and the tiny reuse-recovery check.

The question was narrow:

- does natural route reuse emerge for the unchanged `base_gap` policy once the
  replay slice is enlarged in the smallest already-selected guarded way?
- does the prime-transport fallback advantage survive on that paired replay
  slice?

The policy and boundary were unchanged:

- stored state: `R_min = (b, phi, r, next_return_gap)`
- routing key: `base_gap = (b, r, next_return_gap)`
- fallback: `R_full = (b, spin_H)`
- baseline: `angular_hopf_guarded`

## Real-Signal Slice

The trial used exactly the paired replay artifacts fixed in the decision note:

- `MUDBench/tmp/timing_mode_eval_v1/off_baseline.json.replay.json`
- `MUDBench/tmp/timing_mode_eval_v1/equal-cadence_baseline.json.replay.json`

This keeps the canonical scenario family unchanged while doubling the replay
budget from the earlier single-replay trial.

## Aggregate Result

### `angular_hopf_guarded`

- trace length: `64`
- unique route keys: `4.0`
- unique emitted route keys: `5.6`
- route reuse fraction: `0.3638095238095238`
- promotion route fraction: `0.39`
- promotion step fraction: `0.5657142857142856`
- fallback step fraction: `0.5657142857142856`
- effective resolved fraction: `1.0`
- mean unresolved among nonpromoted routes: `0.0`
- route decision instability: `0.0`

### `base_gap`

- trace length: `64`
- unique route keys: `6.4`
- unique emitted route keys: `6.4`
- route reuse fraction: `0.0`
- promotion route fraction: `0.0`
- promotion step fraction: `0.0`
- fallback step fraction: `0.0`
- effective resolved fraction: `1.0`
- mean unresolved among nonpromoted routes: `0.0`
- route decision instability: `0.0`

## Reading

### Does natural route reuse emerge for `base_gap` on the larger slice?

No.

The paired replay slice doubles the number of replay steps, but it still does
not produce any natural reuse for the unchanged `base_gap` policy:

- route reuse fraction remains `0.0`

So the larger paired slice does not overturn the earlier diagnosis. The real
issue is still the short-trace / exact-base-phase structure of this benchmark
slice rather than a missing one-step coarsening trick.

### Does `base_gap` retain its fallback advantage over guarded angular?

Yes.

On the paired larger slice, `base_gap` still keeps:

- fallback step fraction: `0.0`
- effective resolved fraction: `1.0`

while `angular_hopf_guarded` still needs materially more fallback:

- fallback step fraction: `0.5657142857142856`

So the guarded prime-transport advantage on fallback burden survives this
larger replay check.

### Does the policy remain the right first real-signal routing candidate without further coarsening?

Yes.

The paired larger slice strengthens the conservative practical reading:

- `base_gap` still gives full effective resolution
- `base_gap` still keeps zero instability
- `base_gap` still preserves the lower fallback burden
- the slice still does not justify a new coarsening move

What it does **not** show is natural reusable route classes on real signal. So
the correct reading is not that reuse has been solved; it is that the existing
policy remains the right guarded first real-signal policy because the
fallback-resolution tradeoff still dominates the bounded comparison.

## Conclusion

The larger paired replay slice does **not** produce natural route reuse for the
unchanged `base_gap` policy.

But it also does **not** weaken the current policy decision. The prime-transport
policy remains the correct first guarded real-signal candidate because it still
achieves:

- full effective resolution
- zero instability
- lower fallback burden than guarded angular

So the next larger guarded real-signal step, if taken later, should look for a
longer or more naturally repetitive replay slice rather than for another tiny
coarsening redesign inside the current short paired slice.
