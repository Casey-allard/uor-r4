# Prime Transport Rmin Offline Evaluation

## Purpose

This note records the bounded offline evaluation of the three exact-layer
routing-state candidates:

- `R_static = (b, phi, r)`
- `R_min = (b, phi, r, next_return_gap)`
- `R_full = (b, spin_H)`

It is an offline design check only. No runtime code is changed here.

## Summary Table

The evaluation summary is recorded in:

- `results/prime_transport_recursive_system/prime_transport_rmin_offline_eval_summary.csv`

The key rows are:

- `R_static`
  - predictive membership `1.0`
  - split purity `1.0`
  - label complexity ratio to spin `7.273714418110126`
  - interpretation: exact but too detailed to count as a practical compressed
    router state

- `R_min`
  - predictive membership `1.0`
  - split purity `0.69033629362971`
  - label complexity ratio to spin `0.6121906310452204`
  - combined capture fraction of spin `0.7502981320146836`
  - promotion-needed fraction `0.30966370637028995`

- `R_full`
  - predictive membership `1.0`
  - split purity `1.0`
  - label complexity ratio `1.0`
  - interpretation: reference exact predictor

## Evaluation Reading

The main conclusion is:

- `R_min` is strong enough to justify a first prototype

Why:

- it keeps full membership discrimination on the bounded exact rows
- it preserves a substantial fraction of the spin-side split structure
- it remains genuinely smaller than full spin

Where it fails relative to `R_full`:

- split purity drops from `1.0` to `0.69033629362971`
- about `31%` of the split-partition cases remain unresolved and would still
  need promotion to a richer predictive state
- this failure is consistent with the earlier exact-layer result that richer
  return-grammar structure still exists beyond `next_return_gap`

Whether the gain over `R_static` is large enough to matter:

- yes, in the only sense that matters for a prototype
- `R_static` appears perfect only because it is the exact chart address and is
  therefore not a compressed routing state
- `R_min` trades some predictive precision for a large drop in state complexity,
  from label ratio `7.273714418110126` down to `0.6121906310452204`

So the relevant gain is not better purity than `R_static`. The gain is that
`R_min` provides a usable compressed routing state while keeping a substantial
part of the predictive structure.

## Conclusion

The current offline decision is:

- `R_min = (b, phi, r, next_return_gap)` is a viable first routing prototype
  target

with the following caution:

- it is materially weaker than full spin
- it should therefore be paired with selective promotion rather than treated as
  a final exact predictor
