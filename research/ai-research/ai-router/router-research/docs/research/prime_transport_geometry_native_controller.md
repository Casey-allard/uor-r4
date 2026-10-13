# Prime Transport Geometry-Native Controller

## Chosen Math-Native Signal

The single tested geometry-native controller signal is:

- `visibility_zone = (r_band, next_return_gap_band)`

derived from the existing route class:

- `r`
- `next_return_gap`

This is the smallest first controller-side math signal that directly exploits
the established exact routing geometry because:

- `r` is the scale / wheel-depth coordinate from the exact phase-fiber-scale
  chart
- `next_return_gap` is the currently best validated small predictive residual
- the exact-layer notes already identify delayed visibility as the mechanism by
  which deeper branch structure stays unresolved longer

So this signal is:

- bounded
- interpretable
- smaller than full spin or full branch identity
- directly derived from the exact routing architecture

The concrete controller-side tag is:

- `visibility_zone:shallow|deep:immediate|delayed`

## Why This Is The Best First Math Attempt

The current controller experiments suggest that generic policy tuning has
mostly saturated:

- local ranking helps only modestly
- one-step lookahead does not help
- explicit conflict arbitration does not help
- coordination frame helps slightly but still modestly

The most architecture-native missing signal is therefore not “more logic” in
general, but whether the controller should behave differently in:

- shallower versus deeper refinement zones
- immediate-return versus delayed-return zones

That is exactly what the chosen visibility-zone tag provides.

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
- geometry-native coordination controller:
  - action correctness: `0.8164085914085915`
  - reassignment / handoff correctness: `1.0`
  - joint coordination-loop correctness: `0.45520759282124196`
  - per-entity retrieval accuracy: `1.0`
  - shared-ledger retrieval accuracy: `1.0`
  - promoted-query fraction on reuse: `0.5047922252391286`
  - route reuse fraction: `0.8425859854431283`
  - instability: `0.0`

## Reading

### Does explicit use of the routing geometry improve coordination quality materially?

No.

On this bounded comparison, the tested geometry-native controller exactly
matches the current coordination-frame controller on the aggregate row:

- action correctness stays `0.8164085914085915`
- reassignment / handoff correctness stays `1.0`
- joint coordination-loop correctness stays `0.45520759282124196`

So this first explicit use of routing geometry does **not** improve
coordination quality materially beyond the current best controller.

### Is this the first controller gain that is genuinely architecture-math-driven rather than generic policy tuning?

No, not as a gain.

It is the first clean test of an explicitly architecture-math-driven controller
signal on top of the current stack, but it does not add measurable gain beyond
the coordination-frame baseline.

So the honest reading is:

- this signal class is architecture-math-driven
- but this first instance is only parity, not a new gain

### Does it preserve the stability and retrieval quality already validated?

Yes.

All substrate-side properties remain fixed:

- per-entity retrieval accuracy remains `1.0`
- shared-ledger retrieval accuracy remains `1.0`
- promoted-query fraction on reuse remains `0.5047922252391286`
- route reuse remains `0.8425859854431283`
- route decision instability remains `0.0`

### Is the branch then strong enough to justify a larger bounded router-native systems prototype that explicitly exploits the routing math?

Yes, conservatively.

The branch is strong enough because:

- the substrate remains coherent and exact
- the higher-level coordination-frame controller remains a real positive result
- this first geometry-native signal preserves that best result without regressions

But the current evidence does **not** yet show that routing geometry is buying
additional coordination quality beyond what the generic coordination frame
already captured.

## Conclusion

The tested geometry-native controller signal, `visibility_zone = (r_band,
next_return_gap_band)`, does **not** improve harder coordination quality
materially beyond the current best coordination-frame controller.

It is still a useful experiment because it establishes that:

- explicit routing-geometry use can be added at the controller surface without
  harming retrieval, stability, or the current best coordination result
- this first small math-native signal is too weak to create a new gain by
  itself

So the branch can justify a larger bounded systems prototype that explicitly
exploits routing math, but the next math-native step should likely use a richer
or differently structured geometry signal than this first delayed-visibility
zone tag.
