# ORDER: m1-address-as-read, antigravity-115

Lab: antigravity. Item: [m1-address-as-read](../../../../roadmap/items/m1-address-as-read.md). Base: `50e7fc6bd`.

## Goal

Import the reversible `i16` bipolar integer accumulator (`BipolarAccumulator`, `RollingBipolarContext`, `encode_prime_stride_ngram`) into `uor-r4-core::native_geometric::vsa` and the multiplier-free 4,096-bit hyperdimensional sketch consumer (`Sketch4096`, `SketchAccumulator`, `ScopedAddressDirectory`, `read_hybrid_sieve`, `HaloHeader`, `EpistemicState`) into `uor-r4-object-store::manifold_sketch`, closing the paraphrased sieve-miss retrieval gap on `AddressDirectory` under strict Charter §3.1–§3.3 and D11/D23/D30 integer constraints.

## Cancel probe (run first)

Result (2026-10-28T01:05Z, `origin/main` at `50e7fc6bd`):
- `crates/uor-r4-core/src/native_geometric/vsa/` had no signed integer bipolar accumulator (`BipolarAccumulator`): `Hypervector::bundle` truncated immediately to 1-bit majority per call, preventing incremental $O(1)$ subtraction on overwrite or window eviction.
- `crates/uor-r4-object-store/src/address_read.rs` had exact prime/semiprime lookup and sorted-merge GCD sieve admission (`read_sieve`), which returned `SieveMiss` whenever a paraphrased query shared zero prime factors with stored entries.
- Neither open PR (#3337, #3323) adds `accumulator.rs` or `manifold_sketch.rs`.

**Not cancelled (instrument: none needed).**

## Files in scope

- `crates/uor-r4-core/src/native_geometric/native_support.rs`
- `crates/uor-r4-core/src/native_geometric/vsa/accumulator.rs`
- `crates/uor-r4-core/src/native_geometric/vsa/context_engine.rs`
- `crates/uor-r4-core/src/native_geometric/vsa/hypervector.rs`
- `crates/uor-r4-core/src/native_geometric/vsa/mod.rs`
- `crates/uor-r4-core/src/native_geometric/vsa/tests.rs`
- `crates/uor-r4-core/tests/vsa_conformance.rs`
- `crates/uor-r4-object-store/src/lib.rs`
- `crates/uor-r4-object-store/src/manifold_sketch.rs`
- `crates/uor-r4-object-store/tests/manifold_sketch.rs`
- `docs/mechanisms/exact-memory.md`
- `docs/mechanisms/vsa.md`
- `docs/loop/tasks/m1-address-as-read/antigravity-115/**`

## Done when

1. `BipolarAccumulator` (`2048`/`4096`/`8192`) and `RollingBipolarContext` pass exact algebraic addition/subtraction revert tests (`0` Hamming distance after `+w` and `-w`) and sliding-window parity tests in `uor-r4-core`.
2. `ScopedAddressDirectory` and `read_hybrid_sieve` pass L-010 paired controls (`exact-sieve-only` vs. `hybrid-sieve-sketch`, plus `Absent`/`Evicted` zero-backend-fetch controls), incremental parent-scope overwrite retraction, and $+3\sigma$ (`>= 2144` bits) `Reject` hazard gating in `uor-r4-object-store`.
3. Zero floats, zero multiply instructions on serving paths, and zero D23/D25/D32 violations.

## Stop and decide (never stop and wait)

- Any D23, D25, or D32 ratchet or `banned-presence` increase: stop and replace with pure shift-and-add/XOR/popcount arithmetic.
- Any `Absent` or `Evicted` entry bypassed by sketch fallback: stop and enforce exact `AddressDirectory` provenance gates before sketch candidate evaluation.
