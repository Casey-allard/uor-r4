# Conformance Contract — `uor-addr`

> Normative conformance contract. Each invariant has a stable ID
> (e.g. `CS-V01`) referenced by tests, code comments, and PR
> descriptions. Adding, retiring, or renumbering an ID is a contract
> change. See [ARCHITECTURE.md](ARCHITECTURE.md) for the architectural
> vocabulary, [STANDARDS.md](STANDARDS.md) for the authoritative
> source references each realization conforms to, and
> [VERIFICATION.md](VERIFICATION.md) for the reproducible acceptance
> gate.

## Scope across realizations

This document originally specified the conformance contract for the
JSON realization (`uor_addr::json`). Additional shipped realizations
extend the contract under the same invariant-ID scheme:

| Realization | Imported spec | Test file |
|---|---|---|
| `uor_addr::json` | RFC 8259 + RFC 8785 JCS + UAX #15 NFC | [byte_identity.rs](crates/uor-addr/tests/byte_identity.rs), [conformance.rs](crates/uor-addr/tests/conformance.rs), [jcs_rfc8785.rs](crates/uor-addr/tests/jcs_rfc8785.rs), [typed_input.rs](crates/uor-addr/tests/typed_input.rs) |
| `uor_addr::sexp` | Rivest 1997 + RFC 2693 §3 | [sexp_conformance.rs](crates/uor-addr/tests/sexp_conformance.rs), [sexp_rivest_examples.rs](crates/uor-addr/tests/sexp_rivest_examples.rs) |
| `uor_addr::xml` | W3C XML-C14N 1.1 (subset) | [xml_c14n_1_1.rs](crates/uor-addr/tests/xml_c14n_1_1.rs) |
| `uor_addr::asn1` | ITU-T X.690 DER | [asn1_x690_der.rs](crates/uor-addr/tests/asn1_x690_der.rs) |
| `uor_addr::ring` | UOR-Framework Amendment 43 §2 | [ring_amendment_43.rs](crates/uor-addr/tests/ring_amendment_43.rs) |
| `uor_addr::codemodule` | CCMAS canonical AST | [codemodule_ccmas.rs](crates/uor-addr/tests/codemodule_ccmas.rs) |
| `uor_addr::schema::photo` + `::document` | schema.org/Photograph + schema.org/Article | [schema_org_conformance.rs](crates/uor-addr/tests/schema_org_conformance.rs) |
| `uor_addr::schema::codemodule_signed` | in-toto Statement v1 | [in_toto_statement_v1.rs](crates/uor-addr/tests/in_toto_statement_v1.rs) |
| `uor_addr::variant::storage` | ADR-048 non-default `C` | [variant_storage.rs](crates/uor-addr/tests/variant_storage.rs) |
| Common architectural surface | `AddressInput` trait + `AddressLabel` shape | [common_surface.rs](crates/uor-addr/tests/common_surface.rs) |
| Cross-realization integration | ψ-pipeline + κ-label uniformity | [all_realizations.rs](crates/uor-addr/tests/all_realizations.rs) |

The CS-V01 / CD-D01 / CL-K01 / CL-W01 / CL-R01 / CT-T\* / CT-B\*
invariant classes apply to **every** UOR-ADDR realization (the
architectural commitment is realization-neutral). Per-realization
conformance suites instantiate them over their typed-input shapes
and canonicalization byte-output discipline.

## The contract — what `uor-addr` claims (JSON realization baseline)

For every well-formed JSON byte sequence `b` (of any length — ADR-060
removed the input-size ceiling) whose nesting depth is `≤ MAX_JSON_DEPTH`:

1. **Address determinism (CD-D01).** `address(b)` produces exactly one
   ASCII byte sequence — the κ-label — and the same `b` always produces
   the same κ-label.
2. **κ-derivation identity (CL-K01).** The κ-label is byte-equal to
   `b"sha256:" ‖ hex_lower(SHA-256(jcs_nfc_canonicalise(b)))`, computed by exactly
   **one** σ-projection of the canonical hash axis inside the ψ_9
   resolver — never inside the verb body.
3. **Algebraic-closure shape (CL-A01).** The output shape
   `AddressLabel` carries 71 disjoint `Site` constraints whose Euler
   characteristic is `χ(N(C)) = 71 = SITE_COUNT`. The closure-rank
   residual is 0; the ψ-pipeline converges in `n − χ(N(C)) = 0` residual
   stages.
4. **Invariance under canonicalisation (CD-I01).** Inputs that
   differ only in (i) JSON key ordering, (ii) JSON whitespace, or
   (iii) Unicode normalisation form (NFC vs NFD vs NFKC vs NFKD)
   yield the same κ-label.
5. **Sensitivity (CD-S01).** Distinct canonical-form byte sequences
   yield distinct κ-labels with probability ≥ `1 − 2^{-128}` over any
   fixed N ≤ `2^64` collision-resistance window (the SHA-256 security
   assumption).
