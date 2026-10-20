# Analysis — `uor-addr`

> Empirical analysis of the JCS+NFC canonical form and the κ-derivation
> over arbitrary JSON inputs. The CP class in
> [CONFORMANCE.md](CONFORMANCE.md#cp--probabilistic-class--empirical-scaling)
> is the runtime expression of the analysis here; this document
> derives the sample sizes and significance thresholds.

## 0. Scope

This document asks one operational question:

> Does any structural choice in `uor-addr`'s κ-derivation — the JCS
> canonicalisation, the NFC normalisation, the algebraic-closure
> encoding, or the choice of `Sha256Hasher` as the substitution-axis
> hash — leak non-uniform-random structure into the κ-label?

**Short answer**: No, at the precision the CP test class establishes
(α = 0.001 across N up to 1 000 000 samples). The κ-label is
indistinguishable from a uniform-random 64-hex sample drawn under a
fixed `"sha256:"` prefix, conditional on SHA-256 satisfying its
standard pseudorandom-oracle assumption.

The substantive arguments below justify each CP test's choice of
sample size, distribution under H₀, and significance level.

## 1. Digest-byte uniformity — CP-U01

**Claim.** Under uniform random JSON-leaf inputs, each of the 32 bytes
of the SHA-256 digest is distributed uniformly over `[0, 256)`.

**H₀**: Each byte position is multinomial-uniform over 256 cells.
**H₁**: Any byte position deviates from uniform.

**Sample size.** N = 1 000 000 inputs. Expected count per cell at
byte position 0 is N/256 ≈ 3906. The χ² statistic under H₀ is
χ²(255) with mean 255 and variance 510. The 99.9th-percentile critical
value is ≈ 339.7. We accept the test if χ² < 339.7 on byte 0; we
report the test by-position for visibility.

**Significance.** α = 0.001 ⟹ a false-positive rate of 1/1000 per
run; under repeated CI execution at this level we expect ≈ 1 spurious
failure per 1000 runs, which is acceptable for a hash-uniformity
sanity check.

**Why byte 0 only.** Per-byte tests at all 32 positions are correlated
under H₀ (the digest is computed from one canonical-form input); a
joint test would not multiplicatively tighten α. Byte 0 is the
load-bearing position because it is the lexicographic head of the
hex-encoded suffix.

## 2. Hex-position uniformity — CP-U02

**Claim.** Across the 64 hex positions in the κ-label, each of the
16 possible characters appears with frequency `N/16` per position.

**H₀**: Each hex position is uniform over `{'0'..'9', 'a'..'f'}`.
**H₁**: Any position is non-uniform.

**Sample size.** N = 100 000. Expected count per cell per position
is N/16 = 6 250. χ²(15) 99.9th-percentile critical value is ≈ 37.7.

**Why fewer samples than CP-U01.** Hex characters are 4-bit cells of
the digest; uniformity over hex is implied by uniformity over digest
bytes. We run CP-U02 as a structural cross-check on `hex_lower` (the
encoder), not as an independent test of the hash function.

## 3. Collision absence at scale — CP-C01

**Claim.** Across N = 1 000 000 distinct synthetic JSON inputs, the
emitted κ-labels are pairwise distinct.

**H₀**: Pairwise distinct (κ-labels are injective on the input set).
**H₁**: At least one collision.

**Sample size.** N = 1 000 000. The birthday bound on a 256-bit hash
puts the expected first collision at √(2^256) ≈ 2^128 samples; the
probability of any collision in 10⁶ samples is

  P_collision ≤ (N choose 2) · 2^{-256}
              ≈ N²/2 · 2^{-256}
              ≈ 2^{40-1} · 2^{-256}
              = 2^{-217}.

Observing one collision at this scale would falsify SHA-256's standard
assumption. The test accepts if no collisions are observed.

**Why not run N = 10⁷.** The CP-C01 budget is bounded by the analysis
suite's 60-second release-mode runtime ceiling. Raising N tightens the
falsification window logarithmically, not asymptotically: the test
already establishes `P_collision ≤ 2^{-217}`.

## 4. Avalanche distribution — CP-A01

**Claim.** Mutating one byte of the canonical form changes ≥ 100 of
the 256 digest bits in ≥ 99% of trials.

**Why 100 bits.** Under a pseudorandom oracle, each output bit flips
independently with probability ½ on any input change. The Hamming
distance is then Binomial(256, ½) with mean 128 and standard deviation
8. P(Hamming distance < 100) = Φ((100 − 128)/8) ≈ Φ(−3.5) ≈ 2.3·10⁻⁴.
So in 10⁴ trials we expect ≈ 2.3 trials with distance < 100, well
below the 1% threshold. The test accepts if the fraction of
sub-100-bit trials is ≤ 1%.

**Sample size.** N = 10 000 trials. The expected number of
< 100-bit-distance trials under H₀ is ≈ 2.3 ± 1.5; observing > 100
(>1% threshold) is a 60-σ deviation under H₀, falsifying the
pseudorandom-oracle assumption.

## 5. NFC idempotence at scale — CP-N01

**Claim.** For arbitrary Unicode strings `s`, `nfc(nfc(s)) = nfc(s)`.

**H₀**: NFC is idempotent (a property of the Unicode normalisation
specification — UAX #15 §1.1).

**Empirical role.** This test is a *crate-level cross-check* on the
in-crate [`crate::canonical::nfc`] normalizer (UCD 15.1.0): if the
implementation ever regresses to a non-idempotent NFC, the test
catches it before release. It is not a statistical test — failure is
exact. Companion harness `tests/nfc_uax15_normalization_test.rs`
walks all 19,074 vectors × 5 NFC identities from UCD
`NormalizationTest.txt`.

**Sample size.** N = 100 000 randomly-generated Unicode strings
(stratified across BMP, supplementary planes, and combining-character
sequences). The Lean theorem `UorAddr.NfcIdempotence.nfc_is_idempotent`
(CL-N01) axiomatises this property; the empirical test pins the
*implementation* to the spec.

## 6. JCS+NFC fixed-point — CP-K01

**Claim.** For canonical-form input `b`, `canonicalize(b) = b`.

**Why this matters.** `canonicalize` is the public surface of the
in-`ψ_9`-resolver canonicalizer — the same code path the
κ-derivation runs internally. If the function is not idempotent on
its own output, two semantically-equal inputs that differ only in
already-canonical features could yield different `JsonValue`
canonical forms and therefore different κ-labels. Idempotence of
the output is what makes "the canonical form" canonical.

**Sample size.** N = 100 000 — synthetic JSON values constructed from
JCS-canonical primitives only. The test runs `canonicalize` twice
and compares; failure is exact.

## 7. Deep key-permutation invariance — CP-K02

**Claim.** Permuting object keys at any depth ≤ 4 leaves the κ-label
unchanged.

**Sample size.** N = 10 000 randomly-generated JSON objects at depth
4, with random per-object key permutations applied at each depth.
Failure is exact.

## 7.5. Typed-input case distinction — CT-T

**Claim.** The κ-derivation distinguishes the six JSON cases
structurally: `42` (a number) and `"42"` (a string of the same
digits) produce distinct κ-labels with the SHA-256 sensitivity
bound; `null`, `false`, and `true` are pairwise distinct under the
same bound; arrays and objects with same-shape payloads but
different container types are distinguished by their structural tag
byte.

**Why this matters.** The typed `JsonValue` input shape stamps a
1-byte case tag into the structurally-tagged serialisation the
ψ-pipeline carries. Two inputs whose textual rendering looks alike
but whose typed cases differ end up with distinct tagged bytes (the
tag byte differs by construction); their canonical-form bytes also
differ (RFC 8259 distinguishes the cases at the syntactic level —
`42` vs `"42"` differ in canonical form); SHA-256's sensitivity drives
them to distinct κ-labels. CT-T01..CT-T05 pin five concrete sample
pairs; the universal claim that any two semantically-distinct typed
inputs yield distinct κ-labels follows from canonical-form
distinctness (CT-E01..CT-E04 prove the converse:
canonical-form-equivalent inputs yield identical κ-labels) plus the
SHA-256 sensitivity bound from §4. The Lean theorem
`UorAddr.TypedInput.case_tags_are_pairwise_distinct` mechanises the
tag-byte half.

## 7.6. Typed-input bound enforcement — CT-B

Under ADR-060 inputs are **unbounded**: there is no input-size ceiling
and no per-ψ-stage byte-width cap, so the per-realization byte / count
caps (the former `MAX_STRING_BYTES`, `MAX_NUMBER_DIGITS`,
`MAX_OBJECT_KEYS`, `MAX_ARRAY_ELEMENTS`, `JSON_VALUE_MAX_BYTES`) are
gone. The single remaining typed-input bound is the recursive parser's
**native stack-safety depth guard** `MAX_JSON_DEPTH = 1024`, declared in
`crate::json::shapes::bounds` and enforced *exactly* at
`JsonValue::parse`. It is a stack-overflow guard, not a capacity cap.
There is no statistical claim; failure is total. The Lean theorem
`UorAddr.TypedInput.depth_bound_is_strict` mechanises the depth half
(`maxJsonDepth = 1024`).

- CT-B01 — any input of depth > `MAX_JSON_DEPTH` is rejected at parse
  with `InvalidJson`.
- CT-B02 — an over-wide string (any width) is **admitted** and yields a
  valid κ-label: there is no string-width cap.
- CT-B03 — depth = `MAX_JSON_DEPTH` is accepted (the guard is `≤`).
- CT-B04 — invalid JSON syntax is rejected with the `validUtf8Json`
  violation IRI, distinct from the depth guard.

The depth guard exists purely for native stack safety; otherwise inputs
of any size enter the ψ-pipeline (as a `Borrowed` carrier over the
canonical form) and produce a κ-label. The §8 precision claims condition
on syntactic well-formedness, not size.

## 8. The "arbitrary precision" framing

A frequent question for content-addressing implementations: *to what
precision is the implementation correct?* This crate's answer:

- **Universal precision** (no upper bound) for properties pinned by
  Lean theorems (CL-W01..CL-K02, CL-A01, CL-A02, CL-N01, CL-V01) —
  these hold for every input in the typed domain, mechanically
  checked.
- **Cryptographic precision** for sensitivity / collision absence:
  ≤ `2^{-128}` collision probability across any feasible input set,
  conditional on SHA-256.
- **Statistical precision** for distributional uniformity:
  α = 0.001 over N = 10⁶ samples; raising N moves α toward `2^{-128}`
  asymptotically. The CP test consts in `tests/analysis.rs` are the
  dial.

The composition is: **for any caller-fixed precision target, this
crate is verified up to or beyond that target.** Lean handles the
"infinite" precision target by quantifying over the entire input
domain; CP handles the "finite, statistically calibrated" target by
sampling at the chosen N/α; CD handles the "exact byte-identity"
target by reference fixtures.

## 9. PRNG determinism

All CP tests use a deterministic PRNG seeded from a const literal
(`UOR_ADDR_ANALYSIS_SEED = 0xUOR_ADDR_1`). Failures are reproducible
by re-running the same `cargo test` command in the same environment.

## 10. What this analysis does *not* establish

- It does **not** establish SHA-256's pseudorandom-oracle assumption.
  That is taken as given; CP tests would falsify the assumption if
  the algorithm broke, but their passing does not "prove" it true.
- It does **not** establish the absence of side-channel structure
  exploitable in adversarial settings — the algebra here is
  observable structural, not adversarial.
- It does **not** establish performance bounds. Throughput and
  latency are measured by `just bench` (criterion) and are out of
  the V&V scope.

## GGUF / ONNX canonical-form design notes

**Flat Merkle skeleton (ADR-060).** Both container realizations
canonicalize to a full flat skeleton in which every variable-length leaf
(tensor weights, token-vocabulary arrays, strings) is replaced by its
streamed SHA-256 digest. The skeleton's size grows only with the
KV / tensor / node counts, never with model size; the full skeleton
flows through the ψ-pipeline as a `Borrowed` carrier that ψ₉ folds —
there is no two-level section commitment and no size cap. This is the
streaming realization of content-addressing: logically-equivalent inputs
produce identical κ-labels, and any weight change flips the label.

**GGUF.** Tensor offsets are *recomputed* in sorted-name order (not
preserved) so two files whose tensor-data sections are laid out
differently canonicalize identically; `general.alignment` is read before
sorting; metadata KVs and tensor info sort lexicographically on raw UTF-8
bytes with no Unicode normalization (consistent with the GGUF spec and
gguf-py).

**ONNX.** Node order is canonicalized by Kahn's algorithm with a total
`(name, op_type, domain)` tie-break, so any valid topological input
ordering collapses to one canonical order; protobuf field-order freedom
is removed by schema-aware field selection; typed-data tensor fields are
reduced to the canonical `raw_data` little-endian layout so the two
storage forms canonicalize identically.
