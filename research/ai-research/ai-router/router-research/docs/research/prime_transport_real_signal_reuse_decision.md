# Prime Transport Real-Signal Reuse Decision

## Decision

Yes: `base_gap` is still the correct first real-signal policy.

The current bounded replay result should be frozen as follows:

- zero reuse on the tiny replay set is mainly a short-trace / exact-base-phase
  artifact
- bucketing `next_return_gap` does not solve it
- the first small base-phase coarsening recovers reuse only by losing the
  fallback advantage
- therefore the original `base_gap` policy should remain unchanged for now

## Tiny-Coarsening Verdict

No: further tiny coarsening work is not justified before testing a larger
real-signal slice.

The guarded evidence already shows:

- gap-only coarsening does nothing
- the first base-phase coarsening recovers reuse only by pushing fallback up to
  `0.6057`

That is enough to reject another round of tiny local coarsening on the current
replay set.

## Next Real-Signal Slice

The exact next slice should be:

- `MUDBench/tmp/timing_mode_eval_v1/off_baseline.json.replay.json`
- plus `MUDBench/tmp/timing_mode_eval_v1/equal-cadence_baseline.json.replay.json`

Why this slice:

- it is already available on disk
- it stays fully outside the live seam
- it keeps the same five canonical tiny scenarios
- it doubles the current replay-based step budget from `32` to `64`
- it introduces repeated scenario structure across two timing conditions,
  which is the smallest already-available setting where natural route reuse can
  be tested more fairly without artificial key coarsening

## Frozen Conclusion

The current conservative decision is:

- keep `base_gap` unchanged
- stop tiny coarsening work for now
- move next to the paired `timing_mode_eval_v1` replay slice as the smallest
  larger guarded real-signal reuse test