6. **Wire-format width (CL-W01).** The κ-label is exactly 71 bytes,
   begins with the 7-byte ASCII prefix `"sha256:"`, and continues with
   64 ASCII bytes drawn from `{'0'..'9', 'a'..'f'}`.
7. **TC-05 replay round-trip (CL-R01).** Every `Grounded<AddressLabel>`
   the pipeline emits is replayable through `prism_verify::certify_from_trace`
   into a `Certified<GroundingCertificate>` carrying the **same**
   `ContentFingerprint` (QS-05 — bit-identical round-trip), without the
   verifier re-invoking the canonical hash axis.
8. **Typed-input case distinction (CT-T*).** Different JSON cases —
   `null`, `false`, `true`, number, string, array, object — produce
   structurally-distinct `JsonValue` instances and therefore distinct
   κ-labels, even when the input texts look similar (`42` ≠ `"42"`,
   `null` ≠ `false`).
9. **Typed-input depth guard (CT-B*).** Under ADR-060 inputs are
   unbounded — the byte / count caps are gone. The single remaining
   typed-input bound is the recursive parser's native stack-safety depth
   guard `MAX_JSON_DEPTH = 1024` (`crate::json::shapes::bounds`): an
   input nested deeper than the guard is rejected at `JsonValue::parse`
   with `InvalidJson`; everything else (over-wide strings, many keys,
   long numbers) is admitted and yields a valid κ-label. The constructor
   never silently truncates.
10. **Cost-model selection (CT-C01).** The PrismModel's 5th parameter
    `C` is explicitly bound to `prism::pipeline::EmptyCommitment`
    (wiki ADR-048). The JSON realization carries no auxiliary cost
    surface; cost-model-bearing variants live in
    [`crate::variant`](crates/uor-addr/src/variant/).

## Conformance classes

Each class fixes an enforcement mechanism. Invariant IDs use the
two-letter class prefix.

### CH — Hash σ-axis class — authoritative known-answer tests

The κ-label `<algorithm>:<lowercase-hex>` is produced through a selectable
σ-axis (`uor_addr::hash::AddrHash`): sha256 (default), blake3, sha3-256,
keccak256. Each axis is validated against vectors imported from its
authoritative source, and the pipeline is shown to mint
`<prefix>:<hex(H(canonical_form))>` for the same `H`. Verified by
`crates/uor-addr/tests/hash_kat.rs`.

| ID      | Invariant                                                                     | Source / pinned by                                  |
|---------|-------------------------------------------------------------------------------|-----------------------------------------------------|
| CH-K01  | `Sha256Hasher` reproduces the FIPS 180-4 `""` / `"abc"` digests               | FIPS 180-4 — `hash_kat::sha256_matches_fips_180_4`  |
| CH-K02  | `Sha3_256Hasher` reproduces the FIPS 202 `""` / `"abc"` digests               | FIPS 202 — `hash_kat::sha3_256_matches_fips_202`    |
| CH-K03  | `Keccak256Hasher` reproduces the Keccak `""` / `"abc"` digests                | Keccak submission — `hash_kat::keccak256_matches_keccak_submission` |
| CH-K04  | `Blake3Hasher` reproduces the BLAKE3 reference `""` / `"abc"` digests          | BLAKE3 spec — `hash_kat::blake3_matches_reference_vectors` |
| CH-K05  | `Sha512Hasher` reproduces the FIPS 180-4 `""` / `"abc"` digests (64-byte)      | FIPS 180-4 §6.4 — `hash_kat::sha512_matches_fips_180_4` |
| CH-P01  | Each realization's `address_<algorithm>` emits `<prefix>:<hex(H(canonical))>` | `hash_kat::json_pipeline_mints_each_axis_over_canonical_form`, `cbor_rfc8949::pipeline_mints_each_axis_over_canonical_form` |

### CL-CBOR — CBOR realization (RFC 8949 §4.2)

The CBOR canonical form is RFC 8949 §4.2 Deterministic Encoding (shortest
integer/float heads, definite lengths, bytewise-sorted map keys, canonical
NaN). Validated against RFC 8949 Appendix A encoding vectors by
`crates/uor-addr/tests/cbor_rfc8949.rs`.

| ID        | Invariant                                                                  | Pinned by                                              |
|-----------|----------------------------------------------------------------------------|--------------------------------------------------------|
| CL-CBOR01 | `canonicalize` is the identity on RFC 8949 Appendix A canonical encodings   | `cbor_rfc8949::appendix_a_canonical_encodings_are_idempotent` |
| CL-CBOR02 | Non-shortest integer heads shorten to preferred encoding                    | `cbor_rfc8949::preferred_integer_encoding_shortens_non_minimal_heads` |
| CL-CBOR03 | Indefinite-length items fold to definite length                             | `cbor_rfc8949::indefinite_lengths_fold_to_definite`    |
| CL-CBOR04 | Map keys sort bytewise-lexicographically by encoded key                     | `cbor_rfc8949::map_keys_sort_bytewise_lexicographically` |
| CL-CBOR05 | Floats shorten to the smallest exact representation; NaN → `f97e00`         | `cbor_rfc8949::floats_shorten_to_the_smallest_exact_representation` |
| CL-CBOR06 | Malformed input (trailing bytes, reserved heads, bad UTF-8, dup keys) rejected | `cbor_rfc8949::rejects_malformed_input`             |

