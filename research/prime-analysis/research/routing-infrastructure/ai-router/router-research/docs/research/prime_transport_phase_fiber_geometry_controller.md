# Prime Transport Phase-Fiber Geometry Controller

## Chosen Richer Math-Native Signal

The chosen richer controller-side signal is:

- `phase_fiber_visibility_basin = (r_band, b_sector, phi_bucket, gap_band)`

where:

- `r_band` is a coarse wheel-depth band
- `b_sector` is `b mod 5`
- `phi_bucket` is a bounded parity bucket of the leading fiber coordinate
- `gap_band` is the immediate versus delayed return-gap band

This is architecture-faithful because it is derived from the existing exact
chart plus the already-validated return-memory refinement:

- static chart contribution: `(b, phi, r)`
- minimal residual contribution: `next_return_gap`

It is richer than the earlier visibility-zone tag because it adds a bounded
phase-fiber basin location rather than only scale plus return-depth behavior.

It is still much smaller than:

- full spin
- full branch identity

## Why This Is The Best Next Candidate

The first math-native signal:

- `visibility_zone = (r_band, next_return_gap_band)`

was safe but too weak. It reached parity with the coordination-frame
controller.

The next architecture-faithful step is therefore not a generic controller
change, but a richer basin-location tag that can tell the controller:

- not just how deep or delayed the current branch is
- but which coarse phase-fiber sector of the exact chart it currently sits in

That is the smallest richer signal that still respects the exact-layer
architecture.

## Aggregate Comparison

- current best coordination-frame controller:
  - action correctness: `0.8164085914085915`
  - reassignment / handoff correctness: `1.0`
  - joint coordination-loop correctness: `0.45520759282124196`
  - per-entity retrieval accuracy: `1.0`
  - shared-ledger retrieval accuracy: `1.0`
  - promoted-query fraction on reuse: `0.5047922252391286`
  - route reuse fraction: `0.8425859854431283`
  - instability: `0.0`
- prior visibility-zone controller:
  - action correctness: `0.8164085914085915`
  - reassignment / handoff correctness: `1.0`
  - joint coordination-loop correctness: `0.45520759282124196`
  - per-entity retrieval accuracy: `1.0`
  - shared-ledger retrieval accuracy: `1.0`
  - promoted-query fraction on reuse: `0.5047922252391286`
  - route reuse fraction: `0.8425859854431283`
  - instability: `0.0`
- richer phase-fiber-aware geometry-native controller:
  - action correctness: `0.606943056943057`
  - reassignment / handoff correctness: `1.0`
  - joint coordination-loop correctness: `0.2999959120268171`
  - per-entity retrieval accuracy: `1.0`
  - shared-ledger retrieval accuracy: `1.0`
  - promoted-query fraction on reuse: `0.5047922252391286`
  - route reuse fraction: `0.8425859854431283`
  - instability: `0.0`

## Reading

### Does a richer phase-fiber-aware math-native signal improve coordination quality materially?

No. It hurts coordination quality materially.

On the aggregate row, the richer phase-fiber-aware controller drops:

- action correctness from `0.8164085914085915` to `0.606943056943057`
- joint coordination-loop correctness from `0.45520759282124196` to
  `0.2999959120268171`

So this richer signal is not merely neutral or weak. In this first bounded
form, it is actively harmful.

### Is this the first clear controller gain driven by explicit routing geometry rather than generic policy structure?

No.

The first visibility-zone signal reached parity. This richer phase-fiber basin
signal is the first controller-side geometry experiment that clearly changes
behavior more strongly, but it changes it in the wrong direction.

So the honest reading is:

- explicit routing geometry can matter at the controller surface
- but this particular richer basin tag is not yet a productive controller gain

### Does it preserve the stability and retrieval quality already validated?

Yes.

All substrate-side properties remain fixed:

- per-entity retrieval accuracy remains `1.0`
- shared-ledger retrieval accuracy remains `1.0`
- promoted-query fraction on reuse remains `0.5047922252391286`
- route reuse remains `0.8425859854431283`
- route decision instability remains `0.0`

So the failure is a controller-quality failure, not a retrieval or stability
failure.

### Is the branch then strong enough to justify a larger bounded prototype that explicitly exploits routing geometry as part of the controller surface?

Yes, but not on the strength of this richer signal.

The branch remains strong enough because:

- the substrate is still coherent and exact
- the coordination-frame controller remains a positive bounded result
- the first visibility-zone controller remains a clean parity result

What this richer experiment adds is a warning:

- a stronger phase-fiber-aware signal can easily oversteer the controller if it
  is injected too directly

## Conclusion

The tested richer phase-fiber-aware geometry signal,
`phase_fiber_visibility_basin = (r_band, b_sector, phi_bucket, gap_band)`,
does **not** improve coordination quality. It degrades it materially while
preserving exact retrieval and zero instability.

That means:

- the routing geometry is not irrelevant
- but richer controller-side phase-fiber exposure is not automatically helpful
- the next math-native controller step should be more selective than this broad
  basin-tag override

So this result should be treated as a rejected math-native controller variant,
not as a reason to abandon the broader geometry-native direction.
