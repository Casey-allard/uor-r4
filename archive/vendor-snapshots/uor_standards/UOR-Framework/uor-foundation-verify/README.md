# uor-foundation-verify

Trace-replay verifier for the UOR Foundation.

This crate consumes a `Trace` produced by
`uor_foundation::enforcement::Derivation::replay()` and re-derives a
`Certified<GroundingCertificate>` by folding the trace events through the
ring-operation algebra — **without** invoking the decision procedures the
original pipeline ran. The replay is a deterministic pure function of the
input trace: the same trace produces a bit-identical certificate.

## Use

```rust,no_run
use uor_foundation::enforcement::Derivation;
use uor_foundation_verify::verify_trace;

# fn example(derivation: &Derivation) {
let trace = derivation.replay();
let certified = verify_trace(&trace).expect("trace verifies");
let witt_bits = certified.certificate().witt_bits();
# let _ = witt_bits;
# }
```

## Public API

The crate's public surface is intentionally small:

- `verify_trace(&Trace) -> Result<Certified<GroundingCertificate>, ReplayError>`
- `ReplayError` — enum of the four rejection modes (empty trace, out-of-order,
  zero target, length mismatch)
- Re-exports of `Certified`, `GroundingCertificate`, `Trace`, `TraceEvent`,
  and `ContentAddress` from the foundation, so downstream can import them
  under short paths.

The foundation owns the certificate-construction boundary; this crate is a
thin façade over `uor_foundation::enforcement::replay::certify_from_trace`.

## Feature flags

- `default = []` — strictly `#![no_std]`. No `alloc`, no `std`.
- `serde` — enables optional serde integration on the underlying
  foundation types (implies `alloc`).

## Stability

Ships alongside `uor-foundation` on crates.io under matching version
numbers. The two crates must be used in lockstep.

## License

MIT
