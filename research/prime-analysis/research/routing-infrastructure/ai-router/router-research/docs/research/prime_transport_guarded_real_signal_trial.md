# Prime Transport Guarded Real-Signal Trial

## Purpose

This note records the first guarded research-side integration trial in which
the prime-transport routing policy interacts with real MUDBench benchmark
signal rather than only with the earlier synthetic/exact trace family.

The trial remained fully research-only:

- no live MUDBench seam changes
- no runtime router replacement
- no new benchmark family

## Signal Source

The bounded real-signal source was the existing replay drilldown artifact:

- `MUDBench/tmp/timing_mode_eval_v1/off_baseline.json.replay.json`

This provides five canonical tiny-suite runs with per-step `state_snapshot`
payloads. Those snapshots were used to derive a small benchmark-side signal
trace from existing metrics only:

- action count
- coverage
- objective progress
- quest/social completion
- combat deltas
- survival
- tracker total signals

The resulting guarded comparison used:

1. `angular_hopf`
2. `base_gap`
3. `angular_hopf_guarded` as the resolution-maintaining diagnostic

## Aggregate Result

### `angular_hopf`

- unique route keys: `4.0`
- unique emitted route keys: `4.0`
- route reuse fraction: `0.36380952380952375`
- fallback step fraction: `0.0`
- effective resolved fraction: `0.8033333333333333`
- mean unresolved among nonpromoted routes: `0.19666666666666668`
- route decision instability: `0.0`

### `angular_hopf_guarded`

- unique route keys: `4.0`
- unique emitted route keys: `5.6`
- route reuse fraction: `0.36380952380952375`
- promotion route fraction: `0.39`
- fallback step fraction: `0.5657142857142856`
- effective resolved fraction: `1.0`
- mean unresolved among nonpromoted routes: `0.0`
- route decision instability: `0.0`

### `base_gap`

- unique route keys: `6.4`
- unique emitted route keys: `6.4`
- route reuse fraction: `0.0`
- fallback step fraction: `0.0`
- effective resolved fraction: `1.0`
- mean unresolved among nonpromoted routes: `0.0`
- route decision instability: `0.0`

## Reading

### Does prime transport retain lower fallback burden on real benchmark signal?

Yes.

On this bounded replay-based trial, `base_gap` keeps the strongest fallback
result:

- average fallback step fraction: `0.0`

So the fallback advantage does survive contact with real benchmark-side signal.

### Does angular Hopf still require near-total promotion to maintain resolution?

Not globally on this tiny real-signal set, but it still requires materially
more promotion than prime transport:

- average fallback step fraction: `0.5657`

And on at least one scenario (`tiny-hidden-key`) it does collapse to total
promotion:

- fallback step fraction: `1.0`

So the exact synthetic finding softens, but does not reverse.

### Is the prime-transport advantage robust or does it collapse?

It survives, but in a narrower form.

What survives robustly:

- zero instability
- full effective resolution
- lower fallback burden than guarded angular

What does **not** carry over cleanly yet:

- route reuse

On this tiny replay-based trial, `base_gap` is currently too fine-grained:

- route reuse fraction is `0.0`

So the real-signal result does **not** support a broad claim that prime
transport already wins every practical routing dimension. It supports a more
careful claim:

- the fallback/resolution advantage is robust
- the reuse advantage is not yet demonstrated on this small real-signal trial

## Conclusion

The prime-transport policy does **not** collapse outside the synthetic trace
family.

It remains strong on the guarded predictive criterion that motivated it:

- maintain full effective resolution
- keep fallback burden low
- stay stable

But the current real-signal wrapper exposes one important remaining weakness:

- `base_gap` has not yet shown reusable route classes on these short benchmark
  traces

So the correct next guarded research-side step is:

- keep prime transport as the primary policy under test
- keep angular Hopf as the baseline
- focus the next bounded real-signal work on route reuse / route-class
  coarsening under benchmark signal, not on fallback correctness