### CS — Structural class — shape and typed surface

Verified by **source-grep + compile-time invariants + unit tests** under
`crates/uor-addr/src/`. CI fails if any structural claim drifts.

| ID       | Invariant                                                                                 | Pinned by                                                                                       |
|----------|-------------------------------------------------------------------------------------------|-------------------------------------------------------------------------------------------------|
| CS-T01   | `AddressLabel::SITE_COUNT = 71`                                                          | `model::tests::address_label_site_count_matches_wire_format_width`                              |
| CS-T02   | `AddressLabel::CONSTRAINTS` is exactly 71 disjoint `ConstraintRef::Site` instances        | `model::tests::address_label_carries_seventy_one_disjoint_site_constraints` + `const _` in `resolvers.rs` |
| CS-T03   | `AddressLabel::CONSTRAINTS[i]` pins position `i` for `i ∈ [0, 71)`                        | `model::tests::address_label_constraints_pin_every_wire_format_site` + `const _` in `resolvers.rs` |
| CS-B01   | Two capacity profiles: `AddrBounds` (FP_MAX = 32, `NERVE_SITES_MAX = 74` — the 32-byte axes) and `AddrBounds64` (FP_MAX = 64, `NERVE_SITES_MAX = 135` — the sha512 axis) | `crates/uor-addr/src/bounds.rs` |
| CS-B02   | ADR-060: there is no input-size ceiling and no per-ψ-stage byte-width cap; the canonical form flows as a `TermValue` carrier (`Inline` / `Borrowed` / `Stream`). `ADDR_INLINE_BYTES` is the foundation-derived κ-label inline width, not an input cap | `crates/uor-addr/src/bounds.rs` (`ADDR_INLINE_BYTES`) + `tests::common_surface` |
| CS-V01   | The verb arena contains no `Term::FirstAdmit` / `Term::AxisInvocation` / `Le`/`Lt`/`Ge`/`Gt`/`Concat` | `verbs::tests::verb_arena_contains_no_sigma_residuals`                              |
| CS-V02   | The verb arena contains each of ψ_1, ψ_7, ψ_8, ψ_9                                       | `verbs::tests::verb_arena_contains_psi_{1,7,8,9}_*`                                             |
| CS-S01   | `unsafe` blocks: zero                                                                     | `#![forbid(unsafe_code)]` at lib root + `tests::conformance::no_unsafe_anywhere`                |
| CS-S02   | `unwrap()` / `expect()` in non-test code paths: zero in `src/{verbs,resolvers,pipeline}.rs` | `tests::conformance::no_panic_paths_in_pipeline`                                              |

### CX — Composition class — ADR-061 categorical operations

