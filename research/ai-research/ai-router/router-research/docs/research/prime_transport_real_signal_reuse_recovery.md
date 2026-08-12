# Prime Transport Real-Signal Reuse Recovery

## Purpose

This note records the next guarded real-signal step after the first replay-based
prime-transport trial.

The question was narrow:

- why does `base_gap` show zero route reuse on the tiny MUDBench replay traces?
- can one minimal guarded coarsening recover reuse without losing the
  fallback/resolution advantage?

The comparison remained bounded and research-only.

## Candidates Tested

The same replay-based real-signal trace family was reused.

The compared guarded partitions were:

1. `base_gap = (b, r, next_return_gap)`
2. `base_gap_gapbucket = (b, r, bucket(next_return_gap))`
3. `base_bucket5_gapbucket = (floor(b / 5), r, bucket(next_return_gap))`
4. `angular_hopf_guarded`

The gap bucket was intentionally tiny:

- `0`
- `1`
- `2+`

No new benchmark family and no policy redesign were introduced.

## Aggregate Result

### `base_gap`

- unique route keys: `6.4`
- route reuse fraction: `0.0`
- fallback step fraction: `0.0`
- effective resolved fraction: `1.0`
- route decision instability: `0.0`

### `base_gap_gapbucket`

- unique route keys: `6.4`
- route reuse fraction: `0.0`
- fallback step fraction: `0.0`
- effective resolved fraction: `1.0`
- route decision instability: `0.0`

### `base_bucket5_gapbucket`

- unique route keys: `3.8`
- route reuse fraction: `0.4038095238095238`
- fallback step fraction: `0.6057142857142856`
- effective resolved fraction: `1.0`
- route decision instability: `0.0`

### `angular_hopf_guarded`

- unique route keys: `4.0`
- route reuse fraction: `0.36380952380952375`
- fallback step fraction: `0.5657142857142856`
- effective resolved fraction: `1.0`
- route decision instability: `0.0`

## Reading

### Is zero reuse mainly a short-trace artifact or a true partition problem?

Mainly a short-trace artifact induced by exact base phase.

The decisive observation is:

- bucketing `next_return_gap` alone changes nothing

So the uniqueness is not being driven mainly by exact gap values.

Instead, on these very short traces:

- `b` advances monotonically
- the trace ends before any meaningful base-phase reuse can occur
- the literal `base_gap` key therefore behaves like a short local address

So the zero-reuse result is real, but its main cause is the interaction between
exact base phase and very short real-signal traces.

### Can one minimal guarded coarsening recover meaningful reuse?

Yes, but not cheaply enough.

The first tiny base-phase coarsening:

- `base_bucket5_gapbucket`

does recover reuse:

- route reuse fraction rises from `0.0` to `0.4038`

But it does so by giving back the fallback advantage:

- fallback step fraction rises from `0.0` to `0.6057`

That is worse than the original prime policy and slightly worse than
`angular_hopf_guarded` on fallback burden.

So the minimal tested coarsening is not a good enough tradeoff.

### Does prime transport keep its fallback advantage after that coarsening?

No.

The fallback advantage is preserved only in the original `base_gap` real-signal
partition on this tiny replay set.

Once the tested base-phase coarsening is introduced, the prime-side fallback
advantage disappears.

## Conclusion

The current replay-based zero-reuse result should be interpreted as:

- mainly a short-trace / exact-base-phase artifact
- not mainly a `next_return_gap` granularity problem

The first minimal guarded coarsening does recover reusable route classes, but
it does not recover them at an acceptable fallback cost.

So the bounded conclusion is:

- keep the original `base_gap` real-signal policy unchanged for now
- do not replace it with the tested coarsened variant
- treat real-signal reuse as the remaining open practical problem
- and do not claim that a cheap one-step coarsening has solved it
