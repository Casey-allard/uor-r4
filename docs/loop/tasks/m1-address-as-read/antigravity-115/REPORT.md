# REPORT: m1-address-as-read, antigravity-115

Lab: antigravity. Order: [ORDER.md](ORDER.md). Item: [m1-address-as-read](../../../../roadmap/items/m1-address-as-read.md).

## What did not work

- **Direct port of C++ `manifold.cpp` hashing and entropy helpers:** The reference C++ program used 64-bit hardware multiplications (`0x9E3779B97F4A7C15ULL`, `0xBF58476D1CE4E5B9ULL`), `float` salience weights, and `std::log2` byte-entropy calculations. Those violate `CHARTER.md` §3.2 (D23 no transcendental/squashing functions) and D11 (zero runtime floats and zero multiply instructions on the integer serving path). Every hash, PRNG expansion, salience weight, hierarchical parent-scope blend, and byte-diversity metric had to be redesigned using pure ARX (add-rotate-XOR) rounds, shift-and-add dyadic arithmetic, and integer bit-overlap margins above $D/2$.
- **Single-bit word rotation in `manifold.cpp`'s `rotl_hv`:** The original C++ `rotl_hv` shifted `u64` words by `s % 64` without carrying bits across word boundaries, creating 64 independent 64-bit rings instead of a single $D$-bit cyclic group. In `uor-r4-core` and `uor-r4-object-store`, we use full cross-word cyclic bit rotation (`Hypervector::rotate_left` and `Sketch4096::rotate_left`) with coprime `PRIME_STRIDES`.

## What changed

Size: `tests 3 file(s) +565/-0, code 7 file(s) +1973/-3, docs 4 file(s) +91/-1, generated 0 file(s) +0/-0, other 0 file(s) +0/-0`.

- [`crates/uor-r4-core/src/native_geometric/vsa/accumulator.rs`](../../../../crates/uor-r4-core/src/native_geometric/vsa/accumulator.rs): Added `BipolarAccumulator<const WORDS: usize>` (`BipolarAccumulator2048`, `BipolarAccumulator4096`, `BipolarAccumulator8192`) with signed `i16` bit counters, weighted bundling (`accumulate`), exact $O(1)$ algebraic unbundling (`accumulate(..., -weight)`), multiplier-free dyadic decay (`decay_dyadic`), and top-$k$ seed sparsification (`from_sparsified_seeds`).
- [`crates/uor-r4-core/src/native_geometric/vsa/hypervector.rs`](../../../../crates/uor-r4-core/src/native_geometric/vsa/hypervector.rs): Added `WORDS_8192 = 128`, `Hypervector8192`, `excess_overlap_bits(&self, other: &Self) -> i32`, and `sector_id_u8(&self) -> u8`.
- [`crates/uor-r4-core/src/native_geometric/vsa/context_engine.rs`](../../../../crates/uor-r4-core/src/native_geometric/vsa/context_engine.rs): Added `PRIME_STRIDES`, `encode_prime_stride_ngram`, and `RollingBipolarContext<const WORDS: usize, const MAX_WINDOW: usize>` for $O(1)$ sliding-window context maintenance with exact eviction subtraction.
- [`crates/uor-r4-object-store/src/manifold_sketch.rs`](../../../../crates/uor-r4-object-store/src/manifold_sketch.rs): Added `Sketch4096`, `SketchAccumulator`, `HaloHeader`, `EpistemicState`, `encode_span_sketch`, `blend_hierarchical_overlap`, and `ScopedAddressDirectory` with two-stage hybrid exact-sieve + sketch candidate fallback (`read_hybrid_sieve`, $+3\sigma$ threshold `>= 2144` bits) and `Reject` epistemic hazard gating.
- [`docs/mechanisms/vsa.md`](../../../mechanisms/vsa.md) and [`docs/mechanisms/exact-memory.md`](../../../mechanisms/exact-memory.md): Updated `## Code` sections to index `accumulator.rs` and `manifold_sketch.rs`.

## Measurements

Finite deterministic host controls (no GPU claim):
- **Exact algebraic revert (`uor-r4-core` & `uor-r4-object-store`):** Superimposing 15 hypervectors and adding + subtracting an interfering vector (`+3` then `-3`) restores the exact `i16` counter state and projects to `0` Hamming distance (`4,096 / 4,096` bits identical).
- **L-010 paired hybrid retrieval control (`tests/manifold_sketch.rs`):** On a paraphrased query whose prime factors miss the stored address (`PrimeAddress::from_primes(&[29, 31])` vs. stored `[2, 3]`), Arm A (`read_sieve` exact-sieve-only) returns `SieveMiss` (`0` hits), while Arm B (`read_hybrid_sieve` hybrid-sieve-sketch) recovers the exact verified object bytes via the $+3\sigma$ (`>= 2,144` bit overlap) sketch fallback while preserving `0` backend fetches on `Absent` and `Evicted` addresses.

## Checks run at the head

- `CARGO_TARGET_DIR=local/target ~/.cargo/bin/cargo test -p uor-r4-core --lib vsa::tests`: `19 passed; 0 failed`.
- `CARGO_TARGET_DIR=local/target ~/.cargo/bin/cargo test -p uor-r4-object-store`: `108 passed; 0 failed; 1 ignored`.
- `CARGO_TARGET_DIR=local/target ~/.cargo/bin/cargo clippy -p uor-r4-object-store --all-targets -- -D warnings`: `0 warnings, 0 errors`.
- `~/.cargo/bin/rustfmt --check` on all touched Rust files: `0 diffs`.
- `uor-harness loop pr-check --local` and `uor-harness loop gates --quick`: `PASS`.

## Skills used

- **solution-integrity:** Separated the mathematical wins in `manifold.cpp` (signed `i16` bipolar accumulation, reversible $O(1)$ unbundling, prime-stride multi-scale n-grams, hierarchical parent-scope blending, and $+3\sigma$ admission gating) from its Charter-incompatible implementation details (`float` math, `std::log2`, and 64-bit integer multipliers).
- **scope-guard:** Kept all changes additive within `uor-r4-core::native_geometric::vsa` and `uor-r4-object-store`, with zero changes to `Cargo.toml` or `Cargo.lock`.

## Still open

- Connecting `ScopedAddressDirectory::read_hybrid_sieve` to the learned address-formation evaluation on the 20 sieve misses identified in the M1 serving-time oracle sheet (`m1-address-as-read-oracle-2026-10-26.md`).

## Plain summary

Previously, bundling binary hypervectors immediately rounded each bit to `0` or `1`, so removing an outdated fact or sliding a context window required re-bundling from scratch, and exact prime-sieve memory reads failed completely whenever a question was phrased with different words than the stored fact. We imported a signed integer counter (`BipolarAccumulator` and `SketchAccumulator`) that can add and subtract facts or context tokens in one step without rounding loss, together with a two-stage memory lookup (`ScopedAddressDirectory::read_hybrid_sieve`) that checks exact prime addresses first and uses a 4,096-bit word-and-prefix sketch as a candidate fallback when phrasing differs, while still refusing retracted or unstated facts.