Verified by **behavior tests over the five categorical operations on
the Atlas image inside E₈** (`crates/uor-addr/tests/composition.rs`).
Each operation composes operand κ-labels into a new κ-label on the same
σ-axis (CA-3 σ-axis homogeneity); the test asserts the named algebraic
law (ADR-061 §(3)) holds structurally. Composition is exposed across
every binding (CX-FFI rows). The CS-F4/CS-E7 → CS-E8 digest
coincidence for "positive" operands (an operand lex-≤ its mirror is its
own CS-F4 representative, and an already-ascending quarter layout is its
own CS-E7 lex-min — both equal to CS-E8's identity) is a documented
property, not a defect: the κ-label digests `H(canonical_form)` while
the realization IRI distinguishes the typed shape.

| ID       | Invariant                                                                                 | Pinned by                                                                  |
|----------|-------------------------------------------------------------------------------------------|----------------------------------------------------------------------------|
| CX-G2    | CS-G2 is commutative through the pipeline: `g2(a,b) == g2(b,a)`                            | `tests::composition::g2_product_is_commutative_through_the_pipeline`       |
| CX-F4    | CS-F4 collapses the ± mirror pair to one composed κ-label                                  | `tests::composition::f4_quotient_collapses_the_mirror_pair`                |
| CX-E6E7  | CS-E6 / CS-E7 compose well-formed κ-labels deterministically                              | `tests::composition::e6_e7_are_well_formed_and_deterministic`              |
| CX-E8    | CS-E8 composed κ-label is distinguished from its operand                                  | `tests::composition::e8_embedding_is_distinguished_from_its_operand`       |
| CX-D01   | Operations with distinct canonical-form lengths (g2/e6/e8) yield distinct κ-labels        | `tests::composition::operations_with_distinct_canonical_forms_yield_distinct_labels` |
| CX-A01   | σ-axis homogeneity (CA-3) is enforced: a cross-axis operand is rejected                    | `tests::composition::sigma_axis_homogeneity_is_enforced`                   |
| CX-AX    | Composition holds on the blake3 and sha512 σ-axes (sha512 fingerprint = 64 bytes)          | `tests::composition::composition_axes_blake3_and_sha512`                   |
| CX-FFI-C | C ABI `uor_addr_compose_*[_with_witness]`: label parity with the in-crate path, G2 commutativity, witness TC-05 round-trip, error paths | `uor_addr_c::tests::composition::cl_c_ffi_0{1,2,3,4}__*` |
| CX-FFI-PY | Python `kappa.compose_*[_with_witness]`: well-formed labels, G2 commutativity, witness round-trip, blake3 axis | `bindings/python/tests/test_composition.py` |
| CX-FFI-W | WASM/npm `compose-*` / `composeG2…`: G2 commutativity + witness round-trip per op          | `bindings/npm/scripts/test.mjs` (`composeG2` … `composeE8`)                |
| CX-L01   | Lean: canonical-form widths (g2=2N, e6=N+1, f4/e7/e8=N), G2 commutativity, S₄ orbit = 24 distinct, E6 8:1 partition, F4 involution | `UorAddr/CompositionLaws.lean` |

### CD — Deterministic class — per-input byte identity

Verified by **runtime tests over a fixed fixture set** under
`crates/uor-addr/tests/byte_identity.rs` and
`crates/uor-addr/tests/conformance.rs`. The fixture baseline is the
12 cases harvested from `mcp.uor.foundation/tools/encode_address`
(mcp-uor-server v0.2.1, algorithm `uor-sha256-v1`).

| ID       | Invariant                                                                                 | Pinned by                                                                  |
|----------|-------------------------------------------------------------------------------------------|----------------------------------------------------------------------------|
| CD-D01   | `address(b)` is a pure function: idempotent across N repeated calls                       | `tests::conformance::address_is_pure_function`                             |
| CD-D02   | The 12 reference fixtures reproduce byte-for-byte                                         | `tests::byte_identity::shim_layer_reproduces_harvested_fixtures`           |
| CD-D03   | `canonicalize(raw)` (the in-surface canonicalizer) reproduces the reference canonical-form bytes for each fixture | `tests::byte_identity::canonicalize_kernel_matches_expected_canonical_form`|
| CD-I01a  | Key-order invariance: `{"a":1,"b":2}` ≡ `{"b":2,"a":1}` under `address`                  | `tests::byte_identity::pipeline_key_order_invariant`                       |
| CD-I01b  | Whitespace invariance: `{"foo": "bar"}` ≡ `{"foo":"bar"}` under `address`                | `tests::conformance::whitespace_invariance`                                |
| CD-I01c  | NFC invariance: composed `caf\u{E9}` ≡ decomposed `cafe\u{301}` under `address`          | `tests::byte_identity::pipeline_nfc_invariant`                             |
| CD-I01d  | NFKC equivalence: full-width digits ≡ ASCII digits under `address` (NFKC compatibility)   | `tests::conformance::nfkc_compatibility_class_holds` (informational)       |
| CD-S01a  | Single-byte mutation changes the κ-label                                                  | `tests::byte_identity::pipeline_distinct_inputs_yield_distinct_addresses`  |
| CD-S01b  | Avalanche: mutating one byte of the canonical form changes ≥ 100 of the 256 digest bits   | `tests::conformance::single_byte_avalanche_balanced`                       |
| CD-W01   | Every emitted κ-label is 71 ASCII bytes, begins `"sha256:"`, hex is lowercase             | `tests::byte_identity::pipeline_address_is_seventy_one_ascii_bytes`        |
| CD-G01   | The owned `AddressOutcome::witness` recovers the κ-label: `witness.kappa_label()` equals `outcome.address`, and `witness.verify()` re-certifies to the same label | `tests::byte_identity::pipeline_witness_recovers_kappa_label`              |

### CP — Probabilistic class — empirical scaling

Verified by **parametric large-sample runtime tests** in release mode
under `crates/uor-addr/tests/analysis.rs`. Failures are statistical;
each test names its sample size, significance level, and reference
distribution. See [ANALYSIS.md](ANALYSIS.md) for derivations.

| ID       | Invariant                                                                                                              | Pinned by                                                              | N (samples) | α       |
|----------|------------------------------------------------------------------------------------------------------------------------|------------------------------------------------------------------------|-------------|---------|
| CP-U01   | Digest byte 0 is uniform across `[0, 256)` under uniform JSON-leaf inputs                                              | `tests::analysis::digest_byte_uniformity_chi_squared`                  | 1 000 000   | 0.001   |
| CP-U02   | Digest hex character class `[0-9a-f]` is uniform across the 64 hex positions of the κ-label                            | `tests::analysis::hex_position_uniformity_chi_squared`                 | 100 000     | 0.001   |
| CP-C01   | Pairwise κ-label collisions are absent across N distinct synthetic JSON inputs (birthday bound ≪ `2^{-100}` at N=1e6) | `tests::analysis::no_collisions_at_scale`                              | 1 000 000   | n/a     |
| CP-A01   | Single-byte-mutation Hamming distance to baseline ≥ 100 bits for ≥ 99% of trials                                       | `tests::analysis::avalanche_distance_distribution`                     | 10 000      | 0.001   |
| CP-N01   | NFC round-trip stability: `nfc(nfc(s)) = nfc(s)` for arbitrary Unicode-string JSON leaf inputs                          | `tests::analysis::nfc_idempotent_at_scale`                             | 100 000     | exact   |
| CP-K01   | JCS+NFC canonical form has fixed point: `canonicalize(canonicalize(b)) = canonicalize(b)` for already-canonical inputs | `tests::analysis::cp_k01__canonicalize_idempotent_at_scale`            | 100 000     | exact   |
| CP-K02   | Permuting object keys at depth ≤ 4 leaves the κ-label unchanged                                                        | `tests::analysis::deep_key_permutation_invariance`                     | 10 000      | exact   |

### CT — Typed-input class — `JsonValue` shape claims

Verified by **runtime parser + pipeline tests** at
`crates/uor-addr/tests/typed_input.rs`. The typed `JsonValue` input
shape lets us distinguish JSON cases structurally (not just by
canonical-form serialisation), reject violators of any typed-input
bound at construction, and collapse structural-equivalence classes
to one κ-label.

| ID       | Invariant                                                                                          | Pinned by                                                              |
|----------|----------------------------------------------------------------------------------------------------|------------------------------------------------------------------------|
| CT-T01   | `42` and `"42"` produce distinct κ-labels (integer ≠ string of same digits)                       | `tests::typed_input::ct_t01__integer_distinct_from_string_of_same_digits` |
| CT-T02   | `null` and `false` produce distinct κ-labels                                                      | `tests::typed_input::ct_t02__null_distinct_from_false`                 |
| CT-T03   | `true`, `false`, `null` are pairwise distinct                                                     | `tests::typed_input::ct_t03__three_scalars_pairwise_distinct`          |
| CT-T04   | `{}` and `[]` produce distinct κ-labels                                                           | `tests::typed_input::ct_t04__empty_object_distinct_from_empty_array`   |
| CT-T05   | `[1,2,3]` (numbers) and `["1","2","3"]` (strings) produce distinct κ-labels                       | `tests::typed_input::ct_t05__number_array_distinct_from_string_array`  |
| CT-E01   | Key-order invariance (structural equivalence; restatement of CD-I01a at the typed-input layer)    | `tests::typed_input::ct_e01__key_ordering_invariance`                  |
| CT-E02   | Whitespace invariance (structural equivalence; restatement of CD-I01b)                            | `tests::typed_input::ct_e02__whitespace_invariance`                    |
| CT-E03   | NFC invariance (composed `caf\u{E9}` ≡ decomposed `cafe\u{301}`; restatement of CD-I01c)         | `tests::typed_input::ct_e03__nfc_invariance`                           |
| CT-E04   | Nested key-order invariance through depth 3                                                       | `tests::typed_input::ct_e04__nested_key_ordering_invariance`           |
| CT-B01   | Over-deep nesting (> `MAX_JSON_DEPTH`) is rejected at parse with `InvalidJson` (native stack-safety depth guard) | `tests::typed_input::ct_b01__over_deep_nesting_rejected_at_parse`      |
| CT-B02   | An over-wide string (any width) is **admitted** and yields a valid κ-label — ADR-060 removed the width cap | `tests::typed_input::ct_b02__wide_string_admitted`                    |
| CT-B03   | Exactly-at-bound depth is accepted (the guard is `≤`, not `<`)                                    | `tests::typed_input::ct_b03__exactly_at_depth_bound_accepted`          |
| CT-B04   | Invalid JSON syntax is rejected with `InvalidJson` (distinct from typed-input size violations)    | `tests::typed_input::ct_b04__invalid_json_rejected_distinct_from_size_bound` |
| CT-C01   | The PrismModel's `TypedCommitment` is `EmptyCommitment` (wiki ADR-048; no auxiliary cost surface) | `tests::typed_input::ct_c01__cost_model_is_empty_commitment`           |
| CT-P01   | `JsonValue::parse` returns Ok with non-empty tagged bytes for a valid input                       | `tests::typed_input::ct_p01__parse_returns_tagged_bytes`               |
| CT-P02   | `JsonValue::parse` rejects invalid JSON with the `validUtf8Json` violation IRI                    | `tests::typed_input::ct_p02__parse_rejects_invalid_json`               |

### CN — Network class — cross-validation against reference

Verified by **live HTTP calls to `mcp.uor.foundation`** at
`crates/uor-addr/tests/cross_validation.rs`. Gated behind `#[ignore]`;
runs only under `just cn` (CI optional).

| ID       | Invariant                                                                                  | Pinned by                                          |
|----------|--------------------------------------------------------------------------------------------|----------------------------------------------------|
| CN-RC01  | This crate's κ-label matches `mcp.uor.foundation/tools/encode_address` for the 12 fixtures | `tests::cross_validation::live_fixture_agreement` |
| CN-RC02  | This crate's κ-label matches the reference for 100 freshly-generated random JSON values     | `tests::cross_validation::live_random_agreement`  |

### CL — Formal class — Lean mechanised theorems

Verified by **`lake build`** under `uor-addr-lean/`. Theorems pin
universally quantified claims that no finite sample suite can establish
on its own. The Lean library depends only on the
[UOR-Framework Lean library](https://github.com/UOR-Foundation/UOR-Framework)
(no Mathlib).

| ID       | Theorem name                                                                       | File                                          | Statement                                                       |
|----------|------------------------------------------------------------------------------------|-----------------------------------------------|-----------------------------------------------------------------|
| CL-W01   | `UorAddr.AddressShape.address_label_width_is_seventy_one`                         | `UorAddr/AddressShape.lean`                  | `kappaLabel.size = 71` for every digest input                   |
| CL-W02   | `UorAddr.AddressShape.address_prefix_is_sha256_colon`                             | `UorAddr/AddressShape.lean`                  | `kappaLabel[0..7] = "sha256:".toUInt8Array`                     |
| CL-W03   | `UorAddr.AddressShape.address_hex_digits_are_lowercase`                           | `UorAddr/AddressShape.lean`                  | `∀ i ∈ [7, 71), kappaLabel[i] ∈ {'0'..'9', 'a'..'f'}`           |
| CL-H01   | `UorAddr.HexEncoding.hex_lower_injective`                                         | `UorAddr/HexEncoding.lean`                   | `hexLower` is injective on `[0, 16)`                            |
| CL-H02   | `UorAddr.HexEncoding.hex_byte_pair_roundtrip`                                     | `UorAddr/HexEncoding.lean`                   | `decodeNibble (hexLower n) = n` for `n < 16`                    |
| CL-K01   | `UorAddr.KappaDerivation.kappa_determined_by_digest`                              | `UorAddr/KappaDerivation.lean`               | Equal digests ⟹ equal κ-labels                                 |
| CL-K02   | `UorAddr.KappaDerivation.distinct_digests_yield_distinct_labels`                  | `UorAddr/KappaDerivation.lean`               | Unequal digests ⟹ unequal κ-labels (hex injectivity lifted)    |
| CL-A01   | `UorAddr.AlgebraicClosure.euler_char_eq_site_count`                               | `UorAddr/AlgebraicClosure.lean`              | `β_0 − β_1 + … = 71`                                            |
| CL-A02   | `UorAddr.AlgebraicClosure.free_rank_residual_zero`                                | `UorAddr/AlgebraicClosure.lean`              | After ψ_9 the FreeRank residual is 0                            |
| CL-N01   | `UorAddr.NfcIdempotence.nfc_is_idempotent`                                        | `UorAddr/NfcIdempotence.lean`                | `nfc (nfc s) = nfc s` (axiomatised — Unicode-spec lemma)        |
| CL-V01   | `UorAddr.VerbDiscipline.verb_arena_psi_residuals_only`                            | `UorAddr/VerbDiscipline.lean`                | The verb's term-arena coproduct contains only ψ-Term variants   |
| CL-CT01  | `UorAddr.TypedInput.case_tags_are_pairwise_distinct`                              | `UorAddr/TypedInput.lean`                    | Different JSON cases carry pairwise-distinct structural tag bytes |
| CL-CT02  | `UorAddr.TypedInput.depth_bound_is_strict`                                        | `UorAddr/TypedInput.lean`                    | Admissibility iff `depth ≤ MAX_JSON_DEPTH` (at-bound accepted; over-bound rejected) |
| CL-CT03  | `UorAddr.TypedInput.empty_commitment_is_the_cost_surface`                         | `UorAddr/TypedInput.lean`                    | The PrismModel's `C` is bound to `EmptyCommitment` (ADR-048)    |

### CB — Build class — `no_std` + `no_alloc` substrate

Verified by **out-of-tree `cargo build` invocations** under specific
target / feature combinations. The contract is that every published
crate (`uor-addr`, `uor-addr-c`, `uor-addr-wasm`) is `no_std` and
allocator-free by default; the `alloc` and `std` features layer
ergonomic convenience wrappers on top of the no_alloc core without
changing the κ-derivation byte path.

| ID       | Invariant                                                                                       | Pinned by                                                              |
|----------|-------------------------------------------------------------------------------------------------|------------------------------------------------------------------------|
| CB-N01   | `cargo build -p uor-addr --no-default-features --target thumbv7em-none-eabihf` builds clean    | `just embedded` (Cortex-M4 bare-metal no_std + no_alloc proof)         |
| CB-N02   | `cargo build -p uor-addr-c --no-default-features --target thumbv7em-none-eabihf` builds clean   | `just embedded` (C ABI staticlib on bare-metal)                        |
| CB-A01   | Runtime `[dependencies]` of `uor-addr` does not include `serde_json` or `unicode-normalization`| Cargo.toml inspection at the merge gate                                |
| CB-A02   | NFC normalization is provided by the in-crate `uor_addr::canonical::nfc` module                | UCD `NormalizationTest.txt` 19,074-vector × 5-identity suite passes    |
| CB-A03   | The `alloc` feature is purely additive (Vec-returning convenience APIs); no κ-label depends on it | `tests::byte_identity` passes with both `--no-default-features` and `--features alloc` |
| CB-A04   | The `std` feature is purely additive (re-exports + std-host conveniences); no κ-label depends on it | Cross-feature κ-label byte identity in `tests::all_realizations`     |

### CF — FFI class — language-binding byte identity

Verified by **building each FFI artifact and asserting κ-label byte
identity against the pure-Rust path**. The contract is that every FFI
binding produces the same 71-byte κ-label byte-for-byte as
`uor_addr::<realization>::address`, and that the witness-bearing
variants additionally support TC-05 replay across the FFI boundary.

| ID       | Invariant                                                                                       | Pinned by                                                              |
|----------|-------------------------------------------------------------------------------------------------|------------------------------------------------------------------------|
| CF-C01   | `uor-addr-c` exposes one `extern "C"` entry per realization (9 functions total)                | `crates/uor-addr-c/src/lib.rs` — one `extern "C"` definition per realization |
| CF-C02   | `uor-addr-c` builds cleanly on `thumbv7em-none-eabihf` (Cortex-M4 bare-metal, no allocator)    | `just embedded`                                                        |
| CF-C03   | `uor-addr-c` builds cleanly on hosted x86_64 (`libuor_addr_c.a` + `libuor_addr_c.so`)          | `just build-release`                                                   |
| CF-C04   | A C header is auto-generated at `crates/uor-addr-c/include/uor_addr.h` via `cbindgen`           | `build.rs` regenerates on `src/lib.rs` change                          |
| CF-C05   | `uor-addr-c` exposes a `UorAddrGrounded` opaque handle + `*_with_witness` constructors (9), `_kappa_label`, `_content_fingerprint`, `_verify`, `_free` | `crates/uor-addr-c/tests/grounded_round_trip.rs` |
| CF-W01   | `uor-addr-wasm` exposes one `*-address` function per realization via WIT (9 functions total)   | `crates/uor-addr-wasm/wit/uor-addr.wit`                                |
| CF-W02   | `uor-addr-wasm` builds cleanly on `wasm32-wasip2` (WASI Preview 2 + Component Model)            | `just wasm`                                                            |
| CF-W03   | The generated `.wasm` artifact is consumable from JS / Python / Go / .NET / Ruby via wasmtime  | `cargo build --target wasm32-wasip2 --release` emits `uor_addr_wasm.wasm` |
| CF-W04   | `uor-addr-wasm` exposes a `resource grounded` with `kappa-label` / `content-fingerprint` / `verify` methods + `*-address-with-witness` constructors (9) | `crates/uor-addr-wasm/wit/uor-addr.wit` + `bindings/npm/scripts/test.mjs` |

### CL-R — Replay class — TC-05 round-trip via `uor-prism-verify`

Verified by **runtime round-trip tests** at
`crates/uor-addr/tests/replay.rs` exercising the wiki TC-05
commitment: every `Grounded<AddressLabel>` the address pipeline emits
can be replayed by a downstream verifier through
`prism_verify::certify_from_trace` to produce a
`Certified<GroundingCertificate>` **without** re-invoking the canonical
hash axis on the original input. The replayed certificate's
`ContentFingerprint` is bit-identical to the source (QS-05 replay
equivalence). See [ARCHITECTURE.md](ARCHITECTURE.md)'s "Position at
SD3 — Verification" section for the architectural framing.

| ID       | Invariant                                                                                  | Pinned by                                                       |
|----------|--------------------------------------------------------------------------------------------|-----------------------------------------------------------------|
| CL-R00   | `prism_verify::certify_from_trace(Trace::empty())` returns `ReplayError::EmptyTrace`       | `tests::replay::cl_r00__verifier_facade_is_wired`               |
| CL-R01   | Single-input round-trip: replayed `ContentFingerprint` equals source                       | `tests::replay::cl_r01__grounded_address_label_round_trips_through_verifier` |
| CL-R02   | All 12 reference fixtures round-trip: replayed `ContentFingerprint` equals source per input | `tests::replay::cl_r02__every_reference_fixture_round_trips`    |
| CL-R-FFI-01 | Every `uor_addr_<realization>_with_witness` C ABI call returns a `UorAddrGrounded` whose `_kappa_label` matches the parallel flat-call κ-label byte-for-byte | `uor_addr_c::tests::grounded_round_trip::cl_r_ffi_01__json_witness_label_matches_flat_call` |
| CL-R-FFI-02 | `uor_addr_grounded_verify` returns the same κ-label byte-for-byte (QS-05 replay equivalence; SHA-256 not re-invoked) | `uor_addr_c::tests::grounded_round_trip::cl_r_ffi_02__verify_returns_same_label_as_mint` |
| CL-R-FFI-03 | `uor_addr_grounded_content_fingerprint` returns a 32-byte digest deterministically across calls | `uor_addr_c::tests::grounded_round_trip::cl_r_ffi_03__fingerprint_is_deterministic` |
| CL-R-FFI-04 | `uor_addr_grounded_free(NULL)` is a no-op; cross-realization witness round-trip holds for every shipped realization | `uor_addr_c::tests::grounded_round_trip::cl_r_ffi_04__free_on_null_is_noop` + `cross_realization__*` |
| CL-R-W01 | `bindings/npm` smoke test: every `kappa.*AddressWithWitness(...)` returns a `Grounded` whose `verify()` equals `kappaLabel()` byte-for-byte | `bindings/npm/scripts/test.mjs` |

## Contract evolution

- **Adding an ID.** Append; do not renumber. The PR description must
  cite the new ID and either a Lean theorem, a test path, or both.
- **Retiring an ID.** Mark `(retired @ vX.Y)` inline; do not delete.
  Tests pinning a retired ID may move to `#[ignore]` with a comment.
- **Tightening or loosening N/α.** Treat as a contract change: the PR
  must justify the new bound and update `tests/analysis.rs` consts.
- **All conformance changes pass through [VERIFICATION.md](VERIFICATION.md)
  §1's `just vv` gate.**

## CL-GGUF — closed-loop GGUF conformance

Synthetic fixtures under `crates/uor-addr/tests/fixtures/gguf/` with
committed `.kappa-label` files produced by `tools/canonical-gguf.py`.
`tests/gguf_byte_identity.rs` asserts the Rust κ-label equals the
attested label (CF-W*); `tests/gguf_conformance.rs` asserts format
invariants (label well-formedness, determinism, invariance under
metadata-KV / tensor reordering and tensor-data relayout, sensitivity to
weights and metadata, rejection of malformed input). Runs every CI build.

## CL-ONNX — closed-loop ONNX conformance

Synthetic fixtures under `crates/uor-addr/tests/fixtures/onnx/` with
committed `.kappa-label` files produced by `tools/canonical-onnx.py`.
`tests/onnx_byte_identity.rs` asserts byte-identity; `tests/onnx_conformance.rs`
asserts invariance under node reordering (topological canonicalization)
and `raw_data` vs typed-`float_data` storage, sensitivity to weights and
op types, admission of every known IR revision (`1..=ONNX_IR_VERSION_MAX`,
distinctly bound) with rejection of out-of-range / absent IR versions, and
rejection of unknown dtype / graph cycle.

## CN-GGUF / CN-ONNX — cross-network validation

Gated behind `UOR_ADDR_LIVE=1` (`tests/gguf_cross_validation.rs`,
`tests/onnx_cross_validation.rs`). Run the spec-side Python canonical-form
encoder against reference models and assert the live-computed κ-label
matches the Rust κ-label. The Python encoders are stdlib-only and are the
canonical-form spec attestation.

## CT-GGUF / CT-ONNX — cross-tool validation

Gated behind `UOR_ADDR_LIVE=1` (`tests/gguf_cross_tool.rs`,
`tests/onnx_cross_tool.rs`). POST fixture bytes to
`mcp.uor.foundation/tools/encode_{gguf,onnx}_address` and assert the
returned κ-label matches the Rust κ-label.

## CM-STREAM — streaming / bounded-carrier proof (ADR-060)

`tests/streaming.rs`, **every CI build**. Synthesizes GGUF v3 and ONNX
`ModelProto` buffers with a 64 MiB tensor-data section in-process and
proves the two properties that make arbitrarily large models tractable:

| ID        | Property pinned                                                                                   | Test                                                  |
|-----------|---------------------------------------------------------------------------------------------------|-------------------------------------------------------|
| CM-S01    | Canonical-skeleton (ψ-carrier) size is independent of tensor-data size (1 KiB vs 64 MiB ⇒ equal)  | `{gguf,onnx}_carrier_size_is_independent_of_*`        |
| CM-S02    | Flipping any byte in the (large) tensor-data region changes the κ-label (full-weight binding)     | `{gguf,onnx}_every_tensor_byte_binds`                 |
| CM-S03    | Large-model addressing is deterministic across calls                                              | `{gguf,onnx}_large_model_is_deterministic`            |

## CM-EXT — external real-model validation & verification

`tests/external_models.rs`, gated behind `UOR_ADDR_LIVE=1` (network +
~635 MB). Pins published models (a 531 MB Qwen2-0.5B-Instruct GGUF v3 and
two ONNX models at IR v3 / v6) by download URL, file SHA-256, and κ-label.
For each: verifies the cached bytes against the pinned file SHA-256,
asserts the Rust κ-label equals the pin, asserts the independent Python
reference encoder produces the same label, asserts the carrier (skeleton)
is < 5% of the model (the 531 MB → 28 KB streaming vector), and round-trips
the owned TC-05 witness. Re-run after bumping a pin; the recorded κ-labels
are the external-model attestation.
